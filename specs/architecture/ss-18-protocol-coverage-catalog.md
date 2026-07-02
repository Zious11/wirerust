---
artifact: architecture-section
section: ss-18-protocol-coverage-catalog
subsystem_id: SS-18
traces_to: ARCH-INDEX.md
version: "1.4"
status: draft
producer: architect
timestamp: 2026-07-01T00:00:00Z
feature_cycle: feature-protocol-coverage
modified:
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-2 remediation (F-F2P2-008): Ethernet POWERLINK EtherType 0x88AB verified HIGH confidence against IEEE Registration Authority EtherType registry, IETF ietf-ethertypes YANG module, and Wireshark epan/etypes.h. [unverified] tag removed from catalog row; V2 (EPSG current standard) / V1 obsolete 0x3E3F note added. Full citations in .factory/phase-f1-delta-analysis/powerlink-ethertype-verification.md."
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-5 remediation (F-F2P5-001): `supported_protocols()` Derivation section opening sentence reframed — SUPPORTED_PORTS is NOT a mirror of classify() port-fallback rules; it equals classify() TCP rules + decode-loop DNS path (port 53). DNS/53 and ARP are dissected outside classify() by design; this is permanently intentional, NOT drift."
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-2 remediation: (F-F2P2-001) Drift risk paragraph reworded — VP-041 guards supported_protocols()-vs-SUPPORTED_PORTS only; classify()-vs-SUPPORTED_PORTS is a documented convention NOT compile-time enforcement (ADR-012 Decision 5). (F-F2P2-004) L2 caveat updated to include Ethernet POWERLINK (0x88AB) as the 5th port_detectable:false catalog entry — both the prose list and the mandatory output caveat string now enumerate all 5 L2 protocols consistently with ADR-012 Decision 3a/3c."
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-1 remediation: (F-F2P1-003) ProtocolCategory enum corrected to {ICS, IT} only — removed L2 variant; L2 detection expressed by transport:LinkLayer + port_detectable:false; (F-F2P1-005) HART-IP catalog entry changed from TCP+UDP to canonical UDP:5094 with TCP noted in description; §Transport Model section added documenting single-canonical-transport decision for HART-IP/IEC-104/BACnet; (F-F2P1-006) UDP counter key changed from (Udp, dst_port) to (Udp, min(src_port, dst_port)) — ephemeral-port guard symmetric with TCP lower_port; (F-F2P1-007) CoverageGapsSummary Output tri-state: 'known' corrected to 'known-supported' (authoritative per ADR-012 Decision 2 and BC-2.12.024); (F-F2P1-012) Bounded-Resource Note off-by-one: 2×65,535 → 2×65,536 (port space 0..=65535 = 65,536 values per transport)."
---

# SS-18: Protocol Coverage Catalog

## Subsystem Purpose

SS-18 provides wirerust with two surfaces for reporting protocol coverage:

1. **Static surface** — a hand-curated compile-time catalog (`KNOWN_PROTOCOLS`) of
   ~30 ICS/IT protocols, and pure-core functions to report which are supported versus
   unsupported. Served by the `protocols` CLI subcommand.

2. **Dynamic surface** — per-(transport, port) accumulation of TCP and UDP flows that
   no dissector handled, surfaced as a `CoverageGapsSummary` report section behind
   `--coverage-gaps`. TCP unclassified flows are tracked in SS-05 (`dispatcher.rs`);
   UDP unclassified packets are tracked in the decode loop in `main.rs`. SS-18 provides
   the catalog anchor and vocabulary that gives meaning to those (transport, port)/count
   pairs.

SS-18 exists because "what wirerust knows about but cannot dissect" was previously
implicit — there was no authoritative enumeration and no way to surface live traffic
matched by known-but-unsupported protocols.

---

## Module: C-26 — `src/protocols.rs`

**Classification: PURE CORE.** The module consists entirely of compile-time constants
and pure functions: no I/O, no global mutable state, no database, no network, no
system calls. All inputs are `&'static` or function arguments; all outputs are
deterministic `&'static [KnownProtocol]` slices or `Vec<&'static KnownProtocol>`.
This makes `protocols.rs` amenable to proptest verification (VP-041) and suitable as
a Kani harness target if needed.

**Purity boundary:** The entire module is pure core. The `protocols` CLI subcommand
in `main.rs` that *prints* the output is effectful shell; the catalog lookup and
set-difference computation is pure core, testable independently.

---

## Data Model

```rust
pub enum ProtocolCategory { ICS, IT }   // NO L2 variant — L2 detection is expressed by transport:LinkLayer + port_detectable:false

pub enum Transport { Tcp, Udp, LinkLayer }

pub struct KnownProtocol {
    pub name:            &'static str,
    pub category:        ProtocolCategory,
    pub transport:       Transport,
    pub canonical_ports: &'static [u16],   // empty for L2 protocols
    pub ethertype:       Option<u16>,       // Some only for L2 protocols
    pub port_detectable: bool,              // false for L2/multicast entries
    pub description:     &'static str,
}
```

`port_detectable` is the central invariant: an entry with `port_detectable: false`
is knowable from the static catalog (`protocols --unsupported` lists it) but the
dynamic gap detector will **never** flag it, because the detector keys on
`(transport, port)` pairs and L2/multicast protocols have no port. The spec and
CLI help text MUST state this distinction explicitly (see ADR-012 Decision 3).

---

## Catalog Contents (~30 entries, hand-curated)

### Supported (7) — derived statically from dispatcher port rules

| Entry | Transport | Port(s) | Category |
|-------|-----------|---------|----------|
| Modbus/TCP | TCP | 502 | ICS |
| DNP3 | TCP | 20000 | ICS |
| EtherNet/IP + CIP | TCP | 44818 | ICS |
| TLS | TCP | 443, 8443 | IT |
| ARP | L2 | — | IT |
| DNS | UDP | 53 | IT |
| HTTP | TCP | 80, 8080 | IT |

### ICS Tier-1 Unsupported, Port-Detectable (9)

| Entry | Transport | Port(s) | Notes |
|-------|-----------|---------|-------|
| S7comm | TCP | 102 | Port 102 collision — see §Port Collisions |
| S7comm-plus | TCP | 102 | Port 102 collision |
| IEC 60870-5-104 (IEC-104) | TCP | 2404 | IANA registered |
| IEC 61850 MMS | TCP | 102 | Port 102 collision |
| BACnet/IP | UDP | 47808 | UDP-only; detectable via UDP counter in decode loop (D-320 OQ-5) |
| OPC-UA binary | TCP | 4840 | IANA registered |
| PROFINET RPC | UDP | 34962, 34963, 34964 | — |
| ICCP / TASE.2 | TCP | 102 | Port 102 collision |
| HART-IP | UDP | 5094 | TCP also supported per [P10]; UDP is canonical (initiates session); single-canonical-transport model — see §Transport Model |

### L2/Multicast — NOT Port-Detectable (5)

| Entry | Layer | EtherType | Notes |
|-------|-------|-----------|-------|
| IEC 61850 GOOSE | L2 multicast | 0x88B8 | port_detectable: false |
| IEC 61850 Sampled Values | L2 multicast | 0x88BA | port_detectable: false |
| PROFINET RT/DCP | L2 | 0x8892 | port_detectable: false |
| EtherCAT | L2 | 0x88A4 | port_detectable: false |
| Ethernet POWERLINK | L2 | 0x88AB (34987) | port_detectable: false; V2 (EPSG current standard); obsolete V1 value 0x3E3F intentionally excluded — see `.factory/phase-f1-delta-analysis/powerlink-ethertype-verification.md` |

### IT Core Unsupported (9)

| Entry | Transport | Port(s) | OT Relevance |
|-------|-----------|---------|--------------|
| SSH | TCP | 22 | Remote admin of PLCs/gateways; lateral movement |
| SMB | TCP | 445 | Engineering WS, WannaCry/Industroyer vector |
| RDP | TCP | 3389 | HMI/EWS remote access; top OT intrusion vector |
| FTP | TCP | 21 | Firmware and config transfer |
| Telnet | TCP | 23 | Legacy device CLI |
| SNMP | UDP | 161, 162 | Device management/monitoring |
| NTP | UDP | 123 | Time sync critical for SV/GOOSE/SCADA timestamps |
| SMTP | TCP | 25 | Alarm email from historians/RTUs |
| LDAP | TCP | 389 | AD auth in IT/OT DMZ |

---

## Transport Model

**Decision (F-F2P1-005):** Each `KnownProtocol` entry carries a **single canonical
transport** in its `transport: Transport` field. Dual-stack protocols (those that
historically support both TCP and UDP) are assigned one canonical transport per entry,
with the alternative noted in `description`.

**HART-IP:** canonical `Transport::Udp` (port 5094). UDP initiates the HART-IP session
and is listed first in all authoritative sources including Wireshark HART-IP [P10].
TCP is also defined by the standard but is treated as secondary; the description field
carries the note "TCP also supported per HART-IP specification".

**IEC 60870-5-104 (IEC-104):** canonical `Transport::Tcp` (port 2404). IANA registers
both TCP and UDP for port 2404 [P8], but the standard specifies TCP as primary; UDP
is documented as "rare" in the research [feature-protocol-coverage-research.md Q1].

**BACnet/IP:** canonical `Transport::Udp` (port 47808). UDP-only by default [C8][P4].

**Rationale:** A single-canonical-transport model keeps `Transport` a simple enum with
one value per entry, preserving the pure-core invariant (no collection type needed) and
making proptest strategies straightforward. VP-041 and BC-2.18.001/002 EC-007 encode
the single-transport model. If dual-transport detection becomes required in a future
cycle, the field type can change to `&'static [Transport]` — a breaking API change
documented in ADR-012.

---

## `supported_protocols()` Derivation

`supported_protocols()` MUST NOT be a separate hand-maintained name list. Drift
between it and the actual dispatcher would silently misreport coverage.

**Implementation:** `protocols.rs` declares a `SUPPORTED_PORTS: &[u16]` compile-time
constant equal to the full set of ports wirerust actively dissects — the TCP
port-fallback rules in `dispatcher.rs::classify()` PLUS the decode-loop DNS path
(port 53, dissected in `main.rs` outside `classify()`, as with ARP; port 53 has no
`DispatchTarget` variant — this is intentional, not drift; ADR-012 Decision 5):

```rust
const SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 443, 8443, 80, 8080, 53];
// ARP: handled via DecodedFrame::Arp outside the dispatcher; flagged separately.
```

`supported_protocols()` returns entries from `KNOWN_PROTOCOLS` where any
`canonical_ports` value is in `SUPPORTED_PORTS`, plus the ARP entry (L2 protocol
matched outside the dispatcher). This is a pure-core set-intersection over constants.

**Drift risk:** If a new DispatchTarget variant is added to `dispatcher.rs`, the
implementer MUST update `SUPPORTED_PORTS` in `protocols.rs`. This obligation is
recorded in ADR-012 Decision 5. VP-041 (`proptest_vp041_oracle_cross_check`)
detects `supported_protocols()`-vs-`SUPPORTED_PORTS` implementation drift only —
it verifies that `supported_protocols()` returns exactly the entries whose ports are
in `SUPPORTED_PORTS` (plus ARP). The `classify()`-vs-`SUPPORTED_PORTS` obligation
(keeping `SUPPORTED_PORTS` accurately mirroring the port-fallback rules in
`classify()`) is a **documented convention, NOT compile-time enforcement**
(ADR-012 Decision 5). A compile-time assertion may optionally enforce the count
relationship.

---

## Port 102 Collision

Four protocols share TCP port 102 in the ICS catalog: **S7comm, S7comm-plus,
IEC 61850 MMS, and ICCP/TASE.2**. All use ISO-on-TCP / TPKT framing (RFC 1006).

Consequences for the catalog:
- Port 102 maps to a *family* of protocols, not a single one.
- The `protocols` subcommand lists all four entries independently (each with
  `canonical_ports: &[102]`).
- The dynamic gap detector cannot distinguish between them — a port 102 gap
  report means "one or more of S7comm/S7comm-plus/MMS/ICCP was present and
  not dissected." The `CoverageGapsSummary` output MUST include a footnote
  citing the four-way collision (per ADR-012 Decision 3).

---

## Dynamic Detection Scope (TCP+UDP, this cycle)

The `CoverageGapsSummary` feature tracks **both TCP and UDP flows** (approved scope
D-320 OQ-5). `StreamDispatcher` handles TCP; UDP packets are tracked via a separate
lightweight counter in the decode loop in `main.rs`, outside the dispatcher. Both
surfaces contribute to the unified `CoverageGapsSummary` output. The combined structure
is keyed on `(TransportProto, u16)` — a 2-tuple of transport protocol (`Tcp` or `Udp`)
and canonical port number.

**BACnet/IP (UDP/47808) is now detectable:** BACnet is a Tier-1 catalog ICS protocol
and is UDP-only by default. With TCP+UDP scope, a BACnet gap on UDP/47808 IS flaggable
in the dynamic gap report. The `(Udp, 47808)` key is distinct from any TCP traffic on
the same port, preventing false conflation.

**Remaining structural limitation — L2/multicast protocols:** GOOSE, Sampled Values,
PROFINET-RT/DCP, EtherCAT, and Ethernet POWERLINK have no TCP/UDP port and are
structurally absent from the dynamic gap report regardless of transport scope. The
report MUST include a fixed caveat:

> "Dynamic gap detection covers TCP and UDP flows. Layer-2 protocols (e.g., GOOSE,
> Sampled Values, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK) have no TCP/UDP
> port and are not represented in the gap report. Consult
> `wirerust protocols --unsupported` for L2 protocol coverage."

**Port-102 collision still applies to TCP:** The four-way TCP/102 collision (S7comm,
S7comm-plus, IEC 61850 MMS, ICCP-TASE.2) is unresolved at the port level; a gap on
`(Tcp, 102)` cannot be attributed to a single protocol (see §Port 102 Collision).

---

## `CoverageGapsSummary` Output

The dynamic gap report is a new named section in the analysis output, produced only
when `--coverage-gaps` is passed. It groups undissected flows by `(transport, port)`
with packet counts, using Suricata-style vocabulary (per ADR-012 Decision 2):

- **known-supported** — port maps to a catalog entry whose `supported: true`
  (should never appear in gap report; sanity check only)
- **known-unsupported** — port maps to a catalog entry with `supported: false`
  (the main signal: "we know about this protocol but can't dissect it")
- **unknown** — port maps to no catalog entry

This tri-state allows operators to distinguish "there was Modbus traffic we couldn't
classify" (a bug) from "there was BACnet traffic on 47808 we don't support" (a gap)
from "there was traffic on 9600 we've never seen" (an anomaly).

---

## Subsystem Boundaries

SS-18 depends on nothing within wirerust's source tree (no imports from other modules).
It is a leaf in the dependency graph. Consuming modules:
- `src/cli.rs` (SS-12) — adds `Protocols` subcommand
- `src/main.rs` (SS-12) — adds `run_protocols()` dispatch arm
- `src/dispatcher.rs` (SS-05) — reads `SUPPORTED_PORTS` indirectly via documentation
  invariant; does NOT import from protocols.rs at runtime

---

## Bounded-Resource Note

Two counters back the dynamic gap report:

1. **TCP counter** — `StreamDispatcher.unclassified_port_counts: HashMap<(TransportProto, u16), u64>`
   (SS-05): populated only at `on_flow_close` for `DispatchTarget::None` TCP flows;
   key is `(Tcp, lower_port)`. Overhead is proportional to closed unclassified TCP flows,
   not packet volume.

2. **UDP counter** — `udp_unclassified_counts: HashMap<(TransportProto, u16), u64>` in
   the `main.rs` decode loop: populated per-packet for UDP datagrams not handled by a
   dissector; key is `(Udp, min(src_port, dst_port))` — lower-numbered of the two ports,
   approximating the service/server port and guarding against ephemeral-port noise
   (symmetric with the TCP `lower_port` convention; ADR-012 Decision 6).

Both counters use the same key type and are combined when producing `CoverageGapsSummary`.
Combined bound: at most `2 × 65,536` unique `(TransportProto, port)` keys (65,536 TCP
entries + 65,536 UDP entries — port space 0..=65535 = 65,536 values per transport).
Both maps are read-only after `run_analyze()` returns.
