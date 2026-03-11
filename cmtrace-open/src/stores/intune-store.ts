import { create } from "zustand";
import type {
  IntuneEvent,
  DownloadStat,
  IntuneSummary,
  IntuneDiagnosticInsight,
  IntuneEventType,
  IntuneStatus,
} from "../types/intune";

interface IntuneState {
  events: IntuneEvent[];
  downloads: DownloadStat[];
  summary: IntuneSummary | null;
  diagnostics: IntuneDiagnosticInsight[];
  sourceFile: string | null;
  sourceFiles: string[];
  isAnalyzing: boolean;
  selectedEventId: number | null;
  filterEventType: IntuneEventType | "All";
  filterStatus: IntuneStatus | "All";
  resultRevision: number;

  setResults: (
    events: IntuneEvent[],
    downloads: DownloadStat[],
    summary: IntuneSummary,
    diagnostics: IntuneDiagnosticInsight[],
    sourceFile: string,
    sourceFiles: string[]
  ) => void;
  setAnalyzing: (analyzing: boolean) => void;
  selectEvent: (id: number | null) => void;
  setFilterEventType: (type_: IntuneEventType | "All") => void;
  setFilterStatus: (status: IntuneStatus | "All") => void;
  clear: () => void;
}

const defaultInteractionState = {
  selectedEventId: null,
  filterEventType: "All" as const,
  filterStatus: "All" as const,
};

export const useIntuneStore = create<IntuneState>((set) => ({
  events: [],
  downloads: [],
  summary: null,
  diagnostics: [],
  sourceFile: null,
  sourceFiles: [],
  isAnalyzing: false,
  resultRevision: 0,
  ...defaultInteractionState,

  setResults: (events, downloads, summary, diagnostics, sourceFile, sourceFiles) =>
    set((state) => ({
      events,
      downloads,
      summary,
      diagnostics,
      sourceFile,
      sourceFiles,
      resultRevision: state.resultRevision + 1,
      ...defaultInteractionState,
    })),

  setAnalyzing: (analyzing) =>
    set({
      isAnalyzing: analyzing,
      ...(analyzing ? { selectedEventId: null } : {}),
    }),

  selectEvent: (id) => set({ selectedEventId: id }),
  setFilterEventType: (type_) => set({ filterEventType: type_ }),
  setFilterStatus: (status) => set({ filterStatus: status }),

  clear: () =>
    set({
      events: [],
      downloads: [],
      summary: null,
      diagnostics: [],
      sourceFile: null,
      sourceFiles: [],
      isAnalyzing: false,
      resultRevision: 0,
      ...defaultInteractionState,
    }),
}));
