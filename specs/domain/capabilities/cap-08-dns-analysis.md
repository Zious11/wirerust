---
artifact: L2-cap-08
traces_to: ../domain-spec.md
cap_id: CAP-08
title: DNS Traffic Analysis
status: descriptive (brownfield)
---

# CAP-08: DNS Traffic Analysis

## What the system does today

`DnsAnalyzer` (E-30, C-13) implements `ProtocolAnalyzer`. It increments query and response
counters by inspecting the QR bit in DNS packets. It emits NO `Finding` objects -- its
`analyze()` method returns an empty `Vec<Finding>` unconditionally.

**Sources:** C-13 analyzer/dns.rs. BC-DNS-001..004.

## Implementation (81 LOC)

```
struct DnsAnalyzer {
    query_count:    u64,
    response_count: u64,
}
```

- `can_decode(packet)`: true if `protocol == Tcp || protocol == Udp` AND port 53 is src or dst.
- `analyze(packet)`: inspects payload[2] (QR bit at bit 15 of DNS flags). If set, `response_count += 1`; else `query_count += 1`. Returns `vec![]`.
- `summarize()`: returns `AnalysisSummary` with `query_count` and `response_count` in detail.

## Severity of empty findings (Smell #5)

The always-empty return from `analyze()` is classified as low severity (Smell #5). DNS
query/response counting is operational. Future work would add DNS-based detection findings
(e.g., NXDOMAIN flood, tunneling signatures). Currently, DNS is a statistics-only subsystem.

## BC references

BC-DNS-001..004: QR-bit dispatch (001/002), port-53 filter (003), empty-finding contract (004).
