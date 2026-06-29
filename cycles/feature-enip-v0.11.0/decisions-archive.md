---
document_type: decisions-archive
cycle_id: feature-enip-v0.11.0
archived_from: STATE.md Decisions Log
archived_at: 2026-06-29
archived_decisions: D-228..D-301
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

---

## D-252 — STORY-136 MERGED; develop HEAD = a2cb795 (2026-06-26)

STORY-136 PR #326 squash-merged into develop; new develop HEAD = a2cb795 (was 84be2fb). stories_delivered 84→85. CI 11/11 green; pr-reviewer APPROVE (NITs: PRF-001 close-count cap-bypass assertion deferred STORY-138; PRF-002/003 accepted spec-correct); security PASS (SEC-006 LOW pub-field convention deferred W7.1). input-hash 0846e0e MATCH at merge. Worktree worktree-issue-316-story-136-enip-lifecycle cleaned up. NEXT = STORY-137.

---

## D-254 — STORY-137 MERGED; develop HEAD = 72a9106 (2026-06-26)

STORY-137 PR #327 squash-merged into develop; new develop HEAD = 72a9106 (was a2cb795). stories_delivered 85→86. CI 11/11 green; pr-reviewer APPROVE (0 blocking); security PASS (SEC-137-001 MEDIUM unsafe split-borrow pre-authorized-deferred; SEC-137-002/003 LOW). input-hash f4c8390 MATCH at merge. Demo evidence at docs/demo-evidence/STORY-137/. Feature: frame-walk loop + carry-buffer + T0814 + command_counts single-site + dead-code removal (BC-2.17.016/004/018). Worktree worktree-issue-316-story-137-enip-frame-walk cleaned up. NEXT = Wave-60 integration gate.

---

## D-256 — F-W60-001 RESOLVED; fix-PR #328 MERGED; develop HEAD = 0f345c6 (2026-06-26)

fix-PR #328 (`fix(enip): resolve source_ip to client via port-44818 heuristic`) squash-merged into develop; new develop HEAD = 0f345c6 (was 72a9106). stories_delivered stays 86 (fix-PR, not a story). `resolve_enip_client_ip` port-44818 heuristic (approach a, mirrors DNP3 sibling) ships per RULING-W60-001. 2 value-asserting on_data E2E tests + control + 4 unit tests T1-T4 (incl. fallback); WAVE-60-E2E-TEST-COVERAGE RESOLVED. Residual: DRIFT-ENIP-DIRECTION-001 doc-comment non-blocking. CI 11/11 green; pr-reviewer APPROVE; security PASS (SEC-001 LOW magic-number 44818, SEC-002 INFO fallback — both non-blocking). Fix worktree cleaned up by devops. NEXT = re-run Wave-60 3-pass adversarial convergence on develop @0f345c6.

---

## D-257 — Wave-60 Integration Gate CONVERGED (Re-convergence Passes A/B/C CLEAN) (2026-06-26)

Wave-60 wave-level convergence ACHIEVED — re-convergence passes A/B/C on develop @0f345c6 all CLEAN (0 HIGH/0 CRIT per pass). BC-5.39.001 MET at wave level. Wave-60 integration gate: regression GREEN (0 failures, 80 suites, clippy/fmt clean), fresh-context consistency audit CLEAN (NEW-001 resolved), 3 consecutive clean adversarial passes.

Pass C found F-W60-P-M1 MEDIUM (source_attribution docstrings stale "Current code (lower_ip())" prose — NON-BLOCKING; batched into WAVE-60-TEST-DOC-SWEEP for STORY-138/cycle-close doc sweep; corroborates GREEN-DOC-TENSE-GATE-PATTERN-GAP-001). All other Pass A/B/C findings LOW/deferred-confirmations (dest_ip cosmetic O-1, magic-44818, summarize stub-tense [STORY-138], BC input-hash TBD) — already tracked.

Wave-60 integration gate: CONVERGED — PENDING HUMAN GATE.

---

## D-258 — Wave-60 Integration Gate PASSED — HUMAN APPROVED; Wave 61 STORY-138 IN-PROGRESS (2026-06-26)

**Human approved the Wave-60 integration gate on 2026-06-26.** Decision: "Approve → proceed to STORY-138." develop @0f345c6 accepted.

**Deferrals accepted:**
1. **SPEC-DEFECT-IS-NON-ENIP-DEAD-LATCH** — is_non_enip quarantine dead-latch quarantine semantics unresolved → v0.12.0. PO decision required before BC-2.17.016 PC-4/Inv-4/EC-004 can be revised. Non-blocking for v0.11.0.
2. **F-W60-P1-001** — command_counts split-header double-count fix (count once per logical header) → STORY-138 prerequisite (bundled into Wave-61 delivery).
3. **F-W60-002** — bytes_received BC-2.17.016 v1.1→v1.2 clarification (PC-5 exemption + Invariant 7) → cycle-close SS-17 BC backfill.
4. **WAVE-60-TEST-DOC-SWEEP** (incl. F-W60-P-M1 source_attribution "Current code" docstrings, mitre.rs:358 stale BC-2.17.012 label, dnp3_dispatcher_tests.rs:25, arp_tests RED-tense) → bundled into STORY-138 delivery.
5. **SS-17 BC input-hash backfill** — BC-2.17.007+ carry `input-hash:TBD` → cycle-close.
6. **green-doc-tense gate-pattern enhancement** (GREEN-DOC-TENSE-GATE-PATTERN-GAP-001 patterns for "These tests MUST FAIL" etc.) → cycle-close.

**Wave 60 (STORY-134/135/136/137 + fix-PR #328) = COMPLETE + integration-gate PASSED.**
**Wave 61 STORY-138 IN-PROGRESS.**

---

## D-259 — STORY-138 MERGED; develop HEAD = b4624ef; Wave 61 code-complete (2026-06-26)

**STORY-138 MERGED** — PR #329 (`feat(enip): STORY-138 session lifecycle, stats, DoS guard, analyzer summary`) squash-merged into develop; new develop HEAD = b4624ef (was 0f345c6). stories_delivered 86→87.

**BCs delivered:** BC-2.17.025 (RegisterSession/UnRegisterSession classify+no-finding); BC-2.17.017 (on_flow_close fold); BC-2.17.022 (MAX_FINDINGS/dropped_findings DoS guard); BC-2.17.021 (summarize()→enip_summary canonical keys); BC-2.17.024 (pdu_count).

**Prerequisites shipped in STORY-138:** F-W60-P1-001 command_counts count-once fix; WAVE-60-TEST-DOC-SWEEP resolved.

**Convergence:** Per-story adversarial convergence 3/3 consecutive clean passes (BC-5.39.001 MET). CI 11/11 green; pr-reviewer APPROVE (2 review cycles); security PASS (SEC-001 MEDIUM saturating_add fixed @3f55f11; SEC-002/003/004 LOW). input-hash 0f60353 MATCH at merge.

**Wave 61 code-complete.** All STORY-130..138 merged.

**OPEN items carried forward:**
- F-138-P1-004 (HIGH — functional): EnipAnalyzer::on_flow_close never invoked by dispatcher → enip_summary reports 0 for total_pdu_count/command_distribution/parse_errors/flows_analyzed in real captures (write_count/error_count work). Needs architect ruling → fix-PR → BLOCKS Wave-61 integration gate.
- F-138-P1-002 (LOW): BC-2.17.016 PC-0 wording ambiguity — deferred to cycle close.

**NEXT = Wave-61 integration gate** (fix F-138-P1-004 first).

---

## D-260 — F-138-P1-004 RESOLVED via fix-PR #330 (2026-06-26)

**fix-PR #330** (`fix(enip): summarize folds open flows so enip_summary reflects live traffic`) squash-merged into develop. New develop HEAD = `7ceb670` (was `b4624ef`).

**Root cause fixed:** `summarize()` now folds still-open `self.flows.values()` on top of closed-flow aggregates per RULING-W61-001 (DNP3 parity). `enip_summary` now reflects live traffic for `total_pdu_count`, `command_distribution`, `parse_errors`, and `flows_analyzed` — not just `write_count`/`error_count`.

**Verification:** Discriminating test added (enip_summary non-zero on open flows before on_flow_close) + mixed closed+open fold test added. CI 11/11 green; AI APPROVE; security CLEAN.

**Security note:** SEC-006 MEDIUM = pre-existing unsafe split-borrow in `process_pdu` — already tracked as STORY-137-UNSAFE-SPLIT-BORROW, not introduced by this fix.

**Fix worktree** cleaned up by devops concurrently during this merge.

**BC-2.17.021 prose clarification deferred to cycle close** (ruling-sanctioned per RULING-W61-001 — `summarize()` folds open flows per Precond 4; stale "does NOT re-scan / aggregates must be up-to-date from on_flow_close" Invariant 2 wording to be removed).

**HUMAN DIRECTIVE:** STOP before cutting the v0.11.0 release — proceed through Wave-61 gate + F5 + F6 + F7 convergence, then HALT for human go-ahead before the release pipeline.

**NEXT = Wave-61 integration gate** — full regression on develop @7ceb670 + fresh consistency audit + 3-pass wave-level adversarial convergence → Wave-61 human gate.

---

## D-261 — Wave-61 Integration Gate CONVERGED; Pending Human Gate (2026-06-26)

Wave-61 wave-level convergence ACHIEVED. Regression GREEN @7ceb670 (0 failures, 80 suites, clippy/fmt clean). Consistency audit CLEAN. 3-pass adversarial convergence: Pass 1/2/3 all 0 HIGH/0 CRITICAL. BC-5.39.001 MET at wave level. 26/26 SS-17 BC completeness sweep PASSED (all have implementation paths + non-vacuous tests). Admin status cells fixed: STORY-138.md status ready→completed; STORY-INDEX.md story-table draft→completed; STORY-INDEX.md Wave-Delivery-Progress row updated to DELIVERED & CLOSED. New open items recorded: F-W61-001 MEDIUM (dead pub EnipSummary struct, human decision required), F-W61-002 LOW (SAFETY comment omits self.dropped_findings), O-W61-2 LOW (no-finding command process_pdu test optional), WAVE-60-TEST-DOC-SWEEP stale batch (open_connection_count/close_connection_count doc-comment). Wave 61 = CONVERGED, PENDING HUMAN GATE.

---

## D-262 — Wave-61 Integration Gate PASSED (Human-Approved); fix-PR #331 MERGED; F4 COMPLETE (2026-06-26)

Wave-61 integration gate PASSED (human-approved). Wave 61 CLOSED. Pre-F5 cleanup fix-PR #331 (`refactor(enip): wire summarize through EnipSummary + doc fixes`) squash-merged into develop; new develop HEAD = bd9e507 (was 7ceb670). EnipSummary resolution = wire-through (human-chosen). Resolved: F-W61-001 (EnipSummary now load-bearing, byte-identical output, AI+security APPROVE), F-W61-002 (SAFETY comment now lists dropped_findings), O-1 (connection-count doc-comments corrected), WAVE-60-TEST-DOC-SWEEP stale batch. CI 11/11 green. stories_delivered=87 (refactor, no new story). F4 (TDD implementation) COMPLETE. Entering F5 scoped-adversarial refinement on v0.11.0 ENIP delta @bd9e507.

---

## D-263 — F5 Scoped-Adversarial CONVERGED (2026-06-26)

F5 scoped-adversarial CONVERGED. 3 consecutive clean passes on develop @bd9e507: Pass 1 (whole-feature/CLI/release-readiness), Pass 2 (security/DoS/panic-freedom), Pass 3 (spec-fidelity/detection/RTM/holdout-alignment). ALL 0 HIGH/0 CRITICAL, zero novelty. BC-completeness sweep 26/26 (BC-2.17.001..026 all implemented + tested). Detection-attribute matrix verified; panic-freedom + DoS bounds confirmed; no dead code/debug artifacts; PRD §2.17 RTM complete; holdouts HS-110..122 present + boundary semantics satisfied. All F5 findings pre-adjudicated/deferred (no new blocking items). Entering F6 formal hardening.

---

## D-264 — F6 Formal Hardening DISCHARGED; Fuzz-Harness PR Open (2026-06-26)

F6 formal hardening DISCHARGED @bd9e507. Kani 11/11 PASS (VP-032/VP-004/VP-007; Kani 0.67.0). cargo-fuzz F-P9-002: 8,331,310 runs/91s/0 crashes; harness @447da079. audit/deny/clippy/fmt clean. Mutants: 20/20 viable killed; full 241-mutant run CONTINUING. No product logic changed. Fuzz-harness PR open halted for human merge.

---

## D-265 — F6 Gate PASSED; Fuzz-Harness PR #332 MERGED; develop HEAD = f17d270 (2026-06-26)

F6 gate PASSED. Fuzz-harness PR #332 (`test(fuzz): F-P9-002 cargo-fuzz harness for ENIP CIP parsers`) squash-merged into develop; new develop HEAD = f17d270 (was bd9e507). Test-infra PR — stories_delivered stays 87. All F6 hard obligations discharged (Kani 11/11, fuzz 8.3M/0-crash on develop, audit/deny/clippy/fmt clean). cargo-mutants full run still executing in F6 worktree at time of gate (21 caught/0 missed so far — F6-MUTANTS-FULL-RUN checkpoint; confirm 0-missed at completion before F7 sign-off). Entering F7 delta-convergence. HALT after F7 human gate (D-260) — do NOT run release pipeline without explicit go-ahead.

---

## D-266 — F7 Delta-Convergence CONVERGED; Input-Hash Drift Resolved; Pending Human Gate (2026-06-27)

**F7 delta-convergence CONVERGED.** develop @f17d270. 5-dimensional assessment (spec/tests/implementation/verification/regression) all CONVERGED. Full regression GREEN (0 failures, 80 suites). Report: `cycles/feature-enip-v0.11.0/F7-convergence-report.md`.

**Input-hash drift RESOLVED.** STORY-130/131/132/133 were STALE at F7 gate (BC version-metadata bumps + ADR/arch-delta updates, no semantic story impact). verify-then-refresh confirmed all 4 consistent-as-is (no body changes required). Input-hashes refreshed: STORY-130=63fac3a, STORY-131=ce92886, STORY-132=c33dff8, STORY-133=661f504. All SS-17 stories STORY-130..138 now MATCH. Refresh commit: c65602a.

**Holdouts HS-110..122:** NOT formally evaluated (12 needed pcap fixtures never created). 10/13 behaviors covered by exact-byte unit tests. HS-122 real-world-corpus is the only true gap (confidence-add, not correctness-gate). Architect recommends accept-for-v0.11.0 + schedule formal holdout eval as a post-release validation story. NON-BLOCKING per architect.

**F6-MUTANTS-FULL-RUN:** cargo-mutants enip.rs full 241-mutant run was still executing in F6 worktree at F7 gate entry; 21 caught/0 missed at last poll. Full run completion and 0-missed confirmation is a human gate item.

**Phase status:** F7 CONVERGED — PENDING HUMAN GATE. Items the human is weighing: (1) F6-MUTANTS-FULL-RUN 0-missed confirmation; (2) holdout formal-eval defer-decision; (3) is_non_enip dead-latch release-notes narrative; (4) cycle-close BC prose edits (BC-2.17.021 Inv-2, BC-2.17.016 PC-5/PC-0, F-W60-002) post-release; (5) Dependabot #311/#325 triage. HALT (D-260) — do NOT run the release pipeline without explicit human go-ahead.

---

## D-267 — F7 Human Gate PASSED; E2E pcap tests approved; Holdout eval deferred (2026-06-27)

Human F7 gate (F7-Q1..Q5): (Q1) ENIP F7 APPROVED — STORY-130..138 convergence accepted. (Q2) Holdout formal eval deferred to post-release validation story (HS-110..122 not formally evaluated; covered by exact-byte unit tests). (Q3) F6-MUTANTS-FULL-RUN deferred — 0-missed at last count, confirm post-release. (Q4) BC prose edits (F-W60-002, BC-2.17.016 PC-0/PC-5, BC-2.17.021 Inv-2) deferred to cycle-close. (Q5) Dependabot #311/#325 deferred. STORY-130..138 APPROVED to merge. E2E real-pcap PR #333 plan approved (no new BCs needed). F7 HUMAN-APPROVED (D-045 gate met).

---

## D-268 — STORY-130..138 batch PR #317..331 MERGED; stories_delivered=87 (2026-06-27)

All 9 ENIP STORY-130..138 PRs merged to develop: PR #317 (STORY-130), PR #318 (STORY-131), PR #319 (STORY-132), PR #320 (STORY-133), PR #321 (STORY-134), PR #322 (STORY-135), PR #323 (STORY-136), PR #324 (STORY-137), PR #331 (STORY-138). stories_delivered 78→87. CI green on all. No new worktrees; all feature branches deleted. ENIP analyzer epic (E-10 waves 58-61) COMPLETE + MERGED.

---

## D-269 — ENIP E2E Real-pcap PR #333 MERGED; develop HEAD = fd0c7f3 (2026-06-27)

E2E real-pcap test PR #333 (`test(enip): full-pipeline E2E tests against real ENIP/CIP pcaps [WAVE59-E2E-001]`) squash-merged into develop; new develop HEAD = fd0c7f3 (was ff3dceb). Test-infra PR — stories_delivered stays 87. All ENIP F7 obligations fully discharged. v0.11.0 HELD (D-260) pending EC-X1/EC-X2 fix + explicit human release go-ahead.

---

## D-270 — SESSION PAUSE at STORY-139 F4 (EC-X1/EC-X2 fix-delta) (2026-06-27)

SESSION PAUSE at F4 implementation of EC-X1/EC-X2 fix-delta (STORY-139, Wave 62). Worktree `.worktrees/enip-direction-clock` @63c119a (`fix/enip-direction-and-clock`, base `fd0c7f3`): red-gate complete — crate compiles, clippy clean, 9 STORY-139 tests RED, 170 existing GREEN. 3 stub points pending: (1) per-direction carry select (EC-X1/BC-2.17.016 v2.0); (2) direction-based src_ip/dest_ip (AC-139-002); (3) saturating_sub + strict `> 300` window (EC-X2/BC-2.17.008/012/018). Durable resume checkpoint written.

---

## D-271 — STORY-139 F4 COMPLETE (2026-06-27)

F4 implementation of STORY-139 (EC-X1/EC-X2 fix-delta) COMPLETE on `.worktrees/enip-direction-clock` branch `fix/enip-direction-and-clock`, impl commit `3c688ff` (base red-gate `63c119a`). All 3 stub points filled: (1) per-direction carry select/stash carry_c2s/carry_s2c [EC-X1/BC-2.17.016 v2.0 Inv-7]; (2) direction-based src_ip inline, resolve_enip_client_ip removed [AC-139-002/DRIFT-ENIP-DIRECTION-001]; (3) saturating_sub + strict >300 window [EC-X2/EC-X4/BC-2.17.008/012/018]. Green gate: cargo test --all-targets 179 GREEN / 0 RED; clippy -D warnings clean; fmt clean.

---

## D-272 — STORY-139 F4→F5 fix-burst chain; F-139-02 adjudicated (2026-06-27)

STORY-139 F4→F5 fix-burst chain on worktree enip-direction-clock: impl 3c688ff → F-139-01 doc + green-doc-tense 69cbf18 → carry-clear removal 046cc41 → operator-pin test rewrite 573a977 → VP-033/034 real proptests + resolve_enip_client_ip prose sweep d0b7b78 → EC-X1 false-positive guard assertion + STORY-137 break/continue doc-tense fc29f2f → summarize_drainage doc-tense 0607b82. F-139-02 (malformed-window carry-clear) ADJUDICATED by architect (RULING-EDGECASE-001 addendum): REMOVE carry-clear (restore BC-2.17.018 PC-5 3-reset minimal-fix) + rewrite test_malformed_window_operator_pin_boundary to oversized-declared-frame path; no BC/AC amendment. O-3 (AC-139-002 direction:Some) ADJUDICATED out-of-scope (per-flow aggregates; deferred v0.12.0). F-139-03 dispatcher path fixed in STORY-139 + ruling.

---

## D-273 — STORY-139 F5 CONVERGED (2026-06-27)

STORY-139 F5 scoped-adversarial CONVERGED. 6 fresh-context adversary passes (findings decayed 4→3→2→0): passes 4/5/6 + final confirming pass all CLEAN (zero HIGH/CRITICAL, zero mis-anchors). Post-fixburst consistency audit (consistency-validator, 6 dimensions) verdict CONSISTENT — malformed-window code matches BC-2.17.018 PC-5 exactly (3 resets, no carry-clear), 2 remaining carry-clears are cap-overflow only (BC-2.17.016 PC-4), input-hash 759464a MATCH. OBS-1 (proptest_ prefix on 2 deterministic VP-034 guards) ADJUDICATED ACCEPTED: names spec-cited by AC-139-005 + VP-034 spec.

---

## D-274 — STORY-139 F6 PASS (2026-06-27)

STORY-139 F6 targeted hardening PASS. Worktree commits since F5: 25f8b4a (5 boundary-hardening tests killing carry-direction-select 948 + error/write-window boundary mutants 1152/1336), cee85c0 (cap-check doc-comment corrected). Kani: 36/37 proved (all 5 ENIP harnesses PROVED + non-vacuous; 1 outstanding harness vp025_timestamp_totality_base10 in reader.rs — non-delta, still-solving CBMC SAT, NOT a regression). cargo-fuzz fuzz_enip_cip_parse: 14.99M execs / 0 crashes. cargo-mutants delta: 23/28 caught; of 5 survivors, 3 (carry-overflow cap+clear) are PROVEN-EQUIVALENT permanent survivors and 2 (pre-existing out-of-scope). In-scope killable kill-rate = 23/23 = 100%. Regression 2112/0; VP-033 2/2, VP-034 6/6.

---

## D-275 — ARCHITECT RULING on carry-cap unreachability (RULING-137-002) (2026-06-27)

ARCHITECT RULING (RULING-EDGECASE-001 addendum, 3rd adjudication): ENIP carry-overflow cap (enip.rs:953/967, BC-2.17.016 PC-4, MAX_ENIP_CARRY_BYTES=600) is STRUCTURALLY UNREACHABLE — RULING-137-002 §1 proves max carry after any on_data call is 599 bytes. The 3 cargo-mutants survivors are CONFIRMED EQUIVALENT (excluded from kill denominator). RULING-EDGECASE-001 §3's Path-A reachability claim RETRACTED. Resolution: keep cap-check as belt-and-suspenders dead code with corrected comment; BC-2.17.016 PC-4 additive errata NOTE.

---

## D-276 — Input-hash re-baseline after BC-2.17.016 additive NOTE (2026-06-27)

Input-hash mechanical re-baseline after BC-2.17.016 additive NOTE (story-writer commit 4f4dc76, factory-artifacts). STORY-139: 759464a→581b0fd. `bin/compute-input-hash --write --scan` rewrote 24 stale stories total → MATCH=89 STALE=0. 14 non-ENIP stories (STORY-002/003/004/005/071/076-080/100/101/120/129) were PRE-EXISTING stale from prior released-cycle BC doc-edits; swept clean. 3 pre-existing structural ERRORs (STORY-001 retired-BC-ref, STORY-091, STORY-121 missing inputs blocks) → backlog.

---

## D-277 — STORY-139 MERGED to develop (PR #334, 99a06f4); stories_delivered=88 (2026-06-27)

STORY-139 (ENIP EC-X1/EC-X2 detection-correctness fix) MERGED to develop via PR #334 (merge commit 99a06f4; develop HEAD 99a06f4). Both reviewers APPROVE (pr-reviewer 0 findings; security-reviewer 0 CRITICAL/0 HIGH, 1 MEDIUM = documented-unreachable carry-cap per RULING-137-002 non-blocking, 2 LOW pre-existing). CI 11/11 green (run 28302628291). Pre-merge: ADR-010 Decision 4 amended; input-hash re-baseline c99d7b6 (STORY-130..139; STORY-139=16e5c27). Worktree enip-direction-clock + branch fix/enip-direction-and-clock removed. stories_delivered=88.

---

## D-278 — HUMAN DIRECTIVE: fix DNP3 atomically with ENIP for v0.11.0 (2026-06-27)

HUMAN DIRECTIVE (F7 gate, Q2): fix DNP3 ATOMICALLY with ENIP for v0.11.0 — DRIFT-DNP3-DIRECTION-001 (carry splice) + DRIFT-DNP3-CLOCK-001 (wrapping_sub clock reset) are NO LONGER deferred to v0.12.0; they are now IN SCOPE for v0.11.0 as a sibling fix (new STORY-140, full F1-F7 cycle, mirroring RULING-EDGECASE-001 applied to dnp3.rs / SS-15). HUMAN DIRECTIVE (Q4): v0.11.0 STAYS HELD after the ENIP merge — separate explicit release go-ahead required (extends D-260). v0.11.0 ships ENIP + DNP3 together once both land on develop and human approves release.

---

## D-279 — STORY-140 DNP3 F1-F4 COMPLETE (2026-06-27)

STORY-140 DNP3 F1-F4 complete. F1 RULING-DNP3-SIBLING-001 (88d41fd). F2: 4 SS-15 BCs amended (e04809d: BC-2.15.016 v2.0/010 v1.8/014 v2.1/015 v2.0), ADR-007 amended (.factory 1e39373), VP-035/VP-036 registered + indexes (ab3c270, VP-INDEX v2.13/36 VPs). F3 STORY-140 authored (6d6e3a3, E-15 Wave 63, input-hash d498e66). F4 worktree .worktrees/dnp3-direction-clock from develop 99a06f4: red-gate scaffolding 7a225aa (208 call-sites threaded) + RED tests b761033, impl af66b9d, block-timeout saturating_sub a5ca673, test fixes 28b5673, regression-fix 1dda26b. 2128 GREEN/0 RED. clippy/fmt clean; 0 live wrapping_sub; singular carry gone; resolve_master_ip gone.

---

## D-280 — ARCHITECT adjudications during STORY-140 F4 (2026-06-27)

ARCHITECT adjudications during STORY-140 F4 (RULING-DNP3-SIBLING-001 author): (1) AC-140-002 test rebuilt to standard topology (outstation:20000, master:54321 ephemeral); port-heuristic+direction formula correct (mirrors ENIP). (2) block-timeout (BC-2.15.014/T1691.001) → saturating_sub; old STORY-109 AC-014 test_pending_request_timeout_wrapping_sub SUPERSEDED → renamed test_pending_request_timeout_no_spurious_fire_on_rollover_or_backwards_ts.

---

## D-281 — REGRESSION caught during STORY-140 F4; fixed (2026-06-27)

REGRESSION caught during STORY-140 F4 (orchestrator refused to accept test-writer's 'pre-existing' dismissal; devops bisect confirmed develop@99a06f4 GREEN vs worktree RED). Carry-split refactor changed dnp3 frame-walk loop guard `< 3` → `< 10`, silently dropping parse_errors increments. Fixed (1dda26b) via did_process_in_this_call context tracking (dnp3.rs:442/455/495/535). No tests dropped (2128 = develop 2112 + 16 new).

---

## D-282 — STORY-140 F5 CONVERGED (2026-06-27)

STORY-140 DNP3 F5 scoped-adversarial CONVERGED. 6 fresh-context adversary passes (findings 2 MED → 3 MED+1 LOW → 0 → 1 LOW → 2 LOW → 0): passes 3/4/5 + confirming pass all zero-HIGH/CRITICAL/mis-anchor. Worktree commit chain: 1dda26b → ac8f2b3 → 5bc6caa → 9972037 → e16ee56. 24/24 BC clauses; VP-035/036 genuine non-vacuous proptests; AC-140-002b genuinely discriminates direction.

---

## D-283 — STORY-140 F6 PASS (2026-06-27)

STORY-140 DNP3 F6 targeted hardening PASS @499c778. Kani: 36/37 proved; cargo-fuzz fuzz_dnp3_parse: 5.18M execs/0 crashes (+ pre-existing harness 4-arg signature gap found+fixed+committed b40d1d9). Mutation delta: first remediation (7bcbbaa, 28 tests) had 11 Group-A survivors; second remediation (499c778, 11 targeted tests) VERIFIED via actual cargo-mutants re-run — all 11 Group-A caught. Only 3 Group-D MAX_FINDINGS DoS-cap off-by-one remain — accepted impractical (pre-existing, mirrors modbus.rs). Regression 2168/0; VP-035 2/2, VP-036 6/6.

---

## D-284 — PROCESS NOTE: F6 mutation gate required two orchestrator verification interventions (2026-06-27)

STORY-140 F6 mutation gate required TWO orchestrator verification interventions — (1) refused 'pre-existing' dismissal of 3 f5_resync_accounting RED tests → devops bisect proved STORY-140 regression; (2) refused test-writer 'structurally killed' claim → formal-verifier cargo-mutants re-run found 11 Group-A survivors. Both caught real gaps. Reinforces DF-ADVERSARY-TOOLCHAIN-PAIRING-001 / verify-don't-trust on mutation + regression claims.

---

## D-285 — STORY-140 F7 CONVERGED; Human gate: HOLD MERGE (2026-06-28)

STORY-140 DNP3 F7 delta-convergence CONVERGED (consistency-validator: 5/5 convergence dims + 6/6 consistency, 1 minor non-blocking BC-2.15.014 line-citation finding). docs/adr/0007 develop-tree copy amended on worktree (560efd3). HUMAN GATE outcome: (Q1) APPROVE STORY-140 convergence but HOLD MERGE — separate explicit go-ahead required before merging; (Q2) v0.11.0 STAYS HELD; (Q3) fix both backlog items now. STORY-140 worktree @560efd3, 2168/0 green, ready to merge on go-ahead.

---

## D-286 — Backlog fixes: BC line-citation + input-hash re-baseline (2026-06-28)

BACKLOG FIXES (human-approved): (1) BC-2.15.014 EC-006 + v2.0 changelog stale source-line citation 984-991 → verified post-STORY-140 line 1173-1200 (commit eb406d1). (2) Input-hash mechanical re-baseline after SS-15 BC v2.x amendments + BC-2.15.014 fix (commit a915faa): STORY-140 d498e66→b3a4fd0; full scan MATCH=90 STALE=0.

---

## D-287 — STORY-140 PR #335 OPENED; ready to merge (2026-06-28)

STORY-140 PR #335 (https://github.com/Zious11/wirerust/pull/335) OPENED targeting develop; branch fix/dnp3-direction-and-clock pushed (HEAD 7169963). Human chose 'Push + open PR, hold merge'. Gates: cargo test 2168/0, clippy/fmt clean; CI 11/11 green; pr-reviewer APPROVE (0 findings); security-reviewer APPROVE (0 CRITICAL/HIGH/MEDIUM, 4 LOW non-blocking). READY TO MERGE — merge command held.

---

## D-288 — STORY-140 MERGED to develop (PR #335 squash, b6d7a01); stories_delivered=89 (2026-06-28)

STORY-140 (DNP3 EC-X1/EC-X2 sibling fix) MERGED to develop via PR #335 — SQUASH merge commit b6d7a01 (develop ff-pulled 99a06f4→b6d7a01, 24 files). Human gave merge go-ahead lifting D-285 hold. Worktree .worktrees/dnp3-direction-clock + branch fix/dnp3-direction-and-clock removed; 17 stale refs pruned. stories_delivered 88→89. Both EC-X1/EC-X2 release-blockers now resolved on develop.

---

## D-289 — REPO MERGE POLICY: squash-only set (2026-06-28)

Repo Zious11/wirerust set to SQUASH-ONLY (allow_squash_merge=true, allow_merge_commit=false, allow_rebase_merge=false, delete_branch_on_merge=true). Supersedes prior merge-commit-only policy. FLAG: squash-only is repo-wide, so develop↔main sync should compare by tag/cherry, not commit ancestry. Consider adding develop branch protection (required status checks) as a future hardening item.

---

## D-290 — GitHub branch protection configured on develop + main (2026-06-28)

GitHub branch protection configured on BOTH develop + main (both were previously unprotected). Settings: require PR before merge (required_approving_review_count=0); required_status_checks strict=true with 11 exact contexts (Test, Clippy, Format, Fuzz build, Audit, Deny, Trust-boundary, Help-provenance gate, Action pin gate, Green-doc-tense gate, Semantic PR); required_linear_history=true; required_conversation_resolution=true; allow_force_pushes=false; allow_deletions=false. enforce_admins: develop=false, main=true. Resolves DEVELOP-BRANCH-PROTECTION backlog item.

---

## D-291 — EDGE-CASE HUNT (2026-06-28)

EDGE-CASE HUNT (2026-06-28, post-v0.11.0-merge): 6 parallel adversary hunters across all analyzers + shared infra. Register: cycles/feature-enip-v0.11.0/EDGE-CASE-HUNT-REGISTER-2026-06-28.md. ~30 candidates total (4 CRIT, ~9 HIGH, MED/LOW). Human directive: record register only (no fixes yet) + verify Modbus + scope 2 systemic design passes. Confirmed-clean: reassembly engine, pcapng reader, MAX_FINDINGS cap, TLS SNI 4-way classification. All candidates DF-VALIDATION-001-gated. 2 cross-cutting design notes written (afd7dbb): DESIGN-TIMESTAMP-MONOTONICITY.md + DESIGN-CROSS-DIRECTION-STATE.md.

---

## D-292 — MODBUS EC-X1 + EC-X2 EMPIRICALLY CONFIRMED (2026-06-28)

MODBUS EC-X1 + EC-X2 EMPIRICALLY CONFIRMED via test-vs-control repro (scratch worktree .worktrees/modbus-ecx-verify @ 74f2913). EC-X1: partial C2S ADU carry spliced into S2C response buffer. EC-X2: one backwards-ts write resets burst window. RULING-EDGECASE-001 §1.6 "Modbus already has direction threading and is NOT affected" DISPROVEN. STORY-104 AC-006 mandates the buggy wrapping_sub — spec must change in the same fix story. Material to held v0.11.0 release.

---

## D-293 — 2 cross-cutting design-scope notes written (2026-06-28)

2 cross-cutting design-scope notes (factory-artifacts afd7dbb): (1) DESIGN-TIMESTAMP-MONOTONICITY.md — Modbus has 4 un-swept wrapping_sub sites; recommend saturating_sub consistent with ENIP/DNP3 precedent; ARP uses saturating_sub but denominator-policy is a separate BC decision; WindowClock abstraction deferred. (2) DESIGN-CROSS-DIRECTION-STATE.md — DNP3 is_non_dnp3 desync-latch is one-line fix; Modbus carry is the sole structural gap; full (FlowKey,Direction) keying NOT recommended for counters.

---

## D-294 — WAVE 64 opened; STORY-141 (Modbus) + STORY-142 (DNP3-desync) F1-F3 DONE (2026-06-28)

WAVE 64 opened (human-directed): fix Modbus EC-X1/EC-X2 (STORY-141) + DNP3 desync-latch (STORY-142) before v0.11.0 release; bundle both. F1: RULING-MODBUS-SIBLING-001 + RULING-DNP3-DESYNC-001 + RULING-EDGECASE-001 §1.6 correction (1f1c648). F2: BCs amended — BC-2.14.002 v2.0/016 v2.3/017 v2.7/019 v1.5, BC-2.15.009 v2.0; BC-INDEX v1.87; VP-037/038 registered; VP-INDEX v2.14/38 VPs. F3: STORY-141 (E-14, 13 ACs, hash 41b8662) + STORY-142 (E-15, 3 ACs, hash 16f87c4) authored; STORY-104 AC-006 corrected (wrapping→saturating superseded); STORY-INDEX v3.1 (95 stories/64 waves); input-hash rebaseline STORY-102/104/106 STALE=0.

---

## D-295 — Wave-64 F5 CONVERGED @ab37fb5 (2026-06-28)

Wave-64 F5 scoped-adversarial CONVERGED @ab37fb5 (bundled STORY-141 Modbus + STORY-142 DNP3-desync). 7 fresh-context passes, ALL zero-HIGH/CRITICAL. 18/18 BC clauses covered. Architect VERIFIED the DNP3 fix was INCOMPLETE (both-carries-empty-only missed sub-case ii) → corrected to frame_count==0 guard. Worktree commit chain: d3e5d6e→1851f3d→4dc9b9a→315992d→25b20e9→de6d124→f23b7cf→ab37fb5.

---

## D-296 — Wave-64 F6 PASS @235a4a1 (2026-06-28)

Wave-64 F6 hardening PASS @235a4a1. Kani 36/37 (1 orthogonal reader harness still-solving, 0 failed). cargo-fuzz fuzz_modbus_parse 4.29M + fuzz_dnp3_parse 3.11M, 0 crashes. cargo-mutants delta: DNP3 desync fix-logic mutants ALL caught; Modbus non-cap mutants caught; 4 saturating_sub + sustained `>=` covered by VP-038. Regression 2192/0; VP-037 2/2, VP-038 5/5.

---

## D-297 — ARCHITECT RULING: Modbus carry-cap STRUCTURALLY UNREACHABLE (ADDENDUM-002) (2026-06-28)

ARCHITECT ADJUDICATION (RULING-MODBUS-SIBLING-001 ADDENDUM-002): Modbus carry-cap guards at modbus.rs:1104/1150 are STRUCTURALLY UNREACHABLE — active_carry.clear() at L1075 drains dir_carry to 0 before the walk loop; max operand 259 < 260=MAX_ADU_CARRY_BYTES. The 6 cargo-mutants survivors are EQUIVALENT. F1 'reachable' claim RETRACTED. Follow-ups: ADDENDUM-002 + BC-2.14.002 errata (e773d55), unreachable code comments (235a4a1).

---

## D-298 — Wave-64 F7 CONVERGED + human-APPROVED; DIM1-01/DIM3-01 pre-merge fixes (2026-06-28)

Wave-64 F7 CONVERGED + human-APPROVED. Consistency audit PASS. 2 non-blocking F7 audit items fixed pre-merge: DIM1-01 — RULING-MODBUS-SIBLING-001 ADDENDUM-002 + BC-2.14.002 v2.1 if-guard line anchors reconciled 1104/1150→1110/1162; BC-INDEX bumped v1.87→v1.88. DIM3-01 — VP-037 proptest range corrected 1usize..7→0usize..6; VP-INDEX v2.14 changelog entry added. All non-behavioral. Code PR squash-merge dispatched. v0.11.0 release STILL HELD — separate explicit human go-ahead required.

---

## D-299 — Wave 64 MERGED (PR #336, a13b5c5); stories_delivered=91; worktrees removed (2026-06-28)

Wave 64 (STORY-141 Modbus EC-X1/EC-X2 + STORY-142 DNP3 desync-latch) MERGED to develop via PR #336 — squash merge commit `a13b5c5` (merged 2026-06-28T20:52:37Z). CI 10/10 applicable checks green (run 28335634278). stories_delivered 89→91. All four v0.11.0 EC-X1/EC-X2 fixes on develop: STORY-139 (ENIP, PR #334), STORY-140 (DNP3 sibling, PR #335), STORY-141 (Modbus, PR #336), STORY-142 (DNP3 desync-latch, PR #336). Worktrees .worktrees/wave64-ec-fixes + .worktrees/modbus-ecx-verify removed; remote+local feature branches deleted. Wave-64 sub-cycle closed.

---

## D-300 — v0.11.0 RELEASED (2026-06-29)

v0.11.0 RELEASED. Explicit human release go-ahead received — releases holds D-260/D-278/D-299. Release PR #337 (`chore: release v0.11.0`) squash-merged → main (HEAD 3072e828). Annotated tag `v0.11.0` created on main and pushed. GitHub release published at https://github.com/Zious11/wirerust/releases/tag/v0.11.0 (marked latest). develop synced via PR #338 squash-merged (develop HEAD ecbcd268; Cargo.toml version 0.11.0 + CHANGELOG entry). main + develop in sync. All 11 CI checks green on both merge commits. crates.io publish declined by human — no publish. Contents: four EC-X1/EC-X2 fixes — ENIP (STORY-139, PR #334), DNP3 EC-X1/X2 (STORY-140, PR #335), Modbus EC-X1/X2 (STORY-141, PR #336), DNP3 desync-latch (STORY-142, PR #336). Version: 0.10.0→0.11.0. Cycle feature-enip-v0.11.0 CLOSED. SEC-001 (unsafe split-borrow enip.rs `on_data`, MEDIUM) registered as v0.12.0 candidate.

---

## D-301 — POST-RELEASE: v0.11.0 CHANGELOG correction + STORY-143 (2026-06-29)

POST-RELEASE CORRECTION (docs-only, develop only). v0.11.0 CHANGELOG entry and GitHub release notes initially omitted the entire ENIP analyzer epic (STORY-130..138, PRs #317–#334). Corrections: PR #339 (develop 0b0af26) fixed CHANGELOG footer comparison links. PR #340 (develop ab0b388) authored the complete [0.11.0] CHANGELOG entry (PR span #317–#338 verified; green-doc-tense gate passed). GitHub v0.11.0 release notes edited to mirror the complete entry (40 ENIP/MITRE markers confirmed, still Latest). Release tag/commit unchanged. main CHANGELOG will catch up on next gitflow back-merge. Lesson codified: RELEASE-CHANGELOG-FULL-RANGE-001 in cycles/feature-enip-v0.11.0/lessons.md. STORY-143 (E-11, draft, 3 pts) created to harden the release-changelog step. STORY-INDEX v3.2 (96 stories). develop HEAD = ab0b388.

OPEN QUESTION (awaiting human answer): should the corrected CHANGELOG.md be fast-tracked onto main now (via a docs-only PR), or left to ride to the next gitflow back-merge at v0.12.0 release time? The complete entry exists on develop (ab0b388). main currently has the original short v0.11.0 entry. No functional impact either way.
