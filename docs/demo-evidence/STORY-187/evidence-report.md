# Demo Evidence Report — STORY-187

**Story:** STORY-187: S7comm Flow State Completion, Four-Way protocol_id Dispatch
Skeleton, and parse_s7comm_header Pure-Core Parser
**Wave:** 90
**Date:** 2026-09-25
**Branch:** feature/STORY-187-s7comm-header-dispatch
**Product type:** Library (pure-core parser consumed by the effectful-shell
`S7commAnalyzer` in `src/analyzer/s7comm.rs`) — there is no CLI subcommand or web UI
surface for S7comm yet (dispatcher wiring to the CLI is deferred to STORY-193). The
demonstration vehicle is this story's own test harness: `tests/s7comm_analyzer_tests.rs`,
`mod story_187` (63 tests) is the executable proof of each acceptance criterion.
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test
s7comm_analyzer_tests`, filtered per behavior group)

---

## Full story_187 Module: 63/63 PASS

Command:
```
cargo test --test s7comm_analyzer_tests story_187::
```

Result: **63 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out** (the 20 filtered
out are STORY-186's `story_186` module tests in the same file). The full file
(`story_186` + `story_187`) is **83 passed; 0 failed**, finishing in ~4.8s wall clock
(dominated by STORY-186's three VP-050 proptests; every `story_187` test finishes in
well under a millisecond).

| Artifact | Description |
|----------|-------------|
| `AC-ALL-story_187-green.gif` / `.webm` | Full `story_187` module — 63/63 green |
| `AC-ALL-story_187-green.tape` | VHS script for the full-module run |

---

## AC → Test → Artifact Coverage Map

| AC | Title | BC | Test(s) | Artifact | Path | Verdict |
|----|-------|-----|---------|----------|------|---------|
| AC-187-001 | `S7commFlowState` carries the full field set required by this story's scope | BC-2.21.001 PC-1, PC-2 | `test_BC_2_21_001_flow_state_field_set`, `test_BC_2_21_001_fields_constructible_with_expected_types` | `AC-001-003-flow-state-session.gif/.webm` | Success | PASS |
| AC-187-002 | `S7commFlowState` is created lazily on first `on_data` call | BC-2.21.001 PC-3 | `test_BC_2_21_001_lazy_flow_state_creation`, `test_BC_2_21_001_never_touched_flow_state_never_created` | `AC-001-003-flow-state-session.gif/.webm` | Success | PASS |
| AC-187-003 | `session_established` is set only by an opposite-direction CC after a CR; five negative sequences + most-recent-CR-wins + monotonicity | BC-2.21.001 PC-1, EC-004..EC-009; BC-2.21.002 PC-2 | `test_BC_2_21_001_cr_then_opposite_cc_sets_session_established`, `test_BC_2_21_001_cr_only_session_not_established`, `test_BC_2_21_001_cc_only_no_prior_cr_session_not_established`, `test_BC_2_21_001_cc_before_cr_session_not_established`, `test_BC_2_21_001_same_direction_cc_session_not_established`, `test_BC_2_21_001_repeated_same_direction_cr_session_not_established`, `test_BC_2_21_001_most_recent_cr_direction_wins`, `test_BC_2_21_001_session_established_is_monotonic`, `test_BC_2_21_001_dedup_flags_independent_c2s` | `AC-001-003-flow-state-session.gif/.webm` | Success (all sequences are non-classifying; no error path — CR/CC session tracking has no malformed-input branch) | PASS |
| AC-187-004 | `Some(0x32)` DT frames dispatch to classic dissection when sticky `classified_protocol` is/becomes Classic; malformed too-short frame on a Classic flow emits exactly one T0814 | BC-2.21.002 PC-3; BC-2.21.004 PC-4 | `test_BC_2_21_002_classic_s7comm_dispatch_asserts_classified_protocol_classic` (success), `test_BC_2_21_002_malformed_0x32_frame_on_classic_flow_emits_exactly_one_t0814` (error) | `AC-004-005-012-dispatch-classification.gif/.webm` | Success + Error | PASS |
| AC-187-005 | First DT frame's `protocol_id: Some(byte)` sets `classified_protocol` exactly once (sticky first-write-wins); `protocol_id: None` never classifies | BC-2.21.002 PC-6, EC-003, EC-004 | `test_BC_2_21_002_sticky_first_classification`, `test_BC_2_21_002_none_protocol_id_dt_first_then_0x32_dt_classifies_classic`, `test_BC_2_21_002_two_frames_one_delivery_first_write_wins`, `test_BC_2_21_002_empty_dt_followed_by_frame_same_delivery_stays_unclassified`, `test_BC_2_21_002_unparseable_cotp_does_not_classify` | `AC-004-005-012-dispatch-classification.gif/.webm` | Success (classifying paths) + Error (unparseable-COTP non-classifying path) | PASS |
| AC-187-006 | `parse_s7comm_header` returns `None` for `data.len() < 10`; no panic; one T0814 per direction with boundary-specific evidence | BC-2.21.004 PC-1, PC-2, PC-4 | `test_BC_2_21_004_parse_returns_none_for_len_lt_10`, `test_BC_2_21_004_len_shorter_than_10_returns_none_and_emits_t0814_once`, `test_BC_2_21_004_len_shorter_than_10_emits_t0814_once_s2c`, `test_BC_2_21_004_nine_byte_payload_on_data_too_short_evidence` | `AC-006-009-length-rosctr-parser.gif/.webm` | Error (too-short header is definitionally the malformed condition) | PASS |
| AC-187-007 | `parse_s7comm_header` defensively rejects `data[0] != 0x32` — no `Finding`, caller-hygiene only | BC-2.21.005 PC-1, PC-3 | `test_BC_2_21_005_defensive_reject_wrong_protocol_id_byte`, `test_BC_2_21_005_defensive_reject_zero_byte` | `AC-006-009-length-rosctr-parser.gif/.webm` | Defensive-reject (no on-wire error path — this is a caller-hygiene contract, not an anomaly) | PASS |
| AC-187-008 | `parse_s7comm_header` extracts common header fields for Job/Userdata at the correct byte offsets; Reserved bytes never gate | BC-2.21.006 PC-1..5 | `test_BC_2_21_006_common_header_field_extraction`, `test_BC_2_21_006_nonzero_reserved_bytes_do_not_reject`, `test_BC_2_21_006_byte_asymmetric_big_endian_decode`, `test_BC_2_21_006_canonical_setup_communication_job_frame_on_data` | `AC-006-009-length-rosctr-parser.gif/.webm` | Success | PASS |
| AC-187-009 | `parse_s7comm_header` returns `None` for unrecognized ROSCTR; totality over all 256 `u8` values; shared dedup flag, s2c independence | BC-2.21.007 PC-1..3 | `test_BC_2_21_007_unrecognized_rosctr_returns_none`, `test_BC_2_21_007_shares_dedup_flag_with_004_malformed_header`, `test_BC_2_21_007_unrecognized_rosctr_emits_t0814_once_s2c`, `proptest_bc_2_21_007_rosctr_byte_totality_over_all_256_values` | `AC-006-009-length-rosctr-parser.gif/.webm` | Error (unrecognized ROSCTR is the malformed condition) + exhaustive totality | PASS |
| AC-187-010 | ROSCTR=Ack AND Ack_Data both require 12 bytes, extract `error_class`/`error_code`; truncated Ack/Ack_Data (10/11 bytes) returns `None` with distinguishable evidence | BC-2.21.008 PC-1..4, EC-007 | `test_BC_2_21_008_ack_rosctr_12_byte_minimum_and_error_fields`, `test_BC_2_21_008_canonical_ack_vector_verbatim`, `test_BC_2_21_008_ack_data_12_byte_header_and_error_fields`, `test_BC_2_21_008_error_fields_none_for_job_and_userdata_rosctr`, `test_BC_2_21_008_ack_data_nonzero_error_fields_with_parameter_block` (success); `test_BC_2_21_008_truncated_ack_data_returns_none`, `test_BC_2_21_008_truncated_ack_on_data_emits_t0814_once`, `test_BC_2_21_008_truncated_ack_data_on_data_emits_t0814_once` (error) | `AC-010-011-ack-bounds-check.gif/.webm` | Success + Error | PASS |
| AC-187-011 | Declared `param_length`/`data_length` bounds-checked before any slice access; dedup verified for both flow directions; never borrows bytes from a trailing frame | BC-2.21.009 PC-1..3 | `test_BC_2_21_009_bounds_check_before_parameter_data_slice`, `test_BC_2_21_009_bounds_check_dedup_s2c`, `test_BC_2_21_009_ack_header_len_12_bounds_check`, `test_BC_2_21_009_data_length_overrun_on_data_emits_t0814`, `test_BC_2_21_009_dissection_bounded_to_own_tpkt_frame`, `test_BC_2_21_009_bounds_failure_evidence_reports_declared_and_available`, `test_BC_2_21_009_bounds_failure_evidence_ack_data_header_len_12` (error); `test_BC_2_21_009_bounds_check_passes_exact_match`, `test_BC_2_21_009_empty_parameter_and_data_blocks_trivial_pass`, `test_BC_2_21_009_overflow_free_arithmetic_max_values`, `test_BC_2_21_009_s7comm_bounds_ok_data_length_only_overrun`, `test_BC_2_21_009_s7comm_bounds_ok_helper_matches_bounds_decision`, `proptest_bc_2_21_006_008_some_iff_rosctr_and_length_conditional` (success) | `AC-010-011-ack-bounds-check.gif/.webm` | Success + Error | PASS |
| AC-187-012 | Classic dissection is gated on sticky `classified_protocol == Classic` **AND** current frame `protocol_id == Some(0x32)` — a conjunction, neither alone sufficient | BC-2.21.002 PC-3, Inv-4, EC-005 | `test_BC_2_21_002_0x32_dt_frame_not_dissected_when_sticky_classified_plus_or_unclassified`, `test_BC_2_21_002_non_0x32_dt_frame_on_classic_flow_not_dissected` | `AC-004-005-012-dispatch-classification.gif/.webm` | Negative-gate proof (both are "must NOT dissect" assertions — the gate's own error/reject path) | PASS |
| AC-187-013 | Canonical, independently-sourced classic S7comm Setup Communication frame pair (Job + Ack_Data) and committed fixture pcap validate parser and dispatch | BC-2.21.002 PC-3; BC-2.21.006; BC-2.21.008 PC-1-2; policy DF-CANONICAL-FRAME-HOLDOUT-001 | `test_BC_2_21_006_canonical_setup_communication_job_frame_on_data`, `test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data`, `test_BC_2_21_002_setup_comm_fixture_pcap_well_formed_no_findings` | `AC-013-canonical-frames.gif/.webm` | Success (all three assert zero `Finding`s on well-formed canonical/fixture input — AC-187-013 has no malformed-input scenario of its own; the malformed-header error paths for the SAME byte layouts are covered under AC-187-006/009/010/011) | PASS |

**All 13 acceptance criteria (AC-187-001..013) are covered by at least one recorded
artifact.** `proptest_vp053_protocol_id_dispatch_totality` (VP-053) is additionally
captured in `AC-004-005-012-dispatch-classification.gif/.webm` as supporting evidence
for AC-187-005's four-way classification table.

### Note on artifact overlap

`test_BC_2_21_002_setup_comm_fixture_pcap_well_formed_no_findings` (AC-187-013) shares
BC-2.21.002's dispatch path with AC-187-004/005/012, so the `BC_2_21_002` filter used
for `AC-004-005-012-dispatch-classification` also happens to catch it; likewise
`test_BC_2_21_006_canonical_setup_communication_job_frame_on_data` (AC-187-013) and
`test_BC_2_21_008_canonical_ack_vector_verbatim`/
`test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data` appear
inside the broader `BC_2_21_006`/`BC_2_21_008` filters used for
`AC-006-009-length-rosctr-parser` and `AC-010-011-ack-bounds-check` respectively. This
is intentional context, not a substitute for AC-187-013's own dedicated recording — the
coverage map above cites `AC-013-canonical-frames.gif/.webm` as the primary artifact for
AC-187-013 because it is the only recording that isolates all three of that AC's tests
together with their source-provenance commentary.

---

## Error-Path Coverage Summary

Per-story convention (matching STORY-186), the five malformed-header conditions that
share the `malformed_header_reported_c2s`/`_s2c` dedup flag are each an explicit error
path, distinguished by reason-specific evidence text:

| Reason | AC | Error-path test(s) |
|--------|----|--------------------|
| Header shorter than 10 bytes | AC-187-006 | `test_BC_2_21_004_len_shorter_than_10_returns_none_and_emits_t0814_once`, `test_BC_2_21_004_len_shorter_than_10_emits_t0814_once_s2c`, `test_BC_2_21_004_nine_byte_payload_on_data_too_short_evidence` |
| Unrecognized ROSCTR byte | AC-187-009 | `test_BC_2_21_007_unrecognized_rosctr_emits_t0814_once_s2c`, `test_BC_2_21_007_shares_dedup_flag_with_004_malformed_header` |
| Truncated Ack (10/11 bytes) | AC-187-010 | `test_BC_2_21_008_truncated_ack_on_data_emits_t0814_once` |
| Truncated Ack_Data (10/11 bytes) | AC-187-010 | `test_BC_2_21_008_truncated_ack_data_on_data_emits_t0814_once` |
| Bounds-check failure (`param_length`/`data_length` exceed available bytes) | AC-187-011 | `test_BC_2_21_009_bounds_check_before_parameter_data_slice`, `test_BC_2_21_009_bounds_check_dedup_s2c`, `test_BC_2_21_009_data_length_overrun_on_data_emits_t0814`, `test_BC_2_21_009_bounds_failure_evidence_reports_declared_and_available`, `test_BC_2_21_009_bounds_failure_evidence_ack_data_header_len_12` |

Every one of these five reason classes is exercised for BOTH flow directions
independently (c2s and s2c dedup verified separately, per the story's explicit
"dedup verified for both flow directions" language in AC-187-006/009/011), and the
shared `assert_reason_specific_evidence` test helper confirms the five reasons remain
textually distinguishable from one another.

AC-187-001/002/003/007/012/013 have no on-wire "anomaly" error path of their own (CR/CC
session bookkeeping, the `data[0] != 0x32` caller-hygiene defensive-reject, the
classification gate's negative case, and the canonical/fixture well-formed-input checks
are all non-`Finding`-emitting by design) — their negative/defensive assertions are
covered as noted in the coverage map above.

---

## VP-051 Kani Obligation (executed locally, supporting evidence — no recording)

Per the story's VP-051 obligation, two separate `#[kani::proof]` harnesses are anchored
in `tests/s7comm_analyzer_tests.rs`'s `#[cfg(kani)] mod vp051_kani`:

- `verify_parse_s7comm_header_bounds_safety` — header-extraction half (bounded symbolic
  `data: [u8; 16]` + `len`); `Some`/`None` totality and positive field-extraction
  obligations.
- `verify_s7comm_bounds_ok_bounds_safety` — bounds-check half (independent symbolic
  `data_len: usize` against `s7comm_bounds_ok`); exact-equality and no-panic
  obligations.

Both harnesses were executed locally with `cargo kani` and returned **VERIFICATION
SUCCESSFUL** for both proofs. This is text-only supporting evidence per the demo-
recording scope — Kani symbolic execution output is not a VHS-recordable interactive
CLI demo, and formal-verification artifacts are the `formal-verifier` agent's
deliverable, not the demo-recorder's. No recording was produced for this obligation.

## cargo-mutants Results (supporting evidence — no recording)

Mutation testing for this story's scope reported **43 mutants generated**: all viable,
non-equivalent mutants were killed by the test suite; 1 mutant was classified equivalent
(no test can distinguish it from correct behavior); 2 mutants were unviable (did not
compile). This is text-only supporting evidence per the demo-recording scope — mutation
testing summary tables are the `formal-verifier` agent's deliverable, not a VHS-
recordable interactive CLI demo. No recording was produced for this obligation.

---

## Canonical-Frame Provenance (AC-187-013, policy DF-CANONICAL-FRAME-HOLDOUT-001)

The TPKT length field, the `02 F0 80` COTP DT header bytes, and the S7comm common-
header/Setup-Communication-parameter bytes used in `test_BC_2_21_006_canonical_setup_communication_job_frame_on_data`
and `test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data` are NOT
derived from this story's own behavioral contracts, ADR-014, or any other project
artifact. Sourcing (STORY-187 per-story adversarial pass 5, F-40, human ruling
2026-09-24):

- **Primary citation:** cnblogs, "西门子S7通讯协议引用整理"
  (https://www.cnblogs.com/crcce-dncs/p/10659087.html) — Setup Communication
  request/response byte sequence, including the full 27-byte Ack_Data response frame
  `03 00 00 1B 02 F0 80 32 03 00 00 FF FF 00 08 00 00 00 00 F0 00 00 01 00 01 00 F0`.
- **Corroborating sources (test-vector sourcing only):** Yiqisoft
  (https://www.yiqisoft.cn/blogs/IoT-Gateway/363.html) and the Inductive Automation KB
  ("Loggers - Device Connections: Siemens").

Per ADR-014 Decision 4's 2026-09-24 reconciliation note, publicly posted wire-capture
byte examples are permitted as **test-vector sources only** — parser design and field
semantics for this story continue to derive exclusively from Decision 4's prose sources
(Wireshark wiki, Kleinmann & Wool 2014, Orange-Cyberdefense catalog) and permitted
design references (cisagov/icsnpp-s7comm BSD-3, kprovost/libs7comm BSD-2,
gijzelaerr/python-snap7 MIT). This satisfies DF-CANONICAL-FRAME-HOLDOUT-001, which
requires the canonical byte sequences themselves (not the parser's design) to be
sourced independently of the project's own specification artifacts.

The committed `tests/fixtures/s7comm-setup-comm.pcap` capture is a distinct, synthetic
artifact — generated by `tests/fixtures/mk_s7comm_pcap.py` per ADR-014 Decision 7 — and
is NOT one of the independently-sourced canonical byte literals; it is a regression
fixture built to encode the corrected 12-byte Ack_Data header shape.

---

## Recording Method

This is a pure-core/effectful-shell library story (no CLI binary registered yet — SS-21
`S7commAnalyzer` dispatch wiring to the CLI is STORY-193's obligation per ADR-014). Per
the demo-recording skill's library/test-harness mode (and following the STORY-186
precedent in `docs/demo-evidence/STORY-186/`), evidence is captured as VHS terminal
recordings of `cargo test --test s7comm_analyzer_tests`, filtered per behavior group and
piped through `grep` to show only the relevant `story_187::` test lines and the `test
result:` summary line. This also avoids the `Running tests/... (<worktree
path>/target/debug/deps/...)` line that `cargo test` otherwise prints, which would leak
an absolute local filesystem path into committed evidence (see Path-Scrub Gate below).

Six recordings were produced: five per behavior group (matching the task's suggested AC
groupings, with AC-187-006..011 split across two recordings to keep each recording
readable) plus one top-level full-`story_187`-module run.

| Artifact | Behavior group | ACs covered | Tests |
|----------|-----------------|-------------|-------|
| `AC-001-003-flow-state-session` | Flow state field set + lazy creation + CR/CC session tracking | AC-187-001, 002, 003 | 13 |
| `AC-004-005-012-dispatch-classification` | protocol_id dispatch + sticky first-classification-wins + conjunctive gating | AC-187-004, 005, 012 | 14 |
| `AC-006-009-length-rosctr-parser` | 10-byte minimum + defensive 0x32 reject + Job/Userdata extraction + ROSCTR totality | AC-187-006, 007, 008, 009 | 14 |
| `AC-010-011-ack-bounds-check` | Ack/Ack_Data 12-byte header + error fields + param/data bounds-check | AC-187-010, 011 | 22 |
| `AC-013-canonical-frames` | Canonical Setup Communication frame pair + fixture pcap | AC-187-013 | 4 |
| `AC-ALL-story_187-green` | Full `story_187` module | AC-187-001..013 | 63 |

(Per-recording test counts sum to more than 63 total for the five per-group recordings
because several tests are legitimately relevant to more than one recording's filter, as
noted in "Note on artifact overlap" above; the `AC-ALL-story_187-green` recording is the
authoritative de-duplicated 63-test count.)

---

## Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate command run from the repo root, per
`.factory/maintenance/demo-evidence-scrub-gate.md` (the mandatory absolute-host-path /
tilde-form-home-path grep, applied to this story's evidence directory).

**Result: zero matches.** All `.tape` files and this report use only repo-relative
paths (`tests/fixtures/...`, `src/analyzer/s7comm.rs`) and the `cargo test --test
s7comm_analyzer_tests <filter> 2>&1 | grep -E '...'` command form, which suppresses
cargo's own `Running tests/... (<worktree path>/target/debug/deps/...)` line (the only
place an absolute host path would otherwise leak into rendered GIF/webm text). (Note:
the gate command itself is not reproduced verbatim in this report, since its own regex
literal would trip the same grep it documents — see
`.factory/maintenance/demo-evidence-scrub-gate.md` for the exact command.)

## Doc-Tense Check

`python3 bin/check-green-doc-tense` was run against this report; see the commit-time
tool output for the pass/fail result recorded in this delivery's PR description.
