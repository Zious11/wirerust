---
artifact: architecture-section
section: ss-18-protocol-coverage-catalog
subsystem_id: SS-18
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-07-01T00:00:00Z
feature_cycle: feature-protocol-coverage
---

# SS-18: Protocol Coverage Catalog

## Subsystem Purpose

SS-18 provides wirerust with two surfaces for reporting protocol coverage:

1. **Static surface** — a hand-curated compile-time catalog (`KNOWN_PROTOCOLS`) of
   ~30 ICS/IT protocols, and pure-core functions to report which are supported versus
   unsupported. Served by the `protocols` CLI subcommand.

2. **Dynamic surface** — per-(transport, port) accumulation of flows that the
   `StreamDispatcher` routed to `DispatchTarget::None`, surfaced as a
   `CoverageGapsSummary` report section behind `--coverage-gaps`. This state
   lives in SS-05 (`dispatcher.rs`); SS-18 provides the catalog anchor and
   vocabulary that gives meaning to those port/count pairs.

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
pub enum ProtocolCategory { ICS, IT, L2 }

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
| BACnet/IP | UDP | 47808 | **UDP-only; TCP-only dynamic detector structural gap** |
| OPC-UA binary | TCP | 4840 | IANA registered |
| PROFINET RPC | UDP | 34962, 34963, 34964 | — |
| ICCP / TASE.2 | TCP | 102 | Port 102 collision |
| HART-IP | TCP+UDP | 5094 | — |

### L2/Multicast — NOT Port-Detectable (5)

| Entry | Layer | EtherType | Notes |
|-------|-------|-----------|-------|
| IEC 61850 GOOSE | L2 multicast | 0x88B8 | port_detectable: false |
| IEC 61850 Sampled Values | L2 multicast | 0x88BA | port_detectable: false |
| PROFINET RT/DCP | L2 | 0x8892 | port_detectable: false |
| EtherCAT | L2 | 0x88A4 | port_detectable: false |
| Ethernet POWERLINK | L2 | 0x88AB [unverified] | port_detectable: false |

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

## `supported_protocols()` Derivation

`supported_protocols()` MUST NOT be a separate hand-maintained name list. Drift
between it and the actual dispatcher would silently misreport coverage.

**Implementation:** `protocols.rs` declares a `SUPPORTED_PORTS: &[u16]` compile-time
constant that mirrors the port-fallback rules in `dispatcher.rs::classify()`:

```rust
const SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 443, 8443, 80, 8080, 53];
// ARP: handled via DecodedFrame::Arp outside the dispatcher; flagged separately.
```

`supported_protocols()` returns entries from `KNOWN_PROTOCOLS` where any
`canonical_ports` value is in `SUPPORTED_PORTS`, plus the ARP entry (L2 protocol
matched outside the dispatcher). This is a pure-core set-intersection over constants.

**Drift risk:** If a new DispatchTarget variant is added to `dispatcher.rs`, the
implementer MUST update `SUPPORTED_PORTS` in `protocols.rs`. This obligation is
recorded in ADR-012 and gated by VP-041 (proptest set-difference correctness).
A compile-time assertion may optionally enforce the count relationship.

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

## Dynamic Detection Scope (TCP-only, this cycle)

The `CoverageGapsSummary` feature tracks **TCP flows** only. `StreamDispatcher`
handles TCP; UDP is handled outside the dispatcher.

**Structural gap — BACnet/IP (UDP/47808):** BACnet is a Tier-1 catalog ICS
protocol and is UDP-only by default. The dynamic detector will structurally never
flag a BACnet gap. The report MUST include a fixed caveat:

> "Dynamic gap detection covers TCP flows only. UDP-based protocols (e.g. BACnet/IP
> on 47808, SNMP, NTP, PROFINET RPC) and Layer-2 protocols (GOOSE, Sampled Values,
> PROFINET-RT/DCP, EtherCAT) are not represented in the gap report. Consult the
> static `protocols` catalog for the full known-protocol set."

UDP gap detection (`--coverage-gaps` extended to the decode loop in `main.rs`) is
the planned immediate follow-on cycle. BACnet/IP's UDP prevalence makes it
high-value.

---

## `CoverageGapsSummary` Output

The dynamic gap report is a new named section in the analysis output, produced only
when `--coverage-gaps` is passed. It groups undissected flows by `(transport, port)`
with packet counts, using Suricata-style vocabulary (per ADR-012 Decision 2):

- **known** — port maps to a catalog entry whose `supported: true`
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

`unclassified_port_counts: HashMap<(u16, u16), u64>` in `StreamDispatcher` (SS-05) is
the runtime state backing the dynamic gap report. It accumulates entries only when
`on_flow_close` fires for a `DispatchTarget::None` flow. Port space is bounded:
at most ~65,535 unique port-pair keys (direction-normalized). The map is populated
at flow-close granularity (not on every `on_data` call), so overhead is proportional
to the number of *closed* unclassified flows, not to packet volume.
