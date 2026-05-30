# Demo Evidence Report — STORY-076

**Story:** STORY-076 — JsonReporter: Structure, skipped_packets, and RFC 8259 Byte Handling
**Epic:** E-8 — Reporter Pipeline
**Wave:** 20
**Mode:** brownfield-formalization (zero src/ changes)
**Recording method:** text transcript — this story formalizes existing internal reporter logic, not an observable CLI/UI flow. VHS recordings are not applicable.
**Frozen commit:** d7c4a91 (12 tests green) + 9af6cc1 (pass-1 remediation)
**Evidence date:** 2026-05-29

---

## AC Coverage Summary

| AC | BC | Test Function | Status |
|----|----|--------------|--------|
| AC-001 | BC-2.11.001 | `test_BC_2_11_001_top_level_keys` | PASS |
| AC-002 | BC-2.11.001 | `test_BC_2_11_001_findings_array_length` | PASS |
| AC-003 | BC-2.11.001 | `test_BC_2_11_001_summary_subkeys` | PASS |
| AC-004 | BC-2.11.001 | `test_BC_2_11_001_output_is_pretty_printed` | PASS |
| AC-005 | BC-2.11.002 | `test_BC_2_11_002_skipped_packets_zero_present` | PASS |
| AC-006 | BC-2.11.002 | `test_BC_2_11_002_skipped_packets_nonzero` | PASS |
| AC-007 | BC-2.11.003 | `test_BC_2_11_003_c0_esc_escaped_in_json` | PASS |
| AC-008 | BC-2.11.003 | `test_BC_2_11_003_del_not_escaped_in_json` | PASS |
| AC-009 | BC-2.11.003 | `test_BC_2_11_003_c0_roundtrip` | PASS |
| AC-010 | BC-2.11.004 | `test_BC_2_11_004_cyrillic_preserved_readable` | PASS |
| AC-011 | BC-2.11.005 | `test_BC_2_11_005_c1_passthrough_raw_utf8` | PASS |
| AC-012 | BC-2.11.005 | `test_BC_2_11_005_c0_escaped_c1_passthrough_in_same_string` | PASS |

**Result: 12/12 ACs PASS**

---

## BC-2.11.001: 3-Key JSON Object Structure

### AC-001 — Top-Level Keys (exactly summary/findings/analyzers)

```
Test: test_BC_2_11_001_top_level_keys
Input: render(&[]) — empty findings slice, default Summary
Assert: keys collected, sorted → ["analyzers", "findings", "summary"]
Assert: each of "summary", "findings", "analyzers" present via contains_key

Result: PASS — top-level object has exactly 3 keys, no extras
```

### AC-002 — Findings Array Length

```
Test: test_BC_2_11_001_findings_array_length
Input 1: render(&[]) — empty slice
Assert: findings is array of length 0

Input 2: render(&[finding_A]) — one finding
Assert: findings is array of length 1

Input 3: render(&[finding_A, finding_B]) — two findings
Assert: findings is array of length 2

Result: PASS — findings array length matches input slice length
```

### AC-003 — Summary Subkeys

```
Test: test_BC_2_11_001_summary_subkeys
Input: render(&[]) — default Summary
Required keys: total_packets, total_bytes, skipped_packets, unique_hosts, protocols, services
Assert: each key present via summary.contains_key(key)

Result: PASS — all 6 required summary subkeys present
```

### AC-004 — Pretty-Printed Output

```
Test: test_BC_2_11_001_output_is_pretty_printed
Input: render(&[]) — default Summary
Assert 1: json_str.contains('\n') — has newlines
Assert 2: at least one line starts_with(' ') — has indentation
Assert 3: at least one line starts_with("  \"") — 2-space indented key
Assert 4: json_str.contains("\n  \"summary\"") — top-level key is 2-space indented

Result: PASS — output is pretty-printed with 2-space indentation (serde_json::to_string_pretty confirmed)
```

---

## BC-2.11.002: skipped_packets Always Present

### AC-005 — skipped_packets = 0: Key Present

```
Test: test_BC_2_11_002_skipped_packets_zero_present
Input: Summary { skipped_packets: 0, ..Default }
Assert: summary["skipped_packets"] exists (get() returns Some)
Assert: value is 0 (as_u64() == Some(0))

Result: PASS — skipped_packets key present with value 0, not omitted
```

### AC-006 — skipped_packets = 3: Key and Value Correct

```
Test: test_BC_2_11_002_skipped_packets_nonzero
Input: Summary { skipped_packets: 3, ..Default }
Assert: value["summary"]["skipped_packets"].as_u64() == Some(3)

Result: PASS — skipped_packets present with correct non-zero value
```

---

## BC-2.11.003: C0 Escaped, DEL Raw, Round-Trip

### AC-007 — ESC (0x1B) Escaped as 

```
Test: test_BC_2_11_003_c0_esc_escaped_in_json
Input: Finding { summary: "\x1b[31mRED\x1b[0m" }
Assert: !json_str.as_bytes().contains(&0x1b) — raw ESC byte absent
Assert: json_str.contains("\\u001b") — escape sequence present

Result: PASS — serde_json escapes ESC (C0) per RFC 8259
```

### AC-008 — DEL (0x7F) Not Escaped

```
Test: test_BC_2_11_003_del_not_escaped_in_json
Input: Finding { summary: "before\x7fafter" }
Assert: json_str.as_bytes().contains(&0x7f) — raw DEL byte present
Assert: !json_str.contains("\\u007f") — lowercase escape absent
Assert: !json_str.contains("\\u007F") — uppercase escape absent

Result: PASS — DEL passes through as raw 0x7F
```

### AC-009 — C0 Round-Trip Recovery

```
Test: test_BC_2_11_003_c0_roundtrip
Input: Finding { summary: "\x00null\x07bel\x1b[31mesc-seq\x1b[0m" }

Wire assertions: NUL/BEL/ESC each absent as raw byte, present as \uNNNN
Round-trip: parse JSON, extract findings[0].summary, compare to original

Result: PASS — wire format correct, round-trip recovers exact bytes
```

---

## BC-2.11.004: Non-ASCII Unicode Readable

### AC-010 — Cyrillic Preserved as Raw UTF-8

```
Test: test_BC_2_11_004_cyrillic_preserved_readable
Input: Finding { summary: "TLS SNI: пример.рф" }
Assert: json_str.contains("пример.рф") — Cyrillic literal present
Assert: !json_str.contains("\\u{43f}") — no Debug-format escapes
Assert: !json_str.contains("\\u043f") — no RFC 8259 escape for U+043F
Assert: !json_str.contains("\\u04") — no Cyrillic-block escapes (broad guard)
Round-trip: recovered == "TLS SNI: пример.рф"

Result: PASS — serde_json preserves non-ASCII Unicode as readable raw UTF-8
```

---

## BC-2.11.005: C1 Codepoints Raw UTF-8

### AC-011 — U+009B (C1 CSI) as Raw 0xC2 0x9B

```
Test: test_BC_2_11_005_c1_passthrough_raw_utf8
Input: Finding { summary: "payload: \u{009b}31mINJECTED" }
Assert: bytes.windows(2).any(|w| w == [0xC2, 0x9B]) — raw UTF-8 pair present
Assert: !json_str.contains("\\u009b") — lowercase escape absent
Assert: !json_str.contains("\\u009B") — uppercase escape absent

Result: PASS — C1 CSI passes through as raw 0xC2 0x9B
```

### AC-012 — C0 Escaped, C1 Raw in Same String

```
Test: test_BC_2_11_005_c0_escaped_c1_passthrough_in_same_string
Input: Finding { summary: "\x1b[31m\u{009b}INJECTED\x1b[0m" }

C0: !bytes.contains(&0x1b), json_str.contains("\\u001b")
C1: bytes.windows(2).any(|w| w == [0xC2, 0x9B]), !contains("\\u009b"), !contains("\\u009B")

Result: PASS — C0/C1 asymmetric treatment confirmed in single string
```

---

## Full Suite Verification

```
cargo test --all-targets
  test result: ok. 915 passed; 0 failed; 0 ignored

cargo clippy --all-targets -- -D warnings
  Finished — 0 warnings, 0 errors

cargo fmt --check
  Finished — no differences found

git diff --name-only develop..HEAD -- src/
  (empty — zero src/ changes)
```

---

## VP-017 Deferral Note

VP-017 (universal RFC-8259 round-trip property via proptest) is correctly DEFERRED to Phase-6 formal hardening per story spec `verification_properties: [VP-017]`. Example-based C0 round-trip evidence is supplied via AC-009.
