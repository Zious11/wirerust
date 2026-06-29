#!/usr/bin/env python3
"""
Generate synthetic pcap fixtures for TLS ClientHello fragmentation demo.

CONTROL: ClientHello in a single TLS record (normal case).
EVASION: Same ClientHello split across two TLS records (fragmented).

RFC 5246 §6.2.1 allows the TLS record layer to fragment handshake messages
across multiple TLSPlaintext records. wirerust's TLS analyzer processes each
TLS record independently and does not reassemble split handshake messages.
When the ClientHello spans two records, the SNI and JA3 are silently missed.

Pcap format: classic pcap (magic 0xa1b2c3d4), linktype 1 (Ethernet).
Packet structure: Ethernet II + IPv4 + TCP (no options) + TLS payload.
No checksums are computed (fields set to 0) — wirerust does not validate them.
"""

import struct
import sys

# ── ClientHello payload ────────────────────────────────────────────────────────
# Minimal TLS 1.2 ClientHello with SNI = "example.com", one cipher suite,
# one compression method, and a server_name extension.
#
# Handshake message body (after type+length):
#   client_version:  0x03 0x03  (TLS 1.2)
#   random:          32 bytes of 0xAA
#   session_id_len:  0x00
#   cipher_suites_len: 0x00 0x02
#   cipher_suite:    0x00 0x2F  (TLS_RSA_WITH_AES_128_CBC_SHA)
#   compression_len: 0x01
#   compression:     0x00
#   extensions_len:  0x00 0x14  (20 bytes)
#   extension: server_name (0x00 0x00)
#     ext_len: 0x00 0x10  (16 bytes)
#     list_len: 0x00 0x0E  (14 bytes)
#     name_type: 0x00 (host_name)
#     name_len: 0x00 0x0B  (11 bytes)
#     name: "example.com"

SNI = b"example.com"   # 11 bytes

ext_sni_name = SNI
ext_sni_name_len = len(ext_sni_name)          # 11
ext_sni_entry_len = 1 + 2 + ext_sni_name_len  # name_type(1) + name_len(2) + name = 14
ext_sni_list_len = ext_sni_entry_len           # 14
ext_sni_body = (
    struct.pack('>H', ext_sni_list_len)        # list_len
    + b'\x00'                                   # name_type = host_name
    + struct.pack('>H', ext_sni_name_len)      # name_len
    + ext_sni_name                              # name
)
ext_sni_payload_len = len(ext_sni_body)        # 2+1+2+11 = 16

extensions = (
    struct.pack('>H', 0x0000)                  # server_name extension type
    + struct.pack('>H', ext_sni_payload_len)   # ext data length
    + ext_sni_body
)
extensions_len = len(extensions)               # 20

hs_body = (
    b'\x03\x03'                                # client_version = TLS 1.2
    + b'\xAA' * 32                             # random (32 bytes)
    + b'\x00'                                  # session_id_len = 0
    + b'\x00\x02'                              # cipher_suites_len = 2
    + b'\x00\x2F'                              # TLS_RSA_WITH_AES_128_CBC_SHA
    + b'\x01'                                  # compression_methods_len = 1
    + b'\x00'                                  # no compression
    + struct.pack('>H', extensions_len)        # extensions_len
    + extensions
)

hs_len = len(hs_body)
hs_msg = (
    b'\x01'                                    # HandshakeType = client_hello
    + struct.pack('>I', hs_len)[1:]            # 3-byte length
    + hs_body
)

print(f"Handshake message length: {len(hs_msg)} bytes")
print(f"  - handshake body: {hs_len} bytes")
print(f"  - SNI: {SNI.decode()!r}")
print(f"  - extensions_len: {extensions_len}")


# ── pcap helpers ───────────────────────────────────────────────────────────────

PCAP_GLOBAL_HDR = struct.pack(
    '<IHHiIII',
    0xa1b2c3d4,  # magic
    2, 4,        # version major.minor
    0,           # timezone (UTC)
    0,           # timestamp accuracy
    65535,       # snaplen
    1,           # linktype = Ethernet
)


def pcap_packet(ts_sec: int, ts_usec: int, payload: bytes) -> bytes:
    """Wrap a frame in a pcap packet record."""
    return struct.pack('<IIII', ts_sec, ts_usec, len(payload), len(payload)) + payload


def eth_ip_tcp(src_ip: str, dst_ip: str, src_port: int, dst_port: int,
               seq: int, ack: int, flags: int, data: bytes) -> bytes:
    """Build Ethernet + IPv4 + TCP frame (no checksum computation)."""
    # Ethernet header: dst, src, ethertype
    eth = b'\x00' * 6 + b'\x00' * 6 + b'\x08\x00'

    ip_ihl_version = 0x45
    ip_tos = 0
    ip_total_len = 20 + 20 + len(data)
    ip_id = 0x1234
    ip_frag_off = 0
    ip_ttl = 64
    ip_proto = 6  # TCP
    ip_checksum = 0
    src_addr = bytes(int(x) for x in src_ip.split('.'))
    dst_addr = bytes(int(x) for x in dst_ip.split('.'))

    ip_hdr = struct.pack(
        '>BBHHHBBH4s4s',
        ip_ihl_version, ip_tos, ip_total_len,
        ip_id, ip_frag_off,
        ip_ttl, ip_proto, ip_checksum,
        src_addr, dst_addr,
    )

    tcp_data_offset = (5 << 4)  # 5 32-bit words, no options
    tcp_window = 65535
    tcp_checksum = 0
    tcp_urg = 0

    tcp_hdr = struct.pack(
        '>HHIIBBHHH',
        src_port, dst_port,
        seq, ack,
        tcp_data_offset, flags,
        tcp_window, tcp_checksum, tcp_urg,
    )

    return eth + ip_hdr + tcp_hdr + data


# Common flow parameters
SRC_IP = '192.168.1.100'
DST_IP = '93.184.216.34'  # example.com
SRC_PORT = 54321
DST_PORT = 443
FLAGS_ACK = 0x10  # ACK
FLAGS_PA = 0x18   # PSH+ACK


# ── CONTROL pcap: single TLS record containing the full ClientHello ────────────

def make_tls_record(content_type: int, version: bytes, payload: bytes) -> bytes:
    return bytes([content_type]) + version + struct.pack('>H', len(payload)) + payload


control_record = make_tls_record(0x16, b'\x03\x01', hs_msg)
print(f"\nControl TLS record: {len(control_record)} bytes")

# Sequence numbers
SEQ_BASE = 1000
ACK_BASE = 2000

control_pkt = eth_ip_tcp(
    SRC_IP, DST_IP, SRC_PORT, DST_PORT,
    SEQ_BASE, ACK_BASE, FLAGS_PA, control_record
)

control_pcap = PCAP_GLOBAL_HDR + pcap_packet(1700000000, 0, control_pkt)

with open('/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-control.pcap', 'wb') as f:
    f.write(control_pcap)
print("Written: tls-clienthello-control.pcap")


# ── EVASION pcap: ClientHello split across two TLS records ────────────────────
# Split the handshake message: first record gets bytes 0..split_at,
# second record gets bytes split_at..end.
# Both are TLS content_type=0x16 (Handshake), version 0x03 0x01.

split_at = len(hs_msg) // 2
hs_part1 = hs_msg[:split_at]
hs_part2 = hs_msg[split_at:]

print(f"\nFragmentation split: {split_at} / {len(hs_msg) - split_at}")
print(f"  Part 1: bytes 0..{split_at} (includes handshake type=0x01 and partial header)")
print(f"  Part 2: bytes {split_at}..{len(hs_msg)} (SNI extension is in this half)")

frag_record1 = make_tls_record(0x16, b'\x03\x01', hs_part1)
frag_record2 = make_tls_record(0x16, b'\x03\x01', hs_part2)

print(f"Frag record 1: {len(frag_record1)} bytes")
print(f"Frag record 2: {len(frag_record2)} bytes")

# Two packets in sequence: record1, then record2
seq1 = SEQ_BASE
seq2 = SEQ_BASE + len(frag_record1)

evasion_pkt1 = eth_ip_tcp(SRC_IP, DST_IP, SRC_PORT, DST_PORT, seq1, ACK_BASE, FLAGS_PA, frag_record1)
evasion_pkt2 = eth_ip_tcp(SRC_IP, DST_IP, SRC_PORT, DST_PORT, seq2, ACK_BASE, FLAGS_PA, frag_record2)

evasion_pcap = (
    PCAP_GLOBAL_HDR
    + pcap_packet(1700000000, 0, evasion_pkt1)
    + pcap_packet(1700000000, 100000, evasion_pkt2)
)

with open('/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-fragmented.pcap', 'wb') as f:
    f.write(evasion_pcap)
print("Written: tls-clienthello-fragmented.pcap")

print("\nDone. Run wirerust on both pcaps to observe the evasion gap.")
