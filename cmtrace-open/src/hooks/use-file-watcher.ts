import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useLogStore } from "../stores/log-store";
import { startTail, stopTail, pauseTail, resumeTail } from "../lib/commands";
import type { TailPayload } from "../types/log";

/**
 * Hook that manages the file-tail lifecycle:
 * - Starts tailing after a file is opened
 * - Appends new entries as they arrive via Tauri events
 * - Handles pause/resume
 * - Cleans up on unmount or file change
 */
export function useFileWatcher() {
  const openFilePath = useLogStore((s) => s.openFilePath);
  const formatDetected = useLogStore((s) => s.formatDetected);
  const isPaused = useLogStore((s) => s.isPaused);
  const appendEntries = useLogStore((s) => s.appendEntries);

  // Start/stop tailing when file changes
  useEffect(() => {
    if (!openFilePath || !formatDetected) return;

    // Get byte offset from the store (set after initial parse)
    const byteOffset = useLogStore.getState().byteOffset;

    const currentEntries = useLogStore.getState().entries;
    const nextId =
      currentEntries.length > 0
        ? currentEntries[currentEntries.length - 1].id + 1
        : 0;
    const nextLine =
      currentEntries.length > 0
        ? currentEntries[currentEntries.length - 1].lineNumber + 1
        : 1;

    startTail(openFilePath, formatDetected, byteOffset, nextId, nextLine).catch(
      (err) => console.error("Failed to start tail:", err)
    );

    return () => {
      stopTail(openFilePath).catch((err) =>
        console.error("Failed to stop tail:", err)
      );
    };
  }, [openFilePath, formatDetected]);

  // Handle pause/resume
  useEffect(() => {
    if (!openFilePath) return;

    if (isPaused) {
      pauseTail(openFilePath).catch((err) =>
        console.error("Failed to pause tail:", err)
      );
    } else {
      resumeTail(openFilePath).catch((err) =>
        console.error("Failed to resume tail:", err)
      );
    }
  }, [isPaused, openFilePath]);

  // Listen for new tail entries from the Rust backend
  useEffect(() => {
    const unlisten = listen<TailPayload>("tail-new-entries", (event) => {
      const { entries: newEntries, filePath } = event.payload;
      const currentPath = useLogStore.getState().openFilePath;

      if (!currentPath || currentPath !== filePath || newEntries.length === 0) {
        return;
      }

      appendEntries(newEntries);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [appendEntries]);
}