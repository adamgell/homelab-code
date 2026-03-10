export type Severity = "Info" | "Warning" | "Error";
export type LogFormat = "Ccm" | "Simple" | "Plain";

export interface LogEntry {
  id: number;
  lineNumber: number;
  message: string;
  component: string | null;
  timestamp: number | null;
  timestampDisplay: string | null;
  severity: Severity;
  thread: number | null;
  threadDisplay: string | null;
  sourceFile: string | null;
  format: LogFormat;
  filePath: string;
  timezoneOffset: number | null;
}

export interface ParseResult {
  entries: LogEntry[];
  formatDetected: LogFormat;
  totalLines: number;
  parseErrors: number;
  filePath: string;
  fileSize: number;
  byteOffset: number;
}

/** Payload emitted by the Rust tail watcher */
export interface TailPayload {
  entries: LogEntry[];
  filePath: string;
}
