---
document_type: maintenance-sweep-summary
sweep: tech-debt-register
run_id: maint-2026-07-06
date: 2026-07-06
register_version: "1.3"
producer: maintenance-orchestrator
branch: develop
commit: f7460b4
version: v0.11.4
---

# Tech Debt Register — Maintenance Sweep 8 Summary (maint-2026-07-06)

## Register Version Bump

| Field | Before | After |
|-------|--------|-------|
| version | 1.2 | 1.3 |
| timestamp | 2026-06-29 | 2026-07-06 |

## Item Counts

| Category | Count |
|----------|-------|
| New items added | 13 |
| Existing items updated | 3 |
| Overdue items flagged for human triage | 1 |
| Resolved/closed this sweep | 0 |

## New Items Added

| ID | Source Sweep | Priority | Severity |
|----|-------------|----------|----------|
| TD-MAINT-THRESHOLD-CALIB-001 | risk-assumption-monitoring | P1 | HIGH — 3 unvalidated thresholds past escalation threshold |
| PC-016 | pattern-consistency | P3 | MEDIUM — DNP3 master_addrs_seen cap, no counter; T1692.001 masking REFUTED, reframed as observability parity; IN-PROGRESS FIX-B |
| PC-017 | pattern-consistency | P2 | MEDIUM — DNP3 pending_requests LRU eviction, no counter; T1691.001 degradation CONFIRMED; IN-PROGRESS FIX-B |
| PC-018 | pattern-consistency | P3 | LOW — ENIP+Modbus HashMap non-deterministic distribution keys (canonical PC-018/019) |
| PC-020 | pattern-consistency | P2 | MEDIUM — ENIP missing StreamHandler/StreamAnalyzer trait (canonical PC-020) |
| PC-022 | pattern-consistency | P3 | MEDIUM — ENIP import-style drift batch (canonical PC-013/014/015) |
| DOC-009 | doc-drift | P2 | HIGH — README `protocols` subcommand entirely absent |
| DOC-010 | doc-drift | P3 | MEDIUM — observability counters undocumented + 6 minor doc items |
| HOLDOUT-001 | holdout-freshness | P2 | MEDIUM — 4 stale scenarios (HS-061/064/066/075) |
| HOLDOUT-002 | holdout-freshness | P3 | LOW — HS-018 missing lifecycle_status frontmatter |
| PERF-002 | performance | P3 | REGRESSION — reassembly/tls.pcap +14.0% vs Jun-22 baseline confirmed |
| DEP-006 | dependency-audit | P3 | LOW — deny.toml 8 unused license allowlist entries |
| DEP-007 | dependency-audit | P3 | LOW/INFO — syn v1/v2 build-time duplicate (upstream resolution) |

## Existing Items Updated

| ID | Change |
|----|--------|
| TD-MAINT-RISK-REGISTRY-BACKFILL | Priority P2 → P1; 11 ASM-CAND + 12 R-CAND still untracked; 2 new assumptions (ASM-010/011) since last sweep |
| TD-MAINT-PERF-ARP-HOTPATH | Added note: tls.pcap +14.0% confirmed regression (maint-2026-07-06); cross-ref PERF-002 + STORY-149 |
| ADV-4 | Marked OVERDUE — target was "next maintenance sweep" (maint-2026-07-06); still unaddressed. HUMAN TRIAGE REQUIRED. |

## Overdue Items (Past Target Release) — Human Triage Required

| ID | Original Target | Disposition Needed |
|----|----------------|--------------------|
| ADV-4 | "next maintenance sweep" after maint-2026-06-22 | Address ci.yml build-dep-chain comment or explicitly re-defer with new target |

## GitHub Issue Candidates (DF-VALIDATION-001 gated — NOT filed)

| ID | Rationale |
|----|-----------|
| PC-016 | Detection correctness: DNP3 master_addrs_seen overflow may silence T1692.001 |
| PC-017 | Detection correctness: DNP3 pending_requests eviction may degrade T1691.001 |
| DOC-009 | HIGH severity user-facing doc gap: `protocols` subcommand undiscoverable |
| HOLDOUT-001 | Product-owner BC-shape validation needed for 4 stale scenarios |

All four require research-agent validation per DF-VALIDATION-001 before filing as GitHub issues.

## Source Sweep Coverage

| Sweep | Report | New Register Items |
|-------|--------|--------------------|
| Sweep 1 (dependency-audit) | dependency-audit.md — CLEAN, 2 LOW | DEP-006, DEP-007 |
| Sweep 3 (pattern-consistency) | pattern-consistency.md — 20 findings (8 new) | PC-016, PC-019, PC-020, PC-021, PC-023 |
| Sweep 4 (holdout-freshness) | holdout-freshness.md — 4 stale, 1 gap | HOLDOUT-001, HOLDOUT-002 |
| Sweep 5 (performance) | performance.md — 1 confirmed REGRESSION | PERF-002 |
| Sweep 8 (doc-drift) | doc-drift.md — 8 findings (1 HIGH, 4 MED, 3 LOW) | DOC-009, DOC-010 |
| Sweep 11 (risk-assumption-monitoring) | risk-assumption-monitoring.md — 2 ESCALATE-CRITICAL | TD-MAINT-THRESHOLD-CALIB-001; TD-MAINT-RISK-REGISTRY-BACKFILL P1 |
