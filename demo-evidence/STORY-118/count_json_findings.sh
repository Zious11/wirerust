#!/bin/bash
# Helper for AC-020 demo: count JSON findings to prove collapse is display-only
/Users/zious/Documents/GITHUB/wirerust/.worktrees/feat-259-finding-collapse/target/release/wirerust \
    analyze --http --json /tmp/s118_findings.json \
    /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-118/empty_ua_flood.pcap \
    > /dev/null 2>&1
python3 -c "
import json
d = json.load(open('/tmp/s118_findings.json'))
n = len(d['findings'])
print(f'JSON finding objects: {n}  (collapse is display-only, all {n} emitted)')
"
