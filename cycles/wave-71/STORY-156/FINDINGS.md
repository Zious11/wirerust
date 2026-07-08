# STORY-156 Cross-Pass Findings Tracker

## Summary

| Finding | Pass | Severity | Status | Resolution Reference |
|---------|------|----------|--------|----------------------|
| F-156-P1-001 | 1 | MEDIUM | RESOLVED | Phantom test citation removed — STORY-156 v1.3 |
| F-156-P1-002 | 1 | MEDIUM | RESOLVED | Provenance corrected — STORY-156 v1.3 |
| F-156-P1-003 | 1 | LOW | RESOLVED | Placement notes fixed — STORY-156 v1.3 |
| F-156-P1-004 | 1 | LOW | DEFERRED [spec-coherence] | BC-2.16.016 PC-3 "exactly 11 keys" vs 13-key contract — spec-coherence pass required |
| F-156-P1-005 | 1 | - | OBSERVATION | Documented post-pass |
| F-156-P1-006 | 1 | - | OBSERVATION | Documented post-pass |
| F-156-P2-001 | 2 | MEDIUM | RESOLVED | Body wave header TBD resolved — STORY-156 v1.5 + commit a61950f |
| F-156-P2-002 | 2 | MEDIUM | RESOLVED | BC version cell updated — STORY-156 v1.5 + commit a61950f |
| F-156-P2-003 | 2 | MEDIUM | RESOLVED | --arp anchor mis-attribution corrected — STORY-156 v1.5 + commit a61950f |
| F-156-P2-004 | 2 | LOW | RESOLVED | STORY-156 v1.5 + commit a61950f |
| F-156-P2-005 | 2 | LOW | RESOLVED | STORY-156 v1.5 + commit a61950f |
| F-156-P2-006 | 2 | LOW | RESOLVED | STORY-156 v1.5 + commit a61950f |
| F-156-P3-001 | 3 | LOW | RESOLVED | Test-file line citations — ACCEPT-FIX, folded into v1.5 |
| F-156-P3-002 | 3 | LOW | ACCEPTED-AS-IS | status:draft = convention, no change needed |
| F-156-P4-001 | 4 | MEDIUM | REFUTED | Wrong-tree read: adversary accessed develop copy, not worktree a61950f (see Pass 4 details) |
| O-156-P5-001 | 5 | LOW | ACCEPTED-AS-IS | reassembly/mod.rs:57 citation accurate, no change needed |

## Pass 1 Details

**Date:** 2026-07-07
**Verdict:** NOT-CLEAN
**Classification:** HAS_SUBSTANTIVE
**Head Reviewed:** 7e4fe6d

### Findings

- F-156-P1-001 (MEDIUM): Phantom test citation — referenced a test that does not exist in the story's test file list.
- F-156-P1-002 (MEDIUM): Provenance gap — BC attribution missing correct source reference.
- F-156-P1-003 (LOW): Placement notes ambiguous.
- F-156-P1-004 (LOW): BC-2.16.016 PC-3 states "exactly 11 keys" but the 13-key contract is in effect. Deferred to spec-coherence pass (cannot resolve within story scope).
- F-156-P1-005, F-156-P1-006: Observations, documented.

### Remediation

STORY-156 v1.3 + BC-2.16.016 v1.1 remediated F-156-P1-001 through F-156-P1-003. F-156-P1-004 deferred.

## Pass 2 Details

**Date:** 2026-07-08
**Verdict:** NOT-CLEAN
**Classification:** HAS_SUBSTANTIVE
**Head Reviewed:** 7e4fe6d

### Findings

- F-156-P2-001 (MEDIUM): Body freshness — wave header still showed "TBD" placeholder.
- F-156-P2-002 (MEDIUM): BC version cell outdated relative to BC-2.16.016 v1.1.
- F-156-P2-003 (MEDIUM): --arp flag anchor mis-attributed to wrong subsection.
- F-156-P2-004, F-156-P2-005, F-156-P2-006 (LOW): Minor consistency issues.

### Remediation

STORY-156 v1.5 + worktree commit a61950f resolved all six findings.

## Pass 3 Details

**Date:** 2026-07-08
**Verdict:** CLEAN
**Classification:** NITPICK_ONLY
**Head Reviewed:** a61950f
**Streak:** 1

### Findings

- F-156-P3-001 (LOW): Test-file line citations slightly stale. Triaged ACCEPT-FIX; folded into STORY-156 v1.5 (already applied to worktree).
- F-156-P3-002 (LOW): status:draft label — this is standard convention for in-progress stories, no change needed.

### Outcome

First clean pass. Streak: 1/3.

## Pass 4 Details

**Date:** 2026-07-08
**Verdict:** CLEAN (after triage)
**Classification:** CLEAN-after-triage
**Head Reviewed:** a61950f
**Streak:** 2

### Finding and Refutation

- F-156-P4-001 (MEDIUM): Adversary flagged bc_2_16_016_arp_tests.rs:55 as using integer-style assertion rather than symbol-style.

**REFUTED.** Root cause: adversary read the `develop` branch baseline copy of tests/bc_2_16_016_arp_tests.rs:55, not the worktree copy fixed at commit a61950f. The worktree copy (the authoritative version for this review) correctly uses symbol-style assertions. Orchestrator verified the worktree file directly.

All other review axes were clean. Part A confirmed. Counted as CLEAN per wave-70 triaged-pass precedent (D-396).

### Process Observation

This was an execution miss of DF-ADVERSARY-CHECKOUT-GUARD-001: the guard was asserted in the dispatch preamble but the adversary's reads drifted to the develop branch. Mitigated by adding a mandatory tree-discipline preamble to all subsequent adversary dispatches for this wave.

### Outcome

Second clean pass. Streak: 2/3.

## Pass 5 Details

**Date:** 2026-07-08
**Verdict:** CLEAN
**Classification:** NITPICK_ONLY
**Head Reviewed:** a61950f
**Streak:** 3

### Findings

- O-156-P5-001 (LOW): reassembly/mod.rs:57 line citation. Adversary noted as an observation. Verified as accurate; accepted as-is, no change needed.

### Notes

Pass 4 refutation re-verified in this pass. Adversary confirmed the worktree file is correct. All substantive axes clean.

### Outcome

Third clean pass. **BC-5.39.001 satisfied. STORY-156 CONVERGED 2026-07-08. Streak: 3/3.**

## Deferred Items

### Spec-Coherence: BC-2.16.016 PC-3 Key Count

**Finding:** F-156-P1-004
**Category:** Spec contradiction
**Description:** BC-2.16.016 PC-3 currently states "exactly 11 keys" as a precondition, but the behavioral contract itself specifies a 13-key ARP packet structure. This is an internal spec contradiction that requires a dedicated spec-coherence pass to resolve (not in scope for STORY-156 implementation).
**Status:** DEFERRED — spec-coherence pass required
**Next Step:** Open a maintenance task to align BC-2.16.016 PC-3 with the 13-key reality, or document the 11-key interpretation explicitly with rationale.

## Process Observations

### Adversary Wrong-Tree Read (Pass 4)

**Observation:** DF-ADVERSARY-CHECKOUT-GUARD-001 guard was asserted in the dispatch preamble but the adversary's actual file reads drifted to the `develop` branch rather than the wave-71 worktree.

**Impact:** One spurious MEDIUM finding (F-156-P4-001) that was REFUTED after orchestrator verification.

**Mitigation Applied:** Mandatory tree-discipline preamble added to all subsequent adversary dispatches in wave-71. Preamble explicitly names the worktree path and commit SHA to be used for all file reads.

---

**Convergence Status:** CONVERGED. BC-5.39.001 satisfied 2026-07-08. Streak: 3/3. Step 5 demo recording dispatched.
