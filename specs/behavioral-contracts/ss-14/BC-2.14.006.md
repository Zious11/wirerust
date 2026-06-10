---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified:
  - version: "1.1"
    date: 2026-06-10
    change: "v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Postconditions, Edge Cases, Capability Anchor Justification, L2 Domain Invariants, and Architecture Anchors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md."
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

# BC-2.14.006: Exception Response Detection — FC High Bit Set Identifies Exception and Recovers Original FC

## Description

A Modbus exception response is identified by the high bit of the Function Code byte being set
(`fc >= 0x80`). This is an unambiguous, single-byte signal defined by the Modbus spec: servers
signal errors by returning `request_FC | 0x80` followed by a 1-byte exception code. The
`classify_fc` function (BC-2.14.005) classifies these FCs as `FunctionCodeClass::Exception`.
The original request FC is recovered by masking: `original_fc = fc & 0x7F`. The exception code
byte that follows the FC byte (at PDU offset 1, i.e. `data[8]` in the ADU) is in the range
`0x01..=0x08, 0x0A, 0x0B` — note there is no code `0x09`. Exception FCs appearing in the
server-to-client direction are used by BC-2.14.011 for transaction attribution.

## Preconditions

1. `fc` is the `function_code` byte extracted from a validated Modbus ADU (BC-2.14.001 through
   BC-2.14.004 passed).
2. `fc >= 0x80` — the high bit (bit 7) is set.
3. The ADU is in the server-to-client direction (response direction per BC-2.14.009 direction
   semantics) OR the direction is being corroborated via the exception flag per the research §4
   point 2: "Response FC >= 0x80 is an exception response, unambiguously a response regardless
   of direction inference."
4. The ADU has at least 1 byte of PDU data beyond the FC byte (`h.length >= 2` ensures the
   exception code byte is present at `data[8]`).

## Postconditions

1. `classify_fc(fc)` returns `FunctionCodeClass::Exception`.
2. The original request FC is recovered as `original_fc = fc & 0x7F`. Range: `original_fc`
   is in `0x00..=0x7F`. If `original_fc` is in the Write set, exception attribution applies
   per BC-2.14.011.
3. The exception code byte is read from `data[8]` (PDU byte index 1, i.e. the byte immediately
   after the FC byte in the ADU). The exception code must be in the standard set: `0x01`, `0x02`,
   `0x03`, `0x04`, `0x05`, `0x06`, `0x07`, `0x08`, `0x0A`, `0x0B`. Code `0x09` does NOT exist
   in the Modbus standard; if `data[8] == 0x09`, it is treated as an unknown/malformed exception
   code but the ADU is not rejected.
4. Exception FCs in the Write set (e.g. `0x85` = exception for `0x05`, `0x86` = exception for
   `0x06`, `0x90` = exception for `0x10`) carry forensic significance: they are evidence of
   attempted unauthorized write commands that the server rejected. Attribution is handled in
   BC-2.14.011.
5. Exception FCs with `original_fc` NOT in the Write set (e.g. `0x83` = exception for Read
   Holding Registers) are correlated via the transaction table (BC-2.14.010 and BC-2.14.011)
   but do not trigger T1692.001 attribution on their own.

## Invariants

1. **Biconditional exception identification** (VP-022 sub-property C):
   `classify_fc(fc) == FunctionCodeClass::Exception` if and only if `fc >= 0x80`.
   This is bidirectional: (a) `fc >= 0x80` always produces `Exception`; (b) `Exception` is
   ONLY produced for `fc >= 0x80`. No FC below 0x80 is classified as Exception.
2. **FC recovery lossless**: `original_fc = fc & 0x7F` always yields the correct request FC
   because the server sets the high bit of the ORIGINAL FC byte: `exception_fc = request_fc | 0x80`.
   Masking reverses this exactly. The operation is lossless for all `fc` in [0x80, 0xFF].
3. **Exception code range**: the standard exception codes are `{0x01, 0x02, 0x03, 0x04, 0x05,
   0x06, 0x07, 0x08, 0x0A, 0x0B}` — 10 codes total. `0x09` is explicitly absent from the
   Modbus spec. An exception ADU with code `0x09` is non-standard but not rejected by the parser.
4. **Directionality corroboration**: exception FCs are always responses (server-to-client).
   If a `fc >= 0x80` ADU appears in the client-to-server direction, it is either a misrouted
   response (unusual) or a protocol anomaly. The direction inference from the port (BC-2.14.009)
   takes precedence; the exception bit provides corroboration per research §4 point 2.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC = 0x80 (minimum exception FC, original = 0x00) | `Exception`; `original_fc = 0x00` (undefined FC — flag as anomalous in attribution) |
| EC-002 | FC = 0x83 (exception for FC 0x03, Read Holding Registers) | `Exception`; `original_fc = 0x03`; exception code in data[8] per standard |
| EC-003 | FC = 0x85 (exception for FC 0x05, Write Single Coil) | `Exception`; `original_fc = 0x05` (Write class); attribution via BC-2.14.011 |
| EC-004 | FC = 0x90 (exception for FC 0x10, Write Multiple Registers) | `Exception`; `original_fc = 0x10` (Write class); attribution via BC-2.14.011 |
| EC-005 | FC = 0xFF (exception for FC 0x7F, unknown original) | `Exception`; `original_fc = 0x7F` (Unknown class); no write attribution |
| EC-006 | Exception code = 0x01 (Illegal Function) | Standard; forensic meaning: FC not supported — burst of 0x01s across many FCs = FC scanning |
| EC-007 | Exception code = 0x04 (Server Device Failure) | Standard; forensic meaning: possible device impact |
| EC-008 | Exception code = 0x09 (non-standard) | ADU accepted but exception code treated as malformed; no exception-code-based finding is suppressed |
| EC-009 | Exception code = 0x0B (Gateway Target Device Failed to Respond) | Standard; forensic meaning: device behind gateway silent — potential DoS impact |
| EC-010 | ADU with `h.length == 2` (Unit ID + FC, no exception code byte) | Exception FC detected via `classify_fc`; exception code byte at data[8] is absent if `data.len() < 9`; implementation must guard before reading data[8] |

## Canonical Test Vectors

| Input (hex, full ADU) | FC byte | `classify_fc` result | `original_fc` | Exception code | Category |
|-----------------------|---------|---------------------|---------------|----------------|----------|
| `00 05 00 00 00 03 01 83 02` | `0x83` | `Exception` | `0x03` | `0x02` (Illegal Data Address) | happy-path: Read HR exception |
| `00 06 00 00 00 03 01 85 01` | `0x85` | `Exception` | `0x05` (Write Single Coil) | `0x01` (Illegal Function) | happy-path: Write exception — attribution target |
| `00 07 00 00 00 03 01 90 04` | `0x90` | `Exception` | `0x10` (Write Multiple Registers) | `0x04` (Device Failure) | happy-path: bulk write exception |
| `00 08 00 00 00 02 01 FF` | `0xFF` | `Exception` | `0x7F` | (no code byte — h.length=2, just FC) | edge-case: minimum-length exception ADU |
| `00 09 00 00 00 03 01 03 06` | `0x03` | `Read` (NOT Exception) | n/a | n/a | regression: FC 0x03 below 0x80 must NOT be Exception |

**Annotated canonical exception vector** (0x83 with exception code 0x02):
```
Bytes:  00 05  |  00 00  |  00 03  |  01  |  83  |  02
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  |  ExCode
Value:  0x0005 |  0x0000 |    3    |   1  | 0x83 |  0x02 (Illegal Data Address)
Parse:  —      |  PASS   |  PASS   |  —   | fc>=0x80 → Exception; orig=0x03
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Sub-property C: `classify_fc(fc) == Exception` iff `fc >= 0x80` (biconditional, all 256 values) | Kani: `assert_eq!(classify_fc(fc) == Exception, fc >= 0x80)` symbolic over `fc: u8 = kani::any()` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines exception response detection, which is the mechanism for attributing failed attack attempts (T1692.001 attribution evidence) in the ICS analysis capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format — exception attribution feeds T1692.001 findings) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `classify_fc` + `on_data` exception handling); ADR-005 §2.5 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.005 — composes with (Exception is one of the five classify_fc result variants; this BC details its semantics)
- BC-2.14.010 — depends on (response matching uses exception detection to identify response direction)
- BC-2.14.011 — depends on (exception attribution uses original_fc recovery defined here)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `classify_fc`: `if fc >= 0x80 { return FunctionCodeClass::Exception; }` pre-guard
- `src/analyzer/modbus.rs` — exception code reading: `data[8]` (guarded by `data.len() >= 9`)
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.5` — "Exception detection invariant: classify_fc(fc) == Exception if and only if fc >= 0x80"
- `modbus-tcp-research.md §3` — exception code table; note absence of 0x09

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — sub-property C (Exception biconditional)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.5; modbus-tcp-research.md §3 (exception responses and codes); research §4 point 2 (directionality corroboration) |
| **Confidence** | high — exception mechanism is formally specified by Modbus.org spec V1.1b3; absence of 0x09 is spec-confirmed [SPEC] |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — VP-022 Kani target |
