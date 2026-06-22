# PR #289 Review — docs: F5 sibling sweep (RED-gate → GREEN framing in 5 pcapng test files)

**Verdict: APPROVE** (with one LOW non-blocking finding for follow-up)

Branch: `docs/f5-doctense-sibling-sweep` → `develop`
5 files changed, +263 / -465 (comments / doc-strings / assertion-message text only).
Reviewed as a human reviewer would: diff, PR body, and on-branch source only.

---

## Summary

Documentation-accuracy-only PR. It rewrites stale TDD Red-Gate / `todo!()` prose into
present-tense GREEN regression-guard framing across 5 test files. I verified the change is
genuinely doc-only, that the new prose accurately describes the shipped implementation, and
that no test behavior changed. I found one stale RED-gate comment the sweep did not reach
(non-blocking).

## What I verified

### 1. Doc-only nature — CONFIRMED
- `git diff --name-only` touches **only** `tests/` files. No `src/` file in the diff.
- `#[test]` counts identical develop vs branch: 22/27/20/10/9 — all match.
- Assertion **predicate** call-site counts byte-identical: `predicate::str::contains`
  (11/11), `assert!` (7/7), `assert_eq!` (11/11) in story127; same across all files.
- The only non-comment edits are **assertion failure-message string payloads** (panic /
  `assert!` / `prop_assert_eq!` message text) and removal of trailing `// RED:` inline
  comments. No predicate (`result.is_ok()`, `.success()`, `.failure()`,
  `prop_assert_eq!(... == N ...)`) changed. The story127 `.success()`/`.failure()`
  call-sites are intact (4 success + 1 failure on branch); the apparent -2 token delta in a
  coarse grep was trailing `// RED:` comment text on `.success()` lines, not logic.

### 2. Prose accuracy — CONFIRMED
Every cited source-line reference in the new GREEN prose resolves to the real
implementation on the branch:
- `src/reader.rs:350` → `pub fn pcapng_timestamp_to_secs_usecs(...)` ✓
- `src/reader.rs:652-739` → `pub fn parse_idb_options(body, endianness)` ✓
- `src/reader.rs:770-781` → `pub fn spb_captured_len(original_len, body)` ✓
- `src/main.rs:626-632` → `pub(crate) fn read_magic(path)` ✓
- `src/main.rs:61-89` → `fn format_zero_packet_notice(path, source)` (OPB clause,
  generic-skip segment, pcap-vs-pcapng wording all present) ✓

The past-tense provenance framing ("These tests passed their Red Gate phase (all failed
before implementation) and are now GREEN") is accurate and preserves the TDD-history
signal without claiming the code is still unimplemented.

### 3. Coherence — CONFIRMED
New doc-strings read cleanly and serve their regression-guard purpose: each states what is
now implemented, what behavior the test pins, and what a regression would look like (e.g.
"Regression (re-introducing `?` propagation) would abort on file_a's Err"). Materially more
useful than the old "this MUST FAIL" framing.

## Findings

| Severity | Category | File:Line | Finding | Suggestion |
|----------|----------|-----------|---------|------------|
| LOW | coherence / missing | `tests/bc_2_01_018_story128_tests.rs:1291-1292` | The sweep left one stale RED-gate comment untouched: `// (this currently FAILS too because the notice itself is not yet // implemented — but the gating assertions below pin the format constraint)`. The zero-packet notice **is** implemented (`format_zero_packet_notice`, `src/main.rs:61-89`), so "currently FAILS … not yet implemented" is now false. It sits on a GREEN `.success()` test (`shb-only-gate` neither-segment gate) in the same file this PR edits, but outside any changed hunk. Confirmed pre-existing on `develop`, not introduced here. | In a follow-up (or amend), reword to GREEN framing, e.g.: `// Base notice phrase MUST be present. GREEN: format_zero_packet_notice (src/main.rs:61-89) emits it; the gating assertions below pin the no-parenthetical format for skipped_blocks==0.` Non-blocking — this is exactly the residual the sweep targets, so it is the natural next item, not a defect introduced by this PR. |

## No rubber-stamp note

No predicate, assertion, or test-logic change; no `src/` change; no inaccurate source-line
citation. The new prose matches the shipped code. The single LOW finding is a pre-existing
stale comment the sweep did not reach, not a regression introduced here. Approving;
recommend the story128 line-1291 comment be cleaned up in a follow-up so the file is fully
consistent.
