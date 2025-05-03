use serde::{Deserialize, Serialize};

const DEFAULT_SCOPES: [&str; 1] = ["other"];
const DEFAULT_TYPES: [&str; 8] = [
    "build", "chore", "ci", "docs", "style", "refactor", "perf", "test",
];

/// Task configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
