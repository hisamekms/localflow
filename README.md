# localflow

A local-only task management CLI for single-developer and single-agent workflows.
SQLite-backed, dependency-aware, priority-driven.

ローカル専用のタスク管理CLI。個人開発・単一エージェント向け。
SQLiteベース、依存関係対応、優先度駆動。

---

## Features / 機能

- **Task lifecycle**: `draft` → `todo` → `in_progress` → `completed` / `canceled`
- **Priority levels**: P0 (highest) – P3 (lowest)
- **Dependency tracking**: Tasks block until dependencies are completed
- **Smart next-task selection**: Picks the highest-priority ready task automatically
- **Dual output**: JSON (for AI/automation) and human-readable text
- **Claude Code integration**: `skill-install` generates a skill config for Claude Code
- **Zero setup**: SQLite database auto-created on first run

## Install / インストール

### Build from source

```bash
cargo build --release
```

The binary is at `target/release/localflow`.

### Claude Code integration / Claude Code 連携

```bash
localflow skill-install
```

Generates `SKILL.md` for Claude Code skill integration.

## Quick Start / クイックスタート

```bash
# Create a task / タスク作成
localflow add --title "Implement auth API" --priority p1

# List tasks / タスク一覧
localflow list

# Start the next ready task / 次のタスクを開始
localflow next

# Complete a task / タスク完了
localflow complete 1
```

## Commands / コマンド一覧

### Global Options / グローバルオプション

```
--output <FORMAT>       json or text (default: text)
--project-root <PATH>   Project root (auto-detected if omitted)
```

### `add` – Create a task / タスク作成

```bash
localflow add --title "Write docs" --priority p0
localflow add --title "Fix bug" \
  --background "Users report 500 errors" \
  --definition-of-done "No 500 errors in logs" \
  --in-scope "Error handler" \
  --out-of-scope "Refactoring" \
  --tag backend --tag urgent
```

### `list` – List tasks / タスク一覧

```bash
localflow list                    # All tasks
localflow list --status todo      # Filter by status
localflow list --ready            # Todo tasks with all deps met
localflow list --tag backend      # Filter by tag
```

### `get <id>` – Task details / タスク詳細

```bash
localflow get 1
localflow get 1 --output json
```

### `next` – Start next task / 次のタスクを開始

Selects the highest-priority `todo` task whose dependencies are all completed, and sets it to `in_progress`.

依存タスクがすべて完了済みの最高優先度 `todo` タスクを選択し、`in_progress` に変更します。

```bash
localflow next
localflow next --session-id "session-abc"
```

Selection order: priority (P0 first) → created_at → id.

### `edit <id>` – Edit a task / タスク編集

```bash
# Scalar fields / スカラーフィールド
localflow edit 1 --title "New title"
localflow edit 1 --status todo
localflow edit 1 --priority p0

# Array fields (tags, definition-of-done, scope) / 配列フィールド
localflow edit 1 --add-tag "urgent"
localflow edit 1 --remove-tag "old"
localflow edit 1 --set-tags "a" "b"         # Replace all
```

### `complete <id>` – Complete a task / タスク完了

```bash
localflow complete 1
```

### `cancel <id>` – Cancel a task / タスクキャンセル

```bash
localflow cancel 1 --reason "out of scope"
```

### `deps` – Manage dependencies / 依存関係管理

```bash
localflow deps add 5 --on 3        # Task 5 depends on task 3
localflow deps remove 5 --on 3     # Remove dependency
localflow deps set 5 --on 1 2 3    # Set exact dependencies
localflow deps list 5              # List dependencies of task 5
```

### `skill-install` – Claude Code integration

```bash
localflow skill-install
```

## Status Transitions / ステータス遷移

```
draft → todo → in_progress → completed
                    ↓
                 canceled
```

- `draft` → `todo` → `in_progress` → `completed`: forward-only
- Any active state → `canceled`: always allowed
- Backward transitions and self-transitions are rejected

## Data Storage / データ保存

The database is stored at `<project_root>/.localflow/data.db` (auto-created).

データベースは `<プロジェクトルート>/.localflow/data.db` に自動作成されます。

Project root is detected by searching for `.localflow/`, `.git/`, or using the current directory.

## Testing / テスト

```bash
cargo test                    # Unit tests
bash tests/e2e/run.sh         # E2E tests
```

## License

MIT
