# Gratuitous ARP -> MITRE ATT&CK Attribution (T0830 / T1557.002)

**Research date:** 2026-06-14
**Question:** Should a passively-observed gratuitous ARP (GARP; sender protocol address == target protocol address) be attributed to MITRE T0830 (AiTM, ICS) and T1557.002 (AiTM: ARP Cache Poisoning, Enterprise) for ALL gratuitous ARP, or ONLY for the conflicting case (a GARP claiming an IP already observed bound to a different MAC)?
**Scope:** Research only. No source files modified.

---

## TL;DR Recommendation

**Reserve T0830 and T1557.002 for the CONFLICTING case (B).** Do NOT attach AiTM / ARP-cache-poisoning technique IDs to benign, non-conflicting gratuitous ARP (case A).

This is what the MITRE technique pages actually describe (the *malicious announcement* / *override* of an IP-MAC binding, not the GARP mechanism itself), and it is the universal convention across Suricata, Zeek, Snort, and arpwatch, all of which alert on *binding conflict*, not on gratuitous ARP per se. Tagging every GARP with an AiTM technique at LOW severity is a false-positive generator with no detection value, because legitimate GARP is high-volume routine traffic (interface up, DHCP/DAD, failover, VM migration).

Acceptable middle ground: emit a non-attributed, informational observation for benign GARP (e.g. an `info`/`debug` record "gratuitous ARP observed: IP<->MAC announced") **without** a MITRE technique tag. The technique tag is the load-bearing claim; that is what must be gated on conflict.

---

## Point 1 - MITRE T1557.002 (ARP Cache Poisoning, Enterprise)

**Authoritative page:** https://attack.mitre.org/techniques/T1557/002/
**Parent technique:** https://attack.mitre.org/techniques/T1557/
**Detection strategy:** https://attack.mitre.org/detectionstrategies/DET0387/
**Mitigation M1042:** https://attack.mitre.org/mitigations/M1042/

### What the technique describes
The technique is the *poisoning* of ARP caches with **false** IP-to-MAC bindings to achieve an adversary-in-the-middle position. The page explicitly mentions gratuitous ARP, but only in the *malicious* sense:

> "Adversaries may also send a gratuitous ARP reply that **maliciously announces the ownership of a particular IP address** to all the devices in the local network segment."
> (attack.mitre.org/techniques/T1557/002/)

The defining act is the false/overriding announcement, not the gratuitous mechanism. The page itself distinguishes this from legitimate gratuitous ARP where a device announces *its own* mapping. The race-condition variant ("their reply must be faster than the one made by the legitimate IP address owner") likewise depends on contradicting the true owner.

### Detection guidance (DET0387 / AN1091-AN1093)
All three analytics target *conflict*, not gratuitous ARP in general (verified directly against the detection-strategy page):

- **AN1091** (Windows): "multiple IP addresses resolving to a single MAC, or unsolicited ARP replies from unauthorized devices."
- **AN1092** (Linux): "unsolicited replies **overriding legitimate ARP ownership**."
- **AN1093** (macOS): "multiple IP addresses mapped to the same MAC address" and ARP updates "**inconsistent with expected gateway or DHCP lease assignments**."

### Is benign GARP an instance/indicator of T1557.002?
**No.** Nothing on the page or in DET0387 frames a self-consistent gratuitous ARP (announcing one's own, non-conflicting binding) as an indicator. The indicators are all binding inconsistencies (one MAC owning many IPs, an IP's owner being overridden, mappings inconsistent with DHCP/gateway expectations). The M1042 mitigation ("disabling updating the ARP cache on gratuitous ARP replies") targets the *attack vector*, but its existence as a mitigation does not make every GARP an attack instance.

**Conclusion:** T1557.002 corresponds to case (B), the conflicting GARP. Benign GARP (case A) is not an instance.

---

## Point 2 - MITRE T0830 (Adversary-in-the-Middle, ICS)

**Authoritative page:** https://attack.mitre.org/techniques/T0830/
**Detection strategy referenced on page:** DET0764

### What the technique describes
> Adversaries with privileged network access "intercept traffic to and/or from a particular device" and can then "block, log, modify, or inject traffic into the communication stream."

ARP poisoning is named as one of the "most-common" methods (alongside a proxy). **Gratuitous ARP is NOT mentioned by name** on the T0830 page; the only ARP reference is "ARP poisoning" as an attack method and the detection note that "anomalies may be present in network management protocols (e.g., ARP, DHCP)."

### Detection guidance
T0830 detection emphasizes:
- "network traffic originating from unknown/unexpected hosts" (with attention to source MAC addressing),
- "anomalies associated with known AiTM behavior" in ARP/DHCP,
- newly-created services / proxy processes on engineering workstations,
- the foundational principle that detection requires "consideration of normal network behavior" to spot deviations.

### Is a lone benign GARP an instance/indicator of T0830?
**No.** T0830 is an *interception/manipulation* technique. The detection language is explicitly anomaly-based ("anomalies ... in ... ARP, DHCP", deviations from "normal network behavior"). A single self-consistent GARP from a host announcing its own binding is normal network behavior, not an AiTM anomaly. T0830 is also broader than ARP (proxy-based interception too), so it is even less appropriate to fire it on every GARP packet.

**Conclusion:** T0830 corresponds to case (B) at most (a conflicting/anomalous ARP that indicates interception positioning). Benign GARP (case A) is not an instance.

---

## Point 3 - Detection-engineering / IDS conventions (Suricata, Zeek, Snort, arpwatch)

**Universal convention: alert on IP-to-MAC BINDING CONFLICT, not on gratuitous ARP itself.**
Sources: tool documentation and detection-engineering literature synthesized via Perplexity deep research (sonar-deep-research), cross-checked against the MITRE pages above. Tool-specific config-parameter details below are from secondary synthesis and should be treated as directional rather than verbatim-from-manpage.

- **arpwatch** - the canonical dedicated tool. Builds a historical IP<->MAC database and alerts on *changes/conflicts* ("flip-flop", "changed ethernet address", "new activity"). Self-consistent GARP that matches the known binding is logged as routine, not alerted. This is the decades-old reference behavior the others mirror.
- **Zeek** - logs all ARP neutrally; the community `arp-spoof`-style scripts maintain a state table of the most-recent valid MAC per IP and alert only when a GARP announces a *different* binding. "Gratuitous ARP by itself is neither malicious nor anomalous" is the design stance; detection requires "temporal correlation of binding state."
- **Suricata** - stateful ARP binding tracking; alerts on the same IP appearing with a different MAC within a window, not on gratuitous packets in isolation.
- **Snort** - ARP preprocessor (`arpspoof`) explicitly treats GARP announcing a host's own address as normal and flags only "the same IP address claimed by multiple MAC addresses."

**Common thread:** ARP spoofing is *defined by mapping inconsistency*, not by use of the gratuitous mechanism. Every tool gates the alert on conflict against prior state - exactly the case-(A)-vs-case-(B) distinction.

### False-positive profile of alerting on ALL gratuitous ARP
Synthesized estimates (directional, from detection-engineering literature, not a single audited measurement): legitimate GARP is high-volume routine traffic (~5-15 packets/host/day from boot, DHCP renewal, DAD per RFC 5227, interface toggles; far higher on servers, HA clusters, VMs during migration/failover, VDI boot storms). Blanket GARP alerting yields ~95-99%+ false-positive rates and is consistently described as operationally non-viable; all production guidance warns against signatures that fire on packet *type* rather than binding *change*. The specific per-day FP counts in the literature are illustrative estimates, not authoritative measurements - flag as such.

---

## Point 4 - False-positive implication for a LOW-severity AiTM tag

How common is legitimate GARP: **very common and entirely normal.** RFC 826 permits it; RFC 5227 (IPv4 Address Conflict Detection) uses ARP probes/announcements as standard practice; HSRP/VRRP failover, DHCP assignment, VM live migration, and interface-up all legitimately emit GARP. It is a network-health mechanism, not an anomaly.

Implication of tagging every GARP with T0830/T1557.002 even at LOW severity:
- **Semantic incorrectness.** A MITRE technique tag is an *attribution claim* ("this observation is evidence of adversary technique X"). Benign GARP is not evidence of AiTM; it is routine. LOW severity does not fix a wrong attribution - it just makes a wrong claim quietly.
- **Downstream poisoning.** If wirerust output feeds a SIEM/ATT&CK-coverage view, every host's normal boot/DHCP traffic would light up T1557.002 coverage, inflating technique-sighting counts and eroding analyst trust (alert/normalization fatigue) - the exact failure mode the IDS tools were designed to avoid.
- **Severity is the wrong lever.** The correct lever is *attribution gating*: only the conflicting GARP earns the technique tag. The benign GARP, if recorded at all, should be a plain informational observation with no technique ID.

---

## Recommendation (detailed) + rationale

1. **Conflicting GARP (case B): emit T1557.002 (always) and T0830 (when ICS-context is in scope).** A GARP claiming an IP already observed bound to a different MAC is the textbook ARP-cache-poisoning indicator and matches MITRE's DET0387 analytics verbatim ("overriding legitimate ARP ownership", "multiple IPs to one MAC", "inconsistent with ... DHCP lease"). Severity should reflect target criticality if known (gateway/critical host > workstation), but a conflicting GARP is a genuine, citable AiTM indicator.

2. **Benign / non-conflicting GARP (case A): do NOT attach T0830 or T1557.002.** Either (a) drop it, or (b) emit a non-attributed informational record (e.g. `info`: "gratuitous ARP self-announcement IP<->MAC") that carries NO MITRE technique tag. Option (b) is the legitimate "informational tag" middle ground - the key constraint is that the *technique attribution* is reserved for conflict.

3. **State requirement.** This necessitates a stateful IP<->MAC binding table (as every reference tool maintains). A purely stateless per-packet analyzer cannot distinguish A from B and would be forced into the wrong (blanket-tag) behavior. If wirerust is currently stateless for ARP, the correct fix is to add binding-conflict state, not to blanket-tag.

### Rationale summary
- MITRE's own text defines the technique as the *malicious/overriding announcement*, not the gratuitous mechanism (Point 1, 2).
- MITRE's detection analytics (DET0387, AN1091-AN1093) and the T0830 detection note all key on *conflict/anomaly vs. normal*, never on plain GARP (Point 1, 2).
- Suricata, Zeek, Snort, arpwatch are unanimous: conflict-gated, not type-gated (Point 3).
- Benign GARP is high-volume normal traffic; blanket attribution is both semantically wrong and operationally noisy (Point 4).

### Caveats / where evidence is weaker
- **Volume statistics are directional, not audited.** The "5-15/host/day" and ">95% FP" figures come from detection-engineering synthesis, not a single authoritative benchmark. The *direction* (benign GARP is common; blanket alerting is noisy) is unambiguous and corroborated by tool documentation that explicitly warns against type-based alerting; treat exact numbers as estimates.
- **T0830 / GARP linkage is indirect.** T0830 does not mention gratuitous ARP by name; it names "ARP poisoning" generically. So even for case (B), T0830 is the *ICS framing* of the same conflict indicator; it is correct to emit it only when an ICS context applies and only on conflict.
- **Not genuinely ambiguous.** Both the MITRE primary sources and IDS convention point the same direction. There is no strong contrary case that benign GARP should carry the technique tag. The only defensible "tag everything" argument is "we want raw ARP visibility" - which is satisfied by an *untagged informational record*, not by a MITRE attribution.

---

## Sources (URLs)

Authoritative (MITRE, verified directly via WebFetch):
- T1557.002 ARP Cache Poisoning - https://attack.mitre.org/techniques/T1557/002/
- T1557 Adversary-in-the-Middle (parent) - https://attack.mitre.org/techniques/T1557/
- T0830 Adversary-in-the-Middle (ICS) - https://attack.mitre.org/techniques/T0830/
- DET0387 detection strategy (AN1091-AN1093) - https://attack.mitre.org/detectionstrategies/DET0387/
- M1042 mitigation - https://attack.mitre.org/mitigations/M1042/

Standards:
- RFC 826 (ARP, permits gratuitous ARP)
- RFC 5227 (IPv4 Address Conflict Detection / DAD using ARP)

Detection-engineering / IDS (synthesized via Perplexity sonar-deep-research; treat tool-config specifics as directional):
- Zeek ARP detection (community arp-spoof scripts) - https://docs.zeek.org/
- Suricata documentation - https://docs.suricata.io/
- Snort arpspoof preprocessor - https://www.snort.org/
- arpwatch (LBNL) - man arpwatch
- StartupDefense T1557.002 overview - https://www.startupdefense.io/mitre-attack-techniques/t1557-002-arp-cache-poisoning

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) MITRE T1557.002 + T0830 definitions, detection guidance, benign-vs-malicious GARP; (2) IDS conventions (Suricata/Zeek/Snort/arpwatch), GARP false-positive profile, legitimate GARP frequency |
| WebFetch | 3 | Direct verification of attack.mitre.org pages: T1557.002, T0830, and DET0387 detection-strategy analytics |
| Training data | 1 area | RFC 826 / RFC 5227 framing of gratuitous ARP and DAD (general protocol knowledge; flagged) |

**Total MCP tool calls:** 2 (both `perplexity_research`, high reasoning_effort) + 3 WebFetch verifications.
**Training data reliance:** low - all load-bearing claims (technique semantics, detection analytics language, IDS conventions) verified against authoritative MITRE pages and deep-research synthesis. Only general RFC framing relied on model knowledge, and it is non-controversial.
