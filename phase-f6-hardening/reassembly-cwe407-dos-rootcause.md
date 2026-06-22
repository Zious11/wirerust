# CWE-407 + CWE-401 Reassembly DoS — Root Cause and Fix Summary

**Document type:** Post-fix root-cause analysis  
**Defect ID:** PERF-REASM-DOS-001  
**Decision:** D-197  
**Status:** RESOLVED (PR #298, b5b54d5)  
**Date:** 2026-06-22  

---

## Symptom

`wirerust analyze --all` (and `--reassemble`) ran approximately 50 minutes at 100% CPU
with RSS frozen at ~1.18 GB on a 1 GB synthetic pcapng file built by replicating the
4SICS pcapng capture to produce >100K distinct TCP flows with non-increasing timestamps.

Real, un-replicated captures were unaffected:
- 200 MB 4SICS pcapng `--all`: 1.60 s
- 1.17 GB CUPID native pcapng `--all`: 0.88 s

Discovered during FE-001 large-pcapng scale validation (D-196). The synthetic
replicated-flow input was constructed to stress-test the pcapng reader; it
inadvertently exposed a pre-existing reassembly-engine bug. NOT an FE-001
(pcapng reader) defect — FE-001 remains COMPLETE.

---

## Root Cause

Three defects in the reassembly engine, compounding to produce the observed runaway.

### Defect 2 (Primary) — CWE-407: Off-by-one in null-eviction condition

**File:** `src/reassembly/lifecycle.rs`

The flow-table eviction guard used `flows.len() >= max_flows` as the trigger
condition but broke the eviction loop at `<= max_flows`. This created a null-eviction
cycle: the condition fired on every new-flow packet once the table was full, triggering
the full O(F log F) candidate sort, but evicting zero flows because the break
condition was immediately satisfied.

The `evictions:0` statistic in telemetry hid this — the counter incremented only on
actual evictions, not on null-eviction cycles.

Quantitative model: ~400K null-eviction cycles × ~7 ms per O(F log F) sort ≈ 47 min.
This matches the ~50 min observed.

### Defect 1 (Secondary) — CWE-401: Zombie segment accumulation

**File:** `src/reassembly/segment.rs`, function `insert_segment`

`insert_segment` had no guard for segments arriving below `base_offset`. These
segments were inserted as `OutOfWindow` candidates but never reaped, accumulating
as zombie entries that inflated the per-flow segment list and increased sort cost.

### Defect 3 — Frozen-timestamp expiry suppression

**File:** `src/reassembly/lifecycle.rs`

The idle-expiry sweep used `timestamp > last_expiry_sweep_secs` as its trigger
condition. On the synthetic pcapng input, timestamps were non-increasing (replicated
flow data), so this condition never fired, suppressing all expiry sweeps and allowing
the flow table to fill without relief.

---

## Fix (PR #298, merged b5b54d5)

Four remediations applied:

**R1 — Below-base zombie guard (`segment.rs`)**  
Added `if end_offset < base_offset { return Err(SegmentError::OutOfWindow); }` in
`insert_segment`. Prevents zombie segments from accumulating below the reassembly
window.

**R2 — Off-by-one correction (`lifecycle.rs`)**  
Changed eviction break condition from `<= max_flows` to `< max_flows`. The loop now
evicts until the table is actually below capacity, not until it matches capacity.

**R3 — Batch eviction to 90% headroom (`lifecycle.rs`)**  
Introduced `EvictionTrigger` enum separating the trigger check from the eviction
volume. When eviction fires, the engine now evicts to 90% of max_flows in a single
pass (sorting once per batch, not once per packet). This reduces the O(F log F) sort
to approximately O(1) amortised per packet under sustained flow-table pressure.

**R4 — Production timestamp-independent idle expiry (`lifecycle.rs`)**  
Added a monotonic `packet_index` counter (incremented per packet, independent of
pcap timestamps) and a `last_activity_index` field per flow. The expiry sweep now
fires every `expiry_sweep_interval = 8192` packets unconditionally, expiring flows
idle for more than `idle_packet_threshold = 65536` packets. This ensures expiry runs
regardless of whether pcap timestamps are monotone.

Note on detection-evasion: active flows update `last_activity_index` and `last_seen`
on every packet, so legitimately active flows are never reaped by R4.

---

## Test Evidence

6 RED→GREEN tests covering all four remediations.

Benchmark results:
- **Before fix:** 75 s on a 120K-flow synthetic input (extrapolated from ~50 min at >100K flows)
- **After fix:** 0.76 s on the same 120K-flow input — **~100× improvement**

Code review: APPROVE.  
Security review: detection-evasion paths confirmed non-exploitable (active flows
update `last_activity_index`/`last_seen` per packet, never reaped while active).

---

## New Configuration Fields

Added to reassembly config (with documented defaults):

| Field | Default | Purpose |
|-------|---------|---------|
| `expiry_sweep_interval` | 8192 | Packets between idle-expiry sweeps |
| `idle_packet_threshold` | 65536 | Packets of inactivity before a flow is expired |

---

## CWE Classification

| CWE | Description | Resolved By |
|-----|-------------|-------------|
| CWE-407 | Algorithmic complexity — O(F log F) sort per packet (null-eviction) | R2 + R3 |
| CWE-401 | Zombie memory — below-base segments never reaped | R1 |

---

## Follow-Up (PERF-REASM-NFR-001)

LOW priority: consider a formal NFR and VP for "reassembly per-packet CPU is O(1)
amortised under flow-table pressure." The constraint is currently encoded by regression
tests (R5a/R5b/R3/R4 in `reassembly_engine_tests.rs`). Target: spec-hardening backlog.
