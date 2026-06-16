<#
.SYNOPSIS
    Waltzing installer for Windows (x86_64).

.DESCRIPTION
    Downloads the waltzing compiler, LSP and MCP binaries from the latest
    GitHub release and installs them to a local bin directory, adding it to
    your user PATH.

.EXAMPLE
    irm https://waltzing.awesomike.com/install.ps1 | iex

.EXAMPLE
    # specific version / binary / dir
    .\install.ps1 -Version v0.2.81 -Binary all -InstallDir "$env:LOCALAPPDATA\Programs\waltzing\bin"
#>
[CmdletBinding()]
param(
    [string]$Version = "latest",                                       # "latest" or "v0.2.81"
    [ValidateSet("compiler", "lsp", "mcp", "all")]
    [string]$Binary = "all",
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\waltzing\bin",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$Platform = "windows-x86_64"
$Repo = "awesomike/waltzing"

function Info($m) { Write-Host "==> $m" -ForegroundColor Cyan }
function Ok($m)   { Write-Host "==> $m" -ForegroundColor Green }
function Warn($m) { Write-Host "==> $m" -ForegroundColor Yellow }

# tool spec: <release-asset-stem> -> <installed exe name>
$tools = switch ($Binary) {
    "compiler" { @{ "waltzing" = "waltzing.exe" } }
    "lsp"      { @{ "waltzing-lsp" = "waltzing-lsp.exe" } }
    "mcp"      { @{ "waltzing-mcp" = "waltzing-mcp.exe" } }
    "all"      { [ordered]@{ "waltzing" = "waltzing.exe"; "waltzing-lsp" = "waltzing-lsp.exe"; "waltzing-mcp" = "waltzing-mcp.exe" } }
}

# Release assets are named "<tool>-windows-x86_64.exe".
function Asset-Url([string]$tool) {
    if ($Version -eq "latest") {
        "https://github.com/$Repo/releases/latest/download/$tool-$Platform.exe"
    } else {
        "https://github.com/$Repo/releases/download/$Version/$tool-$Platform.exe"
    }
}

Info "Waltzing installer (Windows $Platform)"
Info "Version: $Version"
Info "Install directory: $InstallDir"

if (-not $DryRun) { New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null }

foreach ($tool in $tools.Keys) {
    $url = Asset-Url $tool
    $dest = Join-Path $InstallDir $tools[$tool]
    if ($DryRun) { Write-Host "  Would download: $url -> $dest"; continue }
    Info "Downloading $tool ..."
    Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
    Ok "Installed $($tools[$tool]) -> $dest"
}

if ($DryRun) { return }

# Add InstallDir to the user PATH (idempotent).
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ';') -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
    Warn "Added $InstallDir to your user PATH — restart your terminal for it to take effect."
}

Ok "Installation complete."
Write-Host ""
Write-Host "Verify with:  waltzing --version"
