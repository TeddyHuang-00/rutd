use std::fmt;

use chrono::Local;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumMessage, EnumString};

// FIXME: Visible aliases for value enum is not yet supported in clap, see
// https://github.com/clap-rs/clap/pull/5480
/// Task Priority
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Default,
    EnumString,
    EnumMessage,
    EnumIter,
    AsRefStr,
)]
pub enum Priority {
    /// Low priority
    #[strum(serialize = "l", serialize = "0", serialize = "low")]
    Low,
    /// Normal priority
    #[default]
    #[strum(serialize = "n", serialize = "1", serialize = "normal")]
    Normal,
    /// High priority
    #[strum(serialize = "h", serialize = "2", serialize = "high")]
    High,
    /// Most urgent
    #[strum(serialize = "u", serialize = "3", serialize = "urgent")]
    Urgent,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

// FIXME: Visible aliases for value enum is not yet supported in clap, see
// https://github.com/clap-rs/clap/pull/5480
/// Task Status
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Default,
    EnumString,
    EnumMessage,
    EnumIter,
    AsRefStr,
)]
pub enum TaskStatus {
    /// Cancelled
    #[strum(serialize = "a", serialize = "aborted")]
    Aborted,
    /// Finished
    #[strum(serialize = "d", serialize = "done")]
    Done,
    /// Pending
    #[default]
    #[strum(serialize = "t", serialize = "todo")]
    Todo,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// Task Structure
#[derive(Clone, Debug, Deserialize, Serialize)]
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
        Self {
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
        assert_eq!(Priority::Urgent.to_string(), "urgent");
        assert_eq!(Priority::High.to_string(), "high");
        assert_eq!(Priority::Normal.to_string(), "normal");
        assert_eq!(Priority::Low.to_string(), "low");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(TaskStatus::Todo.to_string(), "todo");
        assert_eq!(TaskStatus::Done.to_string(), "done");
        assert_eq!(TaskStatus::Aborted.to_string(), "aborted");
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
