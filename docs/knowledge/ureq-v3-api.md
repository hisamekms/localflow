---
id: ureq-v3-api
title: ureq v3のAPI変更点と正しい使い方
description: ureq v2からv3への移行で変わったメソッド名、エラーハンドリング、型の違いをまとめる
tags:
  - ureq
  - http-client
  - rust
  - api-migration
created_at: 2026-03-26
updated_at: 2026-03-26
---

## 概要

ureq v3はv2からAPIが大幅に変更されている。v2の知識で書くとコンパイルエラーになるため、主要な違いを記録する。

## 詳細

### リクエスト送信メソッド

| リクエスト種別 | Builder型 | 送信メソッド |
|---|---|---|
| GET, DELETE, HEAD | `RequestBuilder<WithoutBody>` | `.call()` |
| POST, PUT (JSON付き) | `RequestBuilder<WithBody>` | `.send_json(&data)` (`json` feature必須) |
| POST, PUT (空ボディ) | `RequestBuilder<WithBody>` | `.send_empty()` |
| POST, PUT (文字列/バイト) | `RequestBuilder<WithBody>` | `.send(data)` |

v2の `.call()` (全リクエスト共通) や `.send_json()` (feature不要) とは異なる。

### Agent作成

```rust
// v3
let agent: Agent = Agent::config_builder()
    .timeout_global(Some(Duration::from_secs(30)))
    .http_status_as_error(false)
    .build()
    .into();

// v2 (参考)
let agent = ureq::AgentBuilder::new()
    .timeout(Duration::from_secs(30))
    .build();
```

### エラーハンドリング（最重要）

v3の `Error::StatusCode(u16)` にはレスポンスボディが**含まれない**。APIのエラーメッセージ（例: `{"error": "not found"}`）を取得する方法が2つある:

**方法1: `http_status_as_error(false)` を使う（推奨）**

```rust
let agent: Agent = Agent::config_builder()
    .http_status_as_error(false)  // 4xx/5xxもOk(Response)で返る
    .build()
    .into();

let resp = agent.get(url).call()?;
if resp.status().is_success() {
    Ok(resp.into_body().read_json()?)
} else {
    // レスポンスボディからエラーメッセージを取得可能
    let error_body: serde_json::Value = resp.into_body().read_json()?;
    bail!("{}", error_body["error"]);
}
```

**方法2: デフォルト設定でステータスコードのみ使う**

```rust
match agent.get(url).call() {
    Ok(resp) => { /* 2xx */ },
    Err(ureq::Error::StatusCode(404)) => { /* ボディは取れない */ },
    Err(e) => { /* その他のエラー */ },
}
```

### Response型

v3の `Response` は `ureq::http::Response<ureq::Body>` (= `http::Response<ureq::Body>`)。`ureq::Response` はprivateなので直接使えない。

```rust
use ureq::http::Response;
use ureq::Body;

fn handle(resp: Response<Body>) -> Result<Task> { ... }
```

### `json` feature

`send_json()` と `body.read_json()` を使うには `features = ["json"]` が必要:

```toml
ureq = { version = "3", features = ["json"] }
```

## 解決策 / 推奨事項

- エラーボディが必要な場合は `http_status_as_error(false)` を設定し、ステータスを自分でチェックする
- `ureq::Response` ではなく `ureq::http::Response<ureq::Body>` を型として使う
- GET/DELETE には `.call()`、POST/PUT には `.send_json()` / `.send_empty()` を使う
