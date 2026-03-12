[CmdletBinding()]
param(
    [string]$StagingRoot = (Join-Path $env:ProgramData 'CmtraceOpen\Staging'),
    [string]$StateRoot = (Join-Path $env:ProgramData 'CmtraceOpen\State'),
    [string]$OutputRoot = (Join-Path $env:ProgramData 'CmtraceOpen\Evidence'),
    [string]$TaskName = 'CmtraceOpen-EvidenceCollection-Once',
    [int]$DelayMinutes = 2,
    [int]$ThrottleHours = 24,
    [string]$CollectorProfileUrl = '', #fill out
    [string]$CollectorScriptUrl = '', #fill out
    [string]$SasUrl = '', #fill out
    [string]$BundleLabel = 'intune-endpoint-evidence',
    [string]$CaseReference = '',
    [string]$BlobName = '',
    [string]$OperatorName = 'SYSTEM',
    [string]$OperatorTeam = 'Intune',
    [string]$OperatorContact = '',
    [switch]$LocalOnly,
    [switch]$Force
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Get-UtcTimestamp {
    return (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
}

function Ensure-Directory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -Path $Path -ItemType Directory -Force | Out-Null
    }
}

function Write-JsonFile {
    param(
        [Parameter(Mandatory = $true)]
        [object]$InputObject,
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $utf8Encoding = New-Object System.Text.UTF8Encoding($false)
    $json = $InputObject | ConvertTo-Json -Depth 10
    [System.IO.File]::WriteAllText($Path, $json, $utf8Encoding)
}

function Read-JsonFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $null
    }

    return (Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json -Depth 10)
}

function Quote-TaskArgument {
    param(
        [AllowEmptyString()]
        [string]$Value
    )

    return '"{0}"' -f ($Value -replace '"', '\"')
}

function Test-HttpsUrl {
    param(
        [AllowEmptyString()]
        [string]$Value
    )

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return $false
    }

    $uri = $null
    if (-not [System.Uri]::TryCreate($Value, [System.UriKind]::Absolute, [ref]$uri)) {
        return $false
    }

    return $uri.Scheme -eq 'https'
}

function Test-PlaceholderUrl {
    param(
        [AllowEmptyString()]
        [string]$Value
    )

    return $Value -like 'https://example.invalid/*'
}

function Download-File {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Url,
        [Parameter(Mandatory = $true)]
        [string]$DestinationPath
    )

    if (-not (Test-HttpsUrl -Value $Url)) {
        throw "Only HTTPS URLs are allowed: $Url"
    }

    Invoke-WebRequest -Uri $Url -OutFile $DestinationPath -UseBasicParsing
}

function Get-PowerShellExecutable {
    $pwshCommand = Get-Command pwsh.exe -ErrorAction SilentlyContinue
    if ($pwshCommand -and -not [string]::IsNullOrWhiteSpace($pwshCommand.Source)) {
        return $pwshCommand.Source
    }

    foreach ($candidatePath in @(
            (Join-Path ${env:ProgramFiles} 'PowerShell\7\pwsh.exe'),
            (Join-Path ${env:ProgramFiles(x86)} 'PowerShell\7\pwsh.exe')
        )) {
        if ($candidatePath -and (Test-Path -LiteralPath $candidatePath -PathType Leaf)) {
            return $candidatePath
        }
    }

    throw 'pwsh.exe was not found. Install PowerShell 7 or update the bootstrap to point at a valid pwsh path.'
}

function Test-PowerShellFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $parseErrors = $null
    $tokens = $null
    [System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$tokens, [ref]$parseErrors) | Out-Null
    return ($parseErrors.Count -eq 0)
}

function Test-JsonFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    try {
        Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json -Depth 20 | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

function Get-RedactedUrl {
    param(
        [AllowEmptyString()]
        [string]$Value
    )

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return '[not provided]'
    }

    $uri = $null
    if (-not [System.Uri]::TryCreate($Value, [System.UriKind]::Absolute, [ref]$uri)) {
        return $Value
    }

    if ([string]::IsNullOrWhiteSpace($uri.Query)) {
        return $uri.AbsoluteUri
    }

    return ('{0} [query redacted]' -f $uri.GetLeftPart([System.UriPartial]::Path))
}

function Get-FilePreview {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    try {
        $rawContent = Get-Content -LiteralPath $Path -Raw -ErrorAction Stop
    }
    catch {
        return ('[preview unavailable: {0}]' -f $_.Exception.Message)
    }

    if ([string]::IsNullOrWhiteSpace($rawContent)) {
        return '[empty file]'
    }

    $normalizedPreview = (($rawContent -replace [char]0xFEFF, '') -replace '\r?\n', ' ')
    $normalizedPreview = ($normalizedPreview -replace '\s+', ' ').Trim()

    if ($normalizedPreview.Length -gt 200) {
        return ('{0}...' -f $normalizedPreview.Substring(0, 200))
    }

    return $normalizedPreview
}

function Get-JsonPayloadHint {
    param(
        [AllowEmptyString()]
        [string]$Preview
    )

    if ([string]::IsNullOrWhiteSpace($Preview) -or $Preview -eq '[empty file]') {
        return 'Payload is empty.'
    }

    $trimmedPreview = $Preview.TrimStart()
    if ($trimmedPreview.StartsWith('<')) {
        return 'Payload preview suggests HTML or XML content rather than JSON.'
    }

    if ((-not $trimmedPreview.StartsWith('{')) -and (-not $trimmedPreview.StartsWith('['))) {
        return 'Payload preview suggests plain text or another non-JSON format.'
    }

    return 'Payload could not be parsed as JSON.'
}

function Assert-ValidJsonFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$SourceContext
    )

    try {
        $rawContent = Get-Content -LiteralPath $Path -Raw -ErrorAction Stop
    }
    catch {
        throw ('Downloaded profile payload could not be read. Staged path: {0}. Source: {1}. Error: {2}' -f $Path, (Get-RedactedUrl -Value $SourceContext), $_.Exception.Message)
    }

    if ([string]::IsNullOrWhiteSpace($rawContent)) {
        throw ('Downloaded profile payload is empty. Staged path: {0}. Source: {1}.' -f $Path, (Get-RedactedUrl -Value $SourceContext))
    }

    try {
        $rawContent | ConvertFrom-Json -Depth 20 -ErrorAction Stop | Out-Null
    }
    catch {
        $preview = Get-FilePreview -Path $Path
        $payloadHint = Get-JsonPayloadHint -Preview $preview
        throw ('Downloaded profile payload is not valid JSON. Staged path: {0}. Source: {1}. {2} Parse error: {3}. Payload preview: {4}' -f $Path, (Get-RedactedUrl -Value $SourceContext), $payloadHint, $_.Exception.Message, $preview)
    }
}

function Validate-StagedPayloads {
    param(
        [Parameter(Mandatory = $true)]
        [string]$CollectorPath,
        [Parameter(Mandatory = $true)]
        [string]$ProfilePath,
        [Parameter(Mandatory = $true)]
        [string]$ProfileSource
    )

    if (-not (Test-PowerShellFile -Path $CollectorPath)) {
        throw "Downloaded collector payload is not valid PowerShell: $CollectorPath. Check CollectorScriptUrl."
    }

    Assert-ValidJsonFile -Path $ProfilePath -SourceContext $ProfileSource
}

function Get-Task {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    return Get-ScheduledTask -TaskName $Name -ErrorAction SilentlyContinue
}

function Remove-TaskIfPresent {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    $existingTask = Get-Task -Name $Name
    if ($existingTask) {
        Unregister-ScheduledTask -TaskName $Name -Confirm:$false
    }
}

function Get-ShouldThrottle {
    param(
        [Parameter(Mandatory = $true)]
        [string]$StatePath,
        [Parameter(Mandatory = $true)]
        [int]$WindowHours,
        [switch]$IgnoreState
    )

    if ($IgnoreState) {
        return $false
    }

    $state = Read-JsonFile -Path $StatePath
    if ($null -eq $state) {
        return $false
    }

    if ([string]::IsNullOrWhiteSpace([string]$state.registeredUtc)) {
        return $false
    }

    $registeredUtc = [datetime]::Parse([string]$state.registeredUtc).ToUniversalTime()
    $expiresUtc = $registeredUtc.AddHours($WindowHours)
    return $expiresUtc -gt (Get-Date).ToUniversalTime()
}

function New-CollectorArgumentString {
    param(
        [Parameter(Mandatory = $true)]
        [string]$CollectorPath,
        [Parameter(Mandatory = $true)]
        [string]$ProfilePath,
        [Parameter(Mandatory = $true)]
        [string]$CollectorOutputRoot,
        [Parameter(Mandatory = $true)]
        [string]$CollectorBundleLabel,
        [AllowEmptyString()]
        [string]$CollectorCaseReference,
        [AllowEmptyString()]
        [string]$CollectorBlobName,
        [AllowEmptyString()]
        [string]$CollectorOperatorName,
        [AllowEmptyString()]
        [string]$CollectorOperatorTeam,
        [AllowEmptyString()]
        [string]$CollectorOperatorContact,
        [AllowEmptyString()]
        [string]$ResolvedSasUrl,
        [switch]$RunLocalOnly
    )

    $arguments = New-Object System.Collections.Generic.List[string]
    $arguments.Add('-NoProfile')
    $arguments.Add('-ExecutionPolicy')
    $arguments.Add('Bypass')
    $arguments.Add('-File')
    $arguments.Add((Quote-TaskArgument -Value $CollectorPath))
    $arguments.Add('-CollectorProfilePath')
    $arguments.Add((Quote-TaskArgument -Value $ProfilePath))
    $arguments.Add('-OutputRoot')
    $arguments.Add((Quote-TaskArgument -Value $CollectorOutputRoot))
    $arguments.Add('-BundleLabel')
    $arguments.Add((Quote-TaskArgument -Value $CollectorBundleLabel))
    $arguments.Add('-OperatorName')
    $arguments.Add((Quote-TaskArgument -Value $CollectorOperatorName))
    $arguments.Add('-OperatorTeam')
    $arguments.Add((Quote-TaskArgument -Value $CollectorOperatorTeam))

    if (-not [string]::IsNullOrWhiteSpace($CollectorCaseReference)) {
        $arguments.Add('-CaseReference')
        $arguments.Add((Quote-TaskArgument -Value $CollectorCaseReference))
    }

    if (-not [string]::IsNullOrWhiteSpace($CollectorBlobName)) {
        $arguments.Add('-BlobName')
        $arguments.Add((Quote-TaskArgument -Value $CollectorBlobName))
    }

    if (-not [string]::IsNullOrWhiteSpace($CollectorOperatorContact)) {
        $arguments.Add('-OperatorContact')
        $arguments.Add((Quote-TaskArgument -Value $CollectorOperatorContact))
    }

    if ($RunLocalOnly) {
        $arguments.Add('-LocalOnly')
    }
    elseif (-not [string]::IsNullOrWhiteSpace($ResolvedSasUrl)) {
        $arguments.Add('-SasUrl')
        $arguments.Add((Quote-TaskArgument -Value $ResolvedSasUrl))
    }

    return ($arguments -join ' ')
}

$statePath = Join-Path $StateRoot 'collection-bootstrap.json'
$stagedCollectorPath = Join-Path $StagingRoot 'Invoke-CmtraceEvidenceCollection.ps1'
$stagedProfilePath = Join-Path $StagingRoot 'intune-evidence-profile.json'

Write-Step 'Preparing bootstrap directories'
Ensure-Directory -Path $StagingRoot
Ensure-Directory -Path $StateRoot
Ensure-Directory -Path $OutputRoot

if (Test-PlaceholderUrl -Value $CollectorScriptUrl) {
    throw 'CollectorScriptUrl still points to the example.invalid placeholder. Provide a reachable HTTPS URL for the collector payload.'
}

if (Test-PlaceholderUrl -Value $CollectorProfileUrl) {
    throw 'CollectorProfileUrl still points to the example.invalid placeholder. Provide a reachable HTTPS URL for the collector profile.'
}

if ((-not $LocalOnly) -and [string]::IsNullOrWhiteSpace($SasUrl)) {
    throw 'SasUrl is required unless you use -LocalOnly.'
}

if ((-not $LocalOnly) -and (-not (Test-HttpsUrl -Value $SasUrl))) {
    throw 'SasUrl must be an HTTPS URL.'
}

if ((-not $LocalOnly) -and ($SasUrl -notmatch '\?')) {
    throw 'SasUrl does not appear to contain a query string.'
}

if (Get-ShouldThrottle -StatePath $statePath -WindowHours $ThrottleHours -IgnoreState:$Force) {
    $existingTask = Get-Task -Name $TaskName
    $status = if ($existingTask) { 'skipped-throttled-task-present' } else { 'skipped-throttled' }
    [pscustomobject]@{
        Status    = $status
        TaskName  = $TaskName
        StatePath = $statePath
        Message   = 'Bootstrap skipped because the throttle window is still active.'
    }
    exit 0
}

Write-Step 'Downloading staged collector payloads'
Download-File -Url $CollectorScriptUrl -DestinationPath $stagedCollectorPath
Download-File -Url $CollectorProfileUrl -DestinationPath $stagedProfilePath

Write-Step 'Validating staged collector payloads'
Validate-StagedPayloads -CollectorPath $stagedCollectorPath -ProfilePath $stagedProfilePath -ProfileSource $CollectorProfileUrl

$resolvedSasUrl = $SasUrl

$caseReferenceValue = if ([string]::IsNullOrWhiteSpace($CaseReference)) {
    'bootstrap-{0}' -f (Get-Date -Format 'yyyyMMdd-HHmmss')
}
else {
    $CaseReference
}

$taskArguments = New-CollectorArgumentString -CollectorPath $stagedCollectorPath -ProfilePath $stagedProfilePath -CollectorOutputRoot $OutputRoot -CollectorBundleLabel $BundleLabel -CollectorCaseReference $caseReferenceValue -CollectorBlobName $BlobName -CollectorOperatorName $OperatorName -CollectorOperatorTeam $OperatorTeam -CollectorOperatorContact $OperatorContact -ResolvedSasUrl $resolvedSasUrl -RunLocalOnly:$LocalOnly
$powerShellExecutable = Get-PowerShellExecutable

Write-Step 'Registering one-time SYSTEM scheduled task'
if ($Force) {
    Remove-TaskIfPresent -Name $TaskName
}
elseif (Get-Task -Name $TaskName) {
    Remove-TaskIfPresent -Name $TaskName
}

$triggerTime = (Get-Date).AddMinutes($DelayMinutes)
$taskAction = New-ScheduledTaskAction -Execute $powerShellExecutable -Argument $taskArguments
$taskTrigger = New-ScheduledTaskTrigger -Once -At $triggerTime
$taskSettings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Hours 8) -StartWhenAvailable

Register-ScheduledTask -TaskName $TaskName -Action $taskAction -Trigger $taskTrigger -User 'SYSTEM' -RunLevel Highest -Settings $taskSettings -Force | Out-Null

$state = [ordered]@{
    registeredUtc        = Get-UtcTimestamp
    triggerUtc           = $triggerTime.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    taskName             = $TaskName
    stagingRoot          = $StagingRoot
    outputRoot           = $OutputRoot
    collectorScriptUrl   = $CollectorScriptUrl
    collectorProfileUrl  = $CollectorProfileUrl
    powerShellExecutable = $powerShellExecutable
    sasUrlConfigured     = (-not [string]::IsNullOrWhiteSpace($SasUrl))
    localOnly            = [bool]$LocalOnly
    caseReference        = $caseReferenceValue
}

Write-JsonFile -InputObject $state -Path $statePath

Write-Step 'Bootstrap complete'
[pscustomobject]@{
    Status               = 'scheduled'
    TaskName             = $TaskName
    TriggerTimeUtc       = $triggerTime.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    StatePath            = $statePath
    StagedCollectorPath  = $stagedCollectorPath
    StagedProfilePath    = $stagedProfilePath
    PowerShellExecutable = $powerShellExecutable
    SasUrlConfigured     = (-not [string]::IsNullOrWhiteSpace($SasUrl))
    OutputRoot           = $OutputRoot
}