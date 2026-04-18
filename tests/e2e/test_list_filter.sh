#!/usr/bin/env bash
# e2e test: list subcommand filter options (--status, --tag, --depends-on, --ready)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

echo "--- Test: List Filter Options ---"

# Setup: Create tasks
# Task A: tags=backend,rust, status=todo
A_ID="$(run_lf --output json task add --title "Alpha" --tag backend --tag rust | jq -r '.id')"
run_lf task ready "$A_ID" >/dev/null

# Task B: tags=frontend, status=todo, depends on A
B_ID="$(run_lf --output json task add --title "Beta" --tag frontend | jq -r '.id')"
run_lf task ready "$B_ID" >/dev/null
run_lf task deps add "$B_ID" --on "$A_ID" >/dev/null

# Task C: tags=backend, status=completed
C_ID="$(run_lf --output json task add --title "Gamma" --tag backend | jq -r '.id')"
run_lf task ready "$C_ID" >/dev/null
run_lf task start "$C_ID" >/dev/null
run_lf task complete "$C_ID" >/dev/null

# --- Case 1: --status todo ---
echo "[1] --status todo filter"
LIST_TODO="$(run_lf --output json task list --status todo)"
TODO_COUNT="$(echo "$LIST_TODO" | jq 'length')"
TODO_HAS_A="$(echo "$LIST_TODO" | jq --arg id "$A_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
TODO_HAS_B="$(echo "$LIST_TODO" | jq --arg id "$B_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
TODO_HAS_C="$(echo "$LIST_TODO" | jq --arg id "$C_ID" '[.[] | select(.id == ($id | tonumber))] | length')"

assert_eq "2" "$TODO_COUNT" "status=todo returns 2 tasks"
assert_eq "1" "$TODO_HAS_A" "status=todo includes Alpha"
assert_eq "1" "$TODO_HAS_B" "status=todo includes Beta"
assert_eq "0" "$TODO_HAS_C" "status=todo excludes Gamma"

# --- Case 2: --tag backend ---
echo "[2] --tag backend filter"
LIST_BACKEND="$(run_lf --output json task list --tag backend)"
BACKEND_COUNT="$(echo "$LIST_BACKEND" | jq 'length')"
BACKEND_HAS_A="$(echo "$LIST_BACKEND" | jq --arg id "$A_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
BACKEND_HAS_B="$(echo "$LIST_BACKEND" | jq --arg id "$B_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
BACKEND_HAS_C="$(echo "$LIST_BACKEND" | jq --arg id "$C_ID" '[.[] | select(.id == ($id | tonumber))] | length')"

assert_eq "2" "$BACKEND_COUNT" "tag=backend returns 2 tasks"
assert_eq "1" "$BACKEND_HAS_A" "tag=backend includes Alpha"
assert_eq "0" "$BACKEND_HAS_B" "tag=backend excludes Beta"
assert_eq "1" "$BACKEND_HAS_C" "tag=backend includes Gamma"

# --- Case 3: --depends-on <Task A ID> ---
echo "[3] --depends-on filter"
LIST_DEPS="$(run_lf --output json task list --depends-on "$A_ID")"
DEPS_COUNT="$(echo "$LIST_DEPS" | jq 'length')"
DEPS_HAS_A="$(echo "$LIST_DEPS" | jq --arg id "$A_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
DEPS_HAS_B="$(echo "$LIST_DEPS" | jq --arg id "$B_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
DEPS_HAS_C="$(echo "$LIST_DEPS" | jq --arg id "$C_ID" '[.[] | select(.id == ($id | tonumber))] | length')"

assert_eq "1" "$DEPS_COUNT" "depends-on A returns 1 task"
assert_eq "0" "$DEPS_HAS_A" "depends-on A excludes Alpha"
assert_eq "1" "$DEPS_HAS_B" "depends-on A includes Beta"
assert_eq "0" "$DEPS_HAS_C" "depends-on A excludes Gamma"

# --- Case 4: --ready ---
echo "[4] --ready filter"
LIST_READY="$(run_lf --output json task list --ready)"
READY_HAS_A="$(echo "$LIST_READY" | jq --arg id "$A_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
READY_HAS_B="$(echo "$LIST_READY" | jq --arg id "$B_ID" '[.[] | select(.id == ($id | tonumber))] | length')"

assert_eq "1" "$READY_HAS_A" "ready includes Alpha (no deps, todo)"
assert_eq "0" "$READY_HAS_B" "ready excludes Beta (unmet dep on Alpha)"

test_summary
