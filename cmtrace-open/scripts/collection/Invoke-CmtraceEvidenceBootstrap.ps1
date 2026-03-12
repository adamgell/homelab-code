[CmdletBinding()]
param(
    [string]$StagingRoot = (Join-Path $env:ProgramData 'CmtraceOpen\Staging'),
    [string]$StateRoot = (Join-Path $env:ProgramData 'CmtraceOpen\State'),
    [string]$OutputRoot = (Join-Path $env:ProgramData 'CmtraceOpen\Evidence'),
    [string]$TaskName = 'CmtraceOpen-EvidenceCollection-Once',
    [int]$DelayMinutes = 2,
    [int]$ThrottleHours = 24,
    [string]$CollectorScriptUrl = 'https://example.invalid/cmtrace-open/Invoke-CmtraceEvidenceCollection.ps1',
    [string]$CollectorProfileUrl = 'https://example.invalid/cmtrace-open/intune-evidence-profile.json',
    [string]$SasUrl = '',
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

function Validate-StagedPayloads {
    param(
        [Parameter(Mandatory = $true)]
        [string]$CollectorPath,
        [Parameter(Mandatory = $true)]
        [string]$ProfilePath
    )

    if (-not (Test-PowerShellFile -Path $CollectorPath)) {
        throw "Downloaded collector payload is not valid PowerShell: $CollectorPath. Check CollectorScriptUrl."
    }

    if (-not (Test-JsonFile -Path $ProfilePath)) {
        throw "Downloaded profile payload is not valid JSON: $ProfilePath. Check CollectorProfileUrl."
    }
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
Validate-StagedPayloads -CollectorPath $stagedCollectorPath -ProfilePath $stagedProfilePath

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