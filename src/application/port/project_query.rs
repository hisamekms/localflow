use anyhow::Result;
use async_trait::async_trait;

use crate::domain::project::Project;

#[async_trait]
pub trait ProjectQueryPort: Send + Sync {
    async fn list_projects(&self) -> Result<Vec<Project>>;
}
