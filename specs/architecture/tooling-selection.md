---
artifact: architecture-section
section: tooling-selection
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
---

# Tooling Selection

## Language Context

wirerust is single-crate Rust 2024 edition, MSRV 1.91, stable toolchain only.

## Selected Tools

### Kani (Model Checker)

**Rationale:** Kani performs bounded model checking on safe Rust programs. It is the
primary tool for properties that must hold for ALL inputs within a bounded domain:
state machine transitions, arithmetic invariants, and pointer-safety proofs.

**Properties targeted:** VP-001, VP-002, VP-003, VP-004, VP-005, VP-007, VP-009, VP-015

**Setup:** Add `[dev-dependencies] kani = { ... }` and `#[cfg(kani)] #[kani::proof]`
harnesses inline in the relevant modules or in `tests/`. Run with `cargo kani`.

**Constraint:** Kani works on stable Rust but requires a separate toolchain install.
It works best on bounded loops and finite state machines -- the FlowState machine
and segment buffer are good fits. Unbounded heap structures (HashMap iteration)
require careful scoping.

**Scope for wirerust:**
- `reassembly/flow.rs`: FlowKey canonicity (VP-001), state machine (VP-009)
- `reassembly/segment.rs`: first-wins overlap (VP-002), sequence wraparound (VP-015)
- `reassembly/mod.rs`: MAX_FINDINGS cap (VP-003)
- `dispatcher.rs`: content-first precedence (VP-004)
- `analyzer/tls.rs`: SNI arm selection (VP-005)
- `mitre.rs`: technique ID format (VP-007)

### proptest (Property-Based Testing)

**Rationale:** proptest generates random inputs and checks that properties hold.
It is the right tool for probabilistic invariants (escape function, cross-flow
isolation, JA3 GREASE filter) where Kani's bounded state space would be
prohibitively large.

**Properties targeted:** VP-006, VP-010, VP-011, VP-012, VP-013, VP-014

**Setup:** `proptest = "1"` in `[dev-dependencies]`. Run via `cargo test`.

**Scope for wirerust:**
- `analyzer/http.rs`: poison monotonicity (VP-006), cross-flow isolation (VP-014)
- `reassembly/segment.rs`: buffered_bytes invariant (VP-010), flush monotonicity (VP-011)
- `reporter/terminal.rs`: escape_for_terminal correctness (VP-012)
- `analyzer/tls.rs`: JA3 GREASE filter (VP-013)

### cargo-fuzz (libFuzzer)

**Rationale:** Fuzzing is the correct tool for the no-panic property at parser entry
points. It exercises adversarial byte sequences that neither Kani (bounded) nor
proptest (type-guided) will generate in practice.

**Properties targeted:** VP-008 (decode_packet no-panic)

**Setup:** `cargo install cargo-fuzz`; create `fuzz/fuzz_targets/decode_packet.rs`.
Run with `cargo fuzz run decode_packet`. Corpus stored in `fuzz/corpus/`.

**Scope for wirerust:** Primary target is `decoder.rs` (C-5). Secondary targets
are `analyzer/tls.rs` and `analyzer/http.rs` entry points (`on_data`).

### cargo-mutants (Mutation Testing)

**Rationale:** Mutation testing verifies that test assertions are actually checking
the right behavior -- not just reaching the code. Particularly valuable for the
domain logic modules (analyzers, mitre, findings) where test coverage may be high
but assertion strength may be low.

**Setup:** `cargo install cargo-mutants`. Run with `cargo mutants`.

**Scope for wirerust:** SS-06 (HttpAnalyzer), SS-07 (TlsAnalyzer), SS-08 (DnsAnalyzer),
SS-10 (mitre.rs), SS-09 (findings.rs). Target kill rates per module-criticality.md.

## Tools Considered and Rejected

| Tool | Reason for rejection |
|------|---------------------|
| TLA+ | wirerust has no distributed protocol or concurrent state. Single-threaded synchronous pipeline. TLA+ overhead is not justified. |
| Miri | Miri finds UB in unsafe code. wirerust has zero unsafe blocks. Miri cannot add value here. |
| Loom | Loom tests concurrent data structures. wirerust is single-threaded. Not applicable. |
| Coq / Lean formal proofs | Overkill for a CLI tool. Kani provides sufficient formal assurance for the targeted properties without the proof-engineering investment. |
