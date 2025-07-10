pub mod git;
pub mod logging;
pub mod manager;
pub mod path;
pub mod reflection;
pub mod task;

use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
pub use git::GitConfig;
pub use logging::LogConfig;
pub use manager::ConfigManager;
pub use path::PathConfig;
pub use reflection::{ConfigReflection, collect_config_values};
use serde::{Deserialize, Serialize};
pub use task::TaskConfig;

/// Main configuration structure that holds all configuration options
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    /// Path configuration
    pub path: PathConfig,
    /// Git configuration
    pub git: GitConfig,
    /// Log configuration
    pub log: LogConfig,
    /// Task configuration
    pub task: TaskConfig,
}

impl Config {
    /// Get configurations
    ///
    /// Configurations are loaded in the following precedence:
    /// 1. Environment variables
    /// 2. Configuration file
    /// 3. Default values
    pub fn new() -> Result<Self> {
        // Get the package name from the compiled binary name
        // This should be resolved to `rutd` for all binaries
        // (e.g. `rutd`, `rutd-cli`, `rutd-tui`, etc.)
        let pkg_name = env!("CARGO_PKG_NAME")
            .split_once('-')
            .map_or(env!("CARGO_PKG_NAME"), |(name, _)| name)
            .to_string();
        let config_file = format!("~/.{pkg_name}/config.toml");
        let env_var_prefix = pkg_name.to_uppercase() + "_";

        // Load the configuration
        Self::load(&config_file, &env_var_prefix)
    }

    /// This function loads the configuration from a file and environment
    /// variables
    ///
    /// Also useful for testing purposes
    fn load(config_path: &str, env_var_prefix: &str) -> Result<Self> {
        // Create a base Figment with default values
        let mut figment = Figment::new().merge(Serialized::defaults(Self::default()));

        // Only attempt to load from config file if it exists
        let path = std::path::PathBuf::from(config_path);
        if path.exists() {
            figment = figment.merge(Toml::file(config_path));
        }

        // Add environment variables
        figment = figment.merge(Env::prefixed(env_var_prefix).map(|key| {
            // Convert environment variable keys to a format that matches the config
            // structure For example, "RUTD_PATH__ROOT_DIR" becomes
            // "path.root_dir"
            key.as_str()
                // Use double underscore to separate nested keys
                .replace("__", ".")
                .into()
        }));

        // Extract the config
        Ok(figment.extract()?)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use tempfile::tempdir;

    use super::*;

    // Helper to manage environment variables for testing
    struct EnvVarGuard {
        vars: Vec<String>,
    }

    impl EnvVarGuard {
        fn new() -> Self {
            Self { vars: Vec::new() }
        }

        fn set(&mut self, key: &str, value: &str) {
            unsafe {
                env::set_var(key, value);
            }
            self.vars.push(key.to_string());
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                for key in &self.vars {
                    env::remove_var(key);
                }
            }
        }
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();

        // Check that default configs are set
        assert_eq!(config.path, PathConfig::default());
        assert_eq!(config.git, GitConfig::default());
        assert_eq!(config.log, LogConfig::default());
        assert_eq!(config.task, TaskConfig::default());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        // Serialize to TOML
        let toml_str = toml::to_string(&config).unwrap();

        // Should contain sections for path, git, log, and task
        assert!(toml_str.contains("[path]"));
        assert!(toml_str.contains("[git]"));
        assert!(toml_str.contains("[log]"));
        assert!(toml_str.contains("[task]"));

        // Deserialize back to Config
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        // Should match the original config
        assert_eq!(deserialized.path, config.path);
        assert_eq!(deserialized.git, config.git);
        assert_eq!(deserialized.log, config.log);
        assert_eq!(deserialized.task, config.task);
    }

    #[test]
    fn test_package_name_extraction() {
        // This test verifies that the package name extraction logic works

        // For full package name
        let full_name = "rutd";
        let extracted = full_name
            .split_once('-')
            .map_or(full_name, |(name, _)| name);
        assert_eq!(extracted, "rutd");

        // For sub-package name
        let sub_name = "rutd-core";
        let extracted = sub_name.split_once('-').map_or(sub_name, |(name, _)| name);
        assert_eq!(extracted, "rutd");
    }

    #[test]
    fn test_config_file_loading() {
        // Create a temporary directory for our config file
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create a custom config file
        let config_content = r#"
        [path]
        root_dir = "/test/custom/root"
        tasks_dir = "test-tasks"

        [git]
        username = "test-user"
        password = "test-password"

        [log]
        history = 10

        [task]
        scopes = ["test-scope-1", "test-scope-2"]
        types = ["test-type-1", "test-type-2"]
        "#;

        fs::write(&config_path, config_content).unwrap();

        // Load the config with a specific test prefix to avoid interference
        let config = Config::load(
            config_path.to_str().unwrap(),
            "RUTD_TEST_CONFIG_FILE_LOADING_",
        );

        assert!(config.is_ok());
        let config = config.unwrap();

        let path = PathConfig {
            root_dir: PathBuf::from("/test/custom/root"),
            tasks_dir: PathBuf::from("test-tasks"),
            ..Default::default()
        };

        // Check that values from the config file are loaded
        assert_eq!(config.path.task_dir_path(), path.task_dir_path());
        assert_eq!(
            config.path.active_task_file_path(),
            path.active_task_file_path()
        );
        assert_eq!(config.git.username, "test-user");
        assert_eq!(config.git.password, "test-password");
        assert!(!config.log.console);
        assert_eq!(config.log.history, 10);
        assert_eq!(config.task.scopes, vec!["test-scope-1", "test-scope-2"]);
        assert_eq!(config.task.types, vec!["test-type-1", "test-type-2"]);
    }

    #[test]
    fn test_env_var_config_loading() {
        let mut guard = EnvVarGuard::new();

        // Set test environment variables
        guard.set("RUTD_TEST_ENV_CONFIG_PATH__ROOT_DIR", "/env/test/root");
        guard.set("RUTD_TEST_ENV_CONFIG_PATH__TASKS_DIR", "env-test-tasks");
        guard.set("RUTD_TEST_ENV_CONFIG_GIT__USERNAME", "env-test-user");
        guard.set("RUTD_TEST_ENV_CONFIG_LOG__HISTORY", "0");
        guard.set(
            "RUTD_TEST_ENV_CONFIG_TASK__SCOPES",
            "[env-scope-1, env-scope-2]",
        );

        // Load config (with a non-existent file path to test env-only configuration)
        let config = Config::load("does-not-exist.toml", "RUTD_TEST_ENV_CONFIG_");
        assert!(config.is_ok());

        let config = config.unwrap();

        // Check that values from environment variables are loaded
        assert_eq!(
            config.path.task_dir_path().to_str().unwrap(),
            "/env/test/root/env-test-tasks"
        );
        assert_eq!(config.git.username, "env-test-user");
        assert_eq!(config.git.password, "");
        assert_eq!(config.log.history, 0);
        assert!(!config.log.console);
        assert_eq!(
            config.task.scopes,
            vec!["env-scope-1".to_string(), "env-scope-2".to_string()]
        );
    }

    #[test]
    fn test_config_precedence() {
        // Create a temporary directory for our config file
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create a custom config file with one set of values
        let config_content = r#"
        [git]
        username = "file-user"
        password = "file-password"
        "#;

        fs::write(&config_path, config_content).unwrap();

        let mut guard = EnvVarGuard::new();

        // Set conflicting environment variable which should take precedence
        guard.set("RUTD_TEST_CONFIG_PRECEDENCE_GIT__USERNAME", "env-user");

        // Load the config with the environment variable prefix
        let config = Config::load(
            config_path.to_str().unwrap(),
            "RUTD_TEST_CONFIG_PRECEDENCE_",
        );

        assert!(config.is_ok());
        let config = config.unwrap();

        // Check that environment variable takes precedence over file
        assert_eq!(config.git.username, "env-user");
        // But password from file should still be there
        assert_eq!(config.git.password, "file-password");
    }

    #[test]
    fn test_config_new_with_env_vars() {
        // This test directly tests the Config::new() function instead of Config::load()
        let mut guard = EnvVarGuard::new();

        // Set environment variables that should be picked up by Config::new()
        guard.set("RUTD_PATH__ROOT_DIR", "/env/new/root");
        guard.set("RUTD_GIT__USERNAME", "git-username-test");
        guard.set("RUTD_LOG__CONSOLE", "true");

        // Call Config::new() which should pick up our environment variables
        let config = Config::new();
        assert!(config.is_ok());
        let config = config.unwrap();

        // Verify the environment variables were loaded correctly
        assert_eq!(config.path.root_dir, PathBuf::from("/env/new/root"));
        assert_eq!(config.git.username, "git-username-test");
        assert!(config.log.console);
    }

    #[test]
    fn test_config_with_invalid_env_vars() {
        let mut guard = EnvVarGuard::new();

        // Set an invalid boolean value
        guard.set("RUTD_TEST_INVALID_LOG__CONSOLE", "not-a-bool");

        // Loading should fail
        let config = Config::load("does-not-exist.toml", "RUTD_TEST_INVALID_");
        assert!(config.is_err());
    }

    #[test]
    fn test_config_debug_representation() {
        let config = Config {
            path: PathConfig::default(),
            git: GitConfig {
                username: "debug-user".to_string(),
                password: "debug-password".to_string(),
            },
            log: LogConfig::default(),
            task: TaskConfig::default(),
        };

        // Check the debug representation contains the expected content
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("path"));
        assert!(debug_str.contains("git"));
        assert!(debug_str.contains("log"));
        assert!(debug_str.contains("task"));
        assert!(debug_str.contains("debug-user"));
        // Password should be included in debug output by default
        assert!(debug_str.contains("debug-password"));
    }
}
