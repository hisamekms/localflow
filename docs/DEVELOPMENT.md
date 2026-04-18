# Development

[日本語](DEVELOPMENT.ja.md)

## Status Transitions

```
draft → todo → in_progress → completed
                    ↓
                 canceled
```

- `draft` → `todo` → `in_progress` → `completed`: forward-only
- Any active state → `canceled`: always allowed
- Backward transitions and self-transitions are rejected

## Data Storage

The database is stored at `<project_root>/.senko/data.db` (auto-created).

Project root is detected by searching for `.senko/`, `.git/`, or using the current directory.

## Testing

```bash
cargo test                    # Unit tests
bash tests/e2e/run.sh         # E2E tests
```

## Dependency Updates

- **Cargo / GitHub Actions**: managed by Dependabot (`.github/dependabot.yml`).
- **mise tool versions** (`mise.toml`, `mise.host.toml`): managed by Renovate (`renovate.json5`).

Renovate waits at least 7 days after a release before opening a PR. Automerge is disabled on both sides — every bump gets a manual review.
