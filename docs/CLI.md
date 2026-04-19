# CLI Reference

[日本語](CLI.ja.md) | [Back to README](../README.md)

## Global Options

```
--output <FORMAT>       json or text (default: json)
--project-root <PATH>   Project root (auto-detected if omitted)
--config <PATH>         Path to config file (env: SENKO_CONFIG, default: .senko/config.toml)
--dry-run               Show what would happen without executing (state-changing commands only)
--log-dir <PATH>        Override log output directory (default: $XDG_STATE_HOME/senko)
```

> **Note**: `--output` and `--dry-run` are global flags — place them **before** the subcommand: `senko --output text task list`

## `task add` – Create a task

```bash
senko task add --title "Write docs" --priority p0
senko task add --title "Fix bug" \
  --background "Users report 500 errors" \
  --definition-of-done "No 500 errors in logs" \
  --in-scope "Error handler" \
  --out-of-scope "Refactoring" \
  --tag backend --tag urgent
```

New tasks start in `draft` status. Default priority is `p2`.

## `task list` – List tasks

```bash
senko task list                               # All tasks (default limit 50)
senko task list --status todo                 # Filter by status
senko task list --ready                       # Todo tasks with all deps met
senko task list --tag backend                 # Filter by tag
senko task list --contract 42                 # Filter by contract ID
senko task list --id-min 100 --id-max 199     # ID range (either bound is optional)
senko task list --limit 20 --offset 40        # Pagination (limit 1..=200, default 50)
```

Status values use snake_case in CLI flags: `todo`, `in_progress`, `completed`, `canceled`, `draft`.

Pagination: `--limit` defaults to 50 when omitted and must be in `1..=200` when explicit. `--offset` defaults to 0. Results are ordered by task `id` ascending so pagination is stable across requests.

## `task get <id>` – Task details

```bash
senko task get 1
```

> `get` outputs JSON only (no `--output text` support).

## `task next` – Start next task

Selects the highest-priority `todo` task whose dependencies are all completed, and sets it to `in_progress`.

```bash
senko task next
senko task next --session-id "session-abc"
```

Selection order: priority (P0 first) → created_at → id.

## `task edit <id>` – Edit a task

```bash
# Scalar fields
senko task edit 1 --title "New title"
senko task edit 1 --description "What to do"
senko task edit 1 --plan "How to do it"
senko task edit 1 --clear-description
senko task edit 1 --clear-plan
senko task edit 1 --status todo
senko task edit 1 --priority p0

# Array fields (tags, definition-of-done, scope)
senko task edit 1 --add-tag "urgent"
senko task edit 1 --remove-tag "old"
senko task edit 1 --set-tags "a" "b"         # Replace all

# Definition of Done
senko task edit 1 --add-definition-of-done "Write unit tests"

# PR URL
senko task edit 1 --pr-url "https://github.com/org/repo/pull/42"
senko task edit 1 --clear-pr-url

# Metadata (shallow merge — adds/overwrites keys, preserves unmentioned keys)
senko task edit 1 --metadata '{"sprint":"2026-Q2","points":3}'
# Replace entire metadata (removes all existing keys)
senko task edit 1 --replace-metadata '{"new_key":"only this"}'
# Delete a specific key (set to null in merge)
senko task edit 1 --metadata '{"points":null}'
# Clear all metadata
senko task edit 1 --clear-metadata
```

## `task complete <id>` – Complete a task

```bash
senko task complete 1
senko task complete 1 --skip-pr-check    # Bypass PR merge/review checks
```

Fails if any DoD items are unchecked. Use `dod check` to mark items before completing.

When `merge_via = "pr"` in config, also verifies the PR is merged. Use `--skip-pr-check` to bypass.

## `task cancel <id>` – Cancel a task

```bash
senko task cancel 1 --reason "out of scope"
```

## `task dod` – Manage Definition of Done items

```bash
senko task dod check <task_id> <index>      # Mark DoD item as done (1-based)
senko task dod uncheck <task_id> <index>    # Unmark DoD item
```

## `task deps` – Manage dependencies

```bash
senko task deps add 5 --on 3        # Task 5 depends on task 3
senko task deps remove 5 --on 3     # Remove dependency
senko task deps set 5 --on 1 2 3    # Set exact dependencies
senko task deps list 5              # List dependencies of task 5
```

## `config` – Show or initialize configuration

```bash
senko config              # Show current configuration (JSON)
senko --output text config # Show current configuration (text)
senko config --init       # Generate a template .senko/config.toml
```

Shows current configuration values (including defaults for missing settings). Use `--init` to generate a commented template file.

## `auth` – Authentication commands

Manage OIDC authentication sessions. Requires `[server]` and OIDC configuration on the server.

### `auth login`

```bash
senko auth login
senko auth login --device-name "my-laptop"
```

Opens a browser for OIDC login via OAuth PKCE flow. After authentication, the CLI receives an API token that is stored in the OS keychain.

| Option | Description |
|--------|-------------|
| `--device-name <NAME>` | Optional device name for the session |

### `auth token`

```bash
senko auth token
```

Prints the stored API token to stdout. Useful for passing the token to environments without keychain access (e.g., containers).

### `auth status`

```bash
senko auth status
```

Shows login status and current session info by querying `GET /auth/me`.

### `auth logout`

```bash
senko auth logout
```

Revokes the current session on the server and removes the token from the OS keychain.

### `auth sessions`

```bash
senko auth sessions
```

Lists active sessions via `GET /auth/sessions`.

### `auth revoke`

```bash
senko auth revoke --id <session-id>
senko auth revoke --all
```

Revokes a specific session by ID, or all sessions.

| Option | Description |
|--------|-------------|
| `--id <ID>` | Revoke a specific session |
| `--all` | Revoke all sessions |

## `skill-install` – Claude Code integration

```bash
senko skill-install
```

Generates a skill definition under `.claude/skills/senko/` for Claude Code integration.

## `serve` – Start the JSON API server

```bash
senko serve                # Listen on 127.0.0.1:3142
senko serve --port 8080    # Listen on 127.0.0.1:8080
senko serve --host 0.0.0.0 # Listen on 0.0.0.0:3142 (all interfaces)
```

| Option | Description |
|--------|-------------|
| `--port <PORT>` | Port to listen on (env: `SENKO_SERVER_PORT` or `SENKO_PORT`, default: `3142`) |
| `--host <ADDR>` | Bind address, e.g. `0.0.0.0` or `192.168.1.5` (env: `SENKO_SERVER_HOST` or `SENKO_HOST`, default: `127.0.0.1`) |

> `SENKO_SERVER_PORT`/`SENKO_SERVER_HOST` affect only `senko serve`. `SENKO_PORT`/`SENKO_HOST` affect both `senko serve` and `senko web`.

Provides a full JSON REST API under `/api/v1/...` for all task operations (CRUD, status transitions, dependencies, DoD, config, stats). Hooks fire the same way as CLI commands.

## `web` – Start a read-only web viewer

```bash
senko web                # Listen on 127.0.0.1:3141
senko web --port 8080    # Listen on 127.0.0.1:8080
senko web --host 0.0.0.0 # Listen on 0.0.0.0:3141 (all interfaces)
```

| Option | Description |
|--------|-------------|
| `--port <PORT>` | Port to listen on (env: `SENKO_PORT`, default: `3141`) |
| `--host <ADDR>` | Bind address, e.g. `0.0.0.0` or `192.168.1.5` (env: `SENKO_HOST`, default: `127.0.0.1`) |

## Docker

### Dockerfile

```dockerfile
FROM debian:bookworm-slim
ARG SENKO_VERSION=0.10.0
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
  && rm -rf /var/lib/apt/lists/* \
  && case "${TARGETARCH}" in \
       amd64) TARGET="x86_64-unknown-linux-musl" ;; \
       arm64) TARGET="aarch64-unknown-linux-musl" ;; \
       *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
     esac \
  && curl -fsSL "https://github.com/hisamekms/senko/releases/download/v${SENKO_VERSION}/senko-v${SENKO_VERSION}-${TARGET}.tar.gz" \
     | tar xz -C /usr/local/bin senko
WORKDIR /project
ENTRYPOINT ["senko"]
```

> **Note**: `TARGETARCH` is automatically set by Docker BuildKit based on the build platform. This Dockerfile supports both `amd64` and `arm64`.

### Build and run

```bash
# Build the image
docker build -t senko .

# Run a one-off command
docker run --rm -v "$(pwd)/.senko:/project/.senko" senko task list

# Start the API server
docker run --rm -p 3142:3142 \
  -v "$(pwd)/.senko:/project/.senko" \
  senko serve --host 0.0.0.0
```

### Data persistence with volume mounts

senko stores its SQLite database and configuration in the `.senko/` directory. Mount this directory as a volume to persist data across container runs:

```
-v "$(pwd)/.senko:/project/.senko"
```

This mount includes:
- `tasks.db` – the SQLite database
- `config.toml` – hook and workflow configuration

Without a volume mount, all data is lost when the container stops.

## Hooks – Automatic actions on task and contract state changes

Hooks are shell commands that run automatically when CLI / server commands change task or contract state. They fire inline (no daemon required), so they never block the CLI by default. Each hook is a named entry and its behavior (sync vs async, pre vs post, abort vs warn on failure, required env vars, selection-result filter) is declared on the definition itself. See the full schema in [Configuration Reference → Hooks](CONFIGURATION.md#hooks).

### Configuration

Hooks live under a runtime-specific section matching the binary you run:

- `[cli.<action>.hooks.<name>]` — local CLI (not `senko serve` / `senko serve --proxy`)
- `[server.remote.<action>.hooks.<name>]` — direct server (`senko serve`)
- `[server.relay.<action>.hooks.<name>]` — relay server (`senko serve --proxy`)
- `[workflow.<stage>.hooks.<name>]` — workflow stages driven by the Claude Code skill

The action set for the CLI / server runtimes is fixed. Task actions: `task_add` / `task_ready` / `task_start` / `task_complete` / `task_cancel` / `task_select`. Contract actions: `contract_add` / `contract_edit` / `contract_delete` / `contract_dod_check` / `contract_dod_uncheck` / `contract_note_add`. A `sync`+`pre` hook with `on_failure = "abort"` on a contract action cancels the `senko contract <verb>` command the same way it cancels a task state transition.

Create `.senko/config.toml` to define hooks:

```toml
[cli.task_add.hooks.notify]
command = "echo 'New task' | notify-send -"

[cli.task_ready.hooks.webhook]
command = "curl -X POST https://example.com/ready"

[cli.task_start.hooks.slack]
command = "slack-notify started"

[cli.task_complete.hooks.webhook]
command = "curl -X POST https://example.com/webhook"

[cli.task_cancel.hooks.log]
command = "echo canceled"
```

Multiple hooks per action use separate named entries. Each entry supports `enabled`, `when`, `mode`, `on_failure`, and `env_vars`:

```toml
[cli.task_complete.hooks.notify]
command = "notify-send 'Done'"

[cli.task_complete.hooks.webhook]
command = "curl https://example.com/done"

[[cli.task_complete.hooks.webhook.env_vars]]
name = "WEBHOOK_URL"
required = true
```

| Action | Trigger |
|--------|---------|
| `task_add` | `senko task add` creates a new task |
| `task_ready` | `senko task ready` transitions a task from draft to todo |
| `task_start` | `senko task start` or `senko task next` starts a task |
| `task_complete` | `senko task complete` completes a task |
| `task_cancel` | `senko task cancel` cancels a task |
| `task_select` | `senko task next` selects a task or finds none. Filter via `on_result = "selected"` / `"none"` / `"any"` (`"any"` is the default). `on_result = "none"` replaces the old `on_no_eligible_task` event. |
| `contract_add` | `senko contract add` creates a contract |
| `contract_edit` | `senko contract edit` updates a contract |
| `contract_delete` | `senko contract delete` removes a contract |
| `contract_dod_check` | `senko contract dod check` marks a contract DoD item |
| `contract_dod_uncheck` | `senko contract dod uncheck` unmarks a contract DoD item |
| `contract_note_add` | `senko contract note add` appends a note to a contract |

Hooks receive the full event payload as JSON on **stdin** and are executed via `sh -c`. For contract actions, the contract id and any DoD index live inside the `event.contract` payload on stdin — they are not auto-injected as environment variables. Opt-in env vars can still be declared per hook via `env_vars`.

### Testing hooks

Use `senko hooks test <event_name> [task_id]` to fire a single hook synchronously using a real or sample payload. Valid event names: `task_add`, `task_ready`, `task_start`, `task_complete`, `task_cancel`, `task_select`, `contract_add`, `contract_edit`, `contract_delete`, `contract_dod_check`, `contract_dod_uncheck`, `contract_note_add`.

### Event Payload

The JSON object passed to hooks on stdin (the "hook envelope"):

```json
{
  "runtime": "cli",
  "backend": {
    "type": "sqlite",
    "db_file_path": "/path/to/project/.senko/senko.db"
  },
  "project": {
    "id": 1,
    "name": "default"
  },
  "user": {
    "id": 1,
    "name": "default"
  },
  "event": {
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "event": "task_complete",
    "timestamp": "2026-03-24T12:00:00Z",
    "from_status": "in_progress",
    "task": {
      "id": 7,
      "project_id": 1,
      "title": "Implement webhook handler",
      "background": null,
      "description": "Add webhook endpoint for external integrations",
      "plan": null,
      "priority": "P1",
      "status": "completed",
      "assignee_session_id": null,
      "assignee_user_id": null,
      "created_at": "2026-03-24T10:00:00Z",
      "updated_at": "2026-03-24T12:00:00Z",
      "started_at": "2026-03-24T10:30:00Z",
      "completed_at": "2026-03-24T12:00:00Z",
      "canceled_at": null,
      "cancel_reason": null,
      "branch": "feature/webhook",
      "pr_url": "https://github.com/org/repo/pull/42",
      "metadata": null,
      "definition_of_done": [
        { "content": "Write unit tests", "checked": true },
        { "content": "Update API docs", "checked": true }
      ],
      "in_scope": ["REST endpoint"],
      "out_of_scope": ["GraphQL support"],
      "tags": ["backend", "api"],
      "dependencies": [3, 5]
    },
    "stats": { "draft": 1, "todo": 3, "in_progress": 1, "completed": 5 },
    "ready_count": 2,
    "unblocked_tasks": [{ "id": 3, "title": "Next task", "priority": "P1", "metadata": null }]
  }
}
```

#### Envelope fields

| Field | Type | Description |
|-------|------|-------------|
| `runtime` | string | `"cli"`, `"server.relay"`, or `"server.remote"` |
| `backend` | object | Backend info (`type`, and backend-specific fields) |
| `project` | object | Project context: `id` (integer) and `name` (string) |
| `user` | object | User context: `id` (integer) and `name` (string) |
| `event` | object | Event payload (see below) |

The `project` and `user` fields reflect the current config. When `[project] name` or `[user] name` is set in `config.toml`, the corresponding name is resolved from the backend. Otherwise, the default record (id=1) is used.

#### `event` fields

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | string | UUID v4 unique identifier |
| `event` | string | Event name (e.g. `"task_add"`, `"task_complete"`, `"task_select"`) |
| `timestamp` | string | ISO 8601 (RFC 3339) timestamp |
| `from_status` | string \| null | Previous status before the transition |
| `task` | object | Full task object (same schema as `senko task get` — see below) |
| `stats` | object | Task count by status (`{"todo": 3, "completed": 5, ...}`) |
| `ready_count` | integer | Number of `todo` tasks with all dependencies met |
| `unblocked_tasks` | array \| null | Tasks newly unblocked by this event (only on `task_complete`) |

#### `task` object

The full task object included in the event payload. Same schema as `senko task get` output.

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Task ID |
| `project_id` | integer | Project ID |
| `title` | string | Task title |
| `background` | string \| null | Background context |
| `description` | string \| null | Task description |
| `plan` | string \| null | Implementation plan |
| `priority` | string | `"P0"` – `"P3"` |
| `status` | string | `"draft"`, `"todo"`, `"in_progress"`, `"completed"`, `"canceled"` |
| `assignee_session_id` | string \| null | Assigned session ID |
| `assignee_user_id` | integer \| null | Assigned user ID |
| `created_at` | string | ISO 8601 timestamp |
| `updated_at` | string | ISO 8601 timestamp |
| `started_at` | string \| null | ISO 8601 timestamp (when task was started) |
| `completed_at` | string \| null | ISO 8601 timestamp (when task was completed) |
| `canceled_at` | string \| null | ISO 8601 timestamp (when task was canceled) |
| `cancel_reason` | string \| null | Cancellation reason |
| `branch` | string \| null | Associated git branch |
| `pr_url` | string \| null | Pull request URL |
| `metadata` | object \| null | Arbitrary JSON metadata (`--metadata` shallow-merges, `--replace-metadata` replaces) |
| `definition_of_done` | array | List of DoD items (see below) |
| `in_scope` | array | In-scope items (strings) |
| `out_of_scope` | array | Out-of-scope items (strings) |
| `tags` | array | Tag strings |
| `dependencies` | array | Dependent task IDs (integers) |

Each item in `definition_of_done`:

| Field | Type | Description |
|-------|------|-------------|
| `content` | string | DoD item description |
| `checked` | boolean | Whether the item is checked |

#### `unblocked_tasks` items

Present only in `task_complete` events when completing a task unblocks other tasks.

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Task ID |
| `title` | string | Task title |
| `priority` | string | `"P0"` – `"P3"` |
| `metadata` | object \| null | Task metadata (arbitrary JSON) |

#### Contract events

`contract_*` events share the outer envelope (`runtime`, `backend`, `project`, `user`, `event`) but replace the inner `task` payload with a `contract` payload. The envelope omits `from_status`, `stats`, `ready_count`, and `unblocked_tasks` — those are task-aggregate-only.

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | string | UUID v4 unique identifier |
| `event` | string | Event name (`contract_add`, `contract_edit`, `contract_delete`, `contract_dod_check`, `contract_dod_uncheck`, `contract_note_add`) |
| `timestamp` | string | ISO 8601 (RFC 3339) timestamp |
| `contract` | object \| null | Full contract object (same schema as `senko contract get`: `id`, `title`, `description`, `definition_of_done`, `tags`, `notes`, `is_completed`, …). `null` only on rare failure paths where the aggregate could not be re-read. |

| Level | Description |
|-------|-------------|
| `INFO` | Normal operations (start, event detection, successful hook execution) |
| `WARN` | Hook returned non-zero exit code |
| `ERROR` | Hook execution failure |

## Environment Variables

All settings follow the precedence: **CLI flag > environment variable > config.toml > default**.

### Server

| Variable | Description | Default |
|----------|-------------|---------|
| `SENKO_PORT` | Port for `web` and `serve` commands | `3141` (web) / `3142` (serve) |
| `SENKO_HOST` | Bind address (e.g. `0.0.0.0`, `192.168.1.5`) | `127.0.0.1` |
| `SENKO_SERVER_PORT` | Port for `serve` command only | `3142` |
| `SENKO_SERVER_HOST` | Bind address for `serve` command only | `127.0.0.1` |
| `SENKO_PROJECT_ROOT` | Project root directory | Auto-detected |
| `SENKO_CONFIG` | Path to config file | `.senko/config.toml` |

### Workflow

| Variable | Description | Default |
|----------|-------------|---------|
| `SENKO_MERGE_VIA` | `direct` or `pr` | `direct` |
| `SENKO_AUTO_MERGE` | `true` or `false` | `true` |
| `SENKO_BRANCH_MODE` | `worktree` or `branch` | `worktree` |
| `SENKO_MERGE_STRATEGY` | `rebase` or `squash` | `rebase` |

### Connection

| Variable | Description | Default |
|----------|-------------|---------|
| `SENKO_CLI_REMOTE_URL` | API server URL (enables HTTP backend instead of SQLite) | _(unset = SQLite)_ |
| `SENKO_CLI_REMOTE_TOKEN` | API token for server authentication | _(unset)_ |
| `SENKO_SERVER_RELAY_URL` | Upstream server URL for relay mode | _(unset)_ |
| `SENKO_SERVER_RELAY_TOKEN` | Authentication token for relay upstream | _(unset)_ |

### Log

| Variable | Description | Default |
|----------|-------------|---------|
| `SENKO_LOG_DIR` | Directory for hook log output | `$XDG_STATE_HOME/senko` |

### Hooks

Hook definitions are not configurable via environment variables. Define them in `.senko/config.toml` under the runtime-specific section (`[cli.<action>.hooks.<name>]`, `[server.remote.<action>.hooks.<name>]`, `[server.relay.<action>.hooks.<name>]`, or `[workflow.<stage>.hooks.<name>]`). See [Configuration Reference → Hooks](CONFIGURATION.md#hooks) for the full schema.

### Example: Docker deployment

```bash
docker run -e SENKO_PORT=8080 \
  -e SENKO_HOST=0.0.0.0 \
  -v "$(pwd)/.senko:/root/.senko" \
  senko serve
```

Mount the project's `.senko` directory so `config.toml`, including any `[server.remote.*.hooks.*]` definitions, is picked up by the server.

## Status Transitions

```
draft → todo → in_progress → completed
                            → canceled
(any active status → canceled)
```
