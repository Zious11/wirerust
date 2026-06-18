---
document_type: behavioral-contract
level: L3
version: "1.14"
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
  - "v1.8: PG-ARP-F2-007 — fix stale terminal.rs line anchors shifted by F2 multi-tag additions (STORY-100): render_findings_grouped fn :260-304 → :272-323 (fn decl at 272, closing at 323); tactic loop :290 → :309; verified against current HEAD — 2026-06-13"
  - "v1.6: ADR-006 / Decision 13 §13.7 (F2 v0.3.0) — tactic-grouping uses mitre_techniques[0] as primary bucket key for multi-tag findings; empty vec -> Uncategorized (replaces None path); Precondition 3 updated; Invariant 2 updated; EC-006 added (multi-tag primary-tactic rule). — 2026-06-09"
  - "v1.7: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Description, Invariant 2, EC-006, and Canonical Test Vectors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.9: issue-#259 F2 integrate (v0.8.0 collapse feature) — add Invariant 4 and EC-007 explicitly scoping collapse interaction: when show_mitre_grouping=true, the collapse pass (BC-2.11.025) is NOT applied regardless of collapse_findings field value. Grouped mode renders each finding individually. Collapse within grouped/--mitre mode is DEFERRED to STORY-119 (future cycle). Cross-references BC-2.11.025/028/029. ADR-0003 (display-layer aggregation subsection) cited. — 2026-06-17"
  - "v1.10 2026-06-17: F2 adversarial pass-2 — strengthen EC-007: assert structural suffix-free guarantee via path-(b) wrapper (render_finding_prefix unchanged; grouped path structurally unable to emit suffix) (F-A03)"
  - "v1.11 2026-06-17: F2 adversarial pass-4 — F-F2-A01: EC-007 STRUCTURAL guarantee converted to OBSERVABLE GUARANTEE form; remove call-graph prescription 'render_finding_prefix itself is UNCHANGED; the collapse-aware flat wrapper that appends suffixes is never called from the grouped path'"
  - "v1.12 2026-06-17: issue-#62 F2 BC re-anchor — replace show_mitre_grouping bool references with FindingsRender enum: Precondition 1 and Description 'show_mitre_grouping = true' → 'render = FindingsRender::Grouped'; Invariant 4 scoping boundary reworded to enum form; EC-007 condition reworded. Rationale: illegal-state elimination (grouping=true && collapse=true unrepresentable as FindingsRender::Grouped). No behavioral change."
  - "v1.13 2026-06-18: F5 post-merge re-anchor to develop a4263c7 (terminal.rs line-anchor drift fix; no normative change) — render_findings_grouped fn shifted: :272-323 → :432-483; tactic loop :309 → :469; Architecture Anchor + Source Evidence path updated."
  - "v1.14 2026-06-18: STORY-119 spec-evolution — grouped collapse is NOW SUPPORTED. Precondition 1 updated to struct form (render.grouping == Grouping::Grouped). Invariant 4 rewritten: removes deferral language ('DEFERRED to STORY-119'); grouped collapse is active when render == {grouping: Grouping::Grouped, collapse: Collapse::Collapsed} (BC-2.11.031); the no-collapse guarantee is scoped to {Grouping::Grouped, Collapse::Expanded} only. Related BCs updated to reference BC-2.11.030–034. EC-007 updated to reflect both grouped-expanded (no suffix) and grouped-collapsed ({Grouped,Collapsed}: suffix applies per-bucket per BC-2.11.031) paths. Description updated to struct vocabulary."
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

When `TerminalReporter.render.grouping == Grouping::Grouped` (i.e., `render` is either
`FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` or
`FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`), the FINDINGS
section is reorganized into per-tactic buckets. Tactic headers are emitted in the order returned
by `all_tactics_in_report_order()` (MITRE Enterprise kill-chain order first, then ICS tactics).
An `Uncategorized` bucket is always appended last, collecting findings with an empty
`mitre_techniques` vec or with all IDs unknown to the technique catalog.

For multi-tag findings (e.g., `mitre_techniques: ["T1692.001","T0836"]`), the grouping tactic is
determined by `mitre_techniques[0]` (the first element). This is the primary attribution
approximation per ADR-006 §13.7; full multi-tactic display is a future enhancement.

When `render.collapse == Collapse::Collapsed` (`{Grouped, Collapsed}` — the new default for
`--mitre` since STORY-119), each tactic bucket additionally applies a per-bucket collapse pass
producing `(xN)` suffix annotations for N≥2 groups (BC-2.11.031) and K=3 evidence sampling
(BC-2.11.032). When `render.collapse == Collapse::Expanded` (`{Grouped, Expanded}` —
`--mitre --no-collapse`), each finding is rendered individually with no suffix, preserving
the pre-STORY-119 `--mitre` behavior.

## Preconditions

1. `TerminalReporter.render.grouping == Grouping::Grouped` (set via `--mitre` CLI flag).
   The collapse axis may be either `Collapse::Collapsed` (`--mitre` alone, the new default
   since STORY-119) or `Collapse::Expanded` (`--mitre --no-collapse`, suffix-free).
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
   reporter iterates it directly (terminal.rs:469).
2. Tactic buckets use `HashMap<Option<MitreTactic>, Vec<...>>`; the bucket key is derived as:
   - If `mitre_techniques` is empty: key = `None` (Uncategorized)
   - If `mitre_techniques` is non-empty: key = `technique_tactic(mitre_techniques[0])`, which
     may itself be `None` for unknown IDs (also Uncategorized)
   For multi-tag findings, only the FIRST technique (`mitre_techniques[0]`) determines the
   bucket. Secondary techniques are visible in the finding's inline display but do not
   create additional bucket memberships.
   **Bucketing is deterministic because `mitre_techniques[0]` is the canonical-construction-order
   primary technique.** Findings are constructed with a fixed `vec!["T1692.001","T0836"]` order
   (per ADR-006 §13 canonical ordering); reporters rely on this construction order, not
   on runtime sorting. The reporter does NOT sort `mitre_techniques` before bucketing.
3. A tactic section is SKIPPED if no findings belong to it. This prevents empty section
   headers in the output.
4. **Collapse axis in grouped mode (STORY-119, D-110):** The collapse behavior depends on
   `render.collapse`:
   - When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }`
     (`--mitre --no-collapse`): the collapse pass is NOT applied. Each finding is rendered
     individually via one `render_finding_grouped` call, with no `(xN)` count suffix. This is
     the pre-STORY-119 `--mitre` behavior, now explicitly selected via `--no-collapse`. The
     OBSERVABLE GUARANTEE holds: no ` (xN)` suffix appears in the terminal output for any
     finding, at any input volume.
   - When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`
     (`--mitre` alone, the new default since STORY-119): a per-bucket collapse pass IS applied
     within each tactic bucket. Groups of N≥2 identical-key findings within a bucket emit a
     ` (xN)` suffix per BC-2.11.031. Singletons (N=1) within a bucket render via
     `render_finding_grouped` with no suffix. The `{Grouped, Collapsed}` mode is the new
     STORY-119 default for grouped output (BC-2.11.030). See BC-2.11.031–034 for the full
     grouped-collapse behavioral contract.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All findings have the same tactic (via [0]) | Only that tactic's section + possibly Uncategorized |
| EC-002 | All findings have unknown technique IDs in [0] | Only ## Uncategorized |
| EC-003 | All findings have empty mitre_techniques vec | Only ## Uncategorized |
| EC-004 | Mix: known + unknown + empty mitre_techniques | Named sections + ## Uncategorized last |
| EC-005 | Empty findings slice | No FINDINGS section rendered at all (not grouping-mode specific) |
| EC-006 | Finding with mitre_techniques=["T1692.001","T0836"] (multi-tag) | Groups under MitreTactic::IcsImpairProcessControl (T1692.001's tactic); T0836 visible in inline rendering but does not create a second bucket |
| EC-007a | `render = {Grouping::Grouped, Collapse::Expanded}` (`--mitre --no-collapse`) with multiple identical-key findings | Each finding is rendered individually via `render_finding_grouped`; no `(xN)` count suffix on any finding. OBSERVABLE GUARANTEE: no ` (xN)` suffix appears in the terminal output for any finding, at any input volume. This is the pre-STORY-119 `--mitre` behavior, now explicitly selected via `--no-collapse`. |
| EC-007b | `render = {Grouping::Grouped, Collapse::Collapsed}` (`--mitre` alone, the new default) with multiple identical-key findings in the same tactic bucket | Per-bucket collapse applies. N≥2 identical-key findings within a bucket collapse to one header with ` (xN)` suffix (BC-2.11.031). The suffix appears on the finding-group header line only, not on tactic bucket headers or MITRE lines. See BC-2.11.031–034 for the complete grouped-collapse contract. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Findings with mitre_techniques=["T1036"] (DefenseEvasion) and empty-vec finding | "## Defense Evasion" section then "## Uncategorized" | happy-path |
| Finding with mitre_techniques=["T9999"] and empty-vec finding | "## Uncategorized" only | happy-path |
| Findings spanning 3 tactics (by first element) | 3 named sections in kill-chain order | happy-path |
| Finding with mitre_techniques=["T1692.001","T0836"] | Groups under ICS Impair Process Control section (T1692.001 tactic) | happy-path (multi-tag) |

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
- BC-2.11.025 -- composes with (same CollapseKey four-tuple used in grouped-collapse per-bucket pass; BC-025 governs the flat-mode invocation; grouped-mode invocation governed by BC-2.11.031)
- BC-2.11.028 -- composes with (--no-collapse opt-out now has dual scope: suppresses collapse in both flat AND grouped modes; {Grouped, Expanded} is the explicit suffix-free grouped mode)
- BC-2.11.030 -- depends on (CLI mapping: --mitre alone → {Grouped, Collapsed}; --mitre --no-collapse → {Grouped, Expanded})
- BC-2.11.031 -- composes with (per-bucket count suffix rule when render={Grouped, Collapsed})
- BC-2.11.032 -- composes with (per-bucket evidence sampling when render={Grouped, Collapsed})
- BC-2.11.033 -- composes with (tactic-bucket ordering invariant under grouped-collapse)
- BC-2.11.034 -- composes with (MITRE line format for collapsed groups within tactic buckets)
- BC-2.11.029 -- composes with (JSON/CSV raw-stream invariant; grouped mode does not affect JSON/CSV unmodified-slice guarantee)

## Architecture Anchors

- `src/reporter/terminal.rs:432-483` -- render_findings_grouped implementation
- `src/reporter/terminal.rs:469` -- `for tactic in all_tactics_in_report_order()`
- `tests/reporter_terminal_tests.rs` -- mod story_078 :: test_BC_2_11_013_tactic_headers_in_canonical_order

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:432-483` |
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
