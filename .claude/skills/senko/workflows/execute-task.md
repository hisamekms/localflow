# Execute Task

## Pre-check

> Skip this step if coming from `senko next` (already validated).

```bash
senko get <id>
```

- Verify `status` is `todo`. If not, inform the user and stop.
- Check `dependencies` for incomplete tasks. If any, inform the user and stop.

Then transition:

```bash
senko start <id>
```

## Execution Steps

### Step 1: Review Task

Read full task info from `senko get <id>` output: `description`, `plan`, `definition_of_done`, `in_scope`, `out_of_scope`.

### Step 2: Create Worktree

Generate a branch name from the task title. Use the `/wth` skill to create a worktree.

### Step 3: Plan Mode

Use `EnterPlanMode` to create an implementation plan. Investigate the codebase based on the task's description.

Before creating the plan, generate the workflow-specific Pre-start and Post-completion sections by running:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/generate-plan-sections.sh <id>
```

Include the script's output verbatim in the plan as the Pre-start and Post-completion sections.

Wait for the user to approve the plan.
