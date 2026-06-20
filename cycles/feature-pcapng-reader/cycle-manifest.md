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
| prd.md | pcapng support added — E-INP-008..011; F2 audit FINDING-003 — §7 RTM updated; F2 completeness — v1.31 (§-level completeness note) | v1.28 | v1.31 |
| specs/architecture/decisions/ADR-009 | NEW — pcapng capture-format reader; Option A selected (pcap-file 2.0.0, +0 deps); rev 2 adds Decision 7 (multi-section reject) + obsolete-Packet-Block note; rev 3 corrects Decision 7 rationale (reject = scope decision; pcap-file resets correctly, source-verified; F-06 SUPERSEDED; mergecap -F pcapng aligned) | n/a | rev 3 |
| ARCH-INDEX.md | ADR-009 row added | pre-ADR-009 | updated |
| BC-2.01.009..018 | 10 new BCs for pcapng reader support | n/a | v1.0 each |
| BC-2.01.004 | Retired/inverted — superseded by BC-2.01.009 | active | RETIRED |
| BC-2.01.001/002 | Extended to cover pcapng alongside pcap | v1.x | extended |
| BC-INDEX.md | 10 new rows, 1 retired row | v1.51 | v1.52 |
| error-taxonomy.md | E-INP-008..011 added (F2 spec); E-INP-012 added + E-INP-011 refined (F2 completeness) | v2.2 | v2.4 |
| nfr-catalog.md | pcapng NFR coverage | v2.1 | v2.2 |
| test-vectors.md | pcapng test vectors | v2.1 | v2.2 |
| specs/domain/domain-spec.md | pcapng domain spec | prior | updated |
| specs/domain/capabilities/cap-01-pcap-ingestion.md | pcapng capability | v1.1 | v1.2 |
| specs/module-criticality.md | pcapng module criticality | prior | updated |
| specs/architecture/system-overview.md | pcapng system overview | prior | updated |
| spec-changelog.md | F1+F2 entries added | prior | updated |
| STORY-001.md | Re-anchored to pcapng-aware BCs | v1.5 | v1.6 |
| stories/epics.md | E-INP epic updated for pcapng; F2 audit FINDING-002 — BC-2.11.030..034 added to E-18; total_bcs corrected 297→302 | v1.4 | v1.6 |
| BC-2.01.010 | AC added: multi-section reject + E-INP-012 reference (F2 completeness F-06) | v1.0 | v1.1 |
| BC-2.01.015 | Explicit skip-arm enumeration (NRB/ISB/DSB/SystemdJournal/obsolete-Packet/Unknown) | v1.0 | v1.1 |
| BC-2.01.017 | Trace updated (F2 completeness) | v1.0 | v1.1 |
| BC-2.01.018 | Actionable E-INP-011 message (tcpdump -i any hint) + directory-mode isolation AC (xref BC-2.12.011) | v1.0 | v1.1 |
| research/pcapng-parser-dependency-eval.md | NEW — library eval; Option A justified | n/a | v1.0 |
| research/pcapng-spec-completeness-validation.md | NEW — F2 completeness validation report | n/a | v1.0 |
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

### Completeness-Derived F3 Follow-Ups (from D-138)

5. **F-06 (STORY-123 / SHB parsing):** Implement multi-section reject + craft a 2-section pcapng test fixture. The multi-section reset behavior of pcap-file 2.0.0 is INCONCLUSIVE — verify at implementation.
6. **F-05 (STORY-125 / timestamp):** BC-2.01.014 Kani proof MUST cover the full u8 if_tsresol space (base-2 branch has no corpus coverage) — enforce in the timestamp story.
7. **F-07 (all skip-block stories):** Implementer must provide explicit match arms for every enumerated skip block (NRB/ISB/DSB/SystemdJournal/obsolete-Packet/Unknown) — no todo!()/wildcard that silently drops.

## Tech Debt Created

| ID | Description | Priority | Source |
|----|-------------|----------|--------|
| FE-001 | pcapng support — moved from CANDIDATE to IN PROGRESS this cycle | P1 | FE-001 |

## F2 Completeness Validation (D-138)

**Validation date:** 2026-06-19  
**Validator:** research-agent (pcapng spec + intended unlock corpus)  
**Verdict:** COMPLETE for intended corpus.  
**Report:** `research/pcapng-spec-completeness-validation.md`

| Finding | Severity | Application |
|---------|----------|-------------|
| Confirmed-OK: DSB-for-TLS | n/a | Safely out of scope; skip arm covered |
| Confirmed-OK: power-of-2 if_tsresol | n/a | Already covered by BC-2.01.011/BC-2.01.014 |
| F-06: single-section-only policy | MEDIUM | E-INP-012 added; AC to BC-2.01.010 v1.1; ADR-009 Decision 7 (rev 2). **SUPERSEDED (D-139):** pcap-file 2.0.0 resets correctly; reject retained as scope decision; rationale corrected in ADR-009 rev 3 / BC-2.01.010 v1.2 / error-taxonomy v2.5. |
| F-07: skip-arm enumeration | LOW | BC-2.01.015 v1.1 enumerates all skip-arms |
| F-08: obsolete Packet Block (0x2) | LOW | ADR-009 rev 2 records it as skip-not-read |
| F-11: E-INP-011 actionability | LOW | BC-2.01.018 v1.1: tcpdump hint + directory-mode isolation AC |

**AC delta artifacts (no new BCs; 302 active BCs unchanged):**
- BC-2.01.010 → v1.2 (v1.1: multi-section reject AC + E-INP-012; v1.2: rationale corrected — scope decision, library-reset ack, mergecap -F pcapng)
- BC-2.01.015 → v1.1 (explicit skip-arm enumeration)
- BC-2.01.017 → v1.1 (trace updated)
- BC-2.01.018 → v1.1 (actionable E-INP-011 + directory isolation AC)
- error-taxonomy.md → v2.5 (v2.4: E-INP-012 added; v2.5: E-INP-012 Notes corrected + mergecap -F pcapng aligned)
- ADR-009 → rev 3 (rev 2: Decision 7 + obsolete-Packet-Block; rev 3: rationale corrected; F-06 SUPERSEDED; mergecap -F pcapng aligned)
- prd.md → v1.31

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
- Decision D-138: F2 completeness validation (research-agent) COMPLETE for intended corpus. F-06/F-07/F-08/F-11 applied as AC deltas. 302 active BCs unchanged. Re-audited CLEAN. F2 now spec-complete + consistency-verified + completeness-validated.
- Decision D-139: Multi-section pcapng question RESOLVED (source-level research — pcapng-multisection-decision.md). pcap-file 2.0.0 resets correctly. REJECT retained as scope decision. ADR-009 rev 3, BC-2.01.010 v1.2, error-taxonomy v2.5. F-06 SUPERSEDED. Re-audited CLEAN (f2-consistency-audit.md rationale-correction pass; 2 LOW cosmetics RC-1/RC-2 closed). F2 fully spec-complete, consistency- and completeness-validated. F3 NEXT.
- Scope approved by human: pcapng capture-format reader support, FE-001 IN PROGRESS.
- F3 (story decomposition) is next. Story input-hash generation required at F3 entry.
