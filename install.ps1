# ─────────────────────────────────────────────────────────────────────────────
# jobsense-parker installer — Windows (PowerShell 5.1+)
#
# Usage (PowerShell):
#   irm https://raw.githubusercontent.com/arpanpathak/jobsense-parker/master/install.ps1 | iex
#   irm .../install.ps1 | iex -   # with a pinned version:
#   $v = "v0.4.0"; irm .../install.ps1 -Body @{ version = $v } | iex
#
# Downloads the prebuilt x86_64 Windows binary from the GitHub Release,
# installs it to %USERPROFILE%\.local\bin (or $env:JOBSENSE_INSTALL_DIR),
# and prints PATH help.
# ─────────────────────────────────────────────────────────────────────────────
[CmdletBinding()]
param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

$Repo = "arpanpathak/jobsense-parker"
$InstallDir = if ($env:JOBSENSE_INSTALL_DIR) {
    $env:JOBSENSE_INSTALL_DIR
} else {
    Join-Path $HOME ".local\bin"
}

$Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
    Write-Error "ARM64 Windows binaries are not built yet. Use x64 Windows."
} else {
    "x86_64"
}

# ── Resolve download URL ─────────────────────────────────────────────────────
# GitHub's `/releases/latest/download/<asset>` redirect serves the newest
# release's asset without any API call (the API is rate-limited when
# unauthenticated and can return 403).
$Asset = "jobsense-parker-$Arch-pc-windows-msvc.zip"
$Url = if ($Version -eq "latest") {
    "https://github.com/$Repo/releases/latest/download/$Asset"
} else {
    "https://github.com/$Repo/releases/download/$Version/$Asset"
}
$TmpZip = Join-Path $env:TEMP $Asset

# ── Download + extract ──────────────────────────────────────────────────────
Write-Host "-> Downloading $Asset ($Version)"
Invoke-WebRequest -Uri $Url -OutFile $TmpZip

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$TmpDir = Join-Path $env:TEMP "jobsense-parker-install"
if (Test-Path $TmpDir) { Remove-Item -Recurse -Force $TmpDir }
Expand-Archive -Path $TmpZip -DestinationPath $TmpDir -Force

Copy-Item (Join-Path $TmpDir "jobsense-parker.exe") (Join-Path $InstallDir "jobsense-parker.exe") -Force
Remove-Item -Recurse -Force $TmpDir
Remove-Item -Force $TmpZip

Write-Host "`u{2713} Installed jobsense-parker ($Version) to $InstallDir" -ForegroundColor Green

# ── PATH help ───────────────────────────────────────────────────────────────
$InPath = ($env:Path -split ";" | Where-Object { $_ -eq $InstallDir })
if (-not $InPath) {
    Write-Host ""
    Write-Host "  Add it to your PATH (permanent):"
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', \"$env:Path;$InstallDir\", 'User')"
}

Write-Host ""
Write-Host "  Run 'jobsense-parker' to start hunting."
