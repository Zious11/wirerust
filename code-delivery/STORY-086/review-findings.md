# Review Findings — STORY-086

## Convergence Table

| Cycle | Total Findings | Blocking | Fixed | Remaining | Status |
|-------|---------------|----------|-------|-----------|--------|
| 1 | 2 | 0 | 0 | 2 (non-blocking) | APPROVE |

## Cycle 1 — PR Review Triage

**Reviewer:** pr-review-triage
**Date:** 2026-05-31
**PR:** #163
**Verdict:** APPROVE (0 blocking findings)

### Finding Register

| ID | Severity | Category | Finding | Blocking | Route | Disposition |
|----|----------|----------|---------|----------|-------|-------------|
| R1-001 | Low | coverage | AC-002 `--http --tls` sub-block omits `mitre=false` assertion. The `mitre` field is silently dropped via `..` destructuring in that sub-case, so a mutation that sets `mitre=true` would not be caught here. | No | None | Pre-documented as ADV-P2-001 in adversarial register. `mitre=false` is explicitly asserted in the --dns-only sub-case (same test), AC-004, EC-002 (3 independent coverage points). Optional hardening only. |
| R1-002 | Low | coverage | `test_EC_002_mitre_alone` is logically redundant with `test_mitre_flag_does_not_imply_analyzers` (AC-004). Both test `["wirerust", "analyze", "--mitre", "cap.pcap"]` with identical assertions. | No | None | Both have distinct spec citation anchors (BC-2.12.001 invariant 3 vs EC-005). Redundancy is intentional for spec-completeness traceability. No action needed. |

### Review Checklist

- [x] Diff scope verified: test file only (`tests/cli_story_086_tests.rs`) + demo evidence. Zero `src/` changes.
- [x] All 15 tests structurally sound — parse_ok/parse_err helper pattern is clean and non-duplicated within the module.
- [x] Error kind assertions precise: `MissingRequiredArgument` (AC-003), `UnknownArgument` (AC-007, EC-003, EC-004). No string matching.
- [x] `mod story_086` namespace isolation present — DF-TEST-NAMESPACE-001 satisfied.
- [x] `#![allow(non_snake_case)]` at file level correctly covers `test_EC_*` names.
- [x] All tests have discriminating negative assertions (anti-tautology confirmed).
- [x] AC-002 has 3 distinct sub-cases (dns-only, http+tls, all-only).
- [x] AC-008 tests 3 flag positions + absent case — correct global-flag semantics coverage.
- [x] Test function names match STORY-086.md `Test:` citations exactly (DF-AC-TEST-NAME-SYNC-001).
- [x] No name collisions with existing `tests/cli_tests.rs` 14 functions.
- [x] Demo evidence: 9 GIF/WebM recordings + tape scripts. 4 pure-struct ACs correctly noted as not CLI-observable.
- [x] `_type_check_imports()` compile-time guard is a defensive pattern; `#[allow(dead_code)]` annotation present.
- [x] Adversarial register confirmed CONVERGED (3 passes, 0 Critical/High/Medium).
- [x] Dependency PR #161 (STORY-080) merged — no dependency block.
