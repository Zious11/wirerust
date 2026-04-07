# ADR 0001: Content-First Stream Protocol Dispatch

**Status:** Accepted  
**Date:** 2026-04-07  
**Context:** Issue #2 (TLS ClientHello analyzer) requires routing reassembled TCP streams to multiple protocol analyzers.

## Problem

The TCP reassembly engine (`TcpReassembler::process_packet`) accepts a single `&mut dyn StreamHandler`. Currently only the HTTP analyzer uses it. Adding a TLS analyzer (and eventually SSH, SMB, etc.) requires a mechanism to route each flow's reassembled data to the correct analyzer.

## Decision

Implement a **content-first StreamDispatcher** that classifies flows by inspecting the first bytes of stream data, with port-based fallback when content is ambiguous.

### Classification Logic

On the first `on_data` call for a flow:

1. **TLS**: `data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03` (TLS record header: content type Handshake + SSL/TLS 3.x version family)
2. **HTTP**: First bytes match an HTTP method (`GET `, `POST `, `PUT `, `DELETE `, `HEAD `, `OPTIONS `, `PATCH `, `CONNECT `, `TRACE `) or response (`HTTP/`)
3. **Fallback**: If data is too short (< 5 bytes) or matches neither signature, use port hints: 443/8443 → TLS, 80/8080 → HTTP
4. **None**: If no match, data is dropped (not forwarded to any analyzer)

The classification decision is cached per flow in a `HashMap<FlowKey, DispatchTarget>`.

### StreamDispatcher Struct

```rust
pub struct StreamDispatcher {
    routes: HashMap<FlowKey, DispatchTarget>,
    http: Option<HttpAnalyzer>,
    tls: Option<TlsAnalyzer>,
}

enum DispatchTarget {
    Http,
    Tls,
    None,
}

impl StreamHandler for StreamDispatcher {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64) {
        let target = self.routes.entry(flow_key.clone()).or_insert_with(|| {
            classify(data, flow_key)
        });
        match target {
            DispatchTarget::Http => { /* forward to self.http */ }
            DispatchTarget::Tls => { /* forward to self.tls */ }
            DispatchTarget::None => {}
        }
    }
    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason) {
        // forward to cached analyzer, remove route entry
    }
}
```

## Alternatives Considered

### Port-Based Only

Route by well-known port: 443 → TLS, 80 → HTTP.

- **Pro:** Simplest implementation, zero content inspection overhead.
- **Con:** Misses TLS on non-standard ports (8443, 4443). Fails completely when protocols masquerade on other ports (TLS on port 80, HTTP on port 443). Zeek, Suricata, and Wireshark all moved beyond pure port-based detection for this reason.
- **Rejected:** Insufficient for real-world PCAP forensics.

### Broadcast to All Analyzers

Send all reassembled data to all enabled analyzers. Each self-filters.

- **Pro:** No routing logic needed.
- **Con:** Every analyzer buffers all traffic. HTTP already buffers up to 64KB per flow direction — with N analyzers this multiplies memory usage. Suricata, Zeek, and Wireshark do not use this approach.
- **Rejected:** Unacceptable memory overhead.

### Port-First Hybrid

Check port first (fast path), content detection only for unknown ports.

- **Pro:** Slightly faster for common case.
- **Con:** Misroutes masquerading traffic. If TLS runs on port 80, port check sends it to HTTP. Content detection must override port hints to handle this, making port-first ordering harmful rather than helpful.
- **Rejected:** Content-first is both more correct and equally simple.

## Rationale

- **Matches industry standard.** Zeek's Dynamic Protocol Detection, Suricata's protocol detection engine, and Wireshark's dissector table all use content-based detection as the primary mechanism with ports as hints/fallback. This was validated via Perplexity queries against current documentation.
- **Handles masquerading.** TLS on port 80, HTTP on port 443, and protocols on arbitrary ports are all correctly classified.
- **Minimal overhead.** Classification requires reading 5 bytes on the first data delivery per flow — negligible compared to reassembly and parsing costs.
- **TLS signature is unambiguous for TCP.** The 5-byte TLS record header (`0x16 0x03 0xNN` + 2-byte length) does not collide with any common TCP application protocol. All text-based protocols (HTTP, FTP, SMTP, SIP) start with ASCII bytes. Binary protocol collisions are practically negligible.
- **Extensible.** Adding SSH (first bytes `SSH-`), SMB (first bytes `\x00\x00`+NetBIOS), or other analyzers requires only adding a classification branch and an `Option<Analyzer>` field.

## Consequences

- **New struct:** `StreamDispatcher` in `src/reassembly/handler.rs` or `src/dispatcher.rs`.
- **main.rs changes:** Replace direct `HttpAnalyzer` handler with `StreamDispatcher` wrapping both `Option<HttpAnalyzer>` and `Option<TlsAnalyzer>`.
- **Per-flow routing map:** Small memory overhead (~64 bytes per flow for the HashMap entry). Cleaned up on `on_flow_close`.
- **Future analyzers** register content signatures in the dispatcher rather than changing the reassembly engine.
- **Edge case:** If the first `on_data` delivery has < 5 bytes (extremely rare — requires pathological TCP segmentation), the dispatcher falls back to port hints. This matches Zeek's approach with its `dpd_buffer_size` parameter, though Zeek buffers up to 1024 bytes. For wirerust v1, single-delivery classification with port fallback is sufficient.

## Validation

This decision was validated through Perplexity queries on 2026-04-07:
- Zeek DPD architecture: content signatures primary, ports as hints
- Suricata protocol detection: content-based with app-layer auto-detection
- Wireshark dissector routing: content-based with "Decode As" port override
- TLS record header collision risk: negligible for TCP protocols
- Small initial segment handling: standard buffering, port fallback
