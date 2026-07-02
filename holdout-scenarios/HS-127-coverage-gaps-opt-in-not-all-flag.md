---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-02T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.023.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.022.md
  - .factory/stories/STORY-153.md
  - .factory/stories/STORY-154.md
traces_to: .factory/specs/prd.md
id: "HS-127"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-21"
behavioral_contracts:
  - BC-2.12.023
  - BC-2.12.022
lifecycle_status: active
introduced: v0.12.0-feature-protocol-coverage
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires at least one pcap file for the `wirerust analyze` test cases. Any non-empty pcap from the existing test suite (e.g., one used in prior CLI integration tests) is sufficient. An empty pcap (SHB + IDB, no packets) is also sufficient for the `--coverage-gaps` empty output test."
input-hash: "157f48f"
---

# Holdout Scenario: `--coverage-gaps` Opt-In Semantics — Not Auto-Enabled by `--all`; Subcommand Scope Gating

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

`--coverage-gaps` is a NEW opt-in flag on the `analyze` subcommand. It is INDEPENDENT of
`--all` — enabling all analyzers does not enable gap detection. The flag must also not be
accepted on the `protocols` subcommand (it is an `analyze`-only flag). When `--coverage-gaps`
is set, a `CoverageGapsSummary` named section appears in the analysis output; when it is not
set, the output is IDENTICAL to pre-feature behavior (zero additive changes).

This scenario is not a canonical-value scenario. It verifies the opt-in wiring and scope
gating, not specific protocol port/EtherType constants.

### Case A — `analyze --all`: No CoverageGapsSummary (--all Does NOT Enable --coverage-gaps)

1. The evaluator obtains any valid non-empty pcap file (`test.pcap`).
2. The evaluator runs: `wirerust analyze test.pcap --all --json`
3. The tool exits 0.
4. The JSON output does NOT contain a `"coverage_gaps"` key at the top level.
5. The output is identical to pre-feature `wirerust analyze test.pcap --all --json` behavior.

### Case B — `analyze --coverage-gaps`: CoverageGapsSummary Section Is Present

1. The evaluator runs: `wirerust analyze test.pcap --coverage-gaps`
2. The tool exits 0.
3. The terminal output contains a `CoverageGapsSummary` section (or a section with that name,
   or equivalent text indicating coverage gap reporting).
4. The section appears AFTER any Finding entries in the output.

### Case C — `analyze --all --coverage-gaps`: Both Active Simultaneously

1. The evaluator runs: `wirerust analyze test.pcap --all --coverage-gaps --json`
2. The tool exits 0.
3. The JSON output has BOTH the normal analyzer findings AND a `"coverage_gaps"` key.
   This confirms `--all` and `--coverage-gaps` are independent, additive flags.

### Case D — `analyze` Without `--coverage-gaps`: No CoverageGapsSummary (Baseline Unchanged)

1. The evaluator runs: `wirerust analyze test.pcap --json`
2. The tool exits 0.
3. The JSON output does NOT contain a `"coverage_gaps"` key.
4. The output is identical to pre-feature baseline behavior.

### Case E — `protocols --coverage-gaps`: Clap Error (Flag Not Valid on `protocols` Subcommand)

1. The evaluator runs: `wirerust protocols --coverage-gaps`
2. The tool exits with a NON-ZERO exit code (clap error: `--coverage-gaps` is not defined
   on the `protocols` subcommand).
3. An error message appears on stderr.

### Case F — `analyze --coverage-gaps --json`: JSON Has `"coverage_gaps"` Object

1. The evaluator runs: `wirerust analyze test.pcap --coverage-gaps --json`
2. The tool exits 0.
3. `jq '."coverage_gaps"'` on the output is non-null.
4. The `"coverage_gaps"` object has at minimum a `"caveat_l2"` string field and an
   `"entries"` array field.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.023 | `--coverage-gaps` is NOT included in `--all` expansion | Case A: --all alone produces no CoverageGapsSummary |
| BC-2.12.023 | `--coverage-gaps` flag produces CoverageGapsSummary when set | Case B: section present |
| BC-2.12.023 | Both flags independent and additive | Case C: both --all and --coverage-gaps active |
| BC-2.12.023 | Without `--coverage-gaps`, output identical to pre-feature baseline | Case D: no CoverageGapsSummary |
| BC-2.12.023 | `--coverage-gaps` not valid on `protocols` subcommand | Case E: clap error |
| BC-2.12.023 | JSON mode: `"coverage_gaps"` key with object schema | Case F: valid JSON structure |

<!-- HIDDEN TRACEABILITY: BC-2.12.023 Invariant 1 (--coverage-gaps NOT in --all expansion; ADR-012 Decision 8);
     BC-2.12.023 Invariant 4 (CoverageGapsSummary is purely additive; existing Finding/AnalysisSummary unchanged);
     BC-2.12.023 Invariant 5 (flag not valid on protocols subcommand — EC-007) -->

## Verification Approach

```bash
# Case A — --all alone: no coverage_gaps key
wirerust analyze test.pcap --all --json | jq 'has("coverage_gaps")'
# Expect: false

# Case B — --coverage-gaps: section present in terminal output
wirerust analyze test.pcap --coverage-gaps | grep -i 'CoverageGapsSummary\|Coverage.*Gap\|gap.*summary'
# Expect: at least one line matching

# Case C — both flags
wirerust analyze test.pcap --all --coverage-gaps --json | jq 'has("coverage_gaps")'
# Expect: true

# Case D — no flag: no coverage_gaps
wirerust analyze test.pcap --json | jq 'has("coverage_gaps")'
# Expect: false

# Case E — protocols --coverage-gaps is a clap error
wirerust protocols --coverage-gaps; echo "exit: $?"
# Expect: non-zero exit

# Case F — JSON structure
wirerust analyze test.pcap --coverage-gaps --json | jq '."coverage_gaps" | {has_caveat: (has("caveat_l2")), has_entries: (has("entries"))}'
# Expect: {"has_caveat": true, "has_entries": true}
```

## Evaluation Rubric

- **`--all` does NOT enable coverage gaps** (weight: 0.35): Case A: `"coverage_gaps"` absent
  when only `--all` specified. This is the most important behavioral invariant — the flag
  is opt-in, not included in the convenience group.
- **`--coverage-gaps` produces the section** (weight: 0.30): Case B: section appears.
- **`protocols` subcommand rejects flag** (weight: 0.15): Case E: clap error.
- **JSON schema** (weight: 0.10): Case F: coverage_gaps object with caveat_l2 + entries.
- **Both flags independent** (weight: 0.10): Case C: both active simultaneously.

## Failure Guidance

"HOLDOUT FAIL: HS-127 — coverage-gaps opt-in wiring broken.
Case A failure: `--all` enables `--coverage-gaps`. The flag MUST be a separate, independent
flag on the Analyze subcommand, NOT included in the --all expansion group. See BC-2.12.023
Invariant 1 and ADR-012 Decision 8.
Case E failure: `protocols --coverage-gaps` should be a clap error. If it exits 0, the flag
was accidentally added to the protocols subcommand scope. The flag belongs exclusively on the
analyze subcommand (BC-2.12.023 Invariant 5)."
