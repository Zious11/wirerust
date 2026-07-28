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

## PG-W86-011 (candidate) — spec-prescribed verbatim implementation code is a regression generator

**Class:** Story-writer / AC altitude discipline — verbatim-code vs. behavioral-spec boundary
**Caught by:** Wave-86 adversarial passes 5–9 arc (4 consecutive remediation passes introduced
  new defects in STORY-182's prescribed code; root cause of the story oscillation)
**Severity:** HIGH (the oscillation pattern produced 5 HIGH regressions in pass 9 alone,
  all traceable to verbatim code prescriptions in the v1.8 body)
**Occurrences:** 4 consecutive remediation passes (P5/P6/P7/P8) each introduced new defects
  in STORY-182's prescribed implementation code; pass-9 found 5 HIGH regressions
**Source finding:** F-W86S-P9-001..005 (HIGH ×5, pass 9); root-cause analysis of wave-86
  oscillation arc (D-516..D-525, 9 passes without convergence)
**Vehicle:** Local carry-forward — STORY-182 structural fix pending strategy decision;
  DF-VALIDATION-001 required before filing upstream vsdd-factory issue

### Description

STORY-182 v1.0 was drafted with acceptance criteria that prescribed verbatim Rust code
snippets, include_str! invocations, exact task execution scripts, and specific CI YAML
stanzas. From pass 5 onward, every remediation burst that updated these verbatim prescriptions
introduced new defects: each edit to fix one finding invalidated adjacent code snippets,
created internal inconsistencies, or introduced new non-compilable constructs.

The oscillation manifested as:
- Pass 5: Hermetic harness mechanism was prescribed but non-executable
- Pass 6: include_str! coupling added but self-referential
- Pass 7: Task 6 quoted-phrase mechanism prescribed but not actually written
- Pass 8: Discriminator rewrites left 3 surviving stale loci
- Pass 9: Five HIGH regressions — two-entry COMMITTED_FIXTURES residue, non-compilable
  include_str!, vacuous self-referential predicate, surviving discriminator, false FSR claim

The pattern is diagnostic: whenever a story prescribes *how* to implement (verbatim code,
specific API calls, exact script steps), the adversary can find defects in the implementation
prescription itself, and remediating those defects introduces new ones because the
prescriptions are interdependent. This is the behavioral-altitude violation: stories should
specify *what* behavior is required, not *how* to achieve it. The mechanics belong to the
TDD phase.

### Root Cause

Story-writer's altitude discipline does not apply a sufficient filter at the boundary between
behavioral specification and implementation prescription. Stories for "tooling" epics (E-11)
are particularly susceptible because the "behavior" of a script is its invocation contract
and output guarantees — not the internal implementation mechanics. When story-writer
descends into prescribing `include_str!` vs `std::fs::read_to_string` vs a registry constant,
it is writing implementation code, not behavioral contracts.

### Proposed Fix

Add to story-writer's AC altitude discipline for E-11 (tooling) stories:

> For every AC that prescribes a specific implementation mechanism (a code snippet, a specific
> Rust API call, a specific script construct), ask: "Can this AC be restated as a behavioral
> assertion — an observable outcome — without prescribing the mechanism?" If yes, restate it.
> Only prescribe mechanisms when the mechanism IS the behavior (e.g., the story is specifically
> about introducing a new API surface).
>
> Permitted prescriptions: CLI invocation contracts, observable output format, exit codes,
> file paths written, git index changes, environment variable names.
>
> Not permitted without a specific justification: specific Rust type/method choices, internal
> variable names, include_str! vs fs::read calls, CI YAML stanza internals.

### Disposition

Candidate for wave-086 cycle-close (S-7.02). This is the root cause of the STORY-182
oscillation (9 adversarial passes without convergence). Strategy decision required:
(a) behavioral-altitude refactor [RECOMMENDED] — strip verbatim code bodies, keep decision
records as behavioral ACs, mechanics to TDD phase; (b) mechanical remediation; (c) split
story gates. DF-VALIDATION-001 research-agent validation required before filing upstream.

---

## PG-W86-012 (candidate) — src/**/*.rs glob blind spot in bin/check-green-doc-tense

**Class:** Tool implementation gap / CI scan coverage
**Caught by:** Wave-86 adversarial pass 9 (F-W86S-P9-009 MED [process-gap])
**Severity:** MEDIUM (latent — 0 TIER-1 hits in top-level src/*.rs today; 10 files unscanned
  including src/mitre.rs at 284 lines; future additions silently unscanned)
**Occurrences:** 1 instance identified; structural (all runs of the tool share the defect)
**Source finding:** F-W86S-P9-009 ([process-gap], pass 9)
**Vehicle:** Fix-vehicle decision required at resume — fold into STORY-183 or create follow-up
  story/maintenance item

### Description

`bin/check-green-doc-tense` line 477 uses the glob pattern `src/**/*.rs`. In Python's
`glob.glob()` with `recursive=True`, the `**` component requires at least one intermediate
directory component to match — it does NOT match files directly in `src/`. The pattern
`src/**/*.rs` therefore expands to `src/<subdir>/<file>.rs` but NEVER to `src/<file>.rs`.

Consequence: the following top-level source files are never scanned (10 files as of 2026-07-26):
`src/lib.rs`, `src/main.rs`, `src/mitre.rs` (284 lines), `src/config.rs`, `src/error.rs`,
`src/output.rs`, `src/pcapng.rs`, `src/protocols.rs`, `src/report.rs`, `src/stream.rs`
(list is approximate — the exact count depends on the current src/ root contents).

This is latent today because no TIER-1 tense-violation patterns are expected in those files.
However, any future additions to top-level src/*.rs files are silently unscanned, and the
tool's coverage claims are inaccurate.

### Fix

Replace `src/**/*.rs` with `src/**/*.rs` + `src/*.rs` (two separate globs), or use a
recursive pattern that explicitly covers the root:

```python
# Option A: two globs
globs = ["src/*.rs", "src/**/*.rs", ...]

# Option B: pathlib rglob
files = list(Path("src").rglob("*.rs"))
```

### Disposition

Candidate for wave-086 cycle-close. Fix-vehicle decision required at resume: fold the fix
into STORY-183 (which already modifies bin/check-green-doc-tense) or defer to a follow-up
maintenance item. The fix is trivial but requires a decision on scope. DRIFT-src-glob-blindspot
tracking row added to STATE.md. DF-VALIDATION-001 not required (purely local tooling fix —
no upstream vsdd-factory relevance).

---

## PG-W86-013 — E-11 governance-story tdd_mode:strict / automated-RED mismatch

**Class:** Story-writer template convention gap / E-11 tdd_mode discipline
**Caught by:** Wave-86 adversarial pass 10 (F-W86S-P10-010 LOW [process-gap])
**Severity:** LOW (no delivery blocker; accepted convention; codification value at cycle-close)
**Occurrences:** 2 stories in wave-86 (STORY-182 + STORY-183); also observed in STORY-176
**Source finding:** F-W86S-P10-010 ([process-gap], pass 10)
**Vehicle:** Local carry-forward — codification candidate at wave-86 cycle-close (S-7.02);
  consider template-level fix (explicit manual-RED section in E-11 story template)

### Description

E-11 governance stories carry `tdd_mode: strict` but their task orderings never produce an
automated RED observation. In E-11 template stories, the ACs assert against already-green
artifacts (fixtures, registry entries, ci.yml configurations) — the test harness cannot
produce a RED state mechanically because the subject of the test already exists.

This pattern was observed in STORY-182, STORY-183, and STORY-176. The adversary (pass 10)
flagged it as a systematic E-11 template mismatch: `tdd_mode: strict` implies a
red-green-refactor cycle, but E-11 stories structurally cannot produce automated RED.

**Orchestrator ruling (D-527):** The E-11 tdd_mode convention is accepted. Manual RED
demonstration (developer removes/corrupts the artifact, observes test failure, restores)
is the accepted substitute for automated RED. No task reorder required. An explicit E-11
template note was added to STORY-182 v2.0 and STORY-183 v2.0 documenting the convention.

### Proposed Fix

Add an explicit manual-RED section to the E-11 story template:

```
## RED Demonstration (E-11 Template Convention)
tdd_mode: strict applies. Automated RED is not achievable for governance/tooling stories
because the story's ACs assert against artifacts that exist prior to delivery.
Manual RED demonstration: remove/corrupt the target artifact, run the test suite,
observe test failure, restore the artifact. This substitutes for automated RED.
```

This note should appear in every E-11 story body to make the convention visible without
requiring re-discovery per wave.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Codification candidate: template-level fix
in E-11 story template (explicit manual-RED section). DF-VALIDATION-001 research-agent
validation required before filing upstream vsdd-factory issue.

### Pass-13 Evidence Extension (D-530, 2026-07-26)

The D-528 re-anchor sweep (which introduced the mitigation) only checked intra-document
citations in sections that were **changed** in the D-528 burst. It did not re-verify
sections that were untouched but whose line numbers were shifted by prior-section insertions.

Three stale self-anchors survived into v2.2 (F-W86S-P13-002):
- :740-745 (tdd_mode RED-procedure) — shifted further by D-529 insertions after D-528 re-anchored it
- :698 (move-aside cross-reference) — added in D-528, not in a changed section during D-529
- :612 (AC-182-003 intra-AC cite) — accumulated cumulative drift across D-527/D-528/D-529

**Structural fix (D-530):** Rather than applying a third re-anchor sweep, all intra-document
:NNN self-citations in STORY-182 and STORY-183 were eliminated and replaced with
content-based locators (task names, AC identifiers, section headings). Zero :NNN
self-citations remain in either story post-v2.3.

**Codification sharpened:** The D-528 mitigation prose said "list all :NNN citations and
verify". This is insufficient — the sweep must be CORPUS-WIDE within the document, not
scoped to changed sections. The structural fix (self-anchor elimination) is the definitive
solution. Recommend as E-11 template convention at S-7.02: intra-document :NNN
self-citations prohibited in E-11 stories; use content-based locators.

---

### Pass-11 Evidence Extension (D-528, 2026-07-26)

The v2.0 E-11 tdd_mode note (added per F-W86S-P10-010) was itself defective boilerplate:

- **STORY-183 (F-W86S-P11-007):** The generic "no automated RED reachable" claim was false
  for this story — a real automated RED exists (add BAD_CASES Tasks 7/8 before pattern tuples
  Task 6; selftest exits 1; add patterns; GREEN). The boilerplate asserted epic-level
  unreachability without checking this story's specific structure.

- **STORY-182 (F-W86S-P11-003 + F-W86S-P11-007 context):** The claim was true for STORY-182
  but only due to chosen task ordering, not a structural invariant. The v2.0 text did not
  demonstrate the claim — it simply asserted it.

v2.1 notes are now per-story demonstrated claims (STORY-182: chosen-ordering rationale;
STORY-183: concrete automated RED path prescribed).

**Codification requirement sharpened:** The E-11 template must require stories to DEMONSTRATE
RED unreachability (or specify a concrete RED path), not assert generic boilerplate. The
template fix must include a question: "Does task ordering preclude automated RED? Show why,
or prescribe the RED path."

---

## PG-W86-014 — Intra-story `:NNN` self-citation drift after mid-burst insertions

**Class:** Story artifact / intra-document line-citation drift
**Caught by:** Wave-86 adversarial pass 11 (F-W86S-P11-003 MED) — direct instance
**Severity:** MEDIUM (stale line citations produce misdirected implementer guidance; class
  is structural: any mid-burst insertion without re-anchor sweep produces this defect)
**Occurrences:** F-W86S-P11-003 is the first recorded instance; STORY-182 carries 17+
  intra-document :NNN self-citations creating 17+ potential drift sites on every burst
**Source finding:** F-W86S-P11-003 (MED, pass 11); root-cause analysis of pass-10 insertion arc
**Vehicle:** Local carry-forward — mitigation imposed D-528 (mandatory post-edit re-anchor sweep
  for story-writer dispatches); codification candidate at S-7.02 (extend DF-SIBLING-SWEEP-001
  or story-writer skill checklist)

### Description

Stories with intra-document line citations (`:NNN` references pointing to other lines in the same
document) accumulate citation drift whenever a remediation burst inserts or removes lines in the
AC bodies. The drift is silent: the story body is syntactically valid but implementer guidance
points to wrong lines.

**Specific instance:** Pass-10 remediations inserted ~45 lines into STORY-182's AC bodies. The
tdd_mode RED note cited :669-674 as the move-capture-aside procedure. After insertion, lines
:669-674 pointed to the forbidden-committed guard (a different AC section). F-W86S-P11-003
identified this as a stale citation.

**Structural exposure:** STORY-182 carries 17+ intra-document self-citations. Every remediation
burst that inserts lines anywhere above a cited line in the document silently invalidates all
citations below the insertion point. The volume of citations in STORY-182 makes every burst
a drift event unless a re-anchor sweep is explicitly performed.

### Root Cause

`bin/validate-citations` is a docs-writer preflight for external document citations — it verifies
that cited paths/lines in documentation point to real content. It does not scan story bodies for
intra-document `:NNN` self-citations.

Story-writer remediation dispatches do not include an explicit post-edit step to re-verify that
all intra-document line citations still point to the correct content after line insertions.

### Mitigation Imposed (D-528)

Story-writer dispatches now include a mandatory post-edit intra-document line-citation re-anchor
sweep:

> After any burst that inserts or removes lines from a story body containing intra-document
> `:NNN` citations, the story-writer MUST:
> 1. List all `:NNN` citations in the document (grep for the pattern `:\d\d\d`)
> 2. For each citation, verify the line number still corresponds to the cited content
> 3. Re-anchor any stale citations to the correct current line numbers
> 4. Report the sweep result (e.g., "17 citations checked, 1 re-anchored, table clean")

D-528 executed this sweep on STORY-182: 17+ citations checked, 1 re-anchored (tdd_mode
note :669-674 → :740-745), table clean.

### Codification Target

Extend DF-SIBLING-SWEEP-001 or story-writer skill checklist with this sweep as a mandatory
post-edit step for any story with 3+ intra-document citations. DF-VALIDATION-001 required
before filing upstream vsdd-factory issue.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Mitigation imposed and effective at D-528.
Codification via DF-SIBLING-SWEEP-001 extension or story-writer skill update.

---

## Inert/Self-Referential-Predicate Class — Codification Tracking

**Recurrence count: 5** (updated D-529, 2026-07-26)

| Pass | Finding | Story | Direction | Description |
|------|---------|-------|-----------|-------------|
| P3 | F-W86S-P3-005 | STORY-183 | vacuous-pass | Pattern assertion matched no real inputs — predicate could not fail |
| P5 | F-W86S-P5-001 | STORY-182 | vacuous-pass | fixture_path() test always-succeeds; no false-negative possible |
| P9 | F-W86S-P9-003 | STORY-182 | vacuous-pass | include_str!(file!()) self-referential coupling — predicate tests itself |
| P11 | F-W86S-P11-001 | STORY-182 | **false-FAIL** | concat!-needle: predicate fails for WRONG reason (prose occurrence in comments, not call-site count) |
| P12 | F-W86S-P12-001 | STORY-183 | **false-FAIL** | AC-183-007 fixture-block annotations (:572/:583/:597/:604/:611) quoted 5 literal flagged phrases → story file in-scan post-delivery → 5 false FAILs; locus class: story-prescribed fixture annotations |

**Class now covers both directions:**
- **Vacuous-pass direction:** Predicate cannot fail regardless of subject state (instances P3/P5/P9).
- **False-FAIL direction:** Predicate fails but for spurious reasons unrelated to the subject (P11/P12 instances).

**P12 new locus class:** Story-prescribed fixture-block annotations quoting literal flagged phrases.
Root cause: pass-9-added AC-183-007 block (F-P9-010) was never re-swept with the Task-4/6
no-literal-phrase rule. Any pass that ADDS prose naming a scanned pattern must run the
no-literal-phrase sweep over the added text before commit.

**Standing discipline imposed D-529:** Any story-writer dispatch that adds prose naming a TIER-1
scanned pattern MUST run the no-literal-phrase sweep over the added text before declaring the
burst complete. This is now a mandatory dispatch step, not an optional one.

**Mandatory codification question at S-7.02:** "What makes this predicate fail, and ONLY that?"
Any predicate that either (a) cannot fail regardless of subject state OR (b) can fail for reasons
other than the intended subject failure is in this class.

**Codification target:** Add to adversary checklist AND story-writer AC altitude discipline:

> For every assertion AC: "Can this predicate be satisfied/fail by something other than the
> intended behavior? List: (a) what must be true for it to fail, (b) what must be true for it
> to pass. Any path to failure/pass not involving the intended subject is a self-referential
> predicate defect."
>
> Additionally: for any AC prose that NAMES or QUOTES a phrase matching a TIER-1 pattern in
> DF-GREEN-DOC-TENSE-SWEEP, run the no-literal-phrase sweep before committing. A phrase
> quoted in a fixture annotation IS a literal occurrence in the scanned corpus.

---

## Truth-Inversion-During-Reword Class (Pass-13 New Observation, D-530)

**Class:** Story-writer / remediation discipline — truth-preservation when rewording technical claims
**First instance:** F-W86S-P13-001 (HIGH, pass 13) — pass-12 pathspec "correction" inverted a load-bearing v1.9 claim
**Severity:** HIGH (inverted claim risks implementers dropping the load-bearing src/*.rs glob)
**Standing discipline imposed D-530:** When rewording any technical semantics claim (tool
behavior, glob semantics, CLI argument behavior, etc.):
1. Re-derive the claim from first principles (e.g., run git ls-files to verify glob coverage)
2. Enumerate all loci in the document that state the claim and verify all-loci agreement
3. Do not accept a wording that is "technically true from one angle" if it obscures the
   load-bearing semantics from another angle

**Root cause:** The pass-12 correction of "src/*.rs covers top-level only" to "both globs
cover the same files" was technically accurate (dedup makes file lists equal) but conflated
the file-set result with the glob-precedence semantics. The load-bearing fact is that
src/*.rs is the BROADER pattern — an implementer reading "both cover the same files" could
conclude both are redundant and drop one.

**Codification target at S-7.02:** Add to story-writer remediation discipline:
> When correcting a semantics claim, verify the correction preserves the load-bearing
> interpretation, not just the literal truth value. Test: "Could an implementer, reading
> only the corrected text, make the wrong implementation decision?" If yes, the correction
> is insufficient.

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
| PG-W86-011 | HIGH | candidate, S-7.02 | Root cause of STORY-182 oscillation: behavioral-altitude refactor required (strategy decision pending) |
| PG-W86-012 | MEDIUM | candidate, fix-vehicle decision pending | Local tooling fix: src/**/*.rs blind spot in bin/check-green-doc-tense:477 |
| PG-W86-013 | LOW | carry-forward, S-7.02 (EXTENDED D-528) | E-11 template: per-story demonstrated RED claim required; generic boilerplate prohibited (DF-VALIDATION-001 before filing upstream) |
| PG-W86-014 | MEDIUM | carry-forward, S-7.02 (EXTENDED D-530) | Intra-story :NNN self-citation drift; structural fix D-530: self-anchors eliminated in both stories; recommend as E-11 template convention at S-7.02 |
| Truth-Inversion-During-Reword | HIGH | new class D-530 | Standing discipline imposed: re-derive + all-loci agreement check when rewording semantics claims (F-P13-001 pathspec truth inversion) |
| PG-W86-ADVERSARY-WRITE-PROFILE | LOW | dispatch template fix (D-536, 2026-07-27) | Adversary dispatch must NOT instruct adversary to write files; read-only profile; return-as-text + state-manager route |
| PG-W86-STORY-BASH-NONGATING | HIGH | carry-forward, batch with PG-W84-012 ops task | Five consecutive passes manual-remediated non-gating bash blocks; codification vehicle: bin/lint-story-bash-blocks + bin-selftest CI job (D-537, 2026-07-27) |
| PG-W86-BASELINE-TAUTOLOGY-CHECK | HIGH | carry-forward, batch with PG-W84-012 ops task | Grep-count predicates repeatedly authored to pass on baseline; codification vehicle: extend bin/lint-story-bash-blocks to execute-and-reject already-passing predicates (D-537, 2026-07-27) |
| PG-W86-AM-FSR-AC-COVERAGE | MEDIUM | carry-forward, S-7.02 | No template rule requires AM/FSR row → AC predicate mapping; two independent instances in one wave; codification vehicle: story template compliance skill (D-537, 2026-07-27) |
| PG-W86-SELF-REPORTED-SWEEP | HIGH | carry-forward, batched to PG-W84-012 dispatch | Self-reported sweeps accepted as evidence; mechanical exhaustion required; orchestrator must supply locus list for enumerable classes (D-537, 2026-07-27) |

---

## PG-W86-ADVERSARY-WRITE-PROFILE — adversary dispatch must not instruct adversary to write files

**Class:** Orchestrator dispatch discipline / agent tool-profile awareness
**Caught by:** Wave-86 adversarial pass 19 dispatch (observed during D-536)
**Severity:** LOW (no data lost; one extra hop; adversary correctly returned report as text instead)
**Occurrences:** 1 instance in wave-86 (pass-19 dispatch instructed adversary to write to a
  `.factory/` path; adversary's tool profile is Read/Grep/Glob only — Write denied)
**Source finding:** A5 attestation in pass-19-findings.md; adversary self-reported the deviation
**Vehicle:** Orchestrator dispatch template correction (local; no upstream vsdd-factory issue needed)

### Description

The pass-19 adversary dispatch instructed the adversary agent to write its findings report
directly to `/Users/zious/Documents/GITHUB/wirerust/.factory/cycles/wave-086/adversarial/pass-19-findings.md`.
The adversary agent's tool profile is `read-only` — `Read`, `Grep`, `Glob` only. `Write` and
`Edit` are denied.

The adversary correctly detected the constraint (A5 attestation: "My profile denies Write/Edit,
and my standing instructions forbid emitting report .md files") and returned the report as its
final text output instead. The orchestrator routed the file write through state-manager, which
created the file as part of the D-536 burst. No data was lost.

### Root Cause

Adversary dispatch templates include an instruction to "write findings to
`.factory/cycles/wave-086/adversarial/pass-NN-findings.md`." This instruction is incompatible
with the adversary agent's read-only tool profile. The instruction has persisted unnoticed
because the adversary consistently self-corrects (returns the report as text), but this adds
an extra orchestrator routing step that is unnecessary when the dispatch is correct.

### Proposed Fix

Update adversary dispatch templates to:

1. Remove any instruction to write files to `.factory/` paths.
2. Instruct the adversary: "Return your complete findings report as your final response text."
3. Add a note: "State-manager will persist the report to the cycle adversarial directory."

The dispatch should explicitly name state-manager as the persisting agent so the orchestrator
knows to route the returned text to state-manager before proceeding.

Example corrected dispatch ending:

> Return your complete findings report as your final response text. Do NOT attempt to write
> files — your tool profile is read-only. The orchestrator will route your returned text to
> state-manager for persistence to the cycle adversarial directory.

### Disposition

Fix in orchestrator dispatch template for future wave adversarial passes. No DF-VALIDATION-001
required (purely local orchestrator template fix; no upstream vsdd-factory issue warranted —
this is a dispatch-template wording error, not a factory engine defect). Severity LOW: no data
lost, adversary self-corrected, one extra orchestrator routing hop.

---

## PG-W86-STORY-BASH-NONGATING — story-spec bash verification blocks are non-gating (primary gap of pass 20)

**Class:** Story artifact quality / bash verification block discipline
**Caught by:** Wave-86 adversarial passes P15, P16, P17, P19, P20 (five consecutive passes; F-W86S-P20-006 is the primary pass-20 finding)
**Severity:** HIGH (five consecutive manual remediations failed to converge the class; no mechanical enforcement exists)
**Occurrences:** 5 consecutive passes: P15 (F-W86S-P15-003), P16 (F-W86S-P16-001/002), P17 (P17-001/002), P19 (v2.9 F-002 reported "2 missing" against 5 actual), P20 (F-W86S-P20-006 — 3 remaining after v2.9 reported complete)
**Source finding:** F-W86S-P20-006 (MED [process-gap], pass 20); F-W86S-P15-003; F-W86S-P16-001/002; P17-001/002
**Vehicle:** Codification via `bin/lint-story-bash-blocks` + wire into `bin-selftest` CI job (batch with PG-W84-012, D-525)

### Description

No hook, linter, or checklist step mechanically checks fenced `bash` blocks in `.factory/stories/*.md`
for a head `set -euo pipefail`. Five consecutive adversarial passes manually remediated
"verification block is non-gating" without converging the class.

The v2.9 burst (D-536) reported the class closed at 2 loci. The pass-20 adversary found
3 remaining loci (AC-182-001 Verification `:342`, AC-182-003 Verification `:480`, Task 9
Env A `:1145`). Self-reported sweeps systematically under-count this class.

The orchestrator ran a mechanical enumeration before the D-537 remediation dispatch:

**AUDIT 1 (executable spec — every ```bash fence with >1 command whose first non-blank line is not `set -euo pipefail`):**

```bash
# Run from repo root (where .factory/ is mounted)
python3 - << 'EOF'
import re, pathlib, sys

findings = []
for story_path in sorted(pathlib.Path('.factory/stories').glob('STORY-*.md')):
    text = story_path.read_text()
    # Find all ```bash fences
    fence_re = re.compile(r'```bash\n(.*?)```', re.DOTALL)
    for m in fence_re.finditer(text):
        body = m.group(1)
        # Count non-blank lines
        lines = [l for l in body.splitlines() if l.strip()]
        if len(lines) <= 1:
            continue
        # Check first non-blank line
        if not lines[0].strip().startswith('set -euo pipefail'):
            # Find approximate line number in file
            pos = text[:m.start()].count('\n') + 1
            findings.append(f'{story_path.name}:{pos}')

print(f'Found {len(findings)} bash fences lacking head set -euo pipefail:')
for f in findings:
    print(f'  {f}')
sys.exit(1 if findings else 0)
EOF
```

**Before D-537:** 13 loci (6 in STORY-182 at ~:342/:480/:533/:798/:1145/:1184; 7 in STORY-183
at ~:256/:310/:520/:744/:773/:787/:1153). Agent-reported: 2. Adversary-reported: 3. Mechanical: 13.

**After D-537:** 0 (all 13 hardened).

**Methodological result:** For this enumerable class, mechanical audit was the only exhaustive
method. Both self-sweep (2) and adversary review (3) significantly under-counted (see
PG-W86-SELF-REPORTED-SWEEP). This is the strongest available argument for codifying the
audit as tooling.

Also flag as non-gating: `|| true` patterns and `echo "Exit code: $?"` patterns in bash
verification blocks.

### Codification Vehicle

Implement `bin/lint-story-bash-blocks`:
- Assert every multi-command ```` ```bash ```` fence in `.factory/stories/*.md` begins with
  `set -euo pipefail`
- Flag `|| true` as a non-gating idiom (exception: explicitly-documented expected-failure blocks
  with subsequent predicate)
- Flag `echo "…$?"` / `echo "Exit code: $?"` as non-gating (prints but does not gate)
- Selftest-backed (own `bin/test_lint_story_bash_blocks.py`)

**Batch with PG-W84-012 (D-525):** one devops dispatch covers both PG-W84-012 (bin-selftest
required-status-check for existing bin/*.py tests) and this new linter. Do NOT dispatch two
separate devops tasks.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Codification vehicle: `bin/lint-story-bash-blocks`
wired into `bin-selftest` CI job alongside PG-W84-012. Batched to the PG-W84-012 devops dispatch /
next planning cycle per S-7.02 Cycle-Closing Checklist step 3.

---

## PG-W86-BASELINE-TAUTOLOGY-CHECK — grep-count predicates pass on baseline before implementation

**Class:** Story artifact quality / tautological-predicate discipline (extends D-535 ban)
**Caught by:** Wave-86 adversarial pass 20 (F-W86S-P20-002; also F-W86S-P20-004/005 are related axis-3/4 instances)
**Severity:** HIGH (D-535 banned the tautological-predicate class; v2.9 introduced a brand-new tautology in AC-182-006 one pass after D-535 banned one)
**Occurrences:** AC-182-006 was created in v2.9 to close F-19-004; it immediately reintroduced the banned class
**Source finding:** F-W86S-P20-002 (MED, pass 20)
**Vehicle:** Extend `bin/lint-story-bash-blocks` to execute-and-reject already-passing predicates

### Description

`grep -c 'tests/fixtures/' E2E-PCAPS.md -ge 1` was written into AC-182-006 in v2.9 as a
governance-surface completeness predicate. It already passes on baseline develop because
`tests/fixtures/` appears in `E2E-PCAPS.md` at 4+ pre-existing lines.

D-535 explicitly banned the tautological-predicate class (tautological M==len() form). The
ban was active when AC-182-006 was drafted, but the new AC was not audited against it.

The orchestrator ran a mechanical execution before the D-537 dispatch:

**AUDIT 2 (executable spec — every `test "$(grep -c …)" <op> N` predicate that already passes on baseline):**

```bash
# Run from repo root on clean develop checkout
python3 - << 'EOF'
import re, subprocess, pathlib, sys

findings = []
pred_re = re.compile(
    r'test "\$\(grep -c (\'[^\']+\'|"[^"]+") ([^\)]+)\)" (-eq|-ge|-gt|-ne|-lt|-le) (\d+)'
)

for story_path in sorted(pathlib.Path('.factory/stories').glob('STORY-*.md')):
    text = story_path.read_text()
    for m in pred_re.finditer(text):
        pattern, target_file, op, n = m.group(1), m.group(2).strip(), m.group(3), int(m.group(4))
        pattern = pattern.strip("'\"")
        # Execute grep -c against baseline
        result = subprocess.run(
            ['grep', '-c', pattern] + target_file.split(),
            capture_output=True, text=True
        )
        count = int(result.stdout.strip()) if result.returncode in (0, 1) and result.stdout.strip().isdigit() else None
        if count is None:
            continue
        # Evaluate predicate
        passes = {'-eq': count == n, '-ge': count >= n, '-gt': count > n,
                  '-ne': count != n, '-lt': count < n, '-le': count <= n}.get(op, False)
        if passes:
            lineno = text[:m.start()].count('\n') + 1
            findings.append(f'{story_path.name}:{lineno} — grep -c "{pattern}" {target_file} = {count} {op} {n} ALREADY PASSES')

print(f'Found {len(findings)} tautological predicates (pass on baseline):')
for f in findings:
    print(f'  {f}')
sys.exit(1 if findings else 0)
EOF
```

**Before D-537:** 3 live tautologies (`:492`, `:821`, `:1156`; one MORE than adversary reported).
One additional tautological form (non-gating `|| true`) at AC-182-005 Verification — covered by
AUDIT 1.

**After D-537:** 0 live tautologies (only the immutable v2.2 changelog row at `:1445` still
matches the pattern, correctly left alone as historical record).

### Codification Vehicle

Extend `bin/lint-story-bash-blocks` (see PG-W86-STORY-BASH-NONGATING) with a AUDIT-2 execution
mode: run every `test "$(grep -c …)" <op> N` predicate against the current tree and fail if the
predicate already holds. This directly codifies the D-535 standing ban as tooling.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Batch with PG-W84-012 + PG-W86-STORY-BASH-NONGATING
in the same devops dispatch — one tool covers both audits.

---

## PG-W86-AM-FSR-AC-COVERAGE — Architecture-Mapping / FSR rows with no AC predicate

**Class:** Story template gap / bidirectional AM/FSR-row ↔ AC-predicate coverage
**Caught by:** Wave-86 adversarial pass 20 (F-W86S-P20-003 [process-gap])
**Severity:** MEDIUM (two independent instances in one wave; STORY-183's ci.yml row survived 19 passes without AC coverage)
**Occurrences:** STORY-182 ci.yml deliverable (existed since v1.4 = 16 passes); STORY-183 ci.yml row (existed since v1.1 = 19 passes)
**Source finding:** F-W86S-P20-003 (MED [process-gap], pass 20)
**Vehicle:** Add bidirectional AM/FSR-row ↔ AC-predicate coverage check to story template compliance skill

### Description

Both stories declared ci.yml deliverables in their Architecture Mapping and FSR rows, but
neither story had an AC predicate checking for those ci.yml changes. Specifically:

- **STORY-182** `:855` (AM), `:1328` (FSR) — ci.yml additive step; no AC checked its presence
  until F-W86S-P20-003 prescribed AC-182-006 additions.
- **STORY-183** `:811` (AM), `:1236` (FSR) — ci.yml prose edits at `:434`/`:442`/`:462`; no AC
  covered these until F-W86S-P20-003 prescribed new predicates.

The STORY-183 ci.yml row survived 19 adversarial passes without any AC coverage. The STORY-182
row survived 16 passes.

### Root Cause

No template rule or agent-prompt explicitly requires that every Architecture-Mapping / FSR
row map to at least one checkable AC predicate. Story reviewers (adversary and orchestrator)
check AC→behavior coverage, but not the reverse (declared deliverable→AC coverage).

### Proposed Fix

Add to story-writer's AM/FSR template discipline:

> For every row in the Architecture Mapping and FSR tables, there MUST exist at least one
> AC predicate that asserts the presence or correct content of that deliverable. A deliverable
> row with no corresponding AC predicate is an incomplete story specification.
>
> Bidirectional coverage check:
> (a) For each AC: confirm it maps to at least one AM/FSR row (existing check)
> (b) For each AM/FSR row: confirm at least one AC verifies its presence (new requirement)

Add this bidirectional check to the `vsdd-factory:validate-consistency` skill or the story
template compliance skill's "AM/FSR coverage" section.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Codification via story template compliance skill
(story-writer agent instructions update). DF-VALIDATION-001 not required (local template
discipline, no upstream vsdd-factory filing needed).

---

## PG-W86-SELF-REPORTED-SWEEP — self-reported sweeps accepted as evidence of class closure

**Class:** Orchestrator dispatch discipline / verification completeness
**Caught by:** Wave-86 adversarial pass 20 (F-W86S-P20-006; also v2.9 false closure of the bash-nongating class)
**Severity:** HIGH (DF-SIBLING-SWEEP-001 requires the remediating agent to execute a sweep, but nothing verifies the sweep was exhaustive; evidence: v2.9 reported "2 of 5" and declared complete)
**Occurrences:** Manifested definitively in v2.9 burst (D-536): agent reported 2 loci swept; mechanical AUDIT 1 found 13; adversary found 3. Both under-counts accepted as complete in prior passes.
**Source finding:** F-W86S-P20-006 (MED [process-gap], pass 20); PG-W86-010 (orchestrator grep-evidence mandate, established D-523) extends this class
**Vehicle:** Orchestrator dispatch protocol: supply exhaustive locus list for enumerable classes; post-burst mechanical re-enumeration required

### Description

DF-SIBLING-SWEEP-001 requires the remediating agent to execute a sweep and report hits.
But nothing verifies the sweep was exhaustive — the agent self-selects the sibling set and
reports completion.

Concrete evidence from wave-86 pass 20:
- v2.9 reported the `set -euo` class closed at **2 loci**. Mechanical AUDIT 1 found **13**.
- Adversary (pass 20) reported **3** loci. Still a 10× under-count against mechanical.
- Only mechanical enumeration was exhaustive.

This is a generalization of PG-W86-010 (per-fix grep-evidence mandate, D-523): PG-W86-010
requires the story-writer to RETURN grep evidence. PG-W86-SELF-REPORTED-SWEEP requires the
ORCHESTRATOR to pre-supply exhaustive loci and re-enumerate post-burst.

### Orchestrator Discipline

For any defect class that can be mechanically enumerated:

1. **Pre-dispatch:** orchestrator runs the enumeration script (e.g., AUDIT 1) and includes the
   exhaustive locus list in the remediation dispatch. Do NOT rely on the agent's own sweep.

2. **Post-burst:** orchestrator re-runs the enumeration script after the burst. Only a zero
   result authorizes the burst commit.

3. Self-reported sweep counts are recorded but NOT accepted as evidence of class closure
   for enumerable classes.

This was first applied in the D-537 dispatch: the orchestrator supplied all 13 loci to the
story-writer, rather than asking the agent to "sweep and fix." The post-burst audit confirmed
0 remaining.

### Relationship to Existing Policies

- **PG-W86-010 (D-523):** Story-writer must RETURN grep evidence for each HIGH/CRIT fix.
  PG-W86-SELF-REPORTED-SWEEP is the orchestrator-side complement: orchestrator PRE-SUPPLIES
  the enumeration and RE-CHECKS post-burst.
- **DF-SIBLING-SWEEP-001:** Requires a sweep. This PG clarifies that for enumerable classes,
  the sweep must be mechanical, not manual, and must be verified by the orchestrator.

### Disposition

Carry-forward to wave-086 cycle-close (S-7.02). Codification via orchestrator dispatch protocol
(update to orchestrator skill checklist for HIGH/enumerable finding classes). Batch with
PG-W84-012 devops dispatch for the tooling component (`bin/lint-story-bash-blocks`). The
orchestrator-protocol component (pre-supply loci, post-verify) is an agent-behavior change,
not a code change — codify in the orchestrator skill dispatch checklist.

---

## PG-W86-AUDIT1-TOO-NARROW — AUDIT 1's set-e-safety definition induced F-W86S-P21-002

**Class:** Mechanical audit definition too narrow — induced defect
**Caught by:** Wave-86 adversarial pass 21 (F-W86S-P21-002 MEDIUM; process-gap tagged by adversary)
**Severity:** MEDIUM
**Source finding:** F-W86S-P21-002 (MEDIUM, pass 21)
**Vehicle:** Fold AUDIT 3 into `bin/lint-story-bash-blocks` spec (see PG-W86-STORY-BASH-NONGATING); batch to PG-W84-012 devops dispatch / next planning cycle

### Description

The orchestrator's AUDIT 1 checked that `set -euo pipefail` was PRESENT at the head of each
bash fence — and nothing more. Combined with the pass-20 remediation instruction to fix
"tautological SKIP-count predicates" by adding a guard + variable-assignment form, this
**induced** F-W86S-P21-002: the assignment form `SKIP_COUNT="$(grep -c …)"` aborts under
`set -e` in the expected-pass case (grep exits 1 on zero count), producing a false-RED on
the success condition.

The adversary caught the regression precisely because the dispatch explicitly invited
criticism of the audits' definitions. That invitation is what surfaced the gap.

**AUDIT 3 (created D-538)** — executable specification, preserved verbatim:

```bash
# AUDIT 3: grep -c in ASSIGNMENT POSITION under set -e (can exit non-zero on zero count)
# Flag any line matching: VAR="$(grep -c ...)" where the assignment is NOT protected by
# || true or equivalent.
# Pattern to grep for in bash fence blocks:
grep -nE '^[[:space:]]*[A-Z_]+="[[:dollar:]][(]grep -c' "$STORY_FILE"
# Any hit that is NOT followed by "|| true" on the same line is a candidate defect.
# Safe forms: test "$(grep -c …)" (argument position — set -e cannot fire);
#             VAR="$(grep -c … || true)"  (explicit exit-0);
#             ! grep -q … (inverted, no count needed).
```

**AUDIT 3 — assignment-position command substitution under set -e (orchestrator, D-538):**

```python
import re
for f in ['.factory/stories/STORY-182.md','.factory/stories/STORY-183.md']:
    lines=open(f).read().split('\n'); inblk=False; hasset=False; start=0
    for i,l in enumerate(lines,1):
        s=l.strip()
        if s.startswith('```bash'): inblk=True; hasset=False; start=i; continue
        if inblk and s.startswith('```'): inblk=False; continue
        if inblk:
            if s.startswith('set -euo pipefail'): hasset=True
            m=re.match(r'^([A-Za-z_][A-Za-z0-9_]*)="?\$\((.+)\)"?\s*$',s)
            if m and hasset:
                var,cmd=m.groups()
                if re.match(r'\s*(grep|git cat-file|test|diff|cmp)\b',cmd) and '|| true' not in cmd and '|| echo' not in cmd:
                    print(f'  RISK {f}:{i} (fence@:{start}) {var}')
```

**Empirical verification (D-538):** Before fix — 2 loci (STORY-182 `:496`, `:1184`). After fix — 0.
Three-way discriminating test: artifact ABSENT → exit 1 (guard fires); artifact present
with 0 SKIPs → exit 0; artifact present WITH a SKIP → exit 1. Both failure modes closed simultaneously.

### Codification

Fold AUDIT 3 into the `bin/lint-story-bash-blocks` spec already recorded under
PG-W86-STORY-BASH-NONGATING. The check must flag:
- Any `VAR="$(grep -c …)"` form not protected by `|| true`
- Combined with the AUDIT 1 check (fence head has `set -euo pipefail`)

Future adversary dispatches that supply AUDIT results MUST explicitly invite the adversary
to challenge the audits' definitions — this is what surfaced F-W86S-P21-002.

Carry-forward: batch to PG-W84-012 devops dispatch / next planning cycle.
Checklist step 3 (S-7.02): satisfied by this explicit deferral, not silent drop.

---

## PG-W86-AUDIT2-GUARD-BLINDNESS — AUDIT 2 cannot see same-fence existence guards

**Class:** Mechanical audit false-positive — context-blindness
**Caught by:** Wave-86 adversarial pass 21 (orchestrator AUDIT 2 results for pass 21 — 3 hits, all benign)
**Severity:** LOW
**Source finding:** Orchestrator AUDIT 2 post-burst analysis, pass 21
**Vehicle:** Future `bin/lint-story-bash-blocks` implementation; batch to PG-W84-012 devops dispatch

### Description

AUDIT 2 evaluates each `test "$(grep -c …)"` predicate in isolation and cannot see a
preceding `test -s <same-file>` existence guard in the same fence. This causes false
positives on correctly-guarded predicates.

Pass-21 evidence: AUDIT 2 found 3 matches, all benign:
- `:500` and `:1196` — each preceded by a `test -s coverage-out.txt` existence guard inside
  a `set -euo pipefail` fence, so the block aborts before the count check when the artifact
  is absent. The non-vacuity requirement is satisfied by the guard.
- `:1458` — immutable v2.2 changelog row; not executable code.

The two STORY-182 fixes pass the three-way discriminating test: absent → FAIL (guard fires);
present-and-clean → PASS; present-WITH-violation → FAIL. All correctly guarded.

### Codification

The future checker (`bin/lint-story-bash-blocks`) must:
1. Treat a same-fence `test -s <file>` existence guard on the same target as satisfying
   the non-vacuity requirement for subsequent `grep -c` predicates on that file.
2. Skip changelog-table rows entirely (detect by context: inside a `| vX.Y changelog` table
   cell, or following a `## Changelog` heading within N lines).

This closes the false-positive class that made AUDIT 2 emit spurious warnings on three
correctly-written loci in pass 21.

Carry-forward: batch to PG-W84-012 devops dispatch / next planning cycle.
Checklist step 3 (S-7.02): satisfied by this explicit deferral, not silent drop.

---

## PG-W86-CONTRADICTION-ACCUMULATION-REGIONS — two regions carried findings in three consecutive passes

**Class:** Region-level internal contradiction accumulation despite whole-region rewrites
**Caught by:** Wave-86 adversarial passes 19, 20, 21 (three consecutive passes, same two regions)
**Severity:** MEDIUM
**Source findings:** STORY-182 Task 8 + Task 10a (P19 F-003/F-007, P20 F-007, P21 F-004); AC-182-006 (P19 F-002, P20 F-002/F-008, P21 F-003/F-005)
**Vehicle:** Codification candidate: region-level "claims-vs-command" consistency check; batch to PG-W84-012 devops dispatch / next planning cycle

### Description

Two STORY-182 regions have each carried findings in three consecutive adversarial passes
(P19, P20, P21) despite two whole-region rewrites. The failure mode is that a rewrite
replaces the prose but leaves a superseded clause elsewhere in the region, or introduces a
claim about a command without verifying the command's actual content.

**Region 1: STORY-182 Task 8 + Task 10a**
- P19: `F-003` (Task 8/10a stated different mechanisms for the blocking gate)
- P20: `F-007` (Task 8 and 10a rewrote together but still diverged on blocking/evidence-only)
- P21: `F-004` (Task 8 cited a `grep -qE "test result: ok"` absent from the command it prescribed)

In each case, the whole-region rewrite fixed the explicitly-cited locus but left a related
clause that contradicted the fix.

**Region 2: STORY-182 AC-182-006**
- P19: `F-002` (AC-182-006 added; whole-file predicate was tautological at baseline)
- P20: `F-002/F-008` (AC-182-006 rewritten; new predicate discriminating but environment-scoped incorrectly)
- P21: `F-003/F-005` (AC-182-006 predicate section-blind re ENIP :279; preamble left in wrong form)

### Root Cause

When a region contains both (a) prose describing what a command does AND (b) the command
itself, a rewrite of the prose can leave the command unchanged (or vice versa), producing
a claims-vs-command mismatch. The whole-region rewrite discipline (D-536) prevents
single-locus edits but does not require re-reading the command to verify the claim.

### Proposed Codification

**Region-level "claims-vs-command" consistency check:** Every prose claim that a specific
check exists (e.g., "fails the `grep -qE "test result: ok"` check in this same command")
must name a command block in the same region that demonstrably contains that check. Before
committing any whole-region rewrite, explicitly verify:

1. Every sentence of the form "X does Y" names an artifact (command/file/test) in the same
   region.
2. That artifact is read verbatim and confirmed to contain the named element Y.
3. If Y is absent from the artifact, either add Y to the artifact or remove the claim.

This check is the mechanical complement to the whole-region rewrite discipline (D-536): the
discipline ensures the whole region is rewritten; this check ensures the rewrite is
internally self-consistent.

Per the S-7.02 Cycle-Closing Checklist, codification vehicle named (future
`bin/lint-story-bash-blocks` or a region-consistency probe); batched to PG-W84-012 devops
dispatch / next planning cycle. Checklist step 3 (S-7.02): satisfied by this explicit
deferral, not silent drop.

---

## PG-W86-PREDICATE-LINE-RANGE — AC predicates must be content-anchored when the target file is a story deliverable

**Class:** AC predicate design / line-range drift
**Caught by:** Wave-86 adversarial pass 22 (F-W86S-P22-001 MEDIUM) + AUDIT 4 (new this pass)
**Severity:** MEDIUM
**Source finding:** F-W86S-P22-001 — AC-182-006 `sed -n '337,345p'` predicate on E2E-PCAPS.md;
  story's own Task 7 edits above `:337` would push the target sentence out of the window after
  implementation, making the predicate pass vacuously on the post-implementation tree.

### Description

No standing rule forbade hardcoded absolute line ranges in AC predicates against files the same
story also edits. A predicate anchored to `sed -n 'N,Mp'` can discriminate on the baseline but go
inert after the story's own edits grow the file above line N, pushing the target content past M.

This is structurally different from the tautological-baseline class (PG-W86-BASELINE-TAUTOLOGY-CHECK):
that class covers predicates that pass on the pre-implementation baseline; this class covers predicates
that pass on the **post-implementation** tree for the wrong reason.

**Discipline to adopt (D-539):**
AC predicates MUST be content-anchored, never absolute-line-anchored, when the target file is a
deliverable of the same story. Acceptable anchors include:
- Section-heading patterns: `awk '/^## Section Name/{f=1;next} /^## /{f=0} f'`
- Content patterns: `grep -cF 'literal-string'`, `grep -cE 'pattern'`
- File-existence + content checks: `test -s file && grep -qF 'token' file`

The banned form is `sed -n 'N,Mp'`, `awk 'NR>=N'`, `head -n N | tail`, or any expression that
embeds a literal line number as the extraction bound.

**AUDIT 4 (executable spec):**
```bash
# AUDIT 4: Find hardcoded absolute line-range extractors in AC predicates
# Covers: sed -n 'N,Mp', awk 'NR>=N' / 'NR==N', head -n N | tail
grep -nE "sed -n '[0-9]+,[0-9]+p'|awk 'NR>?=[0-9]+|head -n [0-9]+ \| tail" \
  .factory/stories/STORY-182.md \
  .factory/stories/STORY-183.md
```
Expected post-D-539: 0 results (all predicate line-range extractors removed or converted to
content-anchored form).

**Vehicle:** DF-SIBLING-SWEEP-001 checklist extension; batch to PG-W84-012 devops dispatch /
next planning cycle. S-7.02 step 3: satisfied by explicit deferral.

---

## PG-W86-DELIVERABLE-TASK-COVERAGE — Every Architecture-Mapping / FSR deliverable row must have an actionable Task

**Class:** Story-writer checklist gap / deliverable↔task coverage
**Caught by:** Wave-86 adversarial pass 22 (F-W86S-P22-003 MEDIUM)
**Severity:** MEDIUM
**Source finding:** F-W86S-P22-003 — ci.yml additive step declared in five places (AC-182-004(e),
  AC-182-006, Architecture Mapping, FSR, ACR) but no Task instructed creating it; survived 22 passes
  because DF-SIBLING-SWEEP-001 covers AC/EC↔Tasks↔ACR↔prose but not "Architecture-Mapping / FSR row
  has an actionable Task".

### Description

DF-SIBLING-SWEEP-001's story-writer checklist covers AC/EC↔Tasks↔ACR↔prose sweeps but not the
invariant that every Architecture-Mapping / FSR deliverable row has an actionable Task. The gap
means a deliverable can be prescribed in all five governance surfaces without the implementer being
told to create it, because the actionable instruction lives only in the AC predicate (what to verify)
not in a Task bullet (what to do).

**Discipline to adopt (D-539):**
Every Architecture-Mapping row and FSR deliverable row MUST have a corresponding actionable Task
bullet that names the artifact and prescribes its creation verbatim (including critical parameters
such as step name, placement, and run: block for CI steps). The absence of such a Task is a story
defect regardless of whether the AC predicate covers the artifact.

**Vehicle:** DF-SIBLING-SWEEP-001 checklist — add row: "Every Architecture-Mapping / FSR deliverable
row has an actionable Task bullet." Batch to PG-W84-012 devops dispatch / next planning cycle.
S-7.02 step 3: satisfied by explicit deferral.

---

## PG-W86-SWEEP-CLAIM-VERIFICATION — Changelog rows claiming exhaustiveness must cite a confirming residual grep

**Class:** Evidence discipline / sweep-claim verification
**Caught by:** Wave-86 adversarial pass 22 (F-W86S-P22-002 MEDIUM)
**Severity:** MEDIUM
**Source finding:** F-W86S-P22-002 — v2.11 changelog row claimed "all four loci" for the
  `red-out.txt` sweep; three live loci remained singular and no confirming residual grep was
  documented.

### Description

A changelog row asserting "all N loci" or "all four loci" is unverifiable without a confirming
residual grep that shows the sweep returned the expected count. The v2.11 row claimed four while
three remained, and the verifying AC had no predicate at all — the exhaustiveness claim existed only
in prose, not as a machine-checkable assertion.

This class is related to but distinct from PG-W86-BASELINE-TAUTOLOGY-CHECK (that class covers
predicates that are true at baseline; this class covers prose claims that are stated without any
predicate at all).

**Discipline to adopt (D-539):**
Any remediation changelog row claiming exhaustiveness ("all N loci", "all instances", "all four")
MUST record the confirming residual grep and its expected count inline. Format:
```
Confirming residual: `grep -rF 'old-token' <scope>` → expected: 0 (or N historical-only)
```
If any file was found with the old token but intentionally NOT updated (e.g., a historical
changelog entry), it MUST be named explicitly with justification.

**Vehicle:** Story-writer remediation checklist; batch to PG-W84-012 devops dispatch / next
planning cycle. S-7.02 step 3: satisfied by explicit deferral.

---

## PG-W86-AUDIT-SEAM-PIPEFAIL — Pipeline status masking in single-command pipeline fences

**Class:** AUDIT seam / pipefail discipline
**Caught by:** Wave-86 adversarial pass 22 (AUDIT 5 introduced this pass; found 2 loci)
**Severity:** LOW
**Source:** AUDIT 5 addresses a seam between AUDIT 1 (>1 command fences only) and AUDIT 3
  ($()-under-set-e only): single-command fences containing a pipeline lack `set -euo pipefail`,
  which masks failures in non-final pipeline stages.

### Description

AUDIT 1 only inspects fences with >1 command (where `set -euo pipefail` is clearly multi-step
protection). AUDIT 3 only inspects `$()` assignment under `set -e` (failure at substitution). A
single-command fence containing a pipeline like `cmd1 | cmd2` has no multi-command guard and no
assignment form — it falls in a seam. Without `pipefail`, a failure in `cmd1` is silently masked
by `cmd2`'s exit code.

Both 2 loci found by AUDIT 5 were verified fail-closed by other means (D-539 AUDIT 5 hardening),
so this gap caused no live defect. However, the discipline must be adopted to prevent future misses.

**Discipline to adopt (D-539):**
All bash fences in story files that contain a pipeline operator (`|`) MUST open with
`set -euo pipefail`. This extends AUDIT 1's protection to single-command pipeline fences. AUDIT 5's
script is the executable spec (see PG-W86-PREDICATE-LINE-RANGE entry for the script — AUDIT 5 uses
the same Python-based fence scanner).

**AUDIT 5 (executable spec):**
```bash
# AUDIT 5: Find bash fences in story files with a pipeline but no set -euo pipefail at head
python3 - <<'EOF'
import re, sys
for fname in sys.argv[1:]:
    content = open(fname).read()
    for m in re.finditer(r'```bash\n(.*?)```', content, re.DOTALL):
        block = m.group(1)
        has_pipeline = '|' in block
        has_pipefail = 'set -euo pipefail' in block or 'set -e' in block
        if has_pipeline and not has_pipefail:
            start = content[:m.start()].count('\n') + 1
            print(f"{fname}:{start}: pipeline fence missing set -euo pipefail")
EOF .factory/stories/STORY-182.md .factory/stories/STORY-183.md
```
Expected post-D-539: 0 results.

**Vehicle:** AUDIT 5 added to standing per-burst audit suite; batch to PG-W84-012 devops dispatch /
next planning cycle. S-7.02 step 3: satisfied by explicit deferral.
