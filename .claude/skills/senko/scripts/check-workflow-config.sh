#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SENKO_BIN="${SENKO_BIN:-senko}"
ERRORS=0

CONFIG_JSON=$("$SENKO_BIN" config)

error() {
  echo "ERROR: $1" >&2
  ERRORS=$((ERRORS + 1))
}

warn() {
  echo "WARNING: $1" >&2
}

# Validate merge_via
MERGE_VIA=$(echo "$CONFIG_JSON" | jq -r '.workflow.merge_via')
case "$MERGE_VIA" in
  direct|pr) ;;
  *) error "invalid merge_via: $MERGE_VIA (expected: direct, pr)" ;;
esac

# Validate branch_mode
BRANCH_MODE=$(echo "$CONFIG_JSON" | jq -r '.workflow.branch_mode')
case "$BRANCH_MODE" in
  worktree|branch) ;;
  *) error "invalid branch_mode: $BRANCH_MODE (expected: worktree, branch)" ;;
esac

# Validate merge_strategy
MERGE_STRATEGY=$(echo "$CONFIG_JSON" | jq -r '.workflow.merge_strategy')
case "$MERGE_STRATEGY" in
  rebase|squash) ;;
  *) error "invalid merge_strategy: $MERGE_STRATEGY (expected: rebase, squash)" ;;
esac

# Validate auto_merge is boolean
AUTO_MERGE=$(echo "$CONFIG_JSON" | jq -r '.workflow.auto_merge')
case "$AUTO_MERGE" in
  true|false) ;;
  *) error "invalid auto_merge: $AUTO_MERGE (expected: true, false)" ;;
esac

# Validate workflow events
VALID_POINTS="pre_start pre_merge post_merge pre_complete post_complete pre_pr post_pr"
EVENT_COUNT=$(echo "$CONFIG_JSON" | jq '.workflow.events // [] | length')

for i in $(seq 0 $((EVENT_COUNT - 1))); do
  EVENT=$(echo "$CONFIG_JSON" | jq -c ".workflow.events[$i]")
  ETYPE=$(echo "$EVENT" | jq -r '.type')
  POINT=$(echo "$EVENT" | jq -r '.point')

  case "$ETYPE" in
    command)
      CMD=$(echo "$EVENT" | jq -r '.command // empty')
      if [ -z "$CMD" ]; then
        error "event #$((i+1)) at point '$POINT': command type requires 'command' field"
      fi
      ;;
    prompt)
      CONTENT=$(echo "$EVENT" | jq -r '.content // empty')
      if [ -z "$CONTENT" ]; then
        error "event #$((i+1)) at point '$POINT': prompt type requires 'content' field"
      fi
      ;;
    *)
      error "event #$((i+1)): invalid type '$ETYPE' (expected: command, prompt)"
      ;;
  esac

  FOUND=0
  for vp in $VALID_POINTS; do
    if [ "$POINT" = "$vp" ]; then
      FOUND=1
      break
    fi
  done
  if [ "$FOUND" = "0" ]; then
    warn "event #$((i+1)): unrecognized point '$POINT'"
  fi
done

# Validate referenced merge scripts exist
if [ "$MERGE_VIA" = "direct" ]; then
  if [ "$MERGE_STRATEGY" = "rebase" ] && [ ! -f "$SCRIPT_DIR/rebase-merge.sh" ]; then
    error "rebase-merge.sh not found in $SCRIPT_DIR"
  fi
  if [ "$MERGE_STRATEGY" = "squash" ] && [ ! -f "$SCRIPT_DIR/squash-merge.sh" ]; then
    error "squash-merge.sh not found in $SCRIPT_DIR"
  fi
fi

# Result
if [ "$ERRORS" -gt 0 ]; then
  echo "Validation failed with $ERRORS error(s)" >&2
  exit 1
else
  echo "Workflow configuration is valid"
fi
