---
name: commit
description: コミットメッセージを作成してgit commitを実行する。コード変更後のコミット時に使用。
disable-model-invocation: true
allowed-tools: Bash(git *)
argument-hint: "[prefix] [scope（任意）]"
---

# コミット作成

## コミットメッセージ形式

```
prefix(scope): 概要（50文字以内）

- 変更理由や内容を箇条書きで記述
- 各項目は簡潔に
```

## Prefix一覧

| prefix | 用途 |
|--------|------|
| feat | 新機能の追加 |
| fix | バグ修正 |
| refactor | リファクタリング（機能変更なし） |
| docs | ドキュメントの変更 |
| test | テストの追加・修正 |
| chore | ビルド・設定などの雑務 |
| style | コードスタイルの変更（フォーマット等） |
| perf | パフォーマンス改善 |
| ci | CI/CD設定の変更 |

## Scope（任意）

変更対象の領域やコンポーネント名を括弧内に記述する。省略可。

## 手順

1. `git status` と `git diff --cached` でステージ済みの変更を確認する
2. ステージされていない変更があれば、対象ファイルを確認してステージする
3. 変更内容を分析し、適切なprefix・scopeを判定する
4. コミットメッセージを作成する：
   - 1行目: `prefix(scope): 概要` または `prefix: 概要`
   - 空行
   - 本文: 変更理由や内容を箇条書き（`-`）で記述
5. `git commit` を実行する
6. `git log --oneline -1` で結果を表示する

## 引数の利用

- `$ARGUMENTS` が指定された場合、prefixやscopeのヒントとして利用する
- 例: `/commit feat auth` → `feat(auth): ...`
- 例: `/commit fix` → `fix: ...`

## 注意事項

- コミットメッセージは日本語で記述する
- `Co-Authored-By` ヘッダは付与しない
- `.env` や認証情報を含むファイルはコミットしない
- 1行目の概要は簡潔にし、詳細は本文の箇条書きで補足する
