# Holdout RE-EVALUATION — HS-043 Fix + Reassembly Regression Spot-Check

- Cycle: v0.1.0-greenfield-spec
- Target: merged `develop` @ HEAD `c3cd4bd` (fix: wire idle-flow expiry into process_packet + add `--flow-timeout`, PR #171)
- Binary: `target/debug/wirerust` (built clean, debug-assertions ON)
- Method: black-box, observed behavior only (CLI + JSON output); no source/spec/review access
- Evaluator profile: restricted (Bash + Read)

## Verdict

| HS-id | must_pass | satisfaction | PASS/FAIL | observed note |
|-------|-----------|-------------:|-----------|---------------|
| HS-043 | true | 1.00 | PASS | Idle expiry now observable: `--flow-timeout 3/5` on flow-expiry.pcap (6s gap) -> `flows_expired=1`, exit 0, no crash. Strict `>` confirmed: ft=6 (gap==timeout) -> 0; ft=7 -> 0. Negative control ft=300 (default) -> 0. `--flow-timeout 0` rejected by clap ("0 is not in 1..18446744073709551615"). bytes_reassembled unaffected by expiry. |
| HS-021 | true | 1.00 | PASS (no regression) | RST path (dns-remoteshell): rst=9, completed=9, findings=0. FIN path (tcp-ecn-sample): fin=1, completed=1, findings=0. Idempotent: two runs byte-identical JSON. Close counts preserved even under ft=1 (RST/FIN flows NOT stolen by sweep). |
| HS-026 | true | 1.00 | PASS (no regression) | OOO (http-ooo.pcap): bytes_reassembled=287 (full payload), exit 0, deterministic across runs (byte-identical). Findings (5 missing-Host anomalies) are content-driven, consistent with correctly ordered data, not scramble artifacts. Single-flow/tight spacing -> expired=0 (sweep does not disturb OOO path). |
| HS-028 | true | 1.00 | PASS (no regression) | RST vs FIN byte semantics intact: distinct bytes_reassembled totals preserved (RST corpus 4047, FIN corpus 83559). Under aggressive ft=1 these byte totals AND findings counts are unchanged -> close-payload semantics unaffected by per-packet expiry sweep. |
| HS-044 | true | 1.00 | PASS (no regression) | Debug build (assertions on): all close types + aggressive expiry combined (6 fixtures, ft=1/2/300) -> rc=0, no debug-assert/memory_used panic. bytes_reassembled invariant across default vs ft=1 in every fixture -> no memory-accounting drift introduced. |

## Gate

- **HS-043: PASS** (satisfaction 1.00 >= 0.60 critical floor). Previously 0.50 FAIL — now fully satisfied.
- Regression spot-check: 4/4 scenarios remain PASS at 1.00. **None dropped** from prior PASS.
- Mean satisfaction (these 5): 1.00. Critical minimum: 1.00. **Gate: PASS.**

## Evidence Detail

### HS-043 (primary) — idle-flow expiry now wired

Fixture `tests/fixtures/flow-expiry.pcap`: 2 packets, ts=0.0s and ts=6.0s (one flow goes idle, second packet from another flow triggers the sweep).

| --flow-timeout | gap vs timeout | flows_expired | expected | result |
|---------------:|----------------|--------------:|----------|--------|
| 3 | 6 > 3 | 1 | expiry | OK |
| 5 | 6 > 5 | 1 | expiry | OK |
| 6 | 6 == 6 (strict >) | 0 | NO expiry | OK (boundary) |
| 7 | 6 < 7 | 0 | NO expiry | OK |
| 300 (default) | 6 < 300 | 0 | NO expiry | OK (negative control) |
| 0 | n/a | n/a | rejected | OK (clap range 1..) |

All runs exit 0; bytes_reassembled=0 (handshake-only payloads) unaffected by expiry; no crash. Scenario rubric satisfied across functional correctness (idle expired, active not), edge case (strict-`>` boundary), error quality (no crash, 0 rejected), data integrity (flows_expired accurate, bytes unaffected).

### Regression invariants under aggressive expiry (default vs --flow-timeout 1)

| fixture | bytes default | bytes ft=1 | findings default | findings ft=1 | RST/FIN close preserved |
|---------|--------------:|-----------:|-----------------:|--------------:|-------------------------|
| dns-remoteshell | 4047 | 4047 | 0 | 0 | rst=9/completed=9 both |
| tcp-ecn-sample | 83559 | 83559 | 0 | 0 | fin=1/completed=1 both |
| segmented | 32000 | 32000 | 0 | 0 | n/a (single partial flow) |
| tls | 20245 | 20245 | 4 | 4 | n/a |
| http-ooo | 287 | 287 | 5 | 5 | n/a (expired=0) |

bytes_reassembled and findings are invariant under aggressive expiry in every fixture; RST/FIN close paths keep their exact counts. flows_total rises under ft=1 only because idle flows expire then re-appear when a later same-key packet arrives — expected, and it does not perturb byte/finding/close accounting.

### Test suite (pass/fail only, not source)

`cargo test --all-targets`: all targets green, 0 failed, 0 panics. Reassembly close tests `test_on_flow_close_absent_key_no_panic`, `test_on_flow_close_drops_state_preserves_aggregates`, `test_buffer_overflow_silent_no_counters` pass.

## Findings

No behavioral gaps. The HS-043 fix is correct and exhaustively observable:
- `--flow-timeout` is wired end-to-end (declared, validated, behaviorally effective).
- Strict-`>` boundary semantics implemented (gap == timeout does not expire).
- Underflow/no-crash guards hold; `--flow-timeout 0` rejected at parse time.
- The per-packet expiry sweep is non-regressive on the reassembly hot path: OOO ordering, RST/FIN/timeout close semantics, byte accounting, finding counts, idempotency, and debug-mode memory-consistency assertions all hold unchanged.
