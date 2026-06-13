---
artifact: architecture-section
section: tooling-selection
traces_to: ARCH-INDEX.md
version: "1.3"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-08
    actor: architect
    reason: "Body: added SS-04 cargo-mutants Phase 6 outcomes (D-015, PR #184) and kill-rate results."
  - date: 2026-06-08
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-12 corpus debt cleanup (F-2): Kani 'Properties targeted' list expanded from 8 to 11 VPs — added VP-022 (Modbus MBAP parse safety; analyzer/modbus.rs; shipped v0.6.0), VP-023 (DNP3 data-link frame parse safety; analyzer/dnp3.rs; shipped v0.6.0), VP-024 (ARP frame parse safety and binding-table invariant; analyzer/arp.rs; planned/draft). Kani scope bullets updated to match. Total now reflects 11 Kani VPs per VP-INDEX."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-16 A-01: proptest 'Properties targeted' list expanded from 6 to 7 VPs — added VP-021 (timestamp provenance threading; integration+proptest; counted under proptest per VP-INDEX convention). proptest count now = 7, matching VP-INDEX proptest_count=7."
---

# Tooling Selection

## Language Context

wirerust is single-crate Rust 2024 edition, MSRV 1.91, stable toolchain only.

## Selected Tools

### Kani (Model Checker)

**Rationale:** Kani performs bounded model checking on safe Rust programs. It is the
primary tool for properties that must hold for ALL inputs within a bounded domain:
state machine transitions, arithmetic invariants, and pointer-safety proofs.

**Properties targeted:** VP-001, VP-002, VP-003, VP-004, VP-005, VP-007, VP-009, VP-015, VP-022, VP-023, VP-024 (draft/planned)

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
- `analyzer/modbus.rs`: MBAP parse safety and function-code boundary classification (VP-022) [shipped v0.6.0]
- `analyzer/dnp3.rs`: data-link frame parse safety and function-code classification (VP-023) [shipped v0.6.0]
- `analyzer/arp.rs` + `decoder.rs`: ARP frame parse safety and binding-table invariant (VP-024) [planned/draft]

### proptest (Property-Based Testing)

**Rationale:** proptest generates random inputs and checks that properties hold.
It is the right tool for probabilistic invariants (escape function, cross-flow
isolation, JA3 GREASE filter) where Kani's bounded state space would be
prohibitively large.

**Properties targeted:** VP-006, VP-010, VP-011, VP-012, VP-013, VP-014, VP-021

**Setup:** `proptest = "1"` in `[dev-dependencies]`. Run via `cargo test`.

**Scope for wirerust:**
- `analyzer/http.rs`: poison monotonicity (VP-006), cross-flow isolation (VP-014)
- `reassembly/segment.rs`: buffered_bytes invariant (VP-010), flush monotonicity (VP-011)
- `reporter/terminal.rs`: escape_for_terminal correctness (VP-012)
- `analyzer/tls.rs`: JA3 GREASE filter (VP-013)
- `reassembly/mod.rs`: timestamp provenance threading (VP-021; integration+proptest; counted under proptest per VP-INDEX convention — proptest component covers all-u32 timestamp range + cross-flow isolation)

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

**Scope for wirerust:**

- SS-04 (Reassembly — `flow.rs`, `segment.rs`, `mod.rs`): added to scope in Phase 6
  (D-015, PR #184). Outcome: `flow.rs` 100% kill rate, `segment.rs` effective kill 99.52%
  (`ranges_overlap` adjacency 9/9 caught, 2 proven-equivalent mutants), `mod.rs` 98.54%
  (3 proven-equivalent mutants remain). 16 genuine survivors killed; CRITICAL anti-evasion
  modules are now mutation-verified.
- SS-06 (HttpAnalyzer)
- SS-07 (TlsAnalyzer)
- SS-08 (DnsAnalyzer)
- SS-09 (findings.rs)
- SS-10 (mitre.rs)

Target kill rates per module-criticality.md.

## Tools Considered and Rejected

| Tool | Reason for rejection |
|------|---------------------|
| TLA+ | wirerust has no distributed protocol or concurrent state. Single-threaded synchronous pipeline. TLA+ overhead is not justified. |
| Miri | Miri finds UB in unsafe code. wirerust has zero unsafe blocks. Miri cannot add value here. |
| Loom | Loom tests concurrent data structures. wirerust is single-threaded. Not applicable. |
| Coq / Lean formal proofs | Overkill for a CLI tool. Kani provides sufficient formal assurance for the targeted properties without the proof-engineering investment. |
