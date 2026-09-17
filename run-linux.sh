#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Δεν βρέθηκε το Rust/Cargo. Τρέξε πρώτα ./setup-linux.sh" >&2
  exit 1
fi

if ! cargo tauri --version >/dev/null 2>&1; then
  cargo install tauri-cli --version '^2' --locked
fi

cargo tauri dev
