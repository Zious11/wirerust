---
document_type: story
story_id: STORY-156
epic_id: E-16
version: "1.5"
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
# v1.5 (2026-07-07): Pass-2 remediation F-156-P2-001..006 — wave TBD→71 (body+notes), BC-2.16.016 v1.0→v1.1 (table+authorship comment), --arp anchor corrected to symbol style, AC-003 pairs/20002-frames/assert fidelity, AC-004 clippy-compliant assert form.
# v1.3 (2026-07-07): adversarial Pass-1 remediation F-156-P1-001/002/003/006, wave 71 — AC-001 test name corrected; AC-001 provenance updated (909d55c+eca21e9); all bc_2_16_story113_arp_tests.rs refs updated to actual test locations; EC-003 trimmed to negative assertion.
# v1.2 (2026-07-07): provenance annotation — per-AC delivery notes (AC-001/003 pre-existing commits 909d55c/eca21e9; AC-002 invariant-by-inspection; AC-004 new delivery commit 7e4fe6d). Clarified Notes section. Added Changelog.
# BC status: BC-2.16.016 v1.0 authored 2026-06-23 (fix-pc-013-014-015, D-221; now v1.1). No prior story coverage;
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
input-hash: "934138e"
---

# STORY-156: ARP Findings Output Unbounded-Cap Documentation + Regression Test (BC-2.16.016)

**Epic:** E-16 (ARP Security Analyzer)
**Status:** draft
**Wave:** 71
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
| BC-2.16.016 v1.1 | ARP Findings Output is Unbounded — No MAX_FINDINGS Cap on process_arp Return Vec |

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
`--arp` flag definition in `src/cli.rs` (Architecture Anchor: `src/cli.rs` — `--arp` flag
definition, `long_help` attribute; per BC-2.16.016 Architecture Anchors, no line numbers) has a
`long_help` attribute or doc comment that explicitly states ARP findings output is
NOT bounded by any platform-imposed cap and can grow proportional to the number of triggering ARP
events in the capture. Operators analyzing untrusted captures from adversarial sources must be
informed of this behavior (PC-015 documentation fix).
- **Test:** `test_BC_2_16_016_cli_help_documents_arp_findings_unbounded` (`tests/bc_2_16_016_arp_tests.rs — fn test_BC_2_16_016_cli_help_documents_arp_findings_unbounded`) — use clap introspection or `--help` output
  capture to assert the `--arp` help text contains language indicating unbounded findings (e.g.,
  substring match on `"unbounded"` or `"no cap"` or `"proportional"`). If clap help text is not
  accessible in unit tests, verify manually during implementation and add an inline comment citing
  BC-2.16.016 PC-4 at the `long_help` site.
- **Provenance:** Pre-existing — commit 909d55c (`fix(cli): document ARP findings output is unbounded in --arp long_help [PC-015, BC-2.16.016]`, long_help doc fix) + commit eca21e9 (`test(arp): add BC-2.16.016 characterization + CLI help Red Gate tests [PC-015]`, CLI help test) — both from the fix-pc-013-014-015 cycle (D-221). Satisfied before this story was drafted.

### AC-002 (traces to BC-2.16.016 Invariant 1 — no MAX_FINDINGS on ARP path)
`ArpAnalyzer` does NOT define or reference a `MAX_FINDINGS` constant. Code inspection confirms
`src/analyzer/arp.rs` has no `const MAX_FINDINGS` definition. The only applicable bounds are
`MAX_ARP_BINDINGS = 65,536` (binding-table memory cap) and `MAX_STORM_COUNTERS =
4,096` (storm-counter memory cap); neither caps the findings Vec. This invariant is
enforced by the regression test in AC-003 (if a MAX_FINDINGS cap were accidentally introduced,
that test's `findings.len() > 10,000` assertion would fail).
- **Provenance:** Invariant verified by code inspection — no `MAX_FINDINGS` constant in `src/analyzer/arp.rs`. Enforced implicitly by the AC-003 regression test (commit eca21e9).

### AC-003 (traces to BC-2.16.016 canonical test vectors — Red Gate regression test)
`test_BC_2_16_016_arp_findings_vec_has_no_cap` in `src/analyzer/arp.rs` mod `bc_2_16_016`:

1. Creates `ArpAnalyzer::new(spoof_threshold=1, storm_rate=u32::MAX)` — `spoof_threshold=1` ensures
   the first rebind of any IP triggers a D1 finding immediately; `storm_rate=u32::MAX` suppresses
   D3 storm findings so all findings are D1.
2. Synthesizes 10,001 alternating-MAC pairs of ARP reply frames (`op=2`): each pair uses a unique
   `sender_ip`; the first frame of each pair sends MAC `AA:AA:AA:AA:AA:AA`, the second sends
   `BB:BB:BB:BB:BB:BB`, producing one MAC rebind (D1 finding) per pair. Total frames processed:
   20,002; total D1 findings expected: exactly 10,001 (matching BC-2.16.016 Canonical Test Vectors).
3. Calls `process_arp` for each frame; accumulates all returned `Vec<Finding>` items into a single
   `all_findings` vec.
4. **Primary assert: `assert_eq!(all_findings.len(), 10_001)`** — exactly one D1 finding per pair
   (10,001 pairs × 1 finding = 10,001 total). Secondary assert: `all_findings.len() > 10_000` —
   the finding count exceeds the reassembly-layer `MAX_FINDINGS = 10,000`, confirming no cap is
   applied to the ARP path.
5. The assertion MUST NOT be `all_findings.len() <= 10_000` (that would test the opposite invariant).
- **Test:** `test_BC_2_16_016_arp_findings_vec_has_no_cap`
- **Provenance:** Pre-existing — commit eca21e9 (`test(arp): add BC-2.16.016 characterization + CLI help Red Gate tests [PC-015]`) from the fix-pc-013-014-015 cycle (D-221), in `src/analyzer/arp.rs` mod `bc_2_16_016` and `tests/bc_2_16_016_arp_tests.rs`. Satisfied before this story was drafted.

### AC-004 (traces to BC-2.16.016 Postconditions 2+3 — summarize() NEVER emits dropped_findings)
`ArpAnalyzer::summarize()` output does NOT contain a `"dropped_findings"` key after processing any
sequence of ARP frames, including sequences producing more than 10,000 findings. Adding a
`"dropped_findings"` key would be a breaking change to the 13-key summarize contract (that
contract does not include `"dropped_findings"` and adding it would require a new contract version
plus its own delivery story).
- **Test:** `test_BC_2_16_016_summarize_has_no_dropped_findings_key` — after processing 10,001+
  ARP spoof events, assert `!summary.detail.contains_key("dropped_findings")` (clippy
  `unnecessary_get_then_check` compliance per the test's own docstring).
- **Provenance:** New delivery — `test_BC_2_16_016_summarize_has_no_dropped_findings_key` delivered by STORY-156 commit 7e4fe6d on `feature/STORY-156-arp-unbounded-doc`. This is the only net-new work in this story.

### AC-005 (standard gate)
`cargo test --all-targets` passes without regression; all existing VP-024 Sub-B/C/D harnesses from
STORY-112/113/114 remain green. `cargo clippy --all-targets -- -D warnings` passes.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `--arp` flag `long_help` text (PC-015 doc fix) | `src/cli.rs` | Effectful shell (CLI) |
| `test_BC_2_16_016_arp_findings_vec_has_no_cap` | `src/analyzer/arp.rs` mod `bc_2_16_016` | Test (pure) |
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
| EC-003 | `summarize()` after >10,000 events | no `"dropped_findings"` key (key-count contract covered under BC-2.16.010) |
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
   events, assert `!summary.detail.contains_key("dropped_findings")` (clippy
   `unnecessary_get_then_check` compliance).
4. **`test_BC_2_16_016_cli_help_documents_arp_findings_unbounded`** (AC-001, `tests/bc_2_16_016_arp_tests.rs — fn test_BC_2_16_016_cli_help_documents_arp_findings_unbounded`): already delivered by eca21e9. Verify the test passes in the final `cargo test` run.
5. **Run `cargo test --all-targets`**: all tests green; existing VP-024 harnesses pass.
6. **Run `cargo clippy --all-targets -- -D warnings`**: clean.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_16_016_cli_help_documents_arp_findings_unbounded` (`tests/bc_2_16_016_arp_tests.rs — fn test_BC_2_16_016_cli_help_documents_arp_findings_unbounded`) | Unit / CLI |
| AC-002 | Enforced implicitly via AC-003 | Implicit regression |
| AC-003 | `test_BC_2_16_016_arp_findings_vec_has_no_cap` | Unit (10,001 frames) |
| AC-004 | `test_BC_2_16_016_summarize_has_no_dropped_findings_key` | Unit |
| AC-005 | `cargo test --all-targets` + clippy | CI gate |

## Notes

- **No code behavior change required.** The absence of a MAX_FINDINGS cap on the ARP path is
  already the shipped behavior. This story adds CLI documentation (PC-015 fix) and regression tests
  that assert the existing invariant.
- AC-001 test (`test_BC_2_16_016_cli_help_documents_arp_findings_unbounded`) lives in
  `tests/bc_2_16_016_arp_tests.rs` (created by eca21e9). AC-003 and AC-004 tests live in
  `src/analyzer/arp.rs` mod `bc_2_16_016`. Do NOT add a duplicate test to `tests/bc_2_16_story113_arp_tests.rs`.
- Relationship to STORY-113: STORY-113 holds BC-2.16.016 in its `behavioral_contracts` frontmatter
  (added in the v1.4 amendment after BC-2.16.016 was authored). STORY-156 is the primary DELIVERY
  story for BC-2.16.016 behavioral tests and CLI documentation.
- STORY-116 (E-17, wave 45) also depends on STORY-115 and coexists with this story's wave 71
  assignment. STORY-156 is logically independent of E-17 QinQ/MACsec changes.
- **Delivery provenance context (why the PR diff is small, ~61 lines):** ACs 001, 002, and 003 were
  already satisfied on `develop` before this story was drafted — they were delivered as part of the
  fix-pc-013-014-015 cycle (D-221) via commits 909d55c (AC-001, CLI `long_help` doc fix) and
  eca21e9 (AC-003, regression test). AC-002 is an invariant verified by code inspection with no
  separate test required. Only AC-004 (`test_BC_2_16_016_summarize_has_no_dropped_findings_key`)
  required net-new work, delivered in commit 7e4fe6d on `feature/STORY-156-arp-unbounded-doc`.
  STORY-156 is the **traceability-closure and primary-coverage story for BC-2.16.016** (as indicated
  in its `behavioral_contracts:` frontmatter). Its purpose is to make the existing pre-existing work
  traceable under the VSDD pipeline, not to deliver from scratch.

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
| STORY-113 | Introduced `ArpAnalyzer` with `process_arp` returning `Vec<Finding>` — no `MAX_FINDINGS` cap by design | `process_arp` unbounded on the link-layer path | AC-001 test is in `tests/bc_2_16_016_arp_tests.rs`; AC-003/004 tests are in `src/analyzer/arp.rs` mod `bc_2_16_016` — do NOT add duplicates to `tests/bc_2_16_story113_arp_tests.rs` |
| STORY-115 | Added `storm_rate` constructor parameter | `ArpAnalyzer::new(spoof_threshold, storm_rate)` signature; `storm_rate=u32::MAX` suppresses D3 storm findings | STORY-115 finalizes the 13-key `summarize()` contract — AC-004 is only fully testable after STORY-115 merges |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| No behavior change to `ArpAnalyzer::process_arp` — documentation + tests only | BC-2.16.016 design intent | Code review; test outcome confirms cap absence |
| Must NOT introduce a `MAX_FINDINGS` constant on the ARP path | BC-2.16.016 Invariant 1 | AC-003 regression test asserts `findings.len() > 10_000` |
| Must NOT add `"dropped_findings"` key to `summarize()` output | 13-key summarize contract | AC-004 test asserts `!summary.detail.contains_key("dropped_findings")` (clippy `unnecessary_get_then_check`) |
| AC-001 test is in `tests/bc_2_16_016_arp_tests.rs`; AC-003/004 tests are in `src/analyzer/arp.rs` mod `bc_2_16_016` — do NOT add to `tests/bc_2_16_story113_arp_tests.rs` | No duplicate test files | Code review |
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
| `src/analyzer/arp.rs` mod `bc_2_16_016` | modify | Add `test_BC_2_16_016_arp_findings_vec_has_no_cap` (AC-003) and `test_BC_2_16_016_summarize_has_no_dropped_findings_key` (AC-004) — tests live here per eca21e9/7e4fe6d |

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

## Changelog

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.5 | 2026-07-07 | state-manager | Pass-2 remediation F-156-P2-001..006 — body freshness sweep: wave TBD→71 (body field + notes), BC-2.16.016 v1.0→v1.1 in BC table + authorship comment; --arp anchor corrected to symbol style (removed mis-attributed lines 194–213); AC-003 step 2 updated to pairs/20,002-frames fidelity; AC-003 step 4 documents both primary assert_eq!(len, 10_001) and secondary > 10_000; AC-004 assert updated to clippy-compliant !contains_key form at 3 sites; test-file line citations converted to symbol-style (F-156-P3-001). |
| 1.4 | 2026-07-07 | state-manager | Input re-hash after anchor-only BC amendment BC-2.16.016 v1.1, wave 71; no scope impact. |
| 1.3 | 2026-07-07 | state-manager | Adversarial Pass-1 remediation F-156-P1-001/002/003/006, wave 71 — corrected AC-001 test name to test_BC_2_16_016_cli_help_documents_arp_findings_unbounded (tests/bc_2_16_016_arp_tests.rs:58); updated AC-001 provenance to credit both 909d55c (long_help doc) and eca21e9 (CLI help test); updated all test-file references from bc_2_16_story113_arp_tests.rs to actual placement (AC-001 in tests/bc_2_16_016_arp_tests.rs, AC-003/004 in src/analyzer/arp.rs mod bc_2_16_016); trimmed EC-003 to negative assertion with BC-2.16.010 cross-reference. |
| 1.2 | 2026-07-07 | state-manager | Provenance annotation — per-AC delivery notes (AC-001/003 pre-existing commits 909d55c/eca21e9; AC-002 invariant-by-inspection; AC-004 new delivery commit 7e4fe6d on feature/STORY-156-arp-unbounded-doc). Notes section clarified: traceability-closure/primary-coverage framing for BC-2.16.016; explains why story PR diff is small (~61 lines). |
| 1.1 | 2026-07-07 | state-manager | Wave assigned 71 (v0.12.0 planning gate, 2026-07-07 human approval). Added missing template compliance fields (wave, level, cycle, assumption_validations, risk_mitigations, traces_to) and mandatory sections (Purity Classification, Previous Story Intelligence, Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements). |
