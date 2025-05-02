pub mod repo;

use std::fmt;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MergeStrategy {
    /// Do not automatically merge (alias: n)
    #[value(alias = "n")]
    None,
    /// Prefer local version (alias: l)
    #[value(alias = "l")]
    Local,
    /// Prefer remote version (alias: r)
    #[value(alias = "r")]
    Remote,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Local => write!(f, "Local"),
            Self::Remote => write!(f, "Remote"),
        }
    }
}
