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

# BC-2.14.014: Write FC 0x06/0x10/0x16 in Request Direction Emits T0836 Modify Parameter Finding

## Description

Write operations targeting Modbus holding registers via FC 0x06 (Write Single Register),
0x10 (Write Multiple Registers), or 0x16 (Mask Write Register) are classified as setpoint
or parameter modification and emit a T0836 ("Modify Parameter") finding. These three FCs
write directly to holding registers, which store setpoints, alarm thresholds, process limits,
and configuration values in typical SCADA deployments. This BC co-fires alongside BC-2.14.013
(T0855) for the same PDU — the T0836 finding is a refinement that names the specific parameter
manipulation technique. The discriminator between T0836 and T0835 (BC-2.14.015) is register
type: holding registers = parameters/setpoints → T0836; coil output image = I/O state → T0835.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `function_code` is one of: `0x06`, `0x10`, `0x16`.
4. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Malicious`
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus parameter/setpoint write: FC 0x{fc:02X} to unit {unit_id}"` where
     `{fc}` is the function code byte and `{unit_id}` is the MBAP Unit ID byte.
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_technique: Some("T0836".to_string())`
   - `source_ip: Some(flow_key.client_ip())`
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. A T0855 Finding is ALSO emitted for the same PDU (per BC-2.14.013 postcondition 1).
   Both findings are in `all_findings`; T0855 is pushed first.
3. T0835 is NOT emitted for this PDU when T0836 applies. The T0836 vs T0835 priority rule
   (Decision 7, architecture-delta.md §2.6): T0836 takes priority for FCs {0x06, 0x10, 0x16}.
   When T0836 fires, T0835 is SKIPPED for that PDU. T0835 is reserved for coil-only writes
   (FC 0x05, 0x0F) where T0836 does not apply. This means a single holding-register write PDU
   emits AT MOST TWO findings: T0855 + T0836 (not three: T0855 + T0836 + T0835).
4. `flow.write_count` and `self.total_write_count` incremented (via BC-2.14.013 step 2/3
   — write-count is incremented once per write-class PDU, not once per emitted finding).
5. `self.fn_code_counts.entry(function_code)` incremented by 1.

## Invariants

1. **Technique discriminator rule (T0836 vs T0835 — PRIORITY SELECTION):**
   Per Decision 7 (architecture-delta.md §2.6), a single write PDU emits AT MOST ONE
   write-technique finding from the T0836/T0835 tier:
   - **T0836 (this BC) takes priority**: FC targets holding registers (0x06 / 0x10 / 0x16).
     When T0836 fires, T0835 is NOT emitted for the same PDU.
   - **T0835 (BC-2.14.015) fires only when T0836 does NOT apply**: FC targets coil outputs
     exclusively (0x05, 0x0F). For FC 0x06 and 0x10, earlier versions incorrectly co-fired
     both T0836 and T0835; v1 policy is T0836 takes priority and T0835 is suppressed.
   - **T0855 always fires independently**: T0855 is NOT subject to the T0836/T0835 priority
     selection. It fires for every write-class PDU regardless.
   - T0831 (BC-2.14.016): coordinated sequence detector — separate path, separate emission rule.
     Single write → T0836 + T0855; coordinated sequence → T0836 + T0855 + T0831 (per window).
   - FC 0x16 (Mask Write Register) emits only T0836 and T0855. T0835 NOT emitted for 0x16.
2. The `mitre_technique` field MUST carry `"T0836"` (ICS namespace, `T0xxx` format).
3. FC 0x15 (Write File Record) and 0x17 (Read/Write Multiple Registers) are in the Write
   class but are NOT in the T0836 subset. They emit T0855 only. Write File Record has no
   standard setpoint semantics. FC 0x17 in v1 emits T0855 only — neither T0836 nor T0835 fires
   for 0x17 (T0835 is suppressed for 0x17 under the v1 simplification; see BC-2.14.015 Invariant 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC 0x06 with value field = 0xFFFF | T0836 emitted normally; value is forensic evidence but does not gate the finding. |
| EC-002 | FC 0x10 writing 125 registers in one PDU (max bulk write) | Single T0836 finding; evidence string includes the full ADU byte range. |
| EC-003 | FC 0x16 (Mask Write Register) — AND-mask=0xFFF0, OR-mask=0x000F | T0836 emitted (parameter modification via bit-mask); T0855 emitted; T0835 NOT emitted (T0836 priority). |
| EC-004 | FC 0x10 when `all_findings.len() == MAX_FINDINGS - 1` | T0855 pushed (len becomes MAX_FINDINGS); T0836 NOT pushed (guard fails). Counters still incremented. T0835 is also skipped (T0836 priority means it would not fire anyway). |
| EC-005 | FC 0x06 when `all_findings.len() == MAX_FINDINGS - 2` | T0855 and T0836 both pushed (two findings, two slots available). T0835 is NOT emitted per priority rule regardless of slots. |
| EC-006 | Response direction carrying FC 0x10 echo | NOT a T0836 request event; response-path logic handles echo matching per BC-2.14.010. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ADU hex: `00 01 00 00 00 06 01 06 00 10 01 F4` (FC=0x06, addr=0x0010, value=0x01F4, UnitID=1) — ClientToServer | T0855 Finding + T0836 Finding emitted (T0836 priority: T0835 NOT emitted); `write_count=1`, `fn_code_counts[0x06]=1` | happy-path (T0836 priority over T0835) |
| ADU hex: `00 02 00 00 00 09 02 10 00 00 00 02 04 00 64 00 C8` (FC=0x10, UnitID=2, addr=0, qty=2, values=[100,200]) — ClientToServer | T0855 + T0836 emitted; T0835 NOT emitted (T0836 priority); `write_count=1`, `fn_code_counts[0x10]=1` | happy-path (multi-register, no T0835) |
| ADU hex: `00 03 00 00 00 08 01 16 00 10 FF F0 00 0F` (FC=0x16 Mask Write, UnitID=1, addr=0x0010, AND=0xFFF0, OR=0x000F) — ClientToServer | T0855 + T0836 emitted; T0835 NOT emitted; `write_count=1`, `fn_code_counts[0x16]=1` | happy-path (mask-write, T0836 only) |
| ADU hex: `00 04 00 00 00 06 01 0F 00 00 00 08 01 FF` (FC=0x0F Write Multiple Coils) — ClientToServer | T0855 + T0835 emitted; T0836 NOT emitted (coil write, not in T0836 subset) | negative (not T0836 FC; T0835 applies) |
| ADU hex: `00 05 00 00 00 06 01 03 00 00 00 05` (FC=0x03 Read Holding Registers) — ClientToServer | No T0836 (Read class); no T0855 | negative (read FC) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc total over all 256 u8 values — Write class correctness | Kani (sub-property B/C) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC covers the parameter-modification detection path of the ICS analysis capability, the signal most directly associated with setpoint manipulation attacks |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0836 — Modify Parameter (ATT&CK for ICS; IcsImpairProcessControl tactic) |

## Related BCs

- BC-2.14.006 — depends on (Write-class FC classification)
- BC-2.14.013 — composes with (T0855 co-emitted for same PDU)
- BC-2.14.015 — composes with (T0835 emits only for coil-only 0x05/0x0F; suppressed for register FCs 0x06/0x10/0x16 per T0836 priority)
- BC-2.14.016 — composes with (T0831 emitted if coordinated write sequence detected)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — holding-register write detection branch in `on_data`
- `src/mitre.rs` — `technique_info("T0836")` arm (new per ADR-005 §4.2)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani proof: Write-class completeness; sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.6 (T0836 detection trigger: 0x06/0x10/0x16 holding register writes); modbus-tcp-research.md §5 (T0836 severity rationale: setpoints/alarm thresholds); ADR-005 §4.2 |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell (mutates self.all_findings, counters) |
