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

# BC-2.05.010: `unclassified_port_counts` Populated with (TransportProto, u16) Keys — TCP via Dispatcher None-Target, UDP via Decode-Loop

## Description

Two counters back the dynamic gap report for `CoverageGapsSummary`. Both use the key type
`HashMap<(TransportProto, u16), u64>` where `TransportProto` is a minimal `{Tcp, Udp}`
enum in `dispatcher.rs` (distinct from `protocols::Transport` which has a third `LinkLayer`
variant). The TCP counter lives on `StreamDispatcher` as `unclassified_port_counts`; it is
populated at `on_flow_close` for flows whose target is `DispatchTarget::None`. The UDP counter
lives in the `main.rs` decode loop as `udp_unclassified_counts`; it is populated per-packet
for UDP datagrams that no dissector handles. Both counters are only populated when
`--coverage-gaps` is enabled; they are NOT populated during normal `analyze` runs.

The `(TransportProto, u16)` key type enables TCP and UDP traffic on the same port number to
be counted independently: `(Tcp, 102)` is distinct from `(Udp, 102)`, and `(Udp, 47808)`
(BACnet/IP) is distinct from any hypothetical `(Tcp, 47808)`. This resolves the blocker
F2-SCOPE-DRIFT-UDP-001 (D-322).

## Related BCs

- BC-2.05.011 — composes with (exact/monotonic count properties and TCP-key invariant for the same maps)
- BC-2.12.023 — composes with (`--coverage-gaps` flag gates population of these counters)
- BC-2.12.024 — depends on (CoverageGapsSummary reads these counters and applies the tri-state classification)
- BC-2.05.009 — composes with (on_flow_close dispatch; this BC adds a new side-effect at the None-target arm)

## Preconditions

1. `--coverage-gaps` flag is set; `StreamDispatcher` has `coverage_gaps_enabled: bool` field (or equivalent) set to `true`.
2. For the TCP counter: `StreamDispatcher::on_flow_close` is called for a `FlowKey` whose cached route is `DispatchTarget::None` (i.e., the flow was never classified to a protocol dissector).
3. For the UDP counter: a UDP packet arrives in the decode loop in `main.rs` and ALL UDP dissectors have declined to handle it (e.g., `dns_analyzer.can_decode(&parsed)` returned `false` for this packet). If a dissector accepts the packet (e.g., `DnsAnalyzer` accepts UDP/53), the packet is NOT counted.
4. `TransportProto` is the minimal enum `{ Tcp, Udp }` defined in `dispatcher.rs` (NOT `protocols::Transport`, which has a third `LinkLayer` variant and MUST NOT be imported into the dispatcher per the pure-core boundary rule of SS-18).

## Postconditions

1. **TCP counter — on_flow_close for None-target flows:**
   - Let `lower_port = min(flow_key.src_port, flow_key.dst_port)` (direction-normalized; approximates the server/service port).
   - `StreamDispatcher.unclassified_port_counts.entry((TransportProto::Tcp, lower_port)).or_insert(0) += 1`.
   - The map is updated ONLY for `DispatchTarget::None` flows; classified flows (Http, Tls, Modbus, etc.) do NOT trigger this increment.

2. **UDP counter — per-packet in decode loop:**
   - Let `lower_port = min(udp_header.src_port, udp_header.dst_port)` (direction-normalized; same approach as TCP; computed per-packet from the UDP header).
   - `udp_unclassified_counts.entry((TransportProto::Udp, lower_port)).or_insert(0) += 1`.
   - The map is updated for every UDP packet that is NOT handled by any dissector (i.e., after the `dns_analyzer.can_decode()` check returns false and any other UDP dissectors also decline).

3. **Key type identity:** All keys in `StreamDispatcher.unclassified_port_counts` have `key.0 == TransportProto::Tcp` (the dispatcher handles TCP only). All keys in `udp_unclassified_counts` have `key.0 == TransportProto::Udp`.

4. **Conditional population:** When `--coverage-gaps` is NOT set, neither counter is populated (the HashMap is not allocated or remains empty). This preserves zero-overhead for normal `analyze` runs.

5. **CoverageGapsSummary merge:** At report-generation time, both maps are passed to the reporter (or merged) to produce `CoverageGapsSummary`. The merge uses the same `(TransportProto, u16)` key type so `(Tcp, 102)` and `(Udp, 102)` are distinct entries in the final output.

## Invariants

1. `TransportProto` in `dispatcher.rs` is a minimal `{Tcp, Udp}` enum. It is NOT imported from `protocols.rs`. The `protocols::Transport` enum is not used here (it has `LinkLayer` which is not a valid TCP/UDP transport for the dispatcher context).
2. The TCP map key is ALWAYS `(TransportProto::Tcp, lower_port)` where `lower_port = min(src_port, dst_port)`. A UDP key NEVER appears in the TCP dispatcher map.
3. The UDP map key is ALWAYS `(TransportProto::Udp, min(src_port, dst_port))`. A TCP key NEVER appears in the UDP counter map.
4. The counters are populated only at `DispatchTarget::None` close (TCP) or per-packet-unhandled (UDP). `DispatchTarget::Http`, `Tls`, `Modbus`, `Dnp3`, `Enip` classified flows do NOT increment either counter.
5. The TCP counter is populated at `on_flow_close`, NOT at `on_data`. This bounds overhead to closed flows, not packet volume.
6. Both counters are bounded by the port space: at most 65,536 distinct TCP keys and 65,536 distinct UDP keys (combined max 131,072 unique `(TransportProto, u16)` pairs). (u16 range is 0–65535 = 65,536 distinct values.)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TCP flow on port 47808 (unusual — BACnet typically UDP) closed as None-target | Counter key `(Tcp, 47808)` incremented; distinct from any `(Udp, 47808)` key |
| EC-002 | UDP packet to port 47808 (BACnet/IP) — not handled by any dissector | Counter key `(Udp, 47808)` incremented; BACnet/IP IS flaggable in gap report (D-320 OQ-5) |
| EC-003 | TCP flow on port 102 closed as None-target (S7comm gap) | Counter key `(Tcp, 102)` incremented; four-way collision applies at report time per BC-2.12.024 |
| EC-004 | Classified TCP flow (Modbus on 502) closed | `on_flow_close` route is `DispatchTarget::Modbus`; TCP counter NOT incremented |
| EC-005 | `--coverage-gaps` NOT set | Neither map is populated; zero-overhead for normal analyze runs |
| EC-006 | Two TCP flows on port 502 closed as None-target (unusual — port 502 normally classified) | `(Tcp, 502)` count == 2; TCP counter correctly counts unclassified flows even on otherwise-supported ports |
| EC-007 | Flow with src_port=80, dst_port=54321 — lower_port=80 | Key is `(Tcp, 80)` regardless of flow direction; direction-normalized |
| EC-008 | Flow with src_port=54321, dst_port=80 — lower_port=80 | Same key `(Tcp, 80)` as EC-007; bidirectional flows on same port merge into one counter |
| EC-009 | Multiple UDP packets to port 161 (SNMP) — each increments counter | `(Udp, 161)` count grows by 1 per unhandled UDP packet (per-packet, not per-flow) |
| EC-010 | UDP/53 DNS packet handled by `DnsAnalyzer` (`dns_analyzer.can_decode()` returns true) | Counter NOT incremented; `(Udp, 53)` absent from gap report; DNS traffic is classified, not unclassified |
| EC-011 | UDP/53 packet arrives, src=53 dst=60000 (response direction); `dns_analyzer.can_decode()` false | `lower_port = min(53, 60000) = 53`; key `(Udp, 53)` incremented only if dissector declines this packet |
| EC-012 | UDP packet src=61000 dst=47808 (BACnet query direction) | `lower_port = min(61000, 47808) = 47808`; key `(Udp, 47808)` incremented |
| EC-013 | UDP packet src=47808 dst=61000 (BACnet response direction) | `lower_port = min(47808, 61000) = 47808`; same key `(Udp, 47808)` — response and request merge into one counter |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| None-target TCP flow, src=1234, dst=502 (lower=502) | `unclassified_port_counts[(Tcp, 502)] == 1` | happy-path-tcp |
| UDP packet src=60000 dst=47808 (BACnet query; lower_port=47808; no dissector) | `udp_unclassified_counts[(Udp, 47808)] == 1` | happy-path-udp (BACnet) |
| UDP packet src=60001 dst=161 (SNMP query; lower_port=161; no dissector) | `udp_unclassified_counts[(Udp, 161)] == 1` | happy-path-udp (SNMP) |
| Classified TCP flow (Modbus/502) closed | `unclassified_port_counts` unchanged (no `(Tcp, 502)` increment) | classified-no-increment |
| TCP None-target on port 102 | `unclassified_port_counts[(Tcp, 102)] == 1` | port-102 |
| `(Tcp, 47808)` key vs `(Udp, 47808)` key | Both independently counted; they are distinct keys | transport-discrimination |
| `--coverage-gaps` not set | Both maps empty after any number of flows | conditional-population |
| UDP/53 DNS packet (DnsAnalyzer accepts: `can_decode()` returns true) | `udp_unclassified_counts` does NOT contain `(Udp, 53)` key | dns-handled-not-counted |
| UDP src=61000 dst=47808 (BACnet query, no dissector) | `udp_unclassified_counts[(Udp, 47808)] == 1` (key is min port = 47808) | min-port-normalization |
| UDP src=47808 dst=61000 (BACnet response, no dissector) | `udp_unclassified_counts[(Udp, 47808)] == 2` (same key; second packet increments) | min-port-normalization |

## Verification Properties

| VP-NNN | Sub | Property | Proof Method |
|--------|-----|----------|-------------|
| VP-042 | Sub-A | `unclassified_port_counts.values().sum() == N` after N None-target on_flow_close calls (TCP dispatcher path) | proptest: `proptest_vp042_total_count_equals_n` |
| VP-042 | Sub-B | Per-port count equals input frequency for TCP counter | proptest: `proptest_vp042_per_port_count_equals_frequency` |
| VP-042 | Sub-C | Classified-flow on_flow_close does NOT increment TCP counter | proptest: `proptest_vp042_no_count_spurious_on_classified_flows` |
| VP-043 | — | `udp_unclassified_counts` increments per-packet only for packets all dissectors decline; DNS/53 accepted by DnsAnalyzer does NOT appear; keys use `min(src_port, dst_port)` (main.rs UDP decode loop path) | proptest: `proptest_vp043_udp_counter_exactness` |
| — | TCP keys always have TransportProto::Tcp; UDP keys always have TransportProto::Udp | unit: `test_BC_2_05_010_key_type_identity` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md — the `unclassified_port_counts` HashMap is an extension of `StreamDispatcher`'s dispatch lifecycle: it records which (transport, port) flows were not dispatched to any protocol dissector, i.e., flows for which the Content-First Protocol Dispatch capability produced `DispatchTarget::None` |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-05 (src/dispatcher.rs C-21 — `StreamDispatcher.unclassified_port_counts`); SS-12 (src/main.rs decode loop — `udp_unclassified_counts`) |
| ADR | ADR-012 Decision 6 (TCP+UDP dynamic detection; (TransportProto, u16) key; BACnet/IP UDP/47808 flaggable; L2/multicast structurally absent; TransportProto minimal enum) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/dispatcher.rs` — `StreamDispatcher` struct gains `unclassified_port_counts: HashMap<(TransportProto, u16), u64>` field; populated at `on_flow_close` when `target == DispatchTarget::None && self.coverage_gaps_enabled`
- `src/dispatcher.rs` — `TransportProto` enum: `pub enum TransportProto { Tcp, Udp }` — minimal, defined here, NOT imported from `protocols.rs`
- `src/dispatcher.rs` — `on_flow_close` None-target arm: after incrementing `unclassified_flows`, also increment `(Tcp, lower_port)` key in `unclassified_port_counts`; `lower_port = min(flow_key.src_port, flow_key.dst_port)`
- `src/main.rs` — decode loop UDP path: for each UDP packet after all dissectors decline (i.e., after `dns_analyzer.can_decode(&parsed)` returns false and any other UDP dissectors decline), compute `lower_port = min(udp_header.src_port, udp_header.dst_port)` and increment `udp_unclassified_counts.entry((TransportProto::Udp, lower_port)).or_insert(0) += 1`
- VP-042: `proptest_vp042_total_count_equals_n`, `proptest_vp042_per_port_count_equals_frequency`, `proptest_vp042_no_count_spurious_on_classified_flows` (all in `tests/dispatcher_tests.rs` or equivalent)

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage — expected to be part of STORY-153 or equivalent dispatcher story)

## VP Anchors

- VP-042 Sub-A — `proptest_vp042_total_count_equals_n` (TCP dispatcher path)
- VP-042 Sub-B — `proptest_vp042_per_port_count_equals_frequency`
- VP-042 Sub-C — `proptest_vp042_no_count_spurious_on_classified_flows`
- VP-043 — `proptest_vp043_udp_counter_exactness` (main.rs UDP path; covers DNS exclusion and min-port key)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `StreamDispatcher.unclassified_port_counts` (TCP) and `udp_unclassified_counts` in decode loop (UDP) |
| **Deterministic** | yes (same inputs → same counter increments) |
| **Thread safety** | not thread-safe (`&mut self` on dispatcher; decode loop is single-threaded) |
| **Overall classification** | mixed (stateful mutation; no I/O) |
