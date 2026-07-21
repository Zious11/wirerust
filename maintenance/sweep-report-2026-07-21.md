---
document_type: maintenance-sweep-report
run_id: maint-2026-07-21
date: 2026-07-21
prior_run: maint-2026-07-11
trigger: scheduled
base_commit: 1e967bad
version: v0.13.0+
sweeps_run: [1, 2, 3, 4, 5, 7, 8]
sweeps_skipped: [6, 9]
skip_reasons:
  sweep_6_dtu: "dtu_required: false — no DTU clones to validate"
  sweep_9_a11y: "CLI-only project — no UI surface for accessibility audit"
develop_head_open: 1e967bad
develop_head_close: 6c47c0ef
prs_merged: [422, 423, 424, 425, 431]
---

# Maintenance Sweep Report — maint-2026-07-21

**Run ID:** maint-2026-07-21
**Date:** 2026-07-21
**Prior run:** maint-2026-07-11
**develop HEAD at open:** 1e967bad (v0.13.0+, wave-84 gate-fix final)
**develop HEAD at close:** 6c47c0ef (PR #431 IEC-104 doc-drift fix squash-merged)
**Sweeps executed:** 1 (deps), 2 (doc-drift), 3 (patterns), 4 (holdouts), 5 (perf), 7 (spec-coherence), 8 (P1/P2 review)
**Sweeps skipped:** Sweep 6 (DTU — dtu_required: false), Sweep 9 (a11y/design — CLI product, no UI)

---

## Summary

| Sweep | Status | Findings | PRs Opened | Issues Created |
|-------|--------|----------|-----------|----------------|
| 1 — Dependency Audit | CLEAN | 0C/0H/0M/3L (DEP-007 pre-existing + DEP-008/009 new) | 0 | 0 |
| 2 — Documentation Drift | FINDINGS (all fixed) | 1H+3M+1L new; 8 prior resolved | 0 (PR #431 merged) | 0 |
| 3 — Pattern Consistency | FINDINGS | 4 new (2 LOW + 2 NIT); 7 carried; 2 resolved | 0 | 0 |
| 4 — Holdout Freshness | FINDINGS (repaired) | 18 PASS / 4 FAIL-STALE repaired / 0 bug | 0 | 0 |
| 5 — Performance Baseline | PASS | 5 OK / 2 WARN-noise / 0 CRIT | 0 | 0 |
| 7 — Spec Coherence | FINDINGS (2 fixed) | 7 carried + 4 new; 2 fixed by spec-steward | 0 | 0 |
| 8 — P1/P2 Review | NO P1; P2 target-passed | 2 P2 items marked TARGET-PASSED | 0 | 0 |

## Overall Health: [HEALTHY / NEEDS_ATTENTION / DEGRADED]

**Status: HEALTHY**

0 CRITICAL, 0 HIGH product findings remaining after close. DOC-011 HIGH (IEC-104 absent from README) was found and fixed in the same run via PR #431. All 4 holdout stales repaired (HS-087/123/125/132, HS-INDEX v2.14). Spec-index drift repaired (ARCH-INDEX v2.20, STORY-INDEX v3.87). AC-149-003 clean PASS (23.659µs) in VALID environment — first confirmed quiescent result since Jun-22. Register updated v2.0. 5 PRs merged under explicit human authorization (D-490).

---

## Dependency Audit

**Source:** maintenance/dependency-audit-findings-2026-07-21.md

### New Vulnerabilities

| Dependency | Version | CVE/Advisory | Severity | Fix Available | Action |
|-----------|---------|-------------|----------|--------------|--------|
| (none) | — | — | — | — | No action required |

**Verdict: CLEAN.** `cargo audit` 0 advisories against 175 locked crates (advisory DB: 1166 entries, +7 from maint-2026-07-11; all 7 new advisories verified non-applicable to our tree). `cargo deny` 0 errors, 1 pre-existing warning (DEP-007 syn 1.0.109/2.0.117 duplicate, deferred). Crate count down 193→175 (−18) due to PR #420 dep-soak.

**New duplicate findings:**

| Finding ID | Category | Dependency | Severity | Disposition |
|---|---|---|---|---|
| DEP-008 | build-dep dual version | rand 0.8.7 (phf_generator/tls-parser build) + rand 0.9.5 (proptest dev) | LOW | DEFERRED — no advisory; build/dev paths only |
| DEP-009 | build-dep dual version | rand_core 0.6.4 + rand_core 0.9.5 (sibling of DEP-008) | LOW | DEFERRED — no advisory; resolves alongside DEP-008 |

**Dependabot action PRs (#422–425):** All 4 verified soak-eligible (≥8 days from upstream release). All 4 CI-infrastructure only (workflow YAML; no Cargo.lock impact). Adopted as batch per human D-490.

**syn 3.0.2 major-version note:** DO NOT ADOPT — semver-major from 2.x; multi-crate coordination exercise required. Syn 2.x (2.0.119, soaks 2026-07-23) remains the target via DEP-SOAK-FOLLOWUP-2026-07-27.

---

## Documentation Drift

**Source:** maintenance/doc-drift-findings-2026-07-21.md

**Prior findings:** All 8 findings from maint-2026-07-11 RESOLVED as of HEAD 1e967bad.

### Stale Documentation

| Document | Section | Drift Type | Severity | Action |
|----------|---------|-----------|----------|--------|
| README.md (multiple sections) | Features, Protocol table, Analyze flags, Architecture | IEC-104 analyzer absent entirely — no bullet, no row, no entry, no section | HIGH | PR #431 FIXED |
| docs/adr/0001:36–65 | StreamDispatcher struct snippet + rule table | Missing iec104 field, Iec104 variant, Rule 8 (port 2404) | MEDIUM | PR #431 FIXED |
| docs/adr/0002:149–188 | Existing Analyzers table + Deviations section | Missing IEC-104 row and deviations entry | MEDIUM | PR #431 FIXED |
| CLAUDE.md Project References | docs/adr/ row | ADR-013 omitted from enumeration | MEDIUM | PR #431 FIXED |
| src/cli.rs:259 | ENIP write-burst arg doc-comment | "1-second window" vs "1s window" format inconsistency | LOW | PR #431 FIXED |

All 5 findings resolved by PR #431 (6c47c0ef). PR #431 gate review: APPROVE (pr-reviewer, 1 cycle, 0 blocking). 3 non-blocking residuals deferred to ROUTE-DOC-DEFERRED-2026-07-21 (ADR-0001 Consequences Iec104Analyzer omission; ADR-0002 Deviations heading stale; ADR-0012 stale "7 supported" + missing port 2404). All ACCEPTED/DEFERRED per human D-490.

---

## Pattern Consistency

**Source:** maintenance/pattern-findings-2026-07-21.md

Cargo clippy (`--all-targets -D warnings`) and `cargo fmt --check` both pass cleanly at 1e967bad. No new unjustified `#[allow]` suppressions.

**Prior items resolved (since b5e1e15):**

| ID | Resolution |
|----|------------|
| PC-NEW-001 | 9 spurious `#[allow(unused)]` on pub consts in dnp3.rs removed |
| PC-NEW-002 | All 6 `#[allow(clippy::too_many_arguments)]` in dnp3.rs now have justification comments |

### Inconsistencies Detected

| Pattern | Expected | Found In | Severity | Action |
|---------|----------|----------|----------|--------|
| IEC-104 import-style (PAT-001) | Module-level imports | iec104.rs:343,737,1384 | LOW | Logged — batch next iec104.rs touch (D-490) |
| Bare .unwrap() without invariant messages (PAT-002) | `.expect("...")` with description | reassembly/mod.rs:299,318,372,513,620 | LOW | Logged — batch next reassembly touch (D-490) |
| Exec bit inconsistency on bin/test_*.py (PAT-003) | Consistent exec bits across test files | bin/test_compute_input_hash.py (set) vs 5 others (not set) | NIT | Logged — batch next bin-touch (D-490) |
| Manual sys.argv parsing (PAT-004) | argparse like bin/validate-citations | bin/compute-input-hash:309–329 | NIT | Logged — batch next bin-touch (D-490) |

**Carried items (7):** PC-013, PC-015, PC-018, PC-022, SEC-001, HASHMAP-ENTRY-SATURATING-001 (recount: 13 sites, registered 14), PC-NEW-003 — all verified still-present.

---

## Holdout Scenario Freshness

**Source:** maintenance/holdout-freshness-2026-07-21.md
**Binary:** wirerust 0.13.0 (release build). Product delta: IEC-104 feature (STORY-167..174) — 8th supported protocol.

| Metric | Value |
|--------|-------|
| Total scenarios (HS-INDEX v2.13 at sweep open) | 205 |
| Run this sweep | 22 |
| Still valid (PASS) | 18 |
| Stale (intentional change) | 4 (HS-087, HS-123, HS-125, HS-132) |
| FAIL-BUG-SUSPECT | 0 |
| NOT-RUNNABLE (constraint boundary) | 172 |
| Missing coverage (features with 0 scenarios) | 1 (IEC-104 analyzer — highest-value gap) |

**Stale findings repaired (po-holdout-repair, HS-INDEX v2.14):**

| ID | Finding | Resolution |
|----|---------|------------|
| HS-087 (carried from 2026-07-11) | Part C: directory expansion includes .pcapng (pcapng-reader ADR-009) | RESOLVED — HS-087 v1.1 |
| HS-123 | IEC-104 promoted 7→8 supported; partition assertions stale | RESOLVED — HS-123 v1.1 |
| HS-125 | Case E `--supported` JSON array length 7 → 8 | RESOLVED — HS-125 v1.1 |
| HS-132 | jq invariants `supported==7`, `unsupported==23` stale | RESOLVED — HS-132 v1.1 |

**IEC-104 zero-coverage gap (new, highest-value):** The `--iec104` analyzer (TCP/2404; STORY-167..174) has zero holdout scenarios. Flagged for PO backlog. No action in this run per human D-490 scope.

**HS-129 Case C/D (carried from 2026-07-11):** Verification commands omit analyzer flag; product behavior correct. Carried in ROUTE-BC-DEFERRED-2026-07-11 RC-3.

---

## Performance Baseline

**Source:** maintenance/performance-baseline-2026-07-21.md
**Environment: VALID** — near-quiescent (load 0.26/core on 16-core; best conditions in maintenance sweep history). First VALID environment since Jun-22 controlled re-run.

| Benchmark | Previous (Jun-22 anchor µs) | Current (2026-07-21 µs) | Delta | Status |
|-----------|------------------------------|------------------------|-------|--------|
| decode/segmented.pcap | 1.459 | 1.464 | +0.4% | OK |
| decode/tls.pcap | 3.369 | 3.654 | +8.5% | OK |
| decode/dns-remoteshell.pcap | 4.840 | 5.561 | +14.9% | WARN (noise-suspect — 10/13 severe outliers) |
| summary/segmented.pcap | 0.639 | 0.690 | +8.0% | OK |
| summary/dns-remoteshell.pcap | 2.589 | 2.592 | +0.1% | OK |
| reassembly/segmented.pcap | 5.858 | 6.528 | +11.4% | WARN (noise-suspect — 4/5 severe outliers) |
| reassembly/tls.pcap | 24.429 | 23.659 | −3.2% | OK (improvement) |
| **AC-149-003** | **24.445 µs ceiling** | **23.659 µs** | **−3.2%** | **PASS** |

**WARN items:** Both WARN benchmarks are NOISE-SUSPECT (high outlier-to-severe ratios; no hot-path code change in wave delta for these fixtures). Not actionable.

**PERF-RERUN-001:** Primary metric clean (6 severe outliers at threshold, CI width 5.5%). Closure eligibility evidence recorded. Remains OPEN per human D-489 scope decision — literal "load avg < 3.0" criterion not met (4.12 absolute, but 0.26/core on 16-core machine); human decides formal closure.

---

## Spec Coherence

**Source:** maintenance/spec-coherence-findings-2026-07-21.md
**Artifact versions at sweep:** BC-INDEX v2.34 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.57 / STORY-INDEX v3.86.

**Carried findings (7):** SPEC-001 through SPEC-007 — all unchanged; remain in ROUTE-BC-DEFERRED-2026-07-11 or STATE.md drift items.

**New findings:**

| ID | Severity | Finding | Disposition |
|----|----------|---------|-------------|
| SPEC-008 | MINOR | ARCH-INDEX v2.19 SS-19 BC count 27 (should be 28; BC-2.19.028 added) | RESOLVED — spec-steward ARCH-INDEX v2.20 |
| SPEC-009 | MINOR | STORY-INDEX v3.86 epic TOTAL cell 776 (should be 775) | RESOLVED — spec-steward STORY-INDEX v3.87 |
| SPEC-010 | NIT | STORY-158 input-hash stale (stored=ac92b99; CLAUDE.md modified by wave-84 PRs) | RESOLVED — re-baselined Task 3 |
| SPEC-011 | NIT | CLAUDE.md Project References omits ADR-013 | RESOLVED — PR #431 (DOC-014) |

**Input-hash scan:** 124 MATCH, 7 STALE (known churn clusters — STORY-164/165/175..179; structural decision pending with human), 1 STALE new (STORY-158, resolved Task 3). Total 132 stories.

---

## PR Outcomes

| PR | Type | SHA | Gate Evidence |
|----|------|-----|---------------|
| #422 | Dependabot: cargo-deny-action 2.1.1 (D-490, orchestrator-executed) | (squash) | Soak ≥8d; SHA-pin verified; CI green |
| #423 | Dependabot: harden-runner 2.20.0 (D-490, orchestrator-executed) | (squash) | Soak ≥14d; SHA-pin verified; SCORECARD-ENABLEMENT-RUNBOOK satisfied |
| #424 | Dependabot: action-gh-release 3.0.2 (D-490, orchestrator-executed) | (squash) | Soak ≥8d; SHA-pin verified; CI green |
| #425 | Dependabot: codeql-action 4.37.0 (D-490, orchestrator-executed) | (squash) | Soak ≥13d; SHA-pin verified; CI green |
| #431 | docs(maint): IEC-104 doc drift + cli.rs fix (D-490, human-executed post-classifier-halt) | 6c47c0ef | pr-reviewer APPROVE; CI green; classifier halt = same class as PG-MERGE-AUTH-SUBAGENT-CLASSIFIER |

---

## Human Decisions (D-490)

| Decision | Detail |
|---------|--------|
| Dependabot PRs #422-425 ADOPT | All 4 batch-merged as CI-infrastructure; soak + SHA-pin verified |
| Doc fixes DOC-011..015 ADOPT (Route A via PR #431) | Opened, reviewed, merged; adversary self-blocked correctly → human-executed merge |
| Pattern findings PAT-001/002 LOG-ONLY | LOW severity; batch next code-touch of iec104.rs / reassembly/mod.rs |
| Pattern findings PAT-003/004 LOG-ONLY | NIT severity; batch next bin-touch |
| Holdout repairs HS-087/123/125/132 + HS-INDEX ADOPT | po-holdout-repair executed; HS-INDEX v2.14 |
| Spec-steward index fixes SPEC-008/009 ADOPT | ARCH-INDEX v2.20 + STORY-INDEX v3.87 |
| ROUTE-BC-DEFERRED items (RC-1/3/4/5/6) stay deferred | Only RC-2 (HS-087) resolved this run |
| PERF-RERUN-001 stays open | No carry-forward pickup per D-489 scope; primary metric PASS evidence recorded |

---

## Engine Observations

**ADVERSARY-RELAY-UNRELIABLE-001 — Recurrence (9th+):** sweep1-depscan agent wrote artifact but went silent-idle without relaying report. Required synchronous re-dispatch. Lifetime 9+.

**DRIFT-ENGINE-PRMGR-REPORT-001 — Recurrence (4th+):** prmgr silent-idle before Consolidated Gate Report for PR #431; nudge required. Lifetime 4+.

**DRIFT-ENGINE-CHECKOUT-GUARD-001 — EFFECTIVE:** PR #431 adversary Pass 1 correctly self-BLOCKED on wrong tree; re-dispatched with embedded diff. First positive-evidence note.

**PR #431 Classifier Halt:** Subagent classifier correctly refused to approve its own PR per DF-MERGE-AUTH-CLASSIFIER-001. Human executed merge. Same class as PG-MERGE-AUTH-SUBAGENT-CLASSIFIER (D-463, D-485 precedents).

---

## Deferred / Carry-Forward Items

| Item | Target |
|------|--------|
| DEP-SOAK-FOLLOWUP-2026-07-27 | 4 Dependabot PRs NOW MERGED (remove from carry-forward); 15+ crate soak items remain for 2026-07-27 consolidated batch |
| ROUTE-BC-DEFERRED-2026-07-11 RC-1/3/4/5/6 | Next spec-coherence maintenance sweep / PO backlog |
| ROUTE-DOC-DEFERRED-2026-07-21 | Next doc sweep (ADR-0001/0002/0012 stale items) |
| PERF-RERUN-001 | Human re-triage: per-core criterion met; formal closure decision pending |
| IEC-104 holdout zero-coverage | PO backlog — author IEC-104 holdout scenarios |
| SEC-001 TARGET-PASSED | Human re-triage: iec104 feature wave elapsed without pickup |
| ROUTE-W74-DEFERRED TARGET-PASSED | Human re-triage: bin-touch PRs #426/#427 shipped without pickup |
| STORY-INDEX-IN-INPUTS-CHURN | Human decision pending: 7 known stale churn-cluster stories |
| PAT-001/002 | Next code-touch of iec104.rs / reassembly/mod.rs |
| PAT-003/004 | Next bin-touch PR |
| DEP-008/009 | Deferred; resolves when tls-parser/phf_generator migrate to rand 0.9 |

---

## Trend (Last 5 Sweeps)

| Date | Dependencies | Docs | Patterns | Holdouts | Performance |
|------|-------------|------|----------|----------|-------------|
| maint-2026-07-06 | CLEAN (0) | FINDINGS (8, all fixed) | FINDINGS (PF-001=109) | 21/21 PASS | NOISE-SUSPECT |
| maint-2026-07-08 | CLEAN (0) | FINDINGS (5) | FINDINGS (batch) | 21/21 PASS | NOISE-SUSPECT |
| maint-2026-07-09 | CLEAN (0) | FINDINGS (4) | CLEAN | 132/132 PASS | NOISE-SUSPECT |
| maint-2026-07-11 | CLEAN (0) | FINDINGS (8, all fixed) | FINDINGS (3 new, 2 fixed) | 19/20 PASS (1 stale) | INDETERMINATE (load 52.57) |
| **maint-2026-07-21** | **CLEAN (0)** | **FINDINGS (5, all fixed PR #431)** | **FINDINGS (4 new, 2 prior resolved)** | **18/22 PASS (4 stale — all repaired)** | **PASS AC-149-003 (23.659µs, VALID env)** |
