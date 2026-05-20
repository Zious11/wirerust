---
artifact: architecture-section
section: verification-coverage-matrix
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
---

# Verification Coverage Matrix

## VP-to-Module Mapping

| VP-ID | Property (short) | Module | Tool | Phase | Status |
|-------|-----------------|--------|------|-------|--------|
| VP-001 | FlowKey canonical ordering | reassembly/flow.rs | Kani | P0 | draft |
| VP-002 | First-wins overlap policy | reassembly/segment.rs | Kani | P0 | draft |
| VP-003 | MAX_FINDINGS cap + finalize bypass | reassembly/mod.rs | Kani | P0 | draft |
| VP-004 | Content-first dispatch precedence | dispatcher.rs | Kani | P0 | draft |
| VP-005 | SNI 4-way ordered match (INV-5 boundary) | analyzer/tls.rs | Kani | P0 | draft |
| VP-006 | HTTP poison monotonicity | analyzer/http.rs | proptest | P1 | draft |
| VP-007 | MITRE technique ID format completeness | mitre.rs | Kani | P0 | draft |
| VP-008 | decode_packet no-panic | decoder.rs | fuzz | P0 | draft |
| VP-009 | FlowState machine validity | reassembly/flow.rs | Kani | P0 | draft |
| VP-010 | buffered_bytes invariant | reassembly/segment.rs | proptest | P1 | draft |
| VP-011 | flush_contiguous monotonicity | reassembly/segment.rs | proptest | P1 | draft |
| VP-012 | escape_for_terminal correctness | reporter/terminal.rs | proptest | P1 | draft |
| VP-013 | JA3 GREASE filter | analyzer/tls.rs | proptest | P1 | draft |
| VP-014 | HttpAnalyzer cross-flow isolation | analyzer/http.rs | proptest | P1 | draft |
| VP-015 | TCP sequence wraparound | reassembly/segment.rs | Kani | P1 | draft |
| VP-016 | MITRE tactic grouping order | reporter/terminal.rs | integration | P1 | draft |
| VP-017 | JsonReporter key determinism | reporter/json.rs | integration | P1 | draft |
| VP-018 | CLI mutual exclusion (reassemble flags) | cli.rs | integration | P1 | draft |
| VP-019 | DNS statistics-only (no findings) | analyzer/dns.rs | unit | P1 | draft |
| VP-020 | CSV injection neutralization | reporter/csv.rs | unit | P1 | draft |


## Per-Module Coverage Totals

| Module | Kani | proptest | fuzz | integration/unit | Total VPs |
|--------|------|----------|------|-----------------|-----------|
| reassembly/flow.rs | 2 (VP-001, VP-009) | 0 | 0 | 0 | 2 |
| reassembly/segment.rs | 2 (VP-002, VP-015) | 2 (VP-010, VP-011) | 0 | 0 | 4 |
| reassembly/mod.rs | 1 (VP-003) | 0 | 0 | 0 | 1 |
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
| **Totals** | **8** | **6** | **1** | **5** | **20** |


## Coverage Notes

- reassembly/segment.rs has 2 Kani proofs (VP-002 first-wins overlap, VP-015 sequence
  wraparound) and 2 proptest proofs (VP-010 buffered_bytes invariant, VP-011
  flush_contiguous monotonicity). Row total = 4 VPs.

- All VP statuses are `draft`. They move to `harness-written` when the Kani/proptest
  harness skeleton exists, to `passing` when the harness runs green.

- `module-criticality.md` defines kill-rate targets that constrain the minimum proof
  depth for each module. CRITICAL modules (reassembly/segment.rs, reassembly/flow.rs,
  analyzer/tls.rs) require Kani proofs, not just proptest.
