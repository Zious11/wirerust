# Demo Evidence Report — STORY-188

**Story:** STORY-188: S7comm Job/Ack_Data Function-Code Classification (v1.7, AC-188-001..011)
**Wave:** 91
**Date:** 2026-10-05
**Branch:** feature/STORY-188-s7comm-function-code-classification (HEAD c6bd91e3; per-story adversarial CONVERGED)
**Product type:** Library (pure-core `classify_job_ack_function` consumed by the effectful-shell
`S7commAnalyzer` in `src/analyzer/s7comm.rs`). There is no CLI subcommand or web UI surface for
S7comm yet (dispatcher wiring is STORY-193), so, following the STORY-187 precedent, the
demonstration vehicle is this story's own test harness: `tests/s7comm_analyzer_tests.rs`,
`mod story_188` (34 tests plus one Kani harness).
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test s7comm_analyzer_tests`,
filtered per behavior group, piped through `grep` to show only the relevant `story_188::` lines)

---

## Full story_188 Module: 34/34 PASS

```
cargo test --test s7comm_analyzer_tests story_188::
```

Result: **34 passed; 0 failed; 0 ignored; 0 measured; 83 filtered out** (the 83 are the
`story_186` and `story_187` modules in the same file). Artifact: `AC-ALL-story_188-green.gif/.webm/.tape`.

---

## AC → Test → Artifact Coverage Map

| AC | Title (short) | Test(s) | Artifact | Path | Verdict |
|----|---------------|---------|----------|------|---------|
| AC-188-001 | FC 0xF0 -> Setup Communication | `test_BC_2_21_010_setup_communication_classified`; `canonical::test_BC_2_21_010_canonical_setup_communication_ack_data_classified` | `AC-001-002-setup-read`; `AC-011-canonical-frames`; `AC-FIXTURE-e2e-fc-classification-pcap` | Success | PASS |
| AC-188-002 | FC 0x04 -> Read Var, no area decode | `test_BC_2_21_011_read_var_classified_no_area_decode`; `canonical::test_BC_2_21_011_canonical_read_var_db_classified` | `AC-001-002-setup-read`; `AC-011-canonical-frames` | Success | PASS |
| AC-188-003 | FC 0x05 -> Write Var, first-item area code | `test_BC_2_21_012_write_var_area_code_extraction`; `test_BC_2_21_012_write_var_descriptor_length_boundary_11_12_13_14`; `area_exhaustive::test_BC_2_21_012_write_var_area_code_exhaustive_over_all_u8` | `AC-003-write-var-area` | Success + Error (descriptor lengths 11/12/13 below the 14-byte minimum are rejected; 14 accepted) + exhaustive u8 | PASS |
| AC-188-004 | Download triad classified independently | `test_BC_2_21_013_download_triad_classified_independently`; `vp054::proptest_vp054_download_upload_structural_disjointness` | `AC-004-005-download-upload-triads` | Success | PASS |
| AC-188-005 | Upload triad, disjoint from Download | `test_BC_2_21_014_upload_triad_classified_disjoint_from_download`; `vp054::proptest_vp054_download_upload_structural_disjointness` | `AC-004-005-download-upload-triads` | Success + disjointness (a Download FC never classifies as Upload and vice versa) | PASS |
| AC-188-006 | FC 0x28 PLC Control, PI-service decode | `test_BC_2_21_015_plc_control_service_string_decode`; `test_BC_2_21_015_plc_control_truncated_and_case_variants_unrecognized`; `test_BC_2_21_015_plc_control_trailing_bytes_after_service_string`; `canonical::test_BC_2_21_015_canonical_plc_control_p_program_classified` | `AC-006-007-plc-control-stop`; `AC-011-canonical-frames` | Success + Error (truncated service string, case variants -> unrecognized service) | PASS |
| AC-188-007 | FC 0x29 -> PLC Stop by FC byte only | `test_BC_2_21_016_plc_stop_classified`; `canonical::test_BC_2_21_016_canonical_plc_stop_classified` | `AC-006-007-plc-control-stop`; `AC-011-canonical-frames` | Success (FC-only; `param_length==1` -> PlcStop) | PASS |
| AC-188-008 | Unrecognized FC and empty block are distinct terminal outcomes | `test_BC_2_21_017_unrecognized_fc_and_empty_parameter_block`; `test_BC_2_21_017_unsliceable_parameter_block_returns_no_parameter_block` | `AC-008-009-unrecognized-totality`; `AC-VP051-kani-slicing-safe` (supporting) | Error / edge (unrecognized FC, empty block, unsliceable block) | PASS |
| AC-188-009 | Classification total over all 256 u8 values + empty block | `test_BC_2_21_017_fc_classification_total_over_all_256_values`; `vp052::proptest_vp052_fc_classification_totality` | `AC-008-009-unrecognized-totality`; `AC-VP051-kani-slicing-safe` (supporting) | Exhaustive 256-value + proptest | PASS |
| AC-188-010 | Ack/Ack_Data error_class/error_code -> bounded analyzer-side record + count map | eleven `test_BC_2_21_008_*` tests: `ack_error_class_code_consumed_and_logged`, `ack_data_error_class_code_consumed_and_logged`, `zero_error_class_code_logged_for_ack_and_ack_data`, `ack_error_observation_captures_pdu_reference`, `ack_error_counts_exact_for_mixed_frames`, `ack_error_histogram_counts_beyond_list_cap`, `ack_error_observations_bounded_by_cap_with_dropped_count`, `job_frames_record_no_ack_error_observation`, `userdata_frames_record_no_ack_error_observation`, `job_frames_contribute_no_histogram_key`, `bounds_failing_ack_data_records_no_ack_error_observation` | `AC-010-ack-error-record` | Success (record, count map, cap/dropped, pdu_reference) + Error/negative (Job/Userdata and bounds-failing frames record nothing) | PASS |
| AC-188-011 | Canonical public-reference byte vectors | `canonical::` six tests: Setup Comm Ack_Data (010), Write Var DB (012), Write Var Outputs (012), Read Var DB (011), PLC Control P_PROGRAM (015), PLC Stop (016) | `AC-011-canonical-frames` | Success | PASS |

Additional artifacts: `AC-FIXTURE-e2e-fc-classification-pcap` runs
`test_BC_2_21_010_fc_classification_fixture_pcap_end_to_end` (committed fixture
`tests/fixtures/s7comm-fc-classification.pcap`, classified FCs through `S7commAnalyzer::on_data`;
supports AC-188-001..008). `AC-VP051-kani-slicing-safe` records the VP-051 Kani proof (below).

**All 11 acceptance criteria (AC-188-001..011) map to at least one recorded artifact.**

### Per-recording counts

| Artifact | ACs | Tests shown |
|----------|-----|-------------|
| `AC-001-002-setup-read` | 001, 002 | 2 |
| `AC-003-write-var-area` | 003 | 3 |
| `AC-004-005-download-upload-triads` | 004, 005 | 3 |
| `AC-006-007-plc-control-stop` | 006, 007 | 4 |
| `AC-008-009-unrecognized-totality` | 008, 009 | 4 |
| `AC-010-ack-error-record` | 010 | 11 |
| `AC-011-canonical-frames` | 011 | 6 |
| `AC-FIXTURE-e2e-fc-classification-pcap` | 001..008 (supporting) | 1 |
| `AC-ALL-story_188-green` | 001..011 | 34 (de-duplicated total; 2+3+3+4+4+11+6+1 = 34) |
| `AC-VP051-kani-slicing-safe` | 008, 009 (supporting) | Kani harness |

---

## VP-051 Kani Harness (recorded)

`cargo kani --tests --harness verify_classify_job_ack_function_param_slicing_safe` (the
`#[cfg(kani)] mod vp051_kani` harness in `tests/s7comm_analyzer_tests.rs`) completes in about one
second of verification time locally: **VERIFICATION SUCCESSFUL; 4 of 4 cover properties satisfied;
1 successfully verified harness, 0 failures**. Recorded in `AC-VP051-kani-slicing-safe.gif/.webm`.
Scope per story v1.6/EC-012: the harness covers inputs that pass `s7comm_bounds_ok`; the unsliceable
`NoParameterBlock` path is covered by the unit test cited under AC-188-008.

---

## Notes on the Human Rulings (why AC-188-010 evidence is test-assertion based)

Human ruling 2026-10-04 (STORY-188 per-story adversarial pass 1): the AC-188-010 surface is an
**analyzer-side bounded record with NO stderr or log output** (ADR-0004 flooding rationale).
`S7commAnalyzer` keeps a bounded observation list (cap `MAX_S7_ACK_ERROR_OBSERVATIONS` = 1024, with
a saturating dropped count), an exact per-`(rosctr, error_class, error_code)` count map that keeps
counting beyond the cap, and a `pdu_reference` on each observation. Recording is bounds-gated: Job
and Userdata frames, and Ack/Ack_Data frames that fail the BC-2.21.009 bounds check, record
nothing. Because nothing is printed, a terminal recording has no visible side effect to show, so
the evidence for AC-188-010 is the passing assertions on the analyzer's accessors, named per
behavior in the `AC-010-ack-error-record` recording (11 tests). A zero error class/code is recorded
the same as any other value (BC-2.21.008 EC-004). For Write Var (AC-188-003) the 0xFF area-code
collision is an accepted residual (orchestrator decision, story v1.3).

Classification in `dispatch_classic_s7comm` is a deliberate classification-only placeholder
consumed by STORY-191/192, so no Finding or user-visible output is produced by this story; this
is why no CLI recording exists.

## Canonical-Frame Provenance (AC-188-011, DF-CANONICAL-FRAME-HOLDOUT-001)

The six `story_188::canonical::*` byte vectors are sourced from public references recorded in the
story's canonical-FC-vectors research document (Write Var, PLC Control, PLC Stop, Read Var), plus
the Setup Communication Ack_Data vector cited in BC-2.21.008 Canonical Test Vectors. Per ADR-014
Decision 4, public wire-capture bytes are used as test vectors only; parser design derives from
the permitted prose sources.

## Recording Method

VHS tapes run `cargo test --test s7comm_analyzer_tests [--] <filters> 2>&1 | grep -E
'running [0-9]+ tests|story_188::|test result:'`. The `grep` suppresses cargo's
`Running tests/... (<worktree path>/target/...)` line, which would otherwise leak an absolute host
path into rendered output. Tapes use Menlo, Dracula theme, 1200 px wide, matching STORY-187.
Multiple filters are passed after `--` because cargo accepts only one positional filter before it.

## Path-Scrub Gate (PG-W70-DEMO-SCRUB)

The mandatory absolute-host-path and tilde-home grep from the demo-evidence scrub-gate maintenance
document was run from the repo root against the demo-evidence tree and returned zero matches. The
gate command is not reproduced verbatim here because its regex literal would match this file.
