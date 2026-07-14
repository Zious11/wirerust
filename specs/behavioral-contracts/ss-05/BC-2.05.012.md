---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-05
capability: CAP-05
lifecycle_status: active
introduced: feature-iec104
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-delta-analysis.md
input-hash: "8b69772"
---

# BC-2.05.012: `classify()` Rule 8 — TCP Port 2404 Returns DispatchTarget::Iec104

## Description

`StreamDispatcher::classify(src_port, dst_port, TransportProto::Tcp)` returns
`Some(DispatchTarget::Iec104)` when either `src_port == 2404` or `dst_port == 2404`
and the transport is TCP. This is Rule 8 in the port-fallback classification table,
added alongside the IEC 60870-5-104 analyzer (ADR-013 Decision 1). Port 2404 is the
IANA-assigned port for IEC-104 (RFC reference: IEC 60870-5-104 §5.1). The rule fires
only for TCP; IEC-104 is TCP-only and there is no UDP variant.

## Preconditions

1. `classify(src_port, dst_port, TransportProto::Tcp)` is called.
2. At least one of `src_port == 2404` or `dst_port == 2404` is true.
3. The IEC-104 analyzer has been instantiated by the caller (gated by `--iec104`/`--all` flags per BC-2.12.025); `classify()` itself is flag-agnostic — it is a pure port-mapping function.

## Postconditions

1. Returns `Some(DispatchTarget::Iec104)`.
2. The flow is routed to `Iec104Analyzer::on_data`.
3. No other `DispatchTarget` variant is returned for TCP/2404.

## Invariants

1. **TCP-only**: `classify(2404, any, TransportProto::Udp)` returns `None` (no UDP IEC-104 rule).
2. **Content-first precedence**: if a higher-priority content-based rule fires before Rule 8, that result takes precedence (ADR-013 Decision 1 / ADR-0001).
3. **Rule 8 is port-fallback only**: Rule 8 fires only when no content-based rule matched first.
4. **VP-004 oracle**: adding Rule 8 requires an atomic update to `classify_oracle` in `#[cfg(kani)]` (ADR-013 Decision 9).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | src_port=2404, dst_port=12345, TCP | `Some(Iec104)` |
| EC-002 | src_port=12345, dst_port=2404, TCP | `Some(Iec104)` |
| EC-003 | src_port=2404, dst_port=2404, TCP | `Some(Iec104)` (port appears on both) |
| EC-004 | src_port=2404, dst_port=any, UDP | `None` (no UDP rule) |
| EC-005 | src_port=2403, dst_port=2405, TCP | `None` (port 2404 not present) |

## Canonical Test Vectors

| src_port | dst_port | proto | Expected |
|----------|----------|-------|----------|
| 2404 | 50000 | TCP | `Some(Iec104)` |
| 50000 | 2404 | TCP | `Some(Iec104)` |
| 2404 | 50000 | UDP | `None` |
| 8080 | 80 | TCP | `Some(Http)` (not Rule 8) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | `classify_oracle` in `#[cfg(kani)]` includes Rule 8 atomically; oracle agrees with `classify()` for port 2404 | Kani: `verify_content_first_precedence_exhaustive` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Protocol Classification and Flow Dispatch") per ARCH-INDEX.md §SS-05 |
| Capability Anchor Justification | CAP-05 ("Protocol Classification and Flow Dispatch") per ARCH-INDEX.md §SS-05 — Rule 8 is a new TCP port-fallback classification rule that extends the dispatch table with IEC-104 support |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-05 (src/dispatcher.rs); ADR-013 Decision 1 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — dispatch rule; findings in SS-19) |

## Related BCs

- BC-2.05.001..011 — composes with (existing dispatch rules; Rule 8 is additive)
- BC-2.19.001 — depends on (SS-19 IEC-104 analysis starts after this dispatch)
- BC-2.12.025 — depends on (--iec104 flag enables this rule)

## Architecture Anchors

- `src/dispatcher.rs` — `classify()` Rule 8: `if port == 2404 && proto == Tcp { return Some(Iec104); }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 1` — Rule 8
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 9` — VP-004 oracle obligation

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-004 — `verify_content_first_precedence_exhaustive` (classify_oracle Rule 8 addition)
