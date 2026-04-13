# Config Explain

Explain the current senko configuration values and their meanings.

## Procedure

### Step 1: Get Current Config

```bash
senko config
```

Parse the JSON output to extract all configuration sections.

### Step 2: Explain Each Section

For each section, explain every item's **current value**, whether it's the **default**, what it **means**, and the **available options**.

#### Sections

**workflow**
| Key | Default | Options | Description |
|---|---|---|---|
| `merge_via` | `direct` | `direct`, `pr` | Controls whether the branch is merged directly or via PR. `pr` requires a PR URL and merge status check. |
| `auto_merge` | `true` | `true`, `false` | Applies only to `merge_via = "direct"`. Controls whether the branch is merged automatically or requires user confirmation. Has no effect when `merge_via = "pr"`. |
| `branch_mode` | `worktree` | `worktree`, `branch` | How task branches are created. `worktree` uses git worktrees (parallel work), `branch` uses regular branches. |
| `merge_strategy` | `rebase` | `rebase`, `squash` | Git merge strategy when merging task branches back to main. |
| `branch_template` | `null` | string | Template for branch names (e.g., `task/{{id}}-{{slug}}`). |

**workflow stages** (`workflow.add`, `workflow.start`, `workflow.branch`, `workflow.plan`, `workflow.implement`, `workflow.merge`, `workflow.pr`, `workflow.complete`, `workflow.branch_cleanup`)

Each stage supports:
| Key | Default | Description |
|---|---|---|
| `instructions` | `[]` | Text instructions for the agent at this stage. |
| `pre_hooks` | `[]` | Hooks to run before the stage. Each hook is a string (shell command) or `{command, prompt, on_failure}`. |
| `post_hooks` | `[]` | Hooks to run after the stage. Same format as `pre_hooks`. |

Stage-specific keys:
| Stage | Extra Keys | Description |
|---|---|---|
| `workflow.add` | `default_dod`, `default_tags`, `default_priority` | Defaults applied when creating new tasks. |
| `workflow.start` | `metadata_fields` | Metadata fields collected when starting a task. |
| `workflow.plan` | `required_sections` | Required sections in the implementation plan. |
| `workflow.complete` | `metadata_fields` | Metadata fields collected when completing a task. |

**backend.sqlite**
| Key | Default | Options | Description |
|---|---|---|---|
| `db_path` | auto (`$XDG_DATA_HOME/senko/projects/<hash>/data.db`) | file path | Path to the SQLite database file. |

**backend.postgres** (requires `postgres` feature)
| Key | Default | Options | Description |
|---|---|---|---|
| `url` | `null` | URL string | PostgreSQL connection URL (e.g., `postgres://user:pass@host/db`). Also settable via `--postgres-url` CLI flag. |
| `url_arn` | `null` | ARN string | AWS Secrets Manager ARN for connection URL. Resolved at startup (requires `aws-secrets` feature). |
| `rds_secrets_arn` | `null` | ARN string | AWS Secrets Manager ARN for RDS JSON secret. The JSON must contain `username`, `password`, `host`, and optionally `port` (default 5432) and `dbname` (default `postgres`). |
| `sslrootcert` | `null` | file path | Path to SSL root certificate for TLS connections. When used with `rds_secrets_arn`, appends `sslmode=verify-full` to the built URL. |
| `max_connections` | `null` | positive integer | Maximum number of connections in the database pool. |

**server**
| Key | Default | Options | Description |
|---|---|---|---|
| `host` | `127.0.0.1` | IP address | Bind address for `senko serve`. |
| `port` | `3142` | port number | Port for `senko serve`. |

**server.auth.api_key**
| Key | Default | Options | Description |
|---|---|---|---|
| `master_key` | `null` | string | Direct master API key value for authentication. |
| `master_key_arn` | `null` | ARN string | AWS Secrets Manager ARN for master API key. Resolved at startup (requires `aws-secrets` feature). |

**server.auth.oidc**
| Key | Default | Options | Description |
|---|---|---|---|
| `issuer_url` | `null` | URL string | OIDC issuer URL for JWT verification. |
| `client_id` | `null` | string | OIDC client ID for PKCE authentication. |
| `scopes` | `["openid", "profile"]` | list of strings | OIDC scopes to request. |
| `username_claim` | `null` | string | JWT claim to use as username. |
| `required_claims` | `{}` | map of string to string | Required JWT claims (key-value pairs that must match). |
| `callback_ports` | `[]` (empty) | list of port strings | Local ports for OIDC callback during CLI login. Supports individual ports and ranges (e.g., `["8400", "9000-9010"]`). |

**server.auth.oidc.cli**
| Key | Default | Options | Description |
|---|---|---|---|
| `browser` | `true` | `true`, `false` | Auto-open browser for OIDC login. |

**server.auth.oidc.session**
| Key | Default | Options | Description |
|---|---|---|---|
| `ttl` | `null` | duration string | Session time-to-live (e.g., `24h`). |
| `inactive_ttl` | `null` | duration string | Session inactive timeout (e.g., `7d`). |
| `max_per_user` | `null` | positive integer | Maximum number of sessions per user. |

**cli.remote**
| Key | Default | Options | Description |
|---|---|---|---|
| `url` | `null` | URL string | Remote server URL for CLI client mode. When set, CLI forwards commands to this server. |
| `token` | `null` | string | API token for authentication with the remote server. |

**log**
| Key | Default | Options | Description |
|---|---|---|---|
| `dir` | auto (`$XDG_STATE_HOME/senko`) | directory path | Directory for log files. |
| `level` | `info` | `trace`, `debug`, `info`, `warn`, `error` | Minimum log level. |
| `format` | `json` | `json`, `pretty` | Log output format. |

**project**
| Key | Default | Options | Description |
|---|---|---|---|
| `name` | `null` (auto-detected) | string | Project name. Used for hook environment variables and identification. |

**user**
| Key | Default | Options | Description |
|---|---|---|---|
| `name` | `null` (auto-detected) | string | User name for task assignment. |

**web**
| Key | Default | Options | Description |
|---|---|---|---|
| `host` | `127.0.0.1` | IP address | Bind address for `senko web`. |
| `port` | `null` (auto) | port number | Port for `senko web`. |

**hooks**
| Key | Default | Options | Description |
|---|---|---|---|
| `enabled` | `true` | `true`, `false` | Whether hooks fire on this process (CLI). API server always fires hooks regardless. |

| Event | Description |
|---|---|
| `on_task_added` | Triggered when a new task is created. |
| `on_task_ready` | Triggered when a task moves to `todo` status. |
| `on_task_started` | Triggered when a task moves to `in_progress`. |
| `on_task_completed` | Triggered when a task is completed. |
| `on_task_canceled` | Triggered when a task is canceled. |
| `on_no_eligible_task` | Triggered when `senko next` finds no eligible tasks. |

Each hook entry has: `command` (shell command), `enabled` (bool, default true), `requires_env` (list of required env vars).

### Step 3: Explain Config Layering

Explain how configuration is resolved (highest priority first):
1. **CLI flags** (`--config <path>`)
2. **Environment variables** (`SENKO_*` — see table below)
3. **Local config** (`.senko/config.local.toml` — git-ignored, per-user overrides)
4. **Project config** (`.senko/config.toml` in the project root)
5. **User config** (`~/.config/senko/config.toml`)

Higher-priority sources override lower ones. The `senko config` output shows the **merged** result.

#### Environment Variables

| Variable | Config Key | Notes |
|---|---|---|
| `SENKO_MERGE_VIA` | `workflow.merge_via` | |
| `SENKO_AUTO_MERGE` | `workflow.auto_merge` | |
| `SENKO_BRANCH_MODE` | `workflow.branch_mode` | |
| `SENKO_MERGE_STRATEGY` | `workflow.merge_strategy` | |
| `SENKO_SERVER_URL` | `cli.remote.url` | |
| `SENKO_TOKEN` | `cli.remote.token` | |
| `SENKO_HOOKS_ENABLED` | `hooks.enabled` | |
| `SENKO_HOOK_ON_TASK_ADDED` | `hooks.on_task_added` | Shell command |
| `SENKO_HOOK_ON_TASK_READY` | `hooks.on_task_ready` | Shell command |
| `SENKO_HOOK_ON_TASK_STARTED` | `hooks.on_task_started` | Shell command |
| `SENKO_HOOK_ON_TASK_COMPLETED` | `hooks.on_task_completed` | Shell command |
| `SENKO_HOOK_ON_TASK_CANCELED` | `hooks.on_task_canceled` | Shell command |
| `SENKO_HOOK_ON_NO_ELIGIBLE_TASK` | `hooks.on_no_eligible_task` | Shell command |
| `SENKO_AUTH_API_KEY_MASTER_KEY` | `server.auth.api_key.master_key` | |
| `SENKO_AUTH_API_KEY_MASTER_KEY_ARN` | `server.auth.api_key.master_key_arn` | |
| `SENKO_OIDC_ISSUER_URL` | `server.auth.oidc.issuer_url` | |
| `SENKO_OIDC_CLIENT_ID` | `server.auth.oidc.client_id` | |
| `SENKO_OIDC_USERNAME_CLAIM` | `server.auth.oidc.username_claim` | |
| `SENKO_OIDC_CALLBACK_PORTS` | `server.auth.oidc.callback_ports` | Comma-separated |
| `SENKO_AUTH_OIDC_SESSION_TTL` | `server.auth.oidc.session.ttl` | |
| `SENKO_AUTH_OIDC_SESSION_INACTIVE_TTL` | `server.auth.oidc.session.inactive_ttl` | |
| `SENKO_AUTH_OIDC_SESSION_MAX_PER_USER` | `server.auth.oidc.session.max_per_user` | Parsed as u32 |
| `SENKO_SERVER_HOST` | `server.host` | |
| `SENKO_SERVER_PORT` | `server.port` | Parsed as u16 |
| `SENKO_POSTGRES_URL` | `backend.postgres.url` | |
| `SENKO_POSTGRES_URL_ARN` | `backend.postgres.url_arn` | |
| `SENKO_POSTGRES_RDS_SECRETS_ARN` | `backend.postgres.rds_secrets_arn` | |
| `SENKO_POSTGRES_SSLROOTCERT` | `backend.postgres.sslrootcert` | |
| `SENKO_POSTGRES_MAX_CONNECTIONS` | `backend.postgres.max_connections` | Parsed as u32 |
| `SENKO_USER` | `user.name` | |
| `SENKO_PROJECT` | `project.name` | |
| `SENKO_DB_PATH` | `backend.sqlite.db_path` | |
| `SENKO_LOG_DIR` | `log.dir` | |
| `SENKO_LOG_LEVEL` | `log.level` | |
| `SENKO_LOG_FORMAT` | `log.format` | |
| `SENKO_HOST` | `web.host` + `server.host` | Applies to both web and server |
| `SENKO_PORT` | `web.port` + `server.port` | Applies to both web and server |

### Step 4: Present to User

Format the explanation clearly, highlighting:
- Values that differ from defaults
- Any potentially important settings (e.g., `merge_via`, `hooks.enabled`)
- Hooks that are currently configured
