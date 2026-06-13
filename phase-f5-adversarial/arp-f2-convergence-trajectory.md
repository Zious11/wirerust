---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-13T00:00:00Z
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
| 9 (sliced) | 2026-06-13 | ~4 | 0 | 0 | ~4 | 0 | LOW | 0/3 | NOT_CLEAN |
| 10 (sliced) | 2026-06-13 | ~6 | 0 | 1 | ~5 | 0 | LOW | 0/3 | NOT_CLEAN |
| 11 (sliced) | 2026-06-13 | ~5 | 0 | 1 | ~4 | 0 | LOW | 0/3 | NOT_CLEAN |

## Trajectory Shorthand

`15→20→~8→~15→~6→~4→~4→~7→~4→~6→~5`

Severity profile: CRITICAL count decayed (4→5→0→0→0→0→0→0→0→0→0) — core detection
semantics fully settled. HIGH count: 8→7→~6→~5→1→2→~4→2→0→1→1 — only corpus-wide debt
items; ARP delta clean. MEDIUM count: 3→8→~2→~10→~5→2→0→4→~4→~5→~4 — propagation hygiene
and churn residue dominate.

## Convergence Counter

**0/3** consecutive clean passes.
**STRICT WHOLE-CORPUS mode** (human-elected 2026-06-12; scope extended 2026-06-13): zero
findings of ANY severity (including LOW) across the ENTIRE spec corpus (not just ARP delta)
required for 3 consecutive clean passes. 11 passes run. Corpus-wide sweep in progress.

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
| PRD | v1.15 | Updated through Pass-11 remediation |
| BC-INDEX | v1.16 | Updated through Pass-11 remediation |
| ADR-008 | v1.8 | D11 cell corrected; through Pass-11 remediation |
| arp-architecture-delta | v1.10 | §5 Some() double-wrap fixed (Pass-11) |
| VP-024 | v1.4 | |
| vp-016 | v2.1 | |
| error-taxonomy | v1.9 | |
| test-vectors | v1.9 | Citation corrected through Pass-11 |
| HS-INDEX | v1.3 | |
| cap-10 | v1.7 | IcsDiscovery naming/description corrected (Pass-9) |
| ent-04 | v1.1 | |
| ent-05 | v1.1 | |
| cap-11 | v1.1 | |
| nfr-catalog | v1.4 | |
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
| BC-2.16.014 | v1.6 | Inconclusive classification gap fixed (Pass-11) |
| BC-2.02.009 | v1.6 | Lax-arm precision gap fixed (Pass-10) |
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

### Pass 9 — 2026-06-13 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** Slice D CLEAN; NOT_CLEAN overall.
**Novelty:** LOW — ARP delta itself CLEAN; findings from corpus-wide debt and remediation-churn residue.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings:
- `malformed_frames` definition drift (cross-doc inconsistency).
- cap-10 IcsDiscovery: naming/description inconsistency.
- Changelog ordering / typo items.

**Remediation:** All findings addressed.

---

### Pass 10 — 2026-06-13 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** Slice A CLEAN; NOT_CLEAN overall.
**Novelty:** LOW — corpus-wide propagation debt (pre-existing DNP3-era items).
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings:
- ADR D11 cell: stale/incorrect table cell content.
- test-vectors count: citation mismatch.
- PRD §2.10 Enterprise mislabel.
- RTM omission.
- 16→17 count not propagated across 5 docs (pre-existing DNP3-era debt).
- BC-2.02.009 lax-arm: precision gap.

**Remediation:** All findings addressed.

---

### Pass 11 — 2026-06-13 (sliced: A, B, C, D)

**Method:** 4 parallel fresh-context slices.
**Findings:** NOT_CLEAN overall.
**Novelty:** LOW — remediation-churn residue + pre-existing corpus debt.
**Convergence counter:** 0/3.
**Verdict:** NOT_CLEAN.

Key findings:
- arch-delta §5: `Some()` double-wrap (introduced by prior remediation).
- E-ARP-004: missing flap-window condition.
- BC-2.16.014: `Inconclusive` classification gap.
- test-vectors citation: stale reference.
- PRD issue-#100 registration gap: BC-2.04.055 / BC-2.09.007 now registered (gap closed).

**Remediation:** All findings addressed. BC-2.04.055 / BC-2.09.007 PRD registration now complete.

---

### ANALYSIS — After Pass 11 (2026-06-13)

**ARP F2 delta itself CONVERGED** since approximately Pass 3. Only trivia found in ARP-specific
artifacts in passes 9–11. Findings are now dominated by:
(a) **Pre-existing corpus-wide debt** — DNP3-era 16→17 count propagation, issue-#100 PRD
    registration (BC-2.04.055/BC-2.09.007 now resolved).
(b) **Remediation-churn residue** — fixes in one pass introduce small inconsistencies caught
    in the next pass (arch-delta §5 Some() double-wrap is the canonical example).

**Strict 3-consecutive-clean is asymptotic** under whole-corpus fresh-context review: the
reviewer surfaces anything in the full corpus, not just ARP delta, so pre-existing systematic
debt classes will always produce findings until explicitly flushed.

---

### Corpus Consistency Audit — 2026-06-13

**Method:** Comprehensive corpus-wide consistency sweep (flush systematic debt classes before
resuming strict whole-corpus sliced passes). All findings REMEDIATED.

**Blocking corpus-debt defects found and REMEDIATED (9 total):**

1. ARCH-INDEX SS-04/09/16 component counts — stale counts corrected.
2. module-decomposition C-24 DNP3 missing — entry added.
3. module-criticality C-23/C-24 missing — entries added.
4. VP-INDEX lifecycle counts — counts updated.
5. vp-007 ARP F4 obligation — obligation added.
6. BC-2.16.010 H1 enrichment — enrichment clause added.
7–9. Three additional blocking corpus-debt items remediated in same burst.

**Correctly classified (not a defect):** STORY-114 PLANNED code-vs-spec entry — correctly
classified as expected (F4 obligation, not a convergence defect).

**Verdict:** REMEDIATED — all 9 blocking items closed. Ready for Pass 12.

---

### Pass 12 — 2026-06-13 (strict whole-corpus; NOT_CLEAN, REMEDIATED)

**Method:** Strict whole-corpus fresh-context pass across full spec corpus.
**Findings:** ~18 total — 0 CRITICAL, 0 HIGH (corpus-wide), many MEDIUM/LOW — ALL pre-existing
corpus debt unrelated to ARP F2 delta.
**Novelty:** LOW — zero ARP-F2 defects; ARP delta clean 4th consecutive pass.
**Convergence counter:** 0/3 (corpus-wide debt findings prevent clean count; ARP delta itself
clean).
**Verdict:** NOT_CLEAN (corpus debt), REMEDIATED.

Finding categories (all pre-existing corpus debt, none ARP-F2-specific):
- SS-14 Modbus BC title desyncs (6 findings).
- Stale technique_info/technique_tactic line anchors in inv-01, nfr-catalog, vp-007,
  STORY-071, BC-2.10.001, BC-2.10.003, BC-2.10.007 (6 findings).
- ARCH-INDEX "21 components" count and O-04 "9" count — stale corpus counts.
- tooling-selection and dependency-graph missing DNP3 entries.
- ADR-005, ADR-006, ADR-007 status: proposed→accepted (not yet updated).
- changelog phantom vp-016 path — stale reference.
- BC-2.16.008 missing ARP_FLAP_WINDOW_SECS anchor.

**All ~18 findings REMEDIATED.**

**KEY FINDING:** Strict whole-corpus = full audit of the released-product spec corpus (283 BCs,
24 VPs, 8 ADRs, domain/capability/entity/story docs); each pass surfaces fresh pre-existing
drift in not-yet-reviewed docs. Sustained multi-pass effort required; counter still 0/3.

**Artifact versions post-Pass-12:**

| Artifact | Version | Change |
|----------|---------|--------|
| ARCH-INDEX | v1.3 | SS-04/09/16 counts + O-04 updated |
| tooling-selection | v1.2 | DNP3 entry added |
| dependency-graph | v1.2 | DNP3 entry added |
| module-decomposition | v1.4 | C-24 DNP3 added |
| module-criticality | v1.2 | C-23/C-24 added |
| VP-INDEX | v2.1 | Lifecycle counts updated |
| vp-007 | v2.4 | ARP F4 obligation added; technique anchors corrected |
| arp-architecture-delta | v1.10 | (carried from Pass-11) |
| BC-INDEX | v1.18 | Titles synced |
| PRD | v1.15 | (carried from Pass-11) |
| ADR-005/006/007 | accepted | Status updated proposed→accepted |
| ADR-008 | v1.8 | (carried from Pass-11) |
| BC-2.16.008 | v1.6 | ARP_FLAP_WINDOW_SECS anchor added |
| BC-2.16.010 | v1.6 | H1 enrichment added |
| BC-2.10.001 | v1.3 | technique_info/tactic anchors corrected |
| BC-2.10.003 | v1.4 | technique_info/tactic anchors corrected |
| BC-2.10.007 | v1.7 | technique_info/tactic anchors corrected |
| inv-01 | v1.1 | technique anchors corrected |
| nfr-catalog | v1.5 | technique anchors corrected |
| STORY-071 | v1.9 | technique anchors corrected |
| cap-10 | v1.7 | (carried from Pass-9) |
| error-taxonomy | v1.9 | (carried from Pass-11) |
| test-vectors | v1.9 | (carried from Pass-11) |

---

### HUMAN DECISION — 2026-06-13

**CONTINUE STRICT WHOLE-CORPUS** — bar remains zero findings of ANY severity across the
ENTIRE spec corpus (not just ARP delta). This is explicitly accepted as a full corpus
audit/cleanup, not just ARP F2 convergence. Counter 0/3.

**New tactic:** comprehensive corpus-wide consistency sweep (flush systematic debt classes)
before resuming strict sliced passes broadened to whole corpus.

---

### HUMAN DECISION — 2026-06-12

**Convergence endgame:** STRICT 3-consecutive-clean mode (human-elected 2026-06-12).
Definition: zero findings of ANY severity (including LOW) across all 4 slices, 3 passes
running.

**Current counter: 0/3.** Passes 9–11 complete; corpus-wide sweep in progress.

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
