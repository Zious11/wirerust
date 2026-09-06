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
subsystem: SS-20
capability: CAP-20
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
input-hash: "8f268fc"
---

# BC-2.20.006: `parse_cotp_header` Returns None When the Length Indicator Declares More Bytes Than Are Present (Truncated COTP Header)

## Description

The COTP Length Indicator (LI, `tpkt_payload[0]`) declares the number of header bytes
that follow the LI octet itself, excluding any upper-layer user data (ISO 8073 §13.2).
The fixed portion required to identify and fully parse a TPDU is `1 + LI` bytes total
(LI byte + LI further bytes). When `tpkt_payload.len() < 1 + LI as usize`, the declared
header is truncated — the frame is genuinely malformed or, more commonly, the TCP
segment boundary split the COTP header itself. `parse_cotp_header` returns `None` in
either case; it does not distinguish adversarial truncation from ordinary
segmentation — that distinction is the frame-walk caller's responsibility via the carry
buffer (BC-2.20.013).

## Preconditions

1. `tpkt_payload.len() >= 2` (LI + TPDU-code byte both present — BC-2.20.005 passed).
2. `tpkt_payload.len() < 1 + tpkt_payload[0] as usize` (LI declares more remaining
   header bytes than `tpkt_payload` actually contains).

## Postconditions

1. `parse_cotp_header(tpkt_payload)` returns `None`.
2. No out-of-bounds index occurs: the LI value (`tpkt_payload[0]`, a `u8`, max `255`) is
   only ever used as a bound in a length comparison, never as a direct index without a
   prior bounds check.
3. `S7commAnalyzer` treats this as an incomplete-frame condition: the TPKT frame's
   bytes are stashed to the direction's carry buffer to await the remainder of the
   segment (BC-2.20.013) — this is the ordinary, expected path for a COTP header split
   across a TCP segment boundary, not necessarily an anomaly.

## Invariants

1. **LI is a `u8`**: the Length Indicator field is exactly 1 byte (ISO 8073 §13.2), so
   its maximum declared value is `255`; `1 + LI as usize` cannot overflow `usize` on any
   supported platform.
2. **No panic on any LI value**: the comparison `tpkt_payload.len() < 1 + tpkt_payload[0] as usize`
   is total and safe for every `u8` value of `tpkt_payload[0]`, including `0` (a
   zero-length LI, itself a malformed/degenerate COTP header handled by the TPDU-type
   recognition paths, since `1 + 0 = 1 <= tpkt_payload.len()` is trivially satisfied by
   the BC-2.20.005 precondition and this path is not reached).
3. **Ordinary-segmentation vs. adversarial ambiguity is not resolved here**: this
   function has no notion of "this is the Nth retry" or a byte budget; that state lives
   entirely in `S7commFlowState`'s carry buffer (SS-21), consistent with SS-20 being
   stateless (ADR-014 Decision 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tpkt_payload = [0x06, 0xE0, 0x00, 0x01]` (LI=6, declares 6 more bytes, only 3 present) | Returns `None` — truncated CR/CC header |
| EC-002 | `tpkt_payload = [0x02, 0xF0]` (LI=2, declares 2 more bytes, exactly 1 present after LI+code) | Returns `None` — truncated DT header (TPDU-NR byte missing) |
| EC-003 | `tpkt_payload = [0x00, 0xF0]` (LI=0 — degenerate, but `1+0=1 <= len=2`) | Not truncated by this check; proceeds to TPDU-type recognition (an LI=0 DT frame is handled by BC-2.20.009/010's own bounds logic) |
| EC-004 | This condition arising mid-TCP-stream because the segment boundary split exactly between the LI octet and the TPDU-code byte | Same `None` result as an adversarial truncation — the carry buffer (BC-2.20.013) reassembles across the boundary transparently on the next `on_data` call |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x06, 0xE0, 0x00, 0x01]` (LI=6, only 3 following bytes) | `None` | reject: truncated CR header |
| `[0x02, 0xF0]` (LI=2, only 1 following byte) | `None` | reject: truncated DT header |
| `[0x02, 0xF0, 0x80]` (LI=2, exactly 2 following bytes) | proceeds to DT recognition | accept-boundary — see BC-2.20.009 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all `tpkt_payload` where `len < 1 + LI`, `parse_cotp_header` returns `None`; no panic or out-of-bounds access for any `u8` LI value | Kani P0 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the LI-truncation rejection path, the primary malformed/truncated-COTP-frame error case named in the F2 authoring scope |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decisions 1, 4, 8 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none directly — truncation is handled as incomplete-frame carry, not an anomaly finding, unless the carry bound is exceeded; see BC-2.20.014) |

## Related BCs

- BC-2.20.005 — composes with (2-byte minimum-prefix reject path, evaluated before this check)
- BC-2.20.007 — composes with (CR TPDU recognition, once LI bound passes)
- BC-2.20.008 — composes with (CC TPDU recognition)
- BC-2.20.009 — composes with (DT TPDU recognition)
- BC-2.20.013 — depends on (carry-buffer stash on truncation)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header`: `let li = tpkt_payload[0] as usize; if tpkt_payload.len() < 1 + li { return None }`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 4` — ISO/IEC 8073:1997 §13.2 Length Indicator field semantics

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst per ADR-014 Decision 9,
anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — Kani P0 target |
