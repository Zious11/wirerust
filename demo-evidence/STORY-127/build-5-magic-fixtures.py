#!/usr/bin/env python3
"""Build the 5-magic-value fixture directory for AC-001 demo.

Usage: python3 build-5-magic-fixtures.py <output-dir>

Writes:
  1-be.PCAP       -- CLASSIC_BE magic [D4 C3 B2 A1], LE header, 1 empty pkt
  2-le.CAP        -- CLASSIC_LE magic [A1 B2 C3 D4], BE header, 1 empty pkt
  3-ns-be.data    -- NS_BE magic [4D 3C B2 A1], LE header, 1 empty pkt
  4-ns-le.txt     -- NS_LE magic [A1 B2 3C 4D], BE header, 1 empty pkt
  5-ng.bin        -- PCAPNG magic [0A 0D 0D 0A], SHB+IDB+EPB, 1 empty pkt
  reject.pcap     -- Wrong magic [DE AD BE EF], must be silently excluded
"""
import struct
import os
import sys


def pcap1_pkt(magic: bytes, little_endian: bool) -> bytes:
    """Build a minimal valid classic pcap with 1 empty packet (40 bytes)."""
    if little_endian:
        hdr = magic + struct.pack('<HHiIII', 2, 4, 0, 0, 65535, 1)
        pkt = struct.pack('<IIII', 0, 0, 0, 0)
    else:
        hdr = magic + struct.pack('>HHiIII', 2, 4, 0, 0, 65535, 1)
        pkt = struct.pack('>IIII', 0, 0, 0, 0)
    data = hdr + pkt
    assert len(data) == 40, f"Expected 40 bytes, got {len(data)}"
    return data


def minimal_pcapng() -> bytes:
    """Build a minimal valid pcapng: SHB + IDB(ETHERNET) + EPB(empty)."""
    # SHB: 28 bytes
    shb = (
        b'\x0A\x0D\x0D\x0A'           # block type
        + struct.pack('<I', 28)         # block total length
        + b'\x4D\x3C\x2B\x1A'         # BOM (LE)
        + struct.pack('<H', 1)          # major = 1
        + struct.pack('<H', 0)          # minor = 0
        + struct.pack('<q', -1)         # section length (unspecified)
        + struct.pack('<I', 28)         # trailing btl
    )
    assert len(shb) == 28

    # IDB: 20 bytes (ETHERNET linktype)
    idb = (
        b'\x01\x00\x00\x00'           # block type
        + struct.pack('<I', 20)         # block total length
        + struct.pack('<H', 1)          # linktype = ETHERNET
        + struct.pack('<H', 0)          # reserved
        + struct.pack('<I', 65535)      # snaplen
        + struct.pack('<I', 20)         # trailing btl
    )
    assert len(idb) == 20

    # EPB: 32 bytes (empty payload)
    epb = (
        b'\x06\x00\x00\x00'           # block type
        + struct.pack('<I', 32)         # block total length
        + struct.pack('<I', 0)          # interface_id
        + struct.pack('<I', 0)          # ts_high
        + struct.pack('<I', 0)          # ts_low
        + struct.pack('<I', 0)          # captured_len = 0
        + struct.pack('<I', 0)          # original_len = 0
        + struct.pack('<I', 32)         # trailing btl
    )
    assert len(epb) == 32

    return shb + idb + epb


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <output-dir>", file=sys.stderr)
        sys.exit(1)

    out_dir = sys.argv[1]
    os.makedirs(out_dir, exist_ok=True)

    # 5 valid capture files with non-standard extensions
    with open(os.path.join(out_dir, '1-be.PCAP'), 'wb') as f:
        f.write(pcap1_pkt(b'\xD4\xC3\xB2\xA1', little_endian=True))
    with open(os.path.join(out_dir, '2-le.CAP'), 'wb') as f:
        f.write(pcap1_pkt(b'\xA1\xB2\xC3\xD4', little_endian=False))
    with open(os.path.join(out_dir, '3-ns-be.data'), 'wb') as f:
        f.write(pcap1_pkt(b'\x4D\x3C\xB2\xA1', little_endian=True))
    with open(os.path.join(out_dir, '4-ns-le.txt'), 'wb') as f:
        f.write(pcap1_pkt(b'\xA1\xB2\x3C\x4D', little_endian=False))
    with open(os.path.join(out_dir, '5-ng.bin'), 'wb') as f:
        f.write(minimal_pcapng())

    # Wrong-magic sentinel -- must be silently excluded
    with open(os.path.join(out_dir, 'reject.pcap'), 'wb') as f:
        f.write(b'\xDE\xAD\xBE\xEF' + b'\x00' * 4)

    print(f"Written to {out_dir}:")
    for name in sorted(os.listdir(out_dir)):
        size = os.path.getsize(os.path.join(out_dir, name))
        print(f"  {name:<25} {size:>4} bytes")


if __name__ == '__main__':
    main()
