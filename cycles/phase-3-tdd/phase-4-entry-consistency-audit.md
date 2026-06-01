---
document_type: consistency-report
level: ops
version: "1.0"
status: final
producer: consistency-validator
timestamp: 2026-05-31T00:00:00Z
phase: phase-3-to-4-gate
traces_to: .factory/STATE.md
audit_scope: phase-3-tdd-complete
develop_head: 6158e6e
---

# Phase 3→4 Entry Consistency Audit

**Auditor:** consistency-validator (fresh-context, independent pass)
**Date:** 2026-05-31
**Scope:** Cross-document perimeter validation — Phase 3 complete (48/48 stories,
27/27 waves, develop @ 6158e6e). Validates correctness of the spec-test-BC perimeter
as a whole; the per-story adversary validated within each story's boundary.

---

## Executive Summary

| Area | Verdict | Blocking? |
|------|---------|-----------|
| 1. BC Coverage Completeness | CONSISTENT | No |
| 2. Story↔BC Traceability | MOSTLY CONSISTENT (1 drift) | No — cosmetic |
| 3. Spec Alignment (PRD↔arch↔domain↔stories) | CONSISTENT | No |
| 4. VP Coverage and Classification | CONSISTENT | No |
| 5. Holdout Scenario Semantic Validity (CRITICAL gate) | CONSISTENT | No |
| 6. Index/Registry Integrity | MOSTLY CONSISTENT (1 index drift) | No — cosmetic |

**BOTTOM LINE: READY TO ENTER PHASE 4.** No blocking defects. Two cosmetic drift items
documented below (STORY-053 EC-004 body text, STORY-INDEX delivery progress table
incompleteness). Neither affects Phase 4 execution. Correction recommended before Phase 5.

---

## Area 1: BC Coverage Completeness

### Check: 217 BCs × story assignment × test formalization

**BC File Count:** 217 BC files across 12 subsystems (ss-01 through ss-13, ss-03 absent
by design).

| Subsystem | BC Count (files) | BC Count (index) | Match |
|-----------|-----------------|-----------------|-------|
| ss-01 | 8 | 8 | PASS |
| ss-02 | 15 | 15 | PASS |
| ss-04 | 54 | 54 | PASS |
| ss-05 | 9 | 9 | PASS |
| ss-06 | 26 | 26 | PASS |
| ss-07 | 37 | 37 | PASS |
| ss-08 | 4 | 4 | PASS |
| ss-09 | 6 | 6 | PASS |
| ss-10 | 9 | 9 | PASS |
| ss-11 | 24 | 24 | PASS |
| ss-12 | 21 | 21 | PASS |
| ss-13 | 4 | 4 | PASS |
| **TOTAL** | **217** | **217** | **PASS** |

**BC-INDEX derivation:** 218 draft ingestion BCs − 6 retired (BC-ABS-004..009) = 212 active
from ingestion + 5 post-ingestion CsvReporter additions (BC-2.11.020..024) = 217 active.
Arithmetic verified correct.

**BC-to-Story Assignment:** dependency-graph.md reports "217 / 217 BCs assigned across 48
stories" at line 491 and 560. STORY-INDEX Coverage Verification confirms "All 217 BCs
assigned: Yes (per dependency-graph.md BC to Stories Matrix)".

**Test Formalization Coverage:** All 48 stories delivered per code-delivery/ directories and
git log. Test files confirmed present for all story groups:

| Story Group | Test File | Test Count |
|-------------|-----------|-----------|
| STORY-001 (E-1) | bc_2_01_story001_tests.rs | present |
| STORY-002..005 (E-1) | bc_2_02_story00[2-5]_tests.rs | present |
| STORY-011..021 (E-2) | reassembly_*_tests.rs | present |
| STORY-031..033 (E-3) | dispatcher_tests.rs | present |
| STORY-041..046 (E-4) | http_analyzer_tests.rs | present |
| STORY-051..058 (E-5) | tls_analyzer_tests.rs (226 tests) | present |
| STORY-066 (E-6) | dns_tests.rs | present |
| STORY-069..071 (E-7) | findings_tests.rs, mitre_tests.rs | present |
| STORY-076..080 (E-8) | reporter_json_tests.rs (26), reporter_terminal_tests.rs (60), reporter_csv_tests.rs (50) | present |
| STORY-086..090, 096 (E-9/E-10) | cli_story_086/087/096_tests.rs, main_story_088/089_tests.rs, summary_story_090_tests.rs | present |

**Verdict: CONSISTENT.** All 217 BCs assigned, 217 BC files on disk matching index.
All 48 stories have test file formalization.

---

## Area 2: Story↔BC Traceability

### Check: frontmatter behavioral_contracts ↔ body BC table ↔ AC traces ↔ test names

The following spot-checks were performed on the three BCs specifically called out in the
audit mandate (Wave-18 corrections):

**BC-2.07.002 (ServerHello parsing):**
- Assigned to: STORY-053 (frontmatter line 18, body BC table line 55, AC-001..007 traces confirmed)
- Test file: tls_analyzer_tests.rs — test_BC_2_07_002_* pattern confirmed (8 tests)
- PASS on traceability chain

**BC-2.07.012 (deprecated server protocol):**
- Assigned to: STORY-054 (frontmatter line 26, body BC table line 58, AC-009..010 traces confirmed)
- Test file: tls_analyzer_tests.rs — EC-001/002/003/004/005 tests confirmed
- PASS on traceability chain

**BC-2.07.029 (bad TLS record body):**
- Assigned to: STORY-058 (frontmatter line 25, body line 57 BC table, AC-007/008 traces confirmed)
- Test file: tls_analyzer_tests.rs — test_parse_error_counter confirmed
- PASS on traceability chain

### DRIFT FINDING D-001 (MINOR, non-blocking)

**File:** `.factory/stories/STORY-053.md:100`
**Finding:** STORY-053 EC-004 body text reads "ServerHello version = 0x0200 (SSL 2.0) |
`Anomaly/Likely/High` deprecated-protocol finding emitted (see STORY-054)". This is the
PRE-CORRECTION behavior. The current corrected spec in BC-2.07.002 v1.3 EC-004 states the
OPPOSITE: tls-parser 0.12 rejects a 0x0200 ServerHello at the record layer; parse_errors
increments; handle_server_hello is never reached; NO finding is produced.

**Severity:** Minor/cosmetic. The test is correct — `test_BC_2_07_002_ec004_ssl2_version_parse_behavior_pinned`
(tls_analyzer_tests.rs:5227) pins the post-correction behavior and asserts parse_errors==1,
version_counts[0x0200]==0, no deprecated-protocol finding. The story's EC-004 body text was
not updated when BC-2.07.002 v1.3 was committed on 2026-05-29 (F-S054-P6-001).

**Impact on Phase 4:** NONE. The holdout evaluator reads BC files and tests, not story body EC
tables. HS-059, HS-071, HS-074 all use SSL 3.0 (0x0300), not SSL 2.0. No scenario exercises
the stale EC-004 text.

**Remediation:** Update STORY-053.md:100 EC-004 expected behavior to match BC-2.07.002 v1.3
EC-004 before Phase 5 adversarial review.

---

## Area 3: Spec Alignment

### PRD ↔ Architecture ↔ Domain Spec ↔ Stories

**Subsystem registry consistency:**
BC-INDEX subsystem labels, ARCH-INDEX Subsystem Registry, and BC frontmatter `subsystem:`
fields cross-checked for SS-07, SS-11, SS-12. All match canonical ARCH-INDEX names.

**BC-2.07.002 v1.3 correction (F-S054-P1-002, 2026-05-29):**
The EC-004 SSL2-ServerHello-rejection correction was propagated to:
- BC-2.07.002.md (version 1.3, modified field confirmed)
- BC-2.07.012.md (version 1.4, EC-004/EC-005 documented, upgrade guard added)
- BC-2.07.029.md (version 1.3, invariant-2 arithmetic corrected to parse_errors − truncated_records)
- Tests: pin tests committed in tls_analyzer_tests.rs
- NOT propagated to STORY-053.md EC-004 body text (see D-001 above)

**VP-018/019 proof_method correction (F-W21-VP-METHOD, 2026-05-31):**
Corrected from `manual` to `integration` (VP-018) and `unit` (VP-019) in:
- vp-018-cli-reassemble-mutual-exclusion.md (v1.1 confirmed, modified field present)
- vp-019-dns-statistics-only.md (v1.1 confirmed, modified field present)
- VP-INDEX.md (integration/unit columns confirmed at rows VP-018, VP-019)
- verification-coverage-matrix.md (cli.rs row shows integration; analyzer/dns.rs row shows unit)
- verification-architecture.md (Test Sufficient table rows VP-018/019 confirmed)

All three arch documents in sync. PASS.

**SS-11 reporter BC re-anchoring (DF-SIBLING-SWEEP-001, 2026-05-30):**
VP-017 frontmatter proof_method corrected manual→integration. VP-INDEX integration/unit
count = 5, matching verification-coverage-matrix.md Totals row integration/unit = 5.
BC-2.11.022, BC-2.11.023 unit/proptest→unit corrections propagated.

**CONSISTENT** across all three correction chains examined.

---

## Area 4: VP Coverage

### Check: 20 VPs, correct classification, formalization path into Phase 6

**VP-INDEX arithmetic:**

| Count | Claimed | Verified |
|-------|---------|---------|
| Total VPs | 20 | 20 (file count confirmed) |
| Kani | 8 | 8 (VP-001..005, VP-007, VP-009, VP-015) |
| proptest | 6 | 6 (VP-006, VP-010..014) |
| cargo-fuzz | 1 | 1 (VP-008) |
| integration/unit | 5 | 5 (VP-016..020) |
| Sum | 20 | 20 |
| P0 | 8 | 8 |
| P1 | 7 | 7 |
| test-sufficient | 5 | 5 |
| Sum | 20 | 20 |

VP-INDEX total (20) == verification-architecture.md row count (20) == verification-coverage-matrix.md
VP row count (20). PASS (Criterion 78).

**verification-coverage-matrix.md Totals row:** Kani(8) + proptest(6) + fuzz(1) + integration/unit(5) = 20.
Totals row verified correct (line 56 of verification-coverage-matrix.md). PASS (Criterion 80).

**VP-INDEX → verification-architecture.md completeness:** All 20 VPs appear in verification-architecture.md
across the three catalog sections (Must Prove: VP-001..009, Should Prove: VP-010..015, Test Sufficient:
VP-016..020). Module and tool assignments match VP-INDEX. PASS (Criterion 79).

**Phase 6 formalization path:**
- P0 VPs (VP-001..009): Kani/fuzz harnesses to be written in Phase 6. All feasibility assessments
  state "feasible". No blocking issue.
- P1 VPs (VP-010..015): proptest/Kani harnesses in Phase 6.
- Test-sufficient VPs (VP-016..020): verification by existing or planned integration/unit tests.
  VP-018 (CLI mutual exclusion) and VP-019 (DNS never-emit) are test-sufficient; the test specification
  code in the VP files is coherent with the story ACs.

**Verdict: CONSISTENT.** All VP counts verified. Tool assignments and phase gates are correct.
No VP lacks a documented formalization path.

---

## Area 5: Holdout Scenario Semantic Validity (CRITICAL GATE CHECK)

### Check: HS-* scenarios must NOT assert pre-correction behavior for Wave-18 BC fixes

This is the single most important Phase-4 entry check per the audit mandate.

**Three correction vectors assessed:**

#### 5a. BC-2.07.002 EC-004 / BC-2.07.012 EC-004/005 — SSL2-ServerHello parse rejection

**Pre-correction claim:** A ServerHello with version 0x0200 emits an Anomaly/Likely/High
deprecated-protocol finding.

**Post-correction truth:** tls-parser 0.12 rejects the 0x0200 ServerHello at the record
layer; parse_errors++; no finding emitted.

**Holdout scenarios referencing BC-2.07.002:**
- HS-071 (BC-2.07.002, BC-2.07.003): Tests ClientHello 0x0301 + ServerHello 0x0303 version
  independence. Verification step 7 asserts "findings contains zero deprecated-protocol
  findings (0x0301 and 0x0303 are both above the threshold)". No SSL 2.0 assertion. SAFE.

**Holdout scenarios referencing BC-2.07.012:**
- HS-059 (BC-2.07.009, BC-2.07.010, BC-2.07.011, BC-2.07.012, BC-2.07.030, BC-2.07.036):
  Tests SSL 3.0 (0x0300) ClientHello + ServerHello. Asserts exactly 4 findings. All test
  cases use 0x0300, not 0x0200. No assertion of behavior for SSL 2.0 ServerHello. SAFE.
- HS-074 (BC-2.07.011, BC-2.07.012, BC-2.07.009, BC-2.07.010): Real-world SSL 3.0 pcap.
  Asserts findings for SSL 3.0 ClientHello direction=ClientToServer and ServerHello
  direction=ServerToClient. Uses 0x0300 only. No SSL 2.0 assertion. SAFE.

**Search for SSL 2.0 / 0x0200 in all 100 HS files:** No occurrence found in scenario body
or verification steps of any holdout scenario. PASS.

#### 5b. BC-2.07.029 invariant-2 arithmetic — parse_errors − truncated_records

**Pre-correction invariant-2:** stated incorrectly.
**Post-correction invariant-2:** genuine-parse-failure count = parse_errors − truncated_records;
truncated_records counts only oversized-record DoS-protection drops.

**Holdout scenarios referencing BC-2.07.029:**
- HS-062 (BC-2.07.004, BC-2.07.005, BC-2.07.029, BC-2.07.033, BC-2.07.035):
  Verification steps 3-4 assert `parse_errors >= 1` and `parse_errors == truncated_records`
  for the oversized record case. Step 6 asserts parse_errors NOT incremented for
  non-handshake records.
  
  Analysis: The scenario tests the COMPOUND case (oversized record → both counters
  increment together). It does NOT test the case that contradicted invariant-2
  (non-oversized parse failures). The scenario is testing the relationship between
  parse_errors and truncated_records for a specific scenario where parse_errors ==
  truncated_records holds validly (there is exactly one oversized record and no
  genuine nom parse errors). This does not conflict with the corrected invariant-2
  arithmetic. SAFE.

**Verdict: CONSISTENT.** No holdout scenario asserts pre-correction behavior. The
corrections to BC-2.07.002 EC-004, BC-2.07.012 EC-004/005, and BC-2.07.029 invariant-2
are fully reflected in the test suite but are NOT tested by any holdout scenario —
correct by design (holdouts test observable-from-outside behavior, not internal parse-layer
mechanics).

---

## Area 6: Index and Registry Integrity

### Check: STORY-INDEX count vs files, BC-INDEX vs files, HS-INDEX vs files, VP-INDEX vs files

**STORY-INDEX:**
- Declared total: 48 stories, 27 waves, 282 points
- Actual story file count: 48 (confirmed by BC-to-story matrix)
- All 10 epics present: PASS
- Wave count: 27: PASS

### DRIFT FINDING D-002 (MINOR, non-blocking)

**File:** `.factory/stories/STORY-INDEX.md:67-71` (Index Table) and `:136-154` (Wave Delivery Progress)

**Finding — Part A (story status fields):** STORY-057, STORY-076, STORY-077, STORY-078,
STORY-079, STORY-080 are listed with `status: draft` in both the STORY-INDEX table and
their individual story files (frontmatter `status: draft`). However, all six stories were
demonstrably delivered and closed:
  - STORY-057: git commit 616897e (PR #156, 2026-05-29); wave-history.md Wave 19 CLOSED
  - STORY-076: git commit e5cb2b1 (PR #157, 2026-05-29); wave-history.md Wave 20 CLOSED
  - STORY-077: git commit 594567c (PR #158, 2026-05-30); wave-history.md Wave 21 CLOSED
  - STORY-078: git commit bf16c0b (PR #160, 2026-05-30); wave-history.md Wave 22 CLOSED
  - STORY-079: git commit 41ab24d (PR #159, 2026-05-30); wave-history.md Wave 21 CLOSED
  - STORY-080: git commit 1ecf114 (PR #161, 2026-05-30); wave-history.md Wave 22 CLOSED

**Finding — Part B (Wave Delivery Progress table):** The Wave Delivery Progress table in
STORY-INDEX.md only records waves 1, 2, and 23-27. Waves 3-22 (41 of 48 stories) are
absent from the delivery progress table. The wave-history.md file in
`.factory/cycles/phase-3-tdd/` contains the authoritative per-wave delivery detail; the
STORY-INDEX table was not updated to match.

**Severity:** Minor/cosmetic. STATE.md is the authoritative pipeline state document and
correctly records all 48 stories as delivered and all 27 waves as CLOSED. The wave-history.md
confirms waves 3-22 CLOSED. The STORY-INDEX status fields and Wave Delivery Progress table
are non-authoritative tracking aids that were not kept current — a known process pattern in
brownfield-formalization mode where the per-story `status: draft` meant "not yet dispatched"
rather than tracking post-delivery completion.

**Impact on Phase 4:** NONE. The holdout evaluator uses the HS-INDEX and the BC files,
not STORY-INDEX story status fields.

**Remediation:** Before Phase 5, update STORY-INDEX.md:
  1. Change `status: draft` to `status: completed` for STORY-057, 076, 077, 078, 079, 080
     in the Index Table.
  2. Update the Wave Delivery Progress table to add wave rows 3-22.
  Also update individual story file `status:` frontmatter field for all six stories.

**BC-INDEX:** 217 entries, all `[WRITTEN]`, arithmetic confirmed. PASS.

**HS-INDEX:**
- Declared: 100 scenarios (HS-001..HS-100)
- Actual files: 100 (ls count minus evaluations/ minus HS-INDEX.md minus wave-scenarios/ = 100)
  Verified: sequential 001-100, no gaps, no duplicates
- must_pass: 99, should_pass: 1 (HS-025)
- All 27 waves covered: PASS
- PRD scenario count (100): matches HS-INDEX. PASS.

**VP-INDEX:** 20 VPs (vp-001..vp-020 files confirmed). Matches VP-INDEX declaration. PASS.

**ARCH-INDEX:** All 9 architecture section files present. BC count column matches actual
BC file counts per subsystem. PASS.

**PRD Supplements:** All 4 required files present: interface-definitions.md,
error-taxonomy.md, test-vectors.md, nfr-catalog.md. PASS.

---

## Summary Table

| Check | Result | Severity | Blocking Phase 4? |
|-------|--------|----------|--------------------|
| BC file count: 217 on disk | PASS | — | No |
| BC-INDEX arithmetic (218-6+5=217) | PASS | — | No |
| All 217 BCs assigned to stories | PASS | — | No |
| All 48 stories have test files | PASS | — | No |
| STORY-053 EC-004 body text drift (D-001) | DRIFT | Minor | No |
| BC-2.07.002 v1.3 correction propagated to tests | PASS | — | No |
| BC-2.07.012 v1.4 correction propagated | PASS | — | No |
| BC-2.07.029 v1.3 correction propagated | PASS | — | No |
| VP-018 proof_method fix propagated (3 docs) | PASS | — | No |
| VP-019 proof_method fix propagated (3 docs) | PASS | — | No |
| VP-INDEX total arithmetic (20) | PASS | — | No |
| VP-INDEX == verification-architecture row count | PASS | — | No |
| VP-INDEX == verification-coverage-matrix row count | PASS | — | No |
| Coverage matrix Totals arithmetic | PASS | — | No |
| HS: no pre-correction SSL 2.0 assertions | PASS | CRITICAL | CLEAR |
| HS: BC-2.07.029 invariant-2 arithmetic conflict | PASS | CRITICAL | CLEAR |
| HS count: 100 scenarios (HS-001..HS-100) | PASS | — | No |
| All 27 waves covered by at least 1 scenario | PASS | — | No |
| STORY-INDEX draft status for 6 delivered stories (D-002) | DRIFT | Minor | No |
| Wave Delivery Progress table gaps 3-22 (D-002) | DRIFT | Minor | No |
| ARCH-INDEX subsystem registry BC counts | PASS | — | No |
| All 4 PRD supplements present | PASS | — | No |

---

## Validation Gate Result

**PASS — Phase 4 entry is authorized.**

### Rationale

All CRITICAL checks (holdout scenario semantic validity) are CLEAR. No holdout scenario
asserts pre-correction behavior for the Wave-18 BC corrections. The 100 holdout scenarios
correctly target post-correction semantics (SSL 3.0 triggering, not SSL 2.0 parse failures;
BC-2.07.029 invariant tested in compound-case only). BC, VP, and architecture index counts
are fully consistent across all three documents that must stay synchronized. All 48 stories
are demonstrably delivered with test formalization; the only "draft" label defect is a
cosmetic tracking omission.

### Deferred to Phase 5 (non-blocking)

| Item | Location | Action |
|------|----------|--------|
| D-001: STORY-053 EC-004 body text | `.factory/stories/STORY-053.md:100` | Update EC-004 expected behavior to match BC-2.07.002 v1.3 |
| D-002a: 6 story files `status: draft` | 6 story files + STORY-INDEX.md table | Change to `status: completed` for STORY-057/076/077/078/079/080 |
| D-002b: Wave Delivery Progress gaps | `.factory/stories/STORY-INDEX.md:136-154` | Add wave rows 3-22 from wave-history.md |

### Open Drift Item from STATE.md (inherited, not gated)

| ID | Finding | Phase Gate |
|----|---------|-----------|
| F-W25-S088-P6-001 | [LOW] AC-004 warning-once assertion coverage gap (BC-2.12.009 inv-2 not assertion-covered) | Accept before Phase 5 or add one-line count-assertion |

---

## Consistency Score

Items checked: 22
PASS: 19
DRIFT (minor, non-blocking): 3 (D-001, D-002a, D-002b)
FAIL (blocking): 0

**Consistency score: 19/22 = 86% (all failures are cosmetic tracking omissions, not
behavioral or traceability defects)**

The 14% non-perfect score is entirely attributable to story status field housekeeping
not being updated during the high-velocity Wave 19-22 delivery period — a known artifact
of the brownfield-formalization mode where `status: draft` was the initial state and
per-story completion updates were not scripted. The authoritative delivery record
(STATE.md + wave-history.md + git log) is complete and consistent.
