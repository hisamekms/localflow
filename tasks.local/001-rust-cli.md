---
id: "001"
title: rust-cli
status: draft
session_id:
branch:
depends_on: []
tags: []
started_at:
completed_at:
---

## 概要
Rustプロジェクトを初期化し、CLIの基盤を構築する。

## 詳細
- `cargo init` でプロジェクト作成
- Cargo.toml に依存クレートを追加: clap (derive), rusqlite (bundled), serde, serde_json, anyhow, chrono
- clap derive でサブコマンド構造を定義（add, list, get, next, edit, complete, cancel, deps, skill-install）
- 各サブコマンドは空のハンドラを持つスタブとして実装
- `--output` グローバルオプション（json / text）を定義
- `--project-root` グローバルオプションを定義

## 完了条件
- `cargo build` が成功する
- `localflow --help` で全サブコマンドが表示される
- ユニットテストが通る
