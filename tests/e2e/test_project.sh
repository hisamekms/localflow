#!/usr/bin/env bash
# e2e test: Project management (create, list, delete)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

echo "--- Test: Project Management ---"

# 1. Default project exists
echo "[1] Default project exists"
LIST_OUTPUT="$(run_lf project list)"
DEFAULT_COUNT="$(echo "$LIST_OUTPUT" | jq '[.[] | select(.name == "default")] | length')"
assert_eq "1" "$DEFAULT_COUNT" "default project exists"

# 2. Create a project
echo "[2] Create project"
CREATE_OUTPUT="$(run_lf project create --name "test-project" --description "A test project")"
PROJECT_ID="$(echo "$CREATE_OUTPUT" | jq -r '.id')"
assert_json_field "$CREATE_OUTPUT" '.name' "test-project" "created project name"
assert_json_field "$CREATE_OUTPUT" '.description' "A test project" "created project description"

# 3. List includes new project
echo "[3] List includes new project"
LIST_OUTPUT="$(run_lf project list)"
PROJECT_COUNT="$(echo "$LIST_OUTPUT" | jq 'length')"
assert_eq "2" "$PROJECT_COUNT" "list has 2 projects"
FOUND="$(echo "$LIST_OUTPUT" | jq -r --arg id "$PROJECT_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
assert_eq "1" "$FOUND" "list contains created project"

# 4. Create project without description
echo "[4] Create project without description"
CREATE2_OUTPUT="$(run_lf project create --name "minimal-project")"
assert_json_field "$CREATE2_OUTPUT" '.name' "minimal-project" "minimal project name"
assert_json_field "$CREATE2_OUTPUT" '.description' "null" "minimal project description is null"

# 5. Delete project
echo "[5] Delete project"
DELETE_OUTPUT="$(run_lf project delete "$PROJECT_ID")"
LIST_OUTPUT="$(run_lf project list)"
REMAINING="$(echo "$LIST_OUTPUT" | jq -r --arg id "$PROJECT_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
assert_eq "0" "$REMAINING" "deleted project not in list"

# 6. Delete non-existent project (error)
echo "[6] Delete non-existent project"
assert_exit_code 1 run_lf project delete 9999

# 7. Text output
echo "[7] Text output"
TEXT_LIST="$(run_lf --output text project list)"
assert_contains "$TEXT_LIST" "default" "text list contains default project"

TEXT_CREATE="$(run_lf --output text project create --name "text-test")"
assert_contains "$TEXT_CREATE" "Created project" "text create output"

test_summary
