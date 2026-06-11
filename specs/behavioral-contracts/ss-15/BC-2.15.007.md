---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified:
  - "v1.1: Pass-1 adversarial fix I-1: EC-004 corrected from Some(12) to Some(13) — correct arithmetic is U=1, blocks=ceil(1/16)=1, frame_len=5+6+2*1=13. Removed embedded exploratory-chain prose ('...wait:') from EC-004 cell. — 2026-06-10"
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

# BC-2.15.007: compute_dnp3_frame_len Arithmetic Correct; Result in [10,292]; No Overflow

## Description

`compute_dnp3_frame_len(length: u8) -> Option<usize>` computes the total on-wire size of a
DNP3 link frame from its LENGTH field. It returns `None` for `length < 5` (invalid frame) and
`Some(frame_len)` for `length` in 5..=255, where
`frame_len = 5 + length + 2 * ceil((length - 5) / 16)`. The result is always in [10, 292],
never overflows `usize`, and never panics. This is VP-023 Sub-property D: the only sub-property
unique to DNP3 (no Modbus equivalent), targeting the CRC-block-walk arithmetic that determines
frame consumption boundaries in the carry buffer loop.

## Preconditions

1. `length` is any `u8` value (0x00..=0xFF, all 256 possible values).
2. No other preconditions; the function is total over `u8`.

## Postconditions

**Invalid LENGTH (below minimum):**
1. `compute_dnp3_frame_len(length)` returns `None` when `length < 5` (i.e., `length` in 0..=4).
   [SPEC: minimum LENGTH = 5 per Chipkin AN2013-004b, dnp3-research.md §1.1]

**Valid LENGTH:**
2. For all `length` in 5..=255, returns `Some(frame_len)` where:
   ```
   num_user_octets = (length as usize) - 5
   num_data_blocks = (num_user_octets + 15) / 16   // integer ceil(U/16)
   frame_len       = 5 + (length as usize) + 2 * num_data_blocks
   ```
   This is the canonical formula from ADR-007 Decision 2 and dnp3-research.md §1.3. [SPEC]
3. Result bounds invariant: `frame_len >= 10` for all `length >= 5` (minimum: LENGTH=5 → 10 bytes).
4. Result bounds invariant: `frame_len <= 292` for all `length <= 255` (maximum: LENGTH=255 → 292 bytes). [SPEC: dnp3-research.md §1.4]
5. No integer overflow on any `usize` platform (maximum intermediate value is `5 + 255 + 32 = 292`, well within any 16-bit+ `usize`).
6. The function NEVER panics for any `u8` input.

## Invariants

1. **Formula derivation** [SPEC: dnp3-research.md §1.3]:
   - `num_user_octets = LENGTH - 5` (subtract CONTROL(1)+DEST(2)+SOURCE(2)=5)
   - `num_data_blocks = ceil(U / 16)` — each block is 16 user octets + 2 CRC octets
   - `frame_len = 3 (sync+len) + LENGTH + 2 (hdr CRC) + 2*blocks`
   - Simplified: `frame_len = 5 + LENGTH + 2 * ceil((LENGTH-5)/16)`
2. **Minimum (LENGTH=5)**: `U=0`, `blocks=0`, `frame_len = 5+5+0 = 10 bytes`. This is a
   link-control frame with no user data (CONTROL+DEST+SOURCE only). [SPEC: ADR-007 Decision 2]
3. **Maximum (LENGTH=255)**: `U=250`, `blocks=ceil(250/16)=16`, `frame_len = 5+255+32 = 292 bytes`.
   [SPEC: dnp3-research.md §1.4 "maximum length link layer frame is 292 octets"]
4. **Integer ceil formula**: `(U + 15) / 16` computes ceiling division without floating point.
   This is the canonical pattern (same as integer division ceiling in Rust:
   `(num_user_octets + 15) / 16`). [VP-023 Sub-D proof uses this form]
5. **No overflow path**: maximum value in the formula is `5 + 255 + 2*16 = 292`. On any
   platform where `usize >= 16 bits`, this cannot overflow. The Kani proof on `u8` inputs
   certifies this over the complete domain.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `length = 0` | `None` — below minimum 5 |
| EC-002 | `length = 4` | `None` — below minimum 5 (one below minimum) |
| EC-003 | `length = 5` | `Some(10)` — minimum valid frame: `U=0, blocks=0, frame_len=10` |
| EC-004 | `length = 6` | `Some(13)` — `U=1, blocks=1, frame_len=5+6+2*1=13` |
| EC-005 | `length = 21` | `Some(28)` — `U=16, blocks=1, frame_len=5+21+2=28` — exactly one full 16-octet data block |
| EC-006 | `length = 22` | `Some(31)` — `U=17, blocks=ceil(17/16)=2, frame_len=5+22+4=31` — starts second block |
| EC-007 | `length = 255` | `Some(292)` — maximum valid frame |
| EC-008 | `length = 254` | `Some(290)` — `U=249, blocks=ceil(249/16)=16, frame_len=5+254+32=291`... recalc: `ceil(249/16)=ceil(15.5625)=16`, `5+254+32=291`. Correct: `Some(291)`. |

**Boundary verification for EC-004 through EC-008:**
- LENGTH=6: `U=1, blocks=ceil(1/16)=1, frame_len=5+6+2=13`
- LENGTH=21: `U=16, blocks=ceil(16/16)=1, frame_len=5+21+2=28`
- LENGTH=22: `U=17, blocks=ceil(17/16)=2, frame_len=5+22+4=31`
- LENGTH=37: `U=32, blocks=ceil(32/16)=2, frame_len=5+37+4=46`
- LENGTH=254: `U=249, blocks=ceil(249/16)=16, frame_len=5+254+32=291`
- LENGTH=255: `U=250, blocks=ceil(250/16)=16, frame_len=5+255+32=292`

## Canonical Test Vectors

| `length` | Formula steps | Expected `frame_len` | Category |
|----------|--------------|---------------------|----------|
| 0x00 (0) | `None` (length < 5) | `None` | edge-case: zero |
| 0x04 (4) | `None` (length < 5) | `None` | edge-case: one below minimum |
| 0x05 (5) | `U=0, blocks=0, fl=5+5+0=10` | `Some(10)` | happy-path: minimum valid |
| 0x0E (14) | `U=9, blocks=ceil(9/16)=1, fl=5+14+2=21` | `Some(21)` | happy-path: typical DIRECT_OPERATE (14-byte LENGTH) |
| 0x15 (21) | `U=16, blocks=1, fl=5+21+2=28` | `Some(28)` | happy-path: exactly one 16-octet block |
| 0x16 (22) | `U=17, blocks=2, fl=5+22+4=31` | `Some(31)` | happy-path: first byte of second block |
| 0xFF (255) | `U=250, blocks=16, fl=5+255+32=292` | `Some(292)` | happy-path: maximum frame |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property D: `compute_dnp3_frame_len(length)` returns `None` for `length<5`; returns `Some(formula)` for `length>=5`; result in [10,292]; no overflow/panic over all 256 `u8` values | Kani: `length: u8 = kani::any()`; biconditional assertions; formula comparison; bounds check |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — the frame_len arithmetic determines the carry-buffer consumption boundary in `Dnp3Analyzer::on_data`; an incorrect formula causes over-read (out-of-bounds panic) or under-read (frame misalignment and missed detections); this is the safety foundation for all frame-parsing in the DNP3/ICS analyzer capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — carry-buffer safety prevents panic on adversarial/malformed DNP3 traffic arriving on port 20000) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-23); ADR-007 Decision 2 (frame_len formula), Decision 3 (CRC-block-skip) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — pure arithmetic function; no finding emission) |

## Related BCs

- BC-2.15.004 — depends on (frame_len is only computed after validity gate returns `true`)
- BC-2.15.001 — composes with (the carry buffer in `on_data` uses `frame_len` to determine how many bytes constitute a complete frame to parse)
- BC-2.15.020 — depends on (carry buffer management BC references the `frame_len` boundary computed here)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn compute_dnp3_frame_len(length: u8) -> Option<usize>` pure core function
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §3` — exact formula and signature
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 2` — formula derivation and 292-byte maximum confirmation
- `.factory/research/dnp3-research.md §1.3` — CRC-block layout and formula; §1.4 — 292-byte maximum [SPEC]
- `.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md` — Sub-property D Kani skeleton

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — DNP3 Data-Link Frame Parse Safety and Function-Code Classification (Sub-property D: frame_len arithmetic)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-research.md §1.3 (formula); §1.4 (292 max confirmed: "maximum length link layer frame is 292 octets"); ADR-007 Decision 2; dnp3-architecture-delta.md §3 |
| **Confidence** | high — formula is SPEC-confirmed [SPEC] from dnp3-research.md §1.3 (DNP Users Group Primer Rev A + Chipkin AN2013-004b); 292 maximum confirmed verbatim [SPEC] |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same LENGTH always produces same `frame_len` |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-023 Kani target (Sub-D, arithmetic) |
