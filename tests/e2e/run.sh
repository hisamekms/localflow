#!/usr/bin/env bash
# e2e test runner for localflow CLI

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Building localflow ==="
cd "$PROJECT_ROOT"
cargo build 2>&1
echo ""

export LOCALFLOW="$PROJECT_ROOT/target/debug/localflow"

TOTAL_PASS=0
TOTAL_FAIL=0
FAILED_TESTS=()

for test_file in "$SCRIPT_DIR"/test_*.sh; do
  test_name="$(basename "$test_file")"
  echo "=== Running: $test_name ==="

  if bash "$test_file"; then
    echo ">>> $test_name: OK"
  else
    echo ">>> $test_name: FAILED"
    FAILED_TESTS+=("$test_name")
  fi
  echo ""
done

echo "=== Overall Results ==="
echo "  Test files run: $(ls "$SCRIPT_DIR"/test_*.sh 2>/dev/null | wc -l)"
echo "  Failed: ${#FAILED_TESTS[@]}"

if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
  echo "  Failed tests:"
  for t in "${FAILED_TESTS[@]}"; do
    echo "    - $t"
  done
  exit 1
else
  echo "  All tests passed!"
  exit 0
fi
