---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-31T16:40:00
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
pass: 1
previous_review: null
---

# Adversarial Review: STORY-089 (Pass 1)

**Target:** Implementation review — `tests/main_story_089_tests.rs` (17 assert_cmd tests,
12 AC + 5 EC) against `src/main.rs` (binary-private fns formalized via subprocess CLI
behavior). Perimeter 1 (per-story). Scope: full.

**Mutation-resistance methodology:** Live mutations were applied to `src/main.rs` (worktree
copy — the binary `assert_cmd` builds), the STORY-089 suite was run, then the source was
reverted (verified `git diff --quiet src/main.rs` == clean, 17/17 green restored). Mutation
kill/survival is empirical ground truth for the severity classifications below.

## Finding ID Convention

No `.factory/current-cycle` file exists → cycle segment omitted: `ADV-P<PASS>-<SEV>-<SEQ>`.

## Part B — New Findings (pass 1)

### CRITICAL

_None._ No mutation against an asserted AC postcondition survived. The four CRITICAL
mutation-resistance axes flagged in the dispatch all KILLED:

- **AC-004 warning-once (count assertion):** Mutation M1 removed the
  `if total_decode_errors == 0` guard in `run_analyze` (warning fires on every error, 73×).
  KILLED by `test_decode_error_warning_printed_at_most_once` (line 209, `count == 1`),
  `test_subsequent_decode_errors_silent` (line 122), and
  `test_EC_005_all_packets_fail_one_warning_skipped_count_accurate` (line 720). This is the
  STORY-088 AC-004 lesson resolved: the `.matches(...).count() == 1` form (lines 219, 730) is
  genuinely mutation-resistant, NOT vacuous on this codepath.
- **AC-003 skipped_packets == total_decode_errors:** Mutation M2 changed
  `summary.skipped_packets = total_decode_errors` to `+ 1`. KILLED by AC-003 (line 169),
  AC-002, EC-001, EC-005 (observed `"skipped_packets": 74`).
- **AC-005/AC-006 unclassified_flows injection + is-Some guard:** Mutation M3 (delete the
  `reasm_summary.detail.insert("unclassified_flows", ...)` block, main.rs:205-208) KILLED by
  AC-005 (line 245) and EC-002 (line 607). Mutation M4 (leak the key into the
  `if enable_dns` summary path so it appears WITHOUT a reassembler) KILLED by AC-006
  (line 272). AC-006's absence-assertion is **NOT vacuous** — the `--dns --no-reassemble`
  fixture genuinely exercises a path where the key would surface if guard discipline broke.
- **AC-007/008/009 resolve_format precedence:** Mutation M6 (`--output-format` wins over
  `--json`) KILLED by AC-007 (line 307) and EC-004 (line 673). Mutation M8 (swap Json/Csv
  reporter dispatch arms) KILLED broadly (9 tests). Mutation M9 (default `_` arm →
  JsonReporter) KILLED by AC-009 (line 395) and AC-011 (line 505).
- **AC-010/AC-012 write routing + error context:** Mutation M10 (file-write arm → stdout)
  KILLED by AC-010 (file-content + empty-stdout assertions, lines 468/474) and AC-012.
  Mutation M7 (change "Failed to write JSON output to" context string) KILLED by AC-012
  (line 535).

### HIGH

#### ADV-P01-HIGH-001: run_summary decode-error codepath is entirely untested; BC-2.12.014 applies to it but a mutation survives
- **Severity:** HIGH
- **Category:** coverage-gap
- **Confidence:** HIGH
- **Location:** `src/main.rs:262-278` (run_summary decode handler + skipped_packets
  assignment); `tests/main_story_089_tests.rs` (no `summary` subcommand invocation anywhere).
- **Description:** BC-2.12.014 explicitly scopes its postconditions to BOTH `run_analyze`
  AND `run_summary` ("`run_analyze` **or** `run_summary` is processing packets" —
  precondition 1; Architecture Anchors cite `src/main.rs:266-276` — same pattern in
  run_summary). The STORY-089 suite invokes ONLY the `analyze` subcommand (verified: 0 uses
  of the `summary` subcommand). The entire run_summary copy of the decode-error counter,
  first-error warning, and `summary.skipped_packets = total_decode_errors` assignment is
  unexercised.
- **Evidence:** Empirical — Mutation M11 changed run_summary's
  `summary.skipped_packets = total_decode_errors` (main.rs:278) to `+ 999` while leaving the
  run_analyze copy (main.rs:183) intact. The full STORY-089 suite passed 17/17 unchanged.
  A real regression in run_summary's skipped-packet accounting would ship silently. The BC's
  own Edge Cases (EC-001..EC-004) and Canonical Test Vectors do not distinguish subcommand,
  so they nominally demand coverage of both entry points.
- **Proposed Fix:** Add at least one `summary`-subcommand test asserting
  `skipped_packets == 73` (and exactly one warning) on `dns-remoteshell.pcap`, mirroring
  AC-003/AC-004 but via `["summary", DNS_REMOTE_FIXTURE, "--json"]`. Alternatively, if the
  story intentionally scopes only run_analyze, the AC traces should say so and BC-2.12.014's
  run_summary anchor should be annotated as covered elsewhere (it is not — no other STORY-089
  test reaches it).

### MEDIUM

#### ADV-P01-MED-001: Header docstring states an arithmetically impossible fixture fact ("73 of 58 total failures" / "Total 58 packets; 73 are non-IP")
- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Confidence:** HIGH
- **Location:** `tests/main_story_089_tests.rs:38-39` (header), `:54-57` (DNS_REMOTE_FIXTURE
  const doc), `:709-710` (EC-005 doc "73 of 58 total failures").
- **Description:** The docstrings assert the fixture has "Total 58 packets; 73 are non-IP and
  fail decode_packet" and "73 of 58 total failures (all non-IP packets)." 73 > 58 is
  impossible, and `packets_analyzed (18) + skipped_packets (73) = 91 ≠ 58`. The binary's own
  summary reports `"total_packets": 58`, `"packets_analyzed": 18`, `"skipped_packets": 73`.
  The 73 counter therefore counts decode *attempts* at a finer granularity than "frames in
  the pcap" (e.g., per-sub-packet / per-record attempts), NOT "73 of the 58 packets." The
  test ASSERTIONS are empirically correct (`== 73` matches the binary); only the prose
  explaining *why* is wrong. This is the STORY-088 "identical-fixture / fixture-fact" lesson:
  the load-bearing constant (73) is right, but the narrative justifying it is internally
  contradictory and will mislead a future maintainer trying to reason about the fixture.
- **Evidence:** `cargo run -- analyze tests/fixtures/dns-remoteshell.pcap --dns --json`
  emits `"total_packets": 58`, `"packets_analyzed": 18`, `"skipped_packets": 73`,
  and exactly 1 warning line. 18 + 73 = 91 ≠ 58; 73 > 58.
- **Proposed Fix:** Correct the prose to reflect that `skipped_packets` counts decode-attempt
  failures (73) which exceeds the 58-frame count because decode is attempted per-record, not
  per-frame (or whatever the true unit is — confirm against `decode_packet` / `PcapSource`).
  Do not change the `== 73` assertions; they are correct. Sweep all three sites (lines 38-39,
  54-57, 709-710) per DF-SIBLING-SWEEP-001 TEST-edits checklist.

#### ADV-P01-MED-002: AC-005 is strictly subsumed by EC-002 (same command, weaker assertion) — redundant and the weaker form masks value-correctness
- **Severity:** MEDIUM
- **Category:** coverage-gap
- **Confidence:** HIGH
- **Location:** `tests/main_story_089_tests.rs:245-252` (AC-005,
  `test_unclassified_flows_injected_into_reassembly_summary`) vs `:607-614` (EC-002,
  `test_EC_002_unclassified_flows_zero_still_present_in_detail`).
- **Description:** Both tests run the identical command `analyze http-ooo.pcap --http --json`.
  AC-005 asserts only `.contains("\"unclassified_flows\"")` (key present); EC-002 asserts
  `.contains("\"unclassified_flows\": 0")` (key present AND value 0). EC-002 strictly
  dominates AC-005 — any mutation AC-005 can kill, EC-002 also kills (confirmed: M3 killed
  both; M4 killed AC-006 not these). AC-005's weaker assertion contributes no independent
  discriminating power. Worse, AC-005's `.contains("\"unclassified_flows\"")` would also pass
  if the value were rendered wrong (e.g., a mutation injecting a hardcoded non-zero or a
  string) — only EC-002 pins the value. This is the STORY-088 identical-fixture lesson: two
  tests on the same fixture+command where one is a strict superset.
- **Evidence:** Lines 248 and 610 issue byte-identical args
  `["analyze", HTTP_FIXTURE, "--http", "--json"]`. Line 251 asserts the key string; line 613
  asserts the key+value. M3/M4 mutation matrix shows AC-005 never kills anything EC-002 does
  not.
- **Proposed Fix:** Either (a) strengthen AC-005 to exercise a *different* fixture/scenario
  that actually distinguishes it (e.g., a fixture with N>0 unclassified flows, pinning the
  non-zero value, which would give independent coverage of the count *plumbing* from the
  dispatcher rather than just the constant 0), or (b) keep AC-005 as the trace anchor but
  document that EC-002 carries the load-bearing assertion. Option (a) is preferred — it would
  also harden against a mutation that hardcodes `unclassified_flows: 0` regardless of the
  dispatcher's real count (such a mutation currently survives both AC-005 and EC-002, since
  the fixture's true value is 0; see ADV-P01-MED-003).

#### ADV-P01-MED-003: No fixture exercises a NON-ZERO unclassified_flows count — the dispatcher→summary value plumbing is untested
- **Severity:** MEDIUM
- **Category:** coverage-gap
- **Confidence:** HIGH
- **Location:** `tests/main_story_089_tests.rs:245-252, 607-614` (all unclassified_flows
  tests); `src/main.rs:205-208` (the `serde_json::json!(dispatcher.unclassified_flows())`
  injection).
- **Description:** BC-2.12.015 postcondition 3 requires the injected value to **equal the
  count from `dispatcher.unclassified_flows()`** (Canonical Test Vector:
  `unclassified_flows()=3 → detail["unclassified_flows"]=3`). Every STORY-089 test uses
  `http-ooo.pcap`, whose true unclassified count is 0 (all flows classify as HTTP). A
  mutation that hardcodes the injected value to a constant `0` — i.e.,
  `serde_json::json!(0)` instead of `serde_json::json!(dispatcher.unclassified_flows())` —
  would survive the entire suite, because the fixture's real count happens to be 0. The
  "value equals dispatcher count" postcondition (not just "key present, value 0") is
  unverified.
- **Evidence:** All unclassified_flows assertions check for the literal `0` (EC-002 line 613)
  or mere key presence (AC-005 line 251). No fixture produces N>0 unclassified TCP flows. The
  canonical vector `unclassified_flows()=3` has no corresponding test.
- **Proposed Fix:** Add a fixture (or reuse an existing one) containing at least one TCP flow
  that is neither HTTP nor TLS (so `dispatcher.unclassified_flows() > 0`), and assert the
  rendered value matches that non-zero count. This closes the dispatcher→summary plumbing and
  kills the `json!(0)`-hardcode mutation. If no such fixture exists, note it as a fixture gap.

### LOW

#### ADV-P01-LOW-001: Header docstring references http.pcap as a used fixture, but it is never used
- **Severity:** LOW
- **Category:** spec-fidelity
- **Confidence:** HIGH
- **Location:** `tests/main_story_089_tests.rs:41` ("http.pcap — 1 HTTP packet; minimal; used
  where tiny fixture suffices").
- **Description:** The header's "Fixtures used" list includes `http.pcap`, but no test
  references it — only `dns-remoteshell.pcap` (DNS_REMOTE_FIXTURE) and `http-ooo.pcap`
  (HTTP_FIXTURE) are used. The "used where tiny fixture suffices" claim is false.
- **Evidence:** `grep 'http\.pcap'` matches only the line-41 comment; all 24 fixture-arg
  call sites use the two named consts (verified). `http.pcap` exists on disk (247 bytes) but
  is dead reference in this suite.
- **Proposed Fix:** Remove the `http.pcap` line from the "Fixtures used" header block, OR
  actually use it for one of the minimal cases (e.g., AC-011 default-to-stdout, which needs
  no decode complexity). Cosmetic; does not affect any assertion.

#### ADV-P01-LOW-002: AC-012 docstring claims "exact strings (verified)" with absolute paths but tests only assert a prefix substring — fine, but the docstring overstates
- **Severity:** LOW
- **Category:** spec-fidelity
- **Confidence:** MEDIUM
- **Location:** `tests/main_story_089_tests.rs:525-527` (docstring) vs `:548, :562`
  (assertions).
- **Description:** The docstring promises exact strings
  `"Failed to write JSON output to /nonexistent/dir/out.json"`, but the assertions use
  `predicate::str::contains("Failed to write JSON output to")` — i.e., the path tail is NOT
  asserted. This is the correct (robust) choice, but the docstring's "exact strings
  (verified by binary run)" framing implies the full path is checked. Mutation M7 confirmed
  the prefix substring is sufficient to kill a context-string change, so the test is adequate;
  only the docstring overstates. Note: asserting the absolute `/nonexistent/dir/...` path tail
  would be brittle anyway (platform-dependent), so the prefix-only assertion is right.
- **Evidence:** Lines 548/562 assert only the prefix; lines 526-527 imply the full path.
- **Proposed Fix:** Soften the docstring to "stderr contains the context prefix
  'Failed to write JSON output to' (the path tail is environment-dependent and not asserted)."

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 3 |
| LOW | 2 |

**Overall Assessment:** pass-with-findings
**Convergence:** findings remain — iterate
**Readiness:** requires revision (HIGH coverage gap on run_summary; MEDIUM plumbing/fixture
gaps). The four CRITICAL mutation-resistance axes from the dispatch all hold — the core
behavioral formalization is sound and mutation-resistant. Remaining findings are
coverage-completeness and docstring-fidelity, not correctness defects in the green tests.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 6 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 |
| **Median severity** | 2.5 (MEDIUM) |
| **Trajectory** | 6 |
| **Verdict** | FINDINGS_REMAIN |
