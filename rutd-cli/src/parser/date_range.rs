use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Days, Local, LocalResult, Months, TimeZone, Weekday};
use rutd_core::DateRange;

// Parse date range from string for clap
pub fn parse_date_range(range_str: &str) -> Result<DateRange, anyhow::Error> {
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

    // Helper function to create a date at a specific year, month, day
    fn create_date(year: i32, month: u32, day: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    #[test]
    fn test_date_range_single_date() {
        // Test with a single date (should be treated as exact day range)
        let range_str = "2023/01/01";
        let result = parse_date_range(range_str);
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
        let result = parse_date_range(range_str);
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
        let result = parse_date_range(range_str);
        assert!(result.is_ok());

        let range = result.unwrap();
        assert!(range.from.is_some());
        assert!(range.to.is_none());

        // Test open-ended range (to only)
        let range_str = "-2023/12/31";
        let result = parse_date_range(range_str);
        assert!(result.is_ok());

        let range = result.unwrap();
        assert!(range.from.is_none());
        assert!(range.to.is_some());
    }

    #[test]
    fn test_relative_date_current_day() {
        // Test relative date for the current day
        let range_str = "d";
        let result = parse_date_range(range_str);
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
        let result = parse_date_range(range_str);
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
        let result = parse_date_range(range_str);
        assert!(result.is_ok());

        // A proper test would compare the actual date calculation
        // but this is sufficient to check that parsing succeeded
    }

    #[test]
    fn test_invalid_date_formats() {
        // Test various invalid formats
        let result = parse_date_range("invalid");
        assert!(result.is_err());

        let result = parse_date_range("2023/13/01"); // Invalid month
        assert!(result.is_err());

        let result = parse_date_range("2023/01/32"); // Invalid day
        assert!(result.is_err());
    }
}
