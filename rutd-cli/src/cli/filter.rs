use clap::Args;
use clap_complete::engine::ArgValueCompleter;
use rutd_core::task::{DateRange, Filter, Priority, TaskStatus};

use crate::{completer, parser};

const DATE_LONG_HELP: &str = "
Date range format: [<date>]..[<date>] or <date>

<date> format:
1. Absolute: YYYY/MM/DD, YYYY/MM, YYYY.
2. Relative: [<num>]d, [<num>]w, [<num>]m, [<num>]y; d for days, w for
   weeks, m for months, y for years. <num> defaults to 0, meaning the
   current cycle.

Relative format also supports:
1. '+<date>': exact offset from the current date. Default behavior is to
   round the date to the beginning of the cycle if used as start or the
   end of the cycle if used as end.
2. Combinations: e.g., '5d3w', '+1m2d', etc. The last date
   unit is used to determine the cycle for rounding in non-exact mode.";

/// CLI-specific filter options for task queries with parsing logic
#[derive(Debug, Clone, Default, Args)]
pub struct FilterOptions {
    /// Filter by priority
    #[arg(
        short, long,
        add = ArgValueCompleter::new(completer::complete_priority)
    )]
    pub priority: Option<Priority>,

    /// Filter by scope (project name)
    #[arg(
        short = 's', long = "scope",
        value_name = "SCOPE",
        add = ArgValueCompleter::new(completer::complete_scope)
    )]
    pub task_scope: Option<String>,

    /// Filter by type
    #[arg(
        short = 't', long = "type",
        value_name = "TYPE",
        add = ArgValueCompleter::new(completer::complete_type)
    )]
    pub task_type: Option<String>,

    /// Filter by status
    #[arg(
        short = 'S', long,
        add = ArgValueCompleter::new(completer::complete_status)
    )]
    pub status: Option<TaskStatus>,

    /// Filter by creation date range
    #[arg(
        short = 'c', long = "created",
        value_name = "DATERANGE",
        value_parser = parser::parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    )]
    pub creation_time: Option<DateRange>,

    /// Filter by last update date range
    #[arg(
        short = 'u', long = "updated",
        value_name = "DATERANGE",
        value_parser = parser::parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    )]
    pub update_time: Option<DateRange>,

    /// Filter by completion date range, including cancelled tasks
    #[arg(
        short = 'C', long = "completed",
        value_name = "DATERANGE",
        value_parser = parser::parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    )]
    pub completion_time: Option<DateRange>,

    /// Enable fuzzy matching for description
    #[arg(short, long, value_name = "DESCRIPTION")]
    pub fuzzy: Option<String>,
}

// Implement From trait to convert CliFilterOptions to FilterOptions
impl From<FilterOptions> for Filter {
    fn from(cli_filter: FilterOptions) -> Self {
        Self {
            priority: cli_filter.priority,
            task_scope: cli_filter.task_scope,
            task_type: cli_filter.task_type,
            status: cli_filter.status,
            creation_time: cli_filter.creation_time,
            update_time: cli_filter.update_time,
            completion_time: cli_filter.completion_time,
            fuzzy: cli_filter.fuzzy,
        }
    }
}

#[cfg(test)]
mod tests {

    // TODO: Add tests for FilterOptions
}
