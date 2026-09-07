# AC-185-009 — `protocol_id` Extraction Is a Total, Uninterpreted Identity Mapping

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-009
**Traces to:** BC-2.20.012 postconditions 1–3
**Wave:** 88

---

## Acceptance Criterion

- Given the DT-with-non-empty-payload branch (BC-2.20.009 preconditions hold)
- When `parse_cotp_header` extracts the protocol-ID byte
- Then for any `u8` value `b`, the result is `protocol_id: Some(b)` — no branch, match
  arm, or conditional inside `parse_cotp_header` ever compares `b` against `0x32`,
  `0x72`, or any other specific value (traces to BC-2.20.012 postcondition 2)
- `src/analyzer/iso_on_tcp.rs` contains no reference to the literals `0x32`/`0x72` nor
  the strings "S7comm"/"S7comm-plus" anywhere in its parsing logic (traces to
  BC-2.20.012 postcondition 3)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_012
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 2 tests
test story_185::test_BC_2_20_012_protocol_id_extraction_totality ... ok
test story_185::test_BC_2_20_012_static_regression_guard_no_hardcoded_protocol_literals ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.00s
```

Result: **2/2 PASS**

---

## Test Coverage

| Test Name | Coverage | Result |
|-----------|----------|--------|
| `test_BC_2_20_012_protocol_id_extraction_totality` | Exhaustive `#[test]` loop over all 256 `u8` values (`0u8..=255u8`, not a random/probabilistic sample) — asserts `protocol_id == Some(byte)` for every value while `tpdu_type` and `payload_offset` remain constant | PASS |
| `test_BC_2_20_012_static_regression_guard_no_hardcoded_protocol_literals` | Whole-file substring check on `src/analyzer/iso_on_tcp.rs`, asserting zero occurrences of the literals `0x32` and `0x72` (constructed via string concatenation in the test itself, so the test file never contains those literal substrings either) | PASS |

---

## Totality and Architectural-Boundary Demonstration

Key assertions verified:
- **Full 256-value coverage (not sampled):** the totality test iterates every possible
  `u8` value exhaustively, proving the identity mapping holds for the entire domain,
  including the two byte values (`0x32` classic S7comm, `0x72` S7comm-plus) a
  downstream SS-21 disambiguation table would treat specially — neither value is ever
  written as a literal token in the test file; both arise only at runtime from the loop
  bound.
- **Static source guard:** `src/analyzer/iso_on_tcp.rs` is grepped (via
  `std::fs::read_to_string` inside the test, not an external shell grep) for the
  literals `0x32`/`0x72` and asserted absent — a whole-file check, not scoped to exclude
  doc comments, since a stray literal even in a comment would itself be exactly the kind
  of architectural drift this guard exists to catch (BC-2.20.012 postcondition 3,
  ADR-014's SS-20/SS-21 boundary).

Manual inspection of `parse_cotp_header` (`src/analyzer/iso_on_tcp.rs`, lines 244–279)
and the `CotpHeader`/`CotpTpduType` definitions (lines 167–201) confirms the only
comparisons made against `tpdu_code & 0xF0` are the three TPDU-type high-nibble masks
(`0xE0`, `0xD0`, `0xF0`); the protocol-ID byte itself (`tpkt_payload[payload_offset]`)
is assigned straight into `Some(...)` with no comparison of any kind — consistent with
the static regression-guard test's passing result above.

---

## Verdict

AC-185-009: **PASS** — Both BC-2.20.012 tests green; the protocol-ID extraction proven
total over the full 256-value `u8` domain, and the SS-20/SS-21 architectural boundary
(no hardcoded protocol literals) verified by static source guard.
