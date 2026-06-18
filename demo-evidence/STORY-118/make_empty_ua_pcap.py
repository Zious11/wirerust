#!/usr/bin/env python3
"""
Craft a pcap with N HTTP/1.1 GET requests, each with:
  - User-Agent:\r\n  (present-but-empty — triggers empty-UA finding)
  - Host: example.com\r\n  (avoids missing-Host co-finding)
  - Distinct URIs (/req001 .. /reqN)

Each request is a separate TCP flow (distinct source ports) so that
the TCP reassembler delivers each request independently to the HTTP
analyzer. Using persistent connections on one flow would work too,
but separate flows are simpler to reason about.

Output: empty_ua_flood.pcap
"""

import struct
import socket

N = 25  # number of requests → produces N empty-UA findings

# PCAP global header: magic, version, snaplen, DLT_EN10MB=1
PCAP_GLOBAL_HEADER = struct.pack(
    "<IHHiIII",
    0xA1B2C3D4,  # magic
    2, 4,          # version
    0,             # thiszone
    0,             # sigfigs
    65535,         # snaplen
    1,             # DLT_EN10MB
)


def ethernet_frame(src_mac: bytes, dst_mac: bytes, payload: bytes) -> bytes:
    return dst_mac + src_mac + b"\x08\x00" + payload  # IPv4


def ipv4_header(src_ip: str, dst_ip: str, proto: int, payload_len: int) -> bytes:
    total_len = 20 + payload_len
    hdr = struct.pack(
        "!BBHHHBBH4s4s",
        0x45,       # version=4, IHL=5
        0,          # DSCP
        total_len,
        0,          # identification
        0x40 << 8,  # DF flag, fragment offset=0
        64,         # TTL
        proto,      # protocol
        0,          # checksum placeholder
        socket.inet_aton(src_ip),
        socket.inet_aton(dst_ip),
    )
    # compute checksum
    cs = checksum(hdr)
    hdr = hdr[:10] + struct.pack("!H", cs) + hdr[12:]
    return hdr


def checksum(data: bytes) -> int:
    if len(data) % 2:
        data += b"\x00"
    s = 0
    for i in range(0, len(data), 2):
        w = (data[i] << 8) + data[i + 1]
        s += w
    while s >> 16:
        s = (s & 0xFFFF) + (s >> 16)
    return ~s & 0xFFFF


def tcp_header(
    src_port: int,
    dst_port: int,
    seq: int,
    ack: int,
    flags: int,
    src_ip: str,
    dst_ip: str,
    payload: bytes,
) -> bytes:
    data_offset = 5 << 4  # 20 bytes header, no options
    window = 65535
    hdr = struct.pack(
        "!HHIIBBHHH",
        src_port,
        dst_port,
        seq,
        ack,
        data_offset,
        flags,
        window,
        0,  # checksum placeholder
        0,  # urgent pointer
    )
    # pseudo-header for checksum
    pseudo = (
        socket.inet_aton(src_ip)
        + socket.inet_aton(dst_ip)
        + struct.pack("!BBH", 0, 6, len(hdr) + len(payload))
    )
    cs = checksum(pseudo + hdr + payload)
    hdr = hdr[:16] + struct.pack("!H", cs) + hdr[18:]
    return hdr


def pcap_record(ts_sec: int, ts_usec: int, data: bytes) -> bytes:
    orig_len = len(data)
    return struct.pack("<IIII", ts_sec, ts_usec, orig_len, orig_len) + data


SRC_IP = "192.168.1.100"
DST_IP = "10.0.0.1"
DST_PORT = 80
SRC_MAC = bytes([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])
DST_MAC = bytes([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF])

records = []

for i in range(N):
    src_port = 10000 + i
    uri = f"/req{i + 1:03d}"
    http_req = (
        f"GET {uri} HTTP/1.1\r\n"
        f"Host: example.com\r\n"
        f"User-Agent:\r\n"   # present-but-empty: triggers empty-UA finding
        f"Connection: close\r\n"
        f"\r\n"
    ).encode()

    # SYN
    tcp_syn = tcp_header(src_port, DST_PORT, 1000, 0, 0x02, SRC_IP, DST_IP, b"")
    ip_syn = ipv4_header(SRC_IP, DST_IP, 6, len(tcp_syn))
    eth_syn = ethernet_frame(SRC_MAC, DST_MAC, ip_syn + tcp_syn)
    records.append(pcap_record(1700000000 + i, 0, eth_syn))

    # SYN-ACK
    tcp_synack = tcp_header(DST_PORT, src_port, 2000, 1001, 0x12, DST_IP, SRC_IP, b"")
    ip_synack = ipv4_header(DST_IP, SRC_IP, 6, len(tcp_synack))
    eth_synack = ethernet_frame(DST_MAC, SRC_MAC, ip_synack + tcp_synack)
    records.append(pcap_record(1700000000 + i, 100, eth_synack))

    # ACK (client)
    tcp_ack = tcp_header(src_port, DST_PORT, 1001, 2001, 0x10, SRC_IP, DST_IP, b"")
    ip_ack = ipv4_header(SRC_IP, DST_IP, 6, len(tcp_ack))
    eth_ack = ethernet_frame(SRC_MAC, DST_MAC, ip_ack + tcp_ack)
    records.append(pcap_record(1700000000 + i, 200, eth_ack))

    # HTTP GET request (PSH+ACK)
    tcp_data = tcp_header(src_port, DST_PORT, 1001, 2001, 0x18, SRC_IP, DST_IP, http_req)
    ip_data = ipv4_header(SRC_IP, DST_IP, 6, len(tcp_data) + len(http_req))
    eth_data = ethernet_frame(SRC_MAC, DST_MAC, ip_data + tcp_data + http_req)
    records.append(pcap_record(1700000000 + i, 300, eth_data))

    # FIN+ACK (client closes)
    fin_seq = 1001 + len(http_req)
    tcp_fin = tcp_header(src_port, DST_PORT, fin_seq, 2001, 0x11, SRC_IP, DST_IP, b"")
    ip_fin = ipv4_header(SRC_IP, DST_IP, 6, len(tcp_fin))
    eth_fin = ethernet_frame(SRC_MAC, DST_MAC, ip_fin + tcp_fin)
    records.append(pcap_record(1700000000 + i, 400, eth_fin))

import os
out = os.path.join(os.path.dirname(__file__), "empty_ua_flood.pcap")
with open(out, "wb") as f:
    f.write(PCAP_GLOBAL_HEADER)
    for r in records:
        f.write(r)

print(f"Wrote {out} ({N} flows, {len(records)} packets)")
