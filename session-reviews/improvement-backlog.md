---
document_type: improvement-backlog
producer: session-reviewer
created: 2026-06-16
---

# Improvement Backlog

Proposals from session reviews that are deferred (not yet approved, filed, or rejected).
Human reviews each entry within 72h of session review completion. Proposals auto-defer
if no response within 72h.

---

## From: feature-arp-v0.7.0 Session Review (2026-06-16)

See full proposals in: `.factory/cycles/feature-arp-v0.7.0/session-review.md`

| ID | Category | Priority | Summary | Status |
|---|---|---|---|---|
| PROP-01 | agent (pr-manager) | P1 CRITICAL | Structural fix for pr-manager shortstop — reposition merge completion as primary success criterion | PENDING HUMAN REVIEW |
| PROP-02 | agent (implementer/test-writer) | P1 HIGH | Inject doc-tense sweep as blocking literal checklist in agent dispatch templates | PENDING HUMAN REVIEW |
| PROP-03 | workflow | P2 HIGH | LOW-finding risk-adjusted adjudication gate (assess fix-induced-regression risk before FIX adjudication) | PENDING HUMAN REVIEW |
| PROP-04 | workflow + agent | P2 HIGH | Proactive post-fixburst consumer-sweep mandate — extend DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 | PENDING HUMAN REVIEW |
| PROP-05 | workflow | P2 HIGH | Pre-PR binary-leak diff check in orchestrator; demo-recorder template fix | PENDING HUMAN REVIEW |
| PROP-06 | agent (adversary) | P2 MEDIUM | BC-completeness-sweep negative/reject-branch axis — enumerate Err paths explicitly | PENDING HUMAN REVIEW |
| PROP-07 | agent + workflow | P2 MEDIUM | Mechanism-first verification before fix spec writing (folds into PROP-03) | PENDING HUMAN REVIEW |
| PROP-08 | agent (implementer) | P3 MEDIUM | Implementer must not change production for contradicting test without BC check | PENDING HUMAN REVIEW |
| PROP-09 | workflow (templates) | P3 MEDIUM | Scope guard in all remediation dispatch templates (git diff --name-only scope) | PENDING HUMAN REVIEW |
| PROP-10 | quality | P3 MEDIUM | Finding-emission ACs must assert on Finding object, not proxy counter | PENDING HUMAN REVIEW |
| PROP-11 | quality (policy) | P4 LOW | Cross-subsystem sibling sweep extension to DF-SIBLING-SWEEP-001 | DEFERRED — append at next policy review |
| PROP-12 | infrastructure | P4 LOW | Session-review infrastructure (this file + pattern-database + benchmarks) | COMPLETE — created 2026-06-16 |

---

## From: v0.9.0 / E-18 grouped-collapse Session Review (2026-06-19)

See full proposals in: `.factory/session-reviews/review-2026-06-19-v0.9.0-e18-grouped-collapse.md`

| ID | Category | Priority | Summary | Status |
|---|---|---|---|---|
| PROP-E18-01 | workflow | P1 HIGH | Add pre-F7 doc/spec-currency consolidated sweep step — CHANGELOG bundle audit, release-config reference-not-snapshot, help-text provenance lint, BC Architecture Anchors, post-delivery notes | PENDING HUMAN REVIEW |
| PROP-E18-02 | agent (adversary dispatch) | P1 HIGH | Orchestrator supplies verified HEAD SHA in adversary dispatch prompt preamble — confirmed mitigation for stale-sandbox git-ref false alarms (3+ occurrences) | PENDING HUMAN REVIEW |
| PROP-E18-03 | workflow (pr-manager F4-close) | P2 HIGH | F4-close bundle CHANGELOG audit — confirm all stories in bundle are represented; block F4->F5 if CHANGELOG incomplete | PENDING HUMAN REVIEW |
| PROP-E18-04 | agent (fixburst discipline) | P2 HIGH | Post-fixburst consuming-surface sweep as blocking numbered checklist in PR description (not just policy reference) — PAT-004 third recurrence | PENDING HUMAN REVIEW |
| PROP-E18-05 | workflow (orchestrator<->pr-manager) | P2 MEDIUM | Document merge-authorization handoff protocol — orchestrator merges directly under human authorization; pr-manager is not an authorization relay | PENDING HUMAN REVIEW |
| PROP-E18-06 | agent (test-writer) | P2 HIGH | BC canonical-test-vector verbatim assertion requirement in test-writer dispatch — cite BC section+version, never derive from running implementation | PENDING HUMAN REVIEW |
| PROP-E18-07 | pattern (database) | P3 LOW | Append PAT-007/008/009 to pattern-database.yaml (already done by session-reviewer) | COMPLETE — applied 2026-06-19 |

---

## Auto-Deferral Note

If no human response within 72h of session review completion (2026-06-16), all
PENDING HUMAN REVIEW entries auto-defer to this backlog with status DEFERRED.
