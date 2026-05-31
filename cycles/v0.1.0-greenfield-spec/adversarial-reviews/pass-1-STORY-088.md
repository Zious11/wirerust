# Adversarial Review — STORY-088 (Implementation) — Pass 1

| Field | Value |
|-------|-------|
| Target | implementation (test formalization, brownfield-formalization mode) |
| Scope | STORY-088 — `tests/main_story_088_tests.rs` (19 tests: 14 AC + 5 EC) + traceability to BC-2.12.008..013, STORY-088.md |
| Strategy under review | brownfield-formalization (zero src changes) — assert_cmd subprocess behavioral tests against `run_analyze`/`resolve_targets` |
| Artifacts read | STORY-088.md; BC-2.12.008/009/010/011/012/013; tests/main_story_088_tests.rs; src/main.rs (full); policies.yaml |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 1 |
| Date | 2026-05-31 |
| Branch | feature/STORY-088-run-analyze-orchestration |
| Worktree HEAD | 698595e |
| Base develop | 45fe526 |
| Verdict | **MEDIUM** (3 MEDIUM mutation-resistance gaps; 0 Critical/High) |

## Checkout Guard (DF-ADVERSARY-CHECKOUT-GUARD-001)

- Branch assertion: `git branch --show-current` = `feature/STORY-088-run-analyze-orchestration` — OK (not develop).
- Grep-count assertion: `grep -c '#[test]'` = 19 test functions = 14 AC + 5 EC — matches story. OK.
- Diff scope: `git diff --stat 45fe526..HEAD` = only `tests/main_story_088_tests.rs` (+667). Zero src changes — matches brownfield-formalization claim. OK.
- Source anchors verified against BC citations: warning eprintln at main.rs:90-94 (BC-2.12.009 PC5), resolve_targets at main.rs:340-360 (BC-2.12.011), use_color at main.rs:43 (BC-2.12.010), `*dns||*all` OR-expansion at main.rs:57-59 (BC-2.12.008), progress bar at main.rs:149-177 (BC-2.12.013). All match. OK.
- Factory artifacts read from main-repo path `/Users/zious/Documents/GITHUB/wirerust/.factory/...` (gitignored in worktree). OK.

## Supplied / Self-Run Evidence (DF-ADVERSARY-TOOLCHAIN-PAIRING-001)

| Axis | Result |
|------|--------|
| `cargo build --bin wirerust` | clean (binary built) |
| `cargo test --test main_story_088_tests` (baseline green) | 19 passed; 0 failed |
| `cargo clippy --test main_story_088_tests` | clean |
| Live binary output inspection | "ANALYZER: DNS/HTTP/TLS" present under `--all`, absent w/o flags; "## Uncategorized" present only under `--mitre`; "Packets: 16/32/0" real; `\x1b` present w/o NO_COLOR, absent with NO_COLOR — assertions are NON-vacuous on the color/analyzer/packet channels |

## Method

Fresh-context attack focused on the CRITICAL axis named in the dispatch:
mutation-resistance. For each behavioral assertion I (a) inspected the real
binary output to confirm the asserted string actually appears/absent on the
correct channel (not a vacuous absence of a never-present string), then (b) ran
LIVE mutations against `src/main.rs` in the worktree, rebuilt the binary, and
re-ran the affected test(s) to confirm the test FAILS when the underlying logic
is broken. `src/main.rs` was backed up to `/tmp` and restored after every
mutation; final `git diff src/main.rs` confirmed clean (zero residual changes),
suite green 19/19, clippy clean.

## Live Mutation Results

| # | Mutation (src/main.rs) | AC/EC tested | Result | Mutation-resistant? |
|---|------------------------|--------------|--------|---------------------|
| 1 | Drop `&& !skip_reassembly` on http_analyzer + tls_analyzer construction (L124/129) | AC-005, AC-003 | both FAILED | YES — the core skip-logic concern is caught |
| 2 | Remove the warning `eprintln!` block (L90-94) | AC-004 | FAILED | YES |
| 3 | `ext == "pcap"` → `ext.eq_ignore_ascii_case("pcap")` (L351) | AC-010, EC-002 | both FAILED | YES — the case-sensitivity concern is caught |
| 4 | Drop `\|\| *all` from http/tls expansion (L58/59) | AC-001 | FAILED | YES |
| 5 | Make `resolve_targets` recurse into subdirs (L346) | AC-011 | FAILED | YES |
| 6 | Change bail text "Target not found" → "Path does not exist" (L359) | AC-012 | FAILED | YES |
| 7 | Force `show_mitre_grouping = true` (L60) | AC-002 | FAILED | YES |
| 8a | Remove `pb.finish_and_clear()` (L177) | AC-013 | **PASSED** | **NO — see F-W25-S088-P1-001** |
| 8b | Remove ENTIRE progress bar (new/set_style/inc/finish) | AC-013, AC-014 | **both PASSED** | **NO — see F-W25-S088-P1-001 / -002** |
| 9 | Gate per-packet DNS analysis on `!skip_reassembly` (L158) | AC-006 | **PASSED** | **NO — see F-W25-S088-P1-003** |
| 10 | Drop NO_COLOR env check from use_color (L43) | AC-007, EC-004 | both FAILED | YES |

10 of 13 mutation vectors are caught. The 3 escapes are documented below. All
are MEDIUM (test-strength gaps on LOW/MEDIUM-confidence BC postconditions); none
is a correctness defect in `src/main.rs` (the live code is correct) and none
blocks merge on its own. They reflect over-stated AC↔test traceability claims.

## Findings

### F-W25-S088-P1-001 [MEDIUM] — AC-013 does not test the progress bar at all (mutation-vacuous)

**AC-013** (story) claims the test proves: "The progress bar appears on stderr
(not stdout) and is finished-and-cleared after each file's packet loop
(`pb.finish_and_clear()` called)." It traces to BC-2.12.013 PC3 + PC4 and to
invariant 2 ("`pb.finish_and_clear()` always called").

`test_progress_bar_does_not_appear_in_output` only asserts that
`analyze … --all --no-color` **stdout** contains no `\x1b`. The progress bar is
written to **stderr** (indicatif default — confirmed in code and BC). Therefore:

- Live mutation 8a (remove `pb.finish_and_clear()`): test PASSED.
- Live mutation 8b (remove the ENTIRE progress bar — `ProgressBar::new`,
  `set_style`, `inc`, `finish_and_clear`): test still PASSED.

The test cannot distinguish "progress bar present, on stderr, cleared" from
"no progress bar at all." Its only real content is "stdout has no ANSI under
`--no-color`," which is a duplicate of the AC-007/EC-004 no-color assertion. It
does NOT verify the `finish_and_clear` invariant (BC-2.12.013 inv 2) nor that
the bar is on stderr-not-stdout in a way that would fail if the bar leaked to
stdout (no positive stderr assertion exists either).

Mitigating context: BC-2.12.013 is explicitly **LOW confidence**, its
Refactoring Notes say "asserting specific ANSI cursor-movement bytes would be
fragile and of low value. Recommend keeping as LOW," and its only VP row is
"manual / visual (LOW confidence — cosmetic UI, no assertion)." So a weak test
here is partially sanctioned by the BC. The defect is the **traceability
overclaim**: the AC-013 prose asserts the test proves `finish_and_clear()` is
called and that the bar is on stderr, which it does not.

**Recommendation (pick one):**
- (a) Downgrade the AC-013 prose to match what the test actually proves
  (stdout carries no progress-bar/ANSI bytes under `--no-color`), and drop the
  "`pb.finish_and_clear()` called" / "appears on stderr" claims from the AC; OR
- (b) Strengthen the test to assert the stderr channel positively (e.g. that
  the run completes with no residual progress-bar artifact left on stderr, or
  that stderr is empty of bar bytes after completion — acknowledging
  indicatif's TTY-detection makes this fragile in a piped subprocess, which is
  exactly why the BC rates it LOW).

Given the BC's LOW rating, (a) is the lower-risk fix.

### F-W25-S088-P1-002 [MEDIUM] — AC-014 `test_run_summary_has_no_progress_bar` is also vacuous w.r.t. progress bars

**AC-014** traces to BC-2.12.013 invariant 4 ("`run_summary` has NO progress
bar"). `test_run_summary_has_no_progress_bar` asserts `summary … --no-color`
stdout has no `\x1b`. `run_summary` (main.rs:247-302) provably constructs no
progress bar — but the test would pass identically whether or not a bar existed,
because (i) any bar would go to stderr, and (ii) `--no-color` strips the only
stdout ANSI source. Live mutation 8b (which removed the analyze-path bar) left
this test green, and there is no `run_summary` bar to remove to even attempt the
inverse mutation. The assertion is a tautology for the invariant it claims to
prove: it is structurally incapable of failing if a progress bar were ever added
to `run_summary`, as long as that bar writes to stderr.

**Recommendation:** Same as F-001 — align AC-014 prose with the real assertion
(`summary` stdout carries no ANSI under `--no-color`) OR, if the no-bar
invariant must be enforced, assert it structurally elsewhere. Note BC-2.12.013
is LOW confidence, so prose alignment is the proportionate fix.

### F-W25-S088-P1-003 [MEDIUM] — AC-006 proves the DNS *section header* renders, not that DNS analysis runs without reassembly

**AC-006** traces to BC-2.12.009 PC6 + invariant 4: "`dns_analyzer` is
constructed independently of reassembly … it operates per-packet."
`test_dns_analyzer_constructed_without_reassembly` runs
`analyze dns-remoteshell.pcap --dns --no-reassemble` and asserts stdout contains
"ANALYZER: DNS" (plus no reassembly warning).

The "ANALYZER: DNS" section header is driven solely by `enable_dns` at
main.rs:211-213 (`if enable_dns { analyzer_summaries.push(dns_analyzer.summarize()); }`),
NOT by whether per-packet DNS analysis actually executed. Live mutation 9 gated
the per-packet DNS path (`if enable_dns && !skip_reassembly && dns_analyzer.can_decode(...)`):

- Test PASSED (false-pass).
- Confirmed effect of the mutation on real output: the "ANALYZER: DNS" header
  still printed, but with `Packets analyzed: 0`, `dns_queries: 0`,
  `dns_responses: 0` — i.e. DNS analysis was fully suppressed yet the test
  could not tell.

So AC-006 proves the section renders (a function of `enable_dns`), not the BC's
actual claim that DNS *operates per-packet* independently of reassembly. The
live code IS correct (DNS construction at L83 is unconditional, analysis at L158
is ungated), but the test does not discriminate the per-packet-operation claim.

**Recommendation:** Strengthen the positive assertion to observe DNS *output
content* that only appears when per-packet analysis runs under `--no-reassemble`
— e.g. assert stdout contains a non-zero DNS count (`dns_queries:` followed by a
non-zero value, or a specific DNS finding the fixture produces) rather than just
the header string. Pin to the dns-remoteshell.pcap's known query count
(`dns_queries: 6` was observed) so the assertion fails if DNS analysis is gated.

## Traceability / Policy Checks

- **DF-AC-TEST-NAME-SYNC-001:** All 14 AC `**Test:**` citations in STORY-088.md
  resolve to exactly one `fn test_*` in `tests/main_story_088_tests.rs`. Names
  match verbatim (AC-001→`test_all_flag_enables_all_three_analyzers`, …,
  AC-014→`test_run_summary_has_no_progress_bar`). Unique across the suite
  (mod-wrapped `mod story_088`). PASS.
- **DF-TEST-NAMESPACE-001:** Tests are wrapped in `mod story_088` in a dedicated
  per-story file. PASS (compliant, not a flat namespace).
- **DF-INPUT-HASH-CANONICAL-001:** Story `input-hash: "5ba42e4"`. Not
  recomputed this pass (read-only profile; no BC content changed in scope).
  Flagged EXECUTION-NOT-REQUIRED for this pass — no inputs changed.
- **Story File-Structure drift (informational, not a finding):** STORY-088.md
  "File Structure Requirements" and "Tasks" still name `tests/cli_tests.rs` and
  `serial_test` as the test location/dependency, but the delivered tests live in
  the dedicated `tests/main_story_088_tests.rs` and use assert_cmd subprocess
  env-injection (no `serial_test`). The test file's own header documents this
  deviation deliberately (DF-TEST-NAMESPACE-001 + "No serial_test required").
  This is a stale story-body artifact, not a test defect; noting for the
  story-writer to reconcile the FSR/Tasks/Library tables with the delivered
  layout. NOT counted as a HIGH/CRITICAL.

## Verdict

**MEDIUM** — 0 Critical, 0 High, 3 Medium. No correctness defect in
`src/main.rs`; 10 of 13 mutation vectors caught including all the dispatch's
"CRITICAL focus" items except the progress-bar pair:

- AC-005 (skip-logic removal): **caught** (mutation 1).
- AC-001/002 (--all enablement + mitre exclusion, non-tautological): **caught**
  (mutations 4, 7).
- AC-010 (case-sensitive .PCAP exclusion): **caught** (mutation 3).
- AC-013 (progress-bar-absent-from-stdout discriminates): **NOT caught** —
  vacuous (F-001); AC-014 likewise (F-002).
- AC-006 (DNS independent of reassembly): partially overclaimed (F-003).

The 3 MEDIUM findings are traceability/test-strength gaps on LOW/MEDIUM-confidence
BC postconditions, two of which (F-001/F-002) the BC itself rates LOW and
sanctions a weak test for. Recommend resolving by aligning AC prose to the real
assertions (F-001/F-002) and strengthening AC-006's positive assertion to a DNS
count (F-003) before convergence. Re-dispatch Pass 2 with these as confirmed
invariants.
