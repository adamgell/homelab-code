use std::fs;

use crate::intune::download_stats;
use crate::intune::event_tracker;
use crate::intune::ime_parser;
use crate::intune::models::{IntuneAnalysisResult, IntuneEventType, IntuneStatus, IntuneSummary};
use crate::intune::timeline;

/// Analyze an Intune Management Extension log file and return structured results.
#[tauri::command]
pub fn analyze_intune_logs(path: String) -> Result<IntuneAnalysisResult, String> {
    // Read file content
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Phase 1: Parse IME log into structured lines
    let lines = ime_parser::parse_ime_content(&content);

    if lines.is_empty() {
        return Ok(IntuneAnalysisResult {
            events: Vec::new(),
            downloads: Vec::new(),
            summary: IntuneSummary {
                total_events: 0,
                win32_apps: 0,
                winget_apps: 0,
                scripts: 0,
                remediations: 0,
                succeeded: 0,
                failed: 0,
                in_progress: 0,
                total_downloads: 0,
                log_time_span: None,
            },
            source_file: path,
        });
    }

    // Phase 2: Extract events from parsed lines
    let events = event_tracker::extract_events(&lines, &path);

    // Phase 3: Build timeline (dedup + sort)
    let events = timeline::build_timeline(events);

    // Phase 4: Extract download statistics
    let downloads = download_stats::extract_downloads(&lines);

    // Phase 5: Build summary
    let summary = build_summary(&events, &downloads);

    Ok(IntuneAnalysisResult {
        events,
        downloads,
        summary,
        source_file: path,
    })
}

/// Build summary statistics from events and downloads.
fn build_summary(
    events: &[crate::intune::models::IntuneEvent],
    downloads: &[crate::intune::models::DownloadStat],
) -> IntuneSummary {
    let mut win32_apps = 0u32;
    let mut winget_apps = 0u32;
    let mut scripts = 0u32;
    let mut remediations = 0u32;
    let mut succeeded = 0u32;
    let mut failed = 0u32;
    let mut in_progress = 0u32;

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
            IntuneStatus::Failed => failed += 1,
            IntuneStatus::InProgress => in_progress += 1,
            _ => {}
        }
    }

    let log_time_span = timeline::calculate_time_span(events);

    IntuneSummary {
        total_events: events.len() as u32,
        win32_apps,
        winget_apps,
        scripts,
        remediations,
        succeeded,
        failed,
        in_progress,
        total_downloads: downloads.len() as u32,
        log_time_span,
    }
}
