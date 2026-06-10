---
document_type: behavioral-contract
level: L3
version: "2.2"
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
    change: "UPDATED (v2.0 — Decision 13, f2-fix-directives.md §13.5): T0836 is now a co-tag on the single per-PDU write finding, not a separate Finding object. The T0836 finding is mitre_techniques: [\"T0855\",\"T0836\"] on one finding. Removed all T0836-priority-over-T0835 suppression language; T0836 and T0835 are FC-subset-exclusive by definition (holding-register vs coil). Removed the separate T0855 + T0836 = 2 findings model. Targets v0.3.0."
  - version: "2.1"
    date: 2026-06-09
    change: "BC-DISCREPANCY-001 reconciliation: FC 0x17 (Read/Write Multiple Registers) added to the T0836 holding-register write set. The register-write FC set is now {0x06, 0x10, 0x16, 0x17} (consistent with BC-2.14.016). Title, description, precondition 3, invariant 1, and invariant 4 updated. Invariant 4 no longer lists 0x17 as a T0855-only FC. Orchestrator ruling: 0x17 writes holding registers -> Modify Parameter (T0836)."
  - version: "2.2"
    date: 2026-06-09
    change: "F5 spec defect fix (F7 consistency finding F2): source_ip postcondition changed from flow_key.client_ip() (non-existent accessor — FlowKey only has lower_ip/upper_ip/lower_port/upper_port) to Direction-resolved client/initiator endpoint. Write-class PDUs are always ClientToServer; resolve initiator endpoint from flow_key.lower_ip()/upper_ip() combined with the direction arg passed to on_data. Mirrors BC-2.14.013 v2.2 and BC-2.14.017 v2.2 correction."
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

# BC-2.14.014: Write FC 0x06/0x10/0x16/0x17 in Request Direction Emits Finding Tagged ["T0855","T0836"]

<!-- Previous version (v1.0): "Write FC 0x06/0x10/0x16 in Request Direction Emits T0836 Modify Parameter Finding"
     v1.0 model: separate T0836 Finding object pushed after T0855 Finding (two separate findings per PDU).
       Contained "T0836 takes priority over T0835 for same PDU" suppression language.
     v2.0 model (Decision 13): ONE finding per PDU, mitre_techniques: ["T0855","T0836"].
       No suppression language; T0836 and T0835 are FC-subset-exclusive, not priority-competing.
       Targets v0.3.0.
-->

## Description

Write operations targeting Modbus holding registers via FC 0x06 (Write Single Register),
0x10 (Write Multiple Registers), 0x16 (Mask Write Register), or 0x17 (Read/Write Multiple
Registers) are classified as setpoint or parameter modification. FC 0x17 is included because
it atomically writes holding registers in its write phase, making it functionally equivalent
to a holding-register write for threat-detection purposes (BC-DISCREPANCY-001 ruling;
consistent with BC-2.14.016 register-write set). Per Decision 13 (ADR-006), the single
per-PDU write Finding for these FCs carries `mitre_techniques: vec!["T0855", "T0836"]` —
both "Unauthorized Command Message" and "Modify Parameter" technique tags on one finding.
These four FCs write directly to holding registers, which store setpoints, alarm thresholds,
process limits, and configuration values in typical SCADA deployments. There is no separate
T0836 Finding object; the T0836 attribution is carried inline in the per-PDU finding defined
by BC-2.14.013. The discriminator between T0836 and T0835 (BC-2.14.015) is register type by
FC subset: holding registers {0x06, 0x10, 0x16, 0x17} → T0836 tag; coil outputs {0x05, 0x0F}
→ T0835 tag. These sets are definitionally non-overlapping. Targets v0.3.0.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `function_code` is one of: `0x06`, `0x10`, `0x16`, `0x17`.
   FC 0x17 (Read/Write Multiple Registers) is included as a holding-register write per
   BC-DISCREPANCY-001 ruling (consistent with BC-2.14.016).
4. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. ONE `Finding` is pushed (the same per-PDU finding specified in BC-2.14.013 postcondition 1)
   with `mitre_techniques: vec!["T0855", "T0836"]`:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus write command observed: FC 0x{fc:02X} from unit {unit_id}"`
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_techniques: vec!["T0855", "T0836"]`
   - `source_ip: Some(<client/initiator endpoint>)` — `FlowKey` has no `client_ip()` accessor;
     resolve using the `direction` arg and `flow_key.lower_ip()` / `flow_key.upper_ip()`.
     For `Direction::ClientToServer`, the client/initiator endpoint is determinable from the
     `Direction` value combined with the flow key's lower/upper address pair.
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. When the T0831 coordinated-write condition is also met (2nd+ holding-register write within
   the 5-second window), the same finding carries `mitre_techniques: vec!["T0855","T0836","T0831"]`
   (T0831 is added to the tag vec inline — not a separate finding object — per BC-2.14.016).
3. `flow.write_count` and `self.total_write_count` incremented once per PDU.
4. `self.fn_code_counts.entry(function_code)` incremented by 1.
5. **No separate T0836 finding object is created.** The T0836 attribution is carried in
   `mitre_techniques` of the single per-PDU finding. `all_findings` receives exactly one new
   entry per non-burst, non-T0831 register write.

## Invariants

1. **Tag set for holding-register writes (authoritative per Decision 13 §13.5, updated per BC-DISCREPANCY-001):**
   FC {0x06, 0x10, 0x16, 0x17} → `mitre_techniques = ["T0855", "T0836"]` in the single per-PDU finding.
   FC 0x17 is included because it writes holding registers (Modify Parameter); consistent with
   BC-2.14.016 register-write set and BC-2.14.013 Invariant 2.
   T0831 is appended to this vec when the coordinated-write window fires (see BC-2.14.016).
2. **FC-subset exclusivity (not priority-based):** T0836 and T0835 are never co-tagged on
   the same PDU finding because:
   - T0836 applies only to FCs {0x06, 0x10, 0x16, 0x17} (holding-register writes).
   - T0835 applies only to FCs {0x05, 0x0F} (coil writes).
   - These sets are disjoint; no FC is in both. The concept of "T0836 priority suppresses
     T0835" from v1.0 is SUPERSEDED and must not be used in implementation comments or tests.
     The correct framing is: T0836 and T0835 are mutually exclusive by FC subset definition.
3. **`mitre_techniques` field** (plural, `Vec<String>`) — not the v1.0 `mitre_technique: Option<String>` field.
4. FC 0x15 (Write File Record) is in the Write class but is NOT in the T0836 subset.
   It carries `mitre_techniques: ["T0855"]` only (targets file records, not holding registers).
   FC 0x17 (Read/Write Multiple Registers) IS in the T0836 subset {0x06, 0x10, 0x16, 0x17}
   and carries `mitre_techniques: ["T0855", "T0836"]` — it writes holding registers in its
   write phase (BC-DISCREPANCY-001 reconciliation).
5. **Burst finding is independent:** if the burst threshold is also tripped on this PDU,
   a separate burst Finding with `mitre_techniques: ["T0806","T0855"]` is emitted (from
   BC-2.14.017). That is a 2nd Finding, not a modification of the per-PDU finding.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC 0x06 with value field = 0xFFFF | Single finding `mitre_techniques: ["T0855","T0836"]`; value is forensic evidence captured in evidence string, not a gate. |
| EC-002 | FC 0x10 writing 125 registers (max bulk write) | Single finding with `mitre_techniques: ["T0855","T0836"]`; evidence string includes the full ADU byte range. |
| EC-003 | FC 0x16 (Mask Write Register) — AND=0xFFF0, OR=0x000F | Single finding `mitre_techniques: ["T0855","T0836"]` (parameter modification via bit-mask; 0x16 is in the holding-register subset). |
| EC-004 | FC 0x10 when `all_findings.len() == MAX_FINDINGS - 1` | Finding with `["T0855","T0836"]` would be the (MAX_FINDINGS)th finding — pushed (len was MAX_FINDINGS-1). If T0831 also fires, its tag would be appended to the same finding's vec. No second finding attempted for T0836 separately. |
| EC-005 | FC 0x06 when `all_findings.len() == MAX_FINDINGS` | No finding pushed (poison-skip); `write_count` and `fn_code_counts[0x06]` incremented normally. `dropped_findings += 1`. |
| EC-006 | Response direction carrying FC 0x10 echo | NOT a T0836 request event; response-path logic handles echo matching per BC-2.14.010. |
| EC-007 | FC 0x0F (Write Multiple Coils) | NOT in T0836 subset; single finding with `mitre_techniques: ["T0855","T0835"]` (coil-write tag set, per BC-2.14.015). No T0836 tag. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ADU hex: `00 01 00 00 00 06 01 06 00 10 01 F4` (FC=0x06, UnitID=1) — ClientToServer; 1st write on flow | ONE Finding: `{mitre_techniques=["T0855","T0836"]}` pushed; `write_count=1`, `fn_code_counts[0x06]=1` | happy-path (register write, union tag) |
| ADU hex: `00 02 00 00 00 09 02 10 00 00 00 02 04 00 64 00 C8` (FC=0x10, UnitID=2) — ClientToServer | ONE Finding: `mitre_techniques=["T0855","T0836"]`; `write_count=1`, `fn_code_counts[0x10]=1` | happy-path (multi-register) |
| ADU hex: `00 03 00 00 00 08 01 16 00 10 FF F0 00 0F` (FC=0x16 Mask Write, UnitID=1) — ClientToServer | ONE Finding: `mitre_techniques=["T0855","T0836"]`; `write_count=1`, `fn_code_counts[0x16]=1` | happy-path (mask-write) |
| ADU hex: `00 04 00 00 00 06 01 0F 00 00 00 08 01 FF` (FC=0x0F Write Multiple Coils) — ClientToServer | ONE Finding: `mitre_techniques=["T0855","T0835"]`; no T0836 (coil write, not in holding-register subset) | negative (coil write — T0836 not applicable) |
| ADU hex: FC=0x06; 2nd holding-register write within 5s T0831 window — ClientToServer | ONE Finding: `mitre_techniques=["T0855","T0836","T0831"]`; T0831 co-tagged inline | happy-path (T0831 co-tag) |
| ADU hex: `00 03 00 00 00 09 01 17 00 00 00 01 00 00 00 01 02 00 42` (FC=0x17 RW-Multiple, UnitID=1) — ClientToServer | ONE Finding: `mitre_techniques=["T0855","T0836"]`; `write_count=1`, `fn_code_counts[0x17]=1` (holding-register write; BC-DISCREPANCY-001) | happy-path (RW-multiple, register-write tag) |
| ADU hex: `00 05 00 00 00 06 01 03 00 00 00 05` (FC=0x03 Read Holding Registers) — ClientToServer | No T0836 (Read class); no T0855; no finding | negative (read FC) |

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
| MITRE Techniques | T0836 — Modify Parameter (ATT&CK for ICS; IcsImpairProcessControl tactic); T0855 — Unauthorized Command Message (always co-tagged) |

## Related BCs

- BC-2.14.006 — depends on (Write-class FC classification)
- BC-2.14.013 — composes with (this BC specifies the T0836 tag in the per-PDU finding defined there)
- BC-2.14.015 — composes with (T0835 tag applies for coil writes; FC subsets are non-overlapping, not priority-competing)
- BC-2.14.016 — composes with (T0831 tag may be appended to this finding's mitre_techniques vec)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — holding-register write detection branch in `on_data`; sets `mitre_techniques: vec!["T0855","T0836"]` for FC {0x06, 0x10, 0x16, 0x17}
- `src/mitre.rs` — `technique_info("T0836")` arm (new per ADR-005 §4.2)
- `.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani proof: Write-class completeness; sub-property B/C

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | f2-fix-directives.md §13.5 (per-PDU emission rules: FC 0x06/0x10/0x16 → ["T0855","T0836"]); ADR-006 (multi-tag design); architecture-delta.md §2.6 (T0836 detection trigger: 0x06/0x10/0x16 holding register writes) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell (mutates self.all_findings, counters) |
