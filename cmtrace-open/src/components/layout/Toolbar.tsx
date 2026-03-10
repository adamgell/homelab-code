import { useLogStore } from "../../stores/log-store";
import { useUiStore } from "../../stores/ui-store";
import { useFilterStore } from "../../stores/filter-store";
import { open } from "@tauri-apps/plugin-dialog";
import { openLogFile, stopTail } from "../../lib/commands";

export function Toolbar() {
  const setEntries = useLogStore((s) => s.setEntries);
  const setLoading = useLogStore((s) => s.setLoading);
  const setFormatDetected = useLogStore((s) => s.setFormatDetected);
  const setTotalLines = useLogStore((s) => s.setTotalLines);
  const openFilePath = useLogStore((s) => s.openFilePath);
  const setOpenFilePath = useLogStore((s) => s.setOpenFilePath);
  const setByteOffset = useLogStore((s) => s.setByteOffset);
  const highlightText = useLogStore((s) => s.highlightText);
  const setHighlightText = useLogStore((s) => s.setHighlightText);
  const isPaused = useLogStore((s) => s.isPaused);
  const togglePause = useLogStore((s) => s.togglePause);
  const toggleDetails = useUiStore((s) => s.toggleDetails);
  const toggleInfoPane = useUiStore((s) => s.toggleInfoPane);
  const setShowFindDialog = useUiStore((s) => s.setShowFindDialog);
  const setShowFilterDialog = useUiStore((s) => s.setShowFilterDialog);
  const setShowErrorLookupDialog = useUiStore(
    (s) => s.setShowErrorLookupDialog
  );
  const activeView = useUiStore((s) => s.activeView);
  const setActiveView = useUiStore((s) => s.setActiveView);
  const filterClauses = useFilterStore((s) => s.clauses);
  const clearFilter = useFilterStore((s) => s.clearFilter);

  const handleOpen = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Log Files", extensions: ["log"] },
        { name: "Old Log Files", extensions: ["lo_"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    if (selected) {
      // Stop tailing the previous file
      if (openFilePath) {
        await stopTail(openFilePath).catch(() => {});
      }

      // Clear any active filter
      clearFilter();

      setLoading(true);
      try {
        const result = await openLogFile(selected);
        setEntries(result.entries);
        setFormatDetected(result.formatDetected);
        setTotalLines(result.totalLines);
        setOpenFilePath(result.filePath);
        setByteOffset(result.byteOffset);
      } catch (err) {
        console.error("Failed to open file:", err);
      } finally {
        setLoading(false);
      }
    }
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "6px",
        padding: "4px 8px",
        backgroundColor: "#f0f0f0",
        borderBottom: "1px solid #c0c0c0",
        flexShrink: 0,
      }}
    >
      <button onClick={handleOpen} title="Open (Ctrl+O)">
        Open
      </button>
      <button
        onClick={togglePause}
        title="Pause (Ctrl+U)"
        style={{
          backgroundColor: isPaused ? "#ffcccc" : undefined,
        }}
      >
        {isPaused ? "Resume" : "Pause"}
      </button>

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <button
        onClick={() => setShowFindDialog(true)}
        title="Find (Ctrl+F)"
      >
        Find
      </button>
      <button
        onClick={() => setShowFilterDialog(true)}
        title="Filter (Ctrl+L)"
        style={{
          backgroundColor: filterClauses.length > 0 ? "#cce5ff" : undefined,
        }}
      >
        {filterClauses.length > 0
          ? `Filter (${filterClauses.length})`
          : "Filter"}
      </button>

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <label
        style={{
          fontSize: "12px",
          fontFamily: "'Segoe UI', Tahoma, sans-serif",
        }}
      >
        Highlight:
      </label>
      <input
        type="text"
        value={highlightText}
        onChange={(e) => setHighlightText(e.target.value)}
        placeholder="Enter text to highlight..."
        style={{
          width: "200px",
          fontSize: "12px",
          padding: "2px 4px",
        }}
      />

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <button
        onClick={() => setShowErrorLookupDialog(true)}
        title="Error Lookup (Ctrl+E)"
      >
        Error Lookup
      </button>

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <button onClick={toggleDetails} title="Show/Hide Details (Ctrl+H)">
        Details
      </button>
      <button onClick={toggleInfoPane} title="Toggle Info Pane">
        Info
      </button>

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <button
        onClick={() =>
          setActiveView(activeView === "log" ? "intune" : "log")
        }
        title="Toggle Intune Diagnostics"
        style={{
          backgroundColor: activeView === "intune" ? "#dbeafe" : undefined,
          fontWeight: activeView === "intune" ? 600 : undefined,
        }}
      >
        {activeView === "intune" ? "← Log View" : "Intune"}
      </button>
    </div>
  );
}
