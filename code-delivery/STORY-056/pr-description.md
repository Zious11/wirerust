# [STORY-056] SNI Classification Arms 3 and 4 — Non-ASCII UTF-8 and Non-UTF-8 Byte Preservation

**Epic:** E-5 — TLS Analyzer Formalization
**Mode:** brownfield-formalization (zero production behavior change; tests only)
**Convergence:** CONVERGED after 9 adversarial passes (BC-5.39.001 ACHIEVED — 3-clean P7/P8/P9)

![Tests](https://img.shields.io/badge/tests-888%2F888-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-tests--only-brightgreen)
![Mutation](https://img.shields.io/badge/mutation-N%2FA--test--only-green)
![Holdout](https://img.shields.io/badge/holdout-N%2FA--wave--gate-blue)

This PR delivers formalization tests for TLS SNI classification arms 3 (non-ASCII UTF-8) and 4
(non-UTF-8 bytes) in `src/analyzer/tls.rs`. It is a **TEST-ONLY diff** — zero production source
changes. The 9 new tests in `tests/tls_analyzer_tests.rs` (total suite now 99 TLS tests) formally
assert: `SniValue::NonAsciiUtf8` and `SniValue::NonUtf8` variants, T1027 finding fields
(category/verdict/confidence/summary/evidence/mitre_technique/direction) with exact string
matching, `sni_counts` key format `"<non-utf8:{hex}>"` for arm 4 with collision-avoidance,
raw-byte hex preservation per ADR 0003, and the `is_ascii()` gate that routes arm 2/3
(BC-2.07.037). Adversarial convergence: VP-005 arm-numbering legend fix, SS-07 sibling-BC
anchor sweep (BC-017/019/020 v1.3). Front-loaded exactness kept passes P1–P4 clean.

---

## Architecture Changes

```mermaid
graph TD
    A["tests/tls_analyzer_tests.rs<br/>(+9 new test fns, arm 3 & 4)"] -->|exercises| B["src/analyzer/tls.rs<br/>extract_sni arms 3+4"]
    B -->|returns| C["SniValue::NonAsciiUtf8 { hostname, hex }"]
    B -->|returns| D["SniValue::NonUtf8 { lossy, hex }"]
    E["handle_client_hello<br/>(arm 3 branch)"] -->|pushes| F["Finding: Anomaly/Inconclusive/Low<br/>T1027 + raw hostname"]
    G["handle_client_hello<br/>(arm 4 branch)"] -->|pushes| H["Finding: Anomaly/Inconclusive/Low<br/>T1027 + lossy + hex"]
    G -->|inserts| I["sni_counts key<br/>&lt;non-utf8:{hex}&gt;"]
    style A fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR Reference: ADR 0003 — Reporting Pipeline / Byte Preservation

**Context:** SNI hostnames containing non-ASCII or invalid UTF-8 bytes are evasion vectors
(MITRE T1027). The analyzer must emit lossless hex evidence while the terminal reporter
applies display escaping. This story formalizes the existing implementation against this contract.

**Decision:** Arm 3 (NonAsciiUtf8) preserves the decoded UTF-8 string verbatim in the finding
summary (raw Cyrillic, emoji, CJK intact). Arm 4 (NonUtf8) uses `String::from_utf8_lossy`
(U+FFFD replacements) in the summary and lossless lowercase hex in `evidence[0]`. The
`sni_counts` key for arm 4 is `"<non-utf8:{hex}>"` — not the lossy string — preventing
collision across distinct invalid byte sequences with identical lossy representations.

**Rationale:** ADR 0003 INV-4 mandates that escaping is applied only at the terminal reporter
layer. Hex evidence must be pure ASCII (0-9, a-f) to survive any transport/encoding layer.
The `"<non-utf8:{hex}>"` key format was chosen to guarantee uniqueness and collision-avoidance
in `sni_counts`.

**Alternatives Considered:**
1. Use lossy string as `sni_counts` key — rejected: collides for distinct byte sequences
   sharing the same U+FFFD replacement pattern
2. Apply `escape_for_terminal` in the analyzer — rejected: ADR 0003 INV-4 prohibition

**Consequences:**
- SOC operators searching `sni_counts` for "control" will miss arm 3 entries (arm 3 fires for
  non-ASCII even when C0 bytes are present — documented in EC-007)
- Hex evidence enables full forensic reconstruction of original byte sequence

</details>

---

## Story Dependencies

```mermaid
graph LR
    S055["STORY-055<br/>✅ MERGED (#151)<br/>SNI Arms 1 & 2"] --> S056["STORY-056<br/>🟡 this PR<br/>SNI Arms 3 & 4"]
    S056 --> S057["STORY-057<br/>⏳ pending<br/>SNI downstream"]
    style S056 fill:#FFD700
    style S055 fill:#90EE90
```

---

## Spec Traceability

```mermaid
flowchart LR
    BC017["BC-2.07.017<br/>Non-ASCII UTF-8 SNI<br/>Emits T1027"] --> AC001["AC-001<br/>arm 3 SniValue +<br/>finding fields"]
    BC017 --> AC002["AC-002<br/>raw UTF-8 in summary<br/>(no Debug escaping)"]
    BC017 --> AC003["AC-003<br/>emoji/CJK/Cyrillic<br/>all land arm 3"]
    BC019["BC-2.07.019<br/>Non-UTF-8 SNI<br/>Emits T1027 + hex key"] --> AC004["AC-004<br/>arm 4 SniValue +<br/>finding fields"]
    BC019 --> AC005["AC-005<br/>sni_counts key format<br/>&lt;non-utf8:hex&gt;"]
    BC020["BC-2.07.020<br/>Non-UTF-8 Raw Byte<br/>Preservation"] --> AC006["AC-006<br/>lossy summary +<br/>hex evidence"]
    BC020 --> AC007["AC-007<br/>hex is pure ASCII"]
    BC021["BC-2.07.021<br/>Non-ASCII UTF-8<br/>Raw Byte Preservation"] --> AC008["AC-008<br/>raw Cyrillic in summary<br/>(not escaped)"]
    BC037["BC-2.07.037<br/>is_ascii gate:<br/>arm 2 vs arm 3"] --> AC009["AC-009<br/>C0+non-ASCII fires<br/>arm 3, not arm 2"]
    BC037 --> AC010["AC-010<br/>is_ascii decisive gate<br/>top-down evaluation"]
    AC001 --> T1["test_valid_utf8_non_ascii_sni_emits_finding<br/>test_cyrillic_sni_emits_non_ascii_finding"]
    AC003 --> T2["test_emoji_sni_emits_non_ascii_finding"]
    AC004 --> T3["test_non_utf8_sni_emits_finding_and_counts_under_hex_key<br/>non_utf8_sni_finding_sets_mitre_t1027"]
    AC006 --> T4["test_non_utf8_sni_preserves_raw_bytes_in_summary"]
    AC007 --> T5["test_arm4_hex_evidence_is_pure_ascii"]
    AC009 --> T6["test_c0_plus_non_ascii_fires_arm3_not_arm2"]
    AC010 --> T7["test_is_ascii_gate_routes_arm2_vs_arm3"]
    T1 --> SRC["tests/tls_analyzer_tests.rs"]
    T2 --> SRC
    T3 --> SRC
    T4 --> SRC
    T5 --> SRC
    T6 --> SRC
    T7 --> SRC
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Unit tests | 888/888 pass | 100% | PASS |
| New TLS arm 3/4 tests | 9 added | — | PASS |
| ACs covered | 10/10 | 100% | PASS |
| Coverage delta | test-only (no src change) | neutral | PASS |
| Mutation kill rate | N/A (test-only diff) | — | N/A |
| Holdout satisfaction | N/A — evaluated at wave gate | >= 0.85 | N/A |

### Test Flow

```mermaid
graph LR
    Unit["99 TLS Unit Tests<br/>(+9 this PR)"]
    Suite["888 total tests<br/>all targets"]
    ACs["10/10 ACs covered<br/>5 BCs traced"]

    Unit -->|100% pass| Pass1["PASS"]
    Suite -->|100% pass| Pass2["PASS"]
    ACs -->|9 unique fns| Pass3["PASS"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
    style Pass3 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 9 added, 0 modified |
| **Total suite** | 888 tests PASS (cargo test --all-targets) |
| **Coverage delta** | Test-only diff; no src lines changed |
| **Mutation kill rate** | N/A (no production code changed) |
| **Regressions** | 0 |

<details>
<summary><strong>Detailed Test Results</strong></summary>

### New Tests (This PR)

| Test | AC | Result |
|------|----|--------|
| `test_valid_utf8_non_ascii_sni_emits_finding()` | AC-001 | PASS |
| `test_cyrillic_sni_emits_non_ascii_finding()` | AC-001, AC-002, AC-008 | PASS |
| `test_emoji_sni_emits_non_ascii_finding()` | AC-003 | PASS |
| `test_non_utf8_sni_emits_finding_and_counts_under_hex_key()` | AC-004, AC-005 | PASS |
| `non_utf8_sni_finding_sets_mitre_t1027()` | AC-004 | PASS |
| `test_non_utf8_sni_preserves_raw_bytes_in_summary()` | AC-006 | PASS |
| `test_arm4_hex_evidence_is_pure_ascii()` | AC-007 | PASS |
| `test_c0_plus_non_ascii_fires_arm3_not_arm2()` | AC-009 | PASS |
| `test_is_ascii_gate_routes_arm2_vs_arm3()` | AC-010 | PASS |

### Coverage Analysis

| Metric | Value |
|--------|-------|
| Lines added (src) | 0 (test-only diff) |
| Lines added (tests) | ~350 new test lines |
| Branches added | N/A |
| Uncovered paths | none — all 10 ACs covered |

</details>

---

## Holdout Evaluation

| Metric | Value | Threshold |
|--------|-------|-----------|
| Mean satisfaction | N/A — evaluated at wave gate | >= 0.85 |
| Std deviation | N/A | < 0.15 |
| Must-pass minimum | N/A | >= 0.6 |
| Scenarios evaluated | N/A | >= 5 |
| **Result** | **N/A — wave-gate evaluation** | |

---

## Adversarial Review

| Pass | Findings | Critical | High | Status |
|------|----------|----------|------|--------|
| P1 | 0 | 0 | 0 | CLEAN (front-loaded exactness) |
| P2 | 0 | 0 | 0 | CLEAN |
| P3 | 0 | 0 | 0 | CLEAN |
| P4 | 0 | 0 | 0 | CLEAN |
| P5 | 2 | 0 | 0 | Fixed (VP-005 arm legend, SS-07 anchor sweep) |
| P6 | 1 | 0 | 0 | Fixed (BC-017/019/020 v1.3 sibling-BC sync) |
| P7 | 0 | 0 | 0 | CLEAN |
| P8 | 0 | 0 | 0 | CLEAN |
| P9 | 0 | 0 | 0 | CLEAN — BC-5.39.001 ACHIEVED |

**Convergence:** 3-clean passes P7/P8/P9 achieved. BC-5.39.001 satisfied.

<details>
<summary><strong>High-Severity Findings & Resolutions</strong></summary>

### Finding P5-1: VP-005 Arm-Numbering Legend
- **Location:** Story spec / verification property
- **Category:** spec-fidelity
- **Problem:** VP-005 arm legend did not align with implementation arm numbering (1-indexed vs 0-indexed)
- **Resolution:** Updated VP-005 arm-numbering legend to match `extract_sni` match arm ordering

### Finding P5-2: SS-07 Sibling-BC Anchor Sweep
- **Location:** BC-2.07.017, BC-2.07.019, BC-2.07.020 spec files
- **Category:** spec-fidelity
- **Problem:** Sibling-BC cross-references missing or stale (SS-07 anchor sweep finding)
- **Resolution:** BC files updated to v1.3 with corrected sibling-BC anchors

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
<summary><strong>Security Scan Details</strong></summary>

### SAST
- This is a test-only diff. No production code changed.
- New test code constructs byte slices and calls existing analyzer functions.
- No unsafe code introduced. No new dependencies.
- Critical: 0 | High: 0 | Medium: 0 | Low: 0

### Dependency Audit
- `cargo audit`: CLEAN (no new dependencies added)

### Formal Verification

| Property | Method | Status |
|----------|--------|--------|
| is_ascii() gate correctness | Unit test (AC-010) | VERIFIED |
| sni_counts key collision-avoidance | Unit test (AC-005) | VERIFIED |
| hex field purity (0-9, a-f only) | Unit test (AC-007) | VERIFIED |
| No Debug escaping at analyzer layer | Unit test (AC-002, AC-008) | VERIFIED |

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** Test suite only (tests/tls_analyzer_tests.rs)
- **User impact:** None — zero production behavior change
- **Data impact:** None
- **Risk Level:** LOW (test-only diff)

### Performance Impact
| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| Test suite runtime | ~baseline | ~+0.05s | +9 new tests | OK |
| Production latency p99 | N/A | N/A | 0 (no src change) | OK |
| Memory | N/A | N/A | 0 (no src change) | OK |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert <MERGE_COMMIT_SHA>
git push origin develop
```

**Verification after rollback:**
- `cargo test --all-targets` returns to pre-STORY-056 test count
- TLS SNI arm 3/4 tests are absent from test output

</details>

### Feature Flags
| Flag | Controls | Default |
|------|----------|---------|
| N/A | Test-only story | N/A |

---

## Traceability

| Requirement | Story AC | Test | Verification | Status |
|-------------|---------|------|-------------|--------|
| BC-2.07.017 postcondition 1-3 | AC-001 | `test_valid_utf8_non_ascii_sni_emits_finding()`, `test_cyrillic_sni_emits_non_ascii_finding()` | Unit test | PASS |
| BC-2.07.017 invariant 1 (no Debug escaping) | AC-002 | `test_cyrillic_sni_emits_non_ascii_finding()` | Unit test | PASS |
| BC-2.07.017 invariant 3 (all non-ASCII -> arm 3) | AC-003 | `test_emoji_sni_emits_non_ascii_finding()` | Unit test | PASS |
| BC-2.07.019 postcondition 1-3 | AC-004 | `test_non_utf8_sni_emits_finding_and_counts_under_hex_key()`, `non_utf8_sni_finding_sets_mitre_t1027()` | Unit test | PASS |
| BC-2.07.019 invariant 1 (hex-tagged key) | AC-005 | `test_non_utf8_sni_emits_finding_and_counts_under_hex_key()` | Unit test | PASS |
| BC-2.07.020 postcondition 1-4 | AC-006 | `test_non_utf8_sni_preserves_raw_bytes_in_summary()` | Unit test | PASS |
| BC-2.07.020 invariant 1-2 (hex pure ASCII) | AC-007 | `test_arm4_hex_evidence_is_pure_ascii()` | Unit test | PASS |
| BC-2.07.021 postcondition 1-3 | AC-008 | `test_cyrillic_sni_emits_non_ascii_finding()` | Unit test | PASS |
| BC-2.07.037 postcondition 1-4 | AC-009 | `test_c0_plus_non_ascii_fires_arm3_not_arm2()` | Unit test | PASS |
| BC-2.07.037 invariant 1-2 (is_ascii gate) | AC-010 | `test_is_ascii_gate_routes_arm2_vs_arm3()` | Unit test | PASS |
| VP-005 arm-numbering legend | spec | corrected in story v1.2 | adversarial pass P5 | FIXED |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
BC-2.07.017 -> VP-005 -> AC-001 -> test_valid_utf8_non_ascii_sni_emits_finding -> src/analyzer/tls.rs:257-260 -> ADV-P1-CLEAN -> UNIT-PASS
BC-2.07.017 -> VP-005 -> AC-002 -> test_cyrillic_sni_emits_non_ascii_finding -> src/analyzer/tls.rs:449-467 -> ADV-P1-CLEAN -> UNIT-PASS
BC-2.07.017 -> VP-005 -> AC-003 -> test_emoji_sni_emits_non_ascii_finding -> src/analyzer/tls.rs:449-467 -> ADV-P1-CLEAN -> UNIT-PASS
BC-2.07.019 -> VP-005 -> AC-004 -> test_non_utf8_sni_emits_finding_and_counts_under_hex_key -> src/analyzer/tls.rs:469-488 -> ADV-P1-CLEAN -> UNIT-PASS
BC-2.07.019 -> VP-005 -> AC-005 -> test_non_utf8_sni_emits_finding_and_counts_under_hex_key -> src/analyzer/tls.rs:410-415 -> ADV-P1-CLEAN -> UNIT-PASS
BC-2.07.020 -> VP-005 -> AC-006 -> test_non_utf8_sni_preserves_raw_bytes_in_summary -> src/analyzer/tls.rs:469-488 -> ADV-P1-CLEAN -> UNIT-PASS
BC-2.07.020 -> VP-005 -> AC-007 -> test_arm4_hex_evidence_is_pure_ascii -> src/analyzer/tls.rs:86-93 -> ADV-P1-CLEAN -> UNIT-PASS
BC-2.07.021 -> VP-005 -> AC-008 -> test_cyrillic_sni_emits_non_ascii_finding -> src/analyzer/tls.rs:449-467 -> ADV-P5-FIXED -> UNIT-PASS
BC-2.07.037 -> VP-005 -> AC-009 -> test_c0_plus_non_ascii_fires_arm3_not_arm2 -> src/analyzer/tls.rs:449-467 -> ADV-P6-FIXED -> UNIT-PASS
BC-2.07.037 -> VP-005 -> AC-010 -> test_is_ascii_gate_routes_arm2_vs_arm3 -> src/analyzer/tls.rs:257-260 -> ADV-P7-CLEAN -> UNIT-PASS
```

</details>

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: brownfield-formalization
factory-version: "1.0.0-rc.18"
pipeline-stages:
  spec-crystallization: completed
  story-decomposition: completed
  tdd-implementation: completed (test-only)
  holdout-evaluation: "N/A — evaluated at wave gate"
  adversarial-review: completed (9 passes)
  formal-verification: "N/A — evaluated at Phase 5"
  convergence: achieved
convergence-metrics:
  spec-novelty: N/A
  test-kill-rate: "N/A (test-only diff)"
  implementation-ci: green (888/888)
  holdout-satisfaction: "N/A — wave gate"
  holdout-std-dev: N/A
adversarial-passes: 9
bc-convergence-criterion: "BC-5.39.001 (3-clean passes)"
total-pipeline-cost: estimated
models-used:
  builder: claude-sonnet-4-6
  adversary: claude-sonnet-4-6
  evaluator: claude-sonnet-4-6
wave: 18
epic: E-5
story-points: 8
generated-at: "2026-05-29T00:00:00Z"
```

</details>

---

## Pre-Merge Checklist

- [x] All CI status checks passing
- [x] Coverage delta is positive or neutral (test-only; neutral src coverage)
- [x] No critical/high security findings unresolved (0 findings; test-only diff)
- [x] Rollback procedure validated (git revert)
- [x] No feature flags required (test-only story)
- [x] Demo evidence complete — 10/10 ACs covered in docs/demo-evidence/STORY-056/evidence-report.md
- [x] Dependency PR STORY-055 (#151) MERGED
- [x] Adversarial convergence: 9 passes, BC-5.39.001 ACHIEVED
- [x] Autonomous merge authorized (Wave 15-17 cadence, Level 4)
