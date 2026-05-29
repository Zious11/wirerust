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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: fix é-stripped U-label defect (F-W17-S055-P2-001) — restore café.example at Description, EC-003, and Canonical Test Vector — 2026-05-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.018: Punycode A-label is Pure ASCII; Emits No SNI Finding

## Description

Internationalized domain names sent in TLS SNI should be in Punycode A-label form
(e.g., `xn--caf-dma.example` for `café.example`). A-labels are pure ASCII by
construction (RFC 5890). When the SNI hostname bytes decode as valid UTF-8 with
`is_ascii() == true` and no C0/DEL bytes, arm 1 fires and no finding is emitted. This
BC exists to document that properly-encoded IDN hostnames are NOT anomalies.

## Preconditions

1. The SNI hostname bytes represent a Punycode A-label (e.g., starts with "xn--").
2. All bytes are in the ASCII range (0x20-0x7E, excluding C0 and DEL).

## Postconditions

1. `extract_sni` returns `SniValue::Ascii(hostname)`.
2. No finding is pushed to `all_findings`.
3. The A-label is counted in `sni_counts` under its raw string key.

## Invariants

1. A-labels by definition are pure ASCII; they trivially satisfy arm 1 conditions.
2. This BC is a special case of BC-2.07.013; it exists to document deliberate
   exclusion of correctly-encoded IDN from the T1027 detection surface.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI = "xn--caf-dma.example" | Arm 1; no finding |
| EC-002 | SNI = "xn--" (degenerate A-label prefix) | Arm 1; no finding (pure ASCII) |
| EC-003 | SNI = "café.example" (raw U-label, non-ASCII) | Arm 3; finding emitted (RFC 6066 violation) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI = "xn--caf-dma.example" | No finding; sni_counts has one entry | happy-path |
| SNI = "café.example" (U-label) | Finding(Anomaly/Inconclusive/Low, T1027) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Punycode A-label emits no finding | unit: test_punycode_a_label_does_not_emit_non_ascii_finding; test_punycode_a_label_emits_no_control_finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- documenting that correctly-encoded IDN is excluded from TLS analysis findings |
| L2 Domain Invariants | INV-5 (SNI 4-way classification) |
| Architecture Module | SS-07 (analyzer/tls.rs:251-252, C-13) |
| Stories | STORY-055 |
| Origin BC | BC-TLS-018 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.013 -- composes with (arm 1 is the shared path)
- BC-2.07.017 -- related to (arm 3 handles raw U-labels)

## Architecture Anchors

- `src/analyzer/tls.rs:251-252` -- arm 1 match clause
- `tests/tls_analyzer_tests.rs` -- test_punycode_a_label_does_not_emit_non_ascii_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:251-252` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_punycode_a_label_does_not_emit_non_ascii_finding

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
