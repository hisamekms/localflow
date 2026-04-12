#!/usr/bin/env bash
set -euo pipefail

# Build metadata JSON from config.toml's [workflow.start].metadata_fields.
# Resolves env/value/command sources, reports prompt sources for the caller to handle.
#
# Output (JSON):
#   { "resolved": { "key": "value", ... }, "prompts": [ { "key": "...", "prompt": "..." } ] }

SENKO_BIN="${SENKO_BIN:-senko}"
CONFIG_JSON=$("$SENKO_BIN" config)
FIELDS=$(echo "$CONFIG_JSON" | jq -c '.workflow.start.metadata_fields // []')
FIELD_COUNT=$(echo "$FIELDS" | jq 'length')

if [ "$FIELD_COUNT" -eq 0 ]; then
  echo '{"resolved":{},"prompts":[]}'
  exit 0
fi

RESOLVED="{}"
PROMPTS="[]"

for i in $(seq 0 $((FIELD_COUNT - 1))); do
  FIELD=$(echo "$FIELDS" | jq -c ".[$i]")
  KEY=$(echo "$FIELD" | jq -r '.key')
  SOURCE=$(echo "$FIELD" | jq -r '.source')

  case "$SOURCE" in
    env)
      ENV_VAR=$(echo "$FIELD" | jq -r '.env_var')
      DEFAULT=$(echo "$FIELD" | jq -r '.default // empty')
      VALUE="${!ENV_VAR:-}"
      if [ -z "$VALUE" ]; then
        if [ -n "$DEFAULT" ]; then
          VALUE="$DEFAULT"
        else
          continue
        fi
      fi
      RESOLVED=$(echo "$RESOLVED" | jq --arg k "$KEY" --arg v "$VALUE" '. + {($k): $v}')
      ;;
    value)
      FIELD_VALUE=$(echo "$FIELD" | jq '.value')
      RESOLVED=$(echo "$RESOLVED" | jq --arg k "$KEY" --argjson v "$FIELD_VALUE" '. + {($k): $v}')
      ;;
    command)
      CMD=$(echo "$FIELD" | jq -r '.command')
      DEFAULT=$(echo "$FIELD" | jq -r '.default // empty')
      VALUE=$(eval "$CMD" 2>/dev/null || true)
      if [ -z "$VALUE" ]; then
        if [ -n "$DEFAULT" ]; then
          VALUE="$DEFAULT"
        else
          continue
        fi
      fi
      RESOLVED=$(echo "$RESOLVED" | jq --arg k "$KEY" --arg v "$VALUE" '. + {($k): $v}')
      ;;
    prompt)
      PROMPT_TEXT=$(echo "$FIELD" | jq -r '.prompt')
      PROMPTS=$(echo "$PROMPTS" | jq --arg k "$KEY" --arg p "$PROMPT_TEXT" '. + [{"key": $k, "prompt": $p}]')
      ;;
  esac
done

jq -n --argjson r "$RESOLVED" --argjson p "$PROMPTS" '{"resolved": $r, "prompts": $p}'
