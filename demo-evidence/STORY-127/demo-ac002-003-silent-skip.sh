#!/usr/bin/env bash
# Demo script for AC-002 + AC-003: Error paths -- silent skip of wrong-magic
# and short files (BC-2.12.011 PC2, PC3, Inv5). Called by VHS tape.
# Does NOT modify source code or tests.
set -euo pipefail

WDIR="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
BINARY="$WDIR/target/release/wirerust"

DEMO_DIR=$(mktemp -d)
printf '\xDE\xAD\xBE\xEF\x00\x00\x00\x00' > "$DEMO_DIR/wrong-magic.pcap"
printf '\x0A\x0D\x0D' > "$DEMO_DIR/short-3-bytes.pcap"

echo "=== AC-002: Directory with only wrong-magic .pcap ==="
WRONG_DIR=$(mktemp -d)
printf '\xDE\xAD\xBE\xEF\x00\x00\x00\x00' > "$WRONG_DIR/bad.pcap"
"$BINARY" analyze "$WRONG_DIR" --no-color; echo "Exit code: $?"
rm -rf "$WRONG_DIR"
echo ""

echo "=== AC-003: Directory with only a 3-byte file (too short for magic probe) ==="
SHORT_DIR=$(mktemp -d)
printf '\x0A\x0D\x0D' > "$SHORT_DIR/truncated.pcap"
"$BINARY" analyze "$SHORT_DIR" --no-color; echo "Exit code: $?"
rm -rf "$SHORT_DIR"
echo ""

echo "--- Both silently skipped: Packets: 0, exit 0 (no panic, no error) ---"
rm -rf "$DEMO_DIR"
