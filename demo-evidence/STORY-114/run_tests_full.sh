#!/usr/bin/env bash
set -euo pipefail
cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114
echo "=== cargo test --all-targets ==="
cargo test --all-targets 2>&1 | grep -E '^test result' | awk '{sum += $4} END {print "Total passed:", sum, "| Expected: 1552"}'
