use async_trait::async_trait;

use crate::application::HookTrigger;
use crate::domain::task::{Task, TaskStatus, UnblockedTask};
use crate::infra::config::HookWhen;
use crate::infra::hook::FireOutcome;

/// Port trait for firing hook events at task lifecycle transitions.
/// The implementation decides whether/how to actually fire hooks
/// (e.g., shell scripts, HTTP callbacks, no-op for tests).
#[async_trait]
pub trait HookExecutor: Send + Sync {
    async fn fire(
        &self,
        trigger: &HookTrigger,
        when: HookWhen,
        task: Option<&Task>,
        from_status: Option<TaskStatus>,
        unblocked: Option<Vec<UnblockedTask>>,
    ) -> FireOutcome;
}

/// No-op implementation for testing or when hooks are disabled.
pub struct NoOpHookExecutor;

#[async_trait]
impl HookExecutor for NoOpHookExecutor {
    async fn fire(
        &self,
        _trigger: &HookTrigger,
        _when: HookWhen,
        _task: Option<&Task>,
        _from_status: Option<TaskStatus>,
        _unblocked: Option<Vec<UnblockedTask>>,
    ) -> FireOutcome {
        FireOutcome::Continue
    }
}
