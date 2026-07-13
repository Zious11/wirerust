# DF-VALIDATION-001 Report — wave-75 STORY-165 process-gap candidates

**Validator:** research-agent (fresh context)
**Date:** 2026-07-13
**Scope:** Three deferred `[process-gap]` findings from wave-75 STORY-165 adversarial convergence,
proposed for codification as S-7.02 follow-up items at wave close (Finding 3 added per team-lead follow-up).
**Repo state at validation:** develop HEAD `d6e3be8`; factory-artifacts worktree mounted at `.factory/`.

---

## Summary verdicts

| Finding | Verdict | One-line basis |
|---------|---------|----------------|
| 1 — validate-citations checks line-in-bounds, not symbol-at-line | **VALID** | Tool has no symbol/content assertion; the F-S165P1-001 defect (fabricated test name at an in-bounds line) is direct corroboration. |
| 2 — wave-74 artifacts use dual/colliding finding-ID schemes | **VALID (accurately stated)**, with a **remediation-scope refinement** | Dual scheme confirmed; the F-S165P4-001 defect instantiated exactly this ambiguity. The proposed *remediation framing* ("per-story vs wave-gate") is mis-targeted — the real collision axis is G-less `F-W<NN>P<n>` vs canonical `F-W<NN>G-P<n>`. |
| 3 — wave-74 v3.48 attribution discrepancy | **VALID** | The two records genuinely disagree on what produced STORY-INDEX v3.48; `gate-summary.md:43` is the artifact in error, and this is a concrete instance of Finding 2's root cause. |

All three findings are **sound and still open** on the current repo. None is covered by any
existing story: STORY-165 codifies AC-165-001..004 (bin-selftest CI, PR-desc row-verify,
delivery-doc currency, governance-table audit-first) — none touch citation symbol-resolution
or finding-ID naming (`.factory/stories/STORY-INDEX.md:228`, `:14`). No existing `bin/` tool
performs symbol-at-line checks.

---

## Finding 1 — validate-citations verifies line-in-bounds, not symbol-at-line

### Verdict: VALID

### Evidence — current tool behavior (`bin/validate-citations`)
The tool's entire validation surface is file/line existence. Its own algorithm docstring
enumerates every check (`bin/validate-citations:15-27`): MALFORMED, OUTSIDE REPO, FILE NOT
FOUND, NOT A FILE, UNREADABLE, INVALID LINE, INVALID RANGE, and — the strongest content
check present — `LINE OUT OF RANGE: path:N (file has M lines)` (`:26-27`). The core loop
(`:135-231`) counts lines via `count_lines()` (`:126-128`) and compares the cited number to
the file's line count. There is **no read of the cited line's text and no symbol/name
assertion anywhere** in the file. The citation grammar itself has no symbol field — the regex
is `^\s*([^:]+):(\d+)(?:-(\d+))?\s*(?:#.*)?$` (`:101`), i.e. `path:line[-line]` only.

### Evidence — the gap is not hypothetical
Reproduced against the live tool: a citation to an in-bounds line paired with a fabricated
symbol claim passes preflight:

```
$ printf 'bin/validate-citations:128  # claims def test_fabricated lives here\n' \
    | python3 bin/validate-citations -
PASS: 1 citations verified   (exit 0)
$ sed -n '128p' bin/validate-citations
    return sum(1 for _ in file_path.open("rb"))   # no such symbol on line 128
```

### Evidence — this exact class already shipped a defect (F-S165P1-001)
The wave-75 Pass-1 HIGH defect is a real-world instance: a fabricated test name
`test_T12_malformed_line_counted_in_denominator` was cited at an in-bounds line; ground-truth
is `test_T12_malformed_line_reported` at line 278
(`.factory/stories/STORY-165.md:538`; corrected locus now at `:239`; sibling in
`.factory/maintenance/pr-description-row-verify-mandate.md:156`). The line number was valid,
so a validate-citations preflight would have passed it — the fabrication was in the *symbol
name*, which the tool does not inspect. This is precisely the failure mode the finding
predicts.

### No existing coverage
- No `bin/` tool does symbol-at-line resolution (grep of `bin/` for symbol/ctags/`def test_`
  logic returns nothing outside test files; directory listing shows only
  `changelog-gate-check`, `check-green-doc-tense`, `compute-input-hash`, `fetch-e2e-pcaps`,
  `lint-cycle-artifact`, `validate-citations`).
- STORY-165 does not cover it (ACs are bin-selftest/PR-desc/currency/audit-first only).

### Recommended minimal design
Keep the tool's existing constraint (Python 3 stdlib only, no third-party deps — `bin/validate-citations:3`, mirroring `compute-input-hash`). **Reject ctags/universal-ctags** for an MVP: it adds an external binary dependency and a tag-DB build step, disproportionate to the need and inconsistent with the stdlib-only design. Prefer a **grep-anchored, stdlib-`re` assertion**:

1. Extend the grammar optionally to `path:line:anchor` (or `anchor@path:line`); a bare
   `path:line` remains valid and unchanged (backward-compatible).
2. When an anchor is present, read that one line and assert it matches an anchor pattern.
   For the `def test_*` case the finding calls out, that is
   `^\s*(async\s+)?def\s+<re.escape(anchor)>\b` (Python). Generalize later to language-keyed
   patterns (`fn `, `def `, `class `) if needed; the MVP can scope to `def test_*` anchors.
3. New failure class, symmetric with the existing ones:
   `SYMBOL NOT AT LINE: path:line (expected anchor '<x>', found '<line-text>')`, exit 1.
4. Optional (defer): if the anchor is not at the exact line, scan ±N lines and report *drift*
   rather than absence — this would additionally have caught the F-W74G-P8-001 "+2 line drift"
   class from the wave-74 gate. Keep the MVP strict-line to avoid false negatives.

This is a companion capability on the existing tool, opt-in via the third citation field, so
it does not disturb the 22 existing tests or the docs-writer preflight contract.

---

## Finding 2 — wave-74 artifacts use dual/colliding finding-ID schemes

### Verdict: VALID (claim accurately stated); remediation-scope refinement recommended

### Evidence — two schemes exist for the same wave-74 passes
- **STORY-164 changelog** labels its gate-driven amendments `F-W74P<n>-<seq>` (G-less):
  `F-W74P1-001` (`.factory/stories/STORY-164.md:660`), `F-W74P3-001` (`:659`),
  `F-W74P4-001` (`:658`), `F-W74P6-001` (`:657`), `F-W74P12-001/002` (`:656`),
  `F-W74P13-001` (`:655`). These reference **wave-gate** passes (they run to P12/P13, whereas
  STORY-164's per-story convergence was only 8 passes — `.factory/cycles/wave-74/lessons.md:9`).
- **The authoritative wave-74 gate records** use `F-W74G-P<n>-<seq>` (G-form):
  gate-summary `F-W74G-P1-001/002` (`.factory/cycles/wave-74/wave-gate/gate-summary.md:38`),
  `F-W74G-P3-001` (`:40`), and throughout (`:41,43,45-50`); lessons.md likewise
  (`.factory/cycles/wave-74/lessons.md:28-29,38,49,61-63`).

So within a single wave, the story changelog and the gate artifacts use two different ID
forms for the same pass sequence.

### Evidence — same pass numbers carry different findings
| Pass | STORY-164 changelog (`F-W74P<n>`) | Gate-summary / lessons (`F-W74G-P<n>`) |
|------|-----------------------------------|-----------------------------------------|
| 1 | v1.11 status ready→delivered flip (`STORY-164.md:660`) | currency stale + historical-framing inversion (`gate-summary.md:38`) |
| **3** | **superseded-row added / status vocab** (`STORY-164.md:659`) | **fabricated test-count (HIGH)** (`gate-summary.md:40`) |
| 4 | synonym-note correction (`:658`) | demo index currency stale (`gate-summary.md:41`) |
| 6 | completed-row loci audit (`:657`) | STORY-INDEX "IN PROGRESS" contradiction (`gate-summary.md:43`) |
| 12 | narrative overclaim + AC bullet order (`:656`) | docs-writer §4 example + breaking-change grep (NIT) (`gate-summary.md:49`) |
| 13 | PG-W73-CHANGELOG evidence reframe (`:655`) | STATE.md SHA-shorthand cosmetic (NIT) (`gate-summary.md:50`) |

The finding's specific example (Pass 3: changelog "superseded row" vs gate "fabricated test
count") is **exactly correct**. Every overlapping pass number carries a different referent
across the two records.

### Evidence — the ambiguity actually produced the defect (causal claim SUPPORTED)
The finding's causal claim ("this latent ambiguity enabled F-S165P4-001") is not merely
plausible — it is directly confirmed. The F-S165P4-001 remediation record reads: *"Fabricated
finding-ID **F-W74P8-001** / 'Pass 8' corrected to **F-W74G-P3-001** / 'gate adversarial
convergence Pass 3 (W3)' at all loci"* (`.factory/maintenance/pr-description-row-verify-mandate.md:157`).
The fabricated ID used the **G-less `F-W74P<n>` form** (the STORY-164-changelog scheme) and
the **wrong pass number**. The author reached for the non-canonical form and an unanchored
pass index — the precise degrees of freedom the dual scheme leaves open.

### Refinement (why the proposed remediation is slightly mis-targeted)
The finding proposes distinguishing "per-story pass IDs `F-S<story>P<n>` vs wave-gate IDs
`F-W<wave>G-P<n>`." But `F-S<story>P<n>` is already unambiguous (it begins `F-S`). The real
collision axis is **two wave-level forms**: the canonical `F-W<NN>G-P<n>` (G before P) versus
the G-less `F-W<NN>P<n>`. The G-less form is not a STORY-164 one-off — it appears across the
factory (e.g. wave-70 gate artifacts under `.factory/cycles/wave-70-story-149/wave-gate/`,
and `.factory/policies.yaml`), so both wave forms are in live use repo-wide. The rule must
target that pair, not "per-story vs wave."

### Recommended minimal codification
Add a naming-convention policy to `.factory/policies.yaml` (using `/vsdd-factory:policy-add`)
with three clauses:
1. **Per-story convergence findings** MUST use `F-S<NNN>P<n>-<seq>` (e.g. `F-S165P4-001`).
2. **Wave-gate findings** MUST use the canonical `F-W<NN>G-P<n>-<seq>` (G before P). The
   G-less `F-W<NN>P<n>` form is **deprecated/disallowed** for wave-gate findings.
3. **Cross-references** — when a story artifact (e.g. a changelog row) records an amendment
   prompted by a wave-gate finding, it MUST cite the canonical wave-gate ID, not coin a
   G-less variant or an unanchored pass number.

Optional mechanical enforcement: extend `bin/lint-cycle-artifact` to flag the regex
`F-W[0-9]+P[0-9]` (G-less wave form) as malformed, steering authors to `F-W<NN>G-P<n>`. This
would have caught `F-W74P8-001` at authoring time.

---

## Finding 3 — wave-74 v3.48 attribution discrepancy

### Verdict: VALID

### Evidence — the two records disagree on what produced STORY-INDEX v3.48
- **gate-summary.md** attributes v3.48 to the W6 finding: *"F-W74G-P6-001 MEDIUM: status-legend
  corpus contradiction — STORY-164 `status: delivered` ... but STORY-INDEX wave-74 Delivery
  Progress column still showed 'IN PROGRESS' at one sub-cell; **STORY-INDEX v3.48 fix applied**"*
  (`.factory/cycles/wave-74/wave-gate/gate-summary.md:43`).
- **STORY-INDEX's own changelog** attributes v3.48 to a completely different finding and edit:
  *"v3.48 (2026-07-11): **F-W74P3-001** — superseded row added to status-vocabulary legend
  (STORY-148 ground-truth; loci-rule categories complete; six→seven status values)"*
  (`.factory/stories/STORY-INDEX.md:17`).

These are irreconcilable: one says v3.48 was a Delivery-Progress "IN PROGRESS" cell fix (W6);
the other says v3.48 was the status-vocabulary superseded-row addition (pass 3). The finding is
accurately stated.

### Adjudication — STORY-INDEX changelog is correct; gate-summary.md:43 is the error
The STORY-INDEX changelog is authoritative for the semantics of its own version bumps, and its
v3.48=superseded-row attribution is corroborated three ways:
1. A second locus inside STORY-INDEX itself: *"the wave-74 gate fixed the Status Vocabulary
   legend in three separate passes (v3.48, v3.49, v3.50 — F-W74P3-001, F-W74P4-001,
   F-W74P6-001)"* (`.factory/stories/STORY-INDEX.md:160`).
2. STORY-164's changelog: v1.12 = *"F-W74P3-001: AC-164-001(a) superseded row added (seventh
   status value ...)"* (`.factory/stories/STORY-164.md:659`) — the superseded row is the pass-3
   work.
3. Version-sequence coherence: v3.46 legend added → v3.47 delivered-flip → **v3.48 superseded
   row** → v3.49 synonym-note → v3.50 completed-row audit (`STORY-INDEX.md:15-19`). The
   Delivery-Progress "IN PROGRESS→DELIVERED" edit is recorded separately at **v3.47**
   (`:18`), not v3.48.

`gate-summary.md:43`'s "STORY-INDEX v3.48 fix applied" is therefore wrong: v3.48 was the
F-W74P3-001 superseded-row bump (pass 3), not the W6 Delivery-Progress fix. **gate-summary.md:43
is the artifact that needs correction** — its version cross-reference should point to whatever
STORY-INDEX version actually carried the W6 Delivery-Progress sub-cell fix (ground-truth needed;
it is demonstrably *not* v3.48). The STORY-INDEX changelog needs no change.

### Shared root cause with Finding 2 (confirmed)
This is a concrete instance of the Finding-2 collision, not an independent defect:
- The same W6 pass is recorded under two ID forms — `F-W74G-P6-001` in gate-summary
  (`gate-summary.md:43`) vs `F-W74P6-001` (G-less) in STORY-INDEX (`STORY-INDEX.md:15`) — and
  even those two "P6" records describe different edits (gate: Delivery-Progress cell; STORY-INDEX
  v3.50: completed-row Loci audit).
- With pass numbers P3/P4/P6 shared across two ID forms and divergent referents, a loose
  version cross-reference ("v3.48") lands on the wrong bump. The G-less/G-form ambiguity is the
  enabling condition. Remediating Finding 2 (canonical `F-W<NN>G-P<n>` + deprecating the G-less
  form + the `bin/lint-cycle-artifact` regex flag) would prevent this class; Finding 3 needs no
  separate codification beyond the one-line factual correction to `gate-summary.md:43`.

---

## Inconclusive / flagged items
- The mapping of STORY-164 changelog `F-W74P<n>` rows to specific gate passes cannot be fully
  reconciled from the artifacts (e.g. gate pass W3 records only the pr-description test-count
  finding, on a *different* file than STORY-164.md's superseded-row v1.12 amendment). This
  irreconcilability is itself a manifestation of the ambiguity, not a defect in this
  validation — but it means the changelog's pass labels cannot be authoritatively cross-walked
  to `F-W74G-P<n>` IDs. Flagged, not blocking.
- All three findings are DF-VALIDATION-001-satisfied here (in-process wave-75 convergence
  findings, now research-validated); no external-research dependency was required.
  Context7/web research was not needed — the ctags-vs-grep design tradeoff is resolvable
  from the tool's own stdlib-only constraint.

---

## Correction Record

| Finding | Date | Change |
|---------|------|--------|
| F-W75G-P2-001 | 2026-07-13 | Summary prose at line 19 corrected: "Both findings are sound" → "All three findings are sound"; "Neither is covered" → "None is covered". Stale two-finding count failed to account for Finding 3 (gate-summary.md:43 version attribution discrepancy) added per team-lead follow-up before publication. |
