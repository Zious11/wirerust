---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/decoder.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-02
capability: CAP-02
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.007: Reject Malformed Input Bytes with anyhow Error (No Panic)

## Description

`decode_packet` never panics on malformed input bytes. All etherparse parse failures
(bad IP version, bad IHL, bad TCP data offset, etc.) are captured as `Err(anyhow::Error)`.
The caller counts the error as a skipped packet and continues processing. This contract is
the safety guarantee enabling wirerust to process adversarial or corrupted pcap files without
crashing.

## Preconditions

1. `data` is any byte slice, including bytes that do not form a valid Ethernet/IP/TCP frame.
2. `datalink` is any of the five supported variants.

## Postconditions

1. `decode_packet` returns `Err(anyhow::Error)` with a descriptive message ("Parse error: ...").
2. No `panic!` is triggered; the process remains alive.
3. The error message contains at minimum "Parse error:" and the etherparse error description.
4. The caller (main.rs) increments `summary.skipped_packets` and continues to the next packet.

## Invariants

1. The only three error message prefixes are "Unsupported link type:", "No IP layer found", and
   "Parse error: " (from etherparse). No other panic or unwrap is reachable.
2. `anyhow!` is used for all error construction; `unwrap()` does not appear in `decode_packet`.
3. This applies to ALL input bytes including zero-length slices, random bytes, and truncated headers.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty slice (data.len() == 0) | Err("Parse error: ...") from etherparse |
| EC-002 | 1 byte (0xFF) with ETHERNET link type | Err("Parse error: ...") |
| EC-003 | Valid IP header but truncated TCP header | SliceError::Len triggers lax retry; lax may succeed or return Err |
| EC-004 | Bad IPv4 version nibble (0x05 instead of 0x45) | SliceError structural error; Err immediately (no lax retry) |
| EC-005 | Bad TCP data offset (e.g., value 0) | SliceError structural; Err (no lax retry) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Random 20 bytes with ETHERNET | Err (no panic) | error |
| Empty slice with ETHERNET | Err (no panic) | error |
| Valid IP header + corrupted TCP header | Err with "Parse error:" | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-008 | decode_packet never panics on arbitrary input | fuzz: cargo-fuzz target wrapping decode_packet |
| VP-008 | All error paths return Err not panic | kani/manual: no unwrap in decode_packet |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per domain/capabilities/cap-02-link-type-gating.md |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per domain/capabilities/cap-02-link-type-gating.md -- the no-panic guarantee is the safety property of the decode gate |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-003 |
| Origin BC | BC-DEC-007 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.008 -- related to (unsupported link type is a different Err path, also no panic)
- BC-2.02.009 -- related to (no-IP-layer is a distinct Err, also no panic)

## Architecture Anchors

- `src/decoder.rs:165-171` -- structural error arm: Err(e) => Err(anyhow!("Parse error: {e}"))
- `src/decoder.rs:136` -- unsupported link type: Err(anyhow!("Unsupported link type: ..."))

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:165-171` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: all match arms return Err or Ok; no panic/unwrap
- **documentation**: module comment describes strict-first/lax-fallback design

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed. The no-panic property is amenable to Kani formal verification once
etherparse models are available.
