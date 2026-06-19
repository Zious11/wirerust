---
document_type: story
story_id: STORY-122
epic_id: E-18
version: "1.4"
status: pending
producer: story-writer
timestamp: 2026-06-18T00:00:00Z
phase: f3
points: 3
priority: P2
depends_on: [STORY-120]
blocks: [STORY-119]
behavioral_contracts:
  - BC-2.11.013
  - BC-2.11.014
  - BC-2.11.016
  - BC-2.11.026
  - BC-2.11.027
  - BC-2.11.028
verification_properties: [VP-016]
tdd_mode: strict
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 1
feature_id: e18-finding-collapse
github_issue: 259
wave: 49
assumption_validations: []
risk_mitigations: []
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.014.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.016.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.028.md
  - .factory/phase-f1-delta-analysis/story-119-grouped-mode-collapse-delta-analysis.md
  - .factory/phase-f2-spec-evolution/story-119-type-design.md
  - .factory/phase-f2-spec-evolution/story-119-prd-delta.md
# BC status: 6 BCs authored/converged at F2 (2026-06-18). PO-final versions:
#   013 v1.15, 014 v2.1, 016 v1.10, 026 v1.14, 027 v1.8, 028 v1.10.
#   All BCs are F2-frozen; normative bodies must not be edited as part of STORY-122 F4 implementation.
# Subsystem anchor: SS-11 owns this story's scope because FindingsRender struct reshape is
#   a display-layer type definition living in reporter/terminal.rs — the core SS-11 module.
# Dependency anchor:
#   depends_on: [STORY-120] — STORY-120 introduces the three-variant FindingsRender enum
#   (FindingsRender::Grouped, ::FlatCollapsed, ::FlatExpanded); STORY-122 reshapes that
#   enum into the struct-of-orthogonal-enums (FindingsRender { grouping: Grouping,
#   collapse: Collapse }). The type must exist (STORY-120) before STORY-122 can reshape it.
#   blocks: [STORY-119] — STORY-119/B depends on the struct-of-enums introduced here.
#   STORY-122 cannot be built before STORY-120 ships.
# Wave anchor: wave 49 = max(wave(STORY-120)=48) + 1. STORY-120 is the unique predecessor.
#   STORY-122 and STORY-119 are sequenced 122→119 (wave 49 and 50 respectively), not parallel,
#   because STORY-119/B depends on the {Grouped, Collapsed} arm wiring that STORY-122 establishes.
# Split rationale: D-120 (human-confirmed 2026-06-18) splits the monolithic STORY-119 v1.12 into:
#   A = STORY-122 (this story): enum→struct reshape + 84-site migration (byte-identical output).
#   B = STORY-119 (re-scoped): grouped-collapse render path + CLI flip (net-new behavior).
input-hash: "309f190"
---

# STORY-122: FindingsRender enum→struct reshape + construction-site migration (byte-identical)

## Narrative

- **As a** wirerust maintainer implementing the STORY-119 behavioral split (D-120)
- **I want** the `FindingsRender` type reshaped from a three-variant enum to a
  struct-of-two-orthogonal-enums (`FindingsRender { grouping: Grouping, collapse: Collapse }`)
  and all 84 construction sites across source and test files migrated to the new struct literal
  form, with a four-arm exhaustive tuple dispatch, and the `{Grouped, Collapsed}` arm
  TEMPORARILY routing to `render_findings_grouped` (the existing byte-identical grouped-expanded
  path), while CLI mapping and all observable outputs remain byte-identical to v0.9.0
- **So that** STORY-119/B can be dispatched against a clean struct-based codebase without
  also carrying the mechanical 84-site migration burden, and the compiler enforces exhaustiveness
  over the full 2x2 type space from the moment this story merges

**Scope — what this story does:**
- Defines `pub enum Grouping { Grouped, Flat }`, `pub enum Collapse { Collapsed, Expanded }`,
  and `pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }` (all
  derive `Debug, Clone, Copy, PartialEq, Eq`; no `Default` on any type).
- Migrates all 84 `FindingsRender::Variant` construction sites to struct literal form (see
  per-file census below).
- Replaces the three-arm `match self.render` dispatch with a four-arm
  `match (self.render.grouping, self.render.collapse)` tuple dispatch.
- CRITICAL: The `{Grouped, Collapsed}` arm TEMPORARILY routes to
  `render_findings_grouped(&mut out, findings)` (the existing byte-identical grouped-expanded
  function). This is intentional — the `{Grouped, Collapsed}` combination is UNREACHABLE via
  CLI in this story (STORY-122/A leaves the CLI mapping unchanged; `--mitre` alone still maps
  to `{Grouped, Expanded}` per v0.9.0 behavior). STORY-119/B repoints this arm to
  `render_findings_grouped_collapsed`.

**Scope — what this story does NOT do:**
- Does NOT introduce `render_findings_grouped_collapsed` (that is STORY-119/B, Task 4).
- Does NOT introduce `collapse_findings_pass_refs` (that is STORY-119/B, Task 4).
- Does NOT add per-bucket collapse to any output path.
- Does NOT change the CLI default for `--mitre` (stays at `{Grouped, Expanded}` throughout
  this story — the CLI flip is STORY-119/B).
- Does NOT modify `render_findings_grouped` body.

**Behavior change:** zero — output is byte-identical to v0.9.0 for all inputs across all modes
because all dispatch arms call the same functions as before.

---

## Behavioral Contracts

All 6 BCs governing this story are authored and F2-frozen.

| BC | Version | Role for STORY-122 |
|----|---------|-------------------|
| BC-2.11.028 | v1.10 | PRIMARY: struct construction form `FindingsRender { grouping: ..., collapse: ... }` at the `run_analyze` and `run_summary` construction sites. Invariant 1 defines the two-field struct expression; Postcondition 4 establishes orthogonality. STORY-122/A: preserves byte-identical wiring (--no-collapse dual-scope struct construction preserved; no CLI change). |
| BC-2.11.013 | v1.15 | Tactic-header structure and collapse axis: when `render.grouping == Grouping::Grouped` with `Collapse::Collapsed`, per-bucket collapse applies. STORY-122/A: the four-arm dispatch is established here but `{Grouped, Collapsed}` TEMPORARILY routes to `render_findings_grouped` (byte-identical). |
| BC-2.11.014 | v2.1 | Within-bucket sort — ascending by verdict-rank (Likely=0, Possible=1, Inconclusive=2, Unlikely=3), confidence-rank (High=0, Medium=1, Low=2), then emission-index. Not directly implemented here; sort logic lives in `render_findings_grouped` (unchanged). Governs correctness of the `{Grouped, Expanded}` arm. |
| BC-2.11.016 | v1.10 | Grouped MITRE em-dash+name line format — em-dash + technique name for known IDs, `(unknown)` for unrecognized. Applies to both `{Grouped, Collapsed}` and `{Grouped, Expanded}` paths; `{Grouped, Collapsed}` arm TEMPORARY in STORY-122/A. |
| BC-2.11.026 | v1.14 | Flat-mode `(xN)` suffix rule. PC-4 suffix-free guarantee now scoped to `{Grouped, Expanded}` only. Governs correctness of `{Flat, Collapsed}` arm (unchanged by this story). |
| BC-2.11.027 | v1.8 | Flat-mode K=3 evidence sampling. Governs `{Flat, Collapsed}` arm (unchanged by this story). |

---

## Acceptance Criteria

### AC-001 — `Grouping`, `Collapse`, and `FindingsRender` types defined correctly in `terminal.rs`
The `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` definition in
`src/reporter/terminal.rs` is replaced by:
- `pub enum Grouping { Grouped, Flat }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`
- `pub enum Collapse { Collapsed, Expanded }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`
- `pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }` with
  `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`
No `Default` is derived on any of the three types (deliberate omission, consistent with STORY-120).
(traces to BC-2.11.028 Invariant 1: "…`render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`… The two axes are fully orthogonal: no combination is illegal.")

### AC-002 — Four-arm exhaustive tuple dispatch established in `TerminalReporter::render()`
The `match self.render` three-arm dispatch in `TerminalReporter::render()` is replaced by a
`match (self.render.grouping, self.render.collapse)` four-arm tuple dispatch:
- `(Grouping::Grouped, Collapse::Expanded)` → `self.render_findings_grouped(&mut out, findings)` (UNCHANGED behavior)
- `(Grouping::Grouped, Collapse::Collapsed)` → `self.render_findings_grouped(&mut out, findings)` (TEMPORARY: routes to grouped-expanded path; byte-identical since {Grouped,Collapsed} is unreachable via CLI in STORY-122/A; STORY-119/B repoints this arm to `render_findings_grouped_collapsed`)
- `(Grouping::Flat, Collapse::Collapsed)` → `self.render_findings_collapsed(&mut out, findings)` (UNCHANGED)
- `(Grouping::Flat, Collapse::Expanded)` → `for f in findings { self.render_finding_flat(&mut out, f); }` (UNCHANGED)
(traces to BC-2.11.013 Invariant 4: "When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (`--mitre --no-collapse`): the collapse pass is NOT applied… When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` (`--mitre` alone, the new default since STORY-119): a per-bucket collapse pass IS applied within each tactic bucket." — STORY-122/A establishes the dispatch arms; the {Grouped,Collapsed} behavioral delta activates in STORY-119/B.)

### AC-003 — All 84 `FindingsRender::Variant` construction sites migrated to struct literal form
Every occurrence of `FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, and
`FindingsRender::FlatExpanded` across all source and test files is updated to the corresponding
struct literal per the D-110 migration map:
- `FindingsRender::Grouped` → `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (preserves suffix-free semantics)
- `FindingsRender::FlatCollapsed` → `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`
- `FindingsRender::FlatExpanded` → `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }`
Per-file census (grepped ground-truth from F1 §8 + F2 §9): `src/main.rs` 4 enum-variant
occurrences across 2 logical construction sites; `src/reporter/terminal.rs` 3; `tests/reporter_terminal_tests.rs` 55;
`tests/reporter_tests.rs` 17; `tests/dnp3_f5_remediation_tests.rs` 2; `tests/bc_2_09_100_multitag_tests.rs` 3 — total 84.
After migration: `grep -rn "FindingsRender::Grouped\|FindingsRender::FlatCollapsed\|FindingsRender::FlatExpanded" src/ tests/`
returns zero lines (zero-grep gate). Note: the 3 terminal.rs occurrences (at :203/:209/:216) are match-arm PATTERNS removed by Task 3's 4-arm tuple rewrite, not struct-literal construction sites — they count toward the 84 total and are satisfied by Task 3 (not Task 2).
(traces to BC-2.11.028 Postcondition 4: "The `collapse` axis of the `FindingsRender` struct is determined exclusively by `collapse_findings`; the `grouping` axis by `show_mitre_grouping`. They are fully orthogonal.")

### AC-004 — `run_analyze` construction site migrates 3-arm if-expression to 3 struct literals (byte-identical)
The existing 3-arm `if`-expression at `src/main.rs:381-387` (current v0.9.0 code:
`if show_mitre_grouping { FindingsRender::Grouped } else if collapse_findings { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }`)
is migrated to three parallel struct literals that preserve the same branching logic:
```rust
render: if show_mitre_grouping {
    FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }
} else if collapse_findings {
    FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }
} else {
    FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }
},
```
The in-scope bool params are `show_mitre_grouping` (line 107) and `collapse_findings` (line 108)
inside `run_analyze`. The `--mitre`/`--no-collapse` → bool resolution at `main()` lines 79-80
is UNCHANGED. With this migration: `--mitre` alone (`show_mitre_grouping=true`) → first branch →
`{Grouped, Expanded}` (suffix-free, byte-identical to old `FindingsRender::Grouped`).
`{Grouped, Collapsed}` is NOT produced by `run_analyze` in this story — it is unreachable via
the CLI in STORY-122/A. The construction flip (`--mitre` → `{Grouped, Collapsed}`) is
STORY-119/B's scope (Task 4).
(traces to BC-2.11.028 Invariant 1: "…`render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }`… The two axes are fully orthogonal: no combination is illegal." — NOTE: The invariant describes the FINAL struct wiring form (as installed by STORY-119/B). STORY-122/A preserves the v0.9.0 3-arm-if branching order as 3 struct literals, which is functionally equivalent to BC-2.11.028 Invariant 1 for the 3 CLI-reachable combos AND preserves byte-identical output. The orthogonal form is STORY-119/B's Task 4 — BC-2.11.028's Architecture Anchor explicitly marks it "F4-pending STORY-119 target.")

### AC-005 — `run_summary` construction site uses `{Flat, Collapsed}` struct literal
The `run_summary` construction site in `src/main.rs` uses struct literal form
`render: FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`.
The inert value semantics are unchanged — `run_summary` renders no FINDINGS section.
(traces to BC-2.11.028 Postcondition 4: "The `collapse` axis of the `FindingsRender` struct is determined exclusively by `collapse_findings`; the `grouping` axis by `show_mitre_grouping`. They are fully orthogonal.")

### AC-006 — Output byte-identical to v0.9.0 for all inputs; all existing tests pass
After implementing all tasks, `cargo build --all-targets` compiles with zero errors.
`cargo test --all-targets` passes with zero test failures. All pre-existing tests in
`story_118`, `story_078`, `story_077`, and all other modules pass unmodified. No observable
output change for any combination of CLI flags because the `{Grouped, Collapsed}` combination
routes to `render_findings_grouped` (same as `{Grouped, Expanded}`) and was unreachable via
the old three-variant enum anyway.
(traces to BC-2.11.013 Invariant 4: "When `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (`--mitre --no-collapse`): the collapse pass is NOT applied. Each finding is rendered individually via one `render_finding_grouped` call, with no `(xN)` count suffix. This is the pre-STORY-119 `--mitre` behavior, now explicitly selected via `--no-collapse`.")

### AC-008 — Governing-BC preservation: BC-2.11.016 em-dash MITRE format, BC-2.11.026 flat `(xN)` suffix, and BC-2.11.027 K=3 evidence sampling are preserved byte-identically across the enum→struct reshape
The construction-site migration (Tasks 1–3) is a type-level refactor only; no function bodies in
`render_findings_grouped`, `render_findings_collapsed`, or `render_finding_flat` are modified.
Consequently, all three flat-mode and grouped-mode rendering paths are byte-identical to v0.9.0.
In particular:
- **BC-2.11.016 (em-dash MITRE name expansion):** The MITRE line is rendered by `render_finding_grouped`, called only in grouping mode. After the reshape, the `{Grouped, Expanded}` and TEMPORARY `{Grouped, Collapsed}` arms both route to `render_finding_grouped` — the em-dash expansion function is called identically to v0.9.0. All BC-2.11.016 tests in `mod story_078` (e.g., `test_BC_2_11_016_known_id_em_dash_and_name`) continue to pass without modification.
(traces to BC-2.11.016 Invariant 3: "The MITRE line is rendered by `render_finding_grouped`, called only in grouping mode.")
- **BC-2.11.026 (flat `(xN)` suffix and OBSERVABLE LINE ORDER):** The `{Flat, Collapsed}` dispatch arm routes to `render_findings_collapsed` — unchanged. No edit is made to the flat-mode header line construction, the suffix appending, the colorization path, or the MITRE line ordering within the flat-collapse output. All BC-2.11.026 tests in `mod story_118` continue to pass without modification.
(traces to BC-2.11.026 Postcondition 4: "**OBSERVABLE LINE ORDER — FLAT-COLLAPSE PATH ({Flat, Collapsed}):** For a collapsed group the terminal emits, in order: (1) the header line `  [<Category>] <VERDICT> (<CONFIDENCE>) - <escaped_summary> (x<N>)\n` (colorized, suffix included when N≥2), (2) up to K=3 sampled evidence lines each passed through `escape_for_terminal` (BC-2.11.027), (3) the MITRE line `    MITRE: <ids>\n` where `<ids>` is `mitre_techniques.join(\", \")` from the group representative (`group_members[0]`) — emitted only IF `mitre_techniques` is non-empty (BC-2.11.017 PC-6).")
- **BC-2.11.027 (K=3 evidence sampling for flat-mode collapsed groups):** The `{Flat, Collapsed}` arm routes to `render_findings_collapsed` — unchanged. No edit is made to the evidence sampling loop, the `COLLAPSE_EVIDENCE_SAMPLES = 3` constant, or the positional no-sliding-window algorithm. All BC-2.11.027 tests in `mod story_118` continue to pass without modification.
(traces to BC-2.11.027 Postcondition 1: "The terminal output for a collapsed group contains at most K=3 evidence lines, each rendered as `    > <escaped_evidence_line>\n`.")

### AC-007 — Comment sweep: stale `FindingsRender::Grouped`/`FlatCollapsed`/`FlatExpanded` enum vocabulary in doc-comments updated; stale `verdict-desc`/`confidence-desc` doc-comment corrected; stale three-variant prose updated
A census of all source files and test files confirms zero remaining references to
`FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded` as
enum variant tokens in code or doc-comments (beyond the zero-grep gate already required by AC-003).
The stale doc-comment on `render_findings_grouped` at `src/reporter/terminal.rs:429-430` reading
"verdict-desc, then confidence-desc" is corrected to ascending sort order: "Within each bucket,
findings sort ascending by verdict-rank (Likely=0, Possible=1, Inconclusive=2, Unlikely=3),
then ascending by confidence-rank (High=0, Medium=1, Low=2), then original emission order (stable)."
The stale semantic prose at `tests/reporter_terminal_tests.rs` lines 3949, 3995, 4005, and 4061
referencing "three-variant enum" / "three arms" / "three variants" vocabulary is updated to
two-orthogonal-axis / four-arm vocabulary, and lines 4115 ('All three outputs'), 4137, and 4155
('impossible state') in `test_BC_2_11_019_findings_dispatch_match_exhaustive` and
`test_BC_2_11_025_grouped_mode_bypasses_collapse_structurally` are also updated. The derive test body (lines 3996-4026) is updated to
use struct literals for all four Grouping×Collapse combinations (see Task 4 for detail). The test
at line 4038 (`test_BC_2_11_028_struct_has_exactly_three_fields_post_refactor`) and its body
comment at line 4040 (`these three fields exist on TerminalReporter`) are EXEMPT — they correctly
describe the TerminalReporter struct field count (3 fields: use_color, show_hosts_breakdown,
render), which is UNCHANGED by the FindingsRender reshape. Only the FindingsRender variant token
at line 4045 within that test (`FindingsRender::FlatExpanded`) is migrated mechanically by Task 2.
(traces to BC-2.11.028 Invariant 6: "Per LESSON-P1.04 (no unwired flags): the `no_collapse` field in `cli.rs` MUST be wired to `TerminalReporter.render` in `main.rs` via the two-field struct expression in Invariant 1 (STORY-119 F4 target)…")
(governing-BC trace: BC-2.11.014 Invariant 1: "Verdict ranks: Likely=0, Possible=1, Inconclusive=2, Unlikely=3 (defined by local `verdict_rank` function in terminal.rs:447-454; source-confirmed match arms).")

---

## Verification Properties

| VP-NNN | Property | Coverage in this Story | Proof Method |
|--------|----------|-----------------------|-------------|
| VP-016 | Tactic headers in canonical order (BC-2.11.013/014/015) | AC-006: `all_tactics_in_report_order()` unchanged; existing integration tests `mitre_grouping_emits_tactic_headers_in_canonical_order` and `mitre_grouping_buckets_none_and_unknown_under_uncategorized` continue to pass under `{Grouped, Expanded}` path (byte-identical to v0.9.0 after migration). The four-arm dispatch routes `{Grouped, Expanded}` identically to the old three-arm dispatch. | integration (existing; unchanged) |

---

## Implementation Tasks

### Task 1 — Replace `FindingsRender` three-variant enum with `Grouping` + `Collapse` + `FindingsRender` struct in `src/reporter/terminal.rs`

**File:** `src/reporter/terminal.rs`
**Scope:** Lines **94**-111 (includes the enum doc-comment block at :94-99 and the `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` definition at :100-111; the wholesale replacement removes the stale doc-comment at :94-99 so no orphaned stale doc remains)

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
- `src/reporter/terminal.rs` — 3 occurrences are the OLD 3-arm match-arm patterns at :203/:209/:216; REWRITTEN by Task 3 (4-arm tuple dispatch), NOT migrated by Task 2's struct-literal map. Counted in the 84 census; satisfied by Task 3.
- `tests/reporter_terminal_tests.rs` — 55 sites across all story_NNN blocks
- `tests/reporter_tests.rs` — 17 `mitre_reporter()` helper sites
- `tests/dnp3_f5_remediation_tests.rs` — 2 (mitre_reporter helper)
- `tests/bc_2_09_100_multitag_tests.rs` — 3 (parameterized helper)

**Migration map (apply to every site):**
| Old enum variant | New struct literal |
|------------------|--------------------|
| `FindingsRender::Grouped` | `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` |
| `FindingsRender::FlatCollapsed` | `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` |
| `FindingsRender::FlatExpanded` | `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }` |

**Wiring for `run_analyze` construction site** (`src/main.rs:381-387` — migrates the 3-arm if-expression to 3 struct literals, preserving v0.9.0 branching order for byte-identical output):
```rust
render: if show_mitre_grouping {
    FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }
} else if collapse_findings {
    FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }
} else {
    FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }
},
```
Note: `show_mitre_grouping` (line 107) and `collapse_findings` (line 108) are the in-scope bool
params inside `run_analyze`. The `--mitre`/`--no-collapse` → bool resolution at `main()` lines
79-80 is UNCHANGED. This wiring produces `{Grouped, Expanded}` for `--mitre` alone (byte-identical to
old `FindingsRender::Grouped`). `{Grouped, Collapsed}` is NOT reachable via `run_analyze` in
STORY-122/A — the construction flip to the orthogonal 2-if form is STORY-119/B's Task 4.
The migration map governs this: `FindingsRender::Grouped` → `{Grouped, Expanded}` (first arm); the
flat arms migrate to `{Flat, Collapsed}` and `{Flat, Expanded}` respectively. No orthogonal-axis
recomputation occurs; no arm yields `{Grouped, Collapsed}`.

**Wiring for `run_summary` construction site** (`src/main.rs`):
```rust
render: FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed },
```

**Acceptance gate:** `cargo build --all-targets` compiles clean with zero errors after all sites
updated. `cargo test --all-targets` passes (all pre-existing tests green; no new failing tests
since this story introduces no new test module).

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
        // TEMPORARY (STORY-122/A): routes to render_findings_grouped until STORY-119/B
        // introduces render_findings_grouped_collapsed and repoints this arm.
        // {Grouped, Collapsed} is unreachable via CLI in this story (--mitre alone maps
        // to {Grouped, Expanded} until STORY-119/B flips the CLI default).
        self.render_findings_grouped(&mut out, findings);
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

**Acceptance gate:** `cargo build` compiles. All three existing arms delegate to the same
existing functions as before — zero behavior change for those paths.

---

### Task 4 — Comment sweep: remove stale enum-vocabulary references and fix stale doc-comments

**Files:** `src/reporter/terminal.rs`, `src/main.rs`, all test files.

Census: grep for `FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded`,
stale `verdict-desc`/`confidence-desc` text, and stale `three-variant`/`three fields`/
`All three FindingsRender`/`three.arm`/`three mutually-exclusive`/`three-way`/`impossible state`/
`three.mode` semantic prose.

**Stale semantic prose targets (src/reporter/terminal.rs — confirmed by grep):**
- Line 6-8 — module-level doc `render: FindingsRender field selects among three mutually-exclusive` → update to two-orthogonal-axis vocabulary (struct-of-two-orthogonal-enums)
- Lines 94-99 — stale doc-comment block on old `FindingsRender` enum (removed wholesale by Task 1 scope extension to :94-111; included in Task 1's replacement block)
- Lines 96-99 — `Encodes the three mutually-exclusive rendering modes` / `eliminating the impossible state` → removed by Task 1 scope; no separate edit needed
- Lines 122-124 — `TerminalReporter.render` field doc referencing "mutual exclusion between ... three ... modes" → update to struct-of-enums vocabulary
- Lines 204-206 — dispatch-arm comment "impossible state eliminated by type" → remove or update (this comment was true for the three-variant enum but is now FALSE under the four-arm struct dispatch where no combination is illegal)

**Stale semantic prose targets (src/main.rs — confirmed by grep):**
- Line 378 — inline comment `BC-2.11.028: three-way render mode selection` → update to `BC-2.11.028: render mode selection (3-arm if for byte-identical STORY-122/A; orthogonal 2-if form in STORY-119/B)`

**Stale semantic prose targets (tests/reporter_terminal_tests.rs):**
- Line 3949 — module-level comment `(a three-variant enum)` → update to `(a struct-of-two-orthogonal-enums: Grouping × Collapse)`
- Line 3995 — doc-comment `The enum has exactly three variants` → update to `The struct has exactly two fields: grouping: Grouping and collapse: Collapse`
- Lines 3996-4026 — body of `test_findings_render_derives_debug_clone_copy_partialeq_eq`: after Task-1 reshapes the type and Task-2 migrates the variant tokens, the existing three-variant assertion logic degenerates (only 3 of the 4 Cartesian combos covered; the "three variants" framing is false — there are no variants, and 4 valid struct-literal combos exist). **Migrate the variant tokens to struct literals AND update the assertions and comments:**
  - Replace `FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded` tokens with struct literals per the migration map (Task 2).
  - Add the 4th combo `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` so all four Cartesian pairs are asserted pairwise distinct.
  - Update the comment `// All three variants are distinct.` → `// All four Grouping × Collapse combinations are pairwise distinct.`
  - Rename/rescope the test from "three variants" to the two-orthogonal-axis struct: update the doc-comment line 3995 and the Debug format assertions (lines 4023-4025) to use struct literal Debug output, e.g. `assert!(format!("{:?}", FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }).contains("Grouped"))`.
  - The test name `test_findings_render_derives_debug_clone_copy_partialeq_eq` may be kept or renamed to `test_findings_render_struct_derives_debug_clone_copy_partialeq_eq`. Any name change must not collide with existing test names.
  - This keeps AC-006 ("all existing tests pass") honest: the test must verify what it claims post-reshape. A test asserting `FindingsRender` derive traits with three-variant enum literals is a non-compiling dead letter after Task 1; it must be updated to assert the same traits for the struct form.
- Line 4061 — doc-comment `All three FindingsRender arms` → update to `All four FindingsRender (Grouping × Collapse) dispatch arms`
- Lines 4115-4127 — comments and assertions in `test_BC_2_11_019_findings_dispatch_match_exhaustive` referencing three outputs (`// All three outputs are mutually distinct.`): this test already constructs Grouped/FlatCollapsed/FlatExpanded inline — migrate those variant tokens to struct literals (Task 2) and update the comment at :4115 to `// All three tested outputs are mutually distinct (three of the four Grouping × Collapse combos).` Note: `:4115` is NOT matched by the prior gate regex (`three.variant|All three FindingsRender|three.arm|three mutually-exclusive|three-way|impossible state`); the extended round-3 gate adds `All three outputs` to catch it (machine-enforced sweep).
- Line 4137 — comment in `test_BC_2_11_025_grouped_mode_bypasses_collapse_structurally` referencing stale three-variant vocabulary: update to two-orthogonal-axis / four-arm vocabulary per the sweep.
- Line 4155 — `impossible state` prose in `test_BC_2_11_025_grouped_mode_bypasses_collapse_structurally`: update to reflect the struct-of-enums type where no combination is illegal (the "impossible state" framing was valid for the three-variant enum but is false for the 2×2 struct).

**EXEMPT (do NOT modify — rationale required):**
- `collapse_findings` as a local bool variable name inside `run_analyze` — intentional.
- `show_mitre_grouping` as a local bool variable name inside `run_analyze` — same rationale.
- Historical BCs and ADR text outside `src/` and `tests/` — spec artifacts.
- Changelog stanzas in STORY files and BC files that historically reference "three-variant enum"
  — frozen historical records.
- **`tests/reporter_terminal_tests.rs:4038` test name `test_BC_2_11_028_struct_has_exactly_three_fields_post_refactor` — EXEMPT.** This test asserts that `TerminalReporter` has exactly three fields (`use_color`, `show_hosts_breakdown`, `render`). That field count is UNCHANGED by the FindingsRender reshape; TerminalReporter still has exactly the same three fields. The test name and its body comment at `:4040` ("these three fields exist on TerminalReporter") describe the TerminalReporter struct field count, NOT FindingsRender variant count. Do NOT rename this test and do NOT edit the comment at `:4040`. The only edit to this test block (`:4038-4053`) is the mechanical Task-2 migration of `FindingsRender::FlatExpanded` at `:4045` to the struct literal `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }`.
- `tests/reporter_terminal_tests.rs:394` — `All three boundary` — EXEMPT: refers to U+009F/U+00A0 escape boundary values, unrelated to FindingsRender.
- `tests/reporter_terminal_tests.rs:3512` — `All three findings must appear.` — EXEMPT: refers to a count of three test findings (not FindingsRender variants).

**Falsifiable requirements after sweep:**
1. `grep -rn "FindingsRender::Grouped\|FindingsRender::FlatCollapsed\|FindingsRender::FlatExpanded" src/ tests/` returns zero lines.
2. `grep -n "verdict-desc\|confidence-desc" src/reporter/terminal.rs` returns zero lines.
3. `grep -n "three.variant\|All three FindingsRender\|All three outputs\|three.arm\|three mutually-exclusive\|three-way\|impossible state" tests/reporter_terminal_tests.rs src/reporter/terminal.rs src/main.rs` returns zero lines for non-exempt targets. Note: the bare token `three fields` is EXEMPT at `:4040` (TerminalReporter-field prose) — do NOT include bare `three fields` in the gate pattern. The pattern `All three outputs` is added here to catch `:4115` ("All three outputs are mutually distinct.") which was not covered by the prior round-2 pattern. Use the scoped pattern above which matches `three.variant`, `All three FindingsRender`, `All three outputs`, `three.arm`, `three mutually-exclusive`, `three-way`, `impossible state` — these match only FindingsRender-scoped stale prose, not the TerminalReporter-field comment.
4. `grep -n "three mutually-exclusive\|three-way\|impossible state\|three.mode" src/reporter/terminal.rs src/main.rs` returns zero lines (covers the src/ stale-prose targets).

The grep gate in (3) and (4) are tightened in STORY-122 v1.2 to avoid false-positive matches against the EXEMPT TerminalReporter-field comment at `:4040`. The F4 implementer MUST run all four gates after completing all tasks and confirm all return zero non-exempt lines.

---

### Task 5 — Verify `cargo test --all-targets` green and compute input-hash

Run `cargo clippy --all-targets -- -D warnings` (must be clean). Run `cargo test --all-targets`
(must be green). Run `cargo fmt --check` (must pass). Then run:
```
bin/compute-input-hash --write .factory/stories/STORY-122.md
bin/compute-input-hash .factory/stories/STORY-122.md
```
Record the computed hash in the `input-hash` frontmatter field.

---

## Previous Story Intelligence

**Predecessor:** STORY-120 (F4 complete and merged at commit f851995 on develop). Key applicable lessons:

1. **Variable scope in ACs:** `*mitre` and `no_collapse` are owned by `main()` (lines 79-80), NOT by `run_analyze`. All AC code blocks referencing the construction site use the in-scope params `show_mitre_grouping` (line 107) and `collapse_findings` (line 108) inside `run_analyze`. Do NOT write `self.findings` — `TerminalReporter` has no `findings` field (only `use_color`, `show_hosts_breakdown`, `render`).

2. **Verdict rank is 4-valued, ascending:** Likely=0, Possible=1, Inconclusive=2, Unlikely=3 (source-confirmed in `terminal.rs:447-454`). Within-bucket sort is ascending by rank value. All BC citations on sort direction use this vocabulary.

3. **`{Grouped, Collapsed}` dispatch arm is TEMPORARY in this story:** Under Option X, the `run_analyze` construction site in STORY-122/A uses a 3-arm if migrated to 3 struct literals — `show_mitre_grouping=true` → `{Grouped, Expanded}` (first arm wins; byte-identical to old `FindingsRender::Grouped`). The `{Grouped, Collapsed}` arm EXISTS in the four-arm tuple dispatch but is UNREACHABLE via the CLI in this story (no run_analyze path produces that struct value). The arm must still be present (compiler exhaustiveness requires covering all 4 combinations) and must route to `render_findings_grouped` with an explicit `// TEMPORARY (STORY-122/A)` comment. STORY-119/B's Task 4 replaces the 3-arm if with the orthogonal 2-if form, making `--mitre` alone produce `{Grouped, Collapsed}`, which then dispatches to `render_findings_grouped_collapsed` (STORY-119/B's Task 3).

4. **No Default on FindingsRender/Grouping/Collapse:** Consistent with the deliberate omission in STORY-120 (ADR-0003 "Default Derive: Deliberate Omission"). All construction sites select both axes explicitly.

5. **Content-based citations in cross-indexes:** Use content-based citations (entry text) rather than line numbers when referencing BC-INDEX, as changelog prepends cause line drift.

**From STORY-118** (flat-mode collapse predecessor):
- `escape_for_terminal` is called at render time, not at key-construction time.
- The `COLLAPSE_EVIDENCE_SAMPLES = 3` constant at `terminal.rs:73` is shared; do not duplicate.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and ADR-0003:

1. **ADR-0003 Binding Rule 5 (revised, STORY-122):** The `FindingsRender` type is now a struct-of-two-orthogonal-enums, not an enum. All four Cartesian combinations are valid. This is the outcome of the D-110 gate decision.

2. **No Default on FindingsRender/Grouping/Collapse:** Per ADR-0003 "Default Derive: Deliberate Omission". All 84 migrated construction sites must explicitly specify both `grouping` and `collapse` fields.

3. **Path separation invariant:** The four dispatch arms must remain structurally separate — even though in STORY-122/A the `{Grouped, Collapsed}` arm TEMPORARILY calls `render_findings_grouped`, it must be a distinct arm with a `// TEMPORARY` comment, not merged with `{Grouped, Expanded}`.

4. **L4 Output layer constraint:** SS-11 (`reporter/terminal.rs`) must not import or call modules in L1 Ingest (SS-01/02) or L2 Stream (SS-04). This story introduces no new imports.

5. **ADR-0003 Binding Rule 2:** `escape_for_terminal` must be called on every `summary` and `evidence` string before terminal output. This story does not modify any rendering function bodies — the invariant is maintained by transitivity.

---

## Forbidden Dependencies

- **STORY-122 MUST NOT be dispatched to F4 before STORY-120 is merged and CI is green.** The three-variant `FindingsRender` enum does not exist until STORY-120 ships. The F4 implementer builds against the post-STORY-120 codebase.

- **`render_findings_grouped` body MUST NOT be modified.** Its body is called for both `{Grouped, Expanded}` and the TEMPORARY `{Grouped, Collapsed}` arm. Any modification would break byte-identical guarantee.

- **No new crate dependencies.** This story introduces no new entries in `Cargo.toml`.

- **`render_findings_grouped_collapsed` MUST NOT be introduced in this story.** That function is STORY-119/B's scope. Introducing it here would make the split ineffective.

- **`collapse_findings_pass_refs` MUST NOT be introduced in this story.** That is STORY-119/B's scope (F4-new shared helper).

---

## Library & Framework Requirements

Version pins from `dependency-graph.md` (do not invent version numbers):

| Library | Version (from dep-graph) | Usage in this story |
|---------|--------------------------|---------------------|
| `owo-colors` | per `Cargo.toml` (same as STORY-118) | Color output — no change in this story |
| `etherparse` | 0.20 (per STORY-111 migration) | Packet decode — unchanged by this story |
| `std` `HashMap` | stdlib | Tactic bucket structure — unchanged |

No new crates. This story does not introduce any new external dependencies.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reporter/terminal.rs` | **Modify** | (1) Replace `FindingsRender` 3-variant enum with `Grouping` + `Collapse` + `FindingsRender` struct (Task 1). (2) Update 3-arm dispatch to 4-arm tuple dispatch with TEMPORARY `{Grouped,Collapsed}` → `render_findings_grouped` (Task 3). (3) Comment sweep incl. stale `verdict-desc`/`confidence-desc` fix (Task 4). |
| `src/main.rs` | **Modify** | (1) Update `use` import to include `Collapse, Grouping`. (2) Rewrite `run_analyze` construction site to struct literal (Task 2). (3) Rewrite `run_summary` site to struct literal (Task 2). |
| `tests/reporter_terminal_tests.rs` | **Modify** | Migrate all `FindingsRender::Variant` sites to struct literals (Task 2). Update stale three-variant semantic prose (Task 4). |
| `tests/reporter_tests.rs` | **Modify** | Migrate all `FindingsRender::Grouped/FlatCollapsed/FlatExpanded` sites to struct literals (Task 2). |
| `tests/dnp3_f5_remediation_tests.rs` | **Modify** | Migrate `mitre_reporter` helper `FindingsRender::Grouped` site to struct literal (Task 2). |
| `tests/bc_2_09_100_multitag_tests.rs` | **Modify** | Migrate parameterized helper `FindingsRender` sites to struct literals (Task 2). |
| `Cargo.toml` | **Verify** | Confirm version is `0.9.0`; this story does not change the version (STORY-120 already bumped it). |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `Grouping` enum (new) | `src/reporter/terminal.rs` | Pure data |
| `Collapse` enum (new) | `src/reporter/terminal.rs` | Pure data |
| `FindingsRender` struct (replaces 3-variant enum) | `src/reporter/terminal.rs` | Pure data |
| `match (self.render.grouping, self.render.collapse)` 4-arm dispatch (replaces 3-arm) | `src/reporter/terminal.rs` | Pure core |
| `run_analyze` construction site (struct literal) | `src/main.rs` | Effectful (CLI entry point) |
| `run_summary` construction site (struct literal) | `src/main.rs` | Effectful (CLI entry point) |

**Architecture Anchors (post-STORY-120 / pre-STORY-122 state at HEAD f851995):**
- `src/reporter/terminal.rs:100-111` — current `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` (Task 1 replacement target)
- `src/reporter/terminal.rs:202-224` — current `match self.render` 3-arm dispatch (Task 3 replacement target)
- `src/reporter/terminal.rs:432-483` — `render_findings_grouped` (Task 3 TEMPORARY call target; DO NOT MODIFY its body)
- `src/reporter/terminal.rs:376-423` — `render_findings_collapsed` (flat-mode; unchanged)
- `src/reporter/terminal.rs:447-454` — `verdict_rank` function (4-valued ascending: Likely=0, Possible=1, Inconclusive=2, Unlikely=3)
- `src/main.rs:381-387` — current 3-arm if-expression for `FindingsRender` construction (Task 2 replacement target)
- `src/main.rs:107` — `show_mitre_grouping: bool` in-scope param in `run_analyze`
- `src/main.rs:108` — `collapse_findings: bool` in-scope param in `run_analyze`

**Subsystem anchor:** SS-11 owns this story's scope because FindingsRender struct reshape is a display-layer type definition in reporter/terminal.rs per ARCH-INDEX Subsystem Registry.

**Dependency anchor:** STORY-122 depends on STORY-120 because STORY-120 introduces the three-variant `FindingsRender` enum that STORY-122 reshapes. STORY-122 blocks STORY-119 because STORY-119/B's `render_findings_grouped_collapsed` dispatch requires the struct-of-enums type that STORY-122 establishes.

---

## Token Budget Estimate

| Context item | Estimated tokens |
|-------------|-----------------|
| This story file (STORY-122.md) | ~5,000 |
| BC files (6 BCs × ~1,800 avg) | ~10,800 |
| `src/reporter/terminal.rs` (current, ~500 lines) | ~8,000 |
| `src/main.rs` (current, ~550 lines) | ~8,800 |
| All test files combined (reporter_terminal_tests.rs ~2500 lines + reporter_tests.rs ~600 lines + others ~150 lines) | ~20,000 |
| F2 design note (story-119-type-design.md) | ~3,500 |
| F1 delta-analysis | ~5,000 |
| Tool outputs (grep, cargo test) | ~3,000 |
| **Total estimated** | **~64,100** |

**Assessment:** ~64,100 tokens ≈ 26% of an agent context window (250k tokens). Within the 20-30% guideline. Load BC files on-demand (only the BC for the specific AC being implemented) rather than all 6 at once to stay well within budget.

---

## Dependencies

- `depends_on: [STORY-120]` — STORY-120 introduces the three-variant `FindingsRender` enum; STORY-122 reshapes that enum into the struct-of-orthogonal-enums. The type must exist (STORY-120) before STORY-122 can reshape it.
- `blocks: [STORY-119]` — STORY-119/B requires the `Grouping`, `Collapse`, and `FindingsRender` struct types defined here. STORY-119/B also requires the four-arm dispatch established here so it can repoint the `{Grouped, Collapsed}` arm. STORY-119 cannot be dispatched before STORY-122 merges.

---

## Edge Cases Specific to This Story

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--mitre` alone (no `--no-collapse`) in STORY-122/A | `run_analyze` produces `{Grouped, Expanded}` (first arm of the 3-arm if: `show_mitre_grouping=true` → `{Grouping::Grouped, Collapse::Expanded}`). Routes to `render_findings_grouped`; output byte-identical to v0.9.0 `--mitre` output (which was `FindingsRender::Grouped`). No `(xN)` suffixes. `{Grouped, Collapsed}` is UNREACHABLE via the CLI in STORY-122/A — the arm exists in the 4-arm dispatch but `run_analyze` never produces that struct value. STORY-119/B Task 4 replaces the 3-arm if with the orthogonal 2-if form, making `--mitre` alone produce `{Grouped, Collapsed}`. |
| EC-002 | `--mitre --no-collapse` constructs `{Grouped, Expanded}` | Routes to `render_findings_grouped`; byte-identical to v0.9.0. |
| EC-003 | Default (no `--mitre`, no `--no-collapse`) constructs `{Flat, Collapsed}` | Routes to `render_findings_collapsed`; byte-identical to v0.9.0. |
| EC-004 | Site with `FindingsRender::Grouped` in a test helper | Migrated to `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` — preserves existing test semantics. |
| EC-005 | `run_summary` construction site | Always produces `{Flat, Collapsed}`; inert (no FINDINGS section rendered). Byte-identical to v0.9.0. |
| EC-006 | Tests referencing stale "three-variant" / "three arms" prose | These are in test doc-comments/module-level comments; they must be updated per Task 4 sweep targets. Frozen changelog stanzas in STORY files are EXEMPT. |

---

## Changelog

- **v1.0 (D-120 split, 2026-06-18):** Created as STORY-122/A from the D-120 human-confirmed split of monolithic STORY-119 v1.12. Scope: FindingsRender enum→struct reshape + 84-site migration (byte-identical). ACs drawn from STORY-119 v1.12 AC-005 (struct def), AC-006 (dispatch), AC-007 (84-site migration), AC-028 (byte-identical {Grouped,Expanded}), AC-029 (flat paths byte-identical), AC-030 (comment sweep), and the struct-construction wiring ACs AC-001/AC-003/AC-004. The `{Grouped,Collapsed}` TEMPORARY routing note (AC-002) is new to STORY-122/A — the monolithic story had this arm dispatching to `render_findings_grouped_collapsed` (a function that STORY-122/A does NOT introduce). BC set reduced to 6 (removed BC-2.11.025/030/031/032/033/034 which govern the behavioral render path in STORY-119/B). points set to 3 (migration-only, like STORY-120). wave 49 = max(STORY-120=48)+1.
- **v1.1 (F3-resplit round-1 remediation, 2026-06-18):** Reconciled to Option X (human-approved split design). Previously AC-004 and Task 2 prescribed the orthogonal 2-if struct wiring (`{grouping: if .., collapse: if ..}`), which would produce `{Grouped, Collapsed}` for `--mitre` alone and contradicted the claim in AC-006/EC-001 that `{Grouped, Collapsed}` is unreachable via CLI in Story A. Fixed: AC-004 now prescribes the 3-arm if migrated to 3 struct literals (byte-identical to v0.9.0 branching: `--mitre` alone → `{Grouped, Expanded}`). Task 2 run_analyze wiring updated to match. EC-001 corrected to say `--mitre` alone → `{Grouped, Expanded}` in A; `{Grouped, Collapsed}` unreachable-via-CLI note preserved and made accurate. Task 1 scope extended to Lines 94-111 (include :94-99 doc-comment block in wholesale replacement). Task 4 extended to cover src/ stale prose targets (`src/reporter/terminal.rs:6-8`, `:122-124`, `:204-206`; `src/main.rs:378`) and added falsifiable grep gate (4): `grep -n 'three mutually-exclusive|three-way|impossible state|three.mode' src/reporter/terminal.rs src/main.rs` must return zero lines after F4. Migration map note added: no arm in the 3-arm if yields `{Grouped, Collapsed}`, so the unreachable claim is structurally accurate.
- **v1.4 (F3-resplit round-4 remediation, 2026-06-19):** Fix 1 HIGH (Pass B F-B-001 — STORY-122 BC-016/026/027 undischarged governance): added AC-008 discharging the three governing BCs (BC-2.11.016, BC-2.11.026, BC-2.11.027) that were listed in the frontmatter `bcs:` array and body BC table but had no AC-level traces. AC-008 asserts byte-identical preservation of the em-dash MITRE format, flat `(xN)` suffix observable line order, and K=3 evidence sampling across the enum→struct reshape. Verbatim clause quotes: BC-2.11.016 Invariant 3, BC-2.11.026 Postcondition 4, BC-2.11.027 Postcondition 1. No BC edits, no dep-graph edits, no frontmatter BC set change.
- **v1.3 (F3-resplit round-3 remediation, 2026-06-19):** Four targeted fixes. (1) Fix 1 (Pass A F-A-001 MEDIUM): Relabeled Task-2 census entry for `src/reporter/terminal.rs` from "3 sites (test helpers / inline construction)" to the accurate description — the 3 occurrences are the OLD 3-arm MATCH-ARM PATTERNS at :203/:209/:216 removed by Task 3's 4-arm tuple rewrite, NOT struct-literal construction sites handled by Task 2. Added note to AC-003 that the terminal.rs trio is satisfied by Task 3 (not Task 2) and the zero-grep gate still holds after Task 3. (2) Fix 2 (Pass A F-A-002 MEDIUM): Extended AC-007 stale-prose target list to include lines 4115 ('All three outputs'), 4137, and 4155 ('impossible state'). Extended grep gate (3) in Falsifiable Requirements to add `All three outputs` pattern, making `:4115` machine-caught (previously sweep-by-instruction-only). Added lines 4137 and 4155 to Task-4 target list with per-line rationale. (3) Fix 4 (Pass C NIT): Removed doubled word "byte-identical byte-identical" in the Task-4 `src/main.rs:378` prescribed comment → "byte-identical STORY-122/A". (4) No BC edits, no dep-graph edits.
- **v1.2 (F3-resplit round-2 remediation, 2026-06-18):** Three targeted fixes. (1) Fix 1 (Pass A F-A-002+F-A-003): removed `:4038` and `:4040` from Task-4 stale-prose TARGET list — the test `test_BC_2_11_028_struct_has_exactly_three_fields_post_refactor` and its comment "these three fields exist on TerminalReporter" correctly describe the TerminalReporter field count (still 3: use_color, show_hosts_breakdown, render), NOT FindingsRender variant count; added explicit EXEMPT entry for `:4038`/`:4040` with rationale in Task 4 and AC-007. Tightened grep gate (3): bare `three fields` dropped from the pattern; scoped to `three.variant`, `All three FindingsRender`, `three.arm`, `three mutually-exclusive`, `three-way`, `impossible state` — FindingsRender-scoped only, not TerminalReporter-field prose. (2) Fix 2 (Pass A F-A-001): AC-007 and Task 4 now explicitly instruct updating the derive test body (lines 3996-4026): migrate variant tokens to struct literals AND add the 4th Grouping×Collapse combo `{Grouped, Collapsed}` AND update comments and Debug assertions to reflect the struct form, so the test verifies what it claims post-reshape. (3) Minor: `:4115` comment `All three outputs are mutually distinct` added to Task-4 target list (in `test_BC_2_11_019_findings_dispatch_match_exhaustive`); similarly `:4137`/`:4155` impossible-state prose in `test_BC_2_11_025_grouped_mode_bypasses_collapse_structurally` added.
