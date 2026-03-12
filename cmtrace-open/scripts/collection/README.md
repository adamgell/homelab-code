# Evidence Collection

This folder contains a dependency-light PowerShell collector for building a local evidence bundle that matches the tracked cmtrace-open evidence template as closely as practical.

## Files

- `Invoke-CmtraceEvidenceCollection.ps1`: collects curated logs, registry exports, event-log exports, and command output into a bundle, writes `manifest.json` and `notes.md`, compresses the bundle, and can optionally upload the zip to Azure Blob Storage with a SAS URL.
- `Invoke-CmtraceEvidenceBootstrap.ps1`: stages the collector and profile locally, accepts a direct SAS URL for upload, and registers a one-time `SYSTEM` scheduled task to run the collector outside the assignment process.
- `intune-evidence-profile.json`: curated collection profile consumed by the script.

## Intended execution model

- Windows endpoint execution under Intune or another local management runner.
- No Azure PowerShell modules.
- No external dependencies beyond built-in PowerShell cmdlets and native Windows tools such as `reg.exe`, `wevtutil.exe`, and `dsregcmd.exe`.

## Examples

Local-only collection:

```powershell
pwsh.exe -NoProfile -ExecutionPolicy Bypass -File .\Invoke-CmtraceEvidenceCollection.ps1
```

Local-only collection to a custom root:

```powershell
pwsh.exe -NoProfile -ExecutionPolicy Bypass -File .\Invoke-CmtraceEvidenceCollection.ps1 -OutputRoot 'C:\ProgramData\CmtraceOpen\Evidence' -CaseReference 'INC-12345'
```

Upload to a blob SAS URL that already includes the target zip name:

```powershell
pwsh.exe -NoProfile -ExecutionPolicy Bypass -File .\Invoke-CmtraceEvidenceCollection.ps1 -SasUrl 'https://account.blob.core.windows.net/evidence/cmtrace-case.zip?<sas>'
```

Upload to a container or virtual-folder SAS URL and let the script append a blob name:

```powershell
pwsh.exe -NoProfile -ExecutionPolicy Bypass -File .\Invoke-CmtraceEvidenceCollection.ps1 -SasUrl 'https://account.blob.core.windows.net/evidence/intune?<sas>' -BlobName 'collections/cmtrace-case.zip'
```

Force local-only behavior even if a SAS URL is supplied:

```powershell
pwsh.exe -NoProfile -ExecutionPolicy Bypass -File .\Invoke-CmtraceEvidenceCollection.ps1 -SasUrl 'https://account.blob.core.windows.net/evidence?<sas>' -LocalOnly
```

Bootstrap a one-time scheduled collection using explicit HTTPS payload URLs and a direct upload SAS URL:

```powershell
pwsh.exe -NoProfile -ExecutionPolicy Bypass -File .\Invoke-CmtraceEvidenceBootstrap.ps1 -CollectorScriptUrl 'https://raw.githubusercontent.com/<owner>/<repo>/<ref>/cmtrace-open/scripts/collection/Invoke-CmtraceEvidenceCollection.ps1' -CollectorProfileUrl 'https://raw.githubusercontent.com/<owner>/<repo>/<ref>/cmtrace-open/scripts/collection/intune-evidence-profile.json' -SasUrl 'https://account.blob.core.windows.net/evidence/container-or-blob?<sas>'
```

## Output shape

The script creates a bundle root like this:

```text
CMTRACE-20260311-153000-DEVICE/
├── manifest.json
├── notes.md
└── evidence/
    ├── logs/
    ├── registry/
    ├── event-logs/
    ├── exports/
    ├── screenshots/
    └── command-output/
```

The resulting zip is created beside the bundle root.

## Upload behavior

- If `-SasUrl` is omitted, the script stays in local-only mode.
- Uploads use HTTPS with `Invoke-WebRequest` and `x-ms-blob-type: BlockBlob`.
- The script does not require storage account keys, Azure CLI, or Azure PowerShell.
- The manifest records the intended upload destination without exposing the SAS query string.
- If upload fails, the local bundle and zip still remain available and the script returns the upload error in its final output object.

## Bootstrap behavior

- `Invoke-CmtraceEvidenceBootstrap.ps1` is intended for assignment-side or other bootstrap execution where you do not want the full collector to run inline.
- The bootstrap downloads the collector and profile from HTTPS URLs into `C:\ProgramData\CmtraceOpen\Staging`.
- The bootstrap accepts a direct upload SAS URL and passes it to the collector scheduled task when not running in local-only mode.
- The bootstrap validates that the staged collector payload parses as PowerShell and that the staged profile parses as JSON before it registers the scheduled task.
- The bootstrap registers a one-time `SYSTEM` scheduled task and writes state to `C:\ProgramData\CmtraceOpen\State\collection-bootstrap.json` so repeated execution can be throttled.
- The bootstrap ships with placeholder URLs on `example.invalid`; pass real HTTPS payload URLs at execution time.
- Use commit-pinned raw GitHub URLs instead of `main` if you want deployment-time payload pinning.

## Collection behavior

- Missing or failed artifacts are recorded in `manifest.json` instead of aborting the whole run.
- Current profile coverage includes curated IME logs; narrow Panther setup logs; MDM, IME, and Autopilot registry exports; curated event channels; targeted supporting files under `evidence/exports`; `dsregcmd /status`; and Delivery Optimization snapshots.
- Autopilot and enrollment-adjacent registry coverage includes these roots when present:
  - `HKLM\SOFTWARE\Microsoft\Provisioning\Diagnostics\Autopilot`
  - `HKLM\SOFTWARE\Microsoft\Provisioning\AutopilotSettings`
  - `HKLM\SOFTWARE\Microsoft\Windows\Autopilot\EnrollmentStatusTracking\ESPTrackingInfo\Diagnostics`
  - `HKLM\SOFTWARE\Microsoft\IntuneManagementExtension\Win32Apps`
  - `HKLM\SOFTWARE\Microsoft\Provisioning\NodeCache\CSP`
  - `HKLM\SOFTWARE\Microsoft\Provisioning\OMADM\SyncML\ODJApplied`
- The curated event channel set currently includes:
  - `Microsoft-Windows-AAD/Operational`
  - `Microsoft-Windows-DeviceManagement-Enterprise-Diagnostics-Provider/Admin`
  - `Microsoft-Windows-DeviceManagement-Enterprise-Diagnostics-Provider/Operational`
  - `Microsoft-Windows-DeliveryOptimization/Operational`
  - `Microsoft-Windows-ModernDeployment-Diagnostics-Provider/Autopilot`
  - `Microsoft-Windows-ModernDeployment-Diagnostics-Provider/ManagementService`
  - `Microsoft-Windows-Provisioning-Diagnostics-Provider/Admin`
  - `Microsoft-Windows-Shell-Core/Operational`
  - `Microsoft-Windows-Time-Service/Operational`
  - `Microsoft-Windows-User Device Registration/Admin`
- Targeted file exports include `AutoPilotConfigurationFile.json`, JSON staged under `C:\Windows\ServiceState\Autopilot`, existing `C:\Users\Public\Documents\MDMDiagnostics` output, and `AutopilotDDSZTDFile.json` when present.
- The collector runs `MdmDiagnosticsTool.exe` during collection and harvests the generated `MDMDiagReport.zip` back into the same bundle so fresh diagnostics land beside any pre-existing `MDMDiagnostics` output.
- Delivery Optimization command capture currently includes `Get-DeliveryOptimizationStatus` and `Get-DeliveryOptimizationPerfSnap` snapshots.
- Enrollment `FirstSync` is not exported separately because the `Enrollments` export already covers that state, and `EstablishedCorrelations` is not duplicated because the Autopilot diagnostics export already includes it.
- The profile can be adjusted later without changing app code.

## Operational notes

- Run under a context that can read the targeted logs, registry paths, and event channels. Intune `SYSTEM` is the primary target.
- Missing Autopilot-only or scenario-specific artifacts are normal on devices that are not in an Autopilot flow, never staged the related JSON, did not produce local `MDMDiagnostics` output yet, or simply do not have the targeted Panther/setup traces. Those cases are recorded in `manifest.json` and do not fail the collection run.
- `Compress-Archive` uses the built-in ZIP implementation and is sufficient for typical evidence bundles. Very large bundles should still be sized with care.
- Use short-lived SAS URLs with write permissions scoped only to the target container or blob path.
- If you use the bootstrap flow, keep the upload SAS short-lived and do not commit live SAS values into repo-tracked bootstrap or profile files.
