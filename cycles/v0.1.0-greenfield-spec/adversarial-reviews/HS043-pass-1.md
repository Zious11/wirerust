---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-06-01T00:00:00Z
phase: 5
inputs: [src/reassembly/mod.rs, src/cli.rs, src/main.rs, tests/hs043_flow_expiry_tests.rs, behavioral-contracts/ss-04/BC-2.04.013.md, behavioral-contracts/ss-04/BC-2.04.017.md]
input-hash: "n/a"
traces_to: BC-2.04.013
pass: 1
previous_review: null
---

# Adversarial Review: HS-043 Flow-Expiry Wiring (Pass 1)

**Scope:** Implementation review of the HS-043 production change — wiring idle-flow
expiry into `process_packet` plus a new `--flow-timeout` CLI flag (BC-2.04.013 v1.5 PC0).
Worktree `.worktrees/hs043-flow-expiry`, branch `fix/hs043-flow-expiry-wiring`.

**Method note:** This pass was executed as a LIVE-MUTATION battery by the orchestrator
directly (fresh-context subagent dispatch was not available this turn). All mutations were
applied to `src/` and reverted; the worktree was confirmed clean (`git diff` empty) after the
battery. The operator will dispatch the fresh-context adversary agent for subsequent passes.

## Changed surface

- `src/reassembly/mod.rs` — new field `last_expiry_sweep_secs: u32`; new private
  `expire_idle_by_timeout(current_time, handler)` (time-only, strict-`>`); gated sweep call at
  top of `process_packet` (`if timestamp > self.last_expiry_sweep_secs`).
- `src/cli.rs` — `--flow-timeout u64`, `default_value_t = 300`, `range(1..)`.
- `src/main.rs` — `config.flow_timeout_secs = cli.flow_timeout.min(u32::MAX as u64) as u32`.
- `tests/hs043_flow_expiry_tests.rs` (5 tests) + `tests/fixtures/flow-expiry.pcap`.

## Live-Mutation Battery (results)

| # | Mutation | Expected | Observed | Verdict |
|---|----------|----------|----------|---------|
| 1 | No-op the `expire_idle_by_timeout` call in `process_packet` | HS-043 tests fail | 3/5 fail (PC0 wiring test, timeout+1 boundary, CLI flows_expired) | LOAD-BEARING ✅ |
| 2 | Strict `>` → `>=` in `expire_idle_by_timeout` filter | exact-timeout test fails | `boundary_not_expired_at_exact_timeout` fails (flows_expired 1 vs 0) | BOUNDARY GUARDED ✅ |
| 3 | Gate `timestamp > last_sweep` → `> last_sweep + 10` (skip sweeps) | idle escapes, tests fail | 3/5 fail | GATE LOAD-BEARING ✅ |
| 4 (probe) | active flow re-touched every second t=0..100, timeout=5 | never self-expires (delta-0) | flows_expired stays 0 throughout | DELTA-0 SAFE ✅ |
| 5 (probe) | idle flow + intermediate same-second packets then later packet | always caught at later ts | expired at t=6 (timeout=5) | NO-ESCAPE ✅ |
| 6 (probe) | regressing timestamp (sweep@10 then packet@8) | no sweep, no underflow panic | no panic, `current_time > last_seen` guard holds | UNDERFLOW SAFE ✅ |
| 7 | Wire `expire_flows` (Closed-clause) into `process_packet` instead | implementer claims this regresses | `test_BC_2_04_017_all_non_established_states_evict_first` FAILS (5→4 flows) | SPLIT JUSTIFIED ✅ |
| 8 | `--flow-timeout 0` | rejected | exit 2, "0 is not in 1..18446744073709551615" | RANGE OK ✅ |
| 9 | `--flow-timeout 18446744073709551615` (u64::MAX) | clamp, no panic | exit 0, flows_expired 0 | SATURATING CAST SAFE ✅ |

Full suite after revert: **1083 passed / 0 failed**; `clippy --all-targets -D warnings` clean;
`fmt --check` clean; `git diff` empty.

## Part B — Findings (Pass 1)

### CRITICAL
None.

### HIGH
None.

### MEDIUM
None.

### LOW

#### ADV-HS043-P01-LOW-001: BC-2.04.013 PC0 literal wording says `expire_flows`; implementation wires `expire_idle_by_timeout`
- **Severity:** LOW
- **Category:** spec-fidelity / traceability
- **Location:** BC-2.04.013.md:42-49 (PC0) vs `src/reassembly/mod.rs:166-169` and `:564-590`.
- **Description:** PC0 states `expire_flows` MUST be called from the production per-packet
  path. The implementation instead calls a new private `expire_idle_by_timeout` (time-only;
  omits the `state == FlowState::Closed` OR-clause). The spirit of PC0 (idle flows expired in
  production with the packet timestamp; `stats.flows_expired` increments) is fully satisfied,
  and the deviation is technically forced: wiring the literal `expire_flows` regresses
  `test_BC_2_04_017_all_non_established_states_evict_first` (verified — Mutation 7).
- **Evidence:** In production, a flow reaches `FlowState::Closed` only via `on_fin()` (both
  FINs, flow.rs:255-258) and is removed inline in the same `process_packet` pass
  (mod.rs:196-204). The `Closed`-state OR-clause in `expire_flows` is therefore reachable
  only via the `force_set_flow_state_for_testing` seam (BC-2.04.013 PC5). The two methods are
  behaviorally identical for all production-reachable flow states; they diverge only on
  test-seam-injected `Closed` flows. So `expire_idle_by_timeout` satisfies PC0's intent and is
  strictly safer for the eviction-order contract (BC-2.04.017).
- **Proposed Fix:** Documentation-only — append a one-line note to BC-2.04.013 PC0 (or a v1.6
  modified-entry) acknowledging that the production wiring uses a time-only variant of
  `expire_flows` because the `Closed`-state clause is unreachable in production and applying it
  on the hot path would prematurely remove test-seam Closed flows, regressing BC-2.04.017. The
  source already documents this thoroughly at mod.rs:157-165 and :566-575. No code change.
  Per project policy DF-VALIDATION-001 this LOW finding MUST be research-agent-validated before
  being filed as a GitHub issue. Recommend NOT blocking the merge on it.

## Convergence Assessment

Pass 1: 0 CRIT / 0 HIGH / 0 MED / 1 LOW (documentation-only, non-blocking). The core
correctness claims (load-bearing tests, strict-`>` boundary, gate no-escape, delta-0 safety,
underflow safety, BC-2.04.017 preservation, CLI range/default, saturating cast) are all
empirically confirmed via mutation. This is NOT yet convergence — minimum 3 clean passes
required, and passes 2-3 should be fresh-context to satisfy the Iron Law.

## Project Policy Rubric — Compliance

- **DF-VALIDATION-001** (research-validated deferred findings): The single LOW finding is a
  candidate deferred finding. It is recorded here but NOT filed as an issue; it requires
  research-agent validation before any issue is created. COMPLIANT (no premature issue filed).
