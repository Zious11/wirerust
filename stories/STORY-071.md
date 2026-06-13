---
document_type: story
story_id: STORY-071
epic_id: E-7
version: "1.10"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
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
input-hash: "38c614a"
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
nfr:
  - NFR-OBS-004
  - NFR-MNT-004
  - NFR-MNT-009
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-071`

# STORY-071: MITRE ATT&CK Mapping — Tactic Display, Catalog Lookup, all_tactics_in_report_order

## Narrative
- **As a** SOC operator using `--mitre` grouped output
- **I want** all 17 MITRE ATT&CK tactic variants (14 Enterprise + 3 ICS incl. IcsImpact) to render with canonical display names, appear once each in kill-chain order via `all_tactics_in_report_order`, and have all 23 seeded technique IDs (current; 25 after STORY-114 — PLANNED) resolve correctly while unknown IDs return None
- **So that** the terminal reporter can group findings under correct tactic headers and SIEM consumers can verify technique-to-tactic mapping integrity

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.10.001 | MitreTactic Display Renders Enterprise Tactics with Canonical Spacing |
| BC-2.10.002 | ICS Tactics Render Unprefixed |
| BC-2.10.003 | all_tactics_in_report_order Returns Kill-Chain Order First Then ICS |
| BC-2.10.004 | all_tactics_in_report_order Contains Every Variant Exactly Once |
| BC-2.10.005 | technique_name Returns Some for Every Seeded ID (21 Total) |
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
- **Test:** `test_command_and_control_lowercase_and()`

### AC-003 (traces to BC-2.10.002 postcondition 1)
`MitreTactic::IcsInhibitResponseFunction` displays as `"Inhibit Response Function"` (no "ICS:" prefix).
- **Test:** `test_ics_inhibit_response_function_display()`

### AC-004 (traces to BC-2.10.002 postcondition 2)
`MitreTactic::IcsImpairProcessControl` displays as `"Impair Process Control"` (no "ICS:" prefix).
- **Test:** `test_ics_impair_process_control_display()`

### AC-005 (traces to BC-2.10.003 postcondition 1)
`all_tactics_in_report_order().len()` equals 17.
- **Test:** `test_all_tactics_length_is_17()`

### AC-006 (traces to BC-2.10.003 postcondition 2)
The first 14 elements are the Enterprise tactics in canonical kill-chain order: Reconnaissance, ResourceDevelopment, InitialAccess, Execution, Persistence, PrivilegeEscalation, DefenseEvasion, CredentialAccess, Discovery, LateralMovement, Collection, CommandAndControl, Exfiltration, Impact.
- **Test:** `test_all_tactics_enterprise_kill_chain_order()`

### AC-007 (traces to BC-2.10.003 postcondition 3)
Elements [14], [15], and [16] are `IcsInhibitResponseFunction`, `IcsImpairProcessControl`, and `IcsImpact` respectively.
- **Test:** `test_all_tactics_ics_at_end()`

### AC-008 (traces to BC-2.10.004 postcondition 2)
Collecting `all_tactics_in_report_order()` into a `HashSet` produces a set of size 17 (no duplicates).
- **Test:** `test_all_tactics_no_duplicates()`

### AC-009 (traces to BC-2.10.004 postconditions 1, 3)
No variant is omitted — all 17 variants appear in the slice (checked by comparing HashSet against the full expected set).
- **Test:** `test_all_tactics_all_variants_present()`
- **Note:** BC-2.10.004 postcondition 1 ("each variant appears exactly once") is verified jointly: AC-008's no-duplicate check plus AC-009's no-omission set-equality check together establish "exactly once."

### AC-010 (traces to BC-2.10.005 postcondition 1)
`technique_name("T1027")` returns `Some("Obfuscated Files or Information")`.
- **Test:** `test_technique_name_resolves_all_23_seeded_ids()`

### AC-011 (traces to BC-2.10.005 postcondition 1)
All **23** seeded technique IDs resolve to `Some(name)` (current; 25 after STORY-114 — PLANNED):
- Enterprise (11): T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004, T1083, T1499.002, T1505.003, T1573
- ICS (12): T0846, T1692.001, T1692.002, T0885, T0836, T0814, T0806, T0835, T0831, T0888, T1691.001, T0827
- **Test:** `test_technique_name_resolves_all_23_seeded_ids()` (exhaustive)

### AC-012 (traces to BC-2.10.006 postcondition 1)
`technique_name("T9999")`, `technique_name("")`, and `technique_name("t1027")` (lowercase) all return `None`.
- **Test:** `test_technique_name_returns_none_for_unknown_ids()`

### AC-013 (traces to BC-2.10.007 postcondition 2)
`technique_tactic("T1027")` returns `Some(MitreTactic::DefenseEvasion)`.
- **Test:** `test_technique_tactic_correct_assignments()`

### AC-014 (traces to BC-2.10.007 postcondition 2)
All 23 seeded technique-to-tactic assignments are correct (current; 25 after STORY-114 — PLANNED):
- Enterprise (11): `T1027` => `DefenseEvasion`, `T1036` => `DefenseEvasion`, `T1040` => `CredentialAccess`, `T1046` => `Discovery`, `T1071` => `CommandAndControl`, `T1071.001` => `CommandAndControl`, `T1071.004` => `CommandAndControl`, `T1083` => `Discovery`, `T1499.002` => `Impact`, `T1505.003` => `Persistence`, `T1573` => `CommandAndControl`
- ICS (12): `T0846` => `Discovery`, `T1692.001` => `IcsImpairProcessControl`, `T1692.002` => `IcsImpairProcessControl`, `T0885` => `CommandAndControl`, `T0836` => `IcsImpairProcessControl`, `T0814` => `IcsInhibitResponseFunction`, `T0806` => `IcsImpairProcessControl`, `T0835` => `IcsImpairProcessControl`, `T0831` => `IcsImpairProcessControl`, `T0888` => `Discovery`, `T1691.001` => `IcsInhibitResponseFunction`, `T0827` => `IcsImpact`
- **Test:** `test_technique_tactic_correct_assignments()` (exhaustive)

### AC-015 (traces to BC-2.10.008 postcondition 1)
All **13** currently-emitted technique IDs resolve to `Some(...)` from both `technique_name` and `technique_tactic`:
- Enterprise (6): T1027, T1036, T1046, T1083, T1499.002, T1505.003
- ICS (7): T1692.001, T0836, T0814, T0806, T0835, T0831, T0888
- **Test:** `test_all_emitted_ids_resolve()`

### AC-016 (traces to BC-2.10.009 postcondition 1)
`MitreTactic` has `#[non_exhaustive]` attribute; external code matching on `MitreTactic` without a wildcard arm fails to compile.
- **Test:** `test_mitre_tactic_is_non_exhaustive()` (grep-based: `grep '#\[non_exhaustive\]' src/mitre.rs`)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `MitreTactic` enum | `src/mitre.rs:46-66` | pure-core |
| `all_tactics_in_report_order` | `src/mitre.rs:100-120` | pure-core |
| `technique_info` (lookup table) | `src/mitre.rs:128-182` | pure-core |
| `technique_name` | `src/mitre.rs:186-188` | pure-core |
| `technique_tactic` | `src/mitre.rs:192-194` | pure-core |

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
2. [ ] Verify Red Gate: all 19 tests fail
3. [ ] Define `MitreTactic` enum with all 17 variants + `#[non_exhaustive]`
4. [ ] Implement `fmt::Display for MitreTactic` with all 14 Enterprise + 3 ICS canonical strings
5. [ ] Implement `all_tactics_in_report_order()` returning `&'static [MitreTactic]` of length 17 in kill-chain order
6. [ ] Implement `technique_info(id: &str)` static match table with all 23 seeded entries returning `(name, tactic)` pairs (current; 25 after STORY-114 — PLANNED)
7. [ ] Implement `technique_name(id)` and `technique_tactic(id)` as thin projections over `technique_info`
8. [ ] Verify the technique catalog count is exactly 23 (current; 25 after STORY-114 — PLANNED; 11 Enterprise + 12 ICS)
9. [ ] Write edge-case tests for EC-001 through EC-005
10. [ ] Run `cargo test --all-targets` and `cargo clippy -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-069 | `Finding.mitre_technique: Option<String>` — String technique ID | Technique IDs are plain strings (e.g., `"T1027"`), not typed | Catalog count is 21 post-F2 (11 Enterprise + 10 ICS); was 15 pre-F2, was corrected from 16 at pass-8) |
| STORY-070 | Finding JSON serialization established | None fields absent from JSON | — |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Technique catalog count is exactly 23 (11 Enterprise + 12 ICS; current; 25 after STORY-114 — PLANNED) | BC-2.10.005 invariant 3 | Count seeded entries in match table; test `technique_name` resolves exactly 23 |
| `all_tactics_in_report_order` length is always 17 | BC-2.10.003 invariant 2 | Test: `assert_eq!(all_tactics_in_report_order().len(), 17)` |
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

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-05-21 | story-writer | Initial story decomposition |
| 1.2 | 2026-05-22 | story-writer | Wave 3 Ph3 implementer-confirm anchor-drift correction: `MitreTactic` enum anchor corrected from `:46-90` to `:46-66` (enum declaration only; Display impl is `:68-90`); `technique_info` closing-brace anchor corrected from `:122-155` to `:122-156` |
| 1.4 | 2026-05-22 | story-writer | Wave 3 wave-level adversarial fix F-2: status advanced draft → completed — STORY-071 delivered via PR #113, merge 991e821 |
| 1.3 | 2026-05-22 | story-writer | Wave 3 Ph3 pass-1 adversarial fixes: m-2 AC-008 trace postcondition 1→2; m-3 AC-009 trace postconditions 1,3 + joint-coverage note; m-5 technique_name line anchor :160-162; n-1 AC-002 test reference to standalone function; n-2 Task 2 count 16→19 |
| 1.6 | 2026-06-09 | story-writer | UPDATED (Feature #7 migration note): STORY-071 covers MITRE catalog lookup and `all_tactics_in_report_order`. STORY-100 (v0.3.0) seeds 6 new ICS technique arms (T0836, T0814, T0806, T0835, T0831, T0888) into `technique_info` and updates `SEEDED_TECHNIQUE_ID_COUNT` from 15 to 21. Test assertions in the STORY-071 scope that check seeded-ID count or enumerate seeded IDs are updated by STORY-100 to reflect 21 IDs. The `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>` field rename does not change the MITRE lookup API (technique_name/technique_tactic remain unchanged); only the VP-007 drift-guard grep pattern changes from `mitre_technique: Some` to `mitre_techniques: vec!`. Story status remains `completed`; no re-implementation required. |
| 1.7 | 2026-06-10 | story-writer | issue #222: AC-011 count corrected 15→21 (post-F2 + v19 remap). Full 21-ID enumeration updated: 11 Enterprise (T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004, T1083, T1499.002, T1505.003, T1573) + 10 ICS (T0846, T1692.001, T1692.002, T0885, T0836, T0814, T0806, T0835, T0831, T0888). Ensures T1692.001/T1692.002 appear (not revoked T0855/T0856). Test references updated from `test_technique_name_resolves_all_15_seeded_ids` to `test_technique_name_resolves_all_21_seeded_ids`. Narrative, BC table title, AC-014, Tasks 6+8, Previous Story Intelligence, and Architecture Compliance Rule updated to reflect count 21. |
| 1.8 | 2026-06-10 | story-writer | issue #222: corrected AC-014 tactic labels + AC-015 emitted count to match src/mitre.rs authoritative. AC-014: fixed four wrong tactic assignments (T1692.001/T1692.002 CommandAndControl→IcsImpairProcessControl; T0836 IcsInhibitResponseFunction→IcsImpairProcessControl; T0888 IcsImpairProcessControl→Discovery). AC-014 now lists all 21 IDs with exact MitreTactic:: variants cross-checked against technique_info lines 123-168. AC-015: emitted count corrected 6→13 (6 Enterprise + 7 ICS); ICS emitted IDs (T1692.001, T0836, T0814, T0806, T0835, T0831, T0888) added per EMITTED_IDS array lines 208-224 / BC-2.10.008. |
| 1.9 | 2026-06-13 | product-owner | Pass-12 corpus-cleanup F-C-P12-001/F-C-P12-002: Architecture Mapping table re-anchored to current src/mitre.rs line numbers. all_tactics_in_report_order :95-114→:100-120; technique_info :122-156→:128-182; technique_name :160-162→:186-188; technique_tactic :166-168→:192-194. |
| 1.10 | 2026-06-13 | product-owner | Pass-13 F-C-P13-003: reconcile stale counts to current BCs. Narrative 16 variants (14E+2 ICS) + 21 seeded → 17 variants (14E+3 ICS incl. IcsImpact) + 23 seeded (current; 25 after STORY-114 PLANNED). AC-005/008/009 "16"→"17"; AC-007 ICS positions [14],[15]→[14],[15],[16]+IcsImpact; AC-010/011 test ref 21→23; AC-011 ICS split 10→12 with T1691.001+T0827 added; AC-014 "21"→"23" + ICS 10→12 + T1691.001/T0827 assignments added; Tasks 3-6,8 updated 16/21→17/23; Architecture Compliance Rules 21/16→23/17. |
