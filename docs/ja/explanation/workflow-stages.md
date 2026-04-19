# Workflow stage の設計思想

## なぜ "stage" という層を挟むのか

senko は最終的には CLI を叩くツールですが、Claude Code skill 経由で使われる場面が多く、エージェントは「**今は plan している**」「**今は implement している**」のような **論理的なフェーズ** を持ちます。これらは必ずしも CLI のコマンドと 1:1 対応しません。

例:
- `plan` フェーズ: まだ `senko task edit --plan ...` は叩かれていないが、エージェントは設計を練っている
- `implement` フェーズ: `senko task start` は既に終わっていて、コードを書いている最中
- `branch_set` フェーズ: git branch を切る直前。ブランチ名テンプレートや pre-check を差し込みたい

これを CLI の action (= 実コマンド呼出) と分けて **workflow stage** という独立のカテゴリに置いています。

## 組み込み stage 一覧

| Stage | 意味 |
|---|---|
| `task_add` | 新しいタスクを追加する前後 |
| `task_ready` | draft → todo 遷移 |
| `task_start` | todo → in_progress (または `task next` での自動選択) |
| `task_complete` | in_progress → completed |
| `task_cancel` | canceled に遷移 |
| `task_select` | `task next` でタスクを選ぼうとする時点 (選ばれたか否かは `on_result` で分岐) |
| `branch_set` | 作業ブランチを切る直前 |
| `branch_cleanup` | ブランチを消す前 |
| `branch_merge` | マージ操作の直前 |
| `pr_create` | PR 作成前 |
| `pr_update` | PR 更新前 |
| `plan` | 設計を文章化するフェーズ |
| `implement` | 実装フェーズ |
| `contract_add` / `contract_edit` / `contract_delete` | Contract の CRUD |
| `contract_dod_check` / `contract_dod_uncheck` | Contract DoD の更新 |
| `contract_note_add` | Contract にノートを追記する前 |

**注意**: `[workflow.*]` は **任意の名前を受け付ける**ので、プロジェクト独自の stage を追加して独自 skill から参照しても構いません。組み込み以外は senko skill は発火させませんが、`senko config` 出力に素通しで含まれます。

## Stage が持てるフィールド

各 stage は `[workflow.<stage>]` 配下で以下を宣言できます:

| キー | 型 | 役割 |
|---|---|---|
| `instructions` | string[] | エージェントにこの stage で守らせたい指示文 |
| `hooks.<name>` | HookDef | シェル hook の発火 (他 runtime の hook と同じスキーマ) |
| `metadata_fields` | object[] | この stage で入力させる metadata key と値 |

stage 固有の追加キー:

| Stage | キー | 役割 |
|---|---|---|
| `task_add` | `default_dod` / `default_tags` / `default_priority` | 新規タスクのデフォルト値 |
| `plan` | `required_sections` | 計画ドキュメントに必須のセクション名 |

未知のキーは **破棄されず保持** され、外部スクリプトが `senko config --output json` 経由で参照可能です。

## Stage hook と普通の hook の違い

`[workflow.<stage>.hooks.<name>]` と `[cli.<action>.hooks.<name>]` の両方が「hook」ですが、発火主体が違います:

| Hook の場所 | 発火主体 | タイミング |
|---|---|---|
| `[cli/server.*/server.relay.<action>.hooks.<name>]` | senko binary | 状態遷移の前後 (自動発火) |
| `[workflow.<stage>.hooks.<name>]` | Claude Code skill | skill がその stage に入ったと判断した時 (エージェントが能動的に発火させる) |

そのため workflow hook 特有のフィールドとして **`prompt`** が用意されています。skill はこの文字列を **エージェント自身への instruction として読み込みます** (shell コマンドではなくプロンプト拡張として使われる)。

```toml
[workflow.contract_note_add.hooks.review_before_note]
command = "true"                                       # no-op (エージェントに任せる)
prompt = "Skip the note if the same observation already exists in earlier notes."
when = "pre"
```

この例では、Contract にノートを追加する直前に「同じ観察が既存ノートに無いか確認しろ」とエージェントに指示することになります。

## metadata_fields の使い方

stage で必ず埋めさせたい metadata を宣言できます:

```toml
[[workflow.task_add.metadata_fields]]
key = "team"
source = "value"
value = "backend"

[[workflow.plan.metadata_fields]]
key = "estimate_points"
source = "prompt"
prompt = "フィボナッチ数列で見積もってください"
```

`source` は:

- `value`: 固定値を注入
- `prompt`: `prompt` フィールドの文言でエージェントに入力を求める

Project 単位の [MetadataField](concepts.md#metadatafield) で `required_on_complete = true` にしておくと、stage で注入 → complete 時に検証、という連携が作れます。

## 典型的な stage 設計パターン

### 1. plan stage で設計フォーマットを強制

```toml
[workflow.plan]
required_sections = ["Overview", "Acceptance Criteria", "Risks"]
instructions = [
  "plan は task.plan フィールドに保存する",
  "実装着手前に必ず human にレビューを依頼する",
]
```

### 2. branch_set で命名規則を統一

```toml
[workflow]
branch_template = "senko/{{id}}-{{slug}}"

[workflow.branch_set]
instructions = ["feature/ / fix/ / chore/ prefix は不可 (branch_template で統一済)"]
```

### 3. task_complete で CI 通過を必須に

```toml
[workflow.task_complete.hooks.ci_green]
command = "gh pr checks $SENKO_PR_URL --required"
when = "pre"
mode = "sync"
on_failure = "abort"
```

## skill とこの設定の連動

`senko skill-install` で生成される SKILL.md は、内部で `senko config --output json` を叩いて現在の workflow 設定を読み、stage ごとの instructions / prompt をそのフェーズでのエージェント指示として組み立てます。

つまり:

1. プロジェクトごとに `[workflow.*]` を書く
2. 開発者が `senko skill-install` で SKILL.md を更新
3. Claude Code は `/senko` スキル実行時に workflow 設定を参照しながら動く

## 次に読むもの

- Hook 全般 → [reference/hooks.md](../reference/hooks.md)
- Stage の TOML 詳細 → [reference/config/workflow.md](../reference/config/workflow.md)
- 実例 → [guides/cli/workflow-stages.md](../guides/cli/workflow-stages.md)
