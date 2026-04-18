# senko CLI Reference

All commands use `senko`. Default output is JSON. The `--output` flag is **global** and must precede the subcommand.

```bash
# Add a task (created in draft status)
senko add --title "Title" --priority p1 --description "Description"

# List tasks (with filters)
senko --output text list
senko list --status todo
senko list --status in_progress
senko list --status todo --status in_progress  # combine multiple filters
senko list --ready                 # todo tasks with all deps completed
senko list --tag backend

# Get task details (JSON only, no text output)
senko get <id>

# Auto-select next task (highest-priority ready task → in_progress)
senko next

# Status transitions (dedicated commands)
senko ready <id>                       # draft → todo
senko start <id>                       # todo → in_progress
senko complete <id>                    # in_progress → completed (fails if unchecked DoD)
senko complete <id> --skip-pr-check    # bypass PR merge/review checks
senko cancel <id> --reason "Reason text"  # any active → canceled

# Edit task fields (no status changes — use dedicated commands above)
senko edit <id> --title "New Title" --add-tag backend
senko edit <id> --add-definition-of-done "Write unit tests"
senko edit <id> --pr-url "https://github.com/org/repo/pull/42"
senko edit <id> --plan-file /path/to/plan.md  # read plan from file

# Metadata
senko edit <id> --metadata '{"key":"val"}'         # shallow-merge into existing metadata
senko edit <id> --replace-metadata '{"key":"val"}'  # replace entire metadata
senko edit <id> --clear-metadata                    # remove all metadata

# Definition of Done (DoD) check/uncheck (1-based index)
senko dod check <task_id> <index>      # mark DoD item as done
senko dod uncheck <task_id> <index>    # unmark DoD item

# Dependencies
senko deps add <task_id> --on <dep_id>
senko deps remove <task_id> --on <dep_id>
senko deps set <task_id> --on <dep_id1> <dep_id2>
senko deps list <task_id>

# Contract — independent aggregate shared by sub-tasks of a split
senko contract add --title "Title" [--description "..."] [--definition-of-done "..." ...] [--tag ...] [--metadata '{...}']
senko contract add --from-json                       # read JSON from stdin
senko contract add --from-json-file <path>
senko contract list [--tag <tag>]                    # filtered list
senko contract get <id>
senko contract edit <id> --title "New" --add-tag demo --add-definition-of-done "Verify X"
senko contract edit <id> --description "..." | --clear-description
senko contract edit <id> --metadata '{"k":"v"}' | --replace-metadata '{...}' | --clear-metadata
senko contract edit <id> --set-tags t1 t2 | --add-tag t | --remove-tag t
senko contract edit <id> --set-definition-of-done "a" "b" | --add-definition-of-done "c" | --remove-definition-of-done "a"
senko contract delete <id>

# Contract DoD (1-based index, same semantics as task DoD)
senko contract dod check <contract_id> <index>
senko contract dod uncheck <contract_id> <index>

# Contract notes (append-only, timestamped by server)
senko contract note add <contract_id> --content "..." [--source-task <task_id>]
senko contract note list <contract_id>

# Link a task to a contract (reuses the existing `edit` command)
senko edit <task_id> --contract <contract_id>        # set link
senko edit <task_id> --clear-contract                # remove link

# Configuration
senko config                           # show current configuration
senko config --init                    # generate template config.toml
```

## Important CLI details

- `--output text|json` and `--dry-run` are **global flags** — place them before the subcommand: `senko --output text list`, `senko --dry-run ready 1`
- `--dry-run` shows what would happen without actually executing the command. Available for all state-changing commands (`add`, `edit`, `ready`, `start`, `complete`, `cancel`, `next`, `deps add/remove/set`, `dod check/uncheck`). Read-only commands (`list`, `get`, `deps list`) ignore it.
- **DoD items have a checked state.** `complete` will fail if any DoD items are unchecked. Use `dod check <task_id> <index>` (1-based index) to mark items before completing. Tasks with no DoD items can complete freely.
- **Status transitions use dedicated commands**, not `edit --status`:
  - `ready`: draft → todo
  - `start`: todo → in_progress
  - `complete`: in_progress → completed
  - `cancel`: any active status → canceled
- `get` only outputs JSON (no `--output text` support)
- New tasks start in `draft` status. Status transitions: draft → todo → in_progress → completed. Any active status → canceled.
- Priority levels: `p0` (highest) through `p3` (lowest). Default is `p2`.
- **Workflow configuration** (`[workflow]` in `.senko/config.toml`):
  - `merge_via`: `direct` (default) or `pr`
  - `auto_merge`: `true` (default) / `false` — applies only to `merge_via = "direct"`
  - `branch_mode`: `worktree` (default) or `branch`
  - `merge_strategy`: `rebase` (default) or `squash`
  - `branch_template`: optional branch name template. Supported variables:
    - `{{id}}` — task ID
    - `{{slug}}` — kebab-case slug derived from the task title
    - `{{context.<key>}}` — resolved from session context at branch-setting time. If the value is unavailable, the user is asked via `AskUserQuestion`.
    - `{{<name>:<opt1>|<opt2>|...}}` — enum variable. The skill infers the value from the task content. If unclear, the user is asked to choose. Example: `{{prefix:feat|fix|chore}}`
    - Examples: `task/{{id}}-{{slug}}`, `{{prefix:feat|fix|chore}}/{{id}}-{{slug}}`
  - Stage configs (`add`, `start`, `branch`, `plan`, `implement`, `merge`, `pr`, `complete`, `branch_cleanup`): each stage supports `instructions` (list of text), `pre_hooks` (list of hooks), `post_hooks` (list of hooks). Hooks can be a simple string (shell command) or `{command, prompt, on_failure}`.
  - `metadata_fields`: available on all stages (`add`, `start`, `branch`, `plan`, `implement`, `merge`, `pr`, `complete`, `branch_cleanup`). Each field has `key`, `source` (`env`, `value`, `prompt`, `command`), optional `default`, and `required` flag. `prompt` source fields are collected via `AskUserQuestion`. Values are shallow-merged into existing metadata.
  - **Metadata update semantics**: `--metadata` performs a shallow merge (top-level keys only: add/overwrite existing, null deletes key, unmentioned keys preserved). `--replace-metadata` replaces entirely. `--clear-metadata` removes all metadata.
  - `add.default_dod` / `add.default_tags` / `add.default_priority`: defaults for new tasks
  - `plan.required_sections`: required sections in implementation plans
  - When `merge_via = "pr"`, `complete` requires `pr_url` to be set and the PR to be merged (checked via `gh`). Use `--skip-pr-check` to bypass.

## Contract semantics

- **No status field.** A contract is "completed" iff it has at least one DoD item and every DoD item is checked. Empty DoD always returns false. The `senko contract get` JSON response includes a computed `is_completed` boolean for convenience.
- **Notes are append-only** and server-timestamped. Each note carries `content`, optional `source_task_id`, and `created_at`. There is no update / delete; corrections are expressed by adding a new note.
- **Task ↔ Contract link is directional.** A task carries an `Option<i64> contract_id`; contracts do not hold a list of task IDs. Use `senko list --tag <contract-terminal or other>` combined with `senko get | jq .contract_id` if you need to enumerate siblings — or query the Contract notes (which the skill populates) to find authoritative source tasks.
- **`contract-terminal` tag** is a skill-level convention marking a task whose sole job is to verify the Contract's DoD items at the end of a split. See `workflows/contract-terminal.md`.

