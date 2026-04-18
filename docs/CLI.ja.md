# CLIリファレンス

[English](CLI.md) | [READMEに戻る](README.ja.md)

## グローバルオプション

```
--output <FORMAT>       json または text（デフォルト: json）
--project-root <PATH>   プロジェクトルート（省略時は自動検出）
--config <PATH>         設定ファイルのパス（環境変数: SENKO_CONFIG、デフォルト: .senko/config.toml）
--dry-run               実行せずに結果を表示（状態変更コマンドのみ）
--log-dir <PATH>        ログ出力ディレクトリを上書き（デフォルト: $XDG_STATE_HOME/senko）
```

> **注意**: `--output` と `--dry-run` はグローバルフラグです。サブコマンドの**前**に配置してください: `senko --output text task list`

## `task add` – タスク作成

```bash
senko task add --title "ドキュメント作成" --priority p0
senko task add --title "バグ修正" \
  --background "ユーザーから500エラーの報告" \
  --definition-of-done "ログに500エラーなし" \
  --in-scope "エラーハンドラ" \
  --out-of-scope "リファクタリング" \
  --tag backend --tag urgent
```

新規タスクは `draft` ステータスで作成されます。デフォルト優先度は `p2`。

## `task list` – タスク一覧

```bash
senko task list                               # 全タスク (デフォルト limit 50)
senko task list --status todo                 # ステータスで絞り込み
senko task list --ready                       # 依存解決済みのtodoタスク
senko task list --tag backend                 # タグで絞り込み
senko task list --contract 42                 # contract ID で絞り込み
senko task list --id-min 100 --id-max 199     # ID 範囲 (片方のみも可)
senko task list --limit 20 --offset 40        # ページネーション (limit 1..=200、デフォルト 50)
```

CLIフラグのステータス値はスネークケース: `todo`, `in_progress`, `completed`, `canceled`, `draft`

ページネーション: `--limit` は省略時 50、指定時は `1..=200` の範囲で、範囲外はエラーになります。`--offset` のデフォルトは 0 です。結果は `id` 昇順で返るため、ページングが安定します。

## `task get <id>` – タスク詳細

```bash
senko task get 1
```

> `get` はJSON出力のみ（`--output text` 非対応）。

## `task next` – 次のタスクを開始

依存タスクがすべて完了済みの最高優先度 `todo` タスクを選択し、`in_progress` に変更します。

```bash
senko task next
senko task next --session-id "session-abc"
```

選択順序: 優先度（P0優先）→ 作成日時 → ID

## `task edit <id>` – タスク編集

```bash
# スカラーフィールド
senko task edit 1 --title "新しいタイトル"
senko task edit 1 --status todo
senko task edit 1 --priority p0

# 配列フィールド（タグ、完了定義、スコープ）
senko task edit 1 --add-tag "urgent"
senko task edit 1 --remove-tag "old"
senko task edit 1 --set-tags "a" "b"         # 全置換

# 完了定義（Definition of Done）
senko task edit 1 --add-definition-of-done "ユニットテストを書く"

# PR URL
senko task edit 1 --pr-url "https://github.com/org/repo/pull/42"
senko task edit 1 --clear-pr-url

# メタデータ（シャローマージ — キーの追加・上書き、未指定キーは保持）
senko task edit 1 --metadata '{"sprint":"2026-Q2","points":3}'
# メタデータ全置換（既存キーをすべて削除して置き換え）
senko task edit 1 --replace-metadata '{"new_key":"only this"}'
# 特定キーの削除（マージ時にnullを指定）
senko task edit 1 --metadata '{"points":null}'
# メタデータ全クリア
senko task edit 1 --clear-metadata
```

## `task complete <id>` – タスク完了

```bash
senko task complete 1
senko task complete 1 --skip-pr-check    # PR検証をスキップ
```

未チェックのDoD項目がある場合は失敗します。先に `dod check` でマークしてください。

`merge_via = "pr"` 設定時は、PRがマージ済みであることも検証します。`--skip-pr-check` で検証をスキップできます。

## `task cancel <id>` – タスクキャンセル

```bash
senko task cancel 1 --reason "スコープ外"
```

## `task dod` – 完了定義（DoD）の管理

```bash
senko task dod check <task_id> <index>      # DoD項目をチェック（1始まり）
senko task dod uncheck <task_id> <index>    # DoD項目のチェックを外す
```

## `task deps` – 依存関係管理

```bash
senko task deps add 5 --on 3        # タスク5がタスク3に依存
senko task deps remove 5 --on 3     # 依存を削除
senko task deps set 5 --on 1 2 3    # 依存を一括設定
senko task deps list 5              # タスク5の依存一覧
```

## `config` – 設定の表示・初期化

```bash
senko config              # 現在の設定を表示（JSON）
senko --output text config # 現在の設定を表示（テキスト）
senko config --init       # テンプレート .senko/config.toml を生成
```

現在の設定値（未設定項目はデフォルト値）を表示します。`--init` でコメント付きテンプレートファイルを生成します。

## `skill-install` – Claude Code連携

```bash
senko skill-install
```

Claude Code連携用のスキル定義を `.claude/skills/senko/` に生成します。

## `serve` – JSON APIサーバーを起動

```bash
senko serve                # 127.0.0.1:3142 でリッスン
senko serve --port 8080    # 127.0.0.1:8080 でリッスン
senko serve --host 0.0.0.0 # 0.0.0.0:3142 でリッスン（全インターフェース）
```

| オプション | 説明 |
|--------|-------------|
| `--port <PORT>` | リッスンポート（環境変数: `SENKO_SERVER_PORT` または `SENKO_PORT`、デフォルト: `3142`） |
| `--host <ADDR>` | バインドアドレス（例: `0.0.0.0`, `192.168.1.5`）（環境変数: `SENKO_SERVER_HOST` または `SENKO_HOST`、デフォルト: `127.0.0.1`） |

> `SENKO_SERVER_PORT`/`SENKO_SERVER_HOST` は `senko serve` のみに適用されます。`SENKO_PORT`/`SENKO_HOST` は `senko serve` と `senko web` の両方に適用されます。

`/api/v1/...` 配下で全タスク操作（CRUD、ステータス遷移、依存関係、DoD、設定、統計）をJSON REST APIとして提供します。CLIと同様にhooksが発火します。

## `web` – 読み取り専用Webビューアを起動

```bash
senko web                # 127.0.0.1:3141 でリッスン
senko web --port 8080    # 127.0.0.1:8080 でリッスン
senko web --host 0.0.0.0 # 0.0.0.0:3141 でリッスン（全インターフェース）
```

| オプション | 説明 |
|--------|-------------|
| `--port <PORT>` | リッスンポート（環境変数: `SENKO_PORT`、デフォルト: `3141`） |
| `--host <ADDR>` | バインドアドレス（例: `0.0.0.0`, `192.168.1.5`）（環境変数: `SENKO_HOST`、デフォルト: `127.0.0.1`） |

## Docker

### Dockerfile

```dockerfile
FROM debian:bookworm-slim
ARG SENKO_VERSION=0.10.0
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
  && rm -rf /var/lib/apt/lists/* \
  && case "${TARGETARCH}" in \
       amd64) TARGET="x86_64-unknown-linux-musl" ;; \
       arm64) TARGET="aarch64-unknown-linux-musl" ;; \
       *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
     esac \
  && curl -fsSL "https://github.com/hisamekms/senko/releases/download/v${SENKO_VERSION}/senko-v${SENKO_VERSION}-${TARGET}.tar.gz" \
     | tar xz -C /usr/local/bin senko
WORKDIR /project
ENTRYPOINT ["senko"]
```

> **注意**: `TARGETARCH` はDocker BuildKitがビルドプラットフォームに基づいて自動設定します。このDockerfileは `amd64` と `arm64` の両方に対応しています。

### ビルドと実行

```bash
# イメージをビルド
docker build -t senko .

# コマンドを実行
docker run --rm -v "$(pwd)/.senko:/project/.senko" senko task list

# APIサーバーを起動
docker run --rm -p 3142:3142 \
  -v "$(pwd)/.senko:/project/.senko" \
  senko serve --host 0.0.0.0
```

### ボリュームマウントによるデータ永続化

senkoはSQLiteデータベースと設定を `.senko/` ディレクトリに保存します。コンテナ間でデータを永続化するには、このディレクトリをボリュームとしてマウントしてください:

```
-v "$(pwd)/.senko:/project/.senko"
```

マウント対象:
- `tasks.db` – SQLiteデータベース
- `config.toml` – フックとワークフローの設定

ボリュームマウントなしでは、コンテナ停止時にすべてのデータが失われます。

## フック – タスク状態変更時の自動アクション

フックはCLIコマンドがタスク状態を変更した際に自動実行されるシェルコマンドです。デーモン不要で、fire-and-forget（発火後即座に制御を返す）方式で子プロセスとして実行されるため、CLIをブロックしません。

### 設定

`.senko/config.toml` にフックを定義します。各フックは `[hooks.<イベント>.<名前>]` の名前付きエントリ形式で指定します:

```toml
[hooks.on_task_added.notify]
command = "echo '新しいタスク' | notify-send -"

[hooks.on_task_ready.webhook]
command = "curl -X POST https://example.com/ready"

[hooks.on_task_started.slack]
command = "slack-notify started"

[hooks.on_task_completed.webhook]
command = "curl -X POST https://example.com/webhook"

[hooks.on_task_canceled.log]
command = "echo canceled"
```

同一イベントに複数のフックを登録できます（名前で区別）:

```toml
[hooks.on_task_completed.notify]
command = "notify-send '完了'"

[hooks.on_task_completed.webhook]
command = "curl https://example.com/done"
```

各エントリのフィールド:

| フィールド | 型 | デフォルト | 説明 |
|-----------|------|----------|------|
| `command` | string | _（必須）_ | 実行するシェルコマンド |
| `enabled` | bool | `true` | `false` で一時的に無効化 |
| `requires_env` | string[] | `[]` | 指定した環境変数がすべて設定されている場合のみ実行 |

| フック | トリガー |
|------|---------|
| `on_task_added` | `senko task add` で新しいタスクを作成 |
| `on_task_ready` | `senko task ready` でタスクを draft から todo に遷移 |
| `on_task_started` | `senko task start` または `senko task next` でタスクを開始 |
| `on_task_completed` | `senko task complete` でタスクを完了 |
| `on_task_canceled` | `senko task cancel` でタスクをキャンセル |
| `on_no_eligible_task` | `senko task next` で該当タスクなし |

フックは **stdin** でイベントペイロード（JSON）を受け取り、`sh -c` で実行されます。

### イベントペイロード

フックのstdinに渡されるJSONオブジェクト（「フックエンベロープ」）:

```json
{
  "runtime": "cli",
  "backend": {
    "type": "sqlite",
    "db_file_path": "/path/to/project/.senko/senko.db"
  },
  "project": {
    "id": 1,
    "name": "default"
  },
  "user": {
    "id": 1,
    "name": "default"
  },
  "event": {
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "event": "task_completed",
    "timestamp": "2026-03-24T12:00:00Z",
    "from_status": "in_progress",
    "task": {
      "id": 7,
      "project_id": 1,
      "title": "Webhookハンドラの実装",
      "background": null,
      "description": "外部連携用のWebhookエンドポイントを追加",
      "plan": null,
      "priority": "P1",
      "status": "completed",
      "assignee_session_id": null,
      "assignee_user_id": null,
      "created_at": "2026-03-24T10:00:00Z",
      "updated_at": "2026-03-24T12:00:00Z",
      "started_at": "2026-03-24T10:30:00Z",
      "completed_at": "2026-03-24T12:00:00Z",
      "canceled_at": null,
      "cancel_reason": null,
      "branch": "feature/webhook",
      "pr_url": "https://github.com/org/repo/pull/42",
      "metadata": null,
      "definition_of_done": [
        { "content": "ユニットテストを書く", "checked": true },
        { "content": "APIドキュメントを更新", "checked": true }
      ],
      "in_scope": ["RESTエンドポイント"],
      "out_of_scope": ["GraphQLサポート"],
      "tags": ["backend", "api"],
      "dependencies": [3, 5]
    },
    "stats": { "draft": 1, "todo": 3, "in_progress": 1, "completed": 5 },
    "ready_count": 2,
    "unblocked_tasks": [{ "id": 3, "title": "次のタスク", "priority": "P1", "metadata": null }]
  }
}
```

#### エンベロープフィールド

| フィールド | 型 | 説明 |
|-------|------|-------------|
| `runtime` | string | `"cli"` または `"api"` |
| `backend` | object | バックエンド情報（`type` およびバックエンド固有フィールド） |
| `project` | object | プロジェクト情報: `id`（integer）と `name`（string） |
| `user` | object | ユーザー情報: `id`（integer）と `name`（string） |
| `event` | object | イベントペイロード（下記参照） |

`project` と `user` は現在のconfigを反映します。`config.toml` で `[project] name` や `[user] name` が設定されている場合、対応する名前がバックエンドから解決されます。未設定の場合はデフォルトレコード（id=1）が使用されます。

#### `event` フィールド

| フィールド | 型 | 説明 |
|-------|------|-------------|
| `event_id` | string | UUID v4 一意識別子 |
| `event` | string | イベント名（例: `"task_added"`, `"task_completed"`） |
| `timestamp` | string | ISO 8601（RFC 3339）タイムスタンプ |
| `from_status` | string \| null | 遷移前のステータス |
| `task` | object | タスクオブジェクト全体（`senko task get` と同じスキーマ — 下記参照） |
| `stats` | object | ステータス別タスク数（`{"todo": 3, "completed": 5, ...}`） |
| `ready_count` | integer | 依存解決済みの `todo` タスク数 |
| `unblocked_tasks` | array \| null | このイベントで新たにブロック解除されたタスク（`task_completed` のみ） |

#### `task` オブジェクト

イベントペイロードに含まれるタスクオブジェクト全体。`senko task get` の出力と同じスキーマです。

| フィールド | 型 | 説明 |
|-------|------|-------------|
| `id` | integer | タスクID |
| `project_id` | integer | プロジェクトID |
| `title` | string | タスクタイトル |
| `background` | string \| null | 背景情報 |
| `description` | string \| null | タスクの説明 |
| `plan` | string \| null | 実装計画 |
| `priority` | string | `"P0"` – `"P3"` |
| `status` | string | `"draft"`, `"todo"`, `"in_progress"`, `"completed"`, `"canceled"` |
| `assignee_session_id` | string \| null | 割り当てセッションID |
| `assignee_user_id` | integer \| null | 割り当てユーザーID |
| `created_at` | string | ISO 8601 タイムスタンプ |
| `updated_at` | string | ISO 8601 タイムスタンプ |
| `started_at` | string \| null | ISO 8601 タイムスタンプ（タスク開始日時） |
| `completed_at` | string \| null | ISO 8601 タイムスタンプ（タスク完了日時） |
| `canceled_at` | string \| null | ISO 8601 タイムスタンプ（タスクキャンセル日時） |
| `cancel_reason` | string \| null | キャンセル理由 |
| `branch` | string \| null | 関連gitブランチ |
| `pr_url` | string \| null | プルリクエストURL |
| `metadata` | object \| null | 任意のJSONメタデータ（`--metadata`でシャローマージ、`--replace-metadata`で全置換） |
| `definition_of_done` | array | DoD項目のリスト（下記参照） |
| `in_scope` | array | スコープ内の項目（文字列） |
| `out_of_scope` | array | スコープ外の項目（文字列） |
| `tags` | array | タグ文字列 |
| `dependencies` | array | 依存タスクID（整数） |

`definition_of_done` の各要素:

| フィールド | 型 | 説明 |
|-------|------|-------------|
| `content` | string | DoD項目の内容 |
| `checked` | boolean | チェック済みかどうか |

#### `unblocked_tasks` の要素

`task_completed` イベントで、タスク完了により他のタスクのブロックが解除された場合に含まれます。

| フィールド | 型 | 説明 |
|-------|------|-------------|
| `id` | integer | タスクID |
| `title` | string | タスクタイトル |
| `priority` | string | `"P0"` – `"P3"` |
| `metadata` | object \| null | タスクメタデータ（任意のJSON） |

| レベル | 説明 |
|-------|-------------|
| `INFO` | 通常の操作（起動、イベント検出、フック実行成功） |
| `WARN` | フックが非ゼロ終了コードを返した |
| `ERROR` | フックの実行に失敗した |

## 環境変数

全設定は **CLIフラグ > 環境変数 > config.toml > デフォルト値** の優先順位で適用されます。

### サーバー

| 変数 | 説明 | デフォルト |
|------|------|----------|
| `SENKO_PORT` | `web` / `serve` コマンドのポート | `3141`（web）/ `3142`（serve） |
| `SENKO_HOST` | バインドアドレス（例: `0.0.0.0`, `192.168.1.5`） | `127.0.0.1` |
| `SENKO_SERVER_PORT` | `serve` コマンド専用ポート | `3142` |
| `SENKO_SERVER_HOST` | `serve` コマンド専用バインドアドレス | `127.0.0.1` |
| `SENKO_PROJECT_ROOT` | プロジェクトルートディレクトリ | 自動検出 |
| `SENKO_CONFIG` | 設定ファイルのパス | `.senko/config.toml` |

### ワークフロー

| 変数 | 説明 | デフォルト |
|------|------|----------|
| `SENKO_MERGE_VIA` | `direct` または `pr` | `direct` |
| `SENKO_AUTO_MERGE` | `true` または `false` | `true` |
| `SENKO_BRANCH_MODE` | `worktree` または `branch` | `worktree` |
| `SENKO_MERGE_STRATEGY` | `rebase` または `squash` | `rebase` |

### 接続

| 変数 | 説明 | デフォルト |
|------|------|----------|
| `SENKO_CLI_REMOTE_URL` | APIサーバーURL（設定するとSQLiteの代わりにHTTPバックエンドを使用） | _（未設定 = SQLite）_ |
| `SENKO_CLI_REMOTE_TOKEN` | サーバー認証用APIトークン | _（未設定）_ |
| `SENKO_SERVER_RELAY_URL` | リレーモードの上流サーバーURL | _（未設定）_ |
| `SENKO_SERVER_RELAY_TOKEN` | リレー上流への認証トークン | _（未設定）_ |

### ログ

| 変数 | 説明 | デフォルト |
|------|------|----------|
| `SENKO_LOG_DIR` | フックログの出力ディレクトリ | `$XDG_STATE_HOME/senko` |

### フック

| 変数 | 説明 | デフォルト |
|------|------|----------|
| `SENKO_HOOKS_ENABLED` | このプロセスでのフック実行を有効/無効にする | `true` |

| 変数 | 説明 |
|------|------|
| `SENKO_HOOK_ON_TASK_ADDED` | タスク作成時に実行するシェルコマンド |
| `SENKO_HOOK_ON_TASK_READY` | タスクがready時に実行するシェルコマンド |
| `SENKO_HOOK_ON_TASK_STARTED` | タスク開始時に実行するシェルコマンド |
| `SENKO_HOOK_ON_TASK_COMPLETED` | タスク完了時に実行するシェルコマンド |
| `SENKO_HOOK_ON_TASK_CANCELED` | タスクキャンセル時に実行するシェルコマンド |
| `SENKO_HOOK_ON_NO_ELIGIBLE_TASK` | `senko task next` で該当タスクなし時に実行するシェルコマンド |

フック環境変数は `config.toml` の `[hooks]` セクションの設定をオーバーライドします。

### 例: Dockerデプロイ

```bash
docker run -e SENKO_PORT=8080 \
  -e SENKO_HOST=0.0.0.0 \
  -e SENKO_HOOK_ON_TASK_COMPLETED="curl -X POST https://example.com/webhook" \
  senko serve
```

## ステータス遷移

```
draft → todo → in_progress → completed
                            → canceled
（アクティブなステータスからcanceledへの遷移は常に可能）
```
