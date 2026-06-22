"""Create pcapng fixtures for the zero-packet notice demo."""
import struct
import os


def shb():
    b = bytearray()
    b += b"\x0A\x0D\x0D\x0A"
    b += struct.pack("<I", 28)
    b += b"\x4D\x3C\x2B\x1A"
    b += struct.pack("<HH", 1, 0)
    b += struct.pack("<q", -1)
    b += struct.pack("<I", 28)
    return bytes(b)


def idb():
    return struct.pack("<II", 1, 20) + struct.pack("<HHI", 1, 0, 65535) + struct.pack("<I", 20)


def opb():
    body = struct.pack("<HH", 0, 0) + struct.pack("<IIII", 0, 0, 0, 0)
    btl = 12 + len(body)
    return struct.pack("<II", 2, btl) + body + struct.pack("<I", btl)


def skip_block():
    body = b"\xAA\xBB\xCC\xDD\xEE\xFF\x00\x11"
    btl = 12 + len(body)
    return struct.pack("<II", 0x99, btl) + body + struct.pack("<I", btl)


d = "/tmp/demo-128-notice"
os.makedirs(d, exist_ok=True)
# Case 1: SHB-only -> bare notice (no parenthetical)
open(d + "/shb-only.pcapng", "wb").write(shb())
# Case 2: SHB + IDB + 1 OPB -> notice + OPB clause
open(d + "/opb-bearing.pcapng", "wb").write(shb() + idb() + opb())
# Case 3: SHB + IDB + 2 unknown skip-blocks -> notice + generic segment
open(d + "/unknown-skip.pcapng", "wb").write(shb() + idb() + skip_block() + skip_block())
print("fixtures ready: shb-only, opb-bearing, unknown-skip")
