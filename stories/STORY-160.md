---
document_type: story
story_id: STORY-160
epic_id: E-8
version: "1.5"
status: draft
producer: story-writer
timestamp: 2026-07-08T00:00:00Z
phase: f7
level: feature
cycle: triage-2026-07-08
points: 3
priority: P2
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.11.036
  - BC-2.11.037
  - BC-2.11.001
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: src/findings.rs + src/reporter/json.rs
subsystems:
  - SS-11
estimated_days: 1
wave: "72"
traces_to:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.036.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.037.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.001.md
input-hash: "7e22ccb"
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.036.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.037.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.001.md
---

# STORY-160: Align JSON Finding-Enum Serialization to Lowercase/snake_case + schema_version Envelope

**Epic:** E-8 (Reporting and Output Formats)
**Status:** draft
**Wave:** 72
**Points:** 3
**Priority:** P2

## Narrative

- **As a** consumer of wirerust JSON output (SIEM pipeline, downstream parser, or dashboard)
- **I want** finding enum fields (`verdict`, `confidence`, `category`) to use idiomatic
  lowercase/snake_case values and a `schema_version` field to signal format generation
- **So that** I can integrate wirerust output with tools following Suricata EVE / ECS / OCSF
  conventions without custom case-conversion shims, and reliably gate on schema format

## Behavioral Contracts

- **BC-2.11.036** — JSON enum values use `lowercase` (Verdict/Confidence) and `snake_case`
  (ThreatCategory); terminal Display tokens UNCHANGED. Anchors the three `serde(rename_all)`
  attributes on `src/findings.rs` enums and all 17 variant-level assertions.
- **BC-2.11.037** — JSON report envelope includes `schema_version: "2"`; value is the constant
  string `"2"`; emitted unconditionally; absent from CSV and terminal output.
- **BC-2.11.001 v1.8** — JSON envelope shape; advisory pointer to schema_version addition.

## Background

GitHub issue #255 (`snake_case JSON enums`) was validated and triaged on 2026-07-08
(triage record `triage-2026-07-08`, research verdicts 10/10 CONFIRMED).

The research-validated scope (triage record entry #255):
- `rename_all = "lowercase"` for `Verdict` and `Confidence` (single-word enums).
- `rename_all = "snake_case"` for `ThreatCategory` (multi-word enum) per Suricata EVE / ECS /
  OCSF conventions.
- `schema_version` envelope field added in the same PR to future-proof the next breaking change.
- Hard cutover at v0.12.0. No dual-output mode, no opt-in flag, no deprecation period.
- JSON schema is a governed surface **outside `cargo-semver-checks` scope** — must be
  documented in the CHANGELOG and announced via `schema_version`.

The product owner authored BC-2.11.036 and BC-2.11.037 (commit 00d67b1, BC-INDEX v2.21,
2026-07-08) after the triage. BC-2.11.001 was simultaneously amended to v1.8 with an advisory
pointer to the schema_version addition.

### Representative variant assertions (BC-2.11.036)

| Enum | Variant | Pre-v0.12.0 JSON | v0.12.0+ JSON |
|------|---------|-----------------|--------------|
| `Verdict` | `Likely` | `"Likely"` | `"likely"` |
| `Verdict` | `Inconclusive` | `"Inconclusive"` | `"inconclusive"` |
| `Confidence` | `High` | `"High"` | `"high"` |
| `Confidence` | `Low` | `"Low"` | `"low"` |
| `ThreatCategory` | `LateralMovement` | `"LateralMovement"` | `"lateral_movement"` |
| `ThreatCategory` | `C2` | `"C2"` | `"c2"` |
| `ThreatCategory` | `CredentialAccess` | `"CredentialAccess"` | `"credential_access"` |

### Current state

`src/findings.rs` defines `Verdict`, `Confidence`, and `ThreatCategory` with `#[derive(Serialize)]`
but no `rename_all` attribute. Serialized values are therefore the Rust PascalCase variant names.
`src/reporter/json.rs` has no `schema_version` field in the JSON envelope.

## Acceptance Criteria

### AC-160-001 (Verdict rename_all lowercase)

`Verdict` in `src/findings.rs` carries `#[serde(rename_all = "lowercase")]`. All four variants
serialize to their lowercase form in JSON output:

```
test_BC_2_11_036_verdict_likely_serializes_lowercase — pass
test_BC_2_11_036_verdict_all_variants_lowercase      — pass
```

The `test_BC_2_11_036_verdict_all_variants_lowercase` test asserts that a JSON array of all
four `Verdict` variants contains `["likely", "unlikely", "inconclusive", "possible"]` in some
order, with zero PascalCase occurrences (no `"Likely"`, no `"Unlikely"`, etc.).

### AC-160-002 (Confidence rename_all lowercase)

`Confidence` in `src/findings.rs` carries `#[serde(rename_all = "lowercase")]`. All three
variants serialize to their lowercase form:

```
test_BC_2_11_036_confidence_high_serializes_lowercase  — pass
test_BC_2_11_036_confidence_all_variants_lowercase     — pass
```

### AC-160-003 (ThreatCategory rename_all snake_case — including lateral_movement and c2)

`ThreatCategory` in `src/findings.rs` carries `#[serde(rename_all = "snake_case")]`. All ten
variants serialize to their snake_case form. Representative AC assertions:

```
test_BC_2_11_036_threat_category_lateral_movement_snake_case — pass
    Asserts: ThreatCategory::LateralMovement → "lateral_movement"
test_BC_2_11_036_threat_category_c2_snake_case               — pass
    Asserts: ThreatCategory::C2 → "c2" (no underscore; serde lowercases single letter,
    treats digit as non-alpha continuation, producing "c2")
test_BC_2_11_036_threat_category_all_variants_snake_case     — pass
    Asserts: all 10 variants present with their snake_case forms; no PascalCase occurrence
```

### AC-160-004 (schema_version present in every JSON report)

`src/reporter/json.rs` defines `const SCHEMA_VERSION: &str = "2";` (analogous to the existing
`MITRE_DOMAIN` and `MITRE_ATTACK_VERSION` constants). Every `JsonReporter::render` call includes
`"schema_version": "2"` at the top level of the JSON envelope:

```
test_BC_2_11_037_schema_version_present_in_json           — pass
    Asserts: JSON output contains "schema_version" key
test_BC_2_11_037_schema_version_value_is_two              — pass
    Asserts: value is the JSON string "2", not the integer 2, not null
test_BC_2_11_037_schema_version_unconditional_empty_findings — pass
    Asserts: field present even when findings slice is empty
```

### AC-160-005 (Terminal Display regression — uppercase tokens unchanged)

The `fmt::Display` implementations for `Verdict` and `Confidence` are NOT modified. The serde
`rename_all` attribute on the derive affects only `Serialize`, not `Display`.

```
test_BC_2_11_036_terminal_display_unchanged — pass
    Asserts: Verdict::Likely → "LIKELY"; Confidence::High → "HIGH"
    (Display tokens are PascalCase-derived uppercase, not serde-controlled)
```

The test must use the `Display` trait directly (not `Serialize`) to confirm surface independence.

### AC-160-006 (CSV and terminal schema_version regression)

The CSV reporter and terminal reporter are unaffected.

```
test_BC_2_11_037_schema_version_absent_from_csv      — pass
    Asserts: no "schema_version" in CSV output
test_BC_2_11_037_schema_version_absent_from_terminal — pass
    Asserts: no "schema_version" in terminal output
test_BC_2_11_036_csv_category_unchanged              — pass
    Asserts: ThreatCategory::LateralMovement in CSV renders as "LateralMovement"
    (Debug repr via Display — unchanged by serde annotation)
```

### AC-160-007 (Existing JSON-asserting tests updated)

Any existing test in `tests/reporter_json_tests.rs` or `src/reporter/json.rs` that asserts
exact JSON enum string values (`"Likely"`, `"High"`, `"LateralMovement"`, etc.) is updated
to the new lowercase/snake_case forms. The test suite passes with `cargo test --all-targets`
after the change.

The scan command to find stale JSON string literals (scoped to the two files that contain
JSON value assertions):

```bash
grep -rn '"Likely"\|"Unlikely"\|"Inconclusive"\|"Possible"\|"High"\|"Medium"\|"Low"\|"LateralMovement"\|"CredentialAccess"\|"Reconnaissance"\|"Exfiltration"\|"Persistence"\|"Execution"\|"Anomaly"\|"Suspicious"\|"Impact"\|"C2"' tests/reporter_json_tests.rs src/reporter/json.rs
```

must return zero results in `assert_eq!` or `.contains()` argument slots after the change
(i.e., positions where the string is an asserted expected JSON value, not a struct-field
name or inline comment). Note: these strings remain valid Rust variant names — only JSON
value assertions are targeted.

A second scan clause targets the five legacy envelope key literals in `tests/reporter_json_tests.rs`:

```bash
grep -n '"summary"\|"findings"\|"analyzers"\|"mitre_domain"\|"mitre_attack_version"' tests/reporter_json_tests.rs
```

Before the change this returns exactly one hit (the `test_BC_2_11_001_top_level_keys` vec
assertion); after the update it returns zero. The enum-literal clause above returns zero
both before and after (belt-and-braces check). The envelope-key clause is the active check.

### AC-160-008 (CHANGELOG.md BREAKING CHANGE entry)

`CHANGELOG.md` contains an unreleased section entry for v0.12.0 with a BREAKING CHANGE note
that covers:

1. `verdict`, `confidence`, and `category` JSON field values are now lowercase / snake_case
   (Suricata EVE / ECS / OCSF convention). Full mapping table (or reference to BC-2.11.036).
2. A new `schema_version: "2"` field appears in every JSON report envelope.
3. Terminal Display tokens (`"LIKELY"`, `"HIGH"`) and CSV output are UNCHANGED.
4. JSON schema changes are outside `cargo-semver-checks` scope; this entry is the authoritative
   change notice.

### AC-160-009 (PR type)

The pull request title uses the `feat:` semantic prefix (e.g.,
`feat(reporter): align JSON enum casing + schema_version envelope (#255)`), consistent with
the v0.12.0 breaking JSON change and `feat` type used for prior JSON output additions.

### AC-160-010 (BC-2.11.001 amended to v1.9 in the same PR)

BC-2.11.001 is amended to v1.9 in the same PR as the production code changes. The v1.9
amendment targets the **Description block**, **Postcondition 2**, and **Canonical Test
Vector rows** — the three locations that enumerate the JSON envelope's top-level keys.
**Invariant 1** governs `unwrap()` infallibility of `JsonReporter::render` and is
explicitly **OUT OF SCOPE** for this amendment (it does not enumerate key count or names):

- **Description block** is updated to list six top-level keys (adding `schema_version`).
- **Postcondition 2** is updated from five to six top-level JSON keys, adding `schema_version`
  to the enumerated key list.
- **Canonical Test Vector rows** are updated to include `schema_version` in the expected
  JSON envelope output.
- A **modified-log entry** (`v1.9`) is appended that: (1) resolves the v1.8 advisory pointer
  and (2) includes corrective note: "v1.8 misidentified Invariant 1 as key-enumerating;
  Invariant 1 governs `unwrap()` infallibility; correct amendment scope: Description +
  Postcondition 2 + Canonical Test Vectors."
- The **BC-INDEX row** for BC-2.11.001 (row ~555) is updated in the same factory-artifacts
  delivery burst: the row title is amended to include `schema_version` (six keys, up from
  five), and the `v1.9` annotation is appended to the row's version-annotation trail.
  BC-INDEX version is bumped with a corresponding changelog line. This is a
  **DF-SIBLING-SWEEP-001** requirement — omitting the BC-INDEX update leaves the index
  inconsistent with the BC content.
- The **pre-existing `test_BC_2_11_001_top_level_keys` assertion** at
  `tests/reporter_json_tests.rs:66-111` is updated from the five-key vec to the six-key
  vec in the same develop PR:
  `assert_eq!(keys, vec!["analyzers", "findings", "mitre_attack_version", "mitre_domain", "schema_version", "summary"])`
  (`schema_version` inserts alphabetically between `mitre_domain` and `summary`). This is a
  **DF-SIBLING-SWEEP-001** requirement — this test WILL fail when `schema_version` lands
  and is the primary consuming-test surface for BC-2.11.001 v1.9.

> **Note for implementer:** `.factory/` lives on the orphan `factory-artifacts` branch and
> cannot be included in a `develop`-targeted PR. Commit BC-2.11.001 v1.9 and the BC-INDEX
> row update to `factory-artifacts` in the same delivery burst as the develop PR.
> STORY-160's `input-hash:` is computed from BC-2.11.001 at story-draft time (v1.8) and
> will drift after the v1.9 amendment — recompute with
> `bin/compute-input-hash --write .factory/stories/STORY-160.md` on `factory-artifacts`
> before closing the delivery burst.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| Verdict enum serde annotation | `src/findings.rs` (amend derive) | Pure |
| Confidence enum serde annotation | `src/findings.rs` (amend derive) | Pure |
| ThreatCategory enum serde annotation | `src/findings.rs` (amend derive) | Pure |
| SCHEMA_VERSION constant + envelope wiring | `src/reporter/json.rs` (amend) | Pure |
| New BC-driven unit tests | `tests/` or `src/reporter/json.rs` tests module | Test |
| CHANGELOG | `CHANGELOG.md` (amend) | Documentation |

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `src/findings.rs` | Pure | serde `rename_all` is a compile-time annotation on derive; no runtime I/O or mutable state |
| `src/reporter/json.rs` | Pure | Constant string insertion into in-memory `serde_json::Value`; returns owned String |
| `CHANGELOG.md` | Documentation | Markdown text |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `ThreatCategory::C2` serialized to JSON | `"c2"` — single uppercase letter lowercased; digit `2` treated as non-alpha continuation by serde snake_case algorithm; no underscore inserted |
| EC-002 | `ThreatCategory::LateralMovement` serialized to JSON | `"lateral_movement"` — word boundary before uppercase `M` creates underscore |
| EC-003 | `schema_version` field when findings slice is empty | `"schema_version": "2"` present alongside `"findings": []` |
| EC-004 | `schema_version` value type | JSON string `"2"`, not JSON integer `2` — consumers MUST compare as string |
| EC-005 | Terminal renderer reading Verdict | Reads `fmt::Display`, not `serde`; "LIKELY" is unchanged |
| EC-006 | CSV renderer reading ThreatCategory | Reads `{:?}` Debug repr (`"LateralMovement"`); unchanged |
| EC-007 | Future ThreatCategory variant with multiple uppercase words (e.g., `NewMultiWordThreat`) | Automatically inherits snake_case via `rename_all`; no per-variant override needed |

## Tasks

1. **Add serde rename_all attributes.** In `src/findings.rs`, add:
   - `#[serde(rename_all = "lowercase")]` to the `Verdict` derive attribute group
   - `#[serde(rename_all = "lowercase")]` to the `Confidence` derive attribute group
   - `#[serde(rename_all = "snake_case")]` to the `ThreatCategory` derive attribute group
   Do NOT modify `impl fmt::Display` for any of the three enums.
   The `#[non_exhaustive]` attribute on all three enums is orthogonal to `serde(rename_all)` —
   the attribute affects match exhaustiveness for downstream crates; `rename_all` affects only
   the `Serialize` output.

2. **Add SCHEMA_VERSION constant to json.rs.** In `src/reporter/json.rs`, add:
   ```rust
   const SCHEMA_VERSION: &str = "2";
   ```
   Then wire `"schema_version": SCHEMA_VERSION` into the top-level `serde_json::json!({})` or
   equivalent Value construction in `JsonReporter::render`. Pattern follows `MITRE_DOMAIN` and
   `MITRE_ATTACK_VERSION` (BC-2.11.001 Invariants 4 and 5).

3. **Write BC-driven tests.** Author the fourteen unit tests named in the BC-2.11.036 and
   BC-2.11.037 VP tables (nine from BC-2.11.036 + five from BC-2.11.037). Place them in the
   appropriate module (likely `tests/reporter_json_tests.rs` or the
   `src/reporter/json.rs` tests block). Follow DF-TEST-NAMESPACE-001 mod-wrapper convention
   if applicable.

4. **Update existing JSON-asserting tests.** Run both scan greps from AC-160-007:
   - Enum-literal scan: update any hit in a JSON value assertion context to the new
     lowercase/snake_case forms.
   - Envelope-key scan: the one pre-change hit is `test_BC_2_11_001_top_level_keys` at
     `tests/reporter_json_tests.rs:66-111`. Update its vec from the five-key form to the
     six-key form:
     `assert_eq!(keys, vec!["analyzers", "findings", "mitre_attack_version", "mitre_domain", "schema_version", "summary"])`
     (`schema_version` inserts alphabetically between `mitre_domain` and `summary`.)
   Confirm `cargo test --all-targets` is green after all updates.

5. **Verify clippy.** Run `cargo clippy --all-targets -- -D warnings`. No new warnings.

6. **Update CHANGELOG.md.** Add the BREAKING CHANGE entry (AC-160-008) in the unreleased /
   v0.12.0 section. Include the full mapping table or a clear prose summary.

7. **Open a `feat:` pull request** targeting `develop` with all file changes.

8. **Amend BC-2.11.001 to v1.9 and update BC-INDEX (AC-160-010) — factory-artifacts branch:**
   In `.factory/specs/behavioral-contracts/ss-11/BC-2.11.001.md`, update the Description
   block, Postcondition 2, and Canonical Test Vector rows to enumerate six top-level JSON
   keys (adding `schema_version`). Do NOT amend Invariant 1 — it governs `unwrap()`
   infallibility of `JsonReporter::render`, not key enumeration, and is explicitly out of
   scope. Append a v1.9 modified-log entry that: (1) resolves the v1.8 advisory pointer and
   (2) includes corrective note: "v1.8 misidentified Invariant 1 as key-enumerating; Invariant
   1 governs `unwrap()` infallibility; correct amendment scope: Description + Postcondition 2
   + Canonical Test Vectors." Also update the BC-INDEX row for BC-2.11.001 (row ~555): amend
   the row title to include `schema_version` (six keys) and append the `v1.9` annotation to
   the row's version-annotation trail; bump BC-INDEX version with a changelog line.
   **Commit BC-2.11.001 v1.9 and the BC-INDEX row update to the `factory-artifacts` branch in
   the same delivery burst as the develop PR — do NOT attempt to include `.factory/` paths in
   the develop PR.** Recompute STORY-160's input-hash on `factory-artifacts` after the BC
   amendment (`bin/compute-input-hash --write .factory/stories/STORY-160.md`).

## Previous Story Intelligence

Lessons from closest analogues:

- **STORY-129 (issue #64, wave 57, E-8):** Added `mitre_attack` JSON array to `FindingJsonDto`
  via a new DTO wrapper. Pattern: new field → constant definition → wiring in `render` →
  test per BC VP table. Follow the same test-naming convention (`test_BC_2_NN_NNN_*`).
- **STORY-101 (wave 31, E-13):** Extended the JSON envelope with additional reporter fields.
  `serde_json::json!` macro is the pattern used in `JsonReporter::render`; inspect the current
  envelope construction before adding `schema_version`.
- **DF-AC-TEST-NAME-SYNC-001:** Test names in this story's ACs come directly from BC-2.11.036
  and BC-2.11.037 VP tables. Do NOT rename them — the BC is the canonical source.

## Architecture Compliance Rules

- This story modifies: `src/findings.rs` (three enum derive blocks ONLY — no other changes),
  `src/reporter/json.rs` (SCHEMA_VERSION constant + envelope wiring), `CHANGELOG.md`, and test
  files. No other files.
- `fmt::Display` implementations for `Verdict`, `Confidence`, `ThreatCategory` are
  **explicitly off-limits** — any change there would violate BC-2.09.003 and BC-2.09.004.
- The CSV reporter (`src/reporter/csv.rs`) is **explicitly off-limits** — BC-2.11.020 governs
  nine fixed CSV columns and must not change.
- The terminal reporter (`src/reporter/terminal.rs`) is **explicitly off-limits** for enum
  rendering.

## Library & Framework Requirements

- `serde` with `derive` feature: already a Cargo dependency. No new dependencies.
- `serde_json`: already a Cargo dependency.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/findings.rs` | Modify | Add `#[serde(rename_all = ...)]` to three enum derive blocks |
| `src/reporter/json.rs` | Modify | Add `SCHEMA_VERSION` constant; add `schema_version` to envelope |
| `tests/reporter_json_tests.rs` | Modify/Create | Fourteen BC-driven tests from AC-160-001 through AC-160-006 (nine from BC-2.11.036 VP table + five from BC-2.11.037 VP table) |
| `CHANGELOG.md` | Modify | v0.12.0 BREAKING CHANGE entry |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~4 k |
| `src/findings.rs` annotation changes (3 one-liners) | ~0.1 k |
| `src/reporter/json.rs` constant + wiring | ~0.2 k |
| Fourteen new unit tests (9 from BC-2.11.036 + 5 from BC-2.11.037) | ~2 k |
| Existing test updates | ~0.5 k |
| `CHANGELOG.md` entry | ~0.5 k |
| **Total** | **~7.3 k** |

Well within context window. No story split required.

## Notes

- **Provenance:** GitHub issue #255. Validated by research agent (triage-2026-07-08). BCs authored
  by product owner in commit 00d67b1 (BC-INDEX v2.21, 2026-07-08). Story drafted in wave-72
  planning burst.
- **Breaking surface:** JSON output only. `cargo-semver-checks` does NOT detect JSON value
  serialization changes. CHANGELOG entry and BC enforcement are the authoritative mechanisms.
- **v0.12.0 target:** This story and STORY-161 (VP-024 proof_file_hash re-lock) are wave-72
  candidates. No scheduling dependency between the two — they are independent.
- **C2 note:** `ThreatCategory::C2` → `"c2"` by serde snake_case algorithm. This is consistent
  with EVE/ECS abbreviated category names. Confirm via a direct `serde_json::to_string` unit test
  before claiming the variant is handled correctly.
- **DF-VALIDATION-001 gate:** Both BCs were authored after the triage research-validation pass
  (10/10 CONFIRMED). Story drafting is permitted per policy.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.5 | 2026-07-08 | story-writer | Adversary P5 fixes: F-W72-P5-001 (HIGH) — consuming-test sibling gap closed: AC-160-010 gains new bullet explicitly enumerating test_BC_2_11_001_top_level_keys at tests/reporter_json_tests.rs:66-111 as DF-SIBLING-SWEEP-001 consuming-test surface; vec update documented (five-key → six-key; schema_version inserts alphabetically between mitre_domain and summary). Task 4 updated to enumerate this test explicitly with the required six-key assert_eq! form. F-W72-P5-004 (LOW) — AC-160-007 expanded with second scan clause targeting vec! with five legacy envelope key literals in tests/reporter_json_tests.rs; enum-literal clause returns zero before and after (belt-and-braces); envelope-key clause is active check (exactly one pre-change hit: test_BC_2_11_001_top_level_keys). |
| 1.4 | 2026-07-08 | story-writer | Adversary P4 fixes: F-W72-P4-002 (HIGH) — AC-160-010 extended: BC-INDEX row for BC-2.11.001 (row ~555) MUST be updated in same factory-artifacts burst (six-key title; v1.9 annotation appended; BC-INDEX version bumped); DF-SIBLING-SWEEP-001 requirement. Task 8 extended with BC-INDEX update step. F-W72-P4-003 (MEDIUM) — Task 8 "Include this file in the same PR" rewritten to cross-branch wording: commit BC-2.11.001 v1.9 and BC-INDEX update to factory-artifacts in same delivery burst; do NOT include .factory/ paths in develop PR; recompute input-hash on factory-artifacts. AC-160-010 implementer note updated to match. F-W72-P4-004 (MEDIUM) — Task 3 test-file name corrected (tests/json_reporter_tests.rs → tests/reporter_json_tests.rs); File Structure Requirements row de-vaguified (named tests/reporter_json_tests.rs explicitly). |
| 1.3 | 2026-07-08 | story-writer | Adversary P3 fixes: F-W72-P3-010 (LOW) — Task 1: added note that #[non_exhaustive] on all three enums is orthogonal to serde(rename_all); attribute affects match exhaustiveness, not Serialize output. F-W72-P3-005 (MEDIUM) — AC-160-007: grep scope restricted from tests/ src/ to two named files (tests/reporter_json_tests.rs src/reporter/json.rs); surrounding prose updated to match. F-W72-P3-007 (MEDIUM) — AC-160-010 modified-log bullet and Task 8: added corrective note ("v1.8 misidentified Invariant 1 as key-enumerating; Invariant 1 governs unwrap() infallibility; correct amendment scope: Description + Postcondition 2 + Canonical Test Vectors"). |
| 1.2 | 2026-07-08 | story-writer | Adversary P2 fixes: F-W72-P2-001 (HIGH) — AC-160-010 rewritten: v1.9 amendment targets Description block + Postcondition 2 + Canonical Test Vector rows (not Invariant 1); Invariant 1 governs unwrap() infallibility and is explicitly OUT OF SCOPE; Task 8 updated to match. F-W72-P2-003 (MEDIUM) — "nine unit tests" corrected to "fourteen" in Task 3, Token Budget, and File Structure Requirements (BC-2.11.036 VP table has 9 rows + BC-2.11.037 has 5 = 14). F-W72-P2-011 (LOW) — AC-160-007 grep assertion tightened to assert_eq!/.contains() argument slots rather than vague "JSON-assertion contexts". |
| 1.1 | 2026-07-08 | story-writer | Adversary P1 fixes: F-W72-P1-002 (HIGH) — add AC-160-010 (BC-2.11.001 amended to v1.9 in same PR: Postcondition 2 + Invariant 1 updated from five to six top-level keys, modified-log entry resolves v1.8 advisory pointer; implementer note to recompute input-hash after BC amendment); Task 8 added. F-W72-P1-009 (LOW) — rename test `terminal_display_unchanged_uppercase` → `terminal_display_unchanged` in AC-160-005 (ThreatCategory Display is PascalCase-derived, not "uppercase"). |
| 1.0 | 2026-07-08 | story-writer | Initial authorship — triage-2026-07-08 #255 follow-up: JSON enum casing alignment (BC-2.11.036) + schema_version envelope (BC-2.11.037); wave-72 draft. |
