use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;

use crate::application::port::{ContractOperations, TaskBackend};
use crate::domain::contract::{
    Contract, ContractNote, CreateContractParams, UpdateContractArrayParams, UpdateContractParams,
};

pub struct LocalContractOperations {
    backend: Arc<dyn TaskBackend>,
}

impl LocalContractOperations {
    pub fn new(backend: Arc<dyn TaskBackend>) -> Self {
        Self { backend }
    }
}

fn now_iso8601() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[async_trait]
impl ContractOperations for LocalContractOperations {
    async fn create_contract(
        &self,
        project_id: i64,
        params: &CreateContractParams,
    ) -> Result<Contract> {
        params.validate()?;
        self.backend.create_contract(project_id, params).await
    }

    async fn get_contract(&self, id: i64) -> Result<Contract> {
        self.backend.get_contract(id).await
    }

    async fn list_contracts(&self, project_id: i64) -> Result<Vec<Contract>> {
        self.backend.list_contracts(project_id).await
    }

    async fn edit_contract(
        &self,
        id: i64,
        params: &UpdateContractParams,
        array_params: &UpdateContractArrayParams,
    ) -> Result<Contract> {
        params.validate()?;
        array_params.validate()?;
        self.backend.update_contract(id, params, array_params).await
    }

    async fn delete_contract(&self, id: i64) -> Result<()> {
        self.backend.delete_contract(id).await
    }

    async fn check_dod(&self, contract_id: i64, index: usize) -> Result<Contract> {
        self.backend.check_dod(contract_id, index).await
    }

    async fn uncheck_dod(&self, contract_id: i64, index: usize) -> Result<Contract> {
        self.backend.uncheck_dod(contract_id, index).await
    }

    async fn add_note(
        &self,
        contract_id: i64,
        content: String,
        source_task_id: Option<i64>,
    ) -> Result<ContractNote> {
        let note = ContractNote::new(content, source_task_id, now_iso8601());
        note.validate()?;
        self.backend.add_note(contract_id, &note).await
    }

    async fn list_notes(&self, contract_id: i64) -> Result<Vec<ContractNote>> {
        let contract = self.backend.get_contract(contract_id).await?;
        Ok(contract.notes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::port::TaskBackend;
    use crate::infra::sqlite::SqliteBackend;
    use tempfile::tempdir;

    async fn new_backend() -> (tempfile::TempDir, Arc<dyn TaskBackend>, i64) {
        let dir = tempdir().unwrap();
        let backend = SqliteBackend::new(dir.path(), Some(&dir.path().join("data.db")), None).unwrap();
        let backend: Arc<dyn TaskBackend> = Arc::new(backend);
        // Default project id=1 is seeded by migration v1.
        (dir, backend, 1)
    }

    fn simple_params() -> CreateContractParams {
        CreateContractParams {
            title: "contract-title".to_string(),
            description: Some("desc".to_string()),
            definition_of_done: vec!["dod-1".to_string(), "dod-2".to_string()],
            tags: vec!["tag-a".to_string()],
            metadata: None,
        }
    }

    #[tokio::test]
    async fn create_and_get_contract() {
        let (_dir, backend, project_id) = new_backend().await;
        let ops = LocalContractOperations::new(backend);

        let created = ops
            .create_contract(project_id, &simple_params())
            .await
            .unwrap();
        assert_eq!(created.title(), "contract-title");

        let fetched = ops.get_contract(created.id()).await.unwrap();
        assert_eq!(fetched.id(), created.id());
        assert_eq!(fetched.title(), "contract-title");
        assert_eq!(fetched.tags(), &["tag-a".to_string()]);
        assert_eq!(fetched.definition_of_done().len(), 2);
    }

    #[tokio::test]
    async fn list_contracts_returns_all() {
        let (_dir, backend, project_id) = new_backend().await;
        let ops = LocalContractOperations::new(backend);

        ops.create_contract(project_id, &simple_params()).await.unwrap();
        let mut p2 = simple_params();
        p2.title = "another".to_string();
        ops.create_contract(project_id, &p2).await.unwrap();

        let list = ops.list_contracts(project_id).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn edit_contract_scalar_and_array() {
        let (_dir, backend, project_id) = new_backend().await;
        let ops = LocalContractOperations::new(backend);
        let c = ops
            .create_contract(project_id, &simple_params())
            .await
            .unwrap();

        let update = UpdateContractParams {
            title: Some("new-title".to_string()),
            ..Default::default()
        };
        let array_update = UpdateContractArrayParams {
            add_tags: vec!["extra".to_string()],
            add_definition_of_done: vec!["dod-3".to_string()],
            ..Default::default()
        };

        let edited = ops
            .edit_contract(c.id(), &update, &array_update)
            .await
            .unwrap();
        assert_eq!(edited.title(), "new-title");
        assert!(edited.tags().contains(&"extra".to_string()));
        assert_eq!(edited.definition_of_done().len(), 3);
    }

    #[tokio::test]
    async fn check_and_uncheck_dod() {
        let (_dir, backend, project_id) = new_backend().await;
        let ops = LocalContractOperations::new(backend);
        let c = ops
            .create_contract(project_id, &simple_params())
            .await
            .unwrap();

        let after_check = ops.check_dod(c.id(), 1).await.unwrap();
        assert!(after_check.definition_of_done()[0].checked());
        assert!(!after_check.definition_of_done()[1].checked());

        let after_uncheck = ops.uncheck_dod(c.id(), 1).await.unwrap();
        assert!(!after_uncheck.definition_of_done()[0].checked());
    }

    #[tokio::test]
    async fn add_and_list_notes() {
        let (_dir, backend, project_id) = new_backend().await;
        let ops = LocalContractOperations::new(backend.clone());
        let c = ops
            .create_contract(project_id, &simple_params())
            .await
            .unwrap();

        // Create a real task so source_task_id can reference it (FK constraint).
        let task = backend
            .create_task(
                project_id,
                &crate::domain::task::CreateTaskParams {
                    title: "src".to_string(),
                    background: None,
                    description: None,
                    priority: None,
                    definition_of_done: vec![],
                    in_scope: vec![],
                    out_of_scope: vec![],
                    branch: None,
                    pr_url: None,
                    metadata: None,
                    tags: vec![],
                    dependencies: vec![],
                    assignee_user_id: None,
                    contract_id: None,
                },
            )
            .await
            .unwrap();

        let n1 = ops
            .add_note(c.id(), "first note".to_string(), Some(task.id()))
            .await
            .unwrap();
        assert_eq!(n1.content(), "first note");
        assert_eq!(n1.source_task_id(), Some(task.id()));

        ops.add_note(c.id(), "second".to_string(), None).await.unwrap();

        let notes = ops.list_notes(c.id()).await.unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].content(), "first note");
        assert_eq!(notes[0].source_task_id(), Some(task.id()));
        assert_eq!(notes[1].content(), "second");
        assert_eq!(notes[1].source_task_id(), None);
    }

    #[tokio::test]
    async fn delete_contract_removes_it() {
        let (_dir, backend, project_id) = new_backend().await;
        let ops = LocalContractOperations::new(backend);
        let c = ops
            .create_contract(project_id, &simple_params())
            .await
            .unwrap();

        ops.delete_contract(c.id()).await.unwrap();
        assert!(ops.get_contract(c.id()).await.is_err());
    }

    #[tokio::test]
    async fn create_contract_validates_params() {
        let (_dir, backend, project_id) = new_backend().await;
        let ops = LocalContractOperations::new(backend);

        let mut params = simple_params();
        params.title = "x".repeat(10_000);
        let err = ops.create_contract(project_id, &params).await;
        assert!(err.is_err());
    }
}
