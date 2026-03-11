use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::models::log_entry::ParseResult;
use crate::parser;
use crate::state::app_state::{AppState, OpenFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogSourceKind {
    File,
    Folder,
    Known,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KnownSourcePathKind {
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlatformKind {
    All,
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KnownSourceDefaultFileSelectionBehavior {
    None,
    PreferFileName,
    PreferFileNameThenPattern,
    PreferPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownSourceGroupingMetadata {
    pub family_id: String,
    pub family_label: String,
    pub group_id: String,
    pub group_label: String,
    pub group_order: u32,
    pub source_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownSourceDefaultFileIntent {
    pub selection_behavior: KnownSourceDefaultFileSelectionBehavior,
    pub preferred_file_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LogSource {
    File {
        path: String,
    },
    Folder {
        path: String,
    },
    Known {
        #[serde(rename = "sourceId")]
        source_id: String,
        #[serde(rename = "defaultPath")]
        default_path: String,
        #[serde(rename = "pathKind")]
        path_kind: KnownSourcePathKind,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: Option<u64>,
    pub modified_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderListingResult {
    pub source_kind: LogSourceKind,
    pub source: LogSource,
    pub entries: Vec<FolderEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownSourceMetadata {
    pub id: String,
    pub label: String,
    pub description: String,
    pub platform: PlatformKind,
    pub source_kind: LogSourceKind,
    pub source: LogSource,
    pub file_patterns: Vec<String>,
    #[serde(default)]
    pub grouping: Option<KnownSourceGroupingMetadata>,
    #[serde(default)]
    pub default_file_intent: Option<KnownSourceDefaultFileIntent>,
}

/// Open and parse a log file, auto-detecting its format.
/// Stores the detected format and date order in AppState for tail reading.
#[tauri::command]
pub fn open_log_file(path: String, state: State<'_, AppState>) -> Result<ParseResult, String> {
    let (result, date_order) = parser::parse_file(&path)?;

    // Store in AppState so the tail reader can use date_order later
    let mut open_files = state.open_files.lock().map_err(|e| e.to_string())?;
    open_files.insert(
        PathBuf::from(&path),
        OpenFile {
            path: PathBuf::from(&path),
            entries: vec![], // entries live in the frontend
            format: result.format_detected,
            byte_offset: result.byte_offset,
            date_order,
        },
    );

    Ok(result)
}

fn normalize_path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn metadata_modified_unix_ms(metadata: &fs::Metadata) -> Option<u64> {
    let duration = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    u64::try_from(duration.as_millis()).ok()
}

fn compare_folder_entries(left: &FolderEntry, right: &FolderEntry) -> Ordering {
    match (left.is_dir, right.is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => {
            let left_lower = left.name.to_lowercase();
            let right_lower = right.name.to_lowercase();

            left_lower
                .cmp(&right_lower)
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.path.cmp(&right.path))
        }
    }
}

/// List top-level entries for a folder source.
#[tauri::command]
pub fn list_log_folder(path: String) -> Result<FolderListingResult, String> {
    eprintln!("event=list_log_folder_start path=\"{}\"", path);

    let requested_path = PathBuf::from(&path);

    if !requested_path.exists() {
        return Err(format!(
            "folder does not exist: {}",
            requested_path.display()
        ));
    }

    if !requested_path.is_dir() {
        return Err(format!(
            "path is not a folder: {}",
            requested_path.display()
        ));
    }

    let read_dir = fs::read_dir(&requested_path)
        .map_err(|e| format!("failed to read folder {}: {e}", requested_path.display()))?;

    let mut entries: Vec<FolderEntry> = Vec::new();

    for entry_result in read_dir {
        let entry = match entry_result {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "event=list_log_folder_skip reason=read_dir_entry_error path=\"{}\" error=\"{}\"",
                    requested_path.display(),
                    error
                );
                continue;
            }
        };

        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "event=list_log_folder_skip reason=metadata_error entry_path=\"{}\" error=\"{}\"",
                    entry_path.display(),
                    error
                );
                continue;
            }
        };

        entries.push(FolderEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: normalize_path_string(&entry_path),
            is_dir: metadata.is_dir(),
            size_bytes: if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            },
            modified_unix_ms: metadata_modified_unix_ms(&metadata),
        });
    }

    entries.sort_by(compare_folder_entries);

    eprintln!(
        "event=list_log_folder_complete path=\"{}\" entry_count={}",
        requested_path.display(),
        entries.len()
    );

    Ok(FolderListingResult {
        source_kind: LogSourceKind::Folder,
        source: LogSource::Folder {
            path: normalize_path_string(&requested_path),
        },
        entries,
    })
}

#[cfg(target_os = "windows")]
fn windows_known_source(
    id: &str,
    label: &str,
    description: &str,
    path_kind: KnownSourcePathKind,
    default_path: &str,
    file_patterns: &[&str],
    grouping: KnownSourceGroupingMetadata,
    default_file_intent: Option<KnownSourceDefaultFileIntent>,
) -> KnownSourceMetadata {
    let id_text = id.to_string();

    KnownSourceMetadata {
        id: id_text.clone(),
        label: label.to_string(),
        description: description.to_string(),
        platform: PlatformKind::Windows,
        source_kind: LogSourceKind::Known,
        source: LogSource::Known {
            source_id: id_text,
            default_path: default_path.to_string(),
            path_kind,
        },
        file_patterns: file_patterns
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        grouping: Some(grouping),
        default_file_intent,
    }
}

#[cfg(target_os = "windows")]
fn windows_known_log_sources() -> Vec<KnownSourceMetadata> {
    vec![
        windows_known_source(
            "windows-intune-ime-logs",
            "Intune Management Extension Logs",
            "Primary Intune Win32 app and script diagnostics folder.",
            KnownSourcePathKind::Folder,
            "C:\\ProgramData\\Microsoft\\IntuneManagementExtension\\Logs",
            &[
                "IntuneManagementExtension.log",
                "AppWorkload.log",
                "AppActionProcessor.log",
                "AgentExecutor.log",
                "HealthScripts.log",
                "*.log",
            ],
            KnownSourceGroupingMetadata {
                family_id: "windows-intune".to_string(),
                family_label: "Windows Intune".to_string(),
                group_id: "intune-ime".to_string(),
                group_label: "Intune Management Extension".to_string(),
                group_order: 10,
                source_order: 10,
            },
            Some(KnownSourceDefaultFileIntent {
                selection_behavior:
                    KnownSourceDefaultFileSelectionBehavior::PreferFileNameThenPattern,
                preferred_file_names: vec![
                    "IntuneManagementExtension.log".to_string(),
                    "AppWorkload.log".to_string(),
                    "AppActionProcessor.log".to_string(),
                    "AgentExecutor.log".to_string(),
                    "HealthScripts.log".to_string(),
                ],
            }),
        ),
        windows_known_source(
            "windows-intune-ime-intunemanagementextension-log",
            "Intune Management Extension: IntuneManagementExtension.log",
            "Primary IME log for check-ins, policy processing, and app orchestration.",
            KnownSourcePathKind::File,
            "C:\\ProgramData\\Microsoft\\IntuneManagementExtension\\Logs\\IntuneManagementExtension.log",
            &["IntuneManagementExtension*.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-intune".to_string(),
                family_label: "Windows Intune".to_string(),
                group_id: "intune-ime".to_string(),
                group_label: "Intune Management Extension".to_string(),
                group_order: 10,
                source_order: 20,
            },
            None,
        ),
        windows_known_source(
            "windows-intune-ime-appworkload-log",
            "Intune Management Extension: AppWorkload.log",
            "Win32 and WinGet app download/staging/install diagnostics.",
            KnownSourcePathKind::File,
            "C:\\ProgramData\\Microsoft\\IntuneManagementExtension\\Logs\\AppWorkload.log",
            &["AppWorkload*.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-intune".to_string(),
                family_label: "Windows Intune".to_string(),
                group_id: "intune-ime".to_string(),
                group_label: "Intune Management Extension".to_string(),
                group_order: 10,
                source_order: 30,
            },
            None,
        ),
        windows_known_source(
            "windows-intune-ime-agentexecutor-log",
            "Intune Management Extension: AgentExecutor.log",
            "Script execution and remediation output with exit code tracking.",
            KnownSourcePathKind::File,
            "C:\\ProgramData\\Microsoft\\IntuneManagementExtension\\Logs\\AgentExecutor.log",
            &["AgentExecutor*.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-intune".to_string(),
                family_label: "Windows Intune".to_string(),
                group_id: "intune-ime".to_string(),
                group_label: "Intune Management Extension".to_string(),
                group_order: 10,
                source_order: 40,
            },
            None,
        ),
        windows_known_source(
            "windows-dmclient-logs",
            "DMClient Local Logs",
            "MDM DMClient log folder used for local sync diagnostics.",
            KnownSourcePathKind::Folder,
            "C:\\Windows\\System32\\config\\systemprofile\\AppData\\Local\\mdm",
            &["*.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-intune".to_string(),
                family_label: "Windows Intune".to_string(),
                group_id: "intune-mdm".to_string(),
                group_label: "MDM and Enrollment".to_string(),
                group_order: 20,
                source_order: 10,
            },
            Some(KnownSourceDefaultFileIntent {
                selection_behavior: KnownSourceDefaultFileSelectionBehavior::PreferPattern,
                preferred_file_names: Vec::new(),
            }),
        ),
        windows_known_source(
            "windows-panther-setupact-log",
            "setupact.log (Panther)",
            "Primary Windows setup and Autopilot/OOBE action log.",
            KnownSourcePathKind::File,
            "C:\\Windows\\Panther\\setupact.log",
            &["setupact.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-setup".to_string(),
                family_label: "Windows Setup".to_string(),
                group_id: "setup-panther".to_string(),
                group_label: "Panther".to_string(),
                group_order: 30,
                source_order: 10,
            },
            None,
        ),
        windows_known_source(
            "windows-panther-setuperr-log",
            "setuperr.log (Panther)",
            "Error-focused Windows setup and Autopilot/OOBE triage log.",
            KnownSourcePathKind::File,
            "C:\\Windows\\Panther\\setuperr.log",
            &["setuperr.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-setup".to_string(),
                family_label: "Windows Setup".to_string(),
                group_id: "setup-panther".to_string(),
                group_label: "Panther".to_string(),
                group_order: 30,
                source_order: 20,
            },
            None,
        ),
        windows_known_source(
            "windows-cbs-log",
            "CBS.log",
            "Component-Based Servicing log for update and servicing failures.",
            KnownSourcePathKind::File,
            "C:\\Windows\\Logs\\CBS\\CBS.log",
            &["CBS.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-servicing".to_string(),
                family_label: "Windows Servicing".to_string(),
                group_id: "servicing-core".to_string(),
                group_label: "CBS and DISM".to_string(),
                group_order: 40,
                source_order: 10,
            },
            None,
        ),
        windows_known_source(
            "windows-dism-log",
            "DISM.log",
            "Deployment Image Servicing and Management diagnostics log.",
            KnownSourcePathKind::File,
            "C:\\Windows\\Logs\\DISM\\dism.log",
            &["dism.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-servicing".to_string(),
                family_label: "Windows Servicing".to_string(),
                group_id: "servicing-core".to_string(),
                group_label: "CBS and DISM".to_string(),
                group_order: 40,
                source_order: 20,
            },
            None,
        ),
        windows_known_source(
            "windows-reporting-events-log",
            "ReportingEvents.log",
            "Windows Update transaction history in tab-delimited text.",
            KnownSourcePathKind::File,
            "C:\\Windows\\SoftwareDistribution\\ReportingEvents.log",
            &["ReportingEvents.log"],
            KnownSourceGroupingMetadata {
                family_id: "windows-servicing".to_string(),
                family_label: "Windows Servicing".to_string(),
                group_id: "servicing-update".to_string(),
                group_label: "Windows Update".to_string(),
                group_order: 40,
                source_order: 30,
            },
            None,
        ),
    ]
}

fn build_known_log_sources() -> Vec<KnownSourceMetadata> {
    let mut sources: Vec<KnownSourceMetadata> = Vec::new();

    #[cfg(target_os = "windows")]
    {
        sources.extend(windows_known_log_sources());
    }

    sources
}

/// Return known platform log source metadata.
#[tauri::command]
pub fn get_known_log_sources() -> Result<Vec<KnownSourceMetadata>, String> {
    let sources = build_known_log_sources();

    eprintln!("event=get_known_log_sources count={}", sources.len());

    Ok(sources)
}
