---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-05
capability: CAP-05
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.011: Per-(TransportProto, Port) Counts Are Exact and Monotonically Non-Decreasing; Classified Flows Do Not Update TCP Counter; All TCP Entries Carry TransportProto::Tcp

## Description

This contract formalizes the exactness and monotonicity properties of the two counters
introduced by BC-2.05.010. A count is EXACT if, after N None-target `on_flow_close` calls
on port P, `unclassified_port_counts[(Tcp, P)] == N`. A count is MONOTONICALLY
NON-DECREASING if it never decreases after any `on_flow_close` or packet-receive event.
Additionally, this contract states the negative: classified-flow `on_flow_close` does NOT
update the TCP counter, and all TCP-map entries ALWAYS carry `TransportProto::Tcp` as their
first tuple element (not `TransportProto::Udp` and not any other variant).

## Related BCs

- BC-2.05.010 — composes with (defines WHAT is counted and HOW the keys are constructed; this BC defines the correctness properties of THAT counting)
- BC-2.12.023 — composes with (`--coverage-gaps` gate; counts are zero when gate is disabled)

## Preconditions

1. `--coverage-gaps` is set (`coverage_gaps_enabled == true` on `StreamDispatcher`).
2. For the exactness property: a sequence of `on_flow_close` calls with `DispatchTarget::None` targeting port P has been made.
3. For the classified-flow property: `on_flow_close` is called with a route other than `None` (e.g., `DispatchTarget::Http`, `Tls`, `Modbus`, `Dnp3`, `Enip`, `Arp`).

## Postconditions

1. **Exactness (TCP):** After exactly N `on_flow_close` calls with `DispatchTarget::None` and `lower_port == P`, `StreamDispatcher.unclassified_port_counts[(Tcp, P)] == N`. The count is exact, not an approximation.
2. **Exactness (UDP):** After exactly M UDP packets for which `min(src_port, dst_port) == Q` that are unhandled by any dissector, `udp_unclassified_counts[(Udp, Q)] == M`. The key uses `min(src_port, dst_port)` so request and response direction packets on the same service port accumulate into the same counter.
3. **Monotonicity:** Once a key `(TransportProto::Tcp, P)` has a value `V` in `unclassified_port_counts`, no subsequent operation decreases its value. The counter is monotonically non-decreasing over the lifetime of the analyzer run.
4. **No classified-flow increment:** If `on_flow_close` is called with `DispatchTarget::Http`, `Tls`, `Modbus`, `Dnp3`, `Enip`, `Arp`, or any classified variant (non-None), `unclassified_port_counts` is NOT modified. No key is added, no existing count is incremented.
5. **TCP-map key purity:** Every key in `StreamDispatcher.unclassified_port_counts` has `key.0 == TransportProto::Tcp`. No `TransportProto::Udp` key ever appears in the dispatcher's TCP map. This is a structural invariant of the dispatcher's map (the UDP counter is a separate map in the decode loop, never interleaved with the dispatcher's map).
6. **Zero-count absence:** A key `(TransportProto::Tcp, P)` is absent from `unclassified_port_counts` if and only if zero None-target flows on port P have closed. The map does NOT pre-populate keys with value 0 (uses `entry().or_insert(0) += 1` semantics).

## Invariants

1. The TCP counter is updated ONLY in the `DispatchTarget::None` arm of `on_flow_close`. It is NOT updated on_data, NOT updated on flow creation, NOT updated on classified flow close, and NOT decremented on any event.
2. The UDP counter is updated ONLY for UDP packets not handled by any dissector. It is per-packet, not per-flow. It is NOT decremented.
3. The two counters (`unclassified_port_counts` and `udp_unclassified_counts`) are logically independent. They do NOT share a single HashMap; they share only the key TYPE `(TransportProto, u16)`.
4. The `TransportProto` discriminant in a key is determined by the counter that owns the key: all keys in the dispatcher's `unclassified_port_counts` have `Tcp`; all keys in the decode loop's `udp_unclassified_counts` have `Udp`. Cross-contamination (a `Udp` key in the dispatcher map or a `Tcp` key in the UDP map) MUST NOT occur.
5. VP-042 Sub-C MUST verify that calling `on_flow_close` with a classified target on a port P that ALSO has None-target flows does not change the count for that port (the classified flow and unclassified flows share port space but the classified close must not increment the unclassified counter).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Three None-target TCP flows on port 502 | `(Tcp, 502)` count == 3 (exact) |
| EC-002 | One Modbus-classified flow (Http/502) + one None-target flow (502) | `(Tcp, 502)` count == 1 (only the None-target contributes) |
| EC-003 | One None-target TCP flow on port 80; then one Http-classified flow on port 80 | `(Tcp, 80)` count == 1; Http-classified close does NOT change the count |
| EC-004 | Two UDP packets to port 47808; one UDP packet to port 161 | `(Udp, 47808)` count == 2; `(Udp, 161)` count == 1; TCP map unchanged |
| EC-005 | No flows and no UDP packets | Both maps are empty (no keys with value 0) |
| EC-006 | Port 102: 5 None-target TCP flows | `(Tcp, 102)` count == 5 (exact); four-way collision noted at report time in BC-2.12.024 |
| EC-007 | Enip-classified flow on port 44818 closed | `(Tcp, 44818)` count NOT incremented |
| EC-008 | DNS-classified flow (Dns target on 53) closed | `(Tcp, 53)` count NOT incremented |
| EC-009 | `--coverage-gaps` not set; then coverage_gaps_enabled=true later (within one run) | Counts reflect only the period when coverage_gaps_enabled was true; prior None-target closes before the flag was set were NOT counted |
| EC-010 | u64 counter at max (extremely unlikely; bounded by pcap size) | Saturating semantics preferred; no panic on overflow; u64 headroom is 1.8×10^19 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 None-target TCP closes on port 502 | `unclassified_port_counts[(Tcp, 502)] == 3` | exact-count |
| 1 Http-classified TCP close on port 80; check port-80 counter | `unclassified_port_counts.get(&(Tcp, 80)) == None` (key absent) | no-classified-increment |
| Mixed: 2 None-target + 1 Http on port 80 | `(Tcp, 80)` count == 2 (only None-target) | mixed |
| 5 UDP packets to port 47808 | `udp_unclassified_counts[(Udp, 47808)] == 5` | udp-exact |
| Check no Udp key in dispatcher map | `unclassified_port_counts.keys().all(|(t, _)| *t == TransportProto::Tcp)` == true | key-purity |
| Check no Tcp key in UDP map | `udp_unclassified_counts.keys().all(|(t, _)| *t == TransportProto::Udp)` == true | key-purity |

## Verification Properties

| VP-NNN | Sub | Property | Proof Method |
|--------|-----|----------|-------------|
| VP-042 | Sub-A | `unclassified_port_counts.values().sum() == N` after N None-target closes (TCP dispatcher path) | proptest: `proptest_vp042_total_count_equals_n` |
| VP-042 | Sub-B | Per-port count equals None-target-close frequency for that port | proptest: `proptest_vp042_per_port_count_equals_frequency` |
| VP-042 | Sub-C | Classified-flow close does NOT update TCP counter | proptest: `proptest_vp042_no_count_spurious_on_classified_flows` |
| VP-043 | — | UDP counter exactness and monotonicity: per-packet count == M after M declined-by-all-dissectors UDP packets; keys use `min(src_port, dst_port)`; counter never decreases (main.rs decode-loop path) | proptest: `proptest_vp043_udp_counter_exactness` |
| — | All TCP-map keys have TransportProto::Tcp | unit: `test_BC_2_05_011_tcp_map_key_purity` |
| — | UDP-map keys have TransportProto::Udp; never cross-contaminate TCP map | unit: `test_BC_2_05_011_udp_map_key_purity` |
| — | Counter is monotonically non-decreasing (second None-target close on same port increases count) | unit: `test_BC_2_05_011_monotonic_increment` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md — this BC specifies the correctness and isolation properties (exactness, monotonicity, key purity) of the unclassified-flow counting extension to `StreamDispatcher`, which is the core of the Content-First Protocol Dispatch subsystem |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-05 (src/dispatcher.rs C-21 — `StreamDispatcher.unclassified_port_counts` on_flow_close None-target arm); SS-12 (src/main.rs decode loop — `udp_unclassified_counts` per-packet) |
| ADR | ADR-012 Decision 6 (TCP+UDP dynamic detection; exactness/monotonicity implied by counter semantics; key-type identity via TransportProto discriminant) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/dispatcher.rs` — `on_flow_close` `None` arm: `self.unclassified_port_counts.entry((TransportProto::Tcp, lower_port)).or_insert(0) += 1` — note that `+=1` is the ONLY mutation site for this counter (monotonicity follows)
- `src/dispatcher.rs` — NO `on_data` path increments `unclassified_port_counts` (only `on_flow_close` does)
- `src/main.rs` — UDP decode loop: `udp_unclassified_counts.entry((TransportProto::Udp, min(src_port, dst_port))).or_insert(0) += 1` per-packet (only for packets all dissectors decline)
- `tests/dispatcher_tests.rs` — VP-042 proptest harnesses, plus `test_BC_2_05_011_tcp_map_key_purity`, `test_BC_2_05_011_udp_map_key_purity`, `test_BC_2_05_011_monotonic_increment`

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-042 Sub-A — `proptest_vp042_total_count_equals_n` (TCP dispatcher path)
- VP-042 Sub-B — `proptest_vp042_per_port_count_equals_frequency`
- VP-042 Sub-C — `proptest_vp042_no_count_spurious_on_classified_flows`
- VP-043 — `proptest_vp043_udp_counter_exactness` (main.rs UDP path)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `StreamDispatcher.unclassified_port_counts` (TCP) and `udp_unclassified_counts` in decode loop (UDP) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (`&mut self`) |
| **Overall classification** | mixed (stateful mutation; no I/O) |
