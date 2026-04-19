# `[server.relay.*]` hook の実例

relay サーバ (`senko serve --proxy`) の経路で発火する hook の実践パターン。

スキーマ: [reference/hooks.md](../../reference/hooks.md)

## relay hook の位置づけ

relay は上流へリクエストを転送するだけで DB を持たないので、**"relay で発火させたい hook"** の多くは **監査・観測** 目的になります:

- どの actor が何時どの action を通したか (= 実ユーザ identity は relay 側でしか追跡できない)
- relay → 上流でエラー率が上がっていないか
- 特定条件のリクエストを別系統にも流したい (ログ集約、DLQ など)

重い処理 (外部連携、通知) は **上流側** の `[server.remote.*]` に書く方が構造的にきれい。relay ではなるべく "見るだけ" に留める。

## 監査ログ

実ユーザの identity が残らない (= 上流のログには relay の service account だけ残る) 構成の場合、relay 側で監査ログを取るのが必須:

```toml
[server.relay.task_add.hooks.audit]
command = '''
jq -c "{
  ts: .event.timestamp,
  actor: .user.name,
  actor_id: .user.id,
  project: .project.name,
  action: .event.event,
  task: .event.task.id,
  title: .event.task.title
}" >> /var/log/senko-relay-audit.jsonl
'''
mode = "async"
```

同じ hook を各 action (`task_ready`, `task_start`, `task_complete`, `task_cancel`, `contract_*`) に展開すれば全経路の監査が取れます。

## Fluent Bit / Vector に渡す

ローカルファイルではなく直接 log shipper の socket に流す:

```toml
[server.relay.task_complete.hooks.fluent]
command = 'jq -c "." | nc -u -w 1 127.0.0.1 5140'
mode = "async"
```

## 上流エラーのカウント

relay は上流が返したエラーをそのままクライアントに伝えますが、エラー率の観測は手元で:

```toml
[server.relay.task_add.hooks.error_count]
command = '''
# envelope が届いている = 上流が成功した時のみ hook は呼ばれる。
# つまりこの count を増やせば「成功率」が取れる
curl -s -X POST "$METRICS_URL" --data "senko_relay_success 1"
'''
mode = "async"

[[server.relay.task_add.hooks.error_count.env_vars]]
name = "METRICS_URL"
required = true
```

> **重要**: relay hook は **上流への転送が成功した後** に発火します。上流が 5xx を返した場合は hook は発火しません。失敗率を取るなら nginx / reverse proxy 側の HTTP ログを集計する方が正確。

## クライアント identity をすべて残す

`token` 書き換えモードで使っている場合、上流に届くのは service account 名だけ。relay 側で実 actor を残す:

```toml
[server.relay.task_complete.hooks.who_did_it]
command = '''
echo "$(date -u +%FT%TZ) task_complete project=$(jq -r '.project.name') task=$(jq -r '.event.task.id') actor=$(jq -r '.user.name')" \
  >> /var/log/senko-relay-actors.log
'''
mode = "async"
```

これを S3 に push する、Splunk に流す、等の運用で "上流ログ × relay ログ" の突合せが可能になります。

## Hook を書く時の注意

- **stdin に来る envelope の `runtime` は `"server.relay"`**。cli / server.remote と混在する hook スクリプトを書くならここで分岐
- relay の `project` / `user` は **relay 側で認証されたもの**。上流 DB の project/user id と一致するとは限らない (ID はサーバ間で別)
- hook は fire-and-forget でも **サーバプロセスの死活** には影響しない。ただし非常に大量の log を書く command を sync で走らせるとレイテンシが悪化する → `async` を原則に

## 上流側 hook と使い分ける

| やりたいこと | どこに置く |
|---|---|
| relay を通った全リクエストを audit | `[server.relay.*]` |
| 上流 DB での state 変化に応じた通知 (全経路) | `[server.remote.*]` |
| CLI 実行者個人への通知 | `[cli.*]` |
| エージェントのプロンプト拡張 | `[workflow.*]` |

relay と上流の両方に hook を書くと **2 重に発火** するので、どちらか一方に統一するのが基本です。
