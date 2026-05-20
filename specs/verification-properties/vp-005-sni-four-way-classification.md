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

- **Primary BC:** BC-2.07.014 -- SNI Containing C0/DEL Emits Anomaly/Inconclusive/Low Finding (T1027)
- **Postcondition:** Exactly one SniValue variant is returned for any input
- **Invariant:** INV-5 (SNI 4-Way Classification Ordered Match, inv-01-core-invariants.md)
- **Related BC:** BC-2.07.013 -- Clean ASCII SNI Produces No Finding
- **Related BC:** BC-2.07.015 -- Multiple control bytes in one SNI produce exactly ONE finding
- **Related BC:** BC-2.07.016 -- C0 boundary: 0x1F trips finding; 0x20 (space) does NOT
- **Related BC:** BC-2.07.017 -- Non-ASCII UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027)
- **Related BC:** BC-2.07.019 -- Non-UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027)
- **Related BC:** BC-2.07.037 -- Mixed Non-ASCII+C0 SNI fires arm 3 (NonAsciiUtf8) not arm 2

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- byte slices up to 32 bytes | All classification outcomes for bounded input lengths |
| Property testing | proptest | No -- arbitrary Vec<u8> | Full distribution including valid/invalid UTF-8 and boundary bytes |

Primary tool: Kani (for exhaustive bounded coverage of the C0 boundary at 0x1F/0x20
and the arm-3 priority case). proptest supplemental for longer inputs.

## Proof Harness Skeleton

// Real function signature (src/analyzer/tls.rs:246):
//   fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<SniValue>
//
// `extract_sni` takes a parsed extension list, not a raw byte slice.
// The 4-way classification match operates on the raw hostname bytes
// extracted from the first SNI extension entry (tls.rs:251-265).
//
// Because constructing a synthetic `TlsExtension::SNI` value requires
// tls-parser types that Kani cannot symbolically model directly, the
// proof harness targets the classification logic in isolation by
// replicating the inline match over a kani::any() byte slice. This
// tests the exact same branch ordering and guard conditions as the
// production code without the tls-parser dependency.
//
// The formal-verifier MUST verify that the harness replicates
// tls.rs:251-265 exactly before locking this VP.

```rust
// Kani harnesses -- placed in src/analyzer/tls.rs under #[cfg(kani)]
#[cfg(kani)]
mod kani_proofs {
    // Replicates the inline match at tls.rs:251-265 for symbolic testing.
    fn classify_hostname(hostname: &[u8]) -> u8 {
        // 0=Ascii, 1=AsciiWithControl, 2=NonAsciiUtf8, 3=NonUtf8
        match std::str::from_utf8(hostname) {
            Ok(s) if s.is_ascii() && !s.bytes().any(|b| b < 0x20 || b == 0x7f) => 0,
            Ok(s) if s.is_ascii() => 1,
            Ok(_) => 2,
            Err(_) => 3,
        }
    }

    #[kani::proof]
    #[kani::unwind(33)]
    fn verify_sni_exactly_one_arm_fires_kani() {
        let len: usize = kani::any();
        kani::assume(len <= 32);
        let mut hostname = vec![0u8; len];
        for b in &mut hostname {
            *b = kani::any();
        }
        // Result is 0..=3 by construction -- exactly one arm fires.
        let arm = classify_hostname(&hostname);
        assert!(arm <= 3);

        // INV-5 arm-3-priority: valid UTF-8 + non-ASCII => arm 2 only.
        if let Ok(s) = std::str::from_utf8(&hostname) {
            if !s.is_ascii() {
                assert_eq!(arm, 2);
            }
        }
    }

    #[kani::proof]
    fn verify_c0_boundary_0x1f_triggers_arm1() {
        // 0x1F (last C0 byte) must trigger AsciiWithControl (arm 1).
        let hostname: [u8; 1] = [0x1F];
        assert_eq!(classify_hostname(&hostname), 1);
    }

    #[kani::proof]
    fn verify_0x20_space_yields_arm0() {
        // 0x20 (space) is printable ASCII; must yield Ascii (arm 0).
        let hostname: [u8; 1] = [0x20];
        assert_eq!(classify_hostname(&hostname), 0);
    }
}

// proptest supplement -- arbitrary byte lengths, same invariants.
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;

    fn classify_hostname(hostname: &[u8]) -> u8 {
        match std::str::from_utf8(hostname) {
            Ok(s) if s.is_ascii() && !s.bytes().any(|b| b < 0x20 || b == 0x7f) => 0,
            Ok(s) if s.is_ascii() => 1,
            Ok(_) => 2,
            Err(_) => 3,
        }
    }

    proptest! {
        #[test]
        fn prop_sni_arm3_priority_over_arm1(hostname: Vec<u8>) {
            let arm = classify_hostname(&hostname);
            // Arm 2 (NonAsciiUtf8) fires iff valid UTF-8 + non-ASCII.
            if let Ok(s) = std::str::from_utf8(&hostname) {
                if !s.is_ascii() {
                    prop_assert_eq!(arm, 2);
                }
            }
            // Arm 1 (AsciiWithControl) fires only when all-ASCII.
            if arm == 1 {
                prop_assert!(std::str::from_utf8(&hostname)
                    .map(|s| s.is_ascii()).unwrap_or(false));
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

`src/analyzer/tls.rs:246-269` -- `fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<SniValue>`;
the 4-way classification match is at lines 251-265.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
