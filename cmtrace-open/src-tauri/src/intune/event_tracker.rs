use once_cell::sync::Lazy;
use regex::Regex;

use super::ime_parser::ImeLine;
use super::models::{IntuneEvent, IntuneEventType, IntuneStatus};

// ---- Regex patterns for detecting Intune events ----

// Win32App patterns
static WIN32_APP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\[Win32App\].*(?:processing|executing|installing|detected|not detected|evaluating)"#).unwrap()
});
static WIN32_RESULT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\[Win32App\].*(?:result|completed|success|failed|error)"#).unwrap()
});
static WIN32_GUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:app|application)\s+(?:id|with\s+id)[:\s]+([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"#).unwrap()
});

// WinGet patterns
static WINGET_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)WinGetApp.*(?:processing|installing|detected|evaluating)"#).unwrap()
});

// Script patterns
static SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:PowerShell\s+script|script\s+execution|running\s+script)"#).unwrap()
});
static SCRIPT_RESULT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)script.*(?:completed|exit\s+code|result|output|failed|success)"#).unwrap()
});

// Remediation patterns
static REMEDIATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:Remediation|HealthScript|proactive\s+remediation)"#).unwrap()
});

// ESP (Enrollment Status Page) patterns
static ESP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:ESP|EspBody|EnrollmentStatusPage|enrollment\s+status)"#).unwrap()
});

// Sync session patterns
static SYNC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:sync\s+session|check-in|SyncSession)"#).unwrap()
});

// General GUID extraction
static GUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})"#).unwrap()
});

// Error code extraction
static ERROR_CODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:error\s*(?:code)?|exit\s*code|hresult|hr)\s*[=:]\s*(0x[0-9a-fA-F]+|-?\d+)"#).unwrap()
});

// Status detection
static SUCCESS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:success|succeeded|completed\s+successfully|installed|detected|compliant)"#).unwrap()
});
static FAILED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:fail|error|not\s+detected|not\s+installed|non-compliant|timed?\s*out)"#).unwrap()
});

/// Extract Intune events from parsed IME log lines.
pub fn extract_events(lines: &[ImeLine], source_file: &str) -> Vec<IntuneEvent> {
    let mut events = Vec::new();
    let mut next_id: u64 = 0;

    for line in lines {
        let msg = &line.message;

        // Check each event type in priority order
        let event_type = if WIN32_APP_RE.is_match(msg) || WIN32_RESULT_RE.is_match(msg) {
            Some(IntuneEventType::Win32App)
        } else if WINGET_RE.is_match(msg) {
            Some(IntuneEventType::WinGetApp)
        } else if SCRIPT_RE.is_match(msg) || SCRIPT_RESULT_RE.is_match(msg) {
            Some(IntuneEventType::PowerShellScript)
        } else if REMEDIATION_RE.is_match(msg) {
            Some(IntuneEventType::Remediation)
        } else if ESP_RE.is_match(msg) {
            Some(IntuneEventType::Esp)
        } else if SYNC_RE.is_match(msg) {
            Some(IntuneEventType::SyncSession)
        } else {
            None
        };

        if let Some(evt_type) = event_type {
            // Extract GUID
            let guid = extract_guid(msg);

            // Determine status
            let status = determine_status(msg);

            // Extract error code if present
            let error_code = ERROR_CODE_RE.captures(msg)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());

            // Build display name
            let name = build_event_name(&evt_type, &guid, msg);

            // Truncate detail message to reasonable length
            let detail = if msg.len() > 300 {
                format!("{}...", &msg[..300])
            } else {
                msg.clone()
            };

            events.push(IntuneEvent {
                id: next_id,
                event_type: evt_type,
                name,
                guid,
                status,
                start_time: line.timestamp.clone(),
                end_time: None,
                duration_secs: None,
                error_code,
                detail,
                source_file: source_file.to_string(),
                line_number: line.line_number,
            });

            next_id += 1;
        }
    }

    // Post-process: pair start/end events and calculate durations
    pair_events(&mut events);

    events
}

/// Extract the most relevant GUID from a message.
fn extract_guid(msg: &str) -> Option<String> {
    // First try Win32-specific GUID extraction
    if let Some(cap) = WIN32_GUID_RE.captures(msg) {
        return cap.get(1).map(|m| m.as_str().to_string());
    }

    // Fall back to first GUID found
    GUID_RE.captures(msg)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Determine the status of an event from its message text.
fn determine_status(msg: &str) -> IntuneStatus {
    if FAILED_RE.is_match(msg) {
        IntuneStatus::Failed
    } else if SUCCESS_RE.is_match(msg) {
        IntuneStatus::Success
    } else {
        IntuneStatus::InProgress
    }
}

/// Build a human-readable name for an event.
fn build_event_name(
    event_type: &IntuneEventType,
    guid: &Option<String>,
    msg: &str,
) -> String {
    let type_label = match event_type {
        IntuneEventType::Win32App => "Win32 App",
        IntuneEventType::WinGetApp => "WinGet App",
        IntuneEventType::PowerShellScript => "PowerShell Script",
        IntuneEventType::Remediation => "Remediation",
        IntuneEventType::Esp => "ESP",
        IntuneEventType::SyncSession => "Sync Session",
        IntuneEventType::PolicyEvaluation => "Policy Evaluation",
        IntuneEventType::ContentDownload => "Content Download",
        IntuneEventType::Other => "Other",
    };

    if let Some(guid) = guid {
        // Show truncated GUID for readability
        let short_guid = if guid.len() > 8 {
            &guid[..8]
        } else {
            guid
        };
        format!("{} ({}...)", type_label, short_guid)
    } else {
        // Try to extract a meaningful snippet from the message
        let snippet = msg.chars().take(50).collect::<String>();
        if snippet.len() < msg.len() {
            format!("{}: {}...", type_label, snippet)
        } else {
            format!("{}: {}", type_label, snippet)
        }
    }
}

/// Pair start/end events and calculate durations.
fn pair_events(events: &mut Vec<IntuneEvent>) {
    // Group events by GUID and type, pair starts with ends
    let len = events.len();
    for i in 0..len {
        if events[i].status != IntuneStatus::InProgress {
            continue;
        }
        if events[i].guid.is_none() {
            continue;
        }

        // Look for a completion event with same GUID/type
        for j in (i + 1)..len {
            if events[j].guid == events[i].guid
                && events[j].event_type == events[i].event_type
                && (events[j].status == IntuneStatus::Success
                    || events[j].status == IntuneStatus::Failed)
            {
                // Copy end time to start event
                events[i].end_time = events[j].start_time.clone();
                events[i].status = events[j].status.clone();
                events[i].error_code = events[j].error_code.clone();

                // Calculate rough duration from timestamps if available
                if let (Some(start), Some(end)) = (&events[i].start_time, &events[i].end_time) {
                    events[i].duration_secs = estimate_duration(start, end);
                }
                break;
            }
        }
    }
}

/// Estimate duration between two timestamp strings.
/// Handles formats like "MM-dd-yyyy HH:mm:ss.fff"
fn estimate_duration(start: &str, end: &str) -> Option<f64> {
    // Simple approach: parse the time portion and calculate difference
    let parse_seconds = |ts: &str| -> Option<f64> {
        // Extract time portion after date
        let time_part = ts.split_whitespace().last()?;
        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() >= 3 {
            let h: f64 = parts[0].parse().ok()?;
            let m: f64 = parts[1].parse().ok()?;
            let s: f64 = parts[2].parse().ok()?;
            Some(h * 3600.0 + m * 60.0 + s)
        } else {
            None
        }
    };

    let start_secs = parse_seconds(start)?;
    let end_secs = parse_seconds(end)?;

    let diff = end_secs - start_secs;
    if diff >= 0.0 {
        Some(diff)
    } else {
        // Crossed midnight
        Some(diff + 86400.0)
    }
}
