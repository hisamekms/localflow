#!/usr/bin/env bash
# E2E tests for user API endpoints (master key restriction)
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

MASTER_KEY="test-key"

PORT=$(allocate_port)
BASE="http://127.0.0.1:$PORT/api/v1"

# Start the API server with master key auth
SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" "$SENKO" --project-root "$TEST_PROJECT_ROOT" "${SENKO_DB_ARGS[@]}" serve --port "$PORT" &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null; cleanup_test_env' EXIT

wait_for "API server ready" 10 "curl -sf $BASE/health >/dev/null"

# Helper: get HTTP status code (no auth)
status_no_auth() {
  curl -s -o /dev/null -w '%{http_code}' -H "Content-Type: application/json" "$@"
}

# Helper: get HTTP status code with specific bearer token
status_with_token() {
  local token="$1"; shift
  curl -s -o /dev/null -w '%{http_code}' -H "Content-Type: application/json" \
    -H "Authorization: Bearer $token" "$@"
}

echo "=== POST /users with master key returns 201 ==="
STATUS=$(status_with_token "$MASTER_KEY" -X POST "$BASE/users" -d '{"username":"mk-user","display_name":"MK User"}')
assert_eq "201" "$STATUS" "POST /users with master key returns 201"

echo ""
echo "=== POST /users without auth returns 401 ==="
STATUS=$(status_no_auth -X POST "$BASE/users" -d '{"username":"no-auth-user"}')
assert_eq "401" "$STATUS" "POST /users without auth returns 401"

echo ""
echo "=== POST /users with invalid token returns 401 ==="
STATUS=$(status_with_token "wrong-key" -X POST "$BASE/users" -d '{"username":"bad-token-user"}')
assert_eq "401" "$STATUS" "POST /users with invalid token returns 401"

echo ""
echo "=== POST /users with valid user API key returns 403 ==="
# Create a user via master key, then create an API key for that user
USER=$(curl -sf -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -X POST "$BASE/users" -d '{"username":"regular-user"}')
USER_ID=$(echo "$USER" | jq -r '.id')

API_KEY_RESP=$(curl -sf -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -X POST "$BASE/users/$USER_ID/api-keys" -d '{"name":"test-key"}')
USER_API_KEY=$(echo "$API_KEY_RESP" | jq -r '.key')

STATUS=$(status_with_token "$USER_API_KEY" -X POST "$BASE/users" -d '{"username":"forbidden-user"}')
assert_eq "403" "$STATUS" "POST /users with user API key returns 403"

echo ""
echo "=== PUT /users/{id} self-update with user API key returns 200 ==="
STATUS=$(status_with_token "$USER_API_KEY" -X PUT "$BASE/users/$USER_ID" -d '{"username":"updated-user"}')
assert_eq "200" "$STATUS" "PUT /users/{id} self-update returns 200"

echo ""
echo "=== PUT /users/{id} verify updated username ==="
UPDATED_USER=$(curl -sf -H "Authorization: Bearer $USER_API_KEY" "$BASE/users/$USER_ID")
UPDATED_USERNAME=$(echo "$UPDATED_USER" | jq -r '.username')
assert_eq "updated-user" "$UPDATED_USERNAME" "username was updated"

echo ""
echo "=== PUT /users/{id} update other user with user API key returns 403 ==="
# Create another user
OTHER_USER=$(curl -sf -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -X POST "$BASE/users" -d '{"username":"other-user"}')
OTHER_USER_ID=$(echo "$OTHER_USER" | jq -r '.id')
STATUS=$(status_with_token "$USER_API_KEY" -X PUT "$BASE/users/$OTHER_USER_ID" -d '{"username":"hacked"}')
assert_eq "403" "$STATUS" "PUT /users/{other_id} with user API key returns 403"

echo ""
echo "=== PUT /users/{id} with master key can update any user ==="
STATUS=$(status_with_token "$MASTER_KEY" -X PUT "$BASE/users/$OTHER_USER_ID" -d '{"username":"master-updated"}')
assert_eq "200" "$STATUS" "PUT /users/{id} with master key returns 200"

echo ""
echo "=== PUT /users/{id} without auth returns 401 ==="
STATUS=$(status_no_auth -X PUT "$BASE/users/$USER_ID" -d '{"username":"no-auth"}')
assert_eq "401" "$STATUS" "PUT /users/{id} without auth returns 401"

# ---------------------------------------------------------------------------
# DELETE /users/{id} authorization (regression test for Vuln 2 / task 313)
# ---------------------------------------------------------------------------

echo ""
echo "=== Setup: issue an API key for OTHER_USER_ID and add them to project 1 ==="
OTHER_API_KEY_RESP=$(curl -sf -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -X POST "$BASE/users/$OTHER_USER_ID/api-keys" -d '{"name":"other-key"}')
OTHER_USER_KEY_COUNT_BEFORE=$(curl -sf -H "Authorization: Bearer $MASTER_KEY" \
  "$BASE/users/$OTHER_USER_ID/api-keys" | jq 'length')
assert_eq "1" "$OTHER_USER_KEY_COUNT_BEFORE" "OTHER_USER has 1 api-key before DELETE attempt"

# Add OTHER_USER_ID to project 1 via CLI (so we can verify CASCADE didn't fire)
run_lf project members add --user-id "$OTHER_USER_ID" --role member >/dev/null

echo ""
echo "=== DELETE /users/{other_id} with user API key returns 403 ==="
STATUS=$(status_with_token "$USER_API_KEY" -X DELETE "$BASE/users/$OTHER_USER_ID")
assert_eq "403" "$STATUS" "DELETE /users/{other_id} with user API key returns 403"

echo ""
echo "=== After 403, OTHER_USER row is intact ==="
STATUS=$(status_with_token "$MASTER_KEY" "$BASE/users/$OTHER_USER_ID")
assert_eq "200" "$STATUS" "GET /users/{other_id} still returns 200 (user not deleted)"

echo ""
echo "=== After 403, OTHER_USER's api-keys did not CASCADE-delete ==="
OTHER_USER_KEY_COUNT_AFTER=$(curl -sf -H "Authorization: Bearer $MASTER_KEY" \
  "$BASE/users/$OTHER_USER_ID/api-keys" | jq 'length')
assert_eq "$OTHER_USER_KEY_COUNT_BEFORE" "$OTHER_USER_KEY_COUNT_AFTER" \
  "OTHER_USER's api-keys count unchanged after forbidden DELETE"

echo ""
echo "=== After 403, OTHER_USER's project membership did not CASCADE-delete ==="
STATUS=$(status_with_token "$MASTER_KEY" "$BASE/projects/1/members/$OTHER_USER_ID")
assert_eq "200" "$STATUS" "GET /projects/1/members/{other_id} still returns 200"

echo ""
echo "=== DELETE /users/{id} without auth returns 401 ==="
STATUS=$(status_no_auth -X DELETE "$BASE/users/$OTHER_USER_ID")
assert_eq "401" "$STATUS" "DELETE /users/{id} without auth returns 401"

echo ""
echo "=== Self-delete: user can delete their own account (returns 204) ==="
SELF_DEL_USER=$(curl -sf -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -X POST "$BASE/users" -d '{"username":"self-delete-user"}')
SELF_DEL_USER_ID=$(echo "$SELF_DEL_USER" | jq -r '.id')
SELF_DEL_KEY_RESP=$(curl -sf -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -X POST "$BASE/users/$SELF_DEL_USER_ID/api-keys" -d '{"name":"self-key"}')
SELF_DEL_KEY=$(echo "$SELF_DEL_KEY_RESP" | jq -r '.key')
STATUS=$(status_with_token "$SELF_DEL_KEY" -X DELETE "$BASE/users/$SELF_DEL_USER_ID")
assert_eq "204" "$STATUS" "DELETE /users/{self_id} with own API key returns 204"

echo ""
echo "=== Master-key delete: master can delete any user (returns 204) ==="
MASTER_DEL_USER=$(curl -sf -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -X POST "$BASE/users" -d '{"username":"master-delete-user"}')
MASTER_DEL_USER_ID=$(echo "$MASTER_DEL_USER" | jq -r '.id')
STATUS=$(status_with_token "$MASTER_KEY" -X DELETE "$BASE/users/$MASTER_DEL_USER_ID")
assert_eq "204" "$STATUS" "DELETE /users/{id} with master key returns 204"

test_summary
