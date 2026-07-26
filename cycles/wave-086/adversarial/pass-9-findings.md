---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-26T18:30:00Z
cycle: "wave-086"
pass: 9
verdict: NOT_CONVERGED
novelty: HIGH
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 9

**Date:** 2026-07-26
**Pass:** 9 of N
**Verdict:** NOT CONVERGED
**Novelty:** HIGH — all 5 HIGH findings are pass-8 remediation regressions on STORY-182
**Tally:** 12 findings — 0 CRIT / 5 HIGH / 5 MED / 2 LOW + 6 NIT-observations
**Status:** UNREMEDIATED — human paused at strategy fork (D-525 session wrap)

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P9-001 | HIGH | STORY-182 | OPEN | COMMITTED_FIXTURES two-entry residue in AC-182-001:264 → permanent CI test failure |
| F-W86S-P9-002 | HIGH | STORY-182 | OPEN | `include_str!(file!())` does not compile |
| F-W86S-P9-003 | HIGH | STORY-182 | OPEN | `include_str!` coupling vacuous even if compiled (matches registry literal itself) |
| F-W86S-P9-004 | HIGH | STORY-182 | OPEN | Retired discriminator survives at :134/:193-195/:385 |
| F-W86S-P9-005 | HIGH | STORY-182 | OPEN | Task 7/FSR would write false both-committed claim into E2E-PCAPS.md |
| F-W86S-P9-006 | MED | STORY-182 | OPEN | Dissect sha256 gate dropped on false premise (hash IS recorded at E2E-PCAPS.md:359) |
| F-W86S-P9-007 | MED | STORY-182 | OPEN | AC-182-004 1/4 verification lacks WITHOUT-local-samples precondition |
| F-W86S-P9-008 | MED | STORY-182 | OPEN | FIXTURE_GATED_TESTS + E2E-PCAPS.md missing from Arch Mapping/FSR tables |
| F-W86S-P9-009 | MED [process-gap] | STORY-183 | OPEN | bin/check-green-doc-tense:477 glob `src/**/*.rs` NEVER matches top-level src/*.rs (10 unscanned files) |
| F-W86S-P9-010 | MED | STORY-183 | OPEN | 8 of 12 new BAD_CASES + 6 of 14 GOOD_CASES lack prescribed fixture strings; first-match-wins break-on-first constraint undocumented |
| F-W86S-P9-011 | LOW | STORY-182 | OPEN | coverage-out.txt untracked artifact not gitignored |
| F-W86S-P9-012 | LOW [process-gap] | STORY-183 | OPEN | bin/test_lint_cycle_artifact.py modified but not wired into bin-selftest CI job |

---

## Findings Detail

### F-W86S-P9-001 (HIGH) — COMMITTED_FIXTURES two-entry residue → CI failure
**Story:** STORY-182
**Status:** OPEN
**Location:** AC-182-001:264
**Description:** AC-182-001 at line 264 contains a two-entry `COMMITTED_FIXTURES` residue from
a prior revision. The test asserts both the IEC-104 diverse pcap AND a second entry that was
removed from the fixture manifest in v1.7. This residue causes the test to fail with a fixture
count mismatch on every clean CI run. All 5 HIGH findings in this pass are pass-8 remediation
regressions — this one is a direct consequence of the D-524 manifest cardinality rewrite not
propagating cleanly to line 264.

---

### F-W86S-P9-002 (HIGH) — `include_str!(file!())` does not compile
**Story:** STORY-182
**Status:** OPEN
**Description:** AC-182-005's registry coupling mechanism uses `include_str!(file!())` —
embedding the current source file's own content. This is not valid Rust: `file!()` expands
at compile time to the source file path but `include_str!` of the current file is circular
and the pattern `include_str!(file!())` does not compile at all. This finding is a pass-8
regression introduced when the include_str! coupling was added to v1.8.

---

### F-W86S-P9-003 (HIGH) — `include_str!` coupling vacuous even if compiled (self-referential predicate)
**Story:** STORY-182
**Status:** OPEN
**Description:** Even if F-002 were resolved (e.g., by embedding a separate file), the
proposed coupling asserts that the registry literal string appears somewhere in the embedded
content — but the embedded content IS the file that defines the registry literal. The predicate
`content.contains("fn {name}")` would always match because the included file contains its own
definitions. This is an inert/self-referential-predicate — the test can never fail. Fix requires
`contains("fn {name}")` to check the HARNESS file against the STORY file's declared function
names, not the registry file against itself.
**Codification flag:** Inert/self-referential-predicate class at 3+ recurrences (P3-F005,
P5-F001, P9-F003) — adversary flags this as qualifying for lessons-codification at wave-86
cycle-close (S-7.02).

---

### F-W86S-P9-004 (HIGH) — Retired discriminator survives at :134/:193-195/:385
**Story:** STORY-182
**Status:** OPEN
**Location:** STORY-182 body lines :134, :193-195, :385
**Description:** The discriminator block that was restated/rewritten in pass-8 (D-524) has
three surviving stale instances in the v1.8 body at the cited lines. The rewrite addressed the
`##` heading block but did not propagate the ruling to inline references. These three loci
still contain the pre-D-524 discriminator language, creating an internal contradiction in the
story body.

---

### F-W86S-P9-005 (HIGH) — Task 7/FSR would write false both-committed claim
**Story:** STORY-182
**Status:** OPEN
**Description:** Task 7's FSR (Final State Requirements) section instructs the implementer to
write a statement into E2E-PCAPS.md asserting that BOTH the IEC-104 ITI diverse pcap AND the
dissect capture are committed. However, the dissect capture (`TestDissectIec104.pcap`) has been
gitignored since the D-523 single-capture ruling — only the diverse pcap is committed. Task 7's
claim would be false on delivery and would immediately fail any mechanical audit of E2E-PCAPS.md
against the actual committed fixture list.

---

### F-W86S-P9-006 (MED) — Dissect sha256 gate dropped on false premise
**Story:** STORY-182
**Status:** OPEN
**Description:** AC-182-003's sha256 verification gate for the dissect capture was dropped
from v1.8 with the stated premise that the dissect capture is not committed (gitignored).
However, E2E-PCAPS.md line 359 already records the sha256 hash for TestDissectIec104.pcap.
Dropping the gate on this premise is incorrect: (a) the hash record exists, (b) the gate's
purpose is to verify the download, not the committed state. The gate should be retained for
the CI download-and-verify path even though the file is gitignored.

---

### F-W86S-P9-007 (MED) — AC-182-004 1/4 verification lacks WITHOUT-local-samples precondition
**Story:** STORY-182
**Status:** OPEN
**Description:** AC-182-004 specifies 4 verification checks. Item 1/4 verifies that the
fixture manifest is complete. This check is meaningful only when LOCAL_SAMPLES is absent
(clean checkout). When LOCAL_SAMPLES is present, the check trivially passes because all
fixtures are already present. The AC lacks the explicit `WITHOUT-local-samples` precondition
that would make item 1/4 a genuine gate rather than a no-op in the standard CI environment.

---

### F-W86S-P9-008 (MED) — FIXTURE_GATED_TESTS + E2E-PCAPS.md missing from Arch Mapping/FSR tables
**Story:** STORY-182
**Status:** OPEN
**Description:** The Arch Mapping table and FSR (Final State Requirements) tables in STORY-182
v1.8 do not include rows for the `FIXTURE_GATED_TESTS` registry constant or for the E2E-PCAPS.md
file. Both are new artifacts introduced by this story. Their absence from the mapping tables
means the traceability matrix will have gaps — the adversary cannot verify that all introduced
artifacts are accounted for in the story's delivery scope.

---

### F-W86S-P9-009 (MED) [process-gap] — src/**/*.rs glob blind spot in bin/check-green-doc-tense
**Story:** STORY-183
**Status:** OPEN
**Location:** bin/check-green-doc-tense:477
**Description:** The glob pattern `src/**/*.rs` at line 477 NEVER matches files directly in
`src/` (e.g., `src/lib.rs`, `src/main.rs`, `src/mitre.rs`). The git wildmatch semantics for
`**` require at least one intermediate directory component — `src/**/*.rs` expands to
`src/<dir>/<file>.rs` but NOT `src/<file>.rs`. Consequence: 10 top-level src/*.rs files are
never scanned, including `src/mitre.rs` (284 lines). This is a latent blind spot: zero TIER-1
hits are expected in those files today, but future additions are silently unscanned.
**Process-gap note:** The correct pattern is `src/**/*.rs src/*.rs` (two separate globs) or
`'src/**/*'` with a name filter. Fix vehicle decision required at resume (fold into STORY-183
or follow-up story/maintenance item).

---

### F-W86S-P9-010 (MED) — BAD_CASES/GOOD_CASES fixture gaps + break-on-first undocumented
**Story:** STORY-183
**Status:** OPEN
**Description:** STORY-183 v1.8 adds 12 new BAD_CASES entries, but 8 of 12 lack the prescribed
fixture string (the exact phrase expected to appear in the target file). Similarly, 6 of 14
GOOD_CASES entries lack their prescribed fixture strings. A BAD_CASE without a fixture string
cannot be verified mechanically — the implementer must infer the expected phrase from the
description. Additionally, the first-match-wins / break-on-first constraint (when a file
matches multiple patterns, only the first matching pattern fires) is not documented in the AC,
leaving the pattern priority order underdetermined.

---

### F-W86S-P9-011 (LOW) — coverage-out.txt not gitignored
**Story:** STORY-182
**Status:** OPEN
**Description:** STORY-182's Task 3 generates `coverage-out.txt` as an intermediate artifact.
This file is not listed in `.gitignore`. A developer running the coverage step locally will
produce an untracked file that shows up in `git status`. The story should include a task step
to add `coverage-out.txt` to `.gitignore` or specify it as a temporary file that is cleaned
up by the script.

---

### F-W86S-P9-012 (LOW) [process-gap] — bin/test_lint_cycle_artifact.py not wired into bin-selftest CI job
**Story:** STORY-183
**Status:** OPEN
**Description:** STORY-183 v1.8 modifies `bin/test_lint_cycle_artifact.py` (adding new test
cases for the patterns introduced by this story). However, the story does not include a task
to wire `bin/test_lint_cycle_artifact.py` into the bin-selftest CI job. The analogous test
`bin/test_compute_input_hash.py` suffers from the same gap (PG-W84-012 / PG-W86-003). Adding
new test cases to an unregistered selftest file provides no CI enforcement value. Fix requires
coordination with PG-W84-012 ops task (bin-selftest required-status-check).
**Process-gap note:** This connects to the PG-W84-012 ops task pending devops-engineer dispatch
and human authorization. STORY-183 delivery should at minimum assert the selftest passes
locally; the CI wiring is the ops-task gate.

---

## NIT-Observations (6 items — not counted in tally)

The following NIT-level observations were noted but do not constitute actionable findings
requiring remediation before convergence. Recorded for completeness.

1. **NIT-01:** STORY-182 AC-182-001 fixture-count assertion uses a magic constant (4) in two
   separate ACs without extracting to a named constant or cross-referencing the manifest length.
   Future story version drift could desync these.

2. **NIT-02:** STORY-182 Task 2 invocation comment describes `--check` mode behavior but the
   script does not implement a `--check` flag. The comment is aspirational, not descriptive.

3. **NIT-03:** STORY-183 AC-183-003 uses prose "should fail" where all other ACs use "MUST exit
   non-zero". Inconsistent modal verb weakens the acceptance criterion.

4. **NIT-04:** STORY-182 mentions `iec104-iti-diverse.pcap` and `iec104_iti_diverse.pcap`
   (hyphen vs underscore) in different ACs. Should be normalized to one canonical spelling
   matching the committed filename.

5. **NIT-05:** STORY-183's GOOD_CASES entries do not enumerate which known-legitimate RED
   occurrences they cover vs. newly-introduced ones. A reviewer cannot determine if the
   GOOD_CASE set is complete without cross-referencing DF-GREEN-DOC-TENSE-SWEEP v6.

6. **NIT-06:** STORY-182 Task 8 (CI report step) does not specify whether the step runs on
   `push` only or also on `pull_request`. The gate behavior differs between the two triggers;
   the story should specify.

---

## Codification Flag — Inert/Self-Referential Predicate Class

The adversary flags the inert/self-referential-predicate class for lessons-codification at
wave-86 cycle-close (S-7.02). This class has recurred at 3+ passes:

| Pass | Finding | Description |
|------|---------|-------------|
| Pass 3 | P3-F005 | Self-referential predicate in fixture manifest coupling |
| Pass 5 | P5-F001 | Vacuous gate test (passes on empty fixture set) |
| Pass 9 | F-W86S-P9-003 | include_str! coupling matches registry literal itself |

Pattern: story prescribes an assertion that structurally cannot fail because the checked
value and the checking predicate share the same source. Each instance was caught by the
adversary but not recognized as a class until the third recurrence. Recommended action:
add an explicit "inert/self-referential predicate" check to the adversary dispatch protocol
and to the story-writer's AC self-verification step.

---

## Pass-9 Verdict

**NOT CONVERGED.** Streak: 0/3. Passes to convergence: ≥3 clean passes required after
remediation. Human decision as of 2026-07-26: PAUSE at strategy fork. Pipeline PAUSED (D-525).

Pass tallies (P1–P9): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12. Total across all passes:
178 findings. Remediated: 166. Open: 12.

---

## Remediation (D-526)

**Date:** 2026-07-26
**Strategy:** (b) mechanical remediation — chosen by human at session resume
**Burst:** D-526 STATE BURST — WAVE-86 ADVERSARIAL PASS 9 REMEDIATED + PIPELINE RESUMED

All 12 findings FIXED at STORY-182 v1.9 / STORY-183 v1.9 with per-fix grep evidence
per PG-W86-010 mandate.

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P9-001 (HIGH) | FIXED | STORY-182 v1.9 | COMMITTED_FIXTURES two-entry residue removed from AC-182-001:264 |
| F-W86S-P9-002 (HIGH) | FIXED | STORY-182 v1.9 | include_str!(file!()) non-compilable pattern replaced |
| F-W86S-P9-003 (HIGH) | FIXED | STORY-182 v1.9 | include_str! self-referential coupling rewritten to check harness against story function names |
| F-W86S-P9-004 (HIGH) | FIXED | STORY-182 v1.9 | Retired discriminator instances at :134/:193-195/:385 removed |
| F-W86S-P9-005 (HIGH) | FIXED | STORY-182 v1.9 | Task 7/FSR rewritten to reflect only iec104-iti-diverse.pcap committed |
| F-W86S-P9-006 (MED) | FIXED | STORY-182 v1.9 | sha256 gate REINSTATED scoped to CI download-and-verify path (orchestrator ruling: hash at E2E-PCAPS.md:359) |
| F-W86S-P9-007 (MED) | FIXED | STORY-182 v1.9 | AC-182-004 item 1/4 WITHOUT-local-samples precondition added |
| F-W86S-P9-008 (MED) | FIXED | STORY-182 v1.9 | FIXTURE_GATED_TESTS + E2E-PCAPS.md rows added to Arch Mapping/FSR tables |
| F-W86S-P9-009 (MED) [process-gap] | FIXED | STORY-183 v1.9 | src-glob fold-in per human ruling (D-526): pathspec src/*.rs added alongside src/**/*.rs; mitre.rs scan assertion added (DRIFT-src-glob-blindspot RESOLVED-FOLDED) |
| F-W86S-P9-010 (MED) | FIXED | STORY-183 v1.9 | All 12 new BAD_CASES + 14 GOOD_CASES prescribed fixture strings added; break-on-first constraint documented |
| F-W86S-P9-011 (LOW) | FIXED | STORY-182 v1.9 | coverage-out.txt added to .gitignore task step |
| F-W86S-P9-012 (LOW) [process-gap] | FIXED | STORY-183 v1.9 | AC-183-009 local-selftest-pass AC added; CI wiring stays PG-W84-012 ops task per orchestrator ruling (no CI-wiring tasks added to this story) |
| NIT-03 | FIXED | STORY-183 v1.9 | AC-183-003 "should fail" → "MUST exit non-zero" |
| NIT-04 | FIXED | STORY-182 v1.9 | iec104-iti-diverse.pcap / iec104_iti_diverse.pcap spelling normalized |

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; bin/compute-input-hash --write run by story-writer; hook hash warnings advisory-only
per PG-HASH-HOOK-DIVERGENCE).

**STORY-INDEX:** v4.04→v4.05 (wave-86 row v1.8→v1.9 both stories; no numeric totals changed).

**Streak:** 0/3. Pass 10 pending adversary dispatch.
