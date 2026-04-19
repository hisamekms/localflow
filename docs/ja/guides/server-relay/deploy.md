# `senko serve --proxy` (relay) をデプロイする

Relay サーバは DB を持たず、上流の direct サーバへ HTTP 転送するだけの薄いサーバ。
DB ロジックが無いので軽く、認証ポリシーやトークン書き換えを「入口」でやりたい時に使います。

使い所の判断: [explanation/runtimes.md](../../explanation/runtimes.md)

## 典型ユースケース

1. **AI サンドボックス内からの API アクセス制御**
   - サンドボックス内のエージェントは外部ネットワークに直接出られない
   - サンドボックスごとに relay を 1 つ置き、認可を入口で固めて上流へ流す
2. **マルチテナント relay**
   - テナントごとに認証ポリシーを変えたい (OIDC IdP が違う、API キーが違う)
   - 裏側は 1 つの direct サーバで統一
3. **トークン書き換え**
   - クライアントの credential を relay が持つサービストークンに差し替えて上流に送る
   - クライアントには上流の credential を露出させたくない

## 最小構成

```bash
# env に上流サーバと relay 認証を入れる
export SENKO_SERVER_RELAY_URL="https://senko-upstream.example.com"
export SENKO_SERVER_RELAY_TOKEN="service-token-for-upstream"

# relay を起動
senko serve --proxy --host 0.0.0.0 --port 3142
```

config 版:

```toml
[server]
host = "0.0.0.0"
port = 3142

[server.relay]
url   = "https://senko-upstream.example.com"
token = "service-token-for-upstream"

# relay 自身の認証 (クライアントからの入口側)
[server.auth.api_key]
master_key = "relay-local-admin-key"
```

## 挙動

relay は受け取った HTTP リクエストを以下のように処理します:

1. `[server.auth.*]` で認証 (relay 側の auth モード)
2. リクエストを上流へ転送
   - `[server.relay] token` が設定されていれば → Authorization をこの token に差し替え
   - 未設定なら → クライアントの Authorization をそのまま透過
3. 上流からのレスポンスをそのまま返す
4. `[server.relay.<action>.hooks.<name>]` が該当 action に設定されていれば発火

## 透過 passthrough モード (token 未設定)

```toml
[server.relay]
url = "https://senko-upstream.example.com"
# token を書かない
```

この場合、relay は認証ヘッダに触れず、**クライアントの OIDC JWT / API キーがそのまま上流に届く**。
上流で `[server.auth.oidc]` を有効化しておけば、「relay は経路上の認可フィルタ、認証は上流で」という構成が作れます。

## トークン差し替えモード (token 設定)

```toml
[server.relay]
url   = "https://senko-upstream.example.com"
token = "service-account-token"
```

- **クライアントの credential は上流に届かない** (relay が受け取って捨てる)
- 上流から見ると「relay がすべてのリクエストを service-account-token として代表している」ように見える
- 上流ログには実ユーザ名が残らないため、relay 側で監査ログを取る必要あり (`[server.relay.<action>.hooks.*]`)

## 複合例: sandbox からの relay

```toml
# relay (sandbox 内)
[server]
host = "127.0.0.1"
port = 3142

[server.relay]
url   = "https://senko-upstream.example.com"
token = "sandbox-service-token"

[server.auth.api_key]
master_key = "sandbox-local-admin"

# 監査: sandbox を通った全 task_complete を記録
[server.relay.task_complete.hooks.sandbox_audit]
command = "jq -c '.event.task' >> /var/log/sandbox-senko.jsonl"
mode = "async"
```

## AI サンドボックスでの CLI 設定

sandbox 内のエージェントは:

```bash
export SENKO_CLI_REMOTE_URL="http://127.0.0.1:3142"     # sandbox 内 relay
export SENKO_CLI_REMOTE_TOKEN="sandbox-local-admin"     # relay が発行する短命トークン
senko task list
```

sandbox 内で発行した master key or API キーで relay に認証 → relay が上流のサービストークンに差し替え → 上流で処理、という流れ。

## ヘルスチェック

relay 自体にも `GET /api/v1/health` がある (認証不要、上流を叩かず即 200)。Load Balancer から使えます。

## 運用 Tips

- **relay はステートレス** なので複数インスタンスで簡単にスケールアウト可
- **上流との TLS は証明書検証を省かない**
- **上流への token を Secrets Manager で管理** (aws-secrets feature は relay 側でも使える)
- **relay の hook は audit 専用**に使う。重い処理は上流側か外部システムへ

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| 502 Bad Gateway | 上流が落ちている / ネットワーク不通 |
| 401 が上流から返る | `token` の認可不足 (上流で master key でないと通らない操作を relay 経由でやろうとしている、など) |
| 透過モードなのに 401 | relay の認証で 1 回弾かれている可能性。`[server.auth.*]` の設定を確認 |
| 上流に届かない hook | `[server.relay.*]` に書くべき hook を誤って `[server.remote.*]` に書いていないか |

## 次のステップ

- トークン書き換えの詳細 → [token-relay.md](token-relay.md)
- hook の実例 → [hooks.md](hooks.md)
