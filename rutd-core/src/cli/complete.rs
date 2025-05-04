use std::{collections::HashSet, ffi::OsStr, path::Path};

use clap_complete::CompletionCandidate;

use crate::{
    Config,
    task::{Task, storage},
};

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
    let tasks = get_tasks(&path.task_dir());
    tasks
        .into_iter()
        // Get the task IDs
        .map(|task| task.id)
        // Keep only those that start with the current prefix
        .filter(|id| id.starts_with(current))
        // Remove duplicates
        .collect::<HashSet<_>>()
        .into_iter()
        // Convert to completion candidates
        .map(CompletionCandidate::new)
        .collect()
}

/// Get a list of scopes as completion candidates
pub fn complete_scope(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let Config { path, task, .. } = Config::new().unwrap();
    let tasks = get_tasks(&path.task_dir());
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
    let tasks = get_tasks(&path.task_dir());
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::{PathConfig, TaskConfig},
        task::{Priority, TaskStatus},
    };

    // Mock Config to avoid dependency on env variables and file system
    struct MockConfig {
        path_config: PathConfig,
        task_config: TaskConfig,
    }

    impl MockConfig {
        fn new(task_dir: &Path) -> Self {
            let mut path_config = PathConfig::default();
            path_config.set_root_dir(task_dir);

            let task_config = TaskConfig {
                scopes: vec!["project".to_string(), "feature".to_string()],
                types: vec!["bug".to_string(), "feat".to_string(), "docs".to_string()],
            };

            Self {
                path_config,
                task_config,
            }
        }
    }

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

    // Helper function to create test tasks in the mock task directory
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

    #[test]
    fn test_get_tasks_empty_directory() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Test with an empty directory
        let tasks = get_tasks(task_dir);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_get_tasks_nonexistent_directory() {
        // Create a path to a nonexistent directory
        let temp_dir = tempdir().unwrap();
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        // Test with a nonexistent directory
        let tasks = get_tasks(&nonexistent_dir);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_get_tasks_with_content() {
        // Create a temporary directory with test tasks
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();
        create_test_tasks(task_dir);

        // Test that tasks are loaded properly
        let tasks = get_tasks(task_dir);
        assert_eq!(tasks.len(), 4);

        // Verify task IDs
        let task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
        assert!(task_ids.contains(&"task-123".to_string()));
        assert!(task_ids.contains(&"task-456".to_string()));
        assert!(task_ids.contains(&"task-789".to_string()));
        assert!(task_ids.contains(&"other-task".to_string()));
    }

    // Tests for the completion functions would require mocking Config::new()
    // which is challenging without refactoring the code to be more testable.
    // In a real-world scenario, we would refactor these functions to accept
    // a Config parameter instead of creating one internally.

    // Here's how we could test with mocking if the functions were refactored:

    // Refactored version of complete_id for testability (not used in production)
    fn complete_id_testable(current: &str, path_config: &PathConfig) -> Vec<String> {
        let tasks = get_tasks(&path_config.task_dir());
        tasks
            .into_iter()
            .map(|task| task.id)
            .filter(|id| id.starts_with(current))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    #[test]
    fn test_complete_id_filtering() {
        // Create a temporary directory with test tasks
        let temp_dir = tempdir().unwrap();
        // Create a mock config
        let mock_config = MockConfig::new(temp_dir.path());

        create_test_tasks(&mock_config.path_config.task_dir());

        // Test empty prefix should return all task IDs
        let completions = complete_id_testable("", &mock_config.path_config);
        assert_eq!(completions.len(), 4);

        // Test specific prefix
        let completions = complete_id_testable("task-", &mock_config.path_config);
        assert_eq!(completions.len(), 3);
        assert!(completions.contains(&"task-123".to_string()));
        assert!(completions.contains(&"task-456".to_string()));
        assert!(completions.contains(&"task-789".to_string()));

        // Test more specific prefix
        let completions = complete_id_testable("task-1", &mock_config.path_config);
        assert_eq!(completions.len(), 1);
        assert!(completions.contains(&"task-123".to_string()));

        // Test non-matching prefix
        let completions = complete_id_testable("nonexistent", &mock_config.path_config);
        assert_eq!(completions.len(), 0);
    }

    // Similar testable versions for the other completion functions
    fn complete_scope_testable(
        current: &str,
        path_config: &PathConfig,
        task_config: &TaskConfig,
    ) -> Vec<String> {
        let tasks = get_tasks(&path_config.task_dir());
        task_config
            .scopes
            .clone()
            .into_iter()
            .chain(tasks.iter().filter_map(|task| task.scope.clone()))
            .filter(|scope| scope.starts_with(current))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    #[test]
    fn test_complete_scope_filtering() {
        // Create a temporary directory with test tasks
        let temp_dir = tempdir().unwrap();
        // Create a mock config
        let mock_config = MockConfig::new(temp_dir.path());

        create_test_tasks(&mock_config.path_config.task_dir());

        // Test empty prefix should return all scopes (from tasks and config)
        let completions =
            complete_scope_testable("", &mock_config.path_config, &mock_config.task_config);
        assert_eq!(completions.len(), 3); // project, feature, other (from task)

        // Test 'p' prefix
        let completions =
            complete_scope_testable("p", &mock_config.path_config, &mock_config.task_config);
        assert_eq!(completions.len(), 1);
        assert!(completions.contains(&"project".to_string()));

        // Test 'f' prefix
        let completions =
            complete_scope_testable("f", &mock_config.path_config, &mock_config.task_config);
        assert_eq!(completions.len(), 1);
        assert!(completions.contains(&"feature".to_string()));

        // Test non-matching prefix
        let completions = complete_scope_testable(
            "nonexistent",
            &mock_config.path_config,
            &mock_config.task_config,
        );
        assert_eq!(completions.len(), 0);
    }

    fn complete_type_testable(
        current: &str,
        path_config: &PathConfig,
        task_config: &TaskConfig,
    ) -> Vec<String> {
        let tasks = get_tasks(&path_config.task_dir());
        task_config
            .types
            .clone()
            .into_iter()
            .chain(tasks.iter().filter_map(|task| task.task_type.clone()))
            .filter(|task_type| task_type.starts_with(current))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    #[test]
    fn test_complete_type_filtering() {
        // Create a temporary directory with test tasks
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();
        create_test_tasks(task_dir);

        // Create a mock config
        let mock_config = MockConfig::new(task_dir);

        // Test empty prefix should return all types (from tasks and config)
        let completions =
            complete_type_testable("", &mock_config.path_config, &mock_config.task_config);
        assert_eq!(completions.len(), 3); // bug, feat, docs (after deduplication)

        // Test 'f' prefix
        let completions =
            complete_type_testable("f", &mock_config.path_config, &mock_config.task_config);
        assert_eq!(completions.len(), 1);
        assert!(completions.contains(&"feat".to_string()));

        // Test 'b' prefix
        let completions =
            complete_type_testable("b", &mock_config.path_config, &mock_config.task_config);
        assert_eq!(completions.len(), 1);
        assert!(completions.contains(&"bug".to_string()));

        // Test non-matching prefix
        let completions = complete_type_testable(
            "nonexistent",
            &mock_config.path_config,
            &mock_config.task_config,
        );
        assert_eq!(completions.len(), 0);
    }
}
