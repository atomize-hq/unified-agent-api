[CmdletBinding()]
param(
    [ValidatePattern('^[A-Za-z0-9_-]+$')][string]$Profile,
    [Parameter(Mandatory = $true)][string]$Worktree,
    [Parameter(Mandatory = $true)][string]$Packet,
    [ValidateSet('read-only', 'workspace-write')][string]$Sandbox = 'workspace-write',
    [ValidateRange(1, [int]::MaxValue)][int]$TimeoutSeconds = 3600,
    [string]$OutputLastMessage,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$worktreePath = (Resolve-Path -LiteralPath $Worktree).Path
$packetPath = (Resolve-Path -LiteralPath $Packet).Path

if (-not (Test-Path -LiteralPath (Join-Path $worktreePath '.git'))) {
    throw "Worktree is not a Git worktree: $worktreePath"
}
if ($packetPath.StartsWith($worktreePath + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Packet must be outside the target worktree.'
}

# --sandbox is always passed, so the global sandbox_mode never applies.
$codexArgs = @('exec', '--sandbox', $Sandbox, '--cd', $worktreePath)
if ($Profile) {
    $codexHome = if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $HOME '.codex' }
    $profileFile = Join-Path $codexHome "$Profile.config.toml"
    if (-not (Test-Path -LiteralPath $profileFile -PathType Leaf)) {
        throw "Codex profile overlay does not exist: $profileFile"
    }
    $codexArgs += @('--profile', $Profile)
}
if ($OutputLastMessage) {
    $outputPath = [IO.Path]::GetFullPath($OutputLastMessage)
    $codexArgs += @('--output-last-message', $outputPath)
}
$codexArgs += '-'

$codex = Get-Command codex -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $codex) {
    throw 'codex is not on PATH.'
}

if ($DryRun) {
    Write-Output ('profile: ' + $(if ($Profile) { $Profile } else { '(default configuration)' }))
    Write-Output "packet: $packetPath"
    Write-Output "timeout: $TimeoutSeconds seconds"
    Write-Output ('command: codex ' + ($codexArgs -join ' '))
    exit 0
}

$quotedArgs = $codexArgs | ForEach-Object { '"' + $_ + '"' }
$process = Start-Process -FilePath $codex.Source -ArgumentList $quotedArgs -RedirectStandardInput $packetPath -NoNewWindow -PassThru
$null = $process.Handle  # keep the handle so ExitCode is readable after exit
if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
    try { $process.Kill($true) } catch { $process.Kill() }
    Write-Error "codex did not finish within $TimeoutSeconds seconds and was stopped." -ErrorAction Continue
    exit 124
}
exit $process.ExitCode
