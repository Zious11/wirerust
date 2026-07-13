---
document_type: story
story_id: STORY-166
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-13T00:00:00Z
phase: f7
level: feature
cycle: wave-75
points: 5
priority: P3
depends_on: []
blocks: []
# BC status: pending PO authorship
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: bin/
subsystems: []
estimated_days: 2
wave: "TBD"
traces_to:
  - .factory/cycles/wave-75/process-gap-ledger.md
  - .factory/research/pg-validation-wave-75.md
  - .factory/maintenance/demo-evidence-scrub-gate.md
  - .factory/maintenance/delivery-doc-currency-protocol.md
  - .factory/policies.yaml
  - bin/validate-citations
  - bin/lint-cycle-artifact
  - .factory/cycles/wave-75/wave-gate/code-review.md
inputs:
  - .factory/cycles/wave-75/process-gap-ledger.md
  - .factory/research/pg-validation-wave-75.md
input-hash: "3f3fe8e"
---

# STORY-166: Wave-75 cycle-closing: citation symbol-at-line assertion, finding-ID naming policy, demo-evidence scrub scope extension, mid-gate streak persistence

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 5
**Priority:** P3

## Narrative

- **As a** spec-steward, toolchain maintainer, and future contributor on the wirerust project
- **I want** four process improvements codified: (1) `bin/validate-citations` extended with
  an opt-in `path:line:anchor` grammar that asserts a named symbol exists at the cited line,
  (2) a finding-ID naming policy registered in `.factory/policies.yaml` that canonicalizes
  the `F-W<NN>G-P<n>-<seq>` form and deprecates the G-less variant, (3) the demo-evidence
  scrub discipline extended to `.factory/demo-evidence/` for new captures, and (4) wave-gate
  `findings.md` updated to record every pass verdict (CLEAN passes included) incrementally
- **So that** fabricated symbol names at in-bounds lines no longer pass citation preflight
  silently, wave-gate and story finding IDs are unambiguous across all artifacts, new
  `.factory/demo-evidence/` captures are scrubbed of absolute host paths, and mid-gate
  streak state is rehydratable from persisted findings.md records without relying on
  in-session context

## Behavioral Contracts

_(none -- E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

Wave-75 STORY-165 adversarial convergence and gate code-review surfaced three process gaps
(research-validated in `.factory/research/pg-validation-wave-75.md:15`) and two in-process
gate observations. S-7.02 (cycle-close requirement) mandates codification of wave-execution
process gaps as follow-up stories.

### PG-W75-VALIDATE-CITATIONS-SYMBOL-GAP — citation validator checks line-in-bounds, not symbol-at-line

`bin/validate-citations` validates that cited `file:line` references resolve to a real line
within the file's bounds, but performs no assertion on the content of that line. The citation
grammar regex (`bin/validate-citations:101`) accepts only `path:line[-line]` — no symbol
field. The core loop counts lines via `count_lines()` (`bin/validate-citations:126-128`) and
compares the cited number to the file's line count. No line content is ever read, and no
symbol name is ever asserted.

The gap is not hypothetical. Wave-75 Pass-1 HIGH defect F-S165P1-001 instantiated exactly
this failure class: the test function name `test_T12_malformed_line_counted_in_denominator`
was cited at an in-bounds line; ground-truth is `test_T12_malformed_line_reported` at
`bin/test_validate_citations.py:278`. The cited line number (278) was valid, so the
existing preflight would have passed the citation undetected. The fabrication was in the
symbol name, which the tool does not inspect. The corrected test function name is confirmed
at `bin/test_validate_citations.py:278` (`def test_T12_malformed_line_reported`); T01 is at
`bin/test_validate_citations.py:120` (`def test_T01_valid_line_citation_passes`).

Research validated (VALID) by `.factory/research/pg-validation-wave-75.md:71` ("Recommended
minimal design"). The process-gap is registered at
`.factory/cycles/wave-75/process-gap-ledger.md:22`.

### PG-W75-FINDING-ID-DUAL-SCHEME — wave-74 artifacts use colliding finding-ID schemes

Within wave-74, two different ID forms exist for wave-gate findings: the canonical
`F-W<NN>G-P<n>-<seq>` (G-form, used in gate-summary.md and lessons.md) and the G-less
`F-W<NN>P<n>-<seq>` (used in STORY-164 changelog). Both forms are in live use repo-wide
across multiple waves. With two forms sharing the same pass numbers but pointing to different
findings, authors reach for the non-canonical form and misnumber passes — exactly what
happened with F-S165P4-001 (fabricated ID `F-W74P8-001` corrected to canonical
`F-W74G-P3-001` at all loci).

The real collision axis is G-less `F-W<NN>P<n>` vs canonical `F-W<NN>G-P<n>` — not
"per-story vs wave-gate". The per-story form `F-S<NNN>P<n>` is already unambiguous (begins
`F-S`). Research validated (VALID, with remediation-scope refinement) by
`.factory/research/pg-validation-wave-75.md:15`. Process-gap registered at
`.factory/cycles/wave-75/process-gap-ledger.md:43`.

### PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE — gate W7 observation

`.factory/maintenance/demo-evidence-scrub-gate.md:30-33` scopes the mandatory path-scrub
gate command exclusively to `docs/demo-evidence/`:

```bash
grep -rE '/Users/|/home/|~/' docs/demo-evidence/
```

However, the repository also contains a `.factory/demo-evidence/` tree with 163 host-path
occurrences across 92 files — a pre-existing norm that predates the demo-evidence-scrub-gate
protocol. Any new captures committed under `.factory/demo-evidence/` are not covered by the
current gate and would silently retain absolute host paths.

This is a W7 in-process gate observation — DF-VALIDATION-001-exempt per the in-process
exemption (same pattern as STORY-165 Notes).

### PG-W75-MIDGATE-STREAK-PERSISTENCE — gate W6 observation

Wave-gate `findings.md` records are written only at gate-close (gate-summary.md). CLEAN
passes leave no persisted artifact during an active gate; streak state (consecutive clean
pass count, pass trajectory) is recoverable only from in-session context. If a gate
re-enters mid-wave (e.g., after a PR merge that triggers re-review), the streak counter
must be reconstructed from memory or a manual audit of the gate-summary's pass table rather
than from an append-only findings record.

Recommendation: wave-gate `findings.md` should record EVERY pass verdict incrementally at
pass completion — CLEAN passes as a one-line `Pass N: CLEAN` entry, findings passes with
their finding list. This makes the streak immediately rehydratable from the file at any
mid-gate checkpoint.

This is a W6 in-process gate observation — DF-VALIDATION-001-exempt per the in-process
exemption.

## Acceptance Criteria

### AC-166-001 (traces to PG-W75-VALIDATE-CITATIONS-SYMBOL-GAP — citation symbol-at-line assertion)

`bin/validate-citations` gains an opt-in `path:line:anchor` citation grammar. When a third
colon-delimited field is present, the tool reads that line and asserts the anchor symbol
appears in the line's text. This is backward-compatible: bare `path:line` citations remain
unchanged.

(a) **Grammar extension:** The citation regex is extended to optionally accept a third field
    after the line number: `path:line:anchor` or `path:line-line:anchor`. An `anchor`
    field is any non-blank token following the second colon.

(b) **Symbol assertion:** When an anchor is present, the tool reads line N of the cited file
    and checks that the anchor appears in that line. The check MUST use `re` (stdlib only —
    no ctags or external binary dependencies) and MUST match at least:
    - `def <anchor>` (Python function / method, including `async def`)
    - `fn <anchor>` (Rust function)
    - `class <anchor>` (Python or Rust struct-like)
    A bare substring match against the anchor token is also acceptable as a minimal
    implementation; the `def`/`fn`/`class` prefix patterns are the preferred form.

(c) **New failure class:** If the anchor is not present on the cited line, the tool exits 1
    with the message:
    ```
    SYMBOL NOT AT LINE: path:line (expected anchor '<anchor>', found '<line-text>')
    ```
    The line-text in the failure message MUST be the stripped content of the cited line
    (truncated to ≤80 chars if longer, for readability).

(d) **Backward compatibility:** All existing `path:line` and `path:line-line` citations
    MUST continue to pass without change. The 22 existing tests in
    `bin/test_validate_citations.py` MUST remain green after the extension. No existing
    test's behavior changes.

(e) **New tests:** At least three new tests are added to `bin/test_validate_citations.py`:
    - **T23:** `path:line:anchor` citation where anchor IS present on the cited line — exits
      0 (PASS)
    - **T24:** `path:line:anchor` citation where anchor is NOT present on the cited line —
      exits 1 with `SYMBOL NOT AT LINE` failure class
    - **T25:** bare `path:line` citation on the same file — exits 0 (confirming backward
      compatibility unaffected by the extension)

    Tests MUST use `_run_with_real_files()` as the existing tests do, with controlled
    fixture files. Test function names: `test_T23_anchor_present_passes`,
    `test_T24_anchor_absent_symbol_not_at_line`, `test_T25_bare_citation_still_passes`.

(f) **CHANGELOG obligation:** The AC-166-001 develop PR modifies `bin/validate-citations`
    and `bin/test_validate_citations.py`, both in `bin/`. Per AC-158-001 (trigger set =
    `src/`, `Cargo.toml`, `bin/`), this PR MUST include an `[Unreleased]` CHANGELOG entry.

(g) **ROUTE-W74-DEFERRED items due:** The AC-166-001 develop PR constitutes the
    "next bin-touch PR" for the following deferred findings from wave-74 code review
    (`.factory/cycles/wave-74/wave-gate/code-review.md`) and wave-75 code review
    (`.factory/cycles/wave-75/wave-gate/code-review.md:108-120`,
    `.factory/cycles/wave-75/wave-gate/code-review.md:129`):

    | ID | Severity | File | Description | Action |
    |----|----------|------|-------------|--------|
    | ROUTE-W74 MINOR-1 | MINOR | `bin/test_validate_citations.py` | `_run()` helper is dead code with design mismatch (separate temp dirs for citations file and WIRERUST_REPO_ROOT) | Remove or document |
    | ROUTE-W74 MINOR-2 | MINOR | `bin/validate-citations` | `parse_line()` docstring omits the regex-mismatch `None` return path | Add one-line docstring note |
    | ROUTE-W74 NIT-1 | NIT | `bin/test_validate_citations.py` | `os`, `stat`, `tempfile` imported inline in test bodies instead of at module top | Move to module-level imports |
    | ROUTE-W74 NIT-2 (accepted) | NIT | `bin/changelog-gate-check` | `^+##` filter accepted by design | No action required |
    | ROUTE-W74 NIT-3 (accepted) | NIT | `bin/validate-citations` | `n_valid` name accepted by design | No action required |
    | ROUTE-W74 NIT-4 | NIT | `bin/test_validate_citations.py` | Unnecessary f-string in T21 (no interpolation placeholders) | Remove f-prefix |
    | W75 NIT-1 | NIT | `.github/workflows/ci.yml:465-466, 477, 479` | Hardcoded test counts `(22 tests)`/`(10 tests)` in bin-selftest comment and step names will silently stale when suites grow | Remove parenthetical counts; follow `green-doc-tense-gate` pattern (count-free step names) |

    The W75 NIT-1 disposition was PROPOSED in
    `.factory/cycles/wave-75/wave-gate/code-review.md:129`; this story ratifies it as DUE
    at the AC-166-001 bin-touch PR.

Verification:
```bash
# AC-166-001(a)-(c): anchor grammar and failure class
python3 bin/test_validate_citations.py
# Must pass all tests including new T23/T24/T25

# AC-166-001(d): backward compatibility
grep -c "def test_T" bin/test_validate_citations.py
# Must report >= 25 (22 existing + 3 new)

# AC-166-001(f): CHANGELOG
grep -n "\[Unreleased\]" CHANGELOG.md
# Must emit non-empty output

# AC-166-001(g): W75 NIT-1 resolved
grep -n "(22 tests)\|(10 tests)" .github/workflows/ci.yml
# Must emit empty output (counts removed)
```

### AC-166-002 (traces to PG-W75-FINDING-ID-DUAL-SCHEME — finding-ID naming policy)

A new naming-convention policy is registered in `.factory/policies.yaml`, and
`bin/lint-cycle-artifact` gains a lint check that flags the G-less wave-gate ID pattern in
NEW artifacts.

(a) **Policy registration:** A new policy entry is added to `.factory/policies.yaml`
    following the existing policy structure. The policy MUST have:
    - `id: FINDING-ID-NAMING-001`
    - `name: finding-id-canonical-form`
    - `severity: MEDIUM`
    - Three clauses in `description:`:
      1. **Per-story convergence findings** MUST use `F-S<NNN>P<n>-<seq>` (e.g.,
         `F-S165P4-001`). The `F-S` prefix unambiguously identifies per-story scope.
      2. **Wave-gate findings** MUST use the canonical `F-W<NN>G-P<n>-<seq>` (G before P,
         e.g., `F-W74G-P3-001`). The G-less form `F-W<NN>P<n>-<seq>` (e.g., `F-W74P3-001`)
         is deprecated and disallowed for wave-gate findings.
      3. **Cross-references:** when a story artifact (e.g., a changelog row) records an
         amendment prompted by a wave-gate finding, it MUST cite the canonical wave-gate ID
         (`F-W<NN>G-P<n>-<seq>`), not a G-less variant or an unanchored pass number.

(b) **bin/lint-cycle-artifact extension:** The lint tool is extended to flag the regex
    pattern `F-W\d+P\d` (G-less wave-gate ID form) in NEW artifact files. Historical
    artifacts (all files that pre-date this story's delivery) are explicitly allowlisted or
    treated as exempt. Only files created or modified after the lint rule is added are
    subject to the check.

    Failure message for a G-less pattern detection:
    ```
    WARN: G-less wave-gate finding ID detected: '<match>' in <file> — use F-W<NN>G-P<n>-<seq> form
    ```
    This is a warning (non-blocking) for now; it can be escalated to a hard error in a
    future story once historical occurrences are resolved.

Verification:
```bash
# AC-166-002(a): policy registered
grep -n "FINDING-ID-NAMING-001\|finding-id-canonical-form" .factory/policies.yaml
# Must emit non-empty output

# AC-166-002(b): lint rule present
grep -n "F-W.*P.*G-less\|F-W.d.P.d\|G-less" bin/lint-cycle-artifact
# Must emit non-empty output containing the lint rule
```

### AC-166-003 (traces to PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE — demo-evidence scrub scope extension)

The demo-evidence scrub discipline is extended to cover `.factory/demo-evidence/` for NEW
captures. The 92 pre-existing files (163 host-path occurrences) are exempt with a documented
baseline.

(a) **demo-evidence-scrub-gate.md scope amendment:** A new subsection "`.factory/demo-evidence/`
    — Extended Scope" is added after the existing Gate Command section. It MUST state:
    - The gate command is extended for `.factory/demo-evidence/` — run the path-scrub grep
      against both `docs/demo-evidence/` and `.factory/demo-evidence/` when committing new
      captures to either tree.
    - Pre-existing files (92 files, 163 host-path occurrences as of wave-75 close, 2026-07-13)
      are documented as a baseline exempt from retroactive remediation. Only files created
      or modified AFTER this story's delivery are subject to the extended scope.
    - The extended gate command covering both trees:
      ```bash
      grep -rE '/Users/|/home/|~/' docs/demo-evidence/ .factory/demo-evidence/
      ```

(b) **delivery-doc-currency-protocol.md Step 3 note:** A currency note is added to Step 3
    (`delivery-doc-currency-protocol.md:95`) reminding the sweep operator to check
    `.factory/demo-evidence/` artifacts in addition to `docs/demo-evidence/` for new
    captures committed during the wave's delivery. A one-line note is sufficient:
    > Note: `.factory/demo-evidence/` is also subject to the path-scrub gate for new
    > captures (PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE); see demo-evidence-scrub-gate.md for the
    > extended gate command.

Verification:
```bash
grep -n "\.factory/demo-evidence\|extended scope\|163" \
  .factory/maintenance/demo-evidence-scrub-gate.md
# Must emit non-empty output containing scope extension

grep -n "factory/demo-evidence\|PG-W75-DEMO-EVIDENCE" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output
```

### AC-166-004 (traces to PG-W75-MIDGATE-STREAK-PERSISTENCE — mid-gate streak persistence)

`delivery-doc-currency-protocol.md` is amended (or a new gate-operations note is created)
so that wave-gate operators are required to record every pass verdict (CLEAN passes included)
incrementally to `findings.md` at pass completion.

(a) **Mandatory incremental pass recording:** The currency protocol (or a new gate-operations
    note) MUST state:

    At the start of each adversarial pass during the wave gate, the operator MUST append a
    pass record to the wave's `findings.md` file:
    - For a CLEAN pass: `Pass N: CLEAN (YYYY-MM-DD)` — one line
    - For a findings pass: `Pass N: <count> findings (YYYY-MM-DD)` followed by the finding
      list (IDs and severities)

    These records are written incrementally — at each pass completion, not only at gate-close.
    This ensures the streak counter and pass trajectory are rehydratable from `findings.md`
    at any mid-gate checkpoint without relying on in-session context.

(b) **Placement:** If added to `delivery-doc-currency-protocol.md`, the record MUST appear
    as a new "Step 4 — Incremental Pass Records" within the Mandatory Sweep Steps section,
    or as a separate "Gate Operations" subsection after the Currency Sweep Record section. A
    standalone `.factory/maintenance/gate-operations-note.md` is also acceptable if the
    currency protocol is considered out of scope for this addition.

Verification:
```bash
grep -n "incremental\|Pass N.*CLEAN\|streak\|PG-W75-MIDGATE" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output (OR same check against gate-operations-note.md)
```

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| Citation symbol-at-line assertion | `bin/validate-citations` (amend) | Pure (file-system reads; no network) |
| Symbol assertion tests | `bin/test_validate_citations.py` (amend) | Test harness |
| bin-selftest hardcoded counts fix | `.github/workflows/ci.yml` (amend) | CI configuration |
| ROUTE-W74-DEFERRED housekeeping | `bin/test_validate_citations.py`, `bin/validate-citations` (amend) | Pure / Test harness |
| Finding-ID naming policy | `.factory/policies.yaml` (amend) | Documentation |
| G-less ID lint rule | `bin/lint-cycle-artifact` (amend) | Pure (file-system reads) |
| Demo-evidence scrub scope | `.factory/maintenance/demo-evidence-scrub-gate.md` (amend) | Documentation |
| Step 3 + mid-gate streak note | `.factory/maintenance/delivery-doc-currency-protocol.md` (amend) | Documentation |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `bin/validate-citations` | Pure Python | File-system reads; no network; stdlib only |
| `bin/test_validate_citations.py` | Test harness | Subprocess-based tool invocation |
| `bin/lint-cycle-artifact` | Pure Python | File-system reads; no network |
| `.github/workflows/ci.yml` | CI configuration | Delegates to existing Python scripts |
| `.factory/policies.yaml` | Documentation artifact | Governance YAML |
| `demo-evidence-scrub-gate.md` | Documentation artifact | Governance prose |
| `delivery-doc-currency-protocol.md` | Documentation artifact | Governance prose |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Anchor is present on the line but as a substring of a longer identifier (e.g., `test_T01_valid_line` matching anchor `test_T01`) | Passes — substring match is acceptable; the MVP does not enforce word-boundary anchoring unless the implementer adds `\b` to the regex |
| EC-002 | Anchor field contains special regex characters (e.g., `test_T01_valid_line_citation_passes`) | Tool MUST escape the anchor via `re.escape()` before pattern matching to avoid regex errors |
| EC-003 | `path:line:anchor` citation with a multi-line range (`path:3-7:anchor`) | Anchor assertion applies only to the START line (line 3); range endpoints are still bounds-checked as before |
| EC-004 | `.factory/demo-evidence/` historical files contain host paths that are not scrubbed | Files pre-dating this story's delivery are documented-baseline exempt; the gate applies only to NEW captures committed after delivery |
| EC-005 | `findings.md` does not exist at the start of a wave gate | The gate-operations requirement creates it on first use; an empty-or-absent `findings.md` at the start of Pass 1 is acceptable (write the Pass 1 record as the first entry) |
| EC-006 | G-less wave-gate ID `F-W<NN>P<n>` appears in a historical pre-existing artifact during lint scan | The `bin/lint-cycle-artifact` extension MUST NOT flag historical artifacts; only files modified or created post-delivery are in scope |

## Tasks

1. **Extend bin/validate-citations with anchor grammar (AC-166-001(a)-(e)):** Add optional
   `:anchor` field to the citation regex. When present, read the cited line and assert the
   anchor token appears (using `re.escape()` for safety). Emit `SYMBOL NOT AT LINE: ...`
   failure on mismatch. Keep stdlib-only (no ctags). Add tests T23/T24/T25 to
   `bin/test_validate_citations.py` using `_run_with_real_files()`.

2. **Resolve ROUTE-W74-DEFERRED items in the same develop PR (AC-166-001(g)):**
   - MINOR-1: remove or document the dead `_run()` helper in `bin/test_validate_citations.py`
   - MINOR-2: add one-line docstring note to `parse_line()` in `bin/validate-citations`
     about the regex-mismatch `None` return path
   - NIT-1: move inline `import os`, `import stat`, `import tempfile` from test bodies to
     module top in `bin/test_validate_citations.py`
   - NIT-4: remove the `f`-prefix from the T21 f-string (no interpolation placeholders)

3. **Fix W75 NIT-1 in the same develop PR (W75 code-review.md NIT-1, ratified by AC-166-001(g)):**
   Remove parenthetical test counts from `bin-selftest` step names and comment in
   `.github/workflows/ci.yml` lines 465-466, 477, 479. Follow the `green-doc-tense-gate`
   count-free pattern.

4. **Open develop PR with CHANGELOG entry (AC-166-001(f)):** Create a PR targeting
   `develop` that includes all `bin/` file changes (Tasks 1 and 2) and the ci.yml fix
   (Task 3). The PR MUST include a `CHANGELOG.md` `[Unreleased]` entry (AC-158-001:
   `bin/` is in the trigger set). This is the primary develop-track deliverable.

5. **Register FINDING-ID-NAMING-001 policy in .factory/policies.yaml (AC-166-002(a)):**
   Add the three-clause naming-convention policy using the existing policy YAML structure.
   Factory-artifacts branch commit. Follow the `/vsdd-factory:policy-add` convention for
   policy formatting.

6. **Extend bin/lint-cycle-artifact to flag G-less IDs (AC-166-002(b)):** Add a lint check
   for the regex `F-W\d+P\d` (G-less wave-gate ID form) in NEW artifacts. Historical
   artifacts are exempt (document the exemption criterion in the lint rule's comment).
   Emit a WARN (non-blocking). Include this in the same develop PR as Task 4, or in a
   separate develop PR; either is acceptable. If included in Task 4, it is part of the
   CHANGELOG trigger.

7. **Amend demo-evidence-scrub-gate.md (AC-166-003(a)):** Add the ".factory/demo-evidence/
   — Extended Scope" subsection with the extended gate command and the documented 92-file
   pre-existing baseline. Factory-artifacts branch commit.

8. **Amend delivery-doc-currency-protocol.md (AC-166-003(b) and AC-166-004(a-b)):** Add a
   Step 3 currency note for `.factory/demo-evidence/` scope, and add Step 4 (incremental
   pass records) or a standalone gate-operations subsection for mid-gate streak persistence.
   Factory-artifacts branch commit. Batch both amendments in one burst per
   DF-SIBLING-SWEEP-001.

> **Note for implementer:** The develop PR (Task 4, covering Tasks 1-3 and optionally 6)
> is the primary develop-track deliverable and requires a CHANGELOG entry. Tasks 5, 7, 8,
> and the STORY-INDEX registration are factory-artifacts branch commits. Both tracks must
> complete for the story to be declared delivered. If Task 6 (lint-cycle-artifact extension)
> is included in the develop PR, no separate PR is needed.

## Previous Story Intelligence

Lessons from analogous governance/tooling stories in E-11 — especially STORY-164 and
STORY-165, which immediately precede this story:

- **STORY-165 (wave-75, E-11, 3 pts) — pure-governance precedent:** STORY-165 delivered
  four governance-only ACs (CI wiring, mandate doc, currency protocol, audit-first rule)
  at 3 points. STORY-166 is heavier (5 pts) because AC-166-001 requires real Python code
  changes and tests in `bin/`, unlike STORY-165 which added no new bin/ code.

- **STORY-164 (wave-74, E-11, 4 pts) — bin/ tool delivery precedent:** STORY-164 created
  two new tools (`bin/validate-citations` and `bin/changelog-gate-check`) plus their test
  suites. AC-166-001 extends `bin/validate-citations` rather than creating from scratch,
  which reduces effort. The ROUTE-W74-DEFERRED housekeeping items (2 MINOR + 4 NIT) also
  come due with this PR — they are enumerated in AC-166-001(g).

- **Self-referential quality discipline (meta-irony precedent, STORY-163 and STORY-164):**
  AC-166-001 extends `bin/validate-citations` — the very tool whose preflight gap is being
  closed. The implementer MUST run `bin/validate-citations` on all citation lists in this
  story's Background and ACs before delivery. This story's preflight passed with 14
  citations verified (PASS: 14 citations verified, run 2026-07-13 during story authorship).

- **bin-selftest dogfood (established by STORY-165):** `bin/test_validate_citations.py`
  is wired into the `bin-selftest` CI job (AC-165-001, PR #398). Any additions to the test
  suite (T23/T24/T25) will be automatically exercised by CI on the AC-166-001 develop PR.
  The implementer does not need to add a new CI job — the existing `bin-selftest` job covers
  the extended suite.

## Architecture Compliance Rules

- **stdlib-only for bin/ tools:** `bin/validate-citations` uses Python 3 stdlib only (no
  third-party deps). The anchor assertion extension MUST NOT add any new import that is not
  in the Python 3.10 stdlib. `re` (stdlib) is the correct tool; `ctags`, `universal-ctags`,
  or any external binary is prohibited.
- **Backward-compatible grammar extension:** The existing citation grammar `path:line` and
  `path:line-line` MUST remain valid. The 22 existing tests MUST all pass.
- **test function naming convention:** New tests follow the `test_T<NN>_<description>()`
  naming pattern established by the existing 22 tests.
- **No production Rust source:** No changes to `src/`, `tests/`, or `Cargo.toml`.
- **factory-artifacts vs develop split:** `bin/` changes, ci.yml changes, and CHANGELOG.md
  go on the develop branch in a PR. `.factory/` changes (policies.yaml, maintenance docs,
  STORY-INDEX) go on the factory-artifacts branch.
- **CHANGELOG obligation (AC-158-001):** Any PR touching `bin/` requires a CHANGELOG entry.
  This includes both the primary bin/ changes and any ci.yml changes in the same PR — the
  CHANGELOG entry covers the whole PR.
- **SHA-pinned actions:** The ci.yml amendment for W75 NIT-1 MUST NOT change the
  `actions/checkout` SHA or any other action pin. Only the step name strings change.

## Library & Framework Requirements

- No new Python packages. `re`, `os`, `sys`, `pathlib`, `tempfile`, `subprocess` — all
  stdlib, all already used.
- Python 3.10+ (enforced by `bin/validate-citations` existing constraint).
- No Rust toolchain changes. No Cargo.toml changes.
- No new GitHub Actions used.

## File Structure Requirements

| File | Action | Branch | Notes |
|------|--------|--------|-------|
| `bin/validate-citations` | Modify | develop | Anchor grammar + MINOR-2 docstring fix (AC-166-001) |
| `bin/test_validate_citations.py` | Modify | develop | T23/T24/T25 tests + MINOR-1 + NIT-1 + NIT-4 housekeeping (AC-166-001) |
| `bin/lint-cycle-artifact` | Modify | develop | G-less wave-gate ID lint rule (AC-166-002(b)) |
| `.github/workflows/ci.yml` | Modify | develop | Remove hardcoded test counts from bin-selftest steps (W75 NIT-1) |
| `CHANGELOG.md` | Modify | develop | `[Unreleased]` entry (AC-158-001 obligation) |
| `.factory/policies.yaml` | Modify | factory-artifacts | FINDING-ID-NAMING-001 policy (AC-166-002(a)) |
| `.factory/maintenance/demo-evidence-scrub-gate.md` | Modify | factory-artifacts | Extended scope subsection + 92-file baseline note (AC-166-003(a)) |
| `.factory/maintenance/delivery-doc-currency-protocol.md` | Modify | factory-artifacts | Step 3 scrub note + Step 4 mid-gate streak (AC-166-003(b) + AC-166-004) |
| `.factory/stories/STORY-INDEX.md` | Modify | factory-artifacts | STORY-166 registration (v3.56) |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~4.0 k |
| `bin/validate-citations` (~309 lines + extension) | ~2.5 k |
| `bin/test_validate_citations.py` (~655 lines + new tests) | ~4.5 k |
| `bin/lint-cycle-artifact` (partial read for extension point) | ~1.0 k |
| `.factory/policies.yaml` (existing entries for structure reference) | ~0.5 k |
| `delivery-doc-currency-protocol.md` (amendment target) | ~1.0 k |
| `demo-evidence-scrub-gate.md` (amendment target) | ~0.5 k |
| **Total** | **~14 k** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** PG-W75-VALIDATE-CITATIONS-SYMBOL-GAP and
  PG-W75-FINDING-ID-DUAL-SCHEME are research-validated in
  `.factory/research/pg-validation-wave-75.md` (verdicts VALID and VALID-with-refinement).
  The W6/W7 gate observations (AC-166-004/AC-166-003) are in-process execution findings —
  DF-VALIDATION-001-exempt per the in-process exemption (same pattern as STORY-165 Notes,
  STORY-164 Notes, STORY-163 Notes, STORY-162 Notes).
- **S-7.02 disposition:** Creating this story at draft status codifies three research-validated
  wave-75 process-gap findings (PG-W75-VALIDATE-CITATIONS-SYMBOL-GAP,
  PG-W75-FINDING-ID-DUAL-SCHEME, and PG-W75-GATE-SUMMARY-VERSION-ATTRIBUTION — note: Finding
  3 requires only a one-line factual correction to `gate-summary.md:43` per research report,
  not a separate story; AC-166-002 addresses the root cause) plus two in-process gate
  observations (PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE W7, PG-W75-MIDGATE-STREAK-PERSISTENCE W6)
  for the S-7.02 cycle-close obligation.
- **No behavioral contract required:** E-11 convention (epics.md E-11: "BCs: none
  authored yet -- status: draft; pending PO authorship").
- **Develop/factory split:** AC-166-001 and AC-166-002(b) (`bin/` changes + ci.yml + CHANGELOG)
  touch the develop tree and require a PR. AC-166-002(a) (`.factory/policies.yaml`),
  AC-166-003 (demo-evidence-scrub-gate.md), and AC-166-004
  (delivery-doc-currency-protocol.md) are factory-artifacts branch commits.
- **CHANGELOG obligation for AC-166-001 develop PR:** The PR modifies files in `bin/`
  (AC-158-001 trigger set). A `CHANGELOG.md` `[Unreleased]` entry is REQUIRED. This is
  distinct from STORY-165 (which touched only `.github/workflows/ci.yml`, excluded from the
  trigger set) — adjudication per AC-165-001(b).
- **Points rationale:** 5 pts vs STORY-165's 3 pts (pure governance). AC-166-001 involves
  real Python code changes to `bin/validate-citations`, new tests, and ROUTE-W74-DEFERRED
  housekeeping items that come due with this PR. The remaining three ACs are governance-only.
  This is analogous to STORY-164 (4 pts, which created two new tools) but slightly higher
  because the ROUTE-W74-DEFERRED carry-forward adds implementation surface.
- **Citation preflight:** This story's Background and Acceptance Criteria citations were
  pre-validated by `bin/validate-citations` before writing: 14 citations, all PASS
  (run 2026-07-13 during story authorship).
- **Research report fix applied:** `pg-validation-wave-75.md` line 19 was corrected from
  "Both findings are sound… Neither is covered" to "All three findings are sound… None is
  covered" (F-W75G-P2-001, stale two-finding count; correction footnote added at bottom of
  report per story authorship requirement).
- **Precedent chain:** STORY-166 follows the E-11 S-7.02 pattern: STORY-157 → wave-70;
  STORY-158 → wave-71; STORY-162 → wave-72; STORY-163 → maint-2026-07-09;
  STORY-164 → wave-73; STORY-165 → wave-74; STORY-166 → wave-75.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-13 | story-writer | Initial authorship — wave-75 process-gap codifications: PG-W75-VALIDATE-CITATIONS-SYMBOL-GAP (AC-166-001 citation symbol-at-line assertion + ROUTE-W74-DEFERRED housekeeping), PG-W75-FINDING-ID-DUAL-SCHEME (AC-166-002 finding-ID naming policy + bin/lint-cycle-artifact G-less regex flag), PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE (AC-166-003 extended scrub scope), PG-W75-MIDGATE-STREAK-PERSISTENCE (AC-166-004 incremental pass records). S-7.02 wave-75 cycle-close. bin/validate-citations preflight: PASS on 14-entry anchor list (2026-07-13). Research report pg-validation-wave-75.md line-19 fix applied (F-W75G-P2-001). |
