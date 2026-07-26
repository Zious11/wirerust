---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: state-manager
timestamp: 2026-07-25T03:30:00Z
cycle: "wave-086"
pass: 6
inputs: [stories/STORY-182.md, stories/STORY-183.md, stories/STORY-INDEX.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Pass-6 Findings — wave-086

**Pass date:** 2026-07-25
**Tally:** 20 findings — 0 CRIT / 2 HIGH / 11 MED / 6 LOW / 1 NIT
**Novelty:** MEDIUM (severity decay continues; P5: 0C/3H/15M/8L/2N → P6: 0C/2H/11M/6L/1N)
**Streak after remediation:** 0/3 (not clean; reset)
**Status:** REMEDIATED — Awaiting Pass 7
**Process-gaps:** F-W86S-P6-004 [process-gap] (--exact filter sweep enforcement); F-W86S-P6-010 [process-gap] (stale-RED scrub sites adjudicated — DRIFT-stale-red-scrub)
**Non-blocking observations:** Task-4 reworded-comment fragility (advisory); bin-selftest gap — test_lint_cycle_artifact.py + test_compute_input_hash.py not in CI required-status-checks (pre-existing, attached to PG-W84-012/PG-W86-003 ops-task scope)
**PO ruling:** DF-GREEN-DOC-TENSE-SWEEP v5→v6: bare-RED tokens (RED:/RED-phase/RED reason/RED because) re-tiered TIER-1→TIER-2 with grep evidence (15/17 hits legitimate provenance; tool allowlists RED-phase: with shipped GOOD_CASE; Pattern 30 Expected RED: retained TIER-1 at 0 hits)
**Orchestrator ruling:** F-W86S-P6-002 sibling e2e harnesses (enip_e2e_real_pcaps_tests.rs + bc_2_12_011_story127_tests.rs + e2e_corpus_smoke_tests.rs) deferred wave-86 per scope containment; follow-up story candidate at next planning (DRIFT-e2e-sibling-harnesses)

---

## Finding Index

| ID | Sev | Route | Summary |
|----|-----|-------|---------|
| F-W86S-P6-001 | HIGH | STORY-182 v1.5→v1.6 | Task 1 missing Wireshark-pair curl+sha256 steps (Steps 1e+1f) for local-samples |
| F-W86S-P6-002 | HIGH [orchestrator deferral] | STORY-182 v1.5→v1.6; orchestrator ruling | Sibling e2e harnesses carry same LOCAL_SAMPLES/fixture_present silent-skip idiom — deferred wave-86 scope containment |
| F-W86S-P6-003 | MED | STORY-182 v1.5→v1.6 | assert_eq comment misleading; FIXTURE_GATED_TESTS module-level const registry absent |
| F-W86S-P6-004 | MED [process-gap] | STORY-182 v1.5→v1.6 | --exact flag and "1 passed" grep absent from single-test cargo test invocations |
| F-W86S-P6-005 | MED | STORY-183 v1.5→v1.6 | AC-183-001 incorrectly references CHANGELOG.md as one of the 13 test-file rename targets |
| F-W86S-P6-006 | MED | STORY-183 v1.5→v1.6 | Task 9 hermetic harness includes unnecessary minimal-commit step |
| F-W86S-P6-007 | MED | STORY-183 v1.5→v1.6 | FAIL output format wrong: double-space no-bracket form vs. bracket+line-number form |
| F-W86S-P6-008 | MED | STORY-183 v1.5→v1.6 | TC count in Task 13 docstring stale; should be TC1–21 (21 self-tests) |
| F-W86S-P6-009 | MED [PO propagation] | STORY-183 v1.5→v1.6; PO policy v6 | Bare RED markers (RED:/RED-phase/RED reason/RED because) remain TIER-1 in story Notes despite policy v6 re-tier |
| F-W86S-P6-010 | MED [process-gap, PO propagation] | STORY-183 v1.5→v1.6; PO policy v6 | 2 live stale RED-prose sites not adjudicated: iec104_analyzer_tests.rs:6271 + modbus_detection_tests.rs:2472/:2480 |
| F-W86S-P6-011 | MED | STORY-183 v1.5→v1.6 | Task 2 rename sweep missing :721 prose site (failure-message citing `if not rust_files:` guard) |
| F-W86S-P6-012 | MED | STORY-182 v1.5→v1.6 | input-hash NOTE does not document bash-hook divergence value (0a1812a) |
| F-W86S-P6-013 | MED | STORY-183 v1.5→v1.6 | traces_to missing CHANGELOG.md, .github/workflows/ci.yml, and bin/test_lint_cycle_artifact.py |
| F-W86S-P6-014 | LOW | STORY-182 v1.5→v1.6 | ci.yml report step lacks `if: always()` condition and placement note (after main test step) |
| F-W86S-P6-015 | LOW | STORY-182 v1.5→v1.6 | "comment-only additive step" imprecise language for an executable run step |
| F-W86S-P6-016 | LOW | STORY-182 v1.5→v1.6 | test_fixture_manifest_report() placement inside mod iec104_e2e_real_pcaps not mandated (DF-TEST-NAMESPACE-001) |
| F-W86S-P6-017 | LOW | STORY-183 v1.5→v1.6 | Task 10 ci.yml sweep missing :434 job comment + :462 step name as stale prose sites |
| F-W86S-P6-018 | LOW | STORY-183 v1.5→v1.6 | Task 5 BAD_CASES quote-escaping: :402 is GOOD_CASE, not BAD_CASE |
| F-W86S-P6-019 | LOW | STORY-183 v1.5→v1.6 | Task 13 lacks instruction for state-manager to record DRIFT-docstring-scan |
| F-W86S-P6-020 | NIT | STORY-183 v1.5→v1.6 | AC-183-007: 37 items / 36 tuples — item 5 shares tuple 4; prose must reflect accurately |

---

## Detailed Finding Records

### F-W86S-P6-001 (HIGH) — Missing Wireshark-pair curl+sha256 in Task 1

**Story:** STORY-182 (Task 1)
**Defect:** Task 1 specified fetching from CI corpus (avoid ~400 MB) but did not include explicit Steps 1e+1f to curl `iec104.pcap` and `iec104-sq.pcapng` from the Wireshark sample archive with sha256 verification into the local-samples directory. Env A claimed "4/4 Wireshark captures" but the task steps did not produce all 4 via curl+shasum.
**Remediation:** STORY-182 v1.6 — Task 1 extended with Steps 1e+1f: curl Wireshark pair into `tests/fixtures/local-samples/` with sha256 verification from `bin/fetch-e2e-pcaps`; Env A stays 4/4 Wireshark captures; "avoids ~400 MB corpus" accurate — 4 small files only.
**Status:** REMEDIATED

---

### F-W86S-P6-002 (HIGH) — Sibling e2e harnesses carry same silent-skip idiom [orchestrator deferral]

**Story:** STORY-182 (scope boundary)
**Defect:** The same `LOCAL_SAMPLES` / `fixture_present` silent-skip idiom that STORY-182 fixes in the IEC-104 harness also exists in:
- `tests/enip_e2e_real_pcaps_tests.rs` — same ITI CC-BY-4.0 redistributable class as iec104_e2e_real_pcaps_tests.rs; direct analog
- `tests/bc_2_12_011_story127_tests.rs`
- `tests/e2e_corpus_smoke_tests.rs`

These harnesses would continue silently skipping fixture-gated tests when fixtures are absent, defeating the gate purpose STORY-182 establishes.
**Orchestrator ruling:** Deferred from wave-86 per scope containment. ENIP pair is the same ITI CC-BY-4.0 class (direct analog); bc_2_12_011 and e2e_corpus also confirmed carrying the pattern. Follow-up story candidate at next planning cycle.
**Drift item:** DRIFT-e2e-sibling-harnesses added to STATE.md Drift Items.
**Status:** DEFERRED (scope containment) — DRIFT-e2e-sibling-harnesses tracked

---

### F-W86S-P6-003 (MED) — FIXTURE_GATED_TESTS registry absent; manifest comment misleading

**Story:** STORY-182 (AC-182-005 and test bodies)
**Defect:** The assert_eq comment in the manifest test described a structural assertion when it was only a size drift pin. No module-level const registry mapped test names to fixture names for all 4 fixture-gated tests, making it possible for a test to be added without updating the manifest.
**Remediation:** STORY-182 v1.6 — assert_eq comment relabeled "manifest-size drift pin (fires on manifest growth/shrink only)"; FIXTURE_GATED_TESTS module-level const added mapping all 4 fixture-gated tests (test_name, fixture_name); manifest test iterates and asserts each fixture_name is present in FIXTURE_MANIFEST.
**Status:** REMEDIATED

---

### F-W86S-P6-004 (MED) [process-gap] — --exact flag and "1 passed" grep absent from single-test invocations

**Story:** STORY-182 (AC-182-001/003/004/005 + Task 9)
**Defect:** Single-test cargo test invocations in ACs and tasks lacked `--exact` flag (substring matching risk — unintended test matches) and `| grep -E "1 passed"` assertion (no pass-count verification gate). The ci.yml step also lacked `| tee /dev/stderr | grep -q "Fixture coverage:"` to confirm CI-visible output.
**Process-gap note:** Validates PG-W86-009 class (S-7.01(c) post-fix verification read for HIGH/CRIT); the filter sweep obligation was not applied to all 4 ACs + Task 9 after the pass-5 remediation of F-W86S-P5-004.
**Remediation:** STORY-182 v1.6 — `--exact` added to all single-test filters; `| grep -E "1 passed"` piped to pass-assertions; ci.yml step additionally `| tee /dev/stderr | grep -q "Fixture coverage:"`; sweep applied to AC-182-001/003/004/005 + Task 9 all 4 + ACR.
**Status:** REMEDIATED

---

### F-W86S-P6-005 (MED) — AC-183-001 incorrectly includes CHANGELOG.md in rename target set

**Story:** STORY-183 (AC-183-001)
**Defect:** AC-183-001 stated "and 1 reference in CHANGELOG.md (see Task 2)" as part of the 13 test-file rename targets. The CHANGELOG.md entry is a historical reference protected by DF-SIBLING-SWEEP-001 carve-out (preserve, not rename).
**Remediation:** STORY-183 v1.6 — CHANGELOG.md reference removed from the 13-test-file count in AC-183-001; historical entry preserved per DF-SIBLING-SWEEP-001 (outside the 13 test-file refs).
**Status:** REMEDIATED

---

### F-W86S-P6-006 (MED) — Task 9 hermetic harness creates unnecessary minimal commit

**Story:** STORY-183 (Task 9)
**Defect:** Task 9 included "and create a minimal commit so the index is non-empty". `git ls-files` operates on tracked or staged files and does not require a commit; the commit step was unnecessary and introduced a side effect in the hermetic test environment (non-empty git history in tmp repo).
**Remediation:** STORY-183 v1.6 — "create a minimal commit" step removed; `git add violating.py` only; `git ls-files` works correctly with only staged files.
**Status:** REMEDIATED

---

### F-W86S-P6-007 (MED) — FAIL output format wrong in Task 9

**Story:** STORY-183 (Task 9)
**Defect:** Task 9 specified FAIL output as `FAIL  bin/violating.py: Pattern 32` (double-space, no brackets, no line number). Actual tool output format is `FAIL [bin/violating.py:1]: Pattern 32` (brackets, line number in path). The literal assertion would fail at delivery.
**Remediation:** STORY-183 v1.6 — FAIL format corrected to `FAIL [bin/violating.py:1]: Pattern 32`.
**Status:** REMEDIATED

---

### F-W86S-P6-008 (MED) — TC count stale in Task 13 docstring

**Story:** STORY-183 (Task 13)
**Defect:** Task 13 docstring line 5 carried a stale TC count not updated after v1.5 remediation (Task 12 → Task 13 renumbering, TC count not recomputed).
**Remediation:** STORY-183 v1.6 — TC count updated to TC1–21 (21 self-tests).
**Status:** REMEDIATED

---

### F-W86S-P6-009 (MED) [PO propagation] — Bare RED markers still classified as TIER-1 in story Notes

**Story:** STORY-183 (Notes section)
**Defect:** DF-GREEN-DOC-TENSE-SWEEP v5 classified bare RED tokens (RED:/RED-phase/RED reason/RED because) as TIER-1. Pass-6 adversary ran greps with full evidence; 15/17 hits were legitimate provenance (test headers, narrative context). Policy v6 re-tiers all 4 tokens to TIER-2. STORY-183 Notes still referenced the old TIER-1 classification.
**PO policy v6:** Grep evidence recorded inline in policies.yaml. Tool allowlists `RED-phase:` with shipped `GOOD_CASE`; Pattern 30 (`Expected RED:`) retained TIER-1 (0 live hits confirmed). 2 stale live sites adjudicated (see F-W86S-P6-010).
**Remediation:** STORY-183 v1.6 — Notes updated to reflect v6 re-tier; bare RED markers documented as TIER-2 (context-dependent); deferred scrub obligation noted (iec104_analyzer_tests.rs:6271 + modbus_detection_tests.rs:2472/:2480).
**Status:** REMEDIATED

---

### F-W86S-P6-010 (MED) [process-gap] — 2 live stale RED-prose sites adjudicated, deferred to maintenance

**Story:** STORY-183 (Notes + scope); PO policy v6
**Defect:** Two live sites contain stale RED-prose outside STORY-183's test-file rename scope:
- `tests/iec104_analyzer_tests.rs:6271` — stale RED-heading style; PO reword prescription: change to past-tense delivery note
- `tests/modbus_detection_tests.rs:2472/:2480` — stale RED-phrasing; PO reword prescription: reword to past-tense or remove

Both sites confirmed stale by grep evidence in policy v6. STORY-183 cannot address them (wrong story scope).
**Orchestrator adjudication:** Sites adjudicated STALE; deferred to next maintenance sweep with reword prescriptions in policy v6.
**Drift item:** DRIFT-stale-red-scrub added to STATE.md Drift Items.
**Status:** DEFERRED (next maintenance sweep) — DRIFT-stale-red-scrub tracked

---

### F-W86S-P6-011 (MED) — :721 prose site missing from Task 2 rename sweep

**Story:** STORY-183 (Task 2)
**Defect:** Task 2 rename sweep listed 13 total sites but omitted :721 (failure-message citing `if not rust_files:` guard) as a prose site requiring update to reflect the post-rename filename.
**Remediation:** STORY-183 v1.6 — :721 added to Task 2 as a prose rename site (failure-message cite).
**Status:** REMEDIATED

---

### F-W86S-P6-012 (MED) — input-hash NOTE does not document bash-hook divergence

**Story:** STORY-182 (frontmatter NOTE)
**Defect:** The input-hash NOTE in STORY-182 did not explicitly document that the `validate-input-hash` bash hook reports a divergent value (0a1812a vs canonical Python 9a0f34c) per PG-HASH-HOOK-DIVERGENCE. An implementer seeing the hook error without the NOTE would be tempted to recompute, destroying the canonical value.
**Remediation:** STORY-182 v1.6 — NOTE reworded: "Stored value is the canonical Python hash (9a0f34c); the bash hook reports a divergent value (0a1812a) — advisory only per PG-HASH-HOOK-DIVERGENCE".
**Status:** REMEDIATED

---

### F-W86S-P6-013 (MED) — traces_to missing 3 target files

**Story:** STORY-183 (frontmatter `traces_to`)
**Defect:** `traces_to` field missing three delivery targets: `CHANGELOG.md`, `.github/workflows/ci.yml`, and `bin/test_lint_cycle_artifact.py`. All three are modified by STORY-183 tasks.
**Remediation:** STORY-183 v1.6 — All 3 targets added to `traces_to`.
**Status:** REMEDIATED

---

### F-W86S-P6-014 (LOW) — ci.yml report step lacks `if: always()` and placement note

**Story:** STORY-182 (ci.yml step specification in Tasks)
**Defect:** ci.yml fixture coverage report step specification lacked `if: always()` condition (means step skips on test failure, defeating CI-visibility for broken runs) and a placement note (must appear AFTER main cargo test step to read its output).
**Remediation:** STORY-182 v1.6 — `if: always()` added to ci.yml step spec; placement AFTER main cargo test step noted; "permanently CI-visible" → "visible on every run incl. after test failures (if: always())".
**Status:** REMEDIATED

---

### F-W86S-P6-015 (LOW) — Imprecise "comment-only additive step" language

**Story:** STORY-182 (task prose)
**Defect:** "comment-only additive step" and "comment-style step" described an executable run step imprecisely — the step runs a command, not just adds a comment.
**Remediation:** STORY-182 v1.6 — Language corrected to "additive run step" / "executable run step".
**Status:** REMEDIATED

---

### F-W86S-P6-016 (LOW) — test_fixture_manifest_report() placement not mandated (DF-TEST-NAMESPACE-001)

**Story:** STORY-182 (AC-182-001 + Task 6)
**Defect:** AC-182-001 and Task 6 did not mandate that `test_fixture_manifest_report()` reside inside `mod iec104_e2e_real_pcaps`. DF-TEST-NAMESPACE-001 requires test functions to be in the correct namespace module; without explicit mandate the function could be placed outside the module.
**Remediation:** STORY-182 v1.6 — Placement inside `mod iec104_e2e_real_pcaps` mandated in AC-182-001 and Task 6.
**Status:** REMEDIATED

---

### F-W86S-P6-017 (LOW) — ci.yml :434/:462 missing from Task 10 sweep

**Story:** STORY-183 (Task 10)
**Defect:** Task 10 ci.yml sweep did not include :434 (job comment) and :462 (step name) as stale prose sites referencing the old `bin/check_green_doc_tense` naming.
**Remediation:** STORY-183 v1.6 — :434 job comment + :462 step name added as stale prose sites in Task 10.
**Status:** REMEDIATED

---

### F-W86S-P6-018 (LOW) — :402 misclassified as BAD_CASE in Task 5

**Story:** STORY-183 (Task 5)
**Defect:** Task 5 quote-escaping listed "Two fixtures in BAD_CASES (:91, :97)" implying :402 was also in BAD_CASES. :402 is a GOOD_CASE fixture.
**Remediation:** STORY-183 v1.6 — Corrected: "Two fixtures in BAD_CASES (:91, :97)"; :402 explicitly noted as GOOD_CASE.
**Status:** REMEDIATED

---

### F-W86S-P6-019 (LOW) — Task 13 lacks state-manager DRIFT-docstring-scan instruction

**Story:** STORY-183 (Task 13)
**Defect:** Task 13 did not include an instruction for the state-manager to record the DRIFT-docstring-scan drift item (the deferred docstring-scanning obligation from F-W86S-P4-002 PO ruling policy v5).
**Remediation:** STORY-183 v1.6 — Task 13 updated to include "state-manager records DRIFT-docstring-scan".
**Status:** REMEDIATED

---

### F-W86S-P6-020 (NIT) — AC-183-007 item/tuple count mismatch

**Story:** STORY-183 (AC-183-007)
**Defect:** AC-183-007 stated 37 items / 36 tuples without noting that item 5 shares tuple 4 (two items map to the same tuple). The prose was inconsistent on whether the count referred to items or tuples.
**Remediation:** STORY-183 v1.6 — AC-183-007 prose updated: "37 items / 36 tuples — item 5 shares tuple 4" to make the count semantics explicit.
**Status:** REMEDIATED

---

## Non-Blocking Observations

### Task-4 Reworded-Comment Fragility (advisory)

Task 4 in STORY-182 specifies reworded inline comments in cargo test invocations to prevent self-match by the check-green-doc-tense tool. The fragility: if the tool's TIER-1 pattern set expands in a future policy revision, new TIER-1 tokens could match the reworded comments. Advisory — no AC change required for this wave. Note for future maintenance: verify Task 4 comment forms against any future TIER-1 pattern additions before delivery.

### Bin-Selftest CI Gate Gap (pre-existing — PG-W84-012/PG-W86-003 scope)

`bin/test_lint_cycle_artifact.py` and `bin/test_compute_input_hash.py` pass in manual Gate 1 invocation but are not registered as required GitHub status checks for the `develop` branch. STORY-183 delivery adds `bin/test_check_green_doc_tense.py` (or equivalent) to this surface, increasing the exposure from 2 to 3 ungated bin/ self-tests. This observation is pre-existing (PG-W84-012, wave-84) with a scope extension noted in PG-W86-003. Attached to PG-W84-012/PG-W86-003 ops-task scope — devops-engineer dispatch + human authorization required for branch-protection mutation. No new PG ID assigned (scope extension only).
