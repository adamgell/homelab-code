import { useLogStore } from "../../stores/log-store";

export function InfoPane() {
  const entries = useLogStore((s) => s.entries);
  const selectedId = useLogStore((s) => s.selectedId);

  const selectedEntry = selectedId !== null
    ? entries.find((e) => e.id === selectedId)
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
        Select a log entry to view details
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
        whiteSpace: "pre-wrap",
        wordBreak: "break-word",
      }}
    >
      {selectedEntry.message}
    </div>
  );
}
