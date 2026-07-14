---
document_type: story
story_id: STORY-174
title: "IEC-104 Formal Hardening: VP-044 Kani + VP-045/046 Proptest + VP-047 Fuzz + VP-004/007 Re-run + cargo-mutants"
epic_id: E-22
wave: 83
points: 5
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-iec104
subsystems: [SS-19]
target_module: analyzer/iec104
depends_on: [STORY-173]
blocks: []
behavioral_contracts:
  - BC-2.19.006
  - BC-2.19.009
  - BC-2.19.025
  - BC-2.19.026
  - BC-2.05.012
  - BC-2.10.010
verification_properties:
  - VP-044
  - VP-045
  - VP-046
  - VP-047
  - VP-004
  - VP-007
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.006.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.009.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.025.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.026.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.012.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.010.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-delta-analysis.md
input-hash: "de9d14e"
---

# STORY-174: IEC-104 Formal Hardening: VP-044 Kani + VP-045/046 Proptest + VP-047 Fuzz + VP-004/007 Re-run + cargo-mutants

## Narrative

**As a** security engineer responsible for wirerust correctness,
**I want** all IEC-104 verification properties executed to green — Kani formal proofs,
proptest exhaustive checks, cargo-fuzz no-panic harness, and cargo-mutants mutation sweep —
**so that** the IEC-104 passive analyzer meets the same formal-hardening bar as the existing
DNP3, ENIP, and TLS analyzers before the feature gate is opened.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.006 | `is_valid_iec104_frame` / `parse_apci_header` purity | Anchor for VP-044 Kani target |
| BC-2.19.009 | `classify_frame_format` totality over all 256 u8 CF1 values | Anchor for VP-046 proptest |
| BC-2.19.025 | Directional carry buffers bounded at 255; isolation invariant | Anchor for VP-045 proptest |
| BC-2.19.026 | `on_data` no-panic; loop termination | Anchor for VP-047 fuzz target |
| BC-2.05.012 | `DispatchTarget::Iec104` Rule 8 + VP-004 oracle | Anchor for VP-004 Kani re-run |
| BC-2.10.010 | T0881 six-part atomic; SEEDED count 29 | Anchor for VP-007 Kani re-run |

## Acceptance Criteria

### AC-174-001: VP-044 Kani proof runs to green — parse_apci_header safety
**Traces to:** BC-2.19.006 invariant 2 (purity) + BC-2.19.005 postcondition 5 (accept path)
- Given the `verify_parse_apci_header_safety` Kani harness (skeleton from STORY-167)
- When `cargo kani --harness verify_parse_apci_header_safety` is run
- Then the proof completes with VERIFICATION SUCCESSFUL
- Properties verified: no panics for any symbolic `[u8; N]` input (N ≤ 300); all five facets
  (len<6→None, start≠0x68→None, LEN<4→None, LEN>253→None, valid→Some) are correct
- ADR-013 Decision 8 scope: `parse_apci_header` only; `on_data` no-panic is VP-047

### AC-174-002: VP-045 proptest passes — carry direction isolation
**Traces to:** BC-2.19.025 invariant 1 (directional isolation)
- Given `proptest_vp045_direction_isolation` and `proptest_vp045_independent_run_equivalence`
  (skeletons from STORY-172)
- When `cargo test proptest_vp045` is run
- Then both harnesses pass all proptest cases
- Properties verified: interleaved C2S/S2C deliveries never mix carry buffers; independent
  same-direction runs produce equivalent frame_count; each carry bounded at 255

### AC-174-003: VP-046 proptest passes — classify_frame_format totality
**Traces to:** BC-2.19.009 invariant 1 (totality over all 256 CF1 u8 values)
- Given `proptest_vp046_frame_format_totality` (skeleton from STORY-168)
- When `cargo test proptest_vp046` is run
- Then the harness passes all 256 u8 CF1 values (exhaustive sweep)
- Properties verified: bit0=0→IFormat; bits1:0=0b01→SFormat; bits1:0=0b11→UFormat;
  no CF1 value is unhandled; no panic for any u8

### AC-174-004: VP-047 fuzz harness runs for minimum 60 seconds with no crashes
**Traces to:** BC-2.19.026 postcondition 5 (on_data no-panic)
- Given `fuzz_iec104_parser` cargo-fuzz harness (skeleton from STORY-172)
- When `cargo fuzz run fuzz_iec104_parser -- -max_total_time=60` is run
- Then no crash, panic, or OOB memory access is reported
- Properties verified: `on_data` returns for all arbitrary byte sequences; loop terminates;
  carry buffers remain bounded at 255

### AC-174-005: VP-004 Kani re-run passes with Iec104 branch — dispatcher oracle
**Traces to:** BC-2.05.012 invariant 1 (VP-004 oracle completeness after Iec104 addition)
- Given the updated `classify_oracle` in `src/dispatcher.rs` (updated in STORY-173)
- When `cargo kani --harness verify_dispatcher_oracle` (or equivalent VP-004 harness) is run
- Then the proof completes with VERIFICATION SUCCESSFUL
- The new `DispatchTarget::Iec104` arm is covered; ADR-013 Decision 9 obligation satisfied

### AC-174-006: VP-007 Kani re-run passes — T0881 MITRE catalog integrity
**Traces to:** BC-2.10.010 postcondition 6 (verify_all_emitted_ids_resolve for T0881)
- Given T0881 added to `EMITTED_IDS` and `technique_info` in `src/mitre.rs` (STORY-173)
- When `cargo kani --harness vp007_catalog_drift_guard` is run
- Then the proof completes with VERIFICATION SUCCESSFUL at SEEDED count=29
- When `cargo kani --harness verify_all_emitted_ids_resolve` is run
- Then T0881 resolves correctly (technique_info returns Some for T0881)

### AC-174-007: cargo-mutants sweep reports acceptable mutation score
**Traces to:** BC-2.19.026 invariant 1 (loop termination — structural correctness)
- Given `cargo mutants -- --package wirerust` is run against the IEC-104 module
- When the mutation score is computed for `src/analyzer/iec104.rs`
- Then all surviving mutants (if any) are triaged and either:
  a) Covered by a new targeted test added in this story, OR
  b) Documented as acceptable survivals (e.g., unreachable arms) in `cycles/wave-NN/wave-gate/code-review.md`
- The mutation score for `src/analyzer/iec104.rs` must be ≥ 80% killed

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| VP-044 Kani proof execution | SS-19 | `src/analyzer/iec104.rs` | `#[cfg(kani)]` |
| VP-045 proptest execution | SS-19 | `tests/iec104_analyzer_tests.rs` | Test |
| VP-046 proptest execution | SS-19 | `tests/iec104_analyzer_tests.rs` | Test |
| VP-047 fuzz execution | SS-19 | `fuzz/fuzz_targets/fuzz_iec104_parser.rs` | Fuzz |
| VP-004 Kani re-run | SS-05 | `src/dispatcher.rs` | `#[cfg(kani)]` |
| VP-007 Kani re-run | SS-10 | `src/mitre.rs` | `#[cfg(kani)]` |
| cargo-mutants sweep | SS-19 | `src/analyzer/iec104.rs` | CI tool |

Subsystem anchor: SS-19 owns this story's scope because the formal hardening targets are
primarily the IEC-104 analyzer's pure-core functions per ARCH-INDEX.md §SS-19. The VP-004
and VP-007 re-runs touch SS-05 and SS-10 respectively — these are re-verification runs, not
new code.

## Tasks

- [ ] Run `cargo kani --harness verify_parse_apci_header_safety` → fix until green (VP-044)
- [ ] Run `cargo test proptest_vp045` → fix until green (VP-045 carry isolation)
- [ ] Run `cargo test proptest_vp046` → fix until green (VP-046 frame totality)
- [ ] Run `cargo fuzz run fuzz_iec104_parser -- -max_total_time=60` → fix any crashes (VP-047)
- [ ] Run `cargo kani --harness verify_dispatcher_oracle` → fix until green (VP-004 re-run)
- [ ] Run `cargo kani --harness vp007_catalog_drift_guard` → fix until green (VP-007 T0881)
- [ ] Run `cargo kani --harness verify_all_emitted_ids_resolve` → fix until green (VP-007 EMITTED)
- [ ] Run `cargo mutants` on `src/analyzer/iec104.rs` → triage surviving mutants
- [ ] Add targeted tests for any uncovered mutants or document acceptable survivals
- [ ] All `cargo test --all-targets` passes on the integrated codebase
- [ ] Update wave-gate `code-review.md` with any surviving mutants and disposition

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.006 | Kani symbolic input with len=0 | VP-044 verifies None return, no panic |
| EC-002 | BC-2.19.009 | CF1=0x02 (bits1:0=0b10) in VP-046 | This CF1 pattern: bit0=0 → IFormat; the 0b10 pattern is caught by bit0=0 check |
| EC-003 | BC-2.19.025 | Fuzz input that fills carry to exactly 255 | No T0814; VP-047 harness verifies no OOB |
| EC-004 | BC-2.10.010 | VP-007 harness run before T0881 count=29 | Proof fails — expected; fix is ensuring STORY-173 landed |
| EC-005 | BC-2.05.012 | VP-004 re-run without Iec104 oracle arm | Proof fails — expected; fix is STORY-173 oracle update |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| BC files (6 anchor BCs × ~700 each) | ~4,200 |
| ADR-013 (formal verification decisions) | ~12,000 |
| src/analyzer/iec104.rs (complete, STORY-172) | ~15,000 |
| src/dispatcher.rs (post STORY-173) | ~5,000 |
| src/mitre.rs (post STORY-173) | ~5,000 |
| Test file (complete, all prior stories) | ~8,000 |
| TOTAL | ~52,200 |

Agent context window ~200k tokens. This story uses ~26% — within budget (near top end; if
needed, load only specific modules rather than all files).

## Previous Story Intelligence

**Predecessor:** STORY-173 (dispatcher integration)
- STORY-173 completed the full pipeline: dispatcher, CLI flag, MITRE catalog, SUPPORTED_PORTS
- This story does NOT write new production code — it runs the formal verification harnesses
  and fixes any remaining issues found by Kani/proptest/fuzz/mutants
- Common issues found in similar hardening stories (STORY-110/DNP3, STORY-132/ENIP):
  - Kani harnesses may need bounds annotations (`kani::assume`) to keep proof tractable
  - Proptest may find edge cases in the carry boundary arithmetic (255 vs 256)
  - Fuzz may expose a panic in the bad-start-byte resync scan if carry management is wrong
  - cargo-mutants often survives on `> 12` k-window checks if only a boundary test exists

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 8**: VP-044 scope = `parse_apci_header` ONLY. If the Kani harness tries
  to verify `on_data`, it will be too large for symbolic execution and will time out.
- **ADR-013 Decision 9**: VP-004 oracle re-run is MANDATORY after adding any new
  `DispatchTarget` variant. Do not skip it.
- **ADR-013 Decision 10**: VP-007 T0881 Kani re-run is MANDATORY after adding T0881 to
  mitre.rs. The `verify_all_emitted_ids_resolve` harness specifically validates EMITTED postcondition.
- No new production code changes are expected in this story. If a bug is found, fix it in a
  micro-commit before the proof re-run.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| kani | latest | VP-044, VP-004, VP-007 formal proof execution |
| proptest | latest (from Cargo.toml) | VP-045, VP-046 proptest execution |
| cargo-fuzz | latest | VP-047 fuzz execution |
| cargo-mutants | latest | Mutation testing sweep |

All libraries are already in the dev-dependencies from prior stories.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY (if fixes needed) | Targeted bug fixes from proof/fuzz findings; no new features |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add targeted tests for any surviving mutants |
| `cycles/wave-NN/wave-gate/code-review.md` | CREATE | Document surviving mutants + disposition |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- Do NOT expand VP-044 Kani harness scope to cover `on_data` — it will time out; VP-047 fuzz
  covers the loop; VP-044 is `parse_apci_header` only per ADR-013 Decision 8
