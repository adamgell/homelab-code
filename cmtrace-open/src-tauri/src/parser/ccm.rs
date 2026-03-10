//! CCM/SCCM format parser.
//!
//! Parses log lines in the format:
//!   <![LOG[message text]LOG]!><time="HH:mm:ss.fff+TZO" date="MM-dd-yyyy"
//!     component="Name" context="" type="N" thread="N" file="source.cpp">
//!
//! The regex patterns are derived directly from the scanf format strings
//! extracted from the CMTrace.exe binary (see REVERSE_ENGINEERING.md).

use once_cell::sync::Lazy;
use regex::Regex;

use crate::models::log_entry::{LogEntry, LogFormat, Severity};
use super::severity::detect_severity_from_text;

/// Compiled regex matching a complete CCM log line.
///
/// Based on the binary's scanf pattern:
///   <time="%02u:%02u:%02u.%03u%d" date="%02u-%02u-%04u"
///    component="%100[^"]" context="" type="%u" thread="%u" file="%100[^"]"
static CCM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r#"<!\[LOG\[(?P<msg>[\s\S]*?)\]LOG\]!>"#,
        r#"<time="(?P<h>\d{1,2}):(?P<m>\d{1,2}):(?P<s>\d{1,2})\.(?P<ms>\d+)(?P<tz>[+-]?\d+)""#,
        r#"\s+date="(?P<mon>\d{1,2})-(?P<day>\d{1,2})-(?P<yr>\d{4})""#,
        r#"\s+component="(?P<comp>[^"]*)""#,
        r#"\s+context="[^"]*""#,
        r#"\s+type="(?P<typ>\d)""#,
        r#"\s+thread="(?P<thr>\d+)""#,
        r#"(?:\s+file="(?P<file>[^"]*)")?"#,
    ))
    .expect("CCM regex must compile")
});

/// Parse a single CCM-format log line.
/// Returns None if the line doesn't match the CCM format.
fn parse_line(line: &str) -> Option<CcmParsed> {
    let caps = CCM_RE.captures(line)?;

    let msg = caps.name("msg").map(|m| m.as_str().to_string())?;
    let h: u32 = caps.name("h")?.as_str().parse().ok()?;
    let m: u32 = caps.name("m")?.as_str().parse().ok()?;
    let s: u32 = caps.name("s")?.as_str().parse().ok()?;
    let ms_str = caps.name("ms")?.as_str();
    // Truncate milliseconds to 3 digits (matching CMTrace behavior)
    let ms: u32 = if ms_str.len() > 3 {
        ms_str[..3].parse().ok()?
    } else {
        ms_str.parse().ok()?
    };
    let tz: i32 = caps.name("tz")?.as_str().parse().ok()?;
    let mon: u32 = caps.name("mon")?.as_str().parse().ok()?;
    let day: u32 = caps.name("day")?.as_str().parse().ok()?;
    let yr: i32 = caps.name("yr")?.as_str().parse().ok()?;
    let comp = caps.name("comp").map(|m| m.as_str().to_string());
    let typ: u32 = caps.name("typ")?.as_str().parse().ok()?;
    let thr: u32 = caps.name("thr")?.as_str().parse().ok()?;
    let file = caps.name("file").map(|m| m.as_str().to_string());

    let severity = match typ {
        2 => Severity::Warning,
        3 => Severity::Error,
        _ => Severity::Info, // type=1 or anything else
    };

    // Build timestamp using chrono
    let timestamp = chrono::NaiveDate::from_ymd_opt(yr, mon, day)
        .and_then(|d| d.and_hms_milli_opt(h, m, s, ms))
        .map(|dt| dt.and_utc().timestamp_millis());

    // Format display string: "MM-dd-yyyy HH:mm:ss.fff"
    let timestamp_display = Some(format!(
        "{:02}-{:02}-{:04} {:02}:{:02}:{:02}.{:03}",
        mon, day, yr, h, m, s, ms
    ));

    let thread_display = Some(format!("{} (0x{:04X})", thr, thr));

    Some(CcmParsed {
        message: msg,
        component: comp,
        timestamp,
        timestamp_display,
        severity,
        thread: thr,
        thread_display,
        source_file: file,
        timezone_offset: tz,
    })
}

struct CcmParsed {
    message: String,
    component: Option<String>,
    timestamp: Option<i64>,
    timestamp_display: Option<String>,
    severity: Severity,
    thread: u32,
    thread_display: Option<String>,
    source_file: Option<String>,
    timezone_offset: i32,
}

/// Parse all lines as CCM format.
/// Returns (entries, parse_error_count).
pub fn parse_lines(lines: &[&str], file_path: &str) -> (Vec<LogEntry>, u32) {
    let mut entries = Vec::with_capacity(lines.len());
    let mut errors = 0u32;
    let mut id_counter = 0u64;

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        match parse_line(line) {
            Some(parsed) => {
                entries.push(LogEntry {
                    id: id_counter,
                    line_number: (i + 1) as u32,
                    message: parsed.message,
                    component: parsed.component,
                    timestamp: parsed.timestamp,
                    timestamp_display: parsed.timestamp_display,
                    severity: parsed.severity,
                    thread: Some(parsed.thread),
                    thread_display: parsed.thread_display,
                    source_file: parsed.source_file,
                    format: LogFormat::Ccm,
                    file_path: file_path.to_string(),
                    timezone_offset: Some(parsed.timezone_offset),
                });
                id_counter += 1;
            }
            None => {
                // Line didn't match CCM format — treat as plain text continuation
                // or standalone plain text entry
                entries.push(LogEntry {
                    id: id_counter,
                    line_number: (i + 1) as u32,
                    message: line.to_string(),
                    component: None,
                    timestamp: None,
                    timestamp_display: None,
                    severity: detect_severity_from_text(line),
                    thread: None,
                    thread_display: None,
                    source_file: None,
                    format: LogFormat::Plain,
                    file_path: file_path.to_string(),
                    timezone_offset: None,
                });
                id_counter += 1;
                errors += 1;
            }
        }
    }

    (entries, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ccm_line() {
        let line = r#"<![LOG[Successfully connected to \\server\share]LOG]!><time="08:06:34.590-060" date="09-02-2016" component="ContentTransferManager" context="" type="1" thread="3692" file="datatransfer.cpp">"#;
        let parsed = parse_line(line).expect("should parse");
        assert_eq!(parsed.message, r"Successfully connected to \\server\share");
        assert_eq!(parsed.component.as_deref(), Some("ContentTransferManager"));
        assert_eq!(parsed.severity, Severity::Info);
        assert_eq!(parsed.thread, 3692);
        assert_eq!(parsed.source_file.as_deref(), Some("datatransfer.cpp"));
        assert_eq!(parsed.timezone_offset, -60);
        assert_eq!(
            parsed.timestamp_display.as_deref(),
            Some("09-02-2016 08:06:34.590")
        );
    }

    #[test]
    fn test_parse_ccm_error() {
        let line = r#"<![LOG[Failed to download content. Error 0x80070005]LOG]!><time="14:30:45.123+000" date="11-15-2023" component="ContentAccess" context="" type="3" thread="4480" file="contentaccess.cpp">"#;
        let parsed = parse_line(line).expect("should parse");
        assert_eq!(parsed.severity, Severity::Error);
        assert_eq!(parsed.component.as_deref(), Some("ContentAccess"));
    }

    #[test]
    fn test_parse_ccm_warning() {
        let line = r#"<![LOG[Retrying request]LOG]!><time="10:00:00.000+000" date="01-01-2024" component="Test" context="" type="2" thread="100" file="">"#;
        let parsed = parse_line(line).expect("should parse");
        assert_eq!(parsed.severity, Severity::Warning);
    }

    #[test]
    fn test_severity_from_text() {
        assert_eq!(detect_severity_from_text("An error occurred"), Severity::Error);
        assert_eq!(detect_severity_from_text("Connection failed"), Severity::Error);
        assert_eq!(detect_severity_from_text("Failover to backup"), Severity::Info);
        assert_eq!(detect_severity_from_text("Warning: low disk"), Severity::Warning);
        assert_eq!(detect_severity_from_text("All good"), Severity::Info);
    }
}
