#!/usr/bin/env bash
set -euo pipefail

TASK_ID="${1:?Usage: generate-plan-sections.sh <task-id>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

CONFIG_JSON=$(senko config)
COMPLETION_MODE=$(echo "$CONFIG_JSON" | jq -r '.workflow.completion_mode')
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
if [ "$COMPLETION_MODE" = "merge_then_complete" ]; then
  if [ "$MERGE_STRATEGY" = "squash" ]; then
    echo "- Squash merge the branch into main (all DoD items must be checked before this step):"
    echo "  \`bash \${CLAUDE_SKILL_DIR}/scripts/squash-merge.sh <branch-name>\`"
  else
    echo "- Rebase merge the branch into main using the rebase-merge script (all DoD items must be checked before this step):"
    echo "  \`bash \${CLAUDE_SKILL_DIR}/scripts/rebase-merge.sh <branch-name>\`"
  fi
elif [ "$COMPLETION_MODE" = "pr_then_complete" ]; then
  if [ "$AUTO_MERGE" = "true" ]; then
    echo "- Create PR and merge (all DoD items must be checked before this step)"
  else
    echo "- Create PR (all DoD items must be checked before this step)"
  fi
  echo "- After creating the PR, save the PR URL: \`senko edit ${TASK_ID} --pr-url <pr_url>\`"
  if [ "$AUTO_MERGE" = "false" ]; then
    echo "- Request review and wait for approval before merging"
  fi
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
