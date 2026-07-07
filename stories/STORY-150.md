---
document_type: story
story_id: STORY-150
id: STORY-150
title: "TLS Drain-Loop DRY Refactor (TLS-DRAIN-DUP-001) with Mandatory Kani VP-039 + Mutation Re-run"
epic_id: E-11
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-07-01T00:00:00Z
phase: 3
level: ops
cycle: "v0.12.0"
points: 5
estimated_days: 2
priority: P2
wave: "~"
depends_on: []
blocks: []
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
assumption_validations: []
risk_mitigations: []
traces_to: null
tdd_mode: strict
target_module: "analyzer/tls"
subsystems: [SS-5]
input-hash: "d41d8cd"
inputs: []
---

# STORY-150 — TLS Drain-Loop DRY Refactor (TLS-DRAIN-DUP-001) with Mandatory Kani VP-039 + Mutation Re-run

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 5

## Narrative

- **As a** wirerust maintainer evolving the TLS analyzer
- **I want** the per-direction dispatch duplication in `process_handshake_carry` unified via a
  shared abstraction, and the wave-70 code-review improvement queue addressed
- **So that** correctness fixes and behavioral changes applied to one dispatch arm are
  automatically applied to the other, the same divergence class that required dedicated fix
  stories for ENIP (STORY-139), DNP3 (STORY-140), and Modbus (STORY-141) is eliminated
  structurally, and post-STORY-149 carry-drain code quality is brought to a fully
  maintainable state before the next TLS feature cycle

## Background

The STORY-144/145 carry-drain implementation in `src/analyzer/tls.rs` originally placed
approximately 220 lines of symmetric per-direction logic directly inside `try_parse_records`.
STORY-149 (PR #374, 116100d, 2026-07-07) eliminated the primary structural duplication by
factoring the carry-drain loop into a new `process_handshake_carry` helper function.

**Post-STORY-149 reality (develop 116100d):** The residual per-direction dispatch duplication
(~50 lines) now lives in `process_handshake_carry` (`src/analyzer/tls.rs` lines ~866–984).
The duplicate structure is the `match direction { ClientToServer => { … }, ServerToClient =>
{ … } }` arms inside the carry-drain loop body (lines ~912–959). Both arms:

- Extract `msg_bytes` via the same `carry[consumed..consumed + 4 + body_len].to_vec()` pattern
- Call `parse_tls_message_handshake(&msg_bytes)` with the same error-path handling
- Contain duplicate `Ok(_) => self.parse_errors += 1` and `Err(_) => self.parse_errors += 1`
  arms that are byte-for-byte identical
- Differ only in the dispatched message type (0x01 vs 0x02), the hello-seen flag set
  (`client_hello_seen` vs `server_hello_seen`), and the downstream dispatch function
  (`handle_client_hello` vs `handle_server_hello`)

This duplication was identified as **TLS-DRAIN-DUP-001** in the maint-2026-07-01 sweep.
The practical risk: a correctness fix applied to one arm is silently not applied to the
mirror arm. This is the same root cause that required dedicated fix stories for the ENIP,
DNP3, and Modbus analyzers respectively:

- RULING-EDGECASE-001 → STORY-139 (ENIP carry-direction divergence)
- RULING-DNP3-SIBLING-001 → STORY-140 (DNP3 carry-direction divergence)
- RULING-MODBUS-SIBLING-001 → STORY-141 (Modbus carry-direction divergence)

The VP-039 proof module contains a line-correspondence table mapping Kani harness
assertions to specific line numbers in `process_handshake_carry`. Any structural refactor
of `process_handshake_carry` that shifts line numbering invalidates this table and must be
followed immediately by a Kani re-run to re-confirm spatial proof coverage. Mutation
coverage must also be re-verified on the delta (per PG-MUTANTS-JOBS-001 lesson encoded in
STORY-147).

In addition, the wave-70 code review produced a queue of seven improvement items
(CR-001..CR-007; see §Candidate Scope below) that are structurally related and best
addressed in the same story to avoid partial application.

**Story points note (v1.1 re-estimate):** The primary DRY target is now ~50 lines
(down from ~220 at maint-2026-07-01 authorship), but the wave-70 CR queue (CR-001..CR-007)
adds back meaningful scope — particularly CR-006 (RAII `CarryGuard`) and CR-002
(loop-break-with-value) which each require careful implementation and regression testing.
5 points retained.

## Goal

1. Unify the C2S and S2C dispatch arms in `process_handshake_carry` via a shared
   abstraction (function, closure, or macro) that avoids borrow-checker friction. Each
   arm reduces to a single parameterized call site. The duplicate `Ok(_)/Err(_)`
   parse-error arms are collapsed (CR-003).
2. Re-run all Kani VP-039 harnesses after the refactor and confirm they still prove the
   carry-drain correctness properties without changes to harness assertions.
3. Update the VP-039 line-correspondence table in the proof module to reflect the
   refactored function/line structure. Remove stale references to the old duplicated arms.
4. Re-run `cargo mutants --jobs 1` on the modified files and confirm no new uncovered
   mutants on the carry-drain path.
5. Address the wave-70 CR queue items within scope (see §Candidate Scope).

## Acceptance Criteria

AC-150-001: The C2S and S2C direction-dispatch arms in `process_handshake_carry` are
  unified via a shared abstraction (`drain_dispatch` function, closure, or macro). In the
  final implementation each arm calls this abstraction with direction-specific parameters
  (message-type byte, hello-seen flag setter, dispatch function); no substantive logic
  block — meaning the `msg_bytes` extraction, `parse_tls_message_handshake` call,
  hello-seen flag assignment, parse_errors increment, and dispatch invocation — is
  duplicated between the arms.

AC-150-002: All Kani harnesses in the VP-039 proof module (`#[cfg(kani)]` mod) pass
  without modification to their assertions after the refactor. If any harness required
  updating, the update is accompanied by a comment explaining what structural change
  occurred and why the original proof intent is preserved.

AC-150-003: The VP-039 line-correspondence table (in the proof module or an adjacent
  `// VP-039: line N` comment block) is updated to reference line numbers in the
  refactored code. All stale references to the old duplicated arms are removed.

AC-150-004: `cargo mutants --jobs 1` on the modified files reports no new surviving
  mutants relative to the pre-refactor baseline (mutation score on the carry-drain path
  is not degraded by the refactor). Any provably-equivalent survivors are documented
  with a justification comment following the precedent from fix-tls-clienthello-frag F6.

AC-150-005: `cargo test --all-targets` passes; existing VP-039 and VP-040 unit tests
  remain green. `cargo clippy --all-targets -- -D warnings` passes with no new warnings.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `TlsAnalyzer::process_handshake_carry` (per-direction dispatch unification) | `src/analyzer/tls.rs` | Effectful shell (mutates flow state, parse_errors) |
| `TlsAnalyzer::prepare_record_step` (STORY-149 single-borrow site; untouched by this story unless CR-001 is adopted) | `src/analyzer/tls.rs` | Effectful shell (mutates flow state) |
| VP-039 proof module (`#[cfg(kani)]` mod) | `src/analyzer/tls.rs` | Pure-core (Kani model functions) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Single-record handshake (no fragmentation) | Regression: behavior identical pre/post refactor; existing unit tests must remain green |
| EC-002 | Decision-4 carry-clear path | Refactor must not disturb the Decision-4 body_len-spoof guard (carry.clear() + overflow+1 + break); this path is above the direction dispatch match |
| EC-003 | VP-039 Kani harnesses post-refactor | All harnesses pass; line-correspondence table updated in same commit |
| EC-004 | Mutation survivors on carry-drain delta | Zero new survivors, or existing equivalent survivors re-documented with justification |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/analyzer/tls.rs` — dispatch arm unification | Effectful shell | Calls `flows.get_mut`, increments `parse_errors`, calls `handle_client/server_hello` |
| VP-039 proof module modifications | Pure-core | Kani harness functions operate on local model state; no I/O |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| `src/analyzer/tls.rs` (full TLS analyzer — needed for VP-039 proof module and surrounding context) | ~6,000 |
| BC files (0 BCs — `behavioral_contracts: []`, E-11 ops story) | ~0 |
| Tool outputs (cargo test, cargo mutants, kani) | ~1,500 |
| **Total** | **~10,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~5%** |

## Tasks (MANDATORY)

1. [ ] Read `src/analyzer/tls.rs` `process_handshake_carry` (lines ~866–984) — identify the
       exact per-direction dispatch arms and the duplicate `Ok(_)/Err(_)` parse-error arms
2. [ ] Design the shared dispatch abstraction (function, closure, or macro); prefer a closure
       to avoid borrow-checker friction; confirm it does not require changes outside
       `process_handshake_carry`
3. [ ] Write failing tests for AC-150-001 (dispatch arm duplication eliminated; grep-based
       source-inspection or unit test verifying symmetry)
4. [ ] Implement the shared dispatch abstraction; collapse duplicate `Ok(_)/Err(_)` arms (CR-003)
5. [ ] Address in-scope CR queue items (see §Candidate Scope — at minimum CR-003; others
       at implementer discretion subject to no-regression gate)
6. [ ] Update VP-039 line-correspondence table to new line numbers (AC-150-003)
7. [ ] Re-run Kani VP-039 harnesses: `cargo kani --harness <vp039-harness>` (AC-150-002)
8. [ ] Re-run `cargo mutants --jobs 1` on `src/analyzer/tls.rs`; confirm no new survivors (AC-150-004)
9. [ ] Run `cargo test --all-targets` — VP-039, VP-040, and all existing tests green (AC-150-005)
10. [ ] Run `cargo clippy --all-targets -- -D warnings` — clean

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-144 | Introduced carry-buffer fields (`client_hs_carry` / `server_hs_carry`) on `TlsFlowState` | Per-direction carry-drain loop inside `try_parse_records`; symmetric C2S/S2C arms | Symmetric arms diverge under maintenance — same class as ENIP/DNP3/Modbus |
| STORY-145 | Extended carry-path for S2C (ServerHello) | S2C arm mirrored C2S arm exactly; duplication intentional at time | S2C arm duplicated all Ok(_)/Err(_) handling verbatim |
| STORY-149 | Restructured `try_parse_records` into `prepare_record_step` + `process_handshake_carry`; bounded-borrow budget ≤ 4 | `process_handshake_carry` is now the sole carry-drain function; SINGLE-BORROW INVARIANT marker in source | Duplication moved from `try_parse_records` to `process_handshake_carry`; wave-70 code review produced CR-001..CR-007 improvement queue |
| STORY-147 | Mutation-testing lesson: `cargo mutants --jobs 1` mandatory; load-induced false-clean with `--jobs N>1` | PG-MUTANTS-JOBS-001 lesson encoded | `--jobs 1` is non-negotiable even though it is slower |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| SINGLE-BORROW INVARIANT: exactly 1 `flows.get`/`flows.get_mut` in `try_parse_records` body; `process_handshake_carry` ≤ 3 re-borrows; total budget ≤ 4 | STORY-149 AC-149-001; source-inspection test in `tests/bc_149_single_borrow_invariant_tests.rs` | Source-inspection test (grep-based); CI gate — must remain green after this refactor |
| `cargo mutants --jobs 1` (not `--jobs N>1`) | PG-MUTANTS-JOBS-001 (STORY-147) | Manual gate; implementer instruction |
| VP-039 harnesses must remain green; line-correspondence table must be updated in the same commit as the refactor | AC-150-002/003 | `cargo kani`; code review |
| `cargo clippy --all-targets -- -D warnings` must pass | CLAUDE.md CI gate | CI gate |
| Any new helper function must NOT introduce additional `flows.get_mut` calls inside `process_handshake_carry` beyond the existing 3-site budget | STORY-149 borrow budget | Source-inspection test |

## Library & Framework Requirements (MANDATORY)

| Tool / Library | Version | Purpose |
|---------------|---------|---------|
| Kani | current project version (see `.cargo/config.toml` or CI) | VP-039 harness re-run (AC-150-002) |
| `cargo-mutants` | current project version | Mutation re-run (AC-150-004); `--jobs 1` mandatory |
| Rust stable | per `rust-version` in Cargo.toml (≥1.91) | No new dependencies introduced by this story |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/tls.rs` | modify | Unify C2S/S2C dispatch arms in `process_handshake_carry`; collapse duplicate `Ok(_)/Err(_)` arms; update VP-039 line-correspondence table |
| `tests/bc_149_single_borrow_invariant_tests.rs` | verify (no change expected) | Source-inspection test for borrow budget — must still pass after refactor |

## Candidate Scope: Wave-70 CR Queue

The wave-70 code review (STORY-149 delivery) produced seven improvement items. These are
**candidates** for this story — the implementer may address any subset subject to the
no-regression gate. CR-003 is the primary item because it directly covers the identified
duplication; the others are structurally adjacent.

| ID | Description | Effort |
|----|-------------|--------|
| CR-001 | Move `prepare_record_step` to `impl TlsFlowState` for structural enforcement (the function operates only on flow state, not on `TlsAnalyzer`) | LOW |
| CR-002 | Replace the `decision4_fired` boolean flag with a loop-break-with-value pattern (avoids the flag entirely; cleaner control flow) | LOW |
| CR-003 | Collapse duplicate `Ok(_)/Err(_)` parse-error arms in the C2S/S2C dispatch match — **primary scope of AC-150-001** | LOW (mandatory) |
| CR-004 | Improve the fragmented-handshake bench fixture to use Criterion `iter_batched` for more accurate allocation measurement | LOW |
| CR-005 | `RecordStep` variant naming consistency (verify variant names follow project conventions; rename if needed) | LOW |
| CR-006 | Introduce a RAII `CarryGuard` type for type-enforced carry restore (replaces the manual `if let Some(state) = self.flows.get_mut` restore at the end of `process_handshake_carry`) | MEDIUM |
| CR-007 | Extend borrow-budget source-inspection test to also cover `handle_client_hello` / `handle_server_hello` (confirm they do not introduce unexpected `flows.get_mut` calls) | LOW |

**Note:** CR-004 (`iter_batched` bench improvement) may alternatively be addressed as a
standalone maintenance line item rather than bundled into this story, at the implementer's
discretion.

## Notes

- Source finding: TLS-DRAIN-DUP-001, maint-2026-07-01. Motivation: same arm-divergence
  class as RULING-EDGECASE-001 (ENIP), RULING-DNP3-SIBLING-001 (DNP3), and
  RULING-MODBUS-SIBLING-001 (Modbus) — all required dedicated fix stories after the
  original symmetric arms diverged under maintenance.
- **Re-anchor (v1.1, wave-70 gate F-W70P2-001):** Prior version cited ~220 lines of
  duplication in `try_parse_records`. Post-STORY-149 (PR #374, 116100d, develop 2026-07-07)
  reality: `try_parse_records` was restructured into `prepare_record_step` +
  `process_handshake_carry`; the residual per-direction dispatch duplication (~50 lines)
  now lives in `process_handshake_carry` (lines ~866–984, direction match arms ~912–959).
  All stale anchors to `try_parse_records` as the duplication site have been updated.
- The Kani re-run (AC-150-002/003) is MANDATORY. This is why this is a dedicated story
  rather than a refactor folded into another delivery: the VP-039 harnesses have
  line-level correspondence to `process_handshake_carry`, and a refactor that moves lines
  without re-running Kani silently invalidates the proof's spatial correspondence.
- Mutation re-run uses `--jobs 1` per STORY-147 (PG-MUTANTS-JOBS-001) to prevent
  load-induced false-clean results.
- Primary module: `src/analyzer/tls.rs` (`process_handshake_carry`, lines ~866–984; the
  per-direction dispatch arms within the carry-drain loop at lines ~912–959).
- Wave assignment is TBD — schedule at v0.12.0 planning. STORY-149 has merged (PR #374,
  116100d). STORY-150 should follow as the structural refactor once v0.12.0 planning
  assigns the wave.
- S-7.02 disposition: this story's creation at draft status documents TLS-DRAIN-DUP-001
  for v0.12.0 planning and closes the maint-2026-07-01 refactor-debt open item.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-07-07 | story-writer | **F-W70P2-001 re-anchor:** updated duplication site from `try_parse_records` (~220 lines) to `process_handshake_carry` (~50 lines, lines ~866–984) per post-STORY-149 (PR #374, 116100d) code reality. Updated Background, Goal, AC-150-001, Notes, module references. Added wave-70 CR queue (CR-001..CR-007) as §Candidate Scope. Bumped story points rationale note. Added full template compliance (story-template.md) with all frontmatter keys and mandatory sections. |
| 1.0 | 2026-07-01 | story-writer | Initial authorship at maint-2026-07-01. Cited ~220-line duplication in `try_parse_records` (pre-STORY-149). |
