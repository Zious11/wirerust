---
document_type: story
story_id: STORY-146
title: "TLS Buffer Saturation Telemetry — `buffer_saturation_drops` Counter + `fill_buf_for_testing` Seam (BC-2.07.043 + amended BC-2.07.005)"
epic_id: E-5
wave: 66
points: 3
phase: f3
tdd_mode: strict
status: merged
feature_id: fix-tls-clienthello-frag
github_issue: fix-tls-clienthello-frag
subsystems: [SS-07]
target_module: analyzer/tls
depends_on: [STORY-144]
blocks: []
behavioral_contracts:
  - BC-2.07.043
  - BC-2.07.005
verification_properties:
  - VP-040
assumption_validations: []
risk_mitigations: []
# BC status: all BCs authored and anchored; status remains draft pending PO wave assignment
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.043.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.005.md
  - .factory/specs/architecture/decisions/ADR-011-tls-handshake-reassembly.md
  - .factory/cycles/fix-tls-clienthello-frag/delta-analysis.md
input-hash: "6134dfc"
---

# STORY-146: TLS Buffer Saturation Telemetry — `buffer_saturation_drops` Counter + `fill_buf_for_testing` Seam

## Narrative

**As a** security analyst or operator using wirerust's `summarize()` output,
**I want** the TLS analyzer to count and expose every tail-drop event that occurs when the
per-direction TCP-segment buffer (`client_buf` or `server_buf`) reaches its `MAX_BUF = 65,536`
byte capacity limit,
**so that** operators can distinguish "TLS flow had no relevant data" from "TLS flow was silently
truncated at the TCP-segment buffer layer" (F-EV-001 defense-in-depth — making the previously
silent tail-drop primitive non-silent regardless of future reachability changes).

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.07.043 | v1.3 | Per-Direction Buffer Saturation Tail-Drop Is Observable via `buffer_saturation_drops` Counter | Primary: new `buffer_saturation_drops: u64` aggregate counter; `fill_buf_for_testing` seam; `summarize()` surfacing; borrow-constraint placement; 6 VP-040 canonical Red-Gate tests + 2 EC-coverage tests (8 total) |
| BC-2.07.005 | v1.7 | Per-Direction Buffer Capped at MAX_BUF = 65536 Bytes (Tail-Drop Counted by BC-2.07.043) | Amended: Invariant 3 and Postcondition 4 updated to note `buffer_saturation_drops` increment; byte-drop semantics UNCHANGED; only telemetry added |

## Acceptance Criteria

### AC-146-001: `TlsAnalyzer` gains `buffer_saturation_drops: u64` aggregate counter; initialized to 0; never reset at flow close
**Traces to:** BC-2.07.043 v1.3 Invariants 1–3; ADR-011 Decision 1 (F2 scope-addition)

`TlsAnalyzer` gets one new field:
- `buffer_saturation_drops: u64` — initialized to `0` in `TlsAnalyzer::new()`; NOT on `TlsFlowState`; NOT reset by `on_flow_close`
- Mirrors `truncated_records: u64` at tls.rs:339 and `handshake_reassembly_overflows: u64` (STORY-144)

Accessor method added:
- `TlsAnalyzer::buffer_saturation_drop_count(&self) -> u64` — read-only accessor, same pattern as `parse_error_count()`, `truncated_record_count()`, `handshake_reassembly_overflow_count()`

Test seam added:
- `TlsAnalyzer::fill_buf_for_testing(&mut self, flow_key: &FlowKey, direction: Direction, n: usize)` — fills the per-direction buffer to exactly `n` bytes (precondition: `n <= MAX_BUF`); required to exercise BC-2.07.043 EC-002 (full-drop path: `remaining==0`) since that state is not reachable via the public `on_data` API alone. This is the ONLY accepted seam form (BC-2.07.043 Architecture Anchor). Signature uses `flow_key: &FlowKey` (by reference), matching the existing test-seam convention of all five sibling TLS test seams at tls.rs:957+.

(traces to BC-2.07.043 v1.3 Invariants 1–3; ADR-011 Decision 1)

### AC-146-002: `on_data` detects tail-drop condition; increments `buffer_saturation_drops` exactly once per drop-event call; placement after `&mut state` block closes
**Traces to:** BC-2.07.043 v1.3 Postconditions 1–2, Invariant 4; ADR-011 Decision 1 C-3 borrow constraint

The existing buffer-append block in `on_data` at tls.rs:1137-1161 currently performs a tail-drop silently when `data.len() > remaining` (where `remaining = MAX_BUF.saturating_sub(buf.len())`). This story adds observability:

**Detection pattern (required — borrow-constraint mandated):**
```rust
// INSIDE the per-direction match arm (while &mut state borrow is live):
let remaining = MAX_BUF.saturating_sub(buf.len());
let did_drop = data.len() > remaining;
// ... existing append logic unchanged ...

// AFTER the &mut state block closes (mutable borrow released):
if did_drop {
    // SEC-003 sibling-consistency: saturating_add avoids theoretical u64
    // overflow-check panic under overflow-checks=true; saturation at u64::MAX
    // is safe for an aggregate diagnostic.
    self.buffer_saturation_drops = self.buffer_saturation_drops.saturating_add(1);
}
```

The `did_drop` local bool flag MUST be set INSIDE the `&mut state` block; the `self.buffer_saturation_drops = self.buffer_saturation_drops.saturating_add(1)` increment MUST be placed AFTER the block closes — Rust borrow rules forbid mutating `self` while `state: &mut TlsFlowState` borrows from `self.flows`. This placement is between the buffer-append block and the `try_parse_records` call (ADR-011 Decision 1 / BC-2.07.043 Invariant 4).

The increment condition is `data.len() > remaining` — NOT `to_copy < data.len()`. The form `to_copy < data.len()` misses the full-drop path (`remaining==0`) because `to_copy` is only computed inside the `if remaining > 0` arm (BC-2.07.043 Precondition 3 NOTE; ADR-011 C-3 canonical form).

The counter increments ONCE per `on_data` call regardless of how many bytes are dropped (1 byte dropped or 65,536 bytes dropped = 1 event). Covers both `Direction::ClientToServer` and `Direction::ServerToClient` (same aggregate counter — BC-2.07.043 Postcondition 3).

`parse_errors` NOT incremented. `truncated_records` NOT incremented. No Finding pushed. BC-2.07.005 byte-drop semantics UNCHANGED — only the counter is added. (traces to BC-2.07.043 v1.3 Postconditions 1–6, Invariant 4)

**Red-Gate tests (VP-040 Sub-A):**
- `test_BC_2_07_043_buffer_saturation_observable` — partial-drop path (data 65,537 bytes to empty buffer; remaining=65,536; 1 byte dropped; drops_after==drops_before+1; parse_errors unchanged); no seam needed
- `test_BC_2_07_043_buffer_saturation_full_drop` — full-drop path (uses `fill_buf_for_testing` seam to park buffer at MAX_BUF; remaining==0; drops_after==drops_before+1)

### AC-146-003: Counter NOT incremented when data fits (no-drop boundary)
**Traces to:** BC-2.07.043 v1.3 Invariant 5; EC-005 (exact-fit case)

When `data.len() <= remaining` (no tail-drop), `buffer_saturation_drops` is unchanged. EC-005: `data.len() == remaining` (exact-fit, data fills remaining space exactly) does NOT trigger the drop counter — condition is strictly `>`.

(traces to BC-2.07.043 v1.3 Invariant 5; EC-003, EC-005)

**Red-Gate test (VP-040 Sub-B):** `test_BC_2_07_043_no_drop_no_counter` — small record fits; drops_after==drops_before

### AC-146-004: Counter persists across flow close (aggregate on `TlsAnalyzer`, not `TlsFlowState`)
**Traces to:** BC-2.07.043 v1.3 Postcondition 5; Invariant 2

After triggering a drop on a flow and then calling `on_flow_close`, `buffer_saturation_drop_count()` returns the accumulated value — NOT reset. This confirms the counter lives on `TlsAnalyzer`, not `TlsFlowState`.

**Assertion form (snapshot-delta):** the test MUST use a before/after snapshot comparison, NOT an absolute `==1` assertion. Specifically: (1) capture `drops_before = analyzer.buffer_saturation_drop_count()` before the drop event; (2) trigger a drop; (3) assert `drops_after_drop == drops_before + 1`; (4) call `on_flow_close`; (5) assert `drops_after_close == drops_before + 1` (counter is unchanged by flow close). This pattern is consistent with the VP-039 discipline used throughout STORY-144 and STORY-145 (e.g. `overflow_count == overflows_before + 1`). (traces to BC-2.07.043 v1.3 Postcondition 5, Invariant 2)

**Red-Gate test (VP-040 Sub-C):** `test_BC_2_07_043_counter_persists_across_flows` — snapshot `drops_before`; drop on flow; assert `drops_before + 1`; `on_flow_close`; assert still `drops_before + 1`

### AC-146-005: `summarize()` exposes `"buffer_saturation_drops"` with value-equality (not mere key presence)
**Traces to:** BC-2.07.043 v1.3 Postcondition 4; ADR-011 Decision 1

`TlsAnalyzer::summarize()` inserts `"buffer_saturation_drops"` into the `detail` `HashMap<String, Value>` with value equal to the u64 counter. The assertion is `detail["buffer_saturation_drops"].as_u64() == expected_count` — value-equality, not `contains_key`. The key is ALWAYS present, even when the count is 0 (EC-008 in BC-2.07.043). Mirrors `"truncated_records"` and `"handshake_reassembly_overflows"` surfacing pattern at tls.rs:1223-1226.

(traces to BC-2.07.043 v1.3 Postcondition 4)

**Red-Gate test (VP-040 Sub-D):** `test_BC_2_07_043_summarize_value_equals_drop_count` — trigger 1 drop; `summarize()` detail["buffer_saturation_drops"].as_u64()==1 (value-equality)

### AC-146-006: Both directions increment the SAME aggregate counter
**Traces to:** BC-2.07.043 v1.3 Postcondition 3; EC-007

One `ClientToServer` drop + one `ServerToClient` drop on (possibly separate) flows both increment `self.buffer_saturation_drops`. After two separate drops, `buffer_saturation_drop_count() == initial + 2`.

(traces to BC-2.07.043 v1.3 Postcondition 3; EC-007)

**Red-Gate test (VP-040 Sub-E):** `test_BC_2_07_043_both_directions_increment_same_counter` — C2S drop + S2C drop; drops_final == initial + 2

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `buffer_saturation_drops: u64` field + `buffer_saturation_drop_count()` accessor | `src/analyzer/tls.rs` | Effectful (aggregate counter) |
| `fill_buf_for_testing` seam | `src/analyzer/tls.rs` | Effectful (test-only buffer mutator) |
| `on_data` did_drop detection + post-block increment | `src/analyzer/tls.rs` | Effectful |
| `summarize()` `"buffer_saturation_drops"` key insertion | `src/analyzer/tls.rs` | Effectful |
| VP-040 unit tests | `tests/tls_analyzer_tests.rs` | Pure |

Architecture compliance: SS-07 only. No other files change. This story is deliberately narrow: it adds telemetry to the pre-existing BC-2.07.005 tail-drop path WITHOUT changing drop semantics.

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-C1 | BC-2.07.043 EC-001 | Buffer at 65,535; `data` 2 bytes | 1 byte dropped; `buffer_saturation_drops.saturating_add(1)` (SEC-003); buf.len()==65,536; parse_errors unchanged |
| EC-C2 | BC-2.07.043 EC-002 | Buffer at 65,536 (full); `data` 1,000 bytes | Full-drop: `buffer_saturation_drops.saturating_add(1)` (SEC-003); 0 bytes appended; requires `fill_buf_for_testing` seam to reach this state in tests |
| EC-C3 | BC-2.07.043 EC-003 | Buffer at 65,536; `data` 0 bytes | `data.len()==0`, `remaining==0`; `0 > 0` is false; counter NOT incremented |
| EC-C4 | BC-2.07.043 EC-005 | Buffer at 0; `data` exactly 65,536 bytes | `data.len()==remaining==65,536`; `65,536 > 65,536` is false; no drop; counter NOT incremented |
| EC-C5 | BC-2.07.043 EC-006 | Two consecutive `on_data` calls each trigger a tail-drop | counter incremented twice; final value == initial + 2 |
| EC-C6 | BC-2.07.043 EC-008 | `summarize()` called when `buffer_saturation_drops==0` | `"buffer_saturation_drops": 0` in detail map; key ALWAYS present |

## Estimated Complexity

**Story points: 3** (the existing tail-drop is already implemented; this story adds one counter field, one accessor, one test seam, one `did_drop` detection flag, one post-block increment, and one `summarize()` key insertion; the 6 canonical VP-040 Red-Gate tests are deterministic unit tests with fixed fixtures; 2 additional EC-coverage tests pin EC-C1/EC-C3; 8 tests total)

## Token Budget Estimate

| Context source | Estimated tokens |
|---------------|-----------------|
| This story spec | ~1,800 |
| BC files (2 BCs: 043/005) | ~4,000 |
| ADR-011 (F2 scope-addition section) | ~2,000 |
| `src/analyzer/tls.rs` (with STORY-144+145 changes) | ~26,000 |
| `tests/tls_analyzer_tests.rs` (with STORY-144+145 additions) | ~28,000 |
| VP-040 (harness skeletons) | ~3,000 |
| Tool outputs | ~1,000 |
| **Total estimate** | **~65,800** |

Fits within a 200k context window (~33%). The implementer should read the existing `on_data` buffer-append block at tls.rs:1137-1161 carefully before adding the `did_drop` flag — the borrow constraint pattern is not immediately obvious from the code structure.

## Tasks

1. **Write Red-Gate tests first (TDD Step 1 — all must FAIL before implementation)**
   - `test_BC_2_07_043_buffer_saturation_observable`
   - `test_BC_2_07_043_buffer_saturation_full_drop`
   - `test_BC_2_07_043_no_drop_no_counter`
   - `test_BC_2_07_043_counter_persists_across_flows`
   - `test_BC_2_07_043_summarize_value_equals_drop_count`
   - `test_BC_2_07_043_both_directions_increment_same_counter`
   - All 6 fail before any counter field or seam exists; `fill_buf_for_testing` call will not compile yet

2. **Add `buffer_saturation_drops: u64` to `TlsAnalyzer` struct + init + accessor + seam (AC-146-001)**
   - Add field to struct; init to 0 in `TlsAnalyzer::new()`
   - Add `buffer_saturation_drop_count(&self) -> u64` accessor
   - Add `fill_buf_for_testing(&mut self, flow_key: &FlowKey, direction: Direction, n: usize)` seam
   - Verify: tests compile; all still FAIL (field exists but increment not wired yet)

3. **Wire `did_drop` detection + post-block increment in `on_data` (AC-146-002)**
   - Inside both `Direction::ClientToServer` and `Direction::ServerToClient` buffer-append arms: add `let did_drop = data.len() > remaining;`
   - AFTER the `&mut state` block closes: add `if did_drop { self.buffer_saturation_drops = self.buffer_saturation_drops.saturating_add(1); }` (SEC-003 sibling-consistency: saturating_add avoids theoretical u64 overflow-check panic under overflow-checks=true; saturation at u64::MAX is safe for an aggregate diagnostic)
   - Critical: the increment MUST be between the buffer-append block and the `try_parse_records` call (ADR-011 Decision 1 C-3 borrow constraint)
   - Verify: `test_BC_2_07_043_buffer_saturation_observable` and `test_BC_2_07_043_both_directions_increment_same_counter` turn GREEN

4. **Wire `summarize()` insertion (AC-146-005)**
   - Add `detail.insert("buffer_saturation_drops", Value::from(self.buffer_saturation_drops))` after the existing `truncated_records` insert at tls.rs:1223-1226
   - Verify: `test_BC_2_07_043_summarize_value_equals_drop_count` turns GREEN

5. **Full regression sweep**
   - `cargo test --all-targets` — ALL tests GREEN (STORY-144 + STORY-145 + STORY-146 combined)
   - `cargo clippy --all-targets -- -D warnings` — zero warnings
   - `cargo fmt --check` — clean

6. **Micro-commit and PR** targeting `develop` (wave 66, parallel with STORY-145)

## Previous Story Intelligence

**From STORY-144:** `handshake_reassembly_overflows: u64` was added to `TlsAnalyzer` using the same `truncated_records` pattern. The `summarize()` insertion pattern is already established. The `buffer_saturation_drops` counter follows the identical pattern.

**From STORY-144 borrow-constraint lesson:** The `handshake_reassembly_overflows += 1` increment fires INSIDE the carry overflow check branch (before the borrow), so it has no borrow conflict. In contrast, `buffer_saturation_drops = buffer_saturation_drops.saturating_add(1)` fires AFTER detecting a drop inside the `&mut state` block — so it requires the `did_drop: bool` flag pattern exactly as documented in ADR-011 Decision 1 and VP-040 Property Statement. The `saturating_add` form (rather than `+= 1`) matches the SEC-003 sibling-consistency pattern used by the three `handshake_reassembly_overflows` increments.

**From BC-2.07.043 v1.3:** The seam signature is `fill_buf_for_testing(&mut self, flow_key: &FlowKey, direction: Direction, n: usize)` with `flow_key: &FlowKey` by REFERENCE (not by value). This matches the `&FlowKey` convention of all five sibling TLS test seams. Do not use by-value `FlowKey` — that would force a clone at call sites.

**Dependency rationale (I6):** STORY-146 depends on STORY-144, NOT STORY-145. The `buffer_saturation_drops` telemetry touches the `on_data` TCP-segment buffer tail-drop path (`client_buf`/`server_buf` at tls.rs:1137-1161) and the `summarize()` counter-pattern established in STORY-144 (`handshake_reassembly_overflows`). It does NOT consume the ServerHello drain path added by STORY-145. STORY-146 and STORY-145 are therefore parallel (wave 66), both depending on STORY-144 (wave 65). Scheduling them in parallel saves one wave vs. the original linear STORY-145 → STORY-146 chain.

## Architecture Compliance Rules

Source: `architecture/module-decomposition.md` + ADR-011

1. **`buffer_saturation_drops` is on `TlsAnalyzer`, NOT `TlsFlowState`:** same as `truncated_records` and `handshake_reassembly_overflows`. Any misplacement on `TlsFlowState` would lose the count when a flow closes (BC-2.07.043 Invariant 2).
2. **Increment condition is `data.len() > remaining`:** NOT `to_copy < data.len()`. The latter form misses the `remaining==0` full-drop case where `to_copy` is never computed (ADR-011 C-3).
3. **`did_drop` flag pattern is REQUIRED by the borrow checker:** `self.buffer_saturation_drops = self.buffer_saturation_drops.saturating_add(1)` cannot appear inside the `&mut state` block. Violating this produces a compile error. The `saturating_add` form (not plain `+=`) matches the SEC-003 sibling-consistency pattern; both `buffer_saturation_drops` and all three `handshake_reassembly_overflows` increments must use `saturating_add`.
4. **Byte-drop semantics of BC-2.07.005 are UNCHANGED:** This story adds telemetry only. The actual discard of bytes (`to_copy` vs `data.len()`) must not be modified.
5. **`"buffer_saturation_drops": 0` in `summarize()` when no drops:** the key must always be present (EC-008), not only when the count is non-zero. This mirrors `"parse_errors": 0` and `"truncated_records": 0` always appearing.
6. **Test namespace isolation (DF-TEST-NAMESPACE-001):** ALL 8 test functions delivered by STORY-146 (6 canonical VP-040 Red-Gate + 2 EC-coverage tests) MUST be placed inside a dedicated `mod story_146 { ... }` wrapper in `tests/tls_analyzer_tests.rs`. No new test function from this story may be added at the flat module root.

## Library & Framework Requirements

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `serde_json` | (existing) | `Value::from(self.buffer_saturation_drops)` in `summarize()` detail map |

No new dependencies.

**Forbidden dependencies:** no new crates; no new module dependencies.

## File Structure Requirements

| File | Change Type | Purpose |
|------|------------|---------|
| `src/analyzer/tls.rs` | Modify | Add `buffer_saturation_drops` field + accessor + seam + `did_drop` detection + `summarize()` key |
| `tests/tls_analyzer_tests.rs` | Modify | Add VP-040 6 canonical Red-Gate tests + 2 EC-coverage tests (8 total), all inside `mod story_146 { ... }` per DF-TEST-NAMESPACE-001 |

No new files.

## Test Helper / Seam Ownership (STORY-146 creates or reuses these)

The VP-040 harnesses require the following helpers and seams. The implementer MUST create or reconcile them in this story. This table is the **complete contract** — every harness symbol is accounted for.

**tls.rs seams (new `#[doc(hidden)] pub fn` on `TlsAnalyzer`, all created in AC-146-001):**

| Helper / Seam | Form | Notes |
|---------------|------|-------|
| `TlsAnalyzer::fill_buf_for_testing(&mut self, flow_key: &FlowKey, direction: Direction, n: usize)` | NEW test seam in `src/analyzer/tls.rs` | Created in AC-146-001. Fills the per-direction TCP-segment buffer (`client_buf` or `server_buf`) to exactly `n` bytes; precondition `n <= MAX_BUF`. Required to exercise the full-drop path (EC-002: `remaining==0`) since that state is not reachable via the public `on_data` API alone without first filling the buffer. Signature uses `flow_key: &FlowKey` by reference, matching the convention of all five sibling TLS seams at tls.rs:957+. |
| `TlsAnalyzer::buffer_saturation_drop_count(&self) -> u64` | NEW accessor in `src/analyzer/tls.rs` | Created in AC-146-001. Read-only observer of the `buffer_saturation_drops` aggregate counter on `TlsAnalyzer`. Follows the same pattern as `parse_error_count()`, `truncated_record_count()`, `handshake_reassembly_overflow_count()`. |

**VP-040 pseudo-helper mapping (VP-040 skeleton → concrete symbols):**

The VP-040 skeleton was authored with placeholder pseudo-helpers; this table maps each to the concrete symbol the implementer MUST use:

| VP-040 skeleton pseudo-helper | Concrete symbol | Notes |
|-------------------------------|-----------------|-------|
| `fill_to_capacity(flow_key)` | `analyzer.fill_buf_for_testing(&flow_key, direction, MAX_BUF)` | Sets buffer to exactly MAX_BUF bytes via the `fill_buf_for_testing` seam. |
| `deliver_one_more_byte(flow_key)` | `analyzer.on_data(&flow_key, direction, &[0x00])` | Standard `on_data` call with 1 byte after buffer is full; triggers full-drop path. |
| `FlowKey::new(...)` placeholder | `make_test_flow_key(seed: u8)` (see below) | The VP-040 skeleton `FlowKey::new` placeholder maps to the concrete `make_test_flow_key` local helper. |

**Local test helpers (defined inside `mod story_146 { ... }` in `tests/tls_analyzer_tests.rs`):**

| Helper / Seam | Form | Notes |
|---------------|------|-------|
| `make_test_flow_key(seed: u8) -> FlowKey` | LOCAL helper in `mod story_146` | Re-declared identical copy from `mod story_144` / `mod story_145`. Varies `src_port` or address octets by `seed` so multi-flow tests can create distinct `FlowKey`s. Body is 2–3 lines; re-declaration cost is negligible. |
| `all_findings_len_for_testing` | EXISTING tls.rs seam | The existing findings-count seam name — do NOT rename to `findings_count_for_testing`. |

**Reconciliation rule:** Before creating any new helper, `grep` the existing `tests/tls_analyzer_tests.rs` for the relevant name pattern. Use the real existing name if found at crate scope; re-declare locally only for anything private to `mod story_144` / `mod story_145`.

## Red-Gate Test Set (VP-040 — 6 canonical + 2 EC-coverage = 8 total)

### Canonical VP-040 Red-Gate Tests (6)

All 6 test names below are CANONICAL per VP-040 and BC-2.07.043 VP table. Must appear verbatim (DF-AC-TEST-NAME-SYNC-001). These are the AC-cited tests — do NOT re-map ACs to the EC-coverage tests below.

| Test name | Sub | BC Postcondition | Path |
|-----------|-----|-----------------|------|
| `test_BC_2_07_043_buffer_saturation_observable` | Sub-A | PC-1 (partial drop: remaining>0, 1 byte dropped) | No seam |
| `test_BC_2_07_043_buffer_saturation_full_drop` | Sub-A | PC-1 (full drop: remaining==0) | `fill_buf_for_testing` seam |
| `test_BC_2_07_043_no_drop_no_counter` | Sub-B | PC-1 negative (no drop) | No seam |
| `test_BC_2_07_043_counter_persists_across_flows` | Sub-C | PC-5 (persists across on_flow_close) | No seam |
| `test_BC_2_07_043_summarize_value_equals_drop_count` | Sub-D | PC-4 (summarize value-equality) | Uses Sub-A fixture |
| `test_BC_2_07_043_both_directions_increment_same_counter` | Sub-E | PC-3 (both directions, same counter) | `fill_buf_for_testing` for S2C arm |

### Additional EC-Table Coverage Tests (2, F-146-02)

These two tests were added in commit 6a57eaa (F-146-02) to pin the story's EC table entries EC-C1 and EC-C3. They are ADDITIONAL coverage — they do NOT replace or re-map any ACs. They live inside `mod story_146` per DF-TEST-NAMESPACE-001.

| Test name | EC | Description | Seam |
|-----------|-----|-------------|------|
| `test_BC_2_07_043_partial_drop_boundary` | EC-C1 | Buffer at 65,535 + 2 bytes → drops_before+1, parse_errors unchanged. Pins the partial-drop boundary (`remaining==1`): catches a missing or broken partial-drop detection (e.g., guard removed, condition inverted). Does NOT kill the `to_copy < data.len()` mutant — at `remaining==1`, `to_copy=1 < 2` is true so that mutant also increments; the `to_copy < data.len()` mutant is killed by the full-drop tests (`remaining==0`: `test_BC_2_07_043_buffer_saturation_full_drop` and `test_BC_2_07_043_full_buffer_empty_data_no_count`). | `fill_buf_for_testing(n = MAX_BUF - 1)` |
| `test_BC_2_07_043_full_buffer_empty_data_no_count` | EC-C3 | Buffer at MAX_BUF + 0 bytes → no increment. Fails under a `>=` mutation of the strict `>` predicate. | `fill_buf_for_testing(n = MAX_BUF)` |

## Holdout Scenarios (F4)

STORY-146 specifically gates on:
- **HS-F4-010**: buffer-saturation holdout — `fill_buf_for_testing` + 1,000 byte delivery → `buffer_saturation_drops==1`, `parse_errors==0`, `summarize()["buffer_saturation_drops"]==1`
- **HS-F4-011**: zero-drop holdout — fresh flow, data well within MAX_BUF → `buffer_saturation_drops==0` always present in `summarize()` detail map
- **HS-F4-012**: cross-direction aggregate holdout — one C2S drop + one S2C drop → `buffer_saturation_drops==2`

## Revision History

| Version | Date | Change | Finding IDs |
|---------|------|--------|-------------|
| v1.0 | (initial) | Story authored per BC-2.07.043 v1.3 decomposition | — |
| v1.1 | 2026-06-30 | F-146-01: Change `buffer_saturation_drops += 1` to `buffer_saturation_drops.saturating_add(1)` in all AC bodies, Tasks, Architecture Compliance Rules, and prose (SEC-003 sibling-consistency — matches `handshake_reassembly_overflows` siblings; avoids theoretical overflow-check panic under `overflow-checks = true`). F-146-02: Document 2 additional EC-coverage tests added in commit 6a57eaa (`test_BC_2_07_043_partial_drop_boundary` pins EC-C1; `test_BC_2_07_043_full_buffer_empty_data_no_count` pins EC-C3); test count updated from 6 to 8 throughout. No AC-to-BC traces changed; no `inputs:` list changed; input-hash unchanged. | F-146-01, F-146-02 |
