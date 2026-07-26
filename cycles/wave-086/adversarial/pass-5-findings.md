---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: state-manager
timestamp: 2026-07-25T23:55:00Z
cycle: "wave-086"
pass: 5
inputs: [stories/STORY-182.md, stories/STORY-183.md, stories/STORY-INDEX.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Pass-5 Findings — wave-086

**Pass date:** 2026-07-25
**Tally:** 28 findings — 0 CRIT / 3 HIGH / 15 MED / 8 LOW / 2 NIT
**Novelty:** HIGH (adversary flagged pass-4 partial-fix regressions in F-002/F-003/F-012 class)
**Streak after remediation:** 0/3 (not clean; reset)
**Status:** REMEDIATED — Awaiting Pass 6
**Orchestrator evidence:** F-016 CC-BY-4.0 confirmed via GitHub API; F-018 input-hash MATCH confirmed via canonical tool
**Process-gap:** F-026 [process-gap] → orchestrator ruling: sibling-doc MUST convention

---

## Finding Index

| ID | Sev | Route | Summary |
|----|-----|-------|---------|
| F-W86S-P5-001 | HIGH | STORY-182 v1.4→v1.5 | Vacuous `starts_with` resolver guard; `assert_eq!(path.parent(), ...)` exact-dir check required |
| F-W86S-P5-002 | HIGH | STORY-183 v1.4→v1.5 | Unimplementable hermetic harness: no env-override mechanism; harness must copy script into `<tmp>/bin/` and use `PATH` isolation |
| F-W86S-P5-003 | HIGH | STORY-183 v1.4→v1.5 | Missing `git add violating.py` in hermetic harness leaves zero-file-guard falsely exiting 1 |
| F-W86S-P5-004 | MED | STORY-183 v1.4→v1.5 | Historical `CHANGELOG:741` preservation: DF-SIBLING-SWEEP-001 carve-out must be explicit — rewrite vs preserve |
| F-W86S-P5-005 | MED | STORY-183 v1.4→v1.5 | Fictional `test_changelog_gate_check.py` — correct filename is `test_changelog_gate_content.py` |
| F-W86S-P5-006 | MED | STORY-183 v1.4→v1.5 | Task 6 quoted-phrase advice deleted — literal-phrase rule admits no quoted-phrase exception |
| F-W86S-P5-007 | MED | STORY-183 v1.4→v1.5 | EC-011 task renumber residue: renumbered Task 12 → Task 13 throughout body |
| F-W86S-P5-008 | MED | STORY-183 v1.4→v1.5 | Nine residual v4 governing cites (`DF-GREEN-DOC-TENSE-SWEEP v4`) must be updated to v5 |
| F-W86S-P5-009 | MED | STORY-183 v1.4→v1.5 | P4-002 ruling ID mis-cited at 3 sites; correct ID is P4-004 |
| F-W86S-P5-010 | MED | STORY-183 v1.4→v1.5 | Docstring line 5 carries stale `TC` count; must be updated to match delivered task count |
| F-W86S-P5-011 | MED | STORY-182 v1.4→v1.5 | `README:41-44` all-provenance overclaim; scoped to ITI captures only |
| F-W86S-P5-012 | MED | STORY-182 v1.4→v1.5 | Dropped manifest↔usage coupling: `FIXTURE_MANIFEST.len() == 4` exhaustiveness assertion required in AC-182-005 loops |
| F-W86S-P5-013 | MED | STORY-182 v1.4→v1.5 | Non-discriminating Env A + 400MB fetch: Task 1 bare fetch replaced with direct `curl+shasum` for 4 named files; Env A repointed at 4/4 Wireshark cases |
| F-W86S-P5-014 | MED | STORY-INDEX v1.3→v1.5 (state-manager) | Index body row still shows v1.3 for both stories; must be updated to v1.5 |
| F-W86S-P5-015 | MED | STORY-182 v1.4→v1.5 | Architecture Mapping and Notes propagation gap: rows for `ci.yml`, `CLAUDE.md`, and `fixture-count-gate-entry.md` missing |
| F-W86S-P5-016 | MED | STORY-182 v1.4→v1.5 | Upstream LICENSE verification: AC-182-002 requires explicit CC-BY-4.0 precondition; resolved — CC-BY-4.0 confirmed via GitHub API |
| F-W86S-P5-017 | MED | STORY-183 v1.4→v1.5 | Negative-guard `GOOD_CASE` unassigned in suffix-scoping task; must be task-assigned |
| F-W86S-P5-018 | MED | state-manager (hash repair) | Input-hash EXECUTION-REQUIRED: story-writer stored bash-hook values; resolved — canonical values `9a0f34c` / `9c9b12f` verified via canonical Python tool |
| F-W86S-P5-019 | LOW | STORY-183 v1.4→v1.5 | GOOD_CASES off-by-one: 13→14 `GOOD_CASES` / 25→26 total in Architecture Mapping |
| F-W86S-P5-020 | LOW | STORY-183 v1.4→v1.5 | File line count 914→913 in acceptance criteria |
| F-W86S-P5-021 | LOW | STORY-183 v1.4→v1.5 | Tool own surfaces missing from rename sweep: `bin/check-green-doc-tense` itself must be in sweep scope |
| F-W86S-P5-022 | LOW | STORY-183 v1.4→v1.5 | Optional-vs-mandated Pattern 30: `"may be added"` language weakens the MUST; corrected to `"is added as Pattern 30"` |
| F-W86S-P5-023 | LOW | STORY-182 v1.4→v1.5 | Module-level ambiguity: `mod iec104_e2e_real_pcaps` must be specific module reference, not generic module-level prose |
| F-W86S-P5-024 | LOW | STORY-182 v1.4→v1.5 | `grep -c` exit semantics: non-zero exit on 0 matches; use `grep -c ... \|\| true` + `Expected output: 0` |
| F-W86S-P5-025 | LOW | STORY-183 v1.4→v1.5 | `RED GATE` token present in Task 13 replacement example; dropped per pass-5 ruling |
| F-W86S-P5-026 | LOW [process-gap] | STORY-182 v1.4→v1.5; orchestrator ruling | Gate-entry obligation unwired: enforceable G1 gate obligation text added; orchestrator ruling: sibling-doc MUST convention governs gate-entry annotations (PG-W86-008 candidate) |
| F-W86S-P5-027 | NIT | STORY-183 v1.4→v1.5 | AC-183-001 `.name` attribute comparison is not repo-relative path string; absolute path assertion removed |
| F-W86S-P5-028 | NIT | STORY-182 v1.4→v1.5 | `E2E-PCAPS.md:48-50` overclaim (all-provenance language); scoped to Wireshark 4-file set |

---

## Detailed Finding Records

### F-W86S-P5-001 (HIGH) — Vacuous `starts_with` resolver guard

**Story:** STORY-182 (AC-182-005)
**Defect:** `fixture_path().starts_with(".")` vacuously passes for any relative path including incorrect ones. The coupling predicate must assert the exact parent directory using `assert_eq!(path.parent().unwrap(), Path::new("tests/fixtures"))`.
**Remediation:** STORY-182 v1.5 — AC-182-005 updated with `parent()`-based resolver-coupling predicate.
**Status:** REMEDIATED

---

### F-W86S-P5-002 + F-W86S-P5-003 (HIGH) — Unimplementable hermetic harness

**Story:** STORY-183 (Task 9)
**Defect (F-002):** Hermetic harness Task 9 specified via `PATH` manipulation but assumed the script was accessible as a system command; no env-override mechanism existed to make `check-green-doc-tense` findable without modifying `$PATH` in the subprocess.
**Defect (F-003):** Without `git add violating.py` before running the harness, the zero-file-guard detects no staged Python file and exits 1 prematurely, meaning the harness tests the wrong condition.
**Remediation:** STORY-183 v1.5 — Task 9 revised to copy script into `<tmp>/bin/`, run `git add violating.py` before invocation, and assert literal `FAIL`-prefix output.
**Status:** REMEDIATED (both findings addressed together)

---

### F-W86S-P5-004 (MED) — CHANGELOG historical preservation

**Story:** STORY-183
**Defect:** The CHANGELOG:741 historical line was marked for scrub in v1.4 without noting the DF-SIBLING-SWEEP-001 carve-out for historical entries. This created ambiguity about whether historical entries are exempt or must be rewritten.
**Remediation:** STORY-183 v1.5 — CHANGELOG:741 preserved per DF-SIBLING-SWEEP-001 historical-entry carve-out; explicit annotation added.
**Status:** REMEDIATED

---

### F-W86S-P5-005 (MED) — Fictional `test_changelog_gate_check.py`

**Story:** STORY-183
**Defect:** Task 7 cited `bin/test_changelog_gate_check.py` — a filename that does not exist. The actual self-test file is `bin/test_changelog_gate_content.py`.
**Remediation:** STORY-183 v1.5 — corrected to `test_changelog_gate_content.py` at all citation sites.
**Status:** REMEDIATED

---

### F-W86S-P5-006 (MED) — Task 6 quoted-phrase residue

**Story:** STORY-183 (Task 6)
**Defect:** Task 6 included advice that quoted phrases (phrases in double-quotes) are exempt from the literal-phrase scanning rule. This exception does not exist in the policy; the literal-phrase rule is unconditional.
**Remediation:** STORY-183 v1.5 — Task 6 quoted-phrase exception deleted; literal-phrase rule stated without caveat.
**Status:** REMEDIATED

---

### F-W86S-P5-007 (MED) — EC-011 task renumber residue

**Story:** STORY-183
**Defect:** EC-011 was added as Task 12 in v1.4 but the body still contained references to `Task 12` at locations that should have been renumbered to `Task 13` (the renumbering from the v1.4 burst was incomplete).
**Remediation:** STORY-183 v1.5 — `EC-011 Task 12` → `Task 13` corrected throughout body.
**Status:** REMEDIATED

---

### F-W86S-P5-008 (MED) — Nine residual v4 governing cites

**Story:** STORY-183
**Defect:** Nine instances of `DF-GREEN-DOC-TENSE-SWEEP v4` remained in the body after the v5 transition. The v4→v5 governing cite sweep in v1.4 missed 9 locations.
**Remediation:** STORY-183 v1.5 — all 9 residual v4 cites updated to v5.
**Status:** REMEDIATED

---

### F-W86S-P5-009 (MED) — P4-002/P4-004 mis-cite

**Story:** STORY-183
**Defect:** The orchestrator ruling ID was cited as `P4-002` at 3 sites in the body. The correct ruling ID from the pass-4 burst is `P4-004`.
**Remediation:** STORY-183 v1.5 — `P4-002` → `P4-004` at all 3 sites.
**Status:** REMEDIATED

---

### F-W86S-P5-010 (MED) — Docstring line-5 stale count

**Story:** STORY-183
**Defect:** The tool docstring line 5 carried a stale total task count `TC` that did not match the delivered task list after Task 13 was added in v1.4.
**Remediation:** STORY-183 v1.5 — docstring line 5 stale count corrected to match delivered task count.
**Status:** REMEDIATED

---

### F-W86S-P5-011 (MED) — README all-provenance overclaim

**Story:** STORY-182
**Defect:** `README:41-44` claimed provenance for all capture sources. This is a scope overclaim; the provenance table in `tests/fixtures/README.md` covers only the committed ITI captures.
**Remediation:** STORY-182 v1.5 — `README:41-44` provenance reworded to scope to ITI captures only.
**Status:** REMEDIATED

---

### F-W86S-P5-012 (MED) — Dropped manifest↔usage coupling

**Story:** STORY-182 (AC-182-005)
**Defect:** The manifest exhaustiveness assertion (`FIXTURE_MANIFEST.len() == 4`) was dropped from the AC-182-005 canonical loops in v1.4. Without this assertion, a manifest with extra or missing entries passes the loop without detection.
**Remediation:** STORY-182 v1.5 — `FIXTURE_MANIFEST.len() == 4` exhaustiveness check restored in AC-182-005 loops.
**Status:** REMEDIATED

---

### F-W86S-P5-013 (MED) — Non-discriminating Env A + 400MB fetch

**Story:** STORY-182 (Task 1, Env A)
**Defect:** Env A as specified in v1.4 included a full pcapng corpus fetch that would download ~400MB and was not discriminating for the 4-file Wireshark test case. The Task 1 bare fetch did not specify which files to fetch.
**Remediation:** STORY-182 v1.5 — Task 1 uses direct `curl+shasum` fetches for the 4 named Wireshark files; Env A repointed to the 4/4 Wireshark cases with explicit integrity verification.
**Status:** REMEDIATED

---

### F-W86S-P5-014 (MED) — STORY-INDEX body row v1.3 residue

**Story:** STORY-INDEX (state-manager route)
**Defect:** STORY-INDEX body rows for STORY-182 and STORY-183 still showed `v1.3` versions after the v1.4 remediation burst. The index was updated to v4.00 but the body cells were not updated.
**Remediation:** STORY-INDEX v4.01 — body rows updated to v1.5 for both stories (story-writer; closes F-014).
**Status:** REMEDIATED

---

### F-W86S-P5-015 (MED) — Architecture Mapping/Notes propagation gap

**Story:** STORY-182
**Defect:** Architecture Mapping and Notes sections in STORY-182 lacked rows for `ci.yml`, `CLAUDE.md`, and `fixture-count-gate-entry.md` — three artifacts introduced or modified by this story's deliverables. The sibling-sweep propagation obligation requires these rows.
**Remediation:** STORY-182 v1.5 — Architecture Mapping and Notes rows added for all three artifacts.
**Status:** REMEDIATED

---

### F-W86S-P5-016 (MED) — Upstream LICENSE verification [RESOLVED via orchestrator evidence]

**Story:** STORY-182 (AC-182-002)
**Defect:** AC-182-002 required CC-BY-4.0 license verification for the upstream ITI/ICS-Security-Tools repo, but the precondition text did not cite the verification method or evidence.
**Resolution:** Orchestrator pre-verified: upstream ITI/ICS-Security-Tools repo-level `LICENSE.md` = CC-BY-4.0 confirmed via GitHub API.
**Remediation:** STORY-182 v1.5 — AC-182-002 precondition text updated with CC-BY-4.0 annotation and API-evidence citation.
**Status:** REMEDIATED

---

### F-W86S-P5-017 (MED) — Negative-guard GOOD_CASE unassigned

**Story:** STORY-183
**Defect:** The suffix-scoping negative guard `GOOD_CASE` in AC-183-006 was present but not task-assigned — no task in the story body owned the obligation to implement and verify it.
**Remediation:** STORY-183 v1.5 — negative-guard `GOOD_CASE` task-assigned to Task 8.
**Status:** REMEDIATED

---

### F-W86S-P5-018 (MED) — Input-hash EXECUTION-REQUIRED [RESOLVED via canonical tool]

**Route:** State-manager hash repair (DF-INPUT-HASH-CANONICAL-001)
**Defect:** Story-writer (under blocking-hook pressure from `validate-input-hash` hook) overwrote both stored `input-hash:` values with bash-hook-computed values: STORY-182 stored `0a1812a` (bash-hook), STORY-183 stored `5598136` (bash-hook). Canonical Python values are `9a0f34c` and `9c9b12f` respectively.
**Resolution:** Orchestrator pre-verified canonical values. State-manager ran `bin/compute-input-hash --write` on both stories at STEP 0 of this burst. Stored values restored to `9a0f34c` / `9c9b12f`.
**Status:** REMEDIATED (STEP 0 of this burst). PG-W86-008 candidate filed.

---

### F-W86S-P5-019 (LOW) — GOOD_CASES off-by-one

**Story:** STORY-183 (Architecture Mapping)
**Defect:** Architecture Mapping listed 13 `GOOD_CASES` when the correct count (after Task 8 suffix-scoping addition) is 14, and total was 25 when it should be 26.
**Remediation:** STORY-183 v1.5 — 13→14 `GOOD_CASES`; 25→26 total in Architecture Mapping.
**Status:** REMEDIATED

---

### F-W86S-P5-020 (LOW) — Test file line count

**Story:** STORY-183
**Defect:** AC referenced 914 lines for the test file; correct count post-edit is 913.
**Remediation:** STORY-183 v1.5 — 914→913 line count corrected.
**Status:** REMEDIATED

---

### F-W86S-P5-021 (LOW) — Tool own surfaces in rename sweep

**Story:** STORY-183 (rename sweep scope)
**Defect:** The rename sweep task did not include `bin/check-green-doc-tense` itself as a sweep target, even though the tool name appears in prose as part of its own invocation surface.
**Remediation:** STORY-183 v1.5 — tool own surfaces added to rename sweep scope.
**Status:** REMEDIATED

---

### F-W86S-P5-022 (LOW) — Optional-vs-mandated Pattern 30

**Story:** STORY-183
**Defect:** The text `"may be added as Pattern 30"` used optional language for a mandatory step; Pattern 30 addition is required by the policy.
**Remediation:** STORY-183 v1.5 — `"may be added"` → `"is added as Pattern 30"`.
**Status:** REMEDIATED

---

### F-W86S-P5-023 (LOW) — Module-level ambiguity

**Story:** STORY-182
**Defect:** Body prose referenced `mod iec104_e2e_real_pcaps` as a "module-level" construct without specifying the exact module, making the AC ambiguous for the implementer.
**Remediation:** STORY-182 v1.5 — `mod iec104_e2e_real_pcaps` named specifically as the test module.
**Status:** REMEDIATED

---

### F-W86S-P5-024 (LOW) — `grep -c` exit semantics

**Story:** STORY-182
**Defect:** AC used `grep -c` without `|| true`; `grep -c` exits non-zero when count is 0, which would cause test failures for correct-zero-match assertions.
**Remediation:** STORY-182 v1.5 — `grep -c ... || true` with `Expected output: 0` annotation.
**Status:** REMEDIATED

---

### F-W86S-P5-025 (LOW) — RED GATE token in replacement example

**Story:** STORY-183 (Task 13)
**Defect:** The Task 13 replacement example contained the `RED GATE` token verbatim, which would itself be flagged by the tool being specified. The example must not contain tokens that trigger the gate.
**Remediation:** STORY-183 v1.5 — `RED GATE` token dropped from replacement example.
**Status:** REMEDIATED

---

### F-W86S-P5-026 (LOW) [process-gap] — Gate-entry obligation unwired

**Story:** STORY-182; orchestrator ruling
**Defect:** The `fixture-count-gate-entry.md` maintenance document described a gate-entry obligation but the obligation text was not in enforceable form (no MUST-language; no gate-check protocol).
**Orchestrator ruling:** Gate-entry annotations in `.factory/maintenance/` documents use the sibling-doc MUST convention; enforceable obligation text required.
**Remediation:** STORY-182 v1.5 — gate-entry obligation text made enforceable with MUST-language and sibling-doc gate protocol.
**Process-gap:** PG-W86-008 candidate (see process-gap-ledger.md).
**Status:** REMEDIATED

---

### F-W86S-P5-027 (NIT) — Absolute-path assertion

**Story:** STORY-183 (AC-183-001)
**Defect:** AC-183-001 compared `.name` attribute as a repo-relative path string; `.name` in Python `pathlib` is only the final component, not a full path.
**Remediation:** STORY-183 v1.5 — absolute-path assertion removed; `.name` usage scoped correctly.
**Status:** REMEDIATED

---

### F-W86S-P5-028 (NIT) — E2E-PCAPS over-read

**Story:** STORY-182 (`E2E-PCAPS.md:48-50`)
**Defect:** Lines 48-50 contained all-provenance language implying the entire corpus had CC-BY-4.0 attribution, when only the 4 committed Wireshark captures are attributed at those lines.
**Remediation:** STORY-182 v1.5 — `E2E-PCAPS.md:48-50` scope narrowed to the 4-file Wireshark set.
**Status:** REMEDIATED

---

## Remediation Summary

All 28 findings remediated. STORY-182 v1.4→v1.5; STORY-183 v1.4→v1.5; STORY-INDEX v4.00→v4.01.
Input-hash repair: STORY-182 `0a1812a`(bash-hook)→`9a0f34c`(canonical); STORY-183 `5598136`(bash-hook)→`9c9b12f`(canonical).
Process-gap candidates: PG-W86-008 (hash-repair protocol) and PG-W86-009 (partial-fix regression) filed.
Streak: 0/3 (pass-5 not clean; pass-6 next).
