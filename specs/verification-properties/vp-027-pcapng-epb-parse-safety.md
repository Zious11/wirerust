---
artifact: verification-property
vp_id: VP-027
title: "pcapng EPB Parse Safety, interface_id Discriminant, and Padding-Overrun Classification"
status: verified
phase: P1
tool: Kani
subsystem: SS-01
module: "reader.rs (pcapng_pure_core fns)"
producer: architect
timestamp: 2026-06-19T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-009-pcapng-reader-design.md
feature_cycle: feature-pcapng-reader
source_bc: BC-2.01.012
bcs:
  - BC-2.01.012
verification_lock: true
verified_at_commit: "1ca30a3"
verified_prs: "#287, #293, #294"
---

# VP-027: pcapng EPB Parse Safety, interface_id Discriminant, and Padding-Overrun Classification

## Property Statement

The pure-core function `decode_epb_body(body: &[u8], interface_table: ...)` in
`src/reader.rs` satisfies the following properties for **all possible symbolic inputs**:

1. **No panic:** The function never panics on any input length or content.
2. **interface_id discriminant (two distinct cases — not slash notation):**
   - Empty interface table → `Err(E-INP-009)`.
   - `interface_id` out of bounds on a non-empty table → `Err(E-INP-010)`.
   These two cases are explicitly separate; the original `E-INP-009 / E-INP-010` slash
   notation was ambiguous and has been replaced with two distinct assertions (Decision 22 /
   F-H4 / ADR-009 rev 9).
3. **Padding-overrun and bound-by-body classification:** EPB padding-overrun and
   bound-by-body failures → `Err(E-INP-008)` (wirerust body-decode failures after
   pcap-ng crate framing — NOT E-INP-010, which is reserved for the crate framing layer).
4. **Non-vacuity:** Confirmed via deliberate-flip negative test (changing an assertion
   and verifying Kani reports FAILURE). The proof is not a tautological stub.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.01.012 | EPB parse safety: no panic; interface_id discriminant (E-INP-009 vs E-INP-010); padding-overrun → E-INP-008 | All harnesses |

## Module Anchor Clarification

**Kani target is `decode_epb_body` (pure-core), not `from_pcap_reader<R: Read>` (effectful).**
The module label `reader.rs (pcapng_pure_core fns)` denotes the compilation unit.
BMC tractability: the proof uses an `EpbDecodeError` discriminant twin `decode_epb_body_discriminant`
that mirrors the production `decode_epb_body` path line-by-line. Twin faithfulness confirmed
in PR review.

## SEC-001 Twin-Drift Risk

The `decode_epb_body_discriminant` twin can diverge silently from the production
`decode_epb_body` if the production function is refactored without updating the twin.
A `#[cfg(test)]` equivalence smoke test (tracked follow-up obligation) would detect
divergence. Until present, re-running `cargo kani` is the primary divergence detector.
SEC-001 tracked as a follow-up to the F6 lock.

## Proof History (F-F5P1-001)

VP-027's original harness (pre-F5) was a tautological stub that passed vacuously. The
proof was rewritten in Phase F5 (PR #287 @ develop 97c66b0): the real `decode_epb_body`
is now called directly with symbolic inputs; 687 checks, non-vacuity confirmed via
deliberate-flip. Status changed from `draft` to `active` at F5; locked as `verified`
at F6 (re-confirmed SUCCESSFUL @ develop 1ca30a3, PRs #293 + #294).

## Proof Harness

```rust
// VP-027: decode_epb_body never panics; interface_id discriminant correctly classified;
// padding-overrun → E-INP-008 (not E-INP-010).
// BMC tractability: proof operates on decode_epb_body_discriminant twin.
#[kani::proof]
fn vp027_epb_parse_safety() { ... }
```

Harness reports `cargo kani VERIFICATION SUCCESSFUL` (687 checks). Non-vacuity confirmed.
Locked at develop@1ca30a3 (PRs #293 + #294).

## Feasibility Assessment

**Assessment: FEASIBLE (completed — SUCCESSFUL at F6 lock).**

Pure-core decode function; BMC tractability achieved via discriminant twin. The twin
faithfully mirrors the production decode path (PR review confirmed); 687 checks is within
practical budget. Non-vacuity requirement (DF-KANI-NONVACUITY-001) met by deliberate-flip
negative test.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-027 designed, added to VP-INDEX (ADR-009 rev 4); property amended rev 8/9 | draft |
| F5 (adversarial refinement) | Tautological stub replaced with genuine non-vacuous proof (F-F5P1-001, PR #287 @ develop 97c66b0); 687 checks SUCCESSFUL | draft → active |
| F6 (formal hardening) | Re-confirmed SUCCESSFUL (687 checks) @ develop 1ca30a3 (PRs #293 + #294); non-vacuity confirmed | active → verified |

Lock: `status: verified`, `verification_lock: true` set by state-manager after F6 confirmation.
