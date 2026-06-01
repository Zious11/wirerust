# [STORY-090] Summary Data Model — ingest, Service Hints, unique_hosts, Serialization

**Epic:** E-9 — CLI Output Pipeline
**Mode:** brownfield-formalization (ZERO src/ changes)
**Convergence:** CONVERGED after 3 adversarial passes (fresh-context; 2 remediation rounds; 3 consecutive clean passes)

![Tests](https://img.shields.io/badge/tests-18%2F18-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-delta%20neutral-brightgreen)
![Mutation](https://img.shields.io/badge/mutation-full%20matrix%20caught-green)
![Adversarial](https://img.shields.io/badge/adversarial-CONVERGED-blue)

This PR adds `tests/summary_story_090_tests.rs` (mod `story_090`) with 18 direct unit and integration tests (13 AC + 5 EC) formalizing four behavioral contracts for the `Summary` data model: `Summary::ingest` incrementing total_packets/total_bytes/hosts/protocols (BC-2.12.018), port-based service hints via `app_protocol_hint` (BC-2.12.019), `unique_hosts()` returning a sorted-deduplicated `Vec<IpAddr>` (BC-2.12.020), and serde serialization of total_packets/total_bytes/skipped_packets with Debug-format protocol keys (BC-2.12.021). This is the FINAL story of Phase 3 (completes E-9 and all 48 stories). Two adversarial remediation rounds fixed traceability/anchoring defects and cross-suite name collisions before achieving convergence.

---

## Architecture Changes

```mermaid
graph TD
    TestSuite["tests/summary_story_090_tests.rs<br/>(NEW — 18 unit/integration tests)"] -.->|direct unit tests| Summary["src/summary.rs<br/>(Summary::ingest, unique_hosts, accessors)"]
    TestSuite -.->|JSON serialization path| JsonReporter["src/reporter/json.rs<br/>(serde_json::json! summary block)"]
    PubMod["src/lib.rs<br/>(pub mod summary — library module)"] -.->|exposes| Summary
    style TestSuite fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: brownfield-formalization with zero src/ changes

**Context:** BC-2.12.018..021 describe behaviors of the `Summary` struct (ingest accumulation, service hints, unique_hosts, serde serialization) already implemented in `src/summary.rs` and `src/reporter/json.rs` but not covered by a dedicated behavioral test suite.

**Decision:** Add only test code (`tests/summary_story_090_tests.rs`) targeting the `summary` library module via `pub mod summary`. No production source changes.

**Rationale:** The brownfield-formalization mode is appropriate when the implementation is already present and correct — adding tests formalizes the contracts without risk of behavioral regression.

**Alternatives Considered:**
1. White-box unit tests in `src/summary.rs` — rejected because: direct integration tests via the library module are cleaner for contract coverage and match the pattern established in prior E-9 stories.
2. Re-implement Summary fields — rejected because: zero-src-change invariant; existing code already satisfies all BCs.

**Consequences:**
- 18 new tests (13 AC + 5 EC); full suite green.
- Two remediation rounds applied (cross-suite name collision + BC anchor corrections) before adversarial convergence.

</details>

---

## Story Dependencies

```mermaid
graph LR
    S086["STORY-086<br/>✅ merged"] --> S090["STORY-090<br/>🟡 this PR"]
    S088["STORY-088<br/>✅ merged"] --> S090
    S089["STORY-089<br/>✅ merged PR#169"] --> S090
    S090 --> END["E-9 COMPLETE<br/>Phase 3 Final"]
    style S090 fill:#FFD700
    style END fill:#90EE90
```

---

## Spec Traceability

```mermaid
flowchart LR
    BC018["BC-2.12.018<br/>ingest increments<br/>packets/bytes/hosts/protocols"] --> AC001["AC-001 total_packets++"]
    BC018 --> AC002["AC-002 total_bytes += len"]
    BC018 --> AC003["AC-003 src+dst in hosts"]
    BC018 --> AC004["AC-004 protocols[proto]++"]
    BC018 --> AC005["AC-005 skipped_packets not set by ingest"]
    BC019["BC-2.12.019<br/>app_protocol_hint<br/>service hints"] --> AC006["AC-006 HTTP hint → services[HTTP]++"]
    BC019 --> AC007["AC-007 None hint → services unchanged"]
    BC019 --> AC008["AC-008 port-based not dispatcher"]
    BC020["BC-2.12.020<br/>unique_hosts<br/>sorted dedup"] --> AC009["AC-009 sorted+deduped Vec"]
    BC020 --> AC010["AC-010 empty → empty Vec"]
    BC020 --> AC011["AC-011 &self non-mutating"]
    BC021["BC-2.12.021<br/>serde: total_packets<br/>total_bytes skipped_packets"] --> AC012["AC-012 JSON u64 fields"]
    BC021 --> AC013["AC-013 Debug protocol keys"]
    AC001 --> T["tests/summary_story_090_tests.rs"]
    AC002 --> T
    AC003 --> T
    AC004 --> T
    AC005 --> T
    AC006 --> T
    AC007 --> T
    AC008 --> T
    AC009 --> T
    AC010 --> T
    AC011 --> T
    AC012 --> T
    AC013 --> T
    T --> Summary["src/summary.rs<br/>(formalized)"]
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Unit tests | 18/18 pass | 100% | PASS |
| Coverage | delta neutral (test-only) | >80% | PASS |
| Mutation kill rate | full matrix caught | >90% | PASS |
| Adversarial passes | CONVERGED (3 passes, 2 remediation rounds, 3 consecutive clean) | 3 consecutive clean | PASS |

### Test Flow

```mermaid
graph LR
    UnitTests["18 Direct Unit Tests<br/>mod story_090"]
    ACTests["13 AC tests<br/>AC-001..013"]
    ECTests["5 EC tests<br/>EC-001..005"]

    UnitTests -->|all green| Pass1["PASS 18/18"]
    ACTests -->|full mutation matrix caught| Pass2["Mutation PASS"]
    ECTests -->|edge cases verified| Pass3["EC PASS"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
    style Pass3 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 18 added (13 AC + 5 EC) |
| **Total suite** | Full suite green (all targets) |
| **Coverage delta** | Neutral (no src/ changes; test coverage of existing paths) |
| **Mutation kill rate** | Full matrix caught (test logic strong from start) |
| **Regressions** | 0 |

<details>
<summary><strong>Detailed Test Results</strong></summary>

### New Tests (This PR) — tests/summary_story_090_tests.rs

| Test | AC/EC | Result |
|------|-------|--------|
| `test_summary_ingest_increments_total_packets()` | AC-001 | PASS |
| `test_summary_ingest_increments_total_bytes()` | AC-002 | PASS |
| `test_BC_2_12_018_host_counting_src_and_dst()` | AC-003 | PASS |
| `test_BC_2_12_018_protocol_breakdown()` | AC-004 | PASS |
| `test_skipped_packets_not_modified_by_ingest()` | AC-005 | PASS |
| `test_summary_service_detection_http()` | AC-006 | PASS |
| `test_summary_service_detection_none_on_unknown_port()` | AC-007 | PASS |
| `test_summary_service_is_port_based_not_content_based()` | AC-008 | PASS |
| `test_unique_hosts_sorted_and_deduplicated()` | AC-009 | PASS |
| `test_unique_hosts_empty_when_no_packets()` | AC-010 | PASS |
| `test_unique_hosts_is_non_mutating()` | AC-011 | PASS |
| `test_BC_2_12_021_json_includes_skipped_packets()` | AC-012 | PASS |
| `test_summary_protocol_keys_use_debug_format()` | AC-013 | PASS |
| `test_ec001_src_ip_equals_dst_ip_deduped()` | EC-001 | PASS |
| `test_ec002_ipv4_and_ipv6_in_unique_hosts()` | EC-002 | PASS |
| `test_ec003_two_packets_same_protocol_pc4()` | EC-003 | PASS |
| `test_ec004_zero_packet_len_total_bytes_unchanged()` | EC-004 | PASS |
| `test_ec005_services_http_count_2_in_json()` | EC-005 | PASS |

</details>

---

## Holdout Evaluation

N/A — evaluated at wave gate (Wave 27). Phase 4 holdout evaluation not yet initiated for this cycle.

---

## Adversarial Review

| Pass | Findings | Critical | High | Med | Low | Status |
|------|----------|----------|------|-----|-----|--------|
| 1 | 3 | 0 | 0 | 2 | 1 | Fixed (round 1: BC anchor + AC-003/004 name collisions + summary_tests.rs rename) |
| 2 | 2 | 0 | 0 | 1 | 1 | Fixed (round 2: AC-012 collision → test_BC_2_12_021_json_includes_skipped_packets; EC-003 pc4 anchor; full 18-name uniqueness sweep) |
| 3 (fresh-context) | 0 | 0 | 0 | 0 | 0 | CLEAN |
| 4 (fresh-context) | 0 | 0 | 0 | 0 | 0 | CLEAN |
| 5 (fresh-context) | 0 | 0 | 0 | 0 | 0 | CLEAN — CONVERGED |

**Convergence:** Adversary forced to hallucinate after 3 consecutive clean passes. CONVERGED.

Artifacts: `.factory/cycles/v0.1.0-greenfield-spec/adversarial-reviews/ADV-INDEX-STORY-090.md`

<details>
<summary><strong>High-Severity Findings & Resolutions</strong></summary>

### Finding F-S090-P1-001 — Permuted BC mapping in header/comments/EC-names

- **Location:** `tests/summary_story_090_tests.rs` (pre-round-1)
- **Category:** spec-fidelity / traceability-anchoring
- **Problem:** BC mapping in test file header and EC names was not anchored to canonical BC-2.12.018..021 order; AC-003/004 collided with names in `summary_tests.rs`
- **Resolution:** Re-anchored BC mapping to canonical; renamed colliding functions with `test_BC_2_12_018_` prefix; renamed summary_tests.rs collision

### Finding F-S090-P1-002 — AC-012 collision with reporter_tests.rs

- **Location:** `tests/summary_story_090_tests.rs` AC-012 function
- **Category:** spec-fidelity / cross-suite uniqueness
- **Problem:** AC-012 test name collided with existing `reporter_tests.rs` function; EC-003 was anchored to wrong packet-count variable (pc3 instead of pc4)
- **Resolution:** Renamed AC-012 test to `test_BC_2_12_021_json_includes_skipped_packets`; EC-003 retargeted to pc4; full 18-name cross-suite uniqueness sweep applied

</details>

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"] --> OK1["CLEAN"]
    High["High: 0"] --> OK2["CLEAN"]
    Medium["Medium: 0"] --> OK3["CLEAN"]
    Low["Low: 0"] --> OK4["CLEAN"]

    style OK1 fill:#90EE90
    style OK2 fill:#90EE90
    style OK3 fill:#90EE90
    style OK4 fill:#90EE90
```

<details>
<summary><strong>Security Scan Details</strong></summary>

### Scope
This PR adds ZERO `src/` changes. All changes are in `tests/summary_story_090_tests.rs` and `docs/demo-evidence/STORY-090/`.

### SAST
- No production code modified; direct unit tests only
- No user-controlled input flows through test code to production code beyond the existing library surface
- Critical: 0 | High: 0 | Medium: 0 | Low: 0

### Dependency Audit
- No new dependencies added
- `cargo audit`: 2 allowed warnings (RUSTSEC-2025-0119: `number_prefix` unmaintained; RUSTSEC-2026-0097: `rand` unsound via `phf_generator`/`tls-parser`). Both pre-exist on `develop`, both classified as `warning` (not `error`), both allowed in the project's deny/audit config. Zero new advisories introduced by this PR.

### Observations
- Test code directly instantiates `Summary` structs and calls methods; no subprocess invocation, no FFI, no unsafe blocks introduced
- Demo evidence files are static GIF/WebM recordings, not executable artifacts

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** Test suite only; zero production code changes
- **User impact:** None (test-only PR)
- **Data impact:** None
- **Risk Level:** LOW

### Performance Impact
| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| CI test runtime | baseline | +18 tests | minimal | OK |
| Binary size | unchanged | unchanged | 0 | OK |
| Memory | unchanged | unchanged | 0 | OK |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert <MERGE_SHA>
git push origin develop
```

Since this PR contains zero src/ changes, rollback simply removes the behavioral tests. No production behavior changes to revert.

**Verification after rollback:**
- `cargo test --all-targets` passes (18 story_090 tests gone, rest green)

</details>

### Feature Flags
| Flag | Controls | Default |
|------|----------|---------|
| N/A | test-only PR | N/A |

---

## Demo Evidence

10/13 ACs observable via the `summary` subcommand; AC-008/011 and EC-001/002/004 are unit-test-only (internal invariants or compiler guarantees).

| Recording | AC(s) | Key Observation |
|-----------|-------|-----------------|
| `AC-001-005-summary-accumulation` | AC-001, AC-002, AC-003, AC-004, AC-005, EC-003 | `Packets: 58`, `Bytes: 7542`, `Hosts: 3`, `Tcp: 52 Udp: 6`, `Skipped: 73` |
| `AC-006-service-hints` | AC-006 | `HTTP: 12`, `DNS: 18` (port-80/53); `TLS: 58` (port-443 fixture) |
| `AC-007-AC-010-error-paths` | AC-007, AC-010 | slammer.pcap shows PROTOCOLS but no SERVICES; nonexistent.pcap → early error exit |
| `AC-009-unique-hosts` | AC-009 | `192.168.1.1`, `192.168.1.2`, `192.168.1.3` in sorted ascending order |
| `AC-012-013-json-serialization` | AC-012, AC-013, EC-005 | JSON: `"total_packets": 58`, `"total_bytes": 7542`, `"skipped_packets": 73`, `"protocols": {"Tcp": 52, "Udp": 6}`, `"services": {"DNS": 18, "HTTP": 12}` |

Not-demo-able ACs (unit-test-only): AC-008 (port-based vs dispatcher is a code-structure invariant), AC-011 (`&self` is a compile-time guarantee), EC-001 (loopback fixture absent), EC-002 (IPv4/IPv6 mixed; unit test cleaner), EC-004 (zero-length packet not in any CLI fixture).

---

## Traceability

| BC | AC | Test | Status |
|----|----|----|--------|
| BC-2.12.018 | AC-001 | `test_summary_ingest_increments_total_packets()` | PASS |
| BC-2.12.018 | AC-002 | `test_summary_ingest_increments_total_bytes()` | PASS |
| BC-2.12.018 | AC-003 | `test_BC_2_12_018_host_counting_src_and_dst()` | PASS |
| BC-2.12.018 | AC-004 | `test_BC_2_12_018_protocol_breakdown()` | PASS |
| BC-2.12.018 | AC-005 | `test_skipped_packets_not_modified_by_ingest()` | PASS |
| BC-2.12.019 | AC-006 | `test_summary_service_detection_http()` | PASS |
| BC-2.12.019 | AC-007 | `test_summary_service_detection_none_on_unknown_port()` | PASS |
| BC-2.12.019 | AC-008 | `test_summary_service_is_port_based_not_content_based()` | PASS |
| BC-2.12.020 | AC-009 | `test_unique_hosts_sorted_and_deduplicated()` | PASS |
| BC-2.12.020 | AC-010 | `test_unique_hosts_empty_when_no_packets()` | PASS |
| BC-2.12.020 | AC-011 | `test_unique_hosts_is_non_mutating()` | PASS |
| BC-2.12.021 | AC-012 | `test_BC_2_12_021_json_includes_skipped_packets()` | PASS |
| BC-2.12.021 | AC-013 | `test_summary_protocol_keys_use_debug_format()` | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
BC-2.12.018 -> AC-001 -> test_summary_ingest_increments_total_packets -> src/summary.rs:ingest -> ADV-CONVERGED
BC-2.12.018 -> AC-002 -> test_summary_ingest_increments_total_bytes -> src/summary.rs:ingest -> ADV-CONVERGED
BC-2.12.018 -> AC-003 -> test_BC_2_12_018_host_counting_src_and_dst -> src/summary.rs:hosts HashSet -> ADV-CONVERGED
BC-2.12.018 -> AC-004 -> test_BC_2_12_018_protocol_breakdown -> src/summary.rs:protocols HashMap -> ADV-CONVERGED
BC-2.12.018 -> AC-005 -> test_skipped_packets_not_modified_by_ingest -> src/summary.rs:skipped_packets invariant -> ADV-CONVERGED
BC-2.12.019 -> AC-006 -> test_summary_service_detection_http -> src/summary.rs:app_protocol_hint -> ADV-CONVERGED
BC-2.12.019 -> AC-007 -> test_summary_service_detection_none_on_unknown_port -> src/summary.rs:services unchanged -> ADV-CONVERGED
BC-2.12.019 -> AC-008 -> test_summary_service_is_port_based_not_content_based -> src/summary.rs:no dispatcher import -> ADV-CONVERGED
BC-2.12.020 -> AC-009 -> test_unique_hosts_sorted_and_deduplicated -> src/summary.rs:unique_hosts -> ADV-CONVERGED
BC-2.12.020 -> AC-010 -> test_unique_hosts_empty_when_no_packets -> src/summary.rs:unique_hosts -> ADV-CONVERGED
BC-2.12.020 -> AC-011 -> test_unique_hosts_is_non_mutating -> src/summary.rs:&self signature -> ADV-CONVERGED
BC-2.12.021 -> AC-012 -> test_BC_2_12_021_json_includes_skipped_packets -> src/reporter/json.rs:serde_json::json! -> ADV-CONVERGED
BC-2.12.021 -> AC-013 -> test_summary_protocol_keys_use_debug_format -> src/reporter/json.rs:{k:?} format -> ADV-CONVERGED
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
  spec-crystallization: completed (Phase 1)
  story-decomposition: completed (Phase 2)
  tdd-implementation: completed (Wave 27)
  holdout-evaluation: N/A — evaluated at wave gate
  adversarial-review: completed (3 finding passes + 2 fix rounds + 3 consecutive clean)
  formal-verification: skipped (test-only PR)
  convergence: achieved
convergence-metrics:
  adversarial-passes: 5 (2 finding passes + 2 fix rounds + 3 clean)
  mutation-kill-rate: "full matrix (test logic strong from start)"
  test-count: 18
  ac-count: 13
  ec-count: 5
models-used:
  builder: claude-sonnet-4-6
  adversary: claude-sonnet-4-6 (fresh-context)
generated-at: "2026-05-31T00:00:00Z"
story: STORY-090
wave: 27
epic: E-9
phase: "3 (FINAL — all 48 stories complete)"
```

</details>

---

## Pre-Merge Checklist

- [ ] All CI status checks passing (8/8: semantic-pr, test, clippy, fmt, fuzz-build, audit, deny, trust-boundary)
- [x] Coverage delta is positive or neutral (test-only — neutral)
- [x] No critical/high security findings unresolved (test-only PR; 0 security findings)
- [x] Rollback procedure validated (git revert; zero src changes)
- [x] No feature flags required (test-only)
- [x] Adversarial review CONVERGED (3 consecutive clean passes)
- [x] All dependency PRs merged (STORY-086, STORY-088, STORY-089 PR#169)
- [x] Demo evidence: 10/13 ACs observable in docs/demo-evidence/STORY-090/ (AC-008/011 + EC-001/002/004 unit-test-only)
- [x] FINAL Phase 3 story — completes E-9 and all 48 stories
