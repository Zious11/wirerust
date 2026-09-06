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
subsystem: SS-21
capability: CAP-21
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

# BC-2.21.005: `parse_s7comm_header` Defensively Rejects `data[0] != 0x32`

## Description

`S7commAnalyzer` only calls `parse_s7comm_header` after `parse_cotp_header` has
already reported `protocol_id == Some(0x32)` (BC-2.21.002 Postcondition 3), so
`data[0]` is expected to always equal `0x32` in production. `parse_s7comm_header`
nonetheless independently re-checks `data[0] == 0x32` and returns `None` if it does
not — a defense-in-depth guard against caller-side drift (e.g. a future refactor that
accidentally routes a `0x72` or other-protocol-ID frame into this function) rather
than a condition expected to occur via any wire input alone. This is a pure-function
hygiene contract, not a wire-format edge case.

## Preconditions

1. `data.len() >= 10` (BC-2.21.004's length guard has passed).
2. `data[0] != 0x32`.

## Postconditions

1. `parse_s7comm_header(data)` returns `None`.
2. No other bytes are accessed once `data[0]` fails the equality check.
3. No `Finding` is emitted for this path — this is a defensive programming contract,
   not a wire-observable anomaly (a real `0x32`-prefixed frame reaching this function
   is the analyzer's own responsibility to guarantee via BC-2.21.002's dispatch).

## Invariants

1. **Redundant-by-design**: this check can never fire given a correctly implemented
   BC-2.21.002 dispatch; it exists purely so `parse_s7comm_header` is independently
   correct and testable as a standalone unit, matching the pure-core free-fn
   philosophy (ADR-014 Decision 9) of not trusting caller invariants implicitly.
2. **No finding on this path**: distinguishes this defensive-only reject from
   BC-2.21.004's wire-observable malformed-length reject.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data[0] == 0x72` (would only occur via a caller defect, since `0x72` frames are dispatched to BC-2.21.024's path, not this function) | Returns `None`, no finding |
| EC-002 | `data[0] == 0x00` | Returns `None`, no finding |

## Canonical Test Vectors

| Input (`data[0]`) | Expected result | Category |
|---|---|---|
| `0x32` | Proceeds to ROSCTR parsing (BC-2.21.006) | happy-path |
| `0x72` | `None` | defensive reject (unit-test only; not reachable via BC-2.21.002 dispatch) |
| `0x00` | `None` | defensive reject |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `parse_s7comm_header` returns `None` for any `data[0] != 0x32` given `data.len() >= 10` | cargo-fuzz P1 (combined harness, ADR-014 Decision 9) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — defensive guard on the classic S7comm dissection entry function |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none) |

## Related BCs

- BC-2.21.004 — composes with (length-reject path precedes this check)
- BC-2.21.006 — composes with (accept path when `data[0] == 0x32`)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `fn parse_s7comm_header`, defensive `data[0] == 0x32` guard

## Story Anchor

STORY-187

## VP Anchors

(None dedicated — not in VP-051's registered source_bc {BC-2.21.004, BC-2.21.009}.
No-panic behavior on this defensive-reject path is expected to be exercised
generically by VP-055's combined fuzz harness, but no individual VP-NNN forward-anchor
is registered for this specific BC in VP-INDEX.md v2.48.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — cargo-fuzz P1 target |
