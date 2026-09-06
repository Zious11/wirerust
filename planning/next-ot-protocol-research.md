# Next OT/ICS Protocol Research Brief — wirerust

**Date:** 2026-09-06
**Author:** research-agent (jaredbrichards@gmail.com)
**Scope:** Rank the unsupported OT/ICS protocols in wirerust's catalog and recommend the single
highest-probability next protocol to add. wirerust is a passive, read-only ICS/OT network
forensics/security analyzer (pcap/pcapng dissection, protocol classification, MITRE ATT&CK for ICS
attribution). Currently supports 8 protocols: Modbus/TCP, DNP3, EtherNet/IP+CIP, IEC 60870-5-104,
TLS, ARP, DNS, HTTP.

**Judging dimensions (threat-weighted, per tool mission):**
1. Real-world prevalence / installed base
2. Threat / security relevance (CVEs, CISA advisories, APT/malware, ATT&CK mapping)
3. Implementation feasibility for a passive Rust dissector

> **Method note:** This brief is grounded in three deep `perplexity_research` (sonar-deep-research)
> passes — one per dimension — plus the citations they surfaced (CISA/CISA-ICS, Dragos, Mandiant/
> Google Cloud, MITRE ATT&CK for ICS + Caldera-for-OT, HMS Networks, BACnet International/BSRIA,
> OPC Foundation/VDMA, Censys/Shodan/Bitsight, Wireshark/Zeek-ICSNPP/Suricata registries). See
> Research Methods at the end. Findings are date-stamped "as of 2026-09-06"; the ICS landscape
> shifts quickly.

---

## TL;DR

**Top recommendation: Siemens S7comm (classic, over TCP 102 / ISO-on-TCP).**

It decisively wins the threat dimension (the weighted priority for a forensics tool) and is top-tier
on prevalence, at a *moderate* and *strategically reusable* feasibility cost. It is the only candidate
with **both** a confirmed destructive campaign (Stuxnet) **and** active in-the-wild targeting reported
in 2026 (CISA AA26-231A). Building the TPKT/COTP ISO-on-TCP dispatch layer it requires is a one-time
investment that *also* unlocks IEC 61850 MMS and ICCP/TASE.2 later (all three share port 102).

**Runner-up: BACnet/IP.** The lowest-risk, highest-feasibility option (clean UDP 47808, open spec,
mature Rust prior art) with excellent prevalence and rich ATT&CK mappings. Its only real weakness is
the absence of a marquee incident. Pick this instead of S7comm if the team wants the fastest clean win
over threat-signal maximization.

**De-prioritize:** deep S7comm-**plus** dissection (encrypted/obfuscated — passively infeasible),
ICCP/TASE.2 (sparsest evidence + biggest prior-art gap), HART-IP (lowest footprint, gated spec, v2 is
mandatory TLS/DTLS), and all L2 protocols incl. PROFINET RT/DCP, GOOSE, SV, EtherCAT, POWERLINK
(require a new non-port dispatch path — a materially larger architectural lift).

---

## Ranked shortlist (top 4)

### 1. Siemens S7comm (classic) — **RECOMMENDED**

- **Prevalence (high).** Siemens is the #1 PLC vendor (~20.1% of PLC revenue, one 2025 estimate) and
  S7-300 alone has shipped in the millions; classic S7comm persists on the long-lived S7-300/400
  installed base, while S7-1200/1500 add S7comm-plus. Siemens publishes no auditable installed-base
  total, but S7 is near-ubiquitous in Siemens-heavy manufacturing captures. Censys fingerprinted 4,117
  Internet-exposed S7-1200 hosts (2026-07-30). A raw `port:102` count is unusable (shared with MMS/ICCP).
- **Threat (highest of the pool).** Stuxnet definitively manipulated S7-300 PLC programs (replaced the
  Siemens PLC-comms DLL, downloaded malicious blocks, manipulated I/O and hid it). CISA/NSA/FBI/DOE/EPA
  issued **AA26-231A (Aug 2026)** on active, AI-assisted `snap7`/`python-snap7` tooling enumerating and
  reading/writing S7 PLC memory over **S7comm TCP/102**. High advisory density (10+ clear S7-family CISA
  advisories; e.g. CVE-2020-15782 unauth memory read/write on TCP/102, CVSS 8.1). Richest ATT&CK-for-ICS
  mapping of any candidate: T0843 Program Download, T0889 Modify Program, T0821 Modify Controller
  Tasking, T0836 Modify Parameter, T0835 Manipulate I/O Image, T0888/T0846 discovery, T0851 Rootkit,
  T0873.001 Siemens Project File Infection, etc. For an ATT&CK-attribution forensics tool this is the
  highest-value payload in the catalog.
- **Feasibility (medium — the good kind).** Classic S7comm is proprietary/reverse-engineered but very
  well understood: mature Wireshark `packet-s7comm.c`, CISA/INL **ICSNPP-S7Comm** Zeek analyzer,
  libnodave/libs7comm C, plus reverse-engineered layouts. Framing: TCP reassembly → TPKT length → COTP
  (CR/CC/DT, segmentation) → S7 header/param/data. **Port-102 disambiguation is a simple COTP user-data
  protocol-ID check: `0x32` = classic S7comm, `0x72` = S7comm-plus, OSI Session/Presentation/ACSE =
  MMS-family.** wirerust already does TCP stream dispatch and TLS reassembly, so this fits the existing
  model. **Caveat:** modern **S7comm-plus** uses private integrity/anti-replay algorithms and
  increasingly TLS — not deeply dissectable from pcap without keys. Recommendation is therefore *classic
  S7comm deep dissection + framing-level detection/metadata for S7comm-plus* (`0x72` classification,
  session-setup observation), which is still forensically useful.

### 2. BACnet/IP — **RUNNER-UP (fastest clean win)**

- **Prevalence (very high, in-scope if BAS counts as OT).** BACnet International reports 25M+ devices;
  BSRIA finds BACnet specified in 77% of global building-automation projects (up from 64% in 2018); a
  2024 estimate gives wired BACnet 63.5% of commercial BAS connectivity. Shodan: ~6,850 endpoints on
  UDP/47808 (2026-01-18). Extremely common in HVAC/lighting/access-control captures.
- **Threat (moderate — heavy advisory/exposure, weak confirmed-incident).** 10+ BACnet-explicit CISA
  advisories (Siemens APOGEE/TALON and SAUTER devices reaching CVSS 9.8 for missing auth/cleartext).
  BACnet's lack of network-layer auth is a recurring CVE theme. **But** no well-substantiated named APT
  or purpose-built malware natively manipulates BACnet (unlike Stuxnet/Industroyer). MITRE ships a
  BACnet Caldera-for-OT plugin mapping Who-Is→T0846, Who-Has→T0888, WriteProperty/AtomicWriteFile→
  T0831 Manipulation of Control, ReinitializeDevice→T0816 Device Restart/Shutdown, plus T0801/T0802/
  T0861 — so ATT&CK attribution is well-supported even absent a marquee campaign.
- **Feasibility (highest / lowest-risk).** Open published standard (ANSI/ASHRAE 135 / ISO 16484-5, free
  read-only preview). **Clean UDP 47808 dispatch — no port collision.** UDP preserves message
  boundaries; stack is UDP→BVLC(type/function/length)→NPDU→APDU. Mature Wireshark (`bvlc`/`bacnet`/
  `bacapp`), CISA **ICSNPP-BACnet** Zeek plugin, royalty-free C `bacnet-stack`, and multiple **Rust**
  decoders (`bacnet_parse`, `bacnet-rs`, `rustbac`). Complexity is in breadth (service/object/property
  catalogue, BBMD, APDU segmentation), not in framing. This is the shortest path to a safe Rust passive
  dissector in the whole pool.

### 3. OPC-UA (Binary/UASC over TCP 4840) — **emerging-threat pick**

- **Prevalence (high, fastest-growing).** OPC Foundation cites 52M OPC-enabled applications (mixes
  classic OPC + UA); VDMA 2025 survey: 57% of surveyed machinery firms use OPC-UA productively, ~50% of
  new products UA-capable, 71% rate it highly relevant. Bitsight observed 13,766 exposed servers on
  TCP/4840 (2024-25). Concentrated at supervisory / machine-to-MES / gateway layers and rising.
- **Threat (strong, recent).** **PIPEDREAM/INCONTROLLER** (Dragos-attributed CHERNOVITE) includes
  TAGRUN/MOUSEHOLE, which natively scans OPC-UA servers, enumerates tags, brute-forces creds, and
  reads/writes tag values → T0846.001, T0861 Point & Tag Identification, T0859 Valid Accounts,
  T1692.001 Unauthorized Command Message. High advisory volume (9+ OPC-UA-explicit CISA advisories,
  several CVSS 9.3–9.8). No confirmed disruptive deployment, but it is the most important *emerging*
  multi-vendor target.
- **Feasibility (medium framing; best openly-browsable spec).** OPC UA Part 6 mapping/UASC is free and
  online (IEC 62541). Clean TCP 4840. Explicit chunk header (MSG/OPN/CLO, size, secure-channel ID).
  Mature Wireshark `opcua`; CISA **ICSNPP-OPCUA-Binary** Zeek plugin; strong Rust prior art (async-opcua
  chunk/type decoders, `open62541` C). **Caveat:** encrypted SecureConversation payloads need keys;
  `None`/`Sign` modes and the handshake/metadata are always readable.

### 4. IEC 61850 MMS (over TCP 102) — **strong strategic fit, higher build cost**

- **Prevalence (sector-concentrated).** Dominant in electric-power substations (utility poll: 65% already
  working with IEC 61850; earlier data projected 73% of new international substations), thin elsewhere.
  Top-tier *within* utility substation captures; #5 globally.
- **Threat (strong malware relevance).** CRASHOVERRIDE/INDUSTROYER carried a **native IEC 61850/MMS
  module** (scanned TCP/102, issued MMS GetNameList, read breaker `stVal`). *Correction for the record:*
  INDUSTROYER2 (2022) used **IEC-104 only**, not 61850. MITRE ships an IEC 61850 Caldera-for-OT plugin
  (MMS only — not GOOSE/SV) mapping T0802/T0861/T0801/T0809/T0855/T0836. Moderate advisory volume with
  critical stack CVEs (libIEC61850 CVSS 10.0).
- **Feasibility (high difficulty).** Open standards (ISO 9506 MMS, IEC 61850-8-1) but full OSI stack +
  ASN.1 BER: TCP→TPKT→COTP→Session→Presentation→ACSE→MMS/BER, with presentation-context negotiation and
  invoke-ID correlation. C prior art is excellent (libIEC61850, Wireshark), Rust passive prior art is
  thin. **Shares port 102** — same dispatch layer as S7comm/ICCP. This is the natural *second* port-102
  protocol after S7comm establishes the TPKT/COTP path.

---

## Top recommendation — trade-off stated plainly

**Add classic S7comm next.**

- **Why it wins:** For a threat-forensics/ATT&CK-attribution tool, the threat dimension is weighted
  heavily, and S7comm dominates it — the only candidate with a confirmed destructive campaign (Stuxnet)
  *and* active 2026 in-the-wild targeting (CISA AA26-231A), the richest ATT&CK-for-ICS technique mapping
  in the pool, and top-tier prevalence in Siemens-heavy manufacturing OT.
- **The trade-off you accept:** (a) A TCP-102 ISO-on-TCP dispatch layer (TPKT/COTP) must be built, and
  classification must key off the COTP protocol-ID byte (`0x32`), not the port alone. This is modest and
  fits wirerust's existing stream-dispatch model. (b) Modern **S7comm-plus** (S7-1200/1500) is
  encrypted/obfuscated and **not** deeply dissectable passively — scope the story as *classic S7comm
  full dissection + S7comm-plus framing-level classification/metadata only.*
- **The strategic dividend:** the TPKT/COTP layer is a prerequisite shared by IEC 61850 MMS and
  ICCP/TASE.2. Building it for S7comm makes those two catalog protocols materially cheaper later —
  three catalog entries unlocked from one architectural investment on the same port.

## Runner-up — and why not #1

**BACnet/IP.** If the team prioritizes feasibility and a fast, low-risk delivery over maximizing threat
signal, BACnet is the better pick: clean UDP 47808 (zero port-collision work), a free open spec,
message-boundary-preserving framing, and the deepest Rust prior art of any candidate — plus strong
prevalence and a ready-made MITRE Caldera ATT&CK mapping. It is **not** #1 only because, for a
*security/forensics* analyzer, its threat signal is the pool's weakest: there is no Stuxnet/Industroyer-
class campaign that natively manipulates BACnet. If wirerust's near-term users skew toward
building-automation / smart-building forensics rather than industrial/energy, promote BACnet to #1.

## Protocols to explicitly DE-prioritize

- **S7comm-plus (deep dissection):** private integrity/anti-replay algorithms and increasingly TLS make
  payload decoding from pcap infeasible without keys. Only framing-level classification (`0x72`) is
  viable. Do not scope a full semantic dissector.
- **ICCP/TASE.2:** sparsest public incident + advisory record (essentially one direct CISA stack
  advisory, CVE-2022-38138), **the largest prior-art gap** (no dedicated upstream Wireshark/Zeek/Suricata
  dissector, no credible Rust implementation), low endpoint density (a utility has only a handful of ICCP
  peers), *and* the port-102 OSI-stack cost. High consequence but low probability of encounter and highest
  build cost — defer.
- **HART-IP:** lowest visible IP footprint (most HART is 4–20 mA / WirelessHART, not HART-IP nodes),
  access-controlled spec (packet layouts not freely published), and **HART-IP v2 mandates TLS/DTLS** —
  opaque without keys. Low value now.
- **PROFINET (and all L2 protocols: GOOSE, Sampled Values, PROFINET RT/DCP, EtherCAT, POWERLINK):**
  despite PROFINET's #1 installed base (89.2M nodes end-2025, HMS), these ride on raw Ethernet
  (EtherType 0x8892) / L2 with **no TCP/UDP port**, so they require a **new non-port dispatch path** in a
  currently stream/port-keyed dispatcher — a materially larger architectural lift. PROFINET is also a
  *suite* (DCP + cyclic RT + DCE/RPC PNIO-CM), only ~10% covered even by CISA's Spicy parser, with weak
  confirmed-incident evidence. Defer until wirerust intentionally adds an L2 capture/dispatch capability;
  at that point GOOSE (Industroyer-relevant, utility substation) becomes the most interesting L2 target.

---

## Summary ranking table

| Rank | Protocol | Prevalence | Threat relevance | Passive-dissector feasibility | Port dispatch |
|------|----------|-----------|------------------|-------------------------------|---------------|
| **1** | **S7comm (classic)** | High (Siemens #1 PLC) | **Highest** (Stuxnet + active 2026 CISA AA26-231A; richest ATT&CK map) | Medium (RE'd but well-supported; COTP `0x32`) | TCP 102 — needs TPKT/COTP disambig (reusable) |
| **2** | **BACnet/IP** | Very high (BAS) | Moderate (heavy advisories, no marquee campaign) | **Highest** (open spec, Rust prior art) | **Clean UDP 47808** |
| **3** | OPC-UA | High, fastest-growing | Strong/recent (PIPEDREAM native) | Medium (open spec; encryption caveat) | Clean TCP 4840 |
| **4** | IEC 61850 MMS | Utility-concentrated | Strong (Industroyer native module) | High difficulty (OSI+BER) | TCP 102 — shares S7 dispatch |
| 5 | S7comm-plus (deep) | High | High (S7 family) | **Infeasible passively** (encrypted) | TCP 102 |
| 6 | PROFINET | **#1 installed base** | Moderate (weak incidents) | High (L2 suite) | **L2 — new dispatch path** |
| 7 | ICCP/TASE.2 | Low density | Sparse | High + biggest prior-art gap | TCP 102 |
| 8 | HART-IP | Lowest | Sparse | Gated spec; v2 TLS/DTLS | UDP/TCP 5094 |
| — | GOOSE / SV / EtherCAT / POWERLINK | Varies | GOOSE = Industroyer-relevant | L2 — new dispatch path | **L2** |

---

## Unverified / flagged items

- **S7 installed-base absolute total:** Siemens publishes no auditable figure; prevalence is inferred
  from PLC market share + product longevity + exposure scans. High-confidence qualitative, not a hard number.
- **Port-102 exposure counts** cannot be attributed to S7 vs MMS vs ICCP without application-layer
  fingerprinting; the Censys 4,117 figure is S7-1200-specific only.
- **OPC-UA / BACnet exposure counts** vary widely by scanner methodology (e.g., 13,766 Bitsight vs 1,812
  academic on 4840) — treat as order-of-magnitude, not precise.
- **INDUSTROYER 61850 attribution:** public reporting does not prove the 61850 module *caused* the 2016
  outage; and INDUSTROYER2 used IEC-104 only. Recorded to prevent over-claiming in ATT&CK mapping.
- **PIPEDREAM scope:** publicly documented to target OPC-UA, Modbus, CODESYS/Schneider, Omron FINS — **not**
  S7, BACnet, PROFINET, IEC 61850, ICCP, or HART-IP. No confirmed disruptive deployment.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | Dimension 1 prevalence/installed-base (HMS, BACnet Intl/BSRIA, OPC/VDMA, Shodan/Censys/Bitsight); Dimension 2 threat (CISA advisories, Stuxnet/Industroyer/PIPEDREAM, MITRE ATT&CK-for-ICS + Caldera plugins); Dimension 3 feasibility/dissector prior-art (Wireshark, Zeek-ICSNPP, Suricata, C/Rust libs, port-102 dispatch) |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 1 area | Framing/architecture context only (TPKT/COTP, ISO-on-TCP); all substantive claims sourced to the research passes above |

**Total MCP tool calls:** 3 (all `perplexity_research`, `sonar-deep-research`)
**Training data reliance:** low — every prevalence figure, advisory, campaign, ATT&CK mapping, and
prior-art claim is sourced from the deep-research passes and their citations. Training data used only
for well-established protocol-stack framing background.

> Note: the first prevalence-dimension research call also returned a valid deep result that exceeded the
> tool's inline token cap and was saved to disk; it was superseded by a constrained re-run whose full
> output is captured above, so no content was lost. All three dimensions are backed by a completed
> `perplexity_research` pass.
