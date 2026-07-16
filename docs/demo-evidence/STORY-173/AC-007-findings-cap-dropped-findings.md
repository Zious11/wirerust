# AC-173-007: IEC-104 Findings Cap + dropped_findings Surfaced in summarize()

**AC:** AC-173-007
**BC:** BC-2.19.028 postconditions 1–5, invariant 4 (DoS bound — IEC104-FINDINGS-CAP-001)
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

`Iec104Analyzer` gains a DoS-protection findings cap of 10,000 entries:

- `const MAX_IEC104_FINDINGS: usize = 10_000` added to `src/analyzer/iec104.rs`.
- `dropped_findings: u64` field added to `Iec104Analyzer`, initialized to 0.
- Cap enforced at the `on_data` extend step: when `local_findings` would push
  `all_findings` past `MAX_IEC104_FINDINGS`, the excess findings are silently
  discarded (no `Finding` emitted) and `dropped_findings` is incremented by the
  discarded count.
- Per-flow state (`Iec104FlowState` carry buffers, dedup flags, `ns_expected`,
  `session_started`) continues to update regardless of the cap.
- `summarize()` includes `"dropped_findings"` in its detail map.

This mirrors the `MAX_FINDINGS` / `dropped_findings` pattern from DNP3 (BC-2.15.022) and
EtherNet/IP (BC-2.17.022).

---

## Source confirmation

`src/analyzer/iec104.rs` — constant (line 181):
```rust
pub const MAX_IEC104_FINDINGS: usize = 10_000;
```

`src/analyzer/iec104.rs` — `dropped_findings` field (line 1070):
```rust
/// Count of findings silently dropped because `all_findings` reached `MAX_IEC104_FINDINGS`.
/// (BC-2.19.028 PC-3/PC-4; IEC104-FINDINGS-CAP-001; STORY-173).
/// Surfaced in `summarize()` as `detail["dropped_findings"]`.
pub dropped_findings: u64,
```

`src/analyzer/iec104.rs` — cap enforcement at extend step (lines 1282–1290):
```rust
// BC-2.19.028 PC-2 / IEC104-FINDINGS-CAP-001: cap at MAX_IEC104_FINDINGS.
let remaining_cap = MAX_IEC104_FINDINGS.saturating_sub(self.all_findings.len());
    self.dropped_findings = self
        .dropped_findings
        ...
```

`src/analyzer/iec104.rs` — `summarize()` exposes the counter (lines 1313–1314):
```rust
"dropped_findings".to_string(),
serde_json::Value::Number(self.dropped_findings.into()),
```

---

## Test output

Command:
```
cargo test --test iec104_analyzer_tests story_173
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs

running 4 tests
test story_173::test_BC_2_19_028_findings_cap ... ok
test story_173::test_BC_2_19_028_boundary_at_max_minus_one_allows_one_more ... ok
test story_173::test_BC_2_19_028_dropped_findings_surfaced_in_summarize ... ok
test story_173::test_BC_2_19_028_cap_maintained_across_multiple_on_data_calls ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 192 filtered out; finished in 0.00s
```

---

## Per-test behavior

**`test_BC_2_19_028_findings_cap` (BC-2.19.028 PC-2 + PC-5 — cap fires, dropped > 0):**
1. Pre-fills `analyzer.all_findings` with `MAX_IEC104_FINDINGS` dummy findings.
2. Calls `analyzer.on_data(fk, &stopdt_act(), 0, Direction::ClientToServer)` once.
   STOPDT-act with `session_started=false` → `detect_iec104_threats` would produce 1 T0881 finding.
3. Asserts `analyzer.all_findings.len() <= MAX_IEC104_FINDINGS` — cap enforced, len stays at 10,000.
4. Asserts `analyzer.dropped_findings > 0` — suppressed finding counted.

**`test_BC_2_19_028_boundary_at_max_minus_one_allows_one_more` (EC-001 — boundary guard):**
1. Pre-fills with `MAX_IEC104_FINDINGS - 1` findings.
2. Feeds one STOPDT-act (produces 1 finding); total reaches exactly `MAX_IEC104_FINDINGS`.
3. Asserts `all_findings.len() == MAX_IEC104_FINDINGS` and `dropped_findings == 0` — no cap
   truncation needed at MAX-1.

**`test_BC_2_19_028_cap_maintained_across_multiple_on_data_calls` (EC-004 — N sequential calls):**
1. Pre-fills to `MAX_IEC104_FINDINGS` (10,000).
2. Calls `on_data` five more times on distinct port-2404 flows, each producing a T0881 finding.
3. Asserts `all_findings.len() <= MAX_IEC104_FINDINGS` — cap maintained across all 5 calls.
4. Asserts `dropped_findings == 5` — one dropped per call.

**`test_BC_2_19_028_dropped_findings_surfaced_in_summarize` (BC-2.19.028 PC-5 + F-173-001):**
1. Pre-fills to cap; calls `on_data` with a STOPDT-act (finding suppressed → `dropped_findings == 1`).
2. Calls `analyzer.summarize()`.
3. Asserts `summary.detail["dropped_findings"]` is a JSON number > 0.

---

## Cap-drop path illustrated

```
Initial state: all_findings.len() = 10,000 (at MAX)
on_data call:  STOPDT-act produces 1 T0881 finding
Cap logic:     remaining_cap = 10,000 - 10,000 = 0
               local_findings truncated to 0 entries
               dropped_findings += 1  (now 1)
               all_findings not extended
Final state:   all_findings.len() = 10,000 (unchanged)
               dropped_findings = 1
```

---

## Verdict

PASS — Cap fires at `MAX_IEC104_FINDINGS`, `dropped_findings` counts suppressed findings,
boundary condition is safe (no off-by-one), cap maintains invariant across N calls, and
`summarize()` surfaces `dropped_findings` in the detail map.
