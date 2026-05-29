# Evidence Report — STORY-053
## ServerHello Parsing: JA3S Fingerprinting and Cipher/Version Tracking

**Story:** STORY-053  
**Wave:** 17  
**BC:** BC-2.07.002  
**Implementation strategy:** brownfield-formalization — zero production behavior change; the only
additive seam is `server_hello_seen_for_testing` (one `#[doc(hidden)] pub fn` appended to
`TlsAnalyzer`'s existing test-seam block, consistent with the pattern established in STORY-052
for `client_hello_seen_for_testing`).  
**Evidence type:** `cargo test --test tls_analyzer_tests <test_fn>` output (CLI product — VHS
not applicable; test-output capture per factory brownfield-formalization protocol).  
**Full-suite gate:** `cargo test --all-targets` — all test results OK (zero failures).

---

## AC Coverage Table

| AC | BC postcondition / invariant | Test function | Result | What it proves |
|----|------------------------------|---------------|--------|----------------|
| AC-001 | BC-2.07.002 postcondition 1 — `server_hello_seen` set to true | `test_BC_2_07_002_server_hello_seen_set_true` | PASS | Feeds a ClientHello then a ServerHello via `on_data`; asserts `server_hello_seen_for_testing` is `false` before the ServerHello and `true` after. The `#[doc(hidden)]` seam reads the field directly without relying on `done()` as a proxy. |
| AC-002 | BC-2.07.002 postcondition 2 — ServerHello `version` inserted in `version_counts` | `test_BC_2_07_002_server_version_inserted_in_version_counts` | PASS | Sends ClientHello (version 0x0303) then ServerHello (version 0x0303); asserts `version_counts[0x0303] == 1` after ClientHello and `== 2` after ServerHello. Count staying at 1 would falsify the AC. |
| AC-003 | BC-2.07.002 postcondition 3 — JA3S MD5 hex string computed and inserted in `ja3s_counts` | `test_BC_2_07_002_ja3s_hash_computed_and_inserted` | PASS | Canonical test vector: version=0x0303 (771), cipher=0x1301 (4865=TLS_AES_128_GCM_SHA256), ext=renegotiation_info (0xff01=65281). Pins JA3S string `"771,4865,65281"` → MD5 `9e36d0263f2c16df7144edfdcdd47374`. Format asserted: 32 lowercase hex chars. Any change to field order, decimal encoding, or separator invalidates this pin. |
| AC-004 | BC-2.07.002 postcondition 4 — `cipher_name(cipher)` inserted in `cipher_counts` | `test_BC_2_07_002_cipher_name_inserted_in_cipher_counts` | PASS | Known cipher 0x1301 resolves to human-readable name `"TLS_AES_128_GCM_SHA256"` (not `"0x1301"` or decimal `"4865"`). Asserts `cipher_suites["TLS_AES_128_GCM_SHA256"] == 1` via `summarize().detail`. Guards against accidental hex/decimal fallback for a well-known cipher ID. |
| AC-005 | BC-2.07.002 invariant 1 — GREASE extension IDs filtered from JA3S; cipher not filtered | `test_BC_2_07_002_ja3s_grease_ext_filtered_cipher_not_filtered` | PASS | ServerHello with GREASE ext 0x0a0a + renegotiation_info 0xff01. After filtering `(val & 0x0F0F) == 0x0A0A`, only 65281 remains. Canonical hash `9e36d0263f2c16df7144edfdcdd47374` (= MD5("771,4865,65281")) equals the no-GREASE case. `assert_ne` guards that the non-GREASE ext is not also dropped (hash must differ from no-ext MD5 `e8c07683aecf9b16e8e33f10a5161e4e`). Cipher 0x1301 passes `(0x1301 & 0x0F0F = 0x0101) != 0x0A0A` — not GREASE. |
| AC-006 | BC-2.07.002 invariant 2 — unknown cipher ID renders as `"0x{id:04x}"` lowercase hex | `test_BC_2_07_002_unknown_cipher_id_renders_as_hex_in_cipher_counts` | PASS | Cipher 0xFFFF (unassigned IANA). Asserts `cipher_suites["0xffff"] == 1`. JA3S pin: MD5("771,65535,65281") = `ba59ad1a1874a170125cfbab170feaeb` (decimal 65535 in JA3S string vs. hex "0xffff" in cipher_counts). Regression guards: keys `"65535"` and `"0xFFFF"` must be absent. |
| AC-007 | BC-2.07.002 invariant 3 — ClientHello and ServerHello versions increment `version_counts` independently | `test_BC_2_07_002_version_counts_client_and_server_versions_independent` | PASS | ClientHello version=0x0301 (TLS 1.0, 769), ServerHello version=0x0303 (TLS 1.2, 771). After both: `version_counts` has exactly 2 entries, `[0x0301] == 1` and `[0x0303] == 1`. JA3S anchor: MD5("771,4865,65281") = `9e36d0263f2c16df7144edfdcdd47374`. If `handle_server_hello` used ClientHello's version or skipped the increment, the count would be wrong. |

**Total:** 7 / 7 ACs covered. All PASS.

---

## Brownfield Seam Note

`server_hello_seen_for_testing` (tls.rs line 886) is the single additive seam introduced by this
story. It is annotated `#[doc(hidden)]` and follows the identical trust-boundary pattern used for
`client_hello_seen_for_testing` (STORY-052). The function is append-only to the existing
test-accessor block (lines 819–891); no existing method signature, field, or behavior was
modified. Production code paths are unchanged.

---

## Full Suite Gate

```
cargo test --all-targets
```

All test harnesses reported `test result: ok` with zero failures across unit tests,
integration tests (`tls_integration_tests`), and bench smoke tests.

---

## Individual Test Run Evidence

```
# AC-001
running 1 test
test test_BC_2_07_002_server_hello_seen_set_true ... ok
test result: ok. 1 passed; 0 failed

# AC-002
running 1 test
test test_BC_2_07_002_server_version_inserted_in_version_counts ... ok
test result: ok. 1 passed; 0 failed

# AC-003
running 1 test
test test_BC_2_07_002_ja3s_hash_computed_and_inserted ... ok
test result: ok. 1 passed; 0 failed

# AC-004
running 1 test
test test_BC_2_07_002_cipher_name_inserted_in_cipher_counts ... ok
test result: ok. 1 passed; 0 failed

# AC-005
running 1 test
test test_BC_2_07_002_ja3s_grease_ext_filtered_cipher_not_filtered ... ok
test result: ok. 1 passed; 0 failed

# AC-006
running 1 test
test test_BC_2_07_002_unknown_cipher_id_renders_as_hex_in_cipher_counts ... ok
test result: ok. 1 passed; 0 failed

# AC-007
running 1 test
test test_BC_2_07_002_version_counts_client_and_server_versions_independent ... ok
test result: ok. 1 passed; 0 failed
```
