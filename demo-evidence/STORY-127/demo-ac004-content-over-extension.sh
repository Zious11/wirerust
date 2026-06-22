#!/usr/bin/env bash
# Demo script for AC-004: Content-over-extension (BC-2.12.011 headline behavior).
# Called by VHS tape. Does NOT modify source code or tests.
set -euo pipefail

WDIR="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
BINARY="$WDIR/target/release/wirerust"
DEMO_DIR=$(mktemp -d)

echo "=== Demo directory: $DEMO_DIR ==="
echo ""

# pcapng content written to a .cap extension file (C-2 regression fixture)
cp "$WDIR/tests/fixtures/smb3.pcapng" "$DEMO_DIR/capture-renamed-as.cap"
# Wrong-magic .pcap (content mismatch -- silently excluded)
printf '\xDE\xAD\xBE\xEF\x00\x00\x00\x00' > "$DEMO_DIR/imposter-wrong-magic.pcap"

echo "--- Files in demo directory ---"
ls -la "$DEMO_DIR"
echo ""
echo "  capture-renamed-as.cap   : pcapng magic [0A 0D 0D 0A], .cap extension"
echo "  imposter-wrong-magic.pcap: wrong magic [DE AD BE EF], .pcap extension"
echo ""
echo "--- Analyzing directory (content-based routing) ---"
"$BINARY" analyze "$DEMO_DIR" --no-color
echo ""
echo "--- Result: .cap file detected (54 SMB packets); wrong-magic .pcap silently skipped ---"

rm -rf "$DEMO_DIR"
