---
document_type: adversarial-review-pass
story: STORY-090
pass: 1
round: 1
cycle: v0.1.0-greenfield-spec
perimeter: 1 (per-story)
mode: per-story, fresh-context
target: tests/summary_story_090_tests.rs vs src/summary.rs
verdict: FINDINGS_REMAIN
timestamp: 2026-05-31T00:00:00Z
---

# Adversarial Review — STORY-090 Pass 1 (Round 1)

**Story:** STORY-090 — Summary Data Model — ingest, Service Hints, unique_hosts, Serialization
(BC-2.12.018..021; E-9; 5pts; library module `pub mod summary`).
**Mode:** Brownfield-formalization. ZERO src changes.
**Target:** `tests/summary_story_090_tests.rs` — 18 tests (13 AC + 5 EC) against `src/summary.rs`.

## Summary

Pass 1 produced findings in two categories: (1) traceability/anchoring — BC mapping was
permuted (tests pointed to wrong BC IDs) and AC-003/AC-004 test names collided with an existing
test in `tests/summary_tests.rs`; (2) minor spec-fidelity items. No CRITICAL or HIGH behavioral
coverage gaps; test logic was judged strong throughout.

## Findings

| ID | Sev | Category | One-line |
|----|-----|----------|----------|
| ADV-P01-S090-MED-001 | MED | traceability | BC mapping permuted — tests anchor to wrong BC IDs across AC-001..013 |
| ADV-P01-S090-MED-002 | MED | traceability | AC-003 test name (`test_bc_2_12_018_c0_ingest_empty`) collides with summary_tests.rs |
| ADV-P01-S090-MED-003 | MED | traceability | AC-004 test name (`test_bc_2_12_018_c0_ingest_fields`) collides with summary_tests.rs |
| ADV-P01-S090-LOW-001 | LOW | spec-fidelity | Story v1.0 header lists BC-2.12.018..020 (missing .021) |

## Verdict

FINDINGS_REMAIN — remediation required before Pass 2.

**Round-1 remediation scope:** Re-anchor all 18 test functions to canonical BC-2.12.018..021 mapping;
rename AC-003/AC-004 to eliminate cross-suite collision with summary_tests.rs;
update story header BC list.
