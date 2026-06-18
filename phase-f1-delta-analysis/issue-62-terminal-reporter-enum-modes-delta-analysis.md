---
document_type: f1-delta-analysis
feature_issue: "#62"
feature_title: "Refactor TerminalReporter to enum-of-modes when 3rd render flag is added"
feature_mode_cycle: "v0.8.0-completed → v0.9.0"
date: 2026-06-17
producer: architect
validation_report: n/a — issue #62 is a refactor with no external validation required; the
  trigger condition is a structural code observation, not an external technology claim
validation_verdict: TRIGGER-CONFIRMED
status: draft
supersedes_section: ".factory/phase-f1-delta-analysis/issue-259-finding-collapse-delta-analysis.md §4 Interaction with Issue #62"
modified:
  - "2026-06-18: Census corrected 35→28 construction sites (F3 round-7 bookkeeping fix). The 35 was a pre-grep estimate; ground-truth grep yields 28. reporter_terminal_tests.rs: 12→7 (helpers are 5 construction sites, not 12; counted each helper struct literal once). dnp3_f5_remediation_tests.rs: 2→1 (helper fn line 1069 is the fn signature, literal is at line 1070). bc_2_09_100_multitag_tests.rs: 2→1 (same pattern: fn signature at line 689, literal at line 690). Total: 2+17+7+1+1 = 28. Nine locations updated in total: census table (3 cells), total line, §6 intro, §7 paragraph, summary table, §8 classification test-count, §9 OQ-5 option-B, §9 OQ-6 question, §10 summary top-risks."
---

# F1 Delta Analysis — Issue #62: Refactor TerminalReporter to Enum-of-Modes

## Context: Trigger Already Fired

Issue #62 was filed as a deferred-until-triggered refactor. The trigger condition stated:
"when a 3rd render flag is being added." As of v0.8.0 (STORY-118, issue #259), that
condition has been MET AND EXCEEDED — `TerminalReporter` now carries FOUR boolean fields:

```rust
pub struct TerminalReporter {
    pub use_color: bool,
    pub show_mitre_grouping: bool,
    pub show_hosts_breakdown: bool,
    pub collapse_findings: bool,
}
```

The trigger fired silently when STORY-118 added `collapse_findings` without performing this
refactor. The F1 delta analysis for issue #259 acknowledged this at
`.factory/phase-f1-delta-analysis/issue-259-finding-collapse-delta-analysis.md §4 "Interaction
with Issue #62"` and deferred the decision to the human gate. This document is the deferred
F1 for issue #62 itself.

Two concrete violations of AC #3 ("no inter-flag invariants encoded only in comments") now
exist in the shipped code:

1. **Three-way mutual exclusion as nested bools** (`src/reporter/terminal.rs:187-205`):
   ```rust
   if self.show_mitre_grouping {
       self.render_findings_grouped(&mut out, findings);
   } else if self.collapse_findings {
       self.render_findings_collapsed(&mut out, findings);
   } else {
       for f in findings { self.render_finding_flat(&mut out, f); }
   }
   ```
   The current type permits the nonsensical combination
   `show_mitre_grouping = true && collapse_findings = true`. The code silently ignores
   `collapse_findings` when `show_mitre_grouping` is true (BC-2.11.025 invariant 5 scopes
   collapse to flat mode). This invariant is enforced only in documentation and by the
   dispatch order — not by the type system.

2. **Comment-encoded inert value** (`src/main.rs:443-447`):
   ```rust
   // BC-2.11.028 invariant 4: `run_summary` emits no FINDINGS section;
   // this value is inert. Set to `true` for completeness...
   collapse_findings: true,
   ```
   The `run_summary` path sets `collapse_findings: true` with a comment explaining the value
   does not matter. The type system cannot express "this field is irrelevant in this context."

---

## 1. Impact Boundary

### Files Touched (IN SCOPE)

| File | Change Type | Exact Scope |
|------|-------------|-------------|
| `src/reporter/terminal.rs` | Refactor | Replace the four bool fields with `use_color: bool` + `show_hosts_breakdown: bool` + `render: FindingsRender`. Add `pub enum FindingsRender` with three variants. Update `render()` dispatch (lines 187-205). Update `render_findings_collapsed`, `render_findings_grouped`, `render_finding_flat` — called identically, just reached via `self.render` match instead of `self.show_mitre_grouping` / `self.collapse_findings` if-chain. Update `COLLAPSE_EVIDENCE_SAMPLES` constant — unchanged. No behavioral change. |
| `src/main.rs` | Refactor | Two construction sites. `run_analyze` (~line 373): `show_mitre_grouping` + `collapse_findings` → `render: if show_mitre_grouping { FindingsRender::Grouped } else if collapse_findings { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }`. `run_summary` (~line 439): `render: FindingsRender::FlatCollapsed` (or any variant — field is inert; use enum variant to express intent). The comment explaining the inert value stays but is now attached to the enum variant selection. |
| `tests/reporter_tests.rs` | Refactor | 17 construction sites (lines 449, 473, 524, 543, 565, 645, 753, 862, 949, 1001, 1036, 1071, 1106, 1128, 1155, 1192, 2476). Each inline `TerminalReporter { ... }` literal gets the two rendered bools replaced by a `render: FindingsRender::...` field. Mechanical field-substitution only — zero test logic changes. |
| `tests/reporter_terminal_tests.rs` | Refactor | 7 construction sites (helpers: `plain_reporter` at line 71, `mitre_reporter` at line 662, `collapse_reporter` at line 1789, `collapse_reporter_color` at line 1799, `mitre_collapse_reporter` at line 1809; inline: `reporter_on` at 3346, `reporter_off` at 3359). The five helpers each become one-liner enum assignments. The `mitre_collapse_reporter` helper currently sets `show_mitre_grouping: true, collapse_findings: true` — the impossible-but-ignored combination; with the enum, this becomes `render: FindingsRender::Grouped` (grouped wins, collapse silently ignored in the current code — the new design makes this unrepresentable). |
| `tests/dnp3_f5_remediation_tests.rs` | Refactor | 1 construction site (helper `mitre_reporter`, literal @1070; line 1069 is the fn signature): `show_mitre_grouping: true, collapse_findings: false` → `render: FindingsRender::Grouped`. |
| `tests/bc_2_09_100_multitag_tests.rs` | Refactor | 1 construction site (helper `make_terminal`, literal @690; line 689 is the fn signature): `show_mitre_grouping: mitre_grouping, collapse_findings: false` → `render: if mitre_grouping { FindingsRender::Grouped } else { FindingsRender::FlatExpanded }`. |

**Total construction sites changed: 28** (2 in `src/main.rs`, 17 in `tests/reporter_tests.rs`,
7 in `tests/reporter_terminal_tests.rs`, 1 in `tests/dnp3_f5_remediation_tests.rs`,
1 in `tests/bc_2_09_100_multitag_tests.rs`).

### Files Explicitly NOT Touched

| File | Reason |
|------|--------|
| `src/reporter/terminal.rs` (private methods) | `render_finding_flat`, `render_finding_prefix`, `render_finding_grouped`, `render_findings_collapsed`, `render_findings_grouped`, `collapse_findings_pass`, `escape_for_terminal`, `section`, `COLLAPSE_EVIDENCE_SAMPLES`, `CollapseKey` — all private. None of these change signature or behavior. The refactor touches only the public struct shape and the dispatch in `render()`. |
| `src/reporter/mod.rs` | The `Reporter` trait (`render(summary, findings, analyzer_summaries)`) is unchanged. |
| `src/cli.rs` | Flags `--mitre`, `--no-collapse`, `--hosts` are unchanged. No new CLI flags. No removed flags. |
| `src/reporter/json.rs`, `src/reporter/csv.rs` | Machine-readable reporters — never referenced `TerminalReporter` fields. |
| `src/findings.rs`, all `src/analyzer/` files | Untouched. |
| All other test files | Any test file that does NOT contain a `TerminalReporter { ... }` literal is unaffected. Confirmed: the five files enumerated above are the complete set. |
| `docs/adr/0003-reporting-pipeline-layering.md` | No new ADR section required — see §3. |

### Confirmed Behavior-Preserving Boundary

This is a **pure type-system refactor**. Every byte of output produced by `TerminalReporter`
is governed by `render_finding_flat`, `render_findings_collapsed`, and `render_findings_grouped`
— none of which change. The only change is how the dispatch in `render()` decides which of
those three paths to call. The new dispatch is a `match self.render { ... }` over three
variants that maps 1:1 to the existing `if self.show_mitre_grouping { ... } else if
self.collapse_findings { ... } else { ... }` chain. No output bytes should change.

---

## 2. Affected Specs and BCs

### Existing BCs That Need Re-anchoring

The following BCs reference `show_mitre_grouping` or `collapse_findings` as preconditions,
postconditions, or anchors. After the refactor, the field names in the preconditions change
to `render: FindingsRender::Grouped` or `render: FindingsRender::FlatCollapsed` /
`FindingsRender::FlatExpanded`. The BCs' postconditions and invariants are unchanged — only
the field reference in the Preconditions section requires editing.

| BC ID | Title (abbreviated) | Re-anchoring Required |
|-------|--------------------|-----------------------|
| BC-2.11.019 | TerminalReporter Renders Sections in Correct Order | Precondition references `show_mitre_grouping`/`collapse_findings` — update to `render` variants. |
| BC-2.11.013 | MITRE Grouping Emits Tactic Headers | Precondition 1: `show_mitre_grouping = true` → `render = FindingsRender::Grouped`. |
| BC-2.11.014 | MITRE Bucket Sort | Precondition: `show_mitre_grouping = true` → `render = FindingsRender::Grouped`. |
| BC-2.11.025 | Flat-Mode Collapse Groups Findings | Precondition 1: `collapse_findings = true` → `render = FindingsRender::FlatCollapsed`. Precondition 2: `show_mitre_grouping = false` — becomes a type-system invariant, not a field check. Invariant 5: update scoping statement. Architecture Anchor: update struct field reference. |
| BC-2.11.026 | Count Display Format | Precondition references `collapse_findings` → `render = FindingsRender::FlatCollapsed`. |
| BC-2.11.027 | Evidence Sampling | Precondition references `collapse_findings` → `render = FindingsRender::FlatCollapsed`. |
| BC-2.11.028 | Opt-out Flag (--no-collapse) | Precondition 1: `collapse_findings = true` / Precondition 2: `collapse_findings = false` → enum variants. The wiring comment at `main.rs` (`collapse_findings: !args.no_collapse`) updates to `render: if args.no_collapse { FindingsRender::FlatExpanded } else { FindingsRender::FlatCollapsed }`. |
| BC-2.11.017 | render_finding_flat Multi-ID MITRE | Precondition: `show_mitre_grouping = false` — replace with `render != FindingsRender::Grouped` or narrow to the flat paths. |
| BC-2.11.018 | Colorization | No precondition on these fields; `use_color` stays an orthogonal field — no change needed. |

**Total: 9 BCs in SS-11 need field-name updates in their Preconditions sections.** None
require new postconditions, new invariants, or new test vectors — the observable behavior is
unchanged. This is a re-anchoring pass, not a spec expansion.

### New BCs Required

**None.** This is a pure refactor. The behavior contracts (grouping, collapse, flat expansion,
opt-out) are already fully specified in BC-2.11.013, .014, .017, .025–.029. The enum
is an encoding change that makes the existing contracts' preconditions tighter; it does not
introduce any new behavioral surface.

---

## 3. ADR Decision

### Recommendation: No New ADR; No ADR Amendment

The enum-of-modes refactor is an **internal type-design improvement** within
`src/reporter/terminal.rs`. It does not change the architecture's layering principle
(ADR-0003), the reporter trait interface (`mod.rs`), the data model (`findings.rs`), the CLI
surface, or any cross-subsystem contract.

ADR-0003's display-layer aggregation subsection (added for issue #259) already establishes
that collapse is terminal-reporter-internal. The enum makes that boundary stronger. No new
ADR is warranted — ADR-0002 governs modular analyzers, ADR-0001 governs stream dispatch,
ADR-0006/0007 govern DNP3. None of these are affected.

A brief note in the PR description documenting the enum shape and the rationale for each
variant is sufficient. If the project adopts ADRs for type-level design decisions at this
granularity, a short ADR could record the choice, but that is a process preference, not a
technical requirement. Recommendation: skip the ADR; capture the design rationale in the PR
description and in the `FindingsRender` doc comment.

### Enum Shape Recommendation

Based on the current dispatch and the existing BCs, the recommended enum is:

```rust
/// Governs which rendering path the FINDINGS section uses.
///
/// Replaces the `show_mitre_grouping: bool` + `collapse_findings: bool`
/// pair that existed in v0.8.0. Using an enum makes the three modes
/// mutually exclusive at the type level — the previous struct permitted
/// `show_mitre_grouping = true && collapse_findings = true`, a combination
/// that was silently handled by the dispatch order but was never valid.
///
/// BC-2.11.013 (Grouped), BC-2.11.025–028 (FlatCollapsed), default (FlatExpanded).
pub enum FindingsRender {
    /// Group findings by MITRE tactic (`--mitre` flag).
    /// Corresponds to the previous `show_mitre_grouping = true`.
    Grouped,
    /// Collapse repeated findings into counted groups (default, v0.8.0+).
    /// Corresponds to the previous `collapse_findings = true, show_mitre_grouping = false`.
    FlatCollapsed,
    /// One display line per raw finding (pre-v0.8.0 behavior, `--no-collapse`).
    /// Corresponds to the previous `collapse_findings = false, show_mitre_grouping = false`.
    FlatExpanded,
}

pub struct TerminalReporter {
    pub use_color: bool,
    pub show_hosts_breakdown: bool,
    pub render: FindingsRender,
}
```

### Does `show_hosts_breakdown` Fold into the Enum?

**No.** `show_hosts_breakdown` is orthogonal to findings rendering. It gates the HOSTS
section, which is rendered at `src/reporter/terminal.rs:149-158`, before the FINDINGS
section. It is controlled by the `--hosts` flag on the `summary` subcommand, not the
`analyze` subcommand. Folding it into `FindingsRender` would be incorrect because:

1. The HOSTS section and the FINDINGS section are independent output sections.
2. The `summary` subcommand never renders a FINDINGS section; its `FindingsRender` variant
   is irrelevant. The `show_hosts_breakdown` field remains directly applicable to `summary`.
3. A `FindingsRender::GroupedWithHostsBreakdown` variant would be a category error.

`show_hosts_breakdown: bool` stays as a separate field on `TerminalReporter`.

### Does `use_color` Fold into the Enum?

**No.** `use_color` is orthogonal to all three finding-render modes. Color applies uniformly
across every output section (headers, findings, skipped-packet warnings). It is governed by
the `--no-color` flag and terminal detection, independent of how findings are grouped. It
stays as a separate field.

**Final struct shape: 2 bool fields + 1 enum field.** This is the right primitive for the
current state of the codebase.

---

## 4. CLI Surface Delta

**No CLI surface changes.** The three existing flags remain exactly as-is:

| Flag | CLI Location | Current Wiring | Post-Refactor Wiring |
|------|-------------|-----------------|----------------------|
| `--mitre` | `Commands::Analyze` | `show_mitre_grouping: *mitre` | `render: if *mitre { FindingsRender::Grouped } else if collapse_findings { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }` |
| `--no-collapse` | `Commands::Analyze` | `collapse_findings: !no_collapse` | (absorbed into the `render` match above) |
| `--hosts` | `Commands::Summary` | `show_hosts_breakdown: *hosts` | `show_hosts_breakdown: *hosts` (unchanged) |

`--mitre` and `--no-collapse` interaction is now typed. If a user passes both `--mitre` and
`--no-collapse`, the resolution is deterministic: `--mitre` wins (Grouped variant), and
`--no-collapse` has no effect. This was already the behavior (the `if self.show_mitre_grouping`
check came first); the enum makes it structural.

No new flags. No removed flags. No help-text changes required (the enum is an internal type).

---

## 5. Regression Risk

### Risk Level: LOW

The refactor is behavior-preserving by construction. No rendering logic changes. The risk
surface is:

**Risk 1 (MEDIUM): Compiler does not catch every construction site.** If any construction
site is missed, the code will not compile — Rust struct literals require all fields to be
named, and removing `show_mitre_grouping` + `collapse_findings` is a compile error. This is
the strong form of the safety net: the compiler is the exhaustiveness checker. Risk is low
because a successful `cargo check` confirms completeness.

**Risk 2 (LOW): `mitre_collapse_reporter` helper semantics.** In `reporter_terminal_tests.rs`
at line 1808, `mitre_collapse_reporter` sets `show_mitre_grouping: true, collapse_findings: true`.
This combination is currently valid Rust but represents the nonsensical state — grouped mode
silently suppresses collapse. After the refactor, this helper will set
`render: FindingsRender::Grouped` (grouped wins). The tests that use this helper
(`test_mitre_collapse_grouped_mode_suppresses_collapse` and related) already assert that
collapse does NOT apply when `show_mitre_grouping = true` — so the enum simply makes the
test setup structurally consistent with the behavior it is testing. No test assertion logic
changes.

**Risk 3 (LOW): The `run_summary` inert-value site.** `main.rs:447` currently sets
`collapse_findings: true` with a comment calling it inert. Post-refactor, this becomes
`render: FindingsRender::FlatCollapsed` (or any variant — the FINDINGS section is never
rendered by `run_summary`). Either variant is correct. Using `FlatCollapsed` as the default
is semantically consistent ("if this reporter were ever used to render findings, it would use
the v0.8.0 default").

**Risk 4 (LOW): Public API break on a 0.x crate.** `TerminalReporter` fields are `pub` and
the crate is at v0.8.0 (0.x). Removing `show_mitre_grouping` and `collapse_findings` and
adding `render: FindingsRender` is a breaking change to the public struct API under semver.
However, at 0.x the major version is 0, so breaking changes are permitted by semver without
a major bump (minor version bump suffices). Additionally, `TerminalReporter` is an internal
type — it does not appear in `src/lib.rs` exports (the crate is a binary, not a library).
Risk is effectively zero for downstream consumers but should be documented in `CHANGELOG.md`
for clarity.

**Risk 5 (LOW): Issue #63 interaction (snapshot tests).** Issue #63 is open — no snapshot/
golden tests exist today. The refactor produces zero output changes, so if #63 were
completed before or after this refactor, the snapshots would be identical. There is no
ordering risk between #62 and #63.

### Does Issue #63 Need to Land First?

**No.** The refactor changes no output bytes. If #63 is not yet done, the refactor is safe
to land without it. If #63 is in flight simultaneously, the snapshot baselines captured
before the refactor will match exactly after it. The two issues are independent.

---

## 6. Affected Tests

### Construction Site Translation Table

All 28 construction sites require a mechanical field-substitution. No test assertion logic
changes. The substitution rule is:

| Old Fields | New Field |
|-----------|----------|
| `show_mitre_grouping: true, collapse_findings: false` | `render: FindingsRender::Grouped` |
| `show_mitre_grouping: true, collapse_findings: true` | `render: FindingsRender::Grouped` (was already silently Grouped per dispatch order) |
| `show_mitre_grouping: false, collapse_findings: true` | `render: FindingsRender::FlatCollapsed` |
| `show_mitre_grouping: false, collapse_findings: false` | `render: FindingsRender::FlatExpanded` |

**By file:**

| File | Sites | Current Config(s) | New Config(s) |
|------|-------|-------------------|---------------|
| `src/main.rs:373` | 1 | `show_mitre_grouping, collapse_findings` (runtime flags) | `render: if show_mitre_grouping { Grouped } else if collapse_findings { FlatCollapsed } else { FlatExpanded }` |
| `src/main.rs:439` | 1 | `show_mitre_grouping: false, collapse_findings: true` (inert) | `render: FindingsRender::FlatCollapsed` |
| `tests/reporter_terminal_tests.rs:71` (`plain_reporter`) | 1 | `false, false` | `FlatExpanded` |
| `tests/reporter_terminal_tests.rs:662` (`mitre_reporter`) | 1 | `true, false` | `Grouped` |
| `tests/reporter_terminal_tests.rs:1789` (`collapse_reporter`) | 1 | `false, true` | `FlatCollapsed` |
| `tests/reporter_terminal_tests.rs:1799` (`collapse_reporter_color`) | 1 | `false, true` | `FlatCollapsed` |
| `tests/reporter_terminal_tests.rs:1809` (`mitre_collapse_reporter`) | 1 | `true, true` (currently nonsensical) | `Grouped` |
| `tests/reporter_terminal_tests.rs:3346` (`reporter_on`) | 1 | `false, true` | `FlatCollapsed` |
| `tests/reporter_terminal_tests.rs:3359` (`reporter_off`) | 1 | `false, false` | `FlatExpanded` |
| `tests/reporter_tests.rs` (17 sites) | 17 | `false, false` (most); `true, false` (4 sites: lines 1001, 1036, 1071, 1106) | `FlatExpanded` / `Grouped` respectively |
| `tests/dnp3_f5_remediation_tests.rs:1070` | 1 | `true, false` | `Grouped` |
| `tests/bc_2_09_100_multitag_tests.rs:690` | 1 | `mitre_grouping, false` (parameterized) | `if mitre_grouping { Grouped } else { FlatExpanded }` |

### Tests That Assert the Now-Impossible Combination

Only one site sets `show_mitre_grouping: true, collapse_findings: true` —
`reporter_terminal_tests.rs:1809` (`mitre_collapse_reporter`). The tests using this helper
assert that grouped mode suppresses collapse. After the refactor, `FindingsRender::Grouped`
structurally cannot enable collapse, so these tests become simpler: they still verify that
grouped output does not contain `(xN)` suffixes, but the precondition is now a type guarantee
rather than a runtime dispatch order. No assertion text changes.

### Tests That Assert `collapse_findings = false` Explicitly as a Comment

17 sites in `reporter_tests.rs` and several helpers include the comment
`// STORY-118: new field; false = pre-v0.8.0 non-collapse path`. After the refactor, the
field is `render: FindingsRender::FlatExpanded`. The comments become slightly stale but are
harmless; update or remove them during the refactor pass.

---

## 7. Epic Placement

### Recommendation: Epic E-8 (Reporting and Output Formats)

Issue #62 is the structural companion to issue #259 (STORY-118). Both live in SS-11
(`reporter/terminal.rs`). Epic E-8 already owns SS-11 BCs and the `--mitre` grouped-mode
feature (STORY-078) and the finding-collapse feature (STORY-118).

This refactor is a debt-payment story within E-8 — it completes the type-system cleanup
that STORY-118 was supposed to trigger. It does not belong in a new epic.

### Estimated Story Count: 1 Story

| Story | Scope | Points (est.) |
|-------|-------|----------------|
| STORY-120 (next available) | `TerminalReporter` enum-of-modes refactor: (a) define `pub enum FindingsRender` with `Grouped / FlatCollapsed / FlatExpanded` variants; (b) update `TerminalReporter` struct (remove `show_mitre_grouping`, `collapse_findings`; add `render: FindingsRender`); (c) update `render()` dispatch to `match self.render`; (d) update `main.rs` two construction sites; (e) update all 28 test construction sites (mechanical substitution); (f) update BC-2.11.013, .014, .017, .019, .025–.028 precondition field references (re-anchoring pass); (g) add `FindingsRender` re-export or visibility as needed. Behavioral change: none. Output change: none. TDD mode: refactor (all tests must pass before and after — Red Gate is `cargo check` failing on the old fields after struct change). | 3 pts |

**Total: 1 story (~3 pts).** This is a contained mechanical refactor with a large-but-
mechanical touch surface. The 28 construction sites are individually trivial; the constraint
is thoroughness, not complexity.

---

## 8. Verification-Property Impact

### No New VP Required

The refactor changes no logic. All existing VPs remain valid:

- **VP-012** (`escape_for_terminal` correctness, proptest, P1): `escape_for_terminal` is a
  module-level private function in `terminal.rs`. It is not in the struct, not a method, not
  affected by the struct shape change. VP-012's four proptest properties are unchanged.
- No state machine, no arithmetic, no security boundary, no cross-process interaction is
  introduced.

### Classification: Test-Sufficient (Compile-Verified)

The refactor's correctness is primarily verified by the Rust compiler:

- Missing construction sites: compile error (struct literal missing fields).
- Behavioral regressions: the existing 28 construction-site tests cover every path; the test
  suite passing after the refactor proves behavioral equivalence.

No new proptest or Kani harness is warranted. A `cargo test --all-targets` green run after
the refactor is the sufficient gate.

---

## 9. Open Questions for the F1 Human Gate

These are the decisions the human must adjudicate before F2 (BC re-anchoring and story
authoring) can begin. They are enumerated in priority order.

### OQ-1 (BLOCKING): Enum Variant Names

**Question:** Are `FindingsRender`, `Grouped`, `FlatCollapsed`, `FlatExpanded` the
right names? Alternatives:

- `RenderMode` / `Grouped, Collapsed, Expanded` (shorter, but `RenderMode` is generic)
- `FindingsDisplay` / `ByTactic, CountedGroups, Individual` (more descriptive)
- `FindingsRender` / `Grouped, Collapsed, Flat` (shorter third variant name)

**Recommendation:** `FindingsRender { Grouped, FlatCollapsed, FlatExpanded }`.
- `FindingsRender` scopes the enum to the findings section specifically (not all of rendering).
- `Grouped` is the established term from BC-2.11.013 ("tactic grouping").
- `FlatCollapsed` and `FlatExpanded` are symmetric with `flat mode` as the established
  terminology in BC-2.11.025.
- `FlatExpanded` is preferred over `Flat` or `Individual` because "expanded" is the natural
  antonym of "collapsed" and mirrors the `--no-collapse` opt-out semantics.

**Human gate decision required before naming the enum in the BC re-anchoring pass.**

### OQ-2 (IMPORTANT): Module Location for `FindingsRender`

**Question:** Should `FindingsRender` be:

- (A) Defined in `src/reporter/terminal.rs` and kept `pub(crate)` or `pub`?
- (B) Defined in `src/reporter/mod.rs` as a shared type?

**Recommendation:** Option A — define in `terminal.rs`, mark `pub`. The enum is specific to
`TerminalReporter`'s rendering strategy. No other reporter uses it. Keeping it adjacent to
the struct that uses it follows the locality principle. If a future reporter wants a similar
enum, it defines its own.

**The current `use_color` and `show_hosts_breakdown` fields are already `pub` — there is no
reason to tighten visibility for the new field.**

### OQ-3 (IMPORTANT): Re-export in the Crate Root?

**Question:** Is `FindingsRender` part of the crate's public API surface? This is a binary
crate, so there is no `lib.rs` and no downstream crate consumers. However, test files import
`wirerust::reporter::terminal::TerminalReporter` and would need to also import
`wirerust::reporter::terminal::FindingsRender`.

**Recommendation:** No re-export needed. The test files can add
`use wirerust::reporter::terminal::FindingsRender;` to their existing imports. 26 of the
28 construction sites are in test files that already import `TerminalReporter` from the
same path; the remaining 2 are in `src/main.rs` (`run_analyze`, `run_summary`).

### OQ-4 (REFINEMENT): Derive Traits on `FindingsRender`

**Question:** Should `FindingsRender` derive `Debug`, `Clone`, `Copy`, `PartialEq`?

**Recommendation:** `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. This is the minimal
useful set:
- `Debug`: standard for any `pub` type.
- `Clone` + `Copy`: the enum has no heap data; Copy is free and eliminates `clone()` noise.
- `PartialEq` + `Eq`: allows test assertions like
  `assert_eq!(reporter.render, FindingsRender::FlatCollapsed)` if needed.

### OQ-5 (IMPORTANT): Interaction with STORY-119 (Grouped-Mode Collapse)

**Question:** STORY-119 (deferred — add collapse within `--mitre` grouped mode) was proposed
in the issue #259 F1 analysis. If STORY-119 is eventually implemented, the `Grouped` variant
would need to become two variants: `GroupedCollapsed` and `GroupedExpanded`. Should the
current refactor anticipate this by pre-splitting `Grouped`?

**Options:**
- (A) Pre-split now: `Grouped, GroupedCollapsed, FlatCollapsed, FlatExpanded` (4 variants).
  Anticipated future-proofing; adds variants for a feature not yet scheduled.
- (B) Add variants when STORY-119 is implemented: the STORY-119 F2 spec evolution amends
  the enum and adds the new variant. This is a second 28-site construction sweep.
- (C) Use a struct with `show_hosts_breakdown: bool, render: FindingsRender, collapse_within_groups: bool`:
  handles the interaction without multiplying variants; `collapse_within_groups` is only
  relevant when `render = FindingsRender::Grouped`.

**Recommendation:** Option B (defer). Pre-splitting for an unscheduled feature is YAGNI.
The STORY-119 cycle will have its own F1 and can amend the enum at that time.

### OQ-6 (PROCESS): Should #62 Be Bundled with STORY-119 or Kept Standalone?

**Question:** The 28 construction-site sweep will happen again when STORY-119 adds grouped-
mode collapse. Should the refactor be deferred until STORY-119 is scheduled, to avoid a
double sweep?

**Recommendation:** Deliver issue #62 as STORY-120 standalone. Rationale:
1. The issue #62 trigger has already fired. Every new construction site added between now
   and STORY-119 adds debt.
2. The enum refactor eliminates the impossible-state bug (the `show_mitre_grouping = true &&
   collapse_findings = true` nonsensical combination). This is a type-safety improvement
   independent of STORY-119.
3. STORY-119 is not on the current roadmap. Deferring #62 further compounds the debt.
4. The STORY-120 sweep is mechanical and low-risk. The eventual STORY-119 sweep will then
   be limited to adding one new `FindingsRender` variant and updating the sites that set
   `FindingsRender::Grouped` — a much smaller second pass.

### OQ-7 (REFINEMENT): Release Versioning — v0.8.x Patch or v0.9.0 Minor?

**Question:** The `TerminalReporter` struct shape change (removing public fields, adding a
new public enum field) is technically a breaking API change. At v0.8.0 (0.x), semver
permits this in a minor bump (v0.9.0) without a major version bump. However, since this is a
binary crate with no library consumers, the change is invisible to any downstream user. It
does affect any script that uses `wirerust` as a library (none known).

**Options:**
- (A) v0.9.0 — correct semver for a public-field removal.
- (B) v0.8.1 — justified on the grounds that the crate is a binary with no known library
  consumers and the user-visible behavior is unchanged.

**Recommendation:** v0.9.0, or bundle with the next scheduled minor release. The effort is
trivial either way; versioning consistency matters more than release acceleration.

---

## Summary

| Dimension | Assessment |
|-----------|-----------|
| Impact boundary | `src/reporter/terminal.rs` (struct shape only), `src/main.rs` (2 sites), `tests/reporter_tests.rs` (17 sites), `tests/reporter_terminal_tests.rs` (7 sites), `tests/dnp3_f5_remediation_tests.rs` (1 site), `tests/bc_2_09_100_multitag_tests.rs` (1 site). Total: 28 construction sites. Zero output changes. |
| BCs touched | 9 BCs in SS-11 (BC-2.11.013, .014, .017, .019, .025, .026, .027, .028) — re-anchoring of precondition field names only. No new BCs. No new postconditions. |
| ADR recommendation | No new ADR, no ADR amendment. Capture enum rationale in PR description. |
| Story estimate | 1 story (STORY-120, ~3 pts), Epic E-8, SS-11. |
| Regression risk | LOW. Rust compiler enforces construction-site completeness. No output bytes change. |
| Issue #63 interaction | Independent. Snapshot tests can land before or after #62 without conflict. |
| New VP | None. Test-sufficient + compile-verified. VP-012 unchanged. |
| Top risks | (1) `mitre_collapse_reporter` helper — currently nonsensical state, becomes structurally impossible; verify tests still pass; (2) 28-site sweep must be complete before merge (compiler enforces this); (3) `FindingsRender` must be `use`d in all test files. |
| F1-gate open questions | OQ-1 (enum variant names — BLOCKING); OQ-2 (module location — IMPORTANT); OQ-3 (re-export — REFINEMENT); OQ-4 (derive traits — REFINEMENT); OQ-5 (STORY-119 anticipation — IMPORTANT); OQ-6 (bundle vs standalone — PROCESS); OQ-7 (release versioning — REFINEMENT). |

---

## Related References

- Issue #62: https://github.com/Zious11/wirerust/issues/62
- Issue #63 (snapshot tests, independent): https://github.com/Zious11/wirerust/issues/63
- Issue #259 F1 delta (superseded §4): `.factory/phase-f1-delta-analysis/issue-259-finding-collapse-delta-analysis.md`
- Primary source: `src/reporter/terminal.rs:91-111` (struct shape), `src/reporter/terminal.rs:187-205` (FINDINGS dispatch)
- Main.rs construction sites: `src/main.rs:373-384` (`run_analyze`), `src/main.rs:439-450` (`run_summary`)
- Nonsensical-state site: `tests/reporter_terminal_tests.rs:1808-1815` (`mitre_collapse_reporter`)
- BCs requiring re-anchoring: `.factory/specs/behavioral-contracts/ss-11/BC-2.11.{013,014,017,019,025,026,027,028}.md`
- ADR governing the display layer: `docs/adr/0003-reporting-pipeline-layering.md`
- Next available story ID: STORY-120
- Epic: E-8, SS-11
