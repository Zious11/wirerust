#!/usr/bin/env python3
"""
make_holdout_fixtures.py — Generate holdout PCAP fixtures for the wirerust
information-asymmetric holdout evaluation.

Built INDEPENDENTLY from the authoritative protocol specifications:
  - BACnet/IP: UDP port 47808 (0xBAC0), ASHRAE 135-2016 Annex J §J.2.1.
    BVLC header: byte0=0x81 (BACnet/IP), byte1=function, bytes2-3=length.
  - ISO-on-TCP / TPKT: TCP port 102 (S7comm / MMS / ICCP), RFC 1006.
  - DNS: UDP/TCP port 53, RFC 1035 §4.2.1.

All pcaps are classic libpcap (magic 0xa1b2c3d4, little-endian, microsecond
timestamps), LINKTYPE_ETHERNET=1. Ethernet II + IPv4 + TCP/UDP with correct
lengths and checksums.
"""

import struct
import os

PCAP_MAGIC_LE = 0xA1B2C3D4
LINKTYPE_ETHERNET = 1

OUT_DIR = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# pcap framing
# ---------------------------------------------------------------------------

def pcap_global_header() -> bytes:
    return struct.pack("<IHHiIII", PCAP_MAGIC_LE, 2, 4, 0, 0, 65535, LINKTYPE_ETHERNET)


def pcap_record(ts_sec: int, frame: bytes) -> bytes:
    return struct.pack("<IIII", ts_sec, 0, len(frame), len(frame)) + frame


# ---------------------------------------------------------------------------
# checksums
# ---------------------------------------------------------------------------

def ones_complement(data: bytes) -> int:
    if len(data) % 2:
        data += b"\x00"
    s = sum(struct.unpack("!%dH" % (len(data) // 2), data))
    s = (s >> 16) + (s & 0xFFFF)
    s += (s >> 16)
    return (~s) & 0xFFFF


# ---------------------------------------------------------------------------
# L2 / L3 / L4 builders
# ---------------------------------------------------------------------------

MAC_A = bytes([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])
MAC_B = bytes([0x00, 0x66, 0x77, 0x88, 0x99, 0xAA])
MAC_BCAST = bytes([0xFF] * 6)

IP_A = bytes([192, 168, 1, 10])
IP_B = bytes([192, 168, 1, 100])
IP_C = bytes([192, 168, 1, 20])
IP_BCAST = bytes([192, 168, 1, 255])


def eth(src_mac: bytes, dst_mac: bytes) -> bytes:
    return dst_mac + src_mac + bytes([0x08, 0x00])  # IPv4


def ipv4(src_ip: bytes, dst_ip: bytes, total_len: int, proto: int, ident: int = 0x1234) -> bytes:
    hdr = bytearray([
        0x45, 0x00,
        (total_len >> 8) & 0xFF, total_len & 0xFF,
        (ident >> 8) & 0xFF, ident & 0xFF,
        0x40, 0x00,          # flags=DF
        64, proto,
        0x00, 0x00,          # checksum placeholder
    ]) + src_ip + dst_ip
    ck = ones_complement(bytes(hdr))
    hdr[10] = (ck >> 8) & 0xFF
    hdr[11] = ck & 0xFF
    return bytes(hdr)


def tcp(src_port, dst_port, seq, ack, flags, payload, src_ip, dst_ip) -> bytes:
    offset_flags = (5 << 12) | flags
    hdr = struct.pack("!HHIIHHHH", src_port, dst_port, seq, ack,
                      offset_flags, 65535, 0, 0)  # checksum=0 placeholder, urg=0
    pseudo = src_ip + dst_ip + bytes([0, 6]) + struct.pack("!H", 20 + len(payload))
    ck = ones_complement(pseudo + hdr + payload)
    hdr = hdr[:16] + struct.pack("!H", ck) + hdr[18:]
    return hdr + payload


def udp(src_port, dst_port, payload, src_ip, dst_ip) -> bytes:
    length = 8 + len(payload)
    hdr = struct.pack("!HHHH", src_port, dst_port, length, 0)
    pseudo = src_ip + dst_ip + bytes([0, 17]) + struct.pack("!H", length)
    ck = ones_complement(pseudo + hdr + payload)
    if ck == 0:
        ck = 0xFFFF
    hdr = struct.pack("!HHHH", src_port, dst_port, length, ck)
    return hdr + payload


# TCP flags
SYN, ACK, FIN, PSH = 0x02, 0x10, 0x01, 0x08
SYN_ACK = SYN | ACK
FIN_ACK = FIN | ACK
PSH_ACK = PSH | ACK


def tcp_frame(smac, dmac, sip, dip, sport, dport, seq, ack, flags, payload=b""):
    seg = tcp(sport, dport, seq, ack, flags, payload, sip, dip)
    ip = ipv4(sip, dip, 20 + len(seg), 6)
    return eth(smac, dmac) + ip + seg


def udp_frame(smac, dmac, sip, dip, sport, dport, payload):
    seg = udp(sport, dport, payload, sip, dip)
    ip = ipv4(sip, dip, 20 + len(seg), 17)
    return eth(smac, dmac) + ip + seg


# ---------------------------------------------------------------------------
# TCP open+close flow (SYN, SYN-ACK, ACK, ..., FIN-ACK, FIN-ACK, ACK)
# Produces a flow that OPENS and CLOSES.
# ---------------------------------------------------------------------------

def tcp_open_close(t0, sport, dport, sip=IP_A, dip=IP_B, smac=MAC_A, dmac=MAC_B):
    """Return list of (ts, frame) for a clean SYN..FIN handshake teardown."""
    pkts = []
    cseq, sseq = 1000, 5000
    # 1. SYN  A->B
    pkts.append((t0 + 0, tcp_frame(smac, dmac, sip, dip, sport, dport, cseq, 0, SYN)))
    cseq += 1
    # 2. SYN-ACK  B->A
    pkts.append((t0 + 1, tcp_frame(dmac, smac, dip, sip, dport, sport, sseq, cseq, SYN_ACK)))
    sseq += 1
    # 3. ACK  A->B (handshake complete)
    pkts.append((t0 + 2, tcp_frame(smac, dmac, sip, dip, sport, dport, cseq, sseq, ACK)))
    # 4. FIN-ACK  A->B (initiate close)
    pkts.append((t0 + 3, tcp_frame(smac, dmac, sip, dip, sport, dport, cseq, sseq, FIN_ACK)))
    cseq += 1
    # 5. FIN-ACK  B->A (close other side)
    pkts.append((t0 + 4, tcp_frame(dmac, smac, dip, sip, dport, sport, sseq, cseq, FIN_ACK)))
    sseq += 1
    # 6. ACK  A->B (final ack, flow fully closed)
    pkts.append((t0 + 5, tcp_frame(smac, dmac, sip, dip, sport, dport, cseq, sseq, ACK)))
    return pkts


# ---------------------------------------------------------------------------
# BACnet/IP BVLC payloads (ASHRAE 135 Annex J)
# ---------------------------------------------------------------------------

def bvlc_whois_broadcast() -> bytes:
    # BVLC: 0x81 type, 0x0b Original-Broadcast-NPDU, length
    # NPDU: version 0x01, control 0x20 (dest present), DNET 0xFFFF global, DLEN 0, hopcount 0xFF
    # APDU: 0x10 Unconfirmed-Request, 0x08 Who-Is
    npdu = bytes([0x01, 0x20, 0xFF, 0xFF, 0x00, 0xFF])
    apdu = bytes([0x10, 0x08])
    body = npdu + apdu
    total = 4 + len(body)
    return bytes([0x81, 0x0B, (total >> 8) & 0xFF, total & 0xFF]) + body


def bvlc_iam_broadcast(device_instance: int, vendor_id: int) -> bytes:
    # NPDU: version 0x01, control 0x00
    # APDU: 0x10 Unconfirmed-Request, 0x00 I-Am
    #   ObjectId (app tag C4): device (type 8), instance
    #   Max APDU (app tag 22): 0x01E0 = 480
    #   Segmentation (app tag 91): 0x00 = segmented-both
    #   Vendor Id (app tag 21): vendor_id
    npdu = bytes([0x01, 0x00])
    objid = (8 << 22) | (device_instance & 0x3FFFFF)
    apdu = bytes([0x10, 0x00, 0xC4]) + struct.pack("!I", objid) \
        + bytes([0x22, 0x01, 0xE0, 0x91, 0x00, 0x21, vendor_id & 0xFF])
    body = npdu + apdu
    total = 4 + len(body)
    return bytes([0x81, 0x0B, (total >> 8) & 0xFF, total & 0xFF]) + body


def bvlc_simple_unicast() -> bytes:
    # Minimal BVLC-shaped payload per fixture guidance: 0x81 0x0a 0x00 0x08 + NPDU/APDU
    # 0x0a = Original-Unicast-NPDU, total length 0x0008
    # NPDU version 0x01, control 0x00; APDU 0x10 0x08 (Unconfirmed Who-Is)
    return bytes([0x81, 0x0A, 0x00, 0x08, 0x01, 0x00, 0x10, 0x08])


# ---------------------------------------------------------------------------
# Fixture writers
# ---------------------------------------------------------------------------

def write_pcap(name, packets):
    out = pcap_global_header()
    for ts, frame in packets:
        out += pcap_record(ts, frame)
    path = os.path.join(OUT_DIR, name)
    with open(path, "wb") as f:
        f.write(out)
    print(f"{name}: {len(out)} bytes, {len(packets)} packet(s)")
    return path


def main():
    T0 = 1_717_000_000

    # 1. BACnet UDP/47808 — single datagram with BVLC framing
    write_pcap("hs-bacnet-udp47808.pcap", [
        (T0, udp_frame(MAC_A, MAC_B, IP_A, IP_B, 54321, 47808, bvlc_simple_unicast())),
    ])

    # 2. BACnet TCP/47808 — opening+closing flow, no payload
    write_pcap("hs-bacnet-tcp47808.pcap", tcp_open_close(T0, 54321, 47808))

    # 3. BACnet combined — UDP/47808 datagram + TCP/47808 closing flow
    combined = [(T0, udp_frame(MAC_A, MAC_B, IP_A, IP_B, 54321, 47808, bvlc_simple_unicast()))]
    combined += tcp_open_close(T0 + 10, 54322, 47808)
    write_pcap("hs-bacnet-combined.pcap", combined)

    # 4. TCP/102 (ISO-on-TCP) — opening+closing flow
    write_pcap("hs-tcp102.pcap", tcp_open_close(T0, 49200, 102))

    # 5. TCP/53 (DNS-over-TCP) — opening+closing flow
    write_pcap("hs-tcp53.pcap", tcp_open_close(T0, 49300, 53))

    # 6. Unclassified TCP/9600 — opening+closing flow
    write_pcap("hs-unclassified-tcp9600.pcap", tcp_open_close(T0, 49600, 9600))

    # 7. Empty pcap — global header only
    path = os.path.join(OUT_DIR, "hs-empty.pcap")
    with open(path, "wb") as f:
        f.write(pcap_global_header())
    print(f"hs-empty.pcap: {os.path.getsize(path)} bytes, 0 packet(s)")

    # 8. BACnet corpus — realistic multi-packet Who-Is / I-Am capture
    corpus = []
    t = T0
    # A broadcasts Who-Is
    corpus.append((t, udp_frame(MAC_A, MAC_BCAST, IP_A, IP_BCAST, 47808, 47808,
                                bvlc_whois_broadcast()))); t += 1
    # Three devices answer with I-Am (broadcast)
    corpus.append((t, udp_frame(MAC_B, MAC_BCAST, IP_B, IP_BCAST, 47808, 47808,
                                bvlc_iam_broadcast(260, 15)))); t += 1
    corpus.append((t, udp_frame(MAC_B, MAC_BCAST, IP_C, IP_BCAST, 47808, 47808,
                                bvlc_iam_broadcast(1001, 42)))); t += 1
    corpus.append((t, udp_frame(MAC_A, MAC_BCAST, IP_A, IP_BCAST, 47808, 47808,
                                bvlc_iam_broadcast(7, 8)))); t += 1
    # A second Who-Is round
    corpus.append((t, udp_frame(MAC_A, MAC_BCAST, IP_A, IP_BCAST, 47808, 47808,
                                bvlc_whois_broadcast()))); t += 1
    corpus.append((t, udp_frame(MAC_B, MAC_BCAST, IP_B, IP_BCAST, 47808, 47808,
                                bvlc_iam_broadcast(260, 15)))); t += 1
    # A unicast Who-Is to a specific device and its unicast reply
    corpus.append((t, udp_frame(MAC_A, MAC_B, IP_A, IP_B, 47808, 47808,
                                bvlc_simple_unicast()))); t += 1
    corpus.append((t, udp_frame(MAC_B, MAC_A, IP_B, IP_A, 47808, 47808,
                                bvlc_iam_broadcast(1001, 42)))); t += 1
    write_pcap("hs-bacnet-corpus.pcap", corpus)


if __name__ == "__main__":
    main()
