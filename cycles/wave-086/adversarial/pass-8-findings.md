---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-26T10:02:00Z
cycle: "wave-086"
pass: 8
total_findings: 12
severity_tally: "0C/3H/6M/3L + 1 process-gap"
verdict: REMEDIATED
inputs: [stories/STORY-182.md, stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Wave-86 Adversarial Pass 8 Findings — REMEDIATED

**Pass date:** 2026-07-26
**Tally:** 0 CRIT / 3 HIGH / 6 MED / 3 LOW + 1 [process-gap] (F-P8-013)
**Decay:** P7 14 findings → P8 12 findings (severity-counted). Decay continues.
**STORY-183 materially converged:** adversary assessed 2 MED table gaps only.
**All 12 severity findings + 1 process-gap REMEDIATED in stories v1.8.**

---

## Finding Index

| ID | Severity | Story | Summary | Disposition |
|----|----------|-------|---------|-------------|
| F-W86S-P8-001 | HIGH | STORY-182 | Non-discriminating licensing rationale — F-009 ruling cited absence of per-file provenance rather than positive evidence of upstream-of-ITI origin | REMEDIATED — discriminator restated: positive evidence (filename TestDissectIec104.pcap + E2E-PCAPS.md description) is the exclusion basis; absence-of-provenance framing retired |
| F-W86S-P8-002 | HIGH | STORY-182 | :125 both-captures residue — task step at line :125 prescribed committing both captures despite F-009 single-capture ruling | REMEDIATED — :125 step rewritten to commit only iec104-iti-diverse.pcap; TestDissectIec104.pcap remains gitignored with explicit rationale |
| F-W86S-P8-003 | MED | STORY-182 | False committed-status comment prescription — Task 7 instructed implementer to mark the dissect capture as "committed" in a comment, contradicting its gitignored status | REMEDIATED — Task 7 rewritten with explicit leave-unchanged instruction for the dissect capture comment; false committed-status annotation removed |
| F-W86S-P8-004 | MED | STORY-182 | 7-loci cardinality residue — acceptance criterion enumerated 7 registry loci but story body described a different count at one or more task steps | REMEDIATED — 7-loci cardinality sweep applied; all task-step enumerations reconciled to match the AC count |
| F-W86S-P8-005 | MED | STORY-182 | SIGPIPE-under-pipefail false-red — tee-to-file invocations used pipeline form (cmd piped to tee) that causes false SIGPIPE failures under set -o pipefail | REMEDIATED — all 3 tee-to-file sites rewritten to SIGPIPE-safe process-substitution form |
| F-W86S-P8-006 | MED | STORY-182 | Vacuous Env A regex — Env A assertions used regex patterns that trivially match any string, providing no discriminating validation of the actual fixture content | REMEDIATED — Env A assertions rewritten with 4/4 non-vacuous Wireshark-specific patterns; zero-SKIP assertion added |
| F-W86S-P8-007 | MED | STORY-182 | Unasserted registry test-name column — the FIXTURE_GATED_TESTS registry defined a test-name column but no AC asserted that include_str! path derivation coupled to the registry entries | REMEDIATED — include_str! registry coupling assertion added; test-name column coverage verified |
| F-W86S-P8-008 | HIGH | STORY-182 | False Task-1 precondition + curl clobber risk — Task 1 stated a precondition that was not satisfied by the implementation path; curl download steps lacked clobber-guard (-o vs redirect) | REMEDIATED — Task 1 precondition corrected to honest precondition; curl steps updated with explicit clobber-guard form and honest precondition statement |
| F-W86S-P8-009 | MED | STORY-183 | Missing FSR/Arch Mapping rows — test_lint_cycle_artifact.py was added as an acceptance criterion but lacked corresponding rows in the Functional Specification Reference and Architecture Mapping tables | REMEDIATED — test_lint_cycle_artifact.py rows added to both FSR and Arch Mapping sections |
| F-W86S-P8-010 | LOW | STORY-183 | :125 stale count newly scan-eligible — scrub site at line :125 contained a count that was stale after v1.7 changes and was now within the scan scope defined by the story | REMEDIATED — :125 added to the scrub list (:3,:5,:6,:125); stale count at :125 corrected |
| F-W86S-P8-011 | LOW | STORY-183 | EC-011 scrub-list drift — EC-011 assertion referenced scrub sites :3,:6 but the actual scrub list had grown to :3,:5,:6 in v1.5 and :125 in v1.8 | REMEDIATED — EC-011 updated to reference :3,:5,:6,:125 matching the canonical scrub list |
| F-W86S-P8-012 | LOW | STORY-183 | Sibling story-spec references → preserve adjudication — AC referenced behavioral contracts from STORY-158 and STORY-162 without adjudicating whether historical-spec content should be preserved or superseded | REMEDIATED — STORY-158/162 historical-spec preservation adjudication added; AC clarifies preserve-vs-supersede disposition for each reference |
| F-W86S-P8-013 | [process-gap] | STATE.md | Stale drift-row wording — DRIFT-docstring-scan row cited scrub sites as `:3,6` instead of `:3,:5,:6,:125` after scrub list grew in v1.5/v1.8 | REMEDIATED — DRIFT-docstring-scan row updated to `:3,:5,:6,:125` in state-manager D-524 burst |

---

## Orchestrator Rulings

### F-P8-001 Ruling — F-009 Discriminator Restated (D-524)

The dissect capture exclusion basis is **POSITIVE EVIDENCE OF UPSTREAM-OF-ITI ORIGIN**, not the absence of per-file provenance:

1. **Filename evidence:** `TestDissectIec104.pcap` — the `TestDissect` prefix is a Wireshark naming convention for dissector test captures, indicating Wireshark origin.
2. **E2E-PCAPS.md description:** The file is described as "Wireshark-dissector test capture", confirming non-ITI origin.

The diverse capture (`iec104-iti-diverse.pcap`) was committed because:
- No contrary indication of third-party upstream origin
- Repo-level CC-BY-4.0 license suffices absent contrary indication

Rule: **"repo-level license suffices absent contrary indication of third-party upstream origin."** (D-524, restating D-523 F-009)

---

## Convergence Assessment

| Metric | Value |
|--------|-------|
| Pass | 8 |
| Findings this pass | 12 (severity-counted) + 1 [process-gap] |
| Previous pass (P7) | 14 |
| Delta | -2 severity-counted |
| STORY-183 assessment | Materially converged — adversary identified only 2 MED table gaps |
| STORY-182 state | 3 HIGH residue (single-capture-ruling + discriminator); all REMEDIATED v1.8 |
| Clean streak | 0 of 3 (pass-8 not clean; 3H present) |
| Next pass | 9 |
