---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-12T23:45:00Z
feature: arp-analyzer
cycle: feature-arp-v0.7.0
phase: F2-spec-evolution
inputs: [adversarial-reviews/]
input-hash: TBD
traces_to: STATE.md
---

# Convergence Trajectory — ARP Analyzer F2 Spec Evolution

## Feature Context

**Feature:** ARP security analyzer + etherparse 0.16→0.20.1 migration (sub-delta A).
**Release target:** v0.7.0.
**F2 Scope:** SS-16 behavioral contracts (est. 18-24 new BCs), ADR-008 (DecodedFrame integration),
VP-024, BC-2.02.009 revision, holdout scenarios HS-W38+.
**MITRE:** T0830 ICS AiTM (primary) + T1557.002 Enterprise ARP Cache Poisoning (secondary) —
validated ATT&CK v19.1 (D-066).

## Adversarial Method

**SLICED method** (user-directed; 4 parallel fresh-context slices per pass):

| Slice | Scope |
|-------|-------|
| A | Architecture / verification (ADR-008, VP-024, etherparse migration invariants) |
| B | BC detection semantics (spoof, GARP, storm, rate anomaly — precision/recall correctness) |
| C | MITRE / taxonomy / holdout / catalogue-BCs (ATT&CK mapping, HS alignment, BC-INDEX arithmetic) |
| D | Cross-doc consistency (PRD ↔ BC-INDEX ↔ BC files ↔ ADR-008 ↔ VP-024 — propagation hygiene) |

Minimum 3 consecutive clean passes required for convergence gate (same as F5 standard).

## Finding Progression

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|---------|---------|
| 1 (monolithic) | 2026-06-12 | 15 | 4 | 8 | 3 | 0 | HIGH | 0/3 | NOT_CLEAN |
| 2 (sliced) | 2026-06-12 | 20 | 5 | 7 | 8 | 0 | HIGH | 0/3 | NOT_CLEAN |
| 3 (sliced) | 2026-06-12 | ~8 | 0 | ~6 | ~2 | 0 | MED | 0/3 | NOT_CLEAN |
| 4 (sliced) | 2026-06-12 | ~15 | 0 | ~5 | ~10 | 0 | LOW | 0/3 | NOT_CLEAN |
| 5 (sliced) | 2026-06-12 | ~6 | 0 | 1 | ~5 | 0 | LOW | 0/3 | NOT_CLEAN |
| 6 (sliced) | 2026-06-12 | ~4 | 0 | 2 | 2 | 0 | LOW | 0/3 | NOT_CLEAN |
| 7 (sliced) | 2026-06-12 | ~4 | 0 | ~4 | 0 | 0 | LOW | 0/3 | NOT_CLEAN |
| 8 (sliced) | 2026-06-12 | ~7 | 0 | 2 | 4 | 0 | MED | 0/3 | NOT_CLEAN |
| 9 (sliced) | 2026-06-12 | — | — | — | — | — | — | 0/3 | IN_PROGRESS |

## Trajectory Shorthand

`15→20→~8→~15→~6→~4→~4→~7→(P9 in progress)`

Severity profile: CRITICAL count decayed (4→5→0→0→0→0→0→0) — core detection semantics
fully settled. HIGH count: 8→7→~6→~5→1→2→~4→2 — oscillating on mechanical/anchor hygiene
then two genuine HIGHs in Pass 8 (both resolved). MEDIUM count: 3→8→~2→~10→~5→2→0→4 —
propagation/hygiene dominates.

## Convergence Counter

**0/3** consecutive clean passes.
**STRICT mode** (human-elected 2026-06-12): zero findings of ANY severity (including LOW)
across all 4 slices required for 3 consecutive clean passes.

## Core Semantics — Confirmed Clean (Settled)

The following areas have been repeatedly reviewed and confirmed correct. Future passes should
treat these as LOW-RISK unless new evidence contradicts:

- **11-key `summarize()` set** (ADR-008 Decision 7): canonical key enumeration locked.
- **Reconciliation invariant**: all 11 keys are present in every summary output — verified
  each pass.
- **Storm metric = average-since-window-start** (NOT sustained-rate): formula locked; all
  BC files and PRD aligned after Pass-4 sweep.
- **Spoof escalation logic**: multi-sender vs single-sender threshold semantics confirmed.
- **GARP biconditional**: detection trigger (sender_hw == target_hw) confirmed invariant.
- **Three-way decode**: etherparse `DecodedFrame::Arp` / `DecodedFrame::Ip` / other —
  confirmed exhaustive and correct.
- **MITRE discipline**: T0830→LateralMovement, T1557.002→CredentialAccess (Enterprise)
  ATT&CK v19.1 — confirmed correct, not swapped.
- **Catalogue arithmetic**: BC-2.16.001-015 count (15 BCs in catalogue), total 283 BCs in
  project — confirmed.
- **HS roll-up**: holdout scenario count in HS-INDEX confirmed to match catalogue entries.

## Key Settled Decisions (F2 Spec)

| Decision | Value |
|----------|-------|
| HashMap production / BTreeMap Kani-surrogate | ADR-008 (settled Pass 1) |
| MITRE T0830 tactic mapping | LateralMovement (ICS ATT&CK v19.1) |
| MITRE T1557.002 tactic mapping | CredentialAccess (Enterprise ATT&CK v19.1) |
| Canonical `summarize()` key set | 11 keys (ADR-008 Decision 7) |
| Storm metric formula | average-since-window-start (NOT sustained) |
| Forward-declaration convention | BC-2.10.005/008 SEEDED 23/15 → will be 25/17 at STORY-114 |
| BC-2.16.001-015 | 15 catalogue BCs; SS-16 total 283 BCs project-wide |

## Current Artifact Versions

| Artifact | Version | Notes |
|----------|---------|-------|
| PRD | v1.13 | Updated through Pass-8 remediation |
| BC-INDEX | v1.13 | Updated through Pass-8 remediation |
| ADR-008 | v1.6 | Decision 3 updated: LaxNetSlice::Arp routed explicitly (Pass-8 fix) |
| arp-architecture-delta | v1.8 | §2.2 updated: LaxNetSlice::Arp explicit routing (Pass-8 fix) |
| VP-024 | v1.4 | |
| error-taxonomy | v1.8 | |
| test-vectors | v1.6 | |
| HS-INDEX | v1.3 | |
| cap-10 | v1.4 | |
| BC-2.10.002 | v1.4 | |
| BC-2.10.004 | v1.5 | |
| BC-2.10.005 | v1.10 | Forward-declaration convention |
| BC-2.10.008 | v1.11 | Forward-declaration convention |
| BC-2.16.003 | v1.3 | |
| BC-2.16.004 | v1.5 | |
| BC-2.16.005 | v1.4 | |
| BC-2.16.006 | v1.2 | |
| BC-2.16.007 | v1.1 | |
| BC-2.16.008 | v1.5 | |
| BC-2.16.009 | v1.3 | |
| BC-2.16.010 | v1.4 | |
| BC-2.16.013 | v1.1 | |
| BC-2.16.014 | v1.4 | |
| BC-2.02.009 | v1.5 | Revised BC (etherparse migration) |
| Total BCs | 283 | 268 pre-ARP + 15 SS-16 |

## Per-Pass Details

### Pass 1 — 2026-06-12 (monolithic)

**Method:** Single monolithic adversary pass reviewing all F2 spec artifacts.
**Findings:** 15 total — 4 CRITICAL, 8 HIGH, 3 MEDIUM.
**Novelty:** HIGH — first full review of ARP spec; many structural gaps.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings categories:
- CRITICAL: Missing reconciliation invariant (all 11 keys present in summary); GARP
  biconditional not formally stated; storm metric formula ambiguous (average vs sustained);
  HashMap vs BTreeMap Kani-surrogate not documented.
- HIGH: MITRE tactic mapping not specified in BC text (tactic field missing); BC-2.16 count
  mismatch in BC-INDEX; VP-024 precondition underdetermined; etherparse SliceError::Len
  removal not reflected in error-taxonomy.
- MEDIUM: Test-vector edge cases for zero-table state; PRD version lag.

**Remediation:** architect + PO addressed all 15 findings; spec versions bumped.

---

### Pass 2 — 2026-06-12 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** 20 total — 5 CRITICAL, 7 HIGH, 8 MEDIUM.
**Novelty:** HIGH — partial-fix regressions from Pass-1 remediation introduced new
inconsistencies across slice boundaries (Slice D caught cross-doc drift introduced by Slice
A/B fixes that were not propagated to PRD and BC-INDEX in the same burst).
**Convergence counter:** 0/3 (reset; new findings exceed pass-1 total due to regressions).
**Verdict:** NOT_CLEAN.

Key findings categories:
- CRITICAL: Reconciliation invariant added to ADR-008 but not reflected in VP-024 proof
  obligation; storm averaging formula fixed in BC-2.16.014 but PRD still carried old
  sustained-rate language; spoof escalation threshold not consistently defined across
  BC-2.16.001 and BC-2.16.002; two new BCs (BC-2.16.anchor references) missing from
  BC-INDEX count.
- HIGH: MITRE T0830 tactic field in BCs says "Inhibit Response Function" — wrong tactic
  (should be LateralMovement); T1557.002 missing from one BC; catalogue-BC count 15 vs
  BC-INDEX total 283 disagreement.
- MEDIUM (8, majority propagation): PRD version stale; BC-INDEX titles not matching updated
  BC Invariant headings (DF-SIBLING-SWEEP-001 gap); 6 cross-doc propagation items.

**Remediation:** Full consuming-doc sweep; PRD + BC-INDEX updated in same burst as BC fixes.

---

### Pass 3 — 2026-06-12 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** ~8 total — 0 CRITICAL, ~6 HIGH, ~2 MEDIUM.
**Novelty:** MEDIUM — large blocks confirmed clean; remaining issues are specific and targeted.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings categories:
- HIGH (~6): BC-2.16.004 anchor reference broken (targets a section that was renumbered);
  BC-2.16.013 formula uses symbol `Δ` inconsistently vs rest of catalogue; VP-024 proof
  obligation scope unclear for edge case (empty binding table); ADR-008 Decision 7 key list
  vs BC-2.10.005 SEEDED list count discrepancy (forward-declaration vs final count); 2
  additional items in Slice D.
- MEDIUM (~2): Minor prose consistency items; HS-INDEX feature holdout section incomplete.

Notable: All CRITICAL issues from Passes 1–2 confirmed clean across all 4 slices. Core
detection semantics (11-key, storm averaging, spoof escalation, GARP biconditional, MITRE
discipline) all CONFIRMED CLEAN.

**Remediation:** BC-2.16.004 anchor fixed; BC-2.16.013 formula normalized; VP-024 edge case
scoped; ADR-008 Decision 7 forward-declaration convention documented; HS-INDEX updated.

---

### Pass 4 — 2026-06-12 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** ~15 total — 0 CRITICAL, ~5 HIGH, ~10 MEDIUM.
**Novelty:** LOW — almost entirely propagation hygiene; no new semantic issues.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings categories:
- HIGH (~5): PRD v1.11 section on ARP storm references old sustained-rate formula (consuming-doc
  sweep missed this PRD subsection); BC-INDEX v1.9 total-BCs count 282 not updated to 283
  after adding BC-2.16.015; BC-2.16.004 anchor fix in Pass-3 introduced a dangling xref in
  BC-2.16.007 (sibling propagation gap); 2 additional Slice D items.
- MEDIUM (~10): BC-INDEX title text for 7 BCs not matching updated BC Invariant headings
  (DF-SIBLING-SWEEP-001 / PG-F7-004 — recurring across passes 2,3,4); HS-INDEX HS-W38
  count one less than catalogue-BC-aligned scenario count; PRD version tag in 3 consuming
  references stale.

**Recurring process gap:** catalogue/count fixes land in BC files but not propagated to
PRD/BC-INDEX in the same burst — recurred in passes 2, 3, and 4. See `[process-gap]`
DF-SIBLING-SWEEP-001 below.

**Remediation:** Full consuming-doc sweep (DF-SIBLING-SWEEP-001 protocol): PRD → BC-INDEX →
HS-INDEX updated in same atomic burst as BC-layer fixes. PRD bumped to v1.12; BC-INDEX to
v1.10; all 15 BC-INDEX titles verified against BC Invariant headings.

---

### Pass 5 — 2026-06-12 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** 0C / 1H / ~5M — mechanical (prose/anchor hygiene).
**Novelty:** LOW.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings: 1 HIGH (mechanical) + ~5 MEDIUM mechanical (prose/anchor hygiene items).
Core semantics CONFIRMED CLEAN across all 4 slices.

**Remediation:** All mechanical findings addressed.

---

### Pass 6 — 2026-06-12 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** Slice A CLEAN; 2H / 2M (other slices).
**Novelty:** LOW.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings: Slice A confirmed fully clean. 2 HIGH + 2 MEDIUM across remaining slices.

**Remediation:** All findings addressed.

---

### Pass 7 — 2026-06-12 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** 0C / ~4H — anchor + version hygiene.
**Novelty:** LOW.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings: ~4 HIGH items — anchor reference hygiene and version hygiene (no new semantic issues).

**Remediation:** All findings addressed.

---

### Pass 8 — 2026-06-12 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** 0C / 2 GENUINE HIGH / 4 MED + 1 brownfield obligation.
**Novelty:** MEDIUM — two genuine HIGH findings surfaced.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings:
- GENUINE HIGH (1): Reachable `unreachable!()` panic on truncated ARP via lax decode path.
  **FIXED:** ADR-008 Decision 3 v1.6 + arch-delta §2.2 v1.8 now route `LaxNetSlice::Arp`
  explicitly.
- GENUINE HIGH (1): `Ethernet2Slice::source()` return type scrutiny.
  **CONFIRMED:** `[u8; 6]` by value — code correct, no fix required.
- MEDIUM (4): Additional medium-severity findings; all addressed.
- Brownfield obligation: IcsImpact "Impact (ICS)" mismatch recorded as STORY-114 F4
  obligation.

All passes 5–8 remediated. Core spec confirmed clean repeatedly across passes 3–8.

---

### HUMAN DECISION — 2026-06-12

**Convergence endgame:** STRICT 3-consecutive-clean mode (human-elected 2026-06-12).
Definition: zero findings of ANY severity (including LOW) across all 4 slices, 3 passes
running.

**Current counter: 0/3.** Pass 9 in progress.

---

## Process Gaps (Candidate Policy Items)

### [process-gap] PG-ARP-F2-001 — Adversary tool-profile (S-7.02)

The adversary agent operates with a read-only tool profile and cannot persist its own slice
reports as files. Full slice findings live only in the orchestrator session transcript. This
creates a durability risk: if the session ends before the state-manager persists findings,
the adversary output is lost.

**Candidate remediation:** Either (a) grant adversary agent write access to a scoped
`adversarial-reviews/` path for pass reports, or (b) require orchestrator to invoke
state-manager immediately after each adversary pass to persist findings before proceeding
to remediation.

**Policy candidate:** S-7.02 amendment — adversary-report persistence obligation.

---

### [process-gap] PG-ARP-F2-002 — Catalogue sweep propagation (DF-SIBLING-SWEEP-001)

Recurred in passes 2, 3, and 4: catalogue-level or count fixes land in BC files (or ADR)
but the same burst does NOT update consuming documents (PRD count fields, BC-INDEX total,
HS-INDEX catalogue count). This forces the adversary to re-find the same propagation gap
in the next pass.

**Root cause:** Remediating agent applies the direct fix (BC file) then stops; does not
execute the consuming-doc sweep required by DF-SIBLING-SWEEP-001.

**Candidate remediation:** Add an explicit "consuming-doc checklist" to the F2 remediation
runbook: after any BC add/remove/rename or count change, sweep PRD + BC-INDEX + HS-INDEX +
all consuming-story body-notes in the same burst before committing.

**Policy candidate:** DF-SIBLING-SWEEP-001 sub-rule — count-change consuming-doc sweep
mandatory in same commit. See also PG-F7-004 from DNP3 cycle (same class of defect).

---

## Notes

- Pass 1 was monolithic (pre-SLICED method adoption for this feature). Passes 2+ use the
  4-slice parallel method per user direction.
- Slice reports from passes 2-5 live in the orchestrator session transcript (see
  PG-ARP-F2-001 above). This file captures the distilled per-pass summary.
- The SLICED method is proving effective at surfacing cross-doc consistency issues (Slice D)
  that monolithic passes miss — passes 2-4 all had their largest finding cluster in Slice D.
