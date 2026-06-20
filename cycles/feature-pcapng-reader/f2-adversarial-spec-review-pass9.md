---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
phase: F2
pass: 9
verdict: CLEAN
clean_pass_number: 2
clean_pass_required: 3
policy: BC-5.39.001
novelty: LOW
finding_counts:
  critical: 0
  high: 0
  medium: 1
  low: 3
total: 4
date: 2026-06-20
status: REMEDIATED
---

# F2 Adversarial Spec Review — Pass 9

## Verdict: CLEAN (0 CRITICAL / 0 HIGH) — CLEAN-PASS 2/3

**Novelty class:** LOW — adversary stated "the spec has effectively converged." The pass-9
findings are residual precision gaps in taxonomy parameterization (MEDIUM-1) and minor
annotation/anchor consistency issues (LOW-1/2/3). No new defect classes introduced.

**Trajectory:** 23 / 24 / 17 / 13 / 13 / 13 / 12 / 8 / [4]

**Clean-pass counter (BC-5.39.001):** 2/3. One more consecutive clean pass required before
F3 story decomposition is unblocked.

**Adversary statement (summary):** "The spec has effectively converged. All structural
framing BCs, VP assignments, holdout coverage, and error-routing decisions are internally
consistent. Remaining findings are precision and completeness gaps in the error-taxonomy
annotation layer and one HS-104 holdout anchor precision gap — none introduce new defect
classes."

---

## Pass-9 Findings

### MEDIUM-1: E-INP-009 taxonomy message not parameterized (EPB + SPB asymmetry)

**Severity:** MEDIUM

**Finding:** error-taxonomy E-INP-009 row "Message Format" field carried a single non-
parameterized message string that did not distinguish EPB-before-IDB from SPB-before-IDB.
BC-2.01.012 PC5a mandates the EPB message include the decimal `interface_id` field:
`"EPB references interface_id=<id> but interface table is empty — no IDB has been parsed"`.
BC-2.01.013 PC5/AC-001 mandates a distinct SPB message:
`"SPB encountered but interface table is empty — no IDB has been parsed"`.
The taxonomy was silent on both parameterized forms — an implementer reading the taxonomy
alone would not know either message string. This is a LOW-1 sibling asymmetry for SPB and
a completeness gap for EPB. Mirror of the E-INP-010 parameterization pattern that was
established by Decision 20.

**Status:** FIXED — error-taxonomy v3.5→v3.6: E-INP-009 Message Format field updated to
specify per-block-type parameterized message strings. EPB message (mandated by BC-2.01.012
PC5a) and SPB message (mandated by BC-2.01.013 PC5/AC-001) both stated. Notes updated to
cite BC owners. BC-refs updated to include BC-2.01.012, BC-2.01.013, BC-2.01.017. D-163.

---

### LOW-1: SPB E-INP-009 message string unconstrained in taxonomy

**Severity:** LOW

**Finding:** SPB path to E-INP-009 (empty interface table) was confirmed in BC-2.01.013
AC-001 but the taxonomy E-INP-009 row provided no canonical SPB message string, leaving
implementation unconstrained. Addressed jointly with MEDIUM-1.

**Status:** FIXED — aligned via MEDIUM-1 fix: error-taxonomy v3.6 SPB message string
mandated by BC-2.01.013 cited explicitly in E-INP-009 Notes. D-163.

---

### LOW-2: HS-104 Case E padding-overrun unreachable-on-aligned-block (PC6b now defense-in-depth)

**Severity:** LOW

**Finding:** HS-104 Case E uses btl=47 (non-4-aligned). Per pcapng spec (and crate
enforcement per Decision 20 / E-INP-010 framing path), btl=47 (47%4=3) is REJECTED by the
crate before wirerust body-decode code (PC6b padding-overrun check) can run. The Case E
fixture therefore cannot reach the PC6b padding-overrun path as stated in the holdout
rationale. The holdout asserts E-INP-008 (PC6b) but the actual primary path is
E-INP-010 (crate alignment rejection). BC-2.01.012 v1.8 now carries explicit PC6a/PC6b
anchor labels.

**Status:** FIXED — HS-104 v1.4→v1.5: Case E downgraded — fixture btl=47 (crate alignment
rejection) asserts NO-PANIC / graceful-Err; E-INP-010 (crate alignment rejection) is the
expected primary path; PC6b (padding-overrun → E-INP-008) noted as DEFENSE-IN-DEPTH /
unreachable on a well-framed block per BC-2.01.012 PC6b. BC Linkage table, Evaluation
Rubric, Edge Conditions, Failure Guidance, and Verification Approach updated. D-163.

---

### LOW-3: PC6a/PC6b anchors missing from BC-2.01.012; PC9 dedup with PC6b

**Severity:** LOW

**Finding:** BC-2.01.012 did not carry explicit PC6a / PC6b sub-label anchors in
Postcondition 6, even though HS-104 Case E cited "PC6b (padding-overrun)" and pass-8
BC-2.01.012 v1.7 referenced the precedence postcondition. Without explicit PC6a/PC6b
anchors in the BC, the holdout citation is unresolvable. Also, Postcondition 9 (no-panic)
partially duplicates the defense-in-depth coverage already mandated by PC6b, creating a
redundancy note worth resolving.

**Status:** FIXED — BC-2.01.012 v1.7→v1.8: Postcondition 6 split into labeled PC6a
(captured_len/padding guard — fires when body well-formed but captured_len arithmetic
overflows the body) and PC6b (padding-overrun guard — defense-in-depth on non-aligned
captured_len; unreachable when crate alignment rejection fires first per E-INP-010 primary
path). PC9 dedup note added. D-163.

---

## Summary Table

| ID | Severity | Finding | Status |
|----|----------|---------|--------|
| MEDIUM-1 | MEDIUM | E-INP-009 taxonomy message not parameterized (EPB+SPB) | FIXED — error-taxonomy v3.6 D-163 |
| LOW-1 | LOW | SPB E-INP-009 message string unconstrained in taxonomy | FIXED — aligned via MEDIUM-1 D-163 |
| LOW-2 | LOW | HS-104 Case E padding-overrun unreachable-on-aligned-block (PC6b defense-in-depth) | FIXED — HS-104 v1.5 D-163 |
| LOW-3 | LOW | PC6a/PC6b anchors missing from BC-2.01.012; PC9 dedup note | FIXED — BC-2.01.012 v1.8 D-163 |

---

## Artifacts Updated This Pass (D-163)

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| error-taxonomy | v3.5 | v3.6 | MEDIUM-1 / LOW-1 (E-INP-009 parameterized EPB+SPB messages) |
| BC-2.01.012 | v1.7 | v1.8 | LOW-3 (PC6a/PC6b anchors; PC9 dedup note) |
| BC-2.01.013 | v1.8 | v1.9 | LOW-1 sibling (SPB E-INP-009 AC-001 cross-ref confirmed; no normative change) |
| HS-104 | v1.4 | v1.5 | LOW-2 (Case E downgraded to E-INP-010 primary / PC6b defense-in-depth) |
| BC-INDEX | v1.66 | v1.67 | BC-2.01.012 v1.7→v1.8; BC-2.01.013 v1.8→v1.9 annotations synced |
| spec-changelog | — | — | [pcapng-f2-pass9-clean-and-remediation-2026-06-20] prepended |
| STATE.md | — | — | phase_status + trajectory + D-163 + clean-pass 2/3 recorded |
| f2-review-remediation-tracker.md | — | — | Pass-9 section added; MEDIUM-1/LOW-1/2/3 FIXED; CLEAN-PASS 2/3 recorded |

---

**Clean-pass counter as of D-163: 2/3. Adversary pass-10 pending (targeting clean-pass 3/3 → CONVERGENCE).**
**F3 BLOCKED until pass-10 produces a CLEAN result (clean-pass 3/3).**
