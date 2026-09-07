# AC-184-005 — The Four `parse_tpkt_header` Outcomes Are Jointly Exhaustive and Mutually Exclusive

**Story:** STORY-184: S7comm TPKT Core Parser
**AC:** AC-184-005
**Traces to:** BC-2.20.004 invariant 3
**Wave:** 87

---

## Acceptance Criterion

- Given any `&[u8]` input
- When `parse_tpkt_header(data)` is called
- Then exactly one of BC-2.20.001/002/003's `None` paths or BC-2.20.004's `Some` path
  applies — no input falls outside all four, and no input satisfies more than one
- Unit-level spot check; full exhaustiveness is the VP-048 Kani obligation (see
  `AC-006-vp048-kani-skeleton.md`)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests four_way_partition
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 1 test
test story_184::test_BC_2_20_004_four_way_partition_is_exhaustive ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.00s
```

Companion property-based test — an independently re-derived oracle checked against
`parse_tpkt_header` across randomized inputs (mutation-catcher for the four-way
classification):

```
cargo test --test iso_on_tcp_tests proptest_matches_independent_oracle
```
```
running 1 test
test story_184::proptests::test_BC_2_20_004_proptest_matches_independent_oracle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.00s
```

Result: **2/2 PASS**

---

## Test Coverage

| Test Name | Coverage | Result |
|-----------|----------|--------|
| `test_BC_2_20_004_four_way_partition_is_exhaustive` | 10 hand-picked boundary vectors spanning all 4 outcome classes (too-short, bad-version, bad-length, accept), asserting exact expected outcome for each | PASS |
| `test_BC_2_20_004_proptest_matches_independent_oracle` | Randomized inputs (length 0–16), checked against an independently re-derived 4-way classification oracle (`proptest`, 256 default cases) | PASS |

---

## Partition Boundary Cases Exercised

| Input | Class | Expected |
|-------|-------|----------|
| `[]` | A: too short | `None` |
| `[0x03]` | A: too short | `None` |
| `[0x03, 0x00, 0x00]` | A: too short | `None` |
| `[0x02, 0x00, 0xFF, 0xFF]` | B: bad version (length would otherwise be maximally valid) | `None` |
| `[0x00, 0x00, 0x00, 0x04]` | B: bad version | `None` |
| `[0x03, 0x00, 0x00, 0x00]` | C: bad length (length=0) | `None` |
| `[0x03, 0x00, 0x00, 0x03]` | C: bad length (length=3) | `None` |
| `[0x03, 0x00, 0x00, 0x04]` | C: bad length (length=4, header-only) | `None` |
| `[0x03, 0x00, 0x00, 0x06]` | C: bad length (length=6, one below RFC floor) | `None` |
| `[0x03, 0x00, 0x00, 0x07]` | D: accept (length=7, RFC floor) | `Some(TpktHeader{version:3, length:7})` |
| `[0x03, 0xFF, 0xFF, 0xFF]` | D: accept (length=65535, max) | `Some(TpktHeader{version:3, length:65535})` |

Note the guard-ordering proof embedded in class B's first case: `[0x02, 0x00, 0xFF,
0xFF]` has a length field that would decode to 65535 (a legal accept length), yet the
result is still `None` because the version guard runs first — confirming the guards
fire in `len < 4` -> `version != 0x03` -> `length < 7` order with no accidental
short-circuit past a bad version byte.

---

## Verdict

AC-184-005: **PASS** — Unit-level spot check (10 boundary vectors) and property-based
oracle cross-check (256 randomized inputs) both green. Full formal exhaustiveness over
every possible `&[u8]` input is the VP-048 Kani obligation, executed in STORY-194 (see
`AC-006-vp048-kani-skeleton.md` for the skeleton evidence anchored in this story).
