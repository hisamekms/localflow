# senko CLI Reference

All commands use `senko`. Default output is JSON. The `--output` flag is **global** and must precede the subcommand.

```bash
# Add a task (created in draft status)
senko add --title "Title" --priority p1 --description "Description"

# List tasks (with filters)
senko --output text list
senko list --status todo
senko list --status in_progress
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

# Definition of Done (DoD) check/uncheck (1-based index)
senko dod check <task_id> <index>      # mark DoD item as done
senko dod uncheck <task_id> <index>    # unmark DoD item

# Dependencies
senko deps add <task_id> --on <dep_id>
senko deps remove <task_id> --on <dep_id>
senko deps set <task_id> --on <dep_id1> <dep_id2>
senko deps list <task_id>

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
  - `branch_template`: optional branch name template (e.g., `task/{{id}}-{{slug}}`)
  - Stage configs (`add`, `start`, `branch`, `plan`, `implement`, `merge`, `pr`, `complete`, `branch_cleanup`): each stage supports `instructions` (list of text), `pre_hooks` (list of hooks), `post_hooks` (list of hooks). Hooks can be a simple string (shell command) or `{command, prompt, on_failure}`.
  - `metadata_fields`: available on all stages (`add`, `start`, `branch`, `plan`, `implement`, `merge`, `pr`, `complete`, `branch_cleanup`). Each field has `key`, `source` (`env`, `value`, `prompt`, `command`), optional `default`, and `required` flag. `prompt` source fields are collected via `AskUserQuestion`.
  - `add.default_dod` / `add.default_tags` / `add.default_priority`: defaults for new tasks
  - `plan.required_sections`: required sections in implementation plans
  - When `merge_via = "pr"`, `complete` requires `pr_url` to be set and the PR to be merged (checked via `gh`). Use `--skip-pr-check` to bypass.

