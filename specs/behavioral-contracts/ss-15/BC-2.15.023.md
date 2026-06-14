---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.3: F3 story-anchor back-fill; corrected canonical DISABLE_UNSOLICITED frame App Control byte from 0x82 (FIR=1,FIN=0,CON=0,UNS=0,SEQ=2 — inconsistent with a single complete request) to 0x81 (FIR=1,FIN=0,CON=0,UNS=0,SEQ=1) matching sibling single-frame vectors (BC-2.15.008, BC-2.15.010, BC-2.15.011). Added explicit bit-breakdown annotation to the canonical test vector. — 2026-06-14"
  - "v1.4: F3 Pass-23: canonical-frame LEN reconciled to user-octet count — 05 64 09→08 (3 user octets: transport+app_ctrl+app_fc = 5+3=8). Verified against shipped build_detection_frame (dnp3_detection_tests.rs:64, dnp3_correlation_tests.rs:58: length_byte=8). Sibling to BC-2.15.011 fix (DF-SIBLING-SWEEP-001). — 2026-06-14"
  - "v1.5: F3-convergence Pass-27 — corrected FC 0x13 label in canonical test vector table from STOP_APPL to SAVE_CONFIG (IEEE 1815-2012: FC 0x13 = SAVE_CONFIGURATION; STOP_APPLICATION = 0x12). Naming-consistency fix only; behavioral classification (Management/out-of-scope for v1) unchanged. Verified against sibling BC-2.15.012 line ~102 (SAVE_CONFIG label). DF-SIBLING-SWEEP-001 confirmed no other BCs carry the STOP_APPL mislabel. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/research/dnp3-f2-scope-threshold-validation.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.023: Unsolicited-Response Enable/Disable Abuse — FC 0x15/0x14 Observed Emits T0814

## Description

When a DNP3 application function code DISABLE_UNSOLICITED (0x15) or ENABLE_UNSOLICITED (0x14)
is observed on a FIR=1 fragment, a `Finding` is emitted carrying `T0814` ("Denial of Service").
`DISABLE_UNSOLICITED` (0x15) is the canonical alarm-suppression / event-blinding primitive: an
attacker sends it to silence an outstation's event reporting, preventing the master from receiving
state changes (Inhibit-Response-Function). `ENABLE_UNSOLICITED` (0x14) is the control-plane
counterpart; though often legitimate, it is also attack-surface (Crain/Sistrunk: unsolicited
functions carry a disproportionately large share of DNP3 application-layer vulnerabilities).
Detection is per-occurrence, mirroring BC-2.15.011 (restart). The raw application FC byte is
read DIRECTLY; `classify_dnp3_fc` is NOT consulted (0x14/0x15 fall into the
`Dnp3FcClass::Management` arm and are NOT added to any classification set — BC-2.15.006 and
VP-023 Sub-B remain UNCHANGED). Source validation: dnp3-f2-scope-threshold-validation.md §Q1
GAP-1 [VERIFIED] — Chipkin quick reference FC table confirms 0x14=Enable unsolicited,
0x15=Disable unsolicited.

**Severity / confidence split:**
- FC 0x15 (DISABLE_UNSOLICITED): `Verdict::Likely`, `Confidence::Medium` — active alarm
  suppression; a recognized ICS abuse primitive; rarely legitimate outside commissioning.
- FC 0x14 (ENABLE_UNSOLICITED): `Verdict::Possible`, `Confidence::Low` — enabling unsolicited
  reporting is often legitimate (outstation startup, configuration); flagged informational to
  preserve audit trail without generating high-noise medium-severity findings.

## Preconditions

1. The validity gate (BC-2.15.004) returned `true`.
2. `has_user_data(control)` is `true` (link FC is 0x03 or 0x04).
3. `transport_is_fir(transport_octet)` is `true` (FIR=1, BC-2.15.008).
4. The raw application FC byte (`app_fc`) is `0x15` (DISABLE_UNSOLICITED) **or** `0x14`
   (ENABLE_UNSOLICITED). NOTE: this check is performed DIRECTLY on the raw FC byte, BEFORE
   or INDEPENDENT of `classify_dnp3_fc`. Do NOT pass 0x14/0x15 through classify_dnp3_fc
   to condition finding emission — those values return `Dnp3FcClass::Management` and that
   classification is NOT used by this detection path.
5. `flow.is_non_dnp3 == false`.
6. `self.all_findings.len() < MAX_FINDINGS` (DoS cap, see BC-2.15.022).

## Postconditions

**Finding emission (per-occurrence, one finding per observed 0x14 or 0x15 FC):**
1. Exactly ONE `Finding` is pushed to `self.all_findings`:
   - `category: ThreatCategory::Execution`
   - For FC 0x15 (DISABLE_UNSOLICITED):
     - `verdict: Verdict::Likely`
     - `confidence: Confidence::Medium`
     - `summary`: `"DNP3 DISABLE_UNSOLICITED observed: FC 0x15 from src={src:#06X} to dest={dest:#06X} — alarm suppression / event-blinding primitive"`
   - For FC 0x14 (ENABLE_UNSOLICITED):
     - `verdict: Verdict::Possible`
     - `confidence: Confidence::Low`
     - `summary`: `"DNP3 ENABLE_UNSOLICITED observed: FC 0x14 from src={src:#06X} to dest={dest:#06X} — unsolicited reporting control"`
   - `evidence`: one entry — `"FC=0x{fc:02X} dest={dest:#06X} src={src:#06X}"`
   - `mitre_techniques: vec!["T0814"]`
   - `source_ip: Some(<source endpoint>)` — resolved from flow_key
   - `timestamp: Some(...)` — pcap-relative capture timestamp
2. `flow.fc_counts.entry(app_fc).or_insert(0) += 1`.
3. `self.fn_code_counts.entry(app_fc).or_insert(0) += 1`.

**No one-shot guard:** detection is per-occurrence (mirrors BC-2.15.011 restart style). Each
observed DISABLE_UNSOLICITED or ENABLE_UNSOLICITED FC generates its own finding, subject only
to the global MAX_FINDINGS cap (Precondition 6).

## Invariants

1. **Per-occurrence detection**: one T0814 finding per observed DISABLE_UNSOLICITED or
   ENABLE_UNSOLICITED FC. No threshold or window guard — each occurrence is individually
   significant. This mirrors the BC-2.15.011 (restart) per-occurrence style.
2. **Raw FC check — classify_dnp3_fc NOT consulted**: the detection branch matches the raw
   `app_fc` byte directly (`app_fc == 0x15 || app_fc == 0x14`). This is intentional: the
   classifier returns `Management` for these FCs and is not used to gate this detection.
   `classify_dnp3_fc`, BC-2.15.006, and VP-023 Sub-B are UNCHANGED and remain authoritative
   for the detection-critical FC sets they cover. This BC adds a parallel direct-FC check.
3. **T0814 is the correct v19.1 technique** [MITRE: dnp3-research.md §6]: T0814 "Denial of
   Service" (IcsInhibitResponseFunction TA0107) is active and unchanged in ics-attack-19.1.
   DISABLE_UNSOLICITED maps to T0814 because it inhibits the outstation's response function
   (suppresses event reporting to the master). No new MITRE technique is introduced; T0814
   is already seeded and emitted (BC-2.15.011). MITRE catalog counts remain 23 seeded / 15
   emitted / 8 catalogue-only — UNCHANGED.
4. **Severity asymmetry**: DISABLE_UNSOLICITED (0x15) is Likely/Medium; ENABLE_UNSOLICITED
   (0x14) is Possible/Low. This reflects the attack-primitive asymmetry: disabling is the
   hostile act; enabling is usually benign but warrants an audit trail.
5. **DoS-bounded**: the global MAX_FINDINGS cap (BC-2.15.022) prevents finding flood. Under
   adversarial repetition of DISABLE_UNSOLICITED, findings are capped at MAX_FINDINGS and
   per-flow FC counters continue to update.
6. **No T0827 feed**: `restart_event_count` and `block_event_count` are NOT incremented by
   this detection. The T0827 derived-impact accumulator (BC-2.15.015) is based on restart
   and block-command events only. DISABLE_UNSOLICITED is an independent inhibit-function
   signal, not a loss-of-control precursor in the current correlation model.
7. **No interaction with unsolicited anomaly (BC-2.15.019)**: BC-2.15.019 watches for the
   *response* FC 0x82 (UNSOLICITED_RESPONSE) from the outstation. BC-2.15.023 watches for
   the *request* FCs 0x14/0x15 sent TO the outstation. They are complementary and independent.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single DISABLE_UNSOLICITED (0x15) on a new flow | Finding emitted: T0814, Likely/Medium |
| EC-002 | Single ENABLE_UNSOLICITED (0x14) on a new flow | Finding emitted: T0814, Possible/Low |
| EC-003 | DISABLE_UNSOLICITED followed immediately by ENABLE_UNSOLICITED | Two separate T0814 findings; first Likely/Medium, second Possible/Low |
| EC-004 | FC 0x14 observed — classify_dnp3_fc returns Management | Finding still emitted — detection is on raw FC, not classify output; Management classification is correct and unchanged |
| EC-005 | FC 0x15 to broadcast destination (0xFFFF) | T0814 finding emitted; no separate broadcast anomaly for this detection path (broadcast anomaly is in BC-2.15.018 for Control-class FCs; 0x15 is Management-class) |
| EC-006 | `all_findings.len() == MAX_FINDINGS` when 0x15 arrives | No finding pushed (DoS cap); FC counter still incremented |
| EC-007 | Multiple DISABLE_UNSOLICITED on same flow (adversarial flood) | One T0814 finding per occurrence up to MAX_FINDINGS; then capped |
| EC-008 | FIR=0 fragment carrying 0x15 | Not a first fragment; Precondition 3 not met; no detection (consistent with FIR=1-only App FC extraction policy, BC-2.15.008) |

## Canonical Test Vectors

**DISABLE_UNSOLICITED frame (master 1 to outstation 3):**
```
DNP3 frame:  05 64 08 C4 03 00 01 00 [hdr-crc]  C0 81 15  [data-crc]
Link:        START=0x0564, LEN=8, CTRL=0xC4 (DIR=1, PRM=1, FC=4=UNCONFIRMED_USER_DATA)
             DEST=0x0003, SRC=0x0001
Transport:   0xC0 (FIR=1, FIN=1, SEQ=0)
App Control: 0x81  // bit7=FIR=1, bit6=FIN=0, bit5=CON=0, bit4=UNS=0, bits0-3=SEQ=1
             // Matches sibling single-frame master-request vectors (BC-2.15.008, BC-2.15.010,
             // BC-2.15.011). FIN=0 is normal for master requests; only outstation RESPONSE
             // and UNSOLICITED_RESPONSE conventionally set FIN=1 in single-fragment exchanges.
             // The previous value 0x82 (FIR=1,FIN=0,UNS=0,SEQ=2) used SEQ=2 without
             // justification — corrected to SEQ=1 for consistency with sibling vectors.
App FC:      0x15 → DISABLE_UNSOLICITED → raw match, NOT via classify_dnp3_fc
```
Expected: `Finding { mitre_techniques: ["T0814"], verdict: Likely, confidence: Medium, summary: "DNP3 DISABLE_UNSOLICITED observed: FC 0x15 from src=0x0001 to dest=0x0003 — alarm suppression / event-blinding primitive" }`

**ENABLE_UNSOLICITED frame (master 1 to outstation 3):**
```
App FC:  0x14 → ENABLE_UNSOLICITED → raw match
```
Expected: `Finding { mitre_techniques: ["T0814"], verdict: Possible, confidence: Low, summary: "DNP3 ENABLE_UNSOLICITED observed: FC 0x14 from src=0x0001 to dest=0x0003 — unsolicited reporting control" }`

| FC (hex) | Name | Expected `verdict` | Expected `confidence` | Expected `mitre_techniques` |
|----------|------|--------------------|-----------------------|-----------------------------|
| `0x15` | DISABLE_UNSOLICITED | Likely | Medium | `["T0814"]` |
| `0x14` | ENABLE_UNSOLICITED | Possible | Low | `["T0814"]` |
| `0x13` | SAVE_CONFIG (optional/v2) | (no finding — not in scope for v1) | N/A | N/A |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Per-occurrence finding emission, raw FC check, severity split: effectful shell; unit test | unit test |

Note: VP-023 Sub-B is NOT a verification anchor for this BC. Sub-B verifies that
`classify_dnp3_fc` maps the detection-critical FC sets correctly. Since this BC
deliberately bypasses `classify_dnp3_fc` and keys on the raw FC, Sub-B's correctness
guarantee is orthogonal. The test-sufficient approach (unit tests for 0x14 and 0x15
directly) is appropriate here.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — detecting DISABLE_UNSOLICITED (0x15) and ENABLE_UNSOLICITED (0x14) abuse is a DNP3/ICS threat-detection requirement: these FCs are the control-plane mechanism for silencing outstation event reporting (alarm suppression), which is a recognized ICS attack primitive documented in Crain/Sistrunk research and the DNP3 attack surface literature [VERIFIED: dnp3-f2-scope-threshold-validation.md §Q1 GAP-1] |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings emitted only on port-20000 flows that passed the validity gate) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 5 |
| Stories | STORY-109 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | T0814 — Denial of Service (ICS; Inhibit Response Function tactic TA0107; active in v19.1). No new technique: T0814 is already seeded (BC-2.15.011) and in EMITTED set. Catalog counts 23/15/8 unchanged. |
| Research Source | dnp3-f2-scope-threshold-validation.md §Q1 GAP-1 [VERIFIED gap, JUDGMENT on severity]: "DISABLE_UNSOLICITED (0x15) is the classic alarm-suppression / event-blinding primitive"; Chipkin DNP3 Quick Reference FC table: 0x14=Enable unsolicited, 0x15=Disable unsolicited [VERIFIED]; Crain/Sistrunk: "disproportionate share [of vulns] in unsolicited response functions" [VERIFIED] |

## Related BCs

- BC-2.15.008 — depends on (FIR=1 gate enables App FC extraction)
- BC-2.15.006 — note: 0x14/0x15 return `Dnp3FcClass::Management` from classify_dnp3_fc; this detection does NOT use that return value — it keys on the raw FC directly. BC-2.15.006 and VP-023 Sub-B are UNCHANGED.
- BC-2.15.011 — pattern: per-occurrence detection style (mirrors restart per-occurrence semantics)
- BC-2.15.019 — composes with (BC-2.15.019 watches FC 0x82 UNSOLICITED_RESPONSE from outstation; this BC watches 0x14/0x15 control FCs to outstation — complementary, independent)
- BC-2.15.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::on_data` — new detection branch after existing FC dispatch: `if app_fc == 0x15 || app_fc == 0x14 { /* emit T0814 */ }`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.fc_counts: HashMap<u8, u64>` (existing; 0x14/0x15 entries added here)
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.fn_code_counts: HashMap<u8, u64>` (existing)
- `src/mitre.rs` — `technique_info("T0814")` arm (existing; shared with COLD_RESTART/WARM_RESTART and Modbus force-listen-only)
- `.factory/research/dnp3-f2-scope-threshold-validation.md §Q1 GAP-1` (research validation: must-add detection)
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 5` (detection extension)

## Story Anchor

STORY-109

## VP Anchors

(none — effectful shell; per-occurrence unit test is sufficient; VP-023 Sub-B is UNCHANGED and covers classify_dnp3_fc only)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-f2-scope-threshold-validation.md §Q1 GAP-1 [VERIFIED gap]; Chipkin DNP3 Quick Reference §FC table [VERIFIED: 0x14=Enable unsolicited, 0x15=Disable unsolicited]; Crain/Bratus (Dartmouth) [VERIFIED: disproportionate share of DNP3 vulns in unsolicited response functions]; dnp3-research.md §3.2 [VERIFIED: 0x14/0x15 in FC table] |
| **Confidence** | high for 0x15 (DISABLE_UNSOLICITED — recognized attack primitive); medium for 0x14 (ENABLE_UNSOLICITED — often legitimate, preserved as low-confidence audit signal) |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.fc_counts, fn_code_counts, all_findings |
| **Deterministic** | yes — same frame sequence produces same finding |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
