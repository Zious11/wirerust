---
document_type: adr
adr_id: ADR-012
status: accepted
accepted_date: "2026-07-01"
date: 2026-07-01
modified: []
subsystems_affected:
  - SS-18
  - SS-05
  - SS-12
supersedes: null
superseded_by: null
feature_cycle: feature-protocol-coverage
issue: "D-320"
---

# ADR-012: Protocol Coverage Catalog — Design Decisions

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md`.
> ADR IDs are sequential 3-digit (ADR-001, ADR-002, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.

## Context

The `feature-protocol-coverage` cycle introduces two capabilities:

1. A **static catalog** (`KNOWN_PROTOCOLS`) of ~30 ICS and IT protocols that wirerust
   knows about, plus pure functions to report which are supported versus unsupported.
2. A **dynamic gap detector** that surfaces TCP flows routed to `DispatchTarget::None`
   by transport and port, exposed as `CoverageGapsSummary` in the analysis output.

These capabilities required a set of non-obvious design choices that this ADR records.
Approved scope from human gate D-320 (non-negotiable): OQ-1 through OQ-5 resolved.

---

## Decision 1: Hand-Curated Static Compile-Time Array

**Decision:** `KNOWN_PROTOCOLS` is a `const &[KnownProtocol]` in `src/protocols.rs`
(static compile-time array — F1 delta-analysis Option A). External data files (JSON,
TOML) are rejected.

**Rationale:**

- **Zero I/O.** A CLI forensics tool that reads a local catalog file at startup
  introduces an I/O dependency where none existed. `KNOWN_PROTOCOLS` is a pure-core
  constant; no file path, no error handling, no startup latency.
- **Formally verifiable.** A static array is the correct target for proptest (VP-041)
  and Kani. Runtime file parsing is effectful and cannot be model-checked.
- **Precedent.** `src/mitre.rs` uses the same pattern (`SEEDED_TECHNIQUE_IDS` static
  array). Consistency lowers the learning cost for contributors.
- **No auto-sourcing from IANA.** The IANA port registry has ~14,000 entries and
  lacks L2 protocols entirely. A filtered curated set is the correct tool; IANA is
  used only to verify port numbers of entries that have registrations.

**Rejected Alternative:** External TOML file. Rejected because: (a) it turns a
pure-core lookup into an effectful I/O operation; (b) distribution complexity
(the catalog must be present at the binary's runtime location); (c) a hand-curated
set of ~30 entries does not benefit from external editability.

---

## Decision 2: Tri-State Vocabulary (Suricata-Derived)

**Decision:** The `CoverageGapsSummary` report classifies traffic using three states
borrowed from Suricata's app-layer detection taxonomy [T6]:

| State | Meaning | Analogy |
|-------|---------|---------|
| `known-unsupported` | Port maps to a catalog entry; wirerust has no dissector | Suricata `failed` |
| `unknown` | Port maps to no catalog entry | Suricata `unknown` |
| `known-supported` | Port maps to a catalog entry with a live dissector | (sanity check only — should never appear in gap report) |

**Rationale:** Suricata is the leading open-source network security monitor and the
only tool in the ecosystem that exposes these states as first-class rule keywords [T6].
Using the same vocabulary reduces operator cognitive overhead and provides a clear
prior-art basis. The tri-state is the minimum discriminator that allows operators to
distinguish a dissector *bug* (supported protocol not classified) from a *gap*
(known-but-unsupported) from an *anomaly* (completely unknown port).

**Note:** The Suricata `unknown` state refers to a *flow that has not yet been
classified* (in-progress). wirerust uses it for *flows that closed without a catalog
match*, which is semantically different but pragmatically the best fit.

---

## Decision 3: Port-Based Detection Caveats (Mandatory Documentation)

**Decision:** The following caveats MUST appear in the `CoverageGapsSummary` output
header and in the `protocols --help` text. They are not optional warnings.

**3a. TCP-only dynamic detection:**
> Dynamic gap detection covers TCP flows only. UDP-based protocols (BACnet/IP on
> 47808, SNMP, NTP, PROFINET RPC, DNS on 53) and Layer-2 protocols (GOOSE,
> Sampled Values, PROFINET-RT/DCP, EtherCAT) are never reported here. Consult
> `wirerust protocols --unsupported` for the full known-protocol set.

**3b. Port 102 collision:**
Four distinct ICS protocols share TCP port 102: **S7comm, S7comm-plus, IEC 61850 MMS,
and ICCP/TASE.2**. All use ISO-on-TCP (RFC 1006) framing. A gap on port 102
cannot be attributed to a single protocol. The `CoverageGapsSummary` MUST note this
ambiguity inline rather than attributing port 102 gaps to any one protocol.

Research basis: Wireshark S7comm wiki [P2], scadaprotocols.com IEC 61850 MMS [P5],
PacketViper ICCP/TASE.2 [P1] (see feature-protocol-coverage-research.md references).

**3c. L2/multicast protocols have no port:**
GOOSE (0x88B8), Sampled Values (0x88BA), PROFINET-RT/DCP (0x8892), and EtherCAT
(0x88A4) are EtherType-identified Layer-2 multicast frames. The dynamic gap detector
operates on `(transport, port)` pairs extracted by the TCP dispatcher. These protocols
are structurally invisible to it. The catalog lists them with `port_detectable: false`
and the `protocols --unsupported` output marks them with a `[L2]` transport indicator.

**3d. Port heuristics are not ground truth:**
Port-based protocol identification is a heuristic [C1]. Services frequently run on
non-standard ports; the canonical port may host unrelated traffic or a tunnel. The
`CoverageGapsSummary` states only that an unclassified flow was observed on a given
port — it does not assert which protocol was present.

---

## Decision 4: Catalog Scope — ICS + Core-IT

**Decision:** The catalog includes ICS protocols (both port-detectable and L2) and a
curated set of IT protocols relevant to OT environments. ICS-only is rejected; all-IANA
is rejected.

**Rationale:**
- **ICS-only is too narrow.** Purdue Level 2–3 (HMI, historian, engineering
  workstation) runs heavy IT traffic. Unexpected RDP, SMB, or SSH in a control zone
  is itself a security signal. OT platforms (Dragos, Nozomi) both catalog IT protocols
  for exactly this reason [T9][T10].
- **All-IANA is noise.** 14,000 IANA entries make the unsupported list useless and
  unmaintainable. The catalog is a curated signal for operators, not a port dump.
- **`decoder.rs` already hints SSH (22) and SMB (445)** via `app_protocol_hint`.
  Cataloging them is consistent and low-cost.

**Scope (D-320 approved):** 7 supported + 9 ICS Tier-1 unsupported + 5 L2-flagged +
9 IT core unsupported = ~30 entries. See `ss-18-protocol-coverage-catalog.md` for the
complete table.

---

## Decision 5: Supported-Set Derivation — Static `SUPPORTED_PORTS` Mirror

**Decision:** `supported_protocols()` derives the supported set by intersecting
`KNOWN_PROTOCOLS` with a compile-time `SUPPORTED_PORTS: &[u16]` constant that mirrors
the port-fallback rules in `dispatcher.rs::classify()`. It does NOT use runtime
introspection of the dispatcher.

**Rationale:**
- `dispatcher.rs::classify()` is an effectful method (it has state). Runtime
  introspection would break the pure-core boundary of `protocols.rs`.
- The set of supported ports is stable and known statically (502, 20000, 44818, 443,
  8443, 80, 8080, 53, plus L2 ARP). A compile-time constant is sufficient.
- VP-041 (proptest) can verify the set-difference property without runtime dependency.

**Drift risk:** If a new analyzer is integrated (new `DispatchTarget` variant and port
rule in `classify()`), the implementer MUST update `SUPPORTED_PORTS` in `protocols.rs`.
This is a documented convention, not a compile-time enforcement. To make the obligation
visible, the `SUPPORTED_PORTS` constant carries a doc-comment listing each port and
the corresponding `DispatchTarget` variant it mirrors.

**ARP handling:** ARP is supported via `DecodedFrame::Arp` outside the dispatcher.
The ARP entry in `KNOWN_PROTOCOLS` carries `canonical_ports: &[]` and is flagged
`supported: true` via a special case in `supported_protocols()` (or equivalently via
an ARP-specific constant).

---

## Decision 6: TCP+UDP Scope for Dynamic Detection

**Decision:** The `CoverageGapsSummary` feature covers **TCP flows only** in this
cycle. The catalog is transport-aware (records UDP protocols correctly). UDP gap
detection is explicitly deferred to the next cycle with a planned implementation path.

**Rationale:**
- `StreamDispatcher` handles TCP flows only. Extending dynamic detection to UDP this
  cycle would require new UDP-flow plumbing in `main.rs`, outside the VP-004 Kani zone.
- BACnet/IP (UDP/47808) is a Tier-1 catalog protocol and the most significant UDP gap.
  Deferring it requires the mandatory caveat in Decision 3a above.
- UDP gap tracking (lightweight per-(udp, port) counter in the decode loop) is the
  natural completion; it is filed as the immediate follow-on with high priority.

**Catalog design consequence:** `canonical_ports` entries include UDP protocol entries
(BACnet/IP, SNMP, NTP, PROFINET RPC) with the correct `transport: Transport::Udp`
annotation. This ensures `protocols --unsupported` gives accurate information even
while the dynamic detector is TCP-only.

---

## Decision 7: Category Tagging

**Decision:** Every `KnownProtocol` entry is tagged with one of three categories:

| Category | Use |
|----------|-----|
| `ICS` | Industrial/OT protocols (Modbus, DNP3, S7comm, GOOSE, etc.) |
| `IT` | General IT/enterprise protocols relevant in OT (SSH, RDP, SMB, etc.) |
| `L2` | Layer-2-only protocols without a TCP/UDP port (GOOSE, SV, EtherCAT, etc.) |

Note: `L2` is a transport/detection characteristic, not a mutually exclusive security
category (GOOSE is also `ICS`). In the Rust implementation, `category` captures the
security/domain classification (`ICS` vs. `IT`), while `transport: Transport::LinkLayer`
and `port_detectable: false` capture the detection limitation. Both are required for
the `protocols` subcommand to support filtering (e.g. `--ics-only`).

---

## Decision 8: `--coverage-gaps` Explicit Flag

**Decision:** Dynamic gap detection is gated behind an explicit `--coverage-gaps` flag
on the `analyze` subcommand. It does NOT auto-enable under `analyze --all`.

**Rationale:** `analyze --all` activates every analysis pass (OQ-4 resolution from D-320).
Gap detection adds a new `HashMap` field to `StreamDispatcher` and a report section to
the output. Making it opt-in ensures existing `--all` consumers see unchanged output,
preventing silent behavioral drift for downstream tooling parsing wirerust JSON output.

**Rule:** LESSON-P1.04 ("no unwired flags") requires that any accepted flag must be
fully wired to observable behavior. `--coverage-gaps` is fully wired: when set,
`unclassified_port_counts` is populated and `CoverageGapsSummary` is appended to the
analysis output.

---

## Decision 9: `CoverageGapsSummary` as New Report Section

**Decision:** Dynamic gap results surface as a new **`CoverageGapsSummary`** named
section in the analysis output, NOT as individual `Finding` entries.

**Rationale (OQ-2 resolution from D-320):**
- Finding entries are security events with severity and MITRE annotations. An
  undissected port count does not have a severity or MITRE technique — it is a
  coverage metric, not a threat signal.
- Adding Finding entries for undissected ports would inflate finding counts and
  confuse downstream MITRE-grouping logic.
- A named summary section (analogous to the existing `reassembly_summary` entry in
  `AnalysisSummary`) is the lowest-risk, most semantically correct surface.
  BC-2.12.015 is amended to inject `unclassified_port_counts()` into this new section.

---

## Consequences

- `src/protocols.rs` is a new pure-core module (C-26, SS-18). Zero external crates added.
- `src/dispatcher.rs` gains `unclassified_port_counts: HashMap<(u16, u16), u64>` (SS-05).
  The HashMap is direction-normalized via the existing flow-key port normalization.
- `src/cli.rs` gains `Protocols { supported, unsupported, all }` variant (SS-12).
- `src/main.rs` gains `run_protocols()` and a `Commands::Protocols` arm (SS-12).
- `--coverage-gaps` flag added to `analyze` subcommand; absent by default (Decision 8).
- VP-041 guards catalog set-difference correctness (proptest, P1, SS-18).
- VP-042 guards dispatcher port-count accumulation (proptest, P1, SS-05).
- VP-004 (Kani, dispatcher `classify()`) is NOT affected — the `classify()` function
  and `DispatchTarget` enum are NOT changed. VP-004 must be re-validated at F6 to
  confirm no regression from the new HashMap field.

---

## References

See `.factory/phase-f1-delta-analysis/feature-protocol-coverage-research.md` for
full source citations (P1..P16 protocol sources; T1..T13 tool coverage models;
C1..C17 port-detection caveats).

Key references for this ADR:
- [P1] PacketViper SCADA/ICS port list — port 102 multi-protocol entry
- [P2] Wireshark Wiki S7comm — port 102 confirmation
- [P4] Chipkin BACnet — IANA 47808/UDP confirmation
- [P5] scadaprotocols.com IEC 61850 MMS — port 102
- [C1] Keysight — 3 problems with port-based identification
- [T6] Suricata docs — app-layer `unknown`/`failed` keywords
- [T9] Dragos — "600+ industrial and IT protocols"
- [T12] IANA — Service Name and Port Number Registry
