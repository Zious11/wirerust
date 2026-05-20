---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.013: Clean ASCII SNI Produces No Finding

## Description

When an SNI hostname byte sequence is valid UTF-8, passes `is_ascii()`, and contains
no C0 control bytes (0x00-0x1F) or DEL (0x7F), `extract_sni` classifies it as
`SniValue::Ascii`. No finding is emitted. The hostname is counted in `sni_counts`
under the raw hostname string key. This is arm 1 of the SNI 4-way classification
(INV-5), the "nothing to flag" path.

Note: Arm 1 does not perform full RFC 6066 compliance checking. Spaces, empty strings,
non-LDH printable ASCII, and similar degenerate but non-control values still land in
arm 1 and produce no finding. Broader LDH compliance is out of scope (issue #54).

## Preconditions

1. A TLS ClientHello with an SNI extension is being parsed.
2. The first ServerName entry's hostname satisfies:
   - `str::from_utf8(bytes) == Ok(s)` (valid UTF-8)
   - `s.is_ascii() == true` (all code points in U+0000-U+007F)
   - `contains_c0_or_del(s) == false` (no byte < 0x20 or == 0x7F)

## Postconditions

1. `extract_sni` returns `Some(SniValue::Ascii(hostname))`.
2. The hostname string is inserted/incremented in `sni_counts`.
3. No finding is pushed to `all_findings`.
4. `handshakes_seen` is incremented by BC-2.07.001.

## Invariants

1. Arm 1 is the ONLY arm that produces no finding. All other SniValue arms (2, 3, 4)
   produce a T1027 finding.
2. The sni_counts key is the raw hostname string (no transformation).
3. Printable non-LDH ASCII (spaces, dots, underscores, hyphens) all land in arm 1
   without finding -- arm 1 is not a strict LDH validator.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI = "example.com" | Arm 1; no finding; sni_counts["example.com"]++ |
| EC-002 | SNI = "xn--caf-dma.example" (Punycode A-label) | Arm 1; no finding (see BC-2.07.018) |
| EC-003 | SNI = " " (space, 0x20) | Arm 1 (0x20 is not C0); no finding |
| EC-004 | SNI = "evil.com" (all printable ASCII) | Arm 1; no finding |
| EC-005 | SNI = "" (empty string) | Arm 1; no finding; sni_counts[""]++ (see BC-2.07.023) |
| EC-006 | SNI = "UPPERCASE.COM" | Arm 1; no finding; counted as-is |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello with SNI "example.com" | sni_counts["example.com"]=1; all_findings empty | happy-path |
| ClientHello with SNI "test.local" | sni_counts["test.local"]=1; all_findings empty | happy-path |
| Same SNI "example.com" twice | sni_counts["example.com"]=2 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Clean ASCII SNI produces no finding | unit: test_parse_client_hello; test_ascii_sni_does_not_emit_non_utf8_finding; test_printable_ascii_sni_emits_no_control_finding |
| VP-TBD | SNI is counted in sni_counts | unit: test_parse_client_hello asserts sni_counts |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- the no-finding arm of SNI classification defines the baseline TLS analysis behavior |
| L2 Domain Invariants | INV-5 (SNI 4-way classification ordered match -- arm 1 is the no-finding path) |
| Architecture Module | SS-07 (analyzer/tls.rs:251-252, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.014 -- related to (arm 2: ASCII with control bytes)
- BC-2.07.017 -- related to (arm 3: non-ASCII UTF-8)
- BC-2.07.019 -- related to (arm 4: non-UTF-8)
- BC-2.07.018 -- composes with (Punycode A-labels are arm 1)

## Architecture Anchors

- `src/analyzer/tls.rs:251-252` -- arm 1 match in extract_sni
- `src/analyzer/tls.rs:424-425` -- arm 1 handling in match sni block: `SniValue::Ascii(_) => {}`
- `tests/tls_analyzer_tests.rs` -- test_ascii_sni_does_not_emit_non_utf8_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:251-252` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `Ok(s) if s.is_ascii() && !contains_c0_or_del(s) => SniValue::Ascii(s.to_string())`
- **assertion**: multiple tests verify no finding for clean ASCII SNI

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
