---
title: "PCAPNG Packet Count Verification — arp-baseline-16pkt.cap"
phase: F6
date: 2026-06-21
fixture: tests/fixtures/local-samples/arp-baseline-16pkt.cap
sha256: d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25
wirerust-commit: develop HEAD
verdict: CORRECT
---

# PCAPNG Packet Count Verification

## Fixture

- Path: `tests/fixtures/local-samples/arp-baseline-16pkt.cap`
- Size: 2240 bytes
- SHA-256: `d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25`
- Format: genuine pcapng (magic `0x0A0D0D0A`), despite `.cap` extension
- Capture tool: Dumpcap 1.10.6 on Linux 3.13.0-24-generic

## Ground Truth: Manual Block Walk

No external pcap tools (capinfos, tshark, scapy, dpkt) are present in this environment. Ground truth was derived by manually walking all pcapng blocks via Python using the raw byte stream (`struct.unpack` on block-type and block-length fields at each offset), verifying trailing-length integrity for every block.

### Complete Block Inventory

| # | Offset (hex) | Offset (dec) | Block Type | Length | Trailing OK |
|---|-------------|-------------|------------|--------|-------------|
| 1 | 0x0000 | 0 | SHB (Section Header) | 108 | Yes |
| 2 | 0x006c | 108 | IDB (Interface Description) | 68 | Yes |
| 3 | 0x00b0 | 176 | EPB[0] | 92 | Yes |
| 4 | 0x010c | 268 | EPB[1] | 92 | Yes |
| 5 | 0x0168 | 360 | EPB[2] | 388 | Yes |
| 6 | 0x02ec | 748 | EPB[3] | 92 | Yes |
| 7 | 0x0348 | 840 | EPB[4] | 92 | Yes |
| 8 | 0x03a4 | 932 | EPB[5] | 92 | Yes |
| 9 | 0x0400 | 1024 | EPB[6] | 92 | Yes |
| 10 | 0x045c | 1116 | EPB[7] | 92 | Yes |
| 11 | 0x04b8 | 1208 | EPB[8] | 92 | Yes |
| 12 | 0x0514 | 1300 | EPB[9] | 92 | Yes |
| 13 | 0x0570 | 1392 | EPB[10] | 92 | Yes |
| 14 | 0x05cc | 1484 | EPB[11] | 92 | Yes |
| 15 | 0x0628 | 1576 | EPB[12] | 92 | Yes |
| 16 | 0x0684 | 1668 | EPB[13] | 388 | Yes |
| 17 | 0x0808 | 2056 | EPB[14] | 92 | Yes |
| 18 | 0x0864 | 2148 | EPB[15] | 92 | Yes |

**Total blocks: 18**
- SHB: 1
- IDB: 1
- EPB: 16
- SPB: 0
- NRB/ISB: 0

**Packet-bearing blocks (EPB + SPB): 16**

### Ethertype Breakdown of All 16 EPBs

| EPB index | Offset | cap_len | Ethertype | Protocol |
|-----------|--------|---------|-----------|----------|
| EPB[0] | 176 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[1] | 268 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[2] | 360 | 354 | 0x0154 (= 340 dec) | 802.3 LLC frame (Cisco CDP multicast, dst `01:00:0c:cc:cc:cc`) |
| EPB[3] | 748 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[4] | 840 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[5] | 932 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[6] | 1024 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[7] | 1116 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[8] | 1208 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[9] | 1300 | 60 | 0x0806 | ARP |
| EPB[10] | 1392 | 60 | 0x0806 | ARP |
| EPB[11] | 1484 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[12] | 1576 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[13] | 1668 | 354 | 0x0154 (= 340 dec) | 802.3 LLC frame (Cisco CDP multicast, dst `01:00:0c:cc:cc:cc`) |
| EPB[14] | 2056 | 60 | 0x9000 | Ethernet Loopback (ECTP) |
| EPB[15] | 2148 | 60 | 0x9000 | Ethernet Loopback (ECTP) |

**Note on 0x0154:** A value of 340 decimal is below the IEEE 802.3 ethertype threshold (1536 = 0x0600), so this is a length field, not an ethertype. These are 802.3 LLC-encapsulated frames directed to the Cisco multicast address `01:00:0c:cc:cc:cc` — CDP (Cisco Discovery Protocol) or PVST+ frames. They carry no IP layer.

**Summary by protocol:**
- Ethernet Loopback 0x9000 (ECTP): 12 frames
- 802.3 LLC / Cisco CDP (0x0154): 2 frames
- ARP (0x0806): 2 frames
- IPv4/IPv6: 0 frames

## wirerust Reader Output

### Default run (`--no-color`)
```
Warning: failed to decode packet (No IP layer found). Further errors counted silently.
WIRERUST TRIAGE REPORT
────────────────────────────────────────
  Packets: 0  Bytes: 0  Hosts: 0
  Skipped: 14 packets (decode errors)
```

### With `--arp --json`
```json
{
  "analyzers": [
    {
      "analyzer_name": "ARP",
      "detail": {
        "frames_analyzed": 2,
        "request_count": 1,
        "reply_count": 1,
        "bindings_tracked": 2,
        ...
      },
      "packets_analyzed": 2
    }
  ],
  "summary": {
    "skipped_packets": 14,
    "total_packets": 0
  }
}
```

## Reconciliation

| Category | Ground Truth | wirerust | Match |
|----------|-------------|---------|-------|
| Total packet-bearing blocks | 16 | 16 (14 skipped + 2 to ARP analyzer) | Yes |
| Loopback (0x9000) | 12 | 12 (skipped: no IP layer) | Yes |
| 802.3 LLC/CDP (0x0154) | 2 | 2 (skipped: no IP layer) | Yes |
| ARP (0x0806) | 2 | 2 (processed by ARP analyzer) | Yes |
| IP packets | 0 | 0 | Yes |

**All 16 EPBs are accounted for.** wirerust processes every packet-bearing block:
- 14 non-IP, non-ARP frames hit the "No IP layer found" skip path and are counted in `skipped_packets`.
- 2 ARP frames are dispatched to the ARP analyzer (not counted in `total_packets`, which is IP-only, and not counted in `skipped_packets`).

## Verdict: CORRECT — No Reader Bug

**The wirerust pcapng reader reads all 16 packet-bearing blocks correctly.**

### Explanation of the "16pkt" Name vs. "14 skipped" Report

The "16" in `arp-baseline-16pkt.cap` correctly refers to the 16 Enhanced Packet Blocks present in the file. The smoke test observation of "14 skipped" and "0 IP packets" is NOT evidence of dropped packets.

The apparent discrepancy arises from two separate accounting tracks in wirerust:

1. **`skipped_packets` (=14):** Counts frames that reached the IP-decode path and failed — the 12 Ethernet Loopback frames (ethertype 0x9000) and 2 802.3 LLC/CDP frames (802.3 length field 0x0154). None of these carry an IP layer; the skip is correct behavior.

2. **`total_packets` (=0):** Counts only successfully decoded IP-layer packets. This fixture has zero IPv4/IPv6 frames, so this counter is legitimately zero.

3. **ARP frames (=2):** Processed by the ARP analyzer via a separate dispatch path, reported in `analyzers[].packets_analyzed`. They are neither skipped nor counted as IP packets.

The naming "16pkt" matches the actual EPB count (16). The report showing 14 skipped + 2 ARP-analyzed = 16 total, which equals the ground-truth EPB count exactly.

### Why the Smoke Test Was Misleading

The CLI smoke test (`"14 skipped as No IP layer found, 0 IP"`) was run without `--arp`, so the ARP pathway's packet count was not visible. The raw summary numbers (0 IP, 14 skipped) looked like 2 packets were missing, but the ARP analyzer's `packets_analyzed: 2` accounts for the remaining 2 frames. Running with `--arp --json` exposes this cleanly.

## No Action Required

This is benign. The fixture name is accurate, the reader is correct, and the report numbers are correct given wirerust's IP-centric packet counter semantics. The `arp-baseline-16pkt.cap` fixture is valid for its intended use (ARP analyzer testing with `--arp`).
