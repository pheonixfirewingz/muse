# --- PowerShell 7 Script: DownloadAudio.ps1 ---

# Stop on error
$ErrorActionPreference = "Stop"

# Get the directory where this script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

# Path to the text file containing URLs (one per line)
$InputFile = Join-Path $ScriptDir "urls.txt"

# Path to the yt-audio CLI executable (built from your Rust project)
$CliPath = Join-Path $ScriptDir "YtDownloader.exe"

# Directory to save all downloaded files
$MusicDir = Join-Path $ScriptDir "music"

# Max number of concurrent downloads (handled by CLI)
$MaxConcurrent = 8

# Make sure the music directory exists
if (-not (Test-Path $MusicDir)) {
    New-Item -ItemType Directory -Path $MusicDir | Out-Null
    Write-Host "📁 Created output directory: $MusicDir" -ForegroundColor Yellow
}

# Read URLs from file
$Urls = Get-Content $InputFile | Where-Object { $_.Trim() -ne "" }

if (-not $Urls -or $Urls.Count -eq 0) {
    Write-Host "❌ No URLs found in $InputFile" -ForegroundColor Red
    exit 1
}

# Build CLI arguments
# Example: yt-audio download --output "./music" --max-concurrent 8 <url1> <url2> ...
$Args = @("download", "--output", $MusicDir, "--max-concurrent", $MaxConcurrent) + $Urls

Write-Host "▶ Starting yt-audio with $($Urls.Count) URLs (max $MaxConcurrent concurrent)..." -ForegroundColor Cyan
Write-Host ""

# Run the yt-audio CLI
Start-Process -FilePath $CliPath `
    -ArgumentList $Args `
    -NoNewWindow `
    -Wait

Write-Host ""
Write-Host "✅ All downloads completed! Saved in: $MusicDir" -ForegroundColor Green
