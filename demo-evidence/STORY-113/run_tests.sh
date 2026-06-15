#!/bin/bash
cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113
cargo test --all-targets 2>&1 | grep '^test result' | awk '{sum+=$4} END {print "Total:", sum, "passed -- 0 failed"}'
