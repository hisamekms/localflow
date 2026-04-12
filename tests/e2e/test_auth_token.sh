#!/usr/bin/env bash
# E2E tests for `senko auth token` output format verification
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

# Check if the kernel keyring is functional (needed for keychain-seeded tests).
# keyctl command + working add_key syscall are both required.
KEYRING_AVAILABLE=false
if command -v keyctl &>/dev/null \
  && keyctl add user "keyring-rs:__probe__@senko" "probe" @s >/dev/null 2>&1; then
  KEYRING_AVAILABLE=true
  keyctl unlink "$(keyctl search @s user "keyring-rs:__probe__@senko")" @s 2>/dev/null || true
fi

echo "--- Test: auth token output ---"
echo "  keyring available: $KEYRING_AVAILABLE"

# [1] Without cli.remote.url configured → error
echo "[1] auth token fails without cli.remote.url"
OUTPUT=$("$SENKO" --project-root "$TEST_PROJECT_ROOT" auth token 2>&1 || true)
assert_contains "$OUTPUT" "cli.remote.url is not configured" "error without remote url"

# Start a server so auth token can query /auth/config for auth_mode
PORT=$((20000 + RANDOM % 40000))
API_URL="http://127.0.0.1:$PORT"
MASTER_KEY=test-key

SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --db-path "$TEST_PROJECT_ROOT/.senko/data.db" serve --port "$PORT" >/dev/null 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null; cleanup_test_env' EXIT

wait_for "API server ready" 10 "curl -sf $API_URL/api/v1/health >/dev/null"

# [2] With URL but no keychain entry → error
echo "[2] auth token fails when not logged in"
OUTPUT=$(SENKO_SERVER_URL="$API_URL" "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  auth token 2>&1 || true)
assert_contains "$OUTPUT" "Not logged in" "not logged in error"

# --- Keychain-dependent tests (skip if kernel keyring is not available) ---
if [[ "$KEYRING_AVAILABLE" != "true" ]]; then
  echo ""
  echo "  SKIP: keychain output tests (kernel keyring not available)"
  test_summary
  exit $?
fi

# Create a test user and API key
TEST_TOKEN=$(create_test_user_key "$API_URL" "$MASTER_KEY")

# Seed the Linux kernel keyring with the same format that keyring-rs uses:
# keyring-rs:{user}@{service} where service="senko" and user=api_url
# See: keyring crate v3, keyutils.rs:274
keyctl add user "keyring-rs:${API_URL}@senko" "$TEST_TOKEN" @s >/dev/null

# [3] Text mode: raw token, no decoration
echo "[3] auth token --output text: raw token only"
TEXT_OUTPUT=$(SENKO_SERVER_URL="$API_URL" "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --output text auth token 2>/dev/null)
# print! in Rust has no trailing newline; $() strips trailing newlines → direct comparison
assert_eq "$TEST_TOKEN" "$TEXT_OUTPUT" "text: token matches exactly"

# Verify no decoration
assert_not_contains "$TEXT_OUTPUT" "Token" "text: no 'Token' label"
assert_not_contains "$TEXT_OUTPUT" "token" "text: no 'token' label"
assert_not_contains "$TEXT_OUTPUT" ":" "text: no colon (no key-value format)"

# [4] JSON mode: valid JSON with token field
echo "[4] auth token --output json: JSON output"
JSON_OUTPUT=$(SENKO_SERVER_URL="$API_URL" "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --output json auth token 2>/dev/null)
# Verify it's valid JSON
echo "$JSON_OUTPUT" | jq . >/dev/null 2>&1
VALID_JSON=$?
assert_eq "0" "$VALID_JSON" "json: valid JSON"
assert_json_field "$JSON_OUTPUT" '.token' "$TEST_TOKEN" "json: token field matches"

# [5] JSON output has exactly one key
echo "[5] JSON has only token field"
KEY_COUNT=$(echo "$JSON_OUTPUT" | jq 'keys | length')
assert_eq "1" "$KEY_COUNT" "json: exactly one key"

# Cleanup keyring entry
keyctl unlink "$(keyctl search @s user "keyring-rs:${API_URL}@senko")" @s 2>/dev/null || true

test_summary
