---
document_type: disposition-plan
producer: research-agent
timestamp: 2026-07-19
scope: E-11 stale-draft disposition (STORY-091/121/143/147/155)
precedent: .factory/planning/upstream-codification-filing-plan.md (D-477)
policy: DF-VALIDATION-001
status: validated — NOTHING filed on GitHub by this agent (human-gated step)
---

# E-11 Stale-Draft Disposition Plan

**Purpose:** DF-VALIDATION-001 validation pass for the 5 older E-11 stale draft
stories (STORY-091, STORY-121, STORY-143, STORY-147, STORY-155), following the
D-477 upstream-routing precedent (STORY-175/177/178/179). The human has directed
that wirerust's E-11 backlog must contain ONLY product-local (wirerust) work;
engine/process work belongs upstream in `drbothen/vsdd-factory`.

**Method:** each story's ACs were validated against (a) the current wirerust tree
(delivered-by-drift checks), (b) `bin/validate-citations` source, and (c) the
465-issue upstream corpus dump (`.factory/planning/vsdd-factory-upstream-issues.md`)
by issue *title/body* text, not story-title similarity.

**No issue is filed. No duplicate is filed. Redacted drafts are provided for the
human-gated filing step.**

---

## Summary Table

| Story | Pts | Classification | Dupe hits (upstream) | Recommended action | Confidence |
|-------|-----|----------------|----------------------|--------------------|------------|
| STORY-091 | 5 | OBSOLETE (residual ENGINE) | #622 / #603 / #396 (family, not exact) | (e) supersede-as-obsolete — no filing | HIGH |
| STORY-121 | 3 | ENGINE | #582 (numeric/count surfaces), #396 (consuming-surface sweep) | (a) COMMENT #582 + x-ref #396 | HIGH (engine) / MEDIUM (dupe target) |
| STORY-143 | 3 | ENGINE (thin product projection optional) | #580 (per-story changelog task — related, not exact) | (b) FILE-NEW upstream, x-ref #580 | HIGH (engine) / MEDIUM (new-vs-comment) |
| STORY-147 | 3 | SPLIT (VALIDATED) | #654 (engine half — near-exact) | (d) RETAIN product half re-scoped (2 pts) + (a) COMMENT #654 (engine half) | HIGH |
| STORY-155 | 3 | ENGINE | **#290 (near-exact)**, #600 (adjacent) | (a) COMMENT #290 + x-ref #600 | HIGH |

**Refutations of the orchestrator's preliminary classification:**
- **STORY-091:** orchestrator called it "ENGINE-shaped concern AND locally
  obsoleted." REFINED → the dominant disposition is **OBSOLETE**: the verification
  core is already delivered by `bin/validate-citations`; the residual (regex
  *discovery*/`--scan` across the corpus) is the only engine-shaped remnant, and
  it needs no upstream action (already represented by #622/#603/#396 and realized
  locally by validate-citations). No filing.
- **STORY-155:** orchestrator flagged "#672/#314 re index churn." REFUTED — #314
  (input-hash frontmatter feedback loop) and #672 (hash wrong-from-birth) are
  input-hash issues, NOT STORY-INDEX status automation. The correct duplicate is
  **#290** (post-merge STORY-INDEX status not flipped). #672/#314 are not relevant.

**Net upstream actions (human-gated):** 1 new issue (STORY-143 engine); 3 comments
(STORY-121→#582, STORY-147→#654, STORY-155→#290); 1 supersede-as-obsolete
(STORY-091, no filing); 1 product-local retention re-scope (STORY-147 product half).

---

## STORY-091 — bin/validate-anchors CLI (5 pts)

**Classification: OBSOLETE (verification core) + residual ENGINE (discovery layer).
Confidence HIGH.**

**AC-vs-validate-citations coverage analysis.** `bin/validate-citations` (delivered
STORY-164, extended by pending STORY-166) already covers STORY-091's verification
core:

| STORY-091 AC | validate-citations equivalent | Covered? |
|--------------|-------------------------------|----------|
| AC-002 MATCH valid single-line anchor | `PASS` on in-range line | YES |
| AC-003 MATCH valid range anchor | range endpoints both checked ≤ line count | YES |
| AC-004 STALE out-of-bounds line | `LINE OUT OF RANGE: path:N (file has M lines)` | YES |
| AC-005 MISSING file-not-found | `FILE NOT FOUND: path` | YES |
| EC-003 start>end invalid range | `INVALID RANGE: path:N-M (start > end)` | YES |
| AC-009 repo-root resolution == compute-input-hash | identical `find_repo_root()` (WIRERUST_REPO_ROOT → walk up for .factory/.git) | YES |

STORY-166 goes *further* than STORY-091: it adds an opt-in `path:line:anchor`
grammar asserting a named symbol exists at the cited line — STORY-091 explicitly
did NOT do this (EC-006: a blank cited line still returns MATCH; STORY-091 checks
line-number validity only). The tooling direction has already surpassed STORY-091.

**Surviving delta:** the only STORY-091 capability not present in validate-citations
is automatic regex **discovery** of anchor citations across `.factory/specs/**`
(AC-001, AC-006/007/008 `--scan` mode). validate-citations requires a pre-built
citations file as input; it does not walk-and-extract. That corpus-discovery layer
is the generic ("any spec tree") engine-shaped remnant — and it is already
represented upstream.

**Duplicate check (upstream):** no exact duplicate (STORY-091 is a tooling proposal,
not a defect). Same-class family issues: #622 (source-line citations with no
coordinate baseline), #603 (no cited-artifact resolution preflight — stories anchor
to phantom modules/files/APIs), #396 (full citation-corpus sweep on BC/ADR bump).
These collectively represent the generic "mechanically validate source-line
citations across the corpus" capability.

**Recommended action: (e) supersede-as-obsolete — no upstream filing.** The
verification core is delivered product-locally by `bin/validate-citations`
(STORY-164) and extended by STORY-166; the residual discovery layer is engine-shaped
but requires no new upstream action (covered by #622/#603/#396 and realized locally).
Retire STORY-091 as delivered-by-supersession.

**DF-VALIDATION-001 attestation:** validated against `bin/validate-citations` source
(lines 135–231), STORY-091 ACs, STORY-166 frontmatter/narrative, and the upstream
citation-family issues (#622/#603/#396). Verdict: OBSOLETE; no issue created from an
unvalidated finding; no duplicate filed.

---

## STORY-121 — F1/F2 Numeric Self-Audit + Consuming-Surface Sweep (3 pts, target .factory/process)

**Classification: ENGINE. Confidence HIGH (engine) / MEDIUM (exact dupe target).**

**Reasoning.** `target_module: .factory/process`. Every deliverable is an
agent-process artifact: an `F3-CONVERGENCE-002` policy entry, an F3 dispatch-template
checklist item, and an F1/F2 authoring-guide `## Numeric Self-Audit` section. Zero
wirerust product surface (no `src/`, no product `CLAUDE.md` content, no repo tooling).
Both procedures — (1) a numeric self-audit gate for F1/F2 docs listed in a story's
`inputs:`, and (2) an 8-step post-fix-burst consuming-surface sweep — govern the
VSDD F3 decomposition workflow generically. Squarely ENGINE. This refutes any
product-local reading: the story is pure factory-process codification.

**Duplicate check (upstream):**
- **#582** (best match) — "process-gap(phase-f1): perimeter scan omits
  index/count/traceability surfaces (BC-INDEX, canonical counts) — count-propagation
  gaps surface late in F2." This is the numeric-self-audit facet: F1/F2 count/
  traceability surfaces not reconciled, surfacing late. Strong overlap.
- **#396** — "full citation-corpus sweep on BC/ADR bump — changelog-row-only check
  misses 3–5 stale pins per bump." This is the consuming-surface-sweep facet
  (BC version bump → stale downstream stamps). Cross-ref.
- The specific *mechanism* STORY-121 documents (an F1/F2 doc listed as a story
  `input:` → every edit re-triggers the input-hash recompute → re-enters the
  fresh-context adversarial loop → serial one-finding-per-round churn) may not be
  captured verbatim upstream; that is the FILE-NEW-defensible core.

**Recommended action: (a) COMMENT on #582** with the numeric-self-audit + 8-step
consuming-surface-sweep proposal, cross-ref #396. **FILE-NEW is defensible** if the
input-hash-loop-churn mechanism is judged distinct enough to track independently
(the churn-amplification-via-input-hash-recompute framing is the novel part).

**DF-VALIDATION-001 attestation:** validated against STORY-121 ACs/target_module and
upstream #582/#396. Verdict: ENGINE; COMMENT recommended; no issue filed by this
agent.

---

## STORY-143 — Release-Changelog Full Prev-Tag..HEAD Range Enumeration (3 pts)

**Classification: ENGINE (optional thin product projection). Confidence HIGH (engine)
/ MEDIUM (file-new vs comment).**

**Reasoning.** The durable fix is a **release-skill / devops-engineer behavior**:
release-prep MUST run `git log <prev-tag>..HEAD --first-parent --oneline` to
enumerate all merged PRs before authoring the CHANGELOG, and cite the PR range as a
completeness anchor. `vsdd-factory:release` and `devops-engineer` are engine agents;
this behavior applies to every VSDD-managed project. The story's AC-143-001 offers
"(a) policy in policies.yaml OR (b) CLAUDE.md note" — option (b) is a product-local
projection, but it is a thin shadow of the engine behavior, not the durable fix.
Under the "E-11 = product-local only" directive, the primary disposition is ENGINE.

**Duplicate check (upstream):**
- **#580** — "process-gap(story-writer): story template lacks a mandatory CHANGELOG
  delivery task — PRs merge without changelog entries, caught only at F5." Related
  (changelog-completeness family) but a *different* mechanism: per-story changelog
  task vs release-time full-range enumeration. Not an exact duplicate.
- #660 (validate-changelog-monotonicity `v`-prefix bug), #428 (verify changelog
  attestations against impl) — distinct.
- No exact match for "release-prep CHANGELOG must be enumerated from
  `prev-tag..HEAD --first-parent`, not hand-summarized."

**Recommended action: (b) FILE-NEW upstream** — a release-skill/devops-engineer
process-gap: release-prep CHANGELOG authoring must be driven by
`git log <prev-tag>..HEAD --first-parent` enumeration (not recollection), with the
PR range as a mandatory completeness-anchor field; cross-ref #580 (per-story
changelog delivery sibling). **Optional product-local retention:** a ≤3-line note in
wirerust's own `CLAUDE.md` "Releasing to main" section is defensible as a thin
projection if the human wants a local reminder — but the load-bearing fix is engine.

**DF-VALIDATION-001 attestation:** validated against STORY-143 ACs and upstream
#580/#660/#428. Verdict: ENGINE; FILE-NEW recommended (comment-on-#580 defensible);
no issue filed by this agent.

---

## STORY-147 — Mutation-Testing Defaults: mutants.toml + CLAUDE.md Guidance (3 pts)

**Classification: SPLIT (VALIDATED). Confidence HIGH.**

**Delivered-by-drift check (2026-07-19):** NOT delivered by drift. Verified on the
current tree:
- No `mutants.toml` at repo root; no `.cargo/mutants.toml`.
- No `[package.metadata.mutants]` table in `Cargo.toml`.
- No "Mutation testing" note in `CLAUDE.md` (grep for "mutation" returns nothing).

The product half is genuinely undelivered.

**SPLIT validation:**
- **Product half (RETAIN LOCALLY):** `mutants.toml` at wirerust repo root (jobs=1)
  and a `CLAUDE.md` "Mutation testing" note. Both are wirerust-local files — squarely
  product-local. Survives.
- **Engine half (route upstream):** the mutation-testing skill / formal-verifier
  agent should default to safe parallelism (jobs ≤ 2 or a generous timeout) so ALL
  VSDD projects are safe by default.

**Duplicate check (upstream, engine half):**
- **#654** (near-exact) — "process-gap(formal-verifier): bundle-scoped mutation runs
  need --timeout 480 or --jobs 2 — 240s cap causes timeout-adjudication overhead on
  large diffs." Same root cause: default parallelism/timeout produces
  timeout-adjudication failures in mutation runs. wirerust's evidence (infinite-loop
  mutants pegging all 8 cores → false "0 missed" → 2 real survivors hidden, surfaced
  only by a `--jobs 1` re-run) is strong confirming field data. COMMENT.
- #645 (transient-mutation restore targets last COMMIT), #652 (mutation-coverage
  claims need empirical run), #477 (tautological zero-assertion tests) — distinct.

**Recommended action:**
- **(d) RETAIN product half LOCALLY, re-scoped.** Surviving ACs:
  - **AC-147-001** — `mutants.toml` at repo root sets `jobs = 1` (≤ 2) / generous timeout.
  - **AC-147-002** — `cargo mutants` (no `--jobs`) uses the configured low-parallel default.
  - **AC-147-003** — `CLAUDE.md` "Mutation testing" note: recommended invocation,
    why high `--jobs` is unsafe on this suite, reference to the motivating cycle
    (keep as a local pointer; the engine-skill-default aspiration is dropped from
    the product-local framing).
  - **AC-147-004** — self-audit: fresh-checkout `cargo mutants` cannot silently
    return false-clean.
  - **Re-scoped point estimate: 2 pts** (≤5-line config + ≤10-line doc note +
    self-audit; the engine-skill-default work is removed).
- **(a) COMMENT on #654** (engine half) with redacted evidence.

**DF-VALIDATION-001 attestation:** validated against the current tree (no mutants
config, no CLAUDE.md note) and upstream #654/#645/#652. Verdict: SPLIT; product half
retained (2 pts), engine half → COMMENT #654; no issue filed by this agent.

---

## STORY-155 — Auto-Update STORY-INDEX Status draft→merged on PR Merge (3 pts)

**Classification: ENGINE. Confidence HIGH.**

**Reasoning.** STORY-INDEX is a `.factory/` artifact (factory-artifacts branch); the
per-story-delivery flow (pr-manager post-merge / state-manager cycle-state-update)
is an engine agent workflow. Automating the post-merge status flip (draft→merged,
PR#+SHA stamp, wave-row close), idempotently, is pure factory index automation that
applies to every VSDD project. ENGINE.

**Duplicate check (upstream):**
- **#290 (near-exact)** — "process-gap(state-manager): post-merge STORY-INDEX status
  field not flipped to 'completed'." This is STORY-155's core gap almost verbatim.
- **#600** (adjacent) — "index sweep lands partially — changelog row without
  frontmatter bump + wrong-position insert, while the report asserts the full sweep."
  Cross-ref (partial-sweep failure mode).
- #310 (index TOTAL bumped ahead of on-disk artifacts) — inverse condition, distinct.

**Refutation of orchestrator note:** the orchestrator flagged "#672/#314 re index
churn." REFUTED — #314 (input-hash frontmatter feedback loop) and #672 (hash
wrong-from-birth) are input-hash issues, unrelated to STORY-INDEX status automation.
The correct duplicate is **#290**.

**Recommended action: (a) COMMENT on #290** with redacted field evidence (E-21 F7
consistency-audit finding: 4 stories showed `status: draft` days after their PRs
merged; required a manual reconciliation pass) and the two additive requirements
STORY-155 contributes beyond #290's title: **idempotency** (re-running produces no
net diff) and **wave-delivery-row closure** (PR#+SHA stamp; mark DELIVERED & CLOSED
when all wave stories merged). Cross-ref #600.

**DF-VALIDATION-001 attestation:** validated against STORY-155 ACs and upstream
#290/#600/#310. Verdict: ENGINE; COMMENT on #290 (confirmed gap + additive
requirements); no issue filed by this agent.

---

## Redacted Drafts for Public Posting (human-gated)

Redaction discipline (mirrors D-477): internal decision IDs, story/AC IDs,
wave/pass/cycle names, PG-codenames, PR numbers, and repo-specific file paths are
stripped or genericized. Preserved: mechanisms, failure scenarios, occurrence counts,
proposed remediations, upstream (`drbothen/vsdd-factory`) issue cross-references, and
the "downstream VSDD-factory-managed project" framing. Engine/repo-generic paths
(`.factory/`, `src/`, `CLAUDE.md`, `mutants.toml`, `Cargo.toml`, `STORY-INDEX`) are
retained — they are not project-identifying.

### REDACTED comment — target #582 (STORY-121)

> **Two F3-decomposition convergence gates (downstream VSDD-factory-managed project).**
>
> Confirming this perimeter-scan gap and proposing two complementary gates. Root
> failure: an F1/F2 delta/spec-evolution doc listed in a story's `inputs:` field means
> every edit to that doc re-triggers the story's input-hash recompute, which re-enters
> the fresh-context adversarial loop — so stale counts and version-stamps surface
> serially, one finding per round (one cycle took 10 rounds this way, its most-churned
> phase).
>
> Proposed: (1) a **numeric self-audit gate** — before any F1/F2 doc is declared as a
> story input, every count/sub-count/construction-site total is reconciled against
> `grep` ground-truth and recorded in a `## Numeric Self-Audit` section in the doc
> itself; the story-writer refuses to finalize the input-hash if that section is
> absent. (2) A **post-fix-burst consuming-surface sweep** — after any BC version bump
> or count/construction-site revision, sweep every consuming surface in one atomic
> burst (BC body, BC-INDEX, spec-changelog, consuming-story body/frontmatter/subtable,
> dep-graph matrix, and any F1/F2 input docs) before re-entering adversarial review.
> Cross-ref #396 (the citation-corpus-sweep-on-BC-bump facet).

### REDACTED new issue — (STORY-143)

> **Title:** `process-gap(devops-engineer/release): release CHANGELOG must be enumerated from git log <prev-tag>..HEAD --first-parent, not hand-summarized — an entire epic was omitted from a release entry`
>
> **Body:**
>
> ## Summary
> During a minor release, the CHANGELOG entry and GitHub release notes were authored
> from a hand-summarized "what shipped" recollection scoped to the most recent wave.
> An entire analyzer epic (~9 stories, ~18 PRs) that had merged earlier in the same
> release window was silently omitted, because it was invisible to a recollection-based
> approach. Two post-release docs-only correction PRs were required to complete the entry.
>
> ## Proposed fix
> The release-prep step must run `git log <prev-tag>..HEAD --first-parent --oneline` to
> enumerate every merged PR before authoring the CHANGELOG, and the entry must cite the
> PR range (e.g. "Includes PRs #NNN–#MMM") as a mandatory completeness anchor.
> Hand-summarized / recollection-based authoring must be prohibited as the sole source;
> the commit-range enumeration is the authoritative input, cross-checked against the
> drafted entry before the release PR opens.
>
> ## Distinct from existing issues
> - #580 — per-story CHANGELOG delivery task (PRs merging without entries). Related
>   family, but that is authoring-time-per-story; this is release-time full-range
>   reconciliation. Different mechanism.
>
> ## Provenance
> Observed during a downstream VSDD-factory-managed project's release. Validated per the
> downstream project's finding-validation gate before filing.

### REDACTED comment — target #654 (STORY-147 engine half)

> **Confirming field data — high default `--jobs` hides real survivors behind
> load-induced false timeouts (downstream VSDD-factory-managed project).**
>
> During a hardening phase, `cargo mutants --jobs 8` reported "0 missed mutants" —
> apparently clean. Two real surviving mutants were hidden: infinite-loop mutants pegged
> all 8 cores, inflating every other mutant's wall-clock past the auto-timeout threshold,
> so real survivors were adjudicated as timeouts instead of coverage gaps. A `--jobs 1`
> re-run surfaced them (plus eleven more real gaps that were then closed). This is the
> same timeout-adjudication-under-load failure this issue describes, on a small suite
> rather than a large diff — the trigger is core saturation, not diff size. Supports
> defaulting the mutation-testing skill to safe parallelism (`--jobs` ≤ 2 or a generous
> `--timeout`) so a fresh run cannot silently return false-clean. A committed
> `mutants.toml` / `[package.metadata.mutants]` low-jobs default is the first line of
> defense; the skill default is the durable one.

### REDACTED comment — target #290 (STORY-155)

> **Confirming + additive requirements (downstream VSDD-factory-managed project).**
>
> Confirming this gap with field data: a consistency audit caught four stories still
> showing `status: draft` in the story index days after their PRs had squash-merged;
> correction required a manual reconciliation pass that flipped the index rows, stamped
> PR numbers and merge SHAs, and closed the delivery rows. This recurs every cycle.
>
> Two additive requirements beyond flipping the status field: (1) the post-merge update
> must be **idempotent** — re-running it on an already-merged row produces no net diff;
> and (2) it should also update the **wave/batch delivery row** (record PR# + merge SHA,
> and mark the wave DELIVERED & CLOSED when all its stories are merged), not just the
> per-story status cell. Best placed as a standing post-merge step in the per-story
> delivery flow (pr-manager post-merge or state-manager cycle-state-update), in the same
> commit. Cross-ref #600 (partial index-sweep failure mode).

---

## Inconclusive / flags for the team lead

- **STORY-121 dupe target is MEDIUM confidence.** #582 is the best home for the
  numeric-self-audit facet, but the *input-hash-recompute-amplifies-churn* mechanism
  may warrant a standalone FILE-NEW if the maintainer judges it distinct from #582's
  perimeter-scan framing. Drafts are written to convert cleanly comment→issue.
- **STORY-143 is FILE-NEW vs COMMENT-#580, a judgment call.** #580 is the nearest
  neighbor but a different mechanism (per-story vs release-time). Recommend FILE-NEW;
  downgrade to a comment on #580 if the maintainer prefers to keep the changelog
  family on one thread.
- **STORY-091 files nothing.** If the human wants the generic corpus-discovery
  (`--scan`) layer tracked upstream despite validate-citations covering the core,
  the natural home is #603 (cited-artifact resolution preflight) or #622
  (source-line citation baseline) — but this is optional; no action is the default.
- **STORY-147 product-half re-scope to 2 pts** is my estimate; if the human wants the
  CLAUDE.md note to also carry the engine cross-reference (`see drbothen/vsdd-factory#654`),
  keep it at 3 pts to cover the linkage bookkeeping.
