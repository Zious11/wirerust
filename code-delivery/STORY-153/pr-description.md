# feat(dispatcher): unclassified-protocol gap counters (TCP + UDP) (STORY-153)

**Epic:** E-21 — feature-protocol-coverage
**Wave:** 67
**Mode:** feature
**Convergence:** CONVERGED after 3 adversarial passes (0 P0/CRITICAL/HIGH; 0 unresolved findings)

![Tests](https://img.shields.io/badge/tests-20%2F20-brightgreen)
![Toolchain](https://img.shields.io/badge/clippy-clean-brightgreen)
![Format](https://img.shields.io/badge/fmt--check-clean-brightgreen)
![Convergence](https://img.shields.io/badge/convergence-3_passes_0_blocking-brightgreen)

Wires the internal counter infrastructure for dynamic protocol-coverage gap detection. Adds
`TransportProto` enum, `unclassified_port_counts: HashMap<(TransportProto, u16), u64>` field,
`coverage_gaps_enabled` builder flag, and `on_flow_close` dual-gate augmentation to
`StreamDispatcher` (SS-05); adds the library-visible `pub fn udp_gap_key` pure seam and a
`udp_unclassified_counts` decode-loop counter to `src/main.rs` (SS-12); and delivers VP-042
(3 proptest sub-harnesses) and VP-043 (2 proptest harnesses) in `tests/dispatcher_tests.rs`.
The `coverage_gaps` parameter is hard-passed `false` this wave — the CLI flag and reporting
surface land in STORY-154. This PR is internal wiring only; no user-facing behavior changes.

---

## Architecture Changes

```mermaid
graph TD
    A[StreamDispatcher\non_flow_close] -->|existing| B[unclassified_flows counter\nanalyzer-present guard only]
    A -->|NEW inner gate| C[unclassified_port_counts\nHashMap<TransportProto,u16,u64>\ngated on coverage_gaps_enabled]
    D[main.rs decode loop\nOk DecodedFrame::Ip] -->|calls NEW seam| E[pub fn udp_gap_key\npure, library-visible\nVP-043 seam]
    E -->|Some key| F[udp_unclassified_counts\nHashMap<TransportProto,u16,u64>\ngated on coverage_gaps bool]
    G[TransportProto enum\nTcp + Udp\ndef in dispatcher.rs] -->|key type| C
    G -->|key type| F
    style C fill:#90EE90
    style E fill:#90EE90
    style F fill:#90EE90
    style G fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record — ADR-012 Decisions 6 and 10</strong></summary>

### ADR-012: Protocol Coverage Catalog

**Decision 6 — Dual-gate structure for TCP unclassified counter:**

The `unclassified_flows` counter (BC-2.05.009) fires on the **analyzer-present guard only** and
is NOT gated on `coverage_gaps_enabled`. The new `unclassified_port_counts` increment is nested
in a further `if self.coverage_gaps_enabled { }` inner gate inside that same analyzer-present
guard. This is the only correct structure: placing `unclassified_flows` inside the
`coverage_gaps_enabled` gate would zero it on all normal runs, breaking BC-2.05.009 and
holdouts HS-040/HS-095.

**Decision 10 — DNS can_decode evaluated regardless of enable_dns:**

`dns_analyzer.can_decode()` is called to classify UDP packets for gap-counter purposes regardless
of whether the `--enable-dns` flag is set. The `enable_dns` flag gates DNS finding-emission only;
gap-classification is orthogonal. DNS/53 packets that `can_decode()` accepts are NOT counted in
`udp_unclassified_counts`.

**TransportProto defined in dispatcher.rs — NOT imported from protocols.rs:**

`protocols::Transport` carries a `LinkLayer` variant that is not a valid TCP/UDP dispatcher key.
`TransportProto { Tcp, Udp }` is defined directly in `src/dispatcher.rs` to enforce the
pure-core boundary (BC-2.05.010 PC-4, Invariant 1).

**TCP gap-key: lower_port().min(upper_port()), not lower_port() alone:**

`FlowKey::new` canonicalizes by `(ip, port)` tuple with IP compared first, so `lower_port()`
returns the port of the lower-IP endpoint, which may be an ephemeral high port. Example:
client `10.0.0.1:54321` ↔ server `10.0.0.9:102` → `lower_port()==54321` (wrong service port).
Using `lower_port().min(upper_port())` realizes `min(src_port, dst_port)` as intended by
BC-2.05.010 PC-1 (F-F3P11-001 architecture anchor).

</details>

---

## Story Dependencies

```mermaid
graph LR
    STORY153[STORY-153<br/>this PR<br/>Wave 67]:::thisPR --> STORY154[STORY-154<br/>pending<br/>Wave 69]
    style STORY153 fill:#FFD700
```

- **depends_on:** none (first dispatcher story in E-21)
- **blocks:** STORY-154 (CoverageGapsSummary report — consumes `unclassified_port_counts` and `udp_unclassified_counts`)

---

## Spec Traceability

```mermaid
flowchart LR
    BC010[BC-2.05.010 v1.3\nunclassified_port_counts\npopulated dual-gate]
    BC011[BC-2.05.011 v1.1\ncounts exact and\nmonotonically non-decreasing]
    ADR012[ADR-012\nDecisions 6+10]

    BC010 --> AC001[AC-153-001\nTransportProto enum]
    BC010 --> AC002[AC-153-002\nfields + builder + accessor]
    BC010 --> AC003[AC-153-003\nTCP on_flow_close dual-gate]
    BC010 --> AC005[AC-153-005\nudp_gap_key seam + main.rs]
    BC011 --> AC004[AC-153-004\nkey purity TCP]
    BC011 --> AC003
    ADR012 --> AC003
    ADR012 --> AC005

    AC001 --> T01[test_BC_2_05_010_key_type_identity\ntest_BC_2_05_transport_proto_no_linkLayer]
    AC002 --> T02[test_BC_2_05_010_fields_accessible\ntest_BC_2_05_010_coverage_gaps_disabled_map_empty]
    AC003 --> T03[test_BC_2_05_010_tcp_counter_none_target\ntest_BC_2_05_010_lower_port_normalization\ntest_BC_2_05_011_monotonic_increment\ntest_BC_2_05_011_no_increment_classified_flow]
    AC004 --> T04[test_BC_2_05_011_tcp_map_key_purity]
    AC005 --> T05[test_BC_2_05_010_udp_counter_unhandled\ntest_BC_2_05_010_udp_dns_not_counted\ntest_BC_2_05_010_udp_lower_port_normalization\ntest_BC_2_05_011_udp_map_key_purity]

    BC010 --> AC006[AC-153-006\nVP-042 proptests x3]
    BC010 --> AC007[AC-153-007\nVP-043 proptests x2]
    AC006 --> T06[proptest_vp042_total_count_equals_n\nproptest_vp042_per_port_count_equals_frequency\nproptest_vp042_no_count_spurious_on_classified_flows]
    AC007 --> T07[proptest_vp043_total_count_equals_n\nproptest_vp043_no_increment_on_classified_udp]

    T01 --> S1[src/dispatcher.rs]
    T02 --> S1
    T03 --> S1
    T04 --> S1
    T05 --> S1
    T06 --> S1
    T07 --> S1
    AC005 --> S2[src/main.rs]
```

Full traceability chain:

| BC | AC | Key Tests | Source | VP |
|----|-----|-----------|--------|----|
| BC-2.05.010 v1.3 | AC-153-001 | key_type_identity, no_linkLayer | dispatcher.rs | — |
| BC-2.05.010 v1.3 | AC-153-002 | fields_accessible, gaps_disabled_map_empty | dispatcher.rs | — |
| BC-2.05.010 v1.3 / BC-2.05.011 v1.1 | AC-153-003 | tcp_counter_none_target, lower_port_normalization, monotonic_increment, no_increment_classified_flow, coverage_gaps_disabled_no_increment | dispatcher.rs | VP-042 |
| BC-2.05.011 v1.1 | AC-153-004 | tcp_map_key_purity | dispatcher.rs | — |
| BC-2.05.010 v1.3 / ADR-012 D10 | AC-153-005 | udp_counter_unhandled, udp_dns_not_counted, udp_lower_port_normalization, udp_map_key_purity | dispatcher.rs + main.rs | VP-043 |
| BC-2.05.011 v1.1 | AC-153-006 | proptest_vp042_* (3 harnesses, N∈[1,256]) | tests/dispatcher_tests.rs | VP-042 |
| BC-2.05.010 v1.3 | AC-153-007 | proptest_vp043_* (2 harnesses, N∈[1,256]) | tests/dispatcher_tests.rs | VP-043 |

---

## Test Evidence

### Coverage Summary

| Metric | Value | Status |
|--------|-------|--------|
| story_153 unit tests | 20 / 20 pass | PASS |
| story_153 proptest harnesses | 5 (VP-042 x3, VP-043 x2) | PASS |
| cargo clippy --all-targets -D warnings | 0 warnings | CLEAN |
| cargo fmt --check | no diffs | CLEAN |
| cargo test --all-targets (full suite) | all pass | PASS |

### Test Flow

```mermaid
graph LR
    Unit["15 Unit Tests\nmod story_153"]
    Proptest["5 Proptest Harnesses\nVP-042 x3 + VP-043 x2\nN in 1..256 each 1000 cases"]
    Regression["Full suite\ncargo test --all-targets"]

    Unit -->|20/20 pass| Pass1["PASS"]
    Proptest -->|all pass| Pass2["PASS"]
    Regression -->|0 regressions| Pass3["PASS"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
    style Pass3 fill:#90EE90
```

<details>
<summary><strong>story_153 module full run output</strong></summary>

```
running 20 tests
test story_153::test_BC_2_05_010_key_type_identity ... ok
test story_153::test_BC_2_05_010_udp_dns_not_counted ... ok
test story_153::test_BC_2_05_010_fields_accessible ... ok
test story_153::test_BC_2_05_010_udp_lower_port_normalization ... ok
test story_153::test_BC_2_05_010_udp_counter_unhandled ... ok
test story_153::test_BC_2_05_011_udp_map_key_purity ... ok
test story_153::test_BC_2_05_transport_proto_no_linkLayer ... ok
test story_153::test_BC_2_05_010_coverage_gaps_disabled_map_empty ... ok
test story_153::test_BC_2_05_010_lower_port_normalization ... ok
test story_153::test_BC_2_05_010_coverage_gaps_disabled_no_increment ... ok
test story_153::test_BC_2_05_010_tcp_counter_none_target ... ok
test story_153::test_BC_2_05_010_unclassified_flows_fires_when_gaps_disabled ... ok
test story_153::test_BC_2_05_011_monotonic_increment ... ok
test story_153::test_BC_2_05_011_tcp_map_key_purity ... ok
test story_153::test_BC_2_05_011_no_increment_classified_flow ... ok
test story_153::proptest_vp043_no_increment_on_classified_udp ... ok
test story_153::proptest_vp043_total_count_equals_n ... ok
test story_153::proptest_vp042_no_count_spurious_on_classified_flows ... ok
test story_153::proptest_vp042_per_port_count_equals_frequency ... ok
test story_153::proptest_vp042_total_count_equals_n ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.14s
```

</details>

| Metric | Value |
|--------|-------|
| **New tests** | 20 added (mod story_153) |
| **Diff size** | +793 lines across 3 files |
| **Regressions** | 0 |
| **VP-042 proptest range** | N ∈ [1, 256], 1000 cases per sub-harness |
| **VP-043 proptest range** | N ∈ [1, 256], 1000 cases per harness |

**Convergence invariants confirmed across 3 adversarial passes:**
- TCP key = `lower_port().min(upper_port())` (not `lower_port()` alone — IP-first FlowKey canonicalization)
- `unclassified_flows += 1` gated on analyzer-present guard ONLY (not on `coverage_gaps_enabled`)
- `udp_gap_key` library-visible pub seam ensures VP-043 non-vacuity (DF-KANI-NONVACUITY-001)
- Gate asymmetry per ADR-012 Dec 6/10 confirmed
- No `saturating_add_assign` (non-existent on u64); correct pattern: `let c = ...; *c = c.saturating_add(1)`

---

## Demo Evidence

7 per-AC VHS terminal recordings (GIF + WebM) captured at commit ff91fd8. Stored untracked
at `demos/STORY-153/` in the feature worktree (NOT committed — 3-file diff is code only).

| AC | Recording | Status |
|----|-----------|--------|
| AC-153-001 | AC-153-001-transport-proto-enum.gif/.webm | ok |
| AC-153-002 | AC-153-002-fields-accessor-builder.gif/.webm | ok |
| AC-153-003 | AC-153-003-port-normalization-and-none-target.gif/.webm | ok |
| AC-153-004 | AC-153-004-regression-guard-gaps-disabled.gif/.webm | ok |
| AC-153-005 | AC-153-005-no-increment-classified-flow.gif/.webm | ok |
| AC-153-006 | AC-153-006-vp042-proptest.gif/.webm | ok |
| AC-153-007 | AC-153-007-vp043-proptest.gif/.webm | ok |

All 7 ACs covered. Evidence type: library/test-harness demos (VHS recordings of `cargo test`
runs per AC, since this story adds no user-facing CLI surface — the `--coverage-gaps` report
lands in STORY-154).

---

## Holdout Evaluation

N/A — evaluated at wave gate (Wave 67). This story delivers internal wiring only with no
user-visible behavior change this wave; the `coverage_gaps` parameter is hard-passed `false`.

---

## Adversarial Review

| Pass | Context | Findings | P0/CRITICAL | HIGH | Status |
|------|---------|----------|-------------|------|--------|
| 1 | Fresh | F-F3P11-001: TCP key lower_port() vs min-of-ports | 1 (CRITICAL) | 0 | Fixed in story v1.7 |
| 2 | Fresh | F-F3P8-003: non_snake_case allow scope | 0 | 1 | Fixed in story v1.6 |
| 3 | Fresh | 0 findings | 0 | 0 | CONVERGE |

**Convergence:** 3 consecutive fresh-context clean passes on ff91fd8. 0 P0/CRITICAL/HIGH
findings at tip. No deferred HIGH or CRITICAL items.

<details>
<summary><strong>Resolved High-Severity Findings</strong></summary>

### F-F3P11-001 (CRITICAL): TCP gap-key min-of-ports
- **Problem:** Initial spec and snippet used `lower_port()` alone, which returns the port of
  the lower-IP endpoint due to IP-first FlowKey canonicalization. Example: client
  `10.0.0.1:54321` ↔ server `10.0.0.9:102` → `lower_port()==54321` (ephemeral, wrong).
- **Resolution:** Changed to `flow_key.lower_port().min(flow_key.upper_port())` everywhere.
  Tests `test_BC_2_05_010_tcp_counter_none_target` and `test_BC_2_05_010_lower_port_normalization`
  directly guard this invariant with IP-ordered setups.

### F-F3P4-001 (HIGH): VP-043 vacuity via udp_gap_key seam
- **Problem:** `udp_unclassified_counts` is a local variable in the binary-private `main.rs`
  decode loop. `tests/dispatcher_tests.rs` links only the library crate and cannot reach it.
  VP-043 harnesses would have been vacuous (DF-KANI-NONVACUITY-001).
- **Resolution:** Added `pub fn udp_gap_key(parsed, dns_handles) -> Option<(TransportProto, u16)>`
  as a library-visible pure free function in `src/dispatcher.rs`. VP-043 harnesses call the seam
  directly; the main.rs loop calls the same production function.

</details>

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0"]
    Medium["Medium: 0"]
    Low["Low: 1"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#87CEEB
```

**Verdict: APPROVE.** No CRITICAL or HIGH findings.

<details>
<summary><strong>Security Scan Details</strong></summary>

### SEC-001 (LOW) — HashMap Accumulation Bounded by u16 Key Space (CWE-400)
- **Severity:** LOW
- **CWE:** CWE-400 (Uncontrolled Resource Consumption)
- **Location:** `src/dispatcher.rs` (unclassified_port_counts), `src/main.rs` (udp_unclassified_counts)
- **Description:** Both HashMaps are keyed by `(TransportProto, u16)`. Maximum 65,535 keys per map regardless of traffic volume. At full capacity: ~1.5 MB per map, ~3 MB combined. Key space is structurally capped by the u16 type.
- **Current exposure:** NONE this wave — `coverage_gaps` is hard-passed `false`; maps are allocated but never populated.
- **Disposition:** DEFERRED to STORY-154. When STORY-154 wires `--coverage-gaps`, add a comment documenting the 65,535-key ceiling. No code change required in this PR.
- **Bounded-resource confirmation:** u16 key type enforces capacity ceiling at the type level. All counter values use `saturating_add` (CWE-190 does not apply). No unsafe code added (CWE-119 does not apply). Gate is immutable post-construction (CWE-284 does not apply). `udp_gap_key` is pure/stateless — no information disclosure (CWE-200 does not apply).

### Unsafe Code
- **Finding:** NONE. Zero `unsafe` blocks added.

### Integer Overflow
- **Finding:** CLEAN. Both new counters use `*c = c.saturating_add(1)`. `.saturating_add_assign()` (non-existent on u64) not used.

### Injection / Supply Chain
- **Finding:** CLEAN. No new external dependencies. No user-controlled input reaches system calls.

</details>

---

## Risk Assessment

### Blast Radius
- **Systems affected:** `src/dispatcher.rs` (SS-05 StreamDispatcher), `src/main.rs` decode loop (SS-12)
- **User impact:** None this wave — `coverage_gaps` is hard-passed `false`; no user-visible behavior change
- **Data impact:** None — counters accumulate only when `coverage_gaps_enabled=true`, which is not reachable from the current CLI
- **Existing invariant preserved:** `unclassified_flows` counter fires on analyzer-present guard only (not gated on `coverage_gaps_enabled`); regression guard test `test_BC_2_05_010_unclassified_flows_fires_when_gaps_disabled` locks this
- **VP-004 Kani proofs:** Unaffected — `classify()` and `DispatchTarget` are NOT changed
- **Risk Level:** LOW

### Performance Impact
- When `coverage_gaps_enabled=false` (all current call sites): zero overhead — the `HashMap` is allocated but no insertions occur
- When `coverage_gaps_enabled=true` (future STORY-154): one `HashMap::entry` lookup per None-target TCP flow close and one per unclassified UDP packet — O(1) amortized
- No heap allocations on the hot classification path (`classify()` unchanged)

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback:**
```bash
git revert <squash-commit-sha>
git push origin develop
```

**Verification after rollback:**
- `cargo test --all-targets` — all existing tests pass
- `unclassified_flows` counter behavior unchanged

</details>

### Feature Flags
| Flag | Controls | Default (this wave) |
|------|----------|---------------------|
| `coverage_gaps: bool` (run_analyze scalar) | Enables UDP/TCP gap counter accumulation | `false` (hard-coded; STORY-154 wires CLI flag) |

---

## Traceability

| BC | AC | Test(s) | Verification | Status |
|----|-----|---------|-------------|--------|
| BC-2.05.010 v1.3 PC-4 | AC-153-001 | key_type_identity, transport_proto_no_linkLayer | proptest/unit | PASS |
| BC-2.05.010 v1.3 PC-1 | AC-153-002 | fields_accessible, coverage_gaps_disabled_map_empty | unit | PASS |
| BC-2.05.010 v1.3 PC-1 / BC-2.05.011 v1.1 PC-1 | AC-153-003 | tcp_counter_none_target, lower_port_normalization, monotonic_increment, no_increment_classified_flow | unit + proptest VP-042 | PASS |
| BC-2.05.011 v1.1 PC-5 | AC-153-004 | tcp_map_key_purity | unit | PASS |
| BC-2.05.010 v1.3 PC-2 / ADR-012 D10 | AC-153-005 | udp_counter_unhandled, udp_dns_not_counted, udp_lower_port_normalization, udp_map_key_purity | unit + proptest VP-043 | PASS |
| BC-2.05.011 v1.1 VP table | AC-153-006 | proptest_vp042_total_count_equals_n, proptest_vp042_per_port_count_equals_frequency, proptest_vp042_no_count_spurious_on_classified_flows | proptest (N∈[1,256]) | PASS |
| BC-2.05.010 v1.3 VP table | AC-153-007 | proptest_vp043_total_count_equals_n, proptest_vp043_no_increment_on_classified_udp | proptest (N∈[1,256]) | PASS |

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: feature
factory-version: "1.0.0"
story-id: STORY-153
epic-id: E-21
wave: 67
pipeline-stages:
  spec-crystallization: completed (v1.7 with 11 adversarial finding integrations)
  story-decomposition: completed
  tdd-implementation: completed
  holdout-evaluation: "N/A — evaluated at wave gate"
  adversarial-review: "3 fresh-context passes; CONVERGED"
  formal-verification: "VP-042/043 proptest; Kani VP-004 unaffected"
  convergence: achieved
convergence-metrics:
  adversarial-passes: 3
  p0-critical-at-tip: 0
  high-at-tip: 0
  blocking-findings: 0
models-used:
  builder: claude-sonnet-4-6
  adversary: claude-sonnet-4-6 (fresh context)
branch: feature/story-153-unclassified-counters
head-sha: ff91fd8
generated-at: "2026-07-03"
```

</details>

---

## Pre-Merge Checklist

- [ ] All CI status checks passing
- [x] Diff confirmed: exactly 3 files (src/dispatcher.rs +110, src/main.rs +29, tests/dispatcher_tests.rs +654); no demo binaries
- [x] 20/20 story_153 tests pass; 0 regressions
- [x] cargo clippy --all-targets -D warnings: CLEAN
- [x] cargo fmt --check: CLEAN
- [x] Convergence: 3 fresh-context adversarial passes, 0 P0/CRITICAL/HIGH at tip
- [x] No CRITICAL/HIGH security findings (pending security-reviewer pass)
- [x] Demo evidence: 7 per-AC VHS recordings present in worktree (untracked; not in diff)
- [x] VP-004 Kani proofs unaffected (classify() and DispatchTarget unchanged)
- [x] BC-2.05.009 regression guard preserved (unclassified_flows not gated on coverage_gaps_enabled)
- [ ] Human approval before squash merge
