use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Draft,
    Todo,
    InProgress,
    Completed,
    Canceled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskStatus::Draft => "draft",
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Canceled => "canceled",
        };
        write!(f, "{s}")
    }
}

impl FromStr for TaskStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(TaskStatus::Draft),
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "completed" => Ok(TaskStatus::Completed),
            "canceled" => Ok(TaskStatus::Canceled),
            _ => Err(anyhow::anyhow!("invalid task status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}

impl TryFrom<i32> for Priority {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Priority::P0),
            1 => Ok(Priority::P1),
            2 => Ok(Priority::P2),
            3 => Ok(Priority::P3),
            _ => Err(anyhow::anyhow!("invalid priority: {value}")),
        }
    }
}

impl From<Priority> for i32 {
    fn from(p: Priority) -> i32 {
        p as i32
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Priority::P0 => "P0",
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub definition_of_done: Vec<String>,
    pub in_scope: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_display_roundtrip() {
        let statuses = [
            TaskStatus::Draft,
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Canceled,
        ];
        for status in statuses {
            let s = status.to_string();
            let parsed: TaskStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn status_serde_roundtrip() {
        let status = TaskStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");
        let parsed: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }

    #[test]
    fn status_invalid() {
        assert!("invalid".parse::<TaskStatus>().is_err());
    }

    #[test]
    fn priority_try_from_roundtrip() {
        for v in 0..=3 {
            let p = Priority::try_from(v).unwrap();
            let back: i32 = p.into();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn priority_invalid() {
        assert!(Priority::try_from(4).is_err());
        assert!(Priority::try_from(-1).is_err());
    }

    #[test]
    fn priority_serde_roundtrip() {
        let p = Priority::P2;
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }
}
