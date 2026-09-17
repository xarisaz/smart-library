$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "Smart Library - Windows environment check" -ForegroundColor Cyan

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Rust is not installed." -ForegroundColor Yellow
    Write-Host "Install it from https://rustup.rs and reopen PowerShell."
    exit 1
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Cargo was not found in PATH. Reopen PowerShell after installing Rust." -ForegroundColor Yellow
    exit 1
}

if (-not (Get-Command cargo-tauri -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Tauri CLI 2..." -ForegroundColor Cyan
    cargo install tauri-cli --version "^2.0" --locked
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "Rust:" (rustc --version) -ForegroundColor Green
Write-Host "Cargo:" (cargo --version) -ForegroundColor Green
Write-Host "Tauri CLI is ready." -ForegroundColor Green
Write-Host "Verification: .\verify-windows.cmd"
Write-Host "Run: .\run-windows.cmd"
Write-Host "Installer: .\build-windows.cmd"
