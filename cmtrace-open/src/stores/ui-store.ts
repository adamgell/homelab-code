import { create } from "zustand";

type AppView = "log" | "intune";

export interface UiChromeStatus {
  viewLabel: string;
  detailsLabel: string;
  infoLabel: string;
}

export function getUiChromeStatus(
  activeView: AppView,
  showDetails: boolean,
  showInfoPane: boolean
): UiChromeStatus {
  if (activeView !== "log") {
    return {
      viewLabel: "Intune view",
      detailsLabel: "Details hidden in Intune view",
      infoLabel: "Info hidden in Intune view",
    };
  }

  return {
    viewLabel: "Log view",
    detailsLabel: showDetails ? "Details on" : "Details off",
    infoLabel: showInfoPane ? "Info on" : "Info off",
  };
}

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
  ensureLogViewVisible: (trigger: string) => void;
  toggleInfoPane: () => void;
  toggleDetails: () => void;
  setInfoPaneHeight: (height: number) => void;
  setShowFindDialog: (show: boolean) => void;
  setShowFilterDialog: (show: boolean) => void;
  setShowErrorLookupDialog: (show: boolean) => void;
  setShowAboutDialog: (show: boolean) => void;
  closeTransientDialogs: (trigger: string) => void;
}

export const useUiStore = create<UiState>((set, get) => ({
  activeView: "log" as AppView,
  showInfoPane: true,
  showDetails: true,
  infoPaneHeight: 200,
  showFindDialog: false,
  showFilterDialog: false,
  showErrorLookupDialog: false,
  showAboutDialog: false,

  setActiveView: (view) => {
    const previousView = get().activeView;

    if (previousView === view) {
      return;
    }

    console.info("[ui-store] changing active view", {
      previousView,
      view,
    });

    set({ activeView: view });
  },
  ensureLogViewVisible: (trigger) => {
    if (get().activeView === "log") {
      console.info("[ui-store] log view already visible", { trigger });
      return;
    }

    console.info("[ui-store] switching to log view for command", { trigger });
    set({ activeView: "log" });
  },
  toggleInfoPane: () =>
    set((state) => ({ showInfoPane: !state.showInfoPane })),
  toggleDetails: () =>
    set((state) => ({ showDetails: !state.showDetails })),
  setInfoPaneHeight: (height) => set({ infoPaneHeight: height }),
  setShowFindDialog: (show) => set({ showFindDialog: show }),
  setShowFilterDialog: (show) => set({ showFilterDialog: show }),
  setShowErrorLookupDialog: (show) => set({ showErrorLookupDialog: show }),
  setShowAboutDialog: (show) => set({ showAboutDialog: show }),
  closeTransientDialogs: (trigger) => {
    const state = get();

    if (
      !state.showFindDialog &&
      !state.showFilterDialog &&
      !state.showErrorLookupDialog &&
      !state.showAboutDialog
    ) {
      return;
    }

    console.info("[ui-store] closing transient dialogs", { trigger });

    set({
      showFindDialog: false,
      showFilterDialog: false,
      showErrorLookupDialog: false,
      showAboutDialog: false,
    });
  },
}));
