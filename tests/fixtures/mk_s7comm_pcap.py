#!/usr/bin/env python3
"""
mk_s7comm_pcap.py — Generate tests/fixtures/s7comm-setup-comm.pcap

Produces a minimal libpcap (.pcap) file with classic magic (0xa1b2c3d4,
little-endian) and link-type 1 (Ethernet) containing a classic S7comm (protocol-ID
0x32) session over ISO-on-TCP (TPKT/COTP) on TCP/102.

STORY-187 scope (ADR-014 Decision 7 item 1, "first use in this story"): this
generator hand-crafts the COTP session-establishment handshake (Connect Request /
Connect Confirm) plus a Setup Communication (function 0xF0) Job/Ack_Data PDU pair
and one further minimal classic Job/Ack_Data PDU pair — deliberately synthetic,
dedicated CC0/MIT, following the `mk_modbus_large_pcap.py` precedent (per ADR-014
Decision 7's own citation) (pure-Python libpcap/Ethernet/IPv4/TCP builders, no
external pcap-writing library).

No official public Siemens specification exists for classic S7comm (ADR-014
Decision 4) — the wire layout used here (TPKT/COTP framing, the classic S7comm
common header — 10 bytes for Job (0x01)/Userdata (0x07), 12 bytes for Ack
(0x02)/Ack_Data (0x03) per the 2026-09-24 canonical-frame holdout ruling,
DF-CANONICAL-FRAME-HOLDOUT-001 — and the Setup Communication parameter block) is
derived from free-to-read prose/behavioral sources only (Wireshark wiki prose,
Kleinmann & Wool 2014, the Orange-Cyberdefense awesome-industrial-protocols
catalog) — never from Wireshark's dissector source, Snap7, or libnodave (all
GPL/LGPL-tainted). Zero lines are borrowed from any external implementation.

`S7commAnalyzer` is not yet registered with the dispatcher (STORY-193's scope), so
this fixture is not yet consumed by any CLI/E2E test in this story — it exists to
prove out the generator per this story's File Structure Requirements, establishing a
committed synthetic capture for that later wiring (ADR-014 Decision 7 item 1).

Packet sequence (all timestamps in seconds, realistic 2024-era epoch values):
  1. [t=1_717_100_000] Client→Server SYN (TCP handshake — no payload)
  2. [t=1_717_100_001] Server→Client SYN-ACK
  3. [t=1_717_100_002] Client→Server ACK (handshake complete)
  4. [t=1_717_100_003] Client→Server: COTP Connect Request (CR)
  5. [t=1_717_100_004] Server→Client: COTP Connect Confirm (CC)
  6. [t=1_717_100_005] Client→Server: S7comm Setup Communication request
     (ROSCTR=Job 0x01, parameter function 0xF0)
  7. [t=1_717_100_006] Server→Client: S7comm Setup Communication response
     (ROSCTR=Ack_Data 0x03, parameter function 0xF0, PDU-length negotiated)
  8. [t=1_717_100_007] Client→Server: minimal classic Job PDU (ROSCTR=Job 0x01,
     empty parameter/data blocks — BC-2.21.006 EC-001 shape)
  9. [t=1_717_100_008] Server→Client: minimal classic Ack_Data PDU (ROSCTR=Ack_Data
     0x03, empty parameter/data blocks)
  10. [t=1_717_100_009] Client→Server FIN-ACK

Usage:
  python3 tests/fixtures/mk_s7comm_pcap.py
  # Writes tests/fixtures/s7comm-setup-comm.pcap
"""

import struct
import os

# ---------------------------------------------------------------------------
# libpcap file format helpers (verbatim structure, mirrors mk_modbus_large_pcap.py)
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
# Ethernet / IP / TCP frame builders (mirrors mk_modbus_large_pcap.py)
# ---------------------------------------------------------------------------

CLIENT_MAC = bytes([0x00, 0x11, 0x22, 0x33, 0x44, 0x66])
SERVER_MAC = bytes([0x00, 0x66, 0x77, 0x88, 0x99, 0xBB])
CLIENT_IP = bytes([192, 168, 2, 10])
SERVER_IP = bytes([192, 168, 2, 100])
CLIENT_PORT = 55321  # ephemeral
SERVER_PORT = 102    # ISO-on-TCP / S7comm standard port (RFC 1006, ADR-014)


def eth_header(src_mac: bytes, dst_mac: bytes) -> bytes:
    """14-byte Ethernet II header."""
    return dst_mac + src_mac + bytes([0x08, 0x00])  # EtherType = IPv4


def ip_header(src_ip: bytes, dst_ip: bytes, total_len: int, proto: int = 6) -> bytes:
    """20-byte IPv4 header (no options, TTL=64, identification=0x5678)."""
    hdr = bytearray(
        [
            0x45,  # version=4, IHL=5 (20 bytes)
            0x00,  # DSCP/ECN
            (total_len >> 8) & 0xFF,
            total_len & 0xFF,
            0x56, 0x78,  # identification
            0x40, 0x00,  # flags=DF, fragment offset=0
            64,  # TTL
            proto,  # protocol (6=TCP)
            0x00, 0x00,  # checksum placeholder
        ]
    )
    hdr += src_ip + dst_ip
    words = struct.unpack("!10H", bytes(hdr))
    s = sum(words)
    s = (s >> 16) + (s & 0xFFFF)
    s += s >> 16
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
    """20-byte TCP header. Checksum left as 0 — wirerust does not validate it."""
    offset_flags = (5 << 12) | flags
    hdr = struct.pack(
        "!HHIIHHH",
        src_port,
        dst_port,
        seq,
        ack,
        offset_flags,
        65535,  # window
        0,      # checksum placeholder
    ) + bytes([0, 0])  # urgent pointer
    return bytes(hdr)


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
# TPKT (RFC 1006) / COTP (ISO 8073) framing builders
# ---------------------------------------------------------------------------


def tpkt_frame(cotp_payload: bytes) -> bytes:
    """Wrap a COTP payload in a 4-byte TPKT header (RFC 1006 §6)."""
    total_len = 4 + len(cotp_payload)
    assert total_len <= 0xFFFF, "TPKT length field is a u16 (RFC 1006 §6)"
    return struct.pack("!BBH", 0x03, 0x00, total_len) + cotp_payload


def cotp_cr(dst_ref: int = 0x0000, src_ref: int = 0x0001) -> bytes:
    """
    COTP Connect Request (CR), TPDU code high-nibble 0xE0.

    Fixed part: LI, code|credit, dst-ref(2), src-ref(2), class-option(1) = 6 bytes
    (LI=6 counts everything after the LI byte itself, ISO 8073 §13.3).
    """
    fixed = struct.pack("!BHHB", 0xE0, dst_ref, src_ref, 0x00)
    li = len(fixed)
    return bytes([li]) + fixed


def cotp_cc(dst_ref: int = 0x0001, src_ref: int = 0x0001) -> bytes:
    """COTP Connect Confirm (CC), TPDU code high-nibble 0xD0 — same shape as CR."""
    fixed = struct.pack("!BHHB", 0xD0, dst_ref, src_ref, 0x00)
    li = len(fixed)
    return bytes([li]) + fixed


def cotp_dt(upper_payload: bytes, tpdu_number: int = 0x00) -> bytes:
    """
    COTP Data Transfer (DT), standard ISO 8073 class-0 fixed part: LI (1) +
    code (1, 0xF0) + TPDU-NR+EOT (1) — the code and TPDU-NR+EOT bytes are
    SEPARATE octets (LI=2, counting both bytes that follow the LI byte itself),
    not merged into a single byte. The end-of-TSDU (EOT) bit is bit 7 of the
    TPDU-NR+EOT byte — set here, since every DT in this fixture is a complete,
    unsegmented TSDU — with bits 0-6 carrying the 7-bit TPDU sequence number.
    """
    return bytes([0x02, 0xF0, 0x80 | (tpdu_number & 0x7F)]) + upper_payload


# ---------------------------------------------------------------------------
# Classic S7comm (protocol-ID 0x32) PDU builders
# ---------------------------------------------------------------------------

S7_PROTOCOL_ID = 0x32
ROSCTR_JOB = 0x01
ROSCTR_ACK = 0x02
ROSCTR_ACK_DATA = 0x03


def s7comm_pdu(
    rosctr: int,
    pdu_reference: int,
    parameter: bytes,
    data: bytes,
    error_class: int = 0x00,
    error_code: int = 0x00,
) -> bytes:
    """
    Classic S7comm common header. Per the 2026-09-24 canonical-frame holdout
    ruling (DF-CANONICAL-FRAME-HOLDOUT-001, BC-2.21.006/BC-2.21.008): Job (0x01)
    and Userdata (0x07) use the 10-byte common header — Protocol ID (1) + ROSCTR
    (1) + Reserved (2, always 0x0000 here) + PDU Reference (2 BE) + Parameter
    Length (2 BE) + Data Length (2 BE); Ack (0x02) and Ack_Data (0x03) use a
    12-byte header — the same 10 bytes plus Error Class (1) + Error Code (1) —
    with the parameter block starting at byte 12, not byte 10. `error_class`/
    `error_code` are ignored for Job/Userdata (no such fields exist there).
    """
    header = struct.pack(
        "!BBHHHH",
        S7_PROTOCOL_ID,
        rosctr,
        0x0000,  # reserved
        pdu_reference,
        len(parameter),
        len(data),
    )
    if rosctr in (ROSCTR_ACK, ROSCTR_ACK_DATA):
        header += bytes([error_class & 0xFF, error_code & 0xFF])
    return header + parameter + data


def setup_communication_request(pdu_reference: int) -> bytes:
    """
    Setup Communication (function 0xF0) Job request. Parameter block: function
    (1) + reserved (1) + max AMQ calling (2 BE) + max AMQ called (2 BE) + PDU
    length (2 BE) — the conventional 8-byte Setup Communication parameter shape.
    """
    parameter = struct.pack("!BBHHH", 0xF0, 0x00, 0x0001, 0x0001, 0x03C0)
    return s7comm_pdu(ROSCTR_JOB, pdu_reference, parameter, b"")


def setup_communication_response(pdu_reference: int) -> bytes:
    """
    Setup Communication Ack_Data response — same 8-byte parameter shape, now
    correctly emitted with the 12-byte Ack_Data header (error_class=0x00,
    error_code=0x00), matching the canonical cnblogs Ack_Data layout used by
    `test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data`
    (parameter block at byte 12, per DF-CANONICAL-FRAME-HOLDOUT-001).
    """
    parameter = struct.pack("!BBHHH", 0xF0, 0x00, 0x0001, 0x0001, 0x01E0)
    return s7comm_pdu(ROSCTR_ACK_DATA, pdu_reference, parameter, b"")


def minimal_job_pdu(pdu_reference: int) -> bytes:
    """
    A minimal classic Job PDU with empty parameter/data blocks — the
    BC-2.21.006 EC-001 shape (`param_length == 0`, `data_length == 0`).
    Function-code classification (Groups 3/4) is out of this story's scope
    (STORY-188/189); this PDU exists only to exercise the header-level
    Job/Ack_Data pairing this fixture models.
    """
    return s7comm_pdu(ROSCTR_JOB, pdu_reference, b"", b"")


def minimal_ack_data_pdu(pdu_reference: int) -> bytes:
    """
    The matching minimal classic Ack_Data response, also empty parameter/data
    blocks — a 12-byte Ack_Data header (error_class=0x00, error_code=0x00) per
    DF-CANONICAL-FRAME-HOLDOUT-001; NOT a 10-byte header.
    """
    return s7comm_pdu(ROSCTR_ACK_DATA, pdu_reference, b"", b"")


# ---------------------------------------------------------------------------
# TCP flag constants
# ---------------------------------------------------------------------------
SYN = 0x002
ACK = 0x010
FIN = 0x001
SYN_ACK = SYN | ACK
FIN_ACK = FIN | ACK
PSH_ACK = 0x018  # PSH=0x008 | ACK=0x010


# ---------------------------------------------------------------------------
# Build the complete packet capture
# ---------------------------------------------------------------------------


def build_pcap() -> bytes:
    packets = []

    # Base timestamp: 2024-05-30 20:13:20 UTC (Unix epoch 1717100000)
    t0 = 1_717_100_000

    client_seq = 5000
    server_seq = 9000

    def send(direction_client_to_server, payload, t_offset):
        nonlocal client_seq, server_seq
        if direction_client_to_server:
            frame = build_frame(
                CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                CLIENT_PORT, SERVER_PORT,
                client_seq, server_seq, PSH_ACK, payload,
            )
            packets.append((t0 + t_offset, frame))
            client_seq += len(payload)
        else:
            frame = build_frame(
                SERVER_MAC, CLIENT_MAC, SERVER_IP, CLIENT_IP,
                SERVER_PORT, CLIENT_PORT,
                server_seq, client_seq, PSH_ACK, payload,
            )
            packets.append((t0 + t_offset, frame))
            server_seq += len(payload)

    # --- Packets 1-3: TCP handshake ---
    packets.append(
        (
            t0,
            build_frame(
                CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                CLIENT_PORT, SERVER_PORT, client_seq, 0, SYN, b"",
            ),
        )
    )
    client_seq += 1
    packets.append(
        (
            t0 + 1,
            build_frame(
                SERVER_MAC, CLIENT_MAC, SERVER_IP, CLIENT_IP,
                SERVER_PORT, CLIENT_PORT, server_seq, client_seq, SYN_ACK, b"",
            ),
        )
    )
    server_seq += 1
    packets.append(
        (
            t0 + 2,
            build_frame(
                CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                CLIENT_PORT, SERVER_PORT, client_seq, server_seq, ACK, b"",
            ),
        )
    )

    # --- Packet 4: COTP Connect Request ---
    send(True, tpkt_frame(cotp_cr()), 3)

    # --- Packet 5: COTP Connect Confirm ---
    send(False, tpkt_frame(cotp_cc()), 4)

    # --- Packet 6: Setup Communication request (Job) ---
    send(True, tpkt_frame(cotp_dt(setup_communication_request(0x0001))), 5)

    # --- Packet 7: Setup Communication response (Ack_Data) ---
    send(False, tpkt_frame(cotp_dt(setup_communication_response(0x0001))), 6)

    # --- Packet 8: minimal classic Job PDU (empty parameter/data blocks) ---
    send(True, tpkt_frame(cotp_dt(minimal_job_pdu(0x0002))), 7)

    # --- Packet 9: minimal classic Ack_Data PDU (empty parameter/data blocks) ---
    send(False, tpkt_frame(cotp_dt(minimal_ack_data_pdu(0x0002))), 8)

    # --- Packet 10: Client->Server FIN-ACK ---
    packets.append(
        (
            t0 + 9,
            build_frame(
                CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                CLIENT_PORT, SERVER_PORT, client_seq, server_seq, FIN_ACK, b"",
            ),
        )
    )

    out = pcap_global_header()
    for ts_sec, frame_bytes in packets:
        out += pcap_packet_header(ts_sec, len(frame_bytes))
        out += frame_bytes
    return out


if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    out_path = os.path.join(script_dir, "s7comm-setup-comm.pcap")
    data = build_pcap()
    with open(out_path, "wb") as f:
        f.write(data)
    print(f"Wrote {len(data)} bytes to {out_path}")
    print("  - 10 packets total (3 handshake + CR/CC + Setup Comm req/resp +")
    print("    minimal Job/Ack_Data pair + FIN-ACK)")
    print("  - Port 102 (ISO-on-TCP / S7comm)")
    print(f"  - Timestamps: {1_717_100_000} .. {1_717_100_009} (Unix epoch, 2024-05-30)")
