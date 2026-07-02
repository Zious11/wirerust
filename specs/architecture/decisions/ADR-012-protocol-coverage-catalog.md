---
document_type: adr
adr_id: ADR-012
status: accepted
accepted_date: "2026-07-01"
date: 2026-07-01
modified:
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-9 remediation (F-F2P9-003): Decision 6 Clarification added — unclassified_port_counts increment is placed INSIDE the same analyzer-present guard as unclassified_flows += 1; both counters increment together when ≥1 analyzer configured AND coverage_gaps_enabled=true; VP-042(d) precondition documented; BC-2.05.010 Architecture Anchor wording specified."
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-2 remediation: (F-F2P2-004) Decision 3a and 3c caveat text updated to include Ethernet POWERLINK (0x88AB) as the 5th L2 protocol — catalog has 5 port_detectable:false entries; prior text named only 4. Decision 6 remaining-caveats list also updated. Wording changed to 'e.g.,' prefix for forward-compatibility. (F-F2P2-005) Decision 10 added — UDP gap classification decoupled from enable_dns: when --coverage-gaps is active, dns_analyzer.can_decode() is evaluated regardless of enable_dns for gap-accounting; finding-emission retains the enable_dns gate; rejects catalog-supported-port exclusion alternative. (Observation) Consequences clap-shape updated to Commands::Protocols { filter: ProtocolFilter, json: bool } with ProtocolFilter { All, Supported, Unsupported } enum."
  - date: "2026-07-01"
    actor: architect
    reason: "F2-SCOPE-DRIFT-UDP-001 resolution: corrected Decision 6 from TCP-only to TCP+UDP dynamic detection per approved scope D-320 OQ-5. Updated Decision 3a (TCP-only caveat updated to L2/multicast-only caveat; UDP-based protocols BACnet/IP, SNMP, NTP are now detectable). Updated Consequences section: HashMap key type changed from (u16, u16) direction-normalized port pair to (TransportProto, u16) to support transport-discriminated keying. BACnet/IP UDP/47808 is now flaggable in the dynamic gap report."
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-5 remediation (F-F2P5-001): Decision 5 reframed — 'SUPPORTED_PORTS mirrors classify()' was false because port 53 (DNS) has no classify() rule and no DispatchTarget variant; DNS is handled in main.rs decode loop like ARP. Decision 5 now states SUPPORTED_PORTS = classify() TCP port-fallback rules PLUS decode-loop DNS path (port 53). Doc-comment obligation reframed to permit port → dissection path (classify() variant OR decode-loop)."
  - date: "2026-07-01"
    actor: architect
    reason: "F2 adversarial Pass-1 remediation: (F-F2P1-003) Decision 7 rewritten — removed `L2` category variant; ProtocolCategory is now {ICS, IT} only; L2 detection-class expressed solely by transport:LinkLayer + port_detectable:false; GOOSE.category=ICS; (F-F2P1-010) Decision 7 note updated — no category-based CLI filter ships this cycle per D-320 scope; (F-F2P1-006) Decision 6 UDP key changed from (Udp, dst_port) to (Udp, min(src_port, dst_port)) — ephemeral-port guard symmetric with TCP lower_port convention; Consequences section updated to match."
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
2. A **dynamic gap detector** that surfaces TCP and UDP flows not handled by a dissector,
   keyed by `(transport, port)` and exposed as `CoverageGapsSummary` in the analysis output.

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

**3a. Transport scope — L2/multicast protocols are structurally absent:**
> Dynamic gap detection covers TCP and UDP flows. Layer-2 protocols (e.g., GOOSE,
> Sampled Values, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK) have no TCP/UDP
> port and are never reported in the dynamic gap report regardless of TCP+UDP scope.
> Consult `wirerust protocols --unsupported` for L2 protocol coverage.

**3b. Port 102 collision:**
Four distinct ICS protocols share TCP port 102: **S7comm, S7comm-plus, IEC 61850 MMS,
and ICCP/TASE.2**. All use ISO-on-TCP (RFC 1006) framing. A gap on port 102
cannot be attributed to a single protocol. The `CoverageGapsSummary` MUST note this
ambiguity inline rather than attributing port 102 gaps to any one protocol.

Research basis: Wireshark S7comm wiki [P2], scadaprotocols.com IEC 61850 MMS [P5],
PacketViper ICCP/TASE.2 [P1] (see feature-protocol-coverage-research.md references).

**3c. L2/multicast protocols have no port:**
GOOSE (0x88B8), Sampled Values (0x88BA), PROFINET-RT/DCP (0x8892), EtherCAT
(0x88A4), and Ethernet POWERLINK (0x88AB) are EtherType-identified Layer-2 frames.
The dynamic gap detector operates on `(transport, port)` pairs extracted by the TCP
dispatcher. These protocols are structurally invisible to it. The catalog lists them
with `port_detectable: false` and the `protocols --unsupported` output marks them
with a `[L2]` transport indicator.

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

## Decision 5: Supported-Set Derivation — Static `SUPPORTED_PORTS`

**Decision:** `supported_protocols()` derives the supported set by intersecting
`KNOWN_PROTOCOLS` with a compile-time `SUPPORTED_PORTS: &[u16]` constant equal to
the full set of ports wirerust actively dissects: the TCP port-fallback rules in
`dispatcher.rs::classify()` PLUS the decode-loop DNS path (port 53, handled in
`main.rs` like ARP — outside `classify()`, with no `DispatchTarget` variant). It does
NOT use runtime introspection of the dispatcher.

**Semantics clarification:** `SUPPORTED_PORTS` is NOT a pure mirror of
`classify()` port-fallback rules. It equals `classify()` TCP port-fallback rules
∪ {53} (DNS decode-loop). DNS/53 and ARP are dissected outside `classify()` by
design; they are permanently and intentionally non-mirroring with respect to
`classify()`. This is NOT drift — it is the correct invariant: SUPPORTED_PORTS =
the set of ports wirerust actively dissects by any mechanism.

**Rationale:**
- `dispatcher.rs::classify()` is an effectful method (it has state). Runtime
  introspection would break the pure-core boundary of `protocols.rs`.
- The set of supported ports is stable and known statically (502, 20000, 44818, 443,
  8443, 80, 8080, 53, plus L2 ARP). A compile-time constant is sufficient.
- VP-041 (proptest) can verify the set-difference property without runtime dependency.

**Drift risk:** If a new analyzer is integrated (new `DispatchTarget` variant and port
rule in `classify()`, OR a new decode-loop dissector path), the implementer MUST update
`SUPPORTED_PORTS` in `protocols.rs`. This is a documented convention, not a
compile-time enforcement. To make the obligation visible, the `SUPPORTED_PORTS`
constant carries a doc-comment listing each port and its dissection path: either a
`DispatchTarget` variant (for TCP ports handled by `classify()`) or "decode-loop"
(for ports handled outside `classify()`, e.g., port 53 → DNS decode-loop,
no `DispatchTarget` variant; port-independent ARP → `DecodedFrame::Arp`).

**ARP handling:** ARP is supported via `DecodedFrame::Arp` outside the dispatcher.
The ARP entry in `KNOWN_PROTOCOLS` carries `canonical_ports: &[]` and is flagged
`supported: true` via a special case in `supported_protocols()` (or equivalently via
an ARP-specific constant).

---

## Decision 6: TCP+UDP Dynamic Detection (D-320 OQ-5 Approved Scope)

**Decision:** The `CoverageGapsSummary` feature covers **both TCP and UDP flows** in
this cycle (approved scope D-320 OQ-5). The `unclassified_port_counts` structure uses
`(TransportProto, u16)` as its key — a 2-tuple of transport protocol (`Tcp` or `Udp`)
and canonical port number — enabling TCP and UDP unclassified traffic to be counted
distinctly even when they share the same port number.

**Implementation split:**

- **TCP unclassified flows** — tracked by `StreamDispatcher.unclassified_port_counts:
  HashMap<(TransportProto, u16), u64>` at `on_flow_close` for `DispatchTarget::None`
  flows. The dispatcher handles TCP only; all entries carry `TransportProto::Tcp`.
  Canonical port: lower-numbered port of the direction-normalized flow key (approximates
  the server/service port; aligns with the existing `(lower, upper)` flow-key convention
  and the research caveat against treating ephemeral high ports as protocols).
- **UDP unclassified packets** — tracked by a lightweight counter in the decode loop in
  `main.rs` (outside the TCP-only dispatcher). Key:
  `(TransportProto::Udp, min(src_port, dst_port))` where `min(src_port, dst_port)` is
  the lower-numbered of the two endpoint ports, approximating the service/server port
  (symmetric with the TCP implementation's `lower_port` convention). UDP is
  connectionless; accumulation is per-packet, not per-flow-close.

  **Ephemeral-port guard rationale:** Using `min(src_port, dst_port)` prevents
  ephemeral-port noise. A SNMP response (src: 161 → dst: ephemeral 52000) is correctly
  keyed on `(Udp, 161)`, not `(Udp, 52000)`. Ephemeral ports (≥ 49152) carry no service
  meaning [C11] and must not generate catalog entries (research Q4§1).

**CoverageGapsSummary merge:** The reporter reads both counters; both use the same
`(TransportProto, u16)` key type. The unified view enables the Suricata tri-state
(known-unsupported / unknown) to classify, for example, UDP/47808 as `known-unsupported`
(BACnet/IP catalog match) and TCP/47808 as `unknown` (no catalog entry) independently.

**Rationale:**
- BACnet/IP (UDP/47808) is a Tier-1 catalog ICS protocol. D-320 OQ-5 explicitly
  approved TCP+UDP so that BACnet gaps are flaggable in the dynamic gap report.
- `(TransportProto, u16)` keying prevents false conflation: a BACnet gap on UDP/47808
  is not merged with hypothetical TCP/47808 traffic.
- UDP counter placement in the decode loop (outside the dispatcher) avoids any change
  to the VP-004 Kani-verified `classify()` function or `DispatchTarget` enum.

**Remaining port caveats:**
- L2/multicast protocols (GOOSE, SV, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK)
  have no TCP/UDP port and remain structurally absent from the dynamic gap report
  (Decision 3a, 3c).
- Port-102 four-way TCP collision (S7comm / S7comm-plus / IEC 61850 MMS / ICCP-TASE.2)
  still applies to TCP entries keyed on `(Tcp, 102)` (Decision 3b).

**`TransportProto` type note:** `dispatcher.rs` MUST NOT import from `protocols.rs`
(pure-core boundary, per Decision 5 and SS-18 §Subsystem Boundaries). `TransportProto`
is therefore a minimal enum defined independently in `dispatcher.rs` (or a shared
low-level module), containing only `Tcp` and `Udp` variants. It is NOT the `Transport`
enum from `protocols.rs` (which has a third `LinkLayer` variant unsuitable for the
dispatcher context).

**Decision 6 Clarification — Increment-Site Semantics (F-F2P9-003):**

The `unclassified_port_counts` HashMap increment MUST be placed **INSIDE** the same
analyzer-present guard as the existing `unclassified_flows += 1` statement. Both
increments are co-located within the guard:

```
Some(DispatchTarget::None) | None => {
    if self.http.is_some() || self.tls.is_some() || self.modbus.is_some()
        || self.dnp3.is_some() || self.enip.is_some()
    {
        self.unclassified_flows += 1;
        if coverage_gaps_enabled {
            *self.unclassified_port_counts.entry((TransportProto::Tcp, lower_port))
                .or_insert(0) += 1;
        }
    }
}
```

**Rationale:** `--coverage-gaps` answers "which ports had TCP flows that no configured
analyzer classified?" When zero analyzers are configured, all flows are trivially
`DispatchTarget::None` regardless of port — per-port counting would be noise, not a
coverage gap signal. Placing the increment inside the guard preserves semantic symmetry:
both counters count the same population (None-target flows where analyzers were present
but could not classify the traffic).

**VP-042(d) precondition:** Sub-property (d) ("both counters consistent") holds only
given the harness precondition: ≥1 analyzer configured (`is_some()`) AND
`coverage_gaps_enabled=true`. The proptest harness MUST construct the dispatcher with at
least one analyzer enabled and `coverage_gaps_enabled=true` to make sub-property (d)
reachable.

**BC-2.05.010 Architecture Anchor wording:** PC-1 MUST state that the
`unclassified_port_counts` increment is gated on both (a) `coverage_gaps_enabled=true`
and (b) the analyzer-present guard. When no analyzers are configured, neither counter
fires. The VP-042 proptest harness precondition is ≥1 analyzer `is_some()` AND
`coverage_gaps_enabled=true`.

---

## Decision 7: Category Tagging

**Decision:** Every `KnownProtocol` entry is tagged with one of two categories:

| Category | Use |
|----------|-----|
| `ICS` | Industrial/OT protocols (Modbus, DNP3, S7comm, GOOSE, BACnet, etc.) |
| `IT` | General IT/enterprise protocols relevant in OT environments (SSH, RDP, SMB, etc.) |

`ProtocolCategory` is `{ ICS, IT }` only. There is NO `L2` category variant.
Layer-2 detection characteristics — protocols identified by EtherType rather than port
(GOOSE, Sampled Values, PROFINET-RT/DCP, EtherCAT) — are expressed exclusively by
`transport: Transport::LinkLayer` and `port_detectable: false`, NOT by a separate
`L2` category. L2 protocols such as GOOSE carry `category: ICS`.

Category is retained for display and possible future filtering. No category-based CLI
flag (e.g. `--ics-only`) ships this cycle; D-320 approved scope is OQ-1..OQ-5 only,
and BC-2.12.022 restricts filters to `--all`, `--supported`, and `--unsupported`.

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

## Decision 10: UDP Gap Classification Decoupled from `enable_dns`

**Decision:** When `--coverage-gaps` is active, `dns_analyzer.can_decode()` is
evaluated for UDP gap-accounting purposes **regardless of whether `enable_dns` is
true**. Gap classification is decoupled from finding-emission. A UDP packet on
port 53 for which `dns_analyzer.can_decode()` returns true is **NOT** counted in
the unclassified UDP gap counter, even when `--all` / `--dns` is disabled.

**Rationale:**

- `--coverage-gaps` and `--all` / `enable_dns` are orthogonal flags (D-320 OQ-4).
  Gap reporting expresses "what traffic was unclassifiable by available dissectors,"
  not "what traffic was examined for findings."
- Without this decoupling: if `enable_dns == false`, DNS/53 UDP packets are never
  offered to `can_decode()`, so they enter the unclassified counter as `(Udp, 53)`.
  The tri-state lookup (Decision 2) then classifies `(Udp, 53)` as `known-supported`
  — a false dissector-bug signal. The `known-supported` state is reserved for
  detecting actual dissector bugs (a supported protocol whose live dissector failed
  to classify the traffic), not for "analyzer disabled."
- Correct semantics: a UDP packet is **unclassified for gap purposes** iff no
  available dissector `can_decode()` it, independent of which analyzers are enabled
  for finding-emission.

**Implementation pattern:**

```rust
// Evaluate can_decode unconditionally for gap classification.
// Finding-emission retains the enable_dns gate.
let classified_by_dns = dns_analyzer.can_decode(&parsed);
if enable_dns && classified_by_dns {
    all_findings.extend(dns_analyzer.analyze(&parsed));
}
if coverage_gaps && parsed.is_udp() && !classified_by_dns {
    // increment UDP gap counter keyed on (Udp, min(src_port, dst_port))
}
```

**Rejected alternative:** Exclude catalog-`supported` UDP ports from the unclassified
counter entirely. Rejected because this eliminates the `known-supported` dissector-bug
detection signal for all UDP-supported protocols — not just DNS — defeating
Decision 2's sanity-check purpose.

**VP-043 note:** VP-043's property ("NOT classified by `dns_analyzer.can_decode()`")
remains correct — `can_decode()` is evaluated unconditionally for gap purposes, and
the gate invariant (classified UDP does NOT increment) is unambiguous. Product-owner
must encode in BC-2.05.010: when `--coverage-gaps` is active, `can_decode()` is
evaluated regardless of `enable_dns`; a packet for which `can_decode()` returns true
MUST NOT increment the gap counter.

---

## Consequences

- `src/protocols.rs` is a new pure-core module (C-26, SS-18). Zero external crates added.
- `src/dispatcher.rs` gains `unclassified_port_counts: HashMap<(TransportProto, u16), u64>`
  (SS-05) for TCP None-target flow tracking; key is `(Tcp, lower_port)` where
  `lower_port` is the lower-numbered port of the direction-normalized flow key.
- `src/main.rs` decode loop gains a UDP unclassified counter (same key type
  `HashMap<(TransportProto, u16), u64>`) for UDP packets not routed to any dissector;
  key is `(Udp, min(src_port, dst_port))` — lower-numbered port (ephemeral-port guard,
  symmetric with TCP `lower_port` convention; Decision 6 rationale above).
- `TransportProto` is a minimal `{Tcp, Udp}` enum in `dispatcher.rs`, independent of
  `protocols.rs::Transport` (which has a third `LinkLayer` variant and must not be
  imported into the dispatcher per the pure-core boundary rule).
- `src/cli.rs` gains `Commands::Protocols { filter: ProtocolFilter, json: bool }` variant
  (SS-12), where `ProtocolFilter` is `{ All, Supported, Unsupported }` — an enum that
  enforces mutual exclusion at the type level (or equivalent).
- `src/main.rs` gains `run_protocols()` and a `Commands::Protocols` arm (SS-12).
- `--coverage-gaps` flag added to `analyze` subcommand; absent by default (Decision 8).
- When `--coverage-gaps` is active, `dns_analyzer.can_decode()` is evaluated for UDP
  gap classification regardless of `enable_dns` — gap-accounting is decoupled from
  finding-emission (Decision 10). Finding emission retains the `enable_dns` gate.
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
