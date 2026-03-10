import { useEffect, useRef } from "react";
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
  const entries = useLogStore((s) => s.entries);
  const isPaused = useLogStore((s) => s.isPaused);
  const appendEntries = useLogStore((s) => s.appendEntries);
  const setTotalLines = useLogStore((s) => s.setTotalLines);

  // Store the byte offset and next-id refs to avoid stale closures
  const byteOffsetRef = useRef<number>(0);
  const nextIdRef = useRef<number>(0);
  const nextLineRef = useRef<number>(0);

  // Update refs when entries change (after initial load)
  useEffect(() => {
    if (entries.length > 0) {
      const lastEntry = entries[entries.length - 1];
      nextIdRef.current = lastEntry.id + 1;
      nextLineRef.current = lastEntry.lineNumber + 1;
    }
  }, [entries.length]);

  // Start/stop tailing when file changes
  useEffect(() => {
    if (!openFilePath || !formatDetected) return;

    // Get byte offset from the store (set after initial parse)
    const byteOffset = useLogStore.getState().byteOffset;
    byteOffsetRef.current = byteOffset;

    const currentEntries = useLogStore.getState().entries;
    const nextId = currentEntries.length > 0
      ? currentEntries[currentEntries.length - 1].id + 1
      : 0;
    const nextLine = currentEntries.length > 0
      ? currentEntries[currentEntries.length - 1].lineNumber + 1
      : 1;

    startTail(openFilePath, formatDetected, byteOffset, nextId, nextLine)
      .catch((err) => console.error("Failed to start tail:", err));

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
      const { entries: newEntries } = event.payload;
      if (newEntries.length > 0) {
        appendEntries(newEntries);
        // Update total lines count
        const state = useLogStore.getState();
        setTotalLines(state.totalLines + newEntries.length);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [appendEntries, setTotalLines]);
}
