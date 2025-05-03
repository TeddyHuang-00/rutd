use serde::{Deserialize, Serialize};

/// Git configuration for authentication
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitConfig {
    /// Git username for authentication
    pub username: String,
    /// Git password for authentication
    pub password: String,
}
