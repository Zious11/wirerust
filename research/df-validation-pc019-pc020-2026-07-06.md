---
title: DF-VALIDATION-001 — pattern-consistency findings validation for FIX-B PR
run_id: maint-2026-07-06
date: 2026-07-06
policy: DF-VALIDATION-001 (.factory/policies.yaml)
scope: DNP3 analyzer observability-counter gaps queued for FIX-B
inputs:
  - .factory/maintenance/pattern-consistency.md
  - src/analyzer/dnp3.rs
  - src/analyzer/modbus.rs
  - src/analyzer/enip.rs
input-hash: 213893a
verdicts:
  - PC-016: CONFIRMED (observability gap); T1692.001 masking mechanism REFUTED
  - PC-017: CONFIRMED
  - PC-003: CONFIRMED
---

# DF-VALIDATION-001 — FIX-B Pattern-Consistency Finding Validation

## Nomenclature Reconciliation

Team-lead dispatch referred to the two new DNP3 observability findings as
**PC-019** and **PC-020**.  The canonical finding file
(`.factory/maintenance/pattern-consistency.md`, sweep maint-2026-07-06) assigns
those numbers to different findings (Modbus HashMap key-order, ENIP
`StreamHandler` gap).  The dispatched claims match:

| Team-lead label | Canonical ID | Topic                                   |
|-----------------|--------------|-----------------------------------------|
| PC-019          | **PC-016**   | DNP3 `master_addrs_seen` silent ignore  |
| PC-020          | **PC-017**   | DNP3 `pending_requests` LRU eviction    |
| PC-003          | PC-003       | DNP3 missing `dropped_findings` counter |

Validation below uses the canonical IDs.  This mismatch is a documentation drift
in the dispatch language and should be corrected before FIX-B PR body is drafted.

---

## PC-016 — DNP3 `master_addrs_seen` silent ignore has no observable counter

**Claim:** cap-triggered silent ignore may silence detection of T1692.001
(rogue master).

### Site verification

- **Cap constant:** `MAX_MASTER_ADDRS = 64` declared at
  `src/analyzer/dnp3.rs:146`. Doc comment (line 144) explicitly states
  "Once full, new master source addresses are silently ignored."
- **State field:** `Dnp3FlowState.master_addrs_seen: Vec<u16>` declared at
  `src/analyzer/dnp3.rs:247`.
- **Silent-ignore site:** frame-walk push gate at
  `src/analyzer/dnp3.rs:750-755`:

  ```rust
  if is_master_frame(header.control)
      && !flow.master_addrs_seen.contains(&header.source)
      && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
  {
      flow.master_addrs_seen.push(header.source);
  }
  ```

  The third conjunct silently short-circuits the push when the set is full.
  No `else` branch, no counter increment.

### Counter verification

- `Dnp3Analyzer` struct (`src/analyzer/dnp3.rs:301-332`): no
  `master_addrs_dropped` or `dropped_map_entries` field.
- `Dnp3Analyzer::summarize()` (`src/analyzer/dnp3.rs:1715-1789`): emits five
  detail keys (`function_code_distribution`, `control_operation_counts`,
  `total_frames`, `parse_errors`, `flows_analyzed`) — no eviction/drop key.
- `grep master_addrs_dropped src/analyzer/dnp3.rs` returns no matches.

### Trace of the claimed T1692.001 masking path

Detection logic (`src/analyzer/dnp3.rs:745-746, 817-831`):

```rust
let src_was_known = flow.master_addrs_seen.contains(&header.source);
let expected_set_established = !flow.master_addrs_seen.is_empty();
...
if is_master_frame(header.control) && expected_set_established && !src_was_known {
    Self::detect_unexpected_source_split(...);  // T1692.001
}
```

`src_was_known` is captured **before** the push gate runs.  A full
`master_addrs_seen` (len = 64) does **not** cause `contains(new_src)` to return
`true` — it returns `false` for any address not already present.  So a genuine
rogue source in a full-cap flow still passes the `!src_was_known` guard and
`detect_unexpected_source_split` fires.

The finding narrative's claim that "a full `master_addrs_seen` causes …
future sources [to be treated] as if 'known'" is **factually incorrect** —
the cap gates the push, not the membership check.  The finding's proposed
detection-masking mechanism is not present in the code.

There is a narrow secondary path: if 65 distinct **legitimate** masters
appear in a flow (>MAX_MASTER_ADDRS) the 65th fires a **false-positive**
T1692.001 and burns the `unexpected_source_emitted` one-shot guard (line
1463), silencing any subsequent real intruder.  This is real but unlikely
in the field (>64 unique master link-layer addresses per single TCP flow is
extreme), and the one-shot behavior is by design per BC-2.15.010 Invariant 5
— independent of the cap.

### Verdict

**CONFIRMED — observability gap. REFUTED — proposed T1692.001 masking mechanism.**

Evidence: the cap and silent-ignore site exist verbatim at
`src/analyzer/dnp3.rs:146,750-755`; no `master_addrs_dropped` counter or
summary key exists; but T1692.001 firing does not depend on
`master_addrs_seen` growth, so the proposed direct-masking path via the cap is
incorrect.  The counter should still be added for consistency with the
v0.11.4 pattern (ARP `bindings_evicted`, Modbus `dropped_transactions`,
HTTP/TLS `dropped_map_entries`) and to give operators visibility into
cap-pressure.  **FIX-B rationale should be reframed as observability parity
with the v0.11.4 pattern, not as "T1692.001 silencing."**

---

## PC-017 — DNP3 `pending_requests` LRU eviction has no observable counter

**Claim:** LRU eviction may degrade T1691.001 (unauthorized command)
request/response correlation.

### Site verification

- **Cap constant:** `MAX_PENDING_REQUESTS = 256` declared at
  `src/analyzer/dnp3.rs:123`.  Doc comment states "Oldest entry evicted on
  overflow."
- **State field:** `Dnp3FlowState.pending_requests: HashMap<(u16, u8), u32>`
  declared at `src/analyzer/dnp3.rs:253`.
- **Eviction site:** helper `Dnp3Analyzer::insert_pending_request`
  (`src/analyzer/dnp3.rs:1799-1815`):

  ```rust
  fn insert_pending_request(flow: &mut Dnp3FlowState, key: (u16, u8), request_ts: u32) {
      if flow.pending_requests.len() >= MAX_PENDING_REQUESTS
          && !flow.pending_requests.contains_key(&key)
      {
          if let Some((&oldest_key, _)) = flow.pending_requests
              .iter()
              .min_by_key(|&(_, &request_ts)| request_ts) {
              flow.pending_requests.remove(&oldest_key);
          }
      }
      flow.pending_requests.insert(key, request_ts);
  }
  ```

  Eviction is silent; no counter increment.  The doc comment at
  `src/analyzer/dnp3.rs:1797` explicitly states: **"The evicted entry is
  silently dropped — it generates NO T1691.001 timeout event (PC10)."**

### Counter verification

- `Dnp3Analyzer` struct: no `pending_requests_evicted` field.
- `summarize()` emits no eviction key.
- `grep pending_requests_evicted src/analyzer/dnp3.rs` returns no matches.

### Trace of T1691.001 correlation degradation

T1691.001 detection lives in `scan_block_timeouts`
(`src/analyzer/dnp3.rs:1137-1167`).  It iterates
`flow.pending_requests`, marks entries where
`now_ts.saturating_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS` (10s),
increments `flow.block_event_count`, and removes them.  Emission fires when
`block_event_count >= BLOCK_CMD_THRESHOLD` (3) within
`CORRELATION_WINDOW_SECS` (300).

If an entry is LRU-evicted before its 10-second timeout matures, it is
permanently dropped: it will never be seen by `scan_block_timeouts`, will
never increment `block_event_count`, and cannot contribute to a T1691.001
finding.  The code comment at line 1797 is definitive on this point.

Attack profile: an adversary flooding a Control-class request burst
(≥256 unique `(dest, app_seq)` pairs in a 10s window, then withholding
responses) forces cap pressure and displaces the oldest pending entries —
including their own earlier block-command attempts — hiding the very
signal T1691.001 is designed to catch.  This is a real (if narrow)
detection-degradation path, corroborated by the in-tree code comment.

### Verdict

**CONFIRMED.**

Evidence: cap and LRU-eviction helper exist verbatim at
`src/analyzer/dnp3.rs:123,1799-1815`; the code's own doc comment at
`src/analyzer/dnp3.rs:1797` acknowledges that the eviction produces no
T1691.001 event; no counter or summary key exists.  Both the observability
gap and the detection-degradation mechanism are present as claimed.

---

## PC-003 — DNP3 missing `dropped_findings` counter

**Claim:** `MAX_FINDINGS = 10_000` cap silently discards findings past the
limit; no counter or summary key exposes the count.

### Site verification

- **Cap constant:** `MAX_FINDINGS: usize = 10_000` declared at
  `src/analyzer/dnp3.rs:201`.  Doc comment: "Mirrors `modbus::MAX_FINDINGS`
  (10_000) — consistent DoS cap across analyzers (BC-2.15.022 Invariant 1 /
  ADR-007 Decision 2)."
- **Cap-check sites:** eleven `findings.len() < MAX_FINDINGS` guards in the
  push paths of DNP3's detection branches:

  `src/analyzer/dnp3.rs:987, 1040, 1093, 1171, 1292, 1353, 1416, 1500, 1569,
  1603, 1666` — plus the additional `findings.len() >= MAX_FINDINGS` early
  return at line 1416 (`detect_unexpected_source_split`).

  Every one of these gates falls through to a no-op when the cap is hit.
  No `else` branch, no counter increment.

### Counter verification

- `Dnp3Analyzer` struct fields (`src/analyzer/dnp3.rs:301-332`): `flows`,
  `direct_operate_threshold`, `fn_code_counts`, `all_findings`,
  `closed_flows_count`, `total_frames_closed`, `parse_errors_closed`,
  `closed_flow_direct_operates` — **no `dropped_findings` field**.
- `Dnp3Analyzer::summarize()` emits no `dropped_findings` key.
- `grep dropped_findings src/analyzer/dnp3.rs` returns no matches.

### Cross-analyzer comparator

The finding's claim of "sole analyzer missing this counter" was verified by
grep across all analyzers:

- `src/analyzer/modbus.rs:287` — `pub dropped_findings: u64,` field.
- `src/analyzer/modbus.rs:933` — `self.dropped_findings += 1;` increment on
  the `!(findings.len() < MAX_FINDINGS)` branch.
- `src/analyzer/modbus.rs:973-974` — `dropped_findings` surfaced in
  `summarize()` detail map.
- `src/analyzer/enip.rs:562, 646` — `dropped_findings: u64` field (both
  `EnipAnalyzer` and `EnipAnalysisSummary`).
- `src/analyzer/enip.rs:492-494, 1093-1096, 1199-1201, 1242-1244, 1277+` —
  multiple `dropped_findings.saturating_add(1)` increments on cap-suppressed
  paths per **BC-2.17.022 Post 3**.

DNP3 is the sole analyzer with a `MAX_FINDINGS` cap and no counter.

### Verdict

**CONFIRMED.**

Evidence: the cap exists at `src/analyzer/dnp3.rs:201`; eleven cap-check
sites silently no-op when the limit is hit; the struct at
`src/analyzer/dnp3.rs:301-332` lacks the field; `summarize()` (1715-1789)
lacks the detail key; Modbus and ENIP both carry the counter under
BC-2.14.021 / BC-2.17.022 respectively.  Post-v0.11.4 observability parity
requires DNP3 to carry the same counter.

---

## Summary Table

| ID     | Site verified                         | No counter verified | Technique-impact claim | Verdict                                                                 |
|--------|---------------------------------------|---------------------|------------------------|-------------------------------------------------------------------------|
| PC-016 | `dnp3.rs:146, 750-755`                | Yes                 | REFUTED (mechanism wrong) | **CONFIRMED** for observability gap; **REFUTED** for T1692.001 masking |
| PC-017 | `dnp3.rs:123, 1799-1815`              | Yes                 | CONFIRMED (per in-code comment at 1797) | **CONFIRMED**                                                          |
| PC-003 | `dnp3.rs:201, 11 gate sites, 301-332` | Yes                 | N/A (observability parity) | **CONFIRMED**                                                          |

## Recommendation

FIX-B may proceed with the three counters — `master_addrs_dropped`,
`pending_requests_evicted`, `dropped_findings` — added to `Dnp3Analyzer`
(analyzer-scope; consistent with Modbus/ENIP patterns) and surfaced in the
`summarize()` detail map.

**PR body language for PC-016 must be corrected** away from
"silences T1692.001" and reframed as observability parity with the v0.11.4
counter pattern (ARP, Modbus, HTTP/TLS).  The current
`.factory/maintenance/pattern-consistency.md` PC-016 narrative overstates
the technique impact and should be amended or footnoted before the PR body
cites it as justification.
