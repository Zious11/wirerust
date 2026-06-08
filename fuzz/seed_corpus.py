#!/usr/bin/env python3
"""Seed the fuzz_decode_packet corpus for VP-008.

The fuzz target consumes a raw link-layer FRAME (&[u8]), not a full pcap file.
So we:
  1. Parse each classic .pcap/.cap fixture (stdlib only) and emit each record's
     frame bytes as one corpus file.
  2. Emit truncated variants of a sample of real frames (every header boundary).
  3. Emit synthetic adversarial/malformed frames per VP-008 Corpus Seeding.

Corpus filenames are content-hash based so duplicates collapse.
"""
import hashlib
import os
import struct
import sys

CORPUS = os.path.join(os.path.dirname(__file__), "corpus", "fuzz_decode_packet")
FIXTURES = sys.argv[1]

os.makedirs(CORPUS, exist_ok=True)

written = 0
def emit(data: bytes):
    global written
    h = hashlib.sha1(data).hexdigest()
    path = os.path.join(CORPUS, h)
    if not os.path.exists(path):
        with open(path, "wb") as f:
            f.write(data)
        written += 1

def parse_classic_pcap(raw: bytes):
    """Yield each record's captured frame bytes. Supports LE/BE magic."""
    if len(raw) < 24:
        return
    magic = raw[:4]
    if magic in (b"\xd4\xc3\xb2\xa1", b"\x4d\x3c\xb2\xa1"):  # little-endian (us / ns)
        endian = "<"
    elif magic in (b"\xa1\xb2\xc3\xd4", b"\xa1\xb2\x3c\x4d"):  # big-endian
        endian = ">"
    else:
        return  # not classic pcap (e.g. pcapng) -- skip
    off = 24
    n = len(raw)
    while off + 16 <= n:
        ts_sec, ts_usec, incl_len, orig_len = struct.unpack(
            endian + "IIII", raw[off:off + 16]
        )
        off += 16
        if incl_len > n - off:
            # truncated final record; take what's there as an adversarial seed too
            frame = raw[off:]
            if frame:
                yield frame
            break
        frame = raw[off:off + incl_len]
        off += incl_len
        if frame:
            yield frame

# 1. Real frames from fixtures
real_frames = []
for name in sorted(os.listdir(FIXTURES)):
    if not (name.endswith(".pcap") or name.endswith(".cap") or name.endswith(".trace")):
        continue
    p = os.path.join(FIXTURES, name)
    with open(p, "rb") as f:
        raw = f.read()
    cnt = 0
    for frame in parse_classic_pcap(raw):
        emit(frame)
        real_frames.append(frame)
        cnt += 1
        # cap per-file frames to keep corpus lean but representative
        if cnt >= 200:
            break

# 2. Truncations of a representative sample of real frames at header boundaries.
# Header boundaries of interest: ethernet=14, IPv4 min=20, IPv6=40, TCP min=20,
# Linux SLL=16, plus 1 byte and "1 short of full".
sample = real_frames[:: max(1, len(real_frames) // 60)] if real_frames else []
boundaries = [1, 2, 4, 8, 13, 14, 15, 16, 19, 20, 21, 33, 34, 39, 40, 41, 53, 54]
for frame in sample:
    for b in boundaries:
        if 0 < b < len(frame):
            emit(frame[:b])
    if len(frame) > 1:
        emit(frame[:-1])  # one short of full

# 3. Synthetic adversarial / malformed frames (per VP-008 Corpus Seeding)
emit(b"")                       # empty
emit(b"\x00")                   # single byte
emit(b"\xff")                   # single byte
for i in range(1, 14):          # truncated ethernet header (< 14 bytes)
    emit(b"\x00" * i)
# Minimal ethernet header, IPv4 ethertype, no payload (truncated IP)
emit(bytes.fromhex("ffffffffffff000000000000") + b"\x08\x00")
# Ethernet + IPv4 ethertype + truncated IPv4 header (claims more)
emit(bytes.fromhex("ffffffffffff000000000000") + b"\x08\x00" + b"\x45" + b"\x00" * 5)
# Ethernet + IPv6 ethertype + truncated IPv6 header
emit(bytes.fromhex("ffffffffffff000000000000") + b"\x86\xdd" + b"\x60" + b"\x00" * 10)
# IPv4 frame with oversized total-length field (0xffff) but tiny actual payload
emit(b"\x45\x00\xff\xff\x00\x00\x00\x00\x40\x06\x00\x00" + b"\x01\x02\x03\x04" + b"\x05\x06\x07\x08")
# IPv4 with IHL claiming huge options but truncated
emit(b"\x4f\x00\x00\x3c" + b"\x00" * 8 + b"\x01\x02\x03\x04\x05\x06\x07\x08")
# IPv6 with bogus next-header chain and oversized payload length
emit(b"\x60\x00\x00\x00\xff\xff\x2c\x40" + b"\x00" * 32 + b"\x3b" + b"\x00" * 16)
# TCP-ish: ethernet + IPv4(proto=6) + truncated TCP with huge data offset
emit(
    bytes.fromhex("ffffffffffff000000000000") + b"\x08\x00"
    + b"\x45\x00\x00\x28\x00\x00\x00\x00\x40\x06\x00\x00\x0a\x00\x00\x01\x0a\x00\x00\x02"
    + b"\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\xf0\x02\xff\xff\x00\x00\x00\x00"
)
# Linux SLL (cooked) truncated header (< 16 bytes)
for i in range(1, 16):
    emit(b"\x00\x04" + b"\x00" * (i - 2)) if i >= 2 else emit(b"\x00")
# Linux SLL full 16-byte header + IPv4 ethertype + nothing
emit(b"\x00\x00\x00\x01\x00\x06" + b"\x00" * 8 + b"\x08\x00")
# Wrong/unknown ethertype
emit(bytes.fromhex("ffffffffffff000000000000") + b"\xde\xad" + b"\x01\x02\x03\x04")
# All-0xff jumbo
emit(b"\xff" * 64)
emit(b"\xff" * 1500)
# VLAN-tagged (0x8100) then truncated
emit(bytes.fromhex("ffffffffffff000000000000") + b"\x81\x00\x00\x01\x08\x00" + b"\x45")
# Nested double-VLAN
emit(bytes.fromhex("ffffffffffff000000000000") + b"\x81\x00\x00\x01\x81\x00\x00\x02\x86\xdd")

print(f"WROTE {written} unique seeds; total real frames sampled: {len(real_frames)}")
print(f"CORPUS SIZE: {len(os.listdir(CORPUS))} files")
