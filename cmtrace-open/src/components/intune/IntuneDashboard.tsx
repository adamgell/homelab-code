import { useEffect, useMemo } from "react";
import { useIntuneStore } from "../../stores/intune-store";
import { useAppActions } from "../layout/Toolbar";
import { EventTimeline } from "./EventTimeline";
import { DownloadStats } from "./DownloadStats";
import type {
  IntuneDiagnosticInsight,
  IntuneDiagnosticSeverity,
  IntuneEventType,
  IntuneStatus,
} from "../../types/intune";

type TabId = "timeline" | "downloads" | "summary";

const TAB_LABELS: Record<TabId, string> = {
  timeline: "Timeline",
  downloads: "Downloads",
  summary: "Summary",
};

export function IntuneDashboard() {
  const events = useIntuneStore((s) => s.events);
  const downloads = useIntuneStore((s) => s.downloads);
  const summary = useIntuneStore((s) => s.summary);
  const diagnostics = useIntuneStore((s) => s.diagnostics);
  const sourceContext = useIntuneStore((s) => s.sourceContext);
  const analysisState = useIntuneStore((s) => s.analysisState);
  const isAnalyzing = useIntuneStore((s) => s.isAnalyzing);
  const timelineScope = useIntuneStore((s) => s.timelineScope);
  const activeTab = useIntuneStore((s) => s.activeTab);
  const setActiveTab = useIntuneStore((s) => s.setActiveTab);
  const clearTimelineFileScope = useIntuneStore((s) => s.clearTimelineFileScope);
  const filterEventType = useIntuneStore((s) => s.filterEventType);
  const filterStatus = useIntuneStore((s) => s.filterStatus);
  const setFilterEventType = useIntuneStore((s) => s.setFilterEventType);
  const setFilterStatus = useIntuneStore((s) => s.setFilterStatus);
  const { commandState, openSourceFileDialog, openSourceFolderDialog } = useAppActions();

  const availableTabs = useMemo(
    () => ({
      timeline: events.length > 0,
      downloads: downloads.length > 0,
      summary: summary != null,
    }),
    [downloads.length, events.length, summary]
  );

  const filteredEventCount = useMemo(() => {
    return events.filter((event) => {
      if (filterEventType !== "All" && event.eventType !== filterEventType) {
        return false;
      }
      if (filterStatus !== "All" && event.status !== filterStatus) {
        return false;
      }
      return true;
    }).length;
  }, [events, filterEventType, filterStatus]);

  const hasActiveFilters = filterEventType !== "All" || filterStatus !== "All";

  useEffect(() => {
    if (!availableTabs[activeTab]) {
      if (availableTabs.timeline) {
        setActiveTab("timeline");
        return;
      }
      if (availableTabs.downloads) {
        setActiveTab("downloads");
        return;
      }
      if (availableTabs.summary) {
        setActiveTab("summary");
        return;
      }
      setActiveTab("timeline");
    }
  }, [activeTab, availableTabs, setActiveTab]);

  const hasAnyResult = summary != null || events.length > 0 || downloads.length > 0;
  const sourceFiles = sourceContext.includedFiles;
  const sourceLabel = analysisState.requestedPath ?? sourceContext.analyzedPath;
  const sourceStatusTone =
    analysisState.phase === "error"
      ? "#b91c1c"
      : analysisState.phase === "empty"
        ? "#b45309"
        : analysisState.phase === "analyzing"
          ? "#2563eb"
          : "#6b7280";
  const timelineScopeFileName = timelineScope.filePath ? getFileName(timelineScope.filePath) : null;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "#ffffff",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "6px 12px",
          backgroundColor: "#f3f4f6",
          borderBottom: "1px solid #d1d5db",
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <span
            style={{
              fontSize: "13px",
              fontWeight: 600,
              color: "#1f2937",
              fontFamily: "'Segoe UI', Tahoma, sans-serif",
            }}
          >
            Intune Diagnostics Workspace
          </span>
          <div style={{ width: "1px", height: "16px", backgroundColor: "#cbd5e1" }} />
          <ActionButton
            onClick={() => {
              void openSourceFileDialog();
            }}
            disabled={!commandState.canOpenSources}
            label={isAnalyzing ? "Analyzing..." : "Open IME Log File..."}
          />
          <ActionButton
            onClick={() => {
              void openSourceFolderDialog();
            }}
            disabled={!commandState.canOpenSources}
            label={isAnalyzing ? "Analyzing..." : "Open IME Log Folder..."}
          />

          {(analysisState.phase === "analyzing" || analysisState.phase === "error" || analysisState.phase === "empty") && (
            <span style={{ fontSize: "12px", color: sourceStatusTone, fontWeight: 500, marginLeft: "4px" }}>
              {analysisState.message}
            </span>
          )}
        </div>

        {sourceLabel && (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "flex-end",
              minWidth: 0,
              maxWidth: "400px",
            }}
          >
            <span
              style={{
                fontSize: "11px",
                color: "#4b5563",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                maxWidth: "100%",
                fontWeight: 500,
              }}
              title={sourceLabel}
            >
              {sourceLabel}
            </span>
            {(analysisState.detail || sourceFiles.length > 0) && (
              <span style={{ fontSize: "10px", color: sourceStatusTone }}>
                {analysisState.phase === "error"
                  ? analysisState.detail
                  : analysisState.phase === "empty"
                    ? analysisState.detail
                    : sourceFiles.length > 0
                      ? `${sourceFiles.length} included files`
                      : analysisState.detail}
              </span>
            )}
          </div>
        )}
      </div>

      <div
        style={{
          display: "flex",
          alignItems: "center",
          padding: "0 12px",
          backgroundColor: "#f8fafc",
          borderBottom: "1px solid #e2e8f0",
          minHeight: "40px",
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", gap: "2px", alignItems: "center", height: "100%" }}>
          {(Object.keys(TAB_LABELS) as TabId[]).map((tabId) => (
            <CanvasTabButton
              key={tabId}
              label={TAB_LABELS[tabId]}
              active={activeTab === tabId}
              disabled={isAnalyzing || !availableTabs[tabId]}
              count={tabId === "timeline" ? events.length : tabId === "downloads" ? downloads.length : summary ? 1 : 0}
              onClick={() => setActiveTab(tabId)}
            />
          ))}
        </div>

        {summary && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              marginLeft: "12px",
              flex: 1,
              overflow: "hidden",
            }}
          >
            <div style={{ width: "1px", height: "20px", backgroundColor: "#cbd5e1", marginRight: "12px" }} />
            <div
              style={{
                display: "flex",
                gap: "10px",
                flexWrap: "nowrap",
                overflowX: "auto",
                scrollbarWidth: "none",
                alignItems: "center",
              }}
            >
              <StrongBadge label="Total" value={summary.totalEvents} />
              <StrongBadge label="Success" value={summary.succeeded} color="#16a34a" />
              <StrongBadge label="Fail" value={summary.failed} color="#dc2626" />
              <StrongBadge label="Prog" value={summary.inProgress} color="#2563eb" />
              <StrongBadge label="Win32" value={summary.win32Apps} />
              <StrongBadge label="WinGet" value={summary.wingetApps} />
              {summary.logTimeSpan && (
                <>
                  <div style={{ width: "1px", height: "12px", backgroundColor: "#cbd5e1", margin: "0 4px" }} />
                  <span style={{ fontSize: "11px", color: "#64748b", fontWeight: 500 }}>
                    {summary.logTimeSpan}
                  </span>
                </>
              )}
            </div>
          </div>
        )}

        {activeTab === "timeline" && events.length > 0 && (
          <div style={{ display: "flex", alignItems: "center", gap: "6px", marginLeft: "auto", paddingLeft: "12px" }}>
            <span style={{ fontSize: "10px", color: "#6b7280", fontWeight: 600, textTransform: "uppercase" }}>Filters:</span>
            <select
              value={filterEventType}
              onChange={(e) => setFilterEventType(e.target.value as IntuneEventType | "All")}
              style={selectStyle}
              disabled={isAnalyzing}
            >
              <option value="All">All Types</option>
              <option value="Win32App">Win32</option>
              <option value="WinGetApp">WinGet</option>
              <option value="PowerShellScript">Script</option>
              <option value="Remediation">Remediation</option>
              <option value="Esp">ESP</option>
              <option value="SyncSession">Sync</option>
              <option value="PolicyEvaluation">Policy</option>
              <option value="ContentDownload">Download</option>
              <option value="Other">Other</option>
            </select>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as IntuneStatus | "All")}
              style={selectStyle}
              disabled={isAnalyzing}
            >
              <option value="All">All Statuses</option>
              <option value="Success">Success</option>
              <option value="Failed">Failed</option>
              <option value="InProgress">In Progress</option>
              <option value="Pending">Pending</option>
              <option value="Timeout">Timeout</option>
              <option value="Unknown">Unknown</option>
            </select>
            <button
              onClick={() => {
                setFilterEventType("All");
                setFilterStatus("All");
              }}
              disabled={!hasActiveFilters || isAnalyzing}
              style={{
                marginLeft: "2px",
                fontSize: "10px",
                padding: "2px 6px",
                border: "1px solid #d1d5db",
                borderRadius: "3px",
                backgroundColor: hasActiveFilters ? "#fff" : "#f1f5f9",
                color: hasActiveFilters ? "#1e293b" : "#94a3b8",
                cursor: hasActiveFilters && !isAnalyzing ? "pointer" : "not-allowed",
              }}
            >
              Reset
            </button>
            <span style={{ fontSize: "11px", color: "#64748b", fontWeight: 500, marginLeft: "4px" }}>
              {filteredEventCount}/{events.length}
            </span>
            {timelineScope.filePath && (
              <>
                <div style={{ width: "1px", height: "16px", backgroundColor: "#cbd5e1", margin: "0 2px" }} />
                <span
                  title={timelineScope.filePath}
                  style={{
                    maxWidth: "220px",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    fontSize: "11px",
                    color: "#92400e",
                    backgroundColor: "#fef3c7",
                    border: "1px solid #fcd34d",
                    borderRadius: "999px",
                    padding: "3px 8px",
                    fontWeight: 600,
                  }}
                >
                  Timeline scoped to {timelineScopeFileName}
                </span>
                <button
                  onClick={() => clearTimelineFileScope()}
                  disabled={isAnalyzing}
                  style={{
                    fontSize: "10px",
                    padding: "2px 6px",
                    border: "1px solid #d1d5db",
                    borderRadius: "3px",
                    backgroundColor: "#fff",
                    color: "#1e293b",
                    cursor: isAnalyzing ? "not-allowed" : "pointer",
                  }}
                >
                  Clear Scope
                </button>
              </>
            )}
          </div>
        )}
      </div>

      <div style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column" }}>
        {(analysisState.phase === "error" || analysisState.phase === "empty") && (
          <div
            role="alert"
            style={{
              margin: "12px 12px 0",
              padding: "10px 12px",
              border: analysisState.phase === "empty" ? "1px solid #fde68a" : "1px solid #fecaca",
              backgroundColor: analysisState.phase === "empty" ? "#fffbeb" : "#fef2f2",
              color: analysisState.phase === "empty" ? "#92400e" : "#991b1b",
              fontSize: "12px",
            }}
          >
            <div style={{ fontWeight: 600 }}>{analysisState.message}</div>
            {analysisState.detail && <div style={{ marginTop: "4px" }}>{analysisState.detail}</div>}
          </div>
        )}

        {!hasAnyResult && analysisState.phase !== "analyzing" && analysisState.phase !== "error" ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "#999",
              fontSize: "14px",
            }}
          >
            Open an Intune IME log file or folder to analyze
          </div>
        ) : isAnalyzing && !hasAnyResult ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "#6b7280",
              fontSize: "14px",
            }}
          >
            {analysisState.message}
          </div>
        ) : analysisState.phase === "empty" && !hasAnyResult ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "#92400e",
              fontSize: "14px",
              padding: "0 24px",
              textAlign: "center",
            }}
          >
            {analysisState.detail ?? "No IME log files were found in this folder."}
          </div>
        ) : analysisState.phase === "error" && !hasAnyResult ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "#991b1b",
              fontSize: "14px",
              padding: "0 24px",
              textAlign: "center",
            }}
          >
            {analysisState.detail ?? "The selected Intune source could not be analyzed."}
          </div>
        ) : (
          <>
            {activeTab === "timeline" && <EventTimeline events={events} />}
            {activeTab === "downloads" && <DownloadStats downloads={downloads} />}
            {activeTab === "summary" && summary && (
              <SummaryView
                summary={summary}
                diagnostics={diagnostics}
                sourceFile={sourceContext.analyzedPath}
                sourceFiles={sourceContext.includedFiles}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}

function ActionButton({
  onClick,
  disabled,
  label,
}: {
  onClick: () => void;
  disabled: boolean;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        fontSize: "12px",
        padding: "4px 10px",
        border: "1px solid #94a3b8",
        borderRadius: "4px",
        backgroundColor: disabled ? "#e5e7eb" : "#ffffff",
        color: disabled ? "#6b7280" : "#1f2937",
        cursor: disabled ? "not-allowed" : "pointer",
      }}
    >
      {label}
    </button>
  );
}

const selectStyle: React.CSSProperties = {
  fontSize: "11px",
  padding: "2px 6px",
  borderRadius: "3px",
  border: "1px solid #cbd5e1",
  backgroundColor: "#fff",
  outline: "none",
};

function CanvasTabButton({
  label,
  active,
  disabled,
  count,
  onClick,
}: {
  label: string;
  active: boolean;
  disabled: boolean;
  count: number;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        fontSize: "11px",
        padding: "6px 12px",
        border: "none",
        borderBottom: active ? "2px solid #2563eb" : "2px solid transparent",
        backgroundColor: "transparent",
        color: disabled ? "#94a3b8" : active ? "#1e3a8a" : "#475569",
        fontWeight: active ? 600 : 500,
        cursor: disabled ? "not-allowed" : "pointer",
        height: "100%",
        display: "flex",
        alignItems: "center",
        gap: "6px",
        transition: "all 0.1s ease",
      }}
    >
      <span>{label}</span>
      <span style={{
        fontSize: "9px",
        backgroundColor: active ? "#dbeafe" : "#f1f5f9",
        color: active ? "#1d4ed8" : "#64748b",
        padding: "2px 6px",
        borderRadius: "99px",
        fontWeight: 700,
      }}>
        {count}
      </span>
    </button>
  );
}

function StrongBadge({ label, value, color }: { label: string; value: number | string; color?: string }) {
  return (
    <div style={{ display: "flex", alignItems: "baseline", gap: "4px" }}>
      <span style={{ color: "#64748b", fontSize: "10px", fontWeight: 600, textTransform: "uppercase" }}>{label}</span>
      <span style={{ color: color || "#0f172a", fontSize: "12px", fontWeight: 700 }}>{value}</span>
    </div>
  );
}

function SummaryView({
  summary,
  diagnostics,
  sourceFile,
  sourceFiles,
}: {
  summary: {
    totalEvents: number;
    win32Apps: number;
    wingetApps: number;
    scripts: number;
    remediations: number;
    succeeded: number;
    failed: number;
    inProgress: number;
    pending: number;
    timedOut: number;
    totalDownloads: number;
    successfulDownloads: number;
    failedDownloads: number;
    failedScripts: number;
    logTimeSpan: string | null;
  };
  diagnostics: IntuneDiagnosticInsight[];
  sourceFile: string | null;
  sourceFiles: string[];
}) {
  return (
    <div style={{ padding: "16px", fontSize: "13px" }}>
      <h3
        style={{
          margin: "0 0 12px 0",
          fontSize: "15px",
          fontFamily: "'Segoe UI', Tahoma, sans-serif",
        }}
      >
        Intune Diagnostics Summary
      </h3>

      {sourceFile && (
        <div style={{ marginBottom: "12px", color: "#666" }}>
          <strong>Analyzed Path:</strong> {sourceFile}
        </div>
      )}

      {sourceFiles.length > 0 && (
        <div style={{ marginBottom: "12px", color: "#666" }}>
          <div style={{ marginBottom: "4px" }}>
            <strong>Included IME Log Files:</strong> {sourceFiles.length}
          </div>
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: "6px",
            }}
          >
            {sourceFiles.map((file) => (
              <span
                key={file}
                title={file}
                style={{
                  padding: "2px 8px",
                  borderRadius: "999px",
                  backgroundColor: "#eff6ff",
                  border: "1px solid #bfdbfe",
                  color: "#1e3a8a",
                  fontSize: "11px",
                  fontFamily: "'Courier New', monospace",
                }}
              >
                {getFileName(file)}
              </span>
            ))}
          </div>
        </div>
      )}

      {summary.logTimeSpan && (
        <div style={{ marginBottom: "12px", color: "#666" }}>
          <strong>Log Time Span:</strong> {summary.logTimeSpan}
        </div>
      )}

      {diagnostics.length > 0 && (
        <div style={{ marginBottom: "20px" }}>
          <h4
            style={{
              margin: "0 0 10px 0",
              fontSize: "13px",
              color: "#111827",
            }}
          >
            Diagnostics Guidance
          </h4>
          <div
            style={{
              display: "grid",
              gap: "12px",
            }}
          >
            {diagnostics.map((diagnostic) => (
              <DiagnosticCard key={diagnostic.id} diagnostic={diagnostic} />
            ))}
          </div>
        </div>
      )}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))",
          gap: "12px",
          marginTop: "16px",
        }}
      >
        <SummaryCard title="Total Events" value={summary.totalEvents} />
        <SummaryCard title="Win32 Apps" value={summary.win32Apps} color="#6366f1" />
        <SummaryCard title="WinGet Apps" value={summary.wingetApps} color="#8b5cf6" />
        <SummaryCard title="Scripts" value={summary.scripts} color="#0ea5e9" />
        <SummaryCard title="Remediations" value={summary.remediations} color="#14b8a6" />
        <SummaryCard title="Downloads" value={summary.totalDownloads} color="#f97316" />
        <SummaryCard
          title="Download Successes"
          value={summary.successfulDownloads}
          color="#fb923c"
        />
        <SummaryCard
          title="Download Failures"
          value={summary.failedDownloads}
          color="#f97316"
        />
        <SummaryCard title="Succeeded" value={summary.succeeded} color="#22c55e" />
        <SummaryCard title="Failed" value={summary.failed} color="#ef4444" />
        <SummaryCard title="In Progress" value={summary.inProgress} color="#3b82f6" />
        <SummaryCard title="Pending" value={summary.pending} color="#64748b" />
        <SummaryCard title="Timed Out" value={summary.timedOut} color="#f59e0b" />
        <SummaryCard title="Script Failures" value={summary.failedScripts} color="#dc2626" />
      </div>
    </div>
  );
}

function DiagnosticCard({
  diagnostic,
}: {
  diagnostic: IntuneDiagnosticInsight;
}) {
  const accent = getDiagnosticAccent(diagnostic.severity);

  return (
    <div
      style={{
        border: `1px solid ${accent.border}`,
        borderLeft: `4px solid ${accent.accent}`,
        borderRadius: "6px",
        backgroundColor: accent.background,
        padding: "12px 14px",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "12px",
          marginBottom: "6px",
        }}
      >
        <div style={{ fontSize: "13px", fontWeight: 600, color: "#111827" }}>
          {diagnostic.title}
        </div>
        <span
          style={{
            fontSize: "10px",
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            color: accent.accent,
            fontWeight: 700,
          }}
        >
          {diagnostic.severity}
        </span>
      </div>

      <div style={{ fontSize: "12px", color: "#374151", marginBottom: "10px" }}>
        {diagnostic.summary}
      </div>

      <div style={{ display: "grid", gap: "8px" }}>
        <div>
          <div
            style={{
              fontSize: "11px",
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "#6b7280",
              marginBottom: "4px",
            }}
          >
            Evidence
          </div>
          <ul style={{ margin: 0, paddingLeft: "18px", color: "#1f2937" }}>
            {diagnostic.evidence.map((item) => (
              <li key={item} style={{ marginBottom: "2px" }}>
                {item}
              </li>
            ))}
          </ul>
        </div>

        <div>
          <div
            style={{
              fontSize: "11px",
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "#6b7280",
              marginBottom: "4px",
            }}
          >
            Next Checks
          </div>
          <ul style={{ margin: 0, paddingLeft: "18px", color: "#1f2937" }}>
            {diagnostic.nextChecks.map((item) => (
              <li key={item} style={{ marginBottom: "2px" }}>
                {item}
              </li>
            ))}
          </ul>
        </div>

        {diagnostic.suggestedFixes.length > 0 && (
          <div>
            <div
              style={{
                fontSize: "11px",
                textTransform: "uppercase",
                letterSpacing: "0.05em",
                color: "#6b7280",
                marginBottom: "4px",
              }}
            >
              Suggested Fixes
            </div>
            <ul style={{ margin: 0, paddingLeft: "18px", color: "#1f2937" }}>
              {diagnostic.suggestedFixes.map((item) => (
                <li key={item} style={{ marginBottom: "2px" }}>
                  {item}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}

function getDiagnosticAccent(severity: IntuneDiagnosticSeverity) {
  switch (severity) {
    case "Error":
      return {
        accent: "#b91c1c",
        border: "#fecaca",
        background: "#fef2f2",
      };
    case "Warning":
      return {
        accent: "#b45309",
        border: "#fde68a",
        background: "#fffbeb",
      };
    case "Info":
    default:
      return {
        accent: "#1d4ed8",
        border: "#bfdbfe",
        background: "#eff6ff",
      };
  }
}

function getFileName(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const segments = normalized.split("/");
  return segments[segments.length - 1] || path;
}

function SummaryCard({
  title,
  value,
  color,
}: {
  title: string;
  value: number;
  color?: string;
}) {
  return (
    <div
      style={{
        padding: "12px",
        border: "1px solid #e5e7eb",
        borderRadius: "6px",
        borderLeft: `4px solid ${color || "#9ca3af"}`,
        backgroundColor: "#fafafa",
      }}
    >
      <div
        style={{
          fontSize: "11px",
          color: "#6b7280",
          textTransform: "uppercase",
          letterSpacing: "0.05em",
        }}
      >
        {title}
      </div>
      <div
        style={{
          fontSize: "24px",
          fontWeight: "bold",
          color: color || "#111",
          marginTop: "4px",
        }}
      >
        {value}
      </div>
    </div>
  );
}
