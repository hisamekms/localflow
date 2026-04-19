# senko ドキュメント

senko は、Claude Code とやり取りする前提で設計された **タスク管理ツール** です。
ローカルの SQLite / リモートの PostgreSQL / HTTP API 経由のリレー構成まで、同じ CLI とドキュメントで扱えます。

> **日本語 (このディレクトリ)** / [English](../en/README.md)

## 何を解決するか

- **Claude Code に "次に何をすべきか" を見失わせない** — 依存関係と優先度を持ったタスクから `/senko` で常に適切な 1 件を選べる
- **タスク完了条件 (Definition of Done) をコードと同じ場所に置ける** — `.senko/` 配下に SQLite/TOML で管理し、チェック漏れを hook で検知できる
- **AI の実行フローをプロジェクトごとにカスタマイズできる** — workflow stage (plan / implement / branch_set / pr_create …) 単位で instructions と hook を差し込める
- **"ローカルで 1 人"〜"チームでサーバ運用"まで同じ仕組みで拡張できる** — ローカル CLI → 認証付きサーバ → AI サンドボックス向けリレーまで、設定ファイルの runtime セクションを切り替えるだけ

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

最初の実行で `.senko/senko.db` が自動作成されます。`.gitignore` に `.senko/` を追加しておいてください。

## ドキュメント構成

初めての人は `use-cases/` で自分の構成に近いものを選ぶ → `getting-started/` で動かす、の流れを推奨。既に使っている人は目的別に `guides/`、仕様を引きたい人は `reference/`、設計思想を知りたい人は `explanation/` を参照してください。

### [use-cases/](use-cases/) — よくある構成パターン

3 つの典型構成について、概要・構成図・エンドツーエンドのセットアップ手順を示します。

- [local-sqlite.md](use-cases/local-sqlite.md) — ローカル SQLite (個人開発、1 人)
- [cli-remote-postgres.md](use-cases/cli-remote-postgres.md) — CLI → Remote サーバ → PostgreSQL (チーム運用)
- [cli-relay-remote-postgres.md](use-cases/cli-relay-remote-postgres.md) — CLI → Relay → Remote → PostgreSQL (AI サンドボックス構成、CLI はシークレットレス / Relay がシークレットを集約)

### [getting-started/](getting-started/) — まず動かす

- [local.md](getting-started/local.md) — ローカル CLI だけで使う (個人開発)
- [remote-cli.md](getting-started/remote-cli.md) — チームが立てたサーバに CLI から接続する
- [server.md](getting-started/server.md) — 自分でサーバを立てる

### [guides/](guides/) — 目的別 How-to

**CLI を使う人** — [guides/cli/](guides/cli/)

- [skill-install.md](guides/cli/skill-install.md) — Claude Code skill のインストールと更新
- [workflow-stages.md](guides/cli/workflow-stages.md) — plan / implement / branch / PR など stage の設定
- [hooks.md](guides/cli/hooks.md) — `[cli.*]` hook の実例集
- [backends.md](guides/cli/backends.md) — SQLite / PostgreSQL / HTTP backend の切替

**サーバ運用者 (senko serve)** — [guides/server-remote/](guides/server-remote/)

- [deploy.md](guides/server-remote/deploy.md) — サーバ起動と最小構成
- [auth-api-key.md](guides/server-remote/auth-api-key.md) — API キー認証
- [auth-oidc.md](guides/server-remote/auth-oidc.md) — OIDC (OAuth PKCE) 認証
- [auth-trusted-headers.md](guides/server-remote/auth-trusted-headers.md) — API Gateway 配下で使う信頼ヘッダ
- [aws-deployment.md](guides/server-remote/aws-deployment.md) — API Gateway + Cognito + Lambda Web Adapter 構成
- [hooks.md](guides/server-remote/hooks.md) — `[server.remote.*]` hook の実例

**リレー運用者 (senko serve --proxy)** — [guides/server-relay/](guides/server-relay/)

- [deploy.md](guides/server-relay/deploy.md) — リレー起動と上流サーバへの中継
- [token-relay.md](guides/server-relay/token-relay.md) — CLI → Relay → Remote のトークン中継
- [hooks.md](guides/server-relay/hooks.md) — `[server.relay.*]` hook の実例

### [reference/](reference/) — 引ける辞書

- [cli.md](reference/cli.md) — CLI サブコマンド全量
- [api.md](reference/api.md) — REST API エンドポイント全量
- [data-model.md](reference/data-model.md) — DB スキーマ
- [hooks.md](reference/hooks.md) — Hook envelope / env_vars / trigger マトリクス
- **config/**
  - [overview.md](reference/config/overview.md) — 優先順位・ファイル配置・runtime フィルタ
  - [cli.md](reference/config/cli.md) — `[cli.*]`
  - [server-remote.md](reference/config/server-remote.md) — `[server.remote.*]` `[server.auth.*]` `[backend.*]`
  - [server-relay.md](reference/config/server-relay.md) — `[server.relay.*]`
  - [workflow.md](reference/config/workflow.md) — `[workflow.*]`
  - [common.md](reference/config/common.md) — `[project]` `[user]` `[log]` `[web]`

### [explanation/](explanation/) — 概念・設計

- [concepts.md](explanation/concepts.md) — Task / Contract / Project / User / Dependency / Metadata field
- [runtimes.md](explanation/runtimes.md) — CLI / remote / relay の使い分け判断
- [architecture.md](explanation/architecture.md) — 4 層アーキテクチャ
- [workflow-stages.md](explanation/workflow-stages.md) — stage とスキルの連動思想

### [contributing/](contributing/) — コントリビュータ向け

- [development.md](contributing/development.md) — 開発環境セットアップ
- [testing.md](contributing/testing.md) — unit / e2e テストの流儀
- [releasing.md](contributing/releasing.md) — リリース手順
- [worktree.md](contributing/worktree.md) — worktree ワークフロー

## ライセンス

MIT
