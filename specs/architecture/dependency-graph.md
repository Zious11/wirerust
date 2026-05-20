---
artifact: architecture-section
section: dependency-graph
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
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

| Crate | Version (approx) | Used By | Purpose |
|-------|-----------------|---------|---------|
| `pcap-file` | 2.x | reader.rs (C-4) | Classic pcap file format parsing |
| `etherparse` | 0.15.x | decoder.rs (C-5) | L2-L4 header parsing (Ethernet, IP, TCP, UDP) |
| `anyhow` | 1.x | reader.rs, decoder.rs, main.rs | Error propagation with context |
| `serde` | 1.x | findings.rs, reporter/*.rs | Serialization traits |
| `serde_json` | 1.x | reporter/json.rs, analyzer/{http,tls}.rs | JSON serialization; RFC 8259 escaping |
| `clap` | 4.x | cli.rs | CLI argument parsing |
| `indicatif` | 0.17.x | main.rs | Per-target progress bar on stderr |
| `csv` | 1.x | reporter/csv.rs | CSV serialization |
| `md5` | 0.10.x | analyzer/tls.rs | JA3/JA3S fingerprint computation |
| `colored` | 2.x | reporter/terminal.rs | Terminal colorization |
| `num_cpus` | 1.x | (retained dep; rayon removed by #84) | Potentially unused post-rayon removal |
| `thiserror` | 1.x | findings.rs or error types | Custom error types |
| `chrono` | 0.4.x | main.rs or findings.rs | Timestamp types (O-01: field exists but universally None) |
| `httparse` | 1.x | analyzer/http.rs | HTTP/1.x request + response parsing |

> Exact versions are in Cargo.toml / Cargo.lock. This table lists the functional roles.
> `num_cpus` should be audited post-rayon removal (Smell #8 is closed but num_cpus may
> be a stale transitive dep that was promoted to direct).

## Dev / Test Dependencies

| Crate | Purpose |
|-------|---------|
| `assert_cmd` | CLI integration tests |
| `predicates` | CLI test assertions |
| Inline test modules | All 282 tests in `tests/` + inline in `reporter/terminal.rs` |
