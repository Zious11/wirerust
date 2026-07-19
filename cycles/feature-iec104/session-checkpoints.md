# Session Checkpoints Archive — feature-iec104

Archived checkpoints from STATE.md (superseded by newer session resume points).

---

## Checkpoint archived 2026-07-19 (replaced by D-479 session-wrap checkpoint)

**D-478 DEP-SOAK DELIVERED (2026-07-19). PR #420 squash-merged to develop 492554642c7d. STORY-176 v2.0 + STORY-166 only remaining E-11 local undelivered. Pipeline ACTIVE (steady-state). trajectory-tail →0→0→0→0**

- **Date:** 2026-07-19. Position: D-478 dep-soak COMPLETE; develop=492554642c7d4a3251df128789fd5f149fd2b0a7. Pipeline ACTIVE.
- **Ground truth:** main = `67a06b6f82654d2af79d023b15ac56ab03182ffd` (v0.13.0); develop = `492554642c7d4a3251df128789fd5f149fd2b0a7` (D-478 dep-soak PR #420). DRIFT-BACKMERGE-SQUASH-001 still applies.
- **In-flight work:** NONE. No stories mid-TDD, no open factory PRs, no story worktrees, no adversarial loop active.
- **Open items (not blocking):** (a) PR #407 governance decision (SAFE-WITH-CHANGES, DEFERRED — triage at `.factory/planning/pr-407-security-triage.md`, do NOT re-run security review); (b) STORY-166 (E-11, 3 pts, wave-TBD); (c) STORY-176 v2.0 (E-11, 2 pts, wave-TBD, local gate + tooling hygiene sweeps).
- **Pending human decisions:** PR #407 disposition; wave scheduling for STORY-166 + STORY-176 v2.0; SCORECARD_ENABLED enablement (see SCORECARD-ENABLEMENT-RUNBOOK carry-forward); structural fix for STORY-INDEX-IN-INPUTS-CHURN (remove STORY-INDEX.md from affected story inputs lists); DEP-SOAK-FOLLOWUP-2026-07-27 (next soak sweep on/after 2026-07-27).
- **Spec versions:** BC-INDEX v2.34 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.57 / STORY-INDEX v3.78 / dep-graph v3.9 (137 edges).
- **Resume command:** `/vsdd-factory:next-step` (reads STATE.md, continues from checkpoint). Pipeline ACTIVE — ready for next wave scheduling or PR triage.

---

## Checkpoint archived 2026-07-19 (replaced by D-480 E-11 disposition burst checkpoint)

**D-479 SESSION WRAP (2026-07-19). Human-requested pause at clean milestone post-D-478 dep-soak. Sessions D-475..D-478 (exhaustive) delivered; feature-iec104 CLOSED; v0.13.0 RELEASED; dep-soak PR #420 merged. No in-flight work. Pipeline PAUSED. trajectory-tail →0→0→0→0**

- **Date:** 2026-07-19. Position: steady-state, pipeline PAUSED post-D-478 (v0.13.0 released; feature-iec104 CLOSED; dep-soak delivered).
- **Ground truth:** main = `67a06b6f82654d2af79d023b15ac56ab03182ffd` (v0.13.0); develop = `492554642c7d4a3251df128789fd5f149fd2b0a7` (D-478 dep-soak PR #420); factory-artifacts = this wrap commit. DRIFT-BACKMERGE-SQUASH-001 still applies.
- **In-flight work:** NONE. No stories mid-TDD, no open factory PRs, no story worktrees, no adversarial loop, no sub-agents abandoned mid-step.
- **Pending human decisions:** (a) PR #407 governance (triage preserved at `planning/pr-407-security-triage.md` — do NOT re-run); (b) E-11 local mini-wave scheduling: STORY-166 (3 pts) + STORY-176 v2.0 (2 pts), wave-TBD; (c) STORY-INDEX-IN-INPUTS-CHURN structural decision (remove STORY-INDEX.md from STORY-164/165 inputs?).
- **Dated follow-ups:** DEP-SOAK-FOLLOWUP-2026-07-27 (17 deferred + 4 blocked bumps eligible 2026-07-21..27); harden-runner v2.20.0 Dependabot re-pin watch (~48h from 2026-07-19, manual re-pin SHA `bf7454d06d71f1098171f2acdf0cd4708d7b5920` if absent).
- **Advisory noted this wrap:** CLAUDE_AUTOCOMPACT_PCT_OVERRIDE missing from settings.json (recommend "70" per ADR-026 §D5) — operator decision, not auto-applied.
- **Spec versions:** BC-INDEX v2.34 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.57 / STORY-INDEX v3.78 / dep-graph v3.9 (137 edges).
- **Resume command:** `/vsdd-factory:next-step`. Superseded by D-480 E-11 disposition burst (wave-84 OPENED; STORY-INDEX v3.79).

---

## Checkpoint archived 2026-07-18 (replaced by D-474 session-wrap checkpoint)

**D-473 v0.13.0 RELEASED (2026-07-18). IEC-104 feature (feature-iec104 F1→F7 CONVERGED: F5 5-round, F6 hardening PASS, F7 5/5 dims holdout 0.99) + E2E coverage. Release PR #417 release/0.13.0→main merged 67a06b6 (human --merge). DRIFT-BACKMERGE-SQUASH-001 conflict (3 metadata files, [0.12.1] byte-identical, 0 source conflicts) resolved via main→release merge (9706be7). Tag v0.13.0 (object 03f35e4f); release.yml 29656299919 SUCCESS; GitHub Release 4 assets. Back-merge PR #418 main→develop SQUASH → af3ecbd. release/0.13.0 deleted. DRIFT-BACKMERGE-SQUASH-001 knowingly retained (squash pattern). trajectory-tail →0→0→0→0**

- **Date:** 2026-07-18. Position: D-473 v0.13.0 RELEASED. main=67a06b6, develop=af3ecbd, both 0.13.0. DRIFT-BACKMERGE-SQUASH-001 retained (squash back-merge pattern). trajectory-tail →0→0→0→0
- **Ground truth:** main = `67a06b6f82654d2af79d023b15ac56ab03182ffd` (v0.13.0); develop = `af3ecbdccb73f2ea91fc639290e2ba52ed62e6b7` (0.13.0 back-merged); DRIFT-BACKMERGE-SQUASH-001 still applies (main non-ancestor of develop; trees reconciled at v0.13.0).
- **Wave status:** Waves 76–83 DELIVERED. F4 COMPLETE. FIX-P4-001 DELIVERED (D-464). F5 CONVERGED (D-468): 5 rounds, NITPICK_ONLY; FIX-F5-001..004 all delivered. F6 PASS (D-469). F7 CONVERGED (D-470): 5/5 dims PASS; holdout 0.99. D-471 E2E IEC-104 coverage merged. D-472 PR #407 triaged SAFE-WITH-CHANGES DEFERRED. D-473 v0.13.0 RELEASED.
- **Remaining delivery sequence:** cycle-close lessons codification (B-001/B-002/TypeID-45/process-gaps) — optional, no blocking items. PR #407 security-triaged SAFE-WITH-CHANGES DEFERRED — governance decision pending (no re-triage needed). STORY-166 queued for next wave.
- **Spec versions:** BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 / STORY-INDEX v3.76 / dep-graph v3.9 (137 edges).

---

## Checkpoint archived 2026-07-18 (replaced by D-472 PR #407 security-triage checkpoint)

**D-471 E2E IEC-104 coverage merged (2026-07-17). PR #416 0b65e8e squash-merged to develop, human-executed merge. Human-directed post-F7 task. Closed the IEC-104 e2e gap (corpus had zero IEC-104 captures; 4SICS captures confirmed no port-2404 traffic). 4 real captures added to LOCAL-ONLY corpus (gitignored; sha256-pinned in bin/fetch-e2e-pcaps): iec104.pcap + IEC104_SQ.pcapng (Wireshark, local-use credit), 090813_diverse.pcap + TestDissectIec104.pcap (ITI CC-BY-4.0). Reader-level smoke-test: 105/1/173/147 pins (39 total, 0 mismatch, 0 panic). NEW tests/iec104_e2e_real_pcaps_tests.rs: in-process pipeline, CI-safe self-skip, DF-TEST-NAMESPACE-001 wrapper — iec104.pcap T0836×24+T1692.001×42=66; iti-diverse T0836×10+T1692.001×21=31; iti-dissect T0814×2+T1692.001×9=11; sq.pcapng 0 benign; all dropped_findings=0, 0 parse_errors. Complements STORY-174 synthetic holdout (0.99). CI 13/13. iec104_analyzer_tests 221/0. develop=0b65e8e (13 unreleased). RELEASE HELD (human). trajectory-tail →0→0→0→0**

- **Date:** 2026-07-17. Position: feature-iec104 F7-CONVERGED; E2E IEC-104 gap closed (PR #416); RELEASE HELD; v0.13.0 cut deferred pending human auth. trajectory-tail →0→0→0→0.
- **Ground truth:** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `0b65e8e`. DRIFT-BACKMERGE-SQUASH-001 still applies. 13 unreleased commits: STORY-167..174 + FIX-P4-001 + FIX-F5-001..004 + e2e-iec104-fixtures (PR #416 0b65e8e).
- **Wave status:** Waves 76–83 DELIVERED (D-441/443/445/447/448/455/458/463). F4 COMPLETE. FIX-P4-001 (D-464). F5 CONVERGED (D-468). F6 PASS (D-469). F7 CONVERGED (D-470): 5/5 dims PASS; holdout 0.99; RELEASE HELD. E2E coverage merged (PR #416).
- **Spec versions:** BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 / STORY-INDEX v3.76 / dep-graph v3.9 (137 edges).

---

## Checkpoint archived 2026-07-15 (replaced by D-448 STORY-171 DELIVERED — PAUSE)

**STORY-170 DELIVERED (D-447, 2026-07-15). develop=0bd93f8; stories_delivered=109; STORY-INDEX v3.63. PG-REDGREEN-COMMENT-CLEANUP 3x READY-TO-CODIFY. NEXT: STORY-171 (wave-80, N(S)/N(R) tracking; pre-delivery AC↔BC fidelity check recommended first).**

- **Date:** 2026-07-15. Position: feature-iec104 F4 delta-implementation IN PROGRESS; STORY-170 DELIVERED (D-447, wave-79); pipeline PAUSED. trajectory-tail →0→0→0→0.
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `0bd93f8`. DRIFT-BACKMERGE-SQUASH-001 still applies. 4 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2) + STORY-170 (PR #404 0bd93f8).
- **Wave status:** Wave-76 DELIVERED (D-441): STORY-167. Wave-77 DELIVERED (D-443): STORY-168. Wave-78 DELIVERED (D-445): STORY-169. Wave-79 DELIVERED (D-447): STORY-170 (control-command detection; BC-realigned v2.0; 5-pass adversary 3-clean). 4 of 8 IEC-104 stories complete.
- **Remaining F4:** STORY-171 (wave-80, N(S)/N(R) tracking + first-frame Option guard + desync detection, BC-2.19.023-024) through STORY-174 (wave-83). stories_delivered=109.
- **Pre-delivery recommendation:** Run AC↔BC field/behavior-fidelity check for STORY-171/172/173/174 before coding — cheap, caught real bugs in STORY-169 and STORY-170 (F3-DECOMPOSITION-BC-FIDELITY, 2 confirmed; READY-TO-CODIFY). RETRANSMIT-NS-FALSEPOS-001 carried to STORY-171.
- **Carry-forwards:** ROUTE-BC-DEFER-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; SEC-001-S168 (LOW, carry-append path inert; STORY-172); STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f).
- **Process-gaps READY-TO-CODIFY:** PG-REDGREEN-COMMENT-CLEANUP (3 occurrences: STORY-167+169+170); F3-DECOMPOSITION-BC-FIDELITY (2 confirmed: STORY-169+170).
- **Spec versions:** BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 / SS-19 v1.6 / STORY-INDEX v3.63 / dep-graph v3.9 (137 edges).

---

## Checkpoint archived 2026-07-15 (replaced by D-447 STORY-170 DELIVERED)

**STORY-170 v2.0 BC-realigned (D-446, 2026-07-14) — ready for code cycle. develop=ac01d9f2; stories_delivered=108; STORY-INDEX v3.62. NEXT: STORY-170 code cycle (wave-79, worktree → stub → test → impl → adversarial → demo → PR).**

- **Date:** 2026-07-14. Position: feature-iec104 F4 delta-implementation IN PROGRESS; STORY-170 v2.0 BC-realigned (D-446); pipeline PAUSED pre-wave-79 coding. trajectory-tail →0→0→0→0.
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `ac01d9f2`. DRIFT-BACKMERGE-SQUASH-001 still applies. 3 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2).
- **Wave status:** Wave-76 DELIVERED (D-441): STORY-167. Wave-77 DELIVERED (D-443): STORY-168. Wave-78 DELIVERED (D-445): STORY-169 (BC-realigned v1.1). STORY-170 v2.0 BC-realigned (D-446, pre-delivery).
- **Remaining F4:** STORY-170 (wave-79, control-command detection; v2.0 BC-aligned, ready for coding) through STORY-174 (wave-83). stories_delivered=108.
- **Pre-delivery recommendation:** Run AC↔BC field/behavior-fidelity check for STORY-171/172/173/174 before coding — cheap, caught real bugs in STORY-169 and STORY-170 (F3-DECOMPOSITION-BC-FIDELITY, 2 confirmed occurrences).
- **Carry-forwards:** ROUTE-BC-DEFER-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; SEC-001-S168 (LOW, carry-append path inert; STORY-172); STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f).
- **Process-gaps:** PG-REDGREEN-COMMENT-CLEANUP (2 occurrences: STORY-167+169); F3-DECOMPOSITION-BC-FIDELITY (2 confirmed: STORY-169+170; recommend pre-delivery AC↔BC check for STORY-171-174).
- **Spec versions:** BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 / SS-19 v1.6 / STORY-INDEX v3.62 / dep-graph v3.9 (137 edges).

---

## Checkpoint archived 2026-07-14 (replaced by D-446 STORY-170 v2.0 BC-realignment)

**STORY-167 + STORY-168 + STORY-169 DELIVERED (D-441/D-443/D-445, 2026-07-14) — waves 76–78 complete. develop=ac01d9f2; stories_delivered=108; STORY-INDEX v3.61. NEXT: STORY-170 (wave-79, BC-realignment required before coding).**

- **Date:** 2026-07-14. Position: feature-iec104 F4 delta-implementation IN PROGRESS; waves 76–78 DELIVERED; pipeline PAUSED between waves. trajectory-tail →0→0→0→0.
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `ac01d9f2`. DRIFT-BACKMERGE-SQUASH-001 still applies. 3 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2).
- **Wave status:** Wave-76 DELIVERED (D-441): STORY-167 (APCI core parser, 5 pts, BC-2.19.001-006 + VP-044 skeleton). Wave-77 DELIVERED (D-443): STORY-168 (frame discrimination + U-format session SM, 5 pts, BC-2.19.007-014 + VP-046). Wave-78 DELIVERED (D-445): STORY-169 (ASDU header extraction parse_asdu/Asdu, 3 pts, BC-2.19.015-018 + VP-047; BC-realigned v1.1).
- **Remaining F4:** STORY-170 (wave-79, control-command detection BC-2.19.019-022) through STORY-174 (wave-83). stories_delivered=108.
- **Pre-delivery note for STORY-170:** BC-realignment required before coding — AsduHeader→Asdu rename at 4 source sites + cot_test [TEST]-suppression AC (BC-2.19.017 inv1) missing from STORY-170 draft.
- **Carry-forwards:** ROUTE-BC-DEFER-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; SEC-001-S168 (LOW, carry-append path inert; STORY-172); STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f).
- **Process-gaps:** PG-REDGREEN-COMMENT-CLEANUP (2 occurrences: STORY-167+169); F3-DECOMPOSITION-BC-FIDELITY (STORY-170 pre-known drift).
- **Spec versions:** BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 / SS-19 v1.6 / STORY-INDEX v3.61 / dep-graph v3.9 (137 edges).

---

## Checkpoint archived 2026-07-14 (replaced by D-443 STORY-168 DELIVERED checkpoint)

**STORY-168 GREEN CHECKPOINT (D-442, 2026-07-14) — impl complete, adversarial NOT started. Worktree `.worktrees/STORY-168` durable; branch `feature/STORY-168-iec104-frame-discrimination-session-sm` (base develop e65e0d6); commits `392436b`+`dc8b867` (NOT pushed). NEXT: STORY-168 per-story adversarial 3-clean → demo → PR → merge, then STORY-169..174 (waves 78–83).**

- **Date:** 2026-07-14. Position: feature-iec104 F4 IN PROGRESS; wave-77 STORY-168 GREEN (impl complete); per-story adversarial NOT STARTED; pipeline PAUSED (session checkpoint).
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `e65e0d6eddbd24ce5a1c9482369978d2cc9dff36`. 1 unreleased commit (STORY-167 PR #401).
- **STORY-168 state:** Worktree `.worktrees/STORY-168` EXISTS and durable (local only). Branch `feature/STORY-168-iec104-frame-discrimination-session-sm` (base develop e65e0d6). NOT pushed to origin (local commits only). Commits: `392436b` (impl: classify_frame_format I/S/U + process_u_frame session SM + T0881/T0814, BC-2.19.007-014) + `dc8b867` (CHANGELOG). Tests: 64/64 iec104 tests green; full suite 0 failures; clippy/fmt/release CLEAN.
- **Scope in STORY-168:** classify_frame_format I/S/U; process_u_frame session SM: STARTDT-act→STARTDT-con (T0881 Possible/Likely); STOPDT-act→STOPDT-con; TESTFR-act→TESTFR-con (T0814 non-canonical U); fail-closed on non-canonical U. VP-046 totality covered. Iec104FlowState 5 fields per SS-19 v1.6. N(S) tracking UNWIRED — STORY-171. No dispatch wiring — STORY-173.
- **Carry-forwards:** ROUTE-BC-DEFERRED-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f).
- **Spec versions:** BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 / SS-19 v1.6 / STORY-INDEX v3.59 / dep-graph v3.9 (137 edges).

---

## Checkpoint archived 2026-07-14 (replaced by D-442 STORY-168 GREEN checkpoint)

**feature-iec104 F4 IN PROGRESS — wave-76/STORY-167 DELIVERED (D-441, 2026-07-14). develop=e65e0d6; stories_delivered=106; STORY-INDEX v3.59. NEXT: STORY-168 wave-77.**

- **Date:** 2026-07-14. Position: feature-iec104 F4 delta-implementation IN PROGRESS; wave-76 DELIVERED (D-441); pipeline PAUSED between waves.
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `e65e0d6eddbd24ce5a1c9482369978d2cc9dff36`. DRIFT-BACKMERGE-SQUASH-001: histories diverge (squash back-merge); trees differ by IEC-104 feature code. 1 unreleased commit (STORY-167 PR #401).
- **Active cycle:** feature-iec104 — F4 IN PROGRESS. Wave-76 DELIVERED: STORY-167 (APCI core parser, 5 pts, BC-2.19.001-006 + VP-044 skeleton). Per-story adversarial CONVERGED 4 passes streak 3/3 (BC-5.39.001). Security CLEAN. CI 13/13. Demo 7 artifacts scrub PASS.
- **Remaining F4 work:** STORY-168 (wave-77) → STORY-169 (wave-78) → STORY-170 (wave-79) → STORY-171 (wave-80) → STORY-172 (wave-81) → STORY-173 (wave-82) → STORY-174 (wave-83). All E-22, serialized due to src/analyzer/iec104.rs file contention.
- **F3 handoff carry-observations:** BC-2.10.010 EMITTED harness → STORY-173; RETRANSMIT-NS-FALSEPOS-001 → STORY-171; F3-H-002..005 LOW/NIT doc cleanup.
- **Carry-forwards:** ROUTE-BC-DEFERRED-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f).
- **Next work:** STORY-168 wave-77 plan gate + delivery. STORY-166 wave-TBD (E-11, 3 pts, draft).
- **Spec versions:** BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 / SS-19 v1.6 / STORY-INDEX v3.59 / dep-graph v3.9 (137 edges).

---

## Checkpoint archived 2026-07-14 (replaced by D-441 STORY-167 DELIVERED checkpoint)

**feature-iec104 F2 APPROVED+CLOSED (D-439, 2026-07-14). First-frame-guard mandate applied; Option<u16> enhancement in spec; MITRE ics-attack-19.1 confirmed. F3 story decomposition DONE (D-440); STORY-167..174 registered; plan-gate APPROVED. Pipeline PAUSED awaiting F4 delivery.**

- **Date:** 2026-07-14. Position: feature-iec104 F2 gate CLOSED (D-439); F3 decomposition DONE (D-440); F3 plan-gate approved; F4 delivery starting with STORY-167.
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `7b11b830ed8138136159a45aa6686b9df32cf707`. DRIFT-BACKMERGE-SQUASH-001: trees identical (5e75fd5); history-only divergence. 0 unreleased commits.
- **Active cycle:** feature-iec104 — F2 CLOSED; F3 DONE (D-440); STORY-167..174 (E-22, 8 stories, 36 pts, waves 76–83); dep-graph v3.9 acyclic (137 edges). F3 plan-gate approved (human, 2026-07-14).
- **F3-handoff items (4, LOW/NIT):** F3-H-002 BC-2.19.023 Description prose polish [partially addressed in v1.2]; F3-H-003 BC-2.10.010 VP table to also cite verify_all_emitted_ids_resolve; F3-H-004 feature-iec104-research.md §3.3/3.5/3.8 pre-v19 MITRE names reconciliation note; F3-H-005 BC-2.19.017 COT "2-byte little-endian" → "2 octets" terminology. (F3-H-001 first-frame-guard RESOLVED via Option<u16> D-439.)
- **F3 code obligations (4):** T0881 in src/mitre.rs; port 2404 in src/protocols.rs; DispatchTarget::Iec104 in src/dispatcher.rs; new src/analyzer/iec104.rs.
- **F3 carry-observations:** RETRANSMIT-NS-FALSEPOS-001 (backwards/retransmitted N(S) → large gap → possible T1692.001 false-positive; F3 implementer + F4 holdout to consider retransmit tolerance).
- **Carry-forwards:** ROUTE-BC-DEFERRED-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; STORY-166 wave-76 (E-11, 3 pts, wave-TBD, hash b56924f).
- **Spec versions:** BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 / SS-19 v1.6 / BC-2.19.023 v1.2 / BC-2.19.024 v1.3 / HS-INDEX v2.13 / STORY-INDEX v3.58 / dependency-graph v3.9 (137 edges) / module-criticality v1.6.
