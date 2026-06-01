---
document_type: behavioral-contract
level: L3
version: "1.7"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/mod.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave 8 wave-level adv-pass-1 F-1 HIGH closure (S-7.01 sibling-BC propagation, W7.2 recurrence #5): PC3 enforcement-mode notation — \"remaining buffered data flushed in close_flow\" is structurally a defense-in-depth invariant (per-packet flush at mod.rs:162 drains buffer pre-close); enforced via code-review of close_flow flush loop body at lifecycle.rs:52-59; on_flow_close(Timeout) invocation and stats.flows_expired increment are automated-test-verifiable via STORY-019 AC-009/010/011/012. Mirrors BC-2.04.010 v1.5 PC2 + BC-2.04.029 v1.4 + ADR-0004 amendment precedent. — 2026-05-26"
  - "v1.4: Wave 8 wave-level adv-pass-2 F-1 MEDIUM closure (S-7.01 sibling-discipline): added PC5 documenting the force_set_flow_state_for_testing test seam (lifecycle.rs:232-244) — required by STORY-019 AC-012 to discriminate the Closed-state OR-branch of expire_flows (invariant 2). Mirrors BC-2.04.029 v1.4 PC7 pattern; authorized under ADR-0004 Amendment 2 state-injection seam class. — 2026-05-26"
  - "v1.5: Phase-4 HS-043 scope decision: added PC0 (caller obligation) explicitly requiring expire_flows to be invoked from the production per-packet processing path with the packet timestamp — closes the 'tested directly but never called in production' wiring gap identified in holdout-finding-triage-2026-06-01.md. Direct test-only invocations do not satisfy the production wiring requirement. — 2026-06-01"
  - "v1.6: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:536-552 → mod.rs:593-609 (expire_flows public fn); mod.rs:162 → mod.rs:191 (flush_contiguous_data call in process_packet, cited in PC3 enforcement note). — 2026-06-01"
  - "v1.7: ADV-IMPL-P10-MED-001 fix — PC0 re-anchored: the production per-packet idle-expiry enforcer is `expire_idle_by_timeout` (mod.rs:575-590, private hot-path variant), called from `process_packet` at mod.rs:166-169. `expire_flows` (mod.rs:593-609) is the public direct-call / offline API and is NEVER called from `process_packet` (zero call sites in src/). The memory-bound guarantee semantics are unchanged; only the named function and anchors are corrected. Test citation updated to `test_BC_2_04_013_PC0_idle_expiry_wired_in_process_packet` (per DF-AC-TEST-NAME-SYNC-001, coupled with FIX-P5-004 test rename). Supersedes the previously ACCEPTED ADV-HS043-P02-LOW-001 (same root issue — now properly fixed, not merely accepted). ADV-HS043-P02-MED-001 timestamp-monotonicity caveat carried forward unchanged (separate accepted item). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.013: expire_idle_by_timeout / expire_flows Closes Idle Flows Past flow_timeout_secs

## Description

The TCP reassembler enforces idle-flow expiry through two functions with distinct roles:

- **`expire_idle_by_timeout(current_time, handler)`** (mod.rs:575-590, `fn` — private) is the
  hot-path variant wired into `process_packet` at mod.rs:166-169. It applies only the
  time-based condition (`current_time > last_seen AND current_time - last_seen > timeout`),
  deliberately omitting the `FlowState::Closed` OR-clause (which `process_packet` already
  handles inline; including it on the hot path would violate the eviction-order invariant in
  BC-2.04.017). This is the function that provides the per-packet memory-bound guarantee for
  long-running captures.

- **`expire_flows(current_time, handler)`** (mod.rs:593-609, `pub fn`) is the public
  direct-call / offline API. It applies both clauses (time-based AND Closed-state). It is
  NEVER called from `process_packet`; it is the appropriate call site for offline tools,
  end-of-capture finalization helpers, and test code that needs both cleanup modes.

Each expired flow is closed with `CloseReason::Timeout` and `stats.flows_expired` is
incremented for each one.

## Preconditions

**PC0 (Caller Obligation — Production Wiring Requirement):** `expire_idle_by_timeout` MUST
be called from the production per-packet processing path inside `process_packet`
(mod.rs:166-169), with the arriving packet's timestamp as `current_time`. The sweep fires
whenever `timestamp > last_expiry_sweep_secs`, limiting the O(n) scan to at most once per
unique second of stream time. Calling `expire_flows` only from test code does NOT satisfy
this contract — `expire_flows` is NOT the per-packet production path; it is the public
direct-call API. The production binary must invoke `expire_idle_by_timeout` via
`process_packet` so that idle flows are actually expired during real captures. This
requirement is the basis of the capability-anchor memory bound ("idle flow expiry is required
to bound memory use in long-running captures"). See
`.factory/specs/phase-4-hs043-scope-decision.md` for the governing ruling.

1. `expire_idle_by_timeout(current_time, handler)` is called with a current timestamp from
   the production per-packet path via `process_packet` at mod.rs:166-169 (PC0 above).
2. At least one flow exists with `last_seen + flow_timeout_secs < current_time`.

## Postconditions

1. All flows satisfying `(current_time > last_seen AND current_time - last_seen > timeout)`
   are closed via `close_flow(key, CloseReason::Timeout, handler)` by
   `expire_idle_by_timeout` on the production per-packet path. (Note: the `state == Closed`
   OR-clause is present only in the public `expire_flows` API, not in
   `expire_idle_by_timeout`; the production hot-path enforcer is `expire_idle_by_timeout`.)
2. `stats.flows_expired` increments by the number of flows expired.
3. Each expired flow's remaining buffered data is flushed and `on_flow_close(Timeout)` is
   called. (Enforcement: in the current engine architecture, the per-packet flush at
   `src/reassembly/mod.rs:191` already delivers all contiguous-prefix data BEFORE
   `expire_idle_by_timeout` invokes `close_flow`. The `flush_contiguous` loop at
   `src/reassembly/lifecycle.rs:52-59` inside `close_flow` is therefore structurally a
   defense-in-depth invariant — cannot be triggered to deliver under current engine semantics.
   PC3's flush sub-property is enforced via code-review of the close_flow flush loop body's
   presence; the `on_flow_close(Timeout)` invocation and `stats.flows_expired` increment are
   automated-test-verifiable via STORY-019 AC-009/010/011/012 (mirrors BC-2.04.010 v1.5 PC2 /
   BC-2.04.029 v1.4 PC1-PC3 / ADR-0004 amendment enforcement-mode precedent).)
4. Flows not meeting the time-based expiry condition are untouched by `expire_idle_by_timeout`.
- **PC5 (Test Seam):** A `#[doc(hidden)] pub fn force_set_flow_state_for_testing(reassembler: &mut TcpReassembler, key: &FlowKey, state: FlowState) -> bool` accessor in `src/reassembly/lifecycle.rs` allows tests to directly mutate a flow's state without going through the state machine. Returns `true` if the flow was found and updated; `false` if no flow with the given key exists. Required by STORY-019 AC-012 to discriminate the state-based OR-branch of the public `expire_flows` API (invariant 2) from the time-based clause — the test constructs a flow in `FlowState::Closed` with `last_seen` well within the timeout window, proving the state-based clause fires independently. Note: this seam is relevant to tests calling `expire_flows` directly; the production path `expire_idle_by_timeout` does not apply the Closed-state clause.

  **Hygiene constraints:** `#[doc(hidden)]` (kept out of `cargo doc`); `_for_testing` suffix flags intent; MUST NOT be called from production code paths. Authorized as a NEW test-seam class (state-injection, NOT warning-guard) under ADR-0004 Amendment 2 (2026-05-26) "opt-in per-guard, gated by BC-driven need" doctrine.

## Invariants

1. The expiry check uses wrapping-safe subtraction: `current_time > flow.last_seen` is
   checked BEFORE `current_time - flow.last_seen > timeout` to avoid integer underflow.
   Both `expire_idle_by_timeout` (mod.rs:575-590) and `expire_flows` (mod.rs:593-609) apply
   this guard.
2. A flow that is already `FlowState::Closed` (e.g., FIN-closed but not yet removed) is also
   expired by the public `expire_flows` API. This handles the edge case where close_flow was
   not called in the same pass. **This Closed-clause is deliberately absent from the
   production hot-path `expire_idle_by_timeout`** — `process_packet` already handles
   Closed-state removal inline after FIN processing; duplicating the clause on the hot path
   would violate the eviction-order invariant (BC-2.04.017).
3. `expire_idle_by_timeout` does NOT expire flows that are still active within the timeout
   window. `expire_flows` (public API) also does not expire active flows unless they are
   already `FlowState::Closed`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow last_seen 1 second ago; timeout=300 | Not expired (applies to both expire_idle_by_timeout and expire_flows) |
| EC-002 | Flow last_seen 301 seconds ago; timeout=300 | Expired; flows_expired++ (applies to both functions) |
| EC-003 | current_time == last_seen (no idle time) | Not expired (current_time - last_seen == 0, not > timeout; applies to both functions) |
| EC-004 | Flow with state=Closed (public expire_flows API only) | Expired regardless of idle time via expire_flows; expire_idle_by_timeout does NOT apply this clause (deliberate: hot-path split per BC-2.04.017) |
| EC-005 | current_time < last_seen (timestamp wrap or reorder) | current_time > last_seen is false; NOT expired (underflow guard applies to both functions) |

## Canonical Test Vectors

| Input | Expected Output | Category | Test Function |
|-------|----------------|----------|---------------|
| Flow with last_seen=0, current_time=400, timeout=300 | Flow expired; flows_expired=1 | happy-path | `test_BC_2_04_013_expire_flows_closes_idle_flows` (STORY-019 AC-009) |
| Flow with last_seen=100, current_time=300, timeout=300 | Not expired (300-100=200 <= 300) | edge-case | `test_BC_2_04_013_expire_flows_does_not_close_active_flows` (STORY-019 AC-010) |
| Flow with state=Closed, any timestamps (expire_flows public API) | Expired regardless | edge-case | `test_BC_2_04_013_already_closed_state_is_expired` (STORY-019 AC-012) |
| Flow A at t=0 idle; Flow B SYN at t=6; timeout=5 — process_packet wiring | flows_expired >= 1 after Flow B's process_packet (expire_idle_by_timeout fired internally) | PC0 production-wiring | `test_BC_2_04_013_PC0_idle_expiry_wired_in_process_packet` (HS-043 gating regression; renamed from `test_BC_2_04_013_v15_PC0_expire_flows_called_from_process_packet` per FIX-P5-004 / DF-AC-TEST-NAME-SYNC-001) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Flows past timeout are closed | unit: create flow, expire with past timeout |
| — | Non-expired flows survive | unit: expire; assert surviving flows count |
| — | PC0 production wiring: expire_idle_by_timeout called from process_packet | unit: two-flow sequence; assert flows_expired after second packet's process_packet call |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- idle flow expiry is required to bound memory use in long-running captures |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:575-590 expire_idle_by_timeout, called at :166-169 from process_packet; reassembly/mod.rs:593-609 expire_flows public API) |
| Stories | STORY-019 |
| Origin BC | BC-RAS-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.014 -- related to (total_memory decrements as flows are expired)
- BC-2.04.015 -- related to (max_flows eviction: complementary capacity mechanism)
- BC-2.04.012 -- related to (finalize: end-of-capture expire-all)

## Architecture Anchors

- `src/reassembly/mod.rs:575-590` -- `expire_idle_by_timeout`: hot-path time-only filter + close loop (PRODUCTION WIRED — per-packet enforcer)
- `src/reassembly/mod.rs:166-169` -- call site: `process_packet` invokes `expire_idle_by_timeout` when `timestamp > last_expiry_sweep_secs`
- `src/reassembly/mod.rs:593-609` -- `expire_flows`: public direct-call / offline API; includes both time-based and Closed-state clauses; NOT called from `process_packet`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path (production wired)** | `src/reassembly/mod.rs:575-590` (`expire_idle_by_timeout`, called from `process_packet` at :166-169) |
| **Path (public API)** | `src/reassembly/mod.rs:593-609` (`expire_flows`, direct-call / offline API) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `current_time > flow.last_seen && (current_time - flow.last_seen) > timeout`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.stats, self.total_memory |
| **Deterministic** | yes (given fixed current_time) |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callbacks) |

## Refactoring Notes

No refactoring needed. The guard ordering (current_time > last_seen first) prevents
underflow correctly.
