# Research: Idiomatic Rust Type-Design for a 2-Axis Rendering-Mode Matrix

**Type:** general (technology / type-design)
**Date:** 2026-06-18
**For:** STORY-119 (grouped-mode finding-collapse)
**Status:** complete

---

## Problem Statement

wirerust's `TerminalReporter` carries a `FindingsRender` enum (shipped v0.9.0, STORY-120,
defined in `src/reporter/terminal.rs:101`):

```rust
pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }
```

This 3-variant enum was introduced (ADR-0003 Binding Rule 5, Issue #62) to make an *illegal
state* unrepresentable: in v0.8.0 two bools (`show_mitre_grouping`, `collapse_findings`) could
both be `true`, but grouped mode did not support collapse, so that combination was meaningless.
The enum encoded exactly the 3 valid modes.

STORY-119 **adds** grouped-mode collapse. The two axes therefore become **fully orthogonal**:

- **Axis 1 — grouping:** `Grouped` (by MITRE tactic) vs `Flat`
- **Axis 2 — collapse:** `Collapsed` (repeated findings → `(xN)`) vs `Expanded` (one line each)

All FOUR combinations are now valid: Grouped+Collapsed, Grouped+Expanded, Flat+Collapsed,
Flat+Expanded.

This research evaluates three candidate representations and recommends one.

---

## Key Finding First (decisive constraint)

**wirerust is a binary crate, not a published library.** `Cargo.toml` declares only
`[package]` with a CLI binary and a `[[bench]]` target — there is **no `[lib]` target** and the
crate is not published to crates.io as an API dependency. Therefore `FindingsRender` being `pub`
has **no downstream semver consumers**. The entire "breaking change to API consumers" axis —
which dominates the general Rust discourse on enum evolution — is **moot here**. The only real
cost is **internal construction-site churn within this one crate**.

This single fact reorders the trade-off space substantially. Verified against
`/Users/zious/Documents/GITHUB/wirerust/Cargo.toml` (no `[lib]`, no `crate-type`, `name =
"wirerust"`, binary).

---

## Candidate Representations

### (a) Flat 4-variant enum (cartesian product as a sum type)
```rust
pub enum FindingsRender {
    GroupedCollapsed,
    GroupedExpanded,   // renames the existing `Grouped`
    FlatCollapsed,     // unchanged
    FlatExpanded,      // unchanged
}
```

### (b) Nested-data enum (sum-of-products)
```rust
pub enum Collapse { Collapsed, Expanded }
pub enum FindingsRender { Grouped(Collapse), Flat(Collapse) }
```

### (c) Struct of two orthogonal enums (product-of-sums)
```rust
pub enum Grouping { Grouped, Flat }
pub enum Collapse { Collapsed, Expanded }
pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }
```

All three are **algebraically isomorphic** — each has cardinality 4 (`2 × 2`), and there is a
natural bijection between any two. Equal cardinality does **not** imply equal ergonomics,
extensibility, or clarity; the *shape* of the type drives how code reasons about it
(Alexis King, "Parse, Don't Validate") [4].

---

## Research Question 1 — What is idiomatic when axes are genuinely orthogonal?

**Authoritative / widely-cited literature consensus: when two axes are genuinely orthogonal and
the full Cartesian product is valid, the idiomatic representation is a *product type* — a struct
of small named enums (option c).** Rationale drawn from the sources:

- **Sum vs product types.** Rust `enum` = sum type (a value is *one of* the variants, mutually
  exclusive). Rust `struct` = product type (a value holds *all* fields simultaneously). The
  community guidance is: use enums for "one-of-many" alternatives, structs for "a record of
  independent properties." Two orthogonal axes are, by definition, independent simultaneous
  properties → product type. (Rust Users Forum: enums-vs-structs discussion [1].)

- **"Make illegal states unrepresentable"** (corrode blog [2]) and **"Parse, Don't Validate"**
  (Alexis King [4]) both stress choosing a data structure whose *shape supports the invariants
  and control flow you want* — not merely one with the right cardinality. King explicitly warns
  against denormalized/entangled representations and against bolting boolean flags onto records;
  prefer factoring orthogonal concerns into precise types [4].

- **"Boolean flags are sad little enums"** [10] — the community norm is to replace bare bools
  with named enums. A pair of orthogonal bools (`grouped`, `collapsed`) is the classic
  anti-pattern this addresses. Both option (a) and option (c) fix the bool problem; option (c)
  additionally preserves the *independence* of the two axes at the type level.

- **Rust API Guidelines** [14]: prefer closed sets of named alternatives (enums) over opaque
  ints/bitflags; favor representational precision and future-proofing. (Note: the
  future-proofing page documents C-STRUCT-PRIVATE and C-NEWTYPE-HIDE; it does not itself cover
  `#[non_exhaustive]` — verified by fetching
  `rust-lang.github.io/api-guidelines/future-proofing.html`.)

- **Typestate / state-machine literature** [3][7][15]: typestate is for forbidding *illegal
  transitions/states*. It does **not** apply here — there are no illegal combinations and no
  transition constraints. So typestate machinery is unwarranted.

**Caveat on the literature:** the sources are general type-design essays and forum/blog
consensus, not a single authoritative ruling on this exact 2×2 case. The recommendation below
weights them against wirerust's concrete (binary-crate, 4 small modes) reality.

---

## Research Question 2 — Is a flat cartesian-product enum an anti-pattern? Trade-offs.

**Not inherently an anti-pattern, but it scales poorly (combinatorial variant explosion) and
entangles orthogonal axes.** It is acceptable when the domain is small/stable and the combined
modes are *conceptually atomic* [1][4][10][14].

| Concern | (a) Flat 4-variant | (b) Nested enum | (c) Struct-of-enums |
|---|---|---|---|
| **Exhaustive match on all 4** | Direct, concise (4 arms) | 4 nested arms or `match (g, c)` | `match (mode.grouping, mode.collapse)` — 4 arms |
| **Match on ONE axis only** | Verbose: must OR variants (`GroupedCollapsed \| GroupedExpanded`); fragile when axis extended | Easy for the *outer* axis (`Grouped(_)`); inner axis needs repetition | Easiest, symmetric: inspect `mode.grouping` directly, ignore the other |
| **Add a 3rd axis (e.g. color)** | Variant count **doubles** → 8, then 16 (combinatorial explosion) | Awkward — must nest more data or add fields per variant | Trivial: add one enum + one struct field; old code ignores it |
| **Construction ergonomics** | Name the combined variant; can't compose from independent decisions without a mapping helper | Pair grouping with collapse; grouping is privileged | Fully compositional: compute each axis separately, then bundle |
| **Symmetry of axes** | Implicit (encoded in names only) | Asymmetric — privileges grouping over collapse | Explicit and symmetric |

**Verdict on Q2:** the flat enum entangles the axes and explodes combinatorially if a 3rd axis is
ever added; the nested enum privileges one axis (good only when one axis is clearly primary,
which is *not* the case here — grouping and collapse are co-equal). The struct-of-enums is the
cleanest for orthogonal, co-equal axes and the only one that scales gracefully to a 3rd axis.

---

## Research Question 3 — Does a struct-of-two-enums reintroduce "illegal state" risk?

**No.** With `Grouping` (2 variants) × `Collapse` (2 variants), the struct has exactly 4 inhabit­
ants, each a domain-valid mode. There are no extra/invalid values to guard against — every
combination is legal *by definition of STORY-119* [2][4][11].

The illegal-states mantra targets representations that can encode *invalid* combinations (empty
username, `start > end`, transitionless state machine). That concern does **not** arise when the
domain explicitly declares all combinations valid. A struct *can* admit illegal states in *other*
domains (e.g. `struct { start: Time, end: Time }` allows `start > end`), but only when a
cross-field invariant exists. Here there is none, so the struct is exactly as safe as the flat
enum.

**Bonus — future-proofing if an invariant later appears:** should a future constraint forbid some
combination (e.g. "Grouped+Expanded disallowed in context X"), the struct-of-enums refactors
cleanly via a newtype smart constructor (`struct ValidMode(FindingsRender)` with a checked
`new() -> Result<...>`), per the corrode/King pattern [2][4]. The flat enum makes such a
per-axis constraint clumsier because the axes aren't separable.

---

## Research Question 4 — Precedent in well-known Rust CLI / reporting crates

This is the **most inconclusive** question — I did not find a crisp, citable "orthogonal display
axes modeled as struct-of-enums" canonical example in a single source. General-purpose patterns
observed in the ecosystem (model knowledge + general consensus, flagged as such):

- **`tracing-subscriber` fmt layer** — exposes orthogonal formatting choices (compact/pretty/json
  form, ANSI on/off, span events, target on/off) as **independent builder methods on a `fmt`
  layer** (a builder-struct of independent fields), *not* as one giant enum of every combination.
  This is the builder-of-orthogonal-fields analogue of option (c). *(Pattern recollection, not
  verified against current docs in this session — flagged medium confidence.)*

- **`clap`** — derive-based CLIs model independent flags as independent struct fields (the args
  struct), again a product-of-options rather than a cartesian-product enum. Directly analogous to
  treating grouping and collapse as two fields.

- **`miette` / `ariadne`** diagnostic renderers — expose orthogonal rendering knobs (color,
  unicode vs ascii, context-line count) as independent config fields/builder setters, not as a
  combined-mode enum.

- **`ratatui`** — style/modifier flags are modeled with `bitflags`-style `Modifier` (orthogonal
  toggles) plus separate `Color` fields; orthogonal visual axes are kept independent.

**Honest limitation:** I could not retrieve current Context7/registry docs to pin exact
type signatures for these crates in this session (Context7 not invoked — see Research Methods).
The directional consensus — *well-known Rust tools model orthogonal display axes as independent
fields/builder setters, not as a single cartesian-product enum* — is consistent across the
ecosystem and aligns with the literature in Q1/Q2. Treat specific crate API shapes as
**unverified / medium confidence** until checked against live docs. **No version-specific claims
are made**, so no registry verification was required here.

---

## Research Question 5 — Migration / semver angle and the construction-site cost

**Semver / downstream consumers: N/A — wirerust is a binary crate (verified, see "Key Finding").**
The standard Rust rule that *adding a variant to a non-`#[non_exhaustive]` public enum is a
breaking change* (cargo SemVer reference, `enum-variant-new`; `#[non_exhaustive]` as the escape
hatch) [cargo SemVer ref; effective-rust] is **not binding** here because there is no published
library API. It matters only as a hypothetical if wirerust were ever split into a `lib` + `bin`.

**Actual cost = internal construction-site + match-arm churn.** Measured in the repo:

- `render:\s*FindingsRender` field initializations (true construction sites): **46 occurrences
  across 7 files** — concentrated in `tests/reporter_tests.rs` (17), `tests/reporter_terminal_tests.rs`
  (15), and a handful in `src/main.rs`, `src/reporter/terminal.rs`, docs, and other tests.
  (The prompt's "28 sites" is a close estimate; the literal grep of `render: FindingsRender::`
  patterns is higher once test fixtures are counted. Either way the bulk is **test fixtures**,
  which are cheap, mechanical edits.)
- Exhaustive `match self.render { ... }` dispatch lives in `src/reporter/terminal.rs:202-224`
  (one site, 3 arms today → 4 arms after).

Churn comparison (within-crate only):

| Option | Construction-site churn | Match-arm churn | Preserves existing names? |
|---|---|---|---|
| **(a) Flat 4-variant** | Lowest — only sites using `Grouped` change (→ `GroupedExpanded`); all `FlatCollapsed`/`FlatExpanded` sites untouched; add new `GroupedCollapsed` sites | Add 1 arm; rename 1; keeps flat `match` shape | **Best** — keeps `FlatCollapsed` + `FlatExpanded`; renames only `Grouped` |
| **(b) Nested enum** | All sites rewritten to `Grouped(Collapse::_)` / `Flat(Collapse::_)` | All arms rewritten to nested patterns | None survive |
| **(c) Struct-of-enums** | All sites rewritten to struct-literal `FindingsRender { grouping, collapse }` | All arms rewritten to `match (g, c)` / field patterns (most verbose) | None survive |

Migration-cost ranking (least → most): **(a) < (b) ≈ (c)**, per the semver/reasoning analysis
[cargo SemVer ref; effective-rust]. But note: because nearly all sites are **mechanical test
fixtures**, even (c)'s "all sites change" is a low-risk, search-and-replace edit — not a
genuine engineering risk. The match-arm rewrite for (c) is the only non-trivial part, and it is
a single dispatch site (`terminal.rs:202`).

---

## RECOMMENDATION

**Adopt option (c): a struct of two orthogonal enums.**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping { Grouped, Flat }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collapse { Collapsed, Expanded }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingsRender { pub grouping: Grouping, pub collapse: Collapse }
```

**Rationale, tied to the four stated criteria:**

1. **Illegal-states-unrepresentable:** All four combinations are genuinely valid, so the struct
   reintroduces *zero* illegal states (Q3) [2][4]. The original v0.8.0 illegal state
   (`grouped && collapsed`) was eliminated precisely because grouped-collapse was meaningless —
   STORY-119 makes it meaningful, so the constraint that justified the 3-variant enum **no longer
   exists**. The type must now express orthogonality, and a product type is the faithful encoding.

2. **Exhaustiveness:** `match (mode.grouping, mode.collapse)` is still compiler-checked
   exhaustive over all four arms; the dispatch in `terminal.rs:202` rewrites to a single 4-arm
   tuple match. Code paths that care about only *one* axis (e.g. "is this grouped, regardless of
   collapse?") become a clean `mode.grouping` inspection instead of an error-prone variant-OR
   [1][4].

3. **Extensibility:** This is the strongest argument given wirerust's trajectory (ADR-0003
   reporting pipeline is actively evolving). If a 3rd display axis ever lands (color mode,
   verbosity, sort order), option (c) adds one enum + one field and leaves existing code
   untouched; option (a) doubles the variant count and explodes the dispatch (Q2) [4][14].

4. **Migration cost:** Higher than (a) on paper, but the churn is overwhelmingly **mechanical
   test-fixture edits** (the bulk of the ~46 sites) plus **one** real match-arm rewrite. Since
   wirerust is a **binary crate**, there is **no semver/downstream breakage** — the usual reason
   to prefer the minimal-churn option (a) does not apply. The one-time edit buys a strictly
   better long-term shape.

**When (a) would win instead:** if you were certain no 3rd axis will ever appear AND wanted the
absolute minimum diff AND valued preserving the `FlatCollapsed`/`FlatExpanded` variant names. For
a *published library* enum, (a) + `#[non_exhaustive]` would be the pragmatic call. wirerust is
neither, so (c) is preferred.

**Secondary recommendation (cheap insurance):** regardless of choice, keep the public-but-binary
type ergonomic by adding a couple of named constructors (e.g. `FindingsRender::grouped_collapsed()`)
or `Default` so the most common modes stay one-liners at construction sites and tests read well.

**Inconclusive / flagged:**
- Q4 crate-precedent specifics (tracing-subscriber/miette/ratatui exact API shapes) are
  **medium confidence** — directionally consistent with the recommendation but not verified
  against live docs this session.
- No library-version claims are made; nothing required registry verification beyond confirming
  wirerust's own crate type via `Cargo.toml`.

---

## Sources

Citations [1]–[16] are as numbered in the Perplexity `sonar-deep-research` synthesis. Key
identifiable sources:

- [2] corrode.dev — "Make Illegal States Unrepresentable" (Rust)
- [4] Alexis King — "Parse, Don't Validate" (lexi-lambda.github.io)
- [10] "Boolean flags are sad little enums" (community essay)
- [11] *Programming Rust* (Blandy/Orendorff) — illegal-states / enum guidance (via forum summary)
- [14] Rust API Guidelines — rust-lang.github.io/api-guidelines (future-proofing page verified
  via WebFetch: documents C-STRUCT-PRIVATE, C-NEWTYPE-HIDE, C-STRUCT-BOUNDS; does **not** cover
  `#[non_exhaustive]`)
- [1][3][6][7][12][15][16] — Rust Users Forum / Rust Internals / Hacker News threads on
  enums-vs-structs, typestate, and type-driven design

Semver/migration analysis (perplexity_reason) citations:
- cargo SemVer reference — `doc.rust-lang.org/cargo/reference/semver.html` (enum-variant-new;
  `#[non_exhaustive]`)
- effective-rust.com/semver.html
- predr.ag — "Turning a Rust struct into an enum is not always breaking" /
  "Some Rust breaking changes do not require a major version"
- cargo-semver-checks (crates.io / GitHub) — enum-variant-added lint

Repo facts verified locally:
- `src/reporter/terminal.rs:101` — current `FindingsRender` definition (3 variants)
- `src/reporter/terminal.rs:202-224` — exhaustive dispatch site
- `Cargo.toml` — binary crate, no `[lib]` target (decisive for the semver analysis)
- 46 `render: FindingsRender::…` occurrences across 7 files (bulk in test fixtures)

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source synthesis (reasoning_effort=high) on idiomatic Rust type-design for orthogonal enum axes: flat enum vs nested enum vs struct-of-enums, illegal-states-unrepresentable, parse-don't-validate, typestate, API Guidelines, community norms (Q1–Q3) |
| Perplexity perplexity_reason | 1 | Semver/breaking-change + construction-site churn ranking of the three refactors; cargo SemVer reference + `#[non_exhaustive]` (Q5) |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not invoked — Q4 crate-doc verification deferred; flagged as medium confidence rather than asserted |
| Tavily | 0 | — |
| WebFetch | 1 | Fetched Rust API Guidelines future-proofing page to verify C-STRUCT-PRIVATE / C-NEWTYPE-HIDE and the absence of `#[non_exhaustive]` coverage |
| WebSearch | 0 | — |
| Local repo tools (Read/Grep/Glob) | several | Verified `FindingsRender` definition, dispatch site, construction-site count, and crate type (`Cargo.toml`) |
| Training data | 1 area | Q4 crate-precedent API shapes (tracing-subscriber, clap, miette, ratatui) — explicitly flagged medium confidence, not verified against live docs this session |

**Total MCP tool calls:** 3 (2 Perplexity incl. 1 `perplexity_research`, plus 1 WebFetch grounding)
**Training data reliance:** low-to-medium — the type-design recommendation (Q1–Q3, Q5) is
source-grounded via `perplexity_research` + `perplexity_reason`; only the Q4 crate-precedent
specifics lean on model knowledge and are flagged as such.
