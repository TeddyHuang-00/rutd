use std::fmt;

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate, TimeZone, Weekday};
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

// FIXME: Visible aliases for value enum is not yet supported in clap, see
// https://github.com/clap-rs/clap/pull/5480
/// Task Priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Priority {
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

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Urgent => write!(f, "Urgent"),
            Priority::High => write!(f, "High"),
            Priority::Normal => write!(f, "Normal"),
            Priority::Low => write!(f, "Low"),
        }
    }
}

// FIXME: Visible aliases for value enum is not yet supported in clap, see
// https://github.com/clap-rs/clap/pull/5480
/// Task Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum TaskStatus {
    /// Pending (alias: t, p, pending)
    #[value(aliases = ["t", "p", "pending"])]
    Todo,
    /// Finished (alias: d, f, finished)
    #[value(aliases = ["d", "f", "finished"])]
    Done,
    /// Cancelled (alias: a, x, c, cancelled)
    #[value(aliases = ["a", "x", "c", "cancelled"])]
    Aborted,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Aborted => write!(f, "Aborted"),
        }
    }
}

/// Filter options for task queries
#[derive(Debug, Clone, Default, Args)]
pub struct FilterOptions {
    /// Filter by priority
    #[arg(value_enum, short, long)]
    pub priority: Option<Priority>,

    /// Filter by scope (project name)
    #[arg(short = 'c', long)]
    pub scope: Option<String>,

    /// Filter by task type
    #[arg(short, long)]
    pub task_type: Option<String>,

    /// Filter by status
    #[arg(value_enum, short, long)]
    pub status: Option<TaskStatus>,

    /// Filter by date range
    ///
    /// Date range format:
    /// 1. <date>-<date> (e.g., '2023/01/01-2023/01/31' means from Jan 1, 2023 to
    ///    Jan 31, 2023)
    /// 2. <date>- (e.g., '5d-' means from 5 days ago to now)
    /// 3. -<date> (e.g., '-2023' means earlier than end of year 2023)
    /// 4. <date> (e.g., 'w' means current the week)
    ///
    /// Various date formats are supported:
    /// 1. Absolute date: YYYY/MM/DD, YYYY/MM, YYYY.
    /// 2. Relative date: <num>d, <num>w, <num>m, <num>y; num is a non-negative
    ///    integer, d for days, w for weeks, m for months, y for years. If num
    ///    is omitted, it defaults to 0, meaning the current cycle.
    ///
    /// Relative date also supports:
    /// 1. '+<date>' to specify an exact offset from the current date. This
    ///    changes the default behavior of the date range to rounded to the
    ///    beginning of the cycle. e.g., '5d' means 5 days ago, but the time
    ///    part is set to 00:00:00, while '+5d' means 5 days ago at the exact
    ///    time.
    /// 2. Combination of relative dates, e.g., '5d3w', '+1m2d', etc. The last
    ///    date unit is used to determine the cycle for rounding in non-exact
    ///    mode. NOTE: This is WIP, not yet available.
    #[arg(short, long, value_parser = parse_date_range)]
    pub date_range: Option<DateRange>,

    /// Enable fuzzy matching for description
    #[arg(short, long)]
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
    let date = match *date_parts.as_slice() {
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
            let mut date = NaiveDate::from_ymd_opt(year, month, day)
                .context(format!(
                    "Date does not exist: {:04}/{:02}/{:02}",
                    year, month, day
                ))?
                .and_hms_opt(0, 0, 0)
                .unwrap();
            if is_end {
                date = date.checked_add_days(Days::new(1)).unwrap()
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
            let mut date = NaiveDate::from_ymd_opt(year, month, 1)
                .context(format!(
                    "Date does not exist: {:04}/{:02}/{:02}",
                    year, month, 1
                ))?
                .and_hms_opt(0, 0, 0)
                .unwrap();
            if is_end {
                date = date.checked_add_months(Months::new(1)).unwrap()
            }
            date
        }
        [year] => {
            // YYYY format
            let Ok(year) = year.parse::<i32>() else {
                anyhow::bail!("Invalid year in date string: {}", date_str);
            };
            let mut date = NaiveDate::from_ymd_opt(year, 1, 1)
                .context(format!("Date does not exist: {:04}/01/01", year))?
                .and_hms_opt(0, 0, 0)
                .unwrap();
            if is_end {
                date = date.checked_add_months(Months::new(12)).unwrap()
            }
            date
        }
        _ => anyhow::bail!("Invalid date format: {}", date_str),
    };

    Ok(DateTime::<Tz>::from_naive_utc_and_offset(
        date,
        now.offset().to_owned(),
    ))
}

// TODO: Support relative date with multiple units, e.g., 5d3w, +1m2d, etc.
fn parse_relative_date<Tz: TimeZone>(
    date_str: &str,
    now: DateTime<Tz>,
    is_end: bool,
) -> Result<DateTime<Tz>> {
    // Exact delta by using '+'
    let unit = date_str.chars().last().unwrap();
    let exact = date_str.starts_with('+');
    let date_str = &date_str[exact as usize..date_str.len() - 1];

    // Certain number of days/weeks/months/years
    let num = if date_str.is_empty() {
        // Writing 'd', 'w', 'm', or 'y' means 0 days/weeks/months/years,
        // meaning the current cycle
        0
    } else {
        match date_str[..date_str.len() - 1].parse::<i64>() {
            // If a number is specified, it must be positive
            Ok(num) if num.is_positive() => num as u32,
            _ => anyhow::bail!("Invalid number in date string: {}", date_str),
        }
    };
    let date = match unit {
        'd' => now.clone().checked_sub_days(Days::new(num.into())),
        'w' => now.clone().checked_sub_days(Days::new((num * 7).into())),
        'm' => now.clone().checked_sub_months(Months::new(num)),
        'y' => now.clone().checked_sub_months(Months::new(num * 12)),
        _ => unreachable!(),
    }
    .unwrap();
    let mut date = if !exact {
        match unit {
            // Clear time part
            'd' => date.date_naive().and_hms_opt(0, 0, 0).unwrap(),
            // Set to the first day of the week and clear time part
            'w' => {
                let first_day_of_week = date.date_naive().week(Weekday::Mon);
                first_day_of_week.first_day().and_hms_opt(0, 0, 0).unwrap()
            }
            // Set to the first day of the month and clear time part
            'm' => date
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            // Set to the first day of the year and clear time part
            'y' => date
                .date_naive()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            _ => unreachable!(),
        }
    } else {
        date.naive_local()
    };

    // Adjust the end date to the last moment of the cycle if not exact
    if is_end && !exact {
        date = match unit {
            'd' => date.checked_add_days(Days::new(1)),
            'w' => date.checked_add_days(Days::new(7)),
            'm' => date.checked_add_months(Months::new(1)),
            'y' => date.checked_add_months(Months::new(12)),
            _ => unreachable!(),
        }
        .unwrap();
    }

    Ok(DateTime::<Tz>::from_naive_utc_and_offset(
        date,
        now.offset().to_owned(),
    ))
}

// Parse date range from string for clap
fn parse_date_range(range_str: &str) -> Result<DateRange> {
    DateRange::try_from(range_str)
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

/// Task Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: Priority,
    /// Task scope (project name, etc., optional)
    pub scope: Option<String>,
    /// Task type (e.g., feat, fix, other, etc.)
    pub task_type: Option<String>,
    /// Task status
    pub status: TaskStatus,
    /// Task creation time in ISO format
    pub created_at: String,
    /// Task last update time in ISO format
    pub updated_at: Option<String>,
    /// Task completion time in ISO format
    pub completed_at: Option<String>,
    /// Time spent on task in seconds
    pub time_spent: Option<u64>,
}

impl Task {
    pub fn new(
        id: String,
        description: String,
        priority: Priority,
        scope: Option<String>,
        task_type: Option<String>,
    ) -> Self {
        Task {
            id,
            description,
            priority,
            scope,
            task_type,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        }
    }
}
