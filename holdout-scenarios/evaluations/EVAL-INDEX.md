---
document_type: eval-index
level: ops
version: "1.0"
status: complete
producer: holdout-evaluator
timestamp: 2026-06-08T00:00:00Z
cycle: v0.1.0-greenfield-spec
traces_to:
  - .factory/holdout-scenarios/HS-INDEX.md
  - .factory/cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md
---

# Holdout Evaluation Index — v0.1.0-greenfield-spec

> Authoritative index of all Phase-4 holdout evaluation artifacts for the
> v0.1.0-greenfield-spec cycle. Files were originally produced in
> `cycles/v0.1.0-greenfield-spec/holdout-eval/` and relocated here (M-3)
> to consolidate evaluation artifacts under `holdout-scenarios/evaluations/`.
> History is preserved via `git mv`.

## Evaluation Files

| ID | File | Subject / Scenario Range | Aggregate Score | Result |
|----|------|--------------------------|-----------------|--------|
| EVAL-001 | [chunk1-eval.md](chunk1-eval.md) | Chunk 1 — HS-001..HS-024 range (20 scenarios) | 0.9475 | PASS |
| EVAL-002 | [chunk2-eval.md](chunk2-eval.md) | Chunk 2 — HS-025..HS-049 range (20 scenarios); HS-043 genuine defect (0.50) | 0.945 | PASS (defect filed) |
| EVAL-003 | [chunk3-eval.md](chunk3-eval.md) | Chunk 3 — HS-051..HS-074 range (20 scenarios); initial run, evaluator-coverage artifact on 12 scenarios | 0.612 (initial) | SUPERSEDED by EVAL-004 |
| EVAL-004 | [chunk3-reeval.md](chunk3-reeval.md) | Chunk 3 re-evaluation — 12 unverified scenarios rescored with adversarial pcap inputs | 0.9917 (re-eval) | PASS |
| EVAL-005 | [chunk4-eval.md](chunk4-eval.md) | Chunk 4 — HS-076..HS-100 range (20 scenarios) | 0.948 | PASS |
| EVAL-006 | [hs043-revalidation.md](hs043-revalidation.md) | HS-043 re-validation + reassembly regression spot-check post PR #171 fix | 1.00 | PASS |

## Overall Phase-4 Result

- **Verdict: PASSED** (2026-06-01)
- Mean satisfaction across all chunks (post re-eval): **0.949**
- Must-pass violations (score < 0.6): **0** (HS-043 was a real defect, fixed in PR #171 before gate close)
- Detail: `.factory/cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`
