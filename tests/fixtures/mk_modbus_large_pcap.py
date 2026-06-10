#!/usr/bin/env python3
"""
mk_modbus_large_pcap.py — Generate a LARGER Modbus TCP capture for manual testing.

Writes tests/fixtures/modbus-large.pcap (NOT a committed test fixture — a manual
exploration artifact). Reuses the same pure-Python libpcap/Ethernet/IPv4/TCP
builders as mk_modbus_pcap.py, but lays down FIVE independent TCP flows (distinct
client ephemeral ports) to exercise a broad slice of the analyzer:

  Flow A (:50001) — Recon
      FC 0x11 Report Server ID, FC 0x2B Read Device Identification,
      plus a burst of exception responses (0x01 Illegal Function /
      0x02 Illegal Data Address) modelling an address/function scan.

  Flow B (:50002) — Write-rate (dual-window rate detector)
      34x FC 0x10 Write Multiple Registers: 22 within one 1-second window
      (> burst threshold 20) then 12 more in the next second
      (34 writes / 2s = 17/s > sustained threshold 10).

  Flow C (:50003) — Coil manipulation
      FC 0x05 Write Single Coil, FC 0x0F Write Multiple Coils.

  Flow D (:50004) — Control / register manipulation
      FC 0x06 Write Single Register, FC 0x16 Mask Write Register,
      FC 0x17 Read/Write Multiple Registers.

  Flow E (:50005) — Diagnostics DoS
      FC 0x08 sub 0x0001 Restart Comms, FC 0x08 sub 0x0004 Force Listen-Only.

This script intentionally only *generates traffic*; it makes no claim about which
MITRE techniques fire. Run `wirerust analyze --modbus` against the output and read
what the analyzer actually reports.

Usage:
  python3 tests/fixtures/mk_modbus_large_pcap.py
"""

import struct
import os

# ---------------------------------------------------------------------------
# libpcap + frame builders (identical wire format to mk_modbus_pcap.py)
# ---------------------------------------------------------------------------

PCAP_MAGIC_LE = 0xA1B2C3D4
LINKTYPE_ETHERNET = 1

SERVER_MAC = bytes([0x00, 0x66, 0x77, 0x88, 0x99, 0xAA])
CLIENT_MAC = bytes([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])
CLIENT_IP  = bytes([192, 168, 1, 10])
SERVER_IP  = bytes([192, 168, 1, 100])
SERVER_PORT = 502

SYN = 0x002
ACK = 0x010
FIN = 0x001
SYN_ACK = SYN | ACK
FIN_ACK = FIN | ACK
PSH_ACK = 0x018


def pcap_global_header() -> bytes:
    return struct.pack("<IHHiIII", PCAP_MAGIC_LE, 2, 4, 0, 0, 65535, LINKTYPE_ETHERNET)


def pcap_packet_header(ts_sec: int, ts_usec: int, caplen: int) -> bytes:
    return struct.pack("<IIII", ts_sec, ts_usec, caplen, caplen)


def eth_header(src_mac: bytes, dst_mac: bytes) -> bytes:
    return dst_mac + src_mac + bytes([0x08, 0x00])


def ip_header(src_ip: bytes, dst_ip: bytes, total_len: int, proto: int = 6) -> bytes:
    hdr = bytearray([
        0x45, 0x00,
        (total_len >> 8) & 0xFF, total_len & 0xFF,
        0x12, 0x34, 0x40, 0x00, 64, proto, 0x00, 0x00,
    ])
    hdr += src_ip + dst_ip
    words = struct.unpack("!10H", bytes(hdr))
    s = sum(words)
    s = (s >> 16) + (s & 0xFFFF)
    s += (s >> 16)
    checksum = (~s) & 0xFFFF
    hdr[10] = (checksum >> 8) & 0xFF
    hdr[11] = checksum & 0xFF
    return bytes(hdr)


def tcp_header(src_port, dst_port, seq, ack, flags, payload_len, src_ip, dst_ip) -> bytes:
    offset_flags = (5 << 12) | flags
    hdr = struct.pack("!HHIIHHH", src_port, dst_port, seq, ack, offset_flags, 65535, 0) + bytes([0, 0])
    return bytes(hdr)  # checksum 0 — not validated by wirerust/etherparse path


def build_frame(src_mac, dst_mac, src_ip, dst_ip, src_port, dst_port, seq, ack, flags, payload) -> bytes:
    tcp_hdr = tcp_header(src_port, dst_port, seq, ack, flags, len(payload), src_ip, dst_ip)
    ip_total_len = 20 + 20 + len(payload)
    ip_hdr = ip_header(src_ip, dst_ip, ip_total_len)
    return eth_header(src_mac, dst_mac) + ip_hdr + tcp_hdr + payload


# ---------------------------------------------------------------------------
# Modbus ADU builders
# ---------------------------------------------------------------------------

def modbus_adu(txn_id: int, unit_id: int, fc: int, pdu_data: bytes) -> bytes:
    mbap_length = 1 + 1 + len(pdu_data)
    return struct.pack("!HHHBB", txn_id, 0x0000, mbap_length, unit_id, fc) + pdu_data


U = 0x01  # unit id

def adu_report_server_id(txn):          return modbus_adu(txn, U, 0x11, b"")
def adu_read_device_id(txn):            return modbus_adu(txn, U, 0x2B, bytes([0x0E, 0x01, 0x00]))
def adu_read_input_regs(txn, addr):     return modbus_adu(txn, U, 0x04, struct.pack("!HH", addr, 1))
def adu_exc(txn, fc):                   return modbus_adu(txn, U, fc | 0x80, bytes([0x02]))  # 0x02 Illegal Data Address
def adu_exc_illegal_fn(txn, fc):        return modbus_adu(txn, U, fc | 0x80, bytes([0x01]))  # 0x01 Illegal Function

def adu_write_multi_regs(txn, addr=0x0064):
    values = struct.pack("!HH", 0x0001, 0x0002)
    pdu = struct.pack("!HHB", addr, 2, 4) + values
    return modbus_adu(txn, U, 0x10, pdu)

def adu_write_single_coil(txn, addr=0x0001):
    return modbus_adu(txn, U, 0x05, struct.pack("!HH", addr, 0xFF00))  # ON

def adu_write_multi_coils(txn, addr=0x0010):
    return modbus_adu(txn, U, 0x0F, struct.pack("!HHB", addr, 8, 1) + bytes([0xFF]))

def adu_write_single_reg(txn, addr=0x0002):
    return modbus_adu(txn, U, 0x06, struct.pack("!HH", addr, 0x1234))

def adu_mask_write_reg(txn, addr=0x0004):
    return modbus_adu(txn, U, 0x16, struct.pack("!HHH", addr, 0x00F2, 0x0025))

def adu_read_write_multi(txn):
    pdu = struct.pack("!HHHHB", 0x0003, 0x0006, 0x000E, 0x0003, 6) + struct.pack("!HHH", 1, 2, 3)
    return modbus_adu(txn, U, 0x17, pdu)

def adu_diag(txn, subfunc):
    return modbus_adu(txn, U, 0x08, struct.pack("!HH", subfunc, 0x0000))


# ---------------------------------------------------------------------------
# Flow helper — threads seq/ack/timestamps for one TCP connection
# ---------------------------------------------------------------------------

class Flow:
    def __init__(self, packets, client_port, base_ts):
        self.packets = packets
        self.cport = client_port
        self.cseq = 1000
        self.sseq = 2000
        self.cack = 0
        self.sack = 0
        self.ts = base_ts
        self.usec = 0

    def _emit(self, frame):
        self.packets.append((self.ts, self.usec, frame))

    def handshake(self):
        self._emit(build_frame(CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                               self.cport, SERVER_PORT, self.cseq, 0, SYN, b""))
        self.cseq += 1; self.ts += 1
        self.sack = self.cseq
        self._emit(build_frame(SERVER_MAC, CLIENT_MAC, SERVER_IP, CLIENT_IP,
                               SERVER_PORT, self.cport, self.sseq, self.sack, SYN_ACK, b""))
        self.sseq += 1; self.ts += 1
        self.cack = self.sseq
        self._emit(build_frame(CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                               self.cport, SERVER_PORT, self.cseq, self.cack, ACK, b""))
        self.ts += 1

    def client(self, payload, dt=1):
        self._emit(build_frame(CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                               self.cport, SERVER_PORT, self.cseq, self.cack, PSH_ACK, payload))
        self.cseq += len(payload); self.ts += dt

    def server(self, payload, dt=1):
        self._emit(build_frame(SERVER_MAC, CLIENT_MAC, SERVER_IP, CLIENT_IP,
                               SERVER_PORT, self.cport, self.sseq, self.cseq, PSH_ACK, payload))
        self.sseq += len(payload); self.ts += dt

    def fin(self):
        self._emit(build_frame(CLIENT_MAC, SERVER_MAC, CLIENT_IP, SERVER_IP,
                               self.cport, SERVER_PORT, self.cseq, self.sseq, FIN_ACK, b""))
        self.ts += 1


# ---------------------------------------------------------------------------
# Build the capture
# ---------------------------------------------------------------------------

def build_pcap() -> bytes:
    packets = []
    t0 = 1_717_000_000  # 2024-05-29 UTC

    # --- Flow A: recon (server-id, device-id, function/address scan) ---
    a = Flow(packets, 50001, t0)
    a.handshake()
    a.client(adu_report_server_id(0x0001)); a.server(adu_report_server_id(0x0001))
    a.client(adu_read_device_id(0x0002));   a.server(adu_read_device_id(0x0002))
    # address/function scan: read several input registers, server rejects each
    txn = 0x0010
    for addr in range(8):
        a.client(adu_read_input_regs(txn, 0x1000 + addr), dt=0)
        a.server(adu_exc(0x04, txn), dt=0)   # 0x84 Illegal Data Address
        txn += 1
    a.client(adu_read_input_regs(txn, 0x9999)); a.server(adu_exc_illegal_fn(0x04, txn))  # 0x84 Illegal Function
    a.fin()

    # --- Flow B: write-rate burst + sustained ---
    b = Flow(packets, 50002, t0 + 1)
    b.handshake()
    txn = 0x0100
    for i in range(22):           # 22 writes in the same 1-second window (> burst 20)
        b.client(adu_write_multi_regs(txn, 0x0064 + i), dt=0); txn += 1
    b.ts += 1                      # advance into the next second
    for i in range(12):           # 12 more -> 34 writes / 2s = 17/s (> sustained 10)
        b.client(adu_write_multi_regs(txn, 0x0064 + i), dt=0); txn += 1
    b.fin()

    # --- Flow C: coil manipulation ---
    c = Flow(packets, 50003, t0 + 2)
    c.handshake()
    c.client(adu_write_single_coil(0x0200)); c.server(adu_write_single_coil(0x0200))
    c.client(adu_write_multi_coils(0x0201)); c.server(adu_write_multi_coils(0x0201))
    c.fin()

    # --- Flow D: control / register manipulation ---
    d = Flow(packets, 50004, t0 + 3)
    d.handshake()
    d.client(adu_write_single_reg(0x0300))
    d.client(adu_mask_write_reg(0x0301))
    d.client(adu_read_write_multi(0x0302))
    d.fin()

    # --- Flow E: diagnostics DoS ---
    e = Flow(packets, 50005, t0 + 4)
    e.handshake()
    e.client(adu_diag(0x0400, 0x0001))  # Restart Communications Option
    e.client(adu_diag(0x0401, 0x0004))  # Force Listen Only Mode
    e.fin()

    # sort by (ts, usec) so the capture is chronologically ordered across flows
    packets.sort(key=lambda p: (p[0], p[1]))

    out = pcap_global_header()
    for ts_sec, ts_usec, frame in packets:
        out += pcap_packet_header(ts_sec, ts_usec, len(frame))
        out += frame
    return out, len(packets)


if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    # Output into the gitignored local-samples/ dir (kept local-only, never committed).
    out_dir = os.path.join(script_dir, "local-samples")
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, "modbus-large.pcap")
    data, n = build_pcap()
    with open(out_path, "wb") as f:
        f.write(data)
    print(f"Wrote {len(data)} bytes ({n} packets) to {out_path}")
    print("  Flows: A recon | B write-rate(34) | C coils | D registers | E diagnostics")
