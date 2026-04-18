# AWS Deployment Guide: API Gateway + Cognito + Lambda Web Adapter

[Back to README](../README.md) | [Authentication Setup](AUTH_SETUP.md)

This guide describes how to deploy senko on AWS using API Gateway HTTP API, Amazon Cognito JWT Authorizer, and Lambda Web Adapter with the `trusted_headers` authentication mode.

## Architecture

```
Client (CLI / Relay)
  │
  │  Authorization: Bearer <Cognito JWT>
  ▼
API Gateway HTTP API
  │
  ├─ Cognito JWT Authorizer (validates JWT)
  │
  ├─ Parameter Mapping (JWT claims → x-senko-* headers)
  │
  ▼
Lambda (Web Adapter)
  │
  │  x-senko-user-sub: <sub>
  │  x-senko-user-name: <name>
  │  x-senko-user-email: <email>
  │  x-senko-user-groups: <groups>
  │  x-senko-user-scope: <scope>
  ▼
senko serve (trusted_headers mode)
```

- **API Gateway HTTP API** handles TLS termination and request routing.
- **Cognito JWT Authorizer** validates the JWT in the `Authorization` header and rejects unauthenticated requests.
- **Parameter mapping** extracts claims from the validated JWT and forwards them as `x-senko-*` request headers.
- **Lambda Web Adapter** converts Lambda invoke events into HTTP requests, so senko runs as a standard HTTP server with no AWS Lambda event dependency.
- **senko** reads user identity from the trusted headers — no token validation is performed by senko itself.

> **Security Warning**
>
> In `trusted_headers` mode, senko unconditionally trusts the values in the configured headers. **Never expose senko directly to the internet without API Gateway in front.** API Gateway must be the sole entry point so that only validated, authorizer-injected headers reach senko. If senko is accessible without going through API Gateway, any client can forge identity headers and impersonate any user.

## Prerequisites

- An AWS account
- Amazon Cognito User Pool (acts as the OIDC provider)
- API Gateway HTTP API
- [Lambda Web Adapter](https://github.com/awslabs/aws-lambda-web-adapter) layer
- A Lambda deployment package containing the senko binary

## Step 1: Cognito User Pool

If you already have a Cognito User Pool, skip to the app client setup.

### Create a User Pool

Create a Cognito User Pool with the attributes your team needs (e.g., `email`, `name`). Note the following:

- **User Pool ID**: `ap-northeast-1_XXXXXXXXX`
- **Issuer URL**: `https://cognito-idp.{region}.amazonaws.com/{user-pool-id}`

### Create an App Client

Register an app client for senko CLI login:

| Setting | Value |
|---------|-------|
| **Client type** | Public client (no client secret) |
| **Auth flow** | Authorization code grant with PKCE |
| **Callback URL** | `http://127.0.0.1:8400/callback` |
| **Scopes** | `openid`, `profile` (add `email` if needed) |

Note the **Client ID** for later use.

> For detailed OIDC setup, see the [Remote + OIDC section](AUTH_SETUP.md#remote--oidc-mode) of the Authentication Setup Guide.

## Step 2: API Gateway HTTP API

### Create the API

Create an HTTP API in API Gateway. This will serve as the entry point for all senko API requests.

### Configure the JWT Authorizer

Create a JWT Authorizer on the API:

| Setting | Value |
|---------|-------|
| **Authorizer type** | JWT |
| **Identity source** | `$request.header.Authorization` |
| **Issuer URL** | `https://cognito-idp.{region}.amazonaws.com/{user-pool-id}` |
| **Audience** | Your Cognito App Client ID |

Attach this authorizer to all routes (e.g., `ANY /{proxy+}`).

### Configure Parameter Mapping

The `Authorization` header is reserved for the JWT Authorizer and must not be forwarded to senko. Instead, use API Gateway's **request parameter mapping** to extract JWT claims into new `x-senko-*` headers.

Add the following **request parameter overrides** to your integration:

| Parameter | Mapping |
|-----------|---------|
| `header.x-senko-user-sub` | `$context.authorizer.claims.sub` |
| `header.x-senko-user-name` | `$context.authorizer.claims.name` |
| `header.x-senko-user-email` | `$context.authorizer.claims.email` |
| `header.x-senko-user-groups` | `$context.authorizer.claims.cognito:groups` |
| `header.x-senko-user-scope` | `$context.authorizer.claims.scope` |

Example using AWS CLI:

```bash
aws apigatewayv2 update-integration \
  --api-id <api-id> \
  --integration-id <integration-id> \
  --request-parameters '{
    "overwrite:header.x-senko-user-sub": "$context.authorizer.claims.sub",
    "overwrite:header.x-senko-user-name": "$context.authorizer.claims.name",
    "overwrite:header.x-senko-user-email": "$context.authorizer.claims.email",
    "overwrite:header.x-senko-user-groups": "$context.authorizer.claims.cognito:groups",
    "overwrite:header.x-senko-user-scope": "$context.authorizer.claims.scope"
  }'
```

> **Why `x-senko-*` headers instead of `Authorization`?**
> The `Authorization` header carries the JWT and is consumed by the JWT Authorizer at the API Gateway layer. API Gateway does not forward the decoded claims automatically, so parameter mapping is used to inject them as separate headers that senko can read in `trusted_headers` mode.

## Step 3: Lambda Configuration

### Lambda Web Adapter

[Lambda Web Adapter](https://github.com/awslabs/aws-lambda-web-adapter) runs your HTTP application inside Lambda without any code changes. senko has no AWS Lambda event dependency — it runs as a standard HTTP server, and Lambda Web Adapter handles the event-to-HTTP translation.

Add the Lambda Web Adapter layer to your function:

```
arn:aws:lambda:{region}:753240598075:layer:LambdaAdapterLayerArm64:24
```

### Bootstrap Script

Create a `bootstrap` script (or use the `run.sh` pattern) that starts senko:

```bash
#!/bin/bash
exec senko serve --host 0.0.0.0 --port 8080
```

Lambda Web Adapter forwards requests to port 8080 by default. To use a different port, set the `PORT` environment variable and match it in the `--port` flag.

### Environment Variables

Set the following environment variables on the Lambda function:

```bash
# Trusted headers authentication
SENKO_AUTH_TRUSTED_HEADERS_SUBJECT_HEADER=x-senko-user-sub
SENKO_AUTH_TRUSTED_HEADERS_NAME_HEADER=x-senko-user-name
SENKO_AUTH_TRUSTED_HEADERS_EMAIL_HEADER=x-senko-user-email
SENKO_AUTH_TRUSTED_HEADERS_GROUPS_HEADER=x-senko-user-groups
SENKO_AUTH_TRUSTED_HEADERS_SCOPE_HEADER=x-senko-user-scope

# OIDC settings (returned by GET /auth/config for CLI login)
SENKO_AUTH_TRUSTED_HEADERS_OIDC_ISSUER_URL=https://cognito-idp.{region}.amazonaws.com/{user-pool-id}
SENKO_AUTH_TRUSTED_HEADERS_OIDC_CLIENT_ID=your-cognito-client-id

# Database (use EFS mount or /tmp for ephemeral)
SENKO_DB_PATH=/mnt/efs/senko/senko.db
```

Alternatively, use a config file at `.senko/config.toml`:

```toml
[server.auth.trusted_headers]
subject_header = "x-senko-user-sub"
name_header = "x-senko-user-name"
email_header = "x-senko-user-email"
groups_header = "x-senko-user-groups"
scope_header = "x-senko-user-scope"
oidc_issuer_url = "https://cognito-idp.{region}.amazonaws.com/{user-pool-id}"
oidc_client_id = "your-cognito-client-id"
```

> **Note**: The `oidc_issuer_url` and `oidc_client_id` fields are used by the `GET /auth/config` endpoint, which the CLI calls during `senko auth login` to discover the OIDC provider. They do not affect server-side authentication (which relies solely on the trusted headers).

## Step 4: Client Setup

### Direct CLI Access

Configure the CLI to point at the API Gateway endpoint:

```toml
[cli.remote]
url = "https://<api-id>.execute-api.{region}.amazonaws.com"
```

Or using environment variables:

```bash
export SENKO_CLI_REMOTE_URL="https://<api-id>.execute-api.{region}.amazonaws.com"
```

Log in via Cognito:

```bash
senko auth login
```

The CLI fetches OIDC configuration from the server's `GET /auth/config` endpoint, opens a browser for Cognito authentication, and stores the resulting token in the OS keychain.

### Relay Setup (AI Sandbox / Container)

For environments where the CLI cannot open a browser (e.g., AI sandboxes, CI/CD, containers), use a relay with a pre-obtained token:

```bash
# On a machine with browser access, log in and retrieve the token
senko auth login
export SENKO_CLI_REMOTE_TOKEN=$(senko auth token)

# Start the relay
SENKO_SERVER_RELAY_URL="https://<api-id>.execute-api.{region}.amazonaws.com" \
SENKO_SERVER_RELAY_TOKEN="$SENKO_CLI_REMOTE_TOKEN" \
senko serve --host 127.0.0.1 --port 3142
```

Clients connecting to the relay do not need any credentials — the relay injects its token into forwarded requests:

```bash
# On the client (e.g., inside a container)
export SENKO_CLI_REMOTE_URL="http://localhost:3142"
senko --output text task list
```

> For more relay patterns (token injection vs. passthrough), see the [Relay/Proxy section](AUTH_SETUP.md#relayproxy-mode) of the Authentication Setup Guide.

## Security Considerations

1. **Never expose senko directly in `trusted_headers` mode.** The server trusts header values unconditionally. Without API Gateway as a gatekeeper, any client can set arbitrary `x-senko-*` headers and impersonate any user.

2. **Restrict Lambda network access.** Ensure the Lambda function is not accessible from the public internet except through API Gateway. Use VPC configuration or resource policies as appropriate.

3. **Rotate Cognito tokens.** Configure appropriate session lifetimes in Cognito and senko's session settings to limit the window of token misuse.

4. **Use `overwrite:` prefix in parameter mapping.** The `overwrite:` prefix ensures that any client-supplied `x-senko-*` headers are replaced by the authorizer-derived values, preventing header injection.

## Related Documentation

- [Authentication Setup Guide](AUTH_SETUP.md) — All authentication modes
- [CLI Reference](CLI.md) — Full command details
- [README](../README.md) — Project overview and quickstart
