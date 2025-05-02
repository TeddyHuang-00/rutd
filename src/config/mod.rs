pub mod git;
pub mod path;

use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
pub use git::GitConfig;
pub use path::PathConfig;
use serde::Deserialize;
use shellexpand::tilde;

/// Main configuration structure that holds all configuration options
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Path configuration
    pub path: PathConfig,
    /// Git configuration
    pub git: GitConfig,
}

impl Config {
    /// Get configurations
    ///
    /// Configurations are loaded in the following precedence:
    /// 1. Environment variables
    /// 2. Configuration file
    /// 3. Default values
    pub fn new() -> Result<Self> {
        let config_file = format!("~/.{}/config.toml", env!("CARGO_PKG_NAME"));

        Ok(Figment::new()
            .merge(Env::prefixed("RUTD_"))
            .merge(Toml::file(tilde(&config_file).as_ref()).nested())
            .extract()?)
    }
}
