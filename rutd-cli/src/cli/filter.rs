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
    use chrono::{DateTime, Datelike, Local, TimeZone};

    use super::*;

    // Helper function to create a date at a specific year, month, day
    fn create_date(year: i32, month: u32, day: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    #[test]
    fn test_filter_options_to_filter_conversion() {
        // Create a FilterOptions with all fields filled
        let filter_options = FilterOptions {
            priority: Some(Priority::High),
            task_scope: Some("test-scope".to_string()),
            task_type: Some("test-type".to_string()),
            status: Some(TaskStatus::Todo),
            creation_time: Some(DateRange {
                from: Some(create_date(2023, 1, 1)),
                to: Some(create_date(2023, 12, 31)),
            }),
            update_time: Some(DateRange {
                from: Some(create_date(2023, 1, 1)),
                to: None,
            }),
            completion_time: Some(DateRange {
                from: None,
                to: Some(create_date(2023, 12, 31)),
            }),
            fuzzy: Some("test-description".to_string()),
        };

        // Convert to Filter
        let filter: Filter = filter_options.into();

        // Verify all fields are correctly converted
        assert_eq!(filter.priority, Some(Priority::High));
        assert_eq!(filter.task_scope, Some("test-scope".to_string()));
        assert_eq!(filter.task_type, Some("test-type".to_string()));
        assert_eq!(filter.status, Some(TaskStatus::Todo));

        // Check date ranges
        assert!(filter.creation_time.is_some());
        if let Some(date_range) = filter.creation_time {
            assert!(date_range.from.is_some());
            assert!(date_range.to.is_some());
            let from = date_range.from.unwrap();
            let to = date_range.to.unwrap();
            assert_eq!(from.date_naive().year(), 2023);
            assert_eq!(from.date_naive().month(), 1);
            assert_eq!(from.date_naive().day(), 1);
            assert_eq!(to.date_naive().year(), 2023);
            assert_eq!(to.date_naive().month(), 12);
            assert_eq!(to.date_naive().day(), 31);
        }

        assert!(filter.update_time.is_some());
        if let Some(date_range) = filter.update_time {
            assert!(date_range.from.is_some());
            assert!(date_range.to.is_none());
        }

        assert!(filter.completion_time.is_some());
        if let Some(date_range) = filter.completion_time {
            assert!(date_range.from.is_none());
            assert!(date_range.to.is_some());
        }

        assert_eq!(filter.fuzzy, Some("test-description".to_string()));
    }

    #[test]
    fn test_empty_filter_options() {
        // Create an empty FilterOptions
        let filter_options = FilterOptions::default();

        // Convert to Filter
        let filter: Filter = filter_options.into();

        // Verify all fields are None
        assert_eq!(filter.priority, None);
        assert_eq!(filter.task_scope, None);
        assert_eq!(filter.task_type, None);
        assert_eq!(filter.status, None);
        assert!(filter.creation_time.is_none());
        assert!(filter.update_time.is_none());
        assert!(filter.completion_time.is_none());
        assert_eq!(filter.fuzzy, None);
    }

    #[test]
    fn test_partial_filter_options() {
        // Create a FilterOptions with only some fields filled
        let filter_options = FilterOptions {
            priority: Some(Priority::Urgent),
            status: Some(TaskStatus::Done),
            ..Default::default()
        };

        // Convert to Filter
        let filter: Filter = filter_options.into();

        // Verify only specified fields are filled, others are None
        assert_eq!(filter.priority, Some(Priority::Urgent));
        assert_eq!(filter.task_scope, None);
        assert_eq!(filter.task_type, None);
        assert_eq!(filter.status, Some(TaskStatus::Done));
        assert!(filter.creation_time.is_none());
        assert!(filter.update_time.is_none());
        assert!(filter.completion_time.is_none());
        assert_eq!(filter.fuzzy, None);
    }
}
