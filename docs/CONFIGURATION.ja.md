# 設定リファレンス

[English](CONFIGURATION.md) | [READMEに戻る](README.ja.md)

## 設定ファイルの場所

| ファイル | 説明 |
|---------|------|
| `.senko/config.toml` | プロジェクト設定（gitにコミット） |
| `.senko/config.local.toml` | ローカル上書き（git-ignored、ユーザー個別） |
| `~/.config/senko/config.toml` | ユーザーレベル設定（全プロジェクトに適用） |

コメント付きテンプレートを生成:

```bash
senko config --init
```

## 設定の優先順位

設定値は以下の優先順位で解決されます（上が優先）:

1. **CLIフラグ**（`--config <path>`, `--port`, `--host` 等）
2. **環境変数**（`SENKO_*`）
3. **ローカル設定**（`.senko/config.local.toml`）
4. **プロジェクト設定**（`.senko/config.toml`）
5. **ユーザー設定**（`~/.config/senko/config.toml`）
6. **ビルトインデフォルト値**

## TOML設定セクション

### `[workflow]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `merge_via` | string | `"direct"` | ブランチのマージ方法: `"direct"`（gitマージ）または `"pr"`（PR URLとマージ状態チェックが必要）。 |
| `auto_merge` | bool | `true` | 完了時にブランチを自動マージ。`merge_via = "direct"` の場合のみ有効。 |
| `branch_mode` | string | `"worktree"` | タスクブランチの作成方法: `"worktree"`（git worktree）または `"branch"`（通常のブランチ）。 |
| `merge_strategy` | string | `"rebase"` | gitマージ戦略: `"rebase"` または `"squash"`。 |
| `branch_template` | string | `null` | ブランチ名テンプレート（例: `"task/{{id}}-{{slug}}"`）。 |

### ワークフローステージ

ステージ: `workflow.add`, `workflow.start`, `workflow.branch`, `workflow.plan`, `workflow.implement`, `workflow.merge`, `workflow.pr`, `workflow.complete`, `workflow.branch_cleanup`

各ステージ共通:

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `instructions` | string[] | `[]` | このステージでのエージェント向けテキスト指示。 |
| `pre_hooks` | hook[] | `[]` | ステージ前に実行するフック。文字列（シェルコマンド）または `{command, prompt, on_failure}`。 |
| `post_hooks` | hook[] | `[]` | ステージ後に実行するフック。`pre_hooks` と同じ形式。 |

ステージ固有のキー:

| ステージ | キー | 型 | 説明 |
|---------|------|------|------|
| `workflow.add` | `default_dod` | string[] | 新規タスクのデフォルト完了定義。 |
| `workflow.add` | `default_tags` | string[] | 新規タスクのデフォルトタグ。 |
| `workflow.add` | `default_priority` | string | 新規タスクのデフォルト優先度。 |
| `workflow.start` | `metadata_fields` | field[] | タスク開始時に収集するメタデータフィールド。 |
| `workflow.plan` | `required_sections` | string[] | 実装計画の必須セクション。 |
| `workflow.complete` | `metadata_fields` | field[] | タスク完了時に収集するメタデータフィールド。 |

### `[backend.sqlite]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `db_path` | string | 自動 | SQLiteデータベースファイルのパス。デフォルト: `$XDG_DATA_HOME/senko/projects/<hash>/data.db` |

### `[backend.postgres]`（`postgres` feature が必要）

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `url` | string | `null` | PostgreSQL接続URL（例: `postgres://user:pass@host/db`）。`--postgres-url` でも指定可。 |
| `url_arn` | string | `null` | 接続URL用AWS Secrets Manager ARN（`aws-secrets` feature が必要）。 |
| `rds_secrets_arn` | string | `null` | RDS JSONシークレット用AWS Secrets Manager ARN（`username`, `password`, `host` を含む必要あり。`port`, `dbname` は任意）。 |
| `sslrootcert` | string | `null` | TLS接続用SSLルート証明書のパス。 |
| `max_connections` | u32 | `null` | データベースプールの最大接続数。 |

### `[server]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `host` | string | `"127.0.0.1"` | `senko serve` のバインドアドレス。 |
| `port` | u16 | `3142` | `senko serve` のポート。 |

### `[server.auth.api_key]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `master_key` | string | `null` | 認証用マスターAPIキーの直接値。 |
| `master_key_arn` | string | `null` | マスターAPIキーのAWS Secrets Manager ARN（`aws-secrets` feature が必要）。 |

### `[server.auth.oidc]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `issuer_url` | string | `null` | JWT検証用OIDC発行者URL。 |
| `client_id` | string | `null` | PKCE認証用OIDCクライアントID。 |
| `scopes` | string[] | `["openid", "profile"]` | 要求するOIDCスコープ。 |
| `username_claim` | string | `null` | ユーザー名として使用するJWTクレーム。 |
| `required_claims` | map | `{}` | 必須JWTクレーム（一致する必要があるキーバリューペア）。 |
| `callback_ports` | string[] | `[]` | CLIログイン時のOIDCコールバック用ローカルポート。個別ポートと範囲をサポート（例: `["8400", "9000-9010"]`）。 |

### `[server.auth.oidc.cli]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `browser` | bool | `true` | OIDCログイン時にブラウザを自動起動。 |

### `[server.auth.oidc.session]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `ttl` | string | `null` | セッション有効期限（例: `"24h"`, `"30d"`）。`null` = 無期限。 |
| `inactive_ttl` | string | `null` | セッション非アクティブ時の有効期限（例: `"7d"`）。`null` = 無期限。 |
| `max_per_user` | u32 | `null` | ユーザーあたりの最大セッション数。`null` = 無制限。 |

### `[server.auth.trusted_headers]`

リバースプロキシ（API Gateway等）が検証済みのIDヘッダーを注入する環境で使用。詳細は[AWSデプロイガイド](AWS_DEPLOYMENT.md)を参照。

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `subject_header` | string | `null` | ユーザーのサブジェクト識別子を含むヘッダー。このモードの有効化に必須。 |
| `name_header` | string | `null` | ユーザーの表示名を含むヘッダー。 |
| `display_name_header` | string | `null` | 表示名のフォールバックヘッダー（`name_header` が存在しない場合に使用）。 |
| `email_header` | string | `null` | ユーザーのメールアドレスを含むヘッダー。 |
| `groups_header` | string | `null` | ユーザーのグループを含むヘッダー。 |
| `scope_header` | string | `null` | OAuthスコープを含むヘッダー。 |
| `oidc_issuer_url` | string | `null` | `GET /auth/config` で返すOIDC発行者URL（CLIログインの検出用）。 |
| `oidc_client_id` | string | `null` | `GET /auth/config` で返すOIDCクライアントID（CLIログインの検出用）。 |

> **補足**: 認証モード（APIキー、OIDC、trusted headers）は同時に1つのみ有効にできます。

### `[cli.remote]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `url` | string | `null` | リモートサーバーURL。設定するとCLIはこのサーバーにコマンドを転送。 |
| `token` | string | `null` | リモートサーバー認証用APIトークン。 |

### `[web]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `host` | string | `"127.0.0.1"` | `senko web` のバインドアドレス。 |
| `port` | u16 | `null`（自動） | `senko web` のポート。デフォルト: `3141`。 |

### `[log]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `dir` | string | 自動 | ログファイルのディレクトリ。デフォルト: `$XDG_STATE_HOME/senko` |
| `level` | string | `"info"` | 最小ログレベル: `trace`, `debug`, `info`, `warn`, `error`。 |
| `format` | string | `"json"` | ログ出力形式: `"json"` または `"pretty"`。 |

### `[project]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `name` | string | `null` | プロジェクト名。フックの環境変数や識別に使用。未設定時は自動検出。 |

### `[user]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `name` | string | `null` | タスク割り当て用のユーザー名。未設定時は自動検出。 |

### `[hooks]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `enabled` | bool | `true` | このプロセス（CLI）でフックを発火するか。APIサーバーはこの設定に関わらず常にフックを発火。 |

フックイベントは各イベントキー配下の名前付きエントリとして設定:

```toml
[hooks.on_task_completed.webhook]
command = "curl -X POST https://example.com/webhook"
enabled = true
requires_env = ["WEBHOOK_URL"]
```

| イベント | トリガー |
|---------|---------|
| `on_task_added` | 新しいタスクが作成された。 |
| `on_task_ready` | タスクが `todo` ステータスに遷移した。 |
| `on_task_started` | タスクが `in_progress` に遷移した。 |
| `on_task_completed` | タスクが完了した。 |
| `on_task_canceled` | タスクがキャンセルされた。 |
| `on_no_eligible_task` | `senko next` で実行可能なタスクが見つからなかった。 |

各フックエントリのフィールド:

| フィールド | 型 | デフォルト | 説明 |
|-----------|------|----------|------|
| `command` | string | _（必須）_ | 実行するシェルコマンド（`sh -c` 経由）。 |
| `enabled` | bool | `true` | `false` で一時的に無効化。 |
| `requires_env` | string[] | `[]` | 指定した環境変数がすべて設定されている場合のみ実行。 |

## 環境変数

### ワークフロー

| 変数 | 対応する設定キー | 値 |
|------|-----------------|------|
| `SENKO_MERGE_VIA` | `workflow.merge_via` | `direct`, `pr` |
| `SENKO_AUTO_MERGE` | `workflow.auto_merge` | `true`/`1`/`yes`, `false`/`0`/`no` |
| `SENKO_BRANCH_MODE` | `workflow.branch_mode` | `worktree`, `branch` |
| `SENKO_MERGE_STRATEGY` | `workflow.merge_strategy` | `rebase`, `squash` |

### 接続

| 変数 | 対応する設定キー | 説明 |
|------|-----------------|------|
| `SENKO_SERVER_URL` | `cli.remote.url` | リモートサーバーURL |
| `SENKO_TOKEN` | `cli.remote.token` | APIトークン |

### サーバー

| 変数 | 対応する設定キー | 説明 |
|------|-----------------|------|
| `SENKO_SERVER_HOST` | `server.host` | `senko serve` 専用バインドアドレス |
| `SENKO_SERVER_PORT` | `server.port` | `senko serve` 専用ポート |
| `SENKO_HOST` | `web.host` + `server.host` | `senko web` と `senko serve` の両方のバインドアドレス |
| `SENKO_PORT` | `web.port` + `server.port` | `senko web` と `senko serve` の両方のポート |

> `SENKO_SERVER_HOST`/`SENKO_SERVER_PORT` は `senko serve` のみに適用されます。`SENKO_HOST`/`SENKO_PORT` は `senko serve` と `senko web` の両方に適用されます。

### 認証

| 変数 | 対応する設定キー |
|------|-----------------|
| `SENKO_AUTH_API_KEY_MASTER_KEY` | `server.auth.api_key.master_key` |
| `SENKO_AUTH_API_KEY_MASTER_KEY_ARN` | `server.auth.api_key.master_key_arn` |
| `SENKO_OIDC_ISSUER_URL` | `server.auth.oidc.issuer_url` |
| `SENKO_OIDC_CLIENT_ID` | `server.auth.oidc.client_id` |
| `SENKO_OIDC_USERNAME_CLAIM` | `server.auth.oidc.username_claim` |
| `SENKO_OIDC_CALLBACK_PORTS` | `server.auth.oidc.callback_ports`（カンマ区切り） |
| `SENKO_AUTH_OIDC_SESSION_TTL` | `server.auth.oidc.session.ttl` |
| `SENKO_AUTH_OIDC_SESSION_INACTIVE_TTL` | `server.auth.oidc.session.inactive_ttl` |
| `SENKO_AUTH_OIDC_SESSION_MAX_PER_USER` | `server.auth.oidc.session.max_per_user`（u32としてパース） |

### Trusted Headers

| 変数 | 対応する設定キー |
|------|-----------------|
| `SENKO_AUTH_TRUSTED_HEADERS_SUBJECT_HEADER` | `server.auth.trusted_headers.subject_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_NAME_HEADER` | `server.auth.trusted_headers.name_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_EMAIL_HEADER` | `server.auth.trusted_headers.email_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_GROUPS_HEADER` | `server.auth.trusted_headers.groups_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_SCOPE_HEADER` | `server.auth.trusted_headers.scope_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_OIDC_ISSUER_URL` | `server.auth.trusted_headers.oidc_issuer_url` |
| `SENKO_AUTH_TRUSTED_HEADERS_OIDC_CLIENT_ID` | `server.auth.trusted_headers.oidc_client_id` |

### バックエンド

| 変数 | 対応する設定キー |
|------|-----------------|
| `SENKO_DB_PATH` | `backend.sqlite.db_path` |
| `SENKO_POSTGRES_URL` | `backend.postgres.url` |
| `SENKO_POSTGRES_URL_ARN` | `backend.postgres.url_arn` |
| `SENKO_POSTGRES_RDS_SECRETS_ARN` | `backend.postgres.rds_secrets_arn` |
| `SENKO_POSTGRES_SSLROOTCERT` | `backend.postgres.sslrootcert` |
| `SENKO_POSTGRES_MAX_CONNECTIONS` | `backend.postgres.max_connections`（u32としてパース） |

### フック

| 変数 | 対応する設定キー |
|------|-----------------|
| `SENKO_HOOKS_ENABLED` | `hooks.enabled` |
| `SENKO_HOOK_ON_TASK_ADDED` | `hooks.on_task_added`（シェルコマンド） |
| `SENKO_HOOK_ON_TASK_READY` | `hooks.on_task_ready`（シェルコマンド） |
| `SENKO_HOOK_ON_TASK_STARTED` | `hooks.on_task_started`（シェルコマンド） |
| `SENKO_HOOK_ON_TASK_COMPLETED` | `hooks.on_task_completed`（シェルコマンド） |
| `SENKO_HOOK_ON_TASK_CANCELED` | `hooks.on_task_canceled`（シェルコマンド） |
| `SENKO_HOOK_ON_NO_ELIGIBLE_TASK` | `hooks.on_no_eligible_task`（シェルコマンド） |

### その他

| 変数 | 対応する設定キー | 説明 |
|------|-----------------|------|
| `SENKO_USER` | `user.name` | ユーザー名 |
| `SENKO_PROJECT` | `project.name` | プロジェクト名 |
| `SENKO_LOG_DIR` | `log.dir` | ログディレクトリ |
| `SENKO_LOG_LEVEL` | `log.level` | ログレベル |
| `SENKO_LOG_FORMAT` | `log.format` | ログ形式（`json` または `pretty`） |
| `SENKO_CONFIG` | _（CLIレベル）_ | 設定ファイルのパス |
| `SENKO_PROJECT_ROOT` | _（CLIレベル）_ | プロジェクトルートディレクトリ |

## 後方互換性

以下の非推奨名称が後方互換性のためにサポートされています:

### TOMLキー

| 非推奨 | 現在 | 備考 |
|--------|------|------|
| `workflow.completion_mode` | `workflow.merge_via` | serdeエイリアスで受け付け |
| `merge_then_complete`（値） | `direct` | `merge_via` の値として受け付け |
| `pr_then_complete`（値） | `pr` | `merge_via` の値として受け付け |

### 環境変数

| 非推奨 | 現在 | 備考 |
|--------|------|------|
| `SENKO_COMPLETION_MODE` | `SENKO_MERGE_VIA` | 非推奨警告を表示 |

## 関連ドキュメント

- [認証モード別セットアップガイド](AUTH_SETUP.ja.md) — 認証モードとセットアップ
- [CLIリファレンス](CLI.ja.md) — 全コマンドの詳細
- [AWSデプロイガイド](AWS_DEPLOYMENT.md) — Trusted Headersデプロイ
- [README](README.ja.md) — プロジェクト概要
