---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/terminal.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_terminal_tests.rs mod story_078 formalization (F-W22-BC-ANCHOR) — 2026-05-31"
  - "v1.4: DF-SIBLING-SWEEP-001 — fix stale terminal.rs line anchor: render_finding_flat 223-228 → 230-235 (fn at 230, closing at 235); verified against HEAD cfe0112a — 2026-06-01"
  - "v1.5: ADR-006 / Decision 13 §13.7 (F2 v0.3.0) — multi-tag rendering: single ID emits 'MITRE: T1036'; multi-tag emits 'MITRE: T0855, T0836' (comma-space separated); empty vec emits no MITRE line. Precondition 2 and EC-003 updated; EC-005/EC-006 added. — 2026-06-09"
  - "v1.6: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Description, Postconditions, EC-005, EC-006, and Canonical Test Vectors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.017: Default Rendering Emits MITRE: <id(s)> Only (No Em-Dash)

<!--
  PREVIOUS VERSION SUMMARY (v1.4 -> v1.5):
  Title updated: "<id>" -> "<id(s)>" to reflect multi-ID case
  Precondition 2: "mitre_technique set" -> "mitre_techniques non-empty"
  Postcondition 1: single ID unchanged; multi-ID format: "MITRE: T0855, T0836"
  EC-003: "mitre_technique=None" -> "mitre_techniques=vec![]"
  EC-005 added: Finding with 2 techniques renders comma-space-separated IDs
  EC-006 added: Finding with empty mitre_techniques -> no MITRE line
  Canonical vectors updated with multi-tag example
-->

## Description

When `TerminalReporter.show_mitre_grouping = false` (the default), each finding's MITRE line
reads `    MITRE: <id(s)>\n` -- all technique IDs from `mitre_techniques` joined with `", "`
(comma-space), with no em-dash separator, no technique names, and no `(unknown)` labels.
For singleton vecs the output is identical to the pre-F2 single-technique format (`MITRE: T1036`).
For multi-element vecs the output is `MITRE: T1692.001, T0836`. Findings with empty
`mitre_techniques` produce no MITRE line. Findings are rendered in their original emission
order with no tactic bucketing or sorting.

## Preconditions

1. `TerminalReporter.show_mitre_grouping = false` (default).
2. A finding has a non-empty `mitre_techniques` vec.

## Postconditions

1. The MITRE line reads: `    MITRE: <id1>, <id2>, ...\n` where IDs are joined with `", "`
   in the order they appear in `mitre_techniques`. For singleton vecs: `    MITRE: T1036\n`.
   For two-element vecs: `    MITRE: T1692.001, T0836\n`.
2. No em-dash, no technique names, no `(unknown)` labels.
3. No `## TacticName` or `## Uncategorized` headers in the FINDINGS section.
4. Findings render in their original slice order.
5. Findings with empty `mitre_techniques` produce no MITRE line (no blank line either).

## Invariants

1. Default mode uses `render_finding_flat` (terminal.rs:230-235) which iterates
   `mitre_techniques` and joins IDs with `", "`. If empty, skips the MITRE line entirely.
2. `render_finding_flat` never calls `technique_name()` or `technique_tactic()`.
3. This mode is the "no --mitre flag" case; grouping requires the `--mitre` CLI flag.
4. The join separator is `", "` (comma followed by single space). No trailing separator.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding with mitre_techniques=["T1036"] (single known ID) | "MITRE: T1036\n" (no name) — identical to pre-F2 output |
| EC-002 | Finding with mitre_techniques=["T9999"] (unknown ID) | "MITRE: T9999\n" (no "(unknown)" label) |
| EC-003 | Finding with mitre_techniques=vec![] (empty) | No MITRE line rendered for this finding |
| EC-004 | show_mitre_grouping=false, multiple findings | Rendered in emission order |
| EC-005 | Finding with mitre_techniques=["T1692.001","T0836"] (multi-tag, Modbus register write; T1692.001 = v19 ICS sub-technique, successor to revoked T0855) | "MITRE: T1692.001, T0836\n" (both IDs, comma-space separated) |
| EC-006 | Finding with mitre_techniques=["T0806","T1692.001"] (burst finding) | "MITRE: T0806, T1692.001\n" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding with mitre_techniques=["T1036"], show_mitre_grouping=false | Output contains "MITRE: T1036" without em-dash | happy-path (singleton — backward-compat) |
| Finding with mitre_techniques=["T1692.001","T0836"], show_mitre_grouping=false | Output contains "MITRE: T1692.001, T0836" | happy-path (multi-tag) |
| Finding with mitre_techniques=[], show_mitre_grouping=false | No "MITRE:" line in output for that finding | edge-case (empty) |
| Findings rendered flat | No "## Defense Evasion" header present | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Default mode uses bare MITRE: <id> format | unit: default_rendering_unchanged_when_mitre_flag_off |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the default rendering contract is the baseline terminal output format; the expanded MITRE line format is an opt-in mode |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078 |
| Origin BC | BC-RPT-017 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.016 -- contrasts with (grouped mode adds em-dash + name)
- BC-2.11.013 -- contrasts with (grouped mode adds tactic headers; default mode has none)

## Architecture Anchors

- `src/reporter/terminal.rs:230-235` -- render_finding_flat (default path)
- `tests/reporter_terminal_tests.rs` -- mod story_078 :: test_BC_2_11_017_default_mode_bare_mitre_id

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:230-235` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: default_rendering_unchanged_when_mitre_flag_off

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
