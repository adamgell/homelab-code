# Copilot Instructions

## Validation commands

- `npm run build` — runs `tsc && vite build` for the React frontend bundle.
- `cd src-tauri && cargo test --quiet` — runs the Rust unit and integration tests.
- `cd src-tauri && cargo test parser::detect::tests::test_detect_ccm --quiet` — example of running a single Rust test by full name.
- `cd src-tauri && cargo test -- --list` — list available Rust tests before picking one to run.
- There is no `npm test` or `npm run lint` script in this repository, so do not invent a frontend test or lint step.

## High-level architecture

CMTrace Open is a Tauri desktop app with a React 19 + TypeScript frontend in `src/` and a Rust backend in `src-tauri/src/`.

- `src/components/layout/AppShell.tsx` is the frontend composition root. It wires together the toolbar, dialogs, log view, Intune dashboard, keyboard shortcuts, drag-and-drop support, and the live file watcher.
- `src/lib/commands.ts` is the main IPC bridge for opening logs, starting/stopping tailing, and running Intune analysis.
- Opening a log flows through `openLogFile()` into Rust `open_log_file`, which calls `parser::parse_file()` to auto-detect the log format, parse the entire file, and return `entries`, `formatDetected`, and `byteOffset`.
- Live tailing is split across both layers: `src/hooks/use-file-watcher.ts` starts/stops the tail session and listens for the `tail-new-entries` event, while Rust `commands/parsing.rs` starts a `TailReader` and emits incremental entries back to the UI.
- `src-tauri/src/state/app_state.rs` only keeps per-file metadata and active tail sessions. Parsed entries themselves live in the frontend Zustand stores.
- The parser stack in `src-tauri/src/parser/` currently supports `Ccm`, `Simple`, `Plain`, and `Timestamped` formats. `detect.rs` samples the first 20 non-empty lines and requires at least two timestamp matches before treating a file as `Timestamped`.
- Intune diagnostics are a separate backend pipeline: `commands/intune.rs` parses IME logs, extracts events, builds a timeline, calculates download statistics, and returns a summary for the frontend `IntuneDashboard`.

## Key conventions

- Frontend state is intentionally split across four Zustand stores: `log-store.ts`, `filter-store.ts`, `intune-store.ts`, and `ui-store.ts`. Extend the relevant store instead of duplicating the same state in component-local hooks.
- Preserve the parse-then-tail contract. `open_log_file` stores the detected `date_order` in `AppState`, and `start_tail` reuses it so slash-date timestamped logs continue parsing consistently while tailing.
- Rust structs that cross the Tauri boundary use `#[serde(rename_all = "camelCase")]`, and the TypeScript interfaces in `src/types/` mirror that shape directly. Keep new IPC payloads camelCase-compatible.
- Tauri commands in this repo conventionally return `Result<..., String>` and use `State<'_, AppState>` for shared backend state.
- File decoding intentionally strips a UTF-8 BOM first and falls back to Windows-1252. Preserve that behavior when changing parser I/O because SCCM-style logs are not guaranteed to be UTF-8.
- Filtering is backend-evaluated but entry-local: the frontend sends the currently loaded `entries` plus filter clauses to `apply_filter`, and Rust returns matching entry IDs. Clause matching is AND-based, and string comparisons are case-insensitive.
- `REVERSE_ENGINEERING.md` is the reference for CMTrace parity such as menus, shortcuts, and window layout. `FEATURE_IMPROVEMENTS.md` is the roadmap for additional parsers and the intended detection-order expansion, so consult those docs before changing parser behavior or UI parity.
