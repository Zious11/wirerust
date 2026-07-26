---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
consumer: state-manager
timestamp: 2026-07-26T00:00:00Z
cycle: "wave-086"
pass: 7
total_findings: 14
severity_breakdown: "0 CRIT / 3 HIGH / 6 MED / 5 LOW"
stories_reviewed:
  - STORY-182
  - STORY-183
outcome: REMEDIATED
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
---

# Wave-86 Adversarial Review — Pass 7 Findings

**Pass:** 7 of wave-86 story adversarial review
**Date:** 2026-07-26
**Stories:** STORY-182 v1.6, STORY-183 v1.6
**Total findings:** 14 (0 CRIT / 3 HIGH / 6 MED / 5 LOW)
**Process-gap:** 1 — F-W86S-P7-013 [process-gap]
**Outcome:** ALL 14 REMEDIATED — STORY-182 v1.7 + STORY-183 v1.7; STORY-INDEX v4.03.
**Clean streak:** 0/3. Pass 8 next.
**Partial-fix regression note:** F-W86S-P7-003 (quoted-phrase mechanism) is the 4th consecutive pass recurrence of the same class; orchestrator imposed per-fix grep-evidence mandate effective this burst.

---

## Findings

### F-W86S-P7-001 [HIGH] — module-qualified --exact filters required; bare test-name causes multi-module CI collision

**Story:** STORY-182
**Status:** REMEDIATED (v1.7)
**Summary:** `cargo test -- test_fixture_manifest_all_present` without `--exact` or a module-qualified path matches across all modules; STORY-182 Task 1 Steps and ACs used a bare test name that CI would resolve ambiguously. Fix: all 20 invocation sites updated to module-qualified `--exact` form; bidirectional FIXTURE_GATED_TESTS set-equality check added.

---

### F-W86S-P7-002 [HIGH] — 19 governing-cite sites still reference v5; completeness claim falsified

**Story:** STORY-183
**Status:** REMEDIATED (v1.7)
**Summary:** STORY-183 v1.6 claimed to propagate v6 governing cite throughout, but grep revealed 19 remaining `v5` references in the body. The completeness claim was false. Fix: all 19 sites updated to v6; per-fix grep evidence produced confirming 0 remaining v5 cites.

---

### F-W86S-P7-003 [HIGH] — quoted-phrase Task-6 mechanism resurrected; 4th-pass regression of same class

**Story:** STORY-183
**Status:** REMEDIATED (v1.7)
**Summary:** Task 6 quoted-phrase matching mechanism — purportedly delivered in pass-6 — reappeared in STORY-183 v1.6 in a form that had not been actually delivered. This is the 4th consecutive adversarial pass to find the same quoted-phrase class in this story. Orchestrator evidence: grep confirmed the mechanism was absent from the body text. Fix: Task 6 quoted-phrase mechanism finally delivered with grep-verified body presence; per-fix grep evidence attached.

---

### F-W86S-P7-004 [MED] — TC1–TC21 test cases claimed in changelog but absent from body

**Story:** STORY-183
**Status:** REMEDIATED (v1.7)
**Summary:** STORY-183 v1.6 changelog claimed "TC1–TC21 finally delivered" but the body did not contain the TC1–TC21 test-case specification rows. Fix: TC1–TC21 fully written into the body with grep evidence confirming all 21 rows present.

---

### F-W86S-P7-005 [MED] — rename arithmetic 13+1=14 inconsistent; body count contradicts changelog

**Story:** STORY-183
**Status:** REMEDIATED (v1.7)
**Summary:** The rename map had 13 functional renames plus 1 prose rename = 14 total, but the body section that listed the rename sites showed a different count. Fix: rename arithmetic reconciled to 14; body list enumerated to match; per-fix grep sweep confirmed 14 unique rename sites.

---

### F-W86S-P7-006 [MED] — ci.yml scope covers 4 loci but AC cited only 2; scope contradiction

**Story:** STORY-183
**Status:** REMEDIATED (v1.7)
**Summary:** The AC describing ci.yml update scope referenced 2 loci, but the task body identified 4 loci (:434, :462, :577, :581-585) requiring scope. Fix: AC reworded to enumerate all 4 loci; grep sweep of ci.yml structure confirmed 4 named insertion points.

---

### F-W86S-P7-007 [MED] — FIXTURE_GATED_TESTS registry is one-way (add-only); set-equality never asserted

**Story:** STORY-182
**Status:** REMEDIATED (v1.7)
**Summary:** The FIXTURE_GATED_TESTS registry AC specified that tests must be added to the set when marked `#[ignore]`, but no AC asserted the inverse (that all tests in the set must actually be `#[ignore]`-annotated). A test removed from `#[ignore]` without removing it from the registry would silently produce a stale entry. Fix: bidirectional set-equality check added to AC; registry verified both ways.

---

### F-W86S-P7-008 [MED] — pipefail absent from count-bearing grep step; CI exit code not gated

**Story:** STORY-182
**Status:** REMEDIATED (v1.7)
**Summary:** The count-bearing grep + GATES-the-test-job ACR step in Task 1 lacked `set -o pipefail`, meaning a failed grep in a pipe would not propagate a non-zero exit to the CI job. Fix: `set -euo pipefail` added to the shell invocation; CI step confirmed to gate the test job.

---

### F-W86S-P7-009 [MED] — TestDissectIec104.pcap provenance unverifiable; orchestrator single-capture ruling

**Story:** STORY-182
**Status:** REMEDIATED (v1.7) — skip path retained; provenance ruling recorded
**Summary:** GitHub API query (2026-07-26) showed ITI/ICS-Security-Tools has no per-file provenance for TestDissectIec104.pcap (no folder README; top-level README says only "Various IEC 60870-5-104 Captures"; filename indicates Wireshark-dissector-test origin). Orchestrator ruling: commit only `iec104-iti-diverse.pcap`; TestDissectIec104.pcap remains gitignored as local-samples. Skip path (`iec104-iti-dissect`) retained. Human story gate flagged: dissect e2e carries no CI coverage; diverse carries BC-2.19.029/030 timed-command coverage. Fix: STORY-182 updated to single-capture commit scope with provenance ruling documented.

---

### F-W86S-P7-010 [LOW] — forbidden assertion form used in Task 2 step

**Story:** STORY-182
**Status:** REMEDIATED (v1.7)
**Summary:** Task 2 contained an assertion in the form `assert!(expr)` where `assert_eq!(actual, expected)` or an equivalent comparison form is required by project style (DF-ASSERT-FORM-001). Fix: assertion rewritten to the permitted comparison form.

---

### F-W86S-P7-011 [LOW] — EC-001 cited incorrect detection mechanism in error-case description

**Story:** STORY-182
**Status:** REMEDIATED (v1.7)
**Summary:** EC-001 described the detection mechanism as "file not found" but the actual mechanism is "fixture count mismatch" (the file exists but the manifest entry is absent). Fix: EC-001 reworded to cite the correct mechanism; grep confirmed no other sites carried the false description.

---

### F-W86S-P7-012 [LOW] — two-dot diff (..) used instead of three-dot (...) CI-equivalent diff

**Story:** STORY-183
**Status:** REMEDIATED (v1.7)
**Summary:** The diff command in Task body used `git diff develop..HEAD` (two-dot, symmetric difference including shared history) where `git diff develop...HEAD` (three-dot, CI-equivalent tip-to-merge-base) is required for correct delta isolation. Fix: two-dot replaced with three-dot at all body locations; `.name` comparison also updated.

---

### F-W86S-P7-013 [LOW] [process-gap] — tool self-prose sweep gap; state-manager prose not verified against all output loci

**Story:** STATE.md / process discipline
**Status:** REMEDIATED (v1.7) — process-gap-ledger PG-W86-010 added
**Summary:** The state-manager's own prose output was not verified against all tool-output loci that reference the wave-86 convergence state. Specifically, the "grep-evidence mandate" introduced this burst was not propagated to the process-gap-ledger entry that codifies the remediation discipline. Fix: PG-W86-010 added to process-gap-ledger codifying the per-fix verification evidence mandate; this findings record updated accordingly.

---

### F-W86S-P7-014 [LOW] — STORY-183 Task body contained unauthorized STATE.md write mandate

**Story:** STORY-183
**Status:** REMEDIATED (v1.7)
**Summary:** A task step in STORY-183 body instructed the implementer to write directly to STATE.md to record drift items. STATE.md is state-manager-owned (single-writer discipline TD-VSDD-053); stories must not mandate direct writes to it. Fix: the write mandate removed; replaced with "notify state-manager to record drift item" instruction form.

---

## Remediation Summary

All 14 findings remediated in the D-523 burst:

- STORY-182 v1.6 → v1.7 (F-001, F-007, F-008, F-009, F-010, F-011)
- STORY-183 v1.6 → v1.7 (F-002, F-003, F-004, F-005, F-006, F-012, F-014)
- STORY-INDEX v4.02 → v4.03 (body rows v1.6→v1.7; no numeric totals changed)
- PG-W86-010 added to process-gap-ledger (F-013 [process-gap])

Key orchestrator facts:
- F-009 provenance ruling: commit only `iec104-iti-diverse.pcap`; TestDissectIec104.pcap gitignored
- Per-fix grep-evidence mandate introduced (4th-pass regression broke by mandatory evidence)
- Canonical input-hashes preserved: STORY-182=9a0f34c / STORY-183=9c9b12f

Clean streak: 0/3. Pass 8 next.
