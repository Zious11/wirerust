---
document_type: holdout-scenario-index
level: ops
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-06-14T00:00:00Z
phase: 2
total_scenarios: 100
must_pass_count: 99
should_pass_count: 1
total_waves: 27
feature_holdout_seeds:
  dnp3_waves_35_39: 32
  arp_waves_40_44: 28
traces_to:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/stories/STORY-INDEX.md
---

# wirerust Holdout Scenario Index

> **Authoritative registry of all 100 holdout scenarios for the v0.1.0-greenfield-spec cycle.**
> Holdout scenarios are sealed evaluations used by the holdout-evaluator agent only.
> They must NEVER be shown to implementer or test-writer agents.
>
> Wave columns reflect which delivery waves a scenario exercises, derived from the
> story inputs in each HS file cross-referenced against STORY-INDEX.md wave assignments.
> All HS files carry concrete per-file `inputs` listing the specific BC files and story
> files they trace to; wave derivation uses those story inputs cross-referenced with
> the wave assignments in STORY-INDEX.md.

---

## Verification Results

| Check | Result |
|-------|--------|
| Total HS files present | 100 (HS-001..HS-100) — greenfield set only; see Feature Holdouts section below for DNP3 |
| Sequential numbering (no gaps) | PASS — all integers 1..100 present (greenfield HS-NNN sequence) |
| Duplicate IDs | NONE |
| Empty `behavioral_contracts` fields | NONE — all 100 non-empty |
| All waves 1-27 covered | PASS — see per-wave table below (greenfield waves; DNP3 waves 35-39 are in the feature tree) |

---

## Summary Roll-Ups

### Totals

| Metric | Count |
|--------|-------|
| Total scenarios | 100 |
| must-pass (`must_pass: true`) | 99 |
| should-pass (`must_pass: false`) | 1 |
| Categories | 5 |

### By Category

| Category | Count |
|----------|-------|
| behavioral-subtleties | 36 |
| edge-case-combinations | 19 |
| integration-boundaries | 18 |
| security-probes | 17 |
| real-world-corpus | 10 |
| **TOTAL** | **100** |

### By Epic

| Epic | Description | Count |
|------|-------------|-------|
| E-1 | PCAP Ingestion and Packet Decoding | 8 |
| E-2 | TCP Stream Reassembly Engine | 28 |
| E-3 | Content-First Protocol Dispatch | 5 |
| E-4 | HTTP Traffic Analysis and Threat Detection | 10 |
| E-5 | TLS Traffic Analysis and Fingerprinting | 12 |
| E-6 | DNS Traffic Statistics | 2 |
| E-7 | Forensic Finding Data Model and MITRE Mapping | 7 |
| E-8 | Reporting and Output Formats | 15 |
| E-9 | CLI, Entry Point, and Analysis Orchestration | 12 |
| E-10 | Absent Behavior Contracts (Flag Rejection) | 1 |
| **TOTAL** | | **100** |

> Each scenario is counted once under its primary `epic_id` from frontmatter.
> Counts are derived directly from the Scenario Index rows below.

---

## Per-Wave Coverage Table

> **Note on Count column arithmetic:** The Count column intentionally counts each multi-wave
> scenario once in every wave it spans. A scenario assigned to waves 15-18 therefore contributes
> +1 to each of waves 15, 16, 17, and 18. As a result, the Count column total across all 27 waves
> exceeds 100 by design — it is not an arithmetic error. The authoritative distinct-scenario total
> is **100**, verified by the By-Epic and By-Category tables above, each of which sums to exactly
> 100 because each scenario is counted only once.

Every wave 1-27 has at least one scenario. Column shows count of scenarios covering that wave
(scenarios spanning multiple waves are counted in each wave they cover).

| Wave | Story Wave | Scenarios Covering Wave | Count |
|------|-----------|------------------------|-------|
| 1 | STORY-001, STORY-069 | HS-001, HS-002, HS-006, HS-007, HS-015, HS-016, HS-017, HS-023, HS-024 | 9 |
| 2 | STORY-002, STORY-003, STORY-004, STORY-070 | HS-003, HS-004, HS-005, HS-007, HS-015, HS-017, HS-018, HS-022 | 8 |
| 3 | STORY-005, STORY-071 | HS-005, HS-008, HS-009, HS-015, HS-023, HS-025 | 6 |
| 4 | STORY-011, STORY-066 | HS-010, HS-011, HS-020, HS-023 | 4 |
| 5 | STORY-012 | HS-012, HS-023 | 2 |
| 6 | STORY-013 | HS-013, HS-016, HS-019, HS-021, HS-024 | 5 |
| 7 | STORY-014 | HS-014, HS-016, HS-019, HS-021 | 4 |
| 8 | STORY-015, STORY-019 | HS-026, HS-027, HS-028, HS-029, HS-043, HS-046 | 6 |
| 9 | STORY-016, STORY-020 | HS-030, HS-031, HS-041, HS-044, HS-046 | 5 |
| 10 | STORY-017, STORY-018 | HS-032, HS-033, HS-034, HS-035, HS-041, HS-042, HS-047, HS-048, HS-050 | 9 |
| 11 | STORY-021 | HS-036, HS-037 | 2 |
| 12 | STORY-031 | HS-038, HS-046, HS-049 | 3 |
| 13 | STORY-032 | HS-039 | 1 |
| 14 | STORY-033 | HS-040, HS-045 | 2 |
| 15 | STORY-041, STORY-051 | HS-051, HS-052, HS-053, HS-054, HS-055, HS-056, HS-057, HS-058, HS-059, HS-060, HS-061, HS-062, HS-063, HS-065, HS-066, HS-067, HS-068, HS-069, HS-070, HS-071, HS-072, HS-074 | 22 |
| 16 | STORY-042, STORY-043, STORY-044, STORY-052 | HS-051, HS-052, HS-053, HS-054, HS-055, HS-056, HS-057, HS-058, HS-059, HS-060, HS-061, HS-062, HS-063, HS-065, HS-066, HS-067, HS-068, HS-069, HS-070, HS-071, HS-072, HS-074 | 22 |
| 17 | STORY-045, STORY-053, STORY-055 | HS-051, HS-052, HS-053, HS-054, HS-055, HS-056, HS-057, HS-058, HS-059, HS-060, HS-061, HS-062, HS-063, HS-065, HS-066, HS-067, HS-068, HS-069, HS-070, HS-071, HS-072, HS-074 | 22 |
| 18 | STORY-046, STORY-054, STORY-056, STORY-058 | HS-051, HS-052, HS-053, HS-054, HS-055, HS-056, HS-057, HS-058, HS-059, HS-060, HS-061, HS-062, HS-063, HS-065, HS-066, HS-067, HS-068, HS-069, HS-070, HS-071, HS-072, HS-074 | 22 |
| 19 | STORY-057 | HS-052, HS-055, HS-056, HS-057, HS-059, HS-062, HS-063, HS-066, HS-068, HS-069, HS-071, HS-074 | 12 |
| 20 | STORY-076 | HS-064, HS-073, HS-075, HS-076, HS-077, HS-078, HS-079, HS-080, HS-081, HS-082, HS-083, HS-090, HS-091, HS-092, HS-093, HS-098, HS-099 | 15 |
| 21 | STORY-077, STORY-079 | HS-064, HS-073, HS-075, HS-076, HS-077, HS-078, HS-079, HS-080, HS-081, HS-082, HS-083, HS-090, HS-091, HS-092, HS-093, HS-098, HS-099 | 15 |
| 22 | STORY-078, STORY-080 | HS-064, HS-073, HS-075, HS-076, HS-077, HS-078, HS-079, HS-080, HS-081, HS-082, HS-083, HS-090, HS-091, HS-092, HS-093, HS-098, HS-099 | 15 |
| 23 | STORY-086 | HS-084, HS-085, HS-087, HS-088, HS-089, HS-090, HS-091, HS-094, HS-095, HS-096, HS-097, HS-100 | 12 |
| 24 | STORY-087, STORY-096 | HS-084, HS-085, HS-086, HS-087, HS-088, HS-089, HS-090, HS-091, HS-094, HS-095, HS-096, HS-097, HS-100 | 13 |
| 25 | STORY-088 | HS-084, HS-085, HS-087, HS-088, HS-089, HS-090, HS-091, HS-094, HS-095, HS-096, HS-097, HS-100 | 12 |
| 26 | STORY-089 | HS-084, HS-085, HS-087, HS-088, HS-089, HS-090, HS-091, HS-094, HS-095, HS-096, HS-097, HS-100 | 12 |
| 27 | STORY-090 | HS-084, HS-085, HS-087, HS-088, HS-089, HS-090, HS-091, HS-094, HS-095, HS-096, HS-097, HS-100 | 12 |

**Result: All 27 waves have >= 1 scenario. PASS.**

---

## Scenario Index

All 100 scenarios, one row each, grouped by epic.

### Epic E-1: PCAP Ingestion and Packet Decoding (Waves 1-3)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-001](HS-001-pcap-link-type-gating.md) | PCAP Link-Type Boundary — Accepted vs. Rejected at File Open | integration-boundaries | must-pass | 1 | BC-2.01.001, BC-2.01.004 |
| [HS-002](HS-002-pcap-zero-packet-and-error-surfaces.md) | Empty Capture and Corrupt-Header Behavior at Ingest | edge-case-combinations | must-pass | 1 | BC-2.01.002, BC-2.01.003, BC-2.01.006, BC-2.01.007 |
| [HS-003](HS-003-ethernet-ipv4-ipv6-decode-paths.md) | Ethernet, RAW IPv4, and IPv6 Link-Layer Decode Correctness | integration-boundaries | must-pass | 2 | BC-2.02.001, BC-2.02.003, BC-2.02.005, BC-2.02.007 |
| [HS-004](HS-004-linux-sll-icmp-non-ip-rejection.md) | Linux SLL Cooked Capture, ICMP Classification, and Non-IP Frame Handling | edge-case-combinations | must-pass | 2 | BC-2.02.006, BC-2.02.009, BC-2.02.010, BC-2.02.011 |
| [HS-005](HS-005-protocol-hint-and-packet-len-semantics.md) | App Protocol Hints, Frame Length Accounting, and TCP Flag Extraction | behavioral-subtleties | must-pass | 2, 3 | BC-2.02.012, BC-2.02.014, BC-2.02.015 |
| [HS-015](HS-015-real-world-corpus-clean-pcap.md) | Real-World Corpus — Well-Maintained Public PCAP (Low False Positive Rate) | real-world-corpus | must-pass | 1, 2, 3 | BC-2.01.001, BC-2.01.002, BC-2.02.001, BC-2.04.003 |
| [HS-022](HS-022-decoder-malformed-packet-no-panic.md) | Decoder No-Panic Safety — Malformed and Truncated Packets | security-probes | must-pass | 2 | BC-2.02.007, BC-2.02.008, BC-2.02.009 |
| [HS-023](HS-023-e1-e2-e6-e7-integration-summary.md) | Waves 1-5 Full Integration — PCAP -> Decode -> Reassembly -> DNS -> MITRE | integration-boundaries | must-pass | 1, 3, 4, 5 | BC-2.01.002, BC-2.04.028, BC-2.08.003, BC-2.10.003 |

### Epic E-2: TCP Stream Reassembly Engine (Waves 4-11)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-010](HS-010-flowkey-canonicalization-and-constructor.md) | FlowKey Symmetry — Bidirectional Packets Merge into One Flow | integration-boundaries | must-pass | 4 | BC-2.04.001, BC-2.04.003, BC-2.04.049 |
| [HS-012](HS-012-non-tcp-filter-bytes-reassembled-accounting.md) | Non-TCP Packet Filtering, Reassembly Stats, and Byte Accounting | integration-boundaries | must-pass | 5 | BC-2.04.002, BC-2.04.028, BC-2.04.030 |
| [HS-013](HS-013-tcp-handshake-state-machine-and-direction-tagging.md) | Three-Way Handshake Completion and RST Abrupt Close | behavioral-subtleties | must-pass | 6 | BC-2.04.004, BC-2.04.005, BC-2.04.050, BC-2.04.051, BC-2.04.053 |
| [HS-014](HS-014-mid-stream-join-partial-capture.md) | Mid-Stream Join — Partial Captures Analyzed Without Silent Data Corruption | edge-case-combinations | must-pass | 7 | BC-2.04.009, BC-2.04.031, BC-2.04.032 |
| [HS-016](HS-016-real-world-corpus-evasion-pcap.md) | Real-World Corpus — Known-Problematic PCAP with TCP Evasion Patterns | real-world-corpus | must-pass | 1, 6, 7 | BC-2.04.018, BC-2.04.037, BC-2.09.005 |
| [HS-019](HS-019-tcp-seq-wraparound-reassembly.md) | TCP Sequence Number Wraparound — Reassembly Correctness Across 32-Bit Boundary | edge-case-combinations | must-pass | 6, 7 | BC-2.04.039, BC-2.04.006, BC-2.04.007 |
| [HS-021](HS-021-rst-fin-close-and-timeout-lifecycle.md) | TCP Flow Close Variants — RST, FIN, and Idle Timeout All Release Resources | edge-case-combinations | must-pass | 6, 7 | BC-2.04.010, BC-2.04.011, BC-2.04.012, BC-2.04.013 |
| [HS-024](HS-024-finding-raw-data-e7-source-ip-dispatch.md) | Source IP Field — Present for Reassembly Findings, Absent for HTTP/TLS Findings | behavioral-subtleties | must-pass | 1, 6 | BC-2.09.001, BC-2.04.018 |
| [HS-026](HS-026-ooo-delivery-ordering.md) | Out-of-Order Segment Delivery Preserves Application Byte Order | behavioral-subtleties | must-pass | 8 | BC-2.04.007, BC-2.04.008, BC-2.04.039 |
| [HS-027](HS-027-direction-tagging-accuracy.md) | Bidirectional Data Direction Tags Are Mutually Exclusive and Accurate | behavioral-subtleties | must-pass | 8 | BC-2.04.006 |
| [HS-028](HS-028-flow-close-reasons.md) | Flow Close Semantics — RST Skips Payload, FIN Delivers Payload First | behavioral-subtleties | must-pass | 8 | BC-2.04.010, BC-2.04.011, BC-2.04.013 |
| [HS-029](HS-029-sequence-wraparound.md) | TCP Sequence Number Wraparound Across 32-bit Boundary | edge-case-combinations | must-pass | 8 | BC-2.04.039 |
| [HS-030](HS-030-retransmission-no-false-positive.md) | Normal TCP Retransmissions Do Not Produce False-Positive Findings | behavioral-subtleties | must-pass | 9 | BC-2.04.035, BC-2.04.043, BC-2.04.047 |
| [HS-031](HS-031-lru-eviction-protects-established.md) | Memory Eviction Discards Incomplete Flows Before Established Sessions | edge-case-combinations | must-pass | 9 | BC-2.04.015, BC-2.04.016, BC-2.04.017 |
| [HS-032](HS-032-tcp-evasion-detection.md) | TCP Segment Splicing Evasion Is Detected with T1036 Finding | security-probes | must-pass | 10 | BC-2.04.018, BC-2.04.019, BC-2.04.022 |
| [HS-033](HS-033-small-segment-exemption.md) | Small-Segment Alert Respects Port Exemption List | behavioral-subtleties | must-pass | 10 | BC-2.04.020 |
| [HS-034](HS-034-depth-truncation-bounds.md) | Stream Depth Limit Prevents Memory Exhaustion on Oversized Flows | security-probes | must-pass | 10 | BC-2.04.041, BC-2.04.023, BC-2.04.027 |
| [HS-035](HS-035-out-of-window-rejection.md) | Out-of-Window Segments Are Rejected and Counted | security-probes | must-pass | 10 | BC-2.04.042, BC-2.04.021 |
| [HS-036](HS-036-max-findings-cap.md) | Findings Cap Prevents Memory Exhaustion Under Adversarial Load | security-probes | must-pass | 11 | BC-2.04.024, BC-2.04.054, BC-2.04.025 |
| [HS-037](HS-037-finalize-idempotency.md) | End-of-PCAP Finalizes All Open Flows Without Duplication | behavioral-subtleties | must-pass | 11 | BC-2.04.012 |
| [HS-041](HS-041-evasion-combined-attack.md) | Combined Evasion — Conflicting Bytes Plus Cumulative Overlap Threshold | security-probes | must-pass | 9, 10 | BC-2.04.018, BC-2.04.019, BC-2.04.022, BC-2.04.036 |
| [HS-042](HS-042-segment-limit-mid-stream.md) | Per-Direction Segment Map Cap Prevents BTreeMap Overhead | security-probes | must-pass | 10 | BC-2.04.044, BC-2.04.045, BC-2.04.046 |
| [HS-043](HS-043-timeout-idle-cleanup.md) | Idle Flow Timeout Cleans Up Long-Silent Connections | behavioral-subtleties | must-pass | 8 | BC-2.04.013, BC-2.04.029 |
| [HS-044](HS-044-total-memory-consistency.md) | Memory Accounting Stays Consistent Across Insert, Flush, and Close | behavioral-subtleties | must-pass | 9 | BC-2.04.014 |
| [HS-046](HS-046-real-world-clean-pcap.md) | Real-World Known-Good PCAP Produces Low False-Positive Rate | real-world-corpus | must-pass | 8, 9, 12 | BC-2.04.006, BC-2.04.035, BC-2.05.001 |
| [HS-047](HS-047-real-world-evasion-corpus.md) | Real-World Known-Problematic PCAP Detects TCP Evasion Signatures | real-world-corpus | must-pass | 10 | BC-2.04.018, BC-2.04.019, BC-2.04.041 |
| [HS-048](HS-048-per-direction-depth-independence.md) | Depth Truncation in One Direction Leaves Other Direction Intact | edge-case-combinations | must-pass | 10 | BC-2.04.041, BC-2.04.027 |
| [HS-050](HS-050-evasion-latch-per-direction-independence.md) | Anomaly Alert Latches Are Per-Direction — Both Can Fire Independently | edge-case-combinations | must-pass | 10 | BC-2.04.022, BC-2.04.021, BC-2.04.020 |

### Epic E-3: Content-First Protocol Dispatch (Waves 12-14)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-038](HS-038-content-first-beats-port.md) | TLS on Non-Standard Port Is Detected by Content, Not Port | security-probes | must-pass | 12 | BC-2.05.001, BC-2.05.002, BC-2.05.003 |
| [HS-039](HS-039-classification-cache-stability.md) | Classification Cache Is Immutable and Retry Budget Eventual | behavioral-subtleties | must-pass | 13 | BC-2.05.005, BC-2.05.006 |
| [HS-040](HS-040-unclassified-flows-counter.md) | Unclassified Flow Counter Accurately Reflects Coverage Gaps | behavioral-subtleties | must-pass | 14 | BC-2.05.007, BC-2.05.009 |
| [HS-045](HS-045-dispatcher-no-analyzer-guard.md) | Dispatcher With No Analyzers Configured Does Not Process Data | behavioral-subtleties | must-pass | 14 | BC-2.05.008 |
| [HS-049](HS-049-dispatcher-port-fallback-canonical.md) | Port Fallback Uses Canonical Port Ordering for Non-Standard Source Ports | behavioral-subtleties | must-pass | 12 | BC-2.05.003 |

### Epic E-4: HTTP Traffic Analysis and Threat Detection (Waves 15-18)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-051](HS-051-http-request-parsing-baseline.md) | HTTP Pipelined Requests and Partial Buffering Correctness | behavioral-subtleties | must-pass | 15-18 | BC-2.06.001, BC-2.06.002, BC-2.06.003, BC-2.06.004 |
| [HS-053](HS-053-http-path-traversal-detection.md) | URI Threat Detections Fire Correctly and Independently | security-probes | must-pass | 15-18 | BC-2.06.005, BC-2.06.006, BC-2.06.007, BC-2.06.012 |
| [HS-054](HS-054-http-poisoning-state-machine.md) | HTTP Poisoning Is Per-Direction and Counted Once Per Flow | edge-case-combinations | must-pass | 15-18 | BC-2.06.013, BC-2.06.015, BC-2.06.016, BC-2.06.017, BC-2.06.018, BC-2.06.020 |
| [HS-058](HS-058-http-header-anomaly-detections.md) | HTTP Header Anomaly Detections Are Independent and Threshold-Correct | security-probes | must-pass | 15-18 | BC-2.06.008, BC-2.06.009, BC-2.06.010, BC-2.06.011 |
| [HS-060](HS-060-http-flow-lifecycle-and-caps.md) | HTTP Flow Close Resets Per-Flow State Without Affecting Aggregate Counters | integration-boundaries | must-pass | 15-18 | BC-2.06.019, BC-2.06.021, BC-2.06.022, BC-2.06.024, BC-2.06.025 |
| [HS-061](HS-061-http-summary-output-shape.md) | HTTP Analyzer Summary Is Complete, Deterministic, and Reflects Response-Only Transaction Count | integration-boundaries | must-pass | 15-18 | BC-2.06.023 |
| [HS-065](HS-065-http-too-many-headers-finding.md) | TooManyHeaders Emits Exactly One Finding and Contributes to Poison Counter | security-probes | must-pass | 15-18 | BC-2.06.014, BC-2.06.016, BC-2.06.020 |
| [HS-067](HS-067-http-real-world-clean-traffic.md) | Known-Good HTTP Traffic Corpus Produces Zero False-Positive Findings | real-world-corpus | must-pass | 15-18 | BC-2.06.001, BC-2.06.004, BC-2.06.012, BC-2.06.023 |
| [HS-070](HS-070-http-tls-cross-subsystem-interaction.md) | HTTP and TLS Analyzers Operate Independently on Same pcap Without Cross-Contamination | integration-boundaries | must-pass | 15-18 | BC-2.06.001, BC-2.06.013, BC-2.07.001, BC-2.07.030 |
| [HS-072](HS-072-http-utf8-lossy-header-values.md) | HTTP Header Values With Non-UTF-8 Bytes Are Stored With Replacement Characters | behavioral-subtleties | must-pass | 15-18 | BC-2.06.026, BC-2.06.001 |

### Epic E-5: TLS Traffic Analysis and Fingerprinting (Waves 15-19)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-052](HS-052-ja3-grease-determinism.md) | JA3 Fingerprint Matches Known-Good Reference Value | behavioral-subtleties | must-pass | 15-19 | BC-2.07.006, BC-2.07.007, BC-2.07.008 |
| [HS-055](HS-055-tls-clienthello-done-short-circuit.md) | TLS Analyzer Counts Handshakes Once and Ignores Post-Handshake Data | behavioral-subtleties | must-pass | 15-19 | BC-2.07.001, BC-2.07.003, BC-2.07.034, BC-2.07.032 |
| [HS-056](HS-056-sni-control-byte-detection.md) | SNI Control-Byte Obfuscation Detected With Exact Boundary Semantics | security-probes | must-pass | 15-19 | BC-2.07.013, BC-2.07.014, BC-2.07.015, BC-2.07.016, BC-2.07.018 |
| [HS-057](HS-057-sni-non-ascii-utf8-arm3.md) | Non-ASCII UTF-8 and Invalid UTF-8 SNI Bytes Produce T1027 Findings With Raw Byte Preservation | security-probes | must-pass | 15-19 | BC-2.07.017, BC-2.07.019, BC-2.07.020, BC-2.07.021, BC-2.07.037 |
| [HS-059](HS-059-tls-weak-cipher-findings.md) | Weak Cipher and Deprecated Protocol Findings Are Confidence-Correct and Independent | security-probes | must-pass | 15-19 | BC-2.07.009, BC-2.07.010, BC-2.07.011, BC-2.07.012, BC-2.07.030, BC-2.07.036 |
| [HS-062](HS-062-tls-buffer-and-record-limits.md) | TLS Oversized Records and Buffer Cap Enforced Without Panic | edge-case-combinations | must-pass | 15-19 | BC-2.07.004, BC-2.07.005, BC-2.07.029, BC-2.07.033, BC-2.07.035 |
| [HS-063](HS-063-sni-edge-cases-empty-and-large.md) | SNI Edge Cases — Empty List, Multi-Name, Large SNI, and Count-Cap Decoupling | edge-case-combinations | must-pass | 15-19 | BC-2.07.022, BC-2.07.023, BC-2.07.024, BC-2.07.025, BC-2.07.026, BC-2.07.027, BC-2.07.028 |
| [HS-066](HS-066-tls-summarize-output-completeness.md) | TLS Analyzer Summarize Output Has All Required Keys With Correct Semantics | integration-boundaries | must-pass | 15-19 | BC-2.07.031 |
| [HS-068](HS-068-tls-real-world-modern-session.md) | Known-Good TLS 1.3 Traffic Produces Zero Findings and Correct JA3 Fingerprints | real-world-corpus | must-pass | 15-19 | BC-2.07.001, BC-2.07.002, BC-2.07.030, BC-2.07.031, BC-2.07.034 |
| [HS-069](HS-069-sni-non-utf8-hex-key-uniqueness.md) | Two Invalid UTF-8 SNI Byte Sequences With Same Lossy Form Produce Distinct sni_counts Keys | edge-case-combinations | must-pass | 15-19 | BC-2.07.019, BC-2.07.020 |
| [HS-071](HS-071-tls-server-hello-version-tracking.md) | ServerHello Version Tracked Independently From ClientHello Version | behavioral-subtleties | must-pass | 15-19 | BC-2.07.002, BC-2.07.003 |
| [HS-074](HS-074-tls-ssl30-real-world-pcap.md) | Known-Problematic SSL 3.0 pcap Generates Expected Deprecated-Protocol Findings | real-world-corpus | must-pass | 15-19 | BC-2.07.011, BC-2.07.012, BC-2.07.009, BC-2.07.010 |

### Epic E-6: DNS Traffic Statistics (Wave 4)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-011](HS-011-dns-statistics-never-emit-findings.md) | DNS — Query/Response Counting Without Emitting Any Findings | behavioral-subtleties | must-pass | 4 | BC-2.08.001, BC-2.08.002, BC-2.08.003, BC-2.08.004 |
| [HS-020](HS-020-dns-and-tcp-parallel-wave4.md) | Cross-Subsystem Wave 4 — DNS Statistics Alongside TCP Reassembly | integration-boundaries | must-pass | 4 | BC-2.04.003, BC-2.08.001, BC-2.08.004 |

### Epic E-7: Forensic Finding Data Model and MITRE Mapping (Waves 1-3)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-006](HS-006-finding-display-format-and-verdict-tokens.md) | Finding One-Liner Format — All Verdict and Confidence Combinations | behavioral-subtleties | must-pass | 1 | BC-2.09.002, BC-2.09.003, BC-2.09.004 |
| [HS-007](HS-007-json-serialization-skip-none-fields.md) | JSON Finding Serialization — None Fields Omitted, Raw Bytes Preserved | integration-boundaries | must-pass | 1, 2 | BC-2.09.001, BC-2.09.005, BC-2.09.006 |
| [HS-008](HS-008-mitre-tactic-display-and-kill-chain-order.md) | MITRE ATT&CK Tactic Display Names and Kill-Chain Order Completeness | behavioral-subtleties | must-pass | 3 | BC-2.10.001, BC-2.10.003, BC-2.10.004, BC-2.10.005 |
| [HS-009](HS-009-mitre-technique-lookup-unknown-ids.md) | MITRE Technique Catalog — Known ID Lookup, Unknown ID Graceful Handling | behavioral-subtleties | must-pass | 3 | BC-2.10.005, BC-2.10.006, BC-2.10.007, BC-2.10.008 |
| [HS-017](HS-017-cross-subsystem-e1-e7-finding-construction.md) | E-1 to E-7 Cross-Subsystem — Packet Ingestion Feeds Finding Construction | integration-boundaries | must-pass | 1, 2 | BC-2.01.002, BC-2.09.001, BC-2.09.006, BC-2.10.005 |
| [HS-018](HS-018-raw-data-contract-no-escape-in-json.md) | Forensic Fidelity — Attacker-Controlled Bytes Preserved in JSON, Not Escaped | security-probes | must-pass | 2 | BC-2.09.005, BC-2.09.006 |
| [HS-025](HS-025-ics-tactic-display-and-non-exhaustive.md) | ICS Tactic Display and Non-Exhaustive Enum Stability | behavioral-subtleties | **should-pass** | 3 | BC-2.10.002, BC-2.10.004, BC-2.10.009 |

### Epic E-8: Reporting and Output Formats (Waves 20-22)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-064](HS-064-json-reporter-schema-and-encoding.md) | JSON Reporter Output Matches Stable Schema and Encodes Forensic Bytes Correctly | integration-boundaries | must-pass | 20-22 | BC-2.11.001, BC-2.11.002, BC-2.11.003, BC-2.11.004, BC-2.11.005 |
| [HS-073](HS-073-json-c0-c1-mixed-finding.md) | JSON Reporter Treats C0 and C1 Bytes Differently in the Same Finding | behavioral-subtleties | must-pass | 20-22 | BC-2.11.003, BC-2.11.005 |
| [HS-075](HS-075-json-reporter-skipped-packets-always-present.md) | JSON Reporter Includes skipped_packets Key Even When Zero and Output Is Parseable by jq | integration-boundaries | must-pass | 20-22 | BC-2.11.001, BC-2.11.002 |
| [HS-076](HS-076-terminal-c1-injection-in-finding-summary.md) | Terminal Output Contains No Raw C1 Control Bytes When Finding Summary Has Attacker-Injected CSI | security-probes | must-pass | 20-22 | BC-2.11.007, BC-2.11.009, BC-2.11.010, BC-2.11.012 |
| [HS-077](HS-077-terminal-legitimate-unicode-passes-through.md) | Legitimate Unicode (Cyrillic, Emoji, NBSP) Survives Terminal Output Unchanged | behavioral-subtleties | must-pass | 20-22 | BC-2.11.008, BC-2.11.009 |
| [HS-078](HS-078-terminal-skipped-packets-conditional-display.md) | Skipped-Packets Warning Appears Iff Decode Errors Were Encountered | behavioral-subtleties | must-pass | 20-22 | BC-2.11.006 |
| [HS-079](HS-079-csv-injection-neutralization-formula-chars.md) | CSV Output Neutralizes Formula-Injection Characters in Every Column | security-probes | must-pass | 20-22 | BC-2.11.021, BC-2.11.020 |
| [HS-080](HS-080-csv-nine-column-schema-stable.md) | CSV Output Has Exactly Nine Columns and Correct Header in All Conditions | integration-boundaries | must-pass | 20-22 | BC-2.11.020, BC-2.11.022, BC-2.11.023 |
| [HS-081](HS-081-terminal-mitre-grouping-kill-chain-order.md) | MITRE Grouping Presents Tactics in Kill-Chain Order with Correct Sorting | behavioral-subtleties | must-pass | 20-22 | BC-2.11.013, BC-2.11.014, BC-2.11.015, BC-2.11.016 |
| [HS-082](HS-082-terminal-color-disabled-no-ansi-codes.md) | --no-color Strips All ANSI Escape Codes; Section Order Is Correct | behavioral-subtleties | must-pass | 20-22 | BC-2.11.018, BC-2.11.019, BC-2.11.017 |
| [HS-083](HS-083-csv-optional-fields-none-encoded-as-empty.md) | CSV Optional Fields Use Empty Strings for None; Direction Is CamelCase Debug | behavioral-subtleties | must-pass | 20-22 | BC-2.11.024, BC-2.11.023 |
| [HS-092](HS-092-csv-injection-plus-evidence-join-combined.md) | CSV Evidence Join Then Injection-Neutralization Combined Edge Case | edge-case-combinations | must-pass | 20-22 | BC-2.11.021, BC-2.11.022, BC-2.11.020 |
| [HS-093](HS-093-terminal-escape-both-summary-and-evidence.md) | Escape Applied Independently to Summary, Each Evidence Line, and Analyzer Detail Values | edge-case-combinations | must-pass | 20-22 | BC-2.11.010, BC-2.11.011, BC-2.11.007 |
| [HS-098](HS-098-end-to-end-pcap-to-csv-report.md) | End-to-End pcap -> CSV Output Is Parseable and Injection-Safe (Real-World Corpus) | real-world-corpus | must-pass | 20-22 | BC-2.11.020, BC-2.11.021, BC-2.11.022, BC-2.11.023, BC-2.11.024 |
| [HS-099](HS-099-terminal-backslash-escape-in-windows-paths.md) | Backslash in Finding Summary Is Escaped to Double-Backslash in Terminal Output | edge-case-combinations | must-pass | 20-22 | BC-2.11.007, BC-2.11.008, BC-2.11.010 |

### Epic E-9: CLI, Entry Point, and Analysis Orchestration (Waves 23-27)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-084](HS-084-cli-subcommand-structure-required-targets.md) | CLI Subcommand Parsing Enforces Required Targets and Correct Flag Semantics | integration-boundaries | must-pass | 23-27 | BC-2.12.001, BC-2.12.002, BC-2.12.003, BC-2.12.006 |
| [HS-085](HS-085-cli-reassemble-no-reassemble-conflict.md) | --reassemble and --no-reassemble Together Are Rejected; Output Format Flags Work Independently | edge-case-combinations | must-pass | 23-27 | BC-2.12.007, BC-2.12.004, BC-2.12.005 |
| [HS-087](HS-087-run-analyze-all-flag-analyzer-enablement.md) | --all Enables All Three Analyzers; --no-reassemble Produces Warning and Skips HTTP/TLS | integration-boundaries | must-pass | 23-27 | BC-2.12.008, BC-2.12.009, BC-2.12.010, BC-2.12.011 |
| [HS-088](HS-088-output-format-flag-precedence-routing.md) | --json Flag Wins Over --output-format; Output Routes to File or Stdout Correctly | edge-case-combinations | must-pass | 23-27 | BC-2.12.016, BC-2.12.017, BC-2.12.014 |
| [HS-089](HS-089-summary-model-ingest-unique-hosts-service-hints.md) | Summary Accumulates Correct Counts; unique_hosts Is Sorted and Deduplicated | behavioral-subtleties | must-pass | 23-27 | BC-2.12.018, BC-2.12.019, BC-2.12.020, BC-2.12.021 |
| [HS-090](HS-090-end-to-end-pcap-to-json-report.md) | End-to-End pcap -> JSON Report Pipeline (Real-World Clean Corpus) | real-world-corpus | must-pass | 23-27 | BC-2.12.001, BC-2.12.008, BC-2.12.016, BC-2.12.021, BC-2.11.001 |
| [HS-091](HS-091-end-to-end-pcap-to-terminal-known-problematic.md) | End-to-End pcap -> Terminal Report on Known-Problematic Corpus (False Negative Test) | real-world-corpus | must-pass | 23-27 | BC-2.12.001, BC-2.12.008, BC-2.11.007, BC-2.11.013, BC-2.11.019 |
| [HS-094](HS-094-cli-overlap-threshold-range-enforced.md) | Reassembly Threshold Flags Enforce Numeric Ranges at Parse Time | edge-case-combinations | must-pass | 23-27 | BC-2.12.005 |
| [HS-095](HS-095-unclassified-flows-injected-into-reassembly-summary.md) | Unclassified Flows Count Appears in Reassembly Summary; Absent Without Reassembler | behavioral-subtleties | must-pass | 23-27 | BC-2.12.015, BC-2.12.014 |
| [HS-096](HS-096-no-color-env-var-disables-ansi.md) | NO_COLOR Environment Variable Disables ANSI Output Regardless of --no-color Flag | behavioral-subtleties | must-pass | 23-27 | BC-2.12.010 |
| [HS-097](HS-097-nonexistent-target-error-message.md) | Non-Existent Target Path Produces Descriptive Error Message with Path Included | integration-boundaries | must-pass | 23-27 | BC-2.12.012, BC-2.12.011 |
| [HS-100](HS-100-summary-json-protocol-keys-debug-format.md) | JSON Summary Uses Debug-Format Protocol Keys (CamelCase, Not Uppercase) | behavioral-subtleties | must-pass | 23-27 | BC-2.12.021, BC-2.12.018, BC-2.12.019 |

### Epic E-10: Absent Behavior Contracts — Flag Rejection (Wave 24)

| HS ID | Title | Category | Priority | Waves | Behavioral Contracts |
|-------|-------|----------|----------|-------|---------------------|
| [HS-086](HS-086-removed-flags-rejected-by-clap.md) | Obsolete Flags --threats, --beacon, --filter, --verbose Are Actively Rejected | behavioral-subtleties | must-pass | 24 | BC-2.13.001, BC-2.13.002, BC-2.13.003, BC-2.13.004 |

---

## Anomalies

None detected. All checks passed for the greenfield set:

- HS-001 through HS-100 present with no gaps or duplicates (greenfield HS-NNN sequence)
- All 100 `behavioral_contracts` fields are non-empty
- All 27 waves (1-27) have at least one scenario
- One should-pass scenario: HS-025 (ICS Tactic Display — lower priority feature)
- All 100 HS files carry concrete per-file `inputs` listing specific BC and story paths

> **Note:** This index covers the v0.1.0 greenfield holdout set (HS-NNN sequence, waves 1-27).
> Feature-mode holdouts for SS-15 DNP3 (v0.6.0, waves 35-39) use the HS-W35-NNN / HS-W38-NNN
> namespace and are tracked separately in the feature holdout tree — see the
> "Feature Holdouts (SS-15 DNP3, waves 35-39)" section below.
> Feature-mode holdout SEEDS for SS-16 ARP (v0.7.0, estimated waves 40-44) use the
> HS-W40-NNN / HS-W44-NNN namespace — see the "Feature Holdouts (SS-16 ARP, waves 40-44)"
> section below. Full scenarios are authored in Phase 4 by the holdout-evaluator.

---

## Feature Holdouts (SS-15 DNP3, waves 35-39)

> **Source file:** `.factory/feature/wave-holdout-scenarios/wave-35-39-holdout.md`
>
> These holdouts belong to the v0.6.0 DNP3 feature cycle (issue-008-dnp3-analyzer).
> They use the `HS-W<wave>-<seq>` namespace and are NOT part of the greenfield HS-NNN sequence.
> The HS-001..HS-100 completeness assertions above are scoped to the greenfield set only.
>
> Stories: STORY-106 (wave 35), STORY-107 (wave 36), STORY-108 (wave 37), STORY-109 (wave 38), STORY-110 (wave 39).
> MITRE version: ics-attack-19.1. T0855 and T0803 are REVOKED and must never appear.

### Wave 35 — Pure-Core Parse, Classify, and VP-023 Kani (STORY-106)

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W35-001 | DL Header Parse — Canonical 10-Byte Minimum Vector | P0 | BC-2.15.001, BC-2.15.003 |
| HS-W35-002 | DL Header Parse — Extended Canonical Frame (BC byte-level vector) | P0 | BC-2.15.001, BC-2.15.003 |
| HS-W35-003 | DL Header Parse — Truncation Rejection and LE Disambiguation | P0 | BC-2.15.002, BC-2.15.003 |
| HS-W35-004 | Three-Point Validity Gate — Biconditional Exhaustive | P0 | BC-2.15.004 |
| HS-W35-005 | FC Classification — Totality and Set Membership | P0 | BC-2.15.005, BC-2.15.006 |
| HS-W35-006 | compute_dnp3_frame_len — Formula Correctness at Boundaries | P0 | BC-2.15.007 |
| HS-W35-007 | Transport FIR=1 Gating — Extract vs Skip | P0 | BC-2.15.008 |
| HS-W35-008 | Desync Bail — Non-DNP3 Traffic Silenced | P0 | BC-2.15.009 |

### Wave 36 — Carry Buffer and Pending-Requests Bounds (STORY-107)

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W36-001 | Carry Buffer — Accumulate and Cap at 292 | P0 | BC-2.15.016 |
| HS-W36-002 | Pending-Requests — Bounded at 256 with Oldest-Eviction | P0 | BC-2.15.016 |

### Wave 37 — Direct Detections: T1692.001, T0814, T0836, Co-Emission, Summarize (STORY-108)

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W37-001 | T1692.001 — Direct-Operate Burst at Threshold Boundary | P0 | BC-2.15.010 |
| HS-W37-002 | T1692.001 — Unexpected Source Fires at Count=1 (canonical DIR-bit holdout) | P0 | BC-2.15.010 Invariant 5 |
| HS-W37-003 | T0814 — COLD_RESTART and WARM_RESTART Per-Occurrence (No Threshold) | P0 | BC-2.15.011 |
| HS-W37-004 | T0836 — WRITE Per-Occurrence; NOT Also T1692.001 | P0 | BC-2.15.012 |
| HS-W37-005 | Co-Emission Ordering — Direct Finding Before Derived T0827 | P0 | BC-2.15.013 |
| HS-W37-006 | summarize() — Function-Code Distribution and Zero-Flow Case | P0 | BC-2.15.020 |

### Wave 38 — Correlated/Anomaly: T1691.001, T0827, Broadcast, Unsolicited, DISABLE, Malformed (STORY-109)

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W38-001 | T1691.001 — Block-Command 3-of-300s Threshold | P0 | BC-2.15.014 |
| HS-W38-002 | T1691.001 — Block Events Not Reset at 120s (Trace B Regression) | P0 | BC-2.15.014, BC-2.15.015 |
| HS-W38-003 | T0827 — Combined Restart + Block Accumulation (Trace B) | P0 | BC-2.15.015 |
| HS-W38-004 | Correlation Window — Six-Field Expiry Reset | P0 | BC-2.15.015 |
| HS-W38-005 | Broadcast Control Anomaly — DEST in 0xFFFD/0xFFFE/0xFFFF | P0 | BC-2.15.018 |
| HS-W38-006 | Unsolicited Response Anomaly — UNS Bit / FC=0x82 Without Prior ENABLE | P1 | BC-2.15.019 |
| HS-W38-007 | DISABLE_UNSOLICITED T0814 (Likely/Medium) and ENABLE T0814 (Possible/Low) | P0 | BC-2.15.023 |
| HS-W38-008 | Malformed-Frame Anomaly — 3-of-300s Crain-Sistrunk-Style Threshold | P0 | BC-2.15.024 |
| HS-W38-009 | Negative / False-Positive Guard — Legitimate Low-Rate Control | P0 | (guard for BC-2.15.010, BC-2.15.014) |

### Wave 39 — End-to-End Dispatch, CLI Threshold Flag, VP-004 Oracle (STORY-110)

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W39-001 | Dispatcher — Port-20000 Routes to Dnp3Analyzer (Rule 6) | P0 | BC-2.15.021 |
| HS-W39-002 | Content-First Precedence — TLS/HTTP on Port 20000 Not Stolen | P0 | BC-2.15.021, VP-004 |
| HS-W39-003 | Non-DNP3 Traffic on Port 20000 — is_non_dnp3 Bail, No False Findings | P0 | BC-2.15.021, BC-2.15.009 |
| HS-W39-004 | --dnp3-direct-operate-threshold CLI Flag — Override Changes Firing Point | P0 | BC-2.15.017 |
| HS-W39-005 | End-to-End — Crafted DNP3 Synthetic PCAP with Full Detection Surface | P0 | BC-2.15.021 + all wave 37-38 detections |
| HS-W39-006 | Regression on Existing Analyzers After Waves 35-39 | P0 | VP-004, VP-007, VP-022, VP-023 |
| HS-W39-007 | VP-023 Kani Four Sub-Properties — All Pass | P0 | VP-023 (BC-2.15.001 through BC-2.15.008) |

### Feature Holdout Summary

| Metric | Count |
|--------|-------|
| Total DNP3 feature holdouts | 32 |
| P0 must-pass | 31 |
| P1 nice-to-have | 1 (HS-W38-006) |
| Waves covered | 35, 36, 37, 38, 39 |
| Stories covered | STORY-106, STORY-107, STORY-108, STORY-109, STORY-110 |
| Source file | `.factory/feature/wave-holdout-scenarios/wave-35-39-holdout.md` |

> **Canonical DIR-bit holdout:** HS-W37-002 is the authoritative test for the corrected
> `is_master_frame` bitmask (0x80, bit 7 = DIR). It verifies that unexpected-source detection
> is independent of the burst-count threshold (F-F5-001 REVISION 2 R2-5 amendment).

---

## Feature Holdouts (SS-16 ARP Security Analyzer, waves 40-44)

> **Source file:** `.factory/feature/wave-holdout-scenarios/wave-40-44-holdout.md` (skeleton created by product-owner F3 hand-off, 2026-06-13; Phase 4 holdout-evaluator completes concrete byte vectors)
>
> These holdout SEEDS belong to the v0.7.0 ARP feature cycle (issue-009-arp-security-analyzer).
> They use the `HS-W<wave>-<seq>` namespace consistent with the DNP3 holdout convention.
> They are SEEDS ONLY — full holdout scenarios with concrete PCAP vectors are authored by the
> holdout-evaluator in Phase 4 after F3 story decomposition and wave assignments are confirmed.
> The HS-001..HS-100 completeness assertions above are scoped to the greenfield set only.
>
> **Canonical story decomposition (from arp-architecture-delta.md §6):**
> STORY-111 (wave 40): etherparse migration + DecodedFrame enum + BC-2.02.009 revision
> STORY-112 (wave 41): extract_arp_frame + ArpAnalyzer stub + VP-024 Sub-A Kani
> STORY-113 (wave 42): ArpAnalyzer full impl (binding table, D2 GARP, D11 finding, D12 mismatch, summarize, --arp) + VP-024 Sub-B/C/D
> STORY-114 (wave 43): D1 spoof HIGH escalation + MITRE emission + VP-007 5-part atomic update
> STORY-115 (wave 44): D3 storm detection + --arp-storm-rate CLI flag + wires value of BC-2.16.010 storm_findings key (key 8 of 11; defined from STORY-113)
>
> **MITRE techniques:** T0830 (Adversary-in-the-Middle, `MitreTactic::LateralMovement`);
> T1557.002 (ARP Cache Poisoning, `MitreTactic::CredentialAccess`).
> (Corrected from v1.1: IcsImpairProcessControl was incorrect for T0830; merge-by-name per
> mitre.rs convention maps T0830 → LateralMovement per arp-architecture-delta.md §5.)

### Wave 40 — etherparse Migration, DecodedFrame Enum, BC-2.02.009 Revision (STORY-111)

> Gate: `decode_packet` return type changed to `Result<DecodedFrame>`; three-way postcondition
> (Ethernet/IPv4 ARP → DecodedFrame::Arp; non-Ethernet/IPv4 ARP → E-DEC-004; non-IP non-ARP →
> "No IP layer found"); VP-008 fuzz harness updated to accept Result<DecodedFrame>.

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W40-001 | DecodedFrame Enum — Ethernet/IPv4 ARP produces DecodedFrame::Arp variant (three-way BC-2.02.009 postcondition; ArpFrame field-value correctness is wave-41 scope) | P0 | BC-2.02.009 (revised) |
| HS-W40-002 | Non-Ethernet/IPv4 ARP → E-DEC-004 degraded skip; no DecodedFrame::Arp | P0 | BC-2.02.009 (revised) |
| HS-W40-003 | Non-IP non-ARP → "No IP layer found" (unchanged); no regression | P0 | BC-2.02.009 (Path 3) |
| HS-W40-004 | VP-008 fuzz harness accepts Result<DecodedFrame>; no-panic invariant unchanged | P0 | VP-008 (return-type update) |

### Wave 41 — ARP Frame Extraction, ArpAnalyzer Stub, VP-024 Sub-A Kani (STORY-112)

> Gate: `extract_arp_frame` pure-core correctness (Request and Reply); SLL outer_src_mac=None
> propagation; VP-024 Sub-property A (extraction safety + field correctness) Kani harnesses pass.

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W41-001 | ARP Request — Happy-Path Extraction: all six address fields copied correctly | P0 | BC-2.16.001 |
| HS-W41-002 | ARP Reply — Happy-Path Extraction: op=2, sender/target MACs and IPs | P0 | BC-2.16.002 |
| HS-W41-003 | extract_arp_frame None → returns None on bad hw/proto size; decode-vs-analysis separation: DecodedFrame::Arp always produced for valid Ethernet/IPv4 ARP (finding emission deferred to ArpAnalyzer); VP-024 Sub-A negative harness | P0 | BC-2.16.015, VP-024 Sub-A |
| HS-W41-004 | SLL capture: outer_src_mac=None propagated faithfully into DecodedFrame::Arp; decode-vs-analysis separation upheld (no D12 finding emitted at decode stage) | P0 | BC-2.16.001, BC-2.16.015 |

### Wave 42 — ArpAnalyzer Full Implementation: Binding Table, GARP, D11 Finding, D12, Summarize (STORY-113)

> Gate: binding-table insert/eviction (HashMap<[u8;4], BindingEntry>); GARP LOW vs MEDIUM
> escalation; D11 malformed finding; D12 mismatch finding; summarize() shape; --arp flag;
> VP-024 Sub-B (GARP biconditional Kani), Sub-C (last-write-wins proptest), Sub-D (cap Kani).

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W42-001 | GARP Benign Baseline — GARP with no conflict produces LOW finding; mitre_techniques: [] (no MITRE techniques attributed to benign GARP per D-068); no D1 spoof; VP-024 Sub-B | P0 | BC-2.16.003, VP-024 Sub-B |
| HS-W42-002 | Binding-Table Last-Write-Wins — arbitrary frame sequence; VP-024 Sub-C proptest | P0 | BC-2.16.005, VP-024 Sub-C |
| HS-W42-003 | Binding-Table Cap — 65,537th distinct IP evicts min-ts entry; len never exceeds 65,536; VP-024 Sub-D | P0 | BC-2.16.006, VP-024 Sub-D |
| HS-W42-004 | L2/L3 Mismatch — Ethernet outer MAC differs from ARP sender HW addr: MEDIUM finding; mitre_techniques: [] at wave 42 (MITRE attachment deferred to STORY-114, wave 43 — see HS-W43-005) | P0 | BC-2.16.007 |
| HS-W42-005 | D11 Malformed ARP — non-Ethernet/IPv4 hw/proto sizes produce LOW finding; malformed_frames incremented | P0 | BC-2.16.009, BC-2.16.010 |
| HS-W42-006 | summarize() — 11 required keys present (incl. other_opcode_count); frames_analyzed excludes malformed; malformed_frames correct; reconciliation invariant request_count+reply_count+other_opcode_count==frames_analyzed holds | P0 | BC-2.16.010 |
| HS-W42-007 | Negative / False-Positive Guard — legitimate ARP conversation (stable IP→MAC bindings, no rebind) produces zero findings; binding-table last-write-wins updates correctly | P0 | BC-2.16.005, BC-2.16.003 |
| HS-W42-008 | --arp flag gates analysis — DecodedFrame::Arp produced regardless of --arp flag; ARP findings emitted only when --arp active; without --arp, ArpAnalyzer not invoked | P0 | BC-2.16.011 |

### Wave 43 — D1 ARP Spoof HIGH Escalation, MITRE Emission, VP-007 Atomic (STORY-114)

> Gate: D1 spoof MEDIUM→HIGH escalation (rebind_count >= threshold within window);
> --arp-spoof-threshold=1 → HIGH on first rebind; GARP-that-conflicts dual-finding;
> T0830 (LateralMovement) + T1557.002 (CredentialAccess) emitted; VP-007 5-part atomic
> update passes `cargo test mitre`.

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W43-001 | D1 Spoof — IP→MAC rebind MEDIUM then HIGH within 60s; T0830+T1557.002 emitted | P0 | BC-2.16.004 |
| HS-W43-002 | --arp-spoof-threshold 1 — HIGH on first rebind (no MEDIUM first); T0830+T1557.002 | P0 | BC-2.16.004 EC-008, BC-2.16.012 |
| HS-W43-003 | GARP-That-Conflicts — GARP MEDIUM + D1 finding (MEDIUM or HIGH per escalation state) | P0 | BC-2.16.014, BC-2.16.004 |
| HS-W43-004 | VP-007 Atomic — T0830 + T1557.002 arms in technique_info; SEEDED=25; EMITTED=17; cargo test mitre green (after STORY-114 merges) | P0 | VP-007 (5-part atomic update) |
| HS-W43-005 | D12 MITRE Attachment — same outer-MAC-mismatch frame as HS-W42-004 now carries mitre_techniques: [T0830, T1557.002]; technique_info arms resolve (LateralMovement, CredentialAccess per ADR-008 Decision 6); co-committed with src/mitre.rs catalog seeding (Pass-12 D12-MITRE sequencing fix; see BC-2.16.007's cross-story delivery note and wave-40-44-holdout.md HS-W42-ARP-D11D12 Scenario D) | P0 | BC-2.16.007, VP-007 |

### Wave 44 — D3 Storm Detection, CLI Flags, End-to-End (STORY-115)

> Gate: D3 storm one-shot per MAC per 60s window; rate formula `count/max(1,elapsed)` correct;
> --arp-storm-rate CLI override (BC-2.16.013); storm_findings key (key 8 of 11 in BC-2.16.010,
> defined from STORY-113) value wired by STORY-115; end-to-end PCAP → JSON report contains ARP
> storm findings; regression on SS-02/SS-05/SS-14/SS-15. Note: --arp-spoof-threshold is
> STORY-114 scope (BC-2.16.012, wave 43) — NOT tested in wave 44.

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W44-001 | D3 Storm — source MAC exceeds threshold: one-shot MEDIUM finding per 60s window; rate=count/max(1,elapsed) | P0 | BC-2.16.008 |
| HS-W44-002 | Same-Second Storm Denominator — all frames at ts=N: max(1,0)=1; rate=count; no divide-by-zero | P0 | BC-2.16.008 EC-002 (ARP-AMB-003 RESOLVED) |
| HS-W44-003 | --arp-storm-rate override — custom rate threshold changes storm detection | P1 | BC-2.16.013 |
| HS-W44-004 | storm_findings summarize() key — populates the existing BC-2.16.010 storm_findings key (key 8 of 11; defined from STORY-113, value wired by STORY-115); count >= 1 after storm detection | P0 | BC-2.16.010 (storm_findings key, value wired by STORY-115), BC-2.16.008 |
| HS-W44-005 | Known-Good ARP Corpus — legitimate LAN traffic with ARP produces zero false-positive findings | P0 | real-world corpus: known-good (Wireshark sample LAN traffic with ARP resolution) |
| HS-W44-006 | Known-Problematic ARP Corpus — crafted pcap with ARP spoofing produces T0830+T1557.002 findings | P0 | real-world corpus: known-problematic (crafted or CTF ARP poisoning pcap) |
| HS-W44-007 | Regression on Existing Analyzers After Waves 40-44 — no regression on SS-02, SS-05, SS-14, SS-15 | P0 | VP-008 (no-panic fuzz update), VP-004 (dispatcher), BC-2.02.009 (revised) |

### ARP Feature Holdout Summary (Seeds)

| Metric | Count |
|--------|-------|
| Total ARP feature holdout seeds | 28 |
| P0 must-pass seeds | 27 |
| P1 nice-to-have seeds | 1 (HS-W44-003) |
| Waves covered (estimated) | 40, 41, 42, 43, 44 |
| Stories covered (estimated) | STORY-111, STORY-112, STORY-113, STORY-114, STORY-115 |
| Skeleton holdout file | `.factory/feature/wave-holdout-scenarios/wave-40-44-holdout.md` (created F3 hand-off) |

> **SEED STATUS:** These are seeds only. Concrete byte-level test vectors, PCAP sources,
> and precise precondition/postcondition assertions are authored by the holdout-evaluator
> agent during Phase 4, AFTER F3 story decomposition produces wave assignments and
> implementation code exists for evaluation. F3 story-writer must create the wave-40-44-holdout.md
> skeleton file referencing these seeds.
>
> **Rewrite note (v1.5 — F3 product-owner hand-off, 2026-06-13):** Four targeted corrections
> to align seeds exactly with arp-architecture-delta.md §6 canonical story decomposition:
> (1) HS-W41-003/004: removed BC-2.16.009 and BC-2.16.007 citations from wave 41; both belong
> to STORY-113 (wave 42). Replaced with BC-2.16.015 (decode-vs-analysis separation), the correct
> third primary for STORY-112. (2) HS-W42-007: removed BC-2.16.004 guard citation (D1 spoof
> is not implemented until STORY-114, wave 43); replaced with BC-2.16.005 and BC-2.16.003
> (the actual wave-42 BCs being protected by the negative guard). (3) HS-W44-004: removed
> BC-2.16.015 (wave 41) and BC-2.16.011 (wave 42) citations from wave 44; reformulated as a
> storm_findings summarize() cross-story extension test (BC-2.16.010 + BC-2.16.008), which is
> the actual wave-44 deliverable. (4) Narrative line and wave-44 gate block: removed
> "--arp-spoof-threshold" from STORY-115 scope; it belongs to STORY-114 (BC-2.16.012, wave 43).
>
> **Rewrite note (v1.2 — F2 adversarial Pass 1 remediation):** Waves 40-44 rewritten to match
> the canonical story decomposition in arp-architecture-delta.md §6. The previous wave
> assignments placed D1/D3/D11/D12/summarize and VP-024 sub-properties inconsistently with
> the dependency chain. The authoritative order (per arch-delta §6) is:
> STORY-111 (migration) → STORY-112 (extraction/Sub-A) → STORY-113 (analyzer+binding+D2/D11/D12+summarize+--arp+Sub-B/C/D) → STORY-114 (D1 spoof escalation+MITRE+VP-007) → STORY-115 (D3 storm+CLI flags).
> BC-2.16.016 reconciliation: the arch-delta §6 STORY-115 row cited "BC-2.16.016 (summarize
> storm key)" — no such BC exists. BC-2.16.010 already includes `storm_findings` as one of
> the 11 required summarize() keys (updated to 10 in v1.1 by adding `malformed_frames`;
> updated to 11 in v1.2 by adding `other_opcode_count` per ADR-008 Decision 7).
> The arch-delta citation is a mis-cite; maps to BC-2.16.010. Similarly, arch-delta cited
> "BC-2.16.014 (storm CLI flag)" in STORY-115 which is also a mis-cite; the storm CLI flag
> BC is BC-2.16.013. HS seeds corrected accordingly.
>
> **Real-world corpus notes:**
> - Known-good: Wireshark wiki `arp-storm.pcap` or any public LAN trace with clean ARP
>   (many ARP requests with expected replies; no rebinds). False-positive target: zero D1/D12 findings.
> - Known-problematic: crafted pcap (or CTF capture) with explicit ARP poisoning sequence
>   (attacker sends unsolicited ARP replies rebinding victim IP to attacker MAC). Expected:
>   T0830 + T1557.002 findings with MEDIUM→HIGH escalation path visible.
>
> **F3 implementation ambiguities resolved (ARP-AMB-003 and ARP-AMB-004 — see PRD v1.10):**
> ARP-AMB-003 RESOLVED: storm-rate formula is `count/max(1,elapsed)` (integer-seconds).
> ARP-AMB-004 RESOLVED: malformed frames excluded from frames_analyzed; tracked in malformed_frames.
> ARP-AMB-001/002/005/006 remain legitimate F3 implementation choices.
> HS-W40-003 depends on ARP-AMB-002 resolution. HS-W42-003 depends on ARP-AMB-001 resolution.
