use async_trait::async_trait;

use crate::domain::repository::{ProjectRepository, TaskRepository};

/// Combined trait for backward compatibility.
/// Backends that implement both TaskRepository and ProjectRepository
/// automatically implement TaskBackend via the blanket impl.
#[async_trait]
pub trait TaskBackend: TaskRepository + ProjectRepository {}

impl<T: TaskRepository + ProjectRepository> TaskBackend for T {}
