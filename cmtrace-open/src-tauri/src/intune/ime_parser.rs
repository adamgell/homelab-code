use once_cell::sync::Lazy;
use regex::Regex;

/// A parsed IME log line with extracted timestamp and message.
#[derive(Debug, Clone)]
pub struct ImeLine {
    pub line_number: u32,
    pub timestamp: Option<String>,
    pub message: String,
    pub component: Option<String>,
}

/// Regex for the IME log format:
/// <![LOG[message]LOG]!><time="HH:mm:ss.fff..." date="MM-dd-yyyy" component="..." ...>
static IME_LOG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"<!\[LOG\[(?P<msg>[\s\S]*?)\]LOG\]!><time="(?P<time>[^"]+)" date="(?P<date>[^"]+)" component="(?P<comp>[^"]*)"[^>]*>"#
    ).unwrap()
});

/// Regex for simple timestamped log lines (fallback format):
/// YYYY-MM-DD HH:MM:SS.fff message
static SIMPLE_TS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?P<ts>\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}[\.\d]*)\s+(?P<msg>.+)$"#
    ).unwrap()
});

/// Parse IME log content into structured lines.
pub fn parse_ime_content(content: &str) -> Vec<ImeLine> {
    let mut lines = Vec::new();
    let mut line_number: u32 = 0;

    // Try CCM/IME format first
    for cap in IME_LOG_RE.captures_iter(content) {
        line_number += 1;
        let msg = cap.name("msg").map_or("", |m| m.as_str()).to_string();
        let time = cap.name("time").map_or("", |m| m.as_str());
        let date = cap.name("date").map_or("", |m| m.as_str());
        let comp = cap.name("comp").map_or("", |m| m.as_str());

        let timestamp = if !date.is_empty() && !time.is_empty() {
            // Convert from "MM-dd-yyyy" + "HH:mm:ss.fff" to ISO-ish format
            let time_clean = time.split('+').next().unwrap_or(time);
            let time_clean = time_clean.split('-').next().unwrap_or(time_clean);
            Some(format!("{} {}", date, time_clean))
        } else {
            None
        };

        lines.push(ImeLine {
            line_number,
            timestamp,
            message: msg,
            component: if comp.is_empty() {
                None
            } else {
                Some(comp.to_string())
            },
        });
    }

    // If no CCM format found, try simple format
    if lines.is_empty() {
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(cap) = SIMPLE_TS_RE.captures(trimmed) {
                let ts = cap.name("ts").map(|m| m.as_str().to_string());
                let msg = cap.name("msg").map_or("", |m| m.as_str()).to_string();
                lines.push(ImeLine {
                    line_number: (i + 1) as u32,
                    timestamp: ts,
                    message: msg,
                    component: None,
                });
            } else {
                lines.push(ImeLine {
                    line_number: (i + 1) as u32,
                    timestamp: None,
                    message: trimmed.to_string(),
                    component: None,
                });
            }
        }
    }

    lines
}
