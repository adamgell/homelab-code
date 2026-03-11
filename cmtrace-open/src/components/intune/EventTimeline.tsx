import { useEffect, useMemo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { IntuneEvent, IntuneStatus, IntuneEventType } from "../../types/intune";
import { useIntuneStore } from "../../stores/intune-store";

const STATUS_COLORS: Record<IntuneStatus, string> = {
  Success: "#22c55e",
  Failed: "#ef4444",
  InProgress: "#3b82f6",
  Pending: "#9ca3af",
  Timeout: "#f59e0b",
  Unknown: "#6b7280",
};

const EVENT_TYPE_LABELS: Record<IntuneEventType, string> = {
  Win32App: "Win32",
  WinGetApp: "WinGet",
  PowerShellScript: "Script",
  Remediation: "Remed.",
  Esp: "ESP",
  SyncSession: "Sync",
  PolicyEvaluation: "Policy",
  ContentDownload: "Download",
  Other: "Other",
};

interface EventTimelineProps {
  events: IntuneEvent[];
}

export function EventTimeline({ events }: EventTimelineProps) {
  const selectedEventId = useIntuneStore((s) => s.selectedEventId);
  const selectEvent = useIntuneStore((s) => s.selectEvent);
  const filterEventType = useIntuneStore((s) => s.filterEventType);
  const filterStatus = useIntuneStore((s) => s.filterStatus);

  const filteredEvents = useMemo(() => {
    return events.filter((e) => {
      if (filterEventType !== "All" && e.eventType !== filterEventType) {
        return false;
      }
      if (filterStatus !== "All" && e.status !== filterStatus) {
        return false;
      }
      return true;
    });
  }, [events, filterEventType, filterStatus]);

  useEffect(() => {
    if (selectedEventId == null) {
      return;
    }

    const selectedStillVisible = filteredEvents.some((e) => e.id === selectedEventId);
    if (!selectedStillVisible) {
      selectEvent(null);
    }
  }, [filteredEvents, selectEvent, selectedEventId]);

  const parentRef = useRef<HTMLDivElement>(null);
  const selectedIndex = useMemo(
    () => filteredEvents.findIndex((event) => event.id === selectedEventId),
    [filteredEvents, selectedEventId]
  );

  const virtualizer = useVirtualizer({
    count: filteredEvents.length,
    getScrollElement: () => parentRef.current,
    estimateSize: (index) =>
      filteredEvents[index]?.id === selectedEventId ? 168 : 72,
    overscan: 10,
  });

  useEffect(() => {
    if (selectedIndex >= 0) {
      virtualizer.scrollToIndex(selectedIndex, { align: "center" });
    }
  }, [selectedIndex, virtualizer]);

  if (events.length === 0) {
    return (
      <div style={{ padding: "20px", color: "#666", textAlign: "center" }}>
        No Intune timeline events were found in this analysis.
      </div>
    );
  }

  if (filteredEvents.length === 0) {
    return (
      <div style={{ padding: "20px", color: "#666", textAlign: "center" }}>
        No events match the current filters.
      </div>
    );
  }

  return (
    <div
      ref={parentRef}
      style={{
        overflowY: "auto",
        height: "100%",
        padding: "8px 0",
      }}
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const event = filteredEvents[virtualRow.index];
          const isSelected = selectedEventId === event.id;

          return (
            <div
              key={event.id}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${virtualRow.start}px)`,
              }}
            >
              <div
                onClick={() => selectEvent(isSelected ? null : event.id)}
                style={{
                  display: "flex",
                  alignItems: "flex-start",
                  padding: "6px 12px",
                  cursor: "pointer",
                  backgroundColor: isSelected ? "#e0e7ff" : "transparent",
                  borderLeft: `3px solid ${STATUS_COLORS[event.status]}`,
                  marginBottom: "2px",
                }}
              >
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    marginRight: "10px",
                    minWidth: "20px",
                  }}
                >
                  <div
                    style={{
                      width: "12px",
                      height: "12px",
                      borderRadius: "50%",
                      backgroundColor: STATUS_COLORS[event.status],
                      flexShrink: 0,
                      marginTop: "3px",
                    }}
                  />
                  {virtualRow.index < filteredEvents.length - 1 && (
                    <div
                      style={{
                        width: "2px",
                        flex: 1,
                        minHeight: "20px",
                        backgroundColor: "#d1d5db",
                      }}
                    />
                  )}
                </div>

                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: "6px",
                      marginBottom: "2px",
                    }}
                  >
                    <span
                      style={{
                        fontSize: "10px",
                        fontWeight: "bold",
                        padding: "1px 6px",
                        borderRadius: "3px",
                        backgroundColor: "#e5e7eb",
                        color: "#374151",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {EVENT_TYPE_LABELS[event.eventType]}
                    </span>
                    <span
                      style={{
                        fontSize: "12px",
                        fontWeight: 500,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                      title={event.name}
                    >
                      {event.name}
                    </span>
                  </div>

                  <div
                    style={{
                      display: "flex",
                      gap: "8px",
                      fontSize: "11px",
                      color: "#6b7280",
                    }}
                  >
                    {event.startTime && <span>{event.startTime}</span>}
                    {event.durationSecs != null && (
                      <span style={{ color: "#9ca3af" }}>
                        ({formatDuration(event.durationSecs)})
                      </span>
                    )}
                    {event.errorCode && (
                      <span
                        style={{
                          color: "#ef4444",
                          fontFamily: "'Courier New', monospace",
                        }}
                      >
                        {event.errorCode}
                      </span>
                    )}
                    <span
                      style={{
                        color: "#475569",
                        fontFamily: "'Courier New', monospace",
                      }}
                      title={event.sourceFile}
                    >
                      {formatSourceLabel(event.sourceFile, event.lineNumber)}
                    </span>
                  </div>

                  {isSelected && (
                    <div
                      style={{
                        fontSize: "11px",
                        color: "#4b5563",
                        marginTop: "4px",
                        fontFamily: "'Courier New', monospace",
                        whiteSpace: "pre-wrap",
                        wordBreak: "break-all",
                        maxHeight: "120px",
                        overflow: "auto",
                        backgroundColor: "#f9fafb",
                        padding: "4px 6px",
                        borderRadius: "2px",
                      }}
                    >
                      {event.detail}
                    </div>
                  )}
                </div>

                <div
                  style={{
                    fontSize: "10px",
                    fontWeight: "bold",
                    padding: "1px 6px",
                    borderRadius: "3px",
                    backgroundColor: STATUS_COLORS[event.status],
                    color: "#fff",
                    whiteSpace: "nowrap",
                    marginLeft: "8px",
                    marginTop: "2px",
                  }}
                >
                  {event.status}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function formatSourceLabel(sourceFile: string, lineNumber: number): string {
  const normalized = sourceFile.replace(/\\/g, "/");
  const segments = normalized.split("/");
  const fileName = segments[segments.length - 1] || sourceFile;
  return `${fileName}:${lineNumber}`;
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${Math.round(secs)}s`;
  const m = Math.floor(secs / 60);
  const s = Math.round(secs % 60);
  if (m < 60) return `${m}m ${s}s`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return `${h}h ${rm}m ${s}s`;
}
