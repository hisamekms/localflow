---
name: release
description: "localflowのリリースを実行する。e2eテスト実行→バージョン自動判定→Cargo.toml更新→コミット→タグ作成→pushを一括で行う。Triggers on \"/release\", \"リリース\", \"リリースして\", \"release\", \"バージョンアップ\", \"新バージョン\" or similar release requests."
argument-hint: "[version（省略時は自動判定）]"
---

# Release — localflow リリーススキル

localflow の新バージョンをリリースする。e2e テストの成功を確認してから、バージョン更新・コミット・タグ作成・push を実行する。

## 手順

### Step 1: e2e テストの実行

```bash
bash tests/e2e/run.sh
```

テストが **1つでも失敗したらリリースを中止**し、失敗内容をユーザーに報告して終了する。

### Step 2: バージョン番号の決定

引数でバージョンが指定されている場合はそれを使う（`v` プレフィックスは除去して扱う）。

引数がない場合は、前回のタグからの変更内容を分析して自動判定する：

```bash
# 最新のタグを取得
git describe --tags --abbrev=0 2>/dev/null

# 前回タグからの変更を確認
git log <last-tag>..HEAD --oneline
```

**バージョン判定ルール（semver）：**

| 変更の種類 | バージョンアップ |
|---|---|
| 破壊的変更（API変更、CLI引数変更など） | **メジャー** (x.0.0) |
| 新機能追加（feat） | **マイナー** (0.x.0) |
| バグ修正・リファクタ・ドキュメント | **パッチ** (0.0.x) |

コミットメッセージの prefix（feat / fix / refactor / docs 等）を参考に判定する。

**メジャーバージョンアップの場合は AskUserQuestion でユーザーに確認を取る。** 確認なしにメジャーバージョンを上げてはいけない。

### Step 3: Cargo.toml のバージョン更新

`Cargo.toml` の `version = "..."` 行を新しいバージョンに更新する。Edit ツールを使うこと。

### Step 4: コミットとタグ作成

```bash
# バージョン更新をコミット
git add Cargo.toml
git commit -m "chore: bump version to <version>"

# タグ作成
git tag v<version>
```

### Step 5: push

コミットとタグの両方を push する：

```bash
git push origin HEAD
git push origin v<version>
```

### Step 6: 完了報告

リリースが完了したら以下を報告する：

- リリースバージョン（例: v0.2.0）
- GitHub Actions の release ワークフローが起動されることの案内
- `gh run list --workflow=release.yml --limit 1` でワークフローの状態を確認して表示
