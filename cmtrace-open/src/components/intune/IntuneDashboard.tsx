import { useState, useCallback, useEffect, useMemo, startTransition } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useIntuneStore } from "../../stores/intune-store";
import { analyzeIntuneLogs } from "../../lib/commands";
import { getLogSourcePath, loadLogSource } from "../../lib/log-source";
import { EventTimeline } from "./EventTimeline";
import { DownloadStats } from "./DownloadStats";
import type {
  IntuneDiagnosticInsight,
  IntuneDiagnosticSeverity,
  IntuneEventType,
  IntuneStatus,
} from "../../types/intune";
import type { LogSource } from "../../types/log";

type TabId = "timeline" | "downloads" | "summary";

const TAB_LABELS: Record<TabId, string> = {
  timeline: "Timeline",
  downloads: "Downloads",
  summary: "Summary",
};

export function IntuneDashboard() {
  const [activeTab, setActiveTab] = useState<TabId>("timeline");
  const events = useIntuneStore((s) => s.events);
  const downloads = useIntuneStore((s) => s.downloads);
  const summary = useIntuneStore((s) => s.summary);
  const diagnostics = useIntuneStore((s) => s.diagnostics);
  const sourceFile = useIntuneStore((s) => s.sourceFile);
  const sourceFiles = useIntuneStore((s) => s.sourceFiles);
  const isAnalyzing = useIntuneStore((s) => s.isAnalyzing);
  const resultRevision = useIntuneStore((s) => s.resultRevision);
  const setResults = useIntuneStore((s) => s.setResults);
  const setAnalyzing = useIntuneStore((s) => s.setAnalyzing);
  const filterEventType = useIntuneStore((s) => s.filterEventType);
  const filterStatus = useIntuneStore((s) => s.filterStatus);
  const setFilterEventType = useIntuneStore((s) => s.setFilterEventType);
  const setFilterStatus = useIntuneStore((s) => s.setFilterStatus);

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
  }, [activeTab, availableTabs]);

  useEffect(() => {
    if (resultRevision > 0) {
      setActiveTab("timeline");
    }
  }, [resultRevision]);

  const analyzeSource = useCallback(
    async (source: LogSource) => {
      setAnalyzing(true);
      try {
        await loadLogSource(source).catch((error) => {
          console.warn("[intune] failed to sync log source", { source, error });
        });

        const result = await analyzeIntuneLogs(getLogSourcePath(source));
        startTransition(() => {
          setResults(
            result.events,
            result.downloads,
            result.summary,
            result.diagnostics,
            result.sourceFile,
            result.sourceFiles
          );
        });
      } catch (err) {
        console.error("Intune analysis failed:", err);
      } finally {
        setAnalyzing(false);
      }
    },
    [setAnalyzing, setResults]
  );

  const handleAnalyzeFile = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "IME Logs", extensions: ["log"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    await analyzeSource({ kind: "file", path: selected });
  }, [analyzeSource]);

  const handleAnalyzeFolder = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: true,
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    await analyzeSource({ kind: "folder", path: selected });
  }, [analyzeSource]);

  const hasAnyResult = summary != null || events.length > 0 || downloads.length > 0;

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
          gap: "8px",
          padding: "8px 12px",
          backgroundColor: "#f0f0f0",
          borderBottom: "1px solid #c0c0c0",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: "14px",
            fontWeight: "bold",
            fontFamily: "'Segoe UI', Tahoma, sans-serif",
          }}
        >
          Intune Diagnostics
        </span>
        <ActionButton
          onClick={handleAnalyzeFile}
          disabled={isAnalyzing}
          label={isAnalyzing ? "Analyzing..." : "Open IME Log"}
        />
        <ActionButton
          onClick={handleAnalyzeFolder}
          disabled={isAnalyzing}
          label={isAnalyzing ? "Analyzing..." : "Open IME Folder"}
        />

        {isAnalyzing && (
          <span style={{ fontSize: "11px", color: "#1d4ed8" }}>Analyzing source…</span>
        )}

        {sourceFile && (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "flex-end",
              marginLeft: "auto",
              minWidth: 0,
              maxWidth: "520px",
            }}
          >
            <span
              style={{
                fontSize: "11px",
                color: "#374151",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                maxWidth: "100%",
              }}
              title={sourceFile}
            >
              {sourceFile}
            </span>
            {sourceFiles.length > 0 && (
              <span
                style={{
                  fontSize: "10px",
                  color: "#6b7280",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  maxWidth: "100%",
                }}
                title={sourceFiles.join("\n")}
              >
                Included files: {sourceFiles.length}
              </span>
            )}
          </div>
        )}
      </div>

      {summary && (
        <div
          style={{
            display: "flex",
            gap: "16px",
            padding: "6px 12px",
            backgroundColor: "#f8fafc",
            borderBottom: "1px solid #e2e8f0",
            fontSize: "12px",
            flexWrap: "wrap",
            flexShrink: 0,
          }}
        >
          <SummaryBadge label="Events" value={summary.totalEvents} />
          <SummaryBadge label="Win32" value={summary.win32Apps} color="#6366f1" />
          <SummaryBadge label="WinGet" value={summary.wingetApps} color="#8b5cf6" />
          <SummaryBadge label="Scripts" value={summary.scripts} color="#0ea5e9" />
          <SummaryBadge label="Remed." value={summary.remediations} color="#14b8a6" />
          <div style={{ width: "1px", backgroundColor: "#d1d5db" }} />
          <SummaryBadge label="Succeeded" value={summary.succeeded} color="#22c55e" />
          <SummaryBadge label="Failed" value={summary.failed} color="#ef4444" />
          <SummaryBadge label="In Progress" value={summary.inProgress} color="#3b82f6" />
          <SummaryBadge label="Pending" value={summary.pending} color="#64748b" />
          <SummaryBadge label="Timed Out" value={summary.timedOut} color="#f59e0b" />
          {summary.logTimeSpan && (
            <>
              <div style={{ width: "1px", backgroundColor: "#d1d5db" }} />
              <span style={{ color: "#6b7280" }}>Span: {summary.logTimeSpan}</span>
            </>
          )}
        </div>
      )}

      <div
        style={{
          display: "flex",
          alignItems: "center",
          padding: "4px 12px",
          gap: "4px",
          borderBottom: "1px solid #e2e8f0",
          backgroundColor: "#fafafa",
          flexShrink: 0,
        }}
      >
        {(Object.keys(TAB_LABELS) as TabId[]).map((tabId) => (
          <TabButton
            key={tabId}
            label={TAB_LABELS[tabId]}
            active={activeTab === tabId}
            disabled={isAnalyzing || !availableTabs[tabId]}
            count={tabId === "timeline" ? events.length : tabId === "downloads" ? downloads.length : summary ? 1 : 0}
            onClick={() => setActiveTab(tabId)}
          />
        ))}

        {activeTab === "timeline" && events.length > 0 && (
          <>
            <div
              style={{
                width: "1px",
                height: "20px",
                backgroundColor: "#d1d5db",
                margin: "0 6px",
              }}
            />
            <label style={{ fontSize: "11px", color: "#666" }}>Type:</label>
            <select
              value={filterEventType}
              onChange={(e) =>
                setFilterEventType(e.target.value as IntuneEventType | "All")
              }
              style={{ fontSize: "11px", padding: "1px 4px" }}
              disabled={isAnalyzing}
            >
              <option value="All">All</option>
              <option value="Win32App">Win32 App</option>
              <option value="WinGetApp">WinGet App</option>
              <option value="PowerShellScript">PowerShell Script</option>
              <option value="Remediation">Remediation</option>
              <option value="Esp">ESP</option>
              <option value="SyncSession">Sync Session</option>
              <option value="PolicyEvaluation">Policy Evaluation</option>
              <option value="ContentDownload">Content Download</option>
              <option value="Other">Other</option>
            </select>

            <label style={{ fontSize: "11px", color: "#666" }}>Status:</label>
            <select
              value={filterStatus}
              onChange={(e) =>
                setFilterStatus(e.target.value as IntuneStatus | "All")
              }
              style={{ fontSize: "11px", padding: "1px 4px" }}
              disabled={isAnalyzing}
            >
              <option value="All">All</option>
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
                marginLeft: "4px",
                fontSize: "11px",
                padding: "2px 6px",
                border: "1px solid #d1d5db",
                borderRadius: "4px",
                backgroundColor: hasActiveFilters ? "#ffffff" : "#f3f4f6",
                cursor: hasActiveFilters && !isAnalyzing ? "pointer" : "not-allowed",
              }}
            >
              Reset Filters
            </button>

            <span style={{ marginLeft: "6px", fontSize: "11px", color: "#6b7280" }}>
              {filteredEventCount} / {events.length} events
            </span>
          </>
        )}
      </div>

      <div style={{ flex: 1, overflow: "auto" }}>
        {!hasAnyResult && !isAnalyzing ? (
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
            Open an IntuneManagementExtension.log file or folder to analyze
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
            Running Intune analysis...
          </div>
        ) : (
          <>
            {activeTab === "timeline" && <EventTimeline events={events} />}
            {activeTab === "downloads" && <DownloadStats downloads={downloads} />}
            {activeTab === "summary" && summary && (
              <SummaryView
                summary={summary}
                diagnostics={diagnostics}
                sourceFile={sourceFile}
                sourceFiles={sourceFiles}
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

function SummaryBadge({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color?: string;
}) {
  return (
    <span>
      <span style={{ color: "#6b7280" }}>{label}: </span>
      <span style={{ fontWeight: "bold", color: color || "#111" }}>{value}</span>
    </span>
  );
}

function TabButton({
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
        fontSize: "12px",
        padding: "3px 10px",
        border: "1px solid #d1d5db",
        borderBottom: active ? "2px solid #3b82f6" : "1px solid #d1d5db",
        borderRadius: "3px 3px 0 0",
        backgroundColor: active ? "#fff" : "#f3f4f6",
        color: disabled ? "#9ca3af" : "#111827",
        fontWeight: active ? 600 : 400,
        cursor: disabled ? "not-allowed" : "pointer",
      }}
    >
      {label} ({count})
    </button>
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
        Analysis Summary
      </h3>

      {sourceFile && (
        <div style={{ marginBottom: "12px", color: "#666" }}>
          <strong>Analyzed Path:</strong> {sourceFile}
        </div>
      )}

      {sourceFiles.length > 0 && (
        <div style={{ marginBottom: "12px", color: "#666" }}>
          <div style={{ marginBottom: "4px" }}>
            <strong>Included Files:</strong> {sourceFiles.length}
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
