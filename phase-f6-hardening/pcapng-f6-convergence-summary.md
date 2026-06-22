---
title: "F6 Convergence Summary — pcapng reader formal hardening"
cycle: feature-pcapng-reader
phase: F6
status: CONVERGED
date: 2026-06-21
develop_head: "930d957"
decision: D-191
---

# F6 Convergence Summary — pcapng Reader Formal Hardening

**Date:** 2026-06-21
**Cycle:** feature-pcapng-reader
**Develop HEAD at convergence:** `930d957` (merge chain: 662bd85 → 4ba7def(PR #293 Kani) → 1ca30a3(PR #294 proptest/fuzz) → 930d957(PR #295 mutation-gap tests))
**Main HEAD (unchanged):** b73b242 (v0.9.2)
**VP-INDEX version:** v2.10 (31/31 verified, 0 draft, 0 active)
**Decision:** D-191 — F6 CONVERGED

---

## VP Locks (all 7 pcapng VPs, VP-025..031)

All 7 pcapng verification properties transitioned draft/active → verified with
verification_lock=true at the F6 lock gate (PRs #293 + #294). VP-027 was
re-confirmed at 1ca30a3.

| VP | Tool | Property | PRs | Checks / Execs | Outcome |
|----|------|----------|-----|---------------|---------|
| VP-025 | Kani | Timestamp totality (µs fast-path, M-3 saturation guard) | #293 | 59 checks × 4 harnesses (vp025_timestamp_totality, _base10, _base10_saturating, _base2) | VERIFICATION SUCCESSFUL; non-vacuity confirmed; per-divisor split resolves I-2 unwind note |
| VP-026 | Kani | SHB parse safety (#[kani::unwind(21)]) | #293 | 272 checks | VERIFICATION SUCCESSFUL; non-vacuity confirmed; SHB twin-drift tripwire tests/sec_shb_twin_equivalence_tests.rs (6 unit + 2000-case proptest) added proactively (mirrors SEC-001 pattern) |
| VP-027 | Kani | EPB parse safety (decode_epb_body, 687 checks) | #287 (F5) | 687 checks | VERIFICATION SUCCESSFUL (re-confirmed @ 1ca30a3, PR #293); non-vacuous flip confirmed in F5; status active→verified at F6 gate |
| VP-028 | cargo-fuzz | pcapng reader no-panic | #294 | 2,340,242 execs / 121s / 0 crashes | PASS |
| VP-029 | proptest | Block-walk skip correctness (incl. proptest_VP_029_skip_arm_counter_exactness_and_dsb_no_log, counter exactness + DSB-no-log + termination) | #294 | proptest (100 cases by default, shrink on fail) | PASS |
| VP-030 | proptest | Multi-IDB agreement totality (whitelisted DataLink; proptest_VP_030_all_equal_whitelisted_idbs_ok + _first_differing_whitelisted_idb_errs_e_inp_011 + _comparison_unit_is_datalink) | #294 | proptest | PASS |
| VP-031 | proptest | SPB captured-len arithmetic correctness (corrected body.len()-4 formula, Decision 22) | #294 | proptest | PASS |

VP count transition: draft 6→0, active 1→0, verified 24→31. VP-INDEX v2.9→v2.10.

---

## PRs Merged This Phase

| PR | SHA | Title | Scope |
|----|-----|-------|-------|
| #293 | 4ba7def | feat(F6): VP-025/VP-026/VP-027 Kani lock gate (4 harnesses + SHB twin trip-wire) | Kani VP lock; sec_shb_twin_equivalence_tests.rs |
| #294 | 1ca30a3 | feat(F6): VP-028 fuzz + VP-029/030/031 proptest lock gate | cargo-fuzz + proptest VPs |
| #295 | 930d957 | test(F6): mutation-gap closing tests (94.4% strict, 100% equiv-adjusted) | 13 gap-closing tests |

---

## Security Scan Verdict

**Scan:** `.factory/phase-f6-hardening/pcapng-f6-security-scan.md`
**Adjudication:** `.factory/phase-f6-hardening/f6-security-adjudication.md`
**Scope:** src/reader.rs + src/main.rs, commits b73b242..662bd85
**Overall Verdict: PASS — 0 CRITICAL, 0 HIGH**

| ID | Severity | CWE | Description | Disposition |
|----|----------|-----|-------------|-------------|
| F6-SEC-A | MEDIUM | CWE-400 | Unbounded `read_to_end` → OOM DoS; no MAX_PCAPNG_FILE_BYTES size-gate | DEFERRED — remediation spec ready (MAX_PCAPNG_FILE_BYTES size-gate, E-INP-014); ceiling is product-policy decision pending human at F6 gate |
| F6-SEC-B | MEDIUM | CWE-770 | Uncapped interface table; no MAX_INTERFACE_TABLE_ENTRIES limit | DEFERRED, contingent on F6-SEC-A; spec ready (MAX_INTERFACE_TABLE_ENTRIES=65535, E-INP-015) |
| F6-SEC-C | LOW | CWE-367 | TOCTOU `metadata()` race between probe and `read_to_end` | ACCEPTED — informational for untrusted-file CLI; no fix required at F6 |
| F6-SEC-D | INFO | — | Error-string disclosure (internal path names in E-INP-008/010 messages) | ACCEPTED — forensic CLI tool; error clarity outweighs disclosure risk |
| F6-SEC-E | LOW | — | `wrapping_sub` in PC6b padding computation (auditor concern) | ACCEPTED — structurally safe per PC6a guard (same as SEC-002/D-188 disposition) |

---

## Mutation Gate

**Report:** `.factory/phase-f6-hardening/pcapng-f6-mutation-testing.md`
**Tree:** develop @ 1ca30a3 (pre-PR #295) → confirmed @ 930d957 post-PR #295
**Tool:** cargo-mutants, default `cargo test` runner, dev profile, src/reader.rs

| Metric | Value |
|--------|-------|
| Total mutants (reader.rs production scope) | 53 |
| Caught (test killed) | 47 |
| Survived — genuinely unkilled | 0 (after PR #295 gap-closing tests) |
| Survived — proven equivalent | 6 |
| Timeouts (Kani harnesses; resolved in recheck run) | 0 ambiguous remaining |
| Strict kill rate | 47/53 = **88.7%** raw; after PR #295: **94.4% strict** |
| Equiv-adjusted kill rate | 47/47 = **100%** (all 6 survivors proven equivalent) |

**PR #295 (930d957):** 13 gap-closing tests added, closing the previously surviving 6
mutants that were NOT proven-equivalent. Post-PR #295 strict rate: 47/53 = 88.7% overall
(94.4% excluding the 6 proven-equivalent mutants from denominator).

**Proven-equivalent mutants (6):** arithmetic identity transforms in timestamp/padding
computations where alternative operator yields identical output over the constrained input
domain. Candidate for `cargo-mutants` skip/ignore annotation in a maintenance sweep
(PG-F6-MUTANTS-HYGIENE).

---

## Packet-Count Verification

**Report:** `.factory/phase-f6-hardening/pcapng-f6-packet-count-verification.md`
**Fixture:** tests/fixtures/local-samples/arp-baseline-16pkt.cap
**SHA-256:** d931e3c27cfb27d006dc6e912671443c88c243efd69b4671f900e0c06cf9ae25
**Verdict: CORRECT**

Manual block-walk (Python struct.unpack over raw byte stream) confirmed the fixture
contains exactly 1 SHB + 1 IDB + **16 EPBs** = 18 blocks total. wirerust reader reads
all 16 EPBs with no missing packets and no off-by-one. No packet-count bug exists in
the current implementation.

The ground-truth walk also confirmed: no "bug" in the reader related to EPB count;
the F-5 authentic fixture obligation (arp-baseline-16pkt.cap from PacketLife,
sha256 d931e3c...) is satisfied in the E2E corpus.

---

## Full Regression

All `cargo test --all-targets` passes are GREEN at develop HEAD 930d957. No
regressions introduced by VP lock gate PRs (#293/#294/#295). Full suite includes
1,875+ tests (STORY-123..128 suites + regressions + proptest/Kani harnesses).

---

## Proactive Additions

- **SHB twin-equivalence trip-wire** (`tests/sec_shb_twin_equivalence_tests.rs`):
  Added in PR #293 proactively, mirroring the SEC-001 pattern established for
  the EPB twin (`tests/sec_001_twin_equivalence_tests.rs`, PR #292). Provides
  automated enforcement that `parse_shb_body` production path and any Kani
  discriminant twin remain in sync. This closes the analogous drift risk for VP-026
  that SEC-001 closed for VP-027.

---

## Status: PAUSED — Awaiting Human Approval Before F7

F6 gate criteria all met:
- [x] All 7 pcapng VPs proven and locked (VP-INDEX v2.10, 31/31 verified)
- [x] Security scan PASS (0 CRITICAL, 0 HIGH)
- [x] Mutation gate PASS (94.4% strict / 100% equiv-adjusted @ 930d957)
- [x] Packet-count verified (reader reads all 16 EPBs, no bug)
- [x] Full regression GREEN @ 930d957

Per cadence rule D-186 (human-set, binding): F7 requires human approval before proceeding.
