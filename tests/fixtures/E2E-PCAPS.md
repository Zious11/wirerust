# E2E PCAP samples — index

The large packet captures used for **manual end-to-end validation** of the
analyzers are **not stored in git** (they exceed GitHub's 100 MB push limit and
a shared storage backend is still undecided — see `.factory/STATE.md`
`PCAP-CORPUS-001`). They live, gitignored, under `tests/fixtures/local-samples/`.

This file is the **tracked index** so any developer can reproduce the local
E2E set. To fetch/regenerate everything:

```bash
bin/fetch-e2e-pcaps
```

That downloads the real captures and regenerates the synthetic one into
`tests/fixtures/local-samples/`, verifying every checksum.

## Captures

| File | Size | sha256 | Source | Protocols | Validates |
|------|------|--------|--------|-----------|-----------|
| `4SICS-GeekLounge-151020.pcap` | 25 MB | `8c6ee02dc26b1b5298a7c9b4dc83cc779bd2a3219d5c5cbc51e3d4d325763bc2` | [Netresec 4SICS](https://www.netresec.com/?page=PCAP4SICS) | Modbus/502, DNP3, S7, HTTP, DNS, … | parse robustness / mixed-protocol scale; light Modbus |
| `4SICS-GeekLounge-151021.pcap` | 134 MB | `7365b0ea475b76bf79b207fd8f83baa45e4449aead5da6a9214bbcffbc5fa7de` | [Netresec 4SICS](https://www.netresec.com/?page=PCAP4SICS) | Modbus/502, DNP3, S7, HTTP, TLS, … | recon detection (FC 0x2B/0x11); throughput |
| `4SICS-GeekLounge-151022.pcap` | 200 MB | `82529c23906416dc73d7f1926a0d38b82527f1f2a7ff8c6f755ce3208feb9643` | [Netresec 4SICS](https://www.netresec.com/?page=PCAP4SICS) | Modbus/502 (heavy), DNP3, TLS, … | full Modbus detector set: writes (T0835/T0836/T1692.001), burst (T0806), recon (T0888); DoS finding-cap; determinism |
| `modbus-large.pcap` | ~7 KB | `1286603a7c83ca28de7eb46bc93271acd86ce3121f8fe695a744491cc22e5966` | synthetic — `tests/fixtures/mk_modbus_large_pcap.py` | Modbus/502 (5 crafted flows) | every Modbus detector class in isolation (recon, write-burst, coil/register/control writes, diagnostics DoS) |
| `dnp3dataset_capture.pcap` | ~2.6 MB | `72551ac30b30c80ee1a0a032e950648f1f70592642880656a1f8e4b5306e5b20` | [igbe/DNP3-Dataset-Plus-SnortRules](https://github.com/igbe/DNP3-Dataset-Plus-SnortRules) (no explicit LICENSE — local use only, not redistributed) | DNP3 over TCP (26058 packets, 402 hosts) | DNP3 detector set at scale: unexpected unsolicited-response (T0814, x539), ENABLE_UNSOLICITED (x546), DISABLE_UNSOLICITED, COLD_RESTART (x11), WRITE/parameter-modification (T0836); exercises grouped-collapse + `--mitre` tactic bucketing at scale (~0.02 s). **Note:** this capture exposed BUG-DNP3-CONTROL-OP-DETERMINISM-001 (non-deterministic `control_operation_counts`), fixed in v0.9.2. |
| `rsasnakeoil2.pcap` | ~24 KB | `f3f74008e2585d35479b7a234010a584803c240d82723f1f857bac0eb8a8db57` | [Wireshark SampleCaptures](https://wiki.wireshark.org/SampleCaptures) (no per-file license; public sample — credit Wireshark Foundation; not redistributed) | TLS/TCP (58 packets) | TLS weak-crypto detection: ServerHello+ClientHello SSL 3.0 (RFC 7568 prohibits SSLv3), ClientHello export/NULL/anonymous cipher suites (TLS_RSA_EXPORT_*). The only reliable classic-pcap TLS-handshake fixture found. |
| `dns-tunnel-iodine.pcap` | ~76 KB | `91fd221e07107507c8327a9d9487cd7f7531a1fd87cf543b1a76160ff0609b7b` | [elastic/examples](https://github.com/elastic/examples) (Apache-2.0; credit Elastic) | DNS over UDP (434 packets; 222 queries / 212 responses) | iodine DNS-tunneling capture. **Note:** wirerust currently produces 0 findings (DNS-tunneling detection not yet implemented) — retained as a benign-parse baseline AND a future-detector fixture. Tracked as a coverage/feature gap. |

> A tiny committed fixture, `tests/fixtures/modbus-write.pcap` (8 packets), is
> tracked in git and used by the automated test suite — it is **not** part of
> this local-only set.

## Direct download URLs (real captures)

| File | URL |
|------|-----|
| 151020 | `https://share.netresec.com/s/xYj2qCNbsLEAd6M/download/4SICS-GeekLounge-151020.pcap` |
| 151021 | `https://share.netresec.com/s/camL59aoxbCRyyZ/download/4SICS-GeekLounge-151021.pcap` |
| 151022 | `https://share.netresec.com/s/gw6Y2QzJHqDD5pr/download/4SICS-GeekLounge-151022.pcap` |
| `dnp3dataset_capture.pcap` | `https://raw.githubusercontent.com/igbe/DNP3-Dataset-Plus-SnortRules/master/dnp3dataset_capture.pcap` |
| `rsasnakeoil2.pcap` | `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/rsasnakeoil2.pcap` |
| `dns-tunnel-iodine.pcap` | `https://raw.githubusercontent.com/elastic/examples/master/Security%20Analytics/dns_tunnel_detection/dns-tunnel-iodine.pcap` |

### Link-only captures (cannot be auto-fetched)

These captures are indexed here for reference but are **not** included in the
auto-fetch script. They either require manual browser download (bot-blocked CDN)
or are too large for routine automated fetching.

| File | Approx. Size | sha256 | URL | Source | Notes |
|------|-------------|--------|-----|--------|-------|
| `dnscat2_dns_tunneling_1hr.pcap` | ~2.35 MB | from source / unverified | `https://www.activecountermeasures.com/wp-content/uploads/2021/06/dnscat2_dns_tunneling_1hr.pcap` | Active Countermeasures (public WordPress uploads) | **Manual download required** — Cloudflare bot-protection returns an HTML challenge page to automated curl/HEAD; must be fetched via a browser. SHA256 published by source but not independently verified by us. |
| `dnscat2_dns_tunneling_24hr.pcap` | ~82 MB | from source / unverified | `https://www.activecountermeasures.com/wp-content/uploads/2021/06/dnscat2_dns_tunneling_24hr.pcap` | Active Countermeasures (public WordPress uploads) | Same bot-block caveat as 1hr variant. Optional given size. |
| `maccdc2012_00000.pcap` | ~1 GB | unverified | `https://share.netresec.com/s/7qgDSGNGw2NY8ea/download/maccdc2012_00000.pcap` | Netresec public pcap collection (credit Netresec / maccdc) | Mixed-enterprise scale stressor (HTTP/TLS/DNS/SMB). Optional given 1 GB size; not included in standard fetch. |

## Attribution

The 4SICS Geek Lounge captures are from Netresec's public 4SICS ICS-lab
collection: <https://www.netresec.com/?page=PCAP4SICS>. Per the source's
request, **credit CS3Sthlm / 4SICS** if these captures are redistributed or
used in training material. They are not redistributed via this repo.

- **`dnp3dataset_capture.pcap`**: `igbe/DNP3-Dataset-Plus-SnortRules` (GitHub). No explicit
  LICENSE file — used locally for validation only, not redistributed. Credit: igbe (GitHub).
- **`rsasnakeoil2.pcap`**: Wireshark SampleCaptures wiki. No per-file license stated; public
  sample distributed by the Wireshark project. Used locally only, not redistributed.
  Credit: Wireshark Foundation.
- **`dns-tunnel-iodine.pcap`**: `elastic/examples` repository (Apache-2.0 license).
  Credit: Elastic.

## ARP captures (D1 spoof / D2 GARP / D3 storm / D12 MAC mismatch)

These captures exercise the v0.7.0 ARP Security Analyzer. Validated with
`target/release/wirerust analyze --arp <file>` (default flags unless noted).
They live gitignored under `tests/fixtures/local-samples/` — never committed.

### Captures

| File | Size | sha256 | Source | Protocols | Validates |
|------|------|--------|--------|-----------|-----------|
| `arp-storm.pcap` | 46 KB | `dc101ea9bfda59f56b54bfb949195c3f169032c045b47f98e6952a86933c1b8d` | [Wireshark SampleCaptures](https://wiki.wireshark.org/SampleCaptures) | ARP (622 req, 0 reply) | D3 storm fires at `--arp-storm-rate 10` (1 finding, source MAC `00:07:0D:AF:F4:54`); silent at default 50 fps threshold — burst rate in this cable-modem storm is ~10 fps |
| `gratuitous-arp-hsrp.cap` | 480 B | `e2fcc1276f31535d7e6bc5305e979ca1d5b83c7a0db1967d6334cd9b98afe7ad` | [PacketLife backup (epiecs/packetlife-backup)](https://github.com/epiecs/packetlife-backup) | ARP (6 GARP reply) | D2: 6 GARP findings fire (HSRP active-router sends sender_ip=10.0.0.6 in op=2 replies); no D1/D3 noise |
| `arpspoof.pcap` | 14 MB | `0ce605556689edec01ef50703df7cc88c97a0d1731c4938d54cabcb28a71837a` | [researcher111/ARP-pcap-files](https://github.com/researcher111/ARP-pcap-files) | ARP (50 pkts), TCP/TLS/UDP/ICMP (16 234 total) | D1: 2 spoof findings (IP→MAC rebind for 192.168.1.1); D12: 5 L2/L3 MAC mismatch findings; 1 decode-error skipped; no D2/D3 |
| `ppa-arp.pcap` | 144 B | `ea22826b52c96a2038d1c44eb0e7c35dbf40335f82a20cf94ef70bb821033f65` | [markofu/pcaps (PracticalPacketAnalysis)](https://github.com/markofu/pcaps) | ARP (1 req + 1 reply) | Benign baseline: 0 findings, 2 bindings tracked — no false positives on clean ARP request/reply pair |
| `arp-baseline-16pkt.cap` | 2.2 KB | `d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25` | [PacketLife backup (epiecs/packetlife-backup)](https://github.com/epiecs/packetlife-backup) | ARP (pcapng wrapped in .cap extension) | **Format note:** wirerust accepts this file via content-based magic-byte detection (pcapng SHB magic `0x0A0D0D0A`; BC-2.12.011 / STORY-127 / ADR-009 Decision 11). The `.cap` extension is ignored; the file is detected and parsed by the pcapng reader stack (STORY-123..127), yielding 16 ARP packets. Resolves C-2. |

### Direct download URLs (ARP captures)

| File | URL |
|------|-----|
| `arp-storm.pcap` | `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/arp-storm.pcap` |
| `gratuitous-arp-hsrp.cap` | `https://raw.githubusercontent.com/epiecs/packetlife-backup/master/pcaps/gratuitous%20arp%20hsrp.cap` |
| `arpspoof.pcap` | `https://raw.githubusercontent.com/researcher111/ARP-pcap-files/master/arpspoof.pcap` |
| `ppa-arp.pcap` | `https://raw.githubusercontent.com/markofu/pcaps/master/PracticalPacketAnalysis/ppa-capture-files/arp.pcap` |
| `arp-baseline-16pkt.cap` | `https://raw.githubusercontent.com/epiecs/packetlife-backup/master/pcaps/arp_pcap.pcapng.cap` |

### Attribution

- **`arp-storm.pcap`**: Wireshark Foundation public SampleCaptures collection. No per-file
  license stated; distributed as a public sample. Credit: Wireshark Foundation.
- **`gratuitous-arp-hsrp.cap`** and **`arp-baseline-16pkt.cap`**: PacketLife.net captures by
  Jeremy Stretch, mirrored via `epiecs/packetlife-backup` (no explicit license in mirror repo).
  Used locally for validation only — not redistributed. Credit: PacketLife.net / Jeremy Stretch.
- **`ppa-arp.pcap`**: Practical Packet Analysis sample by Chris Sanders, re-hosted in
  `markofu/pcaps` (no explicit license). Used locally for validation only — not redistributed.
  Credit: Chris Sanders.
- **`arpspoof.pcap`**: `researcher111/ARP-pcap-files` (no LICENSE file — all-rights-reserved).
  Used locally for validation only — not redistributed. Credit: researcher111 (GitHub).

## Coverage gaps and notes

- **pcapng now supported (STORY-123..127):** Wirerust accepts pcapng files via the pcapng
  reader stack (STORY-123..127 / BC-2.12.011 / ADR-009). Content-based magic-byte detection
  (resolve_targets, STORY-127) accepts any file whose first 4 bytes are the pcapng SHB magic
  `0x0A0D0D0A`, regardless of extension (resolves C-2: `arp-baseline-16pkt.cap` now accepted
  and parses to 16 ARP packets). Large TLS-heavy captures previously blocked by pcapng format
  are now candidates for the E2E corpus.
- **DNS tunneling detection not yet implemented:** `dns-tunnel-iodine.pcap` (and the
  dnscat2 captures above) currently produce 0 findings. They are retained as benign-parse
  baselines and future-detector fixtures for whenever DNS-tunneling analysis is added.
- **Full research write-up:** The evaluation methodology, candidate evaluation, and rationale
  for all selections above is documented at `.factory/research/e2e-pcap-candidates.md`.

## Adding a capture

1. Drop the `.pcap` in `tests/fixtures/local-samples/` (gitignored).
2. Add a row to the table above with its `sha256` (`shasum -a 256 <file>`),
   size, source URL (or generator), protocols, and what it validates.
3. Add its URL + checksum to `bin/fetch-e2e-pcaps` so others can fetch it.

This index is the lightweight precursor to the full `PCAP-CORPUS-001` corpus
(orphan-branch manifest + tiered/cached CI runner) — once a storage backend is
chosen, these rows migrate into that manifest.
