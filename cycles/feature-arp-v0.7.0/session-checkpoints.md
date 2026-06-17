---
document_type: session-checkpoints-archive
cycle: feature-arp-v0.7.0
producer: state-manager
timestamp: 2026-06-13T00:00:00Z
---

# Session Checkpoints Archive — ARP Analyzer feature-arp-v0.7.0

Archived per content-routing rules (STATE.md keeps only the LATEST checkpoint).

---

## Archived Checkpoint: 2026-06-15 — F4 wave-level adversarial Pass 1 NOT CLEAN; D-074 REMEDIATED via PR #242; NEXT = fresh F4 wave-level adversarial Pass 1

**Archived from STATE.md on:** 2026-06-15 (replaced by "F4 wave-level adversarial GATE SATISFIED 3/3; NEXT = holdout eval" checkpoint)

### POSITION

- F4 per-story delivery COMPLETE. All 5 ARP stories delivered to develop fee71ee.
- F4 wave-level adversarial convergence IN PROGRESS (0/3 at time of archival).
  - Pass 1 (2026-06-15): NOT CLEAN — F-ARP-F4P1-001 MEDIUM (threshold-0 not rejected).
  - D-074 adjudicated (research-agent HIGH confidence). PR #242 merged fee71ee.
  - BC-2.16.008 v1.7→v1.8; BC-2.16.012 v1.2→v1.3; BC-2.16.013 v1.2→v1.3.
  - STORY-114 v1.1→v1.2; STORY-115 v1.1→v1.2. Input-hashes restamped.
  - Consistency-validator: CONSISTENT. Counter reset to 0/3. Next = fresh Pass 1.
- develop HEAD: fee71ee. develop == origin/develop.
- Decisions active: D-047..D-074; do NOT re-adjudicate D-068/D-069/D-071/D-072/D-073/D-074.
- F3-OBL-STORY114-001/002/003 REVOKED (D-069).

---

## Archived Checkpoint: 2026-06-15 — F4 ARP DELTA-IMPLEMENTATION; STORY-115 Step-4.5 CONVERGED; NEXT = demo-recorder → pr-manager → F4 wave-level convergence

**Archived from STATE.md on:** 2026-06-15 (replaced by "STORY-115 Step-4.5 CONVERGED + DEMO-RECORDED; NEXT = pr-manager" checkpoint)

### POSITION

- STORY-111/112/113/114 ALL DELIVERED to develop 7c0f453.
- Wave 44 / STORY-115: Step-4.5 CONVERGED — HEAD dcdbf95; base 7c0f453; 1571 tests; 3 clean passes (a6f45a32/acbe2f5b/a58db908). D3 storm + --arp-storm-rate + storm_findings value. FINAL E-16 story. PR pending.
- develop HEAD: 7c0f453.
- DF-GREEN-DOC-TENSE-SWEEP v1 added to policies.yaml; PG-ARP-F4-REDTEST-DOC-TENSE sub-rule codified; PG-ARP-F4-MULTIPASS-VALUE positive lesson documented.
- NEXT: demo-recorder → pr-manager (merge STORY-115) → F4-wave-level convergence + holdout eval → F5/F6/F7.

---

## Archived Checkpoint: 2026-06-15 — F4 ARP DELTA-IMPLEMENTATION; STORY-114 DELIVERED PR #240 7c0f453; NEXT = STORY-115

**Archived from STATE.md on:** 2026-06-15 (replaced by "STORY-115 Step-4.5 CONVERGED; NEXT = demo-recorder → pr-manager → F4 wave-level convergence" checkpoint)

### POSITION

- STORY-111 DELIVERED (PR #236 cced898). STORY-112 DELIVERED (PR #238 10e4472).
- STORY-113 DELIVERED (PR #239 7b7dbb2). STORY-114 DELIVERED (PR #240 7c0f453; pr-reviewer APPROVE, zero blocking; 9 CI checks green).
- develop HEAD 7c0f453. src/mitre.rs on develop: SEEDED=25, EMITTED=17.
- Input-hash: all 5 ARP stories MATCH (STORY-115=2e0eca2 verified 2026-06-15).
- Wave 44 / STORY-115: NOT STARTED. Branches from develop HEAD 7c0f453.
- PG-ARP-F4-GREEN-DOC-TENSE codified (lessons.md); PG-ARP-F4-PRMGR-MERGE-SHORTSTOP recurrence #5 logged.
- NEXT: STORY-115 (D3 storm detection + --arp-storm-rate + storm_findings value + final new() arg).

---

## Archived Checkpoint: 2026-06-15 — F4 ARP DELTA-IMPLEMENTATION; STORY-113 Step-4.5 CONVERGED (0437be6; 1535 tests); NEXT = demo-recorder → pr-manager → STORY-114

**Archived from STATE.md on:** 2026-06-15 (replaced by "STORY-113 DELIVERED PR #239 7b7dbb2; NEXT = STORY-114" checkpoint)

### POSITION

- STORY-111 DELIVERED (PR #236 cced898). STORY-112 DELIVERED (PR #238 10e4472).
- Wave 42 / STORY-113: Step-4.5 CONVERGED — 3/3 clean passes (ad044181/ae1383274/ad2223ab) on frozen diff 0437be6. 1535 tests. F-113-01 HIGH RESOLVED. json.rs = develop baseline.
- develop HEAD: 10e4472 (STORY-113 merge pending at time of checkpoint).
- Input-hash: STORY-113=7c61bae MATCH, STORY-114=5705a10 MATCH, STORY-115=2e0eca2 MATCH (re-stamped 68885d4).
- NEXT: demo-recorder → pr-manager (STORY-113). Then STORY-114.

---

## Archived Checkpoint: 2026-06-12 — ARP Feature F1 PASSED / F2 PENDING

**Archived from STATE.md on:** 2026-06-13 (replaced by "F2 STRICT WHOLE-CORPUS CONVERGENCE, mid-loop" checkpoint)

### POSITION

wirerust **v0.6.0 RELEASED 2026-06-12**. Feature: ARP security analyzer + etherparse
0.16→0.20.1 migration — **F1 Delta Analysis PASSED** (human-gated 2026-06-12, D-066).
**F2 Spec Evolution was NEXT** (at time of archival, F2 is IN PROGRESS — 13 passes run).

**F1 outcome:** DecodedFrame{Ip,Arp} integration approach selected. ADR-008 planned.
ArpAnalyzer owns bounded IP↔MAC binding table. etherparse 0.20 migration is sub-delta A
(SliceError::Len removed; 2 non-exhaustive match breaks; DecodedFrame return-type change).
Estimate: 18-24 new BCs (SS-16), 1 revised BC (BC-2.02.009), VP-024, ADR-008, 5-6 stories
(E-16), 3-5 holdout scenarios. MITRE T0830 (primary) + T1557.002 (secondary) — validated
ATT&CK v19.1.

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | 5af59a0 | factory(F2): Pass 13 whole-corpus convergence trajectory + STATE update (2026-06-13) |

### RESUMING-ORCHESTRATOR RUNBOOK (at time of archival)

1. `vsdd-factory:factory-worktree-health` — verify `.factory/` on `factory-artifacts` branch.
2. Read `STATE.md`. Confirm develop==origin/develop, working tree clean.
3. `bin/compute-input-hash --scan` — expect MATCH=62 STALE=0 ERROR=1 (STORY-091 known).
4. Active phase was **Feature F2 Spec Evolution** — dispatch `vsdd-factory:phase-f2-spec-evolution`.

### KEY ARTIFACT POINTERS (at archival)

- F1 delta analysis: `.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md`
- F1 affected files: `.factory/phase-f1-delta-analysis/arp-affected-files.txt`
- F1 MITRE research: `.factory/phase-f1-delta-analysis/mitre-arp-research.md`
- F1 MITRE additional detections: `.factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md`

---

## Archived Checkpoint: 2026-06-13 — Pass 21 REMEDIATED; 0C/0H cosmetic; Slices A+C CLEAN 2nd consecutive

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 22 REMEDIATED; 0C/0H; 5th consecutive; version-pin hardened" checkpoint)

### POSITION

- **21 adversarial passes + 1 corpus consistency audit run. Pass 21 REMEDIATED.**
- F2 adversarial convergence: STRICT WHOLE-CORPUS mode. Counter 0/3.
- Pass 21 (5 findings): 0C/0H; all cosmetic/ledger hygiene. B-01 LOW (PO): BC-INDEX ss-11
  table stray blank line between BC-2.11.001 and BC-2.11.002 split the Markdown table —
  removed (BC-INDEX v1.24→v1.25). D-01 MED (PO): spec-changelog Pass-13 ledger cited
  `specs/behavioral-contracts/ARCH-INDEX.md` — corrected to `specs/architecture/ARCH-INDEX.md`.
  D-02 MED (PO): spec-changelog Pass-13 ledger cited `vp-005-no-panic-guarantee.md` — corrected
  to `vp-005-sni-four-way-classification.md`. D-03 MED (PO): spec-changelog Pass-13 ledger cited
  `vp-008-all-analyzers-pure.md` — corrected to `vp-008-decode-packet-no-panic.md`. D-04 LOW (PO):
  PRD body version-history missing delta notes for 1.13/1.14/1.15/1.16/1.18; notes added (prd.md
  v1.18→v1.19). Slices A+C CLEAN (2nd consecutive clean for both slices).
  Trajectory P14-21: 2C/5H→2C/1H→0C/0H→3C/2H→0C/3H→0C/8H→0C/1H→0C/0H. DECAYING strongly.
- Next action at time of archival: whole-corpus Pass 22 via Claude (first-clean candidate).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |

---

## Archived Checkpoint: 2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 22 REMEDIATED; 0C/0H; 5th consecutive; version-pin hardened

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 23 REMEDIATED; B/C/D CLEAN" checkpoint)

### POSITION

- **22 adversarial passes + 1 corpus consistency audit run. Pass 22 REMEDIATED.**
- F2 adversarial convergence: STRICT WHOLE-CORPUS mode. Counter 0/3.
- Pass 22 (8 raw findings; 3 discarded no-action/NON-BLOCKING; 5 valid): 0C/0H. C-01 MED
  (domain-debt O-04: "21 IDs"→"23 IDs" — Feature #8 DNP3 +2; domain-debt v1.2→v1.3). A-01 LOW
  (verification-architecture: Pass-22 modified entry wording hardened; v1.5→v1.6). A-02 LOW
  (verification-coverage-matrix: VP-024 draft Coverage Note added; v1.4→v1.5). D-01 LOW
  (BC-INDEX: PRD version-pin dropped for robustness — self-induced lag from P21 prd v1.19 bump;
  now version-agnostic; v1.25→v1.26). B-01 LOW (BC-INDEX: double-blank before ss-12 removed;
  v1.25→v1.26). Proactive version-citation robustness sweep run — only 1 current-state cross-doc
  version-pin found; now dropped. PG-ARP-F2-008 noted: 5th consecutive 0-CRIT/HIGH; corpus
  substantively converged; remaining churn cosmetic.
  Trajectory P14-22: 2C/5H→2C/1H→0C/0H→3C/2H→0C/3H→0C/8H→0C/1H→0C/0H→0C/0H. DECAYING strongly.
- Next action at time of archival: whole-corpus Pass 23 via Claude (first-clean candidate).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 23 REMEDIATED; 0C/0H; 6th consecutive; B/C/D CLEAN

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 24 REMEDIATED; 0C/1H; 7th consecutive 0-CRIT; B+C CLEAN" checkpoint)

_(This checkpoint was further superseded by the Pass-25 REMEDIATED checkpoint archived below.)_

### POSITION

- **23 adversarial passes + 1 corpus consistency audit run. Pass 23 REMEDIATED.**
- F2 adversarial convergence: STRICT WHOLE-CORPUS mode. Counter 0/3.
- Pass 23 (5 findings; Slices B/C/D all CLEAN; Slice A only): 0C/0H. A-01 MED (verification-
  coverage-matrix: VP-024 lock-note cited STORY-112/F6 — self-induced from P22 A-02; corrected
  to STORY-113/F6; v1.5→v1.6). A-02 LOW (verification-coverage-matrix: decoder.rs Sub-A
  attribution footnote; v1.5→v1.6). A-03 LOW (verification-architecture: VP-005 harness skeleton
  code-fence fixed; v1.6→v1.7). A-04 LOW (module-criticality: C-22 Modbus technique enumeration
  harmonized with C-23/C-24; v1.2→v1.3). A-05 LOW (arp-architecture-delta: §6 draft-as-
  authoritative note added; v1.10→v1.11). KEY: A-01 was self-induced churn. 3 of 4 slices CLEAN.
  Substantively + cosmetically near-converged. 6th consecutive 0-CRIT/HIGH.
  Trajectory P21-23: 0C/0H → 0C/0H → 0C/0H. DECAYING strongly.
- Next action at time of archival: whole-corpus Pass 24 via Claude (strong first-clean candidate).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 24 REMEDIATED; 0C/1H; 7th consecutive 0-CRIT; B+C CLEAN

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 25 REMEDIATED; 0C/0H; 8th consecutive 0-CRIT; A/B/C CLEAN" checkpoint)

### POSITION

- **24 adversarial passes + 1 corpus consistency audit run. Pass 24 REMEDIATED.**
- F2 adversarial convergence: STRICT WHOLE-CORPUS mode. Counter 0/3.
- Pass 24 (4 findings; Slices B+C CLEAN): 0C/1H. D-01 HIGH genuine (PO): systematic DNP3
  component mislabel — all 24 ss-15 BCs labeled DNP3 as C-23 (canonical C-24; C-23 is PLANNED
  ARP); prd.md §2.15 cited "C-26" (phantom). Fixed: 24 ss-15 BCs (C-23→C-24) + prd
  (C-26→C-24; v1.19→v1.20). A-01 LOW self-induced (arp-architecture-delta §7 row order;
  no bump). D-02/D-03 MED self-induced changelog paths from P23 commit. 3 of 4 findings
  self-induced churn (PG-ARP-F2-008). Mitigations: no-bump reorders + verified changelog
  paths now standard. 7th consecutive 0-CRIT.
  Trajectory P22-P24: 0C/0H → 0C/0H → 0C/1H.
- Next action at time of archival: whole-corpus Pass 25 via Claude.

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 25 REMEDIATED; 0C/0H; 8th consecutive 0-CRIT; A/B/C CLEAN; changelog-path class FLUSHED

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 26 CLEAN; 1/3; ALL 4 SLICES ZERO FINDINGS; CONVERGENCE STREAK STARTED" checkpoint)

### POSITION

- **25 adversarial passes + 1 corpus consistency audit run. Pass 25 REMEDIATED.**
- F2 adversarial convergence: STRICT WHOLE-CORPUS mode. Counter 0/3.
- Pass 25 (2 findings; Slices A/B/C CLEAN; Slice D only): 0C/0H. D-01 MED (PO): spec-changelog
  File column for VP-023 row cited truncated slug vp-023.md → corrected to
  vp-023-dnp3-parse-safety.md. D-02 MED (PO): spec-changelog File column for VP-022 row
  cited truncated slug vp-022.md → corrected to vp-022-modbus-parse-safety.md.
  REMEDIATION: comprehensive changelog-path-phantom flush — scanned ALL .factory/*.md paths
  in spec-changelog.md; found 4 non-resolving paths; fixed 2 active File-column refs
  (VP-022/VP-023 truncated slugs); other 2 remain in corrected-from audit prose only
  (correctly preserved as audit trail). Zero active ledger refs now point at non-resolving
  paths. Changelog-path debt class FLUSHED. 8th consecutive 0-CRIT.
  Trajectory P23-P25: 0C/0H → 0C/1H → 0C/0H.
- Next action at time of archival: whole-corpus Pass 26 via Claude (strong first-clean candidate).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — Pass 27 NOT_CLEAN→REMEDIATED; counter reset 0/3; holdout-pin-hardened

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 28 CLEAN; counter 1/3; streak restarted" checkpoint)

### POSITION

- Pipeline phase: Feature Mode F2 (Spec Evolution) — adversarial convergence, IN PROGRESS.
- F2 adversarial convergence: STRICT WHOLE-CORPUS mode. Counter **0/3** (reset from 1/3 at Pass 27).
- 27 adversarial passes + 1 corpus consistency audit run. Pass 27 NOT_CLEAN→REMEDIATED.
  Pass 27: Slices A+B CLEAN; C-01 MED (HS-008 kill-chain order corrected — C2 between
  Collection and Exfiltration); D-01 MED (HS-INDEX BC-2.02.009 "v1.5" pin dropped;
  holdout layer swept — 1 active pin flushed). Both genuine; fresh-context variance.
  PG-ARP-F2-008: holdout BC-version-pin lag class hardened.
- Next action at time of archival: whole-corpus Pass 28 via Claude adversary.

### VERIFIED SHAs at archival time (Pass 27 checkpoint)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 30 NOT_CLEAN→REMEDIATED; counter 0/3

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 31 CLEAN; counter 1/3" checkpoint)

### POSITION (at archival time)

- Counter: 0/3 (Pass 30 NOT_CLEAN→REMEDIATED; 4 HIGH genuine defects found).
- 30 adversarial passes + 1 corpus consistency audit run.
- Pass 30: Slice D clean; Slices A/B/C found 5 genuine defects (4 HIGH + 1 MED).
  B-01/B-02/B-03 HIGH: BC-2.14.018 v1.2→v1.3 + BC-2.14.020 v2.2→v2.3 (FlowKey accessor fix).
  C-01 HIGH: STORY-100..105 input-hash dup-key removed; all 6 MATCH.
  A-01 MED: ADR-006 FC-0x17 attribution corrected (T0836 bucket).
- Next action at time of archival: whole-corpus Pass 31 via Claude adversary.

### VERIFIED SHAs at archival time (Pass 30 checkpoint)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 31 CLEAN; counter 1/3

**Archived from STATE.md on:** 2026-06-13 (replaced by "Pass 32 CLEAN; counter 2/3" checkpoint)

### POSITION (at archival time)

- Counter: 1/3 (Pass 31 CLEAN; streak restarted after P30 NOT_CLEAN remediation).
- 31 adversarial passes + 1 corpus consistency audit run.
- Pass 31: All 4 slices (A/B/C/D) zero findings. P30 HIGH fixes held (BC-2.14.018 v1.3,
  BC-2.14.020 v2.3, STORY-100..105 input-hash dup-key removed, ADR-006 FC-0x17 corrected).
  Slice B noted BC-INDEX:358 trailing-pipe cosmetic — explicitly ruled non-blocking/not-a-finding.
- Next action at time of archival: whole-corpus Pass 32 via Claude adversary.

### VERIFIED SHAs at archival time (Pass 31 checkpoint)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 32 CLEAN; counter 2/3

**Archived from STATE.md on:** 2026-06-13 (replaced by "F2 CONVERGED — Pass 33 CLEAN; counter 3/3" checkpoint)

### POSITION (at archival time)

- Counter: 2/3 (Pass 31+32 CLEAN; 2 consecutive clean passes).
- 32 adversarial passes + 1 corpus consistency audit run.
- Pass 32: All 4 slices (A/B/C/D) zero findings. Second consecutive clean pass post-P30
  remediation. BC-INDEX:358 trailing-pipe correctly treated non-blocking by all slices
  (consistent with Pass 31 watch-item ruling).
- Next action at time of archival: whole-corpus Pass 33 via Claude adversary (convergence-decider).

### VERIFIED SHAs at archival time (Pass 32 checkpoint)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |

---

## Archived Checkpoint: 2026-06-13 — F2 CONVERGED 3/3; D-067 adjudicated; F2→F3 gate SATISFIED

**Archived from STATE.md on:** 2026-06-13 (replaced by hardened cold-resume F3 brief)

### POSITION (at archival time)

- **F2 adversarial convergence:** STRICT WHOLE-CORPUS mode. Bar = 3 consecutive passes with
  ZERO findings of ANY severity across the ENTIRE spec corpus. Counter 3/3 CONVERGED
  (Pass 31+32+33 CLEAN; 3 consecutive). 33 adversarial passes total.
  F2 STRICT-WHOLE-CORPUS ADVERSARIAL GATE SATISFIED.
- **Pass 33 (Claude):** All 4 slices (A/B/C/D) returned ZERO findings. Slice D noted one
  non-blocking observation: PRD v1.20 delta:285 "C-23 was MbapFramer" — factually-wrong
  historical rationale; within the corrected-from-prose non-blocking exemption; verdict CLEAN.
  Tracked as DRIFT-PRD-V120-MBAPFRAMER-001 (cosmetic; LOW; deferred).
- **D-067 adjudicated 2026-06-13:** IcsImpact Display canonical = "Impact" (spec correct);
  src/mitre.rs:91 "Impact (ICS)" is deviant. Severity LOW. Fix folded into STORY-114.
  F2 NO SPEC CHANGE — convergence preserved. F2→F3 gate condition SATISFIED.
- **Next action at time of archival:** F3 ARP story decomposition (STORY-111..115).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | 4fb17e6 | factory(D-067): IcsImpact Display adjudication — decision record only; F2 convergence preserved |

---

## Archived Checkpoint: 2026-06-13 — F3 ARP STORY DECOMPOSITION; cold-resume hardened (pre-F3-adversarial)

**Archived from STATE.md on:** 2026-06-14 (replaced by "Pass-21 remediation complete; strict 3/3 convergence in progress (0/3)" checkpoint)

### POSITION (at archival time)

- F2 CONVERGED 3/3. D-067 adjudicated; D-068 + D-069 issued (2026-06-14).
- F3 ARP story decomposition: STORY-111..115 CREATED (epic E-16, 47 pts, linear chain).
  HS-INDEX waves 40-44 + holdout scenarios authored. STORY-INDEX, dependency-graph,
  wave-schedule updated. All 5 ARP stories MATCH (d5bda72/268f53f/a767d96/e2f1c95/5ca9835).
- F3 adversarial convergence: STRICT WHOLE-CORPUS mode. Counter 0/3 at time of archival.
  Pass-21 remediation COMPLETE. Next action: Pass 22 (clean-streak attempt 1/3).
- factory HEAD at archival: 2f47145 (factory(state): correct stale story count and wrong ICS variant names)

### KEY FACTS (at archival time)

- D-068: benign GARP emits mitre_techniques: [] (BC-2.16.003 v1.7, ADR-008 v2.0).
- D-069: IcsImpact Display canonical = "Impact (ICS)" — SUPERSEDES D-067.
  F3-OBL-STORY114-001/002/003 REVOKED.
- F3 adversarial passes run: 21 (all remediated). Clean-streak: 0/3 at archival.
  Pass 17 = first fully-clean pass; Pass 18 broke streak (VP title-sync);
  Passes 19/20/21 surfaced+remediated genuine items. Pass-21 remediation COMPLETE.
- 3 flush audits completed + cleared: dependency-graph whole-file, VP-layer + index
  title-sync, story-completeness (BC-PC→AC→test). Wave-28-34 dependency-graph gap closed.

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |

---

## Archived Checkpoint: 2026-06-14 — F3 ARP adversarial convergence — Pass-22 remediation complete

**Archived from STATE.md on:** 2026-06-14 (replaced by "Pass-23 remediation complete" checkpoint)

### POSITION

F3 adversarial convergence STRICT 3/3 IN PROGRESS. Pass-22 remediation COMPLETE. Clean-streak: 0/3.
NEXT ACTION (at archival time): Run Pass 23 (clean-streak attempt 1/3).

Pass-22 remediation items: SS-15 24 BCs story-anchors back-filled (STORY-106..110); PRD
seed-count reconciled 26→28 (v1.22); VP-024 module + VP-INDEX 5-BC note corrected;
dep-graph-extended superseded; DRIFT-PRD-V120-MBAPFRAMER-001 CLOSED. Pass 17 = first
fully-clean pass. Passes 18-22 each surfaced+remediated genuine items.

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | 6fee89a | factory(F3): Pass-22 remediation |
| factory-artifacts HEAD | 2f47145 | factory(state): correct stale story count and wrong ICS variant names |

---

## Archived Checkpoint: 2026-06-14 — F3 ARP adversarial convergence — Pass-31 FULLY CLEAN; clean-streak 1/3; strict 3/3 in progress

**Archived from STATE.md on:** 2026-06-14 (replaced by Pass-32 NOT CLEAN / STORY-115 v1.1 REMEDIATED checkpoint)

### POSITION at archival time

- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED (P33 CLEAN; 3/3). D-068/D-069 applied post-F2.
- F3: STORY-111..115 created (E-16, 47 pts). All 5 ARP stories MATCH (d5bda72/268f53f/a767d96/e2f1c95/5ca9835).
- F3 Adversarial Convergence: STRICT WHOLE-CORPUS, IN PROGRESS.
  Pass-31 FULLY CLEAN — all 4 slices ZERO (Slice A 10th-consec ZERO; B converged; C ZERO
  [mount-guard PASSED; P30 env glitch confirmed resolved]; D ZERO). Clean-streak: **1/3**.
  USER DIRECTIVE: CONTINUE STRICT 3/3 indefinitely.
  NEXT ACTION at archival: Run Pass 32 (clean-streak attempt 2/3 — need 2 more consecutive clean).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | verify via git -C .factory log -1 | see prior burst |

---

## Archived Checkpoint: 2026-06-14 — F3 ARP adversarial convergence — Pass-32 NOT CLEAN; STORY-115 v1.1 REMEDIATED; clean-streak RESET 0/3; strict 3/3 in progress

**Archived from STATE.md on:** 2026-06-14 (replaced by Pass-33 NOT CLEAN / BC-2.15.024 v1.7 REMEDIATED checkpoint)

### POSITION at archival time

- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED (P33 CLEAN; 3/3). D-068/D-069 applied post-F2.
- F3: STORY-111..115 created (E-16, 47 pts). All 5 ARP stories MATCH (d5bda72/268f53f/a767d96/e2f1c95/5ca9835).
- F3 Adversarial Convergence: STRICT WHOLE-CORPUS, IN PROGRESS.
  Pass-32 NOT CLEAN — Slices A/B/D ZERO (A 11th-consec); Slice C 1 MED (STORY-115
  `storm_findings_count` → `storm_findings` cross-story field-name drift vs STORY-113:254
  + BC-2.16.010 summarize key). REMEDIATED STORY-115 v1.1 (6 occurrences corrected).
  Sibling sweep ZERO remaining live `storm_findings_count`. Clean-streak RESET: 1/3 → 0/3.
  USER DIRECTIVE: CONTINUE STRICT 3/3 indefinitely.
  NEXT ACTION at archival: Run Pass 33 (clean-streak attempt 1/3).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | bed0906 | factory(F3): Pass-32 remediation — STORY-115 storm_findings field-name (clean-streak reset 0/3); STATE compaction |

---

## Archived Checkpoint: 2026-06-14 — F3 ARP adversarial convergence — Pass-33 NOT CLEAN; BC-2.15.024 v1.7 REMEDIATED; clean-streak 0/3; strict 3/3 in progress

**Archived from STATE.md on:** 2026-06-14 (replaced by Pass-34 checkpoint)

### POSITION at archival time

- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED (P33 CLEAN; 3/3). D-068/D-069 applied post-F2.
- F3: STORY-111..115 created (E-16, 47 pts). All 5 ARP stories MATCH (d5bda72/268f53f/a767d96/e2f1c95/5ca9835).
- F3 Adversarial Convergence: STRICT WHOLE-CORPUS, IN PROGRESS.
  Pass-33 NOT CLEAN — Slices A/C/D ZERO (Slice D 3rd-consec ZERO); Slice B 1 MED
  (BC-2.15.024 Related-BCs descriptor: `parse_errors` wrongly listed in reset set —
  corrected to `malformed_in_window` per Inv 1/PC5/Arch-Anchors + BC-2.15.015 +
  dnp3.rs:984-995). REMEDIATED BC-2.15.024 v1.7. Sibling sweep CLEAN.
  POST-P33 SS-15 proactive flush COMPLETE: 6 findings remediated (BC-2.15.014 v2.0 six-field
  reset; reciprocal Related-BCs ×4: 014↔016/016↔010/015↔024/022↔016;
  BC-2.15.012 v1.4 + BC-2.15.023 v1.6 SAVE_CONFIGURATION). clean-streak UNCHANGED 0/3.
  USER DIRECTIVE: CONTINUE STRICT 3/3 indefinitely.
  NEXT ACTION at archival: Run Pass 34 (clean-streak attempt 1/3).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | 6993e6a | factory(F3): post-P33 SS-15 flush — BC-2.15.014 six-field reset, reciprocal Related-BCs ×4, FC 0x13 SAVE_CONFIGURATION ×2 |

---

## Archived Checkpoint: 2026-06-14 — F3 ARP adversarial convergence — Pass-36 FULLY CLEAN; all 4 slices ZERO; clean-streak 1/3; strict 3/3 in progress

**Archived from STATE.md on:** 2026-06-14 (replaced by Pass-37 checkpoint)

### POSITION at archival time

- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED (P33 CLEAN; 3/3). D-068/D-069 applied post-F2.
- F3: STORY-111..115 created (E-16, 47 pts). All 5 ARP stories MATCH (d5bda72/268f53f/a767d96/e2f1c95/5ca9835).
- F3 Adversarial Convergence: STRICT WHOLE-CORPUS, IN PROGRESS.
  Pass-36 FULLY CLEAN — all 4 slices ZERO (A 15th-consec, B converged, C converged, D converged);
  mount-guards PASSED. Post-P35 changelog de-pin flush + SS-15 flush eliminated the recurring
  Slice-D/B churn. 2nd fully-clean pass overall (P31 was first, reset by P32 storm_findings).
  Clean-streak 0/3→1/3.
  USER DIRECTIVE: CONTINUE STRICT 3/3 indefinitely.
  NEXT ACTION at archival: Run Pass 37 (clean-streak attempt 2/3).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | verify live | run: git -C .factory log -1 --format='%h %s' |

---

## Archived Checkpoint: 2026-06-14 — F3 ARP adversarial convergence — Pass-37 FULLY CLEAN; all 4 slices ZERO; clean-streak 2/3; strict 3/3 in progress

**Archived from STATE.md on 2026-06-14 when Pass-38 completed and F3 gate was satisfied.**

### Pipeline Position

- F3 Adversarial Convergence: STRICT WHOLE-CORPUS, IN PROGRESS.
  Pass-37 FULLY CLEAN — all 4 slices ZERO (A 16th-consec, B converged, C converged, D converged);
  mount-guards PASSED. clean-streak 1/3→2/3. P36+P37 consecutive clean passes.
  ONE more clean pass (Pass 38) satisfies the F3 strict 3/3 gate.
  USER DIRECTIVE (2026-06-14): CONTINUE STRICT 3/3 indefinitely.
  NEXT ACTION: Run Pass 38 (clean-streak attempt 3/3 — satisfies F3 gate if clean).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 31d1231 | — |
| main HEAD | 3e29891 | chore: release v0.6.0 |
| factory-artifacts HEAD | 1b33aae | factory(F3): Pass 37 FULLY CLEAN — all 4 slices ZERO; clean-streak 2/3 |

---

## Archived Checkpoint: 2026-06-14 — F4 ARP DELTA-IMPLEMENTATION IN PROGRESS; STORY-111 DELIVERED PR #236 cced898; NEXT = STORY-112 delivery

**Archived from STATE.md on 2026-06-14 (replaced by durable cold-resume checkpoint post STORY-112 stub in-progress).**

### Pipeline Position

- Mode: FEATURE. Active feature: ARP security analyzer + etherparse 0.16→0.20. Issue #9. Release target: v0.7.0.
- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED 3/3 (Passes 31/32/33). F3 CONVERGED 3/3 (Passes 36/37/38). F3 human gate PASSED (D-070, 2026-06-14).
- F4 Delta-Implementation: IN PROGRESS — AUTHORIZED (D-070).
  - STORY-111 DELIVERED — PR #236 MERGED to develop (merge commit cced898; wave 40 COMPLETE; D-073).
    etherparse 0.20 migration + DecodedFrame{Ip,Arp} enum + ArpFrame struct + decode_packet→Result<DecodedFrame> +
    symmetric-unreachable ARP decode (D-072) + non-panicking extract_arp_frame placeholder + BC-2.02.009 v1.7 +
    VP-008 fuzz-harness return-type update. 53 test suites green; clippy/fmt clean.
    CI Format failure fixed by aligning local toolchain to CI rolling-stable (rustfmt 1.8.0→1.9.0). pr-reviewer APPROVE.
  - STORY-112 IN PROGRESS NEXT (wave 41): stub-architect/test-writer → implementer → Step-4.5 (3/3) → demo → pr-manager 9-step PR.
- D-068/D-069/D-071/D-072/D-073 all active. F3-OBL-STORY114-001/002/003 REVOKED (D-069).
- Key input-hashes at archival: STORY-111=d05149f, STORY-112=8a4d566, STORY-113=a767d96, STORY-114=e2f1c95, STORY-115=5ca9835.
  Note: STORY-113/114/115 went STALE after arp-architecture-delta v1.16 (D-072) — re-stamp before STORY-113 delivery.

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | cced898 | PR #236 merge commit; local == origin/develop |
| main HEAD | 3e29891 | v0.6.0 |
| factory-artifacts HEAD | c429c92 | factory(F4): STORY-111 DELIVERED (D-073) — PR #236 merged develop cced898; NEXT STORY-112 |
| open PRs | none | gh pr list --state open |

---

## Archived Checkpoint: 2026-06-15 — F4 ARP DELTA-IMPLEMENTATION; STORY-113 DELIVERED (PR #239 7b7dbb2); NEXT = STORY-114

**Archived from STATE.md on 2026-06-15 (replaced by "STORY-114 Step-4.5 CONVERGED; NEXT = demo-recorder → pr-manager → STORY-115" checkpoint).**

### Pipeline Position

- Mode: FEATURE. Active feature: ARP security analyzer + etherparse 0.16→0.20. Issue #9. Release target: v0.7.0.
- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED 3/3 (Passes 31/32/33). F3 CONVERGED 3/3 (Passes 36/37/38). F3 human gate PASSED (D-070, 2026-06-14).
- F4 Delta-Implementation: IN PROGRESS — AUTHORIZED (D-070).
  - STORY-111 DELIVERED — PR #236 merged to develop (merge commit cced898; wave 40 COMPLETE; D-073).
  - STORY-112 DELIVERED — PR #238 merged to develop (merge commit 10e4472; wave 41). pr-reviewer APPROVE (cycle 1). 1512 tests. Step-4.5 CONVERGED BC-5.39.001.
  - STORY-113 DELIVERED — PR #239 merged to develop (merge commit 7b7dbb2; wave 42). pr-reviewer APPROVE; 2 non-blocking items fixed pre-merge (a73fbd6). 1535 tests. Step-4.5 CONVERGED BC-5.39.001 (3/3 clean passes ad044181/ae1383274/ad2223ab). F-113-01 HIGH RESOLVED.
  - STORY-114: worktree established; TDD IN PROGRESS at time of archival.
- develop HEAD: 7b7dbb2. STORY-114 branches from 7b7dbb2.
- Input-hash: STORY-113=7c61bae MATCH (DELIVERED), STORY-114=5705a10 MATCH, STORY-115=2e0eca2 MATCH.
- BC-2.16.010-PC2-SIGNATURE watch-item OPEN (expected resolution at STORY-114 delivery of 2-param new()).

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 7b7dbb2 | STORY-113 PR #239 merge commit |
| main HEAD | 3e29891 | v0.6.0 |
| open PRs | none | all delivered |

---

## Archived Checkpoint: 2026-06-15 — F4 ARP DELTA-IMPLEMENTATION IN PROGRESS; STORY-112 Step-4.5 CONVERGED + DEMO-RECORDED; NEXT = pr-manager (STORY-112 9-step PR)

**Archived from STATE.md on 2026-06-15 (replaced by "STORY-112 DELIVERED PR #238 10e4472; NEXT = STORY-113" checkpoint).**

### Pipeline Position

- Mode: FEATURE. Active feature: ARP security analyzer + etherparse 0.16→0.20. Issue #9. Release target: v0.7.0.
- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED 3/3 (Passes 31/32/33). F3 CONVERGED 3/3 (Passes 36/37/38). F3 human gate PASSED (D-070, 2026-06-14).
- F4 Delta-Implementation: IN PROGRESS — AUTHORIZED (D-070).
  - STORY-111 DELIVERED — PR #236 merged to develop (merge commit cced898; wave 40 COMPLETE; D-073).
  - STORY-112 Step-4.5 CONVERGED + DEMO-RECORDED — final HEAD c68964d on branch
    `worktree-issue-9-story-112-arp-extract-frame` (base cced898). 3/3 clean logic passes
    (frozen diff at 365dbeb); 1512 tests passed / 0 failed; rustfmt 1.9.0-stable (CI-matched).
    All 10 AC banners GREEN. 4 comment-only fix commits resolved non-blocking findings
    (F-1/F-2/F-3/Residual-F-1). VP-024 Sub-A Kani harnesses deferred to F6 (todo!()
    skeletons; verification_lock:false; D-062 precedent). STORY-112.md v1.4 committed
    (92797a2). input-hash: 8a4d566 (unchanged). Demo recordings in `.factory/demo-evidence/STORY-112/`.
    skipped_packets 73→69 (dns-remoteshell.pcap); 0 decode warnings (one-decode-error.pcap).
    NEXT = pr-manager (9-step PR).
  - STORY-113/114/115: NOT STARTED.

### VERIFIED SHAs at archival time

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | cced898 | before STORY-112 PR merge |
| main HEAD | 3e29891 | v0.6.0 |
| STORY-112 worktree HEAD | c68964d | final converged HEAD; Step-4.5 CONVERGED + DEMO-RECORDED |
| open PRs | STORY-112 PR pending | pr-manager 9-step dispatch pending |

---

## Checkpoint Archived 2026-06-15 — F4 wave-level adversarial GATE SATISFIED 3/3 CONVERGED; NEXT = F4 holdout evaluation

**Archived reason:** replaced by new checkpoint (F4 holdout GATE PASS; D-075/076/077 remediated; re-streak RESET 0/3 on 6abcd8f).

### A. PIPELINE POSITION AT ARCHIVAL

- F4 Delta-Implementation: COMPLETE (STORY-111..115 ALL DELIVERED; develop fee71ee post-PR #242).
- F4 Wave-Level Adversarial Convergence: GATE SATISFIED (3/3 CONVERGED, 2026-06-15).
  - Pass 1 NOT CLEAN (F-ARP-F4P1-001 MEDIUM threshold-0) → D-074 + PR #242 fee71ee; counter reset.
  - Clean-streak restart on fee71ee: Pass 1/3 CLEAN, Pass 2/3 CLEAN, Pass 3/3 CLEAN.
  - Final full-corpus consistency: CONSISTENT, zero gaps.
- develop HEAD: fee71ee == origin/develop (verified 2026-06-15).
- Decisions active: D-047..D-074.
- NEXT = F4 holdout evaluation (ARP wave-40-44 holdout scenarios).

### VERIFIED SHAs at archival time (2026-06-15)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | fee71ee | PR #242 D-074 merged |
| main HEAD | 3e29891 | v0.6.0 |
| open PRs | none | all ARP PRs + #237 chrono + #242 D-074 merged |

---

## Checkpoint Archived 2026-06-15 — F4 holdout GATE PASS; D-075/076/077 remediated; re-streak RESET 0/3 on 6abcd8f; re-streak IN PROGRESS

**Archived reason:** replaced by new checkpoint (F4 re-streak 3/3 CONVERGED on bcb1bd6 — F4 COMPLETE; NEXT = F5 scoped-adversarial).

### A. PIPELINE POSITION AT ARCHIVAL

- F4 Delta-Implementation: COMPLETE (STORY-111..115 ALL DELIVERED; develop fee71ee).
- F4 Holdout Evaluation: GATE PASS (2026-06-15).
  - Initial run mean 0.997 (G1=0.95: D1 HIGH verdict defect `Verdict::Possible` → `Verdict::Likely`).
  - D-075 issued; PR #243 (merge 4ee7a9d); G1 re-run = 1.0.
  - Full corpus 15/15 mean 1.0; RFC-826 canonical frame scenario PASS; non-D1 verdicts unregressed.
  - D-076 (PR #244 merge 52437f8): regression-test doc-comments corrected (PG-ARP-F4-REDTEST-DOC-TENSE recurrence).
  - D-077 CRITICAL (PR #245 merge 6abcd8f): `extract_arp_frame` now rejects non-Ethernet hw type + non-IPv4 proto type.
- F4 Post-Convergence Adversary Re-Streak: 0/3 IN PROGRESS (reset after D-077).
  - Prior 3/3 CONVERGED (fee71ee) invalidated by D-075 holdout catch + D-077 CRITICAL.
  - Re-streak restarted on 6abcd8f. Field-value + reject-path verification mandatory each pass.
- develop HEAD: 6abcd8f == origin/develop (verified 2026-06-15).
- Decisions active: D-047..D-077; do NOT re-adjudicate D-068/D-069/D-071/D-072/D-073/D-074/D-075/D-076/D-077.
- F3-OBL-STORY114-001/002/003 REVOKED (D-069).

### VERIFIED SHAs at archival time (2026-06-15)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 6abcd8f | PR #245 D-077 CRITICAL merged |
| main HEAD | 3e29891 | v0.6.0 |
| open PRs | none | — |

---

## Archived Checkpoint: 2026-06-16 — D-078/D-078b burst; F5 IN PROGRESS reset 0/3 on 2d2fadf; NEXT = F5 scoped-adversarial re-run

**Archived from STATE.md on:** 2026-06-16 (replaced by "F-1 VLAN-offset fix; F5 reset 0/3 on 079013d; NEXT = F5 scoped-adversarial re-run" checkpoint)

### POSITION

- **Epoch:** D-078 + D-078b burst complete. F5 streak RESET to 0/3 after code changes.
- **develop HEAD:** 2d2fadf == origin/develop (verified 2026-06-16).
- F1 PASSED (D-066). F2 CONVERGED 3/3. F3 CONVERGED 3/3 (D-070). F4 delivery COMPLETE (5 stories).
- D-078 (PR #247 92c1561): lax None arm bounds-checked-peeks raw 8-byte ARP fixed header; bad type/size → D11.
- D-078b (PR #248 2d2fadf): sibling lax Some(LaxNetSlice::Arp) arm → D11 path-independence; decoder.rs doc sweep.
- BC-2.16.009 v1.4→v1.6, BC-2.16.015 v1.3→v1.5. Input-hashes: STORY-112 8a4d566→8c03924, STORY-113 7c61bae→a05b724.
- F5 streak VOIDED by code change. Counter RESET to 0/3.
- Decisions active: D-047..D-078b. Do NOT re-adjudicate D-068..D-078b.
- ARP input-hash status: STORY-111..115 ALL MATCH post-D-078 recompute.

### NEXT (as of this checkpoint)

1. F5 scoped-adversarial re-run on 2d2fadf (`vsdd-factory:phase-f5-scoped-adversarial`).

### VERIFIED SHAs at archival time (2026-06-16)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 2d2fadf | PR #248 D-078b merged |
| main HEAD | 3e29891 | v0.6.0 |
| open PRs | none | — |

---

## Archived Checkpoint: 2026-06-15 — F4 COMPLETE; re-streak 3/3 CONVERGED on bcb1bd6; NEXT = F5 scoped-adversarial

**Archived from STATE.md on:** 2026-06-16 (replaced by "D-078/D-078b burst; F5 IN PROGRESS reset 0/3 on 2d2fadf" checkpoint)

### POSITION

- **Epoch:** F4 COMPLETE. Re-streak 3/3 CONVERGED on bcb1bd6.
- **develop HEAD:** bcb1bd6 == origin/develop (verified 2026-06-15).
- F1 PASSED (D-066). F2 CONVERGED 3/3. F3 CONVERGED 3/3 (D-070). F4 delivery COMPLETE (5 stories).
- D-074 PR #242 fee71ee. D-075 PR #243 4ee7a9d. D-076 PR #244 52437f8. D-077 PR #245 6abcd8f. O-1 PR #246 bcb1bd6.
- F4 holdout GATE PASS (15/15 mean 1.0; RFC-826 PASS).
- F4 wave-level adversarial re-streak 3/3 CONVERGED on bcb1bd6. GATE SATISFIED.
- Decisions active: D-047..D-077. Do NOT re-adjudicate D-068..D-077.
- ARP input-hash status (post-D-074): STORY-111..115 ALL MATCH.

### NEXT (as of this checkpoint)

1. F5 scoped-adversarial (`vsdd-factory:phase-f5-scoped-adversarial`).
2. F6 formal hardening (VP-024 Sub-A/B/D Kani harnesses; cargo-fuzz VP-008).
3. F7 delta convergence.
4. v0.7.0 release.

### VERIFIED SHAs at archival time (2026-06-15)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | bcb1bd6 | PR #246 O-1 rename-revert + VP-024 v1.9 |
| main HEAD | 3e29891 | v0.6.0 |

---

## Archived Checkpoint: 2026-06-16 — F-1 VLAN-offset fix; F5 reset 0/3 on 079013d; NEXT = F5 scoped-adversarial re-run

**Archived from STATE.md on:** 2026-06-16 (replaced by "F5 GATE SATISFIED 3/3; NEXT = F6 formal hardening" checkpoint)

### POSITION

- F5 IN PROGRESS — RESET to 0/3. develop HEAD 079013d (PR #249 F-1 VLAN-offset fix merged).
- D-078 (PR #247 92c1561): lax None arm bounds-checked-peeks raw 8-byte ARP header; bad type/size → D11. BC-2.16.009 v1.4→v1.6, BC-2.16.015 v1.3→v1.5.
- D-078b (PR #248 2d2fadf): lax Some(LaxNetSlice::Arp) arm → D11 path-independence. Streak VOIDED; reset to 0/3.
- F-1 MEDIUM (PR #249 079013d): D-078 fix hard-coded offset 14, ignoring lax.link_exts. Fix: arp_offset = 14 + lax.link_exts.iter().map(|ext| ext.header_len()).sum(). BC-2.16.009 v1.6→v1.7, BC-2.16.015 v1.5→v1.6. 4 new tests bc_2_16_d078_vlan_offset_tests.rs. Streak reset again to 0/3.
- All 5 ARP input-hashes MATCH post-F-1 recompute (STORY-112 292b3b8; STORY-113 3438b9d).

### NEXT (at archival time)

F5 scoped-adversarial re-run on 079013d. Counter 0/3; need 3 fresh-context CLEAN passes.

### VERIFIED SHAs at archival time (2026-06-16)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 079013d | PR #249 F-1 VLAN-offset fix |
| main HEAD | 3e29891 | v0.6.0 |
| open PRs | none | — |

---

## Archived Checkpoint: 2026-06-16 — F5 GATE SATISFIED 3/3 on 079013d; NEXT = F6 formal hardening

**Archived from STATE.md on:** 2026-06-16 (replaced by "F6 TARGETED HARDENING COMPLETE; NEXT = F7 delta convergence" checkpoint)

### POSITION

- **Epoch:** F5 GATE SATISFIED 3/3 (2026-06-16, develop 079013d). NEXT = F6 formal hardening.
- **develop HEAD:** 079013d == origin/develop (verified 2026-06-16).
- F1 PASSED (human-gated 2026-06-12, D-066). F2 CONVERGED 3/3. F3 CONVERGED 3/3 (D-070). F4 delivery COMPLETE (STORY-111..115; PRs #236..#246). F4 Holdout GATE PASS (15/15 mean 1.0). F4 Wave Adversary Re-Streak 3/3 GATE SATISFIED (bcb1bd6). F5 Scoped Adversarial: 3/3 GATE SATISFIED (079013d; implementation-robustness/security lens; F-1 VLAN-offset fix verified robust).
- Input-hash scan 2026-06-16 (post-F-1 recompute): STORY-111 d05149f MATCH | STORY-112 292b3b8 MATCH | STORY-113 3438b9d MATCH | STORY-114 1325d69 MATCH | STORY-115 bb1d83a MATCH.
- Decisions active: D-047..D-F1. F3-OBL-STORY114-001/002/003 REVOKED (D-069).
- Open follow-ups: O-2/FU-ARP-QINQ-MACSEC-TEST; FU-REPO-WIDE-DOC-DEBT; FU-JSON-CASING; FU-BC-2.10.007-MARKER; FU-STORM-NEW-ATTR; BC-2.10-COUNT-POSTMERGE.

### NEXT (at archival time)

F6 formal hardening: fill+prove 5 VP-024 Kani harness bodies; cargo-fuzz VP-008; cargo-mutants ARP delta; cargo-audit/clippy.

### VERIFIED SHAs at archival time (2026-06-16)

| Ref | Value | Notes |
|-----|-------|-------|
| develop HEAD | 079013d | PR #249 F-1 VLAN-offset fix |
| main HEAD | 3e29891 | v0.6.0 |
| factory-artifacts HEAD | 2bec42a | factory(f5→f6): F5 GATE SATISFIED 3/3 on 079013d; F6 entry |
| open PRs | none | — |

---

## Checkpoint: 2026-06-16 — F7 CONSISTENCY AUDIT REMEDIATED; NEXT = consistency-validator reconfirmation → v0.7.0 release gate

**Archived from STATE.md during F7 final closeout burst (F4 hash recompute + F7 CONVERGED record).**

### A. EXACT PIPELINE POSITION

- Project: wirerust. Mode: FEATURE. Active feature: ARP security analyzer + etherparse 0.16→0.20 migration. GitHub issue #9. Release target: v0.7.0.
- F1 PASSED (human-gated 2026-06-12, D-066).
- F2 CONVERGED 3/3 (Passes 31/32/33; strict whole-corpus gate SATISFIED).
- F3 CONVERGED 3/3 (Passes 36/37/38; 38 passes total; gate SATISFIED; D-070).
- F4 Delta-Implementation: COMPLETE. STORY-111..115 ALL DELIVERED; PRs #236..#246.
- F4 Holdout Evaluation: GATE PASS (15/15 mean 1.0; RFC-826 PASS).
- F4 Wave-Level Adversarial Re-Streak: 3/3 GATE SATISFIED (bcb1bd6).
- F5 Scoped Adversarial: GATE SATISFIED 3/3 (2026-06-16, develop 079013d).
- F6 Targeted Hardening: COMPLETE (PR #250, develop 6e9f2cc, 2026-06-16).
  - 5/5 VP-024 Kani harnesses VERIFICATION:- SUCCESSFUL (46/46 project-wide).
  - VP-024 v2.2 LOCKED; verified_at_commit=6e9f2cc; Sub-D surrogate = insert_binding_lru_array.
  - Fuzz VP-008: 16.2M execs / 0 crashes. Mutants ARP delta: 98.9% kill (1 benign MISSED by design).
  - Security: cargo-audit 1 allowed RUSTSEC-2026-0097 (transitive BUILD-dep); clippy+fmt CLEAN.
- F7 Consistency Audit: ALL 4 FINDINGS REMEDIATED (2026-06-16 — doc/spec only, zero code change).
  - F1 MAJOR: VP-024 coverage-matrix v1.7 — status "verified" (was "draft").
  - F2 MAJOR: Sub-D surrogate rename arp-architecture-delta v1.17 + VP-024 v2.2 + STORY-113 v1.2.
  - F3 MINOR: BC-2.16.008 v1.9/012 v1.4/013 v1.4 + STORY-114 v1.4/115 v1.4 (AC + EC-table fix).
  - F4 MINOR: Input-hashes recomputed (111=3eefa35 112=6115929 113=4069c39 114=02da9e7 115=80be67e ALL MATCH at time of archive; re-drifted by VP-024 v2.3+arp-arch-delta v1.17 in this burst).
  - F7 holistic adversary: PASS CLEAN (zero findings, 6 dimensions CONVERGED).
  - Regression: GREEN (develop e37ec38: build/clippy/fmt clean, 1592 tests/0 fail).
- develop HEAD: e37ec38 (no code change — F7 consistency fixes are doc/spec only).
- Decisions active: D-047..D-F1; do NOT re-adjudicate D-068..D-F1.
- F3-OBL-STORY114-001/002/003 REVOKED (D-069).

### B. INPUT-HASH STATUS (at archive time — pre-final-recompute)

STORY-111=3eefa35 MATCH | STORY-112=6115929 MATCH (stale re-drift caught in closeout burst) | STORY-113=4069c39 MATCH (stale re-drift caught in closeout burst) | STORY-114=02da9e7 MATCH | STORY-115=80be67e MATCH.

### C. SHA TABLE (at archive time)

| Item | SHA | Notes |
|------|-----|-------|
| develop HEAD | e37ec38 | No code change — all doc/spec |
| main HEAD | 3e29891 | v0.6.0 |
| factory-artifacts HEAD | see git -C .factory log -1 | — |
| open PRs | none | — |

---

## Archived Checkpoint: 2026-06-17 — E-17 F3 story decomposition freeze; NEXT = E17-F3 adversarial Pass 1

**Archived from STATE.md on:** 2026-06-17 (replaced by "E-17 F3 adversarial P1 CLEAN; clean-streak 1/3" checkpoint)

### A. EXACT PIPELINE POSITION (at archive time)

- E-17 F1: PASSED (human-gated 2026-06-16). MACsec investigated → NO code bug; documented-limitation; full F1-F7 rigor; v0.7.1 target.
- E-17 F2: COMPLETE — adversarial gate SATISFIED 3/3 (2026-06-16) on frozen baseline factory-artifacts 39f57ea. Final corpus: BC-2.16.009 v1.9, BC-2.16.015 v1.8, arp-architecture-delta v1.19, VP-024 v2.4, verification-coverage-matrix v1.8.
- E-17 F3: COMPLETE (frozen) — STORY-116 (wave 45, QinQ coverage) + STORY-117 (wave 46, MACsec documented-limitation); linear chain STORY-115→116→117; input-hash c389b39 both MATCH; BC-2.16.009 v1.10 + BC-2.16.015 v1.9 backlinks added; epics/dependency-graph/wave-schedule updated. F3 human gate REQUIRED before F4.
- E-17 F3 NEXT: F3 adversarial convergence + consistency-validator → F3 human gate → F4 authorized.
- develop HEAD: 480f8ae == origin/develop.
- main HEAD: dd8e142 (v0.7.0).
- factory-artifacts HEAD: see git -C .factory log -1 --format='%h %s'
- Active worktrees: EXACTLY 2 — main repo (develop) + .factory (factory-artifacts).
- Open PRs: None.

### B. BURST RECORDED

- STORY-116 (QinQ coverage, wave 45) + STORY-117 (MACsec documented-limitation, wave 46) created; linear chain STORY-115→116→117.
- input-hash c389b39 computed and MATCH for both stories.
- BC-2.16.009 v1.10 + BC-2.16.015 v1.9: Traceability backlinks added for STORY-116/STORY-117.
- epics.md, dependency-graph.md (cross_epic_edges 18→19, total_stories 67→69), wave-schedule updated (waves 44→46).
- Drift items added: DRIFT-E16-EPICS-SUMMARY-GAP-001 + DRIFT-E16-BC-BACKLINK-GAP-001 (pre-existing E-16 traceability gaps; DEFERRED LOW).
- spec-changelog.md updated with F3 entry.
- E-17 phase pointer advanced: F3 COMPLETE (frozen) → F3 adversarial convergence.

---

## Archived Checkpoint: 2026-06-17 — E-17 F3 streak RESET 0/3 integrity correction

**Archived from STATE.md on:** 2026-06-17 (replaced by "E-17 F3 adversarial gate SATISFIED 3/3 genuine" checkpoint)

### POSITION

- Project: wirerust. Mode: STEADY-STATE (top-level pipeline IDLE; E-17 sub-cycle IN PROGRESS).
- E-17 F3: IN PROGRESS — adversarial convergence. Corpus frozen at factory-artifacts 5dff3cb. Prior "Pass 1 CLEAN / streak 1/3" record (ae977cb) VOIDED — no fresh-context adversary produced that verdict; real adversary hung (a9f139ef). Clean-streak 0/3 (restarting).
- E-17 F3 NEXT: Pass 1 (real fresh-context adversary; streak attempt 1/3).
- develop HEAD: 480f8ae == origin/develop.
- main HEAD: dd8e142 (v0.7.0).

### B. INTEGRITY CORRECTION

- Prior Pass-1 CLEAN record (ae977cb) VOIDED: no fresh-context adversary actually produced that verdict. Streak reset: 1/3 → 0/3.

---

## Archived Checkpoint: 2026-06-17 — E-17 F3 adversarial gate SATISFIED 3/3 genuine

**Archived from STATE.md on:** 2026-06-17 (replaced by "E-17 F4 wave-adversarial gate SATISFIED 3/3" checkpoint)

### POSITION

- Project: wirerust. Mode: STEADY-STATE (top-level pipeline IDLE; E-17 sub-cycle IN PROGRESS).
- E-17 F1: PASSED (human-gated 2026-06-16).
- E-17 F2: COMPLETE — adversarial gate SATISFIED 3/3 (2026-06-16).
- E-17 F3: ADVERSARIAL GATE SATISFIED 3/3 (genuine, dd34205) — 3 verified fresh-context CLEAN passes (aeddd3a4/aa09cc4e/ab72e18d); each zero MEDIUM+; input-hash discharged (c389b39 MATCH); consistency-validator round-1 remediated; residual LOWs non-blocking. AWAITING F3 human gate.
- E-17 F3 NEXT: F3 human gate (approval of STORY-116/117 decomposition). On PASSED → F4 authorized (STORY-116 first, wave 45).
- develop HEAD: 480f8ae == origin/develop.
- main HEAD: dd8e142 (v0.7.0).
- Open PRs: None.
- Real E-17 F3 adversarial streak must restart from Pass 1 with a genuine fresh-context adversary dispatch.
