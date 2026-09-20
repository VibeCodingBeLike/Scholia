# build_release.ps1
# Builds TheBestMLAWriter for Release (both Portable and Installer)

$ErrorActionPreference = "Stop"
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "  TheBestMLAWriter - Production Release Build    " -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

# 1. Run cargo tests first to verify integrity
Write-Host "`n[1/4] Running automated test suite..." -ForegroundColor Yellow
cargo test --release

# 2. Build release binary with LTO & strip
Write-Host "`n[2/4] Building release binary..." -ForegroundColor Yellow
cargo build --release

# 3. Package portable version
Write-Host "`n[3/4] Packaging portable version..." -ForegroundColor Yellow
& ".\build_portable.ps1"

# 4. Check for Inno Setup compiler (ISCC.exe)
Write-Host "`n[4/4] Checking for Inno Setup Compiler..." -ForegroundColor Yellow
$InnoCandidates = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "iscc"
)

$IsccPath = $null
foreach ($cand in $InnoCandidates) {
    if (Get-Command $cand -ErrorAction SilentlyContinue) {
        $IsccPath = $cand
        break
    } elseif (Test-Path $cand) {
        $IsccPath = $cand
        break
    }
}

if ($IsccPath) {
    Write-Host "Compiling Windows installer with $IsccPath..." -ForegroundColor Green
    & "$IsccPath" "installers\windows\installer.iss"
    Write-Host "Installer created successfully in installers\windows\Output\" -ForegroundColor Green
} else {
    Write-Host "Note: Inno Setup (ISCC.exe) was not found in standard paths." -ForegroundColor Yellow
    Write-Host "The Inno Setup script is prepared at: installers\windows\installer.iss" -ForegroundColor White
    Write-Host "To generate the .exe installer, download Inno Setup (https://jrsoftware.org/isinfo.php) and compile installer.iss." -ForegroundColor White
}

Write-Host "`nRelease build process complete!" -ForegroundColor Green
