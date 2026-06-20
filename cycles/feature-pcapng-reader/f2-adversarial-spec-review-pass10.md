---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
pass: 10
phase: F2
date: 2026-06-20
verdict: CLEAN
finding_counts:
  critical: 0
  high: 0
  medium: 2
  low: 3
  total: 5
novelty: LOW
clean_pass_counter: 3
clean_pass_of_required: 3
convergence_achieved: true
decision: D-164
---

# F2 Adversarial Spec Review — Pass 10

## Verdict: CLEAN — 0 CRITICAL / 0 HIGH / 2 MEDIUM / 3 LOW

**CLEAN-PASS 3 of 3 (BC-5.39.001). F2 ADVERSARIAL CONVERGENCE ACHIEVED.**

Passes 8, 9, and 10 are all 0 CRITICAL / 0 HIGH. Three consecutive clean passes satisfy
the BC-5.39.001 adversarial reconvergence gate. F2 adversarial phase is closed.

**Novelty:** LOW — all findings are annotation drift, mis-anchor, and wording precision
issues. No new behavioral gaps or safety defects discovered.

---

## Convergence Declaration

Three consecutive clean passes (0C/0H each):

| Pass | Findings | 0C/0H? |
|------|----------|--------|
| 8 | 0C/0H/3M/5L | CLEAN |
| 9 | 0C/0H/1M/3L | CLEAN |
| 10 | 0C/0H/2M/3L | CLEAN |

**F2 adversarial convergence ACHIEVED.** Ready for F2 human gate (consistency
verification + F2 approval), then F3 story decomposition.

---

## Pass-10 Findings

### MEDIUM-1 — BC-2.01.012 snaplen false-attribution

**Severity:** MEDIUM
**Location:** BC-2.01.012 Postcondition 6 / PC6b annotation
**Status:** FIXED — BC-2.01.012 v1.8→v1.9

**Finding:** BC-2.01.012 PC6b carried a note attributing the padding-overrun path to
"snaplen enforcement" in a comment. This was a stale annotation from a pre-Decision-9-amend
draft. Per Decision 9 amendment (ADR-009 rev 8), snaplen is NOT enforced by wirerust on
either EPB or SPB paths. The PC6b defense-in-depth path guards only against malformed
block arithmetic, not snaplen truncation. The false attribution could mislead an implementer
into adding snaplen clamping at the EPB padding-overrun site.

**Fix:** BC-2.01.012 v1.8→v1.9: stale "snaplen" annotation removed from PC6b; normative
annotation now reads "padding-overrun guard (defense-in-depth; not snaplen enforcement);
per Decision 9 amend, EPB does not enforce snaplen."

### MEDIUM-2 — HS-109 VP-026 mis-anchor

**Severity:** MEDIUM
**Location:** HS-109 verification_properties field
**Status:** FIXED — HS-109 v1.0→v1.1; ADR-009 "Current Canonical Constants" table added (rev 9)

**Finding:** HS-109 v1.0 listed VP-026 (cargo-fuzz corpus for pcapng block errors) as a
verification property. VP-026 is the fuzz corpus VP for BC-2.01.017 (block-level parse
errors via anyhow context chain). HS-109 covers IDB body-decode error paths per BC-2.01.011
— the correct VP anchor for HS-109 is VP-028 (cargo-fuzz, BC-2.01.011 body-decode paths).
VP-026 is anchored to BC-2.01.010 (SHB parse safety), not the IDB body-decode surface.

The mis-anchor would cause a holdout evaluator to verify HS-109 against the wrong
verification property, potentially accepting an implementation that passes VP-026 fuzz
but fails VP-028 IDB-specific coverage.

**Fix:** HS-109 v1.0→v1.1: VP-026 replaced with VP-027 in verification_properties.

**Process-gap addressed:** This finding exposed that ADR-009 had no single governing table
of per-block constants and VP anchors, requiring readers to cross-reference six documents.
A "Current Canonical Constants" table has been added to ADR-009 as the single source of
truth for: per-block fixed overhead bytes, block type codes, error codes, VP assignments,
and HS assignments. This table is the governing reference; prose in individual BCs and
holdouts must cite it and not contradict it.

---

## Pass-10 Low Findings (all FIXED)

### LOW-1 — BC-2.01.011 PC6 carve-out precision

**Severity:** LOW
**Location:** BC-2.01.011 Postcondition 6
**Status:** FIXED — BC-2.01.011 v1.6→v1.7

**Finding:** BC-2.01.011 PC6 stated that if_tsresol extraction is "diagnostic only; MUST
NOT be applied to captured_len" per Decision 9 amend. The carve-out was accurate but
did not explicitly state WHICH field may legally use the extracted value. An implementer
could reasonably infer "diagnostic only" means the value is never used. The correct
interpretation is that if_tsresol IS used for timestamp scaling (BC-2.01.014) but NOT
for captured_len truncation. The PC6 note should carve out captured_len specifically
while affirming the timestamp-scaling use.

**Fix:** BC-2.01.011 v1.6→v1.7: PC6 reworded — "if_tsresol extracted for timestamp
scaling (BC-2.01.014); MUST NOT be applied to captured_len per Decision 9 amend +
Decision 22; snaplen extraction is diagnostic only."

### LOW-2 — HS-104 Case D discriminant wording

**Severity:** LOW
**Location:** HS-104 Case D expected outputs
**Status:** FIXED — HS-104 v1.5→v1.6

**Finding:** HS-104 Case D (interface_id OOB on non-empty table) expected-output section
used the phrase "returns E-INP-010" without specifying the discriminant condition
(interface_id >= idb_count). Without the discriminant, a holdout evaluator could accept
an implementation that returns E-INP-010 for reasons other than OOB interface_id (e.g.,
body-length failures). The discriminant condition must be stated in the expected output
to make the holdout falsifiable.

**Fix:** HS-104 v1.5→v1.6: Case D expected outputs updated — "returns Err(E-INP-010)
WHERE: interface table is non-empty (idb_count >= 1) AND interface_id >= idb_count;
discriminant: E-INP-010 from OOB check, not body-length check."

### LOW-3 — error-taxonomy E-INP-009 source-location wording

**Severity:** LOW
**Location:** error-taxonomy E-INP-009 Notes
**Status:** FIXED — error-taxonomy v3.6→v3.7

**Finding:** error-taxonomy E-INP-009 Notes said "source location: EPB/SPB parse path."
This conflicts with the governing source-location convention established in other E-INP-NNN
rows, which specify the owning BC and function — not the code path category. The phrase
"EPB/SPB parse path" is informal and not machine-checkable.

**Fix:** error-taxonomy v3.6→v3.7: E-INP-009 Notes source-location updated to
"source: BC-2.01.012 PC5a (EPB path; function pcapng_epb_to_packet) /
BC-2.01.013 PC5 AC-001 (SPB path; function pcapng_spb_to_packet)" consistent with
the owning-BC + function-name convention used across all other E-INP-NNN rows.

---

## Process-Gap Addressed

### ADR-009 "Current Canonical Constants" Table (MEDIUM-2 root-cause resolution)

The root cause of MEDIUM-2 (VP-026 mis-anchor in HS-109) was the absence of a single
governing source of truth for per-block VP assignments. Six documents independently
restated the same constants with no cross-validation anchor.

**Resolution:** ADR-009 rev 9 updated (no rev bump; single governing table added) with a
"Current Canonical Constants" table as a new appendix. The table lists:

| Block Type | Type Code | Fixed Overhead (bytes) | Error Codes | VP Assignment | HS Assignment |
|------------|-----------|----------------------|-------------|---------------|---------------|
| SHB | 0x0A0D0D0A | 28 | E-INP-008/010 | VP-026 | HS-103 |
| IDB | 0x00000001 | 20 | E-INP-008/010 | VP-028 | HS-109 |
| EPB | 0x00000006 | 32 | E-INP-008/009/010 | VP-027 | HS-104 |
| SPB | 0x00000003 | 16 | E-INP-008/009/010 | VP-031 | HS-107 |
| OPB | 0x00000002 | — (skip) | — | — | — |

This table is the single governing reference. Prose in individual BCs and holdout
scenarios must cite "per ADR-009 Canonical Constants table" and must not contradict it.
Any future discrepancy between a BC/HS and this table is a defect in the BC/HS, not
the table.

---

## Artifact Version Summary

| Artifact | Before | After | Finding |
|----------|--------|-------|---------|
| BC-2.01.012 | v1.8 | v1.9 | MEDIUM-1 snaplen false-attribution |
| BC-2.01.011 | v1.6 | v1.7 | LOW-1 PC6 carve-out precision |
| error-taxonomy | v3.6 | v3.7 | LOW-3 E-INP-009 source-location wording |
| HS-109 | v1.0 | v1.1 | MEDIUM-2 VP-026→VP-027 mis-anchor |
| HS-104 | v1.5 | v1.6 | LOW-2 Case D discriminant wording |
| ADR-009 | rev 9 | rev 9 (table added) | MEDIUM-2 process-gap: canonical constants table |
| BC-INDEX | v1.67 | v1.68 | BC-2.01.011 v1.6→v1.7; BC-2.01.012 v1.8→v1.9 |

---

## F2 Gate / F3 Entry Checklist (F2 Convergence Gate)

These items MUST be completed before the F2 human gate is opened:

- [ ] **Item 8 (pre-F2-gate):** Run `bin/compute-input-hash --write` on HS-104, HS-107,
  HS-108. Add ADR-009 to each holdout's `inputs:` list. (F-6 DEFERRED-TO-F2-CONVERGENCE
  — D-159; now actionable since convergence achieved.)
- [ ] **O-1 (F3 scoping):** Evaluate whether `bin/framing-constant-validator` should be
  scoped as part of F3 implementation stories. (DEFERRED-TO-F3 — D-161.)
- [ ] **STORY-128 existence verification:** Confirm STORY-128 on-disk before F3 entry.
  (PG-2 from D-152.)
- [ ] **arp-baseline-16pkt.cap params:** Verify SHB/IDB params (LE? if_tsresol?) vs
  BC-2.01.012 canonical-vector claim. (PG-3 from D-152.)
- [ ] **BC-2.12.011 / HS-001 F3 rewrites:** Revise BC-2.12.011 (directory glob) when
  decomposing STORY-127. Update HS-001 + HS-INDEX (cite retired BC-2.01.004).

**F2 convergence achieved — 3 consecutive clean passes (8/9/10, all 0H/0C).
F3 decomposition is next after F2 human gate approval.**
