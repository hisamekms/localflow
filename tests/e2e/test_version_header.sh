#!/usr/bin/env bash
# E2E tests for X-Senko-Version response header
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

PORT=$(allocate_port)
BASE="http://127.0.0.1:$PORT"
API="$BASE/api/v1"

# Start the API server with API key auth
MASTER_KEY=test-key
SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" "$SENKO" \
  --project-root "$TEST_PROJECT_ROOT" \
  --db-path "$TEST_PROJECT_ROOT/.senko/data.db" \
  serve --port "$PORT" &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null; cleanup_test_env' EXIT

wait_for "API server ready" 10 "curl -sf $API/health >/dev/null"

# Create a test user and API key
TEST_TOKEN=$(create_test_user_key "$BASE" "$MASTER_KEY")

echo "=== Authenticated request has X-Senko-Version header ==="
HEADERS=$(curl -sD - -o /dev/null -H "Authorization: Bearer $TEST_TOKEN" "$API/health")
assert_contains "$HEADERS" "x-senko-version:" "authenticated response includes x-senko-version"

echo ""
echo "=== Version header value matches binary version ==="
VERSION=$(echo "$HEADERS" | grep -i "^x-senko-version:" | tr -d '\r' | awk '{print $2}')
EXPECTED=$("$SENKO" --version | awk '{print $2}')
assert_eq "$EXPECTED" "$VERSION" "header value matches CARGO_PKG_VERSION"

echo ""
echo "=== Unauthenticated request has no X-Senko-Version header ==="
HEADERS_NOAUTH=$(curl -sD - -o /dev/null "$API/health")
assert_not_contains "$HEADERS_NOAUTH" "x-senko-version:" "unauthenticated response has no x-senko-version"

echo ""
echo "=== Invalid token (401) has no X-Senko-Version header ==="
HEADERS_INVALID=$(curl -sD - -o /dev/null -H "Authorization: Bearer invalid-token-xxx" "$BASE/auth/me")
assert_not_contains "$HEADERS_INVALID" "x-senko-version:" "401 response has no x-senko-version"

test_summary
