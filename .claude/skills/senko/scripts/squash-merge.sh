#!/usr/bin/env bash
set -euo pipefail

BRANCH="${1:?Usage: squash-merge.sh <branch-name>}"
LOCKFILE="/tmp/senko-squash-merge.lock"

exec 200>"$LOCKFILE"
if ! flock -n 200; then
  echo "error: another squash-merge is already running" >&2
  exit 1
fi

git checkout main
git pull --ff-only origin main 2>/dev/null || true
git merge --squash "$BRANCH"
git commit --no-edit
