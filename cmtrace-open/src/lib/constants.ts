/** Color rules extracted from CMTrace binary */
export const COLORS = {
  error: {
    background: "#FF0000",
    text: "#FFFF00",
  },
  warning: {
    background: "#FFFF00",
    text: "#000000",
  },
  info: {
    background: "#FFFFFF",
    text: "#000000",
  },
  highlightDefault: "#FFFF00",
} as const;

/** Default update interval in ms (minimum 500, from string table ID=37) */
export const DEFAULT_UPDATE_INTERVAL_MS = 500;

/** Column names from string table IDs 2-5 */
export const COLUMN_NAMES = {
  logText: "Log Text",
  component: "Component",
  dateTime: "Date/Time",
  thread: "Thread",
} as const;
