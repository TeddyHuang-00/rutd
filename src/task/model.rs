use std::fmt;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

// FIXME: Visible aliases for value enum is not yet supported in clap, see
// https://github.com/clap-rs/clap/pull/5480
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

// FIXME: Visible aliases for value enum is not yet supported in clap, see
// https://github.com/clap-rs/clap/pull/5480
/// Task Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum TaskStatus {
    /// Pending (alias: t, p, pending)
    #[value(aliases = ["t", "p", "pending"])]
    Todo,
    /// Finished (alias: d, f, finished)
    #[value(aliases = ["d", "f", "finished"])]
    Done,
    /// Cancelled (alias: a, x, c, cancelled)
    #[value(aliases = ["a", "x", "c", "cancelled"])]
    Aborted,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Aborted => write!(f, "Aborted"),
        }
    }
}

/// Filter options for task queries
#[derive(Debug, Clone, Default, Args)]
pub struct FilterOptions {
    /// Filter by priority
    #[arg(value_enum, short, long)]
    pub priority: Option<Priority>,

    /// Filter by scope (project name)
    #[arg(short = 'c', long)]
    pub scope: Option<String>,

    /// Filter by task type
    #[arg(short, long)]
    pub task_type: Option<String>,

    /// Filter by status
    #[arg(value_enum, short, long)]
    pub status: Option<TaskStatus>,

    /// Filter by completion date (from)
    #[arg(long)]
    pub date_from: Option<String>,

    /// Filter by completion date (to)
    #[arg(long)]
    pub date_to: Option<String>,

    /// Enable fuzzy matching for description
    #[arg(short, long)]
    pub fuzzy: Option<String>,
}

impl FilterOptions {
    /// Get a scope reference if it exists
    pub fn scope_ref(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    /// Get a fuzzy query reference if it exists
    pub fn fuzzy_ref(&self) -> Option<&str> {
        self.fuzzy.as_deref()
    }

    /// Get a data_from reference if it exists
    pub fn date_from_ref(&self) -> Option<&str> {
        self.date_from.as_deref()
    }

    /// Get a date_to reference if it exists
    pub fn date_to_ref(&self) -> Option<&str> {
        self.date_to.as_deref()
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
