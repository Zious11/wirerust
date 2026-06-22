#!/usr/bin/env bash
# Setup script called by VHS tapes to create demo fixture directories.
# This script is sourced/called by the VHS tapes, not run standalone.
set -euo pipefail

WDIR="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
BINARY="$WDIR/target/release/wirerust"

# ── Demo 1: Content-Over-Extension (AC-004 headline) ─────────────────────────
# A real pcapng file named .cap is detected; a wrong-magic .pcap is skipped.
export DEMO1_DIR
DEMO1_DIR=$(mktemp -d)
cp "$WDIR/tests/fixtures/smb3.pcapng" "$DEMO1_DIR/capture-renamed-as.cap"
printf '\xDE\xAD\xBE\xEF\x00\x00\x00\x00' > "$DEMO1_DIR/imposter-wrong-magic.pcap"

# ── Demo 2: All 5 Magic Values (AC-001) ──────────────────────────────────────
# 5 capture files with non-standard extensions, 1 wrong-magic .pcap excluded.
export DEMO2_DIR
DEMO2_DIR=$(mktemp -d)
python3 "$WDIR/.factory/demo-evidence/STORY-127/build-5-magic-fixtures.py" "$DEMO2_DIR"

# ── Demo 3: Silent Skip Error Paths (AC-002 + AC-003) ────────────────────────
export DEMO3_DIR
DEMO3_DIR=$(mktemp -d)
printf '\xDE\xAD\xBE\xEF\x00\x00\x00\x00' > "$DEMO3_DIR/wrong-magic.pcap"
printf '\x0A\x0D\x0D' > "$DEMO3_DIR/short-3-bytes.pcap"

# ── Demo 4: Sorted Output + Mixed Extensions (AC-005) ─────────────────────────
export DEMO4_DIR
DEMO4_DIR=$(mktemp -d)
cp "$WDIR/tests/fixtures/http-ooo.pcap" "$DEMO4_DIR/z.pcap"   # 16 pkts, created first
cp "$WDIR/tests/fixtures/http.pcap"     "$DEMO4_DIR/a.pcap"   # 1 pkt, created second
cp "$WDIR/tests/fixtures/smb3.pcapng"   "$DEMO4_DIR/m.cap"    # pcapng as .cap, created third
