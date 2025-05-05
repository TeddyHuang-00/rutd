use clap::ValueEnum;
use rutd_core::{
    git::MergeStrategy,
    task::{Priority, TaskStatus},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliPriority {
    /// Most urgent (alias: u, 0)
    #[value(aliases = ["u", "0"])]
    Urgent,
    /// High priority (alias: h, 1)
    #[value(aliases = ["h", "1"])]
    High,
    /// Normal priority (alias: n, 2)
    #[value(aliases = ["n", "2"])]
    Normal,
    /// Low priority (alias: l, 3)
    #[value(aliases = ["l", "3"])]
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
    /// Cancelled (alias: a, x, c, cancelled)
    #[value(aliases = ["a", "x", "c", "cancelled"])]
    Aborted,
    /// Pending (alias: t, p, pending)
    #[value(aliases = ["t", "p", "pending"])]
    Todo,
    /// Finished (alias: d, f, finished)
    #[value(aliases = ["d", "f", "finished"])]
    Done,
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
    /// Do not automatically merge (alias: n)
    #[value(aliases = ["n"])]
    None,
    /// Prefer local version (alias: l)
    #[value(aliases = ["l"])]
    Local,
    /// Prefer remote version (alias: r)
    #[value(aliases = ["r"])]
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
