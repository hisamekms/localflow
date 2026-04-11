use std::sync::Arc;

use anyhow::Result;

use crate::application::port::TaskBackend;
use crate::application::auth::{Permission, require_project_role};
use crate::domain::project::{CreateProjectParams, Project};
use crate::domain::task::ListTasksFilter;
use crate::domain::user::{
    AddProjectMemberParams, ProjectMember, Role,
};

pub struct ProjectService {
    backend: Arc<dyn TaskBackend>,
}

impl ProjectService {
    pub fn new(backend: Arc<dyn TaskBackend>) -> Self {
        Self { backend }
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        self.backend.list_projects().await
    }

    pub async fn create_project(&self, params: &CreateProjectParams, caller_user_id: Option<i64>) -> Result<Project> {
        let project = self.backend.create_project(params).await?;
        if let Some(uid) = caller_user_id {
            let member_params = AddProjectMemberParams::new(uid, Some(Role::Owner));
            self.backend.add_project_member(project.id(), &member_params).await?;
        }
        Ok(project)
    }

    pub async fn get_project(&self, id: i64) -> Result<Project> {
        self.backend.get_project(id).await
    }

    pub async fn get_project_by_name(&self, name: &str) -> Result<Project> {
        self.backend.get_project_by_name(name).await
    }

    pub async fn delete_project(&self, id: i64, caller_user_id: Option<i64>) -> Result<()> {
        if let Some(uid) = caller_user_id {
            require_project_role(self.backend.as_ref(), uid, id, Permission::Admin).await?;
        }
        let project = self.backend.get_project(id).await?;
        let tasks = self.backend.list_tasks(id, &ListTasksFilter::default()).await?;
        project.validate_deletable(tasks.len() as i64)?;
        self.backend.delete_project(id).await
    }

    // --- Member management ---

    pub async fn list_project_members(
        &self,
        project_id: i64,
    ) -> Result<Vec<ProjectMember>> {
        self.backend.list_project_members(project_id).await
    }

    pub async fn add_project_member(
        &self,
        project_id: i64,
        params: &AddProjectMemberParams,
        caller_user_id: Option<i64>,
    ) -> Result<ProjectMember> {
        if let Some(uid) = caller_user_id {
            require_project_role(self.backend.as_ref(), uid, project_id, Permission::Admin).await?;
        }
        self.backend.add_project_member(project_id, params).await
    }

    pub async fn remove_project_member(
        &self,
        project_id: i64,
        user_id: i64,
        caller_user_id: Option<i64>,
    ) -> Result<()> {
        if let Some(uid) = caller_user_id {
            require_project_role(self.backend.as_ref(), uid, project_id, Permission::Admin).await?;
        }
        self.backend.remove_project_member(project_id, user_id).await
    }

    pub async fn get_project_member(
        &self,
        project_id: i64,
        user_id: i64,
    ) -> Result<ProjectMember> {
        self.backend.get_project_member(project_id, user_id).await
    }

    pub async fn update_member_role(
        &self,
        project_id: i64,
        user_id: i64,
        role: Role,
        caller_user_id: Option<i64>,
    ) -> Result<ProjectMember> {
        if let Some(uid) = caller_user_id {
            require_project_role(self.backend.as_ref(), uid, project_id, Permission::Admin).await?;
        }
        self.backend.update_member_role(project_id, user_id, role).await
    }
}
