# Evidence Report — STORY-181

**Story:** STORY-181: Fix SEC-001 ENIP Unsafe Split-Borrow in on_data: Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop (Behavior-Preserving Refactor)  
**Wave:** 85  
**Date:** 2026-07-24  
**Branch:** feature/STORY-181-enip-sec001-split-borrow  
**Product type:** Library (behavior-preserving refactor — no CLI/web surface; internal-only change to `on_data` PDU dispatch loop)

---

## ENIP Test Suite: 184/184 PASS

Command:
```
cargo test --test enip_analyzer_tests
```

Output (tail):
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/enip_analyzer_tests.rs (target/debug/deps/enip_analyzer_tests-831d4c1100defc5d)

test result: ok. 184 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

---

## Coverage Map

| AC | Description | Evidence File | Verdict |
|----|-------------|---------------|---------|
| AC-181-001 | Unsafe `*mut EnipFlowState` split-borrow eliminated; take-remove-reinsert in place | `AC-181-001-unsafe-eliminated.md` | PASS |
| AC-181-002 | All 184 ENIP tests pass; 3 carry-path regression witnesses confirmed | `AC-181-002-behavior-identical.md` | PASS |
| AC-181-003 | `process_pdu` signature unchanged; `git diff --stat` shows only enip.rs/bin/CHANGELOG; no Cargo.toml | `AC-181-003-no-api-change.md` | PASS |
| AC-181-004 | `parse_line()` docstring updated; 27/27 `test_validate_citations.py` pass | `AC-181-004-bin-docstring.md` | PASS |

---

## AC-181-001 Summary

`grep -n "flow_ptr\|ptr_as_ptr\|\*mut EnipFlowState\|unsafe" src/analyzer/enip.rs` returns
**zero matches**. The four SEC-001 symbols are absent from the file.

Before (commit 421bf572, lines 985–1000): `let flow_ptr: *mut EnipFlowState = self.flows.get_mut(...)` 
with `self.process_pdu(unsafe { &mut *flow_ptr }, &pdu, ...)` and `#[allow(clippy::ptr_as_ptr)]`.

After (HEAD, lines 978–1001): `self.flows.remove(&flow_key)` → local owned `flow` →
`self.process_pdu(&mut flow, &pdu, ...)` → `self.flows.insert(flow_key, flow)`.
Compiler-enforced disjointness. No unsafe, no raw pointer, no clippy allow.

---

## AC-181-002 Summary

184 ENIP tests pass (0 failures). Three BC-2.17.016 carry-path regression witnesses confirmed:
- `frame_walk::test_carry_buffer_partial_header` — ok
- `frame_walk::test_carry_buffer_two_frames_one_segment` — ok
- `direction_and_clock::test_ec_x1_cross_direction_no_splice` — ok

Full suite (`cargo test --all-targets`): all test-result lines read `0 failed`.

---

## AC-181-003 Summary

`git diff 421bf572..HEAD --stat` output:
```
 CHANGELOG.md           | 23 +++++++++++++++++++++++
 bin/validate-citations |  8 ++++----
 src/analyzer/enip.rs   | 45 ++++++++++++++++++++++++---------------------
 3 files changed, 51 insertions(+), 25 deletions(-)
```

`Cargo.toml` absent. No new dependencies. No public API change.

---

## AC-181-004 Summary

`parse_line()` at `bin/validate-citations` line 116 now documents all three return cases
including `None` when the line fails the citation regex (caller treats as MALFORMED).
`python3 bin/test_validate_citations.py`: 27 passed, 0 failed.

---

## End-to-End CLI Run (Optional)

No ENIP-specific pcap fixture exists in `tests/fixtures/`. No CLI demo was built for
this refactor story per the task specification ("only if a fixture already exists").

---

## Recording Method

This is a behavior-preserving refactor story (no new behavioral surface). Evidence is
captured as annotated CLI transcript markdown files showing real command output for each AC:
- Source-level grep verification (AC-181-001 unsafe elimination)
- `cargo test` run output (AC-181-002 behavior identity)
- `git diff --stat` + signature grep (AC-181-003 no API change)
- Docstring excerpt + `test_validate_citations.py` output (AC-181-004 bin housekeeping)

VHS/Playwright recordings are not applicable at this story scope.

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-181-001-unsafe-eliminated.md` | AC-181-001: grep zero-match + before/after code excerpt |
| `AC-181-002-behavior-identical.md` | AC-181-002: 184/184 enip tests + 3 carry-path witnesses + full suite summary |
| `AC-181-003-no-api-change.md` | AC-181-003: process_pdu signature grep + git diff stat |
| `AC-181-004-bin-docstring.md` | AC-181-004: docstring excerpt + 27/27 test_validate_citations.py |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

Command run before commit (PG-W70-DEMO-SCRUB canonical pattern):
```
grep -rE '<host-path-pattern>' docs/demo-evidence/STORY-181/
```

Result: **zero matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-24).
