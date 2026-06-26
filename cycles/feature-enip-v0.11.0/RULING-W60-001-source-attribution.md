---
document_type: ruling
ruling_id: RULING-W60-001
cycle: feature-enip-v0.11.0
wave: Wave-60
status: binding
issued_by: architect (ADR-010 owner)
date: 2026-06-26
findings_adjudicated:
  - F-W60-001
  - F-W60-002
blocks_convergence:
  F-W60-001: true
  F-W60-002: false
---

# RULING-W60-001: Source IP Attribution and bytes_received Guard Ordering

## Summary

| Finding | Severity | Ruling | Convergence Impact |
|---------|----------|--------|--------------------|
| F-W60-001 — wrong source_ip attribution | HIGH | FIX REQUIRED via port-44818 heuristic (approach a) | BLOCKS Wave-60 convergence |
| F-W60-002 — bytes_received vs is_non_enip | MEDIUM | DEFER — bytes_received is EXEMPT from PC-5; no code change; BC clarification added | NON-BLOCKING (is_non_enip currently unreachable per RULING-137-002) |

---

## PART 1 — F-W60-001: Wrong source_ip Attribution

### Finding Summary

`EnipAnalyzer::on_data` at `src/analyzer/enip.rs:597-598` assigns:

```rust
let src_ip = flow_key.lower_ip();
let dest_ip = flow_key.upper_ip();
```

`FlowKey` is canonicalized by tuple-pair comparison `(ip, port) <= (other_ip, other_port)` (see
`src/reassembly/flow.rs:45-63`). `lower_ip` is therefore the numerically smaller `(ip, port)` tuple,
not the traffic originator. When the EtherNet/IP server IP sorts below the client (approximately 50%
of real captures), every finding from BC-2.17.010 through BC-2.17.015 attributes the attack source
to the victim controller rather than the attacker. The BCs specify `source_ip: Some(<source endpoint>)
— resolved from flow_key` meaning the endpoint that originated the packet, not the lower-sorted
endpoint.

### Approach Selection: (a) Port-44818 Heuristic

**Decision: Approach (a) — add `resolve_enip_client_ip(flow_key: &FlowKey) -> IpAddr`.**

Rationale:

1. **Exact parity with the DNP3 sibling.** The existing `Dnp3Analyzer::resolve_master_ip()` at
   `src/analyzer/dnp3.rs:1463-1469` uses the port-20000 heuristic with an identical documented residual
   limitation and a named `DRIFT-DNP3-DIRECTION-001` comment. The ENIP fix is the mirror image: the
   server listens on port 44818; the opposite endpoint is the client (command originator). Adding
   `resolve_enip_client_ip` follows this established intra-codebase precedent directly.

2. **No dispatcher rewiring required.** Approach (b) — threading `Direction` from the dispatcher —
   would require changing the `EnipAnalyzer::on_data` signature, updating the `DispatchTarget::Enip`
   arm in `dispatcher.rs`, and rippling through all call sites and BC-2.17.016/019 preconditions. The
   DNP3 dispatcher arm comment at `src/dispatcher.rs:360-366` explicitly documents "Direction and
   byte-offset are not threaded through" as an intentional decision, and DRIFT-DNP3-DIRECTION-001 defers
   it. Introducing `Direction` threading for ENIP while DNP3 still lacks it is an asymmetric footgun
   that creates future maintenance risk and violates the sibling-protocol consistency principle.

3. **ADR-010 alignment.** ADR-010 models ENIP integration explicitly after ADR-007 (DNP3) and
   ADR-005 (Modbus). ADR-007 chose port-heuristic-only resolution for DNP3 as a deliberate deferred
   decision. That deferral was re-confirmed in DRIFT-DNP3-DIRECTION-001. Extending the same deferral
   to ENIP via approach (a) is consistent with ADR-010's declared sibling-protocol methodology.

4. **ENIP port semantics are reliable.** Unlike DNP3 where non-standard outstation ports are common
   in fielded systems, EtherNet/IP explicitly-messaging servers always listen on TCP 44818 (IANA
   registration, ODVA normative). Non-standard server ports are rare (legacy non-conformant stacks
   only). The heuristic accuracy is higher for ENIP than for DNP3.

### Exact Fix Specification

Add the following pure-core static method to `EnipAnalyzer` (or as a module-level free function)
in `src/analyzer/enip.rs`. Place it immediately after the last pure-core free function and before the
`EnipAnalyzer` impl block, mirroring the DNP3 layout:

```rust
/// Resolve the EtherNet/IP client (command-originator) endpoint from the flow key.
///
/// **Port-heuristic-only resolution.** EtherNet/IP explicit-messaging servers listen on
/// port 44818 (IANA-registered); the opposite endpoint is therefore the client (the
/// command originator):
///
/// - `lower_port == 44818`  → lower endpoint is the server; upper is the client.
/// - `upper_port == 44818`  → upper endpoint is the server; lower is the client.
/// - neither port is 44818  → both endpoints are ephemeral (non-standard ENIP setup or
///                            future UDP/2222 scope); function silently returns `lower_ip`
///                            as a conservative fallback.
///
/// **Known limitation:** this heuristic is correct for standard EtherNet/IP flows where
/// exactly one endpoint is on port 44818. It cannot unambiguously resolve direction when
/// NEITHER endpoint is on 44818 (non-standard server port or proxied capture). In that
/// case the function silently returns `lower_ip`, which may or may not be the actual
/// command originator.
///
/// **Direction deferral (DRIFT-ENIP-DIRECTION-001):** this function uses only the
/// port-44818 heuristic above; it does NOT use the TCP `Direction` signal that sibling
/// analyzer Modbus (`src/analyzer/modbus.rs` ~355-382) receives. Direction-aware
/// resolution — threading `Direction` into `EnipAnalyzer::on_data` analogously to the
/// Modbus pattern — is deferred to a post-v0.11.0 "ENIP direction-aware source
/// resolution" follow-up chore. Threading `Direction` into `EnipAnalyzer::on_data` would
/// ripple across all Wave-60 STORY-13x call sites and was explicitly deferred following
/// the same DRIFT-DNP3-DIRECTION-001 precedent established for the DNP3 sibling analyzer.
fn resolve_enip_client_ip(flow_key: &FlowKey) -> IpAddr {
    if flow_key.lower_port() == 44818 {
        flow_key.upper_ip()
    } else {
        // Either upper_port == 44818 (standard case) or neither port is 44818
        // (non-standard / fallback). Return lower_ip as conservative fallback in
        // the neither-case, matching DNP3 resolve_master_ip fallback semantics.
        flow_key.lower_ip()
    }
}
```

**Replacement in `on_data` (enip.rs ~597-598):**

Replace:
```rust
let src_ip = flow_key.lower_ip();
let dest_ip = flow_key.upper_ip();
```

With:
```rust
let src_ip = Self::resolve_enip_client_ip(&flow_key);
let dest_ip = if flow_key.lower_ip() == src_ip {
    flow_key.upper_ip()
} else {
    flow_key.lower_ip()
};
```

`dest_ip` is used only in `check_t0814` evidence strings; the swap is correct by construction.

### Residual Limitation Documentation

The comment block in `resolve_enip_client_ip` above is the required residual documentation
(labeled `DRIFT-ENIP-DIRECTION-001`). No separate drift-item file is required; the doc-comment
in the function body is the authoritative location, mirroring `DRIFT-DNP3-DIRECTION-001`.

The implementing story MUST include:
- The `DRIFT-ENIP-DIRECTION-001` label in the function doc-comment (as shown above).
- A test case covering the neither-port-is-44818 fallback path.

### No ADR-010 Revision Required

The fix is a pure implementation correction within the ENIP analyzer's pure-core layer. ADR-010
Decision 1 specifies port-44818 classification at the dispatcher level; it does not prescribe
which endpoint is the client at the analyzer level. No ADR amendment is needed; the decision is
self-evidently correct from the ODVA/IANA port semantics. A note MAY be appended to ADR-010
§Consequences (negative) referencing DRIFT-ENIP-DIRECTION-001, but this is optional.

### Blocking Verdict

**F-W60-001 BLOCKS Wave-60 convergence.** Every merged detection story (STORY-132 through
STORY-135, covering T0846/T0888/T0858/T0816/T0836/T0814/ForwardOpen) emits findings with
an incorrect `source_ip` in approximately 50% of real captures. This is a correctness defect
in the core output observable. It must be fixed in a standalone fix PR on `develop` before
Wave-60 convergence is declared.

**Fix-PR scope:** one new file-change to `src/analyzer/enip.rs` only. No BC changes. No
spec-hash implications. No story regeneration required. No ripple into dispatcher or
reassembly code.

---

## PART 2 — F-W60-002: bytes_received vs is_non_enip Guard Ordering

### Finding Summary

`self.bytes_received` is incremented at `enip.rs:593` BEFORE the `is_non_enip` early-return
at `enip.rs:619`. BC-2.17.016 PC-5 says flows with `is_non_enip == true` are "immediate
no-ops: no parsing, no counter updates, no findings emitted." BC-2.17.019 PC-2 says
`EnipAnalyzer::on_data()` "receives all subsequent TCP bytes for this flow."

The existing code comment at enip.rs:591-593 reads:
```rust
// WIRING-EXEMPT (this line only): routing-confirmation observable for STORY-131
// dispatcher tests (BC-2.17.019 PC-2). Single saturating_add; no branching; no I/O.
self.bytes_received = self.bytes_received.saturating_add(data.len() as u64);
```

This comment identifies the increment as a deliberate "wiring-exempt" placement, intentionally
outside the flow-level dispatch guard. The issue is a spec-level ambiguity between two different
BC-level observables, not a genuine implementation error.

### Reconciliation Decision

**bytes_received is EXEMPT from BC-2.17.016 PC-5's "no counter updates" language.**

Rationale:

1. **Different abstraction levels.** `bytes_received` is an **analyzer-level routing observable**
   (BC-2.17.019 PC-2): it proves that the dispatcher correctly wired port-44818 flows to the ENIP
   analyzer. It counts bytes arriving at the `EnipAnalyzer` dispatch boundary, not bytes processed
   by the per-flow analysis engine. Per-flow counters (`flow.parse_errors`, `flow.command_counts`,
   `flow.pdu_count`, `flow.malformed_in_window`) are what BC-2.17.016 PC-5 governs.

2. **"Counter updates" in PC-5 refers to per-flow counters.** BC-2.17.016's scope is the
   frame-walk loop and per-flow state. The entire precondition structure of BC-2.17.016 is
   `flow.is_non_enip == false at call time` — the BC describes flow-level behavior. A
   field on the `EnipAnalyzer` struct itself (`self.bytes_received`) is outside PC-5's scope.

3. **Latent status preserved.** Per RULING-137-002, `is_non_enip` is currently unreachable in
   production code. F-W60-002 is latent. When `is_non_enip` becomes reachable in a future cycle,
   the correct behavior is already captured: per-flow counters stop at the is_non_enip guard;
   the analyzer-level routing counter does not.

4. **No code change required.** The WIRING-EXEMPT comment in the implementation correctly
   captures the intent. The fix is a BC clarification only.

### Exact BC Clarification

**BC-2.17.016 — version bump to 1.2.** Add the following sentence to Postcondition 5:

> **Current text (PC-5):** "All subsequent `on_data` calls with `flow.is_non_enip == true`
> are immediate no-ops: no parsing, no counter updates, no findings emitted."

> **New text (PC-5):** "All subsequent `on_data` calls with `flow.is_non_enip == true` are
> immediate no-ops for per-flow state: no parsing, no per-flow counter updates (parse_errors,
> command_counts, malformed_in_window, pdu_count), no findings emitted. **Exempt:** the
> analyzer-level `bytes_received` routing-confirmation counter (BC-2.17.019 PC-2 observable)
> is incremented unconditionally before any flow-level dispatch; it counts bytes arriving at
> the EnipAnalyzer dispatch boundary and is not subject to the per-flow no-op guard."

**BC-2.17.016 modified section** — add to Invariants list as Invariant 7:

> **Invariant 7 — bytes_received scope separation (F-W60-002):** `EnipAnalyzer.bytes_received`
> (AC-131-001 / BC-2.17.019 PC-2 observable) is an analyzer-level routing counter, not a
> per-flow analysis counter. It is incremented before any flow-level dispatch and is exempt
> from the `is_non_enip` no-op guard that governs per-flow counters. The architectural intent
> is documented as WIRING-EXEMPT in `EnipAnalyzer::on_data` at the increment site.

**BC-2.17.019 — no change required.** PC-2 "receives all subsequent TCP bytes" already implies
unconditional byte counting. The clarification is one-directional (from .016 to .019).

### Input-Hash Implications

Editing BC-2.17.016 changes the file content. The `input-hash` fields of stories that list
`BC-2.17.016` in their `inputs:` frontmatter will become STALE after this edit. Because:

- All STORY-13x stories are already merged (Wave-60 work is complete).
- The fix-PR for F-W60-001 is a new delivery on `develop`, not a story regen.
- Input-hash staleness on merged stories is not a blocking defect (per CLAUDE.md CI-gate
  deferral: the scan runs pre-Phase-4, and this feature cycle has already completed Phase 4).

**Action required:** after the BC-2.17.016 edit, the spec-steward should run
`bin/compute-input-hash --scan` to identify which stories report STALE, and add a note in
the cycle manifest that hashes were invalidated by F-W60-002 spec clarification (cosmetic
only — no story regeneration needed because the implementation is unaffected).

### Non-Blocking Verdict

**F-W60-002 is NON-BLOCKING for Wave-60 convergence.** The code is correct. The BC text
requires clarification to resolve the apparent contradiction, but no production code change
is needed. The BC edit should be bundled with the F-W60-001 fix-PR or applied as a separate
spec-steward commit, whichever is operationally convenient.

---

## PART 3 — Expected Behavior and Test Guidance

The implementing story for F-W60-001 MUST include the following test cases. These are
binding test obligations derived from this ruling.

### Test Case Set: resolve_enip_client_ip

| # | flow_key | Expected src_ip | Rationale |
|---|----------|-----------------|-----------|
| T1 | client=10.0.0.9:50000 ↔ server=10.0.0.2:44818 | `10.0.0.9` | server has port 44818; client must be returned even though 10.0.0.2 < 10.0.0.9 numerically |
| T2 | client=10.0.0.1:50000 ↔ server=10.0.0.9:44818 | `10.0.0.1` | same; lower IP happens to be the client — still correct |
| T3 | client=192.168.1.100:44818-ephemeral ↔ server=10.0.0.5:44818 | both ports 44818 — treat as server=lower, upper=client (lower_port==44818 branch fires) | degenerate case; lower_port==44818 branch returns upper_ip |
| T4 | peer_a=10.0.0.3:55000 ↔ peer_b=10.0.0.7:60000 (neither is 44818) | `10.0.0.3` (lower_ip fallback) | non-standard / DRIFT-ENIP-DIRECTION-001 fallback |

**Critical assertion for T1** (the primary regression anchor):

```
flow_key = FlowKey::new(
    IpAddr::V4("10.0.0.9".parse()), 50000,
    IpAddr::V4("10.0.0.2".parse()), 44818,
)
// After FlowKey canonicalization: lower=(10.0.0.2, 44818), upper=(10.0.0.9, 50000)
// (10.0.0.2 < 10.0.0.9)
assert_eq!(resolve_enip_client_ip(&flow_key), IpAddr::V4("10.0.0.9".parse()));
// MUST NOT be 10.0.0.2 (the server / victim controller)
```

### Test Case: Finding source_ip End-to-End

An integration-level test (or test calling `process_pdu` directly with a known PDU) MUST assert
that for the flow key above (client=10.0.0.9:50000, server=10.0.0.2:44818), a T0858 or T0846
finding's `source_ip` field is `Some(IpAddr::V4("10.0.0.9".parse()))` — not the server IP.

### Test Case: dest_ip in check_t0814 Evidence

For the same flow, the malformed-frame evidence string containing `src=` MUST use 10.0.0.9.

---

## PART 4 — Spec Edits Summary

| BC | Action | Version | Content |
|----|--------|---------|---------|
| BC-2.17.016 | AMEND | 1.1 → 1.2 | Add exemption clause to PC-5; add Invariant 7 (bytes_received scope separation) |
| BC-2.17.019 | NONE | 1.0 (unchanged) | PC-2 is already correct; no edit needed |
| BC-2.17.010 through BC-2.17.015 | NONE | unchanged | source_ip wording "resolved from flow_key" remains correct post-fix; no BC edit needed |
| ADR-010 | OPTIONAL | unchanged | May append a note to §Consequences (negative) about DRIFT-ENIP-DIRECTION-001; not required for convergence |

---

## PART 5 — Fix-PR Scope

The fix-PR for F-W60-001 on `develop`:

- **One file changed:** `src/analyzer/enip.rs`
- **Change 1:** Add `resolve_enip_client_ip(flow_key: &FlowKey) -> IpAddr` pure-core function
  with `DRIFT-ENIP-DIRECTION-001` doc-comment.
- **Change 2:** Replace `flow_key.lower_ip()` / `flow_key.upper_ip()` assignment at ~line 597-598
  with calls to `resolve_enip_client_ip`.
- **Tests:** Add unit tests for `resolve_enip_client_ip` (4 cases above) and the end-to-end
  `source_ip` assertion in at least one detection BC's test.
- **No BC file edits in the fix-PR.** BC-2.17.016 clarification is a spec-steward commit,
  separate from or bundled in the same PR at the architect's discretion.
- **PR type:** `fix/enip-source-ip-attribution` — semantic PR type `fix`.
- **Stories do not need to be regenerated.** The fix is an implementation correction to code
  that was already merged; the BCs' intent was always correct attribution, and the spec wording
  `source_ip: Some(<source endpoint>) — resolved from flow_key` is satisfied by this fix.

---

## Decision Log

| Decision | Rationale |
|----------|-----------|
| Approach (a) over (b) for F-W60-001 | Parity with DNP3 sibling pattern; avoids signature ripple; consistent with ADR-010 sibling-protocol methodology; ENIP port 44818 semantics more reliable than DNP3 port 20000 semantics |
| DRIFT-ENIP-DIRECTION-001 label in doc-comment | Mirrors DRIFT-DNP3-DIRECTION-001 precedent; makes future Direction-threading chore discoverable by grep |
| bytes_received exempt from PC-5 | Different abstraction levels: analyzer-level routing observable vs per-flow analysis counter; WIRING-EXEMPT comment already captures intent |
| BC-2.17.016 v1.2 clarification, not BC restructuring | Minimal change; one sentence + one invariant; no traceability impact beyond input-hash staleness on merged stories |
| F-W60-001 blocks, F-W60-002 does not | F-W60-001 is a correctness defect in emitted output; F-W60-002 is latent (is_non_enip unreachable) and requires only spec clarity |
