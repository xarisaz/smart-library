$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "Rust/Cargo δεν βρέθηκε. Τρέξε πρώτα .\setup-windows.ps1 από τον βασικό φάκελο."
}

cargo tauri --version | Out-Null
if ($LASTEXITCODE -ne 0) {
  cargo install tauri-cli --version "^2" --locked
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

& (Join-Path $PSScriptRoot "verify-windows.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo tauri build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`nWindows installer(s):"
Get-ChildItem -Path "src-tauri\target\release\bundle\nsis" -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue | ForEach-Object { $_.FullName }
