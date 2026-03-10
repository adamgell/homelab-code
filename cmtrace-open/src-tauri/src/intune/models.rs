use serde::{Deserialize, Serialize};

/// Type of Intune event detected from log analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntuneEventType {
    Win32App,
    WinGetApp,
    PowerShellScript,
    Remediation,
    Esp,
    SyncSession,
    PolicyEvaluation,
    ContentDownload,
    Other,
}

/// Status of an Intune operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntuneStatus {
    Success,
    Failed,
    InProgress,
    Pending,
    Timeout,
    Unknown,
}

/// A single Intune event extracted from log analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntuneEvent {
    /// Unique identifier for this event
    pub id: u64,
    /// Event type category
    pub event_type: IntuneEventType,
    /// Display name (resolved from GUID or extracted from context)
    pub name: String,
    /// GUID identifier if available
    pub guid: Option<String>,
    /// Status of the operation
    pub status: IntuneStatus,
    /// Timestamp of event start (ISO 8601 string)
    pub start_time: Option<String>,
    /// Timestamp of event end (ISO 8601 string)
    pub end_time: Option<String>,
    /// Duration in seconds
    pub duration_secs: Option<f64>,
    /// Error code if failed
    pub error_code: Option<String>,
    /// Additional detail message
    pub detail: String,
    /// Source log file path
    pub source_file: String,
    /// Line number in source file
    pub line_number: u32,
}

/// Download statistics for a content download event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStat {
    /// Name or ID of the content
    pub content_id: String,
    /// Display name
    pub name: String,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Download speed in bytes per second
    pub speed_bps: f64,
    /// Percentage downloaded via Delivery Optimization
    pub do_percentage: f64,
    /// Download duration in seconds
    pub duration_secs: f64,
    /// Whether download succeeded
    pub success: bool,
    /// Timestamp
    pub timestamp: Option<String>,
}

/// Complete result of Intune log analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntuneAnalysisResult {
    /// All detected events
    pub events: Vec<IntuneEvent>,
    /// Download statistics
    pub downloads: Vec<DownloadStat>,
    /// Summary counts
    pub summary: IntuneSummary,
    /// Source file analyzed
    pub source_file: String,
}

/// Summary statistics from Intune analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntuneSummary {
    pub total_events: u32,
    pub win32_apps: u32,
    pub winget_apps: u32,
    pub scripts: u32,
    pub remediations: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub in_progress: u32,
    pub total_downloads: u32,
    pub log_time_span: Option<String>,
}
