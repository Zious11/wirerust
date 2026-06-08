# STORY-054 Demo Evidence Report

**Story:** STORY-054 — Cipher and Protocol Weakness Findings — Weak Ciphers, Deprecated SSL Versions, and Baseline Zero-Finding
**Wave:** 18
**Strategy:** brownfield-formalization (zero production behavior change; tests formalize
existing behavior in `src/analyzer/tls.rs` — `is_weak_cipher`, `is_weak_server_cipher`,
`cipher_name`, and the finding-emission paths in `handle_client_hello`/`handle_server_hello`)
**Test modules:** `tests/tls_analyzer_tests.rs` (AC-001..005, AC-007..013); `tests/tls_integration_tests.rs` (AC-006)
**Date:** 2026-05-29
**Suite result:** 885/885 PASS — `cargo test --all-targets` fully green (no failures across all modules)

---

## Per-AC Evidence Table

| AC | BC | Test Function(s) | Test Module | Result | What It Proves |
|----|----|-----------------|-------------|--------|----------------|
| AC-001 | BC-2.07.009 | `test_weak_cipher_finding_client` | `tls_analyzer_tests` | PASS | `handle_client_hello` with a NULL/ANON/EXPORT cipher in `ch.ciphers` emits exactly one `Anomaly/Likely/High` finding with `summary = "ClientHello offers weak cipher suites (NULL/anonymous/export)"`, `direction = ClientToServer`, and one evidence entry per weak cipher name |
| AC-002 | BC-2.07.009 | `test_weak_cipher_finding_client` | `tls_analyzer_tests` | PASS | Sending multiple weak ciphers (the test exercises the multi-weak path) produces exactly ONE finding, not one per weak cipher — the evidence vec accumulates names, the finding is singular |
| AC-003 | BC-2.07.009 | `test_normal_handshake_no_findings` | `tls_analyzer_tests` | PASS | GREASE-valued and unknown cipher IDs do not trigger the weak-cipher finding: `TlsCipherSuite::from_id` returns `None` for GREASE values, and `is_weak_cipher(None)` returns false; zero findings emitted for a clean modern handshake |
| AC-004 | BC-2.07.010 | `test_weak_cipher_finding_server` | `tls_analyzer_tests` | PASS | `handle_server_hello` with a NULL/ANON/EXPORT/RC4 cipher selected emits exactly one `Anomaly/Likely/Medium` finding with `summary = "ServerHello selected weak cipher suite ({name})"`, `direction = ServerToClient`, and `evidence = ["Selected cipher: {name} (0x{id:04x})"]` |
| AC-005 | BC-2.07.010 | `test_weak_cipher_finding_server` | `tls_analyzer_tests` | PASS | Test uses an RC4 cipher (TLS_RSA_WITH_RC4_128_MD5); RC4 triggers `is_weak_server_cipher` but not `is_weak_cipher`, confirming the server-side check is a strict superset; finding confidence is `Medium` |
| AC-006 | BC-2.07.011 | `test_ssl30_pcap_generates_findings` | `tls_integration_tests` | PASS | Integration path: a synthetically constructed SSL 3.0 ClientHello (version `0x0300`) causes `handle_client_hello` to emit an `Anomaly/Likely/High` finding with summary containing "RFC 7568 prohibits SSLv3" and version name "SSL 3.0" |
| AC-007 | BC-2.07.011 | `test_client_tls10_no_deprecated_finding` | `tls_analyzer_tests` | PASS | TLS 1.0 version `0x0301` does NOT trigger the deprecated-protocol finding; threshold is strictly `<= 0x0300`; zero findings for a TLS 1.0 ClientHello with no weak ciphers; the summary string "RFC 7568" is a mandatory substring when the finding does fire |
| AC-008 | BC-2.07.011 | `test_ssl30_client_weak_cipher_both_findings` | `tls_analyzer_tests` | PASS | A ClientHello with SSL 3.0 version AND a weak cipher fires both the deprecated-protocol finding and the weak-cipher finding independently; `all_findings.len() == 2`, each finding distinct |
| AC-009 | BC-2.07.012 | `test_server_ssl30_deprecated_finding` | `tls_analyzer_tests` | PASS | `handle_server_hello` with version `0x0300` emits one `Anomaly/Likely/High` finding with `summary = "ServerHello negotiated deprecated protocol (SSL 3.0, RFC 7568 prohibits SSLv3)"`, `direction = ServerToClient` |
| AC-010 | BC-2.07.012 | `test_client_and_server_ssl30_distinct_directions` | `tls_analyzer_tests` | PASS | When both ClientHello and ServerHello use SSL 3.0, two separate deprecated-protocol findings are emitted: one with `direction = ClientToServer` and one with `direction = ServerToClient`; TLS 1.0 (`0x0301`) on either side does not trigger |
| AC-011 | BC-2.07.030 | `test_normal_handshake_no_findings` | `tls_analyzer_tests` | PASS | A clean TLS handshake (modern version > 0x0300, no weak ciphers, clean ASCII SNI) produces `all_findings.len() == 0`, `handshakes_seen == 1`, all count maps have exactly one entry, `parse_errors == 0` |
| AC-012 | BC-2.07.036 | `test_cipher_name_unknown_hex_lowercase` | `tls_analyzer_tests` | PASS | `cipher_name(id)` for an unrecognized ID returns `format!("0x{:04x}", id.0)` — a 6-character lowercase string with "0x" prefix and 4 zero-padded hex digits (e.g., `"0x1234"`, `"0xaaaa"`) |
| AC-013 | BC-2.07.036 | `test_cipher_name_recognized_and_ffff` | `tls_analyzer_tests` | PASS | For recognized cipher IDs, `cipher_name` returns the IANA canonical name (e.g., `"TLS_AES_256_GCM_SHA384"`) without any "0x" prefix; for ID `0xFFFF` (unrecognized), returns `"0xffff"` (lowercase) |

---

## Test Run Output

### tls_analyzer_tests (AC-001..005, AC-007..013)

```
running 1 test
test test_weak_cipher_finding_client ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_weak_cipher_finding_server ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_client_tls10_no_deprecated_finding ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_ssl30_client_weak_cipher_both_findings ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_server_ssl30_deprecated_finding ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_client_and_server_ssl30_distinct_directions ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_normal_handshake_no_findings ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_cipher_name_unknown_hex_lowercase ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

```
running 1 test
test test_cipher_name_recognized_and_ffff ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

### tls_integration_tests (AC-006)

```
running 1 test
test test_ssl30_pcap_generates_findings ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

---

## Recording Method

**Type:** text transcript (brownfield test-formalization; no CLI/UI behavior change)
VHS recordings are not applicable — this story formalizes existing internal analyzer logic,
not an observable CLI command or UI flow. Evidence is captured via per-test `cargo test --exact`
invocations against the Rust test harness.

---

## Coverage Summary

- **ACs covered:** 13 / 13 (100%)
- **Unique test functions exercised:** 10
  - `test_weak_cipher_finding_client` (covers AC-001, AC-002)
  - `test_weak_cipher_finding_server` (covers AC-004, AC-005)
  - `test_ssl30_pcap_generates_findings` (covers AC-006; integration test)
  - `test_client_tls10_no_deprecated_finding` (covers AC-007)
  - `test_ssl30_client_weak_cipher_both_findings` (covers AC-008)
  - `test_server_ssl30_deprecated_finding` (covers AC-009)
  - `test_client_and_server_ssl30_distinct_directions` (covers AC-010)
  - `test_normal_handshake_no_findings` (covers AC-003, AC-011)
  - `test_cipher_name_unknown_hex_lowercase` (covers AC-012)
  - `test_cipher_name_recognized_and_ffff` (covers AC-013)
- **BCs traced:** BC-2.07.009, BC-2.07.010, BC-2.07.011, BC-2.07.012, BC-2.07.030, BC-2.07.036
- **Full suite:** `cargo test --all-targets` — 885 passed, 0 failed across all modules
