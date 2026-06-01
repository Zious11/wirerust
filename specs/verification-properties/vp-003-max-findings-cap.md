---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.04.024
bcs:
  - BC-2.04.024
  - BC-2.04.054
module: src/reassembly/mod.rs
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: DF-SIBLING-SWEEP-001 ADV-IMPL-P03-HIGH-001 re-anchor: guard sites mod.rs:432,466,495 → mod.rs:461,495,524; finalize bypass mod.rs:573 → mod.rs:630. HS-043 merge shifted check_anomaly_thresholds by 29 lines and finalize() by 57 lines. — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-003: MAX_FINDINGS Cap with Finalize Bypass

## Property Statement

The reassembly engine's `findings: Vec<Finding>` satisfies:

1. At all points during `process_packet` execution (before `finalize` is called),
   `self.findings.len() <= MAX_FINDINGS` (where `MAX_FINDINGS = 10_000`).
2. After `finalize()` returns, `self.findings.len() <= MAX_FINDINGS + 1`. The
   `+1` accommodates the segment-limit summary finding that `finalize` pushes
   unconditionally (bypassing the cap guard), per BC-2.04.054.
3. Any finding that would exceed the cap during `process_packet` is silently
   dropped, and `ReassemblyStats.dropped_findings` is incremented by 1.
4. The finalize-bypass finding is the ONLY path that allows `len > MAX_FINDINGS`.

## Source Contract

- **Primary BC:** BC-2.04.024 -- Total findings capped at MAX_FINDINGS=10000; excess silently dropped
- **Postcondition:** `findings.len() <= MAX_FINDINGS` after any `process_packet` call
- **Invariant:** INV-6 (MAX_FINDINGS Cap with Cap-Bypass for Finalize, inv-01-core-invariants.md)
- **Related BC:** BC-2.04.054 -- finalize unconditionally bypasses MAX_FINDINGS cap for segment-limit finding

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- bounded number of push operations; unrolled guard checks | All 5 guard sites: 3 in mod.rs + 2 in lifecycle.rs; plus finalize bypass at mod.rs:630 |

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // Verify that after N push attempts, len never exceeds MAX_FINDINGS
    // (before finalize). Test the guard pattern directly.
    #[kani::proof]
    #[kani::unwind(10010)]  // MAX_FINDINGS + small margin
    fn verify_max_findings_cap_holds() {
        let mut findings: Vec<Finding> = Vec::new();
        let n: usize = kani::any();
        kani::assume(n <= 10_005); // bounded: slightly above MAX_FINDINGS

        for _ in 0..n {
            // Mimic the guard pattern at each emission site
            if findings.len() >= MAX_FINDINGS {
                // dropped_findings would increment here; we just break
                break;
            }
            findings.push(Finding::stub_for_test());
        }

        // Before finalize: must be at or below cap
        assert!(findings.len() <= MAX_FINDINGS);
    }

    // Verify finalize may push exactly one extra
    #[kani::proof]
    fn verify_finalize_bypass_adds_at_most_one() {
        let mut findings: Vec<Finding> = Vec::new();
        // Fill to exactly MAX_FINDINGS
        for _ in 0..MAX_FINDINGS {
            findings.push(Finding::stub_for_test());
        }
        assert_eq!(findings.len(), MAX_FINDINGS);

        // Simulate finalize push (unconditional)
        findings.push(Finding::stub_for_test()); // segment-limit summary

        // Post-finalize: len == MAX_FINDINGS + 1 is the maximum possible
        assert_eq!(findings.len(), MAX_FINDINGS + 1);
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | MAX_FINDINGS = 10,000 is a fixed constant; Kani unwind bound is tractable |
| Proof complexity | Medium | Must trace all 5 guard sites: mod.rs:461,495,524 + lifecycle.rs:101,121; finalize bypass at mod.rs:630 |
| Tool support | High | Guard pattern is simple `if len >= MAX_FINDINGS { return; }` (lifecycle.rs) and `if len < MAX_FINDINGS { push }` (mod.rs) |
| Estimated proof time | 5-15 minutes | Unwind bound of ~10,005 is large but guard sites are simple |

## Source Location

`src/reassembly/mod.rs:54` -- `const MAX_FINDINGS: usize = 10_000;` (private const, not pub)

Guard sites in `mod.rs` (check_anomaly_thresholds): `mod.rs:461, 495, 524`
  -- pattern: `if self.findings.len() < MAX_FINDINGS { push } else { dropped_findings += 1 }`

Guard sites in `lifecycle.rs` (generate_*_finding): `lifecycle.rs:101, 121`
  -- pattern: `if self.findings.len() >= MAX_FINDINGS { dropped_findings += 1; return; }`

Finalize unconditional push (bypasses cap): `mod.rs:630`
  -- pushes segment-limit summary finding without a guard; this is the ONLY bypass path.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| v1.1: Re-anchor guard sites + finalize bypass (DF-SIBLING-SWEEP-001 ADV-IMPL-P03-HIGH-001) | 2026-06-01 | product-owner |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
