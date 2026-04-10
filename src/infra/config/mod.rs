use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use crate::domain::task::{BranchMode, MergeStrategy, MergeVia};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
    #[serde(default)]
    pub backend: BackendConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub user: UserConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub skill: SkillConfig,
}

// --- Skill config ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "source")]
pub enum MetadataFieldSource {
    Env {
        env_var: String,
        #[serde(default)]
        default: Option<String>,
    },
    Fixed {
        value: serde_json::Value,
    },
    Prompt {
        prompt: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataField {
    pub key: String,
    #[serde(flatten)]
    pub source: MetadataFieldSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillStartConfig {
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillConfig {
    #[serde(default)]
    pub start: SkillStartConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    pub master_api_key: Option<String>,
    pub master_api_key_arn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogConfig {
    pub dir: Option<String>,
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub format: LogFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[cfg(feature = "dynamodb")]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DynamoDbConfig {
    pub table_name: Option<String>,
    pub region: Option<String>,
}

#[cfg(feature = "postgres")]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PostgresConfig {
    pub url: Option<String>,
    pub url_arn: Option<String>,
    pub max_connections: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    pub db_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendConfig {
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    #[cfg(feature = "dynamodb")]
    #[serde(default)]
    pub dynamodb: Option<DynamoDbConfig>,
    #[cfg(feature = "postgres")]
    #[serde(default)]
    pub postgres: Option<PostgresConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum WorkflowEventType {
    Command { command: String },
    Prompt { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub point: String,
    #[serde(flatten)]
    pub event_type: WorkflowEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default)]
    pub merge_via: MergeVia,
    #[serde(default = "default_true")]
    pub auto_merge: bool,
    #[serde(default)]
    pub branch_mode: BranchMode,
    #[serde(default)]
    pub merge_strategy: MergeStrategy,
    #[serde(default)]
    pub events: Vec<WorkflowEvent>,
}

fn default_true() -> bool {
    true
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            merge_via: MergeVia::default(),
            auto_merge: true,
            branch_mode: BranchMode::default(),
            merge_strategy: MergeStrategy::default(),
            events: Vec::new(),
        }
    }
}

// --- Named hook types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEntry {
    pub command: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub requires_env: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub on_task_added: BTreeMap<String, HookEntry>,
    #[serde(default)]
    pub on_task_ready: BTreeMap<String, HookEntry>,
    #[serde(default)]
    pub on_task_started: BTreeMap<String, HookEntry>,
    #[serde(default)]
    pub on_task_completed: BTreeMap<String, HookEntry>,
    #[serde(default)]
    pub on_task_canceled: BTreeMap<String, HookEntry>,
    #[serde(default)]
    pub on_no_eligible_task: BTreeMap<String, HookEntry>,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            on_task_added: BTreeMap::new(),
            on_task_ready: BTreeMap::new(),
            on_task_started: BTreeMap::new(),
            on_task_completed: BTreeMap::new(),
            on_task_canceled: BTreeMap::new(),
            on_no_eligible_task: BTreeMap::new(),
        }
    }
}

impl HooksConfig {
    /// Get enabled commands for a given event name.
    pub fn commands_for_event(&self, event_name: &str) -> Vec<&str> {
        let map = match event_name {
            "task_added" => &self.on_task_added,
            "task_ready" => &self.on_task_ready,
            "task_started" => &self.on_task_started,
            "task_completed" => &self.on_task_completed,
            "task_canceled" => &self.on_task_canceled,
            "no_eligible_task" => &self.on_no_eligible_task,
            _ => return vec![],
        };
        map.values()
            .filter(|e| e.enabled)
            .map(|e| e.command.as_str())
            .collect()
    }

    /// Get enabled entries with their names for a given event name.
    pub fn entries_for_event(&self, event_name: &str) -> Vec<(&str, &HookEntry)> {
        let map = match event_name {
            "task_added" => &self.on_task_added,
            "task_ready" => &self.on_task_ready,
            "task_started" => &self.on_task_started,
            "task_completed" => &self.on_task_completed,
            "task_canceled" => &self.on_task_canceled,
            "no_eligible_task" => &self.on_no_eligible_task,
            _ => return vec![],
        };
        map.iter()
            .filter(|(_, e)| e.enabled)
            .map(|(name, entry)| (name.as_str(), entry))
            .collect()
    }
}

// --- RawConfig for layered merging ---

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawConfig {
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub workflow: RawWorkflowConfig,
    #[serde(default)]
    pub backend: RawBackendConfig,
    #[serde(default)]
    pub log: RawLogConfig,
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub user: UserConfig,
    #[serde(default)]
    pub auth: RawAuthConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub web: RawWebConfig,
    #[serde(default)]
    pub skill: Option<SkillConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawWorkflowConfig {
    #[serde(alias = "completion_mode")]
    pub merge_via: Option<MergeVia>,
    pub auto_merge: Option<bool>,
    pub branch_mode: Option<BranchMode>,
    pub merge_strategy: Option<MergeStrategy>,
    pub events: Option<Vec<WorkflowEvent>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawBackendConfig {
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    #[cfg(feature = "dynamodb")]
    pub dynamodb: Option<DynamoDbConfig>,
    #[cfg(feature = "postgres")]
    pub postgres: Option<PostgresConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawLogConfig {
    pub dir: Option<String>,
    pub level: Option<String>,
    pub format: Option<LogFormat>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawAuthConfig {
    pub enabled: Option<bool>,
    pub master_api_key: Option<String>,
    pub master_api_key_arn: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawWebConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

impl RawConfig {
    /// Merge two configs: `self` is the base (lower priority), `overlay` wins.
    pub fn merge(self, overlay: RawConfig) -> RawConfig {
        RawConfig {
            hooks: merge_hooks(self.hooks, overlay.hooks),
            workflow: RawWorkflowConfig {
                merge_via: overlay.workflow.merge_via.or(self.workflow.merge_via),
                auto_merge: overlay.workflow.auto_merge.or(self.workflow.auto_merge),
                branch_mode: overlay.workflow.branch_mode.or(self.workflow.branch_mode),
                merge_strategy: overlay.workflow.merge_strategy.or(self.workflow.merge_strategy),
                events: overlay.workflow.events.or(self.workflow.events),
            },
            backend: RawBackendConfig {
                api_url: overlay.backend.api_url.or(self.backend.api_url),
                api_key: overlay.backend.api_key.or(self.backend.api_key),
                #[cfg(feature = "dynamodb")]
                dynamodb: overlay.backend.dynamodb.or(self.backend.dynamodb),
                #[cfg(feature = "postgres")]
                postgres: overlay.backend.postgres.or(self.backend.postgres),
            },
            log: RawLogConfig {
                dir: overlay.log.dir.or(self.log.dir),
                level: overlay.log.level.or(self.log.level),
                format: overlay.log.format.or(self.log.format),
            },
            project: ProjectConfig {
                name: overlay.project.name.or(self.project.name),
            },
            user: UserConfig {
                name: overlay.user.name.or(self.user.name),
            },
            auth: RawAuthConfig {
                enabled: overlay.auth.enabled.or(self.auth.enabled),
                master_api_key: overlay.auth.master_api_key.or(self.auth.master_api_key),
                master_api_key_arn: overlay.auth.master_api_key_arn.or(self.auth.master_api_key_arn),
            },
            storage: StorageConfig {
                db_path: overlay.storage.db_path.or(self.storage.db_path),
            },
            web: RawWebConfig {
                host: overlay.web.host.or(self.web.host),
                port: overlay.web.port.or(self.web.port),
            },
            skill: overlay.skill.or(self.skill),
        }
    }

    /// Resolve to final Config, filling None values with defaults.
    pub fn resolve(self) -> Config {
        Config {
            hooks: self.hooks,
            workflow: WorkflowConfig {
                merge_via: self.workflow.merge_via.unwrap_or_default(),
                auto_merge: self.workflow.auto_merge.unwrap_or(true),
                branch_mode: self.workflow.branch_mode.unwrap_or_default(),
                merge_strategy: self.workflow.merge_strategy.unwrap_or_default(),
                events: self.workflow.events.unwrap_or_default(),
            },
            backend: BackendConfig {
                api_url: self.backend.api_url,
                api_key: self.backend.api_key,
                #[cfg(feature = "dynamodb")]
                dynamodb: self.backend.dynamodb,
                #[cfg(feature = "postgres")]
                postgres: self.backend.postgres,
            },
            log: LogConfig {
                dir: self.log.dir,
                level: self.log.level.unwrap_or_else(default_log_level),
                format: self.log.format.unwrap_or_default(),
            },
            project: self.project,
            user: self.user,
            auth: AuthConfig {
                enabled: self.auth.enabled.unwrap_or(false),
                master_api_key: self.auth.master_api_key,
                master_api_key_arn: self.auth.master_api_key_arn,
            },
            storage: self.storage,
            web: WebConfig {
                host: self.web.host,
                port: self.web.port,
            },
            skill: self.skill.unwrap_or_default(),
        }
    }
}

/// Merge hooks: base hooks + overlay hooks. Same-name hooks: overlay wins.
/// Disabled hooks (enabled=false) are kept in the map (filtered at execution time).
fn merge_hooks(base: HooksConfig, overlay: HooksConfig) -> HooksConfig {
    fn merge_map(
        mut base: BTreeMap<String, HookEntry>,
        overlay: BTreeMap<String, HookEntry>,
    ) -> BTreeMap<String, HookEntry> {
        for (name, entry) in overlay {
            base.insert(name, entry);
        }
        base
    }
    HooksConfig {
        enabled: overlay.enabled,
        on_task_added: merge_map(base.on_task_added, overlay.on_task_added),
        on_task_ready: merge_map(base.on_task_ready, overlay.on_task_ready),
        on_task_started: merge_map(base.on_task_started, overlay.on_task_started),
        on_task_completed: merge_map(base.on_task_completed, overlay.on_task_completed),
        on_task_canceled: merge_map(base.on_task_canceled, overlay.on_task_canceled),
        on_no_eligible_task: merge_map(base.on_no_eligible_task, overlay.on_no_eligible_task),
    }
}

// --- CLI overrides ---

#[derive(Debug, Default)]
pub struct CliOverrides {
    pub log_dir: Option<String>,
    pub db_path: Option<String>,
    pub postgres_url: Option<String>,
    pub project: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub host: Option<String>,
}

impl Config {
    /// Apply environment variable overrides. Call after `RawConfig::resolve()`.
    /// Priority: env > config.toml defaults.
    pub fn apply_env(&mut self) {
        // Workflow settings
        // Check SENKO_MERGE_VIA first, then fallback to deprecated SENKO_COMPLETION_MODE
        let merge_via_env = std::env::var("SENKO_MERGE_VIA")
            .ok()
            .or_else(|| {
                std::env::var("SENKO_COMPLETION_MODE").ok().inspect(|_| {
                    eprintln!("warning: SENKO_COMPLETION_MODE is deprecated, use SENKO_MERGE_VIA");
                })
            });
        if let Some(val) = merge_via_env {
            match val.as_str() {
                "direct" => self.workflow.merge_via = MergeVia::Direct,
                "pr" => self.workflow.merge_via = MergeVia::Pr,
                "merge_then_complete" => {
                    eprintln!("warning: merge_via value \"merge_then_complete\" is deprecated, use \"direct\"");
                    self.workflow.merge_via = MergeVia::Direct;
                }
                "pr_then_complete" => {
                    eprintln!("warning: merge_via value \"pr_then_complete\" is deprecated, use \"pr\"");
                    self.workflow.merge_via = MergeVia::Pr;
                }
                other => eprintln!("warning: unknown SENKO_MERGE_VIA={other}, ignoring"),
            }
        }
        if let Ok(val) = std::env::var("SENKO_AUTO_MERGE") {
            match val.to_lowercase().as_str() {
                "true" | "1" | "yes" => self.workflow.auto_merge = true,
                "false" | "0" | "no" => self.workflow.auto_merge = false,
                other => eprintln!("warning: unknown SENKO_AUTO_MERGE={other}, ignoring"),
            }
        }
        if let Ok(val) = std::env::var("SENKO_BRANCH_MODE") {
            match val.as_str() {
                "worktree" => self.workflow.branch_mode = BranchMode::Worktree,
                "branch" => self.workflow.branch_mode = BranchMode::Branch,
                other => eprintln!("warning: unknown SENKO_BRANCH_MODE={other}, ignoring"),
            }
        }
        if let Ok(val) = std::env::var("SENKO_MERGE_STRATEGY") {
            match val.as_str() {
                "rebase" => self.workflow.merge_strategy = MergeStrategy::Rebase,
                "squash" => self.workflow.merge_strategy = MergeStrategy::Squash,
                other => eprintln!("warning: unknown SENKO_MERGE_STRATEGY={other}, ignoring"),
            }
        }

        // Backend settings
        if let Ok(val) = std::env::var("SENKO_API_URL")
            && !val.is_empty() {
                self.backend.api_url = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_API_KEY")
            && !val.is_empty() {
                self.backend.api_key = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_HOOKS_ENABLED") {
            match val.to_lowercase().as_str() {
                "true" | "1" => self.hooks.enabled = true,
                "false" | "0" => self.hooks.enabled = false,
                other => eprintln!("warning: unknown SENKO_HOOKS_ENABLED={other}, ignoring"),
            }
        }

        // Auth settings
        if let Ok(val) = std::env::var("SENKO_MASTER_API_KEY")
            && !val.is_empty() {
                self.auth.master_api_key = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_MASTER_API_KEY_ARN")
            && !val.is_empty() {
                self.auth.master_api_key_arn = Some(val);
            }

        // DynamoDB settings (feature-gated)
        #[cfg(feature = "dynamodb")]
        {
            if let Ok(val) = std::env::var("SENKO_DYNAMODB_TABLE") {
                if !val.is_empty() {
                    self.backend
                        .dynamodb
                        .get_or_insert_with(DynamoDbConfig::default)
                        .table_name = Some(val);
                }
            }
            if let Ok(val) = std::env::var("SENKO_DYNAMODB_REGION") {
                if !val.is_empty() {
                    self.backend
                        .dynamodb
                        .get_or_insert_with(DynamoDbConfig::default)
                        .region = Some(val);
                }
            }
        }

        // PostgreSQL settings (feature-gated)
        #[cfg(feature = "postgres")]
        {
            if let Ok(val) = std::env::var("SENKO_POSTGRES_URL") {
                if !val.is_empty() {
                    self.backend
                        .postgres
                        .get_or_insert_with(PostgresConfig::default)
                        .url = Some(val);
                }
            }
            if let Ok(val) = std::env::var("SENKO_POSTGRES_URL_ARN") {
                if !val.is_empty() {
                    self.backend
                        .postgres
                        .get_or_insert_with(PostgresConfig::default)
                        .url_arn = Some(val);
                }
            }
            if let Ok(val) = std::env::var("SENKO_POSTGRES_MAX_CONNECTIONS") {
                if !val.is_empty() {
                    if let Ok(n) = val.parse::<u32>() {
                        self.backend
                            .postgres
                            .get_or_insert_with(PostgresConfig::default)
                            .max_connections = Some(n);
                    }
                }
            }
        }

        // Hook commands (insert as named "_env" entry)
        fn insert_env_hook(map: &mut BTreeMap<String, HookEntry>, val: String) {
            map.insert(
                "_env".to_string(),
                HookEntry {
                    command: val,
                    enabled: true,
                    requires_env: vec![],
                },
            );
        }
        if let Ok(val) = std::env::var("SENKO_HOOK_ON_TASK_ADDED")
            && !val.is_empty() {
                insert_env_hook(&mut self.hooks.on_task_added, val);
            }
        if let Ok(val) = std::env::var("SENKO_HOOK_ON_TASK_READY")
            && !val.is_empty() {
                insert_env_hook(&mut self.hooks.on_task_ready, val);
            }
        if let Ok(val) = std::env::var("SENKO_HOOK_ON_TASK_STARTED")
            && !val.is_empty() {
                insert_env_hook(&mut self.hooks.on_task_started, val);
            }
        if let Ok(val) = std::env::var("SENKO_HOOK_ON_TASK_COMPLETED")
            && !val.is_empty() {
                insert_env_hook(&mut self.hooks.on_task_completed, val);
            }
        if let Ok(val) = std::env::var("SENKO_HOOK_ON_TASK_CANCELED")
            && !val.is_empty() {
                insert_env_hook(&mut self.hooks.on_task_canceled, val);
            }
        if let Ok(val) = std::env::var("SENKO_HOOK_ON_NO_ELIGIBLE_TASK")
            && !val.is_empty() {
                insert_env_hook(&mut self.hooks.on_no_eligible_task, val);
            }

        // User settings
        if let Ok(val) = std::env::var("SENKO_USER")
            && !val.is_empty() {
                self.user.name = Some(val);
            }

        // Project settings
        if let Ok(val) = std::env::var("SENKO_PROJECT")
            && !val.is_empty() {
                self.project.name = Some(val);
            }

        // Storage settings
        if let Ok(val) = std::env::var("SENKO_DB_PATH")
            && !val.is_empty() {
                self.storage.db_path = Some(val);
            }

        // Log settings
        if let Ok(val) = std::env::var("SENKO_LOG_DIR")
            && !val.is_empty() {
                self.log.dir = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_LOG_LEVEL")
            && !val.is_empty() {
                self.log.level = val;
            }
        if let Ok(val) = std::env::var("SENKO_LOG_FORMAT") {
            match val.to_lowercase().as_str() {
                "json" => self.log.format = LogFormat::Json,
                "pretty" => self.log.format = LogFormat::Pretty,
                other => eprintln!("warning: unknown SENKO_LOG_FORMAT={other}, ignoring"),
            }
        }

        // Web settings
        if let Ok(val) = std::env::var("SENKO_PORT")
            && let Ok(port) = val.parse::<u16>() {
                self.web.port = Some(port);
            }
        if let Ok(val) = std::env::var("SENKO_HOST")
            && !val.is_empty() {
                self.web.host = Some(val);
            }
    }

    /// Apply CLI argument overrides. Call after `apply_env()`.
    /// Priority: CLI > env > config.toml > defaults.
    pub fn apply_cli(&mut self, overrides: &CliOverrides) {
        if let Some(ref dir) = overrides.log_dir {
            self.log.dir = Some(dir.clone());
        }
        if let Some(ref path) = overrides.db_path {
            self.storage.db_path = Some(path.clone());
        }
        #[cfg(feature = "postgres")]
        if let Some(ref url) = overrides.postgres_url {
            self.backend
                .postgres
                .get_or_insert_with(PostgresConfig::default)
                .url = Some(url.clone());
        }
        if let Some(ref name) = overrides.project {
            self.project.name = Some(name.clone());
        }
        if let Some(ref name) = overrides.user {
            self.user.name = Some(name.clone());
        }
        if let Some(port) = overrides.port {
            self.web.port = Some(port);
        }
        if let Some(ref host) = overrides.host {
            self.web.host = Some(host.clone());
        }
    }

    pub fn web_port_or(&self, default: u16) -> u16 {
        self.web.port.unwrap_or(default)
    }

    pub fn web_port_is_explicit(&self) -> bool {
        self.web.port.is_some()
    }

    pub fn effective_host(&self) -> String {
        self.web
            .host
            .clone()
            .unwrap_or_else(|| "127.0.0.1".to_string())
    }

    /// Resolve secrets from AWS Secrets Manager using ARN config fields.
    /// Call after `apply_env()`. ARN-resolved values overwrite direct values.
    #[cfg(feature = "aws-secrets")]
    pub async fn resolve_secrets(&mut self) -> anyhow::Result<()> {
        use crate::infra::secrets::SecretsManagerClient;

        let region: Option<String> = {
            #[cfg(feature = "dynamodb")]
            {
                self.backend
                    .dynamodb
                    .as_ref()
                    .and_then(|d| d.region.clone())
            }
            #[cfg(not(feature = "dynamodb"))]
            {
                None
            }
        };

        let client = SecretsManagerClient::new(region);
        self.resolve_secrets_with(&client).await
    }

    /// Resolve secrets using the provided client. Separated for testability.
    #[cfg(feature = "aws-secrets")]
    pub(crate) async fn resolve_secrets_with(
        &mut self,
        client: &crate::infra::secrets::SecretsManagerClient,
    ) -> anyhow::Result<()> {
        if let Some(ref arn) = self.auth.master_api_key_arn {
            let secret = client.get_secret(arn).await?;
            self.auth.master_api_key = Some(secret);
        }

        #[cfg(feature = "postgres")]
        if let Some(ref arn) = self
            .backend
            .postgres
            .as_ref()
            .and_then(|p| p.url_arn.clone())
        {
            let secret = client.get_secret(arn).await?;
            self.backend
                .postgres
                .get_or_insert_with(PostgresConfig::default)
                .url = Some(secret);
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "aws-secrets"))]
mod resolve_secrets_tests {
    use super::*;
    use crate::infra::secrets::{SecretFetcher, SecretsManagerClient};
    use async_trait::async_trait;
    use std::collections::HashMap;

    struct FakeSecretFetcher {
        secrets: HashMap<String, String>,
    }

    #[async_trait]
    impl SecretFetcher for FakeSecretFetcher {
        async fn fetch_secret(&self, arn: &str) -> anyhow::Result<String> {
            self.secrets
                .get(arn)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("secret not found: {arn}"))
        }
    }

    fn make_client(secrets: HashMap<String, String>) -> SecretsManagerClient {
        SecretsManagerClient::with_fetcher(Box::new(FakeSecretFetcher { secrets }))
    }

    #[tokio::test]
    async fn arn_overwrites_direct_value() {
        let mut config = Config::default();
        config.auth.master_api_key = Some("direct-value".to_string());
        config.auth.master_api_key_arn = Some("arn:aws:secretsmanager:us-east-1:123:secret:key".to_string());

        let client = make_client(HashMap::from([(
            "arn:aws:secretsmanager:us-east-1:123:secret:key".to_string(),
            "arn-resolved-value".to_string(),
        )]));

        config.resolve_secrets_with(&client).await.unwrap();

        assert_eq!(
            config.auth.master_api_key.as_deref(),
            Some("arn-resolved-value")
        );
    }

    #[tokio::test]
    async fn arn_only_sets_master_key() {
        let mut config = Config::default();
        config.auth.master_api_key_arn = Some("arn:aws:secretsmanager:us-east-1:123:secret:key".to_string());

        let client = make_client(HashMap::from([(
            "arn:aws:secretsmanager:us-east-1:123:secret:key".to_string(),
            "arn-resolved-value".to_string(),
        )]));

        config.resolve_secrets_with(&client).await.unwrap();

        assert_eq!(
            config.auth.master_api_key.as_deref(),
            Some("arn-resolved-value")
        );
    }

    #[tokio::test]
    async fn direct_only_unchanged() {
        let mut config = Config::default();
        config.auth.master_api_key = Some("direct-value".to_string());

        let client = make_client(HashMap::new());

        config.resolve_secrets_with(&client).await.unwrap();

        assert_eq!(
            config.auth.master_api_key.as_deref(),
            Some("direct-value")
        );
    }

    #[tokio::test]
    async fn neither_set_remains_none() {
        let mut config = Config::default();

        let client = make_client(HashMap::new());

        config.resolve_secrets_with(&client).await.unwrap();

        assert!(config.auth.master_api_key.is_none());
    }
}
