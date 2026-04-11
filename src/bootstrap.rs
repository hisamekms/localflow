use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::application::port::auth::AuthProvider;
use crate::application::port::{HookExecutor, PrVerifier};
use crate::application::{HookTestService, LocalTaskOperations, ProjectService, TaskOperations, UserService};
use crate::domain::task::CompletionPolicy;
use crate::application::port::TaskBackend;
use crate::infra::config::{Config, LogConfig, LogFormat, RawConfig};
use crate::infra::http::HttpBackend;
use crate::infra::http::remote_task_ops::RemoteTaskOperations;
use crate::infra::hook::executor::ShellHookExecutor;
use crate::infra::hook::test_executor::ShellHookTestExecutor;
use crate::infra::hook::{RuntimeMode, BackendInfo};
use crate::infra::auth::{ApiKeyProvider, ChainAuthProvider, JwtAuthProvider};
use crate::infra::pr_verifier::GhCliPrVerifier;

// Re-exports for presentation layer (avoid direct infra dependency)
pub use crate::infra::hook;
pub use crate::infra::project_root::resolve_project_root;

pub use crate::domain::{DEFAULT_PROJECT_ID, DEFAULT_USER_ID};

/// Create the appropriate backend based on config (env + CLI already applied).
pub fn create_backend(
    project_root: &Path,
    config: &Config,
) -> Result<Arc<dyn TaskBackend>> {
    // 1. HTTP backend (api_url from env or config.toml)
    if let Some(ref url) = config.backend.api_url {
        let backend = match config.backend.api_key.as_ref() {
            Some(key) => HttpBackend::with_api_key(url, key.clone()),
            None => HttpBackend::new(url),
        };
        return Ok(Arc::new(backend));
    }

    // 2. DynamoDB backend
    #[cfg(feature = "dynamodb")]
    {
        use crate::infra::dynamodb::DynamoDbBackend;

        if let Some(ref ddb_config) = config.backend.dynamodb {
            if let Some(ref table_name) = ddb_config.table_name {
                return Ok(Arc::new(DynamoDbBackend::new(
                    table_name.clone(),
                    ddb_config.region.clone(),
                )));
            }
        }
    }

    // 3. PostgreSQL backend
    #[cfg(feature = "postgres")]
    {
        use crate::infra::postgres::PostgresBackend;

        if let Some(ref pg_config) = config.backend.postgres {
            if let Some(ref database_url) = pg_config.url {
                return Ok(Arc::new(PostgresBackend::new(database_url.clone(), pg_config.max_connections)));
            }
        }
    }

    // 4. Default: SqliteBackend
    let sqlite = crate::infra::sqlite::SqliteBackend::new(
        project_root,
        None,
        config.storage.db_path.as_deref(),
    )?;
    sqlite.sync_config_defaults(config)?;
    Ok(Arc::new(sqlite))
}

pub fn should_fire_client_hooks(config: &Config) -> bool {
    config.hooks.enabled
}

/// Resolve the backend info from config for hook envelope metadata.
/// Mirrors the priority logic of `create_backend`.
pub fn resolve_backend_info(config: &Config, project_root: &Path) -> BackendInfo {
    if let Some(ref url) = config.backend.api_url {
        return BackendInfo::Http { api_url: url.clone() };
    }
    #[cfg(feature = "dynamodb")]
    if config.backend.dynamodb.as_ref().and_then(|d| d.table_name.as_ref()).is_some() {
        return BackendInfo::Dynamodb;
    }
    #[cfg(feature = "postgres")]
    if config.backend.postgres.as_ref().and_then(|p| p.url.as_ref()).is_some() {
        return BackendInfo::Postgresql;
    }
    let db_path = crate::infra::sqlite::resolve_db_path_preview(project_root, config.storage.db_path.as_deref())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<unknown>".to_string());
    BackendInfo::Sqlite { db_file_path: db_path }
}

pub fn create_hook_executor(
    config: Config,
    runtime_mode: RuntimeMode,
    backend_info: BackendInfo,
    backend: Arc<dyn TaskBackend>,
) -> Arc<dyn HookExecutor> {
    let should_fire = should_fire_client_hooks(&config);
    Arc::new(ShellHookExecutor::new(config, should_fire, runtime_mode, backend_info, backend))
}

pub fn create_api_hook_executor(
    config: Config,
    backend_info: BackendInfo,
    backend: Arc<dyn TaskBackend>,
) -> Arc<dyn HookExecutor> {
    // API server always fires hooks
    Arc::new(ShellHookExecutor::new(config, true, RuntimeMode::Api, backend_info, backend))
}

pub fn create_pr_verifier() -> Arc<dyn crate::application::port::PrVerifier> {
    Arc::new(GhCliPrVerifier)
}

/// Validate auth configuration after Config is fully resolved (env + secrets).
/// Call before `create_auth_provider`.
pub fn validate_auth_config(config: &Config) -> Result<()> {
    config.auth.validate()
}

pub fn create_auth_provider(
    config: &Config,
    backend: Arc<dyn TaskBackend>,
) -> Result<Option<Arc<dyn AuthProvider>>> {
    if !config.auth.enabled {
        tracing::info!("authentication disabled");
        return Ok(None);
    }

    let mut providers: Vec<Arc<dyn AuthProvider>> = Vec::new();

    if config.auth.oidc.is_configured() {
        let issuer_url = config.auth.oidc.issuer_url.clone().unwrap();
        let client_id = config.auth.oidc.client_id.clone().unwrap();
        tracing::info!(issuer = %issuer_url, "OIDC JWT authentication enabled");
        providers.push(Arc::new(JwtAuthProvider::new(
            issuer_url,
            client_id,
            backend.clone(),
        )));
    }

    if backend.supports_api_key_auth() {
        tracing::info!("API key authentication enabled");
        providers.push(Arc::new(ApiKeyProvider::new(
            backend,
            config.auth.master_api_key.clone(),
        )));
    }

    if providers.is_empty() {
        bail!(
            "authentication is enabled but no auth providers could be created. \
             This is unexpected after config validation; check backend compatibility."
        );
    }

    if providers.len() == 1 {
        Ok(Some(providers.pop().unwrap()))
    } else {
        tracing::info!("authentication chain: JWT -> API key");
        Ok(Some(Arc::new(ChainAuthProvider::new(providers))))
    }
}

pub fn create_local_task_operations(
    backend: Arc<dyn TaskBackend>,
    config: &Config,
    project_root: &Path,
) -> LocalTaskOperations {
    let backend_info = resolve_backend_info(config, project_root);
    let hooks = create_hook_executor(config.clone(), RuntimeMode::Cli, backend_info, backend.clone());
    let pr_verifier: Arc<dyn PrVerifier> = Arc::new(GhCliPrVerifier);
    let completion_policy = CompletionPolicy::new(config.workflow.merge_via);
    LocalTaskOperations::new(backend, hooks, pr_verifier, completion_policy)
}

pub fn create_remote_task_operations(
    config: &Config,
    project_root: &Path,
    backend: Arc<dyn TaskBackend>,
) -> RemoteTaskOperations {
    let url = config.backend.api_url.as_ref().expect("api_url required for remote operations");
    let api_key = config.backend.api_key.clone();

    let backend_info = resolve_backend_info(config, project_root);
    let hooks = create_hook_executor(
        config.clone(),
        RuntimeMode::Cli,
        backend_info,
        backend,
    );

    RemoteTaskOperations::new(url, api_key, hooks)
}

/// Create the appropriate `TaskOperations` implementation based on config.
/// Returns (task_ops, backend) — backend is still needed for project/user operations.
pub fn create_task_operations(
    project_root: &Path,
    config: &Config,
) -> Result<(Arc<dyn TaskOperations>, Arc<dyn TaskBackend>)> {
    let backend = create_backend(project_root, config)?;
    let using_http = config.backend.api_url.is_some();
    let task_ops: Arc<dyn TaskOperations> = if using_http {
        Arc::new(create_remote_task_operations(config, project_root, backend.clone()))
    } else {
        Arc::new(create_local_task_operations(backend.clone(), config, project_root))
    };
    Ok((task_ops, backend))
}

pub fn create_project_service(backend: Arc<dyn TaskBackend>) -> ProjectService {
    ProjectService::new(backend)
}

pub fn create_user_service(backend: Arc<dyn TaskBackend>) -> UserService {
    UserService::new(backend)
}

pub fn create_hook_test_service(
    backend: Arc<dyn TaskBackend>,
    config: &Config,
    project_root: &Path,
) -> HookTestService {
    let backend_info = resolve_backend_info(config, project_root);
    let hook_test = Arc::new(ShellHookTestExecutor::new(
        config.clone(),
        RuntimeMode::Cli,
        backend_info,
        backend.clone(),
    ));
    HookTestService::new(backend, hook_test)
}

/// Resolve the project ID from config (CLI > env > config.toml already applied).
pub async fn resolve_project_id(
    backend: &dyn TaskBackend,
    config: &Config,
) -> Result<i64> {
    match config.project.name.as_deref() {
        Some(n) => {
            let project = backend
                .get_project_by_name(n)
                .await
                .with_context(|| format!("project not found: {n}"))?;
            Ok(project.id())
        }
        None => Ok(DEFAULT_PROJECT_ID),
    }
}

/// Resolve the user ID from config (CLI > env > config.toml already applied).
pub async fn resolve_user_id(
    backend: &dyn TaskBackend,
    config: &Config,
) -> Result<i64> {
    match config.user.name.as_deref() {
        Some(n) => {
            let user = backend
                .get_user_by_username(n)
                .await
                .with_context(|| format!("user not found: {n}"))?;
            Ok(user.id())
        }
        None => Ok(DEFAULT_USER_ID),
    }
}

pub fn init_tracing(config: &LogConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = tracing_subscriber::registry().with(env_filter);

    match config.format {
        LogFormat::Json => {
            registry
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        LogFormat::Pretty => {
            registry.with(tracing_subscriber::fmt::layer()).init();
        }
    }
}

pub fn load_config(project_root: &Path, explicit_config: Option<&Path>) -> Result<Config> {
    // 1. Load user config + user local overlay
    let (user_raw, user_local) = load_user_config()?;

    // 2. Load project/explicit config + its local overlay
    let (project_raw, project_local) = if let Some(path) = explicit_config {
        let raw = load_config_file(path, true)?;
        let local = load_local_overlay(path)?;
        (Some(raw), local)
    } else if let Some(env_path) = env_config_path() {
        let raw = load_config_file(&env_path, true)?;
        let local = load_local_overlay(&env_path)?;
        (Some(raw), local)
    } else {
        let default_path = project_root.join(".senko").join("config.toml");
        if default_path.exists() {
            let raw = load_config_file(&default_path, false)?;
            let local = load_local_overlay(&default_path)?;
            (Some(raw), local)
        } else {
            (None, None)
        }
    };

    // 3. Merge: user → user local → project → project local
    let mut merged = user_raw.unwrap_or_default();
    if let Some(local) = user_local {
        merged = merged.merge(local);
    }
    if let Some(project) = project_raw {
        merged = merged.merge(project);
    }
    if let Some(local) = project_local {
        merged = merged.merge(local);
    }

    // 4. Resolve to final Config and apply env overrides
    let mut config = merged.resolve();
    config.apply_env();
    Ok(config)
}

/// Return the user-level config path.
/// `$XDG_CONFIG_HOME/senko/config.toml` or `~/.config/senko/config.toml`
fn user_config_path() -> Option<PathBuf> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        })?;
    Some(config_dir.join("senko").join("config.toml"))
}

/// Load user-level config and its local overlay if they exist.
fn load_user_config() -> Result<(Option<RawConfig>, Option<RawConfig>)> {
    let path = match user_config_path() {
        Some(p) if p.exists() => p,
        _ => return Ok((None, None)),
    };
    let raw = load_config_file(&path, false)?;
    let local = load_local_overlay(&path)?;
    Ok((Some(raw), local))
}

/// Load config.local.toml from the same directory as the given config file.
fn load_local_overlay(config_path: &Path) -> Result<Option<RawConfig>> {
    let local_path = config_path.with_file_name("config.local.toml");
    if local_path.exists() {
        Ok(Some(load_config_file(&local_path, false)?))
    } else {
        Ok(None)
    }
}

/// Return the config path from the SENKO_CONFIG env var, if set.
fn env_config_path() -> Option<PathBuf> {
    std::env::var("SENKO_CONFIG")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Load and parse a config file into RawConfig, with legacy hook format detection.
fn load_config_file(path: &Path, must_exist: bool) -> Result<RawConfig> {
    if !path.exists() {
        if must_exist {
            bail!("config file not found: {}", path.display());
        }
        return Ok(RawConfig::default());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    detect_legacy_hook_format(&content, path)?;
    toml::from_str(&content)
        .with_context(|| format!("failed to parse config file: {}", path.display()))
}

/// Check if the config uses the old array-based hook format and return a helpful error.
fn detect_legacy_hook_format(content: &str, path: &Path) -> Result<()> {
    let raw: toml::Value = match toml::from_str(content) {
        Ok(v) => v,
        Err(_) => return Ok(()), // let the real parser produce the error
    };
    if let Some(hooks) = raw.get("hooks").and_then(|v| v.as_table()) {
        for (key, val) in hooks {
            if val.is_str() || val.is_array() {
                bail!(
                    "Legacy hook format detected in {}.\n\
                     The array-based hook format is no longer supported.\n\
                     Please migrate to named hooks:\n\n\
                     Old format:\n  [hooks]\n  {} = \"command\"\n\n\
                     New format:\n  [hooks.{}.my-hook]\n  command = \"command\"\n",
                    path.display(),
                    key,
                    key,
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;

    /// Run `load_config` in an isolated environment where no real user config
    /// or env-var config can leak in.
    /// Isolate env vars so no real user config or env-var config can leak in.
    fn isolate_env(project_root: &Path) {
        let empty = project_root.join("__no_user_config__");
        // SAFETY: all callers are marked #[serial] to avoid env var races.
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &empty);
            std::env::remove_var("SENKO_CONFIG");
            std::env::remove_var("SENKO_USER");
            std::env::remove_var("SENKO_PROJECT");
        }
    }

    /// Run `load_config` in an isolated environment where no real user config
    /// or env-var config can leak in.
    fn load_config_isolated(project_root: &Path) -> Result<Config> {
        isolate_env(project_root);
        load_config(project_root, None)
    }

    #[test]
    #[serial]
    fn load_config_with_local_overlay() {
        let dir = tempfile::tempdir().unwrap();
        let senko_dir = dir.path().join(".senko");
        fs::create_dir_all(&senko_dir).unwrap();

        fs::write(
            senko_dir.join("config.toml"),
            r#"
[user]
name = "project-user"

[project]
name = "my-project"
"#,
        )
        .unwrap();

        fs::write(
            senko_dir.join("config.local.toml"),
            r#"
[user]
name = "local-user"
"#,
        )
        .unwrap();

        let config = load_config_isolated(dir.path()).unwrap();
        assert_eq!(config.user.name.as_deref(), Some("local-user"));
        assert_eq!(config.project.name.as_deref(), Some("my-project"));
    }

    #[test]
    #[serial]
    fn load_config_without_local_file() {
        let dir = tempfile::tempdir().unwrap();
        let senko_dir = dir.path().join(".senko");
        fs::create_dir_all(&senko_dir).unwrap();

        fs::write(
            senko_dir.join("config.toml"),
            r#"
[user]
name = "project-user"
"#,
        )
        .unwrap();

        let config = load_config_isolated(dir.path()).unwrap();
        assert_eq!(config.user.name.as_deref(), Some("project-user"));
    }

    #[test]
    #[serial]
    fn load_config_explicit_config_uses_sibling_local() {
        let dir = tempfile::tempdir().unwrap();
        let custom_dir = dir.path().join("custom");
        fs::create_dir_all(&custom_dir).unwrap();

        fs::write(
            custom_dir.join("config.toml"),
            r#"
[user]
name = "custom-user"
"#,
        )
        .unwrap();

        fs::write(
            custom_dir.join("config.local.toml"),
            r#"
[user]
name = "custom-local-user"
"#,
        )
        .unwrap();

        isolate_env(dir.path());
        let config = load_config(dir.path(), Some(&custom_dir.join("config.toml"))).unwrap();
        assert_eq!(config.user.name.as_deref(), Some("custom-local-user"));
    }

    #[test]
    #[serial]
    fn load_config_explicit_config_ignores_project_local() {
        let dir = tempfile::tempdir().unwrap();
        let senko_dir = dir.path().join(".senko");
        let custom_dir = dir.path().join("custom");
        fs::create_dir_all(&senko_dir).unwrap();
        fs::create_dir_all(&custom_dir).unwrap();

        // Project local overlay should NOT be loaded when --config is used
        fs::write(
            senko_dir.join("config.local.toml"),
            r#"
[user]
name = "project-local-user"
"#,
        )
        .unwrap();

        fs::write(
            custom_dir.join("config.toml"),
            r#"
[user]
name = "custom-user"
"#,
        )
        .unwrap();

        isolate_env(dir.path());
        let config = load_config(dir.path(), Some(&custom_dir.join("config.toml"))).unwrap();
        // Should be "custom-user", NOT "project-local-user"
        assert_eq!(config.user.name.as_deref(), Some("custom-user"));
    }

    #[test]
    #[serial]
    fn load_config_user_local_overlay() {
        let dir = tempfile::tempdir().unwrap();
        let user_config_dir = dir.path().join("user_config").join("senko");
        fs::create_dir_all(&user_config_dir).unwrap();

        fs::write(
            user_config_dir.join("config.toml"),
            r#"
[user]
name = "base-user"
"#,
        )
        .unwrap();

        fs::write(
            user_config_dir.join("config.local.toml"),
            r#"
[user]
name = "user-local-override"
"#,
        )
        .unwrap();

        // Point XDG_CONFIG_HOME to our test dir
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", dir.path().join("user_config"));
            std::env::remove_var("SENKO_CONFIG");
            std::env::remove_var("SENKO_USER");
            std::env::remove_var("SENKO_PROJECT");
        }

        let project_dir = dir.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let config = load_config(&project_dir, None).unwrap();
        assert_eq!(config.user.name.as_deref(), Some("user-local-override"));
    }

    #[test]
    #[serial]
    fn load_config_merge_order() {
        // Verify: user → user local → project → project local
        let dir = tempfile::tempdir().unwrap();
        let user_config_dir = dir.path().join("user_config").join("senko");
        let senko_dir = dir.path().join("project").join(".senko");
        fs::create_dir_all(&user_config_dir).unwrap();
        fs::create_dir_all(&senko_dir).unwrap();

        // User config sets user.name and project.name
        fs::write(
            user_config_dir.join("config.toml"),
            r#"
[user]
name = "user-base"

[project]
name = "user-project"
"#,
        )
        .unwrap();

        // User local overrides user.name only
        fs::write(
            user_config_dir.join("config.local.toml"),
            r#"
[user]
name = "user-local"
"#,
        )
        .unwrap();

        // Project config overrides project.name, sets a new field
        fs::write(
            senko_dir.join("config.toml"),
            r#"
[project]
name = "project-base"
"#,
        )
        .unwrap();

        // Project local overrides project.name
        fs::write(
            senko_dir.join("config.local.toml"),
            r#"
[project]
name = "project-local"
"#,
        )
        .unwrap();

        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", dir.path().join("user_config"));
            std::env::remove_var("SENKO_CONFIG");
            std::env::remove_var("SENKO_USER");
            std::env::remove_var("SENKO_PROJECT");
        }

        let project_dir = dir.path().join("project");
        let config = load_config(&project_dir, None).unwrap();

        // user.name: user-base → user-local (user local wins over user base)
        // project config and project local don't set user.name, so user-local stays
        assert_eq!(config.user.name.as_deref(), Some("user-local"));

        // project.name: user-project → (user local doesn't set it) → project-base → project-local
        assert_eq!(config.project.name.as_deref(), Some("project-local"));
    }

    // --- auth config validation tests ---

    #[test]
    fn validate_auth_disabled_always_ok() {
        let config = Config::default(); // auth.enabled defaults to false
        validate_auth_config(&config).unwrap();
    }

    #[test]
    fn validate_auth_enabled_with_oidc_ok() {
        let mut config = Config::default();
        config.auth.enabled = true;
        config.auth.oidc.issuer_url = Some("https://example.com".to_string());
        config.auth.oidc.client_id = Some("my-client".to_string());
        validate_auth_config(&config).unwrap();
    }

    #[test]
    fn validate_auth_enabled_with_master_key_ok() {
        let mut config = Config::default();
        config.auth.enabled = true;
        config.auth.master_api_key = Some("secret".to_string());
        validate_auth_config(&config).unwrap();
    }

    #[test]
    fn validate_auth_enabled_with_both_ok() {
        let mut config = Config::default();
        config.auth.enabled = true;
        config.auth.oidc.issuer_url = Some("https://example.com".to_string());
        config.auth.oidc.client_id = Some("my-client".to_string());
        config.auth.master_api_key = Some("secret".to_string());
        validate_auth_config(&config).unwrap();
    }

    #[test]
    fn validate_auth_enabled_with_neither_fails() {
        let mut config = Config::default();
        config.auth.enabled = true;
        let err = validate_auth_config(&config).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("master_api_key"), "error should mention master_api_key: {msg}");
        assert!(msg.contains("oidc"), "error should mention oidc: {msg}");
    }
}
