use std::ffi::OsStr;

use clap_complete::CompletionCandidate;
use rutd_core::config::{Config, ConfigReflection};

/// Complete configuration keys for get and set commands
pub fn complete_config_key(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current_str) = current.to_str() else {
        return vec![];
    };

    // Use reflection to get all available config keys
    Config::get_field_paths()
        .into_iter()
        .filter(|key| key.starts_with(current_str))
        .map(CompletionCandidate::new)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    #[test]
    fn test_complete_config_key_empty() {
        let current = OsString::from("");
        let result = complete_config_key(&current);

        // Should return all config keys when input is empty (dynamic discovery finds
        // more than hardcoded)
        assert!(result.len() >= 10);
    }

    #[test]
    fn test_complete_config_key_prefix() {
        let current = OsString::from("git");
        let result = complete_config_key(&current);

        // Should return only git-related keys
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_complete_config_key_full_match() {
        let current = OsString::from("log.console");
        let result = complete_config_key(&current);

        // Should return the exact match
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_complete_config_key_no_match() {
        let current = OsString::from("invalid");
        let result = complete_config_key(&current);

        // Should return no matches
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_complete_config_key_path_prefix() {
        let current = OsString::from("path");
        let result = complete_config_key(&current);

        // Should return all path-related keys (dynamic discovery may find more than
        // hardcoded)
        assert!(result.len() >= 4);
    }

    #[test]
    fn test_complete_config_key_task_prefix() {
        let current = OsString::from("task.");
        let result = complete_config_key(&current);

        // Should return all task-related keys (dynamic discovery finds more than
        // hardcoded)
        assert!(result.len() >= 2);
    }
}
