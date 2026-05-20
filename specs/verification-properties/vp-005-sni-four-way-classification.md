---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.07.014
bcs:
  - BC-2.07.013
  - BC-2.07.014
  - BC-2.07.015
  - BC-2.07.016
  - BC-2.07.017
  - BC-2.07.019
  - BC-2.07.037
module: src/analyzer/tls.rs
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-005: SNI 4-Way Ordered Classification

## Property Statement

The `extract_sni` function classifies any byte slice into exactly one of four
`SniValue` variants. The classification arms are evaluated top-down with strict
precedence:

1. `from_utf8 OK` AND `is_ascii()` AND NOT `contains_c0_or_del()` => `SniValue::Ascii`
   (no finding emitted; BC-2.07.013)
2. `from_utf8 OK` AND `is_ascii()` AND `contains_c0_or_del()` => `SniValue::AsciiWithControl`
   (T1027 finding emitted; BC-2.07.014)
3. `from_utf8 OK` AND NOT `is_ascii()` => `SniValue::NonAsciiUtf8`
   (T1027 finding emitted; BC-2.07.017)
4. `from_utf8 Err` => `SniValue::NonUtf8`
   (T1027 finding emitted; BC-2.07.019)

Critical boundary case (INV-5 / BC-2.07.037): When a byte sequence is valid UTF-8,
is NOT all-ASCII (contains multi-byte codepoints), AND also contains C0/DEL control
bytes, arm 3 fires (NonAsciiUtf8) and NOT arm 2 (AsciiWithControl). This is because
`is_ascii()` is false when any non-ASCII character is present, so arm 2's `is_ascii()`
guard is not satisfied.

The result is unique for any given input: no byte slice can match more than one arm.

## Source Contract

- **Primary BC:** BC-2.07.014 -- SNI Containing C0/DEL Emits Anomaly/Likely/High Finding (T1027)
- **Postcondition:** Exactly one SniValue variant is returned for any input
- **Invariant:** INV-5 (SNI 4-Way Classification Ordered Match, inv-01-core-invariants.md)
- **Related BC:** BC-2.07.013 -- Clean ASCII SNI Produces No Finding
- **Related BC:** BC-2.07.015 -- Multiple control bytes in one SNI produce exactly ONE finding
- **Related BC:** BC-2.07.016 -- C0 boundary: 0x1F trips finding; 0x20 (space) does NOT
- **Related BC:** BC-2.07.017 -- Non-ASCII UTF-8 SNI Emits Anomaly/Likely/High Finding (T1027)
- **Related BC:** BC-2.07.019 -- Non-UTF-8 SNI Emits Anomaly/Likely/High Finding (T1027)
- **Related BC:** BC-2.07.037 -- Mixed Non-ASCII+C0 SNI fires arm 3 (NonAsciiUtf8) not arm 2

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- byte slices up to 32 bytes | All classification outcomes for bounded input lengths |
| Property testing | proptest | No -- arbitrary Vec<u8> | Full distribution including valid/invalid UTF-8 and boundary bytes |

Primary tool: Kani (for exhaustive bounded coverage of the C0 boundary at 0x1F/0x20
and the arm-3 priority case). proptest supplemental for longer inputs.

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(33)]
    fn verify_sni_exactly_one_arm_fires_kani() {
        let len: usize = kani::any();
        kani::assume(len <= 32);
        let mut bytes = vec![0u8; len];
        for b in &mut bytes {
            *b = kani::any();
        }

        let result = extract_sni(&bytes);

        // Exactly one variant fires -- the type system guarantees this,
        // but verify the arm-3 priority explicitly:
        match &result {
            SniValue::AsciiWithControl(_) => {
                // If arm 2 fired, the bytes must be valid UTF-8 AND all-ASCII
                // (if non-ASCII were present, arm 3 would have fired instead)
                let s = std::str::from_utf8(&bytes);
                assert!(s.is_ok());
                assert!(s.unwrap().is_ascii());
            }
            SniValue::NonAsciiUtf8(_) => {
                // Arm 3: valid UTF-8 but NOT all-ASCII
                let s = std::str::from_utf8(&bytes);
                assert!(s.is_ok());
                assert!(!s.unwrap().is_ascii());
            }
            _ => {}
        }
    }

    #[kani::proof]
    fn verify_c0_boundary_0x1f_triggers_arm2() {
        // 0x1F is the last C0 control byte; must trigger AsciiWithControl
        let bytes: [u8; 1] = [0x1F];
        let result = extract_sni(&bytes);
        assert!(matches!(result, SniValue::AsciiWithControl(_)));
    }

    #[kani::proof]
    fn verify_0x20_space_does_not_trigger_arm2() {
        // 0x20 (space) is printable ASCII; must yield Ascii (arm 1)
        let bytes: [u8; 1] = [0x20];
        let result = extract_sni(&bytes);
        assert!(matches!(result, SniValue::Ascii(_)));
    }
}

// proptest supplement
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn prop_sni_arm3_priority_over_arm2(bytes: Vec<u8>) {
            let result = extract_sni(&bytes);
            // If arm 3 fired (NonAsciiUtf8), the bytes must NOT be all-ASCII
            if let SniValue::NonAsciiUtf8(_) = &result {
                let s = std::str::from_utf8(&bytes);
                prop_assert!(s.is_ok());
                prop_assert!(!s.unwrap().is_ascii());
            }
            // If arm 2 fired (AsciiWithControl), bytes must be all-ASCII
            if let SniValue::AsciiWithControl(_) = &result {
                let s = std::str::from_utf8(&bytes);
                prop_assert!(s.is_ok());
                prop_assert!(s.unwrap().is_ascii());
            }
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded for Kani; unbounded for proptest | Kani: 32-byte bound is sufficient to cover all branch conditions |
| Proof complexity | Medium | UTF-8 validity check adds complexity; `from_utf8` is stdlib, not verified separately |
| Tool support | High | `extract_sni` is a pure function; no I/O or global state |
| Estimated proof time | 5-10 minutes (Kani); fast (proptest) | UTF-8 validity is the main computational cost |

## Source Location

`src/analyzer/tls.rs:219-242` -- the match block in `extract_sni`.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
