---
artifact: verification-property
vp_id: VP-029
title: "pcapng Block-Walk Skip Correctness and Forward Progress"
status: verified
phase: P1
tool: proptest
subsystem: SS-01
module: "reader.rs"
producer: architect
timestamp: 2026-06-19T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-009-pcapng-reader-design.md
feature_cycle: feature-pcapng-reader
source_bc: BC-2.01.015
bcs:
  - BC-2.01.015
verification_lock: true
verified_at_commit: "1ca30a3"
verified_prs: "#293, #294"
---

# VP-029: pcapng Block-Walk Skip Correctness and Forward Progress

## Property Statement

The pcapng block-walk skip logic in `src/reader.rs` satisfies the following properties
for **all valid block-type and length sequences**:

1. **Skip counter exactness:** The number of skipped blocks equals exactly the number
   of non-EPB/non-IDB/non-SHB block types encountered; the counter is incremented once
   per skip arm, never more.
2. **DSB no-log:** Decryption Secrets Blocks (DSB, block type 0x0A0D0D0A) are silently
   skipped without logging (intentional design — DSBs are out-of-scope for pcap metadata
   extraction).
3. **Termination:** The block-walk loop always terminates; it does not spin on
   zero-length or pathological block boundaries.
4. **Forward progress:** Every iteration of the block-walk loop advances the file cursor
   by at least the block-type (4 bytes) + block-length (4 bytes) minimum, preventing
   infinite loops on malformed inputs.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.01.015 | pcapng block-walk skip correctness: skip counter exactness; DSB no-log; termination guarantee | All harnesses |

## Proof Harnesses

```rust
proptest! {
    // VP-029: skip counter exactness + DSB silent-skip + termination.
    #[test]
    fn proptest_VP_029_skip_arm_counter_exactness_and_dsb_no_log(
        block_types in prop::collection::vec(any::<u32>(), 0..20),
        block_lengths in prop::collection::vec(8u32..1024, 0..20),
    ) { ... }
}
```

Suite confirmed at F6 lock @ develop 1ca30a3 (PRs #293 + #294).

## Feasibility Assessment

**Assessment: FEASIBLE (completed — proptest suite confirmed at F6 lock).**

The block-walk skip logic is a sequence-processing property over block type/length pairs.
proptest is the appropriate tool (state-machine over sequences); Kani is not well-suited
to the effectful block-walk integration path.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-029 designed, added to VP-INDEX (ADR-009 rev 4) | draft |
| F4 (TDD implementation) | proptest harnesses authored | draft |
| F6 (formal hardening) | proptest suite confirmed in CI; 0 failures at F6 lock | draft → verified |

Lock: `status: verified`, `verification_lock: true` set by state-manager after F6 confirmation
@ develop 1ca30a3 (PRs #293 + #294).
