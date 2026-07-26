---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
reviewer_pass: 3
wave: "086"
stories: [STORY-182, STORY-183]
story_versions: {STORY-182: "1.2", STORY-183: "1.2"}
timestamp: 2026-07-25T00:00:00Z
total_findings: 21
severity_tally: {CRIT: 1, HIGH: 5, MED: 9, LOW: 5, NIT: 1}
process_gap_tagged: [F-W86S-P3-001, F-W86S-P3-003, F-W86S-P3-007, F-W86S-P3-014]
novelty: HIGH
clean_streak_before: 0
clean_streak_after: 0
remediation_burst: "D-519 (2026-07-25)"
traces_to: STATE.md
---

# Adversarial Review — wave-086 Pass 3

**Stories reviewed:** STORY-182 v1.2 + STORY-183 v1.2
**Pass:** 3 of N (convergence target: 3 consecutive clean passes)
**Novelty:** HIGH (headline finding: F-W86S-P3-001 CRIT — PO v3 TIER-1 "0 live uses" claims
falsified by grep; 16 live hits across 3 tokens)
**Streak before this pass:** 0/3
**Streak after remediation:** 0/3 (reset; pass-3 was blocked)

---

## Headline

**F-W86S-P3-001 CRIT** — The PO's DF-GREEN-DOC-TENSE-SWEEP v3 tier table asserted "0 live
uses" for three TIER-1 tokens. Direct grep of the repo tree returned 16 live hits. A policy
document whose motivating evidence is falsified by a 30-second grep cannot be a reliable
steering document for STORY-183's acceptance criteria.

**PO Ruling:** DF-GREEN-DOC-TENSE-SWEEP v3→v4 (committed): grep-verified tier table with
recorded evidence; Patterns 34/36/40 re-tiered TIER-2; §(d) contradiction resolved (tool
pattern list = authoritative TIER-1 registry); `currently fails` site adjudicated legitimate
RED-guard; falls-through-to count corrected 12→10; standing rule: un-grepped tier
assignment is itself a policy violation.

---

## Findings

### F-W86S-P3-001 — CRIT | [process-gap] | PO policy-tier falsification: 16 live hits for 3 "0 live uses" tokens

**Story:** STORY-183 v1.2 / policies.yaml DF-GREEN-DOC-TENSE-SWEEP v3
**Severity:** CRITICAL
**Tagged:** [process-gap]

The DF-GREEN-DOC-TENSE-SWEEP v3 tier table included three TIER-1 tokens with the assertion
"0 live uses in real timed-command test files". Adversarial grep found:
- `currently fails` — 8 hits in src/ and tests/
- `is currently` — 5 hits in tests/
- `still asserts` — 3 hits in tests/
Total: 16 live hits across 3 tokens supposedly having 0 live uses.

A TIER-1 designation means "automated flagging, zero false-positive tolerance." Assigning
TIER-1 to a token with 16 live legitimate uses would cause STORY-183's acceptance tests to
generate 16 false positives on the real codebase — making the delivered tool immediately
non-deployable.

**Root cause:** PO ruling asserted tree state without executing grep verification. The
standing rule in DF-GREEN-DOC-TENSE-SWEEP v4 now requires grep-verified evidence for every
tier assignment.

**Remediation:** PO DF-GREEN-DOC-TENSE-SWEEP v3→v4 (committed); Patterns 34/36/40
re-tiered TIER-2; STORY-183 v1.2→v1.3 pattern set updated; un-grepped tier assignments
are now a policy violation per standing rule.

---

### F-W86S-P3-002 — HIGH | Rename breaks 4 monkey-patch sites in STORY-183

**Story:** STORY-183 v1.2
**Severity:** HIGH

STORY-183 v1.2 renamed the internal function `_is_comment_line` to
`_is_suffix_scoped_comment_line` across the story's AC/code sketch. Four monkey-patch /
override sites in the acceptance-criterion self-test spec retained the old name
`_is_comment_line`, producing a broken test harness on delivery.

**Remediation:** STORY-183 v1.3 — all 4 monkey-patch sites updated to
`_is_suffix_scoped_comment_line`; rename propagated to all 13 self-test reference loci.

---

### F-W86S-P3-003 — HIGH | [process-gap] | changelog-gate-check stdin fictional invocation

**Story:** STORY-183 v1.2
**Severity:** HIGH
**Tagged:** [process-gap]

AC-183-008 specified the `changelog-gate-check` invocation as:
```
changelog-gate-check --stdin < CHANGELOG.md
```
This is a fictional CLI surface. The actual `bin/changelog-gate-check` reads from a file
path argument, not from stdin. The `--stdin` flag does not exist. The acceptance criterion
as written cannot be executed and would fail immediately on delivery.

**Class:** Fictional-invocation (same class as F-W86S-P1-001 from pass-1). Recurred within
one story across two revisions despite pass-1 remediation. Extension of L-W84-002 class.

**Remediation:** STORY-183 v1.3 — AC-183-008 invocation corrected to file-path form;
`bin/changelog-gate-check` contract verified from source.

---

### F-W86S-P3-004 — HIGH | Docstring blindness: tool skips `# type: ignore` and `# noqa` lines

**Story:** STORY-183 v1.2
**Severity:** HIGH

The `_is_suffix_scoped_comment_line` guard in STORY-183 v1.2's design excluded only
`# <word>:` lines (bare section headers). Python inline suppression comments
`# type: ignore`, `# noqa: E501`, and similar would not be excluded, causing false positives
on legitimate suppression annotations if any of the TIER-1 tokens appeared in a suppression
comment suffix.

Known-residual sites: EC-011 (docstring blindness for triple-quoted strings) was also
identified. Four known-residual sites documented and accepted.

**Remediation:** STORY-183 v1.3 — EC-011 (docstring blindness) documented in
Known-Residuals; EC-012 (Rust attribute lines `#[...]`) added as scoped exclusion with
acceptance rationale; 4 known-residual sites enumerated explicitly.

---

### F-W86S-P3-005 — HIGH | Dual-location inert hard-assert in STORY-182

**Story:** STORY-182 v1.2
**Severity:** HIGH

STORY-182 v1.2 specified the fixture-manifest hard-assert in two locations with different
predicates: AC-182-003 said `assert!(count >= 1)` (trivially satisfied) while AC-182-007
said `assert_eq!(count, EXPECTED_MANIFEST_COUNT)` (meaningful gate). The trivially-satisfied
predicate in AC-182-003 would allow an implementation to satisfy the story with an inert
fixture count of 1, bypassing the intent of the gate.

**Remediation:** STORY-182 v1.3 — AC-182-003 predicate updated to direct-path hard-assert
(`assert_eq!(fixture_count, COMMITTED_FIXTURE_COUNT)`); dual-location ambiguity resolved
by making both assertions use the same constant.

---

### F-W86S-P3-006 — HIGH | Mutually exclusive host preconditions in STORY-182

**Story:** STORY-182 v1.2
**Severity:** HIGH

STORY-182 v1.2 AC-182-006 and AC-182-007 required a test to demonstrate BOTH behavior on
a fixture-populated host AND behavior on a clean worktree — but a single test invocation
cannot simultaneously be on both. The preconditions were mutually exclusive, making the
two-environment verification protocol unimplementable as a single test case.

**Remediation:** STORY-182 v1.3 — two-environment verification protocol made explicit:
AC-182-006 covers the fixture-host path (asserts exact count matches committed manifest);
AC-182-007 covers the clean-worktree path (`#[ignore]` gate with documented divergence);
the two ACs are structurally separated.

---

### F-W86S-P3-007 — MED | [process-gap] | Full-TIER-1 overreach + policy self-contradiction

**Story:** STORY-183 v1.2 / policies.yaml DF-GREEN-DOC-TENSE-SWEEP v3
**Severity:** MEDIUM
**Tagged:** [process-gap]

DF-GREEN-DOC-TENSE-SWEEP v3 §(d) stated both: (1) "the tool pattern list is the
authoritative TIER-1 registry" and (2) listed explicit TIER-1 tokens in §(b) that partially
overlapped with the tool pattern list. If the tool pattern list is authoritative, the
policy's own §(b) list is redundant and potentially divergent. The self-contradiction made
the policy document untrustworthy as a steering document.

**Remediation:** DF-GREEN-DOC-TENSE-SWEEP v4 — §(d) contradiction resolved; tool pattern
list is the single authoritative registry; §(b) narrative list is documentation-only (not
definitional).

---

### F-W86S-P3-008 — MED | Rust-attribute latent false positive

**Story:** STORY-183 v1.2
**Severity:** MEDIUM

The `_is_suffix_scoped_comment_line` guard in STORY-183 v1.2 did not exclude Rust
attribute lines (`#[derive(...)`, `#[cfg(...)]`, etc.) from .rs files scanned in the
`bin/*.py` test harness context. A Rust attribute containing a TIER-1 token would produce
a false positive.

**Remediation:** STORY-183 v1.3 — EC-012 added: Rust attribute lines (`#[...]`) are
suffix-scoped by the same guard; known-residual reasoning documented.

---

### F-W86S-P3-009 — MED | False `#[ignore]` rationale in STORY-182

**Story:** STORY-182 v1.2
**Severity:** MEDIUM

STORY-182 v1.2 specified `#[ignore]` on the clean-worktree assertion test with the
rationale "requires network access." The actual rationale is "requires the locally-populated
fixture corpus which is git-ignored and absent on clean checkout / CI." Network access is
not involved.

**Remediation:** STORY-182 v1.3 — `#[ignore]` rationale corrected to "requires
locally-populated git-ignored ITI corpus; absent on clean checkout and CI; divergence
recorded in known-residuals"; `#[ignore]` annotation retained but with accurate justification.

---

### F-W86S-P3-010 — MED | Self-referential count assertions in STORY-182

**Story:** STORY-182 v1.2
**Severity:** MEDIUM

AC-182-009 and AC-182-010 in STORY-182 v1.2 asserted counts derived from the story's own
description text rather than from a committed manifest constant. A story that asserts
`assert_eq!(n, 25)` based on "25 captures described above" creates a circular dependency:
the count is authoritative only because the story says so, not because a manifest was
verified.

**Remediation:** STORY-182 v1.3 — count assertions re-anchored to the committed manifest
(`COMMITTED_FIXTURE_COUNT` constant derived from `tests/fixtures/iti-e2e-manifest.txt`);
runtime-derived manifest checks specified.

---

### F-W86S-P3-011 — MED | CC-BY URI divergence

**Story:** STORY-182 v1.2
**Severity:** MEDIUM

STORY-182 v1.2 cited the CC-BY-4.0 license URI as `https://creativecommons.org/licenses/by/4.0`
(no trailing slash). The canonical form is `https://creativecommons.org/licenses/by/4.0/`
(with trailing slash). Additionally, the attribution referenced the original ITI dataset
name as "IEC-104 ITI Corpus" but the actual dataset has a different name per its CC-BY
attribution record.

**Remediation:** STORY-182 v1.3 — CC-BY URI corrected to canonical trailing-slash form;
rename-accurate attribution added; attribution confirmed against the CC-BY metadata.

---

### F-W86S-P3-012 — MED | E2E-PCAPS sibling loci not swept

**Story:** STORY-182 v1.2
**Severity:** MEDIUM

STORY-182 v1.2 referenced `tests/iec104_e2e_real_pcaps_tests.rs` as the file hosting the
affected fixture-count assertions, but did not include a sweep obligation for the sibling
file `tests/iec104_e2e_pcaps_tests.rs` and any other `*_pcaps_*` test files that may use
the same fixture path resolution. If the sibling file also has stale counts, the story's
ACs would not catch them.

**Remediation:** STORY-182 v1.3 — full `E2E-PCAPS.md` sweep added; acceptance criterion
requires `--error-unmatch` grep across all `*_pcaps_*` test files; 25-capture count
reconciled against sweep evidence.

---

### F-W86S-P3-013 — MED | `ci.yml` `name:` line 462 stale label

**Story:** STORY-183 v1.2
**Severity:** MEDIUM

STORY-183 v1.2 AC-183-012 referenced `.github/workflows/ci.yml` line 462 as the location
of the `bin-selftest` job `name:` field. Line 462 in the current ci.yml contains a
different label (the line has shifted since the AC was written). The AC's line citation
would fail a line-anchored verification.

**Remediation:** STORY-183 v1.3 — ci.yml reference updated to current line and label;
AC-183-012 re-anchored to the correct locus.

---

### F-W86S-P3-014 — MED | [process-gap] | Stale governance artifacts: lessons.md + df-validation carry inaccurate phrase classes

**Story:** wave-085 governance artifacts (lessons.md, planning/df-validation-2026-07-25.md)
**Severity:** MEDIUM
**Tagged:** [process-gap]

`cycles/wave-085/lessons.md` item 5 (PG-W85-003) and
`planning/df-validation-2026-07-25.md` §PG-W85-003 both stated that the 9 stale D-506
sites used `Expected RED:` and `currently falls through` phrase classes. The primary finding
record (`cycles/wave-085/STORY-180/convergence-report.md` lines 63-66) documents the actual
phrase classes as `currently asserts` and `is expected to`.

The inaccurate labels propagated into STORY-183 v1.0 and v1.1 as Patterns 30/31, causing
F-W86S-P2-001 CRIT (zero-match positive-coverage ACs). This is the third instance of the
lesson-summary-vs-finding-record failure mode flagged in df-validation-2026-07-25.md
Cross-Finding Observation 3.

**Remediation (routed to state-manager, this burst):** Correction blocks appended to
`cycles/wave-085/lessons.md` after items 3 and 5; correction block appended to
`planning/df-validation-2026-07-25.md` §PG-W85-003. History not rewritten; corrections
clearly marked "CORRECTION (2026-07-25, F-W86S-P3-014)".

---

### F-W86S-P3-015 — MED | `ls-files` false-green: git-ignored corpus files not visible to git ls-files

**Story:** STORY-182 v1.2
**Severity:** MEDIUM

STORY-182 v1.2 specified a `git ls-files tests/fixtures/` verification step as evidence
that committed fixtures are present. `git ls-files` only lists tracked files; since the
ITI corpus files are git-ignored, they would not appear in `git ls-files` output even on
the fixture-bearing host. The verification step would pass vacuously on both clean checkout
and fixture host, providing no gate signal.

**Remediation:** STORY-182 v1.3 — fixture verification replaced with manifest-based
approach: `tests/fixtures/iti-e2e-manifest.txt` committed as ground truth; gate assertion
reads manifest and verifies each listed file exists on disk; `--error-unmatch` grep used
for the sweep.

---

### F-W86S-P3-016 — LOW | 21→25 count: capture count understated by 4

**Story:** STORY-182 v1.2
**Severity:** LOW

STORY-182 v1.2 AC-182-010 stated "21 committed ITI captures" in the assertion comment. The
correct count from the D-510 gate-fix (#439) evidence is 25 captures (15×TypeID-58/59 +
10×TypeID-61/63×2). The count 21 appears to be a carry-over from an earlier draft.

**Remediation:** STORY-182 v1.3 — count corrected to 25 in all AC loci; citation to
D-510 gate-fix evidence added.

---

### F-W86S-P3-017 — LOW | 12→10 count: falls-through-to count overstated by 2

**Story:** STORY-183 v1.2 / policies.yaml DF-GREEN-DOC-TENSE-SWEEP v3
**Severity:** LOW

DF-GREEN-DOC-TENSE-SWEEP v3 stated "12 `falls through to` sites" as evidence for the
TIER-2 token count. Grep of the tree returned 10 sites. The overcounting (12 vs. 10) was
caused by two lines that contained `falls through to` in a comment about the pattern
itself, not in production code.

**Remediation:** DF-GREEN-DOC-TENSE-SWEEP v4 — count corrected to 10; grep command and
output recorded inline as standing evidence; self-referential lines excluded from count.

---

### F-W86S-P3-018 — LOW | Quote escaping error in STORY-183 AC shell snippet

**Story:** STORY-183 v1.2
**Severity:** LOW

AC-183-011 contained a shell snippet with unbalanced double-quotes around a grep pattern
containing a single-quote. The snippet as written would cause a shell parse error on
execution. The pattern `"currently \\'s"` should be `"currently '"` (or escaped
differently for the intended shell context).

**Remediation:** STORY-183 v1.3 — shell snippet quote escaping corrected; pattern
verified to parse correctly in both bash and zsh.

---

### F-W86S-P3-019 — LOW | Mislabeled AC: "TIER-1" label applied to TIER-2 behavioral assertion

**Story:** STORY-183 v1.2
**Severity:** LOW

AC-183-014 was labeled "TIER-1 behavioral assertion" in the AC heading but the assertion
body described a TIER-2 manual-sweep obligation (checking context before flagging). The
TIER-1 label implied automated zero-FP flagging, but the described behavior required human
judgment.

**Remediation:** STORY-183 v1.3 — AC-183-014 heading corrected to "TIER-2 GOOD_CASE
assertion"; behavioral description aligned with TIER-2 semantics.

---

### F-W86S-P3-020 — NIT | Type annotation missing in Python stub sketch

**Story:** STORY-183 v1.2
**Severity:** NIT

The Python function stub in AC-183-005 lacked a return type annotation (`-> bool`). The
annotation is required for mypy compatibility per the story's own tooling spec (which
requires `mypy --strict` compliance). Missing return type annotation would cause mypy to
infer `None` and fail strict mode.

**Remediation:** STORY-183 v1.3 — return type annotation `-> bool` added to all function
stubs in AC specs; mypy strict mode compatibility verified in acceptance criterion.

---

### F-W86S-P3-021 — LOW | Fictional extension-sensitivity: story assumes `.py` extension check is sufficient

**Story:** STORY-183 v1.2
**Severity:** LOW

STORY-183 v1.2 assumed that filtering `*.py` files by extension was sufficient to avoid
false positives from non-Python files. However, the `bin/*.py` glob can include symlinked
scripts or files without a shebang that would fail the extension-only filter in edge cases
(e.g., a `.py` file that is actually a configuration file). The extension check was
accepted as sufficient by the PO but the story lacked an explicit known-residual note for
this edge case.

**Remediation:** STORY-183 v1.3 — known-residual EC-013 added: extension-only filter
(no shebang validation); accepted as reasonable scope boundary; rationale documented.

---

## Summary Table

| ID | Severity | Story | Tagged | Title | Disposition |
|----|----------|-------|--------|-------|-------------|
| F-W86S-P3-001 | CRIT | STORY-183/policy | [process-gap] | PO TIER-1 "0 live uses" falsified by grep (16 hits) | REMEDIATED — PO v4 grep-verified |
| F-W86S-P3-002 | HIGH | STORY-183 | — | Rename breaks 4 monkey-patch sites | REMEDIATED — v1.3 rename propagated |
| F-W86S-P3-003 | HIGH | STORY-183 | [process-gap] | changelog-gate-check stdin fictional invocation | REMEDIATED — v1.3 invocation corrected |
| F-W86S-P3-004 | HIGH | STORY-183 | — | Docstring blindness + EC-011/EC-012 | REMEDIATED — v1.3 known-residuals documented |
| F-W86S-P3-005 | HIGH | STORY-182 | — | Dual-location inert hard-assert predicate | REMEDIATED — v1.3 direct-path hard-assert |
| F-W86S-P3-006 | HIGH | STORY-182 | — | Mutually exclusive host preconditions | REMEDIATED — v1.3 two-environment protocol |
| F-W86S-P3-007 | MED | STORY-183/policy | [process-gap] | Full-TIER-1 overreach + policy self-contradiction §(d) | REMEDIATED — PO v4 §(d) resolved |
| F-W86S-P3-008 | MED | STORY-183 | — | Rust-attribute latent false positive | REMEDIATED — v1.3 EC-012 added |
| F-W86S-P3-009 | MED | STORY-182 | — | False `#[ignore]` rationale ("requires network") | REMEDIATED — v1.3 rationale corrected |
| F-W86S-P3-010 | MED | STORY-182 | — | Self-referential count assertions | REMEDIATED — v1.3 manifest-constant re-anchor |
| F-W86S-P3-011 | MED | STORY-182 | — | CC-BY URI divergence + attribution rename | REMEDIATED — v1.3 URI + rename corrected |
| F-W86S-P3-012 | MED | STORY-182 | — | E2E-PCAPS sibling loci not swept | REMEDIATED — v1.3 full sweep + --error-unmatch |
| F-W86S-P3-013 | MED | STORY-183 | — | ci.yml `name:` line 462 stale label | REMEDIATED — v1.3 re-anchored |
| F-W86S-P3-014 | MED | governance | [process-gap] | Stale governance artifacts: lessons.md + df-validation phrase classes | REMEDIATED — correction blocks appended (this burst) |
| F-W86S-P3-015 | MED | STORY-182 | — | `ls-files` false-green | REMEDIATED — v1.3 manifest-based approach |
| F-W86S-P3-016 | LOW | STORY-182 | — | 21→25 count: capture count understated | REMEDIATED — v1.3 count corrected |
| F-W86S-P3-017 | LOW | STORY-183/policy | — | 12→10 count: falls-through-to overcounted | REMEDIATED — PO v4 count corrected |
| F-W86S-P3-018 | LOW | STORY-183 | — | Quote escaping error in shell snippet | REMEDIATED — v1.3 escaping fixed |
| F-W86S-P3-019 | LOW | STORY-183 | — | Mislabeled TIER-1/TIER-2 AC heading | REMEDIATED — v1.3 heading corrected |
| F-W86S-P3-020 | NIT | STORY-183 | — | Type annotation missing in Python stub | REMEDIATED — v1.3 `-> bool` added |
| F-W86S-P3-021 | LOW | STORY-183 | — | Fictional extension-sensitivity (known-residual gap) | REMEDIATED — v1.3 EC-013 added |

**All 21 findings REMEDIATED.**
**Clean streak: 0/3 (this pass was blocked; streak reset; pass 4 required).**
