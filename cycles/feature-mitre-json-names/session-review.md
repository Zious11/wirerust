---
document_type: session-review
cycle_id: feature-mitre-json-names
released_version: v0.9.4
develop_head_at_close: 760b6ca
reviewer: session-reviewer (adversary model — independent perspective)
produced_at: "2026-06-23"
pipeline_path: FEATURE_MODE F1→F7 (full cycle; preamble includes issue triage + maintenance resumption)
stories_in_cycle: 1  # STORY-129
prs_delivered: 3  # PR #306 (F4 story), PR #307 (F5 ICS catalog fix), PR #308 (F7 docs)
baseline_cycles:
  - feature-arp-v0.7.0 (2026-06-16)
  - feature-story-119-grouped-collapse (2026-06-19)
  - feature-pcapng-reader (2026-06-22)
---

# Session Review — feature-mitre-json-names (issue #64 + ICS catalog fix → v0.9.4)

**Cycle closed:** 2026-06-23 (D-216, human-authorized). Feature: inline MITRE tactic/name in JSON
output (`mitre_attack` per-finding array, BC-2.11.035); bonus: ICS tactic-catalog correctness fix
(3 new `MitreTactic` variants, 5 technique remaps). F1–F7 all converged. develop=760b6ca.
Release v0.9.4 in progress.

---

## 1. Session Arc and Phase Timeline

### Preamble (D-203 → D-205): Resumption, Issue Triage, Maintenance

The session opened on a fully quiesced pipeline (v0.9.3 released, three D-200-era decision threads
closed, D-203 safe-to-clear checkpoint written). Before entering feature mode, the orchestrator
ran maintenance sweep maint-2026-06-22 and triaged 10 open GitHub issues:

- 10 issues assessed against codebase + research-agent validation (policy DF-VALIDATION-001)
- PR #304: deps hygiene (rayon removed, rand→0.8.6 clears RUSTSEC-2026-0097, zerocopy bump)
- PR #305: docs drift + public ADR-0009 (e4abbe2)
- F-MAJ-001 fixed: ARCH-INDEX updated to v1.6 (BC count corrections)
- Issue #6 identified as obsolete; issue #4 as partially-done
- Issue #64 selected as next feature target

### F1 — Delta Analysis (D-206)

Single-story scope confirmed. Architect produced F1 delta analysis; initial design was flat fields
(`mitre_tactic`, `mitre_name` from `mitre_techniques[0]`). **Research-agent override:** array-of-objects
design adopted (`mitre_attack` array per finding, each element carrying id/name/tactic_id/tactic_name/reference
for ALL techniques). ECS/OCSF alignment justification. Human approved `mitre_attack` field name and array design.

### F2 — Spec Evolution (D-206)

BC-2.11.035 v1.0 authored (10 ACs, EC-001..010). BC-INDEX v1.70; PRD v1.34; interface-definitions v1.3;
BC-2.11.001 v1.7. No new VP (test-sufficient classification, justified by VP-007 coverage of underlying
catalog functions).

### F3 — Story Decomposition (D-206)

STORY-129 authored (Wave 57, 5 pts, input-hash 2a5cee9). STORY-INDEX v2.7 (82 stories / 57 waves / 526 pts).

### F4 — TDD Implementation + Per-Story Convergence (D-207 / D-208)

| Metric | Value |
|--------|-------|
| Implementation commits | b8fea97 → 6d8f172 → 7e020ce (+ demo 2b10298) |
| Tests at convergence | 13 (10 AC + 3 EC: EC-001..010 fully covered) |
| Adversary passes | 3 (b8fea97 / 6d8f172 / 7e020ce) |
| Finding trajectory | 3L → 1M+1L → 1L+1process-gap |
| Open HIGH/CRIT at gate | 0 |
| PR | #306 MERGED → develop 2fa6606 (human-merged, squash disabled) |
| Issue #64 status | CLOSED |

Notable findings caught: F-3 (Pass 1) — EC-009/010 not covered by dedicated tests; M-1 (Pass 2) —
EC-008 mixed-batch scenario untested. Both fixed before final pass. Process gap DRIFT-BC-TEMPLATE-EC-VP-MAP-001
deferred (BC template permits EC rows without VP table counterparts — engine concern, not product defect).

### F5 — Scoped Adversarial + ICS Catalog Fix (D-209 → D-212)

**HIGH finding F-1:** ICS techniques (T0888, T0830, T0846, T0885) were emitting Enterprise-matrix
tactic IDs (TA0007/TA0008/TA0011) while the envelope declared `mitre_domain: "ics-attack"`.
Research-validated against MITRE ATT&CK ICS v19.1 (5 direct attack.mitre.org page fetches).

**Second latent bug surfaced by research agent:** T0830 ("Adversary-in-the-Middle") was mapped to
`LateralMovement` — wrong both in domain (Enterprise TA0008) AND in tactic name. MITRE ICS matrix
assigns T0830 to Collection (TA0100), not Lateral Movement. This bug predated the feature; the feature
exposed it by requiring the new per-finding `tactic_id` field to be correct.

**Comprehensive catalog fix (D-209):**
- 3 new `MitreTactic` variants: IcsDiscovery (TA0102), IcsCollection (TA0100), IcsCommandAndControl (TA0101)
- 5 technique remaps: T0846/T0888 → IcsDiscovery; T0885 → IcsCommandAndControl; T0830 → IcsCollection; T0831 kept (IcsImpact already correct)
- 5 BCs bumped (BC-2.10.002/003/007, BC-2.11.035, BC-2.16.004)
- 3 holdout files corrected (wave-31, wave-40-44, HS-INDEX)
- Authoritative-pin test added: `test_ics_techniques_resolve_authoritative_tactic_ids` (12 exact id→TA-id pairs)

**F5 per-fix adversarial convergence (D-210):** 3 passes (74a48ea / cf22de9 / cf22de9).

| Pass | HEAD | Findings | Verdict |
|------|------|----------|---------|
| 1 | 74a48ea | 1 LOW (stale T0830 comment) + 1 process-gap (no TA-id pin test) | Fix |
| 2 | cf22de9 | 2 LOW (cosmetic grouping; no ARP fixture) | Non-blocking |
| 3 | cf22de9 | 0 novel; all 20 TA-ids re-verified | CONVERGED |

**DRIFT-UNCOMMITTED-TEST-EDITS-001 (D-211):** The implementer committed only `src/mitre.rs` (719816e),
leaving 3 test file corrections as uncommitted working-tree edits. Adversary passes 1–3 reviewed the
working tree (green), while committed SHAs carried old wrong assertions. CI caught on push. pr-manager
committed corrections as 96f0afc; final merged state correct.

PR #307 MERGED → develop 029725b (D-212).

### F6 — Targeted Hardening (D-213)

| Task | Result |
|------|--------|
| Formal (VP-007 Kani re-run) | PASS — 4/4 harnesses VERIFICATION SUCCESSFUL |
| Mutation (json_dto.rs + mitre.rs, cargo-mutants) | PASS — 49/53 viable killed (92.5%); 4 "survivors" are #[cfg(kani)] harness bodies = Kani-verified FPs; 0 production-logic gaps |
| Fuzz (fuzz_decode_packet regression) | PASS — 5.84M runs/91s, 0 crashes; mitre_attack path panic-free by construction |
| Security (cargo audit + cargo deny) | PASS — 0 vulnerabilities, 0 advisory warnings |
| Full regression | PASS — cargo test --all-targets green, 0 failures |

No new VP warranted. DTO logic is pure Option-chaining over the already Kani-verified VP-007 catalog.
`FindingJsonDto::from` has no `.unwrap()`, no indexing, no arithmetic — exhaustive type-level totality.

### F7 — Delta-Convergence + Sibling Sweep (D-214 → D-216)

**Fresh-context consistency audit (D-214):** All core code/tests/BCs fully consistent. Three documentation
gaps found and fixed:
- F7-CV-001 (MEDIUM): README ARP table Tactic column showed technique name instead of tactic for T0830/T1557.002
- F7-CV-002 (LOW): STORY-129 Architecture Mapping table stale "17 variants" → "20"
- F7-CV-003 (LOW): Historical design doc cited TA0111 (wrong ICS Discovery ID) + SUPERSEDED banner missing

**DF-SIBLING-SWEEP-001 gap (D-215):** The F5 BC update propagated the 17→20 variant count to 5 BCs
(BC-2.10.002/003/007, BC-2.11.035, BC-2.16.004) but did NOT sweep the 10 sibling spec layers:
vp-016 v2.6, BC-2.10.004 v1.6, cap-10 v2.0, cap-11 v1.3, ent-04 v1.4, ent-05 v1.2, nfr-catalog v2.4,
test-vectors v2.3, prd.md v1.35, module-criticality v1.5. All still asserted "17 variants / 14 Enterprise
+ 3 ICS." Caught only by an orchestrator-run wide grep for the magic number "17" across all spec layers.
All 10 updated in the D-215 sibling sweep. Input-hashes recomputed.

PR #308 MERGED → develop 760b6ca (D-216). Cycle CONVERGED across all 5 dimensions.

---

## 2. Outcome Summary

| Metric | Value |
|--------|-------|
| Stories delivered | 1 (STORY-129) |
| Total project stories | 78 |
| New BCs | 1 (BC-2.11.035 v1.0→v1.1 by F5) |
| BCs modified | 5 bumped for ICS catalog fix |
| Active BCs | 303 |
| New MitreTactic variants | 3 (IcsDiscovery, IcsCollection, IcsCommandAndControl) |
| ICS techniques remapped | 5 (T0846, T0888, T0885, T0830; T0831 already correct but comment fixed) |
| VPs added | 0 (test-sufficient) |
| PRs merged | 3 (#306 feature, #307 ICS fix, #308 docs) |
| Release version | v0.9.4 (in progress) |
| Open bugs found and shipped | 0 (all found defects remediated before merge) |
| Pre-existing latent bugs fixed | 1 HIGH (ICS tactic-id correctness) + 1 latent (T0830 wrong tactic name) |

---

## 3. What Went Well

### 3.1 Research-Agent Design Override (F1)

The initial flat-field design (`mitre_tactic`, `mitre_name` from first technique only) was replaced
by an order-preserving array-of-objects after research-agent validated ECS/OCSF alignment. The
final design (`mitre_attack` array) is strictly more correct: every technique resolves, unknown IDs
emit partial objects (id+reference only, graceful degradation), SIEM consumers get a structured join
key per finding. The research investment paid off in a qualitatively better API that will not need
a breaking schema change when consumers require multiple tactic/name pairs.

### 3.2 Research-Agent Finding the Second Latent Bug (F5)

The F5 adversary correctly identified that ICS techniques were emitting Enterprise tactic IDs —
a genuine correctness defect. The research-agent then discovered a second, independent bug:
T0830 was mapped to `LateralMovement` (wrong tactic name, not merely wrong matrix). Direct
attack.mitre.org page fetches confirmed T0830 belongs to Collection (TA0100). Without the
research-agent independently verifying the MITRE authoritative source, the fix might have mapped
T0830 to ICS Lateral Movement (TA0109) — wrong in the other direction. The external research
cross-check prevented a correctness half-fix.

### 3.3 F5 Catching What Per-Story Passes Structurally Cannot

The STORY-129 per-story F4 convergence reviewed only the delta (BC-2.11.035 scope). The ICS
catalog bug was pre-existing in `src/mitre.rs` and was masked because the initial EC-010 test
assertion was WRITTEN WITH THE WRONG EXPECTED VALUE (TA0008 / "Lateral Movement"). The per-story
adversary saw a test and a catalog entry that agreed — no red flag. Only the scoped-adversarial
F5 pass, which looks at the full diff-in-context rather than a single BC, caught the
cross-catalog inconsistency. This validates the architectural separation of per-story convergence
(correctness within BC scope) and F5 (cross-cutting correctness).

### 3.4 Authoritative-Pin Test Added Proactively (F5 Pass 1)

Pass 1 found not only the stale comment but a structural test gap: the drift guard asserted
ICS techniques do NOT emit Enterprise IDs (absence check) but did NOT assert the exact TA-id
each technique SHOULD emit (presence + value check). The fix added
`test_ics_techniques_resolve_authoritative_tactic_ids` with 12 exact id→TA-id pairs. This test
will catch any future edit that swaps two ICS TA-ids in the catalog. This is an addition whose
absence would not have been caught without the adversarial pass.

### 3.5 Magic-Number Grep Catching the 17→20 Sibling-Sweep Gap

After F5 and F7 consistency-validator both missed the "17 variants" count in 10 sibling spec
layers, an orchestrator-run-wide grep (`grep -r "17" specs/`) caught all 10 occurrences. This
cost one additional D-215 burst but prevented shipping specs that contradicted the implemented
20-variant enum. The catch validates the magic-number-grep approach as a concrete safety net
for count-change propagation.

### 3.6 F6 Hardening Depth

The F6 report provides exceptionally rigorous justification for why no new Kani VP is warranted:
formal panic-obligation audit of `FindingJsonDto::from`, 3-way mechanical coverage argument
(existing VP-007 proofs + exhaustive drift-guard sweep + 13 BC reporter tests), explicit
accounting for why 4 mutation survivors are Kani-verified false positives. This level of
justification is reusable as a template for future "test-sufficient" VP decisions.

---

## 4. What Went Wrong / Process Gaps

### 4.1 DRIFT-UNCOMMITTED-TEST-EDITS-001 — Convergence Attested to Stale Committed SHAs

**What happened:** After the F5 ICS catalog fix, the implementer committed only `src/mitre.rs`
(719816e). Three test files with corrected assertions remained as uncommitted working-tree edits.
The 3 adversarial passes (74a48ea / cf22de9 / cf22de9) ran `cargo test` against the working tree,
saw green (correct values), and reported CLEAN. The committed SHAs at those points still carried
old wrong test assertions. CI caught the gap only when PR #307 was pushed. pr-manager had to
commit the test corrections as a separate commit 96f0afc. Final merged state is correct.

**Root cause:** No precondition requiring `git status --short` CLEAN before convergence dispatch.
The orchestrator and the adversary have no shared protocol requiring that "what CI sees" matches
"what the adversary reviewed." The adversary operates on the filesystem, CI operates on committed
SHAs.

**Risk materialized:** If CI had been skipped or the test files had carried accidentally correct
assertions, a correctness gap would have merged undetected. In this case CI served as the
backstop, but the backstop was not designed to catch this failure mode — it was incidental.

**Concrete improvement (PROP-MJN-01):** Add a `convergence-clean-tree-guard` policy and
enforcement point:
- Before dispatching any adversarial pass (per-story F4, per-fix F5, any), the orchestrator
  verifies `git status --short` is empty (clean working tree).
- If untracked/modified files exist matching the story's file list, dispatch is BLOCKED with an
  explicit error: "Working tree not clean — commit or stash before adversary dispatch."
- Execution evidence from adversary passes (test output) MUST cite the committed SHA, not assume
  working-tree == committed.
- Affected files: orchestrator per-story-delivery workflow, F5 fix dispatch template.

**Severity:** MEDIUM. Outcome was correct due to CI backstop, but the convergence protocol was
structurally invalid during the 3-pass window.

---

### 4.2 DF-SIBLING-SWEEP-001 Gap — F5 BC Update Did Not Propagate Count to All Spec Layers

**What happened:** The F5 PO bumped the 5 directly-modified BCs to reflect 20 MitreTactic variants.
The per-fix adversarial passes (3 rounds) reviewed the directly-changed files. The F7 consistency-
validator was scoped to the feature delta (BC-2.11.035 touchpoints). All three passes missed the
"17 variants" count assertions embedded in 10 sibling artifacts: vp-016, BC-2.10.004, cap-10,
cap-11, ent-04, ent-05, nfr-catalog, test-vectors, prd.md, module-criticality. The gap was caught
only when the orchestrator ran a wide grep for the literal "17" during F7.

**Root cause:** DF-SIBLING-SWEEP-001 is a policy, not an automated enforcement point. The F5
product-owner, the 3 per-fix adversarial passes, and the F7 consistency-validator all operated
within scoped contexts that did not include the sibling artifact layer. The policy's intent
(propagate to all spec layers) requires explicit triggering on count/variant changes — but
"variant/count change" is not a recognized event class in the current workflow dispatch logic.

**Concrete improvement (PROP-MJN-02):** Extend DF-SIBLING-SWEEP-001 with a triggering rule:

> When any BC, VP, or code artifact changes a variant-count, enumeration-size, or named-constant
> value, the orchestrator MUST explicitly invoke the sibling-sweep before declaring the fix burst
> complete. Minimum sweep scope: ALL spec layers (BCs, VPs, L2 domain specs, NFR catalog,
> test-vectors, prd.md, module-criticality, holdout files). The sweep is a blocking step, not an
> advisory.

Implementation: add a `magic-number-sweep-on-count-change` checklist item to the F5 fix-dispatch
template and to the per-story convergence template, triggered by any AC that mentions enum size,
count, or variant names. This is Lesson 2 from `cycles/feature-mitre-json-names/lessons.md`.

**Severity:** HIGH. The delta was shipped in 10 spec files for multiple hours in the F7 window
before the catch. Those specs would have been wrong on-disk. Consumers of spec documents would
have received inaccurate counts.

---

### 4.3 PR-Manager Agent Shortstop (PAT-001 — Third Instance)

**What happened:** Two pr-manager runs (one for PR #306, one during the F5 fix cycle) came to rest
after generating reviewer verdicts without capturing and consolidating those verdicts into a final
report. The orchestrator had to separately invoke `pr-reviewer` and `security-reviewer` independently
to get the verdicts, then proceed with human merge authorization.

**Root cause:** PAT-001 (first observed feature-arp-v0.7.0, 5 occurrences; one in feature-pcapng-
reader). The pr-manager agent's completion criterion is misaligned: it treats "APPROVE verdicts
received" as success rather than "merge completed + CI confirmed + report written." This is a
structural prompt issue, not a parameter issue — the mitigation of explicit "DO NOT STOP AT APPROVE"
instruction has not been effective.

**Pattern note:** This is the 3rd session with PAT-001 occurrence. The previous two sessions both
generated improvement proposals (PROP-01) that remain PENDING HUMAN REVIEW. This pattern is now
chronic across 3 cycles. The engine cannot learn without the human reviewing the backlog.

**Severity:** MEDIUM (workflow interruption, not a correctness issue). Escalated to CRITICAL pattern
priority given 3-cycle recurrence without resolution.

---

### 4.4 State-Manager and Devops Agent 600s Watchdog Stalls

**What happened (STATE.md notes):** During the D-215 sibling sweep and F7 burst, a state-manager
agent and a devops agent stalled mid-task at the 600-second watchdog limit, leaving partial
uncommitted work that required orchestrator resume and re-execution. The partial state in both
cases was recoverable but required extra orchestrator attention to diagnose and recover.

**Root cause:** No explicit checkpoint-and-resume protocol for long-running spec-update agents.
When a state-manager is given a large burst (10 spec file updates in D-215) within a single
invocation, it can exceed the watchdog before committing. The agent has no internal checkpointing.

**Concrete improvement (PROP-MJN-03):** For spec-update bursts touching more than 5 files,
the orchestrator should decompose the burst into two invocations with a `git commit` between
them (5-file chunks). This prevents watchdog stalls from leaving partial state. Alternatively,
add an explicit checkpoint instruction to the state-manager dispatch template: "Commit every 5
files. Do not wait until all files are updated to commit."

**Severity:** LOW (recoverable with orchestrator intervention). Medium operational cost.

---

### 4.5 Admin-Merge Authorization Classifier Gates (Multiple)

**What happened:** Multiple merges (PR #306, PR #307, PR #308) required human authorization
before the orchestrator could proceed, because the repository has squash-disabled merge policy
and admin merge bypass restrictions. Each gate required a human reply, introducing latency.

**Observation:** This is working as designed (human in the loop for merges). No process gap.
However, the orchestrator's phrasing of merge authorization requests was inconsistent — some
included CI status, some did not. The human should always see: PR number, CI status, current
develop HEAD, proposed action. This is a presentation quality issue, not a workflow gap.

**Concrete improvement (PROP-MJN-04):** Standardize the merge-authorization request template
to always include: (1) PR number + URL, (2) current CI status (pass/fail + run ID), (3) current
develop HEAD SHA, (4) what action will be taken after authorization, (5) any outstanding reviewer
verdicts. This is already partially captured by PROP-E18-05 (PENDING) — this cycle validates
that proposal.

**Severity:** LOW (no incorrect outcomes; presentation quality only).

---

## 5. Convergence / Quality Metrics

### 5.1 Per-Story Adversarial Convergence (F4 — STORY-129)

| Pass | HEAD | Findings | Novelty | Counter |
|------|------|----------|---------|---------|
| 1 | b8fea97 | 3 (0C/0H/0M/3L) | HIGH | 1/3 |
| 2 | 6d8f172 | 2 (0C/0H/1M/1L) | MEDIUM | 2/3 |
| 3 | 7e020ce | 2 (0C/0H/0M/1L+1gap) | LOW | 3/3 CONVERGED |

Finding decay: 3L → 1M+1L → 1L+1process-gap. Clean convergence in 3 passes. No HIGH or CRITICAL
findings at any pass. The MEDIUM finding (M-1, EC-008 mixed-batch untested) was caught and fixed
before Pass 3. Trajectory follows expected healthy pattern: new surface at Pass 1 catches
structural gaps; Pass 2 catches surface added by Pass 1 fix; Pass 3 is clean on novel classes.

### 5.2 F5 Per-Fix Adversarial Convergence (ICS Catalog Fix)

| Pass | HEAD | Findings | Verdict |
|------|------|----------|---------|
| 1 | 74a48ea | 1L + 1 process-gap | Fix L-1; add pin test |
| 2 | cf22de9 | 2L non-blocking | Non-blocking; 20 TA-ids re-verified |
| 3 | cf22de9 | 0 novel | CONVERGED |

Total passes across F4+F5 fix convergence: 6. No streak resets, no re-divergence. Efficient
convergence profile compared to ARP feature (7 F4 passes, 8 F5 passes with 3 streak resets).

### 5.3 F6 Hardening Results

| Check | Result | Detail |
|-------|--------|--------|
| Kani (VP-007 relevant subset) | PASS | 4/4 VERIFICATION SUCCESSFUL (cargo-kani 0.67.0) |
| Mutation kill rate | 92.5% viable / 100% test-reachable | 49/53 viable caught; 4 FPs = cfg(kani) harnesses |
| Fuzz | PASS | 5.84M runs/91s, 0 crashes; new path panic-free by construction |
| cargo audit | PASS | 0 vulnerabilities (1138 advisories checked, 193 crate deps) |
| cargo deny | PASS | advisories/bans/licenses/sources all OK |
| Full regression | PASS | cargo test --all-targets exit 0 |

The 92.5% viable kill rate is below the ARP benchmark (98.9%) but the 4 survivors are
provably false positives (cfg(kani) function bodies invisible to cargo test). Test-reachable
kill rate is 100%, which is the meaningful metric.

### 5.4 F7 Consistency Findings

| Finding | Severity | Type | Resolution |
|---------|----------|------|------------|
| F7-CV-001 | MEDIUM | README tactic column wrong for T0830/T1557.002 | Fixed in PR #308 |
| F7-CV-002 | LOW | STORY-129 stale "17 variants" | Fixed in F7 burst |
| F7-CV-003 | LOW | Historical design doc wrong TA-id + no SUPERSEDED banner | Fixed in PR #308 |
| D-215 sibling sweep | MEDIUM (10 files) | "17 variants" in L2/VP/NFR specs | Fixed in D-215 burst before PR #308 |

Total F7 gaps: 4 (one discovered via grep, not consistency-validator). No behavioral regressions.
All gaps were documentation/spec currency. F7 completion: 1 consistency sweep + 1 sibling-sweep
burst + 1 docs PR. Efficient for the scope.

### 5.5 Benchmark Comparison

| Metric | ARP v0.7.0 | pcapng FE-001 | mitre-json-names (this) |
|--------|-----------|---------------|------------------------|
| Stories in cycle | 5 | 6 | 1 |
| PRs delivered | 15 | 17 | 3 |
| F4 adversary passes | 7 (3+4 restreak) | 18 total | 3 (clean streak) |
| F5 adversary passes | 8 (3 resets) | 8 (1 methodology halt) | 3 (clean) + 3 (fix) = 6 |
| Streak resets | 3 | 0 (but methodology halt) | 0 |
| F7 consistency gaps | 4 | 3 metadata | 4 (3 doc + 1 sibling-sweep) |
| Mutation kill rate | 98.9% | 94.4% | 92.5% viable / 100% test-reachable |
| Fuzz crashes | 0 | 0 | 0 |
| Doc-tense recurrences | 7 | 4+ | 1 (Pass 1 doc-tense; isolated) |
| pr-manager shortstop | 5 | 1 | 2 |
| Fix-induced regressions | 1 | 0 | 0 |

Story scope was smallest of any cycle (1 story). Convergence was cleaner than prior cycles.
The ICS catalog fix was the dominant complexity — it is effectively a separate mini-feature
that surfaced inside F5.

---

## 6. Dimension Analysis

### 6.1 Cost Analysis

No cost-summary.md data available for this cycle (cost tracking at the token/dollar level was
not instrumented). Qualitative observations:

- **Most expensive phase:** F5 (ICS catalog fix). The fix required: research-agent validation
  (5 web fetches + 3 Perplexity calls), an architect scoping pass (f5-ics-catalog-fix-scope.md —
  a 416-line scoping document), 3 adversarial passes, a PO BC-update pass (5 BCs bumped + 3
  holdout files), and a security review.
- **Cost surprise:** The research-agent F5 validation was ~8 MCP tool calls. The architect
  scoping document was expensive to produce but prevented implementation errors — it enumerated
  every affected test, every compile-break, every tactic TA-id. Good investment.
- **Cheapest phases:** F2 (1 BC, low AC count), F3 (single story, minimal decomposition).
- **Self-cost estimate:** This session review requires reading ~12 cycle artifacts and producing
  a structured report. Estimated at 2–3% of total cycle cost. Within the 5% cap.

**No baseline available for per-token comparison.** Recommend instrumenting token costs per phase
in a future cycle.

### 6.2 Timing Analysis

No wall-clock timing data recorded in STATE.md or cycle artifacts. All phases completed within
a single calendar day (2026-06-23). Qualitative timing observations:

- **Longest phase:** F5 (ICS catalog fix including research, scoping, implementation, 3 adversary
  passes, PR review, merge). Estimated 40–50% of cycle elapsed time.
- **Bottleneck:** Human merge authorization gates. PR #306, PR #307, PR #308 each required human
  authorization before proceeding. No human wait time data recorded.
- **Parallelization:** F6 hardening tasks (Kani, mutation, fuzz, security, regression) were
  described as sequential in the report but could be parallelized (Kani and mutation are
  independent; fuzz and security are independent of both). No explicit parallelization occurred.
- **Watchdog stalls:** 2 agent stalls at 600s limit extended total elapsed time by an estimated
  15–30 minutes.

**Recommendation:** Record `started_at` and `completed_at` timestamps per phase in cycle-manifest.
The current cycle-manifest.md has fields for these but both are null (status: in-progress, never
updated to completed).

### 6.3 Convergence Analysis

**F4 per-story:** Clean 3-pass convergence. Finding decay normal (3L → 1M+1L → 1L). No re-streaks.
**F5 per-fix:** Clean 3-pass convergence. No re-streaks. Authoritative-pin test added.
**F7:** 1 fresh-context consistency pass + 1 sibling-sweep. Efficient.
**Overall:** Best convergence profile of any recorded cycle. The 1-story scope helped; the ICS
fix complexity was contained by rigorous scoping (f5-ics-catalog-fix-scope.md).

### 6.4 Agent Behavior Analysis

- **Orchestrator T1 compliance:** No evidence of write/exec attempts by the orchestrator. Stayed
  read-only during analysis phases.
- **Implementer scope discipline:** The implementer committed `src/mitre.rs` only and left test
  files uncommitted (see DRIFT-UNCOMMITTED-TEST-EDITS-001). This was a process gap, but scope
  was correct (no extra files touched). Template adherence was otherwise good.
- **Research-agent:** Exemplary. 8 MCP calls covering web fetches, Perplexity research, and
  reasoning synthesis. Clearly cited sources, flagged confidence levels (✅ vs ⚠), and identified
  a second independent defect beyond the question asked. T1 compliant (no writes).
- **pr-manager:** Shortstop pattern recurred (PAT-001, 3rd session). Still requires orchestrator
  intervention to complete merge step. No tier violation.
- **Adversary:** Fresh-context discipline maintained. No evidence of reading prior adversary pass
  output before producing findings. 3-pass protocol clean.
- **Formal-verifier:** Rigorous VP decision justification. Produced reusable "test-sufficient"
  argument template. Correctly identified 4 cfg(kani) false positives rather than inflating kill
  rate concern.

### 6.5 Gate Outcome Analysis

| Gate | Outcome | Notes |
|------|---------|-------|
| F1 design review | PASS (1st try, research override) | Research-agent changed the design before gate |
| F2 BC review | PASS (1st try) | — |
| F3 story gate | PASS (1st try) | — |
| F4 per-story convergence | PASS (3 clean passes) | No re-streak |
| F4 PR #306 review | PASS (1st try) | pr-reviewer APPROVE; security PASS |
| F5 per-fix convergence | PASS (3 clean passes after fix) | Shortstop on PR creation |
| F5 PR #307 CI + review | PASS (after 96f0afc commit-correction) | CI failed on push; corrected; then CI green |
| F6 all 5 tasks | PASS (1st try) | — |
| F7 consistency audit | PASS (2 bursts: audit + sibling sweep) | — |
| F7 PR #308 merge | PASS | — |

First-try gate pass rate: 8/10 = 80%. Non-first-try gates: F5 PR (uncommitted tests) and F7
sibling-sweep (missed count propagation). Better than prior cycles (ARP: 60%, E-18: 62.5%).

### 6.6 Wall Integrity Analysis

**F4 per-story adversary:** Fresh-context discipline confirmed. Pass 1 saw "3L" not the future
Passes 2/3 results. No cross-pass contamination evidence.

**F5 per-fix adversary:** Three fresh-context passes. Pass 3 re-verified Pass 2 findings as
expected (non-blocking) and added the all-20-TA-id verification table independently.

**Research-agent isolation:** The research-agent validation report (f5-ics-tactic-id-validation.md)
contains independent external verification, not a re-statement of the adversary's assertion. It
separately fetched T0830's tactic from attack.mitre.org and from the TA0100 Collection page —
two independent sources, not the adversary's opinion. Wall held.

**No wall leak detected.** All asymmetry-relevant agents (adversary, research-agent, formal-verifier)
operated within their documented context excludes.

### 6.7 Quality Signal Analysis

| Signal | Value | Interpretation |
|--------|-------|----------------|
| Kani VP-007 re-run | 4/4 SUCCESSFUL | Delta did not break existing proofs |
| Mutation kill rate (viable) | 92.5% | Below 98.9% ARP benchmark; explained by 4 cfg(kani) FPs |
| Mutation kill rate (test-reachable) | 100% | No real production-logic gaps |
| Fuzz crashes | 0/5.84M | Clean |
| Security findings exploitable | 0 | cargo audit + deny both clean |
| ICS correctness defect density at F5 | 1 HIGH + 1 LATENT | Both pre-existing; feature exposed them |
| F7 doc-spec currency findings | 4 (3 doc + 1 sibling-sweep) | 3 predictable; 1 structural (sibling-sweep gap) |

Pre-existing latent ICS bug density: the feature exposed 2 ICS technique tactic errors that had
been present since at least the ARP feature (T0830 was added in STORY-114). The per-story
adversarial passes for STORY-114 did not catch it because the test assertion was written with
the wrong expected value — the adversary saw matching test+code and found no contradiction.
F5 structural scope (cross-catalog) was the only level that could catch it.

### 6.8 Pattern Detection (Cross-Run)

**PAT-001 (pr-manager shortstop):** 3rd consecutive cycle. 2 occurrences this cycle.
Total: 8+ occurrences across 4 cycles. PROP-01 still PENDING HUMAN REVIEW. **Urgent.**

**PAT-002 (doc-tense recurrence):** Only 1 occurrence this cycle (Pass 1 F-1 doc-tense in
demo-note.md). Improvement over ARP (7) and pcapng (4+). The trend is positive but not zero.
Possible cause of improvement: this was a 1-story cycle with small surface area.

**NEW PAT-010 (uncommitted-test-edits / convergence-stale-tree):** First occurrence.
Distinct from PAT-004 (consumer-sweep gap) — this is about git working-tree != committed-tree
during adversary dispatch. Requires its own pattern entry and PROP-MJN-01.

**NEW PAT-011 (sibling-count-propagation-gap):** Closely related to PAT-004 (consumer-sweep
gap) but distinct: PAT-004 is about renamed symbols not propagating to consumers; PAT-011
is about count/variant changes in an enum/slice not propagating to all count-asserting spec
layers. DF-SIBLING-SWEEP-001 is the policy, but it lacks an automated triggering condition.
Requires PROP-MJN-02 extension.

**Positive trends:**
- Convergence cleaner (fewer passes, no streak resets): improving cycle-over-cycle
- Doc-tense recurrences declining: 7 → 4 → 1
- F4 gate first-try rate: 60% → 62.5% → 80% (improving)
- Fix-induced regressions: 1 (ARP) → 0 → 0 → 0

**Negative trends / concerns:**
- pr-manager shortstop chronic; backlog not being reviewed
- Sibling-sweep gap recurring (PAT-004 third manifestation variant)

---

## 7. Prioritized Improvement Proposals

### PROP-MJN-01 — Convergence Clean-Tree Guard (P1 CRITICAL)

**Category:** workflow + agent
**Evidence:** DRIFT-UNCOMMITTED-TEST-EDITS-001; D-211; `lessons.md` Lesson 1
**Root cause:** No precondition checking `git status --short` before adversary dispatch.
**Proposal:**
Add a `convergence-clean-tree-guard` policy and enforcement block to the per-story-delivery
and F5-fix-dispatch orchestrator sequences:
1. Before dispatching any adversary pass, orchestrator verifies `git status --short` returns empty.
2. If working-tree changes exist matching the story's file list, dispatch is BLOCKED.
3. Adversary dispatch prompt preamble states "execution evidence cited in this pass comes from
   committed SHA `<SHA>` verified clean by orchestrator pre-check."
4. If the adversary cites a test run, it must include the commit SHA the run was on.
**Affected files:** orchestrator-per-story-delivery skill, F5 fix-dispatch template, DF policy file.
**Risk:** Low. Adding a precondition check is non-breaking; it will block incorrectly only if
there is a legitimate reason to have uncommitted changes at dispatch time (which should not exist).

---

### PROP-MJN-02 — Magic-Number Sweep on Count/Variant Change (P1 HIGH)

**Category:** workflow + policy (DF-SIBLING-SWEEP-001 extension)
**Evidence:** D-215; `lessons.md` Lesson 2; DF-SIBLING-SWEEP-001 gap
**Root cause:** DF-SIBLING-SWEEP-001 exists as a policy but has no automated triggering condition
for the specific case of count/variant-size changes.
**Proposal:**
Add a triggering rule to DF-SIBLING-SWEEP-001: "When any BC, code artifact, or VP changes an
enum variant count, array/slice size, or named numeric constant, the orchestrator MUST invoke a
sibling-sweep before marking the fix-burst complete." The sibling-sweep scope is:
- All BCs in affected subsystems
- All VPs referencing the changed type
- All L2 domain specs (cap-*, ent-* files)
- NFR catalog
- Test-vector documents
- prd.md
- module-criticality.md
- Holdout files referencing the changed value
Add a blocking checklist item to the F5 fix-dispatch template and per-story convergence template
triggered whenever AC or implementation adds/removes enum variants or changes a count value.
**Affected files:** policies.yaml (DF-SIBLING-SWEEP-001), orchestrator-feature-sequence skill,
per-story delivery template.
**Risk:** Low false-positive rate. Most fix bursts do not change variant counts.

---

### PROP-MJN-03 — PR-Manager Merge-Completion Primary Criterion (P1 CRITICAL — escalated from PAT-001)

**Category:** agent (pr-manager)
**Evidence:** PAT-001 (8+ occurrences, 4 cycles); PROP-01 still PENDING HUMAN REVIEW
**Root cause:** pr-manager's internal success criterion is "APPROVE verdicts received" rather than
"PR merged + CI confirmed + report returned to orchestrator."
**Proposal:** Restructure the pr-manager agent definition so the **primary** success criterion is:
"PR is merged AND CI on merged commit is green AND final report delivered to orchestrator." Frame
APPROVE as a precondition, not a completion. The merge step should be the last action before
returning, not a step the orchestrator has to separately authorize. Note: this does not bypass
human merge authorization — it changes how pr-manager hands off to the human and confirms completion.
This is an escalation of PROP-01 given 4 cycles without resolution.
**Affected files:** vsdd-factory:pr-manager agent definition.
**Risk:** Medium. Changing the agent's primary completion criterion may require careful re-testing
to avoid new shortstop patterns at different steps.

---

### PROP-MJN-04 — Large Spec-Update Burst Decomposition (P2 MEDIUM)

**Category:** workflow
**Evidence:** D-215 state-manager watchdog stall; devops agent watchdog stall
**Root cause:** Long-running spec-update invocations (10+ file updates) can exceed the 600s
watchdog, leaving partial uncommitted state.
**Proposal:** For any spec-update burst touching more than 5 files, the orchestrator decomposes
into sequential 5-file invocations with a `git commit` after each batch. Add this decomposition
rule to the state-manager dispatch template. Alternatively, add an explicit checkpoint instruction:
"After updating 5 files, commit what you have before continuing."
**Affected files:** state-manager agent dispatch template.
**Risk:** Low. Increases commit count but does not change correctness.

---

### PROP-MJN-05 — Merge-Authorization Request Standardization (P3 LOW)

**Category:** workflow (template)
**Evidence:** Multiple PRs required multiple back-and-forth clarification of CI status during
merge authorization requests this cycle.
**Proposal:** Standardize the human merge-authorization request to always include:
1. PR number + full URL
2. Current CI status (pass/fail, run ID or link)
3. Current develop HEAD SHA (with verification command)
4. What action will be taken after authorization
5. Active reviewer verdicts (pr-reviewer + security-reviewer, if applicable)
This extends PROP-E18-05 (PENDING) with the CI-status and HEAD-SHA requirements.
**Affected files:** orchestrator merge-authorization prompt template.
**Risk:** None. Additive information only.

---

### PROP-MJN-06 — Pattern Database: Append PAT-010, PAT-011 (P2 MEDIUM)

**Category:** pattern (database)
**Evidence:** DRIFT-UNCOMMITTED-TEST-EDITS-001 (new pattern); D-215 sibling-count gap (variant of PAT-004)
**Proposal:** Add PAT-010 (convergence-stale-tree) and PAT-011 (sibling-count-propagation-gap)
to `.factory/session-reviews/pattern-database.yaml`. Link both to their improvement proposals.
**Affected files:** `.factory/session-reviews/pattern-database.yaml`
**Risk:** None. Informational.

---

## 8. Summary for Human Review

**Cycle outcome:** Clean delivery. 1 story (STORY-129), 1 pre-existing latent bug found and fixed
(ICS tactic-catalog), 3 PRs merged, v0.9.4 released. F6 hardening passed all 5 tasks. F7 converged.
All quality gates green.

**What worked:** Research-agent catching the second latent T0830 bug; F5 structural scope catching
what per-story passes cannot; authoritative-pin test; magic-number grep catch; F6 rigorous
test-sufficiency justification.

**Two new process gaps needing engine codification:**
1. DRIFT-UNCOMMITTED-TEST-EDITS-001 → PROP-MJN-01 (convergence clean-tree guard)
2. 17→20 sibling-sweep gap → PROP-MJN-02 (magic-number-sweep-on-count-change triggering rule)

**Chronic issue requiring human action:** PROP-01 (pr-manager shortstop) has been PENDING HUMAN
REVIEW for 3 sessions. Now escalated as PROP-MJN-03 (P1 CRITICAL). The improvement backlog
(`.factory/session-reviews/improvement-backlog.md`) has 18 proposals in PENDING HUMAN REVIEW
status across 3 prior cycles. These will accumulate without periodic review.

**New patterns to add:** PAT-010 (stale-tree convergence), PAT-011 (sibling-count propagation).

**Benchmark update:** This cycle shows improved convergence (80% first-try gate rate, 0 streak
resets, doc-tense down to 1) but surfaces two new structural workflow gaps.

---

_Produced by session-reviewer (adversary model / Sonnet 4.6). Read-only analysis — no
code or specs modified. Source artifacts: `.factory/STATE.md`, `.factory/cycles/feature-mitre-json-names/`
(all cycle artifacts), `.factory/session-reviews/` (pattern-database, benchmarks, backlog)._
