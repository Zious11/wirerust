---
artifact: architecture-section
section: dependency-graph
traces_to: ARCH-INDEX.md
version: "1.2"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-08
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-12 corpus debt cleanup (F-3): added analyzer/dnp3.rs (holds Option<Dnp3Analyzer>) [C-24, ADR-007] to dispatcher.rs import DAG, after the modbus.rs line. DNP3 shipped v0.6.0 with DispatchTarget::Dnp3 at src/dispatcher.rs:238/309/345. ARP (C-23) stays absent — PLANNED, NON-BLOCKING."
---

# Dependency Graph

## Internal Module DAG

The import graph is acyclic at the file level except for the single accepted L2<->L3 cycle
documented below.

```
main.rs / lib.rs
  |-- cli.rs
  |-- reader.rs
  |-- decoder.rs
  |-- reassembly/mod.rs
  |     |-- reassembly/config.rs
  |     |-- reassembly/stats.rs
  |     |-- reassembly/flow.rs
  |     |-- reassembly/segment.rs
  |     |-- reassembly/lifecycle.rs
  |     |-- reassembly/handler.rs  <-- defines StreamHandler / StreamAnalyzer traits
  |-- dispatcher.rs
  |     |-- reassembly/handler.rs  (imports StreamHandler trait)
  |     |-- analyzer/http.rs       (holds Option<HttpAnalyzer>)
  |     |-- analyzer/tls.rs        (holds Option<TlsAnalyzer>)
  |     |-- analyzer/modbus.rs     (holds Option<ModbusAnalyzer>) [C-22, ADR-005]
  |     |-- analyzer/dnp3.rs       (holds Option<Dnp3Analyzer>) [C-24, ADR-007]
  |-- analyzer/dns.rs
  |-- findings.rs
  |-- mitre.rs
  |-- summary.rs
  |-- reporter/mod.rs
        |-- reporter/json.rs
        |-- reporter/terminal.rs
        |-- reporter/csv.rs
```

### Layer Dependency Rules

| From Layer | May import | Must not import |
|-----------|------------|----------------|
| L0 Entry | Everything | (no restriction; orchestrates all) |
| L1 Ingest | types (findings, decoder) | L3 analyzers, L4 reporters |
| L2 Stream | L3 types via handler.rs traits | L4 reporters |
| L3 Domain | findings.rs, mitre.rs | L2 engine internals, L4 reporters |
| L4 Output | L3 (findings, mitre, summary), L2 via summarize | L1 ingest, L2 internals |


## The Accepted L2<->L3 Cycle

**Nature:** `reassembly/handler.rs` (L2) defines the `StreamHandler` and `StreamAnalyzer`
traits. `analyzer/http.rs` and `analyzer/tls.rs` (L3) implement those traits. This
creates a logical dependency loop: L2 defines an interface that L3 implements, but L2
also imports L3 types through `dispatcher.rs` which holds the concrete analyzers.

**Why accepted (ADR 0002):** The two-trait split is the core of the modular analyzer
pattern. Splitting the traits out to a separate crate or reversing the dependency
direction would require either a shared-types crate (premature for a single-crate tool)
or would force the reassembly engine to know about concrete analyzers (worse coupling).
The cycle exists at the "interface boundary" level, not as a state-sharing cycle.

**Formal verification impact:** Both sides of the cycle are testable independently. The
segment buffer (C-8) has no knowledge of analyzers. HttpAnalyzer tests bypass the
reassembly engine entirely and call `on_data` directly. The cycle does not prevent
independent property verification.


## External Crate Dependencies (14 direct production deps)

Verified against Cargo.toml @ 0082a0c. Every row name and version matches the
`[dependencies]` table exactly.

| Crate | Version (Cargo.toml) | Used By | Purpose |
|-------|---------------------|---------|---------|
| `httparse` | 1 | analyzer/http.rs | HTTP/1.x request + response parsing |
| `tls-parser` | 0.12 | analyzer/tls.rs | TLS record + handshake parsing |
| `md-5` | 0.11 | analyzer/tls.rs | JA3/JA3S fingerprint computation (exposes `md5` module) |
| `clap` | 4 (derive feature) | cli.rs | CLI argument parsing |
| `etherparse` | 0.16 | decoder.rs (C-5) | L2-L4 header parsing (Ethernet, IP, TCP, UDP); pinned to 0.16 API contract |
| `pcap-file` | 2 | reader.rs (C-4) | Classic pcap file format parsing |
| `serde` | 1 (derive feature) | findings.rs, reporter/*.rs | Serialization traits |
| `serde_json` | 1 | reporter/json.rs, analyzer/{http,tls}.rs | JSON serialization; RFC 8259 escaping |
| `csv` | 1 | reporter/csv.rs | CSV serialization |
| `anyhow` | 1 | reader.rs, decoder.rs, main.rs | Error propagation with context |
| `owo-colors` | 4 | reporter/terminal.rs | Terminal colorization |
| `indicatif` | 0.17 | main.rs | Per-target progress bar on stderr |
| `chrono` | 0.4 (serde feature) | main.rs / findings.rs | Timestamp types (O-01: field exists but universally None) |
| `rayon` | 1 | (present in Cargo.toml; unused in current call-paths; tracked as domain-debt O-07) | Work-stealing parallelism (not yet wired in) |

> Exact pinned versions are in Cargo.lock. Cargo.toml version specs above use
> caret semantics. `rayon` is a real direct production dependency as of
> Cargo.toml:28 -- it has not been removed.

## Dev / Test Dependencies

| Crate | Version (Cargo.toml) | Purpose |
|-------|---------------------|---------|
| `assert_cmd` | 2 | CLI integration tests |
| `predicates` | 3 | CLI test assertions |
| `tempfile` | 3 | Temporary file/directory fixtures for tests |
| `proptest` | 1 | Property-based testing (VP harnesses) |
| `criterion` | 0.8 | Micro-benchmarks for hot pcap-processing paths (`cargo bench`) |
| Inline test modules | -- | 264 tests in `tests/` + 18 inline (11 in `reporter/terminal.rs`, 7 in `analyzer/tls.rs`) = 282 total |
