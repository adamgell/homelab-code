use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

/// Severity level for generated diagnostic guidance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntuneDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

/// Deterministic diagnostic guidance derived from Intune analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntuneDiagnosticInsight {
    pub id: String,
    pub severity: IntuneDiagnosticSeverity,
    pub title: String,
    pub summary: String,
    pub evidence: Vec<String>,
    pub next_checks: Vec<String>,
    #[serde(default)]
    pub suggested_fixes: Vec<String>,
}

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

/// Inclusive timestamp bounds captured from analyzed content.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IntuneTimestampBounds {
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
}

/// Coverage metrics for a single analyzed source file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IntuneDiagnosticsFileCoverage {
    pub file_path: String,
    pub event_count: u32,
    pub download_count: u32,
    pub timestamp_bounds: Option<IntuneTimestampBounds>,
    #[serde(default)]
    pub is_rotated_segment: bool,
    pub rotation_group: Option<String>,
}

/// Summary of which files and time ranges contributed to diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IntuneDiagnosticsCoverage {
    #[serde(default)]
    pub files: Vec<IntuneDiagnosticsFileCoverage>,
    pub timestamp_bounds: Option<IntuneTimestampBounds>,
    #[serde(default)]
    pub has_rotated_logs: bool,
    pub dominant_source: Option<IntuneDominantSource>,
}

/// Primary source contributing to the current diagnostics result.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IntuneDominantSource {
    pub file_path: String,
    pub event_count: u32,
    pub event_share: Option<f64>,
}

/// Confidence bucket for the generated diagnostics result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntuneDiagnosticsConfidenceLevel {
    Unknown,
    Low,
    Medium,
    High,
}

impl Default for IntuneDiagnosticsConfidenceLevel {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Confidence metadata for the generated diagnostics result.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IntuneDiagnosticsConfidence {
    pub level: IntuneDiagnosticsConfidenceLevel,
    pub score: Option<f64>,
    #[serde(default)]
    pub reasons: Vec<String>,
}

/// Grouped repeated failures for follow-on diagnostics phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntuneRepeatedFailureGroup {
    pub id: String,
    pub name: String,
    pub event_type: IntuneEventType,
    pub error_code: Option<String>,
    pub occurrences: u32,
    pub timestamp_bounds: Option<IntuneTimestampBounds>,
    #[serde(default)]
    pub source_files: Vec<String>,
    #[serde(default)]
    pub sample_event_ids: Vec<u64>,
}

/// Complete result of Intune log analysis.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntuneAnalysisResult {
    /// All detected events
    pub events: Vec<IntuneEvent>,
    /// Download statistics
    pub downloads: Vec<DownloadStat>,
    /// Summary counts
    pub summary: IntuneSummary,
    /// Deterministic diagnostic guidance.
    #[serde(default)]
    pub diagnostics: Vec<IntuneDiagnosticInsight>,
    /// Source path analyzed (file path or directory path)
    pub source_file: String,
    /// Expanded source files included in this result.
    #[serde(default)]
    pub source_files: Vec<String>,
    /// Coverage metadata behind the diagnostics contract.
    #[serde(default)]
    pub diagnostics_coverage: IntuneDiagnosticsCoverage,
    /// Confidence metadata behind the diagnostics contract.
    #[serde(default)]
    pub diagnostics_confidence: IntuneDiagnosticsConfidence,
    /// Grouped repeated failures behind the diagnostics contract.
    #[serde(default)]
    pub repeated_failures: Vec<IntuneRepeatedFailureGroup>,
}

impl Serialize for IntuneAnalysisResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("IntuneAnalysisResult", 9)?;
        state.serialize_field("events", &self.events)?;
        state.serialize_field("downloads", &self.downloads)?;
        state.serialize_field("summary", &self.summary)?;
        state.serialize_field("diagnostics", &self.diagnostics)?;
        state.serialize_field("sourceFile", &self.source_file)?;
        state.serialize_field("sourceFiles", &self.source_files)?;
        state.serialize_field("diagnosticsCoverage", &self.diagnostics_coverage)?;
        state.serialize_field("diagnosticsConfidence", &self.diagnostics_confidence)?;
        state.serialize_field("repeatedFailures", &self.repeated_failures)?;
        state.end()
    }
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
    pub pending: u32,
    pub timed_out: u32,
    pub total_downloads: u32,
    pub successful_downloads: u32,
    pub failed_downloads: u32,
    pub failed_scripts: u32,
    pub log_time_span: Option<String>,
}
