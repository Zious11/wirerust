---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-18
capability: CAP-18
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.18.003: `supported_protocols()` Returns Exactly the SUPPORTED_PORTS-Intersecting Entries Plus ARP; `unsupported_protocols()` Returns the Complement

## Description

`supported_protocols()` is a pure-core function in `src/protocols.rs` that returns the
subset of `KNOWN_PROTOCOLS` whose `canonical_ports` intersect the compile-time constant
`SUPPORTED_PORTS` (which mirrors the port-fallback rules in `dispatcher.rs::classify()`),
plus the ARP entry (which is supported via `DecodedFrame::Arp` outside the dispatcher and
therefore cannot be detected by port intersection). `unsupported_protocols()` returns the
complement: every entry NOT in `supported_protocols()`. Both functions are pure and
deterministic; they have no I/O, no mutable state, and no runtime dependencies.

This contract guards against drift between what the catalog reports as "supported" and what
the dispatcher actually handles. The `SUPPORTED_PORTS` constant is the anti-drift mechanism:
when a new analyzer is added, the implementer MUST update `SUPPORTED_PORTS` or VP-041 will
fail.

## Related BCs

- BC-2.18.004 — composes with (partition invariant: supported ∪ unsupported == KNOWN_PROTOCOLS)
- BC-2.18.001 — depends on (the `--supported` / `--unsupported` filter flags in terminal output call these functions)
- BC-2.18.002 — depends on (same for JSON output)

## Preconditions

1. `supported_protocols()` is called as a pure function — no mutable state, no I/O.
2. `KNOWN_PROTOCOLS` is a non-empty static array (compile-time constant).
3. `SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 443, 8443, 80, 8080, 53]` is defined as a compile-time constant mirroring `dispatcher.rs::classify()` port-fallback rules.

## Postconditions

1. `supported_protocols()` returns all entries `p` in `KNOWN_PROTOCOLS` such that:
   - `p.canonical_ports` contains at least one value that is in `SUPPORTED_PORTS`, OR
   - `p.name == "ARP"` (ARP special-case: L2-handled outside the dispatcher).
2. `unsupported_protocols()` returns all entries `p` in `KNOWN_PROTOCOLS` such that:
   - `p` is NOT in `supported_protocols()`.
3. The ARP entry is always in `supported_protocols()`, regardless of port intersection (ARP has `canonical_ports: &[]`, so port intersection alone would exclude it; the ARP special case is required).
4. No entry with `port_detectable: false` and no port-SUPPORTED_PORTS intersection (other than ARP) appears in `supported_protocols()`.
5. Every entry in `KNOWN_PROTOCOLS` appears in exactly one of the two result sets.

## Invariants

1. `SUPPORTED_PORTS` mirrors the port-fallback rules in `dispatcher.rs::classify()`. If a new `DispatchTarget` variant and port rule is added to `classify()`, `SUPPORTED_PORTS` MUST be updated. Failure to update causes VP-041 to detect the drift.
2. `supported_protocols()` is pure and referentially transparent — the same call always returns the same result (given the same compile-time constants).
3. The ARP special case is explicit in the implementation (e.g., `|| p.name == "ARP"` or via an ARP-port constant). It MUST NOT be omitted.
4. `unsupported_protocols()` MUST NOT be a separate hand-maintained list; it must be derived as the complement of `supported_protocols()` within `KNOWN_PROTOCOLS`.
5. The sets are stable across the same binary build. They cannot change at runtime.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ARP entry has `canonical_ports: &[]` — port intersection gives ∅ | ARP still appears in `supported_protocols()` via special-case rule |
| EC-002 | TLS has two ports (443 and 8443) — both are in SUPPORTED_PORTS | TLS appears once in `supported_protocols()`; not duplicated |
| EC-003 | BACnet/IP has port 47808 — NOT in SUPPORTED_PORTS | BACnet/IP appears in `unsupported_protocols()` |
| EC-004 | GOOSE has `canonical_ports: &[]` and is NOT ARP | GOOSE appears in `unsupported_protocols()` |
| EC-005 | A new dissector is added to dispatcher (e.g., port 20000 already present) | `SUPPORTED_PORTS` already contains 20000; no change to supported set |
| EC-006 | `unsupported_protocols()` is called — result is KNOWN_PROTOCOLS minus supported set | Exact complement; no manual list |
| EC-007 | Port 102 entries (S7comm, S7comm-plus, MMS, ICCP) — port 102 is NOT in SUPPORTED_PORTS | All four appear in `unsupported_protocols()` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `supported_protocols()` | Returns 7 entries: Modbus/TCP (502), DNP3 (20000), EtherNet/IP+CIP (44818), TLS (443/8443), ARP, DNS (53), HTTP (80/8080) | happy-path |
| `unsupported_protocols()` | Returns all other entries (~23); ARP absent; BACnet/IP, GOOSE, S7comm, etc. present | happy-path |
| ARP in `supported_protocols()` result | `p.name == "ARP"` present in result | ARP-special-case |
| BACnet/IP NOT in `supported_protocols()` | Port 47808 not in SUPPORTED_PORTS; BACnet/IP absent from supported set | unsupported-udp |
| `supported_protocols().len() + unsupported_protocols().len()` | == `KNOWN_PROTOCOLS.len()` | partition |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-041 | Oracle cross-check: for each entry in KNOWN_PROTOCOLS, `entry ∈ supported_protocols() ⟺ entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) \|\| entry.name=="ARP"` (covers both partition correctness and disjoint invariant) | proptest: `proptest_vp041_oracle_cross_check` |
| — | ARP always in supported set despite no port match | unit: `test_BC_2_18_003_arp_in_supported_set` |
| — | `SUPPORTED_PORTS` entries each have a corresponding supported_protocols() entry | unit: `test_BC_2_18_003_supported_ports_mirror` |
| — | BACnet/IP (47808 not in SUPPORTED_PORTS) is in unsupported_protocols() | unit: `test_BC_2_18_003_bacnet_unsupported` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — `supported_protocols()` and `unsupported_protocols()` are the canonical pure-core functions that partition the Protocol Coverage Catalog into what wirerust dissects versus what it knows about but does not dissect |
| L2 Domain Invariants | None directly (pure-core; no domain-level invariants from brownfield spec apply) |
| Architecture Module | SS-18 (src/protocols.rs C-26); `SUPPORTED_PORTS` compile-time constant; `supported_protocols()` and `unsupported_protocols()` functions |
| ADR | ADR-012 Decision 5 (SUPPORTED_PORTS compile-time mirror; drift risk; ARP special-case handling) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/protocols.rs` — `SUPPORTED_PORTS: &[u16]` compile-time constant: `&[502, 20000, 44818, 443, 8443, 80, 8080, 53]`; doc-comment MUST list each port and its corresponding `DispatchTarget` variant
- `src/protocols.rs` — `pub fn supported_protocols() -> Vec<&'static KnownProtocol>` — returns entries matching SUPPORTED_PORTS intersection OR ARP special case
- `src/protocols.rs` — `pub fn unsupported_protocols() -> Vec<&'static KnownProtocol>` — returns complement of `supported_protocols()` within `KNOWN_PROTOCOLS`
- `src/protocols.rs` — `pub fn all_protocols() -> &'static [KnownProtocol]` — returns full `KNOWN_PROTOCOLS` slice
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_oracle_cross_check` (oracle: `entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) || entry.name=="ARP"`)

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-041 — `proptest_vp041_oracle_cross_check`: oracle checks each entry against the canonical membership predicate; verifies both partition correctness (union == KNOWN_PROTOCOLS) and disjoint (intersection == ∅) in a single pass

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | read-only (`KNOWN_PROTOCOLS` and `SUPPORTED_PORTS` are `&'static` compile-time constants) |
| **Deterministic** | yes (same binary always produces same result) |
| **Thread safety** | yes (no mutable state) |
| **Overall classification** | pure |
