# Wave-75 Gate — Gate Findings Log

**Wave:** 75  
**Pass:** P1 (remediation pass, 2026-07-13)  
**Reviewer:** adversarial gate pass (W1)  
**Gate verdict post-remediation:** PENDING (findings closed; gate continues)

---

## Findings

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| F-W75G-P1-001 | LOW | FIXED | STATE.md D-434 record lacked merge-authorization attribution; appended "squash-merge HUMAN-AUTHORIZED at orchestrator merge-authorization gate — pr-manager executed per DF-MERGE-AUTH-CLASSIFIER-001/DF-PR-MANAGER-COMPLETE-001 steps 8-9" inline after the SHA/date, matching D-431 attribution style. Adjudication note added: delivery D-records SHOULD carry merge-auth attribution going forward (sibling-consistency with D-431). |
| F-W75G-P1-002 | LOW | FIXED | currency-sweep.md Step 2 tense-audit paragraph cited STORY-165.md "line 84" as the delivery-anchor location; ground truth verified as lines 81-82 (gap-closed annotation spans two lines). Corrected to "lines 81-82". Sibling sweep per DF-SIBLING-SWEEP-001: three line citations in record audited — (1) "STORY-INDEX index cell (line 230)" CONFIRMED CORRECT; (2) "line 84" STALE → fixed to "lines 81-82"; (3) AC-165-004.md "line 151 may shift" / "Line 151 shifted to 153" are accurate quotations of AC-165-004.md content, not stale citations. No further drift. |
| F-W75G-P4-001 | MEDIUM | FIXED | currency-sweep.md Step 3 blanket-provenance claim "All four demo-evidence files were captured in worktree `ci/story-165-bin-selftest` at commit 9ae8b04" split per Method: headers: AC-165-001.md = worktree `ci/story-165-bin-selftest` @ 9ae8b04 (pre-merge); AC-165-002/003/004.md = factory-artifacts branch, main repo cwd. Sibling sweep (DF-SIBLING-SWEEP-001): grep for "9ae8b04", "all four", "All four" across currency-sweep.md + findings.md — one legitimate single-file attribution remains (currency-sweep.md:64: AC-165-001.md attributed to worktree `ci/story-165-bin-selftest` @ 9ae8b04, correct per-method); no blanket all-four aggregation claims remain. |

---

**Remediation burst:** NOT COMMITTED (rides the gate-close burst per task instruction).  
**Files touched:** `.factory/STATE.md` (D-434 row), `.factory/cycles/wave-75/wave-gate/currency-sweep.md` (line 58; Step 3 blanket-provenance sentence split F-W75G-P4-001), `.factory/cycles/wave-75/wave-gate/findings.md` (this file, new; F-W75G-P4-001 appended).
