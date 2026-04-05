#!/usr/bin/env bash
set -euo pipefail

BRANCH="${1:?Usage: squash-merge.sh <branch-name>}"
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

# Detect worktree vs primary
GIT_DIR_PATH="$(git rev-parse --path-format=absolute --git-dir)"
GIT_COMMON_DIR_PATH="$(git rev-parse --path-format=absolute --git-common-dir)"

if [ "$GIT_DIR_PATH" != "$GIT_COMMON_DIR_PATH" ]; then
  # --- Worktree flow ---
  PROJECT_ROOT="$(dirname "$GIT_COMMON_DIR_PATH")"

  # Check primary worktree is clean (unstaged, staged, untracked)
  if ! git -C "$PROJECT_ROOT" diff --quiet 2>/dev/null; then
    echo "error: primary worktree has unstaged changes" >&2
    exit 10
  fi
  if ! git -C "$PROJECT_ROOT" diff --cached --quiet 2>/dev/null; then
    echo "error: primary worktree has staged changes" >&2
    exit 10
  fi
  if [ -n "$(git -C "$PROJECT_ROOT" ls-files --others --exclude-standard 2>/dev/null)" ]; then
    echo "error: primary worktree has untracked files" >&2
    exit 10
  fi

  # Fetch latest main
  if ! git fetch origin main; then
    echo "error: failed to fetch origin main" >&2
    exit 1
  fi

  # Rebase current branch onto origin/main
  if ! git rebase origin/main; then
    echo "error: rebase conflict — resolve manually and retry" >&2
    git rebase --abort 2>/dev/null || true
    exit 11
  fi

  # Squash merge on primary
  git -C "$PROJECT_ROOT" merge --squash "$BRANCH"
  git -C "$PROJECT_ROOT" commit --no-edit
else
  # --- Primary flow (legacy) ---
  git checkout main
  git pull --ff-only origin main 2>/dev/null || true
  git merge --squash "$BRANCH"
  git commit --no-edit
fi
