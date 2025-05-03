use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use shellexpand::tilde;

/// Default path for log file
pub const DEFAULT_LOG_FILE: &str = "~/.rutd/rutd.log";

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Log file path
    log_file: Option<PathBuf>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_file: Some(PathBuf::from(tilde(DEFAULT_LOG_FILE).as_ref())),
        }
    }
}

impl GeneralConfig {
    /// Get the log file path
    pub fn log_file(&self) -> Option<PathBuf> {
        self.log_file.clone()
    }
}
