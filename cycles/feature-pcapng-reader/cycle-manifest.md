---
document_type: cycle-manifest
cycle_id: feature-pcapng-reader
cycle_type: feature
version: pending
status: in-progress
started: 2026-06-19T00:00:00Z
completed: pending
producer: orchestrator
---

# Cycle Manifest: feature-pcapng-reader (Feature)

## Delivered (F1 + F2 complete — F3 next)

| Metric | Value |
|--------|-------|
| Stories delivered | STORY-123..127 (decomposed in F3 — not yet delivered) |
| BCs created | 10 new (BC-2.01.009..018), 1 retired (BC-2.01.004), 2 extended (BC-2.01.001/002) |
| VPs created | VP assignments pending post-F2 propagation |
| Holdout scenarios | HS-001 update required in F3 (cites retired BC-2.01.004) |
| Total cost | TBD |
| Adversarial passes | 0 (F5 pending) |
| Final holdout satisfaction | TBD |
| Release version | pending |

## Spec Changes (F1 + F2)

| Artifact | Change | Before | After |
|----------|--------|--------|-------|
| prd.md | pcapng support added — E-INP-008..011; F2 audit FINDING-003 — §7 RTM updated (BC-2.01.004 struck, BC-2.01.009..018 added) | v1.28 | v1.30 |
| specs/architecture/decisions/ADR-009 | NEW — pcapng capture-format reader; Option A selected (pcap-file 2.0.0, +0 deps) | n/a | v1.0 |
| ARCH-INDEX.md | ADR-009 row added | pre-ADR-009 | updated |
| BC-2.01.009..018 | 10 new BCs for pcapng reader support | n/a | v1.0 each |
| BC-2.01.004 | Retired/inverted — superseded by BC-2.01.009 | active | RETIRED |
| BC-2.01.001/002 | Extended to cover pcapng alongside pcap | v1.x | extended |
| BC-INDEX.md | 10 new rows, 1 retired row | v1.51 | v1.52 |
| error-taxonomy.md | E-INP-008..011 added | v2.2 | v2.3 |
| nfr-catalog.md | pcapng NFR coverage | v2.1 | v2.2 |
| test-vectors.md | pcapng test vectors | v2.1 | v2.2 |
| specs/domain/domain-spec.md | pcapng domain spec | prior | updated |
| specs/domain/capabilities/cap-01-pcap-ingestion.md | pcapng capability | v1.1 | v1.2 |
| specs/module-criticality.md | pcapng module criticality | prior | updated |
| specs/architecture/system-overview.md | pcapng system overview | prior | updated |
| spec-changelog.md | F1+F2 entries added | prior | updated |
| STORY-001.md | Re-anchored to pcapng-aware BCs | v1.5 | v1.6 |
| stories/epics.md | E-INP epic updated for pcapng; F2 audit FINDING-002 — BC-2.11.030..034 added to E-18; total_bcs corrected 297→302 | v1.4 | v1.6 |
| research/pcapng-parser-dependency-eval.md | NEW — library eval; Option A justified | n/a | v1.0 |
| phase-f1-delta-analysis/pcapng-reader-support-delta-analysis.md | NEW — F1 delta analysis | n/a | v1.0 |

## Living Spec Snapshot

F1+F2 artifacts captured in factory-artifacts commit at cycle open.
Retrieve: `git -C .factory log --oneline` to find the F2-complete burst SHA.

## Deprecations

| Artifact | Deprecated By | Replacement | Sunset Date |
|----------|--------------|-------------|-------------|
| BC-2.01.004 | BC-2.01.009 | BC-2.01.009 (pcapng format detection contract) | F3 delivery |

## Open F3 Follow-Ups (MUST NOT BE LOST)

1. BC-2.12.011 (directory glob "*.pcapng excluded") must be revised/retired+replaced when STORY-127 is decomposed.
2. HS-001 holdout scenario + HS-INDEX cite retired BC-2.01.004 — holdout must be updated/added in F3 (PO).
3. Story input-hashes for new STORY-123..127 must be generated at F3 entry (`bin/compute-input-hash --write --scan`).
4. VP assignments for BC-2.01.009..018 to be propagated post-F2 (architect/VP-INDEX).

## Tech Debt Created

| ID | Description | Priority | Source |
|----|-------------|----------|--------|
| FE-001 | pcapng support — moved from CANDIDATE to IN PROGRESS this cycle | P1 | FE-001 |

## F2 Consistency Audit (D-137)

**Audit date:** 2026-06-19  
**Verdict:** CLEAN — all 6 findings fixed and re-audited CLEAN.  
**Report:** `cycles/feature-pcapng-reader/f2-consistency-audit.md`

| Finding | Severity | File | Fix |
|---------|----------|------|-----|
| FINDING-001 | HIGH | ADR-009 Status block | Stale "remains active" replaced with retirement fact |
| FINDING-002 | HIGH | epics.md v1.6 | BC-2.11.030–034 added to E-18; total_bcs 297→302 |
| FINDING-003 | MEDIUM | prd.md v1.30 §7 RTM | BC-2.01.004 struck; BC-2.01.009..018 added |
| FINDING-004 | MEDIUM | BC-2.12.011 v1.4 | Stale Related-BCs rationale annotated; F3/STORY-127 forward-action note added |
| FINDING-005 | LOW | BC-INDEX timestamp | Corrected to 2026-06-19 |
| FINDING-006 | LOW | HS-001 + HS-INDEX | lifecycle_status:stale + banner added; rewrite deferred to F3 |

Active BC total: 302 (BC-INDEX v1.52 ground truth). Pre-pcapng baseline was 293 (not 288 as previously miscounted); +10 BC-2.01.009..018, −1 BC-2.01.004 retired = 302.

## Notes

- Decision D-136: F1+F2 complete. ADR-009 created. Option A (pcap-file 2.0.0, +0 deps) selected. Scope includes E2E corpus expansion (human-approved).
- Decision D-137: F2 consistency audit COMPLETE — 6 findings ALL CLOSED. See audit section above.
- Scope approved by human: pcapng capture-format reader support, FE-001 IN PROGRESS.
- F3 (story decomposition) is next. Story input-hash generation required at F3 entry.
