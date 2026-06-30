#!/bin/sh
# Show compact TLS analysis: SNI, JA3, parse_errors
# Uses the STORY-144 release binary in the worktree
PCAP="$1"
LABEL="$2"
WDIR="/Users/zious/Documents/GITHUB/wirerust/.worktrees/story-144-tls-carry-reassembly"
echo "=== $LABEL ==="
"$WDIR/target/release/wirerust" analyze --tls --json /tmp/wrtls-out.json "$PCAP" 2>/dev/null
python3 -c "
import sys, json
with open('/tmp/wrtls-out.json') as f:
    data = json.load(f)
tls = next(a for a in data['analyzers'] if a['analyzer_name'] == 'TLS')
d = tls['detail']
sni = d['top_snis']
ja3 = list(d['ja3_hashes'].keys())
errs = d['parse_errors']
print(f'  SNI:          {sni if sni else \"(empty -- MISSED)\"}')
print(f'  JA3:          {ja3[0][:16]+\"...\" if ja3 else \"(empty -- MISSED)\"}')
print(f'  parse_errors: {errs}')
"
