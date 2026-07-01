---
id: STORY-148
title: "Fix Analyzer Flow-State Lifecycle: EnipAnalyzer on_flow_close Wiring + DNP3 Flow-Map Cap (SEC-005 / SEC-006)"
epic: E-20
wave: "~"
points: 5
status: draft
depends_on: []
input-hash: TBD
inputs: []
---

# STORY-148 — Fix Analyzer Flow-State Lifecycle: EnipAnalyzer on_flow_close Wiring + DNP3 Flow-Map Cap (SEC-005 / SEC-006)

**Epic:** E-20 (EtherNet/IP ENIP/CIP Analyzer)
**Status:** draft
**Wave:** TBD
**Points:** 5

## Background

Maintenance run maint-2026-07-01 identified two related memory-safety defects in the
analyzer flow-state lifecycle:

**SEC-005 (MEDIUM, CWE-400, real bug):** `StreamDispatcher::on_flow_close` in
`src/dispatcher.rs` (~lines 409–414) contains a no-op arm for the ENIP analyzer — it
does not forward the close event to `EnipAnalyzer::on_flow_close`. As a result,
`EnipAnalyzer.flows` (enip.rs ~line 782, `.entry().or_default()`) grows monotonically:
every distinct port-44818 flow inserts an entry that is never removed. A crafted pcap
with a large number of short-lived ENIP flows exhausts heap memory, constituting a
file-based DoS.

Additionally, all the flow-close aggregation logic in `EnipAnalyzer::on_flow_close`
(enip.rs ~line 693) — including final-byte accounting and per-flow statistics folding —
has been dead code since STORY-138 delivered the ENIP session lifecycle. Root cause:
the dispatcher dispatch table was wired for ENIP data delivery (`on_data`) but the
close arm was left as a no-op placeholder.

**SEC-006 (MEDIUM, CWE-400, design decision required):** `Dnp3Analyzer.flows`
(dnp3.rs ~line 303) accumulates all historical flows. The `summarize()` path consumes
them — which appears by design — but the same file-based DoS profile exists for
long-running captures with many distinct DNP3 flows. Unlike SEC-005, SEC-006 requires
an explicit design decision before implementation: either add a hard cap on `flows.len()`
with LRU eviction (analogous to `TcpReassembler.max_flows`) or wire through a DNP3
`on_flow_close` callback. This story captures the design decision and its implementation
as a scoped AC.

## Goal

1. Wire `StreamDispatcher::on_flow_close` to call `EnipAnalyzer::on_flow_close(flow_key)`
   in the ENIP arm — making the long-present aggregation logic reachable and preventing
   unbounded `flows` map growth.
2. Add a regression test that creates multiple ENIP flows, closes each, and asserts
   that the flow map entry is removed and per-flow aggregates are folded into totals.
3. Document and implement the chosen mitigation for DNP3 flow-map growth (hard cap with
   LRU eviction, or `on_flow_close` routing), with a corresponding test.

## Acceptance Criteria

AC-148-001: `StreamDispatcher::on_flow_close` in `src/dispatcher.rs` calls
  `enip.on_flow_close(flow_key)` (or equivalent entry removal + aggregate fold) in the
  ENIP arm — the no-op arm is eliminated. The `EnipAnalyzer::on_flow_close` method at
  enip.rs ~line 693 is no longer dead code.

AC-148-002: A regression test (in `tests/` or `src/analyzer/enip.rs` test module)
  creates N distinct ENIP flows (N >= 2), calls the dispatcher close path for each,
  and asserts that `EnipAnalyzer.flows.len()` is 0 after all closures and that per-flow
  aggregates (bytes_total, packet_count) appear in the analyzer summary.

AC-148-003: An explicit design note is committed (in `docs/adr/` as an addendum to an
  existing ADR, or as a doc-comment policy block in the DNP3 analyzer source) documenting
  the chosen approach for DNP3 flow-map growth: Option A — `max_flows` hard cap with LRU
  eviction (default capped at a value matching `TcpReassembler.max_flows` convention); or
  Option B — DNP3 `on_flow_close` routing mirroring the SEC-005 fix. The note records the
  rationale for the choice.

AC-148-004: The chosen DNP3 mitigation is implemented: either `Dnp3Analyzer.flows` has
  a configurable capacity cap with LRU eviction OR `Dnp3Analyzer::on_flow_close` is
  wired through the dispatcher. A corresponding test asserts the memory bound is respected
  under synthetic flow churn (create N flows > the cap, verify `flows.len()` stays bounded).

AC-148-005: `cargo clippy --all-targets -- -D warnings` and `cargo test --all-targets`
  pass without new warnings or regressions introduced by this change.

## Notes

- Strong v0.12.0 candidate: SEC-005 is a file-based DoS reachable via a crafted pcap
  against any wirerust deployment that analyzes ENIP traffic. SEC-006 is lower urgency
  (the `summarize()` consumption provides partial mitigation for in-process live captures)
  but should be resolved in the same release window to close the CWE-400 class.
- Source findings: SEC-005 (MEDIUM, CWE-400) + SEC-006 (MEDIUM, CWE-400),
  maintenance run maint-2026-07-01.
- Primary modules: `src/dispatcher.rs` (SEC-005 wiring fix), `src/analyzer/enip.rs`
  (aggregation activation), `src/analyzer/dnp3.rs` (SEC-006 mitigation).
- STORY-138 is the original ENIP session-lifecycle story; STORY-148 closes the gap
  left when the dispatcher on_flow_close arm was not wired during STORY-138 delivery.
- Precedent for E-20 maintenance fix pattern: STORY-139 (EC-X1/EC-X2 carry-direction
  fixes, wave 62) — same epic, same pattern of correctness fix added after initial
  delivery wave.
- Wave assignment is TBD — schedule at v0.12.0 planning alongside STORY-091, STORY-121,
  STORY-143, STORY-147, STORY-149, and STORY-150 (all unscheduled).
