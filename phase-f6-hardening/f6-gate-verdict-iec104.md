# Phase F6 — Targeted Hardening Gate Verdict (feature-iec104 delta)

**Feature:** IEC-104 passive analyzer (STORY-167..174) + fix PRs FIX-P4-001, FIX-F5-001
**develop HEAD:** `b36b884` (branch `develop`)
**Date:** 2026-07-17
**Delta scope:** src/analyzer/iec104.rs (primary), src/dispatcher.rs (Rule 8 Iec104),
src/mitre.rs (T0881), src/protocols.rs, src/cli.rs, src/main.rs;
tests/iec104_analyzer_tests.rs; fuzz/fuzz_targets/fuzz_iec104_parser.rs.

Verification performed independently from the spec (VP-044/045/046/047, VP-004, VP-007).
No adversarial-review findings consulted (information-asymmetry wall, DF-025;
`.factory/phase-f5-adversarial/` not read).

Re-run rationale: FIX-P4-001 / FIX-F5-001 modified iec104.rs emit sites and `on_data`
(source_ip/timestamp/direction threaded through process_u_frame / detect_iec104_threats /
track_ns_desync). All iec104.rs-dependent checks were RE-RUN against b36b884.

---

## Per-step results

| Step | Tool / command | Outcome |
|------|----------------|---------|
| 2 Kani VP-044 | `cargo kani --harness verify_parse_apci_header_safety` | SUCCESSFUL — 0 of 89 failed |
| 2 Kani VP-004 | `verify_content_first_precedence_exhaustive` | SUCCESSFUL — 0 of 440 failed |
| 2 Kani VP-004 | `verify_tls_signature_beats_port` | SUCCESSFUL — 0 of 407 failed |
| 2 Kani VP-004 | `verify_none_two_phase_caching` | SUCCESSFUL — 0 of 183 failed |
| 2 Kani VP-007 | `verify_all_emitted_ids_resolve` | SUCCESSFUL — 0 of 122 failed |
| 2 Test VP-007 | `cargo test vp007_catalog_drift_guard` | ok (1 passed); SEEDED_TECHNIQUE_ID_COUNT = 29 confirmed |
| 2 Proptest VP-045/046 | `cargo test --test iec104_analyzer_tests proptest_vp04` | ok (3 passed) |
| 3 Fuzz VP-047 | `cargo +nightly fuzz run fuzz_iec104_parser -- -max_total_time=300` | 0 crashes — 2,642,251 runs, 301 s, ~8,778 exec/s |
| 4 Mutation | `cargo mutants --file src/analyzer/iec104.rs` (scoped re-run) | 95.9% production kill (118/123); 0 killable survivors; 5 equivalent |
| 5 Security | `cargo audit` | exit 0 — 0 vulns / 0 warnings across 193 deps |
| 5 Security | semgrep | SKIPPED (not installed; justified) |
| 6 Regression | `cargo test --all-targets` | 2627 passed, 0 failed, 5 ignored |
| 6 Lint | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings |
| 6 Format | `cargo fmt --check` | clean |

## Mutation measurement caveat (resolved)

First run (`-j4`, full ~2600-test suite, 185 s auto-timeout) oversubscribed CPU and
misclassified 96 caught/missed mutants as TIMEOUT, making `missed=0` untrustworthy.
Proof: `extract_ns >> → <<` was TIMEOUT there but fails 4 tests in 0.07 s (CAUGHT).
Re-run scoped to a fast test set (`-- --lib --test iec104_analyzer_tests`, `--timeout 60`)
gave the authoritative result. Invalid run preserved at `mutants.out.j4-invalid/`.

5 production survivors, all EQUIVALENT: `866` benign-arm ≡ catch-all; `949`/`967`
`| → ^` on bit-disjoint operands; `1195` `> → >=` carry==255 unreachable (residual ≤254);
`1358` `> → >=` findings-cap boundary yields identical state (drop=0, truncate no-op).
6 timeouts = genuine loop non-termination (`pos +=` → `-=`/`*=`) = killed.
35 "missed" are `#[cfg(kani)]` proof-harness sites (not compiled by cargo test) — out of
production scope; harness itself verified by VP-044 Kani.

## Overall F6 gate verdict: PASS

Kani all SUCCESSFUL; fuzz 0 crashes (2.64M, ≥5 min); mutation 95.9% ≥ 90% with 0 killable
survivors; cargo-audit no unresolved CRITICAL/HIGH; regression 2627 pass / 0 fail,
clippy + fmt clean.

## BLOCKERs: None. develop not modified. Steps 7b (DTU) and 7d (accessibility) skipped per instruction.

## Companion artifacts
- kani-results.md, fuzz-results.md, mutation-results.md, security-scan-results.md
- Prior Modbus (#7) generics archived to `archive/feature-7-modbus/`.
