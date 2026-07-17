# Phase F6 — Kani Formal Verification Results (feature-iec104 delta)

**Feature:** IEC-104 passive analyzer (STORY-167..174)
**develop HEAD:** `b36b884`
**Date:** 2026-07-17
**kani version:** cargo-kani 0.67.0 (`~/.cargo/bin/cargo-kani`, CBMC backend present)
**Kani actually ran:** YES — all harnesses executed to completion on current code.

Re-run rationale: FIX-P4-001 and FIX-F5-001 modified iec104.rs emit sites
(source_ip/timestamp/direction population threaded through
`process_u_frame`/`detect_iec104_threats`/`track_ns_desync`, and `on_data`).
`parse_apci_header` itself is unaffected by the enrichment, but VP-044 was
re-run to confirm the delta did not regress no-panic safety. VP-004
(dispatcher) and VP-007 (mitre catalog) source is unchanged by the fixes and
was re-run to confirm.

---

## Summary

| VP | Harness | File | Result | Checks |
|----|---------|------|--------|--------|
| VP-044 | `verify_parse_apci_header_safety` | src/analyzer/iec104.rs | **SUCCESSFUL** | 0 of 89 failed |
| VP-004 | `verify_content_first_precedence_exhaustive` | src/dispatcher.rs | **SUCCESSFUL** | 0 of 440 failed (10 unreachable) |
| VP-004 | `verify_tls_signature_beats_port` | src/dispatcher.rs | **SUCCESSFUL** | 0 of 407 failed (12 unreachable) |
| VP-004 | `verify_none_two_phase_caching` | src/dispatcher.rs | **SUCCESSFUL** | 0 of 183 failed (1 unreachable) |
| VP-007 | `verify_all_emitted_ids_resolve` | src/mitre.rs | **SUCCESSFUL** | 0 of 122 failed (1 unreachable) |

All 5 harnesses: **VERIFICATION SUCCESSFUL.**

## VP-044 — parse_apci_header no-panic + facets

`cargo kani --harness verify_parse_apci_header_safety`

- 89 checks, 0 failed. Verification time 0.71s.
- Confirms all 5 facets (array-bounds safety on the APCI header parse path) hold
  on current code b36b884; parse_apci_header unaffected by emit-site enrichment.

## VP-004 — dispatcher oracle (incl. Rule 8 Iec104)

`cargo kani --harness <name>` for each of the three dispatcher harnesses.

- `verify_content_first_precedence_exhaustive`: content-first precedence holds
  exhaustively across dispatch rules including Rule 8 (Iec104). 0 of 440 failed.
- `verify_tls_signature_beats_port`: 0 of 407 failed.
- `verify_none_two_phase_caching`: 0 of 183 failed.
- Dispatcher source unchanged by FIX-P4-001 / FIX-F5-001; re-run confirms no regression.

## VP-007 — T0881 catalog integrity

`cargo kani --harness verify_all_emitted_ids_resolve` — 0 of 122 failed. Every
emitted technique ID resolves to a name and a tactic.

Companion test `cargo test vp007_catalog_drift_guard`:
`mitre::vp007_format_tests::vp007_catalog_drift_guard ... ok` (1 passed).
`SEEDED_TECHNIQUE_ID_COUNT = 29` confirmed in src/mitre.rs (matches expected SEEDED=29),
locked in lockstep with `SEEDED_TECHNIQUE_IDS.len()` by the drift guard.

## Companion proptests (VP-045 / VP-046)

Run alongside the Kani re-confirm (`cargo test --test iec104_analyzer_tests proptest_vp04`):

- VP-045 `proptest_vp045_direction_isolation` ... ok
- VP-045 `proptest_vp045_independent_run_equivalence` ... ok
- VP-046 `proptest_vp046_frame_format_totality` ... ok

3 passed; 0 failed. Confirms carry directional isolation + run determinism and
`classify_frame_format` totality over all 256 CF1 values on current code.

## Verdict

Kani gate: **PASS** — all 5 harnesses SUCCESSFUL, VP-007 SEEDED=29 confirmed,
companion VP-045/046 proptests green.
