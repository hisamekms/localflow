# OIDC 認証

OAuth 2.0 Authorization Code + PKCE フローによるログイン。社内 SSO や Google / Cognito / Keycloak / Auth0 等の IdP 配下で使う想定。

> **本番の人間ユーザ認証としての推奨方式** です。CI / bot からは [API キー認証](auth-api-key.md) を併用してください (`master_key` の有無に関わらず OIDC と API キーは共存可)。

## どう動くか

```
CLI ── senko auth login ──┐
                          ├── ブラウザで IdP にリダイレクト
                          ├── ログイン → PKCE exchange
                          └── senko が JWT を受け取り、内部で API キーを発行して keychain に保存

その後の senko コマンドは keychain から token を取り出して Bearer で送る
```

JWT をそのまま送り続けるわけではなく、**初回 1 回だけ JWT 検証を行い、以降は senko 内部の API キーに変換** しているのがポイントです。

## サーバ側の設定

```toml
[server.auth.oidc]
issuer_url = "https://accounts.example.com"
client_id  = "senko-cli"
scopes     = ["openid", "profile", "email"]
# username_claim = "preferred_username"   # 指定しないと sub を使う
# required_claims = { email_verified = "true" }
callback_ports = ["8400", "9000-9010"]    # CLI ログイン時にブラウザが開くローカル callback ポート候補

[server.auth.oidc.session]
ttl          = "30d"    # 絶対 TTL
inactive_ttl = "7d"     # 無活動タイムアウト
max_per_user = 10       # 1 ユーザあたりセッション上限
```

- `issuer_url` から `.well-known/openid-configuration` が取得できる必要がある
- `client_id` は IdP 側で "Public client / PKCE" として登録する (secret 不要)
- `callback_ports` は **CLI 側のマシンで開くポート候補**。個別 or range 指定可

## IdP 側の設定

IdP に "Public OAuth Client" として登録:

- **grant types**: authorization_code (PKCE)
- **redirect URIs**: `http://127.0.0.1:<port>/callback` (callback_ports と一致させる)
- **scopes**: `openid profile email`

## クライアント側 (CLI)

```toml
# .senko/config.toml
[cli.remote]
url = "https://senko.example.com"
# token は keychain 経由なのでここには書かない
```

初回ログイン:

```bash
senko auth login [--device-name "alice-laptop"]
```

挙動:

1. ブラウザが立ち上がる (`[cli] browser = false` なら URL が stdout に出るだけ)
2. IdP で認証
3. CLI が callback を受けて PKCE で token 交換
4. サーバ側で JWT 検証 → 内部 API キーを作って返す
5. CLI が OS keychain にその API キーを保存

以降:

```bash
senko auth status     # 今のログイン情報
senko auth sessions   # 発行済みセッション (= 内部 API キー) 一覧
senko auth logout     # 現セッションを revoke + keychain 削除
senko auth revoke <id>        # 他デバイスを revoke
senko auth revoke --all       # 全セッション revoke
```

## keychain の中身

- macOS: Keychain Access → `senko` サービス
- Linux: libsecret / gnome-keyring の `senko` エントリ
- Windows: Credential Manager の `senko`

CI / headless 環境では keychain が使えないため、事前に発行した API キーを env で注入する運用に切り替えてください (→ [auth-api-key.md](auth-api-key.md))。

## セッション管理

サーバ側では OIDC ログイン由来の API キーを "session" として区別します:

- `[server.auth.oidc.session] ttl` 経過で失効 (再ログイン必要)
- `inactive_ttl` 経過 (最終使用から) で失効
- `max_per_user` に達すると古いセッションが落とされる

## 信頼ヘッダと併用できない

`[server.auth.oidc]` と `[server.auth.trusted_headers]` は同時有効化できません。API Gateway 配下で OIDC を処理する構成は `trusted_headers` を使ってください ([auth-trusted-headers.md](auth-trusted-headers.md))。

## トラブルシューティング

| 症状 | 確認点 |
|---|---|
| `senko auth login` でブラウザが開かない | ヘッドレスなら `[cli] browser = false` で URL コピー運用 |
| callback で connection refused | `callback_ports` の範囲がファイアウォールで潰れていないか |
| ログインは成功するが API で 401 | `username_claim` が IdP の claim と合っているか |
| 毎回再ログインを求められる | `[server.auth.oidc.session] ttl` / `inactive_ttl` が短すぎないか |
| SSO 側の groups/roles を senko の権限に反映したい | 現状マッピング機能なし。member を手動で追加するか、`required_claims` で絞る |

## 次のステップ

- API Gateway (Cognito) 配下で OIDC を終端させ、senko は信頼ヘッダで受ける構成 → [auth-trusted-headers.md](auth-trusted-headers.md) と [aws-deployment.md](aws-deployment.md)
