import { useRef, useEffect, useCallback, useMemo } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useLogStore } from "../../stores/log-store";
import { useUiStore } from "../../stores/ui-store";
import { useFilterStore } from "../../stores/filter-store";
import { LogRow } from "./LogRow";
import { COLUMN_NAMES } from "../../lib/constants";

export function LogListView() {
  const entries = useLogStore((s) => s.entries);
  const selectedId = useLogStore((s) => s.selectedId);
  const selectEntry = useLogStore((s) => s.selectEntry);
  const highlightText = useLogStore((s) => s.highlightText);
  const highlightCaseSensitive = useLogStore((s) => s.highlightCaseSensitive);
  const isPaused = useLogStore((s) => s.isPaused);
  const showDetails = useUiStore((s) => s.showDetails);
  const filteredIds = useFilterStore((s) => s.filteredIds);

  // Apply filter to entries
  const displayEntries = useMemo(() => {
    if (!filteredIds) return entries;
    return entries.filter((e) => filteredIds.has(e.id));
  }, [entries, filteredIds]);

  const parentRef = useRef<HTMLDivElement>(null);
  // Track whether the user is scrolled near the bottom (auto-scroll zone)
  const isAtBottomRef = useRef(true);

  const virtualizer = useVirtualizer({
    count: displayEntries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 22,
    overscan: 20,
  });

  // Track scroll position to decide whether to auto-scroll
  const handleScroll = useCallback(() => {
    const el = parentRef.current;
    if (!el) return;
    const threshold = 50; // px from bottom
    isAtBottomRef.current =
      el.scrollHeight - el.scrollTop - el.clientHeight < threshold;
  }, []);

  // Auto-scroll to bottom when new entries arrive (only if at bottom and not paused)
  const prevCount = useRef(displayEntries.length);
  useEffect(() => {
    if (
      displayEntries.length > prevCount.current &&
      displayEntries.length > 0 &&
      isAtBottomRef.current &&
      !isPaused
    ) {
      virtualizer.scrollToIndex(displayEntries.length - 1, { align: "end" });
    }
    prevCount.current = displayEntries.length;
  }, [displayEntries.length, virtualizer, isPaused]);

  // Scroll to selected entry when it changes (e.g., from Find)
  useEffect(() => {
    if (selectedId === null) return;
    const index = displayEntries.findIndex((e) => e.id === selectedId);
    if (index >= 0) {
      virtualizer.scrollToIndex(index, { align: "center" });
    }
  }, [selectedId, displayEntries, virtualizer]);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        overflow: "hidden",
      }}
    >
      {/* Column headers */}
      <div
        style={{
          display: "flex",
          backgroundColor: "#f0f0f0",
          borderBottom: "2px solid #c0c0c0",
          fontSize: "13px",
          fontWeight: "bold",
          fontFamily: "'Segoe UI', Tahoma, sans-serif",
          lineHeight: "24px",
          whiteSpace: "nowrap",
          flexShrink: 0,
        }}
      >
        <div
          style={{
            flex: showDetails ? 3 : 1,
            padding: "1px 4px",
          }}
        >
          {COLUMN_NAMES.logText}
        </div>
        {showDetails && (
          <>
            <div
              style={{
                width: "180px",
                minWidth: "180px",
                padding: "1px 4px",
                borderLeft: "1px solid #c0c0c0",
              }}
            >
              {COLUMN_NAMES.component}
            </div>
            <div
              style={{
                width: "200px",
                minWidth: "200px",
                padding: "1px 4px",
                borderLeft: "1px solid #c0c0c0",
              }}
            >
              {COLUMN_NAMES.dateTime}
            </div>
            <div
              style={{
                width: "120px",
                minWidth: "120px",
                padding: "1px 4px",
                borderLeft: "1px solid #c0c0c0",
              }}
            >
              {COLUMN_NAMES.thread}
            </div>
          </>
        )}
      </div>

      {/* Virtualized list */}
      <div
        ref={parentRef}
        onScroll={handleScroll}
        style={{
          flex: 1,
          overflow: "auto",
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
            const entry = displayEntries[virtualRow.index];
            return (
              <div
                key={entry.id}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                <LogRow
                  entry={entry}
                  isSelected={entry.id === selectedId}
                  showDetails={showDetails}
                  highlightText={highlightText}
                  highlightCaseSensitive={highlightCaseSensitive}
                  onClick={selectEntry}
                />
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
