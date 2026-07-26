---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
reviewer_pass: 4
wave: "086"
stories: [STORY-182, STORY-183]
story_versions: {STORY-182: "1.3", STORY-183: "1.3"}
timestamp: 2026-07-25T00:00:00Z
total_findings: 25
severity_tally: {CRIT: 0, HIGH: 4, MED: 12, LOW: 9, NIT: 0}
process_gap_tagged: [F-W86S-P4-010, F-W86S-P4-015, F-W86S-P4-021, F-W86S-P4-025]
novelty: MEDIUM
clean_streak_before: 0
clean_streak_after: 0
remediation_burst: "D-520 (2026-07-25)"
traces_to: STATE.md
---

# Adversarial Review — wave-086 Pass 4

**Stories reviewed:** STORY-182 v1.3 + STORY-183 v1.3
**Pass:** 4 of N (convergence target: 3 consecutive clean passes)
**Novelty:** MEDIUM (first zero-CRIT pass; severity profile decaying; 4 [process-gap] findings)
**Streak before this pass:** 0/3
**Streak after remediation:** 0/3 (reset; pass-4 was blocked — 4 HIGH findings)

---

## Headline

**First zero-CRIT pass.** Severity profile continues decaying across wave-86 passes.
25 findings: 0C/4H/12M/9L. The 4 HIGH findings are mix of substantive spec defects and
process-gap annotations.

**PO Ruling (F-W86S-P4-002):** DF-GREEN-DOC-TENSE-SWEEP v4→v5 number-agnostic token
registry: numbering owned by the tool's `_VIOLATION_PATTERNS` list; docstring scanning =
tracked known-residual class; STORY-183 scrub obligation for `bin/test_lint_cycle_artifact.py:3`
and `:6` (verified genuinely stale; 2 other candidate sites verified NOT stale); grep record
corrected `-rn`→`-rni`; `will be GREEN currently` adjudicated KEEP (dead weight, flagged for
future rationalization).

**Orchestrator Ruling (F-W86S-P4-014):** Narrowly-scoped additive ci.yml step permitted —
fixture coverage report via `--nocapture` — closes the df-validation "loud, machine-readable"
requirement for F-014.

---

## Findings

### F-W86S-P4-001 — HIGH | Resolver-decoupling false-green: test can pass without exercising real resolution path

**Story:** STORY-182 v1.3
**Severity:** HIGH
**Status:** REMEDIATED

STORY-182's resolver-coupling acceptance criterion was structured so a stub implementation
could satisfy it without exercising the actual fixture-path resolution contract. The assertion
was insufficiently tight to distinguish a vacuously-passing resolver from one that correctly
walks the full resolution chain. STORY-182 v1.4 adds a canonical single hard-assert loop that
directly validates the resolved path exists on disk.

---

### F-W86S-P4-002 — MED | Docstring blindness zero-yield: PO scope ruling applied

**Story:** STORY-183 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED (PO scope ruling — known-residual class)

The adversary noted that `bin/check-green-doc-tense` cannot scan Python docstring interiors
(triple-quote blocks) because the comment-line detection mechanism skips them. The tool would
yield zero detections for stale RED-tense prose inside triple-quoted strings. PO ruling: this
is a tracked known-residual class, NOT a tool defect; docstring scanning is explicitly deferred.
STORY-183 v1.4 documents the known-residual class and confirms the two genuinely stale sites
(`bin/test_lint_cycle_artifact.py:3` and `:6`) are scrubbed. Two other candidate sites verified
NOT stale. Separate future story needed for docstring coverage.

---

### F-W86S-P4-003 — HIGH | Absent local-samples prerequisites + no sha256 integrity checks

**Story:** STORY-182 v1.3
**Severity:** HIGH
**Status:** REMEDIATED

The `fetch-e2e-pcaps` prerequisite task was not documented in STORY-182's delivery prerequisites,
and no sha256 integrity check was specified for the committed fixture captures. An AC that
requires pre-fetched files without documenting the fetch step is undeliverable from a clean
checkout without silent failure. STORY-182 v1.4 adds the `fetch-e2e-pcaps` prerequisite and
sha256 integrity checks to the acceptance criteria.

---

### F-W86S-P4-004 — MED | Policy label divergence: v4 vs v5 citation in STORY-183

**Story:** STORY-183 v1.3 / policies.yaml
**Severity:** MEDIUM
**Status:** REMEDIATED

STORY-183 v1.3 cited DF-GREEN-DOC-TENSE-SWEEP v4 in multiple places after the PO updated it
to v5 (number-agnostic). The story used "v4" labels in acceptance criteria and task descriptions
that referenced the updated policy. STORY-183 v1.4 and policies.yaml updated to v5; all
in-story citations corrected to number-agnostic form per the v5 ruling.

---

### F-W86S-P4-005 — LOW | v3 citation residue in STORY-183 task description

**Story:** STORY-183 v1.3
**Severity:** LOW
**Status:** REMEDIATED

A task description in STORY-183 still referenced DF-GREEN-DOC-TENSE-SWEEP v3 after the v4
update in pass-3. Stale version label; no behavioral impact. Cleaned in STORY-183 v1.4.

---

### F-W86S-P4-006 — MED | False quoted-phrase mitigation: acceptance criterion described incorrect phrase class

**Story:** STORY-183 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

An acceptance criterion in STORY-183 described a mitigation for a quoted-phrase class that
does not match the actual TIER-1 token behavior. The mitigation description was inaccurate
with respect to how the tool treats quoted contexts. STORY-183 v1.4 corrects the quoted-phrase
acceptance criterion description.

---

### F-W86S-P4-007 — MED | Fictional .py classification in STORY-183 test plan

**Story:** STORY-183 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

STORY-183's test plan referenced a `.py` classification of the test file set that was
inconsistent with the actual file list. The classified set did not match the 6-file .py set
that the story's bin/*.py glob actually covers. STORY-183 v1.4 corrects the classification
to match the canonical 6-file set.

---

### F-W86S-P4-008 — MED | Rename-site arithmetic + fabricated line reference :899

**Story:** STORY-183 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

STORY-183 claimed a rename-map covering N functional and M prose sites, but the arithmetic
was incorrect, and line reference `:899` was fabricated (no such line exists in the referenced
file). STORY-183 v1.4 corrects to the accurate 6-functional/7-prose rename map and removes
the fabricated line reference.

---

### F-W86S-P4-009 — MED | Missing suffix-scope negative guard in STORY-183 acceptance criterion

**Story:** STORY-183 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

STORY-183's suffix-scoping acceptance criterion did not include a negative guard asserting
that the tool does NOT flag correctly-scoped identifiers. Without the negative guard, an
implementation that flags everything (including correct identifiers) can satisfy all positive
ACs. STORY-183 v1.4 adds the suffix-scope negative GOOD_CASE guard.

---

### F-W86S-P4-010 — HIGH | [process-gap] No e2e positive-coverage in STORY-182 delivery criteria

**Story:** STORY-182 v1.3
**Severity:** HIGH
**Tagged:** [process-gap]
**Status:** REMEDIATED (PG-W86-007 candidate; STORY-182 v1.4 adds CLAUDE.md reference-row task)

STORY-182 lacked an acceptance criterion asserting that the delivered CI-visible fixture
coverage report step is discoverable from CLAUDE.md. New .factory/maintenance/ protocol
documents created during delivery have no discovery obligation, making gate-entry artifacts
invisible to future sessions. STORY-182 v1.4 adds a CLAUDE.md Project References row task.
Captured as PG-W86-007 candidate in the process-gap ledger.

---

### F-W86S-P4-011 — LOW | Cannot-recur overclaim in STORY-183 EC note

**Story:** STORY-183 v1.3
**Severity:** LOW
**Status:** REMEDIATED

An EC note in STORY-183 claimed that a specific defect class "cannot recur" after the
mitigation. This is an overclaim — the mitigation reduces recurrence risk but cannot guarantee
prevention. STORY-183 v1.4 softens the claim to "reduces recurrence risk."

---

### F-W86S-P4-012 — MED | 21-vs-25 stale finding count + wrong glob pattern in STORY-183

**Story:** STORY-183 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

A section of STORY-183 cited "21 findings" (the pass-3 count) when describing the wave-86
adversarial history, and referenced a glob pattern that does not match the current bin/*.py
scope. STORY-183 v1.4 updates the count to 25 (pass-4) and corrects the glob.

---

### F-W86S-P4-013 — LOW | Duplicate assertion prescription in STORY-182 acceptance criteria

**Story:** STORY-182 v1.3
**Severity:** LOW
**Status:** REMEDIATED

Two acceptance criteria in STORY-182 prescribed the same assertion in slightly different
wording, creating ambiguity about whether one or two distinct checks were required. STORY-182
v1.4 collapses these into one canonical single hard-assert loop description.

---

### F-W86S-P4-014 — LOW | Loud-report requirement demoted — orchestrator ci.yml ruling

**Story:** STORY-182 v1.3
**Severity:** LOW
**Status:** REMEDIATED (orchestrator ruling applied)

STORY-182 included an acceptance criterion requiring a "loud, machine-readable" output format
that was adjudicated overly prescriptive relative to the df-validation requirement. Orchestrator
ruling: a narrowly-scoped additive ci.yml step providing a fixture coverage report via
`--nocapture` is sufficient and permitted. STORY-182 v1.4 updated to reflect this ruling.

---

### F-W86S-P4-015 — HIGH | [process-gap] Gate-entry artifact undiscoverable: new .factory/maintenance/ docs lack CLAUDE.md reference

**Story:** STORY-182 v1.3 (delivery obligation)
**Severity:** HIGH
**Tagged:** [process-gap]
**Status:** REMEDIATED (PG-W86-007 candidate; STORY-182 v1.4 Task added)

Any `.factory/maintenance/` protocol document created during story delivery has no
CLAUDE.md Project References row obligation, making it undiscoverable at gate entry.
The wave-86 pass-4 adversary identified this as a structural gap: new protocol docs are
created without a corresponding discovery entry. STORY-182 v1.4 includes a task to add
a CLAUDE.md Project References row for the delivered maintenance document. Captured as
PG-W86-007 candidate in the process-gap ledger.

---

### F-W86S-P4-016 — MED | Vacuous verification: STORY-182 grep check cannot distinguish real from stub

**Story:** STORY-182 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

A grep-based verification step in STORY-182 was structured as a presence-check that a stub
implementation could satisfy without the fixture resolution logic actually executing. STORY-182
v1.4 adds non-vacuous verification greps that distinguish a real fixture-resolution result from
a stub-produced one.

---

### F-W86S-P4-017 — MED | Mis-anchored line reference :648 should be :890

**Story:** STORY-182 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

A line reference in STORY-182's task section cited `:648` for a function that has moved to
`:890` in the current develop tree. Stale line reference would misdirect the implementer.
STORY-182 v1.4 corrects to `:890`.

---

### F-W86S-P4-018 — LOW | Case-count mis-split in STORY-183 test enumeration

**Story:** STORY-183 v1.3
**Severity:** LOW
**Status:** REMEDIATED

A test-case enumeration in STORY-183 split a case count incorrectly between two sub-categories,
summing to the wrong total. STORY-183 v1.4 corrects the split arithmetic.

---

### F-W86S-P4-019 — LOW | Changelog-gate annotation overclaim

**Story:** STORY-183 v1.3
**Severity:** LOW
**Status:** REMEDIATED

An annotation in STORY-183 claimed that a specific changelog-gate check would be enforced
by CI at a level of strictness that the actual `changelog-gate` job does not guarantee.
STORY-183 v1.4 softens the annotation to accurately reflect the CI job's enforcement scope.

---

### F-W86S-P4-020 — LOW | Authority-claim scoping: incorrect claim about which files are scanned

**Story:** STORY-183 v1.3
**Severity:** LOW
**Status:** REMEDIATED

A statement in STORY-183 claimed the tool scans a broader file set than it actually targets.
The authority claim was inconsistent with the bin/*.py glob scope. STORY-183 v1.4 corrects
the scoping claim to match the actual glob.

---

### F-W86S-P4-021 — MED | [process-gap] v4 grep record uses -i flag — case-insensitive defect

**Story:** STORY-183 v1.3 / policies.yaml
**Severity:** MEDIUM
**Tagged:** [process-gap]
**Status:** REMEDIATED

The DF-GREEN-DOC-TENSE-SWEEP v4 grep record used `-rni` (case-insensitive) rather than the
canonical `-rn` (case-sensitive). Token matching in `bin/check-green-doc-tense` is case-
sensitive; the verification grep should match that sensitivity to be meaningful evidence.
A case-insensitive grep could suppress or inflate match counts relative to what the tool
actually detects. PO corrected grep record in v5 to `-rni`→`-rni` — actually, per PO ruling
the corrected flag is `-rni` (keep the `-i` but with explicit documentation of the case-
insensitive choice). Captured as [process-gap] for future policy-authoring discipline.

---

### F-W86S-P4-022 — MED | Sibling loci: STORY-182 sweep did not cover all co-located sister ACs

**Story:** STORY-182 v1.3
**Severity:** MEDIUM
**Status:** REMEDIATED

The pass-3 extended sibling sweep in STORY-182 covered the primary acceptance criterion but
missed co-located sister ACs in the same acceptance criterion block. STORY-182 v1.4 extends
the sweep to cover all sibling loci in the acceptance criterion block.

---

### F-W86S-P4-023 — LOW | Stale traces_to value: .gitignore reference no longer applies

**Story:** STORY-182 v1.3
**Severity:** LOW
**Status:** REMEDIATED

A `traces_to` annotation in STORY-182 referenced a `.gitignore` constraint that no longer
applies to the current delivery approach (committed fixtures in tests/fixtures/ do not need
.gitignore exclusion). Cleaned in STORY-182 v1.4.

---

### F-W86S-P4-024 — LOW | Inert token KEEP adjudication: `will be GREEN currently` retained

**Story:** STORY-183 v1.3
**Severity:** LOW
**Status:** REMEDIATED (KEEP adjudicated)

The phrase `will be GREEN currently` in STORY-183 was identified as dead weight — the
conjunction of future-tense ("will be") and present-tense ("currently") is logically
redundant. PO adjudication: KEEP (phrase is inert in this context; flagged for future
rationalization). STORY-183 v1.4 retains the phrase per ruling with a note for future cleanup.

---

### F-W86S-P4-025 — MED | [process-gap, route orchestrator] Adversary dispatch cited STORY-INDEX.md at wrong path

**Story:** dispatch issue (orchestrator-routed)
**Severity:** MEDIUM
**Tagged:** [process-gap, route orchestrator]
**Status:** REMEDIATED (orchestrator acknowledged; corrected in subsequent dispatches; PG-W86-006 candidate)

The adversary's dispatch instructions for this pass cited `STORY-INDEX.md` at an incorrect
path (not the canonical `.factory/stories/STORY-INDEX.md` path). This caused the adversary
to operate without confirming the current STORY-INDEX version, which could lead to stale
citation errors. Orchestrator acknowledged the path error and corrected it in subsequent
dispatches. Captured as PG-W86-006 candidate: adversary dispatch must glob-verify artifact
paths before sending.

---

## Summary

| Finding | Story | Severity | Class | Status |
|---------|-------|----------|-------|--------|
| F-W86S-P4-001 | STORY-182 | HIGH | spec-defect | REMEDIATED |
| F-W86S-P4-002 | STORY-183 | MED | spec-defect (PO ruling) | REMEDIATED |
| F-W86S-P4-003 | STORY-182 | HIGH | spec-defect | REMEDIATED |
| F-W86S-P4-004 | STORY-183 / policies.yaml | MED | policy-currency | REMEDIATED |
| F-W86S-P4-005 | STORY-183 | LOW | stale-ref | REMEDIATED |
| F-W86S-P4-006 | STORY-183 | MED | spec-defect | REMEDIATED |
| F-W86S-P4-007 | STORY-183 | MED | spec-defect | REMEDIATED |
| F-W86S-P4-008 | STORY-183 | MED | fabricated-ref | REMEDIATED |
| F-W86S-P4-009 | STORY-183 | MED | missing-guard | REMEDIATED |
| F-W86S-P4-010 | STORY-182 | HIGH | [process-gap] | REMEDIATED |
| F-W86S-P4-011 | STORY-183 | LOW | overclaim | REMEDIATED |
| F-W86S-P4-012 | STORY-183 | MED | stale-count + wrong-glob | REMEDIATED |
| F-W86S-P4-013 | STORY-182 | LOW | duplicate-assertion | REMEDIATED |
| F-W86S-P4-014 | STORY-182 | LOW | governance-ruling | REMEDIATED |
| F-W86S-P4-015 | STORY-182 | HIGH | [process-gap] | REMEDIATED |
| F-W86S-P4-016 | STORY-182 | MED | vacuous-verification | REMEDIATED |
| F-W86S-P4-017 | STORY-182 | MED | stale-line-ref | REMEDIATED |
| F-W86S-P4-018 | STORY-183 | LOW | arithmetic | REMEDIATED |
| F-W86S-P4-019 | STORY-183 | LOW | overclaim | REMEDIATED |
| F-W86S-P4-020 | STORY-183 | LOW | scope-claim | REMEDIATED |
| F-W86S-P4-021 | STORY-183 / policies.yaml | MED | [process-gap] | REMEDIATED |
| F-W86S-P4-022 | STORY-182 | MED | sibling-sweep | REMEDIATED |
| F-W86S-P4-023 | STORY-182 | LOW | stale-ref | REMEDIATED |
| F-W86S-P4-024 | STORY-183 | LOW | inert-token (KEEP) | REMEDIATED |
| F-W86S-P4-025 | dispatch | MED | [process-gap, orchestrator] | REMEDIATED |
