---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-26T08:00:00Z
cycle: "wave-086"
pass: 14
verdict: NOT_CONVERGED
novelty: "medium-low"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 14

**Date:** 2026-07-26
**Pass:** 14 of N
**Verdict:** NOT CONVERGED
**Novelty:** medium-low — "expected residue profile at pass 14"; no new classes introduced; findings are continuation of known sweep-completeness and conventions axes. SECOND consecutive zero-HIGH pass.
**Tally:** 8 findings — 0 CRIT / 0 HIGH / 3 MED / 3 LOW + 2 NITs (all fixed)
**Status:** REMEDIATED — D-531 state burst; STORY-182 v2.3→v2.4 + STORY-183 v2.3→v2.4
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Positives from adversary (verified-clean axes):** ground-truth predicate correctness; inert-predicate elimination (all always() loci confirmed zero or replaced); prescribed-code validity (Task-8 code blocks sound modulo ordering gap F-001); policy-compliance (no-literal-phrase standing discipline D-529 satisfied; DF-SIBLING-SWEEP-001 compliance confirmed); pathspec subsumption direction consistent across all loci (v2.4 confirms 7-loci agreement holds); self-anchor elimination complete (zero :NNN intra-doc self-citations in either story); canonical hashes 9a0f34c/9c9b12f unchanged; convergence-report.md citations fully path-qualified; AC-183-009 gating forms canonical; ci.yml order-dependence labels present.

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P14-001 | MED | STORY-183 | FIXED v2.4 | tdd_mode ordering: Task 8 never adds Patterns 32-37. Task 8 prescribed the tdd_mode RED step for the new TIER-1 patterns but was not split into sub-tasks 8a (add patterns to scanner) + 8b (verify RED in tdd_mode). An implementer following Task 8 as written would add all patterns without verifying that each new pattern triggers RED on a known-violating file first. Fix: Task 8 split into 8a (add patterns 30-37 to PATTERN_LIST with their discriminators and GOOD/BAD annotations) and 8b (run tdd_mode against violating.py and confirm each new pattern produces at least one FAIL line before committing). Ordering restated: 8a must pass before 8b, 8b must pass before Task 9. |
| F-W86S-P14-002 | MED | STORY-182 | FIXED v2.4 | 4th unswept if:always() locus at :540. The D-530 pass-15 sweep of if:always() loci converted 3 loci to if:!cancelled(); a 4th locus at :540 (the fixture-count gate-entry reporting step) retained if:always(). The prior sweep rationale noted three loci explicitly; the :540 locus was added in a subsequent burst and was not covered by the sweep note. Fix: :540 converted to if:!cancelled(); sweep note updated to confirm zero live if:always() loci remain in STORY-182's ci.yml task description. |
| F-W86S-P14-003 | MED | STORY-182 | FIXED v2.4 | Task 7 E2E-PCAPS.md sweep omitted 3 IEC-104-section loci. Task 7 prescribed a provenance-attribution sweep of E2E-PCAPS.md and listed 3 loci (lines ~48-50, ~100-105, ~200-210). Three additional loci in the IEC-104-specific sections of E2E-PCAPS.md (lines ~359-365, ~410-415, ~430-440) carry the same ITI CC-BY-4.0 attribution language and were not included in the sweep list. Fix: sweep extended to 6 loci; Arch Mapping and FSR rows for E2E-PCAPS.md updated for consistency with the extended sweep scope (both tables now list 6 loci, not 3). |
| F-W86S-P14-004 | LOW | STORY-183 | FIXED v2.4 | Pattern 33 comment: "falls through to" phrase appears in the inline code comment that illustrates the TIER-1 pattern. A comment-text word-wrap at exactly "falls" / "through to" would produce a line ending in "falls" followed by a line starting with "through to" — the scan would miss the two-word trigger. Fix: the inline comment illustration for Pattern 33 reworded to use a wrap-safe single-token marker form that cannot be split by a line-wrap, with a note that the actual TIER-1 trigger is the contiguous two-word sequence "falls through to" and implementers must ensure this sequence spans no line boundary in the emitted code comment. |
| F-W86S-P14-005 | LOW | STORY-183 | FIXED v2.4 | Bare asserts outside PASS/FAIL runner convention. Two assert!() calls in the Task 8b verification steps were written as bare assert! outside the pass/fail print-runner scaffold. The PASS/FAIL runner convention (established in prior passes) requires that each predicate be wrapped in if !predicate { println!("FAIL [...]"); } so that failure output is parseable by the tdd_mode harness. Fix: bare assert! calls respecified as PASS/FAIL-convention predicate blocks; tdd_mode invocation note updated to reference the PASS/FAIL output assertion as the gate criterion. |
| F-W86S-P14-006 | LOW | STORY-182 | FIXED v2.4 | /tmp backup pre-existence guards absent. Three Task procedures in STORY-182 that write to a /tmp backup path (e.g., /tmp/e2e_backup_$$.pcap) did not guard against the case where that path already exists (e.g., from a prior interrupted test run). If the path exists, the backup step would overwrite it without warning, potentially hiding a previous run's artifact. Fix: each of the three /tmp backup procedures now includes a guard: `[ -e "$BACKUP" ] && rm -f "$BACKUP"` before writing, with a rationale note explaining that pre-existence indicates a prior interrupted run. |

---

## Findings Detail

### F-W86S-P14-001 (MED) — tdd_mode ordering: Task 8 unsplit; Patterns 32-37 never RED-gated

**Story:** STORY-183
**Status:** FIXED v2.4 — Task 8 split into 8a/8b; ordering restated

**Description:** Task 8 in STORY-183 v2.3 described adding the new TIER-1 patterns (Patterns 30-37)
to the scanner's PATTERN_LIST. The task was a single monolithic block: add all patterns, then run
the scanner. It did not require the implementer to verify that each newly added pattern triggers
a RED (FAIL) result in tdd_mode against a known-violating file before committing the pattern.

Without this RED-gate step, an implementer could add all 8 patterns in one shot, run tdd_mode on a
clean corpus, observe 0 FAILs, and commit — having never verified that the patterns are actually
detectable. A mis-specified pattern (e.g., a regex that never matches) would silently pass.

This finding is the "ordering gap" class: the task body was correct about WHAT to add but left
ambiguous the sequence of verification steps.

**Fix (v2.4):**
- Task 8 split into:
  - **8a: Add patterns** — add Patterns 30-37 to PATTERN_LIST with discriminators and GOOD/BAD
    annotations; run unit self-tests (Task 1 harness) to confirm pattern data structures are valid.
  - **8b: Verify RED in tdd_mode** — for each new pattern, run `bin/check-green-doc-tense
    --tdd-mode` against a file containing exactly one synthetic violation of that pattern; confirm
    FAIL output appears. Only after all 8 patterns produce RED does the implementer commit.
- Ordering note added: 8a must complete before 8b; 8b must complete before Task 9 (hermetic harness).

---

### F-W86S-P14-002 (MED) — 4th unswept if:always() locus at :540

**Story:** STORY-182
**Status:** FIXED v2.4

**Description:** The D-530 pass-13 remediation (F-W86S-P13-015) converted three if:always()
loci in STORY-182's ci.yml task to if:!cancelled(). The sweep note in STORY-182 v2.3 stated
"three loci converted; zero live if:always() remain". However, a 4th ci.yml step (the
fixture-count gate-entry reporting step at :540) had been added in the D-529 burst and was
not covered by the D-530 sweep because the sweep note was written before the D-529 additions
were merged.

At :540, if:always() was prescribed for the gate-entry fixture count report. The correct
form is if:!cancelled(), consistent with the standing discipline established at F-W86S-P13-015.

**Fix (v2.4):** :540 converted to if:!cancelled(); sweep note updated to enumerate all 4
previously-converted loci and confirm zero live if:always() loci remain in STORY-182's
ci.yml task scope.

---

### F-W86S-P14-003 (MED) — Task 7 E2E-PCAPS sweep: 3 IEC-104-section loci omitted

**Story:** STORY-182
**Status:** FIXED v2.4

**Description:** Task 7 in STORY-182 prescribed a provenance-attribution sweep of
`.factory/maintenance/E2E-PCAPS.md` listing 3 target loci. The file contains IEC-104-specific
sections (added during the wave-85 gate-fix PR #439) that carry the same ITI CC-BY-4.0
attribution language. Three additional loci in these sections (~:359-365, ~:410-415, ~:430-440)
were not included in the Task 7 sweep list.

An implementer following Task 7 as written would sweep only the first 3 loci, leaving the
IEC-104 sections with potentially stale attribution language.

**Fix (v2.4):** Task 7 sweep extended from 3 to 6 loci (3 original + 3 IEC-104 section
loci added). Architecture Mapping row for E2E-PCAPS.md updated to reflect 6-loci scope.
FSR (Functional Scope Row) updated for consistency.

---

### F-W86S-P14-004 (LOW) — Pattern 33 comment: "falls through to" word-wrap needle

**Story:** STORY-183
**Status:** FIXED v2.4

**Description:** The TIER-1 Pattern 33 trigger is the two-word sequence "falls through to".
STORY-183 v2.3 included an inline code comment that illustrated the pattern by containing the
literal phrase "falls through to" as part of the comment text. If this comment is word-wrapped
at the line boundary between "falls" and "through to" (possible in editors with hard wrap at
80 or 100 columns), the scan would see "falls" and "through to" on separate lines and fail to
match the two-word trigger.

**Fix (v2.4):** The Pattern 33 illustration comment reworded to use a wrap-safe form that does
not rely on "falls through to" appearing as a contiguous phrase in a comment that could be
wrapped. Rationale note added explaining that the actual TIER-1 trigger must be detected as a
contiguous sequence and that comment-wrap risk is real in auto-formatted code.

---

### F-W86S-P14-005 (LOW) — Bare asserts outside PASS/FAIL runner convention

**Story:** STORY-183
**Status:** FIXED v2.4

**Description:** Task 8b (post-split) prescribed assert!() calls to verify that each newly
added pattern produced RED output. Two of these asserts were written as bare `assert!(condition,
"message")` rather than the `if !condition { println!("FAIL [...]"); } else { println!("PASS
[...]"); }` convention established by prior passes.

The tdd_mode harness parses stdout for lines beginning with "FAIL " or "PASS " to determine
the gate result. A bare assert! that panics produces stderr output, not the expected FAIL line
on stdout, causing the harness to report 0 failures even when the assert fails.

**Fix (v2.4):** Both assert! calls in Task 8b respecified as PASS/FAIL convention blocks;
a cross-reference note added to the PASS/FAIL convention definition in Task 5.

---

### F-W86S-P14-006 (LOW) — /tmp backup pre-existence guards absent

**Story:** STORY-182
**Status:** FIXED v2.4

**Description:** Three Task procedures in STORY-182 write temporary backup files under /tmp
(e.g., `/tmp/e2e_pcap_backup_$$.pcap`). None of the three procedures guarded against the
case where the destination path already existed at the time of the backup write.

If a prior test run was interrupted (e.g., by SIGINT during CI), the /tmp backup file from
that run may still exist. The procedures as written would silently overwrite it, hiding the
prior run's artifact and potentially masking an error condition.

**Fix (v2.4):** Each of the 3 /tmp backup procedures prepends:
```
[ -e "$BACKUP" ] && rm -f "$BACKUP"
```
with a rationale comment: "pre-existence indicates prior interrupted run; remove before
creating fresh backup to prevent silent overwrite". The guard uses `rm -f` to handle any
permission edge cases without failing the test setup.

---

## NIT-Observations (2 items — all actioned)

1. **NIT-01 (ACTIONED):** Duplicate Task-10 bullet in STORY-183: Task 10 had two identical
   "verify the scan output" bullet points (inserted during the v2.3 burst — one from the
   original text, one introduced by the D-530 line-anchor update that accidentally duplicated
   the bullet). Merged into a single bullet with the combined phrasing.

2. **NIT-02 (ACTIONED):** v4→v6 tier cite stale in STORY-183: one governance reference cited
   "DF-GREEN-DOC-TENSE-SWEEP v4" in a Note block that post-dates the v4→v6 policy upgrade.
   Updated to v6 at 1 locus; consistent with standing citation-currency discipline.

---

## EXECUTION-REQUIRED Flags (8 items — carried to delivery)

The following items require execution evidence at delivery or gate time. All 8 are carried
forward from prior passes; pass-14 did not introduce new flags.

**(i) Python selftest exit code:** Confirm `python3 bin/test_compute_input_hash.py` exits 0
on develop=e8841d76. Baseline for AC-183-009 item (c).

**(ii) cargo test/clippy on prescribed block:** Confirm `cargo test --all-targets` and
`cargo clippy --all-targets -- -D warnings` both exit 0 on develop=e8841d76 (pre-delivery
baseline; STORY-182/183 add no src/ changes so this is a no-change confirmation).

**(iii) 66-finding expectation vs committed capture:** Verify the diverse ITI e2e test
holds the 66/20/46 expectation on develop=e8841d76 (PR #439 gate-fix 0ab6f52e set this
expectation; back-merge e8841d76 preserved it — verify still current before delivery).

**(iv) sha256/size of fetched captures:** Document exact sha256 and byte-size of
`tests/fixtures/iec104-iti-diverse.pcap` (and dissect variant if committed) on
develop=e8841d76 to anchor the AC-182-002 integrity verification claim.

**(v) hermetic harness end-to-end:** Execute Task 9 hermetic harness (copy script into
fresh tmp dir, run against violating.py, confirm FAIL line appears) to verify the hermetic
environment isolation before delivery.

**(vi) ci.yml step behavior on runner:** Confirm that the ci.yml step prescribed in
STORY-182 Task 10 produces the expected "Fixture coverage: N/4 committed" output on an
actual GitHub Actions runner against develop=e8841d76.

**(vii) git ls-files result sets:** Document exact file count from
`git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py` on develop=e8841d76 to anchor
STORY-183 "N files scanned" claim per AC-183-008 verification.

**(viii) Pattern self-test exit codes:** Confirm that each of the 8 new TIER-1 patterns
(30-37) produces at least one FAIL line when run against violating.py in tdd_mode on
develop=e8841d76 (post-Task-8b split; verification evidence required at gate).

---

## Verified-Clean Table (pass-14 adversary confirmation)

| Verification Item | Result |
|-------------------|--------|
| All finding counts / pass tallies | EXACT |
| Input-hash values 9a0f34c / 9c9b12f | EXACT — canonical Python tool; unchanged |
| No-literal-phrase sweep (D-529 standing discipline) | CONFIRMED — no TIER-1 literals in annotation text |
| Intra-document :NNN self-citations | CONFIRMED ZERO — structural elimination from D-530 holds |
| Pathspec subsumption direction (7-loci agreement) | CONFIRMED — src/*.rs strictly subsumes src/**/*.rs at all 7 loci in v2.4 |
| DF-GREEN-DOC-TENSE-SWEEP v6 compliance | CONFIRMED — no TIER-1 regressions from v2.3→v2.4 edits |
| DF-SIBLING-SWEEP-001 | CONFIRMED — sweep performed; no sibling regressions |
| AC-183-009 gating forms (canonical test "$(grep -c ...)" -eq 0) | CONFIRMED — 3 predicates correct in v2.4 |
| ci.yml order-dependence labels (baseline-e8841d76) | CONFIRMED — labels present at all absolute ci.yml citation sites |
| Ground-truth axis | CONFIRMED CLEAN — zero findings |
| Inert-predicate axis | CONFIRMED CLEAN — zero live if:always() loci after F-P14-002 fix |
| Prescribed-code-validity axis | CONFIRMED CLEAN (post-fix: Task-8 split restores ordering correctness) |
| Policy-compliance axis | CONFIRMED CLEAN — zero findings |

---

## Pass-14 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: medium-low — "expected residue profile at pass 14". No new classes introduced;
all findings are continuation of known axes (sweep completeness, convention consistency).
SECOND consecutive zero-HIGH pass (P10 was the first, P14 is the second).

Severity profile: P12 0C/1H/4M/5L → P13 0C/2H/4M/9L → P14 0C/0H/3M/3L.
Both HIGH regressions from P13 (pathspec truth inversion + self-anchor sweep gaps) are
confirmed not recurred in v2.4: the structural fixes hold.

Pass-15 next after D-531 remediation burst.

Pass tallies (P1–P14): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14 / 10 / 15 / 8.
Total across all passes: 236 findings. Canonical hashes: 9a0f34c / 9c9b12f.

---

## Remediation (D-531)

**Date:** 2026-07-26
**Burst:** D-531 STATE BURST — WAVE-86 ADVERSARIAL PASS 14 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 8 findings FIXED at STORY-182 v2.4 / STORY-183 v2.4.

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P14-001 (MED) | FIXED v2.4 | STORY-183 | Task 8 split into 8a (add patterns) + 8b (verify RED per pattern in tdd_mode); ordering constraint restated |
| F-W86S-P14-002 (MED) | FIXED v2.4 | STORY-182 | :540 converted if:always()→if:!cancelled(); sweep note updated: zero live if:always() loci confirmed |
| F-W86S-P14-003 (MED) | FIXED v2.4 | STORY-182 | Task 7 sweep extended 3→6 loci (3 IEC-104-section loci added); Arch Mapping + FSR updated for consistency |
| F-W86S-P14-004 (LOW) | FIXED v2.4 | STORY-183 | Pattern 33 illustration reworded wrap-safe; rationale note added re contiguous two-word trigger |
| F-W86S-P14-005 (LOW) | FIXED v2.4 | STORY-183 | Bare assert! blocks respecified as PASS/FAIL-convention blocks; cross-reference to Task 5 convention added |
| F-W86S-P14-006 (LOW) | FIXED v2.4 | STORY-182 | pre-existence guards ([ -e "$BACKUP" ] && rm -f "$BACKUP") added to all 3 /tmp backup procedures |

**2 NITs actioned:** duplicate Task-10 bullet merged (STORY-183); v4→v6 tier cite at 1 locus (STORY-183).

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**Second zero-HIGH pass confirmed:** Pass-14 extends the confirmed-clean HIGH-severity axis.
Pass tallies H-count: P1:6H → P2:4H → P3:5H → P4:4H → P5:3H → P6:2H → P7:3H → P8:3H →
P9:5H → P10:0H → P11:1H → P12:1H → P13:2H → P14:0H. Streak reset to 0 (P10 clean / P11+P12+P13
interrupted by HIGH recurrences). Pass-14 streak 0/3.

**Streak:** 0/3. Pass 15 pending adversary dispatch. Trajectory-tail: →15→8.
