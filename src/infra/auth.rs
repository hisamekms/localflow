use std::sync::Arc;

use async_trait::async_trait;

use crate::application::port::auth::{AuthError, AuthProvider};
use crate::application::port::TaskBackend;
use crate::domain::user::hash_api_key;

pub struct ApiKeyProvider {
    backend: Arc<dyn TaskBackend>,
    master_api_key: Option<String>,
}

impl ApiKeyProvider {
    pub fn new(backend: Arc<dyn TaskBackend>, master_api_key: Option<String>) -> Self {
        Self { backend, master_api_key }
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    async fn authenticate(&self, token: &str) -> std::result::Result<crate::domain::user::User, AuthError> {
        if let Some(ref master_key) = self.master_api_key {
            if token == master_key {
                return Ok(crate::domain::user::User::new(
                    0,
                    "master".to_string(),
                    None,
                    None,
                    String::new(),
                ));
            }
        }

        let key_hash = hash_api_key(token);
        self.backend
            .get_user_by_api_key(&key_hash)
            .await
            .map_err(|_| AuthError::InvalidToken)
    }
}
