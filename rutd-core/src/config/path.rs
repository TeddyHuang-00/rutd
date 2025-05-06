use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use shellexpand::tilde;

/// Default path constants
pub const DEFAULT_ROOT_DIR: &str = "~/.rutd";
pub const DEFAULT_TASKS_DIR: &str = "tasks";
pub const DEFAULT_ACTIVE_FILE: &str = "active_task.toml";
pub const DEFAULT_LOG_FILE: &str = "rutd.log";

/// Path configuration management
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PathConfig {
    /// Root directory path
    pub root_dir: PathBuf,
    /// Tasks directory path
    pub tasks_dir: PathBuf,
    /// Active task file path
    pub active_task_file: PathBuf,
    /// Log file path
    pub log_file: PathBuf,
}

impl Default for PathConfig {
    fn default() -> Self {
        let root_dir = PathBuf::from(tilde(DEFAULT_ROOT_DIR).as_ref());
        let tasks_dir = PathBuf::from(DEFAULT_TASKS_DIR);
        let active_task_file = PathBuf::from(DEFAULT_ACTIVE_FILE);
        let log_file = PathBuf::from(DEFAULT_LOG_FILE);

        Self {
            root_dir,
            tasks_dir,
            active_task_file,
            log_file,
        }
    }
}

impl PathConfig {
    pub fn root_path(&self) -> PathBuf {
        self.root_dir.clone()
    }

    pub fn task_dir_path(&self) -> PathBuf {
        self.root_dir.join(&self.tasks_dir)
    }

    pub fn active_task_file_path(&self) -> PathBuf {
        self.root_dir.join(&self.active_task_file)
    }

    pub fn log_file_path(&self) -> PathBuf {
        self.root_dir.join(&self.log_file)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_default_path_config() {
        let config = PathConfig::default();

        // Check root directory
        assert!(config.root_dir.to_string_lossy().contains(".rutd"));

        // Check tasks directory is "tasks"
        assert_eq!(config.tasks_dir, PathBuf::from("tasks"));

        // Check active task file is "active_task.toml"
        assert_eq!(config.active_task_file, PathBuf::from("active_task.toml"));

        // Check log file is "rutd.log"
        assert_eq!(config.log_file, PathBuf::from("rutd.log"));
    }

    #[test]
    fn test_task_dir_path() {
        let config = PathConfig {
            // Set a specific root directory for testing
            root_dir: PathBuf::from("/test/root"),
            ..Default::default()
        };

        // Check task directory path
        let task_dir = config.task_dir_path();
        assert_eq!(task_dir, Path::new("/test/root/tasks"));
    }

    #[test]
    fn test_active_task_file_path() {
        let config = PathConfig {
            // Set a specific root directory for testing
            root_dir: PathBuf::from("/test/root"),
            ..Default::default()
        };

        // Check active task file path
        let active_task_file = config.active_task_file_path();
        assert_eq!(active_task_file, Path::new("/test/root/active_task.toml"));
    }

    #[test]
    fn test_log_file_path() {
        let config = PathConfig {
            // Set a specific root directory for testing
            root_dir: PathBuf::from("/test/root"),
            ..Default::default()
        };

        // Check log file path
        let log_file = config.log_file_path();
        assert_eq!(log_file, Path::new("/test/root/rutd.log"));
    }

    #[test]
    fn test_custom_paths() {
        // Create a custom path configuration
        let config = PathConfig {
            root_dir: PathBuf::from("/custom/root"),
            tasks_dir: PathBuf::from("custom_tasks"),
            active_task_file: PathBuf::from("custom_active.toml"),
            log_file: PathBuf::from("custom.log"),
        };

        // Check paths
        assert_eq!(
            config.task_dir_path(),
            Path::new("/custom/root/custom_tasks")
        );
        assert_eq!(
            config.active_task_file_path(),
            Path::new("/custom/root/custom_active.toml")
        );
        assert_eq!(config.log_file_path(), Path::new("/custom/root/custom.log"));
    }
}
