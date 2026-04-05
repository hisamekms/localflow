#!/usr/bin/env bash
set -euo pipefail

BRANCH="${1:?Usage: rebase-merge.sh <branch-name>}"
LOCKDIR="/tmp/senko-merge.lock"
MAX_RETRIES=3
RETRY_INTERVAL=1

cleanup() { rm -rf "$LOCKDIR"; }

# stale lock detection
if [ -d "$LOCKDIR" ]; then
  if [ -n "$(find "$LOCKDIR" -maxdepth 0 -mmin +10 2>/dev/null)" ]; then
    echo "warning: removing stale lock (older than 10 minutes)" >&2
    rm -rf "$LOCKDIR"
  fi
fi

# acquire lock with retry
acquired=false
for i in $(seq 1 "$MAX_RETRIES"); do
  if mkdir "$LOCKDIR" 2>/dev/null; then
    acquired=true
    break
  fi
  echo "warning: lock busy, retrying ($i/$MAX_RETRIES)..." >&2
  sleep "$RETRY_INTERVAL"
done

if [ "$acquired" != true ]; then
  echo "error: another merge operation is already running" >&2
  exit 1
fi

trap cleanup EXIT
echo $$ > "$LOCKDIR/pid"

git checkout main
git pull --ff-only origin main 2>/dev/null || true
git rebase main "$BRANCH"
git checkout main
git merge --ff-only "$BRANCH"
