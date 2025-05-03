use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use shellexpand::tilde;

/// Default path constants
pub const DEFAULT_ROOT_DIR: &str = "~/.rutd";
pub const DEFAULT_TASKS_DIR: &str = "tasks";
pub const DEFAULT_ACTIVE_FILE: &str = "active_task.toml";
pub const DEFAULT_LOG_FILE: &str = "rutd.log";

/// Path configuration management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    /// Root directory path
    root_dir: PathBuf,
    /// Tasks directory path
    tasks_dir: PathBuf,
    /// Active task file path
    active_task_file: PathBuf,
    /// Log file path
    log_file: Option<PathBuf>,
}

impl Default for PathConfig {
    fn default() -> Self {
        let root_dir = PathBuf::from(tilde(DEFAULT_ROOT_DIR).as_ref());
        let tasks_dir = PathBuf::from(DEFAULT_TASKS_DIR);
        let active_task_file = PathBuf::from(DEFAULT_ACTIVE_FILE);
        let log_file = Some(PathBuf::from(DEFAULT_LOG_FILE));

        Self {
            root_dir,
            tasks_dir,
            active_task_file,
            log_file,
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

    pub fn log_file(&self) -> Option<PathBuf> {
        self.log_file.clone().map(|p| self.root_dir.join(p))
    }
}
