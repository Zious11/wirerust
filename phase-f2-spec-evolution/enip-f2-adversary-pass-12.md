---
document_type: adversarial-spec-review
pass: 12
cycle: feature-enip-v0.11.0
subsystem: SS-17
verdict: PASS
critical_count: 0
high_count: 0
medium_count: 1
low_count: 2
novelty: LOW-MEDIUM
timestamp: 2026-06-24T00:00:00Z
convergence_criterion: "3/3 passes with 0 HIGH/CRITICAL"
convergence_status: "CRITERION MET — Pass 10/11/12 all 0 HIGH/CRITICAL"
---

# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 12

## Verdict

**PASS (0 HIGH/CRITICAL) — 1 MEDIUM, 2 LOW. Novelty: LOW-MEDIUM.**

The MEDIUM was an orchestrator-induced regression from the Pass-10 polish (now fixed in
final-tidy). The LOWs are carry-forwards from Pass 11 (now resolved). All findings
REMEDIATED in the final-tidy burst prior to this pass record.

## Convergence Status

| Pass | C | H | M | L | Verdict | Counter |
|------|---|---|---|---|---------|---------|
| P6   | 0 | 0 | 2 | 1 | PASS    | 1/3 (reset by P7) |
| P7   | 0 | 1 | 0 | 1 | FAIL    | RESET to 0/3 |
| P8   | 0 | 0 | 0 | 3 | PASS    | 1/3 (reset by P9) |
| P9   | 0 | 1 | 1 | 2 | FAIL    | RESET to 0/3 |
| P10  | 0 | 0 | 0 | 3 | PASS    | 1/3 |
| P11  | 0 | 0 | 0 | 2 | PASS    | 2/3 |
| P12  | 0 | 0 | 1M-fixed | 2L-fixed | **PASS** | **3/3 — CRITERION MET** |

Severity trajectory (full): 4C/7H → 4C/3H → 3C/4H → 0C/1H → 0C/1H → 0C/0H → 0C/1H
→ 0C/0H → 0C/1H → 0C/0H → 0C/0H → **0C/0H (P12, 1M regression + LOWs all remediated)**

## Findings (All Remediated)

### F-P12-001 — MEDIUM (REMEDIATED)

**Finding:** `vp-032-enip-parse-safety.md` frontmatter `module:` field was
over-normalized to bare `analyzer/enip.rs` during the Pass-10 final-polish burst.
All 31 sibling VP frontmatters use the `src/`-prefixed convention. The regression
was introduced by the orchestrator when applying Pass-10 polish and escaped Pass-11
review because the adversary focused on document content, not YAML frontmatter
field conventions.

**Convention:** VP frontmatter `module:` field uses `src/`-prefixed paths (mirrors
the source tree). Index/coverage table `Module` cells correctly stay bare
(`analyzer/enip.rs`) — these reference module names, not file paths.

**Resolution:** `module: "analyzer/enip.rs"` → `module: "src/analyzer/enip.rs"` in
`vp-032-enip-parse-safety.md` frontmatter. Index table cell in
`verification-architecture.md` correctly retained as bare `analyzer/enip.rs`.

**File:** `.factory/specs/verification-properties/vp-032-enip-parse-safety.md`

---

### F-P11-001 — LOW (REMEDIATED, carried from Pass 11)

**Finding:** `verification-architecture.md` line ~110 VP-032 table `Module` cell
contained `src/analyzer/enip.rs` (src/ prefix outlier) while all other Kani VP
table rows use bare `analyzer/<module>.rs` notation.

**Resolution:** Reverted to `analyzer/enip.rs` (bare form, consistent with all 31
sibling VP table rows). The table `Module` column = module name, not file path.

**File:** `.factory/specs/architecture/verification-architecture.md`

---

### F-P11-002 — LOW (REMEDIATED, carried from Pass 11)

**Finding:** `BC-2.17.005` Invariant 3 DoS arithmetic illustration used
`MAX_ENIP_CARRY_BYTES = 600` as if it were the CPF payload length cap. The correct
figure is `header.length ≤ 576` (because `24 + header.length > 600` triggers
frame-walk rejection, so CPF payload ≤ 576 bytes), giving `(576 - 2) / 4 = 143`
max items, not 149 from the carry-cap figure.

**Resolution:** Rewritten to correctly derive payload bound from frame-walk gate:
CPF payload = `data[24 .. 24 + header.length]`; frames where `24 + header.length
> 600` rejected; therefore `payload.len() ≤ 576`; bound = `(576 - 2) / 4 = 143`.
Also clarified that the 600-byte carry cap bounds the carry buffer, not the CPF
payload slice.

**File:** `.factory/specs/behavioral-contracts/ss-17/BC-2.17.005.md`

---

### F-P12-002 — LOW (REMEDIATED)

**Finding:** `BC-2.17.024` Postcondition 4 named `RegisterSession` and
`IndicateStatus` as exemplars of no-finding commands but did not explicitly list all
four no-finding commands from v0.11.0 scope: ListServices (0x0004),
ListInterfaces (0x0064), IndicateStatus (0x0072), and Cancel (0x0075). These four
commands are validity-gated and PDU-counted but emit no finding in v0.11.0 (no MITRE
ICS detection target in current scope). The gap risked implementers mistakenly
expecting a finding emission for these commands.

**Resolution:** Added Postcondition 5 explicitly listing all four no-finding commands
with their hex codes, BC cross-references (BC-2.17.003 for validity gate,
BC-2.17.004 for classification), and rationale (no MITRE ICS detection target in
v0.11.0 scope).

**File:** `.factory/specs/behavioral-contracts/ss-17/BC-2.17.024.md`

---

### F-P12-003 — LOW (REMEDIATED)

**Finding:** `BC-2.17.009` Edge Case EC-006 table row was formatted as a 4-cell
entry with a stray `|` separator in the `Expected Behavior` column, causing the
table to render with a phantom 4th column. The description was also ambiguous about
what "declares 3 segments" means when the path has only 4 bytes.

**Resolution:** Reformatted EC-006 to a clean 3-cell table row (ID | Description |
Expected Behavior) with a precise description: cursor advances to byte 4 after two
segments; `cursor+2 > 4` triggers break; returns 2 segments (or fewer if a byte
was an unrecognized type).

**File:** `.factory/specs/behavioral-contracts/ss-17/BC-2.17.009.md`

---

## 9-Axis Cleanliness Review (Pass 12)

| Axis | Status | Notes |
|------|--------|-------|
| 1. Endianness completeness | CLEAN | LE anchored throughout all BCs; ADR-010 Decision 2 normative |
| 2. CIP table 13/15 command set | CLEAN | 9 commands in VP-032 Sub-B/C match ADR-010 Decision 3; BC-2.17.003/004 consistent |
| 3. MITRE EMITTED count (17→20/28/8) | CLEAN | ARCH-INDEX v1.8 emitted=20; BC-INDEX finding-tags consistent |
| 4. Frame-walk carry-cap soundness | CLEAN | 600-byte carry cap correctly separates from 576-byte CPF payload cap (F-P11-002 fixed) |
| 5. Holdout scenario coverage | CLEAN | 9 holdout axes verified in Pass-11 adversary review |
| 6. Kani non-vacuity (DF-KANI-NONVACUITY-001) | CLEAN | VP-032 Sub-B/C use biconditional; Sub-D primary+partition; Sub-A implicit by unwind(49) |
| 7. Strict `>` threshold semantics | CLEAN | BC-2.17.012/023/025 all use strict `>` per ADR-010 Decision 4 rationale |
| 8. 0x00B2 gating (v0.11.0 scope) | CLEAN | BC-2.17.006/011/015 updated Pass-9; 0x00B1 deferred via ADR-010 Decision 8 |
| 9. BC-completeness (25 BCs SS-17) | CLEAN | BC-2.17.001..025; BC-INDEX v1.75 (330/329 active) |

All 9 axes confirmed clean by Pass 12. Convergence criterion met.

## Final-Tidy Summary

The final-tidy burst applied the following edits to bring the spec to gate-ready state:

| File | Change |
|------|--------|
| `vp-032-enip-parse-safety.md` | Restored `module: "src/analyzer/enip.rs"` frontmatter (F-P12-001 regression fix) |
| `verification-architecture.md` | Reverted VP-032 table Module cell to bare `analyzer/enip.rs` (F-P11-001 fix) |
| `BC-2.17.005.md` | Rewrote Invariant 3 with correct 576/143 figures (F-P11-002 fix) |
| `BC-2.17.009.md` | Fixed EC-006 table formatting 4-cells→3-cells (F-P12-003 fix) |
| `BC-2.17.024.md` | Added Postcondition 5 for 4 no-finding commands (F-P12-002 fix) |

## Next Step

Pass 13: post-tidy confirmation pass — verify no regressions introduced by final-tidy,
confirm 3/3 criterion sustained. F2 human gate follows Pass-13 PASS.

**F2 human gate items to confirm (unchanged from Pass 11):**
1. 0x00B1 CIP request detection scope deferral to v0.12.0 (ADR-010 Decision 8).
2. `--enip-write-burst-threshold` default = 50 writes/1s (OA-001).
3. `ENIP_ERROR_BURST_THRESHOLD` = 5 consecutive errors (OA-001).
4. F-P2-010: SS-10 BC version-bump pending — resolve before F3 entry.
