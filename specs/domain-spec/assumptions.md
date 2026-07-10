---
artifact: L2-assumptions
document_type: domain-spec-shard
level: L2
version: "1.0"
producer: business-analyst
traces_to: ../domain/domain-spec.md
project: wirerust
status: backfill-complete
generated: 2026-07-09
inputs:
  - .factory/maintenance/risk-assumption-monitoring.md
  - .factory/tech-debt-register.md
input-hash: "0447a72"
---

# wirerust Assumptions Register

This shard formalizes the 11 informal ASM-CAND items recorded across maintenance sweeps into
permanent L2 spec artifacts ASM-001..ASM-011. Each entry carries a `formalized_from` field
mapping it to its originating ASM-CAND identifier and preserves the exact validation status
recorded in `risk-assumption-monitoring.md` (maint-2026-07-09, the canonical current-status
source).

This is a **backfill** document: it encodes ground truth only and introduces no new assumptions.
Every row traces to its ASM-CAND source.

**Validation status vocabulary:**
- `validated` — assumption holds; evidence from formal proof, CI, or multiple releases without invalidation.
- `accepted` — assumption acknowledged as unvalidated or unverifiable; formally accepted with written rationale.
- `monitor` — assumption holds today; re-evaluate if the listed trigger conditions occur.
- `open` — assumption not yet validated; validation evidence is needed.

---

## Summary Table

| ID | formalized_from | Title | Validation Status | ADR / Spec Ref |
|----|----------------|-------|-------------------|---------------|
| ASM-001 | ASM-CAND-001 | Port-only classification sufficient for Modbus TCP | validated | ADR-005 Decision 1 |
| ASM-002 | ASM-CAND-002 | Port-only classification sufficient for DNP3 TCP | validated | ADR-007 Decision 1 |
| ASM-003 | ASM-CAND-003 | Anomaly thresholds adequate for forensic use without labelled corpus | accepted | O-03; PR #382 |
| ASM-004 | ASM-CAND-004 | Full pcap eager-load into Vec<RawPacket> is acceptable | open | NFR-VIO-001 |
| ASM-005 | ASM-CAND-005 | TLS byte-0/byte-1 gate (0x16 0x03) is adequate | validated | ADR-011; Smell #10 |
| ASM-006 | ASM-CAND-006 | ARP binding table LRU eviction at capacity 65,536 is acceptable | validated | ADR-008 |
| ASM-007 | ASM-CAND-007 | DNP3 FIR=1-only parse sufficient for v1 detections | monitor | ADR-007 Decision 4 |
| ASM-008 | ASM-CAND-008 | DNP3 CRC skip safe for PCAP replay of real captures | open | ADR-007 Decision 3 |
| ASM-009 | ASM-CAND-009 | ARP storm rate default (50 frames/s) is a conservative engineering choice | accepted | ADR-008; PR #382 |
| ASM-010 | ASM-CAND-010 | Port-44818 classification sufficient for EtherNet/IP TCP | validated | ADR-010 |
| ASM-011 | ASM-CAND-011 | TLS handshake-only reassembly sufficient for v1 TLS detections | validated | ADR-011 |

---

## ASM-001 — Port-Only Classification Sufficient for Modbus TCP

**formalized_from:** ASM-CAND-001
**Title:** Port-only classification is sufficient for Modbus TCP (ADR-005 Decision 1)
**Validation status:** validated
**Validation evidence:** VP-022 Kani proof verifies the three-point dispatch gate is sound.
Assumption holds across all releases without a counter-example from CI or production captures.
**Revisit trigger:** If a non-Modbus protocol is observed in ICS captures on port 502, or if
ADR-005 Decision 1 is revised.

**Statement:** TCP traffic on port 502 is treated as Modbus without content-based verification.
This is sufficient for the ICS forensic triage use case: port 502 is IANA-registered exclusively
for Modbus and is not shared with other protocols in production ICS environments.

**References:** ADR-005 Decision 1; VP-022; see also R-005 (false-routing risk, accepted).

---

## ASM-002 — Port-Only Classification Sufficient for DNP3 TCP

**formalized_from:** ASM-CAND-002
**Title:** Port-only classification is sufficient for DNP3 TCP (ADR-007 Decision 1)
**Validation status:** validated
**Validation evidence:** VP-023 Kani proof verifies the DNP3 dispatch gate. DNP3 observability
counters added in v0.11.5 (PR #370: `dropped_findings`, `master_addrs_dropped`,
`pending_requests_evicted`) improve telemetry fidelity without changing the dispatch gate or
classification assumption.
**Revisit trigger:** If non-DNP3 traffic is observed on port 20000 in production captures, or
if ADR-007 Decision 1 is revised.

**Statement:** TCP traffic on port 20000 is treated as DNP3 without content-based verification.
Port 20000 is IANA-registered for DNP3/Secure Authentication and is not shared with other
protocols in production ICS environments.

**References:** ADR-007 Decision 1; VP-023.

---

## ASM-003 — Anomaly Thresholds Adequate for Forensic Use Without Labelled Corpus

**formalized_from:** ASM-CAND-003
**Title:** Anomaly thresholds are adequate for forensic use without a labelled traffic corpus (O-03)
**Validation status:** accepted
**Validation evidence:** PR #382 (commit 624bae3), 2026-07-08. README § Known Limitations
"Reassembly anomaly thresholds" subsection added, documenting `--overlap-threshold` (50),
`--small-segment-threshold` (100), `--small-segment-max-bytes` (16), and
`--out-of-window-threshold` (100) as uncalibrated engineering defaults with written rationale.
CLI overridability is stated. TD-MAINT-THRESHOLD-CALIB-001 RESOLVED-ACCEPTED.
**Revisit trigger:** If a labelled ICS traffic corpus becomes available for threshold
calibration, or if production use reveals systematic false-positive or false-negative rates at
the defaults. A future calibration exercise remains an optional P3 backlog item.

**Statement:** The reassembly anomaly detection thresholds are CLI-overridable, research-
documented engineering defaults. Without a labelled capture corpus, their FP/TP rates cannot
be empirically validated. The product's forensic triage use case (analyst-reviewed output)
tolerates this uncertainty.

**References:** O-03; domain-debt.md; PR #382; README § Known Limitations.

---

## ASM-004 — Full PCAP Eager-Load into Vec<RawPacket> is Acceptable

**formalized_from:** ASM-CAND-004
**Title:** Full pcap eager-load into Vec<RawPacket> is acceptable for the forensic triage use case (NFR-VIO-001)
**Validation status:** open
**Validation evidence:** None. NFR-VIO-001 is OPEN-DEBT. No streaming refactor has been scoped.
**Revisit trigger:** If users report OOM failures on large captures, or if the product formally
targets capture files larger than available RAM. See also R-002 (README documentation risk).

**Statement:** wirerust ingests pcap files by loading all raw packets into memory before
analysis. This design is acceptable if captures fit within available RAM; it is not suitable
for arbitrarily large capture files. The README claim of "multi-GB capture" support is
inaccurate against this implementation (see R-002).

**References:** NFR-VIO-001; domain-debt.md; see R-002.

---

## ASM-005 — TLS Byte-0/Byte-1 Gate (0x16 0x03) is Adequate

**formalized_from:** ASM-CAND-005
**Title:** TLS byte-0/byte-1 gate (0x16 0x03) is adequate for TLS dispatch (ADR-011; Smell #10)
**Validation status:** validated
**Validation evidence:** ADR-011 records the design decision and rationale. The layering
separation (gate → dispatch → reassembly) is confirmed by ADR-011. VP-039 and VP-040 are in
force for TLS handshake reassembly. No misrouting incidents detected through wave-72.
**Revisit trigger:** If a non-TLS protocol using a 0x16 0x03 two-byte prefix appears in ICS
captures, or if ADR-011 is revised.

**Statement:** The TLS content-type (0x16) plus version byte (0x03) two-byte prefix check,
with a 5-byte buffer minimum, is an adequate TLS dispatch gate for the ICS forensic triage
use case. Architecture Smell #10 (loose TLS gate: byte[2] unchecked) is documented as
theoretical; zero misrouting tests have been observed.

**References:** ADR-011; Smell #10; VP-039; VP-040; INV-2 (content-first dispatch precedence).

---

## ASM-006 — ARP Binding Table LRU Eviction at Capacity 65,536 is Acceptable

**formalized_from:** ASM-CAND-006
**Title:** ARP binding table LRU eviction at capacity 65,536 is acceptable (ADR-008)
**Validation status:** validated
**Validation evidence:** ADR-008 documents the capacity and eviction rationale. The
`bindings_evicted` counter (v0.11.4, PR #365) provides ongoing observability of eviction
events. PF-001 (PR #384) ensured all arp.rs counter sites use `saturating_add`, improving
counter discipline. No eviction-related loss of detection has been observed.
**Revisit trigger:** If production captures show high `bindings_evicted` counts indicating
legitimate ARP churn above 65,536 unique MAC-to-IP bindings, revisit capacity or eviction
policy.

**Statement:** The ARP analyzer maintains a binding table with LRU eviction at a capacity of
65,536 entries. For ICS networks, which typically have a bounded and stable set of endpoints,
this capacity is adequate. High-churn non-ICS environments may encounter evictions under load.

**References:** ADR-008; `bindings_evicted` counter.

---

## ASM-007 — DNP3 FIR=1-Only Parse Sufficient for v1 Detections

**formalized_from:** ASM-CAND-007
**Title:** DNP3 FIR=1-only parse is sufficient for v1 detections (ADR-007 Decision 4)
**Validation status:** monitor
**Validation evidence:** No new DNP3 multi-fragment detections were added through wave-72.
v0.11.5 (PR #370) added observability counters without changing detection scope. The
assumption holds for all current v1 detections.
**Revisit trigger:** If the next ICS feature cycle adds DNP3 multi-fragment detections (for
example, fragmented block commands), re-evaluate whether FIR=1-only parse misses relevant
attack patterns.

**Statement:** The DNP3 analyzer only parses Application Layer Data Units with FIR=1 (first
fragment). This is sufficient for all v1 DNP3 detections (T1691.001, T0827), which are emitted
on the first fragment of a block command or anomalous sequence. Multi-fragment reassembly is
not required for v1 scope.

**References:** ADR-007 Decision 4.

---

## ASM-008 — DNP3 CRC Skip Safe for PCAP Replay of Real Captures

**formalized_from:** ASM-CAND-008
**Title:** DNP3 CRC skip is safe for PCAP replay of real captures (ADR-007 Decision 3)
**Validation status:** open
**Validation evidence:** ADR-007 Decision 3 documents the design decision. The Crain/Sistrunk
caveat normative note to ADR-007 Decision 3 has been recommended but unactioned for three
consecutive sweeps (REC-005).
**Revisit trigger:** If production captures contain corrupted DNP3 CRC bytes (for example,
Crain/Sistrunk fuzzing scenarios), the CRC skip may cause the analyzer to accept corrupted
payloads that a real DNP3 device would reject. Add the Crain/Sistrunk normative note to
ADR-007 Decision 3 in the next documentation PR.

**Statement:** wirerust skips DNP3 CRC validation because PCAPs are replayed captures of
real, already-validated traffic. For synthesized or corrupted captures (for example,
Crain/Sistrunk test vectors), the CRC skip may cause the analyzer to process corrupted
payloads as valid DNP3 data.

**References:** ADR-007 Decision 3; REC-005 (Crain/Sistrunk normative note, third sweep
unactioned as of maint-2026-07-09).

---

## ASM-009 — ARP Storm Rate Default (50 frames/s) is a Conservative Engineering Choice

**formalized_from:** ASM-CAND-009
**Title:** ARP storm rate default (50 frames/s) is a conservative engineering choice (ADR-008)
**Validation status:** accepted
**Validation evidence:** PR #382 (commit 624bae3), 2026-07-08. README § Known Limitations
"ARP storm rate default" subsection added, documenting 50 frames/second as an engineering
choice without OT-network reference guidance, with CLI overridability noted. The
`storm_counters_evicted` counter (v0.11.4) provides ongoing observability. PF-001 (PR #384)
ensured all arp.rs counter-increment sites use `saturating_add`.
TD-MAINT-THRESHOLD-CALIB-001 RESOLVED-ACCEPTED.
**Revisit trigger:** If ICS network traffic data shows that legitimate ARP storm rates in
target environments exceed 50 frames/second per source MAC, or if a calibration exercise
against labelled OT traffic data becomes available.

**Statement:** The ARP storm rate threshold (50 frames/second per source MAC) is an
uncalibrated engineering default. No published OT-network ARP storm rate reference data was
available during design. The threshold is CLI-overridable.

**References:** ADR-008; PR #382; README § Known Limitations; see also R-006 (DNP3 threshold,
resolved via same TD-MAINT-THRESHOLD-CALIB-001 acceptance).

---

## ASM-010 — Port-44818 Classification Sufficient for EtherNet/IP TCP

**formalized_from:** ASM-CAND-010
**Title:** Port-44818 classification is sufficient for EtherNet/IP TCP (ADR-010)
**Validation status:** validated
**Validation evidence:** ADR-010 documents the design decision. IANA-exclusive port 44818
registration is the practical discriminator. No non-ENIP traffic on port 44818 has been
observed across 6 releases (through v0.11.5 and wave-72 develop HEAD). PF-001 (PR #384)
converted ENIP counter increments to `saturating_add` without changing the dispatch gate or
classification logic.
**Revisit trigger:** If non-EtherNet/IP traffic on port 44818 appears in production captures.
A dedicated ENIP gate Kani proof is recommended for the next ENIP hardening cycle (REC-007).

**Statement:** TCP traffic on port 44818 is treated as EtherNet/IP without content-based
verification. Port 44818 is IANA-registered exclusively for EtherNet/IP. No dedicated Kani
gate proof yet exists (see R-011).

**References:** ADR-010; see R-011; REC-007.

---

## ASM-011 — TLS Handshake-Only Reassembly Sufficient for v1 TLS Detections

**formalized_from:** ASM-CAND-011
**Title:** TLS handshake-only reassembly is sufficient for v1 TLS detections (ADR-011)
**Validation status:** validated
**Validation evidence:** ADR-011 documents the scope decision. VP-039 and VP-040 are in force.
BC-INDEX v2.22 maintains the handshake-scope invariant. Wave-72 STORY-160 includes no TLS
analyzer changes. Assumption holds across 5 releases since introduction (through v0.11.5).
**Revisit trigger:** If a future TLS feature requires post-handshake record analysis (for
example, detecting malicious data exfiltration via TLS application records), re-evaluate
whether handshake-only reassembly is sufficient.

**Statement:** The TLS analyzer reassembles and inspects only the TLS handshake phase.
Post-handshake application record data is not analyzed. All v1 TLS detections (weak ciphers,
SNI anomalies, JA3 fingerprints) are grounded in handshake data only.

**References:** ADR-011; VP-039; VP-040; BC-INDEX v2.22.
