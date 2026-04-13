use anyhow::{bail, Context, Result};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse as OAuth2TokenResponse, TokenUrl,
};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::infra::config::OidcConfig;

#[derive(Deserialize)]
struct OidcDiscovery {
    authorization_endpoint: String,
    token_endpoint: String,
}

pub enum LoginResult {
    /// OIDC mode: both access token and API key were saved
    Oidc {
        key_prefix: String,
        expires_at: Option<String>,
    },
    /// trusted_headers mode: only access token was saved (no API key exchange)
    TrustedHeaders,
}

#[derive(Deserialize)]
struct SenkoTokenResponse {
    token: String,
    key_prefix: String,
    expires_at: Option<String>,
}

pub async fn perform_login(
    oidc_config: &OidcConfig,
    api_url: &str,
    device_name: Option<&str>,
    auth_mode: &str,
) -> Result<LoginResult> {
    let issuer_url = oidc_config
        .issuer_url
        .as_deref()
        .context("auth.oidc.issuer_url is not configured")?;
    let client_id = oidc_config
        .client_id
        .as_deref()
        .context("auth.oidc.client_id is not configured")?;

    // Step 1: OIDC Discovery
    let discovery_url = format!(
        "{}/.well-known/openid-configuration",
        issuer_url.trim_end_matches('/')
    );
    let http_client = reqwest::Client::new();
    let discovery: OidcDiscovery = http_client
        .get(&discovery_url)
        .send()
        .await
        .context("failed to fetch OIDC discovery document")?
        .json()
        .await
        .context("failed to parse OIDC discovery document")?;

    // Step 2: PKCE
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Step 3: Start callback server
    let ports = parse_callback_ports(&oidc_config.callback_ports)?;
    let listener = bind_callback_listener(&ports).await?;
    let local_addr = listener.local_addr()?;
    let redirect_uri = format!("http://127.0.0.1:{}/callback", local_addr.port());

    // Build OAuth2 client
    let oauth_client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_auth_uri(AuthUrl::new(discovery.authorization_endpoint)?)
        .set_token_uri(TokenUrl::new(discovery.token_endpoint)?)
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

    // Step 4: Build authorization URL and open browser
    let mut auth_request = oauth_client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge);
    for scope in &oidc_config.scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }
    let (auth_url, csrf_state) = auth_request.url();

    if oidc_config.cli.browser {
        eprintln!("Opening browser for authentication...");
        if let Err(e) = open::that(auth_url.as_str()) {
            eprintln!("Failed to open browser: {e}");
            eprintln!("Please open this URL manually:");
            eprintln!("  {auth_url}");
        }
    } else {
        eprintln!("Open this URL in your browser to authenticate:");
        eprintln!("  {auth_url}");
    }

    // Step 5: Wait for callback
    let (code, state) = receive_callback(&listener).await?;
    if state != *csrf_state.secret() {
        bail!("CSRF state mismatch — possible attack or stale request");
    }

    // Step 6: Exchange code for JWT
    let token_response = oauth_client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .context("failed to exchange authorization code for token")?;

    let jwt = token_response.access_token().secret().to_string();

    // TODO: If the OIDC provider returns a refresh token (token_response.refresh_token()),
    // save it to keychain for automatic token renewal. This would allow the CLI to
    // transparently refresh expired access tokens without requiring re-authentication.

    // Step 7: Save access token to keychain (both modes need this)
    super::keychain::save_access_token(api_url, &jwt)?;

    if auth_mode == "trusted_headers" {
        // In trusted_headers mode, no server-side API key exchange is needed.
        // The access token is sufficient for authentication via the reverse proxy.
        return Ok(LoginResult::TrustedHeaders);
    }

    // Step 8: Exchange JWT for senko API key (oidc mode)
    let token_url = format!("{}/auth/token", api_url.trim_end_matches('/'));
    let mut req = http_client
        .post(&token_url)
        .bearer_auth(&jwt);
    if let Some(name) = device_name {
        req = req.json(&serde_json::json!({ "device_name": name }));
    }
    let resp = req
        .send()
        .await
        .context("failed to call POST /auth/token")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("POST /auth/token failed ({status}): {body}");
    }
    let senko_token: SenkoTokenResponse = resp
        .json()
        .await
        .context("failed to parse /auth/token response")?;

    // Step 9: Save API key to keychain
    super::keychain::save(api_url, &senko_token.token)?;

    Ok(LoginResult::Oidc {
        key_prefix: senko_token.key_prefix,
        expires_at: senko_token.expires_at,
    })
}

fn parse_callback_ports(specs: &[String]) -> Result<Vec<u16>> {
    let mut ports = Vec::new();
    for spec in specs {
        if let Some((start_str, end_str)) = spec.split_once('-') {
            let start: u16 = start_str
                .trim()
                .parse()
                .with_context(|| format!("invalid port range start: {start_str:?}"))?;
            let end: u16 = end_str
                .trim()
                .parse()
                .with_context(|| format!("invalid port range end: {end_str:?}"))?;
            if start > end {
                bail!("invalid port range: {start} > {end}");
            }
            ports.extend(start..=end);
        } else {
            let port: u16 = spec
                .trim()
                .parse()
                .with_context(|| format!("invalid port number: {spec:?}"))?;
            ports.push(port);
        }
    }
    Ok(ports)
}

async fn bind_callback_listener(ports: &[u16]) -> Result<TcpListener> {
    if ports.is_empty() {
        return TcpListener::bind("127.0.0.1:0")
            .await
            .context("failed to bind callback server on 127.0.0.1:0");
    }

    for &port in ports {
        let addr = format!("127.0.0.1:{port}");
        match TcpListener::bind(&addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) => {
                tracing::debug!(port, error = %e, "callback port bind failed, trying next");
            }
        }
    }

    tracing::info!("all configured callback ports failed, falling back to OS-assigned port");
    TcpListener::bind("127.0.0.1:0")
        .await
        .context("failed to bind callback server on 127.0.0.1:0 (fallback)")
}

async fn receive_callback(listener: &TcpListener) -> Result<(String, String)> {
    let (mut stream, _) = listener.accept().await.context("failed to accept callback connection")?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // Parse GET /callback?code=...&state=... HTTP/1.1
    let request_line = request.lines().next().unwrap_or("");
    let path = request_line.split_whitespace().nth(1).unwrap_or("");

    let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
    let params: std::collections::HashMap<&str, &str> = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .collect();

    // Check for error response
    if let Some(error) = params.get("error") {
        let desc = params.get("error_description").unwrap_or(&"");
        let html = format!(
            "<html><body><h1>Authentication Failed</h1><p>{}: {}</p><p>You can close this tab.</p></body></html>",
            error, desc
        );
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        stream.write_all(response.as_bytes()).await.ok();
        stream.shutdown().await.ok();
        bail!("OIDC authentication error: {error}: {desc}");
    }

    let code = params
        .get("code")
        .context("callback missing 'code' parameter")?
        .to_string();
    let state = params
        .get("state")
        .context("callback missing 'state' parameter")?
        .to_string();

    let html = "<html><body><h1>Authentication Successful</h1><p>You can close this tab.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    stream.write_all(response.as_bytes()).await.ok();
    stream.shutdown().await.ok();

    Ok((code, state))
}

