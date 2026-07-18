# AC-173-002: T0881 Six-Part Atomic Catalog Registration

**AC:** AC-173-002
**BC:** BC-2.10.010 postconditions 1–6, invariant 1 (ADR-013 Decision 10)
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

T0881 "Service Stop" is registered in `src/mitre.rs` as a single six-part atomic commit.
All six parts must be delivered together; no partial commit is acceptable because the
`vp007_catalog_drift_guard` `#[test]` (lib unit test) enforces that `SEEDED_TECHNIQUE_IDS`,
`SEEDED_TECHNIQUE_ID_COUNT`, and `technique_info` arm counts are always in sync.

The six parts:
1. `"T0881"` added to `SEEDED_TECHNIQUE_IDS` array (28 → 29 entries)
2. `SEEDED_TECHNIQUE_ID_COUNT` bumped from 28 to 29
3. `EMITTED_IDS` array gains `"T0881"` (IEC-104 STOPDT findings emit this technique)
4. `technique_info("T0881")` arm returns `("Service Stop", MitreTactic::IcsInhibitResponseFunction)`
5. `vp007_catalog_drift_guard` `#[test]` passes at count=29
6. `verify_all_emitted_ids_resolve` Kani harness passes for T0881

---

## Source confirmation

`src/mitre.rs` — SEEDED_TECHNIQUE_IDS (line 475):
```
"T0881",
```

`src/mitre.rs` — SEEDED_TECHNIQUE_ID_COUNT (line 485):
```rust
const SEEDED_TECHNIQUE_ID_COUNT: usize = 29;
```

`src/mitre.rs` — EMITTED_IDS (line 366):
```
"T0881", // Service Stop (IcsInhibitResponseFunction/TA0107; STOPDT-act)
```

`src/mitre.rs` — technique_info arm (lines 249–251):
```rust
// STORY-173 / VP-007 atomic obligation (ADR-013 Decision 10)
// Tactic: IcsInhibitResponseFunction (TA0107). Emitted on STOPDT-act in IEC-104 flows.
"T0881" => ("Service Stop", MitreTactic::IcsInhibitResponseFunction),
```

---

## Test output

Command:
```
cargo test --test mitre_tests story_173
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/mitre_tests.rs

running 5 tests
test story_173::test_BC_2_10_010_t0881_catalog_entry ... ok
test story_173::test_BC_2_10_010_t0881_tactic_id_is_ta0107 ... ok
test story_173::test_BC_2_10_010_t0881_in_emitted_ids_source ... ok
test story_173::test_BC_2_10_010_t0881_in_seeded_ids_source ... ok
test story_173::test_BC_2_10_010_seeded_count_is_29 ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.01s
```

Drift guard test (part 5 of the atomic — runs as `#[test]` inside the lib):
```
cargo test --lib vp007_catalog_drift_guard
```
```
     Running unittests src/lib.rs

running 1 test
test mitre::vp007_format_tests::vp007_catalog_drift_guard ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out; finished in 1.86s
```

Per-test behavior:
- `test_BC_2_10_010_t0881_catalog_entry`: calls `technique_info("T0881")`, asserts name=`"Service Stop"` and tactic=`MitreTactic::IcsInhibitResponseFunction`.
- `test_BC_2_10_010_t0881_tactic_id_is_ta0107`: calls `technique_tactic_id("T0881")`, asserts `Some("TA0107")`.
- `test_BC_2_10_010_seeded_count_is_29`: reads `src/mitre.rs` source, scans for `SEEDED_TECHNIQUE_ID_COUNT` constant declaration, asserts it contains `: usize = 29`.
- `test_BC_2_10_010_t0881_in_seeded_ids_source`: reads `src/mitre.rs`, extracts `SEEDED_TECHNIQUE_IDS` block, asserts `"T0881"` appears in it.
- `test_BC_2_10_010_t0881_in_emitted_ids_source`: reads `src/mitre.rs`, extracts `EMITTED_IDS` block, asserts `"T0881"` appears in it.

Part 6 (`verify_all_emitted_ids_resolve` Kani harness) is a `#[kani::proof]` function and
runs under `cargo kani` (deferred to STORY-174 formal hardening per the story spec). The unit
test `test_BC_2_10_010_t0881_in_emitted_ids_source` provides the P0 source-level regression
guard for the EMITTED_IDS entry.

---

## Verdict

PASS — All five unit-test parts of the six-part atomic commit are verified. The Kani
`verify_all_emitted_ids_resolve` harness (part 6) will be run in STORY-174 formal
hardening. The T0881 tactic is correctly `IcsInhibitResponseFunction` (TA0107), not
`IcsExecution` or `Impact`.
