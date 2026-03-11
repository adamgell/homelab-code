use std::collections::{HashMap, HashSet};
use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;

use super::ime_parser::ImeLine;
use super::models::{IntuneEvent, IntuneEventType, IntuneStatus};

// ---- Regex patterns for detecting Intune events ----

// Win32App patterns
static WIN32_APP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)\[Win32App\].*(?:processing|executing|installing|detected|not detected|evaluating)"#,
    )
    .unwrap()
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
static WINGET_TOKEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:winget|microsoft\.winget)"#).unwrap());

// AppWorkload patterns
static APPWORKLOAD_DOWNLOAD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:download(?:ing|ed)?|delivery\s+optimization|content\s+download|bytes\s+downloaded|staging\s+(?:file|content)|hash\s+validation)"#,
    )
    .unwrap()
});
static APPWORKLOAD_STAGING_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:staging\s+(?:file|content)|hash\s+validation|content\s+cached|cache\s+location)"#)
        .unwrap()
});
static APPWORKLOAD_INSTALL_PHASE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:install(?:ing|ation)?|execution|enforcement|installer|launching\s+install|handoff\s+to\s+install)"#,
    )
    .unwrap()
});
static APPWORKLOAD_INSTALL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:install(?:ing|ation)?|execution|enforcement|installer|launching\s+install|handoff\s+to\s+install|processdetectionrules|detection\s+rule)"#,
    )
    .unwrap()
});

// AppActionProcessor patterns
static POLICY_EVAL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:assignment\s+evaluation|targeted\s+intent|applicability\s*=|\bapplicable\b|not\s+applicable|requirement\s+rule|detection\s+rule|local\s+deadline|grs\s+expired|enforcement\s+classification|will\s+not\s+be\s+enforced)"#,
    )
    .unwrap()
});
static APP_ACTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:app\s+with\s+id:|application\s+action|managed\s+app)"#).unwrap()
});
static APPLICABILITY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:applicability|applicable|not\s+applicable|requirement\s+rule|detection\s+rule)"#)
        .unwrap()
});

// Script patterns
static SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:PowerShell\s+script|script\s+execution|running\s+script)"#).unwrap()
});
static SCRIPT_RESULT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)script.*(?:completed|exit\s+code|result|output|failed|success)"#).unwrap()
});
static AGENTEXECUTOR_SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:powershell\s+script\s+is\s+successfully\s+executed|detection\s+script|remediation\s+script|exit\s+code|script\s+(?:completed|failed|timed?\s*out|execution))"#,
    )
    .unwrap()
});
static DETECTION_SCRIPT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\bdetection\s+script\b"#).unwrap());
static REMEDIATION_SCRIPT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\bremediation\s+script\b"#).unwrap());

// Remediation patterns
static REMEDIATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:Remediation|HealthScript|proactive\s+remediation)"#).unwrap()
});
static HEALTHSCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:healthscript|health\s+script|detection\s+script|remediation\s+script|pre-detect|post-detect|schedule(?:d|ing)?)"#,
    )
    .unwrap()
});

// ESP (Enrollment Status Page) patterns
static ESP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:ESP|EspBody|EnrollmentStatusPage|enrollment\s+status)"#).unwrap()
});

// Sync session patterns
static SYNC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:sync\s+session|check-in|SyncSession)"#).unwrap());

// General GUID extraction
static GUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})"#)
        .unwrap()
});

// Error code extraction
static ERROR_CODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:error\s*(?:code)?|exit\s*code(?:\s+of\s+the\s+script)?|hresult|hr)\s*(?:is|[=:])\s*(0x[0-9a-fA-F]+|-?\d+)"#,
    )
    .unwrap()
});
static EXIT_CODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)exit\s*code(?:\s+of\s+the\s+script)?\s*(?:is|[=:])\s*(-?\d+)"#)
        .unwrap()
});
static PENDING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:pending|queued|waiting|scheduled)"#).unwrap());
static TIMEOUT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:timed?\s*out|timeout)"#).unwrap());
static COMPLIANCE_TRUE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)compliance\s+result.*\bis\s+true\b"#).unwrap());
static COMPLIANCE_FALSE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)compliance\s+result.*\bis\s+false\b"#).unwrap());

// Status detection
static SUCCESS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:success|succeeded|completed\s+successfully|installed|detected|compliant)"#)
        .unwrap()
});
static FAILED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:fail|error|not\s+detected|not\s+installed|non-compliant|timed?\s*out)"#)
        .unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImeSourceKind {
    PrimaryIme,
    AppWorkload,
    AppActionProcessor,
    AgentExecutor,
    HealthScripts,
    Other,
}

/// Extract Intune events from parsed IME log lines.
pub fn extract_events(lines: &[ImeLine], source_file: &str) -> Vec<IntuneEvent> {
    let mut events = Vec::new();
    let mut next_id: u64 = 0;
    let source_kind = classify_source_kind(source_file);

    for line in lines {
        let msg = &line.message;

        // Check each event type in priority order
        let event_type = detect_event_type(line, source_kind);

        if let Some(evt_type) = event_type {
            // Extract GUID
            let guid = extract_guid(msg);

            // Determine status
            let status = determine_status(msg);

            // Extract error code if present
            let error_code = ERROR_CODE_RE
                .captures(msg)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());

            // Build display name
            let name = build_event_name(&evt_type, &guid, msg, source_kind);

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

fn classify_source_kind(source_file: &str) -> ImeSourceKind {
    let file_name = Path::new(source_file)
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| source_file.to_ascii_lowercase());

    if file_name.contains("appworkload") {
        ImeSourceKind::AppWorkload
    } else if file_name.contains("appactionprocessor") {
        ImeSourceKind::AppActionProcessor
    } else if file_name.contains("agentexecutor") {
        ImeSourceKind::AgentExecutor
    } else if file_name.contains("healthscripts") {
        ImeSourceKind::HealthScripts
    } else if file_name.contains("intunemanagementextension") {
        ImeSourceKind::PrimaryIme
    } else {
        ImeSourceKind::Other
    }
}

fn detect_event_type(line: &ImeLine, source_kind: ImeSourceKind) -> Option<IntuneEventType> {
    let msg = line.message.as_str();

    match source_kind {
        ImeSourceKind::AppWorkload => {
            if !is_appworkload_event_candidate(msg) {
                return None;
            }

            if APPWORKLOAD_DOWNLOAD_RE.is_match(msg) {
                return Some(IntuneEventType::ContentDownload);
            }
            if WINGET_TOKEN_RE.is_match(msg) || WINGET_RE.is_match(msg) {
                return Some(IntuneEventType::WinGetApp);
            }
            if APPWORKLOAD_INSTALL_RE.is_match(msg) {
                return Some(IntuneEventType::Win32App);
            }
        }
        ImeSourceKind::AppActionProcessor => {
            if !is_app_action_processor_event_candidate(msg) {
                return None;
            }

            if POLICY_EVAL_RE.is_match(msg) {
                return Some(IntuneEventType::PolicyEvaluation);
            }
            if APP_ACTION_RE.is_match(msg) || APPLICABILITY_RE.is_match(msg) {
                return Some(IntuneEventType::PolicyEvaluation);
            }
        }
        ImeSourceKind::AgentExecutor => {
            if !is_agent_executor_event_candidate(msg) {
                return None;
            }

            if REMEDIATION_RE.is_match(msg) {
                return Some(IntuneEventType::Remediation);
            }
            if AGENTEXECUTOR_SCRIPT_RE.is_match(msg)
                || SCRIPT_RE.is_match(msg)
                || SCRIPT_RESULT_RE.is_match(msg)
            {
                return Some(IntuneEventType::PowerShellScript);
            }
        }
        ImeSourceKind::HealthScripts => {
            if !is_healthscripts_event_candidate(msg) {
                return None;
            }

            if msg.to_ascii_lowercase().contains("exit code of the script") {
                return Some(IntuneEventType::Remediation);
            }

            if HEALTHSCRIPT_RE.is_match(msg) || REMEDIATION_RE.is_match(msg) {
                return Some(IntuneEventType::Remediation);
            }
        }
        ImeSourceKind::PrimaryIme | ImeSourceKind::Other => {}
    }

    if WIN32_APP_RE.is_match(msg) || WIN32_RESULT_RE.is_match(msg) {
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
    }
}

fn is_agent_executor_event_candidate(msg: &str) -> bool {
    let normalized = msg.to_ascii_lowercase();

    if is_agent_executor_noise_line(&normalized) {
        return false;
    }

    normalized.contains("powershell exit code")
        || normalized.contains("powershell script is successfully executed")
        || normalized.contains("detection script")
        || normalized.contains("remediation script")
        || normalized.contains("script completed")
        || normalized.contains("script failed")
        || normalized.contains("script execution")
        || normalized.contains("timed out")
        || normalized.contains("timeout")
        || normalized.contains("exit code")
}

    fn is_agent_executor_noise_line(normalized: &str) -> bool {
        normalized.ends_with(".timeout")
        || normalized.ends_with("quotedtimeoutfilepath.txt")
        || normalized.contains("prepare to run powershell script")
        || normalized.contains("remediation script option gets invoked")
    }

fn is_appworkload_event_candidate(msg: &str) -> bool {
    let normalized = msg.to_ascii_lowercase();

    if normalized.contains("reportingmanager")
        || normalized.contains("reportingcachemanager")
        || normalized.contains("reporting state initialized")
        || normalized.contains("sending reports")
        || normalized.contains("writeabletostorage")
        || normalized.contains("cangenerate")
        || normalized.contains("isappreportable")
    {
        return false;
    }

    normalized.contains("download")
        || normalized.contains("delivery optimization")
        || normalized.contains("staging ")
        || normalized.contains("hash validation")
        || normalized.contains("content cached")
        || normalized.contains("cache location")
        || normalized.contains("install")
        || normalized.contains("installer")
        || normalized.contains("execution")
        || normalized.contains("enforcement")
        || normalized.contains("handoff")
        || normalized.contains("winget")
}

fn is_app_action_processor_event_candidate(msg: &str) -> bool {
    let normalized = msg.to_ascii_lowercase();

    if normalized.contains("processor initializing")
        || (normalized.contains("found:") && normalized.contains("apps with intent"))
        || normalized.contains("evaluating install enforcement actions for app with id")
        || normalized.contains("no action required for app with id")
    {
        return false;
    }

    normalized.contains("app with id:")
        || normalized.contains("assignment evaluation")
        || normalized.contains("targeted intent")
        || normalized.contains("applicability =")
        || normalized.contains("not applicable")
        || normalized.contains("local deadline")
        || normalized.contains("grs expired")
        || normalized.contains("requirement rule")
        || normalized.contains("detection rule")
        || normalized.contains("will not be enforced")
}

fn is_healthscripts_event_candidate(msg: &str) -> bool {
    let normalized = msg.to_ascii_lowercase();

    if normalized.contains("inspect hourly schedule")
        || normalized.contains("job is queued and will be scheduled")
        || normalized.contains("completed user session")
    {
        return false;
    }

    normalized.contains("detection script")
        || normalized.contains("remediation script")
        || normalized.contains("exit code of the script")
        || normalized.contains("compliance result")
        || normalized.contains("pre-detect")
        || normalized.contains("post-detect")
        || normalized.contains("timed out")
        || normalized.contains("timeout")
        || normalized.contains("failed")
}

/// Extract the most relevant GUID from a message.
fn extract_guid(msg: &str) -> Option<String> {
    // First try Win32-specific GUID extraction
    if let Some(cap) = WIN32_GUID_RE.captures(msg) {
        return cap.get(1).map(|m| m.as_str().to_string());
    }

    // Fall back to first GUID found
    GUID_RE
        .captures(msg)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Determine the status of an event from its message text.
fn determine_status(msg: &str) -> IntuneStatus {
    if COMPLIANCE_TRUE_RE.is_match(msg) {
        return IntuneStatus::Success;
    }

    if COMPLIANCE_FALSE_RE.is_match(msg) {
        return IntuneStatus::Failed;
    }

    if let Some(exit_code) = EXIT_CODE_RE
        .captures(msg)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().parse::<i32>().ok())
    {
        return if exit_code == 0 {
            IntuneStatus::Success
        } else {
            IntuneStatus::Failed
        };
    }

    if TIMEOUT_RE.is_match(msg) {
        IntuneStatus::Timeout
    } else if FAILED_RE.is_match(msg) {
        IntuneStatus::Failed
    } else if SUCCESS_RE.is_match(msg) {
        IntuneStatus::Success
    } else if PENDING_RE.is_match(msg) {
        IntuneStatus::Pending
    } else {
        IntuneStatus::InProgress
    }
}

/// Build a human-readable name for an event.
fn build_event_name(
    event_type: &IntuneEventType,
    guid: &Option<String>,
    msg: &str,
    source_kind: ImeSourceKind,
) -> String {
    if let Some(specific_name) = build_source_specific_name(event_type, guid, msg, source_kind) {
        return specific_name;
    }

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

    let source_prefix = match source_kind {
        ImeSourceKind::AppWorkload => Some("AppWorkload"),
        ImeSourceKind::AppActionProcessor => Some("AppActionProcessor"),
        ImeSourceKind::AgentExecutor => Some("AgentExecutor"),
        ImeSourceKind::HealthScripts => Some("HealthScripts"),
        _ => None,
    };

    if let Some(guid) = guid {
        // Show truncated GUID for readability
        let short_guid = if guid.len() > 8 { &guid[..8] } else { guid };
        match source_prefix {
            Some(prefix) => format!("{} {} ({}...)", prefix, type_label, short_guid),
            None => format!("{} ({}...)", type_label, short_guid),
        }
    } else {
        // Try to extract a meaningful snippet from the message
        let snippet = msg.chars().take(50).collect::<String>();
        let base = if snippet.len() < msg.len() {
            format!("{}: {}...", type_label, snippet)
        } else {
            format!("{}: {}", type_label, snippet)
        };

        match source_prefix {
            Some(prefix) => format!("{} {}", prefix, base),
            None => base,
        }
    }
}

fn build_source_specific_name(
    event_type: &IntuneEventType,
    guid: &Option<String>,
    msg: &str,
    source_kind: ImeSourceKind,
) -> Option<String> {
    let short_guid = guid
        .as_deref()
        .map(|value| if value.len() > 8 { &value[..8] } else { value });

    match source_kind {
        ImeSourceKind::AppWorkload => {
            let phase = if APPWORKLOAD_STAGING_RE.is_match(msg) {
                "Staging"
            } else if APPWORKLOAD_INSTALL_PHASE_RE.is_match(msg) {
                "Install"
            } else if APPWORKLOAD_DOWNLOAD_RE.is_match(msg) {
                "Download"
            } else {
                return None;
            };

            Some(match short_guid {
                Some(guid) => format!("AppWorkload {} ({}...)", phase, guid),
                None => format!("AppWorkload {}", phase),
            })
        }
        ImeSourceKind::AppActionProcessor => {
            let area = if APPLICABILITY_RE.is_match(msg) {
                "Applicability"
            } else if POLICY_EVAL_RE.is_match(msg) {
                "Policy Evaluation"
            } else {
                return None;
            };

            Some(match short_guid {
                Some(guid) => format!("AppActionProcessor {} ({}...)", area, guid),
                None => format!("AppActionProcessor {}", area),
            })
        }
        ImeSourceKind::AgentExecutor => {
            let area = if REMEDIATION_SCRIPT_RE.is_match(msg) {
                "Remediation Script"
            } else if DETECTION_SCRIPT_RE.is_match(msg) {
                "Detection Script"
            } else if *event_type == IntuneEventType::Remediation {
                "Remediation Script"
            } else {
                "PowerShell Script"
            };

            Some(match short_guid {
                Some(guid) => format!("AgentExecutor {} ({}...)", area, guid),
                None => format!("AgentExecutor {}", area),
            })
        }
        ImeSourceKind::HealthScripts => {
            let area = if REMEDIATION_SCRIPT_RE.is_match(msg) {
                "Remediation"
            } else if DETECTION_SCRIPT_RE.is_match(msg) {
                "Detection"
            } else {
                "Schedule"
            };

            Some(match short_guid {
                Some(guid) => format!("HealthScripts {} ({}...)", area, guid),
                None => format!("HealthScripts {}", area),
            })
        }
        ImeSourceKind::PrimaryIme | ImeSourceKind::Other => None,
    }
}

/// Pair start/end events and calculate durations.
fn pair_events(events: &mut Vec<IntuneEvent>) {
    let mut consumed_end_indices: HashSet<usize> = HashSet::new();
    let mut open_events: HashMap<String, Vec<usize>> = HashMap::new();

    for index in 0..events.len() {
        let status = events[index].status.clone();
        let Some(identity_key) = event_identity_key(&events[index]) else {
            continue;
        };

        if status == IntuneStatus::InProgress {
            open_events.entry(identity_key).or_default().push(index);
            continue;
        }

        if !(status == IntuneStatus::Success
            || status == IntuneStatus::Failed
            || status == IntuneStatus::Timeout)
        {
            continue;
        }

        let Some(start_index) = open_events
            .get_mut(&identity_key)
            .and_then(|indices| indices.pop())
        else {
            continue;
        };

        if consumed_end_indices.contains(&index) {
            continue;
        }

        events[start_index].end_time = events[index].start_time.clone();
        events[start_index].status = events[index].status.clone();
        events[start_index].error_code = events[index]
            .error_code
            .clone()
            .or_else(|| events[start_index].error_code.clone());

        if let (Some(start), Some(end)) = (&events[start_index].start_time, &events[start_index].end_time) {
            events[start_index].duration_secs = estimate_duration(start, end);
        }

        consumed_end_indices.insert(index);
    }

    if consumed_end_indices.is_empty() {
        return;
    }

    let mut index = 0usize;
    events.retain(|_| {
        let keep = !consumed_end_indices.contains(&index);
        index += 1;
        keep
    });
}

fn event_identity_key(event: &IntuneEvent) -> Option<String> {
    if let Some(guid) = &event.guid {
        return Some(format!(
            "{}|{}|{}",
            event.source_file,
            event_type_identity(&event.event_type),
            guid
        ));
    }

    let normalized_name = normalize_identity_fragment(&event.name);
    if normalized_name.is_empty() {
        None
    } else {
        Some(format!(
            "{}|{}|{}",
            event.source_file,
            event_type_identity(&event.event_type),
            normalized_name
        ))
    }
}

fn event_type_identity(event_type: &IntuneEventType) -> &'static str {
    match event_type {
        IntuneEventType::Win32App => "win32",
        IntuneEventType::WinGetApp => "winget",
        IntuneEventType::PowerShellScript => "script",
        IntuneEventType::Remediation => "remediation",
        IntuneEventType::Esp => "esp",
        IntuneEventType::SyncSession => "sync",
        IntuneEventType::PolicyEvaluation => "policy",
        IntuneEventType::ContentDownload => "download",
        IntuneEventType::Other => "other",
    }
}

fn normalize_identity_fragment(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn line(message: &str, timestamp: &str, line_number: u32) -> ImeLine {
        ImeLine {
            line_number,
            timestamp: Some(timestamp.to_string()),
            message: message.to_string(),
            component: None,
        }
    }

    #[test]
    fn appworkload_extracts_content_download_events() {
        let events = extract_events(
            &[line(
                "AppWorkload reporting content download completed successfully for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AppWorkload.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::ContentDownload);
        assert_eq!(events[0].status, IntuneStatus::Success);
    }

    #[test]
    fn app_action_processor_extracts_policy_evaluation_events() {
        let events = extract_events(
            &[line(
                "Assignment evaluation started for app with id: a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AppActionProcessor.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::PolicyEvaluation);
        assert_eq!(events[0].name, "AppActionProcessor Policy Evaluation (a1b2c3d4...)".to_string());
    }

    #[test]
    fn agent_executor_extracts_script_events() {
        let events = extract_events(
            &[line(
                "AgentExecutor detection script completed with exit code: 0",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AgentExecutor.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::PowerShellScript);
        assert_eq!(events[0].status, IntuneStatus::Success);
        assert_eq!(events[0].name, "AgentExecutor Detection Script".to_string());
    }

    #[test]
    fn agent_executor_skips_argument_parsing_noise() {
        let events = extract_events(
            &[
                line(
                    "Creating command line parser, name delimiter is - and value separator is .",
                    "01-15-2024 10:00:05.000",
                    1,
                ),
                line(
                    "Adding argument remediationScript with value C:\\Windows\\IMECache\\HealthScripts\\abc\\detect.ps1 to the named argument list.",
                    "01-15-2024 10:00:06.000",
                    2,
                ),
                line(
                    "PowerShell path is C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
                    "01-15-2024 10:00:07.000",
                    3,
                ),
            ],
            "C:/Logs/AgentExecutor.log",
        );

        assert!(events.is_empty());
    }

    #[test]
    fn agent_executor_skips_timeout_path_and_prepare_noise() {
        let events = extract_events(
            &[
                line(
                    r#"C:\Windows\IMECache\HealthScripts\79880037-a3c4-489a-a7e6-a6a705b52b78_1\e154babf-cb85-4711-9bd9-3da0a1b846f2_PreDetectScript.timeout"#,
                    "01-15-2024 10:00:05.000",
                    1,
                ),
                line(
                    "Prepare to run Powershell Script ..",
                    "01-15-2024 10:00:06.000",
                    2,
                ),
                line(
                    "remediation script option gets invoked",
                    "01-15-2024 10:00:07.000",
                    3,
                ),
            ],
            "C:/Logs/AgentExecutor.log",
        );

        assert!(events.is_empty());
    }

    #[test]
    fn app_action_processor_skips_summary_noise() {
        let events = extract_events(
            &[
                line(
                    "[Win32App][ActionProcessor] Found: 0 apps with intent to uninstall before enforcing installs: [].",
                    "01-15-2024 10:00:05.000",
                    1,
                ),
                line(
                    "[Win32App][ActionProcessor] Processor initializing. Detection and applicability checks will run for all apps in the subgraph.",
                    "01-15-2024 10:00:06.000",
                    2,
                ),
                line(
                    "[Win32App][ActionProcessor] Evaluating install enforcement actions for app with id: a1b2c3d4-e5f6-7890-abcd-ef1234567890.",
                    "01-15-2024 10:00:07.000",
                    3,
                ),
                line(
                    "[Win32App][ActionProcessor] No action required for app with id: a1b2c3d4-e5f6-7890-abcd-ef1234567890.",
                    "01-15-2024 10:00:08.000",
                    4,
                ),
            ],
            "C:/Logs/AppActionProcessor.log",
        );

        assert!(events.is_empty());
    }

    #[test]
    fn appworkload_skips_reporting_manager_noise() {
        let events = extract_events(
            &[line(
                "[Win32App][ReportingManager] App with id: 174012d1-1931-4852-b64b-0754350ffe88 and prior AppAuthority: V3 has been loaded and reporting state initialized. ReportingState: {\"ApplicationId\":\"174012d1-1931-4852-b64b-0754350ffe88\",\"WriteableToStorage\":true,\"CanGenerateComplianceState\":true,\"CanGenerateEnforcementState\":true,\"IsAppReportable\":true}",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AppWorkload.log",
        );

        assert!(events.is_empty());
    }

    #[test]
    fn healthscripts_extracts_remediation_events() {
        let events = extract_events(
            &[line(
                "HealthScripts detection script scheduled for proactive remediation package",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/HealthScripts.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::Remediation);
        assert_eq!(events[0].status, IntuneStatus::Pending);
        assert_eq!(events[0].name, "HealthScripts Detection".to_string());
    }

    #[test]
    fn healthscripts_skips_schedule_noise() {
        let events = extract_events(
            &[
                line(
                    "[HS] inspect hourly schedule for policy 79880037-a3c4-489a-a7e6-a6a705b52b78: UTC = False, Interval = 1, Time = ",
                    "01-15-2024 10:00:05.000",
                    1,
                ),
                line(
                    "[HS] Runner : Job is queued and will be scheduled to run at UTC 3/2/2026 11:37:43 AM.",
                    "01-15-2024 10:00:06.000",
                    2,
                ),
                line(
                    "[HS] Runner ..................... Completed user session 0, userId: 00000000-0000-0000-0000-000000000000, userSID: ..................... ",
                    "01-15-2024 10:00:07.000",
                    3,
                ),
            ],
            "C:/Logs/HealthScripts.log",
        );

        assert!(events.is_empty());
    }

    #[test]
    fn healthscripts_compliance_result_marks_terminal_status() {
        let success_events = extract_events(
            &[line(
                "[HS] the pre-remdiation detection script compliance result for 79880037-a3c4-489a-a7e6-a6a705b52b78 is True",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/HealthScripts.log",
        );

        assert_eq!(success_events.len(), 1);
        assert_eq!(success_events[0].status, IntuneStatus::Success);

        let failed_events = extract_events(
            &[line(
                "[HS] the pre-remdiation detection script compliance result for 79880037-a3c4-489a-a7e6-a6a705b52b78 is False",
                "01-15-2024 10:00:06.000",
                1,
            )],
            "C:/Logs/HealthScripts.log",
        );

        assert_eq!(failed_events.len(), 1);
        assert_eq!(failed_events[0].status, IntuneStatus::Failed);
    }

    #[test]
    fn healthscripts_exit_code_is_parsed_with_is_syntax() {
        let events = extract_events(
            &[line(
                "[HS] exit code of the script is 0",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/HealthScripts.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].status, IntuneStatus::Success);
    }

    #[test]
    fn appworkload_install_lines_are_named_as_install_phase() {
        let events = extract_events(
            &[line(
                "AppWorkload launching install handoff for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AppWorkload.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::Win32App);
        assert_eq!(events[0].name, "AppWorkload Install (a1b2c3d4...)".to_string());
    }

    #[test]
    fn paired_completion_events_are_collapsed() {
        let events = extract_events(
            &[
                line(
                    "[Win32App] Processing app with id: a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "01-15-2024 10:00:00.000",
                    1,
                ),
                line(
                    "[Win32App] Completed successfully for app with id: a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "01-15-2024 10:01:00.000",
                    2,
                ),
            ],
            "C:/Logs/IntuneManagementExtension.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].status, IntuneStatus::Success);
        assert_eq!(events[0].end_time.as_deref(), Some("01-15-2024 10:01:00.000"));
        assert_eq!(events[0].duration_secs, Some(60.0));
    }
}
