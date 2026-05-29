---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: ["v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21", "v1.3: anchor sweep — correct extract_sni arm-3/arm-4 line citations to 257-260/261-264 (PG-W16-003 sibling sweep from F-S056-P5-001/002/003) — 2026-05-29"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.017: Non-ASCII UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027)

## Description

When an SNI hostname byte sequence decodes as valid UTF-8 but contains at least one
code point outside U+0000-U+007F (i.e., `is_ascii()` returns false), arm 3 of
`extract_sni` fires (`SniValue::NonAsciiUtf8`). A finding is emitted with
`Anomaly/Inconclusive/Low` and MITRE T1027. The summary references "non-ASCII
characters" and the RFC 6066 A-label requirement. Raw UTF-8 hostname bytes are
preserved in the finding (ADR 0003 / INV-4).

## Preconditions

1. `from_utf8(hostname) == Ok(s)` -- valid UTF-8.
2. `s.is_ascii() == false` -- at least one byte >= 0x80 in the decoded string.
3. This includes pure non-ASCII (e.g., Cyrillic, emoji) AND mixed non-ASCII+C0 cases
   (arm 3 fires before arm 2 can, per INV-5 precedence -- see BC-2.07.037).

## Postconditions

1. `extract_sni` returns `Some(SniValue::NonAsciiUtf8 { hostname, hex })`.
2. One finding is pushed to `all_findings` with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Low
   - summary: "TLS SNI contains non-ASCII characters (RFC 6066 requires A-labels per RFC 5890): {hostname}"
   - evidence: ["hex: {hex}"]
   - mitre_technique: Some("T1027")
   - source_ip: None
   - timestamp: None
   - direction: Some(Direction::ClientToServer)
3. The SNI is counted in `sni_counts` under the raw hostname string key.

## Invariants

1. The hostname in summary is the raw decoded UTF-8 string (no escape applied at
   this layer -- ADR 0003 / INV-4).
2. The hex evidence is the lowercase hex encoding of the raw bytes.
3. Any non-ASCII UTF-8 triggers arm 3, including Cyrillic, emoji, CJK, and
   internationalized labels that were not Punycode-encoded.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI = "cafe\xc3\xa9" (UTF-8 U+00E9) | Arm 3; finding with "non-ASCII characters" |
| EC-002 | SNI with Cyrillic (e.g., "\xd0\xbc\xd0\xb8\xd1\x80") | Arm 3; finding emitted |
| EC-003 | SNI with emoji (multi-byte UTF-8) | Arm 3; finding emitted |
| EC-004 | SNI with non-ASCII AND C0 bytes (e.g., "caf\x01\xc3\xa9") | Arm 3 fires (is_ascii()=false); NOT arm 2 (see BC-2.07.037) |
| EC-005 | Punycode "xn--caf-dma.example" (pure ASCII) | Arm 1; no finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI bytes = b"\xd0\xbc\xd0\xb8\xd1\x80" (valid UTF-8 Cyrillic "mir") | Finding(Anomaly/Inconclusive/Low, T1027, direction=ClientToServer) | happy-path |
| SNI = "example.com" (pure ASCII) | No finding | happy-path |
| SNI bytes = b"caf\x01\xc3\xa9" (mixed C0+non-ASCII UTF-8) | Arm 3 fires (not arm 2); summary says "non-ASCII" | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-005 | Non-ASCII UTF-8 SNI produces Anomaly/Inconclusive/Low finding with T1027 | unit: test_valid_utf8_non_ascii_sni_emits_finding; test_cyrillic_sni_emits_non_ascii_finding; test_emoji_sni_emits_non_ascii_finding |
| VP-005 | Raw UTF-8 hostname preserved in summary | unit: test_cyrillic_sni_emits_non_ascii_finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- non-ASCII UTF-8 SNI detection is arm 3 of the SNI 4-way classification |
| L2 Domain Invariants | INV-5 (SNI 4-way classification ordered match), INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:257-260, 449-467, C-13) |
| Stories | STORY-056 |
| Origin BC | BC-TLS-017 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.037 -- related to (disambiguation: mixed C0+non-ASCII goes to arm 3, not arm 2)
- BC-2.07.014 -- related to (arm 2: pure ASCII with C0)
- BC-2.07.019 -- related to (arm 4: non-UTF-8 bytes)
- BC-2.07.018 -- composes with (Punycode A-labels bypass this arm)

## Architecture Anchors

- `src/analyzer/tls.rs:257-260` -- arm 3 in extract_sni match (`Ok(s) => SniValue::NonAsciiUtf8 { ... }`)
- `src/analyzer/tls.rs:449-467` -- NonAsciiUtf8 finding push
- `tests/tls_analyzer_tests.rs` -- test_valid_utf8_non_ascii_sni_emits_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:257-260` (extract_sni arm 3), `src/analyzer/tls.rs:449-467` (emission) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `Ok(s) => SniValue::NonAsciiUtf8 { ... }` (the catch-all Ok arm)
- **assertion**: multiple non-ASCII SNI tests

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
