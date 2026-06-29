# Demo Evidence: TLS ClientHello Fragmentation Evasion (BEFORE fix)

Cycle: `fix-tls-clienthello-frag`  
Branch: `develop` at commit `a2d8c13`  
Date: 2026-06-29

## What this demonstrates

wirerust's TLS analyzer does **not** reassemble a ClientHello that is fragmented
across multiple TLS records. RFC 5246 §6.2.1 explicitly permits the TLS record
layer to split any handshake message across multiple `TLSPlaintext` records. An
attacker can send the ClientHello in two or more TLS records to make wirerust
miss the SNI and JA3 fingerprint entirely.

### Root cause

`TlsAnalyzer::try_parse_records` (`src/analyzer/tls.rs`) calls
`parse_tls_plaintext` on each individual TLS record. When the ClientHello
handshake message is split, the first record contains an incomplete handshake
message body. `parse_tls_plaintext` returns a parse error. The second record
contains the remainder of the message with no valid TLS record framing, so it
also fails. Neither record is identified as a ClientHello. The analyzer has no
cross-record handshake reassembly buffer.

## Fixtures

| File | Description |
|------|-------------|
| `tls-clienthello-control.pcap` | Minimal TLS 1.2 ClientHello, SNI=`example.com`, in a **single** TLS record. Normal case. |
| `tls-clienthello-fragmented.pcap` | Same ClientHello split across **two** TLS records (bytes 0..33 in record 1, bytes 33..67 in record 2). Evasion case. |
| `gen_pcaps.py` | Python 3 script (stdlib only) that generated both pcaps. Re-run to regenerate. |
| `show-tls-result.sh` | Helper script: runs wirerust and prints SNI/JA3/parse_errors compactly. |

Both pcaps use Ethernet/IPv4/TCP framing (linktype 1). No checksums — wirerust
does not validate them.

## Recording

| File | Purpose |
|------|---------|
| `AC-001-tls-frag-evasion-baseline.gif` | VHS recording showing control vs evasion side by side |
| `AC-001-tls-frag-evasion-baseline.webm` | Same recording, archival format |
| `AC-001-tls-frag-evasion-baseline.tape` | VHS script source |

## Results (pre-fix)

**CONTROL** (single TLS record):
```
SNI:          ['example.com']   ← detected
JA3:          6169fabc98e3e6c9...  ← computed
parse_errors: 0
```

**EVASION** (same ClientHello, fragmented across 2 TLS records):
```
SNI:          (empty — MISSED)  ← evasion confirmed
JA3:          (empty — MISSED)  ← evasion confirmed
parse_errors: 2                 ← both records fail to parse
```

### Evasion reproduced: YES

The gap is complete: SNI and JA3 are both absent with `parse_errors: 2`. An
adversary who fragments the ClientHello achieves blind-spot coverage against
wirerust's TLS analyzer on the current develop branch.

## Intended use in F3+

These pcap fixtures are ready to use as holdout test inputs in Phase F3:
- `tls-clienthello-control.pcap`: regression fixture (must still work after fix)
- `tls-clienthello-fragmented.pcap`: correctness fixture (must extract SNI/JA3 after fix)

Additional edge-case fixtures to generate in F3:
- 3-way split (ClientHello across 3 TLS records)
- Split at the very first byte (only `\x01` in record 1)
- Split inside the SNI extension body (not the handshake type boundary)
