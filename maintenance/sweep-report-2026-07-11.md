---
document_type: maintenance-sweep-report
run_id: maint-2026-07-11
date: 2026-07-11
trigger: scheduled
base_commit: b5e1e15
version: v0.12.0
sweeps_run: [1, 2, 3, 4, 5, 7, 8]
sweeps_skipped: [6, 9]
skip_reasons:
  sweep_6_dtu: "dtu_required: false — no DTU clones to validate"
  sweep_9_a11y: "CLI-only project — no UI surface for accessibility audit"
fix_route_a: "PR #396 squash-merged 6779be6 (2026-07-11T16:45:25Z)"
fix_route_b: "DEFERRED (human, 2026-07-11)"
fix_route_c: "DEFERRED (human, 2026-07-11)"
story_164_amended: "v1.1, AC-164-005, 4 pts, W72-L2 codified"
---

# Maintenance Sweep Report — maint-2026-07-11

**Run ID:** maint-2026-07-11
**Date:** 2026-07-11
**Prior run:** maint-2026-07-09
**develop HEAD at open:** b5e1e15 (v0.12.0, 1 unreleased commit)
**develop HEAD at close:** 6779be6 (v0.12.0, 2 unreleased commits — b5e1e15 + PR #396 6779be6)
**Sweeps executed:** 1 (deps), 2 (doc-drift), 3 (patterns), 4 (holdouts), 5 (perf), 7 (spec-coherence), 8 (P1 review)
**Sweeps skipped:** Sweep 6 (DTU — dtu_required: false), Sweep 9 (a11y/design — CLI product, no UI)

---

## Summary

| Sweep | Status | Findings | PRs Opened | Issues Created |
|-------|--------|----------|-----------|----------------|
| 1 — Dependency Audit | CLEAN | 0 new (1 pre-existing LOW DEP-007) | 0 | 0 |
| 2 — Documentation Drift | FINDINGS | 8 | 1 (Route A) | 0 |
| 3 — Pattern Consistency | FINDINGS | 3 new NIT/LOW + register corrections | 1 (Route A) | 0 |
| 4 — Holdout Freshness | FINDINGS | 1 FAIL-STALE + 2 scenario-maintenance | 0 | 0 |
| 5 — Performance Baseline | INDETERMINATE | PERF-RERUN-001 open | 0 | 0 |
| 7 — Spec Coherence | FINDINGS | 4 new MINOR/NIT | 0 | 0 |
| 8 — P1 Open Items | CLEAN | 0 P1 items | 0 | 0 |

## Overall Health: [HEALTHY / NEEDS_ATTENTION / DEGRADED]

**Status: NEEDS_ATTENTION**

0 CRITICAL/HIGH findings. Route A delivered (PR #396, 17 findings fixed, adversary 3/3
CONVERGED). Routes B/C deferred by human. PERF-RERUN-001 open (machine load conditions).
Spec-index drift items (Routes B/C) accumulate but are low-severity.

---

## Dependency Audit

### New Vulnerabilities

| Dependency | Version | CVE/Advisory | Severity | Fix Available | Action |
|-----------|---------|-------------|----------|--------------|--------|
| (none) | — | — | — | — | No action required |

**Verdict: CLEAN.** `cargo audit` 0 advisories against 193 locked crates. Advisory DB: 1159
entries (stable vs maint-2026-07-08, +0 new entries). `cargo deny` 0 errors, 1 pre-existing
warning (DEP-007 syn 1.0.109/2.0.117 duplicate, deferred).

**Dependency maintenance cadence (research-agent online check, 2026-07-11):**

| Crate | Locked | Latest release | Last commit | Verdict |
|-------|--------|----------------|-------------|---------|
| pcap-file (direct) | 2.0.0 | 3.0.0-rc.2 (2026-05-06) | 2026-05-08 | ACTIVE |
| tls-parser (direct) | 0.12.2 | 0.12.2 (2024-09-09, ~22 mo) | 2025-08-13 | SLOW-BUT-MAINTAINED |
| nom-derive (transitive) | 0.10.1 | 0.10.1 (2023-03-20, ~28 mo) | 2025-07-29 | STALE (not abandoned) |

tls-parser: recheck ~2026-Q4 (22 months since release, repo active). nom-derive: STALE-transitive
(28 months, low risk — transitive proc-macro only; no RUSTSEC advisory for any of the three).

---

## Documentation Drift

### Stale Documentation

| Document | Section | Drift Type | Severity | Action |
|----------|---------|-----------|----------|--------|
| README.md:263 | ARP JSON output | Incorrect nested key (`arp_summary` doesn't exist; counters are flat in `analyzers[i].detail`) | MEDIUM | PR #396 FIXED |
| docs/adr/0002:180 | ENIP deviation | Wrong tech-debt ID ("PC-023" should be "PC-020") | LOW | PR #396 FIXED |
| docs/adr/0001:28-40 | StreamDispatcher struct snippet | Missing `unclassified_port_counts` + `coverage_gaps_enabled` fields (STORY-153) | LOW | PR #396 FIXED |
| CHANGELOG.md:806 | v0.7.0 D3 ARP storm entry | Claims "Attributed to T0830"; code emits `mitre_techniques: []` | LOW | PR #396 FIXED |
| src/analyzer/arp.rs:1006 | `detect_storm` doc-comment | Integer division not noted in rate formula | LOW | PR #396 FIXED |
| README.md:408-412 | DNP3 tuning guidance | No bidirectional flow assumption stated | LOW | PR #396 FIXED |
| src/cli.rs:185,192 | Modbus CLI arg doc-comments | Unit format inconsistency: "1-second" vs ">= 2s" | LOW | PR #396 FIXED |
| README.md:117 | `--arp-storm-rate` description | Missing ">= semantics" and calibration note | LOW | PR #396 FIXED |

All 8 findings resolved by PR #396 (6779be6). Also confirmed: ROUTE-B-DEFERRED NEW-002
(README --coverage-gaps missing) was RESOLVED by commit e3ca2bc (PR #393) in maint-2026-07-09.

---

## Pattern Consistency

### Inconsistencies Detected

| Pattern | Expected | Found In | Severity | Action |
|---------|----------|----------|----------|--------|
| `#[allow(unused)]` on used pub consts | Remove on production-used items | dnp3.rs:122,129,139,145,150,158,167,173,180 (9 sites) | NIT | PR #396 FIXED |
| `#[allow(clippy::too_many_arguments)]` rationale | `// N params: <reason>` comment required | dnp3.rs:994,1079,1409,1475,1553,1637 (6 sites) | NIT | PR #396 FIXED |
| Import-style drift (Modbus/DNP3/ARP) | Module-level imports as in http.rs/tls.rs | modbus.rs (28 inline), dnp3.rs (34 inline), arp.rs (10+ function-body) | LOW | B — DEFERRED |

**Register corrections applied in PR #396:** PC-014 RESOLVED (code already fixed, key renamed);
PC-013 line numbers updated (555/576/642/827 → 575/596/669/864); HASHMAP-ENTRY-SATURATING-001
exact count corrected to 14 (prior: ~15; modbus.rs:873 two-liner variant was missed by
one-liner grep).

PF-001 discipline holds: no new plain `+=` violations on diagnostic counters since c4eb1f4.

---

## Holdout Scenario Freshness

| Metric | Value |
|--------|-------|
| Total scenarios | 205 (HS-INDEX v2.13) |
| Run this sweep | 20 |
| PASS | 19 |
| FAIL-STALE | 1 (HS-087 Part C) |
| FAIL-BUG-SUSPECT | 0 |
| Missing coverage (features with 0 scenarios) | 1 (Modbus — highest-value gap) |

**HS-087 Part C (FAIL-STALE):** Directory expansion now includes `.pcapng` and accepts uppercase
`.PCAP`. v0.1.0 expectation predates pcapng-reader feature (ADR-009). Product correct; scenario
stale. → Route B DEFERRED.

**HS-129 Case C/D (scenario under-spec):** Verification commands omit required analyzer flag
for dual-gate. Product behavior correct; scenario needs `--http`/`--all` added. → Route B
DEFERRED.

**Modbus — zero holdout coverage:** No scenario exercises port-502 dispatch or T0806/T1692.001
detection. → Route B DEFERRED (author Modbus holdout scenarios).

---

## Performance Baseline

| Benchmark | Previous (Jun-22 anchor) | Current (2026-07-11) | Delta | Status |
|-----------|--------------------------|----------------------|-------|--------|
| reassembly/tls.pcap | 24.429 µs | 40.388 µs | +65.3% | CRITICAL (NOISE-SUSPECT) |
| decode/segmented.pcap | 1.459 µs | 3.331 µs | +128.3% | CRITICAL (NOISE-SUSPECT) |
| decode/tls.pcap | 3.369 µs | 8.234 µs | +144.4% | CRITICAL (NOISE-SUSPECT) |
| decode/dns-remoteshell.pcap | 4.840 µs | 9.637 µs | +99.1% | CRITICAL (NOISE-SUSPECT) |
| summary/segmented.pcap | 0.639 µs | 1.115 µs | +74.5% | CRITICAL (NOISE-SUSPECT) |
| reassembly/segmented.pcap | 5.858 µs | 9.013 µs | +53.9% | CRITICAL (NOISE-SUSPECT) |

All CRITICAL flags are NOISE-SUSPECT. Machine load avg 52.57/46.80/35.41 at run time
(OrbStack ~480% CPU, 8+ node processes). Wave delta b642c0f..b5e1e15 is cold-path only.
AC-149-003 status: INDETERMINATE. Jun-22 controlled re-run (24.429 µs) remains authoritative.
PERF-RERUN-001 remains OPEN — human deferred; quiescent conditions required before re-run.

---

## Fix Route A — PR #396 (6779be6, 2026-07-11T16:45:25Z)

**17 findings fixed.** Merged BY HUMAN directly (required-review rule unsatisfiable
single-account; admin-bypass declined by orchestrator classifier per
PG-MERGE-AUTH-SUBAGENT-CLASSIFIER). CI: 12/12 green. Security: CLEAN.

**Adversary convergence: 3/3 CONVERGED.**

Trajectory: `VOID → F-P1-001 → F-P1r-002 → 1/3 → F-P2-001 → 1/3 → 2/3 → 3/3`

Pass 1 (VOID): checkout-guard failure — adversary reviewed develop instead of PR branch
(DRIFT-ENGINE-CHECKOUT-GUARD-001 recurrence). Re-dispatched. Pass 1 real: F-P1-001 (finding),
remediation produced F-P1r-002. Clean 1/3. Pass 2: F-P2-001, remediated. Clean 2/3 → 3/3.

---

## STORY-164 Amendment

STORY-164 amended to v1.1 by story-writer (maint-2026-07-11):
- **AC-164-005 added** (PG-W72-BREAKING-HOLDOUT-SWEEP): BREAKING-change holdout-expectation
  sweep obligation; wave-72 Lesson-2 (PROP-V0.12.0-01) codified; creates
  `breaking-change-delivery-protocol.md` + CLAUDE.md reference row.
- **Points:** 3→4. **STORY-INDEX:** v3.43→v3.44 (total_points 722→723).
- W72-L2 codified.

---

## Trend (Last 5 Sweeps)

| Date | Dependencies | Docs | Patterns | Holdouts | Performance |
|------|-------------|------|----------|----------|-------------|
| maint-2026-06-22 | CLEAN (0) | FINDINGS (8) | FINDINGS (batch) | PASS | NOISE-SUSPECT (thermal) |
| maint-2026-07-06 | CLEAN (0) | FINDINGS (8) | FINDINGS (PF-001=109) | 21/21 PASS | NOISE-SUSPECT |
| maint-2026-07-08 | CLEAN (0) | FINDINGS (5) | FINDINGS (PF-001 resolved) | 21/21 PASS | NOISE-SUSPECT |
| maint-2026-07-09 | CLEAN (0) | FINDINGS (4) | CLEAN | 132/132 PASS | NOISE-SUSPECT |
| **maint-2026-07-11** | **CLEAN (0)** | **FINDINGS (8, all fixed PR #396)** | **FINDINGS (3 new, fixed)** | **19/20 PASS** | **INDETERMINATE** |
