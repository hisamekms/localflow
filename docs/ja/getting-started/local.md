# ローカル CLI で使う

個人開発で、1 人の開発者が自分のマシン上だけで senko を使う最小構成です。サーバは立てません。

## 前提

- Rust 製の CLI バイナリ (`senko`) がインストール済み
- git で管理されたプロジェクトディレクトリ

## 1. インストール

```bash
curl -fsSL https://raw.githubusercontent.com/hisamekms/senko/main/install.sh | sh
```

インストール先は `~/.local/bin` (変更するには `SENKO_INSTALL_DIR` を設定)。

バージョンを固定する場合:

```bash
VERSION=v1.0.0 curl -fsSL https://raw.githubusercontent.com/hisamekms/senko/main/install.sh | sh
```

## 2. プロジェクトで初期化

プロジェクトのルートで:

```bash
cd your-project
senko skill-install
```

これで以下が生成されます:

```
.claude/skills/senko/SKILL.md   # Claude Code が読む skill 定義
```

データベースは最初の CLI コマンドで `.senko/senko.db` に自動作成されます (マイグレーション込み)。
`.gitignore` に `.senko/` を追加してコミット対象から外してください。

## 3. 最初のタスクを追加して実行

Claude Code から:

```
/senko task add
```

Claude が背景・DoD・スコープを対話で整理してタスクを作成します。次に:

```
/senko
```

で優先度と依存関係から 1 件が選ばれ、Claude が実行に移ります。

CLI 単体でも同じ操作ができます:

```bash
senko task add --title "Implement webhook handler" --priority p1
senko task list
senko task next
```

## 4. 最低限の設定 (任意)

ホック・workflow stage を使わないなら設定ファイルは不要です。使う場合は `.senko/config.toml` を作成:

```bash
senko config --init    # コメント付きテンプレートを出力
```

例として、タスク完了時に `notify-send` を叩きたいだけなら:

```toml
[cli.task_complete.hooks.notify]
command = "notify-send 'senko: task done'"
mode = "async"
```

詳しくは [guides/cli/hooks.md](../guides/cli/hooks.md) を参照。

## データの置き場所

| パス | 用途 |
|---|---|
| `.senko/senko.db` | SQLite データベース (タスク・プロジェクト・ユーザ等) |
| `.senko/config.toml` | 設定ファイル (任意) |
| `$XDG_STATE_HOME/senko/` | hook 実行ログの既定出力先 |

`.senko/` はプロジェクトルート配下に置かれ、**git には含めない** 運用を推奨します。

## よく使うコマンド

```bash
senko task list             # タスク一覧
senko task get 3            # id=3 の詳細
senko task next             # ready 状態の最優先タスクを開始
senko task complete 3       # in_progress → completed
senko graph                 # 依存関係をテキストグラフ表示 (skill 経由)
senko config                # 現在の設定を表示
senko doctor                # 設定・hook の健全性チェック
```

完全なリファレンスは [reference/cli.md](../reference/cli.md) を参照。

## 次に読むもの

- 概念を知りたい → [explanation/concepts.md](../explanation/concepts.md)
- workflow stage を設定して Claude の挙動を調整したい → [guides/cli/workflow-stages.md](../guides/cli/workflow-stages.md)
- チームで共有したい (リモートサーバに繋ぐ) → [getting-started/remote-cli.md](remote-cli.md)
