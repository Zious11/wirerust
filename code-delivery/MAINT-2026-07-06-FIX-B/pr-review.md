# Fresh-Eyes PR Review — PR #370

**PR:** [#370 feat(dnp3): surface silently-dropped state via observability counters](https://github.com/Zious11/wirerust/pull/370)
**Branch:** `fix/dnp3-observability-counters` → `develop`
**Reviewer role:** pr-reviewer (fresh-eyes, information-asymmetric)
**Verdict:** **APPROVE**

## Overall Assessment

This is a clean, purely additive observability change that faithfully implements what its description promises. Three monotonic counters (`dropped_findings`, `master_addrs_dropped`, `pending_requests_evicted`) are added to `Dnp3Analyzer` and exposed via `summarize()` JSON. The pattern mirrors PR #365/#366 precedent (ARP, Modbus, HTTP, TLS). No detection behavior changes; all invariants are preserved.

I reviewed every changed file in the diff (1 source file, 1 new test file). No BLOCKING or HIGH-severity issues survived verification.

## Findings Summary

| Severity | Count |
|---|---|
| BLOCKING | 0 |
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 3 |

## Detailed Findings

### MEDIUM-1 — `dropped_findings` counter is exercised at only 1 of 11 cap-drop sites

**File:** `tests/bc_2_15_020_dnp3_observability_counters_tests.rs`
**Category:** test-coverage
**Severity:** suggestion

The PR wires `*dropped_findings += 1` at 11 cap-check sites (`detect_control_class_burst_split`, `detect_restart_split`, `detect_write_split`, `scan_block_timeouts`, `maybe_emit_t0827`, `detect_broadcast_anomaly`, `detect_unexpected_source_split`, `detect_unsolicited_anomaly`, `detect_unsolicited_control` ×2 for FC=0x14 and FC=0x15, `check_malformed_anomaly`). Only **one** of these — the `detect_restart_split` path via a COLD_RESTART (FC=0x0D) — is exercised by `test_BC_2_15_022_dropped_findings_increments_when_all_findings_cap_hit`.

**Failure scenario:** A future maintainer refactors the cap-check block in, say, `check_malformed_anomaly` or `detect_broadcast_anomaly` and inadvertently drops the `else { *dropped_findings += 1; }` arm. All 9 tests still pass, but the counter silently under-counts for that class of dropped findings — regressing the observability property this PR was designed to add.

**Suggestion:** Add at least one parameterized (or per-branch) test that hits each of the 11 cap sites and verifies the counter increments monotonically. Not a merge blocker, but the PR description's claim of coverage across "11 cap-check sites" is stronger than the actual test surface. A follow-up PR is fine.

### MEDIUM-2 — PR body overclaims a negative test that isn't present

**File:** PR description
**Category:** description-accuracy
**Severity:** suggestion

The PR body states:
> "`dropped_findings`: verifies counter increments at MAX_FINDINGS boundary; verifies eviction counter does NOT fire on `scan_block_timeouts` age-out or on normal finding completion (negative tests)"

I could not find a dedicated negative test for `dropped_findings` on the `scan_block_timeouts` age-out path in the new test file. Semantically the concern is minor — `scan_block_timeouts` age-out only *removes* entries and only *emits* a finding on threshold crossing, so the counter cannot spuriously fire from age-out itself. But the PR description asserts test evidence that isn't there.

**Suggestion:** Either add the negative test (a `scan_block_timeouts` age-out sequence that verifies `dropped_findings == 0`) or soften the claim in the PR body to reflect what the tests actually assert.

### LOW-1 — Function-signature debt continues to grow

**File:** `src/analyzer/dnp3.rs` (multiple functions)
**Category:** maintainability
**Severity:** nit

Several detection functions now carry 8–11 positional parameters, gated by `#[allow(clippy::too_many_arguments)]`. The code already flags a future refactor to a context struct. This PR adds one more `&mut u64` arg to 10 functions, exacerbating the debt. Not a merge blocker (the pattern matches prior PRs #365/#366), but the context-struct refactor becomes more valuable with each additive counter.

### LOW-2 — `insert_pending_request` return-value semantics could be clearer at the call site

**File:** `src/analyzer/dnp3.rs:875`
**Category:** readability
**Severity:** nit

The new call site reads:
```rust
if Self::insert_pending_request(flow, (dest, app_seq), ts) {
    self.pending_requests_evicted += 1;
}
```

A boolean return without an explicit type alias or wrapper is a mildly opaque API — a reader has to jump to the function docstring to learn `true = eviction occurred`. The docstring is present and clear, so this is a nit, but a named enum (`InsertOutcome::{Inserted, Evicted}`) would be more self-documenting. Matches PR #366 precedent for `insert_binding_lru`, so consistency argues for leaving it as-is.

### LOW-3 — `detect_unsolicited_control` FC=0x14 sets `enable_unsolicited_seen = true` even when the finding is cap-suppressed

**File:** `src/analyzer/dnp3.rs:1677-1680`
**Category:** informational (pre-existing)
**Severity:** nit

Under the FC=0x14 branch, `flow.enable_unsolicited_seen = true;` runs unconditionally *before* the cap gate. This is pre-existing behavior — not introduced by this PR — but it means downstream `detect_unsolicited_anomaly` will treat the flow as "having seen enable" even when the enable-finding itself was dropped. This is the *correct* semantics (context flag != finding emission), and this PR does not change it, but it's worth noting for a maintainer who might later be tempted to move that assignment inside the cap-guarded block.

Not a finding against this PR — flagging only as informational for future work.

---

## Answers to the Fresh-Eyes Review Questions

1. **Does the PR description accurately describe what the code does?**
   Yes, on all major claims — the counter names, cap constants (MAX_FINDINGS=10_000, MAX_MASTER_ADDRS=64, MAX_PENDING_REQUESTS=256), the 11 cap-check sites, the 10 detection functions threading `&mut u64`, and the `insert_pending_request`-returns-bool pattern all match the diff. One minor overclaim: the negative test for `scan_block_timeouts` age-out on `dropped_findings` is asserted in the body but not implemented (MEDIUM-2).

2. **Is the test coverage adequate?**
   Adequate for the two counters with dedicated positive + negative pairs (`master_addrs_dropped`: 65th-address positive + known-address-at-cap negative; `pending_requests_evicted`: 257th-request positive + request/response-completion negative). Under-covered for `dropped_findings`, which is only exercised at 1 of the 11 cap-drop sites the PR introduces (MEDIUM-1). Negative invariant "counter events do not emit Findings" is well covered by the 3-part `test_DNP3_EVICTION_NO_FINDING_NEG_TEST_001`.

3. **Readability/maintainability concerns?**
   The parameter-count debt on the 10 detection functions (LOW-1) and the opaque `bool` return of `insert_pending_request` (LOW-2). Both mirror pre-existing patterns from PR #366, so consistency arguments favor accepting.

4. **Is the `master_addrs_dropped` observability-parity-only claim consistent with the code?**
   Yes. The code does not add any Finding emission on the drop event; the `contains()` check in `detect_unexpected_source_split` still fires for any src not in the (full) `master_addrs_seen` set, so T1692.001 detection is not gated by the cap. The PR body's explicit disclaimer ("PR does NOT claim this counter restores T1692.001 detection") is accurate.

5. **BLOCKING issues?**
   None.

---

## Merge Recommendation

**APPROVE.** This is a clean, purely additive observability change with faithful documentation and consistent pattern-matching to two prior PRs. The two MEDIUM findings are quality improvements — not correctness defects — and can be handled in a follow-up. Detection behavior, Finding emission, and all cap-invariants are preserved. +24 bytes per `Dnp3Analyzer` instance and one add per rare eviction event is a negligible cost for closing three genuine observability gaps.
