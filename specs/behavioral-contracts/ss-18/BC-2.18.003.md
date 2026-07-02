---
document_type: behavioral-contract
level: L3
version: "1.3"
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
modified:
  - "v1.1: F-F2P2-001/002 Pass-2 remediation — VP-041 anti-drift semantics corrected; second VP-041 harness added. 2026-07-01"
  - "v1.2: F-F2P5-001 Pass-5 remediation — SUPPORTED_PORTS semantics reframed per ADR-012 canonical wording (not a pure classify() mirror); DNS/53 decode-loop path documented; Architecture Anchor doc-comment obligation updated verbatim; EC-005 clarified. 2026-07-01"
  - "v1.3: F-F2P7-004 Pass-7 remediation — partition harness non-vacuity mislabeling corrected: proptest_vp041_partition_invariant holds trivially by the complement derivation (unsupported = KNOWN \\ supported); proptest_vp041_oracle_cross_check is the non-vacuous guard. VP table partition row, Architecture Anchors, VP Anchors updated. 2026-07-01"
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
`SUPPORTED_PORTS` (which is the full set of ports wirerust actively dissects by any
mechanism: ports 502, 20000, 44818, 443, 8443, 80, 8080 correspond to `DispatchTarget`
variants in `dispatcher.rs::classify()`; port 53 corresponds to the DNS decode-loop path
in `main.rs` via `dns_analyzer.can_decode()` — there is no `DispatchTarget::Dns` variant
and no port-53 rule in `classify()`; DNS/53 and ARP being non-mirroring with respect to
`classify()` is PERMANENT and BY DESIGN, not drift),
plus the ARP entry (which is supported via `DecodedFrame::Arp` outside the dispatcher and
therefore cannot be detected by port intersection). `unsupported_protocols()` returns the
complement: every entry NOT in `supported_protocols()`. Both functions are pure and
deterministic; they have no I/O, no mutable state, and no runtime dependencies.

This contract guards against drift between what the catalog reports as "supported" and what
`supported_protocols()` computes from `SUPPORTED_PORTS`. VP-041 guards the consistency
between `supported_protocols()` and `SUPPORTED_PORTS` only (ADR-012 Decision 5). It does
NOT detect drift between `classify()` and `SUPPORTED_PORTS`: the VP-041 oracle itself
references `SUPPORTED_PORTS`, so it cannot observe `classify()`-vs-`SUPPORTED_PORTS` drift.
Keeping `classify()` aligned with `SUPPORTED_PORTS` is an UNENFORCED documented convention
(ADR-012 Decision 5); drift there is not auto-detected by any VP.

## Related BCs

- BC-2.18.004 — composes with (partition invariant: supported ∪ unsupported == KNOWN_PROTOCOLS)
- BC-2.18.001 — depends on (the `--supported` / `--unsupported` filter flags in terminal output call these functions)
- BC-2.18.002 — depends on (same for JSON output)

## Preconditions

1. `supported_protocols()` is called as a pure function — no mutable state, no I/O.
2. `KNOWN_PROTOCOLS` is a non-empty static array (compile-time constant).
3. `SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 443, 8443, 80, 8080, 53]` is the compile-time constant equal to the full set of ports wirerust actively dissects: ports 502, 20000, 44818, 443, 8443, 80, 8080 correspond to `DispatchTarget` variants in `classify()`; port 53 is handled via the DNS decode-loop path in `main.rs` (`dns_analyzer.can_decode()`) with no `DispatchTarget::Dns` variant and no port-53 rule in `classify()`. DNS/53 not mirroring `classify()` is PERMANENT and BY DESIGN.

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

1. `SUPPORTED_PORTS` is the compile-time constant equal to the full set of ports wirerust actively dissects by any mechanism (ADR-012 Decision 5). It is NOT a pure mirror of `dispatcher.rs::classify()` port-fallback rules: ports 502, 20000, 44818, 443, 8443, 80, 8080 correspond to `DispatchTarget` variants in `classify()`; port 53 corresponds to the DNS decode-loop path in `main.rs` (`dns_analyzer.can_decode()`) — there is no `DispatchTarget::Dns` variant and no port-53 rule in `classify()`. DNS/53 and ARP being non-mirroring with respect to `classify()` is PERMANENT and BY DESIGN, not drift. If a new `DispatchTarget` variant and port rule is added to `classify()`, `SUPPORTED_PORTS` MUST be updated. VP-041 guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency ONLY. `classify()`-vs-`SUPPORTED_PORTS` drift on the ports 502..8080 entries is an UNENFORCED documented convention: the VP-041 oracle references `SUPPORTED_PORTS` directly and therefore cannot detect when `classify()` and `SUPPORTED_PORTS` diverge for those entries. Drift there is not auto-detected by any VP.
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
| EC-005 | A port is already present in `SUPPORTED_PORTS` (e.g., port 20000 for DNP3 — already a member before this BC was authored) | `SUPPORTED_PORTS` already contains 20000; no change to the supported set |
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
| VP-041 | Oracle cross-check (`proptest_vp041_oracle_cross_check`): for each entry in KNOWN_PROTOCOLS, `entry ∈ supported_protocols() ⟺ entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) \|\| entry.name=="ARP"`. Oracle is computed INDEPENDENTLY — it does NOT call `supported_protocols()` or `unsupported_protocols()` (non-vacuous). Guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency. | proptest: `proptest_vp041_oracle_cross_check` |
| VP-041 | Partition/disjointness (`proptest_vp041_partition_invariant`): `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS` and `supported_protocols() ∩ unsupported_protocols() == ∅`. Verifies union-completeness and disjointness of the two function outputs; holds trivially by the complement derivation (`unsupported = KNOWN \ supported`). `proptest_vp041_oracle_cross_check` provides the non-vacuous guard. | proptest: `proptest_vp041_partition_invariant` |
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
| Stories | STORY-151 (F3 feature-protocol-coverage — src/protocols.rs KNOWN_PROTOCOLS catalog + SUPPORTED_PORTS + pure-core partition functions) |

## Architecture Anchors

- `src/protocols.rs` — `SUPPORTED_PORTS: &[u16]` compile-time constant: `&[502, 20000, 44818, 443, 8443, 80, 8080, 53]`; doc-comment MUST list each port and its dissection path: either a `DispatchTarget` variant (for ports handled by `dispatcher.rs::classify()`) or "decode-loop" (for ports dissected outside `classify()`, e.g., `53 → DNS decode-loop in main.rs, no DispatchTarget variant`). Port-independent protocols (ARP) are flagged separately via a special case in `supported_protocols()`.
- `src/protocols.rs` — `pub fn supported_protocols() -> Vec<&'static KnownProtocol>` — returns entries matching SUPPORTED_PORTS intersection OR ARP special case
- `src/protocols.rs` — `pub fn unsupported_protocols() -> Vec<&'static KnownProtocol>` — returns complement of `supported_protocols()` within `KNOWN_PROTOCOLS`
- `src/protocols.rs` — `pub fn all_protocols() -> &'static [KnownProtocol]` — returns full `KNOWN_PROTOCOLS` slice
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_oracle_cross_check` (oracle: `entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) || entry.name=="ARP"`; oracle computed independently, does NOT call `supported_protocols()` or `unsupported_protocols()` — non-vacuous)
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_partition_invariant` (verifies `supported ∪ unsupported == KNOWN_PROTOCOLS` and `supported ∩ unsupported == ∅`; holds trivially by the complement derivation (`unsupported = KNOWN \ supported`); non-vacuous guard is `proptest_vp041_oracle_cross_check`)

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-041 — `proptest_vp041_oracle_cross_check`: per-entry canonical membership predicate; guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency; oracle computed independently (non-vacuous — does NOT call `supported_protocols()`)
- VP-041 — `proptest_vp041_partition_invariant`: partition/disjointness of `supported_protocols()` and `unsupported_protocols()` over `KNOWN_PROTOCOLS`; holds trivially by the complement derivation (`unsupported = KNOWN \ supported`); non-vacuous guard is `proptest_vp041_oracle_cross_check`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | read-only (`KNOWN_PROTOCOLS` and `SUPPORTED_PORTS` are `&'static` compile-time constants) |
| **Deterministic** | yes (same binary always produces same result) |
| **Thread safety** | yes (no mutable state) |
| **Overall classification** | pure |
