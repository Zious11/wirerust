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
