# Adversarial Review — STORY-096 (Implementation) — Pass 3

| Field | Value |
|-------|-------|
| Target | implementation (test formalization, facade mode) |
| Scope | STORY-096 — `tests/cli_story_096_tests.rs` + traceability to BC-2.13.001/002/003/004, STORY-096.md |
| Strategy under review | brownfield-formalization (zero src changes) — tests prove ABSENCE of removed flags |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 3 |
| Date | 2026-05-31 |
| Branch | feature/STORY-096-absent-flag-rejection |
| Worktree HEAD | abc4b4b (base develop c2445dc) + uncommitted pass-2 AC-006 fix |
| Verdict | **MEDIUM** (1 MEDIUM mutation-resistance gap; 0 Critical/High) |

## Checkout Guard (DF-ADVERSARY-CHECKOUT-GUARD-001)

- Branch: `feature/STORY-096-absent-flag-rejection` — OK (not develop).
- Worktree-base attestation: HEAD abc4b4b, base develop c2445dc.
- `grep -c '#[test]'` = 14 (10 AC + 4 EC) — matches story.

## Supplied / Self-Run Evidence (DF-ADVERSARY-TOOLCHAIN-PAIRING-001)

| Axis | Result |
|------|--------|
| `cargo test --test cli_story_096_tests` | 14 passed; 0 failed |
| `cargo clippy --test cli_story_096_tests -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| ErrorKind probe (8 reject invocations + 3 controls) | every reject = `UnknownArgument` naming the SPECIFIC flag; valid invocations parse OK; missing-target = `MissingRequiredArgument` (distinct) |
| Live mutation — `--threats` field on `Commands::Analyze` (8-space, non-pub) | AC-002 FAILED + AC-001 FAILED — caught |
| Live mutation — global `verbose` (`short,long,global`) | AC-008 FAILED + AC-009 FAILED — caught |
| **Live mutation — `C2BeaconAnalyzer` in `src/summary.rs`** | AC-004 **PASSED** (false-pass) — see F-W24-S096-P3-001 |
| **Live mutation — `C2BeaconAnalyzer` in new `src/analyzer/beacon.rs`** | AC-004 **PASSED** (false-pass) — see F-W24-S096-P3-001 |

## Method

Fresh-context attack on a different axis than the BPF-dependency axis. Re-derived
BC-2.13.001..004 invariants independently. Probed (1) whether the clap-rejection tests
pass for the RIGHT reason via a throwaway ErrorKind probe; (2) AC-002/008/009
field-reintroduction on both the top-level `Cli` struct and the `Commands::Analyze`
subcommand variant; (3) AC-004's `include_str!` file-set coverage against the full
`src/` tree. All probes were live mutations against worktree source; throwaway probe
tests were removed and git status verified clean after each.

## Prior-Fix Propagation Audit (S-7.01)

Pass-2 AC-006 fix (`declares_dep` structural key matcher over
`["pcap","pcap-filter","bpf","bpf-sys","libpcap"]`) is present and verified: 14/14
green, and a standalone 19-case predicate unit-check confirmed all forbidden
crate keys are caught across inline/dotted/table syntaxes while `pcap-file` is never
mis-flagged. No regression from the pass-2 fix. The same defect CLASS (an
absence-predicate narrower than the BC's "absent everywhere in src/" scope) recurs in
AC-004 — see finding below.

## Critical Findings

None.

## Important Findings

### F-W24-S096-P3-001 [MEDIUM, confidence HIGH] — AC-004 only scans 7 hand-listed source files; `C2BeaconAnalyzer` reintroduced anywhere else in `src/` false-passes

**Test:** `test_beacon_analyzer_absent_from_src` (tests/cli_story_096_tests.rs:160-200)
**BC:** BC-2.13.002 invariant 2 — "No `C2BeaconAnalyzer` or equivalent struct exists
in `src/`." (Scope is **all of `src/`**, not the analyzer subset.)

**Defect:** The test `include_str!`s exactly 7 files (`cli.rs`, `analyzer/mod.rs`,
`analyzer/dns.rs`, `analyzer/http.rs`, `analyzer/tls.rs`, `dispatcher.rs`, `lib.rs`,
`main.rs`). The actual `src/` tree contains 24 `.rs` files; **15 are outside the
scanned set** (`decoder.rs`, `mitre.rs`, `reader.rs`, `reassembly/*` ×7,
`reporter/*` ×4, `summary.rs`). A `C2BeaconAnalyzer` struct declared in any of those —
or in a new file such as `src/analyzer/beacon.rs` — satisfies the BC-violating
condition while leaving the test green.

**Live-verified (two vectors):**
- `pub struct C2BeaconAnalyzer;` appended to `src/summary.rs` → AC-004 test PASSED
  (false-pass).
- new file `src/analyzer/beacon.rs` containing `pub struct C2BeaconAnalyzer;` →
  AC-004 test PASSED (false-pass).

**Independent severity adjudication (NOT inheriting prior conclusions):** One could
argue reintroducing beacon detection also needs a `--beacon` flag that AC-003 catches.
I reject that as a sufficient mitigation for THIS invariant: BC-2.13.002 invariant 2 is
a standalone "struct does not exist in src/" guarantee, independent of CLI wiring. A
`C2BeaconAnalyzer` could be landed as internal/dead code or wired via `--all` without
a `--beacon` flag — AC-003 would still pass and AC-004 would still false-pass, so the
invariant would be violated with zero test failure. This is the same defect class as
F-W24-S096-P2-001 (absence predicate narrower than the BC's full-`src/` scope) and the
facade quality gate for this story is mutation-resistance.

**Why MEDIUM (not HIGH):** blast radius = 1 test; the struct-name pin (`BeaconAnalyzer`
/ `C2BeaconAnalyzer`) is consistent with the BC ("or equivalent struct"); and a
realistic full beacon feature would touch multiple ACs. But the absence invariant is
the gate, and it is live-evadable across 15 of 24 source files.

**Suggested fix (test-writer):** scan ALL `src/**/*.rs` at test runtime instead of a
hand-listed `include_str!` set. `include_str!` cannot glob (compile-time literal paths
only), so read the tree at runtime via `env!("CARGO_MANIFEST_DIR")` + a small
recursive directory walk over `src/`, asserting no file contains `struct BeaconAnalyzer`
/ `struct C2BeaconAnalyzer` / `impl (C2)?BeaconAnalyzer`. Keep a positive sanity guard
that the walk actually visited a non-zero, expected number of `.rs` files (so an empty
walk cannot false-green — cf. the CI positive-coverage axis). Re-verify against the two
live vectors above before re-running Pass 4.

## Observations

- **O-1 (non-finding):** clap-rejection tests (AC-001/003/005/007/008 + EC-001/002/003)
  all pass for the RIGHT reason — the ErrorKind probe shows each errors with
  `UnknownArgument` naming the specific flag, and `analyze` (missing target) yields a
  DIFFERENT kind (`MissingRequiredArgument`), proving the tests are discriminating.
- **O-2 (non-finding):** AC-002/009 field-absence are jointly mutation-resistant with
  their behavioral siblings. `--threats` on `Commands::Analyze` (8-space indent) →
  AC-002 caught (the 4-space substring `"    threats:"` is contained in 8-space indent)
  AND AC-001 caught (flag now valid → `parse_err` panics). Global `verbose` → AC-008 +
  AC-009 both caught. Non-global top-level `verbose` → AC-009 catches the field, and
  `-v` after the subcommand genuinely remains rejected (so AC-008 staying green there is
  CORRECT, not a false-pass).
- **O-3 (non-finding):** AC-010/EC-004 positive-parse tests assert the parsed
  `Commands::Analyze` variant + target path, not merely `Ok` — discriminating.
- **O-4 (non-finding):** DF-AC-TEST-NAME-SYNC-001 PASS (10/10 AC citations resolve 1:1);
  DF-TEST-NAMESPACE-001 PASS (14 tests in `mod story_096`). Story FSR `tests/cli_tests.rs`
  vs actual dedicated file — cosmetic, documented in header (matches prior disposition).

## Policy Rubric Compliance

| Policy | Verdict |
|--------|---------|
| DF-AC-TEST-NAME-SYNC-001 (MEDIUM) | PASS |
| DF-TEST-NAMESPACE-001 (MEDIUM) | PASS |
| DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH) | satisfied |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM) | satisfied (cargo + live mutations + ErrorKind probe self-run) |
| DF-SIBLING-SWEEP-001 (CRITICAL) | fix must also confirm `struct BeaconAnalyzer`/`impl …` variants and the new-file vector are covered by the runtime walk |
| DF-VALIDATION-001 (HIGH) | N/A — test-only, fixed in place |

## Novelty Assessment

Novelty: **MEDIUM-HIGH** — F-W24-S096-P3-001 is a genuinely new axis (AC-004 file-set
scope coverage) not probed in passes 1–2. It is the SAME defect class as the AC-006
gap (absence predicate narrower than BC's full-`src/` scope), which strengthens the
case that the root-cause fix pattern is "match the BC's stated scope, not a convenient
subset." All other axes (clap rejection correctness, field-absence joint resistance,
positive-parse discrimination) are confirmed clean.

## Verdict

**NOT CONVERGED after Pass 3.** One MEDIUM mutation-resistance gap
(F-W24-S096-P3-001): AC-004 scans only 7 of 24 `src/` files, so a `C2BeaconAnalyzer`
reintroduced in any of the other 15 files (or a new analyzer file) false-passes,
violating BC-2.13.002 invariant 2 silently. Fix to a runtime full-`src/`-tree walk with
a positive-coverage guard, re-verify against the two live vectors, then re-run Pass 4.
Minimum 3 consecutive clean passes required.

## Post-Pass-3 Remediation (orchestrator, same cycle) — FIXED

Root-cause fix applied to `test_beacon_analyzer_absent_from_src`: replaced the
hand-listed 7-file `include_str!` set with a runtime recursive walk over ALL `*.rs`
files under `src/` (resolved via `env!("CARGO_MANIFEST_DIR")`), asserting absence of
`struct BeaconAnalyzer` / `struct C2BeaconAnalyzer` / `impl (C2)?BeaconAnalyzer` in
every file. Added a positive-coverage guard (`rs_files.len() >= 20`) so an empty or
mis-rooted walk fails loudly rather than vacuously passing the absence checks
(CI-positive-coverage discipline). This matches the BC-2.13.002 invariant-2 scope
("anywhere in `src/`") instead of a convenient analyzer subset — the same root-cause
pattern applied to the AC-006 fix in pass 2.

Verification evidence:
- 14/14 tests pass; clippy `-D warnings` clean; `cargo fmt --check` clean.
- Live re-test of the two pass-3 evasion vectors → both now caught:
  `C2BeaconAnalyzer` in `src/summary.rs` → FAILED; new `src/analyzer/beacon.rs` →
  FAILED.
- Additional vectors: in-set `src/analyzer/dns.rs` → FAILED; deeply-nested
  `src/reassembly/flow.rs` → FAILED. Clean baseline → ok.
- Coverage guard confirmed: walk finds 24 `.rs` files (threshold 20).
- Only `tests/cli_story_096_tests.rs` modified — zero src changes (facade preserved).

F-W24-S096-P3-001 disposition: **RESOLVED**. Clean-pass counter remains 0 (passes 2 and
3 were both DIRTY); 3 consecutive clean passes required from the next clean pass onward.
