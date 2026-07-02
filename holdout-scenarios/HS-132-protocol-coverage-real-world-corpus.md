---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.023.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.024.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.001.md
  - .factory/stories/STORY-152.md
  - .factory/stories/STORY-154.md
traces_to: .factory/specs/prd.md
id: "HS-132"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.12.023
  - BC-2.12.024
  - BC-2.18.001
lifecycle_status: active
introduced: v0.12.0-feature-protocol-coverage
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Two real-world corpus sources needed: (1) a known-good IT network pcap (e.g., Wireshark sample captures from the SampleCaptures archive — any public HTTP/HTTPS/DNS IT traffic pcap); (2) a known-problematic ICS pcap with BACnet/IP UDP/47808 traffic (BACnet capture from a building automation testbed, a Wireshark BACnet sample, or a researcher-released ICS capture with documented BACnet traffic). See scenario body for sources."
canonical_value_scenario: true
canonical_spec_citation: "BACnet/IP uses UDP port 47808 (0xBAC0) per ASHRAE 135-2016 Annex J §J.2.1. The known-problematic corpus is expected to contain BACnet/IP UDP/47808 traffic that wirerust classifies as known-unsupported in CoverageGapsSummary."
input-hash: "94f7144"
---

# Holdout Scenario: Protocol Coverage Real-World Corpus — Known-Good IT (Low False-Positive) and Known-Problematic with BACnet/IP (Known Gap Detection)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

This is the real-world corpus holdout for the feature-protocol-coverage cycle, required
by the product specification's real-world corpus mandate. It exercises two public corpus
sources to validate the `protocols` subcommand and the `CoverageGapsSummary` feature:

1. **Known-good IT corpus**: A publicly-available PCAP of normal IT network traffic (HTTP,
   DNS, TLS) with no ICS protocol traffic. Expected results:
   - `wirerust protocols` exits 0 and lists all 30 entries correctly.
   - `wirerust analyze <known_good.pcap> --all --coverage-gaps` runs without crash.
   - No GOOSE entry in the CoverageGapsSummary (GOOSE is L2, cannot appear — confirmed by
     the L2 caveat).
   - L2 caveat present in all CoverageGapsSummary output.

2. **Known-problematic ICS corpus**: A publicly-available PCAP with BACnet/IP UDP/47808
   traffic from a building automation system or ICS research environment. Expected results:
   - `wirerust analyze <ics_pcap.pcap> --coverage-gaps --json` produces a `known-unsupported`
     gap entry for UDP/47808 labeled "BACnet/IP".
   - The tool handles real-world ICS captures without crash.

### Corpus Sources (Public, Reproducible)

**Known-good IT corpus options (evaluator selects one):**

1. **Wireshark SampleCaptures IT traffic** — The Wireshark project's public sample pcap
   archive includes typical HTTP, HTTPS/TLS, and DNS captures:
   - https://wiki.wireshark.org/SampleCaptures
   - Search for "http.pcap", "ssl.pcap", or any general IT traffic capture.
   - Expected: clean IT traffic, no ICS protocols, zero unexpected gaps.

2. **Any of the test pcaps from prior wirerust integration tests** — The evaluator may
   reuse any pcap that has been used in existing holdout scenarios for the HTTP/TLS/DNS
   analyzers (e.g., from HS-067, HS-068, HS-091 test data). These are known to contain
   no ICS protocol traffic.

**Known-problematic ICS corpus options (evaluator selects one):**

1. **Wireshark BACnet sample captures** — Wireshark sample pcaps include BACnet/IP captures:
   - https://wiki.wireshark.org/Protocols/bacnet
   - A BACnet/IP pcap with UDP/47808 traffic from a building automation system.
   - Expected: UDP/47808 flows unclassified (wirerust has no BACnet dissector) → gap entry.

2. **NMAP BACnet scan trace** — An NMAP scan using the bacnet-info NSE script generates
   UDP/47808 traffic. A capture of such a scan would produce UDP/47808 unclassified packets.
   Expected: `known-unsupported` gap entry for UDP/47808.

3. **ICS-focused CTF or public dataset captures** — Any publicly available ICS pcap with
   documented BACnet/IP traffic (e.g., from the S4 Conference public dataset, Defcon ICS
   Village, or Shodan-adjacent research releases that include BACnet).

### Case A — Known-Good IT Corpus: `wirerust protocols` Catalog Is Correct

1. The evaluator runs: `wirerust protocols --json`
2. The tool exits 0.
3. `jq '.protocols | length'` == 30.
4. `jq '.protocols | map(select(.supported == true)) | length'` == 7.
5. `jq '.protocols | map(select(.supported == false)) | length'` == 23.
6. `jq '.protocols[] | select(.name | test("BACnet"; "i")) | .transport'` == `"UDP"`.
   BACnet/IP is present in the catalog with the correct transport (ASHRAE 135-2016 Annex J §J.2.1).

### Case B — Known-Good IT Corpus: `analyze --all` Runs Without Crash; No GOOSE in Gap Report

1. The evaluator runs: `wirerust analyze <known_good_it.pcap> --all --coverage-gaps --json`
2. The tool exits 0 (no crash, no panic, no OOM).
3. The analysis completes normally.
4. `jq '.coverage_gaps.entries[] | select(.port == 35000)'` — no such entry.
   GOOSE uses EtherType 0x88B8 (35000), not a port; it cannot appear in the gap report
   (which only tracks TCP/UDP port-based observations). This verifies the L2 caveat is correct.
5. `jq '.coverage_gaps.caveat_l2'` is a non-null string (L2 caveat present).
6. The tool does NOT produce a GOOSE finding or a GOOSE gap entry — GOOSE traffic, if present
   in the real-world pcap, is invisible to the TCP/UDP port-based gap detector.

### Case C — Known-Problematic ICS Corpus: UDP/47808 Gap Entry Present

1. The evaluator obtains a public BACnet/IP pcap (see sources above).
2. The evaluator runs: `wirerust analyze <bacnet_ics.pcap> --coverage-gaps --json`
3. The tool exits 0.
4. The JSON `"coverage_gaps"."entries"` contains at least one entry:
   - `"transport": "UDP"`, `"port": 47808`, `"state": "known-unsupported"`, `"name": "BACnet/IP"`
   This confirms: (a) the UDP decode loop correctly counted the BACnet/IP traffic,
   (b) the tri-state classifier correctly identified it as `known-unsupported`,
   (c) the catalog attribution correctly named it "BACnet/IP" (ASHRAE 135-2016 Annex J §J.2.1).
5. Count must be ≥ 1 (at least one BACnet/IP packet was observed and counted).

### Case D — Both Corpus Runs: No Panic, No OOM

Both Case B and Case C must complete without panic, OOM, or crash. Real-world pcap files
may contain:
- Fragmented TCP segments
- TCP retransmissions
- Unusual protocol combinations
The analyzer must process them gracefully with exit code 0.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.18.001 | `wirerust protocols` catalog output is correct (30 entries, 7+23 partition) | Case A |
| BC-2.12.023 | `--coverage-gaps` produces CoverageGapsSummary with L2 caveat | Cases B, C: caveat present |
| BC-2.12.024 | L2 caveat explains GOOSE and L2 protocols not in gap report | Case B: no GOOSE gap entry |
| BC-2.12.024 | UDP/47808 classified as known-unsupported, named BACnet/IP | Case C (ASHRAE 135-2016 Annex J §J.2.1) |
| BC-2.12.024 | Tool handles real-world captures without crash | Case D |

<!-- HIDDEN TRACEABILITY: BC-2.12.024 EC-007 (GOOSE EtherType traffic absent from gap report because L2);
     BC-2.12.024 EC-002 (BACnet/IP UDP/47808 only → known-unsupported entry);
     BC-2.18.001 Postcondition 7 (L2 caveat explains absence of GOOSE from gap report) -->

## Verification Approach

```bash
# Case A — catalog correct (known-good corpus, no pcap needed for protocols subcommand)
wirerust protocols --json | jq '.protocols | {total: length, supported: (map(select(.supported)) | length), unsupported: (map(select(.supported | not)) | length)}'
# Expect: {"total": 30, "supported": 7, "unsupported": 23}

wirerust protocols --json | jq '.protocols[] | select(.name | test("BACnet"; "i")) | .transport'
# Expect: "UDP"

# Case B — known-good IT pcap: no crash; no GOOSE gap entry
wirerust analyze known_good_it.pcap --all --coverage-gaps --json | jq '.coverage_gaps.caveat_l2'
# Expect: non-null string

# Case C — known-problematic BACnet pcap: UDP/47808 gap entry
wirerust analyze bacnet_ics.pcap --coverage-gaps --json | \
  jq '.coverage_gaps.entries[] | select(.transport == "UDP" and .port == 47808)'
# Expect: {transport:"UDP", port:47808, state:"known-unsupported", name:"BACnet/IP", count:>=1}
```

## Evaluation Rubric

- **Catalog correctness (real-world binary)** (weight: 0.25): Case A: protocols subcommand
  produces correct 30/7/23 partition with correct BACnet/IP transport=UDP.
- **Known-good corpus: no crash; L2 caveat** (weight: 0.25): Case B: analyze exits 0; L2 caveat
  present; no GOOSE gap entry (confirms L2 structural limitation is correctly communicated).
- **Known-problematic corpus: BACnet/IP detected** (weight: 0.35): Case C: UDP/47808
  known-unsupported entry present. CANONICAL-VALUE must-pass (ASHRAE 135-2016 Annex J §J.2.1).
- **No crash on real-world captures** (weight: 0.15): Case D: both pcaps exit 0.

## Failure Guidance

"HOLDOUT FAIL: HS-132 — real-world corpus failure.
Case A: if catalog has wrong counts (not 30 total / 7 supported / 23 unsupported), check KNOWN_PROTOCOLS
array length and supported_protocols()/unsupported_protocols() implementations.
Case C: if UDP/47808 gap entry absent: either the UDP counter is not firing for BACnet traffic
(check dns_analyzer.can_decode() exclusion path — BACnet/IP is not DNS and should NOT be excluded),
OR the tri-state lookup is returning 'unknown' for UDP/47808 (check catalog has BACnet/IP with
transport=Udp and canonical_ports=[47808] per ASHRAE 135-2016 Annex J §J.2.1).
Case D: if the tool panics on real-world BACnet captures, check the UDP decode loop's handling
of large or fragmented UDP datagrams."
