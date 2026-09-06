---
document_type: epics
version: "2.4"
status: draft
producer: story-writer
phase: 2
timestamp: 2026-05-21T00:00:00Z
modified:
  - "2026-06-17 v1.2: E-18 Terminal Finding-Collapse (issue #259) added — STORY-118 + STORY-119 (deferred). total_bcs 283→288 (+5 new BC-2.11.025–029; 4 existing BCs extended/versioned — count unchanged)."
  - "2026-06-17 v1.3: Adversarial Burst 3 remediation — E-8 story count 7→5 (Estimated Story Count Summary table; actual E-8 roster is STORY-076..080 = 5 stories). Column sum now 72, matching Total row."
  - "2026-06-17 v1.4: Adversarial Burst 4 remediation — Coverage Check body updated to 288 BCs: added E-18 row to Per-Epic BC Assignment table (BC-2.11.025..029, 5), added E-17 row (extensions, 0), updated TOTAL 283→288, updated Arithmetic Verification block (+E-18 line, ✓ 288/288), updated Coverage confirmed assertion 283→288."
  - "2026-06-19 v1.5: F2 pcapng-reader-support re-anchor — E-1 BC list: BC-2.01.004 struck through [RETIRED], BC-2.01.009–018 (10 new SS-01 BCs) added. E-1 SS-01 count 8→17 active (+9 net). E-1 total 23→32. total_bcs 288→297 (net +9: 10 new BC-2.01.009–018 minus 1 retired BC-2.01.004). Arithmetic Verification and Coverage Confirmed updated."
  - "2026-06-19 v1.6: FINDING-002 correction — BC-2.11.030–034 (5 grouped-collapse BCs added in BC-INDEX v1.44 for STORY-119) were missing from epics.md. Added to E-18 row. total_bcs corrected 297→302 (verified against BC-INDEX v1.52 ground truth: 302 active BCs). Arithmetic Verification and Coverage Confirmed updated."
  - "2026-06-20 v1.7: FE-001 INTEGRATE sub-burst — E-19 pcapng Capture-Format Reader Support added (STORY-123..128, 6 stories, 37 points, Waves 51–56). No new BCs — BC-2.01.009..018 and BC-2.12.011 are pre-existing (counted in E-1 and E-9 respectively since v1.5). Estimated Story Count Summary updated: E-19 row added (6), Total 72→78. total_bcs unchanged at 302."
  - "2026-06-24 v1.8: E-20 EtherNet/IP ENIP/CIP Analyzer INTEGRATE sub-burst (issue #316, feature-enip-v0.11.0) — E-20 epic added (STORY-130..138, 9 stories, 66 points, Waves 58–61). 26 new BCs: BC-2.17.001..026 (SS-17 EtherNet/IP analyzer). total_bcs 302→328. Estimated Story Count Summary updated: E-20 row added (9), Total 78→87. Coverage Check Per-Epic BC table updated with E-20 row. Arithmetic Verification and Coverage Confirmed updated."
  - "2026-06-27 v1.9: RULING-DNP3-SIBLING-001 fix story — STORY-140 added to E-15 (wave 63, 8 pts, dep=STORY-139). No new BCs (BC-2.15.016/010/014/015 are pre-existing, amended by ruling). E-15 story count 5→6. E-15 points 47→55. Estimated Story Count Summary E-15 row 5→6, Total 88→89. total_bcs unchanged at 328."
  - "2026-06-28 v2.0: Wave 64 RULING-MODBUS-SIBLING-001 + RULING-DNP3-DESYNC-001 fix stories — STORY-141 added to E-14 (wave 64, 8 pts, dep=[]). STORY-142 added to E-15 (wave 64, 3 pts, dep=STORY-140). No new BCs (BC-2.14.002/016/017/019 and BC-2.15.009 are pre-existing, amended by rulings). E-14 story count 4→5. E-14 points 37→45. E-15 story count 6→7. E-15 points 55→58. Estimated Story Count Summary E-14 row 4→5, E-15 row 6→7, Total 89→91. total_bcs unchanged at 328."
  - "2026-07-02 v2.1: F3 phase gate (feature-protocol-coverage) — E-21 epic added (STORY-151..154, 4 stories, 32 pts, Waves 67–69). 9 new BCs: BC-2.18.001..004 (SS-18 protocol coverage catalog) + BC-2.05.010..011 (SS-05 dispatcher unclassified-port gap counters) + BC-2.12.022..024 (SS-12 protocols subcommand + --coverage-gaps flag). total_bcs 328→337. Post-v2.0 story-count drift reconciled against STORY-INDEX v3.12: E-5 8→11 (+STORY-144/145/146 fix-tls-clienthello-frag F3 2026-06-29); E-8 5→7 (+STORY-120 FindingsRender enum migration + STORY-129 mitre_attack JSON enrichment); E-11 1→6 (+STORY-121/143/147/149/150 process-gap/tooling stories added 2026-06-18..2026-07-01); E-18 2→3 (+STORY-122 enum→struct reshape D-120 split-A 2026-06-18); E-20 10→11 (+STORY-148 on_flow_close wiring + DNP3 flow-map cap maint-2026-07-01). Estimated Story Count Summary Total 91→107. DISCREPANCY NOTE: epics.md pre-E-21 total_bcs 328 was stale by -6 — BC-2.07.038..043 (TLS carry-reassembly BCs, fix-tls-clienthello-frag F3 2026-06-29) are absent from E-5 Per-Epic BC row and Coverage Check table; true pre-E-21 total = 334; this v2.1 corrects for E-21 only (328+9=337), deferring the E-5 BC row update to a subsequent pass. Residual gap vs BC-INDEX v2.13 (345 active) = 8 (= 6 missing TLS BCs + 2 unresolved)."
  - "2026-09-04 v2.2: DRIFT-EPICS-STALE-v21 currency reconciliation (wave-86 human story-approval gate D-544, 'fix both now') — epics.md was frozen at v2.1 (2026-07-02) while STORY-INDEX advanced to v4.19 (136 stories). Full per-epic reconciliation against STORY-INDEX v4.19 'Stories by Epic' table and BC-INDEX v2.37: (1) E-11 6→23 stories / pts n/a→75 (+STORY-155/157/158/159/161/162/163/164/165/166/175/176/177/178/179/182/183; superseded/delivered statuses reflected per STORY-INDEX, none deleted from history; STORY-182/183 now correctly listed as E-11 members per D-544); (2) E-8 7→8 stories, BC row 24→27 (+BC-2.11.035/036/037; STORY-160); (3) E-16 5→6 stories, BC row 15→16 (+BC-2.16.016; STORY-156) — E-16 has no full epic section (pre-existing structural gap predating v2.1, left as-is; membership/counts reconciled via Coverage Check + Summary table only); (4) E-20 11→12 stories, points 74→82 (+STORY-181; STORY-148 already counted at v2.1); (5) new Epic E-22 (IEC-104 Passive Analyzer) added in full — 9 stories / 41 pts / 33 BCs (STORY-167..174, STORY-180; did not exist at v2.1, predates feature-iec104 decomposition D-440 2026-07-14); (6) E-18 2→3 stories (STORY-122 was already in the Summary table since v1.7 but never reflected in the E-18 section body — corrected); (7) E-19 and E-21 stale 'in-progress'/'draft' status lines corrected to reflect STORY-INDEX-confirmed delivery. E-5 BC discrepancy (flagged unresolved at v2.1, -6 BCs) RESOLVED this pass, not deferred: BC-2.07.038..043 added to E-5's BC list and the Coverage Check table (37→43 active). total_bcs 337→380, reconciled exactly against BC-INDEX v2.37's canonical derivation chain ('Total BCs on disk: 381. Active: 380.') — 0 unassigned, 0 double-assigned, 0 residual gap. Estimated Story Count Summary Total 107→136, verified against STORY-INDEX v4.19 total_stories=136. No pre-existing structural gaps (E-13/E-14/E-16 lacking full '## Epic' sections) were closed beyond E-16's Coverage/Summary reconciliation — full section authoring for those three is out of scope for a currency pass and is NOT tracked as a residual gap of this reconciliation (it predates v2.1 and predates DRIFT-EPICS-STALE-v21)."
  - "2026-09-04 v2.3: DRIFT-EPICS-NARRATIVE-SECTIONS closure (wave-86 gate, pre-delivery fix) — authored the three missing full '## Epic' narrative sections flagged (and deliberately deferred) by the v2.2 changelog entry above: Epic E-13 (Multi-Tag Finding Schema Migration, v0.3.0/issue #7 — grounded in STORY-100/STORY-101 + PRD v1.2/v1.3 f2-bundle-vs-split.md), Epic E-14 (Modbus TCP Analyzer, v0.4.0/issue #7 — grounded in STORY-102/103/104/105/141 + PRD §2.14/ADR-005/ADR-006), and Epic E-16 (ARP Security Analyzer, issue #9 — grounded in STORY-111/112/113/114/115/156 + PRD §2.16/ADR-008). Each new section was inserted at its correct ordinal position (E-13 and E-14 between E-12 and E-15; E-16 between E-15 and E-17) and follows the Goal/BCs/Subsystems-touched/Estimated-stories/Rationale structure used by every other epic section. NO numbers were changed: E-13 remains 2 stories / 0 new BCs (extensions only) / 21 pts; E-14 remains 5 stories / 25 BCs / 45 pts; E-16 remains 6 stories / 16 BCs / 50 pts — all identical to the v2.2 Estimated Story Count Summary and Coverage Check table values, which remain the source of truth for counts. total_bcs unchanged at 380."
  - "2026-09-06 v2.4: Phase F3 INTEGRATE — E-23 S7comm / ISO-on-TCP Protocol Dissection (feature-s7comm, ADR-014) added — STORY-184..194 (11 stories, 71 pts, Waves 87–97). 60 new BCs: BC-2.20.001..016 (SS-20 ISO-on-TCP framing, new subsystem, 16 BCs) + BC-2.21.001..041 (SS-21 S7comm domain analysis, new subsystem, 41 BCs) + BC-2.18.005..006 (SS-18 Support-enum catalog extension, 2 BCs) + BC-2.05.013 (SS-05 dispatcher Rule 9 extension, 1 BC). BC-2.18.003/004 are AMENDED (v1.3→v1.4, v1.2→v1.3 for port 102/S7comm entries per STORY-193), not new BC IDs — already counted in E-21's row. total_bcs 380→440, reconciled exactly against BC-INDEX v2.38.1's canonical derivation chain ('Total BCs on disk: 441. Active: 440.') — 0 unassigned, 0 double-assigned, 0 residual gap. Estimated Story Count Summary Total 136→147, verified against dependency-graph.md v3.13 total_stories=147. Per-Epic BC Assignment table: new E-23 row (SS-20/SS-21/SS-05/SS-18, 60 BCs). Arithmetic Verification: new E-23 line, running total 380→440. All 15+2=17 subsystems table: added SS-20, SS-21 rows. Coverage confirmed updated 380/380→440/440."
total_bcs: 440
traces_to:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/architecture/ARCH-INDEX.md
---

# wirerust Epic Decomposition

> **Brownfield context:** wirerust is a single-crate offline pcap forensic triage CLI.
> The original 217 behavioral contracts described the greenfield-ingested shipped
> implementation (E-1..E-10); every epic from E-12 onward is a Feature Mode F2 addition
> layered on top of that baseline. As of this document's v2.4 pass, the full
> active-BC count across all epics is 440 (BC-INDEX v2.38.1 — see Coverage Check for the
> complete per-epic derivation).
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
  BC-2.01.001, BC-2.01.002, BC-2.01.003, ~~BC-2.01.004~~ [RETIRED 2026-06-19 — superseded by BC-2.01.009; behavioral inversion: pcapng now accepted], BC-2.01.005, BC-2.01.006,
  BC-2.01.007, BC-2.01.008,
  BC-2.01.009, BC-2.01.010, BC-2.01.011, BC-2.01.012, BC-2.01.013, BC-2.01.014,
  BC-2.01.015, BC-2.01.016, BC-2.01.017, BC-2.01.018,
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
  BC-2.07.037,
  BC-2.07.038, BC-2.07.039, BC-2.07.040, BC-2.07.041, BC-2.07.042 (TLS carry-reassembly,
  fix-tls-clienthello-frag F2/F3, 2026-06-29 — TLS-CLIENTHELLO-FRAG-001),
  BC-2.07.043 (buffer_saturation_drops defense-in-depth observability counter,
  F-EV-001, fix-tls-clienthello-frag F2 scope addition)
- **Subsystems touched:** SS-07
- **Estimated stories:** 11 (+3 vs v2.0/v2.1: STORY-144/145/146, TLS-CLIENTHELLO-FRAG-001
  Parts A/B/C, fix-tls-clienthello-frag F3, 2026-06-29 — carry-buffer reassembly, ServerHello
  fragmentation symmetry, and buffer_saturation_drops telemetry)

**Rationale:** TLS analysis (43 BCs — 37 original + 6 carry-reassembly/observability BCs
added by fix-tls-clienthello-frag) covers four distinct analyst needs: handshake
parsing + JA3/JA3S computation, SNI anomaly classification (the most security-sensitive
subsection with 4 arms and boundary tests), cipher/protocol weakness detection, and
(added post-v1.0) cross-segment ClientHello/ServerHello fragmentation reassembly with
saturation telemetry. The original 37 BCs justified ~8 stories covering: ClientHello
parsing, ServerHello/JA3S, SNI 4-way classification, cipher/version findings, buffer
management, and summary; the 6 added carry-reassembly BCs justified 3 more stories
(STORY-144/145/146), bringing the epic to 11 stories / 43 BCs total.

**DRIFT-EPICS-STALE-v21 correction (v2.2, 2026-09-04):** v2.1 carried a documented -6 BC
discrepancy note (BC-2.07.038..043 present in BC-INDEX but absent from this section and
the Coverage Check table). Resolved this pass by reconciling against BC-INDEX v2.37
(380 active BCs total; 43 active for SS-07/E-5). See Coverage Check for the corrected
table row and total_bcs 337→380 reconciliation.

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
  BC-2.11.019, BC-2.11.020, BC-2.11.021, BC-2.11.022, BC-2.11.023, BC-2.11.024,
  BC-2.11.035 (mitre_attack JSON-array enrichment, issue #64, STORY-129),
  BC-2.11.036, BC-2.11.037 (snake_case JSON-enum serialization + schema_version
  envelope, issue #255, STORY-160)
- **Subsystems touched:** SS-11
- **Estimated stories:** 8 (+3 vs v2.0/v2.1: STORY-120 FindingsRender enum migration,
  STORY-129 mitre_attack JSON enrichment, STORY-160 snake_case JSON-enum + schema_version
  envelope)
- **Total points:** 40 (STORY-076: 5, STORY-077: 8, STORY-078: 8, STORY-079: 5,
  STORY-080: 3, STORY-120: 3, STORY-129: 5, STORY-160: 3)

**Rationale:** Reporting (27 BCs — 24 original + BC-2.11.035 mitre_attack enrichment +
BC-2.11.036/037 snake_case/schema_version) covers three distinct output surfaces (JSON,
terminal, CSV) plus the MITRE tactic-grouping logic for terminal output. User value
is clear: the output format is what the analyst or integrator actually sees and
consumes. Stories decompose naturally into: JsonReporter, TerminalReporter
(escaping), TerminalReporter (MITRE grouping/sort/colorization), and CsvReporter,
plus later hardening/enrichment stories (enum migration, MITRE array enrichment,
snake_case + schema_version envelope) that extend the same output-contract BCs
without changing the epic's subsystem boundary.

**DRIFT-EPICS-STALE-v21 correction (v2.2, 2026-09-04):** BC row corrected 24→27
(BC-2.11.035/036/037 were previously only referenced in a Coverage Check footnote,
not reflected in this epic's BC list or the Coverage Check table row); story count
corrected 5→8 (STORY-160 added to the Estimated Story Count Summary but never to
this section's BC/story-count fields).

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
| E-1: PCAP Ingestion and Packet Decoding | SS-01, SS-02 | BC-2.01.001..003, ~~BC-2.01.004~~ [RETIRED], BC-2.01.005..008, BC-2.01.009..018 (F2 pcapng), BC-2.02.001..015 | 32 (17 active SS-01 + 15 SS-02; BC-2.01.004 retired) |
| E-2: TCP Stream Reassembly Engine | SS-04 | BC-2.04.001..054 | 54 |
| E-3: Content-First Protocol Dispatch | SS-05 | BC-2.05.001..009 | 9 |
| E-4: HTTP Traffic Analysis and Threat Detection | SS-06 | BC-2.06.001..026 | 26 |
| E-5: TLS Traffic Analysis and Fingerprinting | SS-07 | BC-2.07.001..037, BC-2.07.038..042 (carry-reassembly, fix-tls-clienthello-frag), BC-2.07.043 (buffer_saturation_drops observability) | 43 |
| E-6: DNS Traffic Statistics | SS-08 | BC-2.08.001..004 | 4 |
| E-7: Forensic Finding Data Model and MITRE Mapping | SS-09, SS-10 | BC-2.09.001..006, BC-2.10.001..009 | 15 |
| E-8: Reporting and Output Formats | SS-11 | BC-2.11.001..024, BC-2.11.035 (mitre_attack enrichment, STORY-129), BC-2.11.036..037 (snake_case + schema_version, STORY-160) | 27 |
| E-9: CLI, Entry Point, and Analysis Orchestration | SS-12 | BC-2.12.001..021 | 21 |
| E-10: Absent Behavior Contracts (Flag Rejection) | SS-13 | BC-2.13.001..004 | 4 |
| E-12: Pcap Timestamp Provenance (issue #100) | SS-04, SS-09 | BC-2.04.055, BC-2.09.007 | 2 |
| E-13: Multi-Tag Finding Schema Migration | SS-09, SS-10, SS-11 | BC-2.09.001/006 (extensions), BC-2.10.005/007/008 (extensions), BC-2.11.001/013/015/017/020/024 (extensions) | 0 (extensions, not new BCs) |
| E-14: Modbus TCP Analyzer | SS-14 (new), SS-05, SS-12 | BC-2.14.001..025 | 25 |
| E-15: DNP3/ICS Analyzer | SS-15 (new), SS-05, SS-12 | BC-2.15.001..024 | 24 |
| E-16: ARP Security Analyzer | SS-16 (new) | BC-2.16.001..015, BC-2.16.016 (unbounded-findings-cap doc + regression, STORY-156, fix-pc-013-014-015 D-221) | 16 |
| E-17: ARP QinQ/MACsec Offset Hardening | SS-16 | BC-2.16.009 EC-008/009, BC-2.16.015 PC-7b/EC-008/009 (extensions) | 0 (extensions, not new BCs) |
| E-18: Terminal Finding-Collapse | SS-11 | BC-2.11.025..029 (flat-mode collapse, STORY-118), BC-2.11.030..034 (grouped-collapse, STORY-119) | 10 |
| E-20: EtherNet/IP (ENIP/CIP) Analyzer | SS-17 (new), SS-05, SS-12 | BC-2.17.001..026 | 26 |
| E-21: Protocol Coverage Catalog | SS-18 (new), SS-05, SS-12 | BC-2.18.001..004, BC-2.05.010..011, BC-2.12.022..024 | 9 |
| E-22: IEC-104 Passive Analyzer | SS-19 (new), SS-05, SS-10, SS-12 | BC-2.19.001..027, BC-2.19.028 (MAX_IEC104_FINDINGS DoS bound), BC-2.19.029..030 (timed control-command, wave-85), BC-2.05.012, BC-2.10.010, BC-2.12.025 | 33 |
| E-23: S7comm / ISO-on-TCP Protocol Dissection | SS-20 (new), SS-21 (new), SS-05, SS-18 | BC-2.20.001..016 (ISO-on-TCP framing), BC-2.21.001..041 (S7comm domain analysis), BC-2.05.013, BC-2.18.005..006 | 60 |
| **TOTAL** | | | **440** (see Arithmetic Verification) |

### Arithmetic Verification

```
E-1:  17 active SS-01 (8 original − 1 retired BC-2.01.004 + 10 new BC-2.01.009–018) + 15 (SS-02) = 32
E-2:  54 (SS-04)              = 54
E-3:  9 (SS-05)               =  9
E-4:  26 (SS-06)              = 26
E-5:  37 (SS-07 original) + 6 (BC-2.07.038..043 carry-reassembly/observability) = 43
E-6:  4 (SS-08)               =  4
E-7:  6 (SS-09) + 9 (SS-10)  = 15
E-8:  24 (SS-11 original) + 1 (BC-2.11.035 mitre_attack) + 2 (BC-2.11.036..037 snake_case/schema_version) = 27
E-9:  21 (SS-12)              = 21
E-10: 4 (SS-13)               =  4
E-12: 2 (BC-2.04.055, BC-2.09.007) = 2
                      --------
                      237 (pre-feature subtotal; was 228 in v2.1, +6 E-5 +3 E-8 = +9)
E-14: 25 (SS-14, BC-2.14.001..025) = 25
E-15: 24 (SS-15, BC-2.15.001..024) = 24
E-16: 15 (SS-16 original) + 1 (BC-2.16.016) = 16
                      --------
                      302 (pre-E-18 subtotal; was 292 in v2.1, +1 E-16 = 302, matches BC-INDEX 302-active milestone)
E-18: 10 (SS-11, BC-2.11.025..029 flat-collapse + BC-2.11.030..034 grouped-collapse) = 10
                      --------
                      312 (pre-E-20 subtotal)
E-20: 26 (SS-17, BC-2.17.001..026 EtherNet/IP ENIP/CIP analyzer) = 26
                      --------
                      338 (pre-E-21 subtotal)
E-21:  9 (SS-18/SS-05/SS-12: BC-2.18.001..004 + BC-2.05.010..011 + BC-2.12.022..024 protocol coverage catalog) =  9
                      --------
                      347 (pre-E-22 subtotal)
E-22: 33 (SS-19: BC-2.19.001..030 = 30 + SS-05 BC-2.05.012 + SS-10 BC-2.10.010 + SS-12 BC-2.12.025 = 33) = 33
                      --------
                      380 (pre-E-23 subtotal; matches BC-INDEX v2.37 total_bcs: 381 on disk / 380 active)
E-23: 60 (SS-20: BC-2.20.001..016 = 16 + SS-21: BC-2.21.001..041 = 41 + SS-05 BC-2.05.013 + SS-18 BC-2.18.005..006 = 2) = 60
                      --------
                      440 / 440  ✓ (matches BC-INDEX v2.38.1 total_bcs: 441 on disk / 440 active)
```

Note: E-11 (Tooling) has 0 BCs authored yet across all 23 members (tooling/process/governance
stories, not production Rust behavior — not expected to ever carry BCs). E-12 BCs are
feature-mode additions (BC-2.04.055 extends SS-04; BC-2.09.007 extends SS-09) and do not
conflict with the greenfield 217-BC assignment. E-23's BC-2.18.003/004 amendments (v1.3→v1.4,
v1.2→v1.3, port 102/S7comm entries per STORY-193) are version bumps on BCs already counted in
E-21's row, not new BC IDs — not double-counted here.

### No BC Double-Assigned

Each BC-2.NN.NNN maps to exactly one epic by construction: the epic corresponds to
the subsystem(s) identified in ARCH-INDEX.md, and subsystem assignments are
non-overlapping. No BC appears in more than one epic row above.

### All 17 Subsystems Covered (SS-14/SS-15/SS-16 pre-existing gap in this table — covered by E-14/E-15/E-16 sections)

| SS-ID | Name | Epic |
|-------|------|------|
| SS-01 | PCAP Ingestion | E-1 |
| SS-02 | Packet Decoding | E-1 |
| SS-03 | (absent — merged into SS-02 per ARCH-INDEX ruling) | E-1 |
| SS-04 | TCP Reassembly | E-2 |
| SS-05 | Protocol Dispatch | E-3, E-21, E-22, E-23 |
| SS-06 | HTTP Analysis | E-4 |
| SS-07 | TLS Analysis | E-5 |
| SS-08 | DNS Analysis | E-6 |
| SS-09 | Finding Emission | E-7 |
| SS-10 | MITRE Mapping | E-7, E-22 |
| SS-11 | Reporting | E-8 |
| SS-12 | CLI / Entry | E-9, E-21, E-22 |
| SS-13 | Absent Behaviors | E-10 |
| SS-17 | EtherNet/IP (ENIP/CIP) Analyzer | E-20 |
| SS-18 | Protocol Coverage Catalog | E-21, E-23 |
| SS-19 | IEC-104 Passive Analyzer | E-22 |
| SS-20 | ISO-on-TCP Transport Framing | E-23 |
| SS-21 | S7comm Domain Analysis | E-23 |

**Coverage confirmed: 440 / 440 active BCs assigned, 0 unassigned, 0 double-assigned**
(per BC-INDEX v2.38.1, "Total BCs on disk: 441. Active: 440."). See the Arithmetic
Verification block above for the full running-subtotal derivation.

**DRIFT-EPICS-STALE-v21 correction (v2.2, 2026-09-04):** The v2.1 Coverage Check was
stale at 337 (itself already known-short by the -6 TLS carry-BC gap it flagged). This
pass reconciles fully against BC-INDEX v2.37's canonical derivation chain (the
narrative under "Total BCs on disk" in BC-INDEX.md), which independently confirms every
subtotal above, including the +43 contributed by the newly-added E-22 epic. Residual
gap: **none** — 380/380 active BCs now accounted for exactly.

---

## Epic E-11: Tooling and Self-Improvement

- **Goal:** Build and govern mechanical tooling that catches spec-drift proactively —
  before adversarial passes surface it as findings — so the cost of each successive
  adversarial cycle falls rather than holding flat. The original deliverable was an
  anchor-validation CLI (`bin/validate-anchors`, STORY-091) verifying every
  `src|tests|fuzz/<path>.rs:NNN` citation in the spec corpus against the current
  source tree. Scope has since grown, wave over wave, into the standing home for
  every S-7.02 cycle-close process-gap codification, wave-gate governance-table
  hygiene fix, and CI/tooling hardening story that does not belong to a product
  epic — 23 stories across waves ~ (pre-wave) through 86 as of STORY-INDEX v4.19.
- **BCs:** _(none authored yet for any E-11 member — status: draft for undelivered
  members; pending PO authorship. E-11 stories are process/governance/tooling
  artifacts, not production Rust behavior, so this epic is not expected to ever
  carry BC-S.SS.NNN entries.)_
- **Subsystems touched:** none (tooling-only; no production Rust subsystem)
- **Estimated stories:** 23
- **Total points:** 75
- **Dispositions:** PROCESS-GAP-P5-001 (S-7.02 cycle-close requirement)

**Story roster (ground truth: STORY-INDEX v4.19 "Stories by Epic" + Index Table):**

| Story | Wave | Points | Status | Note |
|-------|------|--------|--------|------|
| STORY-091 | ~ | 5 | superseded | Anchor-validation tooling (`bin/validate-anchors`) — disposition OBSOLETE/delivered-by-drift via `bin/validate-citations` (STORY-164) + STORY-166 symbol-at-line assertion; residual `--scan` discovery layer represented upstream (drbothen/vsdd-factory#622/#603/#396 family); human-approved no-filing 2026-07-19 (E-11 upstream re-scope burst #2) |
| STORY-121 | ~ | 3 | superseded | F1/F2 story-input analysis docs — mandatory numeric self-audit + consuming-surface sweep checklist; routed upstream (drbothen/vsdd-factory#582 evidence comment, x-ref #396) |
| STORY-143 | ~ | 3 | superseded | Harden release-changelog step (full prev-tag..HEAD range enumeration); routed upstream (drbothen/vsdd-factory#695 new issue, x-ref #580) |
| STORY-147 | 84 | 2 | delivered | Repo-local mutation-testing defaults: `.cargo/mutants.toml` timeout floor + CLAUDE.md guidance (PG-MUTANTS-JOBS-001, fix-tls-clienthello-frag F6); re-scoped v1.0→v2.0 SPLIT survivor (product half retained; engine half routed drbothen/vsdd-factory#654); pts 3→2 |
| STORY-149 | 70 | 5 | merged | TLS carry-path performance recovery + fragmented-handshake benchmark fixture (PERF-001/002, maint-2026-07-01) |
| STORY-150 | 71 | 5 | merged | TLS drain-loop DRY refactor (TLS-DRAIN-DUP-001) with mandatory Kani VP-039 + mutation re-run |
| STORY-155 | ~ | 3 | superseded | Auto-update STORY-INDEX status draft→merged on story PR merge (PG-INDEX-DRIFT-001); routed upstream (drbothen/vsdd-factory#290 evidence comment, x-ref #600) |
| STORY-157 | 71 | 5 | merged | Wave-70 process-gap codifications: adversary attestation preamble, demo-evidence scrub gate, input-hash empty-inputs handling, merge-authorization procedure (PG-S149-001 + PG-W70-DEMO-SCRUB + PG-HASH-EMPTY-INPUTS + PG-W70-MERGE-AUTH); amended 3→5 pts |
| STORY-158 | 72 | 3 | delivered | Wave-71 process-gap codifications: changelog gate, cycle-artifact identity lint, CI scan-guard hardening (PG-W71-CHANGELOG + PG-W71-CYCLE-ARTIFACT-IDENTITY + PG-W71-CI-SCAN-GUARDS) |
| STORY-159 | 72 | 3 | delivered | Author public ADR-012: Protocols Catalog and Coverage-Gaps System (maint-2026-07-08 NEW-001) |
| STORY-161 | 72 | 3 | delivered | Codify multi-file `proof_file_hash` mini-Merkle algorithm + re-lock VP-024 (triage-2026-07-08 #252) |
| STORY-162 | 73 | 3 | completed | Wave-72 S-7.02 cycle-close: LMR-003 template-conformance exemption (F-S161P1-001) + check-green-doc-tense `main()` guard self-tests (F-W72G-P2-OBS-001) |
| STORY-163 | 73 | 2 | delivered | maint-2026-07-09 S-7.02 cycle-close: docs-dispatch citation mandate (PG-RA-P3-ARP-REC006-INVERSION-001) + subagent merge-halt resolution path (PG-MERGE-AUTH-SUBAGENT-CLASSIFIER) |
| STORY-164 | 74 | 4 | delivered | Wave-73 S-7.02 cycle-close: STORY-INDEX status-vocabulary legend + `bin/validate-citations` preflight validator + changelog-gate content assertion + CLAUDE.md guidance row + BREAKING-change holdout-sweep obligation (amended AC-164-005, pts 3→4) |
| STORY-165 | 75 | 3 | delivered | Wave-74 S-7.02 cycle-close: bin-selftest CI wiring + PR-description row-verify mandate + delivery-doc currency sweep + governance-table audit-first rule |
| STORY-166 | 84 | 3 | delivered | Wave-75 S-7.02 cycle-close: citation symbol-at-line assertion + demo-evidence scrub scope extension (project half); re-scoped v1.1 (2026-07-13) — engine halves routed upstream (#638/#635), PG-HASH-HOOK-DIVERGENCE tracked as #637; pts 5→3 |
| STORY-175 | ~ | 2 | superseded | Feature-IEC104 cycle-close: demo evidence JSON accuracy protocol; routed upstream 2026-07-19 (drbothen/vsdd-factory#494, confirmed duplicate) |
| STORY-176 | 84 | 2 | delivered | Feature-IEC104 cycle-close: local gate + tooling hygiene sweeps; re-scoped v2.0 local-gate + tooling-hygiene survivor (absorbed STORY-178 AC-003/AC-004; engine ACs routed #682/#686); pts 3→2 |
| STORY-177 | ~ | 2 | superseded | Feature-IEC104 cycle-close: agent dispatch and reporting discipline; routed upstream (#461, x-ref #457/#637 confirmed duplicates) |
| STORY-178 | ~ | 3 | superseded | Feature-IEC104 cycle-close: pre-delivery spec fidelity gate; routed upstream (#655/#305); AC-003/AC-004 survive locally via STORY-176 |
| STORY-179 | ~ | 2 | superseded | Feature-IEC104 cycle-close: session recovery and multi-worktree verification; routed upstream (#396) |
| STORY-182 | 86 | 4 | draft | E2E fixture manifest + committed representative ITI captures — eliminate false-green `cargo test` in clean worktrees (PG-W85-005); wave-86 human story-approval gate D-544 |
| STORY-183 | 86 | 5 | draft | `check-green-doc-tense`: `bin/*.py` prose coverage + TIER-1 behavioral-absence token coverage (PG-W84-010 + PG-W85-003 combined per DF-VALIDATION coupling ruling); wave-86 human story-approval gate D-544 |

**Rationale:** Phase-5 adversarial refinement repeatedly surfaced source-line-anchor
drift across four dimensions (BC source anchors, BC secondary anchors, consuming
VP/invariant/supplement/entity docs, story bodies) — 83 stale citations corrected
in one pass alone. Root cause: every sweep was reactive (triggered by an adversarial
finding) rather than preventive. PROCESS-GAP-P5-001 requires a durable-fix
disposition at cycle close (S-7.02). A dedicated tooling epic separates this
self-improvement work from product epics and makes future tooling stories easy to
group here. In practice this made E-11 the durable landing zone for every wave's
S-7.02 cycle-close codification and every research-validated upstream-vs-local
disposition (STORY-091/121/143/155/175/177/178/179 superseded and routed upstream
per DF-VALIDATION-001; STORY-147/166/176 re-scoped SPLIT/local-gate survivors) —
none of this changes the epic's tooling-only subsystem classification.

**DRIFT-EPICS-STALE-v21 correction (v2.2, 2026-09-04):** This section was frozen at
v2.1 (2026-07-02) listing only 6 members (STORY-091/121/143/147/149/150). 17 stories
added across waves 71-86 (STORY-155, 157-159, 161-166, 175-179, 182, 183) were never
reflected here even though several already appeared in the STORY-INDEX epic table.
Reconciled against STORY-INDEX v4.19: 23 stories / 75 points, matching the
"E-11 | ... | 23 | 75" row exactly. STORY-182 and STORY-183 (wave-86 human
story-approval gate D-544, "fix both now") are now correctly listed as E-11 members.

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

## Epic E-13: Multi-Tag Finding Schema Migration (v0.3.0 / issue #7)

- **Goal:** A SIEM integrator or SOC analyst consuming wirerust JSON/CSV/terminal output sees
  every `Finding` carry `mitre_techniques: Vec<String>` in place of the old
  `mitre_technique: Option<String>` scalar — so that a single finding can be co-attributed to
  multiple MITRE ATT&CK techniques (e.g. `["T1692.001","T0836"]` for a Modbus write command),
  while every pre-existing single-technique finding continues to serialize correctly as a
  singleton vec. The JSON report envelope gains `mitre_domain`/`mitre_attack_version` fields,
  the CSV reporter semicolon-joins multi-technique cells (empty string for no techniques), and
  the terminal reporter groups findings by `mitre_techniques[0]` tactic. The MITRE technique
  catalog is seeded with 6 new ICS technique IDs (15 → 21 total) so the type-system change and
  the catalog have no unresolved IDs on day one.
- **BCs:** BC-2.09.001 (extension), BC-2.09.006 (extension), BC-2.10.005 (extension),
  BC-2.10.007 (extension), BC-2.10.008 (extension), BC-2.11.001 (extension),
  BC-2.11.013 (extension), BC-2.11.015 (extension), BC-2.11.017 (extension),
  BC-2.11.020 (extension), BC-2.11.024 (extension) — 0 new BCs (all are amended versions of
  pre-existing E-7/E-8 BCs; see Coverage Check for the authoritative 0-count).
- **Subsystems touched:** SS-09, SS-10, SS-11
- **Estimated stories:** 2 (STORY-100, STORY-101)
- **Total points:** 21 (STORY-100: 13, STORY-101: 8)
- **Feature issue:** #7 (schema half of the Feature #7 Modbus decomposition)

**Rationale:** Per PRD v1.2/v1.3 (`f2-bundle-vs-split.md` Decision B2), the multi-tag Finding
schema change is independent of the Modbus analyzer that motivates it and is deliberately
released first as its own breaking-change version (v0.3.0), with Modbus itself landing
purely additively in v0.4.0 (E-14) on top of the stabilized contract. The migration
decomposes into exactly two stories along the pure-model/reporter-surface boundary:
STORY-100 performs the atomic field rename (`mitre_technique` → `mitre_techniques`) across
all ~21 emission sites plus the MITRE catalog seed expansion (15 → 21 IDs), and STORY-101
updates the three reporter surfaces (JSON envelope fields, CSV semicolon-join encoding,
terminal MITRE-tactic grouping) to consume the new `Vec<String>` field. STORY-101 depends on
STORY-100 because it operates on the type STORY-100 introduces. No new subsystem is touched;
all 11 BC references are version-bumped amendments of existing SS-09/SS-10/SS-11 contracts,
which is why this epic contributes 0 new BCs to the total_bcs count despite unblocking every
downstream multi-technique detection (Modbus, DNP3, ARP, ENIP, IEC-104).

**Currency note (v2.3, 2026-09-04):** This section was missing from epics.md prior to this
pass (DRIFT-EPICS-NARRATIVE-SECTIONS) — the epic's story/BC counts were previously visible
only via the Estimated Story Count Summary and Coverage Check tables. Narrative added; no
counts changed (2 stories / 0 new BCs / 21 points, unchanged from v2.2).

---

## Epic E-14: Modbus TCP Analyzer (v0.4.0 / issue #7)

- **Goal:** An ICS/OT security analyst can point wirerust at a pcap containing Modbus TCP
  traffic (port 502, built on the multi-tag `Finding` contract established in E-13) and
  receive structured, multi-tag findings for all 7 MITRE ATT&CK for ICS detection patterns
  the analyzer supports: write-class command co-emission (T1692.001 + applicable technique
  tags), coordinated-write sequences (T0831), dual-window (burst/sustained) write-rate
  anomalies (T0806), diagnostics-function DoS bursts (T0814), exception-response bursts, and
  reconnaissance reads — with a bounded per-flow pending-transaction table correlating
  requests to responses/exceptions, a MAX_FINDINGS cap with poison-skip behavior, a
  `--modbus` CLI flag (`--all` includes it, default off, requires stream reassembly), and
  tunable `--modbus-write-burst-threshold`/`--modbus-write-sustained-threshold` flags —
  formally verified panic-free by Kani proofs over the pure-core MBAP/FC parsing layer.
- **BCs:**
  BC-2.14.001, BC-2.14.002, BC-2.14.003, BC-2.14.004, BC-2.14.005, BC-2.14.006,
  BC-2.14.007, BC-2.14.008, BC-2.14.009, BC-2.14.010, BC-2.14.011, BC-2.14.012,
  BC-2.14.013, BC-2.14.014, BC-2.14.015, BC-2.14.016, BC-2.14.017, BC-2.14.018,
  BC-2.14.019, BC-2.14.020, BC-2.14.021, BC-2.14.022, BC-2.14.023, BC-2.14.024,
  BC-2.14.025
- **Subsystems touched:** SS-14 (new), SS-05, SS-12
- **Estimated stories:** 5 (STORY-102, STORY-103, STORY-104, STORY-105, STORY-141)
- **Total points:** 45 (STORY-102: 8, STORY-103: 8, STORY-104: 13, STORY-105: 8, STORY-141: 8)
- **Feature issue:** #7 (Modbus half of the Feature #7 decomposition; ADR-005/ADR-006)

**Rationale:** Modbus/ICS analysis (25 BCs) decomposes into four strictly linear layers,
matching the pattern later reused for DNP3 (E-15) and ENIP (E-20): (1) pure-core MBAP header
parse + all-256-function-code classification, Kani-verified panic-free (STORY-102, wave 32);
(2) per-flow state — a bounded pending-transaction table keyed on (Transaction ID, Unit ID)
correlating requests to responses/exceptions (STORY-103, wave 33, dep=102); (3) the seven
MITRE detection rules plus `summarize()` and the MAX_FINDINGS cap (STORY-104, wave 33,
dep=103); (4) `StreamDispatcher` Rule 5 integration and the `--modbus`/threshold CLI flags
(STORY-105, wave 34, dep=104). STORY-141 (wave 64) is a later release-blocking hardening fix
— RULING-MODBUS-SIBLING-001 (DRIFT-MODBUS-DIRECTION-001 / DRIFT-MODBUS-CLOCK-001) — that
splits the carry buffer per TCP direction (`carry_c2s`/`carry_s2c`) and switches window-expiry
arithmetic to `saturating_sub`, so bidirectional flows and adversarially injected
backwards-clock timestamps cannot produce phantom findings or suppress in-progress burst
windows. It is grouped in this epic rather than a separate one because it touches no new
subsystem boundary (SS-14 only). The analyzer is purely additive on top of the v0.3.0
multi-tag schema (E-13): no BC outside SS-14/SS-05/SS-12 changes.

**Currency note (v2.3, 2026-09-04):** This section was missing from epics.md prior to this
pass (DRIFT-EPICS-NARRATIVE-SECTIONS) — the epic's story/BC counts were previously visible
only via the Estimated Story Count Summary and Coverage Check tables. Narrative added; no
counts changed (5 stories / 25 BCs / 45 points, unchanged from v2.2).

---

## Epic E-15: DNP3/ICS Analyzer (issue #8)

- **Goal:** A forensic analyst or ICS/OT security engineer can point wirerust at a pcap
  containing DNP3 traffic (TCP port 20000, IEEE 1815-2012) and receive structured findings
  for unauthorized control commands (T1692.001), restart/stop commands (T0814), write-
  register commands (T0836), block-control inference (T1691.001), process impact (T0827),
  and anomaly conditions (broadcast, unsolicited, malformed frames) — with per-flow state
  tracking, a 292-byte carry buffer per direction for segment-spanning frame reassembly
  (carry split per RULING-DNP3-SIBLING-001: `carry_c2s`/`carry_s2c`), backwards-clock-safe
  window expiry arithmetic (`saturating_sub`), and a tunable `--dnp3-direct-operate-threshold`
  CLI flag.
- **BCs:**
  BC-2.15.001, BC-2.15.002, BC-2.15.003, BC-2.15.004, BC-2.15.005, BC-2.15.006,
  BC-2.15.007, BC-2.15.008, BC-2.15.009, BC-2.15.010, BC-2.15.011, BC-2.15.012,
  BC-2.15.013, BC-2.15.014, BC-2.15.015, BC-2.15.016, BC-2.15.017, BC-2.15.018,
  BC-2.15.019, BC-2.15.020, BC-2.15.021, BC-2.15.022, BC-2.15.023, BC-2.15.024
- **Subsystems touched:** SS-15 (new DNP3 analyzer), SS-05 (dispatcher Rule 6), SS-12 (CLI threshold flag)
- **Estimated stories:** 7 (STORY-106..110, STORY-140, STORY-142)
- **Feature issue:** #8
- **STORY-140 (wave 63):** RULING-DNP3-SIBLING-001 detection-correctness fixes — per-direction carry split (`carry_c2s`/`carry_s2c`), `on_data` direction threading, `saturating_sub` window expiry (8 sites: 60s/10s/300s), 300s operator pin (`>= CORRELATION_WINDOW_SECS` → `> CORRELATION_WINDOW_SECS`), `resolve_master_ip` direction fix-along. BCs: BC-2.15.016 v2.0 + BC-2.15.010 v1.8 + BC-2.15.014 v2.1 + BC-2.15.015 v2.0. VPs: VP-035 + VP-036. Release blocker per RULING-DNP3-SIBLING-001 (2026-06-27).
- **STORY-142 (wave 64):** RULING-DNP3-DESYNC-001 desync-latch direction-contamination fix — one-line predicate change at `dnp3.rs:363`: `active_carry!(flow, direction).is_empty()` → `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`. Prevents junk s2c packet from permanently silencing an established c2s DNP3 stream. BC: BC-2.15.009 v2.0. No new VPs (targeted regression test). Dep=STORY-140. Release blocker per RULING-DNP3-DESYNC-001 (2026-06-28).

**Rationale:** DNP3 analysis (24 BCs, IEEE 1815-2012 binary protocol) decomposes into
five natural layers matching the ADR-007 design decisions: (1) pure-core parse + FC
classification (Kani-verifiable, VP-023 anchor), (2) per-flow state + carry buffer +
memory safety bounds, (3) direct detection emissions (T1692.001, T0814 restart, T0836),
(4) correlated/derived + anomaly detections (T1691.001, T0827, broadcast, malformed —
VP-007 atomic-update anchor), (5) dispatcher integration + CLI flag (VP-004 oracle
obligation). Each layer is independently testable; the dependency chain is strictly linear
with no parallelism (each story builds on the previous one's produced types and state).

---

## Epic E-16: ARP Security Analyzer (issue #9)

- **Goal:** A forensic analyst or ICS/OT security engineer can enable `--arp` against a pcap
  containing Ethernet/IPv4 ARP traffic and receive findings across 5 detection types: ARP
  spoofing via IP→MAC rebind (D1, MEDIUM then HIGH, MITRE T0830/T1557.002), Gratuitous ARP
  (D2, LOW normally / MEDIUM when it conflicts with an existing binding and co-emits a D1
  finding — "GARP-that-conflicts"), ARP packet storms via per-source-MAC rate tracking over a
  60-second flap window (D3), malformed non-Ethernet/IPv4 ARP frames (D11), and Ethernet/ARP
  sender-MAC L2/L3 mismatches (D12) — backed by a bounded (LRU-evicted) IP→MAC binding table,
  a `summarize()` surface with 13 canonical keys, and tunable `--arp-spoof-threshold` /
  `--arp-storm-rate` CLI flags. ARP analysis is link-layer only (bypasses the stream
  dispatcher, does not require reassembly) and is opt-in — `--all` does not enable it.
- **BCs:** BC-2.16.001, BC-2.16.002, BC-2.16.003, BC-2.16.004, BC-2.16.005, BC-2.16.006,
  BC-2.16.007, BC-2.16.008, BC-2.16.009, BC-2.16.010, BC-2.16.011, BC-2.16.012, BC-2.16.013,
  BC-2.16.014, BC-2.16.015, BC-2.16.016 (unbounded-findings-cap doc + regression, STORY-156,
  fix-pc-013-014-015 D-221)
- **Subsystems touched:** SS-16 (new)
- **Estimated stories:** 6 (STORY-111, STORY-112, STORY-113, STORY-114, STORY-115, STORY-156)
- **Total points:** 50 (STORY-111: 5, STORY-112: 8, STORY-113: 13, STORY-114: 13,
  STORY-115: 8, STORY-156: 3)
- **Feature issue:** #9 (ADR-008)
- **Status:** DELIVERED/CLOSED — STORY-111..115 shipped v0.7.0; twice research-validated as
  superseded / DELIVERED-BY-DRIFT (D-487, 2026-07-21; DF-VALIDATION-001;
  `planning/e16-e17-arp-draft-disposition-plan.md`) because the code landed on `develop`
  ahead of formal story-PR closure. STORY-156 (wave 71) shipped separately as `merged`.

**Rationale:** ARP security analysis (16 BCs) decomposes into five layers that mirror the
decode-vs-analysis separation mandated by ADR-008 Decisions 1/3: (1) decode-pipeline
scaffolding — upgrading etherparse 0.16→0.20, introducing the `DecodedFrame`/`ArpFrame`
types, and adding symmetric `unreachable!` compile-safety arms for the new three-way dispatch,
with a non-panicking `extract_arp_frame` placeholder (STORY-111, depends on the DNP3 chain's
STORY-110 purely for `develop` sequencing, not an ARP-domain dependency); (2) the real
`extract_arp_frame` implementation wired into both the strict and lax decode paths plus an
`ArpAnalyzer` stub (STORY-112, dep=111); (3) the full stateless/stateful detection surface —
binding table, D2 GARP, D11 malformed, D12 mismatch, `summarize()`, and the `--arp` flag
(STORY-113, dep=112); (4) D1 spoof escalation, GARP-that-conflicts, and the co-committed
5-part MITRE catalog update for T0830/T1557.002 (STORY-114, dep=113); (5) D3 storm detection
and the `--arp-storm-rate` flag, completing the v0.7.0 release scope (STORY-115, dep=114).
This strictly linear chain reflects that decode-layer scaffolding must land before any
ARP-specific analysis can be written, and each detection layer builds on state (the binding
table, the MITRE catalog) introduced by its predecessor. STORY-156 (wave 71) is a later
maintenance story documenting and regression-testing the analyzer's deliberately unbounded
findings output (`process_arp` carries no MAX_FINDINGS cap, unlike every other analyzer) —
grouped here because it touches no new subsystem boundary (SS-16 only) and closes a
spec-coherence gap (BC-2.16.016, F-NEW-MAJ-003) rather than adding new detection behavior.

**Currency note (v2.3, 2026-09-04):** This section was missing from epics.md prior to this
pass (DRIFT-EPICS-NARRATIVE-SECTIONS) — the epic's story/BC counts were previously visible
only via the Estimated Story Count Summary and Coverage Check tables (the v2.2 changelog
entry explicitly flagged this three-epic gap and deferred full-section authoring for E-16).
Narrative added; no counts changed (6 stories / 16 BCs / 50 points, unchanged from v2.2).

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
  BC-2.11.025, BC-2.11.026, BC-2.11.027, BC-2.11.028, BC-2.11.029 (flat-mode collapse — STORY-118);
  BC-2.11.030, BC-2.11.031, BC-2.11.032, BC-2.11.033, BC-2.11.034 (grouped-collapse — STORY-119);
  BC-2.11.010 v1.8, BC-2.11.013 v1.11, BC-2.11.017 v1.13, BC-2.11.019 v1.6 (extended)
- **Subsystems touched:** SS-11 (reporter/terminal.rs), SS-12 (cli.rs, main.rs — thin wiring)
- **Estimated stories:** 3 (STORY-118 delivered Wave 47; STORY-122 delivered Wave 49;
  STORY-119 delivered Wave 50)
- **Total points:** 16 (STORY-118: 8, STORY-122: 3, STORY-119: 5)

**Rationale:** The collapse feature is a pure display-layer transform confined to
`src/reporter/terminal.rs`. It shares no subsystem boundary with JSON/CSV reporters
(BC-2.11.029 invariant 1). The `--no-collapse` CLI flag follows the established
subcommand-scoped boolean pattern (`--mitre`, `--dns`), making it a thin wiring addition
to SS-12. The scope is narrow enough for a single story (STORY-118, 8 points). STORY-122
(FindingsRender enum→struct reshape, D-120 split-A) is a byte-identical construction-site
migration story that sits between STORY-118 and STORY-119 in the dependency chain —
no new BCs. STORY-119 (grouped-mode collapse, `--mitre` render path + CLI flip) depends
on STORY-122's reshaped type.

**Currency note (v2.2, 2026-09-04):** This section's story list was stale — it named
only STORY-118/119 and still marked STORY-119 "deferred" and the epic "Estimated
stories: 2," even though the Estimated Story Count Summary table already carried
STORY-122 (added v1.7) and all three E-18 stories show `completed` in STORY-INDEX
v4.19. Corrected to 3 stories / 16 points, matching the STORY-INDEX "E-18 | ... | 3 | 16" row.

---

## Epic E-19: pcapng Capture-Format Reader Support (FE-001)

- **Goal:** A forensic analyst can point wirerust at a pcapng file (Section Header Block
  + Interface Description Block + Enhanced Packet Block / Simple Packet Block) and have
  every captured packet decoded and analyzed, with correct 64-bit timestamp normalization,
  interface-whitelist validation, structured error surfaces for malformed blocks, and
  per-file error isolation so one corrupt pcapng in a batch does not abort the entire
  analysis run. wirerust accepts pcapng files wherever pcap files are accepted; format
  detection is content-based (magic-byte probe), not extension-based.
- **BCs:**
  BC-2.01.009, BC-2.01.010, BC-2.01.011, BC-2.01.012, BC-2.01.013, BC-2.01.014,
  BC-2.01.015, BC-2.01.016, BC-2.01.017, BC-2.01.018,
  BC-2.12.011
  _(Note: these BCs are pre-existing — added to the E-1 and E-9 BC lists in v1.5/v1.6;
  no new BCs are introduced by E-19. The stories assign implementation ownership to the
  specific BCs without changing the epic-level BC-count totals.)_
- **Subsystems touched:** SS-01 (reader.rs — magic-byte probe, SHB/IDB/EPB/SPB parsers),
  SS-12 (main.rs — resolve_targets content detection, per-file isolation loop)
- **Estimated stories:** 6 (STORY-123..128)
- **Feature ID:** FE-001
- **Total points:** 37 (STORY-123: 5, STORY-124: 8, STORY-125: 8, STORY-126: 8, STORY-127: 5, STORY-128: 3)
- **Waves:** 51–56
- **Status:** delivered/complete (6/6 MERGED, D-184; STORY-INDEX v4.19 shows all 6 `completed`)

**Rationale:** pcapng is the modern successor to the legacy pcap format and is the default
output of Wireshark, tcpdump ≥4.9.3, and most hardware capture appliances. Analysts
increasingly encounter pcapng files; wirerust's current E-INP-004 rejection means these
files are silently unanalyzed. The feature spans two subsystems (SS-01 reader + SS-12
entry) and decomposes into 6 stories following the natural block-type layering of the
pcapng spec (RFC 8126 / draft-tuexen-opsawg-pcapng): SHB (root) → IDB (interface table)
→ EPB (most common packet block) ∥ SPB (compact block) → E2E corpus wiring → per-file
isolation. Each story is independently testable with a stub predecessor.

---

## Epic E-20: EtherNet/IP (ENIP/CIP) Analyzer (issue #316, feature-enip-v0.11.0)

- **Goal:** A forensic analyst or ICS/OT security engineer can point wirerust at a pcap
  containing EtherNet/IP traffic (TCP port 44818, ODVA EtherNet/IP specification) and
  receive structured findings for: ENIP ListIdentity reconnaissance (T0846 Remote System
  Discovery), CIP Identity Object attribute reads (T0888 Remote System Information
  Discovery), CIP error-response bursts (T0888 Pattern B), operating mode change commands
  (T0858 Change Operating Mode), device reset commands (T0816 Device Restart/Shutdown),
  write-attribute bursts (T0836 Modify Parameter Settings), connection lifecycle events
  (ForwardOpen/ForwardClose), carry-buffer robustness against partial frames, non-ENIP
  traffic quarantine on port 44818, and T0814 DoS burst detection — with session state
  tracking (RegisterSession/UnRegisterSession), per-flow statistics, and the MAX_FINDINGS
  DoS guard enforced at finalize() time.
- **BCs:**
  BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004,
  BC-2.17.005, BC-2.17.006, BC-2.17.007, BC-2.17.008, BC-2.17.009,
  BC-2.17.010, BC-2.17.011, BC-2.17.012, BC-2.17.013, BC-2.17.014,
  BC-2.17.015, BC-2.17.016, BC-2.17.017, BC-2.17.018,
  BC-2.17.019, BC-2.17.020, BC-2.17.021, BC-2.17.022, BC-2.17.023,
  BC-2.17.024, BC-2.17.025, BC-2.17.026
- **Subsystems touched:** SS-17 (new EtherNet/IP analyzer), SS-05 (dispatcher Rule 7), SS-12 (CLI flags)
- **Estimated stories:** 12 (STORY-130..139, STORY-148, STORY-181)
- **Feature issue:** #316
- **Feature ID:** feature-enip-v0.11.0
- **Release target:** v0.11.0
- **Total points:** 82 (STORY-130: 8, STORY-131: 8, STORY-132: 8, STORY-133: 5, STORY-134: 8, STORY-135: 8, STORY-136: 5, STORY-137: 8, STORY-138: 8, STORY-139: 8, STORY-148: 5, STORY-181: 3)
- **Waves:** 58–62, 85
- **STORY-139 (wave 62):** EC-X1/EC-X2 detection-correctness fixes — per-direction carry split (`carry_c2s`/`carry_s2c`), `on_data` direction threading, `saturating_sub` window expiry (3 windows), T0814 operator pin (`>= 300` → `> 300`), DRIFT-ENIP-DIRECTION-001 fix-along. BCs: BC-2.17.016 v2.0 + BC-2.17.008 v1.3 + BC-2.17.012 v1.2 + BC-2.17.018 v1.1. VPs: VP-033 + VP-034. Release blocker per RULING-EDGECASE-001 (2026-06-27).
- **STORY-148 (wave ~, superseded):** Fix analyzer flow-state lifecycle — EnipAnalyzer `on_flow_close` wiring + DNP3 flow-map cap (SEC-005/SEC-006, maint-2026-07-01); 5 pts; superseded by PR #362 (D-383, issue #342 closed 2026-07-06). Per D-477/D-480 convention, supersession alone does not remove the story's points from the epic total.
- **STORY-181 (wave 85):** Fix SEC-001 ENIP unsafe split-borrow in `on_data` — eliminate `*mut EnipFlowState` raw pointer in PDU dispatch loop (behavior-preserving refactor) + ROUTE-W74 OBS-1; 3 pts; delivered (D-509, PR #438).

**Rationale:** EtherNet/IP (IEEE 802.3 + ODVA) analysis decomposes into a natural
diamond topology: (1) pure-core ENIP header parse + Kani VP-032 safety proof (STORY-130);
(2) StreamDispatcher Rule 7 + CLI flags (STORY-131); both roots are independent.
Wave 59: (3) CPF item walk + CIP header parse + path extraction (STORY-132, dep=130);
(4) MITRE ICS technique seeding + VP-007 atomic burst (STORY-133, dep=131; ADR-010
Decision 7). Wave 60: four parallel detection stories (recon, command, lifecycle,
robustness) all depend on STORY-132+133 — they share CPF/CIP parsing infrastructure
but emit findings for independent attack patterns. Wave 61: (9) session lifecycle +
statistics + MAX_FINDINGS guard + summarize() (STORY-138, dep=all four Wave-60 stories).
STORY-148 and STORY-181 are later maintenance/security-hardening stories against the
same SS-17 analyzer surface, grouped here rather than in a separate epic because they
touch no new subsystem boundary.

**DRIFT-EPICS-STALE-v21 correction (v2.2, 2026-09-04):** Story count corrected 10→12
and points corrected 74→82 to add STORY-148 (already noted in the Estimated Story
Count Summary since v1.8 but never added to this section) and STORY-181 (new, wave 85,
delivered under D-509), matching the STORY-INDEX "E-20 | ... | 12 | 82" row.
The diamond topology enables 4-way parallelism in Wave 60, reducing total delivery time
vs. a linear chain by 3 waves.

---

## Epic E-21: Protocol Coverage Catalog (feature-protocol-coverage)

- **Goal:** A forensic analyst or security engineer can run `wirerust protocols` to see a
  static catalog of all protocols wirerust can analyze (name, port(s), transport, coverage
  tier) in terminal table or JSON output; and can run `wirerust analyze --coverage-gaps`
  against any pcap to receive a per-port summary of TCP and UDP traffic that wirerust did
  not classify into a known protocol — so that analysts know exactly what wirerust covers
  and where unclassified-traffic gaps exist in any capture.
- **BCs:**
  BC-2.18.001, BC-2.18.002, BC-2.18.003, BC-2.18.004
  (SS-18 protocol coverage catalog + `protocols` subcommand terminal/JSON output),
  BC-2.05.010, BC-2.05.011
  (SS-05 dispatcher `unclassified_port_counts` + UDP decode-loop `udp_unclassified_counts`),
  BC-2.12.022, BC-2.12.023, BC-2.12.024
  (SS-12 `protocols` subcommand dispatch + `--coverage-gaps` flag + `CoverageGapsSummary`)
- **Subsystems touched:** SS-18 (new — protocol coverage catalog, `src/protocols.rs`),
  SS-05 (dispatcher — `unclassified_port_counts` map + UDP gap tracking),
  SS-12 (CLI — `protocols` subcommand + `--coverage-gaps` analyze flag)
- **Estimated stories:** 4 (STORY-151, STORY-152, STORY-153, STORY-154)
- **Feature ID:** feature-protocol-coverage
- **Total points:** 32 (STORY-151: 8, STORY-152: 8, STORY-153: 8, STORY-154: 8)
- **Waves:** 67–69
- **Status:** delivered (4/4 `merged` per STORY-INDEX v4.19)

**Rationale:** Protocol coverage visibility was the last major gap in wirerust's analyst
UX: analysts could not tell which protocols wirerust knows without reading source code, and
had no way to see which traffic in a pcap went unanalyzed. The feature decomposes naturally
into four layers following the pure-core / effectful boundary: (1) pure-core static catalog
(`src/protocols.rs` KNOWN_PROTOCOLS, KnownProtocol struct, SUPPORTED_PORTS set, pure-core
partition functions, VP-041 proptest harnesses — STORY-151, wave 67, dep=none);
(2) dynamic per-flow unclassified-port gap counters in the dispatcher + UDP decode-loop
(BC-2.05.010/011, VP-042/VP-043 — STORY-153, wave 67, dep=none, parallel with STORY-151);
(3) `protocols` CLI subcommand + terminal table renderer + JSON output
(BC-2.12.022/BC-2.18.001/002, dep=STORY-151 — STORY-152, wave 68);
(4) `--coverage-gaps` opt-in flag + `CoverageGapsSummary` tri-state report + L2 caveat
annotation + port-102 note (BC-2.12.023/024, dep=STORY-151+STORY-152+STORY-153 — STORY-154,
wave 69; file-sequencing edge 152→154 enforced per F-F3P2-005 because STORY-152 and
STORY-154 both modify `src/cli.rs`, `src/main.rs`, and `tests/integration_tests.rs` —
parallel dispatch would cause merge conflicts). STORY-151 and STORY-153 are independent
(wave 67 parallel). The four-story linear-with-fork topology enables correct ordering
without unnecessary serialization.

---

## Epic E-22: IEC-104 Passive Analyzer (feature-iec104)

- **Goal:** A forensic analyst or ICS/OT security engineer can point wirerust at a pcap
  containing IEC 60870-5-104 traffic (TCP port 2404) and receive structured findings for
  unauthorized/timed control commands (T1692.001, TypeIDs 45–51 and 58–64), restart/stop
  session-control commands, reserved-TypeID and interrogation anomalies, and T0881
  Impair Process Control technique attribution — with per-flow APCI/ASDU parsing,
  N(S)/N(R) sequence-tracking desync detection, per-direction carry buffers for
  segment-spanning frame reassembly, a MAX_IEC104_FINDINGS DoS bound, and an opt-in
  `--iec104` CLI flag — mirroring the passive-analysis pattern already established for
  Modbus (E-14) and DNP3 (E-15).
- **BCs:**
  BC-2.19.001, BC-2.19.002, BC-2.19.003, BC-2.19.004, BC-2.19.005, BC-2.19.006,
  BC-2.19.007, BC-2.19.008, BC-2.19.009, BC-2.19.010, BC-2.19.011, BC-2.19.012,
  BC-2.19.013, BC-2.19.014, BC-2.19.015, BC-2.19.016, BC-2.19.017, BC-2.19.018,
  BC-2.19.019, BC-2.19.020, BC-2.19.021, BC-2.19.022, BC-2.19.023, BC-2.19.024,
  BC-2.19.025, BC-2.19.026, BC-2.19.027 (SS-19 IEC-104 passive analyzer core, 27 BCs),
  BC-2.19.028 (SS-19 MAX_IEC104_FINDINGS DoS bound, SR-173-02 blocking addition),
  BC-2.19.029, BC-2.19.030 (SS-19 timed control-command detection TypeIDs 58–64,
  wave-85-spec-evolution, IEC104-TIMED-CMD-GAP-001 closure) — 30 BCs total for SS-19;
  BC-2.05.012 (SS-05 dispatcher Rule 8 — IEC-104 content-first classification extension);
  BC-2.10.010 (SS-10 MITRE mapping — T0881 technique seeding extension);
  BC-2.12.025 (SS-12 CLI — `--iec104` flag extension)
- **Subsystems touched:** SS-19 (new — IEC-104 passive analyzer), SS-05 (dispatcher
  Rule 8), SS-10 (MITRE mapping extension), SS-12 (CLI `--iec104` flag)
- **Estimated stories:** 9 (STORY-167..174, STORY-180)
- **Feature ID:** feature-iec104
- **Total points:** 41 (STORY-167: 5, STORY-168: 5, STORY-169: 3, STORY-170: 5,
  STORY-171: 3, STORY-172: 5, STORY-173: 5, STORY-174: 5, STORY-180: 5)
- **Waves:** 76–83, 85
- **Status:** delivered (9/9 `delivered` per STORY-INDEX v4.19)
- **VPs:** VP-044 (Kani), VP-045/VP-046 (proptest), VP-047 (fuzz)

**STORY-180 (wave 85):** IEC-104 timed control-command detection, TypeIDs 58–64
(BC-2.19.029 + BC-2.19.030 + BC-2.19.022 v1.1 regression guard); 5 pts; delivered.

**Rationale:** IEC-104 (IEC 60870-5-104, the TCP/IP-routable companion to IEC 60870-5-101)
follows the same diamond-free, strictly linear decomposition already proven for DNP3
(E-15) and ENIP (E-20), because `src/analyzer/iec104.rs` is a single-file target with
no independent-file parallelism opportunity: (1) pure-core APCI header parse + VP-044
Kani skeleton (STORY-167, wave 76); (2) frame-format discrimination + U-format session
state machine — STARTDT/STOPDT/TESTFR (STORY-168, wave 77, dep=167); (3) ASDU header
extraction — TypeID/VSQ/COT/CASDU/IOA (STORY-169, wave 78, dep=168); (4) control-command
detection, TypeIDs 45–51 + C_RP + interrogation + reserved TypeIDs (STORY-170, wave 79,
dep=169); (5) N(S)/N(R) sequence tracking + desync detection (STORY-171, wave 80,
dep=168+170, file-seq edge); (6) carry buffers + frame-walk loop + flow lifecycle
(STORY-172, wave 81, dep=170+171); (7) dispatcher integration — DispatchTarget::Iec104,
T0881 six-part atomic, `--iec104` flag, SUPPORTED_PORTS (STORY-173, wave 82, dep=172);
(8) formal hardening — Kani/proptest/fuzz/mutation (STORY-174, wave 83, dep=173).
STORY-180 (wave 85) extends the control-command detection layer to the timed-command
TypeID range (58–64) once the wave-85-spec-evolution addressed IEC104-TIMED-CMD-GAP-001.
Serialization is enforced throughout by file contention on `src/analyzer/iec104.rs`
(F-F3P2-005 precedent, matching the STORY-170→171 file-sequencing edge).

**DRIFT-EPICS-STALE-v21 correction (v2.2, 2026-09-04):** This epic did not exist in
epics.md v2.1 (frozen 2026-07-02, before feature-iec104 was decomposed at D-440,
2026-07-14). Added in full this pass, reconciled against STORY-INDEX v4.19 ("E-22 | ...
| 9 | 41" row) and BC-INDEX v2.37's canonical BC-count derivation chain (feature-iec104
v2.23 adds 27 rows to SS-19 + 1 row each to SS-05/SS-10/SS-12; v2.32 adds 1 more SS-19
row; v2.35 adds 2 more SS-19 rows — 30 SS-19 + 3 cross-subsystem extensions = 33 BCs).

---

## Epic E-23: S7comm / ISO-on-TCP Protocol Dissection (feature-s7comm, ADR-014)

- **Goal:** An ICS/OT security analyst can point wirerust at a pcap containing S7comm
  traffic (Siemens S7-300/400/1200/1500 PLC protocol, TCP port 102, ISO-on-TCP/RFC 1006
  transport) and receive structured, multi-tag findings for engineering-workstation
  activity that indicates reconnaissance, unauthorized configuration changes, or
  firmware/program tampering — Read/Write Var operations, program Download/Upload
  session correlation (T0843 Program Download, T0889 Modify Program, T0821 Modify
  Controller Tasking), PLC Control/Stop commands, and Setup Comm negotiation — with a
  frozen ISO-on-TCP (TPKT/COTP) transport-framing layer shared by any future ISO-on-TCP
  protocol (S7comm-plus is observed via DetectionOnly framing only, never dissected),
  per-flow and cross-flow correlation state, an opt-in `--s7comm` CLI flag, and full
  formal verification (Kani bounds-safety proofs, proptest totality properties, and a
  combined-module fuzz harness) — mirroring the passive-analysis pattern already
  established for Modbus (E-14), DNP3 (E-15), ENIP (E-20), and IEC-104 (E-22).
- **BCs:**
  BC-2.20.001, BC-2.20.002, BC-2.20.003, BC-2.20.004, BC-2.20.005, BC-2.20.006,
  BC-2.20.007, BC-2.20.008, BC-2.20.009, BC-2.20.010, BC-2.20.011, BC-2.20.012,
  BC-2.20.013, BC-2.20.014, BC-2.20.015, BC-2.20.016 (SS-20 ISO-on-TCP framing —
  TPKT/COTP pure-core parse, carry-buffer reassembly, resync, frozen module boundary,
  16 BCs, new subsystem);
  BC-2.21.001, BC-2.21.002, BC-2.21.003, BC-2.21.004, BC-2.21.005, BC-2.21.006,
  BC-2.21.007, BC-2.21.008, BC-2.21.009, BC-2.21.010, BC-2.21.011, BC-2.21.012,
  BC-2.21.013, BC-2.21.014, BC-2.21.015, BC-2.21.016, BC-2.21.017, BC-2.21.018,
  BC-2.21.019, BC-2.21.020, BC-2.21.021, BC-2.21.022, BC-2.21.023, BC-2.21.024,
  BC-2.21.025, BC-2.21.026, BC-2.21.027, BC-2.21.028, BC-2.21.029, BC-2.21.030,
  BC-2.21.031, BC-2.21.032, BC-2.21.033, BC-2.21.034, BC-2.21.035, BC-2.21.036,
  BC-2.21.037, BC-2.21.038, BC-2.21.039, BC-2.21.040, BC-2.21.041 (SS-21 S7comm domain
  analysis — flow state, four-way protocol_id dispatch, function-code classification,
  Userdata parsing, session correlation, cross-flow correlation, MITRE emissions, 41 BCs,
  new subsystem);
  BC-2.05.013 (SS-05 dispatcher Rule 9 — content-first TCP port 102 classification
  extension);
  BC-2.18.005, BC-2.18.006 (SS-18 protocol coverage catalog — Support enum promotion +
  S7comm/S7comm-plus/port-102 catalog entries)
- **Subsystems touched:** SS-20 (new — ISO-on-TCP transport framing), SS-21 (new —
  S7comm domain analysis), SS-05 (dispatcher Rule 9), SS-10 (MITRE mapping — T0843/
  T0889/T0821 seeding), SS-12 (CLI `--s7comm` flag), SS-18 (protocol coverage catalog
  Support enum + BC-2.18.003/004 amendments)
- **Estimated stories:** 11 (STORY-184..194)
- **Feature ID:** feature-s7comm
- **Total points:** 71 (STORY-184: 3, STORY-185: 5, STORY-186: 5, STORY-187: 8,
  STORY-188: 8, STORY-189: 5, STORY-190: 5, STORY-191: 8, STORY-192: 8, STORY-193: 8,
  STORY-194: 8)
- **Waves:** 87–97
- **Status:** draft (11/11 `draft` per STORY-INDEX — pre-F3-gate; human F3 gate will
  move to `ready`)
- **VPs:** VP-048 (Kani, TPKT bounds safety), VP-049 (Kani, COTP bounds safety +
  protocol_id totality), VP-050 (proptest, carry-buffer reassembly), VP-051 (Kani,
  S7comm header bounds safety), VP-052 (proptest, function-code/Userdata-group
  classification totality), VP-053 (proptest, protocol_id four-way dispatch totality),
  VP-054 (proptest, program-download/upload structural disjointness), VP-055 (fuzz,
  combined ISO-on-TCP/S7comm parse chain), plus amended VP-004 (dispatcher oracle),
  VP-007 (MITRE catalog completeness), and VP-041 (protocol coverage partition)

**Rationale:** S7comm follows the same diamond-free, strictly linear decomposition
already proven for DNP3 (E-15), ENIP (E-20), and IEC-104 (E-22): a single new-protocol
epic touching two new files (`src/analyzer/iso_on_tcp.rs`, `src/analyzer/s7comm.rs`)
with no independent-file parallelism opportunity across the whole chain. Unlike the
other ICS analyzers, S7comm's transport is not raw TCP payload but a nested framing
stack (TPKT over TCP, COTP over TPKT, S7comm/S7comm-plus over COTP's `protocol_id`
discriminator byte) — so the epic splits into a dedicated ISO-on-TCP transport-framing
layer (SS-20, stories 184-186) that is explicitly frozen as reusable infrastructure for
any future ISO-on-TCP protocol, followed by the S7comm domain-analysis layer (SS-21,
stories 187-193) built on top of it: (1) TPKT core parser + VP-048 Kani skeleton
(STORY-184, wave 87, dep=[]); (2) COTP TPDU-type parser + protocol_id extraction +
VP-049 Kani skeleton (STORY-185, wave 88, dep=184); (3) carry-buffer reassembly,
walk-first frame extraction, resync, and the frozen SS-20/SS-21 module boundary
(STORY-186, wave 89, dep=185); (4) flow-state completion, four-way protocol_id dispatch
skeleton, and `parse_s7comm_header` (STORY-187, wave 90, dep=186); (5) Job/Ack_Data
function-code classification — Setup Comm, Read/Write Var, Download/Upload triads, PLC
Control, PLC Stop (STORY-188, wave 91, dep=187); (6) Userdata structural parse and
function-group classification, including the load-bearing Group 0x03/0x04/0x07
correction (STORY-189, wave 92, dep=188); (7) S7comm-plus DetectionOnly framing +
session-setup metadata + dispatch totality (STORY-190, wave 93, dep=189); (8)
download-session correlation state machine + T0843/T0889/T0821 new MITRE techniques,
six-part atomic (STORY-191, wave 94, dep=190); (9) cross-flow correlation state + reused
MITRE emissions + excluded-technique non-goals (STORY-192, wave 95, dep=191); (10)
dispatcher integration — `DispatchTarget::S7comm` Rule 9, Support enum catalog
promotion, `--s7comm` flag (STORY-193, wave 96, dep=192); (11) formal hardening —
Kani/proptest/fuzz/mutation, VP-048..055 full runs + VP-004/007/041 re-verification
(STORY-194, wave 97, dep=193). Serialization is enforced throughout by file contention:
`src/analyzer/iso_on_tcp.rs` for stories 184-186, then `src/analyzer/s7comm.rs` for
stories 186-193 (STORY-186 straddles both files at the module boundary), with STORY-193
additionally touching `src/dispatcher.rs`/`src/cli.rs`/`src/main.rs`/`src/protocols.rs`
and STORY-194 touching both analyzer files plus `src/mitre.rs` for re-verification —
matching the F-F3P2-005 single-file-serialization precedent already established for
IEC-104 (E-22) and the Protocol Coverage Catalog (E-21). ADR-014 Decision 3 (human
ratification) also resolves the port-102 catalog-model fix: `KnownProtocol` gains a
per-entry `Support` enum (Supported/KnownUnsupported/DetectionOnly, reusing ADR-0012
Decision 2's vocabulary) — S7comm=Supported, S7comm-plus=DetectionOnly ("observed, not
dissected") — delivered in STORY-193 as BC-2.18.005/006 plus amendments to the existing
BC-2.18.003/004 catalog-partition BCs from E-21.

---

## Estimated Story Count Summary

| Epic | Stories (STORY-INDEX v4.19) | Notes |
|------|----------------------------|-------|
| E-1  | 5  | |
| E-2  | 11 | |
| E-3  | 3  | |
| E-4  | 6  | |
| E-5  | 11 | +3 vs v2.0: STORY-144/145/146 (fix-tls-clienthello-frag F3, 2026-06-29) |
| E-6  | 1  | |
| E-7  | 3  | |
| E-8  | 8  | +2 vs v2.0: STORY-120 (FindingsRender enum migration) + STORY-129 (mitre_attack enrichment); +1 vs v2.1: STORY-160 (snake_case JSON-enum + schema_version envelope, 2026-07-08) — DRIFT-EPICS-STALE-v21 remediation |
| E-9  | 5  | |
| E-10 | 1  | |
| E-11 | 23 | +17 vs v2.1: STORY-155, 157, 158, 159, 161, 162, 163, 164, 165, 166 (waves 71-75 S-7.02 cycle-close codifications), STORY-175/176/177/178/179 (feature-iec104 cycle-close, 2026-07-18), STORY-182/183 (wave-86, D-544 human story-approval gate, 2026-07-25) — DRIFT-EPICS-STALE-v21 remediation |
| E-12 | 3  | |
| E-13 | 2  | |
| E-14 | 5  | |
| E-15 | 7  | |
| E-16 | 6  | +1 vs v2.1: STORY-156 (ARP findings unbounded-cap doc + regression test, BC-2.16.016, maint-2026-07-06) — DRIFT-EPICS-STALE-v21 remediation |
| E-17 | 2  | |
| E-18 | 3  | +1 vs v2.0: STORY-122 (enum→struct reshape, D-120 split-A, 2026-06-18) |
| E-19 | 6  | |
| E-20 | 12 | +1 vs v2.1 (v2.1's 11 already included STORY-148): STORY-181 (SEC-001 ENIP split-borrow refactor, wave 85, D-509) — DRIFT-EPICS-STALE-v21 remediation |
| E-21 | 4  | NEW: STORY-151/152/153/154 (feature-protocol-coverage, 2026-07-02) |
| E-22 | 9  | NEW epic (v2.2): STORY-167..174 (feature-iec104 F3 decomposition, D-440, waves 76-83) + STORY-180 (timed control-command TypeIDs 58-64, wave 85, D-493) — DRIFT-EPICS-STALE-v21 remediation |
| E-23 | 11 | NEW epic (v2.4): STORY-184..194 (feature-s7comm F3 decomposition, ADR-014, waves 87-97) |
| **Total** | **147** | Verified against dependency-graph.md v3.13 total_stories=147 |
