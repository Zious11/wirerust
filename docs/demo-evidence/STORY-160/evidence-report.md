# Demo Evidence Report — STORY-160

**Story:** STORY-160 — Align JSON Finding-Enum Serialization to Lowercase/snake_case + schema_version Envelope  
**Wave:** 72  
**Branch:** feature/STORY-160-json-enum-casing  
**Recorded:** 2026-07-09  
**Scrub gate:** PASS — host-path grep (PG-W70-DEMO-SCRUB) returns zero results  

---

## Coverage Map

| AC | Description | Artifacts | Path |
|----|-------------|-----------|------|
| AC-160-001 | Verdict rename_all lowercase | AC-160-001-verdict-lowercase.gif / .webm / .tape | VHS tape below |
| AC-160-002 | Confidence rename_all lowercase | AC-160-002-confidence-lowercase.gif / .webm / .tape | VHS tape below |
| AC-160-003 | ThreatCategory rename_all snake_case | AC-160-003-category-snake-case.gif / .webm / .tape | VHS tape below |
| AC-160-004 | schema_version present in every JSON report | AC-160-004-schema-version-json.gif / .webm / .tape | VHS tape below |
| AC-160-005 | Terminal Display regression — uppercase tokens unchanged | AC-160-005-terminal-display-unchanged.gif / .webm / .tape | VHS tape below |
| AC-160-006 | CSV and terminal schema_version regression | AC-160-006-csv-terminal-regression.gif / .webm / .tape | VHS tape below |
| AC-160-007 | Existing JSON-asserting tests updated; full suite green | AC-160-007-full-test-suite.gif / .webm / .tape | VHS tape below |
| AC-160-008 | CHANGELOG.md BREAKING CHANGE entry | AC-160-008-changelog-entry.gif / .webm / .tape | VHS tape below |
| AC-160-009 | PR semantic prefix feat: | N/A — PR creation step; no recording needed | — |
| AC-160-010 | BC-2.11.001 amended to v1.9 on factory-artifacts | N/A — factory-artifacts branch; no develop-PR artifact | See note below |

---

## AC-160-001: Verdict rename_all lowercase

**BC:** BC-2.11.036  
**Tests verified:** `test_BC_2_11_036_verdict_likely_serializes_lowercase`, `test_BC_2_11_036_verdict_all_variants_lowercase`

Recording shows: `cargo test --test reporter_json_tests test_BC_2_11_036_verdict` — 2 tests pass, `test result: ok. 2 passed; 0 failed`.

- [AC-160-001-verdict-lowercase.gif](AC-160-001-verdict-lowercase.gif)
- [AC-160-001-verdict-lowercase.webm](AC-160-001-verdict-lowercase.webm)
- [AC-160-001-verdict-lowercase.tape](AC-160-001-verdict-lowercase.tape)

---

## AC-160-002: Confidence rename_all lowercase

**BC:** BC-2.11.036  
**Tests verified:** `test_BC_2_11_036_confidence_high_serializes_lowercase`, `test_BC_2_11_036_confidence_all_variants_lowercase`

Recording shows: `cargo test --test reporter_json_tests test_BC_2_11_036_confidence` — 2 tests pass.

- [AC-160-002-confidence-lowercase.gif](AC-160-002-confidence-lowercase.gif)
- [AC-160-002-confidence-lowercase.webm](AC-160-002-confidence-lowercase.webm)
- [AC-160-002-confidence-lowercase.tape](AC-160-002-confidence-lowercase.tape)

---

## AC-160-003: ThreatCategory rename_all snake_case

**BC:** BC-2.11.036  
**Tests verified:** `test_BC_2_11_036_threat_category_lateral_movement_snake_case`, `test_BC_2_11_036_threat_category_c2_snake_case`, `test_BC_2_11_036_threat_category_all_variants_snake_case`

Recording shows: `cargo test --test reporter_json_tests test_BC_2_11_036_threat_category` — 3 tests pass including `lateral_movement` and `c2` edge cases.

- [AC-160-003-category-snake-case.gif](AC-160-003-category-snake-case.gif)
- [AC-160-003-category-snake-case.webm](AC-160-003-category-snake-case.webm)
- [AC-160-003-category-snake-case.tape](AC-160-003-category-snake-case.tape)

---

## AC-160-004: schema_version present in every JSON report

**BC:** BC-2.11.037  
**Tests verified:** `test_BC_2_11_037_schema_version_present_in_json`, `test_BC_2_11_037_schema_version_value_is_two`, `test_BC_2_11_037_schema_version_unconditional_empty_findings`

Recording shows two parts:
1. **Live CLI demo:** `./target/release/wirerust analyze tests/fixtures/dns-remoteshell.pcap --json | jq '{schema_version:.schema_version,...}'` — output shows `"schema_version": "2"` (string, not integer).
2. **Unit tests:** `cargo test --test reporter_json_tests test_BC_2_11_037_schema_version` — 3 tests pass.

- [AC-160-004-schema-version-json.gif](AC-160-004-schema-version-json.gif)
- [AC-160-004-schema-version-json.webm](AC-160-004-schema-version-json.webm)
- [AC-160-004-schema-version-json.tape](AC-160-004-schema-version-json.tape)

---

## AC-160-005: Terminal Display regression — uppercase tokens unchanged

**BC:** BC-2.11.036 (VP row 8)  
**Tests verified:** `test_BC_2_11_036_terminal_display_unchanged` (asserts `Verdict::Likely` → `"LIKELY"`, `Confidence::High` → `"HIGH"`, `ThreatCategory::LateralMovement.to_string()` → `"LateralMovement"`)

Recording shows two parts:
1. **Unit test:** `test_BC_2_11_036_terminal_display_unchanged` passes — confirms `fmt::Display` is independent of `Serialize`.
2. **CLI terminal mode:** `wirerust analyze ... ` (no `--json`) — output shows `WIRERUST TRIAGE REPORT` header with no `schema_version` field visible.

- [AC-160-005-terminal-display-unchanged.gif](AC-160-005-terminal-display-unchanged.gif)
- [AC-160-005-terminal-display-unchanged.webm](AC-160-005-terminal-display-unchanged.webm)
- [AC-160-005-terminal-display-unchanged.tape](AC-160-005-terminal-display-unchanged.tape)

---

## AC-160-006: CSV and terminal schema_version regression

**BC:** BC-2.11.036, BC-2.11.037  
**Tests verified:** `test_BC_2_11_037_schema_version_absent_from_csv`, `test_BC_2_11_037_schema_version_absent_from_terminal`, `test_BC_2_11_036_csv_category_unchanged`

Recording shows two parts:
1. **Unit tests (3):** CSV and terminal regression tests pass — no `schema_version` in either surface; ThreatCategory CSV renders as `"LateralMovement"` (PascalCase Debug repr, unchanged).
2. **CLI CSV mode:** `wirerust analyze ... --csv` — header row is `category,verdict,confidence,...` with no `schema_version` column.

- [AC-160-006-csv-terminal-regression.gif](AC-160-006-csv-terminal-regression.gif)
- [AC-160-006-csv-terminal-regression.webm](AC-160-006-csv-terminal-regression.webm)
- [AC-160-006-csv-terminal-regression.tape](AC-160-006-csv-terminal-regression.tape)

---

## AC-160-007: Existing tests updated; full suite green

**Tests verified:** All test binaries via `cargo test --all-targets`

Recording shows: `cargo test --all-targets 2>&1 | grep -E '^test result:|^running [0-9]'` — all test suites report `test result: ok. N passed; 0 failed`. The full suite includes:
- `reporter_json_tests` (40 tests, including 14 new BC-driven tests and the updated `test_BC_2_11_001_top_level_keys` 6-key form)
- All analyzer tests (no regressions from serde annotation changes)

- [AC-160-007-full-test-suite.gif](AC-160-007-full-test-suite.gif)
- [AC-160-007-full-test-suite.webm](AC-160-007-full-test-suite.webm)
- [AC-160-007-full-test-suite.tape](AC-160-007-full-test-suite.tape)

---

## AC-160-008: CHANGELOG.md BREAKING CHANGE entry

**File:** `CHANGELOG.md` (Unreleased section)

Recording shows: `head -62 CHANGELOG.md` — displays the full BREAKING CHANGE entry including:
1. Enum casing mapping table (all 17 variants: Verdict x4, Confidence x3, ThreatCategory x10)
2. `"schema_version": "2"` envelope field (BC-2.11.037)
3. Terminal Display tokens and CSV output are UNCHANGED
4. JSON schema outside `cargo-semver-checks` scope — CHANGELOG is authoritative
5. Direction enum carve-out (retains PascalCase in v0.12.0)

- [AC-160-008-changelog-entry.gif](AC-160-008-changelog-entry.gif)
- [AC-160-008-changelog-entry.webm](AC-160-008-changelog-entry.webm)
- [AC-160-008-changelog-entry.tape](AC-160-008-changelog-entry.tape)

---

## AC-160-009: PR semantic prefix

No recording needed. The PR title uses `feat:` prefix (e.g., `feat(reporter): align JSON enum casing + schema_version envelope (#255)`), consistent with STORY-160's v0.12.0 breaking JSON change. Enforcement is via CI `amannn/action-semantic-pull-request`.

---

## AC-160-010: BC-2.11.001 amended to v1.9

No develop-branch recording. BC-2.11.001 v1.9 lives on the `factory-artifacts` branch (`.factory/` is an orphan branch not included in develop PRs). The amendment targets Description block, Postcondition 2, and Canonical Test Vector rows (adding `schema_version` to six-key enumeration). Evidence: the consuming-test surface is `test_BC_2_11_001_top_level_keys` in `tests/reporter_json_tests.rs` — this test was updated to the six-key `["analyzers", "findings", "mitre_attack_version", "mitre_domain", "schema_version", "summary"]` form and passes in the AC-160-007 full suite recording.

---

## Scrub Gate Result (PG-W70-DEMO-SCRUB)

Command: host-path grep per PG-W70-DEMO-SCRUB gate document  
Result: **PASS — zero results**

All absolute host paths scrubbed from `.tape` source files. Binary `.gif` / `.webm` recordings are exempt (binary files; not scanned by grep).
