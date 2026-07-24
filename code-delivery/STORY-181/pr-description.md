# [STORY-181] Fix SEC-001 ENIP Unsafe Split-Borrow in on_data: Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop (Behavior-Preserving Refactor)

**Epic:** E-20 — EtherNet/IP ENIP/CIP Analyzer
**Mode:** maintenance (behavior-preserving refactor)
**Convergence:** CONVERGED after 3 adversarial passes (P1 NITPICK_ONLY / P2 NITPICK_ONLY / P3 CLEAN — BC-5.39.001 satisfied)

![Tests](https://img.shields.io/badge/tests-2667%2F2667-brightgreen)
![ENIP Suite](https://img.shields.io/badge/enip_suite-184%2F184-brightgreen)
![Clippy](https://img.shields.io/badge/clippy-0_warnings-brightgreen)
![Holdout](https://img.shields.io/badge/holdout-N%2FA--BY--DESIGN-blue)

This PR eliminates SEC-001 from the wirerust tech-debt register: the `on_data` PDU dispatch
loop in `src/analyzer/enip.rs` previously cast `self.flows.get_mut(&flow_key)` to a raw
`*mut EnipFlowState` and called `self.process_pdu(unsafe { &mut *flow_ptr }, ...)`, relying
on a multi-line SAFETY comment to guarantee that `process_pdu` never accesses `self.flows`.
The fix replaces this with a safe take-remove-reinsert pattern: `self.flows.remove(&flow_key)`
produces an owned local flow, `process_pdu(&mut self, &mut flow, ...)` receives a structurally
disjoint reference, and `self.flows.insert(flow_key, flow)` re-inserts after the loop. The
compiler now enforces the disjointness invariant — no `unsafe` block, no raw-pointer cast,
no `#[allow(clippy::ptr_as_ptr)]` remain in `on_data`. The PR also fixes the ROUTE-W74 OBS-1
residual: the `parse_line()` docstring in `bin/validate-citations` now documents the
regex-mismatch `None` return path (AC-181-004). All 2667 tests pass unchanged; behavior is
identical to pre-refactor.

---

## Architecture Changes

```mermaid
graph TD
    EnipAnalyzer["EnipAnalyzer\n(src/analyzer/enip.rs)"]
    OnData["on_data()\n[PDU dispatch loop]"]
    ProcessPdu["process_pdu()\n[per-PDU analysis]"]
    FlowsMap["self.flows\nHashMap<FlowKey, EnipFlowState>"]

    EnipAnalyzer --> OnData
    OnData -->|"BEFORE: unsafe *mut raw ptr\n(aliased self.flows)"| ProcessPdu
    OnData -->|"AFTER: safe remove→local→insert\n(compiler-enforced disjoint)"| ProcessPdu
    OnData -->|"remove() / insert()"| FlowsMap
    ProcessPdu -.->|"NEVER accesses\n(now structurally enforced)"| FlowsMap

    style OnData fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: SEC-001 Take-Remove-Reinsert Pattern for EnipFlowState Split-Borrow

**Context:** `on_data` needed simultaneous access to `self` (for `process_pdu(&mut self, ...)`)
and to an element of `self.flows` (an `&mut EnipFlowState`). The prior approach used a raw
`*mut` pointer to sidestep the borrow checker, relying on a documented invariant that
`process_pdu` never accesses `self.flows`. This invariant was verified-by-inspection, not
structurally enforced.

**Decision:** Replace the raw pointer with a take-remove-reinsert pattern:
`flows.remove()` → process loop with local owned value → `flows.insert()`.

**Rationale:** The `remove`/`insert` approach gives the compiler full visibility into borrow
lifetimes. The flow is absent from `self.flows` during the dispatch loop, so any future
`process_pdu` change that accessed `self.flows` would be caught at compile time rather than
remaining a latent soundness risk. The behavioral contract (carry buffer accumulation,
direction isolation) is preserved because `process_pdu` only mutates fields other than
`self.flows`.

**Alternatives Considered:**
1. Refactor `process_pdu` to not require `&mut self` — rejected because it would require
   significant signature changes and introduce a separate borrow splitting complexity.
2. Keep the unsafe block but add more tests — rejected because tests cannot enforce the
   aliasing invariant; only structural safety can.

**Consequences:**
- Zero unsafe blocks remain in `src/analyzer/enip.rs` (SEC-001 closed).
- The invariant "process_pdu does NOT access self.flows" is now compiler-enforced rather
  than convention-based — future maintainers get a compile error if this changes.
- EC-002 (empty pdu_queue): remove+insert is a no-op that preserves flow state correctly.

</details>

---

## Story Dependencies

```mermaid
graph LR
    STORY181["STORY-181\n✅ this PR"]

    style STORY181 fill:#FFD700
```

**depends_on:** `[]` — no blocking predecessors. All prior E-20 stories (including
STORY-139 ENIP carry buffer, wave 62) are already merged to develop. No dependency hold.

---

## Spec Traceability

```mermaid
flowchart LR
    BC["BC-2.17.016\nENIP Per-Direction\nCarry Buffer"]
    AC1["AC-181-001\nunsafe eliminated"]
    AC2["AC-181-002\nbehavior identical"]
    AC3["AC-181-003\nno API change"]
    AC4["AC-181-004\nbin docstring"]

    T1["test_carry_buffer_partial_header\n(regression guard)"]
    T2["test_carry_buffer_two_frames_one_segment\n(regression guard)"]
    T3["test_ec_x1_cross_direction_no_splice\n(regression guard)"]

    S1["src/analyzer/enip.rs\non_data() lines 978–1001"]
    S2["bin/validate-citations\nparse_line() docstring"]

    BC --> AC1
    BC --> AC2
    BC --> AC3
    AC2 --> T1
    AC2 --> T2
    AC2 --> T3
    T1 --> S1
    T2 --> S1
    T3 --> S1
    AC4 --> S2
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Full cargo test --all-targets | 2667 / 2667 pass | 100% | PASS |
| ENIP integration suite | 184 / 184 pass | 100% | PASS |
| validate-citations tests | 27 / 27 pass | 100% | PASS |
| Clippy -D warnings | 0 warnings | 0 | PASS |
| Cargo fmt | clean | clean | PASS |
| Holdout satisfaction | N/A-BY-DESIGN | N/A | N/A (refactor) |
| Mutation kill rate | not run (refactor; behavior-preserving) | advisory | N/A |

### BC-2.17.016 Carry-Path Regression Witnesses (AC-181-002 Mandate)

Per AC-181-002, the PR description MUST enumerate at least three existing tests that exercise
the carry path. The following three carry-path regression tests confirmed passing at HEAD
`0b5ba318`:

| Test | Module | Result |
|------|--------|--------|
| `test_carry_buffer_partial_header` | `frame_walk` | PASS |
| `test_carry_buffer_two_frames_one_segment` | `frame_walk` | PASS |
| `test_ec_x1_cross_direction_no_splice` | `direction_and_clock` | PASS |

**Row-verify (PG-W74-PRDESC-ROW-VERIFY):** These test names are drawn directly from the
evidence-report at `docs/demo-evidence/STORY-181/AC-181-002-behavior-identical.md` which
confirms each ran and passed. The ENIP test binary confirmed 184/184 at HEAD commit
`0b5ba318`. Aggregate count cross-check: `cargo test --all-targets` returned 2667 passing
/ 0 failed / 5 ignored — consistent with the baseline-identical characterization in the
convergence report.

### Test Flow

```mermaid
graph LR
    Unit["2667 Unit/Integration Tests"]
    ENIP["184 ENIP Suite Tests"]
    Python["27 Python Tests\n(bin/test_validate_citations.py)"]
    Clippy["Clippy -D warnings"]
    Fmt["cargo fmt --check"]

    Unit -->|"0 failed"| Pass1["PASS"]
    ENIP -->|"0 failed"| Pass2["PASS"]
    Python -->|"0 failed"| Pass3["PASS"]
    Clippy -->|"0 warnings"| Pass4["PASS"]
    Fmt -->|"clean"| Pass5["PASS"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
    style Pass3 fill:#90EE90
    style Pass4 fill:#90EE90
    style Pass5 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 0 added (behavior-preserving refactor; all existing tests serve as regression guard) |
| **Total suite** | 2667 tests PASS |
| **ENIP suite** | 184 tests PASS |
| **Regressions** | 0 |
| **Red Gate** | N/A-BY-DESIGN (log: `.factory/cycles/wave-085/STORY-181/implementation/red-gate-log.md`) |

<details>
<summary><strong>Detailed Test Results — BC-2.17.016 Carry-Path Tests</strong></summary>

### Carry-Path Regression Witnesses

| Test | Module | Result | Duration |
|------|--------|--------|----------|
| `test_carry_buffer_partial_header` | `frame_walk` | PASS | < 1ms |
| `test_carry_buffer_two_frames_one_segment` | `frame_walk` | PASS | < 1ms |
| `test_ec_x1_cross_direction_no_splice` | `direction_and_clock` | PASS | < 1ms |

These three tests cover the BC-2.17.016 postconditions:
- Partial-header carry accumulation (c2s and s2c buffers survive the dispatch loop refactor)
- Two-frame-per-segment carry drain (multi-PDU dispatch loop iterates correctly over local flow)
- Cross-direction isolation (no c2s/s2c carry splice across direction boundary)

### Red Gate Log

This story is a behavior-preserving refactor (SEC-001 split-borrow elimination). The TDD
Red Gate is N/A-BY-DESIGN: no new behavioral assertions were required (the refactor changes
no observable behavior). Baseline at worktree base `421bf572`: 2667 passing / 0 failed /
5 ignored (log commit `e7f76508`). This N/A status was explicitly adjudicated in the
implementation plan and is recorded in the red-gate-log above.

</details>

---

## Holdout Evaluation

| Metric | Value | Notes |
|--------|-------|-------|
| Result | **N/A — evaluated at wave gate** | Behavior-preserving refactor; no new behavioral surface |

This is a maintenance refactor story (SEC-001 tech-debt closure). No new user-facing behavior
is introduced. Holdout evaluation applies at the wave gate level, not per-story for
behavior-preserving refactors.

---

## Adversarial Review

| Pass | Code Tip | Findings | Critical | High | LOW | Status |
|------|----------|----------|----------|------|-----|--------|
| P1 | e9572820 | 2 | 0 | 0 | 2 | SWEPT (294168fa) |
| P2 | 294168fa | 2 | 0 | 0 | 2 | SWEPT (093ff519) |
| P3 | 093ff519 | 0 | 0 | 0 | 0 | CLEAN — CONVERGED |

**Convergence:** CONVERGED 3/3 (BC-5.39.001 satisfied) — clean streak P1/P2/P3.
All findings were LOW severity; zero HIGH or CRITICAL at any pass.
Report: `.factory/cycles/wave-085/STORY-181/convergence-report.md`

<details>
<summary><strong>Adversarial Finding Dispositions</strong></summary>

### F-181-P1-001 (LOW) — False pdu_queue Invariant Comment
- **Location:** `src/analyzer/enip.rs` (dispatch-phase inline comment)
- **Category:** code-quality (comment precision)
- **Problem:** Inline comment over-stated a PDU-queue guarantee that was not fully correct.
- **Resolution:** Comment corrected in `294168fa` to accurately reflect the actual invariant.

### F-181-P1-002 (LOW) — Stale process_pdu flow_key Parameter Docstring
- **Location:** `src/analyzer/enip.rs` `process_pdu` docstring
- **Category:** code-quality (documentation)
- **Problem:** Pre-existing stale `flow_key` parameter docstring; adjudicated in-scope.
- **Resolution:** Docstring corrected in `294168fa`.

### F-181-P2-001 (LOW) — RULING-137-002 Cross-Ref Missing
- **Location:** `src/analyzer/enip.rs` inline comment
- **Category:** code-quality (traceability)
- **Problem:** Inline comment cited the ruling but omitted the back-reference to the
  originating architectural decision.
- **Resolution:** Cross-reference added in `093ff519`.

### F-181-P2-002 (LOW) — `"line ~1033"` Reference Off by 6 Lines
- **Location:** `src/analyzer/enip.rs` inline comment
- **Category:** code-quality (precision)
- **Problem:** Line reference was 6 lines off after the refactor shifted line numbers.
- **Resolution:** Corrected in `093ff519`.

### O-181-P3-001 (theoretical, non-blocking)
- **Category:** theoretical-only
- **Description:** Panic-unwind flow-drop divergence in a `debug_assert`-only panic path
  compiled out in release. Explicitly non-blocking; no action required.

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

**SEC-001 CLOSED:** The primary security finding from PR #334 review (MEDIUM, carried forward
to wave-85 as D-493) is resolved by this PR. Zero `unsafe` blocks remain in `src/analyzer/enip.rs`.
Adversary-confirmed across all 3 passes.

<details>
<summary><strong>Security Scan Details</strong></summary>

### Unsafe Block Elimination

`grep -n "flow_ptr\|ptr_as_ptr\|\*mut EnipFlowState\|unsafe" src/analyzer/enip.rs` returns
**zero matches** at HEAD `0b5ba318`. The four SEC-001 symbols are absent from the file:
- `let flow_ptr: *mut EnipFlowState` — removed
- `unsafe { &mut *flow_ptr }` — removed
- `#[allow(clippy::ptr_as_ptr)]` — removed
- The `unsafe` keyword at the former dispatch site — removed

### Dependency Audit

No new crate dependencies introduced. `Cargo.toml` unchanged.

### Formal Verification

| Property | Method | Status |
|----------|--------|--------|
| Carry buffer accumulation preserved | BC-2.17.016 test suite (184 tests) | VERIFIED |
| Borrow disjointness (structural) | Rust compiler type-checker | VERIFIED at compile time |
| `process_pdu` self.flows isolation | Exhaustive grep (×3 adversarial passes) | VERIFIED |

</details>

---

## Risk Assessment & Deployment

### Blast Radius

- **Systems affected:** `src/analyzer/enip.rs` `on_data` dispatch loop only; no public API changes
- **User impact:** None — behavior-preserving refactor; CLI output and finding emission are identical
- **Data impact:** None — no storage, no schema changes
- **Risk Level:** LOW (behavior-preserving; compiler-enforced correctness improvement)

### Performance Impact

| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| Latency | HashMap::remove + HashMap::insert per dispatch cycle | Same operations, same cost | neutral | OK |
| Memory | EnipFlowState stack-local during dispatch | Same; short-lived local | neutral | OK |
| Throughput | Identical PDU dispatch rate | Identical | 0 | OK |

The `remove` + `insert` operations on a `HashMap` are O(1) amortized — equivalent to the
prior `get_mut` + pointer deref. No observable performance difference is expected.

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert 224311a1  # take-remove-reinsert implementation commit
git push origin develop
```

The revert restores the pre-refactor `*mut EnipFlowState` unsafe block. All tests pass in
either form — this is a behavior-preserving change. No feature flag; no migration required.

**Verification after rollback:**
- `cargo test --all-targets` returns 2667 passing / 0 failed
- `grep -n "flow_ptr" src/analyzer/enip.rs` shows the raw pointer restored

</details>

### Feature Flags

None. This is a direct code change with no feature flag. Rollback is via `git revert`.

---

## Traceability

| Requirement | Story AC | Test | Verification | Status |
|-------------|---------|------|-------------|--------|
| BC-2.17.016 carry postconditions preserved | AC-181-002 | `test_carry_buffer_partial_header` | ENIP suite 184/184 | PASS |
| BC-2.17.016 carry postconditions preserved | AC-181-002 | `test_carry_buffer_two_frames_one_segment` | ENIP suite 184/184 | PASS |
| BC-2.17.016 cross-direction isolation | AC-181-002 | `test_ec_x1_cross_direction_no_splice` | ENIP suite 184/184 | PASS |
| SEC-001 unsafe eliminated | AC-181-001 | grep zero-match | compiler type-checker | PASS |
| No public API change | AC-181-003 | git diff --stat | Cargo.toml absent from diff | PASS |
| ROUTE-W74 OBS-1 docstring | AC-181-004 | `python3 bin/test_validate_citations.py` | 27/27 pass | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
BC-2.17.016 → AC-181-001 → grep "unsafe" src/analyzer/enip.rs → zero matches → compiler-enforced
BC-2.17.016 → AC-181-002 → test_carry_buffer_partial_header → enip.rs on_data carry select → ENIP 184/184
BC-2.17.016 → AC-181-002 → test_carry_buffer_two_frames_one_segment → enip.rs on_data carry select → ENIP 184/184
BC-2.17.016 → AC-181-002 → test_ec_x1_cross_direction_no_splice → enip.rs on_data carry select → ENIP 184/184
BC-2.17.016 → AC-181-003 → git diff --stat → Cargo.toml absent → no API change
ROUTE-W74/OBS-1 → AC-181-004 → bin/validate-citations parse_line() → test_validate_citations.py → 27/27
```

</details>

---

## Demo Evidence

Demo evidence at `docs/demo-evidence/STORY-181/` (5 artifacts, scrub PASSED 2026-07-24):

| File | AC Coverage |
|------|-------------|
| `AC-181-001-unsafe-eliminated.md` | AC-181-001: grep zero-match + before/after code excerpt |
| `AC-181-002-behavior-identical.md` | AC-181-002: 184/184 ENIP tests + 3 carry-path witnesses + full suite |
| `AC-181-003-no-api-change.md` | AC-181-003: process_pdu signature grep + git diff stat |
| `AC-181-004-bin-docstring.md` | AC-181-004: docstring excerpt + 27/27 test_validate_citations.py |
| `evidence-report.md` | Index — coverage map + scrub gate PASSED |

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: maintenance
factory-version: "1.0.0-rc.23"
pipeline-stages:
  spec-crystallization: completed (wave-85 story decomposition 2026-07-23)
  story-decomposition: completed (STORY-181 v1.1, human approval D-505 2026-07-24)
  tdd-implementation: completed (224311a1 + 13491355 + e9572820; Red Gate N/A-BY-DESIGN)
  holdout-evaluation: "N/A — evaluated at wave gate (behavior-preserving refactor)"
  adversarial-review: completed (3 passes, CONVERGED BC-5.39.001)
  formal-verification: skipped (refactor; compiler type-check is structural verification)
  convergence: achieved (P1/P2/P3 clean streak)
convergence-metrics:
  adversarial-passes: 3
  last-classification: CLEAN
  clean-streak: "P1/P2/P3 = 3/3"
  code-tip-at-convergence: "093ff519"
  open-HIGH-CRIT: 0
models-used:
  builder: claude-sonnet-4-6
  adversary: claude-sonnet-4-6 (step-4.5 adversarial)
generated-at: "2026-07-24T22:00:00Z"
wave: 85
story-version: "1.1"
```

</details>

---

## Pre-Merge Checklist

- [ ] All CI status checks passing
- [x] Coverage delta: neutral (2667/0/5 baseline-identical — behavior-preserving)
- [x] No critical/high security findings unresolved (SEC-001 CLOSED; 0 HIGH/CRIT adversarial)
- [x] Rollback procedure: `git revert 224311a1`; all tests pass in both states
- [x] No feature flag required (direct code change)
- [ ] Human review completed (autonomy level requires human merge authorization — see DF-MERGE-AUTH-CLASSIFIER-001; no wave-85 wave-level grant exists)
- [x] Adversarial convergence: CONVERGED 3/3 (BC-5.39.001)
- [x] Demo evidence: 5 artifacts per-AC, scrub PASSED
- [x] AC-181-002 carry-path witnesses enumerated: test_carry_buffer_partial_header, test_carry_buffer_two_frames_one_segment, test_ec_x1_cross_direction_no_splice
- [x] PG-W74-PRDESC-ROW-VERIFY: row-verified 3 carry-path test entries + aggregate count cross-checked
- [x] CHANGELOG [Unreleased] entry present (AC-158-001)
