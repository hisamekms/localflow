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

**backend**
| Key | Default | Options | Description |
|---|---|---|---|
| `api_url` | `null` | URL string | HTTP backend API URL. When set, senko operates in remote mode. |
| `api_key` | `null` | string | API key for authenticating with the remote backend. |

**backend.postgres** (requires `postgres` feature)
| Key | Default | Options | Description |
|---|---|---|---|
| `url` | `null` | URL string | PostgreSQL connection URL (e.g., `postgres://user:pass@host/db`). Also settable via `--postgres-url` CLI flag. |
| `url_arn` | `null` | ARN string | AWS Secrets Manager ARN for connection URL. Resolved at startup (requires `aws-secrets` feature). |
| `rds_secrets_arn` | `null` | ARN string | AWS Secrets Manager ARN for RDS JSON secret. The JSON must contain `username`, `password`, `host`, and optionally `port` (default 5432) and `dbname` (default `postgres`). |
| `sslrootcert` | `null` | file path | Path to SSL root certificate for TLS connections. When used with `rds_secrets_arn`, appends `sslmode=verify-full` to the built URL. |
| `max_connections` | `null` | positive integer | Maximum number of connections in the database pool. |

**auth**
| Key | Default | Options | Description |
|---|---|---|---|
| `enabled` | `false` | `true`, `false` | Enable/disable API authentication. |
| `master_api_key` | `null` | string | Direct master API key value for authentication. |
| `master_api_key_arn` | `null` | ARN string | AWS Secrets Manager ARN for master API key. Resolved at startup (requires `aws-secrets` feature). |

**storage**
| Key | Default | Options | Description |
|---|---|---|---|
| `db_path` | auto (`$XDG_DATA_HOME/senko/projects/<dir-name>/data.db`) | file path | Path to the SQLite database file. |

**log**
| Key | Default | Options | Description |
|---|---|---|---|
| `dir` | auto (`$XDG_STATE_HOME/senko`) | directory path | Directory for log files. |
| `level` | `info` | `trace`, `debug`, `info`, `warn`, `error` | Minimum log level. |
| `format` | `json` | `json`, `text` | Log output format. |

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
| `host` | `127.0.0.1` | IP address | Host address for the web server. |

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
3. **Project config** (`.senko/config.toml` in the project root)
4. **User config** (`~/.config/senko/config.toml`)

Higher-priority sources override lower ones. The `senko config` output shows the **merged** result.

#### Environment Variables

| Variable | Config Key | Notes |
|---|---|---|
| `SENKO_MERGE_VIA` | `workflow.merge_via` | |
| `SENKO_AUTO_MERGE` | `workflow.auto_merge` | |
| `SENKO_BRANCH_MODE` | `workflow.branch_mode` | |
| `SENKO_MERGE_STRATEGY` | `workflow.merge_strategy` | |
| `SENKO_API_URL` | `backend.api_url` | |
| `SENKO_API_KEY` | `backend.api_key` | |
| `SENKO_POSTGRES_URL` | `backend.postgres.url` | |
| `SENKO_POSTGRES_URL_ARN` | `backend.postgres.url_arn` | |
| `SENKO_POSTGRES_RDS_SECRETS_ARN` | `backend.postgres.rds_secrets_arn` | |
| `SENKO_POSTGRES_SSLROOTCERT` | `backend.postgres.sslrootcert` | |
| `SENKO_POSTGRES_MAX_CONNECTIONS` | `backend.postgres.max_connections` | Parsed as u32 |
| `SENKO_AUTH_ENABLED` | `auth.enabled` | Accepts `true`/`1`/`yes` or `false`/`0`/`no` (case-insensitive) |
| `SENKO_MASTER_API_KEY` | `auth.master_api_key` | |
| `SENKO_MASTER_API_KEY_ARN` | `auth.master_api_key_arn` | |
| `SENKO_HOOKS_ENABLED` | `hooks.enabled` | |
| `SENKO_HOOK_ON_TASK_ADDED` | `hooks.on_task_added` | Shell command |
| `SENKO_HOOK_ON_TASK_READY` | `hooks.on_task_ready` | Shell command |
| `SENKO_HOOK_ON_TASK_STARTED` | `hooks.on_task_started` | Shell command |
| `SENKO_HOOK_ON_TASK_COMPLETED` | `hooks.on_task_completed` | Shell command |
| `SENKO_HOOK_ON_TASK_CANCELED` | `hooks.on_task_canceled` | Shell command |
| `SENKO_HOOK_ON_NO_ELIGIBLE_TASK` | `hooks.on_no_eligible_task` | Shell command |
| `SENKO_USER` | `user.name` | |
| `SENKO_PROJECT` | `project.name` | |
| `SENKO_DB_PATH` | `storage.db_path` | |
| `SENKO_LOG_DIR` | `log.dir` | |
| `SENKO_LOG_LEVEL` | `log.level` | |
| `SENKO_LOG_FORMAT` | `log.format` | |
| `SENKO_HOST` | `web.host` | |
| `SENKO_PORT` | `web.port` | |

### Step 4: Present to User

Format the explanation clearly, highlighting:
- Values that differ from defaults
- Any potentially important settings (e.g., `merge_via`, `hooks.enabled`)
- Hooks that are currently configured
