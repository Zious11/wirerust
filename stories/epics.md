---
document_type: epics
version: "1.3"
status: draft
producer: story-writer
phase: 2
timestamp: 2026-05-21T00:00:00Z
modified:
  - "2026-06-17 v1.2: E-18 Terminal Finding-Collapse (issue #259) added — STORY-118 + STORY-119 (deferred). total_bcs 283→288 (+5 new BC-2.11.025–029; 4 existing BCs extended/versioned — count unchanged)."
  - "2026-06-17 v1.3: Adversarial Burst 3 remediation — E-8 story count 7→5 (Estimated Story Count Summary table; actual E-8 roster is STORY-076..080 = 5 stories). Column sum now 72, matching Total row."
total_bcs: 288
traces_to:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/architecture/ARCH-INDEX.md
---

# wirerust Epic Decomposition

> **Brownfield context:** wirerust is a single-crate offline pcap forensic triage CLI.
> The 217 behavioral contracts describe the *current* shipped implementation; 2 additional
> Feature Mode F2 BCs (BC-2.04.055, BC-2.09.007) bring the total to 219.
> Epics are cohesive groupings of user value aligned to PRD capabilities and subsystem
> boundaries. No epic is a pure 1:1 subsystem copy where capabilities naturally compose
> into a larger user-visible deliverable.

---

## Epic E-1: PCAP Ingestion and Packet Decoding

- **Goal:** A forensic analyst can point wirerust at any supported pcap file (Ethernet,
  RAW IPv4/IPv6, Linux SLL) and have every packet read, validated, and decoded into a
  structured ParsedPacket representation — with clear, attributed error messages when
  files are malformed or formats are unsupported (pcapng, unknown link types).
- **BCs:**
  BC-2.01.001, BC-2.01.002, BC-2.01.003, BC-2.01.004, BC-2.01.005, BC-2.01.006,
  BC-2.01.007, BC-2.01.008,
  BC-2.02.001, BC-2.02.002, BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006,
  BC-2.02.007, BC-2.02.008, BC-2.02.009, BC-2.02.010, BC-2.02.011, BC-2.02.012,
  BC-2.02.013, BC-2.02.014, BC-2.02.015
- **Subsystems touched:** SS-01, SS-02
- **Estimated stories:** 5

**Rationale:** PCAP ingestion (SS-01) and packet decoding (SS-02) form the first two
pipeline stages and share a tight coupling — the reader produces `RawPacket`, the decoder
consumes it to produce `ParsedPacket`. A forensic analyst experiences these as a single
"can wirerust read my pcap?" interaction. Splitting them into separate epics would produce
artificially small epics with no standalone user value.

---

## Epic E-2: TCP Stream Reassembly Engine

- **Goal:** A forensic analyst analyzing multi-packet TCP sessions sees correct,
  ordered stream data reconstructed from raw pcap frames — including correct handling
  of retransmissions, out-of-order segments, mid-stream join, RST/FIN termination,
  and configurable resource caps (flow count, memory ceiling) — so that protocol
  analyzers receive complete application-layer payloads rather than fragmented frames.
- **BCs:**
  BC-2.04.001, BC-2.04.002, BC-2.04.003, BC-2.04.004, BC-2.04.005, BC-2.04.006,
  BC-2.04.007, BC-2.04.008, BC-2.04.009, BC-2.04.010, BC-2.04.011, BC-2.04.012,
  BC-2.04.013, BC-2.04.014, BC-2.04.015, BC-2.04.016, BC-2.04.017, BC-2.04.018,
  BC-2.04.019, BC-2.04.020, BC-2.04.021, BC-2.04.022, BC-2.04.023, BC-2.04.024,
  BC-2.04.025, BC-2.04.026, BC-2.04.027, BC-2.04.028, BC-2.04.029, BC-2.04.030,
  BC-2.04.031, BC-2.04.032, BC-2.04.033, BC-2.04.034, BC-2.04.035, BC-2.04.036,
  BC-2.04.037, BC-2.04.038, BC-2.04.039, BC-2.04.040, BC-2.04.041, BC-2.04.042,
  BC-2.04.043, BC-2.04.044, BC-2.04.045, BC-2.04.046, BC-2.04.047, BC-2.04.048,
  BC-2.04.049, BC-2.04.050, BC-2.04.051, BC-2.04.052, BC-2.04.053, BC-2.04.054
- **Subsystems touched:** SS-04
- **Estimated stories:** 11

**Rationale:** TCP Reassembly is the most complex subsystem (54 BCs, ~7 source files).
It delivers a self-contained user value: making multi-packet TCP sessions analyzable.
It also emits its own findings (overlap/evasion anomalies), making it independently
verifiable. The size (54 BCs) justifies decomposing into stories that cover: core
state machine, segment insertion/flush logic, overlap/evasion detection, resource
pressure management, and statistics/summary emission.

---

## Epic E-3: Content-First Protocol Dispatch

- **Goal:** When wirerust classifies which protocol analyzer handles a TCP stream,
  it uses the payload bytes first (not port numbers) — so attackers cannot evade
  analysis by running HTTP on port 9999 or TLS on port 8080. The dispatcher caches
  classifications, retries indeterminate flows, and reports unclassified flow counts
  so analysts can audit coverage.
- **BCs:**
  BC-2.05.001, BC-2.05.002, BC-2.05.003, BC-2.05.004, BC-2.05.005, BC-2.05.006,
  BC-2.05.007, BC-2.05.008, BC-2.05.009
- **Subsystems touched:** SS-05
- **Estimated stories:** 3

**Rationale:** Protocol dispatch (ADR 0001) is a standalone architectural decision
with clear user value: content-first routing means port-obfuscation attacks are
mitigated. Its 9 BCs decompose naturally into classification logic, caching, and
flow lifecycle. It is a pipeline stage upstream of both HTTP and TLS analysis.

---

## Epic E-4: HTTP Traffic Analysis and Threat Detection

- **Goal:** A forensic analyst processing HTTP traffic from a pcap sees: complete
  HTTP/1.1 request/response parsing, detection of path traversal, web shell access,
  admin panel probing, unusual methods, oversized URIs, missing Host headers, and
  empty User-Agent — each emitting a structured finding with MITRE technique ID,
  verdict, and confidence. The analyst trusts that cross-flow isolation, parse-error
  poisoning, and per-direction buffer caps prevent false positives from corrupted data.
- **BCs:**
  BC-2.06.001, BC-2.06.002, BC-2.06.003, BC-2.06.004, BC-2.06.005, BC-2.06.006,
  BC-2.06.007, BC-2.06.008, BC-2.06.009, BC-2.06.010, BC-2.06.011, BC-2.06.012,
  BC-2.06.013, BC-2.06.014, BC-2.06.015, BC-2.06.016, BC-2.06.017, BC-2.06.018,
  BC-2.06.019, BC-2.06.020, BC-2.06.021, BC-2.06.022, BC-2.06.023, BC-2.06.024,
  BC-2.06.025, BC-2.06.026
- **Subsystems touched:** SS-06
- **Estimated stories:** 6

**Rationale:** HTTP analysis (26 BCs) is a complete threat-detection domain with four
natural story groups: request/response parsing, threat detection rules (5+ finding types),
parse-error isolation and poisoning, and resource caps/summary. The user value is clear
and independently deliverable from TLS analysis.

---

## Epic E-5: TLS Traffic Analysis and Fingerprinting

- **Goal:** A malware researcher or forensic analyst sees JA3/JA3S fingerprints for
  every TLS handshake, SNI hostname extraction with 4-way anomaly classification
  (clean ASCII / C0-control / non-ASCII UTF-8 / non-UTF-8 bytes), and findings for
  weak ciphers and deprecated protocol versions — enabling identification of known
  malware TLS profiles and evasion techniques, without decrypting traffic.
- **BCs:**
  BC-2.07.001, BC-2.07.002, BC-2.07.003, BC-2.07.004, BC-2.07.005, BC-2.07.006,
  BC-2.07.007, BC-2.07.008, BC-2.07.009, BC-2.07.010, BC-2.07.011, BC-2.07.012,
  BC-2.07.013, BC-2.07.014, BC-2.07.015, BC-2.07.016, BC-2.07.017, BC-2.07.018,
  BC-2.07.019, BC-2.07.020, BC-2.07.021, BC-2.07.022, BC-2.07.023, BC-2.07.024,
  BC-2.07.025, BC-2.07.026, BC-2.07.027, BC-2.07.028, BC-2.07.029, BC-2.07.030,
  BC-2.07.031, BC-2.07.032, BC-2.07.033, BC-2.07.034, BC-2.07.035, BC-2.07.036,
  BC-2.07.037
- **Subsystems touched:** SS-07
- **Estimated stories:** 8

**Rationale:** TLS analysis (37 BCs) covers three distinct analyst needs: handshake
parsing + JA3/JA3S computation, SNI anomaly classification (the most security-sensitive
subsection with 4 arms and boundary tests), and cipher/protocol weakness detection.
The 37 BCs justify ~8 stories covering: ClientHello parsing, ServerHello/JA3S, SNI
4-way classification, cipher/version findings, buffer management, and summary.

---

## Epic E-6: DNS Traffic Statistics

- **Goal:** A forensic analyst or SOC operator sees accurate DNS query and response
  counts in the analysis summary for any pcap that includes port-53 traffic, without
  any false findings being emitted — providing a baseline DNS traffic picture for
  triage without the complexity of full DNS parsing.
- **BCs:**
  BC-2.08.001, BC-2.08.002, BC-2.08.003, BC-2.08.004
- **Subsystems touched:** SS-08
- **Estimated stories:** 1

**Rationale:** DNS analysis is intentionally limited to statistics-only (4 BCs,
no findings ever emitted). Its user value is narrow but clear: DNS traffic volume
in summary output. It warrants a single story because all 4 BCs are inseparable
(dispatch, count, summarize, never-emit).

---

## Epic E-7: Forensic Finding Data Model and MITRE Mapping

- **Goal:** Every finding emitted by wirerust carries a consistent, structured data
  model (category, verdict, confidence, summary, evidence, MITRE technique ID) that
  serializes to valid JSON with no None fields in output, displays correctly to a
  terminal operator with uppercase verdict/confidence tokens, and maps to the correct
  MITRE ATT&CK tactic via a complete seeded lookup table — enabling SIEM ingestion
  and kill-chain analysis.
- **BCs:**
  BC-2.09.001, BC-2.09.002, BC-2.09.003, BC-2.09.004, BC-2.09.005, BC-2.09.006,
  BC-2.10.001, BC-2.10.002, BC-2.10.003, BC-2.10.004, BC-2.10.005, BC-2.10.006,
  BC-2.10.007, BC-2.10.008, BC-2.10.009
- **Subsystems touched:** SS-09, SS-10
- **Estimated stories:** 3

**Rationale:** The Finding struct (SS-09) and MITRE mapping table (SS-10) form a
cohesive data-model layer — every finding carries a technique ID that resolves through
the MITRE table. Separating them into two micro-epics of 6 and 9 BCs each would
produce epics too small to deliver standalone value. Combined, they form the
"structured forensic output contract" that both reporters and the terminal grouping
function depend on.

---

## Epic E-8: Reporting and Output Formats

- **Goal:** A SOC operator or security toolchain integrator can select JSON, CSV, or
  terminal (default) output from wirerust. JSON output faithfully preserves all raw
  forensic bytes per ADR 0003 (C0 bytes escaped per RFC 8259 by serde, non-ASCII
  Unicode readable, C1 bytes passed through). Terminal output displays findings grouped
  by MITRE tactic in kill-chain order, with escape logic protecting against terminal
  injection, and colorized severity indicators. CSV output produces a fixed 9-column
  format suitable for spreadsheet import and SIEM ingestion, with CSV-injection
  neutralization.
- **BCs:**
  BC-2.11.001, BC-2.11.002, BC-2.11.003, BC-2.11.004, BC-2.11.005, BC-2.11.006,
  BC-2.11.007, BC-2.11.008, BC-2.11.009, BC-2.11.010, BC-2.11.011, BC-2.11.012,
  BC-2.11.013, BC-2.11.014, BC-2.11.015, BC-2.11.016, BC-2.11.017, BC-2.11.018,
  BC-2.11.019, BC-2.11.020, BC-2.11.021, BC-2.11.022, BC-2.11.023, BC-2.11.024
- **Subsystems touched:** SS-11
- **Estimated stories:** 5

**Rationale:** Reporting (24 BCs) covers three distinct output surfaces (JSON,
terminal, CSV) plus the MITRE tactic-grouping logic for terminal output. User value
is clear: the output format is what the analyst or integrator actually sees and
consumes. Stories decompose naturally into: JsonReporter, TerminalReporter
(escaping), TerminalReporter (MITRE grouping/sort/colorization), and CsvReporter.

---

## Epic E-9: CLI, Entry Point, and Analysis Orchestration

- **Goal:** A forensic analyst or SOC operator can invoke wirerust from the command
  line with single or multiple pcap targets (files or directories), select protocol
  analyzers (--http, --tls, --dns, --all), configure reassembly parameters
  (--reassemble, --depth, --memcap, five threshold flags), choose output format
  (--output-format json|csv or legacy --json/--csv with file path), control color
  (--no-color / NO_COLOR env), and receive a per-target progress bar on stderr —
  with mutually exclusive flag enforcement, sensible defaults, and clear error
  messages for invalid input.
- **BCs:**
  BC-2.12.001, BC-2.12.002, BC-2.12.003, BC-2.12.004, BC-2.12.005, BC-2.12.006,
  BC-2.12.007, BC-2.12.008, BC-2.12.009, BC-2.12.010, BC-2.12.011, BC-2.12.012,
  BC-2.12.013, BC-2.12.014, BC-2.12.015, BC-2.12.016, BC-2.12.017,
  BC-2.12.018, BC-2.12.019, BC-2.12.020, BC-2.12.021
- **Subsystems touched:** SS-12
- **Estimated stories:** 5

**Rationale:** CLI (SS-12) and the Summary data model (also in SS-12) are inseparable
from the user's perspective: the analyst types a command, the summary struct accumulates
per-packet data as it runs, and the output lands in their chosen format. All 21 BCs
cover the complete "invocation to output" user journey. Splitting CLI from Summary
would produce an artificially small epic with no standalone deliverable.

---

## Epic E-10: Absent Behavior Contracts (Flag Rejection)

- **Goal:** A forensic analyst who types an obsolete or never-implemented flag
  (--threats, --beacon, --filter, --verbose) receives an immediate, clear error from
  clap's argument parser — preventing silent misuse of removed features and ensuring
  the tool's documented surface matches its actual capabilities.
- **BCs:**
  BC-2.13.001, BC-2.13.002, BC-2.13.003, BC-2.13.004
- **Subsystems touched:** SS-13
- **Estimated stories:** 1

**Rationale:** The 4 absent-behavior contracts form a coherent user-facing guarantee:
removed flags are actively rejected, not silently ignored. This is independently
testable (clap integration test). One story covers all 4 BCs because they share the
same implementation pattern (clap `conflicts_with` / missing flag definition) and
the same test vehicle (CLI invocation with obsolete flag).

---

## Coverage Check

### Per-Epic BC Assignment

| Epic | Subsystems | BCs Assigned | Count |
|------|-----------|--------------|-------|
| E-1: PCAP Ingestion and Packet Decoding | SS-01, SS-02 | BC-2.01.001..008, BC-2.02.001..015 | 23 |
| E-2: TCP Stream Reassembly Engine | SS-04 | BC-2.04.001..054 | 54 |
| E-3: Content-First Protocol Dispatch | SS-05 | BC-2.05.001..009 | 9 |
| E-4: HTTP Traffic Analysis and Threat Detection | SS-06 | BC-2.06.001..026 | 26 |
| E-5: TLS Traffic Analysis and Fingerprinting | SS-07 | BC-2.07.001..037 | 37 |
| E-6: DNS Traffic Statistics | SS-08 | BC-2.08.001..004 | 4 |
| E-7: Forensic Finding Data Model and MITRE Mapping | SS-09, SS-10 | BC-2.09.001..006, BC-2.10.001..009 | 15 |
| E-8: Reporting and Output Formats | SS-11 | BC-2.11.001..024 | 24 |
| E-9: CLI, Entry Point, and Analysis Orchestration | SS-12 | BC-2.12.001..021 | 21 |
| E-10: Absent Behavior Contracts (Flag Rejection) | SS-13 | BC-2.13.001..004 | 4 |
| E-12: Pcap Timestamp Provenance (issue #100) | SS-04, SS-09 | BC-2.04.055, BC-2.09.007 | 2 |
| E-13: Multi-Tag Finding Schema Migration | SS-09, SS-10, SS-11 | BC-2.09.001/006 (extensions), BC-2.10.005/007/008 (extensions), BC-2.11.001/013/015/017/020/024 (extensions) | 0 (extensions, not new BCs) |
| E-14: Modbus TCP Analyzer | SS-14 (new), SS-05, SS-12 | BC-2.14.001..025 | 25 |
| E-15: DNP3/ICS Analyzer | SS-15 (new), SS-05, SS-12 | BC-2.15.001..024 | 24 |
| E-16: ARP Security Analyzer | SS-16 (new) | BC-2.16.001..015 | 15 |
| **TOTAL** | | | **283** |

### Arithmetic Verification

```
E-1:  8 (SS-01) + 15 (SS-02) = 23
E-2:  54 (SS-04)              = 54
E-3:  9 (SS-05)               =  9
E-4:  26 (SS-06)              = 26
E-5:  37 (SS-07)              = 37
E-6:  4 (SS-08)               =  4
E-7:  6 (SS-09) + 9 (SS-10)  = 15
E-8:  24 (SS-11)              = 24
E-9:  21 (SS-12)              = 21
E-10: 4 (SS-13)               =  4
E-12: 2 (BC-2.04.055, BC-2.09.007) = 2
                      --------
                      219 (pre-feature subtotal)
E-14: 25 (SS-14, BC-2.14.001..025) = 25
E-15: 24 (SS-15, BC-2.15.001..024) = 24
E-16: 15 (SS-16, BC-2.16.001..015) = 15
                      --------
                      283 / 283  ✓
```

Note: E-11 (Tooling) has 0 BCs authored yet (STORY-091 pending). E-12 BCs are feature-mode
additions (BC-2.04.055 extends SS-04; BC-2.09.007 extends SS-09) and do not conflict with
the greenfield 217-BC assignment.

### No BC Double-Assigned

Each BC-2.NN.NNN maps to exactly one epic by construction: the epic corresponds to
the subsystem(s) identified in ARCH-INDEX.md, and subsystem assignments are
non-overlapping. No BC appears in more than one epic row above.

### All 12 Subsystems Covered

| SS-ID | Name | Epic |
|-------|------|------|
| SS-01 | PCAP Ingestion | E-1 |
| SS-02 | Packet Decoding | E-1 |
| SS-03 | (absent — merged into SS-02 per ARCH-INDEX ruling) | E-1 |
| SS-04 | TCP Reassembly | E-2 |
| SS-05 | Protocol Dispatch | E-3 |
| SS-06 | HTTP Analysis | E-4 |
| SS-07 | TLS Analysis | E-5 |
| SS-08 | DNS Analysis | E-6 |
| SS-09 | Finding Emission | E-7 |
| SS-10 | MITRE Mapping | E-7 |
| SS-11 | Reporting | E-8 |
| SS-12 | CLI / Entry | E-9 |
| SS-13 | Absent Behaviors | E-10 |

**Coverage confirmed: 283 / 283 BCs assigned, 0 unassigned, 0 double-assigned.**
(219 pre-feature + 25 E-14 Modbus + 24 E-15 DNP3 + 15 E-16 ARP = 283)

---

## Epic E-11: Tooling and Self-Improvement

- **Goal:** Build and govern mechanical tooling that catches spec-drift proactively —
  before adversarial passes surface it as findings — so the cost of each successive
  adversarial cycle falls rather than holding flat. The first deliverable is an
  anchor-validation CLI (`bin/validate-anchors`) that verifies every `src|tests|fuzz/
  <path>.rs:NNN` citation in the spec corpus against the current source tree; the
  second is a codified governance policy (ANCHOR-VALIDATION-001) requiring consistency
  audits after any fix-burst that shifts code lines or renames functions.
- **BCs:** _(none authored yet — status: draft; pending PO authorship)_
- **Subsystems touched:** none (tooling-only; no production Rust subsystem)
- **Estimated stories:** 1 (STORY-091)
- **Dispositions:** PROCESS-GAP-P5-001 (S-7.02 cycle-close requirement)

**Rationale:** Phase-5 adversarial refinement repeatedly surfaced source-line-anchor
drift across four dimensions (BC source anchors, BC secondary anchors, consuming
VP/invariant/supplement/entity docs, story bodies) — 83 stale citations corrected
in one pass alone. Root cause: every sweep was reactive (triggered by an adversarial
finding) rather than preventive. PROCESS-GAP-P5-001 requires a durable-fix
disposition at cycle close (S-7.02). A dedicated tooling epic separates this
self-improvement work from product epics and makes future tooling stories easy to
group here.

---

## Epic E-12: Pcap Timestamp Provenance (issue #100)

- **Goal:** A forensic analyst reviewing wirerust JSON/CSV output sees a `timestamp` field on every Finding, populated from the pcap capture-relative `ts_sec` value, enabling correlation of detections with the original packet capture timeline.
- **BCs:**
  BC-2.04.055 (StreamHandler::on_data timestamp parameter),
  BC-2.09.007 (Finding.timestamp provenance)
- **Subsystems touched:** SS-04 (reassembly), SS-06 (HTTP analyzer), SS-07 (TLS analyzer), SS-09 (findings)
- **Estimated stories:** 3 (STORY-097, STORY-098, STORY-099)
- **Feature issue:** #100

**Rationale:** The timestamp feature (O-01 domain-debt) spans 3 implementation layers: the trait-boundary (SS-04 on_data parameter), the emission sites (SS-06/07 per-flow storage + finding construction), and E2E verification (VP-021). These 3 layers decompose naturally into 3 stories with strict sequential dependency (trait break → emission → verification).

---

## Epic E-15: DNP3/ICS Analyzer (issue #8)

- **Goal:** A forensic analyst or ICS/OT security engineer can point wirerust at a pcap
  containing DNP3 traffic (TCP port 20000, IEEE 1815-2012) and receive structured findings
  for unauthorized control commands (T1692.001), restart/stop commands (T0814), write-
  register commands (T0836), block-control inference (T1691.001), process impact (T0827),
  and anomaly conditions (broadcast, unsolicited, malformed frames) — with per-flow state
  tracking, a 292-byte carry buffer for segment-spanning frame reassembly, and a tunable
  `--dnp3-direct-operate-threshold` CLI flag.
- **BCs:**
  BC-2.15.001, BC-2.15.002, BC-2.15.003, BC-2.15.004, BC-2.15.005, BC-2.15.006,
  BC-2.15.007, BC-2.15.008, BC-2.15.009, BC-2.15.010, BC-2.15.011, BC-2.15.012,
  BC-2.15.013, BC-2.15.014, BC-2.15.015, BC-2.15.016, BC-2.15.017, BC-2.15.018,
  BC-2.15.019, BC-2.15.020, BC-2.15.021, BC-2.15.022, BC-2.15.023, BC-2.15.024
- **Subsystems touched:** SS-15 (new DNP3 analyzer), SS-05 (dispatcher Rule 6), SS-12 (CLI threshold flag)
- **Estimated stories:** 5 (STORY-106..110)
- **Feature issue:** #8

**Rationale:** DNP3 analysis (24 BCs, IEEE 1815-2012 binary protocol) decomposes into
five natural layers matching the ADR-007 design decisions: (1) pure-core parse + FC
classification (Kani-verifiable, VP-023 anchor), (2) per-flow state + carry buffer +
memory safety bounds, (3) direct detection emissions (T1692.001, T0814 restart, T0836),
(4) correlated/derived + anomaly detections (T1691.001, T0827, broadcast, malformed —
VP-007 atomic-update anchor), (5) dispatcher integration + CLI flag (VP-004 oracle
obligation). Each layer is independently testable; the dependency chain is strictly linear
with no parallelism (each story builds on the previous one's produced types and state).

---

## Epic E-17: ARP Decoder VLAN/QinQ/MACsec Offset Hardening (issue #253)

- **Goal:** A forensic analyst running wirerust against pcaps containing QinQ double-tagged
  or MACsec-encapsulated Ethernet frames has regression coverage for ARP offset arithmetic:
  QinQ double-tagged ARP frames are verified at the 22-byte offset (EC-008), and
  MACsec-encapsulated frames are documented as a known limitation (observe-only probe,
  no silent misclassification) — with fixture pcaps and regression tests ensuring no
  offset regression when etherparse is upgraded. Single-VLAN (18-byte offset) handling
  is pre-existing baseline behavior shipped in E-16, not a new E-17 test.
- **BCs:**
  BC-2.16.009, BC-2.16.015
- **Subsystems touched:** SS-16 (ARP analyzer, lax-path offset handling)
- **Estimated stories:** 2 (STORY-116, STORY-117)
- **Feature issue:** #253
- **Release target:** v0.7.1
- **Total points:** 8 (STORY-116: 3 pts, STORY-117: 5 pts)

**Rationale:** The VLAN/QinQ/MACsec offset edge cases (EC-008, EC-009 per BC-2.16.009
and BC-2.16.015) are not delivered as part of the v0.7.0 ARP Security Analyzer (E-16).
They represent a hardening increment that requires dedicated fixture pcaps and regression
tests targeting decode-time offset arithmetic in the ARP lax-path. STORY-116 delivers
VLAN + QinQ fixture coverage; STORY-117 delivers MACsec observe-only documentation and
probe test. The two stories are strictly linear (STORY-116 → STORY-117). Both use
`tdd_mode: facade` because they deliver test files against already-shipped code — no
`todo!()` stub cycle.

---

## Epic E-18: Terminal Finding-Collapse (issue #259, v0.8.0)

- **Goal:** A network security analyst running `wirerust analyze` on a high-volume pcap
  (e.g., an empty-User-Agent flood of 10,000 requests) sees repeated identical findings
  collapsed into a single annotated group with a ` (xN)` count suffix in the terminal
  output, reducing noise and improving triage velocity. JSON and CSV output remain
  unaffected (display-layer only). An explicit `--no-collapse` flag on the `analyze`
  subcommand restores per-finding output for scripting or detailed triage. Grouped/`--mitre`
  mode bypasses collapse in v0.8.0 (deferred to STORY-119).
- **BCs:**
  BC-2.11.025, BC-2.11.026, BC-2.11.027, BC-2.11.028, BC-2.11.029 (new — E-18);
  BC-2.11.010 v1.8, BC-2.11.013 v1.11, BC-2.11.017 v1.13, BC-2.11.019 v1.6 (extended)
- **Subsystems touched:** SS-11 (reporter/terminal.rs), SS-12 (cli.rs, main.rs — thin wiring)
- **Estimated stories:** 2 (STORY-118 scheduled Wave 47; STORY-119 deferred)

**Rationale:** The collapse feature is a pure display-layer transform confined to
`src/reporter/terminal.rs`. It shares no subsystem boundary with JSON/CSV reporters
(BC-2.11.029 invariant 1). The `--no-collapse` CLI flag follows the established
subcommand-scoped boolean pattern (`--mitre`, `--dns`), making it a thin wiring addition
to SS-12. The scope is narrow enough for a single story (STORY-118, 8 points). STORY-119
(grouped-mode collapse) is deferred to a future cycle because grouped mode renders
findings individually in v0.8.0 and the BC forward-references are satisfied by the stub.

---

## Estimated Story Count Summary

| Epic | Stories Est. |
|------|-------------|
| E-1  | 5           |
| E-2  | 11          |
| E-3  | 3           |
| E-4  | 6           |
| E-5  | 8           |
| E-6  | 1           |
| E-7  | 3           |
| E-8  | 5           |
| E-9  | 5           |
| E-10 | 1           |
| E-11 | 1           |
| E-12 | 3           |
| E-13 | 2           |
| E-14 | 4           |
| E-15 | 5           |
| E-16 | 5           |
| E-17 | 2           |
| E-18 | 2           |
| **Total** | **72** |
