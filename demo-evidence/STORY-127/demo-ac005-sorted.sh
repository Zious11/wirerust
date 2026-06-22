#!/usr/bin/env bash
# Demo script for AC-005: Sorted output + mixed-extension detection (BC-2.12.011 PC5/Inv3).
# Files created in z, a, m order -- sorted output processes a, m, z.
# m.cap (pcapng magic, .cap extension) is detected by content.
# Called by VHS tape. Does NOT modify source code or tests.
set -euo pipefail

WDIR="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
BINARY="$WDIR/target/release/wirerust"
DEMO_DIR=$(mktemp -d)

# Create files in reverse sort order to prove sort() is enforced
cp "$WDIR/tests/fixtures/http-ooo.pcap" "$DEMO_DIR/z.pcap"   # 16 pkts, created 1st
cp "$WDIR/tests/fixtures/http.pcap"     "$DEMO_DIR/a.pcap"   # 1 pkt, created 2nd
cp "$WDIR/tests/fixtures/smb3.pcapng"   "$DEMO_DIR/m.cap"    # pcapng-magic .cap, created 3rd

echo "=== Files created in order: z, a, m (without sort: wrong order) ==="
ls -lt "$DEMO_DIR"
echo ""
echo "=== Sorted analysis: a.pcap -> m.cap -> z.pcap (alphabetical) ==="
echo "    m.cap detected by pcapng magic [0A 0D 0D 0A], .cap extension ignored"
echo ""
"$BINARY" analyze "$DEMO_DIR" --no-color
echo ""
echo "--- Total: 1(http) + 54(smb3) + 16(http-ooo) = 71 packets ---"
echo "--- Sorted: a.pcap, m.cap, z.pcap (lexicographic order enforced) ---"

rm -rf "$DEMO_DIR"
