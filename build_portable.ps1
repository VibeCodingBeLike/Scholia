# build_portable.ps1
# Builds TheBestMLAWriter in Portable / Testing mode
param(
    [switch]$RunAfterBuild
)

$ErrorActionPreference = "Stop"
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "  TheBestMLAWriter - Portable / Testing Build    " -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

# 1. Compile portable release binary
Write-Host "`n[1/3] Compiling optimized portable binary with Cargo..." -ForegroundColor Yellow
cargo build --profile portable

$ExeSource = "target\portable\the_best_mla_writer.exe"
if (-not (Test-Path $ExeSource)) {
    # Fallback to standard release if portable profile not targeted
    $ExeSource = "target\release\the_best_mla_writer.exe"
    if (-not (Test-Path $ExeSource)) {
        cargo build --release
    }
}

# 2. Assemble portable distribution directory
$DistDir = "dist\TheBestMLAWriter-Portable"
Write-Host "`n[2/3] Assembling portable directory at $DistDir..." -ForegroundColor Yellow
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

Copy-Item $ExeSource -Destination "$DistDir\the_best_mla_writer.exe" -Force
Copy-Item "README.md" -Destination "$DistDir\README.md" -Force
Copy-Item "LICENSE" -Destination "$DistDir\LICENSE" -Force
if (Test-Path "sample_mla_paper.mladoc") {
    Copy-Item "sample_mla_paper.mladoc" -Destination "$DistDir\sample_mla_paper.mladoc" -Force
}

# 3. Create zip archive
Write-Host "`n[3/3] Creating portable ZIP package..." -ForegroundColor Yellow
$ZipTarget = "dist\TheBestMLAWriter-v0.1.0-portable-windows-x64.zip"
if (Test-Path $ZipTarget) {
    Remove-Item -Force $ZipTarget
}
Compress-Archive -Path "$DistDir\*" -DestinationPath $ZipTarget -Force

Write-Host "`n=================================================" -ForegroundColor Green
Write-Host " Portable Build Complete!" -ForegroundColor Green
Write-Host "  Runnable Folder: $DistDir\the_best_mla_writer.exe" -ForegroundColor White
Write-Host "  Zip Archive:     $ZipTarget" -ForegroundColor White
Write-Host "=================================================" -ForegroundColor Green

if ($RunAfterBuild) {
    Write-Host "`nLaunching portable application..." -ForegroundColor Cyan
    Start-Process "$DistDir\the_best_mla_writer.exe"
}
