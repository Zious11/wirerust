---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-07-15T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: feature-iec104
modified:
  - "v1.1: F-P4-M2 — VP-045 harness names synced to registry: proptest_vp045_direction_isolation + proptest_vp045_independent_run_equivalence. 2026-07-14"
  - "v1.2: F-172-001 WALK-FIRST-RESIDUAL-BOUND (2026-07-15) — PRE-CHECK-DISCARD-ALL semantics replaced with WALK-FIRST-RESIDUAL-BOUND per adversarial finding F-172-001 (HIGH) and research validation report .factory/cycles/feature-iec104/research/f-172-001-carry-bound-validation.md. Rationale: (1) ADR-013 Decision 3 frame-walk loop already describes walk-first design — the old pre-check was the outlier contradicting the ADR; (2) wirerust DNP3 precedent (src/analyzer/dnp3.rs) explicitly hardened against PRE-CHECK-DISCARD-ALL on F-B-002 evasion-DoS grounds — adopting it for IEC-104 would re-open that hole and split SS-19 from SS-15; (3) old semantics created a Ptacek/Newsham-class detection-evasion channel (attacker pads burst just over 255 B to suppress parsing of a head attack frame). New semantics: frame walk extracts ALL complete frames first (no aggregate-size pre-check may discard a delivery before extraction); MAX_IEC104_CARRY_BYTES=255 applies only to the RESIDUAL partial-frame carry after frame extraction. Carry-overflow dedup flags (carry_overflow_reported_c2s / carry_overflow_reported_s2c) added to Iec104FlowState — SEPARATE from malformed_len_reported_* flags (BC-2.19.026 Inv-5) so the two anomaly classes cannot suppress each other. Canonical test vectors (old 1+255→1 and 200+100→200 discard-all vectors were the confirmed defect) removed and replaced with split-frame, multi-delivery, and defensive adversarial vectors. D-452 (pending orchestrator decision ID). Validation: .factory/cycles/feature-iec104/research/f-172-001-carry-bound-validation.md."
  - "v1.3: F-172-201 (2026-07-15) — PC-3/Inv-2 prose precision per F-172-201 (Pass 2 LOW): bound enforced at on_data entry on carry alone, one-call equivalent of post-walk residual; behavior unchanged."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "0e684c9"
---

# BC-2.19.025: Directional Carry Buffers Bounded at MAX_IEC104_CARRY_BYTES=255 (VP-045)

## Description

`Iec104FlowState` maintains two separate carry buffers: `carry_c2s` (client-to-server,
capturing bytes from the client direction that span a TCP segment boundary) and
`carry_s2c` (server-to-client). Each buffer is bounded at `MAX_IEC104_CARRY_BYTES = 255`
bytes.

**WALK-FIRST-RESIDUAL-BOUND semantics (F-172-001):** The frame-walk loop (BC-2.19.026)
concatenates the directional carry with the incoming delivery and extracts ALL complete
APCI frames first. No aggregate-size pre-check may discard a delivery before frame
extraction. `MAX_IEC104_CARRY_BYTES = 255` applies exclusively to the **residual**
partial-frame bytes remaining after the frame walk drains all complete frames. A
spec-conformant partial frame is at most 254 bytes by construction: a 255-byte prefix
constitutes a complete frame (LEN=253 → LEN+2=255) that the walk would already have
consumed. Therefore the 255-byte residual bound is a defensive fail-closed guard
(SEC-001-S168 defense-in-depth) unreachable for conformant IEC-104 traffic. Concretely,
this bound is checked at `on_data` entry on the directional carry buffer (the walk residual
stashed by the prior `on_data` call), before the current delivery is appended and walked;
it fires only on genuinely malformed or adversarial input.

When the residual bound is exceeded (non-conformant or adversarial condition only): the
offending direction's carry is cleared; the analyzer resyncs on subsequent deliveries
(fresh scan for the next 0x68 start byte — not a permanent desync latch; the flow stays
tracked and the analyzer remains active); and ONE T0814 finding is emitted per direction
with per-direction dedup (carry_overflow_reported_c2s / carry_overflow_reported_s2c).

This design follows RULING-DNP3-SIBLING-001 (directional isolation, matching wirerust's
DNP3 sibling analyzer hardened against the same PRE-CHECK-DISCARD-ALL evasion on F-B-002
grounds) and prevents cross-direction carry contamination, which is the primary target of
VP-045 proptest.

## Preconditions

1. `Iec104FlowState` for the flow exists.
2. An `on_data` call delivers bytes in a given direction (C2S or S2C); bytes may represent
   one or more complete APCI frames, a partial frame, junk bytes, or any combination.
3. [Residual-overflow path only — defensive branch, non-conformant input] At `on_data`
   entry, BEFORE the current delivery is appended and the frame-walk loop begins, the
   directional carry buffer (carry bytes stashed by the previous `on_data` call) already
   exceeds `MAX_IEC104_CARRY_BYTES = 255`. This is the one-call-shifted equivalent of a
   post-walk residual bound: the walk's stash path leaves at most 254 bytes by construction
   (an incomplete-frame stash is at most 254 bytes; a lone-start-byte stash is exactly 1
   byte; the malformed-LEN and bad-start-byte advance branches do not add extra bytes to
   the stash past the walk residual), so a carry buffer exceeding 255 at entry can only
   have arisen from adversarial or non-conformant input on a prior `on_data` call.

## Postconditions

1. **Entry-check anti-evasion ordering**: the carry overflow check runs at `on_data` entry
   on the directional carry buffer alone — before the current delivery is appended and the
   frame-walk loop begins. The current delivery is ALWAYS appended and walked regardless of
   the entry carry state (if carry was oversized it is cleared first; the delivery is then
   appended to the empty carry and walked normally). No aggregate-size pre-check on
   `carry.len() + delivery.len()` may discard a delivery before complete-frame extraction
   (anti-evasion clause; cites F-172-001 and Ptacek/Newsham 1998).
2. The directional carry buffer after each `on_data` call contains the residual partial-frame
   bytes, bounded at `MAX_IEC104_CARRY_BYTES = 255`; `carry_c2s` and `carry_s2c` remain
   directionally isolated (no cross-contamination).
3. [Residual-overflow path — defensive branch] If `carry.len() > MAX_IEC104_CARRY_BYTES`
   at `on_data` entry (before the current delivery is appended and walked): (a) the
   offending direction's carry is cleared; (b) the analyzer resyncs
   (subsequent deliveries scan for next `0x68` — NOT a permanent desync latch; flow tracking
   and analyzer remain active); (c) ONE T0814 (ThreatCategory::Anomaly / Verdict::Possible /
   Confidence::Medium) is emitted for the first overflow event in that direction; (d) the
   per-direction dedup flag (`carry_overflow_reported_c2s` for C2S,
   `carry_overflow_reported_s2c` for S2C) is set on first emission and thereafter prevents
   re-emission for that direction within the flow lifetime — subsequent residual-overflow
   events in the same direction trigger carry clear + resync only (no additional T0814).

## Invariants

1. **Directional isolation**: `carry_c2s` and `carry_s2c` are always strictly separate;
   bytes from one direction are never appended to the other's carry buffer. VP-045 proptest
   verifies this exhaustively.
2. **Entry-check anti-evasion ordering**: the carry overflow check fires at `on_data` entry
   on the directional carry buffer alone, before the current delivery is appended and the
   frame-walk loop begins. This is the one-call-shifted equivalent of bounding the previous
   walk's residual: the walk stash path leaves at most 254 bytes by construction (≤ 254
   bytes for an incomplete-frame stash; exactly 1 byte for a lone-start-byte stash;
   malformed-LEN and bad-start-byte branches never stash extra bytes past the walk residual),
   so carry exceeding 255 at entry is only achievable via adversarial input on a prior call —
   confirming the guard is defensive-only and unreachable for conformant IEC-104 traffic.
   The current delivery is always appended and walked regardless; there is no aggregate-size
   pre-check on `carry.len() + delivery.len()` that could discard a delivery before frame
   extraction (anti-evasion clause, F-172-001).
3. **255-byte residual cap (defensive)**: `MAX_IEC104_CARRY_BYTES = 255` is the constant
   residual-carry limit. A spec-conformant partial frame is ≤ 254 bytes by construction
   (a 255-byte prefix is a complete frame with LEN=253 and would have been walked off).
   This bound is a fail-closed defensive guard (SEC-001-S168 defense-in-depth); it is
   unreachable for conformant IEC-104 traffic.
4. **Separate overflow dedup flags**: `Iec104FlowState` carries
   `carry_overflow_reported_c2s: bool` and `carry_overflow_reported_s2c: bool` (both
   initialized false). At most one T0814 carry-overflow finding is emitted per direction
   per flow lifetime. These flags are **intentionally separate** from
   `malformed_len_reported_c2s` / `malformed_len_reported_s2c` (BC-2.19.026 Invariant 5)
   so that one anomaly class cannot suppress the other: a malformed-LEN event in the C2S
   direction does not prevent a carry-overflow finding in that direction from being emitted,
   and vice versa.
5. **Non-permanent resync**: clearing carry on overflow is not a permanent desync latch.
   The flow remains tracked and the analyzer remains active after an overflow event; the
   next delivery for the affected direction starts with an empty carry and proceeds
   normally.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Frame walk produces residual = 254 bytes (largest conformant partial) | Residual stashed into carry; no T0814 |
| EC-002 | Frame walk produces residual = 255 bytes (conformant traffic: unreachable — a 255-byte prefix is a complete frame that the walk would have consumed) | Guard does not fire (255 is not > 255); bytes stashed |
| EC-003 | carry_c2s at `on_data` entry = 256 bytes (state injected directly; adversarial/non-conformant — walk's ≤254-by-construction bound means 256-byte carry requires test-level state injection; first occurrence in C2S) | carry_c2s cleared; ONE T0814 (Anomaly/Possible/Medium) emitted; carry_overflow_reported_c2s set |
| EC-004 | Second carry overflow in same C2S direction (flag already set) | carry_c2s cleared + resync; NO additional T0814 |
| EC-005 | C2S carry overflow (flag set); S2C has no overflow | Flags are independent: carry_overflow_reported_c2s=true; carry_overflow_reported_s2c=false |

## Canonical Test Vectors

**Vector (i) — Split frame across carry/delivery (legit traffic):**

| Field | Value |
|-------|-------|
| Direction | C2S |
| carry_c2s before | 200 bytes: first 200 bytes of a 255-byte frame `[0x68, 0xFD, <198 bytes of payload>]` (LEN=0xFD=253; frame total = LEN+2 = 255 bytes) |
| delivery | 100 bytes: 55 bytes completing the frame (bytes 200–254) + 45 bytes of a partial second frame |
| Working buf | 200 + 100 = 300 bytes |
| Frame walk | First 255 bytes = complete frame → parsed and dispatched; cursor advances to 255 |
| Residual | 45 bytes (bytes 255–299) |
| carry_c2s after | 45 bytes |
| Finding | None |
| carry_overflow_reported_c2s | false (unchanged) |

Arithmetic: carry(200) + delivery(55+45) = 300. Frame = LEN+2 = 253+2 = 255 ≤ 300 → complete. Residual = 300−255 = 45 ≤ 255. No overflow.

**Vector (ii) — Single delivery with complete frame plus tail (no prior carry):**

| Field | Value |
|-------|-------|
| Direction | S2C |
| carry_s2c before | empty (0 bytes) |
| delivery | 300 bytes: `[0x68, 0xFD, <253 bytes of payload>]` (255-byte frame) + 45 bytes of a partial second frame |
| Working buf | 0 + 300 = 300 bytes |
| Frame walk | First 255 bytes = complete frame → parsed and dispatched; cursor advances to 255 |
| Residual | 45 bytes |
| carry_s2c after | 45 bytes |
| Finding | None |

Arithmetic: carry(0) + delivery(255+45) = 300. Frame = 255 bytes ≤ 300 → complete. Residual = 45 ≤ 255. No overflow.

**Vector (iii) — Defensive adversarial carry-overflow (non-conformant, constructed; tests the dedup guard):**

This scenario is unreachable from conformant IEC-104 traffic. Tests inject adversarial state directly.

| Step | Direction | Residual after walk | carry after | Finding | Flag state |
|------|-----------|---------------------|-------------|---------|------------|
| First overflow event | C2S | 256 bytes (injected) | cleared (0 bytes) | ONE T0814 (Anomaly/Possible/Medium) | carry_overflow_reported_c2s → true |
| Second overflow event (same direction) | C2S | 256 bytes (injected) | cleared (0 bytes) | None (flag suppresses re-emission) | carry_overflow_reported_c2s stays true |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-045 | carry_c2s and carry_s2c are never mixed; each is independently bounded at 255 bytes, checked at `on_data` entry on the directional carry before delivery is appended and walked; proptest with arbitrary (direction, data) sequences verifies isolation and ENTRY-CHECK anti-evasion ordering | proptest: `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence` |
| VP-047 | No panic on carry overflow or any byte sequence in the frame-walk + residual-bound path | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — directional carry buffer isolation with walk-first frame extraction is a core correctness and security requirement for the IEC-104 passive analyzer per RULING-DNP3-SIBLING-001 and F-172-001 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence), INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 2, Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" (on carry residual overflow — defensive branch, non-conformant input only) |

## Related BCs

- BC-2.19.026 — depends on (frame-walk loop drives carry buffer lifecycle; BC-2.19.026 PC-1 walk-first ordering is the anti-evasion guarantee; BC-2.19.026 Inv-5 defines separate malformed_len_reported_* dedup flags)
- BC-2.19.027 — depends on (on_flow_close discards carry buffers)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `const MAX_IEC104_CARRY_BYTES: usize = 255;`
- `src/analyzer/iec104.rs` — `Iec104FlowState { carry_c2s: Vec<u8>, carry_s2c: Vec<u8> }`
- `src/analyzer/iec104.rs` — `Iec104FlowState.carry_overflow_reported_c2s: bool`, `carry_overflow_reported_s2c: bool` — per-direction dedup flags for carry-residual-overflow T0814; initialized false; set on first overflow emission per direction; never reset within a flow; SEPARATE from `malformed_len_reported_c2s` / `malformed_len_reported_s2c` (BC-2.19.026 Inv-5)
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 2` — carry buffer design
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` — frame-walk loop walk-first ordering (prepend carry → walk complete frames → stash residual); WALK-FIRST-RESIDUAL-BOUND is the correct reading of this ADR decision
- `ADR-013 §RULING-DNP3-SIBLING-001`
- `.factory/cycles/feature-iec104/research/f-172-001-carry-bound-validation.md` — F-172-001 validation evidence: Ptacek/Newsham 1998 evasion taxonomy; Zeek/Suricata/Snort3 graceful-degradation precedents; DNP3 F-B-002 internal precedent; internal wirerust DNP3 analogy (`src/analyzer/dnp3.rs` on_data Step 2 "Do NOT return early … Do NOT clear+return")

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-045 — `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence` (directional carry isolation; walk-first residual bound verification)
- VP-047 — `fuzz_iec104_parser`
