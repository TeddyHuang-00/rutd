use clap::ValueEnum;
use rutd_core::{
    git::MergeStrategy,
    task::{Priority, TaskStatus},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliPriority {
    Urgent,
    High,
    Normal,
    Low,
}

impl From<CliPriority> for Priority {
    fn from(cli_priority: CliPriority) -> Self {
        match cli_priority {
            CliPriority::Urgent => Priority::Urgent,
            CliPriority::High => Priority::High,
            CliPriority::Normal => Priority::Normal,
            CliPriority::Low => Priority::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliTaskStatus {
    Todo,
    Done,
    Aborted,
}

impl From<CliTaskStatus> for TaskStatus {
    fn from(cli_status: CliTaskStatus) -> Self {
        match cli_status {
            CliTaskStatus::Todo => TaskStatus::Todo,
            CliTaskStatus::Done => TaskStatus::Done,
            CliTaskStatus::Aborted => TaskStatus::Aborted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliMergeStrategy {
    None,
    Local,
    Remote,
}

impl From<CliMergeStrategy> for MergeStrategy {
    fn from(cli_strategy: CliMergeStrategy) -> Self {
        match cli_strategy {
            CliMergeStrategy::None => MergeStrategy::None,
            CliMergeStrategy::Local => MergeStrategy::Local,
            CliMergeStrategy::Remote => MergeStrategy::Remote,
        }
    }
}
