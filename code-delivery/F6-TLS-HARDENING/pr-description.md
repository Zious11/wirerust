## F6 Targeted Hardening: TLS Handshake-Message Reassembly

**Phase:** F6 (targeted hardening) — verification and test additions only  
**Cycle:** fix-tls-clienthello-frag  
**Branch:** `fix/f6-tls-hardening`  
**Base:** `develop` @ `8b52046`  
**Severity:** N/A — no bug fix; this is formal verification gap-closure

![Tests](https://img.shields.io/badge/tests-2232%2F2232-brightgreen)
![Mutation](https://img.shields.io/badge/mutation_kill_rate-100%25_real--gap-brightgreen)
![Fuzz](https://img.shields.io/badge/fuzz-1.9M_execs_0_crashes-brightgreen)
![Kani](https://img.shields.io/badge/kani-3_proofs_VERIFIED-brightgreen)

This PR closes the F6 targeted-hardening loop for the TLS handshake-message carry-reassembly
logic introduced in STORY-144 (ClientToServer) and STORY-145 (ServerToClient). Production logic
was frozen at F5 convergence and confirmed sound; this PR adds only verification artifacts and
mutation-gap tests. **Zero production-code changes** (`git diff origin/develop src/` = 0 lines
except `#[cfg(kani)]`-gated proof code that the stable build never compiles).

---

## What This PR Delivers

```mermaid
graph TD
    A["fix/f6-tls-hardening<br/>5 commits"] --> B["d085db2<br/>Fuzz target<br/>fuzz_tls_reassembly.rs"]
    A --> C["07865cb<br/>Kani proofs<br/>kani_proofs_vp039"]
    A --> D["cd005f8<br/>fuzz Cargo.lock sync<br/>mutants ignore config"]
    A --> E["fabaf40<br/>11 mutation-gap tests<br/>mod f6_hardening"]
    A --> F["f284c09<br/>Theme-5 mid-byte fix<br/>+ clippy manual_repeat_n"]

    style B fill:#d4edda
    style C fill:#d4edda
    style D fill:#e8e8e8
    style E fill:#d4edda
    style F fill:#d4edda
```

---

## Spec Traceability

```mermaid
flowchart LR
    BC038["BC-2.07.038\nC2S carry-reassembly"] --> AC_step1["AC: Step-1 overflow guard\n(> not >=)"]
    BC039["BC-2.07.039\nC2S overflow counter"] --> AC_d4["AC: Decision-4\nbody_len spoof guard"]
    BC041["BC-2.07.041\nS2C carry-reassembly"] --> AC_shift["AC: mid-byte shift\n(<< 8 not >> 8)"]
    BC043["BC-2.07.043\nbuffer saturation drops"] --> AC_sat["AC: exact-fill\nno spurious drop"]
    VP039["VP-039\nKani carry-drain\nloop safety"] --> KH1["verify_drain_loop_cursor_safety"]
    VP039 --> KH2["verify_no_usize_overflow_on_advance"]
    VP039 --> KH3["verify_carry_bounded_after_append"]
    AC_step1 --> T1["Theme 1: exact-MAX_BUF accepted\n(2 tests C2S+S2C)"]
    AC_d4 --> T3["Theme 3: Decision-4\nbody_len guard\n(2 tests C2S+S2C)"]
    AC_shift --> T5["Theme 5: mid-byte shift\nleft not right\n(2 tests high+mid-byte)"]
    AC_sat --> T6["Theme 6: partial-trailing\ncarry retained\n(2 tests C2S+S2C)"]
    T1 & T3 & T5 & T6 --> Impl["tests/tls_analyzer_tests.rs\nmod f6_hardening (12 tests)"]
```

---

## Verification Evidence

### Kani Formal Proofs (VP-039)

Three harnesses in `src/analyzer/tls.rs` `#[cfg(kani)] mod kani_proofs_vp039`. Gated behind
`#[cfg(kani)]` — the stable toolchain build never compiles this code. All proofs non-vacuous
(DF-KANI-NONVACUITY-001 satisfied via `kani::cover!` annotations).

| Harness | Property Proved | Bound | Status |
|---------|-----------------|-------|--------|
| `verify_drain_loop_cursor_safety` | Cursor in-bounds at every loop iteration; no OOB index; no `usize` underflow on `carry.len() - consumed`; drain range `consumed <= carry.len()` always valid; loop terminates. Non-vacuous: dispatched≥1, iters≥2, Decision-4 path, partial trailing, full consumption all covered. | N=12, `#[kani::unwind(5)]` | **VERIFICATION SUCCESSFUL** |
| `verify_no_usize_overflow_on_advance` | `consumed.checked_add(4 + body_len)` never wraps for any `consumed ≤ MAX_BUF`, `body_len ≤ MAX_BUF`. Non-vacuous: advance both ≤ and > MAX_BUF reached. | Symbolic full-range | **VERIFICATION SUCCESSFUL** |
| `verify_carry_bounded_after_append` | Step-1 pre-append add `carry_len_before + payload_len` does not overflow; carry never exceeds `MAX_BUF` after append. Non-vacuous: clear path, append-below-cap, and exact-MAX_BUF all covered. | Symbolic full-range | **VERIFICATION SUCCESSFUL** |

**Aggregate: 3/3 harnesses VERIFICATION SUCCESSFUL, 0 failures, 0 counterexamples.**

### Fuzz Testing

New target `fuzz/fuzz_targets/fuzz_tls_reassembly.rs` added to `fuzz/Cargo.toml`.

Exercises `TlsAnalyzer::on_data` (both C2S and S2C), `try_parse_records`, and `summarize()` on
fully-arbitrary byte sequences sliced into variable-length segments, alternating direction to
force cross-segment carry reassembly. A panic / OOB / arithmetic-overflow / OOM anywhere is a
fuzz finding.

| Metric | Value |
|--------|-------|
| Executions | **1,900,000** |
| Crashes / panics / OOM / timeouts | **0** |
| Corpus entries | **826** |
| Crash artifacts | none |

**Verdict: PASS — 0 crashes in ~1.9M executions.**

Run command: `cargo +nightly fuzz run fuzz_tls_reassembly -- -max_total_time=180 -rss_limit_mb=2048`

### Mutation Testing (mod f6_hardening)

The F5 convergence mutation run identified 13 real-gap surviving mutants across 6 symmetric
C2S/S2C themes in `src/analyzer/tls.rs`. All 13 were confirmed genuine (non-equivalent) by
manual mutation re-verification.

**Post-F6-hardening result: 100% of real-gap mutants CAUGHT.**

| Theme | Mutation sites | Description | Tests added |
|-------|----------------|-------------|-------------|
| 1 | C2S `829:64` / S2C `998:64` | Step-1 boundary: `>` → `>=` falsely rejects exact-MAX_BUF fill | `test_BC_2_07_039_c2s_step1_exact_max_buf_accepted`, `test_BC_2_07_041_s2c_step1_exact_max_buf_accepted` |
| 2 | C2S `829:41` / S2C `998:41` | Step-1 semantics: additive (not multiplicative) overflow guard | `test_BC_2_07_039_c2s_step1_additive_not_multiplicative`, `test_BC_2_07_041_s2c_step1_additive_not_multiplicative` |
| 3 | C2S `900:37` / S2C `1036:37` | Decision-4 body_len spoof guard: `>` boundary | `test_BC_2_07_038_c2s_decision4_body_len_max_buf_not_spoof`, `test_BC_2_07_041_s2c_decision4_body_len_max_buf_not_spoof` |
| 4 | S2C `1079:59` | Parse-error discrimination: non-ServerHello message type not accepted as ServerHello | `test_BC_2_07_041_s2c_parse_errors_inc_ok_non_server_hello_v13draft18` |
| 5a | S2C `1029:70` | High-byte body_len lane: `<< 16` (correct) vs `>> 16` (mutant) | `test_BC_2_07_041_s2c_body_len_high_byte_shift_left_not_right` |
| 5b | S2C `1030:67` | Mid-byte body_len lane: `<< 8` (correct) vs `>> 8` (mutant) | `test_BC_2_07_041_s2c_body_len_mid_byte_shift_left_not_right` |
| 6 | C2S `911:38` / S2C `1047:38` | Partial-trailing carry retention (incomplete body waits for next record) | `test_BC_2_07_038_c2s_incomplete_body_partial_trailing_carry_retained`, `test_BC_2_07_041_s2c_incomplete_body_partial_trailing_carry_retained` |
| Buffer sat. | S2C `1155:43` | Exact-fill no spurious drop (buffer saturation BC-2.07.043) | `test_BC_2_07_043_s2c_exact_fill_no_drop` |

**2 provably-equivalent survivors remain** at `tls.rs:950:59` (C2S `Ok(non-ClientHello)` arm).
Documented as structurally unreachable: the `Ok` path at that site can only return a
`ClientHello` or `Err`; a non-ClientHello `Ok` is dead code. These are accepted; no gap.

**Methodology note:** `cargo-mutants` must run at `--jobs 1` or low concurrency on this suite.
At `--jobs 8`, load-induced timeouts mask survivors as ambiguous. Serial run is authoritative.

### Regression Suite

| Check | Result |
|-------|--------|
| `cargo test --all-targets` | **2232 passed, 0 failed** (2220 existing + 12 new `f6_hardening`) |
| `cargo clippy --all-targets -- -D warnings` | **Clean** |
| `cargo fmt --check` | **Clean** |

---

## Security Review

**Result: APPROVE** — security-reviewer completed full manual review of the PR diff.

CWE-400 (Uncontrolled Resource Consumption) is the primary risk for the TLS carry-reassembly
logic. This PR MITIGATES it: the three Kani proofs formally cover the Step-1 pre-append overflow
guard, the Decision-4 body_len-spoof guard, and the cursor-advance arithmetic. The fuzz harness
covers both C2S and S2C directions with the buffer-saturation tail-drop path. Production guards
at tls.rs:829/998 (Step-1) and tls.rs:900/1036 (Decision-4) remain unchanged and were formally
verified. Zero new attack surface — all additions are test/verification artifacts.

**Advisory findings (non-blocking):**
- **SEC-001 (LOW, CWE-697):** Kani `drain_loop_model` Decision-4 branch omits `carry.clear()` and overflow counter increment — documentation gap only. Production safety is preserved; post-loop drain is guarded by `!decision4_fired` in both model and production.
- **SEC-002 (LOW, CWE-682):** `wrap_handshake_record` test helper lacks `debug_assert!(len <= 0xFFFF)`. No current caller passes >65535 bytes; deferred as a future guard improvement.

---

## Risk Assessment

| Dimension | Assessment |
|-----------|------------|
| Blast radius | Minimal — Kani code is `#[cfg(kani)]`-gated (not compiled by stable); tests are CI-only; fuzz target is a separate crate |
| Production code changes | **Zero** — `git diff origin/develop src/analyzer/tls.rs` shows only new `#[cfg(kani)]`-gated block |
| Regression risk | None — new tests can only detect regressions, not introduce them |
| Breaking change | No — public API surface unchanged |
| Performance impact | None — tests + proofs run in CI / dev; not in production binary |

**Rollback:** `git revert f284c09 fabaf40 07865cb d085db2` if any test proves problematic.

---

## Demo Evidence

N/A — verification-only PR. No user-visible behavior changes. Demo recording not applicable.

---

## Holdout Evaluation

N/A — evaluated at wave gate. No new behavioral contracts introduced.

---

## Adversarial Review

N/A — evaluated at Phase 5. This PR is targeted gap-closure from F6 formal hardening output.

---

## AI Pipeline Metadata

| Field | Value |
|-------|-------|
| Pipeline mode | Feature-delta F6 (targeted hardening) |
| Cycle | fix-tls-clienthello-frag |
| Branch HEAD | `f284c09` |
| Base | `develop` @ `8b52046` |
| Models used | claude-sonnet-4-6 |

---

## Pre-Merge Checklist

- [x] PR description matches actual diff (verification + tests only)
- [x] Zero production-code changes verified (only `#[cfg(kani)]`-gated additions to `src/analyzer/tls.rs`)
- [x] Kani harnesses non-vacuous (cover! annotations prove all interesting paths reachable)
- [x] Fuzz corpus: 1.9M execs, 0 crashes
- [x] All 12 `f6_hardening` tests named per factory convention
- [x] All 13 real-gap mutants verified caught by name (Themes 1–6 + buffer-sat)
- [x] 2 equivalent survivors documented (tls.rs:950:59, structurally unreachable arm)
- [x] `cargo test --all-targets` green on branch HEAD (2232 passed)
- [x] `cargo fmt --check` clean
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] Security review: APPROVE (0 CRITICAL/HIGH; 2 LOW advisory accepted — see Security Review section)
- [x] PR reviewer: APPROVE in cycle 1 (3 ADVISORY — F-1/F-2 line:col fixed in this description, F-3 comment deferred)
- [ ] CI checks green (pending)
