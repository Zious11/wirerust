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

# BC-2.14.013: Write-Class FC in Request Direction Emits T0855 Unauthorized Command Message Finding

## Description

Any Modbus PDU whose function code belongs to the Write class (0x05, 0x06, 0x0F, 0x10, 0x15,
0x16, 0x17) and whose TCP direction is `ClientToServer` (destination port 502) must produce a
`Finding` tagged with MITRE ATT&CK for ICS technique T0855 ("Unauthorized Command Message").
This is the broadest write-class detector: every write FC triggers it regardless of sub-type.
T0836, T0835, and T0831 detectors fire in addition to — not instead of — this BC for their
specific FC subsets. The finding is emitted immediately upon parsing the request PDU (not
deferred to `on_flow_close`).

## Preconditions

1. The MBAP ADU has passed the three-point validity gate:
   `protocol_id == 0x0000 AND 2 <= length <= 253 AND total ADU bytes available`.
2. The TCP direction parameter passed to `on_data` is `Direction::ClientToServer`.
3. `classify_fc(function_code)` returns `FunctionCodeClass::Write` — i.e., `function_code`
   is one of: 0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17.
4. `self.all_findings.len() < MAX_FINDINGS` (cap guard — see BC-2.14.022).

## Postconditions

1. A `Finding` is pushed to `self.all_findings` with:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Malicious` (write to an OT device is high-confidence anomaly)
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus write command observed: FC 0x{fc:02X} from unit {unit_id}"` where
     `{fc}` is the raw function code byte and `{unit_id}` is the MBAP Unit ID byte.
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_technique: Some("T0855".to_string())`
   - `source_ip: Some(flow_key.client_ip())` — the IP of the client-side endpoint.
   - `timestamp: Some(...)` — the pcap-relative capture timestamp from `on_data`'s
     `timestamp: u32` argument, converted to `DateTime<Utc>` per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. `flow.write_count` is incremented by 1.
3. `self.total_write_count` is incremented by 1.
4. `self.fn_code_counts.entry(function_code).or_insert(0)` is incremented by 1.
5. The request is inserted into `flow.pending` keyed on `(transaction_id, unit_id)` if
   `flow.pending.len() < MAX_PENDING_TRANSACTIONS` (per BC-2.14.012 / Group C).
6. Write-rate window counters are updated and the burst detector runs (per BC-2.14.017).
   A T0806 burst finding MAY also be emitted in the same `on_data` call if the rate
   threshold is exceeded (those findings are independent of this BC's postcondition).

## Invariants

1. This finding is emitted for EVERY write-class FC in a request direction PDU, with no
   deduplication. Duplicate write operations on the same target are forensically significant
   (brute-force / repeated unauthorized commands).
2. The `mitre_technique` field MUST carry `"T0855"` — the ICS-matrix discriminator is
   implicit in the `T0xxx` namespace per ADR-005 §4.1.
3. **T0855 fires independently of the T0836/T0835 priority rule.** The per-PDU co-emission
   policy (Decision 7, architecture-delta.md §2.6) governs the T0836 vs T0835 selection:
   - T0836 takes priority over T0835 for the same PDU (FC 0x06/0x10/0x16 emit T0836, NOT T0835).
   - T0835 fires only for coil-only writes (FC 0x05, 0x0F) where T0836 does not apply.
   - T0855 (this BC) is ALWAYS emitted for every write-class PDU, regardless of which of
     T0836/T0835 fires. T0855 represents the broadest "unauthorized command" signal and is
     not subject to the T0836/T0835 priority selection.
   Finding emission order for a single PDU: T0855 first, then T0836 or T0835 (whichever
   applies), then T0831 (if sequence condition met). All are independent `Finding` objects.
4. `classify_fc` is a pure function (VP-022 sub-property B): this invariant holds for all
   256 possible FC byte values without exception.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC 0x17 (Read/Write Multiple Registers) in request direction | Write class: T0855 emitted. Note: 0x17 is simultaneously a write AND a read; the write semantics dominate for threat classification. |
| EC-002 | FC 0x15 (Write File Record) in request direction | Write class: T0855 emitted. Relatively rare but standard Write-class FC — does NOT trigger the unknown-FC anomaly path (BC-2.14.020 covers truly unknown/recon FCs such as 0x11/0x2B/0x0E; 0x15 is a known Write-class FC). |
| EC-003 | FC 0x16 (Mask Write Register) in request direction | Write class: T0855 emitted AND T0836 emitted (0x16 is in the parameter-modification subset per BC-2.14.014). |
| EC-004 | `all_findings.len() == MAX_FINDINGS` when write FC arrives | No finding pushed (poison-skip). `write_count` and `fn_code_counts` still incremented (counters are not gated by the findings cap). |
| EC-005 | Response direction (Direction::ServerToClient) carrying a write-class FC echo | NOT emitted by this BC. Exception responses on write FCs are handled by the response-path attribution logic (exception-response finding per BC-2.14.019). |
| EC-006 | FC 0x05 (Write Single Coil) in request direction | T0855 emitted AND T0835 emitted (coil write is I/O image manipulation per BC-2.14.015). |
| EC-007 | Flow has no prior writes; first PDU is a write FC | T0855 emitted normally. `write_count` transitions from 0 → 1. No special case. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ADU hex: `00 01 00 00 00 06 01 06 00 10 01 F4` (TxnID=1, ProtID=0, Len=6, UnitID=1, FC=0x06, addr=0x0010, value=0x01F4) — direction=ClientToServer | `Finding{category=Execution, verdict=Malicious, confidence=Medium, summary="Modbus write command observed: FC 0x06 from unit 1", mitre_technique=Some("T0855"), direction=Some(ClientToServer)}` pushed; `write_count=1`, `fn_code_counts[0x06]=1` | happy-path |
| ADU hex: `00 02 00 00 00 06 02 0F 00 00 00 08 01 FF` (FC=0x0F Write Multiple Coils, UnitID=2) — direction=ClientToServer | T0855 Finding emitted; T0835 Finding also emitted for same PDU (0x0F is in coil-write subset) | happy-path (multi-technique) |
| ADU hex: `00 03 00 00 00 06 01 10 00 00 00 01 02 00 42` (FC=0x10 Write Multiple Registers) — direction=ClientToServer | T0855 Finding + T0836 Finding emitted; T0835 NOT emitted (T0836 priority suppresses T0835 for 0x10); `write_count=1` | happy-path (T0836 priority) |
| ADU hex: `00 04 00 00 00 06 01 03 00 00 00 05` (FC=0x03 Read Holding Registers) — direction=ClientToServer | No T0855 finding (Read class, not Write); `fn_code_counts[0x03]=1` incremented | negative (read, no emit) |
| `all_findings.len() == 10_000`; FC=0x06 write arrives | No new Finding pushed; `write_count` and `fn_code_counts[0x06]` incremented normally | edge-case (cap) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | `classify_fc(fc) == Write` iff fc in {0x05,0x06,0x0F,0x10,0x15,0x16,0x17} | Kani exhaustive proof over all 256 u8 values |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the primary write-command detection signal that constitutes the ICS analysis capability's threat-detection function for write-class function codes |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — Modbus flows are only routed after TLS/HTTP content rules fail, ensuring this BC never fires on non-Modbus flows) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0855 — Unauthorized Command Message (ATT&CK for ICS; IcsImpairProcessControl tactic) |

## Related BCs

- BC-2.14.001 — depends on (MBAP parse success precondition for this BC)
- BC-2.14.006 — depends on (Write-class FC classification — classify_fc returns Write)
- BC-2.14.014 — composes with (T0836 co-emitted for 0x06/0x10/0x16 subset)
- BC-2.14.015 — composes with (T0835 co-emitted only for coil-only 0x05/0x0F; suppressed for register FCs 0x06/0x10/0x16)
- BC-2.14.016 — composes with (T0831 co-emitted for coordinated write sequence)
- BC-2.14.017 — composes with (T0806 burst detection also checks write-class FCs)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusAnalyzer::on_data` write-detection branch
- `src/analyzer/modbus.rs` — `classify_fc` pure function (VP-022 target)
- `src/findings.rs` — `Finding` struct; `ThreatCategory::Execution`
- `src/mitre.rs` — `technique_info("T0855")` arm; `MitreTactic::IcsImpairProcessControl`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Modbus MBAP parse safety and FC boundary classification (Kani; sub-property B covers Write-class completeness)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.6 (T0855 detection trigger); modbus-tcp-research.md §5 (T0855 severity rationale) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same ADU bytes + direction always produce same finding |
| **Thread safety** | Send + Sync (Finding is owned) |
| **Overall classification** | effectful shell (mutates self.all_findings, counters) |
