---
artifact: verification-property
vp_id: VP-026
title: "pcapng SHB Parse Safety and Byte-Order Detection"
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
source_bc: BC-2.01.010
bcs:
  - BC-2.01.010
verification_lock: true
verified_at_commit: "1ca30a3"
verified_prs: "#293, #294"
---

# VP-026: pcapng SHB Parse Safety and Byte-Order Detection

## Property Statement

The pure-core function that decodes an SHB (Section Header Block) body in `src/reader.rs`
satisfies the following properties for **all possible symbolic `&[u8]` inputs**:

1. **No panic:** The function never panics regardless of input length, content, or
   byte-order magic value.
2. **Byte-order detection:** The SHB body contains a 4-byte byte-order magic field
   (0x1A2B3C4D for big-endian; 0x4D3C2B1A for little-endian). The function correctly
   identifies byte order from this field and returns an appropriate error discriminant
   for unrecognized magic values.
3. **Length gating:** The function returns an error for inputs shorter than the minimum
   valid SHB body length without accessing out-of-bounds bytes.

The proof uses a pure-core `parse_shb_body_discriminant` twin that mirrors the production
`parse_shb_body` path line-by-line, enabling BMC tractability.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.01.010 | SHB parse safety: no panic; byte-order magic detection; length gating | All harnesses |

## Module Anchor Clarification

**Kani target is the pure-core helper function only.** The target is the pure SHB body-decode
function extracted from the SHB parse path in `src/reader.rs`. The effectful
`from_pcap_reader<R: Read>` entry point is not a Kani target (I/O-carrying, generic bounds).
The module label `reader.rs (pcapng_pure_core fns)` denotes the compilation unit.

## SEC-001 Twin-Drift Risk

A `#[cfg(test)]` equivalence smoke test (`tests/sec_shb_twin_equivalence_tests.rs`) guards
`parse_shb_body_discriminant` against divergence from production `parse_shb_body`.
The test suite includes 6 unit tests + a 2000-case proptest. Until a future refactor diverges
the twin, re-running `cargo kani` is also sufficient to detect divergence.

## Proof Harness

```rust
// VP-026: parse_shb_body never panics; correctly detects byte-order magic.
#[kani::proof]
#[kani::unwind(21)]
fn vp026_shb_parse_safety() { ... }
```

Harness reports `cargo kani VERIFICATION SUCCESSFUL` (272 checks), non-vacuity confirmed.
Locked at develop@1ca30a3 (PRs #293 + #294).

## Feasibility Assessment

**Assessment: FEASIBLE (completed — SUCCESSFUL at F6 lock).**

Pure-core decode function; `#[kani::unwind(21)]` is sufficient for the byte-order field
access pattern; 272 checks within practical BMC budget.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-026 designed, added to VP-INDEX (ADR-009 rev 4) | draft |
| F4 (TDD implementation) | Harness authored; twin-drift tripwire added | draft → active |
| F6 (formal hardening) | `vp026_shb_parse_safety` cargo kani VERIFICATION SUCCESSFUL (272 checks); non-vacuity confirmed | active → verified |

Lock: `status: verified`, `verification_lock: true` set by state-manager after F6 confirmation
@ develop 1ca30a3 (PRs #293 + #294).
