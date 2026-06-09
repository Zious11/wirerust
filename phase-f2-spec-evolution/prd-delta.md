---
document_type: prd-delta
feature_id: issue-007-modbus-analyzer
github_issue: 7
title: "F2 PRD Delta — Modbus TCP Analyzer (SS-14)"
status: active
producer: product-owner
created: 2026-06-09
revised: 2026-06-09
base_prd_version: "1.0"
new_prd_version: "1.2"
traces_to:
  - .factory/specs/prd.md
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md
---

# F2 PRD Delta — Issue #7: Modbus TCP Protocol Analyzer

## 1. Overview

This document records all PRD changes introduced by Feature #7 (Modbus/ICS analyzer). It
is the companion to `architecture-delta.md` on the specification side. The state-manager
merges this delta into `prd.md` (done in F2 burst — `prd.md` is now v1.2).

PRD version bumps:
- **1.0 → 1.1** (MINOR: new feature capability added — SS-14 Modbus, 25 BCs, ADR-005)
- **1.1 → 1.2** (BREAKING: `Finding.mitre_techniques` Vec schema change per ADR-006 Decisions 11-13; targets v0.3.0)

---

## 2. New Behavioral Contracts

### SS-14: Modbus/ICS Analysis (25 BCs, BC-2.14.001..025)

All 25 BCs are greenfield (origin: greenfield; lifecycle_status: active;
introduced: v0.3.0-feature-007). Files reside in
`.factory/specs/behavioral-contracts/ss-14/`.

| BC ID | Title | Priority | Group |
|-------|-------|----------|-------|
| BC-2.14.001 | MBAP Header Accepted for Well-Formed 8-Byte-Minimum ADU | P0 | A: MBAP Parse |
| BC-2.14.002 | MBAP Header Rejected for ADU Shorter Than 8 Bytes | P0 | A: MBAP Parse |
| BC-2.14.003 | MBAP Header Rejected When Protocol ID is Not 0x0000 | P0 | A: MBAP Parse |
| BC-2.14.004 | MBAP Header Rejected When Length is Outside [2, 253] | P0 | A: MBAP Parse |
| BC-2.14.005 | classify_fc Is Total Over All 256 FC Values (Covers Read, Write, Diagnostic, Exception, and Unknown Classes) | P0 | B: FC Classification |
| BC-2.14.006 | Exception Response Detection — FC High Bit Set Identifies Exception and Recovers Original FC | P0 | B: FC Classification |
| BC-2.14.007 | Write-Class FC Classification — State-Changing Function Codes Identified as Elevated-Risk | P0 | B: FC Classification |
| BC-2.14.008 | Diagnostic-Class FC Classification and Sub-Function Dispatch (0x08 and 0x2B) | P1 | B: FC Classification |
| BC-2.14.009 | Request PDU Inserted into Per-Flow Pending Table Keyed on (Transaction ID, Unit ID) | P0 | C: Transaction Correlation |
| BC-2.14.010 | Response PDU Matched Against Pending Table and Entry Removed on FC Echo Match | P0 | C: Transaction Correlation |
| BC-2.14.011 | Exception Response PDU Attributed to Originating Request FC via Pending Table Lookup | P0 | C: Transaction Correlation |
| BC-2.14.012 | Pending Table Bounded to MAX_PENDING_TRANSACTIONS=256; New Requests Dropped When Full | P0 | C: Transaction Correlation |
| BC-2.14.013 | Write-Class FC in Request Direction Emits T0855 (Unauthorized Command Message) Finding | P0 | D: Write Detection |
| BC-2.14.014 | Write FC 0x06/0x10/0x16 in Request Direction Emits T0836 (Modify Parameter) Finding | P0 | D: Write Detection |
| BC-2.14.015 | Write FC to Coil (0x05/0x0F) Emits Finding Tagged ["T0855","T0835"] (Manipulate I/O Image; 0x06/0x10/0x16 Excluded) | P0 | D: Write Detection |
| BC-2.14.016 | Coordinated Write Sequence to Holding Registers Within 5-Second Window Tags the Per-PDU Finding with T0831 Inline (Co-Tagged as ["T0855","T0836","T0831"]) | P0 | E: Coordinated Write |
| BC-2.14.017 | Write-Rate Burst Exceeding Either Burst or Sustained Threshold Emits T0806 Brute Force I/O and T0855 Findings | P0 | E: Burst Detection |
| BC-2.14.018 | Diagnostics FC 0x08 Sub-Function 0x0004 or 0x0001 Emits T0814 Denial of Service Finding | P0 | F: Diagnostic/DoS |
| BC-2.14.019 | Exception Response Anomaly — Burst of Exception Codes Emits Anomaly Finding for Recon/Scanning | P0 | F: Exception Burst |
| BC-2.14.020 | Reconnaissance Function Codes (0x11, 0x2B/0x0E) Emit T0888 Remote System Information Discovery Finding (v2.0 — Decision 12; old title referenced T0846) | P1 | G: Anomaly/Recon |
| BC-2.14.021 | summarize() Returns AnalysisSummary with Specified Per-Analyzer Summary Keys (SIX keys including dropped_findings) | P1 | G: Summary/Stats |
| BC-2.14.022 | MAX_FINDINGS Cap (10,000) and Poison-Skip Behavior for ModbusAnalyzer | P0 | G: Bounded Resource |
| BC-2.14.023 | --modbus CLI Flag Enables ModbusAnalyzer; --all Includes Modbus; Default-Off; Requires Stream Reassembly | P0 | H: CLI Integration |
| BC-2.14.024 | --modbus-write-burst-threshold and --modbus-write-sustained-threshold Configure Dual-Window Burst Detection (v2.0 — Decision 11; old title referenced --modbus-write-threshold) | P0 | H: CLI Integration |
| BC-2.14.025 | StreamDispatcher Classifies Port-502 Flows to DispatchTarget::Modbus as Rule 5 (After Content and TLS/HTTP Port Rules) | P0 | H: Dispatcher Integration |

**MITRE ATT&CK for ICS techniques covered (v2 — post-Decision-12):** T0855, T0836, T0814,
T0806, T0835, T0831, T0888 (Remote System Information Discovery — replaces T0846 for recon
FCs). T0846 is seeded in the catalog but NOT emitted by the Modbus analyzer per Decision 12.
Matrix discriminator: T0xxx namespace → ICS matrix; T1xxx-T9xxx → Enterprise matrix.

---

## 3. New NFR Candidates

The following NFRs should be added to `prd-supplements/nfr-catalog.md` in the next
supplement-update burst:

| Proposed ID | Category | Requirement | Numerical Target | Validation Method |
|-------------|----------|-------------|-----------------|-------------------|
| NFR-MOD-001 | RES | `ModbusAnalyzer.all_findings` bounded by MAX_FINDINGS | ≤ 10,000 findings per run | Unit test: pre-fill to cap, verify no growth |
| NFR-MOD-002 | RES | `ModbusFlowState.pending` table bounded by MAX_PENDING_TRANSACTIONS | ≤ 256 entries per flow | Unit test: flood with 300 requests, verify table stays at 256 |
| NFR-MOD-003 | PERF | Modbus PDU parsing throughput (single-threaded, synthetic 8-byte PDUs, no findings) | ≥ 500,000 PDUs/second on reference hardware | Criterion benchmark |
| NFR-MOD-004 | SEC | `parse_mbap_header` never panics on any input | Kani proof (VP-022 sub-property A) | Kani formal verification |
| NFR-MOD-005 | SEC | `classify_fc` never panics on any u8 input | Kani proof (VP-022 sub-property B) | Kani formal verification |

---

## 4. Edge Case Catalog Additions

The following edge cases are notable boundary conditions for the SS-14 capability. They
supplement the individual BC-level EC-NNN entries in each BC file.

### EC-MOD-001: Non-Modbus Traffic on Port 502
**Description:** Port 502 is IANA-registered for Modbus, but any traffic can appear there
(TLS, HTTP, unknown binary). The classifier checks content rules first.
**Expected behavior:** TLS content on port 502 → `DispatchTarget::Tls` (Rule 1 fires).
HTTP content on port 502 → `DispatchTarget::Http` (Rule 2 fires). Non-Modbus binary on
port 502 → `DispatchTarget::Modbus` (Rule 5); `ModbusAnalyzer` receives data; all ADUs
fail the 3-point validity gate; `parse_errors` incremented; no findings emitted.
**Covered by:** BC-2.14.025 EC-001, EC-002, EC-004.

### EC-MOD-002: Fragmented MBAP Header Across TCP Segments
**Description:** The 7-byte MBAP header may be split across two TCP segments
(e.g., first segment: 6 bytes, second segment: the remaining bytes). The reassembler
delivers in-order contiguous bytes to `on_data`, but the first call may carry only 6 bytes.
**Expected behavior:** `parse_mbap_header(6_bytes)` returns `None` (< 8 bytes);
`parse_errors` incremented for the partial call. On the next `on_data` call with accumulated
bytes from the reassembler (reassembler buffers and delivers contiguous bytes), the full ADU
is parsed successfully.
**Note:** The TCP reassembler (SS-04) delivers contiguous in-order bytes to each `on_data`
call. Partial ADU handling is not the analyzer's responsibility — but partial first-call
delivery (when the reassembler has not yet accumulated a full ADU) is expected behavior.
**Covered by:** BC-2.14.025 EC-010; BC-2.14.001 EC-001.

### EC-MOD-003: Pending Table Flood (MAX_PENDING_TRANSACTIONS=256)
**Description:** An attacker sends 300+ outstanding requests on a single flow without
responses, attempting to cause unbounded memory growth in the pending table.
**Expected behavior:** The pending table accepts the first 256 requests. Requests 257-300+
are silently dropped (not inserted; no error, no finding). The 256 existing pending entries
continue to match responses normally. Memory is bounded.
**Covered by:** BC-2.14.012.

### EC-MOD-004: Exception Response Storm
**Description:** An attacker sends many write-class requests and the target device responds
with exception responses (FC high-bit echo). This can indicate a failed attack attempt.
**Expected behavior:** Each exception response where the originating request FC was a
write-class FC generates a T0855 attribution finding (per BC-2.14.011 + BC-2.14.013).
If the exception storm crosses the MAX_FINDINGS cap, subsequent findings are dropped and
`dropped_findings` is incremented.
**Covered by:** BC-2.14.011; BC-2.14.022.

### EC-MOD-005: Write-Burst Window Boundary (Dual-Window Model)
**Description:** Writes arrive at exactly the threshold rate for either the burst or sustained
detector. Both detectors use strict greater-than semantics.
**Burst detector (BC-2.14.017):** Default `write_burst_threshold=20`. Exactly 20 writes in a
1-second window → no burst finding (`window_write_count == 20`, `20 > 20` is false). The 21st
write within the same window tips the threshold → ONE burst Finding emitted with
`mitre_techniques: ["T0806","T0855"]`. Flag: `--modbus-write-burst-threshold` (default 20).
**Sustained detector (BC-2.14.017):** Default `write_sustained_threshold=10`. At elapsed ≥ 2s:
`sustained_window_write_count > write_sustained_threshold * elapsed_secs` (strict `>`). Exactly
at threshold: no firing. One over: fires. Flag: `--modbus-write-sustained-threshold` (default 10).
**Covered by:** BC-2.14.017; BC-2.14.024.

### EC-MOD-006: Multiple ADUs in One TCP Segment
**Description:** The TCP reassembler may deliver a segment containing multiple complete
Modbus ADUs (e.g., a response batch). The `on_data` parsing loop must process all of them.
**Expected behavior:** The offset-advancing loop in `on_data` iterates over all complete ADUs
in the delivered byte slice. Each ADU is parsed, classified, and dispatched independently.
Trailing incomplete ADU bytes are left for the next `on_data` call.
**Covered by:** BC-2.14.001; architecture-delta.md §2.4 (PDU boundary advancement).

### EC-MOD-007: T0831 Coordination Detection — Inline Co-Tag Model
**Description:** T0831 (Manipulation of Control) detects coordinated write sequences to
holding registers within the same flow and time window. For v1, the trigger is: two or more
write FCs targeting holding registers (FC 0x06, 0x10, or 0x16) within a 5-second pcap-timestamp
window within the same flow. T0831 is co-tagged inline on the per-PDU write Finding — no
separate T0831 Finding object is created (Decision 13, ADR-006 §13.5).
**Expected behavior:** First holding-register write → ONE Finding with `mitre_techniques:
["T0855","T0836"]`; `t0831_window_write_count = 1`. Second holding-register write within the
5-second window → ONE Finding with `mitre_techniques: ["T0855","T0836","T0831"]` (T0831 co-tagged
inline); `t0831_burst_emitted = true`. Subsequent writes in the same window → ONE Finding with
`["T0855","T0836"]` (T0831 emit-once exhausted for this window). Window fixed: `T0831_WINDOW_SECS
= 5` (not CLI-configurable). Burst boundary logic is in BC-2.14.017 (write-rate detectors use
separate windows and flags). T0831 detection uses its own independent `t0831_window_*` state fields.
**Covered by:** BC-2.14.016 (fully authored); architecture-delta.md §12.

---

## 5. PRD Section Updates Summary

### 5.1 v1.0 → v1.1 Changes (F2 Initial Burst — ADR-005)

| Section | Change Type | Description |
|---------|-------------|-------------|
| Frontmatter `version` | bump | 1.0 → 1.1 |
| Frontmatter `timestamp` | update | 2026-05-20 → 2026-06-09 |
| Section 1.5 Out of Scope | update | T0855 and 5 ICS techniques removed from "never emitted" note; updated to reflect Modbus analyzer emission |
| Section 2 (BC Index header) | update | Added v1.1 delta note; BC count 219 → 244 |
| Section 2.14 | new | 25 BCs in 8 groups (A-H); Modbus/ICS capability |
| Section 6.3 KD-003 | update | Added BC-2.14.025 row (content-first dispatch for Modbus) |
| Section 6.5 KD-005 | update | Added BC-2.14.013/014/015/016/018 rows; updated seeded ID count 15 → 20 |
| Section 7 RTM | update | Added 25 SS-14 rows |

### 5.2 v1.1 → v1.2 Changes (F2 Modbus Revision — ADR-006, Decisions 11-13)

**Product release target: v0.3.0 (breaking schema change)**

| Section | Change Type | Description |
|---------|-------------|-------------|
| Frontmatter `version` | bump | 1.1 → 1.2 |
| Section 2 BC Index header | update | Added v1.2 delta note (BREAKING schema change, v0.3.0) |
| Section 2 (breaking schema box) | **NEW** | Cross-cutting note: `mitre_technique` → `mitre_techniques`; JSON/CSV breaking changes; affected stories list; ADR-006 refs |
| Section 1.5 Out of Scope | update | T0846 seeded-not-emitted (Decision 12); T0888 replaces T0846 as Modbus-emitted |
| Section 2.10 (note) | update | Seeded count 15→21; emitted count 6→13; T0888 added; T0846 not emitted |
| Section 2.10.005 row | update | "15 Total" → "21 Total" |
| Section 2.14 header note | update | ADR-005+ADR-006; dual-window model; T0888; co-emission model; CLI flags updated |
| Section 2.14.D note | update | v2 co-emission model replacing priority/suppression rule |
| Section 2.14.D rows (013-015) | update | Titles reflect multi-tag co-emission; T0855 co-included in vec |
| Section 2.14.E header + rows (016-017) | update | T0831 co-tagged inline on per-PDU write finding (`["T0855","T0836","T0831"]`); no separate T0831 Finding object; dual-window burst detection (BC-2.14.017) |
| Section 2.14.G row (020) | update | T0888 replaces T0846; 0x07 excluded |
| Section 2.14.H row (024) | update | Dual flags replaces single `--modbus-write-threshold` |
| Section 6.5 KD-005 | update | Seeded count 20→21; T0888 added; multi-tag rendering notes; T0846 not emitted |
| Section 8 O-04 | update | Emitted: T0888 (not T0846); seeded 21; T0846 seeded-not-emitted |

**Existing BCs revised by the v1.2 burst (10 BCs outside SS-14):**

| BC | Version | What Changed |
|----|---------|--------------|
| BC-2.09.001 | v1.4 | `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>`; emission-site count update |
| BC-2.09.006 | v1.5 | `skip_serializing_if` rule: `Vec::is_empty`; multi-tag JSON output case |
| BC-2.10.005 | v1.4 | Seeded count 15 → 21 in title and body |
| BC-2.10.007 | v1.3 | T0888 → Discovery row added to tactic table |
| BC-2.10.008 | v1.5 | grep pattern updated; T0888 replaces T0846 in emitted list; 13 total emitted |
| BC-2.11.013 | v1.6 | Multi-techniques tactic grouping: group by `mitre_techniques[0]` |
| BC-2.11.015 | v1.6 | Empty `mitre_techniques` vec → Uncategorized |
| BC-2.11.017 | v1.5 | Multi-ID rendering: `"MITRE: T0855, T0836"` |
| BC-2.11.020 | v1.5 | Column-6 header renamed `mitre_technique` → `mitre_techniques` |
| BC-2.11.024 | v1.4 | `mitre_technique None` → `mitre_techniques vec![]`; semicolon-join multi-value |

**SS-14 BCs revised by the v1.2 burst (per BC body files, already applied):**

| BC | Version | What Changed |
|----|---------|--------------|
| BC-2.14.013 | v2.0 | Co-emission model; T0855 in multi-tag vec; not standalone |
| BC-2.14.014 | v2.0 | Single finding `["T0855","T0836"]` per PDU |
| BC-2.14.015 | v2.0 | Single finding `["T0855","T0835"]` per PDU |
| BC-2.14.016 | v2.0 | T0831 co-tagged inline on the per-PDU write finding (`["T0855","T0836","T0831"]`); no separate T0831 Finding object |
| BC-2.14.017 | v2.0 | Dual-window burst+sustained; `["T0806","T0855"]` per Decision 11 |
| BC-2.14.020 | v2.0 | T0888 replaces T0846; 0x07 excluded per Decision 12 |
| BC-2.14.022 | v2.0 | Updated per multi-tag model context |
| BC-2.14.024 | v2.0 | Dual CLI flags (`--modbus-write-burst-threshold` + `--modbus-write-sustained-threshold`) |

**Affected existing stories (bc_array_changes_propagate_to_body_and_acs):**

Stories whose scope spans `findings.rs`, `reporter/csv.rs`, `reporter/terminal.rs`, or
`mitre.rs` must update their BC tables, acceptance criteria, and implementation scope to
reflect the mitre_techniques field rename and dual-window model:

| Story | Scope Touch |
|-------|-------------|
| STORY-069 | findings.rs — Finding struct field rename |
| STORY-070 | reporter/csv.rs — column-6 rename + semicolon join |
| STORY-071 | reporter/terminal.rs — multi-ID rendering |
| STORY-078 | mitre.rs — T0888 arm; SEEDED 21; EMITTED 13 |
| STORY-079 | BC-2.14.017 dual-window implementation |
| STORY-080 | BC-2.14.024 dual-flag CLI wiring |

Story-writer must propagate these to story body BC tables and ACs under
`bc_array_changes_propagate_to_body_and_acs` policy.

---

## 6. Unchanged PRD Content

The following sections are NOT modified by this delta (preserved verbatim):
- Section 1.1 Problem Statement
- Section 1.2 Solution Vision
- Section 1.3 Key Differentiators (table — see §6 for BC additions to KD subsections)
- Section 1.4 Target Users
- Sections 2.1–2.13 (all pre-existing capability BCs)
- Sections 3–5 (Interface Definition, NFR, Error Taxonomy — supplements pending separate burst)
- Section 6.1, 6.2, 6.4, 6.6, 6.7 (KD-001, KD-002, KD-004, KD-006, KD-007)
- Section 8 Domain Debt Index
