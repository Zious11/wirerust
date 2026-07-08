---
run: maint-2026-07-08
producer: consistency-validator
date: 2026-07-08
spec_versions:
  BC-INDEX: v2.20
  VP-INDEX: v2.35
  ARCH-INDEX: v2.12
  PRD: v1.51
  STORY-INDEX: v3.23
  HS-INDEX: v2.12
  dependency-graph: v3.7
  epics: v2.1
---

# Spec Coherence Findings — maint-2026-07-08

## Summary Table

| Check | Result | Finding Count |
|-------|--------|---------------|
| L1→L4 chain integrity | PASS | 0 |
| BC coverage completeness | PASS (tally note) | 1 |
| VP alignment with BCs | PASS | 0 |
| Story-to-BC mapping | PASS (tally note) | 1 (shared with coverage) |
| ARCH-INDEX subsystem counts vs actual | PASS | 0 |
| Known open items (7 verified) | 3 correctly OPEN / 2 stale-OPEN / 1 description-inaccurate / 1 correctly deferred | 3 stale entries |

NEW findings not in STATE.md backlog: **5**

---

## Dimension 1 — L1→L4 Chain Integrity

Checked: PRD v1.51 traces_to prd.md in BC-INDEX v2.20; ARCH-INDEX v2.12 traces_to prd.md; VP-INDEX v2.35 traces_to ARCH-INDEX; STORY-INDEX v3.23 traces_to epics.md + dependency-graph.md v3.7; epics.md v2.1 traces_to prd.md + BC-INDEX + ARCH-INDEX. Version numbers declared in STATE.md match files on disk for all eight indexed artifacts. No broken chain links found.

**Result: PASS. 0 findings.**

---

## Dimension 2 — BC Coverage Completeness

BC-INDEX v2.20 reports 346 on disk, 345 active (1 retired: BC-2.01.004). Per-subsystem file counts verified against ARCH-INDEX registry for all 17 active subsystems — all counts match exactly. All 345 active BCs are assigned to stories per STORY-INDEX v3.23 coverage note. Input-hash scan (2026-07-08): MATCH=111, STALE=0.

**NEW-FINDING-1 (STORY-INDEX-TALLY-DRIFT-001):** STORY-INDEX v3.23 explicit coverage tally line reads "335 BCs" but BC-INDEX active = 345. Gap of 10: the tally omits BC-2.01.009..018 net of retired BC-2.01.004 (9 pcapng reader BCs added at BC-INDEX v2.4) and BC-2.16.016 (ARP unbounded-cap, added maint-2026-07-06). These BCs are covered by stories per prose description (STORY-123..128, STORY-156) but are excluded from the arithmetic tally line. This is an arithmetic tally issue only — no actual coverage gap exists.

**Result: PASS WITH NOTE. 1 finding (STORY-INDEX-TALLY-DRIFT-001 — arithmetic tally only, no real uncovered BCs).**

---

## Dimension 3 — VP Alignment with BCs

VP-INDEX v2.35: total_vps=43 (kani=15, proptest=20, fuzz=2, integration_unit=6), p0_count=8, p1_count=29, test_sufficient_count=6. Spot-checked VP-BC cross-references for all recently-added VPs: VP-039 (TLS carry reassembly, BC-2.07.038..042), VP-040 (buffer saturation, BC-2.07.043), VP-041/042/043 (protocol coverage, BC-2.18.003, BC-2.05.010, BC-2.05.011). All references consistent. VP-042 covers exactly 3 harnesses A/B/C (VP042D residual resolved at D-375, 2026-07-04). No drift found.

**Result: PASS. 0 findings.**

---

## Dimension 4 — Story-to-BC Mapping

STORY-INDEX v3.23: 111 stories, 102 delivered, 71 waves, 705 total points (680 in wave-table, 25 in wave-TBD/superseded). STORY-148 status is `superseded` in both STORY-INDEX row and story file. All wave-71 stories (STORY-150/156/157) show DELIVERED and CLOSED. No uncovered BCs found (all 345 active BCs assigned per coverage note).

**Tally note: same as Dimension 2 (STORY-INDEX-TALLY-DRIFT-001).**

**Result: PASS WITH NOTE. See STORY-INDEX-TALLY-DRIFT-001.**

---

## Dimension 5 — ARCH-INDEX Subsystem Counts vs Actual Registry

ARCH-INDEX v2.12 lists 17 active subsystems. File counts from disk compared against ARCH-INDEX registry:

| Subsystem | ARCH-INDEX Count | Actual BC Files | Match |
|-----------|-----------------|-----------------|-------|
| SS-01 | 17 | 17 active (19 files, 1 non-BC doc + 1 retired on disk) | MATCH |
| SS-02 | 14 | 14 | MATCH |
| SS-04 | 8 | 8 | MATCH |
| SS-05 | 11 | 11 | MATCH |
| SS-06 | 18 | 18 | MATCH |
| SS-07 | 43 | 43 | MATCH |
| SS-08 | 14 | 14 | MATCH |
| SS-09 | 15 | 15 | MATCH |
| SS-10 | 17 | 17 | MATCH |
| SS-11 | 35 | 35 | MATCH |
| SS-12 | 24 | 24 | MATCH |
| SS-13 | 10 | 10 | MATCH |
| SS-14 | 19 | 19 | MATCH |
| SS-15 | 22 | 22 | MATCH |
| SS-16 | 16 | 16 | MATCH |
| SS-17 | 9 | 9 | MATCH |
| SS-18 | 4 | 4 | MATCH |

Total active BCs across all subsystems = 345, consistent with BC-INDEX v2.20 header.

**Result: PASS. 0 findings.**

---

## Dimension 6 — Known Open Items: Verified Current State

### 6.1 EPICS-TOTAL-BCS-DRIFT-001

STATE.md status: OPEN — maintenance/phase-5 (DF-VALIDATION-001-gated).

**Verified:** epics.md v2.1 frontmatter `total_bcs: 337`. BC-INDEX v2.20 active = 345. Gap = 8.

Gap decomposition: E-5 Per-Epic BC row lists BC-2.07.001..037 = 37 BCs, missing BC-2.07.038..043 (6 TLS carry-reassembly BCs added fix-tls-clienthello-frag F3, 2026-06-29). E-8 arithmetic off by 1: BC-2.11.035 (MITRE ATT&CK OT cross-ref, added maint-2026-06-22) not reflected in E-8 row total. E-16 Per-Epic BC row lists BC-2.16.001..015 = 15 BCs, missing BC-2.16.016 (ARP unbounded-cap, added maint-2026-07-06). Arithmetic: 337 + 6 + 1 + 1 = 345. Gap fully accounts for all 8 missing BCs.

**Current state: OPEN. Correctly tracked. No change.**

### 6.2 INPUT-HASH-ERROR-STORIES-001

STATE.md status: OPEN — maintenance backlog (DF-VALIDATION-001-gated).

STATE.md description: "STORY-001 retired-BC input reference (→BC-2.01.009); STORY-091/STORY-121 missing inputs: block."

**Verified:** Input-hash scan 2026-07-08: MATCH=111, STALE=0, ERROR=0.

- STORY-001: `inputs:` block present. Includes `BC-2.01.004.md  # retired` with inline comment. `bin/compute-input-hash` (updated by STORY-157, wave-71, 2026-07-07) strips the ` # retired` suffix. Hash stored = `4ae9f11`, computed = `4ae9f11`. MATCH.
- STORY-091: `inputs: []` (empty list present — not "missing"). Hash stored = `d41d8cd`. MATCH.
- STORY-121: `inputs: []` (empty list present — not "missing"). Hash stored = `d41d8cd`. MATCH.

**Current state: OPEN in backlog, but description is inaccurate and the errors no longer exist.** The "ERROR=3" referenced in D-375 (2026-07-04) was with the pre-STORY-157 tool. See NEW-FINDING-3.

### 6.3 STORY-148-BASIS-RESOLVED-001

STATE.md backlog row (line 327) status: OPEN — story-writer must reconcile STORY-148.

**Verified:** D-399 (2026-07-07): "STORY-148-BASIS-RESOLVED-001 CLOSED." STORY-148.md `status: superseded`. STORY-INDEX v3.23 row = `superseded`. The finding is closed in the D-log but the backlog row was not updated. See NEW-FINDING-4.

**Current state: CLOSED per D-399, but backlog row still shows OPEN. Stale backlog entry.**

### 6.4 TLS-SUMMARIZE-MAPTYPE-001

STATE.md status: "Deferred — spec-only gap."

**Verified:** BC-2.07.043 PC-4 line 79 still uses `HashMap<String, Value>` as the return type wording for `TlsAnalyzer::summarize()`. Line 131 in the same file references BTreeMap (the implementation type). VP-040 Sub-D wording still references HashMap. No BC version update since the original deferral.

**Current state: OPEN/DEFERRED. Correctly tracked as spec-only gap. No change.**

### 6.5 BC-2.05.010-EC006-UNREACHABLE-001

STATE.md status: OPEN — phase-5 BC reconciliation (DF-VALIDATION-001-gated).

**Verified:** BC-2.05.010 EC-006 text still claims "(Tcp, 502) count == 2" is reachable. `classify()` Rule 5 always routes `(Tcp, 502)` to `DispatchTarget::Modbus`, making the None-target arm physically unreachable for this pair. Stories navigate correctly (tests use `--modbus` flag so Rule-5 routing fires). BC spec remains inaccurate.

**Current state: OPEN. Correctly tracked. DF-VALIDATION-001-gated. No change.**

### 6.6 BC-2.12.024-PRIORITY-SKEW-001

STATE.md status: OPEN — phase-5 priority reconciliation.

**Verified:** BC-2.12.024 listed as P1 in BC-INDEX v2.20. Holdouts HS-128..131 are P0 must-pass. STORY-154 treats BC-2.12.024 as Primary with mandatory canonical ACs. Priority skew between BC-INDEX and holdouts/story decomposition unreconciled.

**Current state: OPEN. Correctly tracked. No change.**

### 6.7 F-F3P18-O1 (fresh-context perimeter check — not in original six)

STATE.md backlog row (line 310) status: OPEN — phase-5 spec-wording reconciliation.

**Verified:** BC-2.12.024.md is at v1.2 on disk. D-375 (2026-07-04): "BC-2.12.024 v1.2 (PC-4 phantom `supported:` replaced with derived predicate)." Backlog row for BC-2.12.024-PC4-PHANTOM-SUPPORTED-001 (line 323) = "RESOLVED (D-375, 2026-07-04)." F-F3P18-O1 and BC-2.12.024-PC4-PHANTOM-SUPPORTED-001 describe the same issue; only one row was updated to RESOLVED. See NEW-FINDING-5.

**Current state: RESOLVED per D-375, but F-F3P18-O1 backlog row still shows OPEN. Stale backlog entry.**

---

## NEW Findings Not in STATE.md Backlog

### NEW-FINDING-1: STORY-INDEX-TALLY-DRIFT-001

**Artifact:** STORY-INDEX v3.23 (coverage tally line)
**Severity:** LOW
**Description:** The explicit coverage tally line reads "335 BCs" but BC-INDEX v2.20 active = 345. Gap of 10: 9 pcapng reader BCs (BC-2.01.009..018 net of retired BC-2.01.004, added BC-INDEX v2.4) and BC-2.16.016 (added maint-2026-07-06) are mentioned in prose coverage note but omitted from the arithmetic tally. No actual coverage gap; purely an arithmetic stale in the tally line.
**Recommended action:** Update tally line to "345 BCs" in a maintenance sweep. No DF-VALIDATION-001 gate required (description-only update, no spec content change).

### NEW-FINDING-2: EPICS-STORY-COUNT-DRIFT-001

**Artifact:** epics.md v2.1 (Estimated Story Count Summary table)
**Severity:** LOW
**Description:** Summary table shows Total=107, last reconciled to STORY-INDEX v3.12 (F3 convergence, 2026-07-02). STORY-INDEX is now v3.23 with 111 stories (+4). Specific drift:
- E-11 row: 6 stories listed, actual = 9 in STORY-INDEX (STORY-155/157/158 added waves 70-71 not reflected).
- E-16 row: 5 stories listed, actual = 6 (STORY-156 added wave 71 not reflected).
- Grand total drift: 107 → 111 (+4 stories).

Related to EPICS-TOTAL-BCS-DRIFT-001 (same artifact, same maintenance scope).
**Recommended action:** Route to same maintenance sweep as EPICS-TOTAL-BCS-DRIFT-001. DF-VALIDATION-001-gated per same policy scope as parent finding.

### NEW-FINDING-3: INPUT-HASH-DESCRIPTION-DRIFT-001

**Artifact:** STATE.md backlog row for INPUT-HASH-ERROR-STORIES-001
**Severity:** LOW (cosmetic/description accuracy)
**Description:** Backlog description says "STORY-091/STORY-121 missing inputs: block." Both files have `inputs: []` — the field is present, just empty. The ERROR=3 in D-375 (2026-07-04) was caused by the pre-STORY-157 tool that could not handle inline comments in path entries (STORY-001's `BC-2.01.004.md  # retired`). STORY-157 (wave-71, PR #380, 2026-07-07) added inline-comment stripping, resolving the final error. Current scan: MATCH=111, STALE=0, ERROR=0.
**Recommended action:** Update INPUT-HASH-ERROR-STORIES-001 backlog description to reflect actual state: (a) STORY-091/121 `inputs: []` present (not missing); (b) STORY-001 inline-comment handling resolved by STORY-157. Keep entry as informational record or mark RESOLVED.

### NEW-FINDING-4: STATE-BACKLOG-STALE-STORY-148-001

**Artifact:** STATE.md backlog row for STORY-148-BASIS-RESOLVED-001 (line 327)
**Severity:** LOW (bookkeeping)
**Description:** Backlog row status = "OPEN — story-writer must reconcile STORY-148." D-399 (2026-07-07) explicitly records "STORY-148-BASIS-RESOLVED-001 CLOSED." STORY-148.md `status: superseded`. STORY-INDEX v3.23 row = `superseded`. The D-log closure was not propagated to the backlog row.
**Recommended action:** Update backlog row status to "RESOLVED (D-399, 2026-07-07) — STORY-148 superseded; on_flow_close wiring code-verified in develop PR #362."

### NEW-FINDING-5: STATE-BACKLOG-STALE-F-F3P18-O1

**Artifact:** STATE.md backlog row for F-F3P18-O1 (line 310)
**Severity:** LOW (bookkeeping)
**Description:** Backlog row status = "OPEN — phase-5 spec-wording reconciliation." The underlying issue (BC-2.12.024 PC-4 phantom `supported:` field) was resolved at D-375 (2026-07-04) when BC-2.12.024 was updated from v1.1 to v1.2. A co-located backlog row for BC-2.12.024-PC4-PHANTOM-SUPPORTED-001 (line 323) correctly shows "RESOLVED (D-375, 2026-07-04)." The two rows track the same issue; only one was updated. BC-2.12.024.md v1.2 is confirmed on disk with derived predicate notation.
**Recommended action:** Update F-F3P18-O1 backlog row status to "RESOLVED (D-375, 2026-07-04) — same issue as BC-2.12.024-PC4-PHANTOM-SUPPORTED-001; BC-2.12.024 v1.2 applied."

---

## Perimeter Check (Fresh-Context Mandate)

Additional backlog items spot-checked for accuracy:

- **STORY-154-TESTCOUNT-COMMENT-001** (line 321): OPEN. `tests/integration_tests.rs` ~line 1161 comment "All 20 tests pass" stale (21 tests since D-371). Correctly tracked. No change.
- **HS-INDEX-ENIP-WAVE-DRIFT-001** (line 297): OPEN — DF-VALIDATION-001-gated. HS-INDEX lists E-20 waves 63-68 / STORY-131..141 but dep-graph E-20 shows waves 58-61. Correctly tracked. No change.
- **BC-2.12.024-PC4-PHANTOM-SUPPORTED-001** (line 323): RESOLVED (D-375, 2026-07-04). Correctly tracked. (Sibling F-F3P18-O1 is stale — see NEW-FINDING-5.)
- **Input-hash scan 2026-07-08:** MATCH=111, STALE=0. All 111 stories clean under canonical Python tool.

No additional HIGH or CRITICAL findings discovered in perimeter check.
