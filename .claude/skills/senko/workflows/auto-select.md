# Auto-Select

## Step 1: Build metadata

Run the metadata builder script to read `[workflow.start].metadata_fields` from config:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/build-metadata.sh start
```

Parse the JSON output (`{"resolved": {...}, "prompts": [...]}`):

- If `prompts` array is non-empty, ask the user each prompt question using `AskUserQuestion`. Merge user answers into `resolved`.
- If `resolved` is empty (no keys) after merging, do NOT pass `--metadata`.

## Step 2: Select next task

```bash
senko next --metadata '<final-metadata-json>'
```

Omit `--metadata` entirely if there are no metadata fields to pass.

- **Success**: The selected task moves to `in_progress`. Read task info from JSON output and proceed to "Execute Task" Step 2.
- **No eligible tasks**: Inform the user and stop.
