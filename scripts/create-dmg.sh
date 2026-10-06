#!/bin/bash
set -euo pipefail

# create-dmg.sh - Create a macOS .dmg disk image
# Usage: ./scripts/create-dmg.sh <app-bundle-path> <version> <output-path>

APP_PATH="${1:?Usage: create-dmg.sh <app-bundle-path> <version> <output-path>}"
VERSION="${2:?Version required}"
OUTPUT_PATH="${3:?Output path required}"

if [ ! -d "$APP_PATH" ]; then
  echo "Error: app bundle not found: $APP_PATH" >&2
  exit 1
fi

STAGING_DIR=$(mktemp -d)
trap 'rm -rf "$STAGING_DIR"' EXIT

cp -R "$APP_PATH" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"

rm -f "$OUTPUT_PATH"
hdiutil create -volname "Wirerust $VERSION" \
  -srcfolder "$STAGING_DIR" \
  -ov -format UDZO \
  "$OUTPUT_PATH"

echo "Created dmg: $OUTPUT_PATH"
