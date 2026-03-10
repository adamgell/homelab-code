import { invoke } from "@tauri-apps/api/core";
import type { LogFormat, ParseResult } from "../types/log";
import type { IntuneAnalysisResult } from "../types/intune";

export async function openLogFile(path: string): Promise<ParseResult> {
  return invoke<ParseResult>("open_log_file", { path });
}

export async function startTail(
  path: string,
  format: LogFormat,
  byteOffset: number,
  nextId: number,
  nextLine: number
): Promise<void> {
  return invoke("start_tail", { path, format, byteOffset, nextId, nextLine });
}

export async function stopTail(path: string): Promise<void> {
  return invoke("stop_tail", { path });
}

export async function pauseTail(path: string): Promise<void> {
  return invoke("pause_tail", { path });
}

export async function resumeTail(path: string): Promise<void> {
  return invoke("resume_tail", { path });
}

export async function analyzeIntuneLogs(
  path: string
): Promise<IntuneAnalysisResult> {
  return invoke<IntuneAnalysisResult>("analyze_intune_logs", { path });
}
