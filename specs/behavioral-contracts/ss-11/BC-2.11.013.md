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
  - "v1.3: VP-016 proof-method cell unit→integration to match VP-016 frontmatter + VP-INDEX (Wave-21 wave-level consistency lens; SS-11 reporter VP family harmonization — sibling of VP-017 fix in 86113c2; DF-SIBLING-SWEEP-001)"
  - "v1.4: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_terminal_tests.rs mod story_078 formalization (F-W22-BC-ANCHOR) — 2026-05-31"
  - "v1.5: DF-SIBLING-SWEEP-001 — fix stale terminal.rs line anchors: render_findings_grouped range 253-297 → 260-304 (fn starts at 260, closes at 304), tactic loop :283 → :290; verified against HEAD cfe0112a — 2026-06-01"
  - "v1.6: ADR-006 / Decision 13 §13.7 (F2 v0.3.0) — tactic-grouping uses mitre_techniques[0] as primary bucket key for multi-tag findings; empty vec -> Uncategorized (replaces None path); Precondition 3 updated; Invariant 2 updated; EC-006 added (multi-tag primary-tactic rule). — 2026-06-09"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.013: MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last

<!--
  PREVIOUS VERSION SUMMARY (v1.5 -> v1.6):
  Precondition 3: "mitre_technique set to a recognized ID" -> "mitre_techniques non-empty with recognized IDs"
  Postcondition 4: "mitre_technique is None" -> "mitre_techniques is empty"
  Invariant 2: bucket key computed from mitre_techniques[0]; empty vec -> None bucket (Uncategorized)
  EC-006 added: multi-tag finding groups under first technique's tactic
  Canonical vectors: updated to use vec!["T1036"] notation for clarity
-->

## Description

When `TerminalReporter.show_mitre_grouping = true`, the FINDINGS section is reorganized into
per-tactic buckets. Tactic headers are emitted in the order returned by
`all_tactics_in_report_order()` (MITRE Enterprise kill-chain order first, then ICS tactics).
An `Uncategorized` bucket is always appended last, collecting findings with an empty
`mitre_techniques` vec or with all IDs unknown to the technique catalog.

For multi-tag findings (e.g., `mitre_techniques: ["T0855","T0836"]`), the grouping tactic is
determined by `mitre_techniques[0]` (the first element). This is the primary attribution
approximation per ADR-006 §13.7; full multi-tactic display is a future enhancement.

## Preconditions

1. `TerminalReporter.show_mitre_grouping = true`.
2. `findings` is non-empty.
3. At least some findings have a non-empty `mitre_techniques` vec containing a recognized ID;
   others may have empty vecs or unknown IDs.

## Postconditions

1. Tactic section headers appear as `  ## <TacticName>\n` in the output.
2. Tactic sections appear in the order produced by `all_tactics_in_report_order()`.
3. A section is only emitted when at least one finding belongs to that tactic.
4. `  ## Uncategorized\n` is the last section, containing findings where
   `mitre_techniques` is empty OR where `technique_tactic(mitre_techniques[0])` returns None.

## Invariants

1. `all_tactics_in_report_order()` is the authoritative iteration order; the terminal
   reporter iterates it directly (terminal.rs:290).
2. Tactic buckets use `HashMap<Option<MitreTactic>, Vec<...>>`; the bucket key is derived as:
   - If `mitre_techniques` is empty: key = `None` (Uncategorized)
   - If `mitre_techniques` is non-empty: key = `technique_tactic(mitre_techniques[0])`, which
     may itself be `None` for unknown IDs (also Uncategorized)
   For multi-tag findings, only the FIRST technique (`mitre_techniques[0]`) determines the
   bucket. Secondary techniques are visible in the finding's inline display but do not
   create additional bucket memberships.
   **Bucketing is deterministic because `mitre_techniques[0]` is the canonical-construction-order
   primary technique.** Findings are constructed with a fixed `vec!["T0855","T0836"]` order
   (per ADR-006 §13 canonical ordering); reporters rely on this construction order, not
   on runtime sorting. The reporter does NOT sort `mitre_techniques` before bucketing.
3. A tactic section is SKIPPED if no findings belong to it. This prevents empty section
   headers in the output.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All findings have the same tactic (via [0]) | Only that tactic's section + possibly Uncategorized |
| EC-002 | All findings have unknown technique IDs in [0] | Only ## Uncategorized |
| EC-003 | All findings have empty mitre_techniques vec | Only ## Uncategorized |
| EC-004 | Mix: known + unknown + empty mitre_techniques | Named sections + ## Uncategorized last |
| EC-005 | Empty findings slice | No FINDINGS section rendered at all (not grouping-mode specific) |
| EC-006 | Finding with mitre_techniques=["T0855","T0836"] (multi-tag) | Groups under MitreTactic::IcsImpairProcessControl (T0855's tactic); T0836 visible in inline rendering but does not create a second bucket |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Findings with mitre_techniques=["T1036"] (DefenseEvasion) and empty-vec finding | "## Defense Evasion" section then "## Uncategorized" | happy-path |
| Finding with mitre_techniques=["T9999"] and empty-vec finding | "## Uncategorized" only | happy-path |
| Findings spanning 3 tactics (by first element) | 3 named sections in kill-chain order | happy-path |
| Finding with mitre_techniques=["T0855","T0836"] | Groups under ICS Impair Process Control section (T0855 tactic) | happy-path (multi-tag) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | Tactic headers in canonical order | integration: mitre_grouping_emits_tactic_headers_in_canonical_order |
| VP-016 | None/unknown bucket under Uncategorized | integration: mitre_grouping_buckets_none_and_unknown_under_uncategorized |

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

- `src/reporter/terminal.rs:260-304` -- render_findings_grouped implementation
- `src/reporter/terminal.rs:290` -- `for tactic in all_tactics_in_report_order()`
- `tests/reporter_terminal_tests.rs` -- mod story_078 :: test_BC_2_11_013_tactic_headers_in_canonical_order

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:260-304` |
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
