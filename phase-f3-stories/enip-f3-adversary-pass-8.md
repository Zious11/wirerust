# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 8

## Verdict

**FAIL** — 0 CRITICAL, 1 HIGH, 0 MEDIUM, 1 LOW. Novelty: MODERATE.

Pass-7 fixes HELD. Convergence streak BROKEN (was 1/3) — reset to 0/3.

## Finding Summary

| ID | Severity | Title | Disposition |
|----|----------|-------|-------------|
| F8-001 | HIGH | Unknown/invalid ENIP commands structurally uncountable in command_counts — BC-2.17.004 vs BC-2.17.016 contradiction | REMEDIATION IN PROGRESS — PO BC fix + STORY-137/138/130 propagation |
| F8-002 | LOW | STORY-136 LargeForwardOpen shares ForwardOpen summary string — BC-faithful, NOT a defect | NO ACTION |

## Finding Detail

### F8-001 (HIGH) — Unknown/invalid ENIP commands structurally uncountable in command_counts

**Location:** STORY-137 process_pdu pseudocode; BC-2.17.004 PC2/Inv3; BC-2.17.016

**Finding:** BC-2.17.004 PC2/Inv3 mandates counting EVERY parsed header in
`command_counts`, including headers where `is_valid_enip_frame` returns false
(i.e., Unknown/invalid commands must land in the Unknown bucket). However, the
`command_counts` increment site lives in `process_pdu`, which is only reached
via the frame-walk path (BC-2.17.016). The frame-walk processes only frames
where `is_valid_enip_frame` succeeds — frames that fail validity (Unknown
commands) never reach `process_pdu`, so the Unknown bucket increment is
structurally unreachable.

Root cause: The convergence ledger recorded "command_distribution owned by
frame-walk" as resolved in a prior pass, but the resolution was never propagated
to the story bodies (S-7.01(c) partial-fix blind spot). The ledger claim
satisfied the adversary's checklist without verifying the physical AC text in
the stories.

**Contradiction:** BC-2.17.004 (count every parsed header incl. Unknown) vs
BC-2.17.016 (increment site is in process_pdu, reached only after
is_valid_enip_frame succeeds).

**Remediation options:**
- Option A (preferred): Relocate the `command_counts[Unknown] += 1` increment
  to the frame-walk, immediately after `parse_enip_header`, for all parsed
  headers including those where `is_valid_enip_frame` returns false. Remove the
  increment from `process_pdu`. Update BC-2.17.016 to document the single
  authoritative increment site. Propagate updated AC text to STORY-137 (owns
  frame-walk), STORY-138 (aggregate), and STORY-130 (parse foundation).
- Option B: Split counting — known-valid commands counted in process_pdu;
  Unknown bucket incremented at frame-walk. Adds complexity and two increment
  paths, which BC-2.17.021 Invariant 2 disfavors.

**REMEDIATION IN PROGRESS:** PO BC fix (BC-2.17.004 v1.1 + BC-2.17.016 v1.1
documenting single increment site at frame-walk) + STORY-137/138/130 propagation
of updated AC pseudocode.

### F8-002 (LOW) — STORY-136 LargeForwardOpen summary string

**Location:** STORY-136 AC pseudocode summary field

**Finding:** STORY-136 `LargeForwardOpen` produces the same summary string as
`ForwardOpen`. Initial inspection flagged this as a potential missing
differentiation.

**Disposition:** NOT a defect. BC-2.17.015 PC1 specifies the same summary
string for both variants. This is BC-faithful behavior. NO ACTION.

## Process Gap

**[process-gap]:** The convergence checklist should add a mandatory
"ledger-claim → story-body grep" verification step for any claim of the form
"X is owned by Y" or "X increment site is Z". A ledger entry alone is
insufficient — the claim must be corroborated by a grep of the physical AC text
in the affected story files before the finding is closed.

## Confirmed-Held Fixes (Pass-7 findings all HELD)

- VP-032 harnesses (Sub-A/B/C/D + vp032_cip_service_request_partition) — VERIFIED
- 13 holdout scenarios HS-110..122 present and git-tracked — VERIFIED
- dependency-graph E-20 chain acyclic (STORY-133→STORY-136 vec![]/catalog-prereq) — VERIFIED
- flows_analyzed/dead-counter sweep — all 7 enip_summary fields confirmed — VERIFIED
- strict `>` windows (BC-2.17.023/026 threshold logic) — VERIFIED
- ForwardOpen `vec![]` empty emitted_tactic_ids (BC-2.17.015) — VERIFIED
- STORY-134 command_counts single-increment disambiguation note — VERIFIED

## Severity Trajectory

| Pass | CRITICAL | HIGH | Result |
|------|----------|------|--------|
| P1 | 4 | 6 | FAIL |
| P2 | 1 | 3 | FAIL |
| P3 | 0 | 2 | FAIL |
| P4 | 2 | 2 | FAIL |
| P5 | 0 | 1 | FAIL |
| P6 | 0 | 1 | FAIL |
| P7 | 0 | 0 | PASS |
| **P8** | **0** | **1** | **FAIL** |

## Convergence Counter

**0/3** — streak RESET. Pass-7 was the first clean pass; Pass-8 FAIL breaks it.
Pass 9 pending (after remediation of F8-001 command_counts relocation).
