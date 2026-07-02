---
document_type: story
story_id: STORY-151
title: "`src/protocols.rs` — KNOWN_PROTOCOLS Static Catalog + KnownProtocol Struct + SUPPORTED_PORTS + Pure-Core Functions + VP-041 proptest harnesses"
epic_id: E-21
wave: 67
points: 8
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-protocol-coverage
github_issue: feature-protocol-coverage
subsystems: [SS-18]
target_module: protocols
depends_on: []
blocks: [STORY-152, STORY-154]
behavioral_contracts:
  - BC-2.18.003
  - BC-2.18.004
verification_properties:
  - VP-041
assumption_validations: []
risk_mitigations: []
# BC status: all BCs authored and anchored (F2 convergence complete)
# DF-CANONICAL-FRAME-HOLDOUT-001: This story carries canonical-value ACs per obligation #2.
inputs:
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md
  - .factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
  - .factory/specs/architecture/ss-18-protocol-coverage-catalog.md
input-hash: "4a98223"
---

# STORY-151: `src/protocols.rs` — KNOWN_PROTOCOLS Static Catalog + Pure-Core Functions + VP-041

> **DF-CANONICAL-FRAME-HOLDOUT-001 STORY**: This story contains mandatory canonical-value ACs
> for protocol framing invariants (port numbers and EtherType constants) that must be verified
> against authoritative protocol specifications.

## Narrative

**As a** wirerust developer implementing the Protocol Coverage Catalog (CAP-18),
**I want** a new `src/protocols.rs` module containing the `KNOWN_PROTOCOLS` static array,
the `KnownProtocol` struct, the `SUPPORTED_PORTS` compile-time constant, and the three
pure-core partition functions (`all_protocols()`, `supported_protocols()`, `unsupported_protocols()`),
**so that** the CLI subcommand and gap report (STORY-152, STORY-154) have a single, verified,
compile-time source of truth for what protocols wirerust knows about versus what it dissects.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.18.003 | v1.3 | `supported_protocols()` Returns Exactly the SUPPORTED_PORTS-Intersecting Entries Plus ARP; `unsupported_protocols()` Returns the Complement | Primary: defines `SUPPORTED_PORTS`, `supported_protocols()`, `unsupported_protocols()`, ARP special case |
| BC-2.18.004 | v1.2 | Catalog Partition Invariant — Supported ∪ Unsupported == KNOWN_PROTOCOLS and Disjoint | Primary: formalizes the union/disjoint invariant; both VP-041 harnesses |

## Acceptance Criteria

### AC-151-001: `KnownProtocol` struct and `ProtocolCategory`/`Transport` enums compiled and exported
**Traces to:** BC-2.18.003 v1.3 Precondition 2; BC-2.18.004 v1.2 Precondition 1; ADR-012 Decision 1, Decision 7

`src/protocols.rs` defines:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolCategory { ICS, IT }  // EXACTLY two variants (no L2 variant — ADR-012 Decision 7)

#[derive(Debug, Clone, PartialEq)]
pub enum Transport { Tcp, Udp, LinkLayer }  // Note: distinct from dispatcher's TransportProto

pub struct KnownProtocol {
    pub name: &'static str,
    pub category: ProtocolCategory,
    pub transport: Transport,
    pub canonical_ports: &'static [u16],   // empty slice for LinkLayer entries
    pub ethertype: Option<u16>,            // Some(0x88B8) for GOOSE; None for TCP/UDP; None for ARP
    pub port_detectable: bool,             // false for transport=LinkLayer
    pub description: &'static str,
}
```

`src/lib.rs` gains `pub mod protocols;` (one line).

(traces to BC-2.18.003 v1.3 PC-2; BC-2.18.004 v1.2 PC-1; ADR-012 Decision 1)

**Red-Gate tests:**
- `test_BC_2_18_struct_fields_compile` — construct a `KnownProtocol` value; assert all fields accessible
- `test_BC_2_18_category_variants_exactly_two` — `ProtocolCategory::ICS` and `ProtocolCategory::IT` compile; no third variant

### AC-151-002: `SUPPORTED_PORTS` compile-time constant contains exactly the 8 actively-dissected ports
**Traces to:** BC-2.18.003 v1.3 Precondition 3, Invariant 1; ADR-012 Decision 5

```rust
/// Compile-time constant equal to the full set of ports wirerust actively dissects by any
/// mechanism. Port → dissection path:
/// - 502  → DispatchTarget::Modbus in dispatcher.rs::classify()
/// - 20000 → DispatchTarget::Dnp3 in dispatcher.rs::classify()
/// - 44818 → DispatchTarget::Enip in dispatcher.rs::classify()
/// - 443, 8443 → DispatchTarget::Tls in dispatcher.rs::classify()
/// - 80, 8080 → DispatchTarget::Http in dispatcher.rs::classify()
/// - 53 → DNS decode-loop in main.rs (dns_analyzer.can_decode()); NO DispatchTarget::Dns variant
///          in classify(). DNS/53 not mirroring classify() is PERMANENT and BY DESIGN.
/// ARP is NOT in this list; it is handled via DecodedFrame::Arp (ARP special case in
/// supported_protocols()).
pub const SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 443, 8443, 80, 8080, 53];
```

(traces to BC-2.18.003 v1.3 PC-3, Invariant 1; ADR-012 Decision 5)

**Red-Gate test:**
- `test_BC_2_18_003_supported_ports_len` — `SUPPORTED_PORTS.len() == 8`
- `test_BC_2_18_003_supported_ports_contains_canonical` — 502, 20000, 44818, 443, 8443, 80, 8080, 53 each present

> **DF-CANONICAL-FRAME-HOLDOUT-001:** The following test uses canonical values from
> authoritative protocol specifications:
> - Modbus/TCP port 502: IANA registry + Modbus Application Protocol v1.1b3 §4.3.1 "Well-Known
>   TCP Port 0+502"
> - DNP3 port 20000: IEEE Std 1815-2012 §10.3.2 "TCP Port 20000"
> - BACnet/IP UDP 47808: ASHRAE 135-2016 Annex J §J.2.1 "UDP Port Number 47808 (0xBAC0)"
> - DNS UDP 53: RFC 1035 §4.2.1 "Server" (port 53)
>
> `test_BC_2_18_003_supported_ports_canonical` — asserts `SUPPORTED_PORTS.contains(&502)` (Modbus,
> IANA/Modbus App Protocol v1.1b3 §4.3.1), `SUPPORTED_PORTS.contains(&20000)` (DNP3, IEEE
> Std 1815-2012 §10.3.2), `SUPPORTED_PORTS.contains(&53)` (DNS, RFC 1035 §4.2.1)

### AC-151-003: `KNOWN_PROTOCOLS` static array contains exactly 30 entries in catalog-declaration order
**Traces to:** BC-2.18.004 v1.2 Postconditions 1–5; BC-2.18.003 v1.3 PC-2; ADR-012 Decision 1, Decision 4

`KNOWN_PROTOCOLS` is a `pub const &[KnownProtocol]` with exactly 30 entries.
Entries must appear in the order: first the 7 supported (Modbus/TCP, DNP3, EtherNet/IP+CIP,
TLS, ARP, DNS, HTTP), then the 23 unsupported (including the 5 LinkLayer entries and the 4
port-102 entries). This order is the catalog-declaration order per BC-2.18.003 v1.3 PC-2.

The 7 supported entries:
1. Modbus/TCP — ICS, TCP, `canonical_ports: &[502]`, `port_detectable: true`, `supported` via SUPPORTED_PORTS
2. DNP3 — ICS, TCP, `canonical_ports: &[20000]`, `port_detectable: true`
3. EtherNet/IP+CIP — ICS, TCP, `canonical_ports: &[44818]`, `port_detectable: true`
4. TLS — IT, TCP, `canonical_ports: &[443, 8443]`, `port_detectable: true`
5. ARP — IT, LinkLayer, `canonical_ports: &[]`, `ethertype: None`, `port_detectable: false` (ARP special-case supported via DecodedFrame::Arp)
6. DNS — IT, UDP, `canonical_ports: &[53]`, `port_detectable: true`
7. HTTP — IT, TCP, `canonical_ports: &[80, 8080]`, `port_detectable: true`

The 5 LinkLayer unsupported entries must carry:
- IEC 61850 GOOSE: `ethertype: Some(0x88B8)` (35000 decimal)
- IEC 61850 Sampled Values: `ethertype: Some(0x88BA)` (35002 decimal)
- PROFINET-RT/DCP: `ethertype: Some(0x8892)` (34962 decimal)
- EtherCAT: `ethertype: Some(0x88A4)` (34980 decimal)
- Ethernet POWERLINK: `ethertype: Some(0x88AB)` (34987 decimal)

All 5 carry `canonical_ports: &[]`, `port_detectable: false`.

The 4 port-102 unsupported TCP entries: S7comm, S7comm-plus, IEC 61850 MMS, ICCP-TASE.2
all have `canonical_ports: &[102]`, `port_detectable: true`, `transport: TCP`.

(traces to BC-2.18.004 v1.2 PC-1..5; BC-2.18.003 v1.3 PC-2; ADR-012 Decision 1/4)

**Red-Gate tests:**
- `test_BC_2_18_003_known_protocols_len` — `KNOWN_PROTOCOLS.len() == 30`
- `test_BC_2_18_003_arp_linkLayer_port_detectable_false` — ARP entry: `canonical_ports.is_empty() && !port_detectable`

> **DF-CANONICAL-FRAME-HOLDOUT-001 — EtherType canonical values:**
> These EtherType constants are asserted from authoritative IEEE RA registry entries:
> - GOOSE `0x88B8` = 35000 decimal: IEC 61850-8-1 §4, IEEE RA EtherType registry entry "IEC GOOSE"
> - POWERLINK `0x88AB` = 34987 decimal: IEEE RA EtherType registry "ETHERNET Powerlink" (EPSG assignment); confirmed by Wireshark etypes.h `ETHERTYPE_EPL_V2 = 0x88AB`; confirmed by IETF RFC/YANG `ietf-ethertypes` module value 34987
>
> `test_BC_2_18_003_goose_ethertype_canonical` — GOOSE entry: `ethertype == Some(35000)` (0x88B8;
> IEC 61850-8-1 §4; IEEE RA registry "IEC GOOSE")
>
> `test_BC_2_18_003_powerlink_ethertype_canonical` — POWERLINK entry: `ethertype == Some(34987)`
> (0x88AB; IEEE RA registry "ETHERNET Powerlink"; EPSG assignment; Wireshark ETHERTYPE_EPL_V2;
> IETF ietf-ethertypes value 34987)
>
> `test_BC_2_18_003_ethercat_ethertype_canonical` — EtherCAT entry: `ethertype == Some(34980)`
> (0x88A4; IEEE RA EtherType registry "EtherCAT Technology Group"); wrong-value guard: assert
> `Some(34980)` NOT `Some(34962)` (PROFINET) and NOT `Some(35000)` (GOOSE)
>
> `test_BC_2_18_003_profinet_ethertype_canonical` — PROFINET-RT/DCP entry: `ethertype == Some(34962)`
> (0x8892; IEEE RA registry "PROFINET Acyclic Real-Time / PROFINET-DCP"); wrong-value guard: assert
> `Some(34962)` NOT `Some(34980)` (EtherCAT)
>
> `test_BC_2_18_003_sv_ethertype_canonical` — IEC 61850 SV entry: `ethertype == Some(35002)`
> (0x88BA; IEC 61850-8-1 §4); GOOSE-transposition guard: `Some(35002)` MUST NOT equal `Some(35000)`
> (GOOSE 0x88B8); SV is 0x88BA, not 0x88B8
>
> `test_BC_2_18_003_bacnet_udp_canonical` — BACnet/IP entry: `transport == Transport::Udp && canonical_ports == &[47808]` (ASHRAE 135-2016 Annex J §J.2.1 UDP port 0xBAC0 = 47808)

### AC-151-004: `all_protocols()` returns full `KNOWN_PROTOCOLS` slice
**Traces to:** BC-2.18.004 v1.2 PC-1; BC-2.18.003 v1.3 Invariant 2

```rust
pub fn all_protocols() -> &'static [KnownProtocol] {
    KNOWN_PROTOCOLS
}
```

Pure function. No I/O, no mutable state.

(traces to BC-2.18.004 v1.2 PC-1)

**Red-Gate test:**
- `test_BC_2_18_004_all_protocols_len` — `all_protocols().len() == KNOWN_PROTOCOLS.len()`

### AC-151-005: `supported_protocols()` returns exactly the 7 SUPPORTED_PORTS-intersecting entries plus ARP
**Traces to:** BC-2.18.003 v1.3 PC-1, PC-3, Invariant 3; ADR-012 Decision 5

```rust
pub fn supported_protocols() -> Vec<&'static KnownProtocol> {
    KNOWN_PROTOCOLS.iter().filter(|p| {
        p.canonical_ports.iter().any(|port| SUPPORTED_PORTS.contains(port)) || p.name == "ARP"
    }).collect()
}
```

- Returns exactly 7 entries: Modbus/TCP, DNP3, EtherNet/IP+CIP, TLS, ARP, DNS, HTTP
- ARP is in the result because of the explicit `p.name == "ARP"` special case (BC-2.18.003 PC-3, Invariant 3)
- The ARP special case MUST NOT be omitted
- Pure function; no I/O; same call always returns the same result

(traces to BC-2.18.003 v1.3 PC-1, PC-3, Invariants 2–3; BC-2.18.004 v1.2 PC-1..5)

**Red-Gate tests:**
- `test_BC_2_18_003_supported_protocols_len` — `supported_protocols().len() == 7`
- `test_BC_2_18_003_arp_in_supported_set` — `supported_protocols()` contains the entry with `name == "ARP"`
- `test_BC_2_18_003_supported_ports_mirror` — for every port in `SUPPORTED_PORTS` (except 53, which maps to DNS), `supported_protocols()` contains an entry with that port in `canonical_ports`
- `test_BC_2_18_003_bacnet_unsupported` — BACnet/IP (port 47808, not in SUPPORTED_PORTS) is NOT in `supported_protocols()`

### AC-151-006: `unsupported_protocols()` is the exact complement of `supported_protocols()` within `KNOWN_PROTOCOLS`
**Traces to:** BC-2.18.003 v1.3 PC-2, Invariants 4–5; BC-2.18.004 v1.2 PC-1..5; ADR-012 Decision 5

```rust
pub fn unsupported_protocols() -> Vec<&'static KnownProtocol> {
    let supported: Vec<_> = supported_protocols().iter().map(|p| p.name).collect();
    KNOWN_PROTOCOLS.iter().filter(|p| !supported.contains(&p.name)).collect()
}
```

`unsupported_protocols()` MUST NOT be a separate hand-maintained list; it must be derived as the
complement of `supported_protocols()` within `KNOWN_PROTOCOLS` (BC-2.18.003 Invariant 4).

Returns exactly `KNOWN_PROTOCOLS.len() - supported_protocols().len()` entries = 23.

(traces to BC-2.18.003 v1.3 PC-2, Invariants 4–5; BC-2.18.004 v1.2 PC-1..5)

**Red-Gate tests:**
- `test_BC_2_18_003_partition_len` — `supported_protocols().len() + unsupported_protocols().len() == KNOWN_PROTOCOLS.len()`
- `test_BC_2_18_004_disjoint` — `supported_protocols()` and `unsupported_protocols()` share no entry (by name)
- `test_BC_2_18_004_no_phantom_entries` — every entry in `unsupported_protocols()` has a name present in `KNOWN_PROTOCOLS`

### AC-151-007: VP-041 proptest harnesses — oracle cross-check (non-vacuous) and partition/disjointness
**Traces to:** BC-2.18.004 v1.2 Invariant 4; BC-2.18.003 v1.3 VP table; ADR-012 Decision 5

Two proptest harnesses in `tests/protocols_tests.rs` inside `mod story_151 { ... }`:

**Harness A — `proptest_vp041_oracle_cross_check`** (non-vacuous):
For each entry in `KNOWN_PROTOCOLS`, assert:
`entry ∈ supported_protocols() ⟺ entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) || entry.name == "ARP"`
The oracle is computed INDEPENDENTLY — it does NOT call `supported_protocols()` or
`unsupported_protocols()`. This guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency.

**Harness B — `proptest_vp041_partition_invariant`**:
Assert `supported_protocols().len() + unsupported_protocols().len() == KNOWN_PROTOCOLS.len()`
AND that the name sets are disjoint.

Both harnesses MUST pass. `proptest_vp041_oracle_cross_check` is the non-vacuous guard
(oracle computed independently). `proptest_vp041_partition_invariant` holds trivially
by the complement derivation (unsupported = KNOWN \ supported).

(traces to BC-2.18.004 v1.2 Invariant 4; BC-2.18.003 v1.3 VP table)

### AC-151-008: ARCH-INDEX document-map component count doc-fix (F3-carry)
**Traces to:** ADR-012 Decision 1; BC-2.18.003 v1.3 Architecture Anchors; DF-SIBLING-SWEEP-001

F3-carry item: ARCH-INDEX.md Document Map still shows "24 components" (pre-SS-18). With C-26
`src/protocols.rs` (SS-18, new in F2 design layer) and C-27 being the next, the component count
is 26. This story adds the `pub mod protocols;` declaration to `src/lib.rs`, which is the
implementation anchor for C-26.

Task: Update `ARCH-INDEX.md` Document Map "24 components" → "26 components" as part of this
story's implementation.

(traces to ADR-012 Decision 1; DF-SIBLING-SWEEP-001 sibling sweep)

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `KNOWN_PROTOCOLS: &[KnownProtocol]` static array | `src/protocols.rs` (new C-26) | Pure |
| `SUPPORTED_PORTS: &[u16]` compile-time constant | `src/protocols.rs` (new C-26) | Pure |
| `KnownProtocol` struct + `ProtocolCategory` + `Transport` enums | `src/protocols.rs` (new C-26) | Pure |
| `all_protocols()` pure function | `src/protocols.rs` (new C-26) | Pure |
| `supported_protocols()` pure function | `src/protocols.rs` (new C-26) | Pure |
| `unsupported_protocols()` pure function | `src/protocols.rs` (new C-26) | Pure |
| VP-041 proptest harnesses | `tests/protocols_tests.rs` (new test file) | Pure |
| `pub mod protocols;` declaration | `src/lib.rs` | N/A (declaration) |

SS-18 (Protocol Coverage Catalog) — pure-core only; no I/O; no external dependencies.
MUST NOT import from `src/dispatcher.rs` (that dependency direction is forbidden — SS-18 is
a pure-core catalog, not a dispatcher consumer).

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-151-1 | BC-2.18.003 EC-001 | ARP entry: `canonical_ports: &[]` — port intersection gives ∅ | ARP still in `supported_protocols()` via `p.name == "ARP"` special case |
| EC-151-2 | BC-2.18.003 EC-002 | TLS has two ports (443, 8443) — both in SUPPORTED_PORTS | TLS appears ONCE in `supported_protocols()` |
| EC-151-3 | BC-2.18.003 EC-003 | BACnet/IP port 47808 NOT in SUPPORTED_PORTS | BACnet/IP in `unsupported_protocols()` |
| EC-151-4 | BC-2.18.003 EC-004 | GOOSE has `canonical_ports: &[]` and is NOT ARP | GOOSE in `unsupported_protocols()` |
| EC-151-5 | BC-2.18.003 EC-007 | Port-102 entries (S7comm, S7comm-plus, MMS, ICCP) | All four in `unsupported_protocols()` (102 not in SUPPORTED_PORTS) |
| EC-151-6 | BC-2.18.004 EC-005 | Port-102 collision (four entries with `canonical_ports: &[102]`) | All four in unsupported; partition still valid |
| EC-151-7 | BC-2.18.004 EC-004 | ARP is only supported entry with `canonical_ports: &[]` | Partition valid; ARP special case creates no phantom entry |

## Estimated Complexity

**Story points: 8** (new greenfield file src/protocols.rs; ~30 catalog entries requiring
careful canonical value lookup; 3 pure functions; 2 VP-041 proptest harnesses; 10+ unit tests;
canonical EtherType assertions per DF-CANONICAL-FRAME-HOLDOUT-001; ARCH-INDEX doc-fix;
lib.rs declaration; new tests file)

## Token Budget Estimate

| Context source | Estimated tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC-2.18.003 (v1.3) | ~5,500 |
| BC-2.18.004 (v1.2) | ~3,500 |
| ADR-012 (Decisions 1, 4, 5, 7) | ~6,000 |
| ss-18-protocol-coverage-catalog.md (catalog reference) | ~8,000 |
| src/dispatcher.rs (SUPPORTED_PORTS port reference check) | ~6,000 |
| src/lib.rs (one-line addition) | ~200 |
| Tool outputs (cargo check cycles) | ~2,000 |
| **Total estimate** | **~33,700** |

Fits comfortably within a 200k context window (~17%). New file — no pre-existing code to read.

## Tasks

1. **Write Red-Gate tests first (TDD Step 1 — all must FAIL before implementation)**
   Create `tests/protocols_tests.rs` with `mod story_151 { ... }` wrapper containing:
   - `test_BC_2_18_struct_fields_compile` — compile check
   - `test_BC_2_18_category_variants_exactly_two` — no L2 variant
   - `test_BC_2_18_003_supported_ports_len` — len == 8
   - `test_BC_2_18_003_supported_ports_contains_canonical` — 8 ports present
   - `test_BC_2_18_003_supported_ports_canonical` — canonical port values (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_003_known_protocols_len` — len == 30
   - `test_BC_2_18_003_arp_linkLayer_port_detectable_false` — ARP fields
   - `test_BC_2_18_003_goose_ethertype_canonical` — GOOSE ethertype == Some(35000) (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_003_powerlink_ethertype_canonical` — POWERLINK ethertype == Some(34987) (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_003_ethercat_ethertype_canonical` — EtherCAT ethertype == Some(34980) (0x88A4; IEEE RA; DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_003_profinet_ethertype_canonical` — PROFINET-RT/DCP ethertype == Some(34962) (0x8892; IEEE RA; DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_003_sv_ethertype_canonical` — IEC 61850 SV ethertype == Some(35002) (0x88BA; NOT GOOSE 35000; DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_003_bacnet_udp_canonical` — BACnet/IP transport=Udp, port=47808 (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_18_004_all_protocols_len` — all_protocols().len() == len(KNOWN_PROTOCOLS)
   - `test_BC_2_18_003_supported_protocols_len` — 7
   - `test_BC_2_18_003_arp_in_supported_set` — ARP present
   - `test_BC_2_18_003_supported_ports_mirror` — port mirror check
   - `test_BC_2_18_003_bacnet_unsupported` — BACnet/IP absent from supported
   - `test_BC_2_18_003_partition_len` — sum == 30
   - `test_BC_2_18_004_disjoint` — no shared entries
   - `test_BC_2_18_004_no_phantom_entries` — no phantom entries
   All tests MUST FAIL (file doesn't exist yet); proptest harnesses will fail to compile.

2. **Create `src/protocols.rs` with enums, struct, constants, catalog (AC-151-001 through AC-151-003)**
   - Define `ProtocolCategory`, `Transport` enums
   - Define `KnownProtocol` struct
   - Define `SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 443, 8443, 80, 8080, 53]` with
     the exact doc-comment from AC-151-002 listing each port's dissection path
   - Define `KNOWN_PROTOCOLS` with exactly 30 entries (7 supported + 23 unsupported, in
     catalog-declaration order). Use canonical EtherType values from research:
     GOOSE=0x88B8(35000), SV=0x88BA(35002), PROFINET=0x8892(34962), EtherCAT=0x88A4(34980),
     POWERLINK=0x88AB(34987)
   - Add `pub mod protocols;` to `src/lib.rs`
   - Verify: unit tests compile; struct/constant tests turn GREEN; proptest harnesses still fail

3. **Implement `all_protocols()`, `supported_protocols()`, `unsupported_protocols()` (AC-151-004 through AC-151-006)**
   - Implement all three pure functions with the exact logic from ACs
   - The ARP special case (`p.name == "ARP"`) MUST be explicit in `supported_protocols()`
   - `unsupported_protocols()` MUST be derived as complement; not a hand-maintained list
   - Verify: all unit tests turn GREEN; proptest harnesses still fail

4. **Implement VP-041 proptest harnesses (AC-151-007)**
   - Add `proptest_vp041_oracle_cross_check` — oracle computed INDEPENDENTLY (no call to
     `supported_protocols()` or `unsupported_protocols()`)
   - Add `proptest_vp041_partition_invariant` — verifies union-completeness and disjointness
   - Both harnesses MUST pass
   - Verify: `cargo test --all-targets` ALL GREEN

5. **ARCH-INDEX doc-fix (AC-151-008)**
   - Update `specs/architecture/ARCH-INDEX.md` Document Map "24 components" → "26 components"
   - This reflects C-25 (`src/reader.rs` pcapng reader from E-19) and C-26 (`src/protocols.rs` new)

6. **Full regression sweep**
   - `cargo test --all-targets` — ALL tests GREEN (protocols_tests.rs + all existing tests)
   - `cargo clippy --all-targets -- -D warnings` — zero warnings
   - `cargo fmt --check` — clean

7. **Micro-commit and PR** targeting `develop` (wave 67)

## Previous Story Intelligence

**N/A — first story in E-21 (feature-protocol-coverage)**

No predecessor stories in this epic. The catalog is entirely greenfield (new file).

Key lessons from analogous prior work:
- **From STORY-071 (mitre.rs catalog):** wirerust already has a pattern for compile-time
  static catalogs in `src/mitre.rs`. The `KNOWN_PROTOCOLS` catalog follows the same pattern.
  Read `src/mitre.rs` for style guidance on `&'static str` fields and const array layout.
- **From E-3 (dispatcher):** `src/dispatcher.rs::classify()` uses ports 502, 20000, 44818,
  443, 8443, 80, 8080. Verify these match `SUPPORTED_PORTS` before submitting. Port 53 is
  NOT in `classify()` — it's in the DNS decode-loop path. This is BY DESIGN (ADR-012 Decision 5).
- **Critical:** `Transport` enum in `src/protocols.rs` has THREE variants (Tcp, Udp, LinkLayer).
  This is different from `TransportProto` in `src/dispatcher.rs` which has only TWO (Tcp, Udp).
  MUST NOT import one from the other (forbidden boundary per BC-2.05.010 PC-4).

## Architecture Compliance Rules

Source: `architecture/module-decomposition.md` + ADR-012 + BC-2.18.003/004

1. **`ProtocolCategory` has EXACTLY two variants: `ICS` and `IT`** — no `L2` variant (ADR-012 Decision 7; BC-2.18.002 PC-3). L2-ness is expressed via `transport: Transport::LinkLayer` and `port_detectable: false`, not via a third category.
2. **`SUPPORTED_PORTS` MUST have the exact doc-comment** listing each port and its dissection path (AC-151-002; BC-2.18.003 Architecture Anchor). The distinction between `DispatchTarget` variants (502, 20000, 44818, 443, 8443, 80, 8080) and the DNS decode-loop path (53) MUST be documented.
3. **ARP special case MUST be explicit** — `|| p.name == "ARP"` in `supported_protocols()` body (BC-2.18.003 Invariant 3). No implicit handling.
4. **`unsupported_protocols()` MUST be derived as complement** of `supported_protocols()` within `KNOWN_PROTOCOLS` (BC-2.18.003 Invariant 4). A hand-maintained list is a spec violation.
5. **No import from `src/dispatcher.rs`** — SS-18 must not depend on SS-05. The `Transport` enum in `protocols.rs` is independent of `TransportProto` in `dispatcher.rs` (BC-2.05.010 PC-4 pure-core boundary rule).
6. **GOOSE EtherType is `Some(0x88B8)` = `Some(35000)`** — not 34992 (the pre-F2 erroneous value that BC-2.18.002 v1.1 corrected in F-F2P1-001). Use 35000 decimal (DF-CANONICAL-FRAME-HOLDOUT-001).
7. **POWERLINK EtherType is `Some(0x88AB)` = `Some(34987)`** — the EPSG V2 value, not the obsolete V1 `0x3E3F` (confirmed by IEEE RA registry + Wireshark ETHERTYPE_EPL_V2; DF-CANONICAL-FRAME-HOLDOUT-001).
8. **Test namespace isolation (DF-TEST-NAMESPACE-001):** ALL test functions MUST be inside `mod story_151 { ... }` in `tests/protocols_tests.rs`.

## Library & Framework Requirements

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `proptest` | (existing, already in Cargo.toml as dev-dep) | VP-041 proptest harnesses |

No new dependencies. `src/protocols.rs` is a pure-core module with ZERO runtime crate dependencies
(only std types: `&'static str`, `&'static [u16]`, `Option<u16>`, `bool`).

**Forbidden dependencies:** `src/protocols.rs` MUST NOT depend on `dispatcher`, `analyzer/*`,
`reassembly/*`, `reporter/*`, `mitre`, `findings`, or any other wirerust module. It is a
standalone pure-core catalog.

## File Structure Requirements

| File | Change Type | Purpose |
|------|------------|---------|
| `src/protocols.rs` | **NEW** | `KnownProtocol` struct, `ProtocolCategory`/`Transport` enums, `KNOWN_PROTOCOLS`, `SUPPORTED_PORTS`, `all_protocols()`, `supported_protocols()`, `unsupported_protocols()` |
| `src/lib.rs` | Modify | Add `pub mod protocols;` (one line) |
| `tests/protocols_tests.rs` | **NEW** | VP-041 proptest harnesses + unit tests, all in `mod story_151 { ... }` |
| `specs/architecture/ARCH-INDEX.md` | Modify | Document Map "24 components" → "26 components" (F3-carry doc-fix) |

## Revision History

| Version | Date | Change | Finding IDs |
|---------|------|--------|-------------|
| v1.0 | 2026-07-02 | Initial story authored for feature-protocol-coverage F3 decomposition | — |
| v1.1 | 2026-07-02 | F-F3P1-001/006 (P0/MEDIUM): Added EtherCAT, PROFINET-DCP, SV canonical EtherType tests (34980/34962/35002) to AC-151-003 canonical block and Task 1 test list. F-F3P1-005 (MEDIUM): Removed misplaced Task 0 (VP-042 carry belongs to STORY-153) and VP-042(d) note from AC-151-007. LOW: Fixed AC-151-003 cross-ref BC-2.18.001 PC-8 → BC-2.18.003 v1.3 PC-2. | F-F3P1-001, F-F3P1-005, F-F3P1-006 |
