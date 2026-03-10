import { create } from "zustand";
import type { LogEntry, LogFormat } from "../types/log";

interface LogState {
  entries: LogEntry[];
  selectedId: number | null;
  isPaused: boolean;
  isLoading: boolean;
  formatDetected: LogFormat | null;
  totalLines: number;
  openFilePath: string | null;
  highlightText: string;
  highlightCaseSensitive: boolean;
  /** Byte offset in the file after initial parse — used to start tailing */
  byteOffset: number;

  setEntries: (entries: LogEntry[]) => void;
  appendEntries: (entries: LogEntry[]) => void;
  selectEntry: (id: number | null) => void;
  togglePause: () => void;
  setLoading: (loading: boolean) => void;
  setFormatDetected: (format: LogFormat) => void;
  setTotalLines: (count: number) => void;
  setOpenFilePath: (path: string | null) => void;
  setByteOffset: (offset: number) => void;
  setHighlightText: (text: string) => void;
  setHighlightCaseSensitive: (sensitive: boolean) => void;
  clear: () => void;
}

export const useLogStore = create<LogState>((set) => ({
  entries: [],
  selectedId: null,
  isPaused: false,
  isLoading: false,
  formatDetected: null,
  totalLines: 0,
  openFilePath: null,
  highlightText: "",
  highlightCaseSensitive: false,
  byteOffset: 0,

  setEntries: (entries) => set({ entries }),
  appendEntries: (newEntries) =>
    set((state) => ({ entries: [...state.entries, ...newEntries] })),
  selectEntry: (id) => set({ selectedId: id }),
  togglePause: () => set((state) => ({ isPaused: !state.isPaused })),
  setLoading: (loading) => set({ isLoading: loading }),
  setFormatDetected: (format) => set({ formatDetected: format }),
  setTotalLines: (count) => set({ totalLines: count }),
  setOpenFilePath: (path) => set({ openFilePath: path }),
  setByteOffset: (offset) => set({ byteOffset: offset }),
  setHighlightText: (text) => set({ highlightText: text }),
  setHighlightCaseSensitive: (sensitive) =>
    set({ highlightCaseSensitive: sensitive }),
  clear: () =>
    set({
      entries: [],
      selectedId: null,
      formatDetected: null,
      totalLines: 0,
      openFilePath: null,
      byteOffset: 0,
    }),
}));
