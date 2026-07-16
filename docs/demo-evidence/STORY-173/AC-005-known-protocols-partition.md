# AC-173-005: Protocol Catalog Partition Invariant Preserved

**AC:** AC-173-005
**BC:** BC-2.18.004 postconditions 1–2, invariant 1 (VP-041 proptest)
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

After port 2404 is added to `SUPPORTED_PORTS`, the `KNOWN_PROTOCOLS` catalog partition
invariant must still hold:

- `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS` (union completeness)
- `supported_protocols() ∩ unsupported_protocols() == ∅` (disjointness)
- `supported_protocols().len() + unsupported_protocols().len() == KNOWN_PROTOCOLS.len()`

Adding port 2404 moves the `IEC 60870-5-104` entry from `unsupported_protocols()` to
`supported_protocols()` while maintaining the partition. VP-041 proptests verify this
invariant across randomly generated scenarios.

---

## Source confirmation

`src/protocols.rs` — `supported_protocols()` definition uses the port-intersection predicate
(lines 428–441):
```rust
/// Returns protocols whose canonical_ports overlap SUPPORTED_PORTS...
/// DNS, HTTP, IEC 60870-5-104. IEC 60870-5-104 is included because port 2404
/// was added to SUPPORTED_PORTS in STORY-173 (BC-2.18.003 PC-1).
pub fn supported_protocols() -> Vec<&'static KnownProtocol> {
    ...
    .any(|port| SUPPORTED_PORTS.contains(port))
```

`src/protocols.rs` — `unsupported_protocols()` (line 455):
```rust
/// Returns the exact complement of `supported_protocols()` within `KNOWN_PROTOCOLS` —
/// i.e., every entry NOT in `supported_protocols()`.
pub fn unsupported_protocols() -> Vec<&'static KnownProtocol> {
```

---

## VP-041 proptest output

Command:
```
cargo test --test protocols_tests proptest_vp041
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/protocols_tests.rs

running 2 tests
test story_151::proptest_vp041_oracle_cross_check ... ok
test story_151::proptest_vp041_partition_invariant ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.04s
```

These two proptests (`proptest_vp041_partition_invariant` and `proptest_vp041_oracle_cross_check`)
exercise the partition invariant across multiple configurations. They are defined in
`tests/protocols_tests.rs` `mod story_151` — originally introduced by STORY-151 and inherited
as regression guards. They pass cleanly after the STORY-173 port-2404 addition, confirming no
invariant violation was introduced by moving `IEC 60870-5-104` from unsupported to supported.

The four STORY-173 unit tests from AC-173-004 also serve as positive partition confirmation:
- `test_BC_2_18_003_partition_len` (existing, in pre-story_173 module): confirmed counts still sum.
- The union and disjointness invariants are enforced structurally by `unsupported_protocols()`
  being defined as the complement of `supported_protocols()` within `KNOWN_PROTOCOLS`.

---

## Partition state after STORY-173

| Set | Count | IEC 60870-5-104 in set? |
|-----|-------|------------------------|
| `supported_protocols()` | 8 | Yes (port 2404 in SUPPORTED_PORTS) |
| `unsupported_protocols()` | 22 | No (moved to supported) |
| `KNOWN_PROTOCOLS` | 30 | Yes (present in both views) |

Partition check: 8 + 22 == 30. Invariant holds.

---

## Verdict

PASS — VP-041 proptests pass, partition invariant holds after STORY-173 adds port 2404.
`IEC 60870-5-104` now appears in `supported_protocols()` and not in `unsupported_protocols()`.
