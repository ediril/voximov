param([ValidateSet('start','stop','status','logs')][string]$Action)
$ErrorActionPreference = 'Stop'
$projectDirectory = Split-Path -Parent $PSScriptRoot
$studioDirectory = Join-Path $projectDirectory 'build'
$studioExecutable = Join-Path $studioDirectory 'Voximov.exe'
$latestExecutable = if (Test-Path -LiteralPath $studioDirectory -PathType Container) {
    Get-ChildItem -LiteralPath $studioDirectory -Filter 'Voximov-v*.exe' -File |
        Where-Object { $_.BaseName -match '^Voximov-v\d+\.\d+(?:\.\d+)?$' } |
        Sort-Object { [version]($_.BaseName -replace '^Voximov-v', '') } -Descending |
        Select-Object -First 1
}
if ($latestExecutable) { $studioExecutable = $latestExecutable.FullName }
$studioUrl = 'http://127.0.0.1:5174'
$studioLog = Join-Path $env:LOCALAPPDATA 'Voximov\voximov.log'
function Get-StudioStatus {
    try { return Invoke-RestMethod -Uri "$studioUrl/api/health" -TimeoutSec 2 }
    catch { return $null }
}
switch ($Action) {
    'start' {
        $studioStatus = Get-StudioStatus
        if ($studioStatus) { Write-Output "Voximov already running (pid $($studioStatus.pid)): $studioUrl"; exit 0 }
        if (-not (Test-Path -LiteralPath $studioExecutable)) { throw 'Build the app first: bin/build-windows in WSL' }
        Start-Process -FilePath $studioExecutable -WindowStyle Normal | Out-Null
        for ($studioAttempt = 0; $studioAttempt -lt 30; $studioAttempt++) {
            Start-Sleep -Milliseconds 200
            $studioStatus = Get-StudioStatus
            if ($studioStatus) { Write-Output "Voximov started (pid $($studioStatus.pid)): $studioUrl"; exit 0 }
        }
        throw "Voximov did not start. See $studioLog"
    }
    'stop' {
        $studioStatus = Get-StudioStatus
        if (-not $studioStatus) { Write-Output 'Voximov is stopped'; exit 0 }
        Invoke-RestMethod -Method Post -Uri "$studioUrl/api/close" -TimeoutSec 5 | Out-Null
        for ($studioAttempt = 0; $studioAttempt -lt 40; $studioAttempt++) {
            Start-Sleep -Milliseconds 150
            if (-not (Get-StudioStatus)) { Write-Output 'Voximov stopped'; exit 0 }
        }
        Write-Output 'Close requested. Check the app window: save or discard unsaved changes, or finish the current operation.'
    }
    'status' {
        $studioStatus = Get-StudioStatus
        if ($studioStatus) { $studioStatus | Format-List } else { Write-Output 'Voximov is stopped' }
    }
    'logs' {
        if (Test-Path -LiteralPath $studioLog) { Get-Content -LiteralPath $studioLog -Tail 100 -Wait }
        else { Write-Output 'No logs yet. Run: svoice start' }
    }
}
