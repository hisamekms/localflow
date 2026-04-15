#!/usr/bin/env bash
set -euo pipefail

TASK_ID="${1:?Usage: generate-plan-sections.sh <task-id>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SENKO_BIN="${SENKO_BIN:-senko}"

CONFIG_JSON=$("$SENKO_BIN" config)
MERGE_VIA=$(echo "$CONFIG_JSON" | jq -r '.workflow.merge_via')
AUTO_MERGE=$(echo "$CONFIG_JSON" | jq -r '.workflow.auto_merge')
BRANCH_MODE=$(echo "$CONFIG_JSON" | jq -r '.workflow.branch_mode')
MERGE_STRATEGY=$(echo "$CONFIG_JSON" | jq -r '.workflow.merge_strategy')

# Emit instructions for a workflow stage as bullet points.
# Usage: emit_instructions <stage>
emit_instructions() {
  local stage="$1"
  local count
  count=$(echo "$CONFIG_JSON" | jq --arg s "$stage" '.workflow[$s].instructions // [] | length')
  for i in $(seq 0 $((count - 1))); do
    local text
    text=$(echo "$CONFIG_JSON" | jq -r --arg s "$stage" ".workflow[\$s].instructions[$i]")
    echo "- ${text}"
  done
}

# Emit hooks (pre_hooks or post_hooks) for a workflow stage.
# Usage: emit_hooks <stage> <pre_hooks|post_hooks>
# HookDef is either a simple string or {"command": ..., "prompt": ..., "on_failure": ...}
emit_hooks() {
  local stage="$1"
  local phase="$2"
  local count
  count=$(echo "$CONFIG_JSON" | jq --arg s "$stage" --arg p "$phase" '.workflow[$s][$p] // [] | length')
  for i in $(seq 0 $((count - 1))); do
    local hook
    hook=$(echo "$CONFIG_JSON" | jq -c --arg s "$stage" --arg p "$phase" ".workflow[\$s][\$p][$i]")
    local hook_type
    hook_type=$(echo "$hook" | jq -r 'type')
    if [ "$hook_type" = "string" ]; then
      echo "- Run: \`${hook//\"/}\`"
    else
      local cmd prompt
      cmd=$(echo "$hook" | jq -r '.command // empty')
      prompt=$(echo "$hook" | jq -r '.prompt // empty')
      if [ -n "$cmd" ]; then
        echo "- Run: \`${cmd}\`"
      fi
      if [ -n "$prompt" ]; then
        echo "- ${prompt}"
      fi
    fi
  done
}

# Emit metadata collection step for a workflow stage if metadata_fields is configured.
# Usage: emit_metadata_step <stage> <task-id>
emit_metadata_step() {
  local stage="$1"
  local task_id="$2"
  local count
  count=$(echo "$CONFIG_JSON" | jq --arg s "$stage" '.workflow[$s].metadata_fields // [] | length')
  if [ "$count" -gt 0 ]; then
    echo "- Collect metadata for \`${stage}\` stage:"
    echo "  1. Run \`bash \${CLAUDE_SKILL_DIR}/scripts/build-metadata.sh ${stage}\`"
    echo "  2. If \`prompts\` array is non-empty, ask the user each prompt using \`AskUserQuestion\`"
    echo "  3. Merge answers into \`resolved\`, then shallow-merge into existing task metadata"
    echo "  4. Save: \`senko edit ${task_id} --metadata '<merged-json>'\`"
  fi
}

# --- Pre-start ---
cat <<EOF
# Pre-start
- Save this plan to the task:
  1. Write the full approved plan text to a temporary file (e.g., \`/tmp/senko-plan-${TASK_ID}.md\`)
  2. Run \`senko edit ${TASK_ID} --plan-file /tmp/senko-plan-${TASK_ID}.md\`
  3. Delete the temporary file
- This must be done before starting implementation.
EOF

emit_metadata_step "implement" "$TASK_ID"
emit_instructions "implement"
emit_hooks "implement" "pre_hooks"

# --- Post-completion ---
cat <<'HEADER'

# Post-completion
- When implementation is done, verify DoD items using the dod-verifier subagent:
  1. Run `senko get <id>` and check `definition_of_done` for unchecked items
  2. Launch the `dod-verifier` agent (via Agent tool) with the task ID and unchecked DoD items
  3. Process the subagent's results for each item:
     - **VERIFIED**: `senko dod check <id> <index>`
     - **NEEDS_USER_APPROVAL**: Use `AskUserQuestion` to confirm with the user, then check if approved
     - **NOT_ACHIEVED**: Go back and implement the missing item, then re-verify
  4. All DoD items must be checked before proceeding
HEADER

emit_metadata_step "merge" "$TASK_ID"
emit_hooks "merge" "pre_hooks"
emit_instructions "merge"

# --- Merge/PR step ---
if [ "$MERGE_VIA" = "direct" ]; then
  if [ "$MERGE_STRATEGY" = "squash" ]; then
    echo "- Squash merge the branch into main (all DoD items must be checked before this step):"
    echo "  \`bash \${CLAUDE_SKILL_DIR}/scripts/squash-merge.sh <branch-name>\`"
  else
    echo "- Rebase merge the branch into main using the rebase-merge script (all DoD items must be checked before this step):"
    echo "  \`bash \${CLAUDE_SKILL_DIR}/scripts/rebase-merge.sh <branch-name>\`"
  fi
  echo "- If the merge script exits with code 10 (primary worktree has uncommitted changes), use \`AskUserQuestion\` to inform the user and ask them to clean up the primary worktree before retrying"
  echo "- If the merge script exits with code 11 (rebase conflict), use \`AskUserQuestion\` to inform the user about the conflict and ask how to proceed (manual resolution or abort)"
elif [ "$MERGE_VIA" = "pr" ]; then
  cat <<PREOF
- Create PR (all DoD items must be checked before this step)
- After creating the PR, save the PR URL: \`senko edit ${TASK_ID} --pr-url <pr_url>\`
- Begin PR polling loop:
  1. Run \`gh pr view <pr_url> --json state,reviews,comments\`
  2. If there are new review comments or requested changes:
     - Address each review comment (fix code, respond to feedback)
     - Push the changes and continue polling
  3. If the PR state is MERGED:
     - Exit the polling loop and proceed to completion
  4. Otherwise, wait 1 minute and repeat from step 1
PREOF
fi

emit_hooks "merge" "post_hooks"

# --- Complete step ---
emit_metadata_step "complete" "$TASK_ID"
emit_hooks "complete" "pre_hooks"
emit_instructions "complete"

cat <<EOF
- Use \`AskUserQuestion\` to ask the user for completion approval
- Complete the task: \`senko complete ${TASK_ID}\`
EOF

# --- Branch cleanup ---
emit_metadata_step "branch_cleanup" "$TASK_ID"
emit_hooks "branch_cleanup" "pre_hooks"
emit_instructions "branch_cleanup"

if [ "$BRANCH_MODE" = "worktree" ]; then
  echo "- Delete the worktree (using \`/wth\` skill)"
fi

emit_hooks "branch_cleanup" "post_hooks"
