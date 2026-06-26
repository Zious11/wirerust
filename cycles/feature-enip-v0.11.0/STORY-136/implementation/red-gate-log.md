---
story: STORY-136
phase: red-gate
date: 2026-06-26
agent: test-writer
---

# Red Gate Log — STORY-136: CIP Connection-Lifecycle Detection

## Verdict: PASS

All 6 behavioral tests fail via `todo!()` panic. All 4 suppression tests pass.
The Red Gate is valid.

## Test Results

### RED (6 tests — fail via `todo!()` panic at `src/analyzer/enip.rs:804`)

| Test Name | BC Reference | Failure Reason |
|-----------|-------------|---------------|
| `test_forward_open_emits_finding` | BC-2.17.015 PC-1; AC-136-001 | panicked: not yet implemented: STORY-136: CIP connection-lifecycle detection [BC-2.17.015] |
| `test_forward_open_no_mitre_technique` | BC-2.17.015 PC-1; AC-136-001 | panicked: not yet implemented: STORY-136: CIP connection-lifecycle detection [BC-2.17.015] |
| `test_large_forward_open_emits_finding` | BC-2.17.015 PC-1 invariant 5; AC-136-001 | panicked: not yet implemented: STORY-136: CIP connection-lifecycle detection [BC-2.17.015] |
| `test_forward_close_emits_finding` | BC-2.17.015 PC-4; AC-136-002 | panicked: not yet implemented: STORY-136: CIP connection-lifecycle detection [BC-2.17.015] |
| `test_forward_close_no_mitre_technique` | BC-2.17.015 PC-4; AC-136-002 | panicked: not yet implemented: STORY-136: CIP connection-lifecycle detection [BC-2.17.015] |
| `test_connection_counts_tracked` | BC-2.17.015 invariant 3; AC-136-005; EC-008 | panicked: not yet implemented: STORY-136: CIP connection-lifecycle detection [BC-2.17.015] |

### GREEN-by-design (4 tests — pass without hitting the todo!() stub)

| Test Name | BC Reference | Why it passes now |
|-----------|-------------|------------------|
| `test_forward_open_connected_item_no_finding` | BC-2.17.015 PC-3; F-P9-001 | 0x00B1 item skipped by type_id gate; todo!() unreachable |
| `test_forward_open_response_no_finding` | BC-2.17.007 Inv 1; BC-2.17.015 Inv 2 | classify_cip_service(0xD4) returns Response; match arm not entered |
| `test_forward_close_response_no_finding` | BC-2.17.007 Inv 1; BC-2.17.015 Inv 2 | classify_cip_service(0xCE) returns Response; match arm not entered |
| `test_non_enip_suppresses_connection_lifecycle` | BC-2.17.015 PC-4; AC-136-004 | is_non_enip=true returns early; todo!() unreachable |

## DF Policy Compliance

- **DF-AC-TEST-NAME-SYNC-001 v2:** All 10 test names match STORY-136.md Test Plan exactly. No drift found.
- **DF-KANI-NONVACUITY (spirit):** All 6 RED tests assert exact BC-2.17.015 fields: category=Anomaly, verdict=Possible, confidence=Low, mitre_techniques=vec![], summary prefix, source_ip/timestamp present; count fields (open_connection_count, close_connection_count), EC-008 (counts increment at MAX_FINDINGS cap).
- **DF-TEST-CITATION-SWEEP-001:** Each test has `// BC-2.17.015 / AC-NNN` style citations in docstring.
- **DF-GREEN-DOC-TENSE-SWEEP v2:** Gate passes (`python3 bin/check-green-doc-tense` → PASS).
- **cargo clippy --all-targets -- -D warnings:** PASS (zero warnings).
- **cargo fmt --check:** PASS (clean).

## Finalization Changes Made

1. Module header comment corrected: was listing all 10 tests as RED; corrected to 6 RED + 4 green-by-design with rationale for each group.
2. `test_large_forward_open_emits_finding` strengthened: added `summary.contains("CIP ForwardOpen connection establishment observed from src=")` assertion (BC-2.17.015 PC-1 / invariant 5 — LargeForwardOpen uses the same ForwardOpen summary prefix).
3. `cargo fmt` applied to satisfy `max_width = 100` rustfmt.toml requirement.
