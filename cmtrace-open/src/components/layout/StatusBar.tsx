import { useLogStore } from "../../stores/log-store";

export function StatusBar() {
  const entries = useLogStore((s) => s.entries);
  const totalLines = useLogStore((s) => s.totalLines);
  const formatDetected = useLogStore((s) => s.formatDetected);
  const openFilePath = useLogStore((s) => s.openFilePath);
  const selectedId = useLogStore((s) => s.selectedId);
  const isLoading = useLogStore((s) => s.isLoading);

  // Calculate elapsed time matching CMTrace format:
  // "Elapsed time is %luh %lum %lus %lums (%lu.%03lu seconds)"
  let elapsedText = "";
  if (selectedId !== null && entries.length > 0) {
    const firstEntry = entries[0];
    const selectedEntry = entries.find((e) => e.id === selectedId);
    if (firstEntry?.timestamp && selectedEntry?.timestamp) {
      const diffMs = Math.abs(selectedEntry.timestamp - firstEntry.timestamp);
      const totalSeconds = Math.floor(diffMs / 1000);
      const ms = diffMs % 1000;
      const hours = Math.floor(totalSeconds / 3600);
      const minutes = Math.floor((totalSeconds % 3600) / 60);
      const seconds = totalSeconds % 60;
      elapsedText = `Elapsed time is ${hours}h ${minutes}m ${seconds}s ${ms}ms (${totalSeconds}.${String(ms).padStart(3, "0")} seconds)`;
    }
  }

  const fileName = openFilePath
    ? openFilePath.split(/[/\\]/).pop() ?? ""
    : "";

  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "2px 8px",
        backgroundColor: "#f0f0f0",
        borderTop: "1px solid #c0c0c0",
        fontSize: "12px",
        fontFamily: "'Segoe UI', Tahoma, sans-serif",
        flexShrink: 0,
        height: "22px",
      }}
    >
      <span>
        {isLoading
          ? "Loading..."
          : elapsedText || (fileName ? `${fileName}` : "Ready")}
      </span>
      <span>
        {entries.length > 0
          ? `${entries.length} entries | ${totalLines} lines | ${formatDetected ?? "Unknown"} format`
          : ""}
      </span>
    </div>
  );
}
