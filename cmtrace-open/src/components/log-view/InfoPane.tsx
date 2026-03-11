import { useLogStore } from "../../stores/log-store";

export function InfoPane() {
  const entries = useLogStore((state) => state.entries);
  const selectedId = useLogStore((state) => state.selectedId);

  const selectedEntry =
    selectedId !== null
      ? entries.find((entry) => entry.id === selectedId) ?? null
      : null;

  if (!selectedEntry) {
    return (
      <div
        style={{
          padding: "8px",
          fontFamily: "'Courier New', monospace",
          fontSize: "13px",
          color: "#888",
          height: "100%",
          overflow: "auto",
          backgroundColor: "#fafafa",
          borderTop: "2px solid #c0c0c0",
        }}
      >
        {entries.length === 0
          ? "No log entries loaded"
          : "Select a log entry to view details (Arrow keys, Page Up/Down, Home/End supported when list is focused)"}
      </div>
    );
  }

  return (
    <div
      style={{
        padding: "8px",
        fontFamily: "'Courier New', monospace",
        fontSize: "13px",
        height: "100%",
        overflow: "auto",
        backgroundColor: "#fafafa",
        borderTop: "2px solid #c0c0c0",
      }}
    >
      <div style={{ marginBottom: "8px", color: "#444" }}>
        {`Line ${selectedEntry.lineNumber} | ${selectedEntry.severity}${
          selectedEntry.component ? ` | ${selectedEntry.component}` : ""
        }${selectedEntry.timestampDisplay ? ` | ${selectedEntry.timestampDisplay}` : ""}`}
      </div>
      <div
        style={{
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
          color: "#111",
        }}
      >
        {selectedEntry.message}
      </div>
    </div>
  );
}
