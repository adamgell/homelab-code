import { create } from "zustand";
import type {
  IntuneAnalysisSourceKind,
  IntuneAnalysisState,
  IntuneEvent,
  DownloadStat,
  IntuneSummary,
  IntuneDiagnosticInsight,
  IntuneEventType,
  IntuneStatus,
  IntuneSourceContext,
  IntuneSourceSelection,
  IntuneTimelineScope,
} from "../types/intune";

export type IntuneWorkspaceTab = "timeline" | "downloads" | "summary";

function buildSourceContext(
  sourceFile: string | null,
  sourceFiles: string[]
): IntuneSourceContext {
  return {
    analyzedPath: sourceFile,
    includedFiles:
      sourceFiles.length > 0 ? sourceFiles : sourceFile != null ? [sourceFile] : [],
  };
}

function buildSourceSelection(
  filePath: string | null,
  lineNumber: number | null = null
): IntuneSourceSelection {
  return {
    filePath,
    lineNumber,
  };
}

function buildTimelineScope(filePath: string | null): IntuneTimelineScope {
  return { filePath };
}

function isEmptyAnalysisDetail(detail: string): boolean {
  const normalized = detail.toLowerCase();
  return (
    normalized.includes("does not contain any .log files") ||
    normalized.includes("no .log files found in directory")
  );
}

function getAnalysisFailureState(
  error: unknown,
  requestedKind: IntuneAnalysisSourceKind | null
): Pick<IntuneAnalysisState, "phase" | "message" | "detail" | "lastError"> {
  const detail =
    error instanceof Error
      ? error.message
      : typeof error === "string"
        ? error
        : "The selected Intune source could not be analyzed.";

  if (isEmptyAnalysisDetail(detail)) {
    return {
      phase: "empty",
      message: "No IME log files were found in this folder.",
      detail:
        "Choose a folder that contains IME log files such as IntuneManagementExtension.log or AppWorkload.log.",
      lastError: detail,
    };
  }

  const sourceLabel =
    requestedKind === "folder"
      ? "folder"
      : requestedKind === "file"
        ? "file"
        : "source";

  return {
    phase: "error",
    message: `Intune diagnostics could not read the selected ${sourceLabel}.`,
    detail,
    lastError: detail,
  };
}

const emptySourceContext = buildSourceContext(null, []);
const emptyTimelineScope = buildTimelineScope(null);

const defaultAnalysisState: IntuneAnalysisState = {
  phase: "idle",
  requestedPath: null,
  requestedKind: null,
  message: "Choose an Intune log file or folder to analyze.",
  detail: null,
  lastError: null,
};

interface IntuneState {
  events: IntuneEvent[];
  downloads: DownloadStat[];
  summary: IntuneSummary | null;
  diagnostics: IntuneDiagnosticInsight[];
  sourceFile: string | null;
  sourceFiles: string[];
  sourceContext: IntuneSourceContext;
  isAnalyzing: boolean;
  analysisState: IntuneAnalysisState;
  selectedEventId: number | null;
  sourceSelection: IntuneSourceSelection;
  timelineScope: IntuneTimelineScope;
  filterEventType: IntuneEventType | "All";
  filterStatus: IntuneStatus | "All";
  activeTab: IntuneWorkspaceTab;
  resultRevision: number;

  beginAnalysis: (
    requestedPath: string | null,
    requestedKind?: IntuneAnalysisSourceKind
  ) => void;
  setResults: (
    events: IntuneEvent[],
    downloads: DownloadStat[],
    summary: IntuneSummary,
    diagnostics: IntuneDiagnosticInsight[],
    sourceFile: string,
    sourceFiles: string[]
  ) => void;
  failAnalysis: (error: unknown) => void;
  selectEvent: (id: number | null) => void;
  setTimelineFileScope: (path: string | null) => void;
  clearTimelineFileScope: () => void;
  setFilterEventType: (type_: IntuneEventType | "All") => void;
  setFilterStatus: (status: IntuneStatus | "All") => void;
  setActiveTab: (tab: IntuneWorkspaceTab) => void;
  clear: () => void;
}

const defaultInteractionState = {
  selectedEventId: null,
  filterEventType: "All" as const,
  filterStatus: "All" as const,
  activeTab: "timeline" as const,
};

export const useIntuneStore = create<IntuneState>((set) => ({
  events: [],
  downloads: [],
  summary: null,
  diagnostics: [],
  sourceFile: null,
  sourceFiles: [],
  sourceContext: emptySourceContext,
  isAnalyzing: false,
  analysisState: defaultAnalysisState,
  resultRevision: 0,
  sourceSelection: buildSourceSelection(null),
  timelineScope: emptyTimelineScope,
  ...defaultInteractionState,

  beginAnalysis: (requestedPath, requestedKind = "unknown") =>
    set({
      events: [],
      downloads: [],
      summary: null,
      diagnostics: [],
      sourceFile: null,
      sourceFiles: [],
      sourceContext: emptySourceContext,
      isAnalyzing: true,
      analysisState: {
        phase: "analyzing",
        requestedPath,
        requestedKind,
        message:
          requestedKind === "folder"
            ? "Analyzing Intune folder..."
            : "Analyzing Intune log source...",
        detail: requestedPath,
        lastError: null,
      },
      sourceSelection: buildSourceSelection(null),
      timelineScope: emptyTimelineScope,
      ...defaultInteractionState,
    }),

  setResults: (events, downloads, summary, diagnostics, sourceFile, sourceFiles) =>
    set((state) => {
      const sourceContext = buildSourceContext(sourceFile, sourceFiles);
      const requestedKind =
        state.analysisState.requestedKind ?? (sourceFiles.length > 1 ? "folder" : "file");

      return {
        events,
        downloads,
        summary,
        diagnostics,
        sourceFile,
        sourceFiles,
        sourceContext,
        isAnalyzing: false,
        analysisState: {
          phase: "ready",
          requestedPath: state.analysisState.requestedPath ?? sourceFile,
          requestedKind,
          message:
            sourceFiles.length > 1
              ? `Analysis complete (${sourceFiles.length} files)`
              : "Analysis complete",
          detail: sourceFile,
          lastError: null,
        },
        resultRevision: state.resultRevision + 1,
        sourceSelection: buildSourceSelection(null),
        timelineScope: emptyTimelineScope,
        ...defaultInteractionState,
      };
    }),

  failAnalysis: (error) =>
    set((state) => {
      const failureState = getAnalysisFailureState(
        error,
        state.analysisState.requestedKind
      );

      return {
        isAnalyzing: false,
        analysisState: {
          requestedPath: state.analysisState.requestedPath,
          requestedKind: state.analysisState.requestedKind,
          ...failureState,
        },
      };
    }),

  selectEvent: (id) =>
    set((state) => {
      if (id == null) {
        return {
          selectedEventId: null,
          sourceSelection: buildSourceSelection(state.timelineScope.filePath),
        };
      }

      const selectedEvent = state.events.find((event) => event.id === id);

      return {
        selectedEventId: id,
        sourceSelection: buildSourceSelection(
          selectedEvent?.sourceFile ?? state.timelineScope.filePath,
          selectedEvent?.lineNumber ?? null
        ),
      };
    }),
  setTimelineFileScope: (path) =>
    set((state) => {
      const nextPath =
        path != null && state.sourceContext.includedFiles.includes(path) ? path : null;
      const selectedEvent = state.events.find((event) => event.id === state.selectedEventId);
      const keepSelectedEvent =
        selectedEvent != null &&
        (nextPath == null || selectedEvent.sourceFile === nextPath);

      return {
        timelineScope: buildTimelineScope(nextPath),
        selectedEventId: keepSelectedEvent ? state.selectedEventId : null,
        sourceSelection: keepSelectedEvent
          ? buildSourceSelection(selectedEvent?.sourceFile ?? nextPath, selectedEvent?.lineNumber ?? null)
          : buildSourceSelection(nextPath),
      };
    }),
  clearTimelineFileScope: () =>
    set((state) => {
      const selectedEvent = state.events.find((event) => event.id === state.selectedEventId);
      return {
        timelineScope: emptyTimelineScope,
        sourceSelection: buildSourceSelection(
          selectedEvent?.sourceFile ?? null,
          selectedEvent?.lineNumber ?? null
        ),
      };
    }),
  setFilterEventType: (type_) => set({ filterEventType: type_ }),
  setFilterStatus: (status) => set({ filterStatus: status }),
  setActiveTab: (tab) => set({ activeTab: tab }),

  clear: () =>
    set({
      events: [],
      downloads: [],
      summary: null,
      diagnostics: [],
      sourceFile: null,
      sourceFiles: [],
      sourceContext: emptySourceContext,
      isAnalyzing: false,
      analysisState: defaultAnalysisState,
      resultRevision: 0,
      sourceSelection: buildSourceSelection(null),
      timelineScope: emptyTimelineScope,
      ...defaultInteractionState,
    }),
}));
