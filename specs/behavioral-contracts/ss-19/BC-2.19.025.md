---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
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
input-hash: "f5a97d3"
---

# BC-2.19.025: Directional Carry Buffers Bounded at MAX_IEC104_CARRY_BYTES=255 (VP-045)

## Description

`Iec104FlowState` maintains two separate carry buffers: `carry_c2s` (client-to-server,
capturing bytes from the client direction that span a TCP segment boundary) and
`carry_s2c` (server-to-client). Each buffer is bounded at `MAX_IEC104_CARRY_BYTES = 255`
bytes. When a frame-walk loop would add bytes to a carry buffer that would cause it to
exceed 255 bytes, the new bytes are discarded and a T0814 finding is emitted. This design
follows RULING-DNP3-SIBLING-001 (directional isolation) and prevents cross-direction
carry contamination, which is the primary target of VP-045 proptest.

## Preconditions

1. `Iec104FlowState` for the flow exists.
2. A `on_data` call delivers bytes that form a partial APCI frame (less than the expected total frame size).
3. `carry_c2s.len() + new_bytes.len() > 255` (C2S buffer overflow condition) OR same for carry_s2c.

## Postconditions

1. Carry buffer is NOT extended beyond 255 bytes.
2. The excess bytes are discarded.
3. T0814 finding emitted (Anomaly/Possible) noting carry buffer overflow.
4. `carry_c2s` and `carry_s2c` remain directionally isolated — no cross-contamination.

## Invariants

1. **Directional isolation**: `carry_c2s` and `carry_s2c` are always strictly separate; bytes from one direction are never appended to the other's carry buffer. VP-045 proptest verifies this exhaustively.
2. **255-byte cap**: `MAX_IEC104_CARRY_BYTES = 255` is the constant carry buffer limit, matching the maximum APCI frame size (LEN + 2 = 255).
3. **Cap rationale**: since the maximum APCI frame is 255 bytes, a complete frame can always be delivered in ≤ 255 carry bytes. Carry exceeding 255 indicates malformed input or an attack.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Carry + new bytes = 255 | Carry extended to exactly 255; no T0814 |
| EC-002 | Carry + new bytes = 256 | T0814 emitted; bytes discarded |
| EC-003 | C2S carry at 200, S2C carry at 0 | C2S and S2C remain independent |
| EC-004 | Carry reset after successful frame parse | Carry drained to 0; new data accumulates fresh |

## Canonical Test Vectors

| Direction | Carry before | New bytes | Expected carry after | Finding |
|-----------|-------------|-----------|----------------------|---------|
| C2S | 0 | 254 | 254 | none |
| C2S | 1 | 254 | 255 | none (boundary) |
| C2S | 1 | 255 | 1 (discarded) | T0814 |
| S2C | 200 | 100 | 200 (discarded) | T0814 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-045 | carry_c2s and carry_s2c are never mixed; each is independently bounded at 255 bytes; proptest with arbitrary (direction, data) sequences verifies isolation | proptest: `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence` |
| VP-047 | No panic on carry overflow condition | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — directional carry buffer isolation is a core correctness and security requirement for the IEC-104 passive analyzer per RULING-DNP3-SIBLING-001 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence), INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 2 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" (on carry overflow) |

## Related BCs

- BC-2.19.026 — depends on (frame-walk loop drives carry buffer lifecycle)
- BC-2.19.027 — depends on (on_flow_close discards carry buffers)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `const MAX_IEC104_CARRY_BYTES: usize = 255;`
- `src/analyzer/iec104.rs` — `Iec104FlowState { carry_c2s: Vec<u8>, carry_s2c: Vec<u8> }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 2`
- ADR-013 §RULING-DNP3-SIBLING-001

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-045 — `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence` (directional isolation)
- VP-047 — `fuzz_iec104_parser`
