use std::sync::Arc;

use async_trait::async_trait;

use crate::application::port::auth::{AuthError, AuthProvider};
use crate::domain::repository::TaskBackend;
use crate::domain::user::hash_api_key;

pub struct ApiKeyProvider {
    backend: Arc<dyn TaskBackend>,
}

impl ApiKeyProvider {
    pub fn new(backend: Arc<dyn TaskBackend>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    async fn authenticate(&self, token: &str) -> std::result::Result<crate::domain::user::User, AuthError> {
        let key_hash = hash_api_key(token);
        self.backend
            .get_user_by_api_key(&key_hash)
            .await
            .map_err(|_| AuthError::InvalidToken)
    }
}
