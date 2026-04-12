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

# Validate workflow stage hooks (instructions, pre_hooks, post_hooks)
STAGES="add start branch plan implement merge pr complete branch_cleanup"

validate_hook() {
  local stage="$1" phase="$2" index="$3"
  local hook
  hook=$(echo "$CONFIG_JSON" | jq -c --arg s "$stage" --arg p "$phase" ".workflow[\$s][\$p][$index]")
  local hook_type
  hook_type=$(echo "$hook" | jq -r 'type')
  if [ "$hook_type" = "string" ]; then
    if [ -z "$(echo "$hook" | jq -r '.')" ]; then
      error "workflow.${stage}.${phase}[${index}]: empty string hook"
    fi
  elif [ "$hook_type" = "object" ]; then
    local cmd prompt
    cmd=$(echo "$hook" | jq -r '.command // empty')
    prompt=$(echo "$hook" | jq -r '.prompt // empty')
    if [ -z "$cmd" ] && [ -z "$prompt" ]; then
      error "workflow.${stage}.${phase}[${index}]: hook object requires 'command' or 'prompt' field"
    fi
    local on_failure
    on_failure=$(echo "$hook" | jq -r '.on_failure // "abort"')
    case "$on_failure" in
      abort|warn|ignore) ;;
      *) error "workflow.${stage}.${phase}[${index}]: invalid on_failure '${on_failure}' (expected: abort, warn, ignore)" ;;
    esac
  else
    error "workflow.${stage}.${phase}[${index}]: invalid hook type '${hook_type}' (expected: string or object)"
  fi
}

for stage in $STAGES; do
  for phase in pre_hooks post_hooks; do
    local_count=$(echo "$CONFIG_JSON" | jq --arg s "$stage" --arg p "$phase" '.workflow[$s][$p] // [] | length')
    for i in $(seq 0 $((local_count - 1))); do
      validate_hook "$stage" "$phase" "$i"
    done
  done
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
