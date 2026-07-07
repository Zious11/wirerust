# Security Review — STORY-149 / PR #374

**Story:** STORY-149 — TLS Carry-Path Performance Recovery + Fragmented-Handshake Benchmark Fixture
**PR:** #374 — https://github.com/Zious11/wirerust/pull/374
**Branch:** feature/STORY-149-tls-carry-perf → develop
**Reviewer:** vsdd-factory:security-reviewer (automated)
**Date:** 2026-07-07
**Verdict:** APPROVE

---

## Files Reviewed

- `src/analyzer/tls.rs` (hot-path restructure: `prepare_record_step` + `process_handshake_carry`)
- `benches/tls_fragmented.rs` (new Criterion benchmark fixture)
- `tests/common/tls_fragmented_fixture.rs`
- `tests/bc_149_fragmented_fixture_tests.rs`
- `tests/bc_149_single_borrow_invariant_tests.rs`
- `Cargo.toml` (dependency audit)

---

## Summary

| Severity | Count | Blocks Merge? |
|----------|-------|---------------|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 0 | — |
| LOW | 2 | No (test/bench code only) |
| INFO | 5 | — |

**Overall verdict: APPROVE — no CRITICAL, HIGH, or MEDIUM findings.**

---

## Findings

### SEC-001 — `wrap_as_tls_record` silently truncates TLS record length

- **Severity:** LOW
- **CWE:** CWE-704 (Incorrect Type Conversion or Cast); related CWE-190 (Integer Overflow)
- **Location:** `tests/common/tls_fragmented_fixture.rs:19-22`
- **Blocks Merge:** No — test/bench code only; current payloads bounded to 15 bytes (well below u16::MAX)
- **Description:** `wrap_as_tls_record` casts `payload.len()` (usize) to two bytes without
  asserting `payload.len() <= u16::MAX`. Payloads > 65535 bytes would produce a silently
  corrupt TLS record header. Not exploitable against production code.
- **Proposed Mitigation (deferred):**
  ```rust
  debug_assert!(
      payload.len() <= u16::MAX as usize,
      "wrap_as_tls_record: payload length {} exceeds u16::MAX; TLS record length field truncated",
      payload.len()
  );
  ```

### SEC-002 — Borrow-budget inspection test does not cover `self.flows[key]` index syntax

- **Severity:** LOW
- **CWE:** CWE-693 (Protection Mechanism Failure — incomplete invariant enforcement)
- **Location:** `tests/bc_149_single_borrow_invariant_tests.rs` (anti-gameability test)
- **Blocks Merge:** No — forward-looking gap; current code has zero `self.flows[` usages
- **Description:** The borrow-budget CI tests grep for `.get(` / `.get_mut(` but not for
  `self.flows[key]` (HashMap index-operator). A future contributor using Index syntax would
  evade the CI count, silently reintroducing PERF-001 overhead.
- **Proposed Mitigation (deferred):** Add `self.flows[` to the anti-gameability assert:
  ```rust
  assert!(!body.contains("self.flows["), "...index syntax evades borrow-budget count...");
  ```

---

## Confirmed Safe (INFO)

1. **No `unsafe` blocks introduced.** Zero `unsafe`, `unwrap_unchecked`, `get_unchecked`,
   `from_raw`, or `transmute` occurrences in the diff.

2. **`std::mem::take` swap preserves carry-buffer invariants.** Traced across all four
   decision paths: empty-carry hot path, multi-record accumulation, Decision-4 body-len
   spoof guard, Decision-5 pre-append overflow guard. Semantically equivalent to pre-refactor.

3. **No integer overflow in hot-path arithmetic.** All index arithmetic guarded by prior
   bounds checks (u16::from_be_bytes capped at 65535, `MAX_RECORD_PAYLOAD = 18432` guard,
   `MAX_BUF = 65536` carry overflow guard). `consumed <= carry.len()` invariant maintained.

4. **No new malformed-packet exploitation surface.** Structural transformation only; same
   Decision-4 body-len spoof guard intact in `prepare_record_step`.

5. **No injection, authentication bypass, or information disclosure.** Packet-parsing
   pipeline; no user-controlled data used to construct system calls, SQL, or log output.

---

## Dependency Audit

No new dependencies added to `Cargo.toml`. `cargo audit` status: inherited from develop
baseline (workflow run 28846042595 — Audit check: PASS).
