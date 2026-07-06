---
artifact: verification-property
vp_id: VP-031
title: "pcapng SPB Captured-Len Computation Correctness (body.len()-4 formula)"
status: verified
phase: P1
tool: proptest
subsystem: SS-01
module: "reader.rs (pcapng_pure_core fns)"
producer: architect
timestamp: 2026-06-19T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-009-pcapng-reader-design.md
feature_cycle: feature-pcapng-reader
source_bc: BC-2.01.013
bcs:
  - BC-2.01.013
verification_lock: true
verified_at_commit: "1ca30a3"
verified_prs: "#293, #294"
---

# VP-031: pcapng SPB Captured-Len Computation Correctness (body.len()-4 formula)

## Property Statement

The SPB (Simple Packet Block) captured-length computation in `src/reader.rs` satisfies
the following arithmetic correctness property for all valid SPB body inputs:

```
captured_len = min(original_len, spb_data_available)
             = min(original_len, body.len() as u32 - 4)
```

where:
- `original_len` is the 4-byte original packet length field at the start of the SPB body.
- `body.len() - 4` is the count of data bytes remaining after subtracting the 4-byte
  `original_len` header from the block body. This is `spb_data_available`.
- `snaplen` is **NOT** subtracted from the formula (Decision 9 rev 8 / H-3 + M-2
  SPB snaplen asymmetry fix — EPB also ignores snaplen, and SPB is symmetric with EPB).

**Formula correction history:** The formula was corrected from `min(original_len,
body.len() as u32)` (rev 8) to `min(original_len, body.len() as u32 - 4)` (rev 9 /
Decision 22 / F-H2 / F-H3). The rev 8 formula failed to subtract the 4-byte
`original_len` header from the body, yielding a slice length 4 bytes too large.

VP-031 fills the SPB framing VP gap identified in DF-CANONICAL-FRAME-HOLDOUT-001:
cargo-fuzz (VP-028) covers no-panic but cannot express the arithmetic relationship
between `original_len`, `snaplen`, and returned slice length. VP-031 provides the
missing arithmetic correctness property.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.01.013 | SPB captured-len: `min(original_len, body.len() as u32 - 4)`; snaplen NOT subtracted; formula correct for all valid SPB body inputs | Arithmetic proptest |

## Proof Harness

```rust
proptest! {
    // VP-031: captured_len == min(original_len, body.len() as u32 - 4) for all inputs.
    // Snaplen is NOT in the formula; this is the canonical SPB spb_data_available definition.
    fn proptest_vp031_spb_captured_len_formula(
        original_len: u32,
        extra_data in 0u32..65536,
    ) {
        let body_len = 4 + extra_data as usize;  // 4-byte original_len header + data
        let body = vec![0u8; body_len];
        let spb_data_available = body.len() as u32 - 4;
        let expected = original_len.min(spb_data_available);
        let actual = compute_spb_captured_len(original_len, &body);
        prop_assert_eq!(actual, expected);
    }
}
```

Harness confirmed against the `body.len()-4` formula at F6 lock @ develop 1ca30a3
(PRs #293 + #294).

## Feasibility Assessment

**Assessment: FEASIBLE (completed — proptest harness confirmed at F6 lock).**

The SPB captured-len formula is a pure arithmetic computation over two u32 inputs.
proptest is the natural tool for arithmetic invariant verification; the formula is
simple enough that a small proptest corpus (default 100 cases) provides adequate
coverage. The body.len()-4 formula is semantically equivalent across all valid inputs.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-031 added to fill SPB framing VP gap (ADR-009 rev 6 / M-2); formula corrected rev 9 / Decision 22 / F-H2 / F-H3 | draft |
| F4 (TDD implementation) | proptest harness authored and confirmed against corrected formula | draft |
| F6 (formal hardening) | Existing proptest confirmed correct against `body.len()-4` formula; 0 failures | draft → verified |

Lock: `status: verified`, `verification_lock: true` set by state-manager after F6 confirmation
@ develop 1ca30a3 (PRs #293 + #294).
