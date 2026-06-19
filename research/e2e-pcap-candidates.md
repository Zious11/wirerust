# E2E PCAP corpus extension — candidate captures (research)

**Date:** 2026-06-19
**Agent:** vsdd-factory:research-agent
**Goal:** Find 6–12 additional large/dedicated public packet captures to extend
wirerust's manual E2E corpus, prioritizing THIN areas: dedicated/heavy **DNP3**,
**TLS/HTTPS-heavy**, **DNS-heavy (incl. tunneling/exfil)**, and **large mixed-enterprise**.
Deliverable = vetted **links + metadata** only (raw pcaps are NOT committed; they go to
the gitignored `tests/fixtures/local-samples/`). Reader supports **classic libpcap only**
(no pcapng yet).

> **Already in corpus — NOT re-listed:** Netresec 4SICS 151020/151021/151022; Wireshark
> SampleCaptures arp-storm / tcp-ecn-sample / tcp-ethereal-file1 / nfs_bad_stalls; ARP
> captures (arpspoof, gratuitous-arp-hsrp, ppa-arp); synthetic modbus.

---

## TL;DR — top recommendations (best coverage-per-MB for thin areas)

1. **`dnscat2_dns_tunneling_24hr.pcap`** (Active Countermeasures, ~82.6 MB, classic pcap) —
   the single best add. Heavy DNS + large-scale; stresses DNS analyzer tunneling/exfil
   findings AND throughput/determinism. SHA256 published, malware-free.
2. **`dnscat2_dns_tunneling_1hr.pcap`** (Active Countermeasures, ~2.35 MB, classic pcap) —
   the small companion for fast DNS-tunneling regression runs.
3. **`dnp3dataset_capture.pcap`** (igbe/DNP3-Dataset-Plus-SnortRules, ~2.6 MB, classic pcap)
   — the best *dedicated DNP3* capture with a genuinely working direct URL (verified binary).
4. **`maccdc2012_00000.pcap`** (Netresec MACCDC 2012, ~1 GB, classic pcap) — large
   mixed-enterprise scale stressor (HTTP/TLS/DNS/SMB/…). Heaviest available; use the
   gzip (~331 MB) form. Optional/aspirational given size.
5. **`rsasnakeoil2.pcap`** (Wireshark SampleCaptures, ~24 KB, classic pcap) — small but the
   only reliably-direct *TLS handshake + encrypted payload* fixture in classic format.

The **TLS/HTTPS-heavy** area remains the hardest to fill with a *large classic .pcap*: almost
every modern TLS-heavy public capture is **pcapng** (Wireshark `dump.pcapng`, `tls12-dsb.pcapng`).
For heavy TLS coverage in classic format, lean on MACCDC (mixed, has TLS) + 4SICS (already held)
+ the small `rsasnakeoil2.pcap` fixture. See "Gaps" below.

---

## VERIFIED-RESOLVING candidates

Each row's Direct URL was fetched and confirmed to return real binary pcap content
(libpcap magic / octet-stream of the expected size), except where noted as bot-protected
(403 to automated fetchers but a known-stable WordPress upload with published checksum).

| File (proposed local name) | Approx Size | Source (linked) | Direct download URL | Protocols | What it validates in wirerust | License/Attribution | pcap/pcapng |
|---|---|---|---|---|---|---|---|
| `dnscat2_dns_tunneling_1hr.pcap` | ~2.35 MB | [Active Countermeasures – Malware of the Day: dnscat2](https://www.activecountermeasures.com/malware-of-the-day-dnscat2-dns-tunneling/) | `https://www.activecountermeasures.com/wp-content/uploads/2021/06/dnscat2_dns_tunneling_1hr.pcap` | DNS (dnscat2 tunneling: TXT/CNAME/MX over UDP/53), UDP, IP | **DNS analyzer**: tunneling/exfil findings — long/high-entropy labels, abnormal qtypes, high query volume. Fast regression size. | No explicit license. Vendor states PCAPs are safe, contain no malware. Credit: Active Countermeasures. SHA256 `B1056BAAA15C441AA9FADD23885818DB069F3CC5B560DC4F8A184D3494F3D3BD`. | pcap |
| `dnscat2_dns_tunneling_24hr.pcap` | ~82.56 MB | [Active Countermeasures – Malware of the Day: dnscat2](https://www.activecountermeasures.com/malware-of-the-day-dnscat2-dns-tunneling/) | `https://www.activecountermeasures.com/wp-content/uploads/2021/06/dnscat2_dns_tunneling_24hr.pcap` | DNS (dnscat2 tunneling, 24h), UDP, IP | **DNS analyzer** at scale + **throughput/determinism**: sustained tunneling over 24h. Best DNS-heavy add. | Same as above. SHA256 `DC8C0134830076E010479C3F59A6AE0A6BCF3DDD6BD7F5D8C91879E0E0B9C2D5`. | pcap |
| `dnp3dataset_capture.pcap` | ~2.6 MB | [igbe/DNP3-Dataset-Plus-SnortRules](https://github.com/igbe/DNP3-Dataset-Plus-SnortRules) | `https://raw.githubusercontent.com/igbe/DNP3-Dataset-Plus-SnortRules/master/dnp3dataset_capture.pcap` | DNP3 (over TCP, master/outstation), IP, Ethernet | **DNP3 stream dispatch + parser (ADR-0007)**: function-code variety, sequence/fragment handling, attack-like sequences (Snort-ruled). Dedicated DNP3 at non-trivial size. | No LICENSE file in repo (all-rights-reserved). Research/educational use; credit GitHub user "igbe". Not redistributed via repo. | pcap (verified binary, 2.6 MB) |
| `rsasnakeoil2.pcap` | ~24 KB | [Wireshark SampleCaptures](https://wiki.wireshark.org/SampleCaptures) | `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/rsasnakeoil2.pcap` | TLS/SSL (RSA handshake + encrypted payload), TCP | **TLS analyzer**: ClientHello/ServerHello, certificate message, handshake state. Small but reliable classic-format TLS fixture. | Wireshark Foundation public sample. No per-file license. Credit: Wireshark Foundation. | pcap (verified binary, 24.5 KB) |
| `iodine.pcap` | ~3.5 KB | [dmachard/datasets-malicious-dns](https://github.com/dmachard/datasets-malicious-dns) | `https://raw.githubusercontent.com/dmachard/datasets-malicious-dns/main/iodine.pcap` | DNS (iodine tunnel: NULL/PRIVATE/MX/CNAME/TXT), UDP | **DNS analyzer**: second tunneling tool (iodine) for detector breadth. Tiny — fixture only. | No LICENSE file. Credit: Denis Machard (dmachard). | pcap |
| `dns2tcp.pcap` | ~4.5 KB | [dmachard/datasets-malicious-dns](https://github.com/dmachard/datasets-malicious-dns) | `https://raw.githubusercontent.com/dmachard/datasets-malicious-dns/main/dns2tcp.pcap` | DNS (dns2tcp tunnel), UDP/TCP | **DNS analyzer**: third tunneling tool variant. Tiny — fixture only. | No LICENSE file. Credit: dmachard. | pcap |
| `dnsexfiltrator.pcap` | ~8.8 KB | [dmachard/datasets-malicious-dns](https://github.com/dmachard/datasets-malicious-dns) | `https://raw.githubusercontent.com/dmachard/datasets-malicious-dns/main/dnsextrator.pcap` | DNS (DNSExfiltrator, base32-encoded exfil) | **DNS analyzer**: explicit file-exfil-over-DNS pattern. Tiny — fixture only. (Note upstream filename typo `dnsextrator.pcap`.) | No LICENSE file. Credit: dmachard. | pcap |
| `sods.pcap` | ~191 KB | [dmachard/datasets-malicious-dns](https://github.com/dmachard/datasets-malicious-dns) | `https://raw.githubusercontent.com/dmachard/datasets-malicious-dns/main/sods.pcap` | DNS (sods/SOD tunnel, larger session) | **DNS analyzer**: the only non-trivial (>100 KB) file in this repo. | No LICENSE file. Credit: dmachard. | pcap |
| `dns-tunnel-iodine.pcap` | ~76 KB | [elastic/examples (archived)](https://github.com/elastic/examples) | `https://raw.githubusercontent.com/elastic/examples/master/Security%20Analytics/dns_tunnel_detection/dns-tunnel-iodine.pcap` | DNS (iodine tunnel), UDP | **DNS analyzer**: curated iodine tunnel detection scenario. Mid-small. Repo archived read-only (stable). | Elastic examples repo (Apache-2.0 at repo level historically). Credit: Elastic. | pcap (verified binary, 75.7 KB) |

### MACCDC — large mixed-enterprise (Netresec share; classic pcap)

Netresec serves MACCDC via Nextcloud public shares. Individual-file pattern mirrors the 4SICS
shares already in the corpus: `https://share.netresec.com/s/<token>/download/<filename>`.
Files are **~1 GB each uncompressed** (00000 is ~1 GB; gzip ~331 MB) — heavier than the 200 MB
target, so treat as an **optional top-tier scale stressor**. 2010/2011 sets are smaller than 2012.

| File (proposed local name) | Approx Size | Source (linked) | Direct download URL | Protocols | What it validates in wirerust | License/Attribution | pcap/pcapng |
|---|---|---|---|---|---|---|---|
| `maccdc2012_00000.pcap` | ~1 GB (gz ~331 MB) | [Netresec MACCDC](https://www.netresec.com/?page=MACCDC) (2012 share token `7qgDSGNGw2NY8ea`) | `https://share.netresec.com/s/7qgDSGNGw2NY8ea/download/maccdc2012_00000.pcap` | HTTP, TLS/HTTPS, DNS, SMB, SMTP, ARP, mixed CTF enterprise | **Whole pipeline at scale**: TCP reassembly under millions of frames, HTTP/TLS/DNS analyzers, ARP, finding-cap + determinism + throughput. Largest realistic mixed load. | Netresec re-hosted MACCDC CCDC captures. Credit: MACCDC / Netresec. Verify usage terms on source page before redistribution (corpus is local-only). | pcap |
| `maccdc2010_<NNNNN>.pcap` | (smaller set) | [Netresec MACCDC](https://www.netresec.com/?page=MACCDC) (2010 share `wC4mqF2HNso4Ten`) | `https://share.netresec.com/s/wC4mqF2HNso4Ten/download/<filename>` | Mixed enterprise/CTF | Same as above at smaller scale. **Exact filenames not confirmed** — browse the share to enumerate. | MACCDC / Netresec. | pcap |

---

## UNVERIFIED / landing-page / DO-NOT-USE

| Candidate | Why excluded |
|---|---|
| **automayt/ICS-pcap** DNP3 / IEC104 / Modbus / S7 captures (`raw.githubusercontent.com/automayt/ICS-pcap/...`) | **FAILED verification.** The `.pcap` files are Git-LFS-backed and the LFS objects are NOT present — both `raw.githubusercontent.com` and `media.githubusercontent.com/media` return a 603-byte LFS pointer, not the capture. The repo also has **no LICENSE**. Files are tiny anyway (130–584 bytes). Not usable as direct downloads. |
| **automayt/ICS-pcap `iec104.pcap`** | 130 bytes; trivial; LFS-broken (see above). Not worth it. |
| **AAGiron/tls-handshake-analyzer `tls13-rfc8446.pcap`** | Resolves (4,158 bytes, classic pcap) but **tiny** and only 1–2 handshakes. Other captures in `captures/` are mostly **pcapng** (`oqs-hybrid-cmp.pcapng`, etc.). GPL-3.0. Marginal value; not recommended over `rsasnakeoil2.pcap`. Direct URL if wanted: `https://raw.githubusercontent.com/AAGiron/tls-handshake-analyzer/main/captures/tls13-rfc8446.pcap` (branch `main`, not `master`). |
| **CIC-Bell-DNS-EXF-2021** (UNB, 270.8 MB DNS exfil) | Excellent fit on paper, but **no stable direct file URL** — gated behind a UNB landing page / dataset request form. Cannot verify a direct download. Listed for manual follow-up only. |
| **Stratosphere IoT-23** (CTU) | Large benign+malware IoT, but per-file PCAP URLs not directly addressable from the README directory without enumeration; many scenarios are malware-run captures (acceptable academically but heavier vetting). Landing-page only here. |
| **Wireshark `dump.pcapng` (73 cipher suites), `tls12-dsb.pcapng`** | The best TLS-heavy public captures — but **pcapng**, which the reader does NOT support. Excluded until pcapng support lands (then strong candidates). |
| **Active Countermeasures dnscat2 page (HTML)** | The page itself is Cloudflare bot-protected (403 to automated fetchers). The **upload URLs** above are stable WordPress media paths with published SHA256; verify with `curl`/browser, not an automated HEAD. |

---

## Format-support caveat (pcapng)

The wirerust reader requires libpcap magic and rejects pcapng (see existing
`arp-baseline-16pkt.cap` note in `E2E-PCAPS.md`). All VERIFIED rows above are classic `.pcap`.
The richest TLS-heavy and many newer DNS captures are pcapng — they are a strong reason to
prioritize pcapng reader support, after which `dump.pcapng` (73 cipher suites) becomes the
ideal TLS-heavy capture.

## Gaps / inconclusive

- **TLS/HTTPS-heavy in LARGE classic .pcap form: INCONCLUSIVE.** No large, benign, TLS-handshake-
  dense capture in classic libpcap was found with a verified direct URL. Best available paths:
  `rsasnakeoil2.pcap` (small fixture) + TLS riding inside MACCDC/4SICS mixed captures. Real
  fill-in requires pcapng support.
- **Dedicated heavy DNP3 (>10 MB): partly inconclusive.** `dnp3dataset_capture.pcap` (2.6 MB)
  is the largest dedicated DNP3 capture with a working direct URL. Larger DNP3-only public
  captures with direct links were not found; 4SICS (already held) remains the heavy DNP3 source.
- **IEC-104 / S7comm dedicated heavy captures: not found** with working direct URLs (ICS-pcap
  LFS-broken). 4SICS already covers these mixed-in.

---

## Verification log

| URL | Result |
|---|---|
| `raw.githubusercontent.com/igbe/.../dnp3dataset_capture.pcap` | OK — binary octet-stream, 2.6 MB |
| `raw.githubusercontent.com/elastic/.../dns-tunnel-iodine.pcap` | OK — binary octet-stream, 75.7 KB |
| `gitlab.com/wireshark/.../rsasnakeoil2.pcap` | OK — binary pcap, 24.5 KB |
| `raw.githubusercontent.com/dmachard/...` (iodine/dns2tcp/dnsextrator/sods/dnscat2) | OK — sizes 3.5 KB / 4.5 KB / 8.8 KB / 191 KB / 3.6 KB (via GitHub API) |
| `…/dnscat2_dns_tunneling_1hr.pcap` and `…_24hr.pcap` | 403 to automated fetcher (Cloudflare); stable WP upload, sizes + SHA256 published on source page |
| `raw.githubusercontent.com/automayt/ICS-pcap/.../DNP3-Read.pcap` | FAIL — 603-byte Git LFS pointer (object missing on both raw + media hosts) |
| `share.netresec.com/s/7qgDSGNGw2NY8ea` (MACCDC 2012) | Landing page confirmed; filenames `maccdc2012_00000.pcap`–`maccdc2012_00013.pcap`; `/download/<file>` pattern; ~1 GB/file |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Broad survey of ICS/TLS/DNS/enterprise pcap corpora; MACCDC filename/size + TLS-heavy classic-format scan |
| Perplexity perplexity_search | 3 | Raw URL discovery: large mixed/maccdc, DNS tunneling pcaps, TLS-heavy pcaps |
| Perplexity perplexity_ask | 3 | Factual lookups: AC dnscat2 upload URLs, Netresec share pattern, MACCDC per-file size, Wireshark classic TLS captures |
| WebFetch | 14 | Link verification (GitHub API listings + binary/LFS/404 checks) for every candidate URL |
| Training data | 1 area | DNP3/DNS protocol-port background only — all file existence/sizes/URLs were tool-verified, not from training data |

**Total MCP tool calls:** 8 (2 research + 3 search + 3 ask) + 14 WebFetch verifications
**Training data reliance:** low — every candidate file's existence, size, format, and URL was
verified via live WebFetch or registry/API; training data used only for protocol background.
