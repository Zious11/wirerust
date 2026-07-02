---
document_type: story
story_id: STORY-153
title: "Dispatcher `unclassified_port_counts` + UDP Decode-Loop `udp_unclassified_counts` — BC-2.05.010 + BC-2.05.011 + VP-042/VP-043"
epic_id: E-21
wave: 67
points: 8
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-protocol-coverage
github_issue: feature-protocol-coverage
subsystems: [SS-05, SS-12]
target_module: dispatcher
depends_on: []
blocks: [STORY-154]
behavioral_contracts:
  - BC-2.05.010
  - BC-2.05.011
verification_properties:
  - VP-042
  - VP-043
assumption_validations: []
risk_mitigations: []
# BC status: all BCs authored and anchored (F2 convergence complete)
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.010.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.011.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
input-hash: "1c75a19"
---

# STORY-153: Dispatcher `unclassified_port_counts` + UDP `udp_unclassified_counts` + VP-042/VP-043

## Narrative

**As a** wirerust developer implementing the dynamic gap detection feature,
**I want** the `StreamDispatcher` to accumulate per-(TransportProto, port) counts for TCP flows
that close as `DispatchTarget::None`, and the main.rs decode loop to accumulate matching counts
for UDP packets that no dissector handles,
**so that** `STORY-154` can read these counters to produce the `CoverageGapsSummary` report
with per-port unclassified traffic counts.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.05.010 | v1.3 | `unclassified_port_counts` Populated with (TransportProto, u16) Keys — TCP via Dispatcher None-Target, UDP via Decode-Loop | Primary: `TransportProto` enum, `unclassified_port_counts` field + dual-gate, `udp_unclassified_counts`, lower_port normalization |
| BC-2.05.011 | v1.1 | Per-(TransportProto, Port) Counts Are Exact and Monotonically Non-Decreasing; Classified Flows Do Not Update TCP Counter; All TCP Entries Carry TransportProto::Tcp | Primary: exactness, monotonicity, key-purity, no-classified-increment invariants |

## Acceptance Criteria

### AC-153-001: `TransportProto` minimal enum defined in `src/dispatcher.rs`
**Traces to:** BC-2.05.010 v1.3 PC-4, Invariant 1; ADR-012 Decision 6

```rust
/// Minimal transport discriminant for the (TransportProto, u16) gap-counter key.
/// Distinct from protocols::Transport (which has a third LinkLayer variant).
/// NOT imported from protocols.rs — defined here to enforce the pure-core boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportProto { Tcp, Udp }
```

This enum is defined IN `dispatcher.rs`, NOT imported from `protocols.rs`. Using `protocols::Transport`
here is FORBIDDEN (it has a `LinkLayer` variant that is not a valid TCP/UDP dispatcher key).

(traces to BC-2.05.010 v1.3 PC-4, Invariant 1; ADR-012 Decision 6)

**Red-Gate test:**
- `test_BC_2_05_010_key_type_identity` — `TransportProto::Tcp != TransportProto::Udp`; distinct values
- `test_BC_2_05_transport_proto_no_linkLayer` — verify at compile time: `TransportProto` has exactly 2 variants
  (achieved by exhaustive match with no wildcard: `match t { TransportProto::Tcp => .., TransportProto::Udp => .. }`)

### AC-153-002: `StreamDispatcher` gains `unclassified_port_counts` and `coverage_gaps_enabled` fields
**Traces to:** BC-2.05.010 v1.3 PC-1 (dual-gate), Postcondition 4; BC-2.05.011 v1.1 Postconditions 1, 3; ADR-012 Decision 6 Clarification

```rust
// In StreamDispatcher struct:
unclassified_port_counts: HashMap<(TransportProto, u16), u64>,
coverage_gaps_enabled: bool,
```

Both fields initialized in `StreamDispatcher`:
- `unclassified_port_counts: HashMap::new()` (or `HashMap::default()`) — initialized in `new()`
- `coverage_gaps_enabled: bool` — default `false` in `new()`; set via the builder method below

**PREFERRED implementation (lower blast radius — builder pattern):**
```rust
pub fn with_coverage_gaps(mut self, enabled: bool) -> Self {
    self.coverage_gaps_enabled = enabled;
    self
}
```
This is consistent with the existing `with_max_classification_attempts(mut self, ...) -> Self`
builder and BC-2.05.011 EC-009's "or equivalent" wording. All existing `StreamDispatcher::new()`
call sites remain untouched.

**F-F3P6-001 (wave-67 compile fix):** STORY-153 also introduces `coverage_gaps: bool` as a NEW
scalar parameter on `run_analyze()`, passed as `false` from the `Commands::Analyze` dispatch arm
in `main()`. This makes the `if coverage_gaps { ... }` gates in the decode loop compile at
wave 67 independently — before the `--coverage-gaps` CLI flag is added by STORY-154. STORY-154
(wave 69) changes the call-site to pass `*coverage_gaps` destructured from
`Commands::Analyze { ..., coverage_gaps, ... }`.

STORY-154 wires the flag by calling
`StreamDispatcher::new(/* existing 5 analyzer args */).with_coverage_gaps(coverage_gaps)` at the
single `StreamDispatcher::new()` call site in `run_analyze()` (main.rs:306), using the
`coverage_gaps` scalar parameter (not `args.coverage_gaps` — `run_analyze()` takes flat scalars).

If the `new()` parameter approach is used instead, ALL 8 existing call sites must be updated:
`tests/bc_2_14_105_modbus_dispatch_tests.rs`, `tests/tls_integration_tests.rs`,
`tests/timestamp_threading_tests.rs`, `tests/multi_analyzer_e2e_tests.rs`,
`tests/enip_e2e_real_pcaps_tests.rs`, `tests/enip_analyzer_tests.rs`,
`tests/bc_2_15_110_dnp3_dispatcher_tests.rs`, `benches/pipeline.rs`.

**Accessor:** `pub fn unclassified_port_counts(&self) -> &HashMap<(TransportProto, u16), u64>`

(traces to BC-2.05.010 v1.3 PC-1, PC-4; BC-2.05.011 v1.1 PC-1, PC-3; ADR-012 Decision 6 Clarification)

**Red-Gate test:**
- `test_BC_2_05_010_fields_accessible` — construct dispatcher with `.with_coverage_gaps(true)`; accessor returns empty map
- `test_BC_2_05_010_coverage_gaps_disabled_map_empty` — construct without `.with_coverage_gaps()`; after simulated None-target flow close, map still empty

### AC-153-003: TCP counter populated at `on_flow_close` for None-target flows — dual-gate + `lower_port`
**Traces to:** BC-2.05.010 v1.3 PC-1, Postconditions 1–4, Invariants 1–6; BC-2.05.011 v1.1 Postconditions 1, 3, 6; ADR-012 Decision 6 Clarification

In `on_flow_close`, the existing `None` arm (which already increments `self.unclassified_flows`):
```rust
Some(DispatchTarget::None) | None => {
    if self.http.is_some() || self.tls.is_some() || self.modbus.is_some()
        || self.dnp3.is_some() || self.enip.is_some()    // analyzer-present guard ONLY
    {
        self.unclassified_flows += 1;                    // existing counter — NOT gated on coverage_gaps_enabled
        if self.coverage_gaps_enabled {
            // NEW: per-(Tcp, lower_port) unclassified_port_counts increment
            let lower_port = flow_key.lower_port();      // F3-carry: use existing lower_port() method
            let c = self.unclassified_port_counts
                .entry((TransportProto::Tcp, lower_port))
                .or_insert(0);
            *c = c.saturating_add(1);                    // saturating_add (EC-153-10; no panic on u64 overflow)
        }
    }
}
```

**F3-carry item — Architecture Anchor:** Use `flow_key.lower_port()` (the method that already
exists on `FlowKey` in the real codebase, confirmed by `grep` in dispatcher.rs line:
`let ports = [flow_key.lower_port(), flow_key.upper_port()];`). Do NOT use `flow_key.src_port`
or `flow_key.dst_port` directly. The `lower_port()` method gives `min(src_port, dst_port)`,
which is the direction-normalized server/service port.

Gating structure (ADR-012 Decision 6 Clarification EXACT):
- `unclassified_flows += 1` is gated on the **analyzer-present guard ONLY** — NOT on `coverage_gaps_enabled`.
- `unclassified_port_counts` increment is gated on BOTH: (outer) analyzer-present guard AND (inner) `if self.coverage_gaps_enabled`.
- When no analyzers are configured: neither `unclassified_flows` nor `unclassified_port_counts` fires.
- When analyzers are present but `coverage_gaps_enabled=false`: `unclassified_flows` fires; the port counter does NOT.

> **REGRESSION WARNING:** Placing `unclassified_flows += 1` inside `if self.coverage_gaps_enabled`
> would zero `unclassified_flows` on all normal (coverage_gaps=false) runs, breaking BC-2.05.009
> and greenfield holdouts HS-040/HS-095. The code above is the ONLY correct structure per
> ADR-012 Decision 6 Clarification.

(traces to BC-2.05.010 v1.3 PC-1, PC-2, Postconditions 1, 3–4; BC-2.05.011 v1.1 PC-1, PC-3;
BC-2.05.011 Invariant 1; ADR-012 Decision 6 Clarification)

**Red-Gate tests:**
- `test_BC_2_05_010_tcp_counter_none_target` — after 1 None-target flow close on port 9999 (neutral non-classify() port, no payload; with analyzers configured + gaps enabled): `(Tcp, 9999)` count == 1 (port 502 is reserved exclusively for the Modbus-classified no-increment test)
- `test_BC_2_05_011_monotonic_increment` — after 3 None-target flow closes on same port P: `(Tcp, P)` count == 3
- `test_BC_2_05_011_no_increment_classified_flow` — Modbus-classified flow close (DispatchTarget::Modbus, port 502 with data): `(Tcp, 502)` key absent (EC-002 label fix: BC-2.05.011 EC-002 says "Http/502" but correct target is Modbus/502; port 502 ONLY appears in this test)
- `test_BC_2_05_010_lower_port_normalization` — flow with src=1234, dst=9999 (lower_port=9999) AND flow with src=9999, dst=1234 both produce key `(Tcp, 9999)` (direction-normalized; neutral port 9999 avoids classify() Rule 5 Modbus/502 interference)
- `test_BC_2_05_010_coverage_gaps_disabled_no_increment` — `coverage_gaps_enabled=false`; None-target flow; map remains empty

> **F3-carry item — EC-002 label fix (BC-2.05.011 EC-002):**
> BC-2.05.011 EC-002 was authored with the label "Http/502" referring to a Modbus-classified
> flow on port 502 (which should be `DispatchTarget::Modbus`, not `DispatchTarget::Http`).
> This label is wrong. The real `DispatchTarget` for port 502 is `Modbus`. Test
> `test_BC_2_05_011_no_increment_classified_flow` MUST use `DispatchTarget::Modbus` (port 502
> classified) to exercise EC-002 correctly, not `DispatchTarget::Http`. The test comment should
> note "EC-002 label in BC-2.05.011 says Http/502; correct target is Modbus/502".

### AC-153-004: TCP counter key purity — all keys carry `TransportProto::Tcp`
**Traces to:** BC-2.05.010 v1.3 Postcondition 3, Invariant 2; BC-2.05.011 v1.1 Postcondition 5, Invariant 4

Every key in `StreamDispatcher.unclassified_port_counts` has `key.0 == TransportProto::Tcp`.
No `TransportProto::Udp` key ever appears in the dispatcher's TCP map. This is a structural
invariant enforced by the single write site in `on_flow_close`.

(traces to BC-2.05.010 v1.3 PC-3, Invariant 2; BC-2.05.011 v1.1 PC-5, Invariant 4)

**Red-Gate test:**
- `test_BC_2_05_011_tcp_map_key_purity` — `unclassified_port_counts.keys().all(|(t, _)| *t == TransportProto::Tcp)` == true

### AC-153-005: `udp_gap_key` seam function in `src/dispatcher.rs` + decode-loop counter in `src/main.rs`
**Traces to:** BC-2.05.010 v1.3 PC-2..3, Postconditions 2–3, Invariants 3, 7; BC-2.05.011 v1.1 PC-2, Postcondition 2; ADR-012 Decision 6, Decision 10

> **SEAM CONTRACT — VP-043 (non-vacuity):** `udp_unclassified_counts` is a local variable in
> the `src/main.rs` binary-private decode loop. Tests in `tests/dispatcher_tests.rs` link only
> the library crate and CANNOT reach it. Without a library-visible seam, VP-043 harnesses would
> either fail to compile or silently replicate the decision logic — both are vacuous (violates
> DF-KANI-NONVACUITY-001; the exact anti-pattern VP-041 was reframed to avoid).
>
> The fix is a library-visible pure decision function `pub fn udp_gap_key(...)` in
> `src/dispatcher.rs`. VP-043 harnesses call this seam directly against production logic.
> The `main.rs` decode loop calls the same seam and accumulates results.
> BC-2.05.010 stays satisfied: the counter is still populated in the main.rs loop.
> This mirrors the VP-039/VP-040 `fill_buf_for_testing` seam pattern (VP-INDEX lines ~189–240).

**SEAM function definition (add to `src/dispatcher.rs`):**

```rust
/// Returns `Some((TransportProto::Udp, min(src_port, dst_port)))` when `parsed`
/// is a UDP packet that is NOT handled by any registered dissector (`dns_handles == false`).
/// Returns `None` when the packet is already classified (DNS accepted it) or is not UDP.
///
/// # SEAM CONTRACT (VP-043)
/// This is the library-visible boundary that VP-043 proptest harnesses exercise directly.
/// The `main.rs` decode loop calls `udp_gap_key(&parsed, dns_analyzer.can_decode(&parsed))`
/// and accumulates `Some(key)` returns into `udp_unclassified_counts`.
/// BC-2.05.010 is satisfied: the counter is populated in the main.rs loop via this seam.
/// The seam itself is pure and stateless — it does NOT modify any `StreamDispatcher` state.
pub fn udp_gap_key(
    parsed: &crate::decoder::ParsedPacket,
    dns_handles: bool,
) -> Option<(TransportProto, u16)> {
    if dns_handles {
        return None;
    }
    match parsed.transport {
        crate::decoder::TransportInfo::Udp { src_port, dst_port } => {
            Some((TransportProto::Udp, src_port.min(dst_port)))
        }
        _ => None,
    }
}
```

This is a **free function** in `dispatcher.rs` (not a method on `StreamDispatcher`) — it does
not depend on dispatcher state and must be callable from tests that have no `StreamDispatcher`
instance. Placing it in `dispatcher.rs` (not a new module) keeps the protocol-coverage seams
co-located with `TransportProto` (the key type they return).

**Decode-loop integration in `src/main.rs`:**

```rust
// Declare before the packet loop (main.rs):
let mut udp_unclassified_counts: HashMap<(wirerust::dispatcher::TransportProto, u16), u64> = HashMap::new();

// Inside the Ok(DecodedFrame::Ip(parsed)) arm (~line 356 of main.rs):
// `coverage_gaps` is the run_analyze() scalar param introduced by STORY-153 (passed as
// `false` from main()). STORY-154 changes the call-site to `*coverage_gaps` from
// `Commands::Analyze { ..., coverage_gaps, ... }` once the --coverage-gaps CLI flag exists.
if coverage_gaps {
    // ADR-012 Decision 10: dns_analyzer.can_decode() evaluated regardless of enable_dns.
    let dns_handles_this = dns_analyzer.can_decode(&parsed);
    if let Some(key) = wirerust::dispatcher::udp_gap_key(&parsed, dns_handles_this) {
        let c = udp_unclassified_counts.entry(key).or_insert(0);
        *c = c.saturating_add(1);   // saturating_add (EC-153-10; no panic on u64 overflow)
    }
}
```

**Key invariants:**
- Counter incremented per-packet (not per-flow)
- Key: `(TransportProto::Udp, src_port.min(dst_port))` — derived inside the seam from `TransportInfo::Udp { src_port, dst_port }`
- UDP packets arrive via `Ok(DecodedFrame::Ip(parsed))` arm in main.rs (~line 356); there is no separate UDP loop
- `dns_analyzer.can_decode()` is evaluated regardless of `enable_dns` flag (ADR-012 Decision 10; BC-2.05.010 Invariant 7). DNS/53 packets accepted by `can_decode()` → `udp_gap_key` returns `None` → NOT counted (gap-excluded)
- Counter only active when `coverage_gaps` flag is set (Postcondition 4)

(traces to BC-2.05.010 v1.3 PC-2..3, PC-2, Postconditions 2–3, Invariants 3, 7;
BC-2.05.011 v1.1 PC-2, Postcondition 2; ADR-012 Decision 10)

**Red-Gate tests (calling `udp_gap_key` seam directly — all in `tests/dispatcher_tests.rs`):**
- `test_BC_2_05_010_udp_counter_unhandled` — `udp_gap_key(&udp_47808_packet, false)` → `Some((Udp, 47808))`
- `test_BC_2_05_010_udp_dns_not_counted` — `udp_gap_key(&udp_53_packet, true)` → `None`
- `test_BC_2_05_010_udp_lower_port_normalization` — `udp_gap_key` with src=61000/dst=47808 AND src=47808/dst=61000 → both `Some((Udp, 47808))`
- `test_BC_2_05_011_udp_map_key_purity` — collect `Some(key)` returns from several non-DNS UDP inputs; all have `key.0 == TransportProto::Udp`

### AC-153-006: VP-042 proptest harnesses — TCP dispatcher path exactness and monotonicity
**Traces to:** BC-2.05.011 v1.1 VP table; BC-2.05.010 v1.3 VP table; ADR-012 Decision 6

Three proptest harnesses in `tests/dispatcher_tests.rs` inside `mod story_153 { ... }`:

**Sub-A — `proptest_vp042_total_count_equals_n`**:
After N `on_flow_close` calls with `DispatchTarget::None` (with `.with_coverage_gaps(true)`
AND ≥1 analyzer is_some() — precondition per ADR-012 Decision 6 Clarification),
`unclassified_port_counts.values().sum() == N`.

**Sub-B — `proptest_vp042_per_port_count_equals_frequency`**:
For each port P, after a sequence of None-target closes on various ports, the count for `(Tcp, P)`
equals exactly the number of closes that targeted port P.

**Sub-C — `proptest_vp042_no_count_spurious_on_classified_flows`**:
Calling `on_flow_close` with a classified variant (`DispatchTarget::Http`, `Tls`, `Modbus`, `Dnp3`,
or `Enip`) on port P that ALSO has None-target flows does NOT change the count for `(Tcp, P)`.

> **VP-042 sub-property (d) F3-carry resolution:** The VP-INDEX and BC-2.05.010 previously
> listed a 4th sub-property `(d)`. After Pass-9 adversarial convergence, VP-042 is defined with
> EXACTLY 3 sub-properties (A, B, C). The `(d)` row is dropped. This story implements the
> authoritative 3-harness set: Sub-A, Sub-B, Sub-C. The BC-2.05.010 VP table already lists
> exactly these three (VP-042 Sub-A through Sub-C). No additional `(d)` harness is needed.

(traces to BC-2.05.011 v1.1 VP table; BC-2.05.010 v1.3 VP table; ADR-012 Decision 6 Clarification)

### AC-153-007: VP-043 proptest harnesses — UDP gap-key seam exactness and DNS exclusion
**Traces to:** BC-2.05.010 v1.3 VP table; BC-2.05.011 v1.1 VP table; ADR-012 Decision 10

Two proptest harnesses in `tests/dispatcher_tests.rs` inside `mod story_153 { ... }`,
testing `udp_gap_key` directly (the VP-043 seam from AC-153-005):

**`proptest_vp043_total_count_equals_n`**:
For M synthetic UDP `ParsedPacket`s with `min(src_port, dst_port) == Q` and `dns_handles=false`,
call `udp_gap_key(parsed, false)` on each; assert all M calls return `Some((TransportProto::Udp, Q))`.
This verifies total accumulation == M in production (each `Some` return is one counter increment
in the main.rs loop). N ∈ [1, 256] per proptest strategy.

**`proptest_vp043_no_increment_on_classified_udp`**:
For a synthetic UDP `ParsedPacket` at port 53, call `udp_gap_key(parsed, true)` (simulating
`dns_analyzer.can_decode()` returning `true`); assert result is `None`. The seam returns `None`
→ the main.rs loop does NOT increment the counter → gate invariant holds.

(traces to BC-2.05.010 v1.3 VP table; BC-2.05.011 v1.1 VP table; ADR-012 Decision 10)

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `TransportProto` enum | `src/dispatcher.rs` (modify) | Pure (type definition) |
| `unclassified_port_counts: HashMap<(TransportProto, u16), u64>` field | `src/dispatcher.rs` (modify) | Effectful (mutable state) |
| `coverage_gaps_enabled: bool` field | `src/dispatcher.rs` (modify) | Effectful (control flag) |
| `unclassified_port_counts()` accessor | `src/dispatcher.rs` (modify) | Pure (read-only) |
| `on_flow_close` None-target arm augmentation (dual-gate) | `src/dispatcher.rs` (modify) | Effectful (mutation) |
| `udp_gap_key(parsed, dns_handles)` seam function | `src/dispatcher.rs` (modify) | Pure (decision fn; VP-043 seam) |
| `udp_unclassified_counts: HashMap<(TransportProto, u16), u64>` | `src/main.rs` (modify) | Effectful (mutable state) |
| UDP decode-loop — calls `udp_gap_key`; accumulates result | `src/main.rs` (modify) | Effectful (mutation) |
| VP-042 proptest harnesses (3 subs) | `tests/dispatcher_tests.rs` (modify) | Pure (proptest) |
| VP-043 proptest harnesses (2) — call `udp_gap_key` seam | `tests/dispatcher_tests.rs` (modify) | Pure (proptest) |

SS-05 (dispatcher) + SS-12 (main.rs decode loop). VP-004 Kani proofs are NOT affected:
`classify()` and `DispatchTarget` enum are NOT changed.

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-153-1 | BC-2.05.010 EC-002 | UDP packet to port 47808 (BACnet/IP, no dissector) | `(Udp, 47808)` incremented; BACnet IS flaggable |
| EC-153-2 | BC-2.05.010 EC-003 | TCP None-target on port 102 | `(Tcp, 102)` incremented |
| EC-153-3 | BC-2.05.010 EC-004 | Classified TCP Modbus on 502 closed | TCP counter NOT incremented |
| EC-153-4 | BC-2.05.010 EC-005 | `--coverage-gaps` NOT set | Both maps empty (zero overhead) |
| EC-153-5 | BC-2.05.010 EC-007/008 | Flow src=80 dst=54321 vs src=54321 dst=80 | Both produce key `(Tcp, 80)` (lower_port normalization) |
| EC-153-6 | BC-2.05.010 EC-010 | UDP/53 DNS packet handled by DnsAnalyzer | NOT in udp_unclassified_counts |
| EC-153-7 | BC-2.05.010 EC-014 | `--coverage-gaps` set, DNS analysis disabled (`enable_dns=false`), UDP/53 arrives | `can_decode()` evaluated regardless; if it returns true, NOT counted |
| EC-153-8 | BC-2.05.011 EC-002 (label fix) | Modbus-classified flow (DispatchTarget::Modbus, port 502) closed | TCP counter NOT incremented (EC-002 in BC should say Modbus/502 not Http/502) |
| EC-153-9 | BC-2.05.011 EC-009 | Mid-run toggle of `coverage_gaps_enabled` | Structurally impossible: field is immutable post-construction |
| EC-153-10 | BC-2.05.011 EC-010 | u64 counter at max | Saturating semantics preferred (use `saturating_add`) |

## Estimated Complexity

**Story points: 8** (dispatcher.rs: new TransportProto enum, new field, dual-gate modification
in on_flow_close; main.rs: UDP counter integration; VP-042 3 proptest harnesses; VP-043 2
proptest harnesses; unit tests for purity, key purity, monotonicity, lower_port normalization;
F3-carry items: lower_port() architecture anchor, EC-002 label fix, VP-042 sub-property (d) resolution)

## Token Budget Estimate

| Context source | Estimated tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC-2.05.010 (v1.3) | ~6,000 |
| BC-2.05.011 (v1.1) | ~4,500 |
| ADR-012 (Decisions 6, 10) | ~5,000 |
| src/dispatcher.rs (full — VP-004 zone) | ~12,000 |
| src/main.rs (decode loop section) | ~10,000 |
| tests/dispatcher_tests.rs (existing) | ~8,000 |
| Tool outputs (cargo check, proptest runs) | ~2,500 |
| **Total estimate** | **~51,000** |

Fits within a 200k context window (~26%). MEDIUM regression risk: dispatcher.rs carries
VP-004 Kani proofs. Read the full file before modifying. classify() and DispatchTarget MUST
NOT change.

## Tasks

0. **[F3-carry] Confirm lower_port() method exists in FlowKey**
   - `grep 'lower_port' src/dispatcher.rs` — confirms `flow_key.lower_port()` is already used
     in the existing `classify()` call (line: `let ports = [flow_key.lower_port(), flow_key.upper_port()];`)
   - This confirms the architecture anchor. Use `flow_key.lower_port()` in on_flow_close.

1. **Write Red-Gate tests first (TDD Step 1 — all must FAIL before implementation)**
   Add to `tests/dispatcher_tests.rs` in `mod story_153 { ... }`:
   - `test_BC_2_05_010_key_type_identity` — TransportProto variants distinct
   - `test_BC_2_05_transport_proto_no_linkLayer` — exhaustive match compiles with 2 arms
   - `test_BC_2_05_010_fields_accessible` — accessor exists
   - `test_BC_2_05_010_coverage_gaps_disabled_map_empty` — map empty when disabled
   - `test_BC_2_05_010_tcp_counter_none_target` — 1 None-target → count==1 (use port 9999, NOT 502; port 502 only for Modbus-classified no-increment test)
   - `test_BC_2_05_011_monotonic_increment` — 3 None-target → count==3
   - `test_BC_2_05_011_no_increment_classified_flow` — Modbus-classified flow (port 502 with data, DispatchTarget::Modbus) → map unchanged (EC-002 Modbus/502 label fix; port 502 reserved exclusively for this test)
   - `test_BC_2_05_010_lower_port_normalization` — bidirectional flows → same key (use port 9999)
   - `test_BC_2_05_010_coverage_gaps_disabled_no_increment` — disabled → no increment
   - `test_BC_2_05_011_tcp_map_key_purity` — all keys Tcp
   - `test_BC_2_05_010_udp_counter_unhandled` — `udp_gap_key(&udp_47808_packet, false)` → `Some((Udp, 47808))` (FAILS: seam fn not defined)
   - `test_BC_2_05_010_udp_dns_not_counted` — `udp_gap_key(&udp_53_packet, true)` → `None` (FAILS: seam fn not defined)
   - `test_BC_2_05_010_udp_lower_port_normalization` — seam with reversed ports → same key (FAILS: seam fn not defined)
   - `test_BC_2_05_011_udp_map_key_purity` — all `Some(key)` returns have `key.0 == TransportProto::Udp` (FAILS)
   - `proptest_vp042_total_count_equals_n` — Sub-A (proptest, FAILS before impl)
   - `proptest_vp042_per_port_count_equals_frequency` — Sub-B (proptest, FAILS)
   - `proptest_vp042_no_count_spurious_on_classified_flows` — Sub-C (proptest, FAILS)
   - `proptest_vp043_total_count_equals_n` — calls `udp_gap_key` M times; counts Some() returns; asserts == M (proptest, FAILS)
   - `proptest_vp043_no_increment_on_classified_udp` — `udp_gap_key(&udp_packet, true)` → `None` for all inputs (proptest, FAILS)
   All tests MUST FAIL (struct field doesn't exist yet; `TransportProto` undefined; `udp_gap_key` undefined).

2. **Add `TransportProto` enum + fields + builder + accessor to `src/dispatcher.rs` (AC-153-001 through AC-153-002)**
   - Define `pub enum TransportProto { Tcp, Udp }` in dispatcher.rs (NOT imported from protocols.rs)
   - Add `unclassified_port_counts: HashMap<(TransportProto, u16), u64>` field to `StreamDispatcher`
   - Add `coverage_gaps_enabled: bool` field to `StreamDispatcher` (default `false` in `new()`)
   - Add builder method `pub fn with_coverage_gaps(mut self, enabled: bool) -> Self`
     (consistent with existing `with_max_classification_attempts` builder pattern in dispatcher.rs)
   - Do NOT modify `StreamDispatcher::new()` signature — all existing call sites remain untouched
   - Add `pub fn unclassified_port_counts(&self) -> &HashMap<(TransportProto, u16), u64>` accessor
   - Verify: struct tests compile; `test_BC_2_05_010_fields_accessible` (`.with_coverage_gaps(true)`) GREEN; proptest harnesses still fail

3. **Augment `on_flow_close` None-target arm — CORRECT gating structure (AC-153-003 through AC-153-004)**
   - The existing analyzer-present guard wraps `unclassified_flows += 1`. The CORRECT structure is:
     ```
     if analyzer-present guard {                              // outer gate: analyzer-present ONLY
         self.unclassified_flows += 1;                       // NOT gated on coverage_gaps_enabled
         if self.coverage_gaps_enabled {                     // inner gate: port counter only
             let lower_port = flow_key.lower_port();
             let c = self.unclassified_port_counts
                 .entry((TransportProto::Tcp, lower_port))
                 .or_insert(0);
             *c = c.saturating_add(1);
         }
     }
     ```
   - Do NOT put `unclassified_flows += 1` inside `if self.coverage_gaps_enabled` — that is a
     regression breaking BC-2.05.009 + holdouts HS-040/HS-095
   - Use `flow_key.lower_port()` (the existing FlowKey method — F3-carry anchor)
   - Do NOT change `classify()`, `DispatchTarget`, or the classification logic
   - Verify: TCP counter tests GREEN; classified-flow tests GREEN; key-purity test GREEN

4. **Add `udp_gap_key` seam to `src/dispatcher.rs` + `udp_unclassified_counts` to `src/main.rs` decode loop (AC-153-005)**
   - In `src/dispatcher.rs`: add the `pub fn udp_gap_key(parsed: &crate::decoder::ParsedPacket, dns_handles: bool) -> Option<(TransportProto, u16)>` free function (SEAM CONTRACT; pure; VP-043 non-vacuity)
   - In `src/main.rs`: declare `let mut udp_unclassified_counts: HashMap<(TransportProto, u16), u64> = HashMap::new();` before the packet loop
   - Inside `Ok(DecodedFrame::Ip(parsed))` arm (~line 356): call
     `wirerust::dispatcher::udp_gap_key(&parsed, dns_analyzer.can_decode(&parsed))`; if `Some(key)`, do
     `let c = udp_unclassified_counts.entry(key).or_insert(0); *c = c.saturating_add(1);`
   - Gate the entire block on `if coverage_gaps { ... }` — **F-F3P6-001**: `coverage_gaps` is the
     new `run_analyze()` scalar parameter introduced by STORY-153 and passed as `false` from the
     `Commands::Analyze` call site in `main()`. This makes wave-67 code compile independently
     before the `--coverage-gaps` CLI flag exists. STORY-154 (wave 69) changes the call-site
     value to `*coverage_gaps` from the `Commands::Analyze { ..., coverage_gaps, ... }` destructure.
   - ADR-012 Decision 10: `dns_analyzer.can_decode()` is called regardless of `enable_dns`
   - Do NOT inline the UDP-packet-type-check in main.rs — the seam handles it
   - Verify: Red-Gate UDP tests (calling `udp_gap_key` directly) GREEN; VP-043 proptest harnesses GREEN

5. **Implement VP-042 (3 harnesses) and VP-043 (2 harnesses) proptest (AC-153-006 through AC-153-007)**
   - All 5 proptest harnesses use `.with_coverage_gaps(true)` + ≥1 analyzer `is_some()` precondition
   - Sub-C: must verify that a classified close on port P where None-target flows have ALSO
     closed does NOT change the count (mixed-port scenario)
   - VP-042 sub-property (d) is dropped — implement EXACTLY 3 subs (A, B, C)
   - Verify: `cargo test --all-targets` ALL GREEN

6. **Regression sweep — VP-004 Kani proofs unaffected**
   - `cargo test --all-targets` — ALL tests GREEN
   - `cargo kani` (if available in CI) — VP-004 harnesses unaffected (classify/DispatchTarget unchanged)
   - `cargo clippy --all-targets -- -D warnings` — zero warnings
   - `cargo fmt --check` — clean

7. **Micro-commit and PR** targeting `develop` (wave 67, parallel with STORY-151)

## Previous Story Intelligence

**N/A — first dispatcher story in E-21 (feature-protocol-coverage)**

Key lessons from analogous prior work:

**From STORY-033 (E-3, `unclassified_flows` counter):**
The existing `unclassified_flows` counter in `StreamDispatcher` lives in the same `None` arm of
`on_flow_close`. This story adds `unclassified_port_counts` ALONGSIDE it (not replacing it).
CRITICAL: `unclassified_flows += 1` is gated ONLY on the analyzer-present guard — NOT on
`coverage_gaps_enabled`. The NEW `unclassified_port_counts` increment is in a NESTED
`if self.coverage_gaps_enabled { ... }` block inside the analyzer-present guard. Do NOT
move `unclassified_flows` inside the `coverage_gaps_enabled` block — that would zero the
counter on all normal (coverage_gaps=false) runs, breaking BC-2.05.009 + greenfield holdouts
HS-040/HS-095 (regression).

**From STORY-031/032 (E-3, dispatcher):**
`classify()` function and `DispatchTarget` enum are VP-004 Kani-verified. DO NOT TOUCH THEM.
The Kani oracle (`classify_oracle`) references `DispatchTarget` — any change to the enum
would require re-verifying the proofs. This story's changes are additive to `StreamDispatcher`
state only.

**Lower_port precedent:**
The existing `classify()` function uses `flow_key.lower_port()` (grep confirms: `let ports = [flow_key.lower_port(), flow_key.upper_port()]`). Reuse this pattern.

**From STORY-088 (run_analyze orchestration):**
`StreamDispatcher::new()` is called in `run_analyze()`. With the builder approach, NO changes
to existing `StreamDispatcher::new()` call sites in this story. STORY-154 wires the flag by
changing the call-site in `main()` to pass `*coverage_gaps` and applying
`.with_coverage_gaps(coverage_gaps)` at `StreamDispatcher::new(/* existing 5 analyzer args */)`
(main.rs:306). Note: `coverage_gaps` is the scalar parameter on `run_analyze()`, NOT
`args.coverage_gaps` — `run_analyze()` takes flat scalar params, not a struct arg.
The `with_coverage_gaps` builder is consistent with the existing
`with_max_classification_attempts(mut self, ...) -> Self` pattern in dispatcher.rs.

## Architecture Compliance Rules

Source: `architecture/module-decomposition.md` + ADR-012 + BC-2.05.010/011

1. **`TransportProto` MUST be defined in `src/dispatcher.rs`** — NOT imported from `protocols.rs` (BC-2.05.010 PC-4, Invariant 1). Cross-importing would violate the SS-18 pure-core boundary and introduce a dependency on a module that must remain dependency-free.
2. **`classify()` and `DispatchTarget` MUST NOT be changed** — VP-004 Kani proofs depend on these exact types and logic. Any change breaks the proofs.
3. **`unclassified_flows += 1` is inside the analyzer-present guard ONLY** — NOT gated on `coverage_gaps_enabled` (ADR-012 Decision 6 Clarification EXACT). The NEW `unclassified_port_counts` increment is nested inside BOTH: (outer) analyzer-present guard AND (inner) `if self.coverage_gaps_enabled { ... }`. Placing `unclassified_flows` inside `coverage_gaps_enabled` would break BC-2.05.009 + holdouts HS-040/HS-095.
4. **TCP counter key: `(TransportProto::Tcp, flow_key.lower_port())`** — use `flow_key.lower_port()` (the existing FlowKey method), NOT `flow_key.src_port` or `flow_key.dst_port` directly.
5. **UDP counter per-packet, not per-flow** — UDP has no flow lifecycle in wirerust; increment per declined packet.
6. **ADR-012 Decision 10: `dns_analyzer.can_decode()` evaluated regardless of `enable_dns`** — the `enable_dns` flag gates DNS finding-emission only; gap-classification (whether to count the packet) is orthogonal. DNS/53 packets that `can_decode()` returns true for are NOT counted in `udp_unclassified_counts`.
7. **VP-042 has exactly 3 sub-properties (A, B, C)** — not 4. The "(d)" row is dropped per Pass-9 adversarial convergence. Implement 3 harnesses only.
8. **Test namespace isolation (DF-TEST-NAMESPACE-001):** ALL test functions in `mod story_153 { ... }` wrapper.
9. **No panic on u64 overflow** — use `saturating_add` semantics for counter increments (BC-2.05.011 EC-010; SEC-003 sibling-consistency pattern). Use `let c = map.entry(k).or_insert(0); *c = c.saturating_add(1);` — `.saturating_add_assign()` is NOT a real std method on `u64`.
10. **`DispatchTarget` and `classify()` are module-private in `src/dispatcher.rs`** — tests in `tests/` CANNOT directly construct `DispatchTarget` variants or call `classify()`. All test flows must be driven via the public API: `dispatcher.on_data(flow_key, payload)` / `dispatcher.on_flow_close(flow_key)` + the new `pub fn unclassified_port_counts()` accessor + `pub enum TransportProto`. Proptest harnesses in Task 5 must only use the public interface.
11. **`udp_gap_key` seam MUST be library-visible (`pub fn`) in `src/dispatcher.rs`** — `udp_unclassified_counts` is a local variable in the `main.rs` binary-private decode loop and is unreachable from `tests/`. The `pub fn udp_gap_key(...)` free function in `src/dispatcher.rs` is the library-visible boundary that VP-043 proptest harnesses call directly (DF-KANI-NONVACUITY-001). The main.rs decode loop calls the same production seam — no logic duplication. BC-2.05.010 is satisfied: the counter is populated in the main.rs loop via this seam.

## Library & Framework Requirements

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `std::collections::HashMap` | (std) | `HashMap<(TransportProto, u16), u64>` |
| `proptest` | (existing dev-dep) | VP-042 and VP-043 harnesses |

No new external crates.

**Forbidden dependencies:** `src/dispatcher.rs` MUST NOT import from `src/protocols.rs`.
The `TransportProto` enum in `dispatcher.rs` is INDEPENDENT of `protocols::Transport`.

## File Structure Requirements

| File | Change Type | Purpose |
|------|------------|---------|
| `src/dispatcher.rs` | Modify | `TransportProto` enum; `unclassified_port_counts` field; `coverage_gaps_enabled` field; `on_flow_close` augmentation; accessor method; `pub fn udp_gap_key(parsed, dns_handles)` seam (VP-043 non-vacuity) |
| `src/main.rs` | Modify | new `coverage_gaps: bool` scalar param on `run_analyze()` (passed as `false` from `main()` — STORY-154 changes call-site to `*coverage_gaps`); `udp_unclassified_counts` map; UDP decode-loop increment gated on `coverage_gaps`; STORY-154 adds `.with_coverage_gaps(coverage_gaps)` at the `StreamDispatcher::new(/* existing 5 analyzer args */)` call site (main.rs:306) |
| `tests/dispatcher_tests.rs` | Modify | VP-042 (3 harnesses) + VP-043 (2 harnesses) + unit tests in `mod story_153 { ... }` |

No new source files.

## Revision History

| Version | Date | Change | Finding IDs |
|---------|------|--------|-------------|
| v1.0 | 2026-07-02 | Initial story authored for feature-protocol-coverage F3 decomposition | — |
| v1.1 | 2026-07-02 | F-F3P1-002 (HIGH): Fixed AC-153-005 phantom `udp_header` → `if let TransportInfo::Udp { src_port, dst_port } = parsed.transport` pattern inside existing `Ok(DecodedFrame::Ip(parsed))` arm; clarified there is NO separate UDP loop in main.rs; updated Task 4 accordingly. F-F3P1-004 (MEDIUM): None-target tests (tcp_counter_none_target, lower_port_normalization) changed from port 502 to neutral port 9999; port 502 reserved exclusively for Modbus-classified no-increment test. Fixed AC-153-003 Red-Gate tests: Http/80 → Modbus/502 in no_increment_classified_flow annotation (EC-002 label fix). | F-F3P1-002, F-F3P1-004 |
| v1.2 | 2026-07-02 | F-F3P2-001 (CRITICAL): Fixed AC-153-003 code snippet — `unclassified_flows += 1` moved OUTSIDE `coverage_gaps_enabled` gate to analyzer-present guard only; `unclassified_port_counts` increment now nested in inner `if self.coverage_gaps_enabled { }` block (matches ADR-012 Decision 6 Clarification exactly). Removed regression warning + updated descriptive text. Fixed LOW `.saturating_add_assign(1)` (non-real std method) → `let c = ...; *c = c.saturating_add(1)`. F-F3P2-004 (MEDIUM): Changed AC-153-002 / Task 2 / Previous Story Intelligence to use builder method `with_coverage_gaps(mut self, enabled: bool) -> Self` instead of new `new()` parameter — no blast to 8 existing call sites. Updated Architecture Compliance Rule 3 + Previous Story Intelligence STORY-033/088 paragraphs + VP-042 proptest precondition language. Added ACR-10 (module-private `DispatchTarget`/`classify()` note; tests must use public `on_data`/`on_flow_close` + accessor). | F-F3P2-001, F-F3P2-004, LOW |
| v1.3 | 2026-07-02 | F-F3P3-003 (MEDIUM): Fixed AC-153-005 UDP snippet sibling-sweep gap — `udp_unclassified_counts.entry(...).or_insert(0) += 1` was non-compiling (bare `+= 1` on `Entry` return) and violated ACR-9 (saturating_add mandate). Replaced with `let c = ...; *c = c.saturating_add(1);` matching the AC-153-003 TCP sibling pattern (fixed in v1.2) and Architecture Compliance Rule 9. | F-F3P3-003 |
| v1.4 | 2026-07-02 | F-F3P4-001 (HIGH): Introduced `pub fn udp_gap_key(parsed, dns_handles)` library-visible seam in `src/dispatcher.rs` (SEAM CONTRACT). VP-043 proptest harnesses in `tests/dispatcher_tests.rs` link only the library crate and CANNOT reach `udp_unclassified_counts` (main.rs binary-private). Without the seam VP-043 would be vacuous (DF-KANI-NONVACUITY-001). Redesigned AC-153-005 to show seam function definition + main.rs decode loop calling it. Updated AC-153-007 VP-043 harness descriptions to call seam directly. Updated Architecture Mapping (added udp_gap_key row), Task 1 (UDP seam-based test descriptions), Task 4 (seam-call pattern), Architecture Compliance Rule 11 (seam library-visibility mandate), File Structure Requirements (dispatcher.rs row). BC-2.05.010 not violated: counter still populated in main.rs loop via the seam. | F-F3P4-001 |
| v1.5 | 2026-07-02 | F-F3P6-001 (MEDIUM): Fixed independent-compile gap — STORY-153 now explicitly introduces `coverage_gaps: bool` as a new scalar parameter on `run_analyze()` (passed as `false` from `main()`), ensuring wave-67 code compiles before `--coverage-gaps` CLI flag is added by STORY-154. Added note in AC-153-002, code comment in AC-153-005 decode-loop snippet, Task 4 bullet, and File Structure. F-F3P6-003 (LOW): Replaced phantom empty-parens `StreamDispatcher::new()` with `StreamDispatcher::new(/* existing 5 analyzer args */)` in AC-153-002 reference and File Structure. F-F3P6-005 cascade: removed `args.coverage_gaps` phantom struct ref from all STORY-153 occurrences. | F-F3P6-001, F-F3P6-003 |
