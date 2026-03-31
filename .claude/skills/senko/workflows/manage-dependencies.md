# Manage Dependencies

Handle dependency operations based on the subcommand.

## `deps add <task_id> --on <dep_id>`

Add a dependency. senko will reject circular and self-dependencies automatically.

```bash
senko deps add <task_id> --on <dep_id>
```

## `deps remove <task_id> --on <dep_id>`

Remove a dependency.

```bash
senko deps remove <task_id> --on <dep_id>
```

## `deps list <task_id>`

Show all tasks that the given task depends on.

```bash
senko deps list <task_id>
```

Display results to the user. If there are unresolved dependencies, note which ones are blocking.
