---
document_type: domain-capability
capability_id: CAP-18
title: "Protocol Coverage Catalog"
subsystem: SS-18
feature: feature-protocol-coverage
adr: ADR-012
introduced: v0.12.0
producer: product-owner
timestamp: 2026-07-01T19:30:00Z
---

# CAP-18: Protocol Coverage Catalog

## Description

wirerust maintains a static, hand-curated compile-time catalog (`KNOWN_PROTOCOLS`) of
approximately 30 ICS/IT protocols (including 5 L2/multicast protocols with `transport=LinkLayer ∧ port_detectable:false`; ARP is a 6th `transport=LinkLayer` entry that is supported) that are known
to the tool — including those it actively dissects and those it does not. `ProtocolCategory`
has exactly two variants: `ICS` and `IT`. Link-layer/multicast protocols (GOOSE, etc.) are
ICS-category entries with `transport=LinkLayer`, not a third category. The catalog enables two coverage surfaces:

1. **Static surface** — the `protocols` CLI subcommand lists all catalog entries with
   their name, category (`ICS` or `IT`), transport (`TCP`, `UDP`, or `LinkLayer`), canonical
   ports, supported status, and EtherType. L2-ness is expressed via `transport=LinkLayer ∧
   port_detectable:false` — there is no third `L2` category variant. Operators can filter to
   `--supported`, `--unsupported`, or `--all` and can request structured JSON output via the
   global `--json` flag.

2. **Dynamic surface** — when `--coverage-gaps` is passed to `analyze`, wirerust tracks
   TCP and UDP flows/packets that no dissector handled (keyed by `(TransportProto, u16)`)
   and appends a `CoverageGapsSummary` report section classifying each unclassified port
   using a Suricata-derived tri-state vocabulary (`known-unsupported` / `unknown` /
   `known-supported`).

The catalog is a `const &[KnownProtocol]` in `src/protocols.rs` — zero I/O, pure core,
formally verifiable (VP-041 proptest). It covers 7 supported protocols, 9 ICS Tier-1
unsupported (port-detectable), 5 L2/multicast protocols (`port_detectable: false`), and
9 IT core unsupported protocols.

Key caveats encoded in the catalog and surfaced in CLI output:
- **Port 102 hosts a four-way TCP collision** (S7comm, S7comm-plus, IEC 61850 MMS, and
  ICCP/TASE.2, all sharing ISO-on-TCP / TPKT framing) with two distinct layers, split by
  feature-s7comm (ADR-014 Decision 3, RATIFIED option (d); BC-2.18.005/006):
  - **Static catalog partition — RESOLVED.** Each of the four entries now carries an
    explicit, per-entry `Support` value: S7comm is `Supported` (full classic-S7comm
    dissection, SS-21); S7comm-plus is `DetectionOnly` (framing-level classification +
    unencrypted session-setup metadata only — "observed, not dissected"); IEC 61850 MMS
    and ICCP/TASE.2 are both `KnownUnsupported` (out of scope this cycle). The catalog can
    now correctly distinguish "dissected," "partially observed," and "neither" among the
    four, where previously it could only say "none of the four are supported."
  - **Dynamic gap classifier — NOT resolved, deferred to F4.** `main.rs::lookup_
    protocol_state` (the dynamic `CoverageGapsSummary` tri-state classifier) keys on the
    raw `(TransportProto, u16)` port pair with no per-flow protocol identity available; an
    unclassified TCP/102 gap flow is still misattributed to whichever port-102 entry
    matches first by declaration order, regardless of whether the underlying traffic is
    genuinely S7comm-plus, MMS, or ICCP. No catalog-model option — including the `Support`
    enum — can fix this, because the fix requires the analyzer's parsed COTP `protocol_id`
    (SS-20/SS-21), which does not exist at the point `lookup_protocol_state` runs. Gap
    reports on `(Tcp, 102)` still cannot be attributed to a single protocol among the three
    remaining unsupported entries (S7comm no longer contributes to this gap, since it is
    now dissected).
- L2/multicast protocols (e.g., GOOSE, Sampled Values, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK) have no
  TCP/UDP port and are structurally absent from the dynamic gap report. They are listed
  in the catalog with `port_detectable: false`.

## Behavioral Contracts

BC-2.18.001 through BC-2.18.004 (4 BCs; see behavioral-contracts/ss-18/), plus
BC-2.18.005 (the `Support` enum and its exhaustiveness requirement) and BC-2.18.006 (the
four port-102 assignments and the static-fix/dynamic-defer split above), both authored in
feature-s7comm F2 part A (ADR-014 Decision 3, RATIFIED).
