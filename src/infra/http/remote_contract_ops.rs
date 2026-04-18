use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::application::port::ContractOperations;
use crate::domain::contract::{
    Contract, ContractNote, CreateContractParams, UpdateContractArrayParams, UpdateContractParams,
};
use crate::domain::task::MetadataUpdate;

use super::client::HttpClient;
use super::{check_success, read_json_or_error};

/// HTTP client implementing `ContractOperations`.
pub struct RemoteContractOperations {
    http: HttpClient,
}

impl RemoteContractOperations {
    pub fn new(base_url: &str, api_key: Option<String>) -> Self {
        Self {
            http: HttpClient::new(base_url, api_key),
        }
    }

    fn project_url(&self, project_id: i64, path: &str) -> String {
        self.http.project_url(project_id, path)
    }

    fn auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        self.http.auth(builder)
    }

    fn client(&self) -> &reqwest::Client {
        self.http.reqwest()
    }
}

fn create_params_to_json(p: &CreateContractParams) -> Value {
    json!({
        "title": p.title,
        "description": p.description,
        "definition_of_done": p.definition_of_done,
        "tags": p.tags,
        "metadata": p.metadata,
    })
}

fn update_body(params: &UpdateContractParams, array_params: &UpdateContractArrayParams) -> Value {
    let mut map = Map::new();

    if let Some(ref title) = params.title {
        map.insert("title".into(), json!(title));
    }
    if let Some(ref desc) = params.description {
        match desc {
            None => {
                map.insert("clear_description".into(), json!(true));
            }
            Some(v) => {
                map.insert("description".into(), json!(v));
            }
        }
    }
    if let Some(ref meta_update) = params.metadata {
        match meta_update {
            MetadataUpdate::Clear => {
                map.insert("clear_metadata".into(), json!(true));
            }
            MetadataUpdate::Merge(v) => {
                map.insert("metadata".into(), json!(v));
            }
            MetadataUpdate::Replace(v) => {
                map.insert("replace_metadata".into(), json!(v));
            }
        }
    }

    if let Some(ref tags) = array_params.set_tags {
        map.insert("set_tags".into(), json!(tags));
    }
    if !array_params.add_tags.is_empty() {
        map.insert("add_tags".into(), json!(array_params.add_tags));
    }
    if !array_params.remove_tags.is_empty() {
        map.insert("remove_tags".into(), json!(array_params.remove_tags));
    }
    if let Some(ref dod) = array_params.set_definition_of_done {
        map.insert("set_definition_of_done".into(), json!(dod));
    }
    if !array_params.add_definition_of_done.is_empty() {
        map.insert(
            "add_definition_of_done".into(),
            json!(array_params.add_definition_of_done),
        );
    }
    if !array_params.remove_definition_of_done.is_empty() {
        map.insert(
            "remove_definition_of_done".into(),
            json!(array_params.remove_definition_of_done),
        );
    }

    Value::Object(map)
}

#[async_trait]
impl ContractOperations for RemoteContractOperations {
    async fn create_contract(
        &self,
        project_id: i64,
        params: &CreateContractParams,
    ) -> Result<Contract> {
        let url = self.project_url(project_id, "/contracts");
        let resp = self
            .auth(
                self.client()
                    .post(&url)
                    .json(&create_params_to_json(params)),
            )
            .send()
            .await?;
        read_json_or_error(resp).await
    }

    async fn get_contract(&self, project_id: i64, id: i64) -> Result<Contract> {
        let url = self.project_url(project_id, &format!("/contracts/{id}"));
        let resp = self.auth(self.client().get(&url)).send().await?;
        read_json_or_error(resp).await
    }

    async fn list_contracts(&self, project_id: i64) -> Result<Vec<Contract>> {
        let url = self.project_url(project_id, "/contracts");
        let resp = self.auth(self.client().get(&url)).send().await?;
        read_json_or_error(resp).await
    }

    async fn edit_contract(
        &self,
        project_id: i64,
        id: i64,
        params: &UpdateContractParams,
        array_params: &UpdateContractArrayParams,
    ) -> Result<Contract> {
        let url = self.project_url(project_id, &format!("/contracts/{id}"));
        let body = update_body(params, array_params);
        let resp = self
            .auth(self.client().put(&url).json(&body))
            .send()
            .await?;
        read_json_or_error(resp).await
    }

    async fn delete_contract(&self, project_id: i64, id: i64) -> Result<()> {
        let url = self.project_url(project_id, &format!("/contracts/{id}"));
        let resp = self.auth(self.client().delete(&url)).send().await?;
        check_success(resp).await
    }

    async fn check_dod(
        &self,
        project_id: i64,
        contract_id: i64,
        index: usize,
    ) -> Result<Contract> {
        let url = self.project_url(
            project_id,
            &format!("/contracts/{contract_id}/dod/{index}/check"),
        );
        let resp = self.auth(self.client().post(&url)).send().await?;
        read_json_or_error(resp).await
    }

    async fn uncheck_dod(
        &self,
        project_id: i64,
        contract_id: i64,
        index: usize,
    ) -> Result<Contract> {
        let url = self.project_url(
            project_id,
            &format!("/contracts/{contract_id}/dod/{index}/uncheck"),
        );
        let resp = self.auth(self.client().post(&url)).send().await?;
        read_json_or_error(resp).await
    }

    async fn add_note(
        &self,
        project_id: i64,
        contract_id: i64,
        content: String,
        source_task_id: Option<i64>,
    ) -> Result<ContractNote> {
        let url = self.project_url(project_id, &format!("/contracts/{contract_id}/notes"));
        let body = json!({
            "content": content,
            "source_task_id": source_task_id,
        });
        let resp = self
            .auth(self.client().post(&url).json(&body))
            .send()
            .await?;
        read_json_or_error(resp).await
    }

    async fn list_notes(&self, project_id: i64, contract_id: i64) -> Result<Vec<ContractNote>> {
        let url = self.project_url(project_id, &format!("/contracts/{contract_id}/notes"));
        let resp = self.auth(self.client().get(&url)).send().await?;
        read_json_or_error(resp).await
    }
}
