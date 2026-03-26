#!/usr/bin/env bash
# e2e test: Status transition validation using dedicated commands (ready/start/complete/cancel)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

echo "--- Test: Status Transitions ---"

# ===== Valid Transitions =====

echo "[1] Valid: draft → todo (ready)"
OUT="$(run_lf --output json add --title "Valid 1")"
ID="$(echo "$OUT" | jq -r '.id')"
OUT="$(run_lf --output json ready "$ID")"
assert_json_field "$OUT" '.status' "todo" "draft → todo"

echo "[2] Valid: todo → in_progress (start)"
OUT="$(run_lf --output json start "$ID")"
assert_json_field "$OUT" '.status' "in_progress" "todo → in_progress"

echo "[3] Valid: in_progress → completed"
OUT="$(run_lf --output json complete "$ID")"
assert_json_field "$OUT" '.status' "completed" "in_progress → completed"

echo "[4] Valid: draft → canceled"
OUT="$(run_lf --output json add --title "Valid 4")"
ID="$(echo "$OUT" | jq -r '.id')"
OUT="$(run_lf --output json cancel "$ID")"
assert_json_field "$OUT" '.status' "canceled" "draft → canceled"

echo "[5] Valid: todo → canceled"
OUT="$(run_lf --output json add --title "Valid 5")"
ID="$(echo "$OUT" | jq -r '.id')"
run_lf ready "$ID" >/dev/null
OUT="$(run_lf --output json cancel "$ID" --reason "不要")"
assert_json_field "$OUT" '.status' "canceled" "todo → canceled"

echo "[6] Valid: in_progress → canceled"
OUT="$(run_lf --output json add --title "Valid 6")"
ID="$(echo "$OUT" | jq -r '.id')"
run_lf ready "$ID" >/dev/null
run_lf start "$ID" >/dev/null
OUT="$(run_lf --output json cancel "$ID" --reason "中止")"
assert_json_field "$OUT" '.status' "canceled" "in_progress → canceled"

# ===== Invalid Transitions =====

# Helper: create a task in a given status
create_task_in_status() {
  local status="$1"
  local out id
  out="$(run_lf --output json add --title "Task $status")"
  id="$(echo "$out" | jq -r '.id')"
  case "$status" in
    draft) ;;
    todo)
      run_lf ready "$id" >/dev/null
      ;;
    in_progress)
      run_lf ready "$id" >/dev/null
      run_lf start "$id" >/dev/null
      ;;
    completed)
      run_lf ready "$id" >/dev/null
      run_lf start "$id" >/dev/null
      run_lf complete "$id" >/dev/null
      ;;
    canceled)
      run_lf cancel "$id" >/dev/null
      ;;
  esac
  echo "$id"
}

echo "[7] Invalid: completed → todo (ready on completed)"
ID="$(create_task_in_status completed)"
assert_exit_code 1 run_lf ready "$ID"

echo "[8] Invalid: completed → in_progress (start on completed)"
ID="$(create_task_in_status completed)"
assert_exit_code 1 run_lf start "$ID"

echo "[9] Invalid: canceled → todo (ready on canceled)"
ID="$(create_task_in_status canceled)"
assert_exit_code 1 run_lf ready "$ID"

echo "[10] Invalid: canceled → in_progress (start on canceled)"
ID="$(create_task_in_status canceled)"
assert_exit_code 1 run_lf start "$ID"

echo "[11] Invalid: draft → in_progress (skip todo)"
ID="$(create_task_in_status draft)"
assert_exit_code 1 run_lf start "$ID"

echo "[12] Invalid: draft → completed (skip intermediate)"
ID="$(create_task_in_status draft)"
assert_exit_code 1 run_lf complete "$ID"

echo "[13] Invalid: todo → completed (skip in_progress)"
ID="$(create_task_in_status todo)"
assert_exit_code 1 run_lf complete "$ID"

echo "[14] Invalid: in_progress → todo (backwards, ready on in_progress)"
ID="$(create_task_in_status in_progress)"
assert_exit_code 1 run_lf ready "$ID"

echo "[15] Valid: start with --session-id"
OUT="$(run_lf --output json add --title "Session test")"
ID="$(echo "$OUT" | jq -r '.id')"
run_lf ready "$ID" >/dev/null
OUT="$(run_lf --output json start "$ID" --session-id "test-session")"
assert_json_field "$OUT" '.status' "in_progress" "start with session_id"
assert_json_field "$OUT" '.assignee_session_id' "test-session" "session_id set"

echo "[16] edit --status is removed"
OUT="$(run_lf --output json add --title "No status flag")"
ID="$(echo "$OUT" | jq -r '.id')"
assert_exit_code 2 run_lf edit "$ID" --status todo

test_summary
