import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Toolbar } from "./Toolbar";
import { StatusBar } from "./StatusBar";
import { LogListView } from "../log-view/LogListView";
import { InfoPane } from "../log-view/InfoPane";
import { FindDialog } from "../dialogs/FindDialog";
import { FilterDialog } from "../dialogs/FilterDialog";
import { ErrorLookupDialog } from "../dialogs/ErrorLookupDialog";
import { AboutDialog } from "../dialogs/AboutDialog";
import { IntuneDashboard } from "../intune/IntuneDashboard";
import type { FilterClause } from "../dialogs/FilterDialog";
import { useUiStore } from "../../stores/ui-store";
import { useLogStore } from "../../stores/log-store";
import { useFilterStore } from "../../stores/filter-store";
import { useFileWatcher } from "../../hooks/use-file-watcher";
import { useKeyboard } from "../../hooks/use-keyboard";
import { useDragDrop } from "../../hooks/use-drag-drop";

export function AppShell() {
  const activeView = useUiStore((s) => s.activeView);
  const showInfoPane = useUiStore((s) => s.showInfoPane);
  const infoPaneHeight = useUiStore((s) => s.infoPaneHeight);
  const showFindDialog = useUiStore((s) => s.showFindDialog);
  const showFilterDialog = useUiStore((s) => s.showFilterDialog);
  const showErrorLookupDialog = useUiStore((s) => s.showErrorLookupDialog);
  const showAboutDialog = useUiStore((s) => s.showAboutDialog);
  const setShowFindDialog = useUiStore((s) => s.setShowFindDialog);
  const setShowFilterDialog = useUiStore((s) => s.setShowFilterDialog);
  const setShowErrorLookupDialog = useUiStore(
    (s) => s.setShowErrorLookupDialog
  );
  const setShowAboutDialog = useUiStore((s) => s.setShowAboutDialog);

  const entries = useLogStore((s) => s.entries);
  const filterClauses = useFilterStore((s) => s.clauses);
  const setClauses = useFilterStore((s) => s.setClauses);
  const setFilteredIds = useFilterStore((s) => s.setFilteredIds);

  // Start file tailing when a file is opened
  useFileWatcher();
  // Register keyboard shortcuts
  useKeyboard();
  // Handle file drag-and-drop
  useDragDrop();

  const handleApplyFilter = useCallback(
    async (clauses: FilterClause[]) => {
      setClauses(clauses);

      if (clauses.length === 0) {
        setFilteredIds(null);
        return;
      }

      try {
        const ids = await invoke<number[]>("apply_filter", {
          entries,
          clauses,
        });
        setFilteredIds(new Set(ids));
      } catch (err) {
        console.error("Failed to apply filter:", err);
      }
    },
    [entries, setClauses, setFilteredIds]
  );

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        overflow: "hidden",
        backgroundColor: "#ffffff",
      }}
    >
      <Toolbar />

      {activeView === "log" ? (
        <>
          <div
            style={{
              flex: 1,
              display: "flex",
              flexDirection: "column",
              overflow: "hidden",
            }}
          >
            <div
              style={{
                flex: 1,
                overflow: "hidden",
              }}
            >
              <LogListView />
            </div>

            {showInfoPane && (
              <div
                style={{
                  height: `${infoPaneHeight}px`,
                  flexShrink: 0,
                  overflow: "hidden",
                }}
              >
                <InfoPane />
              </div>
            )}
          </div>

          <StatusBar />
        </>
      ) : (
        <div style={{ flex: 1, overflow: "hidden" }}>
          <IntuneDashboard />
        </div>
      )}

      {/* Dialogs */}
      <FindDialog
        isOpen={showFindDialog}
        onClose={() => setShowFindDialog(false)}
      />
      <FilterDialog
        isOpen={showFilterDialog}
        onClose={() => setShowFilterDialog(false)}
        onApply={handleApplyFilter}
        currentClauses={filterClauses}
      />
      <ErrorLookupDialog
        isOpen={showErrorLookupDialog}
        onClose={() => setShowErrorLookupDialog(false)}
      />
      <AboutDialog
        isOpen={showAboutDialog}
        onClose={() => setShowAboutDialog(false)}
      />
    </div>
  );
}
