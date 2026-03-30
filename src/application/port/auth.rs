use async_trait::async_trait;

use crate::domain::user::User;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    Forbidden(String),
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, token: &str) -> std::result::Result<User, AuthError>;
}
