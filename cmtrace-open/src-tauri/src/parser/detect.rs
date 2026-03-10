//! Format auto-detection.
//!
//! Examines the first non-empty lines of file content to determine
//! whether it uses CCM, Simple, Timestamped, or Plain text format.
//!
//! Detection strategy (matches CMTrace binary behavior, extended):
//! - Check for `<![LOG[` marker → CCM format
//! - Check for `$$<` delimiter → Simple format
//! - Check for ` type="` substring → CCM format (fallback indicator)
//! - Check for timestamp patterns (ISO, slash-date, syslog, time-only) → Timestamped
//! - Otherwise → Plain text

use crate::models::log_entry::LogFormat;
use super::timestamped::{self, DateOrder};

/// Result of format detection, including date order for slash-date disambiguation.
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedFormat {
    pub format: LogFormat,
    pub date_order: DateOrder,
}

/// Detect the log format from file content.
/// Examines up to the first 20 non-empty lines.
pub fn detect_format(content: &str) -> DetectedFormat {
    let sample_lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(20)
        .collect();

    let mut ccm_count = 0;
    let mut simple_count = 0;
    let mut timestamp_count = 0;
    let mut has_day_first = false;

    for line in &sample_lines {
        if line.contains("<![LOG[") && line.contains("]LOG]!>") {
            ccm_count += 1;
        } else if line.contains(" type=\"") && line.contains("component=\"") {
            // Fallback CCM detection from the binary's ` type="` check
            ccm_count += 1;
        } else if line.contains("$$<") {
            simple_count += 1;
        } else if timestamped::matches_any_timestamp(line.trim()) {
            timestamp_count += 1;
            // Check for EU-style dates (first field > 12 → must be day)
            if let Some(first_field) = timestamped::slash_date_first_field(line.trim()) {
                if first_field > 12 {
                    has_day_first = true;
                }
            }
        }
    }

    if ccm_count > 0 && ccm_count >= simple_count {
        DetectedFormat {
            format: LogFormat::Ccm,
            date_order: DateOrder::default(),
        }
    } else if simple_count > 0 {
        DetectedFormat {
            format: LogFormat::Simple,
            date_order: DateOrder::default(),
        }
    } else if timestamp_count >= 2 {
        // Require at least 2 timestamp matches to avoid false positives
        DetectedFormat {
            format: LogFormat::Timestamped,
            date_order: if has_day_first {
                DateOrder::DayFirst
            } else {
                DateOrder::MonthFirst
            },
        }
    } else {
        DetectedFormat {
            format: LogFormat::Plain,
            date_order: DateOrder::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ccm() {
        let content = r#"<![LOG[Test message]LOG]!><time="08:00:00.000+000" date="01-01-2024" component="Test" context="" type="1" thread="100" file="">
<![LOG[Another message]LOG]!><time="08:00:01.000+000" date="01-01-2024" component="Test" context="" type="1" thread="100" file="">"#;
        let detected = detect_format(content);
        assert_eq!(detected.format, LogFormat::Ccm);
    }

    #[test]
    fn test_detect_simple() {
        let content = r#"Message one $$<Comp1><01-01-2024 08:00:00.000+000><thread=100>
Message two $$<Comp2><01-01-2024 08:00:01.000+000><thread=200>"#;
        let detected = detect_format(content);
        assert_eq!(detected.format, LogFormat::Simple);
    }

    #[test]
    fn test_detect_plain() {
        let content = "Just some plain text\nAnother line\nNothing special here";
        let detected = detect_format(content);
        assert_eq!(detected.format, LogFormat::Plain);
    }

    #[test]
    fn test_detect_timestamped_iso() {
        let content = "2024-01-15T08:00:00.000Z Starting application\n\
                        2024-01-15T08:00:01.000Z Loading config\n\
                        2024-01-15T08:00:02.000Z Ready";
        let detected = detect_format(content);
        assert_eq!(detected.format, LogFormat::Timestamped);
    }

    #[test]
    fn test_detect_timestamped_us_date() {
        let content = "01/15/2024 08:00:00 Starting application\n\
                        01/15/2024 08:00:01 Loading config";
        let detected = detect_format(content);
        assert_eq!(detected.format, LogFormat::Timestamped);
        assert_eq!(detected.date_order, DateOrder::MonthFirst);
    }

    #[test]
    fn test_detect_timestamped_eu_date() {
        let content = "25/01/2024 08:00:00 Starting application\n\
                        15/01/2024 08:00:01 Loading config";
        let detected = detect_format(content);
        assert_eq!(detected.format, LogFormat::Timestamped);
        assert_eq!(detected.date_order, DateOrder::DayFirst);
    }

    #[test]
    fn test_single_timestamp_line_stays_plain() {
        // Only 1 timestamped line should not trigger Timestamped format
        let content = "2024-01-15T08:00:00Z Starting\nRandom text\nMore text";
        let detected = detect_format(content);
        assert_eq!(detected.format, LogFormat::Plain);
    }
}
