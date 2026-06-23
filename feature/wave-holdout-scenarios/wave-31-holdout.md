---
document_type: holdout-scenario
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-09T00:00:00Z
wave: 31
cycle: v0.3.0-multitag
stories: [STORY-100, STORY-101]
feature_id: issue-007-modbus-analyzer
github_issue: 7
---

# Wave 31 Holdout Scenarios: Multi-Tag Schema Migration (v0.3.0)

> **Purpose:** Cross-story integration holdout for Wave 31. Validates that the
> combined STORY-100 (field rename + catalog seed) and STORY-101 (reporter
> multi-tag + JSON envelope) produce correct end-to-end behavior across all
> output formats. All existing analyzers must continue to function correctly
> (regression guard).

---

## HS-W31-001: Existing HTTP Finding Emits mitre_techniques Array in JSON

**Scope:** Cross-story (STORY-100 field + STORY-101 JSON reporter)
**Priority:** P0

**Setup:** Run `wirerust analyze <http-threat.pcap> --http --format json`
where the pcap contains an HTTP request that triggers a path-traversal finding
(currently emits `mitre_technique: Some("T1055")` or equivalent single technique).

**Assertions:**
1. JSON output contains `"mitre_techniques": ["T1055"]` (array form, not scalar).
2. The old key `"mitre_technique"` does NOT appear anywhere in the JSON output.
3. JSON top-level object contains `"mitre_domain": "ics-attack"` and `"mitre_attack_version": "ics-attack-v15"`.
4. The `"findings"` array is non-empty (at least 1 HTTP finding).
5. `cargo test --all-targets` exits 0 (no regressions in HTTP analyzer test suite).

**Regression guard:** Run the same pcap through `--format csv` and `--format terminal`.
- CSV column 6 value for a single-technique finding is `"T1055"` (identical to old behavior — singleton join).
- Terminal output line is `MITRE: T1055` (identical to old behavior).

---

## HS-W31-002: Existing TLS Finding Emits mitre_techniques Array in JSON

**Scope:** Cross-story (STORY-100 field + STORY-101 JSON reporter)
**Priority:** P0

**Setup:** Run `wirerust analyze <tls-weak.pcap> --tls --format json`
where the pcap triggers a weak-cipher or deprecated-version TLS finding.

**Assertions:**
1. JSON output contains `"mitre_techniques": ["T1573"]` (or the applicable technique) as an array.
2. The old scalar key `"mitre_technique"` is absent from all findings in the JSON.
3. JSON envelope includes `mitre_domain` and `mitre_attack_version` keys.
4. TLS findings are structurally correct (category, verdict, confidence, source_ip fields present).
5. No regression in TLS analyzer behavior (same findings as before the migration).

---

## HS-W31-003: Empty-Technique Finding — No mitre_techniques Key in JSON, Empty String in CSV

**Scope:** STORY-100 (skip_serializing_if = Vec::is_empty) + STORY-101 (CSV empty-string encoding)
**Priority:** P0

**Setup:** Synthesize or identify a Finding emitted with `mitre_techniques: vec![]`
(e.g., a parse-anomaly finding from the reassembly engine that carries no MITRE technique).

**Assertions:**
1. JSON output: the finding object does NOT contain a `"mitre_techniques"` key (Vec::is_empty skip).
2. CSV output: column 6 (mitre_techniques) is an empty string `""` — NOT `"null"`, `"[]"`, or `"N/A"`.
3. Terminal output: no `MITRE:` line is emitted for this finding.
4. The finding appears in the `Uncategorized` tactic bucket in terminal output.

---

## HS-W31-004: Multi-Technique Finding (Co-Attributed) — Correct Rendering in All Formats

**Scope:** STORY-100 + STORY-101 (all reporters)
**Priority:** P0

**Setup:** Directly construct a Finding with `mitre_techniques: vec!["T1692.001", "T0836"]`
in a unit/integration test (or inject via the Modbus analyzer after Wave 32).
Run through all three reporters.

**Assertions (JSON):**
1. `"mitre_techniques": ["T1692.001", "T0836"]` — array, two elements, correct order.
2. No `"mitre_technique"` key.
3. Envelope keys present.

**Assertions (CSV):**
1. Column 6 value: `"T1692.001;T0836"` (semicolons, no spaces).
2. Column count remains exactly 9.
3. Column 6 header: `mitre_techniques` (not `mitre_technique`).

**Assertions (terminal):**
1. MITRE line: `MITRE: T1692.001, T0836` (comma-space separated).
2. Finding bucketed under tactic of T1692.001 (first element = IcsImpairProcessControl).

---

## HS-W31-005: CSV Column 6 Header Rename Verified

**Scope:** STORY-101 (CsvReporter column rename)
**Priority:** P0

**Setup:** Run `wirerust analyze <any.pcap> --http --format csv` with at least one finding.

**Assertions:**
1. CSV header row column 5 (0-indexed) is exactly `mitre_techniques`.
2. Column header row has exactly 9 columns (no added/removed columns).
3. No column named `mitre_technique` (singular, old name) exists.

---

## HS-W31-006: JSON Envelope Fields Present Even with Zero Findings

**Scope:** STORY-101 (JsonReporter envelope)
**Priority:** P1

**Setup:** Run `wirerust analyze <empty.pcap> --http --format json`
(pcap with no HTTP traffic, so no findings are emitted).

**Assertions:**
1. JSON output is a valid JSON object.
2. `"mitre_domain": "ics-attack"` key present at top level.
3. `"mitre_attack_version": "ics-attack-v15"` key present at top level.
4. `"findings": []` (empty array, key still present).
5. `"summary"` key present.
6. `"analyzers"` key present.

---

## HS-W31-007: Regression — Existing Analyzer Behavior Unchanged (Behavior-Preservation Gate)

**Scope:** All existing analyzers (HTTP, TLS, DNS, Reassembly) — regression guard
**Priority:** P0

**Setup:** Run the full existing test suite against the post-Wave-31 codebase.

**Assertions:**
1. `cargo test --all-targets` exits 0.
2. No existing test name that was green before Wave 31 turns red.
3. Specific check: test functions in `tests/findings_tests.rs`, `tests/reporter_tests.rs`,
   `tests/mitre_tests.rs`, `tests/reporter_csv_tests.rs`, `tests/reporter_terminal_tests.rs`
   all pass (these are the 6 existing-story test files updated by STORY-100).
4. VP-016 (mitre-tactic-grouping-order) proof harness passes.
5. VP-020 (csv-injection-neutralization) proof harness passes.
6. VP-021 (timestamp-provenance) tests pass (STORY-099 scope; unaffected by schema change).
7. VP-007 (catalog-drift-guard) passes with `SEEDED_TECHNIQUE_ID_COUNT == 21`.

---

## HS-W31-008: MITRE Catalog Seeded to 21 Techniques — New ICS IDs Resolve

**Scope:** STORY-100 (technique_info + SEEDED_TECHNIQUE_IDS)
**Priority:** P0

**Setup:** Unit test or manual invocation of `technique_name` and `technique_tactic`.

**Assertions (for each of the 6 new ICS technique IDs):**

| ID | Expected Name | Expected Tactic |
|----|---------------|-----------------|
| T0836 | "Modify Parameter" | IcsImpairProcessControl |
| T0814 | "Denial of Service" | IcsInhibitResponseFunction |
| T0806 | "Brute Force I/O" | IcsImpairProcessControl |
| T0835 | "Manipulate I/O Image" | IcsImpairProcessControl |
| T0831 | "Manipulation of Control" | IcsImpact |
| T0888 | "Remote System Information Discovery" | IcsDiscovery |

**Additional assertions:**
- `SEEDED_TECHNIQUE_ID_COUNT == 21` (was 15).
- T0846 is seeded (in `technique_info`) but NOT in `EMITTED_IDS`.
- T0888 IS in both `technique_info` and `EMITTED_IDS`.
- All 13 EMITTED_IDS return `Some` from both `technique_name` and `technique_tactic`.

---

## Wave 31 Release Gate

Before creating the v0.3.0 release tag:

1. All HS-W31-001 through HS-W31-008 pass.
2. `cargo test --all-targets` exits 0 on a clean checkout.
3. `cargo clippy --all-targets -- -D warnings` clean.
4. `cargo fmt --check` clean.
5. FLAG F4 obligation: verify `mitre_attack_version = "ics-attack-v15"` covers all 7 ICS
   emitted techniques at attack.mitre.org before tagging. Update the constant if needed.
6. PR descriptions document the JSON breaking change (scalar → array).
