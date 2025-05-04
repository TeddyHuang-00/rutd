use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate, TimeZone, Weekday};
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
        short = 'c', long,
        add = ArgValueCompleter::new(complete::complete_scope)
    ))]
    pub scope: Option<String>,

    /// Filter by type
    #[cfg_attr(feature = "cli", arg(
        short, long,
        value_name = "type",
        add = ArgValueCompleter::new(complete::complete_type)
    ))]
    pub task_type: Option<String>,

    /// Filter by status
    #[cfg_attr(feature = "cli", arg(value_enum, short, long))]
    pub status: Option<TaskStatus>,

    /// Filter by creation date range
    #[cfg_attr(feature = "cli", arg(
        short = 'a', long,
        value_parser = parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    ))]
    pub creation_time: Option<DateRange>,

    /// Filter by last update date range
    #[cfg_attr(feature = "cli", arg(
        short, long,
        value_parser = parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    ))]
    pub update_time: Option<DateRange>,

    /// Filter by completion date range, including cancelled tasks
    #[cfg_attr(feature = "cli", arg(
        short = 'd', long,
        value_parser = parse_date_range,
        allow_hyphen_values = true,
        long_help = DATE_LONG_HELP
    ))]
    pub completion_time: Option<DateRange>,

    /// Enable fuzzy matching for description
    #[cfg_attr(feature = "cli", arg(short, long))]
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
                    "Date does not exist: {year:04}/{month:02}/{day:02}"
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
                .context(format!("Date does not exist: {year:04}/01/01"))?
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
    let mut date = now
        .clone()
        .checked_sub_months(Months::new(offset_months))
        .unwrap()
        .checked_sub_days(Days::new(offset_days.into()))
        .unwrap();

    // Round the date based on the last unit if not in exact mode
    if !exact {
        date = match last_unit {
            // Clear time part
            'd' => DateTime::<Tz>::from_naive_utc_and_offset(
                date.date_naive().and_hms_opt(0, 0, 0).unwrap(),
                now.offset().to_owned(),
            ),
            // Set to the first day of the week and clear time part
            'w' => {
                let first_day_of_week = date.date_naive().week(Weekday::Mon);
                DateTime::<Tz>::from_naive_utc_and_offset(
                    first_day_of_week.first_day().and_hms_opt(0, 0, 0).unwrap(),
                    now.offset().to_owned(),
                )
            }
            // Set to the first day of the month and clear time part
            'm' => DateTime::<Tz>::from_naive_utc_and_offset(
                date.date_naive()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                now.offset().to_owned(),
            ),
            // Set to the first day of the year and clear time part
            'y' => DateTime::<Tz>::from_naive_utc_and_offset(
                date.date_naive()
                    .with_month(1)
                    .unwrap()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                now.offset().to_owned(),
            ),
            _ => unreachable!(),
        };
    }

    // Adjust the end date to the last moment of the cycle if not exact
    if is_end && !exact {
        let naive_date = date.naive_local();
        let adjusted_date = match last_unit {
            'd' => naive_date.checked_add_days(Days::new(1)),
            'w' => naive_date.checked_add_days(Days::new(7)),
            'm' => naive_date.checked_add_months(Months::new(1)),
            'y' => naive_date.checked_add_months(Months::new(12)),
            _ => unreachable!(),
        }
        .unwrap();

        date = DateTime::<Tz>::from_naive_utc_and_offset(adjusted_date, now.offset().to_owned());
    }

    Ok(date)
}

// Parse date range from string for clap
fn parse_date_range(range_str: &str) -> Result<DateRange> {
    DateRange::try_from(range_str)
}
