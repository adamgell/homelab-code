import { create } from "zustand";

type AppView = "log" | "intune";

interface UiState {
  activeView: AppView;
  showInfoPane: boolean;
  showDetails: boolean;
  infoPaneHeight: number;
  showFindDialog: boolean;
  showFilterDialog: boolean;
  showErrorLookupDialog: boolean;
  showAboutDialog: boolean;

  setActiveView: (view: AppView) => void;
  toggleInfoPane: () => void;
  toggleDetails: () => void;
  setInfoPaneHeight: (height: number) => void;
  setShowFindDialog: (show: boolean) => void;
  setShowFilterDialog: (show: boolean) => void;
  setShowErrorLookupDialog: (show: boolean) => void;
  setShowAboutDialog: (show: boolean) => void;
}

export const useUiStore = create<UiState>((set) => ({
  activeView: "log" as AppView,
  showInfoPane: true,
  showDetails: true,
  infoPaneHeight: 200,
  showFindDialog: false,
  showFilterDialog: false,
  showErrorLookupDialog: false,
  showAboutDialog: false,

  setActiveView: (view) => set({ activeView: view }),
  toggleInfoPane: () =>
    set((state) => ({ showInfoPane: !state.showInfoPane })),
  toggleDetails: () =>
    set((state) => ({ showDetails: !state.showDetails })),
  setInfoPaneHeight: (height) => set({ infoPaneHeight: height }),
  setShowFindDialog: (show) => set({ showFindDialog: show }),
  setShowFilterDialog: (show) => set({ showFilterDialog: show }),
  setShowErrorLookupDialog: (show) => set({ showErrorLookupDialog: show }),
  setShowAboutDialog: (show) => set({ showAboutDialog: show }),
}));
