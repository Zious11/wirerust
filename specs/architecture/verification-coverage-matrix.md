---
artifact: architecture-section
section: verification-coverage-matrix
traces_to: ARCH-INDEX.md
version: "1.3"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-02
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified (propagated from VP-INDEX, all 20 VPs locked). Counts unchanged at 20."
  - date: 2026-06-08
    actor: state-manager
    reason: "Feature Mode F2 (issue #100): VP-021 added (draft/unverified; integration+proptest; source BCs BC-2.09.007 + BC-2.04.055). Total 20→21. proptest 6→7."
  - date: 2026-06-09
    actor: spec-steward
    reason: "F6 lock propagation (FINDING-001): VP-021 Phase F4→test-sufficient, Status draft→verified; coverage note updated to reflect lock @256a490. Internal counts verified consistent."
  - date: 2026-06-09
    actor: spec-steward
    reason: "F7 consistency fix — VP-021 reclassified to proptest column to match VP-INDEX counting convention (proptest 7 / integration-unit 5); prior v1.2 placement contradicted VP-INDEX invariant (verification-coverage-matrix.md Totals must equal Kani 8 / proptest 7 / fuzz 1 / integration-unit 5 = 21)."
---

# Verification Coverage Matrix

## VP-to-Module Mapping

| VP-ID | Property (short) | Module | Tool | Phase | Status |
|-------|-----------------|--------|------|-------|--------|
| VP-001 | FlowKey canonical ordering | reassembly/flow.rs | Kani | P0 | verified |
| VP-002 | First-wins overlap policy | reassembly/segment.rs | Kani | P0 | verified |
| VP-003 | MAX_FINDINGS cap + finalize bypass | reassembly/mod.rs | Kani | P0 | verified |
| VP-004 | Content-first dispatch precedence | dispatcher.rs | Kani | P0 | verified |
| VP-005 | SNI 4-way ordered match (INV-5 boundary) | analyzer/tls.rs | Kani | P0 | verified |
| VP-006 | HTTP poison monotonicity | analyzer/http.rs | proptest | P1 | verified |
| VP-007 | MITRE technique ID format completeness | mitre.rs | Kani | P0 | verified |
| VP-008 | decode_packet no-panic | decoder.rs | cargo-fuzz | P0 | verified |
| VP-009 | FlowState machine validity | reassembly/flow.rs | Kani | P0 | verified |
| VP-010 | buffered_bytes invariant | reassembly/segment.rs | proptest | P1 | verified |
| VP-011 | flush_contiguous monotonicity | reassembly/segment.rs | proptest | P1 | verified |
| VP-012 | escape_for_terminal correctness | reporter/terminal.rs | proptest | P1 | verified |
| VP-013 | JA3 GREASE filter | analyzer/tls.rs | proptest | P1 | verified |
| VP-014 | HttpAnalyzer cross-flow isolation | analyzer/http.rs | proptest | P1 | verified |
| VP-015 | TCP sequence wraparound | reassembly/segment.rs | Kani | P1 | verified |
| VP-016 | MITRE tactic grouping order | reporter/terminal.rs | integration | test-sufficient | verified |
| VP-017 | JsonReporter key determinism | reporter/json.rs | integration | test-sufficient | verified |
| VP-018 | CLI mutual exclusion (reassemble flags) | cli.rs | integration | test-sufficient | verified |
| VP-019 | DNS statistics-only (no findings) | analyzer/dns.rs | unit | test-sufficient | verified |
| VP-020 | CSV injection neutralization | reporter/csv.rs | unit | test-sufficient | verified |
| VP-021 | Timestamp provenance threading | reassembly/mod.rs | integration+proptest | test-sufficient | verified |


## Per-Module Coverage Totals

| Module | Kani | proptest | cargo-fuzz | integration/unit | Total VPs |
|--------|------|----------|------|-----------------|-----------|
| reassembly/flow.rs | 2 (VP-001, VP-009) | 0 | 0 | 0 | 2 |
| reassembly/segment.rs | 2 (VP-002, VP-015) | 2 (VP-010, VP-011) | 0 | 0 | 4 |
| reassembly/mod.rs | 1 (VP-003) | 1 (VP-021) | 0 | 0 | 2 |
| dispatcher.rs | 1 (VP-004) | 0 | 0 | 0 | 1 |
| analyzer/tls.rs | 1 (VP-005) | 1 (VP-013) | 0 | 0 | 2 |
| analyzer/http.rs | 0 | 2 (VP-006, VP-014) | 0 | 0 | 2 |
| mitre.rs | 1 (VP-007) | 0 | 0 | 0 | 1 |
| decoder.rs | 0 | 0 | 1 (VP-008) | 0 | 1 |
| reporter/terminal.rs | 0 | 1 (VP-012) | 0 | 1 (VP-016) | 2 |
| reporter/json.rs | 0 | 0 | 0 | 1 (VP-017) | 1 |
| cli.rs | 0 | 0 | 0 | 1 (VP-018) | 1 |
| analyzer/dns.rs | 0 | 0 | 0 | 1 (VP-019) | 1 |
| reporter/csv.rs | 0 | 0 | 0 | 1 (VP-020) | 1 |
| **Totals** | **8** | **7** | **1** | **5** | **21** |


## Coverage Notes

- reassembly/segment.rs has 2 Kani proofs (VP-002 first-wins overlap, VP-015 sequence
  wraparound) and 2 proptest proofs (VP-010 buffered_bytes invariant, VP-011
  flush_contiguous monotonicity). Row total = 4 VPs.

- VP-001 through VP-020 statuses are `verified` as of Phase-6 gate close (2026-06-02 @ develop 0855f25).
  verification_lock=true is set on all those VP documents.
- VP-021 is `verified` — locked at F6 formal hardening gate (2026-06-09 @ develop 256a490). verification_lock=true.
  Proof evidence: tests/timestamp_threading_tests.rs (integration + proptest). 1147 tests green.

- `module-criticality.md` defines kill-rate targets that constrain the minimum proof
  depth for each module. CRITICAL modules (reassembly/segment.rs, reassembly/flow.rs,
  analyzer/tls.rs) require Kani proofs, not just proptest.
