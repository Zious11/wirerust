---
document_type: story
story_id: STORY-069
epic_id: E-7
version: "1.4"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.002.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.003.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.004.md
input-hash: "2ba8b10"
traces_to: .factory/specs/prd.md
points: 5
depends_on: []
blocks: [STORY-070, STORY-071]
behavioral_contracts:
  - BC-2.09.001
  - BC-2.09.002
  - BC-2.09.003
  - BC-2.09.004
verification_properties: []
priority: P0
cycle: v0.1.0-greenfield-spec
wave: 1
target_module: findings
subsystems: [SS-09]
estimated_days: 2
tdd_mode: strict
nfr:
  - NFR-SEC-001
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-069`

# STORY-069: Finding Struct, Verdict/Confidence Display, and Finding Display Format

## Narrative
- **As a** forensic tool consumer
- **I want** the `Finding` struct to hold all required and optional fields, have correct uppercase Display tokens for `Verdict` and `Confidence`, and render a `[Category] VERDICT (CONFIDENCE) — summary` one-liner via `fmt::Display`
- **So that** all downstream reporters and log consumers receive a consistent, well-typed forensic finding with no escape logic at construction time

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.09.001 | Finding Constructed with Required Fields and Optional Fields |
| BC-2.09.002 | Finding Display Renders [Category] VERDICT (CONFIDENCE) — summary |
| BC-2.09.003 | Verdict Display: Uppercase Tokens |
| BC-2.09.004 | Confidence Display: Uppercase Tokens |

## Acceptance Criteria

### AC-001 (traces to BC-2.09.001 postcondition 1)
A `Finding` constructed with valid `category`, `verdict`, `confidence`, `summary`, `evidence` and all optional fields set to their specified values (including `timestamp: None`, `source_ip: Some(ip)` where applicable per BC invariants) compiles and can be passed to any reporter.
- **Test:** `test_finding_construction_with_all_fields()`

### AC-002 (traces to BC-2.09.001 invariant 1)
All 22 emission sites set `timestamp: None`; no production code path sets `timestamp: Some(...)`.
- **Test:** `test_timestamp_always_none_in_all_emission_sites()` (grep-based assertion)

### AC-003 (traces to BC-2.09.001 invariant 2)
All five reassembly anomaly emission sites set `source_ip: Some(src_ip)`: the three sites in `reassembly/mod.rs` (overlap, small-segment, out-of-window) and the two sites in `reassembly/lifecycle.rs` (conflicting-overlap, stream-depth-exceeded). HTTP and TLS findings have `source_ip: None`.
- **Test:** `test_source_ip_set_at_reassembly_sites()` and `test_source_ip_none_at_http_tls_sites()`

### AC-004 (traces to BC-2.09.002 postcondition 1)
`format!("{finding}")` where `finding.category = Anomaly`, `verdict = Likely`, `confidence = High`, `summary = "test"` produces `"[Anomaly] LIKELY (HIGH) — test"`.
- **Test:** `test_finding_display_format()`

### AC-005 (traces to BC-2.09.002 postcondition 5)
`Finding::Display` includes `summary` as-is; no escaping is applied (raw bytes preserved).
- **Test:** `test_finding_display_preserves_raw_summary()`

### AC-006 (traces to BC-2.09.003 postcondition 1)
`format!("{}", Verdict::Likely)` produces `"LIKELY"`.
- **Test:** `test_verdict_display_likely()`

### AC-007 (traces to BC-2.09.003 postcondition 2)
`format!("{}", Verdict::Unlikely)` produces `"UNLIKELY"`.
- **Test:** `test_verdict_display_unlikely()`

### AC-008 (traces to BC-2.09.003 postcondition 3)
`format!("{}", Verdict::Inconclusive)` produces `"INCONCLUSIVE"`.
- **Test:** `test_verdict_display_inconclusive()`

### AC-009 (traces to BC-2.09.004 postcondition 1)
`format!("{}", Confidence::High)` produces `"HIGH"`.
- **Test:** `test_confidence_display_high()`

### AC-010 (traces to BC-2.09.004 postcondition 2)
`format!("{}", Confidence::Medium)` produces `"MEDIUM"`.
- **Test:** `test_confidence_display_medium()`

### AC-011 (traces to BC-2.09.004 postcondition 3)
`format!("{}", Confidence::Low)` produces `"LOW"`.
- **Test:** `test_confidence_display_low()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Finding` struct | `src/findings.rs:119-146` | pure-core |
| `impl fmt::Display for Finding` | `src/findings.rs:157-168` | pure-core |
| `impl fmt::Display for Verdict` | `src/findings.rs:43-50` | pure-core |
| `impl fmt::Display for Confidence` | `src/findings.rs:68-76` | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `evidence = vec![]` | Valid Finding; reporters handle empty evidence list gracefully |
| EC-002 | `summary = ""` | Display renders `"[Anomaly] LIKELY (HIGH) — "` (trailing space after em-dash) |
| EC-003 | `summary` contains ESC byte (0x1B) | ESC byte appears literally in Display output (no escaping) |
| EC-004 | `direction = Some(ServerToClient)` | Field holds value; Display does not render direction |
| EC-005 | `category = Reconnaissance` | Display renders `"[Reconnaissance] ..."` (Debug format variant name) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/findings.rs` | pure-core | All types are pure value types; Display impls are pure string formatting |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| `src/findings.rs` | ~4,000 |
| `tests/reporter_tests.rs` | ~2,500 |
| BC files (4 BCs) | ~5,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~15,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~8%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-011 (test-writer)
2. [ ] Verify Red Gate: all tests fail
3. [ ] Define `Finding` struct with all required and optional fields and correct serde/derive attributes
4. [ ] Implement `fmt::Display for Verdict` with exact uppercase tokens
5. [ ] Implement `fmt::Display for Confidence` with exact uppercase tokens
6. [ ] Implement `fmt::Display for Finding` with template `"[{cat}] {verdict} ({conf}) — {summary}"` (plain Display on `ThreatCategory`; its `Display` impl delegates to `Debug` internally, producing the variant name)
7. [ ] Add `ThreatCategory`, `Direction` enums as referenced by `Finding`
8. [ ] Write edge-case tests for EC-001 through EC-005
9. [ ] Verify no escape function is called in any Finding construction site (`grep -rn 'escape_for_terminal' src/ | grep -v reporter`)
10. [ ] Run `cargo test --all-targets` and `cargo clippy -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-7 | — | — | — |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `escape_for_terminal` has exactly ONE call site: inside `TerminalReporter` (cross-ref: BC-2.09.005 invariant 1 — owned by STORY-070) | BC-2.09.002 invariant 2 (display boundary) | `grep -rn 'escape_for_terminal' src/ | grep -v reporter` must return nothing |
| `timestamp` field is always `None` at all 22 emission sites | BC-2.09.001 invariant 1 | Grep-based test: no `timestamp: Some` in production source |
| Display uses plain `{cat}` (Display) for `ThreatCategory`; `ThreatCategory::Display` delegates to Debug internally, so the rendered token is the variant name | BC-2.09.002 invariant 2 (v1.3) | Format string must use `{cat}` (plain Display), not `{cat:?}` (Debug) |
| `#[non_exhaustive]` is permitted on `Verdict` and `Confidence` and is applied per LESSON-P2.10; it is not forbidden | BC-2.09.003 invariant 2 / BC-2.09.004 invariant 2 | Code review |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `serde` | workspace version | `#[derive(Serialize)]` on `Finding` |
| `chrono` | workspace version | `Option<DateTime<Utc>>` type for `timestamp` field (always None currently) |
| `std::net::IpAddr` | stdlib | `source_ip: Option<IpAddr>` field |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/findings.rs` | modify | `Finding` struct, all enum types, Display impls |
| `tests/reporter_tests.rs` | modify | Add AC-001..AC-011 test functions and edge-case tests |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-05-21 | phase-3-adversarial-review | Minor-1: AC-003 prose expanded to cover all 5 source_ip: Some sites per BC-2.09.001 invariant 2 — added the two reassembly/lifecycle.rs sites (conflicting-overlap, stream-depth-exceeded) that were missing from the story prose; delivered test already covered all 5 sites |
| 1.2 | 2026-05-21 | phase-3-adversarial-review | Phase 3 per-story adversarial review — corrected Display template wording to match BC-2.09.002 v1.3 and `src/findings.rs`: Task 6 and Architecture Compliance Rules row 3 updated from `{cat:?}` to `{cat}` (plain Display); row 4 reworded to reflect that `#[non_exhaustive]` on `Verdict`/`Confidence` is permitted per LESSON-P2.10, not forbidden |
| 1.1 | 2026-05-21 | story-writer | Initial story decomposition |
