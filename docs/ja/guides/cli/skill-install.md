# Claude Code skill のインストールと更新

`senko skill-install` はプロジェクトに `.claude/skills/senko/SKILL.md` を配置して、Claude Code に `/senko` スラッシュコマンドを認識させるコマンドです。

## 初回インストール

プロジェクトルートで:

```bash
senko skill-install
```

生成されるもの:

```
.claude/
└── skills/
    └── senko/
        └── SKILL.md        # 単一ファイルの skill 定義
```

Claude Code を再起動するか `/help` → skill 一覧で認識を確認してください。

## `/senko` が提供する機能

| スラッシュコマンド | 役割 |
|---|---|
| `/senko` | ready なタスクから 1 件自動選択して実行開始 |
| `/senko task add` | 対話的にタスクを整理して追加 |
| `/senko task list` | 一覧 |
| `/senko task complete <id>` | DoD チェックしつつ完了 |
| `/senko task cancel <id>` | キャンセル |
| `/senko graph` | 依存関係をテキストグラフで可視化 |
| `/senko contract add` | Contract の作成 |
| `/senko contract note add` | Contract に knowledge ノートを追加 |

細かい引数は skill 自身のヘルプを参照。

## 更新

senko のバージョンを上げた後は skill を更新:

```bash
senko skill-install
```

- 既存の `SKILL.md` と内容が同一ならスキップ
- 異なる場合はプロンプトで確認 (`--yes` でスキップ)
- `--force` で senko 所有ディレクトリごと消して再配置

## 配置先の変更

既定は `.claude/` 配下ですが、`--output-dir` で変更可能:

```bash
senko skill-install --output-dir /custom/path
```

ただし Claude Code の規約に従い `.claude/skills/<name>/SKILL.md` が認識されるので、変更する場合は出力先も同じ構造にしてください。

## プロジェクトの workflow 設定との関係

skill は実行時に `senko config --output json` を叩き、`[workflow.*]` の instructions / prompt を読み込んでエージェント指示に混ぜます。

つまり:

1. `.senko/config.toml` の `[workflow.*]` を変更
2. **skill の再インストールは不要**。次回の `/senko` 実行時に最新の設定が読まれる

ただし SKILL.md 自体の骨格を更新 (= senko バイナリを新バージョンにする) した場合は `senko skill-install` で再生成してください。

## 複数プロジェクトで使う

senko skill はプロジェクトローカル (`.claude/`) に配置されるので、プロジェクトごとに別の `[workflow.*]` 設定が使えます。全プロジェクト共通の設定は `~/.config/senko/config.toml` に書くと、project 個別設定よりも低優先度で適用されます。

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| `/senko` が Claude Code に出てこない | `.claude/skills/senko/SKILL.md` が存在するか、Claude Code を再起動 |
| skill が古い挙動をする | `senko skill-install --force` で再配置 |
| workflow 設定が反映されない | `senko config` で `[workflow.*]` が期待通りマージされているか確認。`senko doctor` も実行 |
