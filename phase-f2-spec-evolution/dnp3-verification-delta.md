---
document_type: verification-delta
feature_id: issue-008-dnp3-analyzer
github_issue: 8
title: "F2 Verification Delta — DNP3 TCP Analyzer (SS-15)"
status: draft
producer: architect
created: 2026-06-10
base_commit: fb2c875
branch: develop
traces_to:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/research/dnp3-research.md
  - .factory/phase-f1-delta-analysis/dnp3-delta-analysis.md
---

# F2 Verification Delta — Issue #8: DNP3 TCP Protocol Analyzer

This document is the verification delta for Feature #8. It records the one new
verification property (VP-023), confirms no existing VP (VP-001..022) requires
modification by this feature, and flags the verification *dependencies* that must be
honored in F4 to keep already-locked proofs green.

The architect has already pre-registered VP-023 in the three index files (counts bumped:
Kani 9→10, total 22→23, P1 8→9). This delta does not re-touch those indexes; it
documents the VP-023 spec rationale, the regression surface, and the VP-007 obligation.

---

## 1. New Verification Property: VP-023

**File:** `.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md`

**Title:** DNP3 Data-Link Frame Parse Safety and Function-Code Classification

**Status:** draft (harnesses authored in F4 TDD)

**Tool:** Kani

**Phase:** P1 (consistent with VP-022 Modbus — new code, no legacy debt)

**Module:** `src/analyzer/dnp3.rs`

### VP-023 Sub-property Overview

| Sub-property | Target function | Property | BC anchors |
|--------------|----------------|----------|------------|
| A — DL header parse safety | `parse_dnp3_dl_header` | Never panics; `None` iff `len<10`; LE field decode correct | BC-2.15.001, BC-2.15.002, BC-2.15.003 |
| C — Validity gate biconditional | `is_valid_dnp3_frame_header` | `true` iff sync==0x0564 and LENGTH>=5 | BC-2.15.004 |
| B — FC classification totality + correctness | `classify_dnp3_fc` | Total over all 256 FC values; Control/Restart/Write/Read sets correct | BC-2.15.005, BC-2.15.006 |
| D — frame_len arithmetic | `compute_dnp3_frame_len` | None for len<5; formula correct; result in [10,292]; no overflow/panic | BC-2.15.007 |

**Note on Sub-property D:** This sub-property has no VP-022 Modbus equivalent. DNP3's
interleaved-CRC block structure means the frame-consumption loop depends on an arithmetic
formula (`frame_len = 5 + LENGTH + 2 * ceil((LENGTH-5)/16)`) that must be provably
correct. A Kani proof over the full 256-value `u8` LENGTH domain is a canonical
strength: straight-line integer arithmetic, 256 inputs, runs in milliseconds.

### VP-023 Feasibility Rationale

All four sub-properties operate on small bounded inputs (byte slices ≤12 bytes, single
`u8`). Kani handles these with no bound explosion and no loop-unwind annotation
(Sub-A has no loop; Sub-B/C/D have no loops). VP-022 ran 4/4 SUCCESSFUL under cargo-kani
0.67.0 on analogous functions (`parse_mbap_header`, `is_valid_modbus_adu`, `classify_fc`)
with similar bounded domains. Sub-D's arithmetic proof is structurally simpler than
Sub-B's set-membership proof.

---

## 2. Existing VPs: No Modification Required (with two exceptions)

VP-001 through VP-022 are **verified and locked**. This feature adds VP-023 only; it does
not change any locked VP's property statement.

**Two F4 harness-update obligations exist** for already-locked VPs:

### 2.1 VP-004 (Content-First Dispatch Precedence) — Kani oracle update REQUIRED in F4

VP-004 (`verify_content_first_precedence_exhaustive`) asserts that `classify_oracle()`
matches `classify()` for all port combinations. Adding `DispatchTarget::Dnp3` to
`classify()` (Rule 6, port 20000) REQUIRES an identical update to `classify_oracle()` in
the same implementation story.

This is an **F4 implementation obligation**, not an F2 spec obligation. The oracle must
gain a port-20000 → Dnp3 arm immediately after the port-502 → Modbus arm. VP-004 will
fail at F6 if the oracle is not updated in lockstep with production. This is the same
critical obligation identified in the Modbus cycle (D-032/D-044).

**Regression test invariant:** `verify_tls_signature_beats_port` (which uses an HTTP
fallback port as an adversarial test) should be supplemented with a port-20000 variant
that confirms TLS content (Rule 1) beats Rule 6 (port-20000/Dnp3). This is a new test
in the F4 dispatcher story.

### 2.2 VP-007 (MITRE Technique ID Format and Catalog Completeness) — catalog update REQUIRED in F4

VP-007 (`vp007_catalog_drift_guard`) mechanically sweeps ~10 million technique-ID
candidates and asserts that the catalog count matches `SEEDED_TECHNIQUE_IDS.len()`. Adding
T1691.001 and T0827 to `technique_info()` without updating `SEEDED_TECHNIQUE_IDS` and
`SEEDED_TECHNIQUE_ID_COUNT` causes an **immediate test failure** with an explicit error
message.

This is an **F4 implementation obligation** in the `mitre.rs` story. The full 5-part
atomic update is specified in:
- ADR-007 Decision 5 (§"VP-007 atomic update obligation")
- dnp3-architecture-delta.md §9

**Summary of the update:**
1. Add `"T1691.001"` arm and `"T0827"` arm to `technique_info()`
2. Add `"T1691.001"` and `"T0827"` to `SEEDED_TECHNIQUE_IDS`
3. Bump `SEEDED_TECHNIQUE_ID_COUNT` from 21 → 23
4. Add `"T1691.001"` and `"T0827"` to `EMITTED_IDS` in `kani_proofs`
5. `cargo test mitre` passes before PR merge

**New MitreTactic variant obligation (F4 story, mitre.rs):**
`IcsImpact` must be added to the `MitreTactic` enum, with a `fmt::Display` arm returning
`"Impact"` and appended to `all_tactics_in_report_order()`. Any `match` on `MitreTactic`
in the codebase must be checked for exhaustiveness after this addition.

---

## 3. VP-023 → BC Anchor Map

The product-owner uses this table to write the SS-15 BCs. BC IDs are assigned
sequentially starting at BC-2.15.001.

| BC ID | Concept | VP-023 Sub-property | Anchors (VP-023 proof) |
|-------|---------|-------------------|----------------------|
| BC-2.15.001 | DNP3 DL header accepted for ≥10-byte frame | Sub-A | YES — parse safety, `Some` path |
| BC-2.15.002 | DNP3 DL header rejected for <10-byte frame (truncation) | Sub-A | YES — `None` path |
| BC-2.15.003 | DEST/SOURCE decoded little-endian from offsets 4–7 | Sub-A | YES — field decode |
| BC-2.15.004 | Three-point validity gate: true iff sync==0x0564 and LENGTH>=5 | Sub-C | YES — gate biconditional |
| BC-2.15.005 | `classify_dnp3_fc` total over all 256 FC values | Sub-B | YES — totality |
| BC-2.15.006 | FC classification correctness: Control set, Restart set, Write/Read | Sub-B | YES — set membership |
| BC-2.15.007 | `compute_dnp3_frame_len` arithmetic correct; result in [10,292] | Sub-D | YES — arithmetic |
| BC-2.15.008 + | Detection BCs (unauthorized control, restart DoS, write detection, T0827 derived, broadcast anomaly, ...) | None (test-level) | NO — test coverage only |

**Product-owner note:** BCs anchored to VP-023 (001–007) describe the pure-core parse
behavior. BCs for detection logic (BC-2.15.008 and above) are test-sufficient behavioral
contracts verified by unit/integration tests, not by Kani. Estimate 18–25 total BCs for
SS-15 (same range as Modbus SS-14).

---

## 4. VP-023 Harness Architecture

The Kani harnesses live in `src/analyzer/dnp3.rs` under
`#[cfg(kani)] mod kani_proofs { use super::*; }`, mirroring the VP-022 convention in
`src/analyzer/modbus.rs`.

**Symbolic input strategy per sub-property:**

| Sub-property | Symbolic input | Bound strategy | Proof note |
|--------------|---------------|----------------|-----------|
| A (parse) | `[u8; 12]` + symbolic `len <= 12` | `kani::assume(len <= 12)` | MAX_LEN=12: covers reject-band (0..=9), minimum accept (10), short-extension (11..=12) |
| C (gate) | Symbolic `Dnp3DlHeader` struct fields | None (straight-line) | No indexing; pure boolean over struct fields |
| B (totality) | `fc: u8 = kani::any()` | None (256 values, straight-line match) | No `unreachable!`; wildcard arm proves totality |
| D (frame_len) | `length: u8 = kani::any()` | None (256 values, straight-line arithmetic) | Max result 292 fits in any `usize` |

No `#[kani::unwind(N)]` annotations required (no user-visible loops in any of the four
functions). Estimated total Kani runtime: < 1 second per harness.

---

## 5. Regression Surface

### 5.1 Tests That MUST Stay Green (existing, regression baseline)

- All 1338+ existing tests at develop HEAD fb2c875 (v0.4.0 baseline)
- `tests/dispatcher_tests.rs`: VP-004 tests (TLS-beats-port, HTTP content detection,
  port-502/Modbus Rule 5) must stay green after Rule 6 (port 20000) is added
- `tests/mitre_tests.rs` / `vp007_catalog_drift_guard`: mechanically fails if T1691.001
  or T0827 is added to `technique_info` without updating `SEEDED_TECHNIQUE_IDS` +
  SEEDED_TECHNIQUE_ID_COUNT + EMITTED_IDS

### 5.2 New Tests Required (F4 stories)

- Port-20000 dispatch tests: verify `DispatchTarget::Dnp3` returned for port-20000 flows
- TLS-beats-DNP3-port test: a TLS ClientHello on port 20000 must route to `Tls`, not `Dnp3`
- DNP3 frame parse unit tests: valid/invalid/truncated frames
- DNP3 FC classification tests: spot-check each `Dnp3FcClass` variant
- DNP3 frame_len arithmetic tests: MIN/MAX/edge cases (LENGTH=5→10, LENGTH=255→292)
- DNP3 end-to-end PCAP acceptance test: crafted DNP3 PCAP fixture with known findings

### 5.3 Pre-registered Harnesses (to be authored in F4)

```
verify_parse_dnp3_dl_header_safety   — Sub-property A
verify_is_valid_dnp3_frame_gate      — Sub-property C
verify_classify_dnp3_fc_total        — Sub-property B
verify_compute_dnp3_frame_len        — Sub-property D
```

---

## 6. F2 Open Questions for F3 / Product-Owner

These items are confirmed as [UNVERIFIED] or [JUDGMENT] in dnp3-research.md and must be
resolved before the relevant BCs can be written:

| # | Item | Source | Impact |
|---|------|--------|--------|
| OQ-1 | Broadcast confirm-semantics: exact 0xFFFD vs 0xFFFE per-address behavior | dnp3-research.md §4 [UNVERIFIED] | BC-2.15.NNN (broadcast anomaly BC) |
| OQ-2 | Self-address range: exact value(s) of self-address (0xFFFC?) | dnp3-research.md §4 [UNVERIFIED] | BC for self-address detection (if scoped) |
| OQ-3 | Reserved address lower bound: 0xFFF0? | dnp3-research.md §4 [UNVERIFIED] | BC for malformed-address detection |
| OQ-4 | Default value for `--dnp3-direct-operate-threshold` | dnp3-research.md §5.1 [JUDGMENT] | BC-2.15.NNN (threshold BC) + CLI spec |
| OQ-5 | T1691.001 request-without-response timeout window and correlation key | dnp3-research.md §5.2 [JUDGMENT] | BC-2.15.NNN (block-command inference BC) |
| OQ-6 | T0827 emission guard: N events / window required for derived impact finding | ADR-007 §Open Items [JUDGMENT] | BC-2.15.NNN (T0827 derived finding BC) |

**OQ-1 through OQ-3 require external research (IEEE 1815-2012 primary text) before those
BCs can be written.** Per project policy DF-VALIDATION-001, these items must be validated
by research-agent before BCs are written. The product-owner should invoke the research
skill to resolve them.

---

## 7. Consistency Invariants (post-F2 state)

After this F2 burst, the following arithmetic must hold:

| Invariant | Expected value |
|-----------|---------------|
| VP-INDEX total_vps | 23 |
| VP-INDEX p1_count | 9 |
| VP-INDEX kani_count | 10 |
| verification-architecture.md Should Prove table row count | 8 (VP-010..015, VP-022, VP-023) |
| verification-coverage-matrix.md Totals: Kani | 10 |
| verification-coverage-matrix.md Totals: grand total | 23 |
| SEEDED_TECHNIQUE_ID_COUNT (post-F4 mitre.rs update) | 23 |
| EMITTED_IDS length (post-F4 mitre.rs update) | 15 |
