#!/usr/bin/env python3
"""
mk_spb_fixtures.py -- Generate minimal pcapng fixtures for STORY-126 demo recordings.

Produces three files:
  spb-happy.pcapng    -- SHB + IDB(Ethernet) + SPB(14-byte Ethernet frame)
  spb-skip.pcapng     -- SHB + IDB + NRB + ISB + SJE + EPB (block-skip demo)
  spb-error.pcapng    -- SHB + SPB (no IDB) -> triggers E-INP-009 on read

All pcapng blocks are little-endian. The SHB BOM (0x1A2B3C4D) governs byte-order.
"""

import struct
import os

OUT_DIR = os.path.dirname(os.path.abspath(__file__))

# Block type codes
SHB_BLOCK_TYPE = 0x0A0D_0D0A
IDB_BLOCK_TYPE = 0x0000_0001
EPB_BLOCK_TYPE = 0x0000_0006
SPB_BLOCK_TYPE = 0x0000_0003
NRB_BLOCK_TYPE = 0x0000_0004
ISB_BLOCK_TYPE = 0x0000_0005
SJE_BLOCK_TYPE = 0x0000_0009

# Little-endian BOM
SHB_BOM_LE = bytes([0x4D, 0x3C, 0x2B, 0x1A])
DL_ETHERNET = 1


def pack_u16(v): return struct.pack("<H", v)
def pack_u32(v): return struct.pack("<I", v)
def pack_u64(v): return struct.pack("<Q", v)


def le_shb():
    """Minimal 28-byte LE SHB."""
    btl = 28
    b = pack_u32(SHB_BLOCK_TYPE)
    b += pack_u32(btl)
    b += SHB_BOM_LE
    b += pack_u16(1)                         # major=1
    b += pack_u16(0)                         # minor=0
    b += pack_u64(0xFFFF_FFFF_FFFF_FFFF)     # section_length=-1
    b += pack_u32(btl)
    assert len(b) == 28
    return b


def le_idb(linktype=DL_ETHERNET):
    """Minimal 20-byte LE IDB."""
    btl = 20
    b = pack_u32(IDB_BLOCK_TYPE)
    b += pack_u32(btl)
    b += pack_u16(linktype)
    b += pack_u16(0)                         # reserved
    b += pack_u32(65535)                     # snaplen
    b += pack_u32(btl)
    assert len(b) == 20
    return b


def pad4(data):
    """Pad bytes to 4-byte alignment."""
    n = (4 - len(data) % 4) % 4
    return data + b'\x00' * n


def le_spb(data):
    """LE SPB block with given payload bytes."""
    padded = pad4(data)
    body = pack_u32(len(data)) + padded      # original_len + payload (padded)
    btl = 12 + len(body)
    b = pack_u32(SPB_BLOCK_TYPE)
    b += pack_u32(btl)
    b += body
    b += pack_u32(btl)
    assert len(b) == btl
    return b


def le_epb(data, iface_id=0):
    """LE EPB block with given payload bytes at the given interface."""
    padded = pad4(data)
    captured_len = len(data)
    original_len = len(data)
    # EPB fixed: iface_id(4) ts_high(4) ts_low(4) captured_len(4) original_len(4) = 20 bytes
    body = (pack_u32(iface_id) +
            pack_u32(1) +                    # ts_high (non-palindromic: 0x00000001)
            pack_u32(0) +                    # ts_low
            pack_u32(captured_len) +
            pack_u32(original_len) +
            padded)
    btl = 12 + len(body)
    b = pack_u32(EPB_BLOCK_TYPE)
    b += pack_u32(btl)
    b += body
    b += pack_u32(btl)
    assert len(b) == btl
    return b


def le_skip_block(block_type, body=b''):
    """LE skip block of the given type with given body (must be 4-aligned)."""
    aligned = body + b'\x00' * ((4 - len(body) % 4) % 4)
    btl = 12 + len(aligned)
    b = pack_u32(block_type)
    b += pack_u32(btl)
    b += aligned
    b += pack_u32(btl)
    assert len(b) == btl
    return b


# ── Fixture 1: Happy path — SHB + IDB + SPB ────────────────────────────────
# 14-byte minimal Ethernet frame (6+6+2): dst=00:11:22:33:44:55 src=aa:bb:cc:dd:ee:ff ethertype=0x0800
eth_frame = bytes([
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55,   # dst MAC
    0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,   # src MAC
    0x08, 0x00,                             # ethertype IPv4
])
happy = le_shb() + le_idb() + le_spb(eth_frame)
with open(os.path.join(OUT_DIR, 'spb-happy.pcapng'), 'wb') as f:
    f.write(happy)
print(f"wrote spb-happy.pcapng  ({len(happy)} bytes) -- SHB+IDB+SPB(14-byte Ethernet)")

# ── Fixture 2: Block-skip demo — SHB + IDB + NRB + ISB + SJE + EPB ────────
# Uses an EPB (not SPB) so the CLI output shows packets parsed after skipped blocks.
eth_frame_2 = bytes([
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,   # broadcast dst
    0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02,  # src MAC
    0x08, 0x06,                            # ARP ethertype
])
skip_demo = (
    le_shb() +
    le_idb() +
    le_skip_block(NRB_BLOCK_TYPE, b'\x00' * 8) +   # NRB (8-byte body)
    le_skip_block(ISB_BLOCK_TYPE) +                  # ISB (empty body)
    le_skip_block(SJE_BLOCK_TYPE) +                  # SJE (empty body)
    le_epb(eth_frame_2)
)
with open(os.path.join(OUT_DIR, 'spb-skip.pcapng'), 'wb') as f:
    f.write(skip_demo)
print(f"wrote spb-skip.pcapng   ({len(skip_demo)} bytes) -- SHB+IDB+NRB+ISB+SJE+EPB (3 skip blocks)")

# ── Fixture 3: Error path — SHB + SPB (no IDB) → E-INP-009 ─────────────────
error_payload = bytes([0xDE, 0xAD, 0xBE, 0xEF])
error_case = le_shb() + le_spb(error_payload)
with open(os.path.join(OUT_DIR, 'spb-error.pcapng'), 'wb') as f:
    f.write(error_case)
print(f"wrote spb-error.pcapng  ({len(error_case)} bytes) -- SHB+SPB(no IDB) => E-INP-009")

print("\nAll fixtures written to:", OUT_DIR)
