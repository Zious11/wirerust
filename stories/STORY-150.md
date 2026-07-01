---
id: STORY-150
title: "TLS Drain-Loop DRY Refactor (TLS-DRAIN-DUP-001) with Mandatory Kani VP-039 + Mutation Re-run"
epic: E-11
wave: "~"
points: 5
status: draft
depends_on: []
input-hash: TBD
inputs: []
---

# STORY-150 — TLS Drain-Loop DRY Refactor (TLS-DRAIN-DUP-001) with Mandatory Kani VP-039 + Mutation Re-run

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 5

## Background

The STORY-144/145 carry-drain implementation in `src/analyzer/tls.rs`
`try_parse_records` contains approximately 220 lines of symmetric duplication across
the C2S (client-to-server) and S2C (server-to-client) carry-drain arms. The two arms
are structurally identical but parameterized by:

- The carry buffer field accessed (`client_hs_carry` vs. `server_hs_carry`)
- The overflow counter incremented (the C2S vs. S2C overflow counter)
- The downstream dispatch target (ClientHello parse branch vs. ServerHello parse branch)
- The `decision4` flag that signals carry accumulation for the next record

This duplication was identified as **TLS-DRAIN-DUP-001** in the maint-2026-07-01 sweep.
The practical risk: a correctness fix applied to one arm is silently not applied to the
mirror arm. This class of divergence is the same root cause that required dedicated fix
stories for the ENIP, DNP3, and Modbus analyzers respectively:

- RULING-EDGECASE-001 → STORY-139 (ENIP carry-direction divergence)
- RULING-DNP3-SIBLING-001 → STORY-140 (DNP3 carry-direction divergence)
- RULING-MODBUS-SIBLING-001 → STORY-141 (Modbus carry-direction divergence)

The VP-039 proof module contains a line-correspondence table mapping Kani harness
assertions to specific line numbers in `try_parse_records`. Any structural refactor of
`try_parse_records` that shifts line numbering invalidates this table and must be followed
immediately by a Kani re-run to re-confirm spatial proof coverage. Mutation coverage
must also be re-verified on the delta (per PG-MUTANTS-JOBS-001 lesson encoded in
STORY-147).

## Goal

1. Extract a shared `drain_hs_carry` helper (a function, closure, or macro — whichever
   avoids borrow-checker friction) that encapsulates the carry-drain loop logic. Each
   C2S/S2C match arm reduces to a single call site of ~15 lines.
2. Re-run all Kani VP-039 harnesses after the refactor and confirm they still prove the
   carry-drain correctness properties without changes to harness assertions.
3. Update the VP-039 line-correspondence table in the proof module to reflect the
   refactored function/line structure. Remove stale references to the old duplicated arms.
4. Re-run `cargo mutants --jobs 1` on the modified files and confirm no new uncovered
   mutants on the carry-drain path.

## Acceptance Criteria

AC-150-001: The C2S and S2C carry-drain arms in `try_parse_records` are unified via a
  shared abstraction (`drain_hs_carry` function, closure, or macro). In the final
  implementation each arm calls this abstraction with direction-specific parameters;
  no substantive logic block — meaning the drain loop body, overflow-counter increment,
  and decision4 flag assignment — is duplicated between the arms.

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

## Notes

- Source finding: TLS-DRAIN-DUP-001, maint-2026-07-01. Motivation: same arm-divergence
  class as RULING-EDGECASE-001 (ENIP), RULING-DNP3-SIBLING-001 (DNP3), and
  RULING-MODBUS-SIBLING-001 (Modbus) — all required dedicated fix stories after the
  original symmetric arms diverged under maintenance.
- The Kani re-run (AC-150-002/003) is MANDATORY. This is why this is a dedicated story
  rather than a refactor folded into another delivery: the VP-039 harnesses have
  line-level correspondence to `try_parse_records`, and a refactor that moves lines
  without re-running Kani silently invalidates the proof's spatial correspondence.
- Mutation re-run uses `--jobs 1` per STORY-147 (PG-MUTANTS-JOBS-001) to prevent
  load-induced false-clean results.
- Primary module: `src/analyzer/tls.rs` (`try_parse_records` and its carry-drain arms).
- Wave assignment is TBD — schedule at v0.12.0 planning alongside STORY-149 (TLS perf
  recovery). Suggested order: STORY-149 first (perf fix), then STORY-150 (structural
  refactor), so any residual performance delta is clearly attributable to the refactor
  and not conflated with the perf fix.
- S-7.02 disposition: this story's creation at draft status documents TLS-DRAIN-DUP-001
  for v0.12.0 planning and closes the maint-2026-07-01 refactor-debt open item.
