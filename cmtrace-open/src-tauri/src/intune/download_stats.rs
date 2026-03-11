use std::collections::HashMap;
use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;

use super::ime_parser::ImeLine;
use super::models::DownloadStat;

static DOWNLOAD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:download|downloading|content\s+download|delivery\s+optimization|bytes\s+downloaded|staging\s+(?:file|content)|hash\s+validation|content\s+cached|cache\s+location)"#,
    )
    .unwrap()
});
static SIZE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:content\s+)?size[:\s]+([\d.]+)\s*(bytes|kb|mb|gb)"#).unwrap()
});
static SPEED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:speed|rate)[:\s]+([\d.]+)\s*(bytes?/s|kb/s|mb/s|bps|kbps|mbps)"#)
        .unwrap()
});
static DO_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:delivery\s+optimization|DO)[:\s]+([\d.]+)\s*%"#).unwrap()
});
static CONTENT_ID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:content|app|application)\s*(?:id)?[:\s]+([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})"#).unwrap()
});
static DOWNLOAD_COMPLETE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:download\s+(?:completed|finished|succeeded|done)|content\s+cached|staging\s+completed|hash\s+validation\s+succeeded)"#,
    )
    .unwrap()
});
static DOWNLOAD_FAILED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:download\s+(?:failed|error)|failed\s+to\s+download|hash\s+validation\s+failed|hash\s+mismatch|staging\s+failed|content\s+not\s+found|unable\s+to\s+download|cancelled|aborted)"#,
    )
    .unwrap()
});
static DOWNLOAD_START_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:starting|beginning|queued|requesting|resuming).*(?:download|content\s+download)"#,
    )
    .unwrap()
});
static DOWNLOAD_PROGRESS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:bytes\s+downloaded|downloading|download\s+progress|delivery\s+optimization)"#)
        .unwrap()
});
static DOWNLOAD_STALL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:stalled|not\s+progressing|no\s+progress|timed?\s*out|timeout|retry\s+exhausted)"#)
        .unwrap()
});
static APPWORKLOAD_RETRY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:retry|retrying|reattempt|will\s+retry)"#).unwrap());
static DURATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:duration|took|elapsed)[:\s]+([\d.]+)\s*(s(?:ec(?:ond)?s?)?|m(?:in(?:ute)?s?)?)"#)
        .unwrap()
});

pub fn extract_downloads(lines: &[ImeLine], source_file: &str) -> Vec<DownloadStat> {
    let source_kind = classify_download_source(source_file);
    if source_kind == DownloadSourceKind::Unsupported {
        return Vec::new();
    }

    let mut downloads = Vec::new();
    let mut active: HashMap<String, PartialDownload> = HashMap::new();

    for line in lines {
        let msg = &line.message;
        if !DOWNLOAD_RE.is_match(msg) {
            continue;
        }

        let content_id = extract_content_id(msg).unwrap_or_else(|| "unknown".to_string());

        if APPWORKLOAD_RETRY_RE.is_match(msg) {
            if let Some(stat) = finalize_download(
                active.remove(&content_id),
                Some(content_id.clone()),
                msg,
                line.timestamp.as_deref(),
                false,
            ) {
                downloads.push(stat);
            }

            active.insert(
                content_id.clone(),
                PartialDownload::new(Some(content_id), line.timestamp.clone()),
            );
            continue;
        }

        if DOWNLOAD_START_RE.is_match(msg) || DOWNLOAD_PROGRESS_RE.is_match(msg) {
            let entry = active.entry(content_id.clone()).or_insert_with(|| {
                PartialDownload::new(Some(content_id.clone()), line.timestamp.clone())
            });
            update_download(entry, msg, line.timestamp.as_deref());
        }

        if DOWNLOAD_COMPLETE_RE.is_match(msg) {
            if let Some(stat) = finalize_download(
                active.remove(&content_id),
                Some(content_id),
                msg,
                line.timestamp.as_deref(),
                true,
            ) {
                downloads.push(stat);
            }
            continue;
        }

        if DOWNLOAD_FAILED_RE.is_match(msg) || DOWNLOAD_STALL_RE.is_match(msg) {
            if let Some(stat) = finalize_download(
                active.remove(&content_id),
                Some(content_id),
                msg,
                line.timestamp.as_deref(),
                false,
            ) {
                downloads.push(stat);
            }
        }
    }

    for partial in active.into_values() {
        if partial.saw_failure_signal || partial.saw_retry_signal {
            downloads.push(DownloadStat {
                content_id: partial
                    .content_id
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                name: short_id(partial.content_id.as_deref().unwrap_or("unknown")),
                size_bytes: partial.size_bytes.unwrap_or(0),
                speed_bps: partial.speed_bps.unwrap_or(0.0),
                do_percentage: partial.do_percentage.unwrap_or(0.0),
                duration_secs: partial.duration_secs.unwrap_or(0.0),
                success: false,
                timestamp: partial.last_timestamp.or(partial.start_time),
            });
        }
    }

    downloads
}

struct PartialDownload {
    content_id: Option<String>,
    start_time: Option<String>,
    last_timestamp: Option<String>,
    size_bytes: Option<u64>,
    speed_bps: Option<f64>,
    do_percentage: Option<f64>,
    duration_secs: Option<f64>,
    saw_progress: bool,
    saw_failure_signal: bool,
    saw_retry_signal: bool,
}

impl PartialDownload {
    fn new(content_id: Option<String>, start_time: Option<String>) -> Self {
        Self {
            content_id,
            start_time,
            last_timestamp: None,
            size_bytes: None,
            speed_bps: None,
            do_percentage: None,
            duration_secs: None,
            saw_progress: false,
            saw_failure_signal: false,
            saw_retry_signal: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadSourceKind {
    PrimaryIme,
    AppWorkload,
    Unsupported,
}

fn classify_download_source(source_file: &str) -> DownloadSourceKind {
    let file_name = Path::new(source_file)
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| source_file.to_ascii_lowercase());

    if file_name.contains("appworkload") {
        DownloadSourceKind::AppWorkload
    } else if file_name.contains("intunemanagementextension") {
        DownloadSourceKind::PrimaryIme
    } else {
        DownloadSourceKind::Unsupported
    }
}

fn extract_content_id(msg: &str) -> Option<String> {
    CONTENT_ID_RE
        .captures(msg)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
}

fn update_download(download: &mut PartialDownload, msg: &str, timestamp: Option<&str>) {
    if download.start_time.is_none() {
        download.start_time = timestamp.map(|value| value.to_string());
    }
    download.last_timestamp = timestamp.map(|value| value.to_string());

    if let Some(content_id) = extract_content_id(msg) {
        if download.content_id.is_none() || download.content_id.as_deref() == Some("unknown") {
            download.content_id = Some(content_id);
        }
    }

    if DOWNLOAD_PROGRESS_RE.is_match(msg) {
        download.saw_progress = true;
    }
    if DOWNLOAD_FAILED_RE.is_match(msg) || DOWNLOAD_STALL_RE.is_match(msg) {
        download.saw_failure_signal = true;
    }
    if APPWORKLOAD_RETRY_RE.is_match(msg) {
        download.saw_retry_signal = true;
    }

    if let Some((value, unit)) = capture_number_and_unit(&SIZE_RE, msg) {
        download.size_bytes = Some(convert_size_to_bytes(value, unit));
    }

    if let Some((value, unit)) = capture_number_and_unit(&SPEED_RE, msg) {
        download.speed_bps = Some(convert_speed_to_bps(value, unit));
    }

    if let Some(captures) = DO_RE.captures(msg) {
        if let Some(value) = captures
            .get(1)
            .and_then(|capture| capture.as_str().parse::<f64>().ok())
        {
            download.do_percentage = Some(value);
        }
    }

    if let Some((value, unit)) = capture_number_and_unit(&DURATION_RE, msg) {
        download.duration_secs = Some(if unit.starts_with('m') { value * 60.0 } else { value });
    }
}

fn capture_number_and_unit<'a>(re: &Regex, msg: &'a str) -> Option<(f64, &'a str)> {
    let captures = re.captures(msg)?;
    let value = captures.get(1)?.as_str().parse::<f64>().ok()?;
    let unit = captures.get(2)?.as_str();
    Some((value, unit))
}

fn convert_size_to_bytes(value: f64, unit: &str) -> u64 {
    let multiplier = match unit.to_ascii_lowercase().as_str() {
        "gb" => 1024.0 * 1024.0 * 1024.0,
        "mb" => 1024.0 * 1024.0,
        "kb" => 1024.0,
        _ => 1.0,
    };

    (value * multiplier).round() as u64
}

fn convert_speed_to_bps(value: f64, unit: &str) -> f64 {
    let normalized = unit.to_ascii_lowercase();
    if normalized.contains("mb") {
        value * 1024.0 * 1024.0
    } else if normalized.contains("kb") {
        value * 1024.0
    } else {
        value
    }
}

fn finalize_download(
    partial: Option<PartialDownload>,
    content_id: Option<String>,
    msg: &str,
    timestamp: Option<&str>,
    success: bool,
) -> Option<DownloadStat> {
    let mut partial = partial.unwrap_or_else(|| {
        PartialDownload::new(content_id.clone(), timestamp.map(|value| value.to_string()))
    });
    update_download(&mut partial, msg, timestamp);

    let resolved_content_id = content_id
        .or(partial.content_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    if !success
        && !partial.saw_failure_signal
        && !partial.saw_retry_signal
        && !DOWNLOAD_STALL_RE.is_match(msg)
    {
        return None;
    }

    Some(DownloadStat {
        content_id: resolved_content_id.clone(),
        name: short_id(&resolved_content_id),
        size_bytes: partial.size_bytes.unwrap_or(0),
        speed_bps: partial.speed_bps.unwrap_or(0.0),
        do_percentage: partial.do_percentage.unwrap_or(0.0),
        duration_secs: partial.duration_secs.unwrap_or(0.0),
        success,
        timestamp: timestamp
            .map(|value| value.to_string())
            .or(partial.last_timestamp)
            .or(partial.start_time),
    })
}

fn short_id(id: &str) -> String {
    if id.len() > 8 && id.contains('-') {
        format!("Download ({id}...)", id = &id[..8])
    } else {
        format!("Download: {id}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_download_is_recorded() {
        let lines = vec![
            ImeLine {
                line_number: 1,
                timestamp: Some("01-15-2024 10:00:00.000".to_string()),
                message: "Starting content download for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
                component: None,
            },
            ImeLine {
                line_number: 2,
                timestamp: Some("01-15-2024 10:00:05.000".to_string()),
                message: "Download completed successfully. Content size: 5242880 bytes, speed: 1048576 Bps, Delivery Optimization: 75.5%".to_string(),
                component: None,
            },
        ];

        let downloads = extract_downloads(&lines, "C:/Logs/AppWorkload.log");
        assert_eq!(downloads.len(), 1);
        assert!(downloads[0].success);
        assert_eq!(downloads[0].size_bytes, 5242880);
    }

    #[test]
    fn stalled_download_is_recorded_as_failed() {
        let lines = vec![
            ImeLine {
                line_number: 1,
                timestamp: Some("01-15-2024 10:00:00.000".to_string()),
                message: "Starting content download for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
                component: None,
            },
            ImeLine {
                line_number: 2,
                timestamp: Some("01-15-2024 10:00:30.000".to_string()),
                message: "Content download stalled with no progress for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
                component: None,
            },
        ];

        let downloads = extract_downloads(&lines, "C:/Logs/AppWorkload.log");
        assert_eq!(downloads.len(), 1);
        assert!(!downloads[0].success);
    }

    #[test]
    fn plain_start_line_does_not_create_failed_download() {
        let lines = vec![ImeLine {
            line_number: 1,
            timestamp: Some("01-15-2024 10:00:00.000".to_string()),
            message: "Starting content download for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
            component: None,
        }];

        let downloads = extract_downloads(&lines, "C:/Logs/AppWorkload.log");
        assert!(downloads.is_empty());
    }

    #[test]
    fn retry_creates_failed_attempt() {
        let lines = vec![
            ImeLine {
                line_number: 1,
                timestamp: Some("01-15-2024 10:00:00.000".to_string()),
                message: "Starting content download for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
                component: None,
            },
            ImeLine {
                line_number: 2,
                timestamp: Some("01-15-2024 10:00:05.000".to_string()),
                message: "Download failed, retrying content download for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
                component: None,
            },
        ];

        let downloads = extract_downloads(&lines, "C:/Logs/AppWorkload.log");
        assert_eq!(downloads.len(), 1);
        assert!(!downloads[0].success);
    }
}
