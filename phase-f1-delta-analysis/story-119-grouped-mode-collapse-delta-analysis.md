---
document_type: f1-delta-analysis
story_id: STORY-119
epic_id: E-18
feature_id: e18-finding-collapse
github_issue: 259
version: "1.0"
status: draft
producer: architect
timestamp: 2026-06-18T00:00:00Z
phase: f1
subsystems: [SS-11]
predecessor_story: STORY-120
predecessor_commit: f851995
unblocked_by: FindingsRender enum (STORY-120, Cargo 0.9.0)
---

# F1 Delta-Analysis: STORY-119 — Terminal Finding-Collapse, Grouped Mode / --mitre

**Status:** Draft — awaiting human gate on §3 (type-system design) and §4 (CLI/UX opt-in).

**Purpose:** This analysis defines the implementation delta for STORY-119 (grouped-mode
collapse, deferred from v0.8.0/v0.9.0). It does not author final BCs, write code, or
schedule the story. It presents options and recommendations on the two critical design
decisions, and enumerates all downstream changes required before implementation can begin.

---

## §1 — Goal and Functional Scope

STORY-119 extends the collapse feature (STORY-118, v0.8.0) into the `--mitre` grouped
rendering path. The target behavior mirrors flat-mode collapse (BC-2.11.025–029) but
operates per tactic bucket:

- **Within each tactic bucket** emitted by `render_findings_grouped`, findings sharing
  the same `(category, verdict, confidence, summary)` collapse key are grouped.
- **Count suffix:** A collapsed group of N≥2 within a bucket renders its header with
  ` (xN)` appended before colorization, per the same color-ladder rule as flat-mode
  (BC-2.11.026 PC-6 / terminal.rs:391).
- **Singleton findings** within a bucket render without suffix — byte-identical to the
  current grouped-mode output for that finding.
- **Evidence sampling:** K=3 per group, same positional rule as BC-2.11.027 (first
  `min(N, COLLAPSE_EVIDENCE_SAMPLES)` members, `evidence[0]` from each, window does NOT
  slide past empty-evidence members). K=3 is the existing named constant.
- **MITRE line:** The group representative (`group_members[0]`, per BC-2.11.025 Invariant 6
  / BC-2.11.026 PC-7) provides the MITRE line expansion — same em-dash-plus-name format
  used by `render_finding_grouped`.
- **Bucket ordering:** Tactic bucket headers and their order per `all_tactics_in_report_order()`
  are UNCHANGED. Collapse is a within-bucket transform only; the bucket-emission loop is
  unaffected.

The `escape_for_terminal` invariant (VP-012) is unchanged. Grouped-mode collapse reuses
the same escape call sites as flat-mode collapse.

---

## §2 — Impact Boundary: Exact Functions and Lines

The shipped codebase after STORY-120 (commit f851995, `src/reporter/terminal.rs`) is the
implementation baseline. The relevant function map is:

| Function | Lines (post-STORY-120) | Role in STORY-119 |
|----------|-----------------------|-------------------|
| `render_findings_grouped` | 432–483 | **Primary change target** — must apply per-bucket collapse before calling `render_finding_grouped` per item |
| `collapse_findings_pass` | 340–360 | **Reused as-is** — takes `&[Finding]`, returns `Vec<(CollapseKey, Vec<&Finding>)>`; will be called once per tactic bucket |
| `render_findings_collapsed` | 376–423 | **Not changed** — flat-mode collapse path; grouped mode gets a parallel implementation |
| `render_finding_grouped` | 311–327 | **Called for singletons** — unchanged; used for N=1 bucket members |
| `render_finding_prefix` | 267–291 | **Called for grouped headers** — unchanged; provides the escape + color logic that the collapse-aware grouped path mirrors |
| `escape_for_terminal` | 47–64 | **Unchanged** — called at render time on summary and evidence, same as flat mode |
| `COLLAPSE_EVIDENCE_SAMPLES` | 73 | **Unchanged** — K=3 constant reused |
| `CollapseKey` | 86–92 | **Unchanged** — same four-tuple key; used by `collapse_findings_pass` |

### What changes in `render_findings_grouped` (lines 432–483)

Currently the inner loop over each tactic's items calls `render_finding_grouped` directly:

```rust
// Current (line 472-474):
for (_, f) in items {
    self.render_finding_grouped(out, f);
}
```

STORY-119 inserts a per-bucket collapse pass before this inner loop. The bucket's items
`Vec<(usize, &Finding)>` (index + finding reference) must be:

1. Converted to a `&[Finding]` slice (or equivalent) for `collapse_findings_pass`.
2. Grouped via `collapse_findings_pass`.
3. Rendered: N=1 buckets call `render_finding_grouped` (byte-identical to current output
   for that singleton); N≥2 buckets call a new grouped-collapse header path that mirrors
   `render_findings_collapsed`'s N≥2 branch but uses `render_finding_grouped`'s MITRE
   line format (em-dash + name, not plain ID list).

The same change applies to the Uncategorized bucket (lines 477–482).

### Sorting interaction

The per-bucket sort (verdict-rank, confidence-rank, original index) happens BEFORE the
collapse pass. This is correct: sorting preserves the bucket's semantic order; collapse
then groups within that sorted order. The first occurrence of a key in the sorted order
becomes the group representative. This is consistent with flat-mode collapse (first
occurrence in input slice order) but applied post-sort within the bucket. This ordering
policy must be explicitly specified in the new BCs.

### Construction sites

No `TerminalReporter { ... }` construction sites change — STORY-119 changes only internal
rendering logic, not the struct shape. The `FindingsRender::Grouped` variant is already
wired at all 28 sites (STORY-120). No new struct fields are required under either
design option (see §3).

---

## §3 — DESIGN DECISION: Type-System Representation (CRITICAL — human gate required)

### Background

ADR-0003 Binding Rule 5 (v0.9.0) establishes: "Adding a new rendering mode requires
adding a new variant to `FindingsRender`." STORY-120's EC-006 explicitly noted that
STORY-119 would face this choice: either add a `GroupedCollapsed` variant or use a
companion field. ADR-0003 also pre-rejected `GroupedCollapsed` as YAGNI at the v0.9.0
boundary, leaving the decision to this F1/F2 pass.

The three current variants are:
- `FindingsRender::Grouped` — grouped (--mitre), no collapse (current behavior)
- `FindingsRender::FlatCollapsed` — flat + collapse (default)
- `FindingsRender::FlatExpanded` — flat + no collapse (--no-collapse)

STORY-119 adds a new rendering mode: grouped + collapse. There are three representation
options.

---

### Option A — New enum variant `GroupedCollapsed` (recommended)

```rust
pub enum FindingsRender {
    Grouped,           // --mitre, no collapse (current behavior preserved)
    GroupedCollapsed,  // --mitre + collapse (new — STORY-119)
    FlatCollapsed,     // default
    FlatExpanded,      // --no-collapse
}
```

**How it works:** `Grouped` retains its current semantics (grouped, no collapse). A new
`GroupedCollapsed` variant activates grouped-collapse behavior. The dispatch `match
self.render` in `TerminalReporter::render()` gains a fourth arm:

```rust
FindingsRender::GroupedCollapsed => {
    self.render_findings_grouped_collapsed(&mut out, findings);
}
```

The `run_analyze` construction site in `main.rs` gains a second grouped branch in its
if-expression. This is a mechanical, localized change.

**Exhaustiveness enforcement:** Adding `GroupedCollapsed` to the enum is a compile error
at every existing `match self.render` site that lacks an arm for it. The Rust compiler
forces every construction site and match to be updated — the same exhaustiveness guarantee
that made STORY-120's 28-site migration safe. Zero silent behavioral drift.

**Construction-site impact:** The `run_analyze` 3-way if-expression becomes a 4-way
expression:

```rust
render: if show_mitre_grouping && grouped_collapse {
    FindingsRender::GroupedCollapsed
} else if show_mitre_grouping {
    FindingsRender::Grouped
} else if collapse_findings {
    FindingsRender::FlatCollapsed
} else {
    FindingsRender::FlatExpanded
},
```

Whether `grouped_collapse` is a new bool param or derived from the CLI opt-in decision
is resolved in §4.

**Test helper impact:** `mitre_reporter` helpers across 4 test files currently use
`render: FindingsRender::Grouped`. They are unchanged — they express "grouped without
collapse," which remains the `Grouped` variant. New helpers using `GroupedCollapsed` are
added in STORY-119's test suite.

**Byte-identical preservation:** `FindingsRender::Grouped` arm is untouched. Existing
grouped-mode tests continue to assert on exactly current output. No test churn on the
existing `Grouped` path.

**Semver:** Adding a variant to a `pub` enum is a breaking change under Cargo semver
(RFC 1105; exhaustive matches must be updated). This is the same classification as
STORY-120's struct change. The correct version bump is `0.9.x → 0.10.0` (for a `0.y.z`
crate the breaking component is `y`).

**Pros:**
- Compiler-enforced exhaustiveness at every match site.
- `Grouped` semantics unchanged — zero output drift on existing `--mitre` users.
- Consistent with ADR-0003 Rule 5 ("new rendering mode → new variant").
- Clean orthogonal expression: each of the four cases is explicit.

**Cons:**
- Requires a semver minor bump (`0.9.x → 0.10.0`), though this is a single-crate binary
  not consumed as a library by external users (semver consequence is minor in practice).
- Four construction sites emit `GroupedCollapsed`; minor dispersal vs. today.

---

### Option B — `collapse_within_groups: bool` companion field on `TerminalReporter`

```rust
pub struct TerminalReporter {
    pub use_color: bool,
    pub show_hosts_breakdown: bool,
    pub render: FindingsRender,
    pub collapse_within_groups: bool,  // new — only meaningful when render = Grouped
}
```

**How it works:** The `FindingsRender::Grouped` match arm inspects `self.collapse_within_groups`
to decide whether to apply the per-bucket collapse pass:

```rust
FindingsRender::Grouped => {
    if self.collapse_within_groups {
        self.render_findings_grouped_collapsed(&mut out, findings);
    } else {
        self.render_findings_grouped(&mut out, findings);
    }
}
```

**Pros:**
- No enum change; no semver bump beyond what is already implied by the struct change.
- Orthogonality argument: collapse-within-grouped is "a property of the Grouped mode,
  not a new mode."

**Cons:**
- **Re-introduces an illegal state.** `collapse_within_groups = true` with
  `render = FindingsRender::FlatCollapsed` or `FlatExpanded` is a representable,
  meaningless combination — exactly the category of problem that ADR-0003 / STORY-120
  was designed to eliminate. The Rust compiler provides no enforcement.
- **Violates ADR-0003 Binding Rule 5 literally.** Rule 5 says: "A bool field on
  `TerminalReporter` that encodes a mutually-exclusive rendering mode is prohibited."
  Grouped vs. GroupedCollapsed are mutually exclusive rendering modes.
- **Construction-site exhaustiveness lost.** Adding `collapse_within_groups` to the
  struct is a compile error at all 28 sites (good — forces update), but any site that
  accidentally sets `collapse_within_groups: true` with `render: FlatCollapsed` compiles
  silently.
- Increases struct size and API surface.

**Recommendation: Reject Option B** on grounds of illegal-state reintroduction and ADR-0003
Rule 5 violation.

---

### Option C — Rename `Grouped` to `GroupedExpanded`, add `GroupedCollapsed`

```rust
pub enum FindingsRender {
    GroupedExpanded,   // was: Grouped — --mitre, no collapse
    GroupedCollapsed,  // new — --mitre + collapse
    FlatCollapsed,
    FlatExpanded,
}
```

**How it works:** Same as Option A but renames `Grouped` → `GroupedExpanded` for symmetry
with `FlatExpanded`.

**Pros:** Symmetric naming across the four variants (every variant is `<mode><collapse-state>`).

**Cons:**
- Adds significant migration cost at all 28 construction sites: every `FindingsRender::Grouped`
  must become `FindingsRender::GroupedExpanded`. This is identical in scope to the STORY-120
  migration (26 test-file sites + 2 main.rs sites).
- Naming churn in all 12 BCs that mention `FindingsRender::Grouped` by name, plus ADR-0003.
- No behavioral gain over Option A — symmetry is aesthetic, not functional.
- Produces a v0.10.0 boundary identical to Option A but with larger migration surface.

**Recommendation: Reject Option C** unless there is a strong preference for enum-name
symmetry. The migration cost is disproportionate to the naming benefit.

---

### Recommendation: Option A

**Add `FindingsRender::GroupedCollapsed`; leave `FindingsRender::Grouped` with its current
semantics (--mitre, no collapse, backward-compatible).**

This is the only option consistent with ADR-0003 Binding Rule 5, the illegal-state
elimination rationale, and the Rust exhaustiveness-enforcement pattern established by
STORY-120. The semver consequence (`0.9.x → 0.10.0`) is the correct and expected
classification for a breaking enum change in a `0.y.z` crate.

---

## §4 — CLI/UX DECISION: How Does a User Opt into Grouped Collapse? (CRITICAL — human gate required)

### Background

The current flag semantics are:
- `--mitre` → `FindingsRender::Grouped` (grouped, no collapse)
- default (no flags) → `FindingsRender::FlatCollapsed` (flat + collapse)
- `--no-collapse` → `FindingsRender::FlatExpanded` (flat + no collapse)
- `--mitre` + `--no-collapse` → `FindingsRender::Grouped` (mitre wins; --no-collapse has
  no effect per BC-2.11.013 Invariant 4 / EC-007)

STORY-119 adds grouped collapse. The question is how a user reaches `FindingsRender::GroupedCollapsed`.

---

### Option 4-A — `--mitre` alone now implies grouped-collapse (behavior change)

`--mitre` → `FindingsRender::GroupedCollapsed` (default, grouped + collapse)  
`--mitre --no-collapse` → `FindingsRender::Grouped` (--no-collapse opts out of grouped collapse)

**Behavior change:** Any existing `--mitre` user immediately sees collapsed output.
Findings that were individually listed per tactic now collapse within each bucket. This
is NOT byte-identical to current `--mitre` output.

**Backward-compatibility classification:** Breaking change to observable terminal output.
Scripts that parse `--mitre` terminal output for specific line counts or formats will
break. `--no-collapse` gains a second purpose: "suppress grouped collapse" in addition
to "suppress flat collapse."

**Rationale for:** Matches the flat-mode precedent — flat collapse is default-on because
the flooding problem is the motivation. If grouped collapse is worthwhile, it should be
the default for `--mitre` users for the same reason.

**Rationale against:** `--mitre` is currently documented and tested as producing one
line per finding within each tactic bucket. Changing `--mitre` default output is a
user-visible behavior change that may surprise operators who rely on grouped mode
specifically for its verbosity (per-finding detail, no grouping). The "make it
default-on" argument is stronger for flat mode (where flooding is the common case)
than for grouped mode (where bucket structure already provides organization).

**Interaction complexity:** `--no-collapse` acquires dual semantics (flat: suppress flat
collapse; grouped: suppress grouped collapse). This is coherent but requires BC updates
to BC-2.11.028 to document the dual role.

---

### Option 4-B — `--mitre` stays expanded by default; grouped collapse is implicit when `--no-collapse` is NOT set (recommended)

`--mitre` (alone) → `FindingsRender::GroupedCollapsed` — same as 4-A.

This option is equivalent to 4-A in terms of user behavior. The distinction is framing:
4-B emphasizes that "collapse is the system default; `--no-collapse` opts out of ALL
collapse modes (both flat and grouped)." In practice this is the same change as 4-A.

**Distinction from 4-A:** 4-B's framing is that the *collapse system* is the default
at every rendering level; `--no-collapse` uniformly disables it. 4-A's framing is that
`--mitre` changes behavior. From a user perspective they are identical.

**Backward-compatibility:** Same as 4-A — `--mitre` users see different output.

---

### Option 4-C — New dedicated opt-in flag (e.g., `--collapse-grouped` or `--mitre-collapse`)

`--mitre` → `FindingsRender::Grouped` (unchanged)  
`--mitre --collapse-grouped` → `FindingsRender::GroupedCollapsed` (explicit opt-in)

**Backward-compatibility:** Fully backward-compatible. All existing `--mitre` behavior
is preserved exactly. No output change for any current user. No change to `--no-collapse`.

**Pros:** Zero behavioral regression. Clean explicit semantics. The grouped-collapse
feature is discoverable via `--help` without changing existing behavior.

**Cons:** Adds CLI surface (new flag). Users must discover and use the flag. Does not
address the grouped-mode flooding problem by default. Inconsistent with the flat-mode
precedent (flat collapse is default-on; grouped collapse would be opt-in only).

**Flag naming considerations:**
- `--collapse-grouped`: symmetric with `--no-collapse`
- `--mitre-collapse`: mitre-namespaced, clear association
- Either conflicts with no existing flags

---

### Analysis: Backward-Compatibility Trade-off

The critical question is: how many existing `--mitre` users are likely to be affected by
a default behavior change? The `--mitre` flag activates a structured output mode used by
analysts who want MITRE tactic organization. Within each bucket, findings are already
sorted by verdict/confidence. For captures with many identical findings within a tactic
bucket (e.g., repeated HTTP anomalies all tagged to a given tactic), collapse would reduce
noise. For captures where each finding in a bucket is distinct, grouped collapse is
no-op (all singletons, same output). The output change is therefore only visible to users
who have repeated identical-key findings within a tactic bucket — likely a common case
for the same flooding scenarios that motivated flat collapse.

The `--no-collapse` flag providing an escape hatch (under 4-A/4-B) means the behavioral
regression is recoverable with one flag. The asymmetry between flat collapse (default-on)
and grouped collapse (default-off via 4-C) would be surprising to a user who expects
consistent behavior.

---

### Recommendation: Option 4-A / 4-B (they are equivalent implementations)

**Make grouped collapse the default behavior of `--mitre`, symmetrically with flat
collapse being the default of the flat path. `--no-collapse` suppresses both.**

This is the clean, user-facing consistent design: collapse is on by default; `--no-collapse`
is the universal opt-out. The construction site logic in `run_analyze` becomes:

```rust
render: if show_mitre_grouping && collapse_findings {
    FindingsRender::GroupedCollapsed   // --mitre, collapse on (default)
} else if show_mitre_grouping {
    FindingsRender::Grouped            // --mitre --no-collapse
} else if collapse_findings {
    FindingsRender::FlatCollapsed      // default flat (no --mitre)
} else {
    FindingsRender::FlatExpanded       // --no-collapse, no --mitre
},
```

Where `collapse_findings` (the existing bool derived from `!no_collapse`) controls
collapse across both flat and grouped paths. No new CLI flag is introduced. `--no-collapse`
gains dual scope but its intent ("I want to see every finding individually") is coherent
for both paths.

**FLAG THIS AS A BEHAVIOR CHANGE for the human gate.** This changes existing `--mitre`
output for any capture with repeated identical-key findings within a tactic bucket. All
existing test vectors that assert "no (xN) in grouped mode" (BC-2.11.013 Invariant 4,
BC-2.11.026 EC-007, BC-2.11.025 EC-011) are REWRITTEN to use `FindingsRender::Grouped`
(--mitre --no-collapse) to preserve the "grouped, no collapse" assertion path. The
`FindingsRender::Grouped` variant continues to exist and test the suffix-free guarantee.

If the human gate decides backward-compatibility is paramount: choose Option 4-C (new
flag). This analysis recommends 4-A/4-B on parity-of-design grounds, but acknowledges
the behavior change is non-trivial.

---

## §5 — Affected BCs: Deferral Clauses to Revise and New BCs Required

### Deferral clauses to revise (3 existing BCs)

| BC | Version (current) | Deferral clause location | Required change |
|----|------------------|--------------------------|-----------------|
| BC-2.11.013 | v1.13 | Invariant 4 last sentence: "Collapse within grouped/`--mitre` mode is deferred to STORY-119 (future cycle)" | Remove deferral sentence; update to describe STORY-119-implemented behavior. If 4-A/4-B: `GroupedCollapsed` variant activates grouped collapse; `Grouped` variant remains suffix-free. If 4-C: `Grouped` remains suffix-free; `GroupedCollapsed` requires explicit flag. |
| BC-2.11.025 | v1.10 | Invariant 5 last sentence: "Grouped-mode collapse is deferred to a future cycle (see STORY-119)" | Revise to: grouped-mode collapse is governed by `FindingsRender::GroupedCollapsed` (STORY-119); this BC remains scoped to `FlatCollapsed`. No behavioral change to this BC's core contract. |
| BC-2.11.026 | v1.11 | PC-4 last sentence (grouped/--mitre path MUST NOT emit suffix): "Collapse within grouped/--mitre mode is deferred to STORY-119 (future cycle)" | PC-4's suffix prohibition reworded: `FindingsRender::Grouped` MUST NOT emit ` (xN)` suffix (unchanged); `FindingsRender::GroupedCollapsed` MAY emit ` (xN)` suffix per its own BC. EC-007 and EC-009 updated to scope the suffix-free guarantee to `FindingsRender::Grouped` only. |

### New BCs required (minimum set for full authorship at F2)

| BC ID (proposed) | Title | Key postconditions |
|-----------------|-------|--------------------|
| BC-2.11.030 | Grouped-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key Within Each Tactic Bucket; Post-Sort Order; Deterministic | Same collapse key as BC-2.11.025; applied per-bucket; group order is first-occurrence within the sorted bucket order (not original emission order — sorted by verdict/confidence/index first); Vec accumulator canonical (same rationale as BC-2.11.025 Invariant 7). |
| BC-2.11.031 | Grouped-Mode Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix | Mirrors BC-2.11.026; scoped to `FindingsRender::GroupedCollapsed`; same color-ladder rule; suffix appended before colorization; MITRE line uses `render_finding_grouped`'s em-dash + name format (not flat-mode's plain ID list). |
| BC-2.11.032 | Grouped-Mode Collapse Retains At Most K=3 Representative Evidence Lines per Bucket Group | Mirrors BC-2.11.027; same K=3, same positional rule, same no-sliding-window invariant; scoped to per-bucket groups. |
| BC-2.11.033 | --no-collapse Suppresses Both Flat and Grouped Collapse (under option 4-A/4-B only) | If 4-A/4-B chosen: `--no-collapse` produces `FindingsRender::Grouped` when `--mitre` is also set. `FindingsRender::Grouped` is suffix-free (BC-2.11.013 Invariant 4 unchanged). This BC documents the dual opt-out scope of `--no-collapse`. If 4-C chosen: BC-2.11.028 is amended instead; no new BC needed for flag semantics. |

**Note on bucket ordering interaction (for BC-2.11.030):** The STORY-118 collapse key
contract specifies "first-occurrence order" over the raw input slice. In grouped mode,
the per-bucket sort (verdict-rank, confidence-rank, emission-index) intervenes before
collapse. The new BC must specify whether "first occurrence" means first in the sorted
bucket order or first in the original emission order. The recommendation is: first in
the sorted bucket order (this is the natural implementation; the sort happens first at
terminal.rs:463–467 before the inner rendering loop). This is a distinct but analogous
ordering to BC-2.11.025's input-slice-order guarantee.

---

## §6 — Affected Tests

### Existing grouped-mode tests that assert "no (xN) in grouped" — impact depends on §4 decision

These tests currently use `FindingsRender::Grouped` (or the `mitre_reporter()` helper
which sets `Grouped`) and assert that no ` (xN)` suffix appears:

| Test | File | What it asserts | Impact under 4-A/4-B | Impact under 4-C |
|------|------|-----------------|-----------------------|------------------|
| `test_BC_2_11_013_grouped_mode_suffix_free` | `reporter_terminal_tests.rs` | No ` (xN)` in grouped output for N=100 identical-key findings | Construction site changes: `FindingsRender::Grouped` → still correct (Grouped = no collapse, no suffix). Test continues to pass unchanged. | Unchanged |
| `test_BC_2_11_025_grouped_mode_bypasses_collapse` | `reporter_terminal_tests.rs` (line ~2068-2099) | `render = FindingsRender::Grouped` → no collapse | Construction site remains `FindingsRender::Grouped`; test continues to assert no suffix. | Unchanged |
| BC-2.11.026 EC-007, EC-009 tests | `reporter_terminal_tests.rs` | `render = FindingsRender::Grouped`, N=100 → 100 individual lines, no suffix | Construction site remains `FindingsRender::Grouped`; tests continue to pass (Grouped variant is suffix-free). | Unchanged |
| `mitre_reporter()` helper tests in `reporter_tests.rs` (6 sites) | `reporter_tests.rs` | Various grouped-mode output assertions | Under 4-A/4-B: these helpers use `FindingsRender::Grouped` (no collapse); they remain correct UNLESS their test inputs happen to have repeated identical-key findings AND the test asserts non-collapsed line counts. Must audit: if the test inputs have N≥2 identical-key findings within a bucket AND the test passes `FindingsRender::Grouped`, the test still passes (Grouped = no collapse). No churn. | Unchanged |

**Key insight:** Under the recommendation (Option A + 4-A/4-B), `FindingsRender::Grouped`
retains its current semantics — grouped, no collapse, suffix-free. All existing tests
that use `FindingsRender::Grouped` continue to pass without modification. The only
construction sites that change are the `run_analyze` site in `main.rs` (which now routes
to `GroupedCollapsed` vs. `Grouped` depending on `--no-collapse`) and any new test
helpers that exercise the new `GroupedCollapsed` variant.

### New tests needed (for STORY-119 `mod story_119` block)

| Test name (proposed) | What it verifies |
|---------------------|-----------------|
| `test_BC_2_11_030_per_bucket_collapse_groups_by_key` | N identical-key findings in one tactic bucket → 1 collapsed group with count N; findings in a different bucket are unaffected |
| `test_BC_2_11_030_first_occurrence_in_sorted_bucket_order` | Post-sort order determines group representative; representative is first in verdict/confidence/index order, not original emission order |
| `test_BC_2_11_030_different_buckets_not_cross_collapsed` | Two findings with same collapse key but different MITRE tactic → land in different buckets → two separate collapsed groups (one per bucket); no cross-bucket collapse |
| `test_BC_2_11_031_grouped_collapse_suffix_format` | N=3 in one bucket → ` (x3)` suffix in bucket; singleton in same report → no suffix |
| `test_BC_2_11_031_grouped_collapse_mitre_line_em_dash_format` | MITRE line for a grouped-collapse group uses em-dash + name format (not plain ID list) — inherited from `render_finding_grouped` |
| `test_BC_2_11_031_grouped_collapse_color_ladder` | Likely/High group → suffix inside red-bold span (mirrors BC-2.11.026 EC-008) |
| `test_BC_2_11_032_evidence_sampling_k3_in_bucket` | Group of N=5 in a bucket → at most 3 evidence lines (first 3 members' evidence[0]) |
| `test_BC_2_11_033_no_collapse_suppresses_grouped_collapse` (4-A/4-B only) | `FindingsRender::Grouped` (= --mitre --no-collapse) → no suffix even with N=100 identical-key findings in a bucket |
| `test_BC_2_11_013_grouped_collapsed_preserves_bucket_order` | Tactic bucket order per `all_tactics_in_report_order()` is unchanged after per-bucket collapse |
| `test_BC_2_11_030_singleton_bucket_unchanged` | A bucket with only singleton findings → output byte-identical to current grouped mode for those findings |

### Existing tests NOT affected

- All `FlatCollapsed` tests (BC-2.11.025–029 suite): unchanged.
- All `FlatExpanded` tests: unchanged.
- `escape_for_terminal` proptest (VP-012): unchanged.
- `all_tactics_in_report_order()` ordering tests: unchanged.

---

## §7 — Verification: VP Assessment

**No new VP is required for STORY-119.**

Grouped-mode collapse is a display-layer transform with the same purity classification as
flat-mode collapse (pure core, no I/O, no global state, deterministic). The existing VP
family covers:

- **VP-012** (`escape_for_terminal` correctness, proptest, P1): unchanged. The grouped
  collapse path calls `escape_for_terminal` identically to the flat collapse path; the
  escape invariant is path-independent (BC-2.11.010 Postcondition 3). No new proof
  harness needed.
- **VP-016** (tactic headers in canonical order, integration): unchanged. Tactic bucket
  order is not affected by within-bucket collapse.

STORY-119's correctness is fully covered by deterministic unit tests (§6 above) against
the new BCs. The property "grouped-collapse produces count suffix iff N≥2" is not a
formal-verification target under the current VP tier allocation — it is a
postcondition-level integration test, not a security boundary, arithmetic invariant, or
state-machine property requiring model checking or fuzz coverage.

**Existing VP-INDEX and `verification-coverage-matrix.md` are unchanged by STORY-119.**

---

## §8 — Regression Risk and Construction-Site Census

### Regression risk assessment

| Risk | Severity | Mitigation |
|------|----------|-----------|
| `FindingsRender::Grouped` output changes accidentally | HIGH | Addressed by Option A: `Grouped` variant is untouched; compiler enforces no Grouped arm code changes. |
| `FlatCollapsed` / `FlatExpanded` output changes | HIGH | These paths are not touched. `cargo test --all-targets` is the gate. |
| Cross-bucket collapse (findings from different buckets incorrectly grouped) | MEDIUM | The per-bucket collapse pass receives a per-bucket slice, not the global findings slice. `collapse_findings_pass` operates on the slice it receives; it has no knowledge of other buckets. Test: `test_BC_2_11_030_different_buckets_not_cross_collapsed`. |
| Tactic bucket order changed by collapse | MEDIUM | Collapse is applied within each bucket's inner loop; the `for tactic in all_tactics_in_report_order()` outer loop is unchanged. Test: `test_BC_2_11_013_grouped_collapsed_preserves_bucket_order`. |
| Singleton findings in grouped mode get spurious suffix | HIGH | BC-2.11.031's N=1 rule (same as BC-2.11.026 PC-2): singletons produce no suffix. Covered by singleton tests. |
| `render_finding_grouped` (em-dash MITRE format) accidentally used for flat collapse | LOW | The two paths call different functions. `GroupedCollapsed` calls a new `render_findings_grouped_collapsed` helper; `FlatCollapsed` calls `render_findings_collapsed`. No shared mutable state. |
| Evidence sampling window slides past empty-evidence members | MEDIUM | `collapse_findings_pass` reuse: same `COLLAPSE_EVIDENCE_SAMPLES` constant and same evidence[0] positional logic. Inherited from BC-2.11.027 implementation. |

### Construction-site census

**Files that must change in STORY-119:**

| File | Change | Notes |
|------|--------|-------|
| `src/reporter/terminal.rs` | Add `FindingsRender::GroupedCollapsed` variant; add `GroupedCollapsed` arm to `match self.render`; add `render_findings_grouped_collapsed` function; modify `render_findings_grouped` if it is refactored to share per-bucket logic | Primary implementation file |
| `src/main.rs` | Update `run_analyze` construction site (3-way if → 4-way if); `run_summary` site unchanged (`FlatCollapsed`, inert) | 1 site changes (run_analyze); run_summary unchanged |
| `tests/reporter_terminal_tests.rs` | Add `mod story_119` block; add `grouped_collapse_reporter()` helper; add grouped-collapse tests | No existing tests change (see §6) |
| `tests/reporter_tests.rs` | Possibly add grouped-collapse construction site tests; existing 6 `Grouped` sites unchanged | Likely additive only |
| `tests/dnp3_f5_remediation_tests.rs` | `mitre_reporter` helper unchanged (`FindingsRender::Grouped`) | No change |
| `tests/bc_2_09_100_multitag_tests.rs` | Parameterized helper unchanged | No change |
| `Cargo.toml` | Version bump `0.9.x → 0.10.0` | Breaking enum change |
| `docs/adr/0003-reporting-pipeline-layering.md` | Add subsection "Grouped-Mode Collapse (STORY-119 — v0.10.0)" | Documents the §3 and §4 decisions |

**Files that must change only in F2 (spec evolution, before STORY-119 is dispatched):**

| File | Change |
|------|--------|
| BC-2.11.013 | Remove deferral clause from Invariant 4; update EC-007 |
| BC-2.11.025 | Remove deferral sentence from Invariant 5 |
| BC-2.11.026 | Scope PC-4 suffix prohibition to `FindingsRender::Grouped`; update EC-007/EC-009 |
| New BC-2.11.030, 031, 032, 033 | Author from scratch per §5 |
| STORY-119.md | Status: draft → ready; populate ACs, tasks, inputs, input-hash |

---

## §9 — Open Questions for Human Gate

The following questions require explicit human decision before F2 (spec evolution) can
begin. No implementation work may start until these are resolved.

**OQ-1 (CRITICAL — type-system representation):**
Adopt Option A (`GroupedCollapsed` new enum variant, version → 0.10.0)?
Or Option B (companion bool field, rejected by this analysis)?
Or Option C (rename `Grouped` → `GroupedExpanded`, higher migration cost)?
*Recommendation: Option A.*

**OQ-2 (CRITICAL — CLI/UX opt-in):**
- Option 4-A/4-B: `--mitre` alone now implies grouped collapse (default-on, behavior
  change, `--no-collapse` opts out); OR
- Option 4-C: new explicit flag (`--collapse-grouped` or `--mitre-collapse`) required;
  `--mitre` default output unchanged.
*Recommendation: 4-A/4-B (default-on, consistent with flat-mode precedent), but this
is a user-visible behavior change that requires explicit sign-off.*

**OQ-3 (scoping):** Should BC-2.11.033 (--no-collapse dual scope) be a new BC or an
amendment to BC-2.11.028? A new BC preserves BC-2.11.028's v0.8.0 provenance; an
amendment avoids a new BC number for a minor scope clarification.
*Recommendation: Amendment to BC-2.11.028, not a new BC. The opt-out semantics
extension is narrow and traces to the same `--no-collapse` contract.*

**OQ-4 (version boundary):** Is `0.9.x → 0.10.0` the correct bump, or should this
wait for a larger batch of breaking changes? The crate is a single-binary tool; the
semver consequence is primarily for Cargo dependency resolution if anyone pins the crate.
*Recommendation: Accept 0.10.0 as the correct classification; ship STORY-119 in a
dedicated breaking-change release following the gitflow release branch pattern.*

**OQ-5 (bucket ordering):** Confirm that "first occurrence in sorted-bucket order" is
the correct group-representative definition for BC-2.11.030 (as recommended in §5).
Alternative: use original emission index even after sorting (more complex to implement —
requires tracking pre-sort positions through the collapse pass).
*Recommendation: First in sorted-bucket order. This is the natural implementation,
consistent with the spirit of BC-2.11.025's first-occurrence guarantee, and the sorted
order is already deterministic.*

---

## §10 — Summary of Key Decisions

### Two critical decisions awaiting human gate:

**Decision 1 — Type-system representation:**
Recommendation: **Option A — add `FindingsRender::GroupedCollapsed`** as a fourth enum
variant. `Grouped` retains its current suffix-free semantics (--mitre --no-collapse).
`GroupedCollapsed` activates per-bucket collapse (--mitre, default). This preserves
ADR-0003 Binding Rule 5, the illegal-state elimination rationale, and compiler
exhaustiveness enforcement. Semver: 0.9.x → 0.10.0.

**Decision 2 — CLI/UX opt-in:**
Recommendation: **Option 4-A/4-B — `--mitre` alone implies grouped collapse (default-on)**;
`--no-collapse` suppresses grouped collapse (as well as flat collapse). No new CLI flag.
This is a non-byte-identical behavior change to existing `--mitre` output for captures
with repeated identical-key findings within a tactic bucket. The `FindingsRender::Grouped`
variant continues to exist and represents the `--mitre --no-collapse` combination.
The human gate must explicitly approve this behavior change before F2 spec authorship.

---

*Produced by: architect | Date: 2026-06-18 | Inputs: STORY-119.md, STORY-120.md, BC-2.11.013 v1.13, BC-2.11.025 v1.10, BC-2.11.026 v1.11, docs/adr/0003-reporting-pipeline-layering.md, src/reporter/terminal.rs (post-STORY-120 f851995)*
