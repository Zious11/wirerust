# STORY-056 Demo Evidence Report

**Story:** STORY-056 — SNI Classification Arms 3 and 4 — Non-ASCII UTF-8 and Non-UTF-8 Byte Preservation
**Wave:** 18
**Strategy:** brownfield-formalization (zero production behavior change; tests formalize
existing behavior in `src/analyzer/tls.rs` — `bytes_to_hex`, `extract_sni` arms 3 and 4,
and the arm 3/4 finding-emission paths in `handle_client_hello`)
**Test module:** `tests/tls_analyzer_tests.rs` (AC-001..AC-010)
**Date:** 2026-05-29
**Suite result:** 888/888 PASS — `cargo test --all-targets` fully green (no failures across all modules)

---

## Per-AC Evidence Table

| AC | BC | Test Function(s) | Test Module | Result | What It Proves |
|----|----|-----------------|-------------|--------|----------------|
| AC-001 | BC-2.07.017 | `test_valid_utf8_non_ascii_sni_emits_finding`, `test_cyrillic_sni_emits_non_ascii_finding` | `tls_analyzer_tests` | PASS | `extract_sni` returns `SniValue::NonAsciiUtf8` for bytes that are valid UTF-8 but fail `is_ascii()`; `handle_client_hello` pushes exactly one `Anomaly/Inconclusive/Low` finding with the correct summary template `"TLS SNI contains non-ASCII characters (RFC 6066 requires A-labels per RFC 5890): {hostname}"`, evidence `["hex: {hex}"]`, `mitre_technique = Some("T1027")`, `direction = Some(ClientToServer)`, and `sni_counts` keyed on the raw hostname string |
| AC-002 | BC-2.07.017 | `test_cyrillic_sni_emits_non_ascii_finding` | `tls_analyzer_tests` | PASS | The finding summary contains the raw decoded UTF-8 string (Cyrillic chars present verbatim, not `\u{...}` Debug-escaped); no `escape_for_terminal` or `{:?}` formatting is applied at the analyzer layer |
| AC-003 | BC-2.07.017 | `test_emoji_sni_emits_non_ascii_finding` | `tls_analyzer_tests` | PASS | Multi-byte emoji UTF-8 sequences (e.g., U+1F608 = `[0xF0, 0x9F, 0x98, 0x88]`) trigger arm 3 — `is_ascii()` returns false on any non-ASCII codepoint, so emoji, CJK, and international labels all land in arm 3 |
| AC-004 | BC-2.07.019 | `test_non_utf8_sni_emits_finding_and_counts_under_hex_key`, `non_utf8_sni_finding_sets_mitre_t1027` | `tls_analyzer_tests` | PASS | `extract_sni` returns `SniValue::NonUtf8 { lossy, hex }` for bytes that fail `str::from_utf8`; arm 4 in `handle_client_hello` pushes exactly one `Anomaly/Inconclusive/Low` finding with summary `"TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}"`, evidence `["hex: {hex}"]`, `mitre_technique = Some("T1027")`, `direction = Some(ClientToServer)` |
| AC-005 | BC-2.07.019 | `test_non_utf8_sni_emits_finding_and_counts_under_hex_key` | `tls_analyzer_tests` | PASS | The `sni_counts` key for non-UTF-8 SNI is `"<non-utf8:{hex}>"` — the hex-tagged format, not the lossy string; two distinct invalid byte sequences that share the same `from_utf8_lossy` output map to different `sni_counts` entries |
| AC-006 | BC-2.07.020 | `test_non_utf8_sni_preserves_raw_bytes_in_summary` | `tls_analyzer_tests` | PASS | `finding.summary` holds the `String::from_utf8_lossy` output (U+FFFD replacements for invalid bytes); `finding.evidence[0]` holds `"hex: {hex}"` with lossless lowercase hex; neither field has passed through `escape_for_terminal` or `{:?}` Debug format |
| AC-007 | BC-2.07.020 | `test_arm4_hex_evidence_is_pure_ascii` | `tls_analyzer_tests` | PASS | The hex field produced by `bytes_to_hex` contains only `[0-9a-f]` characters — it is always pure ASCII and requires no escaping; the reporter layer (ADR 0003) is solely responsible for terminal escaping at render time |
| AC-008 | BC-2.07.021 | `test_cyrillic_sni_emits_non_ascii_finding` | `tls_analyzer_tests` | PASS | For arm 3 (NonAsciiUtf8), `finding.summary` contains the decoded UTF-8 hostname with all non-ASCII codepoints intact (raw Cyrillic characters, not `\u{NNNN}` sequences); `finding.evidence[0]` = `"hex: {hex}"` with lossless lowercase hex |
| AC-009 | BC-2.07.037 | `test_c0_plus_non_ascii_fires_arm3_not_arm2` | `tls_analyzer_tests` | PASS | SNI bytes that are valid UTF-8 but contain both C0/DEL control bytes AND non-ASCII codepoints (e.g., `b"caf\x01\xc3\xa9"`) route to arm 3 — `is_ascii()` returns false first, so `contains_c0_or_del` is never evaluated; finding summary says "non-ASCII characters", not "control bytes" |
| AC-010 | BC-2.07.037 | `test_is_ascii_gate_routes_arm2_vs_arm3` | `tls_analyzer_tests` | PASS | The `is_ascii()` predicate is the decisive gate: a single non-ASCII codepoint causes `is_ascii()` to return false and routes the string to arm 3 before `contains_c0_or_del` is checked; arm evaluation is strictly top-down and the test explicitly confirms both directions of the gate |

---

## Test Run Output

### AC-001 — `test_valid_utf8_non_ascii_sni_emits_finding`

```
running 1 test
test test_valid_utf8_non_ascii_sni_emits_finding ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-001 / AC-002 / AC-008 — `test_cyrillic_sni_emits_non_ascii_finding`

```
running 1 test
test test_cyrillic_sni_emits_non_ascii_finding ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-003 — `test_emoji_sni_emits_non_ascii_finding`

```
running 1 test
test test_emoji_sni_emits_non_ascii_finding ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-004 / AC-005 — `test_non_utf8_sni_emits_finding_and_counts_under_hex_key`

```
running 1 test
test test_non_utf8_sni_emits_finding_and_counts_under_hex_key ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-004 — `non_utf8_sni_finding_sets_mitre_t1027`

```
running 1 test
test non_utf8_sni_finding_sets_mitre_t1027 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-006 — `test_non_utf8_sni_preserves_raw_bytes_in_summary`

```
running 1 test
test test_non_utf8_sni_preserves_raw_bytes_in_summary ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-007 — `test_arm4_hex_evidence_is_pure_ascii`

```
running 1 test
test test_arm4_hex_evidence_is_pure_ascii ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-009 — `test_c0_plus_non_ascii_fires_arm3_not_arm2`

```
running 1 test
test test_c0_plus_non_ascii_fires_arm3_not_arm2 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

### AC-010 — `test_is_ascii_gate_routes_arm2_vs_arm3`

```
running 1 test
test test_is_ascii_gate_routes_arm2_vs_arm3 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
```

---

## Recording Method

**Type:** text transcript (brownfield test-formalization; no CLI/UI behavior change)
VHS recordings are not applicable — this story formalizes existing internal analyzer logic,
not an observable CLI command or UI flow. Evidence is captured via per-test `cargo test --exact`
invocations against the Rust test harness.

---

## Coverage Summary

- **ACs covered:** 10 / 10 (100%)
- **Unique test functions exercised:** 9
  - `test_valid_utf8_non_ascii_sni_emits_finding` (covers AC-001)
  - `test_cyrillic_sni_emits_non_ascii_finding` (covers AC-001, AC-002, AC-008)
  - `test_emoji_sni_emits_non_ascii_finding` (covers AC-003)
  - `test_non_utf8_sni_emits_finding_and_counts_under_hex_key` (covers AC-004, AC-005)
  - `non_utf8_sni_finding_sets_mitre_t1027` (covers AC-004)
  - `test_non_utf8_sni_preserves_raw_bytes_in_summary` (covers AC-006)
  - `test_arm4_hex_evidence_is_pure_ascii` (covers AC-007)
  - `test_c0_plus_non_ascii_fires_arm3_not_arm2` (covers AC-009)
  - `test_is_ascii_gate_routes_arm2_vs_arm3` (covers AC-010)
- **BCs traced:** BC-2.07.017, BC-2.07.019, BC-2.07.020, BC-2.07.021, BC-2.07.037
- **Full suite:** `cargo test --all-targets` — 888 passed, 0 failed across all modules
