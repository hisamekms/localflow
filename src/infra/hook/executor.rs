use std::sync::Arc;

use async_trait::async_trait;

use crate::application::HookTrigger;
use crate::application::port::HookDataSource;
use crate::application::port::HookExecutor;
use crate::domain::contract::Contract;
use crate::domain::task::{Task, TaskStatus, UnblockedTask};
use crate::infra::config::{Config, HookWhen};

use super::{BackendInfo, FireOutcome, RuntimeMode, fire, fire_contract};

/// Shell-based hook executor that spawns hook commands as child processes.
pub struct ShellHookExecutor {
    config: Config,
    runtime_mode: RuntimeMode,
    backend_info: BackendInfo,
    backend: Arc<dyn HookDataSource>,
}

impl ShellHookExecutor {
    pub fn new(
        config: Config,
        runtime_mode: RuntimeMode,
        backend_info: BackendInfo,
        backend: Arc<dyn HookDataSource>,
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
impl HookExecutor for ShellHookExecutor {
    async fn fire(
        &self,
        trigger: &HookTrigger,
        when: HookWhen,
        task: Option<&Task>,
        from_status: Option<TaskStatus>,
        unblocked: Option<Vec<UnblockedTask>>,
    ) -> FireOutcome {
        fire(
            &self.config,
            trigger,
            when,
            task,
            self.backend.as_ref(),
            from_status,
            unblocked,
            &self.runtime_mode,
            &self.backend_info,
        )
        .await
    }

    async fn fire_contract(
        &self,
        trigger: &HookTrigger,
        when: HookWhen,
        contract: Option<&Contract>,
    ) -> FireOutcome {
        fire_contract(
            &self.config,
            trigger,
            when,
            contract,
            self.backend.as_ref(),
            &self.runtime_mode,
            &self.backend_info,
        )
        .await
    }
}
