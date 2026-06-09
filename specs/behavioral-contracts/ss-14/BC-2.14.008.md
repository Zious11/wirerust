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

# BC-2.14.008: Diagnostic-Class FC Classification and Sub-Function Dispatch (0x08 and 0x2B)

## Description

`classify_fc(fc: u8)` returns `FunctionCodeClass::Diagnostic` for `fc = 0x08` (Diagnostics)
and `fc = 0x2B` (Encapsulated Interface Transport / MEI). These are management and tunneling
FCs that are rare on a normal SCADA polling segment and carry elevated forensic significance.
FC `0x08` requires further sub-function parsing: the 2-byte big-endian sub-function field at
PDU bytes 1–2 (ADU bytes `data[8..10]`) determines the specific risk. Three sub-functions
are forensically critical: `0x0001` (Restart Communications — state-changing/DoS), `0x0004`
(Force Listen Only Mode — DoS that silences the slave), and `0x000A` (Clear Counters —
anti-forensic evasion). This BC defines what makes Diagnostic FCs "unusual" on a normal SCADA
segment and specifies the sub-function-level dispatch that feeds BC-2.14.018 and BC-2.14.019
(finding emission, handled in the 013+ burst).

## Preconditions

1. `fc` is `0x08` or `0x2B`.
2. For sub-function dispatch: the ADU has `h.length >= 4` (Unit ID byte + FC byte + 2-byte
   sub-function = 4 bytes minimum in the PDU portion), equivalently `data.len() >= 10`
   (7-byte MBAP header + FC byte at data[7] + sub-function at data[8..10] = 10 bytes minimum
   to safely read `data[8]` and `data[9]`).
   The correct guard is **`h.length < 4`** (not `h.length < 3`) for skipping sub-function
   parsing: a Length value of 3 means Unit ID + FC + ONE sub-function byte (only), which is
   insufficient for a 2-byte sub-function field. Length = 4 is the minimum that contains a
   complete 2-byte sub-function.
3. The ADU has passed the 3-point validity gate (BC-2.14.001 through BC-2.14.004).

## Postconditions

1. `classify_fc(0x08)` returns `FunctionCodeClass::Diagnostic`.
2. `classify_fc(0x2B)` returns `FunctionCodeClass::Diagnostic`.
3. For `fc = 0x08` with sufficient data (`data.len() >= 10`), the sub-function is decoded as:
   `sub_func = u16::from_be_bytes([data[8], data[9]])` — big-endian, immediately following the
   Unit ID and FC bytes.
4. The following sub-function values trigger specific detection behavior (handled in BC-2.14.018+):
   - `sub_func = 0x0001` (Restart Communications Option): state-changing; DoS-capable (restarts
     port, may clear comm event log). Maps to MITRE T0814 (Denial of Service).
   - `sub_func = 0x0004` (Force Listen Only Mode): DoS — slave stops responding to ALL subsequent
     commands until reset; single-packet detection signal. Maps to MITRE T0814.
   - `sub_func = 0x000A` (Clear Counters and Diagnostic Register): anti-forensic; wipes
     diagnostic counters/state. Maps to `ThreatCategory::Anomaly` (no single ATT&CK ICS ID
     per ADR-005 §2.6).
5. Sub-function `0x0000` (Return Query Data / loopback) is benign — no finding emitted.
6. Sub-functions not in `{0x0000, 0x0001, 0x0004, 0x000A}` are handled without emitting a
   finding in v1 (they are counted in `fn_code_counts[0x08]` but do not trigger detections).
7. For `fc = 0x2B` (MEI): `classify_fc` returns `Diagnostic`; sub-type parsing (0x0D CANopen,
   0x0E Read Device Identification) is deferred to a follow-on feature. In v1, `0x2B` is
   counted in `fn_code_counts[0x2B]` without sub-type-specific findings.

## Invariants

1. **0x08 and 0x2B are the only Diagnostic FCs**: the Diagnostic class contains exactly these
   two values. No other FC below 0x80 is Diagnostic (except via the Exception pre-guard which
   applies to all fc >= 0x80).
2. **Why Diagnostic, not Write**: FC 0x08 sub-functions can be state-changing or DoS-capable,
   but the detection logic is sub-function-based (T0814), not class-based Write detection
   (T0855/T0836). Separating Diagnostic from Write allows the analyzer to route 0x08 to
   sub-function parsing rather than the write-rate counter. The write-rate counter (`window_write_count`)
   is NOT incremented for `fc = 0x08` ADUs.
   **Note on deliberate write-set narrowness**: the 7-FC write class {0x05, 0x06, 0x0F, 0x10,
   0x15, 0x16, 0x17} intentionally excludes 0x08. State-changing Diagnostics sub-functions
   (0x0001 Restart, 0x0004 Force Listen Only) are detected via the Diagnostic/T0814 path
   (BC-2.14.018), not via the write-rate path. This is a deliberate v1 detection policy:
   the write set covers data-manipulation FCs; the Diagnostic class covers management FCs with
   sub-function semantics. The "WRITE set + 0x08" framing in research §2 describes the general
   risk set, not the exact implementation class boundaries.
3. **Sub-function read guard (corrected)**: `data[8]` and `data[9]` MUST only be read after
   confirming `data.len() >= 10`, equivalently `h.length >= 4`. The guard in code is:
   ```
   if h.length < 4 { /* skip sub-function parsing */ }
   ```
   - `h.length == 2`: Unit ID + FC only — no sub-function byte at all. Skip.
   - `h.length == 3`: Unit ID + FC + ONE byte — only the high byte of the sub-function.
     Insufficient for `u16::from_be_bytes([data[8], data[9]])`. Skip.
   - `h.length >= 4`: Unit ID + FC + TWO or more bytes — sub-function can be safely decoded.
   A guard of `h.length < 3` (the erroneous earlier form) would allow reading with only one
   sub-function byte available, causing a potential out-of-bounds access at `data[9]`.
   The correct guard is `h.length < 4` (i.e., skip when fewer than 4 bytes in the PDU).
   When `h.length < 4`, the 0x08 ADU is counted in `fn_code_counts[0x08]` but no sub-function-
   specific finding is emitted and `parse_errors` is NOT incremented (insufficient sub-function
   bytes is not a 3-point gate failure; it is a benign skip for underpopulated Diagnostics PDUs).
4. **"Unusual on a normal SCADA segment" definition** (per modbus-tcp-research.md §5.2):
   - A normal SCADA segment is READ-dominated (0x03, 0x01, 0x02, 0x04 poll traffic).
   - FC 0x08 Diagnostics is a management operation, rare in steady-state operation.
   - FC 0x2B MEI Read Device ID (sub-type 0x0E) is a reconnaissance/fingerprinting operation.
   - FC 0x11 (Report Server ID) is also reconnaissance but is classified as `Read` not
     `Diagnostic`; its anomalous nature is handled by the finding emission layer (not this BC).
5. **Diagnostic FCs do not increment `window_write_count`**: false-positive prevention.
   Diagnostic FCs increment `total_pdu_count` and `fn_code_counts[fc]` but NOT the write-rate
   detection counters.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC = 0x08, sub_func = 0x0001 (Restart Comms) | `Diagnostic`; sub-function parsed; T0814 candidate (handled BC-2.14.018 — both 0x0001 and 0x0004 emit T0814) |
| EC-002 | FC = 0x08, sub_func = 0x0004 (Force Listen Only) | `Diagnostic`; sub-function parsed; T0814 candidate — highest-value single-packet signal (BC-2.14.018) |
| EC-003 | FC = 0x08, sub_func = 0x000A (Clear Counters) | `Diagnostic`; sub-function parsed; anti-forensic `Anomaly` finding; no ATT&CK ID emitted |
| EC-004 | FC = 0x08, sub_func = 0x0000 (loopback) | `Diagnostic`; sub-function parsed; benign — no finding |
| EC-005 | FC = 0x08, sub_func = 0xFFFF (unknown) | `Diagnostic`; counted; no detection in v1 |
| EC-006 | FC = 0x08, `h.length = 2` (no sub-function bytes: unit+FC only) | `Diagnostic`; sub-function NOT parsed (`h.length < 4` guard fires); no sub-function finding emitted; `pdu_count++`; no `parse_errors` increment |
| EC-007 | FC = 0x08, `h.length = 3` (only ONE sub-function byte present) | `Diagnostic`; sub-function NOT parsed (`h.length < 4` guard fires); same behavior as EC-006; no out-of-bounds access |
| EC-008 | FC = 0x2B (MEI) | `Diagnostic`; sub-type depth skipped in v1; counted in fn_code_counts[0x2B] |
| EC-009 | FC = 0x88 (exception for 0x08) | `Exception` (pre-guard) — NOT Diagnostic; original_fc = 0x08 |
| EC-010 | 0x08 ADU on an otherwise Read-dominated flow | `Diagnostic`; does NOT increment `window_write_count`; detection is via sub-function finding, not write-rate alarm |

## Canonical Test Vectors

| Input (hex, full ADU) | FC | Sub-func | `classify_fc` | Finding trigger | Category |
|-----------------------|----|----------|---------------|----------------|----------|
| `00 01 00 00 00 06 01 08 00 04 00 00` | `0x08` | `0x0004` | `Diagnostic` | T0814 Force Listen Only (BC-2.14.018) | happy-path: single-packet DoS |
| `00 02 00 00 00 06 01 08 00 01 00 00` | `0x08` | `0x0001` | `Diagnostic` | T0814 Restart Comms (BC-2.14.018 — 0x0001 and 0x0004 both handled by BC-018; BC-019 handles 0x000A exception-burst) | happy-path: state-changing |
| `00 03 00 00 00 06 01 08 00 0A 00 00` | `0x08` | `0x000A` | `Diagnostic` | Anomaly: Clear Counters anti-forensic | happy-path: anti-forensic |
| `00 04 00 00 00 06 01 08 00 00 00 00` | `0x08` | `0x0000` | `Diagnostic` | None (benign loopback) | edge-case |
| `00 05 00 00 00 04 01 2B 0E 01` | `0x2B` | n/a (v1) | `Diagnostic` | None in v1 | edge-case: MEI |
| `00 06 00 00 00 02 01 08` | `0x08` | absent | `Diagnostic` | None (h.length=2; no sub-func) | edge-case: short diagnostic |

**Annotated Force Listen Only vector** (ADU bytes):
```
Bytes:  00 01  |  00 00  |  00 06  |  01  |  08  |  00 04  |  00 00
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  | SubFunc | Data
Value:  0x0001 |  0x0000 |    6    |   1  | 0x08 | 0x0004  | 0x0000
Class:  —      |         |         |      | Diag | T0814 DoS!
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Sub-property B: `classify_fc(0x08)` and `classify_fc(0x2B)` return `Diagnostic`; no panic on any `fc: u8` | Kani symbolic fc |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the Diagnostic FC class and its sub-function dispatch logic, which enables T0814 (Denial of Service) and anti-forensic detection in the ICS analysis capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format — sub-function parsing feeds T0814 findings in the ICS matrix) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `classify_fc` Diagnostic arm + `on_data` sub-function branch); ADR-005 §2.5 and §2.6 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.005 — composes with (Diagnostic is one of the five classify_fc classes)
- BC-2.14.006 — sibling (Exception pre-guard applies; 0x88 is Exception, not Diagnostic)
- BC-2.14.007 — sibling (0x08 Diagnostics is NOT Write — distinct class for sub-function detection)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `classify_fc` Diagnostic arm: `0x08 | 0x2B => FunctionCodeClass::Diagnostic`
- `src/analyzer/modbus.rs` — sub-function reading: `u16::from_be_bytes([data[8], data[9]])` guarded by `data.len() >= 10`
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.6` — "Diagnostic sub-function parsing: when fc == 0x08 and PDU has at least 2 more data bytes, read sub_func..."
- `modbus-tcp-research.md §2` — Diagnostics sub-function table: 0x0001/0x0004/0x000A forensic significance

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — sub-property B (Diagnostic classification coverage)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.5 and §2.6; modbus-tcp-research.md §2 (FC 0x08 sub-functions); research §5 (T0814 Diagnostics detection patterns) |
| **Confidence** | high — sub-function codes from Modbus.org spec V1.1b3 [SPEC]; T0814 mapping from MITRE ATT&CK for ICS [MITRE] |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (classify_fc is pure; sub-function read in on_data is effectful shell) |
| **Global state access** | none (classify_fc) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | classify_fc — pure core; sub-function dispatch in on_data — effectful shell |
