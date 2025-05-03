use serde::{Deserialize, Serialize};

/// Default maximum number of lines to keep in log file
pub const DEFAULT_MAX_LOG_HISTORY: usize = 100;

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Maximum number of lines to keep in log file
    pub max_log_history: Option<usize>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_log_history: Some(DEFAULT_MAX_LOG_HISTORY),
        }
    }
}
