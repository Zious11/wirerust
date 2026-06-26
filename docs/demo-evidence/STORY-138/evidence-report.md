# Demo Evidence Report — STORY-138

**Story:** ENIP Session Lifecycle, Statistics, DoS Guard, and Analyzer Summary
**Story ID:** STORY-138
**Wave:** 61
**Product type:** Pure-core library (no CLI/UI surface — EtherNet/IP analyzer session lifecycle, statistics, and summary)
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)
**Recorded:** 2026-06-26
**Test result at recording time:** 20 passed / 0 failed / 0 ignored (mod session_lifecycle)

---

## AC Coverage Map

| AC | Title | BC | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|----|-----------------|---------------|----------------|------|
| AC-138-001 | RegisterSession/UnRegisterSession classified, PDU-counted, no finding | BC-2.17.025, BC-2.17.024 | `test_register_session`, `test_unregister_session`, `test_session_command` | `AC-001-session-handshake.gif` | `AC-001-session-handshake.webm` | `AC-001-session-handshake.tape` |
| AC-138-002 | on_flow_close removes flow state and folds counters into aggregates | BC-2.17.017 | `test_flow_close`, `test_flows_analyzed` | `AC-002-flow-close.gif` | `AC-002-flow-close.webm` | `AC-002-flow-close.tape` |
| AC-138-003 | pdu_count per valid frame; split-header double-count fix (F-W60-P1-001) | BC-2.17.024, BC-2.17.025 Post 2 | `test_pdu_count`, `test_command_count` | `AC-003-pdu-count.gif` | `AC-003-pdu-count.webm` | `AC-003-pdu-count.tape` |
| AC-138-004 | MAX_FINDINGS=10,000 hard cap with dropped_findings counter | BC-2.17.022 | `test_max_findings`, `test_dropped_findings`, `test_stats_accumulate` | `AC-004-dos-guard.gif` | `AC-004-dos-guard.webm` | `AC-004-dos-guard.tape` |
| AC-138-005 | summarize() produces enip_summary with BC-2.17.021 canonical schema | BC-2.17.021 | `test_summarize`, `test_summary` | `AC-005-summarize.gif` | `AC-005-summarize.webm` | `AC-005-summarize.tape` |

**Master green-run** covering all 20 tests (AC-138-001 through AC-138-005):

| Artifact | Description |
|----------|-------------|
| `AC-ALL-session-lifecycle-20-green.gif` | Full `mod session_lifecycle` — 20/20 green |
| `AC-ALL-session-lifecycle-20-green.webm` | Full `mod session_lifecycle` — 20/20 green |
| `AC-ALL-session-lifecycle-20-green.tape` | VHS script for master suite |

---

## Recordings Detail

### AC-001-session-handshake

Demonstrates `RegisterSession (0x0065)` and `UnRegisterSession (0x0066)` are classified and
PDU-counted by the ENIP analyzer but emit NO finding (BC-2.17.025 Postconditions 1–5,
BC-2.17.024 Postconditions 1–2). Session-handle anomaly detection (T0814) is explicitly
deferred to v0.12.0 per BC-2.17.025 Invariant 3 — no session-state tracking is added.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to session handshake tests
- `test_register_session_pdu_counted_no_finding`: 0x0065 frame → `pdu_count=1`; zero findings
- `test_unregister_session_pdu_counted_no_finding`: 0x0066 frame → `pdu_count=1`; zero findings
- `test_session_command_counts_accumulated`: multiple session frames → `command_counts[0x0065]`
  and `command_counts[0x0066]` correctly accumulated; no findings
- All 3 tests pass green

**Tests in recording:**
- `test_register_session_pdu_counted_no_finding`
- `test_unregister_session_pdu_counted_no_finding`
- `test_session_command_counts_accumulated`

---

### AC-002-flow-close

Demonstrates `EnipAnalyzer::on_flow_close(flow_key)` removes the `EnipFlowState` from the
per-flow map, folds `pdu_count`, `parse_errors`, and `command_counts` into aggregate counters,
increments `flows_analyzed`, and is a no-op (no panic) for unknown flow keys
(BC-2.17.017 Postconditions 1–6).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to flow-close and flows_analyzed tests
- `test_flow_close_removes_state`: `HashMap::remove` succeeds; flow is gone from `self.flows`
- `test_flow_close_folds_pdu_count`: `total_pdu_count += flow.pdu_count` after close
  (EC-007 pattern: `pdu_count=10` → `total_pdu_count=10`)
- `test_flow_close_folds_parse_errors`: `self.parse_errors += flow.parse_errors` after close
  (EC-007 pattern: `parse_errors=2` → aggregate `parse_errors=2`)
- `test_flow_close_unknown_key_no_panic`: unknown `flow_key` → no-op, no panic (EC-008)
- `test_flows_analyzed_incremented_on_flow_close`: `flows_analyzed==1` after one open+close;
  unknown-key call does NOT increment `flows_analyzed` (BC-2.17.017 Postcondition 6)
- All 5 tests pass green

**Tests in recording:**
- `test_flow_close_removes_state`
- `test_flow_close_folds_pdu_count`
- `test_flow_close_folds_parse_errors`
- `test_flow_close_unknown_key_no_panic`
- `test_flows_analyzed_incremented_on_flow_close`

---

### AC-003-pdu-count

Demonstrates `flow.pdu_count` increments at the start of each `process_pdu` call (valid frames
only — frames rejected by `is_valid_enip_frame` never reach `process_pdu`). Also demonstrates
the **F-W60-P1-001 regression fix**: when a TCP segment boundary falls exactly after the
24-byte ENIP header (complete header, no payload), the carry-stash arm must NOT increment
`command_counts`; the increment fires only on the subsequent `on_data` call that commits or
rejects the frame — producing exactly ONE count per logical header (BC-2.17.024, BC-2.17.025
Postcondition 2).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to pdu_count and command_count tests
- `test_pdu_count_increments_on_valid_frame`: valid frame → `flow.pdu_count=1`
  (BC-2.17.024 Postcondition 1)
- `test_pdu_count_not_incremented_on_invalid_frame`: structurally invalid frame rejected by
  `is_valid_enip_frame` → `pdu_count=0` (BC-2.17.024 Postcondition 3)
- `test_command_count_accumulates`: multiple frames drive full pipeline; `command_counts`
  reflects frame-walk site counts; `pdu_count` reflects only valid-frame counts
- `test_command_counts_no_double_count_on_split_header`: first `on_data` delivers exactly 24
  bytes (complete header, no payload) → carry stashed; second `on_data` delivers payload →
  frame committed; assert `command_counts[cmd]==1` (not 2) — F-W60-P1-001 regression confirmed
- All 4 tests pass green

**Tests in recording:**
- `test_pdu_count_increments_on_valid_frame`
- `test_pdu_count_not_incremented_on_invalid_frame`
- `test_command_count_accumulates`
- `test_command_counts_no_double_count_on_split_header`

---

### AC-004-dos-guard

Demonstrates the `MAX_FINDINGS = 10_000` hard cap on `self.all_findings`. When `all_findings.len()
>= MAX_FINDINGS`: no new `Finding` is pushed; `dropped_findings` increments; per-flow counters
(`pdu_count`, `command_counts`, `parse_errors`) continue to update. One-shot guards
(`write_burst_emitted`, `error_rate_emitted`, `malformed_anomaly_emitted`) are NOT set on
cap-suppressed findings (BC-2.17.022 Postconditions 1–5, Invariants 1–5).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to DoS guard tests
- `test_max_findings_cap`: fills `all_findings` to `MAX_FINDINGS`; next finding attempt →
  `all_findings.len()` stays at `MAX_FINDINGS` (EC-006; BC-2.17.022 Postcondition 2)
- `test_dropped_findings_incremented_at_cap`: at cap, `dropped_findings` increments on each
  suppressed finding (BC-2.17.022 Postcondition 3)
- `test_stats_accumulate_past_max_findings`: `pdu_count` and other counters continue
  incrementing after cap is reached (BC-2.17.022 Invariant 3 / Postcondition 4)
- All 3 tests pass green

**Tests in recording:**
- `test_max_findings_cap`
- `test_dropped_findings_incremented_at_cap`
- `test_stats_accumulate_past_max_findings`

---

### AC-005-summarize

Demonstrates `EnipAnalyzer::summarize()` produces an `enip_summary` JSON object with exactly
the 7 canonical keys specified by BC-2.17.021: `command_distribution`, `total_pdu_count`,
`parse_errors` (canonical — NOT `total_parse_errors`), `write_count`, `error_count`,
`flows_analyzed`, `dropped_findings`. Zero-flow case produces a valid JSON object with all
counts at 0. The canonical `parse_errors` key name is enforced from day one per the v0.10.0
rename lesson (BC-2.15.020 D-220 / BC-2.17.021 Invariant 1).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to summarize/summary tests
- `test_summarize_produces_enip_summary`: all 7 canonical keys present in the output object
  (BC-2.17.021 Postcondition 1)
- `test_summary_parse_errors_key_canonical`: JSON key is `"parse_errors"` — NOT
  `"total_parse_errors"` (BC-2.17.021 Invariant 1 — critical breaking-change guard)
- `test_summary_zero_flow_case`: 0 flows → all counts are 0; `enip_summary` JSON object
  is still present (BC-2.17.021 Postcondition 2, Invariant 3 — EC-009)
- `test_summary_dropped_findings`: `dropped_findings=50` → `enip_summary.dropped_findings=50`
  (BC-2.17.021 Postcondition 1 / BC-2.17.022 Invariant 4 — EC-012)
- `test_summary_flows_analyzed_nonzero`: open and close one flow; `summarize()` →
  `enip_summary.flows_analyzed >= 1` (BC-2.17.021 canonical vector validation)
- All 5 tests pass green

**Tests in recording:**
- `test_summarize_produces_enip_summary`
- `test_summary_parse_errors_key_canonical`
- `test_summary_zero_flow_case`
- `test_summary_dropped_findings`
- `test_summary_flows_analyzed_nonzero`

---

### AC-ALL-session-lifecycle-20-green

Master green-run for the full `mod session_lifecycle` suite — 20 tests covering all 5
STORY-138 acceptance criteria.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests session_lifecycle`
- All 20 tests in `mod session_lifecycle` execute and pass
- Test result line: `test result: ok. 20 passed; 0 failed; 0 ignored`

**Tests in recording (all 20):**
- `test_command_count_accumulates`
- `test_command_counts_no_double_count_on_split_header`
- `test_dropped_findings_incremented_at_cap`
- `test_flow_close_folds_parse_errors`
- `test_flow_close_folds_pdu_count`
- `test_flow_close_removes_state`
- `test_flow_close_unknown_key_no_panic`
- `test_flows_analyzed_incremented_on_flow_close`
- `test_max_findings_cap`
- `test_pdu_count_increments_on_valid_frame`
- `test_pdu_count_not_incremented_on_invalid_frame`
- `test_register_session_pdu_counted_no_finding`
- `test_session_command_counts_accumulated`
- `test_stats_accumulate_past_max_findings`
- `test_summarize_produces_enip_summary`
- `test_summary_dropped_findings`
- `test_summary_flows_analyzed_nonzero`
- `test_summary_parse_errors_key_canonical`
- `test_summary_zero_flow_case`
- `test_unregister_session_pdu_counted_no_finding`

---

## Full session_lifecycle Test Suite Summary

All 20 tests in `mod session_lifecycle` pass at recording time:

```
test session_lifecycle::test_command_count_accumulates ... ok
test session_lifecycle::test_command_counts_no_double_count_on_split_header ... ok
test session_lifecycle::test_dropped_findings_incremented_at_cap ... ok
test session_lifecycle::test_flow_close_folds_parse_errors ... ok
test session_lifecycle::test_flow_close_folds_pdu_count ... ok
test session_lifecycle::test_flow_close_removes_state ... ok
test session_lifecycle::test_flow_close_unknown_key_no_panic ... ok
test session_lifecycle::test_flows_analyzed_incremented_on_flow_close ... ok
test session_lifecycle::test_max_findings_cap ... ok
test session_lifecycle::test_pdu_count_increments_on_valid_frame ... ok
test session_lifecycle::test_pdu_count_not_incremented_on_invalid_frame ... ok
test session_lifecycle::test_register_session_pdu_counted_no_finding ... ok
test session_lifecycle::test_session_command_counts_accumulated ... ok
test session_lifecycle::test_stats_accumulate_past_max_findings ... ok
test session_lifecycle::test_summarize_produces_enip_summary ... ok
test session_lifecycle::test_summary_dropped_findings ... ok
test session_lifecycle::test_summary_flows_analyzed_nonzero ... ok
test session_lifecycle::test_summary_parse_errors_key_canonical ... ok
test session_lifecycle::test_summary_zero_flow_case ... ok
test session_lifecycle::test_unregister_session_pdu_counted_no_finding ... ok

test result: ok. 20 passed; 0 failed; 0 ignored
```

---

## Approach Note

**Test-run recordings used (not pcap-driven CLI output).** This is the established pattern for
STORY-13x demos: the ENIP analyzer is a pure-core library with no standalone CLI entry point
for injecting hand-crafted EtherNet/IP frames. The acceptance criteria are fully expressed as
unit tests in `tests/enip_analyzer_tests.rs`. Recordings show `cargo test` output filtered to
relevant test names and the final `test result:` line, identical to STORY-134, STORY-135,
STORY-136, and STORY-137 precedent.
