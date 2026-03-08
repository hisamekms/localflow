## Worktreeワークフロー

git管理下のファイルを変更する作業は、worktreeで実施すること。

- mainブランチで直接ファイルを編集しない
- 作業開始時に `/wth` スキルでworktreeを作成してから変更を行う（`wth add`）
- 変更が完了しマージされたら、worktreeを削除する（`wth rm`）

## mainへのマージ

`main` は `/workspace` で利用されているのでそちらを使うこと。
