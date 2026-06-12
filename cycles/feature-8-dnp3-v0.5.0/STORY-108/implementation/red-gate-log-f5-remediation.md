---
document_type: red-gate-log
cycle: feature-8-dnp3-v0.5.0
story: STORY-108 F5-remediation
branch: fix/dnp3-f5-unexpected-source
worktree: /Users/zious/Documents/GITHUB/wirerust/.worktrees/F5-DNP3-FIX
date: 2026-06-11
produced_by: test-writer
---

# Red Gate Log — F5 Remediation Tests

## Scope

Two fix sets:
- **F-F5-001:** BC-2.15.010 Invariant 5 — unexpected-source detection (HS-W37-002 P0)
- **F-F5-002:** MitreTactic::IcsImpact / Impact display collision

## Test File

`tests/dnp3_f5_remediation_tests.rs`

## Compile Stub Added

`src/analyzer/dnp3.rs` — `Dnp3FlowState::unexpected_source_emitted: bool` field added
with `#[derive(Default)]` (zero value = false). No detection logic implemented.

## Red Gate Results

```
test result: FAILED. 1 passed; 6 failed; 0 ignored; 0 measured
```

### Tests FAILING (behavior assertions — correct Red Gate)

| Test fn | File:line | Failure reason |
|---------|-----------|----------------|
| `test_unexpected_source_fires_at_count_1` | tests/dnp3_f5_remediation_tests.rs:170 | `all_findings.len()=0, expected 1` — detect_unexpected_source_split not implemented |
| `test_unexpected_source_independent_of_threshold` | tests/dnp3_f5_remediation_tests.rs:291 | `all_findings.len()=0, expected 1` — detect_unexpected_source_split not implemented |
| `test_unexpected_source_one_shot_guard` | tests/dnp3_f5_remediation_tests.rs:369 | `unexpected-source count=0, expected 1` — detect_unexpected_source_split not implemented |
| `test_unexpected_source_and_burst_both_fire` | tests/dnp3_f5_remediation_tests.rs:493 | `all_findings.len()=1, expected 2` — burst fires but unexpected-source does not |
| `test_ics_impact_display_distinct_from_impact` | tests/dnp3_f5_remediation_tests.rs:580 | `"Impact" == "Impact"` — IcsImpact display not yet distinct from Impact |
| `test_reporter_renders_distinct_impact_sections` | tests/dnp3_f5_remediation_tests.rs:715 | `unique headers=1 {"  ## Impact"}, expected 2` — both tactic buckets render identical header |

### Tests PASSING (correct behavior already exists)

| Test fn | Reason green |
|---------|-------------|
| `test_first_master_is_expected` | No false-positive guard: the first Control FC from the first master correctly emits no finding (expected set is established, not unexpected). `master_addrs_seen=[0x0001]` populated correctly. `unexpected_source_emitted=false`. |

## Summary of Pinned Assertions (F-F5-001)

From F-F5-001-unexpected-source-adjudication.md §2 "Finding fields (exact)":

```
category:   ThreatCategory::Execution
verdict:    Verdict::Likely
confidence: Confidence::High          ← High, NOT Medium (burst is Medium)
summary:    "DNP3 unauthorized control command from unexpected source: \
             src={src:#06X} is not in expected master set \
             {master_set} on dest={dest:#06X}"
mitre_techniques: ["T1692.001"]
```

Tests 1 and 2 assert all five fields. Tests 3-5 assert the summary discriminator
(`contains("unexpected source")`). Test 5 also asserts the burst summary
discriminator (`contains("threshold 3")`).

## Existing Test Suite Status

All 21 pre-existing test binaries: PASSED (0 regressions introduced).

The `unexpected_source_emitted: bool` stub field on `Dnp3FlowState` (added via
`#[derive(Default)]`) introduces zero behavior change to existing logic — it is
a new field with false-default that no existing code reads or writes.

## Handoff to Implementer

Implement:
1. `detect_unexpected_source_split` associated function (signature per adjudication §3).
2. Call site in `on_data` using pre-push snapshot (adjudication §3 recommended approach).
3. Ordering: BEFORE master_addrs_seen push, using `src_was_known` + `expected_set_established`
   snapshot booleans (adjudication §3 "RECOMMENDED approach").
4. Fix `MitreTactic::IcsImpact` Display arm to return a distinct string (e.g. "Impact (ICS)").

Make each failing test pass, one at a time, with minimum code. `test_first_master_is_expected`
must remain green throughout.
