---
artifact: architecture-section
section: purity-boundary-map
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
---

# Purity Boundary Map

## Definition

**Pure Core:** Deterministic, side-effect-free functions. Takes data in, returns a result.
No file I/O, no network, no global mutable state, no `eprintln!`, no `HashMap` seeding from
`OsRng`. Formal verification operates here via Kani / proptest.

**Effectful Shell:** Contains I/O, stderr output, or global-state mutation.
Tested (unit + integration) but not formally proven.

**Mixed:** Module contains both pure computational logic and localized side effects.
The pure portions are extracted for verification; the effectful portions are integration-tested.


## Per-Module Classification

| Module | Classification | Rationale |
|--------|---------------|-----------|
| src/main.rs | Effectful shell | File I/O, stdout/stderr, `indicatif` progress, orchestrates all |
| src/lib.rs | Effectful shell | Re-exports; drives main.rs logic for integration tests |
| src/cli.rs | Pure core | clap struct definitions; no I/O at definition time |
| src/reader.rs | Effectful shell | `BufReader`, file open, pcap-file crate parse; returns `Vec<RawPacket>` |
| src/decoder.rs | **Pure core** | Takes `&[u8]` (link type + raw bytes), returns `Result<ParsedPacket>`. Zero I/O. Deterministic for identical input. Formally verifiable. |
| src/reassembly/config.rs | Pure core | Data struct; no I/O |
| src/reassembly/stats.rs | Pure core | Data struct; no I/O |
| src/reassembly/flow.rs | **Pure core** | FlowKey ordering (INV-1), TcpFlow state machine, direction logic. Deterministic. Formally verifiable. |
| src/reassembly/segment.rs | **Pure core** | FlowDirection buffer: BTreeMap insert/flush. First-wins overlap policy (INV-3). Deterministic. Formally verifiable. |
| src/reassembly/handler.rs | Pure core | Trait definitions only; no implementation |
| src/reassembly/lifecycle.rs | Mixed | close_flow logic is pure table mutation; `CLOSE_FLOW_MISSING_WARNED` AtomicBool is a global side effect |
| src/reassembly/mod.rs | Mixed | `process_packet` hot path is pure segment math; `FINALIZE_SKIPPED_WARNED` Drop tripwire is a global side effect; `eprintln!` calls in Drop |
| src/dispatcher.rs | **Pure core** | `classify()` is deterministic on input bytes + port. `StreamDispatcher` state is per-instance HashMap. No global mutation. Formally verifiable. |
| src/analyzer/mod.rs | Pure core | Trait definition only |
| src/analyzer/dns.rs | **Pure core** | Packet-level; returns `Vec::new()` from `analyze()`; increments per-instance counters only. Formally verifiable. |
| src/analyzer/http.rs | **Pure core** | Stream-level; all state per-instance; no global side effects; `httparse` is deterministic. Formally verifiable. |
| src/analyzer/tls.rs | **Pure core** | Stream-level; all state per-instance; `md5` is deterministic; `extract_sni` is a pure function (INV-5). Formally verifiable. |
| src/findings.rs | Pure core | Data struct + Display impls; no I/O |
| src/mitre.rs | **Pure core** | Static match table; pure lookup functions (INV-9). Formally verifiable. |
| src/summary.rs | Pure core | Per-instance accumulator; no I/O |
| src/reporter/mod.rs | Pure core | Trait definition only |
| src/reporter/json.rs | **Pure core** | `render()` returns an owned `String` via `serde_json::json!` + `to_string_pretty`; zero I/O. Stdout/file write is done exclusively by the caller in `main.rs`. Formally verifiable. |
| src/reporter/terminal.rs | **Pure core** | `render()` pushes into an in-memory `String` and returns it; zero I/O. `escape_for_terminal` is a pure helper. Stdout write is done exclusively by the caller in `main.rs`. Formally verifiable. |
| src/reporter/csv.rs | **Pure core** | `render()` writes to a `Vec<u8>` via `csv::WriterBuilder::from_writer(Vec::new())`, converts to `String`, and returns it; zero I/O. Stdout/file write is done exclusively by the caller in `main.rs`. Formally verifiable. |


## Purity Boundary Diagram

```
                     PURE CORE                       EFFECTFUL SHELL
                   +-------------------------+      +--------------------+
L0 Entry           | cli.rs (structs only)   |      | main.rs, lib.rs    |
                   +-------------------------+      +--------------------+
L1 Ingest          | decoder.rs (C-5)        |      | reader.rs (C-4)    |
                   +-------------------------+      +--------------------+
L2 Stream          | flow.rs (C-7)           |      |                    |
                   | segment.rs (C-8)        |      |     mod.rs (C-6)   |
                   | handler.rs (C-9)        | ---- |  [mixed: hot path  |
                   | config.rs               |      |   pure; Drop/warn  |
                   | stats.rs                |      |   effectful]       |
                   | dispatcher.rs (C-21)    |      | lifecycle.rs (C-15)|
                   +-------------------------+      | [mixed: close_flow |
                                                    |  pure; WARNED atom.|
                                                    |  effectful]        |
                                                    +--------------------+
L3 Domain          | analyzer/dns.rs  (C-11) |      |                    |
                   | analyzer/http.rs (C-12) |      |                    |
                   | analyzer/tls.rs  (C-13) |      |                    |
                   | findings.rs      (C-14) |      |                    |
                   | mitre.rs         (C-16) |      |                    |
                   | summary.rs       (C-17) |      |                    |
                   +-------------------------+      +--------------------+
L4 Output          | reporter/mod.rs  (C-18) |      |                    |
                   | reporter/json.rs (C-19) |      |                    |
                   | reporter/terminal.rs    |      |                    |
                   |   (C-20; escape_for_   |      |                    |
                   |    terminal is pure)    |      |                    |
                   | reporter/csv.rs  (C-21) |      |                    |
                   | [all render()->String;  |      |                    |
                   |  caller in main.rs does |      |                    |
                   |  the stdout/file write] |      |                    |
                   +-------------------------+      +--------------------+
```


## Implications for Verification

The pure core modules are the primary targets for formal verification via Kani and
proptest. Key formally-verifiable properties:

- `decoder.rs`: No panic on malformed input; link-type gate exhaustive
- `reassembly/flow.rs`: FlowKey canonical ordering (INV-1); state machine reachability
- `reassembly/segment.rs`: First-wins overlap (INV-3); buffered_bytes monotonicity (INV-6 partial)
- `dispatcher.rs`: Content-first precedence (INV-2); DispatchTarget::None NOT cached before retry cap (pre-cap: attempts incremented, routes untouched); permanently cached as DispatchTarget::None once cap reached (dispatcher.rs:146-148)
- `analyzer/tls.rs`: SNI 4-way ordered match (INV-5); JA3 GREASE filter correctness
- `analyzer/http.rs`: HTTP poison monotonicity (INV-8); cross-flow state isolation
- `mitre.rs`: technique_id format invariant (INV-9); all_tactics_in_report_order completeness

See `verification-architecture.md` for the full proof strategy per invariant.
