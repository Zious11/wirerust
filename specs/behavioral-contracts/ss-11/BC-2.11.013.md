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

# BC-2.11.013: MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last

## Description

When `TerminalReporter.show_mitre_grouping = true`, the FINDINGS section is reorganized into
per-tactic buckets. Tactic headers are emitted in the order returned by
`all_tactics_in_report_order()` (MITRE Enterprise kill-chain order first, then ICS tactics).
An `Uncategorized` bucket is always appended last, collecting findings with no technique ID
or with an ID not present in the technique catalog.

## Preconditions

1. `TerminalReporter.show_mitre_grouping = true`.
2. `findings` is non-empty.
3. At least some findings have `mitre_technique` set to a recognized ID; others may have
   unknown IDs or None.

## Postconditions

1. Tactic section headers appear as `  ## <TacticName>\n` in the output.
2. Tactic sections appear in the order produced by `all_tactics_in_report_order()`.
3. A section is only emitted when at least one finding belongs to that tactic.
4. `  ## Uncategorized\n` is the last section, containing findings where
   `technique_tactic(id)` returns None or where `mitre_technique` is None.

## Invariants

1. `all_tactics_in_report_order()` is the authoritative iteration order; the terminal
   reporter iterates it directly (terminal.rs:283).
2. Tactic buckets use `HashMap<Option<MitreTactic>, Vec<...>>`; None maps to Uncategorized.
3. A tactic section is SKIPPED if no findings belong to it. This prevents empty section
   headers in the output.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All findings have the same tactic | Only that tactic's section + possibly Uncategorized |
| EC-002 | All findings have unknown technique IDs | Only ## Uncategorized |
| EC-003 | All findings have None technique | Only ## Uncategorized |
| EC-004 | Mix: known + unknown + None technique IDs | Named sections + ## Uncategorized last |
| EC-005 | Empty findings slice | No FINDINGS section rendered at all (not grouping-mode specific) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Findings with T1036 (DefenseEvasion) and no-technique | "## Defense Evasion" section then "## Uncategorized" | happy-path |
| Finding with unknown ID "T9999" and None-technique | "## Uncategorized" only | happy-path |
| Findings spanning 3 tactics | 3 named sections in kill-chain order | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | Tactic headers in canonical order | unit: mitre_grouping_emits_tactic_headers_in_canonical_order |
| VP-016 | None/unknown bucket under Uncategorized | unit: mitre_grouping_buckets_none_and_unknown_under_uncategorized |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the MITRE tactic grouping rendering is a key differentiator in the terminal output mode that organizes forensic findings by attack phase |
| L2 Domain Invariants | INV-9 (MITRE Technique ID Format -- technique IDs that fail catalog lookup go to Uncategorized) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078 |
| Origin BC | BC-RPT-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.014 -- composes with (within-bucket sort order)
- BC-2.11.015 -- composes with (None/unknown bucket definition)
- BC-2.11.016 -- composes with (per-finding line format in grouped mode)
- BC-2.10.003 -- depends on (all_tactics_in_report_order provides the canonical iteration)

## Architecture Anchors

- `src/reporter/terminal.rs:253-297` -- render_findings_grouped implementation
- `src/reporter/terminal.rs:283` -- `for tactic in all_tactics_in_report_order()`
- `tests/reporter_tests.rs` -- mitre_grouping_emits_tactic_headers_in_canonical_order

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:253-297` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: mitre_grouping_emits_tactic_headers_in_canonical_order, mitre_grouping_buckets_none_and_unknown_under_uncategorized

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes (HashMap iteration order compensated by tactic-order iteration) |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
