#!/usr/bin/env bash
# e2e test: User management (create, list, delete)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

echo "--- Test: User Management ---"

# 1. Default user exists
echo "[1] Default user exists"
LIST_OUTPUT="$(run_lf user list)"
DEFAULT_COUNT="$(echo "$LIST_OUTPUT" | jq '[.[] | select(.username == "default")] | length')"
assert_eq "1" "$DEFAULT_COUNT" "default user exists"

# 2. Create user with all fields
echo "[2] Create user with all fields"
CREATE_OUTPUT="$(run_lf user create --username "testuser" --display-name "Test User" --email "test@example.com")"
USER_ID="$(echo "$CREATE_OUTPUT" | jq -r '.id')"
assert_json_field "$CREATE_OUTPUT" '.username' "testuser" "created user username"
assert_json_field "$CREATE_OUTPUT" '.display_name' "Test User" "created user display_name"
assert_json_field "$CREATE_OUTPUT" '.email' "test@example.com" "created user email"

# 3. Create user with minimal fields
echo "[3] Create user with minimal fields"
CREATE2_OUTPUT="$(run_lf user create --username "minimaluser")"
assert_json_field "$CREATE2_OUTPUT" '.username' "minimaluser" "minimal user username"
assert_json_field "$CREATE2_OUTPUT" '.display_name' "null" "minimal user display_name is null"
assert_json_field "$CREATE2_OUTPUT" '.email' "null" "minimal user email is null"

# 4. List includes new users
echo "[4] List includes new users"
LIST_OUTPUT="$(run_lf user list)"
USER_COUNT="$(echo "$LIST_OUTPUT" | jq 'length')"
assert_eq "3" "$USER_COUNT" "list has 3 users"

# 5. Delete user
echo "[5] Delete user"
run_lf user delete "$USER_ID" >/dev/null
LIST_OUTPUT="$(run_lf user list)"
REMAINING="$(echo "$LIST_OUTPUT" | jq -r --arg id "$USER_ID" '[.[] | select(.id == ($id | tonumber))] | length')"
assert_eq "0" "$REMAINING" "deleted user not in list"

# 6. Delete non-existent user (error)
echo "[6] Delete non-existent user"
assert_exit_code 1 run_lf user delete 9999

# 7. Text output
echo "[7] Text output"
TEXT_LIST="$(run_lf --output text user list)"
assert_contains "$TEXT_LIST" "default" "text list contains default user"

test_summary
