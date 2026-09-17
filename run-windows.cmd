@echo off
cd /d "%~dp0"
where cargo >nul 2>&1
if errorlevel 1 (
  echo Rust/Cargo was not found. Run setup-windows.cmd first.
  exit /b 1
)
cargo tauri --version >nul 2>&1
if errorlevel 1 cargo install tauri-cli --version "^2" --locked
if errorlevel 1 exit /b %ERRORLEVEL%
cargo tauri dev
exit /b %ERRORLEVEL%
