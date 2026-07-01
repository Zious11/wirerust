---
document_type: domain-capability
capability_id: CAP-18
title: "Protocol Coverage Catalog"
subsystem: SS-18
feature: feature-protocol-coverage
adr: ADR-012
introduced: v0.12.0
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
---

# CAP-18: Protocol Coverage Catalog

## Description

wirerust maintains a static, hand-curated compile-time catalog (`KNOWN_PROTOCOLS`) of
approximately 30 ICS/IT protocols (including 5 with `transport=LinkLayer`) that are known
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
- Port 102 hosts a four-way TCP collision: S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2
  all share TCP/102 (ISO-on-TCP / TPKT framing). Gap reports on (Tcp, 102) cannot be
  attributed to a single protocol.
- L2/multicast protocols (GOOSE, Sampled Values, PROFINET-RT/DCP, EtherCAT) have no
  TCP/UDP port and are structurally absent from the dynamic gap report. They are listed
  in the catalog with `port_detectable: false`.

## Behavioral Contracts

BC-2.18.001 through BC-2.18.004 (4 BCs; see behavioral-contracts/ss-18/).
