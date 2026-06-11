---
document_type: convergence-summary
story: STORY-108
wave: 37
feature: "#8-dnp3"
cycle: feature-8-dnp3-v0.5.0
producer: state-manager
timestamp: 2026-06-11T19:46:40Z
pr: "#227"
merge_commit: 9c03fde
verdict: CONVERGED
clean_streak: 3/3
bc_gate: BC-5.39.001
---

# STORY-108 Adversarial Convergence Summary

Wave 37, Feature #8 DNP3. CONVERGED after 5 passes (3 consecutive CLEAN).

## Trajectory

| Pass | Verdict | Findings | Fix Commits | Summary |
|------|---------|----------|-------------|---------|
| P1 | FINDINGS | source_ip/timestamp hardcoded None — BC-2.15.010/011/012 PC violation | c216118 | `source_ip` and `timestamp` fields written as `None` instead of being resolved from packet metadata; violated 3 behavioral contracts. Fixed: populate from flow state. |
| P2 | FINDINGS | master-resolution test-vacuity: symmetric port-20000 test key never exercised the heuristic; extracted `resolve_master_ip` helper for DRY; documented direction-deferral | c216118-era tests + 78028cf | Adversary found the test used a symmetric key (both endpoints on port 20000) so the heuristic branch was never executed in any test. Extracted `resolve_master_ip` helper. Documented direction-deferral in code. |
| P3 | CLEAN (1 MINOR) | Stale AC-007 story citation | fb64529 | One doc comment cited the wrong story AC number. Fixed cosmetically. Adversary accepted as CLEAN gate-wise. |
| P4 | CLEAN | — | — | No findings. |
| P5 | CLEAN | — | — | No findings. BC-5.39.001 3/3 clean streak confirmed. |

## Deferred Finding

**DRIFT-DNP3-DIRECTION-001:** DNP3 `source_ip` resolution uses port-20000 heuristic only.
Direction-aware resolution (matching `modbus.rs` ~355-382, using the TCP `Direction` signal)
is deferred to the DNP3 dispatcher-integration story (the story that adds
`DispatchTarget::Dnp3` arm and threads `direction` into `Dnp3Analyzer::on_data`).
Current behavior: correct for standard flows (one endpoint on port 20000); returns
`lower_ip` when neither endpoint is on port 20000 (non-standard/proxied capture).
Documented at `src/analyzer/dnp3.rs` `resolve_master_ip`.
Per DF-VALIDATION-001: recorded as drift/tech-debt only — not filed as GitHub issue
without research-agent validation.

## Delivery Record

- PR #227 merged into develop @ 2026-06-11T19:46:40Z
- Merge commit: `9c03fde`
- Input-hash regenerated to `a4218c5` (commit `350c8b1`) before delivery
- STORY-109 and STORY-110 still carry F3-era hashes — LIKELY STALE (VP-023 v1.4 among inputs);
  run `bin/compute-input-hash --write .factory/stories/STORY-NNN.md` before each delivery
