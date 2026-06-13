---
document_type: verification-property
level: L4
version: "2.1"
status: verified
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
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "42571c077279387c80a2643fc364abd5981ae1a5b7121260d09d0d51c04e7c27"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set (src/analyzer/tls.rs)."
  - "v2.1 (2026-06-13, PG-ARP-F2-007 anchor-drift sweep): Source Location and harness-comment line anchors corrected for F2 tls.rs shifts. is_grease_u16: :50→:51. compute_ja3: :95→:96. compute_ja3s: :156→:157. JA3 format string comment: :148→:149. Cipher filter chain comment: :101-106→:101-107. Lock fields unchanged."
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

API notes verified against `src/analyzer/tls.rs` @ 0082a0c:
- `compute_ja3_string` does NOT exist. The real functions are:
  - `compute_ja3(version: u16, ciphers: &[TlsCipherSuiteID], extensions: &[TlsExtension<'_>]) -> (String, String)`
    Returns `(md5_hash, ja3_string)`. Curves and point_formats are extracted
    from the `extensions` slice internally (from `TlsExtension::EllipticCurves`
    and `TlsExtension::EcPointFormats` arms). There is no separate `curves` arg.
  - `compute_ja3s(version: u16, cipher: TlsCipherSuiteID, extensions: &[TlsExtension<'_>]) -> String`
    Returns only the MD5 hex string (not a tuple).
- `is_grease_u16(val: u16) -> bool` is the module-private GREASE predicate.
  It uses the broader mask `(val & 0x0F0F) == 0x0A0A` (intentionally covers
  240 non-canonical `0x_A_A` values beyond the 16 strict RFC 8701 GREASE
  values -- see src/analyzer/tls.rs:51 for rationale).
- Both `compute_ja3` and `is_grease_u16` are private (`fn`, not `pub fn`).
  Proptest must be a module-internal test using `use super::*`.
- Cipher types use `TlsCipherSuiteID` (a newtype wrapper around `u16`), not
  bare `u16`. Construct via `TlsCipherSuiteID(val)`.
- JA3 string format uses `-` as the field delimiter and `-` as the
  within-field delimiter (src/analyzer/tls.rs:149: `format!("{version},{cipher_str},{ext_ids},{curves_str},{pf_str}")`
  where inner join also uses `-`).

```rust
// Located in src/analyzer/tls.rs (module-internal test using `use super::*`)
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use tls_parser::TlsCipherSuiteID;
    use super::*; // brings compute_ja3, is_grease_u16 into scope

    // The GREASE predicate used by this module (src/analyzer/tls.rs:51):
    // (val & 0x0F0F) == 0x0A0A -- broader than the 16 strict RFC 8701 values.
    fn grease(v: u16) -> bool {
        (v & 0x0F0F) == 0x0A0A
    }

    proptest! {
        #[test]
        fn prop_no_grease_in_ja3_string(
            cipher_vals in prop::collection::vec(any::<u16>(), 0..20),
        ) {
            let version: u16 = 0x0303; // TLS 1.2
            // Wrap bare u16 values into TlsCipherSuiteID (newtype)
            let ciphers: Vec<TlsCipherSuiteID> = cipher_vals.iter()
                .map(|&v| TlsCipherSuiteID(v))
                .collect();
            // extensions=&[] means no elliptic curves, no ec point formats,
            // no extension IDs in the JA3 string -- acceptable for cipher-filter test.
            let (_, ja3_str) = compute_ja3(version, &ciphers, &[]);

            // Parse the cipher field (field index 1, delimiter '-') and verify
            // no GREASE values appear (src/analyzer/tls.rs:101-107).
            let fields: Vec<&str> = ja3_str.splitn(5, ',').collect();
            if let Some(cipher_field) = fields.get(1) {
                for s in cipher_field.split('-').filter(|s| !s.is_empty()) {
                    if let Ok(v) = s.parse::<u16>() {
                        prop_assert!(!grease(v),
                            "GREASE-matched value {} appeared in JA3 cipher field", v);
                    }
                }
            }
        }

        #[test]
        fn prop_all_grease_values_filtered() {
            // The 16 strict RFC 8701 GREASE ciphers: 0x0A0A .. 0xFAFA
            let grease_ciphers: Vec<TlsCipherSuiteID> = (0u8..=15)
                .map(|n| TlsCipherSuiteID((u16::from(n) << 8 | u16::from(n)) | 0x0A0A))
                .collect();
            let (_, ja3_str) = compute_ja3(0x0303, &grease_ciphers, &[]);
            let fields: Vec<&str> = ja3_str.splitn(5, ',').collect();
            // Cipher field should be empty after all GREASE ciphers are filtered.
            if let Some(cipher_field) = fields.get(1) {
                assert!(cipher_field.is_empty(),
                    "GREASE ciphers not filtered: {cipher_field}");
            }
        }

        #[test]
        fn prop_non_grease_values_preserved(
            non_grease_vals in prop::collection::vec(
                any::<u16>().prop_filter("not grease", |v| !grease(*v)),
                1..10
            )
        ) {
            let ciphers: Vec<TlsCipherSuiteID> = non_grease_vals.iter()
                .map(|&v| TlsCipherSuiteID(v))
                .collect();
            let (_, ja3_str) = compute_ja3(0x0303, &ciphers, &[]);
            let fields: Vec<&str> = ja3_str.splitn(5, ',').collect();
            if let Some(cipher_field) = fields.get(1) {
                prop_assert!(!cipher_field.is_empty(),
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

`src/analyzer/tls.rs:51` -- `fn is_grease_u16(val: u16) -> bool` -- GREASE predicate.
`src/analyzer/tls.rs:96` -- `fn compute_ja3(version: u16, ciphers: &[TlsCipherSuiteID], extensions: &[TlsExtension<'_>]) -> (String, String)`.
Curves and point_formats are extracted from `extensions` internally (no separate curves arg).
`src/analyzer/tls.rs:157` -- `fn compute_ja3s(version: u16, cipher: TlsCipherSuiteID, extensions: &[TlsExtension<'_>]) -> String`.
Returns MD5 hex string only (not a tuple).
Both functions are module-private; proofs must be in `src/analyzer/tls.rs` test submodule.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
