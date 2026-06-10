---
document_type: spec-changelog
title: "wirerust Specification Changelog"
status: active
producer: product-owner
---

# wirerust Specification Changelog

All notable changes to the specification artifacts (PRD, BCs, domain spec, architecture)
are recorded here. Entries follow MAJOR.MINOR versioning: MINOR for new capabilities
added without breaking existing BCs; MAJOR for breaking changes (BC retirement, interface
changes, invariant rewrites).

---

## [v19-remap-2026-06-10] — 2026-06-10

### MINOR: MITRE ATT&CK for ICS v19 Remap — T0855 → T1692.001, T0856 → T1692.002

**Summary:** 1:1 technique-ID remap driven by DF-VALIDATION-001-validated defect (issue #222).
MITRE ATT&CK for ICS v19.0 (released 2026-04-28) introduced sub-techniques to the ICS matrix
for the first time and simultaneously revoked T0855 and T0856:

- **T0855 "Unauthorized Command Message" (REVOKED)** → **T1692.001 "Unauthorized Message:
  Command Message"** (ICS sub-technique under parent T1692 "Unauthorized Message")
- **T0856 "Spoof Reporting Message" (REVOKED)** → **T1692.002 "Unauthorized Message:
  Reporting Message"** (ICS sub-technique under T1692)

Tactic assignment unchanged for both: `IcsImpairProcessControl`.

**Scope:** Spec artifacts only. Source code (src/), test files, and story bodies are out of
scope for this burst; those are handled by implementer/story-writer in subsequent bursts.

**Authoritative research docs:**
- `.factory/research/mitre-ics-v19-catalog-audit.md` (audit of all affected IDs)
- `.factory/research/dnp3-mitre-verification.md` (cross-verification for DNP3 techniques)

**BCs updated (all T0855 → T1692.001 in live spec body):**
- SS-14 (Modbus): BC-2.14.006 v1.1, BC-2.14.007 v1.1, BC-2.14.008 v1.1, BC-2.14.011 v1.1,
  BC-2.14.013 v2.3, BC-2.14.014 v2.3, BC-2.14.015 v2.3, BC-2.14.016 v2.2, BC-2.14.017 v2.4,
  BC-2.14.018 v1.2, BC-2.14.019 v1.4, BC-2.14.020 v2.1, BC-2.14.022 v2.1, BC-2.14.024 v2.1
- SS-11 (Reporting): BC-2.11.001 v1.6, BC-2.11.013 v1.7, BC-2.11.017 v1.6, BC-2.11.020 v1.6,
  BC-2.11.024 v1.6
- SS-10 (MITRE catalog): BC-2.10.008 v1.6
- SS-09 (Finding model): BC-2.09.001 v1.5, BC-2.09.006 v1.6

**Other artifacts updated:**
- BC-INDEX.md: titles and group notes for BC-2.14.013/.014/.015/.016/.017 and BC-2.11.017
- prd.md: version bumped to 1.4; all live body T0855 references updated to T1692.001;
  version 1.4 delta note added
- spec-changelog.md: this entry

**Historical references intentionally preserved (not updated):**
- All `modified:` YAML entries predating this change that mention T0855 in their `change:` text
- HTML `<!-- Previous version... -->` comments in BC files (they describe historical spec state)
- v1.5 modified entry in BC-2.11.017 mentioning "MITRE: T0855, T0836" format
- Prior changelog entries (lines below this entry)

**Note on ICS sub-technique format:** T1692.001 and T1692.002 use the `Txxxx.NNN`
sub-technique format introduced in ATT&CK v19 for the ICS matrix. Any BC or validator
that documents the allowed MITRE ID format/regex must accept this format (coordinate with
architect's VP-007 update).

---

## [1.6] — 2026-06-09

### MINOR: Holdout Blemish-1 Fix — BC-2.14.019 Exception-Burst Recon 0x01/0x02 → T0888

**Summary:** Holdout evaluation blemish-1 for Feature #7 v0.4.0 (Modbus TCP analyzer).
The exception-burst anomaly detection for exception codes 0x01 (Illegal Function = FC scanning)
and 0x02 (Illegal Data Address = register-map enumeration) was previously untagged
(`mitre_techniques: vec![]`), citing "no clean single ICS indicator" per research §7.

Orchestrator decision: both anomaly patterns ARE a form of Remote System Information Discovery
(T0888, TA0102 Discovery) and now map to T0888, consistent with the established
recon→T0888 mapping for FCs 0x11 and 0x2B/0x0E (BC-2.14.020, Decision 12):
- Exception 0x01 (Illegal Function): FC scanning discovers which function codes the device
  supports — exactly the query-device-capabilities behavior that T0888 covers.
- Exception 0x02 (Illegal Data Address): register-map enumeration discovers the device's
  address layout — exactly the query-device-address-space behavior that T0888 covers.

Other exception codes (0x03, 0x04, 0x05, 0x06, etc.) and the Clear Counters 0x000A
anti-forensic path retain `mitre_techniques: vec![]` (no clean ICS ATT&CK mapping).

T0888 is already in the SS-14 emitted set (BC-2.14.020, BC-2.10.008 emitted-ID list,
SEEDED_TECHNIQUE_ID_COUNT and EMITTED_IDS unchanged — this is not a catalog expansion).

**Note for the record — holdout blemish-2 disposition:** Port-502 service label in
summary (src/decoder.rs:112) was assessed as CORRECT-BY-DESIGN (standard IANA port-service
hint, parallel to 443→HTTPS). Not a defect; no spec or code change.

**Artifacts changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| BC-2.14.019 | v1.2 → v1.3 | Postcondition Path A: mitre_techniques for exception code 0x01 → vec!["T0888"]; exception code 0x02 → vec!["T0888"]; other codes retain vec![]. Research note updated. Canonical test vectors for 0x01/0x02 cases updated to show ["T0888"]. Traceability MITRE field updated. Path B (Clear Counters) unchanged. |

**Impact:** MINOR. No BC semantics removed. Downstream stories targeting BC-2.14.019 Path A
with exception codes 0x01 or 0x02 must update acceptance criteria to expect
`mitre_techniques=["T0888"]` instead of `mitre_techniques=[]`. Clear Counters and other
exception code paths are unaffected.

**Emitted technique set:** Unchanged (T0888 was already emitted by BC-2.14.020). No change
to `SEEDED_TECHNIQUE_ID_COUNT`, `EMITTED_IDS`, or BC-2.10.008.

---

## [1.5] — 2026-06-09

### MINOR: F5 Combined-Delta Spec Defect Fixes — SS-14 Modbus v0.4.0

**Summary:** Four spec defects discovered during the F5 combined-delta review of Feature #7
(Modbus TCP analyzer). These defects existed in the SS-14 BC corpus and are corrected here
without changing any implementation behavior (the implementation is being authored in parallel
to align with this correction). All changes are MINOR (no BC semantics removed; existing
downstream story acceptance criteria remain valid with updated formulas).

**Defect 1 — Timestamp units: microseconds→seconds (BC-2.14.016, BC-2.14.017, BC-2.14.019)**

The f2-fix-directives §11.5 introduced a microsecond-scale assumption for window math
(`*1_000_000`, `elapsed_us`, `>= 2_000_000`). This was wrong. The pipeline's
`StreamHandler::on_data` delivers `timestamp_secs` (seconds, per BC-2.09.007); TLS/HTTP/
reassembler all confirm this via `DateTime::from_timestamp(ts, 0)`. All four Modbus window
computations have been corrected to seconds-scale math:

| Window | Old check (wrong) | New check (correct) |
|--------|------------------|---------------------|
| T0831 5s coordinated-write | `elapsed > T0831_WINDOW_SECS * 1_000_000` | `elapsed_secs > T0831_WINDOW_SECS` |
| Burst 1s write-rate | `elapsed > WRITE_BURST_WINDOW_SECS * 1_000_000` | `elapsed_secs > WRITE_BURST_WINDOW_SECS` |
| Sustained ≥2s write-rate | `elapsed_us >= 2_000_000 AND count*1_000_000 > threshold*elapsed_us` | `elapsed_secs >= WRITE_SUSTAINED_WINDOW_SECS AND count > threshold * elapsed_secs` |
| Exception 10s recon | `elapsed > 10_000_000` | `elapsed_secs > EXCEPTION_WINDOW_SECS` |

`wrapping_sub` is retained on all windows (u32 second timestamps wrap at ~136 years —
effectively never in practice, but the policy is kept for correctness).
Sub-second rate precision is a future enhancement requiring `timestamp_usecs` threading.

**Defect 2 — Non-existent FlowKey accessor: flow_key.client_ip() / flow_key.server_ip()
(BC-2.14.013, BC-2.14.017, BC-2.14.019)**

The `source_ip` postconditions in three BCs cited `flow_key.client_ip()` and
`flow_key.server_ip()`. These methods DO NOT EXIST on `FlowKey` (which has only
`lower_ip`, `upper_ip`, `lower_port`, `upper_port` plus a `Direction`). Corrected:
`source_ip` is now resolved from the `direction` arg passed to `on_data` combined with
`flow_key.lower_ip()`/`upper_ip()`. The mapping is:
- `Direction::ClientToServer` → initiator/client endpoint (write-class findings, BC-2.14.013; burst/sustained findings, BC-2.14.017; Clear Counters path, BC-2.14.019)
- `Direction::ServerToClient` → responder/server endpoint (exception-response findings, BC-2.14.019 Path A)

**Defect 3 — AnalysisSummary top-level field hallucination (BC-2.14.021)**

BC-2.14.021 postcondition 3 (v1.0) cited `findings_count`, `flows_analyzed`, and `protocol`
as top-level fields of `AnalysisSummary`. These fields DO NOT EXIST in the shared struct
(`src/analyzer/mod.rs`): the struct has exactly three fields — `analyzer_name: String`,
`packets_analyzed: u64`, `detail: BTreeMap<String, Value>`. Postcondition 3 has been
completely rewritten to match the real struct.

The SIX authoritative detail keys (post.1) are UNCHANGED and remain the authoritative
contract for the Modbus summary `detail` map:

| Key | Type | Semantics |
|-----|------|-----------|
| `"pdu_count"` | `Value::Number(u64)` | Total valid ADUs past the three-point gate |
| `"write_count"` | `Value::Number(u64)` | Total write-class FC PDUs |
| `"exception_count"` | `Value::Number(u64)` | Total exception-response PDUs (FC >= 0x80) |
| `"parse_errors"` | `Value::Number(u64)` | Total ADUs failing the validity gate |
| `"function_code_distribution"` | `Value::Object(map)` | FC → count map (hex-string keys, zero-count FCs omitted) |
| `"dropped_findings"` | `Value::Number(u64)` | Findings dropped due to MAX_FINDINGS cap (ALWAYS present, even 0) |

**Defect 4 — f2-fix-directives §11.5 / §11.5b microsecond-scale math superseded**

Added F5-correction banners to §11.5 and §11.5b in f2-fix-directives.md. The microsecond
math (`elapsed_us`, `*1_000_000`) is superseded by the seconds form. The corrected canonical
specification is BC-2.14.017 v2.2.

**Artifacts changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| BC-2.14.013 | v2.1 → v2.2 | source_ip: flow_key.client_ip() → Direction-resolved client endpoint |
| BC-2.14.016 | v2.0 → v2.1 | Window math: microseconds → seconds; edge cases EC-004/005/010 updated; test vectors updated to second-scale timestamps |
| BC-2.14.017 | v2.1 → v2.2 | Window math: microseconds → seconds (both burst and sustained); source_ip: flow_key.client_ip() → Direction-resolved; edge cases EC-004/004b/005/006/010 updated; test vectors updated; constants clarified as seconds |
| BC-2.14.019 | v1.1 → v1.2 | Window math: microseconds → seconds; source_ip Path A: flow_key.server_ip() → Direction-resolved server endpoint; source_ip Path B: flow_key.client_ip() → Direction-resolved; EC-009 updated |
| BC-2.14.021 | v1.0 → v1.1 | post.3 completely rewritten: remove non-existent flows_analyzed/findings_count/protocol fields; align to real AnalysisSummary struct (analyzer_name, packets_analyzed, detail only); six detail keys in post.1 unchanged and remain authoritative |
| f2-fix-directives.md | §11.5, §11.5b | F5-correction banners added; microsecond math identified as superseded; corrected form is seconds-scale per BC-2.14.017 v2.2 |

**Impact:** MINOR. No BC semantics removed; existing downstream story acceptance criteria
remain valid after updating formulas from microsecond to second scale. The implementation
(authored in parallel) is being aligned to seconds-scale math simultaneously with this
spec correction.

---

## [1.4] — 2026-06-09

### MINOR: BC-DISCREPANCY-001 — FC 0x17 Register-Write Set Reconciliation

**Summary:** Reconciled a discrepancy in the FC 0x17 (Read/Write Multiple Registers)
technique-tag mapping across BC-2.14.013, BC-2.14.014, and BC-2.14.015. Per orchestrator
ruling: FC 0x17 writes holding registers in its write phase and is therefore a
Modify-Parameter (T0836) operation. It participates in the T0831 register-write window
set {0x06, 0x10, 0x16, 0x17} and emits the union [T0855, T0836] per PDU. BC-2.14.016
already correctly included 0x17 in this set; the discrepancy was in the other three BCs.

**Root cause:** BC-2.14.013 EC-001 and Invariant 2 grouped FC 0x15 (Write File Record)
and FC 0x17 together as "File/multi writes → [T0855] only". This was stale/wrong for 0x17:
Write File Record targets file records (correctly T0855-only), but Read/Write Multiple
Registers writes holding registers (should carry T0836). BC-2.14.014 and BC-2.14.015
propagated the same error in their FC set definitions.

**Artifacts changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| BC-2.14.013 | v2.0 → v2.1 | EC-001 corrected: 0x17 → ["T0855","T0836"]; Postcondition 1 tag-union bullet updated; Invariant 2 split: {0x06,0x10,0x16,0x17} → T0836; {0x15} → T0855 only; test vector for 0x17 added |
| BC-2.14.014 | v2.0 → v2.1 | Title updated to include 0x17; Description FC set updated to {0x06,0x10,0x16,0x17}; Precondition 3 updated; Invariant 1 updated; Invariant 2 T0836 set updated; Invariant 4 corrected (0x17 is IN T0836 set, not T0855-only); test vector for 0x17 added |
| BC-2.14.015 | v2.0 → v2.1 | Precondition 3 corrected: 0x17 now referenced as holding-register write; Invariant 2 (0x17 entry) updated to T0855+T0836; Invariant 4 disjoint-set updated to include 0x17 in T0836 set; EC-004 and 0x17 test vector corrected |
| BC-2.14.016 | v2.0 (unchanged) | Already correct: FC set {0x06,0x10,0x16,0x17} used throughout; no changes needed |

**Consistency result after reconciliation:**

| Technique | FC set | Authoritative source |
|-----------|--------|---------------------|
| T0855 (Unauthorized Command Message) | {0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17} — all Write class | BC-2.14.013 |
| T0836 (Modify Parameter) | {0x06, 0x10, 0x16, 0x17} — holding-register writes | BC-2.14.014 v2.1 |
| T0835 (Manipulate I/O Image) | {0x05, 0x0F} — coil writes only | BC-2.14.015 |
| T0831 window set | {0x06, 0x10, 0x16, 0x17} — holding-register writes (same as T0836) | BC-2.14.016 |
| T0855-only Write FCs | {0x15} — Write File Record (file records, not registers/coils) | BC-2.14.013 |

T0836 set == T0831 window set == {0x06, 0x10, 0x16, 0x17}. No overlaps between T0836 and
T0835 sets. These three sets are now consistent across all four BCs.

**Impact:** MINOR (backward-compatible addition — extends 0x17's tag set from [T0855] to
[T0855, T0836]; no existing BC semantics removed). Downstream stories that test FC 0x17
behavior must be updated to expect ["T0855","T0836"] instead of ["T0855"] only.

---

## [1.3] — 2026-06-09

### ADDITIVE: F2 Schema Add-Ons + v0.3.0/v0.4.0 Release Split Tagging

**Summary:** Two research-backed schema add-ons from `f2-multitag-schema.md` applied to
existing BCs, plus release sequencing recorded across prd.md and prd-delta.md per human
decision (f2-bundle-vs-split.md B2 — Trivy/Zeek pattern).

**ADD-ON 1 — JSON report envelope fields (BC-2.11.001 v1.5):**

Two top-level JSON report envelope fields added (ONCE per report, NOT per-finding):
- `mitre_domain: "ics-attack"` — identifies the ATT&CK matrix; constant.
- `mitre_attack_version: "ics-attack-v15"` — placeholder; **FLAG for F4 to pin** against
  deployed catalog before v0.3.0 release tag.

Basis: ECS/OCSF recommendation to declare domain+version at envelope level rather than
redundantly per-technique (`T0xxx` prefix already unambiguously identifies ICS matrix).
CSV reporters carry no envelope fields (JSON-only).

**ADD-ON 2 — CSV empty-string clarification (BC-2.11.024 v1.5):**

Existing EC-001 strengthened + EC-015 added:
- When `mitre_techniques = vec![]`, the CSV cell is `""` (empty string) — NOT `"null"`,
  `"[]"`, `"N/A"`, or any sentinel.
- EC-015: Documents required consumer guard: `str.split(';')` on `""` produces `['']` in
  most languages; consumers MUST check `if cell.is_empty()` before splitting and return
  an empty collection, not `['']`.

**Release split tagging (v0.3.0/v0.4.0):**

Feature #7 is split into two releases:
- **v0.3.0** (schema migration; breaking): SS-09 + SS-10 + SS-11 BCs + ADD-ONs.
  Existing analyzers migrated; no new protocol analyzer.
  Compat: `--compat-mitre-scalar` flag for deprecation window.
- **v0.4.0** (Modbus; additive): all SS-14 BCs (BC-2.14.001..025).
  Built on stable v0.3.0 schema; no `**Breaking:**` in v0.4.0 changelog.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.11.001 | v1.4 → v1.5: envelope fields; H1 title updated; PC 7-8; Inv 4-6; EC-006, EC-007 | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.024 | v1.4 → v1.5: EC-001 strengthened; EC-015 added (consumer split guard); Inv 4 updated | `.factory/specs/behavioral-contracts/ss-11/` |
| prd.md | v1.2 → v1.3 note added; BREAKING box updated (envelope fields + CSV EC-015 ref); RELEASE SEQUENCING box added after BREAKING box; Section 2.14 release-target note added | `.factory/specs/prd.md` |
| prd-delta.md | new_prd_version 1.2 → 1.3; §5.3 ADD-ON details; §6 Release Sequencing; old §6 → §7 | `.factory/phase-f2-spec-evolution/prd-delta.md` |

**FLAG — mitre_attack_version not pinned:**
The value `"ics-attack-v15"` is a placeholder. F4 must verify the authoritative MITRE
ATT&CK for ICS version at attack.mitre.org/resources/attack-data-and-tools/ that covers
T0888, T0855, T0836, T0835, T0831, T0814, T0806, and update the constant in
`src/reporter/json.rs` before the v0.3.0 tag.

---

## [1.2] — 2026-06-09

### BREAKING: F2 Modbus Revision — Decisions 11-13 (ADR-006) — targets v0.3.0

**Summary:** Adopts three architect-approved decisions from `f2-fix-directives.md` v2.
Decision 13 is a breaking change to the `Finding` output schema targeting v0.3.0.
Revises 10 existing BCs (SS-09/SS-10/SS-11) + 8 SS-14 BCs already applied to BC body files.

**Adopted decisions:**

| Decision | Summary |
|----------|---------|
| D11 (supersedes D5) | Dual-window write-burst detection: `--modbus-write-burst-threshold` (default 20, 1s) + `--modbus-write-sustained-threshold` (default 10, >=2s). Old `--modbus-write-threshold` removed. |
| D12 (supersedes D8) | T0846 → T0888 correctness fix for recon FCs 0x11 and 0x2B/0x0E. T0888 = Remote System Information Discovery (TA0102 Discovery). T0846 remains seeded but is not emitted by Modbus. FC 0x07 excluded as standalone recon indicator. |
| D13 (supersedes D7) | Multi-tag Finding attribution: `Finding.mitre_technique: Option<String>` → `Finding.mitre_techniques: Vec<String>`. One finding per write PDU with ALL applicable technique tags. Volume control via burst aggregation, not tag-suppression. |

**BREAKING output schema changes (v0.3.0):**
- JSON: `"mitre_technique": "T0836"` → `"mitre_techniques": ["T0836"]` (key rename + type change)
- JSON: field absent when empty (same as prior `None` — `skip_serializing_if = "Vec::is_empty"`)
- JSON: multi-tag: `"mitre_techniques": ["T0855", "T0836"]`
- CSV: column-6 header renamed `mitre_technique` → `mitre_techniques`; multiple values semicolon-joined
- Rust: `Finding.mitre_technique: Option<String>` → `Finding.mitre_techniques: Vec<String>` (all emission sites + test helpers updated)

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| PRD | Version bump 1.1 → 1.2; Section 2 breaking-schema note added; Section 1.5, 2.10, 2.14 (D-H groups), 6.5, 8 updated | `.factory/specs/prd.md` |
| BC-INDEX | Version bump 1.1 → 1.2; SS-09/SS-10/SS-11 rows updated; SS-14 section header + BC-013/014/015/016/017/020/024 rows updated | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| prd-delta.md | Updated: new_prd_version 1.1→1.2; §5.2 added (10-BC revision table + 8 SS-14 BC revision table + affected-stories list) | `.factory/phase-f2-spec-evolution/prd-delta.md` |
| BC-2.09.001 | v1.4: `mitre_technique` field → `mitre_techniques` Vec | `.factory/specs/behavioral-contracts/ss-09/` |
| BC-2.09.006 | v1.5: `skip_serializing_if = "Vec::is_empty"`; multi-tag JSON output | `.factory/specs/behavioral-contracts/ss-09/` |
| BC-2.10.005 | v1.4: count 15 → 21 | `.factory/specs/behavioral-contracts/ss-10/` |
| BC-2.10.007 | v1.3: T0888 → Discovery row | `.factory/specs/behavioral-contracts/ss-10/` |
| BC-2.10.008 | v1.5: grep pattern + T0888 replaces T0846 in emitted list; 13 emitted | `.factory/specs/behavioral-contracts/ss-10/` |
| BC-2.11.013 | v1.6: multi-techniques tactic grouping by `[0]` | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.015 | v1.6: empty `mitre_techniques` vec → Uncategorized | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.017 | v1.5: multi-ID rendering `"MITRE: T0855, T0836"` | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.020 | v1.5: column-6 header rename | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.024 | v1.4: `mitre_techniques vec![]`; semicolon-join | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.14.013..017,020,022,024 | v2.0: co-emission model; T0888; dual-window (bodies already revised) | `.factory/specs/behavioral-contracts/ss-14/` |
| ADR-006 | Registered in ARCH-INDEX ADR table | `.factory/specs/architecture/ARCH-INDEX.md` (already present) |

**MITRE catalog size change:**

| Metric | v1.1 | v1.2 |
|--------|------|------|
| `SEEDED_TECHNIQUE_ID_COUNT` | 20 | **21** (T0888 added) |
| `EMITTED_IDS` count | 12 | **13** (T0888 replaces T0846 in ICS emitted set) |
| ICS SEEDED | 9 | **10** (T0888 added; T0846 already seeded) |
| ICS EMITTED | 6 | **7** {T0855, T0836, T0814, T0806, T0835, T0831, T0888} |
| T0846 status | emitted | **seeded-not-emitted** |

**Affected stories (story-writer must propagate BC table + AC changes):**
STORY-069, STORY-070, STORY-071, STORY-078, STORY-079, STORY-080.

**ADR reference:** ADR-006 — Multi-Technique Finding Attribution
(`.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md`)

---

## [1.1] — 2026-06-09

### MINOR: SS-14 Modbus/ICS Analyzer — Feature #7

**Summary:** Added Modbus TCP protocol analyzer (SS-14, C-22) with 25 behavioral contracts,
VP-022 formal verification target, ADR-005 architecture decision, and 6 MITRE ATT&CK for
ICS technique mappings.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| PRD | Version bump 1.0 → 1.1; Section 2.14 added (25 BCs); Section 7 RTM extended (25 rows); KD-003 and KD-005 sections updated | `.factory/specs/prd.md` |
| BC-INDEX | Version bump 1.0 → 1.1; SS-14 subsystem section added (25 rows); total BC count 219 → 244 | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| BC-2.14.001..022 | Created (F2 create burst, Groups A-G) | `.factory/specs/behavioral-contracts/ss-14/` |
| BC-2.14.023 | Created (Group H: --modbus CLI flag enablement) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.023.md` |
| BC-2.14.024 | Created (Group H: --modbus-write-threshold CLI flag) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.024.md` |
| BC-2.14.025 | Created (Group H: StreamDispatcher port-502 Rule 5 classification) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.025.md` |
| Architecture Delta | Created | `.factory/phase-f2-spec-evolution/architecture-delta.md` |
| PRD Delta | Created | `.factory/phase-f2-spec-evolution/prd-delta.md` |
| VP-022 | Designed (to be authored by formal-verifier in parallel) | `.factory/specs/verification-properties/VP-022.md` (pending) |
| ADR-005 | Created (binary ICS protocol integration decision) | `.factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` |

**New MITRE ATT&CK for ICS techniques (6 total):**
- T0855 — Unauthorized Command Message (IcsImpairProcessControl)
- T0836 — Modify Parameter (IcsImpairProcessControl)
- T0814 — Denial of Service (IcsInhibitResponseFunction)
- T0806 — Brute Force I/O (IcsImpairProcessControl)
- T0835 — Manipulate I/O Image (IcsImpairProcessControl)
- T0831 — Manipulation of Control (IcsImpairProcessControl)

**MITRE catalog size:** 15 → 20 seeded technique IDs
(`SEEDED_TECHNIQUE_ID_COUNT = 15 → 20`; `EMITTED_IDS` extended from 6 to 12).

**Key constants introduced:**
- `MAX_PENDING_TRANSACTIONS = 256` (per-flow pending table cap)
- `WRITE_RATE_WINDOW_SECS = 1` (burst detection window)
- `DEFAULT_MODBUS_WRITE_THRESHOLD = 10` (writes/second before T0806 fires)

**CLI surface changes:**
- `--modbus` flag added to `analyze` subcommand (boolean, default false)
- `--modbus-write-threshold N` flag added (u32, default 10; zero rejected)
- `--all` expansion updated to include `--modbus`
- `needs_reassembly` expression updated: `enable_http || enable_tls || enable_modbus`

**Dispatcher changes:**
- `DispatchTarget::Modbus` variant added (4th variant after Http, Tls, None)
- `StreamDispatcher.modbus: Option<ModbusAnalyzer>` field added
- `classify` Rule 5: port 502 → `DispatchTarget::Modbus` (after content rules 1-2 and TLS/HTTP port rules 3-4)
- `modbus_analyzer()` and `take_modbus_analyzer()` accessors added
- `on_data` and `on_flow_close` Modbus routing arms added
- VP-004 `classify_oracle` must be extended with Rule 5

**Spec debt resolved:**
- O-04 partially resolved: T0855 (previously catalogued-but-never-emitted) is now actively
  emitted by ModbusAnalyzer. Updated in PRD Section 1.5 Out of Scope note.

---

## [1.0] — 2026-05-20

### Initial specification (brownfield ingestion)

Initial PRD and BC set produced by brownfield ingestion of develop HEAD. 219 active BCs
across ss-01 through ss-13 (BC-2.01.001..BC-2.13.004). Includes: 218 ingestion-batch BCs,
6 retired (BC-ABS-004..009), 5 pass-4 additions (BC-2.11.020..024), 2 F2 pcap-timestamp
additions (BC-2.04.055, BC-2.09.007).
