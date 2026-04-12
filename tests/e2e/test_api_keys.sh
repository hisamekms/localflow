#!/usr/bin/env bash
# E2E tests for API key management endpoints
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

MASTER_KEY="test-key"

# Pick a random high port
PORT=$((20000 + RANDOM % 40000))
SERVER_URL="http://127.0.0.1:$PORT"
API_BASE="$SERVER_URL/api/v1"

# Start the API server with master key auth
SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" "$SENKO" --project-root "$TEST_PROJECT_ROOT" --db-path "$TEST_PROJECT_ROOT/.senko/data.db" serve --port "$PORT" &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null; cleanup_test_env' EXIT

wait_for "API server ready" 10 "curl -sf $SERVER_URL/api/v1/health >/dev/null"

# Create a test user (using master key)
USER_JSON=$(curl -sf -X POST "$API_BASE/users" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -d "{\"username\":\"apikey_test_user_$$\"}")
USER_ID=$(echo "$USER_JSON" | jq -r '.id')

# Add user to project
run_lf members add --user-id "$USER_ID" --role owner >/dev/null

# Helper: get HTTP status code (no auth)
status_no_auth() {
  curl -s -o /dev/null -w '%{http_code}' "$@"
}

# Helper: get HTTP status code with specific bearer token
status_with_token() {
  local token="$1"; shift
  curl -s -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $token" "$@"
}

# =============================================
# 1. POST /api/v1/users/{user_id}/api-keys
# =============================================

echo "=== POST api-keys without body returns 201 ==="
KEY1_JSON=$(curl -sf -X POST "$API_BASE/users/$USER_ID/api-keys" \
  -H "Authorization: Bearer $MASTER_KEY")
KEY1_STATUS=$?
assert_eq "0" "$KEY1_STATUS" "POST api-keys without body succeeds"

echo ""
echo "=== POST api-keys response contains required fields ==="
assert_contains "$KEY1_JSON" '"id"' "Response contains id"
assert_contains "$KEY1_JSON" '"key"' "Response contains key"
assert_contains "$KEY1_JSON" '"key_prefix"' "Response contains key_prefix"
assert_contains "$KEY1_JSON" '"created_at"' "Response contains created_at"

KEY1_ID=$(echo "$KEY1_JSON" | jq -r '.id')
KEY1_PREFIX=$(echo "$KEY1_JSON" | jq -r '.key_prefix')

echo ""
echo "=== POST api-keys with name and device_name returns 201 ==="
KEY2_JSON=$(curl -sf -X POST "$API_BASE/users/$USER_ID/api-keys" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -d '{"name":"my-test-key","device_name":"test-laptop"}')
assert_contains "$KEY2_JSON" '"key"' "POST with name/device_name returns key"
assert_json_field "$KEY2_JSON" '.name' "my-test-key" "POST sets name correctly"
assert_json_field "$KEY2_JSON" '.device_name' "test-laptop" "POST sets device_name correctly"

KEY2_ID=$(echo "$KEY2_JSON" | jq -r '.id')
KEY2_PREFIX=$(echo "$KEY2_JSON" | jq -r '.key_prefix')

echo ""
echo "=== POST api-keys without auth returns 401 ==="
STATUS=$(status_no_auth -X POST "$API_BASE/users/$USER_ID/api-keys")
assert_eq "401" "$STATUS" "POST api-keys without auth returns 401"

# =============================================
# 2. GET /api/v1/users/{user_id}/api-keys
# =============================================

echo ""
echo "=== GET api-keys returns 200 ==="
STATUS=$(status_with_token "$MASTER_KEY" "$API_BASE/users/$USER_ID/api-keys")
assert_eq "200" "$STATUS" "GET api-keys returns 200"

echo ""
echo "=== GET api-keys returns array ==="
KEYS_JSON=$(curl -sf -H "Authorization: Bearer $MASTER_KEY" "$API_BASE/users/$USER_ID/api-keys")
IS_ARRAY=$(echo "$KEYS_JSON" | jq 'if type == "array" then "yes" else "no" end' -r)
assert_eq "yes" "$IS_ARRAY" "GET api-keys returns an array"

echo ""
echo "=== GET api-keys contains created keys ==="
KEY_COUNT=$(echo "$KEYS_JSON" | jq 'length')
assert_eq "true" "$([ "$KEY_COUNT" -ge 2 ] && echo true || echo false)" "GET api-keys has at least 2 keys"

echo ""
echo "=== GET api-keys contains key1 by prefix ==="
FOUND_KEY1=$(echo "$KEYS_JSON" | jq -r --arg prefix "$KEY1_PREFIX" '[.[] | select(.key_prefix == $prefix)] | length')
assert_eq "1" "$FOUND_KEY1" "GET api-keys contains key1 (prefix: $KEY1_PREFIX)"

echo ""
echo "=== GET api-keys contains key2 with correct fields ==="
KEY2_FROM_LIST=$(echo "$KEYS_JSON" | jq --arg prefix "$KEY2_PREFIX" '.[] | select(.key_prefix == $prefix)')
assert_json_field "$KEY2_FROM_LIST" '.name' "my-test-key" "Listed key2 has correct name"
assert_json_field "$KEY2_FROM_LIST" '.device_name' "test-laptop" "Listed key2 has correct device_name"
assert_contains "$KEY2_FROM_LIST" '"user_id"' "Listed key2 has user_id"
assert_contains "$KEY2_FROM_LIST" '"created_at"' "Listed key2 has created_at"

echo ""
echo "=== GET api-keys does not expose full key secret ==="
HAS_SECRET=$(echo "$KEYS_JSON" | jq '[.[] | has("key")] | any' -r)
assert_eq "false" "$HAS_SECRET" "GET api-keys does not expose full key"

echo ""
echo "=== GET api-keys without auth returns 401 ==="
STATUS=$(status_no_auth "$API_BASE/users/$USER_ID/api-keys")
assert_eq "401" "$STATUS" "GET api-keys without auth returns 401"

# =============================================
# 3. DELETE /api/v1/users/{user_id}/api-keys/{key_id}
# =============================================

echo ""
echo "=== DELETE api-keys/{key_id} returns 204 ==="
STATUS=$(status_with_token "$MASTER_KEY" -X DELETE "$API_BASE/users/$USER_ID/api-keys/$KEY1_ID")
assert_eq "204" "$STATUS" "DELETE api-keys/$KEY1_ID returns 204"

echo ""
echo "=== Deleted key is removed from list ==="
KEYS_AFTER=$(curl -sf -H "Authorization: Bearer $MASTER_KEY" "$API_BASE/users/$USER_ID/api-keys")
FOUND_DELETED=$(echo "$KEYS_AFTER" | jq -r --arg prefix "$KEY1_PREFIX" '[.[] | select(.key_prefix == $prefix)] | length')
assert_eq "0" "$FOUND_DELETED" "Deleted key no longer in list"

REMAINING_COUNT=$(echo "$KEYS_AFTER" | jq 'length')
assert_eq "true" "$([ "$REMAINING_COUNT" -eq $((KEY_COUNT - 1)) ] && echo true || echo false)" "Key count decreased by 1"

echo ""
echo "=== DELETE api-keys without auth returns 401 ==="
STATUS=$(status_no_auth -X DELETE "$API_BASE/users/$USER_ID/api-keys/$KEY2_ID")
assert_eq "401" "$STATUS" "DELETE api-keys without auth returns 401"

test_summary
