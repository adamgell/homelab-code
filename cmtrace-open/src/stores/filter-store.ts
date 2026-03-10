import { create } from "zustand";
import type { FilterClause } from "../components/dialogs/FilterDialog";

interface FilterState {
  /** Active filter clauses */
  clauses: FilterClause[];
  /** Set of entry IDs that pass the filter (null = no filter active) */
  filteredIds: Set<number> | null;
  /** Whether a filter is currently being applied */
  isFiltering: boolean;

  setClauses: (clauses: FilterClause[]) => void;
  setFilteredIds: (ids: Set<number> | null) => void;
  setIsFiltering: (filtering: boolean) => void;
  clearFilter: () => void;
}

export const useFilterStore = create<FilterState>((set) => ({
  clauses: [],
  filteredIds: null,
  isFiltering: false,

  setClauses: (clauses) => set({ clauses }),
  setFilteredIds: (ids) => set({ filteredIds: ids }),
  setIsFiltering: (filtering) => set({ isFiltering: filtering }),
  clearFilter: () =>
    set({
      clauses: [],
      filteredIds: null,
      isFiltering: false,
    }),
}));
