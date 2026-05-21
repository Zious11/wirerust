---
pass: 7
scope: story-decomposition
date: 2026-05-21
verdict: NOT_CONVERGED
findings: 0C/0H/1M/1L/2N
blocking: 1
convergence_streak: 0/3
---

# Story Adversarial Pass 7 — NOT_CONVERGED (streak reset 1/3 → 0/3)

## Summary

Pass 7 was the intended second confirmation pass on the unchanged story/holdout package. The
adversary found one MEDIUM defect in HS-081: an unseeded MITRE technique (T1021/Lateral Movement)
was used in four load-bearing places where the seeded technique (T1071/Command and Control) was
required. Because the scenario constructs findings with T1021 but the verification rubric tests
for kill-chain ordering between Defense Evasion and Command and Control, the scenario would have
failed correct code — T1021 buckets to Uncategorized, not Command and Control. This is a
fabrication defect, not a cosmetic one.

One LOW was also found in HS-053: T1046 was described as a "tactic" when it is a technique
(Network Service Discovery is the technique; Discovery is the tactic).

Both findings were remediated inline. The convergence streak resets: **1/3 → 0/3**. Pass 8 next.

## Scope Verified (7 Focus Areas)

All 7 structural focus areas were independently re-verified and found clean except where noted:

1. **VP anchoring** — clean; no orphaned VP references.
2. **BC traceability** — 217/217 BCs traced to ≥1 story; no BC left uncovered.
3. **AC quality** — acceptance criteria across stories are specific and testable.
4. **Dependency-graph integrity** — 77 edges, acyclic, wave assignments consistent.
5. **Holdout coverage** — 100 holdout scenarios; all 27 waves covered; HS-081 content defect
   isolated (single file, single technique ID substitution); no systemic fabrication pattern
   detected across the other 99 scenarios.
6. **Cross-artifact counts** — story count (48), BC (217), VP (20), HS (100), wave (27)
   consistent across index files and wave-schedule.
7. **Sprint-state consistency** — sprint-state.yaml 48 entries, current_wave 1, consistent.

The defect was isolated, not systemic: only HS-081 contained the unseeded technique ID;
spot-checks of adjacent holdout scenarios (HS-075–HS-085) and all 17 other security-probe
scenarios found no analogous unseeded technique substitutions.

## Findings

### Medium (1) — remediated

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| F-1 | MEDIUM | `holdout-scenarios/HS-081-terminal-mitre-grouping-kill-chain-order.md` | Unseeded MITRE technique T1021 (Lateral Movement tactic) used in 4 load-bearing places: Scenario step 1, Verification Approach steps 2/4, and Edge Conditions. The seeded technique for the Command and Control attack phase is T1071 (Application Layer Protocol). T1021 would bucket to Uncategorized under the MITRE kill-chain grouping, causing the scenario to fail correct code (kill-chain ordering test between Defense Evasion and Command and Control becomes vacuous without a C2 technique). Fixed → T1071/Command and Control in all 4 sites. |

### Low (1) — remediated

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| F-2 | LOW | `holdout-scenarios/HS-053-http-path-traversal-detection.md` | Scenario step 4 and BC Linkage table describe T1046 as the "MITRE tactic" for admin-panel detection. T1046 is a technique (Network Service Discovery); Discovery is the tactic. The wording "MITRE technique T1046" in the scenario body and verification step is correct, but the BC Linkage table cell reads "Inconclusive/Low; T1046" without the tactic label being wrong — the prose in step 4 said "MITRE technique T1046 (Network Service Discovery, Discovery tactic)" which is now correctly stated. Wording corrected to unambiguously label T1046 as a technique. |

### Nitpick (2) — deferred (carried from Pass 6)

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| F-06-04 | NITPICK | Multiple STORY files | `input_hash` placeholder values — expected pre-dispatch. Not a defect. |
| F-06-05 | NITPICK | `holdout-scenarios/HS-085.md` | Wave attribution loosely derived; internally consistent. Not a defect. |

## Remediation

Both F-1 and F-2 were remediated inline before this report was committed:

- **HS-081**: T1021/Lateral Movement replaced with T1071/Command and Control in all 4
  load-bearing locations (Scenario step 1, Verification Approach steps 2 and 4, Edge Conditions).
- **HS-053**: Technique/tactic wording clarified; T1046 unambiguously identified as a technique
  (Network Service Discovery) under the Discovery tactic.

## Verdict

**NOT_CONVERGED** — one MEDIUM blocking finding (F-1 would cause correct implementation to fail
HS-081). Defect was isolated: single file, single technique-ID substitution, no systemic
fabrication pattern across the 99 other holdout scenarios. Both findings remediated.

Convergence streak resets: **1/3 → 0/3**. Pass 8 next.
