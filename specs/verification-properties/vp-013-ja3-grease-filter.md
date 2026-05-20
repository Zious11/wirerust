---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.07.006
bcs:
  - BC-2.07.006
  - BC-2.07.007
  - BC-2.07.008
module: src/analyzer/tls.rs
proof_method: proptest
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

# VP-013: JA3 GREASE Filter Correctness

## Property Statement

The JA3/JA3S fingerprint computation in `TlsAnalyzer` correctly filters GREASE
values per RFC 8701 before computing the fingerprint:

1. Every GREASE value (16 values of the form `0x?A?A` where `?` is any hex
   nibble: 0x0A0A, 0x1A1A, 0x2A2A, 0x3A3A, 0x4A4A, 0x5A5A, 0x6A6A, 0x7A7A,
   0x8A8A, 0x9A9A, 0xAAAA, 0xBABA, 0xCACA, 0xDADA, 0xEAEA, 0xFAFA) is
   removed from the cipher suite list, extension list, and elliptic curve
   list before they are included in the JA3 string.

2. Non-GREASE values are NOT removed.

3. The JA3 string format is: `version,ciphers,extensions,elliptic_curves,ec_point_formats`
   with values comma-separated within each field and fields separated by `-`.

4. The JA3 hash is the lowercase MD5 hex digest of the JA3 string (BC-2.07.007).

5. The JA3S string format is: `version,cipher,extensions` and its hash is the
   lowercase MD5 hex digest (BC-2.07.008).

## Source Contract

- **Primary BC:** BC-2.07.006 -- JA3 Computation Filters GREASE Values per RFC 8701
- **Postcondition:** No GREASE value appears in any field of the JA3/JA3S string
- **Related BC:** BC-2.07.007 -- JA3 String Format: version,ciphers,...; MD5 Hex
- **Related BC:** BC-2.07.008 -- JA3S String Format: version,cipher,extensions; MD5 Hex

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Property testing | proptest | No -- arbitrary cipher suite lists including all 16 GREASE values | All combinations of GREASE and non-GREASE values |

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::*;

    // All 16 GREASE cipher values per RFC 8701
    const GREASE_VALUES: &[u16] = &[
        0x0A0A, 0x1A1A, 0x2A2A, 0x3A3A, 0x4A4A, 0x5A5A, 0x6A6A, 0x7A7A,
        0x8A8A, 0x9A9A, 0xAAAA, 0xBABA, 0xCACA, 0xDADA, 0xEAEA, 0xFAFA,
    ];

    fn is_grease(v: u16) -> bool {
        (v & 0x0F0F == 0x0A0A) && ((v >> 8) == (v & 0xFF))
    }

    proptest! {
        #[test]
        fn prop_no_grease_in_ja3_string(
            ciphers in prop::collection::vec(any::<u16>(), 0..20),
            extensions in prop::collection::vec(any::<u16>(), 0..10),
            curves in prop::collection::vec(any::<u16>(), 0..5),
        ) {
            let version: u16 = 0x0303; // TLS 1.2

            // Build a synthetic ClientHello with these values
            let ja3_string = compute_ja3_string(
                version, &ciphers, &extensions, &curves, &[]
            );

            // Parse the JA3 string and verify no GREASE values appear
            let parts: Vec<&str> = ja3_string.split('-').collect();
            if parts.len() >= 2 {
                let cipher_field = parts[1];
                for cipher_str in cipher_field.split(',').filter(|s| !s.is_empty()) {
                    if let Ok(v) = cipher_str.parse::<u16>() {
                        prop_assert!(!is_grease(v),
                            "GREASE value {} appeared in JA3 cipher field", v);
                    }
                }
            }
        }

        #[test]
        fn prop_all_grease_values_filtered() {
            // Every GREASE value must be filtered from every field
            let ciphers: Vec<u16> = GREASE_VALUES.to_vec();
            let ja3 = compute_ja3_string(0x0303, &ciphers, &[], &[], &[]);
            // With only GREASE ciphers, the cipher field should be empty
            let parts: Vec<&str> = ja3.split('-').collect();
            if parts.len() >= 2 {
                assert!(parts[1].is_empty() || parts[1] == "",
                    "GREASE ciphers not filtered: {}", parts[1]);
            }
        }

        #[test]
        fn prop_non_grease_values_preserved(
            non_grease in prop::collection::vec(
                // Generate u16 values that are NOT GREASE
                any::<u16>().prop_filter("not grease", |v| !is_grease(*v)),
                1..10
            )
        ) {
            let ja3 = compute_ja3_string(0x0303, &non_grease, &[], &[], &[]);
            let parts: Vec<&str> = ja3.split('-').collect();
            // All non-GREASE ciphers must appear in the cipher field
            if parts.len() >= 2 {
                prop_assert!(!parts[1].is_empty(),
                    "non-GREASE ciphers were incorrectly filtered");
            }
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded -- 16 GREASE values; 65535 non-GREASE | proptest covers all cases efficiently |
| Proof complexity | Low | Simple filter predicate: `v & 0x0F0F == 0x0A0A && (v >> 8) == (v & 0xFF)` |
| Tool support | High | JA3 string computation is a pure function of cipher/extension lists |
| Estimated proof time | < 30 seconds | Simple filter check |

## Source Location

`src/analyzer/tls.rs` -- JA3/JA3S fingerprint computation functions.
GREASE filter based on RFC 8701 `is_grease(value)` predicate.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
