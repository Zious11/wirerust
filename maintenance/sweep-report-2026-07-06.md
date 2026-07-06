---
document_type: maintenance-sweep-report
run_id: maint-2026-07-06
date: 2026-07-06
trigger: scheduled
producer: orchestrator
version: v0.11.4
git_head: f7460b4
sweeps_run: [1, 2, 3, 4, 5, 7, 8, 11]
sweeps_skipped: [6, 9, 10]
skip_reason_6: "No DTU required (wirerust is an offline single-binary tool; dtu_required: false)"
skip_reason_9: "No UI component — wirerust is a CLI tool only"
skip_reason_10: "No UI component — wirerust is a CLI tool only"
gate_result: NON-BLOCKING
findings_count: 39
---

# Maintenance Sweep Report: 2026-07-06

Project wirerust v0.11.4 (develop @ f7460b4). 8 applicable sweeps run (1,2,3,4,5,7,8,11); sweeps 6 (DTU), 9 (a11y), 10 (design-drift) N/A. Sweep 8 required 3 attempts (2 API mid-stream stalls, no partial writes; clean on retry).

## Summary

| Sweep | Status | Findings | PRs Opened | Issues Created |
|-------|--------|----------|-----------|----------------|
| Dependency Audit (1) | CLEAN | 2 LOW hygiene | 0 | 0 |
| Documentation Drift (2) | FINDINGS | 8 (1H/4M/3L) | 0 | 0 (gated DF-VALIDATION-001) |
| Pattern Consistency (3) | FINDINGS | 20 (3H/11M/6L) | 0 | 0 (gated DF-VALIDATION-001) |
| Holdout Freshness (4) | FINDINGS | 4 stale | 0 | 0 |
| Performance Baseline (5) | REGRESSION | 1 (+14.0% tls.pcap) | 0 | 0 |
| Spec Coherence (7) | FINDINGS | 3 new MAJOR + 3 carry-forward | 0 | 0 |
| Tech-Debt Register (8) | UPDATED | v1.2→v1.3; 13 new, 1 OVERDUE | 0 | 0 |
| Risk/Assumption (11) | ESCALATE | 2 escalate (ASM-CAND-003/009) | 0 | 0 |

## Overall Health: [HEALTHY / NEEDS_ATTENTION / DEGRADED]

**Assessment: NEEDS_ATTENTION**

No CRITICAL findings. 1 performance regression (+14.0% tls.pcap, strengthens STORY-149). 3 HIGH pattern-consistency gaps. 2 risk-assumption escalations past threshold. All fix routes classified (A–D + manual triage). No GitHub issues filed — all candidates gated on DF-VALIDATION-001 per policy.

---

## Dependency Audit

Sweep 1 (dependency/supply-chain). 0 vulnerabilities across 193 crates. Prior P1 advisories confirmed executed: rand 0.8.6, zerocopy 0.8.52, anyhow 1.0.103.

### New Vulnerabilities

| Dependency | Version | CVE/Advisory | Severity | Fix Available | Action |
|-----------|---------|-------------|----------|--------------|--------|
| — | — | None | — | — | No action required |

### Hygiene Findings (LOW)

| ID | Finding | Action |
|----|---------|--------|
| DEP-006 | Unused license allowlist entries in cargo-deny config | Maintenance backlog |
| DEP-007 | syn v1/v2 duplicate dependency in tree | Maintenance backlog |

Tooling gaps noted: cargo-outdated and semgrep not installed (no gate failure; informational).

NOTE: sweep-1b claim "remove --ignore RUSTSEC-2026-0097 from ci.yml" verified FALSE — ci.yml runs plain `cargo audit`; advisory mentions are historical comments only.

---

## Documentation Drift

Sweep 2 (doc/comment-drift). 8 findings total (1H/4M/3L). 10/14 June sweep findings confirmed fixed.

### Stale Documentation

| Document | Section | Drift Type | Severity | Action |
|----------|---------|-----------|----------|--------|
| README.md | CLI reference | Missing `protocols` subcommand (DOC-009, v0.11.4/E-21) | HIGH | FIX-A (docs PR) |
| ADR-0001, ADR-0002 | Protocol coverage | EtherNet/IP omissions persist | MEDIUM | FIX-A (docs PR) |
| src/lib.rs | Module-level docs | EtherNet/IP omissions persist | MEDIUM | FIX-A (docs PR) |
| README.md | Counters docs | Missing observability counters (DOC-010) | MEDIUM | FIX-A (docs PR) |
| tests/enip_analyzer_tests.rs | 3 test comments | Stale `RED:` comments (×3) | LOW | FIX-A (docs PR) |

---

## Pattern Consistency

Sweep 3 (code-quality/pattern). 20 findings (3H/11M/6L). 12/12 June findings persist. 8 new.

### Inconsistencies Detected

| Pattern | Expected | Found In | Severity | Action |
|---------|----------|----------|----------|--------|
| PC-003: dropped_findings counter | All analyzers emit counter post-v0.11.4 | DNP3 analyzer — sole analyzer missing it | HIGH | FIX-B (code PR); DF-VALIDATION-001 gated |
| PC-001: StreamHandler trait gap | Consistent StreamHandler impl + no per-packet FlowKey clone | DNP3 | HIGH | FIX-B (code PR) |
| PC-002: findings-import style | Consistent use-import style | Multiple files with drift | HIGH | FIX-B (code PR) |
| PC-019/020: DNP3 eviction counters | bindings_evicted/storm_counters_evicted parity | DNP3 eviction paths | MEDIUM | DF-VALIDATION-001 validation first, then FIX-B |
| 11 additional MEDIUM / 6 LOW | Various pattern gaps | Multiple sites | MEDIUM/LOW | Maintenance backlog |

4 GitHub-issue candidates gated on DF-VALIDATION-001: PC-019, PC-020, DOC-009, HOLDOUT-001.

---

## Holdout Scenario Freshness

Sweep 4 (holdout-freshness).

| Metric | Value |
|--------|-------|
| Total scenarios | 131 |
| Active | 127 |
| Stale (broken by code change) | 4 |
| Retired | 0 |
| Missing coverage (features with 0 scenarios) | 0 |

### Stale Scenarios

| Scenario | Broken By | Fix Route |
|----------|-----------|-----------|
| HS-061 | v0.11.4 observability counters | FIX-C (factory artifacts) → product-owner |
| HS-066 | v0.11.4 observability counters | FIX-C (factory artifacts) → product-owner |
| HS-064 | MITRE envelope change (repeat) | FIX-C (factory artifacts) → product-owner |
| HS-075 | MITRE envelope change (repeat) | FIX-C (factory artifacts) → product-owner |

HS-018 missing `lifecycle_status` field (housekeeping). Issue #342 fix invalidated nothing.

---

## Performance Baseline

Sweep 5 (performance). 6/7 benchmarks PASS. 1 REGRESSION.

| Benchmark | Previous (Jun-22) | Current | Delta | Status |
|-----------|----------|---------|-------|--------|
| reassembly/tls.pcap | baseline | +14.0% | +14.0% | REGRESSION (p<0.05, real) |
| 6 other benchmarks | baseline | within threshold | — | OK |

Cause hypothesis: PR #362 flow-purge + PR #365 counters on hot path (~+3.4µs/iter). Regression strengthens STORY-149 scheduling priority. Recommend: escalate STORY-149 to next wave for investigation.

---

## Spec Coherence

Sweep 7 (spec-coherence). 30/33 checks PASS. 0 CRITICAL.

### New MAJOR Findings

| ID | Finding | Action |
|----|---------|--------|
| F-NEW-MAJ-001 | 10 phantom VP-INDEX entries (entries without corresponding VP files) | FIX-D (factory artifacts) → spec-steward/product-owner |
| F-NEW-MAJ-002 | module-criticality.md frozen pre-SS-17/SS-18 (stale component registry) | FIX-D (factory artifacts) → spec-steward/product-owner |
| F-NEW-MAJ-003 | BC-2.16.016 has no corresponding story | FIX-D (factory artifacts) → spec-steward/product-owner |

3 MAJOR findings carry-forward from prior sweep (unresolved).

---

## Tech-Debt Register

Sweep 8 (tech-debt-register). Updated v1.2 → v1.3.

- 13 new items added
- 3 existing items updated
- 1 OVERDUE item: ADV-4 (requires human triage — re-defer or address)
- 4 GitHub-issue candidates gated on DF-VALIDATION-001: PC-019, PC-020, DOC-009, HOLDOUT-001

---

## Risk / Assumption Monitoring

Sweep 11 (risk-assumption). 2 items require escalation.

| ID | Risk/Assumption | Status | Action |
|----|----------------|--------|--------|
| TD-MAINT-RISK-REGISTRY-BACKFILL | No formal ASM/R registry exists | P2→P1 escalated | Human decision required |
| ASM-CAND-003 | Anomaly detection thresholds (hardcoded) | Past 2-release escalation threshold | Human disposition required |
| ASM-CAND-009 | ARP storm rate assumption | Past 2-release escalation threshold | Human disposition required |

3 risks resolved since June sweep.

---

## Fix Route Summary

| Route | Type | Scope | Assignee |
|-------|------|-------|---------|
| FIX-A | Docs PR (develop) | DOC-009 README protocols + DOC-010 counters + EtherNet/IP ADR/lib.rs + 3 stale RED: comments | technical-writer via fix-pr-delivery |
| FIX-B | Code PR (develop) | PC-003 DNP3 dropped_findings counter (+ PC-019/020 after DF-VALIDATION-001) | TDD fix-pr-delivery |
| FIX-C | Factory artifacts | HOLDOUT-001/002 stale scenario remediation (HS-061/064/066/075) | product-owner |
| FIX-D | Factory artifacts | F-NEW-MAJ-001/002/003 spec hygiene | spec-steward/product-owner |
| MANUAL | Human decision | PERF +14.0% (STORY-149 escalation), ADV-4 OVERDUE, TD-MAINT-RISK-REGISTRY-BACKFILL (P1), ASM-CAND-003/009 | Human triage required |

CRITICAL findings: NONE (no blocking notification required).

---

## Trend (Last 5 Sweeps)

| Date | Dependencies | Docs | Patterns | Holdouts | Performance |
|------|-------------|------|----------|----------|-------------|
| 2026-05-19 | CLEAN | 3 findings | 8 findings | 95% | baseline |
| 2026-06-17 | CLEAN | 6 findings | 14 findings | 96% | +1.2% |
| 2026-06-22 | 0 vulns | 14 findings | 12 findings (carry) | 96.2% | +2.1% |
| 2026-07-01 | 0 vulns | 4 findings | 12 findings (carry) | 96.8% | +2.1% |
| 2026-07-06 | 0 vulns (193) | 8 findings | 20 findings (3H new) | 127/131 (96.9%) | +14.0% REGRESSION |
