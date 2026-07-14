---
document_type: session-review
date: 2026-07-13
run_id: v0.12.1-release-chain
path: composite
path_name: "wave-73-close + maint-2026-07-11 + wave-74 + wave-75 + v0.12.1-release + post-wrap-triage"
product: wirerust
duration: "~4 calendar days (2026-07-11 through 2026-07-14)"
released_version: v0.12.1
prior_released_version: v0.12.0
stories_delivered: 2  # STORY-164 (wave-74) + STORY-165 (wave-75)
prs_reviewed: [396, 397, 398, 399, 400]
decisions: [D-428, D-429, D-430, D-431, D-432, D-433, D-434, D-435, D-436, D-437]
total_cost: N/A
prior_review: session-review-2026-07-10-v0.12.0.md
human_decisions:
  recorded_at: pending
  notes: >-
    Review produced 2026-07-13. Human review within 72h per session-review protocol.
    Proposals pending disposition.
---

# Session Review: wirerust — v0.12.1 Release Chain — 2026-07-13

## Executive Summary

This release chain (D-428..D-437) closed wave-73, completed maint-2026-07-11, delivered
two single-story governance waves (74 and 75), released v0.12.1, and performed post-wrap
engine triage — all in four calendar days. The quality bar held: no exploitable security
findings, all holdout N/A (zero src/ changes throughout), CI green at every merge, and
three governance mandates delivered and self-validated on their own PRs.

The top three findings are: (1) ADVERSARY-RELAY-UNRELIABLE-001 is now at 8+ lifetime
occurrences and reached its highest single-session concentration (5+) during the wave-74
gate — synchronous dispatch (PROP-V0.12.0-03) is applied but insufficient at gate scale;
an engine fix is overdue; (2) wave-74 required 13 gate passes for a 4-point story — the
currency sweep mandated by STORY-165 reduced wave-75 to 7 passes, confirming the causal
theory, but total pass counts for governance waves still far exceed the ≤4 pass target;
(3) the second squash back-merge in two releases (v0.11.5 and v0.12.1) confirms that
BACKMERGE-FAST-FORWARD-001 is a live operational gap that will recur at every release
without explicit codification.

Prior review proposals (PROP-V0.12.0-01..03) saw partial action this period:
PROP-V0.12.0-01 is partially addressed (breaking-change-delivery-protocol.md delivered);
PROP-V0.12.0-03 is applied but insufficient; PROP-V0.12.0-05 superseded by engine issue
vsdd-factory#637. Four new engine issues were filed (D-437) for process gaps surfaced during
wave-75, shrinking STORY-166 from 5 to 3 points and cleanly partitioning project vs. engine work.

---

## Run Overview

| Metric | Value | Benchmark | Status |
|--------|-------|-----------|--------|
| Total cost | N/A (no cost-summary.md) | — | — |
| Duration | ~4 calendar days | — | — |
| Stories delivered | 2 (STORY-164, STORY-165) | — | PASS |
| Story points delivered | 7 pts (STORY-164: 4 + STORY-165: 3) | — | — |
| Waves closed | 2 (74, 75) + wave-73 tail close | — | — |
| Maintenance runs completed | 1 (maint-2026-07-11) | — | — |
| Version released | v0.12.1 (tooling/process patch; zero behavioral src/ changes) | — | — |
| Wave-74 adversary passes | 13 | ≤4 target | OVER (but high gate value: 8 defects caught) |
| Wave-75 adversary passes | 7 | ≤4 target | OVER (improvement from 13; currency sweep credit) |
| Wave-74 substantive defects caught at gate | 8 | — | HIGH VALUE |
| Wave-75 substantive defects caught at gate | 3 | — | HIGH VALUE |
| Wave-74 per-story passes (STORY-164) | 8 passes trajectory 6→4→3→2n→2→1n→2n→1n | — | HIGH |
| Wave-75 per-story passes (STORY-165) | 9 passes trajectory 1→0→0→2→0→1→0→0→0 | — | HIGH |
| ADVERSARY-RELAY-UNRELIABLE-001 recurrences | 3rd (maint-07-11) + 4th+ (wave-74 5+ incidents) | 0 target | FAILED |
| DRIFT-ENGINE-CHECKOUT-GUARD-001 recurrences | 2nd (maint-07-11 Pass 1 VOID) | 0 target | FAILED |
| Engine issues filed (D-437) | 4 (vsdd-factory #635/#636/#637/#638) | — | POSITIVE |
| Input-hash scan at wave-75 gate | MATCH=119 STALE=0 | STALE=0 | PASS |
| Back-merge type | SQUASH (DRIFT-BACKMERGE-SQUASH-001) | merge-commit | FAIL |

---

## 1. Cost Analysis

**No explicit cost data available.** `cost-summary.md` is absent or unpopulated.
Session-density and adversary-pass counts are used as proxy cost indicators.

### Activity sequence and estimated cost density

| Date | Activity | D# | Estimated cost intensity |
|------|-----------|----|--------------------------|
| 2026-07-11 | Wave-73 close | D-428 | LOW (gate already complete; close formalities) |
| 2026-07-11 | maint-2026-07-11 | D-429 | MEDIUM-HIGH (PR #396: 17 findings fixed + adversary convergence + 8 sweeps) |
| 2026-07-11 | Wave-74 plan gate + STORY-164 delivery | D-430, D-431 | HIGH (8 per-story adversary passes; CI 12/12 run) |
| 2026-07-12 | Wave-74 gate close | D-432 | VERY HIGH (13 wave-level adversary passes; highest single-wave cost this project) |
| 2026-07-13 | Wave-75 plan gate + STORY-165 delivery | D-433, D-434 | HIGH (9 per-story adversary passes; CI 13/13 run) |
| 2026-07-13 | Wave-75 gate close + v0.12.1 release | D-435, D-436 | HIGH (7 wave-level adversary passes + release chain) |
| 2026-07-14 | Post-wrap triage | D-437 | LOW (engine issue filing + STORY-166 re-scope) |

### Highest-cost operations (estimated)

1. **Wave-74 gate convergence** — 13 wave-level adversary passes for a 4-point governance
   story. This is the highest single-gate pass count in the project's history at wave scope.
   The ADVERSARY-RELAY-UNRELIABLE-001 incidents (5+ in this session) added extra re-dispatch
   overhead on top of the 13 passes. Estimated 2-3× normal wave-gate cost.

2. **STORY-164 per-story adversary convergence** — 8 passes with trajectory 6→4→3→2n→2→1n→2n→1n.
   First-pass finding count of 6 is the highest recorded for any per-story run to date.
   Governance documentation stories with dense verifiable claims are systematically expensive.

3. **STORY-165 per-story adversary convergence** — 9 passes. Despite being a lighter story
   (3 pts vs 4 pts), per-story passes exceeded STORY-164 by one. The citation-mandate story
   instantiated the exact defect it was designed to prevent (F-S165P1-001 fabricated test
   name). This self-referential catch required additional passes.

4. **maint-2026-07-11 Route A** — 17 findings fixed in PR #396 + complex adversary
   convergence pattern (VOID + F-P1-001 + F-P1r-002 + finding passes before 3/3).
   DRIFT-ENGINE-CHECKOUT-GUARD-001 2nd recurrence (Pass 1 VOID) added one extra dispatch.

### Cost efficiency observations

**This release delivered 7 story points (the lowest per-release in any reviewed cycle) with
disproportionately high adversary overhead.** The v0.12.0 cycle delivered 58 pts across 9
stories with ~38 adversary passes total; the v0.12.1 cycle delivered 7 pts with 21+ wave-level
passes alone (wave-74: 13, wave-75: 7, plus per-story and maintenance passes). The per-point
adversary cost for E-11 governance stories is roughly 5× higher than implementation stories.

**Recommendation:** Track E-11 governance stories under a separate cost-per-point envelope
in benchmarks.yaml. The current ≤4 pass target is calibrated against implementation waves and
is not meaningful for governance-only waves. A separate governance-wave target of ≤8 passes
(reflecting the real baseline) would produce actionable benchmarks. See PROP-V0.12.1-03.

---

## 2. Timing Analysis

**Wall-clock span:** ~4 calendar days (2026-07-11 to 2026-07-14).

### Activity sequence

| Activity | Date | Decisions | Notes |
|----------|------|-----------|-------|
| Wave-73 close | 2026-07-11 | D-428 | 6 passes; S-7.02 satisfied; STORY-164 drafted |
| maint-2026-07-11 | 2026-07-11 | D-429 | 8 sweeps; Routes B/C deferred; 3rd relay recurrence |
| Wave-74 plan gate + delivery | 2026-07-11 | D-430, D-431 | STORY-164 delivered same day as maint close |
| Wave-74 gate close | 2026-07-12 | D-432 | 13 passes; 8 defects; STORY-165 drafted |
| Wave-75 plan gate + delivery | 2026-07-13 | D-433, D-434 | STORY-165 delivered same day as wave-74 gate close |
| Wave-75 gate close + release | 2026-07-13 | D-435, D-436 | 7 passes; v0.12.1 released same session |
| Post-wrap triage | 2026-07-14 | D-437 | 4 engine issues; STORY-166 re-scoped |

**Bottleneck:** Wave-74 gate (2026-07-12) was the peak-density point — 13 adversary passes
across a full day, with 5+ ADVERSARY-RELAY-UNRELIABLE-001 incidents adding re-dispatch
overhead to each affected pass. This single gate consumed more calendar time than the entire
wave-75 delivery plus gate.

**Compression observation:** v0.12.1 was planned, delivered, gated, and released entirely
within 3 active calendar days (2026-07-11/12/13). The compressed timeline was manageable
only because the product delta was zero src/ changes (pure tooling/process patch). A
behavioral release under this schedule would have required holdout evaluation, extending
the timeline by at least one additional day.

**Back-merge latency (DRIFT-BACKMERGE-SQUASH-001):** The squash back-merge PR #400 creates
history divergence (main fedcea4 not ancestor of develop 7b11b83) that must be resolved at
the next release. This is a 0-day latency issue now but will add friction at the next release
PR if not pro-actively resolved. See PROP-V0.12.1-02.

---

## 3. Convergence Analysis

### Wave adversarial convergence — this period

| Wave | Story | Total Passes | Substantive Defects | Streak | Trajectory |
|------|-------|-------------|---------------------|--------|------------|
| Wave-73 | STORY-162+163 | 6 | (prior review period) | 3/3 (P4/P5/P6) | 4→2→1→0→1n→0 |
| Wave-74 | STORY-164 | 13 | 8 | 3/3 (W11/W12/W13) | 2→0→1→1→0→1→0→3→1→1→0→2n→1n |
| Wave-75 | STORY-165 | 7 | 3 | 3/3 (W5/W6/W7) | 2→0→0→1→0→0→0 |

**vs. prior-review targets:**
- Target ≤4 passes: wave-74 = 13 (EXCEEDED 3.25×), wave-75 = 7 (EXCEEDED 1.75×)
- Target NOT-CLEAN ≤1: wave-74 = 8 (EXCEEDED), wave-75 = 3 (EXCEEDED)

**These exceedances are expected and appropriate for governance waves.** The targets were
calibrated against implementation waves (wave-72 = 4 passes). E-11 governance stories produce
documentation artifacts dense with verifiable claims (line anchors, version numbers, status
fields, test counts) — each claim is an adversary target. The wave-74 gate value was concrete:
8 substantive defects that would have persisted in factory-artifacts were found and fixed.

**Currency sweep effectiveness (AC-165-003):** The delivery-doc currency sweep ran as a
mandatory pre-adversary step for wave-75 (first mandatory execution). Wave-75 achieved
convergence in 7 passes vs. wave-74's 13 — a 46% reduction. The sweep did NOT prevent all
currency findings (F-W75G-P1-002 LOW line citation and F-W75G-P4-001 MEDIUM blanket
provenance still reached the gate), confirming Lesson 2: the sweep handles macro drift but
fine-grained citations and internal sweep-document claims require adversary coverage. The
overall pass-count reduction is real and attributable to the pre-gate sweep.

### Per-story adversary convergence

| Story | Passes | First-Pass Findings | Streak | Trajectory |
|-------|--------|---------------------|--------|------------|
| STORY-164 | 8 | 6 | 3/3 (P6/P7/P8) | 6→4→3→2n→2→1n→2n→1n |
| STORY-165 | 9 | 1 | 3/3 (P7/P8/P9) | 1→0→0→2→0→1→0→0→0 |

**STORY-164 first-pass finding count of 6 is the highest recorded for any per-story run.**
The trajectory shows that each finding pass still revealed new issues (4 findings at P2, 3 at P3)
before the nitpick-only phase and final convergence. This is characteristic of a governance
story whose delivery artifacts were not pre-swept.

**STORY-165 had a better first pass (1 finding) despite a higher total pass count (9 vs 8).**
The higher total is explained by the self-referential defect: STORY-165 (citation-mandate story)
had fabricated citations in its own delivery doc (F-S165P1-001 HIGH fabricated test name,
F-S165P4-001 HIGH fabricated finding ID). Both finding rounds produced two findings each (P4:
2 findings including the finding-ID collision). The "checker gets checked" dynamic drove up
the total despite a cleaner first pass.

### Maintenance convergence (maint-2026-07-11)

Route A PR #396 convergence pattern: VOID → F-P1-001 → F-P1r-002 → 1/3 → F-P2-001 →
1/3 → 2/3 → 3/3 CONVERGED. The VOID pass (DRIFT-ENGINE-CHECKOUT-GUARD-001 2nd recurrence)
added one extra dispatch at the top. After the VOID recovery, convergence proceeded normally.
Two finding rounds (P1 and P2) before the 3/3 streak — efficient for 17 findings fixed.

**Adversary false-positive rate this period:**
- Wave-74 gate W9/W10: 2 ACCEPTED-REFUTED findings (adversary filed; orchestrator ground-truth
  read refuted both). Rate: 2/13 passes (15%).
- Wave-75 gate W3: F-W75G-P3-002 filed as finding against ledger; research-adjudicated
  redundant to PG-W75-FINDING-ID-DUAL-SCHEME. No streak impact (ledger-redundancy ≠ false defect,
  but it required an extra validation step).

**Net adversary false-positive rate is acceptable** (~15% for wave-74 gate), but the "claim
first, check second" pattern (now AC-165-004) when applied correctly should reduce this in
wave-76 and beyond.

---

## 4. Agent Behavior Analysis

### Per-agent behavior summary

**Wave-level adversary:**
- Wave-74 gate: W9/W10 ACCEPTED-REFUTED findings (AC-165-004 root cause: "claim first, check
  second"). This was the trigger for PG-W74-GROUND-TRUTH-AUDIT-FIRST codification.
- Wave-75 gate: W3 F-W75G-P3-002 ledger-redundancy finding (research-adjudicated redundant;
  streak not reset). Research-adjudication as the triage path for redundancy claims worked correctly.
- No stale-SHA false alarms in wave-74 or wave-75 (PAT-009 RESOLVED-EFFECTIVE confirmed durable;
  zero incidents across 20 gate passes combined).

**Per-story adversary (STORY-164/165):**
- STORY-164 P1 (6 findings): consistent with governance-document density. No structural
  anomalies in finding types.
- STORY-165 P1 (1 finding): improved first-pass discipline possibly due to STORY-165 being
  explicitly scoped around citation validation — the author may have exercised more citation care.
  But P4 (2 findings including the fabricated finding-ID) shows the discipline did not persist
  across all delivery artifacts.
- STORY-165 fabricated-citation pattern (F-S165P1-001 + F-S165P4-001 HIGH): the citation-
  mandate story's own evidence instantiated the defect it was designed to prevent. This is
  a second recurrence of PAT-012 (docs-agent-content-fabrication) within the governance-story
  class. The findings were caught by adversary convergence before wave gate; no gate-fix PR needed.

**ADVERSARY-RELAY-UNRELIABLE-001 (ANOMALOUS — escalating):**
- maint-2026-07-11 sweep-4: 3rd lifetime occurrence. Synchronous dispatch (PROP-V0.12.0-03)
  was in effect but this sweep was a holdout-freshness agent — apparently non-adversary agents
  can also exhibit the relay failure.
- Wave-74 gate: 5+ silent-idle incidents across 13 passes — the highest concentration in a
  single session. IMPORTANT: synchronous dispatch (`run_in_background: false`) was applied
  per PROP-V0.12.0-03 recommendation (D-423). Some synchronous dispatches also silenced before
  returning structured summaries, meaning the workaround is PARTIALLY EFFECTIVE at best.
- Total lifetime occurrences: 8+ (maint-07-08: 2, maint-07-11: 1, wave-74 gate: 5+).
- No further relay incidents in wave-75 gate (7 passes, zero incidents noted).
- Pattern: the relay failure appears correlated with high-pass-count sessions and large
  output buffers. Wave-75 (lower total output) had zero incidents; wave-74 (largest single-
  session output) had 5+.

**DRIFT-ENGINE-CHECKOUT-GUARD-001 (2nd recurrence):**
- maint-2026-07-11 Pass 1 returned VOID. This is the 2nd confirmed recurrence of the
  checkout-guard issue. No further VOID passes in wave-74 or wave-75 gate runs.
- Pattern: maintenance sweep dispatches are more vulnerable to checkout guard issues than
  wave-gate dispatches (different working-tree setup at dispatch time).

**State-manager:**
- STORY-INDEX maintained correctly across all versions (v3.43 through v3.57).
- The D-432→D-434 cell correction (STORY-INDEX v3.55) was caught and applied at wave-75
  delivery — correct audit behavior.
- STORY-166 re-scope (v1.0→v1.1, 5→3 pts) executed cleanly at D-437.

**pr-manager (STORY-165 merge):**
- HUMAN-AUTHORIZED at orchestrator merge-authorization gate per DF-MERGE-AUTH-CLASSIFIER-001.
  F-W75G-P1-001 subsequently identified that the D-434 record lacked merge-auth attribution.
  Fixed at wave gate. Adjudication note: "delivery D-records SHOULD carry merge-auth attribution
  going forward." This is a recurring documentation discipline gap (not a policy violation).

---

## 5. Gate Outcome Analysis

| Gate | First Try? | Outcome | Notes |
|------|-----------|---------|-------|
| Wave-73 close | NO (not first cycle run) | PASS | 6 passes; prior delivery's tail |
| maint-2026-07-11 Route A | NO (VOID + 2 finding rounds) | PASS | PR #396; 17 findings fixed; 3/3 streak |
| maint-2026-07-11 Routes B/C | DEFERRED | DEFERRED | Human-deferred; ROUTE-BC-DEFERRED-2026-07-11 |
| Wave-74 plan gate | YES (after pre-gate citation fix) | PASS | F-VAL-164-001 caught at pre-gate; fixed before formal opening |
| STORY-164 per-story | NO (6 findings at P1) | PASS | 8 passes; 3/3 streak P6/P7/P8 |
| Wave-74 integration gate | NO (13 passes) | PASS | 8 substantive defects fixed; highest-value gate this period |
| Wave-75 plan gate | YES | PASS | Consistency audit 2 MINORs fixed pre-gate |
| STORY-165 per-story | NO (1 finding at P1) | PASS | 9 passes; 3/3 streak P7/P8/P9 |
| Wave-75 integration gate | NO (7 passes) | PASS | 3 defects fixed; first mandatory currency sweep |
| v0.12.1 release PR | YES | PASS | CI 13/13; fast-forward to main |
| v0.12.1 back-merge PR | YES | PASS | Squash back-merge (DRIFT-BACKMERGE-SQUASH-001) |
| Post-wrap DF-VALIDATION-001 (4 engine issues) | n/a | VALIDATED | All 4 findings confirmed before issue filing |

**First-try pass rate:** 3/12 counted (25%). This is the lowest rate in any reviewed cycle,
driven almost entirely by the multi-pass per-story and wave-gate convergence runs. As noted
in prior reviews, "no first-try" at wave gates is correct behavior — the gates are designed
to find things per-story scope misses.

**Zero gate-fix PRs this cycle:** Unlike wave-71 (PR #381) and wave-72 (PR #391), neither
wave-74 nor wave-75 required a gate-fix PR. All gate findings were fixed within the factory-
artifacts tree (not in product code), consistent with the governance-only nature of both stories.

**AC-165-001 bin-selftest first run (wave-75 CI gate):** CI job 13 (`bin-selftest`) ran for
the first time on PR #398 and passed (22+10=32 bin/ self-tests green). This validated that
the CI wiring delivered by STORY-165 actually executes what it promises.

**AC-165-002 row-verify mandate first compliant execution:** 9 test-evidence table rows
cross-checked against actual CI output on PR #398. All 9 accurate — no fabricated counts.
The mandate passed its own inaugural test.

---

## 6. Wall Integrity Analysis

**All information asymmetry walls held during this period.**

**Holdout wall:** Holdout evaluation was N/A for all stories (zero src/ changes throughout
the entire release chain). No wall-integrity test was exercised by the holdout gate.

**Per-story adversary independence (STORY-164/165):**
- Wave-74 gate adversary correctly read factory-artifacts tree only. No evidence of adversary
  referencing per-story delivery notes during wave-level passes.
- Wave-75 gate adversary independence confirmed: F-W75G-P4-001 (blanket provenance claim) was
  identified by reading the currency-sweep.md document, not by cross-referencing implementation
  internals. Wall intact.

**Research agent (D-437 DF-VALIDATION-001):**
- All 4 engine-issue candidates were validated by research-agent before issue filing per
  DF-VALIDATION-001 policy. pg-validation-wave-75.md: 3 findings VALID (PG-W75-VALIDATE-
  CITATIONS-SYMBOL-GAP, PG-W75-FINDING-ID-DUAL-SCHEME, PG-W75-GATE-SUMMARY-VERSION-ATTRIBUTION).
  No wall violations observed.

**Merge-auth wall:**
- STORY-165 merge was HUMAN-AUTHORIZED at orchestrator gate per DF-MERGE-AUTH-CLASSIFIER-001.
  The F-W75G-P1-001 finding identified missing attribution in the D-record, not a policy
  violation. The merge itself was correctly gated.
- maint-2026-07-11 PR #396 merge by human per same policy (PG-MERGE-AUTH-SUBAGENT-CLASSIFIER,
  admin-bypass declined in auto-mode). Wall behavior correct.

**Engine issue triage wall (D-437):**
- STORY-166 re-scoping required examining STORY-166 v1.0 ACs and the engine issue candidates
  simultaneously. The research agent correctly scoped each AC as either project-side or engine-
  side without cross-contaminating the two contexts.

---

## 7. Quality Signal Analysis

### Holdout scores

No holdout evaluation this period. All stories (STORY-164, STORY-165) are E-11 governance
type with zero src/ changes. Product behavioral surface unchanged from v0.12.0.

### Test count

| Point in time | Cargo tests | Python bin/ tests | Notes |
|---------------|-------------|-------------------|-------|
| v0.12.0 release | 2392 | 32 (22+10) | Wave-72 close baseline |
| v0.12.1 release | 2392 | 32 (22+10) | No change; zero src/ delta |

No test regressions. No new tests (governance-only stories do not add behavioral tests).

### Security

| Finding | Status | Notes |
|---------|--------|-------|
| SEC-001 (wave-74 gate, LOW CWE-59) | ACCEPTED LOW | Accepted by human; DEFERRED to next feature wave per STATE.md |
| SEC-001/002 wave-74 gate LOW | ACCEPTED | No action required |
| SEC-003 (wave-74 gate, INFO) | DEFERRED to #392 | No urgency change |
| Wave-75 security | CLEAN | CI-yaml + CLAUDE.md docs only; no CWE-class exposure; adjudicated at gate |

**No exploitable findings.** Product attack surface unchanged from v0.12.0.

### Spec coherence trajectory

Not re-run this period (no spec changes; governance-only delivery). Last result: maint-2026-07-09
= 33/33 PASS (second consecutive). Third consecutive PASS target remains unverified pending next
maintenance sweep.

### Formal verification

No new VP runs. VP-039/041/042/043 from prior cycles unchanged and not retested (zero src/ delta).

### CI gate health

| CI gate | Status at v0.12.1 | Notes |
|---------|------------------|-------|
| cargo test --all-targets | 904 pass / 0 fail | Unchanged |
| cargo clippy -D warnings | PASS | Unchanged |
| cargo fmt --check | PASS | Unchanged |
| action-pin-gate | PASS | 13/13 jobs; bin-selftest SHA confirmed 40-char |
| changelog-gate | PASS | Unchanged |
| bin-selftest (new, AC-165-001) | PASS (first run) | 32 tests green |
| green-doc-tense-gate | PASS | Unchanged |
| trust-boundary | PASS | Unchanged |
| semantic-pull-request | PASS | All 5 PRs this cycle semantically titled |

**CI gate count increased from 12 to 13** with STORY-165's addition of `bin-selftest`. No regressions.

---

## 8. Pattern Detection (Cross-Run)

### ADVERSARY-RELAY-UNRELIABLE-001 — 3rd + 4th recurrences this period

| Cycle | Incidents | Context | Mitigation applied |
|-------|-----------|---------|-------------------|
| maint-2026-07-08 | 2 | Background dispatch | Sync dispatch as workaround |
| maint-2026-07-11 | 1 | Sweep-4 holdout (sync) | Sync dispatch applied; still silenced |
| wave-74 gate | 5+ | 13 adversary passes (sync) | Sync dispatch applied; 5+ still silenced |
| wave-75 gate | 0 | 7 adversary passes (sync) | No incidents |

**Pattern assessment:** Sync dispatch (`run_in_background: false`) is partially effective.
Wave-74's 5+ incidents confirm that sync dispatch does not eliminate the failure — it reduces
incident rate at low output volume but is overwhelmed at high pass-count sessions. The wave-75
gate (lower total output: 7 passes vs 13) saw zero incidents, consistent with the volume
hypothesis. This is now the most disruptive operational issue in the factory and warrants an
engine-level fix, not just a workaround. No engine issue was filed for this in D-437; see
PROP-V0.12.1-01.

### DRIFT-ENGINE-CHECKOUT-GUARD-001 — 2nd recurrence

| Cycle | Incident | Context |
|-------|----------|---------|
| (prior) | 1st | (from tech-debt register, prior to this review period) |
| maint-2026-07-11 | 2nd | PR #396 Pass 1 VOID |

No further recurrences in waves 74 or 75. Pattern: maintenance sweeps (different dispatch
setup) are more vulnerable than wave-gate dispatches. The 2nd recurrence in 2 reviewed cycles
suggests this is a persistent engine-level issue, not a one-off. Not yet filed as engine issue.

### DRIFT-BACKMERGE-SQUASH-001 — 2nd squash back-merge in 2 releases

| Release | Back-merge type | Effect |
|---------|-----------------|--------|
| v0.11.5 (prior cycle) | Squash | PG-GITFLOW-SQUASH-BACKMERGE; required join commit at v0.12.0 |
| v0.12.1 (this cycle) | Squash PR #400 | DRIFT-BACKMERGE-SQUASH-001; trees identical; history diverges |

**Pattern: squash back-merges are the default behavior without explicit mandate.** Two squash
back-merges in two releases means the pattern recurs each release cycle. The mitigation
(BACKMERGE-FAST-FORWARD-001, identified in prior review as a codification candidate) has not
been codified as a story. Must be acted upon before the next release. See PROP-V0.12.1-02.

### E-11 governance wave cost profile — new pattern observation

| Wave | Stories | Story pts | Per-story passes | Wave-gate passes | Total passes | Passes/pt |
|------|---------|-----------|-----------------|-----------------|--------------|-----------|
| Wave-73 | 2 | 5 | ~6 combined | 6 | ~12 | ~2.4 |
| Wave-74 | 1 | 4 | 8 | 13 | 21 | 5.25 |
| Wave-75 | 1 | 3 | 9 | 7 | 16 | 5.3 |

The passes/pt metric for single-story governance waves (5+) is 2-3× higher than multi-story
governance waves (wave-73: ~2.4) and implementation waves (benchmark ~1-1.5 from prior cycles).
Single-story waves have no opportunity to amortize wave-gate overhead across stories.
Multi-story governance waves (wave-73) converge more efficiently per point. See PROP-V0.12.1-03.

### Follow-through on prior review (session-review-2026-07-10-v0.12.0.md) proposals

| Prior Item | Expected Action | Status |
|------------|----------------|--------|
| PROP-V0.12.0-01 (BREAKING holdout sweep) | Implement before next BREAKING story | PARTIALLY ADDRESSED — breaking-change-delivery-protocol.md created (D-429, STORY-164 v1.1 AC-164-005); workflow not yet wired into story template per se; document exists and is linked from CLAUDE.md |
| PROP-V0.12.0-02 (3/3 for docs PRs) | Formalize as policy before next docs PR | NOT ADDRESSED this period |
| PROP-V0.12.0-03 (sync adversary dispatch) | Add to maintenance-config.yaml | APPLIED but INSUFFICIENT — sync dispatch active, still 5+ relay incidents at wave-74 gate |
| PROP-V0.12.0-04 (VHS guidance) | Before next wave demo recordings | NOT TRIGGERED (zero VHS recordings this period; governance-only) |
| PROP-V0.12.0-05 (E-11 hash hook note) | Next E-11 story authoring | SUPERSEDED — PG-HASH-HOOK-DIVERGENCE now tracked at vsdd-factory#637 (D-437); CLAUDE.md advisory note remains; no local story action needed |
| PROP-V0.12.0-06 (PAT-009 RESOLVED-EFFECTIVE) | At next pattern-database review | CONFIRMED APPLIED — pattern-database.yaml shows RESOLVED-EFFECTIVE (zero recurrences across 20+ passes in this cycle) |
| PROP-V0.12.0-07 (PC-001/013 batch refactor) | Wave-73 planning | NOT ACTED ON — waves 73/74/75 all governance-only; defer to next implementation wave |
| PROP-MAINT-05 (advisory-race annotation) | Next maintenance sweep | NOT TRIGGERED (no advisory races in maint-2026-07-11) |
| PROP-MAINT-06 (HIGH sweep-3 mechanism-verify) | Next maintenance sweep | NOT TRIGGERED (no HIGH sweep-3 findings in maint-2026-07-11) |
| PROP-MAINT-07 (import-style batch story) | Wave-73 planning | NOT ACTED ON — defer to next implementation wave |
| PROP-MAINT-08 (holdout freshness heuristic) | Next maintenance sweep | NOT TRIGGERED (zero BREAKING stories this period) |
| ADV-4 OVERDUE (ci.yml build-dep-chain) | Human disposition 2+ runs overdue | CLOSED-STALE — tech-debt-register confirms "ADV-4 OVERDUE flag stale (already RESOLVED PR #369)" per D-391; entry was a stale pointer; no further action needed |

**Score: 2/12 fully addressed (PROP-V0.12.0-05 superseded positively; ADV-4 closed as stale).
1/12 partially addressed (PROP-V0.12.0-01). 9/12 deferred (either untriggered or awaiting
next wave). Only 1/12 was applied but inadequate (PROP-V0.12.0-03 needs escalation).**

---

## 9. Governance Policy Audit

| Policy | Enforcement Status | Notes |
|--------|--------------------|-------|
| `append_only_numbering` | PASS | IDs preserved throughout; finding-ID discipline maintained in wave-75 gate artifacts (G-form dogfood) |
| `state_manager_runs_last` | PASS | All STATE.md updates followed artifact commits; D-numbers sequential |
| `df_validation_001` | PASS | D-437 engine issues: all 4 validated before filing (pg-validation-wave-75.md) |
| `df_convergence_before_merge_001` | PASS | All PRs (#396/397/398/399/400) achieved 3/3 before merge |
| `df_sibling_sweep_001` | PASS | F-W75G-P4-001 catch: blanket-provenance over-claim in currency-sweep.md; DF-SIBLING-SWEEP-001 pattern applied by adversary |
| `bc_h1_is_title_source_of_truth` | PASS | No BC title changes requiring cascade |
| `vp_index_is_vp_catalog_source_of_truth` | PASS | No VP changes this cycle |
| S-7.02 (wave cycle-close codification) | PASS | Wave-74: STORY-165 (4 PGs codified); Wave-75: STORY-166 drafted (3 PGs + 4 engine issues) |
| `DF-MERGE-AUTH-CLASSIFIER-001` | PASS | Both merges human-authorized in main thread |
| `PG-W71-CODEREVIEW-ARTIFACT` (code-review.md per gate) | PASS | cycles/wave-74/wave-gate/code-review.md + cycles/wave-75/wave-gate/code-review.md both present with full finding enumerations |
| `PG-W74-DELIVERY-DOC-CURRENCY` (first mandatory) | PASS | Currency sweep executed pre-adversary at wave-75 gate; AC-165-003 compliance confirmed |
| `PG-W74-PRDESC-ROW-VERIFY` (first mandatory) | PASS | 9 rows row-verified on PR #398; all accurate |

**New policy in force this cycle:** PG-W74-PRDESC-ROW-VERIFY and PG-W74-DELIVERY-DOC-CURRENCY
both took effect on their own delivery PR/gate — self-validating first executions.

---

## Improvement Proposals

### Proposal 1: File engine issue for ADVERSARY-RELAY-UNRELIABLE-001 (PROP-V0.12.1-01)
- **Category:** agent / engine-escalation
- **Priority:** P1 HIGH
- **Evidence:** 8+ lifetime relay failures. Wave-74 gate: 5+ incidents with sync dispatch active.
  The synchronous workaround is insufficient at high-output sessions. Relay failure wastes one
  full dispatch per incident and creates session-state uncertainty about whether findings were
  delivered. D-437 filed 4 engine issues (#635/#636/#637/#638) but none targeted relay failure.
- **Recommendation:** File a vsdd-factory engine issue for ADVERSARY-RELAY-UNRELIABLE-001
  (separate from #635-#638 which cover different concerns). Issue should describe: the relay
  failure pattern (agent writes artifact, reports brief confirmation, then goes idle); the
  output-volume correlation hypothesis (wave-74 5+ incidents vs wave-75 0 incidents at 7 passes);
  the confirmed partial workaround (sync dispatch); and the remaining gap (sync dispatch fails
  at wave-gate scale). Pending DF-VALIDATION-001 validation.
- **Affected files:** drbothen/vsdd-factory (new issue)
- **Risk:** Issue filing has no risk. Delay in fixing has cost: every 13-pass gate session will
  encounter 3-7 relay failures.
- **Revisit trigger:** Before wave-76 gate dispatch

### Proposal 2: Codify BACKMERGE-FAST-FORWARD-001 as a Story (PROP-V0.12.1-02)
- **Category:** workflow / policy-codification
- **Priority:** P2 MEDIUM
- **Evidence:** Two consecutive releases (v0.11.5 and v0.12.1) used squash back-merges.
  Each squash back-merge severs shared history (DRIFT-BACKMERGE-SQUASH-001) and requires
  resolution at the next release. BACKMERGE-FAST-FORWARD-001 was identified as a codification
  candidate in the prior review (v0.12.0 governance candidate #4) but was not converted to a
  story. There is now a specific drift item in STATE.md that must be resolved at the next release.
- **Recommendation:** Draft a small governance story (E-11, 1-2 pts) that codifies BACKMERGE-
  FAST-FORWARD-001 in CLAUDE.md (add to "Git Workflow" section: "Back-merges from main to
  develop after a release MUST use merge-commit, not squash") and adds a CLAUDE.md Project
  References row pointing to a new `.factory/maintenance/backmerge-policy.md` guidance doc.
  Also resolve the current DRIFT-BACKMERGE-SQUASH-001 item by re-merging at next release.
- **Affected files:** CLAUDE.md, new `.factory/maintenance/backmerge-policy.md`
- **Risk:** Low. Documentation change; no behavioral impact. The policy is already implied by
  the v0.12.0 release precedent (fast-forward back-merge that reunified histories).
- **Revisit trigger:** Before next release cut (wave-76 release)

### Proposal 3: E-11 Governance Wave Cost Envelope — Separate Benchmark (PROP-V0.12.1-03)
- **Category:** workflow / benchmarking
- **Priority:** P2 MEDIUM
- **Evidence:** E-11 single-story governance waves average ~5 adversary passes/pt (wave-74:
  5.25, wave-75: 5.3). Multi-story governance waves average ~2.4 passes/pt (wave-73). The
  ≤4-pass convergence target is calibrated for implementation waves; governance waves will
  always "fail" this target. This masks true convergence quality signals and creates confusion
  about whether a 13-pass gate is a regression.
- **Recommendation:** Add an `e11_governance_wave_target` row to benchmarks.yaml with a
  separate baseline: single-story E-11 target ≤9 passes (reflecting wave-75 result with
  currency sweep), multi-story E-11 target ≤8 passes (wave-73 = 6 passes, headroom). Report
  governance waves against the governance target, not the implementation target. This will
  immediately flag wave-74 (13 passes) as an outlier even within the governance envelope,
  motivating investigation, while validating wave-75 (7 passes) as within target.
- **Affected files:** `.factory/session-reviews/benchmarks.yaml`
- **Risk:** None. Additive benchmark entry; no process changes.
- **Revisit trigger:** Before wave-76 gate dispatch

### Proposal 4: Schedule bin-touch Housekeeping PR for ROUTE-W74-DEFERRED (PROP-V0.12.1-04)
- **Category:** workflow / tech-debt
- **Priority:** P3 LOW
- **Evidence:** ROUTE-W74-DEFERRED now accumulates: wave-74 code-review MINOR ×2 + NIT ×4
  + OBS ×2 (against bin/test_validate_citations.py, bin/validate-citations, bin/changelog-
  gate-check) + wave-75 NIT-1 (hardcoded test counts in ci.yml bin-selftest step names)
  = 11 total deferred items. STORY-166 is scheduled for wave-76 and touches bin/validate-
  citations for AC-166-001 (symbol-at-line validator extension). The STORY-166 PR will be
  a bin-touch PR — exactly the trigger condition for resolving ROUTE-W74-DEFERRED.
- **Recommendation:** When delivering STORY-166, include a resolution pass over ROUTE-W74-
  DEFERRED as part of the PR scope. Specifically: MINOR-1 (dead `_run()` helper), MINOR-2
  (parse_line docstring), NIT-1 (inline imports), NIT-4 (unnecessary f-string), OBS-1
  (docstring repo-root claim), NIT-1-W75 (hardcoded test counts). Mark each item as RESOLVED
  in the wave-76 code-review.md findings table. This prevents the deferred list from growing
  across additional waves.
- **Affected files:** bin/test_validate_citations.py, bin/validate-citations,
  .github/workflows/ci.yml (wave-76 delivery scope)
- **Risk:** Low. Cosmetic code-quality improvements; no behavioral changes.
- **Revisit trigger:** STORY-166 delivery planning

---

## Metrics for Next Run

**Convergence targets (governance waves — PROP-V0.12.1-03 proposed separate envelope):**
- E-11 single-story wave gate: target ≤9 passes (baseline: wave-75 = 7 passes with currency sweep)
- E-11 multi-story wave gate: target ≤8 passes (baseline: wave-73 = 6 passes)
- Per-story adversary first-pass findings: target ≤2 (baseline: STORY-164 = 6, STORY-165 = 1)
- Wave adversary substantive defects: no target (high gate value = correct behavior)

**Convergence targets (implementation waves — unchanged):**
- Wave adversary passes to streak: target ≤4 (benchmark: wave-72=4)
- Wave adversary NOT-CLEAN: target ≤1 (benchmark: wave-72=1)

**Quality targets (unchanged):**
- Holdout mean ≥ 0.85 (sustained; next implementation wave will trigger holdout eval)
- Spec coherence: target 33/33 PASS (third consecutive; next maintenance sweep)
- Input-hash STALE=0 (sustained; MATCH=119 at wave-75 close)
- Test count ≥ 2392 (unchanged; STORY-166 may add bin/ tests)

**Pattern targets:**
- ADVERSARY-RELAY-UNRELIABLE-001: target 0 incidents (current: 8+ lifetime; engine fix needed)
- DRIFT-BACKMERGE-SQUASH-001: target RESOLVED at next release (merge-commit back-merge)

**Carry-forward items for next maintenance run / wave-76:**

| Item | Source | Priority |
|------|---------|----------|
| ADVERSARY-RELAY-UNRELIABLE-001 engine issue | PROP-V0.12.1-01 | P1 — file before wave-76 gate |
| DRIFT-BACKMERGE-SQUASH-001 | STATE.md Drift Items | P2 — resolve at next release |
| BACKMERGE-FAST-FORWARD-001 codification story | PROP-V0.12.1-02 | P2 — wave-76 or adjacent |
| ROUTE-W74-DEFERRED (11 items) | ROUTE-W74-DEFERRED | P3 — resolve during STORY-166 delivery |
| PROP-V0.12.0-02 (3/3 docs PRs) | improvement-backlog | P1 HIGH — before next docs maintenance PR |
| PROP-V0.12.0-04 (VHS guidance) | improvement-backlog | P2 MEDIUM — before next wave with demos |
| PROP-V0.12.0-07 (PC-001/013 batch refactor) | improvement-backlog | P2 MEDIUM — next implementation wave |
| PROP-MAINT-07 (import-style) | improvement-backlog | P2 MEDIUM — 3 reviews old |
| ROUTE-BC-DEFERRED-2026-07-11 (spec-index fixes + holdout repairs) | STATE.md Carry-forwards | P2 MEDIUM |
| SEC-001 (next feature wave) | STATE.md | P2 MEDIUM |
| PERF-RERUN-001 (quiescent conditions) | STATE.md | P2 MEDIUM |
| STORY-166 wave-76 plan gate | STATE.md | P2 MEDIUM — first candidate after session review |
| CR-001/002/003 (wave-73 D-428 code-review deferred) | STATE.md Carry-forwards | P3 LOW |

*Session review authored 2026-07-13, v0.12.1 release chain. Covers D-428..D-437, wave-73
close, maint-2026-07-11, wave-74 (STORY-164), wave-75 (STORY-165), v0.12.1 release, post-
wrap triage. Human decisions pending within 72h.*
