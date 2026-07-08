# Wave-71 Integration Demo Evidence Report

**Wave:** 71  
**Stories:** STORY-150, STORY-156, STORY-157  
**Branch:** develop (HEAD b642c0f — wave-71 merged)  
**Recorded:** 2026-07-08  
**Scrub gate:** PASS — zero `<home>/` or absolute host paths in output dir  
**Recording method:** Transcript fallback (VHS attempted; failed due to wirerust unicode
box-drawing output scrolling past VHS 0.11.0 viewport before `Wait+Screen` match — see
tape RECORDING NOTE comments for details)

---

## Wave Claim Coverage

| Demo | Artifact | Wave Claim Evidenced |
|------|----------|---------------------|
| DEMO-001 | `DEMO-001-combined-analyzer.txt` | TLS analyzer (STORY-150 drain-loop refactor) produces normal triage report; ARP analyzer (STORY-156 BC-2.16.016) produces storm finding on arp-storm.pcap |
| DEMO-002 | `DEMO-002-frag-equivalence.txt` | STORY-150 behavior-preservation: fragmented ClientHello yields same JA3 (`6169fabc98e3e6c9690301eaf306d632`) and SNI (`example.com`) as single-record control |
| DEMO-003 | `DEMO-003-factory-hash-scan.txt` | STORY-157 hash tooling: `bin/compute-input-hash --scan` reports MATCH=110 STALE=0 on merged develop |
| DEMO-004 | `DEMO-004-full-suite.txt` | Full regression gate: 2,378 tests passed, 0 failed across all 91 test harnesses |

---

## DEMO-001: Combined Analyzer Run

**File:** `DEMO-001-combined-analyzer.txt`  
**Command 1:** `./target/release/wirerust analyze tests/fixtures/tls13-rfc8446.pcap --tls`  
**Command 2:** `./target/release/wirerust analyze tests/fixtures/local-samples/arp-storm.pcap --arp --arp-storm-rate 5`

**Evidence:**
- TLS analyzer activates, processes 13 packets, reports 2 TLS flows with valid JA3 hashes — confirms the STORY-150 drain-loop refactor did not break TLS analysis path
- ARP analyzer activates, processes 622 ARP frames, fires D3 storm finding (1 finding) — confirms STORY-156 BC-2.16.016 ARP path is live on merged develop

**Key output (TLS):**
```
ANALYZER: TLS
  Packets analyzed: 2
  ja3_hashes: {"25bd5fc6fcc031c1c87f93613e0252ce":1,"7ed44a80eb19c620609c3b5cbc2eafd0":1}
  parse_errors: 0
```

**Key output (ARP):**
```
FINDINGS
  [Anomaly] POSSIBLE (MEDIUM) - D3: ARP storm detected — source MAC 00:07:0D:AF:F4:54

ANALYZER: ARP
  frames_analyzed: 622
  storm_findings: 1
```

---

## DEMO-002: Fragmented ClientHello Equivalence

**File:** `DEMO-002-frag-equivalence.txt`  
**Command (fragmented):** `wirerust analyze .factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-fragmented.pcap --tls --json | jq {ja3, sni}`  
**Command (control):** `wirerust analyze .factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-control.pcap --tls --json | jq {ja3, sni}`

**Evidence:** Both fixtures produce **identical** JA3 hash and SNI:
```
Fragmented: ja3=6169fabc98e3e6c9690301eaf306d632, sni=["example.com"]
Control:    ja3=6169fabc98e3e6c9690301eaf306d632, sni=["example.com"]
```
This confirms STORY-150's behavior-preservation claim: the drain-loop refactor does not change TLS fingerprinting output when handling fragmented handshakes.

---

## DEMO-003: Factory Tooling Hash Scan

**File:** `DEMO-003-factory-hash-scan.txt`  
**Command:** `WIRERUST_REPO_ROOT=<repo> bin/compute-input-hash --scan | tail -12`

**Evidence:**
```
STORY-150.md     a001aa4    a001aa4    MATCH
STORY-156.md     8c9b0ba    8c9b0ba    MATCH
STORY-157.md     4ca0ad4    4ca0ad4    MATCH
...
MATCH=110 STALE=0
```
All 110 stories report MATCH — no spec drift. STORY-157's hash tooling is running correctly on the merged develop tree.

---

## DEMO-004: Full Test Suite

**File:** `DEMO-004-full-suite.txt`  
**Command:** `cargo test --all-targets 2>&1 | grep '^test result'`

**Evidence:** 91 test harnesses, all reporting `ok`:
```
test result: ok. 229 passed; 0 failed; ...
test result: ok. 160 passed; 0 failed; ...
...
=== Wave-71 gate: 2378 tests passed, 0 failed ===
```
No regressions introduced by any of the three wave-71 stories.

---

## Artifact Inventory

```
.factory/cycles/wave-71/wave-gate/demo-evidence/
├── DEMO-001-combined-analyzer.tape    # VHS tape (reference; transcript is primary evidence)
├── DEMO-001-combined-analyzer.txt     # Transcript: TLS + ARP combined run
├── DEMO-002-frag-equivalence.tape     # VHS tape (reference; transcript is primary evidence)
├── DEMO-002-frag-equivalence.txt      # Transcript: fragmented vs control JA3/SNI equivalence
├── DEMO-003-factory-hash-scan.tape    # VHS tape (reference; transcript is primary evidence)
├── DEMO-003-factory-hash-scan.txt     # Transcript: factory hash scan MATCH=110 STALE=0
├── DEMO-004-full-suite.tape           # VHS tape (reference; transcript is primary evidence)
├── DEMO-004-full-suite.txt            # Transcript: 2,378 tests passed 0 failed
└── wave-evidence-report.md            # This file
```

## Scrub Gate

**Command run:** `grep -rE '<host-path-pattern>' .factory/cycles/wave-71/wave-gate/demo-evidence/`  
**Result:** Zero matches — all absolute host paths replaced with `<repo>` placeholder in tape files; transcripts contain only repo-relative paths.

## VHS Recording Note

VHS recording was attempted for all 4 demos. All tapes failed with `Wait+Screen` timeout because wirerust's unicode box-drawing separator characters (`────────────────────────────────────────`, U+2500 × 40) cause the output to fill VHS 0.11.0's viewport before the matched pattern scrolls into view. The `Wait+Screen` directive only checks the visible viewport (not scrollback). This is a known limitation of VHS with tools that produce long unicode-formatted output.

Mitigation for future VHS recordings of wirerust: pipe output through `grep` or use `--json` with short field extraction to keep visible output under ~20 lines per command.

The tape files are retained as runnable reference scripts (with `<repo>` placeholder) and include comments documenting the VHS limitation.
