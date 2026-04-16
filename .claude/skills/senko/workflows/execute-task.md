# Execute Task

## Pre-check

> Skip this step if coming from `senko next` (already validated).

```bash
senko get <id>
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
senko start <id> --metadata '<final-metadata-json>'
```

Omit `--metadata` entirely if there are no metadata fields to pass.

## Execution Steps

### Step 1: Review Task

Read full task info from `senko get <id>` output: `description`, `plan`, `definition_of_done`, `in_scope`, `out_of_scope`.

### Step 2: Create Worktree

Generate a branch name from the task title. Use the `/wth` skill to create a worktree.

### Step 3: Plan Mode

Use `EnterPlanMode` to create an implementation plan. Investigate the codebase based on the task's description.

Before creating the plan, generate the workflow-specific sections by running:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/generate-plan-sections.sh <id>
```

The script outputs three sections: **Pre-start**, **Finalization**, and **Post-completion**. Include all three sections verbatim in the plan.

Wait for the user to approve the plan.
