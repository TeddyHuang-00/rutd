use std::path::{Path, PathBuf};
use shellexpand::tilde;

/// Path configuration management
#[derive(Debug, Clone)]
pub struct PathConfig {
    /// Root directory path
    root_dir: PathBuf,
    /// Tasks directory path
    tasks_dir: PathBuf,
    /// Active task file path
    active_task_file: PathBuf,
}

/// Default path constants
pub const DEFAULT_ROOT_DIR: &str = "~/.rutd";
pub const DEFAULT_TASKS_DIR: &str = "~/.rutd/tasks";
pub const ACTIVE_TASK_FILENAME: &str = "active_task.toml";

impl Default for PathConfig {
    fn default() -> Self {
        let root_dir = PathBuf::from(tilde(DEFAULT_ROOT_DIR).as_ref());
        let tasks_dir = PathBuf::from(tilde(DEFAULT_TASKS_DIR).as_ref());
        let active_task_file = root_dir.join(ACTIVE_TASK_FILENAME);

        Self {
            root_dir,
            tasks_dir,
            active_task_file,
        }
    }
}

impl PathConfig {
    /// Create a new path configuration
    pub fn new(tasks_dir: &str) -> Self {
        // Create tasks directory path
        let tasks_dir = PathBuf::from(tilde(tasks_dir).as_ref());

        // Infer root directory (parent of tasks directory)
        let root_dir = if tasks_dir.ends_with("tasks") {
            tasks_dir.parent().map_or_else(
                || PathBuf::from(tilde(DEFAULT_ROOT_DIR).as_ref()),
                |p| p.to_path_buf(),
            )
        } else {
            // If not standard structure, use tasks directory as root
            tasks_dir.clone()
        };

        // Create active task file path
        let active_task_file = root_dir.join(ACTIVE_TASK_FILENAME);

        Self {
            root_dir,
            tasks_dir,
            active_task_file,
        }
    }

    /// Get root directory path
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Get tasks directory path
    pub fn tasks_dir(&self) -> &Path {
        &self.tasks_dir
    }

    /// Get active task file path
    pub fn active_task_file(&self) -> &Path {
        &self.active_task_file
    }

    /// Get task file path
    pub fn task_file_path(&self, task_id: &str) -> PathBuf {
        self.tasks_dir.join(format!("{}.toml", task_id))
    }
}