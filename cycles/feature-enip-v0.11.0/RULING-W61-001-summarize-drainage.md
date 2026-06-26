---
ruling_id: RULING-W61-001
finding_id: F-138-P1-004
finding_class: functional gap
status: binding
issued_by: architect
issued_date: 2026-06-26
wave: 61
feature_cycle: feature-enip-v0.11.0
release: v0.11.0
affects:
  - src/analyzer/enip.rs
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.021.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.017.md
block_release: true
---

# RULING-W61-001: Summarize Drainage — Fix Approach for F-138-P1-004

## Finding Summary

`EnipAnalyzer::summarize()` reads four aggregate fields (`total_pdu_count`, `parse_errors`,
`command_distribution`, `flows_analyzed`) that are populated exclusively by
`on_flow_close()`. The dispatcher's ENIP flow-close arm is a documented no-op:

```rust
// src/dispatcher.rs ~L414
Some(DispatchTarget::Enip) => {
    // EnipAnalyzer does not implement StreamHandler; no forwarding needed.
    let _ = reason;
}
```

`main.rs` calls `enip.summarize()` after `take_enip_analyzer()` with no prior drain.
For any real capture where TCP sessions are not explicitly torn down (the common pcap
case), `on_flow_close` is never called, so all four aggregate fields remain at their
zero-initialized values. `write_count` and `error_count` are unaffected because they
are incremented directly in `process_pdu` / `on_data`, not via `on_flow_close`. The
BC-2.17.021 canonical test vector ("3 ListIdentity, 1 flow → flows_analyzed:1,
total_pdu_count:3") is not met in production.

This is a confirmed functional defect in v0.11.0.

---

## Decision: Fix Approach (a) — Summarize-Time Fold of Open Flows

**Chosen approach: (a) — mirror the DNP3 sibling pattern.**

`summarize()` is amended to fold still-open `self.flows.values()` into a transient
combined view ON TOP OF the pre-accumulated aggregate counters at call time. The
`self.flows` map is not mutated; no flow is removed; `flows_analyzed` counts the live
flows by adding `self.flows.len()`.

### Rationale

**Why not (b) — dispatcher drain:**

Option (b) requires threading `on_flow_close` calls through the dispatcher for every
open flow at capture end and for every mid-capture TCP teardown. Two hard obstacles:

1. **Signature mismatch.** `EnipAnalyzer::on_flow_close(&mut self, flow_key: FlowKey)`
   takes `flow_key` by value with no `reason` parameter. The dispatcher's
   `on_flow_close(flow_key, reason)` call signature passes a `CloseReason`. Bridging
   this requires either changing the ENIP signature (BC-2.17.017 anchor edit) or
   writing a throw-away adapter — both carry blast radius beyond a no-production-code
   ruling.

2. **End-of-capture enumeration.** Draining open flows at capture end requires
   collecting all live FlowKeys from `self.flows` and calling `on_flow_close` for each.
   `HashMap` does not support removal-while-iterating; a collect-then-drain loop is
   needed. That is new production logic in `dispatcher.rs` or `main.rs` — more invasive
   than option (a).

3. **The sibling (DNP3) chose (a) deliberately.** `Dnp3Analyzer::summarize()` reads
   `self.flows.len()` and `self.flows.values()` directly — it has no `flows_analyzed`
   aggregate field at all and does not depend on `on_flow_close` having fired. The
   dispatcher's DNP3 flow-close arm is identically a no-op:

   ```rust
   Some(DispatchTarget::Dnp3) => {
       // Dnp3Analyzer does not implement StreamHandler; no forwarding needed.
       let _ = reason;
   }
   ```

   ENIP was designed to mirror DNP3 (ADR-010 Decision 4, "mirrors the DNP3 pattern").
   Deviating to option (b) for ENIP alone would create an asymmetry without justification.

4. **The common pcap case is flows-never-closed.** In offline pcap analysis, TCP FIN/RST
   events may be absent (truncated captures, one-sided captures, etc.). Option (b) fixes
   the explicit-teardown path but does nothing for flows that simply end when the pcap
   ends. Option (a) handles both cases identically.

**Why option (a) is safe (no double-count):**

`on_flow_close` calls `self.flows.remove(&flow_key)`. Once a flow is removed from
`self.flows`, it is no longer in the map. A flow cannot be both in `self.flows` AND have
already contributed to the aggregate counters via `on_flow_close` — removal is the gate.
Therefore folding `self.flows.values()` at summarize time can never double-count a
closed flow. The only flows visible in `self.flows` at summarize time are flows that were
never passed to `on_flow_close` — exactly the population we need to include.

---

## Exact Fix Logic

The change is confined to `EnipAnalyzer::summarize()` in `src/analyzer/enip.rs`.
No changes to `dispatcher.rs`, `main.rs`, BC-2.17.017, or `on_flow_close`.

**Current behavior (broken):**

```rust
pub fn summarize(&self) -> AnalysisSummary {
    // reads self.command_distribution, self.total_pdu_count,
    // self.parse_errors, self.flows_analyzed directly
    // ...
    enip_summary.insert("flows_analyzed", serde_json::json!(self.flows_analyzed));
    enip_summary.insert("total_pdu_count", serde_json::json!(self.total_pdu_count));
    enip_summary.insert("parse_errors", serde_json::json!(self.parse_errors));
    // command_distribution built from self.command_distribution only
}
```

**Fixed behavior:**

```rust
pub fn summarize(&self) -> AnalysisSummary {
    // Start from the pre-accumulated aggregates (covers flows that were
    // explicitly closed via on_flow_close, if any).
    let mut total_pdu_count = self.total_pdu_count;
    let mut parse_errors = self.parse_errors;
    let mut open_flow_count: u64 = 0;

    // Build a combined command distribution: start with the closed-flow aggregate,
    // then fold in still-open flows. Use a local map to avoid mutating self.
    let mut cmd_dist_combined: HashMap<u16, u64> = self.command_distribution.clone();

    // BC-2.17.021 Precondition 4: fold still-open flows at summarize time.
    // self.flows only contains flows NOT yet passed to on_flow_close.
    // No double-count is possible: on_flow_close removes the flow from self.flows
    // before folding; a closed flow cannot appear here.
    for flow in self.flows.values() {
        total_pdu_count = total_pdu_count.saturating_add(flow.pdu_count);
        parse_errors = parse_errors.saturating_add(flow.parse_errors);
        for (&cmd, &count) in &flow.command_counts {
            let e = cmd_dist_combined.entry(cmd).or_insert(0);
            *e = e.saturating_add(count);
        }
        open_flow_count = open_flow_count.saturating_add(1);
    }

    // flows_analyzed = closed flows (on_flow_close increments) + still-open flows
    let flows_analyzed = self.flows_analyzed.saturating_add(open_flow_count);

    // Build the command_distribution JSON map from the combined view.
    let mut cmd_dist_json: serde_json::Map<String, serde_json::Value> =
        serde_json::Map::new();
    for (&cmd, &count) in &cmd_dist_combined {
        if count > 0 {
            cmd_dist_json.insert(format!("0x{cmd:04X}"), serde_json::json!(count));
        }
    }

    // Build enip_summary from combined totals.
    let mut enip_summary: serde_json::Map<String, serde_json::Value> =
        serde_json::Map::new();
    enip_summary.insert("command_distribution", serde_json::Value::Object(cmd_dist_json));
    enip_summary.insert("total_pdu_count", serde_json::json!(total_pdu_count));
    enip_summary.insert("parse_errors", serde_json::json!(parse_errors));
    enip_summary.insert("write_count", serde_json::json!(self.write_count));
    enip_summary.insert("error_count", serde_json::json!(self.error_count));
    enip_summary.insert("flows_analyzed", serde_json::json!(flows_analyzed));
    enip_summary.insert("dropped_findings", serde_json::json!(self.dropped_findings));

    // packets_analyzed mirrors total_pdu_count (combined).
    AnalysisSummary {
        analyzer_name: "EtherNet/IP".to_string(),
        packets_analyzed: total_pdu_count,
        detail: { let mut d = BTreeMap::new();
                  d.insert("enip_summary".to_string(),
                           serde_json::Value::Object(enip_summary));
                  d },
    }
}
```

**Implementation notes for the implementer:**

- The `HashMap` clone of `self.command_distribution` is the only allocation added to
  the hot path. For the pcap-end summarize call (called once per run), this is
  unconditionally acceptable.
- `write_count` and `error_count` are NOT folded from `self.flows.values()` here
  because they are already incremented directly into `self.write_count` /
  `self.error_count` inside `process_pdu` (they are per-event aggregates, not
  per-flow-close aggregates). Folding them again from flow state would double-count
  them. Verify: `EnipFlowState` has `write_count_in_window` (windowed, not lifetime)
  and `error_counts_in_window` (windowed, not lifetime); neither is a per-flow lifetime
  total that needs folding.
- `dropped_findings` is a direct `self.dropped_findings` read — no change needed.
- The `use std::collections::BTreeMap;` import already exists in the current `summarize`
  body. Add `use std::collections::HashMap;` if needed (it is already in scope at
  module level in enip.rs).

---

## Spec Reconciliation

### BC-2.17.021 Invariant 2 — Does this fix contradict it?

Current text of Invariant 2:

> **Aggregate only**: `summarize()` reads `self.command_distribution`, `self.total_pdu_count`,
> `self.flows_analyzed`, etc. It does NOT re-scan flow state. Aggregate counters must be
> up-to-date from `on_flow_close` calls before `summarize()` is invoked.

The last sentence ("Aggregate counters must be up-to-date from `on_flow_close` calls
before `summarize()` is invoked") is the problematic clause. It states a precondition
that is systematically false in production: `on_flow_close` is never called for ENIP
flows in the current wiring. The sentence was written assuming a dispatch wiring that
was never implemented.

**Required clarification (cycle-close BC edit, not blocking the fix):**

Invariant 2 must be updated to reflect the Precondition 4-sanctioned fold:

> **Aggregate plus open-flow fold**: `summarize()` reads `self.command_distribution`,
> `self.total_pdu_count`, `self.flows_analyzed`, etc. as the base aggregates (reflecting
> any flows closed via `on_flow_close`). Per BC-2.17.021 Precondition 4, `summarize()`
> also folds still-open `self.flows.values()` into a transient combined view at call
> time. `self.flows` is not mutated. The invariant "does NOT re-scan flow state" applies
> to the closed-flow aggregate fields only; folding open flows is explicitly permitted
> by Precondition 4.

The second sentence of the current Invariant 2 ("Aggregate counters must be up-to-date
from `on_flow_close` calls before `summarize()` is invoked") should be removed — it
describes the unimplemented wiring variant (option b) and is now incorrect.

**This clarification is a cycle-close edit** (i.e., it can land in the same PR as the
fix, or in a subsequent spec-versioning pass). It does NOT need to precede the fix PR.

**BC-2.17.017 — no change required.** The `on_flow_close` contract remains correct
as written. It is still the right mechanism for mid-capture flow teardown if the
dispatcher is ever wired. The fix does not remove or invalidate `on_flow_close`; it
adds a complementary path in `summarize()`.

### Input-Hash Implications

STORY-138 lists `BC-2.17.021.md` as an input in its frontmatter `inputs:` list. However:

- STORY-138 is already merged (feature branch STORY-138 delivered to `develop`).
- The fix PR is a new delivery — it is not STORY-138 being re-run.
- A new story (e.g., STORY-139 or a direct fix commit) that implements this change will
  list the updated `BC-2.17.021.md` as an input. Its `input-hash` must be computed fresh
  against the post-clarification BC text via `bin/compute-input-hash`.
- STORY-138's stored `input-hash` in `.factory/stories/STORY-138.md` does NOT need to
  be retroactively invalidated — the story was correct given the spec as written at
  implementation time. The spec clarification is a post-hoc correction to an invariant
  that was written against an unimplemented precondition.
- If `bin/compute-input-hash --scan` is run against the factory-artifacts branch after
  the BC edit, STORY-138's hash will register STALE. This is expected and acceptable:
  the story is merged and frozen; the STALE reading is informational only.

---

## BLOCK/DEFER Verdict

**BLOCK — must be fixed before v0.11.0 release.**

### Justification

The `enip_summary` JSON object is the primary operator-visible output of the SS-17
subsystem. Of its seven canonical fields:

| Field | Status in production |
|-------|---------------------|
| `command_distribution` | **ZERO** (empty map) for any real capture |
| `total_pdu_count` | **ZERO** for any real capture |
| `parse_errors` | **ZERO** for any real capture |
| `flows_analyzed` | **ZERO** for any real capture |
| `write_count` | Correct (incremented directly in process_pdu) |
| `error_count` | Correct (incremented directly in process_pdu) |
| `dropped_findings` | Correct |

Four of the seven fields report zero in every real capture. The canonical test vector
in BC-2.17.021 ("3 ListIdentity, 1 flow → flows_analyzed:1, total_pdu_count:3") is
unmet in production. The feature is half-broken. Shipping v0.11.0 with this defect
would deliver a summarize output that is misleading (zeros that look like no traffic
when traffic was clearly analyzed — findings will be present but the aggregate counts
are zeroed).

The fix is low-blast-radius: one method in one file, no API surface changes, no new
BCs, no dispatcher wiring. The discriminating test (see below) verifies the fix
directly. The spec clarification is additive. There is no justification for deferral.

---

## Discriminating Test Guidance

### Test Name

`test_summarize_reflects_open_flows_without_close`

### Scenario

Drive 3 ListIdentity (command 0x0063) frames through `on_data` for a single
`FlowKey`, then call `summarize()` **without calling `on_flow_close`**. Assert that
`enip_summary` reflects the actual traffic.

### Setup

Use the existing test helper pattern from STORY-138 tests:
- Construct a minimal valid ENIP `ListIdentity` frame (24-byte header, command=0x0063,
  length=0, session_handle=0, status=0, sender_context=[0u8;8], options=0).
  Total frame size = 24 bytes (no payload — ListIdentity has no CPF body in request form).
- Call `analyzer.on_data(flow_key, frame_bytes, ts)` 3 times with this frame.
- Call `let summary = analyzer.summarize()` — do NOT call `on_flow_close`.
- Extract `enip_summary` from `summary.detail["enip_summary"]`.

### Expected Values

```
enip_summary.total_pdu_count    == 3
enip_summary.flows_analyzed     == 1   // the 1 still-open flow counted at summarize time
enip_summary.parse_errors       == 0
enip_summary.write_count        == 0
enip_summary.error_count        == 0
enip_summary.dropped_findings   == 0
enip_summary.command_distribution == { "0x0063": 3 }
```

### Regression Guard: on_flow_close still works when called

Add a second assertion block to the same test (or a sibling test):
- After driving 3 frames, call `analyzer.on_flow_close(flow_key)`.
- Call `analyzer.summarize()` again.
- Assert the same expected values — confirming that the close-then-summarize path
  also produces the correct output and that no double-count occurs.

### Why This Is Discriminating

The pre-fix `summarize()` reads only the aggregate fields, which are all zero when
`on_flow_close` has not been called. The test will **fail before the fix** (returns
`total_pdu_count == 0, flows_analyzed == 0`) and **pass after the fix**.
The regression guard confirms the closed-flow path is not broken by the change and
that the no-double-count invariant holds.

---

## ADR-010 Impact

No amendment to ADR-010 is required. The fix is an implementation correction, not
an architectural decision change. ADR-010 Decision 4 documents the EnipFlowState
design; its description of `on_flow_close` folding aggregate counters remains correct
as a description of that method's behavior. The summarize-time fold is the
Precondition-4-sanctioned complement, not a contradicting decision.

If a future ADR-010 revision is authored for v0.12.0 scope (UDP/2222 etc.), it should
note this ruling in its context section as a resolved implementation gap.

---

## Summary Table

| Dimension | Decision |
|-----------|---------|
| Fix approach | (a) summarize-time fold of open flows |
| Production code changes | `src/analyzer/enip.rs` `summarize()` only |
| Spec changes needed | BC-2.17.021 Invariant 2 clarification (cycle-close edit) |
| BC-2.17.017 changes | None |
| ADR-010 changes | None |
| dispatcher.rs changes | None |
| main.rs changes | None |
| STORY-138 input-hash | Expected STALE after BC edit; informational only (story is merged) |
| New story for fix | Required (story-writer assigns; single-story fix PR) |
| Block v0.11.0 release | **YES** |
| Fix blast radius | Low — one method, one file |
