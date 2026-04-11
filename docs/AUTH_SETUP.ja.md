# 認証モード別セットアップガイド

[English](AUTH_SETUP.md) | [READMEに戻る](README.ja.md)

senkoは3つの認証モードをサポートしています。用途に応じて適切なモードを選択してください。

| モード | ユースケース | 必要なインフラ | 認証方法 |
|--------|-------------|---------------|---------|
| Local | 個人開発、単一ユーザー | なし | 認証なし |
| Remote + API key | CI/CD、サービス間連携 | senkoサーバー | APIキー |
| Remote + OIDC | チーム利用、エンタープライズSSO | senkoサーバー + OIDCプロバイダー | OAuth PKCE + APIキー |

## Local モード

最もシンプルな構成です。設定不要で、初回実行時にSQLiteデータベースが自動作成されます。

### 最小構成

設定ファイルなしですぐに利用を開始できます:

```bash
senko add --title "最初のタスク"
senko list
```

初回実行時に `.senko/senko.db`（SQLiteデータベース）が自動作成されます。プロジェクトとユーザーはデフォルト値（id=1、name="default"）で自動的に用意されます。

### カスタム設定（オプション）

プロジェクト名やユーザー名を変更したい場合は `.senko/config.toml` を作成します:

```toml
[project]
name = "my-project"

[user]
name = "alice"
```

テンプレートから生成することもできます:

```bash
senko config --init
```

### データの保存先

| ファイル | 説明 |
|---------|------|
| `.senko/senko.db` | SQLiteデータベース |
| `.senko/config.toml` | 設定ファイル（オプション） |

> **注意**: `.senko/` を `.gitignore` に追加して、ローカルデータをコミットしないようにしてください。

## Remote + API key モード

サーバーでsenkoを稼働させ、APIキーでクライアントから接続します。CI/CDパイプラインやサービス間連携に適しています。

### 前提条件

- senkoサーバーを稼働させるマシン
- クライアントからサーバーへのネットワーク接続

### 管理者の手順

#### 1. マスターAPIキーの生成

マスターAPIキーはシステムのブートストラップ（初期ユーザー作成・APIキー発行）に使用します:

```bash
openssl rand -base64 32
```

#### 2. サーバー設定

サーバー側の `.senko/config.toml`:

```toml
[auth]
enabled = true
master_api_key = "生成したマスターAPIキー"

[auth.token]
ttl = "30d"              # トークンの有効期限（省略時: 無期限）
inactive_ttl = "7d"      # 非アクティブ時の有効期限（省略時: 無期限）
max_per_user = 10        # ユーザーあたりの最大セッション数（省略時: 無制限）
```

環境変数で設定する場合:

```bash
export SENKO_AUTH_ENABLED=true
export SENKO_AUTH_MASTER_API_KEY="生成したマスターAPIキー"
```

#### 3. サーバー起動

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### 4. ユーザー作成

マスターAPIキーを使って初期セットアップを行います:

```bash
# ユーザー作成
curl -s -X POST http://localhost:3142/api/v1/users \
  -H "Authorization: Bearer $SENKO_AUTH_MASTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "display_name": "Alice Smith"}' | jq .
```

CLIからも実行できます:

```bash
senko user create --username alice --display-name "Alice Smith"
```

#### 5. プロジェクト作成

```bash
senko project create --name my-project
```

#### 6. メンバー追加

```bash
# ユーザーID 2 を member ロールで追加（ロール: owner, member, viewer）
senko members add --user-id 2 --role member
```

#### 7. ユーザーAPIキーの発行

```bash
# ユーザーID 2 のAPIキーを発行（1はユーザーIDに置き換え）
curl -s -X POST http://localhost:3142/api/v1/users/2/api-keys \
  -H "Authorization: Bearer $SENKO_AUTH_MASTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name": "alice-default"}' | jq .
```

レスポンスに含まれる `token` フィールドがAPIキーです。**このキーは一度しか表示されません**。

### 利用者の手順

#### 1. クライアント設定

`~/.config/senko/config.toml` またはプロジェクトの `.senko/config.toml`:

```toml
[backend]
api_url = "http://senko-server:3142"
api_key = "管理者から受け取ったAPIキー"
```

環境変数で設定する場合:

```bash
export SENKO_API_URL="http://senko-server:3142"
export SENKO_BACKEND_API_KEY="管理者から受け取ったAPIキー"
```

#### 2. 接続確認

```bash
senko --output text list
```

### CI/CD での利用例

```yaml
# GitHub Actions の例
env:
  SENKO_API_URL: ${{ secrets.SENKO_API_URL }}
  SENKO_BACKEND_API_KEY: ${{ secrets.SENKO_API_KEY }}

steps:
  - name: タスク一覧を取得
    run: senko list --status todo
```

## Remote + OIDC モード

OIDCプロバイダー（Amazon Cognito、Auth0、Okta等）と連携し、ブラウザベースのログインフローでユーザーを認証します。チーム利用やエンタープライズSSO環境に適しています。

### 前提条件

- senkoサーバーを稼働させるマシン
- OIDCプロバイダー（Amazon Cognito、Auth0、Okta等）
- クライアントからサーバーおよびOIDCプロバイダーへのネットワーク接続

### 管理者の手順

#### 1. OIDCプロバイダーの設定

OIDCプロバイダーでアプリケーションを登録します。以下はAmazon Cognitoの例です:

- **アプリケーションタイプ**: Public client（PKCE対応）
- **許可されたコールバックURL**: `http://127.0.0.1:8400/callback`（CLIログイン用）
- **スコープ**: `openid`, `profile`（必要に応じて `email` も追加）
- **認可フロー**: Authorization code grant with PKCE

設定後、以下の情報を控えてください:
- **Issuer URL**: `https://cognito-idp.{region}.amazonaws.com/{user-pool-id}`
- **Client ID**: アプリクライアントID

#### 2. サーバー設定

サーバー側の `.senko/config.toml`:

```toml
[auth]
enabled = true

[auth.oidc]
issuer_url = "https://cognito-idp.ap-northeast-1.amazonaws.com/ap-northeast-1_XXXXXXXXX"
client_id = "1a2b3c4d5e6f7g8h9i0j"
scopes = ["openid", "profile"]    # デフォルト: ["openid", "profile"]

[auth.token]
ttl = "24h"              # トークンの有効期限
inactive_ttl = "7d"      # 非アクティブ時の有効期限
max_per_user = 10        # ユーザーあたりの最大セッション数
```

環境変数で設定する場合:

```bash
export SENKO_AUTH_ENABLED=true
export SENKO_OIDC_ISSUER_URL="https://cognito-idp.ap-northeast-1.amazonaws.com/ap-northeast-1_XXXXXXXXX"
export SENKO_OIDC_CLIENT_ID="1a2b3c4d5e6f7g8h9i0j"
```

> **補足**: OIDCモードとマスターAPIキーを同時に設定することもできます。この場合、JWTとAPIキーの両方で認証が可能になり、人間はOIDC、サービスはAPIキーという使い分けができます。

#### 3. サーバー起動

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### 4. プロジェクト作成

OIDCモードでは、初回ログイン時にユーザーがJWTクレーム（`sub`, `name`, `email`）から自動作成されます。プロジェクトの作成とメンバー追加は管理者が行います:

```bash
senko project create --name my-project
senko members add --user-id 2 --role member
```

### 利用者の手順

#### 1. クライアント設定

`~/.config/senko/config.toml` またはプロジェクトの `.senko/config.toml`:

```toml
[backend]
api_url = "http://senko-server:3142"

[auth.oidc]
issuer_url = "https://cognito-idp.ap-northeast-1.amazonaws.com/ap-northeast-1_XXXXXXXXX"
client_id = "1a2b3c4d5e6f7g8h9i0j"
```

#### 2. ログイン

```bash
senko auth login
```

ブラウザが自動的に開き、OIDCプロバイダーのログイン画面が表示されます。認証が完了するとCLIに戻り、APIキーがOSのキーチェーンに自動保存されます。

デバイス名を指定する場合:

```bash
senko auth login --device-name "my-laptop"
```

#### 3. ログイン状態の確認

```bash
senko auth status
```

#### 4. 利用開始

```bash
senko --output text list
```

### コンテナ連携

コンテナ環境など、キーチェーンが利用できない環境では `senko auth token` でトークンを取得して環境変数に設定します:

```bash
# ホストマシンでトークンを取得
export SENKO_BACKEND_API_KEY=$(senko auth token)

# コンテナに渡す
docker run --rm \
  -e SENKO_API_URL="http://senko-server:3142" \
  -e SENKO_BACKEND_API_KEY="$SENKO_BACKEND_API_KEY" \
  senko list
```

### セッション管理

```bash
# アクティブなセッション一覧
senko auth sessions

# 特定のセッションを無効化
senko auth revoke <session-id>

# 全セッションを無効化
senko auth revoke --all

# ログアウト（現在のセッションを無効化し、キーチェーンからトークンを削除）
senko auth logout
```

## config.toml リファレンス

### 認証関連の設定キー

| セクション | キー | 型 | デフォルト | 説明 | Local | Remote+APIキー | Remote+OIDC |
|-----------|------|------|----------|------|:-----:|:-------------:|:-----------:|
| `[auth]` | `enabled` | bool | `false` | 認証を有効化 | - | 必須 | 必須 |
| `[auth]` | `master_api_key` | string | - | マスターAPIキー | - | 必須 | 任意 |
| `[auth]` | `master_api_key_arn` | string | - | AWS Secrets Manager ARN | - | 任意 | 任意 |
| `[auth.oidc]` | `issuer_url` | string | - | OIDC発行者URL | - | - | 必須 |
| `[auth.oidc]` | `client_id` | string | - | OIDCクライアントID | - | - | 必須 |
| `[auth.oidc]` | `scopes` | array | `["openid", "profile"]` | OIDCスコープ | - | - | 任意 |
| `[auth.oidc.cli]` | `callback_port` | integer | 自動割当 | コールバックポート | - | - | 任意 |
| `[auth.oidc.cli]` | `browser` | bool | `true` | ブラウザ自動起動 | - | - | 任意 |
| `[auth.token]` | `ttl` | string | 無期限 | トークン有効期限（例: `"24h"`, `"30d"`） | - | 任意 | 任意 |
| `[auth.token]` | `inactive_ttl` | string | 無期限 | 非アクティブ時の有効期限 | - | 任意 | 任意 |
| `[auth.token]` | `max_per_user` | integer | 無制限 | ユーザーあたりの最大セッション数 | - | 任意 | 任意 |

### 接続関連の設定キー

| セクション | キー | 型 | デフォルト | 説明 |
|-----------|------|------|----------|------|
| `[backend]` | `api_url` | string | - | APIサーバーURL（設定するとHTTPバックエンドを使用） |
| `[backend]` | `api_key` | string | - | APIキー（クライアント側） |
| `[web]` | `host` | string | `"127.0.0.1"` | サーバーのバインドアドレス |
| `[web]` | `port` | integer | `3142`（serve）/ `3141`（web） | サーバーのリッスンポート |
| `[project]` | `name` | string | `"default"` | プロジェクト名 |
| `[user]` | `name` | string | `"default"` | ユーザー名 |
| `[storage]` | `db_path` | string | `.senko/senko.db` | SQLiteデータベースパス |

### 認証関連の環境変数

| 変数 | 対応する設定キー | 説明 |
|------|-----------------|------|
| `SENKO_AUTH_ENABLED` | `auth.enabled` | 認証の有効/無効 |
| `SENKO_AUTH_MASTER_API_KEY` | `auth.master_api_key` | マスターAPIキー |
| `SENKO_AUTH_MASTER_API_KEY_ARN` | `auth.master_api_key_arn` | マスターAPIキーのAWS ARN |
| `SENKO_OIDC_ISSUER_URL` | `auth.oidc.issuer_url` | OIDC発行者URL |
| `SENKO_OIDC_CLIENT_ID` | `auth.oidc.client_id` | OIDCクライアントID |
| `SENKO_AUTH_TOKEN_TTL` | `auth.token.ttl` | トークン有効期限 |
| `SENKO_AUTH_TOKEN_INACTIVE_TTL` | `auth.token.inactive_ttl` | 非アクティブ時の有効期限 |
| `SENKO_AUTH_TOKEN_MAX_PER_USER` | `auth.token.max_per_user` | 最大セッション数 |
| `SENKO_API_URL` | `backend.api_url` | APIサーバーURL |
| `SENKO_BACKEND_API_KEY` | `backend.api_key` | APIキー（クライアント側） |
| `SENKO_HOST` | `web.host` | サーバーバインドアドレス |
| `SENKO_PORT` | `web.port` | サーバーポート |
| `SENKO_DB_PATH` | `storage.db_path` | SQLiteデータベースパス |
| `SENKO_PROJECT` | - | 操作対象のプロジェクト名 |
| `SENKO_USER` | - | 操作ユーザー名 |

### 設定の優先順位

設定値は以下の優先順位で適用されます（上が優先）:

1. CLIフラグ（`--config`, `--project-root` 等）
2. 環境変数（`SENKO_*`）
3. プロジェクト設定（`.senko/config.toml`）
4. ユーザー設定（`~/.config/senko/config.toml`）
5. ビルトインデフォルト値

## 関連ドキュメント

- [CLIリファレンス](CLI.ja.md) — 全コマンドの詳細
- [README](README.ja.md) — プロジェクト概要とクイックスタート
