---
document_type: behavioral-contract
level: L3
version: "1.3"
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
introduced: v0.6.0-feature-008
modified:
  - "v1.3: F3 story-anchor back-fill. — 2026-06-14"
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

# BC-2.15.004: Three-Point Validity Gate Returns True Iff Sync==0x0564 and LENGTH>=5

## Description

`is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool` implements the post-classification
three-point validity gate. It returns `true` if and only if all three conditions hold:
(1) `h.start1 == 0x05`, (2) `h.start2 == 0x64`, and (3) `h.length >= 5`. This gate is the
compensating control for DNP3's port-only classification (ADR-007 Decision 1): it prevents
false findings from non-DNP3 binary traffic on port 20000. The gate never panics; it reads
only struct fields. Frames failing the gate increment `parse_errors` and are discarded without
emitting findings.

## Preconditions

1. `h` is a `Dnp3DlHeader` struct previously produced by `parse_dnp3_dl_header(data)` returning
   `Some(h)` (BC-2.15.001).
2. The gate is called on the parsed struct (struct fields), not on raw bytes; no additional
   slice bounds checks are needed.

## Postconditions

1. `is_valid_dnp3_frame_header(h)` returns `true` if and only if ALL three conditions hold:
   - `h.start1 == 0x05` (DNP3 START1 sync byte) [SPEC: dnp3-research.md §1.1]
   - `h.start2 == 0x64` (DNP3 START2 sync byte) [SPEC: dnp3-research.md §1.1]
   - `h.length >= 5` (minimum valid LENGTH; minimum frame carries CONTROL+DEST+SOURCE only) [SPEC: dnp3-research.md §1.1]
2. The function returns `false` if ANY of the three conditions fails.
3. The function never panics for any symbolic `Dnp3DlHeader` input.
4. **Caller behavior when gate returns `false`**: the `on_data` caller MUST increment
   `flow.parse_errors` and skip all further processing of this frame (no transport/app parse,
   no finding emission) — see ADR-007 Decision 2 "is_non_dnp3 desync-safe bail."

## Invariants

1. **Biconditional** (VP-023 Sub-property C): the gate is exactly `start1 == 0x05 && start2 == 0x64 && length >= 5`. No additional conditions (link-FC plausibility is a *recommended* fourth
   gate in ADR-007 text but is NOT part of this pure function contract; it may be added in
   `on_data` logic without changing this BC). The Kani proof asserts biconditional equivalence.
2. **LENGTH minimum 5** [SPEC: Chipkin AN2013-004b; dnp3-research.md §1.1]: a frame with LENGTH < 5
   is invalid; the minimum is CONTROL(1)+DEST(2)+SOURCE(2)=5. LENGTH=0..=4 frames are
   structurally impossible per IEEE 1815-2012 and are rejected here.
3. **`start1 == 0x05 AND start2 == 0x64`** is the canonical 2-byte sync word `0x0564`. Both
   bytes must match; a partial match (e.g., `start1 == 0x05` but `start2 != 0x64`) returns
   `false`. [SPEC]
4. **No side effects**: the gate function is pure — it reads only `h`; it does not modify flow
   state, emit findings, or call `on_data`. Side effects (parse_errors increment) are the
   caller's responsibility.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `start1=0x05, start2=0x64, length=5` (minimum valid frame) | `true` — all three conditions met |
| EC-002 | `start1=0x05, start2=0x64, length=255` (maximum valid LENGTH) | `true` — all conditions met |
| EC-003 | `start1=0x05, start2=0x64, length=4` (LENGTH below minimum) | `false` — condition 3 fails |
| EC-004 | `start1=0x05, start2=0x64, length=0` | `false` — LENGTH 0 is below minimum 5 |
| EC-005 | `start1=0x05, start2=0x63` (wrong START2) | `false` — condition 2 fails |
| EC-006 | `start1=0x04, start2=0x64` (wrong START1) | `false` — condition 1 fails |
| EC-007 | `start1=0x00, start2=0x00, length=0` | `false` — all three conditions fail |
| EC-008 | `start1=0x64, start2=0x05` (bytes swapped) | `false` — START1 must be 0x05 not 0x64 |
| EC-009 | `start1=0x05, start2=0x64, length=1` | `false` — LENGTH=1 < 5 |

## Canonical Test Vectors

| Struct field values | Expected result | Category |
|--------------------|----------------|----------|
| `{start1:0x05, start2:0x64, length:0x0E, control:0xC4, dest:0x0003, src:0x0001}` | `true` | happy-path: valid DIRECT_OPERATE header (length=14) |
| `{start1:0x05, start2:0x64, length:0x05, control:0xC0, dest:0x0001, src:0x0001}` | `true` | happy-path: minimum-valid link-control frame |
| `{start1:0x05, start2:0x64, length:0xFF, control:0x44, dest:0x0003, src:0x0001}` | `true` | happy-path: maximum LENGTH=255 |
| `{start1:0x05, start2:0x64, length:0x04, control:0xC4, dest:0x0003, src:0x0001}` | `false` | edge-case: LENGTH=4 below minimum |
| `{start1:0x05, start2=0x63, length:0x0E, control:0xC4, dest:0x0003, src:0x0001}` | `false` | edge-case: wrong START2 (0x63 not 0x64) |
| `{start1:0x00, start2:0x00, length:0x10, control:0x00, dest:0x0000, src:0x0000}` | `false` | edge-case: zero-filled (non-DNP3 traffic on port 20000) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property C: `is_valid_dnp3_frame_header(&h) == (h.start1==0x05 && h.start2==0x64 && h.length>=5)` for all symbolic `Dnp3DlHeader` inputs | Kani: symbolic `Dnp3DlHeader` fields (all fields `kani::any()`); direct biconditional assertion |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — the three-point validity gate is the compensating control for port-only DNP3 classification (ADR-007 Decision 1), preventing false ICS findings from non-DNP3 binary traffic on port 20000; it is a prerequisite for every detection BC in the DNP3/ICS analyzer capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — the validity gate ensures port-classified DNP3 flows do not produce findings unless the frame is structurally valid DNP3) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 2 (three-point validity gate) |
| Stories | STORY-106 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — pure gate function; no finding emission) |

## Related BCs

- BC-2.15.001 — depends on (gate receives `Dnp3DlHeader` returned by `parse_dnp3_dl_header`)
- BC-2.15.002 — composes with (gate is only called when `parse_dnp3_dl_header` returns `Some`)
- BC-2.15.007 — composes with (`compute_dnp3_frame_len` is only called after gate returns `true`)
- BC-2.15.009 through BC-2.15.022 — all detection BCs depend on this gate as a precondition

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool` pure core function
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §3` — "Three-point validity gate: true iff start1==0x05 && start2==0x64 && length>=5"
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 2` — "Three-point post-classification validity gate"
- `.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md` — Sub-property C biconditional assertion

## Story Anchor

STORY-106

## VP Anchors

- VP-023 — DNP3 Data-Link Frame Parse Safety and Function-Code Classification (Sub-property C: validity gate biconditional)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 2; dnp3-research.md §1.1 (LENGTH semantics, minimum 5); dnp3-architecture-delta.md §3 |
| **Confidence** | high — sync word 0x0564 and LENGTH minimum 5 are SPEC-confirmed; gate biconditional is an architectural decision (ADR-007) |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same struct fields always produce same bool |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-023 Kani target (Sub-C) |
