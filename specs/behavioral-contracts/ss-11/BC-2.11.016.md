---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.016: MITRE Grouping Expands Per-Finding Line with Em-Dash and Name

## Description

In MITRE-grouped rendering, each finding's MITRE line is expanded from the bare `MITRE: <id>`
format to `MITRE: <id> -- <TechniqueName>` for technique IDs that are present in the catalog.
The separator is U+2014 (EM DASH), NOT the ASCII hyphen-minus sequence `--`. Unknown IDs
render as `MITRE: <id> (unknown)`.

## Preconditions

1. `TerminalReporter.show_mitre_grouping = true`.
2. A finding has a `mitre_technique` set to a technique ID in the catalog.

## Postconditions

1. The MITRE line reads: `    MITRE: <id> \u{2014} <name>\n` where `<name>` is the string
   returned by `technique_name(id)`.
2. The separator character is U+2014 (EM DASH), not two hyphens `--`.
3. For unknown IDs, the line reads: `    MITRE: <id> (unknown)\n`.
4. For `mitre_technique = None`, no MITRE line is rendered (not even "(unknown)").

## Invariants

1. The em-dash character U+2014 is hardcoded at terminal.rs:241 as `\u{2014}`.
2. `technique_name(id)` is the authoritative source for the name string.
3. The MITRE line is rendered by `render_finding_grouped`, called only in grouping mode.
4. In default (flat) mode, `render_finding_flat` renders `MITRE: <id>` only (no expansion).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Known ID T1036 | "MITRE: T1036 \u{2014} Masquerading\n" (or whatever technique_name returns) |
| EC-002 | Unknown ID T9999 | "MITRE: T9999 (unknown)\n" |
| EC-003 | mitre_technique = None | No MITRE line |
| EC-004 | Downstream grep for ASCII "--" separator | Will miss em-dash; must grep for U+2014 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Finding with mitre_technique="T1036" in grouped mode | MITRE line contains U+2014 and technique name | happy-path |
| Finding with mitre_technique="T9999" (unknown) | "MITRE: T9999 (unknown)" | happy-path |
| Finding with mitre_technique=None in grouped mode | No MITRE line in output | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Em-dash and name in grouped MITRE line | unit: mitre_grouping_expands_per_finding_line_with_technique_name |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- the expanded MITRE line format with em-dash and technique name is a documented output encoding contract that downstream grep-based pipelines must account for |
| L2 Domain Invariants | INV-9 (MITRE Technique ID Format -- the expansion uses the catalog's name mapping) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | S-TBD |
| Origin BC | BC-RPT-016 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.017 -- contrasts with (default mode does NOT expand; BC-016 is grouped-mode only)
- BC-2.11.015 -- composes with (unknown-ID handling uses "(unknown)" not em-dash)
- BC-2.10.005 -- depends on (technique_name provides the expansion string)

## Architecture Anchors

- `src/reporter/terminal.rs:239-244` -- render_finding_grouped MITRE line expansion
- `src/reporter/terminal.rs:241` -- `\u{2014}` em-dash literal
- `tests/reporter_tests.rs` -- mitre_grouping_expands_per_finding_line_with_technique_name

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:239-244` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: mitre_grouping_expands_per_finding_line_with_technique_name
- **guard clause**: `match technique_name(id)` branch at line 240

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed. The U+2014 em-dash character is a hidden output-encoding contract;
downstream pipelines that grep for ASCII `--` will silently miss these lines.
