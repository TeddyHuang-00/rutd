pub mod config;
pub mod display;
pub mod git;
pub mod logging;
pub mod task;

// Re-export commonly used items
pub use config::Config;
pub use display::Display;
pub use git::MergeStrategy;
pub use task::{
    DateRange, Priority, SortCriteria, SortOptions, SortOrder, Task, TaskManager, TaskStatus,
};
