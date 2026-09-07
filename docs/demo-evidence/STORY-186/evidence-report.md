# Demo Evidence Report — STORY-186

**Story:** STORY-186: S7comm ISO-on-TCP Carry-Buffer Reassembly, Walk-First Frame
Extraction, Resync, and the Frozen SS-20/SS-21 Module Boundary
**Wave:** 89
**Date:** 2026-09-07
**Branch:** feature/STORY-186-iso-on-tcp-reassembly
**Product type:** Library (pure-core parser consumed by a new effectful-shell analyzer,
`S7commAnalyzer` in `src/analyzer/s7comm.rs`) — there is no CLI subcommand or web UI
surface for S7comm yet (dispatcher wiring to the CLI is deferred to STORY-193). The
demonstration vehicle is this story's own test harness: `tests/s7comm_analyzer_tests.rs`
(18 tests) is the executable proof of each acceptance criterion.
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test
s7comm_analyzer_tests`, filtered per behavior group)

---

## Full Test Suite: 18/18 PASS

Command:
```
cargo test --test s7comm_analyzer_tests
```

Result: **18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in ~4.6s**
(the 3 VP-050 proptests account for essentially all of the wall-clock time; the 15
unit/regression-guard tests each finish in under a millisecond).

Top-level artifact (all 18 tests, grouped by module, shown in one recording):

| Artifact | Description |
|----------|-------------|
| `AC-ALL-18-green.gif` / `AC-ALL-18-green.webm` | Full `s7comm_analyzer_tests` suite — 18/18 green |
| `AC-ALL-18-green.tape` | VHS script for the master suite run |

---

## AC → Test → Artifact Coverage Map

| AC | Title | BC | Test(s) | Artifact | Verdict |
|----|-------|-----|---------|----------|---------|
| AC-186-001 | Frame-walk loop extracts every complete TPKT frame before any byte-count bound is applied; no aggregate carry+incoming pre-check exists | BC-2.20.013 PC-1, PC-2, Inv-1 | `test_BC_2_20_013_walk_first_no_aggregate_precheck` | `AC-001-003-carry-reassembly.gif/.webm` | PASS |
| AC-186-002 | Adversarial burst with a complete frame at the head is never dropped despite 60,000 bytes of trailing garbage (anti-evasion) | BC-2.20.013 Inv-1 | `test_BC_2_20_013_adversarial_burst_head_frame_not_dropped` | `AC-001-003-carry-reassembly.gif/.webm` | PASS |
| AC-186-003 | Split-frame reassembly across two `on_data` calls — header-only partial stashed, then completed and carry emptied | BC-2.20.013 EC-002 | `test_BC_2_20_013_split_frame_across_two_calls` | `AC-001-003-carry-reassembly.gif/.webm` | PASS |
| AC-186-004 | Carry buffer bounded at 65,535 bytes; at-bound residual (`== 65,535`) is legitimate, not overflow — **LIVE, reachable via real `on_data` traffic** (comparison is strict `>`, not `>=`) | BC-2.20.014 Inv-1, EC-001 | `test_BC_2_20_014_at_bound_residual_no_overflow` | `AC-004-006-defense-in-depth.gif/.webm` | PASS |
| AC-186-005 | **[DEFENSE-IN-DEPTH, SYNTHETIC]** Carry-overflow guard mechanics — clear-not-truncate, resync, exactly one T0814 per direction, dedup on repeat — exercised via direct flow-state injection, not via `on_data`; **plus** the positive on_data-driven unreachability proof (200,000-byte garbage flood emits no T0814, carry stays ≤ 65,534) | BC-2.20.014 PC-1, PC-3, PC-4, EC-004 | `test_BC_2_20_014_overflow_clear_resync_one_t0814_per_direction`, `test_BC_2_20_014_repeated_overflow_dedup_same_direction`, `test_BC_2_20_014_overflow_unreachable_via_on_data` | `AC-004-006-defense-in-depth.gif/.webm` | PASS |
| AC-186-006 | **[DEFENSE-IN-DEPTH, SYNTHETIC]** Overflow dedup flags are independent per direction (c2s dedup has no bearing on s2c) — guard mechanics via direct flow-state injection | BC-2.20.014 PC-4, EC-005 | `test_BC_2_20_014_overflow_dedup_independent_per_direction` | `AC-004-006-defense-in-depth.gif/.webm` | PASS |
| AC-186-007 | Resync advances exactly 1 byte per iteration on a bad TPKT version byte, never 2 | BC-2.20.015 PC-1, Inv-1 | `test_BC_2_20_015_resync_advances_exactly_one_byte` | `AC-007-009-resync.gif/.webm` | PASS |
| AC-186-008 | Resync sub-routine is reused verbatim for both bad-version-byte and post-overflow conditions — exactly one implementation | BC-2.20.015 Inv-3 | `test_BC_2_20_015_single_resync_implementation_shared` | `AC-007-009-resync.gif/.webm` | PASS |
| AC-186-009 | Resync always terminates for finite input (200 bytes of non-anchored garbage, no infinite loop) | BC-2.20.015 Inv-2 | `test_BC_2_20_015_resync_terminates_no_valid_anchor` | `AC-007-009-resync.gif/.webm` | PASS |
| AC-186-010 | `iso_on_tcp.rs` contains zero `impl StreamAnalyzer` blocks / `DispatchTarget::IsoOnTcp`-shaped references (frozen module boundary, static regression guard) | BC-2.20.016 PC-1 | `test_BC_2_20_016_iso_on_tcp_has_no_stream_analyzer_impl` | `AC-010-011-module-boundary.gif/.webm` | PASS |
| AC-186-011 | TPKT/COTP carry buffers live on `S7commFlowState` only; no `IsoOnTcpFlowState` type exists anywhere in the tree (static regression guard) | BC-2.20.016 PC-3 | `test_BC_2_20_016_no_iso_on_tcp_flow_state_type_exists` | `AC-010-011-module-boundary.gif/.webm` | PASS |
| AC-186-012 | `on_flow_close` removes `S7commFlowState` and discards carry bytes with no finding; double-close (or unknown flow_key) is a no-op | BC-2.21.003 PC-1..4 | `test_s7comm_on_flow_close_removes_state_discards_carry`, `test_BC_2_21_003_double_close_same_flow_key_is_idempotent_no_op` | `AC-012-flow-close.gif/.webm` | PASS |

**All 12 acceptance criteria (AC-186-001..012) are covered by at least one recorded
artifact.**

---

## VP-050 Proptest Obligation

| Harness | Property | Test | Artifact |
|---------|----------|------|----------|
| `proptest_vp050_walk_first_residual_bound` | Carry stays `<= MAX_S7_ISO_ON_TCP_CARRY_BYTES` across randomized delivery patterns | `story_186::vp050::proptest_vp050_walk_first_residual_bound` | `VP-050-proptests.gif/.webm` |
| `proptest_vp050_direction_isolation` | `carry_c2s` only ever contains C2S-routed bytes, `carry_s2c` only ever contains S2C-routed bytes | `story_186::vp050::proptest_vp050_direction_isolation` | `VP-050-proptests.gif/.webm` |
| `proptest_vp050_resync_one_byte_advance` | Resync never advances by more than 1 byte per iteration, across randomized garbage-length inputs | `story_186::vp050::proptest_vp050_resync_one_byte_advance` | `VP-050-proptests.gif/.webm` |

All 3 proptest harnesses pass green (default proptest case count). Full walk-first
equivalence property (splitting a byte sequence into `carry + incoming` yields the
identical result as running the walk once on the concatenated bytes) is deferred to
STORY-194 per the story's own VP-050 obligation note.

---

## Defense-in-Depth Reclassification Note (AC-186-004/005/006)

Per BC-2.20.014 v1.1 (STORY-186 adversarial gate F-02/F-03, two independent passes,
human ruling 2026-09-07, Option B — Defense-in-Depth): `residual.len() > 65,535` is
provably unreachable via the real `on_data` data path under the current BC-2.20.013
walk-first + BC-2.20.015 1-byte-resync design, because the TPKT `length` field is
u16-capped — the directional carry is bounded `≤ 65,534` bytes by construction for both
conformant and adversarial input. Consequently:

- **AC-186-004** (the at-bound case, `residual.len() == 65,535`) is **LIVE** —
  reachable via real `on_data` traffic — and is recorded exercising the actual
  `on_data` walk-first path (`test_BC_2_20_014_at_bound_residual_no_overflow`).
- **AC-186-005/006** (the over-bound guard mechanics — clear-not-truncate, resync,
  one-T0814-per-direction, per-direction dedup independence) are recorded as
  **SYNTHETIC**: the tests directly construct/inject an oversized `carry_c2s`/`carry_s2c`
  on `S7commFlowState`, bypassing the normal `on_data` walk-first/resync path entirely.
  These remain the binding specification for the guard's behavior *if* it is ever
  reached (structural defense-in-depth against a future design regression), but are not
  scenarios exercised by feeding bytes through `on_data` today.
- The **positive on_data-driven unreachability proof**
  (`test_BC_2_20_014_overflow_unreachable_via_on_data`) is the counterpart assertion:
  a real 200,000-byte non-anchored garbage flood fed through `on_data` emits **no**
  T0814 for either direction and keeps carry bounded `≤ 65,534` at every observation
  point — confirming the guard's precondition is not reached by real traffic.

All three recordings for this group are captured together in
`AC-004-006-defense-in-depth.gif/.webm`, with the comment line in the recording itself
distinguishing the LIVE case from the SYNTHETIC cases.

---

## Recording Method

This is a pure-core/effectful-shell library story (no CLI binary registered yet — SS-21
`S7commAnalyzer` dispatch wiring to the CLI is STORY-193's obligation per ADR-014).
Per the demo-recording skill's library/test-harness mode, evidence is captured as VHS
terminal recordings of `cargo test --test s7comm_analyzer_tests`, filtered per behavior
group and piped through `grep` to show only the relevant `story_186::` test lines and
the `test result:` summary line (this also avoids the `Running tests/... (<worktree
path>/target/debug/deps/...)` line that `cargo test` otherwise prints, which would
leak an absolute local filesystem path into committed evidence — see the Path-Scrub
Gate section below).

Seven recordings were produced, one per behavior group named in the task plus one
top-level full-suite run:

| Artifact | Behavior group | ACs covered |
|----------|----------------|-------------|
| `AC-001-003-carry-reassembly.gif/.webm` | Walk-first carry-buffer reassembly, adversarial-burst anti-evasion, split-frame reassembly (BC-2.20.013) | AC-186-001, 002, 003 |
| `AC-004-006-defense-in-depth.gif/.webm` | Carry bound + defense-in-depth overflow guard (live at-bound + synthetic guard mechanics + positive unreachability) (BC-2.20.014) | AC-186-004, 005, 006 |
| `AC-007-009-resync.gif/.webm` | 1-byte resync, never 2; shared implementation; termination (BC-2.20.015) | AC-186-007, 008, 009 |
| `AC-010-011-module-boundary.gif/.webm` | Frozen SS-20/SS-21 module boundary static regression guards (BC-2.20.016) | AC-186-010, 011 |
| `AC-012-flow-close.gif/.webm` | Flow-close teardown + double-close idempotency (BC-2.21.003) | AC-186-012 |
| `VP-050-proptests.gif/.webm` | VP-050 proptest obligation (3 harnesses) | VP-050 |
| `AC-ALL-18-green.gif/.webm` | Full suite, all 18 tests | All 12 ACs + VP-050 |

VHS recording settings: `FontFamily "Menlo"`, `Theme "Dracula"`, `Shell "bash"`. No
absolute filesystem path or custom shell prompt is ever typed into any recording — VHS's
own default minimal `>` prompt is used throughout, and every `cargo test` invocation is
piped through `grep` to strip the `Running tests/...` line that would otherwise echo the
worktree's absolute path.

---

## Artifact List

| File | AC / VP Coverage |
|------|-------------------|
| `AC-001-003-carry-reassembly.gif` | AC-186-001, AC-186-002, AC-186-003 |
| `AC-001-003-carry-reassembly.webm` | AC-186-001, AC-186-002, AC-186-003 |
| `AC-001-003-carry-reassembly.tape` | VHS source for the above |
| `AC-004-006-defense-in-depth.gif` | AC-186-004 (live), AC-186-005 (synthetic + positive unreachability), AC-186-006 (synthetic) |
| `AC-004-006-defense-in-depth.webm` | AC-186-004, AC-186-005, AC-186-006 |
| `AC-004-006-defense-in-depth.tape` | VHS source for the above |
| `AC-007-009-resync.gif` | AC-186-007, AC-186-008, AC-186-009 |
| `AC-007-009-resync.webm` | AC-186-007, AC-186-008, AC-186-009 |
| `AC-007-009-resync.tape` | VHS source for the above |
| `AC-010-011-module-boundary.gif` | AC-186-010, AC-186-011 |
| `AC-010-011-module-boundary.webm` | AC-186-010, AC-186-011 |
| `AC-010-011-module-boundary.tape` | VHS source for the above |
| `AC-012-flow-close.gif` | AC-186-012 |
| `AC-012-flow-close.webm` | AC-186-012 |
| `AC-012-flow-close.tape` | VHS source for the above |
| `VP-050-proptests.gif` | VP-050 (3 proptest harnesses) |
| `VP-050-proptests.webm` | VP-050 |
| `VP-050-proptests.tape` | VHS source for the above |
| `AC-ALL-18-green.gif` | All 12 ACs + VP-050 (top-level, full suite) |
| `AC-ALL-18-green.webm` | All 12 ACs + VP-050 |
| `AC-ALL-18-green.tape` | VHS source for the above |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

Gate command run from the repo root before commit, exactly as specified in
`.factory/maintenance/demo-evidence-scrub-gate.md`: a recursive extended-regex search
under this directory for absolute macOS/Linux home-directory path prefixes and
tilde-form home references. The literal pattern is intentionally not reproduced in this
report, since the report itself lives under the directory the gate scans and would
otherwise self-match.

Result: **zero content matches** across all `.tape` script sources and this
`evidence-report.md` file. No absolute host path or tilde-form home reference is present
in any text file in this directory.

Note on the binary `.gif`/`.webm` recordings: every `cargo test` invocation captured in
these recordings is piped through `grep -E 'story_186::...|test result:'`, which
deliberately filters out the `Running tests/... (<worktree>/target/debug/deps/...)` line
`cargo test` prints by default — this is the one line in raw `cargo test` output that
would otherwise echo an absolute local filesystem path. No custom shell prompt (which
could echo a working-directory path) was configured in any tape; VHS's own default
minimal `>` prompt is used throughout instead.

Gate status: **PASSED** (2026-09-07).
