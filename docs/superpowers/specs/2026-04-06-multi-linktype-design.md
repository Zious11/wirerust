# Multi-Link-Type Support Design

**Issue:** #14 — Support non-Ethernet link types (Raw IP, Linux Cooked, pcapng)
**Scope:** Pcap link types only (Ethernet, Raw IP, Linux Cooked SLL). Pcapng deferred.

## Problem

wirerust hardcodes `SlicedPacket::from_ethernet()` in `src/decoder.rs`. Pcap files with non-Ethernet link types silently produce empty results — `decode_packet()` fails and the error is swallowed by `if let Ok(parsed)` in `main.rs`. The user sees `Packets: 0` with no warning.

This affects ~40% of real-world incident response captures (cloud VPC mirrors use Raw IP, `tcpdump -i any` produces Linux Cooked).

## Supported Link Types

| DataLink Variant | Link Type ID | etherparse Parser | Source |
|------------------|-------------|-------------------|--------|
| `DataLink::ETHERNET` | 1 | `SlicedPacket::from_ethernet()` | SPAN ports, Wireshark, most captures |
| `DataLink::RAW` | 101 | `SlicedPacket::from_ip()` | Cloud VPC mirrors, mobile, some tools |
| `DataLink::LINUX_SLL` | 113 | `SlicedPacket::from_linux_sll()` | `tcpdump -i any`, Docker captures |
| `DataLink::IPV4` | 228 | `SlicedPacket::from_ip()` | Raw IPv4 captures (same as RAW but IPv4-only) |
| `DataLink::IPV6` | 229 | `SlicedPacket::from_ip()` | Raw IPv6 captures |

All other link types are rejected at file open with a clear error message.

## Architecture

Three layers change, each with a single responsibility:

### Reader Layer (`src/reader.rs`)

`PcapSource` gains a `datalink: DataLink` field read from `pcap_reader.header().datalink`.

Unsupported link types are rejected immediately in `from_pcap_reader()` with:
`"Unsupported pcap link type: {datalink:?}. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)"`

This is a fail-fast design: unsupported formats error at file open, not silently per-packet.

### Decoder Layer (`src/decoder.rs`)

`decode_packet()` gains a `datalink: DataLink` parameter. A match expression dispatches to the correct etherparse parser:

- `DataLink::ETHERNET` → `SlicedPacket::from_ethernet(data)`
- `DataLink::RAW` | `DataLink::IPV4` | `DataLink::IPV6` → `SlicedPacket::from_ip(data)`
- `DataLink::LINUX_SLL` → `SlicedPacket::from_linux_sll(data)`
- `_` → error (defense-in-depth; reader already rejects unsupported types)

Everything after `SlicedPacket` construction is unchanged. The `net`, `transport`, and `payload` fields on `SlicedPacket` are identical in structure regardless of which `from_*()` method was used. Only the `link` field differs (populated for Ethernet/SLL, empty for Raw IP), and wirerust does not access it.

### Error Surfacing (`src/main.rs`)

The processing loops in `run_analyze()` and `run_summary()` gain a `decode_errors: u32` counter. On first decode failure, a warning is printed to stderr (not stdout, to keep piped output clean):

`"Warning: failed to decode packet (link type: {datalink:?}). Further errors will be counted silently."`

After the loop, if `decode_errors > 0`, the count is included in summary output so the user knows data was lost.

## Data Flow

```
PcapSource::from_file(path)
  → PcapReader::new(reader)
  → header().datalink → validate supported → store on PcapSource
  → packets loaded as before

Processing loop:
  for raw in source.packets:
    decode_packet(&raw.data, source.datalink)  // dispatches by link type
      → SlicedPacket::from_ethernet/from_ip/from_linux_sll
      → extract net/transport/payload (unchanged)
    on Err: increment decode_errors, warn on first
```

## Testing

### Existing Fixtures

All three link types are already present in `tests/fixtures/`:

| File | Link Type | Current Status | Expected After |
|------|-----------|---------------|----------------|
| `tls.pcap` | Ethernet (1) | Works | Works (unchanged) |
| `segmented.pcap` | Raw IP (101) | Silent 0 packets | Parses successfully |
| `http-ooo.pcap` | Raw IPv4 (228) | Silent 0 packets | Parses successfully |

### New Tests

1. **Integration test per link type** — load each fixture via `PcapSource::from_file()`, assert `packets.len() > 0`, decode all packets and verify no errors.
2. **Unsupported link type test** — construct a pcap with an unsupported link type, verify `from_pcap_reader()` returns a clear error (not silent empty output).
3. **Decode error counting** — verify that decode failures are counted (unit test on the counter logic, not full CLI integration).

### Existing Tests

All current tests use Ethernet fixtures or construct packets directly. They pass without modification because `decode_packet()` is backward-compatible (callers just add the `datalink` parameter).

## Acceptance Criteria

From issue #14:
- [ ] `wirerust analyze segmented.pcap` parses packets (currently shows 0)
- [ ] Unsupported link types produce a clear error, not silent empty output
- [ ] Decode failure count shown in summary when packets are skipped

## Not In Scope

- **pcapng format** — structurally different (per-interface link types). Deferred to a separate issue.
- **SLL v2** (`DataLink::LINUX_SLL2`) — etherparse does not have `from_linux_sll2()`. Deferred.
- **Other link types** — can be added incrementally as etherparse adds parsers.
- **MAC address extraction** — the `link` field on `SlicedPacket` is populated for Ethernet/SLL but wirerust doesn't use it yet. Future work.
