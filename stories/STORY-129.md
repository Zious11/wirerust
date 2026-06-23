---
document_type: story
story_id: STORY-129
epic_id: E-8
version: "1.0"
status: draft
# BC status: BC authored and anchored below; all traces complete.
producer: story-writer
timestamp: 2026-06-22T00:00:00Z
phase: f3
points: 5
priority: P0
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.11.035
verification_properties: []
tdd_mode: strict
target_module: reporter/json
subsystems: [SS-11]
estimated_days: 2
feature_id: issue-064-mitre-attack-json-enrichment
github_issue: 64
wave: 57
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.035.md
  - .factory/specs/prd.md
  - .factory/specs/prd-supplements/interface-definitions.md
# Dependency anchor: STORY-129 has no predecessors. src/mitre.rs (the MITRE
#   catalog) and src/findings.rs (the Finding struct) are stable as of Wave 56.
#   The JsonReporter in src/reporter/json.rs is stable as of Wave 31 (STORY-101).
#   No in-progress story modifies these files. STORY-129 is self-contained: it
#   adds FindingJsonDto in a new file (src/reporter/json_dto.rs) and surgically
#   modifies json.rs to use it. No cycle is introduced.
# Subsystem anchor: SS-11 owns this story's scope because the per-finding
#   mitre_attack enrichment lives entirely in the reporter layer (src/reporter/).
#   The catalog extension (technique_tactic_id in src/mitre.rs) is also owned
#   by SS-11 per BC-2.11.035 Architecture Anchors — the mitre module is part of
#   the reporter pipeline supporting data. ARCH-INDEX designates SS-11 as the
#   Reporting and Output subsystem per CAP-11.
input-hash: "93eba63"
---

# STORY-129: Emit Per-Finding `mitre_attack` Array in JSON Output

## Narrative

- **As a** SIEM integrator or SOC analyst consuming wirerust JSON output
- **I want** each finding object in the JSON report to include a `mitre_attack`
  array with one fully-resolved technique object per technique ID (carrying
  `id`, `name`, `tactic_id`, `tactic_name`, and `reference` fields), while
  unknown IDs produce partial objects (id+reference only) and empty
  `mitre_techniques` vecs produce no `mitre_attack` key at all
- **So that** downstream tooling can correlate findings with the MITRE ATT&CK
  knowledge base without performing secondary catalog lookups, while the raw
  `mitre_techniques` array and all other fields remain unchanged (additive,
  non-breaking)

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.11.035 | Per-Finding `mitre_attack` Array Enriches JSON Output with Resolved Technique Objects; Order-Preserving; Unknown IDs Emit Partial Objects; Empty Vec Omits Field |

## Acceptance Criteria

### AC-1 (traces to BC-2.11.035 postcondition 1, postcondition 3a–3e, and EC-002)
**Known single technique produces fully-resolved 5-field object.**
When a `Finding` has `mitre_techniques: vec!["T1046".to_string()]`, the serialized
JSON object for that finding MUST contain:
- `"mitre_techniques": ["T1046"]` (unchanged per postcondition 5)
- `"mitre_attack": [{"id": "T1046", "name": "Network Service Discovery",
  "tactic_id": "TA0007", "tactic_name": "Discovery",
  "reference": "https://attack.mitre.org/techniques/T1046/"}]`

All five fields (`id`, `name`, `tactic_id`, `tactic_name`, `reference`) MUST be present.

**Test:** `test_BC_2_11_035_known_technique_all_five_fields`

### AC-2 (traces to BC-2.11.035 postcondition 4, invariant 1, and EC-001)
**Unknown technique ID: `id` is never lost; partial object emitted.**
When a `Finding` has `mitre_techniques: vec!["T9999".to_string()]`, the serialized
JSON MUST contain `"mitre_attack": [{"id": "T9999",
"reference": "https://attack.mitre.org/techniques/T9999/"}]`.
The keys `name`, `tactic_id`, and `tactic_name` MUST be absent from the element
(`skip_serializing_if` semantics). The element MUST NOT be an empty object and
MUST NOT be missing.

**Test:** `test_BC_2_11_035_unknown_technique_id_never_lost`

### AC-3 (traces to BC-2.11.035 postcondition 4 and EC-001)
**Empty `mitre_techniques` vec omits `mitre_attack` key entirely.**
When a `Finding` has `mitre_techniques: vec![]`, the serialized JSON object for
that finding MUST lack both the `"mitre_techniques"` key and the `"mitre_attack"`
key (`skip_serializing_if = Vec::is_empty` semantics for both).

**Test:** `test_BC_2_11_035_empty_mitre_techniques_omits_mitre_attack`

### AC-4 (traces to BC-2.11.035 postcondition 2, invariant 2, and EC-006)
**Multi-tag: `mitre_attack` array order matches `mitre_techniques` order exactly.**
When a `Finding` has `mitre_techniques: vec!["T1692.001".to_string(), "T0836".to_string()]`,
the `mitre_attack` array MUST have exactly 2 elements, in declaration order:
- Index 0: `{"id": "T1692.001", "name": "Unauthorized Message: Command Message",
  "tactic_id": "TA0106", "tactic_name": "Impair Process Control",
  "reference": "https://attack.mitre.org/techniques/T1692.001/"}`
- Index 1: `{"id": "T0836", "name": "Modify Parameter",
  "tactic_id": "TA0106", "tactic_name": "Impair Process Control",
  "reference": "https://attack.mitre.org/techniques/T0836/"}`

**Test:** `test_BC_2_11_035_multitag_order_preserved`

### AC-5 (traces to BC-2.11.035 invariant 3 and EC-007)
**Duplicate technique IDs produce duplicate (non-deduplicated) elements.**
When a `Finding` has `mitre_techniques: vec!["T1046".to_string(), "T9999".to_string(),
"T1046".to_string()]`, the `mitre_attack` array MUST have exactly 3 elements:
- Index 0: fully resolved T1046 object (5 fields)
- Index 1: partial T9999 object (id+reference only)
- Index 2: fully resolved T1046 object (5 fields, identical to index 0)

The implementer MUST NOT deduplicate. `mitre_attack.len() == mitre_techniques.len()`.

**Test:** `test_BC_2_11_035_duplicate_ids_not_deduplicated`

### AC-6 (traces to BC-2.11.035 postcondition 3e, invariant 4, and EC-005)
**Sub-technique dot separator preserved verbatim in `id` and `reference` URL.**
When a `Finding` has `mitre_techniques: vec!["T1071.001".to_string()]`, the element
MUST be `{"id": "T1071.001", "name": "Web Protocols", "tactic_id": "TA0011",
"tactic_name": "Command and Control",
"reference": "https://attack.mitre.org/techniques/T1071.001/"}`.
The dot in `T1071.001` is preserved verbatim in both `id` and the URL.

**Test:** `test_BC_2_11_035_sub_technique_dot_preserved`

### AC-7 (traces to BC-2.11.035 Catalog Extension and EC-003)
**ICS technique resolves `tactic_id` to ICS-matrix TA-prefix ID.**
When a `Finding` has `mitre_techniques: vec!["T0827".to_string()]` (IcsImpact tactic),
the element MUST be `{"id": "T0827", "name": "Loss of Control",
"tactic_id": "TA0105", "tactic_name": "Impact (ICS)",
"reference": "https://attack.mitre.org/techniques/T0827/"}`.
`tactic_id` MUST be `"TA0105"` (not `"TA0040"` which is the Enterprise Impact tactic).

**Test:** `test_BC_2_11_035_ics_tactic_id_resolved`

### AC-8 (traces to BC-2.11.035 postcondition 5, invariant 5, and EC-002)
**`mitre_techniques` array is unchanged when `mitre_attack` is also present.**
When a `Finding` has `mitre_techniques: vec!["T1046".to_string()]`, the JSON output
MUST contain BOTH `"mitre_techniques": ["T1046"]` (raw ID string array, unchanged)
AND `"mitre_attack": [...]` (enriched array). No existing key is renamed, removed,
or reordered. BC-2.11.035 is purely additive.

**Test:** `test_BC_2_11_035_mitre_techniques_unchanged`

### AC-9 (traces to BC-2.11.035 postcondition 6)
**`mitre_attack` is absent from CSV output (CsvReporter unaffected).**
Running the CsvReporter on a `Finding` with `mitre_techniques: vec!["T1046".to_string()]`
MUST produce CSV output that does NOT contain a `mitre_attack` column or any nested
JSON object. The CsvReporter is unmodified by this story.

**Test:** `test_BC_2_11_035_csv_unaffected`

### AC-10 (traces to BC-2.11.035 postcondition 7)
**`mitre_attack` is absent from terminal output (TerminalReporter unaffected).**
Running the TerminalReporter on a `Finding` with `mitre_techniques: vec!["T1046".to_string()]`
MUST produce terminal output that does NOT contain the string `"mitre_attack"`.
The TerminalReporter is unmodified by this story.

**Test:** `test_BC_2_11_035_terminal_unaffected`

## Behavioral Contracts Table

| BC | Version | Clauses Covered |
|----|---------|-----------------|
| BC-2.11.035 | v1.0 | All 10 ACs (postconditions 1–7; invariants 1–6; EC-001 through EC-010; Catalog Extension; Architecture Anchors). This is the sole BC for this story. |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `FindingJsonDto<'a>` wrapper struct (new) | `src/reporter/json_dto.rs` | Pure (serde Serialize; zero I/O; `From<&'a Finding>` maps via catalog lookup) |
| `MitreAttackEntry` struct (new) | `src/reporter/json_dto.rs` | Pure (serde Serialize; fields: `id: String`, `name: Option<&'static str>`, `tactic_id: Option<&'static str>`, `tactic_name: Option<String>`, `reference: String`) |
| `technique_tactic_id(id: &str) -> Option<&'static str>` (new accessor) | `src/mitre.rs` | Pure (static match on `MitreTactic`; no I/O; deterministic) |
| `JsonReporter::render` modification | `src/reporter/json.rs` | Effectful shell (writes JSON to output) — only the call-site changes from raw slice to `Vec<FindingJsonDto>` |
| Integration tests (13 tests: 10 AC + 3 edge-case: EC-008 mixed-batch, EC-009, EC-010) — all 13 present and GREEN at convergence | `tests/reporter_json_tests.rs` | Test harness |

Architecture section references: `architecture/module-decomposition.md` (SS-11 reporter
module; `src/reporter/` subtree); ADR-0003 (serde delegation pattern — additive field via
wrapper type is consistent); BC-2.11.035 Architecture Anchors.

## Forbidden Dependencies

- STORY-129 MUST NOT modify `src/reporter/csv.rs` or `src/reporter/terminal.rs`. AC-9
  and AC-10 verify these are unaffected; the files must not be touched.
- STORY-129 MUST NOT modify `src/findings.rs`. The `Finding` struct is unchanged;
  `FindingJsonDto` wraps it without mutation.
- STORY-129 MUST NOT rename or remove any existing JSON key. BC-2.11.035 is purely additive.
- STORY-129 MUST NOT add a new `MitreTactic` variant without also updating the
  `technique_tactic_id` match arm (BC-2.11.035 invariant 6).
- STORY-129 MUST NOT use BC-TBD placeholder IDs in any AC trace.
- STORY-129 MUST NOT add any new crate dependency to `Cargo.toml`.
- STORY-129 MUST NOT add a `version` field to individual `mitre_attack` elements. The
  catalog version is communicated by the envelope `mitre_attack_version` field
  (BC-2.11.001); no per-element version is emitted.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `mitre_techniques = []` (empty vec) | `"mitre_attack"` key absent; `"mitre_techniques"` key also absent (both `skip_serializing_if = Vec::is_empty`) |
| EC-002 | `mitre_techniques = ["T1046"]` — known Enterprise technique | Fully resolved 5-field object: id, name, tactic_id=TA0007, tactic_name=Discovery, reference |
| EC-003 | `mitre_techniques = ["T0827"]` — known ICS technique (IcsImpact) | tactic_id=TA0105, tactic_name=Impact (ICS) — ICS-matrix TA-id, not Enterprise TA0040 |
| EC-004 | `mitre_techniques = ["T9999"]` — unknown/unmapped ID | Partial object: id + reference only; name/tactic_id/tactic_name absent |
| EC-005 | `mitre_techniques = ["T1071.001"]` — sub-technique with dot separator | Dot preserved verbatim in id and reference URL; all 5 fields present |
| EC-006 | `mitre_techniques = ["T1692.001", "T0836"]` — two known ICS techniques | Two elements in declaration order; both IcsImpairProcessControl / TA0106 |
| EC-007 | `mitre_techniques = ["T1046", "T9999", "T1046"]` — known + unknown + duplicate | Three elements; index 0 full, index 1 partial, index 2 full (duplicate); no deduplication |
| EC-008 | Mixed batch: one finding with mitre_techniques, one without | `mitre_attack` absent only on the empty-vec finding; other findings unaffected |
| EC-009 | `mitre_techniques = ["T1557.002"]` — Enterprise sub-technique (CredentialAccess) | tactic_id=TA0006, tactic_name=Credential Access; dot preserved |
| EC-010 | `mitre_techniques = ["T0830"]` — ICS Collection | tactic_id=TA0100, tactic_name=Collection (ICS) |

## Tasks

1. **Extend `src/mitre.rs`:** Add `pub fn technique_tactic_id(id: &str) -> Option<&'static str>`.
   - Call `technique_info(id)` to get the `MitreTactic` variant; if `None`, return `None`.
   - Match each `MitreTactic` variant to its canonical TA-prefix ID string per the table
     in BC-2.11.035 Catalog Extension (17 variants: Reconnaissance→TA0043,
     ResourceDevelopment→TA0042, InitialAccess→TA0001, Execution→TA0002,
     Persistence→TA0003, PrivilegeEscalation→TA0004, DefenseEvasion→TA0005,
     CredentialAccess→TA0006, Discovery→TA0007, LateralMovement→TA0008,
     Collection→TA0009, CommandAndControl→TA0011, Exfiltration→TA0010,
     Impact→TA0040, IcsInhibitResponseFunction→TA0107, IcsImpairProcessControl→TA0106,
     IcsImpact→TA0105).
   - Extend the existing `vp007_catalog_drift_guard` test to cross-check this mapping
     doesn't drift from the `MitreTactic` enum.

2. **Create `src/reporter/json_dto.rs`:**
   - Define `MitreAttackEntry` with fields:
     - `id: String` (always serialized)
     - `#[serde(skip_serializing_if = "Option::is_none")] name: Option<&'static str>`
     - `#[serde(skip_serializing_if = "Option::is_none")] tactic_id: Option<&'static str>`
     - `#[serde(skip_serializing_if = "Option::is_none")] tactic_name: Option<String>`
     - `reference: String` (always serialized; synthesized as `format!("https://attack.mitre.org/techniques/{}/", id)`)
   - Define `FindingJsonDto<'a>`:
     - `#[serde(flatten)] inner: &'a Finding`
     - `#[serde(skip_serializing_if = "Vec::is_empty")] mitre_attack: Vec<MitreAttackEntry>`
   - Implement `From<&'a Finding> for FindingJsonDto<'a>`:
     - Map each id in `finding.mitre_techniques` to a `MitreAttackEntry` via `technique_info` and `technique_tactic_id`.
     - `reference = format!("https://attack.mitre.org/techniques/{}/", id)`.
   - Add `mod json_dto;` to `src/reporter/mod.rs` (or `src/reporter/json.rs`, match codebase convention).

3. **Modify `src/reporter/json.rs`:**
   - Replace the raw `&[Finding]` slice serialization in `JsonReporter::render` with
     `findings.iter().map(FindingJsonDto::from).collect::<Vec<_>>()`.
   - Import `FindingJsonDto` from `super::json_dto` (or `crate::reporter::json_dto`).
   - No other changes to `json.rs` — the `#[serde(flatten)]` on `inner` preserves all
     existing finding keys; `mitre_attack` is additive.

4. **Add tests in `tests/reporter_json_tests.rs`** (13 total: one per AC plus 3 dedicated
   edge-case tests, using EXACT test names from BC-2.11.035 Verification Properties table
   for DF-AC-TEST-NAME-SYNC-001 compliance):
   - `test_BC_2_11_035_known_technique_all_five_fields` (AC-1)
   - `test_BC_2_11_035_unknown_technique_id_never_lost` (AC-2)
   - `test_BC_2_11_035_empty_mitre_techniques_omits_mitre_attack` (AC-3)
   - `test_BC_2_11_035_multitag_order_preserved` (AC-4)
   - `test_BC_2_11_035_duplicate_ids_not_deduplicated` (AC-5)
   - `test_BC_2_11_035_sub_technique_dot_preserved` (AC-6)
   - `test_BC_2_11_035_ics_tactic_id_resolved` (AC-7)
   - `test_BC_2_11_035_mitre_techniques_unchanged` (AC-8)
   - `test_BC_2_11_035_csv_unaffected` (AC-9)
   - `test_BC_2_11_035_terminal_unaffected` (AC-10)
   - `test_BC_2_11_035_mixed_batch_per_finding_independence` (EC-008)
   - `test_BC_2_11_035_ec009_enterprise_subtechnique` (EC-009)
   - `test_BC_2_11_035_ec010_ics_collection` (EC-010)
   Each test: construct a `Finding` directly, call the appropriate reporter, parse or
   inspect the output, assert the structural contract.

5. Run `cargo test --all-targets` (verify all prior tests remain green).
6. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-1 | `test_BC_2_11_035_known_technique_all_five_fields` | Unit |
| AC-2 | `test_BC_2_11_035_unknown_technique_id_never_lost` | Unit |
| AC-3 | `test_BC_2_11_035_empty_mitre_techniques_omits_mitre_attack` | Unit |
| AC-4 | `test_BC_2_11_035_multitag_order_preserved` | Unit |
| AC-5 | `test_BC_2_11_035_duplicate_ids_not_deduplicated` | Unit |
| AC-6 | `test_BC_2_11_035_sub_technique_dot_preserved` | Unit |
| AC-7 | `test_BC_2_11_035_ics_tactic_id_resolved` | Unit |
| AC-8 | `test_BC_2_11_035_mitre_techniques_unchanged` | Unit |
| AC-9 | `test_BC_2_11_035_csv_unaffected` | Unit |
| AC-10 | `test_BC_2_11_035_terminal_unaffected` | Unit |
| EC-008 | `test_BC_2_11_035_mixed_batch_per_finding_independence` | Unit |
| EC-009 | `test_BC_2_11_035_ec009_enterprise_subtechnique` | Unit |
| EC-010 | `test_BC_2_11_035_ec010_ics_collection` | Unit |

## Previous Story Intelligence

- STORY-101 (Wave 31) established the reporter multi-tag serialization pattern and
  introduced the `mitre_techniques: Vec<String>` field. STORY-129 builds on that
  foundation by adding the `mitre_attack` enrichment as a serde wrapper DTO. The
  `#[serde(flatten)]` + `skip_serializing_if` pattern mirrors the existing approach
  in STORY-101/BC-2.11.001.
- STORY-071 (Wave 3) owns `technique_info(id) -> Option<(&'static str, MitreTactic)>`
  in `src/mitre.rs` and established the `MitreTactic` enum with `Display` impl.
  STORY-129 adds only `technique_tactic_id` — a new accessor using the same lookup
  function. The `vp007_catalog_drift_guard` test from STORY-071 must be extended
  to cover the new TA-ID mapping.
- STORY-076 (Wave 20) established `JsonReporter` structure. The change required by
  STORY-129 is minimal: replace the raw findings slice serialization with a
  `Vec<FindingJsonDto>`. The reporter trait contract, envelope structure, and all
  other behavior are unchanged.
- `depends_on: []` — both `src/mitre.rs` and `src/reporter/json.rs` are stable
  (Wave 56 is complete; all E-19 pcapng stories are DELIVERED & CLOSED). No
  predecessor story is needed.

## Architecture Compliance Rules

Derived from BC-2.11.035, ADR-0003 (serde delegation pattern), and ADR-006 Decision 13
(mitre_techniques schema):

1. **Additive only** — no existing JSON key is renamed, removed, or reordered.
   The `#[serde(flatten)]` on `FindingJsonDto.inner` ensures this by construction.
2. **DTO wrapper pattern** — `FindingJsonDto` is a zero-copy view (`&'a Finding`)
   decorated with the derived field; never clone or copy the `Finding`.
3. **`skip_serializing_if` symmetry** — `mitre_attack` uses `Vec::is_empty`
   (same predicate as `mitre_techniques`). Optional sub-fields use `Option::is_none`.
4. **`reference` is always synthesized** — never sourced from the catalog. URL is
   `format!("https://attack.mitre.org/techniques/{}/", id)` for ALL IDs.
5. **Catalog extension is atomic** — `technique_tactic_id` MUST land in the same
   commit as `FindingJsonDto`; the `vp007_catalog_drift_guard` extension is part of
   the same story (BC-2.11.035 invariant 6).
6. **CSV and terminal reporters are untouched** — any modification to
   `src/reporter/csv.rs` or `src/reporter/terminal.rs` is a regression.
7. **No new crate deps** — `serde`, `serde_json` are already in `Cargo.toml`.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `serde` | existing (1.x) | `Serialize` derive; `skip_serializing_if`; `flatten` |
| `serde_json` | existing (1.x) | JSON serialization; `json!` macro in tests for assertion |

No new dependencies. All libraries already present in `Cargo.toml`.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reporter/json_dto.rs` | **CREATE** | `MitreAttackEntry` + `FindingJsonDto<'a>` + `From<&'a Finding>` impl |
| `src/reporter/json.rs` | **MODIFY** | Use `Vec<FindingJsonDto>` in `render`; import `json_dto`; minimal delta |
| `src/mitre.rs` | **EXTEND** | Add `pub fn technique_tactic_id(id: &str) -> Option<&'static str>`; extend `vp007_catalog_drift_guard` test |
| `src/reporter/mod.rs` | **MODIFY** | Add `pub mod json_dto;` (or `mod json_dto;` per codebase convention) |
| `tests/reporter_json_tests.rs` | **MODIFY** | Add 13 unit tests (AC-1 through AC-10 + EC-008, EC-009, EC-010) |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~5,000 |
| BC files (1 BC: BC-2.11.035 v1.0) | ~4,000 |
| `src/mitre.rs` (catalog + existing `technique_info` + `vp007_catalog_drift_guard`) | ~4,000 |
| `src/reporter/json.rs` (existing render path, envelope, imports) | ~3,000 |
| `src/findings.rs` (Finding struct shape for flatten correctness) | ~2,000 |
| `src/reporter/json_dto.rs` (new file — small, created by implementer) | ~500 |
| `tests/reporter_json_tests.rs` (existing tests + 10 new AC tests) | ~4,000 |
| Tool outputs (cargo test, clippy) | ~1,000 |
| **Total estimated** | **~23,500** |

Well within 20-30% of agent context window (~200k token window = 40-60k budget limit).

## Dependency Rationale

- `depends_on: []` — STORY-129 requires only stable, already-merged modules:
  `src/mitre.rs` (from STORY-071, Wave 3), `src/reporter/json.rs` (from STORY-076,
  Wave 20, extended by STORY-101, Wave 31), and `src/findings.rs` (from STORY-069,
  Wave 1). All are DELIVERED & CLOSED. No predecessor story is needed.
- `blocks: []` — no downstream story currently depends on the `mitre_attack` JSON
  field. Future stories that consume or extend this enrichment will declare
  `depends_on: [STORY-129]` when authored.
- Wave 57 is assigned (max(56)+1 = 57). STORY-128 occupies Wave 56; STORY-129 has
  no predecessor constraint but is assigned Wave 57 to maintain a clean topological
  ordering in the wave schedule. It can be parallelized with any other Wave 57 story.
- **Cycle check (Kahn's algorithm):** Adding STORY-129 with `depends_on: []` adds
  one new vertex and zero new edges to the dependency graph. No cycle is possible
  with zero incoming edges. The existing 56-wave DAG remains acyclic.
