# Adversarial Review — STORY-087 (Implementation) — Pass 2

| Field | Value |
|-------|-------|
| Target | implementation (test formalization) |
| Scope | STORY-087 — `tests/cli_story_087_tests.rs`, branch `feature/STORY-087-output-reassembly-flags`, worktree `.worktrees/STORY-087` |
| Strategy under review | brownfield-formalization (zero src changes) |
| Artifacts read | STORY-087.md; BC-2.12.004/005/007; tests/cli_story_087_tests.rs; src/cli.rs (lines 14–107); STORY-086.md (FSR-pattern precedent); policies.yaml (DF-AC-TEST-NAME-SYNC-001, DF-TEST-CITATION-SWEEP-001, DF-TEST-NAMESPACE-001, DF-SIBLING-SWEEP-001) |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 2 (continuation; Pass 1 = DIRTY, 2 Low docstring findings F-S087-P1-001/002 FIXED at d9f91bc) |
| Date | 2026-05-31 |
| HEAD | d9f91bc |
| Verdict | **CLEAN** (no Critical/High/Medium; 1 Low/Informational refinement) |

## Method

Fresh-context re-derivation of every test assertion from its authoritative BC
postcondition/invariant and the live `src/cli.rs` ground truth, independent of
the Pass-1 findings. Ground-truth verification executed before review:

- `cargo test --test cli_story_087_tests` → **16 passed; 0 failed** (Green confirmed).
- `cargo clippy --test cli_story_087_tests -- -D warnings` → clean.
- `cargo fmt --check` on the test file → clean (exit 0).

Test inventory: AC-001..AC-012 (12) + EC-001/002/003/005 (4) = **16**, matching
the file docstring and story AC/EC set. EC-004 (`--output-format` + `--json`
precedence) is correctly deferred to STORY-089 per the story EC-004 note.

## Per-AC Verdicts

| AC | BC clause | Test fn | Assertion vs BC | src/cli.rs anchor | Verdict |
|----|-----------|---------|-----------------|-------------------|---------|
| AC-001 | BC-2.12.004 PC-1 | test_output_format_json_flag | `Some(Json)` + ne None + ne Csv | `value_enum`, `Option<OutputFormat>` (cli.rs:48-49) | CLEAN |
| AC-002 | BC-2.12.004 PC-2 | test_output_format_csv_flag | `Some(Csv)` + ne None + ne Json | same | CLEAN |
| AC-003 | BC-2.12.004 PC-3 | test_output_format_absent_is_none | `None` + is_none | same | CLEAN |
| AC-004 | BC-2.12.004 PC-4 | test_output_format_invalid_value_rejected | `ErrorKind::InvalidValue` | `value_enum` → InvalidValue for unknown variant | CLEAN |
| AC-005 | BC-2.12.005 PC-3 | test_reassembly_depth_default_is_10 | `==10`, ne 0, ne 1024 | `default_value_t = 10` (cli.rs:70-71) | CLEAN |
| AC-006 | BC-2.12.005 PC-4 | test_reassembly_memcap_default_is_1024 | `==1024`, ne 10 | `default_value_t = 1024` (cli.rs:74-75) | CLEAN |
| AC-007 | BC-2.12.005 PC-5 | test_reassembly_threshold_flags_default_none | 5×None absent; overlap=Some(42) | five `Option<…>` fields (cli.rs:80-106) | CLEAN |
| AC-008 | BC-2.12.005 PC-6 | test_overlap_threshold_out_of_range_rejected | `ErrorKind::ValueValidation` (256) | `value_parser!(u32).range(0..=255)` (cli.rs:80) | CLEAN |
| AC-009 | BC-2.12.005 Inv-3 | test_small_segment_ignore_ports_comma_delimited | `Some([23,513])`, len 2, order | `Option<Vec<u16>>`, `value_delimiter=','` (cli.rs:100-101) | CLEAN |
| AC-010 | BC-2.12.007 PC-1 | test_reassemble_and_no_reassemble_conflict | `ErrorKind::ArgumentConflict` | `conflicts_with="no_reassemble"` (cli.rs:62) | CLEAN |
| AC-011 | BC-2.12.007 Inv-1 (VP-018) | test_reassemble_conflict_is_symmetric | reversed order → `ArgumentConflict` | clap bidirectional conflict | CLEAN |
| AC-012 | BC-2.12.007 EC-003 | test_reassemble_alone_parses_ok | `reassemble==true`, `no_reassemble==false` | bool flags (cli.rs:63,67) | CLEAN |
| EC-001 | STORY-087 EC-001 | test_EC_001_reassembly_depth_zero_accepted | `depth==0` accepted | `usize`, no range floor | CLEAN |
| EC-002 | STORY-087 EC-002 | test_EC_002_small_segment_max_bytes_zero | `Some(0u16)` | `range(0..=2048)` allows 0 (cli.rs:94) | CLEAN |
| EC-003 | STORY-087 EC-003 | test_EC_003_overlap_threshold_max_accepted | `Some(255u32)` accepted | `range(0..=255)` inclusive | CLEAN |
| EC-005 | STORY-087 EC-005 | test_EC_005_no_reassembly_flags_all_defaults | all defaults / all None | composite | CLEAN |

## Policy Rubric Compliance

- **DF-TEST-NAMESPACE-001 (MEDIUM) — CLEAN.** All 16 tests are wrapped in
  `mod story_087`. No test function exists at the flat module root. BC-prefixed
  `test_EC_00N_*` names are collision-safe; `#![allow(non_snake_case)]` justified
  in the docstring.
- **DF-AC-TEST-NAME-SYNC-001 (MEDIUM) — CLEAN.** Every story AC `**Test:**`
  citation resolves to exactly one `fn test_*` in the cited file:
  AC-001→test_output_format_json_flag … AC-012→test_reassemble_alone_parses_ok.
  All 12 verified present and unique within the suite (file is `mod`-isolated).
  No cross-module ambiguity.
- **DF-TEST-CITATION-SWEEP-001 (HIGH) — CLEAN.** No citation re-point occurred in
  this burst beyond the P1 docstring corrections, which touched only the
  test-file header comments (location 4 of the 5-location sweep) and were
  self-contained — no story FSR row, BC Evidence field, or sibling BC cites a
  STORY-087 test function name. Verified by grep: no BC in the SS-12 set names a
  `test_output_format_*` / `test_reassembly_*` function in its Proof-Method.
- **DF-SIBLING-SWEEP-001 — N/A** (no AC/EC/BC content edit in scope; test-only).
- **DF-VALIDATION-001 — N/A** (no deferred finding being filed).

## Findings

### F-S087-P2-001 — FSR table cites stale flat test file (Low / Informational)

`STORY-087.md` File Structure Requirements row reads
`tests/cli_tests.rs | modify | Add AC-001..AC-012 test functions`, and the Token
Budget row likewise names `tests/cli_tests.rs`. Actual tests were delivered to a
dedicated `tests/cli_story_087_tests.rs` per **DF-TEST-NAMESPACE-001**. The
namespace policy supersedes the story-template FSR row.

- **Disposition:** Informational, non-blocking. This is a known story-template
  artifact, NOT a defect. The identical pattern exists in STORY-086
  (`tests/cli_tests.rs | modify`) and its three converged passes (1–3)
  intentionally did NOT flag it, treating the dedicated-file convention as
  authoritative. Recording for parity only. Does not gate convergence.
- **Severity rationale:** No behavioral, traceability, or test-correctness
  impact. The dedicated file is correct; only the story's prose FSR row lags.

## Trajectory

Pass 1: 2 Low (docstring) — FIXED. Pass 2: 1 Low/Informational (pre-existing
story-template parity, non-blocking). Finding count 2 → 1, monotonic decrease.
No Critical/High/Medium in either pass.

## Verdict

**CLEAN.** Zero Critical/High/Medium. The single Low item is a non-defect
parity note matching the prior story's accepted disposition. Test formalization
faithfully encodes BC-2.12.004/005/007 with discriminating positive+negative
assertions; ground truth Green/clippy/fmt all confirmed.
