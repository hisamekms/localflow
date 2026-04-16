# Add Task

## Normal vs Simple Mode

- **Normal** (`add <description>`): Phase 1 → 2 → 3 → 4 (full workflow)
- **Simple** (`add --simple <description>`): Create draft → set description → `ready` (no planning)

## Procedure

### Phase 1: Planning & Split Decision

> **Skip this phase in simple mode.**

Investigate the task through codebase exploration and conversation. Repeat until no open questions remain:

1. Based on the task description and codebase investigation, list **unclear points and decisions needed**
2. If the list is empty, proceed to split decision
3. For each item, ask the user via `AskUserQuestion`:
   - Present options for each question
   - Mark at least one option with "(Recommended)" when possible
4. After all questions are resolved, return to step 1 — previous answers may raise new questions

Once all questions are resolved, **decide whether to split the task**. Consider these heuristics:

- **Split** when:
  - The task contains independently workable parts (can be done in parallel by separate sessions)
  - Changes span multiple modules or layers with no code-level coupling
  - The task mixes distinct concerns (e.g., auto-fixable lints vs. manual refactoring)
  - Subtasks have different risk levels or review requirements
- **Keep as one** when:
  - All changes are tightly coupled and must be committed together
  - The task is small enough that splitting adds overhead without benefit
  - Splitting would create tasks that are trivial on their own

If splitting, define the sub-tasks with their titles and relationships. Ask the user via `AskUserQuestion` to confirm the proposed split.

### Phase 2: Draft Creation

#### Build add metadata

Run the metadata builder script to read `[workflow.add].metadata_fields` from config:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/build-metadata.sh add
```

Parse the JSON output (`{"resolved": {...}, "prompts": [...]}`):

- If `prompts` array is non-empty, ask the user each prompt question using `AskUserQuestion`. Merge user answers into `resolved`.
- If `resolved` is non-empty (has keys), pass `--metadata '<json>'` to each `senko add` call below.

#### Create draft tasks

Create one or multiple draft tasks based on Phase 1 results.

**Single task:**

```bash
senko add --title "<title>" --assignee-user-id self --metadata '<metadata-json>'
```

**Multiple tasks (split):**

```bash
senko add --title "<sub-task 1 title>" --assignee-user-id self --metadata '<metadata-json>'
senko add --title "<sub-task 2 title>" --assignee-user-id self --metadata '<metadata-json>'
# ...
```

Omit `--metadata` entirely if there are no metadata fields to pass.

Capture the `id` from each JSON output for subsequent phases.

### Phase 3: Dependency Setup

Set up dependencies between tasks:

1. Check existing active tasks for potential dependencies:

```bash
senko list --status todo --status in_progress
```

2. For **split tasks**: set dependencies between the new tasks where one must complete before another can start (sequential relationships). Tasks that can run in parallel should have no dependency between them.

3. For **all new tasks**: check if any depend on existing active tasks.

```bash
senko deps add <task_id> --on <dep_id>
```

### Phase 4: Finalize Tasks

For each created task:

1. Update with planning results
2. Use `AskUserQuestion` to confirm:
   - Title and content are appropriate
   - Dependencies are correct
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
```

4. **Branch setting** (before `senko ready`):
   - Determine whether the task involves repository operations (code changes, file edits, configuration changes, etc.) based on the task's title and description. If unclear, use `AskUserQuestion` to ask the user.
   - If the task does NOT involve repository operations (e.g., investigation only, external service setup), skip branch setting.
   - If the task involves repository operations:
     1. Read `branch_template` from `senko config`. If not set, use `{{id}}-{{slug}}` as the default template.
     2. Resolve template variables:
        - `{{id}}` → task ID
        - `{{slug}}` → kebab-case slug derived from the task title
        - `{{context.<key>}}` → resolve from session context. If unavailable, use `AskUserQuestion` to ask the user for the value.
        - `{{<name>:<opt1>|<opt2>|...}}` → enum variable. Infer the appropriate value from the task's title and description (e.g., new feature → `feat`, bug fix → `fix`, maintenance → `chore`). If unclear, use `AskUserQuestion` to present the options.
     3. Set it: `senko edit <id> --branch <branch-name>`

5. Transition to todo:

```bash
senko ready <id>
```

Display the finalized task details (or task graph if multiple) to the user.

---

**Simple mode procedure:**

1. Create draft: `senko add --title "<description>"`
2. Set description: `senko edit <id> --description "<description>"`
3. **Branch setting**: Same as Phase 4 step 4 above.
4. Transition: `senko ready <id>`
