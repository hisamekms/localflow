#!/usr/bin/env bash
# e2e test: Watch events for ready/started/canceled hooks and from_status field

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

echo "--- Test: Watch Events ---"

HOOK_LOG="$TEST_PROJECT_ROOT/.localflow/hook.log"

# Initialize DB first (creates .localflow/)
run_lf --output json list >/dev/null 2>&1

# Configure hooks for all events
cat > "$TEST_PROJECT_ROOT/.localflow/config.toml" <<EOF
[hooks]
on_task_added = "cat >> $HOOK_LOG"
on_task_ready = "cat >> $HOOK_LOG"
on_task_started = "cat >> $HOOK_LOG"
on_task_completed = "cat >> $HOOK_LOG"
on_task_canceled = "cat >> $HOOK_LOG"
EOF

# Start watch daemon
run_lf watch -d --interval 1 >/dev/null 2>&1
sleep 2

# 1. Create a task → should fire task_added
echo "[1] task_added event"
TASK_ID="$(run_lf --output json add --title "Watch test" | jq -r '.id')"
sleep 3

if [ -f "$HOOK_LOG" ]; then
  ADDED_EVENT="$(grep -c '"event":"task_added"' "$HOOK_LOG" 2>/dev/null || echo 0)"
  if [ "$ADDED_EVENT" -ge 1 ]; then
    echo "  PASS: task_added event fired"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "  FAIL: task_added event not found"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
else
  echo "  FAIL: hook log not created"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# 2. Ready the task → should fire task_ready
echo "[2] task_ready event"
run_lf ready "$TASK_ID" >/dev/null
sleep 3

READY_EVENT="$(grep -c '"event":"task_ready"' "$HOOK_LOG" 2>/dev/null || echo 0)"
if [ "$READY_EVENT" -ge 1 ]; then
  echo "  PASS: task_ready event fired"
  PASS_COUNT=$((PASS_COUNT + 1))
else
  echo "  FAIL: task_ready event not found in hook log"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Check from_status is present in task_ready event
READY_FROM="$(grep '"event":"task_ready"' "$HOOK_LOG" | head -1 | grep -c '"from_status":"draft"' 2>/dev/null || echo 0)"
if [ "$READY_FROM" -ge 1 ]; then
  echo "  PASS: task_ready has from_status=draft"
  PASS_COUNT=$((PASS_COUNT + 1))
else
  echo "  FAIL: task_ready missing from_status=draft"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# 3. Start the task → should fire task_started
echo "[3] task_started event"
run_lf start "$TASK_ID" >/dev/null
sleep 3

STARTED_EVENT="$(grep -c '"event":"task_started"' "$HOOK_LOG" 2>/dev/null || echo 0)"
if [ "$STARTED_EVENT" -ge 1 ]; then
  echo "  PASS: task_started event fired"
  PASS_COUNT=$((PASS_COUNT + 1))
else
  echo "  FAIL: task_started event not found"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

STARTED_FROM="$(grep '"event":"task_started"' "$HOOK_LOG" | head -1 | grep -c '"from_status":"todo"' 2>/dev/null || echo 0)"
if [ "$STARTED_FROM" -ge 1 ]; then
  echo "  PASS: task_started has from_status=todo"
  PASS_COUNT=$((PASS_COUNT + 1))
else
  echo "  FAIL: task_started missing from_status=todo"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# 4. Complete the task → should fire task_completed
echo "[4] task_completed event"
run_lf complete "$TASK_ID" >/dev/null
sleep 3

COMPLETED_EVENT="$(grep -c '"event":"task_completed"' "$HOOK_LOG" 2>/dev/null || echo 0)"
if [ "$COMPLETED_EVENT" -ge 1 ]; then
  echo "  PASS: task_completed event fired"
  PASS_COUNT=$((PASS_COUNT + 1))
else
  echo "  FAIL: task_completed event not found"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# 5. Create and cancel a task → should fire task_canceled
echo "[5] task_canceled event"
TASK2_ID="$(run_lf --output json add --title "Cancel watch" | jq -r '.id')"
sleep 2
run_lf cancel "$TASK2_ID" >/dev/null
sleep 3

CANCELED_EVENT="$(grep -c '"event":"task_canceled"' "$HOOK_LOG" 2>/dev/null || echo 0)"
if [ "$CANCELED_EVENT" -ge 1 ]; then
  echo "  PASS: task_canceled event fired"
  PASS_COUNT=$((PASS_COUNT + 1))
else
  echo "  FAIL: task_canceled event not found"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Stop daemon
run_lf watch stop >/dev/null 2>&1 || true

test_summary
