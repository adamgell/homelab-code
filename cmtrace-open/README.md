# CMTrace Open

An open-source log viewer inspired by Microsoft's CMTrace.exe, built with **Tauri v2 + React + TypeScript + Rust**. Includes built-in Intune Management Extension diagnostics.

![MIT License](https://img.shields.io/badge/license-MIT-blue.svg) [![CMTrace Open: Release](https://github.com/adamgell/homelab-code/actions/workflows/cmtrace-release.yml/badge.svg)](https://github.com/adamgell/homelab-code/actions/workflows/cmtrace-release.yml)

## Features

### Log Viewer

- **CCM format** (`<![LOG[...]LOG]!>`) — full SCCM/ConfigMgr log parsing
- **Simple format** (`$$<` delimited) — text-based severity detection
- **Plain text** fallback with automatic format detection
- **Real-time tailing** — live file watching with pause/resume
- **Virtual scrolling** — smooth performance with 100K+ line files
- **Severity color coding** — Error (red), Warning (yellow), Info (white)
- **Find** (Ctrl+F) with F3/Shift+F3 navigation
- **Filter** — 6 clause types × 4 dimensions (message, component, thread, timestamp)
- **Highlight** — configurable text highlighting
- **Error Lookup** — 120+ embedded Windows/SCCM/Intune error codes
- **Drag & drop** — open files by dropping onto the window
- **Clipboard copy** — Ctrl+C copies selected entries

### Intune Diagnostics

- **IME log analysis** — parses IntuneManagementExtension.log files
- **Event timeline** — color-coded vertical timeline with status indicators
- **Event types** — Win32 App, WinGet App, PowerShell Script, Remediation, ESP, Sync Session
- **Download statistics** — size, speed, Delivery Optimization percentage
- **Summary dashboard** — event counts, success/failure rates, log time span
- **GUID extraction** — automatic app/policy identifier detection

### Keyboard Shortcuts

| Shortcut | Action |
| -------- | ------ |
| Ctrl+O | Open file |
| Ctrl+F | Find |
| Ctrl+L | Filter |
| Ctrl+U | Pause/Resume |
| Ctrl+H | Toggle details |
| Ctrl+E | Error lookup |
| Ctrl+C | Copy selection |
| F3 | Find next |
| Shift+F3 | Find previous |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Platform-specific dependencies (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Build Output

Production builds are located at:

- **macOS**: `src-tauri/target/release/bundle/macos/CMTrace Open.app`
- **Windows**: `src-tauri/target/release/bundle/nsis/CMTrace Open_x.x.x_x64-setup.exe`
- **Linux**: `src-tauri/target/release/bundle/deb/` and `appimage/`

## Architecture

```text
React Frontend (TypeScript)          Tauri IPC           Rust Backend
┌─────────────────────────┐     ┌──────────┐     ┌─────────────────────┐
│ LogListView (virtual)   │◄────┤ invoke() │────►│ Parser (CCM/Simple) │
│ InfoPane (detail view)  │     │ Events   │     │ FileWatcher (notify)│
│ Toolbar (highlight)     │     └──────────┘     │ ErrorDB (120+ codes)│
│ FilterDialog (4×6)      │                      │ IntuneEngine        │
│ IntuneDashboard         │                      │ AppState (Mutex)    │
└─────────────────────────┘                      └─────────────────────┘
```

- **Rust** handles all file I/O, parsing, watching, error lookup, and Intune analysis
- **React** handles UI rendering, state management (Zustand), and keyboard shortcuts
- **IPC**: `invoke()` for request/response, Tauri events for streaming tail data

## Project Structure

```text
src/                          # React frontend
├── components/
│   ├── layout/               # AppShell, Toolbar, StatusBar
│   ├── log-view/             # LogListView, LogRow, InfoPane
│   ├── dialogs/              # Find, Filter, ErrorLookup, About
│   └── intune/               # IntuneDashboard, EventTimeline, DownloadStats
├── stores/                   # Zustand stores (log, ui, filter, intune)
├── hooks/                    # File watcher, keyboard, drag-drop
├── types/                    # TypeScript type definitions
└── lib/                      # Tauri command wrappers

src-tauri/src/                # Rust backend
├── parser/                   # CCM, Simple, Plain parsers + auto-detect
├── watcher/                  # File tail with notify + poll fallback
├── error_db/                 # 120+ embedded error codes
├── intune/                   # IME parser, event tracker, timeline, downloads
├── commands/                 # Tauri IPC command handlers
├── models/                   # Shared data structures
└── state/                    # Application state management
```

## Performance

Benchmarked on Apple Silicon (M-series):

| Operation | 100K lines (18 MB) |
| --- | --- |
| File parse | ~360ms |
| Intune analysis | ~3.4s |
| Parse throughput | ~278K lines/sec |

## File Associations

CMTrace Open registers for `.log` and `.lo_` file extensions.

## License

[MIT](LICENSE)
