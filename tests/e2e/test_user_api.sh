#!/usr/bin/env bash
# E2E tests for user API endpoints (master key restriction)
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

MASTER_KEY="test-key"

# Pick a random high port
PORT=$((20000 + RANDOM % 40000))
BASE="http://127.0.0.1:$PORT/api/v1"

# Start the API server with master key auth
SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" "$SENKO" --project-root "$TEST_PROJECT_ROOT" --db-path "$TEST_PROJECT_ROOT/.senko/data.db" serve --port "$PORT" &
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

test_summary
