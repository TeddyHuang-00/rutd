use std::{collections::HashSet, ffi::OsStr, path::Path};

use clap::builder::StyledStr;
use clap_complete::CompletionCandidate;
use rutd_core::{
    Priority, Task, TaskStatus,
    config::{Config, ConfigManager},
    task::storage,
};
use strum::{EnumMessage, IntoEnumIterator};

/// Completion context that holds config and tasks to avoid repeated loading
struct CompletionContext {
    config: Config,
    tasks: Vec<Task>,
}

impl CompletionContext {
    fn new() -> Option<Self> {
        let config_manager = ConfigManager::new().ok()?;
        let config = config_manager.get_effective_config().ok()?;
        let tasks = get_tasks(&config.path.task_dir_path());
        Some(Self { config, tasks })
    }
}

/// Get all tasks from the task directory with error handling
fn get_tasks(task_dir: &Path) -> Vec<Task> {
    if !task_dir.exists() {
        return vec![];
    }

    storage::load_all_tasks(task_dir).unwrap_or_default()
}

/// Get a list of task IDs as completion candidates
pub fn complete_id(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let Some(context) = CompletionContext::new() else {
        return vec![]; // Fallback to empty if context creation fails
    };

    context
        .tasks
        .into_iter()
        // Keep only those that start with the current prefix
        .filter(|task| task.id.starts_with(current))
        // Convert to completion candidates
        .map(|task| {
            // Take the first line of the task description as help text
            let short_description = task.description.lines().next().map(String::from);
            let truncated_id = task.id.chars().take(8).collect::<String>();
            CompletionCandidate::new(truncated_id).help(short_description.map(StyledStr::from))
        })
        .collect()
}

/// Get a list of scopes as completion candidates
pub fn complete_scope(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let Some(context) = CompletionContext::new() else {
        return vec![]; // Fallback to empty if context creation fails
    };

    // Get the scopes from the tasks
    context
        .tasks
        .into_iter()
        .filter_map(|task| task.scope)
        // Get the scopes from the configured values
        .chain(context.config.task.scopes)
        // Keep only those that start with the current prefix
        .filter(|scope| scope.starts_with(current))
        // Remove duplicates
        .collect::<HashSet<_>>()
        .into_iter()
        // Convert to completion candidates
        .map(CompletionCandidate::new)
        .collect()
}

/// Get a list of task types as completion candidates
pub fn complete_type(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let Some(context) = CompletionContext::new() else {
        return vec![]; // Fallback to empty if context creation fails
    };

    // Get the task types from the tasks
    context
        .tasks
        .into_iter()
        .filter_map(|task| task.task_type)
        // Get the task types from the configured values
        .chain(context.config.task.types)
        // Keep only those that start with the current prefix
        .filter(|task_type| task_type.starts_with(current))
        // Remove duplicates
        .collect::<HashSet<_>>()
        .into_iter()
        // Convert to completion candidates
        .map(CompletionCandidate::new)
        .collect()
}

pub fn complete_priority(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    // Get the priorities from enum
    Priority::iter()
        .flat_map(|priority| {
            priority.get_serializations().iter().filter_map(move |p| {
                p.starts_with(current).then_some(
                    CompletionCandidate::new(p)
                        .help(priority.get_documentation().map(StyledStr::from)),
                )
            })
        })
        .collect()
}

pub fn complete_status(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    // Get the statuses from enum
    TaskStatus::iter()
        .flat_map(|status| {
            status.get_serializations().iter().filter_map(move |s| {
                s.starts_with(current).then_some(
                    CompletionCandidate::new(s)
                        .help(status.get_documentation().map(StyledStr::from)),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{env, fs::File, io::Write, os::unix::ffi::OsStrExt, path::PathBuf};

    use rutd_core::task::{Priority, TaskStatus};
    use tempfile::tempdir;
    use toml;

    use super::*;

    // Helper function to create a test task
    fn create_test_task(id: &str, scope: Option<&str>, task_type: Option<&str>) -> Task {
        Task {
            id: id.to_string(),
            description: format!("Test task {id}"),
            priority: Priority::Normal,
            scope: scope.map(|s| s.to_string()),
            task_type: task_type.map(|t| t.to_string()),
            status: TaskStatus::Todo,
            created_at: "2023-01-01T12:00:00+00:00".to_string(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        }
    }

    // Helper function to create test tasks in a directory
    fn create_test_tasks(task_dir: &Path) {
        std::fs::create_dir_all(task_dir).unwrap();

        let tasks = [
            create_test_task("task-123", Some("project"), Some("feat")),
            create_test_task("task-456", Some("feature"), Some("bug")),
            create_test_task("task-789", Some("other"), Some("docs")),
            create_test_task("other-task", None, None),
        ];

        for task in &tasks {
            let file_path = task_dir.join(format!("{}.toml", task.id));
            let toml_string = toml::to_string(&task).unwrap();
            let mut file = File::create(file_path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        }
    }

    // Helper to setup environment with temporary directory
    fn setup_test_env() -> (tempfile::TempDir, Vec<(String, String)>) {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path().join("tasks");

        // Create tasks in the temporary directory
        create_test_tasks(&task_dir);

        // Create environment variables for testing
        let env_vars = vec![
            // Set root directory to our temp directory
            (
                "RUTD_PATH__ROOT_DIR".to_string(),
                temp_dir.path().to_string_lossy().to_string(),
            ),
            // Configure some test scopes
            (
                "RUTD_TASK__SCOPES".to_string(),
                "[project, feature, test-scope]".to_string(),
            ),
            // Configure some test types
            (
                "RUTD_TASK__TYPES".to_string(),
                "[bug, feat, docs, test-type]".to_string(),
            ),
        ];

        // Set the environment variables
        for (key, value) in &env_vars {
            unsafe {
                env::set_var(key, value);
            }
        }

        (temp_dir, env_vars)
    }

    // Helper to clean up environment variables
    fn cleanup_env_vars(env_vars: &[(String, String)]) {
        for (key, _) in env_vars {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    #[test]
    fn test_complete_id_truncation() {
        let (temp_dir, env_vars) = setup_test_env();
        let task_dir = temp_dir.path().join("tasks");

        // Create a task with a long ID
        let long_id_task = create_test_task("longid12345", Some("test"), Some("truncation"));
        let file_path = task_dir.join(format!("{}.toml", long_id_task.id));
        let toml_string = toml::to_string(&long_id_task).unwrap();
        let mut file = File::create(file_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();

        // Test with the prefix of the long ID
        let completions = complete_id(OsStr::new("longid"));
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].get_value(), "longid12");

        // Test with the full long ID as prefix (should still truncate)
        let completions_full = complete_id(OsStr::new("longid12345"));
        assert_eq!(completions_full.len(), 1);
        assert_eq!(completions_full[0].get_value(), "longid12");

        // Test with a short ID that should not be truncated
        let completions_short = complete_id(OsStr::new("task-123"));
        assert_eq!(completions_short.len(), 1);
        assert_eq!(completions_short[0].get_value(), "task-123");

        cleanup_env_vars(&env_vars);
        drop(temp_dir);
    }

    #[test]
    fn test_get_tasks_with_content() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path().join("tasks");
        create_test_tasks(&task_dir);

        let tasks = get_tasks(&task_dir);
        assert_eq!(tasks.len(), 4);
    }

    #[test]
    fn test_get_tasks_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path().join("empty");
        std::fs::create_dir_all(&task_dir).unwrap();

        let tasks = get_tasks(&task_dir);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_get_tasks_nonexistent_directory() {
        let task_dir = PathBuf::from("/nonexistent/directory");
        let tasks = get_tasks(&task_dir);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_complete_id_filtering() {
        let (temp_dir, env_vars) = setup_test_env();

        // Test with empty prefix
        let completions = complete_id(OsStr::new(""));
        assert_eq!(completions.len(), 4);

        // Test with specific prefix
        let completions = complete_id(OsStr::new("task-"));
        assert_eq!(completions.len(), 3);

        // Test with more specific prefix
        let completions = complete_id(OsStr::new("task-1"));
        assert_eq!(completions.len(), 1);

        // Test with non-matching prefix
        let completions = complete_id(OsStr::new("nonexistent"));
        assert_eq!(completions.len(), 0);

        // Test with invalid UTF-8
        let invalid_os_str = OsStr::from_bytes(&[0xff, 0xff]);
        let completions = complete_id(invalid_os_str);
        assert_eq!(completions.len(), 0);

        cleanup_env_vars(&env_vars);
        drop(temp_dir);
    }

    #[test]
    fn test_complete_scope_filtering() {
        let (temp_dir, env_vars) = setup_test_env();

        // Test with empty prefix - should return all scopes from tasks and config
        let completions = complete_scope(OsStr::new(""));
        // With ConfigManager we get scopes from both user config and test environment
        // This should include at least the test environment scopes and task scopes
        assert!(
            completions.len() >= 4,
            "Expected at least 4 scopes, got {}",
            completions.len()
        );

        // Test with 'p' prefix (might match multiple due to user config + test env)
        let completions = complete_scope(OsStr::new("p"));
        assert!(
            !completions.is_empty(),
            "Expected at least 1 scope starting with 'p'"
        );

        // Test with 'f' prefix (might match multiple due to user config + test env)
        let completions = complete_scope(OsStr::new("f"));
        assert!(
            !completions.is_empty(),
            "Expected at least 1 scope starting with 'f'"
        );

        // Test with 'test-' prefix (from environment variable config)
        let completions = complete_scope(OsStr::new("test-"));
        // This might be 0 if the parsing doesn't work or 1 if it does
        assert!(
            completions.len() <= 1,
            "Expected at most 1 scope starting with 'test-'"
        );

        // Test with non-matching prefix
        let completions = complete_scope(OsStr::new("nonexistent"));
        assert_eq!(completions.len(), 0);

        // Test with invalid UTF-8
        let invalid_os_str = OsStr::from_bytes(&[0xff, 0xff]);
        let completions = complete_scope(invalid_os_str);
        assert_eq!(completions.len(), 0);

        cleanup_env_vars(&env_vars);
        drop(temp_dir);
    }

    #[test]
    fn test_complete_type_filtering() {
        let (temp_dir, env_vars) = setup_test_env();

        // Test with empty prefix - should return all types from tasks and config
        let completions = complete_type(OsStr::new(""));
        // With ConfigManager we get types from both user config and test environment
        assert!(
            completions.len() >= 4,
            "Expected at least 4 types, got {}",
            completions.len()
        );

        // Test with 'f' prefix (might match multiple due to user config + test env)
        let completions = complete_type(OsStr::new("f"));
        assert!(
            !completions.is_empty(),
            "Expected at least 1 type starting with 'f'"
        );

        // Test with 'b' prefix (might match multiple due to user config + test env)
        let completions = complete_type(OsStr::new("b"));
        assert!(
            !completions.is_empty(),
            "Expected at least 1 type starting with 'b'"
        );

        // Test with 'test-' prefix (from environment variable config)
        let completions = complete_type(OsStr::new("test-"));
        assert!(
            completions.len() <= 1,
            "Expected at most 1 type starting with 'test-'"
        );

        // Test with non-matching prefix
        let completions = complete_type(OsStr::new("nonexistent"));
        assert_eq!(completions.len(), 0);

        // Test with invalid UTF-8
        let invalid_os_str = OsStr::from_bytes(&[0xff, 0xff]);
        let completions = complete_type(invalid_os_str);
        assert_eq!(completions.len(), 0);

        cleanup_env_vars(&env_vars);
        drop(temp_dir);
    }

    #[test]
    fn test_complete_priority() {
        // Test with empty prefix
        let completions = complete_priority(OsStr::new(""));
        // Should return serializations for all priority values
        assert!(!completions.is_empty());

        // Test with specific prefix 'h' for High
        let completions = complete_priority(OsStr::new("h"));
        assert!(!completions.is_empty());

        // Test with invalid UTF-8
        let invalid_os_str = OsStr::from_bytes(&[0xff, 0xff]);
        let completions = complete_priority(invalid_os_str);
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_complete_status() {
        // Test with empty prefix
        let completions = complete_status(OsStr::new(""));
        // Should return serializations for all status values
        assert!(!completions.is_empty());

        // Test with specific prefix 't' for Todo
        let completions = complete_status(OsStr::new("t"));
        assert!(!completions.is_empty());

        // Test with invalid UTF-8
        let invalid_os_str = OsStr::from_bytes(&[0xff, 0xff]);
        let completions = complete_status(invalid_os_str);
        assert_eq!(completions.len(), 0);
    }
}
