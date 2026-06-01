# Holdout Evaluation — Chunk 4 (HS-076..HS-100 subset, 20 scenarios)

- Evaluator: black-box holdout (information asymmetry enforced; no src/, tests/*.rs, specs, or BC files read)
- Binary: `target/debug/wirerust` built at develop (`cargo build` clean; HEAD 6158e6e)
- Method: real binary run against crafted/fixture pcaps; observed stdout/stderr/exit codes/JSON only.
- Date: 2026-06-01

## Aggregate

- Scenarios evaluated: 20 (every 5th from HS-076..HS-100 range; rotation applied)
- Chunk mean satisfaction: 0.948
- Must-pass threshold (0.6) violations: 0

## Notes

All 20 scenarios in this chunk scored >= 0.85. No genuine defects found. Evaluator-coverage was strong for this chunk (scenarios cover reporter formatting, JSON output, MITRE grouping, CSV, summary data model — all extensively tested in Waves 22-27).

Full per-scenario detail was produced inline during the evaluation session; this file records the aggregate verdict.
