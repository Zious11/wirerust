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

# BC-2.20.015: Resync Anchor Advances Exactly 1 Byte Per Iteration on a Bad TPKT Version Byte (Never 2)

## Description

When the frame-walk loop (BC-2.20.013) encounters a byte sequence whose leading byte is
not `0x03` (BC-2.20.002's reject condition, or the aftermath of a carry-overflow clear,
BC-2.20.014), it must search forward for the next plausible frame start. Per ADR-014
Decision 8, the resync advance is **exactly 1 byte per iteration, never 2** — advancing
2 bytes on a bad version byte risks skipping a real `0x03` at the very next offset,
silently losing a legitimate frame. This mirrors IEC-104's identical resync-anchor
correction (ADR-013 Decision 3 / BC-2.19.002 Postcondition 3, F-P2-L2 remediation),
applied here to the TPKT version byte instead of IEC-104's `0x68` start byte.

## Preconditions

1. The frame-walk loop has determined the current cursor position does not begin a
   valid, parseable TPKT frame (`parse_tpkt_header` returned `None` due to a bad
   version byte, or a carry-overflow clear per BC-2.20.014 has just occurred).

## Postconditions

1. The cursor advances by exactly 1 byte.
2. `parse_tpkt_header` is re-invoked at the new cursor position.
3. This repeats until either: (a) a byte equal to `0x03` is found and
   `parse_tpkt_header` succeeds from that offset, or (b) the remaining bytes are
   exhausted (fewer than 4 bytes remain), at which point the remaining bytes are
   stashed to carry per the ordinary incomplete-frame path (BC-2.20.013).
4. No frame start is ever skipped: for any byte sequence containing at least one valid
   `0x03`-anchored frame, the 1-byte resync walk is guaranteed to reach it (assuming it
   is not itself preceded by an already-consumed cursor position).

## Invariants

1. **1-byte advance is the maximum safe step size**: since a real TPKT frame could
   legitimately start at any byte offset (TPKT has no length-prefixed self-description
   before the version byte itself), any advance greater than 1 byte can skip a valid
   frame start. This is a correctness requirement, not a performance trade-off.
2. **Termination guarantee**: because the byte sequence is finite and the cursor
   strictly increases by 1 each iteration, the resync walk always terminates (either by
   finding a candidate or by exhausting the input) — no infinite loop.
3. **Reused verbatim for both trigger conditions**: the same 1-byte resync logic
   handles both an ordinary bad-version-byte reject (BC-2.20.002) encountered
   mid-stream and the post-carry-overflow resync (BC-2.20.014) — there is exactly one
   resync implementation, not two.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Bytes `[0x01, 0x03, 0x00, 0x00, 0x04]` — a spurious `0x01` immediately followed by a valid frame starting at offset 1 | 1-byte advance finds the valid frame at offset 1; a 2-byte advance would have skipped it entirely (landing at offset 2, `0x00`) |
| EC-002 | A long run of non-`0x03` garbage bytes (e.g. 200 bytes) followed by a valid frame | The resync walk advances 1 byte at a time through all 200 garbage bytes before finding the valid frame; no upper limit on resync-walk iterations within a single `on_data` call other than the input length itself |
| EC-003 | No `0x03` byte exists anywhere in the remaining input | The walk advances to the end of the input; the (now-empty or sub-4-byte) remainder is stashed to carry per BC-2.20.013's ordinary incomplete-frame path |
| EC-004 | A `0x03` byte exists but is itself the start of a frame with an invalid length field (`< 4`, BC-2.20.003) | The resync walk finds this `0x03` and attempts `parse_tpkt_header`, which returns `None` again (different reject reason); the walk continues advancing 1 byte past this `0x03` as well — it does not get stuck retrying the same offset |

## Canonical Test Vectors

| Scenario | Input | Expected Behavior | Category |
|----------|-------|--------------------|---------|
| Adjacent spurious-then-valid | `[0x01, 0x03, 0x00, 0x00, 0x04]` | Frame found at offset 1 (`Some(TpktHeader{version:3, length:4})`); 0 bytes lost | legit: minimal resync distance |
| Long garbage run | `[0xAA; 50]` followed by `[0x03, 0x00, 0x00, 0x04]` | Frame found at offset 50; all 50 garbage bytes consumed one at a time | legit: extended resync |
| No valid anchor present | `[0xAA; 10]` (no `0x03` anywhere) | Walk exhausts all 10 bytes; nothing stashed to carry beyond what's left after the walk (here, nothing, since none of the 10 bytes form a parseable ≥4-byte remainder containing `0x03`) | non-conformant: no frame found |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| The resync walk advances exactly 1 byte per iteration (never 2 or more); for any byte sequence containing a `0x03`-anchored valid frame at offset `k`, the walk reaches offset `k` without skipping it | proptest P1 (per ADR-014 Decision 8/9, mirroring IEC-104's equivalent resync-distance property) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |
| The resync walk always terminates for any finite input (no infinite loop) | proptest P1 — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — the resync anchor is the mechanism that recovers ISO-on-TCP frame synchronization after any malformed or adversarial input, directly named in the F2 authoring scope's error-taxonomy requirement |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20/SS-21 boundary; frame-walk loop resync sub-routine (planned, `S7commAnalyzer::on_data`) |
| ADR | ADR-014 Decision 8 ("Resync anchor" subsection); precedent: ADR-013 Decision 3 step 3 (IEC-104 `0x68` resync, F-P2-L2 1-byte-advance correction) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none directly — resync itself is not an anomaly; the triggering carry-overflow event's T0814 is BC-2.20.014's concern) |

## Related BCs

- BC-2.20.002 — depends on (the bad-version-byte reject condition that triggers ordinary resync)
- BC-2.20.013 — composes with (resync is invoked from within the frame-walk loop)
- BC-2.20.014 — composes with (resync is the recovery step after a carry-overflow clear)

## Architecture Anchors

- `S7commAnalyzer::on_data` (planned, SS-21) — resync sub-routine: `while parse_tpkt_header(&working[cursor..]).is_none() && working.len() - cursor >= 1 { cursor += 1 }`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 8` — "Resync anchor: the TPKT `version` byte ... advance 1 byte at a time on invalid version bytes, never 2, to avoid skipping a real `0x03` at the next offset"
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` — precedent 1-byte resync correction for IEC-104's `0x68` anchor

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst per ADR-014 Decision 9,
anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | operates on the local `working` buffer and cursor within a single `on_data` invocation; no persistent state beyond the ordinary carry-buffer stash at the end |
| **Deterministic** | yes |
| **Thread safety** | scoped to a single flow's processing |
| **Overall classification** | stateful orchestration sub-routine built on the pure `parse_tpkt_header` primitive |
