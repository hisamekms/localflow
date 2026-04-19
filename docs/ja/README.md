# senko ドキュメント

senko は **AI エージェントが自律的に作業を進めるためのワークフローオーケストレータ** です。
「タスク管理ツール」というより、**プロジェクト固有の進め方を codify してエージェントに教える** 道具に近い位置づけで、Claude Code と連携して使うことを主眼に設計されています。

> **日本語 (このディレクトリ)** / [English](../en/README.md)

## コアコンセプト: 3 つの柱

senko は AI エージェントの自律的動作を、以下 3 つの柱で支えます。

1. **イベントドリブンなワークフロー** — プロジェクト固有のルール (DoD / ブランチ規則 / 必須 metadata / 段階ごとの指示) を、エージェントの行動に合わせて自動で注入・検証する。hook と workflow stage がこの役割を担う
2. **タスク分割 + 順次/並列実行** — 大きな作業を依存関係と優先度を持つタスクに分割し、エージェントは「次にやる 1 件」だけに集中できる。ワンショットの巨大プロンプトに詰め込まない。複数セッションで並列 pick も可
3. **Contract で全体像を保持** — 個々のタスクは短命 (分〜時間) で context をリセットしながら進むが、Contract (週〜月の寿命) と Notes が長期の文脈と知見を保持し、作業の全体像を見失わせない

→ 深掘りは [explanation/core-concept.md](explanation/core-concept.md)

## 30 秒で試す

```bash
# 1. バイナリをインストール
curl -fsSL https://raw.githubusercontent.com/hisamekms/senko/main/install.sh | sh

# 2. プロジェクト直下で skill をインストール
cd your-project
senko skill-install

# 3. Claude Code で
#    /senko task add Implement webhook handler
#    /senko
```

初回実行時に SQLite DB が `$XDG_DATA_HOME/senko/projects/<dir>/data.db` (通常は `~/.local/share/senko/projects/<dir>/data.db`) に自動作成されます。プロジェクトディレクトリ配下には書かれません。

## ドキュメント構成

読者の目的別に 4 層に分かれています。

### まず手を動かしたい — [use-cases/](use-cases/)

3 つの典型構成について、概要・構成図・エンドツーエンドのセットアップ手順を示します。

- [local-sqlite.md](use-cases/local-sqlite.md) — ローカル SQLite (個人開発、1 人)
- [cli-remote-postgres.md](use-cases/cli-remote-postgres.md) — CLI → Remote サーバ → PostgreSQL (チーム運用)
- [cli-relay-remote-postgres.md](use-cases/cli-relay-remote-postgres.md) — CLI → Relay → Remote → PostgreSQL (AI サンドボックス構成、CLI はシークレットレス)

### 考え方を理解したい — [explanation/](explanation/)

3 つの柱を軸に、senko が「なぜこう設計されているか」を説明します。

- [core-concept.md](explanation/core-concept.md) — **3 つの柱と全体マップ** (最初に読む)
- [event-driven-workflow.md](explanation/event-driven-workflow.md) — 柱 1: hook × workflow stage
- [task-decomposition.md](explanation/task-decomposition.md) — 柱 2: 分割・依存・優先度・並列
- [contract.md](explanation/contract.md) — 柱 3: Contract と Notes
- [runtimes.md](explanation/runtimes.md) — デリバリ基盤 (CLI / server.remote / server.relay)
- [architecture.md](explanation/architecture.md) — 4 層アーキテクチャ (コード構造)

### 設定・デプロイ方法を知りたい — [guides/](guides/)

デプロイ形態別に目的の How-To を引きます。

- **CLI を使う人** — [guides/cli/](guides/cli/): skill-install / workflow-stages / hooks / backends
- **サーバ運用者** — [guides/server-remote/](guides/server-remote/): deploy / 認証 3 種 / AWS / hooks
- **リレー運用者** — [guides/server-relay/](guides/server-relay/): deploy / token-relay / hooks

### 仕様を引きたい — [reference/](reference/)

- [cli.md](reference/cli.md) — CLI サブコマンド全量
- [api.md](reference/api.md) — REST API エンドポイント全量
- [data-model.md](reference/data-model.md) — DB スキーマ
- [hooks.md](reference/hooks.md) — Hook envelope / trigger マトリクス
- [config/](reference/config/) — 設定 section を runtime 別に (overview / cli / server-remote / server-relay / workflow / common)

### コントリビュート — [contributing/](contributing/)

- [development.md](contributing/development.md) — 開発環境
- [testing.md](contributing/testing.md) — unit / e2e
- [releasing.md](contributing/releasing.md) — リリース手順
- [worktree.md](contributing/worktree.md) — worktree ワークフロー

## ライセンス

MIT
