# Lessons Learned — feature-iec104 Cycle-Close

S-7.02 cycle-closing requirement: lessons recorded here after feature-iec104 RELEASED (v0.13.0,
D-473, 2026-07-18). All process-gaps codified into E-11 follow-up stories STORY-175..179.

Cycle: feature-iec104 | CLOSED: 2026-07-18 (D-475) | Stories: STORY-167..174 (8 stories, 36 pts, waves 76–83).
PRs merged: #401..409 (STORY-167..174), #410 (FIX-P4-001), #411..415 (FIX-F5-001..004), #416 (E2E coverage),
#417 (v0.13.0 release), #418 (back-merge to develop), #419 (IEC104-DEMO-TYPEID45-MISLABEL doc fix).
F5 scoped adversarial: 5 rounds (CONVERGED D-468). F6 targeted hardening: PASS (D-469).
F7 delta convergence: CONVERGED 5/5 dims holdout 0.99 (D-470).
Process-gaps codified: 9 gaps → 5 E-11 draft stories STORY-175..179 (12 pts; D-475, 2026-07-18).

---

## Lesson 1 — [codified→STORY-175] Demo-Recorder Must Derive JSON from Real Serialized Output

**Observation:**

The demo-recorder produced illustrative JSON/enum values by hand rather than deriving from
actual `cargo run`/`cargo test` serialization during feature-iec104. Three independent
fabricated-demo-JSON occurrences (PG-DEMO-JSON-FABRICATION):

- **FIX-F5-001 R2 (F5R2-02 MEDIUM):** T0881 JSON fragment contained non-existent `tactic`
  enum variant.
- **FIX-P4-001 R3 findings F-B1 HIGH (×3 artifacts):** `category: "Protocol"`,
  `verdict: "Anomaly"`, `confidence: "High"` — non-existent enum variants, non-compiling
  demo `.rs`, wrong MITRE technique.

Root cause: hand-written JSON bypasses the serde serialization path and can diverge from
`rename_all` rules and actual variant names without triggering a compile-time error.
F5 adversarial review (Rounds 2–5) was significantly extended by chasing down fabricated
demo-evidence rather than real code defects.

**Codification vehicle:** STORY-175 — mandatory demo-JSON derivation rule: demo-recorder
MUST generate JSON evidence by capturing actual `cargo run`/`cargo test` stdout; illustrative
JSON written by hand is prohibited; enum variants MUST be verified against `src/` source
before inclusion in any demo-evidence artifact.

**Tags:** `codified`, `demo-json-fabrication`, `serde`, `enum-variants`, `recurring`

---

## Lesson 2 — [codified→STORY-176] Adversarial Gate Vocabulary, Doc-Currency Sweep, Severity Calibration

**Observation:**

Three related adversarial-process gaps accumulated across feature-iec104:

**PG-GATE-VOCAB-BLINDSPOT:** The green-doc-tense gate (AC-174-008) missed `"skeleton"` and
`"seam"` phrasing — stub-era language surviving into green deliveries. Two independent
adversary observations in STORY-174 (P2 Obs-1 + P4 obs). The token list must be extended
to cover at least `skeleton`, `seam`, and equivalent stub-era vocabulary.

**PG-DOC-CURRENCY-SWEEP:** Post-adversarial doc-accuracy drift consumed 12 of 17 STORY-173
adversary passes. FIX-F5-002/003/004 rounds (F5 R3–R5) were entirely doc-accuracy remediation
with no new code findings. A mandatory pre-adversary doc sweep on code comments and test
headers would have absorbed this tail before the adversary pass clock started.

**PG-ADVERSARY-SEVERITY-CALIBRATION:** At late adversary passes with code frozen since P2,
instances diverged on severity calibration — raising findings against code that had not changed.
Whole-source doc sweeps generated advisory findings against frozen code, blurring signal-to-
noise. Adversary instances must re-confirm code-frozen status before escalating severity on
a surface unchanged since a prior clean pass.

**Codification vehicle:** STORY-176 — (1) extend AC-174-008 green-doc-tense token list to
include stub-era vocabulary; (2) add mandatory pre-adversary doc-sweep gate step to all
feature delivery checklists; (3) add adversary severity-calibration guidance for code-frozen
surfaces.

**Tags:** `codified`, `gate-vocab-blindspot`, `doc-currency-sweep`, `adversary-calibration`, `recurring`

---

## Lesson 3 — [codified→STORY-177] Subagent Merge Authorization and Agent Idle-No-Report

**Observation:**

Two agent-behavior process-gaps codified together as a single governance AC:

**PG-MERGE-AUTH-SUBAGENT-CLASSIFIER:** Subagent cannot execute `--admin` merge on relayed
human consent; orchestrator-direct attempt also denied on unnamed `--admin` bypass. The
resolution path established at D-463 is human-direct merge in the main thread. This pattern
recurred identically at PR #419 (IEC104-DEMO-TYPEID45-MISLABEL fix, 2026-07-18): step-8
halt per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER, human-direct merge required. The rule is not
story-specific — it applies to any subagent in any role dispatched by the orchestrator.

**PG-ADVERSARY-IDLE-NO-REPORT (made agent-generic, 2026-07-18):** Adversary agents completing
CLEAN passes sometimes emitted no report, making CLEAN vs idle indistinguishable. A fresh
occurrence arose on 2026-07-18: a spec-steward agent was dispatched and completed work
silently without emitting a completion report. This confirmed the gap is not adversary-specific
— it applies to ALL dispatched agents. Any dispatched agent MUST emit a structured completion
report (CLEAN/CONVERGED/OK or findings list) regardless of pass outcome; silence is never
a valid result.

**Codification vehicle:** STORY-177 — (1) codify PG-MERGE-AUTH-SUBAGENT-CLASSIFIER as a
standing policy AC: subagent `--admin` merge on relayed consent is denied; resolution =
human-direct in main thread; (2) extend PG-ADVERSARY-IDLE-NO-REPORT to an agent-generic
mandatory-report rule covering all dispatched agent types.

**Tags:** `codified`, `merge-auth`, `subagent-classifier`, `idle-no-report`, `agent-generic`, `recurring`

---

## Lesson 4 — [codified→STORY-178] F3 Decomposition BC Fidelity, Spec-Version Currency, and Infrastructure Gaps

**Observation:**

Four distinct F3-decomposition fidelity failures occurred in feature-iec104
(F3-DECOMPOSITION-BC-FIDELITY):

- **STORY-169:** Flat vs broken-out fields; wrong guard conditions.
- **STORY-170:** False-positive T0827; confidence Possible→Likely; reserved-TypeID scope;
  naming divergence.
- **STORY-172:** FlowId→FlowKey nonexistent; carry-overflow discard-all-new semantics;
  malformed-LEN PC4 contradiction.
- **STORY-173:** T0881 tactic string `"impact"` → `MitreTactic` enum; compilation blocker.

All four required BC-realignment before delivery could proceed. A mandatory AC↔BC fidelity
check as a formal F3→F4 gate step (pre-coding checklist item) would have caught each before
implementation began.

Additionally (PG-SPEC-VERSION-CITATION-CURRENCY): spec-version bumps during STORY-172 F4
delivery did not propagate simultaneously to `src/` code comments and CHANGELOG entries,
creating citation-currency drift caught by adversary at a later pass (F-172-301 NIT, D-454).

Two infrastructure gaps also addressed here:

**input-hash-self-referential-drift:** STORY-175..179 input hashes will drift in future
sessions because their spec inputs (PRD + BCs) are being edited in this same cycle-close
commit. Hashes must be recomputed after the cycle-close commit and all downstream BC edits
have settled. This is expected and documented; the re-baseline must not be skipped.

**gitignore-mutants-glob:** `mutants.out/` and `mutants.out.j4-invalid/` directories appeared
as untracked residue in the develop checkout (cargo-mutants output). A `.gitignore` glob
for `mutants.out*/` is needed in the develop tree to prevent these from accumulating across
sessions.

**Codification vehicle:** STORY-178 — (1) mandatory pre-delivery AC↔BC fidelity check as
F3→F4 gate step; (2) spec-version-bump protocol requiring simultaneous src/ comment + CHANGELOG
currency sweep; (3) `.gitignore` glob for `mutants.out*/`.

**Tags:** `codified`, `f3-decomposition-fidelity`, `spec-version-currency`, `input-hash-drift`, `gitignore`, `recurring`

---

## Lesson 5 — [codified→STORY-179] Session-Boundary State Recovery Must Cover All Worktrees

**Observation:**

Two coupled gaps emerged from the same root event during feature-iec104 (PG-STATE-RECOVERY-SCOPE
+ PG-VERIFY-ALL-WORKTREES):

A fix agent committed a change to the main develop checkout rather than its assigned worktree,
creating stray commit `105497f`. This commit had to be discarded (`git reset --hard` to the
pre-commit HEAD). Root gap: session-boundary state recovery verified the story worktree only
and did not confirm the main develop checkout was also in a clean, known state.

Coupled gap: post-agent verification was similarly worktree-scoped. A change committed to
the wrong checkout cannot be detected unless verification explicitly spans all worktrees.
`git worktree list` is not sufficient; each listed worktree requires an independent
`git status` pass.

**Codification vehicle:** STORY-179 — (1) session-boundary state recovery checklist MUST
include an explicit `git status` check on the main develop checkout alongside all story
worktrees; (2) post-agent verification MUST span ALL worktrees via `git worktree list` +
per-worktree `git status`; (3) recovery protocol MUST name the expected SHA at each worktree
HEAD and verify it matches before declaring a session clean.

**Tags:** `codified`, `state-recovery-scope`, `verify-all-worktrees`, `stray-commit`, `multi-worktree`
