import { create } from "zustand";
import type {
  IntuneEvent,
  DownloadStat,
  IntuneSummary,
  IntuneEventType,
  IntuneStatus,
} from "../types/intune";

interface IntuneState {
  events: IntuneEvent[];
  downloads: DownloadStat[];
  summary: IntuneSummary | null;
  sourceFile: string | null;
  isAnalyzing: boolean;
  selectedEventId: number | null;
  filterEventType: IntuneEventType | "All";
  filterStatus: IntuneStatus | "All";

  setResults: (
    events: IntuneEvent[],
    downloads: DownloadStat[],
    summary: IntuneSummary,
    sourceFile: string
  ) => void;
  setAnalyzing: (analyzing: boolean) => void;
  selectEvent: (id: number | null) => void;
  setFilterEventType: (type_: IntuneEventType | "All") => void;
  setFilterStatus: (status: IntuneStatus | "All") => void;
  clear: () => void;
}

export const useIntuneStore = create<IntuneState>((set) => ({
  events: [],
  downloads: [],
  summary: null,
  sourceFile: null,
  isAnalyzing: false,
  selectedEventId: null,
  filterEventType: "All",
  filterStatus: "All",

  setResults: (events, downloads, summary, sourceFile) =>
    set({ events, downloads, summary, sourceFile }),
  setAnalyzing: (analyzing) => set({ isAnalyzing: analyzing }),
  selectEvent: (id) => set({ selectedEventId: id }),
  setFilterEventType: (type_) => set({ filterEventType: type_ }),
  setFilterStatus: (status) => set({ filterStatus: status }),
  clear: () =>
    set({
      events: [],
      downloads: [],
      summary: null,
      sourceFile: null,
      selectedEventId: null,
    }),
}));
