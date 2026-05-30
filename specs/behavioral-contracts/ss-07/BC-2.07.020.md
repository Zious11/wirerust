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

# BC-2.07.020: Non-UTF-8 SNI Preserves Raw Bytes per ADR 0003

## Description

When a non-UTF-8 SNI hostname is classified as arm 4 (`SniValue::NonUtf8`), the
`SniValue::NonUtf8 { lossy, hex }` variant stores both the lossy `from_utf8_lossy`
form (for the summary field) and the lossless hex encoding of the original raw bytes
(for the evidence field). No `escape_for_terminal` or Debug-format escaping is applied
at the analyzer layer. This preserves forensic fidelity per ADR 0003 / INV-4.

## Preconditions

1. `extract_sni` has returned `SniValue::NonUtf8 { lossy, hex }` for a given hostname.
2. The `hex` field was computed via `bytes_to_hex(hostname)` -- lowercase hex of raw bytes.

## Postconditions

1. `finding.summary` contains the `lossy` string (U+FFFD replacements; raw UTF-8
   bytes in valid positions are preserved as-is).
2. `finding.evidence[0]` contains `"hex: {hex}"` with the lossless lowercase hex.
3. Neither field has been passed through `escape_for_terminal` or `{:?}` Debug format.
4. The lossless hex can be used to reconstruct the original byte sequence exactly.

## Invariants

1. Escaping is applied ONLY by the terminal reporter at render time (ADR 0003).
2. The JSON reporter receives the raw lossy summary; serde_json will escape any C0
   bytes in the lossy string (e.g., embedded NUL becomes `\u0000`) per RFC 8259.
3. The hex field is always pure ASCII (0-9, a-f) and needs no escaping.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Invalid bytes that happen to include 0x1b (ESC) in the lossy form | ESC byte is in summary; terminal reporter escapes it on render |
| EC-002 | Very long invalid byte sequence | hex evidence is proportionally long; no truncation |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI bytes = [0xff, 0x00, 0xfe] (invalid UTF-8 + NUL) | summary contains from_utf8_lossy form; evidence = "hex: ff00fe" | happy-path |
| SNI bytes = [0x80] | evidence = "hex: 80"; summary contains U+FFFD | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Non-UTF-8 SNI summary preserves raw bytes (no Debug escaping) | unit: test_non_utf8_sni_preserves_raw_bytes_in_summary |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- raw-byte preservation in non-UTF-8 SNI findings is load-bearing for forensic integrity |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation per ADR 0003) |
| Architecture Module | SS-07 (analyzer/tls.rs:469-488, C-13) |
| Stories | STORY-056 |
| Origin BC | BC-TLS-020 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.019 -- depends on (this BC specifies the preservation property of arm 4)
- BC-2.07.021 -- related to (non-ASCII UTF-8 also preserves raw bytes)

## Architecture Anchors

- `src/analyzer/tls.rs:469-488` -- NonUtf8 finding construction
- `src/analyzer/tls.rs:261-264` -- hex computed in extract_sni for NonUtf8
- `docs/adr/0003-reporting-pipeline-layering.md` -- ADR 0003

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:469-488` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **documentation**: ADR 0003 doc comment at findings.rs:72-80
- **assertion**: test_non_utf8_sni_preserves_raw_bytes_in_summary

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
