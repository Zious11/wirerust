---
document_type: behavioral-contract
level: L3
version: "2.0"
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
  - version: "2.0"
    date: 2026-06-09
    change: "UPDATED (v2.0 — Decision 13, f2-fix-directives.md §13.5): Multi-tag co-emission model. T0855 is no longer a standalone per-write finding. It is co-included in the mitre_techniques vec of every write-class PDU finding alongside T0836 or T0835 (union tagging). One finding per PDU replaces two-plus separate findings. Previous version (v1.0) emitted T0855 as a separate Finding object first, followed by T0836/T0835; this version fuses them. Targets v0.3.0 (mitre_techniques: Vec<String> breaking change)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
  - .factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md
input-hash: TBD
---

# BC-2.14.013: Write-Class FC in Request Direction Emits Multi-Tag Finding Carrying T0855 and Applicable Technique Tags

<!-- Previous version (v1.0): "Write-Class FC in Request Direction Emits T0855 Unauthorized Command Message Finding"
     v1.0 model: T0855 was emitted as a STANDALONE Finding object first, then T0836 or T0835 as a SEPARATE Finding.
     v2.0 model (Decision 13): ONE finding per write-class PDU carrying ALL applicable technique tags in mitre_techniques: Vec<String>.
     This is a v0.3.0 breaking schema change (mitre_technique: Option<String> → mitre_techniques: Vec<String>).
-->

## Description

Any Modbus PDU whose function code belongs to the Write class (0x05, 0x06, 0x0F, 0x10, 0x15,
0x16, 0x17) and whose TCP direction is `ClientToServer` (destination port 502) must produce a
single `Finding` carrying `mitre_techniques: Vec<String>` that includes T0855 ("Unauthorized
Command Message") and all other applicable technique tags for that PDU (T0836 for register
writes, T0835 for coil writes). This is the broadest write-class detector: every write FC
triggers exactly one per-PDU finding with a union of applicable technique tags. T0836, T0835,
and T0831 are now co-tags on this finding rather than separate Finding objects. Volume control
is achieved via burst aggregation (one burst finding per window overflow), not via tag
suppression or separate per-technique findings. The finding is emitted immediately upon
parsing the request PDU (not deferred to `on_flow_close`). Targets v0.3.0.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate:
   `protocol_id == 0x0000 AND 2 <= length <= 253 AND total ADU bytes available`.
2. The TCP direction parameter passed to `on_data` is `Direction::ClientToServer`.
3. `classify_fc(function_code)` returns `FunctionCodeClass::Write` — i.e., `function_code`
   is one of: 0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17.
4. `self.all_findings.len() < MAX_FINDINGS` (cap guard — see BC-2.14.022).

## Postconditions

1. Exactly ONE `Finding` is pushed to `self.all_findings` per write-class PDU, with:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely` (write to an OT device is high-confidence anomaly)
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus write command observed: FC 0x{fc:02X} from unit {unit_id}"` where
     `{fc}` is the raw function code byte and `{unit_id}` is the MBAP Unit ID byte.
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_techniques`: a `Vec<String>` containing ALL applicable technique tags for this PDU:
     - FC in {0x06, 0x10, 0x16}: `vec!["T0855", "T0836"]` — unauthorized command + modify parameter
     - FC in {0x05, 0x0F}: `vec!["T0855", "T0835"]` — unauthorized command + I/O image manipulation
     - FC in {0x15, 0x17}: `vec!["T0855"]` — unauthorized command only (no register/coil subtype)
     - T0831 (coordinated write) contributes to the SAME finding's tag set if the T0831 window
       condition is met on this PDU (see BC-2.14.016 §Postconditions — the T0831 tag is added
       inline to the `mitre_techniques` vec of this per-PDU finding, not emitted as a separate
       finding). Example: a 2nd holding-register write within the T0831 5s window yields
       `vec!["T0855", "T0836", "T0831"]`.
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
   A T0806 burst finding MAY also be emitted in the same `on_data` call as a SEPARATE Finding
   if the rate threshold is exceeded — `mitre_techniques: vec!["T0806", "T0855"]`. That burst
   finding is distinct from the per-PDU finding in postcondition 1.

## Invariants

1. **One finding per write-class PDU** carrying the full union of applicable technique tags.
   There is no separate T0855 finding object; T0855 is always present in `mitre_techniques`
   of the per-PDU write finding.
2. **Tag union rules** (authoritative, per Decision 13 §13.5):
   - Holding-register writes (FC 0x06, 0x10, 0x16): `mitre_techniques = ["T0855", "T0836"]`
   - Coil writes (FC 0x05, 0x0F): `mitre_techniques = ["T0855", "T0835"]`
   - File/multi writes (FC 0x15, 0x17): `mitre_techniques = ["T0855"]`
   - When T0831 window condition is also met (2nd+ holding-register write within 5s):
     `mitre_techniques = ["T0855", "T0836", "T0831"]`
   - T0836 and T0835 are NEVER both in the same finding's tag list. They are definitionally
     exclusive: T0836 is for holding-register writes, T0835 is for coil writes. The previous
     v1.0 "T0836 priority suppresses T0835" language no longer applies — there is no
     suppression because the membership is defined by FC subset, not by priority selection.
3. **No tag suppression, no separate-finding priority:** the v1.0 concept of "T0836 takes
   priority over T0835 for the same PDU" is SUPERSEDED. Tags are added by FC-subset
   membership (union), not by priority selection. Since no FC is simultaneously a
   holding-register write AND a coil write, T0836 and T0835 never co-tag the same PDU.
4. **`mitre_techniques` field** (not `mitre_technique`) is used per ADR-006 and BC-2.09.001
   v2.0. The field is `Vec<String>` with `#[serde(skip_serializing_if = "Vec::is_empty")]`.
5. **Burst finding is independent:** the T0806+T0855 burst finding (from BC-2.14.017) is a
   separate Finding object emitted alongside (not instead of) the per-PDU write finding. The
   burst finding carries its own `mitre_techniques: vec!["T0806", "T0855"]`.
6. `classify_fc` is a pure function (VP-022 sub-property B): this invariant holds for all
   256 possible FC byte values without exception.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC 0x17 (Read/Write Multiple Registers) in request direction | Write class: single finding with `mitre_techniques: ["T0855"]` only. 0x17 is not in the holding-register or coil-write subset. |
| EC-002 | FC 0x15 (Write File Record) in request direction | Write class: single finding with `mitre_techniques: ["T0855"]` only. Not a register or coil write. |
| EC-003 | FC 0x16 (Mask Write Register) in request direction | Single finding with `mitre_techniques: ["T0855", "T0836"]`. 0x16 is in the holding-register subset (BC-2.14.014). |
| EC-004 | `all_findings.len() == MAX_FINDINGS` when write FC arrives | No finding pushed (poison-skip). `write_count` and `fn_code_counts` still incremented. |
| EC-005 | Response direction (Direction::ServerToClient) carrying a write-class FC echo | NOT emitted by this BC. Exception responses handled by BC-2.14.019. |
| EC-006 | FC 0x05 (Write Single Coil) in request direction | Single finding with `mitre_techniques: ["T0855", "T0835"]`. |
| EC-007 | FC 0x0F (Write Multiple Coils) in request direction | Single finding with `mitre_techniques: ["T0855", "T0835"]`. |
| EC-008 | FC 0x06 (2nd holding-register write within T0831 5s window) | Single finding with `mitre_techniques: ["T0855", "T0836", "T0831"]`. T0831 tag is included in this finding's vec; no separate T0831 finding object. |
| EC-009 | FC 0x06 tipping the T0806 burst threshold | Per-PDU finding: `["T0855", "T0836"]`. Additionally, a separate burst Finding: `["T0806", "T0855"]`. Both pushed if cap allows (2 findings total). |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ADU hex: `00 01 00 00 00 06 01 06 00 10 01 F4` (FC=0x06, UnitID=1) — ClientToServer; 1st write on flow | Single Finding: `{category=Execution, verdict=Likely, confidence=Medium, mitre_techniques=["T0855","T0836"]}` pushed; `write_count=1`, `fn_code_counts[0x06]=1` | happy-path (register write) |
| ADU hex: `00 02 00 00 00 06 02 0F 00 00 00 08 01 FF` (FC=0x0F Write Multiple Coils, UnitID=2) — ClientToServer | Single Finding: `mitre_techniques=["T0855","T0835"]`; `write_count=1`, `fn_code_counts[0x0F]=1` | happy-path (coil write) |
| ADU hex: `00 03 00 00 00 06 01 06 00 10 01 F4` (FC=0x06) — ClientToServer; 2nd holding-register write within 5s T0831 window | Single Finding: `mitre_techniques=["T0855","T0836","T0831"]`; T0831 tag co-included (see BC-2.14.016) | happy-path (T0831 co-tag) |
| ADU hex: `00 04 00 00 00 06 01 03 00 00 00 05` (FC=0x03 Read Holding Registers) — ClientToServer | No finding (Read class, not Write); `fn_code_counts[0x03]=1` incremented | negative (read, no emit) |
| `all_findings.len() == 10_000`; FC=0x06 write arrives | No finding pushed; `write_count` and `fn_code_counts[0x06]` incremented normally | edge-case (cap) |
| ADU hex: FC=0x15 Write File Record — ClientToServer | Single Finding: `mitre_techniques=["T0855"]` only (no register/coil subtype) | happy-path (file record write) |

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
| MITRE Techniques | T0855 — Unauthorized Command Message (always); T0836 — Modify Parameter (register writes); T0835 — Manipulate I/O Image (coil writes); T0831 — Manipulation of Control (when coordinated-write window met) |

## Related BCs

- BC-2.14.001 — depends on (MBAP parse success precondition for this BC)
- BC-2.14.006 — depends on (Write-class FC classification — classify_fc returns Write)
- BC-2.14.014 — composes with (T0836 is co-tagged for FC 0x06/0x10/0x16 in this finding's mitre_techniques)
- BC-2.14.015 — composes with (T0835 is co-tagged for FC 0x05/0x0F in this finding's mitre_techniques)
- BC-2.14.016 — composes with (T0831 is co-tagged when coordinated-write window condition met)
- BC-2.14.017 — composes with (T0806 burst detection also runs on write-class FCs; emits separate burst Finding)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusAnalyzer::on_data` write-detection branch
- `src/analyzer/modbus.rs` — `classify_fc` pure function (VP-022 target)
- `src/findings.rs` — `Finding` struct with `mitre_techniques: Vec<String>` (v0.3.0)
- `src/mitre.rs` — `technique_info("T0855")` arm; `MitreTactic::IcsImpairProcessControl`
- `.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Modbus MBAP parse safety and FC boundary classification (Kani; sub-property B covers Write-class completeness)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | f2-fix-directives.md §13.5 (co-emission model; union tagging); ADR-006 (multi-technique finding design decision); architecture-delta.md §2.6 (T0855 detection trigger; per-PDU multi-tag model) |
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
