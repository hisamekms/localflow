#!/usr/bin/env bash
# e2e test: Project member management (add, list, set-role, remove)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

echo "--- Test: Members Management ---"

# Setup: create a user to work with
USER_OUTPUT="$(run_lf user create --username "member-test-user")"
USER_ID="$(echo "$USER_OUTPUT" | jq -r '.id')"

# 1. Default member exists (default user is owner)
echo "[1] Default member exists"
LIST_OUTPUT="$(run_lf members list)"
DEFAULT_MEMBER="$(echo "$LIST_OUTPUT" | jq '[.[] | select(.role == "owner")] | length')"
assert_eq "1" "$DEFAULT_MEMBER" "default owner member exists"

# 2. Add member
echo "[2] Add member"
ADD_OUTPUT="$(run_lf members add --user-id "$USER_ID" --role member)"
assert_json_field "$ADD_OUTPUT" '.user_id' "$USER_ID" "added member user_id"
assert_json_field "$ADD_OUTPUT" '.role' "member" "added member role"

# 3. List includes new member
echo "[3] List includes new member"
LIST_OUTPUT="$(run_lf members list)"
MEMBER_COUNT="$(echo "$LIST_OUTPUT" | jq 'length')"
assert_eq "2" "$MEMBER_COUNT" "list has 2 members"
FOUND="$(echo "$LIST_OUTPUT" | jq -r --arg uid "$USER_ID" '[.[] | select(.user_id == ($uid | tonumber))] | length')"
assert_eq "1" "$FOUND" "list contains new member"

# 4. Set role
echo "[4] Set role"
SET_OUTPUT="$(run_lf members set-role --user-id "$USER_ID" --role viewer)"
assert_json_field "$SET_OUTPUT" '.role' "viewer" "role updated to viewer"

SET_OUTPUT="$(run_lf members set-role --user-id "$USER_ID" --role owner)"
assert_json_field "$SET_OUTPUT" '.role' "owner" "role updated to owner"

# 5. Remove member
echo "[5] Remove member"
REMOVE_OUTPUT="$(run_lf members remove --user-id "$USER_ID")"
LIST_OUTPUT="$(run_lf members list)"
MEMBER_COUNT="$(echo "$LIST_OUTPUT" | jq 'length')"
assert_eq "1" "$MEMBER_COUNT" "list has 1 member after removal"

# 6. Add duplicate member (error)
echo "[6] Add duplicate member"
run_lf members add --user-id "$USER_ID" --role member >/dev/null 2>&1
assert_exit_code 1 run_lf members add --user-id "$USER_ID" --role member

# 7. Text output
echo "[7] Text output"
TEXT_LIST="$(run_lf --output text members list)"
assert_contains "$TEXT_LIST" "owner" "text list contains owner role"

test_summary
