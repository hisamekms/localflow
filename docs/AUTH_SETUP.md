# Authentication Setup Guide

[Japanese](AUTH_SETUP.ja.md) | [Back to README](../README.md)

senko supports four authentication modes. Choose the one that best fits your use case.

| Mode | Use Case | Infrastructure | Auth Method |
|------|----------|---------------|-------------|
| Local | Personal development, single user | None | No auth |
| Remote + API key | CI/CD, service-to-service | senko server | API key |
| Remote + OIDC | Team use, enterprise SSO | senko server + OIDC provider | OAuth PKCE + API key |
| Relay/Proxy | AI sandbox, multi-tenant relay | senko relay + senko remote server | Token injection or passthrough |

## Local Mode

The simplest configuration. No setup required — the SQLite database is created automatically on first run.

### Minimal Setup

Start using senko immediately without any configuration:

```bash
senko add --title "First task"
senko list
```

On first run, `.senko/senko.db` (SQLite database) is created automatically. A default project and user (id=1, name="default") are provisioned automatically.

### Custom Configuration (Optional)

To customize the project or user name, create `.senko/config.toml`:

```toml
[project]
name = "my-project"

[user]
name = "alice"
```

You can also generate a template:

```bash
senko config --init
```

### Data Storage

| File | Description |
|------|-------------|
| `.senko/senko.db` | SQLite database |
| `.senko/config.toml` | Configuration file (optional) |

> **Note**: Add `.senko/` to `.gitignore` to avoid committing local data.

## Remote + API Key Mode

Run a senko server and connect clients using API keys. Suitable for CI/CD pipelines and service-to-service communication.

### Prerequisites

- A machine to run the senko server
- Network connectivity from clients to the server

### Administrator Setup

#### 1. Generate a Master API Key

The master API key is used to bootstrap the system (create initial users and issue API keys):

```bash
openssl rand -base64 32
```

#### 2. Server Configuration

Server-side `.senko/config.toml`:

```toml
[auth.api_key]
master_key = "your-generated-master-api-key"

[auth.oidc.session]
ttl = "30d"              # Token lifetime (omit for no expiration)
inactive_ttl = "7d"      # Inactivity timeout (omit for no expiration)
max_per_user = 10        # Max sessions per user (omit for unlimited)
```

Using environment variables:

```bash
export SENKO_AUTH_API_KEY_MASTER_KEY="your-generated-master-api-key"
```

#### 3. Start the Server

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### 4. Create Users

Use the master API key for initial setup (`POST /users` is restricted to master key holders only):

```bash
# Create a user
curl -s -X POST http://localhost:3142/api/v1/users \
  -H "Authorization: Bearer $SENKO_AUTH_API_KEY_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "display_name": "Alice Smith"}' | jq .
```

You can also use the CLI:

```bash
senko user create --username alice --display-name "Alice Smith"
```

#### 5. Create a Project

```bash
senko project create --name my-project
```

#### 6. Add Members

```bash
# Add user ID 2 as a member (roles: owner, member, viewer)
senko members add --user-id 2 --role member
```

#### 7. Issue User API Keys

```bash
# Issue an API key for user ID 2 (replace with actual user ID)
curl -s -X POST http://localhost:3142/api/v1/users/2/api-keys \
  -H "Authorization: Bearer $SENKO_AUTH_API_KEY_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name": "alice-default"}' | jq .
```

The `token` field in the response is the API key. **This key is shown only once.**

### Client Setup

#### 1. Configuration

`~/.config/senko/config.toml` or project-level `.senko/config.toml`:

```toml
[server]
url = "http://senko-server:3142"
token = "api-key-from-administrator"
```

Using environment variables:

```bash
export SENKO_SERVER_URL="http://senko-server:3142"
export SENKO_TOKEN="api-key-from-administrator"
```

#### 2. Verify Connection

```bash
senko --output text list
```

### CI/CD Example

```yaml
# GitHub Actions example
env:
  SENKO_SERVER_URL: ${{ secrets.SENKO_SERVER_URL }}
  SENKO_TOKEN: ${{ secrets.SENKO_TOKEN }}

steps:
  - name: List tasks
    run: senko list --status todo
```

## Remote + OIDC Mode

Integrate with an OIDC provider (Amazon Cognito, Auth0, Okta, etc.) for browser-based login. Suitable for team use and enterprise SSO environments.

### Prerequisites

- A machine to run the senko server
- An OIDC provider (Amazon Cognito, Auth0, Okta, etc.)
- Network connectivity from clients to both the server and the OIDC provider

### Administrator Setup

#### 1. Configure the OIDC Provider

Register an application with your OIDC provider. Example for Amazon Cognito:

- **Application type**: Public client (PKCE-enabled)
- **Allowed callback URLs**: `http://127.0.0.1:8400/callback` (for CLI login)
- **Scopes**: `openid`, `profile` (add `email` if needed)
- **Authorization flow**: Authorization code grant with PKCE

Note the following from your provider:
- **Issuer URL**: `https://cognito-idp.{region}.amazonaws.com/{user-pool-id}`
- **Client ID**: The application client ID

#### 2. Server Configuration

Server-side `.senko/config.toml`:

```toml
[auth.oidc]
issuer_url = "https://cognito-idp.ap-northeast-1.amazonaws.com/ap-northeast-1_XXXXXXXXX"
client_id = "1a2b3c4d5e6f7g8h9i0j"
scopes = ["openid", "profile"]    # Default: ["openid", "profile"]

# Require specific JWT claims (optional)
[auth.oidc.required_claims]
"custom:tenant" = "my-company"

[auth.oidc.session]
ttl = "24h"              # Token lifetime
inactive_ttl = "7d"      # Inactivity timeout
max_per_user = 10        # Max sessions per user
```

Using environment variables:

```bash
export SENKO_OIDC_ISSUER_URL="https://cognito-idp.ap-northeast-1.amazonaws.com/ap-northeast-1_XXXXXXXXX"
export SENKO_OIDC_CLIENT_ID="1a2b3c4d5e6f7g8h9i0j"
```

> **Note**: You can configure both OIDC and a master API key simultaneously. This enables dual authentication — OIDC for human users and API keys for services.

#### 3. Start the Server

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### 4. Create Projects

In OIDC mode, users are auto-provisioned from JWT claims (`sub`, `name`, `email`) on first login. The user who creates a project is automatically added as an Owner. Create projects and add members as an administrator:

```bash
senko project create --name my-project
senko members add --user-id 2 --role member
```

### Client Setup

#### 1. Configuration

`~/.config/senko/config.toml` or project-level `.senko/config.toml`:

```toml
[server]
url = "http://senko-server:3142"
```

The CLI automatically fetches OIDC configuration (issuer URL, client ID, scopes) from the server via the `GET /auth/config` endpoint, so you do not need to configure OIDC settings on the client side.

#### 2. Login

```bash
senko auth login
```

A browser window opens automatically to the OIDC provider's login page. After authentication, the CLI receives an API key that is stored in the OS keychain.

To specify a device name:

```bash
senko auth login --device-name "my-laptop"
```

#### 3. Check Login Status

```bash
senko auth status
```

#### 4. Start Using

```bash
senko --output text list
```

### Container Integration

In environments where the OS keychain is not available (e.g., containers), use `senko auth token` to retrieve the token and pass it as an environment variable:

```bash
# Retrieve token on the host machine
export SENKO_TOKEN=$(senko auth token)

# Pass to container
docker run --rm \
  -e SENKO_SERVER_URL="http://senko-server:3142" \
  -e SENKO_TOKEN="$SENKO_TOKEN" \
  senko list
```

### Session Management

```bash
# List active sessions
senko auth sessions

# Revoke a specific session
senko auth revoke <session-id>

# Revoke all sessions
senko auth revoke --all

# Logout (revoke current session and remove token from keychain)
senko auth logout
```

## Relay/Proxy Mode

Run a senko instance as a relay that forwards requests to a remote senko server. The relay handles no authentication locally — all auth validation is delegated to the upstream remote server. This is useful for AI sandbox environments where clients should not hold credentials, or for multi-tenant setups where a relay aggregates requests from multiple clients.

> **Note**: The remote server must be set up first using either [Remote + API Key](#remote--api-key-mode) or [Remote + OIDC](#remote--oidc-mode) mode.

### Architecture

```
CLI ──→ Relay Server (senko serve) ──→ Remote Server
         [server.url configured]        [auth enabled]
```

When `server.url` is configured and `senko serve` is run, the instance operates in relay mode:

- Local authentication is skipped (delegated to the upstream server)
- The relay captures the client's Bearer token from the `Authorization` header
- Requests are forwarded to the remote server with either:
  - The relay's own `server.token` (if configured) — takes priority
  - The client's original token (passthrough)

### Pattern A: Token Injection (AI Sandbox)

The relay injects its own token into forwarded requests. Clients do not need any credentials.

#### Relay Server Setup

`.senko/config.toml` on the relay:

```toml
[server]
url = "http://remote-senko:3142"
token = "relay-api-key-issued-by-remote-server"
```

Using environment variables:

```bash
export SENKO_SERVER_URL="http://remote-senko:3142"
export SENKO_TOKEN="relay-api-key-issued-by-remote-server"
```

Start the relay:

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### Client Setup

The client only needs the relay's URL — no token required:

```toml
[server]
url = "http://relay-server:3142"
```

Using environment variables:

```bash
export SENKO_SERVER_URL="http://relay-server:3142"
```

#### Verify

```bash
senko --output text list
```

### Pattern B: Token Passthrough

The relay forwards the client's original token to the remote server. Each client authenticates individually.

#### Relay Server Setup

`.senko/config.toml` on the relay (no `token` — only `url`):

```toml
[server]
url = "http://remote-senko:3142"
```

Using environment variables:

```bash
export SENKO_SERVER_URL="http://remote-senko:3142"
```

Start the relay:

```bash
senko serve --host 0.0.0.0 --port 3142
```

#### Client Setup

The client configures both the relay URL and its own token (API key or OIDC-issued token):

```toml
[server]
url = "http://relay-server:3142"
token = "client-own-api-key"
```

Using environment variables:

```bash
export SENKO_SERVER_URL="http://relay-server:3142"
export SENKO_TOKEN="client-own-api-key"
```

#### Remote Server

The remote server validates the client's token directly. No special configuration is needed beyond the existing [Remote + API Key](#remote--api-key-mode) or [Remote + OIDC](#remote--oidc-mode) setup.

### Summary

| | Pattern A (Token Injection) | Pattern B (Token Passthrough) |
|-|----------------------------|-------------------------------|
| **Use case** | AI sandbox, shared service account | Per-user auth via relay |
| **Client token** | Not required | Required (API key or OIDC token) |
| **Relay config** | `server.url` + `server.token` | `server.url` only |
| **Remote validates** | Relay's token | Client's original token |

## config.toml Reference

### Authentication Configuration Keys

| Section | Key | Type | Default | Description | Local | Remote+API Key | Remote+OIDC | Relay |
|---------|-----|------|---------|-------------|:-----:|:--------------:|:-----------:|:-----:|
| `[auth.api_key]` | `master_key` | string | - | Master API key | - | Required | Optional | - |
| `[auth.api_key]` | `master_key_arn` | string | - | AWS Secrets Manager ARN | - | Optional | Optional | - |
| `[auth.oidc]` | `issuer_url` | string | - | OIDC issuer URL | - | - | Required | - |
| `[auth.oidc]` | `client_id` | string | - | OIDC client ID | - | - | Required | - |
| `[auth.oidc]` | `scopes` | array | `["openid", "profile"]` | OIDC scopes | - | - | Optional | - |
| `[auth.oidc]` | `required_claims` | table | - | Required JWT claims (key-value pairs) | - | - | Optional | - |
| `[auth.oidc.cli]` | `callback_port` | integer | Auto-assign | Callback port | - | - | Optional | - |
| `[auth.oidc.cli]` | `browser` | bool | `true` | Auto-open browser | - | - | Optional | - |
| `[auth.oidc.session]` | `ttl` | string | No expiration | Session lifetime (e.g., `"24h"`, `"30d"`) | - | Optional | Optional | - |
| `[auth.oidc.session]` | `inactive_ttl` | string | No expiration | Inactivity timeout | - | Optional | Optional | - |
| `[auth.oidc.session]` | `max_per_user` | integer | Unlimited | Max sessions per user | - | Optional | Optional | - |

> **Note**: Authentication is implicitly enabled when any `[auth.*]` configuration is present. There is no explicit `auth.enabled` key.

### Connection Configuration Keys

| Section | Key | Type | Default | Description |
|---------|-----|------|---------|-------------|
| `[server]` | `url` | string | - | API server URL (enables HTTP backend) |
| `[server]` | `token` | string | - | API token (client-side) |
| `[serve]` | `host` | string | `"127.0.0.1"` | Server bind address |
| `[serve]` | `port` | integer | `3142` | Server listen port |
| `[project]` | `name` | string | `"default"` | Project name |
| `[user]` | `name` | string | `"default"` | User name |
| `[storage]` | `db_path` | string | `.senko/senko.db` | SQLite database path |

> **Note**: In relay mode, the `[server]` section on the relay server specifies the upstream remote server. `server.url` enables relay mode when `senko serve` is run; `server.token` (if set) is injected into forwarded requests instead of passing through the client's token.

### API Endpoints

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/auth/config` | GET | No | Returns OIDC configuration (issuer URL, client ID, scopes) |
| `/auth/token` | POST | JWT | Exchange OIDC JWT for an API token |
| `/auth/me` | GET | Yes | Current user info and session details |
| `/auth/sessions` | GET | Yes | List active sessions |
| `/auth/sessions` | DELETE | Yes | Revoke all sessions |
| `/auth/sessions/{id}` | DELETE | Yes | Revoke a specific session |
| `/users` | POST | Master key | Create a new user |

### Environment Variables

| Variable | Config Key | Description |
|----------|-----------|-------------|
| `SENKO_AUTH_API_KEY_MASTER_KEY` | `auth.api_key.master_key` | Master API key |
| `SENKO_AUTH_API_KEY_MASTER_KEY_ARN` | `auth.api_key.master_key_arn` | AWS ARN for master API key |
| `SENKO_OIDC_ISSUER_URL` | `auth.oidc.issuer_url` | OIDC issuer URL |
| `SENKO_OIDC_CLIENT_ID` | `auth.oidc.client_id` | OIDC client ID |
| `SENKO_AUTH_OIDC_SESSION_TTL` | `auth.oidc.session.ttl` | Session lifetime |
| `SENKO_AUTH_OIDC_SESSION_INACTIVE_TTL` | `auth.oidc.session.inactive_ttl` | Inactivity timeout |
| `SENKO_AUTH_OIDC_SESSION_MAX_PER_USER` | `auth.oidc.session.max_per_user` | Max sessions per user |
| `SENKO_SERVER_URL` | `server.url` | API server URL |
| `SENKO_TOKEN` | `server.token` | API token (client-side) |
| `SENKO_HOST` | `serve.host` | Server bind address |
| `SENKO_PORT` | `serve.port` | Server port |
| `SENKO_DB_PATH` | `storage.db_path` | SQLite database path |
| `SENKO_PROJECT` | - | Project name to operate on |
| `SENKO_USER` | - | User name to operate as |

### Configuration Priority

Configuration values are applied in the following order (highest priority first):

1. CLI flags (`--config`, `--project-root`, etc.)
2. Environment variables (`SENKO_*`)
3. Project configuration (`.senko/config.toml`)
4. User configuration (`~/.config/senko/config.toml`)
5. Built-in defaults

## Related Documentation

- [CLI Reference](CLI.md) — Full command details
- [README](../README.md) — Project overview and quickstart
