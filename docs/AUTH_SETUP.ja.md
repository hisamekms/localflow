# 認証モード別セットアップガイド

[English](AUTH_SETUP.md) | [READMEに戻る](README.ja.md)

senkoは4つの認証モードをサポートしています。用途に応じて適切なモードを選択してください。

| モード | ユースケース | 必要なインフラ | 認証方法 |
|--------|-------------|---------------|---------|
| Local | 個人開発、単一ユーザー | なし | 認証なし |
| Remote + API key | CI/CD、サービス間連携 | senkoサーバー | APIキー |
| Remote + OIDC | チーム利用、エンタープライズSSO | senkoサーバー + OIDCプロバイダー | OAuth PKCE + APIキー |
| Relay/Proxy | AIサンドボックス、マルチテナント中継 | senko中継サーバー + senkoリモートサーバー | トークン注入またはパススルー |

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
[server.auth.api_key]
master_key = "生成したマスターAPIキー"

[server.auth.oidc.session]
ttl = "30d"              # トークンの有効期限（省略時: 無期限）
inactive_ttl = "7d"      # 非アクティブ時の有効期限（省略時: 無期限）
max_per_user = 10        # ユーザーあたりの最大セッション数（省略時: 無制限）
```

環境変数で設定する場合:

```bash
export SENKO_AUTH_API_KEY_MASTER_KEY="生成したマスターAPIキー"
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
  -H "Authorization: Bearer $SENKO_AUTH_API_KEY_MASTER_KEY" \
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
  -H "Authorization: Bearer $SENKO_AUTH_API_KEY_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name": "alice-default"}' | jq .
```

レスポンスに含まれる `token` フィールドがAPIキーです。**このキーは一度しか表示されません**。

### 利用者の手順

#### 1. クライアント設定

`~/.config/senko/config.toml` またはプロジェクトの `.senko/config.toml`:

```toml
[cli.remote]
url = "http://senko-server:3142"
token = "管理者から受け取ったAPIキー"
```

環境変数で設定する場合:

```bash
export SENKO_SERVER_URL="http://senko-server:3142"
export SENKO_TOKEN="管理者から受け取ったAPIキー"
```

#### 2. 接続確認

```bash
senko --output text list
```

### CI/CD での利用例

```yaml
# GitHub Actions の例
env:
  SENKO_SERVER_URL: ${{ secrets.SENKO_SERVER_URL }}
  SENKO_TOKEN: ${{ secrets.SENKO_TOKEN }}

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
[server.auth.oidc]
issuer_url = "https://cognito-idp.ap-northeast-1.amazonaws.com/ap-northeast-1_XXXXXXXXX"
client_id = "1a2b3c4d5e6f7g8h9i0j"
scopes = ["openid", "profile"]    # デフォルト: ["openid", "profile"]

# 特定のJWTクレームを要求（オプション）
[server.auth.oidc.required_claims]
"custom:tenant" = "my-company"

[server.auth.oidc.session]
ttl = "24h"              # トークンの有効期限
inactive_ttl = "7d"      # 非アクティブ時の有効期限
max_per_user = 10        # ユーザーあたりの最大セッション数
```

環境変数で設定する場合:

```bash
export SENKO_OIDC_ISSUER_URL="https://cognito-idp.ap-northeast-1.amazonaws.com/ap-northeast-1_XXXXXXXXX"
export SENKO_OIDC_CLIENT_ID="1a2b3c4d5e6f7g8h9i0j"
```

> **補足**: 認証モード（APIキー、OIDC、trusted headers）は同時に1つのみ有効にできます。人間はOIDC、サービスはAPIキーという使い分けをするには、OIDCを主モードとして設定し、OIDCログインフローで発行されるAPIキーをサービスに配布してください。

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
[cli.remote]
url = "http://senko-server:3142"
```

CLIはサーバーの `GET /auth/config` エンドポイントからOIDC設定（issuer URL、client ID、スコープ）を自動取得するため、クライアント側でOIDC設定を行う必要はありません。

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
export SENKO_TOKEN=$(senko auth token)

# コンテナに渡す
docker run --rm \
  -e SENKO_SERVER_URL="http://senko-server:3142" \
  -e SENKO_TOKEN="$SENKO_TOKEN" \
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

## Relay/Proxy モード

senkoインスタンスをリレー（中継）サーバーとして稼働させ、リモートのsenkoサーバーにリクエストを転送します。リレーではローカルでの認証を行わず、すべての認証はアップストリームのリモートサーバーに委任されます。クライアントが認証情報を保持すべきでないAIサンドボックス環境や、複数クライアントからのリクエストを集約するマルチテナント構成に適しています。

> **補足**: リモートサーバーは事前に [Remote + API key](#remote--api-key-モード) または [Remote + OIDC](#remote--oidc-モード) モードでセットアップしておく必要があります。

### アーキテクチャ

```
CLI ──→ 中継サーバー (senko serve) ──→ リモートサーバー
         [cli.remote.url 設定済み]       [認証有効]
```

`cli.remote.url` が設定された状態で `senko serve` を実行すると、インスタンスはリレーモードで動作します:

- ローカルでの認証はスキップされます（アップストリームサーバーに委任）
- リレーはクライアントの `Authorization` ヘッダーから Bearer トークンを取得します
- リクエストは以下のいずれかのトークンでリモートサーバーに転送されます:
  - リレー自身の `cli.remote.token`（設定されている場合）— 優先
  - クライアントの元のトークン（パススルー）

### パターンA: トークン注入（AIサンドボックス）

リレーが自身のトークンを転送リクエストに付与します。クライアントは認証情報を必要としません。

#### 中継サーバーの設定

中継サーバーの `.senko/config.toml`:

```toml
[cli.remote]
url = "http://remote-senko:3142"
token = "リモートサーバーで発行されたリレー用APIキー"
```

環境変数で設定する場合:

```bash
export SENKO_SERVER_URL="http://remote-senko:3142"
export SENKO_TOKEN="リモートサーバーで発行されたリレー用APIキー"
```

リレーを起動:

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### 利用者の手順

クライアントはリレーのURLのみ設定します（トークン不要）:

```toml
[cli.remote]
url = "http://relay-server:3142"
```

環境変数で設定する場合:

```bash
export SENKO_SERVER_URL="http://relay-server:3142"
```

#### 接続確認

```bash
senko --output text list
```

### パターンB: トークンパススルー

リレーはクライアントの元のトークンをそのままリモートサーバーに転送します。各クライアントが個別に認証を行います。

#### 中継サーバーの設定

中継サーバーの `.senko/config.toml`（`token` なし — `url` のみ）:

```toml
[cli.remote]
url = "http://remote-senko:3142"
```

環境変数で設定する場合:

```bash
export SENKO_SERVER_URL="http://remote-senko:3142"
```

リレーを起動:

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### 利用者の手順

クライアントはリレーのURLと自身のトークン（APIキーまたはOIDC発行トークン）を設定します:

```toml
[cli.remote]
url = "http://relay-server:3142"
token = "クライアント自身のAPIキー"
```

環境変数で設定する場合:

```bash
export SENKO_SERVER_URL="http://relay-server:3142"
export SENKO_TOKEN="クライアント自身のAPIキー"
```

#### リモートサーバー

リモートサーバーはクライアントのトークンを直接検証します。既存の [Remote + API key](#remote--api-key-モード) または [Remote + OIDC](#remote--oidc-モード) のセットアップ以外に特別な設定は不要です。

### まとめ

| | パターンA（トークン注入） | パターンB（トークンパススルー） |
|-|--------------------------|-------------------------------|
| **ユースケース** | AIサンドボックス、共有サービスアカウント | リレー経由のユーザー個別認証 |
| **クライアントトークン** | 不要 | 必要（APIキーまたはOIDCトークン） |
| **中継サーバー設定** | `cli.remote.url` + `cli.remote.token` | `cli.remote.url` のみ |
| **リモートが検証するもの** | リレーのトークン | クライアントの元のトークン |

## config.toml リファレンス

### 認証関連の設定キー

| セクション | キー | 型 | デフォルト | 説明 | Local | Remote+APIキー | Remote+OIDC | Relay |
|-----------|------|------|----------|------|:-----:|:-------------:|:-----------:|:-----:|
| `[server.auth.api_key]` | `master_key` | string | - | マスターAPIキー | - | 必須 | 任意 | - |
| `[server.auth.api_key]` | `master_key_arn` | string | - | AWS Secrets Manager ARN | - | 任意 | 任意 | - |
| `[server.auth.oidc]` | `issuer_url` | string | - | OIDC発行者URL | - | - | 必須 | - |
| `[server.auth.oidc]` | `client_id` | string | - | OIDCクライアントID | - | - | 必須 | - |
| `[server.auth.oidc]` | `scopes` | array | `["openid", "profile"]` | OIDCスコープ | - | - | 任意 | - |
| `[server.auth.oidc]` | `required_claims` | table | - | 必須JWTクレーム（キーバリューペア） | - | - | 任意 | - |
| `[server.auth.oidc]` | `callback_ports` | array | `[]` (空) | CLIログイン用コールバックポート | - | - | 任意 | - |
| `[server.auth.oidc.cli]` | `browser` | bool | `true` | ブラウザ自動起動 | - | - | 任意 | - |
| `[server.auth.oidc.session]` | `ttl` | string | 無期限 | セッション有効期限（例: `"24h"`, `"30d"`） | - | 任意 | 任意 | - |
| `[server.auth.oidc.session]` | `inactive_ttl` | string | 無期限 | 非アクティブ時の有効期限 | - | 任意 | 任意 | - |
| `[server.auth.oidc.session]` | `max_per_user` | integer | 無制限 | ユーザーあたりの最大セッション数 | - | 任意 | 任意 | - |

> **補足**: `[server.auth.*]` の設定が存在すると認証が暗黙的に有効になります。明示的な `auth.enabled` キーはありません。

### 接続関連の設定キー

| セクション | キー | 型 | デフォルト | 説明 |
|-----------|------|------|----------|------|
| `[cli.remote]` | `url` | string | - | APIサーバーURL（設定するとHTTPバックエンドを使用） |
| `[cli.remote]` | `token` | string | - | APIトークン（クライアント側） |
| `[server]` | `host` | string | `"127.0.0.1"` | サーバーのバインドアドレス |
| `[server]` | `port` | integer | `3142` | サーバーのリッスンポート |
| `[web]` | `host` | string | `"127.0.0.1"` | Web UIバインドアドレス |
| `[web]` | `port` | integer | `3141` | Web UIリッスンポート |
| `[project]` | `name` | string | `"default"` | プロジェクト名 |
| `[user]` | `name` | string | `"default"` | ユーザー名 |
| `[backend.sqlite]` | `db_path` | string | `.senko/senko.db` | SQLiteデータベースパス |

> **補足**: リレーモードでは、中継サーバーの `[cli.remote]` セクションがアップストリームのリモートサーバーを指定します。`cli.remote.url` を設定した状態で `senko serve` を実行するとリレーモードが有効になります。`cli.remote.token`（設定されている場合）はクライアントのトークンの代わりに転送リクエストに付与されます。

### APIエンドポイント

| エンドポイント | メソッド | 認証 | 説明 |
|--------------|---------|------|------|
| `/auth/config` | GET | 不要 | OIDC設定（issuer URL、client ID、スコープ）を返す |
| `/auth/token` | POST | JWT | OIDC JWTをAPIトークンに交換する |
| `/auth/me` | GET | 必要 | 現在のユーザー情報とセッション詳細 |
| `/auth/sessions` | GET | 必要 | アクティブなセッション一覧 |
| `/auth/sessions` | DELETE | 必要 | 全セッションを無効化 |
| `/auth/sessions/{id}` | DELETE | 必要 | 特定のセッションを無効化 |
| `/users` | POST | マスターキー | 新規ユーザーを作成 |

### 環境変数

| 変数 | 対応する設定キー | 説明 |
|------|-----------------|------|
| `SENKO_AUTH_API_KEY_MASTER_KEY` | `server.auth.api_key.master_key` | マスターAPIキー |
| `SENKO_AUTH_API_KEY_MASTER_KEY_ARN` | `server.auth.api_key.master_key_arn` | マスターAPIキーのAWS ARN |
| `SENKO_OIDC_ISSUER_URL` | `server.auth.oidc.issuer_url` | OIDC発行者URL |
| `SENKO_OIDC_CLIENT_ID` | `server.auth.oidc.client_id` | OIDCクライアントID |
| `SENKO_OIDC_USERNAME_CLAIM` | `server.auth.oidc.username_claim` | OIDCユーザー名クレーム |
| `SENKO_OIDC_CALLBACK_PORTS` | `server.auth.oidc.callback_ports` | OIDCコールバックポート（カンマ区切り） |
| `SENKO_AUTH_OIDC_SESSION_TTL` | `server.auth.oidc.session.ttl` | セッション有効期限 |
| `SENKO_AUTH_OIDC_SESSION_INACTIVE_TTL` | `server.auth.oidc.session.inactive_ttl` | 非アクティブ時の有効期限 |
| `SENKO_AUTH_OIDC_SESSION_MAX_PER_USER` | `server.auth.oidc.session.max_per_user` | 最大セッション数 |
| `SENKO_SERVER_URL` | `cli.remote.url` | APIサーバーURL |
| `SENKO_TOKEN` | `cli.remote.token` | APIトークン（クライアント側） |
| `SENKO_SERVER_HOST` | `server.host` | サーバーバインドアドレス |
| `SENKO_SERVER_PORT` | `server.port` | サーバーポート |
| `SENKO_HOST` | `web.host` + `server.host` | Web UI / サーバーバインドアドレス |
| `SENKO_PORT` | `web.port` + `server.port` | Web UI / サーバーポート |
| `SENKO_DB_PATH` | `backend.sqlite.db_path` | SQLiteデータベースパス |
| `SENKO_PROJECT` | `project.name` | 操作対象のプロジェクト名 |
| `SENKO_USER` | `user.name` | 操作ユーザー名 |

### 設定の優先順位

設定値は以下の優先順位で適用されます（上が優先）:

1. CLIフラグ（`--config`, `--project-root` 等）
2. 環境変数（`SENKO_*`）
3. ローカル設定（`.senko/config.local.toml` — git-ignored、ユーザー個別の上書き）
4. プロジェクト設定（`.senko/config.toml`）
5. ユーザー設定（`~/.config/senko/config.toml`）
6. ビルトインデフォルト値

## 関連ドキュメント

- [設定リファレンス](CONFIGURATION.ja.md) — 全設定キーと環境変数
- [CLIリファレンス](CLI.ja.md) — 全コマンドの詳細
- [README](README.ja.md) — プロジェクト概要とクイックスタート
