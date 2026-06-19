---
document_type: story
story_id: STORY-119
epic_id: E-18
version: "1.8"
status: draft
producer: story-writer
timestamp: 2026-06-18T00:00:00Z
phase: f3
points: 8
priority: P2
depends_on: [STORY-120]
blocks: []
behavioral_contracts:
  - BC-2.11.013
  - BC-2.11.014
  - BC-2.11.016
  - BC-2.11.025
  - BC-2.11.026
  - BC-2.11.027
  - BC-2.11.028
  - BC-2.11.030
  - BC-2.11.031
  - BC-2.11.032
  - BC-2.11.033
  - BC-2.11.034
verification_properties: [VP-012, VP-016]
tdd_mode: strict
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 2
feature_id: e18-finding-collapse
github_issue: 259
wave: 49
assumption_validations: []
risk_mitigations: []
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.014.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.016.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.025.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.028.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.030.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.031.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.032.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.033.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.034.md
  - .factory/phase-f1-delta-analysis/story-119-grouped-mode-collapse-delta-analysis.md
  - .factory/phase-f2-spec-evolution/story-119-type-design.md
  - .factory/phase-f2-spec-evolution/story-119-prd-delta.md
input-hash: "87e1b0c"
# BC status: all 12 BCs authored/converged at F2 (2026-06-18). See individual BC version
#   stamps in the Behavioral Contracts table below. All BCs are F2-frozen; normative
#   bodies must not be edited as part of STORY-119 F4 implementation.
# Subsystem anchor: SS-11 owns this story's scope because grouped-mode collapse is a
#   display-layer extension of reporter/terminal.rs — the core SS-11 module.
# Dependency anchor:
#   depends_on: [STORY-120] — STORY-120 introduces the three-variant FindingsRender enum
#   (FindingsRender::Grouped, ::FlatCollapsed, ::FlatExpanded); STORY-119 evolves that
#   enum into the struct-of-orthogonal-enums (FindingsRender { grouping: Grouping,
#   collapse: Collapse }) so that {Grouped, Collapsed} becomes representable, then
#   implements per-bucket grouped collapse. The type must exist (STORY-120) before
#   STORY-119 can reshape it. STORY-119 cannot be built before STORY-120 ships.
#   blocks: [] — No downstream stories depend on grouped-mode collapse.
# Wave anchor: wave 49 = max(wave(STORY-120)=48) + 1. STORY-120 is the unique predecessor.
# Version 1.6 changes (F3 adversarial round-1 remediation, 2026-06-18):
#   BC stamps synced to PO-final versions (014→v2.0, 025→v1.12, 027→v1.7, 031→v1.3,
#   032→v1.4, 033→v1.3); C-1 propagation: collapse_findings_pass → collapse_findings_pass_refs
#   shared helper (F4-new); collapse_findings_pass becomes thin adapter; Task 4/AC-027/
#   Architecture Anchors updated; ADR-0003 "Collapse-API Shape" + F2 §5.2.1 referenced;
#   H-2: Task 7 + AC-030 gain explicit sweep target for stale verdict-desc/confidence-desc
#   doc-comment at terminal.rs:429-430 (correct to ascending); M-1: Task 6/AC-030 updated
#   from nonexistent help= attribute to doc-comment (///); M-2: Forbidden Dependencies gains
#   render_finding_grouped N≥2 prohibition; MEDIUM-1: orphan governing-BC trace anchors
#   added for BC-2.11.014/016/026/027 (verbatim from source BC postconditions/invariants).
# Version 1.5 changes (F3 full decomposition): full ACs (AC-001..AC-031) each tracing
#   verbatim to governing BC postcondition/invariant; full implementation tasks (Tasks 1-9)
#   including struct reshape (~46 sites), render_findings_grouped_collapsed implementation,
#   CLI struct-construction wiring, comment sweep; VP assignments (VP-012, VP-016);
#   wave assignment (wave: 49); inputs list populated; deferred markers removed;
#   CARRY-119-F3-RESIDUALS-001 fixes applied (VP-table test rename, NIT changelog fix).
---

# STORY-119: Terminal Finding-Collapse — Grouped Mode / --mitre

## Narrative

- **As a** network security analyst using `wirerust analyze --mitre`
- **I want** repeated identical findings within the same MITRE tactic bucket to be
  collapsed with a ` (xN)` count suffix, with K=3 evidence sampling, and `--no-collapse`
  as the dual-scope opt-out that suppresses collapse in both flat and grouped modes
- **So that** I can quickly assess the volume of repeated findings per tactic without
  wading through thousands of identical grouped-mode lines, while retaining access to
  the pre-collapse per-finding view via `--no-collapse`

**Scope:** STORY-119 reshapes `FindingsRender` from the three-variant enum introduced
by STORY-120 (`FindingsRender::Grouped`, `::FlatCollapsed`, `::FlatExpanded`) into the
struct-of-orthogonal-enums (`FindingsRender { grouping: Grouping, collapse: Collapse }`)
where all four Cartesian combinations are valid. It then implements grouped-mode collapse:
per-tactic-bucket deduplication with ` (xN)` count suffix, K=3 evidence sampling, em-dash
MITRE line from `group_members[0]`, and tactic-bucket ordering invariant under collapse.

**Behavior change:** `--mitre` alone now produces `{Grouped, Collapsed}` (grouped output
with per-bucket collapse). In v0.9.0, `--mitre` produced `{Grouped, Expanded}` (the old
`FindingsRender::Grouped`). The pre-collapse behavior is preserved exactly via
`--mitre --no-collapse`, which produces `{Grouped, Expanded}`. This change was approved
at the F1 gate (D-110). `--no-collapse` is now dual-scope: it suppresses collapse in
both flat and grouped modes.

---

## Behavioral Contracts

All 12 BCs governing this story are authored and F2-frozen.

| BC | Version | Role for STORY-119 |
|----|---------|-------------------|
| BC-2.11.013 | v1.14 | Tactic-header structure and collapse axis: when `render.grouping == Grouping::Grouped` with `Collapse::Collapsed`, per-bucket collapse applies; with `Collapse::Expanded`, suffix-free. Tactic-bucket ordering and header format unchanged. |
| BC-2.11.014 | v2.0 | Within-bucket sort — ascending by verdict-rank (Likely=0, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank (High=0, Medium=1, Low=2), then emission-index — determines the post-sort group representative (members[0]). |
| BC-2.11.016 | v1.9 | Grouped MITRE em-dash+name line format — em-dash + technique name for known IDs, `(unknown)` for unrecognized; applies to both `{Grouped, Collapsed}` and `{Grouped, Expanded}` paths; consumed by BC-2.11.034 for collapsed N≥2 groups. |
| BC-2.11.025 | v1.13 | Flat-mode collapse key and first-occurrence order: the `(category, verdict, confidence, summary)` four-tuple key and `Vec<(CollapseKey, Vec<&Finding>)>` accumulator structure; `collapse_findings_pass` (flat-mode wrapper) delegates to `collapse_findings_pass_refs`; grouped per-bucket collapse calls `collapse_findings_pass_refs` directly. Scoped to `Grouping::Flat`. |
| BC-2.11.026 | v1.13 | Flat-mode `(xN)` suffix rule and color-ladder requirement; grouped analogue is BC-2.11.031. PC-4 suffix-free guarantee now scoped to `{Grouped, Expanded}` only. Canonical test vectors migrated to struct form in v1.13. |
| BC-2.11.027 | v1.7 | Flat-mode K=3 evidence sampling — positional no-sliding-window; grouped analogue is BC-2.11.032. Vocabulary migrated to D-110 struct form in v1.7. |
| BC-2.11.028 | v1.9 | `--no-collapse` opt-out flag: dual-scope since STORY-119 — disables collapse in both flat and grouped modes. Wiring uses struct construction `FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`. |
| BC-2.11.030 | v1.4 | CLI→render mode mapping: `--mitre` alone → `{Grouped, Collapsed}` (new default); `--mitre --no-collapse` → `{Grouped, Expanded}` (pre-STORY-119 `--mitre` behavior). |
| BC-2.11.031 | v1.3 | Per-bucket `(xN)` count suffix: N≥2 group within a tactic bucket renders header with ` (xN)` suffix before colorization; N=1 singleton renders via `render_finding_grouped` without suffix. Color-ladder requirement identical to BC-2.11.026. |
| BC-2.11.032 | v1.4 | Per-bucket K=3 evidence sampling: first `min(N, K)` members positionally; `evidence[0]` from each if non-empty; window does NOT slide past empty-evidence members. |
| BC-2.11.033 | v1.3 | Tactic-bucket ordering invariant under grouped-collapse: bucket sequence per `all_tactics_in_report_order()` is unchanged; `Uncategorized` last; collapse occurs strictly within buckets. Sort-then-collapse ordering: per-bucket sort (ascending verdict/confidence/emission-index rank) PRECEDES `collapse_findings_pass_refs` for that bucket. |
| BC-2.11.034 | v1.3 | MITRE line format for N≥2 collapsed groups: em-dash name expansion sourced from `group_members[0]`; no `(xN)` suffix on MITRE line; unknown ID → `(unknown)` format. Singletons use `render_finding_grouped` (BC-2.11.016 governs). |

---

## Acceptance Criteria

### AC-001 — `--mitre` alone routes to `{Grouped, Collapsed}` at the construction site
The `TerminalReporter` construction site in `src/main.rs::run_analyze` uses the new struct
literal form: `render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`. When `show_mitre_grouping == true` and `collapse_findings == true` (i.e., `--mitre` alone, no `--no-collapse`), the resulting `render` is `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.
(traces to BC-2.11.030 Postcondition 2: "When `--mitre` is present and `--no-collapse` is absent (the new default): `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.")

### AC-002 — `--mitre --no-collapse` routes to `{Grouped, Expanded}` at the construction site
When `show_mitre_grouping == true` and `collapse_findings == false` (i.e., `--mitre` with `--no-collapse`), the resulting `render` at the `run_analyze` construction site is `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }`.
(traces to BC-2.11.030 Postcondition 3: "When `--mitre` is present and `--no-collapse` is also present: `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }`.")

### AC-003 — flat-mode routing unchanged at construction site
When `show_mitre_grouping == false` and `collapse_findings == true` (default), `render == {Flat, Collapsed}`. When `show_mitre_grouping == false` and `collapse_findings == false`, `render == {Flat, Expanded}`. Both unchanged from v0.8.0 behavior.
(traces to BC-2.11.030 Postconditions 4–5: unchanged flat-mode routing.)

### AC-004 — `run_summary` construction site produces `{Flat, Collapsed}`
The `run_summary` construction site in `src/main.rs` uses struct literal form `render: FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`. The inert value semantics are unchanged — `run_summary` renders no FINDINGS section; the field is structurally present but irrelevant.
(traces to BC-2.11.030 Postcondition 6: "The `run_summary` construction site is unaffected: it always produces `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`.")

### AC-005 — `FindingsRender` three-variant enum replaced by struct-of-two-orthogonal-enums in `terminal.rs`
The `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` at `terminal.rs:100-111` (v0.9.0) is replaced by: `pub enum Grouping { Grouped, Flat }`, `pub enum Collapse { Collapsed, Expanded }`, and `pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }`. All three types derive `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. No `Default` is derived on any of the three types.
(traces to BC-2.11.028 Invariant 1: "…`render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`… The two axes are fully orthogonal: no combination is illegal.")

### AC-006 — `match self.render` three-arm dispatch replaced by four-arm tuple dispatch
The `match self.render` three-arm dispatch in `TerminalReporter::render()` is replaced by a `match (self.render.grouping, self.render.collapse)` four-arm tuple dispatch:
- `(Grouping::Grouped, Collapse::Expanded)` → `self.render_findings_grouped(&mut out, findings)` (UNCHANGED behavior, suffix-free)
- `(Grouping::Grouped, Collapse::Collapsed)` → `self.render_findings_grouped_collapsed(&mut out, findings)` (NEW)
- `(Grouping::Flat, Collapse::Collapsed)` → `self.render_findings_collapsed(&mut out, findings)` (UNCHANGED)
- `(Grouping::Flat, Collapse::Expanded)` → `for f in findings { self.render_finding_flat(&mut out, f); }` (UNCHANGED)
(traces to BC-2.11.013 Invariant 4: "When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (`--mitre --no-collapse`): the collapse pass is NOT applied… When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` (`--mitre` alone, the new default since STORY-119): a per-bucket collapse pass IS applied within each tactic bucket.")

### AC-007 — all 84 `FindingsRender::Variant` construction sites updated to struct literal form
Every occurrence of `FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, and `FindingsRender::FlatExpanded` across all source and test files is updated to the corresponding struct literal per the D-110 migration map:
- `FindingsRender::Grouped` → `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (preserves suffix-free semantics)
- `FindingsRender::FlatCollapsed` → `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`
- `FindingsRender::FlatExpanded` → `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }`
After migration: `cargo build --all-targets` compiles with zero errors. `cargo test --all-targets` passes with zero test failures.
(traces to BC-2.11.028 Postcondition 4: "The `collapse` axis of the `FindingsRender` struct is determined exclusively by `collapse_findings`; the `grouping` axis by `show_mitre_grouping`. They are fully orthogonal.")

### AC-008 — `render_findings_grouped_collapsed` function exists and is dispatched for `{Grouped, Collapsed}`
A new function `render_findings_grouped_collapsed` exists in `src/reporter/terminal.rs`. It is called by the `(Grouping::Grouped, Collapse::Collapsed)` arm of the four-arm tuple dispatch. The function handles tactic bucketing and sorting identically to `render_findings_grouped` (BC-2.11.013), then applies a per-bucket collapse pass before rendering.
(traces to BC-2.11.031 Architecture Anchors: "`render_findings_grouped_collapsed` — F4-pending new function: per-bucket collapse + grouped-collapse header rendering".)

### AC-009 — per-bucket collapse: N≥2 group within a bucket renders header with `(xN)` suffix
For a group of N≥2 findings sharing the same `(category, verdict, confidence, summary)` key within a MITRE tactic bucket under `{Grouped, Collapsed}`, the header line reads: `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary> (x<N>)\n` where `<N>` is the exact decimal integer count (no leading zeros, no space between `x` and `N`).
(traces to BC-2.11.031 Postcondition 1: "For a group of N≥2 within a tactic bucket: the header line reads: `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary> (x<N>)\n`…")
(governing-BC trace: BC-2.11.026 Precondition 1: "`TerminalReporter.render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` (`{Flat, Collapsed}` — the flat-mode collapsed path). The (xN) suffix rule defined in this BC applies to the flat-mode path. The grouped-mode analogue is BC-2.11.031." — this AC covers the grouped-mode analogue defined in BC-2.11.031; BC-2.11.026 governs the canonical suffix-format contract that BC-2.11.031 mirrors.)

### AC-010 — per-bucket collapse: singleton (N=1) within a bucket renders via `render_finding_grouped` with no suffix
For a singleton group (N=1 within a bucket) under `{Grouped, Collapsed}`, `render_finding_grouped` is called for that finding. The output is byte-identical to the `{Grouped, Expanded}` path for the same finding — no count suffix, MITRE name expansion per BC-2.11.016.
(traces to BC-2.11.031 Postcondition 2: "For a singleton group (N=1 within a bucket): `render_finding_grouped` is called for that finding. The output is byte-identical to the `{Grouped, Expanded}` path for the same finding — no count suffix, MITRE name expansion per BC-2.11.016.")
(governing-BC trace: BC-2.11.016 Precondition 1: "`TerminalReporter.render.grouping == Grouping::Grouped` (applies to both `{Grouped, Collapsed}` and `{Grouped, Expanded}` paths; the em-dash MITRE expansion occurs on all `render_finding_grouped` calls regardless of collapse axis)." — the singleton path calls `render_finding_grouped` which is governed by BC-2.11.016 for its MITRE em-dash line.)

### AC-011 — color-ladder requirement: `(xN)` suffix is part of the pre-colorization string
The grouped-collapse header path applies the same verdict/confidence color-ladder as the color-selection block beginning at `terminal.rs:391` to a pre-color string that ALREADY INCLUDES the ` (xN)` suffix. The ladder: `Likely+High` → `red().bold()`; `Likely+other` → `yellow`; `Possible` → `yellow`; `Inconclusive` → `cyan`; `Unlikely` → `dimmed`. Appending the suffix AFTER the ANSI color-reset sequence is NON-CONFORMANT.
(traces to BC-2.11.031 Postcondition 3: "The grouped-collapse header path MUST apply the same verdict/confidence color-selection logic as `terminal.rs:391` to a pre-color string that ALREADY INCLUDES the ` (xN)` suffix… appending the suffix after the ANSI reset is NON-CONFORMANT.")

### AC-012 — `(xN)` suffix does NOT appear on the MITRE line, evidence lines, or tactic bucket headers
The ` (xN)` suffix appears ONLY on the finding-group header line. It must not appear on the MITRE line (`    MITRE: <ids> \u{2014} <name>`), on any `    > <evidence>` line, or on the tactic bucket header (`  ## <TacticName>`).
(traces to BC-2.11.031 Postcondition 4: "The ` (xN)` suffix MUST NOT appear on the MITRE line, any evidence line, or the tactic bucket header (`## <TacticName>`). It appears only on the finding-group header line.")

### AC-013 — cross-bucket suffix independence: same key in different buckets produces independent counts
Two groups with the same `(category, verdict, confidence, summary)` collapse key but in different MITRE tactic buckets each emit their own independent `(xN)` suffix based on their own per-bucket group count. A group of 3 in bucket A and a group of 2 in bucket B produce `(x3)` and `(x2)` respectively; they are never merged to `(x5)`.
(traces to BC-2.11.031 Postcondition 6: "Cross-bucket suffix independence: two groups with the same collapse key in different tactic buckets each emit their own independent `(xN)` suffix based on their own bucket group count.")

### AC-014 — per-bucket evidence sampling: at most K=3 evidence lines per N≥2 group
For a collapsed group of N≥2 within a tactic bucket under `{Grouped, Collapsed}`, the terminal output contains at most K=3 evidence lines (`COLLAPSE_EVIDENCE_SAMPLES = 3`), each rendered as `    > <escaped_evidence_line>\n`. Evidence lines are drawn from the FIRST `min(N, K)` members in post-sort bucket order; for each inspected member, `evidence[0]` is emitted if non-empty; if a member has an empty evidence vec it contributes 0 lines and the window does NOT slide past it.
(traces to BC-2.11.032 Postconditions 1–2: "The terminal output for a grouped-collapse group of N≥2 contains at most K=3 evidence lines… Evidence lines are drawn from the FIRST `min(N, K)` members of the per-bucket group… The window does NOT slide past empty-evidence members.")
(governing-BC trace: BC-2.11.027 Precondition 1: "`TerminalReporter.render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` (`{Flat, Collapsed}` — the flat-mode collapsed path). The grouped-mode analogue for per-bucket evidence sampling is BC-2.11.032." — this AC covers the grouped-mode analogue BC-2.11.032; BC-2.11.027 governs the canonical K=3 positional-sampling contract that BC-2.11.032 mirrors.)

### AC-015 — per-bucket evidence sampling: no "N-K more" elision annotation
When the group has N>K findings within the bucket, exactly K evidence lines appear (assuming selected members have non-empty evidence). No elision marker, annotation, or "and N-K more evidence lines" indicator is rendered. The `(xN)` header suffix is the only indicator of group size.
(traces to BC-2.11.032 Postcondition 4: "When the group has N>K findings within the bucket, exactly K evidence lines appear… No '… and N-K more evidence lines elided' annotation or similar indicator is emitted.")

### AC-016 — tactic-bucket ordering invariant: bucket sequence per `all_tactics_in_report_order()` unchanged
Under `{Grouped, Collapsed}`, tactic bucket headers appear in the order returned by `all_tactics_in_report_order()` — identical to BC-2.11.013 Postcondition 2. A tactic bucket header is emitted only when at least one finding belongs to that bucket (empty-bucket guard per BC-2.11.013 PC-3). Collapse within a bucket does not prevent the bucket header from being emitted.
(traces to BC-2.11.033 Postconditions 1–2: "Tactic bucket headers appear as `  ## <TacticName>\n` in the output, in the order returned by `all_tactics_in_report_order()` — identical to BC-2.11.013 PC-2. A tactic bucket header is emitted only when at least one finding belongs to that bucket…")

### AC-017 — `Uncategorized` bucket emitted last under `{Grouped, Collapsed}`
The `Uncategorized` bucket header (`  ## Uncategorized\n`) appears last among emitted buckets, collecting findings where `mitre_techniques` is empty OR where `technique_tactic(mitre_techniques[0])` returns `None` — identical to BC-2.11.013 PC-4.
(traces to BC-2.11.033 Postcondition 3: "The `Uncategorized` bucket header (`  ## Uncategorized\n`) appears last among emitted buckets, collecting findings where `mitre_techniques` is empty OR where `technique_tactic(mitre_techniques[0])` returns `None` — identical to BC-2.11.013 PC-4.")

### AC-018 — bucket membership unchanged by collapse
A finding assigned to bucket B under `{Grouped, Expanded}` is assigned to the same bucket B under `{Grouped, Collapsed}`. The collapse key `(category, verdict, confidence, summary)` is orthogonal to bucket assignment (which uses `mitre_techniques[0]`). The per-bucket collapse pass does NOT reassign findings to different buckets.
(traces to BC-2.11.033 Postcondition 4: "Bucket membership is unchanged by collapse. A finding assigned to bucket B under `{Grouped, Expanded}` is assigned to the same bucket B under `{Grouped, Collapsed}`.")

### AC-019 — sort-then-collapse ordering: per-bucket sort precedes `collapse_findings_pass_refs`
Within each tactic bucket, findings are sorted ascending by rank — verdict-rank ascending (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank ascending (High=0 first, Medium=1, Low=2), then emission-index ascending — BEFORE `collapse_findings_pass_refs` is applied to that bucket's slice. The group representative `members[0]` is therefore the first finding in the sorted bucket order, not first in the original global emission order.
(traces to BC-2.11.033 Postcondition 5: "Within each bucket, findings are sorted ascending by rank — verdict-rank ascending (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank ascending (High=0 first, Medium=1, Low=2), then emission-index ascending — BEFORE the per-bucket `collapse_findings_pass_refs` is applied.")
(governing-BC trace: BC-2.11.014 Invariant 1: "Verdict ranks: Likely=0, Possible=1, Inconclusive=2, Unlikely=3 (defined by local `verdict_rank` function in terminal.rs:447-454; source-confirmed match arms).")

### AC-020 — group order within bucket is first-occurrence in SORTED bucket order
The per-bucket `collapse_findings_pass_refs` produces groups whose order within the bucket is first-occurrence in the SORTED bucket order (not first-occurrence in the global emission order). Two findings with the same key in a bucket: the one with lower rank value (higher severity) sorts first and becomes `members[0]` (group representative).
(traces to BC-2.11.033 Postcondition 6: "The per-bucket `collapse_findings_pass_refs` produces groups whose order within the bucket is first-occurrence in the SORTED bucket order (not first-occurrence in the global emission order). This is the 'post-sort first-occurrence' definition for grouped-collapse mode.")

### AC-021 — MITRE line for N≥2 collapsed group: em-dash name expansion sourced from `members[0]`
For a collapsed group of N≥2 in a tactic bucket, the MITRE line is rendered from `group_members[0].mitre_techniques`: if non-empty, the format is `    MITRE: <ids_joined> \u{2014} <name>\n` (known ID) or `    MITRE: <ids_joined> (unknown)\n` (unrecognized ID). If `group_members[0].mitre_techniques` is empty, no MITRE line is rendered. The IDs are joined as `mitre_techniques.join(", ")`. The separator is U+2014 (EM DASH), not ASCII `--`.
(traces to BC-2.11.034 Postcondition 1: "For a collapsed group of N≥2 in a tactic bucket: after the header line and evidence lines, the MITRE line is rendered from `group_members[0].mitre_techniques`… if known, the line reads `    MITRE: <ids_joined> \u{2014} <name>\n`; if unknown, the line reads `    MITRE: <ids_joined> (unknown)\n`. The separator is U+2014 (EM DASH).")

### AC-022 — `(xN)` suffix does NOT appear on the MITRE line for N≥2 groups
The `(xN)` count suffix does not appear on the MITRE line for N≥2 collapsed groups. The count suffix is scoped to the header line only (AC-012).
(traces to BC-2.11.034 Postcondition 2: "The `(xN)` count suffix does NOT appear on the MITRE line. The count suffix is scoped to the header line only.")

### AC-023 — other group members' `mitre_techniques` are elided from terminal output; preserved in JSON/CSV
Other group members' `mitre_techniques` (members[1], members[2], …, members[N-1]) are elided from terminal output. Their technique data is preserved unmodified in the raw `findings` slice available to JSON/CSV reporters (BC-2.11.029).
(traces to BC-2.11.034 Postcondition 3: "Other group members' `mitre_techniques` (members[1], members[2], ..., members[N-1]) are elided from terminal output. Their technique data is preserved unmodified in the raw `findings` slice available to JSON/CSV reporters.")

### AC-024 — observable output block order within a collapsed group
For a collapsed N≥2 group, the output order is: (1) header line with `(xN)` suffix, (2) up to K=3 evidence lines (`    > <evidence>`), (3) MITRE line from `members[0]` (if `mitre_techniques` non-empty). The `(xN)` suffix appears ONLY in item (1).
(traces to BC-2.11.034 Postcondition 5: "The observable MITRE line order: within a collapsed group's output block, the order is: (1) header line with `(xN)` suffix, (2) up to K=3 evidence lines (BC-2.11.032), (3) MITRE line from `members[0]` (this PC). The `(xN)` suffix appears ONLY on (1).")

### AC-025 — `{Grouped, Expanded}` path (`--mitre --no-collapse`): zero `(xN)` suffixes even for large N
Under `render = {Grouped, Expanded}`, N=100 identical-key findings in a tactic bucket produce 100 individual finding lines with no ` (xN)` suffix on any line. The `{Grouped, Expanded}` suffix-free guarantee (BC-2.11.013 Invariant 4, pre-STORY-119 `--mitre` behavior) is unchanged and the existing test `test_BC_2_11_013_grouped_mode_suffix_free` continues to pass without modification.
(traces to BC-2.11.013 Invariant 4: "When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (`--mitre --no-collapse`): the collapse pass is NOT applied… The OBSERVABLE GUARANTEE holds: no ` (xN)` suffix appears in the terminal output for any finding, at any input volume.")

### AC-026 — `escape_for_terminal` called on all summary and evidence strings in the grouped-collapse path
All `summary` and `evidence` strings in `render_findings_grouped_collapsed` pass through `escape_for_terminal` before being written to the output buffer. The collapse pass operates on raw (unescaped) `Finding` field values; escape is render-time, not key-time. VP-012 invariant is preserved.
(traces to BC-2.11.031 Precondition 5: "`escape_for_terminal` has been applied to the group representative's `summary` field before the suffix is appended (VP-012 invariant; BC-2.11.010).")

### AC-027 — `collapse_findings_pass_refs` called once per tactic bucket slice (not across global findings)
In `render_findings_grouped_collapsed`, `collapse_findings_pass_refs(&[&Finding])` (the F4-new shared helper) is called once per bucket's per-bucket sorted slice, not once for the global `findings` slice. `collapse_findings_pass` at `:340` becomes a thin adapter: it collects `self.findings.iter().collect()` and delegates to `collapse_findings_pass_refs`. The grouped caller collects `bucket_refs: Vec<&Finding> = items.iter().map(|(_, f)| *f).collect()` then calls `collapse_findings_pass_refs(&bucket_refs)`. The collapse LOGIC is shared/reused via `collapse_findings_pass_refs`; the exact original `collapse_findings_pass` signature is not called from the grouped path. There is no cross-bucket collapse pass. Reference: ADR-0003 "Collapse-API Shape" subsection and F2 design-note §5.2.1.
(traces to BC-2.11.033 Invariant 3: "The per-bucket collapse pass is applied to the sorted-bucket slice for each tactic bucket independently and sequentially in tactic-order. There is no global cross-bucket collapse pass; `collapse_findings_pass_refs` never receives the full global `findings` slice in grouped mode.")
(governing-BC trace: BC-2.11.025 Invariant 5: "This BC and its invariants are scoped to `Grouping::Flat` (the `{Flat, Collapsed}` path)… The grouped-mode collapse (`{Grouped, Collapsed}`) is a distinct per-bucket invocation of `collapse_findings_pass_refs` (the shared collapse-logic helper; `collapse_findings_pass` delegates to it for flat mode), governed by BC-2.11.031.")

### AC-028 — `{Grouped, Expanded}` path output byte-identical to v0.9.0 `FindingsRender::Grouped` path
The `(Grouping::Grouped, Collapse::Expanded)` dispatch arm calls `render_findings_grouped` identically to the v0.9.0 `FindingsRender::Grouped` arm. The output is byte-identical to v0.9.0 for the same input. The `render_findings_grouped` function body is NOT modified by this story (only the dispatch is reshaped).
(traces to BC-2.11.013 Invariant 4: "When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (`--mitre --no-collapse`): the collapse pass is NOT applied. Each finding is rendered individually via one `render_finding_grouped` call, with no `(xN)` count suffix. This is the pre-STORY-119 `--mitre` behavior, now explicitly selected via `--no-collapse`.")

### AC-029 — flat paths byte-identical: `{Flat, Collapsed}` and `{Flat, Expanded}` outputs unchanged
The `(Grouping::Flat, Collapse::Collapsed)` arm calls `render_findings_collapsed` and the `(Grouping::Flat, Collapse::Expanded)` arm calls the `render_finding_flat` loop — both byte-identical to v0.9.0 `FlatCollapsed` and `FlatExpanded` arms. All existing flat-mode tests (`BC-2.11.025`, `BC-2.11.026`, `BC-2.11.027`, `BC-2.11.028`, `BC-2.11.029` suites) continue to pass without modification.
(traces to BC-2.11.025 Invariant 5: "This BC and its invariants are scoped to `Grouping::Flat` (the `{Flat, Collapsed}` path).")

### AC-030 — comment sweep: no stale `FindingsRender::Grouped`/`FlatCollapsed`/`FlatExpanded` enum vocabulary survives
A census of all source files and test files confirms zero remaining references to `FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded` as enum variants. The `///` doc-comment lines on the `no_collapse: bool` field in `cli.rs` (approximately lines 154-159; clap derives `--help` text from these `///` lines; the field has bare `#[arg(long)]` with NO `help = "..."` attribute) are updated to describe dual-scope behavior (flat and grouped modes). The stale doc-comment on `render_findings_grouped` at `src/reporter/terminal.rs:429-430` reading "verdict-desc, then confidence-desc" is corrected to ascending sort order. Stale comments referencing `show_mitre_grouping`/`collapse_findings` bool fields in the context of `FindingsRender` enum dispatch are updated or removed.
(traces to BC-2.11.028 Invariant 6: "Per LESSON-P1.04 (no unwired flags): the `no_collapse` field in `cli.rs` MUST be wired to `TerminalReporter.render` in `main.rs` via the two-field struct expression in Invariant 1 (STORY-119 F4 target)…")

### AC-031 — `cargo test --all-targets` passes green after all changes
After implementing all tasks, `cargo test --all-targets` passes with zero failures and zero compiler warnings. The new `mod story_119` test block contributes at minimum the tests in the VP table below. The migration does not regress any existing test in the `story_118`, `story_078`, `story_077`, or any other test module.
(traces to BC-2.11.030 Canonical Test Vector: "`TerminalReporter { ..., render: FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed } }` constructed directly in test — Dispatches to `render_findings_grouped_collapsed`; grouped-collapse behavior exercised.")

---

## Verification Properties

| VP-NNN | Property | Coverage in this Story | Proof Method |
|--------|----------|-----------------------|-------------|
| VP-012 | `escape_for_terminal` correctness (BC-2.11.010) | AC-026: all summary + evidence in the grouped-collapse path pass through `escape_for_terminal`; proptest harness unchanged; the grouped-collapse path inherits the same call sites as the existing grouped-expanded path — no new escape bypasses introduced | proptest (existing; unchanged) |
| VP-016 | Tactic headers in canonical order (BC-2.11.013/014/015) | AC-016, AC-017: `all_tactics_in_report_order()` unchanged; existing integration tests `mitre_grouping_emits_tactic_headers_in_canonical_order` and `mitre_grouping_buckets_none_and_unknown_under_uncategorized` continue to pass under `{Grouped, Expanded}` path (byte-identical to v0.9.0); new `test_BC_2_11_033_grouped_collapsed_preserves_bucket_order` covers the `{Grouped, Collapsed}` path. | integration (existing + new test) |

---

## Implementation Tasks

### Task 1 — Replace `FindingsRender` three-variant enum with `Grouping` + `Collapse` + `FindingsRender` struct in `src/reporter/terminal.rs`

**File:** `src/reporter/terminal.rs`
**Scope:** Lines 100-111 (current `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }`)

Replace with:
```rust
/// Grouping axis for the FINDINGS section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    Grouped,
    Flat,
}

/// Collapse axis for the FINDINGS section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collapse {
    Collapsed,
    Expanded,
}

/// Rendering mode for the FINDINGS section of [`TerminalReporter`].
/// No [`Default`] is derived — deliberate, consistent with STORY-120.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingsRender {
    pub grouping: Grouping,
    pub collapse: Collapse,
}
```

Update the `use` import in `src/main.rs`: `use wirerust::reporter::terminal::{FindingsRender, TerminalReporter};` → `use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};`

**Acceptance gate:** `cargo build --all-targets` fails at every stale `FindingsRender::Grouped`/`FlatCollapsed`/`FlatExpanded` variant site (exhaustiveness enforcement). Proceed to Task 2.

---

### Task 2 — Mechanical migration: update 84 `FindingsRender::Variant` construction sites

**Files affected (census from F1 §8 + F2 §9):**
- `src/main.rs` — 4 enum-variant occurrences across 2 logical construction sites (the 3-arm if-expression at :382/:384/:386 collapses to one struct literal; the `run_summary` site at :449)
- `tests/reporter_terminal_tests.rs` — 55 sites across all story_NNN blocks
- `tests/reporter_tests.rs` — 17 `mitre_reporter()` helper sites
- `tests/dnp3_f5_remediation_tests.rs` — `mitre_reporter` helper
- `tests/bc_2_09_100_multitag_tests.rs` — parameterized helper

**Migration map (apply to every site):**
| Old enum variant | New struct literal |
|------------------|--------------------|
| `FindingsRender::Grouped` | `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` |
| `FindingsRender::FlatCollapsed` | `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` |
| `FindingsRender::FlatExpanded` | `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }` |

**Wiring for `run_analyze` construction site** (`src/main.rs` — replaces the 3-arm if-expression):
```rust
render: FindingsRender {
    grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat },
    collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded },
},
```
Note: `show_mitre_grouping` (line 107) and `collapse_findings` (line 108) are the in-scope bool params inside `run_analyze`. The `--mitre`/`--no-collapse` → bool resolution at `main()` lines 79-80 is UNCHANGED.

**Wiring for `run_summary` construction site** (`src/main.rs`):
```rust
render: FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed },
```

**Acceptance gate:** `cargo build --all-targets` compiles clean with zero errors after all sites updated. `cargo test --all-targets` shows only the new `mod story_119` tests as new/failing (Red Gate). All pre-existing tests pass.

---

### Task 3 — Rewrite `match self.render` three-arm dispatch as four-arm tuple dispatch in `TerminalReporter::render()`

**File:** `src/reporter/terminal.rs`
**Scope:** The `match self.render { ... }` block inside `TerminalReporter::render()` (approx. lines 202-224 in v0.9.0)

Replace with:
```rust
match (self.render.grouping, self.render.collapse) {
    (Grouping::Grouped, Collapse::Expanded) => {
        self.render_findings_grouped(&mut out, findings);
    }
    (Grouping::Grouped, Collapse::Collapsed) => {
        self.render_findings_grouped_collapsed(&mut out, findings);
    }
    (Grouping::Flat, Collapse::Collapsed) => {
        self.render_findings_collapsed(&mut out, findings);
    }
    (Grouping::Flat, Collapse::Expanded) => {
        for f in findings {
            self.render_finding_flat(&mut out, f);
        }
    }
}
```

The call `self.render_findings_grouped_collapsed` references the new function added in Task 4.

**Acceptance gate:** `cargo build` compiles (will reference the to-be-added function stub). All three existing arms (`Grouped/Expanded`, `Flat/Collapsed`, `Flat/Expanded`) delegate to the same existing functions as before — zero behavior change for those paths.

---

### Task 4 — Implement `render_findings_grouped_collapsed`

**File:** `src/reporter/terminal.rs`
**Location:** Add after `render_findings_grouped` (approx. line 484 in v0.9.0)
**Structural model:** Mirrors `render_findings_grouped` (tactic bucketing + outer loop) with the inner loop replaced by a per-bucket `collapse_findings_pass_refs` invocation and collapsed rendering.

**Behavioral specification (extracted verbatim from BCs):**

1. **Tactic bucketing and outer loop** (identical to `render_findings_grouped`, BC-2.11.013):
   - Build `HashMap<Option<MitreTactic>, Vec<(usize, &Finding)>>` keyed by `technique_tactic(mitre_techniques[0])` or `None` for empty/unknown.
   - Iterate `all_tactics_in_report_order()` for named buckets, then `None` for Uncategorized.
   - Emit `  ## <TacticName>\n` header only when the bucket is non-empty (BC-2.11.033 PC-2).

2. **Per-bucket sort** (BC-2.11.033 PC-5 / BC-2.11.014):
   - Sort ascending: verdict-rank (Likely=0, Possible=1, Inconclusive=2, Unlikely=3), then confidence-rank (High=0, Medium=1, Low=2), then emission-index ascending.
   - This sort already exists in `render_findings_grouped` (~:463-467 in v0.9.0); reproduce or refactor to share.

3. **Per-bucket collapse** (BC-2.11.033 Invariant 3, BC-2.11.025 Invariant 5, C-1 resolution):
   - Collect bucket refs: `let bucket_refs: Vec<&Finding> = items.iter().map(|(_, f)| *f).collect();`
   - Call `collapse_findings_pass_refs(&bucket_refs)` once per bucket slice. This is the F4-new shared helper. The function returns `Vec<(CollapseKey, Vec<&Finding>)>`.
   - `collapse_findings_pass` at `:340` becomes a thin adapter: collects `self.findings.iter().collect()` and delegates to `collapse_findings_pass_refs`. It is NOT called directly from the grouped path.
   - Reference: ADR-0003 "Collapse-API Shape" subsection; F2 design-note §5.2.1.

4. **Render each group** (BC-2.11.031, BC-2.11.032, BC-2.11.034):
   - **N=1 (singleton):** call `render_finding_grouped(out, group_members[0])` — byte-identical to `{Grouped, Expanded}` for that finding.
   - **N≥2 (collapsed group):**
     a. Build pre-color string: `escaped_summary + format!(" (x{})", N)`.
     b. Apply color-ladder (same as the color-selection block beginning at `terminal.rs:391`): `Likely+High` → `red().bold()`; `Likely+other` → `yellow`; `Possible` → `yellow`; `Inconclusive` → `cyan`; `Unlikely` → `dimmed`. Apply with `use_color` guard.
     c. Write header: `"  [<Category>] <VERDICT> (<CONFIDENCE>) - <colored_line>\n"`.
     d. Evidence loop: iterate `members[0..min(N, COLLAPSE_EVIDENCE_SAMPLES)]`; for each member with non-empty `evidence`, write `"    > {}\n"` for `escape_for_terminal(&evidence[0])`. Window does NOT slide past empty-evidence members.
     e. MITRE line: if `group_members[0].mitre_techniques` is non-empty, call the same name-expansion logic as `render_finding_grouped` (BC-2.11.034 Invariant 2): `ids.join(", ")` + `technique_name(ids[0])` → `Some(name)` → `"    MITRE: <ids> \u{2014} <name>\n"` / `None` → `"    MITRE: <ids> (unknown)\n"`. If empty, no MITRE line.

5. **`escape_for_terminal` invariant** (VP-012): all `summary` and `evidence` strings pass through `escape_for_terminal` at render time (not at key-construction time).

---

### Task 5 — Add `mod story_119` test block to `tests/reporter_terminal_tests.rs`

**File:** `tests/reporter_terminal_tests.rs`
**Helper to add:** `grouped_collapse_reporter()` — returns `TerminalReporter { use_color: false, show_hosts_breakdown: false, render: FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed } }`. This helper uses only variables in scope at the test-file level (not `main.rs`-scoped `*mitre`/`no_collapse`).

**Tests to author (one TDD Red step per test before implementing):**

| Test name | BC | What it verifies |
|-----------|-----|-----------------|
| `test_BC_2_11_030_mitre_alone_maps_to_grouped_collapsed` | BC-2.11.030 PC-2 | Construction site produces `{Grouped, Collapsed}` when `show_mitre_grouping=true, collapse_findings=true` |
| `test_BC_2_11_030_mitre_no_collapse_maps_to_grouped_expanded` | BC-2.11.030 PC-3 | Construction site produces `{Grouped, Expanded}` when `show_mitre_grouping=true, collapse_findings=false` |
| `test_BC_2_11_030_flat_routing_unchanged` | BC-2.11.030 PC-4,5 | `{Flat, Collapsed}` and `{Flat, Expanded}` routing unchanged |
| `test_BC_2_11_031_grouped_collapse_suffix_format` | BC-2.11.031 PC-1 | N=3 in one bucket → ` (x3)` suffix in bucket |
| `test_BC_2_11_031_singleton_no_suffix_in_bucket` | BC-2.11.031 PC-2 | Singleton finding in bucket → no `(xN)` suffix; byte-identical to `{Grouped, Expanded}` for that finding |
| `test_BC_2_11_031_grouped_collapse_color_ladder` | BC-2.11.031 PC-3 | N=2, `Likely+High` → suffix inside `red().bold()` span |
| `test_BC_2_11_031_cross_bucket_suffix_independence` | BC-2.11.031 PC-6 | Same key in two buckets → independent `(x3)` and `(x2)` counts; no cross-bucket merge |
| `test_BC_2_11_032_evidence_sampling_k3_in_bucket` | BC-2.11.032 PC-1 | N=5 in bucket, each with evidence → 3 evidence lines rendered |
| `test_BC_2_11_032_evidence_positional_no_slide` | BC-2.11.032 Invariant 2 | Group of N=5, members[0].evidence=[] → window does NOT slide; only members[1] and [2] evidence rendered (2 lines) |
| `test_BC_2_11_033_grouped_collapsed_preserves_bucket_order` | BC-2.11.033 PC-1 / VP-016 | Tactic bucket order per `all_tactics_in_report_order()` unchanged under `{Grouped, Collapsed}` |
| `test_BC_2_11_033_different_buckets_not_cross_collapsed` | BC-2.11.033 Invariant 3 | Two findings with same collapse key but different MITRE tactic → land in different buckets → two separate groups; no cross-bucket collapse |
| `test_BC_2_11_033_first_occurrence_in_sorted_bucket_order` | BC-2.11.033 PC-5/6 | Post-sort order determines group representative; `Likely` finding sorts before `Inconclusive` → becomes `members[0]` |
| `test_BC_2_11_033_uncategorized_last_under_grouped_collapse` | BC-2.11.033 PC-3 | `Uncategorized` bucket emitted last under `{Grouped, Collapsed}` |
| `test_BC_2_11_034_grouped_collapse_mitre_line_em_dash_format` | BC-2.11.034 PC-1 | N≥2 group MITRE line: em-dash + name from `members[0]` |
| `test_BC_2_11_034_unknown_technique_in_grouped_collapse` | BC-2.11.034 PC-1 | N≥2 group, `members[0]` has unknown technique ID → `(unknown)` format on MITRE line |
| `test_BC_2_11_034_suffix_not_on_mitre_line` | BC-2.11.034 PC-2 | `(xN)` suffix is absent from MITRE line |
| `test_BC_2_11_034_divergent_mitre_representative_sourcing` | BC-2.11.034 PC-3 | Divergent `mitre_techniques` across group members → only `members[0]` MITRE data appears in terminal output |
| `test_BC_2_11_025_grouped_mode_bypasses_flat_collapse` | BC-2.11.025 Invariant 5 (updated) | `render.grouping == Grouping::Grouped` with `Collapse::Collapsed` does NOT invoke the global flat-mode collapse pass; per-bucket pass applies instead |
| `test_BC_2_11_028_no_collapse_with_mitre_produces_grouped_expanded` | BC-2.11.028 PC-4 | `{Grouped, Expanded}` path (`--mitre --no-collapse`): no ` (xN)` suffix even for N=100 identical findings in a bucket |

> **Disambiguation note (BC-2.11.025 Invariant 5 VP-table row 6):** The new test `test_BC_2_11_025_grouped_mode_bypasses_flat_collapse` in `mod story_119` is DISTINCT from the pre-existing `test_BC_2_11_025_grouped_mode_bypasses_collapse` at `tests/reporter_terminal_tests.rs:2072`. The pre-existing test verifies the `{Grouped, Expanded}` suffix-free path (no `(xN)` suffix produced by the `--mitre --no-collapse` combination). The new test verifies the `{Grouped, Collapsed}` per-bucket invariant (BC-2.11.025 Invariant 5 — grouped-collapsed mode uses per-bucket `collapse_findings_pass_refs`, never the global flat `collapse_findings_pass` adapter). Both are referenced in BC-2.11.025's VP-table (row 6 maps to Invariant 5). The name resolves uniquely within `mod story_119`; the pre-existing test lives in a sibling `mod` block.

---

### Task 6 — Update `--no-collapse` doc-comment in `src/cli.rs`

**File:** `src/cli.rs`
**Scope:** The `///` doc-comment lines (approximately lines 154-159) on `no_collapse: bool` in `Commands::Analyze`. Clap derives the `--help` text from these `///` doc-comment lines; there is NO `help = "..."` attribute on this field — the field uses bare `#[arg(long)]`.
Replace the current single-mode-scoped comment with dual-scope text that covers both flat and grouped (--mitre) modes:
```rust
/// Disable collapsing of repeated findings in both flat and grouped
/// (--mitre) terminal output. By default, collapse is enabled in both
/// modes. When --mitre is used, collapse groups identical findings
/// within each MITRE tactic bucket with a `(xN)` count suffix.
/// Pass --no-collapse to restore one-line-per-finding output in both modes.
/// Has no effect on --output json or --output csv.
/// BC-2.11.028 precondition 2; dual-scope since STORY-119.
#[arg(long)]
no_collapse: bool,
```

---

### Task 7 — Comment sweep: remove stale enum-vocabulary references

**Files:** `src/reporter/terminal.rs`, `src/main.rs`, all test files.

Census: grep for `FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded`, `show_mitre_grouping` (in struct-field context), `collapse_findings` (in struct-field context). All occurrences must be either (a) migration sites already updated in Tasks 1-2, or (b) legitimate comments/doc-strings that have been updated to struct vocabulary.

**Exempt (do not modify):**
- `collapse_findings` as a local bool variable name inside `run_analyze` — this name is intentional (it is the in-scope param, not the old struct field).
- `show_mitre_grouping` as a local bool variable name inside `run_analyze` — same rationale.
- Historical BCs and ADR text outside `src/` and `tests/` — those are spec artifacts, not code.

**Additional sweep target (H-2):** The doc-comment on `render_findings_grouped` at `src/reporter/terminal.rs:429-430` reads:
```
/// Within each bucket, findings sort by verdict-desc, then
/// confidence-desc, then original emission order (stable).
```
This is WRONG. The code sorts ASCENDING (verdict_rank: Likely=0 first, then Possible=1, Inconclusive=2, Unlikely=3; confidence_rank: High=0 first, then Medium=1, Low=2). Task 7 MUST correct this comment to reflect ascending sort order. The corrected comment should read:
```
/// Within each bucket, findings sort ascending by verdict-rank
/// (Likely=0, Possible=1, Inconclusive=2, Unlikely=3), then ascending
/// by confidence-rank (High=0, Medium=1, Low=2), then original emission order (stable).
```

**Falsifiable requirement after sweep:**
1. `grep -rn "FindingsRender::Grouped\|FindingsRender::FlatCollapsed\|FindingsRender::FlatExpanded" src/ tests/` returns zero lines.
2. `grep -n "verdict-desc\|confidence-desc" src/reporter/terminal.rs` returns zero lines (stale H-2 comment corrected).

---

### Task 8 — Update `Cargo.toml` version to `0.9.0` (if not already done by STORY-120)

**File:** `Cargo.toml`
**Note:** Per F2 design note §7 (semver), STORY-119's `FindingsRender` struct reshape bundles into the unreleased v0.9.0 develop line alongside STORY-120. If STORY-120 already bumped `Cargo.toml` to `0.9.0`, no further version change is needed. Verify before touching. `cargo-semver-checks` will fire `struct_field_missing` on `FindingsRender` — this is expected and documented.

---

### Task 9 — Verify `cargo test --all-targets` green and compute input-hash

Run `cargo clippy --all-targets -- -D warnings` (must be clean). Run `cargo test --all-targets` (must be green). Run `cargo fmt --check` (must pass). Then run:
```
bin/compute-input-hash --write .factory/stories/STORY-119.md
bin/compute-input-hash .factory/stories/STORY-119.md
```
Record the computed hash in the `input-hash` frontmatter field.

---

## Previous Story Intelligence

**Predecessor:** STORY-120 (F4 complete and merged at commit f851995 on develop). Key applicable lessons:

1. **Variable scope in ACs:** `*mitre` and `no_collapse` are owned by `main()` (lines 79-80), NOT by `run_analyze`. All AC code blocks referencing the construction site use the in-scope params `show_mitre_grouping` (line 107) and `collapse_findings` (line 108) inside `run_analyze`.

2. **Verdict rank is 4-valued, ascending:** Likely=0, Possible=1, Inconclusive=2, Unlikely=3 (source-confirmed in `terminal.rs:447-454`). Within-bucket sort is ascending by rank value (lowest value = highest severity = sorted first). All BC citations on sort direction use this vocabulary.

3. **Forbidden dependencies:** `render_findings_grouped_collapsed` MUST NOT call `render_findings_collapsed` or `render_findings_flat` — paths must remain structurally separate (BC-2.11.032 Architecture Anchor: "grouped-collapse replaces this with a bounded K-sampled loop per bucket group").

4. **Vec accumulator is canonical:** `collapse_findings_pass_refs` returns `Vec<(CollapseKey, Vec<&Finding>)>` — an insertion-ordered accumulator with linear-scan `PartialEq` matching. Not a `HashMap`. Not an `IndexMap`. This is the canonical v0.8.0 structure (BC-2.11.025 Invariant 7 and Invariant 9). `collapse_findings_pass` (thin adapter) delegates to `collapse_findings_pass_refs` and thus returns the same type.

5. **Suffix before colorization:** The ` (xN)` suffix MUST be appended to the pre-color string before the color function is applied. Appending after the ANSI reset is NON-CONFORMANT (BC-2.11.031 PC-3 / BC-2.11.026 PC-6). STORY-120's code review found this pattern was applied correctly in `render_findings_collapsed`; replicate that exact pattern.

6. **Content-based citations in cross-indexes:** Use content-based citations (entry text) rather than line numbers when referencing the BC-INDEX, as changelog prepends cause line drift.

**From STORY-118** (flat-mode collapse predecessor, also applicable):
- `escape_for_terminal` is called at render time, not at key-construction time.
- The `COLLAPSE_EVIDENCE_SAMPLES = 3` constant at `terminal.rs:73` is shared; do not duplicate.
- Evidence lines: `"    > {}\n"` format with `escape_for_terminal` applied to each evidence string.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and ADR-0003:

1. **ADR-0003 Binding Rule 2:** `escape_for_terminal` must be called on every `summary` and `evidence` string before terminal output. No collapse path may bypass VP-012. Enforced at render time.

2. **ADR-0003 Binding Rule 4:** JSON and CSV reporters receive the complete, unmodified `&[Finding]` slice. No collapse pass upstream of multi-reporter dispatch. `render_findings_grouped_collapsed` is a private function of `TerminalReporter`; it is never called by `JsonReporter` or `CsvReporter`.

3. **ADR-0003 Binding Rule 5 (revised, STORY-119):** The `FindingsRender` type is now a struct-of-two-orthogonal-enums, not an enum. All four Cartesian combinations are valid. This BC is the outcome of the D-110 gate decision.

4. **Path separation invariant (BC-2.11.032):** `render_findings_grouped_collapsed` MUST NOT call `render_findings_collapsed` (flat-mode collapse path) or `render_finding_flat`. The grouped and flat paths must remain structurally separate. A cross-call would violate the per-bucket isolation contract.

5. **L4 Output layer constraint:** SS-11 (`reporter/terminal.rs`) must not import or call modules in L1 Ingest (SS-01/02) or L2 Stream (SS-04). `render_findings_grouped_collapsed` is purely a display-layer transform.

6. **No Default on FindingsRender/Grouping/Collapse:** Consistent with the deliberate omission in STORY-120 (ADR-0003 "Default Derive: Deliberate Omission"). All construction sites select both axes explicitly.

---

## Forbidden Dependencies

- **STORY-119 MUST NOT be dispatched to F4 before STORY-120 is merged and CI is green.** The struct-of-enums (`Grouping`, `Collapse`, `FindingsRender`) does not exist until STORY-120's `FindingsRender` enum is available to reshape. The F4 implementer builds against the post-STORY-120 codebase.

- **`render_findings_grouped_collapsed` MUST NOT import or call `render_findings_collapsed` or `render_findings_flat`.** Cross-path calls violate the per-bucket isolation contract (BC-2.11.032) and the structural separation invariant.

- **`render_findings_grouped` (the existing suffix-free function) MUST NOT be modified.** Its body is called for the `{Grouped, Expanded}` dispatch arm and remains byte-identical to v0.9.0.

- **`render_finding_grouped` MUST NOT be called for N≥2 groups.** `render_finding_grouped` is a single-finding renderer (AC-010): it is called ONLY for N=1 singleton groups within a bucket. For N≥2 groups, the header (with `(xN)` suffix), K=3-capped evidence lines, and the MITRE line (inline from `members[0]` with em-dash expansion) are ALL rendered inline in `render_findings_grouped_collapsed`. Calling `render_finding_grouped` for an N≥2 group would emit no `(xN)` suffix and uncapped evidence, violating AC-009, AC-014, and AC-024.

- **No new crate dependencies.** This story introduces no new entries in `Cargo.toml` beyond the existing dependencies (same library set as STORY-118 and STORY-120).

---

## Library & Framework Requirements

Version pins from `dependency-graph.md` (do not invent version numbers):

| Library | Version (from dep-graph) | Usage in this story |
|---------|--------------------------|---------------------|
| `owo-colors` | per `Cargo.toml` (same as STORY-118) | Color-ladder application: `red().bold()`, `yellow()`, `cyan()`, `dimmed()` |
| `etherparse` | 0.20 (per STORY-111 migration) | Packet decode — unchanged by this story |
| `std` `HashMap` | stdlib | Tactic bucket structure — same as `render_findings_grouped` |

No new crates. `CollapseKey`, `COLLAPSE_EVIDENCE_SAMPLES`, and `collapse_findings_pass` are shared existing symbols in `terminal.rs` — no duplication. `collapse_findings_pass_refs` is a new function introduced by this story (F4-new); it is not a new external dependency.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reporter/terminal.rs` | **Modify** | (1) Replace `FindingsRender` 3-variant enum with `Grouping` + `Collapse` + `FindingsRender` struct (Task 1). (2) Update 4-arm tuple dispatch in `TerminalReporter::render()` (Task 3). (3) Add `render_findings_grouped_collapsed` function (Task 4). (4) Comment sweep (Task 7). |
| `src/main.rs` | **Modify** | (1) Update `use` import to include `Collapse, Grouping`. (2) Rewrite `run_analyze` construction site (Task 2). (3) Rewrite `run_summary` site (Task 2). |
| `src/cli.rs` | **Modify** | Update `--no-collapse` help text for dual-scope (Task 6). |
| `tests/reporter_terminal_tests.rs` | **Modify** | (1) Migrate all `FindingsRender::Variant` sites to struct literals (Task 2). (2) Add `mod story_119` block with `grouped_collapse_reporter()` helper and all 19 tests (Task 5). |
| `tests/reporter_tests.rs` | **Modify** | Migrate all `FindingsRender::Grouped/FlatCollapsed/FlatExpanded` sites to struct literals (Task 2). |
| `tests/dnp3_f5_remediation_tests.rs` | **Modify** | Migrate `mitre_reporter` helper `FindingsRender::Grouped` site to struct literal (Task 2). |
| `tests/bc_2_09_100_multitag_tests.rs` | **Modify** | Migrate parameterized helper `FindingsRender` sites to struct literals (Task 2). |
| `Cargo.toml` | **Verify/modify** | Confirm version is `0.9.0`; bump if STORY-120 did not already do so (Task 8). |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `Grouping` enum (new) | `src/reporter/terminal.rs` | Pure data |
| `Collapse` enum (new) | `src/reporter/terminal.rs` | Pure data |
| `FindingsRender` struct (replaces 3-variant enum) | `src/reporter/terminal.rs` | Pure data |
| `render_findings_grouped_collapsed` (new function) | `src/reporter/terminal.rs` | Pure core |
| `match (self.render.grouping, self.render.collapse)` 4-arm dispatch (replaces 3-arm) | `src/reporter/terminal.rs` | Pure core |
| `run_analyze` construction site (struct literal) | `src/main.rs` | Effectful (CLI entry point) |
| `run_summary` construction site (struct literal) | `src/main.rs` | Effectful (CLI entry point) |

**Architecture Anchors (post-STORY-120 / pre-STORY-119 state at HEAD f851995):**
- `src/reporter/terminal.rs:100-111` — current `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` (Task 1 replacement target)
- `src/reporter/terminal.rs:202-224` — current `match self.render` 3-arm dispatch (Task 3 replacement target)
- `src/reporter/terminal.rs:432-483` — `render_findings_grouped` (Task 4 structural model; DO NOT MODIFY the function body)
- `src/reporter/terminal.rs:376-423` — `render_findings_collapsed` (flat-mode precedent for evidence loop and color ladder patterns)
- `src/reporter/terminal.rs:340-360` — `collapse_findings_pass` (thin adapter post-C-1; delegates to `collapse_findings_pass_refs`; grouped path calls `collapse_findings_pass_refs` directly per bucket)
- `src/reporter/terminal.rs` — `collapse_findings_pass_refs(&[&Finding])` (F4-new shared helper introduced by this story; implements the shared collapse logic; see ADR-0003 "Collapse-API Shape")
- `src/reporter/terminal.rs:73` — `COLLAPSE_EVIDENCE_SAMPLES = 3` (shared constant; not duplicated)
- `src/reporter/terminal.rs:311-327` — `render_finding_grouped` (called for N=1 singletons; unchanged)
- `src/reporter/terminal.rs:391` — color ladder in `render_findings_collapsed` (normative reference for suffix-in-pre-color-string pattern; `:391` is the color-selection block entry point per BC-2.11.031 PC-3)
- `src/main.rs:381-387` — current 3-arm if-expression for `FindingsRender` construction (Task 2 replacement target)
- `src/main.rs:107` — `show_mitre_grouping: bool` in-scope param in `run_analyze`
- `src/main.rs:108` — `collapse_findings: bool` in-scope param in `run_analyze`

---

## Token Budget Estimate

| Context item | Estimated tokens |
|-------------|-----------------|
| This story file (STORY-119.md) | ~6,000 |
| BC files (12 BCs × ~1,800 avg) | ~21,600 |
| `src/reporter/terminal.rs` (current, ~500 lines) | ~8,000 |
| `src/main.rs` (current, ~550 lines) | ~8,800 |
| All test files combined (reporter_terminal_tests.rs ~2500 lines + reporter_tests.rs ~600 lines + others) | ~20,000 |
| F2 design note (story-119-type-design.md) | ~3,500 |
| F1 delta-analysis | ~5,000 |
| Tool outputs (grep, cargo test) | ~3,000 |
| **Total estimated** | **~75,900** |

**Assessment:** ~75,900 tokens ≈ 30% of an agent context window (250k tokens). Within the 20-30% guideline — borderline. To stay within budget, the F4 implementer should load BC files on-demand (only the BC for the specific AC being implemented) rather than all 12 at once. The story does not require splitting given the 8-point scope.

---

## Dependencies

- `depends_on: [STORY-120]` — STORY-120 introduces the three-variant `FindingsRender` enum (`FindingsRender::Grouped`, `::FlatCollapsed`, `::FlatExpanded`); STORY-119 evolves that enum into the struct-of-orthogonal-enums and depends on STORY-120 because the type must exist first. STORY-119's implementer dispatches on `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` — a state that was unrepresentable under the prior three-variant enum. STORY-120 is the direct predecessor (wave 48). STORY-118's collapse infrastructure (`CollapseKey`, `collapse_findings_pass`, `COLLAPSE_EVIDENCE_SAMPLES`) is transitively available through STORY-120's dependency chain. STORY-119 cannot be built before STORY-120 ships.
- `blocks: []` — No downstream stories currently depend on grouped-mode collapse.

---

## Edge Cases Specific to This Story

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single finding in a tactic bucket (N=1) | `render_finding_grouped` called directly; no `(xN)` suffix; output byte-identical to `{Grouped, Expanded}` for that finding |
| EC-002 | `{Grouped, Expanded}` path with N=100 identical-key findings in a bucket | 100 individual lines; zero `(xN)` suffixes; `render_findings_grouped` unchanged |
| EC-003 | Same collapse key present in two different tactic buckets | Each bucket forms its own independent group; no cross-bucket merge; `(x3)` in bucket A and `(x2)` in bucket B independently |
| EC-004 | Group of N=5, `members[0].evidence=[]`, other members have evidence | Positional window: inspects members[0], [1], [2]. member[0] contributes 0 lines (no sliding). members[1] and [2] contribute 1 line each. Total = 2 evidence lines |
| EC-005 | Multi-tag finding `mitre_techniques=["T1692.001","T0836"]` in collapsed group | Bucket assignment: T1692.001's tactic. MITRE line: `    MITRE: T1692.001, T0836 \u{2014} <name_of_T1692.001>\n` from members[0] |
| EC-006 | `members[0].mitre_techniques=[]` in collapsed N≥2 group | No MITRE line rendered for the group; header + evidence only |
| EC-007 | All findings in a bucket collapse to a single group | One bucket header + one collapsed group header with `(x<N>)` suffix; bucket header still emitted |
| EC-008 | Two findings same key, different verdict ranks (Likely vs Inconclusive), in same bucket | Post-sort: Likely (rank=0) sorts first → becomes `members[0]` (group representative). `(x2)` suffix. Representative's fields used for header rendering. |
| EC-009 | `Uncategorized` bucket with collapsible findings | Collapse applied within Uncategorized bucket per same rules; `Uncategorized` header emitted last |

---

## Changelog

- **v1.0 (STORY-119 initial creation, 2026-06-17):** Created as F2 de-stale stub; F1/F2 complete; ACs/tasks deferred to F3.
- **v1.1 (F2 round-2 de-stale):** Struct vocabulary, full 12-BC set, current anchors, deferred flag removed; full AC/task decomposition pending F3.
- **v1.2 (F2 round-3 remediation):** BC table role-descriptions rewritten to faithfully reflect each BC's actual contract; version column corrected (030 v1.2, 032 v1.3, 034 v1.3); changelog note updated to full 12-BC set; VP comment added.
- **v1.3 (F2 round-4 remediation):** Corrected type-introduction attribution throughout — STORY-120 introduces the three-variant FindingsRender ENUM; STORY-119 evolves it into the struct-of-orthogonal-enums. Fixed "9-BC" → "12-BC" in v1.1 stanza.
- **v1.4 (F2 round-5 remediation):** BC-2.11.030 body-table version cell corrected v1.2 → v1.4 to match live BC file. All other BC version stamps confirmed correct.
- **v1.5 (F3 full decomposition, 2026-06-18):** Full acceptance criteria (AC-001..AC-031) each traced verbatim to governing BC postcondition/invariant. Full implementation tasks (Tasks 1-9): struct reshape (~46 sites), render_findings_grouped_collapsed implementation, CLI struct-construction wiring, comment sweep. VP assignments (VP-012, VP-016). Wave assigned (wave: 49 = max(STORY-120=48)+1). Inputs list populated. Deferred markers removed. CARRY-119-F3-RESIDUALS-001 fixes: VP-table test anchor renamed `test_BC_2_11_033_grouped_collapsed_preserves_bucket_order` (was mis-prefixed `test_BC_2_11_013_...` in F2 round per BC-2.11.033 Verification Properties); spec-changelog NIT applied (`Collapse::Expanded` corrected in BC-2.11.030 v1.2 stanza — closed round historical entry; no normative change).
- **v1.6 (F3 adversarial round-1 remediation, 2026-06-18):** BC stamps synced to PO-final: BC-2.11.014→v2.0, BC-2.11.025→v1.12, BC-2.11.027→v1.7, BC-2.11.031→v1.3, BC-2.11.032→v1.4, BC-2.11.033→v1.3. C-1: collapse API shape propagated — `collapse_findings_pass_refs(&[&Finding])` is the F4-new shared helper; `collapse_findings_pass` at :340 becomes a thin adapter delegating to it; grouped caller collects `bucket_refs` and calls `collapse_findings_pass_refs` directly; AC-027, Task 4 item 3, and Architecture Anchors updated; ADR-0003 "Collapse-API Shape" and F2 design-note §5.2.1 referenced. H-2: Task 7 and AC-030 gain explicit falsifiable sweep target for stale `verdict-desc, confidence-desc` doc-comment at `terminal.rs:429-430` (WRONG — sort is ASCENDING; correction specified). M-1: Task 6 and AC-030 updated — `no_collapse` field uses bare `#[arg(long)]` with `///` doc-comment lines, NOT a `help = "..."` attribute; task now instructs editing those doc-comment lines. M-2: Forbidden Dependencies gains `render_finding_grouped` N≥2 prohibition (calling it for N≥2 groups violates AC-009/AC-014/AC-024). MEDIUM-1: explicit governing-BC trace anchors added for four orphan BCs: AC-019 → BC-2.11.014 Invariant 1 (verbatim verdict-rank enumeration); AC-009 → BC-2.11.026 Precondition 1; AC-014 → BC-2.11.027 Precondition 1; AC-010 → BC-2.11.016 Precondition 1. All trace descriptions copied verbatim from live BC files (PG-62-F3-AC-DESC-FROM-SOURCE). AC-019/AC-020 `collapse_findings_pass` references updated to `collapse_findings_pass_refs` in AC body text. BC-033 body-table description updated to reference `collapse_findings_pass_refs`.
- **v1.7 (F3 adversarial round-2 remediation, 2026-06-18):** Pass B trace-quote symbol fixes: AC-019/AC-020/AC-027 verbatim trace quotes updated `collapse_findings_pass` → `collapse_findings_pass_refs` to match BC-2.11.033 PC-5/PC-6/Invariant 3 source text exactly (retired symbol was still quoted in the parenthetical trace citations). Pass A M-1 site census corrected from ~46 to 84 (grepped ground-truth: main.rs=4, terminal.rs=3, reporter_terminal_tests.rs=55, reporter_tests.rs=17, dnp3_f5_remediation_tests.rs=2, bc_2_09_100_multitag_tests.rs=3); AC-007 and Task 2 header updated to 84; Task 2 per-file descriptors corrected (reporter_terminal_tests.rs=55, reporter_tests.rs=17). Pass A L-1 color-ladder anchor corrected terminal.rs:391 → terminal.rs:392-400 in AC-011, Task 4 item 3b, and Architecture Anchors (line 391 is `let colored =`; ladder match arms are :392-400).
- **v1.8 (F3 adversarial round-3 remediation, 2026-06-18):** Fix 1 (Pass B H-1): reverted AC-011 color-ladder anchor from `terminal.rs:392-400` back to `terminal.rs:391` to match BC-2.11.031 PC-3 verbatim — the quote in the trace parenthetical must be byte-identical to the source BC; prose outside the quote uses "color-selection block beginning at `terminal.rs:391`" to pre-empt off-by-one confusion; same revert applied to Task 4 item 4b and Architecture Anchors entry. Fix 2 (Pass A H-2): Task 2 `src/main.rs` per-file descriptor corrected from "2 sites" to "4 enum-variant occurrences across 2 logical construction sites" with line references (:382/:384/:386 collapsing to one struct literal; :449 `run_summary`) — aligns AC-007 census (84 total) with zero-grep acceptance gate. Fix 3 (Pass A H-1/Pass B L-1): Task 5 gains disambiguation note distinguishing new `test_BC_2_11_025_grouped_mode_bypasses_flat_collapse` (mod story_119, verifies Invariant 5 per-bucket path) from pre-existing `test_BC_2_11_025_grouped_mode_bypasses_collapse` at tests/reporter_terminal_tests.rs:2072 (verifies {Grouped, Expanded} suffix-free path); both referenced in BC-2.11.025 VP-table row 6. Fix 5 (PO-final BC-2.11.025 v1.13): BC-table stamp BC-2.11.025 v1.12 → v1.13.
