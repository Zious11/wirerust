---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: ["v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.016: C0 Boundary: 0x1F Trips Finding; 0x20 (Space) Does NOT

## Description

The `contains_c0_or_del` function checks `b < 0x20 || b == 0x7f`. The condition
`b < 0x20` covers 0x00-0x1F (C0 range) and `b == 0x7f` covers DEL. The byte 0x20
(space, ASCII 32) does NOT satisfy `b < 0x20`, so it does not trigger arm 2. The
boundary is: 0x1F (31, the last C0 byte) trips the finding; 0x20 (32, space) does not.

This BC defines the precise predicate boundary for the AsciiWithControl arm.

## Preconditions

1. `contains_c0_or_del` is called on a pure-ASCII string (invariant of the helper).
2. The string contains the byte 0x1F or 0x7F (trigger), OR the byte 0x20 (no trigger).

## Postconditions

1. If the SNI contains 0x1F (US, Unit Separator) or any byte 0x00-0x1E, arm 2 fires.
2. If the SNI contains 0x7F (DEL), arm 2 fires.
3. If the SNI contains 0x20 (space) and no other C0/DEL bytes, arm 1 fires (no finding).
4. The boundary test `b < 0x20` is exact: 0x1F < 0x20 is true; 0x20 < 0x20 is false.

## Invariants

1. The predicate is `b < 0x20 || b == 0x7f` -- a bitwise disjunction of two conditions.
2. C1 control characters (0x80-0x9F) are NOT checked here; they would only appear
   in non-ASCII UTF-8 strings which are already caught by arm 3.
3. Tab (0x09), LF (0x0A), CR (0x0D) are all C0 bytes and all trip the finding.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI contains 0x1F only (last C0) | Arm 2; finding emitted |
| EC-002 | SNI contains 0x20 only (space) | Arm 1; no finding |
| EC-003 | SNI = "a\x00b" (NUL, start of C0 range) | Arm 2; finding emitted |
| EC-004 | SNI = "a\x7fb" (DEL) | Arm 2; finding emitted |
| EC-005 | SNI = "a\x7eb" (0x7E, tilde -- not DEL) | Arm 1; no finding |
| EC-006 | SNI = "a\x21b" (0x21, ! -- above space) | Arm 1; no finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI = "test\x1fend" (0x1F) | Finding emitted (arm 2) | happy-path |
| SNI = "test\x20end" (0x20 space) | No finding (arm 1) | happy-path |
| SNI = "test\x7fend" (DEL) | Finding emitted (arm 2) | edge-case |
| SNI = "test\x7eend" (0x7E tilde) | No finding (arm 1) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-005 | 0x1F trips finding; 0x20 does not | unit: test_ascii_control_boundary_bytes |
| VP-005 | DEL (0x7F) trips finding | unit: test_ascii_sni_with_del |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- precise predicate boundary definition is load-bearing for SNI classification accuracy |
| L2 Domain Invariants | INV-5 (SNI 4-way classification ordered match) |
| Architecture Module | SS-07 (analyzer/tls.rs:231-238, C-13) |
| Stories | STORY-055 |
| Origin BC | BC-TLS-016 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.014 -- depends on (arm 2 definition)
- BC-2.07.013 -- related to (arm 1 gets 0x20 and above)

## Architecture Anchors

- `src/analyzer/tls.rs:231-238` -- `contains_c0_or_del` function
- `tests/tls_analyzer_tests.rs` -- test_ascii_control_boundary_bytes

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:231-238` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `s.bytes().any(|b| b < 0x20 || b == 0x7f)`
- **assertion**: test_ascii_control_boundary_bytes

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (pure function) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (pure function) |
| **Overall classification** | pure |
