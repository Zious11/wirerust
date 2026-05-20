---
artifact: L2-ent-03
traces_to: ../domain-spec.md
title: Entities -- Dispatch and Protocol Analysis (L2-L3)
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# Entities: Dispatch and Protocol Analysis (L2-L3)

Covers E-16, E-17, E-21, E-22, E-29, E-30, E-31, E-32, E-33, E-34, E-35, E-40, E-41.
Source: pass-2-domain-model.md + pass-2-R2.md + pass-2-R3.md.

## E-16: StreamHandler (src/reassembly/handler.rs:48) [trait]

```rust
pub trait StreamHandler {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64);
    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason);
}
```

The data-sink half of the two-trait split (ADR 0002). No L3 types in signature. Implementors:
`StreamDispatcher`, `HttpAnalyzer`, `TlsAnalyzer`, `RecordingHandler` (test fixture).

## E-17: StreamAnalyzer (src/reassembly/handler.rs:54) [trait]

```rust
pub trait StreamAnalyzer: StreamHandler {
    fn name(&self) -> &'static str;
    fn summarize(&self) -> AnalysisSummary;  // L3 type
    fn findings(&self) -> Vec<Finding>;       // L3 type
}
```

Supertrait of `StreamHandler`. Returns L3 types (`AnalysisSummary`, `Vec<Finding>`). This
is the only upward import (L2 imports L3) in the file-level DAG -- the formalized advisory
module-group cycle accepted by ADR 0002. Implementors: `HttpAnalyzer`, `TlsAnalyzer`.

## E-21: StreamDispatcher (src/dispatcher.rs:42-54)

```
struct StreamDispatcher {
    routes:                     HashMap<FlowKey, DispatchTarget>,  // private
    pub http:                   Option<HttpAnalyzer>,              // pub (Smell #6)
    pub tls:                    Option<TlsAnalyzer>,               // pub (Smell #6)
    unclassified_flows:         u64,                               // private
    max_classification_attempts: u32,                              // configurable knob
    classification_attempts:    HashMap<FlowKey, u32>,             // per-flow attempt counter
}
```

The pub fields expose analyzer internals. Main.rs drains findings by accessing
`dispatcher.http.as_ref().map(...)` and `dispatcher.tls.as_ref().map(...)` directly.

`max_classification_attempts` bounds the unbounded-reclassification cost for flows that
never produce enough bytes to classify (P2.11 / #80). Default:
`DEFAULT_MAX_CLASSIFICATION_ATTEMPTS`. When a flow's attempt count reaches the cap,
subsequent `on_data` calls for that flow forward to no analyzer. Configurable via
`StreamDispatcher::with_max_classification_attempts()`.

## E-22: DispatchTarget (src/dispatcher.rs:23-28) [module-private]

```
enum DispatchTarget { Http, Tls, None }
```

Module-private (no `pub`). `None` operates in two phases (dispatcher.rs:137-148;
LESSON-P2.11):
Phase 1 (attempt count < `max_classification_attempts`): NOT cached in `routes`; triggers
reclassification on each subsequent `on_data` call, incrementing the per-flow
`classification_attempts` counter (INV-2 / BC-DSP-005).
Phase 2 (attempt count reaches `max_classification_attempts`): inserted PERMANENTLY into
`routes`; `classify()` is no longer called for that flow. `classification_attempts` entry
is removed.

## E-29: ProtocolAnalyzer (src/analyzer/mod.rs:52) [trait]

```rust
pub trait ProtocolAnalyzer {
    fn name(&self) -> &'static str;
    fn can_decode(&self, packet: &ParsedPacket) -> bool;
    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding>;
    fn summarize(&self) -> AnalysisSummary;
}
```

Packet-level analyzer trait (ADR 0002). Only `DnsAnalyzer` implements it. Intended for
future ARP, ICMP, and other packet-level protocols.

## E-30: DnsAnalyzer (src/analyzer/dns.rs:15)

```
struct DnsAnalyzer { query_count: u64, response_count: u64 }
```

Implements `ProtocolAnalyzer`. `analyze()` returns `vec![]` unconditionally (Smell #5).

## E-31: HttpAnalyzer (src/analyzer/http.rs:114)

See CAP-06 for full field description. Key counters: `transactions`, `parse_errors`,
`non_http_flows`, `poisoned_bytes_skipped`. `all_findings: Vec<Finding>` is unbounded.
Implements `StreamHandler + StreamAnalyzer`.

## E-32: HttpFlowState (src/analyzer/http.rs:82) [module-private]

See CAP-06. Seven fields. `request_poisoned` / `response_poisoned` are monotonic false->true.
`counted_as_non_http` is a per-flow (not per-direction) one-way latch.

## E-33: TlsAnalyzer (src/analyzer/tls.rs:298)

See CAP-07 for full field description. Bounded by `MAX_BUF=65,536`, `MAX_MAP_ENTRIES=50,000`,
`MAX_RECORD_PAYLOAD=18,432`. `all_findings: Vec<Finding>` is unbounded. `truncated_records: u64`
counter added P1.05 (#73); TlsAnalyzer now conforms to CNV-PAT-002. Implements
`StreamHandler + StreamAnalyzer`.

## E-34: TlsFlowState (src/analyzer/tls.rs:273) [module-private]

```
struct TlsFlowState {
    client_buf:         Vec<u8>,  // max MAX_BUF
    server_buf:         Vec<u8>,  // max MAX_BUF
    client_hello_seen:  bool,
    server_hello_seen:  bool,
}
```

`done()` returns true when both hellos seen; subsequent `on_data` calls early-exit. State
record persists in the HashMap until `on_flow_close` fires.

## E-35: SniValue (src/analyzer/tls.rs:200) [module-private]

```
enum SniValue {
    Ascii(String),
    AsciiWithControl { hostname: String, hex: String },
    NonAsciiUtf8 { hostname: String, hex: String },
    NonUtf8 { lossy: String, hex: String },
}
```

4-way RFC 6066 / DNS-LDH conformance classifier. See CAP-07 for the full precedence rule.
The `is_ascii()` predicate is the controlling gate; mixed control+non-ASCII SNI routes to
`NonAsciiUtf8`, not `AsciiWithControl` (BC-TLS-037; INV-5).

## E-40: ParsedRequest (src/analyzer/http.rs:26) [module-private]

```
struct ParsedRequest {
    bytes_consumed: usize,
    method:         String,
    uri:            String,
    version:        u8,
    host:           Option<String>,
    user_agent:     Option<String>,
}
```

`'static`-safe (all owned types; constructed by copying from httparse's borrow references).
`host` and `user_agent` encode a 3-state space: `None` (absent), `Some("")` (present-empty),
`Some(non_empty)`. Host detection uses all 3 states (both None and Some("") fire findings
post-#71). UA detection uses only the Some("") state; absent UA is intentionally silent
(open item O-02; research rationale documented in http.rs:319-343).

## E-41: ParsedResponse (src/analyzer/http.rs:52) [module-private]

```
struct ParsedResponse { bytes_consumed: usize, status_code: u16 }
```

Minimal response snapshot used for `status_codes` counter increment.
