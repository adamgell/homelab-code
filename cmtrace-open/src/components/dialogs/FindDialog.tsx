import { useState, useEffect, useCallback, useRef } from "react";
import { useLogStore } from "../../stores/log-store";

interface FindDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function FindDialog({ isOpen, onClose }: FindDialogProps) {
  const [searchText, setSearchText] = useState("");
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [statusText, setStatusText] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const entries = useLogStore((s) => s.entries);
  const selectedId = useLogStore((s) => s.selectedId);
  const selectEntry = useLogStore((s) => s.selectEntry);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isOpen]);

  const doFind = useCallback(
    (forward: boolean) => {
      if (!searchText || entries.length === 0) return;

      // Find current selected index
      let startIndex = -1;
      if (selectedId !== null) {
        startIndex = entries.findIndex((e) => e.id === selectedId);
      }

      const count = entries.length;
      const searchLower = caseSensitive ? searchText : searchText.toLowerCase();

      let current = forward
        ? (startIndex + 1) % count
        : startIndex <= 0
          ? count - 1
          : startIndex - 1;

      let wrapped = false;

      for (let i = 0; i < count; i++) {
        const entry = entries[current];
        const message = caseSensitive
          ? entry.message
          : entry.message.toLowerCase();

        if (message.includes(searchLower)) {
          selectEntry(entry.id);
          setStatusText(
            wrapped ? `Found (wrapped) at line ${entry.lineNumber}` : ""
          );
          return;
        }

        if (forward) {
          current = (current + 1) % count;
          if (current === 0) wrapped = true;
        } else {
          current = current <= 0 ? count - 1 : current - 1;
          if (current === count - 1) wrapped = true;
        }
      }

      setStatusText("Not found");
    },
    [searchText, caseSensitive, entries, selectedId, selectEntry]
  );

  // Handle keyboard shortcuts
  useEffect(() => {
    if (!isOpen) return;

    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      } else if (e.key === "Enter" || e.key === "F3") {
        e.preventDefault();
        doFind(!e.shiftKey);
      }
    };

    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [isOpen, onClose, doFind]);

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: "rgba(0,0,0,0.3)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: "80px",
        zIndex: 1000,
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        style={{
          backgroundColor: "#f0f0f0",
          border: "1px solid #999",
          borderRadius: "4px",
          padding: "12px",
          minWidth: "400px",
          boxShadow: "0 4px 12px rgba(0,0,0,0.3)",
        }}
      >
        <div
          style={{
            fontSize: "13px",
            fontWeight: "bold",
            marginBottom: "8px",
          }}
        >
          Find
        </div>

        <div style={{ display: "flex", gap: "6px", marginBottom: "8px" }}>
          <label style={{ fontSize: "12px", lineHeight: "24px" }}>
            Find what:
          </label>
          <input
            ref={inputRef}
            type="text"
            value={searchText}
            onChange={(e) => {
              setSearchText(e.target.value);
              setStatusText("");
            }}
            style={{
              flex: 1,
              fontSize: "12px",
              padding: "2px 4px",
            }}
          />
        </div>

        <div style={{ display: "flex", gap: "8px", alignItems: "center", marginBottom: "8px" }}>
          <label style={{ fontSize: "12px", display: "flex", alignItems: "center", gap: "4px" }}>
            <input
              type="checkbox"
              checked={caseSensitive}
              onChange={(e) => setCaseSensitive(e.target.checked)}
            />
            Match case
          </label>
          {statusText && (
            <span style={{ fontSize: "11px", color: "#666" }}>{statusText}</span>
          )}
        </div>

        <div style={{ display: "flex", gap: "6px", justifyContent: "flex-end" }}>
          <button onClick={() => doFind(false)}>Find Previous</button>
          <button onClick={() => doFind(true)}>Find Next</button>
          <button onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  );
}
