#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod display;
pub mod git;
pub mod logging;
pub mod task;

// Re-export commonly used items
#[cfg(feature = "cli")]
pub use cli::complete;
pub use config::Config;
pub use display::Display;
pub use git::MergeStrategy;
pub use task::{Priority, TaskManager, TaskStatus};
