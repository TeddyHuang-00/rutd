use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Task Priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Priority {
    /// Most urgent (alias: u, 0)
    #[value(aliases = ["u", "0"])]
    Urgent,
    /// High priority (alias: h, 1)
    #[value(aliases = ["h", "1"])]
    High,
    /// Normal priority (alias: n, 2)
    #[value(aliases = ["n", "2"])]
    Normal,
    /// Low priority (alias: l, 3)
    #[value(aliases = ["l", "3"])]
    Low,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Urgent => write!(f, "Urgent"),
            Priority::High => write!(f, "High"),
            Priority::Normal => write!(f, "Normal"),
            Priority::Low => write!(f, "Low"),
        }
    }
}

/// Task Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Aborted,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Aborted => write!(f, "Aborted"),
        }
    }
}

/// Task Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: Priority,
    /// Task scope (project name, etc., optional)
    pub scope: Option<String>,
    /// Task type (e.g., feat, fix, other, etc.)
    pub task_type: Option<String>,
    /// Task status
    pub status: TaskStatus,
    /// Task creation time in ISO format
    pub created_at: String,
    /// Task last update time in ISO format
    pub updated_at: Option<String>,
    /// Task completion time in ISO format
    pub completed_at: Option<String>,
    /// Time spent on task in seconds
    pub time_spent: Option<u64>,
}

impl Task {
    pub fn new(
        id: String,
        description: String,
        priority: Priority,
        scope: Option<String>,
        task_type: Option<String>,
    ) -> Self {
        Task {
            id,
            description,
            priority,
            scope,
            task_type,
            status: TaskStatus::Todo,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        }
    }
}
