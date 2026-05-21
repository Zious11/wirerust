---
document_type: story
story_id: "STORY-078"
epic_id: "E-8"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.014.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.015.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.016.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.018.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.019.md
  - .factory/specs/prd.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-077]
blocks: []
behavioral_contracts:
  - BC-2.11.013
  - BC-2.11.014
  - BC-2.11.015
  - BC-2.11.016
  - BC-2.11.017
  - BC-2.11.018
  - BC-2.11.019
verification_properties: [VP-016]
priority: "P0"
wave: 22
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-verify
---

# STORY-078: TerminalReporter — MITRE Grouping, Section Order, and Colorization

## Narrative
- **As a** forensic analyst using `--mitre` mode
- **I want** terminal output to group findings by MITRE tactic in kill-chain order (Uncategorized last), sort within each tactic by verdict then confidence, show em-dash-separated technique names for known IDs, and display the correct section ordering across all modes
- **So that** I can immediately see which attack phases are represented in the capture and which findings are highest-severity within each tactic

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.11.013 | MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last |
| BC-2.11.014 | Within Tactic Bucket: Sort by Verdict, Confidence, Emission Order |
| BC-2.11.015 | No-Technique or Unknown-ID Findings Land in Uncategorized |
| BC-2.11.016 | MITRE Grouping Expands Per-Finding Line with Em-Dash and Name |
| BC-2.11.017 | Default Rendering Emits MITRE: <id> Only (No Em-Dash) |
| BC-2.11.018 | TerminalReporter Colorization: Likely/High=Red Bold, etc. |
| BC-2.11.019 | TerminalReporter Renders Sections in Correct Order |

## Acceptance Criteria

### AC-001 (traces to BC-2.11.013 postcondition 2)
When `show_mitre_grouping = true`, tactic section headers appear in the order returned by `all_tactics_in_report_order()`. A tactic section is emitted ONLY when at least one finding belongs to it.
- **Test:** `test_BC_2_11_013_tactic_headers_in_canonical_order()`

### AC-002 (traces to BC-2.11.013 postcondition 4)
The `## Uncategorized` section is always the LAST section in the grouped output.
- **Test:** `test_BC_2_11_013_uncategorized_last()`

### AC-003 (traces to BC-2.11.014 postcondition 1)
Within a MITRE tactic bucket, findings with lower verdict rank (Likely=0) appear before those with higher rank (Inconclusive=1, Unlikely=2).
- **Test:** `test_BC_2_11_014_sort_by_verdict_within_bucket()`

### AC-004 (traces to BC-2.11.014 postcondition 2)
Among findings with the same verdict, findings with lower confidence rank (High=0) appear before those with higher rank (Medium=1, Low=2).
- **Test:** `test_BC_2_11_014_sort_by_confidence_within_same_verdict()`

### AC-005 (traces to BC-2.11.014 postcondition 3)
Among findings with the same verdict and confidence, original emission order (slice index) is preserved (stable sort).
- **Test:** `test_BC_2_11_014_stable_emission_order_on_tie()`

### AC-006 (traces to BC-2.11.015 postcondition 1)
Findings with `mitre_technique = None` appear under `## Uncategorized`.
- **Test:** `test_BC_2_11_015_none_technique_uncategorized()`

### AC-007 (traces to BC-2.11.015 postcondition 2)
Findings with an unrecognized technique ID (not in catalog, e.g., "T9999") appear under `## Uncategorized` with the MITRE line reading `MITRE: T9999 (unknown)`.
- **Test:** `test_BC_2_11_015_unknown_id_uncategorized_with_label()`

### AC-008 (traces to BC-2.11.016 postcondition 1)
When `show_mitre_grouping = true` and a finding has a known technique ID (e.g., "T1036"), the MITRE line reads `MITRE: T1036 \u{2014} <TechniqueName>` (U+2014 em-dash, not ASCII `--`).
- **Test:** `test_BC_2_11_016_known_id_em_dash_and_name()`

### AC-009 (traces to BC-2.11.016 invariant 1)
The separator character is U+2014 (EM DASH). Grep for ASCII `--` will NOT match this line.
- **Test:** `test_BC_2_11_016_separator_is_em_dash_not_ascii_hyphen()`

### AC-010 (traces to BC-2.11.017 postcondition 1)
When `show_mitre_grouping = false` (default), a finding with `mitre_technique = "T1036"` produces the MITRE line `MITRE: T1036` with no em-dash, no technique name, and no `(unknown)` label.
- **Test:** `test_BC_2_11_017_default_mode_bare_mitre_id()`

### AC-011 (traces to BC-2.11.017 postcondition 3)
In default mode, no `## TacticName` or `## Uncategorized` section headers appear in the output.
- **Test:** `test_BC_2_11_017_default_mode_no_tactic_headers()`

### AC-012 (traces to BC-2.11.018 postcondition 5)
When `use_color = false`, no ANSI escape codes appear in the rendered output for any verdict/confidence combination.
- **Test:** `test_BC_2_11_018_no_ansi_codes_when_color_disabled()`

### AC-013 (traces to BC-2.11.019 postcondition 1)
The `WIRERUST TRIAGE REPORT` header section is always the first section in the output.
- **Test:** `test_BC_2_11_019_header_is_first_section()`

### AC-014 (traces to BC-2.11.019 postcondition 4)
The FINDINGS section appears only when `findings` is non-empty; it is entirely absent when `findings.is_empty()`.
- **Test:** `test_BC_2_11_019_findings_section_absent_when_empty()`

### AC-015 (traces to BC-2.11.019 postcondition 5)
ANALYZER sections appear last, one per `AnalysisSummary` element, in slice order.
- **Test:** `test_BC_2_11_019_analyzer_sections_last_in_slice_order()`

### AC-016 (traces to BC-2.11.019 invariant 3)
SERVICES section is absent entirely when `service_counts()` returns an empty map.
- **Test:** `test_BC_2_11_019_services_section_absent_when_empty()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| render_findings_grouped | src/reporter/terminal.rs:253-297 | pure |
| render_finding_grouped | src/reporter/terminal.rs:237-245 | pure |
| render_finding_flat | src/reporter/terminal.rs:223-228 | pure |
| TerminalReporter::render (section order) | src/reporter/terminal.rs:83-178 | pure |
| verdict_rank / confidence_rank | src/reporter/terminal.rs:262-275 | pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | All findings have same tactic | That tactic + (possibly) Uncategorized |
| EC-002 | All findings have None technique | Only ## Uncategorized |
| EC-003 | Findings spanning 3 tactics | 3 named sections in kill-chain order |
| EC-004 | All same verdict/confidence | Emission order preserved (stable sort) |
| EC-005 | Unknown ID "T9999" | Under Uncategorized; "(unknown)" label |
| EC-006 | mitre_technique = None in grouped mode | No MITRE line for that finding |
| EC-007 | show_mitre_grouping = false | Bare "MITRE: T1036", no em-dash |
| EC-008 | No findings, no services | Header + PROTOCOLS only |
| EC-009 | show_hosts_breakdown = true | HOSTS section between header and PROTOCOLS |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reporter/terminal.rs | pure | String construction only; HashMap ordering compensated by all_tactics_in_report_order() iteration |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| src/reporter/terminal.rs (grouped rendering, section order) | ~4,000 |
| BC files (7 BCs) | ~7,000 |
| tests/reporter_tests.rs (grouping tests) | ~2,000 |
| Tool outputs overhead | ~500 |
| **Total** | **~16,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~8.3%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-016 (test-writer)
2. [ ] Verify all tests fail at Red Gate
3. [ ] Verify `src/reporter/terminal.rs` already satisfies all ACs (brownfield confirm)
4. [ ] Confirm `all_tactics_in_report_order()` is iterated at terminal.rs:283 in grouped rendering
5. [ ] Confirm sort key is `(verdict_rank, confidence_rank, original_index)` at terminal.rs:262-280
6. [ ] Confirm bucket key for None/unknown is `Option<MitreTactic>::None`
7. [ ] Confirm em-dash is U+2014 (`\u{2014}`) at terminal.rs:241
8. [ ] Confirm `render_finding_flat` at terminal.rs:223-228 emits bare `MITRE: <id>` only
9. [ ] Confirm section order in render(): header, PROTOCOLS, SERVICES (conditional), FINDINGS (conditional), ANALYZERs
10. [ ] Run `cargo test --all-targets` to confirm green

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-076 | JsonReporter schema; skipped_packets always present | Reporter trait; pure renderers | N/A |
| STORY-077 | escape_for_terminal; C0/C1/DEL/backslash escaping; skipped_packets conditional | escape_for_terminal is TerminalReporter-only | U+00A0 (NBSP) must NOT be escaped |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Tactic sections emitted by iterating `all_tactics_in_report_order()` (not HashMap key order) | BC-2.11.013 invariant 1 | Code review: for loop at terminal.rs:283 |
| Within-bucket sort is stable: `sort_by_key` (Rust std guarantee) | BC-2.11.014 invariant 3 | Use `sort_by_key`, not `sort_unstable_by_key` |
| `Uncategorized` bucket key is `Option<MitreTactic>::None` | BC-2.11.015 invariant 1 | Code review of bucket map type at terminal.rs:258 |
| Em-dash separator is U+2014, not ASCII `--` | BC-2.11.016 invariant 1 | Code review of terminal.rs:241; test for em-dash presence |
| `render_finding_flat` NEVER calls `technique_name()` or `technique_tactic()` | BC-2.11.017 invariant 2 | Code review of render_finding_flat at terminal.rs:223-228 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| owo_colors | (per Cargo.lock) | ANSI colorization for Likely/High=red bold; Inconclusive=cyan; Unlikely=dimmed |
| std::collections::HashMap | stdlib | Tactic bucket map |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/reporter/terminal.rs | verify/modify | render_findings_grouped (253-297), render_finding_grouped (237-245), render_finding_flat (223-228), render (83-178) |
| src/mitre.rs | verify | `all_tactics_in_report_order()`, `technique_tactic()`, `technique_name()` |
| tests/reporter_tests.rs | create or modify | AC-001 through AC-016 tests |
