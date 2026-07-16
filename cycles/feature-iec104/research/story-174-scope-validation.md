---
document_type: research-validation
producer: research-agent
date: 2026-07-16
policy: DF-VALIDATION-001
subject: STORY-174 scope validation for three deferred carry-forward findings
source_files:
  - .factory/stories/STORY-174.md
  - .factory/STATE.md
  - .github/workflows/ci.yml
  - bin/check-green-doc-tense
  - tests/iec104_analyzer_tests.rs
  - src/analyzer/iec104.rs
  - src/findings.rs
findings_validated:
  - PG-REDGREEN-COMMENT-CLEANUP
  - F-172-003
  - IEC104-FINDING-DIRECTION-001
---

# STORY-174 Scope Validation — Three Deferred Carry-Forwards

DF-VALIDATION-001 validation of three findings that STATE.md routes into STORY-174
(IEC-104 formal-hardening wave) but which the current STORY-174 spec does not yet
contain as acceptance criteria. Each finding is assessed for (a) technical validity,
(b) whether STORY-174 is the right vehicle, and (c) concrete AC language if it belongs.

STORY-174 is a **formal-hardening** story: Kani proofs, proptest, cargo-fuzz, and
cargo-mutants against `src/analyzer/iec104.rs`. Its spec states explicitly
(STORY-174.md:201–202, ADR-013 Architecture Compliance Rules):

> "No new production code changes are expected in this story. If a bug is found, fix
> it in a micro-commit before the proof re-run."

This constraint is the pivotal lens for Finding 3 in particular.

---

## FINDING 1 — PG-REDGREEN-COMMENT-CLEANUP grep-guard

**Claim:** Stub-era Red-Gate comments ("MUST FAIL", "todo!() stub") survived into
GREEN/delivered code across STORY-167/169/170/171/173 (5 confirmed occurrences,
CODIFY-NOW). Proposed codification: a CI or pre-commit grep guard that FAILS if an
implemented function or test module contains stale Red-Gate phrases.

### VERDICT: VALID-INCLUDE (with a material correction to the proposed mechanism)

The underlying problem is real and confirmed in the live tree. **But the proposal as
worded — "add a CI or pre-commit grep guard" — is partially redundant: such a guard
already exists.** The correct codification is to **extend the existing gate**, not to
build a new one.

### Evidence — the guard already exists

`.github/workflows/ci.yml` already contains a `green-doc-tense-gate` job
(ci.yml:451–462, DF-GREEN-DOC-TENSE-SWEEP) that runs `python3 bin/check-green-doc-tense`
on every CI run, plus a self-test (`bin/test_check_green_doc_tense.py`). This is exactly
the "lint-by-grep for stale Red-Gate comments" mechanism the finding proposes. It was
introduced precisely because "this pattern recurred on all 4 stories of
feature-enip-v0.11.0" (ci.yml:434–450).

`bin/check-green-doc-tense` is a mature implementation: 22 curated regex tokens
(bin/check-green-doc-tense:139–284), comment-line anchoring to avoid string-literal
false positives (`_is_comment_line`, :287–289), a tracked-files-only scan via
`git ls-files` so newly-added files do not false-fail before commit (:292–326), and a
documented allowlist philosophy distinguishing current-state assertions from past-tense
provenance narration (:61–121).

### Evidence — why the 5 occurrences slipped through (the actual gap)

The gate has **false negatives** against the exact phrasings IEC-104 stories used. The
still-present baseline stale headers (the PG-REDGREEN-SIBLING-SWEEP targets) are:

- `tests/iec104_analyzer_tests.rs:662-663` — `// All tests in this module MUST FAIL (Red Gate) because classify_frame_format` / `// and process_u_frame are todo!() stubs. They pass after implementation.`
- `tests/iec104_analyzer_tests.rs:1498` — `// Per AC-168-009: this skeleton compiles and FAILS Red Gate because classify_frame_format`
- `tests/iec104_analyzer_tests.rs:1544` — `// All tests in this module MUST FAIL (Red Gate) because parse_asdu is todo!().`

These are NOT caught because the token regexes require exact adjacency. Pattern 1 is
`re.compile(r"All tests MUST FAIL", re.IGNORECASE)` (:141) but the live text is
`All tests in this module MUST FAIL` — the interposed "in this module" defeats the
match. Pattern 3 (`All tests in this file are designed to FAIL`, :149) uses "in this
file"/"are designed to FAIL", not "in this module"/"MUST FAIL". "FAILS Red Gate"
(:1498) and "are todo!() stubs" (:663) have no corresponding token at all. So CI is
currently GREEN despite the stale headers — which is exactly why STORY-167..173 all
merged with green CI and the findings had to be caught reactively by adversarial review.

### Research — external grounding (house-style match)

Lint-by-grep CI guards are an established pattern; ripgrep/grep jobs that fail a build
on forbidden phrases are widely used [simonw/til grep-tests; nerdhut.de "fail CI on
TODO", 2025; ianlewis/todos]. False-positive management via directory/file-type
exclusions and allowlists is the standard mitigation [tutanota test.yml; ianlewis/todos].
The Rust-native alternative to comment-grep for banning `todo!()`/`unimplemented!()` in
shipped code is Clippy's `clippy::todo` lint and `disallowed_macros`
[rust-clippy#9260; rust-clippy#12254; cloudfunnels/rust-guardian]. Note, however, that
`clippy::todo` only catches the actual `todo!()` **macro**, not stale **comment prose**
about a todo!() that has since been removed — which is the wirerust problem class — so
the existing comment-scanning gate remains the right tool here; a Clippy lint would not
subsume it. On the process question (bundle into a hardening change vs standalone chore
PR), public evidence is thin/under-documented — no strong external norm either way.

This matches wirerust house style exactly: the repo already prefers Python self-tested
grep gates (`green-doc-tense-gate`, `action-pin-gate`, `changelog-gate`,
`trust-boundary`, `help-provenance-gate`) over inline clippy config, and the existing
`bin/check-green-doc-tense` already implements the allowlist discipline the research
recommends.

### Is STORY-174 the right vehicle?

Yes, acceptably — with the mechanism corrected. STORY-174 already modifies
`tests/iec104_analyzer_tests.rs` (to add targeted mutant-killing tests per AC-174-007),
so scrubbing the three baseline stale headers in that same file is a natural, in-scope
edit. Extending `bin/check-green-doc-tense`'s token list + self-test is a small,
well-bounded CI-tooling change appropriate to a "hardening" wave. Caveat: touching
`bin/` trips the `changelog-gate` (ci.yml:499–539, trigger set includes `bin/`), so the
story must add a CHANGELOG `[Unreleased]` entry. The scrub is NOT a production-code
change (tests/ + bin/ only), so it does not violate the STORY-174 "no new production
code" constraint.

### Recommended AC language (new AC for STORY-174)

> **AC-174-008: Stale Red-Gate comment guard extended and baseline scrubbed**
> **Traces to:** PG-REDGREEN-COMMENT-CLEANUP (5 occurrences, CODIFY-NOW);
> PG-REDGREEN-SIBLING-SWEEP
> - Given the existing `green-doc-tense-gate` (DF-GREEN-DOC-TENSE-SWEEP) missed the
>   IEC-104 phrasings "All tests in this module MUST FAIL", "…FAILS Red Gate", and
>   "…are todo!() stubs"
> - When `bin/check-green-doc-tense` token list is extended with case-insensitive
>   patterns for: (a) "All tests … MUST FAIL" with arbitrary interposed words
>   (e.g. `All tests\b.*\bMUST FAIL`), (b) `FAILS?\s+Red Gate`, and
>   (c) `are\s+todo!\(\)\s+stub`
> - And corresponding known-bad + known-good fixtures are added to
>   `bin/test_check_green_doc_tense.py` (the self-test MUST pass, proving no regression
>   against existing allowlisted past-tense prose)
> - And the three baseline stale headers at `tests/iec104_analyzer_tests.rs`
>   ~L662-663, ~L1498, ~L1544 are scrubbed to GREEN-accurate prose
> - Then `python3 bin/check-green-doc-tense` and `python3 bin/test_check_green_doc_tense.py`
>   both exit 0, and a CHANGELOG `[Unreleased]` entry records the gate extension
> - Note: this is NOT a new CI job; it extends the existing green-doc-tense-gate. Do not
>   duplicate the guard.

---

## FINDING 2 — F-172-003: VP-045 proptest vacuity

**Claim:** The VP-045 proptest skeletons (`proptest_vp045_direction_isolation`,
`proptest_vp045_independent_run_equivalence`) use proptest framework calls without
meaningful domain generators; the carrier loop covers no meaningful shrinkage paths,
making the properties vacuously true (LOW, deferred from STORY-172 Pass-1).

### VERDICT: VALID-INCLUDE

Confirmed vacuous by direct inspection. AC-174-002 already schedules VP-045 to green,
but as written both harnesses would pass **trivially** and verify none of the properties
AC-174-002 claims. AC-174-002 needs explicit non-vacuity language.

### Evidence — the harnesses assert nothing

`tests/iec104_analyzer_tests.rs:5336–5394`. Both `proptest!` blocks call `on_data(...)`
and then **stop** — there is no `prop_assert!`, `prop_assert_eq!`, or `assert!` anywhere
in either body. The comments concede this outright:

- :5361 — `// (STORY-174 wires the isolation assertion; this skeleton verifies compile.)`
- :5390 — `// (STORY-174 wires the equivalence assertion; this skeleton verifies compile.)`
- :5344-5345 — `/// Full proptest execution is in STORY-174. This skeleton establishes the harness seam and verifies compilation (AC-172-007).`

A proptest `#[test]` with no assertion passes for every generated input by definition
(the only failure mode left is a panic inside `on_data`, which merely re-tests the fuzz
property, not the directional-isolation or equivalence properties). This is the textbook
"vacuously true" anti-pattern: the property provides no oracle, so shrinkage has nothing
to shrink toward [proptest book; docs.rs proptest `prop_assert`; typeable.io PBT].

### Evidence — the generators also lack the required structure

Beyond the missing assertions, the strategies do not generate the interleaving the
property requires. `direction_isolation` (:5349-5352) draws two independent
fixed-role vectors (`c2s_data`, `s2c_data`) and delivers them in fixed order — one full
C2S delivery, then one full S2C delivery (:5362-5363). There is no interleaving of the
two directions and no arbitrary chunking of a single logical stream into multiple
`on_data` calls. Directional carry-buffer isolation is only meaningfully exercised when
C2S and S2C chunks are **interleaved** with arbitrary boundaries so that a
cross-direction leak would actually manifest. `independent_run_equivalence`
(:5379-5393) drives two analyzers identically but never compares their resulting carry
state, so it cannot detect divergence.

### Research — proptest best practice for stateful/directional properties

For interleaved operation sequences, the idiomatic approach is a custom `Strategy` that
generates a `Vec` of tagged operations (e.g. an enum `Op { C2S(Vec<u8>), S2C(Vec<u8>) }`)
and replays them in generated order, rather than two independent fixed-order vectors
[proptest book "enums" tutorial; docs.rs Strategy trait]. Arbitrary chunk boundaries are
modeled by generating a `Vec<Vec<u8>>` (a stream split into arbitrarily-sized chunks) and
feeding each chunk as a separate `on_data` call. Reference-model equivalence (the
"independent run" property) is the canonical use case for state-machine/model-based
testing — `proptest-state-machine` and `readysettech/proptest-stateful` provide a
`ReferenceStateMachine` + `StateMachineTest` harness where the property asserts the SUT
matches a simple reference model after each command
[proptest-rs state-machine.html; blog.nikosbaxevanis.com 2025-01-10; readysettech/proptest-stateful].
Even without adopting those crates, the minimum bar is a `prop_assert!`/`prop_assert_eq!`
that inspects post-`on_data` carry state.

### Is STORY-174 the right vehicle?

Yes — this is squarely a formal-hardening task (proptest execution to green) and
STORY-174 already owns VP-045 via AC-174-002 and modifies the test file. Wiring real
assertions and richer generators is test-only work; it does not touch production code.

### Recommended AC amendment (strengthen AC-174-002)

Amend AC-174-002 to add non-vacuity requirements:

> **AC-174-002 (amended): VP-045 proptest passes — carry direction isolation (non-vacuous)**
> - The two skeletons MUST be upgraded from compile-only seams to asserting harnesses.
>   Each body MUST contain at least one `prop_assert!`/`prop_assert_eq!` that inspects
>   post-`on_data` state; a body that only calls `on_data` without asserting is REJECTED
>   as vacuous.
> - `direction_isolation`: the generator MUST produce **interleaved** C2S/S2C delivery
>   sequences with **arbitrary chunk boundaries** (e.g. a generated `Vec` of
>   direction-tagged byte chunks replayed in generated order), not two fixed-order
>   vectors. The property MUST assert that `carry_c2s` contains only bytes routed via a
>   C2S delivery and `carry_s2c` only S2C bytes (no cross-direction mixing;
>   BC-2.19.025 invariant 1), and that each carry stays ≤ 255 (MAX_IEC104_CARRY_BYTES).
> - `independent_run_equivalence`: the property MUST `prop_assert_eq!` the resulting
>   per-flow carry state (and/or `frame_count`) of the two independent analyzer
>   instances (BC-2.19.025 invariant 2) — currently it compares nothing.
> - Reviewer check: confirm the strategies exercise interleaving and chunk-splitting;
>   a mutation to `on_data`'s direction dispatch MUST cause at least one proptest case
>   to fail (ties to AC-174-007 cargo-mutants — a vacuous property kills no mutants).

---

## FINDING 3 — IEC104-FINDING-DIRECTION-001

**Claim:** `track_ns_desync` leaves `Finding.direction = None` although direction is
known (it formats direction into the evidence string instead). `Finding.direction:
Option<Direction>` exists for JSON consumers (code-quality MINOR, from pr-review-171).

### VERDICT: VALID-DEFER (defer OUT of STORY-174; route to a maintenance/follow-on touch)

The finding is technically valid and well-founded, but STORY-174 (formal hardening) is
the **wrong vehicle**. Populating the field is a production-code change that alters JSON
output, which directly conflicts with STORY-174's "no new production code changes"
constraint and triggers the output-format-change holdout + CHANGELOG obligations.

### Evidence — the finding is accurate

`src/analyzer/iec104.rs:988–1053`. `track_ns_desync` receives `direction: Direction`
(:991), uses it to select the directional field (:997–1000), and formats it into the
evidence vector at :1041 (`format!("direction={direction:?}")`) — yet sets
`direction: None` on the emitted `Finding` at :1046, with the inline comment
"enriched in STORY-173" (:1025) that was never actioned. So the direction is
demonstrably known at the emit site but is dropped from the structured field. The same
`direction: None` pattern recurs across the other IEC-104 emit sites:
`process_u_frame` (:388, :424) and `detect_iec104_threats` (:759, :800, :815, :845,
:895), plus `on_data` inline emits (:1193, :1262). `detect_iec104_threats`/
`process_u_frame` do not currently receive a direction parameter, so populating those
would require threading direction from the `on_data` dispatch site (a wider change).

### Evidence — this IS a JSON output change (not a pure internal cleanup)

`src/findings.rs:163–164`: `#[serde(skip_serializing_if = "Option::is_none")] pub
direction: Option<Direction>`. Because the field is skipped when `None`, IEC-104
findings currently emit **no** `direction` key. Populating it would **add a new
`direction` key** to those findings' JSON. That is an additive output-format change,
observable by any JSON consumer and by holdout/demo-evidence fixtures that assert exact
finding structure.

Populating direction is the established house pattern for JSON consumers — TLS
(src/analyzer/tls.rs:559,592,613,660,683,746,769), Modbus
(src/analyzer/modbus.rs, 9 sites), HTTP (src/analyzer/http.rs, 9 sites), and the
reassembly layer (src/reassembly/mod.rs:533,573,599) all set `direction: Some(...)`.
So the fix is correct and consistent with the codebase; the only question is the vehicle.

### Why STORY-174 is the wrong vehicle

1. **Direct constraint conflict.** STORY-174.md:201–202 and ADR-013 Decision guidance:
   "No new production code changes are expected in this story." Populating direction is
   net-new production behavior in `src/analyzer/iec104.rs`, not a bug-fix forced by a
   failing proof/fuzz/mutant. It does not arise from any AC-174-001..007 harness.
2. **Output-format / holdout obligation.** Per the breaking-change delivery protocol
   (`.factory/maintenance/breaking-change-delivery-protocol.md`, PG-W72-BREAKING-HOLDOUT-SWEEP),
   output-format-change stories require `holdout-expectations-sweep: COMPLETE` before
   PR. STORY-174 has no such sweep scoped and is not framed as an output-format story.
3. **CHANGELOG.** The change touches `src/` and alters user-visible JSON, so it needs an
   `[Unreleased]` CHANGELOG entry describing the new field — again out of character for a
   pure re-verification wave.
4. **Scope creep risk to mutation score.** Adding an emit-site field mid-hardening would
   change the very module cargo-mutants (AC-174-007) is measuring, muddying the sweep.

### Recommendation

Route IEC104-FINDING-DIRECTION-001 to a **dedicated maintenance touch or a small
follow-on story** (analogous to the LOW#1/LOW#2 counter-enrichment burst done for
STORY-173), NOT into STORY-174. That vehicle should:
- Populate `direction: Some(direction)` in `track_ns_desync` (:1046) and evaluate
  threading direction into `process_u_frame` / `detect_iec104_threats` emit sites so
  IEC-104 findings match the TLS/Modbus/HTTP house pattern;
- Add a CHANGELOG `[Unreleased]` entry noting the additive `direction` JSON key for
  IEC-104 findings;
- Run `holdout-expectations-sweep` per PG-W72-BREAKING-HOLDOUT-SWEEP and refresh any
  IEC-104 demo-evidence/holdout fixtures that assert finding JSON shape;
- Optionally drop the now-redundant `format!("direction={direction:?}")` evidence line
  (:1041) once the structured field carries the same information.

If the orchestrator nonetheless wants it in STORY-174, the story's "no new production
code" clause and the holdout-sweep obligation MUST first be explicitly amended and a
holdout sweep scoped — otherwise it breaches the wave's own contract.

---

## Summary Table

| # | Finding | Verdict | One-line rationale |
|---|---------|---------|--------------------|
| 1 | PG-REDGREEN-COMMENT-CLEANUP grep-guard | **VALID-INCLUDE** (mechanism corrected) | Problem is real, but a grep guard already exists (`green-doc-tense-gate`/`bin/check-green-doc-tense`); codify by EXTENDING its token list + self-test and scrubbing the 3 baseline headers (tests/ + bin/ only — no production code), add CHANGELOG entry. |
| 2 | F-172-003 VP-045 proptest vacuity | **VALID-INCLUDE** | Confirmed vacuous — both harnesses call `on_data` with zero `prop_assert!` and no interleaving/chunking; AC-174-002 must add explicit non-vacuity + interleaved-generator + state-comparison language. |
| 3 | IEC104-FINDING-DIRECTION-001 | **VALID-DEFER** (out of STORY-174) | Finding is valid, but populating `Finding.direction` is production code that adds a `direction` key to JSON (skip_serializing_if) — conflicts with STORY-174's "no new production code" clause and triggers holdout-sweep/CHANGELOG obligations; route to a maintenance/follow-on touch. |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) proptest best practice for stateful/directional/buffer properties, interleaved generators, vacuity anti-pattern, state-machine reference-model testing (Finding 2). (2) lint-by-grep CI guard patterns, false-positive/allowlist management, clippy::todo / disallowed_macros alternatives, bundle-vs-chore process norms (Finding 1). |
| Context7 | 0 | Not needed — proptest/clippy behavior grounded via web sources + in-repo usage. |
| Read | 3 | STORY-174.md, STATE.md, .github/workflows/ci.yml, plus bin/check-green-doc-tense and iec104.rs slices. |
| Grep | 6 | Locate VP-045 harnesses, stale Red-Gate headers, track_ns_desync/direction sites, Finding.direction serde attrs, cross-analyzer direction:Some usage. |
| Training data | 2 areas | Rust serde `skip_serializing_if` semantics (verified against src/findings.rs:163) and general TDD Red-Gate context — flagged; both cross-checked against live code. |

**Total MCP tool calls:** 2 (both `perplexity_research`, reasoning_effort=medium)
**Training data reliance:** low — every technical claim is anchored to either a
file:line in the live tree or an external source; serde/proptest/clippy claims were
cross-verified against in-repo code and web citations.

### Key external sources
- proptest book (altsysrq.github.io/proptest-book), enums tutorial; docs.rs proptest `prop_assert`, `Strategy` trait
- proptest state-machine testing (proptest-rs.github.io state-machine.html); readysettech/proptest-stateful; blog.nikosbaxevanis.com 2025-01-10
- typeable.io PBT overview; proofsandintuitions.net PBT specifications (2026-05)
- Lint-by-grep CI: simonw/til grep-tests; nerdhut.de "fail CI on TODO" (2025); ianlewis/todos; tutanota .github/workflows/test.yml
- Rust macro-ban alternatives: rust-clippy#9260, rust-clippy#12254 (clippy::todo / disallowed_macros); cloudfunnels/rust-guardian
