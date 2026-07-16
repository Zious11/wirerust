---
document_type: research-validation
producer: research-agent
date: 2026-07-16
policy: DF-VALIDATION-001
subject: STORY-174 scope-validation FOLLOW-UP — deferral vehicle + VERDICT-1/VERDICT-2 residual-risk deep dive
supersedes: none (supplements story-174-scope-validation.md)
source_files:
  - .factory/cycles/feature-iec104/research/story-174-scope-validation.md
  - .factory/maintenance/breaking-change-delivery-protocol.md
  - .factory/STATE.md
  - .factory/cycles/feature-enip-v0.11.0/decisions-archive.md
  - bin/check-green-doc-tense
  - src/analyzer/iec104.rs
  - src/analyzer/dnp3.rs
  - src/analyzer/enip.rs
  - tests/iec104_analyzer_tests.rs
  - tests/enip_analyzer_tests.rs
findings_validated:
  - IEC104-FINDING-DIRECTION-001 (deferral vehicle)
  - PG-REDGREEN-COMMENT-CLEANUP (VERDICT-1 residual risk)
  - F-172-003 / VP-045 vacuity (VERDICT-2 residual risk)
---

# STORY-174 Scope Validation — FOLLOW-UP

Supplements `story-174-scope-validation.md`. The prior report's three verdicts stand
unchanged. This follow-up answers the three deeper questions the human raised before
approving the STORY-174 realignment: (1) the OPTIMAL deferral vehicle for
IEC104-FINDING-DIRECTION-001; (2) the false-positive residual risk of the VERDICT-1
green-doc-tense token extension against the current tree; (3) implementability of the
VERDICT-2 VP-045 non-vacuity amendment against the current test-visible API.

---

## Q1 — Deferral vehicle for IEC104-FINDING-DIRECTION-001

### RECOMMENDATION: Option (a) — a small **pre-F5 fix-PR** inside feature-iec104, delivered on `develop` after STORY-174/wave-83 merges but BEFORE F5 scoped-adversarial begins. Deliver it via the `fix-pr-delivery` skill (fix-PR, not a new story), NOT in STORY-174, NOT deferred to maintenance.

Confidence: HIGH.

### The three options, weighed

**Where the fix lands relative to the review perimeter is the deciding axis.** Per
`feature-mode-scoping-rules` (SKILL.md:37, Rule 2 at :49-54), F5 adversarial scope is
`Delta = NEW + MODIFIED + DEPENDENT` files, and `src/analyzer/iec104.rs` is already a
MODIFIED delta file for the whole feature-iec104 cycle. A fix delivered pre-F5 therefore
rides the existing F5 perimeter for free — the adversary reviews the full modified file,
so the new `direction` emit sites get scoped adversarial review as part of the IEC-104
delta. Options (b) and (c) both land the change OUTSIDE that perimeter.

| Option | Perimeter | PG-W72 sweep | STORY-174 clause | Precedent | Net |
|--------|-----------|--------------|------------------|-----------|-----|
| (a) pre-F5 fix-PR in feature-iec104 | INSIDE F5–F7 delta review (iec104.rs already MODIFIED) | cheap & near-empty (see below) | no conflict — not in STORY-174 | ENIP D-262 fix-PR #331; STORY-173 LOW-burst | **RECOMMENDED** |
| (b) next maintenance run (bundled w/ ROUTE-BC-DEFER, PERF-RERUN-001, SEC-001) | OUTSIDE feature review; gets only maintenance-sweep scrutiny | same sweep, but detached from the IEC-104 delta it belongs to | n/a | maintenance carry-forward pattern | rejected — strips feature-scoped adversarial review |
| (c) cycle-close target-TBD | OUTSIDE F5; unbounded target | deferred indefinitely | n/a | E-11 cycle-close backlog | rejected — fix is small & ready; no reason to defer to an indefinite target |

### Does an ADDITIVE `direction` key legally trigger PG-W72-BREAKING-HOLDOUT-SWEEP?

The protocol's Scope Trigger 2 (breaking-change-delivery-protocol.md:37-39) reads
"Observable JSON output schema change: the story changes field names, field types, enum
values, enum casing … or adds/removes a structural envelope." An additive **optional**
key (a field that today serializes to nothing via
`#[serde(skip_serializing_if = "Option::is_none")]`, src/findings.rs:163, and would
begin appearing on IEC-104 findings) is **not literally any of the enumerated examples**
(no rename, no type change, no enum casing, no envelope add/remove). But the trigger
*header* — "Observable JSON output schema change" — is broad, and the added key IS
observable. **Conservative reading: treat it as in-scope and run the sweep.** That costs
almost nothing here, so there is no upside to arguing it out of scope.

**Why the sweep is near-empty (this is what makes option (a) cheap):**

1. **No IEC-104 holdout scenarios exist.** A content scan of `.factory/holdout-scenarios/`
   for `iec104|iec-104|2404` returns zero matches; the only "104" filename is
   `HS-104-pcapng-…` (a pcapng scenario, unrelated to the IEC-104 protocol). There are no
   IEC-104 finding-shape expectations to go stale.
2. **Holdout assertions are "contains"/subset, not exact-match.** E.g. HS-071
   (`.factory/holdout-scenarios/HS-071-tls-server-hello-version-tracking.md:65-70`) uses
   "Assert `findings` contains …" / "detail … contains BOTH …". An additive key does not
   remove or alter any asserted content, so subset assertions survive additive fields.
3. **Additive optional field = backward-compatible by schema-evolution norm.** Adding an
   optional field that was previously omitted is treated as a non-breaking, additive
   change under JSON Schema / Avro / Protobuf "never remove, only add" compatibility rules
   and semver-for-APIs guidance; it is breaking ONLY for consumers doing strict/exact-shape
   validation (`additionalProperties:false` or exact deserialization). Tolerant/subset
   consumers — which is what the holdout harness is — are unaffected [Perplexity research,
   JSON/serde schema-evolution norms, 2026-07-16].

So the PG-W72 obligation is satisfiable in any vehicle and is not a reason to push the fix
out of the cycle. The prudent residual step is a quick demo-evidence check: refresh any
IEC-104 demo-evidence fixture that asserts exact finding JSON shape (there is no evidence
one exists, but the fix-PR should confirm). The `fix-pr-delivery` skill keeps "same rigor
as story PRs" (security + AI review), so the sweep steps ride it cleanly.

### House precedent for a post-F4 / pre-F5 production-code fix

- **ENIP D-262 (decisions-archive.md:432-434, PR #331):** "Pre-F5 cleanup fix-PR #331
  (`refactor(enip): wire summarize through EnipSummary + doc fixes`) squash-merged into
  develop … F4 (TDD implementation) COMPLETE. Entering F5 scoped-adversarial refinement."
  Delivered as a **fix-PR with no new story** ("stories_delivered=87 (refactor, no new
  story)"), at exactly the F4→F5 boundary, and it was careful to be **byte-identical
  output**. This is the closest structural precedent and directly supports option (a)'s
  vehicle — a fix-PR, not a story.
- **STORY-173 pre-merge LOW-fix burst (STATE.md:120, :183):** LOW#1 `flows_analyzed` real
  cumulative counter + LOW#2 `packets_analyzed` frame counter — both **production-code
  observability enrichments to summarize() output** — landed as micro-commits
  (0bfc977/5325cf2) pre-merge inside the story, human-approved, triggering fresh A/B/C
  re-convergence. Shows the house comfortably delivers small production-code JSON/summary
  enrichments inside the feature perimeter with re-review.

The direction fix is the same shape as both precedents: a small, additive, house-pattern
enrichment (`direction: Some(direction)` to match TLS/Modbus/HTTP/DNP3/ENIP emit sites)
that belongs inside the feature perimeter and benefits from F5 adversarial eyes.

### Note (minor correction to prior report)

The inline "enriched in STORY-173" comment at `src/analyzer/iec104.rs:1025` actually
annotates `source_ip`/`timestamp` ("source_ip and timestamp left None — enriched in
STORY-173"), NOT `direction`. The `direction: None` at :1046 carries no explanatory
comment. This does not change the verdict — direction is still demonstrably known at the
emit site (param at :991, formatted into evidence at :1041) and dropped from the
structured field at :1046 — but the fix-PR should not rely on that comment as its marker.

---

## Q2 — VERDICT-1 residual risk (green-doc-tense token extension)

### RESULT: Negligible / effectively-zero false-positive risk against the current tree. NO allowlist/exclusion amendment for `.factory/` or `docs/` is needed. Pattern (a)'s unbounded `.*` cannot span a line.

Confidence: HIGH.

### Every hit of the three proposed patterns, tool-exact semantics

Patterns run comment-line-anchored (stripped line starts with `//`), case-insensitive,
over ALL git-tracked files, replicating `check-green-doc-tense` scan_file semantics:
- (a) `All tests\b.*\bMUST FAIL`
- (b) `FAILS?\s+Red Gate`
- (c) `are\s+todo!\(\)\s+stub`

| Pattern | Hit | In tool scan set? | Classification |
|---------|-----|-------------------|----------------|
| (a) | `tests/iec104_analyzer_tests.rs:662` | YES (tests/*.rs) | **TRUE POSITIVE** (baseline stale header) |
| (a) | `tests/iec104_analyzer_tests.rs:1544` | YES | **TRUE POSITIVE** (baseline stale header) |
| (b) | `tests/iec104_analyzer_tests.rs:1498` | YES | **TRUE POSITIVE** (baseline stale header) |
| (c) | `tests/iec104_analyzer_tests.rs:663` | YES | **TRUE POSITIVE** (baseline stale header) |
| (a) | `bin/check-green-doc-tense:15` | **NO** — bin/, no `.rs` ext | would-be FP only if scan set broadened (it is not) |
| (a) | `bin/test_check_green_doc_tense.py:55` | **NO** — `.py`, bin/ | self-test fixture; not scanned |
| (a) | `bin/test_check_green_doc_tense.py:61` | **NO** | self-test fixture; not scanned |
| (a) | `bin/test_check_green_doc_tense.py:73` | **NO** | self-test fixture; not scanned |

**Within the tool's actual scan set, the ONLY matches are the four lines that make up the
three known baseline stale headers — all true positives, zero false positives.** These are
exactly the PG-REDGREEN-SIBLING-SWEEP targets the extension is meant to catch. The four
`bin/` hits are the tool's own module-docstring example (`check-green-doc-tense:15`) and
its self-test fixtures (`test_check_green_doc_tense.py`); they match the regex text but are
**not files the gate scans**, so they are not false positives in practice.

### Scan-set file-selection logic (why `.factory/`, `docs/`, and `bin/` are already excluded)

`_collect_rust_files` (bin/check-green-doc-tense:292-326) runs
`git ls-files -- "tests/*.rs" "src/**/*.rs"` (:303-308) and keeps only tracked `.rs`
files under those two globs, with a SEC-001 repo-root-escape guard (:319-325). Therefore:
- `.factory/` and `docs/` are **structurally outside the scan set** — no `.rs` files are
  collected from them. An empirical scan of tracked `docs/` and `.factory/` files for all
  three patterns returns **zero hits** anyway, so even a hypothetical scope broadening would
  find nothing there today.
- `bin/` is outside the two globs, and `check-green-doc-tense`/`test_check_green_doc_tense.py`
  are not `.rs` files regardless.

**No allowlist/exclusion amendment is required.** The existing tracked-`.rs`-only,
tests/+src/-only scan set already isolates the gate from provenance prose in `docs/` and
`.factory/` narratives and from the tool's own fixtures.

### Pattern (a)'s `.*` cannot span a line

`scan_file` iterates `text.splitlines()` and calls `pattern.search(stripped)` on each
single physical line (bin/check-green-doc-tense:342-349). Each element passed to
`re.search` is one line with no embedded newline, and `.` does not match `\n` without
`re.DOTALL` (not set). So pattern (a)'s unbounded `.*` is confined to a single physical
line. This is demonstrated by the two-line header at :662-663: it is matched as TWO
independent lines (:662 by pattern (a), :663 by pattern (c)), never as one `.*` span
across the newline. Confirmed — no cross-line over-matching.

### Two minor implementation notes (non-blocking)

1. Pattern (a) `All tests\b.*\bMUST FAIL` **subsumes** existing token 1
   (`All tests MUST FAIL`, :141): with no interposed words, `.*` matches the single space.
   Adding (a) alongside token 1 is harmless (scan_file breaks after the first match per
   line, :349); the extension may either add (a) or replace token 1 with (a). Cosmetic.
2. When `test_check_green_doc_tense.py` gains known-bad/known-good fixtures for the three
   new patterns, those fixtures live inside the `.py` file (not `.rs`, not under tests/ or
   src/) and so cannot self-trigger the production gate. No risk.

---

## Q3 — VERDICT-2 residual risk (VP-045 non-vacuity amendment)

### RESULT: Fully implementable against the CURRENT public API with ZERO new production code. Per-flow `carry_c2s`/`carry_s2c`/`frame_count` are already public and already observed by the existing IEC-104 integration tests. No test-only accessor is needed, so the amendment does NOT violate STORY-174's no-new-production-code clause.

Confidence: HIGH.

### The state is public today

- `Iec104FlowState` — `pub struct` (src/analyzer/iec104.rs:214) with `pub carry_c2s: Vec<u8>`
  (:217), `pub carry_s2c: Vec<u8>` (:220), `pub frame_count: u64` (:259).
- `Iec104Analyzer` — `pub struct` (:1074) with
  `pub flows: HashMap<FlowKey, Iec104FlowState>` (:1076). The field doc even states
  "Tests inspect this field …" (:1078).
- `pub fn on_data(...)` (:1145) and `pub fn summarize(...)` (:1345) are the public drivers
  a proptest replays against.

### The existing IEC-104 tests already observe exactly this state

`tests/iec104_analyzer_tests.rs` reads per-flow carry state through the public `flows` map
in 56 places, e.g.:
- `let state = analyzer.flows.get(&flow_key).unwrap();` then asserts `state.carry_c2s` /
  `state.carry_s2c` (:4491-4498, :4515-4522, :4544-4551).
- residual-tail assertions on `state.carry_c2s.len()` / `state.carry_s2c.len()`
  (:4612-4616, :4664-4672, :4749-4757).
- tests even **write** state directly for injection:
  `analyzer.flows.entry(flow_key).or_default(); state.carry_c2s = overflow_carry;`
  (:4798-4799).

So the VERDICT-2 recommendation — an interleaved direction-tagged-chunk generator replayed
via `on_data`, plus `prop_assert!/prop_assert_eq!` on post-`on_data` `carry_c2s`/`carry_s2c`
isolation, the ≤255 bound, and `frame_count` equivalence across two independent analyzer
runs — is directly expressible with the API as it stands. All of it is test-only code in
`tests/iec104_analyzer_tests.rs`.

### No test-only accessor needed → no clause conflict, and no such precedent exists

The question of "what minimal `#[cfg(test)]`/`#[doc(hidden)]` accessor would be needed" is
**moot: none is needed**, because the fields are `pub` by design. This is the uniform
house pattern across all three stream analyzers, so there is no DNP3/ENIP precedent of
adding a test-only accessor to observe carry state:
- **DNP3:** `Dnp3FlowState` `pub carry_c2s` (src/analyzer/dnp3.rs:212), `pub carry_s2c`
  (:216); `Dnp3Analyzer` `pub flows` (:294).
- **ENIP:** `EnipFlowState` `pub carry_c2s` (src/analyzer/enip.rs:324), `pub carry_s2c`
  (:331); `EnipAnalyzer` `pub flows` (:615). The ENIP test file documents the pattern
  explicitly — "flow state via `analyzer.flows.get(&key).expect(...)`"
  (tests/enip_analyzer_tests.rs:4743-4745) — and uses it in `flow.carry_c2s` assertions
  (:4841-4845, :4871-4875, :4960-4970, :5010-5015). The only `#[cfg(test)]` marker in
  enip.rs is a test-module declaration (:2073), not a state accessor.

Because STORY-110 (DNP3) and STORY-132 (ENIP) hardening observed carry/flow state through
these same public fields, the IEC-104 VP-045 amendment stays on the established path: no
new production surface, no accessor, no `no-new-production-code` conflict.

---

## Summary Table

| Q | Question | Answer | Confidence |
|---|----------|--------|-----------|
| 1 | Optimal deferral vehicle for IEC104-FINDING-DIRECTION-001 | **Option (a): a small pre-F5 fix-PR inside feature-iec104 (via `fix-pr-delivery`, not a new story, not in STORY-174), delivered after wave-83 but before F5.** Keeps the additive `direction` key inside the F5–F7 delta-review perimeter (iec104.rs is already MODIFIED); matches ENIP D-262 pre-F5 fix-PR #331 and STORY-173 LOW-burst precedent; PG-W72 sweep is near-empty (no IEC-104 holdout scenarios; contains-style assertions; additive optional field is backward-compatible), so the sweep is cheap and not a reason to push it out of cycle. (b)/(c) rejected: both land outside F5 and detach the fix from its delta. | HIGH |
| 2 | VERDICT-1 false-positive risk of the three token patterns vs current tree | **Negligible.** In the tool's actual scan set (`git ls-files -- tests/*.rs src/**/*.rs`) the only matches are the 4 baseline-header lines (662/663/1498/1544) — all true positives, zero false positives. The 4 `bin/` hits are the tool's own docstring/self-test fixtures and are not scanned (bin/, non-`.rs`). Scan set excludes `.factory/`/`docs/`/`bin/` by construction (and they have zero hits anyway) → **no allowlist/exclusion amendment needed**. Pattern (a)'s `.*` operates per-line (splitlines + no DOTALL) → **cannot span a line**. | HIGH |
| 3 | VERDICT-2 implementable vs current test-visible API? | **Yes, with zero new production code.** `Iec104Analyzer.flows` (pub, iec104.rs:1076) and `Iec104FlowState.carry_c2s/carry_s2c/frame_count` (pub, :217/:220/:259) are already public and already observed by the IEC-104 tests via `analyzer.flows.get(&key)` (56 uses). No `#[cfg(test)]`/`#[doc(hidden)]` accessor needed → **no conflict with STORY-174's no-new-production-code clause**. Identical pub-field pattern in DNP3 (dnp3.rs:212/216/294) and ENIP (enip.rs:324/331/615); no house precedent of a test-only accessor because none is required. | HIGH |

---

## Research Methods

| Tool | Calls | Purpose |
|------|-------|---------|
| Read | 5 | prior report, breaking-change protocol, STORY-174.md, bin/check-green-doc-tense, iec104.rs slices |
| Bash/Grep + Python (tool-exact regex replay) | ~8 | enumerate the three token patterns over all tracked files with comment-line anchoring; scan-set analysis; holdout-scenario existence + match-mode; ENIP pre-F5 precedent; DNP3/ENIP pub-field + accessor check; 56 carry-state observation sites |
| feature-mode-scoping-rules SKILL.md | 1 | F5 delta-review perimeter definition (Rule 2, Scope-by-Phase table) |
| Perplexity perplexity_research | 1 | serde/JSON additive-optional-field backward-compatibility norm; strict-vs-subset consumer risk (reasoning_effort=low) |

**Total MCP tool calls:** 1 (perplexity_research). Every technical claim is anchored to a
`file:line` in the live tree; the single external call grounds only the additive-field
compatibility norm used in Q1.
