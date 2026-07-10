# Tech Debt Check — maint-2026-07-09

**Run:** maint-2026-07-09, Sweep 8 (tech-debt-register), DF-030
**Date:** 2026-07-09
**Register version:** 1.7 → last_updated 2026-07-09
**develop HEAD at check:** 716054a (14 unreleased commits ahead of v0.11.5)

Sources scanned:

- `.factory/tech-debt-register.md` (primary target)
- `.factory/STATE.md` Open Items / Backlog section
- `.factory/maintenance/sweep-report-2026-07-08.md` (prior sweep verdicts + DF-VALIDATION-001 triage table)
- `.factory/maintenance/backlog-triage-maint-2026-07-08.md` (11-item DF-VALIDATION-001 triage)
- `.factory/maintenance/issue-backlog-triage-2026-07-08.md` (10 GitHub issues)
- `CLAUDE.md` (deferred infrastructure items)

---

## Rows Added (4)

| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| DNP3-CLOSEDFLOW-REOPEN-REUSE-001 | DNP3 `closed_flow_direct_operates` Vec double-lists same FlowKey under NAT port reuse. Spec-conformant per BC-2.15.021 PC-4. Optional docstring rename only. Source: backlog-triage-maint-2026-07-08 §6. | P3 | DEFERRED |
| TD-W7.1-PUBLIC-API-BASELINE | `cargo public-api` two-step setup (CLAUDE.md drift item W7.1). Deferred from W11/W16 per no-flaky-stub policy. Companion to TD-E18-SEMVER-CHECKS-001. | P3 | DEFERRED |
| TD-INPUT-HASH-CI-GATE | `bin/compute-input-hash --scan` not wired into develop CI (factory-artifacts branch). Manual gate in place at Phase-4 entry. CLAUDE.md "CI Gate Decision (deferred)". | P3 | DEFERRED |
| TD-DTOLNAY-PIN-EXEMPTION | `dtolnay/rust-toolchain@stable`/`@nightly` allowlisted in action-pin-gate. CLAUDE.md states "tracked for separate resolution." Resolution approach (formal accept vs alternative) undecided. | P3 | OPEN |

## Rows Updated (3)

| ID | Change |
|----|--------|
| Debt Items section | Added maint-2026-07-09 Sweep 8 summary note. |
| Tech Debt as Feature Mode Cycles | Removed RESOLVED TD-MAINT-THRESHOLD-CALIB-001 from P1 candidate list (resolved PR #382, 2026-07-08). |
| `last_updated` frontmatter | 2026-07-08T00:00:00Z → 2026-07-09T00:00:00Z |

## Rows Resolved This Sweep

None — Sweep 8 is register-reconciliation only; no new PRs in this sweep.

---

## Items Reconciled (No New Row Needed)

| Item | Reason |
|------|--------|
| HS-INDEX-ENIP-WAVE-DRIFT-001 | Already subsumed in existing ROUTE-C-DEFERRED row (human deferred 2026-07-08). |
| EPICS-TOTAL-BCS-DRIFT-001 | Already subsumed in existing ROUTE-C-DEFERRED row (human deferred 2026-07-08). |
| DEP-006 / DEP-007 | Pre-existing rows in register; status DEFERRED unchanged; no update needed. |
| wave-72 S-7.02 PG-W72-LMR003-TEMPLATE-CONFORMANCE + PG-W72-CGDT-MAIN-GUARDS | STORY-162 draft confirmed (STATE.md STORY-INDEX v3.32); story covers both items; no register row needed. |

---

## Overdue / At-Risk Items — Human Triage

| ID | Priority | Status | Concern |
|----|----------|--------|---------|
| TD-MAINT-RISK-REGISTRY-BACKFILL | P1 | DEFERRED (promoted P1 maint-2026-07-06) | No target date set; must complete before next ICS protocol feature cycle. Not yet overdue (no next cycle announced) but should surface to human before next wave planning. |
| SEC-W71-001 | P3 | VALIDATED-PENDING-FILING (CWE-22) | Human deferred GitHub issue filing 2026-07-08 (1 day ago). No target date set. Not overdue. Recommend: file before any feature cycle that touches `bin/compute-input-hash`. |
| TD-DTOLNAY-PIN-EXEMPTION | P3 | OPEN | Resolution approach undecided; no target date. Mention at next CI/supply-chain review. |

No items are currently overdue (past a stated target release/date). No SURFACE-FOR-HUMAN-TRIAGE or WARNING flags raised.

---

## Summary

| Metric | Value |
|--------|-------|
| Rows added | 4 (all P3) |
| Rows updated | 3 |
| Rows resolved | 0 |
| P1 open items | 1 (TD-MAINT-RISK-REGISTRY-BACKFILL) |
| Overdue items | 0 |
| Items needing immediate human decision | 0 |
