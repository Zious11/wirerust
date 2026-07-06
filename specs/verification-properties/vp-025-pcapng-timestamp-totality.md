---
artifact: verification-property
vp_id: VP-025
title: "pcapng Timestamp Conversion Totality (saturation-locked)"
status: verified
phase: P1
tool: Kani
subsystem: SS-01
module: "reader.rs (pcapng_pure_core fns)"
producer: architect
timestamp: 2026-06-19T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-009-pcapng-reader-design.md
feature_cycle: feature-pcapng-reader
source_bc: BC-2.01.014
bcs:
  - BC-2.01.014
verification_lock: true
verified_at_commit: "1ca30a3"
verified_prs: "#293, #294"
---

# VP-025: pcapng Timestamp Conversion Totality (saturation-locked)

## Property Statement

The pure-core function `pcapng_timestamp_to_secs_usecs(ts_sec: u32, ts_frac: u32, type_id: u8)`
in `src/reader.rs` is total — it never panics — and produces output values within the
following bounds for **all possible symbolic inputs**:

1. `ts_usec` is always in `[0, 999_999]` (a valid microsecond component).
2. `ts_sec` is saturated at `u32::MAX` when the true seconds count exceeds `u32::MAX`
   (via `.min(u32::MAX)` — prevents silent wraparound on extreme timestamps).
3. The large-`ts_high` Kani vector (ticks/ticks_per_sec > `u32::MAX`) confirms the
   saturation guard fires on inputs that would overflow without it (M-3 non-vacuity).

The property holds for all three timestamp resolution families supported by pcapng:
base-10 (10^n ticks/second), base-2 (2^n ticks/second), and the µs fast-path (10^6 ticks/second).

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.01.014 | pcapng timestamp conversion: ts_usec in [0, 999999]; ts_sec saturated at u32::MAX; no panic for all (u32, u32, u8) inputs | All harnesses |

## Module Anchor Clarification

**Kani target is the pure-core helper function only.** The target is
`pcapng_timestamp_to_secs_usecs(u32, u32, u8) -> (u32, u32)` — a pure arithmetic
function in `src/reader.rs`. The effectful entry point `from_pcap_reader<R: Read>` is
not a Kani target (it performs I/O and uses generic bounds). The module label
`reader.rs (pcapng_pure_core fns)` denotes the compilation unit, not the top-level
entry point.

**VP-025 Kani provability note (I-2 resolution, ADR-009 rev 5):** The base-10 branch
calls `10u64.checked_pow(e as u32)` which is iterative. The implementation uses a
precomputed lookup table for `e∈[0,19]` to eliminate the loop and make the proof
trivially bounded without `#[kani::unwind(128)]`.

## Proof Harnesses

Four harnesses split by divisor-constant family to resolve the unwind/tractability
constraint (per-divisor-constant split resolves I-2; see ADR-009 rev 5):

```rust
// VP-025 Sub-µs: fast-path (ticks_per_sec == 10^6)
#[kani::proof]
fn vp025_timestamp_totality() { ... }

// VP-025 Sub-base10 (ticks_per_sec = 10^e, e in [0,12])
#[kani::proof]
fn vp025_timestamp_totality_base10() { ... }

// VP-025 Sub-base10-saturating (large-ts_high vector: ticks/ticks_per_sec > u32::MAX)
#[kani::proof]
fn vp025_timestamp_totality_base10_saturating() { ... }

// VP-025 Sub-base2 (ticks_per_sec = 2^e, e in [0,40])
#[kani::proof]
fn vp025_timestamp_totality_base2() { ... }
```

All four harnesses report `cargo kani VERIFICATION SUCCESSFUL` (59 checks each),
non-vacuity confirmed. Locked at develop@1ca30a3 (PRs #293 + #294).

## Feasibility Assessment

**Assessment: FEASIBLE (completed — SUCCESSFUL at F6 lock).**

Pure arithmetic function; no loops in the lookup-table implementation; symbolic `u32 × u32 × u8`
domain fully enumerable by Kani at 59 checks per harness. The per-divisor split was required
to keep each harness tractable.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-025 designed, added to VP-INDEX (ADR-009 rev 4) | draft |
| F4 (TDD implementation) | Harnesses authored for pcapng reader story | draft → active |
| F6 (formal hardening) | All 4 harnesses cargo kani VERIFICATION SUCCESSFUL (59 checks each); non-vacuity confirmed; large-ts_high saturation vector confirms M-3 guard fires | active → verified |

Lock: `status: verified`, `verification_lock: true` set by state-manager after F6 confirmation
@ develop 1ca30a3 (PRs #293 + #294). Mirrors VP-021/VP-022/VP-023/VP-024 lock pattern.
