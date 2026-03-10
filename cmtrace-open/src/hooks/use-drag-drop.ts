import { useEffect } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useLogStore } from "../stores/log-store";
import { useFilterStore } from "../stores/filter-store";
import { openLogFile, stopTail } from "../lib/commands";

/**
 * Hook that handles file drag-and-drop onto the application window.
 * When a .log or .lo_ file is dropped, it opens it automatically.
 */
export function useDragDrop() {
  const setEntries = useLogStore((s) => s.setEntries);
  const setLoading = useLogStore((s) => s.setLoading);
  const setFormatDetected = useLogStore((s) => s.setFormatDetected);
  const setTotalLines = useLogStore((s) => s.setTotalLines);
  const openFilePath = useLogStore((s) => s.openFilePath);
  const setOpenFilePath = useLogStore((s) => s.setOpenFilePath);
  const setByteOffset = useLogStore((s) => s.setByteOffset);
  const clearFilter = useFilterStore((s) => s.clearFilter);

  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();

    const unlisten = appWindow.onDragDropEvent(async (event) => {
      if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        if (paths.length > 0) {
          const filePath = paths[0];

          // Stop tailing previous file
          if (openFilePath) {
            await stopTail(openFilePath).catch(() => {});
          }

          clearFilter();
          setLoading(true);

          try {
            const result = await openLogFile(filePath);
            setEntries(result.entries);
            setFormatDetected(result.formatDetected);
            setTotalLines(result.totalLines);
            setOpenFilePath(result.filePath);
            setByteOffset(result.byteOffset);
          } catch (err) {
            console.error("Failed to open dropped file:", err);
          } finally {
            setLoading(false);
          }
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [
    openFilePath,
    setEntries,
    setLoading,
    setFormatDetected,
    setTotalLines,
    setOpenFilePath,
    setByteOffset,
    clearFilter,
  ]);
}
