#!/usr/bin/env bash
# Demo script for AC-009: E2E corpus wiring (BC-2.12.011 EC-001..002).
# smb3.pcapng via full reader stack. Synthetic 16-pkt pcapng via .cap extension.
# Called by VHS tape. Does NOT modify source code or tests.
set -euo pipefail

WDIR="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
BINARY="$WDIR/target/release/wirerust"

echo "=== E2E Sub-case 1: smb3.pcapng (committed fixture) ==="
"$BINARY" analyze "$WDIR/tests/fixtures/smb3.pcapng" --no-color
echo ""

echo "=== E2E Sub-case 2: synthetic arp-baseline (16 EPBs via .cap extension) ==="
TMPFILE=$(mktemp -t arp-baseline).cap
python3 - "$TMPFILE" <<'PYEOF'
import struct, sys

out = sys.argv[1]

shb = (b'\x0A\x0D\x0D\x0A'
       + struct.pack('<I', 28)
       + b'\x4D\x3C\x2B\x1A'
       + struct.pack('<H', 1) + struct.pack('<H', 0)
       + struct.pack('<q', -1)
       + struct.pack('<I', 28))

idb = (b'\x01\x00\x00\x00'
       + struct.pack('<I', 20)
       + struct.pack('<H', 1) + struct.pack('<H', 0)
       + struct.pack('<I', 65535)
       + struct.pack('<I', 20))

epb = (b'\x06\x00\x00\x00'
       + struct.pack('<I', 32)
       + struct.pack('<IIIII', 0, 0, 0, 0, 0)
       + struct.pack('<I', 32))

with open(out, 'wb') as f:
    f.write(shb + idb + epb * 16)

print(f"  Written: {out} ({len(shb+idb+epb*16)} bytes, pcapng SHB+IDB+16xEPB)")
PYEOF

"$BINARY" analyze "$TMPFILE" --no-color 2>&1 || true
echo "  (16 decode errors = 16 empty EPBs parsed; .cap extension, pcapng content)"
rm -f "$TMPFILE"
