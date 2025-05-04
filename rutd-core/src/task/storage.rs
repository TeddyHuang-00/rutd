use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;

use super::Task;
use crate::git::repo::GitRepo;

/// Save task to TOML file
pub fn save_task(
    root_dir: &Path,
    task: &Task,
    after_action: &str,
    description: &str,
) -> Result<()> {
    // Make sure the tasks directory exists
    fs::create_dir_all(root_dir)?;

    // Initialize the Git repository
    let git_repo = GitRepo::init(root_dir)?;

    // Use the task's UUID as the filename
    let file_path = root_dir.join(format!("{}.toml", task.id));

    // Serialize the task to TOML format
    let toml_string = toml::to_string(task)?;

    // Write the serialized TOML string to a file
    let mut file = File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;

    // Automatically commit changes
    let scope = task.scope.as_deref();
    let task_type = task.task_type.as_deref();
    let commit_message = GitRepo::generate_commit_message(
        after_action,
        scope,
        task_type,
        description,
        task.id.as_str(),
    );
    git_repo.commit_changes(&commit_message)?;

    Ok(())
}

/// Locate all potential task files by ID
///
/// The function searches for task files in the specified directory that
/// match the given task ID.
///
/// Short IDs are supported, so if the task ID is "1234", it will match
/// "1234.toml" and "1234-5678.toml", as long as it is enough to uniquely
/// identify the file.
pub fn locate_all_tasks(root_dir: &Path, task_id: &str) -> Result<Vec<PathBuf>> {
    let mut matching_files = Vec::new();

    // Iterate over all TOML files in the directory
    for entry in fs::read_dir(root_dir)? {
        let path = entry?.path();

        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("toml")
            && path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|stem| stem.starts_with(task_id))
        {
            matching_files.push(path);
        }
    }

    Ok(matching_files)
}

/// Locate task file by ID
///
/// This function utilizes the `locate_all_tasks` function to find all potential
/// task files. It will only return a single file if it is unique. If multiple
/// files are found, or if no files are found, an error will be returned.
pub fn locate_task(root_dir: &Path, task_id: &str) -> Result<PathBuf> {
    // Make sure the directory exists
    // Make sure the directory exists
    if !root_dir.exists() {
        anyhow::bail!("Tasks directory does not exist");
    }

    let matching_files = locate_all_tasks(root_dir, task_id)?;

    match matching_files.len() {
        1 => Ok(matching_files[0].to_owned()),
        0 => anyhow::bail!("No task found with ID starting with {}", task_id),
        _ => anyhow::bail!("Multiple tasks found with ID starting with {}", task_id),
    }
}

/// Load task from TOML file
pub fn load_task(root_dir: &Path, task_id: &str) -> Result<Task> {
    let file = locate_task(root_dir, task_id)?;

    // Read the contents of the file
    let mut file = File::open(&file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the TOML content into a Task struct
    let task = toml::from_str(&contents)?;

    Ok(task)
}

/// Load all tasks
pub fn load_all_tasks(root_dir: &Path) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();

    // Make sure the directory exists
    if !root_dir.exists() {
        return Ok(tasks);
    }

    // Iterate over all TOML files in the directory
    for entry in fs::read_dir(root_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            let mut contents = String::new();
            let mut file = File::open(&path)?;
            file.read_to_string(&mut contents)?;

            if let Ok(task) = toml::from_str(&contents) {
                tasks.push(task);
            }
        }
    }

    Ok(tasks)
}

/// Delete task file
pub fn delete_task(root_dir: &Path, task_ids: &[&str]) -> Result<()> {
    let mut ids = Vec::new();
    for task_id in task_ids {
        // First load the task to get its scope and type before deleting
        let file = locate_task(root_dir, task_id)?;

        // Read task data to get scope and type
        let mut file_content = String::new();
        File::open(&file)?.read_to_string(&mut file_content)?;
        let task: Task = toml::from_str(&file_content)?;

        // Save the id for commit message
        ids.push(task.id.clone());

        // Now delete the file
        fs::remove_file(file)?;
    }

    // Initialize the Git repository
    let git_repo = GitRepo::init(root_dir)?;

    // Automatically commit changes with improved commit message
    let commit_message =
        GitRepo::generate_commit_message("delete", None, None, "Delete tasks", &ids.join("\n"));
    git_repo.commit_changes(&commit_message)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::task::{Priority, TaskStatus};

    // Helper function to create a test task
    fn create_test_task(id: &str) -> Task {
        Task {
            id: id.to_string(),
            description: "Test task".to_string(),
            priority: Priority::Normal,
            scope: Some("test-scope".to_string()),
            task_type: Some("test-type".to_string()),
            status: TaskStatus::Todo,
            created_at: "2023-01-01T12:00:00+00:00".to_string(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        }
    }

    #[test]
    fn test_save_task_creates_file() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        let task = create_test_task("test-id-1");

        // Save task may fail because of Git operations, but file should be created
        let _ = save_task(task_dir, &task, "create", "Test save task");

        // Check that the file was created
        let file_path = task_dir.join(format!("{}.toml", task.id));
        assert!(file_path.exists());

        // Check file contents
        let mut contents = String::new();
        File::open(&file_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert!(contents.contains("test-id-1"));
        assert!(contents.contains("Test task"));
    }

    #[test]
    fn test_locate_all_tasks() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create some test task files directly (bypassing save_task to avoid git
        // dependency)
        let ids = ["task-123", "task-456", "task-789"];

        for id in &ids {
            let task = create_test_task(id);
            let file_path = task_dir.join(format!("{id}.toml"));
            let toml_string = toml::to_string(&task).unwrap();
            let mut file = File::create(file_path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        }

        // Test locating tasks with prefix
        let files = locate_all_tasks(task_dir, "task-").unwrap();
        assert_eq!(files.len(), 3);

        // Test locating specific task
        let files = locate_all_tasks(task_dir, "task-123").unwrap();
        assert_eq!(files.len(), 1);

        // Test locating non-existent task
        let files = locate_all_tasks(task_dir, "nonexistent").unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_locate_task() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create some test task files
        let ids = ["task-abc", "task-def"];

        for id in &ids {
            let task = create_test_task(id);
            let file_path = task_dir.join(format!("{id}.toml"));
            let toml_string = toml::to_string(&task).unwrap();
            let mut file = File::create(file_path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        }

        // Test locating specific task
        let file = locate_task(task_dir, "task-abc");
        assert!(file.is_ok());

        // Test error when multiple tasks match
        let file = locate_task(task_dir, "task-");
        assert!(file.is_err());
        assert!(
            file.unwrap_err()
                .to_string()
                .contains("Multiple tasks found")
        );

        // Test error when no tasks match
        let file = locate_task(task_dir, "nonexistent");
        assert!(file.is_err());
        assert!(file.unwrap_err().to_string().contains("No task found"));
    }

    #[test]
    fn test_load_task() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create a test task file
        let task_id = "test-load-id";
        let task = create_test_task(task_id);
        let file_path = task_dir.join(format!("{task_id}.toml"));
        let toml_string = toml::to_string(&task).unwrap();
        let mut file = File::create(file_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();

        // Load the task
        let loaded_task = load_task(task_dir, task_id);
        assert!(loaded_task.is_ok());

        let loaded_task = loaded_task.unwrap();
        assert_eq!(loaded_task.id, task_id);
        assert_eq!(loaded_task.description, "Test task");
        assert_eq!(loaded_task.status, TaskStatus::Todo);
    }

    #[test]
    fn test_load_all_tasks() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create some test task files
        let ids = ["task-1", "task-2", "task-3"];

        for id in &ids {
            let task = create_test_task(id);
            let file_path = task_dir.join(format!("{id}.toml"));
            let toml_string = toml::to_string(&task).unwrap();
            let mut file = File::create(file_path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        }

        // Load all tasks
        let tasks = load_all_tasks(task_dir).unwrap();
        assert_eq!(tasks.len(), 3);

        // Verify task IDs are present
        let task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
        assert!(task_ids.contains(&"task-1".to_string()));
        assert!(task_ids.contains(&"task-2".to_string()));
        assert!(task_ids.contains(&"task-3".to_string()));
    }

    #[test]
    fn test_load_from_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Try to load from empty directory
        let tasks = load_all_tasks(task_dir).unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_load_from_nonexistent_directory() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        // Try to load from nonexistent directory
        let tasks = load_all_tasks(&nonexistent_dir).unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_delete_task() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create some test task files directly
        let ids = ["delete-task-1", "delete-task-2"];

        for id in &ids {
            let task = create_test_task(id);
            let file_path = task_dir.join(format!("{id}.toml"));
            let toml_string = toml::to_string(&task).unwrap();
            let mut file = File::create(file_path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        }

        // Verify files exist
        for id in &ids {
            let file_path = task_dir.join(format!("{id}.toml"));
            assert!(file_path.exists());
        }

        // Run delete on one task - the delete operation may fail due to git operations,
        // but we'll just verify the file system effects
        let _ = delete_task(task_dir, &["delete-task-1"]);

        // Verify the first file was deleted
        let file_path = task_dir.join("delete-task-1.toml");
        assert!(!file_path.exists());

        // Verify the second file still exists
        let file_path = task_dir.join("delete-task-2.toml");
        assert!(file_path.exists());
    }

    #[test]
    fn test_invalid_toml_handling() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create an invalid TOML file
        let invalid_file_path = task_dir.join("invalid-task.toml");
        let invalid_content = "this is not valid TOML content";
        let mut file = File::create(invalid_file_path).unwrap();
        file.write_all(invalid_content.as_bytes()).unwrap();

        // Create a valid TOML file
        let valid_task = create_test_task("valid-task");
        let valid_file_path = task_dir.join("valid-task.toml");
        let toml_string = toml::to_string(&valid_task).unwrap();
        let mut file = File::create(valid_file_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();

        // Test load_all_tasks should skip the invalid file and only load the valid one
        let tasks = load_all_tasks(task_dir).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "valid-task");
    }

    #[test]
    fn test_partial_task_id_matching() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create test tasks with IDs that share a prefix
        let ids = ["abc123", "abc456", "def789"];

        for id in &ids {
            let task = create_test_task(id);
            let file_path = task_dir.join(format!("{id}.toml"));
            let toml_string = toml::to_string(&task).unwrap();
            let mut file = File::create(file_path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        }

        // Test locating with partial ID that matches multiple tasks
        let result = locate_task(task_dir, "abc");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Multiple tasks found")
        );

        // Test locating with partial ID that uniquely identifies a task
        let result = locate_task(task_dir, "abc1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().file_name().unwrap().to_str().unwrap(),
            "abc123.toml"
        );

        // Test locating with full ID
        let result = locate_task(task_dir, "def789");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().file_name().unwrap().to_str().unwrap(),
            "def789.toml"
        );
    }

    #[test]
    fn test_task_serialization_deserialization() {
        // Test full serialization and deserialization cycle
        let original_task = Task {
            id: "serialize-test".to_string(),
            description: "Test serialization".to_string(),
            priority: Priority::High,
            scope: Some("test-scope".to_string()),
            task_type: Some("test-type".to_string()),
            status: TaskStatus::Todo,
            created_at: "2023-01-01T12:00:00+00:00".to_string(),
            updated_at: Some("2023-01-02T12:00:00+00:00".to_string()),
            completed_at: None,
            time_spent: Some(3600), // 1 hour in seconds
        };

        // Serialize to TOML
        let toml_string = toml::to_string(&original_task).unwrap();

        // Verify TOML contains expected fields
        assert!(toml_string.contains("id = \"serialize-test\""));
        assert!(toml_string.contains("description = \"Test serialization\""));
        assert!(toml_string.contains("priority = \"High\""));
        assert!(toml_string.contains("scope = \"test-scope\""));
        assert!(toml_string.contains("task_type = \"test-type\""));
        assert!(toml_string.contains("time_spent = 3600"));

        // Deserialize back to Task
        let deserialized_task: Task = toml::from_str(&toml_string).unwrap();

        // Verify fields match
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
    fn test_batch_delete_tasks() {
        let temp_dir = tempdir().unwrap();
        let task_dir = temp_dir.path();

        // Create several test task files
        let ids = ["batch-1", "batch-2", "batch-3", "keep-4"];

        for id in &ids {
            let task = create_test_task(id);
            let file_path = task_dir.join(format!("{id}.toml"));
            let toml_string = toml::to_string(&task).unwrap();
            let mut file = File::create(file_path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        }

        // Delete multiple tasks at once
        let batch_ids = ["batch-1", "batch-2", "batch-3"];
        let _ = delete_task(task_dir, &batch_ids);

        // Verify all batch files were deleted
        for id in &batch_ids {
            let file_path = task_dir.join(format!("{id}.toml"));
            assert!(!file_path.exists());
        }

        // Verify the keep file still exists
        let file_path = task_dir.join("keep-4.toml");
        assert!(file_path.exists());
    }
}
