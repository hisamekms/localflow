# DoD Check/Uncheck

Manage the checked state of Definition of Done items. Indices are **1-based** (first item = 1).

## `dod check <task_id> <index>`

Mark a DoD item as done:

```bash
senko dod check <task_id> <index>
```

## `dod uncheck <task_id> <index>`

Unmark a DoD item:

```bash
senko dod uncheck <task_id> <index>
```

## Display format

DoD items show their check state in task output:

- **Text output**: `[x] Write unit tests` / `[ ] E2E tests pass`
- **JSON output**: `{"content": "Write unit tests", "checked": true}`
