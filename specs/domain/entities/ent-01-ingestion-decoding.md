---
artifact: L2-ent-01
traces_to: ../domain-spec.md
title: Entities -- Ingestion and Decoding Layer (L0-L1)
version: "1.1"
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
changelog:
  - "v1.1: Burst-10 (O-01-closure) — E-4 RawPacket note updated: O-01 CLOSED; timestamp_secs now threaded to Finding.timestamp via STORY-097/098/099 (http/tls/reassembly) and STORY-102..110 (modbus/dnp3); BC-2.04.054 retains timestamp:None by design as the sole exception. — 2026-06-13"
---

# Entities: Ingestion and Decoding (L0-L1)

Covers E-1 through E-8. These entities span the CLI, reader, and decoder components
(C-1..C-5). Source: pass-2-domain-model.md entity table (41 entities confirmed).

## E-1: Cli (src/cli.rs:52-126)

Clap-derive `Parser` struct. Root CLI entry point.

Key fields and their current status (all flags are wired as of P1.04 / #74 + P2.05 / #88-#96):

| Field | Type | Status |
|---|---|---|
| no_color | bool | Wired; controls TerminalReporter.use_color |
| output_format | Option<OutputFormat> | Wired; Json -> JsonReporter, Csv -> CsvReporter |
| json | Option<PathBuf> | Wired; routes to JsonReporter with file output |
| csv | Option<PathBuf> | Wired; routes to CsvReporter with file output |
| reassemble | bool | Wired; conflicts_with = "no_reassemble" |
| no_reassemble | bool | Wired |
| reassembly_depth | usize | Wired; default 10 (multiplied to 10 MB) |
| reassembly_memcap | usize | Wired; default 1024 (multiplied to 1 GB) |
| overlap_threshold | u8 | Wired; range 0-255; maps to overlap_alert_threshold (P2.05 / #88) |
| small_segment_threshold | u16 | Wired; range 0-2048; maps to small_segment_alert_threshold (#93) |
| small_segment_max_bytes | u16 | Wired; range 0-2048; maps to small_segment_max_bytes (#93) |
| small_segment_ignore_ports | Vec<u16> | Wired; maps to small_segment_ignore_ports (#93) |
| out_of_window_threshold | u16 | Wired; range 0-2048; maps to out_of_window_alert_threshold (#96) |
| command | Commands | Wired |

Range validation on threshold flags is enforced at parse time (#96). The struct-level comment
in cli.rs documents the "no unwired flags" convention established by P1.04.

The `--verbose`, `--threats`, `--beacon`, `--filter`, and `--services` flags that previously
existed as unwired declarations were removed by P1.04 (#74).

## E-2: Commands (src/cli.rs:129-170)

Clap-derive `Subcommand` enum with two variants:

**`Analyze { targets, dns, http, tls, mitre, all }`**

| Field | Status |
|---|---|
| targets: Vec<PathBuf> | Wired |
| dns: bool | Wired (gates DnsAnalyzer creation) |
| http: bool | Wired (gates HttpAnalyzer creation) |
| tls: bool | Wired (gates TlsAnalyzer creation) |
| mitre: bool | Wired (controls TerminalReporter.show_mitre_grouping) |
| all: bool | Wired (implies dns+http+tls) |

**`Summary { targets, hosts }`**

| Field | Status |
|---|---|
| targets: Vec<PathBuf> | Wired |
| hosts: bool | Wired; controls TerminalReporter.show_hosts_breakdown (P1.03) |

All flags in both subcommands are now wired. The 8 previously-unwired flags
(threats, beacon, filter, services, verbose, and 3 others) were removed by P1.04 (#74).

## E-3: OutputFormat (src/cli.rs:27-30)

```
enum OutputFormat { Json, Csv }
```

Both variants are wired. `Json` routes to `JsonReporter`; `Csv` routes to `CsvReporter`
(implemented P2.03 / #84). Not `#[non_exhaustive]`.

## E-4: RawPacket (src/reader.rs:31-36)

```
struct RawPacket {
    timestamp_secs:  u32,  -- pcap record header; u32 wraps in 2106 (BC-RDR-005)
    timestamp_usecs: u32,
    data:            Vec<u8>,
}
```

Transient DTO. Emitted by PcapSource. Passed into `decode_packet`. Never enters L2+.
`timestamp_secs` is read and threaded to `Finding.timestamp` via STORY-097/098/099 (http/tls/reassembly
emission sites) and STORY-102..110 (modbus/dnp3 analyzers). Domain-debt O-01 is CLOSED (21 of 22
emission sites wired; BC-2.04.054 segment-limit summary retains timestamp:None by design as the sole
exception — see BC-2.09.007 and domain-debt.md).

## E-5: PcapSource (src/reader.rs:38-42)

```
struct PcapSource {
    packets:  Vec<RawPacket>,  -- entire file in memory
    datalink: DataLink,        -- pcap-file crate type (Smell #7)
}
```

In-memory pcap representation. `datalink` leaks the external `pcap-file::DataLink` type.

Reader now accepts snaplen-truncated captures where `orig_len > snap_len` (fixed #87).

## E-6: Protocol (src/decoder.rs:56-62)

```
enum Protocol { Tcp, Udp, Icmp, Other(u8) }
```

Derives: `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize`.
`Hash` enables use as a `Summary.protocols` HashMap key. `Other(u8)` carries the raw IP
protocol byte for protocols not otherwise classified.

## E-7: TransportInfo (src/decoder.rs:64-80)

```
enum TransportInfo {
    Tcp { src_port, dst_port, seq_number, syn, ack, fin, rst },
    Udp { src_port, dst_port },
    None,
}
```

NOT `Serialize`. Stays inside the binary.

## E-8: ParsedPacket (src/decoder.rs:82-90)

```
struct ParsedPacket {
    src_ip:     IpAddr,
    dst_ip:     IpAddr,
    protocol:   Protocol,
    transport:  TransportInfo,
    payload:    Vec<u8>,   -- TCP/UDP application-layer bytes only
    packet_len: usize,     -- link-layer frame length
}
```

`ParsedPacket` is the boundary output of L1 Ingest. It flows into `TcpReassembler`,
`DnsAnalyzer`, and `Summary::ingest`.
