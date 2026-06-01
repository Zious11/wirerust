---
document_type: behavioral-contract
level: L3
version: "1.4"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.017: Default Rendering Emits MITRE: <id> Only (No Em-Dash)

## Description

When `TerminalReporter.show_mitre_grouping = false` (the default), each finding's MITRE line
reads `    MITRE: <id>\n` -- a bare technique ID with no em-dash separator, no technique name,
and no `(unknown)` label. Findings are rendered in their original emission order with no tactic
bucketing or sorting. The `## Tactic` section headers are absent from the output.

## Preconditions

1. `TerminalReporter.show_mitre_grouping = false` (default).
2. A finding has `mitre_technique` set to some technique ID string.

## Postconditions

1. The MITRE line reads: `    MITRE: <id>\n`.
2. No em-dash, no technique name, no `(unknown)` label.
3. No `## TacticName` or `## Uncategorized` headers in the FINDINGS section.
4. Findings render in their original slice order.

## Invariants

1. Default mode uses `render_finding_flat` (terminal.rs:230-235) which always renders
   `MITRE: <id>` only.
2. `render_finding_flat` never calls `technique_name()` or `technique_tactic()`.
3. This mode is the "no --mitre flag" case; grouping requires the `--mitre` CLI flag.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding with known technique ID | "MITRE: T1036\n" (no name) |
| EC-002 | Finding with unknown technique ID | "MITRE: T9999\n" (no "(unknown)" label) |
| EC-003 | Finding with mitre_technique=None | No MITRE line |
| EC-004 | show_mitre_grouping=false, multiple findings | Rendered in emission order |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding with mitre_technique="T1036", show_mitre_grouping=false | Output contains "MITRE: T1036" without em-dash | happy-path |
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
