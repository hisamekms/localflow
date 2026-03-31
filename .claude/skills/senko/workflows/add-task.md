# Add Task

## Normal vs Simple Mode

- **Normal** (`add <description>`): Phase 1 → 2 → 3 → 4 (full workflow)
- **Simple** (`add --simple <description>`): Phase 1 → 3 → 4 (skip planning)

## Procedure

### Phase 1: Create Task (draft)

```bash
senko add --title "<description>"
```

Capture the `id` from JSON output for subsequent phases.

### Phase 2: Planning (clarification loop)

> **Skip this phase in simple mode.**

Plan the task through conversation (do not enter plan mode). Repeat until no open questions remain:

1. Based on the task description and codebase investigation, list **unclear points and decisions needed**
2. If the list is empty, proceed to Phase 3
3. For each item, ask the user via `AskUserQuestion`:
   - Present options for each question
   - Mark at least one option with "(Recommended)" when possible
4. After all questions are resolved, return to step 1 — previous answers may raise new questions

Continue until **no open questions remain**.

### Phase 3: Dependency Discovery

Check existing active tasks for potential dependencies:

```bash
senko list --status todo
senko list --status in_progress
```

Review the list to identify tasks the new task should depend on.

### Phase 4: Finalize Task

**Normal mode:**

1. Update the task with planning results
2. Use `AskUserQuestion` to confirm:
   - Title and content are appropriate
   - Dependencies to add
   - Tags to set
   - Priority (default p2) adjustment
3. Apply confirmed settings:

```bash
senko edit <id> \
  --title "Final title" \
  --description "Planning description" \
  --priority p1 \
  --add-tag backend \
  --add-definition-of-done "Write unit tests" \
  --add-definition-of-done "E2E tests pass"

# Transition to todo
senko ready <id>

# If dependencies exist
senko deps add <id> --on <dep_id>
```

**Simple mode:**

1. Set the user's description as the description
2. Transition to todo:

```bash
senko edit <id> --description "<description>"
senko ready <id>
```

Display the finalized task details to the user.
