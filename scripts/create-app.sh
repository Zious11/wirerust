#!/bin/bash
set -euo pipefail

# create-app.sh - Create a macOS .app bundle
# Usage: ./scripts/create-app.sh <binary-path> <version> <output-dir>

BINARY_PATH="${1:?Usage: create-app.sh <binary-path> <version> <output-dir>}"
VERSION="${2:?Version required}"
OUTPUT_DIR="${3:?Output directory required}"

if [ ! -f "$BINARY_PATH" ]; then
  echo "Error: binary not found: $BINARY_PATH" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_BUNDLE="${OUTPUT_DIR}/Wirerust.app"

rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/wirerust"
chmod +x "$APP_BUNDLE/Contents/MacOS/wirerust"

sed "s/VERSION_PLACEHOLDER/$VERSION/g" "$PROJECT_ROOT/packaging/Info.plist" \
  > "$APP_BUNDLE/Contents/Info.plist"

echo "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

echo "Created app bundle: $APP_BUNDLE"
