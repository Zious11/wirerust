---
document_type: story
story_id: STORY-101
epic_id: E-13
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
phase: 4
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.001.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.015.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.020.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.024.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/phase-f2-spec-evolution/prd-delta.md
input-hash: TBD
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-100]
blocks: []
behavioral_contracts:
  - BC-2.11.001
  - BC-2.11.013
  - BC-2.11.015
  - BC-2.11.017
  - BC-2.11.020
  - BC-2.11.024
verification_properties:
  - VP-016
  - VP-020
priority: P0
cycle: v0.3.0-multitag
wave: 31
target_module: reporter
subsystems: [SS-11]
estimated_days: 3
tdd_mode: strict
feature_id: issue-007-modbus-analyzer
github_issue: 7
# BC status: all BCs authored at v1.5/v1.6 as of 2026-06-09
input-hash: "dccf659"
---

# STORY-101: Multi-Tag Reporter Serialization + JSON Envelope Add-Ons

## Narrative

- **As a** SIEM integrator or SOC analyst consuming wirerust output
- **I want** the JSON report to include `mitre_domain` and `mitre_attack_version` envelope fields, the CSV reporter to correctly semicolon-join multi-technique values (with empty-string for empty vec), and the terminal reporter to render multi-ID technique strings and bucket findings by `mitre_techniques[0]` tactic
- **So that** downstream tooling can identify the ATT&CK matrix used, import multi-technique CSV cells correctly, and terminal reports correctly group Modbus co-attributed findings (e.g., `["T0855","T0836"]`) under the right kill-chain tactic

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.11.001 | JsonReporter Renders JSON Object with summary/findings/analyzers/mitre_domain/mitre_attack_version Keys |
| BC-2.11.013 | Terminal Reporter Groups Findings by mitre_techniques[0] Tactic |
| BC-2.11.015 | No-Technique or Unknown-ID Findings Land in Uncategorized |
| BC-2.11.017 | Default Rendering Emits MITRE: <id1>, <id2> (Multi-ID) |
| BC-2.11.020 | CsvReporter Emits Exactly Nine Columns (Column 6 = mitre_techniques, Semicolon-Join) |
| BC-2.11.024 | CsvReporter Encodes Empty Vec mitre_techniques as Empty String; Multi-Value as Semicolon-Joined |

## Acceptance Criteria

### AC-001 (traces to BC-2.11.001 postcondition 2 — envelope keys)
`JsonReporter::render` produces a top-level JSON object with exactly five keys: `"summary"`, `"findings"`, `"analyzers"`, `"mitre_domain"`, and `"mitre_attack_version"`. The `"mitre_domain"` value is the constant string `"ics-attack"`. The `"mitre_attack_version"` value is the constant string `"ics-attack-v15"` (placeholder — see AC-FLAG-001).
- **Test:** `test_json_report_envelope_has_mitre_domain_and_version()` — parse the JSON output, assert both keys exist at the top level with the expected values.

### AC-FLAG-001 (traces to BC-2.11.001 — version pin flag)
`mitre_attack_version` is defined as a constant in `src/reporter/json.rs`: `const MITRE_ATTACK_VERSION: &str = "ics-attack-v15";`. A `// FLAG(F4): verify this version covers T0888, T0855, T0836, T0835, T0831, T0814, T0806 at attack.mitre.org/resources/attack-data-and-tools/ before v0.3.0 release tag` comment accompanies the constant. F4 implementers MUST update the constant before the v0.3.0 release tag.
- **Test:** `test_mitre_attack_version_constant_has_f4_pin_flag_comment()` — not a runtime test; code review / grep-based assertion that the comment exists.

### AC-002 (traces to BC-2.11.017 — multi-ID terminal rendering)
When a `Finding` has `mitre_techniques: vec!["T0855", "T0836"]`, the terminal reporter emits `"MITRE: T0855, T0836"` (comma-space separated IDs) on the MITRE output line. A single-technique finding with `vec!["T1027"]` emits `"MITRE: T1027"` (unchanged from prior behavior). An empty vec produces no MITRE line.
- **Test:** `test_terminal_renders_multi_id_mitre_string()` — drive the terminal reporter with a two-technique Finding; assert the rendered line contains `"MITRE: T0855, T0836"`.

### AC-003 (traces to BC-2.11.013 — tactic grouping by mitre_techniques[0])
The terminal reporter groups findings by the tactic of `mitre_techniques[0]` (first element). For a finding with `mitre_techniques: vec!["T0855", "T0836"]`, the finding appears under the `IcsImpairProcessControl` tactic bucket (T0855's tactic). The tactic is determined by `technique_tactic(mitre_techniques[0])`. This is deterministic because emission sites follow the canonical order T0806 > T0855 > T0836 > T0835 > T0831 > T0814 > T0888 per ADR-006 sub-decision 3.
- **Test:** `test_terminal_tactic_grouping_uses_first_technique()` — two findings: one with `["T0855","T0836"]` (first = T0855 → `IcsImpairProcessControl`), one with `["T0806","T0855"]` (first = T0806 → `IcsImpairProcessControl`); assert they both land in the same tactic bucket.

### AC-004 (traces to BC-2.11.015 — empty vec → Uncategorized)
A `Finding` with `mitre_techniques: vec![]` lands in the `Uncategorized` terminal bucket. A `Finding` with a non-empty vec where all technique IDs are unknown (not in the MITRE catalog) also lands in `Uncategorized`. The terminal reporter's `Uncategorized` heading appears only when at least one such finding exists.
- **Test:** `test_terminal_empty_techniques_lands_in_uncategorized()` and `test_terminal_unknown_id_lands_in_uncategorized()`.

### AC-005 (traces to BC-2.11.020 postcondition 3 — column 6 header rename)
`CsvReporter` emits the header row `"timestamp,category,verdict,confidence,source_ip,mitre_techniques,summary,direction,evidence"`. Column 6 is `mitre_techniques` (not `mitre_technique`). Column count remains 9. The comma delimiter is explicitly configured (not locale-default).
- **Test:** `test_csv_header_column_6_is_mitre_techniques()` — parse the first row of CSV output and assert column index 5 (0-based) equals `"mitre_techniques"`.

### AC-006 (traces to BC-2.11.024 postcondition — semicolon join and empty string)
- `mitre_techniques: vec![]` → CSV column 6 is `""` (empty string, NOT `"null"`, `"[]"`, or `"N/A"`).
- `mitre_techniques: vec!["T0836"]` → `"T0836"` (identical to prior scalar behavior).
- `mitre_techniques: vec!["T0855", "T0836"]` → `"T0855;T0836"` (semicolons, no space).
- `mitre_techniques: vec!["T0855", "T0836", "T0831"]` → `"T0855;T0836;T0831"`.
- **Test:** `test_csv_multi_technique_semicolon_join()` and `test_csv_empty_technique_is_empty_string()` — assert each encoding.

### AC-007 (traces to BC-2.11.024 invariant — EC-015 consumer guard documented)
The CSV column-6 encoding contract (empty string for empty vec; `str.split(';')` on empty string produces `['']`, not `[]`) is documented in a code comment on the `join(";")` call site. This is the BC-2.11.024 EC-015 consumer split guard.
- **Test:** code review / grep for the comment. Not a runtime assertion.

### AC-008 (traces to BC-2.11.001 invariant — CSV carries no envelope fields)
`CsvReporter` does NOT emit `mitre_domain` or `mitre_attack_version` fields. These are JSON-only envelope fields per BC-2.11.001 and prd-delta.md §5.3. The CSV column count remains 9.
- **Test:** `test_csv_has_no_envelope_fields()` — parse CSV and assert no column named `mitre_domain` or `mitre_attack_version` exists.

### AC-009 (traces to BC-2.11.013 — VP-016 harness green)
The VP-016 proof harness (mitre-tactic-grouping-order) passes `cargo test` after the terminal reporter updates in this story. The harness constructs `Finding { mitre_techniques: vec![...] }` (updated in STORY-100) and tests tactic-group ordering.
- **Test:** `cargo test --all-targets` green including VP-016 test function.

### AC-010 (traces to BC-2.11.024 — VP-020 harness green)
The VP-020 proof harness (csv-injection-neutralization) passes `cargo test` after the CSV reporter update. The harness constructs `Finding { mitre_techniques: vec!["T1036"] }` (updated in STORY-100) and tests that CSV-injection characters in technique IDs are neutralized (they are not present in MITRE IDs but the general guard still applies).
- **Test:** `cargo test --all-targets` green including VP-020 test function.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `JsonReporter::render` (envelope add-on) | `src/reporter/json.rs` | Effectful (I/O output) |
| `MITRE_ATTACK_VERSION` constant | `src/reporter/json.rs` | Pure (constant) |
| `TerminalReporter` (tactic grouping + multi-ID render) | `src/reporter/terminal.rs` | Effectful (I/O output) |
| `CsvReporter` (column 6 rename + semicolon join) | `src/reporter/csv.rs` | Effectful (I/O output) |
| VP-016 proof harness | test/Kani target | Pure (proof) |
| VP-020 proof harness | test/Kani target | Pure (proof) |

**Subsystem anchor justification:** SS-11 owns this story's complete scope. All changes are in `src/reporter/` — the Reporting subsystem (SS-11) per ARCH-INDEX Subsystem Registry. No other subsystem is modified; the `Finding` type changes landed in STORY-100 (SS-09/SS-10). This story consumes the updated `mitre_techniques: Vec<String>` field.

**Dependency anchor justification:** STORY-101 depends on STORY-100 because `mitre_techniques: Vec<String>` must exist in `src/findings.rs` before the reporters can consume it. The reporter changes in this story reference `f.mitre_techniques.join(";")` and `f.mitre_techniques.first()` — calls that do not compile until STORY-100 lands the type rename.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | JSON report with zero findings | `"findings": []`; `"mitre_domain": "ics-attack"`; `"mitre_attack_version": "ics-attack-v15"` still present in envelope |
| EC-002 | CSV with `mitre_techniques: vec!["T0855;injected"]` — semicolon in a technique ID | The `neutralize_csv_injection` guard applies to column values; MITRE IDs do not contain semicolons by spec, but guard still runs |
| EC-003 | Terminal report where two findings have the same first-technique tactic | Both findings appear in the same tactic bucket; ordering within bucket is by finding-emission order |
| EC-004 | Finding with `mitre_techniques: vec!["T9999"]` (unknown ID) | `technique_tactic("T9999")` returns `None` → Uncategorized bucket; terminal renders `"MITRE: T9999"` |
| EC-005 | CSV consumer splits empty column 6 on `;` | Produces `[""]` (one empty element) per EC-015; wirerust is not responsible; the consumer must guard `if cell.is_empty() { return vec![] }` |
| EC-006 | `mitre_attack_version` constant used without F4 verification | Placeholder value `"ics-attack-v15"` shipped; analyst can see it in output; F4 obligation: verify and update before tag |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~4,000 |
| `src/reporter/json.rs` (envelope add-on) | ~2,500 |
| `src/reporter/terminal.rs` (tactic grouping + multi-ID render) | ~5,000 |
| `src/reporter/csv.rs` (column rename + join) | ~2,500 |
| STORY-100 (dependency; need field shape) | ~1,500 (summary only) |
| BC files (6 BCs) | ~9,000 |
| `tests/reporter_tests.rs` (new reporter tests) | ~6,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~31,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~16%** |

## Tasks (MANDATORY)

1. [ ] Write failing test for AC-001: `test_json_report_envelope_has_mitre_domain_and_version()`. Red Gate: `"mitre_domain"` key missing in current output.
2. [ ] Write failing test for AC-002: `test_terminal_renders_multi_id_mitre_string()`. Red Gate: current terminal renders only the first technique or nothing for a multi-vec finding.
3. [ ] Write failing test for AC-005: `test_csv_header_column_6_is_mitre_techniques()`. Red Gate: current column 6 header is `mitre_technique` (singular) — fails on the name check. (Note: STORY-100 already changed the field type; the CSV header rename is this story's responsibility.)
4. [ ] Write failing test for AC-006: `test_csv_multi_technique_semicolon_join()`. Red Gate: multi-technique join not yet implemented.
5. [ ] **Red Gate:** Confirm `cargo test` fails on new assertions before production changes.
6. [ ] Add `mitre_domain` and `mitre_attack_version` to `JsonReporter::render` in `src/reporter/json.rs`. Add the `MITRE_ATTACK_VERSION: &str = "ics-attack-v15"` constant with the F4 pin-flag comment.
7. [ ] Update `TerminalReporter` in `src/reporter/terminal.rs` — tactic grouping: `f.mitre_techniques.first().and_then(|id| technique_tactic(id))` replaces the old `Option`-based access. Multi-ID render: `format!("MITRE: {}", f.mitre_techniques.join(", "))`. Empty-vec handling: skip the MITRE line entirely (same behavior as old `None`).
8. [ ] Update `CsvReporter` in `src/reporter/csv.rs` — column 6 header rename to `mitre_techniques`; value: `f.mitre_techniques.join(";")` (produces `""` for empty vec, `"T0836"` for singleton, `"T0855;T0836"` for two). Add EC-015 consumer-split-guard comment.
9. [ ] Verify `CsvReporter` explicitly uses comma as field delimiter (not semicolon; that is only the intra-cell separator for column 6). Add comment if not already explicit.
10. [ ] Update VP-016 and VP-020 harnesses if any further `Finding` construction changes are needed beyond what STORY-100 covered.
11. [ ] **Green Gate:** `cargo build --all-targets` exits 0. `cargo test --all-targets` green. AC-001 through AC-010 pass.
12. [ ] `cargo clippy --all-targets -- -D warnings` clean.
13. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-100 (direct predecessor) | Field rename: `mitre_techniques: Vec<String>`. CSV reader site updated to `join(";")`. Terminal reader sites updated. However, the CSV HEADER rename and the JSON ENVELOPE add-ons are this story's responsibility (STORY-100 only updated the value-encoding call sites). | `f.mitre_techniques.join(";")` produces `""` for empty vec — verified because `"".split(';')` in Python produces `['']`; document EC-015 comment at the call site. | STORY-100 is Wave 31; this story is also Wave 31 — they are in the same wave but STORY-101 depends on STORY-100. In practice, dispatch STORY-101 only after STORY-100's PR is merged and `cargo build` is green. |
| STORY-079 (existing — CsvReporter structure) | Column layout established. The 9-column header is declared at `csv.rs`. The CsvReporter separates header emission from row emission. Update only the header string and the column-6 join call. | | |
| STORY-078 (existing — MITRE grouping in terminal) | Tactic grouping logic lives in `terminal.rs`. The grouping iterates findings and groups by tactic bucket. | | |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `"mitre_domain"` and `"mitre_attack_version"` appear ONCE at top level of JSON report — NOT per-finding | BC-2.11.001 v1.5; prd-delta.md §5.3 ADD-ON 1 | Code review: confirm keys are in the `json!({...})` object at the report level |
| CSV field delimiter is comma (`,`); semicolon is intra-cell separator only for column 6 | ADR-006 Decision 13 §13.3 sub-decision 2 | Code review: explicit `.comma_delimiter(',')` or equivalent in the CSV writer initialization |
| Tactic grouping uses `mitre_techniques[0]` (first element), not the last or any other | BC-2.11.013 v1.6; ADR-006 Decision 13 §13.7 | Code review; AC-003 test |
| `mitre_attack_version` constant MUST have the F4 pin-flag comment | prd-delta.md §5.3 ADD-ON 1 FLAG note | Code review; AC-FLAG-001 grep check |
| CSV column count remains exactly 9 | BC-2.11.020 v1.5 postcondition | AC-005 test; existing 9-column test must still pass |
| `src/reporter/` MUST NOT import `src/analyzer/` | Architecture layer rule (ARCH-INDEX L4 Output must not depend on L3 analyzer internals) | Compiler module system |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `serde_json` | workspace version | `json!({})` macro in `JsonReporter::render` for envelope fields |
| `csv` crate | workspace version | CSV writer; comma delimiter explicit configuration |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reporter/json.rs` | **modify** | Add `MITRE_ATTACK_VERSION` constant + F4 comment; add `"mitre_domain"` and `"mitre_attack_version"` to the `json!({})` object |
| `src/reporter/terminal.rs` | **modify** | Tactic grouping: `mitre_techniques[0]`; multi-ID render: `join(", ")`; empty-vec: skip MITRE line |
| `src/reporter/csv.rs` | **modify** | Column 6 header rename; value: `join(";")`; EC-015 comment |
| `tests/reporter_tests.rs` (or equivalent) | **modify** | Add AC-001 through AC-010 tests |

## Forbidden Dependencies

`src/reporter/` MUST NOT import `src/analyzer/`. Reporters depend only on `Finding`, `Summary`, `AnalysisSummary` — all in `src/findings.rs` and `src/summary.rs`. The mitre catalog functions (`technique_tactic`) are accessed via `src/mitre.rs`. No analyzer import is needed or allowed.
