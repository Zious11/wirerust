---
document_type: f5-adversarial-round-review
producer: adversary-via-orchestrator
date: 2026-07-17
cycle: feature-iec104
round: 2
reviewed_sha: 9c5aa9a
base: fedcea4
---

# F5 Round 2 — feature-iec104

Attestation: develop @ 9c5aa9a; grep-counts matched (source_ip:None=0; fn test_fix_f5_001=10).

## Part A — Round 1 Finding Verification (F-01..F-05)

All five Round-1 findings verified remediated by FIX-F5-001 (PR #411 9c5aa9a):

- **F-01** [HIGH] BC-2.19.011 PC-3 SATISFIED: T0881 STOPDT finding now carries source_ip + timestamp; 10
  red-first tests (mod fix_f5_001) each assert source_ip + timestamp per finding family. DNP3-parity-exact.
- **F-02** [MEDIUM] All 10 IEC-104 emit sites enriched with source_ip + timestamp (8 function + 2 inline);
  `let _ = ts` pattern eliminated. DNP3/ENIP house-parity achieved.
- **F-03** [MEDIUM] All 9 stale RED-phase prose sites scrubbed GREEN (4 originally listed + 4 unlisted
  siblings + 1 additional site found during fix sweep).
- **F-04** [MEDIUM] False forward-reference "enriched in STORY-173" comment at iec104.rs:1029 removed.
- **F-05** [MEDIUM] protocols_tests.rs:208 count comment corrected to match actual assert counts.

## Part B — New Findings

**F5R2-01 [MEDIUM][doc-accuracy]** Demo-evidence provenance in the FIX-F5-001 PR/report cites STORY-172 and
STORY-173 (IEC-104's own implementation stories) as the source lineage for the Before/After T0881 JSON
comparison. The actual lineage establishing the DNP3/ENIP parity baseline is S-139 (DNP3) and S-140 (ENIP).
Wrong provenance in evidence citations misleads future reviewers about what the "before" state actually was.

**F5R2-02 [MEDIUM][doc-accuracy]** Before/After T0881 demo JSON in the FIX-F5-001 evidence artifacts is
fabricated: the "before" snippet shows `"category": "anomaly"` and `"severity": "high"` — enum variants that
do not exist in the Rust codebase; real serialized output uses `"impact"` / `"medium"` / `"possible"`.
The "after" snippet is correct. Non-existent enum variant names in demo JSON constitute fabricated evidence.

**F5R2-03 [LOW][doc-accuracy]** Year in several evidence file timestamps shows 2025 rather than 2026.

**CHANGELOG direction-parity [LOW-pending]** FIX-P4-001 CHANGELOG entry claims direction:Some(...) was
the DNP3/ENIP direction, suggesting IEC-104 now matches that baseline. DNP3 and ENIP both set `None` for
direction in the pre-fix baseline; the claim implies a set direction exists when it did not. IEC-104 actually
exceeds the baseline by providing direction. Wording needs precision.

## Code Verdict

**CONVERGED — 0 code findings.**

- `direction` → `source_ip` DNP3-parity-exact across all 10 emit sites.
- Tests non-vacuous: 10 fix_f5_001 tests each assert source_ip + timestamp on real finding objects.
- 105 signature-change callers pass None and assert on other fields — no regressions.

Residual findings are docs-accuracy only. Routed to FIX-F5-002.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (4/4) |
| **Median severity** | 2.0 (2 MEDIUM + 2 LOW) |
| **Trajectory** | 5 → 4 (docs-only residual; 0 code) |
| **Verdict** | FINDINGS_REMAIN |

## Classification

FINDINGS (docs-only). All residual items routed to FIX-F5-002. Feature code and tests CONVERGED since Round 2.
