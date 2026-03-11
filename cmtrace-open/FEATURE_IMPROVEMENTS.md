# CMTrace Open — Feature Improvements Roadmap

Internal planning document. Organized by category with effort estimates and priority tiers.

**Priority key**: P0 = critical path, P1 = high value, P2 = nice-to-have, P3 = future consideration
**Effort key**: S = small (< 4 hours), M = medium (4–16 hours), L = large (16–40 hours), XL = 40+ hours

## Completed

The following work is shipped and should be treated as complete unless a regression is found:

- Log sources are first-class inputs: **file**, **folder**, and **known platform sources/presets**.
- Log view supports opening a folder and browsing files from a left sidebar.
- The toolbar includes **Open Folder**.
- Known source presets now include **Windows IME logs**.
- Intune analysis accepts an **IME folder path** (not only a single file).
- Native menu preset selections are emitted to the frontend and routed through the shared source-loading flow.
- IME folder file discovery includes the documented sidecar bundle instead of narrowing back to `IntuneManagementExtension*.log`.
- Intune results retain the expanded source file list in the frontend model and store.
- The Intune summary/header shows included source-file context for folder analysis.
- The Intune timeline shows per-event source-file context.
- Intune extraction now applies file-aware rules for `AppWorkload.log`, `AppActionProcessor.log`, `AgentExecutor.log`, and `HealthScripts.log`.
- Intune sidecar events now use more specific heuristic naming for install phases, policy evaluation, and detection/remediation script context.
- Aggregated IME timelines now sort by parsed timestamps and collapse consumed duplicate completion events more reliably.
- Intune summary output now includes deterministic diagnostics counters such as pending, timed out, failed downloads, successful downloads, and failed scripts.
- Intune summary now includes evidence-based diagnostics guidance with likely issue clusters, supporting evidence, and next checks.
- Intune diagnostics now apply stronger evidence rules for download, install, applicability, and script-failure patterns.
- Intune diagnostics now include rule-based suggested fixes when known error codes or log evidence make remediation guidance specific enough to trust.
- Parser expansion has been reassessed after the richer IME extraction pass and remains lower priority than sample-driven Intune diagnostics refinement.

## Active Focus

The remaining Intune gap is now sample-driven refinement of the Intune evidence library and broader remediation coverage across additional IME sidecar patterns rather than folder selection, provenance display, baseline diagnostics guidance, or parser-priority ambiguity.

### Recommended Next Implementation Slice

The next slice should stay focused on Intune diagnostics quality before any parser expansion work is resumed:

1. Expand the evidence-rule library using real failing log samples, especially for less common AppWorkload and remediation edge cases.
2. Grow guided remediation coverage only where the extracted evidence stays deterministic and auditable.
3. Reintroduce parser expansion only after the next round of Intune sample-driven refinement no longer yields higher diagnostic value.

---

## 1. Additional Log Format Parsers

Currently implemented: CCM, Simple, Plain Text, Timestamped (ISO/syslog/slash-date). The following Windows system log formats are absent and referenced in the log-format-reference.md specification.

### 1.1 CBS/Panther Format Family — P1 / M

**Covers**: `CBS.log`, `dism.log`, `DPX\setupact.log`, `setupact.log`, `setuperr.log`, `WinSetup`, `MoSetup`

These all share the same line structure:

```
YYYY-MM-DD HH:MM:SS, <Level> <Component> <Message>
```

- Levels: `Info`, `Error`, `Warning`, `Perf`
- Components: `CBS`, `CSI`, `DPX`, `DISM`, `SP`, `CONX`, `MOUPG`, `MIG`, `PANTHR`, `IBS`
- Optional hex prefix between level and component on some lines: `[0x0f0082]`
- `[SR]` tag marks SFC entries within CSI component lines
- Multi-line: continuation lines lack a timestamp — append to previous entry
- HRESULT codes at line end: `[HRESULT = 0x80070002]`

**Implementation notes**:

- Single new parser module (`parser/cbs.rs`) covers all 7+ log files in this family.
- Auto-detect trigger: first line matches `^\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2},\s`.
- Extract HRESULT from message tail for error lookup integration.
- Map `[SR]` tagged lines to a synthetic "SFC" component for filter dimension support.

### 1.2 SetupAPI Section-Based Format — P2 / M

**Covers**: `setupapi.dev.log`, `setupapi.setup.log`

Section-delimited format using `>>>` (start) and `<<<` (end) markers. Not line-per-entry — each section is a logical unit.

```
>>>  [Device Install (Hardware initiated) - USB\VID_045E&PID_07A5\...]
>>>  Section start 2026/03/09 14:05:57.100
     dvi: {Build Driver List} 14:05:57.115
<<<  Section end 2026/03/09 14:05:58.220
<<<  [Exit status: SUCCESS]
```

- Encoding: ANSI (system codepage) — requires Windows-1252 fallback.
- Body line prefixes: `ump`, `ndv`, `dvi`, `inf`, `cpy`, `sto`, `sig`, `flq`.
- Sub-operations in curly braces with exit codes: `{Operation - exit(0xNNNNNNNN)}`.
- Exit status: `SUCCESS` or `FAILURE(0xHHHHHHHH)`.

**Implementation notes**:

- Model sections as collapsed entries in the virtual list; expand on click.
- Section title becomes the "message" field; section type becomes "component."
- Map `FAILURE` exit status to Error severity, `SUCCESS` to Info.
- Consider rendering as a tree view within the log list for nested sub-operations.

### 1.3 MSI Multi-Format Parser — P2 / M

**Covers**: `MSI*.LOG` files from `%TEMP%`

MSI logs alternate between several line formats within a single file:

- **Action lines**: `Action start HH:MM:SS: ActionName.`
- **MSI engine lines**: `MSI (s) (PID:TID) [HH:MM:SS:mmm]: <message>`
- **Property lines**: `Property(S): PropertyName = Value`
- **Raw output**: custom action stdout/stderr with no prefix

- PID:TID in hex (e.g., `(C8:0C)`).
- Return values: 0=no action, 1=success, 2=user cancel, 3=fatal error.
- Header/footer: `=== Verbose logging started: ...` / `=== Verbose logging stopped: ...`

**Implementation notes**:

- Classify each line by prefix pattern, then dispatch to sub-parsers.
- Map `Return value 3` to Error severity, `Return value 2` to Warning.
- Extract Property(S)/Property(C) pairs into a structured properties view (info pane enhancement).
- Auto-detect trigger: first line starts with `===` or early lines contain `MSI (`.

### 1.4 WindowsUpdate.log — P1 / S

**Covers**: Generated `WindowsUpdate.log` (decoded from ETL traces)

```
YYYY/MM/DD HH:MM:SS.mmmmmmm PID  TID  <Component>  <Message>
```

- Timestamp: 27 characters with 100-nanosecond precision.
- Components: `Agent`, `Handler`, `DownloadManager`, `Setup`, `Misc`, `Report`, `SLS`, `PT`.
- Key markers: `*START*` / `*END*` delimit operations; `*FAILED*` and `FATAL:` indicate errors.

**Implementation notes**:

- The existing timestamped parser handles ISO and slash-date formats but does not support 7-digit sub-second precision. Extend or add a dedicated variant.
- Detect `*FAILED*` and `FATAL:` in message for error severity assignment.
- Auto-detect trigger: timestamp matches `^\d{4}/\d{2}/\d{2}\s\d{2}:\d{2}:\d{2}\.\d{7}`.

### 1.5 ReportingEvents.log (Tab-Delimited) — P2 / S

**Covers**: `C:\Windows\SoftwareDistribution\ReportingEvents.log`

Tab-delimited with GUID prefix. Fields: GUID, timestamp (with timezone offset), EventID, Category, Level, Update GUID, HRESULT, numeric, Agent name, Status, Operation type, Message.

**Implementation notes**:

- New parser module (`parser/tab_delimited.rs`).
- Split on `\t`; map fields positionally.
- Auto-detect trigger: first data line starts with `{` and contains tab characters.

### 1.6 W3C Extended Log Format — P2 / S

**Covers**: `pfirewall.log`

Self-describing format with `#Fields:` header defining column order. Data lines are space-delimited. Missing values represented by `-`.

**Implementation notes**:

- Parse `#Fields:` line to build dynamic column mapping.
- Skip `#` header lines.
- Map `DROP` action to Warning severity, `ALLOW` to Info.
- Logging is disabled by default — document this in the UI when opening firewall logs.

### 1.7 XML Log Format — P3 / S

**Covers**: `Diagerr.xml`, `Diagwrn.xml`

Standard XML with `<Diagnostic>` elements containing `DateTime`, `Component`, `Level`, `Message`, `ErrorCode`.

**Implementation notes**:

- Use `quick-xml` crate (already a common Rust XML parser).
- Map each `<Diagnostic>` element to a log entry.
- `Diagerr.xml` = Error level; `Diagwrn.xml` = Warning level (override if `<Level>` element differs).

### 1.8 UTF-16 Encoded Logs — P3 / S

**Covers**: `SrtTrail.txt`, `PFRO.log`

Both use UTF-16 LE with BOM. `PFRO.log` has timestamped operations; `SrtTrail.txt` uses section/key-value format.

**Implementation notes**:

- Add UTF-16 LE BOM detection to the encoding handler (currently UTF-8 + Windows-1252 only).
- `PFRO.log`: parse operations (`Deleted file`, `Renamed file`, `Could not delete file`) and extract error codes.
- `SrtTrail.txt`: section-based (dashed separators) with key-value pairs — present as structured entries.

### 1.9 Auto-Detection Algorithm Update

Extend `parser/detect.rs` to support the new formats. Proposed priority order:

```
1. UTF-16 BOM check → UTF-16 decoder first
2. <?xml → XML parser
3. <![LOG[ → CCM parser (existing)
4. #Version: or #Fields: → W3C parser
5. >>> at line start → SetupAPI Section parser
6. === at line start or MSI ( → MSI parser
7. {GUID}\t → ReportingEvents (tab) parser
8. YYYY-MM-DD HH:MM:SS, → CBS/Panther parser
9. YYYY/MM/DD HH:MM:SS.NNNNNNN → WindowsUpdate.log parser
10. Existing timestamped/simple/plain fallback chain
```

---

## 2. Intune Diagnostic Expansion

Currently implemented: IME log analysis with event timeline, download stats, and summary dashboard. Analysis can be launched from a single IME log file or an IME logs folder.

### 2.1 Additional IME Log File Support — P0 / M

Status: partially implemented, with multi-file aggregation and first-pass sidecar-aware extraction now in place.

The following IME log files reside in `C:\ProgramData\Microsoft\IntuneManagementExtension\Logs\` and use the same CMTrace format as `IntuneManagementExtension.log`. The existing CCM parser handles them, and the Intune analysis engine now includes first-pass file-aware extraction for the highest-value sidecar logs, but deeper heuristics are still incomplete.

| Log File | Diagnostic Value | Event Types |
| --- | --- | --- |
| `AppWorkload.log` | Dedicated Win32/WinGet app workload (2408+) — download bytes, DO progress, staging, install | Win32App, WinGetApp, ContentDownload |
| `AppActionProcessor.log` | App action decisions, assignment workflow, applicability | Win32App, PolicyEvaluation |
| `AgentExecutor.log` | PowerShell script execution — command line, stdout, stderr, exit codes | PowerShellScript, Remediation |
| `HealthScripts.log` | Proactive Remediations — detection/remediation scheduling | Remediation |
| `ClientHealth.log` | IME agent health, service startup, connectivity | SyncSession |
| `ClientCertCheck.log` | MDM client certificate validation (2408+) | Other |
| `DeviceHealthMonitoring.log` | Hardware readiness, app crash telemetry | Other |
| `Sensor.log` | SensorFramework — device event subscriptions | Other |
| `Win32AppInventory.log` | WMI-based Win32 app inventory scans | Win32App |
| `ImeUI.log` | Toast notification UI events | Other |

**Implementation notes**:

- Completed: the Intune analysis command accepts a directory path rather than only a single file path.
- Completed: file discovery now includes the known IME sidecar logs in folder analysis instead of preferring only `IntuneManagementExtension*.log` when that file exists.
- Completed: file-aware extraction rules now classify core events from `AppWorkload.log`, `AppActionProcessor.log`, `AgentExecutor.log`, and `HealthScripts.log`.
- Completed: sidecar-derived events now use more specific heuristic names for AppWorkload install phases, AppActionProcessor policy evaluation, and AgentExecutor or HealthScripts detection versus remediation context.
- Completed: aggregated timelines now sort by parsed timestamps and collapse consumed duplicate completion events more reliably.
- Completed: per-event source-file context is exposed in the timeline UI.
- Completed: the expanded source-file list is exposed in the frontend model and summary/header UI.
- Completed: the Intune summary now surfaces deterministic diagnostics counters for download outcomes, pending work, timeouts, and script failures.
- Remaining: deepen extraction heuristics so download/install, applicability, remediation, and script failures carry more specific classifications and evidence.
- `AppWorkload.log` remains the highest-priority addition because it contains the detailed download/install data that moved out of the primary IME log in service release 2408.

### 2.2 IME Log Directory Quick-Open — Implemented

The app now supports the IME directory workflow through known platform sources/presets and shared source loading:

1. Users can select a known IME source from the native menu.
2. The frontend routes the preset selection through the same source-loading flow used by other source types.
3. Users can also open folders directly from the toolbar and browse files in the left sidebar.

This closes the core quick-open gap for IME diagnostics.

### 2.3 Event Log Channel Integration — P1 / L

Windows Event Logs (EVTX) are a primary diagnostic source for MDM, Autopilot, BitLocker, LAPS, and Defender. CMTrace.exe does not support them. Adding event log reading would be a significant differentiator.

**Target channels (priority order)**:

| Channel | Use Case |
|---|---|
| `DeviceManagement-Enterprise-Diagnostics-Provider/Admin` | Primary MDM log — policy errors, CSP operations, enrollment |
| `DeviceManagement-Enterprise-Diagnostics-Provider/Operational` | Day-to-day MDM operations |
| `Microsoft-Windows-AAD/Operational` | Entra ID registration, token requests, device join |
| `Microsoft-Windows-ModernDeployment-Diagnostics-Provider/Autopilot` | Autopilot deployment events |
| `Microsoft-Windows-BitLocker/BitLocker Management` | Encryption, key rotation |
| `Microsoft-Windows-LAPS/Operational` | LAPS password rotation, policy config |
| `Microsoft-Windows-SENSE/Operational` | Defender for Endpoint onboarding/connectivity |
| `Intune-Bootstrapper-Agent` (Applications and Services Logs) | Autopilot v2 (Device Preparation) |

**Implementation notes**:

- Use the `windows` crate (`EvtQuery`, `EvtNext`, `EvtRender`) for native EVTX reading on Windows.
- Map Event ID + Level to the existing log entry model (message, component, severity, timestamp, thread).
- Add an "Event Logs" tab or panel in the sidebar alongside the existing log viewer.
- Include a channel picker dropdown that lists common Intune-relevant channels.
- Consider shipping a curated "channel presets" list for Intune admins (Enrollment, App Deployment, Security, Compliance).
- On non-Windows platforms, support offline `.evtx` file reading via a Rust EVTX parser crate (e.g., `evtx`).

### 2.4 Autopilot Diagnostics Panel — P1 / L

Build a dedicated Autopilot diagnostic view (similar to the existing Intune Dashboard) that aggregates data from:

- Autopilot profile JSON files (`AutopilotDDSZTDFile.json`, `AutopilotConfigurationFile.json`)
- ESP tracking registry (`HKLM\Software\Microsoft\Windows\Autopilot\EnrollmentStatusTracking\`)
- Autopilot event log channel (`ModernDeployment-Diagnostics-Provider/Autopilot`)
- `setupact.log` entries with `MOUPG` / `PANTHR` components
- Autopilot v2: `Intune-Bootstrapper-Agent` event log + `AutopilotDevicePrepHint` registry value

**Panel features**:

- Deployment mode detection (User-Driven, Self-Deploying, Pre-Provisioned, Device Preparation)
- ESP phase timeline (Device Prep → Device Setup → Account Setup) with status indicators
- Profile configuration summary (extracted from JSON)
- Error correlation: link Autopilot event IDs to known failure patterns
- AP v2 workload progress visualization (from `InitialProvisioning` registry JSON)

### 2.5 MDM Policy Viewer — P2 / L

Read and display MDM policy state from registry:

- `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\<Area>\` — effective policies
- `HKLM\SOFTWARE\Microsoft\PolicyManager\Providers\<GUID>\default\Device\<Area>\` — provider-delivered policies
- `HKLM\SOFTWARE\Microsoft\Enrollments\<GUID>\` — enrollment state

**Features**:

- Side-by-side comparison of intended vs. effective policy values.
- Highlight conflicts between MDM and Group Policy (`ControlPolicyConflict\MDMWinsOverGP`).
- Link policy CSP paths to Microsoft Learn documentation URLs.
- Export policy snapshot as JSON for support ticket attachments.

### 2.6 macOS Intune Log Support — P3 / XL

macOS Intune logs use a pipe-delimited format:

```
DateTime | Process | LogLevel | PID | Task | TaskInfo
```

Located at `/Library/Logs/Microsoft/Intune/` (system) and `~/Library/Logs/Microsoft/Intune/` (user).

**Implementation notes**:

- New parser module (`parser/pipe_delimited.rs`).
- Map `I` → Info, `W` → Warning, `E` → Error.
- Add unified logging predicate execution (`log show --predicate '...'`) as a Tauri command for live macOS MDM diagnostics.
- Add a macOS log source picker with curated predicates from the log-intune-reference.md.
- Lower priority because the primary user base is Windows-focused.

### 2.7 Diagnostics Stats and Guided Issue Insight — P1 / M

Add a diagnostics summary layer that turns raw log output into actionable health signals before attempting broader issue explanation.

Status: partially implemented. Deterministic diagnostics counters, issue clustering, evidence-based next checks, and first-pass suggested fixes are now shipped. Remaining work is broader health and coverage stats plus more sample-driven remediation rules.

**Core capabilities**:

- Log health and coverage stats: per-file counts, oldest/newest timestamps, gaps in coverage, rotated-log presence, and dominant source files.
- Failure clustering: repeated error codes, top failing apps/scripts, retry loops, stalled downloads, and recurring remediation failures.
- Evidence-driven issue insight: surface likely cause, supporting evidence, adjacent logs to inspect, and the next checks to run.
- Suggested fixes should be rule-based first, tied to known error codes and patterns, rather than freeform AI-generated troubleshooting.

**Implementation notes**:

- Build this on top of the existing Intune aggregation pipeline so summaries use the same extracted events, downloads, and source-file provenance.
- Start with deterministic heuristics for common Intune cases such as download failures, script exit-code failures, assignment/applicability failures, and repeated timeout patterns.
- Reuse the embedded error lookup database and expand it with Intune-specific patterns so insight text is grounded in known evidence.
- Present results as a compact summary panel with sections like `Likely Cause`, `Evidence`, `Next Checks`, and `Suggested Fix`.
- Completed: current diagnostics now cover download/staging failures, install enforcement failures, applicability blocks, script failures, timeout clusters, and rule-based suggested fixes driven by known evidence.
- Keep the initial implementation rule-based and auditable; only consider broader synthesis after the underlying fact extraction is reliable.

---

## 3. CMTrace Feature Parity

Features present in the original CMTrace.exe (v5.00.9078.1000) that are not yet implemented.

### 3.1 Time Delta Calculation (Ctrl+D) — P0 / S

Display elapsed time between log entries. Original CMTrace behavior:

- Single selection: elapsed time from first entry to selected entry.
- Range selection: elapsed time from first selected to last selected.
- Status bar format: `Elapsed time is Xh Xm Xs Xms (X.XXX seconds)`

**Implementation notes**:

- Calculate delta from parsed timestamps.
- Display in the status bar component.
- Handle entries without timestamps gracefully (show "N/A" or skip).

### 3.2 Save As / Export (Ctrl+S) — P0 / S

Export the current view (respecting active filters) to:

- Plain text (tab-separated, matching CMTrace clipboard format)
- CSV (with headers: Message, Component, DateTime, Thread, Severity)
- Filtered subset only (currently visible entries after filter application)

**Implementation notes**:

- Reuse the existing clipboard copy logic (`commands/file_ops.rs`) as the base.
- Add file save dialog with format selection.
- Respect active filters — export only the visible subset.
- Include a header row for CSV export.

### 3.3 Preferences / Settings Dialog — P1 / M

Persistent application settings stored via Tauri's `tauri-plugin-store` or the filesystem.

**Settings to persist**:

| Setting | Default | Notes |
|---|---|---|
| Highlight color | `#FFFF00` | Color picker in preferences |
| Highlight case sensitivity | false | Checkbox |
| Column widths | Auto | Resizable columns, save on change |
| Column visibility | All visible | Toggle per column |
| Window size/position | System default | Restore on launch |
| Last opened directory | None | Pre-fill Open dialog |
| Refresh interval | 500ms | Configurable (min 500ms per CMTrace) |
| Recent files | Empty | Last 10 opened files |
| Theme | System | Light / Dark / System |
| Font size | 13px | Adjustable |
| Find history | Empty | Last 10 search terms |

**Implementation notes**:

- Add `tauri-plugin-store` dependency.
- Create a `PreferencesDialog.tsx` component.
- Load settings on app startup; write on change.
- Expose settings via Zustand store for reactive UI updates.

### 3.4 Multi-File Support — P1 / L

Support opening multiple log files simultaneously.

**Two modes**:

1. **Tabbed view**: Each file in its own tab. Switch between files. Independent filter/highlight per tab.
2. **Merged view**: Interleave entries from multiple files chronologically (CMTrace's "Merge selected files" option). Add a "Source" column showing the origin filename.

**Implementation notes**:

- Tabbed view: extend `log-store.ts` to hold a map of `tabId → LogState`. Add a tab bar component.
- Merged view: sort entries by timestamp across files; handle clock skew gracefully.
- Merged view requires all files to have parseable timestamps; fall back to tabbed if timestamps are absent.
- Consider memory impact — 5 × 100K-line files = 500K entries in the virtual list.

### 3.5 Print Support (Ctrl+P) — P2 / M

Print the current log view.

**Implementation notes**:

- Generate an HTML representation of visible entries (respecting filters and highlight).
- Use Tauri's `tauri-plugin-printer` or shell-out to the OS print dialog.
- Include header: filename, entry count, filter summary, timestamp range.
- CMTrace uses Courier New for print — maintain monospace font.
- Support portrait and landscape orientation.

### 3.6 Regex Support in Find/Filter — P1 / S

Add a "Regex" toggle to the Find dialog and Filter dialog.

**Implementation notes**:

- Use Rust's `regex` crate (already likely a transitive dependency) for pattern compilation and matching.
- Validate regex on input; show inline error for invalid patterns.
- Default to simple string matching (current behavior); regex is opt-in via toggle.
- Apply regex to the same fields currently supported: message, component, thread, timestamp.

### 3.7 Find History — P2 / S

Store the last 10 search terms in a dropdown within the Find dialog.

**Implementation notes**:

- Persist via the settings store (see 3.3).
- Show dropdown on focus or click of a history icon.
- De-duplicate entries; most recent first.
- CMTrace stores these as `FindText0` through `FindTextN` in the registry.

### 3.8 Recent Files Menu — P2 / S

Add a "Recent Files" submenu or dropdown to the File menu / toolbar.

**Implementation notes**:

- Store last 10 file paths in the settings store.
- Show filename and truncated path.
- Validate existence before offering; remove stale entries.
- Bind Ctrl+R or add to File menu.

### 3.9 Column Customization — P2 / M

Allow users to resize, reorder, and hide columns.

**Implementation notes**:

- Make column headers draggable for reordering.
- Add drag handles on column borders for resizing.
- Right-click column header → context menu with show/hide toggles.
- Persist column state in settings store.

---

## 4. UI/UX Enhancements

### 4.1 Severity Quick-Filter Buttons — P0 / S

Add toolbar buttons to toggle visibility of each severity level:

- **[E]** Errors (red) — toggle on/off
- **[W]** Warnings (yellow) — toggle on/off
- **[I]** Info (default) — toggle on/off

**Implementation notes**:

- These operate independently of the Filter dialog.
- Apply as a pre-filter before the main filter pipeline.
- Show count badges: `E: 42 | W: 128 | I: 9,830`.
- Keyboard shortcuts: Ctrl+1 (errors), Ctrl+2 (warnings), Ctrl+3 (info).

### 4.2 Dark Theme — P1 / S

The app currently uses a fixed color scheme. Add dark theme support.

**Implementation notes**:

- Use CSS custom properties for all colors.
- Detect OS preference via `prefers-color-scheme` media query.
- Allow manual override in Preferences (Light / Dark / System).
- Severity colors need adjustment for dark backgrounds (e.g., dimmer red, amber instead of bright yellow).

### 4.3 Entry Count and Position Indicator — P1 / S

Show current position and total entries in the status bar:

```
Entry 4,521 of 98,432 | Filtered: 1,203 | Errors: 42 | Warnings: 128
```

**Implementation notes**:

- Track scroll position from the virtual list.
- Update on scroll, filter change, and new entries (tail mode).
- Show filter reduction percentage to indicate filter effectiveness.

### 4.4 Go-To Line / Go-To Timestamp — P2 / S

Add a "Go To" dialog (Ctrl+G) that accepts:

- Line number (jump to absolute position)
- Timestamp (jump to nearest entry at or after the specified time)

### 4.5 Bookmark / Pin Entries — P2 / M

Allow users to bookmark specific entries for quick navigation.

- Ctrl+B to toggle bookmark on the selected entry.
- Bookmark gutter indicator (star or flag icon).
- "Next Bookmark" / "Previous Bookmark" navigation.
- Bookmarks persist for the session (optionally to file).

### 4.6 Log Entry Detail Enhancement — P2 / S

Improve the info pane (detail view) for selected entries:

- Show all parsed fields in a structured key-value layout.
- For CMTrace format: display source file attribute (`file="source.cpp:123"`).
- For entries with HRESULT/error codes: inline error lookup result.
- Clickable URLs in message text.
- Monospace font for the raw message; proportional font for metadata.

### 4.7 Tail Mode Indicator — P1 / S

Make the real-time tail state more visible:

- Animated "LIVE" badge in the status bar when tailing is active.
- Auto-scroll indicator (arrow-down icon) that shows when new entries are arriving.
- "Jump to latest" button when the user scrolls away from the bottom during tail mode.
- Entry arrival rate: `~12 entries/sec`.

---

## 5. Error Lookup Expansion

### 5.1 Expand Embedded Error Database — P1 / M

Current: 120+ codes. CMTrace.exe ships with 710 embedded codes.

**Additions needed**:

- Windows Update Agent errors (0x8024xxxx series — partial coverage exists)
- Intune-specific errors (0x87D00xxx through 0x87D1xxxx — expand from current set)
- HRESULT standard codes (0x80004001 through 0x8007xxxx)
- NTSTATUS codes (0xC0000001 through 0xC00002xx)
- MDM enrollment errors (0x80180001 through 0x8018xxxx)
- BitLocker errors (FVE_E_* series)
- LAPS errors
- Autopilot errors (0x800xxxxx series specific to Autopilot)
- WinHTTP errors (expand from current)
- COM/DCOM errors

**Implementation notes**:

- Consider sourcing from Microsoft's public error code documentation programmatically.
- On Windows, supplement with `FormatMessage` system call for unknown codes (CMTrace does this via `FormatMessageW`).
- Structure as a JSON or TOML resource file for easier maintenance than the current hardcoded `codes.rs`.

### 5.2 Inline Error Detection — P1 / S

Automatically detect error codes within log entry messages and provide hover-to-lookup.

- Detect hex patterns: `0x[0-9A-Fa-f]{8}`, `HRESULT = 0x...`, `hr = 0x...`, `Error: 0x...`
- Detect decimal patterns: `error code N`, `exit code N`, `return value N`
- Show tooltip with error description on hover.
- Click to open full Error Lookup dialog pre-populated with the code.

### 5.3 Error Code Hyperlinking — P2 / S

When an error code is recognized, hyperlink it to:

1. The inline lookup tooltip (see 5.2).
2. Optionally, the Microsoft Learn documentation URL for that error family.

---

## 6. Performance and Architecture

### 6.1 Incremental Parsing — P1 / M

Current behavior: re-parse the entire file on each tail cycle. For large files (100K+ lines), this is unnecessary.

**Improvement**:

- Track the byte offset of the last parsed position.
- On tail events, read only new bytes from the last offset.
- Parse only the new chunk and append to the existing entry list.
- This is partially implemented in `watcher/` but the parsing path re-reads fully.

### 6.2 Background Parsing with Progress — P2 / M

For large files (>10 MB), show a progress indicator during initial parse:

- Progress bar in the status bar or a loading overlay.
- Stream entries to the UI as they are parsed (progressive rendering).
- Cancel button for long-running parses.

### 6.3 Parser Plugin System — P3 / XL

Allow users to define custom log format parsers without recompiling.

**Options**:

- TOML/YAML format definition files with regex patterns and field mappings.
- Lua scripting for complex parsing logic (via `mlua` crate).
- WebAssembly parser modules for sandboxed extensibility.

This is a significant architectural change and should be deferred until the core format coverage stabilizes.

### 6.4 Memory-Mapped File Reading — P3 / M

For very large files (>500 MB), use memory-mapped I/O instead of buffered reads.

**Implementation notes**:

- Use the `memmap2` crate.
- Combine with virtual scrolling to render only visible entries.
- Parse on demand (lazy parsing) for entries outside the visible viewport.

---

## 7. Integration and Workflow

### 7.1 Collect Diagnostics Bundle Support — P1 / L

The Intune "Collect Diagnostics" remote action produces a ZIP containing IME logs, event logs, registry exports, and command output. Add direct support for opening these bundles.

**Features**:

- Open a `.zip` or `.cab` file → extract to temp directory → list contained files.
- Auto-identify file types and apply appropriate parsers.
- Unified timeline view across all contained log files.
- Registry dump viewer (parse `.reg` export files).
- Command output viewer (`dsregcmd /status`, `certutil -store`, etc.).

### 7.2 MDM Diagnostic Report Viewer — P2 / M

Parse and display `MDMDiagHtmlReport.html` and `MDMDiagReport.xml`:

- Extract policy CSP values (current vs. default).
- Show enrollment variables.
- Display certificate information.
- Highlight policy conflicts.

### 7.3 Graph API GUID Resolution — P2 / L

The Intune analysis extracts GUIDs for apps and policies. Add optional Microsoft Graph API integration to resolve these GUIDs to human-readable names.

**Features**:

- Authenticate via device code flow or interactive browser login.
- Cache resolved names for the session.
- Display resolved names alongside GUIDs in the event timeline.
- Support: `deviceManagement/mobileApps/{id}`, `deviceManagement/deviceCompliancePolicies/{id}`, `deviceManagement/deviceConfigurations/{id}`.

**Implementation notes**:

- Use `reqwest` for HTTP calls from the Rust backend.
- Make this opt-in — do not require Graph API access for basic functionality.
- Consider shipping a local GUID → name mapping file that users can populate manually as an offline alternative.

### 7.4 Command-Line Interface — P2 / S

Support opening files from the command line:

```
cmtrace-open <filepath>
cmtrace-open --intune-analysis <directory>
cmtrace-open --filter "contains:error" <filepath>
```

**Implementation notes**:

- Parse CLI args in the Tauri `setup` hook.
- Support file associations: `.log`, `.lo_`, `.evtx` (if event log support is added).
- On Windows, register file associations during install (NSIS installer configuration).

### 7.5 Log File Health Check — P3 / S

When opening an IME log directory, provide a quick health summary:

- Log rotation status (are rotated copies present?).
- Time span coverage (oldest entry to newest entry across all files).
- Gaps in coverage (periods with no log entries — may indicate service restarts).
- Log size vs. configured maximum (`LogMaxSize` registry value).

---

## 8. Prioritized Implementation Phases

### Phase 1 — Core Gaps (P0 items)

Target: immediate next release. Estimated effort: 3–5 days.

1. **Severity quick-filter buttons** (4.1) — S
2. **Time Delta calculation** (3.1) — S
3. **Save As / Export** (3.2) — S
4. **IME log directory quick-open** (2.2) — S
5. **Additional IME log file support** (2.1) — M

### Phase 2 — Log Format Expansion (P1 items)

Target: following release. Estimated effort: 1–2 weeks.

1. **CBS/Panther parser** (1.1) — M
2. **WindowsUpdate.log parser** (1.4) — S
3. **Regex find/filter** (3.6) — S
4. **Inline error detection** (5.2) — S
5. **Expand error database** (5.1) — M
6. **Dark theme** (4.2) — S
7. **Tail mode indicator** (4.7) — S
8. **Entry count/position indicator** (4.3) — S

### Phase 3 — Intune Deep Integration (P1 items)

Target: 1–2 months. Estimated effort: 2–3 weeks.

1. **Event log channel integration** (2.3) — L
2. **Autopilot diagnostics panel** (2.4) — L
3. **Collect Diagnostics bundle support** (7.1) — L
4. **Preferences/settings dialog** (3.3) — M
5. **Multi-file support** (3.4) — L
6. **Diagnostics stats and guided issue insight polish** (2.7) — M

### Phase 4 — Polish and Parity (P2 items)

Target: 2–3 months. Estimated effort: 2–3 weeks.

1. SetupAPI section parser (1.2)
2. MSI multi-format parser (1.3)
3. ReportingEvents.log parser (1.5)
4. W3C log format parser (1.6)
5. Print support (3.5)
6. Find history (3.7)
7. Recent files menu (3.8)
8. Column customization (3.9)
9. Go-To line/timestamp (4.4)
10. Bookmark/pin entries (4.5)
11. Log entry detail enhancement (4.6)
12. MDM policy viewer (2.5)
13. MDM diagnostic report viewer (7.2)
14. Graph API GUID resolution (7.3)
15. Command-line interface (7.4)
16. Error code hyperlinking (5.3)

### Phase 5 — Future (P3 items)

Target: long-term. No fixed timeline.

1. XML log format parser (1.7)
2. UTF-16 encoded logs (1.8)
3. macOS Intune log support (2.6)
4. Parser plugin system (6.3)
5. Memory-mapped file reading (6.4)
6. Log file health check (7.5)
7. Background parsing with progress (6.2)
