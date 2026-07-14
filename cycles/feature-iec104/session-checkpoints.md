# Session Checkpoints Archive — feature-iec104

Archived checkpoints from STATE.md (superseded by newer session resume points).

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
