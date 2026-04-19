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

ステージは `[workflow.<stage>]` 配下に定義します。skill が消費する組み込みステージ名は次の通り:

```
task_add         task_ready       task_start       task_complete
task_cancel      task_select      branch_set       branch_cleanup
branch_merge     pr_create        pr_update        plan
implement        contract_add     contract_edit    contract_delete
contract_dod_check                contract_dod_uncheck
contract_note_add
```

ユーザー独自のステージ名も許容されます。未知のステージ名は `senko config` の出力にそのまま残り、外部スクリプトから利用できます。skill は上記の組み込み名のみで発火します。なお `contract_*` ステージのうち、現行の同梱ワークフローが実際に emit するのは `contract_add` / `contract_note_add` / `contract_dod_check` の 3 つだけで、`contract_edit` / `contract_delete` / `contract_dod_uncheck` は組み込み名として認識はされますが、ユーザー定義のワークフロー拡張向けに予約されています。

各ステージ共通のキー:

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `instructions` | string[] | `[]` | このステージでのエージェント向けテキスト指示。 |
| `hooks` | map<string, HookDef> | `{}` | `[workflow.<stage>.hooks.<name>]` で定義する名前付き hook。各 hook の `when` / `mode` / `on_failure` で pre/post と sync/async を制御（下記 [Hooks](#hooks) 参照）。 |
| `metadata_fields` | field[] | `[]` | このステージで収集するメタデータフィールド。値はタスクの metadata にシャローマージされる。 |

ステージ固有のキー（未知のキーは pass-through として保持される）:

| ステージ | キー | 型 | 説明 |
|---------|------|------|------|
| `workflow.task_add` | `default_dod` | string[] | 新規タスクのデフォルト完了定義。 |
| `workflow.task_add` | `default_tags` | string[] | 新規タスクのデフォルトタグ。 |
| `workflow.task_add` | `default_priority` | string | 新規タスクのデフォルト優先度。 |
| `workflow.plan` | `required_sections` | string[] | 実装計画の必須セクション。 |

> **補足**: 旧スキーマの `pre_hooks` / `post_hooks` 配列は廃止されました。各 hook 定義に `when = "pre"` / `when = "post"` を指定して制御してください。

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

### `[server.relay]`

バイナリがリレーサーバー（`senko serve --proxy`）として起動しているときに適用。このセクション配下の hook はこの runtime でのみ発火する。

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `url` | string | `null` | 上流リレーサーバーURL。設定するとサーバーはリレーモードで動作し、このURLにリクエストを転送。 |
| `token` | string | `null` | 上流リレーサーバー認証用APIトークン。 |

タスクアクションの hook — `[server.relay.task_add.hooks.<name>]` / `[server.relay.task_ready.hooks.<name>]` / `[server.relay.task_start.hooks.<name>]` / `[server.relay.task_complete.hooks.<name>]` / `[server.relay.task_cancel.hooks.<name>]` / `[server.relay.task_select.hooks.<name>]`。詳細は [Hooks](#hooks)。

Contract アクションの hook — `[server.relay.contract_add.hooks.<name>]` / `[server.relay.contract_edit.hooks.<name>]` / `[server.relay.contract_delete.hooks.<name>]` / `[server.relay.contract_dod_check.hooks.<name>]` / `[server.relay.contract_dod_uncheck.hooks.<name>]` / `[server.relay.contract_note_add.hooks.<name>]`。詳細は [Hooks](#hooks)。

### `[server.remote]`

バイナリが直接（非リレー）サーバー（`senko serve`）として起動しているときに適用。このセクション配下の hook はこの runtime でのみ発火する。

タスクアクションの hook — `[server.remote.task_add.hooks.<name>]` / `[server.remote.task_ready.hooks.<name>]` / `[server.remote.task_start.hooks.<name>]` / `[server.remote.task_complete.hooks.<name>]` / `[server.remote.task_cancel.hooks.<name>]` / `[server.remote.task_select.hooks.<name>]`。詳細は [Hooks](#hooks)。

Contract アクションの hook — `[server.remote.contract_add.hooks.<name>]` / `[server.remote.contract_edit.hooks.<name>]` / `[server.remote.contract_delete.hooks.<name>]` / `[server.remote.contract_dod_check.hooks.<name>]` / `[server.remote.contract_dod_uncheck.hooks.<name>]` / `[server.remote.contract_note_add.hooks.<name>]`。詳細は [Hooks](#hooks)。

### `[cli]`

バイナリがローカル CLI（`senko serve` / `senko serve --proxy` 以外）として起動しているときに適用。このセクション配下の hook はこの runtime でのみ発火する。

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `browser` | bool | `true` | OIDCログイン時にブラウザを自動起動。 |

タスクアクションの hook — `[cli.task_add.hooks.<name>]` / `[cli.task_ready.hooks.<name>]` / `[cli.task_start.hooks.<name>]` / `[cli.task_complete.hooks.<name>]` / `[cli.task_cancel.hooks.<name>]` / `[cli.task_select.hooks.<name>]`。詳細は [Hooks](#hooks)。

Contract アクションの hook — `[cli.contract_add.hooks.<name>]` / `[cli.contract_edit.hooks.<name>]` / `[cli.contract_delete.hooks.<name>]` / `[cli.contract_dod_check.hooks.<name>]` / `[cli.contract_dod_uncheck.hooks.<name>]` / `[cli.contract_note_add.hooks.<name>]`。詳細は [Hooks](#hooks)。

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
| `hook_output` | string | `"file"` | hook の stdout/stderr の出力先: `"file"`, `"stdout"`, `"both"` のいずれか。 |

### `[project]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `name` | string | `null` | プロジェクト名。フックの環境変数や識別に使用。未設定時は自動検出。 |

### `[user]`

| キー | 型 | デフォルト | 説明 |
|------|------|----------|------|
| `name` | string | `null` | タスク割り当て用のユーザー名。未設定時は自動検出。 |

## Hooks

Hook は runtime ごと・アクションごとに名前付きで定義するシェルコマンドです。キー構造はすべての runtime で統一されています:

```
<runtime>.<aggregate>_<action>.hooks.<name>
```

### Runtime

| Runtime | 有効な場面 | セクションのプレフィックス |
|---------|-----------|----------------|
| `cli` | ローカル CLI バイナリ（`senko serve` / `senko serve --proxy` 以外） | `[cli.<action>.hooks.<name>]` |
| `server.relay` | リレーサーバー（`senko serve --proxy`） | `[server.relay.<action>.hooks.<name>]` |
| `server.remote` | 直接サーバー（`senko serve`） | `[server.remote.<action>.hooks.<name>]` |
| `workflow` | Claude Code skill が消費するワークフローステージ | `[workflow.<stage>.hooks.<name>]` |

### アクション

`cli` / `server.relay` / `server.remote` の各 runtime は、各 aggregate ごとに **固定** のアクションを公開します。タスクアクション:

| アクション | 発火するタイミング |
|-----------|-------------------|
| `task_add` | `senko task add` がタスクを作成したとき |
| `task_ready` | `senko task ready` が draft → todo に遷移させたとき |
| `task_start` | `senko task start` または `senko task next` がタスクを開始したとき |
| `task_complete` | `senko task complete` がタスクを完了させたとき |
| `task_cancel` | `senko task cancel` がタスクをキャンセルしたとき |
| `task_select` | `senko task next` がタスクを選定した / 該当タスクを見つけられなかったとき（`on_result` で絞り込む） |

Contract アクション:

| アクション | 発火するタイミング |
|-----------|-------------------|
| `contract_add` | `senko contract add` が contract を作成したとき |
| `contract_edit` | `senko contract edit` が contract を更新したとき |
| `contract_delete` | `senko contract delete` が contract を削除したとき |
| `contract_dod_check` | `senko contract dod check` が DoD 項目をチェックしたとき |
| `contract_dod_uncheck` | `senko contract dod uncheck` が DoD 項目をアンチェックしたとき |
| `contract_note_add` | `senko contract note add` が note を追加したとき |

`workflow` runtime は **任意** のステージ名を受け付けます。skill が発火対象とする組み込みステージ名については [ワークフローステージ](#ワークフローステージ) を参照。

### `HookDef` フィールド

`<runtime>.<aggregate>_<action>.hooks.<name>` 配下に定義する hook は `HookDef` 型です:

| フィールド | 型 | デフォルト | 説明 |
|-----------|------|----------|------|
| `command` | string | _(必須)_ | `sh -c` 経由で実行するシェルコマンド。イベント envelope が JSON として stdin に渡される。 |
| `when` | `"pre"` / `"post"` | `"post"` | 状態遷移の前後どちらで発火するか。 |
| `mode` | `"sync"` / `"async"` | `"async"` | `sync` は完了を待つ、`async` は spawn して detach する。 |
| `on_failure` | `"abort"` / `"warn"` / `"ignore"` | `"abort"` | 非 0 終了したときの挙動。**`abort` は `sync`+`pre` の hook でのみ有効** — `sync`+`post` や `async` では warn と同等（ログのみ）。 |
| `enabled` | bool | `true` | `false` で定義を残したまま一時的に無効化。 |
| `env_vars` | `EnvVarSpec[]` | `[]` | 検証 / 注入する環境変数のスペック（下記）。 |
| `on_result` | `"selected"` / `"none"` / `"any"` | `"any"` | `task_select` hook でのみ有効。`selected` = 選定成功時のみ / `none` = 該当タスクなし時のみ / `any` = どちらでも発火。`task_select` 以外のタスクアクションと、すべての `contract_*` アクションでは無視される。 |
| `prompt` | string | `null` | `workflow.<stage>.hooks.<name>` 配下でのみ有効で、skill がその stage でエージェント向けの指示として emit する。`cli` / `server.relay` / `server.remote` runtime では無視される。 |

### `EnvVarSpec` フィールド

`env_vars` の各要素は `EnvVarSpec` 型です:

| フィールド | 型 | デフォルト | 説明 |
|-----------|------|----------|------|
| `name` | string | _(必須)_ | 環境変数名。 |
| `required` | bool | `true` | `true` で、発火時に未設定かつ `default` も無い場合、hook は **スキップ** され warn ログが出る。 |
| `default` | string | `null` | 設定されていれば、未設定時にこの値が注入される。 |
| `description` | string | `null` | 設定ファイル読者向けの備考。 |

### 発火時の挙動

- **Runtime フィルタ**: 起動中のプロセスに一致する runtime 配下の hook のみが発火。他 runtime 配下の hook は無視され、プロセス起動時に一度だけ警告ログ（`hooks configured under runtime sections that do not match the active runtime; they will not fire`）が出るため、設定ミスに気付きやすい。
- **`when` フィルタ**: `when = "pre"` は状態遷移の前、`when = "post"` は後に発火。workflow stage とタスクアクションのどちらも pre/post 双方で hook を発火させる。
- **`mode`**: `sync` はコマンド終了までブロック、`async` はプロセスを起動して即座に return する。
- **`on_failure = "abort"` のセマンティクス**: 失敗した hook が `sync` かつ `when = "pre"` のときに限り、状態遷移が `DomainError::HookAborted` で中止される。それ以外の組み合わせでは `abort` は warn ログに縮退する。fire-and-forget で明示的にログだけ残したい場合は `warn` / `ignore` を使うこと。

### Load-time バリデーション

起動時に config をスキャンして以下の警告を出します（該当 hook は load 自体は成功しますが、`abort` / `on_result` 指定は実質無視されます）:

- `pre` + `async` + `on_failure = "abort"` — async hook は abort できないため、`abort` は実質 `warn` と同等。
- `task_select` 以外の hook に `on_result` が指定されている — `on_result` は `task_select` のみで意味を持ち、他では無視される。

### 例: タスク完了時の通知

```toml
[cli.task_complete.hooks.notify]
command = "curl -X POST -d @- $WEBHOOK_URL"
mode = "async"

[[cli.task_complete.hooks.notify.env_vars]]
name = "WEBHOOK_URL"
required = true
```

### 例: `on_result` を使った task_select

旧 `on_no_eligible_task` の代替 — `senko task next` が該当タスクを見つけられなかった場合にのみ発火:

```toml
[cli.task_select.hooks.prompt_for_add]
command = "echo 'no eligible task — consider adding one'"
on_result = "none"
```

選定成功時に発火:

```toml
[cli.task_select.hooks.log_selection]
command = "logger -t senko 'task selected'"
on_result = "selected"
```

### 例: sync+pre+abort によるゲート

`sync`+`pre`+`abort` な hook は非 0 終了時に状態遷移を中止できるため、完了時のローカルチェックのゲートに使えます:

```toml
[workflow.branch_merge.hooks.mise_check]
command = "mise check"
when = "pre"
mode = "sync"
on_failure = "abort"
```

### 例: サーバー側の hook

```toml
[server.remote.task_ready.hooks.metrics]
command = "emit-metric task_ready"
mode = "async"
```

### 例: contract hook

contract hook は `senko contract <verb>` コマンドで発火する。stdin に渡される envelope には `task` ではなく `contract` オブジェクト（`senko contract get` と同じスキーマ）が含まれる。hook 形状はタスク hook と同一で、`when` / `mode` / `on_failure` / `env_vars` がそのまま使え、`sync`+`pre`+`abort` で操作を中止できる点も共通。

```toml
# DoD チェックのサーバーサイド監査ログ
[server.remote.contract_dod_check.hooks.audit]
command = "jq -r '.event.contract.id' | xargs -I{} logger -t senko 'contract {} dod check'"
mode = "async"

# 契約 note 追加前に skill が挿入するプロンプト
[workflow.contract_note_add.hooks.review_before_note]
command = "true"
prompt = "新しい note を追加する前に最新の notes を読み直し、同じ観察がすでに記録されていればスキップすること。"
when = "pre"
```

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
| `SENKO_CLI_REMOTE_URL` | `cli.remote.url` | リモートサーバーURL |
| `SENKO_CLI_REMOTE_TOKEN` | `cli.remote.token` | APIトークン |

### サーバー

| 変数 | 対応する設定キー | 説明 |
|------|-----------------|------|
| `SENKO_SERVER_RELAY_URL` | `server.relay.url` | 上流リレーサーバーURL |
| `SENKO_SERVER_RELAY_TOKEN` | `server.relay.token` | 上流リレーサーバー認証用APIトークン |
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

### Hooks

Hook 定義は環境変数では設定 **できません**。`.senko/config.toml` の runtime 別セクション（`[cli.<action>.hooks.<name>]`、`[server.relay.<action>.hooks.<name>]`、`[server.remote.<action>.hooks.<name>]`、`[workflow.<stage>.hooks.<name>]`）で定義してください。このセクション構造は `task_*` / `contract_*` どちらのアクションでも共通です。

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

## 破壊的変更

hooks 設定スキーマが全面刷新されました。旧スキーマは後方互換なしに **読み込まれません** — 旧 `[hooks]` セクションおよび関連する環境変数は互換シム無しで削除されました。（旧 scalar / array 形式の略記は load 時に拒否され、入れ子の旧 `[hooks]` テーブルは警告は出ますが hook としては発火しません。）

| 旧 | 新 | 備考 |
|-----|-----|------|
| `[hooks]`（top-level） | `[cli.<action>.hooks.<name>]` / `[server.relay.<action>.hooks.<name>]` / `[server.remote.<action>.hooks.<name>]` | どの runtime で発火するかを section で選択 |
| `[hooks].enabled` マスタースイッチ | _（廃止）_ | 個別 hook の `enabled = false` で無効化する |
| `on_task_added` | `task_add` | |
| `on_task_ready` | `task_ready` | |
| `on_task_started` | `task_start` | |
| `on_task_completed` | `task_complete` | |
| `on_task_canceled` | `task_cancel` | |
| `on_no_eligible_task` | `task_select` + `on_result = "none"` | 単一の `task_select` アクションに統合 |
| `requires_env = [...]` | `env_vars = [{ name = "...", required = true }]` | 型付きスペック + デフォルト値対応 |
| `[workflow.<stage>] pre_hooks = [...]` / `post_hooks = [...]` | `[workflow.<stage>.hooks.<name>]` + `when = "pre" \| "post"` | hook 形状を統一 |
| `SENKO_HOOKS_ENABLED` 環境変数 | _（廃止）_ | hooks のマスタースイッチは廃止 |
| `SENKO_HOOK_ON_TASK_*` 環境変数 | _（廃止）_ | hook は `config.toml` でのみ定義する |
| `SENKO_HOOK_ON_NO_ELIGIBLE_TASK` 環境変数 | _（廃止）_ | `[cli.task_select.hooks.*] on_result = "none"` を使う |
| 旧 workflow ステージ名（`add`, `start`, `plan`, `complete`, `branch`, `merge`, `pr`） | `task_add`, `task_start`, `plan`, `task_complete`, `branch_set`, `branch_merge`, `pr_create` | skill は新名でのみ発火する |
| _（なし）_ | `contract_add` / `contract_edit` / `contract_delete` / `contract_dod_check` / `contract_dod_uncheck` / `contract_note_add` | contract aggregate 向けの **新規** アクション。旧スキーマが無いため migration 不要で、同一の runtime セクションに `task_*` と並べて定義できる |

### 他の TOML エイリアス（維持）

| 非推奨 | 現在 | 備考 |
|--------|------|------|
| `workflow.completion_mode` | `workflow.merge_via` | serdeエイリアスで受け付け |
| `merge_then_complete`（値） | `direct` | `merge_via` の値として受け付け |
| `pr_then_complete`（値） | `pr` | `merge_via` の値として受け付け |

### 他の環境変数エイリアス（維持）

| 非推奨 | 現在 | 備考 |
|--------|------|------|
| `SENKO_COMPLETION_MODE` | `SENKO_MERGE_VIA` | 非推奨警告を表示 |

## 関連ドキュメント

- [認証モード別セットアップガイド](AUTH_SETUP.ja.md) — 認証モードとセットアップ
- [CLIリファレンス](CLI.ja.md) — 全コマンドの詳細
- [AWSデプロイガイド](AWS_DEPLOYMENT.md) — Trusted Headersデプロイ
- [README](README.ja.md) — プロジェクト概要
