# チームのサーバに CLI から接続する

チームが既に `senko serve` を立てている前提で、手元の CLI からそこに接続する手順です。
(サーバを立てる側は [getting-started/server.md](server.md))

## 前提

- 手元に `senko` バイナリがインストール済み ([local.md](local.md) の step 1 を参照)
- チームのサーバ URL と、自分用の API トークン or OIDC アカウント

## 接続方式の選び方

| 方式 | 向いているケース | セットアップ |
|---|---|---|
| **API キー** | CI/CD、bot、固定発行したトークンを使う個人 | サーバ管理者から API キーを貰って env or config に保存 |
| **OIDC (OAuth PKCE)** | チーム運用で SSO 配下に居る | `senko auth login` でブラウザが開き、デバイストークンが keychain に保存される |

どちらにすべきか分からない場合はチームのサーバ管理者に確認してください。

## 方式 A: API キー

サーバ管理者から貰った API キーを環境変数に設定するだけ:

```bash
export SENKO_CLI_REMOTE_URL="https://senko.example.com"
export SENKO_CLI_REMOTE_TOKEN="sk_abc123..."
```

もしくは `.senko/config.toml` に:

```toml
[cli.remote]
url = "https://senko.example.com"
token = "sk_abc123..."
```

> トークンをリポジトリにコミットしたくない場合は `.senko/config.local.toml` (git 管理外) に書くか、環境変数を使ってください。

動作確認:

```bash
senko task list
```

## 方式 B: OIDC ログイン

サーバ側で OIDC が有効化されていれば、以下でブラウザ経由のログインができます:

```bash
senko auth login
```

- ブラウザが起動し、IdP (Google / Cognito / Keycloak 等) で認証
- 認証後、CLI が OS の keychain にデバイストークンを保存
- 以降 `senko` コマンドはこのトークンを自動で使う

セッション確認:

```bash
senko auth status     # 現在ログイン中のユーザ・device 名
senko auth sessions   # 発行済みセッション一覧
senko auth logout     # セッション失効 + keychain から削除
```

config には URL だけ書いておけば OK です:

```toml
[cli.remote]
url = "https://senko.example.com"
```

## 接続先の確認

```bash
senko config
```

の出力で `cli.remote.url` が期待するサーバを指しているか確認してください。

## オフライン/ローカル DB に一時的に戻したい

環境変数 `SENKO_CLI_REMOTE_URL` を解除すれば、このコマンド実行中だけローカル SQLite (`.senko/senko.db`) を触ります:

```bash
SENKO_CLI_REMOTE_URL= senko task list
```

## トラブルシューティング

| 症状 | 確認ポイント |
|---|---|
| 401 Unauthorized | トークンが失効していないか。`senko auth status` で確認 |
| 403 Forbidden | プロジェクトメンバーに追加されていない。サーバ管理者に依頼 |
| connection refused | URL のスキーム / ポートが正しいか。サーバが起動しているか |
| `senko auth login` でブラウザが開かない | `[cli] browser = false` になっていないか / ヘッドレス環境では手動で URL を開く |

## 次に読むもの

- 認証モードの詳細 → [guides/server-remote/auth-api-key.md](../guides/server-remote/auth-api-key.md) / [auth-oidc.md](../guides/server-remote/auth-oidc.md)
- 手元の workflow stage を調整する → [guides/cli/workflow-stages.md](../guides/cli/workflow-stages.md)
