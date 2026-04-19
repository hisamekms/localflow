# 自分でサーバを立てる

チーム / 組織向けに `senko serve` を立てて、複数 CLI やエージェントから接続できるようにします。

このページは **最小動作する構成** まで。認証モードや AWS デプロイなどは該当する `guides/server-remote/` を参照してください。

## 構成パターン

senko サーバには 2 モードあります。このページでは **direct mode** を扱います。

| モード | 起動コマンド | 説明 |
|---|---|---|
| **direct** | `senko serve` | DB (SQLite / PostgreSQL) を直接読み書き |
| **relay** | `senko serve --proxy` | 上流の direct サーバへ HTTP 転送するだけ (AI サンドボックス等で使用) → [guides/server-relay/](../guides/server-relay/) |

## 前提

- `senko` バイナリ (`postgres` feature 有効なビルド推奨。ソースから `cargo build --release --features postgres`)
- DB: 手軽に試すなら SQLite、本運用なら PostgreSQL

## 1. 最小の direct 起動 (SQLite、認証なし)

**試用・ローカル検証のみ推奨**。本番では必ず認証を有効化してください。

```bash
# 任意のディレクトリで
mkdir -p /var/lib/senko
cd /var/lib/senko

senko serve --host 0.0.0.0 --port 3142
```

これだけで `/var/lib/senko/.senko/senko.db` に SQLite が作成され、API が 3142 で待受けます。

動作確認:

```bash
curl http://127.0.0.1:3142/api/v1/health
```

## 2. PostgreSQL を使う

```bash
export SENKO_POSTGRES_URL="postgres://senko:password@db.example.com/senko"
senko serve --host 0.0.0.0 --port 3142
```

初回起動時にマイグレーションが適用されます。接続プールの上限は `[backend.postgres] max_connections` で調整可能。

AWS RDS で credential を Secrets Manager に置いている場合:

```toml
[backend.postgres]
rds_secrets_arn = "arn:aws:secretsmanager:..."   # username/password/host を含む JSON secret
```

`aws-secrets` feature 有効ビルドが必要です。

## 3. 認証を有効化する

認証方式は 3 択 (**ユーザ認証としては同時に 1 つだけ**):

| 方式 | 位置づけ | 詳細 |
|---|---|---|
| **OIDC** | **本番の人間ユーザ認証の推奨手段** | [auth-oidc.md](../guides/server-remote/auth-oidc.md) |
| 信頼ヘッダ | API Gateway / Lambda 配下で JWT 検証を前段に逃がす構成 | [auth-trusted-headers.md](../guides/server-remote/auth-trusted-headers.md) |
| API キー | **試用 / CI / bot 用**。人間のログインには OIDC を推奨 | [auth-api-key.md](../guides/server-remote/auth-api-key.md) |

> `master_key` は OIDC / 信頼ヘッダと **併用可能**。ユーザ発行などの bootstrap 用途のみに使い、日常の API 呼び出しには OIDC セッション or 個人 API キーを使う運用が標準です。

試用・初期動作確認用の最小構成 (API キーのみ):

```bash
# master key を発行
MASTER_KEY=$(openssl rand -base64 32)
export SENKO_AUTH_API_KEY_MASTER_KEY="$MASTER_KEY"

senko serve --host 0.0.0.0 --port 3142
```

master key を使って初回ユーザと API キーを発行:

```bash
# ユーザ作成 (master key 必須)
curl -s -X POST http://127.0.0.1:3142/api/v1/users \
  -H "Authorization: Bearer $MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"username":"alice"}' | jq .

# alice に API キー発行
curl -s -X POST http://127.0.0.1:3142/api/v1/users/1/api-keys \
  -H "Authorization: Bearer $MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name":"default"}' | jq .
```

発行された API キーをクライアント側に渡します ([remote-cli.md](remote-cli.md) 参照)。

## 4. ログと hook

hook を使うと、サーバ側で状態遷移を捕まえて webhook / 通知 / 監査ログに流せます:

```toml
# /var/lib/senko/.senko/config.toml
[server.remote.task_complete.hooks.audit]
command = "logger -t senko 'task complete'"
mode = "async"
```

実例は [guides/server-remote/hooks.md](../guides/server-remote/hooks.md)。

## 5. systemd で常駐させる例

```ini
# /etc/systemd/system/senko.service
[Unit]
Description=senko server
After=network.target

[Service]
Type=simple
User=senko
WorkingDirectory=/var/lib/senko
Environment="SENKO_POSTGRES_URL=postgres://senko:..."
Environment="SENKO_AUTH_API_KEY_MASTER_KEY_FILE=/etc/senko/master_key"
ExecStart=/usr/local/bin/senko serve --host 0.0.0.0 --port 3142
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

> **注**: `_FILE` サフィックスによるファイル読込は現行未対応。現時点では `EnvironmentFile=` を使って直接値を注入してください。

```bash
sudo systemctl enable --now senko
sudo journalctl -u senko -f
```

## ヘルスチェック・メトリクス

```bash
curl http://127.0.0.1:3142/api/v1/health
```

- 認証不要 / 200 を返せば OK
- ロードバランサーのヘルスチェックにも使用可

メトリクスエンドポイントは現状なし (v1 時点)。hook + 外部ログ基盤で代替してください。

## 次に読むもの

- 認証モードを選ぶ → [explanation/runtimes.md](../explanation/runtimes.md) → 各 `auth-*.md`
- 本番向け AWS 構成 → [guides/server-remote/aws-deployment.md](../guides/server-remote/aws-deployment.md)
- API の詳細 → [reference/api.md](../reference/api.md)
