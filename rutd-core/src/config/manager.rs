use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use toml_edit::{DocumentMut, Item, Table};

use super::{Config, ConfigReflection};

pub struct ConfigManager {
    config_path: String,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let pkg_name = env!("CARGO_PKG_NAME")
            .split_once('-')
            .map_or(env!("CARGO_PKG_NAME"), |(name, _)| name)
            .to_string();
        let config_path = format!("~/.{pkg_name}/config.toml");
        let config_path = shellexpand::tilde(&config_path).into_owned();

        Ok(Self { config_path })
    }

    fn read_config_file(&self) -> Result<String> {
        fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read config file: {}", self.config_path))
    }

    fn validate_config_key(&self, key: &str) -> Result<()> {
        if !Config::is_valid_path(key) {
            anyhow::bail!("Invalid configuration key: {key}");
        }
        Ok(())
    }

    pub fn get_config_value(&self, key: &str) -> Result<String> {
        // Validate the key using reflection
        self.validate_config_key(key)?;

        // First try to get from the config file if it exists
        if Path::new(&self.config_path).exists() {
            let content = self.read_config_file()?;

            if let Ok(doc) = content.parse::<DocumentMut>()
                && let Some(value) = self.get_value_from_file(&doc, key)
            {
                return Ok(value);
            }
        }

        // Fall back to current config using reflection
        let config = Config::new().with_context(|| "Failed to load current configuration")?;
        config.get_field_value(key)
    }

    pub fn set_config_value(&self, key: &str, value: &str) -> Result<()> {
        let config_dir = Path::new(&self.config_path).parent().unwrap();
        fs::create_dir_all(config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;

        let mut doc = if Path::new(&self.config_path).exists() {
            let content = self.read_config_file()?;
            content
                .parse::<DocumentMut>()
                .with_context(|| format!("Failed to parse config file: {}", self.config_path))?
        } else {
            DocumentMut::new()
        };

        self.set_value_in_file(&mut doc, key, value)?;

        fs::write(&self.config_path, doc.to_string())
            .with_context(|| format!("Failed to write config file: {}", self.config_path))?;

        Ok(())
    }

    pub fn unset_config_value(&self, key: &str) -> Result<()> {
        // Validate the key using reflection
        self.validate_config_key(key)?;

        // If config file doesn't exist, there's nothing to unset
        if !Path::new(&self.config_path).exists() {
            return Ok(());
        }

        let content = self.read_config_file()?;

        let mut doc = content
            .parse::<DocumentMut>()
            .with_context(|| format!("Failed to parse config file: {}", self.config_path))?;

        self.remove_value_from_file(&mut doc, key)?;

        fs::write(&self.config_path, doc.to_string())
            .with_context(|| format!("Failed to write config file: {}", self.config_path))?;

        Ok(())
    }

    /// List configuration values showing user-configured values over defaults
    pub fn list_config_values(&self) -> Result<BTreeMap<String, String>> {
        let default_config = Config::default();

        Config::get_field_paths()
            .into_iter()
            .map(|path| {
                let value = self
                    .get_user_configured_value(&path)?
                    .or_else(|| {
                        default_config
                            .get_field_value(&path)
                            .ok()
                            .map(|v| format!("{v} (default)"))
                    })
                    .unwrap_or_else(|| "Unknown".to_string());
                Ok((path, value))
            })
            .collect::<Result<BTreeMap<_, _>>>()
    }

    /// Get a user-configured value (from config file or env vars), returns None
    /// if using default
    fn get_user_configured_value(&self, key: &str) -> Result<Option<String>> {
        // First check config file
        if Path::new(&self.config_path).exists() {
            let content = self.read_config_file()?;

            let config_value = content
                .parse::<DocumentMut>()
                .ok()
                .and_then(|doc| self.get_value_from_file(&doc, key));

            if let Some(value) = config_value {
                return Ok(Some(value));
            }
        }

        // Then check environment variables
        let pkg_name = env!("CARGO_PKG_NAME")
            .split_once('-')
            .map_or(env!("CARGO_PKG_NAME"), |(name, _)| name)
            .to_uppercase();
        let env_var = format!("{pkg_name}_{}", key.replace('.', "__").to_uppercase());

        Ok(std::env::var(&env_var).ok())
    }

    /// Get the effective configuration (for completion and runtime use)
    pub fn get_effective_config(&self) -> Result<Config> {
        Config::new()
    }

    fn set_value_in_file(&self, doc: &mut DocumentMut, key: &str, value: &str) -> Result<()> {
        let parts = key.split('.').collect::<Vec<_>>();

        let &[section, field] = parts.as_slice() else {
            anyhow::bail!("Invalid configuration key format: {key}");
        };

        // Validate using reflection
        self.validate_config_key(key)?;

        if !doc.contains_key(section) {
            doc[section] = Item::Table(Table::new());
        }

        let table = doc[section]
            .as_table_mut()
            .ok_or_else(|| anyhow::anyhow!("Section '{section}' is not a table"))?;

        // Use reflection to parse the value
        let parsed_value = Config::parse_field_value(key, value)?;
        table[field] = Item::Value(parsed_value);

        Ok(())
    }

    fn get_value_from_file(&self, doc: &DocumentMut, key: &str) -> Option<String> {
        let parts = key.split('.').collect::<Vec<_>>();

        let &[section, field] = parts.as_slice() else {
            return None;
        };

        doc.get(section)?
            .as_table()?
            .get(field)?
            .as_value()
            .map(|v| match v {
                toml_edit::Value::String(s) => s.value().to_string(),
                toml_edit::Value::Integer(i) => i.to_string(),
                toml_edit::Value::Boolean(b) => b.to_string(),
                toml_edit::Value::Array(arr) => {
                    let items: Vec<String> = arr
                        .iter()
                        .filter_map(|item| match item {
                            toml_edit::Value::String(s) => Some(s.value().to_string()),
                            _ => None,
                        })
                        .collect();
                    format!("[{}]", items.join(", "))
                }
                _ => v.to_string(),
            })
    }

    fn remove_value_from_file(&self, doc: &mut DocumentMut, key: &str) -> Result<()> {
        let parts = key.split('.').collect::<Vec<_>>();

        let &[section, field] = parts.as_slice() else {
            anyhow::bail!("Invalid configuration key format: {key}");
        };

        if let Some(section_item) = doc.get_mut(section)
            && let Some(table) = section_item.as_table_mut()
        {
            table.remove(field);

            // If the section is now empty, remove it entirely
            if table.is_empty() {
                doc.remove(section);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_get_config_value() {
        let manager = ConfigManager::new().unwrap();

        let result = manager.get_config_value("git.username");
        assert!(result.is_ok());

        let result = manager.get_config_value("invalid.key");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_config_value() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let manager = ConfigManager {
            config_path: config_path.to_str().unwrap().to_string(),
        };

        let result = manager.set_config_value("git.username", "test-user");
        assert!(result.is_ok());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("username = \"test-user\""));
    }

    #[test]
    fn test_list_config_values() {
        let manager = ConfigManager::new().unwrap();

        let result = manager.list_config_values();
        assert!(result.is_ok());

        let values = result.unwrap();
        assert!(!values.is_empty());
        assert!(values.contains_key("git.username"));
        assert!(values.contains_key("path.root_dir"));
    }
}
