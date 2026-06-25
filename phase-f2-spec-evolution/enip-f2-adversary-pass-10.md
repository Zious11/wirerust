---
document_type: adversarial-review
cycle: feature-enip-v0.11.0
pass: 10
verdict: PASS
critical: 0
high: 0
medium: 0
low: 3
novelty: LOW
reviewer: adversary
timestamp: 2026-06-24T00:00:00Z
---

# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 10

## Verdict: PASS — 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW

**Novelty: LOW.** The Pass-9 0x00B2-only fix propagated cleanly to all 6 BCs + ADR + indices
with no stale "0x00B1 detected" clause remaining. ForwardOpen/Close confirmed in-scope on the
0x00B2 path. All 9 adversary axes clean.

## Findings

All 3 findings are LOW severity. All REMEDIATED in the final-polish burst.

### F-P10-001 [LOW] — VP-032 module label used `src/` prefix (non-sibling-VP convention)

**File:** `.factory/specs/verification-properties/vp-032-enip-parse-safety.md` (frontmatter)
**Before:** `module: "src/analyzer/enip.rs"`
**After:** `module: "analyzer/enip.rs"`

**Rationale:** All sibling VPs (VP-022, VP-023, VP-024) use the repo-relative path without
the `src/` prefix in the `module:` frontmatter field (e.g., `analyzer/modbus.rs`,
`analyzer/dnp3.rs`, `analyzer/arp.rs`). The VP-INDEX catalog row was already corrected in a
prior pass (Pass-N VP-INDEX update); only the vp-032 frontmatter itself retained the `src/`
prefix. Normalized for consistency with sibling-VP convention. The VP-INDEX catalog row and
verification-coverage-matrix row were already corrected to `analyzer/enip.rs` in the staged
worktree changes prior to this pass.

**Status: REMEDIATED** — frontmatter `module:` updated to `analyzer/enip.rs`.

---

### F-P10-002 [LOW] — BC-2.17.014 PC4 mirror-rationale was response-voice ("Mirror of BC-2.17.008")

**File:** `.factory/specs/behavioral-contracts/ss-17/BC-2.17.014.md` (Precondition PC4)
**Before:** `(Mirror of BC-2.17.008 existing 0x00B2-only gate on response parsing.)`
**After:** `(Consistent with the 0x00B2-only request-side gate applied across
BC-2.17.011/012/013, and the symmetric response-side gate in BC-2.17.008.)`

**Rationale:** PC4 governs the *request* side gate for Pattern A (Identity Object reads).
The prior rationale cited BC-2.17.008 (the response-side gate) as the mirror, which made
the language response-framed. The corrected text clarifies that the request-side gate is
consistent *across all request BCs* (BC-2.17.011/012/013) and is *symmetric* with the
response-side gate in BC-2.17.008 — making the request-side nature explicit.

**Status: REMEDIATED** — PC4 rationale reworded to request-side voice.

---

### F-P10-003 [process-gap] — ADR-010 status `proposed` instead of `accepted`; downstream artifacts already locked to it

**File:** `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md` (frontmatter)
**Before:** `status: proposed`, `accepted_date: null`
**After:** `status: accepted`, `accepted_date: "2026-06-24"`

**Rationale:** ADR-010 has been the authoritative architectural decision source for the entire
F2 cycle — all 25 BCs, VP-032, the architecture delta, and the PRD delta trace to it as
accepted truth. The `proposed` status was a carry-forward from the initial F2 authoring session
and was never updated when the document was ratified at the Pass-6 PASS gate. The downstream
artifacts (BCs, VP-032, ARCH-INDEX) were already written as if the ADR were accepted; the
frontmatter status was the only inconsistency. Corrected to `accepted` with `accepted_date:
2026-06-24` (the date of first clean pass).

**Status: REMEDIATED** — frontmatter updated to `status: accepted`, `accepted_date: "2026-06-24"`.

---

## Pass-8 Deferred LOWs — All Cleared

The 3 LOW findings deferred at Pass-8 are resolved in this final-polish burst:

| ID | Summary | Resolution |
|----|---------|------------|
| F8-01 | 0x4B/GetAndClear labeled "firmware download marker" — ODVA grounding is a wirerust convention, not normative. Add citation/vendor-specific note. | RESOLVED: Note added to BC-2.17.007 Invariant 6 and ADR-010 Decision 7 table row, stating 0x4B is a wirerust-internal convention not an ODVA normative common service code. |
| F8-02 | ADR-010 Decision 4 specifies `EnipFlowState` but never sketches the `EnipAnalyzer` aggregate struct. Add struct sketch before F4 implementation. | RESOLVED: Full `EnipAnalyzer` struct sketch with all fields (flows, enip_write_burst_threshold, total_pdu_count, write_count, error_count, parse_errors, all_findings, dropped_findings, command_distribution) added to ADR-010 Decision 4 with BC cross-reference annotations. |
| F8-03 | BC-2.17.014 should state `total_error_count = flow.error_counts_in_window.values().sum()`. | RESOLVED: One-sentence clarification added to BC-2.17.014 Invariant 3 error-burst threshold block. |

**Total remediated in final-polish burst: 6 findings (3 Pass-10 LOWs + 3 deferred Pass-8 LOWs).**

---

## 9-Axis Review Summary

| Axis | Status | Notes |
|------|--------|-------|
| 1. Protocol correctness | CLEAN | 0x00B2-only CIP service detection propagated cleanly to all 6 BCs; ForwardOpen/Close confirmed in-scope on 0x00B2 path |
| 2. Precondition completeness | CLEAN | All PC gates consistent with request-side convention; PC4 mirror-rationale corrected (F-P10-002) |
| 3. Postcondition precision | CLEAN | Pattern A/B finding shapes fully specified in BC-2.17.014 |
| 4. Invariant coverage | CLEAN | 0x4B wirerust-convention note added (F8-01); total_error_count sum clarification added (F8-03) |
| 5. Edge-case coverage | CLEAN | 0x00B1 deferral explicitly tested in EC-009 of BC-2.17.014 |
| 6. MITRE technique mapping | CLEAN | T1693.001 seeded-not-emitted status confirmed; no stale T0857 references |
| 7. ADR consistency | CLEAN | ADR-010 status accepted (F-P10-003); EnipAnalyzer struct added (F8-02); Decision 8 0x00B2-only gate unchanged |
| 8. VP/BC cross-reference | CLEAN | VP-032 module label normalized to sibling-VP convention (F-P10-001); all BC→VP trace links intact |
| 9. Process / governance | CLEAN | No stale proposed/draft status on accepted decisions; all deferred items have explicit F3/F4/F6 obligations recorded |

---

## Severity Trajectory (full)

| Pass | C | H | M | L | Notes |
|------|---|---|---|---|-------|
| P1 | 4 | 7 | 3 | 3 | LE endianness, T0846, frame-skip soundness |
| P2 | 4 | 3 | 3 | 2 | ADR EMITTED 17→20, VP-032 LE, BC batch |
| P3 | 3 | 4 | 4 | — | Dominant pattern: propagation-lag; exhaustive sweep |
| P4 | 0 | 1 | 4 | 2 | Anchor/RTM/capability residues |
| P5 | 0 | 1 | 3 | 1 | ARCH-INDEX EMITTED 17→20, PRD §6.5, SS-17 BC count 24→25 |
| P6 | 0 | 0 | 2 | 1 | **FIRST CLEAN PASS** (convergence counter started: 1/3) |
| P7 | 0 | 1 | 0 | 1 | ADR Decision 4 doc-comment strict `>` reword; counter RESET to 0/3 |
| P8 | 0 | 0 | 0 | 3 | **PASS** — all 9 axes clean; 3 LOWs deferred; counter 1/3 |
| P9 | 0 | 1 | 1 | 2 | 0x00B1 protocol bug (genuinely new); REMEDIATED via scope reduction; counter RESET |
| **P10** | **0** | **0** | **0** | **3** | **PASS** — 3 LOWs REMEDIATED; all 9 axes clean; counter **1/3** |

Convergence counter after Pass 10: **1/3**. Passes 11 and 12 required to confirm on final content.
