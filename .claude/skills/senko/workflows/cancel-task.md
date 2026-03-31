# Cancel Task

Cancel a task. Ask the user for a cancellation reason if not provided.

```bash
senko get <id>
```

Verify the task is not already in a terminal state (`completed` or `canceled`). If it is, inform the user and stop.

```bash
senko cancel <id> --reason "User-provided reason"
```

Display the canceled task info to the user. If there is an associated worktree, remind the user to clean it up.
