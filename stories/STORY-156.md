---
document_type: story
story_id: STORY-156
epic_id: E-16
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-07-06T00:00:00Z
phase: f3
level: feature
cycle: maint-2026-07-06
points: 3
priority: P1
wave: "71"
depends_on: [STORY-115]
blocks: []
behavioral_contracts:
  - BC-2.16.016
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: analyzer/arp
subsystems: [SS-16]
estimated_days: 1
feature_id: issue-009-arp-security-analyzer
github_issue: 9
# BC status: BC-2.16.016 v1.0 authored 2026-06-23 (fix-pc-013-014-015, D-221). No prior story coverage;
# spec-coherence sweep 7 (maint-2026-07-06) finding F-NEW-MAJ-003 identified this gap (criterion 27 FAIL).
# This story closes the gap; it is the primary delivery story for BC-2.16.016 behavioral tests and CLI doc.
# v1.1 (2026-07-07): Wave assigned 71 (v0.12.0 planning gate, 2026-07-07 human approval). Added
#   missing template compliance fields (wave, level, cycle, assumption_validations, risk_mitigations,
#   traces_to) and mandatory sections (Purity Classification, Previous Story Intelligence,
#   Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements).
traces_to:
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.016.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.010.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md
  - src/analyzer/arp.rs
  - src/cli.rs
inputs:
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.016.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.010.md
  - .factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md
input-hash: "ce96d86"
---

# STORY-156: ARP Findings Output Unbounded-Cap Documentation + Regression Test (BC-2.16.016)

**Epic:** E-16 (ARP Security Analyzer)
**Status:** draft
**Wave:** TBD
**Points:** 3

## Narrative

- **As a** ICS/OT security analyst using wirerust
- **I want** the `--arp` CLI flag to explicitly document that ARP findings output is unbounded (no
  MAX_FINDINGS cap on the `process_arp` return Vec), and for a regression test to guard that invariant
- **So that** operators analyzing adversarial captures with massive ARP-storm or ARP-spoof events are
  informed of the proportional findings growth, and any accidental future introduction of a MAX_FINDINGS
  cap on the ARP path is immediately caught by the test suite

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.16.016 v1.0 | ARP Findings Output is Unbounded — No MAX_FINDINGS Cap on process_arp Return Vec |

## Background

`ArpAnalyzer::process_arp` returns a `Vec<Finding>` with NO upper bound on findings. Unlike HTTP, TLS,
Modbus, and DNP3 analyzers (which pass through `TcpReassembler` and are capped at `MAX_FINDINGS =
10,000` from `src/reassembly/mod.rs:57`), ARP operates at the Ethernet link layer and bypasses TCP
reassembly entirely — ARP operates at the Ethernet link layer and bypasses TCP reassembly by design.
The internal binding-table cap (`MAX_ARP_BINDINGS =
65,536`) and storm-counter cap (`MAX_STORM_COUNTERS = 4,096`) are MEMORY
bounds on internal HashMaps, NOT bounds on the findings Vec.

This is intentional design for a CLI forensics tool where operators own their pcap files and need
complete finding records. BC-2.16.016 was authored in fix-pc-013-014-015 (D-221, 2026-06-23) to
contractually specify this absence as a behavioral invariant. Two deliverables remain missing:

1. CLI `--help` text for `--arp` must document the absence of a findings cap
   (BC-2.16.016 Postcondition 4 / PC-015 documentation fix).
2. A regression test `test_BC_2_16_016_arp_findings_vec_has_no_cap` must guard against accidental
   future cap introduction (BC-2.16.016 Invariant 5 / canonical test vectors).

Source gap: spec-coherence sweep 7 (maint-2026-07-06) finding F-NEW-MAJ-003 — criterion 27 FAIL
("all active BCs must have at least one active story").

## Acceptance Criteria

### AC-001 (traces to BC-2.16.016 Postcondition 4 — CLI --arp flag documents unbounded findings)
`--arp` flag definition in `src/cli.rs` (Architecture Anchor: approximately lines 194–213 per
BC-2.16.016) has a `long_help` attribute or doc comment that explicitly states ARP findings output is
NOT bounded by any platform-imposed cap and can grow proportional to the number of triggering ARP
events in the capture. Operators analyzing untrusted captures from adversarial sources must be
informed of this behavior (PC-015 documentation fix).
- **Test:** `test_arp_flag_help_documents_unbounded` — use clap introspection or `--help` output
  capture to assert the `--arp` help text contains language indicating unbounded findings (e.g.,
  substring match on `"unbounded"` or `"no cap"` or `"proportional"`). If clap help text is not
  accessible in unit tests, verify manually during implementation and add an inline comment citing
  BC-2.16.016 PC-4 at the `long_help` site.

### AC-002 (traces to BC-2.16.016 Invariant 1 — no MAX_FINDINGS on ARP path)
`ArpAnalyzer` does NOT define or reference a `MAX_FINDINGS` constant. Code inspection confirms
`src/analyzer/arp.rs` has no `const MAX_FINDINGS` definition. The only applicable bounds are
`MAX_ARP_BINDINGS = 65,536` (binding-table memory cap) and `MAX_STORM_COUNTERS =
4,096` (storm-counter memory cap); neither caps the findings Vec. This invariant is
enforced by the regression test in AC-003 (if a MAX_FINDINGS cap were accidentally introduced,
that test's `findings.len() > 10,000` assertion would fail).

### AC-003 (traces to BC-2.16.016 canonical test vectors — Red Gate regression test)
`test_BC_2_16_016_arp_findings_vec_has_no_cap` in `src/analyzer/arp.rs` `#[cfg(test)] mod tests`
or `tests/bc_2_16_story113_arp_tests.rs`:

1. Creates `ArpAnalyzer::new(spoof_threshold=1, storm_rate=u32::MAX)` — `spoof_threshold=1` ensures
   the first rebind of any IP triggers a D1 finding immediately; `storm_rate=u32::MAX` suppresses
   D3 storm findings so all findings are D1.
2. Synthesizes 10,001 ARP reply frames (`op=2`): each unique `sender_ip`, alternating between two
   sender MACs (even index → `AA:AA:AA:AA:AA:AA`, odd index → `BB:BB:BB:BB:BB:BB`), producing MAC
   rebind events on every second frame per IP.
3. Calls `process_arp` for each frame; accumulates all returned `Vec<Finding>` items into a single
   `all_findings` vec.
4. **Asserts `all_findings.len() > 10_000`** — the finding count exceeds the reassembly-layer
   `MAX_FINDINGS = 10,000`, confirming no cap is applied to the ARP path.
5. The assertion MUST NOT be `all_findings.len() <= 10_000` (that would test the opposite invariant).
- **Test:** `test_BC_2_16_016_arp_findings_vec_has_no_cap`

### AC-004 (traces to BC-2.16.016 Postconditions 2+3 — summarize() NEVER emits dropped_findings)
`ArpAnalyzer::summarize()` output does NOT contain a `"dropped_findings"` key after processing any
sequence of ARP frames, including sequences producing more than 10,000 findings. Adding a
`"dropped_findings"` key would be a breaking change to the 13-key summarize contract (that
contract does not include `"dropped_findings"` and adding it would require a new contract version
plus its own delivery story).
- **Test:** `test_BC_2_16_016_summarize_has_no_dropped_findings_key` — after processing 10,001+
  ARP spoof events, assert `summarize().get("dropped_findings").is_none()`.

### AC-005 (standard gate)
`cargo test --all-targets` passes without regression; all existing VP-024 Sub-B/C/D harnesses from
STORY-112/113/114 remain green. `cargo clippy --all-targets -- -D warnings` passes.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `--arp` flag `long_help` text (PC-015 doc fix) | `src/cli.rs` | Effectful shell (CLI) |
| `test_BC_2_16_016_arp_findings_vec_has_no_cap` | `src/analyzer/arp.rs` tests or `tests/bc_2_16_story113_arp_tests.rs` | Test (pure) |
| `test_BC_2_16_016_summarize_has_no_dropped_findings_key` | same test module | Test (pure) |
| `ArpAnalyzer::process_arp` (NO behavior change — cap absence is the current shipped behavior) | `src/analyzer/arp.rs` | Pure core (stateful) |

Architecture references: ARP link-layer bypass invariant (ARP bypasses TCP reassembly),
13-key summarize contract (adding `dropped_findings` would break it),
`src/reassembly/mod.rs:57` (`MAX_FINDINGS = 10,000` applies to HTTP/TLS/Modbus/DNP3 only, not ARP).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 0 ARP frames processed | `all_findings.len() == 0`; `summarize()` has no `"dropped_findings"` key |
| EC-002 | Exactly 10,001 D1 spoof events | `all_findings.len() >= 10,001` (> 10,000); no cap applied |
| EC-003 | `summarize()` after >10,000 events | 13-key output per the summarize contract; no `"dropped_findings"` key |
| EC-004 | `MAX_ARP_BINDINGS` reached during >10,000-event test | Binding-table eviction fires (LRU eviction policy); `bindings_evicted` incremented in summarize(); findings Vec still unbounded |

## Tasks

1. **Update `--arp` flag `long_help` in `src/cli.rs`**: add documentation that `process_arp`
   findings output is NOT bounded by any platform cap (BC-2.16.016 PC-4 / PC-015 fix). State that
   the binding-table cap (`MAX_ARP_BINDINGS=65,536`) and storm-counter cap (`MAX_STORM_COUNTERS=
   4,096`) are MEMORY bounds only and do not cap the findings Vec. Add inline comment citing
   `BC-2.16.016 PC-4`.
2. **Write `test_BC_2_16_016_arp_findings_vec_has_no_cap`** (AC-003): follow the canonical test
   protocol from BC-2.16.016 §Canonical Test Vectors exactly. Assert `all_findings.len() > 10_000`.
3. **Write `test_BC_2_16_016_summarize_has_no_dropped_findings_key`** (AC-004): after >10,000
   events, assert `summarize().get("dropped_findings").is_none()`.
4. **Add `test_arp_flag_help_documents_unbounded`** (AC-001): if clap `--help` text is accessible
   in unit tests, assert the substring. Otherwise verify manually during implementation.
5. **Run `cargo test --all-targets`**: all tests green; existing VP-024 harnesses pass.
6. **Run `cargo clippy --all-targets -- -D warnings`**: clean.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_arp_flag_help_documents_unbounded` (or manual verification) | Unit / CLI |
| AC-002 | Enforced implicitly via AC-003 | Implicit regression |
| AC-003 | `test_BC_2_16_016_arp_findings_vec_has_no_cap` | Unit (10,001 frames) |
| AC-004 | `test_BC_2_16_016_summarize_has_no_dropped_findings_key` | Unit |
| AC-005 | `cargo test --all-targets` + clippy | CI gate |

## Notes

- **No code behavior change required.** The absence of a MAX_FINDINGS cap on the ARP path is
  already the shipped behavior. This story adds CLI documentation (PC-015 fix) and regression tests
  that assert the existing invariant.
- The test file `tests/bc_2_16_story113_arp_tests.rs` may already exist from STORY-113. If so, add
  these tests to the existing file. Do NOT create a duplicate test file.
- Relationship to STORY-113: STORY-113 holds BC-2.16.016 in its `behavioral_contracts` frontmatter
  (added in the v1.4 amendment after BC-2.16.016 was authored). STORY-156 is the primary DELIVERY
  story for BC-2.16.016 behavioral tests and CLI documentation.
- STORY-116 (E-17, wave 45) also depends on STORY-115 and coexists with this story's wave-TBD
  assignment. STORY-156 is logically independent of E-17 QinQ/MACsec changes.

## Dependency Rationale

- `depends_on: [STORY-115]` — `ArpAnalyzer::new(spoof_threshold, storm_rate)` with the `storm_rate`
  parameter exists only after STORY-115 ships. The regression test uses `storm_rate=u32::MAX` to
  suppress D3 findings. STORY-115 also completes the 13-key `summarize()` contract, making AC-004
  testable (the 13-key summarize contract is finalized by STORY-115).
- `blocks: []` — no downstream stories depend on this story.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `src/cli.rs` — `--arp` flag `long_help` | Effectful shell (CLI) | Updates CLI argument parser definition |
| `src/analyzer/arp.rs` — `process_arp` | Pure core (stateful) | No behavior change; cap absence is the existing behavior |
| Test functions (AC-003/004) | Pure (test-only) | In-memory computation; no I/O side effects |

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-113 | Introduced `ArpAnalyzer` with `process_arp` returning `Vec<Finding>` — no `MAX_FINDINGS` cap by design | `process_arp` unbounded on the link-layer path | Test file `tests/bc_2_16_story113_arp_tests.rs` may already exist — add AC-003/004 tests there, do NOT create a duplicate file |
| STORY-115 | Added `storm_rate` constructor parameter | `ArpAnalyzer::new(spoof_threshold, storm_rate)` signature; `storm_rate=u32::MAX` suppresses D3 storm findings | STORY-115 finalizes the 13-key `summarize()` contract — AC-004 is only fully testable after STORY-115 merges |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| No behavior change to `ArpAnalyzer::process_arp` — documentation + tests only | BC-2.16.016 design intent | Code review; test outcome confirms cap absence |
| Must NOT introduce a `MAX_FINDINGS` constant on the ARP path | BC-2.16.016 Invariant 1 | AC-003 regression test asserts `findings.len() > 10_000` |
| Must NOT add `"dropped_findings"` key to `summarize()` output | 13-key summarize contract | AC-004 test asserts `.get("dropped_findings").is_none()` |
| Reuse existing test file if `tests/bc_2_16_story113_arp_tests.rs` exists | No duplicate test files | Code review |
| `cargo clippy --all-targets -- -D warnings` must pass | CLAUDE.md CI gate | CI |

## Library & Framework Requirements (MANDATORY)

| Tool / Library | Version | Purpose |
|---------------|---------|---------|
| Rust stable | per `rust-version` in Cargo.toml (≥1.91) | No new dependencies introduced |
| clap | existing project version | `--arp` flag `long_help` attribute update |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/cli.rs` | modify | Add `long_help` attribute to `--arp` flag documenting unbounded findings (BC-2.16.016 PC-4 / PC-015) |
| `src/analyzer/arp.rs` test module or `tests/bc_2_16_story113_arp_tests.rs` | modify | Add `test_BC_2_16_016_arp_findings_vec_has_no_cap` (AC-003) and `test_BC_2_16_016_summarize_has_no_dropped_findings_key` (AC-004) |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~3,500 |
| BC-2.16.016 (primary BC) | ~4,000 |
| 13-key summarize contract context | ~2,500 |
| Link-layer bypass invariant context | ~1,500 |
| STORY-113 / STORY-115 context (ArpAnalyzer state post-delivery) | ~2,000 |
| Existing `src/analyzer/arp.rs` (after STORY-113/114/115) | ~3,500 |
| Tool outputs (cargo test) | ~1,500 |
| **Total estimated** | **~18,500** |
