# DF-VALIDATION-001 Research Validation — STORY-176 AC-176-001

**Finding under validation:** Pre-implementation spec-flaw finding on STORY-176 AC-176-001
("extend the green-doc-tense gate with bare-word `skeleton`/`seam` tokens").
**Validator:** vsdd-factory:research-agent
**Date:** 2026-07-20
**Scope note:** PG-GATE-VOCAB-BLINDSPOT itself is a feature-iec104 in-process execution
finding and is DF-VALIDATION-001-*exempt* per the in-process exemption (STORY-176 §Notes).
This report does NOT re-litigate whether the blindspot is real (it is). It validates the
**remediation spec** AC-176-001 wrote for it, before remediation routing.

---

## Executive Verdict

AC-176-001 as written is **substantially INVALID** on four independent factual claims. The
underlying motivation (skeleton/seam stale prose slipped past the gate on STORY-174) is
**VALID**, but every concrete instruction the AC gives for fixing it is wrong: wrong file,
wrong mechanism, wrong false-positive assumption, wrong CHANGELOG conclusion. The AC would
not compile into a working change and its `grep -rn "\bskeleton\b\|\bseam\b"` verification
command would emit **~91 legitimate matches**, not the "empty output" it predicts.

**Recommended disposition: (a) spec-route revision of AC-176-001** (see §Task 5).

---

## Task 1 — Local Provenance of the Two STORY-174 Findings

**Verdict: VALID (motivation confirmed) / INCONCLUSIVE (verbatim flagged lines not recoverable). Confidence: HIGH on substance, MEDIUM on exact per-pass attribution.**

The blindspot is corroborated by three independent local artifacts:

1. **`.factory/cycles/feature-iec104/lessons.md:49-52`** (verbatim):
   > **PG-GATE-VOCAB-BLINDSPOT:** The green-doc-tense gate (AC-174-008) missed `"skeleton"` and
   > `"seam"` phrasing — stub-era language surviving into green deliveries. Two independent
   > adversary observations in STORY-174 (P2 Obs-1 + P4 obs). The token list must be extended
   > to cover at least `skeleton`, `seam`, and equivalent stub-era vocabulary.

2. **`.factory/cycles/feature-iec104/STORY-174/convergence-report.md:184-190`** (F-174-002,
   Pass 2, MEDIUM), verbatim:
   > Stale skeleton/false CI-wiring prose: "stub", "skeleton", and "fails until wired"
   > phrasing on fully-wired passing code. 8 sites across 2 source regions.
   > AC-174-008 token list (3 patterns) caught the main instances; "skeleton" and
   > "seam" missed (see PG-GATE-VOCAB-BLINDSPOT). Fix 038286a + 8-site sibling sweep.

3. **`.factory/STATE.md:151`** (D-462): "PG-GATE-VOCAB-BLINDSPOT filed."

**Recoverability of exact flagged lines:** NOT recoverable from the current tree. The lines
were scrubbed by fix commit `038286a` (P2) during STORY-174 delivery; I have no `Bash`/git
access to inspect the pre-scrub blob. The best-recoverable characterization is the F-174-002
text above: present-tense `"stub" / "skeleton" / "fails until wired"` prose on 8 sites across
2 source regions.

**Attribution caveat (source conflict, flagged per mandate):** STORY-176 §Background claims
"P2 Obs-1 (stale skeleton prose in the story spec itself) and P4 finding (stale seam commentary
in test headers)." The convergence report's actual **P4 finding (F-174-P4-001)** is a
*BC-2.19.025 invariant-2 mis-anchor* (convergence-report.md:197-202), **not** seam commentary.
lessons.md hedges as "P4 obs" without detail. So STORY-176's crisp "P4 = stale seam commentary
in test headers" is **not confirmed** by the convergence artifacts — the seam observation is
documented, but its precise pass label is ambiguous across artifacts. This does not affect the
substance (both tokens were flagged by adversaries and missed by the gate).

---

## Task 2 — AC-174-008 Mechanism Check (Is STORY-176's Claim a Fabrication/Drift?)

**Verdict: STORY-176's claims about AC-174-008 are INVALID (drift/fabrication). Confidence: HIGH.**

STORY-176 AC-176-001 makes three claims about what AC-174-008 established. All three are false
against the delivered artifacts:

| STORY-176 claim (AC-176-001) | What AC-174-008 actually established | Verdict |
|---|---|---|
| "the green-doc-tense gate's **grep command in `.github/workflows/ci.yml`**" (b, verification block) | The gate is **`bin/check-green-doc-tense`** — a Python script with a `_VIOLATION_PATTERNS` list of phrase-level regexes matched against **comment lines only**. `ci.yml` (`green-doc-tense-gate` job, `ci.yml:451-462`) merely **invokes** `python3 bin/check-green-doc-tense` and its self-test. There is no grep, and no token list, in `ci.yml`. STORY-174 §File Structure and §Architecture Mapping both name `bin/check-green-doc-tense` as the modify target, never ci.yml. | **INVALID** |
| A "`# green-doc-tense-gate: allow` allowlist mechanism established by AC-174-008" (b, EC-001, EC-002) | **No such mechanism exists.** Grep of the entire tree for `green-doc-tense-gate: allow` → **0 occurrences**. AC-174-008's documented allowlist is **pattern specificity**: `bin/check-green-doc-tense:61-121` states "The allowlist is implemented by the token specificity above" — patterns require unambiguous current-state assertions so past-tense provenance is never matched. There is no inline-annotation opt-out. | **INVALID (fabrication)** |
| "the token list [is] applied to `src/` and `tests/` Rust source files" via new grep patterns `\bskeleton\b`, `\bseam\b` | The scan set is correct (`git ls-files -- tests/*.rs src/**/*.rs`, `check-green-doc-tense:339`), but the matching model is **phrase-level current-state assertions on comment lines**, not **bare word tokens**. AC-174-008 explicitly added *phrase* patterns 23-25 (`All tests\b.*\bMUST FAIL`, `FAILS?\s+Red Gate`, `are\s+todo!\(\)\s+stub`) — never bare words — precisely because bare words over-match. | **INVALID (design mismatch)** |

AC-174-008 also **required a CHANGELOG entry** ("touching `bin/` trips the `changelog-gate`",
STORY-174 AC-174-008 + Tasks + File Structure), directly contradicting AC-176-001(c). See Task 5.

**Conclusion:** STORY-176 AC-176-001 was written against an imagined ci.yml-grep-with-inline-
allowlist gate that does not exist. It is spec drift: the author appears to have generalized
from a mental model of "a grep gate in CI" rather than reading the AC-174-008 delivery.

---

## Task 3 — Pattern Design (Phrase-Level, Zero-False-Positive)

**Verdict: bare-word approach INVALID; refined phrase patterns VALID. Confidence: HIGH (all
candidates grep-verified against the live scan set).**

### Why bare words fail (execution-verified)

`grep -rn "\bseam\b\|\bskeleton\b"` over `*.rs` returns **~78-79 `seam` + 13 `skeleton`** matches,
essentially **all legitimate**. `seam` is a first-class codebase idiom for the
DF-KANI-NONVACUITY-001 **test-seam / verification-seam** pattern, not stub-era vocabulary:

- `src/analyzer/iec104.rs:721,960` — `/// ## VP-047 seam`, `/// ## VP-045 proptest seam`
- `src/dispatcher.rs:561,567,579` — `UDP gap-key seam (VP-043 non-vacuity)`, `The seam pattern mirrors VP-039/VP-040`
- `src/reassembly/lifecycle.rs:281,312` — `Test-only seam to force a flow's state...`
- `tests/reassembly_engine_tests.rs:6044,6215,6728,6803` — `Test seam accessors added in W8.3`
- `tests/iec104_analyzer_tests.rs:3611` — `/// Traces: ... VP-047 seam.`

`skeleton` is used for **past-tense provenance**, never live-stub assertion:

- `src/analyzer/iec104.rs:17,1444` — `error type skeleton (extended in STORY-168)`, `Harness skeleton originated in STORY-167`
- `src/analyzer/dnp3.rs:2110`, `src/analyzer/modbus.rs:1309` — `Harness structure from VP-023/VP-022 proof skeleton`
- `src/analyzer/arp.rs:4617` — `matching the VP-024 Sub-D skeleton's iteration count`

A bare-word gate would fire on all of these. **`seam` in particular should arguably not be
gated at all** — it is legitimate, pervasive architectural vocabulary in this codebase.

### Orchestrator starting-candidate correction

The orchestrator offered `'write seam accessor' = 0` as a safe starting candidate. **Partially
wrong:** `write seam accessor` (exact imperative) is indeed 0, but the looser `seam accessor`
matches **5 legitimate lines** (`tests/reassembly_engine_tests.rs:6041,6044,6215,6728,6803`
"Test seam accessors"). Any pattern must be the tighter form. `harness skeleton compiles` = 0
confirmed (only `skeleton originated` / `proof skeleton` exist).

### Recommended candidate patterns (each grep-verified → 0 matches in scan set)

All are case-insensitive, comment-line-anchored (consistent with the existing engine), and
target **present-tense stub-state assertions**, with past-tense provenance allowlisted by
specificity:

1. `skeleton\s+compiles?\b` — catches "harness skeleton compiles" (stub-era: it only compiles,
   no real proof/assertions). Allowlists `skeleton originated`, `proof skeleton`, `skeleton's
   iteration`, `error type skeleton (extended...)`. **Verified: 0 matches.**
2. `compile-only\s+seams?\b` — catches "compile-only seam(s)" present-tense (the exact
   STORY-172 VP-045 descriptor for a no-assertion harness; stale once upgraded per AC-174-002).
   Allowlists `test seam`, `Test seam accessors`, `VP-047 seam`. **Verified: 0 matches in scan
   set** (the phrase survives only in `.factory/.../red-gate-log.md`, outside the scan set).
3. `(?:are|is)\s+(?:currently\s+)?compile-only\b` — catches "are currently compile-only seams
   with no assertions" style present-tense claims. **Verified: 0 matches.**
4. `\buntil\b[^\n]*\bwired\b` — catches the F-174-002 "fails until wired" CI-wiring prose
   (asserts code is not yet wired). **Verified: 0 matches.**

Each candidate needs a matched **known-bad + known-good fixture pair** in
`bin/test_check_green_doc_tense.py` (structure at `test_check_green_doc_tense.py:51+`,
`BAD_CASES`/known-good lists), exactly as AC-174-008 did for patterns 23-25. Recommend
dropping the bare `seam` token entirely and gating only the narrow `compile-only seam` phrase;
`skeleton` likewise only via `skeleton compiles`.

---

## Task 4 — Upstream Duplicate Check (drbothen/vsdd-factory)

**Verdict: NOT an engine-level duplicate — the lexical token list is correctly product-local.
Confidence: MEDIUM-HIGH.**

- **Issue #682** (WebFetch, verified): title *"Process-gap in TDD workflow where stale RED-gate
  docstrings persist after implementation."* It recommends a workflow step — "sweep RED-gate
  docstrings to past-tense / historical." It uses `RED GATE stub`, `todo!() stub`,
  `unimplemented!() body` — and **does not mention `skeleton` or `seam`**. It is the engine-level
  generalization of the *pre-adversarial doc-sweep behavior* (the AC-176-002-v1.0 concern), NOT
  the lexical token list.
- **Issue #686** (WebFetch, verified): title *"process-gap(adversary+orchestrator): finding
  decay is non-monotone across fresh-context passes."* It concerns convergence-signal
  calibration (N-consecutive-CLEAN vs decreasing counts). **No mention** of skeleton/seam or the
  green-doc-tense gate.
- **`perplexity_research` (sonar-deep-research):** found no engine-level issue that systematically
  gates a lexical vocabulary of stub-era tokens (skeleton/seam/scaffold/stub) in delivered
  comments, and could not surface #682/#686 content directly (the deep-research index lagged the
  live issues, which WebFetch then resolved). It noted adjacent "comment-truth sweep" and
  "story-decomposition seam" discussions but nothing codifying a lexical-gate-coverage mechanism.

**Conclusion:** `bin/check-green-doc-tense` is a **wirerust-product-local tool** (it lives in this
repo's `bin/`, not the engine). The token-list extension is correctly product-local and is NOT
already tracked upstream. The only engine-adjacent item (#682) covers a different layer (a
prompt/workflow doc-sweep step), which is why STORY-176 v2.0 already routed the v1.0 doc-sweep AC
to #682 and kept AC-176-001 local. That routing is coherent; no additional upstream issue is
warranted for the lexical patterns.

---

## Task 5 — Disposition Recommendation

**RECOMMENDED: (a) spec-route revision of AC-176-001. Confidence: HIGH.**

Do **not** drop the AC (option b): the blindspot is real and the fix is cheap and valuable. Do
**not** implement AC-176-001 as written: every concrete instruction is wrong. Revise it to:

1. **Correct the locus.** Target `bin/check-green-doc-tense` (`_VIOLATION_PATTERNS`) +
   `bin/test_check_green_doc_tense.py` (fixtures) — **not** `.github/workflows/ci.yml`. ci.yml is
   already correct and only invokes the tool; it must not be edited (STORY-176's own §Architecture
   Compliance Rules "MUST NOT change any `uses:` SHA pins" further argues against touching ci.yml).
2. **Replace bare-word patterns with phrase-level patterns** (Task 3 candidates 1-4). Bare
   `\bskeleton\b`/`\bseam\b` are rejected: ~91 legitimate matches. Recommend gating only
   `skeleton\s+compiles?` and `compile-only\s+seams?` (+ optionally the `until...wired` phrase);
   recommend **not** gating bare `seam` (legitimate test-seam idiom).
3. **Delete the fabricated allowlist claim.** There is no `# green-doc-tense-gate: allow`
   mechanism. The allowlist is pattern specificity (past-tense forms are simply not matched).
   Rewrite EC-001/EC-002 accordingly.
4. **Correct the CHANGELOG obligation.** `bin/` **is** in the AC-158-001 changelog-gate trigger
   set, so a `bin/check-green-doc-tense` change **requires** an `[Unreleased]` CHANGELOG entry —
   exactly as AC-174-008 itself did. AC-176-001(c)'s "no CHANGELOG entry required" is INVALID for
   the corrected locus. (It was only true under the mistaken ci.yml-only locus.)
5. **Fix the zero-false-positive verification command.** Replace `grep -rn "\bskeleton\b\|\bseam\b"`
   with `python3 bin/check-green-doc-tense` (exit 0) + `python3 bin/test_check_green_doc_tense.py`
   (exit 0), matching AC-174-008's verification model.
6. **Update §Architecture Mapping / §Tasks / §Notes** which currently say "ci.yml (amend)",
   "No `bin/` changes", and "no CHANGELOG entry required" — all three are now wrong.

This makes AC-176-001 a near-exact structural repeat of AC-174-008 (phrase patterns + fixtures +
CHANGELOG), which is the correct precedent the AC's own §Previous Story Intelligence already cites.

### Per-claim verdict summary

| Claim | Verdict | Confidence |
|---|---|---|
| Blindspot is real (skeleton/seam missed on STORY-174) | VALID | HIGH |
| Gate lives in ci.yml grep command | INVALID | HIGH |
| `# green-doc-tense-gate: allow` inline allowlist exists | INVALID (fabrication) | HIGH |
| Bare `\bskeleton\b`/`\bseam\b` yield zero false positives | INVALID (~91 matches) | HIGH |
| No CHANGELOG entry required | INVALID (bin/ is in trigger set) | HIGH |
| Finding is an engine-level upstream duplicate | INVALID (product-local; #682/#686 unrelated) | MEDIUM-HIGH |
| Refined phrase patterns achievable at zero FP | VALID | HIGH |
| STORY-176's "P4 = stale seam commentary in test headers" attribution | INCONCLUSIVE (artifacts disagree) | MEDIUM |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Upstream engine-level dupe sweep for lexical stub-era vocabulary gating in drbothen/vsdd-factory |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — |
| Tavily (all) | 0 | — |
| WebFetch | 2 | Verified scope of issues #682 and #686 |
| WebSearch | 1 | Secondary check for any gate-vocabulary/skeleton-seam upstream issue |
| Local (Read/Grep/Glob) | 12 | STORY-176/174 specs, convergence report, lessons.md, STATE.md, ci.yml, check-green-doc-tense + self-test, tree grep verification of every pattern claim |
| Training data | 0 areas | All claims sourced to files (path:line) or web fetches |

**Total MCP tool calls:** 1 (perplexity_research) + 3 web (2 WebFetch, 1 WebSearch)
**Training data reliance:** low — every verdict is anchored to a file path + line number (local)
or a fetched URL (upstream). The one lower-confidence item (P4 attribution) is explicitly flagged
INCONCLUSIVE due to a source conflict between STORY-176 and the convergence report.

### Sources

- `.factory/stories/STORY-176.md` (AC-176-001, EC-001/002, §Notes, §Architecture Mapping)
- `.factory/stories/STORY-174.md` (AC-174-008, §File Structure, §Architecture Mapping)
- `bin/check-green-doc-tense` (`_VIOLATION_PATTERNS` L139-319; allowlist doc L61-121; scan set L339)
- `bin/test_check_green_doc_tense.py` (fixture structure L51+)
- `.github/workflows/ci.yml` (`green-doc-tense-gate` job L451-462)
- `.factory/cycles/feature-iec104/lessons.md:49-52`
- `.factory/cycles/feature-iec104/STORY-174/convergence-report.md:184-202`
- `.factory/STATE.md:151` (D-462)
- Live tree grep: `\bseam\b` ~78-79 / `\bskeleton\b` 13 / `green-doc-tense-gate: allow` 0; candidate patterns 1-4 all 0
- https://github.com/drbothen/vsdd-factory/issues/682
- https://github.com/drbothen/vsdd-factory/issues/686
