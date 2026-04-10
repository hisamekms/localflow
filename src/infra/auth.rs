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
        if let Some(ref master_key) = self.master_api_key
            && token == master_key
        {
            return Ok(crate::domain::user::User::new(
                0,
                "master".to_string(),
                None,
                None,
                String::new(),
            ));
        }

        let key_hash = hash_api_key(token);
        self.backend
            .get_user_by_api_key(&key_hash)
            .await
            .map_err(|_| AuthError::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::{ApiKeyRepository, CreateUserParams, NewApiKey, UserRepository};
    use crate::infra::sqlite::SqliteBackend;

    async fn setup_backend_with_api_key() -> (Arc<SqliteBackend>, String) {
        let backend = SqliteBackend::new_in_memory().unwrap();
        let user = backend
            .create_user(&CreateUserParams {
                username: "testuser".to_string(),
                display_name: None,
                email: None,
            })
            .await
            .unwrap();
        let new_key = NewApiKey::generate();
        let raw_key = new_key.raw_key.clone();
        backend
            .create_api_key(user.id(), "test-key", &new_key)
            .await
            .unwrap();
        (Arc::new(backend), raw_key)
    }

    #[tokio::test]
    async fn master_key_match() {
        let backend = Arc::new(SqliteBackend::new_in_memory().unwrap());
        let provider =
            ApiKeyProvider::new(backend, Some("master-secret".to_string()));

        let user = provider.authenticate("master-secret").await.unwrap();

        assert_eq!(user.id(), 0);
        assert_eq!(user.username(), "master");
    }

    #[tokio::test]
    async fn master_key_mismatch_valid_user_key() {
        let (backend, raw_key) = setup_backend_with_api_key().await;
        let provider =
            ApiKeyProvider::new(backend, Some("master-secret".to_string()));

        let user = provider.authenticate(&raw_key).await.unwrap();

        assert_eq!(user.username(), "testuser");
    }

    #[tokio::test]
    async fn master_key_mismatch_invalid() {
        let backend = Arc::new(SqliteBackend::new_in_memory().unwrap());
        let provider =
            ApiKeyProvider::new(backend, Some("master-secret".to_string()));

        let result = provider.authenticate("wrong-key").await;

        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[tokio::test]
    async fn no_master_key_valid_user_key() {
        let (backend, raw_key) = setup_backend_with_api_key().await;
        let provider = ApiKeyProvider::new(backend, None);

        let user = provider.authenticate(&raw_key).await.unwrap();

        assert_eq!(user.username(), "testuser");
    }

    #[tokio::test]
    async fn no_master_key_invalid() {
        let backend = Arc::new(SqliteBackend::new_in_memory().unwrap());
        let provider = ApiKeyProvider::new(backend, None);

        let result = provider.authenticate("garbage").await;

        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }
}
