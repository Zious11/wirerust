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

# BC-2.07.027: Large SNI (16 KB) Under MAX_RECORD_PAYLOAD Parses Successfully

## Description

A TLS ClientHello with an SNI hostname of approximately 16 KB, embedded in a record
whose `payload_len <= MAX_RECORD_PAYLOAD` (18,432 bytes), is accepted and parsed
normally. The large hostname is classified, counted, and potentially triggers a finding
if it contains anomalous bytes. The per-direction buffer cap (MAX_BUF = 65,536) is
large enough to hold such a record. This test confirms that the limits interact
correctly: MAX_RECORD_PAYLOAD is the binding constraint, not MAX_BUF.

## Preconditions

1. A TLS ClientHello record with payload_len <= 18,432 bytes.
2. The SNI hostname is approximately 16 KB (under the record payload limit).
3. The buffer has sufficient capacity (MAX_BUF = 65,536 >> 18,432).

## Postconditions

1. The record is accepted (not truncated).
2. `parse_errors` is NOT incremented.
3. The large hostname is classified and counted in `sni_counts`.
4. `handshakes_seen` is incremented.
5. If the hostname is clean ASCII, no finding is emitted.

## Invariants

1. MAX_RECORD_PAYLOAD (18,432) is the binding size constraint; MAX_BUF (65,536)
   is not the bottleneck for a single record.
2. The test confirms that the system does not have an SNI-length-specific cap below
   MAX_RECORD_PAYLOAD.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI hostname = 16,384 bytes of 'a' | Parsed; counted; no finding (all clean ASCII) |
| EC-002 | SNI hostname = 18,430 bytes (near record limit) | Parsed if record payload fits; no truncation |
| EC-003 | SNI payload causing record to exceed 18,432 | Truncated per BC-2.07.004 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello with ~16 KB clean ASCII SNI hostname | parse_errors=0; handshakes_seen=1; sni_counts has one entry | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | 16 KB SNI parses without error | unit: test_large_sni_near_record_payload_limit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- large SNI handling confirms the interaction between MAX_RECORD_PAYLOAD and MAX_BUF limits |
| L2 Domain Invariants | INV-5 (SNI 4-way classification) |
| Architecture Module | SS-07 (analyzer/tls.rs:641-653 for limit, C-16) |
| Stories | S-TBD |
| Origin BC | BC-TLS-027 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.004 -- related to (MAX_RECORD_PAYLOAD guard; large records that exceed it are rejected)
- BC-2.07.005 -- related to (MAX_BUF is the per-direction buffer cap)

## Architecture Anchors

- `src/analyzer/tls.rs:641-653` -- MAX_RECORD_PAYLOAD guard
- `tests/tls_analyzer_tests.rs` -- test_large_sni_near_record_payload_limit

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:641-653` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_large_sni_near_record_payload_limit

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts, handshakes_seen |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
