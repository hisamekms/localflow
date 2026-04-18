use crate::domain::task::TaskEvent;

/// Result of a task selection attempt (only meaningful for `task_select`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectResult {
    Selected,
    None,
}

impl SelectResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            SelectResult::Selected => "selected",
            SelectResult::None => "none",
        }
    }
}

/// Identifies which hook should fire. Maps domain events to hook config keys.
/// Variants whose `event_name()` returns `None` do not trigger any hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookTrigger {
    Task(TaskEvent),
    /// Task selection outcome (from `task next`).
    TaskSelect {
        project_id: i64,
        result: SelectResult,
    },
}

impl HookTrigger {
    /// Returns the hook config event key name (action form), or `None` if
    /// this trigger does not have a corresponding hook config entry.
    pub fn event_name(&self) -> Option<&'static str> {
        match self {
            HookTrigger::Task(TaskEvent::Created) => Some("task_add"),
            HookTrigger::Task(TaskEvent::Readied) => Some("task_ready"),
            HookTrigger::Task(TaskEvent::Started) => Some("task_start"),
            HookTrigger::Task(TaskEvent::Completed) => Some("task_complete"),
            HookTrigger::Task(TaskEvent::Canceled) => Some("task_cancel"),
            HookTrigger::TaskSelect { .. } => Some("task_select"),
            _ => None,
        }
    }

    /// Valid event names for CLI validation.
    pub fn valid_event_names() -> &'static [&'static str] {
        &[
            "task_add",
            "task_ready",
            "task_start",
            "task_complete",
            "task_cancel",
            "task_select",
        ]
    }

    /// Parse a user-supplied event name string into a HookTrigger.
    /// Used by the CLI `hooks test` subcommand.
    pub fn from_event_name(name: &str) -> Option<Self> {
        match name {
            "task_add" => Some(HookTrigger::Task(TaskEvent::Created)),
            "task_ready" => Some(HookTrigger::Task(TaskEvent::Readied)),
            "task_start" => Some(HookTrigger::Task(TaskEvent::Started)),
            "task_complete" => Some(HookTrigger::Task(TaskEvent::Completed)),
            "task_cancel" => Some(HookTrigger::Task(TaskEvent::Canceled)),
            "task_select" => Some(HookTrigger::TaskSelect {
                project_id: 0,
                result: SelectResult::Selected,
            }),
            _ => None,
        }
    }
}
