---
document_type: process-gap-ledger
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-07-25T23:42:00Z
cycle: "wave-086"
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
---

# Process-Gap Ledger — wave-086

Process-gap candidates captured during wave-086 for cycle-close codification.
Each entry requires DF-VALIDATION-001 research-agent validation before filing
as a GitHub issue (product-local) or upstream drbothen/vsdd-factory issue.

---

## PG-W86-001 — story-writer lacks positive-coverage-assertion checklist for detector/gate stories

**Class:** Story-writer checklist gap / positive-coverage assertion discipline
**Caught by:** Wave-86 adversarial pass 1 (F-W86S-P1-007 HIGH) + pass 2 (F-W86S-P2-002 HIGH)
**Severity:** HIGH (pattern recurred identically in both STORY-182 and STORY-183 during the
  same wave; positive-coverage assertions were either absent or grounded against fabricated
  inputs rather than real stale sites)
**Occurrences:** 2 stories in wave-86 (same defect shape: pass-1 added positive-coverage ACs
  grounded against wrong/fabricated inputs; pass-2 found those ACs ineffective)
**Source finding:** F-W86S-P1-007 (HIGH, wave-86 pass 1) + F-W86S-P2-002 (HIGH, wave-86 pass 2)
**Vehicle:** Local carry-forward (DF-VALIDATION-001 required before filing upstream)

### Description

When story-writer drafts a "detector" or "gate" story (a story whose primary behavior is
detecting/rejecting a bad condition), it systematically omits or mislabels positive-coverage
acceptance criteria — ACs that assert the tool DETECTS a stale/bad artifact when one is
present (the "red path").

The pattern manifested twice in wave-86:

1. **STORY-183 (pass-1):** Pass-1 added AC-183-007/008 as positive-coverage ACs, but they
   were grounded against Patterns 30/31 (`currently falls through`, `is expected to`) which
   came from a lesson-summary that mislabeled the real phrase classes. Pass-2 found that
   these two patterns matched ZERO of 9 real stale sites from D-506 — the positive-coverage
   ACs were testing against fabricated inputs, not real ones (F-W86S-P2-001 CRIT,
   F-W86S-P2-002 HIGH).

2. **STORY-182 (pass-1 + pass-2):** Gate test (`fixture_manifest_all_present`) was specified
   without requiring it to actually fail on clean checkout — the test design made it possible
   to satisfy all ACs with a vacuously-passing implementation. Multiple medium findings
   (F-W86S-P2-007/008/009) revealed that gate tests lacked the failure-mode specification
   needed to make them truly gate-capable.

### Root Cause

Story-writer does not carry a mandatory "positive-coverage checklist" for detector/gate
stories:
- Does this story have an AC that asserts detection FIRES when a real stale input is present?
- Are the positive-coverage fixture inputs sourced from the real finding record (adversarial
  pass convergence reports, PR reviews) rather than from lesson summaries or description text?
- Does the gate test specify an explicit FAILURE mode (not just a success mode)?

### Proposed Fix

Add a **positive-coverage checklist** to the story-writer agent's `STORY.md` template for
`type: maintenance` + `epic: E-11` (tooling/detector) stories:

```
## Positive Coverage Checklist (detector/gate stories)
- [ ] AC asserting the tool exits non-zero on at least one real-world stale input
- [ ] Fixture text sourced from actual finding records (not lesson summaries or descriptions)
- [ ] Gate test specifies failure mode (not only success mode)
- [ ] Efficacy anchor: specific phrase/pattern from a named prior finding (D-NNN citation)
```

This checklist should be mandatory for any story whose primary AC is "tool rejects X" or
"test fails when Y absent".

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). DF-VALIDATION-001 research-agent validation
required before determining vehicle (local template change vs. upstream vsdd-factory issue).

---

## PG-W86-002 — ground efficacy ACs in source finding record, not lesson summaries

**Class:** Specification grounding discipline / finding-record citation mandate
**Caught by:** Wave-86 adversarial pass 2 (F-W86S-P2-001 CRIT, F-W86S-P2-005 HIGH)
**Severity:** HIGH (CRIT finding in pass-2; efficacy ACs tested against fabricated inputs)
**Occurrences:** 1 story (STORY-183) in wave-86; related to PG-W84-002 extension
**Source finding:** F-W86S-P2-001 (CRIT), F-W86S-P2-005 (HIGH)
**Vehicle:** Local carry-forward (extends PG-W84-010 scope; DF-VALIDATION-001 required)

### Description

STORY-183 v1.1 AC-183-009 specified `Expected RED: TypeID 58` as the D-506 efficacy fixture.
This came from the wave-85 lesson summary (`cycles/wave-085/lessons.md`) which described the
pass-1 adversary finding in lesson-summary prose. The lesson summary said "Expected RED:"
was the stale phrase class — but the actual convergence report
(`cycles/wave-085/STORY-180/convergence-report.md` lines 63-66) showed that the real stale
phrases observed were `currently asserts` and `is expected to`.

The lesson summary was an accurate description of the broader phrase gap, but the specific
finding-record evidence (convergence-report.md) showed different phrase text. Story-writer
cited the lesson summary without cross-referencing the primary finding record.

### Root Cause

When story-writer constructs efficacy ACs that cite prior findings (e.g., "D-506 showed that
pattern X occurs"), it must cite the **primary finding record** (adversarial pass convergence
report, PR review diff, or BC violation log) — not lesson summaries or STATE.md decision
entries, which are secondary digests.

Lesson summaries are intentionally condensed; they may rename or generalize phrase classes
for readability, which causes the derived AC to diverge from the actual stale text.

### Proposed Fix

Extend the story-writer's citation discipline (PG-W84-010 scope) to require that efficacy
ACs citing prior adversarial findings MUST include a direct path citation to the primary
finding record:

```
AC-NNN-YYY: Given input containing `{exact_phrase}` (per {source-doc}:{line}),
  tool exits non-zero.
  Source: cycles/wave-085/STORY-180/convergence-report.md:63-66
```

The `{source-doc}:{line}` citation must point to the primary record, not to STATE.md or
lessons.md. This prevents lesson-summary paraphrase from substituting for ground truth.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Extends PG-W84-010 scope (citation mandate
for story-writer). DF-VALIDATION-001 research-agent validation required.

---

## PG-W86-003 (adjacent) — bin-selftest CI gate gap (PG-W84-012 scope extension)

**Class:** CI gate coverage gap / required-status-checks
**Caught by:** Wave-84 S-7.02 (PG-W84-012, original) + wave-86 gate assessment
**Severity:** MEDIUM (bin/ Python self-tests pass in manual invocation but are not
  enforced as a required GitHub status check on PRs)
**Occurrences:** Persists from wave-84; STORY-183 delivery will add a new bin/ test
  (`bin/test_check_green_doc_tense.py`) that also lacks CI gate enforcement
**Source finding:** PG-W84-012 (original) + STORY-183 scope addition
**Vehicle:** Ops task (devops-engineer dispatch + human authorization) — NOT a story

### Description

`bin/test_compute_input_hash.py` and the forthcoming `bin/test_check_green_doc_tense.py`
both run during the wave-gate `Gate 1` manual CI verification but are NOT listed as required
status checks for `develop` branch protection. This means a PR that breaks a bin/ self-test
can be merged via GitHub UI without CI blocking it.

This gap is not new (PG-W84-012). It is noted here because STORY-183 adds a second bin/
self-test to the same surface, increasing the exposure.

### Cross-Reference

- PG-W84-012 (original finding, D-486): "bin-selftest required-status-check gap; bin/
  Python self-tests pass in Gate 1 but not enforced as required status check".
- STATE.md Active Carry-Forwards: PG-W84-012 row: "Ops task PENDING: bin-selftest →
  develop required-status-checks; devops-engineer + human authorization required for
  branch-protection mutation."

### Disposition

Separate from STORY-183 (different gate surface — branch protection, not tool logic).
Devops-engineer dispatch + human authorization required. Not a story. PG-W84-012 remains
the canonical tracking entry. This row notes the scope extension only.

---

---

## PG-W86-004 — policy tier/token assertions must carry grep-verified evidence at ruling time

**Class:** PO policy-discipline / evidence grounding
**Caught by:** Wave-86 adversarial pass 3 (F-W86S-P3-001 CRIT) + pass 2 arc (F-W86S-P2-006 HIGH);
  SECOND INSTANCE: wave-86 adversarial pass 6 (F-W86S-P6-009/010 MED — bare-RED v5 assignment)
**Severity:** CRIT (the CRIT finding it produced — F-W86S-P3-001 — falsified 3 TIER-1 tokens
  that would have generated 16 false positives on a deployed tool)
**Occurrences:** Standing rule applied twice in wave-86 (P2→P3 arc; P5→P6 arc — see Second
  Instance below)
**Source findings:** F-W86S-P2-006 (HIGH, pass 2) + F-W86S-P3-001 (CRIT, pass 3) +
  F-W86S-P6-009/010 (MED, pass 6 — second instance)
**Vehicle:** Local carry-forward (DF-VALIDATION-001 required before filing upstream)

### Description

When the PO authors or updates a phrase-tier policy document (DF-GREEN-DOC-TENSE-SWEEP or
analogous), tier assignments must be supported by grep-verified evidence recorded inline.
"TIER-1: zero live uses" is not a valid assertion unless accompanied by:

```
grep -r "phrase" src/ tests/ bin/   # → 0 matches (YYYY-MM-DD)
```

In wave-86, the PO authored DF-GREEN-DOC-TENSE-SWEEP v3 with three "0 live uses" TIER-1
assertions that were falsified by a 30-second grep (16 live hits). The v4 policy established
a standing rule: **un-grepped tier assignment is itself a policy violation**.

This is a recurring defect: pass-2 F-W86S-P2-006 identified the lack of grep verification,
pass-3 F-W86S-P3-001 found the v3 policy had not remedied it. The v4 standing rule is now
the policy-level codification; this PG entry tracks the process-improvement obligation.

### Second Instance — Pass 5→6 Arc (2026-07-25)

DF-GREEN-DOC-TENSE-SWEEP v5 (pass-5 remediation burst, D-521) assigned four bare-RED tokens
(`RED:`, `RED-phase`, `RED reason`, `RED because`) as TIER-1 with a blanket claim that they
had legitimate-provenance exceptions GOOD_CASE-handled — but did not include grep output to
verify the actual live-hit count and provenance distribution.

Pass-6 adversary ran full grep evidence:

```
grep -r '"RED:"' tests/ src/ bin/       # 15 hits (2026-07-25)
grep -r 'RED-phase' tests/ src/ bin/    # 2 hits — both legitimate test-harness context
grep -r 'RED reason' tests/ src/ bin/   # 0 hits
grep -r 'RED because' tests/ src/ bin/  # 0 hits
```

Result: 15/17 total hits were legitimate provenance (test headers, narrative, harness
description). The tool allowlists `RED-phase:` with a shipped `GOOD_CASE` but does NOT
implement bare `RED:` / `RED-phase` / `RED reason` / `RED because` as TIER-1 patterns.
Calling them TIER-1 was wrong — the tool never enforced them.

Policy v6 (2026-07-25): all 4 tokens re-tiered TIER-2 (context-dependent) with grep
evidence recorded inline. Pattern 30 (`Expected RED:`) retained TIER-1 (0 live hits
confirmed by grep). 2 live stale sites adjudicated: `iec104_analyzer_tests.rs:6271` +
`modbus_detection_tests.rs:2472/:2480` — PO reword prescriptions in policy v6; deferred
to next maintenance sweep (DRIFT-stale-red-scrub).

**Standing rule applied twice:** v3 bare claim → v4 codification; v5 bare claim → v6
grep-verified re-tier. The standing rule (un-grepped tier = policy violation) has now
caught the same defect class in two consecutive policy revisions.

### Proposed Fix

Add to the PO agent's policy-authoring checklist:

> For every TIER-1 token asserted as "zero live uses" or "zero false positives": execute
> `grep -r "phrase" src/ tests/ bin/` and record the command + output (including 0-match
> output) inline in the policy document. An asserted count without a recorded grep command
> is a policy violation per DF-GREEN-DOC-TENSE-SWEEP v4 standing rule.
>
> Additionally: verify that the tool ACTUALLY IMPLEMENTS the token as a pattern before
> assigning TIER-1. A token not in the tool's `_VIOLATION_PATTERNS` registry cannot be
> TIER-1 regardless of live-hit count.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Standing rule codified in
DF-GREEN-DOC-TENSE-SWEEP v4 (policies.yaml); applied again in v6 (second instance).
DF-VALIDATION-001 required before filing upstream vsdd-factory issue.

---

## PG-W86-005 — fictional-invocation class recurred within one story across revisions

**Class:** Story-writer / AC discipline — tool invocation contract verification
**Caught by:** Wave-86 adversarial pass 3 (F-W86S-P3-003 HIGH)
**Severity:** HIGH (AC specified a non-existent CLI surface; would fail immediately on delivery)
**Occurrences:** Pass-1 remediated one fictional-invocation (F-W86S-P1-001 CRIT — fictional
  CLI arg surface). Pass-3 found a second, different fictional invocation in the same story
  (F-W86S-P3-003 — `changelog-gate-check --stdin` flag does not exist).
**Source finding:** F-W86S-P3-003 (HIGH, pass 3)
**Vehicle:** Local carry-forward (DF-VALIDATION-001 required before filing upstream)

### Description

L-W84-002 (wave-84 lessons) codified the fictional-invocation class: story-writer must
verify the CLI surface of every cited tool by reading the tool's help text or source before
writing ACs. STORY-183 demonstrated that this class can recur within a single story across
multiple revisions:

- **Pass-1 F-W86S-P1-001 (CRIT):** Fictional `--glob bin/*.py` CLI argument — fixed in v1.1.
- **Pass-3 F-W86S-P3-003 (HIGH):** Fictional `changelog-gate-check --stdin` flag — not caught
  in v1.1 or v1.2 because the pass-1 fix targeted a different tool invocation.

The class is tool-specific: fixing one fictional invocation does not guarantee other cited
tools in the same story have been verified. The story-writer needs a per-tool verification
step, not a per-finding fix.

### Proposed Fix

Extend L-W84-002 to explicitly state: **"verify the invocation contract of every cited tool
in the story, not only the tool whose invocation was flagged."** After pass-1 remediation of
a fictional-invocation finding, the story-writer must sweep all other `bin/` tool references
in the same story and verify each one against the tool's actual CLI surface (help text or
source) before declaring the story remediated.

This is an extension of the existing L-W84-002 class, not a new class. The extension
ensures the verification scope is story-wide, not finding-local.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Extends L-W84-002 scope. DF-VALIDATION-001
required before determining vehicle (local story-writer checklist vs. upstream vsdd-factory
issue).

---

---

## PG-W86-006 (candidate) — adversary dispatch must glob-verify artifact paths before sending

**Class:** Dispatch discipline / artifact path verification
**Caught by:** Wave-86 adversarial pass 4 (F-W86S-P4-025 MED)
**Severity:** MEDIUM
**Occurrences:** 1 instance in wave-86 (adversary cited STORY-INDEX.md at wrong path)
**Source finding:** F-W86S-P4-025 ([process-gap, route orchestrator], pass 4)
**Vehicle:** Orchestrator/dispatch discipline (DF-VALIDATION-001 required before filing upstream)

### Description

The adversary's pass-4 dispatch cited `STORY-INDEX.md` at an incorrect path (not the canonical
`.factory/stories/STORY-INDEX.md`). The adversary operated without confirming the current
STORY-INDEX version, which could have led to stale-citation findings that reference the wrong
story version context.

This is a dispatch-discipline gap: orchestrator dispatch instructions must glob-verify artifact
paths before including them as context for the adversary. For story-index and story files,
the canonical paths are:
- `.factory/stories/STORY-INDEX.md`
- `.factory/stories/STORY-NNN.md`
- `.factory/cycles/wave-086/adversarial/pass-N-findings.md`

### Proposed Fix

Add to orchestrator dispatch checklist: before sending context paths to the adversary agent,
run `ls .factory/stories/STORY-INDEX.md` (or equivalent glob-verify) to confirm the path
exists and is the current version. If the path does not exist, do not include it in the
dispatch — instead report the missing path and resolve before dispatching.

### Disposition

Candidate for wave-086 cycle-close (S-7.02). Orchestrator acknowledged and corrected in
subsequent dispatches. DF-VALIDATION-001 research-agent validation required before filing
upstream vsdd-factory issue.

---

## PG-W86-007 (candidate) — new .factory/maintenance/ protocol docs require CLAUDE.md Project References row at creation time

**Class:** Discovery / documentation discipline
**Caught by:** Wave-86 adversarial pass 4 (F-W86S-P4-015 HIGH + F-W86S-P4-010 HIGH)
**Severity:** HIGH (gate-entry artifacts become undiscoverable to future sessions)
**Occurrences:** Identified as structural gap in wave-86 — any .factory/maintenance/ doc created
  without a CLAUDE.md row is invisible to the next session's `/factory-health` check
**Source findings:** F-W86S-P4-010 (HIGH, pass 4) + F-W86S-P4-015 (HIGH, pass 4)
**Vehicle:** Local carry-forward (STORY-182 v1.4 adds a task to address the immediate instance;
  structural fix requires DF-VALIDATION-001 before filing upstream)

### Description

When a new `.factory/maintenance/` protocol document is created (e.g., as part of a story
delivery or gate-close), there is no obligation to add a corresponding row to the
`CLAUDE.md` Project References table. This makes the document undiscoverable to future
sessions that rely on `CLAUDE.md` to enumerate active protocol documents.

The wave-86 adversary identified two HIGH findings related to this gap:
- F-W86S-P4-010: STORY-182 lacked an acceptance criterion asserting that the delivered
  CI-visible fixture coverage report step is discoverable from CLAUDE.md.
- F-W86S-P4-015: Gate-entry artifacts created in `.factory/maintenance/` have no CLAUDE.md
  row obligation, making them structurally invisible at the next gate entry.

STORY-182 v1.4 addresses the immediate instance with a task that adds the CLAUDE.md row.
The structural gap remains open for all future deliveries that create `.factory/maintenance/`
documents.

### Proposed Fix

Add to story-writer's delivery checklist for any story that creates a new
`.factory/maintenance/` document:

> For every new `.factory/maintenance/` protocol document created by this story, include a
> task to add a row to `CLAUDE.md` Project References table. The row must include: path,
> document purpose, and the initiating PG/AC that motivated its creation.

This should be a mandatory delivery step, not an optional one — without it, the document
is inaccessible to the `CLAUDE.md`-guided next-session setup.

### Disposition

Candidate for wave-086 cycle-close (S-7.02). STORY-182 v1.4 addresses the STORY-182
instance (adds Task for CLAUDE.md reference row). Structural policy fix requires
DF-VALIDATION-001 validation before filing upstream vsdd-factory issue.

---

---

## PG-W86-008 (candidate) — agents must preserve canonical input-hash under hook blocking pressure

**Class:** Agent discipline / DF-INPUT-HASH-CANONICAL-001 enforcement gap
**Caught by:** Wave-86 adversarial pass 5 (F-W86S-P5-018 MED); repaired by state-manager STEP 0
**Severity:** MEDIUM (incorrect hash stored in frontmatter; state-manager repair required before commit)
**Occurrences:** 1 instance in wave-86 (story-writer appended v1.5 with bash-hook hash values `0a1812a` / `5598136` instead of canonical Python values `9a0f34c` / `9c9b12f`)
**Source finding:** F-W86S-P5-018 (MED, pass 5); DF-INPUT-HASH-CANONICAL-001
**Vehicle:** Local carry-forward — needs agent-facing rule addition to story-writer agent instructions (DF-VALIDATION-001 required before filing upstream)

### Description

The `validate-input-hash` hook blocked story-writer's commit because the hook's bash implementation computes different hash values than the canonical Python tool (documented as PG-HASH-HOOK-DIVERGENCE in CLAUDE.md). Under this blocking pressure, story-writer computed a new hash using the bash implementation (`$(cat file)` subshell, which strips trailing newlines) and stored THOSE values — overwriting the correct canonical values that had been previously written by `bin/compute-input-hash --write`.

This is a violation of DF-INPUT-HASH-CANONICAL-001 (`input-hash:` values MUST be set using the canonical Python tool only). The hook's bash values are known to diverge from canonical Python values for every story with trailing-newline content.

### Root Cause

Story-writer agents do not have a clear protocol for what to do when the `validate-input-hash` hook blocks. The natural reflex is to "fix" the hash discrepancy by recomputing with whatever tool is available — but the only correct tool is `bin/compute-input-hash`, and the hook error must be treated as advisory per CLAUDE.md (PG-HASH-HOOK-DIVERGENCE section).

### Proposed Fix

Add an explicit agent-facing rule to story-writer instructions:

> When the `validate-input-hash` hook blocks a commit reporting a hash mismatch, the agent
> MUST NOT recompute the hash using the bash implementation. The hook result is advisory-only
> per PG-HASH-HOOK-DIVERGENCE (CLAUDE.md). The canonical Python tool is the sole authority.
> If the stored hash was set by `bin/compute-input-hash --write`, it is correct — keep it
> and push with `--no-verify` or accept the hook warning.
>
> If the stored hash is suspected stale (spec inputs changed), recompute with:
> `bin/compute-input-hash --write .factory/stories/STORY-NNN.md`
> Never use the bash hook to determine the correct value.

### Disposition

Candidate for wave-086 cycle-close (S-7.02). DF-VALIDATION-001 research-agent validation required before filing upstream vsdd-factory issue. State-manager repaired the immediate instance at burst STEP 0.

---

## PG-W86-009 (candidate) — partial-fix regression: remediation bursts must self-verify before returning

**Class:** Story-writer / remediation-burst discipline — S-7.01(c) recurrence
**Caught by:** Wave-86 adversarial pass 5 (class recurred from pass-4; F-W86S-P5-002/003/012 are partial-fix regressions)
**Severity:** HIGH (pass-4 remediation claimed to fix F-002/F-003/F-012 class but left regression artifacts; adversary identified them again in pass-5)
**Occurrences:** 2 consecutive passes (pass-4 partial fixes → pass-5 re-found them)
**Source finding:** F-W86S-P5-002/003 (HIGH, pass 5 — hermetic harness), F-W86S-P5-012 (MED, pass 5 — manifest coupling)
**Vehicle:** Local carry-forward (S-7.01(c) recurrence protocol; DF-VALIDATION-001 required before filing upstream)

### Description

S-7.01(c) requires that remediation bursts self-verify the correctness of each fix before declaring it complete. In wave-86:

- **Pass-4 F-002 (HIGH) "unimplementable hermetic harness":** Pass-4 added Task 9 with `PATH` manipulation, but the underlying mechanism (how the script is made findable in the subprocess) was incomplete. Pass-5 found the same functional gap (F-W86S-P5-002/003).

- **Pass-4 F-012 (MED) "dropped manifest coupling":** Pass-4 added `FIXTURE_MANIFEST.contains()` loop to AC-182-005 but dropped the `.len() == 4` exhaustiveness assertion. Pass-5 F-W86S-P5-012 found the coupling was still weak.

In both cases, the story-writer's remediation addressed the surface description of the finding but did not verify that the resulting specification was actually enforceable and complete.

### Root Cause

When remediating HIGH and CRIT findings, story-writer does not perform a **post-fix verification read** to confirm that:
1. The fixed AC is both necessary AND sufficient for the intended behavior
2. The implementation pathway through the fixed AC is executable without ambiguity
3. Adjacent ACs and tasks that depend on the fixed content are consistent with the fix

### Proposed Fix

Add to story-writer's remediation discipline (extension of S-7.01(c)):

> After applying a remediation to a HIGH or CRIT finding, the story-writer MUST read back the
> affected AC(s) and verify:
> (a) The AC is technically executable as written (no fictional CLI invocations, no
>     non-existent mechanisms)
> (b) The AC is sufficient to prevent the defect class, not just the specific defect instance
> (c) Any adjacent ACs or tasks that depend on the fixed content have been swept for
>     consistency

This is a "changelog claim self-verification" step: before declaring a HIGH/CRIT finding
remediated, re-read the claim and verify it against the actual body content.

### Disposition

Candidate for wave-086 cycle-close (S-7.02). Extends S-7.01(c) with a mandatory post-fix verification read for HIGH/CRIT findings. DF-VALIDATION-001 research-agent validation required.

---

## PG-W86-010 (candidate) — remediation-burst partial-fix regression persisted 4 consecutive passes; per-fix grep-evidence mandate required

**Class:** Remediation-dispatch discipline / S-7.01(c) recurrence — extends PG-W86-009
**Caught by:** Wave-86 adversarial passes 4–7 arc (partial-fix regressions in P5 and P7; orchestrator
  imposed per-fix grep-evidence mandate in D-523 burst)
**Severity:** HIGH (the same quoted-phrase mechanism class recurred in passes 4, 5, 6, AND 7 — four
  consecutive adversarial passes unable to converge a single class without explicit evidence mandate)
**Occurrences:** 4 consecutive passes (P4→P5: hermetic harness + manifest coupling; P5→P6: bare-RED
  re-tier; P6→P7: quoted-phrase Task-6 mechanism resurrected for 4th pass)
**Source finding:** F-W86S-P7-003 (HIGH, pass 7 — 4th consecutive pass recurrence of quoted-phrase class);
  extends PG-W86-009 (which caught the P4→P5 arc at 2 consecutive passes)
**Vehicle:** Local carry-forward — extends PG-W86-009; codifies mandatory per-fix verification evidence
  requirement for remediation dispatches (DF-VALIDATION-001 required before filing upstream)

### Description

PG-W86-009 (added after pass 5) codified the "partial-fix regression" class: when remediating
HIGH/CRIT findings, story-writer must perform a post-fix verification read. That codification
was applied to the P4→P5 arc.

PG-W86-010 documents that the same regression class persisted for **two more passes** (P5→P6 and
P6→P7) despite PG-W86-009's codification. The specific instance:

- **P4 F-002 (HIGH) → P5 F-W86S-P5-002/003:** Hermetic harness gap — pass-4 fix was surface-level;
  pass-5 found the same functional gap.
- **P5 F-W86S-P5-018 (MED) → P6:** Bare-RED re-tier — pass-5 made a TIER-1 claim without grep evidence;
  PG-W86-009 codified the post-fix read requirement but did not prevent the same class in v5.
- **P6 (delivered Task 6 quoted-phrase) → P7 F-W86S-P7-003 (HIGH):** Task 6 quoted-phrase mechanism
  was claimed delivered in pass-6 changelog but grep showed the body content was absent. Pass-7
  adversary found the same class for the 4th consecutive time.

The pattern reveals that "post-fix verification read" (PG-W86-009) is necessary but insufficient
when the verification is performed by the same agent that made the fix. A partial fix can survive
a self-verification read because the agent's mental model of "what was written" matches the intent,
not necessarily the actual body text.

### Root Cause

Story-writer's remediation dispatch lacks a **mechanical verification step** — a literal grep or
read-back confirmation that the specific phrase/mechanism the fix introduced is actually present
in the output. Self-verification reads catch logical errors but miss cases where the intended text
was never written (or was written in the wrong section, or was overwritten by a later edit).

The orchestrator identified this gap on pass 7 and imposed the per-fix grep-evidence mandate:

> **Mandate (D-523, effective):** Every remediation dispatch MUST require per-fix verification
> evidence in the return. For each HIGH/CRIT finding being remediated, the story-writer agent
> MUST return:
> 1. The exact text change made (diff or quoted before/after)
> 2. A grep command + output confirming the new text is present in the story body
> 3. A grep command + output confirming the old (incorrect) text is absent
>
> A remediation return that does not include this evidence for each HIGH finding is treated as
> unverified and must be re-dispatched.

### Extends PG-W86-009

PG-W86-009 said: "after applying remediation to HIGH/CRIT, read back the affected ACs and
verify (a) technically executable, (b) sufficient, (c) adjacent ACs swept."

PG-W86-010 extends that with: "the read-back must be a mechanical grep/read-back of the OUTPUT
artifact, not a self-verification of the agent's own recall. The orchestrator (or dispatching
agent) must require explicit grep evidence before accepting the remediation as complete."

This moves the verification obligation from the story-writer agent (who may have confirmation
bias about what they wrote) to the orchestrator (who receives the return and can require evidence).

### Proposed Fix

Add to orchestrator's remediation-dispatch protocol:

> For every HIGH or CRIT finding in a remediation dispatch, require the returning agent to
> include:
> - grep/read evidence that the fix is present in the output artifact
> - grep evidence that the prior incorrect text is absent
>
> Do not mark a finding REMEDIATED until this evidence is received. This is a dispatch-side
> requirement, not a story-writer-side self-check.

Add to story-writer's return format:

> For every HIGH/CRIT finding remediation, return:
> `[FINDING-ID] FIX EVIDENCE: grep -n "new_text" STORY-NNN.md → {output}`
> `[FINDING-ID] CLEAN EVIDENCE: grep -n "old_text" STORY-NNN.md → 0 matches`

### Disposition

Candidate for wave-086 cycle-close (S-7.02). Extends PG-W86-009 (post-fix verification read)
with a mandatory mechanical evidence requirement. Per-fix grep-evidence mandate introduced and
effective in D-523 burst. DF-VALIDATION-001 research-agent validation required before filing
upstream vsdd-factory issue.

---

## Summary

| ID | Severity | Status | Vehicle |
|----|----------|--------|---------|
| PG-W86-001 | HIGH | carry-forward, S-7.02 | Local (DF-VALIDATION-001 before filing) |
| PG-W86-002 | HIGH | carry-forward, S-7.02 | Local, extends PG-W84-010 (DF-VALIDATION-001 before filing) |
| PG-W86-003 | MEDIUM | adjacent, scope extension of PG-W84-012 | Ops task (devops-engineer, separate from STORY-183) |
| PG-W86-004 | CRIT | carry-forward, S-7.02 | Local (DF-VALIDATION-001 before filing upstream) |
| PG-W86-005 | HIGH | carry-forward, S-7.02 | Local, extends L-W84-002 (DF-VALIDATION-001 before filing) |
| PG-W86-006 | MEDIUM | candidate, S-7.02 | Orchestrator/dispatch discipline (DF-VALIDATION-001 before filing upstream) |
| PG-W86-007 | HIGH | candidate, S-7.02 | Local (STORY-182 v1.4 immediate instance; structural fix pending DF-VALIDATION-001) |
| PG-W86-008 | MEDIUM | candidate, S-7.02 | Agent-facing rule: canonical hash under hook pressure (DF-VALIDATION-001 before filing) |
| PG-W86-009 | HIGH | candidate, S-7.02 | S-7.01(c) extension: post-fix verification read for HIGH/CRIT (DF-VALIDATION-001 before filing) |
| PG-W86-010 | HIGH | candidate, S-7.02 | Extends PG-W86-009: orchestrator-side per-fix grep-evidence mandate (DF-VALIDATION-001 before filing) |
