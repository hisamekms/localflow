#!/usr/bin/env bash
# E2E test: `senko auth status` through a proxy/relay server.
# Verifies that /auth/me on the relay forwards the client's Bearer token to
# upstream and returns the upstream user's info (not a 401).

source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

UPSTREAM_PID=""
RELAY_PID=""
MASTER_KEY=test-key

UPSTREAM_PORT=$(allocate_port 0)
RELAY_PORT=$(allocate_port 1)
UPSTREAM_URL="http://127.0.0.1:$UPSTREAM_PORT"
RELAY_URL="http://127.0.0.1:$RELAY_PORT"

cleanup_all() {
  if [[ -n "$RELAY_PID" ]]; then
    kill "$RELAY_PID" 2>/dev/null || true
    wait "$RELAY_PID" 2>/dev/null || true
  fi
  if [[ -n "$UPSTREAM_PID" ]]; then
    kill "$UPSTREAM_PID" 2>/dev/null || true
    wait "$UPSTREAM_PID" 2>/dev/null || true
  fi
  cleanup_test_env
}
trap cleanup_all EXIT

# Start upstream server (SQLite backend, master key auth)
SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" \
  "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --db-path "$TEST_PROJECT_ROOT/.senko/data.db" \
  serve --port "$UPSTREAM_PORT" >/dev/null 2>&1 &
UPSTREAM_PID=$!
wait_for "upstream ready" 10 "curl -sf $UPSTREAM_URL/api/v1/health >/dev/null"

# Start relay server (proxy_mode, forwarding to upstream). No server-side token
# configured, so client tokens flow through via PASSTHROUGH_TOKEN.
SENKO_SERVER_RELAY_URL="$UPSTREAM_URL" \
  "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  serve --port "$RELAY_PORT" >/dev/null 2>&1 &
RELAY_PID=$!
wait_for "relay ready" 10 "curl -sf $RELAY_URL/api/v1/health >/dev/null"

# Create a test user + API key on the upstream. We need the username so we can
# assert `auth status` returns the correct identity.
TEST_USERNAME="test_user_$$_$(date +%s%N)"
USER_JSON=$(curl -sf -X POST "$UPSTREAM_URL/api/v1/users" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -d "{\"username\":\"$TEST_USERNAME\"}")
USER_ID=$(echo "$USER_JSON" | jq -r '.id')
run_lf project members add --user-id "$USER_ID" --role owner >/dev/null
KEY_JSON=$(curl -sf -X POST "$UPSTREAM_URL/api/v1/users/$USER_ID/api-keys" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $MASTER_KEY" \
  -d '{}')
TEST_TOKEN=$(echo "$KEY_JSON" | jq -r '.key')

run_relay() {
  SENKO_CLI_REMOTE_URL="$RELAY_URL" SENKO_CLI_REMOTE_TOKEN="$TEST_TOKEN" \
    "$SENKO" --project-root "$TEST_PROJECT_ROOT" "$@"
}

echo "=== Section 1: senko auth status via relay returns upstream user info ==="

echo "[1.1] auth status succeeds and reports logged_in=true"
STATUS_OUT=$(run_relay auth status)
assert_json_field "$STATUS_OUT" '.logged_in' "true" "auth status: logged_in=true"
assert_json_field "$STATUS_OUT" '.username' "$TEST_USERNAME" "auth status: username matches upstream user"
assert_json_field "$STATUS_OUT" '.api_url' "$RELAY_URL" "auth status: api_url is relay URL"

echo "[1.2] direct GET /auth/me on relay returns 200"
ME_STATUS=$(curl -s -o /dev/null -w '%{http_code}' \
  -H "Authorization: Bearer $TEST_TOKEN" "$RELAY_URL/auth/me")
assert_eq "200" "$ME_STATUS" "GET /auth/me via relay: 200"

echo "[1.3] direct GET /auth/me returns user.username matching upstream"
ME_BODY=$(curl -sf -H "Authorization: Bearer $TEST_TOKEN" "$RELAY_URL/auth/me")
assert_json_field "$ME_BODY" '.user.username' "$TEST_USERNAME" "GET /auth/me via relay: username"

echo ""
echo "=== Section 2: invalid token through relay returns 401 ==="

echo "[2.1] auth status with invalid token fails"
BAD_OUT=$(SENKO_CLI_REMOTE_URL="$RELAY_URL" SENKO_CLI_REMOTE_TOKEN="invalid-token-xxxxx" \
  "$SENKO" --project-root "$TEST_PROJECT_ROOT" auth status 2>&1 || true)
assert_contains "$BAD_OUT" "Not logged in" "auth status invalid token: Not logged in error"

echo "[2.2] direct GET /auth/me via relay with invalid token returns 401"
BAD_STATUS=$(curl -s -o /dev/null -w '%{http_code}' \
  -H "Authorization: Bearer invalid-token-xxxxx" "$RELAY_URL/auth/me")
assert_eq "401" "$BAD_STATUS" "GET /auth/me via relay invalid token: 401"

echo "[2.3] direct GET /auth/me via relay without token returns 401"
NO_AUTH_STATUS=$(curl -s -o /dev/null -w '%{http_code}' "$RELAY_URL/auth/me")
assert_eq "401" "$NO_AUTH_STATUS" "GET /auth/me via relay no token: 401"

test_summary
