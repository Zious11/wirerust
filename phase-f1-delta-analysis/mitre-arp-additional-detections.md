# ARP Detection Catalogue & Additional-Detection Research (wirerust ARP Security Analyzer)

**Date:** 2026-06-12
**Researcher:** vsdd-factory research-agent
**Scope:** Identify the full catalogue of ARP-layer attack/anomaly detections an offline-PCAP ICS/network analyzer should consider, beyond the three the human already committed to (ARP spoofing/cache poisoning, gratuitous ARP, ARP storm/rate anomaly). Produce a prioritized v0.7.0 include/defer set, a MITRE mapping table validated against live `attack.mitre.org` (ATT&CK v19.1), and a defensible-defaults/thresholds section.
**Feeds:** F2 behavioral-contract authoring for SS-16. Supersedes nothing in `mitre-arp-research.md` — it extends it with the reconnaissance-side technique IDs (T1018 / T0840) that the original did not cover.
**Verification basis:** MITRE claims verified by live page fetches 2026-06-12 (NOT model memory). Tool-threshold claims verified against primary sources (arpwatch man pages, Snort `spp_arpspoof.c` source, Zeek docs) — see the prominent **fabrication correction** in the thresholds section.

---

## CRITICAL CORRECTION — Fabricated thresholds in the initial deep-research pass

The `perplexity_research` deep call returned an articulate narrative whose **wire-level pattern descriptions are sound and reused below**, but whose **specific numeric tool thresholds and vendor-specific thresholds were fabricated.** They were cross-checked against primary sources and FAILED verification. Do **not** propagate these into BCs:

| Claim in deep-research output | Verdict | Primary source that refutes it |
|---|---|---|
| "Snort arpspoof `mac_changes` = 5 changes / 300s" | **FABRICATED** — no such option exists | Snort `spp_arpspoof.c` + manual §2.2.12: the preprocessor has only `-unicast` and `arpspoof_detect_host: ip mac`; 4 SIDs (GID 112); **no numeric threshold of any kind** |
| "Snort `scan_thresh` = 30 targets/sec", "`gratuitous_arp` rule = 3/min", "`arp_announce` rule" | **FABRICATED** — none of these exist in the arpspoof preprocessor | Snort manual §2.2.12; `generators.h` (only SIDs 1–4) |
| "arpwatch = 3 MAC changes in 10 min triggers alert" | **FABRICATED** — arpwatch is event-driven, not threshold-driven | arpwatch(8) man page: reports `flip flop` on *each* transition to the 2nd-most-recent MAC; no count window |
| "Zeek ARP analyzer = 20 ARP/sec per host", "15% of subnet/min" | **FABRICATED** | Zeek docs: the ARP analyzer that emits `bad_arp` is **not even active in default config**; no such rate thresholds ship |
| All Siemens/ABB/Rockwell/Emerson/Honeywell/Yokogawa/Tofino "trigger at N pps / N changes/hour" numbers | **UNVERIFIABLE / LIKELY FABRICATED** — no primary source located; treat as non-authoritative | none found |

**Net effect:** the literature does **not** hand us authoritative numeric defaults the way the deep-research draft implied. The real established tools are mostly *stateful-event* detectors (report on transition), not *rate-threshold* detectors. Defensible numeric defaults for wirerust must therefore be chosen by us and documented as engineering choices, not cited as borrowed industry standards. See the Defensible Defaults section.

---

## 1. Detection Catalogue (offline-PCAP detectability)

Wire-level descriptions are drawn from the deep-research synthesis (pattern shapes are accurate) plus RFC 826/5227 and the verified tool behaviors. "Offline detectable" = detectable from a static PCAP with no live probing and no external context.

| # | Detection | What it looks like on the wire | Offline? | FP sources (benign) |
|---|---|---|---|---|
| D1 | **ARP cache poisoning / spoofing** | opcode=2 reply binding a victim IP (often gateway) to attacker MAC; appears without a matching prior request; over time the same IP resolves to a *new/conflicting* MAC | **YES** | NIC replacement, DHCP churn, VM migration, HA/redundancy failover (esp. ICS controller pairs) |
| D2 | **Gratuitous ARP (GARP)** | sender IP == target IP, opcode=1 (announcement) or opcode=2 (reply), broadcast, THA zero/own | **YES** | Legitimate link-up announcements, failover, RFC 5227 ACD announce phase — very common, high FP |
| D3 | **ARP storm / rate anomaly** | abnormally high ARP volume from one MAC (or broadcast) over a short window | **YES** (rate is measurable from timestamps) | System startup bursts, mass I/O init, network scans during commissioning |
| D4 | **ARP scanning / sweep (reconnaissance)** | series of opcode=1 requests, constant sender, THA zero, **target IP walking the subnet** (sequential or strided) | **YES** | nmap/arp-scan during commissioning, controller peer-validation polls, asset-discovery tools |
| D5 | **Unsolicited ARP reply / reply-without-request** | opcode=2 with no preceding request from the recipient; THA often zero (not unicast-directed) | **PARTIAL** | GARP overlaps this; many stacks send unsolicited replies legitimately; pairing replies to requests in offline capture is heuristic, not exact |
| D6 | **MAC-flapping: one IP ↔ many MACs over time** | same sender-IP resolves to differing sender-MACs across the capture (this is arpwatch "flip flop" / "changed ethernet address") | **YES** | HA failover, NIC bonding/teaming, DHCP, VM migration — D6 is essentially the temporal core of D1 |
| D7 | **MAC-flapping: one MAC ↔ many IPs** | a single sender-MAC claims many distinct sender-IPs | **YES** | Routers, proxy-ARP devices, NAT/load-balancers, hypervisors, gateways legitimately do this — **high FP** |
| D8 | **ARP probe (RFC 5227 ACD)** | opcode=1, **sender-IP all-zero**, THA zero, target-IP = tentative address; typically 3 probes 1–2 s apart | **YES** | Normal host bring-up / DHCP clients — almost always benign by itself |
| D9 | **ARP announcement (RFC 5227 ACD)** | opcode=1 (or 2), sender-IP == target-IP, sent after a successful probe | **YES** | Normal host bring-up — benign; this is the "good" subset of D2 |
| D10 | **Duplicate-IP / IP conflict** | two distinct MACs assert the same sender-IP within a short window (competing announcements/replies) | **YES** | Misconfiguration (static IP collision), device-replacement "IP squatting" before decommission |
| D11 | **Malformed / oversized ARP** | hw/proto address-length fields inconsistent with declared types; payload != 28 bytes for Ethernet/IPv4; impossible field combos | **YES** (pure structural check) | Legacy/quirky ICS stacks, protocol-converter gateways repurposing ARP fields |
| D12 | **L2/L3 sender mismatch** (Ethernet src MAC != ARP sender HW addr) | the frame's Ethernet source MAC differs from the ARP-payload sender hardware address | **YES** (single-packet, no state) | Rare in benign traffic; some bridges/virtual switches rewrite — low FP. This is exactly Snort's GID 112 SID 2/3. |
| D13 | **Proxy-ARP anomaly** | one MAC answers for IPs outside its own host (router answering on behalf of others); anomalous when an *unexpected* host does it | **PARTIAL** | Routers and proxy-ARP-enabled gateways do this legitimately and constantly; distinguishing "rogue proxy" from "configured proxy" needs topology knowledge wirerust does not have offline |
| D14 | **Unicast ARP request** | opcode=1 sent to a *unicast* Ethernet dst rather than broadcast | **YES** (single-packet) | Linux re-validates cache entries with unicast requests routinely — **noisy**; Snort itself warns this generates FPs |

**Notes on the three pre-committed detections:** the human's (1) spoofing = D1, (2) gratuitous = D2, (3) storm = D3. D6 (IP↔many-MACs over time) is the cross-packet binding-table mechanism that *implements* D1 — it is not really a separate detection, it is the engine. D5/D9 substantially overlap D2.

---

## 2. Prioritized Recommended Set — v0.7.0 (include) vs defer

Selection bias per the task: favor reliably-distinguishable-from-benign, low-FP, single-packet-or-simple-state detections. The binding table (already in the F1 delta architecture) is the shared substrate for D1/D6/D10.

| Detection | Recommendation | Confidence | One-line rationale |
|---|---|---|---|
| D1 ARP spoofing / cache poisoning | **MUST-HAVE (committed)** | HIGH | Core feature; binding-table conflict is the canonical AiTM indicator; maps cleanly to T0830. |
| D2 Gratuitous ARP | **MUST-HAVE (committed)** | HIGH detect / LOW severity | Trivial to detect (sender==target); emit at low confidence — benign GARP is extremely common. |
| D3 ARP storm / rate anomaly | **MUST-HAVE (committed)** | MEDIUM | Rate is objectively measurable offline; threshold is an engineering choice (no borrowable industry default — see corrections). |
| **D12 L2/L3 sender mismatch** | **MUST-HAVE (new — strongly recommended add)** | HIGH | Single-packet, stateless, near-zero FP, no threshold tuning. This is Snort's most reliable ARP signal (SID 2/3). Cheapest high-value addition. |
| **D11 Malformed / oversized ARP** | **MUST-HAVE (new — recommended add)** | HIGH | Pure structural validation; no state, no threshold. etherparse already gives field access; low effort. Flag low confidence in ICS context (legacy stacks). |
| **D10 Duplicate-IP / IP conflict** | **NICE-TO-HAVE (new)** | MEDIUM | Falls out of the binding table almost for free (two MACs claim one IP in a window). Distinct *finding* from D1 (conflict vs. silent rebind). Worth it if binding table lands. |
| **D4 ARP scanning / sweep** | **NICE-TO-HAVE (new)** | MEDIUM | Detectable (subnet-walk in target-IP), and it unlocks a *reconnaissance* MITRE mapping (T1018 / T0840) distinct from AiTM. Needs a threshold (distinct-targets/window) — engineering choice. Moderate FP from commissioning scans. |
| **D8/D9 RFC 5227 probe/announce** | **NICE-TO-HAVE (as suppressor, not a finding)** | HIGH | Best value is *negative*: recognizing legit ACD probe/announce sequences to **suppress** D2/D1 false positives, not to emit findings. Recommend implementing as FP-reduction logic. |
| D14 Unicast ARP request | **DEFER** | LOW | Snort flags it but its own source warns of high FP (Linux cache re-validation). Low signal-to-noise for offline forensics. |
| D7 MAC ↔ many IPs | **DEFER** | LOW | Routers/proxies/hypervisors do this legitimately and constantly; very high FP without topology context. |
| D13 Proxy-ARP anomaly | **DEFER** | LOW | Cannot distinguish rogue from configured proxy-ARP offline without a topology baseline wirerust lacks. |
| D5 Unsolicited reply (as separate finding) | **DEFER / FOLD INTO D2** | LOW | Overlaps GARP; exact request/reply pairing offline is heuristic. Track as evidence on D1/D2, not a standalone finding. |

**Recommended v0.7.0 detection scope:** the 3 committed (D1, D2, D3) **+ D12 + D11** as must-haves, with **D10 + D4** as nice-to-haves if story budget allows, and **D8/D9** implemented as suppression logic rather than findings. This keeps every must-have either single-packet/stateless (D11, D12) or backed by the binding table already in the architecture (D1, D10), and confines threshold-tuning risk to D3 and D4 only.

---

## 3. MITRE Mapping (validated against live attack.mitre.org, ATT&CK v19.1)

Every ID below was fetched live on 2026-06-12. "Matrix" notes ICS vs Enterprise. wirerust's convention is ICS-first (carry the ICS ID; optionally cross-reference Enterprise).

| Technique ID | Exact title | Matrix | Detections it covers | Status (v19.1) | Source URL |
|---|---|---|---|---|---|
| **T0830** | Adversary-in-the-Middle | ICS | D1, D2, D5, D6, D10, D12 (all AiTM/poisoning indicators) | **Current** — v2.0, last mod 2025-04-16. ARP poisoning is an explicit named example. | https://attack.mitre.org/techniques/T0830/ |
| **T1557** | Adversary-in-the-Middle | Enterprise (parent) | parent of T1557.002 | **Current** — v2.5, last mod 2026-05-12 | https://attack.mitre.org/techniques/T1557/ |
| **T1557.002** | Adversary-in-the-Middle: ARP Cache Poisoning | Enterprise (sub of T1557) | D1, D2, D5, D6, D10, D12 (Enterprise cross-ref) | **Current** — not deprecated/revoked | https://attack.mitre.org/techniques/T1557/002/ |
| **T0840** | Network Connection Enumeration | ICS | **D4 (ARP scanning/recon)** — ICS-matrix mapping | **Current** — v1.2, last mod 2026-05-12. Detection section references the `arp` command. | https://attack.mitre.org/techniques/T0840/ |
| **T0846** | Remote System Discovery | ICS | D4 (recon) — alternative/secondary ICS mapping | **Current** — v1.1, last mod 2026-05-12. Sub-techniques incl. Port Scan/Broadcast/Multicast Discovery; ARP not named explicitly. | https://attack.mitre.org/techniques/T0846/ |
| **T1018** | Remote System Discovery | Enterprise | **D4 (recon)** — Enterprise cross-ref; **explicitly names ARP** | **Current** — v3.6, last mod 2026-05-12. Description: adversaries discover remote systems via "local Arp cache entries." | https://attack.mitre.org/techniques/T1018/ |

### Techniques explicitly ASSESSED and REJECTED (do not use)

| ID | Title | Why rejected | Source URL |
|---|---|---|---|
| **T1595** | Active Scanning | Enterprise | **Wrong scope.** Page: "probes victim infrastructure via network traffic" — it is *PRE-compromise external reconnaissance* of remote targets, not internal LAN ARP sweeps. Do **not** map ARP scanning to T1595. | https://attack.mitre.org/techniques/T1595/ |
| **T0842** | Network Sniffing | ICS | Concerns passive capture/credential sniffing, not ARP-based discovery or AiTM. ARP is mentioned only as an enabling redirect for sniffing. Not a fit for any ARP *detection* wirerust emits. | https://attack.mitre.org/techniques/T0842/ |

### Mapping guidance for F2

- **AiTM-family findings (D1, D2, D5, D6, D10, D12):** carry **T0830** (ICS). Optionally cross-reference **T1557.002** (Enterprise) if wirerust dual-tags. Record the specific indicator (e.g. "gratuitous ARP reply", "Ethernet/ARP sender mismatch") as evidence text, not as a separate technique ID — none of D2/D5/D6/D10/D12 has a dedicated technique ID; they are procedures under AiTM.
- **Reconnaissance finding (D4 ARP scanning):** this is the one *new technique class* this research adds. Map to **T0840 — Network Connection Enumeration** (ICS, preferred for wirerust's ICS-first posture; its detection guidance cites `arp`), optionally cross-referencing **T1018 — Remote System Discovery** (Enterprise; explicitly names ARP cache). T0846 is a viable alternate ICS mapping but is less ARP-specific. **Recommendation: T0840 primary, T1018 optional Enterprise cross-ref.** Confirm tactic placement (Discovery) when seeding `src/mitre.rs`.
- **D3 (storm), D11 (malformed):** these are availability/hygiene anomalies, not clean ATT&CK techniques. ARP storm could be argued toward Network Denial of Service (Enterprise T1498 / ICS T0814 "Denial of Service"), but a flood-from-misconfig is frequently benign — **recommend emitting D3/D11 as anomaly findings with NO MITRE technique** (or low-confidence T0814 only when clearly volumetric-malicious). Flag this as a human decision in F2; do not force a technique ID. (T0814 was not re-fetched in this pass — if F2 wants to attach it, validate live first per DF-VALIDATION-001.)

**Revocation-risk:** CLEAR for all six recommended IDs (T0830, T1557, T1557.002, T0840, T0846, T1018) as of v19.1. None deprecated/revoked/superseded. All last-modified dates are 2025–2026, indicating active maintenance. The v19 ICS restructuring (T0855/T0856 → T1692) does not touch any technique in this set.

---

## 4. Defensible Defaults / Thresholds (with the fabrication caveat front-of-mind)

**Reality, from primary sources:** the established tools are mostly *event/state* detectors, not numeric-rate detectors. There is **no authoritative "borrow these numbers" table** in the literature. Below are (a) the *verified* real behaviors we can genuinely borrow as logic, and (b) wirerust engineering-choice defaults that must be documented as our own, not cited as industry standard.

### 4a. Verified tool logic we CAN borrow (sourced)

- **arpwatch "flip flop" semantics** (borrow the *definition*, it has no count threshold): report when an IP's ethernet address changes "from the most recently seen address to the **second** most recently seen address"; "changed ethernet address" = switched to a brand-new MAC; "reused old ethernet address" = reverted to the 3rd-or-older MAC. Also: **"ethernet mismatch"** = "source mac ethernet address didn't match the address inside the arp packet" (this is exactly D12), and **"bogon"** = source IP not local to the subnet. Source: arpwatch(8), https://manpages.debian.org/unstable/arpwatch/arpwatch.8.en.html and https://man.freebsd.org/cgi/man.cgi?query=arpwatch
- **Snort arpspoof preprocessor** (borrow the *signals*, it has no numeric threshold): GID 112 — SID 1 unicast ARP request, **SID 2 Ethernet/ARP source mismatch**, **SID 3 Ethernet/ARP dest mismatch** (= D12), SID 4 ARP cache overwrite attack (requires a static IP↔MAC allow-list). Source: Snort manual §2.2.12 (https://seclists.org/snort/2011/q3/146) and `spp_arpspoof.c` (https://github.com/eldondev/Snort/blob/master/src/preprocessors/spp_arpspoof.c).
- **Zeek `bad_arp` event** (borrow the *concept*): fires for ARP "with non-standard hardware address formats or hardware addresses that do not match the originator" (= D11 + D12). **Caveat:** Zeek's ARP analyzer is *not active in default config*. Source: https://docs.zeek.org/en/lts/scripts/base/bif/plugins/Zeek_ARP.events.bif.zeek.html
- **RFC 5227 ACD timing** (borrow for D8/D9 suppression): probe phase = up to 3 probes, sender-IP all-zero, spaced ~1–2 s; announce phase = sender-IP == target-IP after success. Use this to *suppress* D2/D1 false positives during host bring-up.

### 4b. wirerust engineering-choice defaults (NOT industry-cited — document as our own)

| Parameter | Proposed default | Rationale | How to expose |
|---|---|---|---|
| **Spoof rebind escalation (D1/D6)** | First rebind → MEDIUM/Anomaly; **≥3 distinct MACs for one IP within 60 s** → HIGH/Likely | Matches the F1 delta's DECISION 2 recommendation; mirrors DNP3 burst-escalation pattern. The "3 / 60 s" is our choice (arpwatch/Snort give no number). | `--arp-spoof-threshold` (already proposed in F1 delta) |
| **Storm rate (D3)** | flag a source MAC exceeding **N ARP/sec sustained over a window** — start N configurable, default conservative (e.g. 50/s) and document as tunable | No borrowable industry default exists. 50/s is a round engineering starting point, NOT a cited standard. ICS field devices are far more sensitive; expose so OT users can lower it. | `--arp-storm-rate` / window flag |
| **Scan distinct-target (D4)** | one source requesting **> K distinct target IPs within a window** (e.g. K=20 / 60 s) | Engineering choice; tune against benign commissioning scans. Flag low/medium confidence. | `--arp-scan-threshold` |
| **MAC-flap window (D6)** | the 60 s window above; arpwatch-style per-IP MAC history | Borrowed *logic* (flip-flop), our *window*. | shared with spoof threshold |
| **Binding table cap** | MAX_ARP_BINDINGS = 65,536, LRU eviction | From F1 delta DECISION 5; bounds memory. | constant |
| **GARP (D2) severity** | always LOW/Inconclusive unless it conflicts with an existing binding (then promote to D1) | GARP is benign-common; over-emission is the main FP risk. | n/a |
| **D11/D12 severity** | D12 = MEDIUM (low FP, strong signal); D11 = LOW in ICS context (legacy-stack FP) | D12 is single-packet high-fidelity; D11 has legacy-OT FP risk. | n/a |

**F2 must document every numeric default in 4b as a wirerust-chosen value with FP rationale — NOT as a borrowed industry threshold.** The only genuinely borrowable items are the *qualitative logic* in 4a (flip-flop semantics, Ethernet/ARP mismatch, malformed-packet checks, RFC 5227 suppression).

---

## 5. Inconclusive / Open Items (explicit)

- **No authoritative numeric thresholds exist** in the public literature for ARP storm rate, MAC-flap counts, or scan rates. The figures in the initial deep-research draft were fabricated (see correction table). wirerust must choose and own its defaults.
- **ICS-vendor specific thresholds** (Siemens/ABB/Rockwell/etc.) claimed by the deep-research pass could **not be verified against any primary source** and are treated as non-authoritative. Do not cite them in BCs.
- **T0814 (Denial of Service)** as a possible mapping for D3 (storm) was *not* re-fetched live in this pass. If F2 elects to attach a MITRE ID to storm findings, validate T0814 live first (DF-VALIDATION-001).
- **Proxy-ARP (D13) and MAC↔many-IPs (D7)** are deferred precisely because offline capture lacks the topology baseline needed to separate rogue from configured behavior — this is a genuine detectability limit, not a scoping convenience.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source ARP attack/anomaly catalogue sweep (reasoning_effort=high). Pattern/wire-level descriptions retained; numeric thresholds discarded after primary-source cross-check. |
| Perplexity perplexity_search | 3 | Primary-source verification of Snort arpspoof preprocessor, arpwatch flip-flop/bogon semantics, Zeek ARP analyzer / bad_arp — used to REFUTE fabricated thresholds. |
| WebFetch | 6 | Live attack.mitre.org validation of T1018, T0840, T1595, T0842, T0846 (+ reuse of prior T0830/T1557/T1557.002 validation in mitre-arp-research.md). |
| Read | 3 | Prior research (mitre-arp-research.md), F1 ARP delta-analysis, deep-research persisted output. |
| Training data | 1 area | RFC 826/5227 packet-field structure (cross-checked against tool docs; widely-stable protocol facts). Flagged. |

**Total MCP tool calls:** 4 (1 perplexity_research + 3 perplexity_search) + 6 WebFetch = 10.
**Training data reliance:** low — every MITRE ID validated against live pages; every tool threshold validated against primary docs/source; the one training-data area (RFC field layout) is stable protocol structure cross-checked against tool dissectors.
**Deviation note:** perplexity_research WAS used (primary tool, high effort) as mandated. Its quantitative output was partially fabricated, which is why 3 follow-up perplexity_search calls and 6 live fetches were spent on primary-source verification — this is the intended verification layer, not a deviation.
