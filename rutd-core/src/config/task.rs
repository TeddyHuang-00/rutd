use serde::{Deserialize, Serialize};

const DEFAULT_SCOPES: [&str; 1] = ["other"];
const DEFAULT_TYPES: [&str; 8] = [
    "build", "chore", "ci", "docs", "style", "refactor", "perf", "test",
];

/// Task configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskConfig {
    /// Pinned task scopes for autocompletion
    pub scopes: Vec<String>,
    /// Pinned task types for autocompletion
    pub types: Vec<String>,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            scopes: DEFAULT_SCOPES.iter().map(|&s| s.to_string()).collect(),
            types: DEFAULT_TYPES.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_task_config() {
        let config = TaskConfig::default();

        // Check default scopes
        assert_eq!(config.scopes.len(), 1);
        assert_eq!(config.scopes[0], "other");

        // Check default types
        assert_eq!(config.types.len(), 8);
        assert!(config.types.contains(&"build".to_string()));
        assert!(config.types.contains(&"chore".to_string()));
        assert!(config.types.contains(&"ci".to_string()));
        assert!(config.types.contains(&"docs".to_string()));
        assert!(config.types.contains(&"style".to_string()));
        assert!(config.types.contains(&"refactor".to_string()));
        assert!(config.types.contains(&"perf".to_string()));
        assert!(config.types.contains(&"test".to_string()));
    }

    #[test]
    fn test_custom_task_config() {
        let custom_scopes = vec!["project1".to_string(), "project2".to_string()];
        let custom_types = vec!["feature".to_string(), "bugfix".to_string()];

        let config = TaskConfig {
            scopes: custom_scopes.clone(),
            types: custom_types.clone(),
        };

        // Check custom values were set correctly
        assert_eq!(config.scopes, custom_scopes);
        assert_eq!(config.types, custom_types);
    }

    #[test]
    fn test_task_config_serialization() {
        let config = TaskConfig::default();

        // Serialize to TOML
        let toml_str = toml::to_string(&config).unwrap();

        // Check it contains scope and type sections
        assert!(toml_str.contains("scopes"));
        assert!(toml_str.contains("types"));
        assert!(toml_str.contains("other"));
        assert!(toml_str.contains("build"));

        // Deserialize back to TaskConfig
        let deserialized: TaskConfig = toml::from_str(&toml_str).unwrap();

        // Should match the original config
        assert_eq!(deserialized.scopes, config.scopes);
        assert_eq!(deserialized.types, config.types);
    }
}
