---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-076.md
  - .factory/stories/STORY-077.md
  - .factory/stories/STORY-078.md
  - .factory/stories/STORY-079.md
  - .factory/stories/STORY-080.md
  - .factory/stories/STORY-086.md
  - .factory/stories/STORY-087.md
  - .factory/stories/STORY-088.md
  - .factory/stories/STORY-089.md
  - .factory/stories/STORY-090.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.007.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.019.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.001.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.008.md
input-hash: "d2026ba"
traces_to: .factory/stories/STORY-086.md
id: "HS-091"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.001
  - BC-2.12.008
  - BC-2.11.007
  - BC-2.11.013
  - BC-2.11.019
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: End-to-End pcap → Terminal Report on Known-Problematic Corpus (False Negative Test)

## Scenario

A forensic analyst runs wirerust against a publicly-available capture that is known to contain
suspicious or malicious traffic. The tool must detect the known issues and report them without
false-negatives — and must still escape any attacker-controlled bytes in the terminal output.

**Known-problematic corpus:** The Wireshark sample capture `telnet-cooked.pcap` from
`https://wiki.wireshark.org/SampleCaptures` (telnet with cleartext credentials) or, as an
alternative, any pcap from the Malware Traffic Analysis repository
(https://www.malware-traffic-analysis.net/training-exercises.html) containing known suspicious
HTTP patterns. The selected pcap should demonstrate at least HTTP anomaly detection.

1. The tool is invoked: `wirerust analyze --all --http --mitre <suspicious.pcap>`
2. The tool exits with code 0.
3. The terminal output contains at least one finding.
4. All finding summaries and evidence in the terminal output contain no raw C0 or C1 bytes —
   any attacker-controlled content is escaped.
5. If MITRE grouping is requested (`--mitre`), tactic section headers appear in kill-chain order.
6. The `WIRERUST TRIAGE REPORT` header is the first line of the output.
7. ANALYZER section(s) appear last in the output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.001 | Postcondition 1: analyze subcommand processes target | Successful invocation with suspicious pcap |
| BC-2.12.008 | Postcondition 1: --all enables all analyzers | All three analyzers active in this run |
| BC-2.11.007 | Postcondition 6: no raw C0/C1 in output | Real-world attacker content must be escaped |
| BC-2.11.013 | Postcondition 2: tactic headers in kill-chain order | MITRE section ordering in real run |
| BC-2.11.019 | Postcondition 1: header first; analyzers last | Section ordering in real-world output |

## Verification Approach

1. Use a Wireshark sample pcap known to have suspicious content (e.g., cleartext credentials,
   abnormal HTTP methods, or known evasion patterns). The pcap must be publicly available.
2. Run: `wirerust analyze --all --mitre <suspicious.pcap>` and capture stdout.
3. Assert exit code == 0.
4. Assert stdout contains at least one finding line (non-empty findings section).
5. Scan the raw bytes of stdout for `\x00`-`\x1f` (C0), `\x7f` (DEL), and the UTF-8 encoding
   of C1 range (bytes `\xc2\x80` through `\xc2\x9f`). Assert none of these appear.
6. Assert "WIRERUST TRIAGE REPORT" appears before any finding lines.
7. If MITRE tactic headers are present, assert they are in kill-chain order by checking the
   position of any two adjacent tactic names that have a known relative order.

### Real-World Corpus Metadata

| Field | Description |
|-------|-------------|
| corpus_source | Wireshark Sample Captures: https://wiki.wireshark.org/SampleCaptures — `telnet-cooked.pcap` or any HTTP anomaly pcap from malware-traffic-analysis.net |
| corpus_size | Variable; target < 10 MB for reasonable test time |
| known_edge_cases | Cleartext credentials in payload; unusual HTTP methods or paths; potential control characters in payloads |
| false_positive_threshold | N/A (expect findings) |
| false_negative_threshold | At least 1 finding expected; 0 findings = potential false-negative issue |

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): At least 1 finding detected on known-suspicious traffic; valid terminal output structure.
- **Edge case handling** (weight: 0.15): Attacker-controlled bytes in payloads are escaped before terminal display.
- **Error quality** (weight: 0.1): No panic; exit code 0 even with suspicious content.
- **Performance** (weight: 0.15): Completes in < 60 seconds for < 10 MB pcap.
- **Data integrity** (weight: 0.2): No raw C0/C1/DEL bytes in stdout; section order is correct.

## Edge Conditions

- If the selected pcap has no HTTP traffic, HTTP findings may be absent — that is acceptable.
- If all traffic is non-TCP, TLS/HTTP analyzers produce nothing — acceptable.
- At least one finding from any analyzer is required for a meaningful false-negative test.
- The tool must not crash or panic on any well-formed pcap, regardless of payload content.

## Failure Guidance

"HOLDOUT LOW: HS-091 (satisfaction: 0.XX) -- End-to-end run on suspicious corpus produced zero findings (potential false-negative), contained raw control bytes in terminal output (escaping failure), or had incorrect section ordering."
