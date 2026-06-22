"""Create pcapng fixtures for the per-file isolation demo."""
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


def idb(lt=1):
    return struct.pack("<II", 1, 20) + struct.pack("<HHI", lt, 0, 65535) + struct.pack("<I", 20)


def epb():
    btl = 32
    return struct.pack("<II", 6, btl) + struct.pack("<IIIII", 0, 0, 0, 0, 0) + struct.pack("<I", btl)


d = "/tmp/demo-128-iso"
os.makedirs(d, exist_ok=True)
# a-conflict.pcapng: ETHERNET + LINUX_SLL IDBs -> E-INP-011 conflict (sorts FIRST)
open(d + "/a-conflict.pcapng", "wb").write(shb() + idb(1) + idb(113))
# b-valid.pcapng: SHB + ETHERNET IDB + 1 EPB (sorts SECOND)
open(d + "/b-valid.pcapng", "wb").write(shb() + idb(1) + epb())
print("fixtures ready: a-conflict.pcapng (bad), b-valid.pcapng (good)")
