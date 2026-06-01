---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.3: VP-016 proof-method cell unit→integration to match VP-016 frontmatter + VP-INDEX (Wave-21 wave-level consistency lens; SS-11 reporter VP family harmonization — sibling of VP-017 fix in 86113c2; DF-SIBLING-SWEEP-001)"
  - "v1.4: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_terminal_tests.rs mod story_078 formalization (F-W22-BC-ANCHOR) — 2026-05-31"
  - "v1.5: DF-SIBLING-SWEEP-001 — fix stale terminal.rs line anchors: render_finding_grouped 237-245 → 244-252 (fn at 244, None-arm at 249), Uncategorized bucket 291-296 → 298-303 (if let Some at 298); outer Path range 237-296 → 244-303; verified against HEAD cfe0112a — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.015: No-Technique or Unknown-ID Findings Land in Uncategorized

## Description

In MITRE-grouped rendering, findings with `mitre_technique = None` AND findings with a
`mitre_technique` that is not present in the technique catalog (i.e., `technique_tactic(id)`
returns None) both land in the `Uncategorized` bucket. Unknown-ID findings render with an
`(unknown)` label in the MITRE line. This ensures no finding is silently dropped from output
due to an unrecognized technique ID.

## Preconditions

1. `TerminalReporter.show_mitre_grouping = true`.
2. At least one finding has `mitre_technique = None` or has an ID not in the catalog.

## Postconditions

1. Findings with `mitre_technique = None` appear under `## Uncategorized`.
2. Findings with an unrecognized technique ID appear under `## Uncategorized`.
3. For findings with an unrecognized ID, the MITRE line reads `MITRE: <id> (unknown)`.
4. For findings with `mitre_technique = None`, no MITRE line is rendered.
5. The `Uncategorized` bucket is rendered AFTER all named tactic buckets.

## Invariants

1. Bucket key `None` (the `Option<MitreTactic>` None variant) collects both:
   - `f.mitre_technique.is_none()` cases (technique_tactic was never called)
   - Cases where `technique_tactic(id)` returns None (ID not in catalog)
2. Both produce the same None bucket key because both go through
   `f.mitre_technique.as_deref().and_then(technique_tactic)` at terminal.rs:265.
3. Known and unknown IDs are kept in separate buckets only when they map to different
   tactics -- but unknown IDs always produce None from technique_tactic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding with mitre_technique=None | Under ## Uncategorized; no MITRE line |
| EC-002 | Finding with id="T9999" (not in catalog) | Under ## Uncategorized; "MITRE: T9999 (unknown)" |
| EC-003 | Mix of None and unknown-ID findings | Both under same ## Uncategorized |
| EC-004 | Known ID in same output | Known ID in its tactic section; unknowns in Uncategorized |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| findings=[{mitre=None}, {mitre="T9999"}] | "## Uncategorized" with both findings | happy-path |
| findings=[{mitre="T1036" (known)}, {mitre=None}] | "## Defense Evasion" + "## Uncategorized" | happy-path |
| all findings with None technique | Only "## Uncategorized" section | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | None and unknown-ID bucket under Uncategorized | integration: mitre_grouping_buckets_none_and_unknown_under_uncategorized |
| VP-016 | Known and unknown IDs in separate sections | integration: mitre_grouping_keeps_known_and_unknown_ids_in_separate_buckets |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the Uncategorized bucket prevents finding loss; all findings must appear in output regardless of technique catalog coverage |
| L2 Domain Invariants | INV-9 (MITRE Technique ID Format -- IDs not in catalog are not dropped but rendered as unknown) |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078 |
| Origin BC | BC-RPT-015 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.013 -- composes with (Uncategorized is the last bucket in the tactic iteration)
- BC-2.11.016 -- composes with (known IDs render with em-dash; unknown IDs render with "(unknown)")
- BC-2.10.006 -- depends on (technique_tactic returns None for unknown IDs)

## Architecture Anchors

- `src/reporter/terminal.rs:244-252` -- render_finding_grouped (fn decl at 244, closing brace at 252); None-arm `(unknown)` label at :249
- `src/reporter/terminal.rs:298-303` -- Uncategorized bucket rendering (`if let Some(items) = buckets.get(&None)` at :298)
- `tests/reporter_terminal_tests.rs` -- mod story_078 :: test_BC_2_11_015_none_technique_uncategorized

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:244-303` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: mitre_grouping_buckets_none_and_unknown_under_uncategorized, mitre_grouping_keeps_known_and_unknown_ids_in_separate_buckets

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
