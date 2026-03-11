# CMTrace Open next implementation plan

This document replaces the completed folder browser and platform log menu plan. It captures what is already shipped and what should be implemented next based on a code review of the current branch.

## Status review

The following work is implemented and should be treated as complete:

- First-class log sources: file, folder, and known platform sources.
- Folder browsing in the left sidebar for log viewing.
- Toolbar support for Open Folder.
- Native app menu entries for known Windows log sources.
- Intune analysis command support for either a file path or a directory path.

The following gaps are still present in the implementation:

- True multi-file IME aggregation is now complete at the file-discovery layer. Folder analysis includes the documented IME sidecar bundle instead of narrowing back to `IntuneManagementExtension.log`, but extraction depth across those sidecar logs is still incomplete.
- The backend returns per-event `source_file` data and an expanded `source_files` list, but the frontend Intune result types and store still collapse the analysis down to a single `sourceFile` string.
- The timeline UI does not show which file produced each event, so cross-log correlation is still hard even though the data exists in the backend model.
- Parser auto-detection still only covers CCM, Simple, Timestamped, and Plain text. The next Windows-focused parsers in the roadmap are still absent.

## Recommended next slice

The next implementation slice should focus on making Intune folder analysis actually useful for current service releases before adding more parser families.

1. Finish IME folder aggregation.
2. Surface source-file context in the Intune UI.
3. Expand event extraction to the highest-value sidecar logs.
4. Start the next parser family after the Intune path is solid.

## Phase 1: Fix IME folder aggregation

Goal: make directory analysis include the logs that matter in the IME logs folder instead of silently narrowing back to the primary log.

- Completed: IME file discovery now includes the known sidecar files first: `IntuneManagementExtension.log`, `AppWorkload.log`, `AppActionProcessor.log`, `AgentExecutor.log`, `HealthScripts.log`, `ClientHealth.log`, `ClientCertCheck.log`, `DeviceHealthMonitoring.log`, `Sensor.log`, `Win32AppInventory.log`, and `ImeUI.log`.
- Keep the file list deterministic and ordered so timeline output is stable across runs.
- Add tests that cover a folder containing both `IntuneManagementExtension.log` and `AppWorkload.log` so the regression cannot return.

Primary files:

- `src-tauri/src/commands/intune.rs`
- `src-tauri/src/intune/models.rs`

## Phase 2: Show file provenance in the Intune UI

Goal: let users see which source log produced each event and confirm which files were included in a folder analysis.

- Extend the frontend `IntuneAnalysisResult` type to include `sourceFiles`.
- Store `sourceFiles` in the Intune Zustand store.
- Show included files in the summary view for folder-based analysis.
- Add source-file labels or a dedicated column in the timeline so events from `AppWorkload.log` and `AgentExecutor.log` are distinguishable.

Primary files:

- `src/types/intune.ts`
- `src/stores/intune-store.ts`
- `src/components/intune/IntuneDashboard.tsx`
- `src/components/intune/EventTimeline.tsx`

## Phase 3: Improve high-value IME event extraction

Goal: make the extra files count by recognizing the event families they contain.

- Add explicit patterns for `AppWorkload.log` download, staging, and install events.
- Add policy and assignment tracking from `AppActionProcessor.log`.
- Add script execution and exit-code emphasis for `AgentExecutor.log`.
- Add remediation scheduling signals from `HealthScripts.log`.
- Revisit summary counters so newer event types are not hidden inside generic buckets.

Primary files:

- `src-tauri/src/intune/event_tracker.rs`
- `src-tauri/src/intune/download_stats.rs`
- `src-tauri/src/intune/timeline.rs`

## Phase 4: Resume parser expansion

Once the IME folder workflow is correct, the next parser work should be:

1. CBS/Panther format family.
2. WindowsUpdate.log high-precision timestamp support.
3. SetupAPI section-based logs.

This order matches the current known-source menu and gives the app better coverage for Intune, Autopilot, servicing, and Windows setup triage.

## Validation

When the next slice is implemented, validate with:

1. Frontend build.
2. Rust test suite.
3. Manual folder analysis against a real IME logs directory containing both `IntuneManagementExtension.log` and at least one sidecar log such as `AppWorkload.log`.
