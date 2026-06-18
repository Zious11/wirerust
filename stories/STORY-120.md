---
document_type: story
story_id: STORY-120
epic_id: E-8
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-18T00:00:00Z
phase: f3
points: 3
priority: P0
depends_on: []
blocks: [STORY-119]
behavioral_contracts:
  - BC-2.11.010
  - BC-2.11.013
  - BC-2.11.014
  - BC-2.11.015
  - BC-2.11.016
  - BC-2.11.017
  - BC-2.11.019
  - BC-2.11.025
  - BC-2.11.026
  - BC-2.11.027
  - BC-2.11.028
  - BC-2.11.029
verification_properties: [VP-012]
tdd_mode: strict
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 1
feature_id: issue-62-enum-modes
github_issue: 62
wave: 48
# BC status: BC-2.11.013 v1.12, BC-2.11.014 v1.7, BC-2.11.015 v1.8, BC-2.11.016 v1.7,
#             BC-2.11.017 v1.14, BC-2.11.019 v1.7, BC-2.11.025 v1.8, BC-2.11.026 v1.9,
#             BC-2.11.027 v1.5, BC-2.11.028 v1.5, BC-2.11.029 v1.4, BC-2.11.010 v1.9
#             — all 12 re-anchored and CONVERGED (F2 passes complete 2026-06-17/18).
# Subsystem anchor: SS-11 owns this story's scope because TerminalReporter and the new
#   FindingsRender enum live in src/reporter/terminal.rs (SS-11 per ARCH-INDEX Subsystem
#   Registry). The two main.rs construction sites are thin SS-12 wiring — insufficient
#   scope to split into a separate story (following the --no-collapse / --mitre precedent
#   from STORY-118 and STORY-078).
# Dependency anchor:
#   depends_on: [] — STORY-120 refactors the TerminalReporter struct that already exists
#   (landed in STORY-077/078, extended in STORY-118). No new build-order predecessor is
#   required; the v0.8.0 fields being replaced are already present in the codebase.
#   STORY-118 is completed and its struct is the refactor target.
#   blocks: [STORY-119] — STORY-119 (grouped-mode collapse) references FindingsRender
#   variants in its own scope; it is logically easier to author against the enum vocabulary.
#   More importantly, the STORY-119 implementer will build on the enum struct introduced by
#   STORY-120. If STORY-119 were delivered before STORY-120, its construction sites would
#   use the old bool fields and would require a second sweep when STORY-120 landed. The
#   blocks relationship preserves logical sequencing without introducing a hard compile
#   dependency (STORY-119 is deferred/unscheduled).
# ADR: ADR-0003 "Render-Mode Enum (Issue #62 — v0.9.0)" subsection + Binding Rule 5.
#   The STORY-120 implementation PR MUST include the ADR-0003 subsection if it has not
#   already landed on develop. The F2 PRD delta confirms the subsection was added
#   (docs/adr/0003-reporting-pipeline-layering.md amended 2026-06-17).
inputs:
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.014.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.015.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.016.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.019.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.025.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.028.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.029.md
  - .factory/phase-f1-delta-analysis/issue-62-terminal-reporter-enum-modes-delta-analysis.md
  - docs/adr/0003-reporting-pipeline-layering.md
input-hash: "cfa60a9"
---

# STORY-120: TerminalReporter FindingsRender Enum Migration (v0.9.0)

## Narrative

- **As a** wirerust contributor adding or testing TerminalReporter behavior
- **I want** the three mutually-exclusive rendering modes (grouped, flat-collapsed,
  flat-expanded) expressed as a single `FindingsRender` enum field instead of two
  independent bools
- **So that** the impossible state (`show_mitre_grouping = true && collapse_findings = true`)
  is structurally unrepresentable, the dispatch becomes an exhaustive `match`, and all 28
  construction sites are trivially correct by Rust's struct-literal exhaustiveness check

## Behavioral Contracts

| BC | Version | Title |
|----|---------|-------|
| BC-2.11.013 | v1.12 | MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last |
| BC-2.11.014 | v1.7  | Within Tactic Bucket: Sort by Verdict, Confidence, Emission Order |
| BC-2.11.015 | v1.8  | No-Technique or Unknown-ID Findings Land in Uncategorized |
| BC-2.11.016 | v1.7  | MITRE Grouping Expands Per-Finding Line with Em-Dash and Name |
| BC-2.11.017 | v1.14 | Default Rendering Emits MITRE: <id(s)> Only (No Em-Dash) |
| BC-2.11.019 | v1.7  | TerminalReporter Renders Sections in Correct Order |
| BC-2.11.025 | v1.8  | Flat-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key; First-Occurrence Order; Deterministic |
| BC-2.11.026 | v1.9  | Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix |
| BC-2.11.027 | v1.5  | Collapsed Group Retains at Most K=3 Representative Evidence Lines; Remainder Elided from Terminal Display |
| BC-2.11.028 | v1.5  | --no-collapse Opt-Out Flag Disables Terminal Collapse and Restores One-Line-Per-Finding Rendering; JSON/CSV Unaffected |
| BC-2.11.029 | v1.4  | Collapse is Display-Layer Only; JSON/CSV Reporters Receive Unmodified findings Slice; Non-Repeated Findings Individually Visible in All Outputs |
| BC-2.11.010 | v1.9  | TerminalReporter Escapes Both Summary AND Each Evidence Line |

## Acceptance Criteria

---

### AC-001 — FindingsRender enum defined with exactly three variants and correct derives
*(traces to BC-2.11.025 invariant 5, BC-2.11.013 invariant 4 — type-level mutual exclusion)*

`pub enum FindingsRender` is defined in `src/reporter/terminal.rs` with exactly three
variants: `Grouped`, `FlatCollapsed`, `FlatExpanded`. The enum carries
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. `Default` is NOT derived. The enum
is `pub` (same visibility as `TerminalReporter`).

The variant doc comments match ADR-0003 Binding Rule 5 (byte-identical):
- `Grouped`:
  ```
  /// Group findings by MITRE tactic (`--mitre` flag).
  /// Corresponds to the previous `show_mitre_grouping = true`.
  ```
- `FlatCollapsed`:
  ```
  /// Collapse repeated findings into counted groups (default, v0.8.0+).
  /// Corresponds to the previous `collapse_findings = true, show_mitre_grouping = false`.
  ```
- `FlatExpanded`:
  ```
  /// One display line per raw finding (pre-v0.8.0 behavior, `--no-collapse`).
  /// Corresponds to the previous `collapse_findings = false, show_mitre_grouping = false`.
  ```

- **Test:** `test_findings_render_derives_debug_clone_copy_partialeq_eq`
  (in `tests/reporter_terminal_tests.rs`)

---

### AC-002 — TerminalReporter struct has exactly 3 fields after refactor
*(traces to BC-2.11.028 precondition 3 — struct shape governs render wiring)*

`pub struct TerminalReporter` has exactly three public fields after STORY-120 lands:
`use_color: bool`, `show_hosts_breakdown: bool`, `render: FindingsRender`.
The fields `show_mitre_grouping: bool` and `collapse_findings: bool` are REMOVED.
`use_color` and `show_hosts_breakdown` remain as orthogonal bools (per ADR-0003 Binding
Rule 5 rationale — they are not part of the mutually-exclusive findings-render axis).

- **Verification:** `cargo check --all-targets` is the exhaustiveness gate for this AC.
  A Rust struct-literal requires every field to be named; removing the two old fields
  while adding `render: FindingsRender` means any of the 28 construction sites that
  still references `show_mitre_grouping` or `collapse_findings` is a compile error.
  Conversely, any site that compiles with the three-field struct proves the shape is
  correct. This gate is shared with AC-007: the same `cargo check` run that proves
  all 28 sites compile also proves the struct has exactly the three specified fields.

---

### AC-003 — Dispatch in render() is an exhaustive match over FindingsRender
*(traces to BC-2.11.019 invariant 7 — FINDINGS dispatch routing;*
*BC-2.11.014 postcondition 1 — within-tactic bucket sort by verdict, confidence, emission order invoked via Grouped arm calling render_findings_grouped;*
*BC-2.11.015 postcondition 1 — no-technique / unknown-ID findings land in the Uncategorized bucket, invoked via Grouped arm calling render_findings_grouped;*
*BC-2.11.016 postcondition 1 — grouped mode expands each per-finding MITRE line with em-dash and technique name, invoked via Grouped arm calling render_findings_grouped)*

The FINDINGS section dispatch in `TerminalReporter::render` replaces the
`if self.show_mitre_grouping { ... } else if self.collapse_findings { ... } else { ... }`
chain with a `match self.render { FindingsRender::Grouped => ..., FindingsRender::FlatCollapsed => ..., FindingsRender::FlatExpanded => ... }`.
Rust's exhaustive match enforcement means all three arms are required to compile.
No behavior changes: `Grouped` calls `render_findings_grouped` (which owns the within-tactic
bucket sort per BC-2.11.014, the Uncategorized bucket for no-technique/unknown-ID findings
per BC-2.11.015, and the em-dash + technique-name MITRE line expansion per BC-2.11.016),
`FlatCollapsed` calls the collapse pass (established in STORY-118),
`FlatExpanded` iterates `render_finding_flat` directly.

- **Test:** `test_BC_2_11_019_findings_dispatch_match_exhaustive`
  (in `tests/reporter_terminal_tests.rs` — verifies all three render paths are reachable)

---

### AC-004 — Impossible state is structurally unrepresentable
*(traces to BC-2.11.025 invariant 5 — "FindingsRender::FlatCollapsed implies flat mode by type")*

The combination `show_mitre_grouping = true && collapse_findings = true` that existed as
a representable-but-invalid struct value in v0.8.0 is now unrepresentable. `FindingsRender`
has no variant encoding both `Grouped` and collapsed-flat simultaneously. The previous
`mitre_collapse_reporter` test helper (set `show_mitre_grouping: true, collapse_findings:
true` — a nonsensical combination handled by dispatch order) now maps to
`render: FindingsRender::Grouped` — structurally consistent with what the code was
already doing.

- **Test:** `test_BC_2_11_025_grouped_mode_bypasses_collapse`
  (existing test, updated construction site; still asserts no `(xN)` suffix in grouped mode)

---

### AC-005 — run_analyze construction site wired correctly in main.rs
*(traces to BC-2.11.028 postconditions 1–2, invariant 1 — CLI flag wiring)*

The `TerminalReporter { ... }` construction in `run_analyze` (src/main.rs ~line 373) uses
the in-scope bool params `show_mitre_grouping: bool` (line 107) and `collapse_findings:
bool` (line 108) — NOT the raw CLI flags `*mitre` or `no_collapse`, which are only in
scope inside `main()` (src/main.rs:55-56, 66-83). The construction site reads:
```rust
render: if show_mitre_grouping {
    FindingsRender::Grouped
} else if collapse_findings {
    FindingsRender::FlatCollapsed
} else {
    FindingsRender::FlatExpanded
},
```
This mirrors the F1 migration map exactly. `show_mitre_grouping` is true exactly when
`--mitre` is passed (resolved at the `main()` call site, lines 79-80, UNCHANGED), so
`if show_mitre_grouping` structurally wins over `collapse_findings` — precedence is
preserved. The `show_mitre_grouping` and `collapse_findings` bool fields are removed
from this struct literal (they are now params, not fields).

- **Test:** `test_BC_2_11_028_flag_wired_to_reporter_field`
  (in `tests/reporter_terminal_tests.rs:3326` — post-STORY-120, this test's construction
  sites use `render: FindingsRender::FlatCollapsed` / `render: FindingsRender::FlatExpanded`
  in place of the v0.8.0 `collapse_findings: true/false` bool fields)

---

### AC-006 — run_summary construction site uses FindingsRender::FlatCollapsed (inert)
*(traces to BC-2.11.028 invariant 4 — run_summary emits no FINDINGS section; render field is inert)*

The `TerminalReporter { ... }` construction in `run_summary` (src/main.rs ~line 439) uses
`render: FindingsRender::FlatCollapsed`. The value is structurally a placeholder —
`run_summary` never invokes the FINDINGS section — but `FlatCollapsed` is chosen because
it expresses "if this reporter were ever used to render findings, it would use the v0.8.0
default."

The migrated line comment MUST use enum vocabulary. The old v0.8.0 comment:
```rust
// Set to true for completeness (Rust requires all struct fields to be initialized).
// collapse_findings: true,
```
MUST be replaced with something like:
```rust
// BC-2.11.028 invariant 4: render field is inert for run_summary — no FINDINGS section.
// FlatCollapsed expresses the v0.8.0 default intent for any hypothetical future use.
render: FindingsRender::FlatCollapsed,
```
No surviving `collapse_findings: true` or `"Set to true for completeness"` bool phrasing
is permitted on the migrated line or its adjacent comment.

- **Test:** validated by `cargo check` (all struct fields must be provided; the value is
  not under test for behavioral correctness since `run_summary` has no findings to render)

---

### AC-007 — All 28 construction sites updated; cargo check is the exhaustiveness gate
*(traces to BC-2.11.028 precondition 3 — struct field wiring; BC-2.11.025 precondition 1)*

All 28 `TerminalReporter { ... }` struct literals in the codebase are updated per the F1
migration map. The file counts and variant assignments below are derived mechanically from
`grep -rn 'TerminalReporter {' src/ tests/` (fn-signature lines excluded) cross-referenced
against `grep -rn 'show_mitre_grouping: true'` and `grep -rn 'collapse_findings:'`:

| File | Sites | Grouped | FlatCollapsed | FlatExpanded | Notes |
|------|-------|---------|---------------|--------------|-------|
| `src/main.rs` | 2 | — | 1 (line 439, inert) | — | Line 373 is a 3-way `if` expression (see AC-005); line 439 → `FlatCollapsed` (see AC-006) |
| `tests/reporter_tests.rs` | 17 | 6 | 0 | 11 | Grouped: lines 1001,1036,1071,1106,1155,1192 (all `show_mitre_grouping: true`); remaining 11 → `FlatExpanded` |
| `tests/reporter_terminal_tests.rs` | 7 | 2 | 3 | 2 | Helpers: plain_reporter(71)→FlatExpanded, mitre_reporter(662)→Grouped, collapse_reporter(1789)→FlatCollapsed, collapse_reporter_color(1799)→FlatCollapsed, mitre_collapse_reporter(1809)→Grouped; inline: reporter_on(3346)→FlatCollapsed, reporter_off(3359)→FlatExpanded |
| `tests/dnp3_f5_remediation_tests.rs` | 1 | 1 | 0 | 0 | `mitre_reporter` helper (line 1070) → `Grouped` |
| `tests/bc_2_09_100_multitag_tests.rs` | 1 | — | — | — | `make_terminal` helper (line 690): parameterized — `if mitre_grouping { Grouped } else { FlatExpanded }` |
| **TOTAL** | **28** | **9** | **4** | **13+1param** | |

A successful `cargo check` after the struct change proves all 28 sites are updated — any
missed site is a compile error. Note the distinction between what `cargo check` and
`cargo test` prove: **`cargo check` proves field PRESENCE** (every construction site
compiles with the three-field struct); only **`cargo test` proves variant CORRECTNESS**
(the regression suite verifies that `Grouped`, `FlatCollapsed`, and `FlatExpanded` each
produce the expected output). Both gates are required; AC-008 covers the `cargo test` gate.

- **Test:** `cargo check --all-targets` green (zero `show_mitre_grouping` or
  `collapse_findings` field references remaining in the codebase)

---

### AC-008 — cargo test --all-targets passes with byte-identical output
*(traces to BC-2.11.025 postconditions 1–9, BC-2.11.026 postconditions 1–7, BC-2.11.027 postconditions 1–6, BC-2.11.028 postconditions 1–4, BC-2.11.029 postconditions 1–5)*

After STORY-120, `cargo test --all-targets` passes with zero test regressions. Output is
byte-identical to pre-refactor: all rendering logic in `render_finding_flat`,
`render_finding_prefix`, `render_findings_grouped`, `render_findings_collapsed`,
`collapse_findings_pass`, and `escape_for_terminal` is unchanged. Only the dispatch
mechanism changes (if-chain → match).

- **Test:** all pre-existing tests in `tests/reporter_terminal_tests.rs`,
  `tests/reporter_tests.rs`, `tests/dnp3_f5_remediation_tests.rs`,
  `tests/bc_2_09_100_multitag_tests.rs` pass unchanged.

---

### AC-009 — FindingsRender is importable in test files without re-export
*(traces to BC-2.11.013 precondition 1 — struct field resolution)*

`FindingsRender` is defined `pub` in `src/reporter/terminal.rs`. All test files that
construct `TerminalReporter` with a `render:` field add:
```rust
use wirerust::reporter::terminal::FindingsRender;
```
alongside the existing `use wirerust::reporter::terminal::TerminalReporter;` import. No
re-export via `src/lib.rs` or `src/reporter/mod.rs` is required (this is a binary crate
with no downstream library consumers).

- **Test:** compile success of all four affected test files proves the import path resolves
  (`reporter_terminal_tests.rs`, `reporter_tests.rs`, `dnp3_f5_remediation_tests.rs`,
  `bc_2_09_100_multitag_tests.rs`).

---

### AC-010 — cargo clippy -- -D warnings passes clean
*(traces to BC-2.11.019 invariant 7 — no unreachable arms, no dead code)*

`cargo clippy --all-targets -- -D warnings` produces zero warnings after the refactor.
The `match self.render` dispatch satisfies `fn_params_excessive_bools` (now 2 bools, under
the default threshold of 3). No `dead_code` warnings for removed fields. No `irrefutable_let_patterns`
or partial-match warnings (enum is exhaustive).

- **Test:** CI gate (`cargo clippy --all-targets -- -D warnings` step in `.github/workflows/ci.yml`)

---

### AC-011 — cargo fmt --check passes clean
*(traces to BC-2.11.019 postcondition 1 — structural correctness implies formatting compliance)*

`cargo fmt --check` passes after the refactor. `rustfmt.toml` settings (edition 2024,
max_width = 100) apply to the new enum definition, the struct, and the match arm in
`render()`. The `render: if show_mitre_grouping { ... } else if collapse_findings { ... } else { ... }`
expression in `run_analyze` in main.rs is formatted per rustfmt's expression-alignment rules.

- **Test:** CI gate (`cargo fmt --check` step)

---

### AC-012 — Grouped mode: no (xN) suffix emitted under FindingsRender::Grouped
*(traces to BC-2.11.013 invariant 4 — grouped path is structurally suffix-free)*

Given `TerminalReporter { render: FindingsRender::Grouped, ... }`, when `render()` receives
N=100 findings all sharing the same collapse key, the output contains 100 individual finding
lines and NO ` (xN)` suffix appears anywhere in the FINDINGS section. The `match`
arm for `FindingsRender::Grouped` calls `render_findings_grouped` directly; the collapse
pass is not reachable from this arm.

- **Test:** `test_BC_2_11_013_grouped_mode_suffix_free` (existing, updated construction site)

---

### AC-013 — FlatCollapsed: collapse behavior preserved byte-for-byte
*(traces to BC-2.11.025 postcondition 1 — identical groups collapse; BC-2.11.026 postcondition 1 — (xN) suffix format)*

Given `TerminalReporter { render: FindingsRender::FlatCollapsed, ... }`, all collapse
behaviors established by STORY-118 (BC-2.11.025 through BC-2.11.029) are preserved
without change. The `match` arm for `FindingsRender::FlatCollapsed` calls the same
`collapse_findings_pass` and rendering logic as the pre-refactor `if self.collapse_findings`
branch.

- **Test:** all `test_BC_2_11_025_*`, `test_BC_2_11_026_*`, `test_BC_2_11_027_*`,
  `test_BC_2_11_028_*`, `test_BC_2_11_029_*` tests pass unchanged.

---

### AC-014 — FlatExpanded: one-line-per-finding behavior preserved byte-for-byte
*(traces to BC-2.11.028 postcondition 2 — --no-collapse restores pre-v0.8.0 rendering;*
*BC-2.11.017 postcondition 1 — default/flat rendering emits `MITRE: <id(s)>` only, no em-dash,*
*exercised by both FlatCollapsed and FlatExpanded arms which call render_finding_flat)*

Given `TerminalReporter { render: FindingsRender::FlatExpanded, ... }`, the output is
byte-identical to the pre-v0.8.0 flat rendering: one header line per finding, no ` (xN)`
suffix, full evidence per finding. The `match` arm for `FindingsRender::FlatExpanded`
iterates `render_finding_flat` directly, identical to the pre-refactor `else` branch.
The `FlatCollapsed` arm also invokes `render_finding_flat` for each representative line,
so BC-2.11.017's MITRE-id emission guarantee (no em-dash; plain `MITRE: <id>` format) is
preserved across both flat variants.

- **Test:** `test_BC_2_11_028_no_collapse_flag_one_line_per_finding` (existing, updated)

---

### AC-015 — escape_for_terminal invariant unchanged across all render paths
*(traces to BC-2.11.010 postconditions 1–3, invariant 4 — escape function guarantee is path-independent)*

The `escape_for_terminal` function is unchanged. All three `FindingsRender` match arms
call the same rendering helpers that invoke `escape_for_terminal` — no escape bypass is
introduced by the refactor. VP-012 (escape_for_terminal correctness) is satisfied by
the existing proptest suite without new tests.

- **Test:** `test_BC_2_11_010_escape_in_collapse_path` (existing, updated construction site)

---

### AC-016 — Semver: v0.8.x → v0.9.0; cargo-semver-checks struct_field_missing fires as expected
*(traces to ADR-0003 Binding Rule 5 — semver consequence documented)*

Removing `show_mitre_grouping: bool` and `collapse_findings: bool` (public fields) and
adding `render: FindingsRender` (new public field) constitutes a breaking struct API change
under Cargo semver (RFC 1105). For a `0.y.z` crate, this requires a minor bump: `0.8.x →
0.9.0`. Running `cargo semver-checks` against a `0.8.x` baseline will fire the
`struct_field_missing` lint for the two removed fields. This is expected and correct — not
a defect. The Cargo.toml version is bumped to `0.9.0` in the STORY-120 PR.

- **Test:** `cargo-semver-checks check` fires `struct_field_missing` (informational, expected);
  the PR description documents this as the intentional v0.9.0 semver boundary.

---

### AC-017 — No test-file comment references removed bool fields post-refactor
*(process guard: DF-GREEN-DOC-TENSE-SWEEP / DF-SIBLING-SWEEP — stale comments are a*
*documentation correctness obligation; traces to BC-2.11.028 postcondition 3 — the*
*struct shape change propagates to all four test files including their comment vocabulary)*

After STORY-120 lands, `grep -n 'collapse_findings\|show_mitre_grouping' tests/` returns
only the residue listed in the **EXEMPT allow-list** below. Every non-exempt match is a
forward-facing assertion about the removed struct fields and MUST be rewritten to enum
vocabulary (`FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`,
`FindingsRender::FlatExpanded`).

#### Forward-Facing Comment Sweep Targets (MUST be rewritten)

The following lines reference removed fields in a forward-facing/assertive manner and are
in scope for Task 7b rewriting. Line numbers are pre-refactor baselines; minor shifts are
expected — locate by surrounding context.

| File | Approx. Line | Content (pre-refactor) | Rewrite action |
|------|-------------|------------------------|----------------|
| `reporter_terminal_tests.rs` | 696 | `/// When show_mitre_grouping = true, tactic section headers appear` | Replace `show_mitre_grouping = true` with `render = FindingsRender::Grouped` |
| `reporter_terminal_tests.rs` | 1011 | `/// When show_mitre_grouping = true and a finding has a known technique ID` | Replace with `render = FindingsRender::Grouped` |
| `reporter_terminal_tests.rs` | 1086 | `/// When show_mitre_grouping = false (default)` | Replace with `render != FindingsRender::Grouped (FlatExpanded/FlatCollapsed)` |
| `reporter_terminal_tests.rs` | 1100 | `// Canonical test vector: mitre="T1036", show_mitre_grouping=false.` | Replace `show_mitre_grouping=false` with `render=FlatExpanded` |
| `reporter_terminal_tests.rs` | 1107 | `// Use plain_reporter() (show_mitre_grouping = false).` | Replace with `render=FindingsRender::FlatExpanded` |
| `reporter_terminal_tests.rs` | 2068 | `/// AC-005: show_mitre_grouping=true suppresses collapse` | Replace with `render=FindingsRender::Grouped suppresses collapse` |
| `reporter_terminal_tests.rs` | 2078 | `// BC-2.11.025 invariant 5: when show_mitre_grouping=true, collapse does NOT run.` | Replace with `render=FindingsRender::Grouped` |
| `reporter_terminal_tests.rs` | 2390 | `// Collapse reporter (collapse_findings=true) with a single finding.` | Replace with `render=FindingsRender::FlatCollapsed` |
| `reporter_terminal_tests.rs` | 2400 | `// Output must be byte-identical to the pre-v0.8.0 path (collapse_findings=false).` | Replace `collapse_findings=false` with `render=FindingsRender::FlatExpanded` |
| `reporter_terminal_tests.rs` | 3218 | `/// This guards the opt-out path: with collapse_findings=false, 5 identical-key` | Replace `collapse_findings=false` with `render=FindingsRender::FlatExpanded` |
| `reporter_terminal_tests.rs` | 3221 | `/// checking that collapse_findings=true produces a different (collapsed) result.` | Replace with `render=FindingsRender::FlatCollapsed` |
| `reporter_terminal_tests.rs` | 3222 | `/// FAILS if collapse_findings=false collapses` | Replace with `FlatExpanded` |
| `reporter_terminal_tests.rs` | 3226 | `// BC-2.11.028 pc2: collapse_findings=false → one header per finding, no suffix.` | Replace `collapse_findings=false` with `render=FindingsRender::FlatExpanded` |
| `reporter_terminal_tests.rs` | 3240 | `// plain_reporter() has collapse_findings=false.` | Replace with `render=FindingsRender::FlatExpanded` |
| `reporter_terminal_tests.rs` | 3257 | `// Contrast assertion: collapse_findings=true on the same input MUST produce` | Replace with `FindingsRender::FlatCollapsed` |
| `reporter_terminal_tests.rs` | 3270 | `/// FAILS if a future change makes collapse_findings=true and collapse_findings=false` | Replace both with `FlatCollapsed` and `FlatExpanded` |
| `reporter_terminal_tests.rs` | 3320–3324 | docstring: `--no-collapse flag is wired: no_collapse=true → collapse_findings=false` + `FAILS if the collapse_findings field does not exist` | Rewrite: `FAILS if FlatCollapsed and FlatExpanded produce identical output / if the render variant is mis-wired to the wrong rendering path.` |
| `reporter_terminal_tests.rs` | 3328–3329 | `// collapse_findings=true → collapse active` + `// collapse_findings=false → collapse inactive` | Replace with `FlatCollapsed` / `FlatExpanded` vocabulary |
| `reporter_terminal_tests.rs` | 3543 | `/// This guards that the collapse_findings field on TerminalReporter does not leak` | Replace with `render field (FindingsRender::FlatCollapsed)` |
| `reporter_terminal_tests.rs` | 3550–3551 | `// BC-2.11.029 pc5: JSON output must be identical regardless of collapse_findings.` + `// The collapse_findings field belongs to TerminalReporter only.` | Replace `collapse_findings` with `render` / `FindingsRender` |
| `reporter_terminal_tests.rs` | 3659 | `/// regardless of collapse_findings flag; 0 (xN) suffixes in any volume.` | Replace `collapse_findings flag` with `render variant` |
| `reporter_terminal_tests.rs` | 3669 | `// 50 identical-key findings with collapse_findings=true AND show_mitre_grouping=true.` | Replace with `FindingsRender::FlatCollapsed AND FindingsRender::Grouped` |
| `reporter_terminal_tests.rs` | 3774 | `/// AC-025: overall section order is unchanged when collapse_findings=true.` | Replace `collapse_findings=true` with `render=FindingsRender::FlatCollapsed` |
| `bc_2_09_100_multitag_tests.rs` | 766 | `/// Flat view (show_mitre_grouping=false).` | Replace with `render=FindingsRender::FlatExpanded` |

#### EXEMPT Allow-List (post-sweep residue — these matches are expected to remain)

The following lines match `collapse_findings\|show_mitre_grouping` but are EXEMPT from
the sweep. After Task 7b, `grep -n 'collapse_findings\|show_mitre_grouping' tests/` MUST
return ONLY these lines (plus any assertion-message string-literals-under-test that capture
current test failure messages):

| File | Approx. Line | Exemption reason |
|------|-------------|-----------------|
| `reporter_terminal_tests.rs` | 68 | Past-tense STORY-118 changelog narration: `STORY-118: collapse_findings field added; default false so existing tests` — historical, past-tense |
| `reporter_terminal_tests.rs` | 659 | Past-tense STORY-118 changelog narration: `STORY-118: collapse_findings field added; false here since grouped mode` — historical, past-tense |
| `reporter_terminal_tests.rs` | 2099 | Assertion message string literal under test: `collapse_findings=true; got:\n{out}` — testing pre-enum behavior described in the message; becomes stale after Task 7 construction-site update but the assertion message itself may remain if the test behavior is preserved |
| `reporter_terminal_tests.rs` | 3247 | Assertion message string literal: `"BC-2.11.028 pc2: collapse_findings=false must render 5 individual header lines; \"` — assertion message, not a forward-facing struct-field claim |
| `reporter_terminal_tests.rs` | 3254 | Assertion message string literal: `"BC-2.11.028 pc2: collapse_findings=false must produce no (xN) suffix; got:\n{out}"` — assertion message |
| `reporter_terminal_tests.rs` | 3262 | Assertion message string literal: `"BC-2.11.028 pc2 contrast: collapse_findings=true must produce '(x5)' for same \"` — assertion message |
| `reporter_terminal_tests.rs` | 3345 | Inline comment `// Reporter with collapse_findings = true → produces "(x3)".` — this comment accompanies the construction site (Task 7 scope); becomes `FlatCollapsed` after Task 7 rewrite |
| `reporter_terminal_tests.rs` | 3355 | Assertion message: `"BC-2.11.028 inv1: collapse_findings=true must produce '(x3)' suffix; got:\n{out_on}"` — string literal under test |
| `reporter_terminal_tests.rs` | 3358 | Inline comment `// Reporter with collapse_findings = false → no collapse, no suffix.` — Task 7 construction-site comment; becomes `FlatExpanded` after Task 7 rewrite |
| `reporter_terminal_tests.rs` | 3368 | Assertion message: `"BC-2.11.028 inv1: collapse_findings=false must produce no (xN) suffix; \"` — string literal under test |
| `reporter_terminal_tests.rs` | 3375 | Assertion message: `"BC-2.11.028 inv1: collapse_findings=true and collapse_findings=false must produce \"` — string literal under test |
| `reporter_terminal_tests.rs` | 3565 | Inline comment `// JsonReporter does not have a collapse_findings field` — refers to JsonReporter, not TerminalReporter; factually correct after refactor (JsonReporter never had this field) |
| `reporter_terminal_tests.rs` | 3689 | Assertion message: `" of collapse_findings=true; 50 identical findings; got:\n{out}"` — string literal under test |

**Falsifiability rule:** After Task 7b completes, `grep -n 'collapse_findings\|show_mitre_grouping' tests/` MUST return only lines whose content matches one of the exemption reasons above (historical narration, assertion message string literals, or construction-site comments that were rewritten by Task 7). Any line NOT in this allow-list that still matches is a Task 7b omission and must be fixed before PR merge. The reviewer MUST diff the grep output against this table at PR review time.

Passing check: `grep -n 'collapse_findings\|show_mitre_grouping' tests/` yields at most the
exempt lines listed above — never a current docstring that asserts the field exists on the struct.

- **Test:** manual grep census during Task 7b review against this explicit allow-list;
  auditable at PR review time by diffing grep output against the EXEMPT table.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` (new) | `src/reporter/terminal.rs` | Pure data |
| `TerminalReporter.render: FindingsRender` (new field, replaces 2 bools) | `src/reporter/terminal.rs` | Pure data |
| `TerminalReporter::render()` dispatch (if-chain → match) | `src/reporter/terminal.rs` | Pure core |
| `run_analyze` construction site wiring | `src/main.rs` | Effectful (glue) |
| `run_summary` construction site wiring (inert) | `src/main.rs` | Effectful (glue) |
| `plain_reporter`, `mitre_reporter`, `collapse_reporter`, et al. helpers | `tests/reporter_terminal_tests.rs` | Test infrastructure |
| All 26 remaining test construction sites | 4 test files (reporter_tests:17, reporter_terminal_tests:7, dnp3_f5_remediation:1, bc_2_09_100_multitag:1) | Test infrastructure |

Architecture section references:
- `architecture/module-decomposition.md` (SS-11 C-20, `src/reporter/terminal.rs`)
- `docs/adr/0003-reporting-pipeline-layering.md` — "Render-Mode Enum (Issue #62 — v0.9.0)" subsection + Binding Rule 5

## Subsystem Anchor Justification

SS-11 owns this story's scope because `FindingsRender` and `TerminalReporter` live in
`src/reporter/terminal.rs`, which is the canonical SS-11 module per ARCH-INDEX Subsystem
Registry. The two `src/main.rs` construction sites are thin SS-12 wiring glue — the same
pattern established by `--no-collapse` / `--mitre` in STORY-118 / STORY-078 (BC-2.11.028
v1.5). This is insufficient scope to split into a separate SS-12 story.

## Dependency Anchor Justification

- `depends_on: []` — STORY-120 refactors the `TerminalReporter` struct that already exists
  in the shipped codebase (landed in STORY-077/078, extended in STORY-118). STORY-118 is
  completed. No new build-order predecessor is required — the refactor replaces existing
  fields, it does not introduce a new module or type that must be defined first.
- `blocks: [STORY-119]` — STORY-119 (grouped-mode collapse, deferred) will build on the
  `FindingsRender` enum introduced by STORY-120. If STORY-119 were dispatched against the
  old bool fields, its construction sites would need a second sweep when STORY-120 landed.
  The blocks relationship preserves logical sequencing. STORY-119 is deferred/unscheduled;
  this edge does not create a scheduling constraint.

## VP Anchor Justification

VP-012 (`escape_for_terminal` correctness, proptest, P1) is anchored to STORY-077 where
the test vehicle was built. STORY-120 does not move the anchor — it carries VP-012 as a
carried reference confirming the invariant is not weakened by the refactor. No new VP is
required (per F1 §8 and F2 Verification Delta).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Both `--mitre` and `--no-collapse` passed simultaneously | `FindingsRender::Grouped` selected (mitre wins; --no-collapse has no effect); behavior matches pre-refactor silent dispatch-order rule — now structural |
| EC-002 | `mitre_collapse_reporter` helper (previously `show_mitre_grouping: true, collapse_findings: true`) | Translates to `render: FindingsRender::Grouped`; tests asserting no `(xN)` suffix in grouped mode pass unchanged |
| EC-003 | `run_summary` construction site: `render` field is inert | `render: FindingsRender::FlatCollapsed` is structurally required but never consulted by `run_summary`; value chosen to express v0.8.0 default intent |
| EC-004 | Missing `use wirerust::reporter::terminal::FindingsRender;` in a test file | Compile error (`cannot find type FindingsRender`); all 5 test files must add the import |
| EC-005 | `cargo-semver-checks` fires `struct_field_missing` on 0.8.x baseline | Expected; v0.9.0 bump is the correct response; not a defect |
| EC-006 | STORY-119 builds on this enum later | `FindingsRender::Grouped` variant will gain a `collapse_within_groups: bool` companion or a new variant per STORY-119's F1/F2; no pre-splitting needed now (YAGNI per ADR-0003) |

## Tasks

1. **[F4 scope — RED]** Read `src/reporter/terminal.rs` (full file) to confirm current
   struct shape (four bool fields: `use_color`, `show_mitre_grouping`, `show_hosts_breakdown`,
   `collapse_findings`) and the existing FINDINGS dispatch block (`if self.show_mitre_grouping
   { ... } else if self.collapse_findings { ... } else { ... }`). Read `src/main.rs`
   (lines ~370-450) to confirm both construction sites. Read all 5 test files to enumerate
   all construction sites before writing stubs. **Also note**: as part of this initial read,
   run `grep -n 'collapse_findings\|show_mitre_grouping' tests/` to capture all comment and
   docstring references to the removed fields (AC-017 scope); record the line numbers for
   Task 7b.

2. **[F4 scope — RED]** Write test stubs in `tests/reporter_terminal_tests.rs` in a new
   `mod story_120` block for ACs 001–005, 007–016 that do not already have a corresponding
   passing test. Use `todo!()` bodies to satisfy the Red Gate density check (≥50% todo!()
   bodies required before implementer dispatch). ACs 008, 010, 011, 012, 013, 014 are
   satisfied by existing tests that must be updated (not new stubs), so focus new stubs on
   ACs 001, 002, 003, 004, 005.

3. **[F4 scope — GREEN — terminal.rs]** Define `pub enum FindingsRender` above
   `pub struct TerminalReporter` in `src/reporter/terminal.rs`. Add the `#[derive(Debug,
   Clone, Copy, PartialEq, Eq)]` attribute. Add the three variants with doc comments per
   AC-001. Remove `show_mitre_grouping: bool` and `collapse_findings: bool` from the struct.
   Add `pub render: FindingsRender`.

4. **[F4 scope — GREEN — terminal.rs dispatch]** Replace the `if self.show_mitre_grouping {
   ... } else if self.collapse_findings { ... } else { ... }` dispatch chain in
   `TerminalReporter::render()` with `match self.render { FindingsRender::Grouped => ...,
   FindingsRender::FlatCollapsed => ..., FindingsRender::FlatExpanded => ... }`. The bodies
   of the three arms are identical to the three branches they replace — no logic changes.

5. **[F4 scope — GREEN — src/main.rs run_analyze]** Replace the two-bool construction at
   ~line 373 with the three-way enum expression per AC-005, using the in-scope bool params
   `show_mitre_grouping` (line 107) and `collapse_findings` (line 108) — NOT the raw CLI
   references `*mitre` / `no_collapse`, which are only in scope in `main()`:
   `render: if show_mitre_grouping { FindingsRender::Grouped } else if collapse_findings { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }`.
   The `--mitre`/`--no-collapse` → bool resolution at the `main()` call site (lines 79-80)
   is UNCHANGED. Add `use wirerust::reporter::terminal::FindingsRender;` to main.rs imports
   if needed. `run_analyze` signature is UNCHANGED.

6. **[F4 scope — GREEN — src/main.rs run_summary]** Replace the inert `collapse_findings:
   true` construction at ~line 439 with `render: FindingsRender::FlatCollapsed` per AC-006.
   Update the adjacent comment to reference the enum variant.

7. **[F4 scope — GREEN — test files]** Update all 26 test-file construction sites across
   four test files using the F1 §6 migration table (counts verified by grep census):
   - `tests/reporter_tests.rs` (17 sites): 6 → `Grouped` (lines 1001,1036,1071,1106,1155,1192); 11 → `FlatExpanded`
   - `tests/reporter_terminal_tests.rs` (7 sites): plain_reporter(71)→`FlatExpanded`; mitre_reporter(662)→`Grouped`; collapse_reporter(1789)→`FlatCollapsed`; collapse_reporter_color(1799)→`FlatCollapsed`; mitre_collapse_reporter(1809)→`Grouped`; reporter_on(3346)→`FlatCollapsed`; reporter_off(3359)→`FlatExpanded`
   - `tests/dnp3_f5_remediation_tests.rs` (1 site): mitre_reporter(1070) → `Grouped`
   - `tests/bc_2_09_100_multitag_tests.rs` (1 site): make_terminal(690) → parameterized `if mitre_grouping { FindingsRender::Grouped } else { FindingsRender::FlatExpanded }`
   Add `use wirerust::reporter::terminal::FindingsRender;` to each file's imports.

7b. **[F4 scope — GREEN — comment sweep, parallel with Task 7]** Sweep all 4 test files for
    comment and docstring references to the removed `collapse_findings` and `show_mitre_grouping`
    fields. Run `grep -n 'collapse_findings\|show_mitre_grouping' tests/` from the repo root and
    rewrite every forward-facing match to enum vocabulary (`FindingsRender::Grouped`,
    `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded`).

    **Use the AC-017 Forward-Facing Sweep Targets table** (24 file:line entries) as the
    authoritative work list. Every entry in that table MUST be rewritten before marking this
    AC complete. Specifically:

    - Rewrite the `test_BC_2_11_028_flag_wired_to_reporter_field` docstring
      (`tests/reporter_terminal_tests.rs:3320-3324`): the current text says "FAILS if the
      `collapse_findings` field does not exist" — replace with "FAILS if `FlatCollapsed` and
      `FlatExpanded` produce identical output / if the render variant is mis-wired to the
      wrong rendering path."
    - Rewrite all 23 additional forward-facing matches per the sweep-targets table (lines
      ~696, 1011, 1086, 1100, 1107, 2068, 2078, 2390, 2400, 3218, 3221, 3222, 3226, 3240,
      3257, 3270, 3328-3329, 3543, 3550-3551, 3659, 3669, 3774 in reporter_terminal_tests.rs;
      line ~766 in bc_2_09_100_multitag_tests.rs).
    - Do NOT rewrite the EXEMPT allow-list entries (historical STORY-118 narration at lines
      ~68, ~659; assertion message string-literals-under-test at lines ~2099, 3247, 3254,
      3262, 3355, 3368, 3375, 3689; JsonReporter factual comment at ~3565).

    **Verification:** After all rewrites, run `grep -n 'collapse_findings\|show_mitre_grouping'
    tests/` and diff the output against the AC-017 EXEMPT allow-list. Any match NOT in the
    allow-list is an omission that must be fixed. Guards against DF-GREEN-DOC-TENSE-SWEEP /
    DF-SIBLING-SWEEP recurrence. See AC-017.

8. **[F4 scope — VERIFY]** Run `cargo check --all-targets` (zero compile errors confirms
   all 28 sites updated). Run `cargo test --all-targets` (zero test regressions confirms
   byte-identical output). Run `cargo clippy --all-targets -- -D warnings` (zero warnings).
   Run `cargo fmt --check` (clean). Bump `Cargo.toml` version from `0.8.x` to `0.9.0`.

9. **(Post-delivery)** Compute and update `input-hash:` via
   `bin/compute-input-hash --write .factory/stories/STORY-120.md`. Verify MATCH.

## Previous Story Intelligence

This story is the direct successor to STORY-118 (E-18, issue #259, v0.8.0) and closes the
technical debt that STORY-118 created when it added `collapse_findings: bool` as a fourth
bool to `TerminalReporter` — triggering issue #62's deferred-until-triggered condition.

**STORY-118 lessons directly applicable here:**
- The `TerminalReporter` struct is in `src/reporter/terminal.rs`. Its current shape (post
  STORY-118) has four bool fields: `use_color`, `show_mitre_grouping`, `show_hosts_breakdown`,
  `collapse_findings`. STORY-120 removes the last two and adds `render: FindingsRender`.
- The `escape_for_terminal` function is private to `terminal.rs` and is NOT changed.
- `render_finding_flat`, `render_findings_grouped`, `render_findings_collapsed`,
  `collapse_findings_pass`, and all private helpers are NOT changed — only the dispatch
  entry point changes.
- `mitre_collapse_reporter` in `reporter_terminal_tests.rs` currently sets
  `show_mitre_grouping: true, collapse_findings: true`. This is the only site that sets
  a previously-nonsensical combination. Its tests assert that grouped mode does NOT apply
  collapse — which is exactly what `FindingsRender::Grouped` guarantees structurally. No
  test assertion text changes; only the construction site changes.
- The four affected test files are the complete and confirmed set from F1 §1 (Impact
  Boundary table). No other test files contain `TerminalReporter { ... }` literals.
  (`reporter_terminal_tests.rs`, `reporter_tests.rs`, `dnp3_f5_remediation_tests.rs`,
  `bc_2_09_100_multitag_tests.rs`)

**Key lesson from STORY-118 adversarial convergence:** Read ALL four test files before
writing stubs to get exact line numbers and helper names. The F1 §6 migration table is
authoritative but line numbers may shift slightly. `cargo check` after the struct change
is the definitive completeness check — every missed site is a compile error.

**Scope boundary for implementer:** `collapse_findings_from_flag` (src/main.rs:498) and
its unit tests (~lines 535-541) are UNCHANGED — this helper maps `--no-collapse` (a
`bool` CLI flag) to the `collapse_findings: bool` parameter. The `run_analyze` signature
is UNCHANGED. Only the bool → `FindingsRender` enum translation at the construction site
inside `run_analyze` (~line 373) is new in this story. Do not touch `main()` flag parsing
or `collapse_findings_from_flag`.

## Architecture Compliance Rules

Derived from ADR-0003 Binding Rule 5 and the F1/F2 design decisions:

1. **FindingsRender is the sole render-mode discriminant** — no bool field on
   `TerminalReporter` may encode a mutually-exclusive rendering mode after STORY-120.
   `use_color` and `show_hosts_breakdown` remain as bools because they are orthogonal
   (all combinations valid). Any future rendering mode addition requires a new enum variant.
2. **No Default derive on FindingsRender** — explicit construction at every site is
   required. `Default::default()` is not available and must not be added without a
   documented, deliberate API commitment.
3. **Dispatch is match, not if-chain** — the `match self.render` in `render()` is
   exhaustive by the Rust compiler. Adding a new variant without a new arm is a compile
   error. The if-chain pattern is prohibited after this story.
4. **Zero output-byte changes** — the refactor is behavior-preserving by construction.
   All rendering helpers (`render_finding_flat`, `render_findings_grouped`,
   `render_findings_collapsed`, `collapse_findings_pass`, `escape_for_terminal`) are
   called identically; only the dispatch path changes.
5. **Reporter::render trait is immutable** — `src/reporter/mod.rs` is not changed.
   The trait signature `render(summary, findings, analyzer_summaries)` is unchanged.
6. **FindingsRender is defined adjacent to TerminalReporter** — it lives in
   `src/reporter/terminal.rs`, not in `src/reporter/mod.rs` or `src/lib.rs`. No other
   reporter uses it; locality principle applies.

## Forbidden Dependencies

- `src/reporter/json.rs` and `src/reporter/csv.rs` MUST NOT be modified by STORY-120.
  Any PR touching these files for this issue is a scope violation.
- `src/reporter/mod.rs` MUST NOT be modified (Reporter trait is unchanged).
- `src/cli.rs` MUST NOT be modified (flags `--mitre`, `--no-collapse`, `--hosts` are
  unchanged; the flag names and types are not affected by the struct refactor).
- `src/findings.rs`, `src/analyzer/` MUST NOT be modified (Finding struct and all
  analyzers are unaffected).
- `FindingsRender` MUST NOT gain a `Default` implementation without a separate deliberate
  API decision.
- `collapse_findings_from_flag` (src/main.rs:498) and its unit tests (~lines 535-541) are
  UNCHANGED — the `--no-collapse` → `collapse_findings: bool` mapping is preserved at the
  `main()` call site. Only the bool → enum translation at the `run_analyze` construction
  site is new (this story). The `run_analyze` function signature is UNCHANGED.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `owo-colors` | existing | Used by rendering helpers — unchanged; no new color calls |
| No new dependencies | — | Enum + struct change uses only stdlib; zero new crate dependencies |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reporter/terminal.rs` | **Modify** | Add `pub enum FindingsRender`; remove `show_mitre_grouping: bool` and `collapse_findings: bool` from `TerminalReporter`; add `pub render: FindingsRender`; replace FINDINGS dispatch if-chain with `match self.render` |
| `src/main.rs` | **Modify** | Update two `TerminalReporter { ... }` construction sites (run_analyze ~line 373, run_summary ~line 439); add `FindingsRender` import if not already present |
| `tests/reporter_terminal_tests.rs` | **Modify** | Update 7 construction sites (5 helpers: lines 71,662,1789,1799,1809 + 2 inline: lines 3346,3359); add `use wirerust::reporter::terminal::FindingsRender;`; add `mod story_120` test block |
| `tests/reporter_tests.rs` | **Modify** | Update 17 construction sites (6 → Grouped, 11 → FlatExpanded); add `FindingsRender` import |
| `tests/dnp3_f5_remediation_tests.rs` | **Modify** | Update 1 construction site (mitre_reporter helper, line 1070 → Grouped); add `FindingsRender` import |
| `tests/bc_2_09_100_multitag_tests.rs` | **Modify** | Update 1 construction site (make_terminal helper, line 690 → parameterized); add `FindingsRender` import |
| `Cargo.toml` | **Modify** | Bump `version = "0.8.x"` → `version = "0.9.0"` |
| `src/reporter/json.rs` | **No change** | Not affected |
| `src/reporter/csv.rs` | **No change** | Not affected |
| `src/reporter/mod.rs` | **No change** | Reporter trait unchanged |
| `src/cli.rs` | **No change** | Flags unchanged |
| `src/findings.rs` | **No change** | Finding struct unchanged |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~6,000 |
| BC files (12 BCs — all re-anchored to enum vocabulary) | ~30,000 |
| `src/reporter/terminal.rs` (full file, ~340 lines post-STORY-118) | ~7,000 |
| `src/main.rs` (run_analyze + run_summary sections, ~80 lines) | ~2,000 |
| `tests/reporter_terminal_tests.rs` (7 construction sites — 5 helpers + 2 inline, existing + mod story_120) | ~10,000 |
| `tests/reporter_tests.rs` (17 construction sites, ~50 lines relevant context) | ~2,000 |
| `tests/dnp3_f5_remediation_tests.rs` (1 construction site — mitre_reporter helper line 1070, ~10 lines) | ~500 |
| `tests/bc_2_09_100_multitag_tests.rs` (1 construction site — make_terminal helper line 690, ~10 lines) | ~500 |
| F1 delta analysis (migration table section, ~3 pages) | ~3,000 |
| ADR-0003 Render-Mode Enum subsection | ~2,000 |
| Tool outputs (cargo check, test, clippy) | ~1,500 |
| **Total estimated** | **~64,500** |

Within 20–30% of agent context window (200k context → 40–60k). This story is at the
boundary. If the implementer chooses to read all 12 BC files in full detail, the budget
may reach 80k — still under 40% of a 200k window. Recommend reading BC files selectively:
only the Precondition sections are relevant for construction sites; postconditions/invariants
were already verified during STORY-118 and are unchanged by this refactor.

## BC Frontmatter ↔ Body Cross-Check

Every BC in the `behavioral_contracts:` frontmatter array is cited by at least one AC:

- BC-2.11.013: AC-004 (invariant 4 — grouped path structurally suffix-free), AC-012
- BC-2.11.014: AC-003 (postcondition 1 — Grouped arm calls `render_findings_grouped` which owns within-tactic bucket sort by verdict, confidence, emission order)
- BC-2.11.015: AC-003 (postcondition 1 — Grouped arm calls `render_findings_grouped` which owns no-technique/unknown-ID findings landing in the Uncategorized bucket)
- BC-2.11.016: AC-003 (postcondition 1 — Grouped arm calls `render_findings_grouped` which owns grouped-mode em-dash + technique-name MITRE line expansion)
- BC-2.11.017: AC-014 (postcondition 1 — default/flat rendering emits `MITRE: <id(s)>` only, no em-dash; FlatExpanded and FlatCollapsed arms call render_finding_flat)
- BC-2.11.019: AC-003 (invariant 7 — dispatch routing), AC-010
- BC-2.11.025: AC-004 (invariant 5 — type-level mutual exclusion), AC-013
- BC-2.11.026: AC-013 (postcondition 1 — (xN) suffix preserved in FlatCollapsed)
- BC-2.11.027: AC-013 (postconditions 2, 5, 6 — evidence sampling preserved)
- BC-2.11.028: AC-005, AC-006, AC-007 (flag wiring, inert site, construction sites)
- BC-2.11.029: AC-008 (all reporters receive same findings slice — unchanged by refactor)
- BC-2.11.010: AC-015 (escape invariant unchanged across all render paths)

Every AC cites a BC trace clause. All 12 BCs appear in both frontmatter and body.
