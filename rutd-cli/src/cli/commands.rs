use clap::{Parser, Subcommand};
use rutd_core::{
    git::MergeStrategy,
    task::{Priority, model::FilterOptions},
};

/// RuTD - A Rust based To-Do list manager for your rushing to-dos
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new task
    ///
    /// Add a new task to the to-do list, supporting specification of
    /// description, priority, scope, and type
    #[command(visible_aliases = ["a"])]
    Add {
        /// Task description
        description: String,

        /// Task priority
        #[arg(value_enum, short, long, default_value_t = Priority::Normal)]
        priority: Priority,

        /// Task scope (project name)
        #[arg(short, long)]
        scope: Option<String>,

        /// Task type (e.g., feat, fix, other, etc.)
        #[arg(short, long)]
        task_type: Option<String>,
    },
    /// List tasks
    ///
    /// List tasks in the to-do list, supporting filtering by priority, scope,
    /// type, and status
    #[command(visible_aliases = ["l"])]
    List {
        /// Filter options
        #[command(flatten)]
        filter: FilterOptions,

        /// Show statistics (counts, total time spent)
        #[arg(long)]
        stats: bool,
    },
    /// Mark task as completed
    ///
    /// Mark the task with the specified ID as completed
    #[command(visible_aliases = ["d", "f"])]
    Done {
        /// Task ID
        id: String,
    },
    /// Edit task description
    ///
    /// Edit the description of the task with the specified ID using the default
    /// editor
    #[command(visible_aliases = ["e"])]
    Edit {
        /// Task ID
        id: String,
    },
    /// Start working on a task
    ///
    /// Mark the task with the specified ID as in progress and start time
    /// tracking
    #[command(visible_aliases = ["s"])]
    Start {
        /// Task ID
        id: String,
    },
    /// Stop working on active task
    ///
    /// Pause time tracking for the active task
    #[command(visible_aliases = ["p"])]
    Stop {},
    /// Abort a task
    ///
    /// Mark the task with the specified ID as aborted
    #[command(visible_aliases = ["x", "c"])]
    Abort {
        /// Task ID, if not specified, abort the active task
        id: Option<String>,
    },
    /// Clean tasks
    ///
    /// Remove tasks based on filters
    #[command(visible_aliases = ["purge", "delete", "rm"])]
    Clean {
        /// Filter options
        #[command(flatten)]
        filter: FilterOptions,

        /// Confirm deletion without prompting
        #[arg(short, long)]
        force: bool,
    },
    /// Sync with remote repository
    ///
    /// Fetch, pull and push changes to the remote repository
    #[command(visible_aliases = ["y", "u"])]
    Sync {
        /// Conflict resolution preference when merging
        #[arg(short, long, value_enum, default_value_t = MergeStrategy::None)]
        prefer: MergeStrategy,
    },
    /// Clone a remote repository
    ///
    /// Clone a remote repository to the local tasks directory
    #[command(visible_aliases = ["pull"])]
    Clone {
        /// Remote repository URL to clone
        url: String,
    },
}
