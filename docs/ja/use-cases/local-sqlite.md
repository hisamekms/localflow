# ユースケース: ローカル SQLite

**個人開発者 1 人がローカルマシンだけで使う** 最小構成。サーバは立てない。

```
┌─────────────────────────┐
│  Developer's machine    │
│                         │
│   senko CLI             │
│     │                   │
│     ▼                   │
│   .senko/senko.db       │
│   (SQLite, git ignored) │
└─────────────────────────┘
```

## いつ選ぶか

- 個人プロジェクト、1 人開発
- `/senko` スキル経由で Claude Code にタスクを管理させたい
- サーバ運用したくない / 必要性がない
- データは手元のリポジトリ単位で完結していて良い

逆にこの構成では **無理** なこと:

- 複数開発者で同じタスク DB を共有
- 別マシンからの read/write (手動で `.senko/senko.db` を同期しない限り)
- サーバ監査ログ
- SSO 連携

これらが必要なら [cli-remote-postgres.md](cli-remote-postgres.md) へ。

## 構成要素

| コンポーネント | 役割 | 設定 |
|---|---|---|
| senko CLI | タスク操作、skill ホスト | インストールのみ |
| SQLite DB | データ保存 (`.senko/senko.db`) | 初回自動作成 |
| Claude Code skill | `/senko` コマンドの提供 | `senko skill-install` で配置 |

## セットアップ手順

### 1. バイナリをインストール

```bash
curl -fsSL https://raw.githubusercontent.com/hisamekms/senko/main/install.sh | sh
```

既定で `~/.local/bin/senko` に配置 (`SENKO_INSTALL_DIR` で変更可)。`~/.local/bin` が `PATH` に入っているか確認。

### 2. プロジェクトで初期化

```bash
cd your-project
senko skill-install
```

以下が配置される:

```
.claude/skills/senko/SKILL.md
```

### 3. `.gitignore` を設定

```
# .gitignore
.senko/
```

DB ファイル (`.senko/senko.db`) は **コミットしない** こと。

### 4. 最初のタスクを追加

CLI から直接:

```bash
senko task add --title "Implement webhook handler" --priority p1
senko task list
```

Claude Code から:

```
/senko task add Implement webhook handler
/senko                                      # ready なタスクを自動選択
```

初回 `senko` 実行時に `.senko/senko.db` と初期マイグレーションが走ります。

## 推奨オプション設定

最低限は設定不要ですが、`.senko/config.toml` があると便利です:

```bash
senko config --init > .senko/config.toml     # コメント付きテンプレート
```

よく使う設定例:

```toml
# Claude に毎タスク必ず書かせたい DoD
[workflow.task_add]
default_dod = [
  "Unit tests pass",
  "CHANGELOG updated",
]

# ブランチ命名規則 (worktree 運用)
[workflow]
branch_template = "feat/{{id}}-{{slug}}"
branch_mode = "worktree"

# 完了時にデスクトップ通知 (macOS)
[cli.task_complete.hooks.notify]
command = "osascript -e 'display notification \"task done\" with title \"senko\"'"
mode = "async"
on_failure = "ignore"
```

## データの場所

| パス | 用途 |
|---|---|
| `<project>/.senko/senko.db` | SQLite 本体 (git ignored) |
| `<project>/.senko/config.toml` | 設定 (任意、git commit 可) |
| `<project>/.senko/config.local.toml` | 開発者個人の上書き (git ignored) |
| `$XDG_STATE_HOME/senko/` | hook 実行ログ (既定 `~/.local/state/senko/`) |

`.senko/` がどうしても作れない場所では、自動的に `$XDG_DATA_HOME/senko/projects/<hash>/data.db` にフォールバックします。

## バックアップ・移行

- DB ファイルを **そのままコピー** すれば別マシンで復元可能
- バージョン更新で未適用マイグレーションがあれば次回起動時に自動実行
- バージョンを下げる (ダウングレード) は **非対応**。必要なら別 DB で試すこと

```bash
# 手動バックアップ例
cp .senko/senko.db .senko/senko.db.bak.$(date +%Y%m%d)

# 別マシンへ
scp .senko/senko.db other-host:/path/to/project/.senko/
```

## リモート構成への移行タイミング

以下に 1 つでも該当したら [cli-remote-postgres.md](cli-remote-postgres.md) を検討:

- 2 人目の開発者が同じタスク DB を使いたい
- PR / CI から `senko` を叩きたい (複数クライアントで書き込みが発生)
- 監査ログを取りたい
- SSO 配下でアクセス制御したい

## 参考

- 詳細な初期セットアップ → [getting-started/local.md](../getting-started/local.md)
- workflow stage の設定 → [guides/cli/workflow-stages.md](../guides/cli/workflow-stages.md)
- hook 実例 → [guides/cli/hooks.md](../guides/cli/hooks.md)
