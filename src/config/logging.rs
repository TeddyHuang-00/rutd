use serde::{Deserialize, Serialize};

/// Default maximum number of lines to keep in log file
pub const DEFAULT_MAX_LOG_HISTORY: usize = 100;

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Maximum number of lines to keep in log file
    ///
    /// Set to 0 to disable log rotation
    pub max_history: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            max_history: DEFAULT_MAX_LOG_HISTORY,
        }
    }
}
