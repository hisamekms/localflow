use std::sync::Arc;

use anyhow::Result;

use crate::application::port::TaskBackend;
use crate::domain::duration::parse_duration;
use crate::domain::user::{
    ApiKey, ApiKeyWithSecret, CreateUserParams, NewApiKey, User,
};
use crate::infra::config::TokenConfig;

pub struct UserService {
    backend: Arc<dyn TaskBackend>,
}

impl UserService {
    pub fn new(backend: Arc<dyn TaskBackend>) -> Self {
        Self { backend }
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        self.backend.list_users().await
    }

    pub async fn create_user(&self, params: &CreateUserParams) -> Result<User> {
        self.backend.create_user(params).await
    }

    pub async fn get_user(&self, id: i64) -> Result<User> {
        self.backend.get_user(id).await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        self.backend.get_user_by_username(username).await
    }

    pub async fn delete_user(&self, id: i64) -> Result<()> {
        self.backend.delete_user(id).await
    }

    // --- API Key management ---

    pub async fn create_api_key(
        &self,
        user_id: i64,
        name: &str,
        device_name: Option<&str>,
    ) -> Result<ApiKeyWithSecret> {
        let new_key = NewApiKey::generate();
        self.backend.create_api_key(user_id, name, device_name, &new_key).await
    }

    pub async fn list_api_keys(&self, user_id: i64) -> Result<Vec<ApiKey>> {
        self.backend.list_api_keys(user_id).await
    }

    pub async fn delete_api_key(&self, key_id: i64) -> Result<()> {
        self.backend.delete_api_key(key_id).await
    }

    // --- Session management ---

    /// Get a user by username, creating them if they don't exist.
    pub async fn get_or_create_user(
        &self,
        username: &str,
        display_name: Option<&str>,
        email: Option<&str>,
    ) -> Result<User> {
        match self.backend.get_user_by_username(username).await {
            Ok(user) => Ok(user),
            Err(_) => {
                let params = CreateUserParams {
                    username: username.to_string(),
                    display_name: display_name.map(String::from),
                    email: email.map(String::from),
                };
                self.backend.create_user(&params).await
            }
        }
    }

    /// Create a session token (API key) for a user, enforcing `max_per_user`.
    /// When the limit is reached, the oldest key is revoked to make room.
    pub async fn create_session_token(
        &self,
        user_id: i64,
        device_name: Option<&str>,
        token_config: &TokenConfig,
    ) -> Result<ApiKeyWithSecret> {
        if let Some(max) = token_config.max_per_user {
            let keys = self.backend.list_api_keys(user_id).await?;
            if keys.len() as u32 >= max {
                // Revoke oldest keys to make room
                let mut sorted = keys;
                sorted.sort_by(|a, b| a.created_at().cmp(b.created_at()));
                let to_remove = (sorted.len() as u32) - max + 1;
                for key in sorted.iter().take(to_remove as usize) {
                    self.backend.delete_api_key(key.id()).await?;
                }
            }
        }

        let new_key = NewApiKey::generate();
        self.backend.create_api_key(user_id, "", device_name, &new_key).await
    }

    /// List active (non-expired) sessions for a user.
    pub async fn list_active_sessions(
        &self,
        user_id: i64,
        token_config: &TokenConfig,
    ) -> Result<Vec<ApiKey>> {
        let keys = self.backend.list_api_keys(user_id).await?;
        let now = chrono::Utc::now();
        let filtered = keys
            .into_iter()
            .filter(|k| !is_key_expired(k, token_config, now))
            .collect();
        Ok(filtered)
    }

    /// Revoke a specific session, verifying ownership.
    pub async fn revoke_session(&self, key_id: i64, user_id: i64) -> Result<()> {
        self.backend.delete_api_key_for_user(key_id, user_id).await
    }

    /// Revoke all sessions for a user.
    pub async fn revoke_all_sessions(&self, user_id: i64) -> Result<()> {
        self.backend.delete_all_api_keys_for_user(user_id).await
    }
}

/// Check whether an API key is expired based on token config.
pub fn is_key_expired(
    key: &ApiKey,
    token_config: &TokenConfig,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    // Check absolute TTL
    if let Some(ref ttl_str) = token_config.ttl {
        if let Ok(ttl) = parse_duration(ttl_str) {
            if let Ok(created) = chrono::DateTime::parse_from_rfc3339(key.created_at()) {
                let elapsed = now.signed_duration_since(created);
                if elapsed > chrono::Duration::from_std(ttl).unwrap_or(chrono::Duration::MAX) {
                    return true;
                }
            }
        }
    }

    // Check inactive TTL
    if let Some(ref inactive_ttl_str) = token_config.inactive_ttl {
        if let Ok(inactive_ttl) = parse_duration(inactive_ttl_str) {
            if let Some(last_used) = key.last_used_at() {
                if let Ok(last) = chrono::DateTime::parse_from_rfc3339(last_used) {
                    let elapsed = now.signed_duration_since(last);
                    if elapsed > chrono::Duration::from_std(inactive_ttl).unwrap_or(chrono::Duration::MAX) {
                        return true;
                    }
                }
            }
        }
    }

    false
}
