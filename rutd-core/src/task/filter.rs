use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Days, Local, Months, TimeZone, Weekday, offset::LocalResult};
#[cfg(feature = "cli")]
use clap::Args;
#[cfg(feature = "cli")]
use clap_complete::engine::ArgValueCompleter;

use super::{Priority, TaskStatus};
#[cfg(feature = "cli")]
use crate::complete;

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
/// Filter options for task queries
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "cli", derive(Args))]
pub struct FilterOptions {
    /// Filter by priority
    #[cfg_attr(feature = "cli", arg(value_enum, short, long))]
    pub priority: Option<Priority>,

    /// Filter by scope (project name)
    #[cfg_attr(feature = "cli", arg(
        short = 'c', long = "scope",
        value_name = "SCOPE",
        add = ArgValueCompleter::new(complete::complete_scope)
    ))]
    pub task_scope: Option<String>,

    /// Filter by type
    #[cfg_attr(feature = "cli", arg(
        short = 't', long = "type",
        value_name = "TYPE",
        add = ArgValueCompleter::new(complete::complete_type)
    ))]
    pub task_type: Option<String>,

    /// Filter by status
    #[cfg_attr(feature = "cli", arg(value_enum, short, long))]
    pub status: Option<TaskStatus>,

    /// Filter by creation date range
    #[cfg_attr(feature = "cli", arg(
        short = 'a', long = "added",
        value_name = "DATERANGE",
        value_parser = parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    ))]
    pub creation_time: Option<DateRange>,

    /// Filter by last update date range
    #[cfg_attr(feature = "cli", arg(
        short = 'u', long = "updated",
        value_name = "DATERANGE",
        value_parser = parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    ))]
    pub update_time: Option<DateRange>,

    /// Filter by completion date range, including cancelled tasks
    #[cfg_attr(feature = "cli", arg(
        short = 'd', long = "done",
        value_name = "DATERANGE",
        value_parser = parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    ))]
    pub completion_time: Option<DateRange>,

    /// Enable fuzzy matching for description
    #[cfg_attr(feature = "cli", arg(short, long, value_name = "DESCRIPTION"))]
    pub fuzzy: Option<String>,
}

/// DateRange struct for robust date parsing
#[derive(Debug, Clone, Default)]
pub struct DateRange {
    /// Start date limit (None if no lower bound)
    pub from: Option<DateTime<Local>>,
    /// End date limit (None if no upper bound)
    pub to: Option<DateTime<Local>>,
}

impl TryFrom<&str> for DateRange {
    type Error = anyhow::Error;

    fn try_from(range_str: &str) -> Result<Self> {
        let parts: Vec<&str> = range_str.split('-').collect();
        let now = Local::now();
        match *parts.as_slice() {
            // Single date - treat as exact day range
            [start] => {
                let from = Some(parse_date(start, now, false)?);
                let to = Some(parse_date(start, now, true)?);
                Ok(DateRange { from, to })
            }
            // Start-end range
            [start, end] => {
                let from = if start.is_empty() {
                    None
                } else {
                    Some(parse_date(start, now, false)?)
                };
                let to = if end.is_empty() {
                    None
                } else {
                    Some(parse_date(end, now, true)?)
                };
                Ok(DateRange { from, to })
            }
            _ => anyhow::bail!("Invalid date range format: {}", range_str),
        }
    }
}

/// Try parsing the date string from the current date
fn parse_date<Tz: TimeZone>(
    date_str: &str,
    now: DateTime<Tz>,
    is_end: bool,
) -> Result<DateTime<Tz>> {
    // Trim whitespace
    let date_str = date_str.trim();

    if date_str.is_empty() {
        anyhow::bail!("Empty date string")
    }

    // TODO: Add config option to specify the first day of the week
    // Check if is a relative date, e.g., 5d, 3w, 2m, 1y, d, w, m, y
    if "dwmy".contains(date_str.chars().last().unwrap()) {
        return parse_relative_date(date_str, now, is_end);
    }

    // Otherwise, treat it as an absolute date
    parse_absolute_date(date_str, now, is_end)
}

fn parse_absolute_date<Tz: TimeZone>(
    date_str: &str,
    now: DateTime<Tz>,
    is_end: bool,
) -> Result<DateTime<Tz>> {
    // Check if the date string is a absolute date in the format YYYY/MM/DD or
    // YYYY/MM or YYYY
    let date_parts = date_str.split('/').collect::<Vec<_>>();
    let datetime = match *date_parts.as_slice() {
        // YYYY/MM/DD format
        [year, month, day] => {
            let Ok(year) = year.parse::<i32>() else {
                anyhow::bail!("Invalid year in date string: {}", date_str);
            };
            let Ok(month) = month.parse::<u32>() else {
                anyhow::bail!("Invalid month in date string: {}", date_str);
            };
            let Ok(day) = day.parse::<u32>() else {
                anyhow::bail!("Invalid day in date string: {}", date_str);
            };
            let mut date = unwrap_ambiguous_date(
                now.timezone().with_ymd_and_hms(year, month, day, 0, 0, 0),
                is_end,
            )?;
            if is_end {
                date = date
                    .checked_add_days(Days::new(1))
                    .context(format!("Failed to add day to date: {date_str}"))?;
            }
            date
        }
        [year, month] => {
            // YYYY/MM format
            let Ok(year) = year.parse::<i32>() else {
                anyhow::bail!("Invalid year in date string: {}", date_str);
            };
            let Ok(month) = month.parse::<u32>() else {
                anyhow::bail!("Invalid month in date string: {}", date_str);
            };
            let mut date = unwrap_ambiguous_date(
                now.timezone().with_ymd_and_hms(year, month, 1, 0, 0, 0),
                is_end,
            )?;
            if is_end {
                date = date
                    .checked_add_months(Months::new(1))
                    .context(format!("Failed to add month to date: {date_str}"))?;
            }
            date
        }
        [year] => {
            // YYYY format
            let Ok(year) = year.parse::<i32>() else {
                anyhow::bail!("Invalid year in date string: {}", date_str);
            };
            let mut date = unwrap_ambiguous_date(
                now.timezone().with_ymd_and_hms(year, 1, 1, 0, 0, 0),
                is_end,
            )?;
            if is_end {
                date = date
                    .checked_add_months(Months::new(12))
                    .context(format!("Failed to add year to date: {date_str}"))?;
            }
            date
        }
        _ => anyhow::bail!("Invalid date format: {}", date_str),
    };

    Ok(datetime)
}

fn parse_relative_date<Tz: TimeZone>(
    date_str: &str,
    now: DateTime<Tz>,
    is_end: bool,
) -> Result<DateTime<Tz>> {
    // Check if exact mode (starts with '+')
    let exact = date_str.starts_with('+');
    let date_str = if exact { &date_str[1..] } else { date_str };

    // Parse multiple units
    let mut remaining = date_str;
    let mut offset_days = 0;
    let mut offset_months = 0;
    let mut last_unit = 'd'; // Default unit

    // Regex would be cleaner but avoiding additional dependencies
    while !remaining.is_empty() {
        // Find the next unit (d, w, m, y)
        let Some(pos) = remaining.find(|c| "dwmy".contains(c)) else {
            anyhow::bail!("Missing unit (d/w/m/y): {}", remaining);
        };

        let unit = remaining.chars().nth(pos).unwrap();
        let num_str = &remaining[..pos];

        // Parse the number (empty means 0, like in 'd', 'w')
        let num = if num_str.is_empty() {
            0
        } else {
            match num_str.parse::<u32>() {
                Ok(num) => num,
                _ => anyhow::bail!("Invalid number in date component: {}", num_str),
            }
        };

        // Accumulate the offset
        match unit {
            'd' => offset_days += num,
            'w' => offset_days += num * 7,
            'm' => offset_months += num,
            'y' => offset_months += num * 12,
            _ => unreachable!(),
        }

        // Update last unit and remaining string
        last_unit = unit;
        remaining = &remaining[pos + 1..];
    }

    // Calculate the date by subtracting the accumulated offset
    let mut datetime = now
        .clone()
        .checked_sub_months(Months::new(offset_months))
        .context(format!(
            "Failed to subtract {offset_months} months from date: {date_str}"
        ))?
        .checked_sub_days(Days::new(offset_days.into()))
        .context(format!(
            "Failed to subtract {offset_days} days from date: {date_str}"
        ))?;

    // Round the date based on the last unit if not in exact mode
    if !exact {
        let date = match last_unit {
            // Clear time part
            'd' => datetime.date_naive(),
            // Set to the first day of the week and clear time part
            'w' => {
                let first_day_of_week = datetime.date_naive().week(Weekday::Mon);
                first_day_of_week.first_day()
            }
            // Set to the first day of the month and clear time part
            'm' => datetime.date_naive().with_day(1).unwrap(),
            'y' => datetime
                .date_naive()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap(),
            _ => unreachable!(),
        };
        datetime = unwrap_ambiguous_date(
            now.timezone()
                .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0),
            is_end,
        )?;
        // Adjust the end date to the last moment of the cycle if not exact
        if is_end {
            datetime = match last_unit {
                'd' => datetime
                    .checked_add_days(Days::new(1))
                    .context(format!("Failed to add day to date: {date_str}"))?,
                'w' => datetime
                    .checked_add_days(Days::new(7))
                    .context(format!("Failed to add week to date: {date_str}"))?,
                'm' => datetime
                    .checked_add_months(Months::new(1))
                    .context(format!("Failed to add month to date: {date_str}"))?,
                'y' => datetime
                    .checked_add_months(Months::new(12))
                    .context(format!("Failed to add year to date: {date_str}"))?,
                _ => unreachable!(),
            };
        }
    }

    Ok(datetime)
}

// Parse date range from string for clap
fn parse_date_range(range_str: &str) -> Result<DateRange> {
    DateRange::try_from(range_str)
}

fn unwrap_ambiguous_date<Tz: TimeZone>(
    date: LocalResult<DateTime<Tz>>,
    is_end: bool,
) -> Result<DateTime<Tz>> {
    match date {
        LocalResult::Single(datetime) => Ok(datetime),
        LocalResult::Ambiguous(earliest, latest) => Ok(if is_end { latest } else { earliest }),
        LocalResult::None => anyhow::bail!("Date does not exist"),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Timelike};

    use super::*;
    use crate::task::{Priority, Task, TaskStatus};

    // Helper function to create a date at a specific year, month, day
    fn create_date(year: i32, month: u32, day: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    // Helper function to create a default filter
    fn create_default_filter() -> FilterOptions {
        FilterOptions::default()
    }

    struct EasyTask<'a>(
        &'a str,
        &'a str,
        Priority,
        Option<&'a str>,
        Option<&'a str>,
        TaskStatus,
        &'a str,
        Option<&'a str>,
        Option<&'a str>,
    );

    // Helper function to create a test task with specific properties
    fn create_test_task(task: EasyTask) -> Task {
        let EasyTask(
            id,
            description,
            priority,
            scope,
            task_type,
            status,
            created_at,
            updated_at,
            completed_at,
        ) = task;
        Task {
            id: id.to_string(),
            description: description.to_string(),
            priority,
            scope: scope.map(|s| s.to_string()),
            task_type: task_type.map(|t| t.to_string()),
            status,
            created_at: created_at.to_string(),
            updated_at: updated_at.map(|s| s.to_string()),
            completed_at: completed_at.map(|s| s.to_string()),
            time_spent: None,
        }
    }

    // Helper function to apply a filter to a list of tasks
    fn apply_filter(tasks: &[Task], filter: &FilterOptions) -> Vec<Task> {
        tasks
            .iter()
            .filter(|task| {
                // Filter by priority
                if let Some(priority) = filter.priority {
                    if task.priority != priority {
                        return false;
                    }
                }

                // Filter by scope
                if let Some(ref scope) = filter.task_scope {
                    match &task.scope {
                        Some(task_scope) if task_scope == scope => {}
                        _ => return false,
                    }
                }

                // Filter by type
                if let Some(ref task_type) = filter.task_type {
                    match &task.task_type {
                        Some(tt) if tt == task_type => {}
                        _ => return false,
                    }
                }

                // Filter by status
                if let Some(status) = filter.status {
                    if task.status != status {
                        return false;
                    }
                }

                // Filter by creation time
                if let Some(ref date_range) = filter.creation_time {
                    let created_at = match DateTime::parse_from_rfc3339(&task.created_at) {
                        Ok(dt) => dt.with_timezone(&Local),
                        Err(_) => return false, // Skip tasks with invalid dates
                    };

                    if let Some(from) = date_range.from {
                        if created_at < from {
                            return false;
                        }
                    }

                    if let Some(to) = date_range.to {
                        if created_at >= to {
                            return false;
                        }
                    }
                }

                // Filter by update time
                if let Some(ref date_range) = filter.update_time {
                    match &task.updated_at {
                        Some(updated_at_str) => {
                            let updated_at = match DateTime::parse_from_rfc3339(updated_at_str) {
                                Ok(dt) => dt.with_timezone(&Local),
                                Err(_) => return false, // Skip tasks with invalid dates
                            };

                            if let Some(from) = date_range.from {
                                if updated_at < from {
                                    return false;
                                }
                            }

                            if let Some(to) = date_range.to {
                                if updated_at >= to {
                                    return false;
                                }
                            }
                        }
                        None => return false, // No update time, doesn't match filter
                    }
                }

                // Filter by completion time
                if let Some(ref date_range) = filter.completion_time {
                    match &task.completed_at {
                        Some(completed_at_str) => {
                            let completed_at = match DateTime::parse_from_rfc3339(completed_at_str)
                            {
                                Ok(dt) => dt.with_timezone(&Local),
                                Err(_) => return false, // Skip tasks with invalid dates
                            };

                            if let Some(from) = date_range.from {
                                if completed_at < from {
                                    return false;
                                }
                            }

                            if let Some(to) = date_range.to {
                                if completed_at >= to {
                                    return false;
                                }
                            }
                        }
                        None => return false, // No completion time, doesn't match filter
                    }
                }

                // Filter by fuzzy search
                if let Some(ref fuzzy) = filter.fuzzy {
                    if !task
                        .description
                        .to_lowercase()
                        .contains(&fuzzy.to_lowercase())
                    {
                        return false;
                    }
                }

                // Task passed all filters
                true
            })
            .cloned()
            .collect()
    }

    #[test]
    fn test_empty_filter_options() {
        let filter = FilterOptions::default();

        assert!(filter.priority.is_none());
        assert!(filter.task_scope.is_none());
        assert!(filter.task_type.is_none());
        assert!(filter.status.is_none());
        assert!(filter.creation_time.is_none());
        assert!(filter.update_time.is_none());
        assert!(filter.completion_time.is_none());
        assert!(filter.fuzzy.is_none());
    }

    #[test]
    fn test_date_range_single_date() {
        // Test with a single date (should be treated as exact day range)
        let range_str = "2023/01/01";
        let result = DateRange::try_from(range_str);
        assert!(result.is_ok());

        let range = result.unwrap();
        assert!(range.from.is_some());
        assert!(range.to.is_some());

        // From should be start of day
        let from = range.from.unwrap();
        assert_eq!(from.year(), 2023);
        assert_eq!(from.month(), 1);
        assert_eq!(from.day(), 1);
        assert_eq!(from.time().hour(), 0);
        assert_eq!(from.time().minute(), 0);
        assert_eq!(from.time().second(), 0);

        // To should be start of next day
        let to = range.to.unwrap();
        assert_eq!(to.year(), 2023);
        assert_eq!(to.month(), 1);
        assert_eq!(to.day(), 2);
        assert_eq!(to.time().hour(), 0);
        assert_eq!(to.time().minute(), 0);
        assert_eq!(to.time().second(), 0);
    }

    #[test]
    fn test_date_range_start_end() {
        // Test range with start and end dates
        let range_str = "2023/01/01-2023/12/31";
        let result = DateRange::try_from(range_str);
        assert!(result.is_ok());

        let range = result.unwrap();
        assert!(range.from.is_some());
        assert!(range.to.is_some());

        if let Some(from) = range.from {
            assert_eq!(from.date_naive().year(), 2023);
            assert_eq!(from.date_naive().month(), 1);
            assert_eq!(from.date_naive().day(), 1);
        }

        if let Some(to) = range.to {
            assert_eq!(to.date_naive().year(), 2024);
            assert_eq!(to.date_naive().month(), 1);
            assert_eq!(to.date_naive().day(), 1);
        }
    }

    #[test]
    fn test_date_range_open_ended() {
        // Test open-ended range (from only)
        let range_str = "2023/01/01-";
        let result = DateRange::try_from(range_str);
        assert!(result.is_ok());

        let range = result.unwrap();
        assert!(range.from.is_some());
        assert!(range.to.is_none());

        // Test open-ended range (to only)
        let range_str = "-2023/12/31";
        let result = DateRange::try_from(range_str);
        assert!(result.is_ok());

        let range = result.unwrap();
        assert!(range.from.is_none());
        assert!(range.to.is_some());
    }

    #[test]
    fn test_relative_date_current_day() {
        // Test relative date for the current day
        let range_str = "d";
        let result = DateRange::try_from(range_str);
        assert!(result.is_ok());

        let range = result.unwrap();
        let now = Local::now();

        if let Some(from) = range.from {
            assert_eq!(from.date_naive().year(), now.date_naive().year());
            assert_eq!(from.date_naive().month(), now.date_naive().month());
            assert_eq!(from.date_naive().day(), now.date_naive().day());
            assert_eq!(from.time().hour(), 0);
            assert_eq!(from.time().minute(), 0);
            assert_eq!(from.time().second(), 0);
        }

        if let Some(to) = range.to {
            // "to" should be start of next day
            let next_day = now.date_naive().succ_opt().unwrap();
            assert_eq!(to.date_naive().year(), next_day.year());
            assert_eq!(to.date_naive().month(), next_day.month());
            assert_eq!(to.date_naive().day(), next_day.day());
            assert_eq!(to.time().hour(), 0);
            assert_eq!(to.time().minute(), 0);
            assert_eq!(to.time().second(), 0);
        }
    }

    #[test]
    fn test_date_range_with_specific_days_ago() {
        // Test with specific number of days ago
        let range_str = "7d";
        let result = DateRange::try_from(range_str);
        assert!(result.is_ok());

        // Would require more complex testing to verify exact values
        // since it depends on the current date
    }

    #[test]
    fn test_parse_absolute_date() {
        // Test various absolute date formats
        let now = create_date(2025, 5, 1);

        // YYYY/MM/DD format
        let result = parse_absolute_date("2023/01/01", now, false);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.date_naive().year(), 2023);
        assert_eq!(date.date_naive().month(), 1);
        assert_eq!(date.date_naive().day(), 1);

        // YYYY/MM format
        let result = parse_absolute_date("2023/02", now, false);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.date_naive().year(), 2023);
        assert_eq!(date.date_naive().month(), 2);
        assert_eq!(date.date_naive().day(), 1);

        // YYYY format
        let result = parse_absolute_date("2023", now, false);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.date_naive().year(), 2023);
        assert_eq!(date.date_naive().month(), 1);
        assert_eq!(date.date_naive().day(), 1);

        // Test invalid formats
        let result = parse_absolute_date("invalid", now, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_exact_mode_relative_date() {
        // Test exact mode (with + prefix)
        let range_str = "+3d";
        let result = DateRange::try_from(range_str);
        assert!(result.is_ok());

        // A proper test would compare the actual date calculation
        // but this is sufficient to check that parsing succeeded
    }

    #[test]
    fn test_invalid_date_formats() {
        // Test various invalid formats
        let result = DateRange::try_from("invalid");
        assert!(result.is_err());

        let result = DateRange::try_from("2023/13/01"); // Invalid month
        assert!(result.is_err());

        let result = DateRange::try_from("2023/01/32"); // Invalid day
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_by_priority() {
        // Create test tasks with different priorities
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Urgent task",
                Priority::Urgent,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "High priority task",
                Priority::High,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Normal priority task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-4",
                "Low priority task",
                Priority::Low,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by priority
        let mut filter = create_default_filter();

        filter.priority = Some(Priority::Urgent);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.priority = Some(Priority::High);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.priority = Some(Priority::Normal);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");

        filter.priority = Some(Priority::Low);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-4");
    }

    #[test]
    fn test_filter_by_scope() {
        // Create test tasks with different scopes
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Project A task",
                Priority::Normal,
                Some("project-a"),
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Project B task",
                Priority::Normal,
                Some("project-b"),
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "No scope task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by scope
        let mut filter = create_default_filter();

        filter.task_scope = Some("project-a".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.task_scope = Some("project-b".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.task_scope = Some("nonexistent".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_type() {
        // Create test tasks with different types
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Feature task",
                Priority::Normal,
                None,
                Some("feat"),
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Bug task",
                Priority::Normal,
                None,
                Some("bug"),
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "No type task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by type
        let mut filter = create_default_filter();

        filter.task_type = Some("feat".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.task_type = Some("bug".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.task_type = Some("nonexistent".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_status() {
        // Create test tasks with different statuses
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Todo task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Done task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-01-02T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-3",
                "Aborted task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Aborted,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-01-03T12:00:00+00:00"),
            )),
        ];

        // Test filter by status
        let mut filter = create_default_filter();

        filter.status = Some(TaskStatus::Todo);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.status = Some(TaskStatus::Done);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.status = Some(TaskStatus::Aborted);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");
    }

    #[test]
    fn test_filter_by_creation_time() {
        // Create test tasks with different creation times
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Created in January",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Created in February",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-02-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Created in March",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-03-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by creation time
        let mut filter = create_default_filter();

        // Only January
        filter.creation_time = Some(DateRange::try_from("2023/01").unwrap());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // January through February
        filter.creation_time = Some(DateRange::try_from("2023/01-2023/02").unwrap());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "task-1");
        assert_eq!(filtered[1].id, "task-2");

        // All of 2023
        filter.creation_time = Some(DateRange::try_from("2023").unwrap());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_by_update_time() {
        // Create test tasks with different update times
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Updated in January",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                Some("2023-01-15T12:00:00+00:00"),
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Updated in February",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                Some("2023-02-15T12:00:00+00:00"),
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Never updated",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by update time
        let mut filter = create_default_filter();

        // Only January
        filter.update_time = Some(DateRange::try_from("2023/01").unwrap());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // January through February
        filter.update_time = Some(DateRange::try_from("2023/01-2023/03").unwrap());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "task-1");
        assert_eq!(filtered[1].id, "task-2");
    }

    #[test]
    fn test_filter_by_completion_time() {
        // Create test tasks with different completion times
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Completed in January",
                Priority::Normal,
                None,
                None,
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-01-15T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-2",
                "Completed in February",
                Priority::Normal,
                None,
                None,
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-02-15T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-3",
                "Not completed",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by completion time
        let mut filter = create_default_filter();

        // Only January
        filter.completion_time = Some(DateRange::try_from("2023/01").unwrap());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // January through February
        filter.completion_time = Some(DateRange::try_from("2023/01-2023/03").unwrap());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "task-1");
        assert_eq!(filtered[1].id, "task-2");
    }

    #[test]
    fn test_filter_by_fuzzy_description() {
        // Create test tasks with different descriptions
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Implement feature A",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Fix bug in module B",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Write documentation",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by fuzzy description
        let mut filter = create_default_filter();

        filter.fuzzy = Some("feature".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.fuzzy = Some("bug".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.fuzzy = Some("doc".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");

        // Case insensitive
        filter.fuzzy = Some("FEATURE".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // No match
        filter.fuzzy = Some("nonexistent".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_combine_multiple_filters() {
        // Create test tasks with various properties
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Urgent feature task",
                Priority::Urgent,
                Some("project-a"),
                Some("feat"),
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                Some("2023-01-15T12:00:00+00:00"),
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "High priority bug fix",
                Priority::High,
                Some("project-a"),
                Some("bug"),
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                Some("2023-02-15T12:00:00+00:00"),
                Some("2023-02-20T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-3",
                "Normal priority documentation",
                Priority::Normal,
                Some("project-b"),
                Some("docs"),
                TaskStatus::Todo,
                "2023-02-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test combining multiple filters
        let mut filter = create_default_filter();

        // Test priority + scope
        filter.priority = Some(Priority::Urgent);
        filter.task_scope = Some("project-a".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // Test priority + scope + type
        filter.priority = Some(Priority::High);
        filter.task_scope = Some("project-a".to_string());
        filter.task_type = Some("bug".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        // Test scope + fuzzy
        filter = FilterOptions::default();
        filter.task_scope = Some("project-b".to_string());
        filter.fuzzy = Some("document".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");

        // Test filter that should match no tasks
        filter = FilterOptions::default();
        filter.priority = Some(Priority::Low);
        filter.task_scope = Some("project-a".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }
}
