use serde::{Deserialize, Serialize};

/// Git configuration for authentication
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GitConfig {
    /// Git username for authentication
    pub username: String,
    /// Git password for authentication
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_git_config() {
        let config = GitConfig::default();

        // Default values should be empty strings
        assert_eq!(config.username, "");
        assert_eq!(config.password, "");
    }

    #[test]
    fn test_custom_git_config() {
        let config = GitConfig {
            username: "test-user".to_string(),
            password: "test-password".to_string(),
        };

        // Check custom values were set correctly
        assert_eq!(config.username, "test-user");
        assert_eq!(config.password, "test-password");
    }

    #[test]
    fn test_git_config_serialization() {
        let config = GitConfig {
            username: "test-user".to_string(),
            password: "test-password".to_string(),
        };

        // Serialize to TOML
        let toml_str = toml::to_string(&config).unwrap();

        // Check it contains username and password
        assert!(toml_str.contains("username"));
        assert!(toml_str.contains("test-user"));
        assert!(toml_str.contains("password"));
        assert!(toml_str.contains("test-password"));

        // Deserialize back to GitConfig
        let deserialized: GitConfig = toml::from_str(&toml_str).unwrap();

        // Should match the original config
        assert_eq!(deserialized.username, config.username);
        assert_eq!(deserialized.password, config.password);
    }
}
