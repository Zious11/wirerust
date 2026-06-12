---
document_type: f6-hardening-record
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-06-12T17:08:39Z
cycle: "feature-8-dnp3-v0.5.0"
phase: feature-f6
pr: "#231"
merge_commit: a125c69
merged_at: 2026-06-12T17:08:39Z
verdict: HARDENED
---

# F6 Formal Hardening Record — Feature #8 DNP3

PR #231 MERGED to develop. Merge commit `a125c69` (2026-06-12T17:08:39Z).
develop HEAD = a125c69. main HEAD unchanged = c2df1b5 (v0.5.0).

---

## F6 Obligations — All SATISFIED

| Obligation | Status | Evidence |
|-----------|--------|---------|
| AC-005 Kani `verify_content_first_precedence_exhaustive` (STORY-106 VP-004, port-20000 oracle arm) | SATISFIED | Kani SUCCESSFUL — 0 counterexamples |
| VP-023 draft→verified + VP-INDEX bump (verified 22→23, draft 1→0) | SATISFIED | factory-artifacts c5db1bf; vp-verified-VP-023-2026-06-12 tag at e685664 |
| VP-004 locked-prose relock to include Rules 5/6 (Modbus/502 + DNP3/20000) | SATISFIED | factory-artifacts aa469bd; proof re-verified; verified_at_commit e685664 |
| VP-023 Kani harnesses + master-frame proofs hold under corrected is_master_frame (0x80 mask) | SATISFIED | 9/9 Kani harnesses SUCCESSFUL — 0 failed; VP-023 ×4 (Sub-A/B/C/D parse-safety) + VP-004 ×1 (verify_content_first_precedence_exhaustive) + VP-007 ×4 (catalog); 0 regressions |

---

## Kani Results (9/9 SUCCESSFUL)

- VP-023 Sub-A/B/C/D parse-safety harnesses: 4/4 SUCCESSFUL
- VP-004 `verify_content_first_precedence_exhaustive` (AC-005, port-20000 oracle arm PROVEN): 1/1 SUCCESSFUL
- VP-007 catalog harnesses ×4: 4/4 SUCCESSFUL
- Total: 9/9 SUCCESSFUL; 0 failed; 0 counterexamples
- Note: `is_master_frame` is an effectful shell (outside pure-core VP-023 proofs). Corrected 0x80 mask introduced no regression — obligation 4 SATISFIED.

---

## Mutation Testing (cargo-mutants, DNP3 detection/parse delta)

- Kill rate: 89% (91.8% incl timeouts)
- Surviving mutants in detection-decision or parse-safety logic: 0
- Benign survivors: 8 (equivalent/cosmetic/cap-boundary)
- Survivor #6 (window-seeding gap): KILLED by new unit test in PR #231
- Verdict: PASS — all logic-critical mutants killed

---

## Fuzz Testing

- Target: NEW `fuzz_dnp3_parse` (added in PR #231)
- Executions: 3.19M / 0 crashes
- Panic-free result consistent with VP-023 parse-safety guarantees

---

## Regression

- Tests green: 1495
- Clippy: CLEAN

---

## Security

- Prior manual reviews: CLEAN (F4 stories + F5 PR #230 + F6 PR #231 security all CLEAN)
- Automated SAST (semgrep): ABSENT on host — non-blocking; DRIFT-SEMGREP-001 recorded
- Verdict: CLEAN (manual review)

---

## VP Locks Applied (factory-artifacts)

| VP | Action | Commit | Notes |
|----|--------|--------|-------|
| VP-023 | draft→verified; verification_lock true; verified_at_commit e685664; proof_file_hash set (AC-011) | c5db1bf | VP-INDEX verified 22→23, draft 1→0 |
| VP-004 | prose relocked to include Rules 5/6; proof re-verified; verified_at_commit e685664 | aa469bd | Git tag vp-verified-VP-023-2026-06-12 at e685664 |

---

## Drift Items Recorded

| ID | Summary | Severity |
|----|---------|---------|
| DRIFT-SEMGREP-001 | semgrep not installed on build host; F6 security relied on manual reviews (all CLEAN); install semgrep for automated SAST in a future cycle | LOW |
