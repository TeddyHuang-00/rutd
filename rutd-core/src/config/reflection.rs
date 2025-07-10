use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use toml_edit::{Array, Formatted, Value as TomlValue};

use super::Config;

/// Information about a configuration field discovered through introspection
#[derive(Clone, Debug)]
struct FieldInfo {
    path: String,
    value_type: String,
}

/// Trait for types that can provide configuration field information
pub trait ConfigReflection {
    /// Get all field paths that can be configured
    fn get_field_paths() -> Vec<String>;

    /// Get the value at a specific path
    fn get_field_value(&self, path: &str) -> Result<String>;

    /// Set a value at a specific path, returning the TOML representation
    fn parse_field_value(path: &str, value_str: &str) -> Result<TomlValue>;

    /// Validate if a path is valid for this configuration
    fn is_valid_path(path: &str) -> bool;
}

impl ConfigReflection for Config {
    fn get_field_paths() -> Vec<String> {
        let default_config = Config::default();
        let json_value = serde_json::to_value(&default_config)
            .expect("Config should always be serializable to JSON");

        discover_all_paths(&json_value)
            .into_iter()
            .map(|field_info| field_info.path)
            .collect()
    }

    fn get_field_value(&self, path: &str) -> Result<String> {
        // Convert config to JSON first for easier path navigation
        let json_value =
            serde_json::to_value(self).context("Failed to serialize config to JSON")?;

        let value = access_value_by_path(&json_value, path)
            .with_context(|| format!("Configuration key '{path}' not found"))?;

        format_json_value_for_display(value)
    }

    fn parse_field_value(path: &str, value_str: &str) -> Result<TomlValue> {
        // Get the expected type for this path from the default config
        let default_config = Config::default();
        let json_value = serde_json::to_value(&default_config)
            .context("Failed to serialize default config to JSON")?;

        let field_info = discover_all_paths(&json_value)
            .into_iter()
            .find(|info| info.path == path)
            .ok_or_else(|| anyhow::anyhow!("Unknown configuration key: {path}"))?;

        // Parse based on the detected type
        parse_value_by_type(&field_info.value_type, value_str, path)
    }

    fn is_valid_path(path: &str) -> bool {
        Self::get_field_paths().contains(&path.to_string())
    }
}

/// Recursively discover all configuration paths in a JSON structure
fn discover_all_paths(value: &JsonValue) -> Vec<FieldInfo> {
    let mut path_value_pairs = vec![("".to_string(), value)];
    let mut paths = Vec::new();

    while let Some((current_path, current_value)) = path_value_pairs.pop() {
        match current_value {
            JsonValue::Object(map) => {
                for (key, val) in map {
                    let new_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{current_path}.{key}")
                    };

                    // Only add terminal paths (not intermediate objects)
                    if !val.is_object() {
                        let value_type = determine_value_type(val);
                        paths.push(FieldInfo {
                            path: new_path.clone(),
                            value_type,
                        });
                    } else {
                        // Continue recursing for nested structures
                        path_value_pairs.push((new_path, val));
                    }
                }
            }
            // Terminal values are handled by parent object
            _ => unreachable!(),
        }
    }
    paths
}

/// Determine the type of a JSON value
fn determine_value_type(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(_) => "boolean".to_string(),
        JsonValue::Number(n) => {
            if n.is_i64() {
                "integer".to_string()
            } else if n.is_f64() {
                "float".to_string()
            } else {
                "number".to_string()
            }
        }
        JsonValue::String(_) => "string".to_string(),
        JsonValue::Array(_) => "array".to_string(),
        JsonValue::Object(_) => "object".to_string(),
    }
}

/// Access a value in JSON using dot notation
fn access_value_by_path<'a>(json: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in parts {
        current = current.get(part)?;
    }

    Some(current)
}

/// Parse a string value according to its expected type
fn parse_value_by_type(value_type: &str, value_str: &str, path: &str) -> Result<TomlValue> {
    match value_type {
        "boolean" => {
            let bool_val = value_str
                .parse::<bool>()
                .with_context(|| format!("Invalid boolean value for {path}: {value_str}"))?;
            Ok(TomlValue::Boolean(Formatted::new(bool_val)))
        }
        "integer" => {
            let int_val = value_str
                .parse::<i64>()
                .with_context(|| format!("Invalid integer value for {path}: {value_str}"))?;
            Ok(TomlValue::Integer(Formatted::new(int_val)))
        }
        "float" => {
            let float_val = value_str
                .parse::<f64>()
                .with_context(|| format!("Invalid float value for {path}: {value_str}"))?;
            Ok(TomlValue::Float(Formatted::new(float_val)))
        }
        "array" => {
            // Try to parse as JSON array first
            if value_str.starts_with('[') && value_str.ends_with(']') {
                let parsed: Vec<String> = serde_json::from_str(value_str)
                    .with_context(|| format!("Invalid array value for {path}: {value_str}"))?;
                let mut array = Array::new();
                for item in parsed {
                    array.push(item);
                }
                Ok(TomlValue::Array(array))
            } else {
                // Treat as single string value in an array
                let mut array = Array::new();
                array.push(value_str);
                Ok(TomlValue::Array(array))
            }
        }
        "string" => Ok(TomlValue::String(Formatted::new(value_str.to_string()))),
        _ => Ok(TomlValue::String(Formatted::new(value_str.to_string()))),
    }
}

/// Format a JSON value for display to the user
fn format_json_value_for_display(value: &JsonValue) -> Result<String> {
    match value {
        JsonValue::String(s) => Ok(s.clone()),
        JsonValue::Number(n) => Ok(n.to_string()),
        JsonValue::Bool(b) => Ok(b.to_string()),
        JsonValue::Array(arr) => {
            let items: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            Ok(format!("[{}]", items.join(", ")))
        }
        JsonValue::Null => Ok(String::new()),
        JsonValue::Object(_) => Ok(serde_json::to_string_pretty(value)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GitConfig, LogConfig, PathConfig, TaskConfig};

    fn create_test_config() -> Config {
        Config {
            path: PathConfig::default(),
            git: GitConfig {
                username: "test-user".to_string(),
                password: "secret".to_string(),
            },
            log: LogConfig {
                console: true,
                history: 50,
            },
            task: TaskConfig {
                scopes: vec!["backend".to_string(), "frontend".to_string()],
                types: vec!["feat".to_string(), "fix".to_string()],
            },
        }
    }

    #[test]
    fn test_dynamic_field_path_discovery() {
        let paths = Config::get_field_paths();

        // Should contain expected paths without hardcoding them in the test
        assert!(!paths.is_empty());
        assert!(paths.contains(&"git.username".to_string()));
        assert!(paths.contains(&"log.console".to_string()));
        assert!(paths.contains(&"task.scopes".to_string()));

        // Should discover all paths dynamically
        let expected_min_paths = 10; // We know there should be at least this many
        assert!(paths.len() >= expected_min_paths);
    }

    #[test]
    fn test_discover_all_paths() {
        let config = Config::default();
        let json_value = serde_json::to_value(&config).unwrap();
        let field_infos = discover_all_paths(&json_value);

        assert!(!field_infos.is_empty());

        // Check that we have field info for known paths
        let paths: Vec<String> = field_infos.iter().map(|f| f.path.clone()).collect();
        assert!(paths.contains(&"git.username".to_string()));
        assert!(paths.contains(&"log.console".to_string()));

        // Check that types are correctly detected
        let git_username_info = field_infos
            .iter()
            .find(|f| f.path == "git.username")
            .unwrap();
        assert_eq!(git_username_info.value_type, "string");

        let log_console_info = field_infos
            .iter()
            .find(|f| f.path == "log.console")
            .unwrap();
        assert_eq!(log_console_info.value_type, "boolean");
    }

    #[test]
    fn test_get_field_value() {
        let config = create_test_config();

        assert_eq!(config.get_field_value("git.username").unwrap(), "test-user");
        assert_eq!(config.get_field_value("log.console").unwrap(), "true");
        assert_eq!(config.get_field_value("log.history").unwrap(), "50");
        assert_eq!(
            config.get_field_value("task.scopes").unwrap(),
            "[backend, frontend]"
        );
    }

    #[test]
    fn test_dynamic_parse_field_value() {
        // Test boolean parsing
        let value = Config::parse_field_value("log.console", "true").unwrap();
        assert!(value.as_bool().unwrap());

        // Test integer parsing
        let value = Config::parse_field_value("log.history", "42").unwrap();
        assert_eq!(value.as_integer().unwrap(), 42);

        // Test string parsing
        let value = Config::parse_field_value("git.username", "testuser").unwrap();
        assert_eq!(value.as_str().unwrap(), "testuser");

        // Test array parsing
        let value = Config::parse_field_value("task.scopes", "[\"web\", \"api\"]").unwrap();
        assert!(value.as_array().is_some());
    }

    #[test]
    fn test_is_valid_path() {
        assert!(Config::is_valid_path("git.username"));
        assert!(Config::is_valid_path("log.console"));
        assert!(!Config::is_valid_path("invalid.key"));
        assert!(!Config::is_valid_path("git.invalid"));
    }

    #[test]
    fn test_access_value_by_path() {
        let json = serde_json::json!({
            "git": {
                "username": "test",
                "password": "secret"
            },
            "log": {
                "console": true
            }
        });

        let value = access_value_by_path(&json, "git.username").unwrap();
        assert_eq!(value.as_str().unwrap(), "test");

        let value = access_value_by_path(&json, "log.console").unwrap();
        assert!(value.as_bool().unwrap());

        // Test invalid path
        assert!(access_value_by_path(&json, "invalid.path").is_none());
    }

    #[test]
    fn test_determine_value_type() {
        assert_eq!(determine_value_type(&JsonValue::Bool(true)), "boolean");
        assert_eq!(
            determine_value_type(&JsonValue::Number(42.into())),
            "integer"
        );
        assert_eq!(
            determine_value_type(&JsonValue::String("test".to_string())),
            "string"
        );
        assert_eq!(determine_value_type(&JsonValue::Array(vec![])), "array");
    }
}
