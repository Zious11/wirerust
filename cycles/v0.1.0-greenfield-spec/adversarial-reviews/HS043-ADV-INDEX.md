---
document_type: adversarial-review-index
level: ops
version: "1.0"
status: in-review
producer: adversary
timestamp: 2026-06-01T00:00:00Z
phase: 5
pass: 2
inputs: [src/reassembly/mod.rs, src/cli.rs, src/main.rs, tests/hs043_flow_expiry_tests.rs, BC-2.04.013.md, BC-2.04.017.md, holdout-scenarios/HS-043-timeout-idle-cleanup.md]
traces_to: BC-2.04.013
total_findings: 3
severity_distribution: { CRIT: 0, HIGH: 0, MED: 1, LOW: 2 }
---

# Adversarial Review Index — HS-043 Flow-Expiry Wiring

## Pass Log

| Pass | File | CRIT | HIGH | MED | LOW | Method | Notes |
|------|------|------|------|-----|-----|--------|-------|
| 1 | HS043-pass-1.md | 0 | 0 | 0 | 1 | live-mutation (orchestrator) | core correctness confirmed via 9-mutation battery; 1 doc-only LOW |
| 2 | HS043-pass-2.md | 0 | 0 | 1 | 1 | fresh-context (probe-confirmed) | NEW MED: monotonic sweep gate suppresses expiry for sustained lower-ts runs (multi-file/reordered); LOW re-surface of PC0 wording |
| 3 | (pending) | — | — | — | — | fresh-context adversary | after MED disposition; min 3 clean passes |

## Finding Catalog

| ID | Severity | Category | Title | Status | Blocks Merge? |
|----|----------|----------|-------|--------|---------------|
| ADV-HS043-P01-LOW-001 | LOW | spec-fidelity | BC-2.04.013 PC0 wording says `expire_flows`; impl wires time-only `expire_idle_by_timeout` (forced by BC-2.04.017) | open | No |
| ADV-HS043-P02-MED-001 | MED | correctness / memory-bound | Monotonic sweep gate (`timestamp > last_expiry_sweep_secs`) suppresses expiry for sustained lower-timestamp runs (multi-file/reordered captures); idle flows accumulate unbounded, defeats HS-043 memory bound | open | triage required |
| ADV-HS043-P02-LOW-001 | LOW | spec-fidelity | re-surface of P01-LOW-001 (PC0 wording vs `expire_idle_by_timeout`) | open | No |

## Convergence Status

NOT CONVERGED — Pass-2 (first fresh-context pass) surfaced 1 MEDIUM + 1 LOW (0/0/1/1).
The MEDIUM (ADV-HS043-P02-MED-001) is a probe-confirmed production correctness gap in the
headline memory-bound capability and must be triaged — either fixed (decouple sweep cadence
from timestamp monotonicity) or formally accepted with a BC-2.04.013 amendment scoping the
memory bound to monotonic captures. Trajectory increased 1→2 findings; this reflects
first-fresh-context discovery (Pass-1 was an orchestrator mutation battery), not a
fix-introduced regression. Minimum 3 clean passes still required; Pass-3 fresh-context to
run after MED disposition.
