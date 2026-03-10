import { useEffect } from "react";

interface AboutDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AboutDialog({ isOpen, onClose }: AboutDialogProps) {
  useEffect(() => {
    if (!isOpen) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [isOpen, onClose]);

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
        alignItems: "center",
        justifyContent: "center",
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
          padding: "20px",
          minWidth: "350px",
          textAlign: "center",
          boxShadow: "0 4px 12px rgba(0,0,0,0.3)",
        }}
      >
        <div
          style={{
            fontSize: "16px",
            fontWeight: "bold",
            marginBottom: "8px",
          }}
        >
          CMTrace Open
        </div>
        <div style={{ fontSize: "12px", color: "#666", marginBottom: "12px" }}>
          Version 0.1.0
        </div>
        <div style={{ fontSize: "12px", marginBottom: "16px" }}>
          Open-source CMTrace log viewer
          <br />
          with built-in Intune diagnostics
        </div>
        <div
          style={{
            fontSize: "11px",
            color: "#888",
            marginBottom: "16px",
          }}
        >
          Built with Tauri + React + TypeScript + Rust
        </div>
        <button onClick={onClose}>OK</button>
      </div>
    </div>
  );
}
