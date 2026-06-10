---
document_type: behavioral-contract
level: L3
version: "1.2"
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
    date: 2026-06-09
    change: "ADR-006 migration: mitre_technique: Some(\"T0814\") → mitre_techniques: vec![\"T0814\"] in postconditions, invariants, and canonical vectors. No behavioral change."
  - version: "1.2"
    date: 2026-06-10
    change: "v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. T0855 reference in Invariant 3 updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md."
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

# BC-2.14.018: Diagnostics FC 0x08 Sub-Function 0x0004 or 0x0001 Emits T0814 Denial of Service Finding

## Description

Modbus Diagnostics function code 0x08, when used with sub-function 0x0004 (Force Listen Only
Mode) or sub-function 0x0001 (Restart Communications Option), constitutes a Denial of Service
against the target device. Force Listen Only Mode (sub-func 0x0004) causes the slave to stop
responding to all subsequent commands — the device "goes silent" and is effectively taken
offline from the operator's perspective. Restart Communications (sub-func 0x0001) forces a
port restart, potentially clearing the comm event log (anti-forensic side-effect). Both are
single-packet, near-zero-false-positive signals that warrant immediate T0814 ("Denial of
Service") findings. This BC covers both sub-functions in a single contract because they share
the same Diagnostics FC, the same detection mechanism, and the same MITRE technique. The
sub-function value is extracted from the 2 bytes immediately following the FC byte in the PDU.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `function_code == 0x08` (Diagnostics FC, classified as `FunctionCodeClass::Diagnostic`).
4. The ADU has at least 10 bytes total (MBAP 7 bytes + FC 1 byte + sub-function 2 bytes),
   i.e., `data[offset..end].len() >= 10`.
5. `sub_func = u16::from_be_bytes([data[offset + 8], data[offset + 9]])`.
6. `sub_func` is one of: `0x0004` (Force Listen Only Mode) or `0x0001` (Restart Communications).
7. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Anomaly`
     (Per architecture-delta.md §2.6: T0814 maps to `ThreatCategory::Anomaly`, consistent
     with the DoS/Inhibit-Response semantic — the action prevents normal device function
     rather than executing an unauthorized command.)
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
     (Force Listen Only and Restart Comms are near-zero-false-positive: these sub-functions
     have no legitimate steady-state polling use. High confidence justified by
     research §5 "cheap, high-value single-packet detectors".)
   - `summary` (sub-func 0x0004): `"Modbus DoS: Force Listen Only Mode sent to unit {unit_id} — device will stop responding"`
   - `summary` (sub-func 0x0001): `"Modbus DoS: Restart Communications sent to unit {unit_id}"`
   - `evidence`: one entry — `"FC=0x08 SubFunc=0x{sub_func:04X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_techniques: vec!["T0814".to_string()]`
   - `source_ip: Some(flow_key.client_ip())`
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. `self.fn_code_counts.entry(0x08)` incremented by 1.
3. `self.total_pdu_count` incremented by 1 (via general PDU processing — done for all valid PDUs).

## Invariants

1. **Sub-function byte parsing** (pure, Kani-verifiable): the sub-function is a 2-byte
   big-endian value at bytes 8–9 of the ADU (offset 0 is ADU start):
   ```
   sub_func = u16::from_be_bytes([adu[8], adu[9]])
   ```
   This extraction is only safe when `adu.len() >= 10`, equivalently `h.length >= 4`
   (Unit ID + FC + 2 sub-function bytes). The correct guard in code is `h.length < 4`
   (consistent with BC-2.14.008 Invariant 3 — do NOT use `h.length < 3` which would
   allow reading with only one sub-function byte, causing a potential out-of-bounds access).
   If `h.length < 4` (ADU too short for a 2-byte sub-function), skip sub-function dispatch:
   do NOT emit any finding, do NOT increment `parse_errors` (insufficient bytes for
   sub-function is a benign PDU; it is not a 3-point gate failure). `fn_code_counts[0x08]`
   is still incremented.
2. **Only two sub-functions trigger T0814:** `0x0001` and `0x0004`. Other sub-functions
   of 0x08 do not emit T0814:
   - `0x0000` (Return Query Data / loopback): benign; no finding.
   - `0x000A` (Clear Counters): emits Anomaly finding per BC-2.14.019 (anti-forensic).
   - All other sub-functions: no finding in v1.
3. **No T1692.001 co-emission for this BC.** FC 0x08 is classified as `FunctionCodeClass::Diagnostic`,
   not `FunctionCodeClass::Write`. BC-2.14.013 (T1692.001) fires only for Write-class FCs. Diagnostic
   FCs do NOT trigger T1692.001.
4. The `mitre_techniques` vec MUST carry `vec!["T0814"]` — exactly one element, the ICS
   namespace technique ID `"T0814"` (per ADR-006 Vec<String> migration).
5. The `ThreatCategory::Anomaly` assignment for T0814 is consistent with the architecture
   delta §2.6 table: "T0814 (DoS) → `ThreatCategory::Anomaly`". This distinguishes DoS-class
   findings from execution/write findings (`ThreatCategory::Execution`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC=0x08, sub-func=0x0004 (Force Listen Only) in request direction | T0814 emitted with `summary` naming Force Listen Only; `confidence=High`. |
| EC-002 | FC=0x08, sub-func=0x0001 (Restart Communications) in request direction | T0814 emitted with `summary` naming Restart Communications. |
| EC-003 | FC=0x08, sub-func=0x0000 (Return Query Data / loopback) | No T0814 finding; benign sub-function. `fn_code_counts[0x08]` incremented. |
| EC-004 | FC=0x08, sub-func=0x000A (Clear Counters) | No T0814; Clear Counters is handled by BC-2.14.019 as Anomaly (anti-forensic). |
| EC-005 | FC=0x08 with h.length=3 (only ONE sub-function byte present; equiv. 9 total ADU bytes) | Sub-function dispatch skipped (`h.length < 4` guard fires). No finding. `fn_code_counts[0x08]` incremented. `parse_errors` NOT incremented (benign short PDU, not a gate failure). |
| EC-006 | FC=0x08, sub-func=0x0004 in response direction (ServerToClient) | NOT a T0814 request event. Response-side 0x08 echo means the device acknowledged the command — no additional finding emitted. Correlation via pending table (per BC-2.14.010). |
| EC-007 | Multiple Force Listen Only commands in the same flow | Each one independently emits T0814 (no deduplication — each represents a repeated attack). Cap guard applies per emission. |
| EC-008 | FC=0x08, sub-func=0x0004, when `all_findings.len() == MAX_FINDINGS` | No finding pushed (poison-skip); `fn_code_counts[0x08]` still incremented. |

## Canonical Test Vectors

| Input (hex ADU) | Expected Output | Category |
|-----------------|----------------|----------|
| `00 01 00 00 00 06 01 08 00 04 00 00` — (TxnID=1, ProtID=0, Len=6, UnitID=1, FC=0x08, SubFunc=0x0004, Data=0x0000) — ClientToServer | T0814 Finding{category=Anomaly, verdict=Likely, confidence=High, summary="Modbus DoS: Force Listen Only Mode sent to unit 1 — device will stop responding", mitre_techniques=vec!["T0814"]} | happy-path (Force Listen Only) |
| `00 02 00 00 00 06 02 08 00 01 00 00` — (UnitID=2, FC=0x08, SubFunc=0x0001, Data=0x0000) — ClientToServer | T0814 Finding{summary="Modbus DoS: Restart Communications sent to unit 2", confidence=High, mitre_techniques=vec!["T0814"]} | happy-path (Restart Comms) |
| `00 03 00 00 00 06 01 08 00 00 FF FF` — (FC=0x08, SubFunc=0x0000 Return Query Data) — ClientToServer | No finding; `fn_code_counts[0x08]` incremented | negative (loopback sub-func) |
| `00 04 00 00 00 03 01 08 00` — (Len=3, h.length=3 < 4 — only ONE sub-func byte) | No finding; `fn_code_counts[0x08]` incremented; `parse_errors` NOT incremented | edge-case (insufficient sub-func bytes, benign skip) |
| `00 05 00 00 00 06 01 03 00 00 00 05` — (FC=0x03 Read Holding Registers) — ClientToServer | No T0814 (wrong FC) | negative |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc(0x08) == Diagnostic (no panic, correct classification) | Kani (sub-property B) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC covers Denial of Service detection via Diagnostics sub-functions, the highest-confidence single-packet ICS attack signal in the approved feature scope |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; Diagnostics sub-function parsing) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0814 — Denial of Service (ATT&CK for ICS; IcsInhibitResponseFunction tactic) |

## Related BCs

- BC-2.14.001 — depends on (MBAP parse success precondition)
- BC-2.14.008 — depends on (FC 0x08 classified as Diagnostic; defines the Diagnostic class)
- BC-2.14.019 — related to (FC 0x08 sub-func 0x000A Clear Counters: Anomaly, not T0814)
- BC-2.14.020 — related to (unusual/unknown FC anomaly — 0x08 with uncommon sub-funcs)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — Diagnostics sub-function detection branch in `on_data`
- `src/analyzer/modbus.rs` — sub-function byte extraction: `u16::from_be_bytes([data[8], data[9]])`
- `src/mitre.rs` — `technique_info("T0814")` arm (new per ADR-005 §4.2); `MitreTactic::IcsInhibitResponseFunction`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Diagnostic-class sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.6 (T0814: 0x08/0x0004 Force Listen Only + 0x0001 Restart Comms); modbus-tcp-research.md §2 (Diagnostics sub-function table: 0x0001=Restart, 0x0004=Force Listen Only); modbus-tcp-research.md §5 (T0814 severity: "near-zero FP, cheap single-packet detector") |
| **Confidence** | high [SPEC] — sub-function codes verified against Modbus.org spec V1.1b3 §6 |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell |
