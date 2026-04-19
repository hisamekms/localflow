# トークン中継 (Token Relay) パターン

クライアントの credential と、上流 senko サーバが受け付ける credential を **分離** したいケース。relay が仲介して token を差し替えます。

## なぜ必要か

**典型シナリオ**: AI エージェントを sandbox 内で動かし、サンドボックス外の senko (本番 DB) にタスクを記録させたい。

- サンドボックス内エージェントに本番 senko の token を直接持たせたくない (スコープ過大)
- でも全操作を無視したいわけではなく、「決められた操作だけ許可」したい
- サンドボックス側で発行する使い捨てトークンで relay を認証し、relay が上流のサービスアカウント token に差し替える

## 3 つのパターン

### A. 透過 passthrough

```toml
[server.relay]
url = "https://senko-upstream.example.com"
# token は書かない
```

- クライアントの `Authorization` ヘッダがそのまま上流へ
- 上流側で個別ユーザ認証したい時に使う
- 例: 上流が OIDC で、relay はネットワーク経路上の通過点にすぎないケース

### B. 一括サービストークン (最もよく使う)

```toml
[server.relay]
url   = "https://senko-upstream.example.com"
token = "upstream-service-account-token"
```

- 上流から見ると全リクエストが同一の service account 由来
- 実ユーザの identity は **失われる** ので、relay 側で監査が必要
- クライアントの token は relay の `[server.auth.*]` で検証するが、上流には伝わらない

### C. ヘッダ書き換え (現状未対応、回避策あり)

クライアント JWT の claim を見て、上流へは対応する service token に切り替える動的な挙動は現状 relay 単体では未対応。必要なら:

- relay の **前段** で API Gateway / Lambda を挟んで動的に書き換える
- または上流側で `trusted_headers` を有効化し、relay で `x-senko-user-sub` 等を注入する構成に変える

## サービストークンの発行 (上流側)

relay 用の service account を作って専用 token を発行:

```bash
# 1. 上流で専用ユーザを作る
curl -s -X POST https://senko-upstream.example.com/api/v1/users \
  -H "Authorization: Bearer $UPSTREAM_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"username":"relay-sandbox-a"}'

# 2. そのユーザを対象プロジェクトのメンバーに
curl -s -X POST https://senko-upstream.example.com/api/v1/projects/1/members \
  -H "Authorization: Bearer $UPSTREAM_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"user_id": 7, "role": "member"}'

# 3. API キーを発行
curl -s -X POST https://senko-upstream.example.com/api/v1/users/7/api-keys \
  -H "Authorization: Bearer $UPSTREAM_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name":"relay-sandbox-a"}'
# => key を relay の SENKO_SERVER_RELAY_TOKEN に設定
```

## 監査の戻し方

relay 経由だと上流ログで実ユーザが特定できないため、relay 側で hook を仕込む:

```toml
[server.relay.task_add.hooks.audit]
command = '''
jq -c "{
  ts: .event.timestamp,
  actor: .user.name,
  actor_id: .user.id,
  action: \"task_add\",
  task: .event.task.id,
  title: .event.task.title
}" >> /var/log/senko-relay-audit.jsonl
'''
mode = "async"
```

`.user` は **relay 側で認証された** identity (= sandbox 内のユーザ)。これを監査ログに残しておけば、上流 DB の task id と突き合わせて追跡可能。

## セキュリティ考慮

- 上流トークンが漏洩するとプロジェクト全体が危険。Secrets Manager + `[server.relay] token` を env or ARN で注入
- relay 自身の認証は API キー推奨 (OIDC は CLI 起動の UX が sandbox で扱いにくい)
- sandbox 内で短命トークンを発行 → relay を通す仕組みで、再ログインのハードルを下げつつ有効期限でガード

## よくある間違い

- **上流側の `[cli.remote]` で relay URL を指定する** — これは CLI (人間/エージェント) が relay に繋ぐときの設定。**relay 自体の上流設定は `[server.relay]`**。混同しやすい
- **relay と上流で `[server.auth.api_key] master_key` が同じ** — 分けること。relay 側は relay 管理用、上流側は上流管理用
- **token を透過しながら上流で trusted_headers を期待** — trusted_headers は Authorization ヘッダではなく別のヘッダ (`x-senko-*`) を見るので、透過 passthrough と matching しない。関連構成を使うなら API Gateway を前段に置く

## 次のステップ

- relay 全般の運用 → [deploy.md](deploy.md)
- hook で監査ログ → [hooks.md](hooks.md)
