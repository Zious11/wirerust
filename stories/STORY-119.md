---
document_type: story
story_id: STORY-119
epic_id: E-18
version: "2.2"
status: pending
producer: story-writer
timestamp: 2026-06-18T00:00:00Z
phase: f3
points: 5
priority: P2
depends_on: [STORY-122]
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
wave: 50
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
# BC status: all 12 BCs authored/converged at F2 (2026-06-18). PO-final versions:
#   013 v1.15, 014 v2.1, 016 v1.10, 025 v1.14, 026 v1.14, 027 v1.8, 028 v1.10,
#   030 v1.5, 031 v1.4, 032 v1.5, 033 v1.4, 034 v1.4.
#   All BCs are F2-frozen; normative bodies must not be edited as part of STORY-119/B F4 implementation.
# Subsystem anchor: SS-11 owns this story's scope because grouped-mode collapse is a
#   display-layer extension of reporter/terminal.rs — the core SS-11 module.
# Dependency anchor:
#   depends_on: [STORY-122] — STORY-122/A reshapes `FindingsRender` from the three-variant enum
#   into the struct-of-orthogonal-enums (FindingsRender { grouping: Grouping, collapse: Collapse })
#   and establishes the four-arm dispatch with the TEMPORARY {Grouped,Collapsed}→render_findings_grouped
#   arm. STORY-119/B repoints that arm to render_findings_grouped_collapsed, which cannot exist
#   before STORY-122's struct-of-enums and four-arm dispatch are in place.
#   blocks: [] — No downstream stories depend on grouped-mode collapse.
# Wave anchor: wave 50 = max(wave(STORY-122)=49) + 1. STORY-122 is the unique predecessor.
# Split rationale: D-120 (human-confirmed 2026-06-18) splits monolithic STORY-119 v1.12 into:
#   A = STORY-122: enum→struct reshape + 84-site migration (byte-identical, wave 49).
#   B = STORY-119 (this story): grouped-collapse render path + CLI flip (net-new behavior, wave 50).
# Re-scope from v1.12: removed struct-reshape/84-site-migration ACs (AC-005, AC-007) and the
#   byte-identical-grouped-expanded AC (AC-028 as formerly numbered). Removed dispatch-existence
#   AC (AC-006 as formerly numbered — dispatch is STORY-122's scope). Kept all behavioral ACs:
#   CLI mapping (AC-001..004), render path (AC-008..027), flat-paths-unchanged (AC-029),
#   comment sweep (AC-030 updated), test green (AC-031). ACs renumbered sequentially.
input-hash: "4a8c93f"
---

# STORY-119 (B): Terminal Finding-Collapse — Grouped Mode / --mitre (render path + CLI flip)

## Narrative

- **As a** network security analyst using `wirerust analyze --mitre`
- **I want** repeated identical findings within the same MITRE tactic bucket to be
  collapsed with a ` (xN)` count suffix, with K=3 evidence sampling, and `--no-collapse`
  as the dual-scope opt-out that suppresses collapse in both flat and grouped modes
- **So that** I can quickly assess the volume of repeated findings per tactic without
  wading through thousands of identical grouped-mode lines, while retaining access to
  the pre-collapse per-finding view via `--no-collapse`

**Scope (B — net-new behavioral delta only):** STORY-119/B builds on top of STORY-122/A, which
has already reshaped `FindingsRender` into the struct-of-orthogonal-enums and established the
four-arm dispatch with a TEMPORARY `{Grouped, Collapsed}` → `render_findings_grouped` arm.
STORY-119/B:
1. Introduces `collapse_findings_pass_refs(&[&Finding])` — the F4-new shared collapse-logic helper.
2. Makes `collapse_findings_pass` a thin adapter delegating to it (using the `findings` PARAMETER,
   not any nonexistent `self.findings` field).
3. Implements `render_findings_grouped_collapsed` — per-tactic-bucket deduplication with `(xN)`
   count suffix, K=3 evidence sampling, em-dash MITRE line from `group_members[0]`.
4. Repoints the `{Grouped, Collapsed}` dispatch arm from `render_findings_grouped` (STORY-122/A
   TEMPORARY) to the new `render_findings_grouped_collapsed`.
5. Flips the `--mitre` CLI default to `{Grouped, Collapsed}` (collapse by default) by replacing the
   3-arm if construction (STORY-122/A's byte-identical form) with the orthogonal 2-if struct wiring at
   the `run_analyze` construction site in `src/main.rs` (Task 4). This IS a code change to the
   construction site — it is the owner of the CLI flip under Option X.
   `--no-collapse` becomes dual-scope (suppresses collapse in BOTH grouped and flat modes).
6. Updates the `--no-collapse` doc-comment in `src/cli.rs` for dual-scope.

**What this story does NOT do:**
- Does NOT reshape `FindingsRender` or migrate construction sites (STORY-122/A's scope).
- Does NOT introduce the `Grouping`/`Collapse` enums or `FindingsRender` struct (STORY-122/A).
- Does NOT add a new test module for struct-reshape tests (STORY-122/A's scope).

**Behavior change:** `--mitre` alone now produces `{Grouped, Collapsed}` (grouped output with
per-bucket collapse). In v0.9.0, `--mitre` produced `{Grouped, Expanded}` (old behavior).
The pre-collapse behavior is preserved exactly via `--mitre --no-collapse`, which produces
`{Grouped, Expanded}`. This change was approved at the F1 gate (D-110). `--no-collapse` is
now dual-scope: it suppresses collapse in both flat and grouped modes.

---

## Behavioral Contracts

All 12 BCs governing this story are authored and F2-frozen.

| BC | Version | Role for STORY-119/B |
|----|---------|---------------------|
| BC-2.11.013 | v1.15 | Tactic-header structure and collapse axis: when `render.grouping == Grouping::Grouped` with `Collapse::Collapsed`, per-bucket collapse applies; with `Collapse::Expanded`, suffix-free. Tactic-bucket ordering and header format unchanged. |
| BC-2.11.014 | v2.1 | Within-bucket sort — ascending by verdict-rank (Likely=0, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank (High=0, Medium=1, Low=2), then emission-index — determines the post-sort group representative (members[0]). |
| BC-2.11.016 | v1.10 | Grouped MITRE em-dash+name line format — em-dash + technique name for known IDs, `(unknown)` for unrecognized; applies to both `{Grouped, Collapsed}` and `{Grouped, Expanded}` paths; consumed by BC-2.11.034 for collapsed N≥2 groups. |
| BC-2.11.025 | v1.14 | Flat-mode collapse key and first-occurrence order: the `(category, verdict, confidence, summary)` four-tuple key and `Vec<(CollapseKey, Vec<&Finding>)>` accumulator structure; `collapse_findings_pass` (flat-mode wrapper) delegates to `collapse_findings_pass_refs`; grouped per-bucket collapse calls `collapse_findings_pass_refs` directly. Scoped to `Grouping::Flat`. |
| BC-2.11.026 | v1.14 | Flat-mode `(xN)` suffix rule and color-ladder requirement; grouped analogue is BC-2.11.031. PC-4 suffix-free guarantee now scoped to `{Grouped, Expanded}` only. |
| BC-2.11.027 | v1.8 | Flat-mode K=3 evidence sampling — positional no-sliding-window; grouped analogue is BC-2.11.032. |
| BC-2.11.028 | v1.10 | `--no-collapse` opt-out flag: dual-scope since STORY-119/B — disables collapse in both flat and grouped modes. Wiring uses struct construction `FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`. |
| BC-2.11.030 | v1.5 | CLI→render mode mapping: `--mitre` alone → `{Grouped, Collapsed}` (new default, STORY-119/B); `--mitre --no-collapse` → `{Grouped, Expanded}` (pre-STORY-119 `--mitre` behavior). |
| BC-2.11.031 | v1.4 | Per-bucket `(xN)` count suffix: N≥2 group within a tactic bucket renders header with ` (xN)` suffix before colorization; N=1 singleton renders via `render_finding_grouped` without suffix. Color-ladder requirement identical to BC-2.11.026. |
| BC-2.11.032 | v1.5 | Per-bucket K=3 evidence sampling: first `min(N, K)` members positionally; `evidence[0]` from each if non-empty; window does NOT slide past empty-evidence members. |
| BC-2.11.033 | v1.4 | Tactic-bucket ordering invariant under grouped-collapse: bucket sequence per `all_tactics_in_report_order()` is unchanged; `Uncategorized` last; collapse occurs strictly within buckets. Sort-then-collapse ordering: per-bucket sort (ascending verdict/confidence/emission-index rank) PRECEDES `collapse_findings_pass_refs` for that bucket. |
| BC-2.11.034 | v1.4 | MITRE line format for N≥2 collapsed groups: em-dash name expansion sourced from `group_members[0]`; no `(xN)` suffix on MITRE line; unknown ID → `(unknown)` format. Singletons use `render_finding_grouped` (BC-2.11.016 governs). |

---

## Acceptance Criteria

### AC-001 — `--mitre` alone routes to `{Grouped, Collapsed}` at the construction site
The `TerminalReporter` construction site in `src/main.rs::run_analyze` uses the struct
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

### AC-005 — `render_findings_grouped_collapsed` function exists and is dispatched for `{Grouped, Collapsed}`
A new function `render_findings_grouped_collapsed` exists in `src/reporter/terminal.rs`. It is called by the `(Grouping::Grouped, Collapse::Collapsed)` arm of the four-arm tuple dispatch (replacing the TEMPORARY `render_findings_grouped` call from STORY-122/A). The function handles tactic bucketing and sorting identically to `render_findings_grouped` (BC-2.11.013), then applies a per-bucket collapse pass before rendering.
(traces to BC-2.11.031 Architecture Anchors: "`render_findings_grouped_collapsed` — F4-pending new function: per-bucket collapse + grouped-collapse header rendering".)

### AC-006 — per-bucket collapse: N≥2 group within a bucket renders header with `(xN)` suffix
For a group of N≥2 findings sharing the same `(category, verdict, confidence, summary)` key within a MITRE tactic bucket under `{Grouped, Collapsed}`, the header line reads: `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary> (x<N>)\n` where `<N>` is the exact decimal integer count (no leading zeros, no space between `x` and `N`).
(traces to BC-2.11.031 Postcondition 1: "For a group of N≥2 within a tactic bucket: the header line reads: `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary> (x<N>)\n`…")
(governing-BC trace: BC-2.11.026 Precondition 1: "`TerminalReporter.render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` (`{Flat, Collapsed}` — the flat-mode collapsed path). The (xN) suffix rule defined in this BC applies to the flat-mode path. The grouped-mode analogue is BC-2.11.031." — this AC covers the grouped-mode analogue defined in BC-2.11.031; BC-2.11.026 governs the canonical suffix-format contract that BC-2.11.031 mirrors.)

### AC-007 — per-bucket collapse: singleton (N=1) within a bucket renders via `render_finding_grouped` with no suffix
For a singleton group (N=1 within a bucket) under `{Grouped, Collapsed}`, `render_finding_grouped` is called for that finding. The output is byte-identical to the `{Grouped, Expanded}` path for the same finding — no count suffix, MITRE name expansion per BC-2.11.016.
(traces to BC-2.11.031 Postcondition 2: "For a singleton group (N=1 within a bucket): `render_finding_grouped` is called for that finding. The output is byte-identical to the `{Grouped, Expanded}` path for the same finding — no count suffix, MITRE name expansion per BC-2.11.016.")
(governing-BC trace: BC-2.11.016 Precondition 1: "`TerminalReporter.render.grouping == Grouping::Grouped` (applies to both `{Grouped, Collapsed}` and `{Grouped, Expanded}` paths; the em-dash MITRE expansion occurs on all `render_finding_grouped` calls regardless of collapse axis)." — the singleton path calls `render_finding_grouped` which is governed by BC-2.11.016 for its MITRE em-dash line.)

### AC-008 — color-ladder requirement: `(xN)` suffix is part of the pre-colorization string
The grouped-collapse header path applies the same verdict/confidence color-ladder as the color-selection block beginning at `terminal.rs:391` to a pre-color string that ALREADY INCLUDES the ` (xN)` suffix. The ladder: `Likely+High` → `red().bold()`; `Likely+other` → `yellow`; `Possible` → `yellow`; `Inconclusive` → `cyan`; `Unlikely` → `dimmed`. Appending the suffix AFTER the ANSI color-reset sequence is NON-CONFORMANT.
(traces to BC-2.11.031 Postcondition 3: "The grouped-collapse header path MUST apply the same verdict/confidence color-selection logic as `terminal.rs:391` to a pre-color string that ALREADY INCLUDES the ` (xN)` suffix… appending the suffix after the ANSI reset is NON-CONFORMANT.")

### AC-009 — `(xN)` suffix does NOT appear on the MITRE line, evidence lines, or tactic bucket headers
The ` (xN)` suffix appears ONLY on the finding-group header line. It must not appear on the MITRE line (`    MITRE: <ids> \u{2014} <name>`), on any `    > <evidence>` line, or on the tactic bucket header (`  ## <TacticName>`).
(traces to BC-2.11.031 Postcondition 4: "The ` (xN)` suffix MUST NOT appear on the MITRE line, any evidence line, or the tactic bucket header (`## <TacticName>`). It appears only on the finding-group header line.")

### AC-010 — cross-bucket suffix independence: same key in different buckets produces independent counts
Two groups with the same `(category, verdict, confidence, summary)` collapse key but in different MITRE tactic buckets each emit their own independent `(xN)` suffix based on their own per-bucket group count. A group of 3 in bucket A and a group of 2 in bucket B produce `(x3)` and `(x2)` respectively; they are never merged to `(x5)`.
(traces to BC-2.11.031 Postcondition 6: "Cross-bucket suffix independence: two groups with the same collapse key in different tactic buckets each emit their own independent `(xN)` suffix based on their own bucket group count.")

### AC-011 — per-bucket evidence sampling: at most K=3 evidence lines per N≥2 group
For a collapsed group of N≥2 within a tactic bucket under `{Grouped, Collapsed}`, the terminal output contains at most K=3 evidence lines (`COLLAPSE_EVIDENCE_SAMPLES = 3`), each rendered as `    > <escaped_evidence_line>\n`. Evidence lines are drawn from the FIRST `min(N, K)` members in post-sort bucket order; for each inspected member, `evidence[0]` is emitted if non-empty; if a member has an empty evidence vec it contributes 0 lines and the window does NOT slide past it.
(traces to BC-2.11.032 Postconditions 1–2 and Invariant 2: "The terminal output for a grouped-collapse group of N≥2 contains at most K=3 evidence lines… Evidence lines are drawn from the FIRST `min(N, K)` members of the per-bucket group… The window does NOT slide past empty-evidence members.")
(governing-BC trace: BC-2.11.027 Precondition 1: "`TerminalReporter.render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` (`{Flat, Collapsed}` — the flat-mode collapsed path). The grouped-mode analogue for per-bucket evidence sampling is BC-2.11.032." — this AC covers the grouped-mode analogue BC-2.11.032; BC-2.11.027 governs the canonical K=3 positional-sampling contract that BC-2.11.032 mirrors.)

### AC-012 — per-bucket evidence sampling: no "N-K more" elision annotation
When the group has N>K findings within the bucket, exactly K evidence lines appear (assuming selected members have non-empty evidence). No elision marker, annotation, or "and N-K more evidence lines" indicator is rendered. The `(xN)` header suffix is the only indicator of group size.
(traces to BC-2.11.032 Postcondition 4: "When the group has N>K findings within the bucket, exactly K evidence lines appear… No "N-K more evidence lines elided" annotation or similar indicator is emitted.")

### AC-013 — tactic-bucket ordering invariant: bucket sequence per `all_tactics_in_report_order()` unchanged
Under `{Grouped, Collapsed}`, tactic bucket headers appear in the order returned by `all_tactics_in_report_order()` — identical to BC-2.11.013 Postcondition 2. A tactic bucket header is emitted only when at least one finding belongs to that bucket (empty-bucket guard per BC-2.11.013 PC-3). Collapse within a bucket does not prevent the bucket header from being emitted.
(traces to BC-2.11.033 Postconditions 1–2: "Tactic bucket headers appear as `  ## <TacticName>\n` in the output, in the order returned by `all_tactics_in_report_order()` — identical to BC-2.11.013 PC-2. A tactic bucket header is emitted only when at least one finding belongs to that bucket…")

### AC-014 — `Uncategorized` bucket emitted last under `{Grouped, Collapsed}`
The `Uncategorized` bucket header (`  ## Uncategorized\n`) appears last among emitted buckets, collecting findings where `mitre_techniques` is empty OR where `technique_tactic(mitre_techniques[0])` returns `None` — identical to BC-2.11.013 PC-4.
(traces to BC-2.11.033 Postcondition 3: "The `Uncategorized` bucket header (`  ## Uncategorized\n`) appears last among emitted buckets, collecting findings where `mitre_techniques` is empty OR where `technique_tactic(mitre_techniques[0])` returns `None` — identical to BC-2.11.013 PC-4.")

### AC-015 — bucket membership unchanged by collapse
A finding assigned to bucket B under `{Grouped, Expanded}` is assigned to the same bucket B under `{Grouped, Collapsed}`. The collapse key `(category, verdict, confidence, summary)` is orthogonal to bucket assignment (which uses `mitre_techniques[0]`). The per-bucket collapse pass does NOT reassign findings to different buckets.
(traces to BC-2.11.033 Postcondition 4: "Bucket membership is unchanged by collapse. A finding assigned to bucket B under `{Grouped, Expanded}` is assigned to the same bucket B under `{Grouped, Collapsed}`.")

### AC-016 — sort-then-collapse ordering: per-bucket sort precedes `collapse_findings_pass_refs`
Within each tactic bucket, findings are sorted ascending by rank — verdict-rank ascending (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank ascending (High=0 first, Medium=1, Low=2), then emission-index ascending — BEFORE `collapse_findings_pass_refs` is applied to that bucket's slice. The group representative `members[0]` is therefore the first finding in the sorted bucket order, not first in the original global emission order.
(traces to BC-2.11.033 Postcondition 5: "Within each bucket, findings are sorted ascending by rank — verdict-rank ascending (Likely=0 first, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank ascending (High=0 first, Medium=1, Low=2), then emission-index ascending — BEFORE the per-bucket `collapse_findings_pass_refs` is applied.")
(governing-BC trace: BC-2.11.014 Invariant 1: "Verdict ranks: Likely=0, Possible=1, Inconclusive=2, Unlikely=3 (defined by local `verdict_rank` function in terminal.rs:447-454; source-confirmed match arms).")

### AC-017 — group order within bucket is first-occurrence in SORTED bucket order
The per-bucket `collapse_findings_pass_refs` produces groups whose order within the bucket is first-occurrence in the SORTED bucket order (not first-occurrence in the global emission order). Two findings with the same key in a bucket: the one with lower rank value (higher severity) sorts first and becomes `members[0]` (group representative).
(traces to BC-2.11.033 Postcondition 6: "The per-bucket `collapse_findings_pass_refs` produces groups whose order within the bucket is first-occurrence in the SORTED bucket order (not first-occurrence in the global emission order). This is the 'post-sort first-occurrence' definition for grouped-collapse mode.")

### AC-018 — MITRE line for N≥2 collapsed group: em-dash name expansion sourced from `members[0]`
For a collapsed group of N≥2 in a tactic bucket, the MITRE line is rendered from `group_members[0].mitre_techniques`: if non-empty, the format is `    MITRE: <ids_joined> \u{2014} <name>\n` (known ID) or `    MITRE: <ids_joined> (unknown)\n` (unrecognized ID). If `group_members[0].mitre_techniques` is empty, no MITRE line is rendered. The IDs are joined as `mitre_techniques.join(", ")`. The separator is U+2014 (EM DASH), not ASCII `--`.
(traces to BC-2.11.034 Postcondition 1: "For a collapsed group of N≥2 in a tactic bucket: after the header line and evidence lines, the MITRE line is rendered from `group_members[0].mitre_techniques`… if known, the line reads `    MITRE: <ids_joined> \u{2014} <name>\n`; if unknown, the line reads `    MITRE: <ids_joined> (unknown)\n`. The separator is U+2014 (EM DASH).")

### AC-019 — `(xN)` suffix does NOT appear on the MITRE line for N≥2 groups
The `(xN)` count suffix does not appear on the MITRE line for N≥2 collapsed groups. The count suffix is scoped to the header line only (AC-009).
(traces to BC-2.11.034 Postcondition 2: "The `(xN)` count suffix does NOT appear on the MITRE line. The count suffix is scoped to the header line only.")

### AC-020 — other group members' `mitre_techniques` are elided from terminal output; preserved in JSON/CSV
Other group members' `mitre_techniques` (members[1], members[2], …, members[N-1]) are elided from terminal output. Their technique data is preserved unmodified in the raw `findings` slice available to JSON/CSV reporters (BC-2.11.029).
(traces to BC-2.11.034 Postcondition 3: "Other group members' `mitre_techniques` (members[1], members[2], ..., members[N-1]) are elided from terminal output. Their technique data is preserved unmodified in the raw `findings` slice available to JSON/CSV reporters.")

### AC-021 — observable output block order within a collapsed group
For a collapsed N≥2 group, the output order is: (1) header line with `(xN)` suffix, (2) up to K=3 evidence lines (`    > <evidence>`), (3) MITRE line from `members[0]` (if `mitre_techniques` non-empty). The `(xN)` suffix appears ONLY in item (1).
(traces to BC-2.11.034 Postcondition 5: "The observable MITRE line order: within a collapsed group's output block, the order is: (1) header line with `(xN)` suffix, (2) up to K=3 evidence lines (BC-2.11.032), (3) MITRE line from `members[0]` (this PC). The `(xN)` suffix appears ONLY on (1).")

### AC-022 — `{Grouped, Expanded}` path (`--mitre --no-collapse`): zero `(xN)` suffixes even for large N
Under `render = {Grouped, Expanded}`, N=100 identical-key findings in a tactic bucket produce 100 individual finding lines with no ` (xN)` suffix on any line. The `{Grouped, Expanded}` suffix-free guarantee (BC-2.11.013 Invariant 4, pre-STORY-119 `--mitre` behavior) is unchanged and the existing test `test_BC_2_11_013_grouped_mode_suffix_free` continues to pass without modification.
(traces to BC-2.11.013 Invariant 4: "When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (`--mitre --no-collapse`): the collapse pass is NOT applied… The OBSERVABLE GUARANTEE holds: no ` (xN)` suffix appears in the terminal output for any finding, at any input volume.")

### AC-023 — `escape_for_terminal` called on all summary and evidence strings in the grouped-collapse path
All `summary` and `evidence` strings in `render_findings_grouped_collapsed` pass through `escape_for_terminal` before being written to the output buffer. The collapse pass operates on raw (unescaped) `Finding` field values; escape is render-time, not key-time. VP-012 invariant is preserved.
(traces to BC-2.11.031 Precondition 5: "`escape_for_terminal` has been applied to the group representative's `summary` field before the suffix is appended (VP-012 invariant; BC-2.11.010).")

### AC-024 — `collapse_findings_pass_refs` called once per tactic bucket slice (not across global findings)
In `render_findings_grouped_collapsed`, `collapse_findings_pass_refs(&[&Finding])` (the F4-new shared helper) is called once per bucket's per-bucket sorted slice, not once for the global `findings` slice. `collapse_findings_pass` at `:340` becomes a thin adapter: it collects `findings.iter().collect()` (the `&[Finding]` PARAMETER, NOT any `self.findings` phantom field) and delegates to `collapse_findings_pass_refs`. The adapter body is `let refs: Vec<&Finding> = findings.iter().collect(); self.collapse_findings_pass_refs(&refs)`. The grouped caller collects `bucket_refs: Vec<&Finding> = items.iter().map(|(_, f)| *f).collect()` then calls `collapse_findings_pass_refs(&bucket_refs)`. The collapse LOGIC is shared/reused via `collapse_findings_pass_refs`; the exact original `collapse_findings_pass` signature is not called from the grouped path. There is no cross-bucket collapse pass. Reference: ADR-0003 "Collapse-API Shape" subsection and F2 design-note §5.2.1.
(traces to BC-2.11.033 Invariant 3: "The per-bucket collapse pass is applied to the sorted-bucket slice for each tactic bucket independently and sequentially in tactic-order. There is no global cross-bucket collapse pass; `collapse_findings_pass_refs` never receives the full global `findings` slice in grouped mode.")
(governing-BC trace: BC-2.11.025 Invariant 5: "This BC and its invariants are scoped to `Grouping::Flat` (the `{Flat, Collapsed}` path)… The grouped-mode collapse (`{Grouped, Collapsed}`) is a distinct per-bucket invocation of `collapse_findings_pass_refs` (the shared collapse-logic helper; `collapse_findings_pass` delegates to it for flat mode), governed by BC-2.11.031.")

### AC-025 — flat paths byte-identical: `{Flat, Collapsed}` and `{Flat, Expanded}` outputs unchanged
The `(Grouping::Flat, Collapse::Collapsed)` arm calls `render_findings_collapsed` and the `(Grouping::Flat, Collapse::Expanded)` arm calls the `render_finding_flat` loop — both byte-identical to v0.9.0 `FlatCollapsed` and `FlatExpanded` arms. All existing flat-mode tests (`BC-2.11.025`, `BC-2.11.026`, `BC-2.11.027`, `BC-2.11.028`, `BC-2.11.029` suites) continue to pass without modification.
(traces to BC-2.11.025 Invariant 5: "This BC and its invariants are scoped to `Grouping::Flat` (the `{Flat, Collapsed}` path).")

### AC-026 — comment sweep: `--no-collapse` doc-comment in `cli.rs` updated for dual-scope
The `///` doc-comment lines (approximately lines 154-159) on `no_collapse: bool` in `Commands::Analyze`
are updated to describe dual-scope behavior (flat and grouped/--mitre modes). The field uses bare
`#[arg(long)]` with NO `help = "..."` attribute — clap derives `--help` text from these `///` lines.
(traces to BC-2.11.028 Invariant 6: "Per LESSON-P1.04 (no unwired flags): the `no_collapse` field in `cli.rs` MUST be wired to `TerminalReporter.render` in `main.rs` via the two-field struct expression in Invariant 1 (STORY-119 F4 target)…")

### AC-027 — `cargo test --all-targets` passes green after all changes
After implementing all tasks, `cargo test --all-targets` passes with zero failures and zero compiler warnings. The new `mod story_119` test block contributes at minimum the tests in the VP table below. The migration does not regress any existing test in the `story_118`, `story_078`, `story_077`, or any other test module.
(traces to BC-2.11.030 Canonical Test Vector: "`TerminalReporter { ..., render: FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed } }` constructed directly in test — Dispatches to `render_findings_grouped_collapsed`; grouped-collapse behavior exercised.")

---

## Verification Properties

| VP-NNN | Property | Coverage in this Story | Proof Method |
|--------|----------|-----------------------|-------------|
| VP-012 | `escape_for_terminal` correctness (BC-2.11.010) | AC-023: all summary + evidence in the grouped-collapse path pass through `escape_for_terminal`; proptest harness unchanged; the grouped-collapse path inherits the same call sites as the existing grouped-expanded path — no new escape bypasses introduced | proptest (existing; unchanged) |
| VP-016 | Tactic headers in canonical order (BC-2.11.013/014/015) | AC-013, AC-014: `all_tactics_in_report_order()` unchanged; existing integration tests `mitre_grouping_emits_tactic_headers_in_canonical_order` and `mitre_grouping_buckets_none_and_unknown_under_uncategorized` continue to pass under `{Grouped, Expanded}` path (byte-identical to v0.9.0); new `test_BC_2_11_033_grouped_collapsed_preserves_bucket_order` covers the `{Grouped, Collapsed}` path. | integration (existing + new test) |

---

## Implementation Tasks

### Task 1 — Introduce `collapse_findings_pass_refs` shared helper and make `collapse_findings_pass` a thin adapter

**File:** `src/reporter/terminal.rs`
**Scope:** The existing `collapse_findings_pass` function at approx. `:340-360`.

Introduce new function `collapse_findings_pass_refs<'a>(&self, refs: &[&'a Finding]) -> Vec<(CollapseKey, Vec<&'a Finding>)>` that contains the shared collapse logic (the current body of `collapse_findings_pass`, adapted to work with `&[&Finding]` rather than `&[Finding]`). Then make `collapse_findings_pass` a thin adapter:

```rust
fn collapse_findings_pass<'a>(&self, findings: &'a [Finding]) -> Vec<(CollapseKey, Vec<&'a Finding>)> {
    let refs: Vec<&Finding> = findings.iter().collect();
    self.collapse_findings_pass_refs(&refs)
}
```

Note: use the `findings` PARAMETER (the `&'a [Finding]` argument), NOT any `self.findings` phantom field. `TerminalReporter` has no `findings` field — only `use_color`, `show_hosts_breakdown`, and `render`.

Reference: ADR-0003 "Collapse-API Shape" subsection; F2 design-note §5.2.1.

---

### Task 2 — Implement `render_findings_grouped_collapsed`

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
   - Call `collapse_findings_pass_refs(&bucket_refs)` once per bucket slice. Returns `Vec<(CollapseKey, Vec<&Finding>)>`.
   - `collapse_findings_pass` at `:340` is the thin adapter (Task 1); NOT called from the grouped path directly.
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

### Task 3 — Repoint `{Grouped, Collapsed}` dispatch arm from TEMPORARY to `render_findings_grouped_collapsed`

**File:** `src/reporter/terminal.rs`
**Scope:** The `(Grouping::Grouped, Collapse::Collapsed)` arm of the four-arm tuple dispatch established by STORY-122/A.

Replace the TEMPORARY `self.render_findings_grouped(&mut out, findings)` call with:
```rust
(Grouping::Grouped, Collapse::Collapsed) => {
    self.render_findings_grouped_collapsed(&mut out, findings);
}
```

Remove the `// TEMPORARY (STORY-122/A)` comment when replacing.

---

### Task 4 — Flip `--mitre` CLI default to `{Grouped, Collapsed}` — replace 3-arm if with orthogonal 2-if at construction site

**File:** `src/main.rs`
**Scope:** The `run_analyze` construction site — STORY-122/A left it as a 3-arm if-expression with 3 struct literals (byte-identical to v0.9.0). STORY-119/B CHANGES this to the orthogonal 2-if form so that `--mitre` alone produces `{Grouped, Collapsed}`.

Replace the STORY-122/A 3-arm if at the `run_analyze` construction site:
```rust
// STORY-122/A form (3-arm if — byte-identical to v0.9.0):
render: if show_mitre_grouping {
    FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }
} else if collapse_findings {
    FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }
} else {
    FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }
},
```
with the orthogonal 2-if struct wiring (BC-2.11.028 Invariant 1 target):
```rust
// STORY-119/B form (orthogonal 2-if — CLI flip: --mitre alone → {Grouped, Collapsed}):
render: FindingsRender {
    grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat },
    collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded },
},
```
Since `collapse_findings` defaults to `true` (from `collapse_findings_from_flag(no_collapse=false)`),
`--mitre` alone (`show_mitre_grouping=true, collapse_findings=true`) now produces
`{Grouped, Collapsed}`, which dispatches to `render_findings_grouped_collapsed` (Task 3).
`--mitre --no-collapse` (`show_mitre_grouping=true, collapse_findings=false`) produces
`{Grouped, Expanded}`, preserving the pre-STORY-119 `--mitre` behavior exactly.
This IS a code change to the construction site — the CLI flip is implemented here, not just in the
dispatch arm repoint.

**Note:** `show_mitre_grouping` (line 107) and `collapse_findings` (line 108) are the in-scope bool
params inside `run_analyze`. Do NOT reference `*mitre` or `no_collapse` — those live only in `main()` lines 79-80.
(traces to BC-2.11.030 Postcondition 2: "When `--mitre` is present and `--no-collapse` is absent: `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.")
(traces to BC-2.11.028 Invariant 1: "The two axes are fully orthogonal: no combination is illegal.")

---

### Task 5 — Update `--no-collapse` doc-comment in `src/cli.rs`

**File:** `src/cli.rs`
**Scope:** The `///` doc-comment lines (approximately lines 154-159) on `no_collapse: bool` in `Commands::Analyze`. Clap derives the `--help` text from these `///` doc-comment lines; there is NO `help = "..."` attribute on this field — the field uses bare `#[arg(long)]`.

Replace the current single-mode-scoped comment with dual-scope text:
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

### Task 6 — Add `mod story_119` test block to `tests/reporter_terminal_tests.rs`

**File:** `tests/reporter_terminal_tests.rs`
**Helper to add:** `grouped_collapse_reporter()` — returns `TerminalReporter { use_color: false, show_hosts_breakdown: false, render: FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed } }`. This helper uses only fields that exist on `TerminalReporter` (`use_color`, `show_hosts_breakdown`, `render`) — NOT any `self.findings` phantom field, NOT main()-scoped `*mitre`/`no_collapse`.

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

> **Disambiguation note (BC-2.11.025 Invariant 5 VP-table row 6):** The new test `test_BC_2_11_025_grouped_mode_bypasses_flat_collapse` in `mod story_119` is DISTINCT from two pre-existing same-stem siblings: (1) `test_BC_2_11_025_grouped_mode_bypasses_collapse` at `tests/reporter_terminal_tests.rs:2072` — verifies the `{Grouped, Expanded}` suffix-free path (no `(xN)` suffix produced by the `--mitre --no-collapse` combination via the OLD three-variant enum); (2) `test_BC_2_11_025_grouped_mode_bypasses_collapse_structurally` at `tests/reporter_terminal_tests.rs:4140` — verifies the OLD `FindingsRender::Grouped` enum variant's structural/suffix-free property via the `{Grouped, Expanded}` migration, surviving the reshape from three-variant enum to struct-of-enums. Both pre-existing siblings remain valid after the reshape and must not be removed. The new test in `mod story_119` verifies the `{Grouped, Collapsed}` per-bucket invariant (BC-2.11.025 Invariant 5 — grouped-collapsed mode uses per-bucket `collapse_findings_pass_refs`, never the global flat `collapse_findings_pass` adapter). All three are referenced in BC-2.11.025's VP-table (row 6 maps to Invariant 5). The name `test_BC_2_11_025_grouped_mode_bypasses_flat_collapse` resolves uniquely within `mod story_119`; the two pre-existing tests live in sibling `mod` blocks.

---

### Task 7 — Verify `cargo test --all-targets` green and compute input-hash

Run `cargo clippy --all-targets -- -D warnings` (must be clean). Run `cargo test --all-targets`
(must be green). Run `cargo fmt --check` (must pass). Then run:
```
bin/compute-input-hash --write .factory/stories/STORY-119.md
bin/compute-input-hash .factory/stories/STORY-119.md
```
Record the computed hash in the `input-hash` frontmatter field.

---

## Previous Story Intelligence

**Predecessor:** STORY-122/A (struct reshape + 84-site migration). Key applicable lessons:

1. **Variable scope in ACs:** `*mitre` and `no_collapse` are owned by `main()` (lines 79-80), NOT by `run_analyze`. All AC code blocks referencing the construction site use the in-scope params `show_mitre_grouping` (line 107) and `collapse_findings` (line 108) inside `run_analyze`.

2. **Verdict rank is 4-valued, ascending:** Likely=0, Possible=1, Inconclusive=2, Unlikely=3 (source-confirmed in `terminal.rs:447-454`). Within-bucket sort is ascending by rank value (lowest value = highest severity = sorted first). All BC citations on sort direction use this vocabulary.

3. **No `self.findings` field:** `TerminalReporter` has fields `use_color`, `show_hosts_breakdown`, and `render` only. The `collapse_findings_pass` thin adapter body is `let refs: Vec<&Finding> = findings.iter().collect(); self.collapse_findings_pass_refs(&refs)` where `findings` is the `&[Finding]` PARAMETER — not `self.findings`.

4. **Forbidden dependencies:** `render_findings_grouped_collapsed` MUST NOT call `render_findings_collapsed` or `render_findings_flat` — paths must remain structurally separate (BC-2.11.032 Architecture Anchor: "grouped-collapse replaces this with a bounded K-sampled loop per bucket group").

5. **Vec accumulator is canonical:** `collapse_findings_pass_refs` returns `Vec<(CollapseKey, Vec<&Finding>)>` — an insertion-ordered accumulator with linear-scan `PartialEq` matching. Not a `HashMap`. Not an `IndexMap`. This is the canonical v0.8.0 structure (BC-2.11.025 Invariant 7 and Invariant 9).

6. **Suffix before colorization:** The ` (xN)` suffix MUST be appended to the pre-color string before the color function is applied. Appending after the ANSI reset is NON-CONFORMANT (BC-2.11.031 PC-3 / BC-2.11.026 PC-6). STORY-120's code review found this pattern was applied correctly in `render_findings_collapsed`; replicate that exact pattern.

7. **Content-based citations in cross-indexes:** Use content-based citations (entry text) rather than line numbers when referencing the BC-INDEX, as changelog prepends cause line drift.

8. **STORY-122/A TEMPORARY note:** After STORY-122/A merges, the `{Grouped, Collapsed}` arm in `TerminalReporter::render()` has a `// TEMPORARY (STORY-122/A)` comment and routes to `render_findings_grouped`. STORY-119/B's Task 3 removes this comment and repoints the arm.

**From STORY-118** (flat-mode collapse predecessor, also applicable):
- `escape_for_terminal` is called at render time, not at key-construction time.
- The `COLLAPSE_EVIDENCE_SAMPLES = 3` constant at `terminal.rs:73` is shared; do not duplicate.
- Evidence lines: `"    > {}\n"` format with `escape_for_terminal` applied to each evidence string.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and ADR-0003:

1. **ADR-0003 Binding Rule 2:** `escape_for_terminal` must be called on every `summary` and `evidence` string before terminal output. No collapse path may bypass VP-012. Enforced at render time.

2. **ADR-0003 Binding Rule 4:** JSON and CSV reporters receive the complete, unmodified `&[Finding]` slice. No collapse pass upstream of multi-reporter dispatch. `render_findings_grouped_collapsed` is a private function of `TerminalReporter`; it is never called by `JsonReporter` or `CsvReporter`.

3. **ADR-0003 Binding Rule 5 (revised, STORY-122):** The `FindingsRender` type is now a struct-of-two-orthogonal-enums, not an enum. All four Cartesian combinations are valid. STORY-119/B builds on this type established by STORY-122/A.

4. **ADR-0003 "Collapse-API Shape":** `collapse_findings_pass_refs` is the shared collapse-logic function. `collapse_findings_pass` is the thin adapter for flat-mode callers. Grouped-mode callers invoke `collapse_findings_pass_refs` directly per bucket.

5. **Path separation invariant (BC-2.11.032):** `render_findings_grouped_collapsed` MUST NOT call `render_findings_collapsed` (flat-mode collapse path) or `render_finding_flat`. The grouped and flat paths must remain structurally separate.

6. **L4 Output layer constraint:** SS-11 (`reporter/terminal.rs`) must not import or call modules in L1 Ingest (SS-01/02) or L2 Stream (SS-04). `render_findings_grouped_collapsed` is purely a display-layer transform.

7. **No Default on FindingsRender/Grouping/Collapse:** Established by STORY-122/A. All construction sites in STORY-119/B continue to use explicit both-field struct expressions.

---

## Forbidden Dependencies

- **STORY-119/B MUST NOT be dispatched to F4 before STORY-122/A is merged and CI is green.** The struct-of-enums (`Grouping`, `Collapse`, `FindingsRender`) and four-arm dispatch do not exist until STORY-122/A ships.

- **`render_findings_grouped_collapsed` MUST NOT import or call `render_findings_collapsed` or `render_findings_flat`.** Cross-path calls violate the per-bucket isolation contract (BC-2.11.032) and the structural separation invariant.

- **`render_findings_grouped` (the existing suffix-free function) MUST NOT be modified.** Its body is called for the `{Grouped, Expanded}` dispatch arm and remains byte-identical to v0.9.0.

- **`render_finding_grouped` MUST NOT be called for N≥2 groups.** `render_finding_grouped` is a single-finding renderer: it is called ONLY for N=1 singleton groups within a bucket. For N≥2 groups, the header (with `(xN)` suffix), K=3-capped evidence lines, and the MITRE line (inline from `members[0]` with em-dash expansion) are ALL rendered inline in `render_findings_grouped_collapsed`. Calling `render_finding_grouped` for an N≥2 group would emit no `(xN)` suffix and uncapped evidence, violating AC-006, AC-011, and AC-021.

- **No new crate dependencies.** This story introduces no new entries in `Cargo.toml` beyond the existing dependencies.

- **`self.findings` phantom field MUST NOT appear anywhere.** `TerminalReporter` has no `findings` field. The thin adapter body must use the `findings` PARAMETER, not any `self.` field.

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
| `src/reporter/terminal.rs` | **Modify** | (1) Introduce `collapse_findings_pass_refs` shared helper + make `collapse_findings_pass` thin adapter (Task 1). (2) Implement `render_findings_grouped_collapsed` (Task 2). (3) Repoint `{Grouped, Collapsed}` arm from TEMPORARY to new function (Task 3). |
| `src/main.rs` | **Modify** | Replace the 3-arm if construction (STORY-122/A byte-identical form) with the orthogonal 2-if struct wiring at the `run_analyze` construction site (Task 4). This is the CLI flip: `--mitre` alone now produces `{Grouped, Collapsed}`. |
| `src/cli.rs` | **Modify** | Update `--no-collapse` help text for dual-scope (Task 5). |
| `tests/reporter_terminal_tests.rs` | **Modify** | Add `mod story_119` block with `grouped_collapse_reporter()` helper and all 19 tests (Task 6). |
| `Cargo.toml` | **Verify** | Confirm version is `0.9.0`; no change required (STORY-120 and STORY-122 already handled this). |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `collapse_findings_pass_refs` (new shared helper) | `src/reporter/terminal.rs` | Pure core |
| `collapse_findings_pass` (thin adapter) | `src/reporter/terminal.rs` | Pure core |
| `render_findings_grouped_collapsed` (new function) | `src/reporter/terminal.rs` | Pure core |
| `{Grouped, Collapsed}` dispatch arm (repointed) | `src/reporter/terminal.rs` | Pure core |
| `run_analyze` construction site (modified in Task 4: 3-arm if → orthogonal 2-if) | `src/main.rs` | Effectful (CLI entry point) |

**Architecture Anchors (post-STORY-122/A / pre-STORY-119/B state):**
- `src/reporter/terminal.rs` — `collapse_findings_pass` (locate by function signature `fn collapse_findings_pass`; becomes thin adapter in Task 1; line range shifts after STORY-122/A grows the dispatch block above it)
- `src/reporter/terminal.rs` — `collapse_findings_pass_refs` (F4-new; to be added adjacent to `collapse_findings_pass`)
- `src/reporter/terminal.rs` — `match (self.render.grouping, self.render.collapse)` four-arm dispatch in `TerminalReporter::render()` (locate by the match expression `match (self.render.grouping, self.render.collapse)`; Task 3 repoints the `{Grouped, Collapsed}` arm; line range shifts after STORY-122/A — do NOT hard-code `:202-224`, use content-based search per lesson #7)
- `src/reporter/terminal.rs` — `render_findings_grouped` (locate by function signature `fn render_findings_grouped`; Task 2 structural model; DO NOT MODIFY the function body; line range shifts after STORY-122/A)
- `src/reporter/terminal.rs` — `render_findings_collapsed` (locate by function signature `fn render_findings_collapsed`; flat-mode precedent for evidence loop and color ladder patterns; line range shifts after STORY-122/A)
- `src/reporter/terminal.rs:73` — `COLLAPSE_EVIDENCE_SAMPLES = 3` (shared constant; not duplicated; stable line, pre-dispatch block)
- `src/reporter/terminal.rs` — `render_finding_grouped` (locate by function signature `fn render_finding_grouped`; called for N=1 singletons; unchanged)
- `src/reporter/terminal.rs` — color ladder in `render_findings_collapsed` (locate by the `Likely` + `High` branch: `red().bold()` block; normative reference for suffix-in-pre-color-string pattern; line range shifts after STORY-122/A)
- `src/main.rs:107` — `show_mitre_grouping: bool` in-scope param in `run_analyze`
- `src/main.rs:108` — `collapse_findings: bool` in-scope param in `run_analyze`

**Subsystem anchor:** SS-11 owns this story's scope because grouped-mode collapse is a display-layer extension of reporter/terminal.rs per ARCH-INDEX Subsystem Registry.

**Dependency anchor:** STORY-119/B depends on STORY-122/A because the `Grouping`, `Collapse`, `FindingsRender` struct types and the four-arm dispatch with the TEMPORARY `{Grouped,Collapsed}` arm must exist before STORY-119/B can introduce `render_findings_grouped_collapsed` and repoint the arm.

---

## Token Budget Estimate

| Context item | Estimated tokens |
|-------------|-----------------|
| This story file (STORY-119.md, re-scoped) | ~5,500 |
| BC files (12 BCs × ~1,800 avg) | ~21,600 |
| `src/reporter/terminal.rs` (post-STORY-122/A, ~530 lines) | ~8,500 |
| `src/main.rs` (current, ~550 lines) | ~8,800 |
| All test files combined (~2500 lines + others) | ~20,000 |
| F2 design note (story-119-type-design.md) | ~3,500 |
| F1 delta-analysis | ~5,000 |
| Tool outputs (grep, cargo test) | ~3,000 |
| **Total estimated** | **~75,900** |

**Assessment:** ~75,900 tokens ≈ 30% of an agent context window (250k tokens). Borderline — load BC files on-demand (only the BC for the specific AC being implemented) rather than all 12 at once.

---

## Dependencies

- `depends_on: [STORY-122]` — STORY-122/A reshapes `FindingsRender` into the struct-of-orthogonal-enums and establishes the four-arm dispatch with a TEMPORARY `{Grouped, Collapsed}` arm. STORY-119/B repoints that arm to `render_findings_grouped_collapsed` and introduces the shared `collapse_findings_pass_refs` helper. The struct types and dispatch must exist (STORY-122/A) before STORY-119/B can build.
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

- **v1.0–v1.12:** See monolithic STORY-119 history (archived in version control). v1.12 was the last converged version before the D-120 split.
- **v2.0 (D-120 re-scope, 2026-06-18):** Re-scoped per D-120 human-confirmed split decision (2026-06-18). STORY-119/B now covers only the net-new behavioral delta: `render_findings_grouped_collapsed` implementation, `collapse_findings_pass_refs` shared helper, `collapse_findings_pass` thin adapter, dispatch arm repointing, CLI flip (`--mitre` default → `{Grouped, Collapsed}`), dual-scope `--no-collapse`. Removed from scope (moved to STORY-122/A): struct definition, enum→struct reshape, 84-site migration, four-arm dispatch establishment, comment sweep of three-variant/verdict-desc stale prose. ACs renumbered: old AC-001..004 (CLI mapping) → new AC-001..004 (identical text); old AC-005 (struct def) → moved to STORY-122/A; old AC-006 (dispatch existence) → moved to STORY-122/A; old AC-007 (84-site migration) → moved to STORY-122/A; old AC-008..029 → renumbered AC-005..026 (behavioral path ACs preserved); old AC-030 (comment sweep) → simplified AC-026 (dual-scope doc-comment only; three-variant sweep moved to STORY-122/A); old AC-031 (test green) → new AC-027. depends_on updated [STORY-120] → [STORY-122]. wave updated 49 → 50. points updated 8 → 5 (behavioral delta only; migration burden moved to STORY-122 3pts). Input BC list unchanged (same 12 BCs govern the complete feature). PO-final BC versions applied: 013 v1.15, 014 v2.1, 016 v1.10, 025 v1.14, 026 v1.14, 027 v1.8, 028 v1.10, 030 v1.5, 031 v1.4, 032 v1.5, 033 v1.4, 034 v1.4.
- **v2.1 (F3-resplit round-1 remediation, 2026-06-18):** Reconciled to Option X (human-approved split design). Previously Task 4 said "No code change needed to the construction site itself — the CLI flip happens by virtue of repointing the dispatch arm." That was Option Y (wrong). Fixed: Task 4 now prescribes REPLACING the STORY-122/A 3-arm if with the orthogonal 2-if form at `run_analyze` — this IS the construction flip that STORY-119/B owns under Option X. `src/main.rs` in File Structure Requirements changed from **Verify** to **Modify**. Architecture Mapping entry for `run_analyze` updated to reflect the Task 4 change. Scope section item 5 updated to explicitly state "This IS a code change to the construction site." AC-005 trace quote corrected from verbatim `"F4-new function"` to `"F4-pending new function:"` (verbatim from BC-2.11.031.md:165). The Option X coherence: A leaves `{Grouped, Collapsed}` unreachable-via-CLI (3-arm if never produces it); B makes `run_analyze` produce it via orthogonal 2-if and repoints the dispatch arm to `render_findings_grouped_collapsed`.
- **v2.2 (F3-resplit round-2 remediation, 2026-06-18):** Fix 3 (Pass A F-A-004): Architecture Anchors section updated to use content-based citations for all post-STORY-122 source functions whose line ranges will drift. The brittle `src/reporter/terminal.rs:202-224` line range (the four-arm dispatch block that STORY-122/A grows from 3→4 arms) replaced with a content-based citation: "locate by the match expression `match (self.render.grouping, self.render.collapse)`". Also converted `collapse_findings_pass` (:340), `render_findings_grouped` (:432-483), `render_findings_collapsed` (:376-423), `render_finding_grouped` (:311-327), and the color-ladder entry point (:391) to content-based anchors with locate-by instructions — all of these shift after STORY-122/A's dispatch block grows. Stable pre-dispatch reference `:73` (COLLAPSE_EVIDENCE_SAMPLES) retained as a line anchor. Per lesson #7 in Previous Story Intelligence: use content-based citations (entry text) rather than line numbers when referencing functions whose position shifts due to predecessor edits.
