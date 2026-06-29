#!/bin/sh
# Show compact TLS analysis: SNI, JA3, parse_errors
PCAP="$1"
LABEL="$2"
echo "=== $LABEL ==="
./target/release/wirerust analyze --tls --no-color "$PCAP" --json 2>/dev/null \
  | python3 -c "
import sys, json
data = json.load(sys.stdin)
tls = next(a for a in data['analyzers'] if a['analyzer_name'] == 'TLS')
d = tls['detail']
sni = d['top_snis']
ja3 = list(d['ja3_hashes'].keys())
errs = d['parse_errors']
print(f'  SNI:          {sni if sni else \"(empty — MISSED)\"}')
print(f'  JA3:          {ja3[0][:16]+\"...\" if ja3 else \"(empty — MISSED)\"}')
print(f'  parse_errors: {errs}')
"
