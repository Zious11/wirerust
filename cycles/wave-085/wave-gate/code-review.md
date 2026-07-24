---
document_type: wave-gate-code-review
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-07-24T00:00:00Z
cycle: "wave-085"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Wave-85 Gate Code Review

**Gate:** wave-085 integration gate (D-510)  
**Reviewer verdict:** APPROVE — 0 MAJOR / 1 MINOR / 5 NIT  
**Security verdict:** APPROVE — 0 CRITICAL / 0 HIGH / 0 MEDIUM / 0 LOW (SEC-001 closure confirmed sound)

This file satisfies AC-158-006 / PG-W71-CODEREVIEW-ARTIFACT: every MINOR and NIT finding from the gate-level code review is enumerated here with its full text and disposition. A gate with zero findings of a class still documents that fact; this gate has zero MAJOR findings.

---

## Code Review Findings

### CR-001 — MINOR: evidence-vector helper dedup in iec104.rs

**Severity:** MINOR  
**Location:** `src/analyzer/iec104.rs` approximately lines 48–64 (evidence-vector construction helpers)  
**Finding text:** The evidence-vector construction helper has duplicated arms at lines 48..=51 and 61..=64. Both arms build the same `vec![]` pattern with identical logic for the TypeID-58/59 case vs. TypeID-61/63/65 case. The duplication is a refactoring opportunity: a shared helper that takes a `TypeId` parameter could eliminate the repeated match arms, reducing the risk of a future bug where one arm is updated but not the other.

**Disposition:** DEFERRED — Added to tech-debt register as `CR-W85G-001` (P3). No behavioral impact; purely structural duplication. Batch into next `iec104.rs` touch.

---

### CR-002 — NIT: enip.rs approximate line references in docstrings

**Severity:** NIT  
**Location:** `src/analyzer/enip.rs` (approximate — specific line reference TBD in next doc sweep)  
**Finding text:** Several docstrings in `enip.rs` reference adjacent line numbers (e.g., "see line NNN") that have shifted after the STORY-181 take-remove-reinsert refactor. The references are approximate and do not affect correctness, but they could mislead future contributors navigating the code.

**Disposition:** DEFERRED — batch into next doc sweep of `enip.rs`. No behavioral impact.

---

### CR-003 — NIT: story_180 module-header "currently" RED prose

**Severity:** NIT  
**Location:** `src/analyzer/iec104.rs` story_180 module header (approximate)  
**Finding text:** The module-header comment for the TypeID 58–64 implementation uses the present-tense word "currently" in a description that describes stable, shipped behavior ("currently falls through to the detection path"). This is a green-doc-tense violation — "currently" is stale-RED phrasing that the `bin/check-green-doc-tense` tool is meant to catch, but PG-W85-003 documents that the tool's pattern set does not yet cover the `"Expected RED:"`/`"currently falls through"` class.

**Disposition:** DEFERRED — next doc sweep of `iec104.rs`. Note: PG-W85-003 tracks the tooling gap that allowed this to pass the Step-4 gate undetected; see `cycles/wave-085/lessons.md`.

---

### CR-004 — NIT: CHANGELOG TDD vocabulary

**Severity:** NIT  
**Location:** `CHANGELOG.md` (wave-85 entries, approximate)  
**Finding text:** Several CHANGELOG entries for STORY-180 use the phrase "test-driven" in a context where "red-first" or "BC-anchored" would be more precise per the project's vocabulary conventions. The phrasing is not incorrect but is inconsistently precise compared to wave-84 entries.

**Disposition:** DEFERRED — next release-notes prep sweep. No traceability impact.

---

### CR-005 — NIT: make_asdu duplication across test mods

**Severity:** NIT  
**Location:** Test modules for STORY-180 (approximate)  
**Finding text:** The `make_asdu` helper function is duplicated across multiple test modules in the wave-85 test suite. Each module defines its own local copy with near-identical structure. A shared test-fixture crate or `tests/common/iec104_fixture.rs` module would eliminate the duplication and reduce the risk of fixture divergence as the IEC-104 test suite grows.

**Disposition:** DEFERRED — next test-hygiene sweep. No correctness impact; purely a maintainability concern.

---

### CR-006 — NIT: "owned-out" docstring phrasing

**Severity:** NIT  
**Location:** `src/analyzer/enip.rs` (post-STORY-181 refactor, approximate)  
**Finding text:** The SAFETY comment above the take-remove-reinsert block uses the phrase "owned-out flow" to describe the local `EnipFlowState` variable that has been removed from `self.flows`. This phrase is not standard Rust convention; "owned local flow" or "flow taken from map" would be clearer.

**Disposition:** DEFERRED — next doc sweep of `enip.rs`. No semantic impact.

---

## Adversary Pass Observations (P2 and P3)

The following observations were raised by the adversarial reviewer during gate passes P2 and P3. They are recorded here per PG-W71-CODEREVIEW-ARTIFACT to ensure permanent recoverability.

### F-W85G-P3-001 — LOW: tech-debt-register SEC-001 line-cite

**Pass:** P3  
**Severity:** LOW  
**Finding text:** The tech-debt-register `SEC-001` row cites the PDU dispatch loop at `enip.rs:992-999` but the actual line range (accounting for file growth since the finding was first recorded) is `993-1000`. The one-line shift is due to a preceding declaration added during the STORY-174 formal hardening pass. The historic description accurately describes the code pattern; only the line numbers are off-by-one.

**Disposition:** FIXED this burst — `tech-debt-register.md` SEC-001 row line-cite corrected to `993-1000`. Resolution History row updated to match.

---

### F-W85G-P3-002 — LOW: no-action observation

**Pass:** P3  
**Severity:** LOW  
**Finding text:** Minor phrasing observation in factory-side documentation about the STORY-181 AC scope description. The description is accurate and consistent with the BC; the observation noted a slightly informal word choice ("wired out" vs. "extracted and re-inserted") that is a matter of style.

**Disposition:** No action — phrasing is clear and not misleading. Informational note; no open item.

---

### F-W85G-P3-003 — INFO: informational

**Pass:** P3  
**Severity:** INFO (informational only)  
**Finding text:** Informational observation that the wave-85 holdout score for HS-136 (0.9) reflects corpus availability rather than a product limitation; the comment noted that if richer IEC-104 timed-command captures become available in the ITI corpus, a re-evaluation would be straightforward and expected to score 1.0.

**Disposition:** No action — recorded in gate-summary.md holdout table as corpus-caveat. Not a product defect.

---

## Summary

| Finding | Severity | Disposition |
|---------|----------|-------------|
| CR-001 | MINOR | DEFERRED — tech-debt register `CR-W85G-001` (P3); iec104.rs next touch |
| CR-002 | NIT | DEFERRED — next enip.rs doc sweep |
| CR-003 | NIT | DEFERRED — next iec104.rs doc sweep (PG-W85-003 tooling gap) |
| CR-004 | NIT | DEFERRED — next release-notes prep |
| CR-005 | NIT | DEFERRED — next test-hygiene sweep |
| CR-006 | NIT | DEFERRED — next enip.rs doc sweep |
| F-W85G-P3-001 | LOW | FIXED this burst — tech-debt-register SEC-001 line-cite 992-999 → 993-1000 |
| F-W85G-P3-002 | LOW | No action — style-only, non-misleading |
| F-W85G-P3-003 | INFO | No action — informational corpus-caveat note |
