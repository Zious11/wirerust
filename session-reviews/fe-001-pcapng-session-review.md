---
document_type: session-review
cycle_id: feature-pcapng-reader
released_version: null  # release deferred per D-194; latest release v0.9.2 on develop
develop_head_at_close: fcb8dce
reviewer: session-reviewer (adversary model — independent perspective)
produced_at: "2026-06-22"
pipeline_path: FEATURE_MODE F1→F7 (full cycle, new feature on brownfield)
stories_in_cycle: 6  # STORY-123..128 (E-19 epic)
prs_delivered: 17  # PRs #281..#297 (F4: #281-286; F5: #287-292; F6: #293-296; F7: #297)
baseline_cycles:
  - feature-arp-v0.7.0 (2026-06-16)
  - feature-story-119-grouped-collapse (2026-06-19)
---

# Session Review — FE-001 pcapng Capture-Format Reader

**Cycle closed:** 2026-06-22 (D-194, human-approved). Feature: pcapng capture-format reader (E-19 epic, ADR-009 rev 13, 6 stories STORY-123..128). F1–F7 all converged + human-approved. develop=fcb8dce. Release deferred per human decision (D-194); v0.9.2 remains latest release.

---

## 1. Outcome Summary

**What shipped (develop @ fcb8dce, not yet released):**

| Metric | Value |
|--------|-------|
| Stories delivered (cycle) | 6 (STORY-123..128, E-19 epic COMPLETE) |
| Total stories delivered (project) | 77 |
| New behavioral contracts | 10 (BC-2.01.009–018) + 1 extended (BC-2.12.011) |
| BC retired | BC-2.01.004 |
| Active BCs | 302 |
| Verification properties locked | 7 new (VP-025–031); total 31/31 verified |
| Kani harnesses (new) | 6 (VP-025: 4 × 59 checks; VP-026: 272 checks; VP-027: 687 checks) |
| cargo-fuzz target (new) | 1 (VP-028: 2,340,242 execs, 0 crashes) |
| proptest suites (new) | 3 (VP-029/030/031) |
| Test count at cycle close | 1,891 (up from ~1,700 at F4 entry) |
| Mutation kill rate | 94.4% strict / 100% equiv-adjusted |
| Security verdict | PASS — 0 CRITICAL, 0 HIGH |
| F5 adversary passes | 8 (5 substantive + Pass-2a methodology halt + 3 clean) |
| F6 hardening PRs | 4 (VP locks #293/#294, mutation-gap #295, SEC hardening #296) |
| F7 remediation items | 3 (all metadata/documentation; 0 behavioral regressions) |
| PRs merged (cycle) | 17 (PRs #281–#297) |
| ADR created | ADR-009 rev 13 (28 decisions) |
| Error codes added | E-INP-008..E-INP-015 (pcapng-specific error taxonomy) |

---

## 2. Convergence Analysis

### F5 Scoped Adversarial Refinement (8 Passes — Primary Focus)

**Trajectory:** Pass 1: 5 findings (1H/2M/2L) → Pass 2a: METHODOLOGY HALT → Pass 2: 3 findings (MED doc-tense) → Pass 3: 1 finding (MED sibling-sweep miss) → Pass 4: 1 finding (MED per-test RED comments) → Pass 5: 2 findings (1MED doc-divergence, 1LOW short-read) → Pass 6: CLEAN → Pass 7: CLEAN → Pass 8: CLEAN.

**Why did doc-tense take 4 iterations (Passes 2–5)?**

Root cause: incomplete grep token coverage in the DF-GREEN-DOC-TENSE-SWEEP policy. The token set checked for obvious markers (`"RED:"`, `"todo!()"`, `"not yet implemented"`) but missed the following phrasing variants that occur throughout the pcapng test corpus:

- Bare `/// RED:` per-test-body lines (Pass 4 gap — header-level sweep caught module headers but not individual test body doc lines in story126)
- `"falls to wildcard"` / `"falls to \`_\`"` / `"currently has NO"` — implementation-status phrases written during RED-gate authoring (Pass 5)
- `"doesn't exist yet"` / `"no <X> arm"` — negative-capability phrases in in-code comments
- `"currently satisfied by"` — passive-implementation-status phrase

The single-site-at-a-time fix pattern compounded this. Passes 2 and 3 swept module headers and top-level doc blocks but skipped per-function `/// RED:` lines in test bodies (a different AST-depth). Pass 4 caught story126 individual test bodies. Pass 5 found a separate category (doc claiming wrong block-arm structure). Each pass exposed a previously-invisible layer of the same underlying problem.

**What would have caught it in one pass:**
A comprehensive grep sweep with a token list covering ALL known RED-gate-era phrasing families — not just leading markers. The PG-F5-DOCTENSE-TOKENS-001 process-gap captures this precisely: the token list must be expanded to include bare `RED:`, behavioral absence phrases, and wildcard-fallthrough statements. With an expanded token list run at Pass-2 dispatch, all three remaining categories would have surfaced simultaneously.

**Cost:** 3 additional adversary rounds × ~3 agent invocations each = ~9 extra agent invocations attributable to the incomplete sweep token list.

### F4 Per-Story Adversarial Convergence

Each of the 6 stories required 3 clean passes (BC-5.39.001). Notable per-story findings:
- STORY-123: Pass-1 caught crate-mandated reimpl regression (critical BE decode bug) + 2 HIGH
- STORY-128: Pass-1 caught CRITICAL zero-packet notice omission (both mandatory BC-2.01.009 PC6 parenthetical segments absent — guaranteed Phase-4 holdout failure if undetected)
- STORY-125: Pass-1 fixed VP-025 stale saturation vector + BC-2.01.012 Inv6 contradiction

The per-story adversarial loop caught a guaranteed Phase-4 holdout failure in STORY-128 (PC6 zero-packet notice omission). Without the per-story loop, this would have been a Phase-4 failure requiring a full cycle reset. High-value catch.

### F2 Adversarial Spec Review (10 Passes)

F2 convergence trajectory: 23/24/17/13/13/13/12/8/4/5 (last 3 passes = 0C/0H). Ten passes to converge is above the F4 per-story precedent (3 passes) and reflects the complexity of specifying a new binary parser format for the first time on this codebase. Fourteen process-gap lessons were captured (lessons.md), indicating the F2 work was genuinely novel spec territory.

### F6 Mutation Testing (Two-Run Artifact)

The F6 mutation testing required two runs — a primary 29-minute run that produced misleading "0 MISSED" results due to 39 TIMEOUT artifacts, and a confirmation recheck (37 minutes). The timeout artifact had two independent causes:

1. **Unrelated heavy test binaries:** `cargo-mutants` runs the full workspace suite per mutant. At 120s cap, DNP3/Modbus test binaries consumed budget before pcapng tests could resolve. The 16-binary baseline took ~31s alone — leaving only ~89s for a mutated run, insufficient for suites with proptest shrink.
2. **proptest shrink amplification:** Arithmetic mutants in bounds/allocation computations triggered proptest's shrink loop, causing individual test invocations to run far past the 120s cap.

The recheck (600s timeout, pcapng-scope test binaries only) resolved all 39 ambiguous timeouts into clean CAUGHT (18) or genuine MISSED (21). The final verdict was 94.4% strict / 100% equiv-adjusted — a clear pass.

**Cost:** Two full mutation runs (29m + 37m = ~66 minutes wall time, plus agent overhead for analysis and PR). A better-scoped primary run (longer timeout, pcapng test binaries only) would have been sufficient in one pass.

---

## 3. Process Gaps and Lessons

### PG-F5-FRESHNESS-001 — Local Develop Stale After Server-Side Merge

**What happened:** Pass-2 adversary was dispatched at develop=e75a797 (stale), while the actual develop was 97c66b0 (post-PR #287). The server-side `gh pr merge` did not advance the local branch. The entire Pass-2a review was void.

**What worked:** The adversary immediately flagged the staleness (F-F5P2-001) rather than silently reviewing wrong code. The halt-and-restart was correct.

**Cost:** One full adversary invocation wasted. Negligible in absolute terms; high in disruption to the convergence counter.

**Concrete proposal (PG-F5-FRESHNESS-001 follow-up):** Codify a mandatory `git pull --ff-only` step in the pr-manager post-merge protocol, executed before any same-session adversary dispatch. This is a one-line workflow addition with zero risk. Status: MITIGATED this session; awaiting codification as permanent pr-manager workflow step.

---

### PG-F5-DOCTENSE-TOKENS-001 — Incomplete grep Token Set (HIGH — 4 wasted passes)

**What happened:** DF-GREEN-DOC-TENSE-SWEEP policy exists in policies.yaml with a documented token list. The token list was too narrow. Four adversary passes (2, 3, 4, 5) each found new phrasing variants in the same conceptual category (stale RED-gate prose) because the grep used in each remediation pass searched only for the token found in the current pass, not the full universe of RED-gate-era phrasing.

**What worked:** The policy existed. The adversary reliably identified each instance. The per-pass fixes were correct.

**What cost iterations:** The policy lacked operationalized tokens for: bare `/// RED:` in test body lines (vs module/function headers), behavioral-absence phrases (`"no <X> arm"`, `"falls to wildcard"`), implementation-status markers (`"currently has NO"`, `"doesn't exist yet"`).

**Concrete proposal (PROP-FE001-01):** Expand the DF-GREEN-DOC-TENSE-SWEEP token set in policies.yaml to include at minimum:
- `"RED:"` (bare, without `///`)
- `"falls to"` (catches "falls to wildcard", "falls to `_`")
- `"doesn't exist yet"`, `"does not yet"`, `"not yet implemented"`
- `"no .* arm"` (regex)
- `"currently has NO"`, `"currently satisfied by"`
- `"wildcard"` (when appearing in doc-comment context)
- `"currently"` (as a trigger for review — often precedes a RED-gate status claim)

The sweep should operate at ALL depth levels (module doc, function doc, per-test `/// ...` in test body), not only at the first doc line. A single invocation with an expanded token list at Pass-2 dispatch would have closed all three subsequent tense-category findings simultaneously.

---

### VP-027 META-GAP — Tautological Kani Harness Surviving Per-Story Review

**What happened (F-F5P1-001):** The VP-027 Kani harness in STORY-125's delivery inlined the EPB decode logic directly instead of calling `decode_epb_body`. The proof was vacuously true by re-derivation — it proved the inline copy matched itself, not that the production function met the property. This survived:
- Per-story adversarial convergence (3 clean passes in STORY-125)
- F4 gate consistency check (97/100 score)
- Human F4 gate approval (D-186)

It was caught only by the F5 scoped adversarial sweep with a different adversary scope (holistic cross-story scan rather than per-story scope).

**Why per-story review missed it:** Per-story adversary scope is the story's own behavioral contracts (BC-2.01.012/014 for STORY-125). The tautological harness was in a `#[cfg(kani)]` proof module. Its vacuity was only visible when reviewing the VP-027 non-vacuity obligation from VP-INDEX — a cross-story, VP-level lens that per-story adversary doesn't carry.

**What worked:** F5 fresh-context adversary with VP-level scope caught it definitively. The F5 catch prevented a false-proven VP-027 from being locked at F6. Had the harness been locked at F6, the formal verification record would have been permanently corrupted.

**Concrete proposal (PROP-FE001-02):** Add a non-vacuity gate to the per-story VP verification checklist. For any story that delivers a Kani harness: the harness MUST call the production function by name (not re-implement it inline), and this MUST be verified by the per-story adversary as a blocking criterion. Inject: "Does each Kani harness call the named production function rather than re-implementing its logic inline? If the proof body contains a copy of the production function's internal logic, the proof is tautological and MUST be flagged CRITICAL."

---

### SEC-001 / SHB Twin-Drift Risk — Proactive Trip-Wire Pattern

**What happened:** F5 Pass-1 identified O-2: the `decode_epb_body_discriminant` twin (used for Kani tractability) had no automated equivalence enforcement against the production `decode_epb_body`. If they drifted, the Kani proofs would silently become vacuous again. PR #292 added `tests/sec_001_twin_equivalence_tests.rs` to mechanically enforce this. The F6 team proactively mirrored this with `tests/sec_shb_twin_equivalence_tests.rs` (PR #293) for VP-026's `parse_shb_body_discriminant` twin.

**What worked:** The proactive identification of the structural drift risk and the instantiation of a trip-wire test before F6 locked the VPs. The twin-equivalence pattern is now an artifact that future spec maintainers will see and understand.

**Concrete proposal (PROP-FE001-03):** Codify the discriminant-twin trip-wire pattern as a required artifact for any story that uses a Kani BMC twin (a copy of a production function used only within `#[cfg(kani)]`). The formal-verifier agent should include: "For each `#[cfg(kani)]` discriminant twin, a `#[cfg(test)]` proptest or unit test suite (`tests/sec_NNN_twin_equivalence_tests.rs`) MUST be co-authored enforcing equiv-class equivalence over the production function's full input domain subset." This makes the trip-wire a mandatory deliverable, not a proactive add-on.

---

### Mutation Testing Scheduling-Timeout Artifact

**What happened:** Primary mutation run produced 39 TIMEOUTs with misleading "0 MISSED" result. The TIMEOUTs were caused by unrelated test binaries (DNP3/Modbus) consuming the 120s per-mutant budget before pcapng tests could complete.

**Why this is recurring risk:** The wirerust codebase accumulates test binaries across features. Each new feature adds test binaries that appear in `cargo test --package=wirerust`, increasing baseline wall time per mutant. As the test suite grows, the 120s cap becomes increasingly insufficient for focused mutation testing.

**Concrete proposal (PROP-FE001-04):** Update the F6 mutation testing protocol to use `--minimum-test-timeout 300 --timeout 600` as defaults (not just the recheck). Additionally, scope mutation runs to the feature's production files using `--file src/<feature-module>.rs` and restrict the test harness with `-- --test <pcapng-test-binaries>` to exclude unrelated slow binaries. Document this scoping approach in the formal-verifier agent dispatch template for all future cycles.

---

### Input-Hash STALE on F7 Entry (Post-F6-SEC)

**What happened:** After F6-SEC hardening (PR #296), STORY-123..128 all showed STALE in `bin/compute-input-hash --scan` because ADR-009 and several BCs received post-delivery revisions (F6-SEC additions). This required a BENIGN adjudication pass at F7 entry (D-193) before input-hash rebaseline.

**Assessment:** This is expected behavior — F6-SEC added behaviors (E-INP-014/015) that were deliberately post-story additions. The adjudication mechanism worked correctly. All 6 STALE verdicts were correctly classified BENIGN.

**Observation:** The 3 pre-existing ERROR stories (STORY-001/091/121) with broken `inputs:` fields remain unresolved. These are outside FE-001 scope but create noise in every scan run.

**Concrete proposal (PROP-FE001-05, LOW):** Fix the 3 pre-existing ERROR stories in the next maintenance sweep: (1) remove the retired BC reference from STORY-001's `inputs:` list, (2) add `inputs: []` placeholder blocks to STORY-091 and STORY-121. This eliminates scan noise from these pre-existing errors.

---

## 4. Cost and Efficiency Observations

**No cost-summary.md available** (gap carried from prior cycles; cost analysis is qualitative). The cost tracker is absent from the factory infrastructure. All cost assessment is therefore relative and approximate.

### Primary Cost Drivers

| Driver | Estimated Agent Invocations | Notes |
|--------|--------------------------|-------|
| F4 per-story delivery (6 stories × 3 clean passes + remediation) | ~60–80 | Largest phase by volume |
| F2 spec adversarial convergence (10 passes) | ~30–40 | Highest F2 pass count in project history |
| F5 doc-tense long-tail (Passes 2–5) | ~15–20 (excess) | Attributable to PG-F5-DOCTENSE-TOKENS-001 |
| F6 mutation testing (two runs, analysis, PR) | ~10 | Attributable to timeout artifact |
| F5 Pass-2a methodology halt | ~3 (waste) | Attributable to PG-F5-FRESHNESS-001 |
| F7 remediation (3 doc items) | ~6 | All metadata sync; zero behavioral |
| F6 VP lock gate (7 VPs, 3 PRs) | ~15 | Expected; VPs are high value |

**Key efficiency finding:** The doc-tense long-tail in F5 (Passes 2–5) is the most actionable cost sink. The 4 excess passes each resulted in a PR that changed only documentation/comments — no behavior. Each such pass costs approximately 4–5 agent invocations (adversary, adjudicator, implementer, pr-manager, state-manager). Expanding the DF-GREEN-DOC-TENSE-SWEEP token set (PROP-FE001-01) eliminates this class of waste.

### pr-manager Pausing

The PAT-001 pattern (pr-manager stopping at APPROVE without completing steps 7–9) was observed during F4 story deliveries. Per prior session reviews, rate has been declining (100% in v0.7.0, 1 occurrence in v0.9.0), but the structural fix (STORY-121 / PROP-01) remains unimplemented. No new data on this cycle specifically — the burst-log records pr-manager dispatch but does not document shortstop occurrences. Classify as continuing background friction.

### Mutation Testing Runs (29m + 37m)

Total mutation testing wall time: ~66 minutes. The correct approach (one scoped run with adequate timeouts) would take ~37 minutes. The extra 29-minute run was entirely a consequence of the 120s cap + full-workspace test invocation. This is recoverable via protocol update.

---

## 5. Gate Outcome Analysis

| Gate | Outcome | Notes |
|------|---------|-------|
| F1 delta analysis | PASS | ADR-009 created, Option A selected |
| F2 spec adversarial convergence | CONVERGED (10 passes) | 14 process-gap lessons; no baseline for comparison |
| F2 human gate (D-164) | PASS first try | Human-approved same session |
| F3 consistency audit | CONDITIONAL PASS → PASS (3 findings, all remediated same session) | |
| F3 human gate (D-168) | PASS first try | |
| F4 per-story convergence × 6 | ALL CONVERGED (3 clean each; BC-5.39.001) | STORY-123: 3+3 rounds; STORY-125/128 notable catches |
| F4 gate (D-186) | PASS first try | Consistency 97/100; input-drift RESOLVED |
| F5 Pass-2a methodology halt | NOT A GATE — process gap | Stale-develop halt; resolved by fast-forward |
| F5 convergence (BC-5.39.001) | CONVERGED (3/3 clean at Pass 6/7/8) | 5 substantive pass rounds total |
| F5 human gate (D-190, post-SEC-001) | PASS first try | |
| F6 mutation gate (pre-PR-295) | FAIL → PASS (2-run artifact) | Resolved by PR #295 (13 gap-closing tests) |
| F6 VP lock gate (7 VPs) | PASS (PRs #293/#294) | All 7 pcapng VPs locked |
| F6 security gate | PASS (0 CRIT/HIGH) | 2 MED deferred (F6-SEC-A/B with remediation spec); landed PR #296 |
| F6 human gate (D-191/D-192) | PASS first try | |
| F7 consistency audit | CONVERGED (3 doc gaps, all non-blocking; remediated same session) | |
| F7 human gate (D-194) | PASS first try | |

**Gate first-try pass rate:** 12/14 substantive gates = 85.7% (excluding Pass-2a methodology halt which is not a gate). This is a meaningful improvement over baseline (v0.7.0: 60%, v0.9.0: 62.5%). The improvement reflects the structured phase-by-phase cadence (D-186) and the successful F4 per-story convergence loop.

---

## 6. Agent Behavior Analysis

### Tier Compliance

No T1/T2 tier violations detected in evidence across all phase artifacts. State-manager correctly operated on state/index files only (lessons.md Lesson 1 addressed an earlier violation in D-139 where state-manager edited ADR content — this was caught and reverted; the lesson was codified before F4 began).

### Fresh-Context Independence

The F5 adversary correctly identified the VP-027 tautological harness (F-F5P1-001) that had survived 3 clean per-story passes. This is a clear demonstration of fresh-context value: the per-story adversary operated under STORY-125's BC scope; the F5 adversary operated under a VP-level holistic scope and cross-story accountability. The independent perspective found what a scope-bounded reviewer could not.

The F5 adversary also correctly halted on Pass-2a due to a stale develop tree — demonstrating that the adversary's context-integrity checking is operating correctly.

### Template Adherence

No template deviations flagged. Session-checkpoints.md shows consistent D-NNN checkpoint structure throughout.

### Scope Discipline

The F6 formal-verifier proactively added the SHB twin-equivalence trip-wire (PR #293) without being prompted, mirroring the SEC-001 pattern for VP-026. This is appropriate initiative within the formal-verifier's scope — not scope creep.

---

## 7. Wall Integrity Analysis

**Wall integrity MAINTAINED throughout.**

- F5 adversary operated fresh-context at all passes. Pass-1 findings (VP-027 tautological harness, doc-tense) were independent of the per-story adversary's prior findings and not cross-contaminated by knowledge of what the per-story adversary had already flagged.
- F6 formal-verifier did not reference F5 adversary findings in its proof strategy (the VP lock gate was executed independently).
- F7 consistency-validator assessed the full delta independently; its 3 findings (VP status annotation lag, BC-INDEX annotation lag, README stale row) were metadata gaps not previously identified by any prior adversary pass.

No cross-wall information leaks detected in any phase artifact reviewed.

---

## 8. Quality Signal Analysis

| Signal | Value | Target | Status |
|--------|-------|--------|--------|
| Test count at cycle close | 1,891 | — | Up from ~1,700 (F4 entry) |
| Test failures | 0 | 0 | PASS |
| Mutation kill rate (strict) | 94.4% (136/144) | ≥90% | PASS |
| Mutation kill rate (equiv-adjusted) | 100% (137/137) | ≥95% for critical input parser | PASS |
| Kani VP-025 | 59 checks × 4 harnesses | PASS | PASS |
| Kani VP-026 | 272 checks | PASS | PASS |
| Kani VP-027 | 687 checks; non-vacuous confirmed | PASS | PASS |
| VP-028 cargo-fuzz | 2,340,242 execs, 0 crashes | 0 crashes | PASS |
| VP-029/030/031 proptest | PASS (100 cases default, shrink on fail) | PASS | PASS |
| All 31 VPs locked | 31/31 verified in VP-INDEX v2.10 | 31/31 | PASS |
| Security findings exploitable | 0 (0 CRIT, 0 HIGH) | 0 | PASS |
| F6-SEC-A (CWE-400) | RESOLVED — E-INP-014 4 GiB gate (PR #296) | Resolved | PASS |
| F6-SEC-B (CWE-770) | RESOLVED — E-INP-015 65535-IDB cap (PR #296) | Resolved | PASS |
| F6-SEC-C TOCTOU (CWE-367) | ACCEPTED — fstat-on-fd fix applied (PR #296) | Accepted | PASS |
| Spec coherence (F7) | 82% overall (9/11 non-trivial; 3 metadata gaps all remediated) | — | PASS |
| Input-drift (final) | 6 STALE (all BENIGN); 3 pre-existing ERROR | — | ACCEPTABLE |
| CI green on merge | All 17 PRs CI green | 100% | PASS |

**Notable quality achievement:** The mutation kill rate of 94.4% strict / 100% equiv-adjusted on `src/reader.rs` for a new binary parser is strong. The concentrated weakness identified (7 of 15 real gaps in `parse_idb_options` TLV-walk skip/advance machinery) was fully closed by PR #295's 13 gap-closing tests. The residual 8 survivors are all provably equivalent.

**Outstanding quality debt:**
- `src/main.rs` was not mutated (deliberately excluded from F6 scope as CLI/orchestration glue). Main.rs mutation would be a separate lower-value pass.
- SEC-008: `from_pcap_reader<R: Read>` stream path lacks the F6-SEC-A file-size gate (CWE-400 latent). Acknowledged design limitation; not CLI-reachable. Requires a future hardening story.
- 6 proven-equivalent mutants could receive `cargo-mutants` skip/ignore annotations (PG-F6-MUTANTS-HYGIENE) to clean up future mutation reports.

---

## 9. Pattern Detection (Cross-Run Comparison)

### Existing Patterns — Update

| Pattern | Prior Status | FE-001 Occurrences | Updated Status |
|---------|--------------|-------------------|----------------|
| PAT-001 (pr-manager shortstop) | OPEN; 6 total prior occurrences | Background occurrence (rate not fully counted this cycle due to burst-log granularity) | OPEN; structural fix (PROP-01 / STORY-121) still unimplemented |
| PAT-002 (doc-tense recurrence) | OPEN; policy exists (DF-GREEN-DOC-TENSE-SWEEP); policy ineffective | 4 occurrences (F5 Passes 2/3/4/5) — strongest recurrence yet | OPEN, HIGH PRIORITY; policy token list inadequate; operationalize via PROP-FE001-01 |
| PAT-003 (fix-induced regression from LOW) | OPEN; 1 prior occurrence | 0 occurrences — `read_magic` short-read fix (F-F5P5-002) used `read_exact`, standard library; no induced regression | Stable |
| PAT-004 (consumer-sweep gap post-fixburst) | OPEN; 3 prior occurrences | FINDING-F7-001/002: VP status annotation lag + BC-INDEX annotation lag (F7 metadata sweep) | 4th/5th occurrences; policy not enforced as blocking checklist item |
| PAT-006 (multipass adversary catches single-pass misses) | DOCUMENTED-POSITIVE | F5 adversary caught VP-027 tautological harness (missed by 3 clean per-story passes) | POSITIVE CONFIRMATION — 3rd occurrence of same dynamic |
| PAT-009 (adversary stale git-ref) | OPEN (confirmed mitigation pending PROP-E18-02) | Pass-2a (develop stale after gh pr merge) — different mechanism (local branch lag, not sandbox git-ref cache) | RELATED but distinct from PAT-009; logged as new variant PAT-010 below |

### New Patterns Observed

**PAT-010 — Local Develop Lag After Server-Side gh pr merge:**

Distinct from PAT-009 (sandbox git-ref cache stale-SHA reporting). In PAT-010, the orchestrator's local checkout does not advance when a PR is merged server-side via `gh pr merge`. This is a workflow gap rather than a tooling isolation artifact. The adversary correctly detected the staleness and halted (correct behavior). Mitigation: mandatory `git pull --ff-only` after every `gh pr merge` before same-session adversary dispatch. Currently codified as PG-F5-FRESHNESS-001 mitigation; awaiting permanent pr-manager workflow codification.

**PAT-011 — Kani Tautological Harness Surviving Per-Story Review:**

A proof harness that re-implements production logic inline instead of calling the production function generates a vacuous proof. This survived 3 clean per-story adversary passes because per-story scope is bounded by BC compliance, not VP non-vacuity. F5 holistic scope caught it. The corrective action (non-vacuity gate in per-story VP checklist) is PROP-FE001-02. First observed: FE-001 (F-F5P1-001 / D-188).

**PAT-012 — Mutation Timeout Artifact from Full-Workspace Test Invocation:**

When `cargo-mutants` runs the full workspace test suite per mutant, unrelated heavy test binaries consume per-mutant time budgets, producing spurious TIMEOUTs classified as ambiguous (not MISSED). This caused a misleading "0 MISSED" result in the primary mutation run that required a second run to resolve. Root cause is the growing test suite width: as features accumulate, baseline test time grows, squeezing the margin available for mutated-code runs under a fixed timeout cap. First observed: FE-001 F6 mutation pass. See PROP-FE001-04.

### Cross-Run Trends

| Metric | v0.7.0 | v0.9.0 (E-18) | FE-001 (pcapng) | Trend |
|--------|--------|---------------|-----------------|-------|
| F5 adversary passes total | 8 | 2 | 8 | Regressed (doc-tense long-tail) |
| F5 code defects found | 1 (LOW→MEDIUM regression) | 1 HIGH | 2 (VP-027 tautological H; short-read L) | Stable |
| Doc-tense recurrences | 7 | 0 | 4 | Regressed (new pcapng test corpus; policy token gap) |
| PR-manager shortstop rate | 100% (5/5) | 1 occurrence | Background (undercounted) | Unclear |
| Gate first-try pass rate | 60% | 62.5% | 85.7% | Improving |
| Mutation kill rate (strict) | 98.9% | 85% | 94.4% | Comparable (different scope each cycle) |
| F7 metadata gaps | 4 | 16 | 3 | Improved (fewer cross-document drift items) |
| F7 code defects | 0 | 0 | 0 | Consistent zero |
| Test count at cycle close | 1,592 | ~1,700 | 1,891 | Steadily growing |
| Kani harnesses locked | 5 | 0 | 6 new (+existing) | Growing coverage |

**Trend summary:** The gate-first-try rate improvement (85.7% vs 60%/62.5%) is the strongest positive trend. F5 adversary pass count regressed to 8 (same as v0.7.0 but for a different root cause — doc-tense policy token gap vs fix-induced regression). F7 metadata gaps dropped significantly (3 vs 16), suggesting the pre-F7 doc sweep from prior cycles is having residual effect.

---

## 10. What Went Well (Preserve)

**Fresh-context adversarial catches:**

1. **VP-027 tautological harness (F-F5P1-001):** F5 adversary caught a vacuous Kani proof that had passed 3 per-story clean passes. Without F5's VP-level holistic scope, a formally-documented-but-vacuous VP-027 would have been locked as "verified" at F6. The catch is permanent and high-value.

2. **Pass-2a methodology halt:** The adversary independently identified that its review target was stale and halted rather than completing a void review. This demonstrates the adversary's integrity-checking operating at the right level — not just checking content but verifying the review basis.

3. **STORY-128 zero-packet notice catch:** Per-story adversary caught a guaranteed Phase-4 holdout failure (PC6 zero-packet notice omitting both mandatory parenthetical segments). The holdout evaluator would have detected this; the per-story adversary caught it 2 phases earlier.

**Proactive trip-wires:**

4. **SEC-001 + SHB twin-drift trip-wire pattern:** PR #292 (SEC-001 EPB twin-equivalence tests) and PR #293 (SHB twin-equivalence, proactive) together provide mechanical enforcement that Kani discriminant twins remain in sync with production functions. This is a structural improvement that persists across future spec changes.

5. **F6-SEC hardening (PR #296):** The formal-verifier correctly identified F6-SEC-A (CWE-400 unbounded read_to_end) and F6-SEC-B (CWE-770 uncapped interface table) as deferred from the F5/F6 security scan and produced a complete remediation spec (E-INP-014/015 + ADR-009 Decisions 27/28 + MAX constants) that landed cleanly in one PR.

**Structured human gates:**

6. **Phase-by-phase cadence (D-186):** The human requiring approval before each of F5/F6/F7 added a human checkpoint at each phase boundary. All human approvals were first-try and same-session, suggesting the gates added confidence without adding friction.

7. **No shipped-code regressions across 17 PRs, 1,891 tests:** Implementation quality was consistently high throughout F4. The complex binary format parsing (endianness, multi-block dispatch, variable-length options TLV parsing) shipped with zero functional regressions across the test suite.

---

## Improvement Proposals

---

### PROP-FE001-01 — Expand DF-GREEN-DOC-TENSE-SWEEP Token Set (HIGH PRIORITY)

**Category:** agent / policy
**Priority:** P1 HIGH
**Evidence:** PG-F5-DOCTENSE-TOKENS-001 (from F5 convergence summary). 4 adversary passes (2/3/4/5) consumed by doc-tense-phrasing variants missed by current token set. Estimated waste: ~15–20 extra agent invocations.
**Affected file:** `.factory/policies.yaml` (DF-GREEN-DOC-TENSE-SWEEP entry)

**Recommendation:** Update the DF-GREEN-DOC-TENSE-SWEEP grep token list to include:
```
"RED:"                         # bare per-test-body marker (not "/// RED:")
"falls to"                     # "falls to wildcard", "falls to `_`"
"doesn't exist yet"
"does not yet"
"no .* arm"                    # regex: "no <X> arm"
"currently has NO"
"currently satisfied by"
"currently"                    # trigger for manual review
"wildcard"                     # when in doc-comment context
"not yet implemented"
"TODO"                         # existing; confirm included
```

Additionally: specify that the sweep operates at ALL doc-comment depth levels — module/crate doc, function doc, AND per-statement inline `/// ...` lines within test bodies. A header-only sweep misses per-test-body tense markers.

**Risk:** Low (purely additive to a grep list; no behavioral change).
**Estimated savings:** 3–4 adversary passes per cycle containing a doc-tense-bearing test corpus (approx. 12–15 agent invocations).

---

### PROP-FE001-02 — Non-Vacuity Gate in Per-Story VP Harness Checklist

**Category:** agent (per-story adversary checklist)
**Priority:** P1 HIGH
**Evidence:** PAT-011 (VP-027 tautological Kani harness surviving 3 per-story clean passes; F-F5P1-001 / D-188).
**Affected file:** Per-story adversary dispatch template; formal-verifier agent template.

**Recommendation:** Inject a blocking non-vacuity checklist item in the per-story adversary dispatch for any story that delivers or modifies a Kani harness:

  > "For each Kani harness in this story: (1) Does the harness body call the named production function by its exact function name? If the harness re-implements the function logic inline rather than calling it, flag CRITICAL — tautological proof. (2) Does a negative test exist (a counterexample-triggering input) that causes the property to fail WITHOUT the fix in place? If not, flag HIGH — harness may be non-discriminating."

Additionally: add to the formal-verifier F6 lock gate a specific check that VP-027-equivalent (BMC-twin-based) harnesses call the production function (not inline it) before locking status to `verified`.

**Risk:** Low (additive checklist item; adds ~30 seconds of adversary runtime per Kani-bearing story).

---

### PROP-FE001-03 — Discriminant-Twin Trip-Wire as Required Deliverable

**Category:** workflow / agent (formal-verifier)
**Priority:** P2 MEDIUM
**Evidence:** PAT-011 follow-up. The twin-equivalence trip-wire (SEC-001 for EPB, SHB twin for VP-026) was proactive work that prevents silent proof vacuity if production functions drift from their discriminant twins. Currently ad hoc; should be required.
**Affected file:** Formal-verifier dispatch template; F6 hardening checklist.

**Recommendation:** For any story that uses a `#[cfg(kani)]` discriminant twin (a copy of a production function used for Kani tractability), a co-authored `tests/sec_NNN_twin_equivalence_tests.rs` is a REQUIRED deliverable. The formal-verifier F6 gate should verify that this file exists and contains proptest/unit tests exercising the equivalence over the production function's representative input domain.

**Risk:** Low (makes explicit a practice already applied twice in FE-001).

---

### PROP-FE001-04 — F6 Mutation Testing Protocol: Scoped Invocation as Default

**Category:** workflow (formal-verifier dispatch)
**Priority:** P2 MEDIUM
**Evidence:** PAT-012 (mutation testing scheduling-timeout artifact). Two-run waste (29m + 37m) attributable to full-workspace test invocation under insufficient timeout.
**Affected file:** Formal-verifier dispatch template; mutation testing protocol.

**Recommendation:** Update the F6 mutation testing protocol to default to:
1. `--minimum-test-timeout 300 --timeout 600` (not 120s)
2. `--file src/<feature-module>.rs` (scope to the feature's production file)
3. `-- --test <feature-test-binaries>` (exclude unrelated heavy test binaries from the per-mutant test run)

Document that the full workspace suite verification (cargo test --all-targets) is a separate required step; mutation scoping to feature modules is appropriate and does not compromise mutation coverage of the production logic under test.

The scoping approach is documented in the F6 mutation report (`pcapng-f6-mutation-testing.md` lines 25–43) — promote it from an ad-hoc observation to a standing protocol.

**Risk:** Low (reduces false timeout ambiguity; does not change mutation coverage of the scoped module).
**Estimated savings:** ~30 minutes per cycle that requires a two-run artifact resolution.

---

### PROP-FE001-05 — Mandatory post-merge fast-forward in pr-manager workflow

**Category:** workflow (pr-manager)
**Priority:** P2 MEDIUM
**Evidence:** PG-F5-FRESHNESS-001 (Pass-2a methodology halt due to local develop lag after server-side gh pr merge).
**Affected file:** pr-manager agent dispatch template / orchestrator feature-sequence workflow.

**Recommendation:** Add as a mandatory numbered step in the pr-manager post-merge protocol:

  > "After gh pr merge completes: (1) run `git pull --ff-only origin develop` to advance local develop to the server-side merge commit, (2) verify `git rev-parse HEAD` matches the expected merge SHA before dispatching any subsequent adversary pass or dependent agent."

This prevents the Pass-2a class of waste (full adversary invocation on a stale tree). The mitigation was applied manually in FE-001; it should be structural.

**Risk:** None (additive workflow step; git pull --ff-only is safe by construction — it only advances if the server is ahead, never creates a merge commit).

---

### PROP-FE001-06 — Fix Pre-Existing Input-Hash ERROR Stories (LOW)

**Category:** quality / infrastructure
**Priority:** P4 LOW
**Evidence:** STORY-001/091/121 produce ERROR in every `bin/compute-input-hash --scan` run. Three consecutive cycles have carried this noise. STORY-001's `inputs:` list references a retired BC (BC-2.01.004); STORY-091 and STORY-121 have empty `inputs: []` arrays.
**Affected file:** `.factory/stories/STORY-001.md`, `.factory/stories/STORY-091.md`, `.factory/stories/STORY-121.md`

**Recommendation:** In the next maintenance sweep, state-manager should: (1) remove the retired BC reference from STORY-001's inputs list, (2) add a proper `inputs: []` block (or appropriate BC inputs) to STORY-091 and STORY-121 so the scan produces MATCH/STALE rather than ERROR.

**Risk:** None (administrative fix; no behavioral change to any story).

---

### PROP-FE001-07 — Update Pattern Database and Benchmarks

**Category:** pattern (database maintenance)
**Priority:** P3 LOW
**Evidence:** Three new patterns observed (PAT-010/011/012) and benchmarks table requires a new entry for FE-001.

**Recommendation:** State-manager should update:
1. `.factory/session-reviews/pattern-database.yaml` — append PAT-010 (local develop lag after gh pr merge), PAT-011 (Kani tautological harness), PAT-012 (mutation timeout artifact from full-workspace invocation)
2. `.factory/session-reviews/benchmarks.yaml` — append FE-001 benchmark entry with: stories_in_cycle=6, prs_delivered=17, test_count_at_cycle_close=1891, mutation_kill_rate_pct=94.4, mutation_equiv_adjusted_pct=100, kani_new_harnesses=6, fuzz_executions_millions=2.34, fuzz_crashes=0, f5_adversary_passes=8, f5_doc_tense_recurrences=4, f7_metadata_gaps=3, f7_code_defects=0, gate_first_try_rate_pct=85.7

**Risk:** None (documentation only).

---

## Open Follow-Ups — Cross-Check

| Item | Status | Notes |
|------|--------|-------|
| PROP-01..PROP-11 (v0.7.0 backlog) | PENDING HUMAN REVIEW (auto-deferred) | Oldest pending items — structural pr-manager fix (PROP-01) most critical |
| PROP-E18-01..PROP-E18-06 (v0.9.0 backlog) | PENDING HUMAN REVIEW | Pre-F7 doc sweep (PROP-E18-01) directly addresses recurring PAT-007 |
| PG-F5-FRESHNESS-001 | MITIGATED; codification pending | PROP-FE001-05 addresses |
| PG-F5-DOCTENSE-TOKENS-001 | OPEN HIGH | PROP-FE001-01 addresses — recommend immediate application before next doc-tense-bearing cycle |
| PG-F6-MUTANTS-HYGIENE | DEFERRED — maintenance | Annotate 6 proven-equivalent mutants with cargo-mutants skip/ignore |
| SEC-008 (from_pcap_reader stream path unbounded) | DEFERRED — hardening story pending DF-VALIDATION-001 | |
| STORY-121 (E-11 self-improvement) | DRAFT, not started | Holds relay-trust handoff, consuming-surface sweep, test-vector canonicalization, etc. |
| DRIFT-F5-O1-017STRINGS | DEFERRED LOW | BC-2.01.017 PC1 illustrative string reconciliation |
| INPUT-HASH-ERROR-PRESTORY | DEFERRED maintenance | PROP-FE001-06 addresses |
| PAT-001 (pr-manager shortstop) | OPEN — structural fix not yet implemented | PROP-01 + STORY-121 target this |

---

## Top-5 Recommendations Summary

1. **[P1 HIGH — PROP-FE001-01] Expand DF-GREEN-DOC-TENSE-SWEEP token set.** The incomplete token list caused 4 adversary passes of purely docstring churn (Passes 2–5 of F5), wasting ~15 agent invocations. Adding bare `"RED:"`, behavioral-absence phrases, and wildcard-fallthrough markers to the policies.yaml grep list eliminates this class of waste at near-zero risk. Apply before the next TDD-heavy cycle.

2. **[P1 HIGH — PROP-FE001-02] Non-vacuity gate in per-story VP harness checklist.** The VP-027 tautological Kani harness (F-F5P1-001) survived 3 clean per-story adversary passes and would have locked a vacuous proof at F6 without F5's holistic scope. A single checklist question — "Does this harness call the named production function rather than re-implementing it inline?" — closes this gap permanently with negligible overhead.

3. **[P2 MEDIUM — PROP-FE001-05] Mandatory post-merge fast-forward in pr-manager.** The Pass-2a methodology halt (stale local develop after gh pr merge) wasted one full adversary invocation. A mandatory `git pull --ff-only` step after every `gh pr merge` eliminates this class of waste with no risk. Trivial to implement; recurring impact.

4. **[P2 MEDIUM — PROP-FE001-04] Scoped mutation testing as default protocol.** The two-run artifact (29m primary + 37m recheck = ~66m total vs 37m optimal) is entirely avoidable by defaulting to longer timeouts and pcapng-scope test binaries. Document the scoped invocation pattern from this cycle's mutation report as the standing F6 protocol.

5. **[P1 HIGH — PROP-01 / PROP-E18-01, carried from backlog] Structural pr-manager shortstop fix + pre-F7 doc sweep.** These two items from the v0.7.0 and v0.9.0 backlogs have been pending human review across 3 cycles. The pr-manager shortstop (PROP-01) affects every PR delivery cycle. The pre-F7 doc sweep (PROP-E18-01) directly addresses the recurring PAT-007 pattern that drove 5 F7 rounds in v0.9.0. Both require human direction to route into STORY-121 or dedicated implementation stories.

---

*Session review produced by session-reviewer agent (adversary model) — read-only analysis of factory artifacts only. No pipeline code or specs were modified. State-manager writes this output to `.factory/session-reviews/`.*
