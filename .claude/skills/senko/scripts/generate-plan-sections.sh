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
EVENTS=$(echo "$CONFIG_JSON" | jq -c '.workflow.events // []')

emit_events() {
  local point="$1"
  echo "$EVENTS" | jq -c --arg p "$point" '.[] | select(.point == $p)' | while IFS= read -r event; do
    local etype
    etype=$(echo "$event" | jq -r '.type')
    case "$etype" in
      command)
        local cmd
        cmd=$(echo "$event" | jq -r '.command')
        echo "- Run: \`${cmd}\`"
        ;;
      prompt)
        local content
        content=$(echo "$event" | jq -r '.content')
        echo "- ${content}"
        ;;
    esac
  done
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

emit_events "pre_start"

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

emit_events "pre_merge"

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

emit_events "post_merge"

# --- Common tail ---
cat <<EOF
- Use \`AskUserQuestion\` to ask the user for completion approval
- Complete the task: \`senko complete ${TASK_ID}\`
EOF

# --- Worktree cleanup ---
if [ "$BRANCH_MODE" = "worktree" ]; then
  echo "- Delete the worktree (using \`/wth\` skill)"
fi
