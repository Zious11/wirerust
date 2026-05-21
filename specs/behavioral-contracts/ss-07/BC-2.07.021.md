---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.021: Non-ASCII UTF-8 SNI Preserves Raw Bytes per ADR 0003

## Description

When a non-ASCII but valid UTF-8 SNI is classified as arm 3 (`SniValue::NonAsciiUtf8`),
the decoded UTF-8 `hostname` string is stored directly in the finding's summary field,
and the lowercase hex of the raw bytes is stored in evidence. No Debug-format or
escape_for_terminal is applied at the analyzer layer, per ADR 0003 / INV-4. The raw
Cyrillic, emoji, CJK, or other non-ASCII codepoints are preserved verbatim in the
summary string.

## Preconditions

1. `extract_sni` has returned `SniValue::NonAsciiUtf8 { hostname, hex }`.
2. `hostname` is a valid UTF-8 String containing at least one non-ASCII code point.

## Postconditions

1. `finding.summary` contains the decoded `hostname` string with all non-ASCII
   codepoints intact (e.g., Cyrillic chars, emoji).
2. `finding.evidence[0]` = `"hex: {hex}"` with lossless lowercase hex.
3. No `{:?}` Debug escaping is applied (which would insert `\u{NNNN}` escape
   sequences and corrupt the forensic data).

## Invariants

1. Terminal reporter applies escaping at render time (ADR 0003); this layer does not.
2. The summary string is safe for JSON emission via serde_json (which handles Unicode
   correctly in JSON strings).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cyrillic "мир" in SNI | summary = "TLS SNI contains non-ASCII characters ...: мир" (raw Cyrillic) |
| EC-002 | Emoji "😈" in SNI | summary contains raw emoji codepoint |
| EC-003 | Mixed ASCII + non-ASCII "cafe.cafe\xc3\xa9" | summary contains raw UTF-8 with accent |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI bytes = b"\xd0\xbc\xd0\xb8\xd1\x80" ("мир" in UTF-8) | summary contains "мир" as raw Cyrillic; evidence has hex | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Non-ASCII UTF-8 SNI summary preserves raw Cyrillic (no Debug escaping) | unit: test_cyrillic_sni_emits_non_ascii_finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- raw-byte preservation in non-ASCII UTF-8 SNI is load-bearing for forensic integrity |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation per ADR 0003) |
| Architecture Module | SS-07 (analyzer/tls.rs:449-467, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-021 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.020 -- related to (same ADR 0003 contract, for non-UTF-8 arm)
- BC-2.07.017 -- depends on (arm 3 is the trigger)

## Architecture Anchors

- `src/analyzer/tls.rs:449-467` -- NonAsciiUtf8 finding construction
- `docs/adr/0003-reporting-pipeline-layering.md` -- ADR 0003

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:449-467` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_cyrillic_sni_emits_non_ascii_finding

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
