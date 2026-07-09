# STORY-158 Implementation Evidence

**Story:** STORY-158 — Wave-71 process-gap codifications: changelog gate, cycle-artifact identity lint, CI scan-guard hardening  
**Branch:** `feature/STORY-158-changelog-gate-cycle-lint`  
**Head reviewed:** 44fbaca  
**Recording date:** 2026-07-09  
**Evaluator:** demo-recorder

---

## AC-158-001: CHANGELOG CI gate (PG-W71-CHANGELOG)

The `changelog-gate` CI job in `.github/workflows/ci.yml` (line 450) runs on
`pull_request` events targeting `develop` only (`if: github.event_name == 'pull_request' &&
github.base_ref == 'develop'`). It diffs `origin/develop...HEAD` and fails when
`src/`, `Cargo.toml`, or `bin/` files are touched without a `CHANGELOG.md` update.

### PASS simulation — this branch diff (trigger hits + CHANGELOG present)

```
$ CHANGED=$(git diff --name-only origin/develop...HEAD)
$ echo "$CHANGED"
.github/workflows/ci.yml
CHANGELOG.md
CLAUDE.md
bin/check-green-doc-tense
bin/lint-cycle-artifact
bin/test_check_green_doc_tense.py
bin/test_lint_cycle_artifact.py

$ TRIGGERS=$(echo "${CHANGED}" | grep -E '^(src/|Cargo\.toml$|bin/)' || true)
$ echo "$TRIGGERS"
bin/check-green-doc-tense
bin/lint-cycle-artifact
bin/test_check_green_doc_tense.py
bin/test_lint_cycle_artifact.py

PASS: CHANGELOG.md updated alongside trigger-set changes.
[exit 0]
```

### FAIL simulation — synthetic diff (bin/ change, no CHANGELOG)

```
$ CHANGED_SYNTHETIC="bin/some-new-tool
src/lib.rs"
$ TRIGGERS=$(echo "${CHANGED_SYNTHETIC}" | grep -E '^(src/|Cargo\.toml$|bin/)' || true)
$ echo "$TRIGGERS"
bin/some-new-tool
src/lib.rs

FAIL: AC-158-001 / PG-W71-CHANGELOG — this PR modifies files in the
CHANGELOG-gate trigger set (src/, Cargo.toml, or bin/) but does not
include a CHANGELOG.md update.

Trigger-set files changed:
bin/some-new-tool
src/lib.rs

Add an [Unreleased] entry to CHANGELOG.md describing the change.
(Reference: AC-158-001 in STORY-158; CI gate introduced in wave-72.)
[exit 1]
```

Message names AC-158-001 and PG-W71-CHANGELOG. Exit is non-zero (not a warning).

---

## AC-158-002 + AC-158-006: CLAUDE.md new sections

### AC-158-002 — CHANGELOG obligation (Git Workflow section)

Text in `CLAUDE.md` (Git Workflow section):

```
- **CHANGELOG obligation (AC-158-001, PG-W71-CHANGELOG):** PRs that modify files under
  `src/`, `Cargo.toml`, or `bin/` MUST include an `[Unreleased]` CHANGELOG entry
  (enforced by CI via the `changelog-gate` job in `.github/workflows/ci.yml`; see
  AC-158-001 and PG-W71-CHANGELOG). `tests/`, `.github/`, `docs/`, and `Cargo.lock` are
  excluded from the trigger set (process-internal or self-documenting surfaces).
```

### AC-158-006 — Wave Gate Code-Review Artifact Protocol (CI / Supply Chain section)

Text in `CLAUDE.md` (CI / Supply Chain section):

```
### Wave Gate Code-Review Artifact Protocol (AC-158-006, PG-W71-CODEREVIEW-ARTIFACT)

Before a wave gate is declared closed, a `cycles/wave-NNN/wave-gate/code-review.md`
artifact MUST be written enumerating every MINOR and NIT finding from the gate-level
code review together with its disposition (accepted / deferred / fixed). A gate with
zero findings MUST still create the file with a "No findings" note. This ensures gate-
level review output is permanently recoverable — the wave-71 gap (PG-W71-CODEREVIEW-
ARTIFACT) showed that a one-line summary in `gate-summary.md` leaves individual finding
text unrecoverable after the review session ends.
```

Both sections reference the relevant PG-W71-* identifiers and AC numbers.

---

## AC-158-003: bin/lint-cycle-artifact (identity lint)

### Test suite: 21/21 pass

```
$ python3 bin/test_lint_cycle_artifact.py

test_tc1_missing_frontmatter:
  [PASS] TC1: missing frontmatter → exit 1, exact rule-1 error message present

test_tc2_empty_bcs_correct_path:
  [PASS] TC2: empty bcs: at correct path → exit 0, PASS message emitted

test_tc3_unresolvable_bc_id:
  [PASS] TC3: unresolvable BC ID → exit 1, ID listed

test_tc4_prose_bc_id_not_flagged:
  [PASS] TC4: prose-only BC ID → exit 0 (not flagged)

test_tc5_missing_bcs_key:
  [PASS] TC5: missing bcs: key → exit 1, rule-1 error

test_tc6_story_id_directory_mismatch:
  [PASS] TC6: story_id mismatch → exit 1, both STORY-999 and STORY-158 named

test_tc7_borrowed_bc_id:
  [PASS] TC7: borrowed BC ID → exit 1, BC-2.11.036 listed

test_tc8_no_wave_intermediate:
  [PASS] TC8: no wave-NNN intermediate → exit 1, exact invalid-path error

test_tc9_comment_interleaved_bcs:
  [PASS] TC9: comment-interleaved bcs: → exit 1, post-comment BC-9.99.999 listed

test_tc10_inline_comment_suffix_stripped:
  [PASS] TC10: inline comment suffixes stripped → exit 0

test_tc11_no_factory_cycles_ancestor:
  [PASS] TC11: no .factory/cycles/ ancestor → exit 1, exact invalid-path error

test_tc12_blank_line_interleaved_bcs:
  [PASS] TC12: blank-line-interleaved bcs: → exit 1, post-blank BC-99.99.999 listed

test_tc13_non_utf8_artifact:
  [PASS] TC13: non-UTF-8 artifact → exit 2, controlled ERROR message (no traceback)

test_tc14_duplicate_story_id_key:
  [PASS] TC14: duplicate story_id: key → exit 1, exact duplicate-key error

test_tc15_zero_indent_bcs_item:
  [PASS] TC15: zero-indent bcs: item → exit 1, BC-9.99.999 listed (not dropped)

test_tc16_wrapped_inline_list_bcs:
  [PASS] TC16: wrapped inline bcs: list → exit 1, wrapped BC-9.99.999 listed

test_tc17_exotic_frontmatter_construct:
  [PASS] TC17: exotic construct (nested map) → exit 1, unsupported-syntax error (fail-closed)

test_tc18_quoted_scalars_accepted:
  [PASS] TC18: quoted story_id + bcs entry → exit 0 (quotes stripped)

test_tc19_scalar_bcs_hard_fails:
  [PASS] TC19: scalar bcs: → exit 1, scalar-bcs error

test_tc20_parent_story_with_hyphen_key:
  [PASS] TC20: parent story with input-hash: → exit 0 (rule 7 reached)

test_tc21_scalar_behavioral_contracts_hard_fails:
  [PASS] TC21: scalar behavioral_contracts → exit 1, malformed-list error

==================================================
Results: 21 passed, 0 failed
All tests passed.
```

### Live hermetic fixture runs (AC-158-003 rule coverage)

Fixtures constructed under `tempfile.TemporaryDirectory()` with `WIRERUST_REPO_ROOT`
set to the temp root, mirroring `bin/test_compute_input_hash.py` convention.
All commands shown as `bin/lint-cycle-artifact --story <story> --artifact <artifact>`.

**PASS: valid artifact, `bcs: []` (EC-005 — rule-2 short-circuit)**

```
$ bin/lint-cycle-artifact --story .factory/stories/STORY-158.md \
      --artifact .factory/cycles/wave-72/STORY-158/pass-artifact.md

PASS: pass-artifact.md identity valid (empty bcs -- no BC claims to validate)
[exit 0]
```

**FAIL rule-1: artifact missing frontmatter entirely**

```
$ bin/lint-cycle-artifact --story .factory/stories/STORY-158.md \
      --artifact .factory/cycles/wave-72/STORY-158/no-frontmatter.md

ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) -- see current cycle-artifact template (STORY-158)
[exit 1]
```

**FAIL rule-3: fabricated BC ID (no on-disk BC file)**

```
$ bin/lint-cycle-artifact --story .factory/stories/STORY-158.md \
      --artifact .factory/cycles/wave-72/STORY-158/fabricated-bc.md
  # artifact bcs: [BC-9.99.999] — no BC file exists at .factory/specs/...

ERROR: unresolvable BC IDs (no on-disk file found): BC-9.99.999
[exit 1]
```

**FAIL rule-6: `story_id:` mismatch (declared STORY-999, dir-derived STORY-158)**

```
$ bin/lint-cycle-artifact --story .factory/stories/STORY-158.md \
      --artifact .factory/cycles/wave-72/STORY-158/wrong-story.md
  # artifact story_id: STORY-999

ERROR: story_id: STORY-999 does not match directory-derived STORY-158
[exit 1]
```

Both the declared value (STORY-999) and the expected value (STORY-158) are named.

**FAIL rule-7: borrowed BC ID (exists on disk, not owned by STORY-158)**

```
$ bin/lint-cycle-artifact --story .factory/stories/STORY-158.md \
      --artifact .factory/cycles/wave-72/STORY-158/borrowed-bc.md
  # artifact bcs: [BC-2.99.001]; STORY-158 behavioral_contracts: []

ERROR: BC IDs not owned by STORY-158 (not in behavioral_contracts:): BC-2.99.001
[exit 1]
```

**FAIL: fail-closed (unsupported YAML shape — exotic nested-map construct)**

```
$ bin/lint-cycle-artifact --story .factory/stories/STORY-158.md \
      --artifact .factory/cycles/wave-72/STORY-158/exotic.md
  # artifact bcs: has nested map value (unsupported syntax)

ERROR: unsupported frontmatter syntax at line 4: 'nested:' -- the lint accepts only the canonical cycle-artifact template forms (see .factory/templates/cycle-artifact.md)
[exit 1]
```

---

## AC-158-004: trust-boundary src/ guard

### Guard block in `.github/workflows/ci.yml` (lines 196–203)

```yaml
# AC-158-004: SEC-001-style src/ existence guard (mirrors help-provenance-gate).
# If src/ is renamed or deleted, grep exits 2 and || true would suppress it,
# yielding a false PASS. The guard fires first so misconfiguration is loud.
if ! test -d src/; then
  echo "FAIL: trust-boundary: src/ directory not found — seam scan target moved?"
  echo "Update the scan target in .github/workflows/ci.yml before merging."
  exit 1
fi
```

### Simulation in a directory without `src/`

```
$ cd <tmpdir-without-src>
$ if ! test -d src/; then
>   echo "FAIL: trust-boundary: src/ directory not found — seam scan target moved?"
>   echo "Update the scan target in .github/workflows/ci.yml before merging."
>   echo "[exit 1]"
> fi

FAIL: trust-boundary: src/ directory not found — seam scan target moved?
Update the scan target in .github/workflows/ci.yml before merging.
[exit 1]
```

After the fix, the `trust-boundary` job cannot silently PASS when `src/` is absent.

---

## AC-158-005: check-green-doc-tense zero-file guard

### Test suite: 55/55 pass (includes AC-158-005 assertion)

```
$ python3 bin/test_check_green_doc_tense.py

...
=== AC-158-005 zero-file guard (must exit non-zero when no files found) ===
  PASS  [zero-file guard: exits non-zero when _collect_rust_files returns [] (AC-158-005)]

Results: 55 passed, 0 failed.
```

The TC patches `_collect_rust_files` to return `[]` and asserts the tool exits
non-zero (exit 1) and emits to stderr:

```
ERROR: no tracked Rust files found; scan target may be wrong. Verify the scan target in bin/check-green-doc-tense.
```

The old behavior was `WARNING: ...` followed by `exit 0` (false CI PASS).
The new behavior exits 1 so the CI job is marked FAILED.

---

## AC-158-007: CHANGELOG [Unreleased] entry with `[process-gap]` provenance

`CHANGELOG.md` `[Unreleased]` section (excerpt):

```markdown
## [Unreleased]

### Added

- **CHANGELOG CI gate, `bin/lint-cycle-artifact`, and `bin/check-green-doc-tense`
  zero-file-guard hardening (STORY-158, wave-72) [process-gap].** Four wave-71 process
  gaps codified as durable project artifacts: (1) `changelog-gate` CI job (pull_request
  only) fails when `src/`, `Cargo.toml`, or `bin/` are modified without a corresponding
  `CHANGELOG.md` update, enforcing the CHANGELOG obligation that wave-71 PRs missed
  (PG-W71-CHANGELOG). (2) `bin/lint-cycle-artifact` (Python 3, stdlib-only) validates
  cycle artifact identity fields (`story_id:` and `bcs:` frontmatter) against the parent
  story and on-disk BC files, catching fabricated or borrowed BC IDs before adversarial
  review (PG-W71-CYCLE-ARTIFACT-IDENTITY). (3) `bin/check-green-doc-tense` now exits
  non-zero when no tracked Rust files are found, preventing a silent false-CI-PASS if the
  scan target moves (PG-W71-CI-SCAN-GUARDS). (4) `trust-boundary` CI job gains a
  `test -d src/` guard before the grep scan, mirroring the SEC-001 pattern in
  `help-provenance-gate` (PG-W71-CI-SCAN-GUARDS).
```

Entry covers all three items required by AC-158-007 (changelog-gate CI step,
`bin/lint-cycle-artifact`, `bin/check-green-doc-tense` zero-file-guard hardening)
and includes `[process-gap]` provenance per VSDD convention.
Bootstrap self-consistency confirmed: the gate introduced by this PR would require
this entry, and the entry is present.

---

## AC-158-008: PR type

PR-time evidence — title uses `ci:` semantic prefix at PR creation.
Expected title: `ci: CHANGELOG gate + cycle-artifact identity lint + scan-guard hardening`

---

## Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate command (`grep -rE` for host-path patterns) returned zero results against
`.factory/cycles/wave-72/STORY-158/`. All host-specific paths scrubbed — temp dir paths
from hermetic fixture runs were replaced with repo-relative placeholders in the
transcripts above. Gate: PASS.
