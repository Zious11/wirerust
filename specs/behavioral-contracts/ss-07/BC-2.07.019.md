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

# BC-2.07.019: Non-UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027); Count Key Tagged

## Description

When SNI hostname bytes fail UTF-8 decoding (`str::from_utf8` returns `Err`), arm 4 of
`extract_sni` fires (`SniValue::NonUtf8`). The lossy representation is computed via
`String::from_utf8_lossy` for the summary. The hex encoding of the raw bytes is used
as the `sni_counts` map key, wrapped in `<non-utf8:HEX>` format. A finding is emitted
with `Anomaly/Inconclusive/Low` and MITRE T1027.

## Preconditions

1. `str::from_utf8(hostname_bytes) == Err(_)` -- bytes are not valid UTF-8.
2. This is arm 4 of the SNI classification (last-evaluated arm).

## Postconditions

1. `extract_sni` returns `Some(SniValue::NonUtf8 { lossy, hex })`.
2. The `sni_counts` key is formatted as `"<non-utf8:{hex}>"` where `hex` is the
   lowercase hex of the raw bytes. This prevents byte-sequence collisions from
   different invalid sequences that produce the same U+FFFD-replacement lossy form.
3. One finding is pushed to `all_findings` with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Low
   - summary: "TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}"
   - evidence: ["hex: {hex}"]
   - mitre_technique: Some("T1027")
   - source_ip: None
   - timestamp: None
   - direction: Some(Direction::ClientToServer)
4. The summary's hostname is the lossy `from_utf8_lossy` form (U+FFFD replacements for
   invalid bytes); the evidence has the lossless hex form.

## Invariants

1. The sni_counts key is the hex-tagged form `<non-utf8:...>`, NOT the lossy form.
   This avoids collisions: two distinct byte sequences that produce the same
   U+FFFD-replacement output would otherwise map to the same key.
2. The lossy form in the summary preserves raw bytes per ADR 0003 -- no further
   escaping at the analyzer layer.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI = [0xff, 0xfe] (invalid UTF-8) | Arm 4; key = "<non-utf8:fffe>" |
| EC-002 | Two different invalid sequences with same U+FFFD replacement | Different sni_counts keys (hex-tagged) |
| EC-003 | SNI = [0x80] (lone continuation byte) | Arm 4; finding with T1027 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI bytes = [0xff, 0xfe] (invalid UTF-8) | Finding(Anomaly/Inconclusive/Low, T1027); key="<non-utf8:fffe>" | happy-path |
| Two distinct invalid byte sequences with same lossy form | Two different sni_counts entries | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Non-UTF-8 SNI produces Anomaly/Inconclusive/Low finding with T1027 | unit: test_non_utf8_sni_emits_finding_and_counts_under_hex_key; non_utf8_sni_finding_sets_mitre_t1027 |
| VP-TBD | sni_counts key is hex-tagged form | unit: test_non_utf8_sni_emits_finding_and_counts_under_hex_key |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- non-UTF-8 SNI detection is arm 4 of the SNI 4-way classification |
| L2 Domain Invariants | INV-5 (SNI 4-way classification), INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:260-264, 410-415, 469-488, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-019 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.020 -- composes with (raw bytes preserved in summary)
- BC-2.07.017 -- related to (arm 3: valid UTF-8 but non-ASCII)
- BC-2.07.028 -- composes with (count cap; finding still fires at cap)

## Architecture Anchors

- `src/analyzer/tls.rs:261-264` -- arm 4 match in extract_sni (`Err(_) => SniValue::NonUtf8 { ... }`)
- `src/analyzer/tls.rs:410-415` -- sni_counts key selection for NonUtf8
- `src/analyzer/tls.rs:469-488` -- NonUtf8 finding push
- `tests/tls_analyzer_tests.rs` -- test_non_utf8_sni_emits_finding_and_counts_under_hex_key

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:261-264` (arm 4), `src/analyzer/tls.rs:469-488` (emission) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `Err(_) => SniValue::NonUtf8 { ... }`; key = `format!("<non-utf8:{hex}>")`
- **assertion**: test_non_utf8_sni_emits_finding_and_counts_under_hex_key

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
