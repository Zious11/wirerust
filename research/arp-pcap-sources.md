# ARP PCAP sources for wirerust v0.7.0 ARP Security Analyzer — verified link list

Research date: 2026-06-16. Research agent run.

Goal: find publicly-downloadable real packet captures containing ARP traffic to
validate the v0.7.0 ARP Security Analyzer detections (D1 spoofing/poisoning,
D2 gratuitous ARP, D3 ARP storm, D11 malformed ARP, D12 L2/L3 MAC mismatch,
plus benign baseline). Captures stay LOCAL (gitignored under
`tests/fixtures/local-samples/`); only the download links are tracked, in
`tests/fixtures/E2E-PCAPS.md`.

I did NOT download these files (no shell). Each URL below was probed with
WebFetch/WebSearch; the "verified-live?" column records the result of that probe.

## Candidate table (drop-in for E2E-PCAPS.md)

| suggested filename | direct URL | source | ARP behaviors (D-codes) | attribution/license | verified-live? |
|---|---|---|---|---|---|
| `arp-storm.pcap` | `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/arp-storm.pcap` | Wireshark SampleCaptures wiki (now GitLab-hosted), ARP/RARP section | **D3** (>20 ARP req/s on a cable-modem link — the canonical ARP-storm sample), incidental **D2** | Wireshark wiki sample captures; no explicit per-file license, public sample-capture collection. Credit Wireshark Foundation. | YES — `wiki.wireshark.org/uploads/...` 302-redirects to this GitLab URL; fetched as binary `application/octet-stream`, ~46.2 KB |
| `220703_arp-storm.pcapng` | `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/f59564719471dc67295224d1f18c4857/220703_arp-storm.pcapng` | Wireshark SampleCaptures wiki (GitLab), ARP/RARP section | **D3** (same storm content as arp-storm.pcap, re-saved as pcapng with a Name Resolution Block) | Wireshark Foundation, public sample captures | YES — fetched as binary `application/octet-stream`, ~68.1 KB |
| `arpspoof.pcap` | `https://raw.githubusercontent.com/researcher111/ARP-pcap-files/master/arpspoof.pcap` | GitHub repo `researcher111/ARP-pcap-files` (single-file ARP-spoof capture) | **D1** spoofing/cache-poisoning (arpspoof-generated), with **D12** L2/L3 MAC mismatch and likely **D2** unsolicited replies | No LICENSE file in repo — treat as all-rights-reserved; use locally for validation only, do not redistribute | YES (indirect) — raw URL resolved; file is >10 MB (exceeded the 10 MB fetch cap), confirming a substantial real spoofing capture. Size unverified-exact; large. |
| `ppa-arp.pcap` | `https://raw.githubusercontent.com/markofu/pcaps/master/PracticalPacketAnalysis/ppa-capture-files/arp.pcap` | GitHub `markofu/pcaps` → `PracticalPacketAnalysis/ppa-capture-files/arp.pcap` (Practical Packet Analysis book sample) | **baseline** benign ARP request/reply (GitHub reports 144 bytes → ~2 packets); no-false-positive check | markofu/pcaps is a re-host ("not my creation, free and open to the public"); no formal license. PPA book sample by Chris Sanders. | YES — raw URL fetched as binary, 144 bytes; GitHub blob page confirms 144-byte size |
| `gratuitous-arp-hsrp.cap` | `https://raw.githubusercontent.com/epiecs/packetlife-backup/master/pcaps/gratuitous%20arp%20hsrp.cap` | PacketLife.net capture backup (GitHub mirror `epiecs/packetlife-backup`, `pcaps/` dir) | **D2** gratuitous ARP — HSRP active-router GARP (src MAC `00:00:0c:07:ac:01`), 6 packets | PacketLife.net captures by Jeremy Stretch; mirror repo unlicensed. Credit PacketLife.net. | YES — raw URL fetched as binary, ~480 bytes |
| `arp_pcap.pcapng.cap` | `https://raw.githubusercontent.com/epiecs/packetlife-backup/master/pcaps/arp_pcap.pcapng.cap` | PacketLife.net backup (GitHub `epiecs/packetlife-backup`) | **baseline** ARP request/reply (16 packets, ~2.2 KB) — secondary benign baseline | PacketLife.net / Jeremy Stretch; mirror unlicensed. Credit PacketLife.net. | YES — raw URL fetched as binary, ~2.2 KB (Cisco 3745 IOS 12.4 traffic visible) |
| `4SICS-GeekLounge-151022.pcap` | `https://share.netresec.com/s/gw6Y2QzJHqDD5pr/download/4SICS-GeekLounge-151022.pcap` | Netresec 4SICS ICS lab (already in this repo's E2E index) | **baseline at scale** — large ICS capture with background ARP; D3/false-positive-rate stress under heavy mixed traffic | Netresec 4SICS; **credit CS3Sthlm / 4SICS** if redistributed/used in training | YES — already a tracked, fetched entry in `E2E-PCAPS.md` (200 MB; sha256 on record) |

## Best primary validators (use these first)

1. **`arpspoof.pcap`** (`researcher111/ARP-pcap-files`) — the headline **D1** ARP
   spoofing / cache-poisoning validator. Real arpspoof-generated MITM traffic,
   substantial size (>10 MB), so it should exercise IP→multiple-MAC rebinding
   (D1), unsolicited replies (D2), and L2/L3 MAC mismatch (D12) together.
   Caveat: no license in the repo — keep it local-only (which matches our
   gitignored-fixtures policy) and do not redistribute.

2. **`arp-storm.pcap`** (Wireshark SampleCaptures) — the canonical **D3** ARP
   storm sample (>20 ARP req/s). This is literally the file Wireshark's own
   "Detect ARP request storms" preference and SharkFest training use, so it is
   the cleanest, most reputable storm validator. Pair with the `.pcapng`
   variant if you want to test pcapng parsing of the same content.

3. **`gratuitous-arp-hsrp.cap`** (PacketLife backup) — focused **D2** gratuitous
   ARP validator (HSRP failover GARP). Tiny and unambiguous: good for asserting
   the GARP / GARP-binding-conflict path fires on a real-world GARP without
   storm/spoof noise.

Then use **`ppa-arp.pcap`** + **`arp_pcap.pcapng.cap`** as benign **baselines**
(no-false-positive checks) and **`4SICS-GeekLounge-151022.pcap`** (already
indexed) for baseline-at-scale / false-positive-rate under heavy mixed traffic.

## Coverage gaps / flags (be explicit about what was NOT found)

- **D11 malformed ARP:** I did not find a reputable public capture *purpose-built*
  for malformed ARP (bad hlen/plen, truncated, wrong hardware-type). The
  spoofing/storm captures above may incidentally contain odd frames, but for a
  deterministic D11 fixture you will likely need a **synthetic crafted pcap**
  (same pattern as the existing `tests/fixtures/mk_modbus_large_pcap.py` /
  `modbus-large.pcap`). Recommend generating a small Scapy-built malformed-ARP
  fixture rather than relying on a found capture. FLAGGED as inconclusive.
- **D12 L2/L3 MAC mismatch:** expected to be exercised *within* `arpspoof.pcap`
  (spoofing inherently desyncs ARP-header MAC vs Ethernet-header MAC), but no
  standalone "MAC mismatch only" public capture was found. Confirm by inspection
  after download; otherwise synthesize.
- **Exact sizes/packet counts** for `arpspoof.pcap` are unverified-precise
  (fetch hit the 10 MB cap before completing); known only to be >10 MB.
- **packetlife.net live site** returned ECONNREFUSED during this run; I switched
  to the GitHub mirror `epiecs/packetlife-backup` (master branch) for stable raw
  URLs. If you prefer the canonical host, the live paths are
  `https://packetlife.net/media/captures/<file>` but were not reachable at probe
  time — FLAGGED.
- **TryHackMe `ARP_Poison.pcapng` / Wireshark Traffic-Analysis room** is a strong
  ARP-poisoning capture but is **gated behind the TryHackMe platform login** — no
  stable public direct URL. Excluded per the "no login/agreement" preference;
  noted here for completeness.
- **`tinkerlev/ARP-Spoofing`** (surfaced in search) was inspected and contains
  only `README.md` + a `Terminal` log — **no actual .pcap files**. Excluded.
- **Netresec license note:** the 4SICS set asks for **CS3Sthlm / 4SICS** credit;
  this is already captured in the repo's existing attribution section.
- **GitHub re-host repos** (`markofu/pcaps`, `epiecs/packetlife-backup`,
  `researcher111/ARP-pcap-files`) lack explicit licenses. They are fine for
  *local, non-redistributed* validation fixtures (which is exactly our
  gitignored-`local-samples/` model), but should NOT be vendored into the tracked
  tree. Original attribution: Wireshark Foundation, Chris Sanders (PPA),
  Jeremy Stretch (PacketLife).

## Suggested fetch entries (mirror of bin/fetch-e2e-pcaps style)

```
# D3 ARP storm (canonical)
arp-storm.pcap            https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/arp-storm.pcap
220703_arp-storm.pcapng   https://gitlab.com/wireshark/wireshark/-/wikis/uploads/f59564719471dc67295224d1f18c4857/220703_arp-storm.pcapng
# D1 spoofing / poisoning (headline)
arpspoof.pcap             https://raw.githubusercontent.com/researcher111/ARP-pcap-files/master/arpspoof.pcap
# D2 gratuitous ARP
gratuitous-arp-hsrp.cap   https://raw.githubusercontent.com/epiecs/packetlife-backup/master/pcaps/gratuitous%20arp%20hsrp.cap
# benign baselines (no-false-positive)
ppa-arp.pcap              https://raw.githubusercontent.com/markofu/pcaps/master/PracticalPacketAnalysis/ppa-capture-files/arp.pcap
arp_pcap.pcapng.cap       https://raw.githubusercontent.com/epiecs/packetlife-backup/master/pcaps/arp_pcap.pcapng.cap
```

Compute and pin `sha256` for each after first download (the index requires a
checksum column), exactly as done for the 4SICS rows.

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep of ARP pcap sources (Wireshark wiki, Netresec, MTA.net, GitHub). Output exceeded token cap; pivoted to targeted search+fetch verification. |
| Perplexity perplexity_search | 2 | Ranked-URL discovery of Wireshark ARP samples and ettercap/arpspoof/MITM pcaps |
| WebSearch | 5 | Locating arppoison/ARP_Poison/ettercap/researcher111 captures and packetlife mirror |
| WebFetch | 11 | Verifying each direct download URL resolves to real binary pcap (status/type/size); enumerating repo file trees |
| Training data | 1 area | D-code → behavior mapping framing only; all file/URL claims are web-verified |

**Total MCP tool calls:** 3 (1 perplexity_research + 2 perplexity_search)
**Training data reliance:** low — every download URL was probed live with
WebFetch/WebSearch; "verified-live?" reflects actual probe results, and
unverifiable/gated items are explicitly flagged.
