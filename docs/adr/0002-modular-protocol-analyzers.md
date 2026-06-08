# ADR 0002: Modular Protocol Analyzer Pattern

**Status:** Accepted  
**Date:** 2026-04-07  
**Context:** wirerust has two analyzer patterns (DNS packet-level, HTTP stream-level) and is adding TLS. Codifying the pattern prevents drift as more analyzers are added.

## Decision

Protocol analyzers are self-contained modules that implement one of two traits depending on whether they operate on individual packets or reassembled TCP streams.

### Two Trait Levels

**Packet-level** — `ProtocolAnalyzer` trait. Receives individual parsed packets. No TCP reassembly required. Used for protocols that fit in a single packet (DNS over UDP, ARP, ICMP).

```rust
pub trait ProtocolAnalyzer {
    fn name(&self) -> &'static str;
    fn can_decode(&self, packet: &ParsedPacket) -> bool;
    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding>;
    fn summarize(&self) -> AnalysisSummary;
}
```

**Stream-level** — `StreamAnalyzer` trait (extends `StreamHandler`). Receives reassembled, ordered TCP stream data. Used for protocols that span multiple packets or require connection context (HTTP, TLS, SSH, SMB).

```rust
pub trait StreamHandler {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64);
    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason);
}

pub trait StreamAnalyzer: StreamHandler {
    fn name(&self) -> &'static str;
    fn summarize(&self) -> AnalysisSummary;
    fn findings(&self) -> Vec<Finding>;
}
```

### Internal Structure Pattern

Every analyzer follows the same internal structure:

```rust
pub struct FooAnalyzer {
    // 1. Per-flow state (stream analyzers only)
    flows: HashMap<FlowKey, FooFlowState>,

    // 2. Aggregate counters (keyed by protocol-specific dimensions)
    some_counts: HashMap<String, u64>,

    // 3. Findings
    all_findings: Vec<Finding>,

    // 4. Error tracking
    parse_errors: u64,
}
```

**Per-flow state** tracks buffered data and parsing progress for each TCP connection. Cleaned up on `on_flow_close`. Stream analyzers only.

**Aggregate counters** accumulate protocol-specific metrics across all flows (e.g., HTTP method counts, TLS SNI counts, DNS query counts). Bounded by `MAX_MAP_ENTRIES` to prevent memory exhaustion from cardinality explosion.

**Findings** are security-relevant observations with category, verdict, confidence, optional MITRE technique, summary, and evidence. Accumulated in `Vec<Finding>` and returned by `findings()`.

**Error tracking** counts parse failures. Surfaced in `summarize()` output so users know if data was lost. Not logged to stderr — the counter is the signal.

### Required Methods and Accessors

| Method | Purpose | Required |
|--------|---------|----------|
| `new()` | Constructor with zero-initialized state | Yes |
| `name()` | Returns `&'static str` like `"HTTP"`, `"TLS"`, `"DNS"` | Yes |
| `summarize()` | Returns `AnalysisSummary` with `detail: HashMap<String, serde_json::Value>` | Yes |
| `findings()` | Returns `Vec<Finding>` | Yes (stream), via `analyze()` return (packet) |
| `parse_error_count()` | Returns `u64` | Yes |
| Domain-specific accessors | e.g., `sni_counts()`, `method_counts()` | For testing |

### Adding a New Analyzer

1. Create `src/analyzer/{protocol}.rs`
2. Implement `ProtocolAnalyzer` (packet-level) or `StreamAnalyzer` (stream-level)
3. Add `pub mod {protocol}` to `src/analyzer/mod.rs`
4. **Packet-level**: Add `can_decode` + `analyze` call in the packet loop in `main.rs`
5. **Stream-level**: Add `Option<FooAnalyzer>` to `StreamDispatcher`, add content signature to classification logic (ADR 0001)
6. Add CLI flag to `src/cli.rs` if needed (or reuse existing flag)
7. Wire up `findings()` and `summarize()` collection in `main.rs`
8. Add `tests/{protocol}_analyzer_tests.rs`

### AnalysisSummary Format

All analyzers produce the same output structure:

```rust
pub struct AnalysisSummary {
    pub analyzer_name: String,
    pub packets_analyzed: u64,
    pub detail: HashMap<String, serde_json::Value>,
}
```

The `detail` map contains protocol-specific fields as `serde_json::Value`. This allows the reporter to render any analyzer's output without knowing its internal structure. The JSON reporter serializes it directly; the terminal reporter can pattern-match on known keys.

### Finding Generation Guidelines

- Generate findings only for **unambiguous security concerns** — not informational observations
- Follow the existing verdict/confidence model: `Likely`/`Inconclusive`/`Unlikely` x `High`/`Medium`/`Low`
- Include MITRE ATT&CK technique ID only when there's a clean mapping; `None` is better than a forced fit
- Include actionable evidence (specific values, not just "something was wrong")
- Cap findings with `MAX_FINDINGS` to prevent memory exhaustion on adversarial input
- **Output sanitization is a reporter responsibility, not an analyzer responsibility.** Store raw bytes (post-`from_utf8_lossy`) in `Finding.summary` and `Finding.evidence`. Do not escape, debug-format, or otherwise pre-encode untrusted bytes at the analyzer. See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`) for the full layering principle.

## Alternatives Considered

### Single Unified Trait

One trait covering both packet-level and stream-level analyzers.

- **Rejected:** Packet analyzers don't need `on_data`/`on_flow_close`, and stream analyzers don't need `can_decode`/`analyze(packet)`. A unified trait forces empty implementations.

### Plugin System with Dynamic Loading

Load analyzers as shared libraries at runtime.

- **Rejected:** Premature. wirerust has 3 analyzers. Dynamic loading adds complexity (ABI stability, unsafe FFI) with no current benefit. Can revisit if the analyzer count grows significantly.

### Analyzer Registry with Auto-Discovery

Analyzers register themselves in a global registry (e.g., via `inventory` crate).

- **Rejected:** Magic registration obscures control flow. Explicit wiring in `main.rs` is clearer and easier to debug. The number of analyzers is small enough that manual wiring is not a burden.

## Consequences

- **Consistency**: All analyzers follow the same pattern, making the codebase predictable for contributors.
- **Testability**: Public accessors on each analyzer enable direct unit testing without going through the full pipeline.
- **Isolation**: Each analyzer owns its state. No shared mutable state between analyzers. The dispatcher routes data; analyzers don't know about each other.
- **Bounded memory**: `MAX_MAP_ENTRIES` on counters, `MAX_FINDINGS` on findings, per-flow buffer caps — all analyzers must respect these.
- **Adding analyzers is cheap**: ~1 new file + trait impl + wiring in main.rs + dispatcher registration. No framework overhead.

## Existing Analyzers

| Analyzer | Trait | File | Since |
|----------|-------|------|-------|
| DNS | `ProtocolAnalyzer` | `src/analyzer/dns.rs` | v0.1.0 |
| HTTP | `StreamAnalyzer` | `src/analyzer/http.rs` | v0.1.0 |
| TLS | `StreamAnalyzer` | `src/analyzer/tls.rs` | Issue #2 (planned) |
