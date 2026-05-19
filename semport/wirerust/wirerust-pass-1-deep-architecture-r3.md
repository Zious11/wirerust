# Pass 1 Deepening (Round 3, convergence-anchor): Architecture -- wirerust

Triangulation-only follow-up to R2. Three R2-stipulated tasks executed.

## Triangulation 1 -- main.rs cleanup-gap audit beyond finalize()

Three `?` early-return points fire AFTER reassembler construction (line 84) and BEFORE finalize (line 142):

- **Line 102**: `resolve_targets(target)?` -- target-not-found or unreadable directory
- **Line 105**: `PcapSource::from_file(path)?` -- malformed/truncated/permission-denied pcap
- **Line 110**: `ProgressStyle::with_template(...)?` -- theoretical only (compile-time-known literal)

**Severity:** Lines 102 and 105 are realistic on multi-target invocations (e.g., `wirerust analyze a.pcap b.pcap c.pcap`). If pcaps 1 and 2 succeed and pcap 3 errors at open, the prior buffered findings (BC-RAS-054 finalize-side segment-limit summary and on-close events for open flows) are silently lost, AND the terminal/JSON render at line 186 never runs.

**Significance:** This broadens Smell #9 from "panic-unwind loss" to "panic-unwind OR ordinary `Result::Err` propagation". The `?`-Err path is more common than panic-unwind. The R2-recommended fix (an `impl Drop` guard that calls finalize) closes both paths in one change, since `Drop` runs in both cases. No new smell category, no recommendation change.

## Triangulation 2 -- loose-TLS-gate (Smell #10) test exercise check

Searched all 18 test files for `0x16`. Six hits, all benign:

- `dispatcher_tests.rs:23, 49` -- real TLS-shaped records (`[0x16, 0x03, 0x03, 0x00, 0x05, ...]`); intended TLS route, not misroute
- `dispatcher_tests.rs:64` -- 2-byte `[0x16, 0x03]`; too short for the 5-byte content gate, falls to port fallback
- `tls_analyzer_tests.rs:126, 167, 209, 896` -- all bypass the dispatcher, calling `TlsAnalyzer.on_data` directly (test the analyzer's internal parse-error counter, not the dispatcher's gate)

**Zero tests exercise the loose-gate misroute path.** Smell #10 is theoretical. Tightening the gate to check `data[2] <= 0x04` and record length sanity would leave every existing test green (all use `data[2] == 0x03` and sane lengths). R2 recommendation stands as-is.

## Triangulation 3 -- per-direction alert granularity in tests

Searched all 18 test files for `_alert_fired|alert_count|overlap_count|small_segment_count|out_of_window_count`. Eight assertion sites, all single-direction:

- `reassembly_segment_tests.rs:89, 110, 149, 235, 240, 247, 262` -- unit-level on one `FlowDirection`; not even bidirectional
- `reassembly_engine_tests.rs:802` (`test_overlap_anomaly_finding`) -- 51 dups on C2S only; uses `.find(...).expect(...)` so doesn't constrain count
- `reassembly_engine_tests.rs:1083` (`test_out_of_window_threshold_alert`) -- 101 OOW on C2S only; `is_some()` check
- `reassembly_engine_tests.rs:1134` (`test_out_of_window_alert_fires_only_once`) -- 200 OOW on C2S only; asserts `count == 1` (single-direction de-dup, not bidirectional pinning)

**Zero tests pin per-direction alert granularity.** BC-RAS-022 (up to 6 findings per flow, gated per-`FlowDirection`) is **inferred from code only**. No test would fail if the implementation were changed to per-`TcpFlow` gating. Q-A6's recommendation (add direction tag to Finding + bidirectional threshold test) becomes Pass 3 / Pass 5 follow-up.

## Delta Summary

- New items: zero new smells, zero new contracts
- Refined: Smell #9 broadened to cover `?`-Err propagation in addition to panic-unwind; Smell #10 confirmed theoretical; Q-A6 confirmed under-tested
- Remaining gaps: Q-A1/A3/A4 (workspace concerns) -- out of scope; none inside P1 frame

## Novelty Assessment

Novelty: **NITPICK**

Every finding is confirmation or evidence-weight for an R2 item. Removing R3 would not change the spec: the `Drop` guard, TLS-gate tightening, and direction-on-finding recommendations all stand as drafted in R2. The binary test ("would removing this round's findings change how you'd spec the system?") returns no -> NITPICK.

## Convergence Declaration

**Pass 1 has converged -- findings are nitpicks, not gaps.**

P1 R2's prediction ("R3 is likely to be NITPICK; remaining open questions are workspace concerns") is borne out exactly. Pass 1 is complete after R3. Downstream Pass 8 deep-synthesis should treat the union of P1 R1 + R2 + R3 as architectural ground truth, with R2 carrying the bulk of novel content and R3 adding evidence-weight for Smells #9 and #10.

## State Checkpoint

```yaml
pass: 1
round: 3
status: complete
triangulations_completed: 3
new_smells_added: 0
existing_smells_strengthened: 2
q_a_items_remaining: 3
timestamp: 2026-05-19T00:00:00Z
novelty: NITPICK
convergence: PASS_1_CONVERGED
next_round: null
```

