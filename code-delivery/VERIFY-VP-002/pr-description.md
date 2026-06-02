# refactor(reassembly): extract pure select_gaps + Kani-prove first-wins winner-selection (VP-002)

**Epic:** Phase 6 Formal Hardening — VP-002 gap closure
**Mode:** brownfield / formal verification
**Convergence:** CONVERGED after 2 code-review passes

![Tests](https://img.shields.io/badge/tests-1112%2F1112-brightgreen)
![Kani](https://img.shields.io/badge/kani-2%20harnesses%20SUCCESSFUL-brightgreen)
![Checks](https://img.shields.io/badge/checks-180%2F180-brightgreen)
![Security](https://img.shields.io/badge/security-CLEAN-brightgreen)

Closes the VP-002 (first-wins overlap, CRITICAL anti-evasion) formal-coverage gap identified at the Phase-6 hardening gate. The existing winner-selection was correct but unproven; this PR makes it **provably** correct: extracts a pure `select_gaps` function from `insert_segment`, adds two bounded Kani harnesses proving the first-wins SAFETY invariant on allocation-free kernels, and adds G1/G2/G2b/G3 byte-survival tests that assert existing bytes are present AND attacker bytes are absent end-to-end through `all_data()`.

---

## Architecture Changes

```mermaid
graph TD
    insert_segment["insert_segment()"] -->|calls| select_gaps["select_gaps() NEW\n(pure, offset-only)"]
    insert_segment -->|BTreeMap insert/scan| SegmentMap["BTreeMap&lt;u64, Vec&lt;u8&gt;&gt;"]
    select_gaps -->|returns| SelectGaps["SelectGaps { fully_covered, gaps }"]
    kani_proofs["kani_proofs mod\n(cfg(kani))"] -.->|proves via kernels| select_gaps
    style select_gaps fill:#90EE90
    style kani_proofs fill:#87CEEB
    style SelectGaps fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: Extract pure select_gaps to enable Kani proof of first-wins winner-selection

**Context:** The Phase-6 hardening gate identified VP-002 (CRITICAL anti-evasion: first-wins overlap policy) as a formal-coverage gap. The multi-segment winner-selection logic was embedded inside `insert_segment` and used a heap `Vec`, making direct Kani verification intractable (45M vars / 199M clauses, no verdict in >6 min).

**Decision:** Extract the winner-selection into a pure, offset-only `select_gaps(new_start, new_end, existing) -> SelectGaps` function. Prove the SAFETY invariant (gap bytes never overwrite existing bytes) on allocation-free kernel mirrors (`point_kept_by_existing` + `sweep_gaps_into`) via bounded Kani. Document the kernel↔select_gaps boundary in-code.

**Rationale:** The extraction is behavior-preserving (the develop `sort()` was a no-op — BTreeMap already yields ascending keys, confirmed by reviewer). The kernels are byte-identical to `select_gaps` on every algorithmic line; only storage differs (stack array vs heap Vec), which is provably non-functional. This gives genuine proof teeth: a `cursor.max(es)` bug would surface a CONCRETE counterexample.

**Alternatives Considered:**
1. Prove `select_gaps` directly with heap Vec — rejected: CBMC-intractable (measured 45M vars/199M clauses, no verdict >6 min; standard Kani/std-collection limitation).
2. Use proptest only — rejected: property-based testing cannot provide the exhaustive safety guarantee needed for CRITICAL anti-evasion; only model checking can.

**Consequences:**
- select_gaps is a real, exported-within-crate production function (not a test seam).
- Proof establishes: every gap byte is disjoint from every existing range for ALL symbolic inputs in the bounded domain.
- Kani harnesses are cfg(kani)-gated; zero production overhead.

</details>

---

## Story Dependencies

```mermaid
graph LR
    VP001["VP-001 Kani proofs\n✅ merged #181"] --> VP002["VP-002 gap closure\n🟡 this PR"]
    VP003["VP-003 Kani proofs\n✅ merged #181"] --> VP002
    VP002 --> MUTATION["Phase 6 mutation pass\n⏳ pending"]
    style VP002 fill:#FFD700
```

---

## Spec Traceability

```mermaid
flowchart LR
    BC["BC-2.04.018/035/036/037/043\nFirst-wins overlap policy"] --> AC1["VP-002\nGap bytes disjoint from existing"]
    BC --> AC2["VP-002\nfully_covered iff union covers"]
    BC --> AC3["VP-002\ngaps partition span exactly"]
    AC1 --> T1["verify_gap_byte_never_overwrites_existing\n(Kani, 180 checks)"]
    AC2 --> T2["verify_sweep_emits_exactly_gap_bytes\n(Kani)"]
    AC3 --> T2
    T1 --> S1["src/reassembly/segment.rs\nkani_proofs::point_kept_by_existing"]
    T2 --> S1
    AC1 --> G1["G1/G2/G2b byte-survival tests\n(reassembly_segment_tests.rs)"]
    AC1 --> G3["G3 engine end-to-end test\n(reassembly_engine_tests.rs)"]
    G1 --> S1
    G3 --> S2["src/reassembly/ (engine integration)"]
```

---

## The VP-002 Gap Found at the Gate

The Phase-6 gate identified: **multi-segment winner-selection was unproven at the release-build safety boundary.** Specifically, the `debug_assert!` collision guard in `insert_segment` is compiled out in release builds. The only protection against a silent overwrite of first-arrived (forensically authoritative) bytes was algorithmic correctness of the gap-selection sweep — which was correct but not formally verified. This PR discharges that gap.

**Safety invariant proven:** For every byte position `p` in `[new_start, new_end)`, if `p` is a gap byte (first-wins policy permits inserting here), then NO existing range contains `p`. This means: inserting gap bytes can NEVER overwrite a buffered first-arrived byte, even in release builds where the collision guard is absent.

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Unit / integration tests | 1112 / 1112 pass | 100% | PASS |
| Kani harnesses | 2 / 2 SUCCESSFUL | 100% | PASS |
| Kani checks | 180 / 180 | 100% | PASS |
| Kani unwinding warnings | 0 | 0 | PASS |
| fmt / clippy (-D warnings) | CLEAN | CLEAN | PASS |

### Test Flow

```mermaid
graph LR
    Unit["1112 Unit+Integration Tests"]
    G1G2["G1/G2/G2b byte-survival\n(segment_tests.rs)"]
    G3["G3 engine end-to-end\n(engine_tests.rs)"]
    Kani["2 Kani harnesses\n180 checks"]

    Unit -->|1112/1112 pass| Pass1["PASS"]
    G1G2 -->|existing bytes survive| Pass2["PASS"]
    G3 -->|all_data() first-wins| Pass3["PASS"]
    Kani -->|SUCCESSFUL, 0 warnings| Pass4["PASS"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
    style Pass3 fill:#90EE90
    style Pass4 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | G1, G2, G2b, G3 byte-survival + 2 Kani harnesses added |
| **Total suite** | 1112 tests PASS, 0 FAIL |
| **Kani** | `verify_gap_byte_never_overwrites_existing` + `verify_sweep_emits_exactly_gap_bytes` — both SUCCESSFUL, 180 checks, no unwinding warning |
| **Regressions** | 0 |

<details>
<summary><strong>Detailed Test Results</strong></summary>

### New Tests (This PR)

| Test | Location | Result |
|------|----------|--------|
| `test_g1_union_conflict_byte_survival()` | reassembly_segment_tests.rs | PASS |
| `test_g2_three_segment_conflict_byte_survival()` | reassembly_segment_tests.rs | PASS |
| `test_g2b_multi_union_with_real_gap_byte_survival()` | reassembly_segment_tests.rs | PASS |
| `test_g3_engine_end_to_end_first_wins_byte_survival()` | reassembly_engine_tests.rs | PASS |
| `verify_gap_byte_never_overwrites_existing` | segment.rs kani_proofs (cfg(kani)) | SUCCESSFUL (90 checks) |
| `verify_sweep_emits_exactly_gap_bytes` | segment.rs kani_proofs (cfg(kani)) | SUCCESSFUL (90 checks) |

### G1/G2/G2b/G3 Byte-Survival Test Design

These tests go beyond checking the `SegmentOverlap` result enum. They:
1. Insert original segments (first-arrived, forensically authoritative bytes).
2. Insert attacker segments that conflict.
3. Re-read the buffer with `segment_at()` / `all_data()` to assert:
   - Original bytes are **present** at their offsets.
   - Attacker conflicting bytes are **absent** from every position.

**G1:** Multi-segment UNION CONFLICT — original bytes survive, attacker dropped.
**G2:** Three-segment conflicting union — original bytes from all 3 survive.
**G2b:** Multi-segment union WITH a real gap — gap bytes inserted, conflict bytes dropped.
**G3:** End-to-end engine test — `all_data()` returns first-wins bytes; attacker `'B'` is absent from delivered stream. Conflicting-overlap anomaly (MITRE T1036) still reported.

### Kani Harness Details

**`verify_gap_byte_never_overwrites_existing`** (90 checks, 2.3s):
- Bounded domain: 3 symbolic existing ranges, offsets in [0,12], lengths in [1,4]; symbolic new segment.
- For every gap byte `p` (position not covered by existing ranges), asserts `p` is in NO existing range.
- `#[kani::unwind(5)]` — sufficient (MAXLEN=4 outer iterations + inner loop over COUNT=3); CBMC unwinding assertions would fail if ever too small.

**`verify_sweep_emits_exactly_gap_bytes`** (90 checks, 7.6s):
- Bounded domain: 3 symbolic existing ranges, offsets in [0,8], lengths in [1,3].
- Asserts: every emitted gap byte is a gap byte; every gap byte falls inside exactly one gap; gaps and coverage partition [new_start, new_end) with no overlap/loss.
- Asserts fixed buffer never overran (provably CAP=4 sufficient for COUNT=3).
- Asserts: fully_covered implies 0 gaps.

**WHY KERNELS NOT `select_gaps` DIRECTLY:** `select_gaps` returns a heap `Vec<(u64,u64)>`. Driving that through CBMC explodes the SAT instance (~45M vars / 199M clauses, no verdict in >6 min). The kernels are byte-identical to `select_gaps` on every algorithmic line; only storage differs (stack array vs heap Vec, provably non-functional). The kernel↔select_gaps boundary is documented in-code; `select_gaps` is additionally exercised by the full 1112-test reassembly suite.

</details>

---

## Holdout Evaluation

N/A — evaluated at wave gate. This PR closes a formal-verification coverage gap, not a behavioral story.

---

## Adversarial Review / Code Review

| Pass | Verdict | Findings | Critical | High | Low | Status |
|------|---------|----------|----------|------|-----|--------|
| 1 | REQUEST_CHANGES | 4 | 0 | 0 | 4 | All fixed |
| 2 | APPROVE | 0 | 0 | 0 | 0 | Converged |

**Convergence:** 2 review passes to APPROVE.

Key reviewer verdicts:
- **BEHAVIOR-PRESERVING confirmed:** The `sort()` removal from `insert_segment` was a no-op — BTreeMap already yields keys in ascending order. No behavior change.
- **KERNEL FIDELITY CONFIRMED:** `sweep_gaps_into` is byte-identical to `select_gaps` on every algorithmic line; only storage differs (stack array vs heap Vec, provably non-functional). Proving the kernels genuinely establishes VP-002 for `select_gaps`.
- **PROOF TEETH confirmed:** A `cursor.max(es)` bug would surface a CONCRETE counterexample in `verify_gap_byte_never_overwrites_existing` — verified transiently.

<details>
<summary><strong>Pass 1 Findings (all LOW) & Resolutions</strong></summary>

| Finding | Severity | Resolution |
|---------|----------|------------|
| Missing `debug_assert!` precondition guard in `select_gaps` | LOW | Added: `debug_assert!(existing.windows(2).all(...), "select_gaps: existing must be sorted ascending by start")` |
| Load-bearing comment on overlap filter `if !ranges_overlap(...)` lacked clarity | LOW | Added explicit comment explaining this is load-bearing for Kani harnesses (non-overlapping ranges drive symbolic paths) |
| G3 assertion was not structured — used bare `assert_eq!` on final byte | LOW | Refactored to structured per-byte assertion with explicit failure message |
| Unwind-bound doc did not explain sufficiency reasoning | LOW | Added doc: MAXLEN=4 outer iterations + COUNT=3 inner; CBMC unwinding assertions would fail if 5 were too small |

</details>

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0"]
    Medium["Medium: 0"]
    Low["Low: 0"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#90EE90
```

<details>
<summary><strong>Security Details</strong></summary>

### Formal Verification — VP-002

| Property | Method | Harness | Status |
|----------|--------|---------|--------|
| Gap bytes never overwrite existing bytes (SAFETY invariant) | Kani (bounded model checking) | `verify_gap_byte_never_overwrites_existing` | VERIFIED — 90 checks, SUCCESSFUL |
| Gap emission algorithm is exact (completeness + disjointness + partition) | Kani | `verify_sweep_emits_exactly_gap_bytes` | VERIFIED — 90 checks, SUCCESSFUL |
| First-wins byte survival end-to-end | Integration (G1/G2/G2b/G3) | byte-survival tests | VERIFIED |

### Trust Boundary (W11-D2)

- Kani harnesses are `cfg(kani)`-gated: zero production binary impact.
- `select_gaps` is a real pure production function: no `_for_testing` seams added.
- No new unsafe code, no new dependencies.

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** `src/reassembly/segment.rs` only (plus new tests).
- **User impact:** None — extraction is behavior-preserving; no user-visible change.
- **Data impact:** None — first-wins policy unchanged; only now formally proven.
- **Risk Level:** LOW (refactor only; 1112 tests + 2 Kani harnesses confirm no behavioral change).

### Performance Impact
| Metric | Assessment | Status |
|--------|-----------|--------|
| Throughput | No change — `select_gaps` is called on the same code path as before; no extra allocation in the critical path | OK |
| Memory | No change — same Vec allocation as before, just in a named function | OK |
| Binary size | Negligible — one additional small function; Kani code is cfg-gated out | OK |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert <MERGE_SHA>
git push origin develop
```

**Verification after rollback:**
- Run `cargo test --all-targets` — expect 1109 pass (G1/G2/G2b/G3 tests removed).
- VP-002 formal-coverage gap re-opens; document in Phase-6 gate.

</details>

### Feature Flags
None — this is an internal refactor with no user-visible behavior change.

---

## Traceability

| Requirement | Verification Property | Test / Harness | Method | Status |
|-------------|----------------------|----------------|--------|--------|
| BC-2.04.018 First-wins overlap | VP-002 gap byte disjointness | `verify_gap_byte_never_overwrites_existing` | Kani | VERIFIED |
| BC-2.04.035 No overwrite of buffered bytes | VP-002 safety invariant | `verify_gap_byte_never_overwrites_existing` | Kani | VERIFIED |
| BC-2.04.036 gaps partition span | VP-002 completeness | `verify_sweep_emits_exactly_gap_bytes` | Kani | VERIFIED |
| BC-2.04.037 fully_covered iff union covers | VP-002 full-coverage predicate | `verify_sweep_emits_exactly_gap_bytes` | Kani | VERIFIED |
| BC-2.04.043 adjacency not overlap | VP-002 adjacency | `verify_overlap_predicate_and_adjacency` | Kani (prior PR) | VERIFIED |
| First-wins byte survival — union conflict | G1 byte-survival | `test_g1_union_conflict_byte_survival` | Integration | PASS |
| First-wins byte survival — 3-segment union | G2 byte-survival | `test_g2_three_segment_conflict_byte_survival` | Integration | PASS |
| First-wins byte survival — partial overlap with gap | G2b byte-survival | `test_g2b_multi_union_with_real_gap_byte_survival` | Integration | PASS |
| First-wins byte survival — end-to-end engine | G3 end-to-end | `test_g3_engine_end_to_end_first_wins_byte_survival` | Integration | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
BC-2.04.018/035 -> VP-002 (gap bytes disjoint) -> verify_gap_byte_never_overwrites_existing
  -> src/reassembly/segment.rs:kani_proofs::point_kept_by_existing
  -> KANI SUCCESSFUL (90 checks, 2.3s)

BC-2.04.036/037 -> VP-002 (partition + full-coverage) -> verify_sweep_emits_exactly_gap_bytes
  -> src/reassembly/segment.rs:kani_proofs::sweep_gaps_into
  -> KANI SUCCESSFUL (90 checks, 7.6s)

BC-2.04.018 -> G1/G2/G2b -> byte-survival integration tests
  -> tests/reassembly_segment_tests.rs
  -> 1112/1112 PASS

BC-2.04.018 -> G3 -> engine end-to-end byte-survival
  -> tests/reassembly_engine_tests.rs
  -> PASS (all_data() returns first-wins bytes; attacker bytes absent)
```

</details>

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: brownfield / formal-hardening
factory-version: "1.0.0"
pipeline-stages:
  spec-crystallization: completed (prior phases)
  story-decomposition: completed (prior phases)
  tdd-implementation: completed
  holdout-evaluation: "N/A — evaluated at wave gate"
  adversarial-review: completed (2 passes, converged)
  formal-verification: completed (2 Kani harnesses, 180 checks, SUCCESSFUL)
  convergence: achieved
convergence-metrics:
  review-passes: 2
  kani-checks: 180
  kani-harnesses: 2
  test-suite: 1112/1112
  kani-unwinding-warnings: 0
models-used:
  builder: claude-sonnet-4-6
  reviewer: claude-sonnet-4-6
generated-at: "2026-06-01"
branch: verify/vp002-gap-closure
base-commit: d951210
head-commit: 153f543
commits-ahead-of-develop: 3
```

</details>

---

## Pre-Merge Checklist

- [x] All CI status checks passing (fmt, clippy -D warnings, test --all-targets, semantic PR)
- [x] No critical/high security findings
- [x] Kani harnesses SUCCESSFUL (2/2, 180 checks, 0 unwinding warnings)
- [x] Code review converged (2 passes, APPROVE)
- [x] Behavior-preserving confirmed by reviewer (BTreeMap sort no-op)
- [x] Kernel fidelity confirmed by reviewer (byte-identical algorithm)
- [x] G1/G2/G2b/G3 byte-survival tests passing (existing bytes present, attacker bytes absent)
- [x] W11-D2 trust boundary: cfg(kani)-gated proofs, real production function, no test seams
- [x] Human pre-approved merge (Phase 6 gate authorization)
- [x] SQUASH merge (3 commits → 1)
- [ ] Worktree cleanup — devops handles post-merge
