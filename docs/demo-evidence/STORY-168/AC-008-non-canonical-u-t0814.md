# AC-168-008 — Non-Canonical U-Frame CF1 Emits T0814 (CVE-2026-1773)

**Story:** STORY-168: IEC-104 Frame Format Discrimination + U-Format Session State Machine  
**AC:** AC-168-008  
**Traces to:** BC-2.19.014 postconditions 1–2 and invariant 1  
**Wave:** 77

---

## Acceptance Criterion

- Given a U-format frame (CF1 bits1:0 = `0b11`) where CF1 does not match any of:
  STARTDT-act (`0x07`), STARTDT-con (`0x0B`), STOPDT-act (`0x13`), STOPDT-con (`0x23`),
  TESTFR-act (`0x43`), TESTFR-con (`0x83`)
- When `process_u_frame(&mut state, cf1)` is called
- Then a T0814 "Denial of Service" finding is emitted with `Verdict::Possible`
- Session state is NOT advanced (invariant 1 — fail-closed)
- This matches the CVE-2026-1773 non-canonical U-frame attack vector

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests story_168::test_BC_2_19_014
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 6 tests
test story_168::test_BC_2_19_014_non_canonical_cf1_0x03_emits_t0814_possible ... ok
test story_168::test_BC_2_19_014_non_canonical_cf1_0x0F_emits_t0814_possible_canonical_vector ... ok
test story_168::test_BC_2_19_014_non_canonical_cf1_0x1B_emits_t0814_possible ... ok
test story_168::test_BC_2_19_014_non_canonical_cf1_0xFF_emits_t0814_possible_canonical_vector ... ok
test story_168::test_BC_2_19_014_negative_canonical_cf1_values_do_not_emit_t0814 ... ok
test story_168::test_BC_2_19_014_invariant_non_canonical_u_frame_does_not_advance_session_state ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 0.00s
```

Result: **6/6 PASS**

---

## Test Coverage

| Test Name | CF1 | bits1:0 | Finding | Verdict | Condition Exercised |
|-----------|-----|---------|---------|---------|---------------------|
| `test_BC_2_19_014_non_canonical_cf1_0x03_emits_t0814_possible` | `0x03` | `0b11` | T0814 | Possible | Minimal non-canonical U-frame (BC-2.19.014 PC1) |
| `test_BC_2_19_014_non_canonical_cf1_0x0F_emits_t0814_possible_canonical_vector` | `0x0F` | `0b11` | T0814 | Possible | BC-2.19.014 canonical vector |
| `test_BC_2_19_014_non_canonical_cf1_0x1B_emits_t0814_possible` | `0x1B` | `0b11` | T0814 | Possible | Additional non-canonical value |
| `test_BC_2_19_014_non_canonical_cf1_0xFF_emits_t0814_possible_canonical_vector` | `0xFF` | `0b11` | T0814 | Possible | BC-2.19.014 canonical vector (all bits set) |
| `test_BC_2_19_014_negative_canonical_cf1_values_do_not_emit_t0814` | 0x07, 0x0B, 0x13, 0x23, 0x43, 0x83 | `0b11` | None | — | Negative: canonical U-frames must NOT trigger T0814 |
| `test_BC_2_19_014_invariant_non_canonical_u_frame_does_not_advance_session_state` | `0x0F` | `0b11` | T0814 | Possible | Fail-closed: session state not modified (BC-2.19.014 invariant 1) |

---

## Finding Content Demonstration

### Non-Canonical U-Frame CF1=0x0F (BC-2.19.014 canonical vector)

```
Precondition:  state.session_started == false   (cold flow)
Input:         CF1=0x0F (bits1:0=0b11 → UFormat; not in canonical set)
Call:          process_u_frame(&mut state, 0x0F)

Postcondition: state.session_started == false   (unchanged — fail-closed; BC-2.19.014 invariant 1)

Finding emitted:
  category:          ThreatCategory::Anomaly
  verdict:           Verdict::Possible
  confidence:        Confidence::Medium
  mitre_techniques:  ["T0814"]
  summary:           "IEC-104 non-canonical U-frame CF1=0x0F: CF1 bits1:0=0b11 but not in
                      canonical set {0x07,0x0B,0x13,0x23,0x43,0x83} — potential
                      CVE-2026-1773 denial-of-service attack (T0814; BC-2.19.014)"
  evidence:          ["CF1=0x0F not in canonical U-frame set
                       {STARTDT-act=0x07, STARTDT-con=0x0B, STOPDT-act=0x13,
                       STOPDT-con=0x23, TESTFR-act=0x43, TESTFR-con=0x83}"]
```

### Non-Canonical U-Frame CF1=0xFF (all bits set)

```
Precondition:  state.session_started == false
Input:         CF1=0xFF (bits1:0=0b11 → UFormat; non-canonical)
Call:          process_u_frame(&mut state, 0xFF)

Postcondition: state.session_started == false   (unchanged)

Finding emitted:
  category:          ThreatCategory::Anomaly
  verdict:           Verdict::Possible
  mitre_techniques:  ["T0814"]
  summary:           "IEC-104 non-canonical U-frame CF1=0xFF: ..."
```

### Canonical U-Frames Do NOT Emit T0814 (negative test)

The six canonical CF1 values are dispatched to their session-SM handlers and never reach
the T0814 branch. The negative test exercises all six:

| CF1 | Command | Expected |
|-----|---------|---------|
| `0x07` | STARTDT-act | session_started=true, no finding |
| `0x0B` | STARTDT-con | session_started=true, no finding |
| `0x13` | STOPDT-act | T0881 (not T0814) |
| `0x23` | STOPDT-con | session_started=false, no finding |
| `0x43` | TESTFR-act | no finding |
| `0x83` | TESTFR-con | no finding |

---

## CVE-2026-1773 Attack Context

Non-canonical U-frame CF1 values (bits1:0=0b11, not in the six canonical commands) have been
observed in CVE-2026-1773 denial-of-service attack traffic against IEC-104 devices. Some
implementations pass unrecognized U-frames to internal handlers without validation, causing
resource exhaustion or state corruption.

wirerust detects this pattern with `Verdict::Possible` (confidence Medium) and `ThreatCategory::Anomaly`.
The fail-closed rule (session state not advanced) ensures the attack does not silently advance
the session model.

---

## Verdict

AC-168-008: **PASS** — 6/6 BC-2.19.014 tests green; T0814 emitted for all four non-canonical CF1 vectors; session state not advanced (fail-closed invariant); canonical U-frames correctly excluded from T0814 path.
