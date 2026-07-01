---
document_type: feature-research
feature_id: feature-protocol-coverage
cycle: feature-protocol-coverage
title: "Protocol Coverage Catalog + Dynamic Gap Detection — Research"
producer: research-agent
created: 2026-07-01
base_commit: 3a60317
branch: develop
status: complete
feeds:
  - ADR-012 (Protocol Coverage Catalog design)
  - OQ-1 (catalog scope: ICS-only vs ICS+IT)
  - OQ-5 (TCP-only vs TCP+UDP dynamic detection)
  - src/protocols.rs KNOWN_PROTOCOLS catalog contents
---

# Research — Protocol Coverage Catalog & Dynamic Gap Detection

Supports the `feature-protocol-coverage` cycle: (a) a curated "known protocols"
catalog so wirerust can report what it does NOT dissect, and (b) dynamic
detection of undissected traffic by transport+port during `analyze`.

**wirerust currently dissects (the "supported" set):** Modbus/TCP, DNP3,
EtherNet/IP + CIP, TLS handshake, ARP, DNS, HTTP; reads pcap + pcapng.
(Per `feature-protocol-coverage-delta-analysis.md` §6 regression baseline and
the decoder's `app_protocol_hint` map.)

> **Sourcing note.** Every non-obvious port number and transport claim below is
> cited to an authoritative or vendor source. Where a source is a community
> reference (PacketViper, scadaprotocols.com, whatportis) rather than IANA or a
> primary standard, that is flagged. Model-knowledge-only claims are explicitly
> marked **[unverified]**. Findings are as of **2026-07-01**; port registries
> and vendor defaults drift.

---

## Q1 — ICS/OT Protocol Universe

Enumeration of significant industrial/OT protocols for the coverage catalog,
prioritized by real-world prevalence. Transport and default port(s) verified
against IANA where a registration exists; otherwise against vendor docs or
Wireshark, with the source flagged.

### Tier 1 — High prevalence, port-detectable, NOT yet in wirerust

| Protocol | Transport | Default port(s) | Purpose | Port source |
|----------|-----------|-----------------|---------|-------------|
| **S7comm** | TCP (ISO-on-TCP / TPKT, RFC 1006) | **102/TCP** | Siemens S7-300/400 PLC programming, data exchange, diagnostics | Wireshark S7comm wiki [P2] |
| **S7comm-plus** | TCP (ISO-on-TCP) | **102/TCP** (same as S7comm) | Successor for S7-1200/1500; app-layer auth/integrity. Shares port 102 → payload disambiguation required | Litmus/Siemens [P3]; **[unverified]** exact port not in a primary standard |
| **IEC 60870-5-104 (IEC-104)** | TCP (UDP allowed, rare) | **2404/TCP** (IANA `iec-104`, TCP+UDP) | Telecontrol / SCADA between control centers and substations/RTUs (power) | IANA [P8]; whatportis [C9] |
| **IEC 61850 MMS** | TCP (ISO-on-TCP) | **102/TCP** (standardized, mandatory, IEC 61850-8-1) | Substation client/server config, control, reporting. Shares port 102 with S7comm + ICCP/TASE.2 | scadaprotocols [P5] |
| **BACnet/IP** | **UDP** (TCP variant exists) | **47808/UDP** (0xBAC0; IANA `bacnet`, TCP+UDP) | Building automation: HVAC, lighting, access control, fire | IANA [C8]; Chipkin [P4] |
| **OPC-UA (binary)** | TCP | **4840/TCP** (IANA `opcua-tcp`) | Platform-independent industrial data access, alarms, historian | IANA [P8] |
| **OPC-UA (HTTP/SOAP)** | TCP (HTTP/HTTPS) | 80/443 (indistinguishable from web at port layer) | OPC-UA XML/SOAP mapping over web ports | PacketViper [P1] |
| **PROFINET (RPC/config)** | UDP | 34962 (RT unicast), 34963 (RT multicast), 34964 (RPC CM), 53247 (RPC) | Discovery/RPC/config for PROFINET IO | HMS Networks [P14] |
| **ICCP / TASE.2** | TCP (ISO-on-TCP) | **102/TCP** (shared with S7comm/MMS) | Inter-control-center data exchange (power) | PacketViper [P1] |

### Tier 2 — Real but vendor-specific / configurable ports

| Protocol | Transport | Default port(s) | Purpose | Port source |
|----------|-----------|-----------------|---------|-------------|
| **HART-IP** | TCP + UDP | **5094** (both; UDP initiates session, may migrate) | IP extension of HART smart-instrument protocol | Wireshark HART-IP [P10] |
| **OMRON FINS** | TCP (also UDP) | **9600/TCP** (configurable) | Omron PLC comms over Ethernet/serial | EMQX [P11]; PacketViper [P1] |
| **GE-SRTP** | TCP | **18245/TCP** (configurable) | GE (Fanuc) PLC memory read/write, program up/download | AutomationDirect C-more [P12] |
| **Foundation Fieldbus HSE** | TCP + UDP | **1089–1091** (both) | High-Speed-Ethernet variant of FF for process plants | PacketViper [P1]; connected.app [P13] |
| **CODESYS** | TCP | **Vendor-specific** (no single default; commonly 1217/2455 **[unverified]**) | Runtime/programming for 250+ OEM PLCs | PacketViper flags "vendor application specific" [P1] |
| **MELSEC / MC Protocol** | TCP + UDP | **Vendor-specific** (not in IANA; often 5000-range **[unverified]**) | Mitsubishi PLC programming/IO | PacketViper (no port given) [P1] |
| **PCCC** | TCP (encapsulated in CIP) | 44818/TCP (no unique port — rides EtherNet/IP) | Legacy Allen-Bradley command set inside CIP | PacketViper [P1] |
| **CIP Safety** | TCP + UDP (same as base CIP) | 44818/TCP, 2222/UDP | Safety-certified CIP extension; **no separate port** | PacketViper [P1] |
| **ROC Plus** | TCP + UDP | 4000 | Emerson RTU SCADA | PacketViper [P1] |
| **Niagara Fox (Tridium)** | TCP | 1911, 4911 | Building-automation integration framework | PacketViper [P1] |
| **PCWorx (Phoenix Contact)** | TCP | 1962, 20547, 2455 | Phoenix Contact PLC programming | PacketViper [P1] |
| **Red Lion Crimson** | TCP | 789 | Red Lion HMI configuration | PacketViper [P1] |

### Tier 3 — Layer-2 / multicast: a PORT-BASED DETECTOR CANNOT SEE THESE

These are the critical flag for OQ-5 / spec caveats. They have **no TCP/UDP
port**; they are identified by EtherType and (usually) multicast MAC. wirerust
already handles one L2 protocol (ARP) outside the dispatcher, so the pattern
exists.

| Protocol | Layer | EtherType | Purpose | Source |
|----------|-------|-----------|---------|--------|
| **IEC 61850 GOOSE** | L2 multicast (802.1Q VLAN/priority) | **0x88B8** | Sub-4ms protection/interlock trip messaging between IEDs | iGrid [P6]; scadaprotocols [C12] |
| **IEC 61850 Sampled Values (SV)** | L2 multicast | **0x88BA** | High-rate digitized CT/VT current/voltage streams (process bus) | scadaprotocols [P7][C12] |
| **PROFINET RT / DCP / cyclic IO** | L2 (some UDP for config) | **0x8892** (PN-DCP/IO); MRP **0x88E3** | Real-time cyclic IO + device discovery/config. Core IO is L2-only | HMS Networks [P14]; PROFINET Univ [C10] |
| **EtherCAT** | L2 (UDP tunnel optional 34980) | **0x88A4** | Deterministic motion/IO; frames processed on-the-fly | EtherCAT.org [P16]; Wireshark [C13] |
| **Ethernet POWERLINK** | L2 | dedicated EtherType (**[unverified]** — 0x88AB commonly cited) | Time-triggered deterministic Industrial Ethernet | model knowledge; not in cited primary sources |
| **CC-Link IE (Field/Control)** | L2 | vendor-specific | High-speed Mitsubishi Industrial Ethernet | **[inconclusive]** — no port/EtherType in cited sources |
| **CC-Link (original)** | RS-485 serial | n/a | Non-IP fieldbus; invisible to any packet capture | model knowledge |

**Key finding for the spec:** GOOSE, SV, PROFINET-RT/DCP, EtherCAT, POWERLINK
and CC-Link IE are structurally invisible to a port keyed detector. The dynamic
gap surface (keyed on `(transport, port)`) will **never** flag them. If the
catalog lists them as "known but undissected," the static `protocols` command
must present them with a transport marker like `Link-Layer` / `EtherType 0xNNNN`
and the dynamic detector must NOT be expected to corroborate them. This mirrors
wirerust's existing ARP handling (L2, no port, decoded via `DecodedFrame::Arp`).

**Note on port 102 collision:** S7comm, S7comm-plus, IEC 61850 MMS, and
ICCP/TASE.2 all default to TCP/102 [P1][P2][P5]. A catalog keyed on port cannot
distinguish them; the catalog entry for 102 should either list the family or
note the ambiguity. This is the single most important port-collision case in
the ICS space and should be documented in ADR-012.

---

## Q2 — Common IT / Enterprise Protocols (OQ-1 scope)

For a tool whose primary focus is ICS, the question is which IT protocols to
list in the coverage catalog. Evidence: OT monitoring platforms (Dragos, Nozomi)
explicitly advertise inspecting **both** "industrial and IT protocols" — Dragos
claims "600+ industrial and IT protocols … deep focus on Layer 7" [T9]. IT
protocols matter in OT because Purdue Level 2–3 (HMI, historian, engineering
workstation, jump hosts) runs heavy IT traffic, and unexpected IT protocols
(RDP, SMB, SSH) in a control zone are themselves security-relevant.

**Recommended IT protocols worth cataloging (in an ICS-focused tool):**

| Protocol | Transport | Port(s) | Why relevant in OT | wirerust status |
|----------|-----------|---------|--------------------|-----------------|
| TLS | TCP | 443, 8443, + any | already dissected | **SUPPORTED** |
| HTTP | TCP | 80, 8080 | already dissected; HMI web UIs | **SUPPORTED** |
| DNS | UDP/TCP | 53 | already dissected | **SUPPORTED** |
| SSH | TCP | 22 | remote admin of PLCs/gateways; lateral movement | catalog-only |
| SMB | TCP | 445 (139) | file share, engineering WS, WannaCry/Industroyer vector | catalog-only |
| RDP | TCP | 3389 | HMI/EWS remote access; top OT intrusion vector | catalog-only |
| FTP/SFTP | TCP | 21 / 22 | firmware & config transfer | catalog-only |
| Telnet | TCP | 23 | legacy device CLI (still common in OT) | catalog-only |
| SNMP | UDP | 161/162 | device management/monitoring | catalog-only |
| NTP | UDP | 123 | time sync (critical for SV/GOOSE/SCADA timestamps) | catalog-only |
| SMTP | TCP | 25/587 | alarm email from historians/RTUs | catalog-only |
| LDAP/Kerberos | TCP/UDP | 389 / 88 | AD auth in IT/OT DMZ | catalog-only |
| SIP/RTP | UDP/TCP | 5060 / dynamic | VoIP/intercom in facilities | lower priority |

`decoder.rs` already emits `app_protocol_hint` values for SSH (22) and SMB (445)
per the delta-analysis regression baseline — so the tool already "knows about"
these ports without dissecting them, which fits the catalog-only tier cleanly.

**Recommendation on scope (feeds OQ-1): ICS + a curated core-IT set.** Not
ICS-only, not "all IANA." Rationale:

1. **ICS-only is too narrow.** Real OT captures are dominated by IT traffic at
   the SCADA/supervisory layers; a coverage report that stays silent on
   TLS/SSH/RDP/SMB gives operators a false sense that the tool "saw everything
   industrial" while ignoring the actual intrusion vectors. Dragos/Nozomi both
   catalog IT protocols for exactly this reason [T9][T10].
2. **All-IANA is noise.** The IANA registry has ~14k entries [T12]; dumping them
   makes the "unsupported" list useless and unmaintainable. Coverage catalogs
   are a curated signal, not a port dump.
3. **A curated core-IT tier (~12 protocols above) is the sweet spot** — it
   covers the OT intrusion-relevant IT protocols and aligns with what the
   decoder already hints (SSH/SMB), at negligible catalog size cost.

Mark each catalog entry with a `category` (ICS | IT | Both) so the `protocols`
command can filter (e.g. `--ics-only`). This is already anticipated by
BC-2.18.002 (`category: ICS/Network`) in the delta-analysis.

---

## Q3 — How Comparable Tools Model Protocol Coverage

### Wireshark — dissector tables keyed by port + heuristics
Wireshark registers each protocol via `proto_register_protocol()` and hooks
subdissectors into **dissector tables** such as `"tcp.port"` and `"udp.port"`;
the parent (TCP/UDP) dissector consults the table by port to pick a child
dissector [T13][T1]. Unclaimed payloads fall through to a generic **"Data"**
node — Wireshark's implicit "unknown" state — and there is an explicit "Disable
dissection of all protocols" mode [T2][T3]. **Reusable dataset:** the cited docs
do **not** document a machine-readable export of all dissectors, BUT
`tshark -G protocols` / `tshark -G dissectors` are the well-known CLI dumps
(**[unverified]** — not in cited sources, but standard Wireshark behavior).
Wireshark also ships `manuf` (OUI→vendor) and `services` files. Verdict:
partially sourceable, but requires a Wireshark install to export.

### Zeek — analyzer framework + Dynamic Protocol Detection (DPD)
Zeek attaches **analyzers** to *connections* (not ports); protocols are
represented as analyzers, and DPD uses **payload signatures** (`@load-sigs`) to
attach analyzers **port-independently** — e.g. HTTP on a non-standard port or
SSH on 443 [T4][C2]. It logs detection outcomes to `dpd.log`. Zeek's
`proto-analyzers` docs enumerate analyzers (DNS over UDP+TCP, etc.) but as
human-readable docs, not a stable API [T5]. This is the model wirerust's dynamic
gap detection is a (much simpler) cousin of: Zeek proves that content-first
detection beats port-first — which is exactly wirerust's existing dispatcher
precedence (VP-004: "TLS signature beats port").

### Suricata — app-layer detection with explicit unknown/failed states
Suricata detects app-layer protocols on **flows** and — uniquely — exposes
`unknown` and `failed` as **first-class rule keywords**: `failed` = detection
ran and no known protocol matched; `unknown` = not yet resolved [T6][T11]. This
is the cleanest published taxonomy for "known vs undissected" and is directly
reusable as wirerust's vocabulary: wirerust's `DispatchTarget::None` maps to
Suricata's `failed`, and an in-progress flow maps to `unknown`. **Recommend
adopting the known / unknown / failed(undissected) tri-state naming in ADR-012
and the `protocols` output**, citing Suricata as prior art.

### Malcolm / Arkime — aggregation + tagging (no independent catalog)
Malcolm is an aggregator that inherits protocol coverage from Zeek + Suricata +
Arkime and **tags sessions** with the union of their outputs [T7]. Its coverage
= union of upstream; it has no independent machine-readable protocol catalog
beyond distinct tag values seen in traffic. Arkime indexes sessions with
protocol tags; missing tag = unknown. Not a catalog source, but validates the
"report what was seen vs what is knowable" distinction relevant to OQ-2.

### OT platforms — Dragos & Nozomi (marketing lists, no open API)
Dragos advertises "600+ industrial and IT protocols … deep focus on Layer 7"
[T9] via its "Intelligence Fabric" [T8]. Nozomi publishes a "Protocol Support
List" [T10]. **Both are human-readable marketing resources, not machine-readable
APIs** — not programmatically sourceable. Useful as a naming/prioritization
reference for which ICS protocols to include, not as a data feed.

### Reusable open datasets / taxonomies — recommendation
| Source | Machine-readable? | Use for wirerust catalog |
|--------|-------------------|--------------------------|
| **IANA Service Name & Port Registry** [T12][C4] | **Yes** — published `.txt`/`.csv`/`.xml`, parseable | Authoritative default-port validation; **but** ~14k entries, no L2 protocols, no ICS semantics. Use as a *validation reference*, not the catalog itself |
| Wireshark `tshark -G protocols`, `manuf` | Yes (needs install) [T13] | Cross-check names; not a build dependency |
| Zeek proto-analyzers docs | Semi (scrape) [T5] | Reference for content-detectable protocol set |
| Suricata app-layer keyword list | Semi [T6] | Adopt known/unknown/failed taxonomy |
| Dragos/Nozomi lists | No (marketing) [T9][T10] | Prioritization only |

**Verdict:** No single open dataset can auto-generate the catalog, because the
most important ICS protocols (a) frequently lack IANA ports, and (b) include L2
protocols IANA does not cover. **A hand-curated static catalog (delta-analysis
Option A) is the correct call**, with IANA used to *verify* the port numbers of
entries that have registrations. This directly supports the delta-analysis
recommendation of a compile-time `KNOWN_PROTOCOLS` array over an external file.

---

## Q4 — Port-Based Detection Caveats (for the spec)

Strong, well-sourced evidence that port→protocol mapping is a heuristic, not
ground truth. These become explicit caveats in the spec / `protocols --unsupported`
help text / dynamic-gap report footnotes.

1. **Ephemeral / dynamic ports (49152–65535) carry no service meaning** [C11].
   A SCADA master polls devices from ephemeral source ports; only the
   *server-side* port is meaningful, and only in combination with role + IP.
   The dynamic detector must key on the *listening* side, not the ephemeral
   side. wirerust's flow-key `(lower, upper)` port normalization already
   sidesteps direction but must not treat an ephemeral high port as a protocol.

2. **Protocols run on non-standard ports** [C1][C16]. Port assignment "is not
   set in stone" (Keysight [C1]); admins move services freely. DNP3 docs
   explicitly warn that moving off 20000 "does not add security" — proving it
   happens [C16]. Consequences: **false negatives** (protocol on a non-default
   port is missed) and **false positives** (default port hosting something else
   is mislabeled). The catalog's port list is "canonical/expected," never
   "guaranteed."

3. **Port sharing / multiplexing on 80 & 443** [C1][C6]. "With so many
   applications using port 80 and 443, port-based identification is no longer
   feasible" [C1]. HTTPS also commonly runs on 8443 (Tomcat) [C6]; a single 443
   endpoint may host many apps demuxed by SNI/Host. wirerust's TLS analyzer is
   content-first, which is the right mitigation — but the *catalog* should not
   claim 443 == TLS-only.

4. **L2 / multicast protocols have NO port** [C10][C12][C13][C14]. ARP "has no
   protocol number … it is a layer-2 protocol" [C14]; GOOSE/SV are Ethernet
   multicast [C12]; PROFINET DCP is "an Ethernet link layer protocol" [C10];
   EtherCAT is EtherType 0x88A4 [C13]. **The dynamic gap detector will never see
   these.** The spec MUST state that absence from the gap report does not mean
   absence from the wire — L2 protocols are out of scope for port-based gap
   detection by construction.

5. **Tunneling & encryption hide the inner protocol** [C1][C16]. VPN/TLS
   wrapping presents only the outer port; DNP3-Secure (19999) is "rarely used —
   most deployments tunnel over VPN or use DNP3-SA on 20000" [C16]; Modbus
   Security uses TLS on **802** [C15]. A port-based view sees the tunnel, not
   the payload.

6. **Registry uniqueness ≠ wire uniqueness** [C4]. IANA's service+transport+port
   tuple is unique *in the registry only*; it does not constrain what a host
   actually runs [C4]. Also, the same port means different things on different
   hosts.

7. **UDP is stateless** [C8][C16]. No handshake to validate a guess; "UDP 47808
   == BACnet" is shallow and cannot confirm the payload is really BACnet [C8].
   Matters directly for OQ-5.

8. **Best practice is content-first** [C1][C2][C3][C7]. Zeek DPD and nDPI both
   treat the port as a *hint that orders which dissector to try first*, then
   validate against payload; nDPI tries SSH first on :22 but falls through if
   the payload doesn't match, labeling "Unknown" only after all dissectors fail
   [C3]. A signature-based study cut unidentified traffic by ~11% vs baseline
   [C7]; Moore & Papagiannaki established port-based unreliability years ago
   [C17]. **wirerust's dispatcher is already content-first (VP-004)** — the
   catalog/gap layer should inherit that philosophy and never regress to
   port-as-truth.

**Suggested spec caveat block (verbatim-ready):**
> Port-based coverage detection is a heuristic. Reported gaps reflect
> *observed transport+port pairs that no analyzer claimed*; they are neither a
> complete list of undissected protocols (Layer-2/multicast protocols such as
> GOOSE, Sampled Values, PROFINET-RT/DCP, and EtherCAT have no port and are
> never reported here) nor an authoritative protocol identification (a service
> may run on a non-standard port, and a canonical port may host unrelated
> traffic or a tunnel). Catalog port numbers are canonical defaults, not
> guarantees.

---

## Q5 — Recommendation

### Recommended initial catalog (this cycle)

Include **~28–32 entries** across three categories, keyed by the fields already
specified in BC-2.18.002 (name, transport, canonical_port(s), category,
description). Ship these now:

**Supported (7)** — derived from the dispatcher per BC-2.18.003, not hand-listed:
Modbus/TCP (502), DNP3 (20000), EtherNet/IP+CIP (44818/2222), TLS (443/8443),
ARP (L2), DNS (53), HTTP (80/8080).

**ICS — known/unsupported, port-detectable (Tier 1, include all ~9):**
S7comm (102), S7comm-plus (102), IEC-104 (2404), IEC 61850 MMS (102),
BACnet/IP (47808/UDP), OPC-UA binary (4840), PROFINET RPC (34962–34964),
ICCP/TASE.2 (102), HART-IP (5094).

**ICS — L2/multicast, known/undetectable (include, flagged `Link-Layer`, ~5):**
GOOSE (0x88B8), Sampled Values (0x88BA), PROFINET-DCP/RT (0x8892),
EtherCAT (0x88A4), Ethernet POWERLINK (EtherType, mark **[unverified]**).

**IT — known/unsupported, curated core (include ~9):**
SSH (22), SMB (445), RDP (3389), FTP (21), Telnet (23), SNMP (161/162),
NTP (123), SMTP (25), LDAP (389).

**Defer to a follow-on cycle (document as "not yet cataloged"):**
- Vendor-specific low-prevalence ICS with fuzzy ports: OMRON FINS, GE-SRTP,
  Foundation Fieldbus HSE, CODESYS, MELSEC/MC, PCWorx, ROC Plus, Niagara Fox,
  Red Lion, CC-Link IE. (Include a *few* high-value ones — FINS 9600, GE-SRTP
  18245 — only if catalog authoring cost is trivial; otherwise defer. Their
  configurable/unverified ports add maintenance burden for low signal.)
- CIP Safety / PCCC as *sub-entries* of EtherNet/IP rather than standalone rows
  (they share 44818/2222 — no distinct port to key on).
- OPC-UA over HTTP/HTTPS (indistinguishable from web at port layer — would
  create false catalog signal).

Rationale for the ~30 number: large enough to be a genuinely useful coverage
signal spanning the OT protocols operators actually ask about, small enough to
hand-maintain and to keep the `unsupported` list actionable. Matches the
curated-signal philosophy that distinguishes a coverage catalog from a port
dump (Q3).

### Recommendation on OQ-5 (TCP-only vs TCP+UDP dynamic detection)

**Recommend: TCP-only dynamic detection THIS cycle, but design the catalog and
report to be UDP-aware, and commit to UDP gap detection as the immediate
follow-on.**

Reasoning:
- The delta-analysis is correct that `StreamDispatcher` handles only TCP, so
  TCP-only is the low-risk path that avoids touching the VP-004 Kani zone and
  avoids new UDP-flow plumbing in `main.rs` this cycle.
- **HOWEVER, the evidence shows UDP is not a minor gap for ICS.** The single
  most prevalent building-automation protocol, **BACnet/IP, is UDP/47808**
  [C8][P4], and it is one of the Tier-1 catalog entries. Several other Tier-1/2
  entries are UDP or UDP-capable: DNP3 (UDP/20000 allowed) [C16], PROFINET RPC
  (UDP 34962–34964) [P14], SNMP, NTP, DNS. A TCP-only dynamic detector will
  **structurally never** flag a BACnet gap even though BACnet is exactly the
  kind of "undissected but present" traffic the feature exists to surface.
- This creates a **documentation hazard**: an operator running a BACnet-heavy
  capture would see "no coverage gaps" and wrongly conclude full coverage.

Therefore:
1. Ship TCP-only dynamic detection now (matches delta-analysis OQ-5 option a).
2. **Mandatory caveat in the report**: "Dynamic gap detection currently covers
   TCP flows only. UDP-based protocols (e.g. BACnet/IP on 47808, SNMP, NTP) and
   Layer-2 protocols are not represented in the dynamic gap report; consult the
   static `protocols` catalog for the full known-protocol set."
3. Keep `canonical_ports` transport-tagged (TCP vs UDP vs Link-Layer) in the
   catalog so the static command already tells the truth about UDP/L2 protocols
   even while the dynamic surface is TCP-only.
4. File the UDP-flow gap-tracking (a lightweight per-`(udp, port)` counter in
   the decode loop, outside the dispatcher) as the very next story — it is the
   natural completion and the evidence says BACnet makes it high-value.

This is a stronger position than the delta-analysis's "UDP can be a follow-on"
phrasing: the follow-on should be **planned and caveated now**, not left open,
precisely because BACnet/UDP is a Tier-1 catalog protocol.

---

## Inconclusive / Low-Confidence Items (flagged)

- **Ethernet POWERLINK EtherType** — not in cited primary sources; 0x88AB is
  commonly cited from model knowledge. Mark `[unverified]` in the catalog or
  omit the EtherType value.
- **CC-Link IE port/EtherType** — no port or EtherType found in cited sources;
  treat as L2 vendor-specific, **inconclusive**. Recommend deferring.
- **S7comm-plus port** — 102/TCP is vendor/practice, not a cited primary
  standard [P3]; high-confidence but flagged.
- **CODESYS / MELSEC default ports** — "vendor application specific" [P1]; any
  specific number (1217, 2455, 5000-range) is model knowledge, `[unverified]`.
  Defer or catalog without a firm port.
- **`tshark -G` machine-readable dumps** — standard behavior but not in the
  cited Wireshark docs [T13]; verify against a local install before relying on
  it as a data source.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) ICS/OT protocol universe with ports/transports/L2 flags [Q1]; (2) how Wireshark/Zeek/Suricata/Malcolm/Arkime/Dragos/Nozomi model coverage + open datasets [Q3]; (3) port-based detection limitations + ICS caveats [Q4]. All `reasoning_effort: high`, `strip_thinking: true`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not needed — no single-library API question |
| Tavily | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read (local) | 5 | Delta-analysis context + 3 large research result files + directory checks |
| Training data | ~4 areas | POWERLINK EtherType, CODESYS/MELSEC ports, tshark -G, wirerust supported-set mapping — ALL flagged `[unverified]` inline |

**Total MCP tool calls:** 3 (all `perplexity_research`, PRIMARY tool)
**Training data reliance:** low — every model-knowledge claim is explicitly
marked `[unverified]` or `[inconclusive]`; all port numbers and transports are
cited to IANA, vendor docs, Wireshark, or flagged.

---

## References

**Q1 — Protocol universe (research stream 1)**
- [P1] PacketViper — Known SCADA/ICS Network Ports — https://help.packetviper.com/portal/en/kb/articles/known-scada-ics-network-ports
- [P2] Wireshark Wiki — S7comm — https://wiki.wireshark.org/S7comm
- [P3] Litmus Edge — Siemens S7comm-plus — https://docs.litmus.io/litmusedge/quickstart-guide/industrial-systems-connection-guide/siemens/siemens-s7commplus
- [P4] Chipkin — Changing default BACnet port 47808 — https://store.chipkin.com/articles/how-to-change-the-default-bacnet-port-47808-in-cas-bacnet-explorer
- [P5] scadaprotocols.com — IEC 61850 MMS port 102 — https://scadaprotocols.com/iec-61850-mms-port-number-tcp-102/
- [P6] iGrid T&D — GOOSE messaging — https://www.igrid-td.com/smartguide/iec61850/goose-messaging/
- [P7] scadaprotocols.com — IEC 61850 Sampled Values — https://scadaprotocols.com/iec-61850-sampled-values-explained/
- [P8] IANA — Service Name & Transport Protocol Port Number Registry — https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml
- [P9] Unified Automation — OPC-UA troubleshooting — https://documentation.unified-automation.com/uasdknet/4.1.0/html/L1Trouble.html
- [P10] Wireshark Wiki — HART-IP — https://wiki.wireshark.org/HART-IP
- [P11] EMQX Neuron — OMRON FINS — https://docs.emqx.com/en/neuron/latest/configuration/south-devices/omron-fins/omron-fins.html
- [P12] AutomationDirect C-more — GE Ethernet SRTP (port 18245) — https://cdn.automationdirect.com/static/helpfiles/c-more/cm5/Content/370.htm
- [P13] connected.app — Port 1089 (Foundation Fieldbus HSE) — https://www.connected.app/ports/1089
- [P14] HMS Networks — TCP/UDP ports used by PROFINET — https://www.hms-networks.com/support/tech-support/kb-articles/6794937733394-Which-TCP-UDP-ports-are-used-by-PROFINET-
- [P15] Teltonika community — TSW212 blocks PROFINET DCP multicast — https://community.teltonika.lt/t/tsw212-blocks-profinet-dcp-identify-request-multicast/12609
- [P16] EtherCAT Technology Group — Technology (EtherType 0x88A4) — https://www.ethercat.org/en/technology.html

**Q3 — Tool coverage models (research stream 2)**
- [T1] Wireshark Q&A — UDP port number / dissector tables — https://osqa-ask.wireshark.org/questions/62419/udp-port-number/
- [T2] Wireshark man page — https://www.wireshark.org/docs/man-pages/wireshark.html
- [T3] Wireshark Q&A — Unknown protocol analysis — https://osqa-ask.wireshark.org/questions/16996/unknown-protocol-analysis/
- [T4] Zeek — Dynamic Protocol Detection howto — https://old.zeek.org/development/howtos/dpd.html
- [T5] Zeek docs (LTS) — proto-analyzers — https://docs.zeek.org/en/lts/script-reference/proto-analyzers.html
- [T6] Suricata docs — app-layer keywords (unknown/failed) — https://docs.suricata.io/en/latest/rules/app-layer.html
- [T7] Malcolm — Capabilities and limitations — https://malcolm.fyi/docs/capabilities-and-limitations.html
- [T8] Dragos — Platform AI/OT visibility (Intelligence Fabric) — https://www.dragos.com/blog/dragos-platform-ai-ot-security-visibility
- [T9] Dragos — Network monitoring ("600+ protocols") — https://www.dragos.com/cybersecurity-platform/network-monitoring
- [T10] Nozomi Networks — Protocol Support List — https://www.nozominetworks.com/resources/protocol-support-list
- [T11] Stamus Networks — What protocols are used in Suricata — https://www.stamus-networks.com/blog/what-protocols-are-used-in-suricata
- [T12] IANA — Port registry (.txt) — https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.txt
- [T13] Wireshark Dev Guide — Adding a basic dissector — https://www.wireshark.org/docs/wsdg_html_chunked/ChDissectAdd.html

**Q4 — Port-based detection caveats (research stream 3)**
- [C1] Keysight — 3 problems with using port numbers to identify applications — https://www.keysight.com/blogs/en/tech/nwvs/2020/05/22/3-problems-with-using-port-numbers-to-identify-applications
- [C2] Zeek docs — dpd.log — https://docs.zeek.org/en/v6.0.9/logs/dpd.html
- [C3] ntop — nDPI internals & FAQ — https://www.ntop.org/ndpi-internals-and-frequent-questions/
- [C4] IETF — draft-ietf-tsvwg-iana-ports (IANA port procedures) — https://www.ietf.org/archive/id/draft-ietf-tsvwg-iana-ports-10.html
- [C5] Chipkin — Modbus types (RTU/TCP) — https://store.chipkin.com/articles/different-types-of-modbus-such-as-rtu-tcp-etc
- [C6] Router-Switch — HTTPS port 443 vs 8443 — https://www.router-switch.com/faq/difference-between-https-port-443-and-8443.html
- [C7] IEEE Xplore — Signature-based traffic identification — https://ieeexplore.ieee.org/document/4238832/
- [C8] IANA — bacnet / 47808 — https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml?search=47808
- [C9] whatportis — IEC 60870-5-104 / 2404 — https://whatportis.com/ports/2404_iec-60870-5-104-used-to-send-electric-power-telecontrol-messages-between-two-systems-via-directly-connected-data-circuits
- [C10] PROFINET University — PROFINET DCP — https://profinetuniversity.com/naming-addressing/profinet-dcp/
- [C11] Coursera — Ephemeral ports — https://www.coursera.org/articles/ephemeral-ports
- [C12] scadaprotocols.com — IEC 61850 Sampled Values — https://scadaprotocols.com/iec-61850-sampled-values-explained/
- [C13] Wireshark Wiki — EtherCAT — https://wiki.wireshark.org/Protocols/ethercat
- [C14] Cisco Learning Network — ARP (no protocol number, L2) — https://learningnetwork.cisco.com/s/question/0D53i00000Kt4QJCAZ/arp
- [C15] scadaprotocols.com — Modbus TCP/IP port 502 (IANA) — https://scadaprotocols.com/modbus-tcp-ip-port-502-iana/
- [C16] scadaprotocols.com — DNP3 port 20000 — https://scadaprotocols.com/dnp3-port-20000/
- [C17] Moore & Papagiannaki, "Towards the Accurate Identification of Network Applications" (via clustering study) — https://www.scienceopen.com/hosted-document?doi=10.14236%2Fewic%2FICS2018.13
</content>
</invoke>
