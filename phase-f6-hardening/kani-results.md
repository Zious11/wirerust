# Phase F6 — Kani Formal Verification Results (Feature #100)

**Feature:** issue-100-pcap-timestamps (pcap per-packet timestamp → `Finding.timestamp`)
**VP in scope:** VP-021 (timestamp-provenance-threading) — only VP in F6 scope
**develop HEAD:** `256a490`
**Delta range:** `afe93a1^..256a490`
**Date:** 2026-06-09
**kani version:** cargo-kani 0.67.0 (installed and available)

---

## Summary

| Metric | Value |
|--------|-------|
| Kani harnesses attempted | 0 (justified non-applicability) |
| Kani harnesses passed | 0 |
| VP-021 discharge | **Justified via exhaustive proptest + boundary tests**, NOT Kani |
| Anti-pattern check | PASS — no cfg-gated/debug-only guard added or relied upon |

**Verdict: VP-021 is appropriately discharged WITHOUT Kani.** No Kani harness is written,
for the documented reasons below. This is not a skip-by-omission: the property is fully
covered by existing integration tests, boundary tests, and a property test over the entire
`0..=u32::MAX` input domain.

---

## The candidate property considered for Kani

The strongest pure-core candidate in the delta is the **totality of the timestamp
conversion**:

> For all `v: u32`, `DateTime::from_timestamp(v as i64, 0)` returns `Some(_)` — it never
> returns `None` and never panics.

This is the only timestamp-dependent computation introduced by the feature. It appears
**inline at 21 emission sites** (9 in `src/analyzer/http.rs`, 7 in `src/analyzer/tls.rs`,
3 in `src/reassembly/mod.rs`, 2 in `src/reassembly/lifecycle.rs`), always in the exact form:

```rust
timestamp: DateTime::from_timestamp(<u32-valued expr> as i64, 0),
```

where `<u32-valued expr>` is either `last_ts` (per-flow stored `u32`) or `timestamp` (the
`u32` parameter threaded from the reassembler).

## Why Kani is NOT used (and is NOT the right tool here)

### 1. The conversion is inline chrono library code, not a wirerust pure helper

There is **no custom pure function in the wirerust source tree** that performs the u32→DateTime
conversion. It is a direct call to `chrono::DateTime::<Utc>::from_timestamp`, a third-party
library function. A Kani harness over a nondeterministic `u32` would do one of two things:

- **Symbolically execute chrono's `from_timestamp` internals** — which construct a
  `NaiveDateTime` via `NaiveDate::from_num_days_from_ce_opt` and modular date arithmetic.
  Kani would be re-proving a **chrono library invariant**, not a wirerust property. Per the
  Dark Factory verification discipline, we prove **our** code, not vendored library internals
  that already carry the library's own test suite and are out of our change-control scope.
- **Stub/abstract chrono** — which would make the proof vacuous (we'd be asserting our own
  stub's behavior, proving nothing about the real conversion).

Either way, a Kani harness here adds no assurance over what already exists.

### 2. The full VP-021 property is not a bounded pure-arithmetic invariant

VP-021's complete property statement (two-case hot-path/close-flush semantics, per-flow
`HashMap<FlowKey, u32>` timestamp storage, the full TCP-reassembly + HTTP/TLS analyzer
pipeline, and `DateTime<Utc>` value equality) is explicitly **out of Kani's tractable
bounds**. The VP document itself records this under "Why NOT Kani" (lines 53–57) and the
feasibility table ("Kani suitability: NOT SUITABLE"). This F6 assessment independently
confirms that conclusion from the source code and the spec.

### 3. The totality claim is dischargeable by exhaustive domain reasoning + tests

The totality claim is **trivially true by the type/range argument**, and does not need a
model checker:

- For any `v: u32`, the widening cast `v as i64` is lossless and yields a value in the
  closed interval `[0, 4_294_967_295]` (`u32::MAX`).
- `chrono::DateTime::from_timestamp(secs, 0)` returns `None` only when `secs` falls outside
  roughly `±262,000` years from the Unix epoch (i64 seconds beyond chrono's `NaiveDateTime`
  range). The entire u32 range maps to `1970-01-01T00:00:00Z` (`v=0`) through approximately
  `2106-02-07T06:28:15Z` (`v=u32::MAX`) — comfortably inside chrono's representable range.
- Therefore `Some(_)` is returned for **every** `u32`, with no panic and no `None`.

This is precisely the claim asserted by BC-2.09.007 Invariant 2 and Edge Cases EC-003/EC-004.

## How VP-021 IS discharged (the appropriate tools)

The existing test harness in `tests/timestamp_threading_tests.rs` (1096 lines, all green at
HEAD `256a490`) covers the property:

| Coverage | Test | What it proves |
|----------|------|----------------|
| Hot-path threading (HTTP) | `test_finding_timestamp_hot_path` | flush-path `Finding.timestamp == Some(from_timestamp(ts_sec,0))` |
| Hot-path threading (TLS) | `test_finding_timestamp_hot_path_tls` | TLS emission sites carry the threaded timestamp |
| Close-flush threading | `test_finding_timestamp_close_flush` | `close_flow` uses `flow.last_seen` |
| Segment-limit summary = None | `test_segment_limit_summary_timestamp_is_none_and_absent_from_json` | finalize aggregate retains `None`; JSON omits the key |
| JSON ISO-8601 serialization | `test_finding_timestamp_json_serialization` | `Some(ts)` serializes correctly |
| **Boundary: 0 and u32::MAX** | `test_timestamp_conversion_boundary_values` | EC-003 (epoch) and EC-004 (u32::MAX → year 2106, `Some`, no panic); asserts `0` and `u32::MAX` map to distinct `Some` values |
| **Property over full u32 domain** | `prop_finding_timestamp_matches_on_data_timestamp` (proptest, `ts_sec in 0u32..=u32::MAX`, 256 cases) | randomized sampling across the entire input domain; each case constructs the expected `DateTime` via `from_timestamp(...).expect(...)` — a `None` would fail the `.expect` |
| **Cross-flow isolation** | `prop_cross_flow_timestamp_isolation` (proptest, 128 cases) | flow A's timestamp never appears in flow B's findings (VP-014-aligned) |

**Note on "exhaustive":** The proptest strategy `0u32..=u32::MAX` samples (256 cases) across
the full domain rather than enumerating all 2^32 values. Combined with the explicit boundary
tests at the two domain extremes (`0` and `u32::MAX`) and the closed-form totality argument
above, the conversion's totality and value-correctness are established. The conversion has
exactly one branch of interest (Some vs None), and the boundary tests pin both domain
endpoints, so the sampled proptest plus endpoint tests cover the only behavior that could vary.

## Anti-pattern guard (explicit, per orchestrator instruction)

A prior human gate rejected the anti-pattern of adding a debug-only / `cfg`-gated guard and
calling the property "proven." This assessment:

- Adds **no** `#[cfg(kani)]`, `#[cfg(debug_assertions)]`, or any other cfg-gated guard to the
  source tree.
- Makes **no** change to `src/`.
- Discharges VP-021 by the genuine existing tests + a sound range argument, not by a
  conditionally-compiled assertion.

## Recommendation for VP-021 document

VP-021 currently declares `proof_method: integration + proptest` and `status: draft`,
`verification_lock: false`. This F6 assessment **confirms** that proof method is correct and
that Kani is appropriately excluded. The VP's existing "Why NOT Kani" rationale stands and is
independently corroborated here. Locking VP-021 (setting `verification_lock: true`, recording
the proof file hash of `tests/timestamp_threading_tests.rs`) is a spec-steward action at the
F6 gate; the verification evidence required for that lock is present and green.
