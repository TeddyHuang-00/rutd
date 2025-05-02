use clap::{Parser, Subcommand};

use crate::task::{Priority, TaskStatus};

/// RuTD - A Rust based To-Do list manager for your rushing to-dos
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new task
    ///
    /// Add a new task to the to-do list, supporting specification of
    /// description, priority, scope, and type
    #[command()]
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
    #[command()]
    List {
        /// Filter by priority
        #[arg(value_enum, short, long)]
        priority: Option<Priority>,

        /// Filter by scope (project name)
        #[arg(short = 'c', long)]
        scope: Option<String>,

        /// Filter by task type
        #[arg(short, long)]
        task_type: Option<String>,

        /// Filter by status
        #[arg(value_enum, short, long)]
        status: Option<TaskStatus>,

        /// Filter by completion date (from)
        #[arg(long)]
        from_date: Option<String>,

        /// Filter by completion date (to)
        #[arg(long)]
        to_date: Option<String>,

        /// Enable fuzzy matching for description
        #[arg(short, long)]
        fuzzy: Option<String>,

        /// Show statistics (counts, total time spent)
        #[arg(long)]
        stats: bool,
    },
    /// Mark task as completed
    ///
    /// Mark the task with the specified ID as completed
    #[command()]
    Done {
        /// Task ID
        id: String,
    },
    /// Edit task description
    ///
    /// Edit the description of the task with the specified ID using the default
    /// editor
    #[command()]
    Edit {
        /// Task ID
        id: String,
    },
    /// Start working on a task
    ///
    /// Mark the task with the specified ID as in progress and start time
    /// tracking
    #[command()]
    Start {
        /// Task ID
        id: String,
    },
    /// Stop working on a task
    ///
    /// Pause time tracking for the task without changing its status
    #[command()]
    Stop {
        /// Task ID
        id: String,
    },
    /// Abort a task
    ///
    /// Mark the task with the specified ID as aborted
    #[command()]
    Abort {
        /// Task ID
        id: String,
    },
    /// Clean tasks
    ///
    /// Remove tasks based on filters
    #[command()]
    Clean {
        /// Filter by priority
        #[arg(value_enum, short, long)]
        priority: Option<Priority>,

        /// Filter by scope (project name)
        #[arg(short = 'c', long)]
        scope: Option<String>,

        /// Filter by task type
        #[arg(short, long)]
        task_type: Option<String>,

        /// Filter by status
        #[arg(value_enum, short, long)]
        status: Option<TaskStatus>,

        /// Filter by completion date (older than n days)
        #[arg(long)]
        older_than: Option<u32>,

        /// Confirm deletion without prompting
        #[arg(short, long)]
        force: bool,
    },
    /// Sync with remote repository
    ///
    /// Fetch, pull and push changes to the remote repository
    #[command()]
    Sync {},
}
