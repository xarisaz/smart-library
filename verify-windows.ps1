$ErrorActionPreference = "Stop"

Set-Location $PSScriptRoot

Write-Host "Smart Library v0.3.4 - Windows verification" -ForegroundColor Cyan

function Assert-NativeCommandSucceeded {
    param([string]$Step)

    if ($LASTEXITCODE -ne 0) {
        Write-Host "$Step failed with exit code $LASTEXITCODE." -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Cargo was not found. Run .\setup-windows.cmd first." -ForegroundColor Yellow
    exit 1
}

Write-Host "Checking application configuration..." -ForegroundColor Cyan
$tauriConfig = Get-Content ".\src-tauri\tauri.conf.json" -Raw | ConvertFrom-Json
$linuxConfig = Get-Content ".\src-tauri\tauri.linux.conf.json" -Raw | ConvertFrom-Json
$windowsConfig = Get-Content ".\src-tauri\tauri.windows.conf.json" -Raw | ConvertFrom-Json
if ($tauriConfig.version -ne "0.3.4") {
    Write-Host "The Tauri version does not match v0.3.4." -ForegroundColor Red
    exit 1
}
if ($windowsConfig.bundle.targets -notcontains "nsis") {
    Write-Host "The Windows NSIS target is missing." -ForegroundColor Red
    exit 1
}
if (($linuxConfig.bundle.targets -notcontains "deb") -or ($linuxConfig.bundle.targets -notcontains "appimage")) {
    Write-Host "The Linux DEB/AppImage targets are missing." -ForegroundColor Red
    exit 1
}

if (Get-Command node -ErrorAction SilentlyContinue) {
    Write-Host "Checking JavaScript syntax..." -ForegroundColor Cyan
    node --check .\frontend\app.js
    Assert-NativeCommandSucceeded "JavaScript syntax check"
}

Write-Host "Formatting Rust sources..." -ForegroundColor Cyan
cargo fmt --manifest-path .\src-tauri\Cargo.toml
Assert-NativeCommandSucceeded "Rust formatting"

Write-Host "Checking Rust formatting..." -ForegroundColor Cyan
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
Assert-NativeCommandSucceeded "Rust formatting check"

Write-Host "Checking the Rust/Tauri project..." -ForegroundColor Cyan
cargo check --manifest-path .\src-tauri\Cargo.toml
Assert-NativeCommandSucceeded "Rust project check"

Write-Host "Running Rust unit tests..." -ForegroundColor Cyan
cargo test --manifest-path .\src-tauri\Cargo.toml
Assert-NativeCommandSucceeded "Rust unit tests"

Write-Host "All available source checks passed." -ForegroundColor Green
