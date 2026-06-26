---
document_type: decisions-archive
cycle_id: feature-enip-v0.11.0
archived_from: STATE.md Decisions Log
archived_at: ~
archived_decisions: D-228..
---

# Decisions Archive — feature-enip-v0.11.0 (D-228+)

*Active cycle decisions are recorded here as they are archived from STATE.md.*

---

## D-228 — Feature Mode Opened; F1 Delta Analysis PASSED; Human-Approved Scope (2026-06-24)

Pipeline transitions QUIESCED → FEATURE-MODE for GitHub issue #316 (EtherNet/IP + CIP ICS analyzer, SS-17). F1 delta analysis complete at `.factory/feature-f1-delta-analysis/enip-delta-analysis.md`. Human-approved scope at F1 gate:

- **In scope for v0.11.0:** TCP/44818 explicit messaging + UDP/2222 cyclic (implicit) I/O + CIP ForwardOpen connection-lifecycle tracking.
- **Deferred:** TLS/2221 encrypted channel.
- **Carry-buffer cap:** 600 bytes per flow.

Planned spec delta: new subsystem SS-17 (CAP-17), new analyzer `src/analyzer/enip.rs`, ADR-010, VP-032, ~24+ BCs (BC-2.17.xxx), 7-9 stories. DTU NOT required (passive parser).

Research inputs registered:
- `.factory/research/next-ics-protocol-prevalence.md` — protocol selection rationale (EtherNet/IP #1).
- `.factory/research/enip-mitre-ics-tagging.md` — MITRE ATT&CK for ICS v19.1 technique mapping.

MITRE key findings for F2 carry-forward: CIP Stop → T0858, CIP Reset → T0816, CIP firmware → T1693.001 (T0857 REVOKED), identity read → T0888/T0846, SetAttribute/write → T0836, UDP/2222 I/O abuse → T1692.001/.002, ForwardOpen → no dedicated technique (document gap in ADR-010). Do NOT seed T0855/T0856/T0857 (revoked). Open design item: T0858 and T1693.001 carry multi-tactic pairings the single-tactic `MitreTactic` enum does not currently model — VP-007 atomic-obligation decision needed in F2.

Ground truth at open: develop=ff4b82b, main=0cbe922 (v0.10.0). Next: F2 Spec Evolution.

---

## D-229 — F2 Scope Refinement: UDP/2222 Deferred to v0.12.0 (2026-06-24)

At the F2 architecture review, the architect found that UDP/2222 cyclic I/O requires UDP-reassembly infrastructure plus cross-transport ForwardOpen session-correlation that is not present in wirerust (wirerust is TCP-stream-oriented in dispatch). Human approved deferring UDP/2222 to a follow-on cycle (v0.12.0).

**v0.11.0 scope (revised):** TCP/44818 explicit messaging + CIP ForwardOpen connection-lifecycle detection (over TCP only). No T1692.001/.002 (UDP cyclic I/O abuse) BCs written this cycle.

**v0.12.0 backlog:** UDP/2222 cyclic I/O + cross-transport ForwardOpen session-correlation + T1692.001/.002 detection.

ADR-010 Decision 5 documents the deferral rationale. 24 BCs authored (BC-2.17.001..024) covering TCP/44818 path; no SS-17 UDP BCs exist. BC-INDEX v1.74 (329 total / 328 active). OA-001 open: `--enip-write-burst-threshold` default (20/1s) awaiting human confirm at F2 gate.

---

## D-230 — F2 Human Gate APPROVED; F2 Addendum BC-2.17.026 (--enip-error-burst-threshold) (2026-06-24)

F2 human gate PASSED. Human decisions recorded:

1. **Proceed to F3** — F2 adversarial convergence (4 consecutive 0-H/C passes P10-P13) accepted; consistency audit complete; addendum scoped re-validation pending before F3 entry.
2. **0x00B2-only CIP detection scope accepted** — 0x00B1 connected-item detection remains DEFERRED to v0.12.0 (ADR-010 Decision 8). v0.11.0 detects CIP request operations on 0x00B2 unconnected carriers only.
3. **Both detection thresholds accepted as tunable defaults** — write-burst default=50 (`--enip-write-burst-threshold`, BC-2.17.023) and error-burst default=5 (`--enip-error-burst-threshold`, BC-2.17.026 NEW). Both require `--enip`/`--all` to activate. Neither is an absolute hard-stop; operators may recalibrate via CLI flag.
4. **Recalibrate F6** — the addition of two tunable CLI flags means F6 targeted hardening should include boundary / off-by-one testing for both threshold paths. F6 scope note recorded here for orchestrator.

**F2 addendum committed (feature-enip-v0.11.0, factory-artifacts):**

- `BC-2.17.026` CREATED — `--enip-error-burst-threshold` CLI flag configures T0888 error-burst detection sensitivity; u32, default 5, strict `>` semantics, symmetric with BC-2.17.023 write-burst flag.
- `ADR-010` Decision 9 added — flag spec: `--enip-error-burst-threshold <N>` (u32, default 5); `EnipAnalyzer` gains `enip_error_burst_threshold: u32` field initialised from CLI arg; `ENIP_ERROR_BURST_THRESHOLD` compile-time constant RETIRED in favour of the instance field.
- `BC-2.17.014` updated — replaced hardcoded `ENIP_ERROR_BURST_THRESHOLD` constant reference with configurable `self.enip_error_burst_threshold` field; added BC-2.17.026 cross-reference.
- `BC-2.17.020` updated — added `--enip-error-burst-threshold` to CLI surface (three ENIP flags: `--enip`, `--enip-write-burst-threshold`, `--enip-error-burst-threshold`); added BC-2.17.026 to Related BCs.
- `BC-INDEX` v1.75→v1.76 — SS-17: 25→26 BCs; total on disk: 330→331; active: 329→330.
- `ARCH-INDEX` — SS-17 row updated to BC count 26.
- `prd.md` — §2.17 section header updated; §7 RTM BC-2.17.026 row added.
- `cap-17-enip-cip-analysis.md` — BC-2.17.026 registered.

**Ground truth at D-230:** develop=ff4b82b, main=0cbe922 (v0.10.0), factory-artifacts=this commit.

---

## D-231 — F3 CONVERGED + HUMAN-APPROVED; Wave-by-Wave F4 Cadence (2026-06-24)

F3 adversarial story decomposition converged after 12 passes (3 consecutive 0-H/0-C clean passes: P10/P11/P12). Trajectory: 4C/6H→1C/3H→0C/2H→2C/2H→0C/1H→0C/1H→0C/0H→0C/1H→0C/2H→0C/0H→0C/0H→0C/0H. Consistency audit CONSISTENT (`.factory/phase-f3-stories/enip-f3-consistency-audit-final.md`). Human gate APPROVED.

**F3 deliverables:**
- 9 stories: STORY-130..138 (epic E-20, waves 58-61, 66 pts). All 26 BC-2.17.001..026 assigned.
- 13 holdout scenarios: HS-110..122 (all must-pass; 12 require pcap fixtures; HS-121 synthetic).
- Adversarial pass files: `.factory/phase-f3-stories/enip-f3-adversary-pass-1..12.md`.
- Final consistency audit: `.factory/phase-f3-stories/enip-f3-consistency-audit-final.md`.
- STORY-INDEX.md v2.8 (91 stories / 61 waves). epics.md v1.8 (E-20). BC-INDEX v1.79 (331/330 active; SS-17=26).

**Human gate decisions (D-231):**
1. Proceed to F4 TDD Implementation — APPROVED.
2. F4 cadence: wave-by-wave with human checkpoints at each wave gate (report at waves 58/59/60/61).
3. Deferred LOW (non-blocking, carry to F4): dep-graph STORY-133→137 T0814 rationale prose imprecision; BC-2.17.010 "per-occurrence" PO BC fix; BC frontmatter input-hash:TBD (F4 obligation); STORY-133 EMITTED/SEEDED baseline reverify vs src/mitre.rs at F4.
4. 12 pcap fixtures needed for holdouts HS-110..122 minus HS-121 — F4 obligation.

Wave 58 (STORY-130 + STORY-131) STARTING at D-231.

---

## D-232 — SAFE-TO-CLEAR Checkpoint; F4 Wave 58 STORY-130 mid-TDD (2026-06-25)

Session paused mid-F4 Wave 58 with STORY-130 at Red Gate. All F1/F2/F3 pipeline artifacts are durable on factory-artifacts branch. This checkpoint makes the session SAFE TO CLEAR.

**Exact pause state:**
- Cycle: `feature-enip-v0.11.0` — EtherNet/IP + CIP ICS analyzer (SS-17, issue #316). Target v0.11.0.
- Phase: F4 (TDD Implementation), wave-by-wave cadence (D-231), Wave 58 in progress.
- STORY-130 worktree: `.worktrees/STORY-130-enip-pure-core-parse`, branch `worktree-issue-316-story-130-enip-pure-core-parse`, base develop `ff4b82b`.
- Red Gate commit: `1f9c656` (`enip.rs` stubs + tests; `cargo check`/`clippy` GREEN; 14 tests FAIL as expected).
- **A test-writer was IN-FLIGHT** authoring `tests/enip_analyzer_tests.rs` (mod `parse_header`, BC-2.17.001-004) when the session was paused. A `test(enip): STORY-130 ... failing tests` commit may or may not have landed.

**Ground-truth HEADs at D-232:**
- develop: `ff4b82b` (unchanged this cycle — all spec/story work is on factory-artifacts).
- main: `0cbe922` (v0.10.0).
- factory-artifacts: this D-232 checkpoint commit (verify: `git -C .factory log -1`).

**Resume instruction (abbreviated — full RESUME PROCEDURE in STATE.md):**
1. `vsdd-factory:factory-worktree-health` (BLOCKING).
2. Read STATE.md + cycle-manifest fully.
3. Check STORY-130 worktree log: `git -C .worktrees/STORY-130-enip-pure-core-parse log --oneline -5`.
   - If test commit present → dispatch implementer.
   - If not → dispatch test-writer first.
4. Continue STORY-130: implementer → adversarial convergence (3 clean passes) → demo → push → pr-manager (9-step) → worktree cleanup.
5. Then STORY-131 → Wave-58 gate → REPORT TO HUMAN.

**Remaining F4 work:**
- Wave 58: STORY-130 (resume) + STORY-131 (dispatch Rule 7 + CLI flags, BC-2.17.019/020/023/026).
- Wave 59: STORY-132 (CPF/CIP parse + VP-032 Sub-D), STORY-133 (MITRE seeding + VP-007 6-part atomic burst — mitre.rs/SS-10: add IcsExecution MitreTactic variant; seed T0858/T0816/T1693.001; EMITTED_IDS += T0858/T0816/T0846; vp007 drift-guard).
- Wave 60: STORY-134/135/136/137. Wave 61: STORY-138.

**F4 carry-forward obligations:**
- 12 pcap fixtures for holdouts HS-110..122 (minus HS-121 synthetic).
- STORY-133: re-verify EMITTED 17→20 / SEEDED 25→28 baselines vs live `src/mitre.rs` HEAD (post-STORY-129) before asserting counts.
- VP-007 atomic burst (STORY-133): add IcsExecution MitreTactic variant; seed T0858/T0816/T1693.001; EMITTED_IDS += T0858/T0816/T0846; vp007 drift-guard.
- F6 fuzz obligation: `parse_cip_header` + `parse_cpf_items` cargo-fuzz (F-P9-002).
- Deferred LOW: BC-2.17.010 Description "per-occurrence" → fix to one-shot (PO); dep-graph STORY-133→137 T0814 rationale prose imprecision.
- `docs/adr/0010-*.md` uncommitted on develop working tree → commit with F4 code (STORY-131 or first ENIP code PR).

**Pre-existing backlog (non-blocking):** Dependabot #311; PO-BACKLOG-MAINT holdout coverage; engine-improvement backlog incl. PROPAGATION-LAG-001 + ledger-claim-grep process-gap.

---

## D-233 — STORY-130 mid-TDD adversarial convergence in progress (2026-06-25)

Code green at `42de2d0` (21/21 tests, clippy/fmt clean, VP-032 Sub-A/B/C Kani harnesses preserved). Pass 1 = 1 HIGH (DF-GREEN-DOC-TENSE, fixed @42de2d0 in develop worktree, no factory-artifacts impact). Pass 2 = PASS: 0 HIGH/CRITICAL, 1 MEDIUM F-130-P2-001 (BC-2.17.002→v1.1 field-count 10→6 + ADR-010 §Decision 8 "6 fields" fix; STORY-130 input-hash dc8a2c9→272738c; BC-INDEX v1.79→v1.80). Convergence counter: 1 clean pass (Pass 2); need 2 more consecutive clean passes per BC-5.39.001. NEXT = adversarial Pass 3.

---

## D-234 — STORY-130 per-story delivery COMPLETE (2026-06-25)

Adversarial convergence ACHIEVED: 3 consecutive clean passes (Pass 2/3/4, 0 HIGH/CRITICAL, BC-5.39.001 MET). Pass 1: 1 HIGH (DF-GREEN-DOC-TENSE, fixed). Pass 2: 1 MEDIUM (BC-2.17.002/ADR-010 field-count 10→6, fixed). Pass 4: 1 LOW (AC-130-001 postcondition citation precision "1-9" vs "1-8") — non-blocking, logged. Code: 21/21 tests green, clippy/fmt clean, VP-032 Sub-A/B/C Kani harnesses preserved. SEC-002 latent-panic hardening applied (try_into().expect() → byte-literal array). Demo evidence: docs/demo-evidence/STORY-130/. ADR-0010 (F4 obligation) shipped. Merged via PR #317, develop HEAD now 235ae60. Worktree cleaned up. NEXT = STORY-131.

---

## D-235 — STORY-131/132 on_data boundary decision (2026-06-25)

STORY-131 implements minimal `EnipAnalyzer::on_data` (bytes_received counter only) + dispatcher Rule 7 + CLI flags + reassembly guard. CIP frame-walk/CPF/findings/VP-032 Sub-D deferred to STORY-132 (Wave 59). Rationale: PC-2 wiring guarantee requires non-panicking on_data (DNP3 precedent); white-box classify() tests alone insufficient for BC-2.17.019 PC-2. bytes_received counter stable across STORY-131→132. Boundary doc: cycles/feature-enip-v0.11.0/story-131-132-ondata-boundary.md. STORY-131 Pass-1 adversarial: 1 HIGH DF-GREEN-DOC-TENSE (dispatch test docs — fixed @5e61682) + 2 MEDIUM (M1 STORY-131.md EC-007 overload fixed, M2 BC-INDEX BC-2.17.020 title sync v1.80→v1.81 fixed). Code green @5e61682 (15/15 dispatch + 21/21 parse, clippy/fmt clean, VP-004 oracle 44818 arm present). Convergence in progress: Pass 2 clean; Pass 3 running; need 3 consecutive clean passes (BC-5.39.001).

---

## D-236 — STORY-131 adversarial Pass 3 PASS with fixes (2026-06-25)

Pass 3 = PASS (0 HIGH/CRITICAL) with 1 MEDIUM [process-gap] M-1 (false warn!/log requirement: ADR-010 Decision 9 root + STORY-131 + STORY-138 propagation — all fixed to eprintln!/no-log-crate convention) + 2 LOW (L-1 BC-2.17.023/026 Precondition "N≥1" vs 0-accepted — fixed to 0..=u32::MAX v1.0→v1.1; L-2 dispatcher.rs module-doc ENIP omission — fixed @0018a54). Code green @0018a54 (15/15 dispatch + 21/21 parse, clippy/fmt clean). BC-INDEX v1.81→v1.82. STORY-131 input-hash 6d892c4→a119157. M-1 codified as [codified] WARN-LOG-CRATE-001 in cycles/feature-enip-v0.11.0/lessons.md. STORY-132..138 remain STALE (pending F4 per-story refresh).

---

## D-237 — Wave-58 (STORY-130+131) delivered+merged (2026-06-25)

develop@edce3bd; regression PASS (1955 tests green, clippy/fmt/release clean, ENIP surface present). Per-story convergence 3/3 each. Consistency-audit H-001 FIXED (STORY-130 input-hash 272738c→e3c0a6a — D-236 ADR-010 Decision-9 eprintln! change was declared input; only STORY-131 hash refreshed in D-236 burst). Consistency-audit L-001 FIXED (STORY-INDEX.md STORY-130/131 status draft→completed; Wave-58 delivery-progress row draft→DELIVERED & CLOSED). M-001 OUTSTANDING: deferred to STORY-132 PR obligation. stories_delivered: 79→80.

---

## D-238 — Wave-58 wave-level adversarial convergence ACHIEVED (2026-06-25)

3 consecutive clean passes (W58-P1/P2/P3, all 0 HIGH/CRITICAL, BC-5.39.001 MET) reviewing integrated develop@edce3bd. Integration verified: STORY-130 parse ↔ STORY-131 dispatch seam coherent; 5-arg StreamDispatcher::new ripple complete; both exhaustive DispatchTarget matches (on_data, on_flow_close) + classify_oracle updated with Enip arm; sibling routing (HTTP/TLS/Modbus/DNP3) unaffected; reporter take_enip_analyzer integration symmetric with DNP3; early-exit guard includes self.enip.is_none(). Wave 58 FULLY CLOSED. STORY-132 obligations logged: M-001 (docs/adr/0010 sync), WAVE59-E2E-001 (combined e2e test), WAVE59-DEADCODE-001 (#![allow(dead_code)] removal). Wave 59 pending human go-ahead (D-231 cadence).

---

## D-239 — STORY-132 per-story delivery COMPLETE (2026-06-25)

Adversarial convergence ACHIEVED: 3/3 (Pass 2/3/4 clean, 0 HIGH/CRITICAL). Pass 1: 1 HIGH (DF-GREEN-DOC-TENSE test-module header — fixed; codified GREEN-DOC-TENSE-TEST-HEADER-001 in lessons.md). Pass 3: 1 LOW (Vec::with_capacity amplification factor → capped). Pass 4: 1 LOW (test PC citations). BCs: BC-2.17.005/006/007/009 (CPF item walk + CIP header parse + path extraction). VP-032 Sub-D Kani harnesses present (run at F6). F-P9-002 fuzz obligation doc comments present (harnesses deferred to F6). 19 cpf_cip tests green. M-001 RESOLVED: docs/adr/0010-ethernet-ip-cip-stream-dispatch.md synced to .factory ADR-010 (field count + eprintln! guard). Merged via PR #319, develop HEAD now 16d3ce7. stories_delivered: 80→81. WAVE59-E2E-001 + WAVE59-DEADCODE-001 re-targeted to STORY-137 (BC-2.17.016 frame-walk — STORY-132 adds pure parse fns only). Process-gap codified: GREEN-DOC-TENSE-TEST-HEADER-001.

---

## D-240 — STORY-133 adversarial Pass-1 REMEDIATED (2026-06-25)

Pass-1: 2 CRITICAL + 2 HIGH. Root cause: STORY-133 prose carried wrong MITRE catalog mapping for T1693.001 — name was "Exploit Public-Facing Application: EtherNet/IP" (Enterprise technique, wrong) vs ADR-010 Decision 7 authoritative "Modify Firmware: System Firmware"; tactic was IcsInitialAccess (wrong) vs IcsInhibitResponseFunction/TA0107 (correct). ALL FIXED at code commit `ffca717` (impl + test pin + mitre_tests.rs authoritative-TA-id pin-table extended with T1693.001→TA0107 + stale-count fn renames + RED-tense scrub). Story prose corrected: 4 T1693.001 references corrected. STORY-133 input-hash UNAFFECTED. VP-007 invariants intact. Codified as MITRE-CATALOG-ADR-AUTHORITATIVE-001 in cycles/feature-enip-v0.11.0/lessons.md.

---

## D-241 — STORY-133 per-story delivery COMPLETE (2026-06-25)

Adversarial convergence ACHIEVED: 3/3 (Pass 2/3/4 clean). Pass 1: 2 CRITICAL + 2 HIGH (T1693.001 wrong name/tactic — all fixed, D-240). VP-007 6-step atomic burst SATISFIED: T0858/T0816/T1693.001 seeded; SEEDED 25→28; EMITTED_IDS 17→20 (T0858/T0816/T0846); IcsExecution variant added (Display "Execution (ICS)", tactic_id "TA0104"); `cargo test mitre` all 10 mitre_seeding tests green. Wave-59 regression PASS (1984 tests). Merged via PR #320, develop HEAD now 7f040de. stories_delivered: 81→82.

---

## D-242 — Wave 59 FULLY CONVERGED & CLOSED (2026-06-25)

STORY-132+133 merged (PR #319/#320), regression PASS on develop d562ccc. Per-story adversarial convergence 3/3 each. Wave-level adversarial convergence 3/3 (confirmation passes D/E/F all 0 HIGH/CRITICAL on develop d562ccc). Remediation history: C-1 (T0846 stale write_burst_emitted guard cross-story regression — fixed PR #321 + green-doc-tense CI gate); F-WAVE59-C-001/M-2 (stale cross-story count-snapshot prose + RED-tense test comments — fixed PR #322); F-W59-M01 (BC-2.17.012 TA-id wrong TA0105→TA0106 — fixed in factory-artifacts burst: BC-2.17.012 v1.0→v1.1, BC-INDEX v1.82→v1.83). Full SS-17 detection-BC MITRE-tuple audit: BC-2.17.010/011/013/014/018 all correct; only 012 was wrong. Follow-ups logged: WAVE-60-TEST-DOC-SWEEP, GREEN-DOC-TENSE-GATE-COVERAGE-001.

---

## D-243 — STORY-134 Green Gate reached (2026-06-25)

process_pdu + EnipFlowState implemented at worktree `worktree-issue-316-story-134-enip-recon`; HEAD f54b9dc. 18/18 recon tests pass; full repo green; clippy/fmt/green-doc-tense clean. Implements: T0846 ListIdentity one-shot, T0888 Pattern A (Identity-read per-occurrence) + Pattern B (error-burst >threshold one-shot), CIP error-window accumulation (10s), is_non_enip suppression gate. Scope confirmed: STORY-134 owns process_pdu + EnipFlowState; command_counts NOT touched; on_data NOT wired (STORY-137 owns frame-walk per BC-2.17.016). BCs covered: BC-2.17.008/010/014. Red Gate @5845ff6 (stubs @25e751e; 18 failing recon tests).

---

## D-244 — STORY-134 adversarial convergence Pass-3/4 spec fixes APPLIED (2026-06-25)

Pass-3 found 2 HIGH spec contradictions (F-134-P3-001: BC-2.17.010 pseudo-code commanded command_counts increment — contradicts F8-001; F-134-P3-002: same). Pass-4 found 1 MEDIUM (M-1: BC-2.17.008 PC-2 used `error_window_start_ts==0` as unseeded sentinel — fails at ts=0). ALL resolved via SPEC corrections; code @ac04edd was already correct. BC-2.17.010 v1.0→v1.1: F8-001 amendment applied — PC-1 command_counts increment removed from process_pdu; reattributed to BC-2.17.016 frame-walk (on_data PC-0); Architecture Anchor updated; PC-3 corrected. F8-001 now fully propagated — BC-2.17.010 was the last unamended SS-17 BC. BC-2.17.008 v1.1→v1.2: PC-2 sentinel fix — replaced `error_window_start_ts==0` with `flow.error_window_active == false`; PC-4 guard updated; EC-008 added; Architecture Anchors updated. ADR-010 Decision 4 EnipFlowState roster: `error_window_active: bool` field + doc-comment added. STORY-134.md: AC-134-001/002 + Architecture Mapping + Tasks aligned. STORY-134 input-hash 604b9de→16d03a6. BC-INDEX v1.83→v1.84. Convergence reset: 3 clean passes needed. Lesson codified: F8-001-PROPAGATION-COMPLETENESS.

---

## D-245 — STORY-134 convergence Pass-G ADR-decision citation fix (2026-06-25)

Pass-G adversary found 2 MEDIUM mis-anchors (F-134-PG-001/002): enip.rs + STORY-134.md cited ADR-010 Decision 6 for detection-order and Decision 5 for MAX_FINDINGS, but Decision 4 = "EnipFlowState design and frame-walk algorithm" (owns both); Decision 5 = ForwardOpen; Decision 6 = UDP/2222-deferred. FIXED: enip.rs @0115bf5 (worktree, 8 sites), STORY-134.md (3 sites, this factory-artifacts commit). Passes H/I were clean; re-confirmation round (J/K/L) running on worktree HEAD 0115bf5. Lesson appended: ADR-DECISION-NUMBER-MIS-ANCHOR-001 in cycles/feature-enip-v0.11.0/lessons.md.

---

## D-246 — STORY-134 per-story adversarial convergence ACHIEVED (2026-06-25)

3 consecutive clean passes M/N/O (all 0 HIGH/CRITICAL, BC-5.39.001 MET) on worktree HEAD 68e3394. Full remediation trajectory: Pass-1 HIGH ts=0 error-window sentinel (fixed via `error_window_active: bool`); Pass-3 2×HIGH F8-001 BC-2.17.010 command_counts-in-process_pdu spec contradiction + M-1 BC-2.17.008 ==0 sentinel (BC-2.17.010 v1.1 + BC-2.17.008 v1.2 + ADR-010 Decision 4 roster); Pass-G/J/K/L MEDIUM ADR-decision mis-citations Decision 5/6→4 swept full worktree (src/analyzer/enip.rs + tests/enip_analyzer_tests.rs + STORY-134.md). 20 recon tests green. Follow-ups: (a) STORY-134.md AC narrative cites `flow_key` param vs actual signature `(flow: &mut EnipFlowState, pdu, timestamp, src_ip)` — prose alignment LOW non-blocking; (b) redundant `service & 0x80 == 0` re-check in enip.rs Pattern A path — harmless/optional cleanup LOW. Both added to WAVE-60-TEST-DOC-SWEEP.

---

## D-247 — STORY-134 per-story delivery COMPLETE (2026-06-25)

Demo recorded. PR #323 MERGED to develop `e330ccc` (merge-commit strategy). stories_delivered: 82→83. 20 recon tests green (T0846 ListIdentity one-shot, T0888 Pattern A Identity-read per-occurrence + Pattern B error-burst one-shot; BC-2.17.008/010/014). 1 LOW SEC-001 saturating_add fixed in PR (commit `652fcff`). CI 11/11 green incl green-doc-tense gate. Per-story adversarial convergence 3/3 (passes M/N/O, 0 HIGH/CRITICAL). Wave 60 progress: STORY-134 DONE; STORY-135/136/137 remain.

---

## D-248 — STORY-135 per-story adversarial convergence ACHIEVED (2026-06-25)

Passes 5/6/7, all 0 HIGH/CRITICAL, BC-5.39.001 MET on worktree HEAD 5963ca4. T0858/T0816/T0836 command detections (BC-2.17.011/012/013); 16 command_detections tests. Detection logic confirmed correct throughout. Multi-round remediation (doc/test completeness only): Pass-1 doc-prose + green-doc-tense gate coverage hole (gate strengthened patterns 12-18); Pass-2/3/4 F-135-P2-001 test verdict/confidence/verbatim-summary assertion gap (tests now pin normative BC strings) + stale "before reaching todo!()" prose (gate patterns 19-22) + EC-007 threshold-0 test added (16 tests); Pass-5/6/7 LOW doc cleanups (test-count comment 15→16, BC-table titles verbatim, AC-135-002 trace). green-doc-tense gate now 22 patterns / self-test 54 cases. GREEN-DOC-TENSE-GATE-COVERAGE-001 RESOLVED.

---

## D-249 — STORY-135 per-story delivery COMPLETE (2026-06-26)

Demo recorded. PR #324 MERGED to develop `84be2fb` (merge-commit strategy). stories_delivered: 83→84. 16 command_detections tests green (T0858 ListCommand one-shot, T0816 SetAttributeSingle one-shot, T0836 GetAttributeSingle one-shot; BC-2.17.011/012/013). Per-story adversarial convergence 3/3 (passes 5/6/7, 0 HIGH/CRITICAL). Green-doc-tense gate strengthened (patterns 12-22, self-test 54) now on develop. GREEN-DOC-TENSE-GATE-COVERAGE-001 RESOLVED. PR also shipped STORY-134/131 sibling stale-todo prose scrubs. CI 11/11 green incl green-doc-tense gate. Wave 60 progress: STORY-134+135 DONE; STORY-136/137 remain. Residual: mitre.rs:358 stale BC-2.17.012 label on T0816 (cross-story cleanup, non-blocking; batch Wave-60 doc sweep). NEXT = STORY-136.

---

## D-251 — SESSION PAUSE: STORY-136 per-story adversarial convergence ACHIEVED @b003547 (2026-06-26)

Per-story adversarial convergence ACHIEVED (BC-5.39.001 MET). Trajectory: 2H→0H(1MED)→CLEAN→CLEAN→CLEAN. 3 consecutive clean passes (passes 3/4/5 on frozen artifact @b003547).

**Remediation summary:**
- Pass-1 2×HIGH: evidence: vec![] empty — BC-2.17.015 PC-1/PC-4 violated. story-writer added evidence postcondition to AC-136-001/002 (factory commit 44c1c7c, body-only edit; input-hash 0846e0e UNCHANGED MATCH). test-writer added RED evidence assertions @bdd0248; implementer populated evidence + removed dead is_open binding @9c9e1bf.
- Pass-2 1×MEDIUM: F-136-ADV-001 stale RED-gate banner (DF-GREEN-DOC-TENSE-SWEEP). test-writer @b003547: banner past-tense, per-occurrence sweep, summary-suffix coverage.
- Passes 3/4/5: 0 findings all passes. CONVERGED.

**Toolchain pairing verified at b003547 (orchestrator):** 10/10 connection_lifecycle tests pass; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean; input-hash STORY-136 = 0846e0e MATCH; 0 behind develop.

No [process-gap] findings across any pass (S-7.02 checklist: nothing to codify).

**NEXT:** demo-recorder → push → pr-manager 9-step PR (halt before merge for human auth per D-231) → merge+cleanup.

---

## D-250 — SESSION PAUSE: STORY-136 mid-TDD Red Gate @1b5d300 (2026-06-26)

Session paused mid-F4 Wave 60 with STORY-136 at Red Gate. SAFE-TO-CLEAR. All F1/F2/F3 + STORY-130-135 artifacts durable on factory-artifacts branch.

**Exact pause state:**
- Cycle: `feature-enip-v0.11.0` — EtherNet/IP + CIP ICS analyzer (SS-17, issue #316). Target v0.11.0.
- Phase: F4 (TDD Implementation), Wave 60 in progress. E-20 (9 stories STORY-130..138). stories_delivered=84.
- Worktree: `.worktrees/STORY-136-enip-lifecycle`, branch `worktree-issue-316-story-136-enip-lifecycle`, base develop `84be2fb`.
- Red Gate commit: `1b5d300` — stub-architect added 2 new EnipFlowState fields (`open_connection_count: u32`, `close_connection_count: u32`) + a `todo!()` process_pdu branch matching ForwardOpen(0x54)/LargeForwardOpen(0x5B)/ForwardClose(0x4E). cargo check/clippy GREEN; 105 prior tests green; 6 RED hold the Red Gate.
- Stub state: stub-architect DRAFTED 10 `connection_lifecycle` tests (6 RED via todo!(), 4 green-by-design suppression-path). Input-hash refreshed 2af89b5→0846e0e (committed 5bb327c on factory-artifacts).

**DO-NOT-REDO fence:**
- STORY-130 MERGED PR #317 @235ae60 (D-234). STORY-131 MERGED PR #318 @edce3bd (D-237). STORY-132 MERGED PR #319 @16d3ce7 (D-239). STORY-133 MERGED PR #320 @7f040de (D-241). STORY-134 MERGED PR #323 @e330ccc (D-247). STORY-135 MERGED PR #324 @84be2fb (D-249). develop=84be2fb.
- STORY-136 Red Gate @1b5d300 + input-hash 0846e0e DONE — do NOT re-run stub-architect; do NOT re-author input-hash.

**BC-2.17.015 exact finding fields (do not re-derive):**
- ForwardOpen(0x54)/LargeForwardOpen(0x5B) REQUEST on 0x00B2: one finding — category=ThreatCategory::Anomaly, verdict=Possible, confidence=Low, mitre_techniques=vec![] (empty per BC-2.17.015 PC-1 + ADR-010 Decision 7), summary="CIP ForwardOpen connection establishment observed from src={src_ip}: connection lifecycle anomaly", source_ip=Some, timestamp=Some; increment open_connection_count (both 0x54 and 0x5B → open_connection_count per Invariant 5).
- ForwardClose(0x4E) REQUEST on 0x00B2: one finding — Anomaly/Possible/Low, mitre vec![], summary="CIP ForwardClose connection teardown observed from src={src_ip}: connection lifecycle closed"; increment close_connection_count.
- EC-008: counts increment BEFORE the MAX_FINDINGS push gate (counts accurate even when all_findings is capped). Response services (0xD4/0xCE) → no count, no finding. 0x00B2 gate (F-P9-001) + is_non_enip gate. Does NOT touch command_counts (F8-001). STORY-136 adds to process_pdu (NOT on_data frame-walk = STORY-137). No window/timestamp state (plain counters) → no F-134-001 ts=0 risk.

**Ground-truth HEADs at D-250:**
- develop: `84be2fb` (STORY-135 merged, PR #324).
- main: `0cbe922` (v0.10.0).
- factory-artifacts: this D-250 checkpoint commit (verify: `git -C .factory log -1`).

---

## D-253 — SESSION PAUSE: STORY-137 per-story adversarial convergence ACHIEVED @c4644f9 (2026-06-26)

Per-story adversarial convergence ACHIEVED (BC-5.39.001 MET). 3 consecutive clean passes (B/C/D on frozen worktree HEAD c4644f9). 0 HIGH/CRITICAL across all three. Branch: `worktree-issue-316-story-137-enip-frame-walk`, base develop `a2cb795`, 0 behind.

**Full trajectory:** 2CRIT+2HIGH (Pass-1) → architect RULING-137-001 → fix → 2HIGH (Pass-2) → RULING-137-002 → fix → CLEAN(1MED Pass-A) → fix → CLEAN × 3 (B/C/D).

**Pass-1 remediation (2CRIT+2HIGH):**
- F-137-P1-001 CRIT: byte-walk resync used `break` instead of `continue` — valid trailing frames silently dropped (detection-evasion). Fixed.
- F-137-P1-002 CRIT: tests locked `break` as correct expected behavior. Reauthored.
- F-137-P1-003/004 HIGH: frame-skip path also used `break`; counting expectations wrong (parse_errors=1 for 24-garbage-byte block should be 24).
- RULING-137-001 issued (architect): `continue` mandatory on both paths; per-offset counting IS intended (crash-probe T0814 threat model). Binding. No BC/ADR/story amendment required.

**Pass-2 remediation (2HIGH):**
- F-137-P2-001 HIGH: carry-overflow test used impossible scenario (frame-skip never stashes into carry).
- F-137-P2-002 HIGH: carry-overflow latch `is_non_enip` structurally unreachable (max carry 599 < cap 600 by algorithm invariant).
- RULING-137-002 issued (architect): genuine design gap (option b); `is_non_enip` never latched by `on_data`; quarantine feature inert; deferred to v0.12.0. Does NOT block STORY-137 convergence.

**Pass-A remediation (1MED):**
- F-137-ADV-001 MED: test-name prose did not reflect dead-code status per RULING-137-002. Test names updated with explicit dead-code annotation.

**Toolchain pairing verified at c4644f9 (orchestrator):** 2058 tests green (`cargo test --all-targets`); `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean; green-doc-tense PASS; input-hash STORY-137 = f4c8390 MATCH; 0 behind develop a2cb795.

**Key implementation facts (do not re-derive):**
- `on_data` is the carry-buffer frame-walk loop.
- `pub flows: HashMap<FlowKey, EnipFlowState>` added to `EnipAnalyzer`.
- `command_counts` relocated to SINGLE canonical frame-walk site (BC-2.17.016 PC-0), counts all commands including Unknown. Removed from `process_pdu` (which now owns `pdu_count` only).
- `#![allow(dead_code)]` removed from `src/analyzer/enip.rs` (WAVE59-DEADCODE-001 resolved).
- Byte-walk resync path and oversized-frame-skip path both use `continue` (RULING-137-001 binding).
- `is_non_enip` carry-overflow latch is dead code (RULING-137-002); never set to `true` by any reachable `on_data` path.

**S-7.02 follow-up items codified:**
1. SPEC-DEFECT-IS-NON-ENIP-DEAD-LATCH — v0.12.0; PO decision on quarantine semantics.
2. ADVERSARY-REACHABILITY-PROOF-OBLIGATION — [process-gap] engine adversarial checklist improvement.
3. HS-117-CASE-D-UNIT-COVERAGE — [process-gap] max-length panic-safety unit test; F4 / Wave-60 sweep.
4. STORY-137-UNSAFE-SPLIT-BORROW — [LOW] unsafe split-borrow in process_pdu; Wave-60 or v0.12.0.
5. T0814-EVIDENCE-TEST — [LOW] no test asserts T0814 evidence field; Wave-60 doc/test sweep.

**Binding ruling documents:**
- `cycles/feature-enip-v0.11.0/STORY-137/frame-walk-counting-ruling.md` — RULING-137-001.
- `cycles/feature-enip-v0.11.0/STORY-137/RULING-137-002-carry-overflow-unreachability.md` — RULING-137-002.

**NEXT:** demo-recorder → push branch `worktree-issue-316-story-137-enip-frame-walk` → pr-manager 9-step PR (halt before merge per D-231) → human approves → merge+cleanup → Wave-60 integration gate (STORY-134-137 integrated, 3-pass wave-level convergence).

---

## D-255 — Wave-60 Integration Gate IN-PROGRESS: F-W60-001 BLOCKS Convergence (2026-06-26)

Wave-60 integration gate initiated with develop HEAD `72a9106`. STORY-134/135/136/137 all merged (stories_delivered=86).

**Phase results:**

1. **Full regression @72a9106:** GREEN. All test suites pass; clippy/fmt clean.
2. **Fresh-context consistency audit:** CLEAN. One LOW finding (NEW-001): STORY-INDEX showed STORY-134/135/136/137 as `draft` — corrected to `completed` / `DELIVERED & CLOSED` in Index Table, Wave Delivery Progress row 60, and E-20 epic note. SS-17 BC files carry `status:draft` + `input-hash:TBD` (tracked SS-17-BC-INPUT-HASH-BACKFILL, cycle-close item).
3. **3-pass Wave-60 adversarial convergence:** NOT CONVERGED.
   - Pass 1: CLEAN (0 findings).
   - Pass 2: F-W60-001 HIGH (BLOCKS) + F-W60-002 MEDIUM (NON-BLOCKING). 3-clean counter RESET.
   - Pass 3: CLEAN (0 findings). Counter = 1/3 (not yet at 3 — blocked).

**RULING-W60-001 issued** (architect, ADR-010 owner). See `cycles/feature-enip-v0.11.0/RULING-W60-001-source-attribution.md` for full adjudication.

**F-W60-001 [HIGH — BLOCKS]:** `EnipAnalyzer::on_data` assigns `src_ip = flow_key.lower_ip()` (the numerically smaller IP, not the traffic originator). All CIP detections (T0846/T0888/T0858/T0816/T0836/ForwardOpen/ForwardClose/T0814) mis-attribute source_ip to the lower-sorting endpoint (~50% of captures will attribute attack to victim controller). RULING-W60-001: FIX via approach (a) — add `resolve_enip_client_ip(flow_key: &FlowKey) -> IpAddr` using port-44818 heuristic (mirrors DNP3 `resolve_master_ip`). Fix-PR branch: `fix/enip-source-ip-attribution`. Residual: `DRIFT-ENIP-DIRECTION-001` documented in function doc-comment.

**F-W60-002 [MEDIUM — NON-BLOCKING]:** `self.bytes_received` updated before `is_non_enip` guard vs BC-2.17.016 PC-5. RULING-W60-001 Part 2: bytes_received is EXEMPT (analyzer-level routing observable per BC-2.17.019 PC-2; not a per-flow analysis counter). Code correct. BC-2.17.016 v1.1→v1.2 clarification (PC-5 exemption sentence + Invariant 7) deferred to cycle-close SS-17 BC backfill to avoid mid-wave input-hash churn on merged stories.

**After fix-PR merge:** re-run full 3-pass Wave-60 adversarial convergence on updated develop HEAD. Then human gate.
