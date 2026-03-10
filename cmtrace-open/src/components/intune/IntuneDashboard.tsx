import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useIntuneStore } from "../../stores/intune-store";
import { analyzeIntuneLogs } from "../../lib/commands";
import { EventTimeline } from "./EventTimeline";
import { DownloadStats } from "./DownloadStats";
import type { IntuneEventType, IntuneStatus } from "../../types/intune";

type TabId = "timeline" | "downloads" | "summary";

export function IntuneDashboard() {
  const [activeTab, setActiveTab] = useState<TabId>("timeline");
  const events = useIntuneStore((s) => s.events);
  const downloads = useIntuneStore((s) => s.downloads);
  const summary = useIntuneStore((s) => s.summary);
  const sourceFile = useIntuneStore((s) => s.sourceFile);
  const isAnalyzing = useIntuneStore((s) => s.isAnalyzing);
  const setResults = useIntuneStore((s) => s.setResults);
  const setAnalyzing = useIntuneStore((s) => s.setAnalyzing);
  const filterEventType = useIntuneStore((s) => s.filterEventType);
  const filterStatus = useIntuneStore((s) => s.filterStatus);
  const setFilterEventType = useIntuneStore((s) => s.setFilterEventType);
  const setFilterStatus = useIntuneStore((s) => s.setFilterStatus);

  const handleAnalyze = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "IME Logs", extensions: ["log"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    if (!selected) return;

    setAnalyzing(true);
    try {
      const result = await analyzeIntuneLogs(selected);
      setResults(
        result.events,
        result.downloads,
        result.summary,
        result.sourceFile
      );
    } catch (err) {
      console.error("Intune analysis failed:", err);
    } finally {
      setAnalyzing(false);
    }
  }, [setResults, setAnalyzing]);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "#ffffff",
      }}
    >
      {/* Header */}
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
        <button onClick={handleAnalyze} disabled={isAnalyzing}>
          {isAnalyzing ? "Analyzing..." : "Open IME Log"}
        </button>
        {sourceFile && (
          <span
            style={{
              fontSize: "11px",
              color: "#666",
              marginLeft: "auto",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
              maxWidth: "400px",
            }}
            title={sourceFile}
          >
            {sourceFile}
          </span>
        )}
      </div>

      {/* Summary bar */}
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
          <SummaryBadge
            label="Win32"
            value={summary.win32Apps}
            color="#6366f1"
          />
          <SummaryBadge
            label="WinGet"
            value={summary.wingetApps}
            color="#8b5cf6"
          />
          <SummaryBadge
            label="Scripts"
            value={summary.scripts}
            color="#0ea5e9"
          />
          <SummaryBadge
            label="Remed."
            value={summary.remediations}
            color="#14b8a6"
          />
          <div style={{ width: "1px", backgroundColor: "#d1d5db" }} />
          <SummaryBadge
            label="Succeeded"
            value={summary.succeeded}
            color="#22c55e"
          />
          <SummaryBadge
            label="Failed"
            value={summary.failed}
            color="#ef4444"
          />
          <SummaryBadge
            label="In Progress"
            value={summary.inProgress}
            color="#3b82f6"
          />
          {summary.logTimeSpan && (
            <>
              <div style={{ width: "1px", backgroundColor: "#d1d5db" }} />
              <span style={{ color: "#6b7280" }}>
                Span: {summary.logTimeSpan}
              </span>
            </>
          )}
        </div>
      )}

      {/* Tabs + Filters */}
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
        <TabButton
          label="Timeline"
          active={activeTab === "timeline"}
          onClick={() => setActiveTab("timeline")}
        />
        <TabButton
          label="Downloads"
          active={activeTab === "downloads"}
          onClick={() => setActiveTab("downloads")}
        />
        <TabButton
          label="Summary"
          active={activeTab === "summary"}
          onClick={() => setActiveTab("summary")}
        />

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
                setFilterEventType(
                  e.target.value as IntuneEventType | "All"
                )
              }
              style={{ fontSize: "11px", padding: "1px 4px" }}
            >
              <option value="All">All</option>
              <option value="Win32App">Win32 App</option>
              <option value="WinGetApp">WinGet App</option>
              <option value="PowerShellScript">PowerShell Script</option>
              <option value="Remediation">Remediation</option>
              <option value="Esp">ESP</option>
              <option value="SyncSession">Sync Session</option>
            </select>

            <label style={{ fontSize: "11px", color: "#666" }}>Status:</label>
            <select
              value={filterStatus}
              onChange={(e) =>
                setFilterStatus(e.target.value as IntuneStatus | "All")
              }
              style={{ fontSize: "11px", padding: "1px 4px" }}
            >
              <option value="All">All</option>
              <option value="Success">Success</option>
              <option value="Failed">Failed</option>
              <option value="InProgress">In Progress</option>
            </select>
          </>
        )}
      </div>

      {/* Tab content */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {events.length === 0 && !isAnalyzing ? (
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
            Open an IntuneManagementExtension.log file to analyze
          </div>
        ) : (
          <>
            {activeTab === "timeline" && <EventTimeline events={events} />}
            {activeTab === "downloads" && (
              <DownloadStats downloads={downloads} />
            )}
            {activeTab === "summary" && summary && (
              <SummaryView summary={summary} sourceFile={sourceFile} />
            )}
          </>
        )}
      </div>
    </div>
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
      <span style={{ fontWeight: "bold", color: color || "#111" }}>
        {value}
      </span>
    </span>
  );
}

function TabButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        fontSize: "12px",
        padding: "3px 10px",
        border: "1px solid #d1d5db",
        borderBottom: active ? "2px solid #3b82f6" : "1px solid #d1d5db",
        borderRadius: "3px 3px 0 0",
        backgroundColor: active ? "#fff" : "#f3f4f6",
        fontWeight: active ? 600 : 400,
        cursor: "pointer",
      }}
    >
      {label}
    </button>
  );
}

function SummaryView({
  summary,
  sourceFile,
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
    totalDownloads: number;
    logTimeSpan: string | null;
  };
  sourceFile: string | null;
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
          <strong>Source:</strong> {sourceFile}
        </div>
      )}

      {summary.logTimeSpan && (
        <div style={{ marginBottom: "12px", color: "#666" }}>
          <strong>Log Time Span:</strong> {summary.logTimeSpan}
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
        <SummaryCard
          title="Win32 Apps"
          value={summary.win32Apps}
          color="#6366f1"
        />
        <SummaryCard
          title="WinGet Apps"
          value={summary.wingetApps}
          color="#8b5cf6"
        />
        <SummaryCard
          title="Scripts"
          value={summary.scripts}
          color="#0ea5e9"
        />
        <SummaryCard
          title="Remediations"
          value={summary.remediations}
          color="#14b8a6"
        />
        <SummaryCard
          title="Downloads"
          value={summary.totalDownloads}
          color="#f97316"
        />
        <SummaryCard
          title="Succeeded"
          value={summary.succeeded}
          color="#22c55e"
        />
        <SummaryCard
          title="Failed"
          value={summary.failed}
          color="#ef4444"
        />
        <SummaryCard
          title="In Progress"
          value={summary.inProgress}
          color="#3b82f6"
        />
      </div>
    </div>
  );
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
