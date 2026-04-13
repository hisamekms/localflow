use serde::{Deserialize, Serialize};

use crate::application::{CompleteResult, PreviewResult};
use crate::infra::config::Config;
use crate::domain::metadata_field::{MetadataField, MetadataFieldType};
use crate::domain::project::Project;
use crate::domain::task::{DodItem, Task};
use crate::domain::user::{ApiKey, ApiKeyWithSecret, ProjectMember, User};

// --- Project ---

#[derive(Serialize)]
pub struct ProjectResponse {
    id: i64,
    name: String,
    description: Option<String>,
    created_at: String,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id(),
            name: p.name().to_owned(),
            description: p.description().map(|s| s.to_owned()),
            created_at: p.created_at().to_owned(),
        }
    }
}

// --- MetadataField ---

#[derive(Serialize)]
pub struct MetadataFieldResponse {
    id: i64,
    project_id: i64,
    name: String,
    field_type: MetadataFieldType,
    required_on_complete: bool,
    description: Option<String>,
    created_at: String,
}

impl From<MetadataField> for MetadataFieldResponse {
    fn from(f: MetadataField) -> Self {
        Self {
            id: f.id(),
            project_id: f.project_id(),
            name: f.name().to_owned(),
            field_type: f.field_type(),
            required_on_complete: f.required_on_complete(),
            description: f.description().map(|s| s.to_owned()),
            created_at: f.created_at().to_owned(),
        }
    }
}

// --- Task ---

#[derive(Serialize)]
pub struct DodItemResponse {
    content: String,
    checked: bool,
}

impl From<&DodItem> for DodItemResponse {
    fn from(d: &DodItem) -> Self {
        Self {
            content: d.content().to_owned(),
            checked: d.checked(),
        }
    }
}

#[derive(Serialize)]
pub struct TaskResponse {
    id: i64,
    project_id: i64,
    title: String,
    background: Option<String>,
    description: Option<String>,
    plan: Option<String>,
    priority: String,
    status: String,
    assignee_session_id: Option<String>,
    assignee_user_id: Option<i64>,
    created_at: String,
    updated_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    canceled_at: Option<String>,
    cancel_reason: Option<String>,
    branch: Option<String>,
    pr_url: Option<String>,
    metadata: Option<serde_json::Value>,
    definition_of_done: Vec<DodItemResponse>,
    in_scope: Vec<String>,
    out_of_scope: Vec<String>,
    tags: Vec<String>,
    dependencies: Vec<i64>,
}

impl From<Task> for TaskResponse {
    fn from(t: Task) -> Self {
        Self {
            id: t.task_number(),
            project_id: t.project_id(),
            title: t.title().to_owned(),
            background: t.background().map(|s| s.to_owned()),
            description: t.description().map(|s| s.to_owned()),
            plan: t.plan().map(|s| s.to_owned()),
            priority: t.priority().to_string(),
            status: t.status().to_string(),
            assignee_session_id: t.assignee_session_id().map(|s| s.to_owned()),
            assignee_user_id: t.assignee_user_id(),
            created_at: t.created_at().to_owned(),
            updated_at: t.updated_at().to_owned(),
            started_at: t.started_at().map(|s| s.to_owned()),
            completed_at: t.completed_at().map(|s| s.to_owned()),
            canceled_at: t.canceled_at().map(|s| s.to_owned()),
            cancel_reason: t.cancel_reason().map(|s| s.to_owned()),
            branch: t.branch().map(|s| s.to_owned()),
            pr_url: t.pr_url().map(|s| s.to_owned()),
            metadata: t.metadata().cloned(),
            definition_of_done: t.definition_of_done().iter().map(DodItemResponse::from).collect(),
            in_scope: t.in_scope().to_vec(),
            out_of_scope: t.out_of_scope().to_vec(),
            tags: t.tags().to_vec(),
            dependencies: t.dependencies().to_vec(),
        }
    }
}

// --- Task ViewModel (for web/HTML rendering) ---

pub struct DodItemViewModel {
    pub content: String,
    pub checked: bool,
}

pub struct TaskViewModel {
    pub id: i64,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub canceled_at: Option<String>,
    pub cancel_reason: Option<String>,
    pub background: Option<String>,
    pub description: Option<String>,
    pub plan: Option<String>,
    pub assignee_session_id: Option<String>,
    pub assignee_user_id: Option<i64>,
    pub branch: Option<String>,
    pub pr_url: Option<String>,
    pub definition_of_done: Vec<DodItemViewModel>,
    pub in_scope: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub dependencies: Vec<i64>,
}

impl From<Task> for TaskViewModel {
    fn from(t: Task) -> Self {
        Self {
            id: t.task_number(),
            title: t.title().to_owned(),
            status: t.status().to_string(),
            priority: t.priority().to_string(),
            tags: t.tags().to_vec(),
            created_at: t.created_at().to_owned(),
            updated_at: t.updated_at().to_owned(),
            started_at: t.started_at().map(|s| s.to_owned()),
            completed_at: t.completed_at().map(|s| s.to_owned()),
            canceled_at: t.canceled_at().map(|s| s.to_owned()),
            cancel_reason: t.cancel_reason().map(|s| s.to_owned()),
            background: t.background().map(|s| s.to_owned()),
            description: t.description().map(|s| s.to_owned()),
            plan: t.plan().map(|s| s.to_owned()),
            assignee_session_id: t.assignee_session_id().map(|s| s.to_owned()),
            assignee_user_id: t.assignee_user_id(),
            branch: t.branch().map(|s| s.to_owned()),
            pr_url: t.pr_url().map(|s| s.to_owned()),
            definition_of_done: t
                .definition_of_done()
                .iter()
                .map(|d| DodItemViewModel {
                    content: d.content().to_owned(),
                    checked: d.checked(),
                })
                .collect(),
            in_scope: t.in_scope().to_vec(),
            out_of_scope: t.out_of_scope().to_vec(),
            dependencies: t.dependencies().to_vec(),
        }
    }
}

// --- Complete Task ---

#[derive(Serialize)]
pub struct CompleteTaskResponse {
    pub task: TaskResponse,
    pub unblocked_tasks: Vec<UnblockedTaskInfo>,
}

impl From<CompleteResult> for CompleteTaskResponse {
    fn from(r: CompleteResult) -> Self {
        Self {
            task: TaskResponse::from(r.task),
            unblocked_tasks: r
                .unblocked
                .into_iter()
                .map(|t| UnblockedTaskInfo {
                    id: t.task_number(),
                    title: t.title().to_owned(),
                    status: "todo".to_owned(),
                    priority: t.priority().to_string(),
                })
                .collect(),
        }
    }
}

// --- Preview Transition ---

#[derive(Serialize, Deserialize)]
pub struct PreviewTransitionResponse {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub target_status: String,
    pub operations: Vec<String>,
    pub unblocked_tasks: Vec<UnblockedTaskInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct UnblockedTaskInfo {
    pub id: i64,
    pub title: String,
    pub status: String,
    pub priority: String,
}

impl From<PreviewResult> for PreviewTransitionResponse {
    fn from(r: PreviewResult) -> Self {
        Self {
            allowed: r.allowed,
            reason: r.reason,
            target_status: r.target_status.to_string(),
            operations: r.operations,
            unblocked_tasks: r
                .unblocked_tasks
                .into_iter()
                .map(|t| UnblockedTaskInfo {
                    id: t.task_number(),
                    title: t.title().to_owned(),
                    status: t.status().to_string(),
                    priority: t.priority().to_string(),
                })
                .collect(),
        }
    }
}

// --- User ---

#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
    sub: String,
    display_name: Option<String>,
    email: Option<String>,
    created_at: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id(),
            username: u.username().to_owned(),
            sub: u.sub().to_owned(),
            display_name: u.display_name().map(|s| s.to_owned()),
            email: u.email().map(|s| s.to_owned()),
            created_at: u.created_at().to_owned(),
        }
    }
}

// --- ProjectMember ---

#[derive(Serialize)]
pub struct ProjectMemberResponse {
    id: i64,
    project_id: i64,
    user_id: i64,
    role: String,
    created_at: String,
}

impl From<ProjectMember> for ProjectMemberResponse {
    fn from(m: ProjectMember) -> Self {
        Self {
            id: m.id(),
            project_id: m.project_id(),
            user_id: m.user_id(),
            role: m.role().to_string(),
            created_at: m.created_at().to_owned(),
        }
    }
}

// --- ApiKey ---

#[derive(Serialize)]
pub struct ApiKeyResponse {
    id: i64,
    user_id: i64,
    key_prefix: String,
    name: String,
    device_name: Option<String>,
    created_at: String,
    last_used_at: Option<String>,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(k: ApiKey) -> Self {
        Self {
            id: k.id(),
            user_id: k.user_id(),
            key_prefix: k.key_prefix().to_owned(),
            name: k.name().to_owned(),
            device_name: k.device_name().map(|s| s.to_owned()),
            created_at: k.created_at().to_owned(),
            last_used_at: k.last_used_at().map(|s| s.to_owned()),
        }
    }
}

// --- ApiKeyWithSecret ---

#[derive(Serialize)]
pub struct ApiKeyWithSecretResponse {
    id: i64,
    user_id: i64,
    key: String,
    key_prefix: String,
    name: String,
    device_name: Option<String>,
    created_at: String,
}

impl From<ApiKeyWithSecret> for ApiKeyWithSecretResponse {
    fn from(k: ApiKeyWithSecret) -> Self {
        Self {
            id: k.id(),
            user_id: k.user_id(),
            key: k.key().to_owned(),
            key_prefix: k.key_prefix().to_owned(),
            name: k.name().to_owned(),
            device_name: k.device_name().map(|s| s.to_owned()),
            created_at: k.created_at().to_owned(),
        }
    }
}

// --- Session ---

#[derive(Serialize)]
pub struct SessionResponse {
    pub id: i64,
    pub key_prefix: String,
    pub name: String,
    pub device_name: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

impl From<ApiKey> for SessionResponse {
    fn from(k: ApiKey) -> Self {
        Self {
            id: k.id(),
            key_prefix: k.key_prefix().to_owned(),
            name: k.name().to_owned(),
            device_name: k.device_name().map(|s| s.to_owned()),
            created_at: k.created_at().to_owned(),
            last_used_at: k.last_used_at().map(|s| s.to_owned()),
        }
    }
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub id: i64,
    pub key_prefix: String,
    pub expires_at: Option<String>,
}

// --- Auth config (public) ---

#[derive(Serialize)]
pub struct AuthConfigResponse {
    pub auth_mode: String,
    pub oidc: Option<AuthConfigOidc>,
}

#[derive(Serialize)]
pub struct AuthConfigOidc {
    pub issuer_url: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub callback_ports: Vec<String>,
}

// --- Me (auth status) ---

#[derive(Serialize)]
pub struct MeResponse {
    pub user: UserResponse,
    pub session: SessionResponse,
}

// --- Config ---

#[derive(Serialize)]
pub struct ConfigResponse(serde_json::Value);

const MASKED: &str = "****";

const SENSITIVE_PATHS: &[&[&str]] = &[
    &["auth", "api_key", "master_key"],
    &["auth", "api_key", "master_key_arn"],
    &["backend", "postgres", "url"],
    &["backend", "postgres", "url_arn"],
    &["backend", "postgres", "rds_secrets_arn"],
];

impl ConfigResponse {
    fn mask_sensitive(mut value: serde_json::Value) -> serde_json::Value {
        for path in SENSITIVE_PATHS {
            Self::mask_at_path(&mut value, path);
        }
        value
    }

    fn mask_at_path(value: &mut serde_json::Value, path: &[&str]) {
        if path.len() == 1 {
            if let Some(obj) = value.as_object_mut() {
                if let Some(field) = obj.get(path[0]) {
                    if !field.is_null() {
                        obj.insert(path[0].to_string(), serde_json::Value::String(MASKED.to_string()));
                    }
                }
            }
            return;
        }
        if let Some(obj) = value.as_object_mut() {
            if let Some(child) = obj.get_mut(path[0]) {
                Self::mask_at_path(child, &path[1..]);
            }
        }
    }
}

impl From<Config> for ConfigResponse {
    fn from(c: Config) -> Self {
        let raw = serde_json::to_value(c).unwrap_or_default();
        Self(Self::mask_sensitive(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mask_sensitive_replaces_string_values() {
        let input = json!({
            "auth": {
                "api_key": {
                    "master_key": "secret-key-123",
                    "master_key_arn": "arn:aws:secretsmanager:us-east-1:123:secret:key"
                }
            },
            "backend": {
                "postgres": {
                    "url": "postgres://user:pass@host/db",
                    "url_arn": "arn:aws:secretsmanager:us-east-1:123:secret:url",
                    "rds_secrets_arn": "arn:aws:secretsmanager:us-east-1:123:secret:rds",
                    "max_connections": 10
                }
            },
            "workflow": { "merge_via": "direct" }
        });

        let result = ConfigResponse::mask_sensitive(input);
        let obj = result.as_object().unwrap();

        assert_eq!(obj["auth"]["api_key"]["master_key"], json!("****"));
        assert_eq!(obj["auth"]["api_key"]["master_key_arn"], json!("****"));
        assert_eq!(obj["backend"]["postgres"]["url"], json!("****"));
        assert_eq!(obj["backend"]["postgres"]["url_arn"], json!("****"));
        assert_eq!(obj["backend"]["postgres"]["rds_secrets_arn"], json!("****"));
        assert_eq!(obj["backend"]["postgres"]["max_connections"], json!(10));
        assert_eq!(obj["workflow"]["merge_via"], json!("direct"));
    }

    #[test]
    fn mask_sensitive_preserves_null_fields() {
        let input = json!({
            "auth": {
                "api_key": {
                    "master_key": null,
                    "master_key_arn": null
                }
            }
        });

        let result = ConfigResponse::mask_sensitive(input);
        let obj = result.as_object().unwrap();

        assert_eq!(obj["auth"]["api_key"]["master_key"], json!(null));
        assert_eq!(obj["auth"]["api_key"]["master_key_arn"], json!(null));
    }

    #[test]
    fn mask_sensitive_handles_missing_intermediate_objects() {
        let input = json!({
            "auth": { "enabled": false },
            "workflow": { "merge_via": "direct" }
        });

        let result = ConfigResponse::mask_sensitive(input);
        let obj = result.as_object().unwrap();

        assert_eq!(obj["workflow"]["merge_via"], json!("direct"));
        assert!(obj.get("backend").is_none());
    }
}
