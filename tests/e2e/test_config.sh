#!/usr/bin/env bash
# e2e tests for config subcommand
set -euo pipefail
source "$(dirname "$0")/helpers.sh"

setup_test_env
trap cleanup_test_env EXIT

echo "--- Test: Config ---"

echo "[1] config shows defaults when no config file exists"
JSON_OUT="$(run_lf config)"
assert_json_field "$JSON_OUT" '.workflow.merge_via' "direct" "default merge_via"
assert_json_field "$JSON_OUT" '.workflow.auto_merge' "true" "default auto_merge"

echo "[2] config text output shows defaults"
TEXT_OUT="$(run_lf --output text config)"
assert_contains "$TEXT_OUT" "direct" "text shows merge_via"
assert_contains "$TEXT_OUT" "auto_merge: true" "text shows auto_merge"

echo "[3] config --init creates config file"
INIT_OUT="$(run_lf config --init)"
assert_json_field "$INIT_OUT" '.action' "created" "init action is created"
if [[ -f "$TEST_PROJECT_ROOT/.senko/config.toml" ]]; then
  echo "  PASS: config.toml file created"
  PASS_COUNT=$((PASS_COUNT + 1))
else
  echo "  FAIL: config.toml file not created"
  FAIL_COUNT=$((FAIL_COUNT + 1))
fi

echo "[3b] config --init template uses new hooks schema"
TEMPLATE_CONTENT="$(cat "$TEST_PROJECT_ROOT/.senko/config.toml")"
# New runtime.action.hooks schema should be present as example comments
assert_contains "$TEMPLATE_CONTENT" "cli.task_" "template mentions [cli.task_*] runtime"
assert_contains "$TEMPLATE_CONTENT" "workflow." "template mentions [workflow.*] runtime"
assert_contains "$TEMPLATE_CONTENT" "server.remote" "template mentions [server.remote.*] runtime"
# Legacy schema markers must not appear
assert_not_contains "$TEMPLATE_CONTENT" "on_task_added" "template has no on_task_added"
assert_not_contains "$TEMPLATE_CONTENT" "on_task_completed" "template has no on_task_completed"
assert_not_contains "$TEMPLATE_CONTENT" "on_no_eligible_task" "template has no on_no_eligible_task"
assert_not_contains "$TEMPLATE_CONTENT" "SENKO_HOOKS_ENABLED" "template has no SENKO_HOOKS_ENABLED"

echo "[4] config --init fails when file already exists"
INIT2_OUT="$(run_lf config --init 2>&1 || true)"
assert_contains "$INIT2_OUT" "already exists" "init fails with existing file"

echo "[5] config reads custom values from config.toml"
cat > "$TEST_PROJECT_ROOT/.senko/config.toml" <<'EOF'
[workflow]
merge_via = "pr"
auto_merge = false

[cli.task_add.hooks.my-hook]
command = "echo added"
EOF
CUSTOM_OUT="$(run_lf config)"
assert_json_field "$CUSTOM_OUT" '.workflow.merge_via' "pr" "custom merge_via"
assert_json_field "$CUSTOM_OUT" '.workflow.auto_merge' "false" "custom auto_merge"

echo "[6] config text output shows custom values"
TEXT_CUSTOM="$(run_lf --output text config)"
assert_contains "$TEXT_CUSTOM" "pr" "text shows custom merge_via"
assert_contains "$TEXT_CUSTOM" "auto_merge: false" "text shows custom auto_merge"

echo "[7] config --init text output"
rm "$TEST_PROJECT_ROOT/.senko/config.toml"
INIT_TEXT="$(run_lf --output text config --init)"
assert_contains "$INIT_TEXT" "Created" "text init shows Created"

echo "[8] backward compat: old TOML key completion_mode still works"
cat > "$TEST_PROJECT_ROOT/.senko/config.toml" <<'EOF'
[workflow]
completion_mode = "pr_then_complete"
EOF
COMPAT_OUT="$(run_lf config)"
assert_json_field "$COMPAT_OUT" '.workflow.merge_via' "pr" "old TOML key completion_mode parsed as merge_via"

echo "[9] backward compat: old TOML values merge_then_complete/pr_then_complete still work under new key"
cat > "$TEST_PROJECT_ROOT/.senko/config.toml" <<'EOF'
[workflow]
merge_via = "merge_then_complete"
EOF
COMPAT_OUT2="$(run_lf config)"
assert_json_field "$COMPAT_OUT2" '.workflow.merge_via' "direct" "old value merge_then_complete maps to direct"

echo "[10] workflow.task_start.metadata_fields parsed correctly"
cat > "$TEST_PROJECT_ROOT/.senko/config.toml" <<'EOF'
[[workflow.task_start.metadata_fields]]
key = "assigned_by"
source = "env"
env_var = "USER"
default = "unknown"

[[workflow.task_start.metadata_fields]]
key = "team"
source = "value"
value = "backend"

[[workflow.task_start.metadata_fields]]
key = "estimate"
source = "prompt"
prompt = "Estimated time?"
EOF
SKILL_OUT="$(run_lf config)"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields | length' "3" "metadata_fields count"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[0].key' "assigned_by" "field 0 key"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[0].source' "env" "field 0 source"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[0].env_var' "USER" "field 0 env_var"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[0].default' "unknown" "field 0 default"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[1].key' "team" "field 1 key"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[1].source' "value" "field 1 source"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[1].value' "backend" "field 1 value"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[2].key' "estimate" "field 2 key"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[2].source' "prompt" "field 2 source"
assert_json_field "$SKILL_OUT" '.workflow.task_start.metadata_fields[2].prompt' "Estimated time?" "field 2 prompt"

echo "[11] workflow.task_start defaults to empty metadata_fields"
cat > "$TEST_PROJECT_ROOT/.senko/config.toml" <<'EOF'
[project]
name = "test"
EOF
EMPTY_SKILL="$(run_lf config)"
# With no config, workflow.stages is an empty map; task_start section is not present.
# Accessing .workflow.task_start yields null, so length defaults via // 0.
assert_json_field "$EMPTY_SKILL" '(.workflow.task_start.metadata_fields // []) | length' "0" "default empty metadata_fields"

test_summary
