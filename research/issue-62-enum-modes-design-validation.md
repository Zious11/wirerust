# Issue #62 — Enum-of-Modes Design Validation (External Best-Practice Research)

**Date:** 2026-06-17
**Researcher:** vsdd-factory research agent
**Subject:** `TerminalReporter` render-bools → `FindingsRender` enum refactor (wirerust, GitHub issue #62)
**Plan under validation (F1-approved):**

```rust
pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }
pub struct TerminalReporter {
    pub use_color: bool,
    pub show_hosts_breakdown: bool,
    pub render: FindingsRender,
}
```

> Replaces three render bools (`show_mitre_grouping`, `collapse_findings`, plus the implicit flat-expanded default) that today permit the nonsensical `grouping=true && collapse=true` state. `use_color` and `show_hosts_breakdown` stay as orthogonal bools.

---

## Verdict Summary (one line per question)

| # | Question | Verdict | Confidence |
|---|----------|---------|------------|
| 1 | Bool-flags vs enum for mutually-exclusive state | **CONFIRMS** — enum is the idiomatic "make illegal states unrepresentable" choice; builder/typestate are for multi-step/cross-field construction, not one-shot mode selection | High |
| 2 | "3–4 parameter" smell threshold | **CONFIRMS (with refinement)** — Clippy's machine-enforced default is `>3` bool params; "acute at 3–4" is well-supported but is a heuristic, and applies per-axis, not raw count | High |
| 3 | Keep orthogonal bools, extract only the mutually-exclusive axis | **CONFIRMS** — this hybrid (enum for the mode axis, bools for orthogonal toggles) is exactly the recommended design | High |
| 4 | Semver impact of replacing public struct fields (0.8.x → 0.9.0) | **CONFIRMS** — removing/replacing public fields is a major breaking change; for 0.y.z the `y` component is the breaking component, so 0.8.x → 0.9.0 is correct and required | High |
| 5 | Derives `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` | **CONFIRMS** — this is the conventional set; add `Hash` only if used as a map/set key; treat `Default` as a deliberate, documented API commitment, not a reflex | High |

**Overall:** The research **confirms the enum-of-modes plan** on every axis. No contradictions. Three refinements to fold into the F2 spec (see Risks/caveats).

---

## Q1 — Bool-flags vs enum-of-modes for mutually-exclusive state — CONFIRMS

**Finding:** Multiple authoritative and community sources converge on the same rule: once boolean fields/flags become mutually exclusive along a single axis, the idiomatic Rust move is to collapse them into a dedicated enum so that illegal combinations cannot be represented. The current `grouping=true && collapse=true` state is precisely the "illegal state" these sources tell you to eliminate.

- **Rust API Guidelines — Type Safety (C-NEWTYPE / "use a deliberate type"):** advises using "a deliberate type (whether enum, struct, or tuple) to convey interpretation and invariants." A bool-pair that admits illegal combinations is the canonical anti-example. — https://rust-lang.github.io/api-guidelines/type-safety.html
- **Alexis King, "Parse, don't validate":** "use a data structure that makes illegal states unrepresentable" and explicitly warns: "avoid the temptation to just stick a `Bool` in a record somewhere because it's needed by the function you're currently writing." — https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/
- **Matthias Endler (corrode.dev), "Make Illegal States Unrepresentable":** Rust-specific application — model the domain with self-contained custom types so invalid states cannot be constructed. — https://corrode.dev/blog/illegal-state/
- **The Rust Book, ch. 6 "Enums and Pattern Matching":** enums define a type "by enumerating its possible variants"; exhaustive `match` forces every case to be handled — the ergonomic backbone that makes the enum refactor a net simplification, replacing nested `if/else if` bool dispatch with a `match`. — https://doc.rust-lang.org/book/ch06-00-enums.html
- **Rust for Rustaceans (Jon Gjengset):** type-driven design / reifying domain concepts as enums and newtypes rather than loose primitives. — https://rust-for-rustaceans.com
- **LanceDB AGENTS.md (real-world Rust style rule):** verbatim "Replace mutually exclusive boolean flags with a single enum/mode parameter." — https://github.com/lancedb/lance/blob/main/AGENTS.md
- **Rust Users Forum, "One case of making illegal states unrepresentable":** community instinctively recommends an enum when ~3 bools encode a small state machine — directly analogous to the #62 case. — https://users.rust-lang.org/t/one-case-of-making-illegal-states-unrepresentable/11715

**When does a builder beat a plain enum here?** It does NOT, for this case. Builders (and typestate) are warranted when construction is *multi-step and user-driven* with *cross-field invariants* (e.g., "if mode A then field X required, field Y forbidden") or many optional/interacting parameters. Sources frame the builder pattern as a finite-state-machine front-end on top of enums, not a replacement for them.
  - Rust API Guidelines builder section: builders are for "incrementally configuring a `T`" with many options. — https://rust-lang.github.io/api-guidelines/type-safety.html
  - Nicolas Fränkel, "Making illegal state unrepresentable" (ITNEXT): "the Builder pattern is a finite state machine"; combine builder + typestate for sequenced construction. — https://itnext.io/making-illegal-state-unrepresentable-fc0299945cf1
  - Typestate threads: powerful but heavier; reserved for protocols/lifecycles where misuse is catastrophic. — https://internals.rust-lang.org/t/type-state-pattern-powered-by-enum-options/21055 , https://users.rust-lang.org/t/using-phantomdata-with-the-type-state-builder-pattern/99087

`TerminalReporter` configuration is a one-shot mode selection (pick exactly one of Grouped / FlatCollapsed / FlatExpanded). There is no multi-step protocol, no cross-field dependency between the render axis and the orthogonal toggles. A plain enum is the correct, minimal tool; a builder or typestate would be over-engineering. **The plan is idiomatic.**

---

## Q2 — Refactor threshold ("bool-parameter smell acute around 3–4") — CONFIRMS (with refinement)

**Finding:** The "acute around 3–4" heuristic is well-supported and is *machine-enforced* in Rust tooling, but it is a heuristic and is better stated per-axis than as a raw count.

- **Clippy `fn_params_excessive_bools` (machine-enforced, primary-source verified):** config option `max-fn-params-bools`, **Default Value: `3`**. The lint fires when a function has *more than 3* bool parameters. — https://doc.rust-lang.org/clippy/lint_configuration.html (verified directly, 2026-06-17)
- **Martin Fowler, "FlagArgument":** a flag argument "tells the function to carry out a different operation depending on its value" — and the advice is to avoid them, preferring distinct methods. — https://martinfowler.com/bliki/FlagArgument.html
- **Steve Smith (Ardalis), "Are Boolean Flags on Methods a Code Smell?":** quantifies the combinatorial explosion — 1 bool → 2 states, 3 bools → 8 states, many nonsensical. — https://ardalis.com/are-boolean-flags-on-methods-a-code-smell/
- **Mark Story / Clean Code, "The argument for flag arguments":** flag args break single-responsibility; refactor into named methods/types. — http://mark-story.com/posts/view/the-argument-for-flag-arguments

**Refinement for F2:** Two nuances the original #62 deferral note should absorb:
1. The Clippy threshold is a *function-parameter* lint, not a struct-field lint. It is the closest tool-enforced numeric anchor (`>3`), and its reasoning transfers to fields, but cite it as *analogous*, not as a direct field rule.
2. The smell here is not raw count — it is **non-orthogonality**. Two mutually-exclusive bools (`show_mitre_grouping`, `collapse_findings`) are already a smell because they encode one axis with `2^2 = 4` representable states, of which only 3 are legal and 1 (`grouping && collapse`) is nonsensical. The refactor is justified by the *illegal-state* argument (Q1) even before the count argument. The count heuristic reinforces; the illegal-state argument is decisive.

---

## Q3 — Orthogonal-field placement (keep `use_color`/`show_hosts_breakdown` as bools) — CONFIRMS

**Finding:** Keeping genuinely-orthogonal booleans as separate, clearly-named fields while extracting only the mutually-exclusive axis into an enum is exactly the recommended hybrid. "Make illegal states unrepresentable" does **not** mean "ban all bools."

The sources explicitly distinguish three roles:
- **Orthogonal bools** — independent properties, all `2^n` combinations valid (e.g., colored output may be on/off independently of hosts breakdown). Bools are appropriate; no illegal states are created.
- **Mutually-exclusive modes** — exactly one of N alternatives → enum.
- **Combinable flag sets** — arbitrary meaningful subsets → bitflags / `bitflags` crate (NOT an enum).

Supporting evidence:
- Rust API Guidelines C-NEWTYPE/type-safety: deliberate types when invariants exist; bools are fine when no invariant is violated. — https://rust-lang.github.io/api-guidelines/type-safety.html
- "Parse, don't validate" (the temptation it warns against is bools that *create* illegal states, not orthogonal toggles). — https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/
- HN discussion on Rust enums vs bitflags: enums are for closed mutually-exclusive sets, not arbitrary combinations — confirming that orthogonal/combinable toggles should NOT be forced into the enum. — https://news.ycombinator.com/item?id=24748202

`use_color` and `show_hosts_breakdown` are genuinely orthogonal to the findings render mode and to each other — every combination is meaningful. Leaving them as bools is correct. **Do not** model them differently; doing so would add complexity without removing any illegal state. The plan's hybrid split is the textbook recommendation.

---

## Q4 — Semver impact: public struct field replacement for a 0.x crate — CONFIRMS

**Finding (primary-source verified):** Removing, renaming, or replacing a public struct field is a **major (breaking)** change under Cargo's SemVer model and RFC 1105. For a `0.y.z` crate the **`y` component is the breaking component**, so `0.8.x → 0.9.0` is the correct and *required* bump.

- **Cargo SemVer Compatibility reference (verified directly, 2026-06-17):**
  - "The absence of a publicly exposed item will cause any uses of that item to fail to compile" — classified under *Major: renaming/moving/removing any public items*.
  - Even *adding* a public field to an all-public-fields struct is a major breaking change (breaks struct-literal construction) — so replacing the field set is unambiguously major.
  - 0.y.z rule, quoted verbatim: *"Initial development releases starting with `0.y.z` can treat changes in `y` as a major release, and `z` as a minor release. `0.0.z` releases are always major changes. This is because Cargo uses the convention that only changes in the left-most non-zero component are considered incompatible."*
  - — https://doc.rust-lang.org/cargo/reference/semver.html
- **RFC 1105 "API Evolution":** classifies removal / incompatible type-change of reachable public fields as major breaking changes (not minor-tolerable breakage). — https://rust-lang.github.io/rfcs/1105-api-evolution.html
- **RFC 0001 "private fields":** marking a field `pub` is an opt-out of encapsulation that makes the field part of the external interface — hence any change to it is a compatibility commitment. — https://rust-lang.github.io/rfcs/0001-private-fields.html
- **predrag (cargo-semver-checks author), "Some Rust breaking changes don't require a major version":** distinguishes RFC-1105 "major" changes (must bump) from tolerated minor breakage; field removal is in the must-bump class. — https://predr.ag/blog/some-rust-breaking-changes-do-not-require-major-version/
- **cargo-semver-checks tooling:** encodes this as lints (`struct_field_missing` → "pub struct field removed", default `required-update = "major"`; analogous `enum_variant_missing`). Renaming = remove + add; type change = remove + add-incompatible. — https://github.com/obi1kenobi/cargo-semver-checks , https://github.com/obi1kenobi/cargo-semver-checks/blob/main/src/lints/enum_variant_missing.ron , https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/
- **Caret/resolver mechanics:** `"0.8.2"` resolves to `>=0.8.2, <0.9.0`, so consumers pinned in the 0.8.x line will NOT auto-receive 0.9.0 — exactly the desired containment for a breaking change. — https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html , https://doc.rust-lang.org/cargo/reference/resolver.html

**Semver verdict: `0.8.x → 0.9.0` is correct and mandatory.** Replacing `show_mitre_grouping` + `collapse_findings` with `render: FindingsRender` removes public fields and adds one — a major breaking change. A patch/minor bump (0.8.x → 0.8.x+1) would violate Cargo's compatibility contract. (Note: this is breaking regardless of the enum choice — even keeping bools but renaming them would be breaking.)

---

## Q5 — Derives for the small mode enum — CONFIRMS

**Finding:** `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` is the conventional set for a small unit-variant mode enum. Add `Hash` only if used as a map/set key; treat `Default` as a deliberate API commitment.

- **Rust API Guidelines C-DEBUG:** "all public types implement `Debug`" and the Debug representation is never empty. `Debug` is effectively mandatory for a public type. — https://rust-lang.github.io/api-guidelines/debuggability.html
- **Rust API Guidelines C-COMMON-TRAITS (interoperability):** eagerly implement `Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default` *where they make sense*. — https://rust-lang.github.io/api-guidelines/interoperability.html (referenced via the guidelines set)
- For a small all-unit-variant enum: `Clone` + `Copy` are free and harmless (no resources, cheap to copy; `Copy` implies `Clone`). `PartialEq` + `Eq` are natural (modes get compared). Clippy `derive_partial_eq_without_eq` nudges you to add `Eq` whenever you add `PartialEq` and total equality holds — which it does here.
- **`Hash`:** add ONLY if `FindingsRender` will be used as a `HashMap`/`HashSet` key. Backwards-compatible to add later, so omitting it now is safe.
- **`Default`:** RFC 3107 enables `#[derive(Default)]` + `#[default]` on a unit variant. Add it only if there is a clear, documented default mode (e.g., if `FlatExpanded` is the canonical default). Changing the default later is a *behavioral* breaking change not caught by the compiler or cargo-semver-checks — so make it deliberately, not reflexively. — https://rust-lang.github.io/rfcs/3107-derive-default-enum.html
- **`PartialOrd`/`Ord`:** avoid unless an ordering among modes is part of the documented contract; derived order follows declaration order and is usually semantically meaningless for modes.

**Recommendation for F2:** ship `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. Add `Hash` iff a keyed-collection use site exists. Decide `Default` explicitly: if the current default render is well-defined (e.g. the existing default when all bools were false), deriving `Default` with that variant marked `#[default]` is reasonable and improves ergonomics — but document it as a stability commitment.

---

## Risks / Caveats for the F2 Spec Author

1. **Semver bump is mandatory and must be in the PR.** This is a public-API breaking change; the release must go `0.8.x → 0.9.0`. Update `Cargo.toml`, CHANGELOG, and any version references. Do NOT ship as a patch/minor. (Verified against the Cargo SemVer reference, not just synthesized.) Consider running `cargo-semver-checks` against the 0.8.x baseline in the release flow to make the breaking-change classification machine-visible (`struct_field_missing` lint will fire as expected — that is correct, not a defect).

2. **The breaking change is unavoidable regardless of design.** Even a minimal rename of the two bools would be breaking. So the enum refactor incurs no *additional* semver cost beyond what any field change already requires — there is no "less breaking" alternative worth trading correctness for. Bundle any other planned `TerminalReporter` field changes into the same 0.9.0 release to amortize the break.

3. **Migration ergonomics.** Because the fields are `pub` and presumably constructed via struct literals downstream (and in wirerust's own tests/CLI wiring), every construction site changes. Provide a clear migration note mapping old bool combinations to the new enum:
   - `show_mitre_grouping=true` → `render: FindingsRender::Grouped`
   - `collapse_findings=true` (grouping false) → `render: FindingsRender::FlatCollapsed`
   - both false → `render: FindingsRender::FlatExpanded`
   - the previously-nonsensical `grouping=true && collapse=true` is now *unrepresentable* — call this out as the win, and audit any current code that could have produced it to confirm behavior is preserved (likely the `if grouping {} else if collapse {}` order meant grouping won; verify the enum mapping matches that precedence).

4. **`Default` decision is a real API choice.** If you derive `Default`, pick the variant that matches today's "all bools false" behavior and mark it `#[default]`. Document it. Changing it post-0.9.0 is a silent behavioral break.

5. **Heuristic, not law (Q2).** The "3–4 bool" threshold is a heuristic; the *decisive* justification in the F2 rationale should be illegal-state-elimination (the `grouping && collapse` contradiction), with the Clippy `max-fn-params-bools = 3` default cited as corroborating tooling consensus, not as the primary driver.

6. **Keep the orthogonal bools as bools (Q3).** Resist any reviewer suggestion to also "enum-ify" `use_color`/`show_hosts_breakdown` — they are orthogonal, all combinations are valid, and wrapping them would add complexity without removing any illegal state. (If `use_color` ever grows a third state like `Auto`, *then* it becomes a `ColorChoice { Auto, Always, Never }` enum — but that is a separate, future axis, not part of #62.)

7. **Inconclusive / lower-confidence items:** None of the five questions are inconclusive. The only soft spot is that several supporting citations are community/blog sources (corrode.dev, predr.ag, ardalis.com, ITNEXT, forum threads) rather than normative spec — but the *load-bearing* claims (Clippy default, Cargo semver field rule + 0.y.z rule) were verified directly against primary sources and are High confidence.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (a) Rust type-driven design: enums vs builder vs typestate for mutually-exclusive bools, flag-argument smell, orthogonal-flag placement; (b) Cargo/RFC 1105 semver for public struct fields & enums, 0.x conventions, cargo-semver-checks, conventional derives. Both run at `reasoning_effort: high`. |
| WebFetch | 2 | Direct primary-source verification of the two load-bearing claims: Clippy `fn_params_excessive_bools` default (`max-fn-params-bools = 3`) and Cargo SemVer reference (public-field removal = major; 0.y.z y-as-breaking-component). |
| Grep / Read (local) | 4 | Reading the large MCP result files (saved to disk due to token cap) and extracting citation URLs. |
| Training data | 0 areas | No claim rests on training data alone; every claim is sourced to web findings, and the two decisive claims were independently re-verified against rust-lang.org primary docs. |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated primary tool)
**Training data reliance:** low — all findings are web-sourced with URLs; the two highest-stakes claims (Clippy threshold, Cargo semver) were cross-verified directly against official rust-lang.org documentation.

### Primary / authoritative sources cited
- Cargo SemVer Compatibility — https://doc.rust-lang.org/cargo/reference/semver.html (verified)
- Cargo Specifying Dependencies — https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
- Cargo Dependency Resolution — https://doc.rust-lang.org/cargo/reference/resolver.html
- RFC 1105 API Evolution — https://rust-lang.github.io/rfcs/1105-api-evolution.html
- RFC 0001 Private Fields — https://rust-lang.github.io/rfcs/0001-private-fields.html
- RFC 3107 Derive Default Enum — https://rust-lang.github.io/rfcs/3107-derive-default-enum.html
- Rust API Guidelines (Type Safety / Debuggability / Interoperability) — https://rust-lang.github.io/api-guidelines/type-safety.html , https://rust-lang.github.io/api-guidelines/debuggability.html
- The Rust Book ch.6 — https://doc.rust-lang.org/book/ch06-00-enums.html
- Clippy lint configuration — https://doc.rust-lang.org/clippy/lint_configuration.html (verified)

### Secondary / community sources cited
- Alexis King, Parse don't validate — https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/
- Matthias Endler, Make Illegal States Unrepresentable — https://corrode.dev/blog/illegal-state/
- Amos, Aiming for correctness with types — https://fasterthanli.me/articles/aiming-for-correctness-with-types
- Rust for Rustaceans — https://rust-for-rustaceans.com
- Martin Fowler, FlagArgument — https://martinfowler.com/bliki/FlagArgument.html
- Steve Smith, Boolean Flags code smell — https://ardalis.com/are-boolean-flags-on-methods-a-code-smell/
- Mark Story, The argument for flag arguments — http://mark-story.com/posts/view/the-argument-for-flag-arguments
- Nicolas Fränkel, Making illegal state unrepresentable — https://itnext.io/making-illegal-state-unrepresentable-fc0299945cf1
- predrag, Rust breaking changes / semver tooling — https://predr.ag/blog/some-rust-breaking-changes-do-not-require-major-version/ , https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/
- cargo-semver-checks — https://github.com/obi1kenobi/cargo-semver-checks
- LanceDB AGENTS.md — https://github.com/lancedb/lance/blob/main/AGENTS.md
- Rust forum / internals threads — https://users.rust-lang.org/t/one-case-of-making-illegal-states-unrepresentable/11715 , https://internals.rust-lang.org/t/type-state-pattern-powered-by-enum-options/21055 , https://users.rust-lang.org/t/using-phantomdata-with-the-type-state-builder-pattern/99087
