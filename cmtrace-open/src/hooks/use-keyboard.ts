import { useEffect } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useLogStore } from "../stores/log-store";
import { useUiStore } from "../stores/ui-store";

/**
 * Keyboard shortcut handler matching CMTrace's accelerator table.
 * From REVERSE_ENGINEERING.md:
 *   Ctrl+O  → Open
 *   Ctrl+F  → Find
 *   F3      → Find Next
 *   Shift+F3→ Find Previous
 *   Ctrl+U  → Pause/Resume
 *   Ctrl+H  → Toggle Details
 *   Ctrl+L  → Filter
 *   Ctrl+C  → Copy (tab-separated selected entry)
 *   Ctrl+E  → Error Lookup
 *   F5      → Refresh
 */
export function useKeyboard() {
  const togglePause = useLogStore((s) => s.togglePause);
  const toggleDetails = useUiStore((s) => s.toggleDetails);
  const setShowFindDialog = useUiStore((s) => s.setShowFindDialog);
  const setShowFilterDialog = useUiStore((s) => s.setShowFilterDialog);
  const setShowErrorLookupDialog = useUiStore(
    (s) => s.setShowErrorLookupDialog
  );

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey || e.metaKey;

      // Ignore keyboard shortcuts when typing in an input
      const target = e.target as HTMLElement;
      const isInput =
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.tagName === "SELECT";

      if (ctrl && e.key === "o") {
        e.preventDefault();
        const openBtn = document.querySelector(
          '[title="Open (Ctrl+O)"]'
        ) as HTMLButtonElement;
        openBtn?.click();
      } else if (ctrl && e.key === "f") {
        e.preventDefault();
        setShowFindDialog(true);
      } else if (e.key === "F3" && !isInput) {
        e.preventDefault();
        setShowFindDialog(true);
      } else if (ctrl && e.key === "u") {
        e.preventDefault();
        togglePause();
      } else if (ctrl && e.key === "h") {
        e.preventDefault();
        toggleDetails();
      } else if (ctrl && e.key === "l") {
        e.preventDefault();
        setShowFilterDialog(true);
      } else if (ctrl && e.key === "e") {
        e.preventDefault();
        setShowErrorLookupDialog(true);
      } else if (ctrl && e.key === "c" && !isInput) {
        // Ctrl+C: Copy selected entry as tab-separated text
        e.preventDefault();
        const state = useLogStore.getState();
        if (state.selectedId !== null) {
          const entry = state.entries.find((e) => e.id === state.selectedId);
          if (entry) {
            const text = [
              entry.message,
              entry.component ?? "",
              entry.timestampDisplay ?? "",
              entry.threadDisplay ?? "",
            ].join("\t");
            try {
              await writeText(text);
            } catch (err) {
              console.error("Failed to copy:", err);
            }
          }
        }
      } else if (e.key === "Escape" && !isInput) {
        setShowFindDialog(false);
        setShowFilterDialog(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    togglePause,
    toggleDetails,
    setShowFindDialog,
    setShowFilterDialog,
    setShowErrorLookupDialog,
  ]);
}
