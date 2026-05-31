---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-31T17:00:00
phase: 5
inputs:
  - tests/main_story_089_tests.rs
  - src/main.rs
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.014.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.015.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.016.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.017.md
  - .factory/stories/STORY-089.md
input-hash: "n/a"
traces_to: prd.md
pass: 3
previous_review: pass-2-STORY-089.md
---

# Adversarial Review: STORY-089 (Pass 3)

**Target:** Same as passes 1-2. Fresh-context re-derivation. Attack lens this pass:
**CLI-surface invariants (clap mutual-exclusion), structurally-unreachable precedence
states, run_summary blast-radius, nested-Option (Some(None)/Some(Some)) routing semantics.**

**Mutation-resistance methodology:** identical (apply to worktree `src/main.rs`, run, revert,
verify clean + 17/17 green).

## Part A — Fix Verification

No fix burst applied between passes 2 and 3 (zero-src formalization story; findings are
advisory). Carry-forward status:

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-P01-HIGH-001 (run_summary untested) | HIGH | UNRESOLVED — **blast radius widened** | see ADV-P03-HIGH-001 |
| ADV-P01-MED-001..003, ADV-P01-LOW-001..002 | — | UNRESOLVED | unchanged |
| ADV-P02-LOW-001..002 | LOW | UNRESOLVED | unchanged |

## Part B — New Findings (pass 3)

### CRITICAL

_None._ Two further orthogonal mutation axes KILLED:

- **stdout default-routing arm (BC-2.12.017 PC3):** Mutation E redirected `write_output`'s
  default `_` arm from `println!` to `std::fs::write("/tmp/...")`. KILLED broadly (EC-003,
  AC-011, and every stdout-reading test) — the "no file path ⇒ stdout" routing is firmly
  pinned.
- (Mutation D survival is a coverage finding, not a defect — see HIGH-001.)

### HIGH

#### ADV-P03-HIGH-001: run_summary coverage gap is broader than pass-1 scope — it also leaves BC-2.12.016 (format selection) and BC-2.12.017 (routing) unverified in run_summary, not only BC-2.12.014
- **Severity:** HIGH
- **Category:** coverage-gap
- **Confidence:** HIGH
- **Location:** `src/main.rs:280-302` (run_summary's `resolve_format` + reporter match +
  `write_output` call); tests: no `summary`-subcommand invocation exists.
- **Description:** Pass 1 (ADV-P01-HIGH-001) established that run_summary's decode-error
  counting (BC-2.12.014) is untested. This pass shows the gap is wider: run_summary contains
  its OWN copies of the format-selection match (BC-2.12.016 PC4 "reporter selected matches
  resolved format") and the `write_output` routing call (BC-2.12.017). All four BCs in
  STORY-089's scope nominally apply to both entry points, but only run_analyze is exercised.
- **Evidence:** Empirical — Mutation D swapped the Json/Csv reporter arms in run_summary's
  match (main.rs:282-289). The full STORY-089 suite passed 17/17 unchanged — a run_summary
  format-routing inversion ships silently. Combined with Mutation M11 from pass 1
  (run_summary skipped_packets `+999`, also survived), the run_summary path is unprotected
  for BC-2.12.014, BC-2.12.016, and BC-2.12.017.
- **Proposed Fix:** Add a small `summary`-subcommand test block (2-3 tests) covering at least:
  (a) `summary dns-remoteshell.pcap --json` ⇒ `skipped_packets == 73` + exactly one warning
  (BC-2.12.014 in run_summary); (b) `summary http-ooo.pcap --csv` ⇒ CSV header
  (BC-2.12.016/017 routing in run_summary). This is a superset of the pass-1 proposed fix and
  supersedes it. NOTE for the orchestrator: if STORY-089 is intentionally scoped to
  run_analyze only, that scoping decision must be explicit in the story AND BC-2.12.014/016/017
  must record that their run_summary anchors are covered by a different story — currently no
  story claims them (verified: STORY-089 is the sole story in all four BCs' `Stories:` field).

### MEDIUM

#### ADV-P03-MED-001: BC-2.12.016 invariant 3 ("--json wins over --csv") is structurally untestable AND untested — the precedence between resolve_format clauses 1 and 2 is unreachable, making the asserted invariant vacuous
- **Severity:** MEDIUM
- **Category:** verification-gaps
- **Confidence:** HIGH
- **Location:** `src/main.rs:312-320` (resolve_format clauses `if cli.json.is_some()` /
  `else if cli.csv.is_some()`); `cli.rs` clap `conflicts_with` on the json/csv fields;
  STORY-089 Architecture Compliance Rules table cites "json > csv" indirectly via BC-2.12.016
  invariant 3.
- **Description:** resolve_format's clause ordering (json checked before csv) encodes a
  "json wins over csv" precedence. But `--json` and `--csv` are mutually exclusive at the
  clap layer (verified: `--json --csv` ⇒ `error: the argument '--json [<JSON>]' cannot be used
  with '--csv [<CSV>]'`, exit 2). The state "both `cli.json.is_some()` AND `cli.csv.is_some()`"
  is therefore unreachable. The relative order of clauses 1 and 2 is unobservable through any
  CLI path. Empirical proof from pass 1: Mutation M5 swapped the two clauses (csv checked
  first) and SURVIVED the full suite. The story's own precedence story ("--json > --csv >
  --output-format") is only verifiable for the json-vs-output_format and csv-vs-output_format
  pairs (both ARE tested — AC-007/AC-008/EC-004); the json-vs-csv pair at the top of the
  chain is vacuous. This is NOT a test defect to fix by adding a test (no test CAN reach it
  without bypassing clap) — it is a spec-fidelity issue: BC-2.12.016 invariant 3 and the
  story's precedence narrative overstate what is observable/meaningful given the clap
  constraint.
- **Evidence:** `--json --csv` rejected by clap (shown above). Mutation M5 (swap clauses)
  survived 17/17. The story explicitly declines a `resolve_format` lib extraction
  (test header lines 30-33), so a unit test that constructs an illegal
  `Cli { json: Some(_), csv: Some(_) }` directly is also out of scope by author decision.
- **Proposed Fix:** Annotate BC-2.12.016 invariant 3 and the story to state that the
  json-vs-csv precedence is *defensive/unreachable* given clap mutual-exclusion (invariant 2
  already documents the exclusion) — i.e., the clause order is belt-and-suspenders, not a
  testable behavior. This removes the false impression that the suite verifies "json beats
  csv." The observable precedence claims (json/csv each beat --output-format) remain valid and
  tested. No source change and no new test required; this is a documentation-accuracy fix.

### LOW

_None new._

## Observations

- AC-009 (`test_resolve_format_falls_back_to_output_format`) is the strongest single test in
  the suite: it exercises all three terminal/json/csv output formats via `--output-format` in
  one test (lines 395-425). It correctly anchors the "fallback to cli.output_format" branch
  and the three-way reporter dispatch. No gap.
- `[process-gap]` candidate (LOW, informational): The recurring "two near-identical copies of
  the decode-error + format-selection + routing logic in run_analyze and run_summary"
  structure (main.rs:166-243 vs 262-301) is a maintenance smell that makes every STORY-089 BC
  apply to two code sites while the test strategy covers one. This is a source-structure
  observation, not a test defect; flagged for the orchestrator in case a future refactor
  (extract a shared `process_packets`/`emit_output` helper) would let one test protect both
  paths. Not blocking.

## Summary

| Severity | Count (NEW this pass) |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 (refines/widens ADV-P01-HIGH-001) |
| MEDIUM | 1 |
| LOW | 0 |

**Overall Assessment:** pass-with-findings
**Convergence:** findings remain — iterate. NOT yet 3 clean passes (each pass has surfaced
genuinely new findings: pass 1 = 6, pass 2 = 2, pass 3 = 2). Trajectory of NEW findings is
decreasing but non-zero; convergence (3 consecutive clean passes) NOT reached.
**Readiness:** All eight dispatch-flagged mutation axes hold (mutation-resistant). The open
findings are coverage-completeness (run_summary), one structural verification-gap
(vacuous json>csv precedence), and docstring fidelity. None invalidate the 17 green tests.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 2 |
| **Duplicate/variant findings** | 1 (HIGH-001 refines pass-1 HIGH-001) |
| **Novelty score** | 0.67 |
| **Median severity** | 3.5 (between MEDIUM and HIGH) |
| **Trajectory** | 6 → 2 → 2 |
| **Verdict** | FINDINGS_REMAIN |

<!-- New findings this pass: ADV-P03-HIGH-001 (widens, counts as variant of pass-1 HIGH-001)
and ADV-P03-MED-001 (genuinely new: vacuous precedence). Novelty 0.67. Convergence requires
3 CONSECUTIVE clean passes; none of passes 1-3 are clean. Recommend a fix burst on the HIGH
(run_summary coverage) + MED (precedence doc) before resuming convergence passes. -->
