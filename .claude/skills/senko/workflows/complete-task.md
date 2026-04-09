# Complete Task

Mark a task as completed. `complete` will fail if any DoD items are unchecked.

```bash
senko get <id>
```

1. Verify the task is in `in_progress` status. If not, inform the user and stop.
2. Check if any DoD items are unchecked (`"checked": false` in JSON, or `[ ]` in text output). If unchecked items exist:
   - Launch the `dod-verifier` agent (via Agent tool) with the task ID and unchecked DoD items
   - Process the subagent's results for each item:
     - **VERIFIED**: `senko dod check <id> <index>`
     - **NEEDS_USER_APPROVAL**: Use `AskUserQuestion` to confirm with the user, then check if approved
     - **NOT_ACHIEVED**: Inform the user that the item is not yet achieved
   - All DoD items must be checked before proceeding to complete
3. Check the workflow configuration (`senko config`):
   - If `merge_via = "pr"`:
     - Ensure `pr_url` is set on the task (`senko edit <id> --pr-url <url>`)
     - Begin PR polling loop:
       1. Run `gh pr view <pr_url> --json state,reviews,comments`
       2. If there are new review comments or requested changes:
          - Address each review comment (fix code, respond to feedback)
          - Push the changes and continue polling
       3. If the PR state is MERGED:
          - Exit the polling loop and proceed to `senko complete <id>`
       4. Otherwise, wait 1 minute and repeat from step 1
     - Use `--skip-pr-check` to bypass the merged check if needed
   - If `merge_via = "direct"` (default): no PR checks are performed

```bash
senko complete <id>
```

Display the completed task info to the user. If there is an associated worktree, remind the user to clean it up.
