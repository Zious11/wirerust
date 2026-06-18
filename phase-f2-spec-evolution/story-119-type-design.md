# STORY-119 Type Design: Struct-of-Orthogonal-Enums for FindingsRender

**Phase:** F2 (Spec Evolution — Design Half)
**Date:** 2026-06-18
**Story:** STORY-119 — Grouped-mode finding collapse (issue #259 tail)
**F1 gate locked:** 2026-06-17 (D-110 decisions)
**Status:** Design approved; F4 (implementation) pending

---

## 1. Context

`FindingsRender` (shipped as a 3-variant enum in STORY-120 / v0.9.0 unreleased develop line,
`src/reporter/terminal.rs:101`) represents the rendering mode for the FINDINGS section of
`TerminalReporter`. The v0.9.0 enum was the correct representation *at the time*: grouped mode
did not support collapse, so the combination `Grouped + Collapsed` was a meaningless illegal
state, and a 3-variant sum type encoded exactly the three valid modes.

STORY-119 **adds** grouped-mode collapse: collapsed output within each MITRE tactic bucket
(`(xN)` suffix, K=3 evidence sampling). This makes the two rendering axes **fully orthogonal**:

- **Grouping axis:** group by MITRE tactic vs render flat
- **Collapse axis:** collapse repeated findings into counted groups vs one line per finding

All four combinations are now valid domain states. The 3-variant enum's illegal-state
justification no longer applies. The approved F1 decision (D-110) is to replace the enum with
a struct-of-two-orthogonal-enums — the product type that faithfully encodes the orthogonal
domain.

Research basis: `.factory/research/story-119-render-mode-typedesign.md` (Perplexity
sonar-deep-research + perplexity_reason, 2026-06-18). The research concludes that when two
axes form a full Cartesian product with no illegal combinations, a struct of named enums is the
idiomatic Rust representation (Alexis King "Parse, Don't Validate"; Rust community consensus;
`tracing-subscriber`/`clap`/`miette`/`ratatui` precedent pattern).

---

## 2. Approved Type Definitions

### 2.1 New public types (replaces `FindingsRender` 3-variant enum)

```rust
/// Grouping axis for the FINDINGS section.
///
/// `Grouped` organizes findings by MITRE ATT&CK tactic (`--mitre`).
/// `Flat` renders findings in emission order with no tactic headers.
///
/// Combined with [`Collapse`] in [`FindingsRender`] to form the complete
/// rendering-mode product type. See ADR-0003 Binding Rule 5 (revised).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    /// Group findings by MITRE tactic — renders tactic-bucket headers
    /// (`## Tactic Name`) and sorts within each bucket by verdict-desc,
    /// confidence-desc, then emission order (stable).
    /// Corresponds to `--mitre` flag / `show_mitre_grouping = true`.
    Grouped,
    /// Render findings in original emission order with no tactic headers.
    /// Corresponds to the absence of `--mitre`.
    Flat,
}

/// Collapse axis for the FINDINGS section.
///
/// `Collapsed` groups repeated findings into counted summaries (`(xN)` suffix,
/// K=3 evidence sampling). `Expanded` renders one display line per raw finding.
///
/// Combined with [`Grouping`] in [`FindingsRender`] to form the complete
/// rendering-mode product type. See ADR-0003 Binding Rule 5 (revised).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collapse {
    /// Collapse repeated findings: groups sharing `(category, verdict,
    /// confidence, summary)` render as one header with ` (xN)` suffix
    /// when N ≥ 2, plus up to K=3 sampled evidence lines.
    /// Default for terminal output (`--no-collapse` not passed).
    Collapsed,
    /// One display line per raw finding. Pre-v0.8.0 behavior; restored
    /// by `--no-collapse`.
    Expanded,
}

/// Rendering mode for the FINDINGS section of [`TerminalReporter`].
///
/// Replaces the v0.9.0 three-variant [`FindingsRender`] enum with a product
/// type that encodes the two now-orthogonal rendering axes:
///
/// - [`Grouping`]: whether to group by MITRE tactic or render flat.
/// - [`Collapse`]: whether to collapse repeated findings or expand each one.
///
/// All four combinations are valid since STORY-119 adds grouped-mode collapse.
/// The v0.9.0 sum type was correct when `{Grouped, Collapsed}` was an illegal
/// state; the orthogonality realization (STORY-119) requires the product type.
///
/// No [`Default`] is derived — consistent with STORY-120's deliberate omission.
/// All construction sites carry sufficient context to select both axes
/// explicitly; `Default::default()` would obscure which mode is being selected.
///
/// ADR-0003 Binding Rule 5 (revised, STORY-119).
/// BC-2.11.013 / BC-2.11.025 / BC-2.11.026 / BC-2.11.027 / BC-2.11.028.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingsRender {
    pub grouping: Grouping,
    pub collapse: Collapse,
}
```

### 2.2 Derives rationale

Each enum: `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` — consistent with the v0.9.0
`FindingsRender` enum derives; no `Hash` (parallel to `CollapseKey` policy in v0.8.0; the
linear-scan Vec accumulator pattern for `ThreatCategory`/`Verdict`/`Confidence` not adding
`Hash` is already established). `Default` omitted on all three types — deliberate, matches
STORY-120's rationale (see ADR-0003 "Default Derive: Deliberate Omission").

---

## 3. Migration Map from v0.9.0 Three-Variant Enum

Every `FindingsRender::Variant` in the codebase translates to a two-field struct literal:

| Old `FindingsRender` variant | New `FindingsRender` struct | Notes |
|------------------------------|-----------------------------|-------|
| `FindingsRender::Grouped` | `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` | Preserves suffix-free grouped behavior (BC-2.11.025 Inv5 OLD: MITRE grouped path never had collapse) |
| `FindingsRender::FlatCollapsed` | `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` | Unchanged semantics |
| `FindingsRender::FlatExpanded` | `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }` | Unchanged semantics |
| *(new)* | `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` | STORY-119 grouped-collapse mode; was the old illegal state |

---

## 4. Dispatch Redesign in `TerminalReporter::render()`

### 4.1 Current dispatch (v0.9.0, `src/reporter/terminal.rs:202-224`)

```rust
match self.render {
    FindingsRender::Grouped => {
        self.render_findings_grouped(&mut out, findings);
    }
    FindingsRender::FlatCollapsed => {
        self.render_findings_collapsed(&mut out, findings);
    }
    FindingsRender::FlatExpanded => {
        for f in findings {
            self.render_finding_flat(&mut out, f);
        }
    }
}
```

### 4.2 New dispatch (STORY-119 / F4 target)

```rust
match (self.render.grouping, self.render.collapse) {
    (Grouping::Grouped, Collapse::Expanded) => {
        // Current grouped path — suffix-free, UNCHANGED behavior.
        // BC-2.11.013: tactic bucketing + sort. render_finding_grouped()
        // for each finding (MITRE name expansion, no (xN) suffix).
        self.render_findings_grouped(&mut out, findings);
    }
    (Grouping::Grouped, Collapse::Collapsed) => {
        // NEW (STORY-119): grouped-mode collapse.
        // Collapses within each MITRE tactic bucket independently.
        // Each bucket applies the collapse pass (same CollapseKey as flat mode),
        // then renders with (xN) suffix for N≥2 and K=3 evidence sampling.
        // Tactic ordering and bucket headers from render_findings_grouped unchanged.
        self.render_findings_grouped_collapsed(&mut out, findings);
    }
    (Grouping::Flat, Collapse::Collapsed) => {
        // Current flat-collapsed path — UNCHANGED behavior.
        // BC-2.11.025 / BC-2.11.026 / BC-2.11.027.
        self.render_findings_collapsed(&mut out, findings);
    }
    (Grouping::Flat, Collapse::Expanded) => {
        // Current flat-expanded path — UNCHANGED behavior.
        // One render_finding_flat() call per finding.
        for f in findings {
            self.render_finding_flat(&mut out, f);
        }
    }
}
```

### 4.3 2×2 Dispatch Matrix

| `Grouping` \ `Collapse` | `Collapsed` | `Expanded` |
|-------------------------|-------------|------------|
| **Grouped** | NEW: `render_findings_grouped_collapsed` — tactic-bucket collapse, `(xN)` suffix per bucket, K=3 evidence sampling, name-expanded MITRE line from `group[0]` (STORY-119) | EXISTING: `render_findings_grouped` — suffix-free grouped, MITRE name expansion (BC-2.11.013) |
| **Flat** | EXISTING: `render_findings_collapsed` — flat collapse, `(xN)` suffix, K=3 evidence sampling (BC-2.11.025/026/027) | EXISTING: `render_finding_flat` loop — one line per finding (pre-v0.8.0) |

---

## 5. `render_findings_grouped_collapsed` — New Function Spec (F4 Target)

This is the only **new** render function introduced by STORY-119. All other functions are
structurally unchanged.

### 5.1 Behavior Specification

The function performs tactic bucketing and sorting identically to the existing
`render_findings_grouped` (BC-2.11.013: `mitre_techniques[0]` determines bucket; verdict-desc
then confidence-desc then emission-index sort within each bucket). Then, **within each tactic
bucket**, it applies the existing `collapse_findings_pass` (same `CollapseKey` semantics as flat
mode: `(category, verdict, confidence, summary)`) and renders each resulting group using the
collapse rendering rules:

- **N = 1 (singleton within bucket):** delegates to `render_finding_grouped` — byte-identical
  to the existing grouped-expanded path; suffix-free; MITRE name expansion.
- **N ≥ 2 (repeated within bucket):** renders header with ` (xN)` suffix appended BEFORE
  colorization (suffix inside ANSI color span, same convention as flat collapse per
  BC-2.11.026 PC-6). Evidence sampling: first `min(N, COLLAPSE_EVIDENCE_SAMPLES)` members
  positionally; `evidence[0]` from each inspected member if non-empty; window does NOT slide
  past empty-evidence members (BC-2.11.027 invariant 2). MITRE line: sourced from
  `group_members[0]` if non-empty; uses `render_finding_grouped`-style name expansion
  (technique name via `technique_name()`, `— Name` suffix or `(unknown)`).

The `escape_for_terminal` invariant (VP-012) is preserved: all `summary` and `evidence` strings
pass through `escape_for_terminal` before being written to the output buffer. The collapse pass
itself operates on unescaped raw `Finding` field values (escape is render-time, not key-time).

### 5.2 Key Implementation Notes (for F4)

- Reuse `collapse_findings_pass` without modification — it accepts `&[Finding]` and returns
  `Vec<(CollapseKey, Vec<&Finding>)>` with first-occurrence ordering. Call it once per
  bucket's finding slice, not across all findings.
- `COLLAPSE_EVIDENCE_SAMPLES = 3` constant is shared with flat-collapse; no duplication needed.
- The tactic-bucket data structure (HashMap keyed on `Option<MitreTactic>`) and iteration
  over `all_tactics_in_report_order()` carry over without change.
- Bucket headers (`## Tactic Name`, `## Uncategorized`) carry over without change.
- `render_finding_grouped` is called for singletons (preserves the name-expansion MITRE line
  format). For N≥2 groups, the MITRE line is rendered inline (not via `render_finding_grouped`)
  to incorporate the name-expansion format on the group representative (`members[0]`).

---

## 6. CLI/UX Wiring in `src/main.rs` `run_analyze`

### 6.1 Current wiring (v0.9.0)

```rust
// src/main.rs ~line 381 (TerminalReporter construction site)
render: if show_mitre_grouping {
    FindingsRender::Grouped
} else if collapse_findings {
    FindingsRender::FlatCollapsed
} else {
    FindingsRender::FlatExpanded
},
```

### 6.2 New wiring (STORY-119 / F4 target)

```rust
// src/main.rs — TerminalReporter construction site inside run_analyze
// show_mitre_grouping ← *mitre (CLI flag)
// collapse_findings ← collapse_findings_from_flag(*no_collapse) (unchanged)
render: FindingsRender {
    grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat },
    collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded },
},
```

`collapse_findings_from_flag` is **unchanged**: `fn collapse_findings_from_flag(no_collapse: bool) -> bool { !no_collapse }`. The `run_analyze` function signature is **unchanged**
(`show_mitre_grouping: bool, collapse_findings: bool`).

### 6.3 CLI → Render Mode Table (APPROVED behavior, post-STORY-119)

| CLI flags | `show_mitre_grouping` | `collapse_findings` | Resulting `FindingsRender` | Behavior |
|-----------|----------------------|---------------------|----------------------------|----------|
| *(default)* | false | true | `{Flat, Collapsed}` | Flat collapse — default terminal output. UNCHANGED. |
| `--no-collapse` | false | false | `{Flat, Expanded}` | Flat expanded — one line per finding. UNCHANGED. |
| `--mitre` | true | true | `{Grouped, Collapsed}` | **NEW default for grouped mode** — MITRE tactic buckets with per-bucket collapse. BEHAVIOR CHANGE from v0.9.0 `Grouped` (was expanded). |
| `--mitre --no-collapse` | true | false | `{Grouped, Expanded}` | Grouped expanded — old `--mitre` behavior; suffix-free, name-expanded MITRE lines. `--no-collapse` now suppresses collapse in BOTH modes. |

### 6.4 Approved Behavior Change

`--mitre` alone now produces `{Grouped, Collapsed}` — grouped output with per-bucket collapse.
In v0.9.0, `--mitre` produced `{Grouped, Expanded}` (the old `FindingsRender::Grouped`).

**Rationale (F1 gate, D-110):** Collapse is the default for terminal output (default-on since
v0.8.0). It is inconsistent for `--mitre` to opt out of collapse by default. The new behavior
makes collapse default-on symmetrically across both modes; `--no-collapse` now acts as a
dual-scope flag that suppresses collapse in both flat and grouped modes. This is a deliberate
UX improvement, not a regression.

### 6.5 `run_summary` Site

The `run_summary` construction site remains:

```rust
render: FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed },
```

This replaces the v0.9.0 `render: FindingsRender::FlatCollapsed`. The inert-value semantics
are unchanged — `run_summary` never renders a FINDINGS section; the field is structurally
present but irrelevant.

---

## 7. Semver Note

This is a further breaking change to the (unreleased) v0.9.0 `FindingsRender` public type,
which was itself a breaking change from v0.8.0. Per D-110: **no separate version bump**.
This change bundles into the unreleased 0.9.0 develop line. `FindingsRender` has not shipped
in any released crate version (v0.8.0 shipped the bool fields, not the enum). The release of
v0.9.0 is **held** pending STORY-119 completion; v0.9.0 will include both the STORY-120 enum
introduction and the STORY-119 struct-of-enums evolution as a unified breaking change from v0.8.0.

`cargo-semver-checks` will fire `struct_field_missing` on `FindingsRender` (the enum fields
`Grouped`/`FlatCollapsed`/`FlatExpanded` are replaced by struct fields `grouping`/`collapse`).
This is expected and correct. The relevant baseline for the semver check is v0.8.x.

---

## 8. BC Revisions Required (Product Owner — F2 BC Half)

The product owner must revise or author the following behavioral contracts before F3 story
decomposition. This list is authoritative for the F2 BC half.

### 8.1 Existing BCs to Revise

**BC-2.11.013** (MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last)
- **Current:** Preconditions reference `FindingsRender::Grouped` (the 3-variant enum variant).
  Invariants and postconditions describe grouped rendering without any collapse consideration.
- **Required change:** Revise preconditions to reference the new struct:
  `render.grouping == Grouping::Grouped`. Add invariants covering the `{Grouped, Collapsed}`
  case: within-bucket collapse applies the same `CollapseKey` as flat mode; tactic bucket
  ordering and header format are unchanged; singleton handling within a bucket is the same as
  `render_finding_grouped`. Revise any Inv referencing "Grouped implies no collapse" —
  **this invariant is now false** and must be removed or replaced.

**BC-2.11.025** (Flat-Mode Collapse Groups Findings by Key; First-Occurrence Order; Deterministic)
- **Current:** Invariant 4 or similar may state that collapse only applies when
  `show_mitre_grouping = false` / `FindingsRender::FlatCollapsed`. The "Flat-Mode" title
  scopes this to flat mode only.
- **Required change:** The invariant that collapse is flat-only (from the STORY-118
  scope-control decision, ADR-0003 §"Flat Mode Only for v0.8.0") must be **explicitly
  retired/narrowed**. The BC should now scope its invariants to `Grouping::Flat` only (the
  `collapse_findings_pass` function is shared, but the flat-mode invocation contract is
  separate from the grouped-mode invocation). Consider a new title: "Flat-Mode Collapse
  Groups Findings by CollapseKey" (existing semantics unchanged for flat mode).

**BC-2.11.026** (Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton Renders Without)
- **Current:** Postcondition 4 (PC-4) constrains the observable line-order of the collapse
  output. May implicitly scope to flat mode only or reference `FindingsRender::FlatCollapsed`.
- **Required change:** Broaden to cover both flat-mode and grouped-mode collapse: the `(xN)`
  suffix rule (N≥2 gets suffix, N=1 no suffix) applies in BOTH modes. Ensure PC-4 and any
  observable-line-order postconditions are valid for the grouped-mode path too. Clarify that
  in grouped mode the singleton/N≥2 rule applies per-bucket.

**BC-2.11.028** (--no-collapse Opt-Out Flag Disables Collapse; JSON/CSV Unaffected)
- **Current:** BC scopes `--no-collapse` to flat-mode only (because v0.8.0 grouped mode never
  had collapse). Invariant 1 or title may state the flag "disables terminal collapse" but
  the architecture anchor references the pre-STORY-119 if-chain dispatch that routes `--mitre`
  to `FindingsRender::Grouped` regardless of `no_collapse`.
- **Required change:** Broaden to dual-scope: `--no-collapse` suppresses collapse in BOTH
  flat mode AND grouped mode. Update precondition / architecture anchor to reflect the new
  struct-construction wiring (Section 6.2 above). New invariant: "when `--mitre` and
  `--no-collapse` are both passed, the output is grouped-expanded (`{Grouped, Expanded}`) —
  suffix-free, MITRE name-expanded, no collapse." Update the CLI wiring anchor from the old
  if-chain to the new two-field struct construction.

### 8.2 New BCs to Author

The following BCs do not yet exist and must be authored for STORY-119. Suggested numbers
are BC-2.11.030 onward (next available after BC-2.11.029).

**New BC: Grouped-Collapse Default Behavior (`--mitre` alone)**
- Specifies that `--mitre` without `--no-collapse` produces `{Grouped, Collapsed}` (the new
  default). `--mitre --no-collapse` produces `{Grouped, Expanded}`.
- Postconditions: the output is organized into tactic-bucket headers; within each bucket,
  findings are collapsed by `CollapseKey`; `(xN)` suffix rule applies per-bucket.

**New BC: Per-Bucket Collapse — Count Suffix and Singleton Handling**
- Within a MITRE tactic bucket under `{Grouped, Collapsed}`: a group of N ≥ 2 identical
  findings (same `CollapseKey`) renders as one header line with ` (xN)` suffix appended
  before colorization. A singleton renders via `render_finding_grouped` (no suffix, MITRE
  name expansion). The `(xN)` suffix format matches flat-mode (same constant, same
  colorization convention per BC-2.11.026).

**New BC: Per-Bucket Evidence Sampling in Grouped-Collapse Mode**
- Within a tactic bucket, collapsed group of N ≥ 2: at most K = COLLAPSE_EVIDENCE_SAMPLES
  (= 3) evidence lines are rendered, sourced from the first `min(N, K)` group members
  positionally. The window does NOT slide past empty-evidence members (inherits BC-2.11.027
  invariant 2 semantics). The remainder are elided from terminal display only.

**New BC: Tactic-Bucket Ordering Invariant Under Grouped-Collapse**
- Tactic-bucket ordering under `{Grouped, Collapsed}` is identical to `{Grouped, Expanded}`:
  buckets appear in `all_tactics_in_report_order()` order; `Uncategorized` bucket is last.
  Collapse does not alter tactic ordering or bucket membership.

**New BC: MITRE Line Format in Grouped-Collapse**
- For N ≥ 2 groups under `{Grouped, Collapsed}`: the MITRE line is sourced from
  `group_members[0]` and uses the grouped name-expansion format (`MITRE: T1234 — Name` or
  `MITRE: T1234 (unknown)`), consistent with `render_finding_grouped`. The name-expanded
  MITRE line does not include the `(xN)` count.

---

## 9. Files Affected (F4 Implementation Scope)

| File | Change |
|------|--------|
| `src/reporter/terminal.rs` | (1) Replace 3-variant `FindingsRender` enum with `Grouping` enum + `Collapse` enum + `FindingsRender` struct. (2) Add `render_findings_grouped_collapsed` function. (3) Rewrite 3-arm `match self.render` to 4-arm `match (self.render.grouping, self.render.collapse)`. All existing function names (`render_findings_grouped`, `render_findings_collapsed`, `render_finding_flat`, `render_finding_grouped`) are UNCHANGED. (4) Update module-level doc comment. |
| `src/main.rs` | (1) Update `use` import: `FindingsRender` → `FindingsRender, Grouping, Collapse`. (2) Rewrite `TerminalReporter` construction site in `run_analyze` to struct literal (Section 6.2). (3) Rewrite `run_summary` site to struct literal (Section 6.5). |
| `tests/reporter_tests.rs` | Mechanical: update all `render: FindingsRender::Grouped/FlatCollapsed/FlatExpanded` to struct literals per migration map (Section 3). Add test fixtures for `{Grouped, Collapsed}`. |
| `tests/reporter_terminal_tests.rs` | Same mechanical update as above. |
| Any other test files | Any remaining `FindingsRender::` variant references — mechanical update. |

---

## 10. Invariants Carried Forward

The following invariants from existing BCs are **unchanged** by this design:

- `escape_for_terminal` is called on every `summary` and `evidence` string before terminal
  output — no collapse path may bypass VP-012 (ADR-0003 Rule 2).
- `collapse_findings_pass` operates on raw (unescaped) `Finding` field values; escape is
  render-time, not key-time.
- JSON and CSV reporters receive the complete, unmodified `&[Finding]` slice — no collapse
  pass upstream of multi-reporter dispatch (ADR-0003 Rule 4).
- `COLLAPSE_EVIDENCE_SAMPLES = 3` is a shared named constant; no duplication.
- `No Default` on `FindingsRender` — deliberate omission, consistent with STORY-120.
