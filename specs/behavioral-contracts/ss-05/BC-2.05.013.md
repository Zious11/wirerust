---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-05
capability: CAP-05
lifecycle_status: active
introduced: feature-s7comm
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ARCH-INDEX.md
input-hash: "cf116b5"
---

# BC-2.05.013: `classify()` Rule 9 — TCP Port 102 Returns a Single `DispatchTarget::S7comm`; Disambiguation Deferred to the Analyzer

## Description

`StreamDispatcher::classify(data, flow_key)` returns `Some(DispatchTarget::S7comm)`
whenever the transport is TCP and either `src_port == 102` or `dst_port == 102`,
following no higher-priority content rule matching first. This is Rule 9 in the
port-fallback classification table, added by ADR-014 Decision 2 for the S7comm/
ISO-on-TCP feature. Unlike Modbus (Rule 5), DNP3 (Rule 6), ENIP (Rule 7), and IEC-104
(Rule 8) — where a single dispatcher target maps to a single fully-dissected protocol —
port 102 is shared on the wire by four distinct protocols (classic S7comm, S7comm-plus,
IEC 61850 MMS, ICCP/TASE.2, per ADR-012's documented "port-102 four-way collision").
Rule 9 deliberately does **not** attempt to disambiguate among them at the dispatcher:
it routes **all** TCP/102 traffic to exactly one target, `DispatchTarget::S7comm`, and
`S7commAnalyzer` (SS-21) performs the actual protocol-identity disambiguation
internally by parsing the TPKT/COTP framing (SS-20) and branching on the COTP
`protocol_id` byte (BC-2.21.002). This BC is the dispatcher-layer half of that split;
the analyzer-layer disambiguation contract is BC-2.21.002 and its four downstream
branches (BC-2.21.024 S7comm-plus, BC-2.21.027 unrecognized protocol-ID, BC-2.21.028
unparseable COTP payload).

The prior "no match" fallthrough arm (formerly Rule 9, `DispatchTarget::None`) is
renumbered Rule 10 by this change; no existing rule's behavior changes.

## Preconditions

1. `classify(data, flow_key)` is called with `TransportProto::Tcp`.
2. At least one of `flow_key`'s `src_port == 102` or `dst_port == 102` is true.
3. No higher-priority rule (Rules 1-8: TLS/HTTP content signatures, then the 443/8443,
   80/8080, 502, 20000, 44818, 2404 port-fallback rules) has already matched.
4. The S7comm analyzer has been instantiated by the caller (gated by a
   `--s7comm`/`--all`-equivalent flag, per the established `--iec104`/BC-2.12.025
   precedent — the concrete flag name and its BC-2.12.NNN registration are deferred to
   the story that wires CLI enablement); `classify()` itself remains flag-agnostic — a
   pure port-mapping function that does not read CLI state.

## Postconditions

1. Returns `Some(DispatchTarget::S7comm)`.
2. The flow is routed to `S7commAnalyzer::on_data` (SS-21).
3. No other `DispatchTarget` variant is returned for TCP/102 by `classify()` itself —
   there is no `DispatchTarget::S7commPlus`, `DispatchTarget::Mms`, or
   `DispatchTarget::Iccp` variant, and none is introduced by this BC. Protocol-identity
   disambiguation among S7comm/S7comm-plus/MMS/ICCP is a downstream, in-analyzer
   concern (BC-2.21.002), never a dispatcher-level one.
4. Non-classic-S7comm ISO-on-TCP traffic on port 102 (S7comm-plus, MMS, ICCP, or any
   unparseable COTP payload) is **still** dispatched to `DispatchTarget::S7comm` at
   this layer — Rule 9 does not, and cannot, reject it before the analyzer inspects
   the COTP payload. Postcondition 3's guarantee is about which *dispatcher* variant
   is returned, not about what the *analyzer* subsequently reports; see Invariant 5
   for the analyzer-side guarantee that such traffic is never force-fit to a
   classified S7comm result.

## Invariants

1. **TCP-only**: `classify(data, flow_key)` with `TransportProto::Udp` never returns
   `Some(DispatchTarget::S7comm)` — S7comm/ISO-on-TCP has no UDP variant.
2. **Content-first precedence**: if a higher-priority content-based rule (Rule 1 TLS,
   Rule 2 HTTP) fires first, that result takes precedence over Rule 9 (ADR-0001 /
   ADR-014 Decision 2, Rationale).
3. **Rule 9 is port-fallback only**: Rule 9 fires only when no content-based rule
   matched first, identical in kind to Rules 3-8.
4. **VP-004 oracle atomicity (six-step obligation, ADR-014 Decision 2)**: adding Rule 9
   to `classify()` MUST be accompanied, in the same commit, by: (a) the
   `DispatchTarget::S7comm` variant addition, (b) the mirrored `S7comm` arm in
   `classify_oracle` inside `#[cfg(kani)] mod kani_proofs`, (c) the early-exit guard
   extension to include `self.s7comm.is_none()`, (d) `S7comm` match arms in `on_data`
   and `on_flow_close`, and (e) a passing re-run of
   `verify_content_first_precedence_exhaustive` (VERIFICATION SUCCESSFUL). Oracle and
   production `classify()` must never diverge; a Rule 9 addition to one without the
   other invalidates the VP-004 proof (mirrors ADR-013 Decision 9 / BC-2.05.012
   Invariant 4).
5. **No force-fit at the analyzer layer**: dispatching non-classic-S7comm port-102
   traffic to `DispatchTarget::S7comm` (Postcondition 4) never causes `S7commAnalyzer`
   to report it as a classified S7comm session. Per ADR-014 Decision 2's
   disambiguation table and its SS-21 contracts: a COTP DT-TPDU with `protocol_id ==
   Some(0x72)` is reported only as an observed S7comm-plus session with
   `Support::DetectionOnly` catalog status (BC-2.21.024; ADR-014 Decision 3); a
   DT-TPDU with any other/unrecognized `protocol_id`, or an unparseable COTP payload,
   is left `S7Protocol::Unclassified` and continues to surface through the existing
   `(TransportProto, u16)` `unclassified_port_counts` gap mechanism (BC-2.21.027,
   BC-2.21.028) — it is never counted as S7comm-classified traffic. This invariant is
   the load-bearing correctness property ADR-014 Decision 2 names explicitly: "a COTP
   DT-TPDU on port 102 whose protocol-ID is not `0x32` or `0x72` must never be
   misattributed to S7comm."
6. **Rule ordering / no regression**: Rule 9's insertion after Rule 8 (port 2404,
   IEC-104) does not alter the outcome of Rules 1-8 for any input that does not also
   match port 102. The prior fallthrough arm (`DispatchTarget::None`) is renumbered
   Rule 10 with unchanged behavior — it still fires only when no Rule 1-9 has matched.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | src_port=102, dst_port=50000, TCP | `Some(S7comm)` |
| EC-002 | src_port=50000, dst_port=102, TCP | `Some(S7comm)` |
| EC-003 | src_port=102, dst_port=102, TCP | `Some(S7comm)` (port appears on both) |
| EC-004 | src_port=102, dst_port=any, UDP | `None` (no UDP rule, Invariant 1) |
| EC-005 | src_port=101, dst_port=103, TCP | `None` (port 102 not present; Rule 9 does not fire) |
| EC-006 | data begins with TLS signature (`0x16 0x03`), src/dst_port=102, TCP | `Some(Tls)` (Rule 1 content-first precedence wins over Rule 9; Invariant 2) |
| EC-007 | port=102, TCP, COTP DT-TPDU carries `protocol_id: Some(0x72)` (S7comm-plus) inside the delivered payload | Dispatcher still returns `Some(S7comm)` (Postcondition 4); `S7commAnalyzer` reports it as an observed S7comm-plus session only, `Support::DetectionOnly`, never a classified-S7comm finding (Invariant 5, BC-2.21.024) |
| EC-008 | port=102, TCP, COTP DT-TPDU carries an unrecognized `protocol_id` (e.g., MMS/ICCP) or an unparseable COTP payload | Dispatcher still returns `Some(S7comm)` (Postcondition 4); `S7commAnalyzer` sets `S7Protocol::Unclassified` and the flow surfaces via `unclassified_port_counts`, never as classified S7comm traffic (Invariant 5, BC-2.21.027/028) |
| EC-009 | port=2404 (IEC-104, Rule 8), TCP | `Some(Iec104)`, not `Some(S7comm)` — confirms Rule 9's insertion does not shadow or regress Rule 8 (Invariant 6) |

## Canonical Test Vectors

| src_port | dst_port | proto | data prefix | Expected |
|----------|----------|-------|-------------|----------|
| 102 | 50000 | TCP | (no TLS/HTTP signature) | `Some(S7comm)` |
| 50000 | 102 | TCP | (no TLS/HTTP signature) | `Some(S7comm)` |
| 102 | 50000 | UDP | (any) | `None` |
| 102 | 50000 | TCP | `0x16 0x03 ...` (TLS signature) | `Some(Tls)` (not Rule 9) |
| 2404 | 50000 | TCP | (no TLS/HTTP signature) | `Some(Iec104)` (Rule 8, unaffected by Rule 9 insertion) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | `classify_oracle` in `#[cfg(kani)]` includes Rule 9 atomically; oracle agrees with `classify()` for port 102 | Kani: `verify_content_first_precedence_exhaustive` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md §CAP-05 — this BC is a new TCP port-fallback classification rule extending `StreamDispatcher`'s content-first dispatch table, which is exactly what CAP-05 defines, following the same pattern as the five prior binary-ICS port-fallback rules (Modbus, DNP3, ENIP, IEC-104) already anchored to this capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-05 (`src/dispatcher.rs`); ADR-014 Decision 2 |
| ADR | ADR-014 Decision 2 (port-102 dispatch: single `DispatchTarget::S7comm`; in-analyzer disambiguation; VP-004 atomic obligation) |
| Feature | feature-s7comm |
| Stories | (TBD — F3 story decomposition) |
| MITRE Techniques | (none — dispatch rule; findings emitted in SS-21) |

## Related BCs

- BC-2.05.001..012 — composes with (existing dispatch rules; Rule 9 is additive, does not modify Rules 1-8's behavior)
- BC-2.21.001 — depends on (SS-21 `S7commFlowState` lazily created on first dispatch to this flow)
- BC-2.21.002 — depends on (SS-21 in-analyzer four-way disambiguation on COTP `protocol_id`, the direct downstream consumer of this dispatch)
- BC-2.21.024 — depends on (S7comm-plus `DetectionOnly` observed-session outcome for non-classic traffic dispatched here)
- BC-2.21.027 — depends on (unrecognized-`protocol_id` unclassified-gap outcome for non-classic traffic dispatched here)
- BC-2.21.028 — depends on (unparseable-COTP-payload unclassified-gap outcome for non-classic traffic dispatched here)
- BC-2.18.001 — composes with (SS-18 `Support::DetectionOnly`/`KnownUnsupported` per-entry catalog assignments for the four port-102 protocols, ADR-014 Decision 3)

## Architecture Anchors

- `src/dispatcher.rs` — `classify()` Rule 9: `if port == 102 && proto == Tcp { return Some(S7comm); }` (planned; not yet in src tree)
- `src/dispatcher.rs` — `#[cfg(kani)] mod kani_proofs::classify_oracle` — mirrored Rule 9 arm (VP-004 atomic obligation, planned)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — Rule 9, disambiguation table, VP-004 six-step atomic obligation
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — SS-20/SS-21 module split consumed by Rule 9's downstream analyzer

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-004 — `verify_content_first_precedence_exhaustive` (classify_oracle Rule 9 addition)
