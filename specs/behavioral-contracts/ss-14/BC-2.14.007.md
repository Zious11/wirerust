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

# BC-2.14.007: Write-Class FC Classification — State-Changing Function Codes Identified as Elevated-Risk

## Description

`classify_fc(fc: u8)` returns `FunctionCodeClass::Write` for the 7 Modbus function codes that
perform state-changing writes to the physical process: `0x05` (Write Single Coil), `0x06`
(Write Single Register), `0x0F` (Write Multiple Coils), `0x10` (Write Multiple Registers),
`0x15` (Write File Record), `0x16` (Mask Write Register), and `0x17` (Read/Write Multiple
Registers). Write-class classification is the primary forensic risk signal for ICS/OT analysis:
these FCs directly actuate physical process outputs (coils), alter setpoints (holding registers),
or modify persistent storage (files). They are the target FCs for T0855, T0836, T0835, T0806,
and T0831 detection (handled in BC-2.14.013+).

## Preconditions

1. `fc` is one of `{0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17}`.
2. The ADU is in the client-to-server direction (request direction per BC-2.14.009) — Write
   FCs are expected from the requesting client. Exception versions of Write FCs (e.g. `0x85`,
   `0x90`) are classified as `Exception` by BC-2.14.006 (the pre-guard fires first), NOT as
   `Write`.

## Postconditions

1. `classify_fc(fc)` returns `FunctionCodeClass::Write`.
2. The following per-FC risk characteristics apply (used by detection logic in BC-2.14.013+):
   - `0x05` (Write Single Coil): single output bit toggled; T0835 candidate (I/O image manipulation).
   - `0x06` (Write Single Register): single 16-bit holding register written; setpoint/parameter
     change; T0836 candidate (Modify Parameter).
   - `0x0F` (Write Multiple Coils): bulk output bit write; T0835 candidate.
   - `0x10` (Write Multiple Registers): bulk holding register write; T0836 and T0831 candidate
     (Modify Parameter / Manipulation of Control).
   - `0x15` (Write File Record): persistent file-record write; elevated risk (unusual in steady
     state; possible firmware/config tampering indicator).
   - `0x16` (Mask Write Register): AND/OR-mask modification of a holding register; subtle
     parameter mutation; T0836 candidate.
   - `0x17` (Read/Write Multiple Registers): atomic read+write; write half is state-changing;
     T0836 candidate.
3. Any validated ADU with a Write-class FC increments `ModbusFlowState.window_write_count`
   and `ModbusFlowState.write_count` (rate detection feeds BC-2.14.016+).
4. Write-class FCs in the request direction are inserted into the pending transaction table
   (BC-2.14.009) so that exception responses can be attributed (BC-2.14.011).

## Invariants

1. **Exactly 7 Write FCs**: the Write set is `{0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17}` —
   7 members. No other FC below 0x80 is Write. This is the definitive, closed set per
   Modbus.org spec V1.1b3 §6.
2. **Exception pre-guard**: `classify_fc` checks `fc >= 0x80` BEFORE the Write match arm.
   Write exception responses (`0x85`, `0x86`, etc.) are classified as `Exception`, not `Write`.
3. **0x17 is Write**: even though FC 0x17 (Read/Write Multiple Registers) performs a read AND
   a write atomically, it is classified as Write because the write side is state-changing. The
   read result in the response does not reduce the risk classification of the request.
4. **0x08 (Diagnostics) is NOT Write**: Diagnostics FCs are classified as `Diagnostic`
   (BC-2.14.008) even though some sub-functions are state-changing or DoS-capable. This is a
   deliberate separation — the detection logic for Diagnostics is sub-function-based (T0814),
   not class-based.
5. **Write class drives the rate counter**: every Write-class FC increments the flow's write
   window counter. Read-class FCs at any rate must NOT increment the write counter (a normal
   segment is dominated by Read polling at high frequency; conflating Reads with Writes would
   cause constant false-positive T0806 alerts).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC = 0x05 (Write Single Coil) | `Write` — single coil output; T0835 candidate |
| EC-002 | FC = 0x06 (Write Single Register) | `Write` — setpoint; T0836 candidate |
| EC-003 | FC = 0x16 (Mask Write Register) | `Write` — subtle AND/OR mask mutation; easy to miss in manual review |
| EC-004 | FC = 0x17 (Read/Write Multiple Registers) | `Write` — write half governs; read half does not downgrade risk |
| EC-005 | FC = 0x15 (Write File Record) | `Write` — unusual in steady state; elevated suspicion |
| EC-006 | FC = 0x85 (exception for Write Single Coil) | `Exception` (pre-guard) — NOT `Write`; attribution via BC-2.14.011 |
| EC-007 | FC = 0x03 (Read Holding Registers) | `Read` — must NOT be Write; regression check that high-frequency polls do not increment write counter |
| EC-008 | FC = 0x08 (Diagnostics) | `Diagnostic` — must NOT be Write even though some sub-functions are state-changing |
| EC-009 | Write-class FC in response direction | Unusual protocol condition; classified as Write by `classify_fc` regardless; direction-based logic in `on_data` may flag this as an anomaly (request direction expected for Writes) |

## Canonical Test Vectors

| FC (hex) | Expected Class | Risk Rationale |
|----------|----------------|----------------|
| `0x05` | `Write` | Single coil — T0835 I/O image |
| `0x06` | `Write` | Single register — T0836 parameter |
| `0x0F` | `Write` | Bulk coil write — T0835 |
| `0x10` | `Write` | Bulk register write — T0836 / T0831 |
| `0x15` | `Write` | File record — unusual; config/firmware risk |
| `0x16` | `Write` | Mask register — subtle param mutation |
| `0x17` | `Write` | Atomic R+W — write half state-changing |
| `0x03` | `Read` (NOT Write) | Regression: dominant polling FC must not be Write |
| `0x86` | `Exception` (NOT Write) | Regression: Write exception must be Exception class, not Write |

**High-risk write vector** (Write Multiple Registers — T0836 / T0831 candidate):
```
Bytes:  00 0A  |  00 00  |  00 0B  |  01  |  10  | 00 01  00 02  04  00 64  00 C8
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  | addr   qty     bct  reg1    reg2
Value:  0x000A |  0x0000 |   11    |   1  | 0x10 | addr=1 qty=2   4b   100     200
Class:  —      |         |         |      | WRITE|
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Sub-property B (totality) covers Write class; no FC in [0x80,0xFF] returns Write due to pre-guard | Kani: symbolic `fc: u8 = kani::any()` — `assert!(classify_fc(fc) != Write || fc < 0x80)` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC enumerates the exact Write-class FC set and their risk characteristics, which is the foundational classification that enables T0855, T0836, T0835, T0806, and T0831 detection in the ICS analysis capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format — Write-class FCs drive ICS-matrix technique findings) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `classify_fc` Write arm; `on_data` write counter logic); ADR-005 §2.5 and §2.6 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.005 — composes with (Write is one of the five classify_fc classes; this BC details Write class)
- BC-2.14.006 — sibling (Exception pre-guard ensures exception FCs for Write requests are classified as Exception, not Write)
- BC-2.14.009 — depends on (Write-class FCs in request direction are inserted into pending table)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `classify_fc` Write arm: `0x05 | 0x06 | 0x0F | 0x10 | 0x15 | 0x16 | 0x17 => FunctionCodeClass::Write`
- `src/analyzer/modbus.rs` — `ModbusFlowState.window_write_count` and `write_count` incremented on `Write` class
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.5` — Write set definition; §2.6 MITRE mapping table
- `modbus-tcp-research.md §2` — WRITE set confirmed: "0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17" per spec V1.1b3

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — sub-property B (totality; no Write result for fc >= 0x80)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.5; modbus-tcp-research.md §2 ("WRITE set (treat as state-changing / elevated risk)") |
| **Confidence** | high — FC values from Modbus.org spec V1.1b3 §6 [SPEC] |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
