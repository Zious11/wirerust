---
document_type: adversarial-convergence
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-28T00:00:00Z
cycle: v0.1.0-greenfield-spec
wave: 16
traces_to: STATE.md
stories: [STORY-042, STORY-043, STORY-044, STORY-052]
---

# Wave 16 Adversarial Convergence Detail

Stories: STORY-042 (URI-based threat detections), STORY-043 (header/method anomaly detections),
STORY-044 (parse-error isolation + poisoning state machine), STORY-052 (ClientHello parsing — TLS epic E-5).

PRs merged (pre-convergence): STORY-042 PR #140 (ca5ea1c), STORY-052 PR #141 (80efb79),
STORY-043 PR #142 (7eef78d), STORY-044 PR #143 (0352aba).

BC-5.39.001 gate: 3 consecutive clean per-story passes required per story. Wave closes when all
4 stories achieve 3-clean streak simultaneously.

---

## Pass-1 (Retroactive — 2026-05-28)

**develop HEAD at pass:** 4aed2a7 (PR #144 — test-quality fixes)

| Story | Findings | Verdict | Remediation |
|-------|----------|---------|-------------|
| STORY-042 | 0 | CLEAN (streak=1) | None required |
| STORY-052 | 0 | CLEAN (streak=1) | None required |
| STORY-043 | 2 (MEDIUM: BC-2.07.003 done-check citation; LOW: prose) | DIRTY (streak reset) | BC-2.07.003 v1.3 + STORY-043 v1.1 (factory-only) |
| STORY-044 | 2 (HIGH F-W16-S044-P1-001: all 13 AC test citations not BC-prefixed; MEDIUM: BC-2.06.005 backslash prose) | DIRTY (streak reset) | PR #144 → 4aed2a7 (test fixes) + BC-2.06.005 v1.3 + STORY-044 v1.3 (factory-only) |

Pass-1 remediation: PR #144 → 4aed2a7 merged to develop. Factory-only: BC-2.07.003 v1.3, BC-2.06.005 v1.3, STORY-043 v1.1, STORY-044 v1.3.

---

## Pass-2 (Retroactive — 2026-05-28)

**develop HEAD at pass:** 4aed2a7 (unchanged from Pass-1)

| Story | Streak Before | Findings | Verdict | Streak After | Remediation |
|-------|--------------|----------|---------|-------------|-------------|
| STORY-042 | 1 | 0 | CLEAN | 2 | None |
| STORY-043 | 0 (post-P1-remediation) | 0 | CLEAN | 1 | None |
| STORY-044 | 0 (post-P1-remediation) | 0 | CLEAN | 1 | None |
| STORY-052 | 1 | 2 (MEDIUM F-W16-S052-P2-001: BC-2.07.032 VP table missing discriminating unit test for EC-001; LOW anchor sweep: 6 BCs stale) | DIRTY | 0 | Factory-only: BC-2.07.032 v1.3, BC-2.07.001 v1.3, BC-2.06.005 v1.4, BC-2.06.007 v1.3, BC-2.06.015 v1.3, STORY-052 v1.3, STORY-044 v1.4, STORY-042 v1.2 |

Pass-2 remediation: factory-only burst; develop_head unchanged 4aed2a7. No develop PR.

Drift item filed: F-W16-S052-P2-002 [coverage-gap, LOW, DEFERRED] — BC-2.07.001 EC-002 extension-block
parse failure path (src/analyzer/tls.rs:391-396 inner Err arm) has no discriminating test; no AC enumerates
it. Requires research-agent validation per DF-VALIDATION-001 before any GitHub issue is filed.

---

## Pass-3 (Retroactive — 2026-05-28)

**develop HEAD at pass start:** 4aed2a7

| Story | Streak Before | Findings | Verdict | Streak After | Remediation |
|-------|--------------|----------|---------|-------------|-------------|
| STORY-052 | 0 (post-P2-remediation) | 0 | CLEAN | 1 | None |
| STORY-042 | 2 | (pending) | — | — | — |
| STORY-043 | 1 | (pending) | — | — | — |
| STORY-044 | 1 | (pending) | — | — | — |

**Pass-3 findings requiring remediation (this burst):**

**F-W16-S044-P3-001** [MEDIUM, process-gap] — STORY-044 Architecture Mapping table body (line 124)
cites "Poison transition: src/analyzer/http.rs:467 (resp)" but BC-2.06.015 v1.3 anchor was tightened
to 467-468 in the Pass-2 burst; the STORY-044 consuming body was not swept in that same burst.
Instance of DF-SIBLING-SWEEP-001 v3 where BC-edit sibling-sweep should extend to consuming-story
bodies citing the same source anchor.
Remediation: STORY-044 v1.4→v1.5: Architecture Mapping anchor corrected to `467-468 (resp)`.

**F-W16-S044-P3-002** [MEDIUM] — BC-2.06.017 invariants cite
"request_poisoned early-return (http.rs:509-511)" and "response_poisoned early-return (http.rs:521-523)"
but actual guard spans one extra line each: 509-512 and 521-524.
Remediation: BC-2.06.017 v1.2→v1.3: invariants 1-2 anchor precision fix.

**F-W16-S044-P3-003** [LOW] — AC-013 Test citation lacked the response-arm sibling test
`test_BC_2_06_020_invariant_real_too_many_headers_after_success_suppressed_response`
(http_analyzer_tests.rs:4488) for symmetric BC-2.06.020 invariant 3 coverage.
Remediation: STORY-044 v1.5: AC-013 Test citation extended with response-arm sibling.

Pass-3 remediation vehicles:
- Track B (develop): PR #145 → 16d938d (test-quality fixes for STORY-042/043)
- Track A (factory): BC-2.06.017 v1.3 + STORY-044 v1.5 (this burst)

**Process-gap note (F-W16-S044-P3-001):** The P2 burst tightened BC-2.06.015's anchor to 467-468
but did not sweep the consuming STORY-044 Architecture Mapping body (line 124) in the same burst.
This is a DF-SIBLING-SWEEP-001 v3 instance where BC-edit sibling-sweep should extend to consuming-story
bodies that cite the same source anchor. Flagged for codification follow-up at wave-close.

---

## Consecutive-Clean Streak Status (post-Pass-3 remediation)

| Story | Pass-1 | Pass-2 | Pass-3 | Streak (post-P3-rem) | Gate Status |
|-------|--------|--------|--------|---------------------|-------------|
| STORY-042 | CLEAN | CLEAN | DIRTY→remediated | 0 (streak reset by P3 Track B fix) | NOT YET — needs 3 clean |
| STORY-043 | DIRTY | CLEAN | DIRTY→remediated | 0 (streak reset by P3 Track B fix) | NOT YET — needs 3 clean |
| STORY-044 | DIRTY | CLEAN | DIRTY→remediated | 0 (streak reset by P3 remediation) | NOT YET — needs 3 clean |
| STORY-052 | CLEAN | DIRTY | CLEAN (streak=1) | 1 | IN PROGRESS — needs 2 more clean |

**Note:** STORY-042 and STORY-043 had Track B develop fixes in PR #145 which reset their streaks.
All 4 stories require additional clean passes for BC-5.39.001 gate. Wave 16 remains OPEN.

---

## Pass-4 (Retroactive — 2026-05-28)

**develop HEAD at pass:** 16d938d (unchanged from Pass-3)

| Story | Streak Before | Findings | Verdict | Streak After | Remediation |
|-------|--------------|----------|---------|-------------|-------------|
| STORY-052 | 1 | 0 | CLEAN | 2 | None |
| STORY-042 | 0 | 0 | CLEAN | 1 | None |
| STORY-043 | 0 | 0 | CLEAN | 1 | None |
| STORY-044 | 0 | 2 (MEDIUM F-W16-S042-P4-001: BC-2.06.005 wrong brace-prose at inv-1 line 191; MEDIUM F-W16-S052-P4-002: BC-2.07.001 missing VP rows for invariant-2 capacity tests) | DIRTY | 0 | Factory-only: BC-2.06.005 v1.5, BC-2.07.001 v1.4, STORY-042 v1.3 (input-hash 7f9b0ab→60e0389), STORY-052 v1.4 (input-hash 09f5faa→39b997a), STORY-044 v1.6 (line anchor 3868→3888 + finding-ID label corrections) |

**Finding details:**

**F-W16-S042-P4-001** [MEDIUM] — BC-2.06.005 v1.4 invariant 1 contained factually wrong prose claiming
"line 191 is the closing brace". Actual code: line 191 is the opening `{` of the if-body; the closing `}`
is at line 203. The four `.contains()` calls span lines 187-190 only.
Remediation: BC-2.06.005 v1.4→v1.5: corrected invariant 1 prose to "line 191 is the opening `{` of the
if-body (the closing `}` is at line 203)". Side-effect: STORY-042 input-hash updated (BC-2.06.005 is in
STORY-042's `inputs:` list).

**F-W16-S052-P4-002** [MEDIUM] — BC-2.07.001 v1.3 Verification Properties table was missing VP rows for
invariant 2 (capacity bound on `version_counts` and `ja3_counts`). The discriminating tests
`test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries` (tests/tls_analyzer_tests.rs:2747)
and `test_BC_2_07_001_inv2_ja3_counts_bounded_at_max_map_entries` (tests/tls_analyzer_tests.rs:2811)
existed in the codebase but were not cited in the VP table.
Remediation: BC-2.07.001 v1.3→v1.4: two VP rows added citing the discriminating tests. Side-effect:
STORY-052 input-hash updated (BC-2.07.001 is in STORY-052's `inputs:` list).

Pass-4 remediation: factory-only burst; develop_head unchanged 16d938d. No develop PR.

**Convergence-policy decision recorded:** Pass-5+ only MEDIUM+ findings trigger remediation. LOW nits
(observations, cosmetic items) ride without remediation. A CLEAN verdict with only LOW nits sustains the
per-story consecutive-clean streak per BC-5.39.001 project convention (established in W14 "NITPICK_ONLY"
passes).

---

## Consecutive-Clean Streak Status (post-Pass-4 remediation)

| Story | Pass-1 | Pass-2 | Pass-3 | Pass-4 | Streak (post-P4-rem) | Gate Status |
|-------|--------|--------|--------|--------|---------------------|-------------|
| STORY-042 | CLEAN | CLEAN | DIRTY→rem | CLEAN | 1 | IN PROGRESS — needs 2 more clean |
| STORY-043 | DIRTY | CLEAN | DIRTY→rem | CLEAN | 1 | IN PROGRESS — needs 2 more clean |
| STORY-044 | DIRTY | CLEAN | DIRTY→rem | DIRTY→rem | 0 | NOT YET — needs 3 clean |
| STORY-052 | CLEAN | DIRTY | CLEAN | CLEAN | 2 | IN PROGRESS — needs 1 more clean |

Wave 16 remains OPEN. BC-5.39.001 NOT YET ACHIEVED.

---

## Drift Items Filed This Wave

| ID | Category | Severity | Status | Description |
|----|----------|----------|--------|-------------|
| F-W16-S052-P2-002 | coverage-gap | LOW | DEFERRED | BC-2.07.001 EC-002 extension-block parse failure path has no discriminating test. Requires research-agent validation per DF-VALIDATION-001. Target: future TLS hardening story. |
| F-W16-S043-P3-002 | coverage-gap | LOW | DEFERRED | BC-2.06.010 invariant 2 "truncate_uri UTF-8 char-boundary safe" claim never exercised; all long-URI test inputs are pure ASCII. Feasibility unclear — httparse may reject non-ASCII URI tokens at this layer. Requires research-agent validation per DF-VALIDATION-001. Target: future TLS/HTTP hardening story. |
