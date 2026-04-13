use anyhow::Result;
use async_trait::async_trait;

use crate::domain::metadata_field::{CreateMetadataFieldParams, MetadataField};

/// Application-level port that exposes metadata field operations.
///
/// Both local (`MetadataFieldService`) and remote implementations can satisfy
/// this trait, allowing the presentation layer to depend only on the
/// abstraction rather than a concrete service type.
#[async_trait]
pub trait MetadataFieldOperations: Send + Sync {
    async fn create_metadata_field(
        &self,
        project_id: i64,
        params: &CreateMetadataFieldParams,
    ) -> Result<MetadataField>;

    async fn list_metadata_fields(&self, project_id: i64) -> Result<Vec<MetadataField>>;

    async fn delete_metadata_field_by_name(
        &self,
        project_id: i64,
        name: &str,
    ) -> Result<()>;
}
