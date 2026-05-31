---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-31T16:50:00
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
pass: 2
previous_review: pass-1-STORY-089.md
---

# Adversarial Review: STORY-089 (Pass 2)

**Target:** Same as pass 1. Fresh-context re-derivation. Attack lens this pass:
**silent-failure / channel discrimination (stdout vs stderr) / exit-code regressions /
predicate strictness / CSV-vs-Json routing pinning** — angles orthogonal to pass 1's
guard/counter/precedence mutations.

**Mutation-resistance methodology:** identical to pass 1 (apply to worktree `src/main.rs`,
run suite, revert, verify `git diff --quiet` clean + 17/17 green).

## Part A — Fix Verification (prior-pass findings)

No fix burst was applied between pass 1 and pass 2 (STORY-089 is a zero-src
brownfield-formalization; pass-1 findings are coverage/docstring advisories that do not
invalidate the green suite). Prior-pass findings therefore remain OPEN and are re-stated as
carry-forward below (not re-counted as NEW — see Novelty Assessment).

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-P01-HIGH-001 (run_summary untested) | HIGH | UNRESOLVED | re-confirmed this pass via Mutation A scoping |
| ADV-P01-MED-001 (73-of-58 impossible) | MEDIUM | UNRESOLVED | docstring unchanged |
| ADV-P01-MED-002 (AC-005 subsumed by EC-002) | MEDIUM | UNRESOLVED | unchanged |
| ADV-P01-MED-003 (no non-zero unclassified fixture) | MEDIUM | UNRESOLVED | unchanged |
| ADV-P01-LOW-001 (http.pcap unused) | LOW | UNRESOLVED | unchanged |
| ADV-P01-LOW-002 (AC-012 docstring overstates) | LOW | UNRESOLVED | unchanged |

## Part B — New Findings (pass 2)

### CRITICAL

_None._ Three additional mutation axes (orthogonal to pass 1) all KILLED:

- **Decode-error must NOT propagate (BC-2.12.014 PC4 "loop continues, no early exit"):**
  Mutation A inserted `anyhow::bail!` into the run_analyze `Err` arm (early-abort on first
  decode error). KILLED by 5 tests (AC-001, AC-002, AC-003, AC-004, EC-005) — the
  `.success()` + `skipped_packets == 73` assertions require the run to complete all packets.
- **Warning channel (stderr, not stdout):** Mutation B changed the warning `eprintln!` →
  `println!` in run_analyze. KILLED by 4 tests (AC-001, AC-002, AC-004, EC-005) — stderr
  warning count drops to 0 (observed "found 0 times. stderr: <empty>"). The stdout/stderr
  channel split is genuinely discriminated, not conflated.
- **CSV routing (AC-008):** Mutation C mis-routed the `cli.csv.is_some()` branch to
  `Some(OutputFormat::Json)`. KILLED by AC-008 (`test_resolve_format_csv_flag`, line 359) —
  the CSV-header `starts_with("category,verdict,confidence,")` assertion (line 366) fails when
  CSV is rendered as JSON.

### MEDIUM

_None new._

### LOW

#### ADV-P02-LOW-001: EC-005 docstring says it is "covered by AC-001 + AC-003 together," but it is a standalone test with its own assertions — the prose is self-contradictory
- **Severity:** LOW
- **Category:** spec-fidelity
- **Confidence:** HIGH
- **Location:** `tests/main_story_089_tests.rs:707-710` (docstring) vs `:719-741` (test body).
- **Description:** The EC-005 docstring states "This edge case is **covered by AC-001 +
  AC-003 together**." But `test_EC_005_all_packets_fail_one_warning_skipped_count_accurate`
  has an independent body that asserts BOTH `warning_count == 1` (line 731) AND
  `"skipped_packets": 73` (line 738) in one invocation — it is NOT delegating to AC-001/AC-003,
  it re-implements the combined invariant. The "covered by … together" framing wrongly implies
  EC-005 is redundant/vacuous when it is in fact a legitimate standalone combined-invariant
  test. (It does, however, duplicate AC-002's exact command+assertions — see ADV-P02-LOW-002.)
  Also embeds the same "73 of 58" impossible claim flagged in ADV-P01-MED-001.
- **Evidence:** Lines 720-741 contain a full `Command::...output()` + two `assert*` calls; no
  reference to AC-001/AC-003 functions.
- **Proposed Fix:** Reword the docstring to "EC-005 independently asserts the combined
  invariant (exactly one warning AND skipped_packets == N) on the all-fail fixture," dropping
  the misleading "covered by AC-001 + AC-003 together" clause.

#### ADV-P02-LOW-002: AC-002 and EC-005 are near-duplicates (same fixture, same command, overlapping assertions)
- **Severity:** LOW
- **Category:** coverage-gap
- **Confidence:** HIGH
- **Location:** `tests/main_story_089_tests.rs:122-148` (AC-002,
  `test_subsequent_decode_errors_silent`) vs `:719-741` (EC-005,
  `test_EC_005_all_packets_fail_one_warning_skipped_count_accurate`).
- **Description:** Both run `analyze dns-remoteshell.pcap --dns --json` via `.output()`, both
  assert `warning_count == 1` and `"skipped_packets": 73`. AC-002 uses
  `.lines().filter(...).count()` (line 132-135) while EC-005 uses `.matches(...).count()`
  (line 730) — functionally equivalent here (each warning is one line). The two tests are
  near-identical; the mutation matrix confirms they always kill together (M1, M2, Mutation A,
  Mutation B). This is acceptable as AC-vs-EC trace separation, but the redundancy is worth
  noting per the STORY-088 identical-command lesson. The distinct line-vs-matches counting
  methods are a (very) minor robustness diversification, not a true coverage difference.
- **Evidence:** Lines 125 and 723 issue identical args; lines 137/145 and 731/738 assert the
  same two facts.
- **Proposed Fix:** Acceptable as-is (AC and EC traces legitimately need separate named
  tests). Optionally differentiate EC-005 to assert an additional fact AC-002 does not (e.g.,
  `.success()` exit AND that `findings` array is present/empty per BC-2.12.014 EC-004 "no
  findings produced" — currently unasserted), giving EC-005 independent value.

## Observations

- The `--http` flag is used on `http-ooo.pcap` for all format/routing tests (AC-007..012,
  EC-002..004). `--http` activates the TcpReassembler, which is *required* for AC-005/AC-006/
  EC-002 (unclassified_flows) but is incidental for the pure format/routing tests
  (AC-007..012). This is harmless but means the format tests carry reassembly overhead they
  do not need; not a correctness concern.
- BC-2.12.014 EC-004 ("All packets fail decode → skipped_packets=total; **no findings
  produced**") — the "no findings produced" half is never asserted. EC-005 asserts
  skipped_packets but not the absence of findings. LOW-severity gap folded into
  ADV-P02-LOW-002's proposed fix rather than filed separately.

## Summary

| Severity | Count (NEW this pass) |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 2 |

**Overall Assessment:** pass-with-findings
**Convergence:** findings remain — iterate (carry-forward HIGH/MEDIUM from pass 1 still open;
new findings this pass are LOW only)
**Readiness:** core axes re-confirmed mutation-resistant from a second, orthogonal angle. New
findings are docstring/redundancy LOW items. Trajectory decreasing (6 → 2 new).

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 2 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 |
| **Median severity** | 1.0 (LOW) |
| **Trajectory** | 6 → 2 |
| **Verdict** | FINDINGS_REMAIN |

<!-- Novelty score counts only NEW findings this pass (2). Both are genuinely new (not
restatements of pass-1 findings), hence score 1.00; but median severity collapsed to LOW and
new-finding count is decreasing — early convergence signal. Carry-forward pass-1 findings
remain open and are tracked in Part A, not double-counted here. -->
