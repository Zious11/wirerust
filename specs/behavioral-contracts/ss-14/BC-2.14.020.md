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
    change: "UPDATED (v2.0 — Decision 12, f2-fix-directives.md §12): T0846 → T0888 correctness fix. Recon FCs 0x11 (Report Server ID) and 0x2B/MEI 0x0E (Read Device ID) now emit T0888 (Remote System Information Discovery, TA0102 Discovery tactic), NOT T0846. T0846 (Remote System Discovery) was a documented misattribution: T0846 applies to network scanning for device existence, while T0888 applies to querying device make/model/firmware/version — which is exactly what these FCs return. FC 0x07 (Read Exception Status) REMOVED as standalone recon indicator (insufficient signal for a standalone Finding). Title updated. Traceability MITRE field updated. T0888 is a new seeded+emitted technique (per Decision 12 §12.2, SEEDED count 15→21, EMITTED 6→13). Targets v0.3.0."
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

# BC-2.14.020: Reconnaissance Function Codes (0x11, 0x2B/0x0E) Emit T0888 Remote System Information Discovery Finding

<!-- Previous version (v1.0): "Unusual or Unknown Function Code Observed Emits Anomaly Finding"
     v1.0 recon path: mitre_technique: Some("T0846") — INCORRECT attribution.
       T0846 ("Remote System Discovery") applies to network scanning to enumerate that hosts exist.
       T0888 ("Remote System Information Discovery") applies to querying device identity information
       (make/model/firmware/configuration) — which is what FC 0x11 and FC 0x2B/0x0E return.
     v2.0 correction (Decision 12): recon FCs 0x11 and 0x2B/0x0E emit mitre_techniques: ["T0888"].
       FC 0x07 (Read Exception Status) REMOVED as standalone recon indicator.
       T0888 is a new seeded+emitted ICS technique. T0846 remains seeded but is NOT emitted by Modbus.
       Targets v0.3.0.
-->

## Description

When a Modbus PDU contains a function code that is not in the standard set (i.e.,
`classify_fc` returns `FunctionCodeClass::Unknown`) or carries a Diagnostics FC 0x08 with
a sub-function not explicitly handled (not 0x0001, 0x0004, or 0x000A), the analyzer emits
an `Anomaly` finding. Reconnaissance FCs — FC 0x11 (Report Server ID) and FC 0x2B with MEI
sub-type 0x0E (Read Device Identification) — additionally emit a Finding attributed to
MITRE ATT&CK for ICS technique T0888 ("Remote System Information Discovery", TA0102 Discovery
tactic), because these FCs return device identity information: make/model, firmware version,
vendor ID, and capabilities. T0888 is the correct attribution per Decision 12 (f2-fix-directives.md §12):
T0846 ("Remote System Discovery") covers network-scanning behavior (enumerating that devices
exist); T0888 covers querying device make/model/firmware/version — the precise behavior of
these FCs. FC 0x07 (Read Exception Status) is NOT a standalone recon indicator in v1 and
does not emit a T0888 or T0846 finding. This BC does NOT cover the standard Diagnostics
sub-functions 0x0001/0x0004 (BC-2.14.018) or 0x000A (BC-2.14.019) — those have their own BCs.
Exception FCs (>= 0x80) are handled by BC-2.14.007 / BC-2.14.019.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. One of these conditions applies:
   - **Unknown FC path**: `classify_fc(function_code)` returns `FunctionCodeClass::Unknown`
     (i.e., `function_code` is not in the standard set and not >= 0x80).
   - **Reconnaissance FC path**: `function_code` is `0x11` (Report Server ID) or
     `function_code` is `0x2B` (MEI Encapsulated Interface Transport with sub-type 0x0E
     Read Device Identification, if parseable).
   - **Unhandled Diagnostics sub-function path**: `function_code == 0x08` AND
     `sub_func` is present but is not `0x0000`, `0x0001`, `0x0004`, or `0x000A`.
3. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

### Unknown FC path

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Inconclusive`
   - `confidence: Confidence::Low`
     (Unknown FCs may be vendor-specific extensions; low confidence avoids over-alerting.)
   - `summary`: `"Modbus unknown function code: 0x{fc:02X} on unit {unit_id}"`
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id} ADU bytes {start}..{end}"`.
   - `mitre_techniques: vec![]` (empty — no technique attributed for genuinely unknown FCs)
   - `source_ip: Some(flow_key.client_ip())` if ClientToServer; `Some(flow_key.server_ip())` if ServerToClient.
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(direction)` — the direction parameter from `on_data`.

### Reconnaissance FC path (0x11 or 0x2B/0x0E)

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Inconclusive`
   - `confidence: Confidence::Medium`
     (0x11 and 0x2B/0x0E in a steady-state environment are medium-confidence recon signals.)
   - `summary` (FC=0x11): `"Modbus recon: Report Server ID (FC 0x11) from unit {unit_id}"`
   - `summary` (FC=0x2B/0x0E): `"Modbus recon: Read Device Identification (MEI 0x2B/0x0E) on unit {unit_id}"`
   - `evidence`: one entry — same pattern as Unknown FC.
   - `mitre_techniques: vec!["T0888"]` — MITRE ATT&CK for ICS T0888 "Remote System Information
     Discovery" (TA0102 Discovery tactic). FC 0x11 (Report Server ID) returns device type,
     firmware version, and identification data. FC 0x2B/0x0E (Read Device Identification) returns
     vendor name, product name/code, and major/minor revision. Both FCs directly enumerate
     device identity information, matching T0888's definition precisely per Decision 12
     (f2-fix-directives.md §12). T0846 ("Remote System Discovery") is NOT used — that technique
     applies to scanning networks to enumerate whether devices exist (address-sweep behavior),
     not to querying device identity information.
   - `source_ip`, `timestamp`, `direction` as above.

### Unhandled Diagnostics sub-function path

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Inconclusive`
   - `confidence: Confidence::Low`
   - `summary`: `"Modbus unusual diagnostic sub-function: FC 0x08 SubFunc 0x{sf:04X} on unit {unit_id}"`
   - `evidence`: `"FC=0x08 SubFunc=0x{sf:04X} TxnID={txn_id:#06X} UnitID={unit_id}"`.
   - `mitre_techniques: vec![]` (empty)
   - `source_ip`, `timestamp`, `direction` as above.

## Invariants

1. **classify_fc Unknown range**: any `u8` value not in {0x01,0x02,0x03,0x04,0x05,0x06,0x07,
   0x08,0x0B,0x0C,0x0F,0x10,0x11,0x14,0x15,0x16,0x17,0x18,0x2B} and not >= 0x80
   maps to `FunctionCodeClass::Unknown`. This is the residue of the exhaustive match in
   `classify_fc` (the `_` arm). VP-022 sub-property B covers this: every u8 returns one of
   the five variants.
2. **FC 0x11 is Read-class but also recon**: `classify_fc(0x11)` returns `FunctionCodeClass::Read`.
   This is correct for the classification logic. The recon Anomaly finding for 0x11 is emitted
   ADDITIONALLY by this BC — the Unknown FC path is NOT used for 0x11. Both actions occur:
   `fn_code_counts[0x11]++` (normal accounting) and recon Anomaly finding with `["T0888"]` pushed.
3. **FC 0x2B (MEI)** is classified as `Diagnostic` by `classify_fc`. The recon anomaly for
   0x2B is only emitted when the MEI sub-type byte (byte at `offset+8`) is 0x0E (Read Device
   Identification). Other MEI sub-types (e.g., 0x0D CANopen) are not flagged in v1.
   If the ADU is too short to contain the MEI type byte, no 0x2B recon finding is emitted.
4. **FC 0x07 (Read Exception Status) is NOT a standalone recon indicator in v1.** A single
   status byte provides insufficient signal for a standalone Finding. FC 0x07 may be
   considered as a low-weight corroborator in a future sequence-aware scan detector (not in
   scope F2). Do NOT emit a T0888 or T0846 finding for FC 0x07 alone.
5. **T0846 is seeded but NOT emitted by the Modbus analyzer.** The `technique_info("T0846")`
   arm remains in `src/mitre.rs` for catalog completeness and potential future use (e.g., an
   address-sweep / arp-scan detector). No Modbus BC emits `T0846`. Using T0846 for recon FCs
   was a documented misattribution corrected by Decision 12.
6. **T0888 is a new seeded and emitted technique.** It must be present in both
   `SEEDED_TECHNIQUE_IDS` (21 total) and `EMITTED_IDS` (13 total, 7 ICS) after the F2
   feature commit. See f2-fix-directives.md §12.2 and §12.3 for authoritative counts.
7. **`mitre_techniques` field** (plural, `Vec<String>`) per ADR-006. Empty vec for no-technique
   findings; `vec!["T0888"]` for recon-path findings.
8. **No deduplication**: unknown FC and recon FC anomalies are emitted on every occurrence.
   Repeated probes are all forensically significant. Only the exception-burst (BC-2.14.019)
   uses rate-gated deduplication; this BC does not.
9. **Interaction with per-PDU T0855**: FC 0x11 and FC 0x2B are Read-class and Diagnostic-class
   respectively — neither triggers the Write-class T0855 detection path. No T0855 co-emission
   for this BC.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | FC=0x2C (not in standard set, not exception) | Unknown FC path: Anomaly{confidence=Low, summary="unknown function code: 0x2C", mitre_techniques=[]}. |
| EC-002 | FC=0x11 (Report Server ID) in request direction | Recon Anomaly{confidence=Medium, summary="Report Server ID (FC 0x11)", mitre_techniques=["T0888"]}. `fn_code_counts[0x11]++`. |
| EC-003 | FC=0x2B with MEI sub-type=0x0E (Read Device ID) | Recon Anomaly{confidence=Medium, summary="Read Device Identification (MEI 0x2B/0x0E)", mitre_techniques=["T0888"]}. |
| EC-004 | FC=0x2B with MEI sub-type=0x0D (CANopen) | No recon Anomaly (0x0D is not flagged in v1). `fn_code_counts[0x2B]++` only. |
| EC-005 | FC=0x08 with sub-func=0x0002 (not in {0,1,4,0xA}) | Unhandled Diagnostics: Anomaly{confidence=Low, summary includes "SubFunc=0x0002", mitre_techniques=[]}. |
| EC-006 | FC=0x08 with sub-func=0x0004 | Handled by BC-2.14.018 (T0814 path). This BC does NOT additionally emit an Anomaly. |
| EC-007 | FC=0x07 (Read Exception Status) | NOT flagged as standalone recon. No T0888 or T0846 finding. `fn_code_counts[0x07]++` normal accounting only. |
| EC-008 | `all_findings.len() == MAX_FINDINGS` when unknown FC arrives | No finding pushed (poison-skip). `fn_code_counts` incremented normally. |
| EC-009 | Multiple consecutive FC=0x11 PDUs from the same source | Each emits a separate T0888 Anomaly finding (no deduplication). Cap guard applies per finding. |
| EC-010 | FC=0x11 in response direction (server to client) | Recon Anomaly still emitted (the response confirms the device responded to the recon probe; the recon signal exists regardless of direction). `source_ip` = `flow_key.server_ip()`. |

## Canonical Test Vectors

| Input (hex ADU) | Expected Output | Category |
|-----------------|----------------|----------|
| `00 01 00 00 00 03 01 2C 00` — (FC=0x2C, not standard) — ClientToServer | Anomaly{category=Anomaly, verdict=Inconclusive, confidence=Low, summary="Modbus unknown function code: 0x2C on unit 1", mitre_techniques=[]} | happy-path (unknown FC, no technique) |
| `00 02 00 00 00 02 01 11` — (FC=0x11 Report Server ID, UnitID=1) — ClientToServer | Anomaly{confidence=Medium, summary="Modbus recon: Report Server ID (FC 0x11) from unit 1", mitre_techniques=["T0888"]} | happy-path (recon FC, T0888) |
| `00 03 00 00 00 04 01 2B 0E 00` — (FC=0x2B, MEI sub-type=0x0E, UnitID=1) — ClientToServer | Anomaly{confidence=Medium, summary="Modbus recon: Read Device Identification (MEI 0x2B/0x0E) on unit 1", mitre_techniques=["T0888"]} | happy-path (MEI recon, T0888) |
| `00 04 00 00 00 06 01 08 00 02 00 00` — (FC=0x08, SubFunc=0x0002) — ClientToServer | Anomaly{confidence=Low, summary includes "SubFunc 0x0002", mitre_techniques=[]} | happy-path (unhandled diag) |
| `00 05 00 00 00 06 01 08 00 04 00 00` — (FC=0x08, SubFunc=0x0004 Force Listen Only) | T0814 Finding from BC-2.14.018; no Anomaly from this BC | negative (handled by BC-2.14.018) |
| `00 06 00 00 00 03 01 07 00` — (FC=0x07 Read Exception Status) — ClientToServer | No T0888 finding; `fn_code_counts[0x07]++` only. FC 0x07 is not a standalone recon indicator. | negative (0x07 excluded) |
| `00 07 00 00 00 02 01 11` — (FC=0x11) — PRIOR: mitre_technique=Some("T0846") | mitre_techniques=["T0888"] (T0846 was the prior incorrect attribution; T0888 is the correct technique per Decision 12) | regression-guard (T0846 must NOT appear) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc Unknown arm correctness: every u8 maps to exactly one FunctionCodeClass | Kani (sub-property B) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC covers anomaly detection for unusual/unknown FCs and reconnaissance FCs, a forensic signal specific to ICS protocol analysis that identifies probing and device-identity enumeration |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; classify_fc Unknown arm; recon-FC detection branch) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0888 — Remote System Information Discovery (ATT&CK for ICS; TA0102 Discovery tactic) for recon FCs 0x11/0x2B/0x0E; empty for Unknown FC path and Unhandled Diagnostics sub-function path. T0846 is NOT emitted by the Modbus analyzer. |

## Related BCs

- BC-2.14.006 — depends on (Exception response detection — FC high bit set (>= 0x80) is Exception, not Unknown)
- BC-2.14.007 — depends on (Write-class FC classification — confirms state-changing FCs are Write, not Unknown)
- BC-2.14.008 — related to (Diagnostic class includes 0x2B; unusual Diagnostics sub-funcs handled here)
- BC-2.14.018 — related to (0x08 sub-func 0x0001/0x0004 → T0814; this BC handles unhandled sub-funcs)
- BC-2.14.019 — related to (0x08 sub-func 0x000A → anti-forensic Anomaly; this BC handles others)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — Unknown FC branch in `on_data` after `classify_fc`
- `src/analyzer/modbus.rs` — Reconnaissance FC branch (0x11, 0x2B/0x0E) in `on_data`; sets `mitre_techniques: vec!["T0888"]`
- `src/analyzer/modbus.rs` — `classify_fc` pure function (VP-022 Unknown-arm)
- `src/mitre.rs` — `technique_info("T0888")` arm — NEW seeded+emitted (Decision 12 §12.2)
- `src/mitre.rs` — `technique_info("T0846")` arm — KEPT seeded, NOT emitted by Modbus
- `.factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` §Decision 12

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Unknown-class sub-property B (exhaustive match)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | f2-fix-directives.md §12 (Decision 12: T0846→T0888 correctness fix; §12.1 current state; §12.2 corrected technique_info arms; §12.3 corrected MITRE counts); architecture-delta.md §4.2 (T0888 new arm; T0846 non-emitted note); architecture-delta.md §4.3 (corrected SEEDED=21, EMITTED=13 counts) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell |
