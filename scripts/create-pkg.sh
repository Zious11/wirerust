#!/bin/bash
set -euo pipefail

# create-pkg.sh - Create a signed macOS pkg installer
# Usage: ./scripts/create-pkg.sh <binary-path> <version> <signing-identity> <output-path>

BINARY_PATH="${1:?Usage: create-pkg.sh <binary-path> <version> <signing-identity> <output-path>}"
VERSION="${2:?Version required}"
SIGNING_IDENTITY="${3:?Signing identity required}"
OUTPUT_PATH="${4:?Output path required}"

if [ ! -f "$BINARY_PATH" ]; then
  echo "Error: binary not found: $BINARY_PATH" >&2
  exit 1
fi

STAGING_DIR=$(mktemp -d)
trap 'rm -rf "$STAGING_DIR"' EXIT

mkdir -p "$STAGING_DIR/usr/local/bin"
cp "$BINARY_PATH" "$STAGING_DIR/usr/local/bin/wirerust"
chmod +x "$STAGING_DIR/usr/local/bin/wirerust"

pkgbuild --root "$STAGING_DIR" \
  --identifier com.arcavenae.wirerust \
  --version "$VERSION" \
  --install-location / \
  --sign "$SIGNING_IDENTITY" \
  "$OUTPUT_PATH"

echo "Created pkg: $OUTPUT_PATH"
