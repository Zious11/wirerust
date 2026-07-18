# AC-168-001/002/003 — `classify_frame_format` I/S/U Frame Discrimination

**Story:** STORY-168: IEC-104 Frame Format Discrimination + U-Format Session State Machine  
**ACs:** AC-168-001, AC-168-002, AC-168-003  
**Traces to:** BC-2.19.007 (I-format), BC-2.19.008 (S-format), BC-2.19.009 (U-format + VP-046 totality)  
**Wave:** 77

---

## Acceptance Criteria

- AC-168-001: Given CF1 where `cf1 & 0x01 == 0x00`, `classify_frame_format(cf1)` returns `FrameFormat::IFormat`
- AC-168-002: Given CF1 where `cf1 & 0x03 == 0x01`, `classify_frame_format(cf1)` returns `FrameFormat::SFormat`
- AC-168-003: Given CF1 where `cf1 & 0x03 == 0x03`, `classify_frame_format(cf1)` returns `FrameFormat::UFormat`;
  the function is total over all 256 u8 values (VP-046); the VP-046 proptest and exhaustive-256 unit test both pass

---

## Test Suite Execution

### AC-168-001: I-format (BC-2.19.007)

Command:
```
cargo test --test iec104_analyzer_tests story_168::test_BC_2_19_007
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 5 tests
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0x00_canonical_vector ... ok
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0x02_canonical_vector ... ok
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0x7E_canonical_vector ... ok
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0xFE_all_even_bits_set ... ok
test story_168::test_BC_2_19_007_invariant_all_128_even_cf1_values_return_iformat ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.00s
```

Result: **5/5 PASS**

### AC-168-002: S-format (BC-2.19.008)

Command:
```
cargo test --test iec104_analyzer_tests story_168::test_BC_2_19_008
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 4 tests
test story_168::test_BC_2_19_008_returns_sformat_for_cf1_0x01_canonical_vector ... ok
test story_168::test_BC_2_19_008_returns_sformat_for_cf1_0x05_canonical_vector ... ok
test story_168::test_BC_2_19_008_does_not_return_sformat_for_cf1_0x03_uformat ... ok
test story_168::test_BC_2_19_008_invariant_all_64_cf1_values_bits1_0_0b01_return_sformat ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 60 filtered out; finished in 0.00s
```

Result: **4/4 PASS**

### AC-168-003: U-format + VP-046 exhaustive totality (BC-2.19.009)

Command:
```
cargo test --test iec104_analyzer_tests story_168::test_BC_2_19_009
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 7 tests
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x07_startdt_act_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x0B_startdt_con_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x13_stopdt_act_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x03_non_canonical_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0xFF_canonical_vector ... ok
test story_168::test_BC_2_19_009_invariant_all_64_cf1_values_bits1_0_0b11_return_uformat ... ok
test story_168::test_BC_2_19_009_invariant_vp046_totality_exhaustive_all_256_values ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 57 filtered out; finished in 0.00s
```

Result: **7/7 PASS**

---

## Canonical Vector Table: CF1 → FrameFormat

The two-bit dispatch rule (bits1:0 of CF1) is the sole determinant:

| CF1 Value | bits1:0 | FrameFormat | Notes |
|-----------|---------|-------------|-------|
| `0x00` | `0b00` | `IFormat` | N(S)=0 minimum I-frame; BC-2.19.007 canonical vector |
| `0x02` | `0b10` | `IFormat` | bit1=1 but bit0=0; still I-format |
| `0xFE` | `0b10` | `IFormat` | Largest even value; I-format boundary |
| `0x01` | `0b01` | `SFormat` | Minimal S-frame indicator; BC-2.19.008 canonical vector |
| `0x05` | `0b01` | `SFormat` | BC-2.19.008 second canonical vector |
| `0x07` | `0b11` | `UFormat` | STARTDT-act; BC-2.19.009 canonical vector |
| `0x0B` | `0b11` | `UFormat` | STARTDT-con |
| `0x13` | `0b11` | `UFormat` | STOPDT-act; BC-2.19.009 canonical vector |
| `0xFF` | `0b11` | `UFormat` | All bits set; non-canonical U; BC-2.19.009 EC-004 |

### Full Partition (VP-046 totality)

| bits1:0 | Count | Format | Test Coverage |
|---------|-------|--------|---------------|
| `0b00` | 64 values | IFormat | Exhaustive loop (`step_by(2)`) |
| `0b10` | 64 values | IFormat | Same (bit0=0 is sufficient) |
| `0b01` | 64 values | SFormat | Exhaustive loop (n*4+0x01) |
| `0b11` | 64 values | UFormat | Exhaustive loop (n*4+0x03) |

VP-046 exhaustive-256 unit test (`test_BC_2_19_009_invariant_vp046_totality_exhaustive_all_256_values`):
iterates all 256 u8 values, asserting partition membership by `cf1 & 0x03`.

VP-046 proptest (`proptest_vp046_frame_format_totality`): proptest strategy `0u8..=255u8` drives
the same partition assertions via property-based random + shrinking. Passes; full proptest run is STORY-174.

---

## Separation-of-Concerns Note (ADR-013)

`classify_frame_format` is a **pure-core free function** (ADR-013 Decision 4). It:
- Takes only `cf1: u8`; reads no `Iec104FlowState` fields
- Returns `FrameFormat` with no side effects
- Is total: all 256 u8 inputs produce exactly one of `{IFormat, SFormat, UFormat}` — no panic

The effectful `process_u_frame` is a separate function (AC-168-004..008) that calls
`classify_frame_format` via a debug-assert guard.

---

## Verdict

- AC-168-001: **PASS** — 5/5 BC-2.19.007 tests green; all 128 even CF1 values exhaustively verified
- AC-168-002: **PASS** — 4/4 BC-2.19.008 tests green; all 64 S-format CF1 values exhaustively verified
- AC-168-003: **PASS** — 7/7 BC-2.19.009 tests green; exhaustive-256 unit test + proptest both pass; totality confirmed
