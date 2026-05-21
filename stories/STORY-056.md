---
document_type: story
story_id: "STORY-056"
epic_id: "E-5"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.017.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.019.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.020.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.021.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.037.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-055]
blocks: [STORY-057]
behavioral_contracts:
  - BC-2.07.017
  - BC-2.07.019
  - BC-2.07.020
  - BC-2.07.021
  - BC-2.07.037
verification_properties: [VP-005]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 18
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield
---

> **Execute:** `/vsdd-factory:deliver-story STORY-056`

# STORY-056: SNI Classification Arms 3 and 4 — Non-ASCII UTF-8 and Non-UTF-8 Byte Preservation

## Narrative
- **As a** forensic analyst
- **I want** the TLS analyzer to detect and flag SNI hostnames containing non-ASCII Unicode (arm 3) or invalid UTF-8 bytes (arm 4) — each emitting a T1027 finding with the raw bytes preserved per ADR 0003 — and to correctly route mixed non-ASCII+C0 hostnames to arm 3 rather than arm 2
- **So that** internationalized-label evasion and binary-garbage SNI injection are reliably detected, with lossless hex evidence enabling forensic reconstruction of the original byte sequence

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.017 | Non-ASCII UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027) |
| BC-2.07.019 | Non-UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027); Count Key Tagged |
| BC-2.07.020 | Non-UTF-8 SNI Preserves Raw Bytes per ADR 0003 |
| BC-2.07.021 | Non-ASCII UTF-8 SNI Preserves Raw Bytes per ADR 0003 |
| BC-2.07.037 | SNI with Both Non-ASCII and C0 Control Bytes Fires Arm 3 (NonAsciiUtf8), Not Arm 2 |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.017 postcondition 1-3)
When `extract_sni` receives SNI bytes that are valid UTF-8 but fail `is_ascii()` (at least one code point >= 0x80), it returns `SniValue::NonAsciiUtf8 { hostname, hex }`. One `Finding` is pushed with: `category = Anomaly`, `verdict = Inconclusive`, `confidence = Low`, `summary = "TLS SNI contains non-ASCII characters (RFC 6066 requires A-labels per RFC 5890): {hostname}"`, `evidence = ["hex: {hex}"]`, `mitre_technique = Some("T1027")`, `direction = Some(Direction::ClientToServer)`. The SNI is counted in `sni_counts` under the raw hostname string key.
- **Test:** `test_valid_utf8_non_ascii_sni_emits_finding`; `test_cyrillic_sni_emits_non_ascii_finding`

### AC-002 (traces to BC-2.07.017 invariant 1)
The hostname in the finding summary is the raw decoded UTF-8 string — Cyrillic chars, emoji, CJK, and other non-ASCII codepoints are present verbatim. No Debug-format (`{:?}`) or `escape_for_terminal` escaping is applied at the analyzer layer.
- **Test:** `test_cyrillic_sni_emits_non_ascii_finding` (assert summary contains raw Cyrillic, not `\u{...}` sequences)

### AC-003 (traces to BC-2.07.017 invariant 3)
Any non-ASCII UTF-8 triggers arm 3, including: Cyrillic, emoji, CJK, and international labels that were not Punycode-encoded. Emoji bytes (multi-byte UTF-8 sequences) also land in arm 3.
- **Test:** `test_emoji_sni_emits_non_ascii_finding`

### AC-004 (traces to BC-2.07.019 postcondition 1-3)
When `extract_sni` receives SNI bytes that fail `str::from_utf8` (invalid UTF-8), arm 4 fires: `SniValue::NonUtf8 { lossy, hex }`. The `sni_counts` key is formatted as `"<non-utf8:{hex}>"` (not the lossy form). One `Finding` is pushed with: `category = Anomaly`, `verdict = Inconclusive`, `confidence = Low`, `summary = "TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}"`, `evidence = ["hex: {hex}"]`, `mitre_technique = Some("T1027")`, `direction = Some(Direction::ClientToServer)`.
- **Test:** `test_non_utf8_sni_emits_finding_and_counts_under_hex_key`; `non_utf8_sni_finding_sets_mitre_t1027`

### AC-005 (traces to BC-2.07.019 invariant 1)
The `sni_counts` key for non-UTF-8 SNI is `"<non-utf8:{hex}>"` — the hex-tagged format. Two distinct invalid byte sequences that produce the same `from_utf8_lossy` form (same U+FFFD replacement) map to DIFFERENT keys in `sni_counts`.
- **Test:** `test_non_utf8_sni_emits_finding_and_counts_under_hex_key` (assert key format is `<non-utf8:...>`)

### AC-006 (traces to BC-2.07.020 postcondition 1-4)
For arm 4, `finding.summary` contains the `lossy` string from `String::from_utf8_lossy` (U+FFFD replacements for invalid bytes). `finding.evidence[0]` contains `"hex: {hex}"` with the lossless lowercase hex. Neither field has been passed through `escape_for_terminal` or `{:?}` Debug format.
- **Test:** `test_non_utf8_sni_preserves_raw_bytes_in_summary`

### AC-007 (traces to BC-2.07.020 invariant 1-2)
Escaping is applied ONLY by the terminal reporter at render time (ADR 0003). The hex field is always pure ASCII (0-9, a-f) and needs no escaping. The JSON reporter receives the raw lossy summary; serde_json handles Unicode encoding per RFC 8259.
- **Test:** Unit test asserting `finding.evidence[0].starts_with("hex: ")` and hex field contains only `[0-9a-f]`

### AC-008 (traces to BC-2.07.021 postcondition 1-3)
For arm 3 (NonAsciiUtf8), `finding.summary` contains the decoded UTF-8 hostname with all non-ASCII codepoints intact (e.g., raw Cyrillic characters). `finding.evidence[0]` = `"hex: {hex}"` with lossless lowercase hex. No `{:?}` Debug escaping is applied (which would insert `\u{NNNN}` sequences).
- **Test:** `test_cyrillic_sni_emits_non_ascii_finding` (assert raw Cyrillic in summary, not escaped)

### AC-009 (traces to BC-2.07.037 postcondition 1-4)
When SNI bytes are valid UTF-8 but contain BOTH non-ASCII characters AND C0/DEL control bytes (e.g., `b"caf\x01\xc3\xa9"`), arm 3 fires (NonAsciiUtf8), NOT arm 2 (AsciiWithControl). The finding summary says "non-ASCII characters" — NOT "control bytes". The control byte information is only recoverable from the hex evidence field.
- **Test:** Unit test with `b"caf\x01\xc3\xa9"` (valid UTF-8 with C0 + non-ASCII); assert `SniValue::NonAsciiUtf8` and summary contains "non-ASCII"

### AC-010 (traces to BC-2.07.037 invariant 1-2)
The `is_ascii()` predicate is the decisive gate between arm 2 and arm 3. Even one non-ASCII code point causes `is_ascii()` to return false, which routes the string to arm 3 before `contains_c0_or_del` is checked. Arm evaluation is strictly top-down.
- **Test:** Unit test confirming that for `b"caf\x01\xc3\xa9"`, `is_ascii()` returns false (arm 3 fires); for `b"evil\x01.com"`, `is_ascii()` returns true (arm 2 fires)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `extract_sni` (arm 3) | src/analyzer/tls.rs:256-260 | pure-core (returns SniValue::NonAsciiUtf8) |
| `extract_sni` (arm 4) | src/analyzer/tls.rs:260-264 | pure-core (returns SniValue::NonUtf8) |
| `bytes_to_hex` | src/analyzer/tls.rs:86-93 | pure-core |
| Arm 3 match in `handle_client_hello` | src/analyzer/tls.rs:449-467 | effectful-shell (mutates sni_counts, all_findings) |
| Arm 4 match in `handle_client_hello` | src/analyzer/tls.rs:410-415, 469-488 | effectful-shell (mutates sni_counts, all_findings) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | SNI bytes = b"\xd0\xbc\xd0\xb8\xd1\x80" (Cyrillic "мир") | Arm 3; finding with "non-ASCII characters" in summary |
| EC-002 | SNI with emoji (multi-byte UTF-8) | Arm 3; finding emitted |
| EC-003 | SNI bytes = b"caf\x01\xc3\xa9" (C0 + non-ASCII UTF-8) | Arm 3 fires (is_ascii() false); summary says "non-ASCII" not "control" |
| EC-004 | SNI bytes = b"\xff\xfe" (invalid UTF-8) | Arm 4; key = `"<non-utf8:fffe>"` |
| EC-005 | SNI bytes = b"\x80" (lone continuation byte) | Arm 4; finding with T1027 |
| EC-006 | Two distinct invalid sequences with same U+FFFD replacement | Two different `sni_counts` entries (hex-tagged) |
| EC-007 | Non-ASCII + C0: summary contains "non-ASCII" | SOC operator searching "control" will miss this — documented observable behavior |
| EC-008 | Emoji "😈" = bytes [0xF0, 0x9F, 0x98, 0x88] | Valid UTF-8; arm 3 (is_ascii() false) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (extract_sni arms 3+4, bytes_to_hex) | pure-core | No I/O, no state mutation; pure transformations returning SniValue |
| src/analyzer/tls.rs (arm 3+4 in handle_client_hello) | effectful-shell | Mutates sni_counts and all_findings |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| Referenced code (tls.rs lines 86-93, 256-264, 410-415, 449-488) | ~4,000 |
| Test files (tls_analyzer_tests.rs SNI arm 3/4 tests) | ~5,000 |
| BC files (5 BCs) | ~5,500 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~20,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~10%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all AC tests fail before implementation
3. [ ] Implement `bytes_to_hex(bytes: &[u8]) -> String` producing lowercase hex string per tls.rs:86-93
4. [ ] Implement `extract_sni` arm 3: `Ok(s) => SniValue::NonAsciiUtf8 { hostname: s.to_string(), hex: bytes_to_hex(hostname) }` (catch-all Ok arm after arm 2 guard)
5. [ ] Implement `extract_sni` arm 4: `Err(_) => SniValue::NonUtf8 { lossy: String::from_utf8_lossy(hostname).into_owned(), hex: bytes_to_hex(hostname) }`
6. [ ] Implement arm 3 handling in `handle_client_hello`: insert `sni_counts` under raw hostname key; push one Anomaly/Inconclusive/Low T1027 finding with raw hostname in summary and hex in evidence
7. [ ] Implement arm 4 handling in `handle_client_hello`: insert `sni_counts` under `"<non-utf8:{hex}>"` key; push one Anomaly/Inconclusive/Low T1027 finding with lossy summary and hex evidence
8. [ ] Write arm 3/4 disambiguation test: `b"caf\x01\xc3\xa9"` must fire arm 3, not arm 2
9. [ ] Write collision test: two distinct invalid byte sequences with same lossy form must produce two distinct sni_counts keys
10. [ ] Run all tests; verify all pass
11. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-055 | Arm 1 and arm 2 share the `is_ascii()` check as the 2/3 boundary gate; arm 2 guard is `Ok(s) if s.is_ascii() && contains_c0_or_del(s)` | `extract_sni` match arms are evaluated top-down: arm 1 first (clean ASCII), arm 2 second (ASCII with C0), arm 3 third (non-ASCII UTF-8), arm 4 last (non-UTF-8) | Arm 3 must come AFTER arm 2's `is_ascii()` guard — otherwise `is_ascii()` is never checked for C0-only strings |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Arm 3 fires before `contains_c0_or_del` is evaluated for non-ASCII strings | BC-2.07.037 invariant 1 | Code review: confirm ordered match arms; arm 2 guard uses `if s.is_ascii()` before `contains_c0_or_del` |
| No `escape_for_terminal` or `{:?}` Debug escaping in arm 3/4 finding construction | ADR 0003 / INV-4; BC-2.07.021 invariant 1 | Code review: no escape call in arm 3/4 finding push |
| Non-UTF-8 `sni_counts` key is `"<non-utf8:{hex}>"` — hex-tagged, not the lossy form | BC-2.07.019 invariant 1 | Unit test: AC-005 asserts key format |
| `lossy` in arm 4 finding summary uses `String::from_utf8_lossy` (U+FFFD replacements); `hex` in evidence is lossless | BC-2.07.020 postcondition 1-4 | Unit test: AC-006 asserts summary contains lossy form, evidence contains hex |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `TlsExtension::SNI`, SNI bytes extraction |
| Rust std | 2024 edition (stable) | `str::from_utf8`, `String::from_utf8_lossy`, `str::is_ascii`, iterator formatting |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | modify | `bytes_to_hex` (86-93), `extract_sni` arms 3+4 (256-264), arm 3+4 handling in `handle_client_hello` (410-415, 449-488) |
| tests/tls_analyzer_tests.rs | modify | `test_valid_utf8_non_ascii_sni_emits_finding`, `test_cyrillic_sni_emits_non_ascii_finding`, `test_emoji_sni_emits_non_ascii_finding`, `test_non_utf8_sni_emits_finding_and_counts_under_hex_key`, `test_non_utf8_sni_preserves_raw_bytes_in_summary`, arm 2/3 disambiguation test |
