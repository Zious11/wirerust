---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-06-01T00:00:00Z
phase: 5
inputs: [src/reassembly/mod.rs, src/main.rs, src/cli.rs, tests/hs043_flow_expiry_tests.rs, behavioral-contracts/ss-04/BC-2.04.013.md, holdout-scenarios/HS-043-timeout-idle-cleanup.md]
input-hash: "n/a"
traces_to: BC-2.04.013
pass: 2
previous_review: HS043-pass-1.md
scope: "implementation --scope=diff-from:6158e6e (HS-043 fix surface: PR #171 + #172)"
---

# Adversarial Review: HS-043 Flow-Expiry Wiring (Pass 2)

**Scope:** Implementation review of the HS-043 production change (PR #171 + #172):
the gated idle-flow expiry sweep wired into `process_packet`, the new private
`expire_idle_by_timeout`, the `--flow-timeout` CLI flag, and the 8 regression
guards in `tests/hs043_flow_expiry_tests.rs`. Reviewed against BC-2.04.013 v1.5
(PC0 production-wiring obligation, the underflow invariant, and the EC table) and
the HS-043 holdout scenario (memory-bound guarantee for long-running captures).

**Method note:** Fresh-context review of the changed surface. Prior-pass findings
were deliberately not used as the basis for analysis (Iron Law information
asymmetry); the new finding below was reached independently by reasoning about the
sweep-gate semantics against the production packet-feed loop in `src/main.rs`, and
confirmed empirically with a throwaway probe test (since removed; tree clean).

## Changed surface reviewed

- `src/reassembly/mod.rs:150-169` — gated sweep at top of `process_packet`:
  `if timestamp > self.last_expiry_sweep_secs { self.last_expiry_sweep_secs = timestamp; self.expire_idle_by_timeout(timestamp, handler); }`
- `src/reassembly/mod.rs:575-590` — `expire_idle_by_timeout` (time-only, strict `>`, underflow-guarded).
- `src/main.rs:158-168` — production loop feeds `raw.timestamp_secs` per packet in capture order across `targets → pcap_files → packets` into ONE shared `reassembler`.
- `src/main.rs:122` — saturating `u64→u32` cast for `--flow-timeout`.
- `src/cli.rs:108-112` — `--flow-timeout`, `default_value_t = 300`, `range(1..)`.
- `tests/hs043_flow_expiry_tests.rs` — 8 tests (boundary `=timeout` not expired, `timeout+1` expired, delta-0 active flow, gated-sweep no-escape same-second, regressing-timestamp no-underflow-panic, CLI happy/zero-rejected, PC0 wiring).

## Findings (Pass 2)

### CRITICAL
None.

### HIGH
None.

### MEDIUM

#### ADV-HS043-P02-MED-001: Monotonic sweep gate suppresses expiry for sustained lower-timestamp runs (multi-file / reordered captures) — defeats the memory-bound guarantee
- **Severity:** MEDIUM
- **Category:** correctness / memory-bound (capability-anchor regression risk)
- **Location:** `src/reassembly/mod.rs:166-169` (gate) interacting with `src/main.rs:147-168` (multi-target/multi-file shared-reassembler loop).
- **Description:** The expiry sweep fires **only** when `timestamp > self.last_expiry_sweep_secs`,
  and `last_expiry_sweep_secs` advances monotonically to the running maximum stream
  timestamp ever seen. Consequently, once the reassembler has observed a high
  timestamp, **no expiry sweep fires for any subsequent packet whose timestamp is
  less than or equal to that running maximum** — for the entire duration of the
  lower-timestamp run. During that run, idle flows accumulate with `flows_expired`
  stuck at 0 and the flow table growing unbounded. This is precisely the
  unbounded-memory failure mode HS-043 and BC-2.04.013's capability anchor
  ("idle flow expiry is required to bound memory use in long-running captures")
  exist to prevent.
- **Production reachability:** `src/main.rs:146-185` feeds packets in raw capture
  order into a **single shared** `TcpReassembler` across an outer loop over
  `targets`, then `pcap_files`, then packets. Two realistic triggers:
  1. **Multi-file / multi-target run** (e.g. `wirerust analyze a.pcap b.pcap` or a
     glob target): independent captures have independent clocks. If `a.pcap` ends at
     a high epoch timestamp and `b.pcap` begins at a lower one, the gate stays stuck
     at `a.pcap`'s maximum for **all of `b.pcap`** — zero idle expiry across the
     entire second file regardless of how long flows sit idle.
  2. **Reordered / clock-adjusted single capture:** any sustained run of
     out-of-order or backward-stepping `ts_sec` values (merged pcaps, NTP step,
     wraparound near `u32::MAX`) suppresses expiry for the duration of the run.
- **Empirical confirmation:** A probe drove one flow at `t=100` (sets the gate to
  100), then 20 distinct new flows at `t=10` with `flow_timeout_secs=5`. Result:
  `flows_expired=0`, `flow_count=21` — every idle-eligible flow escaped expiry.
  Contrast: the same 20 flows fed at monotonically increasing timestamps expire as
  expected. (Probe was a temporary integration test using only public API —
  `process_packet`, `stats()`, `flow_count()`; removed after the run, `git status`
  clean.)
- **Why prior coverage misses it:** The existing
  `test_..._regressing_timestamp_no_underflow_panic` test only asserts the *underflow
  guard* (no panic) for a single regressing packet; it does not assert that idle
  flows are *expired* across a sustained low-timestamp run. The holdout HS-043 pcap
  fixture is strictly monotonic (t=0 then t=6), so neither the holdout nor any
  committed regression guard exercises the non-monotonic case. Pass-1's
  Mutation 6 probe likewise tested only single-packet underflow safety, not
  multi-packet expiry escape.
- **Spec status:** BC-2.04.013 does not carve out non-monotonic captures. PC0
  requires expiry to actually fire "during real captures" to honor the memory
  bound; EC-005 / Inv-1 address *underflow safety* for `current_time < last_seen`
  (a single comparison), but say nothing about the gate *suppressing the sweep
  entirely* for a run of such packets. This is a behavioral gap, not merely a doc
  gap.
- **Proposed fix (code) — choose one, then add a regression guard:**
  - **(a) Decouple the sweep cadence from monotonicity.** Track the gate as a
    "last swept" value but trigger a sweep whenever the timestamp *differs* from the
    last-swept second (`timestamp != last_expiry_sweep_secs`) rather than strictly
    increases — or sweep every packet (the scan is already O(n) bounded once per
    unique second; per-packet is acceptable for typical flow counts and removes the
    gate hazard entirely). Note: with a non-monotonic clock, the meaning of "idle
    past timeout" itself needs definition — expiry is relative to each flow's
    `last_seen`, which is itself stamped from possibly-regressing timestamps, so
    the engine should sweep using the *current packet's* timestamp regardless of
    global max.
  - **(b) Reset the gate per pcap file** at `src/main.rs` file-boundary (set
    `last_expiry_sweep_secs` back so the first packet of each new file re-arms the
    sweep) — addresses trigger (1) but not (2). Weaker; (a) is preferred.
  - Add a regression test feeding ≥2 idle-eligible flows during a sustained
    lower-timestamp run and asserting `flows_expired >= 1` (mutation-prove it by
    confirming it fails against the current monotonic gate).
- **Note:** Per DF-VALIDATION-001 this finding is recorded here but is NOT to be
  filed as a GitHub issue until research-agent-validated against current `develop`.
  Severity is the adversary's call (MEDIUM): it is a real production correctness gap
  in the headline capability, but requires a multi-file or non-monotonic capture to
  trigger and does not crash — single-file monotonic captures (the common case and
  the holdout fixture) behave correctly.

### LOW

#### ADV-HS043-P02-LOW-001 (re-surface of P01-LOW-001): PC0 literal wording says `expire_flows`; implementation wires `expire_idle_by_timeout`
- **Severity:** LOW
- **Category:** spec-fidelity / traceability
- **Location:** BC-2.04.013.md:42-49 (PC0) vs `src/reassembly/mod.rs:166-169`, `:575-590`.
- **Description:** PC0 names `expire_flows` as the method to wire into the per-packet
  path; the implementation wires the time-only `expire_idle_by_timeout` instead. The
  split is justified (wiring the literal `expire_flows` would run its
  `FlowState::Closed` OR-clause on the hot path and regress BC-2.04.017 eviction
  order), and source comments at `mod.rs:158-165` / `:564-574` document this. This
  is a documentation reconciliation only.
- **Reached independently** by reading PC0 against the wired method name; recorded
  here for completeness. Matches the prior-pass observation.
- **Proposed fix:** Documentation-only — a BC-2.04.013 v1.6 modified-entry noting the
  production wiring uses the time-only variant by design. No code change. Not
  merge-blocking.

## Convergence Assessment

Pass 2: **0 CRIT / 0 HIGH / 1 MED / 1 LOW.**

**Trajectory note (monotonicity):** Pass-1 reported 0/0/0/1; Pass-2 reports 0/0/1/1.
Total findings increased (1 → 2) and a new MEDIUM appeared. This is NOT a
fix-introduced regression and NOT a scope expansion — Pass-1 was an
orchestrator-run live-mutation battery (explicitly not fresh-context, by its own
method note), and this is the first genuine fresh-context pass, which the Iron Law
required precisely because round-1 systematically misses gaps. The new MEDIUM is a
real, empirically-confirmed correctness gap that Pass-1's mutation set did not probe
(it tested single-packet underflow, not sustained-low-timestamp expiry escape).
Recording the increase transparently per the monotonicity rule; root cause is
fresh-context discovery, not a defect introduced by the HS-043 fix.

**NOT CONVERGED.** The MEDIUM should be triaged (fix via option (a), or formally
accept-with-rationale and amend BC-2.04.013 to scope the memory-bound guarantee to
monotonic captures). After disposition, run Pass-3 fresh-context. Minimum 3 clean
passes still required; pass-2 is not clean.

## Project Policy Rubric — Compliance

- **DF-VALIDATION-001** (research-validated deferred findings): Both findings are
  candidate deferred findings. Recorded here; NEITHER filed as a GitHub issue.
  Research-agent validation required before any issue creation. COMPLIANT.
- **DF-SIBLING-SWEEP-001** (mandatory sibling sweep on remediation): Applies to the
  *fix burst* for ADV-HS043-P02-MED-001, not to this review pass. When the MED is
  remediated, the dispatch MUST include a sibling sweep: (a) grep for every call
  site of `expire_idle_by_timeout` / `expire_flows` / `last_expiry_sweep_secs`;
  (b) verify the `finalize()` end-of-capture path still expires-all (it does —
  `mod.rs:614-623` closes all flows unconditionally, so end-of-run memory is
  reclaimed even if mid-run sweeps were suppressed); (c) update BC-2.04.013 anchors
  and any consuming-story body references if line numbers shift. NOTED for the
  remediation burst.
