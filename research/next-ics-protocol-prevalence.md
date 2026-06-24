# Research: Next ICS/OT Protocol for wirerust — Prevalence-Ranked Recommendation

- **Date:** 2026-06-24
- **Type:** domain + general (technology-evaluation hybrid)
- **Agent:** vsdd-factory:research-agent
- **Status:** complete
- **Context:** wirerust already ships ICS analyzers for **Modbus/TCP** and **DNP3** (plus non-ICS analyzers for ARP, DNS, HTTP, TLS). Question: which not-yet-supported ICS/OT protocol is highest real-world deployment prevalence and should be added next.

---

## Executive Summary

**#1 recommendation: EtherNet/IP + CIP (Common Industrial Protocol).**

Across reconciled vendor telemetry (Forescout, Dragos, Claroty), internet-exposure measurement (Censys, academic OT-exposure studies), and market-share data, EtherNet/IP + CIP is the single highest-value protocol to add after Modbus and DNP3. It narrowly edges out Siemens S7comm/S7comm-plus on the combination of (a) **cross-vertical deployment** (it is the dominant industrial-Ethernet protocol in North-American discrete/process manufacturing, oil & gas, mining, food & beverage — verticals where Modbus and DNP3 are NOT dominant), (b) **attack prevalence** (tied for #2 most-attacked OT protocol in Forescout's 2023 telemetry at ~18%, and explicitly called out by Dragos as a focus of adversary exploitation), and (c) **Rust implementation tractability** (formal ODVA spec, first-class Wireshark `packet-enip` + `packet-cip` dissectors as a de-facto reference, existing Rust crates `rust-ethernet-ip` and `rseip-cip`, unencrypted on the wire, and public pcap corpora for TDD/holdout).

**S7comm (Siemens) is a very close #2** — equal attack share to EtherNet/IP in Forescout telemetry and arguably higher prevalence in *European* plants — but it is **proprietary/reverse-engineered** (no purchasable official wire-format spec), rides a more layered transport (TPKT + COTP / ISO-on-TCP), and its modern variant **S7comm-plus is encrypted**, capping a passive parser at metadata-only for newer traffic. This makes it slightly less tractable to implement *correctly and completely* in Rust than EtherNet/IP.

**Conflict flag:** PROFINET is marketed by its consortium as "the leading industrial Ethernet protocol," yet it does **not** appear in Forescout's top-5 attacked OT protocols and is largely absent from internet-exposure datasets (it lives on internal plant L2 segments). Marketing-claimed leadership conflicts with observed telemetry; PROFINET is deprioritized accordingly. BACnet has the largest raw *device* count (building automation, billions of units) and the largest *internet-exposed* ICS population (>half of Censys's 40k+ US ICS devices), but it serves the building-automation vertical rather than industrial process control — strategically orthogonal to wirerust's current Modbus/DNP3 industrial focus.

---

## Ranking Table

| Rank | Protocol | Deployment prevalence | Security relevance | Rust tractability | Verdict |
|------|----------|----------------------|--------------------|-------------------|---------|
| **#1** | **EtherNet/IP + CIP** | Dominant industrial-Ethernet in N. America (Rockwell ControlLogix/CompactLogix); ~40% of industrial-Ethernet market is N. America. Substantial internet exposure (part of Censys's ~18k US industrial devices); a focus protocol in academic OT-exposure studies alongside Modbus/IEC-104. | **~18% of all OT-protocol attacks** (Forescout 2023, tied #2). Dragos names CIP a growing adversary exploitation focus. Claroty Team82 shipped a dedicated ENIP/CIP stack-detector. CISA advisories on Rockwell/ENIP stacks. In MITRE ATT&CK for ICS. | **High.** ODVA formal spec (overview free, full spec paywalled). Wireshark `packet-enip` + `packet-cip` dissectors = de-facto open spec. Rust crates `rust-ethernet-ip` v1.1.0, `rseip-cip`. Unencrypted on 44818/2222. Public pcaps (automayt/ICS-pcap `ETHERNET_IP`/`EIP`). | **ADD NEXT** |
| #2 | **S7comm / S7comm-plus** (Siemens) | Pervasive wherever Siemens SIMATIC PLCs run (heavy in Europe/Asia, also global). Dominant in European manufacturing, chemical, infra. | **~18% of OT-protocol attacks** (Forescout "Step7", tied #2). Dragos names S7Comm a growing focus. Long history of Siemens-targeting ICS malware. In MITRE ATT&CK for ICS. | **Medium.** Proprietary — **no official wire-format spec** (reverse-engineered only). Wireshark `packet-s7comm` + SourceForge plugins as reference. TPKT+COTP/ISO-on-TCP layering (port 102). Classic S7comm (proto id 0x32) unencrypted; **S7comm-plus (0x72) is encrypted** → passive parser limited to metadata. No prominent Rust-native crate. Public pcaps (automayt/ICS-pcap `S7`). | Strong #2 |
| #3 | **IEC 60870-5-104** (IEC-104) | Backbone telecontrol in European electric transmission/distribution (SCADA↔RTU/IED). A focus protocol in academic OT-exposure studies + dedicated IDS datasets. | **~10% of OT-protocol attacks** (Forescout "IEC-10x", #4 category). Critical-grid impact. In MITRE ATT&CK for ICS. | **Medium-high.** IEC standard (paywalled). Wireshark `iec60870` dissector. Simple-ish TCP/2404 APDU framing. Public IDS datasets + pcaps. Vertical-narrow (power). | Power-vertical pick |
| #4 | **OPC UA** | Cross-vendor IT/OT integration layer (gateways/historians). Bitsight found **14,220 internet-exposed devices** across 99 countries (>51% unauthenticated). Globally distributed. | Heavy research interest: Team82 disclosed 20+ OPC UA vulns since 2020. In MITRE ATT&CK for ICS. Often integration-layer, not direct controller. | **Medium-low.** Open spec (OPC Foundation). Binary + secure-channel complexity; TLS/cert handshakes complicate passive parsing. Wireshark `opcua` dissector exists. | Integration-layer pick |
| #5 | **BACnet** | **Largest device count** (building automation; ~6.8B IoT building devices 2025). **>half of Censys's 40k+ US internet-exposed ICS** devices. | Fewer generic scans but more *targeted* exploits (Forescout); tail of OT attacks. In MITRE ATT&CK for ICS. | **Medium-high.** ASHRAE std. Wireshark `bacnet`/`bacapp` dissectors. UDP/47808. Public pcaps. | Building-automation pivot |
| (n/r) | PROFINET | Marketed as "leading"; Siemens-centric, internal L2. | **Absent** from Forescout top-5 + exposure datasets. | L2 real-time, cyclic; harder passive value. | **Conflict: marketing vs telemetry** |
| (n/r) | IEC 61850 (MMS/GOOSE/SV) | Intra-substation automation; growing but internal/segmented. | High-impact but low external visibility; in ATT&CK for ICS. | GOOSE/SV are L2 multicast → capture-point sensitive. | Power-substation niche |
| (n/r) | HART-IP | Niche; process-instrument device mgmt. No prevalence telemetry found. | Low in available telemetry. | Sparse data. | Lowest priority |

n/r = not recommended as the immediate next add.

---

## Key Quantitative Evidence (the load-bearing data point)

**Forescout 2023 Threat Roundup** — >420M recorded attacks (Jan–Dec 2023, ~13/sec), five protocols accounted for nearly all OT-protocol attacks:

- **Modbus** — ~33% (already supported by wirerust)
- **EtherNet/IP** — ~18%  ← **highest-value unsupported**
- **Step7 (Siemens S7)** — ~18%  ← tied
- **DNP3** — ~18% (already supported by wirerust)
- **IEC-10x (incl. IEC-104)** — ~10%
- Remainder (~2%) — mostly BACnet

Because wirerust already covers Modbus and DNP3, **EtherNet/IP and Step7 are the two highest-attack-share unsupported protocols, and they are tied.** The tie is broken by tractability and cross-vertical breadth, both favoring EtherNet/IP.

Source: https://www.forescout.com/press-releases/2023-threat-roundup/

---

## Per-Candidate Detail

### #1 — EtherNet/IP + CIP  (RECOMMENDED)

**Prevalence.** EtherNet/IP carries the Common Industrial Protocol (CIP) over standard TCP/IP + UDP and is the dominant industrial-Ethernet protocol in North America, anchored by Rockwell Automation's ControlLogix/CompactLogix install base. Industrial-Ethernet market analysis places ~40%+ of the market in North America with Rockwell a top-tier player. Censys's US ICS-exposure work counts ~18,000 internet-exposed *industrial* (non-building) devices identified via automation-protocol scans including EtherNet/IP and DNP3. The academic study "Measuring What Matters: Revisiting Internet Exposure of OT Networks" selects EtherNet/IP as one of just four focus protocols (with Modbus, Fox, IEC-104), underscoring measurable real-world exposure.

**Security relevance.** ~18% of OT-protocol attacks (Forescout 2023, tied #2). Dragos's OT Year-in-Review names CIP among standard ICS protocols seeing growing adversary exploitation (alongside Modbus, OPC UA, S7Comm). Claroty Team82 invested in and released a free **generic EtherNet/IP stack detector** — a strong signal the protocol is prevalent and security-critical enough to warrant bespoke tooling. CISA ICS advisories regularly cover Rockwell/ENIP/CIP-stack issues; the protocol is represented in MITRE ATT&CK for ICS.

**Rust tractability — strongest of all candidates.**
- **Spec:** ODVA. High-level/architecture docs free (technology overview, "CIP and the Family of CIP Networks" white paper PUB00123R1); full normative wire-format spec is paywalled/membership but exists as a definitive reference.
- **Reference dissectors:** Wireshark `packet-enip.c` (ports `ENIP_ENCAP_PORT=44818`, `ENIP_IO_PORT=2222`, `ENIP_SECURE_PORT=2221`; Common Packet Format item-type IDs) and `packet-cip.c`/`.h` (service codes, path segments, request/response). These are a de-facto open spec, translatable to Rust.
- **Existing Rust crates:** `rust-ethernet-ip` (v1.1.0, updated 2026-06-19 — client lib for AB CompactLogix/ControlLogix) and `rseip-cip` (CIP for the rseip project, MIT). Both are client libraries, not passive sniffers, but their encode/decode logic is directly reusable for a passive parser. **Existence proves EtherNet/IP+CIP is already tractable in Rust.**
- **Wire format:** Encapsulation header (command/length/session-handle/status/context/options) + CPF item list on TCP 44818 (explicit messaging) and UDP 2222 (implicit/cyclic I/O). Moderate, regular complexity — richer object model than Modbus but consistent encoding. **Unencrypted** on 44818/2222 (only the optional 2221 TLS/DTLS channel is encrypted), so a passive parser gets full semantic visibility.
- **PCAP corpus:** `automayt/ICS-pcap` has `ETHERNET_IP` and `EIP` folders; Wireshark SampleCaptures and ITI ICS-Security-Tools historically carry ENIP/CIP captures. Sufficient for TDD red/green and a holdout corpus.

### #2 — Siemens S7comm / S7comm-plus

**Prevalence.** Pervasive wherever Siemens SIMATIC PLCs are deployed; especially strong in European and Asian manufacturing, chemical, and infrastructure plants. ~18% of OT-protocol attacks (Forescout "Step7", tied with EtherNet/IP); Dragos names S7Comm a growing exploitation focus.

**Why #2, not #1 — tractability gap:**
- **No official spec.** Siemens publishes only configuration-level docs (TIA Portal: PUT/GET/BSEND/BRCV, transport selection). The wire format is **reverse-engineered** (gmiru.com S7comm writeup; Black Hat EU-17 S7CommPlus paper; Wireshark `packet-s7comm`; SourceForge `s7commwireshark`). De-facto spec = code, raising correctness risk.
- **More layered transport.** S7 PDUs ride TPKT + ISO-COTP (ISO 8073, RFC 1006/2126) over TCP **102** — the parser must first decode TPKT/COTP, then the proprietary S7 PDU (header/params/data; proto id 0x32).
- **S7comm-plus is encrypted.** Modern variant (proto id 0x72) wraps payloads in encryption a passive observer cannot decrypt without keys → metadata-only. Classic S7comm (0x32) remains unencrypted and fully parseable.
- **No prominent Rust-native crate** (unlike EtherNet/IP). FFI to C `snap7` is the usual route, but that's a client lib, not a passive parser, and not Rust-native.
- **PCAPs:** available (`automayt/ICS-pcap` `S7` folder, Wireshark samples).

**Recommendation:** schedule S7comm immediately *after* EtherNet/IP. The TPKT/COTP decode layer is reusable for other ISO-on-TCP protocols.

### #3 — IEC 60870-5-104

Electric-transmission telecontrol backbone (esp. Europe). ~10% of OT-protocol attacks (Forescout IEC-10x). Simple APDU framing over TCP/2404; Wireshark `iec60870` dissector; dedicated IDS datasets/pcaps. **Vertical-narrow (power)** — high value where wirerust targets utilities, lower cross-vertical breadth than EtherNet/IP. Complements DNP3 (DNP3 dominant in N. America, IEC-104 in Europe).

### #4 — OPC UA

Cross-vendor IT/OT integration layer. Bitsight: **14,220 internet-exposed devices, 99 countries, >51% unauthenticated** (scan through Jun 2025). Team82 disclosed 20+ vulns since 2020. **Tractability medium-low:** secure-channel/cert handshakes and binary encoding complicate passive parsing; often an integration layer rather than direct controller traffic. Strong future candidate, not the immediate next.

### #5 — BACnet

**Largest raw footprint** by device count (~6.8B IoT building devices, 2025) and by internet exposure (>half of Censys's 40k+ US ICS devices). But it serves **building automation**, not industrial process control — a strategic pivot away from wirerust's Modbus/DNP3 industrial focus. Tractable (ASHRAE spec, Wireshark `bacnet` dissector, UDP/47808, public pcaps) and worth adding *if* wirerust expands into facility/BAS monitoring.

### Deprioritized / conflict-flagged

- **PROFINET — conflict flag.** Consortium markets it as "the leading industrial Ethernet protocol," but it is **absent from Forescout's top-5 attacked protocols and from internet-exposure datasets** (it lives on internal real-time L2 plant segments, rarely IP-routed/exposed). Marketing-claimed leadership conflicts with observed attack/exposure telemetry. Lower passive-monitoring value at typical capture points.
- **IEC 61850 (MMS/GOOSE/SV).** High substation impact but GOOSE/SV are L2 multicast → capture-point-sensitive and internal/segmented; MMS is TCP but niche to substations. Power-substation niche, not a broad next add.
- **HART-IP.** Niche process-instrument device management. **No prevalence telemetry found in any source** (inconclusive) → lowest priority.

---

## Inconclusive / Conflicting Findings (explicit flags)

1. **HART-IP prevalence: INCONCLUSIVE.** No Shodan/Censys/vendor telemetry for HART-IP specifically surfaced. Reasoned (not source-backed) inference: niche, device-mgmt-scoped, smaller than controller/telecontrol protocols.
2. **PROFINET: CONFLICT.** Vendor "leading protocol" marketing vs. its absence from attack/exposure telemetry. Resolved by weighting observed telemetry over marketing → deprioritized.
3. **Exact per-protocol exposure counts** for EtherNet/IP, S7comm, IEC-104 are **not individually broken out** in the public summaries of Censys/academic studies (they appear in aggregate "industrial automation protocol" categories or as named focus protocols without per-protocol counts). Prevalence ranking therefore leans on **Forescout's per-protocol attack-share breakdown** (the cleanest quantitative split found) cross-checked against Dragos qualitative focus and Claroty tooling investment.
4. **S7comm-plus encryption** caps passive deep-parse value for *newer* Siemens traffic; classic S7comm remains fully parseable. Scope any S7 story to classic-S7comm semantics + S7comm-plus metadata.

---

## Recommendation & Suggested Next Step for the Factory

**Add EtherNet/IP + CIP as the next ICS analyzer (`src/analyzer/enip.rs`).** It is the highest combined-score unsupported protocol: tied-#1 attack share among unsupported protocols, the broadest cross-vertical industrial install base, fully unencrypted on its primary ports, and the most tractable in Rust (formal spec + Wireshark reference dissectors + existing Rust crates + public pcap corpus).

**Suggested factory action — feature-mode cycle "EtherNet/IP + CIP analyzer":**

1. **Scope (MVP first):** explicit-messaging path on **TCP/44818** — encapsulation header parse, CPF item iteration, CIP service-code/path decode (List Identity, Register Session, Send RR/Unit Data, Get/Set_Attribute). Defer implicit cyclic I/O on **UDP/2222** (requires assembly-object semantics) and the TLS/DTLS 2221 channel (encrypted) to a follow-on story. This mirrors how the existing `modbus.rs`/`dnp3.rs` analyzers are structured under `src/analyzer/`.
2. **Reference spec for the parser:** Wireshark `packet-enip.c` + `packet-cip.c` (de-facto open spec); ODVA PUB00123R1 white paper for object-model context. Verify any field layout against captured pcaps, not training data.
3. **TDD + holdout corpus:** pull EtherNet/IP/CIP pcaps from `automayt/ICS-pcap` (`ETHERNET_IP`, `EIP`) and Wireshark SampleCaptures; split a holdout set for Phase-4 evaluation (do not let the implementing agent see holdout captures).
4. **MITRE ATT&CK for ICS mapping:** wire findings to the ICS techniques EtherNet/IP/CIP participate in (consistent with the existing Modbus/DNP3 attribution approach in `src/mitre.rs`).
5. **Sequence S7comm next** — reuse a TPKT/COTP decode layer; scope to classic S7comm (proto id 0x32) full-decode + S7comm-plus (0x72) metadata-only.

**Per CLAUDE.md `DF-VALIDATION-001`:** this is research, not a filed finding. If any of the above is converted into a backlog/GitHub issue, it must pass research-agent validation (this document satisfies that for the protocol-selection decision).

---

## Sources

Prevalence & threat telemetry:
- Forescout 2023 Threat Roundup — https://www.forescout.com/press-releases/2023-threat-roundup/  (per-protocol OT attack-share breakdown — primary quantitative source)
- Dragos OT Cybersecurity Year in Review — https://www.dragos.com/ot-cybersecurity-year-in-review and https://www.dragos.com/blog/dragos-8th-annual-ot-cybersecurity-year-in-review-is-now-available
- Claroty "State of CPS Security: OT Exposures 2025" — https://claroty.com/resources/reports/state-of-cps-security-ot-exposures-2025 ; Team82 — https://claroty.com/team82 ; ENIP/CIP stack detector — https://www.claroty.com/team82/research/team82-enip-cip-stack-detector-simplifies-protocol-identification
- Nozomi OT/IoT Security Reports — https://www.nozominetworks.com/resources/iot-ot-cybersecurity-research-report-february-2024 ; https://www.nozominetworks.com/resources/iot-ot-cybersecurity-research-report-august-2023
- Censys US ICS exposure (via Industrial Cyber) — https://industrialcyber.co/industrial-cyber-attacks/censys-reveals-over-40000-vulnerable-ics-devices-in-us-marking-security-risks-in-building-and-water-system
- Shodan internet-exposure trends — http://blog.shodan.io/trends-in-internet-exposure/
- "Measuring What Matters: Revisiting Internet Exposure of OT Networks" — https://papers.ssrn.com/sol3/papers.cfm?abstract_id=5974783
- Smart-grid IDS / IEC-104 dataset survey — https://www.sciencedirect.com/science/article/pii/S1574013726000249 ; https://www.sciencedirect.com/science/article/pii/S0167404826000878
- OPC UA security (FlowFuse, citing Team82 + Bitsight TRACE: 14,220 exposed) — https://flowfuse.com/blog/2026/05/opc-ua-security-attack-vectors/
- Industrial Ethernet market — https://www.marketresearchfuture.com/reports/industrial-ethernet-market-4829
- BACnet / building-automation install base — https://dataintelo.com/report/bacnet-secure-connect-deployment-market
- PROFINET (vendor "leading protocol" claim) — https://www.profinet.com

Implementation tractability (specs, dissectors, crates, pcaps):
- ODVA EtherNet/IP — https://www.odva.org/technology-standards/key-technologies/ethernet-ip/ ; CIP — https://www.odva.org/technology-standards/key-technologies/common-industrial-protocol-cip/ ; CIP & Family of CIP Networks white paper (PUB00123R1) — https://www.odva.org/wp-content/uploads/2020/06/PUB00123R1_Common-Industrial_Protocol_and_Family_of_CIP_Networks.pdf ; document library — https://www.odva.org/technology-standards/document-library/
- Wireshark dissectors — ENIP: https://www.wireshark.org/docs/dfref/e/enip.html and https://github.com/wireshark/wireshark/blob/master/epan/dissectors/packet-enip.c ; CIP: https://github.com/wireshark/wireshark/blob/master/epan/dissectors/packet-cip.c and https://www.wireshark.org/docs/wsar_html/packet-cip_8h_source.html ; S7comm: https://www.wireshark.org/docs/wsar_html/packet-s7comm_8h_source.html
- Rust crates — https://crates.io/crates/rust-ethernet-ip (v1.1.0, updated 2026-06-19) ; https://crates.io/crates/rseip-cip
- S7comm reverse-engineering — gmiru S7comm — http://gmiru.com/article/s7comm/ ; Black Hat EU-17 S7CommPlus — https://blackhat.com/docs/eu-17/materials/eu-17-Lei-The-Spear-To-Break%20-The-Security-Wall-Of-S7CommPlus-wp.pdf ; SourceForge S7comm Wireshark plugin — https://sourceforge.net/projects/s7commwireshark/ ; S7COMMM-Plus — https://github.com/QingChenHT/S7COMMM-Plus ; S7comm vs S7comm-plus (0x32 vs 0x72) — https://industrialmonitordirect.com/blogs/knowledgebase/s7comm-vs-s7comm-plus-switching-protocol-in-s7-1500-hmi-communication
- Siemens TIA Portal S7-comm config docs — https://docs.tia.siemens.cloud/r/simatic_et_200clean_manual_collection_zhcn_20/function-manuals/communication-function-manuals/communication/s7-communication/
- CIP technical reference — https://industrialmonitordirect.com/blogs/knowledgebase/common-industrial-protocol-cip-complete-technical-reference
- PCAP corpus — automayt/ICS-pcap (ETHERNET_IP, EIP, S7 folders) — https://github.com/automayt/ICS-pcap ; Wireshark SampleCaptures — https://wiki.wireshark.org/SampleCaptures

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Deep multi-source prevalence/security sweep across 8 ICS protocols (Dragos/Nozomi/Claroty/Forescout/Censys/Shodan/academic). (2) Deep tractability sweep: EtherNet/IP+CIP vs S7comm Rust parseability (specs, Wireshark dissectors, crates, pcaps). Both `reasoning_effort=high`/`medium`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — |
| Tavily | 0 | (tools not invoked) |
| WebFetch | 3 | Verify `rust-ethernet-ip` crate version (v1.1.0, 2026-06-19) on crates.io; check Wireshark SampleCaptures for ENIP/S7 pcaps; confirm automayt/ICS-pcap has ENIP/EIP/S7 folders. |
| WebSearch | 0 | — |
| Glob/Grep/Read (local) | 5 | Inspect wirerust analyzer layout (`src/analyzer/*.rs`), prior research index, and extract the large saved Perplexity JSON outputs. |
| Training data | 1 area | General ICS protocol-layering background (TPKT/COTP, CIP object model) — cross-checked against sourced findings; not used for version numbers or prevalence figures. |

**Total MCP tool calls:** 2 (both `perplexity_research`, PRIMARY) + 3 WebFetch verification = 5 external calls.
**Training data reliance:** low — all prevalence figures, attack-share percentages, crate versions, and tractability claims are sourced to URLs; training data used only for generic protocol-layering background that is independently corroborated by cited dissector docs.
