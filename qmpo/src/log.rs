//! Simple file-based logging for qmpo.
//!
//! Disabled by default. Enable with `cargo build --features logging`.
//!
//! When enabled, writes logs to `~/.local/share/qmpo/qmpo.log` (Linux),
//! `~/Library/Application Support/qmpo/qmpo.log` (macOS),
//! or `%LOCALAPPDATA%\qmpo\qmpo.log` (Windows).

/// Log an info message. No-op unless built with the `logging` feature.
#[cfg(not(feature = "logging"))]
pub fn info(_message: &str) {}

/// Log an error message. No-op unless built with the `logging` feature.
#[cfg(not(feature = "logging"))]
pub fn error(_message: &str) {}

#[cfg(feature = "logging")]
pub fn info(message: &str) {
    log("INFO", message);
}

#[cfg(feature = "logging")]
pub fn error(message: &str) {
    log("ERROR", message);
}

#[cfg(feature = "logging")]
use std::fs::{self, OpenOptions};
#[cfg(feature = "logging")]
use std::io::Write;
#[cfg(feature = "logging")]
use std::path::PathBuf;

#[cfg(feature = "logging")]
use directories::BaseDirs;

#[cfg(feature = "logging")]
const MAX_LOG_SIZE: u64 = 1024 * 1024; // 1MB

/// Get the log file path.
#[cfg(feature = "logging")]
fn log_path() -> Option<PathBuf> {
    Some(
        BaseDirs::new()?
            .data_local_dir()
            .join("qmpo")
            .join("qmpo.log"),
    )
}

/// Write a log entry. Silently fails if logging is not possible.
#[cfg(feature = "logging")]
fn log(level: &str, message: &str) {
    let Some(path) = log_path() else {
        return;
    };

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Rotate log if too large
    if fs::metadata(&path).is_ok_and(|m| m.len() > MAX_LOG_SIZE) {
        let _ = fs::remove_file(&path);
    }

    // Append to log file
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };

    let timestamp = chrono_lite_timestamp();
    let _ = writeln!(file, "{} [{}] {}", timestamp, level, message);
}

/// Simple timestamp without external chrono dependency.
#[cfg(feature = "logging")]
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple UTC timestamp calculation
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Approximate date calculation (not accounting for leap years perfectly, but good enough for logs)
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_months: [u64; 12] = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}
