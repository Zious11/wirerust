# F2 Decomposition & Sequencing — Breaking Type Change (`mitre_techniques: Vec<String>`) + Modbus Analyzer

**Type:** general (process / engineering-method)
**Date:** 2026-06-09
**Status:** complete
**Purpose:** Evidence-backed recommendation for how to sequence, into TDD-driven per-story PRs, (a) a cross-cutting breaking type change to `Finding` and (b) the new Modbus TCP analyzer that motivates it — keeping the full regression suite green throughout.
**Related prior research:** `.factory/research/modbus-tcp-research.md`, `.factory/research/modbus-f2-design-decisions.md`.

> **Confidence legend:** [VERIFIED] = stated in a cited authoritative source. [INFERRED] = reasoned synthesis across sources. [JUDGMENT] = design judgment where evidence under-determines the answer (but is constrained by the cited principles).

---

## 0. TL;DR — the decisive recommendation

1. **Do the type change as a single ATOMIC, behavior-preserving migration commit/PR — NOT three-phase Parallel Change.** The compiler forces every call site to change together (a `struct` field type change is a hard error at every use site), there are zero external consumers, and the crate is pre-1.0. The entire raison d'être of expand-contract — letting some consumers stay on the old shape while others move — does not apply. [VERIFIED principle + JUDGMENT on application]

2. **The type migration is its own isolated wave, merged GREEN, BEFORE the Modbus analyzer is built on it.** This is the textbook application of "make the change easy, then make the easy change" (Beck/Fowler preparatory refactoring): the `Option<String>` shape makes the multi-technique feature impossible, so you reshape first, then the feature "just slips in." [VERIFIED]

3. **NOT interleaved.** Do not fold the type change into the Modbus feature PR. Refactor hat and feature hat are separate commits/PRs (Fowler "two hats"). [VERIFIED]

4. **Update the 6 existing stories' tests INSIDE the migration commit, as a behavior-PRESERVING test refactor** — `Some("T1234")` → `vec!["T1234".into()]`, `None` → `vec![]`. The *asserted output* (JSON/CSV/terminal bytes) stays identical; only the test's internal *construction* of the expected `Finding` changes. That is a refactor of test setup, not a behavior change, so it belongs in the same green commit — not a separate pre-step. [VERIFIED distinction + JUDGMENT]

5. **Dependency graph:** `[Wave 1: type + catalog + reporters + existing analyzers + their tests, atomic & green]` → `[Wave 2: Modbus analyzer + multi-technique display + new RED-first tests]`. Within Wave 1 the internal edit order is irrelevant (the commit is only green/red as a whole). Across waves, Wave 2 strictly depends on Wave 1; any *other* unrelated work can parallelize on trunk once Wave 1 lands.

---

## 1. The concrete surface (verified against the codebase)

I read the actual code. The migration surface is small and entirely internal:

| Site class | File(s) | Count / shape |
|---|---|---|
| The type itself | `src/findings.rs:135` | `pub mitre_technique: Option<String>` + the `#[serde(skip_serializing_if = "Option::is_none")]` attribute |
| Emission sites (set the field) | `src/analyzer/http.rs` (8), `src/analyzer/tls.rs` (7), `src/reassembly/mod.rs` (4) + `lifecycle.rs` (2) | ~21 `mitre_technique: Some(..)/None` literals |
| Reporter consumers (read the field) | `src/reporter/csv.rs:82`, `src/reporter/terminal.rs:232/246/265`, `src/reporter/json.rs` (via derive + skip attr) | 3 reporters |
| Catalog lookup | `src/mitre.rs` `technique_info`/`technique_tactic` (consumes the ID string, not the field) | unchanged signature; only the *call shape* in terminal.rs changes from `Option` to iterating a `Vec` |
| Tests asserting single-technique output | `tests/{http_analyzer,tls_analyzer,reporter_json,reporter_csv,reporter_terminal,mitre,findings}_tests.rs` + the 6 shipped stories' tests | mechanical `Some/None` → `vec![..]/vec![]` |

**Behavior-preservation insight [VERIFIED against the serializers]:**
- JSON: `#[serde(skip_serializing_if = "Option::is_none")]` on `Option<String>` omits the key when `None`. The vec equivalent is `#[serde(skip_serializing_if = "Vec::is_empty")]`, which omits the key when `vec![]`. A singleton vec `vec!["T1234"]` serializes as a JSON **array** `["T1234"]`, **NOT** the old scalar `"T1234"` — *this is a real shape change for JSON and must be a conscious decision* (see §3.1 caveat).
- CSV (`csv.rs:82`): `f.mitre_technique.as_deref().unwrap_or("")` → join the vec with a chosen separator; for a singleton vec the cell is byte-identical to before.
- Terminal (`terminal.rs:232/246/265`): `if let Some(ref t)` → iterate the vec; for a singleton vec the rendered line is byte-identical.

This is why the migration is genuinely behavior-preserving **for the singleton case** and the existing tests can stay green with only setup edits — with the one JSON-array caveat flagged in §3.1.

---

## 2. Pattern selection: atomic rename, NOT three-phase Parallel Change

### 2.1 What the literature says Parallel Change is *for*

Fowler's **Parallel Change / expand–migrate–contract** (martinfowler.com/bliki/ParallelChange.html) exists to roll out a **backward-incompatible** interface change **without a red window for consumers who cannot all be changed at once** — multiple services, external clients, independent deploy schedules, "flag day" avoidance. [VERIFIED] The expand phase adds the new form *alongside* the old; migrate moves consumers gradually; contract removes the old. Its headline benefit per Fowler: "the code can be released in any of the three phases." [VERIFIED]

The same pattern at schema level is **expand/contract** (Sohoni, DevOpsDays 2019; Confluent Schema Registry): add new column/field additively, dual-write, migrate readers, drop the old. The Schema Registry docs even prescribe upgrade *order* by compatibility mode (backward → upgrade consumers first; forward → producers first; full → any order). [VERIFIED]

### 2.2 Why it does **not** apply here

Every precondition that makes Parallel Change *worth its overhead* is **absent** in this case:

| Parallel-Change precondition | Present here? |
|---|---|
| Consumers that can't all change at once | **No** — Rust compiler forces all ~32 sites to change in lockstep; you literally cannot commit a half-migrated state that compiles. |
| External API stability guarantee | **No** — pre-1.0 (0.x). SemVer itself says 0.y.z means "anything may change at any time." [VERIFIED, semver.org §4] |
| Independent deploy schedules / multiple services | **No** — single crate, single binary, single team. |
| Long migrate phase needed | **No** — one IDE/compiler-guided pass. |

Both the deep-research synthesis and the focused reasoning pass converge **decisively**: with the compiler enforcing atomicity and no external consumers, **adding a second parallel field (`mitre_technique` + `mitre_techniques`) would add transitional duplication, a dual-field-sync hazard, and extra test surface for ZERO compatibility benefit.** [VERIFIED reasoning] The canonical sources don't address the pre-1.0-internal-type case explicitly (they flag this gap), but the *principles* are unambiguous: the value of expand-contract scales with fan-out × runtime-risk × inability-to-coordinate-consumers, and here the last factor is *forced to zero by the type system*.

**RECOMMENDATION — pattern:** **Single atomic, behavior-preserving type-migration commit.** Change the field, all ~32 sites, the three reporters, and the affected tests in one commit; the suite stays green; push. The only "red" is on your local working tree before the commit — never on trunk. [VERIFIED: trunk-based "every pushed commit compiles & is green"; the in-progress local state is irrelevant to trunk.]

> Branch-by-abstraction and Strangler Fig are likewise **overkill** here — they target subsystem/library replacement and system-boundary modernization respectively, not a single internal value type. [VERIFIED scope distinction, Confluent/Fowler.]

---

## 3. Sequencing: type migration FIRST, as its own wave (not interleaved)

### 3.1 "Make the change easy, then make the easy change" — applied

This is the single most load-bearing principle for the question. Kent Beck's maxim ("for each desired change, make the change easy — *warning: this may be hard* — then make the easy change") and Fowler's **preparatory refactoring** essay (martinfowler.com/articles/preparatory-refactoring-example.html) are *exactly* this situation. [VERIFIED]

- The `Option<String>` shape **cannot represent** Modbus's `[T0855, T0836]` multi-technique findings — it makes the feature impossible.
- Reshaping to `Vec<String>` is the "make the change easy" step — a **behavior-preserving** enabling refactor.
- The Modbus analyzer is then the "easy change" built on the ready shape.

Fowler/Beck and the trunk-based + CD literature all support doing the **enabling refactor as a distinct, behavior-preserving unit of work, merged green, before the dependent feature.** [VERIFIED] The "two hats" metaphor (Fowler, *Refactoring* 2nd ed.) makes the separation a rule: a commit either restructures (tests stay green) **or** changes behavior (tests go red first) — never both. [VERIFIED]

**Interleaving the type change into the Modbus feature PR is explicitly discouraged**, because it conflates the structural change with the behavioral one, defeats the ability to answer "did the migration preserve behavior?" independently of "does Modbus work?", and produces an un-reviewable mega-PR. [VERIFIED via Arcand "Purposeful Commits", Mick "breaking large PRs apart", code-review best-practice sources.]

**CAVEAT — the JSON array shape decision (a genuine behavior change to isolate):** Because `Some("T1234")` serialized as scalar `"mitre_technique":"T1234"` but `vec!["T1234"]` serializes as array `"mitre_techniques":["T1234"]` (and the key *name* changes), the JSON output is **not** byte-identical even for singletons. This is unavoidable and correct (it's the whole point), but it means:
- The JSON reporter's *single-technique* tests **will** change their asserted output — that part is a **behavior change**, not a pure refactor.
- **Handle it inside Wave 1 still** (it is intrinsic to the type migration and the new JSON contract is the *target* contract), but call it out in the PR description as the one place where Wave 1 changes an asserted external representation. CSV and terminal singleton output **do** stay byte-identical and are pure refactors.
- This is consistent with TDD: for the JSON tests you write the new expected array shape first (red), then the serializer change makes it green — all within Wave 1.

### 3.2 The 6 shipped stories' tests — refactor-in-place, not pre-step

The reasoning pass settles this decisively. Distinguish two things the literature treats very differently [VERIFIED, Beck/Fowler/Shore TDD]:

- **Test refactor** = change the test's *structure/setup* while the *asserted behavior* is identical. Stays green. Belongs in the refactor commit.
- **Test migration** = change *what the test asserts*. Goes red, then code makes it green. Is a behavior change.

For STORY-069/070/071/078/079/080's tests asserting single-technique output:
- The change from `Some("Txxxx")` to `vec!["Txxxx".into()]` in the *expected `Finding` construction* is a **test refactor** — the CSV cell / terminal line they assert is unchanged. → **Bundle into the Wave-1 migration commit.** (Option (a).)
- Any of those tests that assert *JSON scalar* shape cross into **test migration** (per §3.1) — update the expectation to the array form, still inside Wave 1.

A *separate* pre-step that edits tests to use a `Vec` field **before** the production type exists is rejected: it would reference a type that doesn't compile yet, creating an artificial red window for no benefit. [VERIFIED reasoning] Production type and its tests change shape **together** in one green commit.

> Net: there is **no** "update the existing stories' tests first" pre-wave. Their test edits are an inseparable, mostly-mechanical part of the atomic migration commit.

---

## 4. Regression-safety & ordering: minimizing the red window

### 4.1 Within the migration commit — order is irrelevant to trunk

Trunk-based development evaluates green/red **only at the commit boundary**: a pushed commit must compile and pass. [VERIFIED] The intermediate states while you edit (type changed but reporters not yet fixed → compile errors everywhere) live **only on your local working tree** and never reach trunk. Therefore **the question "should I change the reporter before the analyzer before the catalog?" has no effect on trunk safety** — they all land in one commit.

For *developer ergonomics* (not correctness), the natural compiler-driven order is:
1. Change the type definition + the serde attribute (`skip_serializing_if = "Vec::is_empty"`).
2. `cargo check` → the compiler hands you the exhaustive list of broken sites.
3. Fix production code: catalog call shape (terminal), then the three reporters, then the ~21 emission sites (mechanical `Some(x)`→`vec![x]`, `None`→`vec![]`).
4. Fix the tests (setup refactor + the JSON-array expectation update).
5. `cargo test --all-targets` + `cargo clippy --all-targets -- -D warnings` green → commit → push.

This is the Rust-idiomatic "lean on the compiler" refactor: the type error list *is* your migration checklist; you cannot forget a site.

### 4.2 If Wave 1 must be split (optional, only if the single PR feels too large)

A ~32-site behavior-preserving change in a single crate is a *normal-sized* atomic refactor and a single PR is fine and preferred. But if review ergonomics demand splitting, the **only** split that keeps every intermediate commit green is **by introducing a temporary internal adapter** — which reintroduces Parallel-Change overhead and is **not recommended here**. The cleaner lever for reviewability is **micro-commits within one PR** (each green is not achievable mid-migration, so instead: one squashable WIP branch, review the final diff). **Recommendation: keep Wave 1 as one atomic commit/PR.** [JUDGMENT]

### 4.3 Across waves — the dependency graph

```
Wave 1  (preparatory refactor — behavior-preserving except the JSON-array contract)
  ├─ findings.rs: Option<String> → Vec<String> (+ serde skip_if = Vec::is_empty)
  ├─ mitre.rs:    no signature change (consumes the ID string); terminal call-shape adapts
  ├─ reporters:   json (array shape), csv (join), terminal (iterate)   ─┐ all in
  ├─ analyzers:   http / tls / reassembly emission sites → singleton vecs │ ONE atomic
  └─ tests:       6 shipped stories + reporter/analyzer tests adapted    ─┘ green commit
        │
        ▼  (strict dependency: Wave 2 needs the Vec field to exist)
Wave 2  (feature — behavior change, RED-first TDD)
  ├─ Modbus module (new): parsing + detection, emits Vec with ≥2 technique IDs
  ├─ dispatcher integration (register Modbus analyzer)
  ├─ reporters: multi-technique DISPLAY (terminal "MITRE:" multi-line; CSV multi-ID cell;
  │             JSON already array-shaped from Wave 1 — no further change)
  ├─ mitre.rs:  seed the new ICS technique IDs (T0836, T0835, T0888, etc.) — see modbus-f2 research
  └─ tests:     ~25 new Modbus behavioral contracts + multi-technique reporter assertions
```

**What parallelizes:**
- **Within Wave 2**, once the Modbus module's `Finding`-emission contract is fixed, the **multi-technique reporter-display work** and the **MITRE catalog seeding** (`technique_info` arms + `SEEDED_TECHNIQUE_IDS` + count, per `mitre.rs`'s drift-guard) can proceed in **parallel** with the detection logic — they share only the new technique-ID *strings*, which can be agreed up front.
- **After Wave 1 merges**, any unrelated feature work can resume on trunk against the new `Vec` field without conflict.
- **Wave 2 cannot start its feature behavior before Wave 1 merges** — that is the one hard ordering edge. (You *can* spike/branch Modbus parsing in parallel, but it can't emit multi-technique findings until the type exists.)

> **MITRE catalog note:** `src/mitre.rs` has a mechanical drift guard (`vp007_catalog_drift_guard`) that fails if `technique_info` arms diverge from `SEEDED_TECHNIQUE_IDS`/`SEEDED_TECHNIQUE_ID_COUNT`. New Modbus IDs must be added to **all three** in lockstep, and the Kani `EMITTED_IDS` list updated. This is **pure-additive catalog work** (no breaking change), so it can land in Wave 2 (or even a tiny Wave 1.5 catalog-seed PR ahead of Modbus, exactly as `mitre.rs`'s "staged entries" doc-comment anticipates).

---

## 5. Recommended story/wave plan (action-ready)

### Wave 1 — `refactor: migrate Finding.mitre_technique → mitre_techniques (Vec)` (one PR)
- **Hat:** refactoring (behavior-preserving, except the deliberate JSON-array contract in §3.1).
- **Content:** type + serde attr; all ~21 emission sites → singleton/empty vecs; csv/terminal readers; json array shape; mechanical test-setup updates across the 6 shipped stories + reporter/analyzer tests; JSON singleton-shape test expectations updated to array form.
- **Gate:** `cargo test --all-targets` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check` green. Suite passes with **no behavior delta** for CSV/terminal; the only asserted-output change is JSON scalar→array, documented in the PR body.
- **Why first:** preparatory refactoring — "make the change easy." Enables everything downstream; reviewable in isolation; safely revertible.

### Wave 1.5 (optional) — `feat(mitre): seed Modbus ICS technique IDs` (tiny additive PR)
- Add T0836/T0835/T0888 (+ any others from `modbus-f2-design-decisions.md`) to `technique_info`, `SEEDED_TECHNIQUE_IDS`, `SEEDED_TECHNIQUE_ID_COUNT`, Kani lists. Pure-additive catalog seeding (the module is explicitly designed for staged entries). Keeps Wave 2's feature PR smaller. Can also live inside Wave 2.

### Wave 2 — `feat(analyzer): Modbus TCP analyzer with multi-technique findings` (one PR, internally TDD)
- **Hat:** feature (behavior change; red-first per technique slice/behavioral contract).
- **Content:** new Modbus module (parse + detect), dispatcher registration, multi-technique reporter *display*, ~25 new behavioral-contract tests + multi-technique reporter assertions.
- **TDD:** each behavioral contract = write the failing test (red) → implement to green → refactor. Never leave trunk red between commits; never mass-edit assertions and leave them unsatisfied.
- **Depends on:** Wave 1 (hard). Internally, detection-logic vs reporter-display vs catalog-seed can parallelize once the technique-ID set and emission contract are fixed.

**Decisive answers to the five posed questions:**
1. **Migration first, isolated wave, then Modbus** — yes. Not interleaved. (Preparatory refactoring + two hats.)
2. **Atomic rename, not expand-contract** — the compiler forces lockstep change and there are no external consumers, so the parallel-field pattern is pure overhead.
3. **Minimal red window:** trunk only ever sees the green atomic commit; intra-commit edit order is irrelevant; cross-wave, refactor precedes feature.
4. **6 stories' tests:** behavior-preserving setup refactor bundled INTO the migration commit (option (a)); the single JSON-array expectation change is the one behavior-change exception, also in Wave 1.
5. **Dependency graph:** type → (reporters ∥ catalog ∥ analyzers, all in Wave 1) → Modbus (Wave 2); within Wave 2, detection ∥ display ∥ catalog-seed parallelize.

---

## 6. Where the literature is explicit vs silent (honesty flags)

- **Explicit / [VERIFIED]:** preparatory refactoring & "make the change easy then make the easy change" (Beck maxim, Fowler essay); two-hats separation of refactor vs feature commits; Parallel Change's purpose and the "releasable in any phase" benefit; trunk-based "every pushed commit green & releasable / don't break the build"; TDD red-green-refactor with refactor = behavior-preserving (tests stay green) and behavior change = test-first red; SemVer 0.y.z "anything may change at any time"; producer→consumer upgrade ordering by compatibility mode (Schema Registry).
- **Silent / [INFERRED–JUDGMENT]:** No canonical source addresses the *exact* case "internal type, pre-1.0, single crate, compiler-enforced atomic call-site update, no external consumers." Both deep-research passes explicitly flag this gap. The recommendation is therefore a *synthesis*: the cited principles (fan-out, runtime risk, inability-to-coordinate-consumers drive Parallel-Change value; here the type system zeroes out the last) point unambiguously to the atomic-refactor-first plan. The decisive reasoning pass concurs.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Parallel Change / expand-contract phases & green-build mechanics, branch-by-abstraction, Strangler Fig, Feathers characterization tests/Sprout-Wrap, CD backward-compatible migrations, SemVer pre-1.0 — with explicit "when is the overhead worth it vs atomic change for an internal pre-1.0 type" framing. (2) Sequencing/decomposing a cross-cutting breaking change + feature into TDD per-PR units: trunk-based small-batch, "make the change easy then make the easy change," TDD stance on migrating tests, producer→consumer ordering, two-hats refactor/feature separation. Both at `reasoning_effort: high`, `strip_thinking: true`. |
| Perplexity perplexity_reason | 1 | Decisive synthesis over the gathered principles against the CONCRETE wirerust facts (compiler-enforced atomicity, singleton-vec behavior-preservation, ~32 sites, no external consumers) — answering the four operative sub-questions (parallel vs atomic; refactor-first wave; where to update the 6 stories' tests; intra/inter-commit ordering). `search_context_size: medium`. |
| Perplexity perplexity_search / ask | 0 | — |
| Context7 | 0 | — (no library-API question; serde behavior verified by reading the crate's own serializers) |
| Tavily | 0 | — |
| WebFetch / WebSearch | 0 | — |
| Read / Grep (codebase) | 6 | Verified the real migration surface: `findings.rs` (the type + serde attr), `mitre.rs` (catalog + drift guard), `csv.rs`/`terminal.rs` reporter read-sites, and the full `grep` of `mitre_technique` across `src/` (21 emission sites + 3 reporters). This grounds the singleton-vec behavior-preservation claim and the JSON-array caveat in the actual code, not assumptions. |
| Training data | 1 area | Rust compiler semantics (struct field type change → exhaustive compile errors at use sites) and serde `skip_serializing_if` behavior — both cross-checked against the crate's own `findings.rs`/`csv.rs`. |

**Total MCP tool calls:** 3 (2 × `perplexity_research` PRIMARY + 1 × `perplexity_reason`).
**Training data reliance:** low — every method claim is anchored to a cited source (Fowler bliki ParallelChange/BranchByAbstraction/StranglerFig/preparatory-refactoring, trunkbaseddevelopment.com, Atlassian TBD, Humble/Farley CD, Beck TDD/Tidy-First, Feathers WELC summaries, semver.org, Confluent Schema Registry, Arcand/Mick/thoughtbot on atomic commits & PR splitting); all wirerust-specific facts verified by reading the code.

### Key sources (verified)
- Fowler — ParallelChange (expand/migrate/contract), BranchByAbstraction, StranglerFigApplication, preparatory-refactoring-example, *Refactoring* "two hats" (martinfowler.com/bliki, /articles).
- Kent Beck — "make the change easy, then make the easy change"; *TDD by Example* red-green-refactor; Tidy First structural-vs-behavioral separation.
- trunkbaseddevelopment.com (Paul Hammant) + branch-by-abstraction page; Atlassian trunk-based-development guide.
- Humble & Farley — *Continuous Delivery*; Fowler *Continuous Integration*.
- Michael Feathers — *Working Effectively with Legacy Code* (characterization tests, seams, Sprout/Wrap) via understandlegacycode.com / khalilstemmler.com summaries.
- semver.org §4 (0.y.z "anything may change at any time").
- Confluent Schema Registry compatibility modes & upgrade ordering; Sohoni DevOpsDays 2019 expand/contract.
- Arcand "Purposeful Commits", Mick "breaking large PRs apart", thoughtbot "splitting commits" (atomic-commit / PR-decomposition discipline).
- wirerust source (verified): `src/findings.rs`, `src/mitre.rs`, `src/reporter/{csv,terminal,json}.rs`, `src/analyzer/{http,tls}.rs`, `src/reassembly/{mod,lifecycle}.rs`.
