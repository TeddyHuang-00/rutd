use crate::task::model::{Priority, TaskStatus, TaskType};
use clap::{Parser, Subcommand, ValueEnum};

/// RuTD - A Rust based To-Do list manager for your rushing to-dos
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new task
    ///
    /// Add a new task to the to-do list, supporting specification of description, priority, scope, and type
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

        /// Task type
        #[arg(value_enum, short, long, default_value_t = TaskTypeArg::Feat)]
        task_type: TaskTypeArg,
    },
    /// List tasks
    ///
    /// List tasks in the to-do list, supporting filtering by priority, scope, type, and status
    #[command()]
    List {
        /// Filter by priority
        #[arg(value_enum, short, long)]
        priority: Option<Priority>,

        /// Filter by scope (project name)
        #[arg(short, long)]
        scope: Option<String>,

        /// Filter by task type
        #[arg(value_enum, short, long)]
        task_type: Option<TaskTypeArg>,

        /// Filter by status
        #[arg(value_enum, short, long)]
        status: Option<TaskStatus>,
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
    /// Edit the description of the task with the specified ID using the default editor
    #[command()]
    Edit {
        /// Task ID
        id: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum TaskTypeArg {
    Feat,
    Fix,
    Docs,
    Other,
}

impl From<TaskTypeArg> for TaskType {
    fn from(arg: TaskTypeArg) -> Self {
        match arg {
            TaskTypeArg::Feat => TaskType::Feat,
            TaskTypeArg::Fix => TaskType::Fix,
            TaskTypeArg::Docs => TaskType::Docs,
            TaskTypeArg::Other => TaskType::Other("other".to_string()),
        }
    }
}
