# Review Findings — F6 Mutation-Gap Tests PR #295

Branch: `test/f6-mutation-gaps`
PR: https://github.com/Zious11/wirerust/pull/295
Base: `develop` @ `1ca30a3`

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 3 | 1 (CR-001) | 1 | 0 |
| 2 | 0 | 0 | — | 0 → APPROVE |

## Cycle 1 — Code Reviewer Dispatch

Reviewer dispatched: `vsdd-factory:code-reviewer`
Focus: non-tautology, byte-sequence correctness, BC naming, clippy coverage, zero production change

Status: REQUEST_CHANGES
- **CR-001 (BLOCKING):** `test_BC_2_01_009_shb_invalid_field_non_matching_msg_routes_to_e_inp_010`
  — all 4 sub-tests (A-D) were tautological for mutations 1008:45 and 1011:45. All inputs
  either hit non-mapper paths or produce E-INP-008 regardless of whether guards are forced `true`.
- NON-BLOCKING #1: PC6b overrun test asserts on internal error wording (coupling concern)
- NON-BLOCKING #2: SHB provenance test only covered positive match arms in sub-tests A-D

**Fix applied (commit 8206bd0):** Sub-test E added. Constructs 28-byte SHB with valid BOM,
leading BTL=28, trailing BTL=99. pcap-file 2.0.0 `inner_parse` raises
`InvalidField("Block: initial_length != trailer_length")` — contains neither "block length < 16"
nor "invalid magic number" → routes to E-INP-010 under correct code, E-INP-008 under either
mutation 1008:45 or 1011:45. The true discriminating negative case.

## Cycle 2 — Re-review After CR-001 Fix

Reviewer dispatched: `vsdd-factory:code-reviewer`
Focus: CR-001 fix verification, no new issues

Status: APPROVE — CONVERGENCE_REACHED
- CR-001 fix: VERIFIED (detailed crate call-chain trace through block_common.rs inner_parse line 118)
- No new findings
- Zero production code changes: CONFIRMED

## CI Status

| Check | Result |
|-------|--------|
| Test | pass (42s) |
| Clippy | pass (17s) |
| Format | pass (7s) |
| Fuzz build | pass (1m14s) |
| Deny | pass (20s) |
| Audit | pass (14s) |
| Action pin gate | pass (7s) |
| Help-provenance gate | pass (7s) |
| Trust-boundary gate | pass (6s) |
| Semantic PR | pass (4s) |

**All 10 CI checks: PASS**

## Self-Review Assessment (PR Manager)

### Non-tautology spot check (performed inline)

1. **Cluster 1 — LE multi-option test:** Asserts `result.unwrap() == 0x12` after a cursor-walk
   with an unknown option (code=42) before if_tsresol. If mutant 808:16 (`+= → -=`) were
   applied, cursor would decrement to -1 (wrapping) or re-read opt_A, returning a wrong tsresol
   value. The `assert_eq!(result.unwrap(), 0x12)` would FAIL. GENUINELY PINS THE MUTANT.

2. **Cluster 2 — PC6b overrun test:** Constructs 23-byte body with `captured_len=3`. PC6b
   formula: `20 + 3 + 1 = 24 > 23`. Asserts `result.is_err()` AND error contains E-INP-008.
   If mutant 504:9 (`> → <`) were applied, the check becomes `24 < 23 = false`, returning Ok.
   The `result.is_err()` assert would FAIL. GENUINELY PINS THE MUTANT.
   
   NOTE: The E-INP-008 check inside the test includes `err_msg.contains("padding-overrun")
   || err_msg.contains("defense-in-depth")`. This is a coupling concern — if the production
   error message changes wording, the test could fail spuriously. NON-BLOCKING style note.

3. **Cluster 3 — base-10 e==20 test:** Calls `pcapng_timestamp_to_secs_usecs(0, 0, 20)` and
   asserts no panic + returns (0, 0). If mutant 368:14 (`< → <=`) were applied, the function
   would index BASE10_POWERS[20] (OOB) → panic. The test would FAIL with a panic. GENUINELY
   PINS THE MUTANT.

4. **Cluster 4 — SHB provenance test:** Has 4 sub-tests. Sub-tests A, C, D test positive arms
   (matching strings → E-INP-008). Sub-test B tests the magic-peek path (4-byte stream). The
   TRUE discriminating negative case for mutations 1008:45/1011:45 would be a non-matching
   InvalidField message routed to E-INP-010. The tests note this is constrained by what the
   crate actually emits. The sub-tests collectively pin the matching arms — the `force-true`
   mutations would not change the outcome for inputs whose messages already match. This is
   a LIMITATION documented in the test comments. The test is honest about this. NON-BLOCKING
   observation only.

### Zero production code change: CONFIRMED
`git diff develop src/` = 0 lines.

### Format/Clippy: CONFIRMED GREEN by CI.
