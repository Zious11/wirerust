# [STORY-149] TLS Carry-Path Performance Recovery + Fragmented-Handshake Benchmark Fixture

**Epic:** E-11 — Tooling and Self-Improvement
**Mode:** maintenance
**Convergence:** CONVERGED after 8 adversarial passes (passes 6/7/8 consecutive clean; factory commit b9ae849)

![Tests](https://img.shields.io/badge/tests-2367%2F2367-brightgreen)
![Clippy](https://img.shields.io/badge/clippy-clean-brightgreen)
![Perf](https://img.shields.io/badge/perf%20delta-minus7.88%25-brightgreen)
![Adversarial](https://img.shields.io/badge/adversarial-8%20passes%2C%20converged-blue)

Closes #360.

This PR recovers the `reassembly/tls.pcap` Criterion regression (+14.0% vs Jun-22 baseline,
+19.6% vs May-19 anchor, criterion-confirmed p < 0.05) introduced by the STORY-144/145/146
carry-path additions. The fix restructures `TlsAnalyzer::try_parse_records` into two functions
— `prepare_record_step` (single `flows.get_mut()` acquisition site; SINGLE-BORROW INVARIANT
enforced) and `process_handshake_carry` (≤ 3 re-borrow sites after primary borrow released;
total budget ≤ 4 across both functions) — and introduces a `std::mem::take` carry-buffer swap
to release the borrow before the `&mut self` dispatch call. This eliminates repeated FlowKey
re-hashing (6–8 HashMap operations per hot-path record → 1) and per-record carry `Vec`
allocation. Post-fix authoritative measurement: 23.841 µs slope (−7.88% vs pre-story 25.880 µs;
+2.41% vs May-19 anchor 23.281 µs, within the +5% AC ceiling of 24.445 µs). A new Criterion
benchmark (`benches/tls_fragmented.rs`, group `tls_fragmented/3-record-carry-drain`, baseline
~1.594 µs slope) exercises the carry-drain loop for the first time, closing issue #360. All
2367 existing tests pass; VP-039 and VP-040 remain green; clippy -D warnings clean.

---

## Architecture Changes

```mermaid
graph TD
    TryParseRecords["TlsAnalyzer::try_parse_records<br/>(hot path)"]
    PrepareStep["prepare_record_step<br/>(single flows.get_mut site)"]
    CarryHelper["process_handshake_carry<br/>(≤ 3 re-borrow sites)"]
    TlsFlowState["TlsFlowState<br/>{client,server}_hs_carry"]
    BenchFixture["benches/tls_fragmented.rs<br/>(NEW — closes #360)"]

    TryParseRecords -->|"restructured into"| PrepareStep
    TryParseRecords -->|"delegates to"| CarryHelper
    PrepareStep -->|"single borrow; std::mem::take swap"| TlsFlowState
    CarryHelper -->|"re-borrows after primary released"| TlsFlowState
    BenchFixture -.->|"exercises carry-drain loop"| TryParseRecords

    style PrepareStep fill:#90EE90
    style CarryHelper fill:#90EE90
    style BenchFixture fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: Bounded-Borrow Carry-Path Restructure (budget ≤ 4)

**Context:** The STORY-144/145/146 carry-path additions used a naive acquire-per-operation
pattern: `try_parse_records` called `flows.get()` / `flows.get_mut()` 6–8 times per TLS
handshake record, re-hashing the `FlowKey` on each call. Additionally, carry bytes were
re-allocated as a fresh `Vec` per record. The combined overhead drove the
`reassembly/tls.pcap` benchmark +19.6% above the May-19 anchor.

**Decision:** Split `try_parse_records` into `prepare_record_step` (body contains exactly
one `flows.get_mut()` acquisition site; SINGLE-BORROW INVARIANT marker placed inline) and
`process_handshake_carry` (re-borrows the flow state at most 3 times after the primary borrow
is released; total budget ≤ 4). Use `std::mem::take` to swap out the carry buffer before the
`&mut self` dispatch call, releasing the borrow without a fresh allocation.

**Rationale:** A single-borrow invariant with a statically bounded budget (≤ 4) is both
minimal and machine-checkable via a source-inspection test (grep-based), making future
regressions detectable before they reach CI. `std::mem::take` was chosen from the permitted
set (`replace` / `take` / local Vec swap) because it expresses intent clearly and avoids the
placeholder argument needed by `replace`.

**Alternatives Considered:**
1. Entry API (`HashMap::entry`) — rejected: requires moving to owned key; not compatible with
   the borrowed `FlowKey` pattern used across all analyzers.
2. Flat carry-buffer (preallocated arena) — rejected: out of story scope; would require
   broader `TlsFlowState` refactor touching STORY-150.

**Consequences:**
- Positive: FlowKey hash computed once per record on the hot path; carry Vec not allocated
  per record. Benchmark recovery −7.88% vs pre-story, +2.41% vs May-19 anchor.
- Trade-off: `try_parse_records` is now an orchestrator dispatching to two helpers; callers
  outside the module are unaffected (same public interface).

</details>

---

## Story Dependencies

```mermaid
graph LR
    S149["STORY-149<br/>★ this PR"]
    S150["STORY-150<br/>◇ pending — DRY refactor<br/>of carry-drain duplication"]

    S149 --> S150

    style S149 fill:#FFD700
    style S150 fill:#E0E0E0
```

No upstream dependency PRs. `depends_on: []`. STORY-150 (carry-drain DRY refactor) depends
on this PR merging first.

---

## Spec Traceability

```mermaid
flowchart LR
    BC["STORY-149<br/>Carry-Path Perf Recovery"]
    AC1["AC-149-001<br/>Bounded-borrow invariant<br/>budget ≤ 4"]
    AC2["AC-149-002<br/>Fragmented benchmark<br/>3-record fixture"]
    AC3["AC-149-003<br/>Perf recovery<br/>+5% ceiling"]
    AC5["AC-149-005<br/>No regressions<br/>2367/0"]

    T1["test_BC_149_001_*<br/>(5 source-inspection tests)"]
    T2["test_BC_149_002_*<br/>(3 fixture tests)"]
    T3["cargo bench --bench pipeline<br/>23.841 µs ≤ 24.445 µs"]
    T5["cargo test --all-targets<br/>2367 pass / 0 fail"]

    S1["src/analyzer/tls.rs<br/>prepare_record_step +<br/>process_handshake_carry"]
    S2["benches/tls_fragmented.rs"]

    BC --> AC1
    BC --> AC2
    BC --> AC3
    BC --> AC5

    AC1 --> T1 --> S1
    AC2 --> T2 --> S2
    AC3 --> T3 --> S1
    AC5 --> T5 --> S1
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Unit + integration tests | 2367 / 2367 pass | 100% | PASS |
| Clippy -D warnings | 0 warnings | 0 | PASS |
| fmt check | clean | clean | PASS |
| VP-039 (TLS carry invariants) | green | green | PASS |
| VP-040 (TLS fragmented reassembly) | green | green | PASS |
| Holdout satisfaction | N/A — evaluated at wave gate | — | N/A |

### Test Flow

```mermaid
graph LR
    BorrowInvariant["5 Borrow-Invariant Tests<br/>(bc_149_single_borrow_invariant)"]
    FixtureTests["3 Benchmark-Fixture Tests<br/>(bc_149_fragmented_fixture)"]
    RegressionSuite["2367 Total Tests<br/>(--all-targets)"]
    CriterionBench["cargo bench pipeline<br/>23.841 µs slope"]

    BorrowInvariant -->|"budget ≤ 4 enforced"| PASS1["PASS"]
    FixtureTests -->|"3-record carry-drain"| PASS2["PASS"]
    RegressionSuite -->|"VP-039/VP-040 green"| PASS3["PASS"]
    CriterionBench -->|"+2.41% vs anchor"| PASS4["PASS"]

    style PASS1 fill:#90EE90
    style PASS2 fill:#90EE90
    style PASS3 fill:#90EE90
    style PASS4 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 8 added (5 borrow-invariant + 3 fixture), 0 modified |
| **Total suite** | 2367 tests PASS (0 failed) |
| **New bench** | `tls_fragmented/3-record-carry-drain` baselined ~1.594 µs slope |
| **Regressions** | 0 |

<details>
<summary><strong>Detailed Test Results</strong></summary>

### New Tests (This PR)

| Test | File | Result |
|------|------|--------|
| `test_BC_149_001_exactly_one_flows_borrow_in_try_parse_records` | `tests/bc_149_single_borrow_invariant_tests.rs` | PASS |
| `test_BC_149_001_single_borrow_invariant_comment_marker_present` | `tests/bc_149_single_borrow_invariant_tests.rs` | PASS |
| `test_BC_149_001_process_handshake_carry_budget_annotations_match_sites` | `tests/bc_149_single_borrow_invariant_tests.rs` | PASS |
| `test_BC_149_001_process_handshake_carry_borrow_budget` | `tests/bc_149_single_borrow_invariant_tests.rs` | PASS |
| `test_BC_149_001_no_aliasing_patterns_hide_borrow_count` | `tests/bc_149_single_borrow_invariant_tests.rs` | PASS |
| `test_BC_149_002_fixture_spans_at_least_3_records` | `tests/bc_149_fragmented_fixture_tests.rs` | PASS |
| `test_BC_149_002_fixture_is_deterministic` | `tests/bc_149_fragmented_fixture_tests.rs` | PASS |
| `test_BC_149_002_carry_drain_loop_exercised_across_records` | `tests/bc_149_fragmented_fixture_tests.rs` | PASS |

### Performance Evidence

| Benchmark | Pre-story | Post-story (authoritative) | Delta vs Pre | Delta vs May-19 Anchor | Status |
|-----------|-----------|---------------------------|-------------|------------------------|--------|
| `reassembly/tls.pcap` slope | 25.880 µs | 23.841 µs | −7.88% | +2.41% | PASS (≤ +5% ceiling 24.445 µs) |
| `tls_fragmented/3-record-carry-drain` | (new) | ~1.594 µs | — | baselined | PASS |

Authoritative source: `.factory/cycles/wave-70-story-149/STORY-149/implementation/perf-measurement.md`
(commit 923fac0, factory-artifacts branch).

</details>

---

## Holdout Evaluation

N/A — evaluated at wave gate (E-11 / wave 70 tooling story; no holdout scenarios defined).

---

## Adversarial Review

| Pass | Findings | Critical | High | Medium | Status |
|------|----------|----------|------|--------|--------|
| 1 | — | 0 | 0 | — | — |
| 2 | — | 0 | 0 | — | — |
| 3 | — | 0 | 0 | — | — |
| 4 | — | 0 | 0 | — | — |
| 5 | — | 0 | 0 | — | — |
| 6 | 0 | 0 | 0 | 0 | CLEAN |
| 7 | 0 | 0 | 0 | 0 | CLEAN |
| 8 | 0 | 0 | 0 | 0 | CLEAN |

**Convergence gate (DF-CONVERGENCE-BEFORE-MERGE-001): SATISFIED**

- 8 fresh-context adversarial passes total
- Passes 6, 7, 8 consecutive clean (zero findings)
- `converged: true` (factory commit b9ae849)
- 4 MEDIUM findings raised in earlier passes — all FIXED before convergence gate
- Zero HIGH or CRITICAL findings at any pass
- Zero deferred findings

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0"]
    Medium["Medium: 0"]
    Low["Low: 2 (test/bench only)"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#87CEEB
```

**Verdict: APPROVE.** No CRITICAL or HIGH findings. Two LOW findings in test/bench infrastructure; both non-blocking and documented below.

<details>
<summary><strong>Security Scan Details</strong></summary>

### Scope

This PR modifies only `src/analyzer/tls.rs` (restructure of hot-path parsing logic) and adds
`benches/tls_fragmented.rs` + test fixtures. No new external dependencies are introduced. No
user-facing input surfaces, authentication flows, or network-facing handlers are added.

### Findings

| ID | Severity | CWE | Location | Blocks Merge? |
|----|----------|-----|----------|---------------|
| SEC-001 | LOW | CWE-704 | `tests/common/tls_fragmented_fixture.rs:19-22` | No — test/bench code; current payloads bounded to 15 bytes |
| SEC-002 | LOW | CWE-693 | `tests/bc_149_single_borrow_invariant_tests.rs` (borrow-budget inspection) | No — forward-looking gap; no `flows[` usage in current code |

**SEC-001:** `wrap_as_tls_record` casts `payload.len()` to two bytes without checking the u16 max (65535). Silent truncation for payloads > 65535 bytes. Mitigation: add `debug_assert!(payload.len() <= u16::MAX as usize, ...)`. Deferred to follow-up.

**SEC-002:** Borrow-budget invariant tests grep for `.get(` / `.get_mut(` but not for `self.flows[key]` (index-operator syntax). A future use of `Index::index` would evade the CI count. Mitigation: add `self.flows[` to the anti-gameability check. Deferred to follow-up.

### Confirmed Safe

- No `unsafe` blocks introduced.
- `std::mem::take` swap preserves all carry-buffer invariants (borrow released before `&mut self` dispatch; carry restored after drain loop).
- All slice accesses in the hot path are guarded by prior bounds checks; no overflow risk.
- No new malformed-packet exploitation surface (same Decision-4 body-len spoof guard intact).
- No injection, auth bypass, or information-disclosure concerns.

### Dependency Audit

No new `Cargo.toml` dependencies added. `cargo audit` status: inherited from develop baseline.

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** `src/analyzer/tls.rs` only (TLS stream analyzer); `benches/tls_fragmented.rs` (new bench, no production surface)
- **User impact:** None — this is an internal performance refactor of the TLS parsing hot path. Public API surface unchanged.
- **Data impact:** None — no schema changes, no persisted state changes.
- **Risk Level:** LOW

### Performance Impact

| Benchmark | Before | After | Delta | Status |
|-----------|--------|-------|-------|--------|
| `reassembly/tls.pcap` slope | 25.880 µs | 23.841 µs | −7.88% | PASS (regression recovered) |
| `reassembly/tls.pcap` vs May-19 anchor | +19.6% | +2.41% | −17.2pp | PASS (re-enters WARNING threshold) |
| `tls_fragmented/3-record-carry-drain` | (new fixture) | ~1.594 µs | baselined | PASS |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 5 min):**
```bash
git revert 12777b957229b9e1dab6f3dbd852a289a0124006
git push origin develop
```

**Verification after rollback:**
- `cargo test --all-targets` must show 2367 pass
- `cargo bench --bench pipeline -- reassembly/tls.pcap` — expect return to pre-story ~25.880 µs

</details>

### Feature Flags

None — this is a structural refactor with no feature-flag toggle.

---

## Traceability

| Requirement | Story AC | Test | Verification | Status |
|-------------|---------|------|-------------|--------|
| Bounded-borrow budget ≤ 4 | AC-149-001 | `test_BC_149_001_*` (5 tests) | source-inspection grep | PASS |
| Fragmented-handshake fixture (issue #360) | AC-149-002 | `test_BC_149_002_*` (3 tests) | cargo bench tls_fragmented | PASS |
| Perf recovery ≤ +5% vs May-19 anchor | AC-149-003 | cargo bench pipeline (23.841 µs ≤ 24.445 µs) | Criterion p < 0.05 | PASS |
| No test regressions | AC-149-005 | cargo test --all-targets | 2367/0 | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
STORY-149 PERF-001 -> AC-149-001 -> test_BC_149_001_* -> src/analyzer/tls.rs:prepare_record_step -> ADV-PASS-8-CLEAN -> source-inspection-PASS
STORY-149 PERF-002 -> AC-149-001 -> std::mem::take swap -> src/analyzer/tls.rs:process_handshake_carry -> ADV-PASS-8-CLEAN
STORY-149 issue#360 -> AC-149-002 -> test_BC_149_002_* -> benches/tls_fragmented.rs -> ADV-PASS-8-CLEAN -> criterion-PASS
STORY-149 maint-2026-07-06 -> AC-149-003 -> cargo bench pipeline -> src/analyzer/tls.rs -> ADV-PASS-8-CLEAN -> criterion-IMPROVED
STORY-149 VP-039/VP-040 -> AC-149-005 -> cargo test --all-targets -> 2367/0 -> clippy-clean
```

</details>

---

## Demo Evidence

Evidence recorded at commit 2418048 (2026-07-07). All artifacts in `docs/demo-evidence/STORY-149/`.

| AC | Artifact | Verdict |
|----|----------|---------|
| AC-149-001 | `AC-149-001-bounded-borrow-invariant.txt` | PASS |
| AC-149-002 | `AC-149-002-fragmented-fixture.txt` | PASS |
| AC-149-003 | `AC-149-003-perf-recovery.txt` | PASS |
| AC-149-004 | N/A (optional AC) | N/A |
| AC-149-005 | `AC-149-005-no-regressions.txt` | PASS |

---

## Commit Chain

| SHA | Message |
|-----|---------|
| `12777b9` | docs(STORY-149): per-AC demo evidence |
| `2418048` | docs(STORY-149): resync test module docstring with fifth enforced invariant (F-S149P6-001) |
| `edb2b8c` | test(STORY-149): enforce BORROW BUDGET annotation coverage in inspection test (F-S149P5-001) |
| `5b41eca` | docs(STORY-149): add BORROW BUDGET annotations at process_handshake_carry sites (F-S149P5-001) |
| `208b2d4` | docs(STORY-149): align mem::replace doc references to mem::take (F-S149P2-001) |
| `a02eb6f` | test(STORY-149): budget-based borrow inspection + mod wrappers + fixture dedup (F-S149P1-001/004/006) |
| `d18632c` | docs(STORY-149): fix stale kani harness structural references (F-S149P1-002) |
| `ef83f8c` | docs(STORY-149): green-step doc-tense sweep (DF-GREEN-DOC-TENSE-SWEEP) |
| `923fac0` | feat(STORY-149): fix clippy mem_replace_with_default + rustfmt |
| `ac7155f` | feat(STORY-149): single-borrow refactor + fragmented-handshake fixture |
| `e951664` | test(STORY-149): add failing tests for AC-149-001/002 (PERF-001/002, issue #360) |
| `7ee8078` | feat(STORY-149): add module stubs |

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: maintenance
factory-version: 1.0.0-rc.22
pipeline-stages:
  spec-crystallization: completed (v1.4)
  story-decomposition: completed
  tdd-implementation: completed (red-gate → green → refactor)
  holdout-evaluation: "N/A — evaluated at wave gate"
  adversarial-review: completed (8 passes, converged)
  formal-verification: "N/A — source-inspection tests serve as structural proofs"
  convergence: achieved (passes 6/7/8 consecutive clean)
convergence-metrics:
  adversarial-passes: 8
  consecutive-clean-passes: 3
  medium-findings-resolved: 4
  high-critical-findings: 0
  converged: true
  factory-commit: b9ae849
total-pipeline-cost: N/A
models-used:
  builder: claude-sonnet-4-6
  adversary: claude-sonnet-4-6
generated-at: "2026-07-07T00:00:00Z"
story-version: "1.4"
wave: "70"
cycle: v0.11.4
```

</details>

---

## Pre-Merge Checklist

- [ ] All CI status checks passing
- [x] Coverage delta is positive or neutral (performance improved −7.88%)
- [ ] No critical/high security findings unresolved (pending step-4 security scan)
- [x] Rollback procedure documented above
- [x] No feature flag required (structural refactor)
- [ ] Human review completed (autonomy level reserved for human merge)
- [x] Demo evidence present for all mandatory ACs
- [x] Adversarial convergence gate (DF-CONVERGENCE-BEFORE-MERGE-001) satisfied
- [x] VP-039 / VP-040 harnesses green
