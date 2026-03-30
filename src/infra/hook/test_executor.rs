use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::application::port::{HookTestPort, TaskBackend};
use crate::domain::task::Task;
use crate::infra::config::Config;

use super::{
    build_event, execute_hook_sync, get_commands_for_event, resolve_envelope_context,
    BackendInfo, HookEnvelope, NoEligibleTaskEvent, RuntimeMode,
};

/// Shell-based implementation of [`HookTestPort`].
/// Uses the existing infra hook functions for envelope construction and synchronous execution.
pub struct ShellHookTestExecutor {
    config: Config,
    runtime_mode: RuntimeMode,
    backend_info: BackendInfo,
    backend: Arc<dyn TaskBackend>,
}

impl ShellHookTestExecutor {
    pub fn new(
        config: Config,
        runtime_mode: RuntimeMode,
        backend_info: BackendInfo,
        backend: Arc<dyn TaskBackend>,
    ) -> Self {
        Self {
            config,
            runtime_mode,
            backend_info,
            backend,
        }
    }
}

#[async_trait]
impl HookTestPort for ShellHookTestExecutor {
    async fn build_task_event_envelope(
        &self,
        _project_id: i64,
        event_name: &str,
        task: &Task,
    ) -> Result<String> {
        let (envelope_project, envelope_user) =
            resolve_envelope_context(&self.config, self.backend.as_ref()).await;
        let event = build_event(event_name, task, self.backend.as_ref(), None, None).await;
        let envelope = HookEnvelope {
            runtime: self.runtime_mode.clone(),
            backend: self.backend_info.clone(),
            project: envelope_project,
            user: envelope_user,
            event,
        };
        Ok(serde_json::to_string_pretty(&envelope)?)
    }

    async fn build_no_eligible_task_envelope(&self, project_id: i64) -> Result<String> {
        let (envelope_project, envelope_user) =
            resolve_envelope_context(&self.config, self.backend.as_ref()).await;
        let stats = self
            .backend
            .task_stats(project_id)
            .await
            .unwrap_or_default();
        let ready_count = self.backend.ready_count(project_id).await.unwrap_or(0);
        let event = NoEligibleTaskEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event: "no_eligible_task".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            stats,
            ready_count,
        };
        let envelope = HookEnvelope {
            runtime: self.runtime_mode.clone(),
            backend: self.backend_info.clone(),
            project: envelope_project,
            user: envelope_user,
            event,
        };
        Ok(serde_json::to_string_pretty(&envelope)?)
    }

    fn get_commands(&self, event_name: &str) -> Option<Vec<String>> {
        get_commands_for_event(&self.config, event_name)
    }

    fn execute_sync(&self, command: &str, json: &str) -> Result<std::process::ExitStatus> {
        execute_hook_sync(command, json)
    }
}
