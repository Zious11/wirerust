---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.5.0-feature-008
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/phase-f2-spec-evolution/dnp3-verification-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/vp-023-dnp3-parse-safety.md
input-hash: TBD
---

# BC-2.15.002: DNP3 DL Header Rejected for Frame Shorter Than 10 Bytes (Truncation Safety)

## Description

`parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>` returns `None` when the input
slice is shorter than 10 bytes. This is the truncation-safety reject contract complementary to
BC-2.15.001. The 10-byte minimum requirement corresponds to the full DNP3 data-link header (8
header octets + 2 header-CRC octets). No partial parse is performed; no struct fields are
populated. The function never panics for any input length, including zero-length slices.

## Preconditions

1. `data` is a `&[u8]` slice of any length, including length 0.
2. `data.len() < 10` — the slice is shorter than the minimum complete DNP3 link-layer header.

## Postconditions

1. `parse_dnp3_dl_header(data)` returns `None`.
2. No `Dnp3DlHeader` struct is created.
3. The function does not panic, does not perform out-of-bounds indexing, and does not
   mutate any state.
4. No finding is emitted (pure function, no side effects).

## Invariants

1. **Strict length guard**: the function returns `None` if and only if `data.len() < 10`.
   The converse (`data.len() >= 10` implies `Some`) is specified in BC-2.15.001.
2. **No partial parse**: when the length guard fails, no indexing into `data` occurs. The
   implementation uses an early `if data.len() < 10 { return None; }` guard — verified by
   Kani Sub-property A.
3. **Panic freedom**: the function never panics for any `&[u8]` input, regardless of length.
   Kani Sub-property A proves this over all symbolic lengths 0..=12. [VP-023 Sub-A]

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero-length input `&[]` | Returns `None` — no indexing; no panic |
| EC-002 | 1-byte input (only START1) | Returns `None` — truncated; no panic |
| EC-003 | 9-byte input (header missing last CRC byte) | Returns `None` — one byte short of minimum |
| EC-004 | Exactly 10 bytes | Returns `Some(...)` — this is the BC-2.15.001 boundary; NOT this BC's postcondition |
| EC-005 | Input with valid sync bytes but length < 10 | Returns `None` — length check precedes any byte inspection |

## Canonical Test Vectors

| Input (hex) | Expected Result | Category |
|-------------|----------------|----------|
| `<empty>` (0 bytes) | `None` | edge-case: zero-length |
| `05` (1 byte) | `None` | edge-case: single byte |
| `05 64 05 C0 01 00 03 00 A1` (9 bytes) | `None` | edge-case: one byte short of minimum; valid sync/content but truncated |
| `05 64 05 C0 01 00 03 00 A1 B2` (10 bytes) | `Some(...)` | boundary: exactly 10 bytes returns `Some` (BC-2.15.001 happy path) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property A: `parse_dnp3_dl_header` returns `None` for `data.len() < 10`; never panics for any input | Kani: symbolic `[u8; 12]` + symbolic `len <= 12`; asserts `parsed.is_none()` when `len < 10` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — this BC defines the truncation-safety reject postcondition for the DNP3 header parser, which is essential for memory-safe processing of incomplete TCP segments in the ICS/OT analyzer capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — panic freedom and truncation safety prevent false findings on non-DNP3 traffic routed via port 20000) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23 Dnp3Analyzer); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — pure parse reject path; no finding emission) |

## Related BCs

- BC-2.15.001 — composes with (happy-path `Some` return for `data.len() >= 10`)
- BC-2.15.003 — composes with (LE field decode is only reached when `Some` path fires)
- BC-2.15.004 — depends on (validity gate is only called after `parse_dnp3_dl_header` returns `Some`)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>` — early `if data.len() < 10 { return None; }` guard
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §3` — "Returns None if data.len() < 10 (truncated frame). Never panics for any input."
- `.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md` — Sub-property A: "(A.1) `len<10` => `None`"

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — DNP3 Data-Link Frame Parse Safety and Function-Code Classification (Sub-property A: truncation safety, `None` path)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-architecture-delta.md §3; vp-023-dnp3-parse-safety.md Sub-property A; dnp3-research.md §1.1 |
| **Confidence** | high — the 10-byte minimum is fixed by IEEE 1815-2012 header layout (8 header + 2 CRC octets) |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same length always produces same `None` result |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-023 Kani target (Sub-A, `None` path) |
