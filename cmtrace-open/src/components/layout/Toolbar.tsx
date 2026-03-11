import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type CSSProperties,
  type ChangeEvent,
} from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  getStreamStateSnapshot,
  useLogStore,
} from "../../stores/log-store";
import { useUiStore } from "../../stores/ui-store";
import { useFilterStore } from "../../stores/filter-store";
import {
  getKnownSourceMetadataById,
  loadLogSource,
  refreshKnownLogSources,
  resolveKnownSourceIdFromCatalogAction,
  type KnownSourceCatalogActionIds,
} from "../../lib/log-source";
import type { LogSource } from "../../types/log";

function normalizeDialogSelection(
  selected: string | string[] | null
): string | null {
  if (!selected) {
    return null;
  }

  return Array.isArray(selected) ? selected[0] ?? null : selected;
}

function resolveRefreshSource(
  activeSource: LogSource | null,
  openFilePath: string | null
): LogSource | null {
  if (activeSource) {
    return activeSource;
  }

  if (openFilePath) {
    return { kind: "file", path: openFilePath };
  }

  return null;
}

export interface OpenKnownSourceCatalogAction
  extends KnownSourceCatalogActionIds {
  trigger: string;
}

export interface AppCommandState {
  canOpenSources: boolean;
  canPauseResume: boolean;
  canFind: boolean;
  canFilter: boolean;
  canRefresh: boolean;
  canToggleDetailsPane: boolean;
  canToggleInfoPane: boolean;
  isLoading: boolean;
  isPaused: boolean;
  hasActiveSource: boolean;
  isDetailsVisible: boolean;
  isInfoPaneVisible: boolean;
  activeFilterCount: number;
  isFiltering: boolean;
  filterError: string | null;
  activeView: "log" | "intune";
}

export interface AppActionHandlers {
  commandState: AppCommandState;
  openLogFileDialog: () => Promise<void>;
  openLogFolderDialog: () => Promise<void>;
  openKnownSourceCatalogAction: (
    action: OpenKnownSourceCatalogAction
  ) => Promise<void>;
  openKnownSourceById: (sourceId: string, trigger: string) => Promise<void>;
  openKnownSourcePresetByMenuId: (presetMenuId: string) => Promise<void>;
  showFindDialog: () => void;
  showFilterDialog: () => void;
  showErrorLookupDialog: () => void;
  showAboutDialog: () => void;
  togglePauseResume: () => void;
  refreshActiveSource: () => Promise<void>;
  toggleDetailsPane: () => void;
  toggleInfoPane: () => void;
  dismissTransientDialogs: (trigger: string) => void;
}

function getToolbarControlStyle(options: {
  disabled: boolean;
  active?: boolean;
  tone?: "neutral" | "busy" | "warning" | "error";
}): CSSProperties {
  const { disabled, active = false, tone = "neutral" } = options;

  const toneColors: Record<string, string> = {
    neutral: active ? "#dbeafe" : "#ffffff",
    busy: "#fef3c7",
    warning: "#ffedd5",
    error: "#fecaca",
  };

  return {
    border: "1px solid #9ca3af",
    borderRadius: "2px",
    backgroundColor: disabled ? "#e5e7eb" : toneColors[tone],
    color: disabled ? "#6b7280" : "#111827",
    fontWeight: active ? 600 : 400,
    opacity: disabled ? 0.75 : 1,
    cursor: disabled ? "not-allowed" : "pointer",
  };
}

export function useAppActions(): AppActionHandlers {
  const isLoading = useLogStore((s) => s.isLoading);
  const isPaused = useLogStore((s) => s.isPaused);
  const entriesCount = useLogStore((s) => s.entries.length);
  const activeSource = useLogStore((s) => s.activeSource);
  const openFilePath = useLogStore((s) => s.openFilePath);
  const selectedSourceFilePath = useLogStore((s) => s.selectedSourceFilePath);

  const activeView = useUiStore((s) => s.activeView);
  const showDetails = useUiStore((s) => s.showDetails);
  const showInfoPane = useUiStore((s) => s.showInfoPane);
  const setShowFindDialog = useUiStore((s) => s.setShowFindDialog);
  const setShowFilterDialog = useUiStore((s) => s.setShowFilterDialog);
  const setShowErrorLookupDialog = useUiStore(
    (s) => s.setShowErrorLookupDialog
  );
  const setShowAboutDialog = useUiStore((s) => s.setShowAboutDialog);

  const activeFilterCount = useFilterStore((s) => s.clauses.length);
  const isFiltering = useFilterStore((s) => s.isFiltering);
  const filterError = useFilterStore((s) => s.filterError);

  const refreshSource = useMemo(
    () => resolveRefreshSource(activeSource, openFilePath),
    [activeSource, openFilePath]
  );

  const commandState = useMemo<AppCommandState>(
    () => ({
      canOpenSources: !isLoading,
      canPauseResume: !isLoading && refreshSource !== null,
      canFind: entriesCount > 0,
      canFilter: entriesCount > 0 && !isFiltering,
      canRefresh: !isLoading && refreshSource !== null,
      canToggleDetailsPane: activeView === "log",
      canToggleInfoPane: activeView === "log",
      isLoading,
      isPaused,
      hasActiveSource: refreshSource !== null,
      isDetailsVisible: showDetails,
      isInfoPaneVisible: showInfoPane,
      activeFilterCount,
      isFiltering,
      filterError,
      activeView,
    }),
    [
      activeFilterCount,
      activeView,
      entriesCount,
      filterError,
      isFiltering,
      isLoading,
      isPaused,
      refreshSource,
      showDetails,
      showInfoPane,
    ]
  );

  const loadSource = useCallback(async (source: LogSource, trigger: string) => {
    useUiStore.getState().ensureLogViewVisible(trigger);
    useFilterStore.getState().clearFilter();

    try {
      await loadLogSource(source);
    } catch (error) {
      console.error("[app-actions] failed to load source", {
        source,
        trigger,
        error,
      });
    }
  }, []);

  const openKnownSourceCatalogAction = useCallback(
    async (action: OpenKnownSourceCatalogAction) => {
      const sourceId = resolveKnownSourceIdFromCatalogAction(action);

      if (!sourceId) {
        console.warn("[app-actions] could not resolve known source for action", {
          action,
        });
        return;
      }

      const metadata = await getKnownSourceMetadataById(sourceId);

      if (!metadata) {
        throw new Error(
          `[app-actions] known source metadata was not found for id '${sourceId}'`
        );
      }

      await loadSource(metadata.source, action.trigger);
    },
    [loadSource]
  );

  const openLogFileDialog = useCallback(async () => {
    if (!commandState.canOpenSources) {
      return;
    }

    const selected = await open({
      multiple: false,
      filters: [
        { name: "Log Files", extensions: ["log"] },
        { name: "Old Log Files", extensions: ["lo_"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    const filePath = normalizeDialogSelection(selected);

    if (!filePath) {
      return;
    }

    await loadSource({ kind: "file", path: filePath }, "app-actions.open-file");
  }, [commandState.canOpenSources, loadSource]);

  const openLogFolderDialog = useCallback(async () => {
    if (!commandState.canOpenSources) {
      return;
    }

    const selected = await open({
      multiple: false,
      directory: true,
    });

    const folderPath = normalizeDialogSelection(selected);

    if (!folderPath) {
      return;
    }

    await loadSource(
      { kind: "folder", path: folderPath },
      "app-actions.open-folder"
    );
  }, [commandState.canOpenSources, loadSource]);

  const openKnownSourceById = useCallback(
    async (sourceId: string, trigger: string) => {
      await openKnownSourceCatalogAction({
        sourceId,
        trigger,
      });
    },
    [openKnownSourceCatalogAction]
  );

  const openKnownSourcePresetByMenuId = useCallback(
    async (presetMenuId: string) => {
      await openKnownSourceCatalogAction({
        presetMenuId,
        trigger: "native-menu.log-preset-selected",
      });
    },
    [openKnownSourceCatalogAction]
  );

  const showFindDialog = useCallback(() => {
    if (!commandState.canFind) {
      return;
    }

    useUiStore.getState().ensureLogViewVisible("app-actions.show-find");
    setShowFindDialog(true);
  }, [commandState.canFind, setShowFindDialog]);

  const showFilterDialog = useCallback(() => {
    if (!commandState.canFilter) {
      return;
    }

    useUiStore.getState().ensureLogViewVisible("app-actions.show-filter");
    setShowFilterDialog(true);
  }, [commandState.canFilter, setShowFilterDialog]);

  const showErrorLookupDialog = useCallback(() => {
    setShowErrorLookupDialog(true);
  }, [setShowErrorLookupDialog]);

  const showAboutDialog = useCallback(() => {
    setShowAboutDialog(true);
  }, [setShowAboutDialog]);

  const togglePauseResume = useCallback(() => {
    if (!commandState.canPauseResume) {
      return;
    }

    useLogStore.getState().togglePause();
  }, [commandState.canPauseResume]);

  const refreshActiveSource = useCallback(async () => {
    if (!commandState.canRefresh || !refreshSource) {
      return;
    }

    useUiStore.getState().ensureLogViewVisible("app-actions.refresh");
    useFilterStore.getState().clearFilter();

    await loadLogSource(refreshSource, {
      selectedFilePath: selectedSourceFilePath,
    });
  }, [commandState.canRefresh, refreshSource, selectedSourceFilePath]);

  const toggleDetailsPane = useCallback(() => {
    if (!commandState.canToggleDetailsPane) {
      return;
    }

    useUiStore.getState().toggleDetails();
  }, [commandState.canToggleDetailsPane]);

  const toggleInfoPane = useCallback(() => {
    if (!commandState.canToggleInfoPane) {
      return;
    }

    useUiStore.getState().toggleInfoPane();
  }, [commandState.canToggleInfoPane]);

  const dismissTransientDialogs = useCallback((trigger: string) => {
    useUiStore.getState().closeTransientDialogs(trigger);
  }, []);

  return {
    commandState,
    openLogFileDialog,
    openLogFolderDialog,
    openKnownSourceCatalogAction,
    openKnownSourceById,
    openKnownSourcePresetByMenuId,
    showFindDialog,
    showFilterDialog,
    showErrorLookupDialog,
    showAboutDialog,
    togglePauseResume,
    refreshActiveSource,
    toggleDetailsPane,
    toggleInfoPane,
    dismissTransientDialogs,
  };
}

export function Toolbar() {
  const highlightText = useLogStore((s) => s.highlightText);
  const setHighlightText = useLogStore((s) => s.setHighlightText);
  const knownSourceToolbarGroups = useLogStore((s) => s.knownSourceToolbarGroups);
  const isLoading = useLogStore((s) => s.isLoading);
  const isPaused = useLogStore((s) => s.isPaused);
  const activeSource = useLogStore((s) => s.activeSource);
  const openFilePath = useLogStore((s) => s.openFilePath);

  const activeView = useUiStore((s) => s.activeView);
  const setActiveView = useUiStore((s) => s.setActiveView);

  const {
    commandState,
    openLogFileDialog,
    openLogFolderDialog,
    openKnownSourceCatalogAction,
    showErrorLookupDialog,
    togglePauseResume,
    refreshActiveSource,
    toggleDetailsPane,
    toggleInfoPane,
  } = useAppActions();

  const [selectedOpenAction, setSelectedOpenAction] = useState("");
  const [selectedKnownSourceId, setSelectedKnownSourceId] = useState("");

  useEffect(() => {
    refreshKnownLogSources().catch((error) => {
      console.warn("[toolbar] failed to refresh known sources", { error });
    });
  }, []);

  const handleOpenActionChange = async (
    event: ChangeEvent<HTMLSelectElement>
  ) => {
    const action = event.target.value;
    setSelectedOpenAction(action);

    if (!action) {
      return;
    }

    try {
      if (action === "open-file") {
        await openLogFileDialog();
      } else if (action === "open-folder") {
        await openLogFolderDialog();
      }
    } catch (error) {
      console.error("[toolbar] failed to open source from toolbar dropdown", {
        action,
        error,
      });
    } finally {
      setSelectedOpenAction("");
    }
  };

  const handleKnownSourceChange = async (
    event: ChangeEvent<HTMLSelectElement>
  ) => {
    const sourceId = event.target.value;
    setSelectedKnownSourceId(sourceId);

    if (!sourceId) {
      return;
    }

    try {
      await openKnownSourceCatalogAction({
        sourceId,
        trigger: "toolbar.known-source-select",
      });
    } catch (error) {
      console.error("[toolbar] failed to open known source", { sourceId, error });
    } finally {
      setSelectedKnownSourceId("");
    }
  };

  const streamState = getStreamStateSnapshot(
    isLoading,
    isPaused,
    activeSource,
    openFilePath
  );

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
      <select
        value={selectedOpenAction}
        onChange={handleOpenActionChange}
        title="Open"
        style={{
          ...getToolbarControlStyle({ disabled: !commandState.canOpenSources }),
          fontSize: "12px",
          padding: "2px 4px",
          minWidth: "120px",
        }}
        disabled={!commandState.canOpenSources}
      >
        <option value="">Open...</option>
        <option value="open-file">Open File</option>
        <option value="open-folder">Open Folder</option>
      </select>
      <select
        value={selectedKnownSourceId}
        onChange={handleKnownSourceChange}
        title="Open a known log source"
        style={{
          ...getToolbarControlStyle({
            disabled:
              !commandState.canOpenSources || knownSourceToolbarGroups.length === 0,
          }),
          fontSize: "12px",
          padding: "2px 4px",
          minWidth: "260px",
        }}
        disabled={!commandState.canOpenSources || knownSourceToolbarGroups.length === 0}
      >
        <option value="">
          {knownSourceToolbarGroups.length > 0
            ? "Open Known Source..."
            : "No Known Sources"}
        </option>
        {knownSourceToolbarGroups.map((group) => (
          <optgroup key={group.id} label={group.label}>
            {group.sources.map((source) => (
              <option key={source.id} value={source.id} title={source.description}>
                {source.label}
              </option>
            ))}
          </optgroup>
        ))}
      </select>
      <button
        onClick={togglePauseResume}
        title={`Pause / Resume (Ctrl+U) • ${streamState.label}`}
        disabled={!commandState.canPauseResume}
        aria-pressed={commandState.isPaused}
        style={getToolbarControlStyle({
          disabled: !commandState.canPauseResume,
          active: commandState.isPaused,
          tone: commandState.isPaused ? "warning" : "neutral",
        })}
      >
        {commandState.isPaused ? "Resume" : "Pause"}
      </button>
      <button
        onClick={() => {
          refreshActiveSource().catch((error) => {
            console.error("[toolbar] failed to refresh source", { error });
          });
        }}
        title="Refresh (F5)"
        disabled={!commandState.canRefresh}
        style={getToolbarControlStyle({ disabled: !commandState.canRefresh })}
      >
        Refresh
      </button>

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <label
        style={{
          fontSize: "12px",
          fontFamily: "'Segoe UI', Tahoma, sans-serif",
          color: commandState.activeView === "log" ? "#111827" : "#6b7280",
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
          border: "1px solid #9ca3af",
          borderRadius: "2px",
          backgroundColor: commandState.activeView === "log" ? "#ffffff" : "#f3f4f6",
        }}
      />

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <button
        onClick={showErrorLookupDialog}
        title="Error Lookup (Ctrl+E)"
        style={getToolbarControlStyle({ disabled: false })}
      >
        Error Lookup
      </button>

      <div
        style={{ width: "1px", height: "20px", backgroundColor: "#c0c0c0" }}
      />

      <button
        onClick={toggleDetailsPane}
        title="Show / Hide Details (Ctrl+H)"
        disabled={!commandState.canToggleDetailsPane}
        aria-pressed={commandState.isDetailsVisible}
        style={getToolbarControlStyle({
          disabled: !commandState.canToggleDetailsPane,
          active: commandState.isDetailsVisible,
        })}
      >
        Details
      </button>
      <button
        onClick={toggleInfoPane}
        title="Toggle Info Pane"
        disabled={!commandState.canToggleInfoPane}
        aria-pressed={commandState.isInfoPaneVisible}
        style={getToolbarControlStyle({
          disabled: !commandState.canToggleInfoPane,
          active: commandState.isInfoPaneVisible,
        })}
      >
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
        aria-pressed={activeView === "intune"}
        style={getToolbarControlStyle({
          disabled: false,
          active: activeView === "intune",
        })}
      >
        {activeView === "intune" ? "← Log View" : "Intune"}
      </button>
    </div>
  );
}

