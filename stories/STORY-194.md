---
document_type: story
level: ops
story_id: STORY-194
title: "S7comm Formal Hardening: VP-048..055 Full Runs + VP-004/007/041 Re-Verification + cargo-mutants"
epic_id: E-23
version: "1.0"
status: ready
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 8
priority: P1
cycle: feature-s7comm
wave: 97
target_module: analyzer/s7comm
subsystems: [SS-05, SS-10, SS-18, SS-20, SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-193]
blocks: []
behavioral_contracts: [BC-2.20.001, BC-2.20.013, BC-2.21.002, BC-2.21.009, BC-2.21.017, BC-2.21.019, BC-2.21.022]
verification_properties: [VP-048, VP-049, VP-050, VP-051, VP-052, VP-053, VP-054, VP-055, VP-004, VP-007, VP-041]
inputs:
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.001.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.013.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.002.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.009.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.017.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.019.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.022.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/verification-properties/VP-INDEX.md
input-hash: "28043d3"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced. This story writes NO new
> production features — it runs formal verification harnesses to green and fixes any
> defects they surface.

# STORY-194: S7comm Formal Hardening

## Narrative

**As a** security engineer responsible for wirerust correctness,
**I want** all S7comm/ISO-on-TCP verification properties executed to green — Kani formal
proofs for the two SS-20 header parsers and the SS-21 bounds-safety function, proptest
exhaustive checks for carry-buffer isolation and classification totality, a combined
cargo-fuzz no-panic harness, re-verification of the amended VP-004/VP-007/VP-041
obligations, and a cargo-mutants sweep —
**so that** the S7comm passive analyzer meets the same formal-hardening bar as the
existing Modbus, DNP3, ENIP, and IEC-104 analyzers before the feature gate opens.

## Behavioral Contracts (anchors — no new BCs; this story re-verifies existing ones)

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.20.001 | `parse_tpkt_header` Returns None for Input Shorter Than 4 Bytes | Anchor for VP-048 full Kani run |
| BC-2.20.013 | TPKT Frames Reassembled via Directional Carry Buffers, Walk-First Semantics | Anchor for VP-050 full proptest run |
| BC-2.21.002 | `S7commAnalyzer::on_data` Four-Way Dispatch on protocol_id | Anchor for VP-053 full non-vacuous run |
| BC-2.21.009 | Declared param_length/data_length Bounds-Checked Before Slice Access | Anchor for VP-051 full Kani run |
| BC-2.21.017 | Unrecognized Job/Ack_Data FC — Totality Anchor | Anchor for VP-052 FC-totality sub-part |
| BC-2.21.019 | Userdata Block Functions (Group 0x03) — Load-Bearing Correction | Anchor for VP-052 Userdata-group sub-part non-vacuity |
| BC-2.21.022 | Userdata Time Functions (Group 0x07) — Corrected | Anchor for VP-052 Userdata-group sub-part non-vacuity |

## Acceptance Criteria

### AC-194-001: VP-048 Kani proof runs to green — parse_tpkt_header safety and four-way totality
(traces to BC-2.20.001 invariant 2)
- Given the `verify_parse_tpkt_header_safety` Kani harness (skeleton from STORY-184)
- When `cargo kani --harness verify_parse_tpkt_header_safety` is run
- Then the proof completes with VERIFICATION SUCCESSFUL: no panics for any symbolic
  `[u8; N]` input; the four outcomes (the three length/version/length-field reject paths
  and the happy-path accept, per STORY-184) are exhaustive and mutually exclusive;
  `h.length` decoding never overflows
- **Test:** `verify_parse_tpkt_header_safety`

### AC-194-002: VP-049 Kani proof runs to green — parse_cotp_header safety, TPDU-type exhaustiveness, protocol-ID totality
- Given the `verify_parse_cotp_header_safety` Kani harness (skeleton from STORY-185)
- When `cargo kani --harness verify_parse_cotp_header_safety` is run
- Then the proof completes with VERIFICATION SUCCESSFUL: no panics or OOB reads for any
  input (including the LI-truncation bounds check); the TPDU-type match is exhaustive
  over all 16 nibble values; the protocol-ID extraction is a total identity mapping over
  all 256 `u8` values
- **Test:** `verify_parse_cotp_header_safety`

### AC-194-003: VP-050 proptest passes — TPKT/COTP carry-buffer reassembly, overflow isolation, 1-byte resync (non-vacuous)
(traces to BC-2.20.013 invariant 1)
- Given the `proptest_vp050_*` skeletons from STORY-186
- When upgraded to asserting harnesses (interleaved direction-tagged-chunk generators,
  per the VP-045/IEC-104 non-vacuity precedent) and `cargo test proptest_vp050` is run
- Then all harnesses pass with meaningful property assertions: walk-first-residual-bound
  equivalence (splitting a byte sequence into carry+incoming yields the identical result
  as running the walk once on the concatenation); `carry_c2s`/`carry_s2c` never mix
  across directions; the residual-bound overflow reaction (clear+resync+one T0814 per
  direction) fires correctly; the resync sub-routine advances exactly 1 byte per
  iteration
- **Non-vacuity requirement:** each proptest body MUST contain at least one
  `prop_assert!`/`prop_assert_eq!` inspecting post-`on_data` state — a body that only
  calls `on_data` without asserting is REJECTED as vacuous
- **Test:** `proptest_vp050_walk_first_residual_bound`,
  `proptest_vp050_direction_isolation`, `proptest_vp050_resync_one_byte_advance`

### AC-194-004: VP-051 Kani proof runs to green — S7comm header bounds-before-slice safety
(traces to BC-2.21.009 postcondition 1)
- Given the `verify_parse_s7comm_header_bounds_safety` Kani harness (skeleton from
  STORY-187)
- When `cargo kani --harness verify_parse_s7comm_header_bounds_safety` is run
- Then the proof completes with VERIFICATION SUCCESSFUL: `parse_s7comm_header` never
  panics or reads OOB for `data.len() < 10`; the caller-side bounds obligation
  (`header_len + param_length + data_length` cannot overflow `usize`, no slice beyond
  `data.len()` is ever constructed) holds for all `u16 × u16` combinations
- **Test:** `verify_parse_s7comm_header_bounds_safety`

### AC-194-005: VP-052 proptest passes — FC and Userdata-group classification totality, non-vacuous 0x03/0x07 assertion
(traces to BC-2.21.017 invariant, BC-2.21.019 postcondition 5, BC-2.21.022
postcondition 2)
- Given `proptest_vp052_fc_classification_totality` (STORY-188) and
  `proptest_vp052_userdata_group_totality` (STORY-189)
- When `cargo test proptest_vp052` is run
- Then both harnesses pass: the Job/Ack_Data FC match is exhaustive over all 256 `u8`
  values plus the empty-parameter-block case; the Userdata function-group match is
  exhaustive over all 16 nibble values
- **Non-vacuity requirement (load-bearing for this feature specifically):** the harness
  MUST explicitly assert group `0x03` classifies as `BlockFunctions` and group `0x07`
  classifies as `TimeFunctions` — a transposed implementation MUST fail this test; a
  mutation swapping the two match arms MUST be killed
- **Test:** `proptest_vp052_fc_classification_totality`,
  `proptest_vp052_userdata_group_totality`

### AC-194-006: VP-053 proptest passes — protocol_id four-way dispatch totality, never-misattribute property (non-vacuous)
(traces to BC-2.21.002 postcondition 5)
- Given `proptest_vp053_protocol_id_dispatch_totality` (started in STORY-187, completed
  in STORY-190)
- When `cargo test proptest_vp053` is run
- Then the harness sweeps all 256 `u8` `protocol_id` values (plus the session-TPDU and
  unparseable-COTP cases) and asserts: for every value not in `{0x32, 0x72}`, the
  resulting flow's `classified_protocol` is `Some(Unclassified)`, never `Some(Classic)`
  or `Some(Plus)`
- A mutation to the `protocol_id` dispatch's equality checks (e.g. `0x32` vs `0x33`)
  MUST be killed by this proptest
- **Test:** `proptest_vp053_protocol_id_dispatch_totality`

### AC-194-007: VP-054 proptest passes — Program-Download/Upload structural disjointness
- Given `proptest_vp054_download_upload_structural_disjointness` (skeleton from
  STORY-188)
- When `cargo test proptest_vp054` is run
- Then the harness confirms no Download-triad value is ever classified as, aliased to,
  or conflated with any Upload-triad value, and vice versa, across the full
  `0x1A..=0x1F` range
- **Test:** `proptest_vp054_download_upload_structural_disjointness`

### AC-194-008: VP-055 cargo-fuzz harness runs for minimum 60 seconds with no crashes
- Given `fuzz_s7comm_parser` cargo-fuzz harness (created in this story, mirrors
  `fuzz_iec104_parser`/`fuzz_enip_parser` precedent)
- When `cargo fuzz run fuzz_s7comm_parser -- -max_total_time=60` is run
- Then no crash, panic, or OOB memory access is reported across the combined
  TPKT->COTP->S7comm parse chain (`parse_tpkt_header` -> `parse_cotp_header` ->
  `parse_s7comm_header` -> FC/Userdata-group classification), exercised as one
  integrated harness rather than per-function unit proofs
- Directional carry buffers remain bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`
  after any input sequence; the frame-walk loop terminates for every input
- **Test:** `fuzz_s7comm_parser` (cargo-fuzz harness)

### AC-194-009: VP-004 Kani re-run passes with S7comm branch — dispatcher oracle
- Given the updated `classify_oracle` in `src/dispatcher.rs` (STORY-193)
- When `cargo kani --harness verify_content_first_precedence_exhaustive` is run
- Then the proof completes with VERIFICATION SUCCESSFUL; the new
  `DispatchTarget::S7comm` arm is covered
- **Test:** `verify_content_first_precedence_exhaustive`

### AC-194-010: VP-007 Kani re-run passes — T0843/T0889/T0821 MITRE catalog integrity
- Given T0843/T0889/T0821 added to `SEEDED_TECHNIQUE_IDS`/`EMITTED_IDS`/`technique_info`
  in `src/mitre.rs` (STORY-191), with `SEEDED_TECHNIQUE_ID_COUNT` at 32
- When `cargo test vp007_catalog_drift_guard` and `cargo test
  verify_all_seeded_ids_resolve` are run
- Then both pass at count=32
- When `cargo kani --harness verify_all_emitted_ids_resolve` is run
- Then T0843/T0889/T0821 all resolve correctly
- **Test:** `vp007_catalog_drift_guard`, `verify_all_seeded_ids_resolve`,
  `verify_all_emitted_ids_resolve`

### AC-194-011: VP-041 proptest re-run passes — Support-enum partition, DetectionOnly retention non-vacuous
- Given the amended `proptest_vp041_oracle_cross_check` and
  `proptest_vp041_partition_invariant` harnesses (STORY-193)
- When `cargo test proptest_vp041` is run
- Then both pass: the oracle cross-check confirms `entry ∈ supported_protocols() ⟺
  entry.support == Support::Supported` for all ~30 `KNOWN_PROTOCOLS` entries; the
  partition invariant holds with S7comm-plus's `DetectionOnly` entry present and
  correctly retained in `unsupported_protocols()`
- A mutation changing `unsupported_protocols()`'s filter from `!= Supported` to
  `== KnownUnsupported` MUST be killed by these harnesses
- **Test:** `proptest_vp041_oracle_cross_check`, `proptest_vp041_partition_invariant`

### AC-194-012: cargo-mutants sweep reports acceptable mutation score for the S7comm module surface
- Given `cargo mutants` is run against `src/analyzer/iso_on_tcp.rs` and
  `src/analyzer/s7comm.rs`
- When the mutation score is computed
- Then all surviving mutants (if any) are triaged and either (a) covered by a new
  targeted test added in this story, or (b) documented as acceptable survivals in
  `cycles/wave-NNN/wave-gate/code-review.md`
- The mutation score for both files must be >= 80% killed
- **Test:** `cargo mutants` run report (serial, per `.cargo/mutants.toml`'s
  `minimum_test_timeout` floor — never with a high `--jobs` value, per
  PG-MUTANTS-JOBS-001)

## Out of Scope

No new production features are implemented in this story beyond targeted bug fixes
discovered by the formal verification harnesses themselves. Any bug found by
Kani/proptest/fuzz/mutants is fixed in a micro-commit before the corresponding proof
re-run — following the STORY-174 (IEC-104 hardening) precedent exactly.

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| VP-048/VP-049 Kani proof execution | SS-20 | `src/analyzer/iso_on_tcp.rs` | `#[cfg(kani)]` |
| VP-050/VP-052/VP-053/VP-054 proptest execution | SS-21 | `tests/s7comm_analyzer_tests.rs` | Test |
| VP-051 Kani proof execution | SS-21 | `src/analyzer/s7comm.rs` | `#[cfg(kani)]` |
| VP-055 fuzz execution | SS-20/SS-21 (combined) | `fuzz/fuzz_targets/fuzz_s7comm_parser.rs` | Fuzz |
| VP-004 Kani re-run | SS-05 | `src/dispatcher.rs` | `#[cfg(kani)]` |
| VP-007 Kani re-run | SS-10 | `src/mitre.rs` | `#[cfg(kani)]` |
| VP-041 proptest re-run | SS-18 | `src/protocols.rs` | Test |
| cargo-mutants sweep | SS-20/SS-21 | `src/analyzer/iso_on_tcp.rs`, `src/analyzer/s7comm.rs` | CI tool |

Subsystem anchor: SS-20/SS-21 own the primary formal-hardening targets per
ARCH-INDEX.md §SS-20/§SS-21. The VP-004/VP-007/VP-041 re-runs touch SS-05/SS-10/SS-18
respectively — these are re-verification runs, not new code.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| All functions under formal verification in this story | pure-core or effectful-shell (unchanged from their originating stories) | This story runs proofs against existing code; it does not reclassify purity boundaries |

## Tasks

- [ ] Run `cargo kani --harness verify_parse_tpkt_header_safety` -> fix until green
      (VP-048)
- [ ] Run `cargo kani --harness verify_parse_cotp_header_safety` -> fix until green
      (VP-049)
- [ ] Upgrade `proptest_vp050_*` skeletons to asserting harnesses with interleaved
      direction-tagged-chunk generators; run `cargo test proptest_vp050` -> fix until
      green (VP-050)
- [ ] Run `cargo kani --harness verify_parse_s7comm_header_bounds_safety` -> fix until
      green (VP-051)
- [ ] Run `cargo test proptest_vp052` (both FC and Userdata-group sub-parts, with the
      explicit non-vacuous group `0x03`/`0x07` assertion) -> fix until green (VP-052)
- [ ] Run `cargo test proptest_vp053` (full 256-value sweep) -> fix until green (VP-053)
- [ ] Run `cargo test proptest_vp054` -> fix until green (VP-054)
- [ ] Create `fuzz/fuzz_targets/fuzz_s7comm_parser.rs` calling `S7commAnalyzer::on_data`
      directly across the combined parse chain; run `cargo fuzz run fuzz_s7comm_parser
      -- -max_total_time=60` -> fix any crashes (VP-055)
- [ ] Run `cargo kani --harness verify_content_first_precedence_exhaustive` -> fix until
      green (VP-004 re-run)
- [ ] Run `cargo test vp007_catalog_drift_guard`, `cargo test
      verify_all_seeded_ids_resolve`, `cargo kani --harness
      verify_all_emitted_ids_resolve` -> fix until green (VP-007 re-run)
- [ ] Run `cargo test proptest_vp041` -> fix until green (VP-041 re-run)
- [ ] Run `cargo mutants` on `src/analyzer/iso_on_tcp.rs` and `src/analyzer/s7comm.rs`
      (serial; never a high `--jobs` value per PG-MUTANTS-JOBS-001) -> triage surviving
      mutants
- [ ] Add targeted tests for any uncovered mutants or document acceptable survivals in
      `cycles/wave-NNN/wave-gate/code-review.md`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` -> fix any
      new lints introduced across the epic's stories
- [ ] Run `python3 bin/check-green-doc-tense` -> exits 0 (no stale Red-Gate prose left in
      `tests/s7comm_analyzer_tests.rs`/`tests/iso_on_tcp_tests.rs` from earlier stories)
- [ ] All `cargo test --all-targets` passes on the fully integrated epic
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` summarizing the completed
      formal-hardening pass, before creating the PR

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | VP-048 | Kani symbolic input with `len=0` | Verifies `None` return, no panic |
| EC-002 | VP-050 | Fuzz/proptest input that fills carry to exactly 65,535 bytes | No T0814; VP-055 harness verifies no OOB |
| EC-003 | VP-007 | Kani re-run before T0843/T0889/T0821 count=32 lands | Proof fails as expected — fix is confirming STORY-191 landed correctly |
| EC-004 | VP-004 | Kani re-run without the S7comm oracle arm | Proof fails as expected — fix is confirming STORY-193's oracle update landed |
| EC-005 | cargo-mutants | A mutation swaps the group `0x03`/`0x07` match arms in `classify_userdata_function` | MUST be killed by VP-052's non-vacuous proptest — if it survives, this is a HIGH-severity gap requiring a new targeted test, not a documented acceptable survival |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~5,500 |
| BC anchor files (7 BCs, summary-level re-read) | ~5,000 |
| ADR-014 (formal verification decisions 8, 9) | ~8,000 |
| src/analyzer/iso_on_tcp.rs + src/analyzer/s7comm.rs (complete, all prior stories) | ~20,000 |
| src/dispatcher.rs, src/protocols.rs, src/mitre.rs (post STORY-191/193) | ~10,000 |
| Test files (complete, all prior stories) | ~12,000 |
| **Total** | **~60,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~30%** (near the top end; if needed, load only specific modules under proof rather than the full accumulated test suite at once) |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-186 through STORY-193 | Full `S7commAnalyzer`/`iso_on_tcp.rs` implementation complete; dispatcher/CLI/catalog wired | Every proptest obligation across the epic was written as a compile-only skeleton in its originating story, deferred to full non-vacuous assertion here | Common issues from the IEC-104/ENIP/DNP3 hardening precedent that likely recur here: Kani harnesses needing `kani::assume` bounds to stay tractable; proptest finding edge cases in the 65,535/254-byte carry-boundary arithmetic; fuzz exposing a panic in the bad-version-byte resync scan if carry management has an off-by-one; cargo-mutants surviving on `>` vs `>=` boundary checks (e.g. the carry-overflow strict-`>` comparison established in STORY-186) if only a single boundary test exists rather than both `==` and `>` cases |

The single highest-risk mutation-survival surface in this epic is the group
`0x03`/`0x07` Userdata classification (STORY-189) and its downstream T0888 emission
consumer (STORY-192) — prioritize triaging any surviving mutant there first.

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 9**: VP-048/VP-049/VP-051 scope is strictly the named pure-core
  functions (`parse_tpkt_header`, `parse_cotp_header`, the bounds-check component of
  `parse_s7comm_header`) — do NOT expand Kani harness scope to cover `on_data`'s full
  loop; that is VP-050/VP-055's territory (proptest/fuzz), mirroring VP-044's IEC-104
  precedent exactly (a Kani harness over the full loop will time out).
- No new production code changes are expected in this story. If a bug is found, fix it
  in a micro-commit before the proof re-run, per the STORY-174 precedent.
- Mutation testing MUST run serially (bare `cargo mutants` or explicit `--jobs 1`) per
  `CLAUDE.md`'s PG-MUTANTS-JOBS-001 incident — never pass a high `--jobs` value.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| kani | Latest | VP-048, VP-049, VP-051, VP-004, VP-007 formal proof execution |
| proptest | 1 (pinned in `Cargo.toml`) | VP-050, VP-052, VP-053, VP-054, VP-041 proptest execution |
| cargo-fuzz | Latest | VP-055 fuzz execution |
| cargo-mutants | Latest | Mutation testing sweep (serial only) |

All libraries are already in the dev-dependencies from prior epics.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iso_on_tcp.rs` | MODIFY (if fixes needed) | Targeted bug fixes from proof/fuzz findings; no new features |
| `src/analyzer/s7comm.rs` | MODIFY (if fixes needed) | Targeted bug fixes from proof/fuzz findings; no new features |
| `tests/iso_on_tcp_tests.rs`, `tests/s7comm_analyzer_tests.rs` | MODIFY | Upgrade all proptest skeletons to non-vacuous asserting harnesses; add targeted tests for any surviving mutants |
| `fuzz/fuzz_targets/fuzz_s7comm_parser.rs` | CREATE | VP-055 combined-chain fuzz harness |
| `CHANGELOG.md` | MODIFY | `[Unreleased]` entry for the completed formal-hardening pass |
| `cycles/wave-NNN/wave-gate/code-review.md` | CREATE | Document surviving mutants (if any) and disposition |

## Forbidden Dependencies

- Do NOT expand VP-048/VP-049/VP-051 Kani harness scope to cover `on_data`'s full loop —
  it will time out; VP-050/VP-055 cover the loop
- Do NOT pass a high `--jobs` value to `cargo mutants` under any circumstance
  (PG-MUTANTS-JOBS-001)
- Do NOT implement any new production feature in this story beyond fixes surfaced by the
  formal verification harnesses themselves

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — full formal-hardening pass: VP-048 through VP-055 executed to green, VP-004/VP-007/VP-041 re-verification, cargo-mutants sweep, AC-194-001..012. |
