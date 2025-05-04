use serde::{Deserialize, Serialize};

/// Default maximum number of lines to keep in log file
pub const DEFAULT_MAX_LOG_HISTORY: usize = 100;

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogConfig {
    /// Maximum number of lines to keep in log file
    ///
    /// Set to 0 to disable log rotation
    pub history: usize,
    /// Write to console
    pub console: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            history: DEFAULT_MAX_LOG_HISTORY,
            console: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_log_config() {
        let config = LogConfig::default();

        // Check default values
        assert_eq!(config.history, DEFAULT_MAX_LOG_HISTORY);
        assert_eq!(config.history, 100); // Explicit check against the constant value
        assert!(!config.console);
    }

    #[test]
    fn test_custom_log_config() {
        // Create a custom log configuration
        let config = LogConfig {
            history: 200,
            console: true,
        };

        // Check custom values
        assert_eq!(config.history, 200);
        assert!(config.console);
    }

    #[test]
    fn test_log_config_serialization() {
        let config = LogConfig::default();

        // Serialize to TOML
        let toml_str = toml::to_string(&config).unwrap();

        // Check serialized content
        assert!(toml_str.contains("history"));
        assert!(toml_str.contains("100"));
        assert!(toml_str.contains("console"));
        assert!(toml_str.contains("false"));

        // Deserialize back to LogConfig
        let deserialized: LogConfig = toml::from_str(&toml_str).unwrap();

        // Should match the original config
        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_zero_history_disable_rotation() {
        // Test that setting history to 0 works as expected
        let config = LogConfig {
            history: 0,
            console: false,
        };

        // This doesn't actually test the behavior, just that the value can be set
        // The actual log rotation behavior would be tested in the logging
        // implementation
        assert_eq!(config.history, 0);
    }
}
