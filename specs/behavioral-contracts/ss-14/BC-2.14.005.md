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

# BC-2.14.005: classify_fc Is Total Over All 256 FC Values — Complete Classification Enum

## Description

`classify_fc(fc: u8) -> FunctionCodeClass` is a pure total function over all 256 possible
`u8` function-code values. Every input maps to exactly one of five variants:
`{Read, Write, Diagnostic, Exception, Unknown}`. The function is exhaustive by construction
(via a `_ => Unknown` wildcard arm) and never panics. This BC defines the complete, authoritative
mapping from FC byte to class; BCs 2.14.006, 2.14.007, and 2.14.008 specify individual class
behaviors. This function is a VP-022 sub-property B Kani target.

## Preconditions

1. `fc` is any `u8` value (0x00..=0xFF) — the function accepts all 256 values.
2. The function is called after `is_valid_modbus_adu` has passed — i.e., only on the Function
   Code byte of a validated ADU. (The function itself imposes no constraint on when it's called;
   it is safe to call on any u8 at any time.)

## Postconditions

1. `classify_fc(fc)` returns exactly one `FunctionCodeClass` variant.
2. The following 10 FC values map to `FunctionCodeClass::Read`:
   `0x01` (Read Coils), `0x02` (Read Discrete Inputs), `0x03` (Read Holding Registers),
   `0x04` (Read Input Registers), `0x07` (Read Exception Status), `0x0B` (Get Comm Event
   Counter), `0x0C` (Get Comm Event Log), `0x11` (Report Server/Slave ID), `0x14` (Read
   File Record), `0x18` (Read FIFO Queue).
3. The following 7 FC values map to `FunctionCodeClass::Write`:
   `0x05` (Write Single Coil), `0x06` (Write Single Register), `0x0F` (Write Multiple Coils),
   `0x10` (Write Multiple Registers), `0x15` (Write File Record), `0x16` (Mask Write
   Register), `0x17` (Read/Write Multiple Registers — write half is state-changing).
4. The following 2 FC values map to `FunctionCodeClass::Diagnostic`:
   `0x08` (Diagnostics — management/DoS-capable sub-functions), `0x2B` (Encapsulated
   Interface Transport / MEI — recon-capable).
5. All FC values with the high bit set (`fc >= 0x80`, i.e. `fc & 0x80 != 0`) map to
   `FunctionCodeClass::Exception`. This is the FIRST check applied (see BC-2.14.006 for
   exception semantics).
6. All remaining FC values (not in sets 2–5) map to `FunctionCodeClass::Unknown`. This
   includes FC values `0x00`, `0x09`, `0x0A`, `0x0D`, `0x0E`, `0x12`, `0x13`, `0x19`–`0x27`,
   `0x29`–`0x7F` (excluding 0x2B).
7. The function never panics for any `u8` input.
8. The function is deterministic: the same `fc` always returns the same variant.

## Invariants

1. **Totality**: the `_ => FunctionCodeClass::Unknown` wildcard arm guarantees every `u8`
   input is handled — no gaps in the pattern match.
2. **Exception-first check**: the `if fc >= 0x80 { return Exception; }` guard PRECEDES the
   `match fc` arms, ensuring exception responses are classified correctly even if their
   masked FC (fc & 0x7F) would match a Read or Write arm.
3. **Write set is the state-change risk set** (per modbus-tcp-research.md §2): FCs in the
   Write class are state-changing operations that alter the physical process (coils, registers,
   files). Their detection is the primary forensic risk signal.
4. **0x17 (Read/Write Multiple Registers)**: classified as Write because the write half is
   state-changing. The read result returned in the response is secondary; the write side
   determines risk classification.
5. **0x2B (MEI) classified as Diagnostic**, not Read: while 0x2B sub-type 0x0E reads device
   identification (recon), 0x2B is a management/tunneling FC, not a pure data read. Diagnostic
   classification ensures it receives elevated attention in detection logic.
6. **No FC is assigned to two classes**: the match is non-overlapping. Given the Exception
   pre-guard, no fc in [0x80, 0xFF] ever reaches the match arms.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC = 0x00 (undefined) | `Unknown` — not in any standard set |
| EC-002 | FC = 0x03 (Read Holding Registers — most common SCADA poll) | `Read` |
| EC-003 | FC = 0x10 (Write Multiple Registers — most common bulk write) | `Write` |
| EC-004 | FC = 0x08 (Diagnostics — DoS-capable) | `Diagnostic` |
| EC-005 | FC = 0x80 (lowest exception FC) | `Exception` — high bit set; pre-guard fires |
| EC-006 | FC = 0x83 (exception for FC 0x03) | `Exception` — pre-guard fires; original FC = 0x83 & 0x7F = 0x03 |
| EC-007 | FC = 0xFF (highest byte value) | `Exception` — fc >= 0x80 |
| EC-008 | FC = 0x17 (Read/Write Multiple Registers) | `Write` — state-changing write half governs |
| EC-009 | FC = 0x2B (MEI) | `Diagnostic` — tunneling/management, not pure Read |
| EC-010 | FC = 0x7F (highest non-exception byte) | `Unknown` — not in standard set |

## Canonical Test Vectors

| FC (hex) | FC (decimal) | Expected Class | Standard Name |
|----------|-------------|----------------|---------------|
| `0x01` | 1 | `Read` | Read Coils |
| `0x02` | 2 | `Read` | Read Discrete Inputs |
| `0x03` | 3 | `Read` | Read Holding Registers |
| `0x04` | 4 | `Read` | Read Input Registers |
| `0x05` | 5 | `Write` | Write Single Coil |
| `0x06` | 6 | `Write` | Write Single Register |
| `0x07` | 7 | `Read` | Read Exception Status |
| `0x08` | 8 | `Diagnostic` | Diagnostics |
| `0x0B` | 11 | `Read` | Get Comm Event Counter |
| `0x0C` | 12 | `Read` | Get Comm Event Log |
| `0x0F` | 15 | `Write` | Write Multiple Coils |
| `0x10` | 16 | `Write` | Write Multiple Registers |
| `0x11` | 17 | `Read` | Report Server ID |
| `0x14` | 20 | `Read` | Read File Record |
| `0x15` | 21 | `Write` | Write File Record |
| `0x16` | 22 | `Write` | Mask Write Register |
| `0x17` | 23 | `Write` | Read/Write Multiple Registers |
| `0x18` | 24 | `Read` | Read FIFO Queue |
| `0x2B` | 43 | `Diagnostic` | MEI / EIT |
| `0x00` | 0 | `Unknown` | (undefined) |
| `0x7F` | 127 | `Unknown` | (reserved) |
| `0x80` | 128 | `Exception` | (exception floor) |
| `0xFF` | 255 | `Exception` | (all high-byte FCs) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Sub-property B: `classify_fc` is total over all 256 FC values; never panics | Kani: `fc: u8 = kani::any(); let _ = classify_fc(fc);` — no assertion needed beyond "does not panic" |
| VP-022 | Sub-property C: `classify_fc(fc) == Exception` iff `fc >= 0x80` | Kani: `assert_eq!(classify_fc(fc) == Exception, fc >= 0x80)` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the complete function-code classification taxonomy that underlies all Modbus threat detection in the ICS analysis capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format — FCs classified as Write drive MITRE-tagged findings in the ICS matrix) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `classify_fc`); ADR-005 §2.5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.006 — composes with (Exception class behavior — detailed semantics of `fc >= 0x80`)
- BC-2.14.007 — composes with (Write class behavior — state-change risk and MITRE mapping)
- BC-2.14.008 — composes with (Diagnostic class behavior — sub-function handling)
- BC-2.14.001 — depends on (function_code field from MBAP parse is input to this function)
- BC-2.14.009 through BC-2.14.012 — depends on (classify_fc result drives pending-table insert logic for Write-class FCs)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `classify_fc(fc: u8) -> FunctionCodeClass`
- `src/analyzer/modbus.rs` — `FunctionCodeClass` enum: `{ Read, Write, Diagnostic, Exception, Unknown }`
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.5` — complete function-code classification table and enum definition

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — sub-properties B and C (classify_fc totality and Exception biconditional)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.5; modbus-tcp-research.md §2 (full FC table with READ/WRITE classification) |
| **Confidence** | high — FC values and names are from Modbus.org spec V1.1b3 §6 |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same fc always returns same class |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — VP-022 Kani target sub-properties B and C |
