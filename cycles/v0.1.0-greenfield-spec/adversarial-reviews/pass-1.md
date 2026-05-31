# Adversarial Review — STORY-086 (Implementation) — Pass 1

| Field | Value |
|-------|-------|
| Target | implementation (test formalization) |
| Scope | STORY-086 — `tests/cli_story_086_tests.rs`, branch `feature/STORY-086-cli-subcommand-parsing` |
| Strategy under review | brownfield-formalization (zero src changes) |
| Artifacts read | STORY-086.md; BC-2.12.001/002/003/006; tests/cli_story_086_tests.rs; src/cli.rs; tests/cli_tests.rs (existing); policies.yaml (DF-AC-TEST-NAME-SYNC-001, DF-TEST-CITATION-SWEEP-001, DF-TEST-NAMESPACE-001) |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 1 |
| Date | 2026-05-31 |
| Verdict | CLEAN (no Critical/High; 3 Low/Informational refinements) |

## Method

Fresh-context read of the four behavioral contracts, the story, and the 543-line
test file, cross-checked against `src/cli.rs` (the formalized source) and the
14 pre-existing informal tests in `tests/cli_tests.rs`. Verified Red Gate
integrity from git history (commit `8d2eaa1` stub → `de34e65` implementation),
ran the suite (15/15 pass), ran clippy `-D warnings` and `fmt --check` (both
green). Executed the verification steps for the three in-scope test policies.

## Invariants Confirmed (carry forward to Pass 2+)

- **Struct shape match.** `Commands::Analyze { targets: Vec<PathBuf>, dns, http, tls, mitre, all }`
  and `Commands::Summary { targets: Vec<PathBuf>, hosts }` in the tests exactly
  match `src/cli.rs:113-155`. `no_color: bool` global flag matches `cli.rs:44-45`.
  Exhaustive `match` arms (no wildcard arm on the variant-discriminating tests)
  would force a compile error if the variant set drifted.
- **`required = true` on `targets`** — AC-003 asserts `ErrorKind::MissingRequiredArgument`,
  matches `cli.rs:117` / `cli.rs:144`. Verified.
- **`--services` removed** — AC-007/EC-004 assert `ErrorKind::UnknownArgument`;
  `grep services src/cli.rs` returns nothing. Matches BC-2.12.002 inv 4 + LESSON-P1.04.
- **`no_color` is plain `bool`, not `Option<bool>`** — AC-009 + AC-006 enforce via
  `let _: bool = ...` type assertions that fail to compile under `Option<bool>`.
  Matches BC-2.12.003 inv 2.
- **`--no-color` global semantics** — AC-008 covers all three placements (before
  subcommand, after subcommand name, after positional). Matches `global = true`
  at `cli.rs:44`.
- **Order + duplicate preservation** — AC-010 / EC-005 assert positional order and
  no parse-time dedup. Matches BC-2.12.006 post 1-2.
- **DF-AC-TEST-NAME-SYNC-001 CLEAN** — all 10 AC `**Test:**` citations resolve to
  exactly one `fn test_*` in the cited file; no ambiguity, no zero-match.
- **DF-TEST-NAMESPACE-001 CLEAN** — all 15 tests wrapped in `mod story_086`; no
  flat-namespace test functions. BC-prefixed `test_EC_00N_*` names collision-safe.
- **DF-TEST-CITATION-SWEEP-001 CLEAN** — no re-pointing occurred this story; header
  comment block maps AC/EC labels to function names consistently.
- **Red Gate integrity** — stub commit `8d2eaa1` used `assert!(false, "RED GATE STUB…")`
  on every test (correct for brownfield-formalization where source pre-exists and
  must not be stubbed). `de34e65` replaced stubs with real assertions. Source
  (`src/cli.rs`) untouched across all three commits — confirms zero-src-change claim.

## Findings

### F-P1-001 (LOW / Informational) — `-a` short flag for `--all` is untested

BC-2.12.001 Description explicitly states: "plus the short form `-a` for `--all`",
and `src/cli.rs:137` declares `#[arg(short, long)]` on `all`, so `-a` is a live
part of the contracted surface. No AC or EC exercises `-a`. AC-002/EC-001 only use
the long `--all` form. This is a coverage gap against the BC, not a defect — the
behavior works — but the short form is an un-formalized branch of an in-scope BC.
Severity LOW because STORY-086's ACs do not claim to cover it and the long form is
covered; a future hardening test (`["wirerust","analyze","-a","cap.pcap"] → all=true`)
would close it.

### F-P1-002 (LOW / Informational) — EC-005 (BC-2.12.006) "quoted path with spaces" not formalized

BC-2.12.006 EC-005 ("Target with spaces in path (quoted) → single PathBuf") has no
corresponding test. The story's own EC table (EC-005) re-scopes to duplicate-target
preservation instead, and the story does not claim BC-2.12.006 EC-005. So this is a
BC edge case the story consciously did not pull in, not a story-to-test drift.
Recorded for completeness; no action required for STORY-086 acceptance.

### F-P1-003 (LOW / Informational) — AC-008 doc-comment cites "BC-2.12.003 EC-002" for the post-positional placement; BC EC-002 wording is "after subcommand and targets"

The test doc-comment for the third placement (`analyze cap.pcap --no-color`) cites
"BC-2.12.003 EC-002". BC-2.12.003 EC-002 reads "--no-color after subcommand and
targets → no_color=true (global flag)", which matches. The second placement
(`analyze --no-color cap.pcap`) is cited as "EC-001 / global = true" — but BC EC-001
is "before subcommand". This is a minor citation imprecision in a doc-comment (the
mid-position case is technically neither BC EC-001 nor EC-002 verbatim; it is an
additional valid placement implied by `global = true`). Assertion correctness is
unaffected. Severity LOW; cosmetic. Could relabel to "global = true (placement not
enumerated verbatim in BC EC table)".

## Novelty Assessment

Pass 1 baseline. 0 Critical, 0 High, 0 Medium, 3 Low/Informational. All three Low
findings are coverage/citation refinements against the broader BC surface, none of
which contradict STORY-086's stated ACs or the zero-src-change brownfield mandate.
The implementation faithfully formalizes the 10 ACs and 5 ECs the story claims.

## Recommendation

CLEAN pass. The three Low findings do not block acceptance — they document BC-surface
coverage gaps the story did not claim and one cosmetic doc-comment citation. Minimum
3 clean passes required by the skill; continue to Pass 2 with the invariant list above
carried forward.
