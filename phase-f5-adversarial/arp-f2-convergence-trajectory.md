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
| 13 (whole-corpus) | 2026-06-13 | ~8 | 0 | 0 | ~8 | 0 | LOW | 0/3 | NOT_CLEAN, REMEDIATED |
| 14 (whole-corpus) | 2026-06-13 | 22 | 2 | 5 | ~11 | ~4 | MED | 0/3 | NOT_CLEAN |
| 15 (whole-corpus, Claude) | 2026-06-13 | 8 | 2 | 1 | 3 | 2 | MED | 0/3 | NOT_CLEAN→REMEDIATED |

## Trajectory Shorthand

`15→20→~8→~15→~6→~4→~4→~7→~4→~6→~5→~18→~8→~22(P14: 2C/5H NEW corpus-debt; trend broke; ARP delta clean 6th pass)→P15(8 findings: holdout-layer field-rename + regression; REMEDIATED)`

Severity profile: CRITICAL count: 4→5→0→0→0→0→0→0→0→0→0→0→0→2 — REGRESSION at Pass 14
(2 genuinely-new CRITICAL corpus-debt findings, not ARP delta defects). HIGH count:
8→7→~6→~5→1→2→~4→2→0→1→1→0→0→5 — REGRESSION at Pass 14 (5 new HIGH findings in architecture
and domain docs not reached by prior 13 passes); ARP delta itself clean. MEDIUM count:
3→8→~2→~10→~5→2→0→4→~4→~5→~4→~18→~8→~11 — propagation hygiene and pre-existing corpus debt.
Trend BROKE at Pass 14 — Passes 12-13 showed 0 CRIT/0 HIGH; Pass 14 surfaced 2 CRITICAL + 5 HIGH
genuinely-new shipped-code-vs-spec drift not reached by 13 prior passes.
Slice B (all 283 BC H1 titles) verified CLEAN in Pass 13; ARP delta clean 6th consecutive pass.

## Convergence Counter

**0/3** consecutive clean passes.
**STRICT WHOLE-CORPUS mode** (human-elected 2026-06-12; scope extended 2026-06-13): zero
findings of ANY severity (including LOW) across the ENTIRE spec corpus (not just ARP delta)
required for 3 consecutive clean passes. 15 passes run. Pass 14 REMEDIATED (22 findings:
mitre_techniques field-rename corpus sweep + O-01 closure propagation + architect ×2 + PO ×10
bursts + consistency audit CONSISTENT). Pass 15 REMEDIATED (8 findings: holdout-scenarios
field-rename sweep [C-01/02/03, 16 files] + inv-01 YAML regression [C-04] + VP-024 scope
reconciliation [A-01] + 4 more; consistency audit CONSISTENT all 7 dimensions). Next = Pass 16
via agy (Gemini, additional quota provided by human). Counter 0/3 — remediation does NOT
advance counter.

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
| PRD | v1.16 | Updated through Pass-13 remediation |
| BC-INDEX | v1.19 | T1692.001 literal corrected (Pass-13) |
| ADR-008 | v1.8 | (carried from Pass-11) |
| ADR-007 | accepted + drift-note | IcsImpact "Impact" vs shipped "Impact (ICS)" drift-note added (Pass-13) |
| ARCH-INDEX | v1.4 | Updated through Pass-13 remediation |
| arp-architecture-delta | v1.10 | §5 Some() double-wrap fixed (Pass-11) |
| verification-architecture | v1.5 | extract_sni anchor corrected (Pass-13) |
| VP-024 | v1.4 | |
| vp-005 | v2.1 | Updated through Pass-13 remediation |
| vp-007 | v2.4 | (carried from Pass-12) |
| vp-008 | v2.1 | Stale BC title corrected (Pass-13) |
| vp-016 | v2.1 | |
| VP-INDEX | v2.2 | VP-024 BC-scope corrected; non-Kani footnote (Pass-15) |
| error-taxonomy | v1.9 | |
| test-vectors | v1.9 | Citation corrected through Pass-11 |
| HS-INDEX | v1.3 | |
| cap-10 | v1.7 | IcsDiscovery naming/description corrected (Pass-9) |
| ent-04 | v1.1 | |
| ent-05 | v1.1 | |
| cap-11 | v1.1 | |
| nfr-catalog | v1.6 | NFR-OBS-004 seeded/emitted mislabel corrected (Pass-13) |
| module-criticality | v1.2 | Document-Map gap closed (Pass-13) |
| tooling-selection | v1.2 | (carried from Pass-12) |
| dependency-graph | v1.2 | (carried from Pass-12) |
| module-decomposition | v1.4 | (carried from corpus audit) |
| BC-2.10.002 | v1.4 | |
| BC-2.10.004 | v1.5 | |
| BC-2.10.005 | v1.10 | Forward-declaration convention |
| BC-2.10.006 | v1.3 | Stale src/mitre.rs anchor (:153→:179) + "15"→23/25 counts corrected (Pass-13) |
| BC-2.10.008 | v1.11 | Forward-declaration convention |
| BC-2.16.003 | v1.3 | |
| BC-2.16.004 | v1.5 | |
| BC-2.16.005 | v1.4 | |
| BC-2.16.006 | v1.2 | |
| BC-2.16.007 | v1.1 | |
| BC-2.11.024 | v1.7 | Evidence "four Option fields"→3 Option + mitre_techniques.join; csv.rs anchor corrected (Pass-15) |
| BC-2.16.008 | v1.6 | (carried from Pass-12) |
| BC-2.16.009 | v1.3 | |
| BC-2.16.010 | v1.6 | (carried from Pass-12) |
| BC-2.16.013 | v1.1 | |
| BC-2.16.014 | v1.6 | Inconclusive classification gap fixed (Pass-11) |
| BC-2.02.009 | v1.6 | Lax-arm precision gap fixed (Pass-10); BC-INDEX pin corrected (Pass-15) |
| STORY-071 | v1.10 | Stale 16/21→17/23 counts corrected (Pass-13) |
| ADR-005/006 | accepted | (carried from Pass-12) |
| inv-01 | v1.2 | Duplicate `version:` YAML key deduped (Pass-15 regression fix) |
| BC-2.10.001 | v1.3 | (carried from Pass-12) |
| BC-2.10.003 | v1.4 | (carried from Pass-12) |
| BC-2.10.007 | v1.7 | (carried from Pass-12) |
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

### Pass 13 — 2026-06-13 (strict whole-corpus; NOT_CLEAN, REMEDIATED)

**Method:** Strict whole-corpus fresh-context pass across full spec corpus.
**Findings:** ~8 total — 0 CRITICAL, 0 HIGH, ~8 MEDIUM — ALL pre-existing corpus debt
unrelated to ARP F2 delta. **Slice B (all 283 BC H1 titles) verified CLEAN.**
**Novelty:** LOW — zero ARP-F2 defects; ARP delta clean 5th consecutive pass.
**Convergence counter:** 0/3 (strict whole-corpus; a fully-clean all-4-slice pass not yet
achieved; each pass still surfaces residual pre-existing drift in not-yet-swept docs).
**Verdict:** NOT_CLEAN (corpus debt), REMEDIATED.

Finding categories (all pre-existing corpus debt, none ARP-F2-specific):

- BC-2.10.006 sibling-orphan: stale `src/mitre.rs:153` anchor (corrected to `:179`) + "15"→23/25
  sibling counts.
- nfr-catalog NFR-OBS-004: seeded/emitted mislabel corrected.
- STORY-071 body: stale 16/21→17/23 counts corrected.
- BC-INDEX:356: stale `["T0855"...]` literal corrected to T1692.001.
- ADR-007: IcsImpact "Impact" vs shipped "Impact (ICS)" drift-note added (accepted; F4 obligation
  tracks the code-side rename).
- extract_sni anchor off-by-one: vp-005/verification-architecture corrected.
- VP-008: stale BC title corrected.
- module-criticality: Document-Map gap closed.

**All ~8 findings REMEDIATED.**

**KEY FINDING:** Trajectory decaying — corpus audit 9 findings → Pass 12 ~18 → Pass 13 ~8.
Slice B (all 283 BC H1 titles) confirmed CLEAN. Zero ARP-F2 defects (delta clean 5th
consecutive pass). Strict whole-corpus counter still 0/3; sustained multi-pass corpus cleanup
ongoing.

**Artifact versions post-Pass-13:**

| Artifact | Version | Change |
|----------|---------|--------|
| ARCH-INDEX | v1.4 | Updated |
| verification-architecture | v1.5 | extract_sni anchor corrected |
| vp-005 | v2.1 | Updated |
| vp-008 | v2.1 | Stale BC title corrected |
| ADR-007 | accepted + drift-note | IcsImpact drift-note added |
| BC-2.10.006 | v1.3 | Stale anchor + count corrected |
| nfr-catalog | v1.6 | NFR-OBS-004 seeded/emitted mislabel corrected |
| STORY-071 | v1.10 | Stale 16/21→17/23 counts corrected |
| BC-INDEX | v1.19 | T1692.001 literal corrected |
| PRD | v1.16 | Updated |

---

### Pass 14 — 2026-06-13 (strict whole-corpus; NOT_CLEAN)

**Method:** Strict whole-corpus fresh-context pass across full spec corpus; 4 parallel slices.
**Factory-artifacts HEAD reviewed:** 69da05c.
**Findings:** 22 total — 2 CRITICAL, 5 HIGH, ~11 MEDIUM, ~4 LOW — ALL pre-existing corpus
debt unrelated to ARP F2 delta; all 4 slices NOT_CLEAN.
**Novelty:** MED — 2 CRITICAL + 5 HIGH are genuinely-new shipped-code-vs-spec drift not
reached by 13 prior passes (trend broke; Passes 12-13 were 0 CRIT/0 HIGH).
**Convergence counter:** 0/3 (counter stays 0/3; ARP F2 delta CLEAN 6th consecutive pass).
**Verdict:** NOT_CLEAN.

**PERSISTENCE-ONLY — findings NOT remediated this burst. Awaiting human strategic decision
(continue strict whole-corpus remediation + Pass 15 vs off-ramp).**

---

### Pass 14 — REMEDIATION COMPLETE (2026-06-13)

**Decision:** CONTINUE STRICT WHOLE-CORPUS. All 22 findings remediated across architect ×2
bursts + PO ×10 bursts + consistency audit + O-01 closure sweep.

#### Architect bucket (Slice A 9 findings + D-OBS-01)

Dispatch 1 (A-01/A-03/A-04/A-06/A-07/D-OBS-01 — field-rename + component gaps + peer disagreement):

| Artifact | Change |
|----------|--------|
| `specs/architecture/api-surface.md` | v1.1→v1.3 — `mitre_technique: Option<String>` renamed to `mitre_techniques: Vec<String>` in Finding row; 3 missing analyze flags added (`--modbus-write-burst-threshold`, `--modbus-write-sustained-threshold`, `--dnp3-direct-operate-threshold`); `decode_packet` PLANNED marker added uniformly |
| `specs/architecture/purity-boundary-map.md` | v1.1→v1.2 — C-23 arp.rs + C-24 dnp3.rs entries added; `mitre.rs` implications updated to include T0888 + T1691.001/T0827; PLANNED markers added |
| `specs/architecture/system-overview.md` | v1.1→v1.2 — L3 analyzer list updated to include C-22 Modbus, C-23 ARP, C-24 DNP3; "C-1..C-20" note corrected to "C-1..C-24"; mitre.rs technique count updated 15→23 (target 25) |
| `specs/architecture/module-decomposition.md` | v1.4→v1.6 — C-16/C-22 MITRE lists updated to include T0888; etherparse version corrected to 0.20 (PLANNED for C-5); peer-disagreement resolved uniformly |
| `specs/architecture/dependency-graph.md` | v1.2→v1.4 — etherparse 0.16→0.20 corrected; PLANNED marker added for DecodedFrame return-type transition |
| `specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` | modified[] — `[2,253]` length range corrected to `[2,254]` (D-OBS-01 sweep with D-01) |

Dispatch 2 (A-02/A-05/A-08/A-09 — remaining Slice A items):

| Artifact | Change |
|----------|--------|
| `specs/architecture/api-surface.md` | (same file, same version bump as above — carried) |
| `specs/architecture/purity-boundary-map.md` | (same file, same version bump — carried) |
| `specs/architecture/system-overview.md` | (same file, same version bump — carried) |
| `specs/architecture/module-decomposition.md` | (same file, same version bump — carried) |

#### PO bursts 1–10 (Slice B/C/D + corpus-wide sweeps)

**Burst 1 — cap-09 (C-01/C-02: authoritative schema + emission-site count):**

| Artifact | Change |
|----------|--------|
| `specs/domain/capabilities/cap-09-finding-emission.md` | v1.1→v1.1 (new file; field-rename + emission-site recount applied) — `mitre_technique: Option<String>` replaced with `mitre_techniques: Vec<String>`; emission-site count updated from "22" to current shipped count (STORY-097..110); 3 Option fields corrected; BC refs updated |

**Burst 2 — PRD/BC-INDEX/VPs/delta-analysis (D-01/D-02 + mitre_techniques corpus sweep):**

| Artifact | Change |
|----------|--------|
| `specs/prd.md` | v1.16→v1.18 — §2.14.A BC-2.14.004 reject range corrected `[2,253]`→`[2,254]`; BC-INDEX status line PRD version updated; mitre_techniques field-rename applied to all current-state Finding schema occurrences |
| `specs/behavioral-contracts/BC-INDEX.md` | v1.19→v1.23 — PRD version-pin updated v1.15→v1.18; `mitre_technique` → `mitre_techniques` in all current-state annotations |
| `specs/verification-properties/vp-007-mitre-technique-id-format.md` | v2.4→v2.5 — field-rename applied to current-state Finding schema snippet |
| `specs/verification-properties/vp-016-mitre-tactic-grouping-order.md` | v2.1→v2.2 — field-rename applied |
| `specs/verification-properties/vp-020-csv-injection-neutralization.md` | v2.0→v2.1 — field-rename applied |
| `phase-f1-delta-analysis/arp-analyzer-delta-analysis.md` | modified[] — mitre_techniques field-rename applied to current-state snippets; C-06 `mitre_research_status` note reviewed (intentional frozen F1 snapshot confirmed; prose clarified) |

**Burst 3 — ss-14 BC bodies (B-01/B-02/B-03/B-04):**

| Artifact | Change |
|----------|--------|
| `specs/behavioral-contracts/ss-14/BC-2.14.017.md` | modified[] — MITRE Techniques name corrected from "Unauthorized Command Message" (revoked-T0855 name) to "Unauthorized Message: Command Message" (T1692.001 canonical) |
| `specs/behavioral-contracts/ss-14/BC-2.14.024.md` | modified[] — same stale name corrected |
| `specs/behavioral-contracts/ss-14/BC-2.14.020.md` | modified[] — Invariant 6 SEEDED/EMITTED counts updated 21/13→25/17; Source-Evidence stale counts annotated as Decision-12-era superseded |
| `specs/behavioral-contracts/ss-14/BC-2.14.004.md` | modified[] — reject range `[2,253]`→`[2,254]` (D-01 sibling sweep) |

**Burst 4 — ss-04 + ss-09 BC bodies (mitre_techniques field-rename):**

| Artifact | Change |
|----------|--------|
| `specs/behavioral-contracts/ss-04/BC-2.04.018.md` | modified[] |
| `specs/behavioral-contracts/ss-04/BC-2.04.019.md` | modified[] |
| `specs/behavioral-contracts/ss-04/BC-2.04.020.md` | modified[] |
| `specs/behavioral-contracts/ss-04/BC-2.04.021.md` | modified[] |
| `specs/behavioral-contracts/ss-04/BC-2.04.023.md` | modified[] |
| `specs/behavioral-contracts/ss-04/BC-2.04.025.md` | modified[] |

**Burst 5 — ss-06 + ss-10 BC bodies (mitre_techniques field-rename):**

| Artifact | Change |
|----------|--------|
| `specs/behavioral-contracts/ss-06/BC-2.06.005.md` | modified[] |
| `specs/behavioral-contracts/ss-06/BC-2.06.006.md` | modified[] |
| `specs/behavioral-contracts/ss-06/BC-2.06.007.md` | modified[] |
| `specs/behavioral-contracts/ss-06/BC-2.06.008.md` | modified[] |
| `specs/behavioral-contracts/ss-06/BC-2.06.009.md` | modified[] |
| `specs/behavioral-contracts/ss-06/BC-2.06.010.md` | modified[] |
| `specs/behavioral-contracts/ss-06/BC-2.06.011.md` | modified[] |
| `specs/behavioral-contracts/ss-06/BC-2.06.014.md` | modified[] |

**Burst 6 — ss-07 BC bodies (mitre_techniques field-rename):**

| Artifact | Change |
|----------|--------|
| `specs/behavioral-contracts/ss-07/BC-2.07.009.md` | modified[] |
| `specs/behavioral-contracts/ss-07/BC-2.07.010.md` | modified[] |
| `specs/behavioral-contracts/ss-07/BC-2.07.011.md` | modified[] |
| `specs/behavioral-contracts/ss-07/BC-2.07.012.md` | modified[] |
| `specs/behavioral-contracts/ss-07/BC-2.07.014.md` | modified[] |
| `specs/behavioral-contracts/ss-07/BC-2.07.017.md` | modified[] |
| `specs/behavioral-contracts/ss-07/BC-2.07.019.md` | modified[] |

**Burst 7 — ss-11 + BC-2.11.016 (mitre_techniques field-rename):**

| Artifact | Change |
|----------|--------|
| `specs/behavioral-contracts/ss-11/BC-2.11.016.md` | v1.4→v1.5 — `mitre_techniques` field-rename applied |

**Burst 8 — interface-definitions + nfr-catalog (corpus sweep):**

| Artifact | Change |
|----------|--------|
| `specs/prd-supplements/interface-definitions.md` | v1.0→v1.1 — `mitre_technique` → `mitre_techniques` in Finding struct definition |
| `specs/prd-supplements/nfr-catalog.md` | v1.6→v1.8 — field-rename applied; technique anchor lines updated |

**Burst 9 — domain docs (ent-01/cap-01/cap-10/cap-11/ent-04/domain-debt/inv-01):**

| Artifact | Change |
|----------|--------|
| `specs/domain/entities/ent-01-ingestion-decoding.md` | v1.1 (new) — created; O-01 closure noted |
| `specs/domain/capabilities/cap-01-pcap-ingestion.md` | v1.1 (new) — created; O-01 closure noted |
| `specs/domain/capabilities/cap-10-mitre-mapping.md` | v1.7→v1.8 — field-rename applied |
| `specs/domain/capabilities/cap-11-reporting-output.md` | v1.1→v1.2 — field-rename applied |
| `specs/domain/entities/ent-04-findings-output.md` | v1.1→v1.2 — E-26 schema corrected: `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>` (C-03 fix; sibling to E-27 sweep missed in Pass-10) |
| `specs/domain/domain-debt.md` | v1.1→v1.2 — O-01 (Finding.timestamp) reframed from OPEN to CLOSED: STORY-097/098/099 (E-12) wired timestamp; O-01 closure propagated |
| `specs/domain/invariants/inv-01-core-invariants.md` | v1.1→v1.2 — O-01 closure noted; field-rename applied to Finding invariants |

**Burst 10 — test-vectors/error-taxonomy/spec-changelog:**

| Artifact | Change |
|----------|--------|
| `specs/prd-supplements/test-vectors.md` | v1.9→v2.0 — `mitre_techniques` field-rename applied; `input-hash:TBD` note added with PLANNED rationale (src/analyzer/arp.rs not yet in develop; by-design per DRIFT-BC-INPUTHASH-TBD-001) |
| `specs/prd-supplements/error-taxonomy.md` | v1.9→v2.0 — field-rename applied; `input-hash:TBD` rationale added; C-07 storm-rate prose corrected |
| `spec-changelog.md` | modified[] — all version bumps from P14 remediation recorded |

#### Two systematic debt classes flushed

**Debt class 1 — STORY-100/ADR-006 multi-tag field rename:**
`mitre_technique: Option<String>` was renamed to `mitre_techniques: Vec<String>` in
STORY-100 (E-13, completed, v0.3.0; ADR-006 accepted). Propagated to ALL current-state
Finding schema snippets corpus-wide. Affected: cap-09, ent-04, interface-definitions,
nfr-catalog, vp-007, vp-016, vp-020, BC-INDEX annotations, all ss-04/ss-06/ss-07/ss-11/ss-14
BCs that carry current-state Finding schema references, prd.md, test-vectors, error-taxonomy,
api-surface. History/migration prose in ADR-006, STORY-100 body, and changelog entries
correctly preserved (those describe the rename event itself — intentionally retain the old name
as the "before" side). Zero current-state singular-field snippets remaining after sweep (grep
confirmed).

**Debt class 2 — domain-debt O-01 closure propagation:**
O-01 (Finding.timestamp universally None — genuine domain debt) was closed by
STORY-097/098/099 (E-12, completed, v0.3.0) which wired timestamp to actual capture time.
O-01 was still framed as OPEN in domain-debt.md + ent-01 + cap-01 + cap-09 emission-site
note + cap-10 + inv-01 + test-vectors annotation. All instances reframed to CLOSED with
STORY-097..099 as the closing reference. Final grep confirms zero open-framed O-01 across
corpus.

#### Consistency audit (DF-CONSISTENCY-AUDIT-POST-FIXBURST-001)

**Verdict: CONSISTENT** on 5/6 dimensions.

- Dimension 1 (BC-INDEX ↔ BC files): CONSISTENT — all 283 BC titles match.
- Dimension 2 (PRD counts ↔ BC-INDEX): CONSISTENT — 283 BCs, 24 VPs, 17 tactics.
- Dimension 3 (architecture cross-doc): CONSISTENT — api-surface/purity-boundary-map/
  module-decomposition/dependency-graph/system-overview peer-aligned after P14 architect bursts.
- Dimension 4 (field-rename saturation): CONSISTENT — zero current-state `mitre_technique`
  (singular) snippets remaining; grep clean.
- Dimension 5 (O-01 closure): CONSISTENT — zero open-framed O-01 remaining; grep clean.
- Dimension 6 (F1-F4 document residuals): F1-F4 documents (phase-f1-delta-analysis,
  arp-analyzer-delta-analysis) carried O-01 residuals — FOUND AND FIXED. Post-fix: CONSISTENT.

#### Note [process-gap]

O-01 closure (from a prior cycle, completed v0.3.0 via STORY-097..099) was never fully
propagated to its consuming documents (domain-debt, ent-01, cap-01, cap-09, cap-10, inv-01,
test-vectors annotation). This surface only under strict whole-corpus review across all
document types simultaneously — the same class of defect as DF-SIBLING-SWEEP-001
(catalogue-level change not propagated to consuming docs in the same burst). Candidate for
codification follow-up: DF-SIBLING-SWEEP-001 sub-rule — story-close propagation obligation
(when a story closes a domain-debt item, sweep ALL consuming docs that reference that item
as open in the same burst).

#### Slice A (route: architect) — 9 findings

- **A-01 HIGH** | `architecture/api-surface.md:148` | Finding row type `mitre_technique: Option<String>`; ADR-006 (accepted, v0.3.0) shipped `mitre_techniques: Vec<String>` in `src/findings.rs:148`. Shipped-code drift.
- **A-02 HIGH** | `architecture/api-surface.md:47-57` | `analyze` flag table omits 3 SHIPPED flags: `--modbus-write-burst-threshold`, `--modbus-write-sustained-threshold` (ADR-005), `--dnp3-direct-operate-threshold` (ADR-007).
- **A-03 HIGH** | `architecture/purity-boundary-map.md:30-58,80-97` | Omits C-24 `dnp3.rs` (SHIPPED v0.6.0) and C-23 `arp.rs`; Pass-12/13 audit updated module-decomposition/criticality/dependency-graph but not purity-boundary-map.
- **A-04 HIGH** | `architecture/system-overview.md:54-69,72-74` | L3 lists only dns/http/tls; omits C-22 Modbus, C-23 ARP, C-24 DNP3 (Modbus+DNP3 SHIPPED); "C-1..C-20" note stale vs canonical 24 components.
- **A-05 MEDIUM** | `architecture/system-overview.md:61` | "mitre.rs ... (15 technique IDs)" stale; canonical 23 current / 25 target.
- **A-06 MEDIUM** | `api-surface.md:141` + `purity-boundary-map.md:38` vs `module-decomposition.md:46` | `decode_packet` shown as current `Result<ParsedPacket>` in two peers, target `Result<DecodedFrame>` in one — unmarked asymmetry (add uniform PLANNED marker).
- **A-07 MEDIUM** | `architecture/dependency-graph.md:94` | etherparse pinned `0.16` vs module-decomposition C-5 "etherparse 0.20" — peer disagreement, no PLANNED marker.
- **A-08 MEDIUM** | `module-decomposition.md:70,72` | C-16/C-22 MITRE lists omit T0888 (Remote System Information Discovery), shipped recon emitter per ADR-005 D12/ADR-006.
- **A-09 LOW** | `purity-boundary-map.md:113` | `mitre.rs` implications omit T0888 + DNP3 IDs (T1691.001, T0827).
- **D-OBS-01 (architect sweep)** | `architecture/decisions/ADR-005-...modbus-tcp.md:105` | "[2,253]" Modbus length range, same defect as D-01; sweep with D-01 fix.

#### Slice B (route: product-owner) — 4 findings

- **B-01 MEDIUM** | `ss-14/BC-2.14.017.md:329` | MITRE Techniques field "T1692.001 — Unauthorized Command Message" uses revoked-T0855 NAME; canonical "Unauthorized Message: Command Message". ID was swept, name was not (DF-SIBLING-SWEEP).
- **B-02 MEDIUM** | `ss-14/BC-2.14.024.md:214` | Same stale name as B-01.
- **B-03 MEDIUM** | `ss-14/BC-2.14.020.md:151-153` (Invariant 6) | Stale Decision-12 counts "SEEDED 21 / EMITTED 13"; canonical 25/17.
- **B-04 LOW** | `ss-14/BC-2.14.020.md:238` | Source-Evidence cites stale "SEEDED=21, EMITTED=13"; annotate as Decision-12-era superseded.
- (NOTE: ss-16 ARP all 15 H1 titles CLEAN; ss-10 anchors all CLEAN; settled ARP semantics intact. Slice B coverage limitation: ss-04/ss-07 bodies not fully opened this pass — spot-checked clean.)

#### Slice C (route: product-owner; architect for VP/mitre.rs facts) — 7 findings

- **C-01 CRITICAL** | `domain/capabilities/cap-09-finding-emission.md:22-43,119` | Finding schema is pre-STORY-100 single-tag form: declares `mitre_technique: Option<String>` + "all four Option fields"; STORY-100 (E-13, completed, v0.3.0) replaced with `mitre_techniques: Vec<String>` (3 Option fields now). Authoritative schema contradicts shipped code. BC ref "BC-2.09.001..006" also stale.
- **C-02 CRITICAL** | `cap-09:14-16,50-99` | "22 emission sites (authoritative)" + "all 22 set timestamp:None" stale: STORY-097/098/099 (E-12, completed) wired timestamp; Modbus (STORY-102..105) + DNP3 (STORY-106..110) add emission sites. Undercounts shipped reality.
- **C-03 HIGH** | `domain/entities/ent-04-findings-output.md:53-62` | E-26 inherits C-01 stale single-tag schema; Pass-10 swept E-27 (16→17 tactics) but not sibling E-26.
- **C-04 HIGH** | `domain/domain-debt.md:49-67` (O-01) | Timestamp debt listed "OPEN/genuine debt on develop today" but STORY-097/098/099 closed it (Option A done).
- **C-05 HIGH** | `prd-supplements/test-vectors.md:19,24` + `prd-supplements/error-taxonomy.md:20,21` | `input-hash:TBD` because `inputs:` lists `src/analyzer/arp.rs` which does not exist in develop HEAD (compute-input-hash errors on missing input). Gate behind PLANNED marker or document TBD rationale (DF-INPUT-HASH-CANONICAL-001).
- **C-06 LOW** | `phase-f1-delta-analysis/arp-analyzer-delta-analysis.md:27-32` | Frontmatter `mitre_research_status` says T0830/T1557.002 "TBD-pending-research/placeholders" though validation landed; may be intentionally frozen F1 snapshot.
- **C-07 LOW** | `error-taxonomy.md:115` (E-ARP-002) | Storm-rate prose "within the average since window-start within the 60-second flap window" awkward double-nesting.
- (NOTE: cap-10 MITRE mapping CLEAN; `src/mitre.rs` anchors all CLEAN @128/179/192-194/100-120/89-91; 17 MitreTactic variants consistent; ARP holdout roll-up 26/24/2 verified; summarize 11-key + reconciliation invariant CLEAN.)

#### Slice D (route: product-owner) — 2 findings + observations

- **D-01 HIGH** | `prd.md:747` (§2.14.A BC-2.14.004 row) | Reject range "[2, 253]"; canonical BC-2.14.004 + VP-022:117 + BC-INDEX:344 all say "[2, 254]". Understates valid upper bound by 1 (len=254 valid). Sweep ADR-005:105 (D-OBS-01) in same burst.
- **D-02 LOW** | `behavioral-contracts/BC-INDEX.md:36` | Status line cites PRD "(v1.15)"; PRD now v1.16 (pass-13 bump). Stale version-pin; "all 283 registered" still accurate.
- (NOTE: Master counts all reconciled PASS — 283 BCs, 24 VPs, 17 tactics, ARP MITRE mappings, release targets, changelog ledger completeness, ADR-008/VP-024 registration all CLEAN.)

**Key observation — trend break:** Passes 12-13 showed 0 CRIT/0 HIGH. Pass 14 surfaced 2
CRITICAL (cap-09 authoritative schema pre-STORY-100; cap-09 emission-site count stale) + 5 HIGH
across architecture docs not reached by prior passes. These are genuine shipped-code-vs-spec drift
items, not ARP-delta defects. Counter remains 0/3.

---

### Pass 15 — 2026-06-13 (whole-corpus, Claude adversary; NOT_CLEAN → REMEDIATED)

**Method:** Whole-corpus fresh-context pass; 4 slices via Claude vsdd-factory:adversary agent.
**Note on method:** agy (Gemini cross-family) was attempted for Pass 15 but its print-mode hit
a ~40-step agentic cap (broad slices read files but never synthesized) AND then hit
RESOURCE_EXHAUSTED (429, individual quota, resets ~5 days). Pass 15 was run via Claude
vsdd-factory:adversary (4 slices) per human direction. Human later provided additional agy quota
for Pass 16+.
**Factory-artifacts HEAD reviewed:** (post-P14-remediation burst).
**Findings:** 8 total — 2 CRITICAL, 1 HIGH, 3 MEDIUM, 2 LOW — ALL REMEDIATED.
**Novelty:** MED — C-01/02/03 holdout-scenarios field-rename is the largest class; it is the
sibling layer MISSED by the Pass-14 field-rename sweep (which scoped only .factory/specs/).
**Convergence counter:** 0/3 (remediation does NOT advance counter; next = Pass 16 via agy).
**Verdict:** NOT_CLEAN → REMEDIATED.

#### Findings and Remediation

**A-01 MED (architect) — VP-INDEX ↔ VP-024 BC-scope reconciled:**
VP-INDEX v2.1 listed VP-024 as covering "BC-2.16.001-015 (6 BCs)"; VP-024 itself scopes to
BC-2.16.001-006 (spoof/GARP/storm/rate anomaly BCs). Also added non-Kani footnote for
BC-2.16.007 (test-sufficient; non-Kani coverage acceptable). VP-INDEX bumped v2.1→v2.2.

**C-04 MED (REGRESSION introduced in Pass-14 Burst-1) — inv-01-core-invariants.md duplicate
top-level `version:` key:**
Pass-14 PO Burst 9 appended a second `version:` YAML key instead of replacing the existing
one, producing malformed frontmatter. Deduped to single `version: v1.2`. BC-2.11.024 Evidence
also corrected (see D-01 below).

**D-01 MED — BC-2.11.024 Evidence "all four Option fields"→3 Option + mitre_techniques.join;
csv.rs anchor 82-85→87-90:**
Evidence text described "all four Option fields" (pre-STORY-100 phrasing) and cited stale
csv.rs line anchor 82-85 (now 87-90 post-refactor). Corrected to "three Option fields"
(timestamp, src_ip, dst_ip) + `mitre_techniques.join("|")` serialization.
BC-2.11.024 bumped v1.6→v1.7.

**B-01 LOW — BC-INDEX BC-2.02.009 narrative prose version pin:**
BC-INDEX v1.23 inline annotation for BC-2.02.009 cited v1.4 (stale; file is v1.6). Corrected
in 3 locations. BC-INDEX bumped v1.23→v1.24.

**C-05 LOW — interface-definitions "four fields"→"four optional-presence fields" clarified:**
"four fields" phrasing was ambiguous; corrected to "four optional-presence fields" to
distinguish from the now-separate `mitre_techniques` (Vec<String>, non-optional presence).
interface-definitions bumped v1.1→v1.2.

**C-01 CRITICAL / C-02 CRITICAL / C-03 HIGH — holdout-scenarios field-rename sweep
(16 HS files fixed):**
The Pass-14 mitre_techniques field-rename sweep scoped to `.factory/specs/` only and MISSED
the `.factory/holdout-scenarios/` sibling layer. All 16 affected HS files corrected:

- **H1 pass** (8 files — field-rename only): HS-032, HS-046, HS-047, HS-056, HS-057, HS-058,
  HS-059, HS-065 — each carried `mitre_technique_id:` and/or `mitre_tactic:` phantom keys
  (never existed in the Finding struct; both introduced as guessed substitutes for the
  pre-rename `mitre_technique` field). Corrected to `mitre_techniques: [...]` array.
  All 8 bumped to v1.1.

- **H2 pass** (8 files — field-rename + additional corrections): HS-074, HS-080, HS-083,
  HS-098, HS-007, HS-009, HS-016, HS-017 — phantom `mitre_technique_id`/`mitre_tactic` keys
  corrected; CSV headers fixed byte-for-byte vs csv.rs (column order, naming); timestamp and
  O-01 claims corrected for HS-007/HS-017 (timestamp now wired per STORY-097..099, not None).

- **H3 pass** (2 frozen eval-run records): `evaluations/chunk1-eval.md` and
  `evaluations/chunk3-eval.md` — historical eval-run records carry expectations referencing
  the old schema. Given these are frozen audit-trail records (not living specs), added dated
  errata sections (ERRATA 2026-06-13) recording the field-rename while preserving original
  history. No history rewritten.

#### Consistency audit (DF-CONSISTENCY-AUDIT-POST-FIXBURST-001)

**Verdict: CONSISTENT** — all 7 dimensions checked:
1. BC-INDEX ↔ BC files: CONSISTENT (BC-INDEX v1.24 titles match).
2. PRD counts ↔ BC-INDEX: CONSISTENT (283 BCs, 24 VPs, 17 tactics).
3. Architecture cross-doc: CONSISTENT (carried from P14 architect sweep).
4. Field-rename saturation (specs layer): CONSISTENT (grep confirms zero `mitre_technique`
   singular in .factory/specs/).
5. Field-rename saturation (holdout-scenarios layer): CONSISTENT (all 16 HS files corrected;
   grep confirms zero phantom `mitre_technique_id`/`mitre_tactic` keys).
6. O-01 closure: CONSISTENT (zero open-framed O-01 remaining).
7. inv-01 YAML structure: CONSISTENT (single `version:` key; no duplicate YAML keys).

Regression confirmed resolved: inv-01 malformed YAML (C-04) deduped; no other P14-churn
regressions found.

#### Process gaps noted [process-gap]

**PG-ARP-F2-003:** Pass-14 field-rename sweep scoped to `.factory/specs/` only and MISSED the
`.factory/holdout-scenarios/` sibling layer — DF-SIBLING-SWEEP must include holdout-scenarios
in the propagation perimeter for any Finding-schema change.

**PG-ARP-F2-004:** A PO remediation burst appended a second `version:` YAML key instead of
replacing it (inv-01), introducing malformed YAML caught only at the next pass — version bumps
must replace-in-place, and a frontmatter dup-key lint should run pre-commit.

#### Artifact versions post-Pass-15

| Artifact | Version | Change |
|----------|---------|--------|
| VP-INDEX | v2.2 | BC-scope for VP-024 corrected; non-Kani footnote added |
| inv-01-core-invariants | v1.2 | Duplicate `version:` key deduped |
| BC-2.11.024 | v1.7 | Evidence "four Option fields"→3 Option + mitre_techniques.join; csv.rs anchor corrected |
| BC-INDEX | v1.24 | BC-2.02.009 version pin corrected (3 locations) |
| interface-definitions | v1.2 | "four optional-presence fields" clarification |
| HS-032, HS-046, HS-047, HS-056, HS-057, HS-058, HS-059, HS-065 | v1.1 | mitre_techniques field-rename (H1 sweep) |
| HS-074, HS-080, HS-083, HS-098, HS-007, HS-009, HS-016, HS-017 | various | mitre_techniques + CSV headers + O-01 corrections (H2 sweep) |
| evaluations/chunk1-eval.md, evaluations/chunk3-eval.md | — | ERRATA 2026-06-13 appended (H3; history preserved) |
| spec-changelog | updated | All P15 version bumps recorded |

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

### [process-gap] PG-ARP-F2-003 — Holdout-scenarios layer excluded from field-rename sweep

Pass-14 field-rename sweep (mitre_techniques corpus sweep) scoped to `.factory/specs/` only and
MISSED the `.factory/holdout-scenarios/` sibling layer. 16 HS files still carried phantom
`mitre_technique_id`/`mitre_tactic` keys (never existed in the Finding struct) and were caught
only at Pass 15.

**Candidate policy codification:** DF-SIBLING-SWEEP-001 must explicitly enumerate
`.factory/holdout-scenarios/` in the propagation perimeter for any Finding-schema change, not
just `.factory/specs/`.

---

### [process-gap] PG-ARP-F2-004 — Version bump must replace-in-place (no YAML key duplication)

Pass-14 PO Burst 9 appended a second `version:` YAML frontmatter key to inv-01-core-invariants.md
instead of replacing the existing one. This produced malformed YAML (duplicate key) that was
caught only at Pass 15 (C-04).

**Candidate policy codification:** Version bumps in YAML frontmatter must use replace-in-place
(Edit tool targeting the exact `version: vX.Y` line), never append. A frontmatter dup-key lint
(e.g., `python3 -c "import yaml; yaml.safe_load(open(f).read())"` over modified .md files)
should run pre-commit.

---

## Notes

- Pass 1 was monolithic (pre-SLICED method adoption for this feature). Passes 2+ use the
  4-slice parallel method per user direction.
- Slice reports from passes 2-5 live in the orchestrator session transcript (see
  PG-ARP-F2-001 above). This file captures the distilled per-pass summary.
- The SLICED method is proving effective at surfacing cross-doc consistency issues (Slice D)
  that monolithic passes miss — passes 2-4 all had their largest finding cluster in Slice D.
