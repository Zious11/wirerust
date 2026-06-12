# MITRE Technique Mapping Research — ARP Security Analyzer (wirerust)

**Date:** 2026-06-12
**Researcher:** vsdd-factory research-agent
**Scope:** Validate MITRE ATT&CK technique IDs for an ARP-spoofing / ARP-cache-poisoning / gratuitous-ARP detection finding in wirerust (ICS-focused packet analyzer).
**Verification basis:** Live `attack.mitre.org` pages fetched 2026-06-12 (NOT model memory). The initial deep-research call fell back to training data because no live results were injected; all conclusions below are sourced from direct page fetches and live search, not that call.

---

## Current ATT&CK Version

**ATT&CK v19.1 — current as of April 28, 2026.**
Source: https://attack.mitre.org/resources/versions/ ("ATT&CK v19.1 — April 28, 2026 — current"; catalog uses a `major.minor` schema).

All findings below were verified against the v19.1 content release.

---

## Recommendation Table (verified against live pages)

| Technique ID | Exact Title | Matrix | Status (v19.1) | Source URL |
|---|---|---|---|---|
| **T0830** | Adversary-in-the-Middle | ICS | **Current** — Version 2.0, created 2020-05-21, last modified 2025-04-16. NOT deprecated/revoked/superseded. | https://attack.mitre.org/techniques/T0830/ |
| **T1557** | Adversary-in-the-Middle | Enterprise | **Current** — Version 2.5, last modified 2026-05-12. NOT deprecated. Parent of T1557.002. | https://attack.mitre.org/techniques/T1557/ |
| **T1557.002** | Adversary-in-the-Middle: ARP Cache Poisoning | Enterprise (sub-technique of T1557) | **Current** — NOT deprecated/revoked/superseded. | https://attack.mitre.org/techniques/T1557/002/ |

---

## Findings by Research Question

### 1. ICS — "Adversary-in-the-Middle" = T0830 (CONFIRMED current)
- **ID and title confirmed:** T0830, titled **"Adversary-in-the-Middle"** (NOT the older "Man in the Middle" wording — MITRE renamed AiTM techniques in earlier releases; the live page shows the current "Adversary-in-the-Middle" title). Source: https://attack.mitre.org/techniques/T0830/
- **ARP is an explicit example/procedure:** The T0830 page states some of the most common AiTM methods are "Address Resolution Protocol (ARP) poisoning and the use of a proxy." The mitigation section also notes "statically defined ARP entries can prevent manipulation… as some AiTM techniques depend on sending spoofed ARP messages." So ARP spoofing/poisoning is a named technique-level example under T0830, not a separate ICS technique.
- **Version:** T0830 Version 2.0; content release ATT&CK v19.1.
- **No revocation.** Page is not flagged deprecated/revoked/superseded.

### 2. Enterprise — T1557.002 "ARP Cache Poisoning" (CONFIRMED current)
- **ID/title confirmed:** T1557.002, **"Adversary-in-the-Middle: ARP Cache Poisoning"**. Source: https://attack.mitre.org/techniques/T1557/002/
- **Parent confirmed:** T1557 "Adversary-in-the-Middle" (Version 2.5, current). Sub-techniques of T1557 in v19.1: T1557.001 (LLMNR/NBT-NS Poisoning and SMB Relay / Name Resolution Poisoning), **T1557.002 (ARP Cache Poisoning)**, T1557.003 (DHCP Spoofing), T1557.004 (Evil Twin). Source: https://attack.mitre.org/techniques/T1557/
- **No revocation.** Neither T1557 nor T1557.002 is flagged deprecated/revoked/superseded.

### 3. Gratuitous ARP — indicator/procedure, NOT a named technique
- Gratuitous ARP is **not** a standalone MITRE technique. It appears as a procedural indicator inside the AiTM ARP techniques. The T1557.002 page explicitly describes adversaries sending "a gratuitous ARP reply that maliciously announces the ownership of a particular IP address." Source: https://attack.mitre.org/techniques/T1557/002/
- **Classification guidance:** Detection tools treat gratuitous ARP (and unsolicited/spoofed ARP replies) as an *indicator* of the AiTM technique, mapping the finding to the AiTM technique ID rather than inventing a "gratuitous ARP" ID. For an ICS tool, that maps to **T0830**; the Enterprise-equivalent indicator maps to **T1557.002**.

### 4. Cross-check vs wirerust convention + revocation-risk flag
- wirerust's existing convention mixes ICS IDs (T0827, T0814, T0836) with Enterprise-style IDs that carry an ICS-adapted form (T1692.001, T1691.001 from the DNP3 work). The DNP3 IDs reflect the v19 ICS restructuring where **T0855/T0856 were revoked and folded into T1692 "Unauthorized Message"** (confirmed live: T1692 is present and current in the ICS technique list — https://attack.mitre.org/techniques/ics/). This is the revocation that previously bit the project.
- **Revocation-risk check for ARP IDs — CLEAR.** Explicitly verified against v19.1:
  - **T0830** — Version 2.0, current, last modified 2025-04-16. Survived the v19 ICS restructuring untouched. No revocation/supersession.
  - **T1557 / T1557.002** — Enterprise, current (T1557 v2.5, modified 2026-05-12). No revocation/supersession.
  - The v19 churn that affected ICS command-message techniques (T0855/T0856 → T1692.x) and the Enterprise Defense-Evasion/Impair-Defenses restructuring (T1562, T1685, etc.) does **not** touch any AiTM/ARP technique. Sources: https://attack.mitre.org/resources/updates/ , https://attack.mitre.org/techniques/ics/
- **None of the ARP-relevant IDs (T0830, T1557, T1557.002) have been revoked or superseded in ATT&CK v19.1.**

---

## Recommended Mapping for wirerust ARP Findings

**Primary (carry on every ARP-spoofing / ARP-cache-poisoning / gratuitous-ARP detection):**

> **T0830 — Adversary-in-the-Middle (ICS matrix, ATT&CK v19.1).**

Rationale:
- wirerust is an ICS-focused tool that maps detections to **ICS** technique IDs. T0830 is the correct ICS-matrix technique, ARP poisoning is an explicit example under it, and it is current/non-revoked in v19.1.
- Using T0830 keeps the ARP analyzer consistent with the existing ICS-first convention (T0827, T0814, T0836).

**Optional secondary / Enterprise cross-reference (only if wirerust already dual-tags ICS+Enterprise):**

> **T1557.002 — Adversary-in-the-Middle: ARP Cache Poisoning (Enterprise matrix).**

Use this only if the project's reporting convention already carries Enterprise IDs alongside ICS IDs. It is the most ARP-specific ID in the catalog and is current. If wirerust tags a single ID per finding, prefer **T0830** for consistency with its ICS posture and do not add T1557.002.

**Gratuitous ARP specifically:** map to the same technique ID as ARP spoofing (T0830, optionally T1557.002) and record "gratuitous ARP reply" as the *indicator/procedure* in the finding evidence — do not seek a dedicated technique ID, as none exists.

**Confidence: HIGH.** All technique IDs, titles, parent relationships, deprecation status, and the current version were verified against live attack.mitre.org pages on 2026-06-12. Nothing in this conclusion is inconclusive.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Initial AiTM/ARP technique sweep — NOTE: returned a training-data fallback (no live results injected); its content was NOT used for conclusions, only to scope follow-up fetches. |
| Perplexity perplexity_search | 1 | Verify T0855/T0856 → T1692 revocation context and v19 changes (live ranked URLs). |
| WebFetch | 5 | Live verification of T0830, T1557, T1557.002 pages; ATT&CK current-version page; T0855 (came back empty x2 — see below). |
| Training data | 0 areas | Not relied upon — all claims sourced to live pages/search. |

**Total MCP tool calls:** 2 (1 perplexity_research, 1 perplexity_search) + 5 WebFetch.
**Training data reliance:** low — every technique ID, title, status, and version was confirmed against live attack.mitre.org fetches or live search; the deep-research call's training-data fallback was explicitly discarded.

### Caveats / partial results
- The `perplexity_research` deep call did not receive live search results and fell back to pre-cutoff knowledge; per agent policy it was treated as non-authoritative and superseded by direct page fetches.
- The T0855 page (`https://attack.mitre.org/techniques/T0855/`) returned empty content on two WebFetch attempts (revoked/redirected ICS pages render differently). The T0855/T0856 → T1692 revocation was instead confirmed indirectly via the live ICS technique index (T1692 "Unauthorized Message" present and current) and the v19 updates page. This does not affect any ARP conclusion.
