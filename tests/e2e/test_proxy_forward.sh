#!/usr/bin/env bash
# E2E tests for proxy mode: verify all 8 endpoints forward to upstream
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

# --- Start upstream server ---
UPSTREAM_PORT=$((20000 + RANDOM % 20000))
UPSTREAM_URL="http://127.0.0.1:$UPSTREAM_PORT"
MASTER_KEY=test-master-key

SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" "$SENKO" --project-root "$TEST_PROJECT_ROOT" --db-path "$TEST_PROJECT_ROOT/.senko/data.db" serve --port "$UPSTREAM_PORT" &
UPSTREAM_PID=$!

# --- Start proxy server ---
PROXY_PORT=$((40000 + RANDOM % 20000))
PROXY_URL="http://127.0.0.1:$PROXY_PORT"
PROXY_DIR="$(mktemp -d)"
mkdir -p "$PROXY_DIR"

# Proxy server: points to upstream, no local auth
SENKO_SERVER_URL="$UPSTREAM_URL" "$SENKO" --project-root "$PROXY_DIR" --db-path "$PROXY_DIR/.senko/data.db" serve --port "$PROXY_PORT" &
PROXY_PID=$!

trap 'kill $UPSTREAM_PID $PROXY_PID 2>/dev/null; rm -rf "$PROXY_DIR"; cleanup_test_env' EXIT

# Wait for both servers
wait_for "upstream server ready" 10 "curl -sf $UPSTREAM_URL/api/v1/health >/dev/null"
wait_for "proxy server ready" 10 "curl -sf $PROXY_URL/api/v1/health >/dev/null"

# Create a user and API key on the upstream
TEST_TOKEN=$(create_test_user_key "$UPSTREAM_URL" "$MASTER_KEY")

echo "--- Test: Proxy Forward Mode ---"

echo "[1] GET /api/v1/health via proxy"
HEALTH=$(curl -sf "$PROXY_URL/api/v1/health")
assert_json_field "$HEALTH" '.status' "ok" "health: forwarded from upstream"

echo "[2] GET /auth/config via proxy"
AUTH_CONFIG=$(curl -sf "$PROXY_URL/auth/config")
# Upstream has no OIDC configured, so oidc should be null
assert_json_field "$AUTH_CONFIG" '.oidc' "null" "auth config: oidc is null (from upstream)"

echo "[3] GET /api/v1/config via proxy"
CONFIG=$(curl -sf -H "Authorization: Bearer $TEST_TOKEN" "$PROXY_URL/api/v1/config")
# Should return upstream's config (which has workflow section)
assert_json_field "$CONFIG" '.workflow.merge_via' "direct" "config: forwarded from upstream"

echo "[4] POST /auth/token via proxy"
TOKEN_RESP=$(curl -sf -X POST "$PROXY_URL/auth/token" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"device_name": "proxy-test"}')
SESSION_TOKEN=$(echo "$TOKEN_RESP" | jq -r '.token')
SESSION_ID=$(echo "$TOKEN_RESP" | jq -r '.id')
assert_eq "false" "$([ -z "$SESSION_TOKEN" ] || [ "$SESSION_TOKEN" = "null" ] && echo true || echo false)" "token: got session token via proxy"
assert_eq "false" "$([ -z "$SESSION_ID" ] || [ "$SESSION_ID" = "null" ] && echo true || echo false)" "token: got session id via proxy"

echo "[5] GET /auth/me via proxy"
ME=$(curl -sf -H "Authorization: Bearer $SESSION_TOKEN" "$PROXY_URL/auth/me")
assert_eq "false" "$(echo "$ME" | jq -r '.user.id' | grep -q '^[0-9]' && echo false || echo true)" "me: got user info via proxy"
assert_eq "false" "$(echo "$ME" | jq -r '.session.id' | grep -q '^[0-9]' && echo false || echo true)" "me: got session info via proxy"

echo "[6] GET /auth/sessions via proxy"
SESSIONS=$(curl -sf -H "Authorization: Bearer $SESSION_TOKEN" "$PROXY_URL/auth/sessions")
SESSION_COUNT=$(echo "$SESSIONS" | jq 'length')
assert_eq "true" "$([ "$SESSION_COUNT" -ge 1 ] && echo true || echo false)" "sessions: at least 1 session via proxy"

# Find a session ID to delete (not the current one if possible, or use the one we have)
DELETE_SESSION_ID="$SESSION_ID"

echo "[7] DELETE /auth/sessions/{id} via proxy"
# Create another token first so we have something to delete
TOKEN_RESP2=$(curl -sf -X POST "$PROXY_URL/auth/token" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"device_name": "to-delete"}')
DELETE_ID=$(echo "$TOKEN_RESP2" | jq -r '.id')
DELETE_STATUS=$(curl -sf -o /dev/null -w "%{http_code}" -X DELETE \
  -H "Authorization: Bearer $SESSION_TOKEN" \
  "$PROXY_URL/auth/sessions/$DELETE_ID")
assert_eq "204" "$DELETE_STATUS" "revoke session: 204 via proxy"

echo "[8] DELETE /auth/sessions via proxy (revoke all)"
# Create a fresh token to use for this test (the previous session_token might still work)
TOKEN_RESP3=$(curl -sf -X POST "$PROXY_URL/auth/token" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"device_name": "revoke-all-test"}')
REVOKE_TOKEN=$(echo "$TOKEN_RESP3" | jq -r '.token')
REVOKE_STATUS=$(curl -sf -o /dev/null -w "%{http_code}" -X DELETE \
  -H "Authorization: Bearer $REVOKE_TOKEN" \
  "$PROXY_URL/auth/sessions")
assert_eq "204" "$REVOKE_STATUS" "revoke all sessions: 204 via proxy"

echo "[9] Verify upstream data was actually modified (not local proxy)"
# The token we used for revoke-all should now be invalid on upstream
VERIFY_STATUS=$(curl -o /dev/null -w "%{http_code}" -X GET \
  -H "Authorization: Bearer $REVOKE_TOKEN" \
  "$UPSTREAM_URL/auth/sessions" 2>/dev/null)
assert_eq "401" "$VERIFY_STATUS" "upstream: revoked token is invalid (confirms proxy forwarded correctly)"

test_summary
