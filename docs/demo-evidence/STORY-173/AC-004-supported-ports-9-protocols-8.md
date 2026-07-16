# AC-173-004: SUPPORTED_PORTS 8→9 + supported_protocols 7→8

**AC:** AC-173-004
**BC:** BC-2.18.003 postconditions 1–2
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

Port 2404 (IEC 60870-5-104, IANA-registered TCP) is added to the `SUPPORTED_PORTS` compile-time
constant in `src/protocols.rs`. This increases `SUPPORTED_PORTS.len()` from 8 to 9. Because the
`IEC 60870-5-104` entry was already present in `KNOWN_PROTOCOLS` as an unsupported entry, adding
port 2404 to `SUPPORTED_PORTS` is all that is required — `supported_protocols()` now returns it
via the port intersection, increasing its count from 7 to 8.

---

## Source confirmation

`src/protocols.rs` — `SUPPORTED_PORTS` constant (line 74):
```rust
pub const SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 2404, 443, 8443, 80, 8080, 53];
```

`src/protocols.rs` — comment confirming IEC 60870-5-104 promotion (lines 81–84):
```
/// IEC 60870-5-104 is functionally supported
/// (port 2404 in `SUPPORTED_PORTS` since STORY-173; BC-2.18.003 PC-1) but is
/// [physically in the Tier-1 block]; membership via port-filter on port 2404.
```

---

## Test output

Command:
```
cargo test --test protocols_tests story_173
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running tests/protocols_tests.rs

running 4 tests
test story_173::test_BC_2_18_003_supported_ports_len_is_9 ... ok
test story_173::test_BC_2_18_003_supported_ports_contains_2404 ... ok
test story_173::test_BC_2_18_003_supported_protocols_len_is_8 ... ok
test story_173::test_BC_2_18_003_iec104_in_supported_protocols ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out; finished in 0.00s
```

Per-test behavior:
- `test_BC_2_18_003_supported_ports_contains_2404`: asserts `SUPPORTED_PORTS.contains(&2404)`.
- `test_BC_2_18_003_supported_ports_len_is_9`: asserts `SUPPORTED_PORTS.len() == 9`.
- `test_BC_2_18_003_supported_protocols_len_is_8`: asserts `supported_protocols().len() == 8`.
- `test_BC_2_18_003_iec104_in_supported_protocols`: asserts `supported_protocols()` contains an
  entry whose name contains `"60870-5-104"`.

---

## Verdict

PASS — `SUPPORTED_PORTS.len() == 9`, `2404` is present, `supported_protocols().len() == 8`,
and `IEC 60870-5-104` is returned by `supported_protocols()`.
