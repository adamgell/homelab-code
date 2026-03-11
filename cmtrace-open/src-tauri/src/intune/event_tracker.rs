use std::collections::{HashMap, HashSet};
use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;

use super::ime_parser::ImeLine;
use super::models::{IntuneEvent, IntuneEventType, IntuneStatus};

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
static WINGET_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)WinGetApp.*(?:processing|installing|detected|evaluating)"#).unwrap()
});
static WINGET_TOKEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:winget|microsoft\.winget)"#).unwrap());
static APPWORKLOAD_DOWNLOAD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:download(?:ing|ed)?|delivery\s+optimization|content\s+download|bytes\s+downloaded|download\s+progress|download\s+session|content\s+retrieval)"#,
    )
    .unwrap()
});
static APPWORKLOAD_STAGING_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:staging\s+(?:file|content)|hash\s+validation|content\s+cached|cache\s+location|expanded\s+content|extract(?:ed|ing))"#,
    )
    .unwrap()
});
static APPWORKLOAD_HASH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:hash\s+validation|hash\s+mismatch|hash\s+check)"#).unwrap());
static APPWORKLOAD_INSTALL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:install(?:ing|ation)?|execution|enforcement|installer|launching\s+install|handoff\s+to\s+install|processdetectionrules|detection\s+rule)"#,
    )
    .unwrap()
});
static APPWORKLOAD_RETRY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:retry|retrying|reattempt|will\s+retry|attempt\s+\d+\s+of\s+\d+)"#)
        .unwrap()
});
static APPWORKLOAD_STALL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:stalled|not\s+progressing|no\s+progress|timed?\s*out|timeout|hung|retry\s+exhausted)"#,
    )
    .unwrap()
});
static APPWORKLOAD_QUEUE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:queued|queueing|requesting\s+download|waiting\s+for\s+download|waiting\s+for\s+content|pending\s+download)"#,
    )
    .unwrap()
});
static APPWORKLOAD_FAILURE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:download\s+failed|failed\s+to\s+download|hash\s+validation\s+failed|hash\s+mismatch|staging\s+failed|unable\s+to\s+download|content\s+not\s+found|cancelled|aborted|retry\s+exhausted)"#,
    )
    .unwrap()
});
static APPWORKLOAD_SUCCESS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:download\s+completed|download\s+succeeded|staging\s+completed|content\s+cached|hash\s+validation\s+succeeded|install\s+completed|completed\s+successfully)"#,
    )
    .unwrap()
});
static POLICY_EVAL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:assignment\s+evaluation|targeted\s+intent|applicability\s*=|\bapplicable\b|not\s+applicable|requirement\s+rule|detection\s+rule|local\s+deadline|grs\s+expired|grs\s+not\s+expired|enforcement\s+classification|will\s+not\s+be\s+enforced)"#,
    )
    .unwrap()
});
static APP_ACTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:app\s+with\s+id:|application\s+action|managed\s+app)"#).unwrap()
});
static APPLICABILITY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:applicability|applicable|not\s+applicable|requirement\s+rule|detection\s+rule|will\s+not\s+be\s+enforced)"#)
        .unwrap()
});
static APPLICABILITY_BLOCK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:not\s+applicable|applicability\s*(?:=|:)\s*false|will\s+not\s+be\s+enforced|requirement\s+rule.*(?:not\s+satisfied|failed|false)|assignment.*(?:not\s+applicable|not\s+targeted)|enforcement\s+classification.*not\s+applicable)"#,
    )
    .unwrap()
});
static APPLICABILITY_SUCCESS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:applicability\s*(?:=|:)\s*true|requirement\s+rule.*(?:passed|satisfied|true)|assignment\s+evaluation\s+completed|applicable\s*=\s*true)"#,
    )
    .unwrap()
});
static POLICY_PENDING_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:local\s+deadline|grs\s+not\s+expired|scheduled|queued|pending)"#).unwrap()
});
static SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:PowerShell\s+script|script\s+execution|running\s+script)"#).unwrap()
});
static SCRIPT_RESULT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)script.*(?:completed|exit\s+code|result|output|failed|success)"#).unwrap()
});
static AGENTEXECUTOR_SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:powershell\s+script\s+is\s+successfully\s+executed|detection\s+script|remediation\s+script|exit\s+code|script\s+(?:completed|failed|timed?\s*out|execution)|stdout|stderr)"#,
    )
    .unwrap()
});
static DETECTION_SCRIPT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\b(?:pre-)?detection\s+script\b|\bpre-detect\b"#).unwrap());
static REMEDIATION_SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\b(?:post-)?remediation\s+script\b|\bpost-detect\b|\bremediation\b"#).unwrap()
});
static SCRIPT_FAILURE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:access\s+is\s+denied|unauthorized|permission\s+denied|term\s+'.+'\s+is\s+not\s+recognized|cannot\s+find\s+path|path\s+not\s+found|file\s+not\s+found|module\s+.*\s+not\s+found|parsererror|syntax\s+error|execution\s+policy|digitally\s+signed|failed\s+to\s+execute|exception|stderr)"#,
    )
    .unwrap()
});
static REMEDIATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:Remediation|HealthScript|proactive\s+remediation)"#).unwrap()
});
static HEALTHSCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:healthscript|health\s+script|detection\s+script|remediation\s+script|pre-detect|post-detect|schedule(?:d|ing)?|compliance\s+result)"#,
    )
    .unwrap()
});
static ESP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:ESP|EspBody|EnrollmentStatusPage|enrollment\s+status)"#).unwrap()
});
static SYNC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:sync\s+session|check-in|SyncSession)"#).unwrap());
static GUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})"#)
        .unwrap()
});
static ERROR_CODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:error\s*(?:code)?|exit\s*code(?:\s+of\s+the\s+script)?|hresult|hr|result|return\s*code)\s*(?:is|[=:])\s*(0x[0-9a-fA-F]+|-?\d+)"#,
    )
    .unwrap()
});
static EXIT_CODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)exit\s*code(?:\s+of\s+the\s+script)?\s*(?:is|[=:])\s*(-?\d+)"#)
        .unwrap()
});
static PENDING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:pending|queued|waiting|scheduled|requesting)"#).unwrap());
static TIMEOUT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:timed?\s*out|timeout|stalled|hung|not\s+progressing|no\s+progress)"#)
        .unwrap()
});
static COMPLIANCE_TRUE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)compliance\s+result.*\bis\s+true\b"#).unwrap());
static COMPLIANCE_FALSE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)compliance\s+result.*\bis\s+false\b"#).unwrap());
static SUCCESS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:success|succeeded|completed\s+successfully|installed|detected|compliant|validated|passed)"#,
    )
    .unwrap()
});
static FAILED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(?:fail|error|not\s+detected|not\s+installed|non-compliant|cancelled|aborted|exception)"#,
    )
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

pub fn extract_events(lines: &[ImeLine], source_file: &str) -> Vec<IntuneEvent> {
    let mut events = Vec::new();
    let mut next_id = 0u64;
    let source_kind = classify_source_kind(source_file);

    for line in lines {
        let Some(event_type) = detect_event_type(&line.message, source_kind) else {
            continue;
        };

        let guid = extract_guid(&line.message);
        let status = determine_status(&line.message, source_kind);
        let name = build_event_name(&event_type, &guid, &line.message, source_kind);
        let detail = if line.message.len() > 300 {
            format!("{}...", &line.message[..300])
        } else {
            line.message.clone()
        };

        events.push(IntuneEvent {
            id: next_id,
            event_type,
            name,
            guid,
            status,
            start_time: line.timestamp.clone(),
            end_time: None,
            duration_secs: None,
            error_code: extract_error_code(&line.message),
            detail,
            source_file: source_file.to_string(),
            line_number: line.line_number,
        });
        next_id += 1;
    }

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

fn detect_event_type(msg: &str, source_kind: ImeSourceKind) -> Option<IntuneEventType> {
    match source_kind {
        ImeSourceKind::AppWorkload => {
            if !is_appworkload_event_candidate(msg) {
                return None;
            }
            if WINGET_TOKEN_RE.is_match(msg) || WINGET_RE.is_match(msg) {
                return Some(IntuneEventType::WinGetApp);
            }
            if APPWORKLOAD_DOWNLOAD_RE.is_match(msg)
                || APPWORKLOAD_STAGING_RE.is_match(msg)
                || APPWORKLOAD_RETRY_RE.is_match(msg)
                || APPWORKLOAD_STALL_RE.is_match(msg)
            {
                return Some(IntuneEventType::ContentDownload);
            }
            if APPWORKLOAD_INSTALL_RE.is_match(msg) {
                return Some(IntuneEventType::Win32App);
            }
        }
        ImeSourceKind::AppActionProcessor => {
            if is_app_action_processor_event_candidate(msg)
                && (POLICY_EVAL_RE.is_match(msg)
                    || APP_ACTION_RE.is_match(msg)
                    || APPLICABILITY_RE.is_match(msg))
            {
                return Some(IntuneEventType::PolicyEvaluation);
            }
        }
        ImeSourceKind::AgentExecutor => {
            if !is_agent_executor_event_candidate(msg) {
                return None;
            }
            if REMEDIATION_SCRIPT_RE.is_match(msg) || REMEDIATION_RE.is_match(msg) {
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
            if is_healthscripts_event_candidate(msg)
                && (HEALTHSCRIPT_RE.is_match(msg) || REMEDIATION_RE.is_match(msg))
            {
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
    if normalized.ends_with(".timeout")
        || normalized.ends_with("quotedtimeoutfilepath.txt")
        || normalized.contains("prepare to run powershell script")
        || normalized.contains("remediation script option gets invoked")
        || normalized.contains("creating command line parser")
        || normalized.contains("adding argument")
        || normalized.contains("powershell path is")
    {
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
        || normalized.contains("stdout")
        || normalized.contains("stderr")
        || normalized.contains("exit code")
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

    APPWORKLOAD_DOWNLOAD_RE.is_match(msg)
        || APPWORKLOAD_STAGING_RE.is_match(msg)
        || APPWORKLOAD_INSTALL_RE.is_match(msg)
        || APPWORKLOAD_RETRY_RE.is_match(msg)
        || APPWORKLOAD_STALL_RE.is_match(msg)
        || WINGET_TOKEN_RE.is_match(msg)
}

fn is_app_action_processor_event_candidate(msg: &str) -> bool {
    let normalized = msg.to_ascii_lowercase();
    !(normalized.contains("processor initializing")
        || (normalized.contains("found:") && normalized.contains("apps with intent"))
        || normalized.contains("evaluating install enforcement actions for app with id")
        || normalized.contains("no action required for app with id"))
        && (normalized.contains("app with id:")
            || normalized.contains("assignment evaluation")
            || normalized.contains("targeted intent")
            || normalized.contains("applicability =")
            || normalized.contains("not applicable")
            || normalized.contains("local deadline")
            || normalized.contains("grs expired")
            || normalized.contains("grs not expired")
            || normalized.contains("requirement rule")
            || normalized.contains("detection rule")
            || normalized.contains("will not be enforced"))
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
        || normalized.contains("schedule")
}

fn extract_guid(msg: &str) -> Option<String> {
    WIN32_GUID_RE
        .captures(msg)
        .and_then(|cap| cap.get(1))
        .map(|value| value.as_str().to_string())
        .or_else(|| {
            GUID_RE
                .captures(msg)
                .and_then(|cap| cap.get(1))
                .map(|value| value.as_str().to_string())
        })
}

fn extract_error_code(msg: &str) -> Option<String> {
    ERROR_CODE_RE
        .captures(msg)
        .and_then(|cap| cap.get(1))
        .map(|value| value.as_str().to_string())
}

fn determine_status(msg: &str, source_kind: ImeSourceKind) -> IntuneStatus {
    if COMPLIANCE_TRUE_RE.is_match(msg) {
        return IntuneStatus::Success;
    }
    if COMPLIANCE_FALSE_RE.is_match(msg) {
        return IntuneStatus::Failed;
    }
    if let Some(exit_code) = EXIT_CODE_RE
        .captures(msg)
        .and_then(|cap| cap.get(1))
        .and_then(|value| value.as_str().parse::<i32>().ok())
    {
        return if exit_code == 0 {
            IntuneStatus::Success
        } else {
            IntuneStatus::Failed
        };
    }

    match source_kind {
        ImeSourceKind::AppWorkload => {
            if APPWORKLOAD_STALL_RE.is_match(msg) || TIMEOUT_RE.is_match(msg) {
                IntuneStatus::Timeout
            } else if APPWORKLOAD_RETRY_RE.is_match(msg) || APPWORKLOAD_FAILURE_RE.is_match(msg) {
                IntuneStatus::Failed
            } else if APPWORKLOAD_SUCCESS_RE.is_match(msg) {
                IntuneStatus::Success
            } else if APPWORKLOAD_QUEUE_RE.is_match(msg) || PENDING_RE.is_match(msg) {
                IntuneStatus::Pending
            } else {
                IntuneStatus::InProgress
            }
        }
        ImeSourceKind::AppActionProcessor => {
            if APPLICABILITY_BLOCK_RE.is_match(msg) {
                IntuneStatus::Failed
            } else if APPLICABILITY_SUCCESS_RE.is_match(msg) {
                IntuneStatus::Success
            } else if POLICY_PENDING_RE.is_match(msg) {
                IntuneStatus::Pending
            } else {
                IntuneStatus::InProgress
            }
        }
        ImeSourceKind::AgentExecutor | ImeSourceKind::HealthScripts => {
            if TIMEOUT_RE.is_match(msg) {
                IntuneStatus::Timeout
            } else if SCRIPT_FAILURE_RE.is_match(msg) || FAILED_RE.is_match(msg) {
                IntuneStatus::Failed
            } else if SUCCESS_RE.is_match(msg) {
                IntuneStatus::Success
            } else if PENDING_RE.is_match(msg) {
                IntuneStatus::Pending
            } else {
                IntuneStatus::InProgress
            }
        }
        ImeSourceKind::PrimaryIme | ImeSourceKind::Other => {
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
    }
}

fn build_event_name(
    event_type: &IntuneEventType,
    guid: &Option<String>,
    msg: &str,
    source_kind: ImeSourceKind,
) -> String {
    if let Some(name) = build_source_specific_name(event_type, guid, msg, source_kind) {
        return name;
    }

    let label = match event_type {
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
        let short = short_guid(guid);
        format!("{label} ({short}...)")
    } else {
        format!("{label}: {}", msg.chars().take(50).collect::<String>())
    }
}

fn build_source_specific_name(
    event_type: &IntuneEventType,
    guid: &Option<String>,
    msg: &str,
    source_kind: ImeSourceKind,
) -> Option<String> {
    let short_guid = guid.as_deref().map(short_guid);

    match source_kind {
        ImeSourceKind::AppWorkload => {
            let phase = if APPWORKLOAD_STALL_RE.is_match(msg) {
                "Download Stall"
            } else if APPWORKLOAD_RETRY_RE.is_match(msg) {
                "Download Retry"
            } else if APPWORKLOAD_HASH_RE.is_match(msg) {
                "Hash Validation"
            } else if APPWORKLOAD_STAGING_RE.is_match(msg) {
                "Staging"
            } else if APPWORKLOAD_INSTALL_RE.is_match(msg) {
                "Install"
            } else if APPWORKLOAD_DOWNLOAD_RE.is_match(msg) {
                "Download"
            } else {
                return None;
            };
            Some(match short_guid {
                Some(short) => format!("AppWorkload {phase} ({short}...)") ,
                None => format!("AppWorkload {phase}"),
            })
        }
        ImeSourceKind::AppActionProcessor => {
            let area = if APPLICABILITY_BLOCK_RE.is_match(msg) || APPLICABILITY_RE.is_match(msg) {
                "Applicability"
            } else if contains_case_insensitive(msg, "requirement rule") {
                "Requirement Rule"
            } else if contains_case_insensitive(msg, "detection rule") {
                "Detection Rule"
            } else if POLICY_EVAL_RE.is_match(msg) {
                "Policy Evaluation"
            } else {
                return None;
            };
            Some(match short_guid {
                Some(short) => format!("AppActionProcessor {area} ({short}...)") ,
                None => format!("AppActionProcessor {area}"),
            })
        }
        ImeSourceKind::AgentExecutor => {
            let area = if REMEDIATION_SCRIPT_RE.is_match(msg) || *event_type == IntuneEventType::Remediation {
                "Remediation Script"
            } else if DETECTION_SCRIPT_RE.is_match(msg) {
                "Detection Script"
            } else if TIMEOUT_RE.is_match(msg) {
                "Script Timeout"
            } else {
                "PowerShell Script"
            };
            Some(match short_guid {
                Some(short) => format!("AgentExecutor {area} ({short}...)") ,
                None => format!("AgentExecutor {area}"),
            })
        }
        ImeSourceKind::HealthScripts => {
            let area = if REMEDIATION_SCRIPT_RE.is_match(msg) {
                "Remediation"
            } else if DETECTION_SCRIPT_RE.is_match(msg)
                || contains_case_insensitive(msg, "compliance result")
            {
                "Detection"
            } else {
                "Schedule"
            };
            Some(match short_guid {
                Some(short) => format!("HealthScripts {area} ({short}...)") ,
                None => format!("HealthScripts {area}"),
            })
        }
        ImeSourceKind::PrimaryIme | ImeSourceKind::Other => None,
    }
}

fn short_guid(value: &str) -> &str {
    if value.len() > 8 {
        &value[..8]
    } else {
        value
    }
}

fn contains_case_insensitive(value: &str, needle: &str) -> bool {
    value.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
}

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

        let Some(start_index) = open_events.get_mut(&identity_key).and_then(|indices| indices.pop()) else {
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
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ")
}

fn estimate_duration(start: &str, end: &str) -> Option<f64> {
    let parse_seconds = |ts: &str| -> Option<f64> {
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
    fn appworkload_extracts_stalled_download_events() {
        let events = extract_events(
            &[line(
                "AppWorkload download stalled with no progress for app id: a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AppWorkload.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::ContentDownload);
        assert_eq!(events[0].status, IntuneStatus::Timeout);
    }

    #[test]
    fn app_action_processor_marks_not_applicable_as_failed() {
        let events = extract_events(
            &[line(
                "Assignment evaluation found app is not applicable and will not be enforced for app with id: a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AppActionProcessor.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::PolicyEvaluation);
        assert_eq!(events[0].status, IntuneStatus::Failed);
    }

    #[test]
    fn agent_executor_extracts_remediation_timeout() {
        let events = extract_events(
            &[line(
                "AgentExecutor remediation script timed out for package a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "01-15-2024 10:00:05.000",
                1,
            )],
            "C:/Logs/AgentExecutor.log",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, IntuneEventType::Remediation);
        assert_eq!(events[0].status, IntuneStatus::Timeout);
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
            ],
            "C:/Logs/AgentExecutor.log",
        );

        assert!(events.is_empty());
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
        assert_eq!(events[0].duration_secs, Some(60.0));
    }
}
