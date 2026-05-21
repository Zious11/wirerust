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

# BC-2.07.015: Multiple Control Bytes in One SNI Produce Exactly ONE Finding

## Description

When an SNI hostname contains multiple C0/DEL bytes (e.g., `"evil\x00\x01.com"`),
`extract_sni` classifies the entire hostname as `SniValue::AsciiWithControl` and the
finding emission site calls `self.all_findings.push(...)` exactly once. There is no
per-byte loop; the classification is at the hostname level, not the byte level. The hex
evidence field contains the hex encoding of the entire raw hostname (all bytes), not
just the control bytes.

## Preconditions

1. A TLS ClientHello SNI hostname is classified as arm 2 (`SniValue::AsciiWithControl`).
2. The hostname contains two or more bytes in [0x00-0x1F, 0x7F].

## Postconditions

1. Exactly ONE finding is pushed to `all_findings`, regardless of how many control
   bytes are in the hostname.
2. The finding's `evidence` contains one entry: `"hex: {hex}"` where `hex` is the
   lowercase hex of all raw hostname bytes.
3. The finding's `summary` contains the entire hostname string (with embedded control
   bytes, raw, per ADR 0003).

## Invariants

1. Finding count is O(1) per SNI hostname, not O(control_bytes_count).
2. The hex evidence is the full hostname hex, not a filtered hex of just the control bytes.
3. There is no deduplication needed because there can only be one AsciiWithControl arm
   match per hostname.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI = "a\x01\x02\x03b" (3 control bytes) | ONE finding; evidence = hex of full hostname |
| EC-002 | SNI with all 32 C0 values | ONE finding |
| EC-003 | SNI = "\x01" (single byte, NUL+1) | ONE finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SNI = "evil\x00\x01.com" | all_findings.len() == 1; evidence[0] starts with "hex: " | happy-path |
| SNI = "\x1f\x1e\x1d" (three C0 bytes) | all_findings.len() == 1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-005 | Multiple control bytes produce exactly one finding | unit: test_multiple_control_bytes_in_sni_produces_single_finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- this BC constrains the cardinality of SNI findings, a core TLS analysis property |
| L2 Domain Invariants | INV-5 (SNI 4-way classification), INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:424-447, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-015 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.014 -- depends on (arm 2 is the trigger; this BC specifies cardinality)
- BC-2.07.016 -- composes with (boundary: 0x1F vs 0x20)

## Architecture Anchors

- `src/analyzer/tls.rs:424-447` -- AsciiWithControl match arm and single push

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:424-447` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_multiple_control_bytes_in_sni_produces_single_finding

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
