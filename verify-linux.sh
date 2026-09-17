#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

echo "Smart Library v0.3.4 - Linux verification"

for command_name in cargo rustc pkg-config; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Λείπει το $command_name. Τρέξε πρώτα ./setup-linux.sh" >&2
    exit 1
  fi
done

for package_name in gtk+-3.0 webkit2gtk-4.1 openssl dbus-1; do
  if ! pkg-config --exists "$package_name"; then
    echo "Λείπει το Linux build dependency: $package_name" >&2
    echo "Τρέξε πρώτα ./setup-linux.sh" >&2
    exit 1
  fi
done

echo "Έλεγχος μορφοποίησης Rust..."
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

echo "Έλεγχος Rust/Tauri project..."
cargo check --manifest-path src-tauri/Cargo.toml

echo "Εκτέλεση Rust unit tests..."
cargo test --manifest-path src-tauri/Cargo.toml

if command -v node >/dev/null 2>&1; then
  echo "Έλεγχος JavaScript..."
  node --check frontend/app.js
fi

echo "Όλοι οι διαθέσιμοι έλεγχοι Linux ολοκληρώθηκαν επιτυχώς."
