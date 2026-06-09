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
    change: "UPDATED (v2.0 — Decision 13, f2-fix-directives.md §13.5): T0835 is now a co-tag on the single per-PDU write finding for coil writes, not a separate Finding object. The coil-write finding carries mitre_techniques: [\"T0855\",\"T0835\"]. Removed all \"T0835 suppressed / T0836 priority\" language. T0835 applies for coil FCs {0x05,0x0F} because they are coil writes (FC-subset definition), not because T0836 does not suppress them. Precondition 3 simplified to FC membership only. Targets v0.3.0."
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

# BC-2.14.015: Write FC to Coil Output Only ({0x05, 0x0F}) Emits Finding Tagged ["T0855","T0835"]

<!-- Previous version (v1.0): "Write FC to Coil Output Only ({0x05, 0x0F}) Emits T0835 Manipulate I/O Image Finding"
     v1.0 model: T0835 was emitted as a SEPARATE Finding object; precondition 3 framed T0835 as firing
       "because T0836 does not apply / T0836 priority suppresses T0835 for register FCs".
     v2.0 model (Decision 13): ONE finding per PDU, mitre_techniques: ["T0855","T0835"] for coil writes.
       T0835 membership is defined by FC subset {0x05,0x0F}, not by T0836 suppression logic.
       All "suppressed", "priority", and "most-specific" language removed. Targets v0.3.0.
-->

## Description

Write operations that alter the Modbus coil output image — FC 0x05 (Write Single Coil) and
FC 0x0F (Write Multiple Coils) — emit a single per-PDU Finding carrying `mitre_techniques:
vec!["T0855", "T0835"]`. T0835 ("Manipulate I/O Image") is included because these FCs write
directly to the coil output image, the device's in-memory representation of its physical
process state. Writing to it can mask a real process condition or trigger actuators without
authorization. Per Decision 13 (ADR-006), the T0835 attribution and the T0855 attribution
are co-tagged on one finding — there is no separate T0835 Finding object. The T0835 tag
applies to {0x05, 0x0F} by FC-subset definition (coil-write class). T0836 applies to
{0x06, 0x10, 0x16} (holding-register writes). These sets are disjoint; there is no
suppression mechanism — the tags are determined by what kind of write the FC performs.
Targets v0.3.0.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `function_code` is one of: `0x05`, `0x0F`. These are the coil-output write FCs for
   which T0835 ("Manipulate I/O Image") applies. Holding-register write FCs {0x06, 0x10, 0x16}
   are handled by BC-2.14.014 (carry T0836 tag instead). FC 0x17 and FC 0x15 carry T0855
   only (per BC-2.14.013). Precondition 3 is purely an FC-subset membership check, not a
   priority or suppression rule.
4. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. ONE `Finding` is pushed (the same per-PDU finding specified in BC-2.14.013 postcondition 1
   for coil-write FCs) with `mitre_techniques: vec!["T0855", "T0835"]`:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus write command observed: FC 0x{fc:02X} from unit {unit_id}"`
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_techniques: vec!["T0855", "T0835"]`
   - `source_ip: Some(flow_key.client_ip())`
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. `flow.write_count` and `self.total_write_count` incremented once per PDU.
3. `self.fn_code_counts.entry(function_code)` incremented by 1.
4. **No separate T0835 finding object is created.** The T0835 attribution is carried in
   `mitre_techniques` of the single per-PDU finding. The T0855 and T0835 attributions
   are fused into one Finding, reducing finding count per PDU from 2 (v1.0) to 1 (v2.0).
5. A T0836 tag is NOT present in this finding because FC 0x05 and FC 0x0F are not
   holding-register writes. This is an FC-membership fact, not a priority/suppression decision.

## Invariants

1. **T0835 FC subset (v2 — coil-only writes):** {0x05, 0x0F}.
   These FCs are coil-output writes. T0835 ("Manipulate I/O Image") tags apply to them
   because they write to the coil I/O image. This is a definitional assignment:
   - FC 0x05 Write Single Coil: directly flips one output bit in the coil I/O image.
   - FC 0x0F Write Multiple Coils: bulk flip of output coil image.
2. **FCs NOT in T0835 subset and why:**
   - {0x06, 0x10, 0x16}: holding-register writes — carry T0836 tag (BC-2.14.014).
   - 0x15 Write File Record: targets file records, not the real-time I/O image.
   - 0x17 Read/Write Multiple Registers: carries T0855 only per BC-2.14.013 (neither register
     nor coil subtype tag applies under the v1 simplification).
   - 0x08 Diagnostics: state management, not I/O image (T0814, see BC-2.14.018).
3. **`mitre_techniques` field** (plural, `Vec<String>`) per ADR-006 — not `mitre_technique: Option<String>`.
4. **T0835 and T0836 are never co-tagged on the same PDU finding.** This follows from the
   disjoint FC subsets: {0x05, 0x0F} and {0x06, 0x10, 0x16} have no common element. No
   implementation guard is needed; the FC-subset check enforces it automatically.
5. **Burst finding is independent:** if the burst threshold is also tripped on this PDU,
   a separate burst Finding with `mitre_techniques: ["T0806","T0855"]` is emitted (from
   BC-2.14.017). That is an additional Finding, not a modification of the per-PDU finding.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC 0x05 writing coil 0x0000 (coil 0, value=ON, 0xFF00) | ONE finding: `mitre_techniques: ["T0855","T0835"]`. No T0836 (0x05 is not a holding-register write FC). |
| EC-002 | FC 0x05 writing coil 0x0000 with value 0x0000 (= OFF) | ONE finding: `mitre_techniques: ["T0855","T0835"]`. Turning off an actuator is equally significant forensically. |
| EC-003 | FC 0x0F bulk-writing 2000 coils (max spec-compliant bulk coil write) | ONE finding `mitre_techniques: ["T0855","T0835"]`; the quantity field is captured in evidence bytes. |
| EC-004 | FC 0x17 (Read/Write Multiple Registers) | ONE finding `mitre_techniques: ["T0855"]` only (not in coil-write subset; carries T0855 only per BC-2.14.013). |
| EC-005 | FC 0x16 in request direction | ONE finding `mitre_techniques: ["T0855","T0836"]` — mask-write is a holding-register operation, handled by BC-2.14.014. T0835 not applicable (not a coil write). |
| EC-006 | FC 0x15 (Write File Record) | ONE finding `mitre_techniques: ["T0855"]` only. T0835 not applicable. |
| EC-007 | `all_findings` at MAX_FINDINGS - 1 when FC 0x05 (coil write) arrives | The ONE finding with `["T0855","T0835"]` fills the last slot. `dropped_findings` stays 0. Counters incremented. If cap is already at MAX_FINDINGS, finding is dropped and `dropped_findings += 1`. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ADU: `00 01 00 00 00 06 01 05 00 00 FF 00` (FC=0x05 Write Single Coil, coil=0, value=ON, UnitID=1) — ClientToServer | ONE finding: `mitre_techniques=["T0855","T0835"]`; `write_count=1`, `fn_code_counts[0x05]=1` | happy-path (coil write, union tag) |
| ADU: `00 02 00 00 00 08 01 0F 00 00 00 04 01 0F` (FC=0x0F Write Multiple Coils, UnitID=1, qty=4, byte=0x0F) — ClientToServer | ONE finding: `mitre_techniques=["T0855","T0835"]`; `write_count=1`, `fn_code_counts[0x0F]=1` | happy-path (bulk coil) |
| ADU: `00 03 00 00 00 09 01 17 00 00 00 01 00 00 00 01 02 00 42` (FC=0x17 RW-Multiple, UnitID=1) — ClientToServer | ONE finding: `mitre_techniques=["T0855"]` only (FC 0x17 carries T0855 only; not a coil-write FC) | edge-case (RW-multiple, T0855 only) |
| ADU: `00 04 00 00 00 06 01 06 00 10 01 F4` (FC=0x06 Write Single Register) — ClientToServer | ONE finding: `mitre_techniques=["T0855","T0836"]`; T0835 NOT applicable (holding-register write, not coil write) | negative (register write — T0835 does NOT apply to 0x06) |
| ADU: `00 05 00 00 00 08 01 16 00 10 FF F0 00 0F` (FC=0x16 Mask Write) — ClientToServer | ONE finding: `mitre_techniques=["T0855","T0836"]`; T0835 NOT applicable (mask-write is holding-register) | negative (mask-write — T0835 excluded) |

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
| MITRE Techniques | T0835 — Manipulate I/O Image (ATT&CK for ICS; IcsImpairProcessControl tactic); T0855 — Unauthorized Command Message (always co-tagged) |

## Related BCs

- BC-2.14.013 — composes with (this BC specifies the T0835 co-tag in the per-PDU finding defined there for coil FCs)
- BC-2.14.014 — composes with (T0836 tag applies for holding-register FCs; FC subsets are non-overlapping by definition, not by priority)
- BC-2.14.016 — composes with (T0831 window tracks holding-register writes only; coil writes do NOT contribute to T0831 window counter)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — I/O image write detection branch in `on_data`; sets `mitre_techniques: vec!["T0855","T0835"]` for FC {0x05,0x0F}
- `src/mitre.rs` — `technique_info("T0835")` arm (new per ADR-005 §4.2)
- `.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Write-class sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | f2-fix-directives.md §13.5 (per-PDU emission rules: FC 0x05/0x0F → ["T0855","T0835"]); ADR-006 (multi-tag design decision); architecture-delta.md §2.6 (T0835 detection trigger: coil-only I/O image writes {0x05, 0x0F}) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell |
