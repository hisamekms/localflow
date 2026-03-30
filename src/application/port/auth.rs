use async_trait::async_trait;

use crate::domain::user::User;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    Forbidden(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "missing authorization header"),
            AuthError::InvalidToken => write!(f, "invalid api key"),
            AuthError::Forbidden(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AuthError {}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, token: &str) -> std::result::Result<User, AuthError>;
}
