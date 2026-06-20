---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
phase: F2
pass: 7
date: 2026-06-20
reviewer: adversary (fresh context)
inputs:
  - ADR-009 rev 9
  - error-taxonomy v3.4
  - VP-INDEX v2.8
  - HS-101..108
  - BC-2.01.009..018 (post-D-156/D-157 versions)
verdict: NOT CLEAN
finding_counts:
  critical: 1
  high: 3
  medium: 4
  low: 4
  total: 12
novelty: MODERATE
convergence_axes_clean: 2
trajectory: "P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13 / P7:12"
decision: D-158
---

# F2 Adversarial Spec Review — Pass 7

**Reviewer:** adversary (fresh context — no prior pass memory)
**Inputs at review time:** ADR-009 rev 9, error-taxonomy v3.4, VP-INDEX v2.8, HS-101..108
**Verdict:** NOT CLEAN
**Finding counts:** 1 CRITICAL / 3 HIGH / 4 MEDIUM / 4 LOW — total 12
**Novelty:** MODERATE (down from HIGH; two convergence axes confirmed CLEAN)
**Trajectory:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13 / P7:12

## Convergence Status

Two axes CONVERGED (confirmed CLEAN this pass, no new findings):
- **SPB body.len()-4 arithmetic** (Decision 22 / VP-031 formula) — CONVERGED
- **VP-INDEX self-consistency** (total_vps=31, count propagation) — CONVERGED

Remaining open cluster: **OPB zero-packet-notice subsystem** (rev-8/9 propagation lag from H-2 pass-5) and **symbol-rename incompleteness** (block_body_available → spb_data_available not fully propagated).

---

## Critical Findings

### F-1 [CRITICAL] — OPB counter model contradiction between BC-2.01.015 and HS-108

**Location:** BC-2.01.015 PC9/AC-003/AC-006 vs HS-108 Case D/Case E

**Description:** BC-2.01.015 PC9 (pass-5 H-2 fix, D-153) established a dual-counter model:
- `skipped_blocks:u32` — total blocks skipped (ALL skip types, including OPBs)
- `opb_skipped:u32` — sub-counter for obsolete Packet Blocks specifically
- Invariant: `opb_skipped <= skipped_blocks` (OPBs are a subset of total skips)
- AC-003: OPB skip arm INCREMENTS BOTH counters
- AC-006: main.rs emits notice using BOTH fields

HS-108 Cases D and E contradict the BC invariant:
- **Case D** (OPB-only 3 OPBs): `skipped_blocks=0, opb_skipped=3` — this makes `opb_skipped > skipped_blocks`, violating the subset invariant
- **Case E** (2 NRBs + 1 OPB): `skipped_blocks=2, opb_skipped=1` — this counts NRBs in skipped_blocks but NOT the OPB, making `skipped_blocks` the non-OPB count rather than the total count

The holdout and the BC are mutually exclusive: a test suite written against the BC will FAIL HS-108 Case D (skipped_blocks would be 3, not 0), and HS-108 Case E (skipped_blocks would be 3, not 2). The holdout CANNOT pass against its governing BC.

**Root cause:** HS-108 was authored in D-150 (pass-4 H-4) before the opb_skipped sub-counter was added in D-153 (pass-5 H-2). D-153 rewrote PC9 and added AC-003/AC-006 but did not update HS-108 Cases D/E to reflect the new counter semantics.

**Fix:** Keep the BC-2.01.015 "both" counter model (it is correct). Update HS-108:
- Case D: 3 OPBs → `skipped_blocks=3, opb_skipped=3` (OPBs count as skipped_blocks too)
- Case E: 2 NRBs + 1 OPB → `skipped_blocks=3, opb_skipped=1` (NRBs + OPB all in total)

---

## High Findings

### F-2 [HIGH] — HS-108 uses non-existent field `obsolete_packet_blocks` (6 occurrences)

**Location:** HS-108 Cases D/E — field name in expected output / notice format assertions

**Description:** HS-108 (v1.3 as of D-156/D-157) refers to a field `obsolete_packet_blocks` six times in Case D and Case E expected outputs and notice format assertions. The canonical field name established in D-153 (BC-2.01.015 v1.6 AC-003 and BC-2.01.009 v1.5) is `opb_skipped`. The name `obsolete_packet_blocks` never existed in any BC version; it is a stale draft artifact from HS-108 v1.0 (D-150) that was not updated when PC9 introduced the canonical naming in D-153.

An implementation coded to the BCs would use `opb_skipped`; an implementation coded to HS-108 would attempt to access a non-existent field, failing to compile or producing a wrong field lookup. The holdout test vectors are therefore incorrect and would not validate a correct implementation.

**Fix:** In HS-108 Cases D and E, replace all 6 occurrences of `obsolete_packet_blocks` with `opb_skipped`.

---

### F-3 [HIGH] — Zero-packet notice DISPLAY arithmetic undefined: generic skip segment unspecified

**Location:** BC-2.01.009 PC6 / BC-2.01.015 PC9 / HS-108 Case E

**Description:** BC-2.01.009 PC6 (pass-5 M-5 fix, D-153) defines the canonical notice format:
```
notice: <filename>: 0 packets read from pcapng file (N blocks skipped; M obsolete packet blocks)
```
where N = `skipped_blocks` (total) and M = `opb_skipped`.

When `opb_skipped > 0` AND `skipped_blocks > opb_skipped` (mixed case: some non-OPB skips + some OPB skips), the notice must display:
- A "generic" skip segment (non-OPB blocks) = `skipped_blocks - opb_skipped`
- An OPB segment = `opb_skipped`

BC-2.01.009 PC6 only specifies appending an OPB clause when `opb_skipped > 0`; it does NOT specify subtracting `opb_skipped` from the generic count to produce the non-OPB display value. The formula for the "N blocks skipped" segment is ambiguous: does N = `skipped_blocks` (total including OPBs) or N = `skipped_blocks - opb_skipped` (non-OPB skips only)?

HS-108 Case E (2 NRBs + 1 OPB) shows a notice with "2 generic + 1 OPB" format, but this is not derivable from the BC spec as written without knowing the subtraction convention. The current spec would allow implementations where N = 3 (total) with the OPB clause appended, producing a misleading "3 blocks skipped (incl. 1 OPB)" notice that double-counts the OPB.

**Fix:** BC-2.01.009 PC6: explicitly define the generic-segment display value as `(skipped_blocks - opb_skipped)`, emitted only when `> 0`. Full notice format (normalized):
- `skipped_blocks > 0, opb_skipped == 0`: `"N blocks skipped"` (N = skipped_blocks)
- `skipped_blocks == opb_skipped > 0`: `"M obsolete packet blocks skipped"` (OPB-only)
- `skipped_blocks > opb_skipped > 0`: `"G blocks skipped; M obsolete packet blocks skipped"` (G = skipped_blocks - opb_skipped)

---

### F-4 [HIGH] — EPB decode precedence unpinned; Precondition 1 contradicts PC5a

**Location:** BC-2.01.012 Precondition 1 / PC5a / EC-006

**Description:** BC-2.01.012 (EPB parsing, v1.6 as of D-156) has two issues:

**Issue 4a — Precondition 1 contradicts PC5a:**
Precondition 1 states: "The interface table is non-empty (at least one IDB has been parsed before the first EPB)." PC5a then defines the behavior when the interface table IS empty: return E-INP-009. These two clauses are in direct logical conflict — a precondition asserts the condition cannot occur; a postcondition specifies behavior when it does. A precondition violation is undefined behavior; PC5a makes the behavior defined. One of the two must be removed: if it is truly a precondition (caller obligation), PC5a is dead spec; if PC5a defines required behavior, Precondition 1 is wrong.

**Issue 4b — EPB decode step ordering (precedence) not pinned:**
The BC specifies: body.len() guard, interface_id read, empty-table check, OOB check, captured_len/padding check — but the ORDER in which these checks fire is not stated as a formal precedence list. An implementation that checks empty-table before body.len() would return E-INP-009 rather than E-INP-008 for a truncated EPB with no IDB. The existing EC-006 / EC-007 examples assume a specific ordering but do not assert it as a postcondition constraint.

**Fix:**
- Remove Precondition 1's "non-empty" assertion (contradicted by PC5a).
- Add a postcondition to BC-2.01.012 making the decode precedence explicit:
  1. `body.len() >= 20` — else E-INP-008 (body-too-short)
  2. Read `interface_id` from body[0..4]
  3. `interface_table.is_empty()` — else E-INP-009
  4. `interface_id >= interface_table.len()` on non-empty table — else E-INP-010
  5. `captured_len` bound + padding check — else E-INP-008

---

## Medium Findings

### F-5 [MEDIUM] — HS-107 still uses retired symbol `block_body_available` alongside canonical `spb_data_available`

**Location:** HS-107 Cases A/B/C/D/E — field name in assertions / rationale prose

**Description:** Decision 22 (ADR-009 rev 9, D-156) established the canonical symbol rename: the data-bytes quantity for SPB is `spb_data_available = body.len() - 4` (not the pre-rev-9 `block_body_available = btl - 16`). BC-2.01.013 v1.6 was updated to use `body.len()-4` consistently. However HS-107 v1.5 (post-D-156) retains `block_body_available` in several case rationales and assertion expressions, numerically equal to `body.len()-4` but using the retired symbol. An implementation coded to BC-2.01.013 v1.6 would expose `spb_data_available`; the holdout assertions using `block_body_available` would not match.

**Fix:** Rename `block_body_available` → `spb_data_available` throughout HS-107 (all cases where the symbol appears in assertions or rationale arithmetic).

---

### F-6 [MEDIUM, process-gap] — HS-104/107/108 input-hash "tbd" disables drift detection on must-pass holdouts

**Location:** HS-104 / HS-107 / HS-108 frontmatter `input-hash:` field

**Description:** HS-104, HS-107, and HS-108 carry `input-hash: tbd` in their frontmatter. Per DF-INPUT-HASH-CANONICAL-001 (Lesson 3), input-hash must be computed before F3 story decomposition. These three holdout scenarios are the must-pass scenarios for the F2 convergence gate (EPB interface_id discriminant, SPB framing, zero-packet notice). An incorrect input-hash (or "tbd") means that if their governing BCs change without the holdout being regenerated, the drift tripwire does not fire.

Additionally, Decisions 15/17/19/20/22 (introduced across passes 2-6) are the BCs' governing decisions; ADR-009 is not listed as a holdout input for HS-104/107/108 even though the cases were authored WITH reference to these decisions. A future Decision revision would not trigger holdout regeneration.

**Fix:** Run `bin/compute-input-hash --write` on HS-104, HS-107, HS-108. Add ADR-009 to each holdout's `inputs:` list (alongside the referenced BCs) so that decision revisions invalidate the holdout hash.

---

### F-7 [MEDIUM] — BC-2.01.013 EC-001/002/003 + Canonical Test Vectors use retired `block_body_available`

**Location:** BC-2.01.013 v1.6 — EC-001, EC-002, EC-003, and "Canonical Test Vectors" section

**Description:** BC-2.01.013 v1.6 (post-D-156 Decision 22 fix) updated PC1/AC-002/EC-007 and the block-level formula to use `body.len()-4`. However, the version changelog claimed "block_body_available=body.len()-4 everywhere" but EC-001, EC-002, EC-003, and the Canonical Test Vectors table still use `block_body_available` in their assertions/prose. The earlier changelog entry (v1.6 changelog) stated full rename was complete, but the EC/test-vector sections lagged. This is the same changelog-lie defect class as pass-3 C-1 (Lesson 8 / DF-CHANGELOG-DISK-VERIFY-001).

**Fix:** Rename `block_body_available` → `spb_data_available` in BC-2.01.013 EC-001/002/003 and Canonical Test Vectors.

---

### F-8 [MEDIUM] — BC-2.01.013 lacks the btl=14 misaligned crate-rejection fixture

**Location:** BC-2.01.013 — Canonical Test Vectors / EC-005

**Description:** HS-107 Case E uses `btl=14` to test the alignment-violation rejection path (14 % 4 = 2, pcapng requires 4-byte-aligned btl → crate rejects → E-INP-010). The HS-107 Case E rationale was corrected in D-156 F-M1 to state the alignment reason. However, BC-2.01.013 EC-005 (crate-framing rejection example) only illustrates `btl=8` (which is < 12, below minimum), not the alignment case. A reader of the BC alone cannot derive the Case E scenario; the alignment-violation class is only visible in HS-107.

The BC should illustrate at least two constructible E-INP-010 rejection paths: (a) btl<12 (too short), and (b) btl misaligned (4-byte alignment violation at valid length >= 12). Without a BC example, the alignment path has no normative coverage — only a holdout that references a BC lacking the case.

**Fix:** Add a btl=14 misaligned-alignment-violation → E-INP-010 example to BC-2.01.013 EC-005 (or as EC-005b) alongside the existing btl=8 example.

---

## Low Findings

### F-L1 [LOW] — VP-INDEX total count and formula GREEN (CONVERGED)

VP-INDEX v2.8 total_vps=31 is correct and self-consistent. Count propagation sweep: VP-INDEX, BC-INDEX, error-taxonomy next_free all agree. This axis is CONVERGED — no action required.

---

### F-L2 [LOW] — SPB formula GREEN (CONVERGED)

Decision 22 / BC-2.01.013 PC1 / VP-031 formula `min(original_len, body.len()-4)` is internally consistent across BC-2.01.013 v1.6, VP-031, and ADR-009 rev 9. This axis is CONVERGED — no action required.

---

### F-L3 [LOW] — Section-wide endianness covers option code/length GREEN

BC-2.01.011 TLV option parsing (option_code u16 LE, option_length u16 LE) is correctly specified throughout. The BOM-Canonical anchor pattern (Lesson 13, BC-2.01.010 §BOM-Canonical) is in place. No new findings here.

---

### F-L4 [LOW] — BC-2.01.018 VP-030/STORY-128 anchor note (pending-intent — likely intentional)

BC-2.01.018 carries a "STORY-128 pending" note for per-file-isolation AC-002. STORY-128 is listed in the Decisions Log (D-142) and in STATE.md F3 entry checklist. This is a tracked deferral per Lesson 11 — not a defect. Flagged informational only.

---

## Pass-7 Summary

| Axis | Status |
|------|--------|
| SPB body.len()-4 arithmetic (Decision 22 / VP-031) | CONVERGED |
| VP-INDEX self-consistency (total_vps=31) | CONVERGED |
| OPB counter model (BC-2.01.015 PC9 vs HS-108 D/E) | OPEN — F-1 CRITICAL, F-2 HIGH, F-3 HIGH |
| Symbol rename completeness (block_body_available → spb_data_available) | OPEN — F-5 MED, F-7 MED |
| EPB decode precedence / Precondition 1 contradiction | OPEN — F-4 HIGH |
| Holdout input-hash "tbd" (drift detection disabled) | OPEN — F-6 MED [process-gap] |
| BC-2.01.013 btl=14 alignment fixture | OPEN — F-8 MED |

**Clean-pass counter: 0/3.**
**Convergence approaching** — findings localized to OPB-notice subsystem (rev-8/9 propagation lag) + symbol-rename lag. Remediation round-7 pending.
