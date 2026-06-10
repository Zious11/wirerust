# Red Gate Log — STORY-100 + STORY-101 (v0.3.0 Multi-Tag Migration)

**Date:** 2026-06-09
**Worktree:** `.worktrees/story-100-multitag`
**Branch:** `feature/story-100-multitag-migration`
**Test file:** `tests/bc_2_09_100_multitag_tests.rs`

## Red Gate Status: CONFIRMED

`cargo build --all-targets` fails with **13 compile errors** — all from
`tests/bc_2_09_100_multitag_tests.rs`. The existing code compiles cleanly; only
the new test file introduces failures, which is the intended Red Gate state.

## Compile Error Summary

```
error[E0560]: struct `Finding` has no field named `mitre_techniques`   (×2)
error[E0609]: no field `mitre_techniques` on type `Finding`            (×11)

error: could not compile `wirerust` (test "bc_2_09_100_multitag_tests") due to 13 previous errors
```

All 13 errors originate in `tests/bc_2_09_100_multitag_tests.rs`. The root
cause is that `Finding` still declares `mitre_technique: Option<String>` at
`src/findings.rs:135`. The migration to `mitre_techniques: Vec<String>` is
the implementer's task.

## Test Coverage

The file covers all BCs for both STORY-100 and STORY-101 (atomically delivered
as v0.3.0 because the Rust compiler forces the type change in lockstep):

| Test function | BC / AC | Red reason |
|---|---|---|
| `test_BC_2_09_001_constructs_finding_with_multi_technique_vec` | BC-2.09.001 pc1 | `mitre_techniques` field missing |
| `test_BC_2_09_001_constructs_finding_with_empty_techniques_vec` | BC-2.09.001 pc1 | field missing |
| `test_BC_2_09_001_constructs_finding_with_singleton_technique_vec` | BC-2.09.001 EC-002 | field missing |
| `test_BC_2_09_001_invariant_field_is_vec_not_option` | BC-2.09.001 inv6 | field missing |
| `test_BC_2_09_001_no_option_string_field_in_source` | BC-2.09.001 AC-008 | runtime grep fails |
| `test_BC_2_09_001_all_emission_sites_use_vec_field` | BC-2.09.001 AC-002 | runtime grep finds stale sites |
| `test_BC_2_09_006_empty_vec_produces_absent_mitre_techniques_key` | BC-2.09.006 EC-001 | field missing (compile) |
| `test_BC_2_09_006_single_technique_serializes_as_json_array` | BC-2.09.006 EC-002 | field missing (compile) |
| `test_BC_2_09_006_multi_technique_serializes_as_json_array` | BC-2.09.006 EC-006 | field missing (compile) |
| `test_BC_2_09_006_no_scalar_mitre_technique_key_in_json` | BC-2.09.006 inv4 | field missing (compile) |
| `test_BC_2_10_005_technique_name_resolves_all_21_seeded_ids` | BC-2.10.005 pc3 | T0836/T0814/T0806/T0835/T0831/T0888 → None |
| `test_BC_2_10_005_technique_name_resolves_t0888_remote_system_info_discovery` | BC-2.10.005 EC-007 | T0888 → None |
| `test_BC_2_10_005_technique_name_resolves_t0836_modify_parameter` | BC-2.10.005 EC-008 | T0836 → None |
| `test_BC_2_10_005_seeded_technique_id_count_is_21` | BC-2.10.005 inv3 | source has count=15 |
| `test_BC_2_10_007_technique_tactic_correct_for_all_21_seeded_ids` | BC-2.10.007 pc2 | 6 new ICS arms missing |
| `test_BC_2_10_007_t0888_maps_to_discovery_tactic` | BC-2.10.007 EC-004 | T0888 → None |
| `test_BC_2_10_007_t0806_maps_to_ics_impair_process_control` | BC-2.10.007 EC-005 | T0806 → None |
| `test_BC_2_10_007_t0814_maps_to_ics_inhibit_response_function` | BC-2.10.007 EC-006 | T0814 → None |
| `test_BC_2_10_008_all_emitted_ids_resolve_in_lookup` | BC-2.10.008 pc1 (VP-007) | 7 ICS IDs missing |
| `test_BC_2_10_008_t0846_seeded_but_not_in_emitted_set` | BC-2.10.008 inv4 | EMITTED_IDS not yet 13 |
| `test_BC_2_10_008_vp007_grep_comment_updated_to_new_field_name` | BC-2.10.008 inv3 | comment not updated |
| `test_BC_2_10_008_vp007_new_ics_ids_resolve_positive_coverage` | VP-007 coverage | new ICS IDs missing |
| `test_BC_2_11_001_json_report_envelope_has_mitre_domain_and_version` | BC-2.11.001 pc2+7+8 | envelope keys missing in JSON |
| `test_BC_2_11_001_envelope_fields_present_with_zero_findings` | BC-2.11.001 EC-001 | envelope keys missing |
| `test_BC_2_11_001_mitre_attack_version_constant_has_f4_pin_flag_comment` | BC-2.11.001 AC-FLAG-001 | constant missing from json.rs |
| `test_BC_2_11_013_terminal_tactic_grouping_uses_first_technique` | BC-2.11.013 AC-003 | field missing (compile) |
| `test_BC_2_11_015_terminal_empty_techniques_lands_in_uncategorized` | BC-2.11.015 AC-004 | field missing (compile) |
| `test_BC_2_11_015_terminal_unknown_id_lands_in_uncategorized` | BC-2.11.015 AC-004 | field missing (compile) |
| `test_BC_2_11_017_terminal_renders_multi_id_mitre_string` | BC-2.11.017 AC-002 | field missing (compile) |
| `test_BC_2_11_017_terminal_singleton_technique_render_unchanged` | BC-2.11.017 AC-002 | field missing (compile) |
| `test_BC_2_11_017_terminal_empty_vec_produces_no_mitre_line` | BC-2.11.017 AC-002 | field missing (compile) |
| `test_BC_2_11_020_csv_header_column_6_is_mitre_techniques` | BC-2.11.020 pc3 | header still says mitre_technique |
| `test_BC_2_11_020_csv_has_no_envelope_fields` | BC-2.11.001/AC-008 | passes today (good) |
| `test_BC_2_11_024_csv_empty_technique_is_empty_string` | BC-2.11.024 EC-001 | field missing (compile) |
| `test_BC_2_11_024_csv_singleton_technique_is_plain_id` | BC-2.11.024 EC-002 | field missing (compile) |
| `test_BC_2_11_024_csv_multi_technique_semicolon_join` | BC-2.11.024 pc | field missing (compile) + no semicolon join |
| `test_BC_2_11_024_csv_three_technique_semicolon_join` | BC-2.11.024 | field missing (compile) |
| `test_BC_2_11_020_csv_column_count_stays_9_with_multitag` | BC-2.11.020 | field missing (compile) |
| `test_BC_2_09_001_singleton_vec_json_output_is_array_not_scalar` | AC-010 regression | field missing (compile) |
| `test_BC_2_09_001_singleton_vec_csv_output_byte_identical_to_pre_migration` | AC-010 regression | field missing (compile) |
| `test_BC_2_09_001_singleton_vec_terminal_output_byte_identical_to_pre_migration` | AC-010 regression | field missing (compile) |
| `test_BC_2_09_001_vp021_helper_pattern_uses_vec_field` | AC-011 (VP-021) | field missing (compile) |

Total: **43 new tests** all RED.

## Existing Test Files Requiring Migration (AC-010)

The following test files contain `mitre_technique:` literals that will become
compile errors after STORY-100 renames the field. The implementer must update
these files as part of the migration (tasks 13–15 in STORY-100):

| File | Old pattern count | Migration needed |
|------|------------------|-----------------|
| `tests/findings_tests.rs` | 2 constructions | `mitre_technique: Some(...)` → `mitre_techniques: vec![...]`; `None` → `vec![]` |
| `tests/reporter_tests.rs` | 2 constructions | same |
| `tests/reporter_json_tests.rs` | 1 construction | same |
| `tests/reporter_csv_tests.rs` | ~8 sites | same + column 6 header assertion update |
| `tests/reporter_terminal_tests.rs` | ~5 sites | same |
| `tests/mitre_tests.rs` | 1 comment + test update | update `test_all_emitted_ids_resolve` to 13 IDs |
| `tests/reassembly_engine_tests.rs` | ~10 sites | same |
| `tests/tls_analyzer_tests.rs` | ~6 sites | same |
| `tests/tls_integration_tests.rs` | 2 sites | same |
| `tests/timestamp_threading_tests.rs` | 2 constructions | same |
| `tests/http_analyzer_tests.rs` | check for any | same |
| `tests/cli_integration_tests.rs` | check for any | same |

Additionally:
- `tests/reporter_json_tests.rs`: `test_BC_2_11_001_top_level_keys` currently
  asserts exactly 3 top-level keys (`["analyzers", "findings", "summary"]`).
  After STORY-101 this must be updated to 5 keys (adding `mitre_attack_version`
  and `mitre_domain`).

## F4-PIN Obligation

`mitre_attack_version = "ics-attack-v15"` is a placeholder. Before the v0.3.0
release tag, verify the authoritative ATT&CK for ICS version at
https://attack.mitre.org/resources/attack-data-and-tools/ and update the
constant in `src/reporter/json.rs`.

## Hand-Off to Implementer

All 43 tests fail. Red Gate is verified. Proceed with STORY-100 implementation
following the canonical edit order from `f2-decomposition-sequencing.md §4.1`:

1. Rename field in `src/findings.rs`
2. Run `cargo check` to get the exhaustive broken-site list (~21 emission sites)
3. Fix all production code: `src/reporter/csv.rs`, `src/reporter/terminal.rs`,
   `src/reporter/json.rs`, `src/analyzer/http.rs`, `src/analyzer/tls.rs`,
   `src/reassembly/mod.rs`, `src/reassembly/lifecycle.rs`
4. Add 6 new ICS arms to `src/mitre.rs`; update constants; update EMITTED_IDS
5. Fix test files (existing stories' scope + the reporter_json_tests.rs key-count)
6. Green Gate: `cargo test --all-targets` passes with no regressions
