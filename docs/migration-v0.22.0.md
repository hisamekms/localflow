# senko config.toml マイグレーションガイド (v0.21.0 → v0.22.0)

このドキュメントは、senko v0.21.0 の `config.toml` を v0.22.0 形式に変換するための手順書です。

## AI への指示

ユーザーの既存 `config.toml` を読み取り、以下のルールに該当するセクションのみ変換してください。存在しないセクションは無視してください。コメントアウトされた行もルールに従って変換してください。

---

## ルール 1: `[auth]` → `[server.auth.api_key]`

`[auth]` セクション全体を `[server.auth.api_key]` に移動し、フィールド名を変更する。

**Before:**
```toml
[auth]
enabled = true
master_api_key = "secret-key"
master_api_key_arn = "arn:aws:secretsmanager:us-east-1:123:secret:key"
```

**After:**
```toml
[server.auth.api_key]
master_key = "secret-key"
master_key_arn = "arn:aws:secretsmanager:us-east-1:123:secret:key"
```

**変換ルール:**
- `[auth]` → `[server.auth.api_key]`
- `master_api_key` → `master_key`
- `master_api_key_arn` → `master_key_arn`
- `enabled` → **削除**（設定の有無で自動判定されるようになった）

---

## ルール 2: `[storage]` → `[backend.sqlite]`

`[storage]` セクションを `[backend.sqlite]` にリネームする。フィールド名は変更なし。

**Before:**
```toml
[storage]
db_path = "/custom/path/to/data.db"
```

**After:**
```toml
[backend.sqlite]
db_path = "/custom/path/to/data.db"
```

---

## ルール 3: `[backend]` の api_url / api_key → `[cli.remote]`

`[backend]` セクション内の `api_url` と `api_key` を `[cli.remote]` に移動し、フィールド名を変更する。

**Before:**
```toml
[backend]
api_url = "http://127.0.0.1:3142"
api_key = "my-api-key"
```

**After:**
```toml
[cli.remote]
url = "http://127.0.0.1:3142"
token = "my-api-key"
```

**変換ルール:**
- `[backend]` → `[cli.remote]`
- `api_url` → `url`
- `api_key` → `token`

**注意:** `[backend]` に `dynamodb` や `postgres` のサブセクションがある場合、それらは `[backend.dynamodb]`、`[backend.postgres]` としてそのまま残す。`api_url` と `api_key` のみを `[cli.remote]` に移動する。

---

## ルール 4: `[skill.start]` → `[workflow.start]`

`[skill.start]` セクションを `[workflow.start]` に移動する。メタデータフィールドの `source = "fixed"` を `source = "value"` に変更する。

**Before:**
```toml
[skill.start]
[[skill.start.metadata_fields]]
key = "team"
source = "fixed"
value = "backend"

[[skill.start.metadata_fields]]
key = "assigned_by"
source = "env"
env_var = "USER"
default = "unknown"

[[skill.start.metadata_fields]]
key = "estimate"
source = "prompt"
prompt = "Estimated time for this task?"
```

**After:**
```toml
[workflow.start]
[[workflow.start.metadata_fields]]
key = "team"
source = "value"
value = "backend"

[[workflow.start.metadata_fields]]
key = "assigned_by"
source = "env"
env_var = "USER"
default = "unknown"

[[workflow.start.metadata_fields]]
key = "estimate"
source = "prompt"
prompt = "Estimated time for this task?"
```

**変換ルール:**
- `[skill.start]` → `[workflow.start]`
- `[[skill.start.metadata_fields]]` → `[[workflow.start.metadata_fields]]`
- `source = "fixed"` → `source = "value"`
- `source = "env"` / `source = "prompt"` → 変更なし

---

## ルール 5: `[[workflow.events]]` → 名前付きワークフローセクション

`[[workflow.events]]` 配列を、イベントポイントごとの名前付きセクションに分解する。

**Before:**
```toml
[[workflow.events]]
point = "pre_merge"
type = "command"
command = "cargo test --all"

[[workflow.events]]
point = "post_pr"
type = "prompt"
content = "Add reviewers to the PR"

[[workflow.events]]
point = "pre_implement"
type = "command"
command = "cargo fmt --check"
```

**After:**
```toml
[workflow.merge]
pre_hooks = ["cargo test --all"]

[workflow.pr]
post_hooks = [{ prompt = "Add reviewers to the PR", on_failure = "warn" }]

[workflow.implement]
pre_hooks = [{ command = "cargo fmt --check", on_failure = "abort" }]
```

**変換ルール:**

1. `point` の値を `pre_` / `post_` プレフィックスとセクション名に分解する
   - `pre_<section>` → `[workflow.<section>]` の `pre_hooks` に追加
   - `post_<section>` → `[workflow.<section>]` の `post_hooks` に追加

2. `type` に応じて hook の形式を決定する
   - `type = "command"` → 文字列 `"<command の値>"` または `{ command = "...", on_failure = "abort" }`
   - `type = "prompt"` → `{ prompt = "<content の値>", on_failure = "warn" }`

3. 同じセクション・同じプレフィックスの events は 1 つの配列にまとめる

**利用可能なセクション名:** `add`, `start`, `branch`, `plan`, `implement`, `merge`, `pr`, `complete`, `branch_cleanup`

---

## 環境変数の変更

config.toml と合わせて、環境変数やデプロイ設定も更新が必要です。

### リネームされた環境変数

| v0.21.0 | v0.22.0 |
|---------|---------|
| `SENKO_MASTER_API_KEY` | `SENKO_AUTH_API_KEY_MASTER_KEY` |
| `SENKO_MASTER_API_KEY_ARN` | `SENKO_AUTH_API_KEY_MASTER_KEY_ARN` |
| `SENKO_API_URL` | `SENKO_SERVER_URL` |
| `SENKO_API_KEY` | `SENKO_TOKEN` |

### 削除された環境変数

| v0.21.0 | 備考 |
|---------|------|
| `SENKO_AUTH_ENABLED` | 設定の有無で自動判定されるため不要 |

### 変更なし（そのまま使える）

`SENKO_DB_PATH`, `SENKO_HOST`, `SENKO_PORT`, `SENKO_USER`, `SENKO_PROJECT`, `SENKO_MERGE_VIA`, `SENKO_AUTO_MERGE`, `SENKO_BRANCH_MODE`, `SENKO_MERGE_STRATEGY`, `SENKO_HOOKS_ENABLED`, `SENKO_LOG_DIR`, `SENKO_LOG_LEVEL`, `SENKO_LOG_FORMAT`, `SENKO_COMPLETION_MODE`（非推奨エイリアス）

---

## 変更不要なセクション

以下のセクションは v0.21.0 と v0.22.0 で互換性があり、変更不要です。

- `[hooks]` — CLIライフサイクルフック（`on_task_added` 等）
- `[log]` — ログ設定
- `[project]` — プロジェクト名
- `[user]` — ユーザー名
- `[web]` — Web UI のホスト・ポート
- `[workflow]` の `merge_via`, `auto_merge`, `branch_mode`, `merge_strategy` — 値もそのまま使える
  - `completion_mode`（`merge_via` の旧名）も引き続き動作する
  - `merge_then_complete` / `pr_then_complete`（旧値）も引き続き動作する

---

## 新機能（任意で追加可能）

v0.22.0 で追加された設定項目です。マイグレーション時に必要に応じて追加してください。

### ワークフロー設定

```toml
[workflow]
branch_template = "senko/{id}-{slug}"  # ブランチ名テンプレート

[workflow.add]
default_dod = ["Write unit tests", "Update documentation"]  # デフォルトの完了定義
default_tags = ["backend"]                                   # デフォルトタグ
default_priority = "p2"                                      # デフォルト優先度

[workflow.plan]
required_sections = ["Overview", "Acceptance Criteria"]  # 計画の必須セクション
```

### ワークフローイベントの instructions

各ワークフローセクションに `instructions` を追加できます（AI への指示）。

```toml
[workflow.implement]
instructions = ["Follow project coding standards", "Add tests for new features"]
pre_hooks = [{ command = "cargo fmt --check", on_failure = "abort" }]
post_hooks = ["cargo test --all"]
```

### メタデータフィールドの新機能

```toml
[[workflow.start.metadata_fields]]
key = "git_branch"
source = "command"                          # 新しいソースタイプ: コマンド実行
command = "git rev-parse --abbrev-ref HEAD"

[[workflow.complete.metadata_fields]]       # complete でもメタデータ設定可能に
key = "completed_at"
source = "command"
command = "date -u +%Y-%m-%dT%H:%M:%SZ"
required = true                             # 新しいフィールド: 必須フラグ
```

### サーバー設定

```toml
[server]
host = "0.0.0.0"
port = 3142

[server.auth.oidc]
issuer_url = "https://auth.example.com"
client_id = "senko-app"
username_claim = "sub"
scopes = ["openid", "profile"]
required_claims = { role = "admin" }

[server.auth.oidc.session]
ttl = "24h"
inactive_ttl = "1h"
max_per_user = 5

[server.auth.oidc.cli]
callback_ports = ["9999"]
browser = true
```
