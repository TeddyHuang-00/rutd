use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Active Task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTask {
    /// Task ID
    pub task_id: String,
    /// Time when the task was started
    pub started_at: String,
}

impl ActiveTask {
    /// Create a new active task record
    pub const fn new(task_id: String, started_at: String) -> Self {
        Self {
            task_id,
            started_at,
        }
    }
}

/// Save the currently active task
pub fn save_active_task(file_path: &Path, active_task: &ActiveTask) -> Result<()> {
    log::debug!("Saving active task to {}", file_path.display());

    // Make sure the directory exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize the active task to TOML
    let toml_string = toml::to_string(active_task)?;

    // Write to file
    let mut file = File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;

    Ok(())
}

/// Load the currently active task, if any
pub fn load_active_task(file_path: &Path) -> Result<Option<ActiveTask>> {
    log::trace!("Checking for active task at {}", file_path.display());

    if !file_path.exists() {
        log::debug!("No active task file found");
        return Ok(None);
    }

    // Read the file
    let mut contents = String::new();
    let mut file = File::open(file_path).context(format!(
        "Failed to open active task file at {}",
        file_path.display()
    ))?;
    file.read_to_string(&mut contents)?;

    // Deserialize from TOML
    let active_task: ActiveTask =
        toml::from_str(&contents).context("Failed to parse active task TOML")?;

    Ok(Some(active_task))
}

/// Clear the active task
pub fn clear_active_task(file_path: &Path) -> Result<()> {
    if !file_path.exists() {
        log::debug!("No active task file to clear");
        return Ok(());
    }

    // Remove the file
    fs::remove_file(file_path)?;

    log::info!("Cleared active task");
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_active_task_new() {
        let task_id = "test-task-id".to_string();
        let started_at = Local::now().to_rfc3339();

        let active_task = ActiveTask::new(task_id.clone(), started_at.clone());

        assert_eq!(active_task.task_id, task_id);
        assert_eq!(active_task.started_at, started_at);
    }

    #[test]
    fn test_save_load_active_task() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let active_task_file = temp_dir.path().join("active_task.toml");

        // Create an active task
        let task_id = "test-task-id".to_string();
        let started_at = Local::now().to_rfc3339();
        let active_task = ActiveTask::new(task_id.clone(), started_at.clone());

        // Save the active task
        let save_result = save_active_task(&active_task_file, &active_task);
        assert!(save_result.is_ok());
        assert!(active_task_file.exists());

        // Load the active task
        let load_result = load_active_task(&active_task_file);
        assert!(load_result.is_ok());

        let loaded_task_opt = load_result.unwrap();
        assert!(loaded_task_opt.is_some());

        let loaded_task = loaded_task_opt.unwrap();
        assert_eq!(loaded_task.task_id, task_id);
        assert_eq!(loaded_task.started_at, started_at);
    }

    #[test]
    fn test_clear_active_task() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let active_task_file = temp_dir.path().join("active_task.toml");

        // Create and save an active task
        let task_id = "test-task-id".to_string();
        let started_at = Local::now().to_rfc3339();
        let active_task = ActiveTask::new(task_id, started_at);

        let save_result = save_active_task(&active_task_file, &active_task);
        assert!(save_result.is_ok());
        assert!(active_task_file.exists());

        // Clear the active task
        let clear_result = clear_active_task(&active_task_file);
        assert!(clear_result.is_ok());
        assert!(!active_task_file.exists());
    }

    #[test]
    fn test_load_nonexistent_active_task() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.toml");

        // Try to load a nonexistent task
        let load_result = load_active_task(&nonexistent_file);
        assert!(load_result.is_ok());

        let loaded_task_opt = load_result.unwrap();
        assert!(loaded_task_opt.is_none());
    }

    #[test]
    fn test_clear_nonexistent_active_task() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.toml");

        // Try to clear a nonexistent task
        let clear_result = clear_active_task(&nonexistent_file);
        assert!(clear_result.is_ok());
    }

    #[test]
    fn test_active_task_serialization() {
        // Create an active task
        let task_id = "test-task-id".to_string();
        let started_at = "2023-01-01T12:00:00+00:00".to_string();
        let active_task = ActiveTask::new(task_id, started_at);

        // Serialize to TOML
        let serialize_result = toml::to_string(&active_task);
        assert!(serialize_result.is_ok());

        let toml_string = serialize_result.unwrap();

        // Deserialize from TOML
        let deserialize_result: Result<ActiveTask, _> = toml::from_str(&toml_string);
        assert!(deserialize_result.is_ok());

        let deserialized_task = deserialize_result.unwrap();
        assert_eq!(deserialized_task.task_id, active_task.task_id);
        assert_eq!(deserialized_task.started_at, active_task.started_at);
    }
}
