use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Deserializer, Serialize};

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
    pub server: ServerConfig,
    #[serde(default)]
    pub cli: CliConfig,
    #[serde(default)]
    pub web: WebConfig,
}

// --- Hook definition types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OnFailure {
    #[default]
    Abort,
    Warn,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookObject {
    pub command: Option<String>,
    pub prompt: Option<String>,
    #[serde(default)]
    pub on_failure: OnFailure,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum HookDef {
    Simple(String),
    Complex(HookObject),
}

impl<'de> Deserialize<'de> for HookDef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Simple(String),
            Complex(HookObject),
        }
        match Helper::deserialize(deserializer)? {
            Helper::Simple(s) => Ok(HookDef::Simple(s)),
            Helper::Complex(o) => Ok(HookDef::Complex(o)),
        }
    }
}

// --- Metadata field types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "source")]
pub enum MetadataFieldSource {
    Env {
        env_var: String,
    },
    Prompt {
        prompt: String,
    },
    Value {
        value: serde_json::Value,
    },
    Command {
        command: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataField {
    pub key: String,
    #[serde(flatten)]
    pub source: MetadataFieldSource,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub required: bool,
}

// --- Workflow event configs ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowEventConfig {
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub pre_hooks: Vec<HookDef>,
    #[serde(default)]
    pub post_hooks: Vec<HookDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowAddConfig {
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
    #[serde(default)]
    pub default_dod: Vec<String>,
    #[serde(default)]
    pub default_tags: Vec<String>,
    #[serde(default)]
    pub default_priority: Option<String>,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub pre_hooks: Vec<HookDef>,
    #[serde(default)]
    pub post_hooks: Vec<HookDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowStartConfig {
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub pre_hooks: Vec<HookDef>,
    #[serde(default)]
    pub post_hooks: Vec<HookDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowPlanConfig {
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
    #[serde(default)]
    pub required_sections: Vec<String>,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub pre_hooks: Vec<HookDef>,
    #[serde(default)]
    pub post_hooks: Vec<HookDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowCompleteConfig {
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub pre_hooks: Vec<HookDef>,
    #[serde(default)]
    pub post_hooks: Vec<HookDef>,
}

// --- Web config ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

// --- CLI config ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliRemoteConfig {
    pub url: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    #[serde(default = "default_true")]
    pub browser: bool,
    #[serde(default)]
    pub remote: CliRemoteConfig,
}

// --- Server config ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerRelayConfig {
    pub url: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub relay: ServerRelayConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

// --- Auth types (unchanged) ---

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct OidcConfig {
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub username_claim: Option<String>,
    #[serde(default = "default_oidc_scopes")]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub required_claims: HashMap<String, String>,
    #[serde(default)]
    pub callback_ports: Vec<String>,
    #[serde(default)]
    pub session: SessionConfig,
}

impl OidcConfig {
    /// Returns true when both issuer_url and client_id are set,
    /// meaning OIDC JWT verification should be enabled.
    pub fn is_configured(&self) -> bool {
        self.issuer_url.is_some() && self.client_id.is_some()
    }
}

fn default_oidc_scopes() -> Vec<String> {
    vec!["openid".to_string(), "profile".to_string()]
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SessionConfig {
    pub ttl: Option<String>,
    pub inactive_ttl: Option<String>,
    pub max_per_user: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ApiKeyConfig {
    pub master_key: Option<String>,
    pub master_key_arn: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TrustedHeadersConfig {
    pub subject_header: Option<String>,
    pub name_header: Option<String>,
    pub display_name_header: Option<String>,
    pub email_header: Option<String>,
    pub groups_header: Option<String>,
    pub scope_header: Option<String>,
    pub oidc_issuer_url: Option<String>,
    pub oidc_client_id: Option<String>,
}

impl TrustedHeadersConfig {
    pub fn is_configured(&self) -> bool {
        self.subject_header.is_some()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AuthConfig {
    #[serde(default)]
    pub api_key: ApiKeyConfig,
    #[serde(default)]
    pub oidc: OidcConfig,
    #[serde(default)]
    pub trusted_headers: TrustedHeadersConfig,
}

impl AuthConfig {
    /// Returns true when at least one authentication method is configured.
    pub fn is_configured(&self) -> bool {
        self.oidc.is_configured()
            || self.api_key.master_key.is_some()
            || self.trusted_headers.is_configured()
    }

    /// Returns an error message if more than one authentication mode is configured.
    pub fn validate_exclusive(&self) -> Result<(), String> {
        let count = [
            self.oidc.is_configured(),
            self.api_key.master_key.is_some(),
            self.trusted_headers.is_configured(),
        ]
        .iter()
        .filter(|&&v| v)
        .count();

        if count > 1 {
            Err(
                "only one authentication mode may be configured at a time \
                 (api_key, oidc, or trusted_headers)"
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }
}

// --- Project / User ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub name: Option<String>,
}

// --- Log config ---

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

// --- Backend config ---

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
    pub rds_secrets_arn: Option<String>,
    pub sslrootcert: Option<String>,
    pub max_connections: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SqliteConfig {
    pub db_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendConfig {
    #[serde(default)]
    pub sqlite: SqliteConfig,
    #[cfg(feature = "dynamodb")]
    #[serde(default)]
    pub dynamodb: Option<DynamoDbConfig>,
    #[cfg(feature = "postgres")]
    #[serde(default)]
    pub postgres: Option<PostgresConfig>,
}

// --- Workflow config ---

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
    pub branch_template: Option<String>,
    #[serde(default)]
    pub add: WorkflowAddConfig,
    #[serde(default)]
    pub start: WorkflowStartConfig,
    #[serde(default)]
    pub branch: WorkflowEventConfig,
    #[serde(default)]
    pub plan: WorkflowPlanConfig,
    #[serde(default)]
    pub implement: WorkflowEventConfig,
    #[serde(rename = "merge", default)]
    pub merge_event: WorkflowEventConfig,
    #[serde(default)]
    pub pr: WorkflowEventConfig,
    #[serde(default)]
    pub complete: WorkflowCompleteConfig,
    #[serde(default)]
    pub branch_cleanup: WorkflowEventConfig,
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
            branch_template: None,
            add: WorkflowAddConfig::default(),
            start: WorkflowStartConfig::default(),
            branch: WorkflowEventConfig::default(),
            plan: WorkflowPlanConfig::default(),
            implement: WorkflowEventConfig::default(),
            merge_event: WorkflowEventConfig::default(),
            pr: WorkflowEventConfig::default(),
            complete: WorkflowCompleteConfig::default(),
            branch_cleanup: WorkflowEventConfig::default(),
        }
    }
}

// --- Named hook types (CLI hooks, unchanged) ---

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
    pub server: RawServerConfig,
    #[serde(default)]
    pub cli: RawCliConfig,
    #[serde(default)]
    pub web: RawWebConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawWorkflowConfig {
    #[serde(alias = "completion_mode")]
    pub merge_via: Option<MergeVia>,
    pub auto_merge: Option<bool>,
    pub branch_mode: Option<BranchMode>,
    pub merge_strategy: Option<MergeStrategy>,
    pub branch_template: Option<String>,
    #[serde(default)]
    pub add: Option<WorkflowAddConfig>,
    #[serde(default)]
    pub start: Option<WorkflowStartConfig>,
    #[serde(default)]
    pub branch: Option<WorkflowEventConfig>,
    #[serde(default)]
    pub plan: Option<WorkflowPlanConfig>,
    #[serde(default)]
    pub implement: Option<WorkflowEventConfig>,
    #[serde(rename = "merge", default)]
    pub merge_event: Option<WorkflowEventConfig>,
    #[serde(default)]
    pub pr: Option<WorkflowEventConfig>,
    #[serde(default)]
    pub complete: Option<WorkflowCompleteConfig>,
    #[serde(default)]
    pub branch_cleanup: Option<WorkflowEventConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawBackendConfig {
    #[serde(default)]
    pub sqlite: RawSqliteConfig,
    #[cfg(feature = "dynamodb")]
    pub dynamodb: Option<DynamoDbConfig>,
    #[cfg(feature = "postgres")]
    pub postgres: Option<PostgresConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawSqliteConfig {
    pub db_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawLogConfig {
    pub dir: Option<String>,
    pub level: Option<String>,
    pub format: Option<LogFormat>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawServerRelayConfig {
    pub url: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub relay: RawServerRelayConfig,
    #[serde(default)]
    pub auth: RawAuthConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawCliConfig {
    pub browser: Option<bool>,
    #[serde(default)]
    pub remote: RawCliRemoteConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawCliRemoteConfig {
    pub url: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawWebConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawOidcConfig {
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub username_claim: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub required_claims: Option<HashMap<String, String>>,
    pub callback_ports: Option<Vec<String>>,
    #[serde(default)]
    pub session: RawSessionConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawSessionConfig {
    pub ttl: Option<String>,
    pub inactive_ttl: Option<String>,
    pub max_per_user: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawApiKeyConfig {
    pub master_key: Option<String>,
    pub master_key_arn: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawTrustedHeadersConfig {
    pub subject_header: Option<String>,
    pub name_header: Option<String>,
    pub display_name_header: Option<String>,
    pub email_header: Option<String>,
    pub groups_header: Option<String>,
    pub scope_header: Option<String>,
    pub oidc_issuer_url: Option<String>,
    pub oidc_client_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawAuthConfig {
    #[serde(default)]
    pub api_key: RawApiKeyConfig,
    #[serde(default)]
    pub oidc: RawOidcConfig,
    #[serde(default)]
    pub trusted_headers: RawTrustedHeadersConfig,
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
                merge_strategy: overlay
                    .workflow
                    .merge_strategy
                    .or(self.workflow.merge_strategy),
                branch_template: overlay
                    .workflow
                    .branch_template
                    .or(self.workflow.branch_template),
                add: overlay.workflow.add.or(self.workflow.add),
                start: overlay.workflow.start.or(self.workflow.start),
                branch: overlay.workflow.branch.or(self.workflow.branch),
                plan: overlay.workflow.plan.or(self.workflow.plan),
                implement: overlay.workflow.implement.or(self.workflow.implement),
                merge_event: overlay.workflow.merge_event.or(self.workflow.merge_event),
                pr: overlay.workflow.pr.or(self.workflow.pr),
                complete: overlay.workflow.complete.or(self.workflow.complete),
                branch_cleanup: overlay
                    .workflow
                    .branch_cleanup
                    .or(self.workflow.branch_cleanup),
            },
            backend: RawBackendConfig {
                sqlite: RawSqliteConfig {
                    db_path: overlay
                        .backend
                        .sqlite
                        .db_path
                        .or(self.backend.sqlite.db_path),
                },
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
            server: RawServerConfig {
                host: overlay.server.host.or(self.server.host),
                port: overlay.server.port.or(self.server.port),
                relay: RawServerRelayConfig {
                    url: overlay.server.relay.url.or(self.server.relay.url),
                    token: overlay.server.relay.token.or(self.server.relay.token),
                },
                auth: RawAuthConfig {
                    api_key: RawApiKeyConfig {
                        master_key: overlay
                            .server
                            .auth
                            .api_key
                            .master_key
                            .or(self.server.auth.api_key.master_key),
                        master_key_arn: overlay
                            .server
                            .auth
                            .api_key
                            .master_key_arn
                            .or(self.server.auth.api_key.master_key_arn),
                    },
                    trusted_headers: RawTrustedHeadersConfig {
                        subject_header: overlay
                            .server
                            .auth
                            .trusted_headers
                            .subject_header
                            .or(self.server.auth.trusted_headers.subject_header),
                        name_header: overlay
                            .server
                            .auth
                            .trusted_headers
                            .name_header
                            .or(self.server.auth.trusted_headers.name_header),
                        display_name_header: overlay
                            .server
                            .auth
                            .trusted_headers
                            .display_name_header
                            .or(self.server.auth.trusted_headers.display_name_header),
                        email_header: overlay
                            .server
                            .auth
                            .trusted_headers
                            .email_header
                            .or(self.server.auth.trusted_headers.email_header),
                        groups_header: overlay
                            .server
                            .auth
                            .trusted_headers
                            .groups_header
                            .or(self.server.auth.trusted_headers.groups_header),
                        scope_header: overlay
                            .server
                            .auth
                            .trusted_headers
                            .scope_header
                            .or(self.server.auth.trusted_headers.scope_header),
                        oidc_issuer_url: overlay
                            .server
                            .auth
                            .trusted_headers
                            .oidc_issuer_url
                            .or(self.server.auth.trusted_headers.oidc_issuer_url),
                        oidc_client_id: overlay
                            .server
                            .auth
                            .trusted_headers
                            .oidc_client_id
                            .or(self.server.auth.trusted_headers.oidc_client_id),
                    },
                    oidc: RawOidcConfig {
                        issuer_url: overlay
                            .server
                            .auth
                            .oidc
                            .issuer_url
                            .or(self.server.auth.oidc.issuer_url),
                        client_id: overlay
                            .server
                            .auth
                            .oidc
                            .client_id
                            .or(self.server.auth.oidc.client_id),
                        username_claim: overlay
                            .server
                            .auth
                            .oidc
                            .username_claim
                            .or(self.server.auth.oidc.username_claim),
                        scopes: overlay
                            .server
                            .auth
                            .oidc
                            .scopes
                            .or(self.server.auth.oidc.scopes),
                        required_claims: overlay
                            .server
                            .auth
                            .oidc
                            .required_claims
                            .or(self.server.auth.oidc.required_claims),
                        callback_ports: overlay
                            .server
                            .auth
                            .oidc
                            .callback_ports
                            .or(self.server.auth.oidc.callback_ports),
                        session: RawSessionConfig {
                            ttl: overlay
                                .server
                                .auth
                                .oidc
                                .session
                                .ttl
                                .or(self.server.auth.oidc.session.ttl),
                            inactive_ttl: overlay
                                .server
                                .auth
                                .oidc
                                .session
                                .inactive_ttl
                                .or(self.server.auth.oidc.session.inactive_ttl),
                            max_per_user: overlay
                                .server
                                .auth
                                .oidc
                                .session
                                .max_per_user
                                .or(self.server.auth.oidc.session.max_per_user),
                        },
                    },
                },
            },
            cli: RawCliConfig {
                browser: overlay.cli.browser.or(self.cli.browser),
                remote: RawCliRemoteConfig {
                    url: overlay.cli.remote.url.or(self.cli.remote.url),
                    token: overlay.cli.remote.token.or(self.cli.remote.token),
                },
            },
            web: RawWebConfig {
                host: overlay.web.host.or(self.web.host),
                port: overlay.web.port.or(self.web.port),
            },
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
                branch_template: self.workflow.branch_template,
                add: self.workflow.add.unwrap_or_default(),
                start: self.workflow.start.unwrap_or_default(),
                branch: self.workflow.branch.unwrap_or_default(),
                plan: self.workflow.plan.unwrap_or_default(),
                implement: self.workflow.implement.unwrap_or_default(),
                merge_event: self.workflow.merge_event.unwrap_or_default(),
                pr: self.workflow.pr.unwrap_or_default(),
                complete: self.workflow.complete.unwrap_or_default(),
                branch_cleanup: self.workflow.branch_cleanup.unwrap_or_default(),
            },
            backend: BackendConfig {
                sqlite: SqliteConfig {
                    db_path: self.backend.sqlite.db_path,
                },
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
            server: ServerConfig {
                host: self.server.host,
                port: self.server.port,
                relay: ServerRelayConfig {
                    url: self.server.relay.url,
                    token: self.server.relay.token,
                },
                auth: AuthConfig {
                    api_key: ApiKeyConfig {
                        master_key: self.server.auth.api_key.master_key,
                        master_key_arn: self.server.auth.api_key.master_key_arn,
                    },
                    oidc: OidcConfig {
                        issuer_url: self.server.auth.oidc.issuer_url,
                        client_id: self.server.auth.oidc.client_id,
                        username_claim: self.server.auth.oidc.username_claim,
                        scopes: self
                            .server
                            .auth
                            .oidc
                            .scopes
                            .unwrap_or_else(default_oidc_scopes),
                        required_claims: self
                            .server
                            .auth
                            .oidc
                            .required_claims
                            .unwrap_or_default(),
                        callback_ports: self
                            .server
                            .auth
                            .oidc
                            .callback_ports
                            .unwrap_or_default(),
                        session: SessionConfig {
                            ttl: self.server.auth.oidc.session.ttl,
                            inactive_ttl: self.server.auth.oidc.session.inactive_ttl,
                            max_per_user: self.server.auth.oidc.session.max_per_user,
                        },
                    },
                    trusted_headers: TrustedHeadersConfig {
                        subject_header: self.server.auth.trusted_headers.subject_header,
                        name_header: self.server.auth.trusted_headers.name_header,
                        display_name_header: self.server.auth.trusted_headers.display_name_header,
                        email_header: self.server.auth.trusted_headers.email_header,
                        groups_header: self.server.auth.trusted_headers.groups_header,
                        scope_header: self.server.auth.trusted_headers.scope_header,
                        oidc_issuer_url: self.server.auth.trusted_headers.oidc_issuer_url,
                        oidc_client_id: self.server.auth.trusted_headers.oidc_client_id,
                    },
                },
            },
            cli: CliConfig {
                browser: self.cli.browser.unwrap_or(true),
                remote: CliRemoteConfig {
                    url: self.cli.remote.url,
                    token: self.cli.remote.token,
                },
            },
            web: WebConfig {
                host: self.web.host,
                port: self.web.port,
            },
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
    pub server_port: Option<u16>,
    pub server_host: Option<String>,
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

        // Server relay settings
        if let Ok(val) = std::env::var("SENKO_SERVER_RELAY_URL")
            && !val.is_empty() {
                self.server.relay.url = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_SERVER_RELAY_TOKEN")
            && !val.is_empty() {
                self.server.relay.token = Some(val);
            }

        // CLI remote settings
        if let Ok(val) = std::env::var("SENKO_CLI_REMOTE_URL")
            && !val.is_empty() {
                self.cli.remote.url = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_CLI_REMOTE_TOKEN")
            && !val.is_empty() {
                self.cli.remote.token = Some(val);
            }

        // CLI hooks
        if let Ok(val) = std::env::var("SENKO_HOOKS_ENABLED") {
            match val.to_lowercase().as_str() {
                "true" | "1" => self.hooks.enabled = true,
                "false" | "0" => self.hooks.enabled = false,
                other => eprintln!("warning: unknown SENKO_HOOKS_ENABLED={other}, ignoring"),
            }
        }

        // Server auth settings
        if let Ok(val) = std::env::var("SENKO_AUTH_API_KEY_MASTER_KEY")
            && !val.is_empty() {
                self.server.auth.api_key.master_key = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_API_KEY_MASTER_KEY_ARN")
            && !val.is_empty() {
                self.server.auth.api_key.master_key_arn = Some(val);
            }

        // Server OIDC settings
        if let Ok(val) = std::env::var("SENKO_OIDC_ISSUER_URL")
            && !val.is_empty() {
                self.server.auth.oidc.issuer_url = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_OIDC_CLIENT_ID")
            && !val.is_empty() {
                self.server.auth.oidc.client_id = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_OIDC_USERNAME_CLAIM")
            && !val.is_empty() {
                self.server.auth.oidc.username_claim = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_OIDC_CALLBACK_PORTS")
            && !val.is_empty() {
                self.server.auth.oidc.callback_ports = val
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }

        // Server trusted headers settings
        if let Ok(val) = std::env::var("SENKO_AUTH_TRUSTED_HEADERS_SUBJECT_HEADER")
            && !val.is_empty() {
                self.server.auth.trusted_headers.subject_header = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_TRUSTED_HEADERS_NAME_HEADER")
            && !val.is_empty() {
                self.server.auth.trusted_headers.name_header = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_TRUSTED_HEADERS_EMAIL_HEADER")
            && !val.is_empty() {
                self.server.auth.trusted_headers.email_header = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_TRUSTED_HEADERS_GROUPS_HEADER")
            && !val.is_empty() {
                self.server.auth.trusted_headers.groups_header = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_TRUSTED_HEADERS_SCOPE_HEADER")
            && !val.is_empty() {
                self.server.auth.trusted_headers.scope_header = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_TRUSTED_HEADERS_OIDC_ISSUER_URL")
            && !val.is_empty() {
                self.server.auth.trusted_headers.oidc_issuer_url = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_TRUSTED_HEADERS_OIDC_CLIENT_ID")
            && !val.is_empty() {
                self.server.auth.trusted_headers.oidc_client_id = Some(val);
            }

        // Server OIDC session settings
        if let Ok(val) = std::env::var("SENKO_AUTH_OIDC_SESSION_TTL")
            && !val.is_empty() {
                self.server.auth.oidc.session.ttl = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_OIDC_SESSION_INACTIVE_TTL")
            && !val.is_empty() {
                self.server.auth.oidc.session.inactive_ttl = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_AUTH_OIDC_SESSION_MAX_PER_USER")
            && !val.is_empty()
            && let Ok(n) = val.parse::<u32>() {
                self.server.auth.oidc.session.max_per_user = Some(n);
            }

        // Server host/port
        if let Ok(val) = std::env::var("SENKO_SERVER_HOST")
            && !val.is_empty() {
                self.server.host = Some(val);
            }
        if let Ok(val) = std::env::var("SENKO_SERVER_PORT")
            && let Ok(port) = val.parse::<u16>() {
                self.server.port = Some(port);
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
            if let Ok(val) = std::env::var("SENKO_POSTGRES_RDS_SECRETS_ARN") {
                if !val.is_empty() {
                    self.backend
                        .postgres
                        .get_or_insert_with(PostgresConfig::default)
                        .rds_secrets_arn = Some(val);
                }
            }
            if let Ok(val) = std::env::var("SENKO_POSTGRES_SSLROOTCERT") {
                if !val.is_empty() {
                    self.backend
                        .postgres
                        .get_or_insert_with(PostgresConfig::default)
                        .sslrootcert = Some(val);
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

        // Backend SQLite settings
        if let Ok(val) = std::env::var("SENKO_DB_PATH")
            && !val.is_empty() {
                self.backend.sqlite.db_path = Some(val);
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

        // Server settings (same env vars apply to both web and server)
        if let Ok(val) = std::env::var("SENKO_PORT")
            && let Ok(port) = val.parse::<u16>() {
                self.server.port = Some(port);
            }
        if let Ok(val) = std::env::var("SENKO_HOST")
            && !val.is_empty() {
                self.server.host = Some(val);
            }
    }

    /// Apply CLI argument overrides. Call after `apply_env()`.
    /// Priority: CLI > env > config.toml > defaults.
    pub fn apply_cli(&mut self, overrides: &CliOverrides) {
        if let Some(ref dir) = overrides.log_dir {
            self.log.dir = Some(dir.clone());
        }
        if let Some(ref path) = overrides.db_path {
            self.backend.sqlite.db_path = Some(path.clone());
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
        if let Some(port) = overrides.server_port {
            self.server.port = Some(port);
        }
        if let Some(ref host) = overrides.server_host {
            self.server.host = Some(host.clone());
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

    pub fn server_port_or(&self, default: u16) -> u16 {
        self.server.port.unwrap_or(default)
    }

    pub fn server_port_is_explicit(&self) -> bool {
        self.server.port.is_some()
    }

    pub fn effective_server_host(&self) -> String {
        self.server
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
        use anyhow::Context;

        if let Some(ref arn) = self.server.auth.api_key.master_key_arn {
            let secret = client.get_secret(arn).await?;
            self.server.auth.api_key.master_key = Some(secret);
        }

        #[cfg(feature = "postgres")]
        {
            let pg = self.backend.postgres.as_ref();
            let rds_arn = pg.and_then(|p| p.rds_secrets_arn.clone());
            let url_arn = pg.and_then(|p| p.url_arn.clone());
            let sslrootcert = pg.and_then(|p| p.sslrootcert.clone());

            if let Some(ref arn) = rds_arn {
                let secret = client.get_secret(arn).await?;
                let url = build_rds_url(&secret, sslrootcert.as_deref())
                    .with_context(|| format!("failed to parse RDS JSON secret from {arn}"))?;
                self.backend
                    .postgres
                    .get_or_insert_with(PostgresConfig::default)
                    .url = Some(url);
            } else if let Some(ref arn) = url_arn {
                let secret = client.get_secret(arn).await?;
                self.backend
                    .postgres
                    .get_or_insert_with(PostgresConfig::default)
                    .url = Some(secret);
            }
        }

        Ok(())
    }
}

#[cfg(all(feature = "postgres", feature = "aws-secrets"))]
fn build_rds_url(json_str: &str, sslrootcert: Option<&str>) -> anyhow::Result<String> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

    #[derive(serde::Deserialize)]
    struct RdsSecret {
        username: String,
        password: String,
        host: String,
        port: Option<u16>,
        dbname: Option<String>,
    }

    let secret: RdsSecret = serde_json::from_str(json_str)?;
    let port = secret.port.unwrap_or(5432);
    let dbname = secret.dbname.as_deref().unwrap_or("postgres");
    let encoded_user = utf8_percent_encode(&secret.username, NON_ALPHANUMERIC);
    let encoded_pass = utf8_percent_encode(&secret.password, NON_ALPHANUMERIC);

    let mut url = format!(
        "postgres://{encoded_user}:{encoded_pass}@{}:{port}/{dbname}",
        secret.host
    );

    if let Some(cert_path) = sslrootcert {
        url.push_str("?sslmode=verify-full&sslrootcert=");
        url.push_str(cert_path);
    }

    Ok(url)
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
        config.server.auth.api_key.master_key = Some("direct-value".to_string());
        config.server.auth.api_key.master_key_arn =
            Some("arn:aws:secretsmanager:us-east-1:123:secret:key".to_string());

        let client = make_client(HashMap::from([(
            "arn:aws:secretsmanager:us-east-1:123:secret:key".to_string(),
            "arn-resolved-value".to_string(),
        )]));

        config.resolve_secrets_with(&client).await.unwrap();

        assert_eq!(
            config.server.auth.api_key.master_key.as_deref(),
            Some("arn-resolved-value")
        );
    }

    #[tokio::test]
    async fn arn_only_sets_master_key() {
        let mut config = Config::default();
        config.server.auth.api_key.master_key_arn =
            Some("arn:aws:secretsmanager:us-east-1:123:secret:key".to_string());

        let client = make_client(HashMap::from([(
            "arn:aws:secretsmanager:us-east-1:123:secret:key".to_string(),
            "arn-resolved-value".to_string(),
        )]));

        config.resolve_secrets_with(&client).await.unwrap();

        assert_eq!(
            config.server.auth.api_key.master_key.as_deref(),
            Some("arn-resolved-value")
        );
    }

    #[tokio::test]
    async fn direct_only_unchanged() {
        let mut config = Config::default();
        config.server.auth.api_key.master_key = Some("direct-value".to_string());

        let client = make_client(HashMap::new());

        config.resolve_secrets_with(&client).await.unwrap();

        assert_eq!(
            config.server.auth.api_key.master_key.as_deref(),
            Some("direct-value")
        );
    }

    #[tokio::test]
    async fn neither_set_remains_none() {
        let mut config = Config::default();

        let client = make_client(HashMap::new());

        config.resolve_secrets_with(&client).await.unwrap();

        assert!(config.server.auth.api_key.master_key.is_none());
    }

    #[cfg(feature = "postgres")]
    mod postgres_secrets {
        use super::*;

        fn pg_config(config: &mut Config) -> &mut PostgresConfig {
            config
                .backend
                .postgres
                .get_or_insert_with(PostgresConfig::default)
        }

        #[tokio::test]
        async fn rds_secrets_arn_builds_url_from_json() {
            let rds_json = r#"{"username":"admin","password":"s3cret","host":"mydb.cluster-abc.us-east-1.rds.amazonaws.com","port":5432,"dbname":"myapp"}"#;
            let mut config = Config::default();
            pg_config(&mut config).rds_secrets_arn = Some("arn:rds".to_string());

            let client = make_client(HashMap::from([
                ("arn:rds".to_string(), rds_json.to_string()),
            ]));

            config.resolve_secrets_with(&client).await.unwrap();

            assert_eq!(
                config.backend.postgres.as_ref().unwrap().url.as_deref(),
                Some("postgres://admin:s3cret@mydb.cluster-abc.us-east-1.rds.amazonaws.com:5432/myapp")
            );
        }

        #[tokio::test]
        async fn rds_secrets_arn_defaults_port_and_dbname() {
            let rds_json = r#"{"username":"admin","password":"pass","host":"db.example.com"}"#;
            let mut config = Config::default();
            pg_config(&mut config).rds_secrets_arn = Some("arn:rds".to_string());

            let client = make_client(HashMap::from([
                ("arn:rds".to_string(), rds_json.to_string()),
            ]));

            config.resolve_secrets_with(&client).await.unwrap();

            assert_eq!(
                config.backend.postgres.as_ref().unwrap().url.as_deref(),
                Some("postgres://admin:pass@db.example.com:5432/postgres")
            );
        }

        #[tokio::test]
        async fn rds_secrets_arn_takes_priority_over_url_arn() {
            let rds_json =
                r#"{"username":"u","password":"p","host":"rds.example.com"}"#;
            let mut config = Config::default();
            let pg = pg_config(&mut config);
            pg.rds_secrets_arn = Some("arn:rds".to_string());
            pg.url_arn = Some("arn:url".to_string());

            let client = make_client(HashMap::from([
                ("arn:rds".to_string(), rds_json.to_string()),
                (
                    "arn:url".to_string(),
                    "postgres://from-url-arn/db".to_string(),
                ),
            ]));

            config.resolve_secrets_with(&client).await.unwrap();

            let url = config
                .backend
                .postgres
                .as_ref()
                .unwrap()
                .url
                .as_deref()
                .unwrap();
            assert!(
                url.contains("rds.example.com"),
                "should use RDS secret, got: {url}"
            );
        }

        #[tokio::test]
        async fn url_arn_still_works_without_rds_arn() {
            let mut config = Config::default();
            pg_config(&mut config).url_arn = Some("arn:url".to_string());

            let client = make_client(HashMap::from([(
                "arn:url".to_string(),
                "postgres://direct-url/db".to_string(),
            )]));

            config.resolve_secrets_with(&client).await.unwrap();

            assert_eq!(
                config.backend.postgres.as_ref().unwrap().url.as_deref(),
                Some("postgres://direct-url/db")
            );
        }

        #[tokio::test]
        async fn sslrootcert_appended_to_rds_url() {
            let rds_json = r#"{"username":"u","password":"p","host":"db.example.com","port":5432,"dbname":"app"}"#;
            let mut config = Config::default();
            let pg = pg_config(&mut config);
            pg.rds_secrets_arn = Some("arn:rds".to_string());
            pg.sslrootcert = Some("/etc/ssl/rds-ca.pem".to_string());

            let client = make_client(HashMap::from([
                ("arn:rds".to_string(), rds_json.to_string()),
            ]));

            config.resolve_secrets_with(&client).await.unwrap();

            assert_eq!(
                config.backend.postgres.as_ref().unwrap().url.as_deref(),
                Some("postgres://u:p@db.example.com:5432/app?sslmode=verify-full&sslrootcert=/etc/ssl/rds-ca.pem")
            );
        }

        #[tokio::test]
        async fn rds_json_parse_error_has_clear_message() {
            let mut config = Config::default();
            pg_config(&mut config).rds_secrets_arn = Some("arn:bad".to_string());

            let client = make_client(HashMap::from([(
                "arn:bad".to_string(),
                "not-valid-json".to_string(),
            )]));

            let err = config.resolve_secrets_with(&client).await.unwrap_err();
            let msg = format!("{err:#}");
            assert!(
                msg.contains("failed to parse RDS JSON secret from arn:bad"),
                "error: {msg}"
            );
        }

        #[tokio::test]
        async fn rds_password_with_special_chars_is_encoded() {
            let rds_json =
                r#"{"username":"admin","password":"p@ss:w/rd?#","host":"db.example.com"}"#;
            let mut config = Config::default();
            pg_config(&mut config).rds_secrets_arn = Some("arn:rds".to_string());

            let client = make_client(HashMap::from([
                ("arn:rds".to_string(), rds_json.to_string()),
            ]));

            config.resolve_secrets_with(&client).await.unwrap();

            let url = config
                .backend
                .postgres
                .as_ref()
                .unwrap()
                .url
                .as_deref()
                .unwrap();
            // Password should be percent-encoded
            assert!(
                !url.contains("p@ss:w/rd?#"),
                "password should be encoded, got: {url}"
            );
            assert!(
                url.contains("p%40ss%3Aw%2Frd%3F%23"),
                "expected encoded password, got: {url}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hookdef_simple_string() {
        let json = r#""echo hello""#;
        let hook: HookDef = serde_json::from_str(json).unwrap();
        match hook {
            HookDef::Simple(s) => assert_eq!(s, "echo hello"),
            HookDef::Complex(_) => panic!("expected Simple"),
        }
    }

    #[test]
    fn hookdef_complex_object() {
        let json = r#"{"command": "cargo test", "on_failure": "warn"}"#;
        let hook: HookDef = serde_json::from_str(json).unwrap();
        match hook {
            HookDef::Complex(h) => {
                assert_eq!(h.command.as_deref(), Some("cargo test"));
                assert_eq!(h.on_failure, OnFailure::Warn);
                assert!(h.prompt.is_none());
            }
            HookDef::Simple(_) => panic!("expected Complex"),
        }
    }

    #[test]
    fn hookdef_prompt_only() {
        let json = r#"{"prompt": "Review the code", "on_failure": "ignore"}"#;
        let hook: HookDef = serde_json::from_str(json).unwrap();
        match hook {
            HookDef::Complex(h) => {
                assert!(h.command.is_none());
                assert_eq!(h.prompt.as_deref(), Some("Review the code"));
                assert_eq!(h.on_failure, OnFailure::Ignore);
            }
            HookDef::Simple(_) => panic!("expected Complex"),
        }
    }

    #[test]
    fn on_failure_variants() {
        assert_eq!(
            serde_json::from_str::<OnFailure>(r#""abort""#).unwrap(),
            OnFailure::Abort
        );
        assert_eq!(
            serde_json::from_str::<OnFailure>(r#""warn""#).unwrap(),
            OnFailure::Warn
        );
        assert_eq!(
            serde_json::from_str::<OnFailure>(r#""ignore""#).unwrap(),
            OnFailure::Ignore
        );
    }

    #[test]
    fn metadata_field_source_env() {
        let json = r#"{"key": "sprint", "source": "env", "env_var": "SPRINT"}"#;
        let field: MetadataField = serde_json::from_str(json).unwrap();
        assert_eq!(field.key, "sprint");
        assert!(!field.required);
        assert!(field.default.is_none());
        match field.source {
            MetadataFieldSource::Env { env_var } => assert_eq!(env_var, "SPRINT"),
            _ => panic!("expected Env"),
        }
    }

    #[test]
    fn metadata_field_source_value() {
        let json = r#"{"key": "team", "source": "value", "value": "backend"}"#;
        let field: MetadataField = serde_json::from_str(json).unwrap();
        match field.source {
            MetadataFieldSource::Value { value } => assert_eq!(value, "backend"),
            _ => panic!("expected Value"),
        }
    }

    #[test]
    fn metadata_field_source_command() {
        let json = r#"{"key": "version", "source": "command", "command": "git describe --tags"}"#;
        let field: MetadataField = serde_json::from_str(json).unwrap();
        match field.source {
            MetadataFieldSource::Command { command } => {
                assert_eq!(command, "git describe --tags");
            }
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn metadata_field_source_prompt() {
        let json = r#"{"key": "priority", "source": "prompt", "prompt": "What priority?"}"#;
        let field: MetadataField = serde_json::from_str(json).unwrap();
        match field.source {
            MetadataFieldSource::Prompt { prompt } => assert_eq!(prompt, "What priority?"),
            _ => panic!("expected Prompt"),
        }
    }

    #[test]
    fn metadata_field_default_and_required() {
        let json = r#"{"key": "sprint", "source": "env", "env_var": "SPRINT", "default": "Q1", "required": true}"#;
        let field: MetadataField = serde_json::from_str(json).unwrap();
        assert_eq!(field.key, "sprint");
        assert!(field.required);
        assert_eq!(field.default, Some(serde_json::Value::String("Q1".to_string())));
    }

    #[test]
    fn workflow_add_config_deser() {
        let toml_str = r#"
            default_dod = ["Write tests", "Update docs"]
            default_tags = ["backend"]
            default_priority = "p1"
            instructions = ["Be thorough"]

            [[metadata_fields]]
            key = "team"
            source = "value"
            value = "backend"
        "#;
        let config: WorkflowAddConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default_dod, vec!["Write tests", "Update docs"]);
        assert_eq!(config.default_tags, vec!["backend"]);
        assert_eq!(config.default_priority.as_deref(), Some("p1"));
        assert_eq!(config.instructions, vec!["Be thorough"]);
        assert!(config.pre_hooks.is_empty());
        assert_eq!(config.metadata_fields.len(), 1);
        assert_eq!(config.metadata_fields[0].key, "team");
    }

    #[test]
    fn workflow_start_config_deser() {
        let json = r#"{
            "metadata_fields": [
                {"key": "sprint", "source": "env", "env_var": "SPRINT"}
            ],
            "instructions": ["Check prerequisites"]
        }"#;
        let config: WorkflowStartConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.metadata_fields.len(), 1);
        assert_eq!(config.instructions, vec!["Check prerequisites"]);
    }

    #[test]
    fn workflow_plan_config_deser() {
        let toml_str = r#"
            required_sections = ["Context", "Verification"]
            instructions = ["Include diagrams"]

            [[metadata_fields]]
            key = "estimate"
            source = "prompt"
            prompt = "Estimated time?"
        "#;
        let config: WorkflowPlanConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.required_sections,
            vec!["Context", "Verification"]
        );
        assert_eq!(config.metadata_fields.len(), 1);
        assert_eq!(config.metadata_fields[0].key, "estimate");
    }

    #[test]
    fn workflow_complete_config_deser() {
        let json = r#"{
            "metadata_fields": [
                {"key": "review_notes", "source": "prompt", "prompt": "Any review notes?"}
            ],
            "instructions": ["Verify all tests pass"]
        }"#;
        let config: WorkflowCompleteConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.metadata_fields.len(), 1);
        assert_eq!(config.instructions, vec!["Verify all tests pass"]);
    }

    #[test]
    fn workflow_config_full_toml() {
        let toml_str = r#"
            merge_via = "direct"
            auto_merge = true
            branch_mode = "worktree"
            merge_strategy = "rebase"
            branch_template = "feat/{id}-{slug}"

            [add]
            default_dod = ["Tests"]
            default_tags = ["backend"]

            [start]
            instructions = ["Review task"]

            [plan]
            required_sections = ["Context"]

            [complete]
            instructions = ["Run tests"]
        "#;
        let config: WorkflowConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.merge_via, MergeVia::Direct);
        assert!(config.auto_merge);
        assert_eq!(config.branch_mode, BranchMode::Worktree);
        assert_eq!(config.merge_strategy, MergeStrategy::Rebase);
        assert_eq!(
            config.branch_template.as_deref(),
            Some("feat/{id}-{slug}")
        );
        assert_eq!(config.add.default_dod, vec!["Tests"]);
        assert_eq!(config.start.instructions, vec!["Review task"]);
        assert_eq!(config.plan.required_sections, vec!["Context"]);
        assert_eq!(config.complete.instructions, vec!["Run tests"]);
    }

    #[test]
    fn mixed_hooks_in_event() {
        let json = r#"{
            "pre_hooks": ["echo start", {"command": "cargo check", "on_failure": "abort"}],
            "post_hooks": [{"prompt": "Verify output", "on_failure": "warn"}]
        }"#;
        let config: WorkflowEventConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.pre_hooks.len(), 2);
        match &config.pre_hooks[0] {
            HookDef::Simple(s) => assert_eq!(s, "echo start"),
            HookDef::Complex(_) => panic!("expected Simple"),
        }
        match &config.pre_hooks[1] {
            HookDef::Complex(h) => {
                assert_eq!(h.command.as_deref(), Some("cargo check"));
                assert_eq!(h.on_failure, OnFailure::Abort);
            }
            HookDef::Simple(_) => panic!("expected Complex"),
        }
        assert_eq!(config.post_hooks.len(), 1);
    }

    #[test]
    fn workflow_config_full_toml_all_stages() {
        let toml_str = r#"
            merge_via = "pr"
            auto_merge = false

            [add]
            default_dod = ["Tests"]

            [start]
            instructions = ["Check prereqs"]

            [branch]
            instructions = ["Create feature branch"]
            pre_hooks = ["echo branching"]

            [[branch.metadata_fields]]
            key = "branch_name"
            source = "command"
            command = "git rev-parse --abbrev-ref HEAD"

            [plan]
            required_sections = ["Context"]

            [[plan.metadata_fields]]
            key = "estimate"
            source = "prompt"
            prompt = "Estimated time?"

            [implement]
            instructions = ["Follow style guide"]
            pre_hooks = ["cargo fmt --check"]
            post_hooks = ["cargo test"]

            [[implement.metadata_fields]]
            key = "complexity"
            source = "value"
            value = "medium"

            [merge]
            instructions = ["Squash commits"]
            pre_hooks = ["echo pre-merge"]

            [pr]
            instructions = ["Add reviewers"]
            post_hooks = ["echo pr-created"]

            [complete]
            instructions = ["Run tests"]

            [branch_cleanup]
            instructions = ["Delete remote branch"]
            post_hooks = ["git remote prune origin"]
        "#;
        let config: WorkflowConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.merge_via, MergeVia::Pr);
        assert!(!config.auto_merge);
        assert_eq!(config.add.default_dod, vec!["Tests"]);
        assert_eq!(config.start.instructions, vec!["Check prereqs"]);
        assert_eq!(config.branch.instructions, vec!["Create feature branch"]);
        assert_eq!(config.branch.pre_hooks.len(), 1);
        assert_eq!(config.branch.metadata_fields.len(), 1);
        assert_eq!(config.branch.metadata_fields[0].key, "branch_name");
        assert_eq!(config.plan.required_sections, vec!["Context"]);
        assert_eq!(config.plan.metadata_fields.len(), 1);
        assert_eq!(config.plan.metadata_fields[0].key, "estimate");
        assert_eq!(config.implement.instructions, vec!["Follow style guide"]);
        assert_eq!(config.implement.pre_hooks.len(), 1);
        assert_eq!(config.implement.post_hooks.len(), 1);
        assert_eq!(config.implement.metadata_fields.len(), 1);
        assert_eq!(config.implement.metadata_fields[0].key, "complexity");
        assert_eq!(config.merge_event.instructions, vec!["Squash commits"]);
        assert_eq!(config.merge_event.pre_hooks.len(), 1);
        assert_eq!(config.pr.instructions, vec!["Add reviewers"]);
        assert_eq!(config.pr.post_hooks.len(), 1);
        assert_eq!(config.complete.instructions, vec!["Run tests"]);
        assert_eq!(
            config.branch_cleanup.instructions,
            vec!["Delete remote branch"]
        );
        assert_eq!(config.branch_cleanup.post_hooks.len(), 1);
    }

    #[test]
    fn workflow_event_config_toml_with_mixed_hooks() {
        let toml_str = r#"
            instructions = ["Review carefully"]

            [[pre_hooks]]
            command = "cargo check"
            on_failure = "abort"

            [[post_hooks]]
            prompt = "Verify output"
            on_failure = "warn"
        "#;
        let config: WorkflowEventConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.instructions, vec!["Review carefully"]);
        assert_eq!(config.pre_hooks.len(), 1);
        match &config.pre_hooks[0] {
            HookDef::Complex(h) => {
                assert_eq!(h.command.as_deref(), Some("cargo check"));
                assert_eq!(h.on_failure, OnFailure::Abort);
            }
            HookDef::Simple(_) => panic!("expected Complex"),
        }
        assert_eq!(config.post_hooks.len(), 1);
        match &config.post_hooks[0] {
            HookDef::Complex(h) => {
                assert_eq!(h.prompt.as_deref(), Some("Verify output"));
                assert_eq!(h.on_failure, OnFailure::Warn);
            }
            HookDef::Simple(_) => panic!("expected Complex"),
        }
    }

    #[test]
    fn unknown_toml_section_silently_ignored() {
        let toml_str = r#"
            [workflow]
            merge_via = "direct"

            [skill.start]
            metadata_fields = []
        "#;
        let config: RawConfig = toml::from_str(toml_str).unwrap();
        let resolved = config.resolve();
        assert_eq!(resolved.workflow.merge_via, MergeVia::Direct);
        assert!(resolved.workflow.start.metadata_fields.is_empty());
    }

    #[test]
    fn workflow_start_metadata_fields_toml() {
        let toml_str = r#"
            [[start.metadata_fields]]
            key = "assigned_by"
            source = "env"
            env_var = "USER"
            default = "unknown"

            [[start.metadata_fields]]
            key = "team"
            source = "value"
            value = "backend"

            [[start.metadata_fields]]
            key = "estimate"
            source = "prompt"
            prompt = "Estimated time?"
        "#;
        let config: WorkflowConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.start.metadata_fields.len(), 3);
        assert_eq!(config.start.metadata_fields[0].key, "assigned_by");
        match &config.start.metadata_fields[0].source {
            MetadataFieldSource::Env { env_var } => assert_eq!(env_var, "USER"),
            _ => panic!("expected Env"),
        }
        assert_eq!(
            config.start.metadata_fields[0].default,
            Some(serde_json::Value::String("unknown".to_string()))
        );
        assert_eq!(config.start.metadata_fields[1].key, "team");
        match &config.start.metadata_fields[1].source {
            MetadataFieldSource::Value { value } => assert_eq!(value, "backend"),
            _ => panic!("expected Value"),
        }
        assert_eq!(config.start.metadata_fields[2].key, "estimate");
        match &config.start.metadata_fields[2].source {
            MetadataFieldSource::Prompt { prompt } => assert_eq!(prompt, "Estimated time?"),
            _ => panic!("expected Prompt"),
        }
    }

    #[test]
    fn backend_sqlite_deser() {
        let toml_str = r#"
            [sqlite]
            db_path = "/data/senko.db"
        "#;
        let config: BackendConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.sqlite.db_path.as_deref(),
            Some("/data/senko.db")
        );
    }

    #[test]
    fn cli_remote_deser() {
        let toml_str = r#"
            [remote]
            url = "http://api.senko.local:3141"
            token = "sk_test"
        "#;
        let config: CliConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.remote.url.as_deref(),
            Some("http://api.senko.local:3141")
        );
        assert_eq!(config.remote.token.as_deref(), Some("sk_test"));
    }

    #[test]
    fn web_config_deser() {
        let toml_str = r#"
            host = "0.0.0.0"
            port = 8080
        "#;
        let config: WebConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.host.as_deref(), Some("0.0.0.0"));
        assert_eq!(config.port, Some(8080));
    }

    #[test]
    fn server_auth_deser() {
        let toml_str = r#"
            host = "127.0.0.1"
            port = 3142

            [auth.api_key]
            master_key = "sk_master"

            [auth.oidc]
            issuer_url = "https://auth.example.com"
            client_id = "my_client"
        "#;
        let config: ServerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.host.as_deref(), Some("127.0.0.1"));
        assert_eq!(config.port, Some(3142));
        assert_eq!(
            config.auth.api_key.master_key.as_deref(),
            Some("sk_master")
        );
        assert_eq!(
            config.auth.oidc.issuer_url.as_deref(),
            Some("https://auth.example.com")
        );
    }

    #[test]
    fn raw_config_merge_events() {
        let base: RawConfig = toml::from_str(
            r#"
            [workflow.start]
            instructions = ["base instruction"]
            [workflow.plan]
            required_sections = ["Context"]
        "#,
        )
        .unwrap();

        let overlay: RawConfig = toml::from_str(
            r#"
            [workflow.start]
            instructions = ["overlay instruction"]
        "#,
        )
        .unwrap();

        let merged = base.merge(overlay);
        let start = merged.workflow.start.unwrap();
        assert_eq!(start.instructions, vec!["overlay instruction"]);
        // plan was only in base, should survive
        let plan = merged.workflow.plan.unwrap();
        assert_eq!(plan.required_sections, vec!["Context"]);
    }

    #[test]
    fn resolve_full() {
        let raw: RawConfig = toml::from_str(
            r#"
            [project]
            name = "test"

            [backend.sqlite]
            db_path = "/tmp/test.db"

            [cli.remote]
            url = "http://localhost:3141"

            [web]
            host = "0.0.0.0"
            port = 8080

            [server]
            host = "127.0.0.1"
            port = 3142

            [server.auth.api_key]
            master_key = "secret"

            [workflow]
            merge_via = "pr"
            branch_template = "feat/{id}"

            [workflow.start]
            instructions = ["Check prerequisites"]
        "#,
        )
        .unwrap();

        let config = raw.resolve();
        assert_eq!(config.project.name.as_deref(), Some("test"));
        assert_eq!(
            config.backend.sqlite.db_path.as_deref(),
            Some("/tmp/test.db")
        );
        assert_eq!(
            config.cli.remote.url.as_deref(),
            Some("http://localhost:3141")
        );
        assert_eq!(config.web.host.as_deref(), Some("0.0.0.0"));
        assert_eq!(config.web.port, Some(8080));
        assert_eq!(config.server.host.as_deref(), Some("127.0.0.1"));
        assert_eq!(config.server.port, Some(3142));
        assert_eq!(
            config.server.auth.api_key.master_key.as_deref(),
            Some("secret")
        );
        assert_eq!(config.workflow.merge_via, MergeVia::Pr);
        assert_eq!(
            config.workflow.branch_template.as_deref(),
            Some("feat/{id}")
        );
        assert_eq!(
            config.workflow.start.instructions,
            vec!["Check prerequisites"]
        );
        // Defaults
        assert!(config.workflow.auto_merge);
        assert_eq!(config.workflow.branch_mode, BranchMode::Worktree);
        assert_eq!(config.workflow.merge_strategy, MergeStrategy::Rebase);
    }

    #[test]
    fn apply_env_new_paths() {
        let mut config = Config::default();

        // Simulate env vars using direct assignment (apply_env reads from std::env)
        config.backend.sqlite.db_path = Some("/env/db.db".to_string());
        config.cli.remote.url = Some("http://env:3141".to_string());
        config.cli.remote.token = Some("env_token".to_string());
        config.web.host = Some("0.0.0.0".to_string());
        config.web.port = Some(9090);
        config.server.host = Some("10.0.0.1".to_string());
        config.server.port = Some(3142);
        config.server.auth.api_key.master_key = Some("env_key".to_string());

        assert_eq!(
            config.backend.sqlite.db_path.as_deref(),
            Some("/env/db.db")
        );
        assert_eq!(
            config.cli.remote.url.as_deref(),
            Some("http://env:3141")
        );
        assert_eq!(config.cli.remote.token.as_deref(), Some("env_token"));
        assert_eq!(config.web.host.as_deref(), Some("0.0.0.0"));
        assert_eq!(config.web.port, Some(9090));
        assert_eq!(config.server.host.as_deref(), Some("10.0.0.1"));
        assert_eq!(config.server.port, Some(3142));
        assert_eq!(
            config.server.auth.api_key.master_key.as_deref(),
            Some("env_key")
        );
    }

    #[test]
    fn web_port_helpers() {
        let mut config = Config::default();
        assert_eq!(config.web_port_or(3141), 3141);
        assert!(!config.web_port_is_explicit());

        config.web.port = Some(8080);
        assert_eq!(config.web_port_or(3141), 8080);
        assert!(config.web_port_is_explicit());
    }

    #[test]
    fn effective_host_default() {
        let config = Config::default();
        assert_eq!(config.effective_host(), "127.0.0.1");
    }

    #[test]
    fn effective_host_custom() {
        let mut config = Config::default();
        config.web.host = Some("0.0.0.0".to_string());
        assert_eq!(config.effective_host(), "0.0.0.0");
    }

    #[test]
    fn server_port_helpers() {
        let mut config = Config::default();
        assert_eq!(config.server_port_or(3142), 3142);
        assert!(!config.server_port_is_explicit());

        config.server.port = Some(4000);
        assert_eq!(config.server_port_or(3142), 4000);
        assert!(config.server_port_is_explicit());
    }

    #[test]
    fn effective_server_host_default() {
        let config = Config::default();
        assert_eq!(config.effective_server_host(), "127.0.0.1");
    }

    #[test]
    fn effective_server_host_custom() {
        let mut config = Config::default();
        config.server.host = Some("0.0.0.0".to_string());
        assert_eq!(config.effective_server_host(), "0.0.0.0");
    }

    #[test]
    fn apply_cli_server_overrides() {
        let mut config = Config::default();
        config.apply_cli(&CliOverrides {
            server_port: Some(5000),
            server_host: Some("10.0.0.1".to_string()),
            ..Default::default()
        });
        assert_eq!(config.server.port, Some(5000));
        assert_eq!(config.server.host.as_deref(), Some("10.0.0.1"));
        // web should remain unset
        assert!(config.web.port.is_none());
        assert!(config.web.host.is_none());
    }

    #[test]
    fn cli_browser_deserialize_false() {
        let raw: RawConfig = toml::from_str(
            r#"
            [cli]
            browser = false
        "#,
        )
        .unwrap();
        assert_eq!(raw.cli.browser, Some(false));
    }

    #[test]
    fn cli_browser_default_true() {
        let raw: RawConfig = toml::from_str("").unwrap();
        let config = raw.resolve();
        assert!(config.cli.browser);
    }

    #[test]
    fn cli_browser_merge_overlay_wins() {
        let base: RawConfig = toml::from_str(
            r#"
            [cli]
            browser = true
        "#,
        )
        .unwrap();
        let overlay: RawConfig = toml::from_str(
            r#"
            [cli]
            browser = false
        "#,
        )
        .unwrap();
        let merged = base.merge(overlay);
        assert_eq!(merged.cli.browser, Some(false));
    }
}
