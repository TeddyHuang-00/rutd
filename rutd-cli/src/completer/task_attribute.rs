use std::{collections::HashSet, ffi::OsStr, path::Path};

use clap::builder::StyledStr;
use clap_complete::CompletionCandidate;
use rutd_core::{Config, Priority, Task, TaskStatus, task::storage};
use strum::{EnumMessage, IntoEnumIterator};

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

    let Config { path, .. } = Config::new().unwrap();
    let tasks = get_tasks(&path.task_dir_path());
    tasks
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

    let Config { path, task, .. } = Config::new().unwrap();
    let tasks = get_tasks(&path.task_dir_path());
    // Get the scopes from the task configuration
    task.scopes
        .into_iter()
        // Get the scopes from the tasks
        .chain(tasks.iter().filter_map(|task| task.scope.clone()))
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

    let Config { path, task, .. } = Config::new().unwrap();
    let tasks = get_tasks(&path.task_dir_path());
    // Get the task types from the task configuration
    task.types
        .into_iter()
        // Get the task types from the tasks
        .chain(tasks.iter().filter_map(|task| task.task_type.clone()))
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
        assert_eq!(completions[0].value(), "longid12");

        // Test with the full long ID as prefix (should still truncate)
        let completions_full = complete_id(OsStr::new("longid12345"));
        assert_eq!(completions_full.len(), 1);
        assert_eq!(completions_full[0].value(), "longid12");
        
        // Test with a short ID that should not be truncated
        let completions_short = complete_id(OsStr::new("task-123"));
        assert_eq!(completions_short.len(), 1);
        assert_eq!(completions_short[0].value(), "task-123");


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
        // After deduplication: project, feature, other, test-scope
        assert_eq!(completions.len(), 4);

        // Test with 'p' prefix
        let completions = complete_scope(OsStr::new("p"));
        assert_eq!(completions.len(), 1);

        // Test with 'f' prefix
        let completions = complete_scope(OsStr::new("f"));
        assert_eq!(completions.len(), 1);

        // Test with 'test-' prefix
        let completions = complete_scope(OsStr::new("test-"));
        assert_eq!(completions.len(), 1);

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
        // After deduplication: bug, feat, docs, test-type
        assert_eq!(completions.len(), 4);

        // Test with 'f' prefix
        let completions = complete_type(OsStr::new("f"));
        assert_eq!(completions.len(), 1);

        // Test with 'b' prefix
        let completions = complete_type(OsStr::new("b"));
        assert_eq!(completions.len(), 1);

        // Test with 'test-' prefix
        let completions = complete_type(OsStr::new("test-"));
        assert_eq!(completions.len(), 1);

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
