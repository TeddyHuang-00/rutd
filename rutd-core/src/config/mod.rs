pub mod git;
pub mod logging;
pub mod path;
pub mod task;

use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
pub use git::GitConfig;
use log::info;
pub use logging::LogConfig;
pub use path::PathConfig;
use serde::{Deserialize, Serialize};
use shellexpand::tilde;
pub use task::TaskConfig;

/// Main configuration structure that holds all configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Path configuration
    pub path: PathConfig,
    /// Git configuration
    pub git: GitConfig,
    /// Log configuration
    pub log: LogConfig,
    /// Task configuration
    pub task: TaskConfig,
}

impl Config {
    /// Get configurations
    ///
    /// Configurations are loaded in the following precedence:
    /// 1. Environment variables
    /// 2. Configuration file
    /// 3. Default values
    pub fn new() -> Result<Self> {
        // Get the package name from the compiled binary name
        // This should be resolved to `rutd` for all binaries
        // (e.g. `rutd`, `rutd-cli`, `rutd-tui`, etc.)
        let pkg_name = env!("CARGO_PKG_NAME")
            .split_once('-')
            .map_or(env!("CARGO_PKG_NAME"), |(name, _)| name)
            .to_string();
        let config_file = format!("~/.{}/config.toml", pkg_name);
        let env_var_prefix = pkg_name.to_uppercase() + "_";

        Ok(Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file(tilde(&config_file).as_ref()).nested())
            .merge(Env::prefixed(&env_var_prefix).map(|key| {
                // Convert environment variable keys to a format that matches the config
                // structure For example, "RUTD_PATH__ROOT_DIR" becomes
                // "path.root_dir"
                let key = key
                    .as_str()
                    // Use double underscore to separate nested keys
                    .replace("__", ".");
                info!("Loading environment variable: {key}");
                key.into()
            }))
            .extract()?)
    }
}
