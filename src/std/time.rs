// std.time module - Time and date operations
// Requirements: 7.1.7

use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Represents a point in time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    timestamp: u64, // Milliseconds since Unix epoch
}

impl Time {
    /// Get current time
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));

        Self {
            timestamp: duration.as_millis() as u64,
        }
    }

    /// Create time from Unix timestamp (seconds)
    pub fn from_timestamp(seconds: u64) -> Self {
        Self {
            timestamp: seconds * 1000,
        }
    }

    /// Create time from Unix timestamp (milliseconds)
    pub fn from_timestamp_millis(millis: u64) -> Self {
        Self { timestamp: millis }
    }

    /// Get Unix timestamp in seconds
    pub fn timestamp(&self) -> u64 {
        self.timestamp / 1000
    }

    /// Get Unix timestamp in milliseconds
    pub fn timestamp_millis(&self) -> u64 {
        self.timestamp
    }

    /// Add duration to time
    pub fn add(&self, duration: TimeDuration) -> Self {
        Self {
            timestamp: self.timestamp + duration.total_millis(),
        }
    }

    /// Subtract duration from time
    pub fn subtract(&self, duration: TimeDuration) -> Self {
        Self {
            timestamp: self.timestamp.saturating_sub(duration.total_millis()),
        }
    }

    /// Calculate duration between two times
    pub fn duration_since(&self, other: &Time) -> TimeDuration {
        if self.timestamp >= other.timestamp {
            TimeDuration::from_millis(self.timestamp - other.timestamp)
        } else {
            TimeDuration::from_millis(0)
        }
    }

    /// Format time as ISO 8601 string (UTC)
    pub fn format_iso8601(&self) -> String {
        let seconds = self.timestamp / 1000;
        let millis = self.timestamp % 1000;

        // Simple conversion (doesn't account for leap years perfectly)
        let days_since_epoch = seconds / 86400;
        let seconds_today = seconds % 86400;

        let hours = seconds_today / 3600;
        let minutes = (seconds_today % 3600) / 60;
        let secs = seconds_today % 60;

        // Approximate year calculation
        let mut year = 1970;
        let mut remaining_days = days_since_epoch;

        // Rough year calculation
        while remaining_days >= 365 {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days >= days_in_year {
                remaining_days -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }

        // Approximate month and day
        let (month, day) = days_to_month_day(remaining_days as u32, is_leap_year(year));

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            year, month, day, hours, minutes, secs, millis
        )
    }

    /// Format time with custom format string
    pub fn format(&self, format: &str) -> String {
        let seconds = self.timestamp / 1000;
        let millis = self.timestamp % 1000;

        let days_since_epoch = seconds / 86400;
        let seconds_today = seconds % 86400;

        let hours = seconds_today / 3600;
        let minutes = (seconds_today % 3600) / 60;
        let secs = seconds_today % 60;

        // Calculate year, month, day
        let mut year = 1970;
        let mut remaining_days = days_since_epoch;

        while remaining_days >= 365 {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days >= days_in_year {
                remaining_days -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }

        let (month, day) = days_to_month_day(remaining_days as u32, is_leap_year(year));

        // Simple format string replacement
        format
            .replace("%Y", &format!("{:04}", year))
            .replace("%m", &format!("{:02}", month))
            .replace("%d", &format!("{:02}", day))
            .replace("%H", &format!("{:02}", hours))
            .replace("%M", &format!("{:02}", minutes))
            .replace("%S", &format!("{:02}", secs))
            .replace("%f", &format!("{:03}", millis))
    }
}

/// Represents a duration of time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeDuration {
    millis: u64,
}

impl TimeDuration {
    /// Create duration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self { millis }
    }

    /// Create duration from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self {
            millis: secs * 1000,
        }
    }

    /// Create duration from minutes
    pub fn from_mins(mins: u64) -> Self {
        Self {
            millis: mins * 60 * 1000,
        }
    }

    /// Create duration from hours
    pub fn from_hours(hours: u64) -> Self {
        Self {
            millis: hours * 60 * 60 * 1000,
        }
    }

    /// Create duration from days
    pub fn from_days(days: u64) -> Self {
        Self {
            millis: days * 24 * 60 * 60 * 1000,
        }
    }

    /// Get total milliseconds
    pub fn total_millis(&self) -> u64 {
        self.millis
    }

    /// Get total seconds
    pub fn total_secs(&self) -> u64 {
        self.millis / 1000
    }

    /// Get total minutes
    pub fn total_mins(&self) -> u64 {
        self.millis / (60 * 1000)
    }

    /// Get total hours
    pub fn total_hours(&self) -> u64 {
        self.millis / (60 * 60 * 1000)
    }

    /// Get total days
    pub fn total_days(&self) -> u64 {
        self.millis / (24 * 60 * 60 * 1000)
    }

    /// Add two durations
    pub fn add(&self, other: &TimeDuration) -> Self {
        Self {
            millis: self.millis + other.millis,
        }
    }

    /// Subtract duration (saturating at 0)
    pub fn subtract(&self, other: &TimeDuration) -> Self {
        Self {
            millis: self.millis.saturating_sub(other.millis),
        }
    }

    /// Multiply duration by scalar
    pub fn multiply(&self, factor: u64) -> Self {
        Self {
            millis: self.millis * factor,
        }
    }

    /// Divide duration by scalar
    pub fn divide(&self, divisor: u64) -> Self {
        if divisor == 0 {
            Self { millis: 0 }
        } else {
            Self {
                millis: self.millis / divisor,
            }
        }
    }
}

/// Performance timing utilities
pub struct Stopwatch {
    start_time: Instant,
    elapsed: Duration,
    running: bool,
}

impl Stopwatch {
    /// Create new stopwatch
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed: Duration::from_secs(0),
            running: false,
        }
    }

    /// Start the stopwatch
    pub fn start(&mut self) {
        if !self.running {
            self.start_time = Instant::now();
            self.running = true;
        }
    }

    /// Stop the stopwatch
    pub fn stop(&mut self) {
        if self.running {
            self.elapsed += self.start_time.elapsed();
            self.running = false;
        }
    }

    /// Reset the stopwatch
    pub fn reset(&mut self) {
        self.elapsed = Duration::from_secs(0);
        self.running = false;
    }

    /// Restart the stopwatch (reset and start)
    pub fn restart(&mut self) {
        self.reset();
        self.start();
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> TimeDuration {
        let total_elapsed = if self.running {
            self.elapsed + self.start_time.elapsed()
        } else {
            self.elapsed
        };

        TimeDuration::from_millis(total_elapsed.as_millis() as u64)
    }

    /// Check if stopwatch is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Sleep and timing utilities
pub mod sleep {
    use super::*;

    /// Sleep for specified duration
    pub fn sleep(duration: TimeDuration) {
        thread::sleep(Duration::from_millis(duration.total_millis()));
    }

    /// Sleep for milliseconds
    pub fn sleep_millis(millis: u64) {
        thread::sleep(Duration::from_millis(millis));
    }

    /// Sleep for seconds
    pub fn sleep_secs(secs: u64) {
        thread::sleep(Duration::from_secs(secs));
    }
}

/// Time measurement utilities
pub mod measure {
    use super::*;

    /// Measure execution time of a function
    pub fn time<F, R>(f: F) -> (R, TimeDuration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();

        (
            result,
            TimeDuration::from_millis(elapsed.as_millis() as u64),
        )
    }

    /// Measure execution time and print result
    pub fn time_it<F, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = time(f);
        println!("{}: {}ms", name, duration.total_millis());
        result
    }
}

/// Date and time parsing utilities
pub mod parse {
    use super::*;

    /// Parse ISO 8601 timestamp (basic implementation)
    pub fn parse_iso8601(s: &str) -> Result<Time, String> {
        // Expected format: YYYY-MM-DDTHH:MM:SS.sssZ or YYYY-MM-DDTHH:MM:SSZ
        if s.len() < 19 {
            return Err("Invalid ISO 8601 format".to_string());
        }

        let parts: Vec<&str> = s.split('T').collect();
        if parts.len() != 2 {
            return Err("Invalid ISO 8601 format: missing T separator".to_string());
        }

        let date_part = parts[0];
        let time_part = parts[1].trim_end_matches('Z');

        // Parse date
        let date_parts: Vec<&str> = date_part.split('-').collect();
        if date_parts.len() != 3 {
            return Err("Invalid date format".to_string());
        }

        let year: u32 = date_parts[0].parse().map_err(|_| "Invalid year")?;
        let month: u32 = date_parts[1].parse().map_err(|_| "Invalid month")?;
        let day: u32 = date_parts[2].parse().map_err(|_| "Invalid day")?;

        // Validate ranges
        if month < 1 || month > 12 {
            return Err("Invalid month".to_string());
        }
        if day < 1 || day > 31 {
            return Err("Invalid day".to_string());
        }

        // Parse time
        let time_parts: Vec<&str> = time_part.split(':').collect();
        if time_parts.len() < 2 {
            return Err("Invalid time format".to_string());
        }

        let hour: u32 = time_parts[0].parse().map_err(|_| "Invalid hour")?;
        let minute: u32 = time_parts[1].parse().map_err(|_| "Invalid minute")?;

        // Validate time ranges
        if hour > 23 {
            return Err("Invalid hour".to_string());
        }
        if minute > 59 {
            return Err("Invalid minute".to_string());
        }

        let (second, millis) = if time_parts.len() >= 3 {
            let sec_part = time_parts[2];
            if sec_part.contains('.') {
                let sec_parts: Vec<&str> = sec_part.split('.').collect();
                let sec: u32 = sec_parts[0].parse().map_err(|_| "Invalid second")?;
                if sec > 59 {
                    return Err("Invalid second".to_string());
                }
                let ms: u32 = if sec_parts.len() > 1 {
                    let ms_str = &sec_parts[1][..3.min(sec_parts[1].len())];
                    ms_str.parse().unwrap_or(0)
                } else {
                    0
                };
                (sec, ms)
            } else {
                let sec: u32 = sec_part.parse().map_err(|_| "Invalid second")?;
                if sec > 59 {
                    return Err("Invalid second".to_string());
                }
                (sec, 0)
            }
        } else {
            (0, 0)
        };

        // Convert to timestamp (simplified calculation)
        let days_since_epoch = days_since_1970(year, month, day);
        let seconds_since_epoch = days_since_epoch * 86400
            + (hour as u64 * 3600)
            + (minute as u64 * 60)
            + (second as u64);
        let millis_since_epoch = seconds_since_epoch * 1000 + (millis as u64);

        Ok(Time::from_timestamp_millis(millis_since_epoch))
    }

    /// Parse timestamp from string
    pub fn parse_timestamp(s: &str) -> Result<Time, String> {
        let timestamp: u64 = s.parse().map_err(|_| "Invalid timestamp format")?;
        Ok(Time::from_timestamp(timestamp))
    }
}

// Helper functions
fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_to_month_day(day_of_year: u32, is_leap: bool) -> (u32, u32) {
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining_days = day_of_year;
    for (i, &days_in_month) in days_in_months.iter().enumerate() {
        if remaining_days < days_in_month {
            return ((i + 1) as u32, remaining_days + 1);
        }
        remaining_days -= days_in_month;
    }

    (12, 31) // Fallback
}

fn days_since_1970(year: u32, month: u32, day: u32) -> u64 {
    let mut days = 0u64;

    // Add days for complete years
    for y in 1970..year {
        days += if is_leap_year(y as u64) { 366 } else { 365 };
    }

    // Add days for complete months in the current year
    let days_in_months = if is_leap_year(year as u64) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    for i in 0..(month - 1) as usize {
        if i < days_in_months.len() {
            days += days_in_months[i] as u64;
        }
    }

    // Add remaining days
    days += (day - 1) as u64;

    days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_creation() {
        let now = Time::now();
        assert!(now.timestamp() > 0);

        let time = Time::from_timestamp(1609459200); // 2021-01-01 00:00:00 UTC
        assert_eq!(time.timestamp(), 1609459200);

        let time_millis = Time::from_timestamp_millis(1609459200000);
        assert_eq!(time_millis.timestamp_millis(), 1609459200000);
    }

    #[test]
    fn test_time_arithmetic() {
        let time = Time::from_timestamp(1609459200);
        let duration = TimeDuration::from_hours(1);

        let later = time.add(duration);
        assert_eq!(later.timestamp(), 1609459200 + 3600);

        let earlier = later.subtract(duration);
        assert_eq!(earlier.timestamp(), 1609459200);

        let diff = later.duration_since(&earlier);
        assert_eq!(diff.total_secs(), 3600);
    }

    #[test]
    fn test_duration_creation() {
        let duration = TimeDuration::from_secs(60);
        assert_eq!(duration.total_secs(), 60);
        assert_eq!(duration.total_millis(), 60000);

        let duration = TimeDuration::from_mins(5);
        assert_eq!(duration.total_mins(), 5);
        assert_eq!(duration.total_secs(), 300);

        let duration = TimeDuration::from_hours(2);
        assert_eq!(duration.total_hours(), 2);
        assert_eq!(duration.total_mins(), 120);

        let duration = TimeDuration::from_days(1);
        assert_eq!(duration.total_days(), 1);
        assert_eq!(duration.total_hours(), 24);
    }

    #[test]
    fn test_duration_arithmetic() {
        let d1 = TimeDuration::from_secs(30);
        let d2 = TimeDuration::from_secs(20);

        let sum = d1.add(&d2);
        assert_eq!(sum.total_secs(), 50);

        let diff = d1.subtract(&d2);
        assert_eq!(diff.total_secs(), 10);

        let multiplied = d1.multiply(3);
        assert_eq!(multiplied.total_secs(), 90);

        let divided = multiplied.divide(3);
        assert_eq!(divided.total_secs(), 30);
    }

    #[test]
    fn test_stopwatch() {
        let mut stopwatch = Stopwatch::new();
        assert!(!stopwatch.is_running());

        stopwatch.start();
        assert!(stopwatch.is_running());

        sleep::sleep_millis(10);

        stopwatch.stop();
        assert!(!stopwatch.is_running());

        let elapsed = stopwatch.elapsed();
        assert!(elapsed.total_millis() >= 10);

        stopwatch.reset();
        assert_eq!(stopwatch.elapsed().total_millis(), 0);
    }

    #[test]
    fn test_time_formatting() {
        let time = Time::from_timestamp(1609459200); // 2021-01-01 00:00:00 UTC
        let formatted = time.format_iso8601();
        assert!(formatted.starts_with("2021-01-01T00:00:00"));

        let custom = time.format("%Y-%m-%d %H:%M:%S");
        assert!(custom.starts_with("2021-01-01 00:00:00"));
    }

    #[test]
    fn test_measure() {
        let (result, duration) = measure::time(|| {
            sleep::sleep_millis(10);
            42
        });

        assert_eq!(result, 42);
        assert!(duration.total_millis() >= 10);
    }

    #[test]
    fn test_parse_iso8601() {
        let parsed = parse::parse_iso8601("2021-01-01T00:00:00Z").unwrap();
        assert_eq!(parsed.timestamp(), 1609459200);

        let parsed_with_millis = parse::parse_iso8601("2021-01-01T00:00:00.123Z").unwrap();
        assert_eq!(parsed_with_millis.timestamp_millis(), 1609459200123);

        // Test invalid formats
        assert!(parse::parse_iso8601("invalid").is_err());
        assert!(parse::parse_iso8601("2021-01-01").is_err());
    }

    #[test]
    fn test_parse_timestamp() {
        let parsed = parse::parse_timestamp("1609459200").unwrap();
        assert_eq!(parsed.timestamp(), 1609459200);

        assert!(parse::parse_timestamp("invalid").is_err());
    }

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000)); // Divisible by 400
        assert!(is_leap_year(2004)); // Divisible by 4, not by 100
        assert!(!is_leap_year(1900)); // Divisible by 100, not by 400
        assert!(!is_leap_year(2001)); // Not divisible by 4
    }
}
