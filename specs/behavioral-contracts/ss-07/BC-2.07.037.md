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

# BC-2.07.037: SNI with Both Non-ASCII and C0 Control Bytes Fires Arm 3 (NonAsciiUtf8), Not Arm 2

## Description

For SNI bytes that are valid UTF-8 but contain BOTH non-ASCII characters AND C0/DEL control
bytes (e.g., `caf\x01\xe9` -- ASCII with a control byte followed by a non-ASCII UTF-8 byte),
arm 3 of `extract_sni` fires (NonAsciiUtf8), NOT arm 2 (AsciiWithControl). This is because
the `is_ascii()` predicate is the controlling gate between arms 2 and 3: `is_ascii()` returns
false when ANY code point is outside U+0000-U+007F.

The practical consequence: the finding summary says "non-ASCII characters" rather than "contains
control bytes." The control-byte signal is still present in the hex evidence field, but a
SOC operator searching for "control" in summary text will miss this finding. This is documented
observable behavior, not a bug (pass-2 R3 Target 2).

## Preconditions

1. An SNI hostname byte sequence is being classified by `extract_sni`.
2. `str::from_utf8(bytes) == Ok(s)` -- bytes are valid UTF-8.
3. `s.is_ascii() == false` -- at least one code point is outside U+0000-U+007F.
4. The bytes ALSO contain at least one C0/DEL byte (e.g., 0x01-0x1F or 0x7F).

## Postconditions

1. Arm 3 fires: `SniValue::NonAsciiUtf8(s)` is returned.
2. A Finding is emitted with summary text about "non-ASCII characters" (NOT "control bytes").
3. The control-byte information is recoverable only from the hex evidence field.
4. The finding has MITRE T1027, direction ClientToServer, Anomaly/Likely/High.

## Invariants

1. `is_ascii()` check at arm 2/3 boundary is the decisive predicate. A string with even one
   non-ASCII code point fails `is_ascii()` and falls to arm 3.
2. Arm evaluation is strictly top-down; once arm 3 triggers, arm 2 cannot fire.
3. Both arm 2 and arm 3 ultimately produce T1027 findings; the distinction is in summary text.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | "caf\x01\xe9" (café with C0 byte) | Arm 3: NonAsciiUtf8; summary says "non-ASCII"; control byte only in evidence |
| EC-002 | "\x01example.com" (C0 only, all ASCII) | Arm 2: AsciiWithControl; summary says "control bytes" |
| EC-003 | "caf\xe9.example.com" (non-ASCII, no C0) | Arm 3: NonAsciiUtf8 |
| EC-004 | SOC operator's string search for "control" | Will miss EC-001; must search evidence field |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI bytes = b"caf\x01\xc3\xa9" (valid UTF-8: "caf" + SOH + U+00E9) | SniValue::NonAsciiUtf8; Finding summary contains "non-ASCII" | happy-path |
| SNI bytes = b"evil\x01.com" (all ASCII + C0) | SniValue::AsciiWithControl; Finding summary contains "control" | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Mixed non-ASCII + C0 SNI fires arm 3 not arm 2 | unit: assert SniValue::NonAsciiUtf8 result for mixed bytes |
| VP-TBD | is_ascii() is the arm 2/3 gate (not contains_c0_or_del) | unit: confirm arm 3 fires before C0 check is evaluated |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- this BC clarifies the critical disambiguation rule in the SNI 4-way classification that affects SOC operator searches |
| L2 Domain Invariants | INV-5 (SNI 4-way classification ordered match -- this BC specifies the disambiguation for the arm 2/3 boundary) |
| Architecture Module | SS-07 (analyzer/tls.rs:219-242, C-16) |
| Stories | S-TBD |
| Origin BC | BC-TLS-037 (pass-3 ingestion corpus, HIGH confidence; pass-2 R3 Target 2) |

## Related BCs

- BC-2.07.014 -- related to (arm 2: pure AsciiWithControl case)
- BC-2.07.017 -- related to (arm 3: pure NonAsciiUtf8 case)
- BC-2.07.016 -- related to (boundary: 0x1F vs 0x20 in the is_ascii context)

## Architecture Anchors

- `src/analyzer/tls.rs:219-242` -- extract_sni ordered match; is_ascii() at arm 2/3 boundary
- `src/analyzer/tls.rs:405` vs `src/analyzer/tls.rs:424` -- different summary text for arm 2 vs arm 3

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:219-242` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: ordered match arms; is_ascii() evaluated before contains_c0_or_del in arm 3 path
- **documentation**: pass-2 R3 Target 2 analysis confirmed this behavior

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
