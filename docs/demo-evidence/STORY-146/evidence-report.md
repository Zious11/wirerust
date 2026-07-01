# Demo Evidence Report — STORY-146

**Story:** TLS buffer-saturation telemetry (`buffer_saturation_drops` counter)  
**Story ID:** STORY-146  
**Recorded:** 2026-06-30  
**Toolchain:** VHS 0.11.0 · wirerust release binary · cargo test

---

## Coverage Map

| Acceptance Criteria | Demo Artifact | What It Shows |
|---------------------|--------------|---------------|
| AC-146-005 (summarize key present, value=0 when no saturation) | `AC-146-005-summarize-key-present.webm` / `.gif` | `wirerust --no-color analyze --tls tests/fixtures/tls12-aes256gcm.pcap` — terminal output includes `ANALYZER: TLS` section with `buffer_saturation_drops: 0`; without `--tls` the section is absent (error path). |
| AC-146-001 (`fill_buf_for_testing` seam), AC-146-002 (counter increments on tail-drop), AC-146-004 (accessor `buffer_saturation_drop_count`), AC-146-006 (both directions, same aggregate counter) | `AC-146-001-002-004-006-test-suite.webm` / `.gif` | `cargo test --test tls_analyzer_tests story_146` — all 8 tests pass, including `test_BC_2_07_043_buffer_saturation_observable`, `test_BC_2_07_043_buffer_saturation_full_drop`, `test_BC_2_07_043_both_directions_increment_same_counter`, `test_BC_2_07_043_counter_persists_across_flows`, and `test_BC_2_07_043_summarize_value_equals_drop_count`; misspelled module name shows 0 tests run (error path). |

---

## Artifacts

```
demos/story-146/
  AC-146-005-summarize-key-present.tape    — VHS script source
  AC-146-005-summarize-key-present.gif     — 379K GIF recording
  AC-146-005-summarize-key-present.webm    — 288K WebM recording

  AC-146-001-002-004-006-test-suite.tape   — VHS script source
  AC-146-001-002-004-006-test-suite.gif    — 219K GIF recording
  AC-146-001-002-004-006-test-suite.webm   — 256K WebM recording
```

---

## Notes on Coverage Approach

AC-146-002 and AC-146-006 require the per-direction TCP segment buffer to reach
MAX_BUF=65,536 bytes before a drop occurs. This threshold is not reachable with the
existing small pcap fixtures without engineering a synthetic 65K+ TLS capture.
The `fill_buf_for_testing` seam (AC-146-001) exists precisely to make this path
testable: it parks the buffer at capacity so a single subsequent `on_data` call
triggers a drop. The `story_146` test module exercises this seam directly, making
the test suite the correct evidence vehicle for AC-146-001/002/004/006.
AC-146-005 is demonstrated via the live CLI since `buffer_saturation_drops: 0`
is always present in `summarize()` output, even with no saturation events.

---

## AC Coverage Status

| AC | Covered | Vehicle |
|----|---------|---------|
| AC-146-001 | Yes | test suite (8 passing) |
| AC-146-002 | Yes | test suite (`test_BC_2_07_043_buffer_saturation_observable`, `_full_drop`) |
| AC-146-004 | Yes | test suite (`buffer_saturation_drop_count()` accessor calls) |
| AC-146-005 | Yes | live CLI on `tls12-aes256gcm.pcap` |
| AC-146-006 | Yes | test suite (`test_BC_2_07_043_both_directions_increment_same_counter`) |
