use std::path::PathBuf;

use serde::Deserialize;
use shellexpand::tilde;

/// Default path constants
pub const DEFAULT_ROOT_DIR: &str = "~/.rutd";
pub const DEFAULT_TASKS_DIR: &str = "tasks";
pub const ACTIVE_TASK_FILENAME: &str = "active_task.toml";

/// Path configuration management
#[derive(Debug, Clone, Deserialize)]
pub struct PathConfig {
    /// Root directory path
    root_dir: PathBuf,
    /// Tasks directory path
    tasks_dir: PathBuf,
    /// Active task file path
    active_task_file: PathBuf,
}

impl Default for PathConfig {
    fn default() -> Self {
        let root_dir = PathBuf::from(tilde(DEFAULT_ROOT_DIR).as_ref());
        let tasks_dir = PathBuf::from(DEFAULT_TASKS_DIR);
        let active_task_file = PathBuf::from(ACTIVE_TASK_FILENAME);

        Self {
            root_dir,
            tasks_dir,
            active_task_file,
        }
    }
}

impl PathConfig {
    pub fn root_dir(&self) -> PathBuf {
        self.root_dir.clone()
    }

    pub fn task_dir(&self) -> PathBuf {
        self.root_dir.join(&self.tasks_dir)
    }

    pub fn active_task_file(&self) -> PathBuf {
        self.root_dir.join(&self.active_task_file)
    }
}
