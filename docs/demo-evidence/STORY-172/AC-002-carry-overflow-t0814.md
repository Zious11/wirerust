# AC-172-002 — Carry Overflow: Walk-First Residual-Bound + T0814 + Dedup

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
**AC:** AC-172-002
**Traces to:** BC-2.19.025 postconditions 1–3, invariants 1–5; F-172-001
**Wave:** 81

---

## Acceptance Criterion

The carry overflow check fires at `on_data` entry on the directional carry buffer (prior
walk's residual), before the current delivery is appended and the frame-walk loop begins.
This is the walk-first residual-bound (BC-2.19.025 v1.3, one-call-shifted equivalent of a
post-walk residual bound).

- Frame extraction happens first (walk-first ordering); no aggregate-size pre-check discard
- At `on_data` entry, if `carry.len() > 255`: clear carry + resync + ONE T0814 Anomaly/Possible/Medium
- Per-direction dedup flag (`carry_overflow_reported_c2s` or `carry_overflow_reported_s2c`) set on
  first emission; subsequent overflow in the same direction triggers clear+resync only — no
  additional T0814

---

## Test Suite Execution — BC-2.19.025 v1.2 canonical vectors

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_025"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 4 tests
test story_172::test_BC_2_19_025_v12_ec001_max_conformant_partial_254_no_t0814 ... ok
test story_172::test_BC_2_19_025_v12_vector_ii_single_delivery_s2c_walk_first_no_t0814 ... ok
test story_172::test_BC_2_19_025_v12_vector_iii_defensive_overflow_dedup_c2s ... ok
test story_172::test_BC_2_19_025_v12_vector_i_split_frame_c2s_walk_first_no_t0814 ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 188 filtered out; finished in 0.00s
```

Result: **4/4 PASS**

---

## Test Coverage

| Test Name | Scenario | Expected | Result |
|-----------|----------|----------|--------|
| `test_BC_2_19_025_v12_vector_i_split_frame_c2s_walk_first_no_t0814` | C2S carry=200 bytes (first 200 of a 255-byte frame) + delivery=100 bytes → frame dispatched; residual=45 ≤ 255 | No T0814 | PASS |
| `test_BC_2_19_025_v12_vector_ii_single_delivery_s2c_walk_first_no_t0814` | S2C carry=empty + delivery=300 bytes (255-byte complete frame + 45-byte partial) → frame dispatched; residual=45 ≤ 255 | No T0814 | PASS |
| `test_BC_2_19_025_v12_vector_iii_defensive_overflow_dedup_c2s` | C2S residual=256 bytes (adversarial) → carry cleared; ONE T0814; flag set; second trip → carry cleared; NO additional T0814 (dedup) | T0814 on first; silent on second | PASS |
| `test_BC_2_19_025_v12_ec001_max_conformant_partial_254_no_t0814` | Residual=254 bytes (conformant maximum partial frame) → stashed; no T0814 (254 ≤ 255) | No T0814 | PASS |

---

## Overflow Reaction Decision Table

| carry.len() at entry | Action | Finding |
|----------------------|--------|---------|
| 0–254 | No check fires; proceed normally | None |
| 255 | No check fires (255 is not > 255; conformant: unreachable, but guarded) | None |
| 256+ (first occurrence) | Clear carry; set dedup flag; resync | ONE T0814 Anomaly/Possible/Medium |
| 256+ (second occurrence, same direction) | Clear carry; resync | None (dedup flag already set) |

---

## Walk-First Ordering Verified (Anti-Evasion Clause)

Vectors i and ii confirm that carry bytes plus delivery bytes are walked for complete
frames BEFORE any overflow check is considered for the current call's residual. The
256-byte overflow check at entry (Vector iii) fires against the PRIOR call's residual,
not the current concatenated window. This prevents an adversary from hiding valid frames
behind an aggregate-size gate (Ptacek/Newsham 1998 evasion taxonomy; RULING-DNP3-SIBLING-001).

---

## Verdict

AC-172-002: **PASS** — Walk-first residual-bound confirmed. Carry overflow triggers ONE
T0814 per direction with correct dedup flag behavior. Conformant maximum partial (254 bytes)
does not trigger overflow. Second overflow event in same direction is silently handled
(carry cleared, no additional finding).
