use std::fmt;

use chrono::Local;
#[cfg(feature = "cli")]
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

// FIXME: Visible aliases for value enum is not yet supported in clap, see
// https://github.com/clap-rs/clap/pull/5480
/// Task Priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[derive(Default)]
pub enum Priority {
    /// Most urgent (alias: u, 0)
    #[cfg_attr(feature = "cli", value(aliases = ["u", "0"]))]
    Urgent,
    /// High priority (alias: h, 1)
    #[cfg_attr(feature = "cli", value(aliases = ["h", "1"]))]
    High,
    /// Normal priority (alias: n, 2)
    #[cfg_attr(feature = "cli", value(aliases = ["n", "2"]))]
    #[default]
    Normal,
    /// Low priority (alias: l, 3)
    #[cfg_attr(feature = "cli", value(aliases = ["l", "3"]))]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[derive(Default)]
pub enum TaskStatus {
    /// Pending (alias: t, p, pending)
    #[cfg_attr(feature = "cli", value(aliases = ["t", "p", "pending"]))]
    #[default]
    Todo,
    /// Finished (alias: d, f, finished)
    #[cfg_attr(feature = "cli", value(aliases = ["d", "f", "finished"]))]
    Done,
    /// Cancelled (alias: a, x, c, cancelled)
    #[cfg_attr(feature = "cli", value(aliases = ["a", "x", "c", "cancelled"]))]
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
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        // Create a new task
        let id = "test-id".to_string();
        let description = "Test task".to_string();
        let priority = Priority::Normal;
        let scope = Some("test-scope".to_string());
        let task_type = Some("test-type".to_string());

        let task = Task::new(
            id.clone(),
            description.clone(),
            priority,
            scope.clone(),
            task_type.clone(),
        );

        // Verify task properties
        assert_eq!(task.id, id);
        assert_eq!(task.description, description);
        assert_eq!(task.priority, priority);
        assert_eq!(task.scope, scope);
        assert_eq!(task.task_type, task_type);
        assert_eq!(task.status, TaskStatus::Todo);
        assert!(task.updated_at.is_none());
        assert!(task.completed_at.is_none());
        assert!(task.time_spent.is_none());

        // Verify creation time is in RFC3339 format
        assert!(chrono::DateTime::parse_from_rfc3339(&task.created_at).is_ok());
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(Priority::Urgent.to_string(), "Urgent");
        assert_eq!(Priority::High.to_string(), "High");
        assert_eq!(Priority::Normal.to_string(), "Normal");
        assert_eq!(Priority::Low.to_string(), "Low");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(TaskStatus::Todo.to_string(), "Todo");
        assert_eq!(TaskStatus::Done.to_string(), "Done");
        assert_eq!(TaskStatus::Aborted.to_string(), "Aborted");
    }

    #[test]
    fn test_priority_equality() {
        assert_eq!(Priority::Urgent, Priority::Urgent);
        assert_ne!(Priority::Urgent, Priority::High);
        assert_ne!(Priority::High, Priority::Normal);
        assert_ne!(Priority::Normal, Priority::Low);
    }

    #[test]
    fn test_status_equality() {
        assert_eq!(TaskStatus::Todo, TaskStatus::Todo);
        assert_ne!(TaskStatus::Todo, TaskStatus::Done);
        assert_ne!(TaskStatus::Done, TaskStatus::Aborted);
        assert_ne!(TaskStatus::Aborted, TaskStatus::Todo);
    }

    #[test]
    fn test_priority_default() {
        // Default priority should be Normal
        let default_priority = Priority::default();
        assert_eq!(default_priority, Priority::Normal);
    }

    #[test]
    fn test_status_default() {
        // Default status should be Todo
        let default_status = TaskStatus::default();
        assert_eq!(default_status, TaskStatus::Todo);
    }

    #[test]
    fn test_task_clone() {
        // Create a task
        let original = Task::new(
            "test-id".to_string(),
            "Test description".to_string(),
            Priority::High,
            Some("test-scope".to_string()),
            Some("test-type".to_string()),
        );

        // Clone the task
        let cloned = original.clone();

        // Verify the clone has the same values
        assert_eq!(cloned.id, original.id);
        assert_eq!(cloned.description, original.description);
        assert_eq!(cloned.priority, original.priority);
        assert_eq!(cloned.scope, original.scope);
        assert_eq!(cloned.task_type, original.task_type);
        assert_eq!(cloned.status, original.status);
        assert_eq!(cloned.created_at, original.created_at);
        assert_eq!(cloned.updated_at, original.updated_at);
        assert_eq!(cloned.completed_at, original.completed_at);
        assert_eq!(cloned.time_spent, original.time_spent);
    }

    #[test]
    fn test_task_serialization_deserialization() {
        // Create a task with all fields populated
        let original_task = Task {
            id: "test-id".to_string(),
            description: "Test description".to_string(),
            priority: Priority::High,
            scope: Some("test-scope".to_string()),
            task_type: Some("test-type".to_string()),
            status: TaskStatus::Todo,
            created_at: "2023-01-01T12:00:00+00:00".to_string(),
            updated_at: Some("2023-01-02T12:00:00+00:00".to_string()),
            completed_at: None,
            time_spent: Some(3600),
        };

        // Serialize to TOML
        let toml_string = toml::to_string(&original_task).unwrap();

        // The serialized string should contain all the task fields
        assert!(toml_string.contains("id = \"test-id\""));
        assert!(toml_string.contains("description = \"Test description\""));
        assert!(toml_string.contains("priority = \"High\""));
        assert!(toml_string.contains("scope = \"test-scope\""));
        assert!(toml_string.contains("task_type = \"test-type\""));
        assert!(toml_string.contains("status = \"Todo\""));
        assert!(toml_string.contains("created_at = \"2023-01-01T12:00:00+00:00\""));
        assert!(toml_string.contains("updated_at = \"2023-01-02T12:00:00+00:00\""));
        assert!(!toml_string.contains("completed_at")); // This is None, so shouldn't be in the string
        assert!(toml_string.contains("time_spent = 3600"));

        // Deserialize from TOML
        let deserialized_task: Task = toml::from_str(&toml_string).unwrap();

        // The deserialized task should match the original
        assert_eq!(deserialized_task.id, original_task.id);
        assert_eq!(deserialized_task.description, original_task.description);
        assert_eq!(deserialized_task.priority, original_task.priority);
        assert_eq!(deserialized_task.scope, original_task.scope);
        assert_eq!(deserialized_task.task_type, original_task.task_type);
        assert_eq!(deserialized_task.status, original_task.status);
        assert_eq!(deserialized_task.created_at, original_task.created_at);
        assert_eq!(deserialized_task.updated_at, original_task.updated_at);
        assert_eq!(deserialized_task.completed_at, original_task.completed_at);
        assert_eq!(deserialized_task.time_spent, original_task.time_spent);
    }

    #[test]
    fn test_priority_clone_and_copy() {
        let p1 = Priority::Urgent;
        let p2 = p1; // This uses Copy
        let p3 = p1; // This uses Clone

        assert_eq!(p1, p2);
        assert_eq!(p1, p3);

        // Verify that modifying one doesn't affect the others
        // (though this is guaranteed by the Copy trait)
        let p4 = Priority::Low;
        assert_ne!(p1, p4);
        assert_ne!(p2, p4);
        assert_ne!(p3, p4);
    }

    #[test]
    fn test_status_clone_and_copy() {
        let s1 = TaskStatus::Done;
        let s2 = s1; // This uses Copy
        let s3 = s1; // This uses Clone

        assert_eq!(s1, s2);
        assert_eq!(s1, s3);

        // Verify that modifying one doesn't affect the others
        // (though this is guaranteed by the Copy trait)
        let s4 = TaskStatus::Aborted;
        assert_ne!(s1, s4);
        assert_ne!(s2, s4);
        assert_ne!(s3, s4);
    }

    #[test]
    fn test_debug_representation() {
        // Test Debug representation of Priority
        let debug_str = format!("{:?}", Priority::Urgent);
        assert!(debug_str.contains("Urgent"));

        // Test Debug representation of TaskStatus
        let debug_str = format!("{:?}", TaskStatus::Done);
        assert!(debug_str.contains("Done"));

        // Test Debug representation of Task
        let task = Task::new(
            "debug-test".to_string(),
            "Test description".to_string(),
            Priority::Normal,
            None,
            None,
        );
        let debug_str = format!("{task:?}");
        assert!(debug_str.contains("debug-test"));
        assert!(debug_str.contains("Test description"));
        assert!(debug_str.contains("Normal"));
        assert!(debug_str.contains("Todo"));
    }
}
