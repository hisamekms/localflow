# Execute Task

## Pre-check

> Skip this step if coming from `senko task next` (already validated).

```bash
senko task get <id>
```

- Verify `status` is `todo`. If not, inform the user and stop.
- Check `dependencies` for incomplete tasks. If any, inform the user and stop.

### Build metadata

Run the metadata builder script to read `[workflow.start].metadata_fields` from config:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/build-metadata.sh start
```

Parse the JSON output (`{"resolved": {...}, "prompts": [...]}`):

- If `prompts` array is non-empty, ask the user each prompt question using `AskUserQuestion`. Merge user answers into `resolved`.
- If `resolved` is empty (no keys) after merging, do NOT pass `--metadata`.

Then transition:

```bash
senko task start <id> --metadata '<final-metadata-json>'
```

Omit `--metadata` entirely if there are no metadata fields to pass.

## Execution Steps

> **Terminal-task redirect**: if the task carries the `contract-terminal` tag, do NOT follow the rest of this file. Switch to `${CLAUDE_SKILL_DIR}/workflows/contract-terminal.md` — terminal tasks verify the linked Contract rather than planning and implementing code.

### Step 1: Review Task

Read full task info from `senko task get <id>` output: `description`, `plan`, `definition_of_done`, `in_scope`, `out_of_scope`, and `contract_id`.

**If `contract_id` is set (non-null)**, also load the Contract context — these notes are the shared memory across every sub-task that is linked to this Contract:

```bash
senko contract get <contract_id>
senko contract note list <contract_id>
```

Surface the Contract's title, description, DoD checklist, and the full note list into the assistant's working context before moving on. Prior sessions may have recorded decisions, gotchas, or scope clarifications there.

### Step 2: Create Worktree

Use the `branch` field from `senko task get <id>` as the branch name. If `branch` is not set (non-repo task), skip worktree creation and proceed to Step 3. Use the `/wth` skill to create a worktree.

### Step 3: Plan Mode

Use `EnterPlanMode` to create an implementation plan. Investigate the codebase based on the task's description.

Before creating the plan, generate the workflow-specific sections by running:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/generate-plan-sections.sh <id>
```

The script outputs three sections: **Pre-start**, **Finalization**, and **Post-completion**. Include all three sections verbatim in the plan.

Wait for the user to approve the plan.

## Contract note recording

> This subsection applies **only** when the task has a `contract_id` set. Skip entirely for Contract-less tasks.

Notes are the shared memory between sibling sub-tasks and the terminal task. Record one — via the command below — at each of the following moments. Each note should be 1–2 sentences; before adding, re-read `senko contract note list <contract_id>` and skip the write if the same observation is already present.

```bash
senko contract note add <contract_id> --content "<text>" --source-task <task_id>
```

1. **Major design decisions**: as soon as a non-trivial technical choice is made (library or pattern selection, architectural change, non-obvious trade-off), write a note naming the decision and the reason. Do this during planning or implementation, whichever is earlier.
2. **Pitfalls / surprises**: when a non-obvious bug, undocumented constraint, or reproducible gotcha is hit, record it so the next sibling doesn't repeat the loss. One sentence of what went wrong + one sentence of what to do about it is enough.
3. **Task-completion summary**: just before running `senko task complete <id>` in the Finalization section, add a short summary note — what was done, what is explicitly left for other sub-tasks or the terminal, and any cross-cutting invariants newly established.
