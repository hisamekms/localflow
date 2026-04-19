# `[server.relay.*]` 設定

`senko serve --proxy` (relay mode) として動く時に有効な section。

relay サーバは DB を持たず、受け取った API リクエストを上流の direct サーバへ HTTP 転送します。詳細: [explanation/runtimes.md](../../explanation/runtimes.md)

## `[server]`

`[server]` は direct / relay で共通。host/port の設定。[server-remote.md](server-remote.md) 参照。

## `[server.relay]`

| キー | 型 | 既定 | 説明 |
|---|---|---|---|
| `url` | string | `null` | **必須**。上流 direct サーバ URL。設定すると relay モードが有効化 |
| `token` | string | `null` | 上流への認証 token (relay が一括で持つサービストークン) |

env override: `SENKO_SERVER_RELAY_URL` / `SENKO_SERVER_RELAY_TOKEN`

### token の扱い

3 パターンあります:

| パターン | 挙動 |
|---|---|
| `token` 設定あり | relay は全上流リクエストにこの token を使う (クライアントの token は捨てる) |
| `token` 設定なし | クライアントから来た `Authorization` ヘッダをそのまま上流へ透過 |
| クライアントが OIDC JWT を送る | 上流の `[server.auth.oidc]` が検証する構成と組み合わせると、relay は透過 passthrough に徹する |

## `[server.auth.*]`

relay 自身も認証層を持てます。設定できる auth モードは direct と同じ (`api_key` / `oidc` / `trusted_headers`)。

一般的な組合せ:

| 想定 | relay 側 auth | relay → 上流 |
|---|---|---|
| AI サンドボックス内からのアクセス制御 | trusted_headers or API キー | サービストークンに差し替え |
| OIDC SSO 配下のマルチテナント relay | OIDC | 透過 passthrough |
| 内部ネットワーク + API キー | API キー | サービストークンに差し替え |

## `[server.relay.<action>.hooks.<name>]`

relay 経路で状態遷移 API が通った時に発火する hook。

```toml
[server.relay.task_add.hooks.request_log]
command = "jq -c '.event.task | {id, title}' >> /var/log/senko-relay.jsonl"
mode = "async"

[server.relay.task_complete.hooks.audit]
command = "logger -t senko-relay 'task complete'"
mode = "async"
```

発火は **上流へのリクエストが成功した後**。上流で失敗した場合は発火しない (または失敗 log に記録される)。

## Relay を使うべきでないケース

- ただ HTTP プロキシが欲しいだけ → nginx / Caddy の reverse proxy で十分
- クライアント → 上流の素直な接続ができる → direct サーバに直接繋ぐ方が低レイテンシ
- クライアントごとに認証が違う → relay ではなく上流で直接 OIDC を受ける方がシンプル

relay が活きるのは、**送信元ネットワークから上流へ直接到達できない** or **認証の差し替え (サービストークン化) が必要** なケース。

## 最小構成例

```toml
[server]
host = "0.0.0.0"
port = 3142

[server.relay]
url = "https://senko.example.com"
# token を設定すれば全リクエストをこのトークンで上流に再発行。
# 未設定なら Authorization ヘッダを透過する。

[server.auth.api_key]
master_key = "relay-local-admin-key"
```
