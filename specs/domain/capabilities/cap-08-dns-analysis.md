---
artifact: L2-cap-08
traces_to: ../domain-spec.md
cap_id: CAP-08
title: DNS Traffic Analysis
status: descriptive (brownfield)
reconciled: 2026-05-20
---

# CAP-08: DNS Traffic Analysis

## What the system does today

`DnsAnalyzer` (E-30, C-11) implements `ProtocolAnalyzer`. It increments query and response
counters by inspecting the QR bit in DNS packets. It emits NO `Finding` objects -- its
`analyze()` method returns an empty `Vec<Finding>` unconditionally.

**Sources:** C-11 analyzer/dns.rs (module-decomposition.md). BC-2.08.001..004.

## Implementation (89 LOC)

```rust
struct DnsAnalyzer {
    query_count:    u64,
    response_count: u64,
}
```

- `can_decode(packet)`: true when `TransportInfo` is `Udp` or `Tcp` AND `is_dns_port(src, dst)`
  returns true (i.e., src_port == 53 OR dst_port == 53). Source: dns.rs:52-60.
- `is_query(payload)`: inspects `payload[2] & 0x80` (byte 2, bit 7 of the DNS header flags
  word -- the QR bit). Returns `false` (counted as a RESPONSE) when `payload.len() < 12`,
  before the flag byte is accessed. Source: dns.rs:38-44.
- `analyze(packet)`: calls `is_query(payload)`. If true, `query_count += 1`; else
  `response_count += 1`. Returns `vec![]` unconditionally. Source: dns.rs:62-70.
- `summarize()`: returns `AnalysisSummary` with `dns_queries` and `dns_responses` keys in
  the detail map. Source: dns.rs:72-88.

## Short-packet edge case

`is_query` returns `false` -- i.e., the packet is counted as a response -- when
`payload.len() < 12`. The 12-byte minimum is the DNS header size (RFC 1035 S4.1.1).
A truncated or malformed packet that does not carry a full DNS header is therefore
classified as a response, not a query. This is a conservative false-negative (under-counting
queries) rather than a panic or undefined behavior.

## Severity of empty findings (Smell #5)

The always-empty return from `analyze()` is classified as low severity (Smell #5).
DNS query/response counting is operational. Future work would add DNS-based detection
findings (e.g., NXDOMAIN flood, tunneling signatures). Currently, DNS is a
statistics-only subsystem.

## BC references

BC-2.08.001: port-53 filter (can_decode gate).
BC-2.08.002: QR-bit dispatch (response_count if set; query_count otherwise).
BC-2.08.003: summarize emits AnalysisSummary with dns_queries/dns_responses.
BC-2.08.004: DnsAnalyzer NEVER emits Findings (statistics-only by design).
