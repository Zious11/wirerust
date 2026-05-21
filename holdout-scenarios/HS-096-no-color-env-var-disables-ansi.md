---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs: [stories/, behavioral-contracts/, prd.md]
input-hash: "[md5-pending]"
traces_to: ""
id: "HS-096"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.010
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: NO_COLOR Environment Variable Disables ANSI Output Regardless of --no-color Flag

## Scenario

The `NO_COLOR` environment variable is a cross-ecosystem convention (https://no-color.org)
that tools should honor. wirerust must disable colorized output when `NO_COLOR` is set in
the environment, regardless of whether the `--no-color` flag was also given.

1. The environment variable `NO_COLOR` is set to any value (including an empty string `""`).
2. The tool is invoked WITHOUT the `--no-color` flag.
3. The terminal output contains no ANSI escape sequences (`\x1b[` sequences).
4. The content of the output is otherwise intact — the lack of color does not remove findings
   or other report content.

Separately:
5. When `NO_COLOR` is NOT set and `--no-color` is also absent, `use_color = true`.
6. With color enabled, at least some ANSI escape sequences appear in the output for findings
   with Likely/High verdict.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.010 | Postcondition 1: NO_COLOR env var → use_color = false | Items 1-4: env-driven color disable |
| BC-2.12.010 | Postcondition 2: no NO_COLOR + no --no-color → use_color = true | Items 5-6: default color enabled |

## Verification Approach

**Test 1 — env var disables color:**
Spawn the tool in a subprocess with `NO_COLOR=""` in the environment (no `--no-color` flag).
Capture stdout. Assert: no `\x1b[` byte sequence appears in stdout.
Assert: the output still contains report content (not empty).

**Test 2 — absence means color:**
Spawn the tool without `NO_COLOR` and without `--no-color`, against a pcap that produces a
Likely/High finding.
Capture stdout. Assert: at least one `\x1b[` sequence appears (color is active).

Note: These tests must run serially (not in parallel with other tests that modify the `NO_COLOR`
environment variable) to prevent cross-test contamination.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): NO_COLOR=any_value produces no ANSI codes; absent NO_COLOR uses color.
- **Edge case handling** (weight: 0.2): NO_COLOR="" (empty value) still disables color — any set value counts.
- **Error quality** (weight: 0.1): Tool does not crash when NO_COLOR is set to unusual values.
- **Performance** (weight: 0.05): Env var check adds no measurable overhead.
- **Data integrity** (weight: 0.15): Report content (findings, summary counts) is identical with or without color.

## Edge Conditions

- `NO_COLOR` with any non-empty value (e.g., `NO_COLOR=1`, `NO_COLOR=yes`): disables color.
- `NO_COLOR=""` (empty string): disables color (the convention is: any set value, including empty).
- Both `NO_COLOR` and `--no-color` set: both lead to `use_color = false`; no conflict.
- `NO_COLOR` is unset (not present in environment): does not influence color mode.

## Failure Guidance

"HOLDOUT LOW: HS-096 (satisfaction: 0.XX) -- Setting NO_COLOR in the environment did not disable ANSI output, or an empty-string value of NO_COLOR was treated as 'unset' when it should disable color."
