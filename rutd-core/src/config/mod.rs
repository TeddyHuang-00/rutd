pub mod git;
pub mod logging;
pub mod path;
pub mod task;

use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
pub use git::GitConfig;
pub use logging::LogConfig;
pub use path::PathConfig;
use serde::{Deserialize, Serialize};
pub use task::TaskConfig;

/// Main configuration structure that holds all configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
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
        let mut figment = Figment::new().merge(Serialized::defaults(Config::default()));

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
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();

        // The cloned config should be equal to the original
        assert_eq!(cloned.path, config.path);
        assert_eq!(cloned.git, config.git);
        assert_eq!(cloned.log, config.log);
        assert_eq!(cloned.task, config.task);
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

    // Testing Config::new() is more challenging as it depends on the environment
    // and file system, but we can test some basic aspects
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

        // Create a custom config file without leading whitespace
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

        println!("Config file path: {}", config_path.display());
        println!("Config file exists: {}", config_path.exists());
        println!(
            "Config file content: {}",
            fs::read_to_string(&config_path).unwrap()
        );

        let config = Config::load(
            config_path.to_str().unwrap(),
            // Set the weird prefix to avoid conflicts
            "RUTD_TEST_CONFIG_FILE_LOADING_",
        );

        if let Err(ref e) = config {
            println!("Failed to load config: {e}");
        }

        assert!(config.is_ok());
        let config = config.unwrap();

        println!("{config:?}");

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
        // Save original environment variables if they exist
        let original_vars = vec![
            (
                "RUTD_TEST_ENV_CAR_CONFIG_LOADING_PATH__ROOT_DIR",
                "/env/test/root",
            ),
            (
                "RUTD_TEST_ENV_CAR_CONFIG_LOADING_PATH__TASKS_DIR",
                "env-test-tasks",
            ),
            (
                "RUTD_TEST_ENV_CAR_CONFIG_LOADING_GIT__USERNAME",
                "env-test-user",
            ),
            ("RUTD_TEST_ENV_CAR_CONFIG_LOADING_LOG__HISTORY", "0"),
            (
                "RUTD_TEST_ENV_CAR_CONFIG_LOADING_TASK__SCOPES",
                "[env-scope-1, env-scope-2]",
            ),
        ];

        unsafe {
            // Set test environment variables
            for &(var, new) in original_vars.iter() {
                env::set_var(var, new);
            }

            let config = Config::load("does-not-exist.toml", "RUTD_TEST_ENV_CAR_CONFIG_LOADING_");
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

            // Restore original environment variables
            for (var, _) in original_vars {
                env::remove_var(var)
            }
        }
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

        unsafe {
            // Set conflicting environment variable which should take precedence
            env::set_var("RUTD_TEST_CONFIG_PRECEDENCE_GIT__USERNAME", "env-user");

            // Load the config with the environment variable prefix
            let config = Config::load(
                config_path.to_str().unwrap(),
                "RUTD_TEST_CONFIG_PRECEDENCE_",
            );

            assert!(config.is_ok());

            let config = config
                .inspect_err(|e| eprintln!("Failed to load config: {e}"))
                .unwrap();

            // Check that environment variable takes precedence over file
            assert_eq!(config.git.username, "env-user");
            // But password from file should still be there
            assert_eq!(config.git.password, "file-password");

            // Restore original environment variables
            env::remove_var("RUTD_TEST_CONFIG_PRECEDENCE_GIT__USERNAME");
        }
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
