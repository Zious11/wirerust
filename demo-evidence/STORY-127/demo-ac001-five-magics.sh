#!/usr/bin/env bash
# Demo script for AC-001: All 5 magic values detected by content (BC-2.12.011 PC1/Inv1/Inv2).
# Called by VHS tape. Does NOT modify source code or tests.
set -euo pipefail

WDIR="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
BINARY="$WDIR/target/release/wirerust"
EVDIR="/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-127"
DEMO_DIR=$(mktemp -d)

python3 "$EVDIR/build-5-magic-fixtures.py" "$DEMO_DIR"
echo ""
echo "  Extensions: .PCAP .CAP .data .txt .bin .pcap (wrong magic)"
echo "  All non-standard extensions except 'reject.pcap' which has wrong magic"
echo ""
echo "--- Analyzing directory ---"
"$BINARY" analyze "$DEMO_DIR" --no-color 2>&1 || true
echo ""
echo "--- 'Skipped: 5 packets (decode errors)' = all 5 magic variants detected ---"
echo "--- reject.pcap silently excluded (wrong magic, never passed to reader)   ---"

rm -rf "$DEMO_DIR"
