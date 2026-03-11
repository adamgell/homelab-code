import { useEffect, useMemo, useRef, useState } from "react";
import { useIntuneStore } from "../../stores/intune-store";
import { useAppActions } from "../layout/Toolbar";
import { EventTimeline } from "./EventTimeline";
import { DownloadStats } from "./DownloadStats";
import type {
  DownloadStat,
  IntuneDiagnosticInsight,
  IntuneDiagnosticSeverity,
  IntuneDiagnosticsConfidence,
  IntuneDiagnosticsCoverage,
  IntuneDiagnosticsFileCoverage,
  IntuneEvent,
  IntuneEventType,
  IntuneRepeatedFailureGroup,
  IntuneStatus,
  IntuneSummary,
  IntuneTimestampBounds,
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

      <div style={{ flex: 1, minHeight: 0, overflow: "auto", display: "flex", flexDirection: "column" }}>
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
                events={events}
                downloads={downloads}
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
  events,
  downloads,
  sourceFile,
  sourceFiles,
}: {
  summary: IntuneSummary;
  diagnostics: IntuneDiagnosticInsight[];
  events: IntuneEvent[];
  downloads: DownloadStat[];
  sourceFile: string | null;
  sourceFiles: string[];
}) {
  const setActiveTab = useIntuneStore((s) => s.setActiveTab);
  const setFilterEventType = useIntuneStore((s) => s.setFilterEventType);
  const setFilterStatus = useIntuneStore((s) => s.setFilterStatus);
  const selectEvent = useIntuneStore((s) => s.selectEvent);
  const setTimelineFileScope = useIntuneStore((s) => s.setTimelineFileScope);
  const clearTimelineFileScope = useIntuneStore((s) => s.clearTimelineFileScope);

  const [showAllConfidenceReasons, setShowAllConfidenceReasons] = useState(false);
  const [showAllRepeatedFailures, setShowAllRepeatedFailures] = useState(false);
  const [showCoverageDetails, setShowCoverageDetails] = useState(false);

  const coverageSectionRef = useRef<HTMLDivElement | null>(null);
  const confidenceSectionRef = useRef<HTMLDivElement | null>(null);
  const repeatedFailuresSectionRef = useRef<HTMLDivElement | null>(null);
  const diagnosticsGuidanceSectionRef = useRef<HTMLDivElement | null>(null);

  const repeatedFailures = useMemo(
    () => summary.repeatedFailures ?? buildDerivedRepeatedFailures(events),
    [events, summary.repeatedFailures]
  );

  const diagnosticsCoverage = useMemo(
    () => summary.diagnosticsCoverage ?? buildDerivedCoverage(sourceFiles, events, downloads),
    [downloads, events, sourceFiles, summary.diagnosticsCoverage]
  );

  const diagnosticsConfidence = useMemo(
    () =>
      summary.diagnosticsConfidence ??
      buildDerivedConfidence(summary, diagnosticsCoverage, repeatedFailures, events),
    [diagnosticsCoverage, events, repeatedFailures, summary, summary.diagnosticsConfidence]
  );

  const contributingFileCount = diagnosticsCoverage.files.filter(
    (file) => file.eventCount > 0 || file.downloadCount > 0
  ).length;
  const visibleConfidenceReasons = showAllConfidenceReasons
    ? diagnosticsConfidence.reasons
    : diagnosticsConfidence.reasons.slice(0, 2);
  const hiddenConfidenceReasonCount = Math.max(
    diagnosticsConfidence.reasons.length - visibleConfidenceReasons.length,
    0
  );
  const visibleRepeatedFailures = showAllRepeatedFailures
    ? repeatedFailures
    : repeatedFailures.slice(0, 2);
  const hiddenRepeatedFailureCount = Math.max(
    repeatedFailures.length - visibleRepeatedFailures.length,
    0
  );
  const conclusions = useMemo(
    () =>
      buildSummaryConclusions({
        summary,
        diagnostics,
        diagnosticsCoverage,
        diagnosticsConfidence,
        repeatedFailures,
      }),
    [diagnostics, diagnosticsConfidence, diagnosticsCoverage, repeatedFailures, summary]
  );

  function scrollToSection(section: SummaryConclusionSection) {
    const sectionRef =
      section === "coverage"
        ? coverageSectionRef
        : section === "confidence"
          ? confidenceSectionRef
          : section === "repeatedFailures"
            ? repeatedFailuresSectionRef
            : diagnosticsGuidanceSectionRef;

    sectionRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  function handleConclusionClick(conclusion: SummaryConclusion) {
    if (conclusion.action.kind === "section") {
      scrollToSection(conclusion.action.section);
      return;
    }

    const action = conclusion.action;
    const nextEventType = action.eventType ?? "All";
    const nextStatus = action.status ?? "All";
    const nextFilePath = action.filePath;
    const firstMatchingEventId = action.selectFirstMatch
      ? events.find((event) => matchesTimelineAction(event, action))?.id ?? null
      : null;

    setActiveTab("timeline");
    setFilterEventType(nextEventType);
    setFilterStatus(nextStatus);

    if (nextFilePath === null) {
      clearTimelineFileScope();
    } else if (nextFilePath) {
      setTimelineFileScope(nextFilePath);
    }

    if (firstMatchingEventId != null) {
      selectEvent(firstMatchingEventId);
    }
  }

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

      {conclusions.length > 0 && (
        <div
          style={{
            position: "sticky",
            top: 0,
            zIndex: 1,
            marginBottom: "12px",
            paddingBottom: "8px",
            background:
              "linear-gradient(180deg, rgba(255,255,255,0.98) 0%, rgba(255,255,255,0.98) 78%, rgba(255,255,255,0) 100%)",
          }}
        >
          <div
            style={{
              border: "1px solid #dbe3ee",
              borderRadius: "8px",
              backgroundColor: "#f8fafc",
              padding: "10px 12px",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "baseline",
                justifyContent: "space-between",
                gap: "10px",
                marginBottom: "8px",
                flexWrap: "wrap",
              }}
            >
              <div style={{ fontSize: "12px", fontWeight: 700, color: "#0f172a" }}>Conclusions</div>
              <div style={{ fontSize: "11px", color: "#64748b" }}>Click to jump to proof or focus the timeline.</div>
            </div>
            <div style={{ display: "grid", gap: "6px" }}>
              {conclusions.map((conclusion) => (
                <ConclusionButton
                  key={conclusion.id}
                  conclusion={conclusion}
                  onClick={() => handleConclusionClick(conclusion)}
                />
              ))}
            </div>
          </div>
        </div>
      )}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1.4fr) minmax(280px, 1fr)",
          gap: "12px",
          marginBottom: "16px",
        }}
      >
        <div ref={coverageSectionRef}>
          <SectionCard
            title="Diagnostics Coverage"
            subtitle="Source continuity, timestamp bounds, and dominant evidence."
          >
            <div
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: "8px",
                marginBottom: diagnosticsCoverage.files.length > 0 ? "10px" : 0,
              }}
            >
              <CompactFact label="Files" value={String(diagnosticsCoverage.files.length)} />
              <CompactFact label="Contributing" value={String(contributingFileCount)} color="#2563eb" />
              <CompactFact
                label="Rotated"
                value={diagnosticsCoverage.hasRotatedLogs ? "Yes" : "No"}
                color={diagnosticsCoverage.hasRotatedLogs ? "#b45309" : "#475569"}
              />
              {diagnosticsCoverage.dominantSource && (
                <CompactFact
                  label="Dominant"
                  value={buildDominantSourceLabel(diagnosticsCoverage.dominantSource)}
                  color="#0f766e"
                />
              )}
            </div>

            {diagnosticsCoverage.timestampBounds && (
              <div
                style={{
                  marginBottom: diagnosticsCoverage.files.length > 0 ? "10px" : 0,
                  padding: "8px 10px",
                  borderRadius: "6px",
                  backgroundColor: "#f8fafc",
                  border: "1px solid #e2e8f0",
                  color: "#334155",
                  fontSize: "12px",
                }}
              >
                <strong style={{ color: "#0f172a" }}>Timestamp Bounds:</strong>{" "}
                {formatTimestampBounds(diagnosticsCoverage.timestampBounds)}
              </div>
            )}

            {diagnosticsCoverage.files.length > 0 ? (
              <div>
                <button
                  onClick={() => setShowCoverageDetails((current) => !current)}
                  style={secondaryToggleButtonStyle}
                >
                  {showCoverageDetails
                    ? "Hide file coverage"
                    : `Show file coverage (${diagnosticsCoverage.files.length})`}
                </button>
                {showCoverageDetails && (
                  <div style={{ display: "grid", gap: "6px", marginTop: "10px" }}>
                    {diagnosticsCoverage.files.map((file) => (
                      <CoverageRow key={file.filePath} file={file} />
                    ))}
                  </div>
                )}
              </div>
            ) : (
              <EmptyStateText label="No file-level coverage evidence was available." />
            )}
          </SectionCard>
        </div>

        <div ref={confidenceSectionRef}>
          <SectionCard
            title="Confidence"
            subtitle="Why this summary is strong, partial, or still tentative."
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                gap: "12px",
                marginBottom: "10px",
                flexWrap: "wrap",
              }}
            >
              <ConfidenceBadge confidence={diagnosticsConfidence} />
              <div style={{ fontSize: "12px", color: "#475569" }}>
                {diagnosticsConfidence.score != null
                  ? `Score ${(diagnosticsConfidence.score * 100).toFixed(0)}%`
                  : "Score unavailable"}
              </div>
            </div>

            {diagnosticsConfidence.reasons.length > 0 ? (
              <>
                <ul style={{ margin: 0, paddingLeft: "18px", color: "#1f2937" }}>
                  {visibleConfidenceReasons.map((reason) => (
                    <li key={reason} style={{ marginBottom: "4px", lineHeight: 1.35 }}>
                      {reason}
                    </li>
                  ))}
                </ul>
                {(hiddenConfidenceReasonCount > 0 || diagnosticsConfidence.reasons.length > 2) && (
                  <button
                    onClick={() => setShowAllConfidenceReasons((current) => !current)}
                    style={{
                      ...secondaryToggleButtonStyle,
                      marginTop: "8px",
                    }}
                  >
                    {showAllConfidenceReasons
                      ? "Show less"
                      : `Show all (${diagnosticsConfidence.reasons.length})`}
                  </button>
                )}
              </>
            ) : (
              <EmptyStateText label="No confidence rationale was available." />
            )}
          </SectionCard>
        </div>
      </div>

      <div ref={repeatedFailuresSectionRef}>
        <SectionCard
          title="Repeated Failures"
          subtitle="Recurrence is grouped by subject and failure reason to keep the summary compact."
        >
          {visibleRepeatedFailures.length > 0 ? (
            <div style={{ display: "grid", gap: "8px" }}>
              {visibleRepeatedFailures.map((group) => (
                <RepeatedFailureRow key={group.id} group={group} />
              ))}
              {hiddenRepeatedFailureCount > 0 && (
                <div style={{ fontSize: "12px", color: "#64748b" }}>
                  {hiddenRepeatedFailureCount} more repeated failure group(s) were detected.
                </div>
              )}
              {(hiddenRepeatedFailureCount > 0 || repeatedFailures.length > 2) && (
                <button
                  onClick={() => setShowAllRepeatedFailures((current) => !current)}
                  style={secondaryToggleButtonStyle}
                >
                  {showAllRepeatedFailures ? "Show less" : `Show all (${repeatedFailures.length})`}
                </button>
              )}
            </div>
          ) : (
            <EmptyStateText label="No repeated failure patterns were detected." />
          )}
        </SectionCard>
      </div>

      {diagnostics.length > 0 && (
        <div ref={diagnosticsGuidanceSectionRef} style={{ marginBottom: "20px" }}>
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

      <div style={{ marginTop: "16px" }}>
        <div
          style={{
            fontSize: "11px",
            textTransform: "uppercase",
            letterSpacing: "0.05em",
            color: "#64748b",
            marginBottom: "8px",
            fontWeight: 700,
          }}
        >
          Activity Metrics
        </div>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
            gap: "10px",
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
    </div>
  );
}

type SummaryConclusionSection = "coverage" | "confidence" | "repeatedFailures" | "guidance";

type SummaryConclusionAction =
  | {
    kind: "section";
    section: SummaryConclusionSection;
  }
  | {
    kind: "timeline";
    eventType?: IntuneEventType | "All";
    status?: IntuneStatus | "All";
    filePath?: string | null;
    selectFirstMatch?: boolean;
  };

interface SummaryConclusion {
  id: string;
  text: string;
  tone: "neutral" | "info" | "warning" | "critical";
  hint: string;
  action: SummaryConclusionAction;
}

const secondaryToggleButtonStyle: React.CSSProperties = {
  fontSize: "11px",
  padding: "4px 8px",
  borderRadius: "4px",
  border: "1px solid #cbd5e1",
  backgroundColor: "#ffffff",
  color: "#334155",
  cursor: "pointer",
};

function ConclusionButton({
  conclusion,
  onClick,
}: {
  conclusion: SummaryConclusion;
  onClick: () => void;
}) {
  const tone = getConclusionTone(conclusion.tone);

  return (
    <button
      onClick={onClick}
      style={{
        width: "100%",
        display: "grid",
        gridTemplateColumns: "auto minmax(0, 1fr) auto",
        gap: "10px",
        alignItems: "center",
        textAlign: "left",
        padding: "8px 10px",
        borderRadius: "6px",
        border: `1px solid ${tone.border}`,
        backgroundColor: tone.background,
        cursor: "pointer",
      }}
    >
      <span
        style={{
          width: "8px",
          height: "8px",
          borderRadius: "999px",
          backgroundColor: tone.accent,
          flexShrink: 0,
        }}
      />
      <span style={{ fontSize: "12px", color: "#0f172a", lineHeight: 1.35 }}>{conclusion.text}</span>
      <span
        style={{
          fontSize: "10px",
          fontWeight: 700,
          textTransform: "uppercase",
          letterSpacing: "0.05em",
          color: tone.label,
          whiteSpace: "nowrap",
        }}
      >
        {conclusion.hint}
      </span>
    </button>
  );
}

function SectionCard({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle: string;
  children: React.ReactNode;
}) {
  return (
    <div
      style={{
        border: "1px solid #e5e7eb",
        borderRadius: "8px",
        backgroundColor: "#ffffff",
        padding: "12px 14px",
      }}
    >
      <div style={{ marginBottom: "10px" }}>
        <div style={{ fontSize: "13px", fontWeight: 700, color: "#111827" }}>{title}</div>
        <div style={{ fontSize: "11px", color: "#6b7280", marginTop: "2px" }}>{subtitle}</div>
      </div>
      {children}
    </div>
  );
}

function CompactFact({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color?: string;
}) {
  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "baseline",
        gap: "6px",
        padding: "5px 8px",
        borderRadius: "999px",
        border: "1px solid #dbe3ee",
        backgroundColor: "#f8fafc",
      }}
    >
      <span style={{ fontSize: "10px", fontWeight: 700, color: "#64748b", textTransform: "uppercase" }}>
        {label}
      </span>
      <span style={{ fontSize: "12px", fontWeight: 700, color: color ?? "#0f172a" }}>{value}</span>
    </div>
  );
}

function CoverageRow({ file }: { file: IntuneDiagnosticsFileCoverage }) {
  const hasActivity = file.eventCount > 0 || file.downloadCount > 0;

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "minmax(0, 1fr) auto",
        gap: "8px",
        alignItems: "center",
        padding: "8px 10px",
        borderRadius: "6px",
        border: "1px solid #e5e7eb",
        backgroundColor: hasActivity ? "#fcfcfd" : "#f8fafc",
      }}
    >
      <div style={{ minWidth: 0 }}>
        <div
          title={file.filePath}
          style={{
            fontSize: "12px",
            fontWeight: 600,
            color: "#111827",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {getFileName(file.filePath)}
        </div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "6px", marginTop: "4px" }}>
          <RowStat label="Events" value={file.eventCount} color="#2563eb" />
          <RowStat label="Downloads" value={file.downloadCount} color="#ea580c" />
          {file.rotationGroup && (
            <span
              style={{
                fontSize: "10px",
                padding: "2px 6px",
                borderRadius: "999px",
                backgroundColor: file.isRotatedSegment ? "#fef3c7" : "#e0f2fe",
                color: file.isRotatedSegment ? "#92400e" : "#0f766e",
                fontWeight: 700,
              }}
            >
              {file.isRotatedSegment ? "Rotated segment" : "Rotation base"}
            </span>
          )}
        </div>
      </div>
      <div style={{ textAlign: "right", fontSize: "11px", color: "#64748b" }}>
        {file.timestampBounds ? formatTimestampBounds(file.timestampBounds) : "No timestamps"}
      </div>
    </div>
  );
}

function RowStat({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: string;
}) {
  return (
    <span
      style={{
        fontSize: "10px",
        padding: "2px 6px",
        borderRadius: "999px",
        backgroundColor: "#eef2ff",
        color,
        fontWeight: 700,
      }}
    >
      {label} {value}
    </span>
  );
}

function ConfidenceBadge({ confidence }: { confidence: IntuneDiagnosticsConfidence }) {
  const tone = getConfidenceTone(confidence.level);
  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "8px",
        padding: "6px 10px",
        borderRadius: "999px",
        border: `1px solid ${tone.border}`,
        backgroundColor: tone.background,
      }}
    >
      <span style={{ fontSize: "10px", fontWeight: 700, color: tone.labelColor, textTransform: "uppercase" }}>
        Confidence
      </span>
      <span style={{ fontSize: "12px", fontWeight: 700, color: tone.valueColor }}>{confidence.level}</span>
    </div>
  );
}

function RepeatedFailureRow({ group }: { group: IntuneRepeatedFailureGroup }) {
  return (
    <div
      style={{
        border: "1px solid #e5e7eb",
        borderRadius: "6px",
        padding: "10px 12px",
        backgroundColor: "#fcfcfd",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          gap: "12px",
          alignItems: "baseline",
          flexWrap: "wrap",
        }}
      >
        <div style={{ fontSize: "12px", fontWeight: 700, color: "#111827" }}>
          {buildRepeatedFailureConclusion(group)}
        </div>
        <span style={{ fontSize: "11px", color: "#b91c1c", fontWeight: 700 }}>
          {group.occurrences} occurrence{group.occurrences === 1 ? "" : "s"}
        </span>
      </div>

      <div style={{ fontSize: "12px", color: "#374151", marginTop: "4px" }}>{group.name}</div>

      <div style={{ display: "flex", flexWrap: "wrap", gap: "8px", marginTop: "6px", fontSize: "11px", color: "#64748b" }}>
        <span>{formatEventTypeLabel(group.eventType)}</span>
        <span>{group.sourceFiles.length} file(s)</span>
        {group.errorCode && <span>Error {group.errorCode}</span>}
        {group.timestampBounds && <span>{formatTimestampBounds(group.timestampBounds)}</span>}
      </div>
    </div>
  );
}

function EmptyStateText({ label }: { label: string }) {
  return <div style={{ fontSize: "12px", color: "#64748b" }}>{label}</div>;
}

function buildSummaryConclusions({
  summary,
  diagnostics,
  diagnosticsCoverage,
  diagnosticsConfidence,
  repeatedFailures,
}: {
  summary: IntuneSummary;
  diagnostics: IntuneDiagnosticInsight[];
  diagnosticsCoverage: IntuneDiagnosticsCoverage;
  diagnosticsConfidence: IntuneDiagnosticsConfidence;
  repeatedFailures: IntuneRepeatedFailureGroup[];
}): SummaryConclusion[] {
  const conclusions: SummaryConclusion[] = [];
  const topRepeatedFailure = repeatedFailures[0];
  const topDiagnostic =
    diagnostics.find((diagnostic) => diagnostic.severity === "Error") ??
    diagnostics.find((diagnostic) => diagnostic.severity === "Warning") ??
    diagnostics[0];

  if (topRepeatedFailure) {
    conclusions.push({
      id: "repeated-failure",
      text: `Start with ${truncateText(topRepeatedFailure.name, 88)}: ${topRepeatedFailure.occurrences} ${formatEventTypeLabel(topRepeatedFailure.eventType).toLowerCase()} failures repeat with the same outcome.`,
      tone: "critical",
      hint: "Filter timeline",
      action: {
        kind: "timeline",
        eventType: topRepeatedFailure.eventType,
        status: "Failed",
        filePath: null,
        selectFirstMatch: true,
      },
    });
  } else if (summary.failed > 0 || summary.timedOut > 0) {
    conclusions.push({
      id: "failed-events",
      text: `Review the failure queue: ${summary.failed + summary.timedOut} event(s) finished failed in this analysis window.`,
      tone: "warning",
      hint: "Filter timeline",
      action: {
        kind: "timeline",
        eventType: "All",
        status: "Failed",
        filePath: null,
        selectFirstMatch: true,
      },
    });
  }

  if (topDiagnostic) {
    conclusions.push({
      id: `diagnostic-${topDiagnostic.id}`,
      text: `Next check: ${topDiagnostic.title}. ${toSentence(topDiagnostic.summary)}`,
      tone:
        topDiagnostic.severity === "Error"
          ? "critical"
          : topDiagnostic.severity === "Warning"
            ? "warning"
            : "info",
      hint: "Jump to guidance",
      action: {
        kind: "section",
        section: "guidance",
      },
    });
  }

  if (diagnosticsCoverage.dominantSource) {
    const dominantSource = diagnosticsCoverage.dominantSource;
    conclusions.push({
      id: "dominant-source",
      text: `Use ${getFileName(dominantSource.filePath)} as the lead evidence file: it contributes ${formatEventShare(dominantSource.eventShare ?? 0)} of extracted events.`,
      tone: diagnosticsConfidence.level === "Low" ? "warning" : "neutral",
      hint: "Scope timeline",
      action: {
        kind: "timeline",
        eventType: "All",
        status: "All",
        filePath: dominantSource.filePath,
      },
    });
  } else if (diagnosticsConfidence.reasons[0]) {
    conclusions.push({
      id: "confidence",
      text: `Treat this summary as ${diagnosticsConfidence.level.toLowerCase()} confidence because ${toSentence(diagnosticsConfidence.reasons[0]).replace(/[.]$/, "")}.`,
      tone: diagnosticsConfidence.level === "Low" ? "warning" : "info",
      hint: "Jump to confidence",
      action: {
        kind: "section",
        section: "confidence",
      },
    });
  }

  return conclusions.slice(0, 3);
}

function matchesTimelineAction(
  event: IntuneEvent,
  action: Extract<SummaryConclusionAction, { kind: "timeline" }>
): boolean {
  if (action.filePath != null && event.sourceFile !== action.filePath) {
    return false;
  }

  if (action.eventType != null && action.eventType !== "All" && event.eventType !== action.eventType) {
    return false;
  }

  if (action.status != null && action.status !== "All" && event.status !== action.status) {
    return false;
  }

  return true;
}

function buildDerivedCoverage(
  sourceFiles: string[],
  events: IntuneEvent[],
  downloads: DownloadStat[]
): IntuneDiagnosticsCoverage {
  const filePaths = new Set<string>(sourceFiles);
  for (const event of events) {
    filePaths.add(event.sourceFile);
  }

  const eventCounts = new Map<string, number>();
  const fileEvents = new Map<string, IntuneEvent[]>();
  for (const event of events) {
    eventCounts.set(event.sourceFile, (eventCounts.get(event.sourceFile) ?? 0) + 1);
    const existing = fileEvents.get(event.sourceFile);
    if (existing) {
      existing.push(event);
    } else {
      fileEvents.set(event.sourceFile, [event]);
    }
  }

  const rotationEntries = Array.from(filePaths).map((filePath) => ({
    filePath,
    rotation: detectRotationMetadata(filePath),
  }));
  const rotationCounts = new Map<string, number>();
  for (const entry of rotationEntries) {
    if (entry.rotation.rotationGroup) {
      rotationCounts.set(
        entry.rotation.rotationGroup,
        (rotationCounts.get(entry.rotation.rotationGroup) ?? 0) + 1
      );
    }
  }

  const files = rotationEntries
    .map(({ filePath, rotation }) => {
      const groupedEvents = fileEvents.get(filePath) ?? [];
      const timestampBounds = buildTimestampBounds(groupedEvents, []);
      const rotationGroup =
        rotation.rotationGroup && (rotationCounts.get(rotation.rotationGroup) ?? 0) > 1
          ? rotation.rotationGroup
          : null;

      return {
        filePath,
        eventCount: eventCounts.get(filePath) ?? 0,
        downloadCount: 0,
        timestampBounds,
        isRotatedSegment: rotationGroup != null ? rotation.isRotatedSegment : false,
        rotationGroup,
      } satisfies IntuneDiagnosticsFileCoverage;
    })
    .sort((left, right) => {
      const leftActivity = left.eventCount + left.downloadCount;
      const rightActivity = right.eventCount + right.downloadCount;
      return rightActivity - leftActivity || left.filePath.localeCompare(right.filePath);
    });

  const overallTimestampBounds = buildTimestampBounds(events, downloads);
  const mergedTimestampBounds = mergeTimestampBounds([
    ...files
      .map((file) => file.timestampBounds)
      .filter((value): value is IntuneTimestampBounds => value != null),
    ...(overallTimestampBounds ? [overallTimestampBounds] : []),
  ]);

  return {
    files,
    timestampBounds: mergedTimestampBounds,
    hasRotatedLogs: files.some((file) => file.rotationGroup != null),
    dominantSource: buildDominantSource(files, events),
  };
}

function buildDominantSourceLabel(
  dominantSource: NonNullable<IntuneDiagnosticsCoverage["dominantSource"]>
): string {
  const share = dominantSource.eventShare != null ? ` (${formatEventShare(dominantSource.eventShare)})` : "";
  return `${getFileName(dominantSource.filePath)}${share}`;
}

function buildDerivedConfidence(
  summary: IntuneSummary,
  coverage: IntuneDiagnosticsCoverage,
  repeatedFailures: IntuneRepeatedFailureGroup[],
  events: IntuneEvent[]
): IntuneDiagnosticsConfidence {
  if (summary.totalEvents === 0 && summary.totalDownloads === 0) {
    return {
      level: "Unknown",
      score: null,
      reasons: ["No Intune events or download evidence were available."],
    };
  }

  let score = 0.15;
  const reasons: string[] = [];
  const failedEvents = events.filter((event) => event.status === "Failed" || event.status === "Timeout").length;
  const distinctKinds = distinctSourceKinds(coverage.files);
  const contributingFiles = coverage.files.filter((file) => file.eventCount > 0 || file.downloadCount > 0).length;

  if (summary.totalEvents >= 20) {
    score += 0.25;
    reasons.push(`${summary.totalEvents} events were extracted across the selected logs.`);
  } else if (summary.totalEvents >= 8) {
    score += 0.15;
    reasons.push(`${summary.totalEvents} events were extracted across the selected logs.`);
  } else if (summary.totalEvents > 0) {
    score += 0.05;
    reasons.push(`Only ${summary.totalEvents} event(s) were extracted, so the evidence set is narrow.`);
  }

  if (failedEvents >= 4) {
    score += 0.2;
    reasons.push(`${failedEvents} failed or timed-out event(s) were available for review.`);
  } else if (failedEvents > 0) {
    score += 0.1;
    reasons.push(`${failedEvents} failed or timed-out event(s) were available for review.`);
  }

  if (distinctKinds >= 3) {
    score += 0.2;
    reasons.push(`Evidence spans ${distinctKinds} distinct Intune log families.`);
  } else if (distinctKinds === 2) {
    score += 0.1;
    reasons.push("Evidence spans two distinct Intune log families.");
  }

  if (coverage.timestampBounds) {
    score += 0.1;
    reasons.push("Parsed timestamps were available for the overall diagnostics window.");
  }

  if (repeatedFailures.length > 0) {
    score += 0.15;
    reasons.push(`${repeatedFailures.length} repeated failure group(s) were identified deterministically.`);
  }

  if (coverage.hasRotatedLogs) {
    score += 0.05;
    reasons.push("Rotated log segments were available, which improves continuity across retries.");
  }

  if (contributingFiles <= 1) {
    score -= 0.15;
    reasons.push("Evidence comes from a single contributing source file.");
  }

  if (coverage.files.some((file) => (file.eventCount > 0 || file.downloadCount > 0) && file.timestampBounds == null)) {
    score -= 0.1;
    reasons.push("Some contributing files had no parseable timestamps, which weakens ordering confidence.");
  }

  if (summary.totalEvents === 0 && summary.totalDownloads > 0) {
    score -= 0.2;
    reasons.push("Only download statistics were available; no correlated Intune events were extracted.");
  }

  if (summary.inProgress + summary.pending > summary.failed + summary.succeeded && summary.totalEvents > 0) {
    score -= 0.1;
    reasons.push("Most observed work is still pending or in progress, so the failure picture may be incomplete.");
  }

  if (hasAppOrDownloadFailures(events) && !hasSourceKind(coverage.files, "appworkload")) {
    score -= 0.15;
    reasons.push("AppWorkload evidence was not available for app or download failures.");
  }

  if (hasPolicyFailures(events) && !hasSourceKind(coverage.files, "appactionprocessor")) {
    score -= 0.15;
    reasons.push("AppActionProcessor evidence was not available for applicability or policy failures.");
  }

  if (hasScriptFailures(events) && !hasSourceKind(coverage.files, "agentexecutor") && !hasSourceKind(coverage.files, "healthscripts")) {
    score -= 0.15;
    reasons.push("AgentExecutor or HealthScripts evidence was not available for script-related failures.");
  }

  score = Math.max(0, Math.min(1, score));

  return {
    level: score >= 0.75 ? "High" : score >= 0.45 ? "Medium" : "Low",
    score: Math.round(score * 1000) / 1000,
    reasons,
  };
}

function buildDerivedRepeatedFailures(events: IntuneEvent[]): IntuneRepeatedFailureGroup[] {
  const groups = new Map<
    string,
    {
      name: string;
      eventType: IntuneEventType;
      errorCode: string | null;
      occurrences: number;
      sourceFiles: Set<string>;
      sampleEventIds: number[];
      earliest: string | null;
      latest: string | null;
      reasonDisplay: string;
    }
  >();

  for (const event of events) {
    if (event.status !== "Failed" && event.status !== "Timeout") {
      continue;
    }

    const reason = normalizeFailureReason(event);
    const subjectKey = event.guid ?? normalizeIdentifier(event.name);
    const key = `${event.eventType}|${subjectKey}|${reason.key}`;
    const existing = groups.get(key);

    if (existing) {
      existing.occurrences += 1;
      existing.sourceFiles.add(event.sourceFile);
      if (existing.sampleEventIds.length < 5) {
        existing.sampleEventIds.push(event.id);
      }
      if (event.name.length < existing.name.length) {
        existing.name = event.name;
      }
      if (!existing.errorCode && event.errorCode) {
        existing.errorCode = event.errorCode;
      }
      const timestamp = event.startTime ?? event.endTime;
      if (timestamp) {
        existing.earliest = pickEarlierTimestamp(existing.earliest, timestamp);
        existing.latest = pickLaterTimestamp(existing.latest, timestamp);
      }
      continue;
    }

    const timestamp = event.startTime ?? event.endTime;
    groups.set(key, {
      name: event.name,
      eventType: event.eventType,
      errorCode: event.errorCode,
      occurrences: 1,
      sourceFiles: new Set([event.sourceFile]),
      sampleEventIds: [event.id],
      earliest: timestamp,
      latest: timestamp,
      reasonDisplay: reason.display,
    });
  }

  return Array.from(groups.entries())
    .filter(([, group]) => group.occurrences >= 2)
    .map(([key, group]) => ({
      id: `repeated-${normalizeIdentifier(key)}`,
      name: `${group.name}: ${group.reasonDisplay}`,
      eventType: group.eventType,
      errorCode: group.errorCode,
      occurrences: group.occurrences,
      timestampBounds:
        group.earliest && group.latest
          ? {
            firstTimestamp: group.earliest,
            lastTimestamp: group.latest,
          }
          : null,
      sourceFiles: Array.from(group.sourceFiles).sort((left, right) => left.localeCompare(right)),
      sampleEventIds: group.sampleEventIds,
    }))
    .sort((left, right) => right.occurrences - left.occurrences || left.name.localeCompare(right.name));
}

function buildRepeatedFailureConclusion(group: IntuneRepeatedFailureGroup): string {
  const subject =
    group.eventType === "Win32App" || group.eventType === "WinGetApp"
      ? "Repeated app failures for the same reason"
      : group.eventType === "PowerShellScript" || group.eventType === "Remediation"
        ? "Repeated script failures for the same reason"
        : "Repeated failures for the same reason";

  return subject;
}

function normalizeFailureReason(event: IntuneEvent): { key: string; display: string } {
  if (event.errorCode) {
    return {
      key: `code-${normalizeIdentifier(event.errorCode)}`,
      display: event.errorCode,
    };
  }

  const detail = event.detail.toLowerCase();
  const patterns: Array<[string, string]> = [
    ["access is denied", "access is denied"],
    ["permission denied", "permission denied"],
    ["unauthorized", "unauthorized"],
    ["not applicable", "not applicable"],
    ["will not be enforced", "will not be enforced"],
    ["requirement rule", "requirement rule blocked enforcement"],
    ["detection rule", "detection rule blocked enforcement"],
    ["hash validation failed", "hash validation failed"],
    ["hash mismatch", "hash mismatch"],
    ["timed out", "operation timed out"],
    ["timeout", "operation timed out"],
  ];

  for (const [pattern, label] of patterns) {
    if (detail.includes(pattern)) {
      return {
        key: normalizeIdentifier(label),
        display: label,
      };
    }
  }

  const compactDetail = event.detail.trim().replace(/\s+/g, " ");
  const fallback = compactDetail.length > 72 ? `${compactDetail.slice(0, 69)}...` : compactDetail;
  return {
    key: normalizeIdentifier(fallback || event.status),
    display: fallback || event.status,
  };
}

function buildTimestampBounds(
  events: IntuneEvent[],
  downloads: DownloadStat[]
): IntuneTimestampBounds | null {
  const timestamps = [
    ...events.flatMap((event) => [event.startTime, event.endTime]),
    ...downloads.map((download) => download.timestamp),
  ].filter((value): value is string => Boolean(value));

  let earliest: string | null = null;
  let latest: string | null = null;
  for (const timestamp of timestamps) {
    earliest = pickEarlierTimestamp(earliest, timestamp);
    latest = pickLaterTimestamp(latest, timestamp);
  }

  if (!earliest || !latest) {
    return null;
  }

  return {
    firstTimestamp: earliest,
    lastTimestamp: latest,
  };
}

function mergeTimestampBounds(boundsList: IntuneTimestampBounds[]): IntuneTimestampBounds | null {
  let earliest: string | null = null;
  let latest: string | null = null;

  for (const bounds of boundsList) {
    if (bounds.firstTimestamp) {
      earliest = pickEarlierTimestamp(earliest, bounds.firstTimestamp);
    }
    if (bounds.lastTimestamp) {
      latest = pickLaterTimestamp(latest, bounds.lastTimestamp);
    }
  }

  if (!earliest || !latest) {
    return null;
  }

  return {
    firstTimestamp: earliest,
    lastTimestamp: latest,
  };
}

function buildDominantSource(
  files: IntuneDiagnosticsFileCoverage[],
  events: IntuneEvent[]
): IntuneDiagnosticsCoverage["dominantSource"] {
  const totalEvents = events.length;
  const scores = new Map<string, number>();

  for (const event of events) {
    scores.set(event.sourceFile, (scores.get(event.sourceFile) ?? 0) + eventSignalScore(event));
  }

  const rankedFiles = files
    .map((file) => ({ file, score: scores.get(file.filePath) ?? 0 }))
    .filter((entry) => entry.score > 0)
    .sort((left, right) => {
      return (
        right.score - left.score ||
        right.file.eventCount - left.file.eventCount ||
        right.file.downloadCount - left.file.downloadCount ||
        left.file.filePath.localeCompare(right.file.filePath)
      );
    });

  const best = rankedFiles[0];
  if (!best) {
    return null;
  }

  return {
    filePath: best.file.filePath,
    eventCount: best.file.eventCount,
    eventShare: totalEvents > 0 ? best.file.eventCount / totalEvents : null,
  };
}

function eventSignalScore(event: IntuneEvent): number {
  const statusWeight =
    event.status === "Failed" || event.status === "Timeout"
      ? 5
      : event.status === "Success"
        ? 2
        : 1;
  const typeWeight =
    event.eventType === "ContentDownload"
      ? 4
      : event.eventType === "Win32App" || event.eventType === "WinGetApp"
        ? 4
        : event.eventType === "PowerShellScript" || event.eventType === "Remediation"
          ? 4
          : event.eventType === "PolicyEvaluation"
            ? 3
            : 1;
  const errorWeight = event.errorCode ? 1 : 0;
  return statusWeight + typeWeight + errorWeight;
}

function distinctSourceKinds(files: IntuneDiagnosticsFileCoverage[]): number {
  return new Set(
    files
      .filter((file) => file.eventCount > 0 || file.downloadCount > 0)
      .map((file) => sourceKindKey(file.filePath))
  ).size;
}

function hasSourceKind(files: IntuneDiagnosticsFileCoverage[], kind: string): boolean {
  return files.some(
    (file) => (file.eventCount > 0 || file.downloadCount > 0) && sourceKindKey(file.filePath) === kind
  );
}

function sourceKindKey(filePath: string): string {
  const name = getFileName(filePath).toLowerCase();
  if (name.includes("appworkload")) {
    return "appworkload";
  }
  if (name.includes("appactionprocessor")) {
    return "appactionprocessor";
  }
  if (name.includes("agentexecutor")) {
    return "agentexecutor";
  }
  if (name.includes("healthscripts")) {
    return "healthscripts";
  }
  if (name.includes("intunemanagementextension")) {
    return "intunemanagementextension";
  }
  return "other";
}

function hasAppOrDownloadFailures(events: IntuneEvent[]): boolean {
  return events.some(
    (event) =>
      (event.status === "Failed" || event.status === "Timeout") &&
      (event.eventType === "Win32App" || event.eventType === "WinGetApp" || event.eventType === "ContentDownload")
  );
}

function hasPolicyFailures(events: IntuneEvent[]): boolean {
  return events.some(
    (event) =>
      (event.status === "Failed" || event.status === "Timeout") && event.eventType === "PolicyEvaluation"
  );
}

function hasScriptFailures(events: IntuneEvent[]): boolean {
  return events.some(
    (event) =>
      (event.status === "Failed" || event.status === "Timeout") &&
      (event.eventType === "PowerShellScript" || event.eventType === "Remediation")
  );
}

function detectRotationMetadata(filePath: string): {
  isRotatedSegment: boolean;
  rotationGroup: string | null;
} {
  const fileName = getFileName(filePath);
  const stem = fileName.replace(/\.[^.]+$/, "");

  for (const separator of [".", "-", "_"]) {
    const index = stem.lastIndexOf(separator);
    if (index > 0) {
      const base = stem.slice(0, index);
      const suffix = stem.slice(index + 1);
      if (isRotationSuffix(suffix)) {
        return {
          isRotatedSegment: true,
          rotationGroup: base.toLowerCase(),
        };
      }
    }
  }

  return {
    isRotatedSegment: false,
    rotationGroup: stem ? stem.toLowerCase() : null,
  };
}

function isRotationSuffix(value: string): boolean {
  const normalized = value.trim().toLowerCase();
  if (!normalized) {
    return false;
  }
  if (/^\d+$/.test(normalized)) {
    return true;
  }
  if (normalized.startsWith("lo_") || normalized === "bak" || normalized === "old") {
    return true;
  }
  return /^\d{8}$/.test(normalized);
}

function pickEarlierTimestamp(current: string | null, candidate: string): string {
  if (!current) {
    return candidate;
  }
  const currentValue = Date.parse(current);
  const candidateValue = Date.parse(candidate);
  if (Number.isNaN(currentValue) || Number.isNaN(candidateValue)) {
    return candidate.localeCompare(current) < 0 ? candidate : current;
  }
  return candidateValue < currentValue ? candidate : current;
}

function pickLaterTimestamp(current: string | null, candidate: string): string {
  if (!current) {
    return candidate;
  }
  const currentValue = Date.parse(current);
  const candidateValue = Date.parse(candidate);
  if (Number.isNaN(currentValue) || Number.isNaN(candidateValue)) {
    return candidate.localeCompare(current) > 0 ? candidate : current;
  }
  return candidateValue > currentValue ? candidate : current;
}

function formatTimestampBounds(bounds: IntuneTimestampBounds): string {
  const start = bounds.firstTimestamp ? formatTimestamp(bounds.firstTimestamp) : "Unknown start";
  const end = bounds.lastTimestamp ? formatTimestamp(bounds.lastTimestamp) : "Unknown end";
  return `${start} to ${end}`;
}

function formatTimestamp(value: string): string {
  const timestamp = new Date(value);
  if (Number.isNaN(timestamp.getTime())) {
    return value;
  }
  return timestamp.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatEventShare(value: number): string {
  return `${(value * 100).toFixed(value >= 0.1 ? 0 : 1)}%`;
}

function getConfidenceTone(level: IntuneDiagnosticsConfidence["level"]) {
  switch (level) {
    case "High":
      return {
        border: "#86efac",
        background: "#f0fdf4",
        labelColor: "#166534",
        valueColor: "#166534",
      };
    case "Medium":
      return {
        border: "#fde68a",
        background: "#fffbeb",
        labelColor: "#92400e",
        valueColor: "#92400e",
      };
    case "Low":
      return {
        border: "#fecaca",
        background: "#fef2f2",
        labelColor: "#991b1b",
        valueColor: "#991b1b",
      };
    case "Unknown":
    default:
      return {
        border: "#cbd5e1",
        background: "#f8fafc",
        labelColor: "#475569",
        valueColor: "#334155",
      };
  }
}

function normalizeIdentifier(value: string): string {
  return value.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "") || "unknown";
}

function formatEventTypeLabel(eventType: IntuneEventType): string {
  switch (eventType) {
    case "Win32App":
      return "Win32 app";
    case "WinGetApp":
      return "WinGet app";
    case "PowerShellScript":
      return "PowerShell script";
    case "PolicyEvaluation":
      return "Policy evaluation";
    case "ContentDownload":
      return "Content download";
    default:
      return eventType;
  }
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

function getConclusionTone(tone: SummaryConclusion["tone"]) {
  switch (tone) {
    case "critical":
      return {
        accent: "#b91c1c",
        border: "#fecaca",
        background: "#fff7f7",
        label: "#991b1b",
      };
    case "warning":
      return {
        accent: "#b45309",
        border: "#fde68a",
        background: "#fffbeb",
        label: "#92400e",
      };
    case "info":
      return {
        accent: "#2563eb",
        border: "#bfdbfe",
        background: "#eff6ff",
        label: "#1d4ed8",
      };
    case "neutral":
    default:
      return {
        accent: "#475569",
        border: "#dbe3ee",
        background: "#ffffff",
        label: "#475569",
      };
  }
}

function toSentence(value: string): string {
  const normalized = value.trim().replace(/\s+/g, " ");
  if (!normalized) {
    return "No further detail was available.";
  }

  const firstSentence = normalized.match(/^.+?[.!?](?:\s|$)/)?.[0]?.trim() ?? normalized;
  return /[.!?]$/.test(firstSentence) ? firstSentence : `${firstSentence}.`;
}

function truncateText(value: string, maxLength: number): string {
  if (value.length <= maxLength) {
    return value;
  }

  return `${value.slice(0, Math.max(0, maxLength - 3)).trimEnd()}...`;
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
        padding: "10px 11px",
        border: "1px solid #e5e7eb",
        borderRadius: "6px",
        borderLeft: `3px solid ${color || "#9ca3af"}`,
        backgroundColor: "#ffffff",
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
          fontSize: "20px",
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
