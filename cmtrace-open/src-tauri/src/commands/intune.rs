use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tauri::async_runtime;

use crate::error_db::lookup::lookup_error_code;
use crate::intune::download_stats;
use crate::intune::event_tracker;
use crate::intune::ime_parser;
use crate::intune::models::{
    DownloadStat, IntuneAnalysisResult, IntuneDiagnosticInsight, IntuneDiagnosticSeverity,
    IntuneEvent, IntuneEventType, IntuneStatus, IntuneSummary,
};
use crate::intune::timeline;

const IME_LOG_PATTERNS: &[&str] = &[
    "intunemanagementextension",
    "appworkload",
    "appactionprocessor",
    "agentexecutor",
    "healthscripts",
    "clienthealth",
    "clientcertcheck",
    "devicehealthmonitoring",
    "sensor",
    "win32appinventory",
    "imeui",
];

/// Analyze Intune Management Extension logs and return structured results.
///
/// Supports either:
/// - A single IME log file path
/// - A directory containing IME logs (aggregated)
#[tauri::command]
pub async fn analyze_intune_logs(path: String) -> Result<IntuneAnalysisResult, String> {
    async_runtime::spawn_blocking(move || analyze_intune_logs_blocking(path))
        .await
        .map_err(|error| format!("Intune analysis task failed: {}", error))?
}

fn analyze_intune_logs_blocking(path: String) -> Result<IntuneAnalysisResult, String> {
    let analysis_started = Instant::now();
    eprintln!("event=intune_analysis_start path=\"{}\"", path);

    let input_path = Path::new(&path);
    let source_paths = collect_input_paths(input_path)?;
    eprintln!(
        "event=intune_analysis_sources_resolved path=\"{}\" source_count={}",
        path,
        source_paths.len()
    );

    let source_files: Vec<String> = source_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let mut all_events = Vec::new();
    let mut all_downloads = Vec::new();

    for source_path in &source_paths {
        let file_started = Instant::now();
        let source_file = source_path.to_string_lossy().to_string();
        eprintln!("event=intune_analysis_file_start file=\"{}\"", source_file);
        let content = fs::read_to_string(source_path)
            .map_err(|e| format!("Failed to read file '{}': {}", source_file, e))?;

        let lines = ime_parser::parse_ime_content(&content);
        if lines.is_empty() {
            eprintln!(
                "event=intune_analysis_file_complete file=\"{}\" line_count=0 event_count=0 download_count=0 elapsed_ms={}",
                source_file,
                file_started.elapsed().as_millis()
            );
            continue;
        }

        let file_events = event_tracker::extract_events(&lines, &source_file);
        let file_downloads = download_stats::extract_downloads(&lines, &source_file);

        eprintln!(
            "event=intune_analysis_file_complete file=\"{}\" line_count={} event_count={} download_count={} elapsed_ms={}",
            source_file,
            lines.len(),
            file_events.len(),
            file_downloads.len(),
            file_started.elapsed().as_millis()
        );

        all_events.extend(file_events);
        all_downloads.extend(file_downloads);
    }

    if all_events.is_empty() {
        let total_downloads = all_downloads.len() as u32;
        let successful_downloads = all_downloads.iter().filter(|download| download.success).count()
            as u32;
        let failed_downloads = all_downloads.iter().filter(|download| !download.success).count()
            as u32;
        let summary = IntuneSummary {
            total_events: 0,
            win32_apps: 0,
            winget_apps: 0,
            scripts: 0,
            remediations: 0,
            succeeded: 0,
            failed: 0,
            in_progress: 0,
            pending: 0,
            timed_out: 0,
            total_downloads,
            successful_downloads,
            failed_downloads,
            failed_scripts: 0,
            log_time_span: None,
        };
        let diagnostics = build_diagnostics(&[], &all_downloads, &summary);

        eprintln!(
            "event=intune_analysis_complete path=\"{}\" source_count={} event_count=0 download_count={} diagnostics_count={} elapsed_ms={}",
            path,
            source_files.len(),
            all_downloads.len(),
            diagnostics.len(),
            analysis_started.elapsed().as_millis()
        );

        return Ok(IntuneAnalysisResult {
            events: Vec::new(),
            downloads: all_downloads,
            summary,
            diagnostics,
            source_file: path,
            source_files,
        });
    }

    let events = timeline::build_timeline(all_events);
    let summary = build_summary(&events, &all_downloads);
    let diagnostics = build_diagnostics(&events, &all_downloads, &summary);
    let payload_chars: usize = events
        .iter()
        .map(|event| {
            event.name.len()
                + event.detail.len()
                + event.source_file.len()
                + event.error_code.as_ref().map_or(0, |value| value.len())
        })
        .sum();

    eprintln!(
        "event=intune_analysis_complete path=\"{}\" source_count={} event_count={} download_count={} diagnostics_count={} payload_chars={} elapsed_ms={}",
        path,
        source_files.len(),
        events.len(),
        all_downloads.len(),
        diagnostics.len(),
        payload_chars,
        analysis_started.elapsed().as_millis()
    );

    Ok(IntuneAnalysisResult {
        events,
        downloads: all_downloads,
        summary,
        diagnostics,
        source_file: path,
        source_files,
    })
}

/// Resolve a single file or a directory of Intune logs into a deterministic file list.
fn collect_input_paths(path: &Path) -> Result<Vec<PathBuf>, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to access path '{}': {}", path.display(), e))?;

    if metadata.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    if !metadata.is_dir() {
        return Err(format!(
            "Path '{}' is neither a file nor a directory",
            path.display()
        ));
    }

    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory '{}': {}", path.display(), e))?;

    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|entry_path| entry_path.is_file())
        .collect();

    files.sort_by_key(|p| {
        p.file_name()
            .map(|name| name.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default()
    });

    let mut ime_files: Vec<PathBuf> = files
        .iter()
        .filter(|p| is_ime_related_log_file(p))
        .cloned()
        .collect();

    if ime_files.is_empty() {
        ime_files = files.iter().filter(|p| is_log_file(p)).cloned().collect();
    }

    if ime_files.is_empty() {
        return Err(format!(
            "No .log files found in directory '{}'",
            path.display()
        ));
    }

    Ok(ime_files)
}

fn is_log_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case("log"))
        .unwrap_or(false)
}

fn is_ime_related_log_file(path: &Path) -> bool {
    if !is_log_file(path) {
        return false;
    }

    path.file_name()
        .map(|name| {
            let name = name.to_string_lossy().to_ascii_lowercase();
            IME_LOG_PATTERNS.iter().any(|pattern| name.contains(pattern))
        })
        .unwrap_or(false)
}

/// Build summary statistics from events and downloads.
fn build_summary(
    events: &[IntuneEvent],
    downloads: &[DownloadStat],
) -> IntuneSummary {
    let mut win32_apps = 0u32;
    let mut winget_apps = 0u32;
    let mut scripts = 0u32;
    let mut remediations = 0u32;
    let mut succeeded = 0u32;
    let mut failed = 0u32;
    let mut in_progress = 0u32;
    let mut pending = 0u32;
    let mut timed_out = 0u32;
    let mut failed_scripts = 0u32;

    for event in events {
        match event.event_type {
            IntuneEventType::Win32App => win32_apps += 1,
            IntuneEventType::WinGetApp => winget_apps += 1,
            IntuneEventType::PowerShellScript => scripts += 1,
            IntuneEventType::Remediation => remediations += 1,
            _ => {}
        }

        match event.status {
            IntuneStatus::Success => succeeded += 1,
            IntuneStatus::Failed => {
                failed += 1;
                if event.event_type == IntuneEventType::PowerShellScript {
                    failed_scripts += 1;
                }
            }
            IntuneStatus::InProgress => in_progress += 1,
            IntuneStatus::Pending => pending += 1,
            IntuneStatus::Timeout => {
                timed_out += 1;
                failed += 1;
                if event.event_type == IntuneEventType::PowerShellScript {
                    failed_scripts += 1;
                }
            }
            _ => {}
        }
    }

    let log_time_span = timeline::calculate_time_span(events);
    let successful_downloads = downloads.iter().filter(|download| download.success).count() as u32;
    let failed_downloads = downloads.iter().filter(|download| !download.success).count() as u32;

    IntuneSummary {
        total_events: events.len() as u32,
        win32_apps,
        winget_apps,
        scripts,
        remediations,
        succeeded,
        failed,
        in_progress,
        pending,
        timed_out,
        total_downloads: downloads.len() as u32,
        successful_downloads,
        failed_downloads,
        failed_scripts,
        log_time_span,
    }
}

fn build_diagnostics(
    events: &[IntuneEvent],
    downloads: &[DownloadStat],
    summary: &IntuneSummary,
) -> Vec<IntuneDiagnosticInsight> {
    let mut insights = Vec::new();

    let failed_download_events: Vec<&IntuneEvent> = events
        .iter()
        .filter(|event| {
            event.event_type == IntuneEventType::ContentDownload
                && matches!(event.status, IntuneStatus::Failed | IntuneStatus::Timeout)
        })
        .collect();
    let install_failures: Vec<&IntuneEvent> = events
        .iter()
        .filter(|event| {
            matches!(event.event_type, IntuneEventType::Win32App | IntuneEventType::WinGetApp)
                && matches!(event.status, IntuneStatus::Failed | IntuneStatus::Timeout)
                && contains_any(
                    &event.detail,
                    &[
                        "install",
                        "installer",
                        "execution",
                        "enforcement",
                        "launching install",
                    ],
                )
        })
        .collect();

    if summary.failed_downloads > 0 {
        let download_case = classify_download_failure_case(&failed_download_events, downloads);
        let mut evidence = vec![format!(
            "{} download attempt(s) ended in failure or retry exhaustion.",
            summary.failed_downloads
        )];
        evidence.extend(top_failed_download_labels(downloads, 2));
        evidence.extend(top_event_detail_matches(&failed_download_events, 2));
        if let Some(retries) = repeated_retry_evidence(&failed_download_events) {
            evidence.push(retries);
        }

        insights.push(IntuneDiagnosticInsight {
            id: "download-failures".to_string(),
            severity: IntuneDiagnosticSeverity::Error,
            title: download_case.title.to_string(),
            summary: download_case.summary.to_string(),
            evidence,
            next_checks: vec![
                "Review AppWorkload download, staging, and hash-validation lines for the affected content IDs.".to_string(),
                "Verify Delivery Optimization, proxy, VPN, or content-source reachability on the device.".to_string(),
                "Confirm the app payload is still available and matches the expected hash in Intune.".to_string(),
            ],
            suggested_fixes: download_case
                .suggested_fixes
                .into_iter()
                .map(|item| item.to_string())
                .collect(),
        });
    }

    if !install_failures.is_empty() {
        let install_hint = best_error_hint(&install_failures);
        let mut evidence = vec![format!(
            "{} app install or enforcement event(s) failed after download or staging work began.",
            install_failures.len()
        )];
        evidence.extend(top_event_labels(&install_failures, 3));
        evidence.extend(top_event_detail_matches(&install_failures, 2));
        if let Some(error_hint) = &install_hint {
            evidence.push(format!(
                "Most specific error observed: {} ({})",
                error_hint.code, error_hint.description
            ));
        }

        insights.push(IntuneDiagnosticInsight {
            id: "install-enforcement-failures".to_string(),
            severity: IntuneDiagnosticSeverity::Error,
            title: "App install or enforcement failures detected".to_string(),
            summary: "The workload progressed past content acquisition but failed during installer launch, enforcement, or completion tracking.".to_string(),
            evidence,
            next_checks: vec![
                "Inspect AppWorkload install and enforcement rows near the failure for the last successful phase before the installer returned control.".to_string(),
                "Compare the installer command, return code handling, and detection rule behavior for the affected app.".to_string(),
                "Correlate the failure with AgentExecutor or remediation activity if the deployment depends on prerequisite scripts.".to_string(),
            ],
            suggested_fixes: install_failure_suggested_fixes(install_hint),
        });
    }

    let timed_out_events: Vec<&IntuneEvent> = events
        .iter()
        .filter(|event| event.status == IntuneStatus::Timeout)
        .collect();
    if !timed_out_events.is_empty() {
        let mut evidence = vec![format!(
            "{} event(s) timed out before reporting a clean success or failure.",
            timed_out_events.len()
        )];
        evidence.extend(top_event_labels(&timed_out_events, 2));

        insights.push(IntuneDiagnosticInsight {
            id: "operation-timeouts".to_string(),
            severity: IntuneDiagnosticSeverity::Error,
            title: "Timed-out operations detected".to_string(),
            summary: "One or more app or script operations stalled long enough to be treated as failures.".to_string(),
            evidence,
            next_checks: vec![
                "Inspect the matching event rows around the timeout for the last successful phase before the stall.".to_string(),
                "Check whether install commands, detection scripts, or remediation scripts are waiting on external resources or user state.".to_string(),
                "Look for repeated retries or follow-on failure codes in AppWorkload, AgentExecutor, or HealthScripts logs.".to_string(),
            ],
            suggested_fixes: vec![
                "Shorten or optimize long-running installers or scripts that routinely exceed the IME execution window.".to_string(),
                "Remove dependencies on user interaction, mapped drives, or transient network resources during enforcement.".to_string(),
                "If the timeout is expected during first install, validate whether the assignment deadline or retry cadence needs adjustment.".to_string(),
            ],
        });
    }

    let script_failures: Vec<&IntuneEvent> = events
        .iter()
        .filter(|event| {
            matches!(
                event.event_type,
                IntuneEventType::PowerShellScript | IntuneEventType::Remediation
            ) && matches!(event.status, IntuneStatus::Failed | IntuneStatus::Timeout)
        })
        .collect();
    if !script_failures.is_empty() {
        let mut evidence = vec![format!(
            "{} script or remediation event(s) failed or timed out.",
            script_failures.len()
        )];
        evidence.extend(top_event_labels(&script_failures, 3));
        evidence.extend(top_event_detail_matches(&script_failures, 2));
        let script_hint = best_error_hint(&script_failures);
        if let Some(error_hint) = &script_hint {
            evidence.push(format!(
                "Most specific script error observed: {} ({})",
                error_hint.code, error_hint.description
            ));
        }

        insights.push(IntuneDiagnosticInsight {
            id: "script-failures".to_string(),
            severity: IntuneDiagnosticSeverity::Error,
            title: "Script execution failures detected".to_string(),
            summary: "Detection or remediation logic returned a non-zero outcome or never completed, which can block compliance or app enforcement.".to_string(),
            evidence,
            next_checks: vec![
                "Review AgentExecutor and HealthScripts entries for stdout, stderr, and explicit exit-code lines around the affected script.".to_string(),
                "Separate detection-script failures from remediation-script failures before deciding whether the issue is logic, environment, or permissions.".to_string(),
                "Validate script prerequisites such as execution context, file paths, and any required network or service dependencies.".to_string(),
            ],
            suggested_fixes: script_failure_suggested_fixes(&script_failures, script_hint),
        });
    }

    let policy_events: Vec<&IntuneEvent> = events
        .iter()
        .filter(|event| {
            event.event_type == IntuneEventType::PolicyEvaluation
                && event.status != IntuneStatus::Success
        })
        .collect();
    if !policy_events.is_empty() {
        let mut evidence = vec![format!(
            "{} policy or applicability event(s) did not end in success.",
            policy_events.len()
        )];
        evidence.extend(top_event_labels(&policy_events, 2));
        evidence.extend(top_event_detail_matches(&policy_events, 2));
        let not_applicable = policy_events
            .iter()
            .any(|event| contains_any(&event.detail, &["not applicable", "requirement rule", "detection rule"]));

        insights.push(IntuneDiagnosticInsight {
            id: "policy-applicability".to_string(),
            severity: IntuneDiagnosticSeverity::Warning,
            title: if not_applicable {
                "Applicability or requirement rules blocked enforcement".to_string()
            } else {
                "Policy applicability needs review".to_string()
            },
            summary: if not_applicable {
                "The deployment appears to be present in policy evaluation, but requirement or detection logic is stopping enforcement from continuing.".to_string()
            } else {
                "Assignment or applicability evaluation may be preventing enforcement even when content and scripts are available.".to_string()
            },
            evidence,
            next_checks: vec![
                "Review AppActionProcessor requirement-rule, detection-rule, and applicability lines for the affected app GUIDs.".to_string(),
                "Confirm the assignment intent, targeting, and any deadline or GRS behavior for the device or user.".to_string(),
                "Correlate policy-evaluation events with the later AppWorkload or AgentExecutor phases to see where enforcement stopped.".to_string(),
            ],
            suggested_fixes: if not_applicable {
                vec![
                    "Correct requirement-rule logic so the targeted device actually qualifies for the deployment.".to_string(),
                    "Verify that the detection rule is not falsely reporting the app as already present or already compliant.".to_string(),
                    "If the app should not target this device, adjust the assignment scope instead of forcing enforcement.".to_string(),
                ]
            } else {
                vec![
                    "Review assignment targeting, intent, and any deadlines or retry windows for the affected policy.".to_string(),
                    "Validate that prerequisite policies or dependent apps are not blocking the enforcement path.".to_string(),
                ]
            },
        });
    }

    if insights.is_empty() {
        if summary.in_progress > 0 || summary.pending > 0 {
            insights.push(IntuneDiagnosticInsight {
                id: "work-in-progress".to_string(),
                severity: IntuneDiagnosticSeverity::Info,
                title: "Workload still in progress".to_string(),
                summary: "The current IME snapshot shows pending or in-progress work without a dominant failure pattern yet.".to_string(),
                evidence: vec![
                    format!("{} event(s) are still in progress.", summary.in_progress),
                    format!("{} event(s) are still pending.", summary.pending),
                ],
                next_checks: vec![
                    "Re-check the logs after the next IME processing cycle to confirm whether the pending work resolves or fails.".to_string(),
                    "Use the timeline ordering to identify the most recent active app, download, or script phase.".to_string(),
                ],
                suggested_fixes: vec![
                    "Allow the current IME cycle to finish before changing the deployment unless a repeated stall pattern appears.".to_string(),
                ],
            });
        } else if summary.total_events > 0 {
            insights.push(IntuneDiagnosticInsight {
                id: "no-dominant-blocker".to_string(),
                severity: IntuneDiagnosticSeverity::Info,
                title: "No dominant blocker detected".to_string(),
                summary: "The analyzed IME logs do not show a strong failure cluster in downloads, scripts, policy evaluation, or timeouts.".to_string(),
                evidence: vec![
                    format!("{} event(s) succeeded.", summary.succeeded),
                    format!("{} total event(s) were analyzed.", summary.total_events),
                ],
                next_checks: vec![
                    "Inspect the timeline for the last non-success event if the user still reports a problem.".to_string(),
                    "Correlate IME activity with device state, portal assignment status, or Windows Event Logs if symptoms continue.".to_string(),
                ],
                suggested_fixes: vec![
                    "Do not change packaging or targeting yet; gather one failing sample with adjacent logs before tuning heuristics further.".to_string(),
                ],
            });
        }
    }

    insights
}

fn top_failed_download_labels(downloads: &[DownloadStat], limit: usize) -> Vec<String> {
    let mut labels = Vec::new();

    for download in downloads.iter().filter(|download| !download.success) {
        let label = if download.name.trim().is_empty() {
            format!("Affected content ID: {}", download.content_id)
        } else {
            format!("Affected content: {}", download.name)
        };

        if !labels.contains(&label) {
            labels.push(label);
        }

        if labels.len() >= limit {
            break;
        }
    }

    labels
}

struct ErrorHint {
    code: String,
    description: String,
}

struct DownloadFailureCase {
    title: &'static str,
    summary: &'static str,
    suggested_fixes: Vec<&'static str>,
}

fn classify_download_failure_case(
    events: &[&IntuneEvent],
    downloads: &[DownloadStat],
) -> DownloadFailureCase {
    if events
        .iter()
        .any(|event| contains_any(&event.detail, &["hash validation", "hash mismatch", "hash"]))
    {
        return DownloadFailureCase {
            title: "Content hash or staging validation failed",
            summary: "The device downloaded content, but staging or hash verification indicates the package may be incomplete, stale, or mismatched.",
            suggested_fixes: vec![
                "Re-upload or redistribute the app content in Intune so the device receives a clean package revision.",
                "Verify that the package contents and detection logic still match the deployed app version.",
                "Clear any stale cached content on the test device before retrying if hash mismatches keep repeating.",
            ],
        };
    }

    if events
        .iter()
        .any(|event| contains_any(&event.detail, &["staging", "content cached", "cache location"]))
    {
        return DownloadFailureCase {
            title: "Content staging failed after download",
            summary: "The workload reached caching or staging, but the local handoff into install-ready content did not complete successfully.",
            suggested_fixes: vec![
                "Validate local disk space and permissions on the IME content cache path.",
                "Retry with a fresh content download if cached payloads appear stale or partially written.",
                "Check antivirus or endpoint protection exclusions if staging repeatedly stops after the download completes.",
            ],
        };
    }

    if downloads.iter().any(|download| !download.success && download.do_percentage == 0.0) {
        return DownloadFailureCase {
            title: "Content retrieval failed before local staging",
            summary: "The workload is failing during content acquisition rather than install, and the logs do not show healthy Delivery Optimization contribution.",
            suggested_fixes: vec![
                "Validate proxy, VPN, firewall, and Delivery Optimization reachability for the content source.",
                "Test the same deployment on a network path without restrictive content filtering.",
                "Confirm the app content is still available and correctly assigned in Intune.",
            ],
        };
    }

    DownloadFailureCase {
        title: "Content download failures detected",
        summary: "App content did not download cleanly, so enforcement may never reach install or detection stages.",
        suggested_fixes: vec![
            "Confirm the app payload is still available and matches the expected content in Intune.",
            "Check device network reachability to Microsoft content endpoints and any proxy path in between.",
            "Retry with fresh logs after the next IME cycle to confirm whether this is a transient retrieval failure or a repeatable pattern.",
        ],
    }
}

fn best_error_hint(events: &[&IntuneEvent]) -> Option<ErrorHint> {
    for event in events {
        let Some(error_code) = &event.error_code else {
            continue;
        };

        let lookup = lookup_error_code(error_code);
        if lookup.found {
            return Some(ErrorHint {
                code: lookup.code_hex,
                description: lookup.description,
            });
        }

        return Some(ErrorHint {
            code: error_code.clone(),
            description: lookup.description,
        });
    }

    None
}

fn install_failure_suggested_fixes(error_hint: Option<ErrorHint>) -> Vec<String> {
    let mut fixes = vec![
        "Validate the install command line, return-code mapping, and required install context for the affected app.".to_string(),
        "Check whether the detection rule is declaring failure because the installer succeeded but the post-install signal is wrong.".to_string(),
        "Review prerequisite scripts or dependencies if the installer only fails when launched by IME.".to_string(),
    ];

    if let Some(hint) = error_hint {
        let description = hint.description.to_ascii_lowercase();
        if description.contains("access is denied") {
            fixes.insert(
                0,
                "Run the installer in the same system or user context expected by Intune and fix any file, registry, or service permission gaps.".to_string(),
            );
        } else if description.contains("file not found") || description.contains("path not found") {
            fixes.insert(
                0,
                "Verify that the installer command references files that actually exist after IME staging and extraction.".to_string(),
            );
        }
    }

    fixes
}

fn script_failure_suggested_fixes(
    events: &[&IntuneEvent],
    error_hint: Option<ErrorHint>,
) -> Vec<String> {
    let detection_failures = events
        .iter()
        .any(|event| contains_any(&event.name, &["detection script", "detection"]));
    let remediation_failures = events
        .iter()
        .any(|event| contains_any(&event.name, &["remediation script", "remediation"]));

    let mut fixes = Vec::new();
    if detection_failures {
        fixes.push(
            "Correct detection-script logic first; a false negative there can block install success even when the app is already present.".to_string(),
        );
    }
    if remediation_failures {
        fixes.push(
            "If remediation failed, validate every command path and dependency under the same execution context IME uses on the device.".to_string(),
        );
    }

    if let Some(hint) = error_hint {
        let description = hint.description.to_ascii_lowercase();
        if description.contains("access is denied") {
            fixes.push(
                "Grant the script access to the filesystem, registry, certificate store, or service endpoints it needs, or move the action to a supported elevation context.".to_string(),
            );
        } else if description.contains("file not found") || description.contains("path not found") {
            fixes.push(
                "Package or create any required script dependencies locally before the script runs, and avoid relying on missing relative paths.".to_string(),
            );
        }
    }

    fixes.push(
        "Capture stdout and stderr from the failing script path and test the same logic outside IME to isolate environment assumptions.".to_string(),
    );
    fixes
}

fn top_event_detail_matches(events: &[&IntuneEvent], limit: usize) -> Vec<String> {
    let mut labels = Vec::new();

    for event in events {
        let snippet = event.detail.trim();
        if snippet.is_empty() {
            continue;
        }

        let shortened = if snippet.len() > 120 {
            format!("{}...", &snippet[..120])
        } else {
            snippet.to_string()
        };
        let evidence = format!("Observed detail: {}", shortened);

        if !labels.contains(&evidence) {
            labels.push(evidence);
        }

        if labels.len() >= limit {
            break;
        }
    }

    labels
}

fn repeated_retry_evidence(events: &[&IntuneEvent]) -> Option<String> {
    let retry_count = events
        .iter()
        .filter(|event| contains_any(&event.detail, &["retry", "retrying", "reattempt", "will retry"]))
        .count();

    if retry_count > 0 {
        Some(format!(
            "Retry behavior was observed in {} failed download event(s).",
            retry_count
        ))
    } else {
        None
    }
}

fn contains_any(value: &str, terms: &[&str]) -> bool {
    let normalized = value.to_ascii_lowercase();
    terms.iter().any(|term| normalized.contains(&term.to_ascii_lowercase()))
}

fn top_event_labels(events: &[&IntuneEvent], limit: usize) -> Vec<String> {
    let mut labels = Vec::new();

    for event in events {
        let mut label = event.name.clone();
        if let Some(error_code) = &event.error_code {
            label.push_str(&format!(" (error {})", error_code));
        }

        let evidence = format!("Affected event: {}", label);
        if !labels.contains(&evidence) {
            labels.push(evidence);
        }

        if labels.len() >= limit {
            break;
        }
    }

    labels
}

#[cfg(test)]
mod tests {
    use super::{build_diagnostics, collect_input_paths};
    use crate::intune::models::{
        DownloadStat, IntuneDiagnosticSeverity, IntuneEvent, IntuneEventType, IntuneStatus,
        IntuneSummary,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn collect_input_paths_includes_ime_sidecar_logs_with_primary_log() {
        let test_dir = create_temp_dir("intune-aggregation");

        fs::write(test_dir.join("IntuneManagementExtension.log"), "primary")
            .expect("write primary log");
        fs::write(test_dir.join("AppWorkload.log"), "sidecar")
            .expect("write app workload log");
        fs::write(test_dir.join("AppActionProcessor.log"), "app actions")
            .expect("write app action processor log");
        fs::write(test_dir.join("AgentExecutor.log"), "executor")
            .expect("write agent executor log");
        fs::write(test_dir.join("HealthScripts.log"), "health scripts")
            .expect("write health scripts log");
        fs::write(test_dir.join("ClientHealth.log"), "client health")
            .expect("write client health log");
        fs::write(test_dir.join("ClientCertCheck.log"), "client cert")
            .expect("write client cert check log");
        fs::write(
            test_dir.join("DeviceHealthMonitoring.log"),
            "device health",
        )
        .expect("write device health monitoring log");
        fs::write(test_dir.join("Sensor.log"), "sensor")
            .expect("write sensor log");
        fs::write(test_dir.join("Win32AppInventory.log"), "inventory")
            .expect("write win32 app inventory log");
        fs::write(test_dir.join("ImeUI.log"), "ui")
            .expect("write ime ui log");
        fs::write(test_dir.join("random.log"), "other")
            .expect("write unrelated log");

        let collected = collect_input_paths(&test_dir).expect("collect input paths");
        let file_names: Vec<String> = collected
            .iter()
            .filter_map(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
            .collect();

        assert_eq!(
            file_names,
            vec![
                "AgentExecutor.log".to_string(),
                "AppActionProcessor.log".to_string(),
                "AppWorkload.log".to_string(),
                "ClientCertCheck.log".to_string(),
                "ClientHealth.log".to_string(),
                "DeviceHealthMonitoring.log".to_string(),
                "HealthScripts.log".to_string(),
                "ImeUI.log".to_string(),
                "IntuneManagementExtension.log".to_string(),
                "Sensor.log".to_string(),
                "Win32AppInventory.log".to_string(),
            ]
        );

        fs::remove_dir_all(&test_dir).expect("remove temp dir");
    }

    fn create_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{}-{}", prefix, unique));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn build_diagnostics_reports_download_and_script_failures() {
        let events = vec![
            IntuneEvent {
                id: 1,
                event_type: IntuneEventType::PowerShellScript,
                name: "AgentExecutor Detection Script (abcd1234...)".to_string(),
                guid: None,
                status: IntuneStatus::Failed,
                start_time: Some("01-15-2024 10:00:05.000".to_string()),
                end_time: None,
                duration_secs: None,
                error_code: Some("27".to_string()),
                detail: "Script failed".to_string(),
                source_file: "C:/Logs/AgentExecutor.log".to_string(),
                line_number: 12,
            },
            IntuneEvent {
                id: 2,
                event_type: IntuneEventType::PolicyEvaluation,
                name: "AppActionProcessor Applicability (abcd1234...)".to_string(),
                guid: None,
                status: IntuneStatus::Pending,
                start_time: Some("01-15-2024 10:01:05.000".to_string()),
                end_time: None,
                duration_secs: None,
                error_code: None,
                detail: "Applicability pending".to_string(),
                source_file: "C:/Logs/AppActionProcessor.log".to_string(),
                line_number: 18,
            },
            IntuneEvent {
                id: 3,
                event_type: IntuneEventType::ContentDownload,
                name: "AppWorkload Staging (abcd1234...)".to_string(),
                guid: None,
                status: IntuneStatus::Failed,
                start_time: Some("01-15-2024 10:02:05.000".to_string()),
                end_time: None,
                duration_secs: None,
                error_code: None,
                detail: "Hash validation failed after staging cached content".to_string(),
                source_file: "C:/Logs/AppWorkload.log".to_string(),
                line_number: 22,
            },
            IntuneEvent {
                id: 4,
                event_type: IntuneEventType::Win32App,
                name: "AppWorkload Install (abcd1234...)".to_string(),
                guid: None,
                status: IntuneStatus::Failed,
                start_time: Some("01-15-2024 10:03:05.000".to_string()),
                end_time: None,
                duration_secs: None,
                error_code: Some("0x80070005".to_string()),
                detail: "Installer execution failed with error code: 0x80070005".to_string(),
                source_file: "C:/Logs/AppWorkload.log".to_string(),
                line_number: 28,
            },
        ];
        let downloads = vec![DownloadStat {
            content_id: "content-1".to_string(),
            name: "Contoso App Payload".to_string(),
            size_bytes: 10,
            speed_bps: 1.0,
            do_percentage: 0.0,
            duration_secs: 5.0,
            success: false,
            timestamp: Some("01-15-2024 10:00:00.000".to_string()),
        }];
        let summary = IntuneSummary {
            total_events: 4,
            win32_apps: 1,
            winget_apps: 0,
            scripts: 1,
            remediations: 0,
            succeeded: 0,
            failed: 3,
            in_progress: 0,
            pending: 1,
            timed_out: 0,
            total_downloads: 1,
            successful_downloads: 0,
            failed_downloads: 1,
            failed_scripts: 1,
            log_time_span: None,
        };

        let diagnostics = build_diagnostics(&events, &downloads, &summary);

        assert_eq!(diagnostics.len(), 4);
        assert_eq!(diagnostics[0].id, "download-failures");
        assert_eq!(diagnostics[0].severity, IntuneDiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].title, "Content hash or staging validation failed");
        assert!(diagnostics[0]
            .evidence
            .iter()
            .any(|item| item.contains("Contoso App Payload")));
        assert!(diagnostics[0]
            .suggested_fixes
            .iter()
            .any(|item| item.contains("Re-upload or redistribute")));
        assert!(diagnostics.iter().any(|item| item.id == "script-failures"));
        assert!(diagnostics
            .iter()
            .any(|item| item.id == "install-enforcement-failures"));
        assert!(diagnostics.iter().any(|item| item.id == "policy-applicability"));

        let install = diagnostics
            .iter()
            .find(|item| item.id == "install-enforcement-failures")
            .expect("install diagnostic present");
        assert!(install
            .evidence
            .iter()
            .any(|item| item.contains("Access is denied")));
        assert!(install
            .suggested_fixes
            .iter()
            .any(|item| item.contains("same system or user context")));
    }
}
