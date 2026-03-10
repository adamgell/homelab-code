import { useMemo } from "react";
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
      if (filterEventType !== "All" && e.eventType !== filterEventType)
        return false;
      if (filterStatus !== "All" && e.status !== filterStatus) return false;
      return true;
    });
  }, [events, filterEventType, filterStatus]);

  if (filteredEvents.length === 0) {
    return (
      <div style={{ padding: "20px", color: "#666", textAlign: "center" }}>
        No events match the current filters.
      </div>
    );
  }

  return (
    <div
      style={{
        overflowY: "auto",
        maxHeight: "100%",
        padding: "8px 0",
      }}
    >
      {filteredEvents.map((event, index) => (
        <div
          key={event.id}
          onClick={() => selectEvent(event.id)}
          style={{
            display: "flex",
            alignItems: "flex-start",
            padding: "6px 12px",
            cursor: "pointer",
            backgroundColor:
              selectedEventId === event.id ? "#e0e7ff" : "transparent",
            borderLeft: `3px solid ${STATUS_COLORS[event.status]}`,
            marginBottom: "2px",
          }}
        >
          {/* Timeline connector */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              marginRight: "10px",
              minWidth: "20px",
            }}
          >
            {/* Dot */}
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
            {/* Line */}
            {index < filteredEvents.length - 1 && (
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

          {/* Content */}
          <div style={{ flex: 1, minWidth: 0 }}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "6px",
                marginBottom: "2px",
              }}
            >
              {/* Type badge */}
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
              {/* Name */}
              <span
                style={{
                  fontSize: "12px",
                  fontWeight: 500,
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                }}
              >
                {event.name}
              </span>
            </div>

            {/* Timestamp + duration */}
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
            </div>

            {/* Detail (truncated) */}
            {selectedEventId === event.id && (
              <div
                style={{
                  fontSize: "11px",
                  color: "#4b5563",
                  marginTop: "4px",
                  fontFamily: "'Courier New', monospace",
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-all",
                  maxHeight: "100px",
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

          {/* Status badge */}
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
      ))}
    </div>
  );
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
