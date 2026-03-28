use std::fmt;
use std::str::FromStr;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Owner,
    Member,
    Viewer,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Role::Owner => "owner",
            Role::Member => "member",
            Role::Viewer => "viewer",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Role::Owner),
            "member" => Ok(Role::Member),
            "viewer" => Ok(Role::Viewer),
            _ => Err(anyhow::anyhow!(
                "invalid role: {s} (expected owner, member, or viewer)"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserParams {
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMember {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub role: Role,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProjectMemberParams {
    pub user_id: i64,
    pub role: Role,
}

impl AddProjectMemberParams {
    pub fn new(user_id: i64, role: Option<Role>) -> Self {
        Self {
            user_id,
            role: role.unwrap_or(Role::Member),
        }
    }
}

// --- API Key types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i64,
    pub user_id: i64,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyWithSecret {
    pub id: i64,
    pub user_id: i64,
    pub key: String,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyParams {
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewApiKey {
    pub raw_key: String,
    pub key_hash: String,
    pub key_prefix: String,
}

impl NewApiKey {
    pub fn generate() -> Self {
        let raw_key = format!("lf_{}", Uuid::new_v4().simple());
        let key_hash = hash_api_key(&raw_key);
        let key_prefix = raw_key[..11].to_string();
        Self {
            raw_key,
            key_hash,
            key_prefix,
        }
    }
}

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}
