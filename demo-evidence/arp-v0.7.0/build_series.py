#!/usr/bin/env python3
"""Dependency-free PCAP builder for the wirerust v0.7.0 ARP analyzer demo series.

Classic libpcap format (magic 0xa1b2c3d4, little-endian), LINKTYPE_ETHERNET=1.
One .pcap per scenario, written into this directory.

ARP frame layout (RFC 826, Ethernet/IPv4):
  Ethernet [dst6|src6|0x0806]
  ARP      [htype=0x0001|ptype=0x0800|hlen=6|plen=4|oper(2)|sha6|spa4|tha6|tpa4]
"""
import struct
import os

OUT = os.path.dirname(os.path.abspath(__file__))


def pcap_global(linktype=1):
    # magic, vmaj, vmin, thiszone, sigfigs, snaplen, network
    return struct.pack("<IHHiIII", 0xA1B2C3D4, 2, 4, 0, 0, 65535, linktype)


def pcap_rec(ts_sec, ts_usec, data, orig_len=None):
    # caplen may be < orig_len for snaplen-truncated frames
    caplen = len(data)
    if orig_len is None:
        orig_len = caplen
    return struct.pack("<IIII", ts_sec, ts_usec, caplen, orig_len) + data


def mac(s):
    return bytes(int(x, 16) for x in s.split(":"))


def ip(s):
    return bytes(int(x) for x in s.split("."))


def eth(dst, src, ethertype=0x0806):
    return mac(dst) + mac(src) + struct.pack(">H", ethertype)


def arp_body(op, sha, spa, tha, tpa, htype=1, ptype=0x0800, hlen=6, plen=4):
    h = struct.pack(">HHBBH", htype, ptype, hlen, plen, op)
    return h + mac(sha) + ip(spa) + mac(tha) + ip(tpa)


def eth_arp(dst, src, op, sha, spa, tha, tpa, **kw):
    return eth(dst, src, 0x0806) + arp_body(op, sha, spa, tha, tpa, **kw)


def write(name, records, linktype=1):
    path = os.path.join(OUT, name)
    with open(path, "wb") as f:
        f.write(pcap_global(linktype))
        for r in records:
            f.write(r)
    print(f"wrote {name} ({len(records)} frame(s))")


BCAST = "FF:FF:FF:FF:FF:FF"
ZERO = "00:00:00:00:00:00"

# ---------------------------------------------------------------------------
# 1. benign-baseline: a few normal request/reply exchanges, distinct IP<->MAC.
#    No rebinds, no conflicts, no GARP, well-formed -> EXPECT zero findings.
# ---------------------------------------------------------------------------
recs = []
# host A (mac ..A1, .10) asks for .20; host B (mac ..B2) replies. Then A asks for .30.
recs.append(pcap_rec(1000, 0, eth_arp(BCAST, "00:11:22:33:44:A1", 1,
                                      "00:11:22:33:44:A1", "192.168.10.10",
                                      ZERO, "192.168.10.20")))
recs.append(pcap_rec(1000, 500, eth_arp("00:11:22:33:44:A1", "00:11:22:33:44:B2", 2,
                                        "00:11:22:33:44:B2", "192.168.10.20",
                                        "00:11:22:33:44:A1", "192.168.10.10")))
recs.append(pcap_rec(1001, 0, eth_arp(BCAST, "00:11:22:33:44:A1", 1,
                                      "00:11:22:33:44:A1", "192.168.10.10",
                                      ZERO, "192.168.10.30")))
recs.append(pcap_rec(1001, 500, eth_arp("00:11:22:33:44:A1", "00:11:22:33:44:C3", 2,
                                        "00:11:22:33:44:C3", "192.168.10.30",
                                        "00:11:22:33:44:A1", "192.168.10.10")))
write("01_benign-baseline.pcap", recs)

# ---------------------------------------------------------------------------
# 2. d1-spoof: sender IP 192.168.1.10 rebinds across 4 different MACs within 60s.
#    Replies (op=2), eth_src == sha (no D12), target unicast (no GARP).
#    EXPECT D1 spoof: MEDIUM, MEDIUM, then HIGH (rebind_count == threshold 3 -> Likely).
# ---------------------------------------------------------------------------
recs = []
spoof_ip = "192.168.1.10"
spoof_macs = ["AA:BB:CC:DD:EE:01", "AA:BB:CC:DD:EE:02",
              "AA:BB:CC:DD:EE:03", "AA:BB:CC:DD:EE:04"]
for i, m in enumerate(spoof_macs):
    recs.append(pcap_rec(2000 + i, 0, eth_arp(BCAST, m, 2, m, spoof_ip,
                                              ZERO, "192.168.1.1")))
write("02_d1-spoof.pcap", recs)

# ---------------------------------------------------------------------------
# 3. d2-garp-benign: a single gratuitous ARP (sender_ip == target_ip), no prior
#    binding. EXPECT D2 GARP, LOW, no MITRE.
# ---------------------------------------------------------------------------
recs = []
recs.append(pcap_rec(3000, 0, eth_arp(BCAST, "AA:BB:CC:DD:EE:FF", 1,
                                      "AA:BB:CC:DD:EE:FF", "192.168.5.1",
                                      ZERO, "192.168.5.1")))
write("03_d2-garp-benign.pcap", recs)

# ---------------------------------------------------------------------------
# 4. d2-garp-conflict: establish 192.168.5.1 -> MAC_A via a normal reply, then a
#    GARP announcing 192.168.5.1 with MAC_B. EXPECT D2 GARP upgraded to MEDIUM +
#    co-emitted D1, both with T0830/T1557.002.
# ---------------------------------------------------------------------------
recs = []
mac_a = "AA:BB:CC:DD:EE:0A"
mac_b = "AA:BB:CC:DD:EE:0B"
# normal reply establishing 192.168.5.1 -> MAC_A (target unicast -> not GARP)
recs.append(pcap_rec(4000, 0, eth_arp(BCAST, mac_a, 2, mac_a, "192.168.5.1",
                                      ZERO, "192.168.5.99")))
# GARP (sender_ip == target_ip) announcing same IP with MAC_B -> conflict
recs.append(pcap_rec(4001, 0, eth_arp(BCAST, mac_b, 1, mac_b, "192.168.5.1",
                                      ZERO, "192.168.5.1")))
write("04_d2-garp-conflict.pcap", recs)

# ---------------------------------------------------------------------------
# 5. d3-storm: flood of ARP frames from one source MAC in the same second.
#    12 frames, run with --arp-storm-rate 10. Vary sender IP to avoid D1 rebind.
#    EXPECT D3 storm, MEDIUM, no MITRE (T0814 withheld).
# ---------------------------------------------------------------------------
recs = []
storm_mac = "DE:AD:BE:EF:00:01"
for i in range(12):
    spa = "10.20.0.%d" % (i + 1)
    recs.append(pcap_rec(5000, i, eth_arp(BCAST, storm_mac, 1, storm_mac, spa,
                                          ZERO, "10.20.0.254")))
write("05_d3-storm.pcap", recs)

# ---------------------------------------------------------------------------
# 6. d11-malformed: an ARP frame with a bad fixed header (htype=0x0006 token-ring
#    + hlen=8), full length. Non-Ethernet/IPv4 -> EXPECT D11 malformed, LOW.
#    sha/tha are 8 bytes to match hlen=8 so the frame is structurally complete.
# ---------------------------------------------------------------------------
recs = []
# Build raw ARP with htype=0x0006, hlen=8 (8-byte hw addrs), plen=4.
htype, ptype, hlen, plen, op = 0x0006, 0x0800, 8, 4, 1
sha8 = bytes.fromhex("0011223344556677")
tha8 = bytes.fromhex("0000000000000000")
spa = ip("172.16.0.1")
tpa = ip("172.16.0.254")
arp_bad = struct.pack(">HHBBH", htype, ptype, hlen, plen, op) + sha8 + spa + tha8 + tpa
frame = eth(BCAST, "00:AA:BB:CC:DD:EE", 0x0806) + arp_bad
recs.append(pcap_rec(6000, 0, frame))
write("06_d11-malformed.pcap", recs)

# ---------------------------------------------------------------------------
# 7. d12-mismatch: ARP reply where Ethernet src MAC != ARP sender hw addr.
#    EXPECT D12 L2/L3 mismatch, MEDIUM, T0830/T1557.002.
# ---------------------------------------------------------------------------
recs = []
recs.append(pcap_rec(7000, 0, eth_arp("00:11:22:33:44:55", "DE:AD:00:00:00:01", 2,
                                      "BE:EF:00:00:00:02", "192.168.7.7",
                                      "00:11:22:33:44:55", "192.168.7.1")))
write("07_d12-mismatch.pcap", recs)

# ---------------------------------------------------------------------------
# 8. canonical-rfc826: textbook RFC 826 ARP request (htype=1, ptype=0x0800,
#    hlen=6, plen=4). EXPECT clean decode, binding tracked, zero findings.
# ---------------------------------------------------------------------------
recs = []
recs.append(pcap_rec(8000, 0, eth_arp(BCAST, "00:53:00:00:00:01", 1,
                                      "00:53:00:00:00:01", "10.0.0.1",
                                      ZERO, "10.0.0.2")))
write("08_canonical-rfc826.pcap", recs)

# ---------------------------------------------------------------------------
# 9. vlan-tagged (F-1 fix): VLAN-tagged (802.1Q, 0x8100) ARP, benign, but
#    snaplen-truncated (variable section cut off). EXPECT NOT flagged as a
#    false-positive D11 -- should be a clean/truncated decode, no spurious
#    malformed finding.
#    Ethernet [dst6|src6|0x8100|TCI(2)|0x0806] then ARP fixed header (valid
#    htype/ptype/hlen/plen) followed by a TRUNCATED variable section.
# ---------------------------------------------------------------------------
recs = []
vlan_dst = mac(BCAST)
vlan_src = mac("00:CC:00:00:00:01")
tci = struct.pack(">H", 0x000A)              # VLAN id 10, prio 0
qtag = struct.pack(">H", 0x8100) + tci
inner_ethertype = struct.pack(">H", 0x0806)  # ARP
# Valid ARP fixed header (htype=1 Ether, ptype=0x0800 IPv4, hlen=6, plen=4, op=1)
arp_fixed = struct.pack(">HHBBH", 1, 0x0800, 6, 4, 1)
# Begin the variable section but TRUNCATE it (only sha + spa + 2 bytes of tha).
sha = mac("00:CC:00:00:00:01")
spa = ip("192.168.99.5")
partial_tha = bytes.fromhex("0000")          # only 2 of 6 target hw addr bytes
truncated_arp = arp_fixed + sha + spa + partial_tha
frame = vlan_dst + vlan_src + qtag + inner_ethertype + truncated_arp
# orig_len reflects the full untruncated on-wire size (snaplen cut the capture).
full_len = len(vlan_dst + vlan_src + qtag + inner_ethertype) + 8 + 6 + 4 + 6 + 4
recs.append(pcap_rec(9000, 0, frame, orig_len=full_len))
write("09_vlan-tagged.pcap", recs)

print("\nAll ARP demo pcaps built in", OUT)
