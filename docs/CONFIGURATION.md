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

Stages: `workflow.add`, `workflow.start`, `workflow.branch`, `workflow.plan`, `workflow.implement`, `workflow.merge`, `workflow.pr`, `workflow.complete`, `workflow.branch_cleanup`

Each stage supports:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `instructions` | string[] | `[]` | Text instructions for the agent at this stage. |
| `pre_hooks` | hook[] | `[]` | Hooks to run before the stage. Each is a string (shell command) or `{command, prompt, on_failure}`. |
| `post_hooks` | hook[] | `[]` | Hooks to run after the stage. Same format as `pre_hooks`. |

Stage-specific keys:

| Stage | Key | Type | Description |
|-------|-----|------|-------------|
| `workflow.add` | `default_dod` | string[] | Default Definition of Done items for new tasks. |
| `workflow.add` | `default_tags` | string[] | Default tags for new tasks. |
| `workflow.add` | `default_priority` | string | Default priority for new tasks. |
| `workflow.start` | `metadata_fields` | field[] | Metadata fields collected when starting a task. |
| `workflow.plan` | `required_sections` | string[] | Required sections in implementation plans. |
| `workflow.complete` | `metadata_fields` | field[] | Metadata fields collected when completing a task. |

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

### `[cli]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `browser` | bool | `true` | Auto-open browser for OIDC login. |

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

### `[project]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | `null` | Project name. Used for hook environment variables and identification. Auto-detected if unset. |

### `[user]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | `null` | User name for task assignment. Auto-detected if unset. |

### `[hooks]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `true` | Whether hooks fire on this process (CLI). API server always fires hooks regardless. |

Hook events are configured as named entries under each event key:

```toml
[hooks.on_task_completed.webhook]
command = "curl -X POST https://example.com/webhook"
enabled = true
requires_env = ["WEBHOOK_URL"]
```

| Event | Trigger |
|-------|---------|
| `on_task_added` | A new task is created. |
| `on_task_ready` | A task moves to `todo` status. |
| `on_task_started` | A task moves to `in_progress`. |
| `on_task_completed` | A task is completed. |
| `on_task_canceled` | A task is canceled. |
| `on_no_eligible_task` | `senko next` finds no eligible tasks. |

Each hook entry has:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `command` | string | _(required)_ | Shell command to execute (via `sh -c`). |
| `enabled` | bool | `true` | Set to `false` to temporarily disable. |
| `requires_env` | string[] | `[]` | Only run if all listed environment variables are set. |

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
| `SENKO_SERVER_URL` | `cli.remote.url` | Remote server URL |
| `SENKO_TOKEN` | `cli.remote.token` | API token |

### Server

| Variable | Config Key | Description |
|----------|-----------|-------------|
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

| Variable | Config Key |
|----------|-----------|
| `SENKO_HOOKS_ENABLED` | `hooks.enabled` |
| `SENKO_HOOK_ON_TASK_ADDED` | `hooks.on_task_added` (shell command) |
| `SENKO_HOOK_ON_TASK_READY` | `hooks.on_task_ready` (shell command) |
| `SENKO_HOOK_ON_TASK_STARTED` | `hooks.on_task_started` (shell command) |
| `SENKO_HOOK_ON_TASK_COMPLETED` | `hooks.on_task_completed` (shell command) |
| `SENKO_HOOK_ON_TASK_CANCELED` | `hooks.on_task_canceled` (shell command) |
| `SENKO_HOOK_ON_NO_ELIGIBLE_TASK` | `hooks.on_no_eligible_task` (shell command) |

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

## Backward Compatibility

The following deprecated names are supported for backward compatibility:

### TOML Keys

| Deprecated | Current | Notes |
|-----------|---------|-------|
| `workflow.completion_mode` | `workflow.merge_via` | Accepted via serde alias |
| `merge_then_complete` (value) | `direct` | Accepted as value for `merge_via` |
| `pr_then_complete` (value) | `pr` | Accepted as value for `merge_via` |

### Environment Variables

| Deprecated | Current | Notes |
|-----------|---------|-------|
| `SENKO_COMPLETION_MODE` | `SENKO_MERGE_VIA` | Prints deprecation warning |

## Related Documentation

- [Authentication Setup](AUTH_SETUP.md) — Auth modes and setup
- [CLI Reference](CLI.md) — Full command details
- [AWS Deployment](AWS_DEPLOYMENT.md) — Trusted headers deployment
- [README](../README.md) — Project overview
