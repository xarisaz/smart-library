#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust/Cargo δεν βρέθηκε. Εγκατέστησε Rust με rustup πρώτα." >&2
  exit 1
fi

if ! cargo tauri --version >/dev/null 2>&1; then
  cargo install tauri-cli --version '^2' --locked
fi

"$PROJECT_DIR/verify-linux.sh"
cargo tauri build

echo
echo "Linux bundles:"
find src-tauri/target/release/bundle -maxdepth 3 -type f \( -name '*.deb' -o -name '*.AppImage' \) -print 2>/dev/null || true
