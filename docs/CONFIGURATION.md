# Configuration Reference

[日本語](CONFIGURATION.ja.md) | [Back to README](../README.md)

## Config File Locations

| File | Description |
|------|-------------|
| `.senko/config.toml` | Project configuration (committed to git) |
| `.senko/config.local.toml` | Local overrides (git-ignored, per-user) |
| `~/.config/senko/config.toml` | User-level configuration (applies to all projects) |

Generate a commented template with:

```bash
senko config --init
```

## Configuration Priority

Values are resolved in the following order (highest priority first):

1. **CLI flags** (`--config <path>`, `--port`, `--host`, etc.)
2. **Environment variables** (`SENKO_*`)
3. **Local config** (`.senko/config.local.toml`)
4. **Project config** (`.senko/config.toml`)
5. **User config** (`~/.config/senko/config.toml`)
6. **Built-in defaults**

## TOML Configuration Sections

### `[workflow]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `merge_via` | string | `"direct"` | How the branch is merged: `"direct"` (git merge) or `"pr"` (requires PR URL and merge check). |
| `auto_merge` | bool | `true` | Auto-merge branch on completion. Only applies when `merge_via = "direct"`. |
| `branch_mode` | string | `"worktree"` | How task branches are created: `"worktree"` (git worktrees) or `"branch"` (regular branches). |
| `merge_strategy` | string | `"rebase"` | Git merge strategy: `"rebase"` or `"squash"`. |
| `branch_template` | string | `null` | Template for branch names (e.g., `"task/{{id}}-{{slug}}"`). |

### Workflow Stages

Stages live under `[workflow.<stage>]`. The skill consumes these built-in stage names:

```
task_add         task_ready       task_start       task_complete
task_cancel      task_select      branch_set       branch_cleanup
branch_merge     pr_create        pr_update        plan
implement        contract_add     contract_edit    contract_delete
contract_dod_check                contract_dod_uncheck
contract_note_add
```

Additional, user-defined stage names are accepted as well — unknown stages are preserved in the `senko config` output so external scripts can consume them. The skill only fires on the built-in names listed above. Of the `contract_*` stages, the bundled workflows currently emit only `contract_add`, `contract_note_add`, and `contract_dod_check`; `contract_edit`, `contract_delete`, and `contract_dod_uncheck` are recognized as built-ins but are reserved for user-defined workflow extensions.

Each stage supports:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `instructions` | string[] | `[]` | Text instructions for the agent at this stage. |
| `hooks` | map<string, HookDef> | `{}` | Named hook definitions under `[workflow.<stage>.hooks.<name>]`. `when` / `mode` / `on_failure` on each hook determine pre-vs-post and sync-vs-async behavior (see [Hooks](#hooks) below). |
| `metadata_fields` | field[] | `[]` | Metadata fields collected at this stage. Values are shallow-merged into the task's metadata. |

Stage-specific keys (unknown keys are preserved as pass-through extras):

| Stage | Key | Type | Description |
|-------|-----|------|-------------|
| `workflow.task_add` | `default_dod` | string[] | Default Definition of Done items for new tasks. |
| `workflow.task_add` | `default_tags` | string[] | Default tags for new tasks. |
| `workflow.task_add` | `default_priority` | string | Default priority for new tasks. |
| `workflow.plan` | `required_sections` | string[] | Required sections in implementation plans. |

> **Note**: The old `pre_hooks` / `post_hooks` arrays are gone. Use `hooks` with `when = "pre"` or `when = "post"` on each hook definition instead.

### `[backend.sqlite]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `db_path` | string | auto | Path to the SQLite database file. Default: `$XDG_DATA_HOME/senko/projects/<hash>/data.db` |

### `[backend.postgres]` (requires `postgres` feature)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `url` | string | `null` | PostgreSQL connection URL (e.g., `postgres://user:pass@host/db`). Also settable via `--postgres-url`. |
| `url_arn` | string | `null` | AWS Secrets Manager ARN for connection URL (requires `aws-secrets` feature). |
| `rds_secrets_arn` | string | `null` | AWS Secrets Manager ARN for RDS JSON secret (must contain `username`, `password`, `host`; optionally `port`, `dbname`). |
| `sslrootcert` | string | `null` | Path to SSL root certificate for TLS connections. |
| `max_connections` | u32 | `null` | Maximum number of connections in the database pool. |

### `[server]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | string | `"127.0.0.1"` | Bind address for `senko serve`. |
| `port` | u16 | `3142` | Port for `senko serve`. |

### `[server.auth.api_key]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `master_key` | string | `null` | Direct master API key value for authentication. |
| `master_key_arn` | string | `null` | AWS Secrets Manager ARN for master API key (requires `aws-secrets` feature). |

### `[server.auth.oidc]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `issuer_url` | string | `null` | OIDC issuer URL for JWT verification. |
| `client_id` | string | `null` | OIDC client ID for PKCE authentication. |
| `scopes` | string[] | `["openid", "profile"]` | OIDC scopes to request. |
| `username_claim` | string | `null` | JWT claim to use as username. |
| `required_claims` | map | `{}` | Required JWT claims (key-value pairs that must match). |
| `callback_ports` | string[] | `[]` | Local ports for OIDC callback during CLI login. Supports individual ports and ranges (e.g., `["8400", "9000-9010"]`). |

### `[server.auth.oidc.session]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ttl` | string | `null` | Session time-to-live (e.g., `"24h"`, `"30d"`). `null` = no expiration. |
| `inactive_ttl` | string | `null` | Session inactive timeout (e.g., `"7d"`). `null` = no expiration. |
| `max_per_user` | u32 | `null` | Maximum number of sessions per user. `null` = unlimited. |

### `[server.auth.trusted_headers]`

Used for deployments behind a reverse proxy (e.g., API Gateway) that injects validated identity headers. See [AWS Deployment Guide](AWS_DEPLOYMENT.md).

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `subject_header` | string | `null` | Header containing the user's subject identifier. Required to enable this mode. |
| `name_header` | string | `null` | Header containing the user's display name. |
| `display_name_header` | string | `null` | Fallback header for display name (used if `name_header` is not present). |
| `email_header` | string | `null` | Header containing the user's email address. |
| `groups_header` | string | `null` | Header containing the user's groups. |
| `scope_header` | string | `null` | Header containing the OAuth scope. |
| `oidc_issuer_url` | string | `null` | OIDC issuer URL returned by `GET /auth/config` (for CLI login discovery). |
| `oidc_client_id` | string | `null` | OIDC client ID returned by `GET /auth/config` (for CLI login discovery). |

> **Note**: Only one authentication mode (API key, OIDC, or trusted headers) can be active at a time.

### `[server.relay]`

Applies when the binary runs as a relay server (`senko serve --proxy`). Hooks defined under this section fire only in that runtime.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `url` | string | `null` | Upstream relay server URL. When set, the server operates in relay mode and forwards requests to this URL. |
| `token` | string | `null` | API token for authentication with the upstream relay server. |

Task action hooks — `[server.relay.task_add.hooks.<name>]` / `[server.relay.task_ready.hooks.<name>]` / `[server.relay.task_start.hooks.<name>]` / `[server.relay.task_complete.hooks.<name>]` / `[server.relay.task_cancel.hooks.<name>]` / `[server.relay.task_select.hooks.<name>]`. See [Hooks](#hooks).

Contract action hooks — `[server.relay.contract_add.hooks.<name>]` / `[server.relay.contract_edit.hooks.<name>]` / `[server.relay.contract_delete.hooks.<name>]` / `[server.relay.contract_dod_check.hooks.<name>]` / `[server.relay.contract_dod_uncheck.hooks.<name>]` / `[server.relay.contract_note_add.hooks.<name>]`. See [Hooks](#hooks).

### `[server.remote]`

Applies when the binary runs as the direct (non-relay) server (`senko serve`). Hooks defined under this section fire only in that runtime.

Task action hooks — `[server.remote.task_add.hooks.<name>]` / `[server.remote.task_ready.hooks.<name>]` / `[server.remote.task_start.hooks.<name>]` / `[server.remote.task_complete.hooks.<name>]` / `[server.remote.task_cancel.hooks.<name>]` / `[server.remote.task_select.hooks.<name>]`. See [Hooks](#hooks).

Contract action hooks — `[server.remote.contract_add.hooks.<name>]` / `[server.remote.contract_edit.hooks.<name>]` / `[server.remote.contract_delete.hooks.<name>]` / `[server.remote.contract_dod_check.hooks.<name>]` / `[server.remote.contract_dod_uncheck.hooks.<name>]` / `[server.remote.contract_note_add.hooks.<name>]`. See [Hooks](#hooks).

### `[cli]`

Applies when the binary runs as a local CLI (i.e., not `senko serve` / `senko serve --proxy`). Hooks defined under this section fire only in that runtime.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `browser` | bool | `true` | Auto-open browser for OIDC login. |

Task action hooks — `[cli.task_add.hooks.<name>]` / `[cli.task_ready.hooks.<name>]` / `[cli.task_start.hooks.<name>]` / `[cli.task_complete.hooks.<name>]` / `[cli.task_cancel.hooks.<name>]` / `[cli.task_select.hooks.<name>]`. See [Hooks](#hooks).

Contract action hooks — `[cli.contract_add.hooks.<name>]` / `[cli.contract_edit.hooks.<name>]` / `[cli.contract_delete.hooks.<name>]` / `[cli.contract_dod_check.hooks.<name>]` / `[cli.contract_dod_uncheck.hooks.<name>]` / `[cli.contract_note_add.hooks.<name>]`. See [Hooks](#hooks).

### `[cli.remote]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `url` | string | `null` | Remote server URL. When set, CLI forwards commands to this server. |
| `token` | string | `null` | API token for authentication with the remote server. |

### `[web]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | string | `"127.0.0.1"` | Bind address for `senko web`. |
| `port` | u16 | `null` (auto) | Port for `senko web`. Default: `3141`. |

### `[log]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `dir` | string | auto | Directory for log files. Default: `$XDG_STATE_HOME/senko` |
| `level` | string | `"info"` | Minimum log level: `trace`, `debug`, `info`, `warn`, `error`. |
| `format` | string | `"json"` | Log output format: `"json"` or `"pretty"`. |
| `hook_output` | string | `"file"` | Where hook stdout/stderr goes: `"file"`, `"stdout"`, or `"both"`. |

### `[project]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | `null` | Project name. Used for hook environment variables and identification. Auto-detected if unset. |

### `[user]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | `null` | User name for task assignment. Auto-detected if unset. |

## Hooks

Hooks are named shell commands defined on a per-runtime, per-action basis. The key structure is uniform across every runtime:

```
<runtime>.<aggregate>_<action>.hooks.<name>
```

### Runtimes

| Runtime | Active when | Section prefix |
|---------|-------------|----------------|
| `cli` | Local CLI binary (not `senko serve` / `senko serve --proxy`) | `[cli.<action>.hooks.<name>]` |
| `server.relay` | Relay server (`senko serve --proxy`) | `[server.relay.<action>.hooks.<name>]` |
| `server.remote` | Direct server (`senko serve`) | `[server.remote.<action>.hooks.<name>]` |
| `workflow` | Workflow stages consumed by the Claude Code skill | `[workflow.<stage>.hooks.<name>]` |

### Actions

The `cli` / `server.relay` / `server.remote` runtimes expose a **fixed** set of actions for each aggregate. Task-aggregate actions:

| Action | Fires when |
|--------|-----------|
| `task_add` | `senko task add` creates a task |
| `task_ready` | `senko task ready` transitions draft → todo |
| `task_start` | `senko task start` or `senko task next` starts a task |
| `task_complete` | `senko task complete` completes a task |
| `task_cancel` | `senko task cancel` cancels a task |
| `task_select` | `senko task next` selects a task or finds none (filter with `on_result`) |

Contract-aggregate actions:

| Action | Fires when |
|--------|-----------|
| `contract_add` | `senko contract add` creates a contract |
| `contract_edit` | `senko contract edit` updates a contract |
| `contract_delete` | `senko contract delete` removes a contract |
| `contract_dod_check` | `senko contract dod check` marks a DoD item |
| `contract_dod_uncheck` | `senko contract dod uncheck` unmarks a DoD item |
| `contract_note_add` | `senko contract note add` appends a note |

The `workflow` runtime accepts **any** stage name — see [Workflow Stages](#workflow-stages) for the built-in names that the skill fires on.

### `HookDef` fields

Each named hook under `<runtime>.<aggregate>_<action>.hooks.<name>` is a `HookDef`:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `command` | string | _(required)_ | Shell command to execute (via `sh -c`). Receives the event envelope as JSON on stdin. |
| `when` | `"pre"` / `"post"` | `"post"` | Fire before or after the state transition. |
| `mode` | `"sync"` / `"async"` | `"async"` | `sync` waits for completion; `async` spawns and detaches. |
| `on_failure` | `"abort"` / `"warn"` / `"ignore"` | `"abort"` | Behavior when the command exits non-zero. **`abort` only takes effect on `sync`+`pre` hooks** — for `sync`+`post` or `async` hooks, `abort` degrades to a log entry. |
| `enabled` | bool | `true` | Set to `false` to temporarily disable without removing the definition. |
| `env_vars` | `EnvVarSpec[]` | `[]` | Environment variables to validate / inject (see below). |
| `on_result` | `"selected"` / `"none"` / `"any"` | `"any"` | Only meaningful for `task_select` hooks. `selected` = fire only on successful selection; `none` = fire only when no eligible task exists; `any` = fire in either case. Ignored on every task action other than `task_select` and on every `contract_*` action. |
| `prompt` | string | `null` | Only meaningful for `workflow.<stage>.hooks.<name>` entries — emitted by the skill as an agent instruction at that stage. Ignored by the `cli` / `server.relay` / `server.remote` runtimes. |

### `EnvVarSpec` fields

Each entry in `env_vars` is an `EnvVarSpec`:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | _(required)_ | Environment variable name. |
| `required` | bool | `true` | If `true` and the variable is unset at fire time with no `default`, the hook is **skipped** and a warning is logged. |
| `default` | string | `null` | If set, used when the variable is unset at fire time. |
| `description` | string | `null` | Human-readable note for config readers. |

### Fire-time behavior

- **Runtime filter**: Only hooks under the runtime matching the active process fire. Hooks under other runtimes are ignored; the process logs a single startup warning listing any mismatched sections so misconfigurations are easy to spot (`hooks configured under runtime sections that do not match the active runtime; they will not fire`).
- **`when` filter**: A hook with `when = "pre"` fires before the state transition; `when = "post"` fires after. Workflow stages and task actions both fire hooks in both positions.
- **`mode`**: `sync` blocks until the command exits; `async` starts the process and returns immediately.
- **`on_failure = "abort"` semantics**: The state transition is aborted (`DomainError::HookAborted`) only when the failing hook is both `sync` and `when = "pre"`. In any other combination, the `abort` setting degrades to a warning log. Use `warn` or `ignore` if you want the fire-and-forget logging behavior with an explicit label.

### Load-time validation

At startup the config is walked and the following warnings are emitted (the offending hook is accepted but its `abort` / `on_result` setting is effectively ignored):

- `pre` + `async` + `on_failure = "abort"` — async hooks cannot abort; `abort` is effectively `warn`.
- `on_result` set on a non-`task_select` hook — `on_result` is only meaningful for `task_select` and is ignored elsewhere.

### Example: notify on task completion

```toml
[cli.task_complete.hooks.notify]
command = "curl -X POST -d @- $WEBHOOK_URL"
mode = "async"

[[cli.task_complete.hooks.notify.env_vars]]
name = "WEBHOOK_URL"
required = true
```

### Example: task_select with `on_result`

Replacement for the old `on_no_eligible_task` event — fire only when `senko task next` finds nothing:

```toml
[cli.task_select.hooks.prompt_for_add]
command = "echo 'no eligible task — consider adding one'"
on_result = "none"
```

Fire on successful selection:

```toml
[cli.task_select.hooks.log_selection]
command = "logger -t senko 'task selected'"
on_result = "selected"
```

### Example: sync+pre+abort gating

A `sync`+`pre`+`abort` hook aborts the state transition when it exits non-zero — useful for gating completion on local checks:

```toml
[workflow.branch_merge.hooks.mise_check]
command = "mise check"
when = "pre"
mode = "sync"
on_failure = "abort"
```

### Example: server-side hooks

```toml
[server.remote.task_ready.hooks.metrics]
command = "emit-metric task_ready"
mode = "async"
```

### Example: contract hooks

Contract hooks fire on `senko contract <verb>` commands. The envelope on stdin carries a `contract` object (full `senko contract get` schema) instead of `task`. The hook shape is otherwise identical to task hooks — `when` / `mode` / `on_failure` / `env_vars` all apply, and `sync`+`pre`+`abort` still aborts the operation.

```toml
# Server-side audit of DoD check events
[server.remote.contract_dod_check.hooks.audit]
command = "jq -r '.event.contract.id' | xargs -I{} logger -t senko 'contract {} dod check'"
mode = "async"

# Skill-side prompt emitted before adding a contract note
[workflow.contract_note_add.hooks.review_before_note]
command = "true"
prompt = "Re-read the most recent notes on this contract before adding a new one — skip if the same observation already exists."
when = "pre"
```

## Environment Variables

### Workflow

| Variable | Config Key | Values |
|----------|-----------|--------|
| `SENKO_MERGE_VIA` | `workflow.merge_via` | `direct`, `pr` |
| `SENKO_AUTO_MERGE` | `workflow.auto_merge` | `true`/`1`/`yes`, `false`/`0`/`no` |
| `SENKO_BRANCH_MODE` | `workflow.branch_mode` | `worktree`, `branch` |
| `SENKO_MERGE_STRATEGY` | `workflow.merge_strategy` | `rebase`, `squash` |

### Connection

| Variable | Config Key | Description |
|----------|-----------|-------------|
| `SENKO_CLI_REMOTE_URL` | `cli.remote.url` | Remote server URL |
| `SENKO_CLI_REMOTE_TOKEN` | `cli.remote.token` | API token |

### Server

| Variable | Config Key | Description |
|----------|-----------|-------------|
| `SENKO_SERVER_RELAY_URL` | `server.relay.url` | Upstream relay server URL |
| `SENKO_SERVER_RELAY_TOKEN` | `server.relay.token` | API token for upstream relay server |
| `SENKO_SERVER_HOST` | `server.host` | Bind address for `senko serve` only |
| `SENKO_SERVER_PORT` | `server.port` | Port for `senko serve` only |
| `SENKO_HOST` | `web.host` + `server.host` | Bind address for both `senko web` and `senko serve` |
| `SENKO_PORT` | `web.port` + `server.port` | Port for both `senko web` and `senko serve` |

> `SENKO_SERVER_HOST`/`SENKO_SERVER_PORT` only affect `senko serve`. `SENKO_HOST`/`SENKO_PORT` affect both `senko serve` and `senko web`.

### Authentication

| Variable | Config Key |
|----------|-----------|
| `SENKO_AUTH_API_KEY_MASTER_KEY` | `server.auth.api_key.master_key` |
| `SENKO_AUTH_API_KEY_MASTER_KEY_ARN` | `server.auth.api_key.master_key_arn` |
| `SENKO_OIDC_ISSUER_URL` | `server.auth.oidc.issuer_url` |
| `SENKO_OIDC_CLIENT_ID` | `server.auth.oidc.client_id` |
| `SENKO_OIDC_USERNAME_CLAIM` | `server.auth.oidc.username_claim` |
| `SENKO_OIDC_CALLBACK_PORTS` | `server.auth.oidc.callback_ports` (comma-separated) |
| `SENKO_AUTH_OIDC_SESSION_TTL` | `server.auth.oidc.session.ttl` |
| `SENKO_AUTH_OIDC_SESSION_INACTIVE_TTL` | `server.auth.oidc.session.inactive_ttl` |
| `SENKO_AUTH_OIDC_SESSION_MAX_PER_USER` | `server.auth.oidc.session.max_per_user` (parsed as u32) |

### Trusted Headers

| Variable | Config Key |
|----------|-----------|
| `SENKO_AUTH_TRUSTED_HEADERS_SUBJECT_HEADER` | `server.auth.trusted_headers.subject_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_NAME_HEADER` | `server.auth.trusted_headers.name_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_EMAIL_HEADER` | `server.auth.trusted_headers.email_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_GROUPS_HEADER` | `server.auth.trusted_headers.groups_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_SCOPE_HEADER` | `server.auth.trusted_headers.scope_header` |
| `SENKO_AUTH_TRUSTED_HEADERS_OIDC_ISSUER_URL` | `server.auth.trusted_headers.oidc_issuer_url` |
| `SENKO_AUTH_TRUSTED_HEADERS_OIDC_CLIENT_ID` | `server.auth.trusted_headers.oidc_client_id` |

### Backend

| Variable | Config Key |
|----------|-----------|
| `SENKO_DB_PATH` | `backend.sqlite.db_path` |
| `SENKO_POSTGRES_URL` | `backend.postgres.url` |
| `SENKO_POSTGRES_URL_ARN` | `backend.postgres.url_arn` |
| `SENKO_POSTGRES_RDS_SECRETS_ARN` | `backend.postgres.rds_secrets_arn` |
| `SENKO_POSTGRES_SSLROOTCERT` | `backend.postgres.sslrootcert` |
| `SENKO_POSTGRES_MAX_CONNECTIONS` | `backend.postgres.max_connections` (parsed as u32) |

### Hooks

Hook definitions are **not** configurable via environment variables. Define them in `.senko/config.toml` under the runtime-specific sections (`[cli.<action>.hooks.<name>]`, `[server.relay.<action>.hooks.<name>]`, `[server.remote.<action>.hooks.<name>]`, `[workflow.<stage>.hooks.<name>]`). The same section layout applies to both `task_*` and `contract_*` actions.

### Other

| Variable | Config Key | Description |
|----------|-----------|-------------|
| `SENKO_USER` | `user.name` | User name |
| `SENKO_PROJECT` | `project.name` | Project name |
| `SENKO_LOG_DIR` | `log.dir` | Log directory |
| `SENKO_LOG_LEVEL` | `log.level` | Log level |
| `SENKO_LOG_FORMAT` | `log.format` | Log format (`json` or `pretty`) |
| `SENKO_CONFIG` | _(CLI-level)_ | Path to config file |
| `SENKO_PROJECT_ROOT` | _(CLI-level)_ | Project root directory |

## Breaking Changes

The hooks configuration schema has been fully redesigned. The old schema is **not** accepted — old `[hooks]` sections and their related environment variables are removed without a compatibility shim. (Legacy scalar / array shorthand forms are rejected at load time; nested legacy `[hooks]` tables are warned about but their contents will not fire.)

| Old | New | Notes |
|-----|-----|-------|
| `[hooks]` (top-level) | `[cli.<action>.hooks.<name>]` / `[server.relay.<action>.hooks.<name>]` / `[server.remote.<action>.hooks.<name>]` | Runtime now selects which hooks fire |
| `[hooks].enabled` master switch | _(removed)_ | Disable hooks individually via `enabled = false` |
| `on_task_added` | `task_add` | |
| `on_task_ready` | `task_ready` | |
| `on_task_started` | `task_start` | |
| `on_task_completed` | `task_complete` | |
| `on_task_canceled` | `task_cancel` | |
| `on_no_eligible_task` | `task_select` with `on_result = "none"` | Consolidated into a single `task_select` action |
| `requires_env = [...]` | `env_vars = [{ name = "...", required = true }]` | Typed specs with optional defaults |
| `[workflow.<stage>] pre_hooks = [...]` / `post_hooks = [...]` | `[workflow.<stage>.hooks.<name>]` with `when = "pre" \| "post"` | Unified hook shape |
| `SENKO_HOOKS_ENABLED` env | _(removed)_ | Hooks master switch no longer exists |
| `SENKO_HOOK_ON_TASK_*` env | _(removed)_ | Define hooks only in `config.toml` |
| `SENKO_HOOK_ON_NO_ELIGIBLE_TASK` env | _(removed)_ | Use `[cli.task_select.hooks.*] on_result = "none"` |
| Legacy workflow stage names (`add`, `start`, `plan`, `complete`, `branch`, `merge`, `pr`) | `task_add`, `task_start`, `plan`, `task_complete`, `branch_set`, `branch_merge`, `pr_create` | The skill fires on the new names only |
| _(none)_ | `contract_add` / `contract_edit` / `contract_delete` / `contract_dod_check` / `contract_dod_uncheck` / `contract_note_add` | **New** actions for the contract aggregate — no migration needed; they coexist with `task_*` actions under the same runtime sections |

### Other TOML aliases (retained)

| Deprecated | Current | Notes |
|-----------|---------|-------|
| `workflow.completion_mode` | `workflow.merge_via` | Accepted via serde alias |
| `merge_then_complete` (value) | `direct` | Accepted as value for `merge_via` |
| `pr_then_complete` (value) | `pr` | Accepted as value for `merge_via` |

### Other environment variable aliases (retained)

| Deprecated | Current | Notes |
|-----------|---------|-------|
| `SENKO_COMPLETION_MODE` | `SENKO_MERGE_VIA` | Prints deprecation warning |

## Related Documentation

- [Authentication Setup](AUTH_SETUP.md) — Auth modes and setup
- [CLI Reference](CLI.md) — Full command details
- [AWS Deployment](AWS_DEPLOYMENT.md) — Trusted headers deployment
- [README](../README.md) — Project overview
