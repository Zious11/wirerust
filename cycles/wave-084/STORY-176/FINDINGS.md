---
document_type: cross-pass-findings-tracker
story: STORY-176
wave: 84
cycle: wave-084
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-07-20T22:15:00Z
pass_count: 3
converged: false
---

# STORY-176 Step-4.5 Cross-Pass Findings Tracker

Cross-pass adversarial findings tracker for STORY-176 "Feature-IEC104 Cycle-Close: Local Gate + Tooling Hygiene Sweeps". Updated after each adversarial pass until convergence (BC-5.39.001 clean-streak of 3).

---

## Pass 1 — 2026-07-20 | Classification: FINDINGS | Code Tip Reviewed: 056afabe

**Adversary:** Fresh (no prior-pass context).
**Code tip reviewed:** `056afabe`
**Fixes committed through:** `08fc7d88` (code); story spec v2.4 (AC-ID reuse footnote; input-hash 8dc205a canonical)

### Findings Summary

| ID | Severity | Category | Disposition | Fix |
|----|----------|----------|-------------|-----|
| F-S176P1-001 | MEDIUM | pattern logic | FIXED | `61f6db4c` + fixture `08fc7d88` |
| F-S176P1-002 | MEDIUM | pattern logic | FIXED | `61f6db4c` |
| F-S176P1-003 | MEDIUM | test prose | FIXED | `08fc7d88` |
| F-S176P1-004 | LOW | pattern scope | ACCEPTED | AC-permitted; gate ratification |
| F-S176P1-005 | LOW | documentation | FIXED | `61f6db4c` |
| F-S176P1-006 | LOW | test quality | FIXED | `08fc7d88` |
| F-S176P1-007 | LOW | process-gap | LEDGERED | PG-W84-010 filed |
| F-S176P1-008 | LOW | spec | FIXED | story v2.4 |

**3 MEDIUM / 5 LOW. HIGH/CRITICAL: 0.**

---

### Finding Details

#### F-S176P1-001 — MEDIUM — pattern-29 missed bare "fails until wired"

Pattern-29 (`fails\s+until\s+wired`) did not catch the bare phrase "fails until wired"
in fixture prose. The initial negative lookahead was too narrow, allowing the phrase
through without a match.

**Disposition:** FIXED
**Fix:** Commit `61f6db4c` (pattern-29 re-narrowed with corrected negative lookahead)
+ fixture added at `08fc7d88` to cover the bare-phrase case.

---

#### F-S176P1-002 — MEDIUM — pattern-26 missing trailing \b

Pattern-26 lacked a trailing word-boundary anchor (`\b`), causing false matches on
past-tense forms such as "compiled" (the pattern matched "compiles" embedded in
"compiled").

**Disposition:** FIXED
**Fix:** Commit `61f6db4c` (trailing `\b` appended to pattern-26 regex).

---

#### F-S176P1-003 — MEDIUM — stale RED-phase prose in bin test files (3 loci)

Three loci in `bin/` test files — including a STORY-174 sibling — contained stale
RED-phase prose describing tests as "expected to fail" or "currently failing". This
language would itself trigger the gate check in a production run.

DF-SIBLING-SWEEP executed and verified: all bin/*.py test files scanned; only the
three identified loci contained stale prose. Fixture payloads deliberately retained
(they test the gate's detection logic; removing them would reduce coverage).

**Disposition:** FIXED
**Fix:** Commit `08fc7d88` (past-tense reframe at all three loci; fixture content preserved).

---

#### F-S176P1-004 — LOW — pattern-(b) verb-set narrowing (accepted)

Pattern (b)'s verb set is narrower than the full "are/is" case universe. However,
pattern (c) independently subsumes the are/is cases. The narrowing is therefore not
a coverage gap.

**Disposition:** ACCEPTED
**Rationale:** Pattern (c) provides full coverage for are/is cases; AC-permitted
narrowing. Non-blocking. Carried for gate ratification.

---

#### F-S176P1-005 — LOW — docstring TOKEN LIST 23-25 gap

The `bin/check-green-doc-tense` docstring TOKEN LIST was incomplete — tokens 23-25
were absent from the enumeration (the list jumped from 22 to 26).

**Disposition:** FIXED
**Fix:** Commit `61f6db4c` (docstring completed; tokens 1..29 fully enumerated).

---

#### F-S176P1-006 — LOW — self-test label-blind assertions for patterns 26-29

Self-test assertions for patterns 26-29 verified only whether the match fired, not
which specific pattern label fired. Label-blind assertions reduce the diagnostic value
of test failures.

**Disposition:** FIXED
**Fix:** Commit `08fc7d88` (expected-label assertions added for all patterns 26-29
fixtures; failures now report which pattern misfired).

---

#### F-S176P1-007 — LOW — [process-gap] gate scan set Rust-only

`bin/check-green-doc-tense` scans only Rust source files (`*.rs`). This means the
gate cannot police its own Python test harness (`bin/test_check_green_doc_tense.py`)
for stale RED-phase prose — which is exactly where F-S176P1-003 stale prose resided.

Out of story scope (the story's ACs are for Rust source gating only; extending
coverage to bin/*.py would require a separate story).

**Disposition:** LEDGERED
**Vehicle:** PG-W84-010 in `cycles/wave-084/process-gap-ledger.md`. DF-VALIDATION-001
research-agent validation required before filing as a GitHub issue.

---

#### F-S176P1-008 — LOW — Disposition-table AC-ID reuse

The story's Disposition table contained an AC-ID reuse: the same AC-ID was used for
two distinct disposition entries, creating potential ambiguity at gate ratification.

**Disposition:** FIXED
**Fix:** Story v2.4 (disambiguating footnote added to the reused AC-ID entry;
input-hash re-baselined to `8dc205a` via canonical Python tool `bin/compute-input-hash`).

---

## Pass 2 — 2026-07-20 | Classification: FINDINGS | Code Tip Reviewed: 08fc7d88

**Adversary:** Fresh (no prior-pass context).
**Code tip reviewed:** `08fc7d88`
**Fixes committed through:** `b583c4b4`

### Part A — Pass-1 Fix Verification

| Finding | Status |
|---------|--------|
| F-S176P1-001 | VERIFIED-FIXED |
| F-S176P1-002 | VERIFIED-FIXED |
| F-S176P1-003 | VERIFIED-FIXED |
| F-S176P1-004 | ACCEPTED — not re-litigated |
| F-S176P1-005 | VERIFIED-FIXED |
| F-S176P1-006 | VERIFIED-FIXED |
| F-S176P1-007 | LEDGERED — not re-raised |
| F-S176P1-008 | VERIFIED-FIXED |

All pass-1 fixable findings VERIFIED-FIXED. F-S176P1-004 ACCEPTED (AC-permitted narrowing, not re-litigated). F-S176P1-007 LEDGERED (PG-W84-010, out of story scope, not re-raised).

### Part B — New Findings

| ID | Severity | Category | Disposition | Fix |
|----|----------|----------|-------------|-----|
| F-S176P2-001 | MEDIUM | documentation | FIXED | `b583c4b4` |
| F-S176P2-002 | LOW | test coverage | FIXED | `b583c4b4` |
| F-S176P2-003 | LOW | pattern logic | ACCEPTED | adversary verdict: informational |
| Obs-1 | LOW | pattern scope | ACCEPTED | consistent with F-S176P1-004 |
| Obs-2 | LOW | test quality | ACCEPTED | harmless duplicate |

**1 MEDIUM / 2 LOW actionable (both FIXED). 2 ACCEPTED observations. HIGH/CRITICAL: 0.**

---

### Finding Details

#### F-S176P2-001 — MEDIUM — stale CHANGELOG self-test count

CHANGELOG self-test count read 89 but the actual self-test suite reached 91 after
pass-1's fixture additions (S-7.01 partial-fix propagation: count was not synced
when pass-1 added the new fixture).

**Disposition:** FIXED
**Fix:** Commit `b583c4b4` (count synced to final 91; sibling sweep of all count
claims across CHANGELOG and test prose confirmed clean).

---

#### F-S176P2-002 — LOW — no regression-guard fixture for pattern-26 trailing-\b

Pattern-26's trailing word-boundary anchor (`\b`) fix from pass-1 had no dedicated
GOOD regression-guard fixture. Without a fixture that demonstrates the
boundary-anchored match, a future regression could re-introduce the "compiled"
false-positive silently.

**Disposition:** FIXED
**Fix:** Commit `b583c4b4` (`// harness skeleton compiled cleanly in STORY-NNN` GOOD
fixture added; regression guard in place).

---

#### F-S176P2-003 — LOW — pattern-29 lookahead over/under-shoot on inflected objects

Pattern-29's lookahead is potentially both over-broad ("wired its/their" is a
FP-latent form) and under-narrow ("wired the same way" is a FN-latent form) for
inflected object phrases.

**Adversary's own verdict:** Informational. Zero current-tree matches hit either
edge case. AC zero-FP holds. No fix required for convergence.

**Disposition:** ACCEPTED
**Rationale:** Adversary classified this informational; zero current-tree impact
confirmed; AC zero-FP constraint satisfied. Carried as accepted risk.

---

#### Obs-1 — LOW — "exposes a compile-only seam" evasion

Observation that the phrase "exposes a compile-only seam" could potentially evade
the gate if used in stale RED-phase prose.

**Disposition:** ACCEPTED
**Rationale:** Consistent with F-S176P1-004 accepted verb-narrowing disposition.
Pattern (c) provides coverage for the are/is cases; this specific phrasing is
within the AC-permitted scope of the narrowing decision ratified at pass-1.

---

#### Obs-2 — LOW — duplicate GOOD-fixture payload across pattern-c/d allowlists

The GOOD-fixture payload used to document the pattern-c and pattern-d allowlist
entries is identical in both locations.

**Disposition:** ACCEPTED
**Rationale:** Harmless duplication. Each entry documents a distinct rationale
even though the fixture string is the same. No test coverage gap.

---

**Orchestrator-verified post-fix (b583c4b4):** self-test 91/0 exit 0; gate PASS
114 files; gitignore test 2/0; CHANGELOG count matches actual.

---

## Pass 3 — 2026-07-20 | Classification: FINDINGS | Code Tip Reviewed: b583c4b4

**Adversary:** Fresh (no prior-pass context). NOTE: First pass-3 dispatch died mid-stream
on API "Response stalled" error after attestation only — no findings produced. A fresh
pass-3 adversary was re-dispatched and completed. This record documents the re-dispatched
(successful) adversary.

**Ops note (infrastructure transient):** First pass-3 adversary stalled mid-stream on API
"Response stalled" error after attestation only. No findings were produced before the stall.
A fresh adversary was re-dispatched immediately. The re-dispatch is the pass-3 record. No
process gap — infrastructure transient; retry succeeded.

**Code tip reviewed:** `b583c4b4`
**Fixes committed through:** spec-only (story v2.5/a90c4b4); code tip UNCHANGED (`b583c4b4`)

### Part A — Pass-1 and Pass-2 Fix Verification

Independent count reconciliation: 40 BAD + 45 GOOD + 6 hermetic = 91 confirmed.

| Finding | Status |
|---------|--------|
| F-S176P1-001 | VERIFIED-FIXED |
| F-S176P1-002 | VERIFIED-FIXED |
| F-S176P1-003 | VERIFIED-FIXED |
| F-S176P1-004 | ACCEPTED — not re-litigated |
| F-S176P1-005 | VERIFIED-FIXED |
| F-S176P1-006 | VERIFIED-FIXED |
| F-S176P1-007 | LEDGERED — not re-raised |
| F-S176P1-008 | VERIFIED-FIXED |
| F-S176P2-001 | VERIFIED-FIXED |
| F-S176P2-002 | VERIFIED-FIXED |
| F-S176P2-003 | ACCEPTED — not re-litigated |
| Obs-1 | ACCEPTED — not re-litigated |
| Obs-2 | ACCEPTED — not re-litigated |

All 8 pass-1 and 2 pass-2 fixable findings VERIFIED-FIXED.

### Part B — New Findings

| ID | Severity | Category | Disposition | Fix |
|----|----------|----------|-------------|-----|
| F-S176P3-001 | MEDIUM | spec coherence | FIXED | story v2.5 (a90c4b4) |
| Obs-A | INFO | pattern logic | ACCEPTED | analogous to accepted F-S176P2-003 |
| Obs-B | INFO | verification breadth | ACCEPTED | residue of accepted F-S176P1-004 |

**1 MEDIUM (spec-route → story v2.5). 2 informational observations ACCEPTED.
HIGH/CRITICAL: 0. No code changes — code tip b583c4b4 UNCHANGED.**

---

### Finding Details

#### F-S176P3-001 — MEDIUM — bin/test_gitignore_mutants_glob.py absent from story Architecture Mapping and traces_to

The new tracked deliverable `bin/test_gitignore_mutants_glob.py` (added during Steps 1–4
delivery) was absent from the story's Architecture Mapping section and `traces_to` list —
a strict-TDD coherence gap.

**Disposition:** FIXED
**Fix:** Story v2.5 (deliverable-map rows added, AC-176-003 regression-guard note added;
DF-SIBLING-SWEEP 5/5 develop files + factory doc verified PRESENT, no phantom ci.yml
entries added; input-hash re-baselined a90c4b4 canonical via `bin/compute-input-hash`,
orchestrator-verified).

---

#### Obs-A — INFO — pattern-28 leading-boundary latency

Pattern-28 exhibits leading-boundary latency on certain match sequences, analogous to the
accepted F-S176P2-003 informational observation (pattern-29 lookahead over/under-shoot).
Zero current-tree impact confirmed.

**Disposition:** ACCEPTED
**Rationale:** Faithful to spec; analogous to accepted F-S176P2-003. No fix required for
convergence.

---

#### Obs-B — INFO — verification-command breadth conservative-and-sound

The verification command breadth is conservative-and-sound, consistent with the residue of
accepted F-S176P1-004 (pattern-scope narrowing). Zero coverage gap identified.

**Disposition:** ACCEPTED
**Rationale:** Conservative-and-sound residue of accepted F-S176P1-004. AC constraints
satisfied. No fix required for convergence.

---

<!-- Update this file after each pass. Add Pass N section with same structure. -->
<!-- Set converged: true in frontmatter when BC-5.39.001 clean-streak of 3 achieved. -->
