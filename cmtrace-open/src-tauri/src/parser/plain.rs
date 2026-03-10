//! Plain text fallback parser.
//!
//! Used when no structured format (CCM or Simple) is detected.
//! Each line becomes a LogEntry with text-based severity detection.

use crate::models::log_entry::{LogEntry, LogFormat, Severity};

/// Parse all lines as plain text.
pub fn parse_lines(lines: &[&str], file_path: &str) -> (Vec<LogEntry>, u32) {
    let mut entries = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let lower = line.to_lowercase();
        let severity = if lower.contains("error")
            || (lower.contains("fail") && !lower.contains("failover"))
        {
            Severity::Error
        } else if lower.contains("warn") {
            Severity::Warning
        } else {
            Severity::Info
        };

        entries.push(LogEntry {
            id: i as u64,
            line_number: (i + 1) as u32,
            message: line.to_string(),
            component: None,
            timestamp: None,
            timestamp_display: None,
            severity,
            thread: None,
            thread_display: None,
            source_file: None,
            format: LogFormat::Plain,
            file_path: file_path.to_string(),
            timezone_offset: None,
        });
    }

    // Plain text never has parse errors (every line is valid)
    (entries, 0)
}
