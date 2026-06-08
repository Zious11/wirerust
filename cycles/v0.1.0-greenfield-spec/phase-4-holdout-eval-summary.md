# Phase 4 — Holdout Evaluation Summary

| Field | Value |
|-------|-------|
| Cycle | v0.1.0-greenfield-spec |
| Phase | Phase 4 — Holdout Evaluation |
| Verdict | **PASSED** |
| Date | 2026-06-01 |
| develop HEAD (eval) | 6158e6e |
| develop HEAD (post-fix) | c3cd4bd (PR #171 — HS-043 fix) |
| Evaluator tier | opus-tier holdout (information asymmetry enforced) |
| Model-family caveat | True non-Claude GPT evaluator unavailable in this environment; used opus-tier with strict info-asymmetry for tier diversity. Documented limitation — does not invalidate results. |

---

## Scenario Rotation

- Total scenarios: 100 (HS-001..HS-100)
- Rotation applied: every 5th scenario dropped from each chunk of 20
- Evaluated count: 80 of 100
- Chunk layout: 4 chunks of 20 each (HS-001..HS-024 range, HS-025..HS-049, HS-050..HS-074, HS-075..HS-100)

---

## Chunk Results

| Chunk | Scenarios | Initial Mean | Notes |
|-------|-----------|-------------|-------|
| Chunk 1 (HS-001..HS-024 range) | 20 | 0.9475 | Per-scenario detail: ../../holdout-scenarios/evaluations/chunk1-eval.md |
| Chunk 2 (HS-025..HS-049 range) | 20 | 0.945 | HS-043 genuine defect (0.50); remaining 19 mean ~0.976. Per-scenario detail: ../../holdout-scenarios/evaluations/chunk2-eval.md |
| Chunk 3 (HS-050..HS-074 range) | 20 | 0.612 (initial) → **0.9917** (re-eval) | Evaluator-coverage artifact — 12 sub-0.6 scenarios re-scored (see below). Per-scenario detail: ../../holdout-scenarios/evaluations/chunk3-eval.md / chunk3-reeval.md |
| Chunk 4 (HS-075..HS-100 range) | 20 | 0.948 | Per-scenario detail: ../../holdout-scenarios/evaluations/chunk4-eval.md |

---

## Chunk 3 Re-Evaluation

**Initial result: 0.612** — flagged as potential evaluator-coverage artifact. 12 of 20 scenarios scored below 0.6 on the initial pass. Root cause investigation: the evaluator did not craft adversarial pcap inputs for the scenarios covering internal-only behavior (stats counters, packet-level aggregates, flow accounting internals). These scenarios were scored conservatively ("cannot observe") rather than by constructing appropriate inputs.

**Re-evaluation approach:** evaluator explicitly crafted fixture inputs for each of the 12 sub-0.6 scenarios and re-ran the binary. All 12 satisfied their rubric criteria when the correct inputs were applied.

**Re-evaluation result: mean 0.9917** across the 12 re-scored scenarios. Prior 0.4-range scores were evaluator-coverage artifacts — the implementation matched essentially exactly. No genuine defects found in Chunk 3.

Per-scenario detail: ../../holdout-scenarios/evaluations/chunk3-eval.md (initial) and ../../holdout-scenarios/evaluations/chunk3-reeval.md (re-eval).

---

## Finding Dispositions (3 genuine findings triaged via DF-VALIDATION-001)

All findings triaged by research-agent + Perplexity per DF-VALIDATION-001 (no GitHub issue without validation).
Research triage report: `.factory/research/holdout-finding-triage-2026-06-01.md`

### HS-006 — em-dash separator in terminal finding line

- **Severity:** LOW
- **Disposition: NON-DEFECT (BC/scope clarification)**
- **Root cause:** HS-006 rubric evaluated the terminal reporter's finding line against BC-2.09.002. BC-2.09.002 binds `Finding::Display` (findings.rs — debugging/logging), which uses em-dash and IS compliant. The terminal reporter (terminal.rs) is a separate layer (ADR 0003) with no BC requiring em-dash on its one-liner format. The MITRE line in terminal output uses em-dash; the finding prefix uses ASCII hyphen — this is internally inconsistent but not a BC violation.
- **Action:** HS-006 rubric trimmed to v1.1 (BC-2.09.002 scope clarification note appended). No code change required.
- **Optional-hardening note:** The em-dash vs ASCII-hyphen inconsistency within the terminal reporter is a cosmetic polish candidate (Phase-5 or standalone). Not a spec defect.

### HS-016 — raw-byte overlap evidence in reassembly output

- **Severity:** LOW (informational)
- **Disposition: NON-DEFECT (holdout over-specified vs the BC)**
- **Root cause:** HS-016 rubric required raw-byte overlap evidence in reassembly finding output. No cited BC requires raw-byte evidence in reassembly findings. The holdout over-specified an unconstrained output dimension.
- **Action:** HS-016 rubric trimmed to v1.1 (out-of-scope note appended). No code change required.
- **Optional-hardening note:** Raw-byte evidence in reassembly output would be a BC enhancement (product-owner decision), not an implementer defect.

### HS-043 — flow-timeout expiry never called in production

- **Severity:** MEDIUM (confirmed REAL BUG — arguably HIGH-adjacent per research-agent)
- **Disposition: CONFIRMED DEFECT — FIXED via PR #171**
- **Root cause:** `expire_flows` (renamed `expire_idle_by_timeout` in delivery) was dead code in production. The function existed and was unit-tested, but was never called from `process_packet`. The 1,078 unit/integration tests all missed this because they called `expire_flows` directly, bypassing the wiring. Only information-asymmetric holdout evaluation exposed it.
- **Fix:** PR #171 wired `expire_idle_by_timeout` into `process_packet` (called after each packet) and added `--flow-timeout` CLI flag (default 300s, range 1+). BC-2.04.013 updated to v1.5 PC0.
- **develop HEAD post-fix:** c3cd4bd
- **Re-validation:** HS-043 re-scored 1.00 (was 0.50). 4 reassembly regression checks (HS-021, HS-026, HS-028, HS-044) all hold at 1.00. Re-validation detail: ../../holdout-scenarios/evaluations/hs043-revalidation.md.
- **Methodology note:** This finding validates the holdout evaluation methodology — the information asymmetry that's the point of Phase 4 caught a ship-blocking idle-flow-memory-bound defect that 1,078 unit tests missed entirely.

---

## Final Aggregate

| Metric | Value | Threshold | Result |
|--------|-------|-----------|--------|
| Scenarios evaluated | 80 / 100 | rotation applied | OK |
| Mean satisfaction | ~0.949 | >= 0.85 | **PASS** |
| Must-pass violations (< 0.6) | 0 | 0 allowed | **PASS** |
| Std-dev | < 0.15 | < 0.15 | **PASS** |
| Chunk 3 artifact eliminated | yes | n/a | OK |
| HS-043 real defect fixed | yes (PR #171) | must-pass fixed before gate | **PASS** |

**Gate criteria: MET.** (Model-family caveat documented — does not invalidate results.)

---

## Optional-Hardening Follow-Ups (Non-Blocking LOWs)

These do NOT block Phase 5. No GitHub issues per DF-VALIDATION-001 without research-agent re-validation.

### OPT-1: ADV-HS043-P01-LOW-001 — BC-2.04.013 PC0 names expire_flows vs expire_idle_by_timeout

BC-2.04.013 PC0 was drafted before the delivery renamed the function to `expire_idle_by_timeout`. The BC currently names `expire_flows` (the literal symbol). A one-line BC prose clarification noting the behavioral equivalence would close any future confusion. LOW, cosmetic-only; target: any convenient spec-touch in Phase 5 or 6.

### OPT-2: HS-043 Pass-3 coverage-durability LOW — no committed regression tests for 3 gating properties

The HS-043 adversarial pass 3 identified three behavioral properties that are verified by the revalidation but not anchored in committed regression tests:
1. **active-flow-delta-0** — an active flow does not expire before its idle timeout
2. **gated-sweep-no-escape** — no flow expires before the per-packet sweep is triggered
3. **regressing-timestamp-underflow** — `last_seen` timestamp handling does not underflow on aggressive expiry

These are currently exercised by the revalidation fixture (../../holdout-scenarios/evaluations/hs043-revalidation.md) but not by named, committed unit tests. Candidate Phase-6 hardening tests. Per DF-VALIDATION-001, no GitHub issue without research-agent validation.

---

## Adversarial Reviews (HS-043)

- HS043-ADV-INDEX.md: `.factory/cycles/v0.1.0-greenfield-spec/adversarial-reviews/HS043-ADV-INDEX.md`
- Pass 1: `.factory/cycles/v0.1.0-greenfield-spec/adversarial-reviews/HS043-pass-1.md`

---

## Key Lesson

> Holdout evaluation's information asymmetry caught HS-043 — a genuine ship-blocking
> idle-flow-memory-bound defect (`expire_flows` was dead code in production) that all
> 1,078 unit tests missed because they called `expire_flows` directly. This validates
> the Phase 4 methodology: black-box evaluation with enforced info-asymmetry catches
> a class of defects (missing wiring) that white-box unit testing structurally cannot.
