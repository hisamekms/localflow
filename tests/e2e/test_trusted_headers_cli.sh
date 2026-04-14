#!/usr/bin/env bash
# E2E tests for trusted_headers CLI auth: ensure_cli_token resolves cached
# auth_mode and loads the access token from keychain into the HTTP backend.
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

# Check if the kernel keyring is functional
KEYRING_AVAILABLE=false
if command -v keyctl &>/dev/null \
  && keyctl add user "keyring-rs:__probe__@senko" "probe" @s >/dev/null 2>&1; then
  KEYRING_AVAILABLE=true
  keyctl unlink "$(keyctl search @s user "keyring-rs:__probe__@senko")" @s 2>/dev/null || true
fi

if [[ "$KEYRING_AVAILABLE" != "true" ]]; then
  echo "SKIP: trusted_headers CLI tests (kernel keyring not available)"
  test_summary
  exit $?
fi

echo "--- Test: trusted_headers CLI auth ---"

# Start a server with API key auth (master key) so we can create users/keys.
PORT=$(allocate_port)
API_URL="http://127.0.0.1:$PORT"
MASTER_KEY=test-master-key

SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY" \
  "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --db-path "$TEST_PROJECT_ROOT/.senko/data.db" serve --port "$PORT" >/dev/null 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null; cleanup_test_env' EXIT

wait_for "API server ready" 10 "curl -sf $API_URL/api/v1/health >/dev/null"

# Create a test user and API key
TEST_TOKEN=$(create_test_user_key "$API_URL" "$MASTER_KEY")

# Seed the kernel keyring with the test token as an access token
# keyring-rs format: keyring-rs:{user}@{service}
# For access tokens: service = "senko-access-token", user = api_url
keyctl add user "keyring-rs:${API_URL}@senko-access-token" "$TEST_TOKEN" @s >/dev/null

# Seed the auth_mode cache with trusted_headers
mkdir -p "$XDG_CACHE_HOME/senko"
echo "{\"${API_URL}\": \"trusted_headers\"}" > "$XDG_CACHE_HOME/senko/auth_mode.json"

# [1] CLI project list should succeed (not 401) using cached auth_mode + keychain token
echo "[1] project list succeeds with trusted_headers cached auth"
OUTPUT=$(SENKO_CLI_REMOTE_URL="$API_URL" "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --output json project list 2>&1)
EXIT_CODE=$?
assert_eq "0" "$EXIT_CODE" "project list exits 0"

# [2] CLI user list should succeed
echo "[2] user list succeeds with trusted_headers cached auth"
OUTPUT=$(SENKO_CLI_REMOTE_URL="$API_URL" "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --output json user list 2>&1)
EXIT_CODE=$?
assert_eq "0" "$EXIT_CODE" "user list exits 0"

# [3] Without the cache, the same commands should fail (no token sent)
echo "[3] project list fails without auth_mode cache"
rm -f "$XDG_CACHE_HOME/senko/auth_mode.json"
OUTPUT=$(SENKO_CLI_REMOTE_URL="$API_URL" "$SENKO" --project-root "$TEST_PROJECT_ROOT" \
  --output json project list 2>&1 || true)
# Without the cache, ensure_cli_token won't load the token → server returns 401
assert_contains "$OUTPUT" "401\|Unauthorized\|unauthorized\|missing" \
  "project list fails without cache"

# Cleanup keyring entries
keyctl unlink "$(keyctl search @s user "keyring-rs:${API_URL}@senko-access-token")" @s 2>/dev/null || true

test_summary
