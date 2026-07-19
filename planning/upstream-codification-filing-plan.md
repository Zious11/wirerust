# Upstream Codification Filing Plan — feature-iec104 Cycle-Close Process-Gaps

**Purpose:** DF-VALIDATION-001 validation pass for the 12 feature-iec104 cycle-close
process-gap candidates directed for filing on the ENGINE repo `drbothen/vsdd-factory`
(instead of delivery as wirerust E-11 stories STORY-175..179).

**Author:** research/validation agent · **Date:** 2026-07-19
**Scope:** validate + dedup each candidate against the 465-issue upstream tracker; produce
verdicts and drafts. **NOTHING is filed on GitHub by this agent.** No issue is filed
unvalidated; no duplicate is filed.

**Method:** each near-duplicate call was checked against the actual upstream issue *body*
(fetched via `gh issue view <n> --repo drbothen/vsdd-factory`), not title similarity.
Issues fetched and read in full: #269, #305, #314, #327, #344, #350, #380, #395, #396,
#400, #457, #461, #494, #567, #573, #579, #586, #622, #623, #626, #635, #636, #637, #641,
#653, #655, #656, #669, #672, #680, #682, #686, #687.

---

## Summary Table

| # | Candidate | Verdict | Target | Confidence |
|---|-----------|---------|--------|------------|
| 1 | PG-DEMO-JSON-FABRICATION | COMMENT | #494 | MEDIUM (FILE-NEW defensible) |
| 2 | PG-MERGE-AUTH-SUBAGENT-CLASSIFIER | COMMENT | #461 | HIGH |
| 3 | PG-ADVERSARY-IDLE-NO-REPORT | DUPLICATE | #457 | HIGH |
| 4 | PG-ADVERSARY-SEVERITY-CALIBRATION | COMMENT | #686 | MEDIUM |
| 5 | PG-DOC-CURRENCY-SWEEP | COMMENT | #682 (x-ref #687) | MEDIUM-HIGH |
| 6 | F3-DECOMPOSITION-BC-FIDELITY | COMMENT | #305 (x-ref #400/#327/#573/#669) | MEDIUM |
| 7 | Input-hash self-referential drift | PRODUCT-LOCAL | (opt. COMMENT #314) | MEDIUM |
| 8 | PG-STATE-RECOVERY-SCOPE + PG-VERIFY-ALL-WORKTREES | COMMENT | #655 | HIGH |
| 9 | validate-count-propagation "E-11"→"11" false-positive | FILE-NEW | new bug(hooks) | HIGH |
| 10 | PG-HASH-HOOK-DIVERGENCE | DUPLICATE (no-action) | #637 | HIGH |
| 11 | PG-GATE-VOCAB-BLINDSPOT (skeleton/seam) | PRODUCT-LOCAL | (opt. COMMENT #682) | HIGH |
| 12 | PG-SPEC-VERSION-CITATION-CURRENCY | COMMENT | #396 (x-ref #395/#471) | HIGH |

**Net upstream actions:** 1 new issue (candidate 9); 6 substantive comments (1,2,4,5,6,8,12
— note 12 total comment targets: 1,2,4,5,6,8,12 = 7); 2 duplicates (3 no-action confirming
comment optional, 10 pure no-action); 2 product-local (7,11) with optional family-thread
comments.

---

## Candidate 1 — PG-DEMO-JSON-FABRICATION

**Verdict: COMMENT-on-#494.** Confidence MEDIUM (FILE-NEW is a defensible alternative).

**Defect:** demo-recorder hand-writes illustrative JSON/enum values in demo-evidence rather
than capturing real `cargo run`/`cargo test` serde output. 3 occurrences in feature-iec104
(F5R2-02 MEDIUM: non-existent `tactic` variant; F-B1 HIGH ×3 artifacts: `category:"Protocol"`,
`verdict:"Anomaly"` — non-existent enum variants — `confidence:"High"` wrong casing vs
`rename_all="lowercase"`, wrong MITRE technique). Root cause D-467: hand-written JSON
bypasses the serde path and diverges from actual variant names/rename rules without a
compile error.

**Dedup analysis:**
- **#636 (compared, distinct):** demo host-path scrub. Different defect class entirely
  (path privacy, not content fabrication). #636's own body lists #494 as the
  evidence-truthfulness sibling. NOT a duplicate.
- **#586 (distinct family):** fabricated *tool surfaces* in operator docs (CLI cmds, git
  config keys, field names). Fabrication family, but the surface is runbook prose, not
  serialized JSON evidence.
- **#494 (best match):** "demo-recorder+adversary: attestation quality — fabricated evidence
  text." Axis A is *evidence text describing architecture that never existed* — same root
  cause #494 states explicitly: "the framework has no contract that evidence text must be
  generated from actual command execution or live observation." wirerust's JSON fabrication
  is a direct instance of that class.

**Why COMMENT not DUPLICATE:** wirerust adds a materially new *manifestation and mechanical
remedy* #494 does not cover: structured **JSON/serde enum-variant** fabrication in a non-
interactive CLI/library product (vs #494's interactive-signal / delta-AC framing), which is
**mechanically checkable** — every enum-carrying JSON field can be validated against the
serde-serialized form emitted by the real binary and against the variant set in `src/`.
This is a concrete, gate-able extension of #494's "attach raw output" ask.

**Why FILE-NEW is defensible:** the proposed engine change (demo-recording skill must derive
JSON from captured stdout + a serde-variant validation gate) is specific enough to track
independently. If the maintainer prefers, this can be a standalone
`process-gap(demo-recorder): JSON demo-evidence must be derived from real serde output, not
hand-authored — enum-variant fabrication survives to adversarial HIGH`. Recommend COMMENT
first; escalate to FILE-NEW only if #494 is closed/too broad.

**Draft comment (on #494):**

> **Additional field data — structured-JSON / serde enum-variant fabrication (wirerust
> feature-iec104, 3 occurrences, D-467).**
>
> Same root cause as this issue's Axis A (evidence text not generated from real execution),
> in a non-interactive CLI/library product where the "evidence text" is a serialized-JSON
> block rather than an interactive signal trace:
> - F5 R2 (F5R2-02, MEDIUM): demo JSON carried a non-existent `MitreTactic` variant.
> - F-P4/F5 R3 (F-B1, HIGH ×3 artifacts): `category:"Protocol"` and `verdict:"Anomaly"` —
>   variants that do not exist in the shipped enums; `confidence:"High"` in the wrong case
>   (serialized form is lowercase under `#[serde(rename_all="lowercase")]`); plus a wrong
>   MITRE technique in the accompanying prose. The demo `.rs` did not compile.
> - Cost: the feature code was CONVERGED by F5 R2, yet R3–R5 were consumed chasing
>   fabricated demo JSON rather than real defects.
>
> **Why this manifestation is mechanically gate-able (a remedy beyond "attach raw output"):**
> for structured output, every enum-carrying JSON field can be (a) captured from real
> `cargo run -- … --format json` / `cargo test -- --nocapture` stdout, and (b) validated
> against the serde-serialized form and the variant set declared in `src/`. Hand-authored
> illustrative JSON should be prohibited for any product with a real serialization path.
> Suggest the demo-recording skill gain an explicit "derive JSON from captured stdout +
> spot-check ≥1 enum-carrying field against source" step. Cross-ref #636 (path-scrub sibling
> in the same protocol), #586 (fabricated tool surfaces).

---

## Candidate 2 — PG-MERGE-AUTH-SUBAGENT-CLASSIFIER

**Verdict: COMMENT-on-#461.** Confidence HIGH.

**Defect:** subagent cannot execute `--admin` merge on relayed human consent; orchestrator-
direct `--admin` bypass ALSO denied; a named `--admin` bypass also denied. Resolution =
human-direct merge in the main thread (D-463, STORY-174 wave-83, 2026-07-17; reconfirmed on
PR #419 2026-07-18 and PR #414 2026-07-18/19).

**Dedup analysis:**
- **#626 (compared, distinct):** self-review `--approve/--request-changes` forbidden from
  the author account → empty `reviewDecision`. That is a *review-verdict* gate, not merge
  execution. Distinct.
- **#674 (compared, distinct):** per-story-delivery step (f) degrades to self-review when
  the subagent cannot spawn a pr-reviewer. Distinct mechanism (spawn capability), not merge
  auth.
- **#461 (best match):** "auto-mode classifier permission-laundering breaks agent-driven
  merge." Exactly this class: agent teammates blocked from `gh pr merge` on their own PRs;
  peer→parent laundering shape identified; resolution = surface fully-converged PR to the
  human, who merges. #461 explicitly asks the maintainer to decide whether to document the
  "known ceiling" in per-story-delivery.md.
- Adjacent (relayed-authorization trust family, cross-ref only): #350 (classifier blocks
  relayed signed *commits*), #380 (subagent refuses relayed approval on principle), #269
  (SendMessage authorization treated as untrusted). These are about *commits/approvals*, not
  the `--admin` *merge* ceiling; cite as related.

**Why COMMENT not DUPLICATE:** wirerust adds material NEW evidence that sharpens #461's open
scoping question: the ceiling is not limited to peer subagents under auto-mode — the
**orchestrator-direct `--admin` merge is also denied**, and a **named `--admin` bypass is
also denied**. This establishes that *no* agent principal (peer, or the orchestrator acting
directly) can bypass; only human-direct action in the main thread merges. That is a stronger
claim than #461's teammate-launder framing and directly supports #461's "document the known
ceiling" proposal. Plus two fresh reconfirmations (PR #419, PR #414).

**Draft comment (on #461):**

> **Confirming + extending field data (wirerust feature-iec104, D-463; PRs #419, #414).**
>
> This ceiling is broader than the peer-teammate case. In wirerust we confirmed that *no*
> agent principal can bypass the merge gate:
> 1. Subagent `--admin` merge on orchestrator-relayed human consent — denied.
> 2. **Orchestrator-direct `--admin` merge (not via a subagent) — also denied.**
> 3. A **named `--admin` bypass — also denied.**
>
> Resolution path, reconfirmed three times: **human-direct merge in the main thread** is the
> only valid execution when the merge-auth classifier conditions are unmet. Evidence:
> D-463 (STORY-174, 2026-07-17); PR #419 (2026-07-18, two classifier halts then human-direct
> merge); PR #414 (2026-07-18/19, same pattern). The classifier halt is *correct behavior*,
> not a bug.
>
> This supports your "document the known ceiling" proposal — but the doc should state the
> ceiling applies to the orchestrator acting directly, not only to peer teammates, so
> operators stop attempting an orchestrator-direct `--admin` as a fallback. Related but
> distinct: #350 (relayed signed *commits*), #380/#269 (relayed *approval* trust). This
> issue is specifically the `gh pr merge` execution ceiling.

---

## Candidate 3 — PG-ADVERSARY-IDLE-NO-REPORT

**Verdict: DUPLICATE-of-#457** (optional one-line confirming comment; no material new
evidence → primarily no-action). Confidence HIGH.

**Defect:** agents complete a pass then idle without emitting a report; CLEAN vs idle
indistinguishable. Made agent-generic by a 2026-07-18 spec-steward occurrence.

**Dedup analysis:**
- **#457 (exact match):** "completed-but-unreported liveness gap — work finishes, report
  never delivered without an explicit ping." Already **agent-generic** — documents 3 roles
  in one session (adversary, state-manager, consistency-validator), the exact "single ping
  suffices" behavior, and the validated mitigation (dispatch-template report-before-idle
  line). This is the same gap, already generalized beyond adversary.
- #211 (distinct): read-only adversary cannot *persist* its own report (tool-profile /
  round-trip cost), not a report-before-idle liveness gap.

**Why DUPLICATE not COMMENT:** #457 already documents cross-role genericity + a validated
fix. wirerust's spec-steward occurrence is confirmatory, not materially new — #457 already
covers a superset (3 roles + fix). Filing new content would be redundant. If desired, a
single confirming line adds trivial weight:

> **+1 confirming occurrence:** wirerust feature-iec104, spec-steward dispatch 2026-07-18 —
> completed silently, no report; single ping recovered it. Consistent with the 3-role
> pattern already documented here; the dispatch-template report-before-idle mandate remedy
> holds.

---

## Candidate 4 — PG-ADVERSARY-SEVERITY-CALIBRATION

**Verdict: COMMENT-on-#686.** Confidence MEDIUM.

**Defect:** late passes emit advisory findings against code frozen since an early pass;
adversary instances diverge on severity for that unchanged surface (STORY-173 P9–P14: one
instance MEDIUM, another LOW/advisory on equivalent frozen-code findings). Proposed rule:
findings against code frozen-since-pass-N are capped at LOW unless they demonstrate an
observable behavioral regression at current HEAD.

**Dedup analysis:**
- **#686 (best match):** "finding decay is non-monotone across fresh-context passes — review
  technique depth varies, and **policy-trigger interpretation drifts between reviewers**."
  #686's second observation is exactly inter-reviewer calibration divergence (two fresh
  reviewers ruled oppositely on the same facts). wirerust's severity-vs-freeze-state
  divergence is a sibling facet of the same "fresh context erases prior disposition →
  re-litigation" root.
- **#344 (related, counter mechanism):** cosmetic/frozen-code bursts should not reset the
  clean-streak counter; proposes a machine-checkable "cosmetic burst" test and terminal-round
  judgment when "production source is byte-frozen for ≥2 rounds." This is the *counter*
  treatment; wirerust proposes the *severity-rating* treatment.
- **#579 (related, reopening):** recurrence of a resolved LOW/NITPICK should be a REOPENING,
  escalated to MEDIUM before it resets the streak. Adjacent but about recurrence, not
  freeze-state severity ceiling.

**Why COMMENT not FILE-NEW:** the calibration-divergence root is #686's territory; the
frozen-code severity ceiling is a complementary proposed rule best attached there, cross-
referencing #344 (counter) and #579 (reopening) so the three facets stay linked rather than
fragmenting into a fourth near-adjacent issue.

**Draft comment (on #686):**

> **Third facet of reviewer calibration divergence: severity-vs-code-freeze-state (wirerust
> feature-iec104, STORY-173 P9–P14).**
>
> Beyond technique-depth and policy-trigger drift, we observed severity divergence on *frozen
> code*: production code unchanged since Pass 2, reviewed again at P9–P14 by fresh instances
> — one rated findings MEDIUM, another rated equivalent findings LOW/advisory. The
> reconciliation overhead blurred whether a pass was genuinely CLEAN.
>
> Proposed calibration rule (complements #344's counter treatment and #579's reopening rule):
> a finding against code that has been byte-frozen since a *named* earlier pass is a
> retrospective re-assessment, not a forward regression scan; it should be capped at **LOW/
> advisory unless the reviewer demonstrates an observable behavioral regression at current
> HEAD.** If behavior is unchanged since the freeze point, the finding was equally reportable
> in the earlier (accepted/deferred) pass, so MEDIUM+ is not appropriate. Dispatches could
> carry a neutral "code frozen since Pass N" fact (the same shape as your policy-timing
> sequencing-fact suggestion) so fresh reviewers calibrate consistently. Cross-ref #344, #579.

---

## Candidate 5 — PG-DOC-CURRENCY-SWEEP

**Verdict: COMMENT-on-#682** (cross-ref #687). Confidence MEDIUM-HIGH.

**Defect:** no pre-adversarial code-comment/test-header doc sweep step; 12 of 17 STORY-173
passes were doc-drift (stale comments/test headers referencing earlier spec versions, left-
over TODOs), with the feature code CONVERGED by Pass 2.

**Dedup analysis:**
- **#682 (best match):** "stale RED-gate / todo!() docstrings survive the Red→Green
  transition." Proposes (item 2) an explicit implementer checklist step to sweep RED-gate
  docstrings across *all* files, and (item 3) a doc-integrity sub-gate driven to zero in
  parallel without resetting the behavioral counter. wirerust's ask *generalizes* item 2 to
  all stale spec-version/name citations in code comments + test headers (not only RED-gate
  markers) and positions it as a mandatory *pre-adversarial-dispatch* step.
- **#687 (strong sibling, cross-ref):** Gap 2 proposes running a comprehensive citation
  self-audit *once, in full, BEFORE entering strict N-consecutive-clean convergence* — the
  identical "front-load to drain the trickle before the clock starts" idea, but for *spec
  documents*. wirerust's surface is *code comments + test headers*.
- **#395 (cross-ref):** test-file header/docstring version stamps not scanned — the tests/
  slice of wirerust's ask.
- **#344 (cross-ref):** why the residual doc-drift stream stalls convergence.

**Why COMMENT not FILE-NEW:** the pieces exist across #682 (code docstrings, implementer
step, sub-gate), #687 (front-load timing), #395 (test headers). wirerust's contribution is
(a) generalizing #682's sweep beyond RED-gate markers to all stale spec-version/name
citations, (b) making it a named pre-adversarial pipeline step, and (c) quantified field
data (12/17 = ~70% of passes were doc-drift). That is material new weight on #682, not a new
class.

**Draft comment (on #682):**

> **Generalize the pre-Red→Green sweep to a mandatory pre-adversarial doc-currency step —
> field data: 12 of 17 passes were doc-drift (wirerust feature-iec104, STORY-173).**
>
> STORY-173 converged (code-wise) by Pass 2, then took 15 more passes; post-analysis showed
> 12 of 17 passes were driven by doc-accuracy findings — stale code comments, test-header
> prose citing earlier spec versions, leftover implementation-pass TODOs — not behavioral
> findings. This is item 2 of this issue widened past RED-gate markers: any comment/test-
> header referencing an AC/BC/field/version that changed during delivery is the same class.
>
> Suggest the implementer workflow gain a named **pre-adversarial-dispatch doc sweep** (run
> after CI is green, before the adversary is dispatched) covering `src/` inline comments and
> `tests/` headers/docstrings for stale spec-version/name references and leftover TODO/FIXME.
> Recording "doc sweep: PASS" before dispatch drains the trickle before the convergence clock
> starts — the same front-load logic #687 Gap 2 proposes for spec artifacts. Cross-ref
> #687 (spec-artifact sibling), #395 (test-header version stamps), #344 (why the residual
> stream stalls the counter).

---

## Candidate 6 — F3-DECOMPOSITION-BC-FIDELITY

**Verdict: COMMENT-on-#305** (cross-ref #400, #327, #573, #669). Confidence MEDIUM.

**Defect:** story ACs drifted from their traced BCs between F3 decomposition and F4 delivery,
4 confirmed occurrences: STORY-169 (`AsduHeader`→`Asdu` rename, wrong min-length guard);
STORY-170 (false-positive T0827 where BC says no-emit; confidence Possible→Likely; reserved-
TypeID scope; naming); STORY-172 (`FlowId`→`FlowKey` non-existent; carry-overflow semantics;
malformed-LEN PC4 contradiction); STORY-173 (`"impact"` tactic string vs `MitreTactic` enum —
compilation blocker). All fixed by ad-hoc BC-realignment at delivery. Proposed: mandatory
implementer **pre-coding AC↔BC fidelity check** as an F3→F4 gate producing a written table.

**Dedup analysis (this candidate maps across several existing issues — no single duplicate):**
- **#305 (best umbrella):** "story decomposition can produce unbuildable stories: AC-collapse,
  API-name drift, hidden infrastructure deps." Directly covers API-name drift
  (`SaveManager.load` vs `load_game` vs `load_save`) and BC↔story AC fidelity; proposes
  decomposition-time coverage ledger, API-name validation vs real code, and a Phase-3-entry
  dependency-satisfiability gate. wirerust's field-rename / wrong-guard / compilation-blocker
  cases are the same class.
- **#400 (facet):** AC↔BC PC-level trace table absence → wrong-PC anchors. Covers the AC-vs-BC
  precondition-anchoring facet.
- **#327 (facet):** AC trace citations not resolved against cited BC (fabricated/mis-anchored
  sub-anchors). Deterministic-lint proposal.
- **#573 (facet):** symbols/call-forms in normative AC postconditions never mechanically
  verified vs code — covers the `FlowKey`/tactic-enum symbol/type mismatch and the
  compilation-illegal-signature class.
- **#669 (facet):** polarity inversion (nouns right, verb flipped) — covers STORY-170's
  emit-vs-no-emit inversion specifically.

**Why COMMENT not FILE-NEW:** the mechanisms are collectively covered by #305 + #400 + #327 +
#573 + #669; a standalone new issue would largely restate them. wirerust's genuinely distinct
contribution is a *lifecycle placement*: an **implementer-side pre-coding gate at F3→F4** (all
four occurrences were caught only by ad-hoc realignment at delivery, i.e., the upstream
decomposition/authoring gates in #305/#400/#327 did not fire). Best captured as a comment on
#305 proposing the implementer-side gate as the last-line-of-defense complement to #305's
decomposition-time proposals.

**Draft comment (on #305):**

> **Field data + an implementer-side last-line-of-defense proposal (wirerust feature-iec104,
> 4 occurrences).**
>
> Four F4 deliveries hit AC↔BC drift that the decomposition/authoring gates did not catch;
> each was rescued by ad-hoc BC-realignment *at delivery time*:
> - STORY-169: `AsduHeader`→`Asdu` rename + wrong minimum-length guard (API-name drift, your
>   defect #2).
> - STORY-170: AC specced a false-positive emission where the BC mandates no finding
>   (emit-vs-no-emit polarity inversion — cross-ref #669), plus confidence Possible→Likely.
> - STORY-172: `FlowId`→`FlowKey` (symbol absent from code — cross-ref #573), carry-overflow
>   semantics contradicting the BC.
> - STORY-173: tactic string `"impact"` vs the `MitreTactic` enum — a **compilation blocker**
>   on first implementation (symbol/type mismatch — cross-ref #573).
>
> Because all four surfaced only at coding time, we propose an **implementer pre-coding AC↔BC
> fidelity check as an explicit F3→F4 gate step**: before writing any code, the implementer
> produces a written table mapping each AC to the *current* version of its traced BC and
> confirms field names, guards/conditions, confidence/verdict levels, and emit/no-emit
> decisions match; discrepancies force BC-realignment before coding. This complements the
> decomposition-time ledger / dependency gate you propose — it's the last line of defense
> when the upstream gates miss. Cross-ref #400 (PC-level trace table), #327 (sub-anchor
> resolution lint), #573 (normative-AC symbol/signature verification), #669 (polarity).

---

## Candidate 7 — Input-hash self-referential drift (post-delivery re-baseline)

**Verdict: PRODUCT-LOCAL** (optional light COMMENT-on-#314 for the family thread).
Confidence MEDIUM.

**Defect:** a delivered story whose ACs edit the very spec files it lists as `inputs:` will
ALWAYS have a stale `input-hash` immediately after delivery; needs a standard post-delivery
re-baseline step (`compute-input-hash --write`). Observed STORY-164/165, re-baselined
2026-07-18.

**Dedup analysis:**
- **#314 (closest, but distinct mechanism):** input-hash includes YAML frontmatter →
  populating an artifact's own hash *spuriously* drifts downstream consumers even though the
  body is unchanged. #314's drift is **spurious** (body byte-identical) and its fix is
  "strip frontmatter before hashing." wirerust's drift is **legitimate** — the input file's
  *body content* genuinely changed (a BC was amended during the story's own delivery) — and
  #314's frontmatter-strip fix would NOT eliminate it. Different root cause, different fix.
- **#672 (distinct):** hash wrong-from-birth (decompose omits terminal `--update`). Not this.
- **#623 (distinct):** `--update` no-ops when the field is absent. Not this.

**Why PRODUCT-LOCAL:** the drift is *correct* drift-detector behavior (not a bug), and the
remedy is a documented delivery-checklist step — exactly what STORY-178 AC-178-003 places in
wirerust's `delivery-doc-currency-protocol.md`. The pattern is tied to wirerust's convention
of governance stories (E-11/E-22) that trace to actively-revised specs. There is no engine
*defect* to fix; the engine already provides `--write`. Keep it as the wirerust checklist
item.

**Optional light comment (on #314), only if maintainer wants the family thread complete:**

> **Sibling case — *legitimate* self-referential drift (not the frontmatter feedback loop).**
> Distinct from this issue's spurious drift: when a story's own delivery amends a BC/spec
> file that the story lists as an input, the story's `input-hash` goes stale on *real* body-
> content change — correct detection, not a frontmatter artifact. Frontmatter-stripping (this
> issue's fix) would not address it. The remedy is a documented post-delivery re-baseline
> step (`compute-input-hash --write <story>` after wave close). Filing here only so the input-
> hash family thread notes the legitimate-drift case alongside the spurious-drift one.

---

## Candidate 8 — PG-STATE-RECOVERY-SCOPE + PG-VERIFY-ALL-WORKTREES

**Verdict: COMMENT-on-#655.** Confidence HIGH.

**Defect:** session-boundary recovery and post-agent verification must span ALL worktrees
AND the main develop checkout. A fix agent committed to the main develop checkout instead of
its worktree → stray commit `105497f` (D-458, STORY-172 wave-81, 2026-07-15), undetected
because both recovery and post-agent verification were worktree-scoped only.

**Dedup analysis:**
- **#655 (best match):** "resume prompt must include explicit worktree path + branch name +
  mandatory pre-commit branch assertion." Same incident *shape* — a resumed agent committing
  to the *wrong* location (in #655, a sibling story's worktree; in wirerust, the main
  develop checkout). #655's fix is *prevention* (pre-commit branch assertion + explicit
  worktree path in the dispatch/resume prompt).
- **#635 (compared, distinct):** convergence-streak persistence across a crash. Different
  subject (streak counter, not stray commits). NOT a duplicate.
- Adjacent: #293 (worktree-identity tuple in dispatch), #355 (parallel-worktree baseline
  contamination) — cross-ref.

**Why COMMENT not FILE-NEW:** #655 owns the wrong-location-commit class and the prevention-
side guard. wirerust adds a materially new *detection-side* dimension #655 lacks: the **main
develop checkout** is an always-present commit target that a worktree-only scan misses, so
both **session-recovery** and **post-agent verification** must enumerate `git worktree list`
*and* the main checkout. #655's pre-commit branch assertion would also have caught the
wirerust case, so it is the correct home; the comment extends its scope to detection.

**Draft comment (on #655):**

> **Detection-side extension: the main checkout is an always-present stray-commit target that
> worktree-only verification misses (wirerust feature-iec104, stray commit `105497f`, D-458).**
>
> Same wrong-location-commit class as this issue, with the target being the **main develop
> checkout** rather than a sibling worktree: a fix agent committed to the repo-root develop
> checkout instead of its assigned worktree (stray commit `105497f`, D-458, STORY-172
> wave-81, 2026-07-15). It went undetected because both (a) post-agent verification and
> (b) session-boundary recovery were worktree-scoped and never inspected the main checkout.
>
> Your pre-commit branch-assertion guard would have caught this at commit time — so this is
> the right home. Complementary *detection-side* addition: session-recovery and post-agent
> verification must enumerate **`git worktree list` plus the main checkout** and run
> `git -C <path> status` / `git -C <path> log` on *each* location (the main checkout is always
> present alongside any worktrees). A verification pass that checks only the active worktree
> is incomplete and must not be recorded as clean. Cross-ref #293 (worktree-identity tuple),
> #355 (parallel-worktree baseline contamination).

---

## Candidate 9 — validate-count-propagation regex parses "E-11 stories" as count "11 stories"

**Verdict: FILE-NEW** — `bug(hooks/validate-count-propagation)`. Confidence HIGH.

**Defect:** the `validate-count-propagation` hook parsed the epic label token "E-11 stories"
as a numeric count "11 stories," forcing a STATE.md rewording workaround (2026-07-19). This
is the same ID-like-substring false-match class as #641, but in a DIFFERENT hook, and a
DIFFERENT mechanism from the other known `validate-count-propagation` false-positive (#567).

**Dedup analysis:**
- **#641 (same class, different hook):** decision-chain-citation freshness hook false-
  positives on ID-like substrings ("RED-4", "TD-001"). Same *class* (bare `[A-Z]+-\d+`-shaped
  substring mis-parsed) but a *different hook* (decision-chain-citation, not count-propagation)
  and a different validated invariant. NOT a duplicate; cross-reference it.
- **#567 (same hook, different mechanism):** `validate-count-propagation`'s
  `count_propagation_drift` false-positives by comparing *historical* changelog counts against
  current counts. That is a historical-vs-current-count bug. wirerust's is a *tokenizer* bug —
  an epic identifier `E-11` mis-read as the number 11 in "E-11 stories." Same hook, different
  defect. NOT a duplicate; cross-reference it.

**Why FILE-NEW:** no existing issue covers this specific defect (ID/label token mis-parsed as
a count in `validate-count-propagation`). It sits at the intersection of #641's class and
#567's hook but is neither.

**Drafted issue:**

**Title:** `bug(hooks/validate-count-propagation): epic/ID label "E-11 stories" mis-parsed as count "11 stories" — ID-like substring false-positive forces STATE.md rewording`

**Body:**

> ## Summary
>
> The `validate-count-propagation` hook's count extractor mis-parses an **epic/identifier
> label** as a numeric count. The token "E-11 stories" (referring to epic E-11) was read as
> the count "11 stories," firing a count-propagation drift false-positive and forcing an
> operator to reword STATE.md to avoid the pattern rather than fixing a real inconsistency.
>
> ## Environment
>
> - vsdd-factory plugin; hook `validate-count-propagation`.
> - Observed on the wirerust project, STATE.md edit, 2026-07-19.
>
> ## Reproduction
>
> 1. Write a STATE.md line referencing an epic by ID next to the word "stories", e.g.
>    "9 gaps → 5 E-11 stories (12 pts)".
> 2. The count extractor matches `11` out of the `E-11` epic identifier and treats it as a
>    claimed count of "11 stories".
> 3. It compares that phantom `11` against the real story count and reports
>    count-propagation drift, even though every genuine count site is correct and mutually
>    consistent.
> 4. Workaround applied: reword the STATE.md prose so the `E-11` token is not adjacent to a
>    countable noun — a prose contortion to satisfy a false-positive.
>
> ## Root cause (inferred)
>
> The count scanner extracts digit runs adjacent to countable nouns without excluding
> digit runs that are part of an identifier token (`E-<n>`, epic/story IDs). `E-11` is an
> identifier, not a quantity; the trailing `11` is not a count.
>
> ## Why this is distinct from existing issues
>
> - **#567** — same hook (`validate-count-propagation`), but that is the *historical-vs-
>   current* count-comparison false-positive (frozen changelog counts flagged as drift).
>   This is a *tokenizer* false-positive: an identifier's digits read as a count. Different
>   mechanism, same hook.
> - **#641** — same *class* (an ID-like substring matched by a too-permissive pattern:
>   "RED-4", "TD-001"), but a *different hook* (decision-chain-citation freshness). This is
>   that class instantiated in `validate-count-propagation`.
>
> ## Proposed fix
>
> Anchor the count extractor to require the digit run be a standalone quantity, not part of
> an identifier: exclude matches immediately preceded by an identifier prefix (`E-`, `S-`,
> `STORY-`, `BC-`, `TD-`, `D-`, etc. — i.e., `[A-Za-z]+-` immediately before the digits), or
> require a word boundary that is not a hyphen preceded by a letter. Equivalently, tokenize
> and drop identifier tokens before counting. Same remedy family as #641 (anchor to the real
> pattern, don't match bare digit/ID shapes). Cross-ref #567 (sibling false-positive in the
> same hook), #641 (same ID-substring class, different hook).
>
> ## Provenance
>
> wirerust feature-iec104 cycle-close, STATE.md edit 2026-07-19. Validated per
> DF-VALIDATION-001.

---

## Candidate 10 — PG-HASH-HOOK-DIVERGENCE

**Verdict: DUPLICATE-of-#637 — NO ACTION.** Confidence HIGH.

**Dedup analysis:** #637's body *already documents PG-HASH-HOOK-DIVERGENCE by name* with the
exact wirerust wave-71 evidence — the `$(cat)` trailing-newline stripping root cause, the
`hooks/validate-input-hash.sh` hard-block (exit 2), and the three concrete divergences
(STORY-156 `ce96d86` vs `7b7dc6b`; STORY-150 `c5acbe4` vs `26416e1`; STORY-157 `357bca5` vs
`4a47ab6`) — identical to the CLAUDE.md record. wirerust has NO material new evidence beyond
what #637 already contains.

**Action:** none. Do not file, do not comment. This candidate originated as wirerust's own
upstream report (#637 already carries the wirerust field data). Confirmed fully covered.

---

## Candidate 11 — PG-GATE-VOCAB-BLINDSPOT (skeleton/seam tokens)

**Verdict: PRODUCT-LOCAL** (optional light COMMENT-on-#682 for the generic vocabulary point).
Confidence HIGH.

**Defect:** the `"skeleton"` / `"seam"` stub-era tokens are missing from wirerust's LOCAL
green-doc-tense gate (the `green-doc-tense-gate` job in wirerust's own `ci.yml`, established
by AC-174-008). Two STORY-174 adversary observations (P2 Obs-1, P4).

**Dedup analysis:**
- The green-doc-tense gate is a **wirerust-specific CI job** with a wirerust-maintained token
  list. Extending it with `skeleton`/`seam` is a wirerust product change (STORY-176
  AC-176-001, a develop-branch PR touching `ci.yml`). There is no engine artifact to change.
- **#682 (generic sibling):** the engine-level doc-integrity gate #682 proposes greps for
  RED-gate/`todo!()` markers ("Status: RED", "RED GATE:", "todo!() stub", "FAILS until") —
  not stub-era architecture vocabulary like `skeleton`/`seam`. The *specific tokens* are
  wirerust-local; the *idea that a doc-integrity gate's token list should include stub-era
  vocabulary* is a generic point that could be noted on #682.

**Why PRODUCT-LOCAL:** the fix lives in wirerust's own `ci.yml` and token list. Keep as
STORY-176 AC-176-001.

**Optional light comment (on #682), only if maintainer wants the generic vocabulary note:**

> If the engine ships a reference doc-integrity token list, consider including stub-era
> *architecture* vocabulary (`skeleton`, `seam`) alongside RED-gate/`todo!()` markers — these
> survive into green deliveries too. Field note: wirerust's local green-doc-tense gate missed
> both on STORY-174 (two adversary observations) until the token list was extended.

---

## Candidate 12 — PG-SPEC-VERSION-CITATION-CURRENCY

**Verdict: COMMENT-on-#396** (cross-ref #395, #471). Confidence HIGH.

**Defect:** spec-version bumps must include `src/` inline comments and `CHANGELOG.md` entries
in the citation-currency sweep set. Surfaced by F-172-301 NIT (D-454): a BC version bump left
stale version citations in `src/` comments / CHANGELOG uncovered by the existing sweep, which
scoped only `docs/` and `.factory/` artifacts.

**Dedup analysis:**
- **#396 (best match):** "full citation-corpus sweep on BC/ADR bump — changelog-row-only
  check misses 3–5 stale pins per bump." Proposes a full-corpus grep sweep after any BC/ADR
  version bump — but its scope is `.factory/specs/ .factory/stories/` (the spec tree). It does
  NOT include `src/` code comments or `CHANGELOG.md`. wirerust adds exactly that surface.
- **#395 (sibling, cross-ref):** test-file header/docstring version stamps not scanned —
  the `tests/` slice of the same problem.
- **#471 (cross-ref):** enforce a version floor on spec citations in test docstrings.
- **#622 (compared, distinct):** source-*line-number* citations with no baseline — a
  different citation kind (line numbers, not version pins); #622 itself cross-refs #396 as
  the spec-version-citation class. NOT this.
- **#687 (compared, distinct):** spec-document *internal* citation coherence (AC-body vs
  obligations-table). Not the src/CHANGELOG surface.

**Why COMMENT not FILE-NEW:** wirerust extends #396's "full-corpus sweep" corpus beyond the
`.factory/` tree to the code tree (`src/` comments + `CHANGELOG.md`). That is a material
scope addition to #396, not a new class.

**Draft comment (on #396):**

> **Extend the post-bump sweep corpus beyond `.factory/` to `src/` comments and CHANGELOG
> (wirerust feature-iec104, F-172-301 / D-454).**
>
> This issue scopes the full-corpus sweep to `.factory/specs/ .factory/stories/`. We hit a
> stale pin *outside* that corpus: on a BC version bump during STORY-172 F4 delivery, `src/`
> inline doc comments and a `CHANGELOG.md` entry that cited the old BC version went stale and
> were not covered by the sweep (caught late as F-172-301, D-454). Suggest adding `src/`
> (inline code comments citing spec versions) and `CHANGELOG.md` to the grep corpus, e.g.
> `grep -rn "BC-X.YY.ZZZ v" src/ CHANGELOG.md` alongside the `.factory/` sweep. Cross-ref
> #395 (the `tests/` header/docstring slice of this same class) and #471 (version-floor
> enforcement on test-docstring citations).

---

## Inconclusive / flags for the team lead

- **Candidate 1 & 6 verdicts are COMMENT with a defensible FILE-NEW alternative.** If the
  maintainer prefers standalone tracking of (1) the demo-recorder serde-JSON derivation gate
  or (6) the implementer pre-coding AC↔BC fidelity gate, escalate those two to FILE-NEW. The
  drafts above are written so the comment bodies convert cleanly to issue bodies.
- **#656 title/body mismatch (informational):** the upstream list titles #656 as "relocation
  stories must include a BC Source citation sweep task — 9 stale citations," but the fetched
  body is about pinned function signatures exceeding `clippy::too_many_arguments`. Neither
  matches any candidate, so it does not affect verdicts — but the divergence is worth a
  heads-up to whoever files, in case #656 was retitled/edited upstream.
- **No candidate produced a clean single-issue DUPLICATE except #10 (#637) and #3 (#457).**
  All others required the multi-issue comment/cross-ref treatment because the upstream tracker
  decomposes these classes finely across many adjacent issues.

---

## REDACTED DRAFTS FOR PUBLIC POSTING (approved 2026-07-19)

Redaction applied per team-lead direction: internal decision IDs, story/AC IDs, wave/pass/
cycle names, PG-codenames, PR numbers, and repo-specific file paths are stripped or
genericized. Preserved: occurrence counts, concrete mechanisms, failure scenarios, proposed
remediations, upstream (`drbothen/vsdd-factory`) issue cross-references, and the framing that
evidence comes from a downstream VSDD-factory-managed project. Product-specific type names
(enum/struct/type identifiers) and MITRE technique IDs are genericized to their mechanism
class, since they leak the downstream product domain without adding evidentiary value.

### REDACTED comment — target #494 (candidate 1)

> **Additional field data — structured-JSON / serde enum-variant fabrication (downstream
> VSDD-factory-managed project, 3 occurrences).**
>
> Same root cause as this issue's Axis A (evidence text not generated from real execution),
> in a non-interactive CLI/library product where the "evidence text" is a serialized-JSON
> block rather than an interactive signal trace. Across one feature cycle:
> - 1 MEDIUM: demo JSON carried an enum variant that does not exist in the shipped enum.
> - 1 HIGH, replicated across 3 demo-evidence artifacts: two more non-existent enum variants,
>   plus a field emitted in the wrong case (the shipped type serializes lowercase under
>   `#[serde(rename_all="lowercase")]`), plus a wrong technique ID in the accompanying prose.
>   The illustrative demo `.rs` did not even compile.
> - Cost: the feature code had already converged, yet several subsequent adversarial passes
>   were consumed chasing fabricated demo JSON rather than real defects.
>
> **Why this manifestation is mechanically gate-able (a remedy beyond "attach raw output"):**
> for structured output, every enum-carrying JSON field can be (a) captured from real
> `cargo run -- … --format json` / `cargo test -- --nocapture` stdout, and (b) validated
> against the serde-serialized form and the variant set declared in the source. Hand-authored
> illustrative JSON should be prohibited for any product with a real serialization path.
> Suggest the demo-recording skill gain an explicit "derive JSON from captured stdout +
> spot-check ≥1 enum-carrying field against source" step. Cross-ref #636 (path-scrub sibling
> in the same protocol), #586 (fabricated tool surfaces).

**Redacted from original:** wave/pass/round labels and the in-cycle finding IDs → "one
feature cycle" / plain counts; the decision-log root-cause ID → dropped; product type names
(threat-category / verdict / tactic enums) → "the shipped enum"; the specific MITRE technique
→ "a wrong technique ID". Kept: 3 occurrences, HIGH×3-artifacts, the serde/`rename_all`
mechanism, the cargo capture + variant-validation remedy, cross-refs #636/#586.

### REDACTED comment — target #461 (candidate 2)

> **Confirming + extending field data (downstream VSDD-factory-managed project).**
>
> This ceiling is broader than the peer-teammate case. We confirmed that *no* agent principal
> can bypass the merge gate:
> 1. Subagent `--admin` merge on orchestrator-relayed human consent — denied.
> 2. **Orchestrator-direct `--admin` merge (not via a subagent) — also denied.**
> 3. A **named `--admin` bypass — also denied.**
>
> Resolution path, reconfirmed across three separate merge deliveries (including two on
> consecutive days): **human-direct merge in the main thread** is the only valid execution
> when the merge-auth classifier conditions are unmet. The classifier halt is *correct
> behavior*, not a bug.
>
> This supports your "document the known ceiling" proposal — but the doc should state the
> ceiling applies to the orchestrator acting directly, not only to peer teammates, so
> operators stop attempting an orchestrator-direct `--admin` as a fallback. Related but
> distinct: #350 (relayed signed *commits*), #380/#269 (relayed *approval* trust). This
> issue is specifically the `gh pr merge` execution ceiling.

**Redacted from original:** decision-log ID and the story/wave label → dropped; the two PR
numbers → "two on consecutive days"; explicit dates → dropped. Kept: all three denial
mechanisms, the human-direct resolution, "reconfirmed across three deliveries", the
classifier-halt-is-correct point, cross-refs #350/#380/#269.

### REDACTED comment — target #686 (candidate 4)

> **Third facet of reviewer calibration divergence: severity-vs-code-freeze-state (downstream
> VSDD-factory-managed project).**
>
> Beyond technique-depth and policy-trigger drift, we observed severity divergence on *frozen
> code*: during late adversarial passes on a delivered story, production code that had been
> unchanged since an early pass was reviewed again by fresh instances — one rated findings
> MEDIUM, another rated equivalent findings LOW/advisory. The reconciliation overhead blurred
> whether a pass was genuinely CLEAN.
>
> Proposed calibration rule (complements #344's counter treatment and #579's reopening rule):
> a finding against code that has been byte-frozen since a *named* earlier pass is a
> retrospective re-assessment, not a forward regression scan; it should be capped at **LOW/
> advisory unless the reviewer demonstrates an observable behavioral regression at current
> HEAD.** If behavior is unchanged since the freeze point, the finding was equally reportable
> in the earlier (accepted/deferred) pass, so MEDIUM+ is not appropriate. Dispatches could
> carry a neutral "code frozen since Pass N" fact (the same shape as your policy-timing
> sequencing-fact suggestion) so fresh reviewers calibrate consistently. Cross-ref #344, #579.

**Redacted from original:** story ID and the explicit pass-range (P9–P14 / "since Pass 2") →
"a delivered story" / "an early pass". Kept: the two-instance MEDIUM-vs-LOW divergence
mechanism, the frozen-code severity-ceiling proposal, cross-refs #344/#579. ("Pass N" retained
only as a generic variable in the proposed rule.)

### REDACTED comment — target #682 (candidate 5)

> **Generalize the pre-Red→Green sweep to a mandatory pre-adversarial doc-currency step —
> field data: 12 of 17 passes were doc-drift (downstream VSDD-factory-managed project).**
>
> On one story the code converged early, then took many more passes; post-analysis showed
> **12 of 17 passes were driven by doc-accuracy findings** — stale code comments, test-header
> prose citing earlier spec versions, leftover implementation-pass TODOs — not behavioral
> findings. This is item 2 of this issue widened past RED-gate markers: any comment/test-
> header referencing an AC/BC/field/version that changed during delivery is the same class.
>
> Suggest the implementer workflow gain a named **pre-adversarial-dispatch doc sweep** (run
> after CI is green, before the adversary is dispatched) covering source-tree inline comments
> and test headers/docstrings for stale spec-version/name references and leftover TODO/FIXME.
> Recording "doc sweep: PASS" before dispatch drains the trickle before the convergence clock
> starts — the same front-load logic #687 Gap 2 proposes for spec artifacts. Cross-ref
> #687 (spec-artifact sibling), #395 (test-header version stamps), #344 (why the residual
> stream stalls the counter).

**Redacted from original:** story ID and "by Pass 2" → "one story" / "converged early";
`src/`/`tests/` literal paths → "source-tree" / "test headers". Kept: the 12-of-17 count
(explicitly preserved per instruction), the mechanism, the pre-adversarial-step remedy,
cross-refs #687/#395/#344.

### REDACTED comment — target #305 (candidate 6)

> **Field data + an implementer-side last-line-of-defense proposal (downstream VSDD-factory-
> managed project, 4 occurrences in one feature cycle).**
>
> Four deliveries hit AC↔BC drift that the decomposition/authoring gates did not catch; each
> was rescued by ad-hoc BC-realignment *at coding time*:
> - A struct was renamed in the BC, but the story AC kept the old flat field layout and a
>   wrong minimum-length guard (API-name drift, your defect #2).
> - An AC specified a false-positive detection where the BC mandates *no* finding (an
>   emit-vs-no-emit polarity inversion — cross-ref #669), plus a confidence level bumped one
>   tier above what the BC allows.
> - A field was cited under a name that does not exist anywhere in the code (symbol absent —
>   cross-ref #573), alongside overflow semantics contradicting the BC.
> - A string literal was used where the code requires an enum variant — a **compilation
>   blocker** on first implementation (symbol/type mismatch — cross-ref #573).
>
> Because all four surfaced only at coding time, we propose an **implementer pre-coding AC↔BC
> fidelity check as an explicit gate step before any code is written**: the implementer
> produces a written table mapping each AC to the *current* version of its traced BC and
> confirms field names, guards/conditions, confidence/verdict levels, and emit/no-emit
> decisions match; discrepancies force BC-realignment before coding. This complements the
> decomposition-time ledger / dependency gate you propose — it's the last line of defense
> when the upstream gates miss. Cross-ref #400 (PC-level trace table), #327 (sub-anchor
> resolution lint), #573 (normative-AC symbol/signature verification), #669 (polarity).

**Redacted from original:** the four story IDs → "four deliveries"; product type/field names
(the struct rename, the flow-key rename, the detection-technique ID, the tactic enum) →
described by mechanism class only; "F3→F4 gate" → "a gate step before any code is written"
(dropped the phase labels). Kept: 4 occurrences + one compilation blocker, each mechanism, the
implementer pre-coding fidelity-table remedy, cross-refs #400/#327/#573/#669.

### REDACTED comment — target #655 (candidate 8)

> **Detection-side extension: the main checkout is an always-present stray-commit target that
> worktree-only verification misses (downstream VSDD-factory-managed project).**
>
> Same wrong-location-commit class as this issue, with the target being the **main checkout**
> rather than a sibling worktree: a fix agent committed to the repo-root checkout instead of
> its assigned worktree, producing a stray commit. It went undetected because both (a)
> post-agent verification and (b) session-boundary recovery were worktree-scoped and never
> inspected the main checkout.
>
> Your pre-commit branch-assertion guard would have caught this at commit time — so this is
> the right home. Complementary *detection-side* addition: session-recovery and post-agent
> verification must enumerate **`git worktree list` plus the main checkout** and run
> `git -C <path> status` / `git -C <path> log` on *each* location (the main checkout is always
> present alongside any worktrees). A verification pass that checks only the active worktree
> is incomplete and must not be recorded as clean. Cross-ref #293 (worktree-identity tuple),
> #355 (parallel-worktree baseline contamination).

**Redacted from original:** the stray commit SHA → "a stray commit"; decision-log ID, story
ID, wave label, and date → dropped ("a fix agent"). Kept: the commit-to-main-checkout
mechanism, the worktree-only-blindspot failure scenario, the `git worktree list` + main-checkout
detection remedy, cross-refs #293/#355.

### REDACTED comment — target #396 (candidate 12)

> **Extend the post-bump sweep corpus beyond the spec tree to source comments and CHANGELOG
> (downstream VSDD-factory-managed project).**
>
> This issue scopes the full-corpus sweep to the `.factory/` spec/story tree. We hit a stale
> pin *outside* that corpus: on a BC version bump during a story's delivery, source-tree
> inline doc comments and a CHANGELOG entry that cited the old BC version went stale and were
> not covered by the sweep — caught late in adversarial review as a low-severity nit. Suggest
> adding the source tree (inline code comments citing spec versions) and CHANGELOG to the grep
> corpus, e.g. `grep -rn "BC-X.YY.ZZZ v" src/ CHANGELOG.md` alongside the `.factory/` sweep.
> Cross-ref #395 (the test-header/docstring slice of this same class) and #471 (version-floor
> enforcement on test-docstring citations).

**Redacted from original:** the finding ID and decision-log ID → "caught late in adversarial
review as a low-severity nit"; story ID → "a story's delivery". Kept: the src/+CHANGELOG
corpus-extension mechanism and the representative grep (with generic `BC-X.YY.ZZZ` placeholder),
cross-refs #395/#471. (`.factory/`, `src/`, `CHANGELOG.md` retained — engine/repo-generic
paths, not project-identifying.)

---

### #690 verdict — NOT clean; redacted replacement body provided

Issue #690 (the already-posted candidate-9 bug) **contains internal identifiers** and needs a
body edit before it is public-safe: it names the downstream project twice ("the wirerust
project", "wirerust feature-iec104 cycle-close") and carries the feature-cycle codename and
observation date in the provenance line. Per instruction, the literal **"E-11 stories" trigger
string is retained** (it is the bug repro). Replacement body below.

**REDACTED #690 BODY:**

> ## Summary
>
> The `validate-count-propagation` hook's count extractor mis-parses an **epic/identifier
> label** as a numeric count. The token "E-11 stories" (referring to an epic whose ID is
> `E-11`) was read as the count "11 stories," firing a count-propagation drift false-positive
> and forcing an operator to reword STATE.md to avoid the pattern rather than fixing a real
> inconsistency.
>
> ## Environment
>
> - vsdd-factory plugin; hook `validate-count-propagation`.
> - Observed on a downstream VSDD-factory-managed project during a STATE.md edit.
>
> ## Reproduction
>
> 1. Write a STATE.md line referencing an epic by ID next to the word "stories", e.g.
>    "… → 5 E-11 stories …".
> 2. The count extractor matches `11` out of the `E-11` epic identifier and treats it as a
>    claimed count of "11 stories".
> 3. It compares that phantom `11` against the real story count and reports
>    count-propagation drift, even though every genuine count site is correct and mutually
>    consistent.
> 4. Workaround applied: reword the STATE.md prose so the `E-11` token is not adjacent to a
>    countable noun — a prose contortion to satisfy a false-positive.
>
> ## Root cause (inferred)
>
> The count scanner extracts digit runs adjacent to countable nouns without excluding
> digit runs that are part of an identifier token (`E-<n>`, epic/story IDs). `E-11` is an
> identifier, not a quantity; the trailing `11` is not a count.
>
> ## Why this is distinct from existing issues
>
> - **#567** — same hook (`validate-count-propagation`), but that is the *historical-vs-
>   current* count-comparison false-positive (frozen changelog counts flagged as drift).
>   This is a *tokenizer* false-positive: an identifier's digits read as a count. Different
>   mechanism, same hook.
> - **#641** — same *class* (an ID-like substring matched by a too-permissive pattern:
>   "RED-4", "TD-001"), but a *different hook* (decision-chain-citation freshness). This is
>   that class instantiated in `validate-count-propagation`.
>
> ## Proposed fix
>
> Anchor the count extractor to require the digit run be a standalone quantity, not part of
> an identifier: exclude matches immediately preceded by an identifier prefix (`E-`, `S-`,
> `STORY-`, `BC-`, `TD-`, `D-`, etc. — i.e., `[A-Za-z]+-` immediately before the digits), or
> require a word boundary that is not a hyphen preceded by a letter. Equivalently, tokenize
> and drop identifier tokens before counting. Same remedy family as #641 (anchor to the real
> pattern, don't match bare digit/ID shapes). Cross-ref #567 (sibling false-positive in the
> same hook), #641 (same ID-substring class, different hook).
>
> ## Provenance
>
> Observed during a downstream project's feature cycle-close, on a STATE.md edit. Validated
> per the downstream project's finding-validation gate before filing.

**Redacted from #690:** "the wirerust project" → "a downstream VSDD-factory-managed project";
"wirerust feature-iec104 cycle-close" + the `DF-VALIDATION-001` policy name + observation date
→ generic downstream-project / feature-cycle-close / "finding-validation gate" phrasing; the
illustrative repro line's specific counts (`9 gaps → 5 … (12 pts)`) → neutral `… → 5 E-11
stories …`. **Retained:** the "E-11 stories" trigger string (bug repro), the tokenizer
mechanism, the STATE.md artifact name (engine-generic), and cross-refs #567/#641.
