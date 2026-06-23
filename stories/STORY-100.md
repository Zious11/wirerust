---
document_type: story
story_id: STORY-100
epic_id: E-13
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
phase: 4
inputs:
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.005.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.007.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.008.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/phase-f2-spec-evolution/prd-delta.md
  - .factory/research/f2-decomposition-sequencing.md
traces_to: .factory/specs/prd.md
points: 13
depends_on: []
blocks: [STORY-101, STORY-102]
behavioral_contracts:
  - BC-2.09.001
  - BC-2.09.006
  - BC-2.10.005
  - BC-2.10.007
  - BC-2.10.008
verification_properties:
  - VP-007
  - VP-016
  - VP-020
  - VP-021
priority: P0
cycle: v0.3.0-multitag
wave: 31
target_module: findings
subsystems: [SS-09, SS-10, SS-06, SS-07, SS-04]
estimated_days: 5
tdd_mode: strict
feature_id: issue-007-modbus-analyzer
github_issue: 7
# BC status: all BCs authored and confirmed at v1.4/v1.5 as of 2026-06-09
input-hash: "2713edb"
---

# STORY-100: Multi-Tag Finding Schema Migration (Atomic Type Rename + Catalog Seed)

## Narrative

- **As a** SIEM integrator consuming wirerust JSON output
- **I want** the `Finding` type to carry `mitre_techniques: Vec<String>` instead of `mitre_technique: Option<String>`, with the MITRE catalog seeded for all 21 post-F2 technique IDs
- **So that** co-attributed ICS findings (e.g., `["T1692.001","T0836"]`) are expressible in the type system, all existing single-technique findings continue to work (singleton vec), and the JSON schema change ships as one atomic, behavior-preserving wave before the Modbus analyzer is built on top of it

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.09.001 | Finding Constructed with Required Fields and Optional Fields |
| BC-2.09.006 | Finding JSON Serialization: Empty Vec Fields Omitted; mitre_techniques Serialized as Array |
| BC-2.10.005 | technique_name Returns Some for Every Seeded ID (21 Total) |
| BC-2.10.007 | technique_tactic Returns Correct Tactic for Every Seeded ID |
| BC-2.10.008 | All Emitted Technique IDs Resolve in Lookup |

## Acceptance Criteria

### AC-001 (traces to BC-2.09.001 postcondition 1 — field rename)
`Finding.mitre_techniques` is declared as `pub mitre_techniques: Vec<String>` in `src/findings.rs` with `#[serde(skip_serializing_if = "Vec::is_empty")]`. The old field `mitre_technique: Option<String>` is removed. `cargo build --all-targets` exits 0 after all ~21 emission sites are updated.
- **Test:** compile-time — the compiler forces all emission sites to update atomically; any stale `mitre_technique:` literal is a compile error.

### AC-002 (traces to BC-2.09.001 postcondition 1 — empty vec migration)
All emission sites that previously set `mitre_technique: None` now set `mitre_techniques: vec![]`. All emission sites that previously set `mitre_technique: Some("TXXXX")` now set `mitre_techniques: vec!["TXXXX"]`. The blast-radius files are: `src/analyzer/http.rs` (~8 sites), `src/analyzer/tls.rs` (~7 sites), `src/reassembly/mod.rs` (~4 sites), `src/reassembly/lifecycle.rs` (~2 sites).
- **Test:** `test_all_emission_sites_use_vec_field()` — grep-based or compile-time assertion that no `mitre_technique:` literal exists outside of comments; `cargo build --all-targets` green.

### AC-003 (traces to BC-2.09.006 postcondition 2 — JSON array output)
A `Finding` with `mitre_techniques: vec!["T1027"]` serializes to JSON with `"mitre_techniques": ["T1027"]` (array, not scalar string). A `Finding` with `mitre_techniques: vec![]` produces no `"mitre_techniques"` key in JSON (Vec::is_empty skip).
- **Test:** `test_single_technique_serializes_as_json_array()` and `test_empty_techniques_key_absent()` in `tests/reporter_tests.rs` — parse the JSON output and assert key type and presence.

### AC-004 (traces to BC-2.09.006 invariant 4 — no scalar regression)
No JSON output produced by `JsonReporter` contains a `"mitre_technique"` key (old scalar field). The old key is gone from all serialized output after this story.
- **Test:** `test_no_scalar_mitre_technique_key_in_json()` — serialize a batch of findings including single-technique, multi-technique, and empty-technique; assert old key is absent; assert new key is present or absent per `skip_serializing_if` semantics.

### AC-005 (traces to BC-2.10.005 postcondition 3 — seeded count 21)
`technique_name` returns `Some(_)` for all 21 seeded technique IDs post-F2. The 6 new ICS arms are added to `technique_info`: T0836 ("Modify Parameter"), T0814 ("Denial of Service"), T0806 ("Brute Force I/O"), T0835 ("Manipulate I/O Image"), T0831 ("Manipulation of Control"), T0888 ("Remote System Information Discovery"). `SEEDED_TECHNIQUE_ID_COUNT` is updated from 15 to 21.
- **Test:** `test_technique_name_resolves_all_21_seeded_ids()` — assert `Some(_)` for each of the 21 IDs. Previously tested 15; update to all 21.

### AC-006 (traces to BC-2.10.007 postcondition 2 — tactic assignments)
`technique_tactic` returns the correct `MitreTactic` for all 6 new ICS IDs: T0836 → `IcsImpairProcessControl`, T0814 → `IcsInhibitResponseFunction`, T0806 → `IcsImpairProcessControl`, T0835 → `IcsImpairProcessControl`, T0831 → `IcsImpairProcessControl`, T0888 → `Discovery`.
- **Test:** `test_technique_tactic_correct_for_all_21_ids()` — exhaustive assertion over all seeded IDs including the 6 new ICS entries.

### AC-007 (traces to BC-2.10.008 — emitted IDs resolve; grep pattern update)
`EMITTED_IDS` in the Kani proof module is updated to 13 entries (6 Enterprise + 7 ICS): the 7 ICS emitted IDs are T1692.001, T0836, T0814, T0806, T0835, T0831, T0888. T0846 is NOT in `EMITTED_IDS` (seeded but not emitted). The VP-007 drift-guard comment in `mitre.rs` is updated: grep pattern changes from `mitre_technique: Some` to `mitre_techniques: vec!`.
- **Test:** `test_all_emitted_ids_resolve_in_lookup()` (VP-007 Kani / unit) — assert all 13 EMITTED_IDS return `Some` from `technique_name` and `technique_tactic`.

### AC-008 (traces to BC-2.09.001 invariant 6 — no Option<String> remains)
No `Option<String>` type is used for technique attribution at any emission site after this story. `grep -r 'mitre_technique' src/` returns only comments and the `SEEDED_TECHNIQUE_IDS` constant guard (no struct field or literal usage).
- **Test:** compile-time; confirmed by green `cargo build --all-targets`.

### AC-009 (traces to BC-2.09.006 — VP-016/VP-020 harness updates)
The VP-016 (mitre-tactic-grouping-order) and VP-020 (csv-injection-neutralization) proof harnesses in `src/` or `tests/` are updated: any `Finding { mitre_technique: ... }` construction is replaced with `Finding { mitre_techniques: vec![...] }`. These harnesses must compile and pass Kani / unit tests after the field rename.
- **Test:** `cargo test --all-targets` green with VP-016 and VP-020 test functions included.

### AC-010 (traces to BC-2.09.001 — existing stories' tests updated; behavior-preserving)
The 6 existing stories' test files (STORY-069/070/071/078/079/080 scope — specifically: `tests/findings_tests.rs`, `tests/reporter_tests.rs`, `tests/mitre_tests.rs`, and any test file containing `mitre_technique:` literals) are updated from `Some("TXXXX")` / `None` to `vec!["TXXXX"]` / `vec![]`. CSV and terminal output assertions are unchanged (singleton behavior is byte-identical for those reporters). JSON assertions are updated to assert array form `["TXXXX"]` not scalar `"TXXXX"`.
- **Test:** `cargo test --all-targets` green with no regressions on the existing reporter/analyzer/mitre test suites.

### AC-011 (traces to BC-2.09.001 invariant 6 — VP-021 test helpers updated)
The VP-021 timestamp-provenance test helpers in `tests/` that construct `Finding { mitre_technique: ... }` are updated to `mitre_techniques: vec![...]`. These tests must pass after the field rename.
- **Test:** `cargo test --all-targets` includes VP-021 tests passing with the updated `Finding` construction.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Finding` struct field rename | `src/findings.rs` | Pure (data type) |
| `technique_info` new arms (6 ICS) | `src/mitre.rs` | Pure (static match) |
| `SEEDED_TECHNIQUE_IDS` + count update | `src/mitre.rs` | Pure (constant) |
| `EMITTED_IDS` update | `src/mitre.rs` (Kani/test module) | Pure (constant) |
| VP-007 drift-guard comment | `src/mitre.rs` | N/A (comment) |
| `HttpAnalyzer` emission sites (~8) | `src/analyzer/http.rs` | Effectful (analyzer) |
| `TlsAnalyzer` emission sites (~7) | `src/analyzer/tls.rs` | Effectful (analyzer) |
| Reassembly emission sites (~6) | `src/reassembly/mod.rs`, `src/reassembly/lifecycle.rs` | Effectful |
| `CsvReporter` reader site | `src/reporter/csv.rs` | Effectful (I/O) |
| `TerminalReporter` reader sites | `src/reporter/terminal.rs` | Effectful (I/O) |
| `JsonReporter` | `src/reporter/json.rs` | Effectful (serde derive; no manual change) |
| Test files (6 existing stories' scope) | `tests/*.rs` | Test infrastructure |

**Subsystem anchor justification:**
- SS-09 owns this story's primary scope because `Finding` lives in `src/findings.rs` — the Finding Emission subsystem per ARCH-INDEX.
- SS-10 is touched because `mitre.rs` (MITRE Mapping) gains 6 new technique arms and updated constants.
- SS-06, SS-07, SS-04 are mechanical update sites (emission literals only) — no new logic.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Emission site sets `mitre_techniques: vec![]` (was `None`) | JSON: no `"mitre_techniques"` key (Vec::is_empty skip); CSV: empty string in column 6; terminal: no MITRE line |
| EC-002 | Emission site sets `mitre_techniques: vec!["T1027"]` (was `Some("T1027")`) | JSON: `"mitre_techniques": ["T1027"]` (array, not scalar); CSV: `"T1027"` (identical to before); terminal: `MITRE: T1027` (identical to before) |
| EC-003 | `technique_name("T0888")` called | `Some("Remote System Information Discovery")` — new arm added in this story |
| EC-004 | `technique_name("T0846")` called | `Some("Remote System Discovery")` — pre-existing arm, unchanged, not emitted by Modbus |
| EC-005 | VP-007 drift guard checks `SEEDED_TECHNIQUE_ID_COUNT` | Now 21; test fails if someone adds a technique arm without updating the count constant |
| EC-006 | `cargo check` on partially-migrated tree (only `findings.rs` updated, not all emission sites) | Compile error at every stale emission site — compiler generates the migration checklist |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~5,000 |
| `src/findings.rs` (Finding struct + serde attrs) | ~3,000 |
| `src/mitre.rs` (technique_info + constants + Kani module) | ~6,000 |
| `src/analyzer/http.rs` (~8 emission sites) | ~8,000 |
| `src/analyzer/tls.rs` (~7 emission sites) | ~7,000 |
| `src/reassembly/mod.rs` + `lifecycle.rs` (~6 emission sites) | ~5,000 |
| `src/reporter/csv.rs` (reader site) | ~3,000 |
| `src/reporter/terminal.rs` (reader sites) | ~4,000 |
| `src/reporter/json.rs` (comment update only) | ~1,500 |
| `tests/` (existing stories' test files — 6 story scope) | ~12,000 |
| BC files (5 BCs: BC-2.09.001, BC-2.09.006, BC-2.10.005, BC-2.10.007, BC-2.10.008) | ~8,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~64,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~32%** |

> **Note:** 32% is slightly above the 20–30% guideline. The story cannot be split further because the compiler forces atomic migration of all ~21 emission sites. The research note in `f2-decomposition-sequencing.md §4.2` explicitly confirms this is a normal-sized atomic refactor for a single crate and should remain one commit/PR. The implementer should use `cargo check` to get the exhaustive list of broken sites before editing.

## Tasks (MANDATORY)

1. [ ] Write failing test for AC-003: `test_single_technique_serializes_as_json_array()` — currently passes because `vec!["T1027"]` does not exist yet; write as a build-time error test by referencing the new field name before it exists. Red Gate: `cargo build` fails on the test file because `mitre_techniques` field does not exist.
2. [ ] Write failing tests for AC-005: update `test_technique_name_resolves_every_seeded_id` to assert all 21 IDs. Currently only 15 IDs; the 6 new ones return `None` before this story — test goes RED on `technique_name("T0888")`.
3. [ ] **Red Gate:** Confirm `cargo test` FAILS on the new assertions (AC-003, AC-005) before any production changes.
4. [ ] Rename `Finding.mitre_technique: Option<String>` → `mitre_techniques: Vec<String>` in `src/findings.rs`. Update `#[serde(skip_serializing_if)]` from `"Option::is_none"` to `"Vec::is_empty"`.
5. [ ] Run `cargo check` — compiler emits the complete list of ~21 broken emission sites. Use this as the migration checklist.
6. [ ] Fix all emission sites in `src/analyzer/http.rs`: `Some("TXXXX")` → `vec!["TXXXX"]`; `None` → `vec![]`.
7. [ ] Fix all emission sites in `src/analyzer/tls.rs`: same pattern.
8. [ ] Fix all emission sites in `src/reassembly/mod.rs` and `src/reassembly/lifecycle.rs`: same pattern.
9. [ ] Fix `CsvReporter` reader at `src/reporter/csv.rs:82`: `f.mitre_technique.as_deref().unwrap_or("")` → `f.mitre_techniques.join(";")`.
10. [ ] Fix `TerminalReporter` reader sites in `src/reporter/terminal.rs` (3 sites): `if let Some(ref t) = f.mitre_technique` → iterate `f.mitre_techniques`; tactic grouping uses `f.mitre_techniques.first().map(|s| s.as_str())`.
11. [ ] Update `src/reporter/json.rs` module-level comment to note `mitre_techniques` array schema (serde derive handles the rest automatically).
12. [ ] Add 6 new ICS arms to `technique_info` in `src/mitre.rs`: T0836, T0814, T0806, T0835, T0831, T0888. Update `SEEDED_TECHNIQUE_IDS` from 15 to 21 entries. Update `SEEDED_TECHNIQUE_ID_COUNT` from 15 to 21. Update `EMITTED_IDS` in the Kani module to 13 entries (6 Enterprise + 7 ICS). Update VP-007 comment: grep pattern from `mitre_technique: Some` to `mitre_techniques: vec!`.
13. [ ] Update test files for existing story scope (STORY-069/070/071/078/079/080): replace `mitre_technique: Some("TXXXX")` with `mitre_techniques: vec!["TXXXX"]`; replace `mitre_technique: None` with `mitre_techniques: vec![]`; update JSON assertions to array form.
14. [ ] Update VP-016 and VP-020 proof harnesses: replace `Finding { mitre_technique: ... }` with `Finding { mitre_techniques: vec![...] }`.
15. [ ] Update VP-021 test helpers: same replacement.
16. [ ] **Green Gate:** `cargo build --all-targets` exits 0. `cargo test --all-targets` green with no regressions. AC-003, AC-005, AC-010 tests pass.
17. [ ] `cargo clippy --all-targets -- -D warnings` clean.
18. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-097 | Introduced `timestamp: u32` as 5th param to `StreamHandler::on_data` — a BREAKING trait change requiring all implementors to update atomically. Same pattern applies here: `Finding` field rename forces atomic update at all ~21 emission sites. | Compiler-enforced atomicity: use `cargo check` to get the exhaustive broken-site list before editing anything. Never merge a half-migrated state. | The `RecordingHandler` in `tests/reassembly_engine_tests.rs` is a hidden implementor; grep for all `Finding {` literals across `src/` AND `tests/` before committing — there are test-file emission sites that are easy to miss. |
| STORY-098 | Per-flow storage pattern in `HttpAnalyzer` and `TlsAnalyzer` stores `last_ts: u32` keyed by `FlowKey`. Relevant: emission sites in those files now need `mitre_techniques: vec!["TXXXX"]` instead of `mitre_technique: Some("TXXXX")`. | The 2026-06-09 research (`f2-decomposition-sequencing.md §3.1`) explicitly calls out that JSON singleton output changes from scalar to array — this IS a behavior change for JSON consumers; document it in the PR description as the one intended break. CSV and terminal output are byte-identical for singleton vecs. | |
| STORY-069 (existing) | Original `Finding` struct definition story. Tests assert `mitre_technique` field. ALL of these tests must be updated as part of this story — they are the "6 existing stories' tests" referenced in the research. | | |

**Design reference:** Per `f2-decomposition-sequencing.md §4.1`, the canonical edit order for developer ergonomics is: (1) change type in `findings.rs`, (2) run `cargo check` to get the broken-site list, (3) fix production code (reporters → analyzers → reassembly), (4) fix tests. This order is not required for correctness (the commit is green/red as a whole) but minimizes confusion.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `mitre_techniques: Vec<String>` with `#[serde(skip_serializing_if = "Vec::is_empty")]` — NOT `Option<String>` | BC-2.09.001 invariant 6; ADR-006 Decision 13 | Compiler: struct field type; `cargo build` fails if any site uses `Option` |
| CSV column 6 header MUST be `mitre_techniques` (not `mitre_technique`) | BC-2.11.020 v1.5 | Code review; existing CSV header test must assert new column name |
| CSV join uses semicolon: `f.mitre_techniques.join(";")` | ADR-006 Decision 13 §13.3 | Code review; AC-003 / test |
| Terminal tactic grouping uses `mitre_techniques[0]` (first element) | ADR-006 Decision 13 §13.7; BC-2.11.013 v1.6 | Code review; VP-016 proof harness |
| `SEEDED_TECHNIQUE_ID_COUNT` MUST be 21 after this story | BC-2.10.005 invariant 3; VP-007 drift guard | Unit test: `vp007_catalog_drift_guard` in `mitre.rs` fails if count diverges from arm count |
| T0846 REMAINS seeded (in `technique_info` and `SEEDED_TECHNIQUE_IDS`) but is NOT in `EMITTED_IDS` | BC-2.10.005 invariant 2; f2-fix-directives.md Decision 12 | Code review: T0846 arm present; T0846 absent from `EMITTED_IDS` |
| T0888 IS in both `technique_info`, `SEEDED_TECHNIQUE_IDS`, AND `EMITTED_IDS` | f2-fix-directives.md Decision 12 §12.3 | AC-007 unit test |
| `src/findings.rs` must NOT import any analyzer module | Architecture layer rule (ARCH-INDEX) | Compiler module system; `cargo build` fails on circular import |
| Canonical `vec![]` order at emission sites for co-emission: T0806 > T1692.001 > T0836 > T0835 > T0831 > T0814 > T0888 | ADR-006 Decision 13 §13.7 sub-decision 3 | Code review at STORY-104 emission sites (this story only migrates existing single-technique sites; canonical order enforced in STORY-104) |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `serde` | workspace version (current: `1.0`) | `#[derive(Serialize, Deserialize)]` on `Finding`; `skip_serializing_if` attribute |
| `serde_json` | workspace version | `to_string_pretty` in `JsonReporter`; test JSON parsing |
| `proptest` | workspace version | VP-016 / VP-020 harnesses that construct `Finding` |
| `kani` | workspace version (nightly, gated) | VP-007 drift guard in `mitre.rs` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/findings.rs` | **modify** | Field rename: `mitre_technique` → `mitre_techniques`; type `Option<String>` → `Vec<String>`; serde attr update |
| `src/mitre.rs` | **modify** | Add 6 new ICS technique arms; update `SEEDED_TECHNIQUE_IDS` (15→21); update `SEEDED_TECHNIQUE_ID_COUNT` (15→21); update `EMITTED_IDS` (6+7=13); update VP-007 grep comment |
| `src/analyzer/http.rs` | **modify** | All `mitre_technique: Some(...)` → `mitre_techniques: vec![...]`; `None` → `vec![]` (~8 sites) |
| `src/analyzer/tls.rs` | **modify** | Same pattern (~7 sites) |
| `src/reassembly/mod.rs` | **modify** | Same pattern (~4 sites) |
| `src/reassembly/lifecycle.rs` | **modify** | Same pattern (~2 sites) |
| `src/reporter/csv.rs` | **modify** | `f.mitre_technique.as_deref().unwrap_or("")` → `f.mitre_techniques.join(";")` |
| `src/reporter/terminal.rs` | **modify** | 3 reader sites: `Option` iteration → `Vec` iteration; tactic grouping uses `[0]` |
| `src/reporter/json.rs` | **modify** | Comment update only; serde derive handles array automatically |
| `tests/reporter_tests.rs` (or equivalent) | **modify** | Update `Finding` constructions; update JSON assertions to array form |
| `tests/findings_tests.rs` (or equivalent) | **modify** | Update `Finding` constructions |
| `tests/mitre_tests.rs` (or equivalent) | **modify** | Add 6 new ICS assertion rows; update count from 15 to 21 |
| VP-016 proof harness file | **modify** | `mitre_technique` → `mitre_techniques: vec![...]` |
| VP-020 proof harness file | **modify** | Same |
| VP-021 test helper file | **modify** | Same |

## Forbidden Dependencies

`src/findings.rs` MUST NOT import any of: `src/analyzer/`, `src/reporter/`, `src/reassembly/mod.rs`. The data flow is `analyzers → findings ← reporters`. Any import of an analyzer from `findings.rs` creates a cycle and the build MUST fail.
