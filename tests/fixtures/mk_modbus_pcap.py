#!/usr/bin/env python3
"""
mk_modbus_pcap.py — Generate tests/fixtures/modbus-write.pcap

Produces a minimal libpcap (.pcap) file with classic magic (0xa1b2c3d4,
little-endian) and link-type 1 (Ethernet) containing a Modbus TCP session on
port 502.

Packet sequence (all timestamps in seconds, realistic 2024-era epoch values):
  1. [t=1_717_000_000] Client→Server SYN (TCP handshake — no payload)
  2. [t=1_717_000_001] Server→Client SYN-ACK
  3. [t=1_717_000_002] Client→Server ACK (handshake complete)
  4. [t=1_717_000_003] Client→Server: FC=0x11 Report Server ID (recon → T0888)
  5. [t=1_717_000_004] Server→Client: FC=0x11 response (also triggers T0888, direction-independent)
  6. [t=1_717_000_005] Client→Server: FC=0x10 Write Multiple Registers (write → T1692.001+T0836)
  7. [t=1_717_000_006] Server→Client: FC=0x90 Exception response (0x10|0x80) for txn 0x0003
  8. [t=1_717_000_007] Client→Server: FIN-ACK

Findings expected from wirerust analyze --modbus:
  - T0888 (recon): FC=0x11 from client (packet 4)
  - T0888 (recon): FC=0x11 from server (packet 5, direction-independent)
  - T1692.001+T0836 (write): FC=0x10 Write Multiple Registers (packet 6)
  - Exception response (packet 7) increments exception_count

Summary fields:
  - analyzer_name: "modbus"
  - pdu_count: >= 4 (packets 4-7 each carry one valid Modbus ADU)
  - write_count: >= 1 (packet 6)
  - exception_count: >= 1 (packet 7)

Usage:
  python3 tests/fixtures/mk_modbus_pcap.py
  # Writes tests/fixtures/modbus-write.pcap
"""

import struct
import os

# ---------------------------------------------------------------------------
# libpcap file format helpers
# ---------------------------------------------------------------------------

PCAP_MAGIC_LE = 0xA1B2C3D4  # little-endian native, microsecond timestamps
LINKTYPE_ETHERNET = 1


def pcap_global_header() -> bytes:
    """24-byte global header (little-endian)."""
    return struct.pack(
        "<IHHiIII",
        PCAP_MAGIC_LE,  # magic_number
        2,              # version_major
        4,              # version_minor
        0,              # thiszone (UTC)
        0,              # sigfigs
        65535,          # snaplen
        LINKTYPE_ETHERNET,
    )


def pcap_packet_header(ts_sec: int, caplen: int) -> bytes:
    """16-byte per-packet header (little-endian). ts_usec=0 (whole-second)."""
    return struct.pack(
        "<IIII",
        ts_sec,   # ts_sec
        0,        # ts_usec
        caplen,   # incl_len
        caplen,   # orig_len
    )


# ---------------------------------------------------------------------------
# Ethernet / IP / TCP frame builders
# ---------------------------------------------------------------------------

CLIENT_MAC = bytes([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])
SERVER_MAC = bytes([0x00, 0x66, 0x77, 0x88, 0x99, 0xAA])
CLIENT_IP  = bytes([192, 168, 1, 10])
SERVER_IP  = bytes([192, 168, 1, 100])
CLIENT_PORT = 54321  # ephemeral
SERVER_PORT = 502    # Modbus TCP standard port


def eth_header(src_mac: bytes, dst_mac: bytes) -> bytes:
    """14-byte Ethernet II header."""
    return dst_mac + src_mac + bytes([0x08, 0x00])  # EtherType = IPv4


def ip_header(src_ip: bytes, dst_ip: bytes, total_len: int, proto: int = 6) -> bytes:
    """20-byte IPv4 header (no options, TTL=64, identification=0x1234)."""
    # Build without checksum first, then compute.
    hdr = bytearray([
        0x45,             # version=4, IHL=5 (20 bytes)
        0x00,             # DSCP/ECN
        (total_len >> 8) & 0xFF,
        total_len & 0xFF,
        0x12, 0x34,       # identification
        0x40, 0x00,       # flags=DF, fragment offset=0
        64,               # TTL
        proto,            # protocol (6=TCP)
        0x00, 0x00,       # checksum placeholder
    ])
    hdr += src_ip + dst_ip
    # One's complement checksum.
    words = struct.unpack("!10H", bytes(hdr))
    s = sum(words)
    s = (s >> 16) + (s & 0xFFFF)
    s += (s >> 16)
    checksum = (~s) & 0xFFFF
    hdr[10] = (checksum >> 8) & 0xFF
    hdr[11] = checksum & 0xFF
    return bytes(hdr)


def tcp_header(
    src_port: int,
    dst_port: int,
    seq: int,
    ack: int,
    flags: int,
    payload_len: int,
    src_ip: bytes,
    dst_ip: bytes,
) -> bytes:
    """20-byte TCP header with correct checksum."""
    # Data offset = 5 (20 bytes, no options), window = 65535.
    offset_flags = (5 << 12) | flags
    hdr = struct.pack(
        "!HHIIHHH",
        src_port,
        dst_port,
        seq,
        ack,
        offset_flags,
        65535,   # window
        0,       # checksum placeholder
    ) + bytes([0, 0])  # urgent pointer
    # Pad to even length for checksum.
    payload_pad = payload_len
    # TCP pseudo-header for checksum.
    tcp_len = 20 + payload_len
    pseudo = src_ip + dst_ip + bytes([0, 6]) + struct.pack("!H", tcp_len)
    chk_data = pseudo + hdr
    if payload_len % 2 == 1:
        # Odd-length payload: add a zero pad byte for checksum computation only.
        # The actual payload is NOT included here (checksum is over header only for
        # packets without application payload in this builder); for data packets the
        # caller concatenates the payload before computing the checksum.
        pass
    # Recompute with correct method below.
    def ones_complement_sum(data: bytes) -> int:
        if len(data) % 2:
            data += b"\x00"
        words = struct.unpack("!%dH" % (len(data) // 2), data)
        s = sum(words)
        s = (s >> 16) + (s & 0xFFFF)
        s += (s >> 16)
        return (~s) & 0xFFFF

    return bytes(hdr)  # checksum left as 0 — wireshark/wirerust do not validate TCP checksum


def build_frame(
    src_mac: bytes,
    dst_mac: bytes,
    src_ip: bytes,
    dst_ip: bytes,
    src_port: int,
    dst_port: int,
    seq: int,
    ack: int,
    flags: int,
    payload: bytes,
) -> bytes:
    """Build a complete Ethernet+IPv4+TCP frame."""
    tcp_hdr = tcp_header(src_port, dst_port, seq, ack, flags, len(payload), src_ip, dst_ip)
    ip_total_len = 20 + 20 + len(payload)  # IP + TCP + payload
    ip_hdr = ip_header(src_ip, dst_ip, ip_total_len)
    eth_hdr = eth_header(src_mac, dst_mac)
    return eth_hdr + ip_hdr + tcp_hdr + payload


# ---------------------------------------------------------------------------
# Modbus ADU builders
# ---------------------------------------------------------------------------

def modbus_adu(txn_id: int, unit_id: int, fc: int, pdu_data: bytes) -> bytes:
    """Build a Modbus TCP ADU: 6-byte MBAP header + unit_id + fc + pdu_data."""
    # MBAP length field = unit_id (1) + fc (1) + len(pdu_data)
    mbap_length = 1 + 1 + len(pdu_data)
    return struct.pack(
        "!HHHBB",
        txn_id,      # transaction_id
        0x0000,      # protocol_id (always 0 for Modbus TCP)
        mbap_length, # length
        unit_id,     # unit identifier
        fc,          # function code
    ) + pdu_data


# FC=0x11: Report Server ID (no PDU data in request)
def adu_report_server_id(txn_id: int) -> bytes:
    return modbus_adu(txn_id, 0x01, 0x11, b"")


# FC=0x11 response: Report Server ID response (server sends back some ID bytes)
def adu_report_server_id_response(txn_id: int) -> bytes:
    # Minimal response: byte count + server ID bytes + run indicator
    server_id = b"\x05\x01\x02\x03\x04\xFF"  # byte_count=5, id bytes, run=0xFF(on)
    return modbus_adu(txn_id, 0x01, 0x11, server_id)


# FC=0x10: Write Multiple Registers
def adu_write_multiple_registers(txn_id: int) -> bytes:
    # Write 2 registers starting at address 0x0064 (100)
    # PDU: starting_addr (2) + quantity (2) + byte_count (1) + values (quantity*2)
    starting_addr = 0x0064
    quantity = 2
    byte_count = quantity * 2
    values = struct.pack("!HH", 0x0001, 0x0002)  # register values: 1, 2
    pdu = struct.pack("!HHB", starting_addr, quantity, byte_count) + values
    return modbus_adu(txn_id, 0x01, 0x10, pdu)


# FC=0x90: Exception response for FC=0x10 (Write Multiple Registers)
# Exception FC = 0x10 | 0x80 = 0x90
def adu_write_multiple_registers_exception(txn_id: int) -> bytes:
    # Exception code 0x01 = Illegal Function
    return modbus_adu(txn_id, 0x01, 0x90, bytes([0x04]))  # code 0x04 = Server Failure


# ---------------------------------------------------------------------------
# TCP flag constants
# ---------------------------------------------------------------------------
SYN = 0x002
ACK = 0x010
FIN = 0x001
RST = 0x004
SYN_ACK = SYN | ACK
FIN_ACK = FIN | ACK
PSH_ACK = 0x018  # PSH=0x008 | ACK=0x010


# ---------------------------------------------------------------------------
# Build the complete packet capture
# ---------------------------------------------------------------------------

def build_pcap() -> bytes:
    packets = []

    # Base timestamp: 2024-05-29 ~18:53:20 UTC (Unix epoch 1717000000)
    t0 = 1_717_000_000

    # Sequence numbers: client starts at 1000, server starts at 2000.
    client_seq = 1000
    server_seq = 2000
    client_ack = 0
    server_ack = 0

    # Packet 1: Client→Server SYN
    frame = build_frame(
        CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
        CLIENT_PORT, SERVER_PORT,
        client_seq, 0, SYN, b""
    )
    packets.append((t0 + 0, frame))
    client_seq += 1  # SYN consumes one sequence number

    # Packet 2: Server→Client SYN-ACK
    server_ack = client_seq
    frame = build_frame(
        SERVER_MAC, CLIENT_MAC, SERVER_IP, CLIENT_IP,
        SERVER_PORT, CLIENT_PORT,
        server_seq, server_ack, SYN_ACK, b""
    )
    packets.append((t0 + 1, frame))
    server_seq += 1  # SYN consumes one sequence number

    # Packet 3: Client→Server ACK (completes handshake)
    client_ack = server_seq
    frame = build_frame(
        CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
        CLIENT_PORT, SERVER_PORT,
        client_seq, client_ack, ACK, b""
    )
    packets.append((t0 + 2, frame))

    # Packet 4: Client→Server — FC=0x11 Report Server ID (recon → T0888)
    payload4 = adu_report_server_id(txn_id=0x0001)
    frame = build_frame(
        CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
        CLIENT_PORT, SERVER_PORT,
        client_seq, client_ack, PSH_ACK, payload4
    )
    packets.append((t0 + 3, frame))
    client_seq += len(payload4)

    # Packet 5: Server→Client — FC=0x11 response (also triggers T0888, direction-independent)
    payload5 = adu_report_server_id_response(txn_id=0x0001)
    frame = build_frame(
        SERVER_MAC, CLIENT_MAC, SERVER_IP, CLIENT_IP,
        SERVER_PORT, CLIENT_PORT,
        server_seq, client_seq, PSH_ACK, payload5
    )
    packets.append((t0 + 4, frame))
    server_seq += len(payload5)

    # Packet 6: Client→Server — FC=0x10 Write Multiple Registers (write → T1692.001+T0836)
    payload6 = adu_write_multiple_registers(txn_id=0x0002)
    frame = build_frame(
        CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
        CLIENT_PORT, SERVER_PORT,
        client_seq, server_seq, PSH_ACK, payload6
    )
    packets.append((t0 + 5, frame))
    client_seq += len(payload6)

    # Packet 7: Server→Client — FC=0x90 Exception response for FC=0x10
    payload7 = adu_write_multiple_registers_exception(txn_id=0x0002)
    frame = build_frame(
        SERVER_MAC, CLIENT_MAC, SERVER_IP, CLIENT_IP,
        SERVER_PORT, CLIENT_PORT,
        server_seq, client_seq, PSH_ACK, payload7
    )
    packets.append((t0 + 6, frame))
    server_seq += len(payload7)

    # Packet 8: Client→Server FIN-ACK
    frame = build_frame(
        CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
        CLIENT_PORT, SERVER_PORT,
        client_seq, server_seq, FIN_ACK, b""
    )
    packets.append((t0 + 7, frame))

    # Assemble the pcap file.
    out = pcap_global_header()
    for ts_sec, frame_bytes in packets:
        out += pcap_packet_header(ts_sec, len(frame_bytes))
        out += frame_bytes
    return out


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    out_path = os.path.join(script_dir, "modbus-write.pcap")
    data = build_pcap()
    with open(out_path, "wb") as f:
        f.write(data)
    print(f"Wrote {len(data)} bytes to {out_path}")
    print(f"  - 8 packets total")
    print(f"  - Modbus ADUs: FC=0x11 (recon x2), FC=0x10 (write), FC=0x90 (exception)")
    print(f"  - Expected findings: T0888 (x2), T1692.001+T0836 (x1)")
    print(f"  - Timestamps: {1_717_000_000} .. {1_717_000_007} (Unix epoch, 2024-05-29)")
