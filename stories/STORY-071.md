---
document_type: story
story_id: STORY-071
epic_id: E-7
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.001.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.002.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.003.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.004.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.005.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.006.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.007.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.008.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.009.md
input-hash: "b3130e2"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-069, STORY-070]
blocks: [STORY-041, STORY-051, STORY-076]
behavioral_contracts:
  - BC-2.10.001
  - BC-2.10.002
  - BC-2.10.003
  - BC-2.10.004
  - BC-2.10.005
  - BC-2.10.006
  - BC-2.10.007
  - BC-2.10.008
  - BC-2.10.009
verification_properties: [VP-007, VP-016]
priority: P0
cycle: v0.1.0-greenfield-spec
wave: 3
target_module: mitre
subsystems: [SS-10]
estimated_days: 3
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-071`

# STORY-071: MITRE ATT&CK Mapping — Tactic Display, Catalog Lookup, all_tactics_in_report_order

## Narrative
- **As a** SOC operator using `--mitre` grouped output
- **I want** all 16 MITRE ATT&CK tactic variants (14 Enterprise + 2 ICS) to render with canonical display names, appear once each in kill-chain order via `all_tactics_in_report_order`, and have all 15 seeded technique IDs resolve correctly while unknown IDs return None
- **So that** the terminal reporter can group findings under correct tactic headers and SIEM consumers can verify technique-to-tactic mapping integrity

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.10.001 | MitreTactic Display Renders Enterprise Tactics with Canonical Spacing |
| BC-2.10.002 | ICS Tactics Render Unprefixed |
| BC-2.10.003 | all_tactics_in_report_order Returns Kill-Chain Order First Then ICS |
| BC-2.10.004 | all_tactics_in_report_order Contains Every Variant Exactly Once |
| BC-2.10.005 | technique_name Returns Some for Every Seeded ID (15 Total) |
| BC-2.10.006 | technique_name Returns None for Unknown IDs |
| BC-2.10.007 | technique_tactic Returns Correct Tactic for Every Seeded ID |
| BC-2.10.008 | All Emitted Technique IDs Resolve in Lookup |
| BC-2.10.009 | MitreTactic is #[non_exhaustive] |

## Acceptance Criteria

### AC-001 (traces to BC-2.10.001 postcondition 1)
All 14 Enterprise `MitreTactic` variants render with their canonical ATT&CK tactic name strings (e.g., `CommandAndControl` => `"Command and Control"` with lowercase "and").
- **Test:** `test_all_enterprise_tactic_display_strings()`

### AC-002 (traces to BC-2.10.001 invariant 3)
`"Command and Control"` uses lowercase "and" (canonical ATT&CK form), not "And".
- **Test:** `test_command_and_control_lowercase_and()` (part of AC-001 test)

### AC-003 (traces to BC-2.10.002 postcondition 1)
`MitreTactic::IcsInhibitResponseFunction` displays as `"Inhibit Response Function"` (no "ICS:" prefix).
- **Test:** `test_ics_inhibit_response_function_display()`

### AC-004 (traces to BC-2.10.002 postcondition 2)
`MitreTactic::IcsImpairProcessControl` displays as `"Impair Process Control"` (no "ICS:" prefix).
- **Test:** `test_ics_impair_process_control_display()`

### AC-005 (traces to BC-2.10.003 postcondition 1)
`all_tactics_in_report_order().len()` equals 16.
- **Test:** `test_all_tactics_length_is_16()`

### AC-006 (traces to BC-2.10.003 postcondition 2)
The first 14 elements are the Enterprise tactics in canonical kill-chain order: Reconnaissance, ResourceDevelopment, InitialAccess, Execution, Persistence, PrivilegeEscalation, DefenseEvasion, CredentialAccess, Discovery, LateralMovement, Collection, CommandAndControl, Exfiltration, Impact.
- **Test:** `test_all_tactics_enterprise_kill_chain_order()`

### AC-007 (traces to BC-2.10.003 postcondition 3)
Elements [14] and [15] are `IcsInhibitResponseFunction` and `IcsImpairProcessControl` respectively.
- **Test:** `test_all_tactics_ics_at_end()`

### AC-008 (traces to BC-2.10.004 postcondition 1)
Collecting `all_tactics_in_report_order()` into a `HashSet` produces a set of size 16 (no duplicates).
- **Test:** `test_all_tactics_no_duplicates()`

### AC-009 (traces to BC-2.10.004 postcondition 3)
No variant is omitted — all 16 variants appear in the slice (checked by comparing HashSet against the full expected set).
- **Test:** `test_all_tactics_all_variants_present()`

### AC-010 (traces to BC-2.10.005 postcondition 1)
`technique_name("T1027")` returns `Some("Obfuscated Files or Information")`.
- **Test:** `test_technique_name_resolves_all_15_seeded_ids()`

### AC-011 (traces to BC-2.10.005 postcondition 1)
All 15 seeded technique IDs resolve to `Some(name)`: T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004, T1083, T1499.002, T1505.003, T1573, T0846, T0855, T0856, T0885.
- **Test:** `test_technique_name_resolves_all_15_seeded_ids()` (exhaustive)

### AC-012 (traces to BC-2.10.006 postcondition 1)
`technique_name("T9999")`, `technique_name("")`, and `technique_name("t1027")` (lowercase) all return `None`.
- **Test:** `test_technique_name_returns_none_for_unknown_ids()`

### AC-013 (traces to BC-2.10.007 postcondition 2)
`technique_tactic("T1027")` returns `Some(MitreTactic::DefenseEvasion)`.
- **Test:** `test_technique_tactic_correct_assignments()`

### AC-014 (traces to BC-2.10.007 postcondition 2)
All 15 seeded technique-to-tactic assignments are correct (e.g., `T1046` => `Discovery`, `T1499.002` => `Impact`, `T0885` => `CommandAndControl`).
- **Test:** `test_technique_tactic_correct_assignments()` (exhaustive)

### AC-015 (traces to BC-2.10.008 postcondition 1)
All 6 currently-emitted technique IDs (T1027, T1036, T1046, T1083, T1499.002, T1505.003) resolve to `Some(...)` from both `technique_name` and `technique_tactic`.
- **Test:** `test_all_emitted_ids_resolve()`

### AC-016 (traces to BC-2.10.009 postcondition 1)
`MitreTactic` has `#[non_exhaustive]` attribute; external code matching on `MitreTactic` without a wildcard arm fails to compile.
- **Test:** `test_mitre_tactic_is_non_exhaustive()` (grep-based: `grep '#\[non_exhaustive\]' src/mitre.rs`)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `MitreTactic` enum | `src/mitre.rs:46-90` | pure-core |
| `all_tactics_in_report_order` | `src/mitre.rs:95-114` | pure-core |
| `technique_info` (lookup table) | `src/mitre.rs:122-155` | pure-core |
| `technique_name` | `src/mitre.rs` | pure-core |
| `technique_tactic` | `src/mitre.rs:166-168` | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `technique_name("T1059")` (real ATT&CK but not seeded) | `None` |
| EC-002 | `technique_name("T1046.001")` (sub-technique of seeded parent, not itself seeded) | `None` |
| EC-003 | `technique_name(" T1027")` (leading space) | `None` (no trimming) |
| EC-004 | `technique_tactic("T9999")` | `None` |
| EC-005 | `all_tactics_in_report_order()[0]` | `MitreTactic::Reconnaissance` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/mitre.rs` | pure-core | All functions are pure: static match tables, static slice, string formatting |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| `src/mitre.rs` | ~5,000 |
| `tests/mitre_tests.rs` (new or existing) | ~3,000 |
| BC files (9 BCs) | ~9,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~22,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~11%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-016 (test-writer)
2. [ ] Verify Red Gate: all 16 tests fail
3. [ ] Define `MitreTactic` enum with all 16 variants + `#[non_exhaustive]`
4. [ ] Implement `fmt::Display for MitreTactic` with all 14 Enterprise + 2 ICS canonical strings
5. [ ] Implement `all_tactics_in_report_order()` returning `&'static [MitreTactic]` of length 16 in kill-chain order
6. [ ] Implement `technique_info(id: &str)` static match table with all 15 seeded entries returning `(name, tactic)` pairs
7. [ ] Implement `technique_name(id)` and `technique_tactic(id)` as thin projections over `technique_info`
8. [ ] Verify the technique catalog count is exactly 15 (not 16 — pass-8 correction)
9. [ ] Write edge-case tests for EC-001 through EC-005
10. [ ] Run `cargo test --all-targets` and `cargo clippy -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-069 | `Finding.mitre_technique: Option<String>` — String technique ID | Technique IDs are plain strings (e.g., `"T1027"`), not typed | Catalog count is 15, NOT 16 (pass-8 correction of pass-6 claim) |
| STORY-070 | Finding JSON serialization established | None fields absent from JSON | — |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Technique catalog count is exactly 15 (not 16) | BC-2.10.005 invariant 3 | Count seeded entries in match table; test `technique_name` resolves exactly 15 |
| `all_tactics_in_report_order` length is always 16 | BC-2.10.003 invariant 2 | Test: `assert_eq!(all_tactics_in_report_order().len(), 16)` |
| `#[non_exhaustive]` on `MitreTactic` enum | BC-2.10.009 postcondition 1 | Grep: `grep '#\[non_exhaustive\]' src/mitre.rs` |
| `"Command and Control"` uses lowercase "and" | BC-2.10.001 invariant 3 | Test: exact string equality `"Command and Control"` |
| ICS tactic names have no "ICS:" prefix | BC-2.10.002 invariant 2 | Test: Display string does NOT start with "ICS:" |
| ICS techniques (T0xxx) appear AFTER all 14 Enterprise tactics | BC-2.10.003 postcondition 3 | Test: `all_tactics_in_report_order()[14]` is `IcsInhibitResponseFunction` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `std::fmt` | stdlib | `impl fmt::Display for MitreTactic` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/mitre.rs` | modify | `MitreTactic` enum, Display impl, `all_tactics_in_report_order`, `technique_info`, `technique_name`, `technique_tactic` |
| `tests/mitre_tests.rs` | create or modify | All AC test functions and edge-case tests |
