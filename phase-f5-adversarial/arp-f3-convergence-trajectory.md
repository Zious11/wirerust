---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-14T00:00:00Z
feature: arp-analyzer
cycle: feature-arp-v0.7.0
phase: F3-story-decomposition-adversarial-convergence
inputs: [phase-f5-adversarial/adversarial-reviews/]
input-hash: TBD
traces_to: STATE.md
---

# Convergence Trajectory — ARP Analyzer F3 Story Decomposition Adversarial Convergence

## Feature Context

**Feature:** ARP security analyzer + etherparse 0.16→0.20.1 migration (sub-delta A).
**Release target:** v0.7.0.
**F3 Scope:** STORY-111..115 (epic E-16, 47 pts, linear chain). 15 SS-16 BCs. Waves 40-44 holdouts.
**MITRE:** T0830 ICS AiTM (primary) + T1557.002 Enterprise ARP Cache Poisoning (secondary).

## Adversarial Method

**STRICT WHOLE-CORPUS method** (4 fresh-context slices per pass):
- Slice A = architecture + VPs
- Slice B = all 283 BC bodies
- Slice C = domain / holdout / MITRE / stories
- Slice D = PRD + indexes + changelog ledger

**Bar:** ZERO findings of ANY severity (incl LOW) across ALL 4 slices; 3 CONSECUTIVE
fully-clean passes required.

**Adversary:** CLAUDE (Agent tool, `vsdd-factory:adversary`). agy UNUSABLE (40-step cap +
quota-exhausted). Absolute paths, no cd (DF-ADVERSARY-METHODOLOGY-001).

---

## Pass History (Passes 1–32)

### Pass 1 (2026-06-14)

Slice A: findings TBD. Slice B: findings TBD. Slice C: findings TBD. Slice D: findings TBD.
Detail: first F3 pass after STORY-111..115 created. [Inline detail archived from STATE.md §B.]

### Passes 1–21 (archived from STATE.md §B inline narrative)

Full per-pass narrative for Passes 1–21 was maintained inline in STATE.md §B during active
convergence work. Archived to this file as a block on 2026-06-14 (Pass-32 compaction).

Key milestones in this range:
- **Pass 17** = first fully-clean pass (all 4 slices ZERO).
- **Pass 18** broke streak (VP title-sync finding).
- **Passes 19–21** each surfaced and remediated genuine items. All REMEDIATED; streak reset 0/3.

### Passes 22–28 (archived from STATE.md §B inline narrative)

Full inline pass narrative maintained in STATE.md §B through Pass-28. Archived to this file
on 2026-06-14 (Pass-32 compaction).

- **Pass-22 (2026-06-14):** 5 findings cosmetic; 0C/0H; version-pin hardened; REMEDIATED. Clean-streak 0/3.
- **Pass-23 (2026-06-14):** 5 findings; B/C/D CLEAN; Slice-A only; 0C/0H; REMEDIATED. Clean-streak 0/3.
- **Pass-24 (2026-06-14):** 4 findings: D-01 DNP3-C24 sweep genuine + 3 self-induced; 0C/1H; B+C CLEAN; REMEDIATED. (DNPXX→DNP3 rename regression caught and reverted.) Clean-streak 0/3.
- **Pass-25 (2026-06-14):** 2 findings; A/B/C CLEAN; changelog-path flush; 0C/0H; REMEDIATED. Clean-streak 0/3.
- **Pass-26 (2026-06-14) REMEDIATION COMPLETE:** Slice A ZERO [5th consec]; Slice B 1 MED [BC-2.15.007 EC-008 frame_len Some(290)→Some(291)]; Slice C ZERO [3rd consec]; Slice D 1 HIGH [STORY-INDEX total_points 447→457, wave-TOTAL 442→452, epic-TOTAL 447→457, pre-ARP 400→410 — grand totals stale; root cause: pre-ARP grand total was 10 low]; ALL REMEDIATED; BC-2.15.007 v1.4; STORY-INDEX v1.5; dep-graph 452/457; STATE summary 457 pts. Post-Pass-26 proactive consistency-flush COMPLETE (16 latent defects remediated). Clean-streak UNCHANGED 0/3.
- **Pass-27 (2026-06-14) REMEDIATION COMPLETE:** Slice A ZERO 6th-consec, Slice C ZERO converged; Slice B: SS-12×6 version-field 1.3→1.4 [self-introduced by flush burst] + BC-2.15.023 FC 0x13 STOP_APPL→SAVE_CONFIG; Slice D: spec-changelog phantom paths pass-24/25, wave-schedule title scope; ALL REMEDIATED. Clean-streak 0/3.
- **Pass-28 (2026-06-14) REMEDIATION COMPLETE:** Slice A ZERO 7th-consec, Slice C ZERO converged; Slice B: BC-2.15.007→v1.5 + BC-2.15.009→v1.6 [Related-BC cross-ref .020→.016] + BC-2.15.014→v1.9 + BC-2.15.015→v1.8 + BC-2.15.024→v1.6 [stale (NEW) markers removed; SS-15 now fully de-NEW-ed]; Slice D: feature/wave-schedule.md→v1.3 [T0855→T1692.001 ×3 live occurrences; HS-INDEX:322 enforcement rule corrected]; ALL REMEDIATED. Clean-streak 0/3.

### Pass 29 (2026-06-14)

Slice A: ZERO [8th-consec]. Slice B: ZERO [2nd clean/converged]. Slice C: ZERO [converged].
Slice D: 1 MED [HS-INDEX HS-W39-007 VP-023 Kani BC scope "BC-2.15.001..008"→".001..007";
BC-2.15.008 is unit-test-only, no Kani harness; HS-INDEX→v1.7]. CLEANEST pass yet — 3/4 slices
clean. REMEDIATED. Clean-streak 0/3.

### Pass 30 (2026-06-14)

Slice A: ZERO [9th-consec]. Slice B: ZERO [converged]. Slice C: INVALID [env glitch —
empty-dir confabulation; .factory verified populated: 69 STORY files, 101 HS files; DISCARDED].
Slice D: 1 HIGH [PRD v1.25 ledger gap — delta note+changelog entry verified present; anchor
resolves] + 1 LOW [BC-2.15.017 v1.3 text tautology fixed]; 1 FALSE-POSITIVE rejected
[BC-2.15.017:122 path CORRECT]; genuine findings REMEDIATED. Clean-streak 0/3.

### Pass 31 (2026-06-14)

Slice A: ZERO [10th-consec]. Slice B: ZERO [converged]. Slice C: ZERO [mount-guard PASSED;
P30 env glitch confirmed resolved]. Slice D: ZERO. ALL 4 SLICES ZERO. FULLY CLEAN.
Clean-streak 0/3 → **1/3**. FIRST fully-clean pass. NEXT = Pass 32 (clean-streak attempt 2/3).

### Pass 32 (2026-06-14)

Slice A: ZERO [11th-consec]. Slice B: ZERO. Slice C: 1 MEDIUM [STORY-115 — ArpAnalyzer
analyzer field name `storm_findings_count` does not match STORY-113 canonical field declaration
(STORY-113:254 `storm_findings: u64`) + sibling-field convention + BC-2.16.010 summarize key;
cross-story field-name drift]. Slice D: ZERO.

**Finding:** 1 MEDIUM total (Slice C). NOT a clean pass.

**Remediation:** STORY-115 → v1.1 (2026-06-14). Changed 6 occurrences of `storm_findings_count`
→ `storm_findings` to match STORY-113 canonical declaration, sibling convention, and
BC-2.16.010 summarize key. Input-hash UNCHANGED (5ca9835 MATCH — body edit, not input-file
edit). Sibling sweep: ZERO remaining live `storm_findings_count` in corpus. REMEDIATED.

**Clean-streak RESET: 1/3 → 0/3.** NEXT = Pass 33 (clean-streak attempt 1/3 restart).

### Pass 33 (2026-06-14)

Slice A: ZERO [12th-consec]. Slice B: 1 MEDIUM [BC-2.15.024 Related-BCs descriptor for
BC-2.15.015 incorrectly listed `parse_errors` in the 300s-expiry reset set. The windowed
field BC-2.15.015 resets alongside `malformed_anomaly_emitted` is `malformed_in_window`,
NOT `parse_errors`. `parse_errors` is the LIFETIME/monotonic counter per BC-2.15.024
Invariant 1 / PC5 / Architecture Anchors + BC-2.15.015 + dnp3.rs:984-995 (which resets
six windowed fields only, never parse_errors). Residual stale cross-ref from v1.1
two-counter split.]. Slice C: ZERO. Slice D: ZERO [3rd-consec].

**Finding:** 1 MEDIUM total (Slice B). NOT a clean pass.

**Remediation:** BC-2.15.024 → v1.7 (2026-06-14). Related-BCs descriptor for BC-2.15.015
corrected: "parse_errors and malformed_anomaly_emitted reset at 300s expiry" →
"malformed_in_window and malformed_anomaly_emitted reset at 300s expiry by BC-2.15.015
(parse_errors is the LIFETIME counter — NOT in the reset set, per Invariant 1)".
DF-SIBLING-SWEEP-001: no other artifact in .factory/ wrongly lists parse_errors in a
reset set — sibling sweep CLEAN. REMEDIATED.

**clean-streak 0/3.** NEXT = Pass 34 (clean-streak attempt 1/3).

### Post-Pass-33 SS-15 Proactive Consistency Flush (2026-06-14)

**NOT an adversary pass.** Proactive pre-Pass-34 flush targeting recurring SS-15 Related-BCs
/ counter-semantics residue class. clean-streak UNCHANGED at 0/3. NEXT = Pass 34.

**6 findings remediated:**

**FIX A [MED] — BC-2.15.014 → v2.0: EC-006 + Invariant 7 four-field → six-field reset:**
EC-006 and Invariant 7 enumerated only four windowed fields being reset at 300s expiry
(`restart_event_count`, `block_event_count`, `block_finding_emitted_this_window`,
`loss_of_control_emitted`). BC-2.15.015 v1.5 (Invariant 6) added `malformed_in_window` and
`malformed_anomaly_emitted` to the reset set; BC-2.15.014's EC-006 and Invariant 7 were never
updated. Corrected to the canonical SIX windowed fields. `parse_errors` explicitly noted as
LIFETIME/monotonic (NOT reset). Verified against BC-2.15.015 Inv 6 and
src/analyzer/dnp3.rs maybe_expire_correlation_window lines 984-991.

**FIX B [MED] — BC-2.15.014 → v2.0: reciprocal Related-BC → BC-2.15.016 added:**
BC-2.15.016 already cites BC-2.15.014; reciprocal citation in BC-2.15.014 was missing.

**FIX C [MED] — BC-2.15.016 → v1.6: reciprocal Related-BC → BC-2.15.010 added:**
BC-2.15.010 already cites BC-2.15.016; reciprocal citation in BC-2.15.016 was missing.

**FIX D [MED] — BC-2.15.015 → v1.9: reciprocal Related-BC → BC-2.15.024 added:**
BC-2.15.024 already cites BC-2.15.015; reciprocal citation in BC-2.15.015 was missing.

**FIX E [MED] — BC-2.15.022 → v1.4: reciprocal Related-BC → BC-2.15.016 added:**
BC-2.15.016 already cites BC-2.15.022; reciprocal citation in BC-2.15.022 was missing.

**FIX F [LOW] — BC-2.15.012 → v1.4 + BC-2.15.023 → v1.6: FC 0x13 SAVE_CONFIG → SAVE_CONFIGURATION:**
IEEE 1815-2012 canonical name for FC 0x13 is SAVE_CONFIGURATION (full name). The abbreviated
SAVE_CONFIG label was introduced in BC-2.15.023 v1.5 (Pass-27 fix from STOP_APPL). Sibling FCs
0x14 ENABLE_UNSOLICITED and 0x15 DISABLE_UNSOLICITED are already unabbreviated throughout.
No shipped `SAVE_CONFIG` symbol in src/ (grep confirmed zero hits). Sealed v1.5 changelog
history entry in BC-2.15.023 retained verbatim (records the STOP_APPL→SAVE_CONFIG correction
— immutable audit trail; not updated).

**Sweep classes checked (CLEAN):**
- Class 3 (frame arithmetic): CLEAN
- Class 4 (constants/labels): CLEAN (SAVE_CONFIGURATION correction applied above; no other FC abbreviation drift found)
- Class 6 (version/changelog sync): CLEAN

### Pass 34 (2026-06-14)

Slice A: ZERO [13th-consec]. Slice B: ZERO [converged]. Slice C: ZERO [10th-consec].
Slice D: 1 LOW [spec-changelog entry [prd-v1.25-ss15-titlesync-2026-06-14] missing
Artifacts-changed table — presentational sibling-consistency gap; all other recent ACTIVE
entries carry the table in the standard Artifact/Change format].

**Finding:** 1 LOW total (Slice D). NOT a clean pass.

**Remediation:** spec-changelog.md [prd-v1.25-ss15-titlesync-2026-06-14] entry updated —
Artifacts-changed table added matching pass-29 entry format (2 rows: prd.md + spec-changelog.md).
Paths resolve on disk. Sibling sweep: all other recent ACTIVE entries already carry the table —
CLEAN. Corpus substantively converged: Slices A/B/C achieved ZERO for 13th/converged/10th
consecutive pass respectively; only presentational LOW surfaced at Slice D. REMEDIATED.

**clean-streak 0/3.** NEXT = Pass 35 (clean-streak attempt 1/3).

### Pass 35 (2026-06-14)

Slice A: ZERO [14th-consec]. Slice B: ZERO [converged]. Slice C: ZERO [converged].
Slice D: 1 LOW [d-069 changelog entry [d-069-icsimpact-display-impact-ics-2026-06-14]
Artifacts table carried stale raw PRD line-pins: "PRD §85 (v1.5 delta note)" with
trailing "line 85" file reference, and "PRD §882 (F2 addition note)" with trailing
"line 882" file reference. Line 882 is Modbus prose; actual IcsImpact note is in PRD §2.15
"New ICS tactic variant" (at prd.md:974). Line 85 is a different section. The Artifacts
table column carried raw line numbers as location identifiers — a drift-prone pattern].

**Finding:** 1 LOW total (Slice D). NOT a clean pass.

**Remediation:** spec-changelog.md [d-069-icsimpact-display-impact-ics-2026-06-14] Artifacts
table updated — both rows de-pinned to §-section/concept anchors:
- "PRD §85 (v1.5 delta note)" → "PRD v1.5 delta note (IcsImpact variant)"
- "PRD §882 (F2 addition note)" → "PRD §2.15 'New ICS tactic variant' note (IcsImpact)"
Trailing "line NNN" file references removed from both rows.

Sweep of all 2026-06-12+ ACTIVE entries: no other drifted live-state spec line-pins found.
Remaining numerics in spec-changelog are sealed historical correction records, src refs,
§-anchors, ~approximates, or intra-BC audit notes — do NOT flag. Recurring Slice-D
changelog line-pin drift class flushed via this de-pin sweep.

Note: P33/P34/P35 all had A/B/C ZERO with only a single Slice-D LOW each (asymptotic
changelog-cosmetic churn). This de-pin sweep eliminates the drift class at its root.

**clean-streak 0/3.** NEXT = Pass 36 (clean-streak attempt 1/3).

### Pass 36 (2026-06-14)

Slice A: ZERO [15th-consec]. Slice B: ZERO [converged]. Slice C: ZERO [converged].
Slice D: ZERO. ALL 4 SLICES ZERO. Mount-guards PASSED. FULLY CLEAN.

**Finding:** ZERO findings total. FULLY CLEAN pass. No remediation required.

Post-P35 changelog de-pin flush (active-zone 2026-06-12+ line-pins removed) + SS-15 proactive
consistency flush (post-P33: six-field reset set, reciprocal Related-BCs, SAVE_CONFIGURATION)
together eliminated the asymptotic Slice-D/B churn that had persisted through P33/P34/P35.

2nd fully-clean pass overall (P31 was first; P32 reset the streak with STORY-115 storm_findings field drift).

**Clean-streak 0/3 → 1/3.** NEXT = Pass 37 (clean-streak attempt 2/3 — need 2 more consecutive).

---

## Summary Table

| Pass | Slice A | Slice B | Slice C | Slice D | Total | Clean-streak | Notes |
|------|---------|---------|---------|---------|-------|-------------|-------|
| 1–16 | varies | varies | varies | varies | varies | 0/3 | Early passes; P17 first clean pass |
| 17 | ZERO | ZERO | ZERO | ZERO | 0 | 1/3 | FIRST fully-clean pass |
| 18 | — | — | — | — | >0 | 0/3 | VP title-sync broke streak |
| 19–21 | varies | varies | varies | varies | varies | 0/3 | Genuine items remediated |
| 22 | ZERO | varies | varies | varies | >0 | 0/3 | Version-pin hardened |
| 23 | varies | ZERO | ZERO | ZERO | >0 | 0/3 | Slice A only |
| 24 | varies | ZERO | ZERO | varies | >0 | 0/3 | DNPXX regression caught+reverted |
| 25 | varies | ZERO | ZERO | varies | >0 | 0/3 | Changelog-path flush |
| 26 | ZERO | 1 MED | ZERO | 1 HIGH | 2 | 0/3 | STORY-INDEX totals stale |
| 27 | ZERO | varies | ZERO | varies | >0 | 0/3 | SS-12×6 version-field lag |
| 28 | ZERO | varies | ZERO | varies | >0 | 0/3 | SS-15 de-NEW-ed; wave-schedule T0855→T1692.001 |
| 29 | ZERO | ZERO | ZERO | 1 MED | 1 | 0/3 | HS-W39-007 VP-023 scope |
| 30 | ZERO | ZERO | INVALID | 1H+1L | 2+ | 0/3 | P30 Slice C env glitch (discarded) |
| 31 | ZERO | ZERO | ZERO | ZERO | 0 | **1/3** | FULLY CLEAN — first of 3 needed |
| 32 | ZERO | ZERO | 1 MED | ZERO | 1 | 0/3 | STORY-115 field storm_findings_count→storm_findings; REMEDIATED |
| 33 | ZERO | 1 MED | ZERO | ZERO | 1 | 0/3 | BC-2.15.024 reset-set cross-ref parse_errors→malformed_in_window; REMEDIATED v1.7 |
| POST-P33 SS-15 FLUSH | — | — | — | — | 6 | 0/3 (UNCHANGED) | PROACTIVE pre-Pass-34 flush: BC-2.15.014 v2.0 four→six-field reset (EC-006+Inv7); reciprocal Related-BCs 014↔016/016↔010/015↔024/022↔016; FC 0x13 SAVE_CONFIGURATION in BC-2.15.012 v1.4+BC-2.15.023 v1.6. Sweep classes 3/4/6 CLEAN. NOT an adversary pass; clean-streak UNCHANGED 0/3; NEXT = Pass 34. |
| 34 | ZERO (13th-consec) | ZERO (converged) | ZERO (10th-consec) | 1 LOW | 1 | 0/3 | Slice D 1 LOW: spec-changelog [prd-v1.25-ss15-titlesync-2026-06-14] entry missing Artifacts-changed table (presentational sibling-consistency gap vs all peer ACTIVE entries). REMEDIATED: table added to spec-changelog.md. Corpus substantively converged — only presentational LOW surfaced. NEXT = Pass 35 (clean-streak attempt 1/3). |
| 35 | ZERO (14th-consec) | ZERO (converged) | ZERO (converged) | 1 LOW | 1 | 0/3 | Slice D 1 LOW: d-069 changelog entry [d-069-icsimpact-display-impact-ics-2026-06-14] carried stale PRD raw line-pins in Artifacts table (§882→"PRD §2.15 'New ICS tactic variant' note (IcsImpact)" and §85→"PRD v1.5 delta note"). REMEDIATED: both rows de-pinned to §-section/concept anchors in spec-changelog.md. Swept all 2026-06-12+ ACTIVE entries — no other drifted live-state spec line-pins found (remaining numerics are sealed historical correction records, src refs, §-anchors, ~approximates, or intra-BC audit notes). Recurring Slice-D changelog line-pin drift class flushed via de-pin sweep. Note: P33/P34/P35 all had A/B/C ZERO with only a single Slice-D LOW each (asymptotic changelog-cosmetic churn) — now flushed. clean-streak 0/3. NEXT = Pass 36 (clean-streak attempt 1/3). |
| 36 | ZERO (15th-consec) | ZERO (converged) | ZERO (converged) | ZERO | 0 | **1/3** | FULLY CLEAN — all 4 slices ZERO; mount-guards PASSED. Post-P35 changelog de-pin flush + SS-15 flush eliminated the recurring Slice-D/B churn. 2nd fully-clean pass overall (P31 was first, reset by P32 storm_findings). Clean-streak 0/3→1/3. NEXT = Pass 37 (clean-streak attempt 2/3 — need 2 more consecutive). |
