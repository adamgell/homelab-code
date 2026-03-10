import type { DownloadStat } from "../../types/intune";

interface DownloadStatsProps {
  downloads: DownloadStat[];
}

export function DownloadStats({ downloads }: DownloadStatsProps) {
  if (downloads.length === 0) {
    return (
      <div style={{ padding: "20px", color: "#666", textAlign: "center" }}>
        No download events found in the log.
      </div>
    );
  }

  return (
    <div style={{ overflowX: "auto" }}>
      <table
        style={{
          width: "100%",
          borderCollapse: "collapse",
          fontSize: "12px",
          fontFamily: "'Segoe UI', Tahoma, sans-serif",
        }}
      >
        <thead>
          <tr
            style={{
              backgroundColor: "#f3f4f6",
              borderBottom: "2px solid #d1d5db",
            }}
          >
            <th style={thStyle}>Content</th>
            <th style={thStyle}>Size</th>
            <th style={thStyle}>Speed</th>
            <th style={thStyle}>DO %</th>
            <th style={thStyle}>Duration</th>
            <th style={thStyle}>Status</th>
            <th style={thStyle}>Timestamp</th>
          </tr>
        </thead>
        <tbody>
          {downloads.map((dl, i) => (
            <tr
              key={i}
              style={{
                borderBottom: "1px solid #e5e7eb",
                backgroundColor: i % 2 === 0 ? "#fff" : "#f9fafb",
              }}
            >
              <td style={tdStyle} title={dl.contentId}>
                {dl.name}
              </td>
              <td style={{ ...tdStyle, fontFamily: "'Courier New', monospace" }}>
                {formatBytes(dl.sizeBytes)}
              </td>
              <td style={{ ...tdStyle, fontFamily: "'Courier New', monospace" }}>
                {dl.speedBps > 0 ? `${formatBytes(dl.speedBps)}/s` : "—"}
              </td>
              <td style={tdStyle}>
                {dl.doPercentage > 0 ? (
                  <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
                    <div
                      style={{
                        width: "40px",
                        height: "8px",
                        backgroundColor: "#e5e7eb",
                        borderRadius: "4px",
                        overflow: "hidden",
                      }}
                    >
                      <div
                        style={{
                          width: `${Math.min(dl.doPercentage, 100)}%`,
                          height: "100%",
                          backgroundColor:
                            dl.doPercentage > 50 ? "#22c55e" : "#3b82f6",
                          borderRadius: "4px",
                        }}
                      />
                    </div>
                    <span>{dl.doPercentage.toFixed(1)}%</span>
                  </div>
                ) : (
                  "—"
                )}
              </td>
              <td style={{ ...tdStyle, fontFamily: "'Courier New', monospace" }}>
                {dl.durationSecs > 0
                  ? `${dl.durationSecs.toFixed(1)}s`
                  : "—"}
              </td>
              <td style={tdStyle}>
                <span
                  style={{
                    display: "inline-block",
                    padding: "1px 6px",
                    borderRadius: "3px",
                    fontSize: "10px",
                    fontWeight: "bold",
                    backgroundColor: dl.success ? "#dcfce7" : "#fee2e2",
                    color: dl.success ? "#166534" : "#991b1b",
                  }}
                >
                  {dl.success ? "OK" : "FAIL"}
                </span>
              </td>
              <td style={{ ...tdStyle, color: "#6b7280" }}>
                {dl.timestamp || "—"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

const thStyle: React.CSSProperties = {
  padding: "6px 8px",
  textAlign: "left",
  fontWeight: 600,
  whiteSpace: "nowrap",
};

const tdStyle: React.CSSProperties = {
  padding: "5px 8px",
  whiteSpace: "nowrap",
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i > 0 ? 1 : 0)} ${units[Math.min(i, units.length - 1)]}`;
}
