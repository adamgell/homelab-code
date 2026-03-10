use once_cell::sync::Lazy;
use regex::Regex;

use super::ime_parser::ImeLine;
use super::models::DownloadStat;

// ---- Regex patterns for download-related log messages ----

/// Matches content download start/progress/completion messages
static DOWNLOAD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:download|downloading|content\s+download)"#).unwrap()
});

/// Extracts content size in bytes from messages like "Content size: 12345678 bytes"
static SIZE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:content\s+)?size[:\s]+(\d+)\s*(?:bytes|KB|MB|GB)"#).unwrap()
});

/// Extracts download speed from messages
static SPEED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:speed|rate)[:\s]+([\d.]+)\s*(?:bytes?/s|KB/s|MB/s|Bps|KBps|MBps)"#)
        .unwrap()
});

/// Extracts Delivery Optimization percentage
static DO_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:delivery\s+optimization|DO)[:\s]+([\d.]+)\s*%"#).unwrap()
});

/// Extracts content/app ID (GUID) from download messages
static CONTENT_ID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:content|app)\s*(?:id)?[:\s]+([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})"#).unwrap()
});

/// Matches download completion messages
static DOWNLOAD_COMPLETE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)download\s+(?:completed|finished|succeeded|done)"#).unwrap()
});

/// Matches download failure messages
static DOWNLOAD_FAILED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)download\s+(?:failed|error|timed?\s*out)"#).unwrap()
});

/// Extracts duration from messages like "Duration: 45 seconds" or "took 2.5s"
static DURATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:duration|took|elapsed)[:\s]+([\d.]+)\s*(?:s(?:ec(?:ond)?s?)?|m(?:in(?:ute)?s?)?)"#).unwrap()
});

/// Extract download statistics from parsed IME log lines.
pub fn extract_downloads(lines: &[ImeLine]) -> Vec<DownloadStat> {
    let mut downloads: Vec<DownloadStat> = Vec::new();
    let mut current_download: Option<PartialDownload> = None;

    for line in lines {
        let msg = &line.message;

        // Only look at download-related messages
        if !DOWNLOAD_RE.is_match(msg) {
            continue;
        }

        // Try to extract content ID
        let content_id = CONTENT_ID_RE
            .captures(msg)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string());

        // Check if this is a download completion
        if DOWNLOAD_COMPLETE_RE.is_match(msg) || DOWNLOAD_FAILED_RE.is_match(msg) {
            let success = DOWNLOAD_COMPLETE_RE.is_match(msg);
            let stat = finalize_download(
                current_download.take(),
                content_id.clone(),
                msg,
                line.timestamp.as_deref(),
                success,
            );
            if let Some(s) = stat {
                downloads.push(s);
            }
            continue;
        }

        // Start or update a download tracking entry
        let download = current_download.get_or_insert_with(|| PartialDownload {
            content_id: content_id.clone(),
            start_time: line.timestamp.clone(),
            size_bytes: None,
            speed_bps: None,
            do_percentage: None,
            duration_secs: None,
        });

        // Update content ID if we found one
        if content_id.is_some() && download.content_id.is_none() {
            download.content_id = content_id;
        }

        // Extract size
        if let Some(cap) = SIZE_RE.captures(msg) {
            if let Some(size_str) = cap.get(1) {
                if let Ok(size) = size_str.as_str().parse::<u64>() {
                    // Check unit and convert to bytes
                    let unit = cap.get(0).map_or("", |m| m.as_str());
                    download.size_bytes = Some(if unit.contains("GB") {
                        size * 1024 * 1024 * 1024
                    } else if unit.contains("MB") {
                        size * 1024 * 1024
                    } else if unit.contains("KB") {
                        size * 1024
                    } else {
                        size
                    });
                }
            }
        }

        // Extract speed
        if let Some(cap) = SPEED_RE.captures(msg) {
            if let Some(speed_str) = cap.get(1) {
                if let Ok(speed) = speed_str.as_str().parse::<f64>() {
                    let unit = cap.get(0).map_or("", |m| m.as_str());
                    download.speed_bps = Some(if unit.contains("MB") {
                        speed * 1024.0 * 1024.0
                    } else if unit.contains("KB") {
                        speed * 1024.0
                    } else {
                        speed
                    });
                }
            }
        }

        // Extract DO percentage
        if let Some(cap) = DO_RE.captures(msg) {
            if let Some(pct_str) = cap.get(1) {
                if let Ok(pct) = pct_str.as_str().parse::<f64>() {
                    download.do_percentage = Some(pct);
                }
            }
        }

        // Extract duration
        if let Some(cap) = DURATION_RE.captures(msg) {
            if let Some(dur_str) = cap.get(1) {
                if let Ok(dur) = dur_str.as_str().parse::<f64>() {
                    let unit = cap.get(0).map_or("", |m| m.as_str());
                    download.duration_secs = Some(if unit.contains("min") {
                        dur * 60.0
                    } else {
                        dur
                    });
                }
            }
        }
    }

    // Finalize any remaining partial download
    if let Some(partial) = current_download.take() {
        let cid = partial.content_id.unwrap_or_else(|| "unknown".to_string());
        let stat = DownloadStat {
            name: short_id(&cid),
            content_id: cid,
            size_bytes: partial.size_bytes.unwrap_or(0),
            speed_bps: partial.speed_bps.unwrap_or(0.0),
            do_percentage: partial.do_percentage.unwrap_or(0.0),
            duration_secs: partial.duration_secs.unwrap_or(0.0),
            success: false, // incomplete
            timestamp: partial.start_time,
        };
        downloads.push(stat);
    }

    downloads
}

/// Intermediate state while building a download stat.
struct PartialDownload {
    content_id: Option<String>,
    start_time: Option<String>,
    size_bytes: Option<u64>,
    speed_bps: Option<f64>,
    do_percentage: Option<f64>,
    duration_secs: Option<f64>,
}

/// Finalize a partial download into a DownloadStat.
fn finalize_download(
    partial: Option<PartialDownload>,
    content_id: Option<String>,
    msg: &str,
    timestamp: Option<&str>,
    success: bool,
) -> Option<DownloadStat> {
    let p = partial.unwrap_or(PartialDownload {
        content_id: content_id.clone(),
        start_time: timestamp.map(|t| t.to_string()),
        size_bytes: None,
        speed_bps: None,
        do_percentage: None,
        duration_secs: None,
    });

    let cid = content_id
        .or(p.content_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Try to extract additional data from the completion message
    let size = p.size_bytes.or_else(|| {
        SIZE_RE
            .captures(msg)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u64>().ok())
    });

    let speed = p.speed_bps.or_else(|| {
        SPEED_RE
            .captures(msg)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
    });

    let do_pct = p.do_percentage.or_else(|| {
        DO_RE
            .captures(msg)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
    });

    Some(DownloadStat {
        content_id: cid.clone(),
        name: short_id(&cid),
        size_bytes: size.unwrap_or(0),
        speed_bps: speed.unwrap_or(0.0),
        do_percentage: do_pct.unwrap_or(0.0),
        duration_secs: p.duration_secs.unwrap_or(0.0),
        success,
        timestamp: timestamp.map(|t| t.to_string()).or(p.start_time),
    })
}

/// Create a short display name from a GUID or content ID.
fn short_id(id: &str) -> String {
    if id.len() > 8 && id.contains('-') {
        format!("Download ({}...)", &id[..8])
    } else {
        format!("Download: {}", id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_detection() {
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

        let downloads = extract_downloads(&lines);
        assert_eq!(downloads.len(), 1);
        assert!(downloads[0].success);
        assert_eq!(downloads[0].size_bytes, 5242880);
    }

    #[test]
    fn test_short_id() {
        assert_eq!(
            short_id("a1b2c3d4-e5f6-7890-abcd-ef1234567890"),
            "Download (a1b2c3d4...)"
        );
        assert_eq!(short_id("short"), "Download: short");
    }
}
