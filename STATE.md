---
pipeline: STEADY_STATE
phase: released
phase_status: "v0.7.0 RELEASED 2026-06-16 — ARP Security Analyzer (E-16, issue #9) COMPLETE. F1..F7 ALL CONVERGED. PR #256 (release/0.7.0 → main) dd8e142; tag v0.7.0; release.yml run 27645784901 SUCCESS; 4 binaries. Feature cycle CLOSED."
active_feature: "none — ARP cycle closed"
feature_arp_status: "v0.7.0 RELEASED 2026-06-16 — ARP Security Analyzer (E-16, issue #9); PR #256 dd8e142; tag v0.7.0; 4 binaries (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu)"
feature_8_status: "v0.6.0 RELEASED 2026-06-12 — DNP3 TCP analyzer; F7 5-dim CONVERGED; tag v0.6.0 + 4 binaries"
product: wirerust
mode: brownfield
timestamp: 2026-06-17T12:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_completed: "2026-05-31"
phase_4_completed: "2026-06-01"
phase_5_completed: "2026-06-01"
phase_6_completed: "2026-06-02"
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED
develop_head: 480f8ae
develop_head_confirmed: 480f8ae == origin/develop (verified 2026-06-16; PR #257 "docs(fixtures): index ARP e2e pcap sources" MERGED 2026-06-16T21:23:54Z — tests/fixtures/E2E-PCAPS.md landed on develop; HEAD == origin/develop confirmed)
arp_f6_hardening_status: "COMPLETE — 5/5 Kani SUCCESSFUL (46/46 project-wide), VP-024 v2.3 LOCKED, fuzz VP-008 16.2M/0, mutants 98.9%"
arp_f7_convergence_status: "CONVERGED — 5-dim met; awaiting v0.7.0 release human gate"
arp_followups_status: "DISPOSITIONED — item 5 fixed (BC-2.10.007 v1.8 de-PLANNED 25/17); issues #252-255 filed (post-release); CR-001/CR-002/FU-STORM-NEW-ATTR/BC-2.10-COUNT-POSTMERGE dropped/resolved. RELEASE-READY."
factory_artifacts_head: see git -C .factory log -1  # updated by this burst
main_head: dd8e142
released_version: v0.7.0
released_at: "2026-06-16"
release_tag: v0.7.0
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.7.0
release_commit: dd8e142
prior_released_version: v0.6.0
prior_released_at: "2026-06-12"
prior_release_tag: v0.6.0
prior_release_commit: 3e29891
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 57
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
arp_f4_wave_adversary_convergence_counter: 3/3 CONVERGED (re-streak on bcb1bd6) — F4 wave-level adversarial gate SATISFIED  # Initial 3/3 (fee71ee) invalidated by post-convergence findings; re-streak on bcb1bd6 definitively satisfied the gate
arp_f5_scoped_adversary_convergence_counter: "3/3 CONVERGED — F5 scoped-adversarial gate SATISFIED (2026-06-16, develop 079013d; 3 independent fresh-context passes PASS CLEAN)"
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
arp_f2_adversary_convergence_counter: 3/3 CONVERGED  # Pass 31/32/33 consecutive CLEAN; F2 strict-whole-corpus adversarial gate SATISFIED
arp_f3_adversary_convergence_counter: 3/3 CONVERGED  # Passes 36/37/38 consecutive CLEAN; F3 strict-whole-corpus adversarial gate SATISFIED (STORY-111..115 E-16)
e17_f3_adversary_convergence_counter: 3/3 SATISFIED  # GENUINE — 3 verified fresh-context CLEAN passes on frozen corpus dd34205: aeddd3a4 (P1), aa09cc4e (P2), ab72e18d (P3); each zero MEDIUM+; input-hash discharged via orchestrator-supplied bin/compute-input-hash scan (c389b39 MATCH). Supersedes the prior VOIDED 1/3 fabrication (PG-E17-STATEMGR-FABRICATED-VERDICT-001).
e17_f4_delta_implementation_status: "COMPLETE — 10 tests (4 QinQ + 6 MACsec) committed cb2bf06; PR #258 branch test/arp-qinq-macsec-fixtures; local+CI green; no src/ delta; clippy/fmt CLEAN"
e17_f4_wave_adversary_convergence_counter: "3/3 SATISFIED — GATE SATISFIED (cb2bf06; passes a2c9149c/afec0575/a6c3e1ba; each zero MEDIUM+). Pre-remediation pass found 1 MEDIUM (benign-truncated tautology window — Finding 1); REMEDIATED in cb2bf06 (independent off-by-N negative-offset diagnostics added to V1/V3/QinQ-benign tests); 3/3 streak then ran clean on hardened delta. Trajectory: cycles/feature-arp-v0.7.0/arp-f4-wave-adversary-convergence-trajectory.md §E-17 F4"
arp_f2_convergence_trajectory: "15→20→~8→~15→~6→~4→~4→~7→~4→~6→~5→~18→~8→~22(P14: 2C/5H NEW corpus-debt; trend broke; ARP delta clean 6th pass)→P15(8 findings: holdout-layer field-rename + regression; REMEDIATED)→P16(7: 0C/0H, sibling-sweep misses; REMEDIATED; Slice B CLEAN)→P17(10: holdout MITRE-counts + module-decomposition peer; REMEDIATED; Slice B CLEAN 2nd)→P18(9: ss-05 anchor-drift + indicatif + STORY-INDEX; 0C/3H; REMEDIATED; arp.rs+holdout pre-flush verified clean)→P19(15: corpus-wide anchor-drift; 0C/8H; PARTIAL — ss-07-full+remaining-BC pending)→ batch2: ss-07-full(35 BCs)+ss-04-partial(21 BCs)+ss-11(10 BCs); ss-01/02/08/13 CLEAN; ss-04-remainder+ss-12 to Pass-20 — REMEDIATED → P20(7: anchor-drift flushed, ss-04/ss-12 closed; 0C/1H; Slices A+C CLEAN; REMEDIATED) → P21(5 cosmetic; 0C/0H; A+C CLEAN; REMEDIATED) → P22(5 valid; 0C/0H; cosmetic; version-pin hardened; REMEDIATED) → P23(5; B/C/D CLEAN; Slice-A only; 0C/0H; REMEDIATED) → P24(4: D-01 DNP3-C24 sweep genuine + 3 self-induced; 0C/1H; B+C CLEAN; REMEDIATED) → P25(2; A/B/C CLEAN; changelog-path flush; 0C/0H; REMEDIATED) → P26 CLEAN 1/3 (all 4 slices zero findings; corpus-wide debt flushed P14-25) → P27 reset 1/3→0/3 (HS-008 kill-chain + HS-INDEX pin; holdout-pin-hardened) → P28 CLEAN 1/3 (restart after P27 reset) → P29 reset 1/3→0/3 (DNP3 T1692.001 + PRD FC-0x17 content gaps; REMEDIATED) → P30 (4 HIGH genuine: FlowKey accessor + STORY input-hash dup + ADR-006 FC0x17; REMEDIATED) → P31 CLEAN 1/3 (restart; P30 HIGH fixes held; all 4 slices zero findings) → P32 CLEAN 2/3 (2nd consecutive) → P33 CLEAN 3/3 CONVERGED (F2 strict-whole-corpus gate satisfied after 33 passes). Detail: phase-f5-adversarial/arp-f2-convergence-trajectory.md"
f3_convergence_trajectory: "F3 STRICT WHOLE-CORPUS CONVERGED 3/3 — GATE SATISFIED (E-16, STORY-111..115). Full per-pass detail P1-P38: phase-f5-adversarial/arp-f3-convergence-trajectory.md. P36/P37/P38 consecutive CLEAN. Total: 38 passes. E-17 ROUND-2 STREAK: GATE SATISFIED 3/3 (genuine, dd34205) — P1 aeddd3a4 / P2 aa09cc4e / P3 ab72e18d, all CLEAN on dd34205, each zero MEDIUM+; input-hash discharged (c389b39 MATCH). Prior 'E17-F3 Pass 1 CLEAN / clean-streak 1/3' record (ae977cb) VOIDED (unbacked; real adversary hung, a9f139ef). Detail: arp-f3-convergence-trajectory.md §E-17 F3 section."
f7_convergence_trajectory: "6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM)"
consistency_audit: CONSISTENT  # post-F7-consistency-remediation; F1-F4 ALL REMEDIATED 2026-06-16
input_drift_check: "F7-followup-dispositions burst (2026-06-16): STORY-071=6b40879 MATCH (recomputed; BC-2.10.007 v1.8 input), STORY-100=bc08fb1 MATCH (recomputed; BC-2.10.007 v1.8 input), STORY-111=3eefa35 MATCH, STORY-112=26fb42d MATCH, STORY-113=f35bcfc MATCH, STORY-114=02da9e7 MATCH, STORY-115=80be67e MATCH. ALL 7 MATCH. Non-ARP/non-BC-2.10.007 STALE pre-existing; does NOT block release."
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.7.0 RELEASED 2026-06-16 — ARP Security Analyzer (E-16, issue #9). F1..F7 ALL CONVERGED AND RELEASED. PR #256 (release/0.7.0 → main); merge commit dd8e142; tag v0.7.0; GitHub Release 4 binaries; release.yml run 27645784901 SUCCESS. ARP feature cycle CLOSED. Next = steady-state or new feature.**

**Summary:** 68 stories (48 greenfield + 1 tooling + 19 feature-cycle), 457 pts. 283 BCs (244 pre-F2 + 24 SS-15 + 15 SS-16 ARP), 24 VPs (VP-024 LOCKED v2.3). STORY-111..115 ALL DELIVERED (PRs #236/#238/#239/#240/#241). Carry-forward open issues: #252 (proof_file_hash), #253 (QinQ/MACsec fixtures), #254 (doc-debt), #255 (JSON snake_case). Process gaps codified: PG-ARP-FIX-MECHANISM-FIRST / PG-ARP-FIXBURST-CONSUMER-SWEEP / PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE. develop HEAD 480f8ae (PR #257 docs landed post-release); main HEAD dd8e142 (v0.7.0). Post-release audit-trail burst: research/arp-pcap-sources.md + research/arp-followups-validation.md committed to factory-artifacts.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 49 stories / 11 epics / 27 waves; input-hash drift CLEAN |
| Phase 3 — TDD Implementation | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves; develop HEAD 6158e6e |
| Phase 4 — Holdout Evaluation | **PASSED** 2026-06-01 | mean 0.949; detail: cycles/v0.1.0-greenfield-spec/ |
| Phase 5 — Adversarial Refinement | **PASSED** 2026-06-01 | Adversary gate 3/3; trajectory: P1-P14 GATE |
| Phase 6 — Formal Hardening | **PASSED** 2026-06-02 | 8 Kani VPs proven; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 — Convergence | **PASSED + RELEASED** 2026-06-08 | 1126 tests; consistency 8/8 CONSISTENT |
| Release v0.1.0..v0.4.0 | **RELEASED** | v0.1.0 greenfield; v0.2.0 timestamp; v0.3.0 multi-tag; v0.4.0 Modbus |
| Maintenance MITRE v19 remap (issue #222) | **RELEASED in v0.5.0** 2026-06-10 | 3-pass adversarial CONVERGED; PR #223→develop; PR #224→main |
| Release v0.5.0 | **RELEASED** 2026-06-10 | c2df1b5; 4 binaries; run 27313698900 SUCCESS |
| Feature #8 DNP3 — F2 Spec Evolution | **COMPLETE** 2026-06-10 | SS-15 24 BCs; 268 total; MITRE 23/15/8 |
| Feature #8 DNP3 — F3 Story Decomposition | **PASSED** (human-gated 2026-06-11) | 5 stories STORY-106..110; linear chain; VP placements |
| Feature #8 DNP3 — F4 Delta Implementation | **COMPLETE** 2026-06-12 | waves 35-39 / stories 106-110 ALL DELIVERED |
| Feature #8 DNP3 — F5 Scoped Adversarial + Remediation | **COMPLETE** 2026-06-12 | PR #230 e685664; 4 issues fixed; 10-pass 3/3 CLEAN |
| Feature #8 DNP3 — F6 Formal Hardening | **COMPLETE** 2026-06-12 | PR #231 a125c69; 9/9 Kani; 89% mut kill; VP-023 LOCKED |
| Feature #8 DNP3 — F7 Delta Convergence | **CONVERGED** 2026-06-12 | 5-dim convergence; 6 fresh-context passes (final 3 consecutive CONVERGED); PRs #232/#233; BC-2.15.009 v1.3 |
| Release v0.6.0 | **RELEASED** 2026-06-12 | PR #234 (release/0.6.0 → main 3e29891); fixup fb3935c; tag v0.6.0; 4 binaries (release.yml); develop merge-back 04f8ccb |
| Maintenance: Dependabot sweep (post-v0.6.0) | **COMPLETE** 2026-06-12 | 5 PRs merged (#203/#204/#207/#235/#206), 2 closed (#202 superseded, #205 deferred); develop 31d1231 |
| Feature: ARP analyzer — F1 Delta Analysis | **PASSED** (human-gated 2026-06-12) | DecodedFrame{Ip,Arp} integration, ADR-008 planned, F2→F7 authorized; artifacts: `.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md` |
| Feature: ARP analyzer — F2 Spec Evolution | **CONVERGED 3/3** (Pass 33, 2026-06-13); 33 passes total; P31/P32/P33 consecutive CLEAN; F2 strict-whole-corpus adversarial gate SATISFIED | 4-slice method; ARP delta SETTLED P9+; corpus-wide debt flushed P14-25; P26/P28/P31/P32/P33 CLEAN; P27/P29/P30 reset cycles surfaced+fixed genuine defects; trajectory: `phase-f5-adversarial/arp-f2-convergence-trajectory.md` |
| Feature: ARP analyzer — F3 Story Decomposition | **CONVERGED 3/3** (Passes 36/37/38, 38 passes total incl. post-P26/P33 consistency flushes); F3 STRICT WHOLE-CORPUS ADVERSARIAL GATE SATISFIED; F3 human gate PASSED (D-070, 2026-06-14) | STORY-111..115 (E-16, 47 pts, linear chain); 15 SS-16 BCs; waves 40-44 holdouts; HS-INDEX v1.7; wave-schedule v1.3; SS-15 fully de-NEW-ed; corpus canonical 457 pts; trajectory: phase-f5-adversarial/arp-f3-convergence-trajectory.md |
| Feature: ARP analyzer — F4 Delta Implementation | **COMPLETE** — ALL 5 STORIES DELIVERED. STORY-111 (PR #236 cced898; wave 40); STORY-112 (PR #238 10e4472; wave 41); STORY-113 (PR #239 7b7dbb2; wave 42; 9 CI green); STORY-114 (PR #240 7c0f453; wave 43; D1 spoof+MITRE 25/17+--arp-spoof-threshold); STORY-115 (PR #241 d038711; wave 44; pr-reviewer APPROVE zero blocking; 9 CI green; Step-4.5 CONVERGED; D3 storm+--arp-storm-rate+storm_findings; FINAL E-16 story). develop HEAD fee71ee. ARP Security Analyzer (E-16, issue #9) CODE-COMPLETE. | per-story TDD; waves 40-44; v0.7.0 target |
| Feature: ARP analyzer — F4 Wave-Level Adversarial Convergence | **CONVERGED 3/3 — GATE SATISFIED** (2026-06-15). Pass 1 NOT clean → D-074 + PR #242 fee71ee → clean-streak restart: P1/3 CLEAN, P2/3 CLEAN, P3/3 CLEAN (fee71ee; fresh-context; DF-BC-COMPLETENESS-SWEEP all 15 SS-16 BCs; policy rubric). Final full-corpus consistency: CONSISTENT. Trajectory: `1M→(remediated)→P1/3→P2/3→P3/3-GATE-SATISFIED`. Detail: `cycles/feature-arp-v0.7.0/arp-f4-wave-adversary-convergence-trajectory.md` | GATE SATISFIED (superseded by re-streak below) |
| Feature: ARP analyzer — F4 Holdout Evaluation | **GATE PASS** (2026-06-15). Initial run mean 0.997 (G1=0.95: D1 HIGH verdict defect → D-075 PR #243); G1 re-run = 1.0 post-fix; full corpus 15/15 mean 1.0; canonical RFC-826 frame scenario PASS; non-D1 verdicts unregressed. | PASSED |
| Feature: ARP analyzer — F4 Post-Convergence Adversary Re-Streak | **CONVERGED 3/3 — GATE SATISFIED** (re-streak on bcb1bd6; 2026-06-15). Three independent fresh-context passes; each verified field VALUES (D1 HIGH → `Verdict::Likely`) + reject path (non-Ethernet hw/proto → `Err`; D-077) + all 15 BCs' full precondition sets. Pass 3/3 solo for strict independence. Trajectory: `cycles/feature-arp-v0.7.0/arp-f4-wave-adversary-convergence-trajectory.md`. Open LOW: arp.rs:2501 `// RED: will fail if stub not wired` comment on passing STORY-114 test — fold into FU-REPO-WIDE-DOC-DEBT. | GATE SATISFIED |
| Feature: ARP analyzer — F5 Scoped Adversarial | **GATE SATISFIED 3/3** (2026-06-16, develop 079013d). Full F5 journey: P1+P2 CLEAN on bcb1bd6 → O-A LOW → human FIX → D-078 (PR #247) + D-078b (PR #248) → re-run P1 on 2d2fadf found F-1 MEDIUM VLAN false-positive → PR #249 (079013d) → 3 consecutive fresh-context CLEAN passes on 079013d. Lens: implementation-robustness/security; panic-safety, DoS/LRU caps, integration, etherparse-migration, silent-failure all CLEAN; F-1 VLAN-offset fix verified robust across all link_exts configs. Trajectory: `cycles/feature-arp-v0.7.0/arp-f5-scoped-adversarial-trajectory.md`. | GATE SATISFIED |
| Feature: ARP analyzer — F6 Formal Hardening | **COMPLETE** (PR #250, develop 6e9f2cc, 2026-06-16). 5/5 VP-024 Kani harnesses VERIFICATION:- SUCCESSFUL (Sub-A ×3, Sub-B ×1, Sub-D ×1); 46/46 project-wide; VP-024 v2.1 LOCKED (verified_at_commit=6e9f2cc; proof_file_hash deferred — FU-F6-KANI-CLEANUP). Sub-D array surrogate confirmed FAITHFUL + branch-fidelity test ADEQUATE + cfg-gate compliant (zero production-binary impact). Fuzz VP-008 decoder: 16.2M execs/0 crashes (covers O-2 QinQ/MACsec paths). Mutants ARP delta: 98.9% kill (1 benign MISSED — `<` vs `<=` tie-break in array surrogate, out of Sub-D Kani scope, by design). Security: cargo-audit 1 allowed warning (RUSTSEC-2026-0097 rand — BUILD-dep via tls-parser/phf_codegen, not runtime, not exploitable); clippy+fmt CLEAN. Code review APPROVE; security PASS. Open follow-ups recorded: FU-F6-KANI-CLEANUP (CR-001/002/003), O-2 (fuzz 16.2M partially addresses it). | COMPLETE |
| Feature: ARP analyzer — F7 Delta Convergence | **CONVERGED — 5-dim COMPLETE** (2026-06-16, develop e37ec38). (1) Regression GREEN (build/clippy/fmt clean, 1592 tests/0 fail); (2) Verification GREEN (VP-024 v2.3 LOCKED, 5/5 Kani SUCCESSFUL, fuzz 16.2M/0, mutants 98.9%); (3) Implementation/spec convergence (F4 3/3 + holdout); (4) Robustness (F5 3/3); (5) Documentation/coherence (F7 consistency CONSISTENT — 4 gaps + VP-024 v2.3 residual ALL REMEDIATED; holistic adversary PASS CLEAN). Final input-hashes: 111=3eefa35 112=26fb42d 113=f35bcfc 114=02da9e7 115=80be67e ALL MATCH. | CONVERGED |
| Release v0.7.0 | **RELEASED 2026-06-16** — PR #256 (release/0.7.0 → main); merge commit dd8e142; tag v0.7.0; GitHub Release https://github.com/Zious11/wirerust/releases/tag/v0.7.0; 4 binaries (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu); release.yml run 27645784901 SUCCESS. ARP Security Analyzer (E-16, issue #9). develop merge-back: dd8e142 (branch-protection bypass; gitflow sync; CI-verified via PR #256). | **RELEASED** |
| E-17: ARP QinQ/MACsec offset hardening (issue #253) — F1 Delta Analysis | **PASSED** (human-gated 2026-06-16) — MACsec offset investigated → NO code bug; documented-limitation evidence-backed. Full F1-F7 rigor authorized; v0.7.1 target. Artifacts: `.factory/phase-f1-delta-analysis/` | PASSED |
| E-17: ARP QinQ/MACsec offset hardening (issue #253) — F2 Spec Evolution | **SATISFIED 3/3 COMPLETE** (2026-06-16) — 3 consecutive fresh-context CLEAN passes (zero MEDIUM-or-above) on frozen baseline factory-artifacts 39f57ea. Each pass corroborated: etherparse 0.20.2 citations vs vendored source, all 10 test names, VP-024 v2.4 lock integrity, EC-009 sibling identity, §2.2 snippet-vs-shipped-code parity. Full trajectory: HIGH mis-anchor (4 governance docs) → MEDIUM symbol/tense → MEDIUM changelog traceability gap → 3/3 CLEAN. Final F2 corpus versions: BC-2.16.009 v1.9, BC-2.16.015 v1.8, arp-architecture-delta v1.19, VP-024 v2.4, verification-coverage-matrix v1.8. | COMPLETE |
| E-17: ARP QinQ/MACsec offset hardening (issue #253) — F3 Story Decomposition | **ADVERSARIAL GATE SATISFIED 3/3 (genuine, dd34205)** — STORY-116 (wave 45, QinQ coverage) + STORY-117 (wave 46, MACsec documented-limitation); 3 genuine fresh-context CLEAN passes (aeddd3a4/aa09cc4e/ab72e18d), each zero MEDIUM+; input-hash discharged (c389b39 MATCH); consistency-validator round-1 findings remediated; residual LOWs (AC-003 EC-008-vs-PC-7a citation, STORY-INDEX "68-story" parenthetical, etherparse volatile-line cites) tracked as non-blocking F4 polish. Prior "Pass 1 CLEAN / streak 1/3" record (ae977cb) VOIDED — unbacked; real adversary hung (a9f139ef). NEXT = F3 human gate. | ADVERSARIAL GATE SATISFIED — awaiting F3 human gate |
| E-17: ARP QinQ/MACsec offset hardening (issue #253) — F4 Delta Implementation | **COMPLETE** — 10 tests (4 QinQ + 6 MACsec) committed cb2bf06 on PR #258 branch test/arp-qinq-macsec-fixtures; local+CI green; no src/ delta; clippy/fmt CLEAN. | COMPLETE |
| E-17: ARP QinQ/MACsec offset hardening (issue #253) — F4 Wave-Level Adversarial Convergence | **GATE SATISFIED 3/3** (cb2bf06; 2026-06-17). Pre-remediation pass found 1 MEDIUM (Finding 1: benign-truncated test tautology window); REMEDIATED in cb2bf06 by adding independent off-by-N negative-offset diagnostics to V1/V3/QinQ-benign tests; 3 verified fresh-context CLEAN passes on hardened delta: a2c9149c (P1), afec0575 (P2), a6c3e1ba (P3); each zero MEDIUM+. Residual LOWs (V5/V6 stop short of end-to-end decode; under-count-only diagnostics) tracked non-blocking. NEXT = F4 holdout evaluation. Trajectory: `cycles/feature-arp-v0.7.0/arp-f4-wave-adversary-convergence-trajectory.md` §E-17 F4. | GATE SATISFIED |

## Session Resume Checkpoint (2026-06-17 — E-17 F4 wave-adversarial gate SATISFIED 3/3)

**Previous checkpoint (2026-06-17 — E-17 F3 adversarial gate SATISFIED 3/3 genuine) archived to:
`cycles/feature-arp-v0.7.0/session-checkpoints.md`**

### A. EXACT PIPELINE POSITION

- **Project:** wirerust. Mode: STEADY-STATE (top-level pipeline IDLE; E-17 sub-cycle IN PROGRESS).
- **E-17 "ARP QinQ/MACsec offset hardening" (issue #253): CYCLE OPEN.**
- **E-17 F1:** PASSED (human-gated 2026-06-16).
- **E-17 F2:** COMPLETE — adversarial gate SATISFIED 3/3 (2026-06-16).
- **E-17 F3:** ADVERSARIAL GATE SATISFIED 3/3 (genuine, dd34205) — F3 human gate status unknown; presumed PASSED given F4 was authorized.
- **E-17 F4 Delta Implementation:** COMPLETE — 10 tests (4 QinQ + 6 MACsec) committed cb2bf06 on PR #258 branch test/arp-qinq-macsec-fixtures; local+CI green; no src/ delta; clippy/fmt CLEAN.
- **E-17 F4 Wave-Level Adversarial Convergence:** GATE SATISFIED 3/3 (cb2bf06). Pre-remediation pass found 1 MEDIUM (Finding 1: benign-truncated tautology window); REMEDIATED in cb2bf06; 3/3 streak CLEAN on hardened delta: a2c9149c (P1), afec0575 (P2), a6c3e1ba (P3); each zero MEDIUM+.
- **E-17 F4 NEXT:** F4 holdout evaluation.
- **develop HEAD: 480f8ae** == origin/develop.
- **main HEAD: dd8e142 (v0.7.0).**
- **PR #258:** Open on branch test/arp-qinq-macsec-fixtures (commit cb2bf06); CI green.
- **factory-artifacts HEAD:** see `git -C .factory log -1 --format='%h %s'`
- **Active worktrees:** EXACTLY 2 — main repo (develop) + .factory (factory-artifacts).

### B. GATE RESULT (2026-06-17)

- E-17 F4 wave-level adversarial gate SATISFIED 3/3. Pre-remediation pass found 1 MEDIUM (benign-truncated test tautology window — Finding 1); REMEDIATED in cb2bf06 by adding independent off-by-N negative-offset diagnostics to V1/V3/QinQ-benign tests.
- Three verified fresh-context CLEAN passes on hardened delta (cb2bf06): a2c9149c (P1), afec0575 (P2), a6c3e1ba (P3); each zero MEDIUM+.
- Test delta: 10 tests (4 QinQ + 6 MACsec), all green, clippy/fmt CLEAN, zero src/ change.
- Residual LOWs: V5/V6 stop short of end-to-end decode; under-count-only diagnostics — tracked non-blocking.

### C. CARRY-FORWARD (open items)

- **E-17 F4 holdout evaluation:** NEXT STEP — run `vsdd-factory:phase-f4-holdout-evaluation` scoped to E-17 delta (PR #258, cb2bf06).
- **Residual LOWs (V5/V6):** Under-count-only diagnostics, non-blocking.
- **O-2 (deferred LOW):** dep-graph.md lines 204/586 STORY-117 label fix — schedule in a dep-graph label sweep.
- **Non-blocking F4 polish:** AC-003 EC-008-vs-PC-7a citation, STORY-INDEX "68-story" parenthetical, etherparse volatile-line cites.
- **#252** VP-024 proof_file_hash + re-lock (post-release).
- **#254** Repo-wide RED-prose doc cleanup (post-release; 71 occurrences).
- **#255** JSON enum casing → snake_case (post-release).

### D. RESUME PROCEDURE (E-17 F4 HOLDOUT — SESSION-CLEAR SAFE 2026-06-17)

**CONTEXT FOR FRESH SESSION:**
- **Project:** wirerust. Mode: STEADY-STATE (top-level). E-17 sub-cycle IN PROGRESS at F4 holdout evaluation.
- **develop HEAD:** 480f8ae == origin/develop.
- **PR #258:** test/arp-qinq-macsec-fixtures (cb2bf06); CI green.
- **main HEAD:** dd8e142 (v0.7.0).
- **factory-artifacts HEAD:** see `git -C .factory log -1 --format='%h %s'`

**Step 1 (BLOCKING):** `vsdd-factory:factory-worktree-health`

**Step 2 — Verify SHAs:**
```bash
git -C /Users/zious/Documents/GITHUB/wirerust rev-parse HEAD  # expect 480f8ae prefix
gh pr list --state open                                        # expect PR #258
```

**Step 3 — WHAT IS COMPLETE (do NOT re-do):**
E-17 F1 PASSED. E-17 F2 adversarial gate SATISFIED 3/3. E-17 F3 story decomposition COMPLETE and frozen (dd34205). E-17 F3 adversarial gate SATISFIED 3/3. E-17 F4 delta implementation COMPLETE (cb2bf06, PR #258, CI green). E-17 F4 wave-adversarial gate SATISFIED 3/3 (cb2bf06 — P1 a2c9149c / P2 afec0575 / P3 a6c3e1ba).

**Step 4 — NEXT ACTION:**
Run E-17 F4 holdout evaluation scoped to PR #258 delta (10 tests: 4 QinQ + 6 MACsec). Use `vsdd-factory:phase-f4-holdout-evaluation`.

### E. KEY ARTIFACT POINTERS

- ARP architecture delta: `.factory/specs/architecture/arp-architecture-delta.md` (v1.19)
- VP-024: `.factory/specs/verification-properties/vp-024-arp-parse-safety.md` (v2.4 LOCKED; verified_at_commit=6e9f2cc)
- Verification coverage matrix: `.factory/specs/architecture/verification-coverage-matrix.md`
- E-17 F1 delta analysis: `.factory/phase-f1-delta-analysis/`
- STORY-116: `.factory/stories/STORY-116.md` (wave 45, QinQ coverage)
- STORY-117: `.factory/stories/STORY-117.md` (wave 46, MACsec documented-limitation)
- ARP cycle artifacts: `.factory/cycles/feature-arp-v0.7.0/`
- F4 convergence trajectory: `.factory/cycles/feature-arp-v0.7.0/arp-f4-wave-adversary-convergence-trajectory.md` §E-17 F4
- F3 convergence trajectory: `.factory/phase-f5-adversarial/arp-f3-convergence-trajectory.md` §E-17 F3 section
- Archived checkpoints: `.factory/cycles/feature-arp-v0.7.0/session-checkpoints.md`

## Decisions Log

D-001..D-054 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md` (D-047..D-054 in Feature #8 / v0.5.0 section).

| ID | Decision | Date |
|----|----------|------|
| D-055 | Feature #8 F3 human gate PASSED — 5 stories accepted; VP placements; strictly-linear chain. F4 TDD authorized. | 2026-06-11 |
| D-056 | STORY-106 DELIVERED — PR #225 d0f3586. VP-023 4/4 Kani SUCCESSFUL. | 2026-06-11 |
| D-057 | STORY-107 DELIVERED — PR #226 ebb4751. Carry-walk gate-before-count; STORY-106 frames wire-valid. | 2026-06-11 |
| D-058 | STORY-108 DELIVERED — PR #227 9c03fde. 5-pass adversarial 3/3 CLEAN. DRIFT-DNP3-DIRECTION-001 recorded. | 2026-06-11 |
| D-059 | STORY-109 DELIVERED — PR #228 34443f9. 13-pass 3/3 CLEAN; MitreTactic::IcsImpact; VP-007 seed. | 2026-06-12 |
| D-060 | STORY-110 DELIVERED — PR #229 ddfa576. Rule 6 + CLI flags + VP-004 oracle. F4 COMPLETE. | 2026-06-12 |
| D-061 | Feature #8 F5 COMPLETE — PR #230 e685664. 4 issues fixed (DIR-bit P0; unexpected-source P0; IcsImpact display; resync). 10-pass 3/3 CLEAN. | 2026-06-12 |
| D-062 | Feature #8 F6 HARDENED — PR #231 a125c69. 9/9 Kani; 89% mut; 3.19M fuzz/0; VP-023 LOCKED v1.5; VP-004 relocked. 4/4 F6 obligations SATISFIED. | 2026-06-12 |
| D-063 | Feature #8 F7 CONVERGED — 5-dim delta; 6 fresh-context adversarial passes (final 3/3 CONVERGED); F-S2-001/F-S1-001/F-PG-001/F-CC-001..004 remediated (PRs #232/#233). develop f217f27. | 2026-06-12 |
| D-064 | v0.6.0 RELEASED — gitflow release/0.6.0 → PR #234 → main 3e29891; fixup fb3935c; tag v0.6.0; GitHub Release WITH 4 binaries (release.yml auto-build); develop merge-back 04f8ccb. DNP3 TCP analyzer is the headline feature. | 2026-06-12 |
| D-065 | Dependabot sweep post-v0.6.0 COMPLETE — #203 serde_json/#204 assert_cmd/#207 clap/#206 rayon routine bumps merged; #235 manual SHA-pin actions/checkout 6.0.3 (replacing tag-ref #202); #205 etherparse 0.16→0.20 closed and deferred as migration story (new drift DRIFT-ETHERPARSE-0.20-MIGRATION-001). develop 31d1231. | 2026-06-12 |
| D-066 | Feature ARP analyzer F1 gate APPROVED — full F1-F7, release target v0.7.0. DecodedFrame{Ip,Arp} integration (ADR-008); ArpAnalyzer bounded IP↔MAC table; etherparse 0.20 sub-delta A. SS-16 (18-24 BCs), VP-024, ADR-008, E-16 (5-6 stories). MITRE T0830+T1557.002. Detections: spoof/cache-poison + GARP + storm/rate + research-agent additional. DRIFT-ETHERPARSE-0.20-MIGRATION-001 folded in. | 2026-06-12 |
| D-067 | IcsImpact Display adjudication — canonical Display = "Impact" (spec correct; BC-2.10.002 PC3/PC4, PRD §85/823, cap-10, spec-changelog unanimous). src/mitre.rs:91 "Impact (ICS)" is DEVIANT (introduced F-F5-002 as "No BC change" tactical test fix). " (ICS)" suffix does NOT break merge-by-name report bucketing (terminal.rs render_findings_grouped keys on MitreTactic enum variant, not Display string); severity LOW (terminal section-header label only). F2 SPEC CHANGE: NONE — F2 3/3 strict-whole-corpus convergence preserved unaffected. Fix folded into STORY-114 (obligations: 1-line mitre.rs:91 fix; HS-008:75 "Impact (ICS)"→"Impact"; Display unit test; two-bucket enum-level report test). F2→F3 gate condition SATISFIED. **SUPERSEDED BY D-069.** | 2026-06-13 |
| D-068 | Benign gratuitous ARP emits mitre_techniques: [] (LOW/Anomaly severity); T0830 + T1557.002 apply ONLY when GARP conflicts with binding table (BC-2.16.014). Research-backed: MITRE ATT&CK v19.1 T1557.002/DET0387 + T0830; arpwatch/Zeek/Suricata all gate techniques on conflict-detection. Corrected latent over-tagging defect in BC-2.16.003 (→v1.7) and ADR-008 (→v2.0). Propagated to §3.3/STORY-113 AC-003, holdouts, and error-taxonomy. | 2026-06-14 |
| D-069 | IcsImpact Display canonical = "Impact (ICS)" (distinct from Enterprise "Impact" TA0040). SUPERSEDES D-067. Research-backed: MITRE TA0040 (Enterprise Impact) vs TA0105 (ICS Impact) are distinct tactic families; WCAG 2.4.6 requires unique headings/labels. src/mitre.rs:91 "Impact (ICS)" is CORRECT — not deviant. STORY-114 D-067 revert obligations (F3-OBL-STORY114-001/002/003) REVOKED. 2 shipped DNP3 F5 distinctness tests preserved (they test enum-variant identity, not Display string equality). Spec side corrected: BC-2.10.002 (→v1.5), PRD §85/882, ADR-007, arp-architecture-delta §5.0. | 2026-06-14 |
| D-070 | Feature ARP F3 human gate PASSED (2026-06-14) — STORY-111..115 (E-16, 47 pts) accepted as-is; F3 strict whole-corpus adversarial convergence SATISFIED (3/3, Passes 36/37/38; 38 passes total + 3 consistency flushes). F4 delta-implementation AUTHORIZED: per-story TDD on the linear chain STORY-111→112→113→114→115; release target v0.7.0. etherparse 0.16→0.20 migration folded into STORY-111 (sub-delta A). Human review questions (scope/MITRE/linear-chain/etherparse) — no changes requested; approved as-is. | 2026-06-14 |
| D-073 | STORY-111 DELIVERED — PR #236 MERGED to develop (merge commit cced898; repo allows merge-commits only). etherparse 0.20 migration + DecodedFrame{Ip,Arp} enum + ArpFrame struct + decode_packet→Result<DecodedFrame> + symmetric-unreachable ARP decode (D-072) + non-panicking extract_arp_frame placeholder + BC-2.02.009 v1.7 + VP-008 fuzz-harness return-type update. 53 test suites green; clippy/fmt clean; Step-4.5 adversarial 3/3. CI Format failure fixed by aligning local toolchain to CI rolling-stable (rustfmt 1.8.0→1.9.0). pr-reviewer APPROVE. Wave 40 complete. NEXT = STORY-112 (extract_arp_frame + ArpAnalyzer stub + main.rs DecodedFrame wiring + VP-024 Sub-A Kani). | 2026-06-14 |
| D-072 | F4-surfaced ARP decode design inconsistency (2026-06-14) — STORY-111 implementation revealed arp-architecture-delta §2.2 contained an internally-inconsistent + non-type-implementable lax design (BLOCK 1 'lax_ip_triple routes ARP / NOT unreachable!' vs authoritative BLOCK 2 'decode_packet intercepts ARP'). lax_ip_triple returns IpTriple and cannot route ARP. Architect ruled SYMMETRIC design authoritative: decode_packet routes ARP in both strict Ok(slice) and lax Err(SliceError::Len) arms; both strict_ip_triple AND lax_ip_triple have provably-dead unreachable! ARP arms; VP-008/VP-024 Sub-A no-panic via decode_packet interception + panic-free extract_arp_frame. Reconciled: arp-architecture-delta v1.16, ADR-008 Decision 3 v2.1, BC-2.02.009 v1.7, BC-2.16.015 v1.3, STORY-111 v1.4 (hash d05149f), STORY-112 v1.3 (hash 8a4d566). SUPERSEDES BC-2.16.015 v1.2 'lax NOT unreachable' fix (which had aligned to the inconsistent BLOCK-1 prose). STORY-111 GREEN implementation was correct throughout and matches the reconciled design — no code change. 2nd F4-surfaced spec defect after D-071; reinforces strict-TDD value. | 2026-06-14 |
| D-071 | F4-surfaced STORY-111 decomposition fix (2026-06-14) — strict-TDD stub-architect Red-Gate (BC-5.38.005 self-check) caught that STORY-111 ACs (001/002/004/007/008) asserted STORY-112's extract_arp_frame end-to-end ARP-decode behavior, unsatisfiable within STORY-111's §6 scaffolding scope and duplicative of STORY-112 AC-006/007/004. Re-scoped STORY-111→v1.1 (scaffolding-only ACs AC-003/005/005b/006/009/010 + AC-005b non-panicking extract_arp_frame placeholder preserving VP-008); added STORY-112 AC-012→v1.1 (decode_packet-level Err("Non-Ethernet/IPv4 ARP frame")) closing the one coverage gap. BC-2.02.009 unedited (primary STORY-111; ARP-Ok postcondition behaviorally satisfied in STORY-112 — story-level framing). Both stories input-hash MATCH (d5bda72/268f53f — body-only edits). 38 strict F3 passes did not catch this (AC-satisfiable-within-dependency-scope feasibility check, not spec-internal-consistency) — validates strict-TDD value. Scoped adversarial re-review of STORY-111/112 in progress before resuming TDD. Worktree stub commit 4e22ef9 was based on old over-scoped STORY-111 and must be re-aligned (extract_arp_frame todo!()→non-panicking placeholder). **F4 scoped post-fix adversarial re-review (2026-06-14) surfaced 7 findings (1 HIGH: BC-2.16.015 lax_ip_triple unreachable! mis-anchor — sibling-propagation gap from BC-2.02.009 v1.6 correction, would have caused a reachable VP-008/VP-024 Sub-A violating panic; 2 MED + 4 LOW: AC-seam/type/count issues). ALL 7 remediated: BC-2.16.015→v1.2 (lax unreachable! → explicit routing; Architecture Anchors corrected), STORY-111→v1.2 (AC-005 seam + AC-005b type + Task-8 + coverage-map), STORY-112→v1.2 (AC count AC-001..AC-011→AC-001..AC-012; input-hash 268f53f→c8c1a64). BC-2.02.009 anchoring judged coherent — no BC split needed. BC-2.16.015 lax-unreachable mis-anchor shows the BC-2.02.009 v1.6 sibling-sweep did not reach SS-16 sibling BCs — cross-subsystem sibling sweep gap. Confirming scoped re-review in progress; on clean → resume STORY-111 TDD.** | 2026-06-14 |
| D-074 | Reject `--arp-storm-rate 0` and `--arp-spoof-threshold 0` at CLI with fail-fast `anyhow::bail!`. ARP comparisons are inclusive (`>=`) so 0 is a degenerate always-fire condition — mirrors Modbus reject-0 precedent; reconciles with DNP3 (strict `>`, accepts 0) under the invariant "accept 0 only where comparison is strict; reject where inclusive." Research-agent validated HIGH confidence (`.factory/research/arp-threshold-zero-convention.md`). Surfaced by F4 wave-level adversarial Pass 1 finding F-ARP-F4P1-001 (MEDIUM). Spec: BC-2.16.008 v1.7→v1.8 (EC-006), BC-2.16.012 v1.2→v1.3 (PC2+EC-004), BC-2.16.013 v1.2→v1.3 (PC2+EC-004). Stories: STORY-114 v1.1→v1.2 (AC-006+EC-014), STORY-115 v1.1→v1.2 (EC-011+AC-011). Code: PR #242 merged develop fee71ee (src/main.rs +10 lines; tests/bc_2_16_d074_arp_threshold_zero_tests.rs 4 tests; code review APPROVE 0 findings; 9 CI green). | 2026-06-15 |
| D-075 | HIGH-confidence D1 ARP-spoof finding carries `Verdict::Likely` (was `Verdict::Possible`). Holdout-caught defect: static adversary 3/3 scenarios flagged Likely; consistency-validator missed it (checked structure not field value). BC-2.16.004 L45/L74/L118. PR #243 (merge 4ee7a9d). | 2026-06-15 |
| D-076 | D-075 regression-test doc-comments corrected from present-tense RED prose to past-tense regression-guard framing. Recurrence of DF-GREEN-DOC-TENSE-SWEEP sub-rule d / PG-ARP-F4-REDTEST-DOC-TENSE — codified policy text alone did not prevent recurrence; agent-prompt/hook strengthening needed (open follow-up, see PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE in drift items). PR #244 (merge 52437f8). | 2026-06-15 |
| D-077 | CRITICAL: `extract_arp_frame` now rejects non-Ethernet hw type (`hw_addr_type != ETHERNET`) and non-IPv4 proto type (`proto_addr_type != IPV4`). BC-2.16.001 PC2/PC3, BC-2.16.009 PC3a/3b/EC-001/EC-002. Half-implemented D11 security boundary — crafted valid-size/wrong-type ARP admitted into detection pipeline. Surfaced by F4 3-pass adversary re-streak. Missed by 4 prior adversary passes + holdout + static adversary (impl+unit+Kani all consistently omitted type check; self-consistent omission invisible to structural review). F-2 LOW: GARP-conflict summary now states "with binding conflict" (BC-2.16.014 PC1). Security review PASS (CWE-20, panic-free). PR #245 (merge 6abcd8f). F4 adversary counter RESET to 0/3; re-streak restarted on 6abcd8f. | 2026-06-15 |
| D-078 | F5 O-A finding adjudicated FIX (human 2026-06-15/16): lax `None` arm (lax.net==None, stop_err==Layer::Arp) now bounds-checked-peeks raw 8-byte ARP fixed header (offset from lax.link Ethernet2); bad type/size → "Non-Ethernet/IPv4 ARP frame" → D11; valid-but-truncated/non-Ethernet → "truncated ARP frame" decode-error. Closes CWE-693 D11-evasion; bounds-safe (security review CLEAR). Spec corrected twice — initial hypothesis "lax builds slice + extract None" was impossible; actual mechanism = None-arm raw peek. BC-2.16.009 v1.4→v1.6, BC-2.16.015 v1.3→v1.5, STORY-111 v1.4→v1.6, STORY-112 v1.4→v1.6. PR #247 (merge 92c1561). | 2026-06-15/16 |
| D-078b | Completion — sibling lax `Some(LaxNetSlice::Arp)` arm also routes extract_arp_frame returning None to "Non-Ethernet/IPv4 ARP frame" → D11 (defensive path-independence). Arm is structurally unreachable via integration (etherparse raises SliceError::Len before populating Arp); documented in tests/bc_2_16_d078b_lax_some_arm_tests.rs. Plus decoder.rs doc-comment correctness sweep (3 loci). F5 streak VOIDED by D-078/D-078b code change; counter reset to 0/3. PR #248 (merge 2d2fadf). | 2026-06-16 |
| D-F1 | F5 Pass 1/3 (re-run on 2d2fadf) found F-1 MEDIUM: D-078 lax None-arm peek hard-coded Ethernet2 offset 14, ignoring `lax.link_exts` — for VLAN-tagged ARP, peeked the 802.1Q TCI bytes as ARP htype → false-positive D11 (fix-induced regression: D-078 LOW fix cascaded into MEDIUM false-positive). Fix: `arp_offset = 14 + lax.link_exts.iter().map(|ext| ext.header_len()).sum()` (correct for single VLAN/QinQ/MACsec). Security review CLEAN. Spec: BC-2.16.015 v1.5→v1.6, BC-2.16.009 v1.6→v1.7. New tests: tests/bc_2_16_d078_vlan_offset_tests.rs (4). F5 counter reset to 0/3 (re-run in progress on 079013d). PR #249 (merge 079013d). Meta-lesson: LOW-fix (O-A) cascaded 3 PRs + MEDIUM regression; fix-induced-regression risk should weigh into whether a LOW finding is worth fixing vs documenting. | 2026-06-16 |

## Blocking Issues

None open.

## Drift Items / Tech Debt Pointers

All items require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Full tech-debt register: `.factory/tech-debt-register.md`.

| ID | Summary | Status |
|----|---------|--------|
| ADV-HS043-P02-MED-001 | Idle-flow expiry monotonic watermark stalls on multi-epoch captures | ACCEPTED — gated on live-capture support |
| O-07 | rayon declared in Cargo.toml but unused | OPEN P2 |
| O-08 | dns.rs module doc-comment stale | OPEN P3 |
| F-W25-S088-P6-001 | AC-004 warning .contains() — weaker than count-assertion | OPEN — target next main.rs touch or accept |
| RUSTSEC-2026-0097 | rand 0.8.5 unsound (transitive via tls-parser→phf 0.11); upstream-only fix | ACCEPTED-TRANSITIVE |
| FE-001 | pcapng input format not supported — v2 idea | deferred / v2 |
| ACTION-PIN-001 | dtolnay/rust-toolchain @stable/@nightly exempt in pin gate | OPEN P3 |
| PCAP-CORPUS-001 | E2E pcap test-corpus storage backend — PR #221 landed; large pcaps gitignored | TABLED — human decision pending |
| MITRE-V19-REMAP-001 | T0855/T0856 remap; PR #223 develop; PR #224 main | CLOSED — RELEASED in v0.5.0 |
| DRIFT-F2-COUNT-001 | Stale "15 seeded IDs" count in BC-2.10.006.md, prd-supplements, HS-008/009 | DEFERRED |
| DRIFT-SUPERPOWERS-001 | docs/superpowers/ carries stale pre-F2 catalog | DEFERRED |
| SEC-106-001..002 | CWE-129 gate-before-count; CWE-400 MAX_MASTER_ADDRS cap | SATISFIED |
| STORY-107-CARRY-001 | BC EC-004/EC-006/PC4 deferrals; multi-block indexing | SATISFIED |
| DRIFT-DNP3-DIRECTION-001 | source_ip resolution port-heuristic-only; direction-aware deferred post-v0.6.0 | DEFERRED |
| DRIFT-MITRE-EMITTED-LABEL-001 | kani EMITTED_IDS T0835/T0831 over-label; system-level | DEFERRED LOW |
| DRIFT-BC-2.15.024-EC006-PROSE-001 | EC-006 prose vs BC-2.15.009 PC5 conflict; PO prose-refresh | DEFERRED LOW |
| DRIFT-SEMGREP-001 | semgrep absent; manual CLEAN; non-blocking | DEFERRED LOW |
| DRIFT-ENGINE-CHECKOUT-GUARD-001 | adversary dispatch template missing checkout-guard; engine fix needed | ENGINE-NOTE HIGH |
| DRIFT-ENGINE-PRMGR-REPORT-001 | pr-manager omitting consolidated report on 4/5 PRs; engine fix needed | ENGINE-NOTE MEDIUM |
| DRIFT-ENGINE-RELEASECONFIG-STALE-001 | release-config.yaml human-prose fields refreshed this burst (human_approval_prompt version-agnostic; test counts updated to 1496 / 23 VPs; release.yml stale note corrected); engine template follow-up (version_sources) DEFERRED | PARTIALLY RESOLVED |
| DRIFT-DNP3-DOC-T0814-COMPLETENESS-001 | RESOLVED in v0.6.0 — README/CHANGELOG T0814 ENABLE/DISABLE_UNSOLICITED trigger sources added on release branch; also corrected pre-existing README "—" technique error for unsolicited-response row → T0814 | RESOLVED |
| DRIFT-BC-INPUTHASH-TBD-001 | all 24 SS-15 BC files carry input-hash:TBD; compute-input-hash scopes to .factory/stories/ per CLAUDE.md; by-design; known/accepted, non-blocking | BY-DESIGN LOW |
| PG-F7-001 | BC version bump must re-stamp all consuming stories in same burst; F4/F5/F6 gates run live compute-input-hash --scan not trust STATE value. Policy candidate: DF-INPUT-HASH-CANONICAL-001 sub-rule. Backing: lessons.md PG-F7-001. | DEFERRED — next feature cycle |
| PG-F7-002 | After behavior-changing adjudication, grep + re-validate all holdout assertions on the changed path against impl. Policy candidate: F5 remediation playbook step. Backing: lessons.md PG-F7-002. | DEFERRED — next feature cycle |
| PG-F7-003 | Adjudicating agent must Read() current BC text and verify each Invariant before writing "BC needs no update". Engine agent-prompt note. Backing: lessons.md PG-F7-003. | DEFERRED — engine agent-prompt note |
| PG-F7-004 | DF-SIBLING-SWEEP v5: BC Invariant text change must sweep BC-INDEX titles + consuming-story body notes. Policy candidate: DF-SIBLING-SWEEP-001 protocol-BC sub-rule. Backing: lessons.md PG-F7-004. | DEFERRED — next feature cycle |
| PG-F7-005 | Story status (body frontmatter + STORY-INDEX) advances to completed at merge, not draft. Add to per-story delivery close-out. Backing: lessons.md PG-F7-005. | DEFERRED — engine workflow note |
| PG-F7-006 | Shipping a feature moves README planned→implemented + adds CHANGELOG Unreleased entry at delivery, not release scramble. Backing: lessons.md PG-F7-006. | DEFERRED — engine workflow note |
| PG-F7-007 | Agents must check gh run list for in-flight tag-triggered workflows before reporting missing CI/release assets. Backing: lessons.md PG-F7-007. | DEFERRED LOW — engine devops checklist note |
| DRIFT-ETHERPARSE-0.20-MIGRATION-001 | etherparse 0.20 adds Arp variants to NetSlice/LaxNetSlice/InternetSlice; non-exhaustive match at src/decoder.rs:210,232. Folded into ARP analyzer feature cycle (D-066, sub-delta A). | IN-PROGRESS — ARP feature cycle |
| PG-ARP-F2-003 | Pass-14 field-rename sweep scoped to .factory/specs/ only — MISSED .factory/holdout-scenarios/ sibling layer; 16 HS files caught at Pass 15. DF-SIBLING-SWEEP must include holdout-scenarios in propagation perimeter for any Finding-schema change. | DEFERRED — policy codification |
| PG-ARP-F2-004 | PO burst appended second `version:` YAML frontmatter key (inv-01) instead of replacing existing one, introducing malformed YAML caught at Pass 15 (C-04). Version bumps must replace-in-place; pre-commit dup-key lint recommended. | DEFERRED — policy codification |
| PG-ARP-F2-005 | Sweep globs must cover sibling naming variants (chunk*-eval.md missed chunk3-reeval.md; caught Pass 16). Partial-fix discipline: when fixing one of N instances of same defect, enumerate ALL siblings before committing (ADR-005 :74 missed after :108 fixed; api-surface STORY-114 introduced when arp-architecture-delta already cited STORY-111). | DEFERRED — policy codification |
| PG-ARP-F2-006 | Holdout-scenarios carry count assertions (tactic/seeded/emitted/cat-only) that drift when feature cycles change the MITRE catalog. HS-008/009/025 carried greenfield-era counts (16 tactics/15 seeded/5 emitted/9 cat-only) not swept across DNP3 cycle. F-cycle close-out must sweep holdout count-assertions. Candidate: extend DF-CANONICAL-FRAME-HOLDOUT-001 / holdout-maintenance policy. | DEFERRED — policy codification |
| PG-ARP-F2-007 | src-line-anchor drift class — feature cycles that insert code into a shared file (dispatcher.rs via Modbus/DNP3) leave EVERY citing BC's anchors stale. F-cycle close-out must re-run anchor-resync sweep across ALL BCs citing touched src files (dispatcher.rs→ss-04/ss-05; mitre.rs→ss-10 [done]; findings.rs→ss-09; reassembly/http/tls→ss-04/06/07 [verify P19]). Candidate: anchor-drift lint or F-cycle anchor-resync checklist. | DEFERRED — policy codification |
| PG-ARP-F2-008 | Under STRICT zero-any-severity whole-corpus, each remediation burst can introduce ~1-2 new trivial propagation/whitespace items (version-pin lag, blank-line residue) the next pass flags — asymptotic. Mitigation: drop brittle current-state version-pins (done P22 D-01), and treat sustained 0 CRIT/HIGH/MED as practical convergence if LOW-cosmetic churn persists. (Human elected strict 3/3; continuing per directive.) Corpus substantively converged as of P22. | NOTED — asymptotic LOW churn; version-pin hardening done |
| PG-ARP-F2-009 | F5 FlowKey-accessor fix (v2.2) swept 5 of 7 ss-14 direction-resolution BCs but missed 018/020; sibling-sweep completeness for code-vs-spec API fixes must enumerate ALL sibling BCs in same subsystem before closing. STORY input-hash dup-key (TBD placeholder + real hash both present) is a new frontmatter-validity defect class — pre-commit dup-key lint recommended. Caught at Pass 30; REMEDIATED. | DEFERRED — policy codification |
| DRIFT-PRD-V120-MBAPFRAMER-001 | PRD v1.20 delta:285 "C-23 was MbapFramer" historical rationale was factually wrong — no MbapFramer component ever existed; ss-15/DNP3 was renumbered C-23→C-24 when ARP took C-23. | RESOLVED — PRD v1.22 corrected MbapFramer prose (Pass-22 burst 2026-06-14) |
| F3-OBL-STORY114-001/002/003 | D-067 obligations (mitre.rs:91 "Impact"→"Impact (ICS)", HS-008 alignment, test obligations). **ALL REVOKED by D-069 (2026-06-14): src/mitre.rs:91 "Impact (ICS)" is CORRECT — canonical Display; no revert required. VP-007 obligation in STORY-114 unchanged.** | REVOKED — superseded by D-069 |
| DNPXX-SOURCE-RENAME-001 | src constant `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` is an ugly placeholder-style name shipped in v0.6.0; candidate code-cleanup rename DNPXX_→DNP3_ (6 files: dnp3.rs/cli.rs/main.rs/tests + arp-delta + STORY-110) — OUT OF F3 SCOPE (spec/story only); defer to a code-quality maintenance sweep. Requires DF-VALIDATION-001 research-agent validation before GitHub issue filing. | DEFERRED LOW |
| DF-SIBLING-SWEEP-CROSS-SS-001 | F-cycle BC-invariant corrections that change routing semantics shared across subsystems (e.g., strict/lax unreachable! vs explicit-routing pattern in SS-02+SS-16) MUST sweep cross-subsystem sibling BCs, not only the originating subsystem. BC-2.02.009 v1.6 sibling-sweep missed BC-2.16.015 SS-16 sibling; caught at F4 scoped re-review as HIGH would-be-panic. Candidate: extend DF-SIBLING-SWEEP-001 with a cross-subsystem enumeration rule for shared decode-architecture invariants. (PG-ARP-F4-SIBLING-SWEEP-CROSS-SUBSYSTEM) | DEFERRED — policy codification |
| DRIFT-ARP-STORY-113-115-HASH-STALE | STORY-113/114/115 input-hashes stale after arp-architecture-delta v1.16 (D-072): stored a767d96/e2f1c95/5ca9835 vs computed 7c61bae/5705a10/2e0eca2. Re-stamp with `bin/compute-input-hash --write` before STORY-113 delivery; verify no stale "lax NOT unreachable" framing first. Non-blocking for STORY-112. | CLOSED — re-stamped 2026-06-15, commit 68885d4; all three MATCH |
| PG-ARP-F4-REDBANNER-SWEEP | RED-gate banner sibling-sweep missed across 3 successive comment-fix bursts (module docstring fixed but per-test banners + AC-004 block left stale). Recurrence of DF-SIBLING-SWEEP-001 in the doc-comment dimension. Candidate: extend sibling-sweep checklist to enumerate per-test section banners + doc-comments as explicit targets when module-level status changes. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — policy codification follow-up |
| PG-ARP-F4-PRECLEAR-PROPAGATION | Orchestrator propagated a prior adversary pass's "leave as-is" pre-clearance into a fix dispatch (AC-004 banner), which a later fresh pass overturned as HIGH. Lesson: fix dispatches MUST NOT pre-clear regions based on an earlier pass's judgment; each fresh adversary examines the full perimeter pre-clearance-free. Candidate: DF-ADVERSARY-METHODOLOGY-001 language extension. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — policy codification follow-up |
| PG-ARP-F4-GUARD-WORDING | Checkout-guard premise "main repo does NOT have this function" was inaccurate (main repo has the STORY-111 None placeholder). Guard must key on BODY CONTENT (placeholder None vs real extraction) / the transitional error string, not function presence. Candidate: extend DF-ADVERSARY-CHECKOUT-GUARD-001. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — extend DF-ADVERSARY-CHECKOUT-GUARD-001 |
| PG-ARP-F4-DEMO-LEAK | Demo-recorder committed 4 gif+webm+tape sets to the develop-bound worktree branch under `.factory-demos/STORY-112/`, dodging the `.factory/` gitignore (different dir name). Caught by pre-PR diff inspection; commit 76bdf16 dropped; `.factory-demos/` added to .gitignore (bec7a76, PR #238). Lesson: demo evidence belongs ONLY on factory-artifacts (.factory/demo-evidence/). Demo-recorder dispatch must not commit demo artifacts to develop-bound branches. Orchestrator must run pre-PR binary-leak diff check. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — demo-recorder template + orchestrator pre-PR check |
| PG-ARP-F4-PRMGR-MERGE-SHORTSTOP | RECURRENCE #6 (PR #241 STORY-115): pr-manager again halted at step 6 (APPROVE) without executing steps 7-9; required orchestrator SendMessage to complete. 6/6 (100%) recurrence on EVERY ARP-feature PR (#236/#238/#239/#240/#241 + DNP3 F5). Confirmed agent-prompt defect. Per DF-VALIDATION-001: research-agent validation required before GitHub issue. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — file agent-prompt-defect post DF-VALIDATION-001 research-agent validation; DF-PR-MANAGER-COMPLETE-001 escalated CRITICAL |
| PG-ARP-F4-INVERTED-TDD | STORY-113: implementer changed production reporter (json.rs) to satisfy a mis-named test instead of flagging and stopping. Correct response: if a test demands a production change that contradicts a BC, STOP and surface it — do NOT change production. Caught by orchestrator reading BC-2.11.001/BC-2.16.010 Inv4 before adversary dispatch. Candidate: implementer prompt mandate. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — implementer dispatch template note |
| PG-ARP-F4-PROXY-COUNTER-TEST | STORY-113 F-113-01: AC-011 finding-emission verified via proxy counter (malformed_findings increment) that passed against an impl emitting no Finding. Candidate: adversary/review axis requiring finding-emission ACs to assert on Finding object (confidence/category/evidence), not a proxy counter. RISK: could recur in STORY-114/115 D1/D3 finding-emission ACs — apply proactively. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — adversary dispatch template axis; apply to STORY-114/115 |
| PG-ARP-F4-STALE-SKELETON-DOC | STORY-113 O-4: stale "skeleton/Red-Gate stubs/todo!()-bodies" doc-comments survived into GREEN commit (recurrence of PG-ARP-F4-REDBANNER-SWEEP from STORY-112). Candidate: GREEN-commit checklist step to sweep module/test-file headers from transitional language; adversary doc-accuracy axis enumeration. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — implementer GREEN-commit checklist; adversary doc axis |
| PG-ARP-F4-GREEN-DOC-TENSE | HIGH RECURRENCE (~7x ARP feature): TDD-phase doc-comments (RED-gate/scaffold/stub/"uncalled todo!()"/stale-count) written during RED phases NOT converted at Green step. Recurred: STORY-112 (PG-ARP-F4-REDBANNER-SWEEP), STORY-113 (O-4), STORY-114 F-1/F-2/F-3, STORY-115 newly-added regression test doc-comments (C1/GARP-flood + F-1/LRU). Sub-rule PG-ARP-F4-REDTEST-DOC-TENSE: regression-test prose must be in REGRESSION-GUARD framing from start. Codified DF-GREEN-DOC-TENSE-SWEEP in lessons.md + policies.yaml (2026-06-15 v1). | CODIFIED — policy DF-GREEN-DOC-TENSE-SWEEP in policies.yaml (v1 added 2026-06-15); sub-rule REDTEST-DOC-TENSE added |
| PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE | PG-ARP-F4-REDTEST-DOC-TENSE recurred in D-075 regression test (PR #243) despite codification. Codified policy text alone insufficient — needs agent-prompt/hook strengthening: test-writer must write regression-guard framing from the start; implementer GREEN-sweep must check the fix's OWN new test comments. Open self-improvement epic or justified deferral. | OPEN — agent-prompt/hook strengthening needed; self-improvement epic |
| PG-ARP-F4-TYPE-BRANCH-NARROWING | NEW [type-branch-narrowing]: impl + unit tests + (deferred) Kani harness consistently omitted hw/proto type-reject branch (D-077), making the omission self-consistent and invisible to structural review across 4 adversary passes AND holdout. Lesson: DF-BC-COMPLETENESS-SWEEP must cross-check EACH BC's FULL precondition/edge-case set against code (negative/reject branches), not just happy-path + present structure. Strongest evidence yet for holdout + multi-pass fresh-context re-streak catching what single-perimeter review misses. VP-024 Sub-A Kani harness (currently todo!()) MUST cover type-field rejection (not just size) when filled in F6. | OPEN — DF-BC-COMPLETENESS-SWEEP policy extension; Kani note for F6 |
| PG-ARP-F4-MULTIPASS-VALUE | POSITIVE LESSON: GARP-storm bypass (C1 in STORY-115; whole-attack-class gap, detect_storm unreachable for all GARP) was MISSED by pass 1 but CAUGHT by passes 2 and 3 via DF-BC-COMPLETENESS-SWEEP + GARP/D3 interaction analysis. Direct evidence for retaining 3-fresh-pass requirement (BC-5.39.001) + BC-completeness sweep. Detail: lessons.md PG-ARP-F4-MULTIPASS-VALUE. | DOCUMENTED — positive lesson; no policy change |
| FU-JSON-CASING | Align serde enum casing to snake_case (ECS/OCSF best-practice; maintainer chose snake_case). Governed cross-cutting JSON-contract change; touches BC-2.09.004 / BC-2.11.001 / ADR-0003. | FILED #255 — post-release |
| FU-BC-2.10.007-MARKER | BC-2.10.007 PLANNED marker for technique_tactic; siblings .005/.008 already at 25/17. | FIXED — BC-2.10.007 v1.8 de-PLANNED 23→25; factory-artifacts commit 147aa63 (2026-06-16) |
| FU-STORM-NEW-ATTR | src/analyzer/arp.rs ~line 272 doc mis-attributes storm_rate param. | DROPPED — NOT GENUINE; every storm_rate reference already correctly attributes to STORY-115 (validated 2026-06-16) |
| PG-ARP-F4-DOCSWEEP-OVERREACH | STORY-114 remediation doc-sweep over-reached to 13 out-of-scope files (modbus/dnp3/reassembly/csv). Reverted; scope restored to 7 story-scoped diff files. Lesson: remediation dispatches MUST scope greps+edits to `git diff develop..HEAD --name-only` only. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — remediation dispatch template hardening |
| FU-REPO-WIDE-DOC-DEBT | 13 test files (bc_2_15_110, bc_2_14_105, bc_2_14_103, modbus_detection, modbus_parse, dnp3_detection, dnp3_parse_core, dnp3_flow_state, dnp3_f5_remediation, reassembly_engine, reassembly_flow, reassembly_segment, reporter_csv) carry stale RED-gate prose from prior feature cycles. Schedule standalone docs chore PR after STORY-114 merges. Do NOT bundle into a feature story. | REGISTERED — post-STORY-114-merge chore |
| BC-2.10-COUNT-POSTMERGE | BC-2.10.005 / BC-2.10.008 25/17 markers; BC-2.10.007 PLANNED residual. | RESOLVED — .005 v1.11/.008 v1.13 already at 25/17; .007 fixed via FU-BC-2.10.007-MARKER above (2026-06-16) |
| PG-ARP-FIXBURST-CONSUMER-SWEEP | NEW [fixburst-consumer-sweep]: VP-024 v1.8 harness rename (O-1) didn't sweep its 11 consuming artifacts (DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 dim 3 not applied at the rename burst); resolved by reverting rename via PR #246. Lesson: any canonical-symbol rename must grep+update ALL consumers in the same burst, or avoid cosmetic renames entirely. | OPEN — policy codification follow-up |
| PG-ARP-FIX-MECHANISM-FIRST | F5 O-A adjudication: spec for D-078 was written from incorrect mechanism hypothesis ("lax builds slice + extract None" is impossible) before code mechanism was verified; caused two rounds of spec+story correction (BC v1.4→v1.6) and sibling-seam (D-078b) discovered only at PR review. F-1 (PR #249) strengthens the lesson: a fix that hand-rolls offset/parsing logic (vs delegating to the library) MUST be stress-tested against the library's full input model (here: lax.link_exts / VLAN). Meta-lesson: LOW-severity O-A fix cascaded into 3 PRs + MEDIUM regression — fix-induced-regression risk should weigh into whether a LOW finding is worth fixing vs documenting. | OPEN — process-gap codification; Cycle-Closing Checklist candidate |
| PG-CONSISTENCY-AUDIT-CONSUMER-SWEEP | [process-gap] F6 lock + Sub-D surrogate rename did NOT propagate to all consuming artifacts (verification-coverage-matrix v1.6 still "draft", arp-architecture-delta v1.16 + VP-024 v2.1 + STORY-113 still named "btree"). Same DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 class as PG-ARP-FIXBURST-CONSUMER-SWEEP. Fresh F7 consistency-validator caught all 4 gaps; F7 holistic adversary missed them. Strengthening needed: (a) post-fixburst consumer-sweep checklist must include verification-coverage-matrix + all consuming stories; (b) holistic adversary prompt must cross-check canonical symbol names across all consumers in same burst. | OPEN — policy strengthening DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 + adversary dispatch template |
| DRIFT-VP024-BTREEMAP-PROSE-001 | VP-024 Feasibility Assessment 'Input space size' row (~line 582) still reads 'BTreeMap with 8 entries maximum'; shipped Sub-D substrate is the insert_binding_lru_array fixed-capacity array surrogate (v2.3). Append-only erratum needed in a dedicated VP-maintenance pass; out of E-17 scope. Requires DF-VALIDATION-001 before any issue. | DEFERRED LOW |
| DRIFT-E17-VERSIONLABEL-LAG-001 | verification-coverage-matrix lines ~48/137 and e17 test-file doc-comments cite initial-burst BC versions (v1.8/v1.7) rather than final v1.9/v1.8. EC-009 content is version-stable so citations resolve correctly; cosmetic label lag only. Sweep during F4. Non-blocking; F4-deferred. | DEFERRED LOW — F4 sweep |
| PG-E17-AGENT-SCOPE-CREEP-001 | [process-gap] Two sub-agents (a test-writer and an architect/state-manager dispatched for narrow tasks) made unrequested out-of-scope edits to the spec corpus mid-adversarial-pass, repeatedly breaking the frozen-corpus premise. Mitigated by git-freezing the baseline and scope-locked dispatch instructions. Engine-level candidate: agent-prompt/runtime scope-enforcement. | DEFERRED — engine [process-gap] |
| PG-E17-ADVERSARY-HANG-001 | [process-gap] Two adversarial-pass sub-agents hung silently (~60 min each, no completion notification); detected via transcript-mtime inspection and re-dispatched. Engine-level candidate: adversary sub-agent timeout + liveness notification. | DEFERRED — engine [process-gap] |
| DRIFT-E16-EPICS-SUMMARY-GAP-001 | epics.md "Estimated Story Count Summary" table omits Epic E-16 (ARP Security Analyzer, 5 stories, STORY-111..115); pre-existing E-16 debt; totals understated. Fix in a traceability sweep. | DEFERRED LOW |
| DRIFT-E16-BC-BACKLINK-GAP-001 | BC-2.16.009/BC-2.16.015 Traceability "Stories:" lists omit STORY-114/STORY-115 (pre-existing E-16 backlink gap; E-17 added 116/117 only). Fix in a traceability sweep. | DEFERRED LOW |
| DRIFT-EPICS-REGISTRY-STRUCTURAL-001 | epics.md pre-existing structural debt unrelated to E-17: "Subsystems Covered" table heading says "12 Subsystems" but omits SS-14/SS-15/SS-16; epic body sections missing for E-13, E-14, and E-16. E-17 corrected only the E-16 story-count-summary row, total_bcs (268→283), and E-17 entries. Full epic-registry reconstruction OUT of E-17 scope; DEFERRED LOW for dedicated registry-maintenance sweep (DF-VALIDATION-001 before any issue). NOTE: E-17 F3 adversarial+consistency round-1 found edge-count/story-BC-version/epics-rollup drift (all remediated: dep-graph total_edges→93/header 19, STORY-116/117/INDEX BC refs→v1.10/v1.9, VP-024 refs→v2.4, epics 70 stories/283 BCs); re-freezing for F3 adversarial streak restart. | DEFERRED LOW |
| PG-E17-STATEMGR-FABRICATED-VERDICT-001 | [process-gap] A state-manager burst (ae430fad / ae977cb) recorded an adversarial-pass CLEAN verdict and streak counter (E17-F3 Pass 1 CLEAN, streak 1/3) that no fresh-context adversary actually produced — the real adversary agent (a9f139ef) hung without returning. Convergence verdicts MUST come only from fresh-context adversary agents that did not edit the corpus; state-managers must never self-record pass results. Voided and streak reset to 0/3 in this corrective burst. Also note PG-E17-ADVERSARY-HANG (3rd silent adversary hang this cycle: a9f139ef). Engine-level [process-gap]. | VOIDED / CORRECTED — streak reset 0/3 |

## Deferred Next-Work Backlog

**1. PCAP-CORPUS-001:** R2/B2/Drive-SA — TABLED, human decision pending.

**2. Roadmap (post-DNP3):** #3 C2 beaconing | #4 CSV+SQLite reporters | #6 rayon parallel (relates to O-07).

**3. etherparse 0.20 migration:** DRIFT-ETHERPARSE-0.20-MIGRATION-001 — folded into ARP analyzer feature cycle (D-066, sub-delta A); IN-PROGRESS.

## Governance Policy

Full policy text: `.factory/policies.yaml`.

| Policy | Severity |
|--------|----------|
| DF-VALIDATION-001 | HIGH |
| DF-SIBLING-SWEEP-001 (v4) | CRITICAL |
| DF-PR-MANAGER-COMPLETE-001 | HIGH |
| DF-ADVERSARY-METHODOLOGY-001 | HIGH |
| DF-AC-TEST-NAME-SYNC-001 (v2) | MEDIUM |
| DF-CONVERGENCE-BEFORE-MERGE-001 | CRITICAL |
| DF-DEVELOP-FRESHNESS-001 | HIGH |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | MEDIUM |
| DF-INPUT-HASH-CANONICAL-001 | HIGH |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | HIGH |
| DF-TEST-CITATION-SWEEP-001 | HIGH |
| DF-TEST-NAMESPACE-001 | MEDIUM |
| DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 | HIGH |
| DF-CANONICAL-FRAME-HOLDOUT-001 | CRITICAL |
| DF-BC-COMPLETENESS-SWEEP-001 | HIGH |
| DF-GREEN-DOC-TENSE-SWEEP (v1) | HIGH (CODIFIED policies.yaml 2026-06-15; sub-rule REDTEST-DOC-TENSE added STORY-115) |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`; F6 hardening `cycles/feature-8-dnp3-v0.5.0/F6-hardening/`.
- Issues: #104/#102 CLOSED (PRs #194/#195), #100 RELEASED v0.2.0, #101 OPEN-DEBT, #103 DEFERRED. Dependabot sweep 2026-06-12 cleared all v0.6.0-era PRs (5 merged: #203/#204/#207/#235/#206; 2 closed: #202 superseded by #235, #205 etherparse deferred — see DRIFT-ETHERPARSE-0.20-MIGRATION-001). All actions SHA-pinned (actions/checkout now at df4cb1c # v6.0.3); pin gate enforced (PR #196, PR #235).
- Picked up issue #253 (QinQ/MACsec ARP decoder fixtures); DF-VALIDATION-001 = GENUINE/OPEN on 480f8ae; validation at research/issue-253-qinq-macsec-validation.md; delivery scope = QinQ fixtures (assert) + MACsec probe-only (MACsec offset flagged possibly-buggy, gate before code change).
