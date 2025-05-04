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
            CliPriority::Urgent => Self::Urgent,
            CliPriority::High => Self::High,
            CliPriority::Normal => Self::Normal,
            CliPriority::Low => Self::Low,
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
            CliTaskStatus::Todo => Self::Todo,
            CliTaskStatus::Done => Self::Done,
            CliTaskStatus::Aborted => Self::Aborted,
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
            CliMergeStrategy::None => Self::None,
            CliMergeStrategy::Local => Self::Local,
            CliMergeStrategy::Remote => Self::Remote,
        }
    }
}
