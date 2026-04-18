# CLI Command Structure Policy

senko CLI の設計方針と、各サブコマンドの配置ルールをまとめる。

## 設計原則

senko のリソースは **集約 (aggregate)** の単位で管理される。CLI もそれに対応させ、`senko <aggregate> <verb>` の 2 段構えを基本形とする。

- **集約**: Task / Contract / Project / User / Auth / Hooks
- **動詞 (verb)**: `add` / `list` / `get` / `edit` / `delete` / `complete` / ...

```
senko task add --title "..."
senko task list
senko contract add --title "..."
senko contract list
senko project create --name "..."
senko user list
```

### 例外: 運用・モード系コマンド

以下は「集約に対する操作」ではなく、senko バイナリ自体のモードや付随機能なので top-level のまま据え置く:

| コマンド | 役割 |
|---|---|
| `senko serve` | JSON REST API サーバ起動 |
| `senko web` | 読み取り専用 Web ビューア起動 |
| `senko config` | 設定ファイルの表示・初期化 |
| `senko doctor` | 設定・環境のヘルスチェック |
| `senko skill-install` | skill の配置 |
| `senko auth` | 認証サブコマンド (login/token/status/...) |
| `senko hooks` | hook サブコマンド (log/test) |

## サブコマンド配置ルール

### Task 集約

タスクに対する操作はすべて `senko task` 配下に入れる。

```
senko task add
senko task list
senko task get <id>
senko task next
senko task ready <id>
senko task start <id>
senko task edit <id>
senko task complete <id>
senko task cancel <id>
senko task dod check|uncheck <task_id> <index>
senko task deps add|remove|set|list <task_id>
```

Contract が `senko contract <verb>` になっているのに合わせ、Task もサブコマンド化することで**集約ごとの対称性**を保つ。

### Project 集約

プロジェクト自体の CRUD と、**プロジェクトに従属する概念** (メタデータフィールド、メンバー) を `senko project` 配下にまとめる。

```
senko project list
senko project create --name "..."
senko project delete <id>
senko project metadata-field add|list|remove
senko project members list|add|remove|set-role
```

`members` はプロジェクト固有の概念であり、`metadata-field` と並列に `senko project` 配下に属する。

### Contract 集約

```
senko contract add
senko contract list
senko contract get <id>
senko contract edit <id>
senko contract delete <id>
senko contract dod check|uncheck
senko contract note add|list
```

## エイリアスを提供しない理由

旧 top-level コマンド (`senko add`, `senko list`, `senko members` など) は**エイリアスを設けず完全に削除**する。

- skill / e2e テスト / 公式ドキュメント はすべて同時に新体系へ移行する
- エイリアスが残ると "新旧どちらを推奨するか" が曖昧になり、ヘルプテキストが肥大化する
- 旧名を呼び出したらエラー終了する方が、誤使用を早期に顕在化できる

## 移行マップ

| 旧 | 新 |
|---|---|
| `senko add` | `senko task add` |
| `senko list` | `senko task list` |
| `senko get <id>` | `senko task get <id>` |
| `senko next` | `senko task next` |
| `senko ready <id>` | `senko task ready <id>` |
| `senko start <id>` | `senko task start <id>` |
| `senko edit <id>` | `senko task edit <id>` |
| `senko complete <id>` | `senko task complete <id>` |
| `senko cancel <id>` | `senko task cancel <id>` |
| `senko dod check <task_id> <index>` | `senko task dod check <task_id> <index>` |
| `senko dod uncheck <task_id> <index>` | `senko task dod uncheck <task_id> <index>` |
| `senko deps add \| remove \| set \| list` | `senko task deps add \| remove \| set \| list` |
| `senko members list` | `senko project members list` |
| `senko members add --user-id <id> [--role <role>]` | `senko project members add --user-id <id> [--role <role>]` |
| `senko members remove --user-id <id>` | `senko project members remove --user-id <id>` |
| `senko members set-role --user-id <id> --role <role>` | `senko project members set-role --user-id <id> --role <role>` |

## 将来の拡張ガイド

- **新しい集約を追加するとき**: 集約名をサブコマンドにし、その配下に verb を並べる (`senko <new-aggregate> <verb>`)
- **集約に従属する概念を追加するとき**: 親集約のサブコマンドのさらに下に入れる (例: `senko project members`)
- **モード的なコマンドを追加するとき**: 集約操作でなければ top-level に置く (例: `senko doctor`)
