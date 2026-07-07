---
document_type: story
story_id: STORY-149
title: "TLS Carry-Path Performance Recovery + Fragmented-Handshake Benchmark Fixture"
epic_id: E-11
version: "1.3"
status: pending
producer: story-writer
timestamp: 2026-07-06T00:00:00Z
phase: f3
points: 5
priority: P1
wave: "70"
depends_on: []
blocks: []
behavioral_contracts: []
verification_properties: []
tdd_mode: strict
target_module: analyzer/tls
subsystems: [SS-5]
estimated_days: 3
assumption_validations: []
risk_mitigations: []
level: ops
traces_to: null
cycle: v0.11.4
github_issue: 360
input-hash: "d41d8cd"
inputs: []
---

# STORY-149 — TLS Carry-Path Performance Recovery + Fragmented-Handshake Benchmark Fixture

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** pending
**Wave:** 70
**Points:** 5

## Narrative

- **As a** wirerust maintainer tracking performance regressions across releases
- **I want** the TLS carry-path (`try_parse_records`) restructured for bounded-borrow HashMap access
  (budget ≤ 4 acquisition sites across `try_parse_records` body and `process_handshake_carry`) and
  a new Criterion benchmark fixture that exercises the carry-drain loop
- **So that** the `reassembly/tls.pcap` regression (+14.0% vs Jun-22 baseline, criterion-confirmed
  p < 0.05) is substantially recovered, carry-path regressions are detectable in future sweeps, and
  the project re-enters the +10% WARNING threshold relative to the May-19 anchor

## Background

The STORY-144/145/146 carry-path additions (fix-tls-clienthello-frag, waves 65–66)
introduced measurable overhead on the `reassembly/tls.pcap` Criterion benchmark: the
criterion crossed the +10% threshold relative to the May-19 baseline, as first measured in
the maint-2026-07-01 performance sweep.

The maint-2026-07-06 performance sweep (`.factory/maintenance/performance.md`, run
maint-2026-07-06) confirmed and strengthened the regression:
- **`reassembly/tls.pcap` +14.0% vs Jun-22 baseline** (27.842 µs vs 24.429 µs mean)
- **+19.6% vs May-19 anchor** (27.842 µs vs 23.281 µs)
- **Criterion: "Performance has regressed" (p < 0.05, +7.6% vs stored criterion base)**
- 13 high-severe outliers in 100 samples; mean shift of +3.4 µs is above thermal noise

The Jun-22 controlled re-run had classified the then-current +4.9% as noise; the current
reading at +14.0% vs Jun-22 represents a new, statistically confirmed regression in the
v0.9.3→v0.11.4 interval. Plausible contributors: observability counter increments on the
hot path (PR #365) and per-flow state purge on flow close (PR #362). This escalation
(maint-2026-07-06, human-approved at gate) moves STORY-149 from wave-TBD to wave 70.

Related: issue #360 (fragmented-handshake benchmark fixture) — tracked as the upstream
issue for AC-149-002.

Root-cause analysis identified two allocation hotspots in `try_parse_records`
(`src/analyzer/tls.rs`):

- **PERF-001 (HIGH):** The current implementation acquires `flows.get()` / `flows.get_mut()`
  multiple times per 0x16 (handshake) TLS record — re-hashing the `FlowKey` on each
  call. In the hot path this typically means 6–8 repeated HashMap operations against
  the same key per record.
- **PERF-002 (HIGH):** Redundant `Vec` allocations occur per record: carry bytes are
  re-allocated rather than re-used via a local swap pattern.

Secondary allocation smells (LOW severity) also identified:
- **PERF-003:** Hex-encoding in cipher-suite logging allocates a `String` per record.
- **PERF-004:** Cipher-suite `Vec<u16>` is cloned in the `summarize()` path.
- **PERF-005:** Intermediate `String` allocation in JA3 assembly is not pooled across
  records.

Additionally, the existing `tls.pcap` benchmark fixture does not exercise the
multi-record carry-drain path — it delivers complete single-record TLS handshakes. The
carry-drain loop introduced by STORY-144/145 is therefore never exercised by the
existing criterion suite, so regression detection for that path is blind.

## Goal

1. Restructure `try_parse_records` into `prepare_record_step` (single `flows.get_mut()`
   acquisition site in the body; SINGLE-BORROW INVARIANT marker) plus
   `process_handshake_carry` (helper that re-borrows only after the primary borrow is
   released, with at most 3 acquisition sites; total budget ≤ 4 across both functions),
   using a carry-buffer swap (`std::mem::replace`, `std::mem::take`, or local Vec swap)
   to release the borrow before the `&mut self` dispatch call. This eliminates repeated
   FlowKey re-hashing (PERF-001) and the per-record carry Vec allocation (PERF-002).
2. Add a Criterion benchmark fixture (at `benches/tls_fragmented.rs` or as a new bench
   group in the existing TLS bench file) that delivers a genuinely fragmented multi-record
   TLS handshake — one that exercises the carry-drain loop. Establish this as the
   regression baseline for future carry-path changes. (See also issue #360.)
3. Verify the combined fix recovers ~5% on the `reassembly/tls.pcap` criterion,
   bringing it back under the WARNING (+10%) threshold relative to the May-19 baseline.
4. Optionally address PERF-003/004/005 if they fall within scope without expanding
   story points (secondary ACs).

## Acceptance Criteria

AC-149-001: `try_parse_records` in `src/analyzer/tls.rs` contains exactly one
  `flows.get`/`flows.get_mut` acquisition site within the `try_parse_records` body (the
  SINGLE-BORROW INVARIANT marker lives there); helper `process_handshake_carry` re-borrows
  only after the primary borrow is released, with at most 3 acquisition sites within that
  helper (total budget ≤ 4 acquisition sites across both functions). Both constraints are
  verified by a source-inspection test (grep-based). The carry-buffer swap uses
  `std::mem::replace`, `std::mem::take`, or a local Vec swap rather than a fresh allocation
  per record (implementation uses `std::mem::take` — functionally identical to
  replace-with-default).

AC-149-002: A Criterion benchmark fixture exists at `benches/tls_fragmented.rs` (or
  as a new bench group in an existing TLS bench file) that delivers a synthetic TLS
  handshake message spanning at least 3 TLS records — i.e., the carry-drain loop
  executes at least twice per synthetic handshake. The fixture is deterministic and
  repeatable. (Closes issue #360.)

AC-149-003: Running `cargo bench --bench pipeline` (or equivalent) against a comparable
  baseline shows the `reassembly/tls.pcap` criterion within +5% of the May-19 baseline
  (a stricter bar than the +10% WARNING threshold, i.e., the regression is substantially
  recovered).

AC-149-004 (optional): At least one of PERF-003/004/005 is resolved — hex-encoding
  alloc moved out of the hot path, cipher-suite clone replaced with a reference, or
  JA3 assembly buffer reused across calls.

AC-149-005: `cargo test --all-targets` passes without regression; existing VP-039 and
  VP-040 harnesses remain green. `cargo clippy --all-targets -- -D warnings` passes.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `TlsAnalyzer::try_parse_records` / `prepare_record_step` / `process_handshake_carry` (bounded-borrow restructure; budget ≤ 4) | `src/analyzer/tls.rs` | Effectful shell (mutates flow state) |
| `TlsFlowState.carry` (carry-buffer swap pattern) | `src/analyzer/tls.rs` | Pure-core data (byte buffer) |
| Fragmented-handshake benchmark fixture | `benches/tls_fragmented.rs` | Effectful shell (bench harness) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Carry-drain loop with exactly 3 partial records | Loop executes twice; carry drained on third record; one allocation instead of two (per-record `record_bytes` clone eliminated by `std::mem::take`) |
| EC-002 | Empty carry at entry to `try_parse_records` | `std::mem::take` returns the existing carry without copying; one allocation instead of two: the per-record `record_bytes` clone is eliminated |
| EC-003 | Single-record complete handshake (existing fixture) | Regression benchmark `reassembly/tls.pcap` shows recovery vs Jun-22 |
| EC-004 | VP-039 / VP-040 harnesses | Remain green; no behavioral regression from structural refactor |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/analyzer/tls.rs` — carry-buffer swap | pure-core mutation | Operates entirely on borrowed state; no I/O |
| `benches/tls_fragmented.rs` | effectful shell | Criterion harness measures wall-clock time; file-system neutral |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,000 |
| `src/analyzer/tls.rs` (full TLS analyzer) | ~6,000 |
| Existing bench file (`benches/pipeline.rs` or equivalent) | ~1,500 |
| Tool outputs (cargo bench, cargo test) | ~1,000 |
| **Total** | **~10,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~5%** |

## Tasks (MANDATORY)

1. [ ] Read `src/analyzer/tls.rs` `try_parse_records` — identify all `flows.get()` / `flows.get_mut()` call sites
2. [ ] Restructure `try_parse_records` into `prepare_record_step` (single acquisition site in body) + `process_handshake_carry` (≤ 3 re-borrows after primary released; total budget ≤ 4) (PERF-001)
3. [ ] Replace per-record carry Vec allocation with `std::mem::take` swap pattern (or `std::mem::replace` / local Vec swap from permitted set) (PERF-002)
4. [ ] Add inline comment asserting SINGLE-BORROW INVARIANT at the `prepare_record_step` acquisition site; add budget comment (≤ 4 total) at `process_handshake_carry` sites
5. [ ] Write failing test for AC-149-001 (bounded-borrow budget ≤ 4 invariant: exactly 1 acquisition site in `try_parse_records` body, ≤ 3 in `process_handshake_carry`, verified by source-inspection via grep)
6. [ ] Create `benches/tls_fragmented.rs` (or bench group) — synthetic 3-record fragmented TLS handshake (AC-149-002; closes issue #360)
7. [ ] Run `cargo bench --bench pipeline` — verify `reassembly/tls.pcap` regression recovery (AC-149-003)
8. [ ] (Optional) Address PERF-003, PERF-004, or PERF-005 if within scope (AC-149-004)
9. [ ] Run `cargo test --all-targets` — verify VP-039 / VP-040 green, no regressions (AC-149-005)
10. [ ] Run `cargo clippy --all-targets -- -D warnings` — clean

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-144 | Introduced carry struct for fragmented TLS handshake | Carry-drain loop in `try_parse_records`; naive acquire-per-operation pattern used | Naive pattern causes 6–8 HashMap re-hashes per record on hot path |
| STORY-145 | Extended carry-path for multi-record ClientHello | Carry-drain loop verified correct | Existing bench fixture does not exercise carry-drain loop (complete single-record fixture) |
| STORY-146 | Final carry-path fix in fix-tls-clienthello-frag wave 65–66 | Carry-buffer now part of shipped code | No regression baseline for carry-drain path |
| STORY-147 | E-11 tooling pattern: performance / mutation testing follow-up | E-11 stories have `behavioral_contracts: []`, no VPs defined by the story | Wave-TBD assignment until escalation gate |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Exactly 1 `flows.get`/`flows.get_mut` acquisition site in `try_parse_records` body; `process_handshake_carry` ≤ 3 re-borrows after primary released; total budget ≤ 4 across both functions | AC-149-001 inline comments (SINGLE-BORROW INVARIANT marker + budget annotation) | Source-inspection test (grep-based); CI gate |
| Carry swap via `std::mem::take` (permitted set: `replace` / `take` / local Vec swap; no per-record Vec allocation) | PERF-002 root-cause | Code inspection; confirmed by `cargo bench` recovery |
| `cargo clippy --all-targets -- -D warnings` must pass | CLAUDE.md CI gate | CI gate |
| VP-039 / VP-040 harnesses must remain green | AC-149-005 | `cargo test --all-targets` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `criterion` | `0.8` (current, per Cargo.toml) | Benchmark harness for fragmented-handshake fixture |
| Rust stdlib `std::mem::take` (permitted set: `std::mem::replace`, `std::mem::take`, local Vec swap) | stable | Carry-buffer swap (no new dependency) |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/tls.rs` | modify | Restructure `try_parse_records` into `prepare_record_step` + `process_handshake_carry` (bounded-borrow budget ≤ 4) + `std::mem::take` carry swap |
| `benches/tls_fragmented.rs` | create (or modify existing bench file) | Fragmented-handshake benchmark fixture (closes issue #360) |

## Notes

- Source findings: PERF-001/002 (HIGH) + PERF-003/004/005 (LOW), maint-2026-07-01.
- Escalation evidence: `.factory/maintenance/performance.md` (run maint-2026-07-06) —
  `reassembly/tls.pcap` +14.0% vs Jun-22 baseline (p<0.05, criterion-confirmed),
  +19.6% vs May-19 anchor. Human-approved escalation to wave 70 (maint-2026-07-06 gate).
- Related issue: #360 (fragmented-handshake benchmark fixture, AC-149-002).
- Primary module: `src/analyzer/tls.rs` (`try_parse_records`).
- The borrow-constraint root cause: `flows.get_mut()` returns a `&mut TlsFlowState`
  borrow on `self.flows`. This borrow conflicts with the `&mut self` call to downstream
  dispatch, requiring a carry swap to drop the borrow before dispatch. STORY-144
  introduced the carry struct but used a naive acquire-per-operation pattern; this story
  consolidates it to a bounded-borrow pattern (budget ≤ 4 across `try_parse_records` body
  and `process_handshake_carry`).
- Relationship to STORY-150: this story (149) fixes the performance regression first.
  STORY-150 then DRY-refactors the carry-drain duplication. Doing the perf fix first
  avoids attributing any residual regression to the structural refactor.
- Precedent: STORY-147 (mutation-testing defaults, E-11, wave TBD), STORY-143
  (changelog hardening, E-11, wave TBD) — same E-11 pattern of a cycle follow-up
  encoding a lesson into project tooling or infrastructure.
- Version 1.0 → 1.1 amendment: escalated to wave 70 per maint-2026-07-06 gate; added
  maint-2026-07-06 performance evidence, issue #360 reference, and full template
  compliance (story-template.md). Original story authorship predates this amendment.
- Version 1.1 → 1.2 amendment: AC-149-001 and EC-002 wording aligned to implementation
  (F-S149P1-001/003/005): `try_parse_records` restructured into `prepare_record_step`
  (single acquisition site in body; SINGLE-BORROW INVARIANT) plus `process_handshake_carry`
  (≤ 3 re-borrows after primary released; total budget ≤ 4); carry swap permitted set
  expanded to `std::mem::replace` / `std::mem::take` / local Vec swap (implementation uses
  `std::mem::take`); EC-001/EC-002 "no allocation / no extra allocation" replaced with
  accurate one-allocation wording. Sibling sweep applied to all live occurrences.
- Version 1.2 → 1.3 amendment: AC-149-003 bench name corrected from `--bench tls`
  (nonexistent target) to `--bench pipeline` (F-S149P3-001); AC-149-003 parenthetical
  reworded to make clear that +5% is the AC's own stricter recovery target, not
  equivalent to the +10% WARNING threshold (F-S149P3-002). Sibling sweep confirmed no
  other live occurrences of `--bench tls[^_]` in story or wave-70 artifacts.
