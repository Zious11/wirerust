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
| 16 (whole-corpus, Claude) | 2026-06-13 | 7 | 0 | 0 | 5 | 2 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 17 (whole-corpus, Claude) | 2026-06-13 | 10 | 3 | 2 | 2 | 3 | MED | 0/3 | NOT_CLEAN→REMEDIATED |
| 18 (whole-corpus, Claude) | 2026-06-13 | 9 | 0 | 3 | 2 | 4 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 19 (whole-corpus, Claude) | 2026-06-13 | 15 | 0 | 8 | 2 | 5 | HIGH | 0/3 | NOT_CLEAN→PARTIAL |
| 20 (whole-corpus, Claude) | 2026-06-13 | 7 | 0 | 1 | 3 | 3 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 21 (whole-corpus, Claude) | 2026-06-13 | 5 | 0 | 0 | 4 | 1 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 22 (whole-corpus, Claude) | 2026-06-13 | 5 | 0 | 0 | 1 | 4 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 23 (whole-corpus, Claude) | 2026-06-13 | 5 | 0 | 0 | 1 | 4 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 24 (whole-corpus, Claude) | 2026-06-13 | 4 | 0 | 1 | 2 | 1 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 25 (whole-corpus, Claude) | 2026-06-13 | 2 | 0 | 0 | 2 | 0 | LOW | 0/3 | NOT_CLEAN→REMEDIATED |
| 26 (whole-corpus, Claude) | 2026-06-13 | 0 | 0 | 0 | 0 | 0 | NONE | **1/3** | **CLEAN** |
| 27 (whole-corpus, Claude) | 2026-06-13 | 2 | 0 | 0 | 2 | 0 | MED | **0/3 (reset from 1/3)** | NOT_CLEAN→REMEDIATED |
| 28 (whole-corpus, Claude) | 2026-06-13 | 0 | 0 | 0 | 0 | 0 | NONE | **1/3** | **CLEAN** |
| 29 (whole-corpus, Claude) | 2026-06-13 | 3 | 0 | 0 | 2 | 1 | MED | **0/3 (reset from 1/3)** | NOT_CLEAN→REMEDIATED |
| 30 (whole-corpus, Claude) | 2026-06-13 | 5 | 0 | 4 | 1 | 0 | HIGH | **0/3** | NOT_CLEAN→REMEDIATED |
| 31 (whole-corpus, Claude) | 2026-06-13 | 0 | 0 | 0 | 0 | 0 | NONE | **1/3** | **CLEAN** |
| 32 (whole-corpus, Claude) | 2026-06-13 | 0 | 0 | 0 | 0 | 0 | NONE | **2/3** | **CLEAN** |
| 33 (whole-corpus, Claude) | 2026-06-13 | 0 | 0 | 0 | 0 | 0 | NONE | **3/3 CONVERGED** | **CLEAN** |

## Trajectory Shorthand

`15→20→~8→~15→~6→~4→~4→~7→~4→~6→~5→~18→~8→~22(P14: 2C/5H NEW corpus-debt; trend broke; ARP delta clean 6th pass)→P15(8 findings: holdout-layer field-rename + regression; REMEDIATED)→P16(7: 0C/0H, sibling-sweep misses; REMEDIATED; Slice B CLEAN all 283 BCs + field-rename verified)→P17(10: holdout MITRE-counts + module-decomposition peer; REMEDIATED; Slice B CLEAN 2nd)→P18(9: ss-05 anchor-drift + indicatif + STORY-INDEX; 0C/3H; REMEDIATED; arp.rs+holdout pre-flush verified clean)→P19(15: corpus-wide anchor-drift; 0C/8H; PARTIAL — ss-07-full+remaining-BC pending)→ P20(7: anchor-drift flushed, ss-04/ss-12 closed; 0C/1H; Slices A+C CLEAN; REMEDIATED)→P21(5 cosmetic; 0C/0H; A+C CLEAN 2nd consecutive; REMEDIATED)→P22(5 valid; 0C/0H; cosmetic; version-pin hardened; REMEDIATED)→P23(5; B/C/D CLEAN; Slice-A only; 0C/0H; REMEDIATED)→P24(4: D-01 DNP3-C24 sweep genuine + 3 self-induced; 0C/1H; B+C CLEAN; REMEDIATED)→P25(2; A/B/C CLEAN; changelog-path flush; 0C/0H; REMEDIATED)→ P26 CLEAN 1/3 (all 4 slices zero findings; corpus-wide debt flushed P14-25) → P27 reset 1/3→0/3 (HS-008 kill-chain + HS-INDEX pin; holdout-pin-hardened) → P28 CLEAN 1/3 (restart after P27 reset; all 4 slices zero findings; on post-P27 corpus with holdout kill-chain + version-pin fixes) → P29(3: DNP3 T1692.001 + PRD FC-0x17 + anchor; reset 1/3→0/3; REMEDIATED) → P30(5: FlowKey non-existent-accessor HIGH ×3 + STORY input-hash dup-key HIGH + ADR-006 FC-0x17; 4H; REMEDIATED — grind found real bugs) → P31 CLEAN 1/3 (restart; P30 HIGH fixes held; all 4 slices zero findings) → P32 CLEAN 2/3 (2nd consecutive) → P33 CLEAN 3/3 CONVERGED (F2 strict-whole-corpus gate satisfied after 33 passes)`

Severity profile: CRITICAL count: 4→5→0→0→0→0→0→0→0→0→0→0→0→2→2→0→3→0→0→0→0→0→0 — DECAYING on CRITICAL
(0 for 7 of last 8 passes: P16+P18+P19+P20+P21+P22+P23+P24).
HIGH count: 8→7→~6→~5→1→2→~4→2→0→1→1→0→0→5→1→0→2→3→8→1→0→0→0→1 — P21+P22+P23 consecutive 0H;
P24 1H (D-01 genuine DNP3 systematic mislabel; 7th consecutive 0-CRIT).
MEDIUM count: 3→8→~2→~10→~5→2→0→4→~4→~5→~4→~18→~8→~11→3→5→2→2→2→3→4→1→2 — P24 B+C CLEAN;
Slices B+C clean; D-01 genuine systematic sweep. Substantively converged.
Trend BROKE at Pass 14 — Passes 12-13 showed 0 CRIT/0 HIGH; Pass 14 surfaced 2 CRITICAL + 5 HIGH.
DECAYING P14→P16: 2C/5H → 2C/1H → 0C/0H. NON-MONOTONIC at P17: 3C/2H (new corpus corner:
holdout-scenarios MITRE-catalog count assertions; module-decomposition never deep-reviewed for
PLANNED/ARP markers). Slice B CLEAN 2nd consecutive (P16+P17). P18: 0C/3H — ss-05 dispatcher
anchor-drift (systematic: all 9 ss-05 BCs stale due to Modbus/DNP3 code insertions shifting
dispatcher.rs line offsets) + indicatif 0.17→0.18 version self-invariant + STORY-INDEX 48-vs-49
ambiguity. arp.rs pre-flush STORY-112 anchor-uniformity verified CLEAN by Slice A (proactive fix
verified). Holdout tree (101 files, Slice C) CLEAN. P19: 0C/8H — PG-ARP-F2-007 confirmed
CORPUS-WIDE: ss-09 (findings.rs; 6 BCs re-anchored + BC-2.09.003 Possible-verdict variant added),
ss-06 (http.rs; ALL 26 BCs re-anchored), ss-04 (BC-2.04.055/BC-2.04.024/020 mod.rs),
ss-07 partial (BC-2.07.037/016/008 off-by-one; full tls.rs re-anchor PENDING), purity-boundary-map
v1.4→v1.5 (A-01/A-02), VP-sweep 9 files re-anchored (vp-003/004/006/010/011/013/014/015/021),
HS-009 T1083→Discovery (C-01), nfr-catalog/nfr-story-map/inv-01 INV-2 dispatcher anchors,
domain straggler sweep. Slice D CLEAN. Ground truth src maps (dispatcher.rs/http.rs/tls.rs/
findings.rs/mod.rs/segment.rs/lifecycle.rs/flow.rs) recorded. ss-07 FULL re-anchor + audit
ss-01/02/04-rest/08/11/12/13 STILL PENDING before Pass 20. Counter 0/3.

## Convergence Counter

**3/3 CONVERGED.** **Counter SATISFIED at Pass 33 (third consecutive clean pass; P31+P32+P33 all-4-slice zero findings). F2 STRICT-WHOLE-CORPUS ADVERSARIAL GATE SATISFIED after 33 passes (P1-P33).**
**STRICT WHOLE-CORPUS mode** (human-elected 2026-06-12; scope extended 2026-06-13): zero
findings of ANY severity (including LOW) across the ENTIRE spec corpus (not just ARP delta)
required for 3 consecutive clean passes. 33 passes run. Pass 14 REMEDIATED (22 findings:
mitre_techniques field-rename corpus sweep + O-01 closure propagation + architect ×2 + PO ×10
bursts + consistency audit CONSISTENT). Pass 15 REMEDIATED (8 findings: holdout-scenarios
field-rename sweep [C-01/02/03, 16 files] + inv-01 YAML regression [C-04] + VP-024 scope
reconciliation [A-01] + 4 more; consistency audit CONSISTENT all 7 dimensions). Pass 16
REMEDIATED (7 findings: VP-021 added to proptest list [A-01]; decode_packet diagrams STORY-111
markers [A-02]; api-surface STORY-114→STORY-111 [A-03]; dependency-graph 282→pointer [A-04];
A-05 DISCARDED (ADR-008 proposed correct); chunk3-reeval ERRATUM [C-01]; ADR-005 :74 [2,254]
[D-01]; 6 remediated, 1 discarded). Pass 17 REMEDIATED (10 findings: module-decomposition
PLANNED markers [A-01/A-02/A-03/A-04]; HS-025/HS-008/HS-009 holdout MITRE-catalog count
assertions [C-01/C-02/C-03/C-04]; nfr-catalog NFR-OBS-010 disambiguation [D-01]; domain-spec
§Summary-Metrics erratum [D-02]; 10 remediated). Pass 18 REMEDIATED (9 findings:
dependency-graph indicatif 0.17→0.18 [A-01]; verification-coverage-matrix VP-023 lock-evidence
[A-02]; purity-boundary-map VP-024 arp.rs bullet [A-03]; ss-05 systematic dispatcher anchor
re-sync all 9 BCs [B-01/B-02]; BC-2.05.007/008 4-analyzer guard prose [B-03]; STORY-INDEX
48-vs-49 clarification [C-01/D-01]; cap-10 changelog self-ref line anchors [C-02];
BC-2.04.055 on_data :144→:245 [CARRY-OVER]; 0C/3H). Proactive pre-pass fix (arp.rs/C-23
PLANNED STORY-111→STORY-112 in system-overview v1.3→v1.4 + purity-boundary-map v1.2→v1.3)
verified CLEAN by Slice A. Holdout tree 101 files (Slice C) CLEAN. Trajectory P14-18:
2C/5H → 2C/1H → 0C/0H → 3C/2H → 0C/3H. Decaying on CRITICAL (0 for 2 of last 3).
Pass 19 REMEDIATED (15 findings: 0C/8H/2M/5L; PG-ARP-F2-007 corpus-wide anchor re-sync; ~128
files re-anchored across two batches; ss-07-full 35 BCs + ss-04-partial 21 BCs + ss-11 10 BCs;
ss-01/02/08/13 CONFIRMED CLEAN). Trajectory P14-19:
2C/5H → 2C/1H → 0C/0H → 3C/2H → 0C/3H → 0C/8H. REGRESSION at P19 HIGH.
Pass 20 REMEDIATED (7 findings: 0C/1H/3M/3L; cap-09 version straggler + ss-04-remainder
BC-2.04.012/013/014 + ss-12 BC-2.12.005 + ADR-008 D-02 matrix-label; anchor-drift class
PG-ARP-F2-007 FLUSHED corpus-wide; ss-04/ss-12 COMPLETE; Slices A+C CLEAN;
over-correction spot-check PASSED — all P19 sweeps verified correct, no anchor moved to wrong
line). Trajectory P17-20: 3C/2H → 0C/3H → 0C/8H → 0C/1H. DECAYING strongly.
Pass 21 REMEDIATED (5 findings: 0C/0H/4M/1L; all cosmetic/ledger hygiene — no behavioral
changes). B-01 LOW: BC-INDEX ss-11 stray blank line between BC-2.11.001 and BC-2.11.002
split Markdown table (BC-INDEX v1.24→v1.25). D-01 MED: spec-changelog Pass-13 ledger ARCH-INDEX
path `behavioral-contracts/ARCH-INDEX.md` → `architecture/ARCH-INDEX.md`. D-02 MED:
spec-changelog Pass-13 ledger vp-005 slug `vp-005-no-panic-guarantee.md` →
`vp-005-sni-four-way-classification.md`. D-03 MED: spec-changelog Pass-13 ledger vp-008 slug
`vp-008-all-analyzers-pure.md` → `vp-008-decode-packet-no-panic.md`. D-04 LOW: PRD body
version-history missing delta notes for 1.13/1.14/1.15/1.16/1.18; notes added (prd.md
v1.18→v1.19). Slices A+C CLEAN (2nd consecutive clean for both). Trajectory P19-21:
0C/8H → 0C/1H → 0C/0H. DECAYING strongly.

Pass 22 REMEDIATED (8 raw findings; 3 discarded as no-action/NON-BLOCKING per adversary:
A-03 verified anchor no-action, A-04 ADR-008 proposed NON-BLOCKING, A-05 VP-021 dual-axis
convention documented. 5 valid findings: 0C/0H; cosmetic + 1 stale count). C-01 MED:
domain-debt O-04 technique count 21→23 — Feature #8 DNP3 added T1691.001 + T0827 to
technique_info; count was not propagated from mitre.rs SEEDED_TECHNIQUE_ID_COUNT=23 (domain-debt
v1.2→v1.3). A-01 LOW: verification-architecture Pass-22 modified entry wording hardened
(v1.5→v1.6). A-02 LOW: verification-coverage-matrix VP-024 draft Coverage Note added
(v1.4→v1.5). D-01 LOW: BC-INDEX PRD version-pin dropped for robustness — self-induced lag from
P21 prd v1.19 bump; now version-agnostic (v1.25→v1.26). B-01 LOW: BC-INDEX double-blank before
ss-12 removed (v1.25→v1.26). Proactive version-citation robustness sweep run across all
.factory/specs/ files — only 1 current-state cross-doc version-pin found (BC-INDEX PRD line);
now dropped. Future version-lag churn minimized. PG-ARP-F2-008 noted: 5th consecutive
0-CRIT/HIGH; corpus substantively converged; remaining churn cosmetic. Trajectory P20-22:
0C/1H → 0C/0H → 0C/0H. 5th consecutive 0-CRIT/HIGH pass.
Counter 0/3 — remediation does NOT advance counter. Next = whole-corpus Pass 23 via Claude
adversary (4 slices STRICT) — first-clean candidate. If clean → 1/3 streak begins.

Pass 23 REMEDIATED (5 findings; Slices B/C/D CLEAN; Slice A only; 0C/0H). KEY: 3 of 4 slices
returned CLEAN — first pass with majority-clean slices. A-01 MED was self-induced churn from
P22 A-02 (VP-024 lock-note cited wrong story STORY-112→STORY-113; now correct per §6).
A-02 LOW: decoder.rs Sub-A attribution footnote in verification-coverage-matrix. A-03 LOW:
VP-005 harness skeleton markdown code-fence missing closing backticks in verification-architecture.
A-04 LOW: C-22 Modbus technique enumeration in module-criticality harmonized with C-23/C-24 style.
A-05 LOW: arp-architecture-delta §6 draft-as-authoritative intentionality note added to prevent
re-flagging. All 5 findings architect-routed (Slice A). Trajectory P21-P23: 0C/0H → 0C/0H → 0C/0H.
Substantively and cosmetically near-converged. Counter 0/3. Next = whole-corpus Pass 24 via Claude
adversary — strong first-clean candidate (B/C/D were clean in P23).

Pass 24 REMEDIATED (4 findings; Slices B+C CLEAN; 0C/1H). 3 of 4 findings were self-induced
churn from P23 (PG-ARP-F2-008). D-01 HIGH (PO, genuine, substantive): systematic DNP3 component
mislabel — ALL 24 ss-15 BCs labeled DNP3 analyzer component as C-23 (canonical C-24; C-23 is the
PLANNED ARP analyzer) + prd.md §2.15 cited "C-26" (phantom component). Root cause: the
ARP-cycle component renumbering (ARP→C-23, DNP3→C-24) was never propagated to ss-15 BCs or PRD.
Fixed: all 24 ss-15 BCs (C-23→C-24) + prd C-26→C-24 (prd v1.19→v1.20). A-01 LOW (architect,
self-induced from P23 A-05): arp-architecture-delta §7 changelog rows out of ascending order
(1.11 before 1.10) — reordered (no version bump, cosmetic only). D-02 MED (PO, self-induced
from P23 commit): spec-changelog Pass-23 ledger row had phantom path
specs/architecture/module-criticality.md → corrected to specs/module-criticality.md. D-03 MED
(PO, self-induced from P23 commit): spec-changelog Pass-23 ledger row had phantom path
phase-f2-spec-evolution/arp-architecture-delta.md → corrected to
specs/architecture/arp-architecture-delta.md. KEY mitigation: A-01 reorder took NO version
bump (eliminates one class of self-induced churn); D-02/D-03 changelog paths verified-to-resolve
before writing (prevents phantom-path class). 7th consecutive 0-CRIT. Counter 0/3.
Trajectory P22-P24: 0C/0H → 0C/0H → 0C/1H. Next = whole-corpus Pass 25 via Claude adversary.

Pass 25 REMEDIATED (2 findings; Slices A/B/C CLEAN; 0C/0H). 8th consecutive 0-CRIT pass.
Only Slice D found issues: D-01 MED (PO): spec-changelog "File" column for VP-023 row cited
truncated slug vp-023.md → corrected to vp-023-dnp3-parse-safety.md (pre-existing historical
phantom path). D-02 MED (PO): spec-changelog "File" column for VP-022 row cited truncated slug
vp-022.md → corrected to vp-022-modbus-parse-safety.md (pre-existing historical phantom path).
REMEDIATION: comprehensive changelog-path-phantom flush — scanned ALL .factory/*.md paths
referenced in spec-changelog.md; found 4 distinct non-resolving paths; fixed the 2 active
"File"-column references (VP-022/VP-023 truncated slugs); the other 2 (arp-architecture-delta,
module-criticality) remain only in "corrected-from" audit prose (correctly preserved as audit
trail, not active ledger entries). Zero active ledger references now point at non-resolving
paths. Changelog-path debt class FLUSHED. KEY: Slices A, B, C ALL CLEAN — 3 of 4 slices
clean for 2nd consecutive pass. Counter 0/3 (remediation does not advance counter).
Trajectory P23-P25: 0C/0H → 0C/1H → 0C/0H. Next = whole-corpus Pass 26 via Claude adversary
(strong first-clean candidate; A/B/C clean in P25; changelog-path class flushed).

Pass 26 CLEAN — **FIRST FULLY-CLEAN WHOLE-CORPUS PASS. Convergence streak STARTED: 1/3.**
All 4 slices (A/B/C/D) returned ZERO findings of ANY severity. Slice B confirmed factory HEAD
b008b178 via git guard. This is the first pass across 26 total where every slice was clean
simultaneously, reflecting the corpus-wide debt flush completed across P14-P25 (field-rename,
O-01, MITRE counts, anchor-drift PG-ARP-F2-007, version-pins, changelog-paths, DNP3 component
IDs). No remediation performed — clean pass advances counter.
Counter **1/3** after Pass 26. Need 2 more consecutive all-4-slice-clean passes for convergence gate.
Trajectory P24-P26: 0C/1H → 0C/0H → 0 (CLEAN).

Pass 27 NOT_CLEAN→REMEDIATED — **STREAK BROKEN; counter reset 1/3→0/3.**
Slices A+B CLEAN. C-01 MED (HS-008 kill-chain order; C2 appeared after Exfiltration — wrong;
corrected to canonical all_tactics_in_report_order: Collection→C2→Exfiltration→Impact→[3 ICS]).
D-01 MED (HS-INDEX:~489 BC-2.02.009 "v1.5" stale version-pin; dropped for robustness; swept
holdout layer — 1 active pin found and flushed). Both genuine items; fresh-context variance.
Mitigation applied: holdout BC-version-pin lag class hardened (PG-ARP-F2-008).
Counter **0/3** after Pass 27 reset. Next = whole-corpus Pass 28 via Claude adversary.

Pass 28 CLEAN — **SECOND FULLY-CLEAN WHOLE-CORPUS PASS. Convergence streak RESTARTED: 1/3.**
All 4 slices (A/B/C/D) returned ZERO findings of ANY severity. Reviewed the post-P27 corpus
(holdout kill-chain order corrected + HS-INDEX BC-2.02.009 version-pin dropped). No remediation
performed — clean pass advances counter. Note: Slice D misreported factory HEAD as d734664 under
its read-only profile; actual HEAD is d0a392f; all four slices reviewed the current corpus
content and returned zero findings — verdict CLEAN stands.
Counter **1/3** after Pass 28. Need 2 more consecutive all-4-slice-clean passes for convergence gate.
Trajectory P26-P28: 0 (CLEAN 1/3) → 2 (NOT_CLEAN→REMEDIATED) → 0 (CLEAN 1/3 restart).

Pass 29 NOT_CLEAN→REMEDIATED — **STREAK BROKEN; counter reset 1/3→0/3.**
Slices B+C CLEAN. A-01 MED (architect): DNP3 emitted technique-set omitted T1692.001 in 4
module-inventory docs — module-decomposition.md, system-overview.md, purity-boundary-map.md,
and module-criticality.md each listed only 4 DNP3 techniques, missing T1692.001
(T0855→T1692.001 MITRE v19 remap, propagated to ADR-007 but not all inventory docs).
Canonical 5-ID emitted set {T1692.001, T1691.001, T0814, T0836, T0827} per dnp3.rs and ADR-007;
all 4 docs corrected. D-01 MED (PO): PRD BC-2.14.014 write-set enumeration omitted FC 0x17
in 4 locations within PRD §2.14 / BC-2.14.014 prose; canonical Modbus write-set is
{0x06, 0x10, 0x16, 0x17} (FC 0x16 = Mask Write Register; FC 0x17 = Read/Write Multiple
Registers); all 4 locations corrected. D-02 LOW (PO): PRD version-history changelog anchor
slug at PRD:261 used [pass-13-2026-06-13] which does not match the actual heading slug
[pass-13-corpus-cleanup-2026-06-13]; corrected. Additionally, PRD v1.20 delta row (DNP3 C-23→C-24
and PRD C-26→C-24 fix from P24 D-01) was missing from the version-history table; backfilled.
Both A-01 and D-01 are GENUINE content defects (incomplete technique set / dropped FC), not
cosmetics — fresh-context variance surfaced real gaps that P26 and P28 overlooked. Pattern:
alternating clean(P26/P28)/found(P27/P29); each fix improves corpus correctness.
Counter **0/3** after Pass 29 reset. Next = whole-corpus Pass 30 via Claude adversary.
Trajectory P27-P29: 2 (NOT_CLEAN→REMEDIATED) → 0 (CLEAN 1/3 restart) → 3 (NOT_CLEAN→REMEDIATED).

Pass 30 NOT_CLEAN→REMEDIATED — **Slice D CLEAN; Slices A/B/C found 5 genuine defects (4 HIGH + 1 MED). Counter stays 0/3.**
Slice D returned ZERO findings. Slices A, B, C surfaced 4 genuine HIGH + 1 genuine MED defects that
16 prior passes had missed — concrete proof the whole-corpus grind has real ongoing value.

B-01 HIGH (PO): BC-2.14.018 postcondition and EC mandated `flow_key.client_ip()` / `flow_key.server_ip()` —
non-existent FlowKey accessors. src/analyzer/modbus.rs:375 explicitly comments these BCs "cite a
non-existent API"; an implementer would write non-compiling code. Root cause: the v2.2 F5 sibling
fix that corrected BC-2.14.013/014/015/017/019 to direction-resolved endpoint form was never
propagated to BC-2.14.018. Fixed: BC-2.14.018 postconditions and EC rewritten to direction-resolved
form using flow_key.lower_ip()/upper_ip() + direction. BC-2.14.018 v1.2→v1.3.

B-02 HIGH (PO): BC-2.14.020 postcondition mandated `flow_key.client_ip()` / `flow_key.server_ip()` —
same non-existent FlowKey accessor class as B-01. Same root cause (v2.2 F5 sibling fix missed
BC-2.14.020 along with BC-2.14.018). Fixed: BC-2.14.020 postconditions rewritten to direction-resolved
form using flow_key.lower_ip()/upper_ip() + direction. BC-2.14.020 v2.2→v2.3.

B-03 HIGH (PO): BC-INDEX entries for BC-2.14.018 and BC-2.14.020 carried stale title/version
annotations reflecting the pre-fix postcondition text. Fixed: BC-INDEX updated to reflect v1.3
and v2.3 respectively.

C-01 HIGH (PO): STORY-100..STORY-105 (6 files) each had DUPLICATE `input-hash:` YAML keys —
a TBD placeholder from story generation was never removed when the real hash was stamped in,
leaving two `input-hash:` keys in the YAML frontmatter (invalid YAML per spec; first key would
shadow second in strict parsers). Removed the TBD placeholder key from all 6 files; re-stamped
via `bin/compute-input-hash --write` (all 6 now report MATCH). STORY-104 hash legitimately changed
(old hash was computed against pre-B-01/B-02 BCs; new hash e5c9d7e reflects the corrected
BC-2.14.018 v1.3 input).

A-01 MED (architect): ADR-006 attribution table at lines 112 and 125 mis-bucketed FC 0x17 —
placed it in the T1692.001-only bucket when the canonical/shipped modbus.rs:519 implementation
has 0x17 in the register-write set {0x06, 0x10, 0x16, 0x17} mapping to T0836 (not T1692.001).
Fixed: ADR-006 attribution table lines 112 and 125 corrected to place FC 0x17 in the T0836
register-write bucket.

Counter **0/3** after Pass 30 — 4 HIGH genuine defects found; counter does not advance on NOT_CLEAN pass.
Process note (PG-ARP-F2-009): the F5 FlowKey-accessor fix (v2.2) swept 5 of 7 ss-14 direction-resolution
BCs but missed 018/020; sibling-sweep completeness for code-vs-spec API fixes must enumerate ALL
sibling BCs in the same subsystem before closing a fix burst. STORY input-hash dup-key (TBD
placeholder + real hash both present) is a new frontmatter-validity defect class.
Trajectory P28-P30: 0 (CLEAN 1/3 restart) → 3 (NOT_CLEAN→REMEDIATED) → 5 (NOT_CLEAN→REMEDIATED; 4H genuine).

Pass 31 CLEAN — **THIRD FULLY-CLEAN WHOLE-CORPUS PASS. Convergence streak RESTARTED: 1/3.**
All 4 slices (A/B/C/D) returned ZERO findings of ANY severity. Confirms the P30 HIGH fixes
held: FlowKey accessor correction (BC-2.14.018 v1.3 + BC-2.14.020 v2.3), STORY-100..105
input-hash dup-key removal (all 6 MATCH), and ADR-006 FC-0x17 attribution correction — all
verified clean by fresh-context review. Slice B noted a cosmetic trailing-pipe at BC-INDEX:358
but explicitly ruled it non-blocking/not-a-finding — renders identically, no semantic drift;
tracked as a watch-item only (fix only if a future pass flags it; no severity bump).
No remediation performed — clean pass advances counter.
Counter **1/3** after Pass 31. Need 2 more consecutive all-4-slice-clean passes for convergence gate.
Trajectory P29-P31: 3 (NOT_CLEAN→REMEDIATED) → 5 (NOT_CLEAN→REMEDIATED; 4H genuine) → 0 (CLEAN 1/3 restart).

Pass 32 CLEAN — **FOURTH FULLY-CLEAN WHOLE-CORPUS PASS. Convergence streak ADVANCES: 2/3.**
All 4 slices (A/B/C/D) returned ZERO findings of ANY severity. Second consecutive clean pass
post-P30 remediation. BC-INDEX:358 trailing-pipe correctly treated non-blocking by all slices
(consistent with Pass 31 watch-item ruling; no severity assignment, no remediation triggered).
No remediation performed — clean pass advances counter.
Counter **2/3** after Pass 32. Pass 33 is the convergence-decider.
Trajectory P30-P32: 5 (NOT_CLEAN→REMEDIATED; 4H genuine) → 0 (CLEAN 1/3 restart) → 0 (CLEAN 2/3; 2nd consecutive).

Pass 33 — 2026-06-13 (Claude adversary; CLEAN — 3/3 CONVERGED)
**FIFTH FULLY-CLEAN WHOLE-CORPUS PASS. Convergence streak COMPLETED: 3/3. F2 STRICT-WHOLE-CORPUS CONVERGENCE ACHIEVED.**
All 4 slices (A/B/C/D) returned ZERO findings of ANY severity. Third consecutive clean pass
(P31/P32/P33). Slice D noted one non-blocking observation: PRD v1.20 delta:285 "C-23 was
MbapFramer" — factually-wrong historical rationale. No MbapFramer component ever existed;
ss-15/DNP3 was renumbered C-23→C-24 when ARP took C-23. This falls within the
corrected-from-prose non-blocking exemption; verdict CLEAN. Tracked as
DRIFT-PRD-V120-MBAPFRAMER-001 (cosmetic; LOW; deferred to F3 cycle or maintenance).
No remediation performed — clean pass satisfies the 3/3 gate.
Counter **3/3 CONVERGED** after Pass 33. **F2 STRICT-WHOLE-CORPUS ADVERSARIAL GATE SATISFIED.**

**Summary of the F2 adversarial journey (33 passes, 2026-06-12 → 2026-06-13):**
ARP F2 delta converged ~P9; corpus-wide debt flushed P14-P25 (field-rename, O-01, MITRE
counts, corpus-wide src-anchor drift, version-pins, changelog-paths, component-IDs);
P26/P28/P31/P32/P33 CLEAN; P27/P29/P30 were reset cycles that surfaced and fixed genuine
defects (including 4 HIGH at P30: FlowKey non-existent-accessor, STORY input-hash dup-keys,
ADR-006 FC-0x17). Total: 33 passes; 5 fully-clean passes (P26/P28/P31/P32/P33); gate satisfied
on 3rd consecutive trio P31/P32/P33.

**PG-ARP-F2-003..009 codification follow-up:** All 7 process-gap findings are logged in
STATE.md Drift Items with follow-up stories or justified deferrals. Detailed policy
codification deferred to F7/cycle-close (not blocking F2→F3).

**Next step:** consistency-validator final full-corpus audit → F2→F3 human gate →
F3 story decomposition (STORY-111..115 per arp-architecture-delta §6).
Trajectory P31-P33: 0 (CLEAN 1/3 restart) → 0 (CLEAN 2/3) → 0 (CLEAN 3/3 CONVERGED ★).

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
| tooling-selection | v1.3 | VP-021 added to proptest list (Pass-16 A-01) |
| dependency-graph | v1.5 | Hardcoded "282 tests" generalized to verification-coverage-matrix pointer (Pass-16 A-04) |
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

**Post-Pass-16 (partial — only changed artifacts):**

| Artifact | Version | Change |
|----------|---------|--------|
| tooling-selection | v1.3 | VP-021 added to proptest list (A-01) |
| system-overview | v1.3 | decode_packet diagram (:44,:98) PLANNED→DecodedFrame/STORY-111 markers (A-02) |
| api-surface | v1.4 | decode_packet PLANNED anchor corrected STORY-114→STORY-111 (A-03) |
| dependency-graph | v1.5 | Hardcoded "282 tests" generalized to verification-coverage-matrix pointer (A-04) |
| evaluations/chunk3-reeval.md | — | Dated ERRATUM added: mitre=null frozen run-record; P15 H3 sweep missed sibling (C-01) |
| ADR-005 | modified[] | Rationale :74 "2..=253"→"2..=254" corrected (D-01; P14 D-OBS-01 fixed :108, missed :74) |
| spec-changelog | updated | All P16 version bumps recorded |

---

### Pass 16 — 2026-06-13 (whole-corpus, Claude adversary; NOT_CLEAN → REMEDIATED)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
**Note on method:** agy (Gemini CLI) was ATTEMPTED for Pass 16 via 3 invocation modes: (1) broad
file-tool slices ended on a tool-call with no synthesis output; (2) --conversation resume stalled
silently; (3) tool-free content-paste hung >14 minutes ignoring --print-timeout. No synthesized
output produced across all 3 modes. agy set aside for adversary use pending a working headless
invocation. Pass 16 run via Claude vsdd-factory:adversary per human direction.
**Findings:** 7 total — 0 CRITICAL, 0 HIGH, 5 MEDIUM, 2 LOW — 6 REMEDIATED, 1 DISCARDED.
**Novelty:** LOW — all localized partial-fix sibling-sweep misses; no new debt layers.
**Convergence counter:** 0/3 (remediation does NOT advance counter; next = Pass 17 via Claude).
**Verdict:** NOT_CLEAN → REMEDIATED.

#### Findings and Remediation

**A-01 MED (architect) — tooling-selection.md proptest list VP-021 missing:**
Pass-14/15 remediation added VP-021 to the verification corpus but the proptest tool list in
tooling-selection.md still enumerated 6 proptest VPs (omitting VP-021). List updated to 7.
tooling-selection bumped v1.2→v1.3.

**A-02 MED (architect) — system-overview.md decode_packet diagrams stale (:44,:98):**
Two sequence/data-flow diagrams still showed `decode_packet` returning `Result<ParsedPacket>`
(pre-ADR-008); both needed PLANNED→DecodedFrame/STORY-111 markers matching arp-architecture-delta
convention. Added PLANNED annotation with STORY-111 scope reference.
system-overview bumped v1.2→v1.3.

**A-03 MED (architect) — api-surface.md decode_packet PLANNED anchor STORY-114→STORY-111:**
Pass-14 A-06 introduced the PLANNED marker for decode_packet's `Result<DecodedFrame>` return type
but cited STORY-114 (the wrong story; that is the ARP implementation story). Canonical story for
etherparse DecodedFrame integration at the decoder layer is STORY-111. Corrected STORY-114→STORY-111
(verified vs arp-architecture-delta §6 which cites STORY-111 for DecodedFrame at decoder boundary).
api-surface bumped v1.3→v1.4.

**A-04 LOW (architect) — dependency-graph.md hardcoded "282 tests" stale:**
A hardcoded "282 tests" count in dependency-graph.md was stale (current total 283 + ongoing
Pass-16-era additions). Generalized to a prose pointer to verification-coverage-matrix (living
document) rather than a hardcoded count that will drift with each test addition.
dependency-graph bumped v1.4→v1.5.

**A-05 LOW — DISCARDED (ADR-008 status `proposed` correct/NON-BLOCKING):**
Adversary flagged ADR-008 status as `proposed` rather than `accepted`. This is correct by
convention: ADR-008 is forward-declared for F4 implementation (STORY-111); `proposed` is the
right state until implementation lands. Adversary itself noted the consistency — non-blocking
by the NON-BLOCKING forward-declaration policy. Discarded.

**C-01 MED (PO) — evaluations/chunk3-reeval.md `mitre=null` (H3 sweep miss):**
Pass-15 H3 sweep added ERRATA to chunk1-eval.md and chunk3-eval.md (two frozen run records)
but missed the sibling `chunk3-reeval.md`. The reeval record carried `mitre=null` from the
pre-STORY-100 single-tag era. File is a frozen audit-trail record; dated ERRATUM appended
(2026-06-13) recording the field-rename without rewriting history. Same H3 pattern applied.

**D-01 MED (architect) — ADR-005.md:74 Rationale "2..=253"→"2..=254":**
Pass-14 D-OBS-01 corrected the `[2,254]` length range at ADR-005:108. The Rationale section
at :74 carried a second instance — "2..=253" — which was missed in the same burst
(partial-fix sibling sweep miss). Corrected to "2..=254" (verified vs src/analyzer/modbus.rs
which validates `len >= 2 && len <= 254`). Modified[] entry appended to ADR-005 frontmatter.

#### Consistency verification (targeted grep)

Post-remediation targeted grep sweep PASS:
- No current-state `[2,253]` or `2..=253` in ADR-005 (both :74 and :108 now correct).
- api-surface decode_packet PLANNED anchor cites STORY-111 (not STORY-114).
- tooling-selection proptest list = 7 VPs.
- chunk3-reeval.md ERRATUM present (2026-06-13 dated).
- No duplicate YAML version-keys in modified files.
- dependency-graph "282 tests" hardcode removed.

Full fresh-context re-check deferred to Pass 17 (edits were tiny/non-structural).

#### Process gap noted [process-gap]

**PG-ARP-F2-005:** erratum/sweep glob patterns must cover sibling naming variants
(chunk*-eval.md glob matched chunk1-eval.md and chunk3-eval.md but missed chunk3-reeval.md).
And partial-fix discipline: when a fix touches one of N siblings (e.g., one of two `2..=253`
instances in ADR-005; one of three decode_packet peers), sweep ALL siblings in the same burst
before committing.

---

### Pass 17 — 2026-06-13 (whole-corpus, Claude adversary; NOT_CLEAN → REMEDIATED)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
**Factory-artifacts HEAD reviewed:** (post-P16-remediation burst).
**Findings:** 10 total — 3 CRITICAL, 2 HIGH, 2 MEDIUM, 3 LOW — ALL REMEDIATED.
**Novelty:** MED — two not-previously-deep-reviewed corpus corners: (1) module-decomposition.md
— the ONE architecture peer never swept for decode_packet/ARP PLANNED markers (asserted
etherparse-0.20/DecodedFrame migration as shipped when current src=ParsedPacket/0.16); (2) a
new holdout sub-layer — MITRE-catalog holdouts HS-008/009/025 carried greenfield-era counts
(16 tactics/15 seeded/5 emitted/9 cat-only) contradicting their updated anchor BCs.
**Convergence counter:** 0/3 (remediation does NOT advance counter; next = Pass 18 via Claude).
**Verdict:** NOT_CLEAN → REMEDIATED.
**Trajectory note:** NON-MONOTONIC — P14 2C/5H → P15 2C/1H → P16 0C/0H → P17 3C/2H. Slice B
CLEAN 2nd consecutive. Ground truth (src/mitre.rs verified): 23 seeded / 15 emitted / 8
cat-only / 17 MitreTactic variants (14E+3 ICS incl IcsImpact) — current shipped values.

#### Findings and Remediation

**A-01 HIGH (architect) — module-decomposition C-5 row + modified-log asserted
decode_packet→DecodedFrame/etherparse-0.20 as shipped:**
module-decomposition.md C-5 row described etherparse-0.20 and DecodedFrame migration as current
shipped state. Current src=ParsedPacket/etherparse-0.16 (migration is STORY-111, F4 obligation).
PLANNED→STORY-111 markers added to C-5 row and to any prose asserting DecodedFrame migration as
complete. module-decomposition bumped v1.6→v1.7.

**A-02 HIGH (architect) — module-decomposition C-23 ArpAnalyzer + preamble asserted as
shipped:**
C-23 ArpAnalyzer entry and related preamble described arp.rs as present in source tree.
src/analyzer/arp.rs does NOT exist in develop HEAD (ARP implementation is STORY-112/F4,
ADR-008). PLANNED→STORY-112/ADR-008 markers added. module-decomposition bumped (same file,
same version v1.6→v1.7).

**A-03 LOW (architect) — module-decomposition peer cross-refs resolved by A-01:**
Peer references in module-decomposition that cross-cited now-PLANNED rows were covered by
the same PLANNED marker sweep in A-01/A-02. Resolved as part of A-01/A-02 fix.

**A-04 LOW (architect) — module-decomposition modified-log entries:**
Modified-log entries asserting etherparse-0.20/ArpAnalyzer changes as complete were annotated
with PLANNED scope references. Resolved as part of A-01/A-02 sweep.

**C-01 CRITICAL (PO) — HS-025 "16 entries (14E+2 ICS)"→17 (14E+3 ICS incl IcsImpact):**
HS-025 holdout scenario (ICS tactic display and non-exhaustive guard) carried greenfield-era
count "16 entries (14E+2 ICS)". STORY-109 (DNP3 F4) shipped MitreTactic::IcsImpact as the 3rd
ICS variant; canonical count is 17 (14 Enterprise + 3 ICS). 7 count-bearing assertions updated.
HS-025 bumped v1.0→v1.1.

**C-02 CRITICAL (PO) — HS-008 "(16 total)"→17, "15 seeded"→23:**
HS-008 holdout scenario (MITRE tactic display and kill-chain order) carried "(16 total)" tactic
count and "15 seeded" IDs (greenfield-era). Canonical: 17 MitreTactic variants; 23 seeded IDs
(src/mitre.rs verified). Updated. HS-008 bumped v1.0→v1.1.

**C-03 CRITICAL (PO) — HS-009 "15 seeded"→23, "5 emitted"→15 (enumerated EMITTED_IDS),
"9 cat-only"→8:**
HS-009 holdout scenario (MITRE technique lookup — unknown IDs) carried "15 seeded" / "5
emitted" / "9 cat-only" from the greenfield era. Canonical (src/mitre.rs): 23 seeded / 15
emitted / 8 cat-only. Emitted IDs enumerated inline (current EMITTED_IDS list confirmed). HS-009
bumped v1.1→v1.2.

**C-04 MED (PO) — HS-009 T0886 phantom ICS ID corrected:**
HS-009 used T0886 as an example unknown ICS technique ID. T0886 is not in the MITRE ICS
ATT&CK catalog (verified); the correct adjacent ID is T0885 (Theft of Operational Information).
Prose corrected: T0886 replaced with a note that the real unknown-ID path uses T9999 (reserved
for testing) and cites T0885 as the nearest-neighbor real ID. HS-009 bumped (same file,
v1.1→v1.2).

**D-01 LOW (PO) — nfr-catalog NFR-OBS-010 "all four fields" disambiguation:**
nfr-catalog NFR-OBS-010 carried "all four fields" phrasing (parallel to Pass-15 C-05
interface-definitions fix). Disambiguated to: mitre_techniques Vec + 3 Option fields
(timestamp, src_ip, dst_ip). nfr-catalog bumped v1.8→v1.9.

**D-02 LOW (PO) — domain-spec §Summary-Metrics "21 components" erratum:**
domain-spec §Summary-Metrics cited "21 components". This is a FROZEN pre-F2 ingestion baseline
(develop@0082a0c; 21 C-numbered components, 24 source files). The adversary misread the
24-source-files count as 24-components and flagged the row. The row is a frozen snapshot, NOT
a live count — it correctly describes the brownfield ingestion baseline. Dated erratum/disclaimer
added pointing to ARCH-INDEX for the current 24-component count; the rows themselves NOT
rewritten (frozen snapshot integrity preserved). NOT a count regression.

#### Artifact versions post-Pass-17

| Artifact | Version | Change |
|----------|---------|--------|
| module-decomposition | v1.7 | C-5/C-23 PLANNED→STORY-111/STORY-112/ADR-008 markers; modified-log annotated |
| HS-025 | v1.1 | 16→17 tactic count (14E+3 ICS incl IcsImpact); 7 count assertions updated |
| HS-008 | v1.1 | "(16 total)"→17; "15 seeded"→23 |
| HS-009 | v1.2 | "15 seeded"→23; "5 emitted"→15 (EMITTED_IDS enumerated); "9 cat-only"→8; T0886 phantom ID corrected |
| nfr-catalog | v1.9 | NFR-OBS-010 "all four fields" disambiguated |
| domain-spec | modified | §Summary-Metrics erratum/disclaimer added (frozen baseline preserved) |
| spec-changelog | updated | All P17 version bumps recorded |

#### Process gap noted [process-gap]

**PG-ARP-F2-006:** Holdout-scenarios carry count assertions (tactic/seeded/emitted/cat-only)
that drift when feature cycles change the MITRE catalog. HS-008/009/025 carried greenfield-era
counts not updated across DNP3 cycle (STORY-109 added IcsImpact; STORY-109 expanded seeded
IDs). F-cycle close-out must sweep holdout count-assertions in addition to field-names. Candidate
policy: extend DF-CANONICAL-FRAME-HOLDOUT-001 / holdout-maintenance policy to require a
count-assertion sweep whenever src/mitre.rs seeded/emitted/catalog size changes.

---

### Pass 18 — 2026-06-13 (whole-corpus, Claude adversary; NOT_CLEAN → REMEDIATED)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
**Factory-artifacts HEAD reviewed:** (post-P17-remediation burst).
**Findings:** 9 total — 0 CRITICAL, 3 HIGH, 2 MEDIUM, 4 LOW — ALL REMEDIATED.
**Novelty:** LOW — ss-05 dispatcher anchor-drift class is the largest cluster (systematic:
+94..+235 line shift across all 9 ss-05 BCs from Modbus Rule-5/DNP3 Rule-6 + accessor
insertions, last sync v1.3 pre-ICS); indicatif 0.17→0.18 version self-invariant violation; STORY-INDEX
48-vs-49 prose ambiguity. Proactive pre-pass fix (arp.rs/C-23 PLANNED markers STORY-111→STORY-112)
verified CLEAN by Slice A. Entire holdout tree 101 files (Slice C) verified CLEAN.
**Convergence counter:** 0/3 (remediation does NOT advance counter; next = Pass 19 via Claude).
**Verdict:** NOT_CLEAN → REMEDIATED.

#### Pre-pass proactive fix (committed in this burst)

**arp.rs/C-23 PLANNED marker correction (architect, pre-pass):**
system-overview.md C-23 PLANNED note cited STORY-111 for ArpAnalyzer. Correct story is
STORY-112 (arp.rs created in STORY-112; STORY-111 = decode_packet/DecodedFrame only per
arp-architecture-delta §6). system-overview bumped v1.3→v1.4. purity-boundary-map C-23 PLANNED
note also cited STORY-111→corrected to STORY-112; purity-boundary-map bumped v1.2→v1.3
(continuing chain from P17 v1.1→v1.2; full session chain v1.1→v1.2→v1.3→v1.4 for
purity-boundary-map across P14/P18 architect work). Verified consistent across all architecture
peers by Pass-18 Slice A.

#### Findings and Remediation

**A-01 HIGH (architect) — dependency-graph indicatif 0.17 self-invariant violation:**
dependency-graph.md listed indicatif as "0.17" in the crate version table. Cargo.toml
(develop HEAD 31d1231, post-PR #206) specifies indicatif = "0.18". Self-invariant: the
dependency-graph document asserts it lists "current Cargo.toml versions" — listing a stale
version violates this invariant. All 14 crate rows verified against Cargo.toml; indicatif row
corrected 0.17→0.18. dependency-graph bumped v1.5→v1.6.

**A-02 MED (architect) — verification-coverage-matrix VP-023 lock-evidence note missing:**
verification-coverage-matrix listed VP-023 without the lock-evidence annotation that was added
at commit @e685664 (Feature #8 F6 hardening, PR #231). The lock-evidence note records that
VP-023 was LOCKED (Kani+fuzz 9/9 + 3.19M/0) by the DNP3 F6 burst. Added lock-evidence bullet
to VP-023 row. verification-coverage-matrix bumped v1.3→v1.4.

**A-03 LOW (architect) — purity-boundary-map VP-024 arp.rs verification bullet absent:**
purity-boundary-map v1.3 (post-pre-pass bump) listed C-23 arp.rs entry but did not include
the VP-024 verification bullet noting arp.rs as the primary subject of VP-024
(spoof/GARP/storm/rate anomaly proofs). Bullet added. purity-boundary-map bumped v1.3→v1.4
(full session chain: P14 v1.1→v1.2 [field-rename/C-23+C-24]; pre-pass fix v1.2→v1.3
[STORY-111→STORY-112]; A-03 v1.3→v1.4 [VP-024 bullet]).

**B-01/B-02 HIGH (PO) — ss-05 SYSTEMATIC stale src/dispatcher.rs anchors (all 9 BCs):**
All 9 ss-05 BCs (BC-2.05.001 through BC-2.05.009) carried stale dispatcher.rs line anchors
from the pre-ICS era (v1.3 last sync). Modbus Rule-5 (STORY-102..105) and DNP3 Rule-6
(STORY-106..110) inserted code into src/dispatcher.rs, shifting line offsets by +94..+235
depending on location. Additionally, accessor method insertions (on_data, on_flow_close,
classify) shifted anchor targets. All 9 BCs re-anchored against current src/dispatcher.rs
using ground-truth line map:
- fn classify: :184
- on_data: :245
- cache block: :269-289
- on_flow_close: :322-361
- DEFAULT_MAX: :58
- 4-analyzer guard: :256-259

Each of the 9 BCs bumped by one version (BC-2.05.001 through BC-2.05.009). BC-INDEX
annotations for ss-05 updated with new version pins.

**B-03 LOW (PO) — BC-2.05.007/008 unconfigured-guard prose widened to 4-analyzer:**
BC-2.05.007 and BC-2.05.008 described the "unconfigured analyzer" guard as covering "http and
tls analyzers". Current dispatcher.rs:256-259 guards http/tls/modbus/dnp3 (4 analyzers since
Modbus+DNP3 integration). Prose widened to enumerate all 4. BCs bumped (same version bump as
B-01/B-02).

**C-01/D-01 MED/LOW (PO) — STORY-INDEX line 20 "49 stories" ambiguity:**
STORY-INDEX line 20 stated "(49 stories)" without distinguishing the 48 greenfield-product
stories from STORY-091 (tooling story, compute-input-hash). Corrected to
"(48 greenfield product + 1 tooling STORY-091 = 49 stories)" to resolve the 48-vs-49 apparent
discrepancy seen in cross-doc counts. STORY-INDEX version annotated.

**C-02 LOW (PO) — cap-10 changelog self-referential line anchors stale:**
cap-10 changelog (spec-changelog.md entries for cap-10 v1.8) cited self-referential line
anchors 81/83-85 that shifted to 87/89-91 after the Pass-14 burst that bumped cap-10 v1.7→v1.8.
Line anchors corrected in changelog entry. cap-10 spec-changelog bump recorded; cap-10 itself
not re-versioned (changelog-only correction).

**CARRY-OVER HIGH (PO, found during ss-05 sweep) — BC-2.04.055 Architecture Anchor
dispatcher.rs:144→:245:**
During the ss-05 anchor re-sync sweep, BC-2.04.055 was discovered to carry a stale
`dispatcher.rs:144` Architecture Anchor for `on_data`. Current ground truth: `on_data` is
at dispatcher.rs:245 (shifted by Feature #7 Modbus + Feature #8 DNP3 insertions). BC-2.04.055
bumped v1.0.1→v1.0.2. This is the same line-shift class as the ss-05 BCs (PG-ARP-F2-007).

#### Ground truth (verified, 2026-06-13)

src/dispatcher.rs line map (develop HEAD 31d1231):
- `fn classify` → :184
- `fn on_data` → :245
- cache block (binding entry + analyzer dispatch) → :269-289
- `fn on_flow_close` → :322-361
- `DEFAULT_MAX_STORED_HEADERS` → :58
- 4-analyzer unconfigured guard → :256-259

#### Artifact versions post-Pass-18

| Artifact | Version | Change |
|----------|---------|--------|
| system-overview | v1.4 | C-23 PLANNED STORY-111→STORY-112 (pre-pass fix) |
| purity-boundary-map | v1.4 | C-23 STORY-111→STORY-112 (pre-pass v1.2→v1.3) + VP-024 arp.rs bullet (A-03 v1.3→v1.4) |
| dependency-graph | v1.6 | indicatif 0.17→0.18 (A-01) |
| verification-coverage-matrix | v1.4 | VP-023 lock-evidence note (A-02) |
| BC-2.05.001 | bumped | ss-05 dispatcher anchor re-sync (B-01/B-02) |
| BC-2.05.002 | bumped | ss-05 dispatcher anchor re-sync (B-01/B-02) |
| BC-2.05.003 | bumped | ss-05 dispatcher anchor re-sync (B-01/B-02) |
| BC-2.05.004 | bumped | ss-05 dispatcher anchor re-sync (B-01/B-02) |
| BC-2.05.005 | bumped | ss-05 dispatcher anchor re-sync (B-01/B-02) |
| BC-2.05.006 | bumped | ss-05 dispatcher anchor re-sync (B-01/B-02) |
| BC-2.05.007 | bumped | ss-05 dispatcher anchor re-sync + 4-analyzer guard prose (B-01/B-02/B-03) |
| BC-2.05.008 | bumped | ss-05 dispatcher anchor re-sync + 4-analyzer guard prose (B-01/B-02/B-03) |
| BC-2.05.009 | bumped | ss-05 dispatcher anchor re-sync (B-01/B-02) |
| BC-2.04.055 | v1.0.2 | on_data anchor :144→:245 (CARRY-OVER) |
| BC-INDEX | updated | ss-05 version pins + BC-2.04.055 annotation |
| STORY-INDEX | updated | 48-vs-49 ambiguity resolved (C-01/D-01) |
| cap-10 / spec-changelog | updated | changelog anchor correction (C-02) |
| spec-changelog | updated | All P18 version bumps recorded (incl. pre-pass proactive fix) |

#### Process gap noted [process-gap]

**PG-ARP-F2-007:** src-line-anchor drift class — feature cycles that insert code into a shared
file (dispatcher.rs via Modbus/DNP3) leave EVERY citing BC's anchors stale; F-cycle close-out
must re-run an anchor-resync sweep across ALL BCs citing the touched src files
(dispatcher.rs → ss-04/ss-05; mitre.rs → ss-10 [done]; findings.rs → ss-09;
reassembly/http/tls → ss-04/06/07 [verify next pass]). Candidate: anchor-drift lint or
F-cycle anchor-resync checklist. ANCHOR-DRIFT WATCH: ss-10 (mitre.rs) + ss-05/ss-04-055
(dispatcher.rs) anchors re-synced this pass; ss-09 (findings.rs) + ss-06/ss-07
(http.rs/tls.rs/reassembly, potentially shifted by STORY-097/098/099 timestamp wiring) NOT
yet anchor-audited — likely Pass-19 Slice B targets.

---

### Pass 19 — 2026-06-13 (whole-corpus, Claude adversary; NOT_CLEAN → REMEDIATION IN PROGRESS)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
**Factory-artifacts HEAD reviewed:** (post-P18-remediation burst).
**Findings:** 15 total — 0 CRITICAL, 8 HIGH, 2 MEDIUM, 5 LOW — PARTIAL REMEDIATION (this checkpoint).
**Novelty:** HIGH — PG-ARP-F2-007 confirmed CORPUS-WIDE. P19's directed anchor-drift audit
revealed the drift class extends far beyond ss-05/BC-2.04.055 (fixed at P18): src line-number
shifts from F2 features (Modbus/DNP3→dispatcher.rs; timestamp-wiring STORY-097/098/099→http.rs/
tls.rs/reassembly; multi-tag STORY-100→findings.rs) left stale src-anchors across BCs, VPs,
domain docs, and prd-supplements. Slice D CLEAN.
**Convergence counter:** 0/3 (remediation does NOT advance counter; ss-07-full + remaining-BC
anchor audit PENDING before Pass 20).
**Verdict:** NOT_CLEAN → REMEDIATION IN PROGRESS (partial checkpoint committed).

#### Findings

- **B-01..B-06 (PO, ss-09)** — BC-2.09.001..005/007 (findings.rs anchors stale); BC-2.09.003
  missing Possible-verdict variant (content gap — new finding beyond anchor-drift).
- **B-07 (PO, ss-04/06)** — BC-2.04.055 http.rs/tls.rs on_data partial-fix regression
  (http.rs + tls.rs anchor also stale, introduced as partial-fix at P18).
- **B-08 (PO, ss-06)** — ALL 26 ss-06 http.rs BCs: stale anchors from STORY-097/098/099
  timestamp-wiring shifts.
- **B-09 (PO, ss-04)** — BC-2.04.024/020 mod.rs anchors stale.
- **B-10 (PO, ss-07)** — BC-2.07.037/016/008 off-by-one anchor errors (3 BCs; full tls.rs
  re-anchor across remaining ~34 ss-07 BCs STILL PENDING).
- **A-01 (architect)** — purity-boundary-map sub-letter numbering inconsistency (v1.4→v1.5).
- **A-02 (architect)** — dispatcher.rs ground-truth anchor in purity-boundary-map stale.
- **C-01 (PO)** — HS-009 MITRE technique lookup: T1083→Discovery mapping fact fix.
- **C-02 (PO)** — nfr-catalog/nfr-story-map dispatcher anchors stale.
- **C-03 (PO)** — inv-01 INV-2 dispatcher anchors stale.

#### Remediated this checkpoint (~62 files committed)

**Architect:**
- purity-boundary-map v1.4→v1.5 (A-01/A-02): sub-letter fix + dispatcher ground-truth anchor.

**VP sweep (9 files re-anchored):**
- vp-003 v2.0→v2.1, vp-004 v2.1→v2.2, vp-006/010/011/013/014/015/021 v2.0→v2.1.

**PO — BC subsystems:**
- ss-09: 6 BCs (BC-2.09.001/002/003/004/005/007) re-anchored vs findings.rs; BC-2.09.003
  Possible-verdict variant ADDED (B-06 content gap).
- ss-06: ALL 26 BCs (BC-2.06.001..026) re-anchored vs http.rs current lines.
- ss-04: BC-2.04.055 http.rs/tls.rs on_data + BC-2.04.024/020 mod.rs anchors corrected.
- ss-07 partial: BC-2.07.037/016/008 off-by-one corrected.

**PO — holdout/domain/prd-supp:**
- HS-009: T1083→Discovery MITRE-fact fix (C-01).
- nfr-catalog/nfr-story-map: dispatcher anchors (C-02).
- inv-01 INV-2: dispatcher anchors (C-03).
- Domain straggler sweep: cap-05/06/07/09, ent-03/04, inv-01 INV-2/5/6/8, nfr-catalog
  NFR-RES-011..017, error-taxonomy, test-vectors.

**Ground truth src maps recorded (verified vs develop HEAD 31d1231):**
- src/dispatcher.rs: fn classify :184; on_data :245; cache :269-289; on_flow_close :322-361;
  DEFAULT_MAX :58; 4-analyzer guard :256-259.
- src/http.rs, src/tls.rs, src/findings.rs, src/analyzer/mod.rs, src/segment.rs,
  src/lifecycle.rs, src/flow.rs — line maps captured during this anchor sweep.

#### STILL PENDING (before Pass 20)

1. **ss-07 FULL re-anchor** — ~34 remaining ss-07 BCs cite tls.rs across many regions;
   e.g., BC-2.07.031 tls.rs:771-773 top_snis-sort is stale (actual @860-862). Full tls.rs
   re-anchor sweep required.
2. **Remaining BC subsystem audit** — ss-01/02/04-rest/08/11/12/13: spot-check for shifted-file
   anchors vs current src.

#### Batch 2 (anchor-drift continuation) — 2026-06-13

**ss-07 FULL re-anchor vs current tls.rs (35 BCs changed):**
BC-2.07.001-015/017-029/031-037 re-anchored. BC-2.07.016 and BC-2.07.030 confirmed already
clean (no change needed). Full ss-07 tls.rs re-anchor COMPLETE.

**ss-04 PARTIAL re-anchor vs current reassembly src (21 BCs changed):**
BCs changed: 029/030/032-038/040-048/051/052/054.
BCs confirmed clean (no change): 001/003/004/009/031/039/049/050/053.
BCs done earlier in P19: 020/024/055.
REMAINING / UNVERIFIED ss-04 BCs: 002/005/010/011/013/015/016 + any of
006-008/012/014/017-019/021-023/025-028/056+ not yet covered — defer to Pass-20 precise
per-anchor src verification. Blind heuristic scans produce false positives on top-of-file
symbols; adversary per-anchor src-verification is the reliable method.

**ss-11 re-anchor vs current reporter src (10 BCs changed):**
BC-2.11.009/013/014/015/016/017/018/021/022/024 re-anchored. 14 ss-11 BCs confirmed clean.
(reporter csv.rs/terminal.rs shifted less than the analyzers.)

**Confirmed clean (zero shifted-src citations):**
ss-01/02/08/13 — all anchors verified correct vs current src.

**Deferred to Pass-20 precise audit:**
ss-12 BC-2.12.005 cites shifted src — NOT yet re-anchored.
ss-04 remainder (~12-20 BCs) — precise per-anchor src verification required.

**Lesson recorded:** Blind proactive anchor sweeps risk over-correction + incompleteness
(ss-04 burst came back partial). The reliable method is adversary per-anchor src-verification
followed by targeted fix. PG-ARP-F2-007 strongly motivates a mechanical file:line-anchor
validation tool.

**Anchor-drift status after Batch 2:**
- ss-05 (dispatcher.rs): COMPLETE (P18)
- ss-06 (http.rs): COMPLETE (P19 Batch 1)
- ss-07 (tls.rs): COMPLETE (P19 Batch 2 — all 35 changed BCs re-anchored)
- ss-09 (findings.rs): COMPLETE (P19 Batch 1)
- ss-11 (reporter): COMPLETE (P19 Batch 2)
- ss-01/02/08/13: CONFIRMED CLEAN (P19 Batch 2)
- ss-04 (reassembly): PARTIAL — 21 BCs done Batch 2 + 3 earlier = ~24 total; ~12-20 remain
- ss-12: NOT YET (BC-2.12.005 defer to Pass-20)
- Estimated: ~85% of anchor-drift flushed; expect convergence approach once ss-04-remainder closed.

#### Artifact versions post-Pass-19 checkpoint

| Artifact | Version | Change |
|----------|---------|--------|
| purity-boundary-map | v1.5 | A-01/A-02: sub-letter fix + dispatcher ground-truth anchor |
| vp-003 | v2.1 | anchor re-sync |
| vp-004 | v2.2 | anchor re-sync |
| vp-006 | v2.1 | anchor re-sync |
| vp-010 | v2.1 | anchor re-sync |
| vp-011 | v2.1 | anchor re-sync |
| vp-013 | v2.1 | anchor re-sync |
| vp-014 | v2.1 | anchor re-sync |
| vp-015 | v2.1 | anchor re-sync |
| vp-021 | v2.1 | anchor re-sync |
| BC-2.09.001 | bumped | ss-09 findings.rs anchor re-sync |
| BC-2.09.002 | bumped | ss-09 findings.rs anchor re-sync |
| BC-2.09.003 | bumped | ss-09 anchor re-sync + Possible-verdict variant ADDED |
| BC-2.09.004 | bumped | ss-09 findings.rs anchor re-sync |
| BC-2.09.005 | bumped | ss-09 findings.rs anchor re-sync |
| BC-2.09.007 | bumped | ss-09 findings.rs anchor re-sync |
| BC-2.06.001..026 | bumped | ALL 26 ss-06 http.rs anchors re-synced |
| BC-2.04.055 | bumped | http.rs/tls.rs on_data anchor (B-07) |
| BC-2.04.024 | bumped | mod.rs anchor (B-09) |
| BC-2.04.020 | bumped | mod.rs anchor (B-09) |
| BC-2.07.037 | bumped | off-by-one anchor (B-10 partial) |
| BC-2.07.016 | bumped | off-by-one anchor (B-10 partial) |
| BC-2.07.008 | bumped | off-by-one anchor (B-10 partial) |
| HS-009 | bumped | T1083→Discovery MITRE-fact fix (C-01) |
| nfr-catalog | bumped | dispatcher anchors (C-02) + NFR-RES-011..017 sweep |
| nfr-story-map | bumped | dispatcher anchors (C-02) |
| inv-01 | bumped | INV-2 dispatcher anchors (C-03) |
| cap-05/06/07/09 | bumped | domain straggler sweep |
| ent-03/04 | bumped | domain straggler sweep |
| error-taxonomy | bumped | domain straggler sweep |
| test-vectors | bumped | domain straggler sweep |
| BC-INDEX | updated | version pins for all re-anchored BCs |
| spec-changelog | updated | all P19 checkpoint version bumps recorded |

---

### Pass 20 — 2026-06-13 (whole-corpus, Claude adversary; NOT_CLEAN → REMEDIATED)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
**Factory-artifacts HEAD reviewed:** (post-P19-Batch-2-remediation burst).
**Findings:** 7 total — 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW — ALL REMEDIATED.
**Novelty:** LOW — ss-04 remainder (BC-2.04.012/013/014) and ss-12 (BC-2.12.005) are the last
anchor-drift pockets from PG-ARP-F2-007; cap-09 frontmatter version straggler is a unapplied
bump from the P19 body sweep; ADR-008 T0830 matrix-label adjacent-sentence reconciliation is
prose only (no mapping change). Slices A and C CLEAN.
**Over-correction spot-check:** PASSED — all large P19 sweeps (ss-07-full 35 BCs, ss-04-partial
21 BCs, ss-11 10 BCs, ss-06 26 BCs, VP sweep 9 files) verified correct; no anchor moved to a
wrong line. Anchor-drift class PG-ARP-F2-007 FLUSHED corpus-wide.
**Convergence counter:** 0/3 (remediation does NOT advance counter; next = Pass 21 via Claude
adversary — strong first-clean-pass candidate).
**Verdict:** NOT_CLEAN → REMEDIATED.

#### Findings and Remediation

**D-01 HIGH (PO) — cap-09-finding-emission.md version field stuck at 1.1 (P19 straggler):**
The P19 straggler anchor sweep applied body updates to cap-09 and added a second `modified[]`
entry, bumping the changelog/body to version 1.2, but the frontmatter `version:` field remained
at `"1.1"` (unapplied bump from P19 sweep). Bumped frontmatter `version:` to `"1.2"`.
cap-09 now consistent at v1.2 throughout.

**B-01 LOW (PO) — BC-2.04.012 v1.9→v2.0: Invariant-1 latch prose mod.rs:618→:647:**
Invariant 1 prose cited `self.finalized = true` at `mod.rs:618`. Actual is `mod.rs:647`
(verified: `grep -n "finalized" src/reassembly/mod.rs` returns `647: self.finalized = true`).
The Refactoring Notes already cited 647 correctly; only the Invariant body sentence was missed
in the P19 sweep.

**B-02 MED (PO) — BC-2.04.013 v1.8→v1.9: expire call-site :166-169→:168-171 (2 occurrences):**
Architecture Module row and Source Evidence row both cited `process_packet` call site at
`mod.rs:166-169`. Actual call site (`expire_idle_by_timeout` invocation) is at `mod.rs:168-171`
(verified). Architecture Anchors and prose already had 168-171 correct. Fixed both stale
occurrences.

**B-03 MED (PO) — BC-2.04.014 v1.5→v1.6: lifecycle.rs:60→:66:**
Architecture Module row and Architecture Anchors bullet cited `lifecycle.rs:60` for
`total_memory -= flow_mem on close`. Actual is `lifecycle.rs:66` (verified:
`grep -n "total_memory"` returns `66: self.total_memory -= flow_mem`; line 60 is
`let flushed = flow_dir.flush_contiguous()`).

**B-04 MED (PO) — BC-2.12.005 v1.4→v1.5: main.rs:87-122→:139-166 / Invariant 4 :104-117→:147-161:**
Architecture Anchor `src/main.rs:87-122` (described as "reassembly configuration applied in
run_analyze") was stale. The `ReassemblyConfig` struct literal is at lines 140-144; CLI override
`if let Some(v)` blocks run 147-161; `flow_timeout_secs` wire at 165; `TcpReassembler::new` at
166. Correct range: `main.rs:139-166`. Invariant 4 cited `main.rs:104-117` for CLI override
application; actual override block is `main.rs:147-161`.

**B-05 LOW (PO) — BC-2.12.005 (same version bump): cli.rs:71-122→:73-124:**
Architecture Anchor and Source Evidence cited `cli.rs:71-122` for the reassembly flag block.
Line 71 is the `--csv` flag tail; the `--reassemble` `#[arg]` annotation starts at line 73.
The block ends with `pub flow_timeout: u64` at line 124. Correct range: `cli.rs:73-124`.

**D-02 LOW (architect) — ADR-008 v1.8→v1.9: T0830 ICS/Enterprise matrix-label adjacent-sentence reconciliation:**
Decision 6 MitreTactic enum assessment paragraph contained two adjacent sentences using
inconsistent labels for T0830's tactic source: opening sentence said "ICS matrix (TA0109)"
while the bullet said "Enterprise Lateral Movement variant", creating an apparent contradiction.
No mapping change — T0830 maps to MitreTactic::LateralMovement (shared variant) via
merge-by-name policy, and its home matrix is ICS (TA0109). Both sentence and bullet now
consistently describe this. ADR-008 bumped v1.8→v1.9.

#### Slices verified CLEAN this pass

- **Slice A (architecture/):** All architecture docs CLEAN. Only D-02 was in ADR-008 (LOW
  prose reconciliation); all VP files, api-surface, purity-boundary-map, system-overview,
  module-decomposition, dependency-graph CLEAN.
- **Slice C (domain/ + prd-supplements + holdout-scenarios):** D-01 cap-09 version straggler
  (HIGH — but cosmetic metadata, not behavioral). Domain invariants, capability docs, HS files,
  nfr-catalog, error-taxonomy, test-vectors all otherwise CLEAN.

#### Over-correction spot-check result (PASSED)

Verified that the following P19 bulk sweeps are correct (no anchor moved to wrong line):
- ss-07 FULL 35 BCs (tls.rs): all anchors verified against current develop HEAD.
- ss-04 PARTIAL 21 BCs (reassembly): all changed anchors verified correct.
- ss-11 10 BCs (reporter): all anchors verified correct.
- ss-06 26 BCs (http.rs): all anchors verified correct (COMPLETE P19 Batch 1).
- VP sweep 9 files: all correct.
No over-corrections found.

#### Anchor-drift class (PG-ARP-F2-007) — FLUSHED

After Pass-20 remediation, anchor-drift status across all subsystems:

| Subsystem | Status | Completed |
|-----------|--------|-----------|
| ss-05 (dispatcher.rs) | COMPLETE | P18 |
| ss-06 (http.rs) | COMPLETE | P19 Batch 1 |
| ss-07 (tls.rs) | COMPLETE | P19 Batch 2 (35 BCs) |
| ss-09 (findings.rs) | COMPLETE | P19 Batch 1 |
| ss-11 (reporter) | COMPLETE | P19 Batch 2 (10 BCs) |
| ss-01/02/08/13 | CONFIRMED CLEAN | P19 Batch 2 |
| ss-04 (reassembly) | COMPLETE | P20 (B-01/B-02/B-03; remainder BC-2.04.012/013/014 now closed) |
| ss-12 (main/cli) | COMPLETE | P20 (B-04/B-05; BC-2.12.005 now closed) |

**PG-ARP-F2-007 FLUSHED corpus-wide.** All subsystems that cite shifted src files are now
anchored to develop HEAD 31d1231. Anchor-drift class is CLOSED.

#### Artifact versions post-Pass-20

| Artifact | Version | Change |
|----------|---------|--------|
| cap-09-finding-emission | v1.2 | Frontmatter version field corrected (D-01) |
| BC-2.04.012 | v2.0 | Invariant-1 latch prose :618→:647 (B-01) |
| BC-2.04.013 | v1.9 | expire call-site :166-169→:168-171 × 2 (B-02) |
| BC-2.04.014 | v1.6 | lifecycle.rs :60→:66 (B-03) |
| BC-2.12.005 | v1.5 | main.rs :87-122→:139-166 + Inv-4 :104-117→:147-161; cli.rs :71-122→:73-124 (B-04/B-05) |
| ADR-008 | v1.9 | T0830 matrix-label adjacent-sentence reconciliation (D-02) |
| BC-INDEX | updated | Version pin annotations for BC-2.04.012/013/014 + BC-2.12.005 |
| spec-changelog | updated | All P20 version bumps recorded |

---

### Pass 26 — 2026-06-13 (Claude adversary; CLEAN — 1/3)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
4 parallel fresh-context slices (A/B/C/D). STRICT mode — report ANY finding of ANY severity.
**Factory-artifacts HEAD reviewed (Slice B git guard):** b008b178.
**Findings:** 0 total — 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.
**Novelty:** NONE — first fully-clean whole-corpus pass.
**Convergence counter:** **1/3** (first clean pass advances streak; 2 more consecutive clean
passes required for F2 convergence gate).
**Verdict:** **CLEAN.**

#### Slice verdicts

| Slice | Scope | Verdict |
|-------|-------|---------|
| A | Architecture + verification-properties | CLEAN — 0 findings |
| B | BC-INDEX + all ss-01..ss-16 BC bodies (283 BCs) | CLEAN — 0 findings; git guard confirmed b008b178 |
| C | Domain + prd-supplements + HS-INDEX + ss-10 + STORY-INDEX | CLEAN — 0 findings |
| D | PRD + all indexes + spec-changelog + cross-doc counts | CLEAN — 0 findings |

#### Significance

This is the first pass in 26 total where all 4 slices simultaneously returned zero findings.
The corpus-wide debt flush across passes 14-25 eliminated all known defect classes:
- Field-rename propagation (pass 14-15)
- O-01 closure propagation (pass 14)
- MITRE counts in holdout scenarios (pass 17)
- Anchor-drift PG-ARP-F2-007 across all subsystems (passes 18-20, FLUSHED)
- Version-pin lag / self-induced churn (pass 22, hardened)
- changelog-path phantom slugs (passes 21, 24, 25, class FLUSHED)
- DNP3 component IDs C-23/C-24 (pass 24)

No remediation performed this pass. Clean count advances: **1/3**.

---

### Pass 27 — 2026-06-13 (Claude adversary; NOT_CLEAN→REMEDIATED; counter reset 1/3→0/3)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
4 parallel fresh-context slices (A/B/C/D). STRICT mode — report ANY finding of ANY severity.
**Findings:** 2 total — 0 CRITICAL, 0 HIGH, 2 MEDIUM, 0 LOW.
**Novelty:** MED — genuine items the P26-clean pass overlooked; FRESH-CONTEXT VARIANCE
(each fresh pass surfaces ~1-2 items prior passes missed; see PG-ARP-F2-008).
**Convergence counter:** **0/3** (reset from 1/3; remediation required; streak broken).
**Verdict:** NOT_CLEAN→REMEDIATED.

#### Slice verdicts

| Slice | Scope | Verdict |
|-------|-------|---------|
| A | Architecture + verification-properties | CLEAN — 0 findings |
| B | BC-INDEX + all ss-01..ss-16 BC bodies (283 BCs) | CLEAN — 0 findings |
| C | Domain + prd-supplements + HS-INDEX + ss-10 + STORY-INDEX | NOT_CLEAN — C-01 MED |
| D | PRD + all indexes + spec-changelog + cross-doc counts | NOT_CLEAN — D-01 MED |

#### Findings

**C-01 MED (PO, holdout layer):** HS-008 kill-chain narrative had Command-and-Control (C2)
appearing after Exfiltration — wrong position. Canonical all_tactics_in_report_order in
src/mitre.rs places C2 between Collection and Exfiltration:
...Collection → C2 → Exfiltration → Impact → [3 ICS tactics].
Fix: corrected HS-008 kill-chain narrative to match canonical sequence per src.

**D-01 MED (PO, holdout layer):** HS-INDEX line ~489 carried stale "v1.5" version-pin for
BC-2.02.009. The BC is currently at v1.6 (lax-arm fix Pass-10 + BC-INDEX pin corrected Pass-15).
Pin was stale and created BC-version-pin lag vulnerability (PG-ARP-F2-008 class).
Fix: dropped version pin for robustness (now version-agnostic, like P22 D-01 spec-side
hardening). Swept holdout layer for additional active version-pins — 1 found and flushed.

#### Remediation

Both findings were holdout-layer items in HS-008 and HS-INDEX. Remediation:
- HS-008: kill-chain narrative corrected (C2 order).
- HS-INDEX: BC-2.02.009 version-pin dropped.
- spec-changelog: Pass-27 remediation ledger entry added.

**Holdout BC-version-pin lag class hardened** (parallel to P22 spec-side version-pin
hardening): active version-pins in holdout scenario files drop-and-flush policy applied.
Progressively reducing the surface fresh passes can find.

**Class note (PG-ARP-F2-008):** Corpus is substantively converged. Fresh-context variance
at this stage is expected to produce ~1-2 MED items per pass that prior passes missed. Each
fix + class-hardening reduces the residual surface. The 0/3 counter reset is correct per
strict protocol; next aim is 3 consecutive all-slice-clean.

Trajectory P25-P27: 0C/0H → 0 (CLEAN P26) → 2 MED (P27 reset). Counter reset 0/3.
Next = whole-corpus Pass 28 via Claude adversary.

---

### Pass 28 — 2026-06-13 (Claude adversary; CLEAN — 1/3)

**Method:** Whole-corpus fresh-context pass; Claude vsdd-factory:adversary per human direction.
4 parallel fresh-context slices (A/B/C/D). STRICT mode — report ANY finding of ANY severity.
**Findings:** 0 total — 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.
**Novelty:** NONE — all 4 slices zero findings on the post-P27 corpus (holdout kill-chain
order corrected + HS-INDEX BC-2.02.009 version-pin dropped).
**Convergence counter:** **1/3** (streak RESTARTED after P27 reset; need 2 more consecutive clean).
**Verdict:** CLEAN.

Note: Slice D misreported factory HEAD as d734664 under its read-only profile; actual HEAD
is d0a392f (verified via `git -C .factory log -1 --format='%h %s'`). All four slices reviewed
the current corpus content and returned zero findings — read-only profile SHA misreport is a
known cosmetic artifact (PG-ARP-F2-001); verdict CLEAN stands.

#### Slice verdicts

| Slice | Scope | Verdict |
|-------|-------|---------|
| A | Architecture + verification-properties | CLEAN — 0 findings |
| B | BC-INDEX + all ss-01..ss-16 BC bodies (283 BCs) | CLEAN — 0 findings |
| C | Domain + prd-supplements + HS-INDEX + ss-10 + STORY-INDEX | CLEAN — 0 findings |
| D | PRD + all indexes + spec-changelog + cross-doc counts | CLEAN — 0 findings |

#### Findings

None. All 4 slices returned zero findings of ANY severity.

#### Remediation

None — clean pass. Counter advances to 1/3.

Trajectory P26-P28: 0 (CLEAN 1/3) → 2 MED (P27 reset) → 0 (CLEAN 1/3 restart).
Next = whole-corpus Pass 29 via Claude adversary. P29 clean → 2/3; P30 clean → 3/3 CONVERGED.

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

### [process-gap] PG-ARP-F2-005 — Sibling naming variants in sweep globs + partial-fix discipline

Pass-15 H3 sweep used glob `chunk*-eval.md` which matched chunk1-eval.md and chunk3-eval.md
but MISSED chunk3-reeval.md (the sibling reeval record). The erratum was caught only at Pass 16.

Additionally, partial-fix discipline failed twice in this cycle: (1) Pass-14 D-OBS-01 corrected
ADR-005:108 `[2,253]` but missed :74 (caught at Pass 16 D-01); (2) Pass-14 A-06 introduced
decode_packet STORY-114 anchor in one peer (api-surface) but the arp-architecture-delta §6
already cited STORY-111 — the wrong story number was introduced into the newer file rather than
verified against the authoritative doc.

**Candidate policy codification:** DF-SIBLING-SWEEP-001 sub-rule — when a fix touches one of N
instances of the same defect across sibling files or within a single file, enumerate ALL siblings
before committing. Sweep globs must include all naming variants: `chunk*eval.md`, `chunk*-eval.md`,
`chunk*reeval.md` — or use `find` over the directory. No partial-fix commits on multi-occurrence
defects.

---

### [process-gap] PG-ARP-F2-006 — Holdout count-assertions drift across feature cycles

HS-008/009/025 carried MITRE catalog count assertions (tactic count, seeded IDs, emitted IDs,
cat-only IDs) from the greenfield era. These counts were not swept when STORY-109 (DNP3 F4)
shipped MitreTactic::IcsImpact (17th variant) and expanded seeded/emitted IDs. The counts
drifted silently across the entire DNP3 feature cycle (F1-F7) + release v0.6.0 and were caught
only at Pass 17.

**Root cause:** Holdout-scenario count assertions are not live (they are scenario-file prose,
not test assertions), so compile/test CI does not catch them. No close-out sweep discipline
covered count-bearing holdout prose.

**Candidate policy codification:** Extend DF-CANONICAL-FRAME-HOLDOUT-001 (or add a sub-rule)
requiring that whenever src/mitre.rs seeded/emitted set or MitreTactic variant count changes,
a sweep of ALL holdout-scenarios carrying count assertions (grep for "seeded", "emitted",
"cat-only", "tactics", "MitreTactic variants") is mandatory in the same burst as the code
change. Candidate trigger: F-cycle close-out checklist step.

---

### [process-gap] PG-ARP-F2-007 — src-line-anchor drift across feature cycles (dispatcher.rs)

Feature cycles that insert code into a shared file (e.g., dispatcher.rs via Modbus Rule-5 and
DNP3 Rule-6) shift the line offsets of EVERY function in that file, leaving EVERY citing BC's
line anchors stale. Pass 18 discovered all 9 ss-05 BCs and BC-2.04.055 were stale for the same
reason — the last dispatcher.rs anchor sync (v1.3) predated ICS feature integration.

**Root cause:** F-cycle close-out does not include an anchor-resync sweep over BCs citing src
files that were modified during delivery. CI cannot enforce this (anchors are prose in spec
files, not executable).

**Anchor-drift watch (post-P18):**
- ss-10 (mitre.rs): RE-SYNCED (Pass 13 + ongoing checks).
- ss-05 (dispatcher.rs): RE-SYNCED this pass (Pass 18).
- BC-2.04.055 (dispatcher.rs on_data): RE-SYNCED this pass.
- ss-09 (findings.rs): NOT YET audited — likely shifted by STORY-097/098/099 timestamp wiring.
- ss-06/ss-07 (http.rs/tls.rs/reassembly): NOT YET audited — potentially shifted by
  STORY-097/098/099 or earlier reassembly work.
- ss-04 (dispatcher.rs: classify, other): partially covered by BC-2.04.055; remaining ss-04
  anchors should be spot-checked.

**Candidate policy codification:** F-cycle close-out checklist step: for each src file touched
by the feature's delivered stories, grep all BC files for citations of that src file and
re-verify each cited line number against the current source. Candidate lint: anchor-drift
checker script that reads BC `Architecture Anchor` fields and spot-checks line content against
current src.

---

## Notes

- Pass 1 was monolithic (pre-SLICED method adoption for this feature). Passes 2+ use the
  4-slice parallel method per user direction.
- Slice reports from passes 2-5 live in the orchestrator session transcript (see
  PG-ARP-F2-001 above). This file captures the distilled per-pass summary.
- The SLICED method is proving effective at surfacing cross-doc consistency issues (Slice D)
  that monolithic passes miss — passes 2-4 all had their largest finding cluster in Slice D.
