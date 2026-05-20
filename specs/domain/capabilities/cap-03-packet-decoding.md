---
artifact: L2-cap-03
traces_to: ../domain-spec.md
cap_id: CAP-03
title: Packet Decoding (L2-L4)
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# CAP-03: Packet Decoding (L2-L4)

## What the system does today

For each `RawPacket`, `decode_packet` in decoder.rs strips the link-layer header and parses
the IP (L3) and transport (L4) headers using the `etherparse` crate. The result is a
`ParsedPacket` (E-8) that carries `src_ip`, `dst_ip`, `protocol`, `transport`, `payload`,
and `packet_len`.

**Sources:** C-5 decoder.rs. BC-DEC-001..015.

## Output: ParsedPacket (E-8)

```
ParsedPacket {
    src_ip: IpAddr,       -- IPv4 or IPv6
    dst_ip: IpAddr,
    protocol: Protocol,   -- Tcp / Udp / Icmp / Other(u8)
    transport: TransportInfo, -- Tcp{...} / Udp{...} / None
    payload: Vec<u8>,     -- TCP or UDP application-layer payload
    packet_len: usize,    -- link-layer frame length (for bytes-accounting)
}
```

## Protocol identification

`Protocol` (E-6) is set from the IP protocol number:
- 6 -> `Protocol::Tcp`
- 17 -> `Protocol::Udp`
- 1 -> `Protocol::Icmp`
- other -> `Protocol::Other(u8)`

## Transport field population

`TransportInfo` (E-7):
- For TCP: src/dst ports, seq_number, and the four flag booleans (syn, ack, fin, rst).
- For UDP: src/dst ports only.
- For ICMP and unknown: `TransportInfo::None`.

## Snaplen-truncated packet handling (#91 / #94)

Captures produced by `tcpdump -s <snap>` have packets truncated at the snaplen boundary.
The decoder uses a strict-first, lax-fallback strategy:

1. Try `etherparse::SlicedPacket::from_ethernet` (or link-type equivalent) for strict parse.
2. If the strict parse returns `SliceError::Len` (length error -- the truncation signal),
   retry with `etherparse::LaxSlicedPacket` which clamps fields at the available bytes.
3. If the strict parse returns any OTHER error class (malformed, wrong type, etc.), the
   packet is rejected. Lax recovery is never applied to structurally malformed packets.

This restores the TCP header and as much of the payload as survives; the reassembly engine
then handles the resulting shorter-than-expected payload normally. `nfs_bad_stalls.cap` went
from 2,376 to 7,032 decoded TCP packets after this fix.

**Coupling note:** the fallback triggers on `SliceError::Len` specifically. This coupling to
etherparse's error taxonomy is documented in `decoder.rs` and in `Cargo.toml`; the
`"0.16"` version constraint is the guard against a future etherparse API break.

## Application-protocol hint

`ParsedPacket::app_protocol_hint()` returns a `&'static str` label (`DNS`, `HTTP`, `TLS`,
`SSH`, `SMB`, `Modbus`, `DNP3`, or `None`) based on port numbers alone. Used by `Summary`
for service-accounting only; NOT used for dispatch (which is content-first, see CAP-05).

## IPv4 / IPv6

Both are supported transparently via `IpAddr`. The link-type dispatch handles ETHERNET
(IPv4 or IPv6 EtherType), RAW/IPV4/IPV6 (header-only), and LINUX_SLL (both).

## BC references

BC-DEC-001..015 cover: all five link-type paths, IPv4/IPv6, TCP/UDP/ICMP flag extraction,
error paths, and the app_protocol_hint mapping.
