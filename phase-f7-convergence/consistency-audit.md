---
document_type: consistency-audit
audit_scope: feature-iec104
gate: F7
auditor: consistency-validator
develop_sha: b36b884
factory_artifacts_sha: a67a3c4
date: 2026-07-16
---

# F7 Delta-Convergence Fresh-Context Consistency Audit — feature-iec104

**Scope**: STORY-167, STORY-168, STORY-169, STORY-170, STORY-171, STORY-172 (IEC-104
delta stories, all status DELIVERED); pre-existing drift STORY-164/STORY-165 flagged
only.

**Mandate**: READ-ONLY adjudication per DF-CONSISTENCY-AUDIT-POST-FIXBURST-001.
No files were modified or hashes rewritten.

---

## Mandate A — Input-Hash Drift Adjudication

### Method

Staleness cause traced via `git log --follow` on the factory-artifacts branch. Each
changed input file's `modified:` frontmatter was read to characterize the nature of
each post-delivery BC update. The characterization was cross-referenced against
`src/analyzer/iec104.rs` to verify that the delivered code either (a) already
implements the new behavior, or (b) is unaffected by the documentation change.

### Verdict Table

| Story | Status | Re-baseline safe? | Classification | Staleness cause |
|-------|--------|-------------------|----------------|-----------------|
| STORY-167 | STALE | YES | BENIGN-REBASELINE | BC-2.19.002 v1.2 carry-not-cleared docfix; BC-2.19.004 v1.1 VP attribution docfix (VP-044→VP-047); BC-2.19.006 v1.2 not-a-production-gate constraint (already correctly implemented); shared arch files updated during STORY-172 remediation passes |
| STORY-168 | STALE | YES | BENIGN-REBASELINE | BC-2.19.010 v1.1 STARTDT-con clarification (code implements `U_STARTDT_CON` path); BC-2.19.011 v1.1 STOPDT-con no-T0881 clarification (code implements `U_STOPDT_CON => session_started=false; None`); shared arch files |
| STORY-169 | STALE | YES | BENIGN-REBASELINE | BC-2.19.015 v1.1 VP-044 over-scope docfix (parse_asdu VP-047 attribution correction — behavior unchanged); BC-2.19.016 v1.1 TypeID upper-bound clarity (does not affect parse_asdu scope); shared arch files |
| STORY-170 | STALE | YES | BENIGN-REBASELINE | BC-2.19.019 v1.1 title-only fix "Set-Point + Bitstring TypeIDs 48–51"; BC-2.19.020 v1.1 T0827 confidence Possible→Likely (code emits Likely; story v2.0 incorporated before delivery); shared arch files |
| STORY-171 | STALE | YES | BENIGN-REBASELINE | BC-2.19.023 v1.2 Option<u16> postcondition update (code uses Option<u16>; story incorporates Option<u16> first-frame guard); BC-2.19.024 v1.3 prose precision, arithmetic unchanged; shared arch files |
| STORY-172 | STALE | YES | BENIGN-REBASELINE | BC-2.19.025 v1.3 WALK-FIRST-RESIDUAL-BOUND entry-check prose precision (story v3.1 and code at iec104.rs:1189–1222 implement it); BC-2.19.026 v1.6 EMIT-WITH-DEDUP (story v2.0 and code at iec104.rs:1262–1291 implement per-direction flags); BC-2.19.027 v1.1 FlowId→FlowKey rename (code uses FlowKey); BC-2.19.006 v1.2 (not-a-production-gate update post-STORY-173 — code correctly implements); shared arch files |

**Pre-existing drift (flagged, out of scope for F7 gate):**

| Story | Note |
|-------|------|
| STORY-164 | STALE; pre-dates feature-iec104 delta; out of scope — flag for separate rebaseline pass |
| STORY-165 | STALE; pre-dates feature-iec104 delta; out of scope — flag for separate rebaseline pass |

**Overall re-baseline safe? YES** — all 6 delta stories exhibit benign post-delivery spec
cleanup. No story has a STALE hash caused by a new behavioral requirement that was not
incorporated into the delivered code.

---

## Mandate B — Delta Consistency Audit (6 Dimensions)

### Dimension 1 — Source-Line Anchor Citations vs Current Tree

All BC Architecture Anchors for SS-19 BCs cite functions and constants that exist in
`src/analyzer/iec104.rs` at the expected locations:

- BC-2.19.001–005: `fn parse_apci_header(data: &[u8]) -> Option<ApciHeader>` — present
  at iec104.rs:461
- BC-2.19.006: `fn is_valid_iec104_frame(data: &[u8]) -> bool` — present; doc-comment
  and Invariant-3 correctly state NOT called in production on_data frame-walk path
- BC-2.19.007–014: `fn classify_frame_format`, `fn process_u_frame`, session state
  fields — all present
- BC-2.19.015–018: `fn parse_asdu` — present at iec104.rs:645
- BC-2.19.019–022: `fn detect_iec104_threats` — present at iec104.rs:730
- BC-2.19.023–024: `fn extract_ns`, `fn extract_nr`, `fn track_ns_desync` with
  `Option<u16>` — all present (iec104.rs:946, 963, 1001)
- BC-2.19.025–027: `Iec104Analyzer::on_data` carry check at entry (iec104.rs:1189–1222),
  EMIT-WITH-DEDUP per-direction flags in `Iec104FlowState`, `on_flow_close` — all
  present
- `MAX_IEC104_CARRY_BYTES = 255` (iec104.rs:170) and `MAX_IEC104_FINDINGS = 10_000`
  (iec104.rs:184) — both match BC-cited constants

**STATUS: CLEAN** with one MINOR inconsistency noted (see Finding B-001 below).

### Dimension 2 — BC H1/Title Propagation (BC-INDEX, VP-INDEX, ARCH-INDEX, PRD RTM)

- BC-INDEX: 28 SS-19 entries (BC-2.19.001..028) — confirmed; BC-2.19.006 title updated
  to "Standalone Pure Frame-Validity Predicate" in index
- VP-INDEX: total=47 (kani=16, proptest=22, fuzz=3, integration_unit=6); VP-044/045/046/047
  registered for SS-19 — confirmed
- ARCH-INDEX §SS-19: VP-044..VP-047 registered; bad-start-byte silenced (no finding)
  confirmed — consistent with code
- PRD §2.19.A RTM: BC-2.19.006 row still reads "Post-Classification Validity Gate" —
  INCONSISTENT with BC-2.19.006 v1.2 new title and BC-INDEX update

**Finding B-001 (MINOR, Dimension 2):** PRD §2.19.A RTM row for BC-2.19.006 at
`.factory/specs/prd.md:2276` retains the old title "Post-Classification Validity Gate".
BC-2.19.006 was retitled to "Standalone Pure Frame-Validity Predicate" in v1.2 (commit
`6536bda`, post-STORY-173). BC-INDEX was updated; PRD RTM row was not. No code impact.
Not F7-blocking.

### Dimension 3 — VP Prose vs Harness Semantics

VP-044 (Kani): harness `verify_parse_apci_header_safety` at iec104.rs:1476 covers all
5 BC-2.19.001–005 facets (89 checks) — consistent with VP-044 registry entry.

VP-045 (proptest carry isolation): harnesses `proptest_vp045_direction_isolation`
(tests/iec104_analyzer_tests.rs:5383) and `proptest_vp045_independent_run_equivalence`
(tests/iec104_analyzer_tests.rs:5485) — both present; test strategy matches the
directional carry isolation property specified in VP-045 (RULING-DNP3-SIBLING-001 pattern).

VP-046 (proptest frame format totality): harness `proptest_vp046_frame_format_totality`
(tests/iec104_analyzer_tests.rs:1534) — present; exercises all 256 CF1 values confirming
totality of `classify_frame_format` — consistent with VP-046 registry entry.

VP-047 (cargo-fuzz no-panic): `fuzz/fuzz_targets/fuzz_iec104_parser.rs` — file exists;
covers `parse_apci_header`, `is_valid_iec104_frame`, `parse_asdu`, and `on_data` entry
point per ADR-013 Decision 8 — consistent with VP-047 registry entry.

**STATUS: CLEAN**

### Dimension 4 — Story Architecture-Mapping Anchors vs BC Anchor Citations

All 6 delta stories carry Architecture-Mapping tables that cite component names and
BC identifiers consistent with the BC Architecture Anchor sections. Spot-checked:

- STORY-172 Architecture-Mapping: cites `on_data` WALK-FIRST-RESIDUAL-BOUND,
  `on_flow_close`, `Iec104FlowState` — all match BC-2.19.025/026/027 Architecture Anchors
  and delivered code
- STORY-171 Architecture-Mapping: cites `Option<u16>` first-frame guard for N(S)/N(R) —
  matches BC-2.19.023/024 and delivered code at iec104.rs:1001
- STORY-167 Architecture-Mapping: cites `parse_apci_header` pure free function — matches
  BC-2.19.001–005 and delivered code at iec104.rs:461

**STATUS: CLEAN** with one MINOR noted (see Finding B-002 below).

### Dimension 5 — Input-Hash Drift

Covered by Mandate A above. All 6 delta stories: BENIGN-REBASELINE.

**STATUS: CLEAN** (per Mandate A adjudication)

### Dimension 6 — Index Row Counts vs Actual File Counts

- BC-INDEX SS-19 rows: 28 — `ls .factory/specs/behavioral-contracts/ss-19/BC-2.19.*.md`
  returns 28 files — MATCH
- VP-INDEX total: 47 — VP count by VP-INDEX claimed totals (kani=16, proptest=22,
  fuzz=3, integration_unit=6) — consistent with registered entries
- STORY-INDEX data rows: 127 — `ls .factory/stories/STORY-*.md` (excluding STORY-INDEX.md
  itself) returns 127 files — MATCH

**STATUS: CLEAN**

---

## Additional MINOR Finding

**Finding B-002 (MINOR, Dimension 1/Dimension 4):** BC-2.19.002 (`.factory/specs/
behavioral-contracts/ss-19/BC-2.19.002.md`) PC-2 and Architecture Anchor section retain
the text "frame-walk caller emits T0814 for bad start byte." This is inconsistent with:
(a) BC-2.19.026 PC-4 (the authoritative frame-walk BC), which explicitly specifies
"bad start byte → advance 1 byte (resync scan to next 0x68 candidate, carry NOT cleared)"
with no T0814 emission; (b) STORY-172 AC-172-004 "NO finding emitted for bad-start-byte
per BC-2.19.026 and ADR-013 Decision 3"; (c) delivered code at iec104.rs:1238 which
advances pos by 1 with no finding. BC-2.19.002 PC-2 was not cleaned up when BC-2.19.026
was reconciled as the authoritative frame-walk BC. The delivered code is CORRECT per
BC-2.19.026; this is a documentation inconsistency only. Not F7-blocking.

---

## Summary

### Mandate A — Per-Story Verdict

| Story | Verdict | Re-baseline safe? |
|-------|---------|-------------------|
| STORY-167 | BENIGN-REBASELINE | YES |
| STORY-168 | BENIGN-REBASELINE | YES |
| STORY-169 | BENIGN-REBASELINE | YES |
| STORY-170 | BENIGN-REBASELINE | YES |
| STORY-171 | BENIGN-REBASELINE | YES |
| STORY-172 | BENIGN-REBASELINE | YES |

**Overall re-baseline safe: YES**

Pre-existing out-of-scope: STORY-164, STORY-165 (flagged; separate rebaseline pass
required before those stories' gates, not F7-blocking).

### Mandate B — Findings

| ID | Severity | Dimension | Location | Description |
|----|----------|-----------|----------|-------------|
| B-001 | MINOR | 2 (BC H1/title propagation) | `.factory/specs/prd.md:2276` | PRD §2.19.A RTM row for BC-2.19.006 retains old title "Post-Classification Validity Gate"; BC-INDEX updated, PRD not. No code impact. |
| B-002 | MINOR | 1/4 (source-line anchors / story architecture mapping) | `.factory/specs/behavioral-contracts/ss-19/BC-2.19.002.md` PC-2 + Architecture Anchor | Stale claim that frame-walk caller emits T0814 for bad start byte; superseded by BC-2.19.026 PC-4, STORY-172 AC-172-004, and delivered code. Code is correct per BC-2.19.026. |

F7-BLOCKING findings: **0**

Dimensions 3, 5, 6 are fully CLEAN. Dimensions 1, 2, 4 have MINOR doc inconsistencies
only — no code corrections required before F7 gate.

---

F7 CONSISTENCY: FINDINGS (2 MINOR, 0 F7-BLOCKING)
