# senko

> **Alpha**: 本プロジェクトは開発初期段階です。API、CLIインターフェース、データ形式は予告なく変更される可能性があります。

Claude Code向けのローカルタスク管理ツール。SQLiteベース、依存関係対応、優先度駆動。
Claude Codeスキルとして動作し、AIエージェントによるタスク管理・実行を可能にします。

[English](../README.md)

## 機能

- **タスクライフサイクル**: `draft` → `todo` → `in_progress` → `completed` / `canceled`
- **優先度**: P0（最高）〜 P3（最低）
- **依存関係管理**: 依存タスクが完了するまでブロック
- **次タスク自動選択**: 最高優先度の実行可能タスクを自動選択
- **2種類の出力**: JSON（AI/自動化向け）とテキスト（人間向け）
- **Claude Codeスキル**: `/senko` スキルによるシームレスなAI駆動タスク管理
- **セットアップ不要**: SQLiteデータベースは初回実行時に自動作成

> **注意**: senkoはプロジェクトルート直下の `.senko/` にデータを保存します。`.gitignore` に `.senko/` を追加して、ローカルデータをコミットしないようにしてください。

## インストール

```bash
curl -fsSL https://raw.githubusercontent.com/hisamekms/senko/main/install.sh | sh
```

バージョンを指定する場合:

```bash
VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/hisamekms/senko/main/install.sh | sh
```

デフォルトでは `~/.local/bin` にインストールされます。`SENKO_INSTALL_DIR` で変更できます。

### ソースからビルド

```bash
cargo build --release
```

バイナリは `target/release/senko` に生成されます。`PATH` に追加してください。

## Claude Code連携

senkoは主にClaude Codeスキルとして使用します。`skill-install` でセットアップします:

```bash
senko skill-install
```

プロジェクトに `.claude/skills/senko/SKILL.md` が生成され、Claude Codeに `/senko` スキルが登録されます。

### スキルで何ができるか

`/senko` スキルはClaude Codeに完全なタスク管理ワークフローを提供します:

- 次の実行可能タスクを**自動選択して実行**
- 対話的な計画フェーズ付きで**タスクを追加**（シンプルモードも対応）
- **タスク一覧**の表示と**依存関係グラフ**の可視化
- DoD（完了定義）チェック付きのタスク**完了・キャンセル**
- タスク間の**依存関係管理**

## 典型的な使い方

スキルをインストールしたら、Claude Code内で直接使用できます:

```
/senko add ユーザー認証の実装
```
対話的な計画フェーズ付きでタスクを追加。Claudeが確認事項を質問し、依存関係を検出し、タスクを確定します。

```
/senko
```
最高優先度の実行可能タスクを自動選択して作業を開始します。

```
/senko list
```
全タスクのステータスと優先度を表示します。

```
/senko graph
```
タスクの依存関係をテキストベースのグラフで可視化します。

```
/senko complete 3
```
タスク#3を完了としてマーク（DoD項目を先にチェックします）。

## Hooks

フックは、CLIコマンドがタスクの状態を変更した際に自動実行されるシェルコマンドです。デーモン不要 — fire-and-forgetの子プロセスとしてインラインで実行されます。各フックは名前付きエントリで、個別に有効/無効を切り替えられます。`.senko/config.toml` で設定します:

```toml
[hooks.on_task_added.notify]
command = "echo '新しいタスク' | notify-send -"

[hooks.on_task_completed.webhook]
command = "curl -X POST https://example.com/webhook"

[hooks.on_task_completed.log]
command = "echo 'タスク完了' >> /tmp/tasks.log"

[hooks.on_no_eligible_task.notify]
command = "notify-send '該当タスクなし'"
```

フックはstdinでイベントペイロード（JSON）を受け取り、`sh -c` で実行されます。すべてのライフサイクルイベントに対応しています: `on_task_added`, `on_task_ready`, `on_task_started`, `on_task_completed`, `on_task_canceled`, `on_no_eligible_task`。

詳細は [CLIリファレンス – Hooks](CLI.ja.md#hooks--タスク状態変更時の自動アクション) を参照してください。

## ワークフロー設定

`.senko/config.toml`の`[workflow]`セクションでタスク完了時の動作を制御できます：

```toml
[workflow]
merge_via = "pr"        # または "direct"（デフォルト）
auto_merge = false      # デフォルト: true
```

| 設定 | 値 | 説明 |
|------|------|------|
| `merge_via` | `direct`（デフォルト）, `pr` | `pr`の場合、`complete`コマンドが`gh`でPRのマージ状況を検証 |
| `auto_merge` | `true`（デフォルト）, `false` | `merge_via = "direct"` でのみ有効。ブランチの自動マージを制御 |

`senko config`で現在の設定を表示、`senko config --init`でテンプレートを生成できます。

カスタムパスの設定ファイルを使用するには、`--config` フラグまたは `SENKO_CONFIG` 環境変数を使用します:

```bash
senko --config /path/to/config.toml list
SENKO_CONFIG=/path/to/config.toml senko list
```

## マスターAPIキー

マスターAPIキーを使うと、既存のユーザーアカウントなしでシステムをブートストラップ（ユーザー作成・APIキー発行）できます。認証が有効な場合、マスターキーが最初にチェックされ、一致しなければ通常のユーザーAPIキー照合にフォールバックします。

### キーの生成

```bash
openssl rand -base64 32
```

### AWS Secrets Managerへの保存

```bash
aws secretsmanager create-secret \
  --name senko/master-api-key \
  --secret-string "$(openssl rand -base64 32)"
```

### 設定方法

環境変数で設定:

```bash
# 直接値を設定
export SENKO_AUTH_API_KEY_MASTER_KEY="<your-key>"

# または AWS Secrets Manager ARN で設定（aws-secrets feature が必要）
export SENKO_AUTH_API_KEY_MASTER_KEY_ARN="arn:aws:secretsmanager:us-east-1:123456789:secret:senko/master-api-key-AbCdEf"
```

`.senko/config.toml` で設定:

```toml
[server.auth.api_key]
master_key = "<your-key>"
# または ARN で指定（aws-secrets feature が必要）:
# master_key_arn = "arn:aws:secretsmanager:..."
```

### ブートストラップ手順

マスターAPIキーを設定した後のセットアップフロー:

```bash
# 1. マスターキーを生成して設定
export SENKO_AUTH_API_KEY_MASTER_KEY="$(openssl rand -base64 32)"

# 2. サーバーを起動
senko serve

# 3. ユーザーを作成
curl -s -X POST http://localhost:3142/api/v1/users \
  -H "Authorization: Bearer $SENKO_AUTH_API_KEY_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"username": "alice"}' | jq .

# 4. ユーザーのAPIキーを発行（1はステップ3で返されたユーザーIDに置き換え）
curl -s -X POST http://localhost:3142/api/v1/users/1/api-keys \
  -H "Authorization: Bearer $SENKO_AUTH_API_KEY_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name": "default"}' | jq .

# 5. 発行されたAPIキーを以降のリクエストに使用
export SENKO_TOKEN="<ステップ4で取得したキー>"
curl -s http://localhost:3142/api/v1/projects \
  -H "Authorization: Bearer $SENKO_TOKEN" | jq .
```

## 認証

senkoはLocal、Remote + APIキー、Remote + OIDCの3つの認証モードをサポートしています。詳細は[認証モード別セットアップガイド](AUTH_SETUP.ja.md)を参照してください。

## CLIリファレンス

CLIを直接使用する場合は[CLIリファレンス](CLI.ja.md)を参照してください。

## 開発

[開発ガイド](DEVELOPMENT.ja.md)にステータス遷移、データ保存、テストの情報があります。

## ライセンス

MIT
