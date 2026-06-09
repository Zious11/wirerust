---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.015: Write FC to Coil Output Only ({0x05, 0x0F}) Emits T0835 Manipulate I/O Image Finding

## Description

Write operations that alter the Modbus coil output image — FC 0x05 (Write Single Coil) and
FC 0x0F (Write Multiple Coils) — emit a T0835 ("Manipulate I/O Image") finding. The I/O
image is the device's in-memory representation of its physical process state. Writing to it
can mask a real process condition (e.g., forcing a coil to "on" when the physical actuator is
off) or trigger actuators without authorization. This BC covers only the coil-output write
subset {0x05, 0x0F}; holding-register writes (0x06, 0x10, 0x16) are handled exclusively by
T0836 (BC-2.14.014), which takes priority, and T0835 is suppressed for those FCs. FC 0x17
and FC 0x16 are also excluded per the v1 priority rule (see Invariant 1).

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `function_code` is one of: `0x05`, `0x0F`. (Coil-only writes — the FCs for which T0836
   does NOT apply.) FC 0x06, 0x10, and 0x16 are in the T0836 subset; T0836 takes priority
   for those FCs and T0835 is suppressed. FC 0x17 was previously in this set but is excluded
   in v1 per the priority rule (0x17 does not trigger T0836 but T0835 is also suppressed for
   it due to the v1 policy simplification; see Invariant 1 note below).
4. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Malicious`
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus I/O image write: FC 0x{fc:02X} to unit {unit_id}"`
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_technique: Some("T0835".to_string())`
   - `source_ip: Some(flow_key.client_ip())`
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. A T0855 Finding is ALSO emitted for the same PDU (BC-2.14.013).
3. A T0836 Finding is NOT emitted alongside T0835 — by precondition 3, this BC fires only for
   FC 0x05 and FC 0x0F, which are coil-only writes outside the T0836 subset {0x06, 0x10, 0x16}.
   FC 0x06, 0x10, and 0x16 are handled exclusively by T0836 (BC-2.14.014); T0835 is suppressed
   for those FCs per the priority rule. FC 0x05 and 0x0F emit T0835 but NOT T0836.
4. `flow.write_count` and `self.total_write_count` incremented once per PDU.
5. `self.fn_code_counts.entry(function_code)` incremented by 1.

## Invariants

1. **T0835 FC subset (v1 — coil-only writes)**: {0x05, 0x0F}.
   Per the co-emission priority rule (Decision 7, architecture-delta.md §2.6):
   - T0836 takes priority for FCs {0x06, 0x10, 0x16} (holding-register writes). T0835 is
     suppressed for those FCs.
   - T0835 fires ONLY for FC 0x05 (Write Single Coil) and FC 0x0F (Write Multiple Coils) —
     the coil-only write FCs where T0836 does not apply.
   - FC 0x17 (Read/Write Multiple Registers): in v1, emits T0855 only. Neither T0836 (0x17
     is not in the holding-register subset) nor T0835 (suppressed under v1 simplification)
     fires for 0x17. If this is a concern for specific deployments, a future BC revision can
     re-enable T0835 for 0x17.
   - FC 0x05 Write Single Coil: directly flips one output bit in the coil I/O image.
   - FC 0x0F Write Multiple Coils: bulk flip of output coil image.
2. **FCs excluded from T0835** (and rationale):
   - 0x06, 0x10, 0x16: T0836 takes priority (holding-register parameter writes).
   - 0x15 Write File Record: targets file records, not the real-time I/O image.
   - 0x17 Read/Write Multiple Registers: T0855 only in v1 (see above).
   - 0x08 Diagnostics: state management, not I/O image (T0814, see BC-2.14.018).
3. The `mitre_technique` field MUST carry `"T0835"` (ICS namespace).
4. **T0835 does NOT co-fire with T0836 for the same PDU.** The priority rule ensures only one
   of T0836/T0835 fires per PDU. T0855 always fires alongside whichever of T0836/T0835 fires.
   Emission order for a coil-write PDU: T0855 → T0835 (T0836 is absent for coil-only writes).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC 0x05 writing coil 0x0000 (coil 0, the first output) with value 0xFF00 (= ON) | T0855 + T0835 emitted (coil I/O image flip); T0836 NOT emitted. |
| EC-002 | FC 0x05 writing coil 0x0000 with value 0x0000 (= OFF) | T0855 + T0835 emitted (turning off an actuator is equally significant forensically). |
| EC-003 | FC 0x0F bulk-writing 2000 coils (max spec-compliant bulk coil write) | Single T0855 + T0835 pair; the quantity field is captured in evidence bytes. |
| EC-004 | FC 0x17 (Read/Write Multiple Registers): write qty = 1, read qty = 1 | T0855 only emitted (v1 simplification: T0836 does not apply, T0835 suppressed for 0x17). |
| EC-005 | FC 0x16 in request direction | T0855 + T0836 emitted; T0835 NOT emitted (T0836 priority takes over for 0x16). |
| EC-006 | FC 0x15 (Write File Record) | T0855 only; T0835 NOT emitted; T0836 NOT emitted. |
| EC-007 | `all_findings` at MAX_FINDINGS - 1 when FC 0x05 (coil write) arrives | T0855 fills the last slot. T0835 is skipped by the cap guard (`dropped_findings=1`). T0836 is not involved (coil write is not in T0836 subset). Counters incremented. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ADU: `00 01 00 00 00 06 01 05 00 00 FF 00` (FC=0x05 Write Single Coil, coil=0, value=ON, UnitID=1) — ClientToServer | T0855 + T0835 emitted; T0836 NOT in findings; `write_count=1`, `fn_code_counts[0x05]=1` | happy-path (coil) |
| ADU: `00 02 00 00 00 08 01 0F 00 00 00 04 01 0F` (FC=0x0F Write Multiple Coils, UnitID=1, start=0, qty=4, byte=0x0F) — ClientToServer | T0855 + T0835 emitted; `write_count=1`, `fn_code_counts[0x0F]=1` | happy-path (bulk coil) |
| ADU: `00 03 00 00 00 09 01 17 00 00 00 01 00 00 00 01 02 00 42` (FC=0x17 RW-Multiple, UnitID=1) — ClientToServer | T0855 only emitted (v1: neither T0836 nor T0835 for 0x17 per v1 simplification); `fn_code_counts[0x17]=1` | happy-path (RW-multiple, T0855 only) |
| ADU: `00 04 00 00 00 06 01 06 00 10 01 F4` (FC=0x06 Write Single Register) — ClientToServer | T0855 + T0836 emitted; T0835 NOT emitted (T0836 priority suppresses T0835) | negative (T0836 priority — T0835 does NOT fire for 0x06) |
| ADU: `00 05 00 00 00 08 01 16 00 10 FF F0 00 0F` (FC=0x16 Mask Write) — ClientToServer | T0855 + T0836 emitted; T0835 NOT emitted | negative (mask-write: T0836 priority, T0835 excluded) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc Write-class completeness over all 256 FC values | Kani (sub-property B) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC covers I/O image manipulation detection, a core ICS forensic signal where a write command falsifies the process state visible to operators |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0835 — Manipulate I/O Image (ATT&CK for ICS; IcsImpairProcessControl tactic) |

## Related BCs

- BC-2.14.013 — composes with (T0855 co-emitted for same PDU)
- BC-2.14.014 — composes with (T0836 takes priority for register FCs 0x06/0x10/0x16; this BC fires only for coil-only FCs 0x05/0x0F where T0836 does not apply)
- BC-2.14.016 — composes with (T0831 may co-emit if coordinated write sequence detected)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — I/O image write detection branch in `on_data`
- `src/mitre.rs` — `technique_info("T0835")` arm (new per ADR-005 §4.2)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Write-class sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.6 (T0835 detection trigger: coil-only I/O image writes {0x05, 0x0F}); modbus-tcp-research.md §5 (T0835 — "Writes to coil output image"); ADR-005 §Decision |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell |
