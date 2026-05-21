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

# BC-2.02.009: Surface No IP Layer Found Error for Non-IP Frames

## Description

When etherparse successfully parses the link-layer framing but finds no IP layer (e.g., an
ARP frame, or a pure Ethernet frame with a non-IP EtherType), `decode_packet` returns
`Err(anyhow!("No IP layer found"))`. Lax parsing is not attempted because lax parsing cannot
conjure an IP layer that is structurally absent -- the lax fallback is only for snaplen-
truncated length mismatches, not for absent IP layers.

## Preconditions

1. `data` is a valid link-layer frame that does not contain an IP layer.
2. `datalink` is one of the five accepted variants.
3. etherparse `SlicedPacket::from_*` returns `Ok` but with `net == None`.

## Postconditions

1. Returns `Err(anyhow!("No IP layer found"))`.
2. No panic occurs.
3. The caller increments `summary.skipped_packets`.

## Invariants

1. The "No IP layer found" error fires whenever `slice.net` is `None` after a successful
   strict parse (decoder.rs:148-151).
2. The same check fires on the lax path after a length-error retry (decoder.rs:162-164).
3. Neither ARP packets nor other non-IP EtherTypes cause lax-retry; they are rejected on
   the strict-parse-no-IP path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ARP frame (EtherType 0x0806) with ETHERNET link type | Strict parses OK but net=None; Err("No IP layer found") |
| EC-002 | IPv6 content via ETHERNET with IPv6 EtherType | IP layer present; Ok returned (no error) |
| EC-003 | Snaplen-truncated frame with no IP bytes at all | Lax retry; if lax also finds net=None, Err("No IP layer found") |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Ethernet ARP frame bytes | Err containing "No IP layer found" | error |
| Ethernet frame with EtherType 0x9000 (custom) | Err containing "No IP layer found" | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-008 | Non-IP Ethernet frames produce "No IP layer found" error | unit: construct ARP frame bytes, assert Err message |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 -- non-IP frame rejection is part of the decode gate that limits processing to IP traffic |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-003 |
| Origin BC | BC-DEC-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.007 -- related to (both produce Err without panic; different conditions)

## Architecture Anchors

- `src/decoder.rs:148-151` -- `None => Err(anyhow!("No IP layer found"))` (strict path)
- `src/decoder.rs:162-164` -- same guard on lax path

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:148-151` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: match on `slice.net` returning Err when None

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed.
