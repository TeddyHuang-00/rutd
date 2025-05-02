use serde::Deserialize;

/// Git configuration for authentication
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GitConfig {
    /// Git username for authentication
    pub username: Option<String>,
    /// Git password for authentication
    pub password: Option<String>,
}
