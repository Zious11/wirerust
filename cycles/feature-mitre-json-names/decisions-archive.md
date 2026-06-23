---
document_type: decisions-archive
cycle_id: feature-mitre-json-names
archived_from: STATE.md Decisions Log
archived_at: 2026-06-23
archived_decisions: D-206..D-217
---

# Decisions Archive — feature-mitre-json-names (D-206..D-217)

*Archived from STATE.md during cycle-close burst 2026-06-23. D-136..D-202 previously archived to `cycles/feature-pcapng-reader/decisions-archive.md`.*

---

## D-206 — Feature Mode Opened; F1/F2/F3 Complete (2026-06-23)

Feature Mode opened for GitHub issue #64 (inline MITRE tactic/name in JSON). F1 delta analysis complete (1 BC, 1 story, additive/non-breaking). Research-agent override of the initial flat-field design: adopt an order-preserving ARRAY of per-technique objects under new field `mitre_attack` (id, name?, tactic_id?, tactic_name?, reference), aligning with ECS/OCSF; raw `mitre_techniques` unchanged. Human-approved field name `mitre_attack` and array design. F2 complete: BC-2.11.035 authored (10 ACs); BC-INDEX v1.70, PRD v1.34, interface-definitions v1.3, BC-2.11.001 v1.7. Catalog extension in scope: add `technique_tactic_id()` to src/mitre.rs (tactic_id not currently exposed; reference synthesized from technique ID). F3 complete: STORY-129 (Wave 57, ~5 pts, input-hash 2a5cee9, depends_on []); STORY-INDEX v2.7. No new Verification Property (pure Option-chaining over Kani-verified VP-007 → test-sufficient). F4 TDD implementation next.

---

## D-207 — STORY-129 Per-Story Adversarial Convergence (2026-06-23)

STORY-129 (issue #64 mitre_attack JSON enrichment) per-story adversarial convergence CONVERGED: 3 clean fresh-context passes (b8fea97/6d8f172/7e020ce), zero HIGH/CRITICAL. 13 BC-2.11.035 tests (EC-001..010 fully covered), full gates green (cargo test --all-targets, clippy -D warnings, fmt). Demo evidence recorded (modbus-write.pcap, T1692.001/T0836→TA0106). Process-gap DRIFT-BC-TEMPLATE-EC-VP-MAP-001 deferred (engine BC-template, LOW). PR next.

---

## D-208 — STORY-129 PR #306 MERGED; Issue #64 CLOSED (2026-06-23)

STORY-129 (issue #64 mitre_attack JSON enrichment) PR #306 MERGED to develop via merge commit 2fa6606 (squash disabled on repo; human merged). Issue #64 CLOSED. pr-reviewer APPROVE + security-reviewer PASS (no CRITICAL/HIGH; technique IDs are compile-time literals, serde-escaped, bounded alloc). CI 10/10. Worktree + branch cleaned up; develop ff to 2fa6606. stories_delivered 77→78. Human authorized FULL F5-F7. F5 scoped-adversarial next.

---

## D-209 — F5 HIGH Finding F-1 Found + ICS Tactic-Catalog Fix (2026-06-23)

F5 scoped adversarial found HIGH finding F-1: ICS techniques emitted Enterprise tactic IDs (Discovery TA0007 not ICS TA0102, etc.) under mitre_domain=ics-attack; research-validated against MITRE ATT&CK ICS v19.1 and found a 2nd bug (T0830 Adversary-in-the-Middle is Collection/TA0100 not Lateral Movement, and T0831 Manipulation of Control is Impact/TA0105 not Impair Process Control). Human authorized comprehensive catalog fix. Fix on branch fix/ics-tactic-ids: 3 new MitreTactic variants (IcsDiscovery TA0102, IcsCollection TA0100, IcsCommandAndControl TA0101); 5 techniques remapped (T0846/T0888→IcsDiscovery, T0885→IcsCommandAndControl, T0830→IcsCollection, T0831→IcsImpact). src/mitre.rs commit 719816e; demo re-recorded 74a48ea. 5 BCs bumped (BC-2.10.002 v1.5→v1.6, BC-2.10.003 v1.4→v1.5, BC-2.10.007 v1.8→v1.9, BC-2.11.035 v1.0→v1.1, BC-2.16.004 v1.7→v1.8), 3 holdouts corrected (wave-31-holdout.md, wave-40-44-holdout.md, HS-INDEX.md), STORY-129 EC-010 test renamed ec010_ics_collection, input-hashes recomputed (STORY-071/100/114/129 all MATCH; STORY-129 2a5cee9→93eba63). BC-INDEX v1.70→v1.71. Full suite green, clippy/fmt clean. Per-story adversarial convergence + fix-PR next.

---

## D-210 — ICS Tactic-Catalog Fix Adversarial Convergence (2026-06-23)

ICS tactic-catalog fix (F5 F-1 remediation) CONVERGED: 3 clean fresh-context adversarial passes (74a48ea/cf22de9/cf22de9), zero HIGH/CRITICAL. All 20 MitreTactic TA-ids verified vs authoritative MITRE ATT&CK ICS v19.1; consolidated authoritative-table test added (`test_ics_techniques_resolve_authoritative_tactic_ids`, 12 exact id→TA-id pairs — closes Pass-1 process gap). Branch fix/ics-tactic-ids @ cf22de9. 2 LOW backlog deferrals recorded: DRIFT-ARP-DEMO-FIXTURE-001 (no ARP pcap fixture for live T0830→TA0100 demo; correctness unit-tested), DRIFT-MITRE-SUBSET-COUNT-TESTS-001 (mitre/multitag dual-count subset tests 21/13 vs 25/17 — pre-existing cruft, no correctness impact). Convergence report: cycles/feature-mitre-json-names/f5-ics-fix-convergence.md. fix-PR to develop next.

---

## D-211 — ICS Fix PR #307 Created + Reviewed (2026-06-23)

ICS fix PR #307 created (fix: correct ICS-matrix tactic IDs), CI 10/10 green at head 96f0afc. security-reviewer PASS (pure static lookup remap, no new surface). Confirmation adversary pass on COMMITTED 96f0afc CLEAN (all 5 remaps + 20 TA-ids correct, no stale assertions, no Enterprise regression, BC-aligned). Orchestrator verified worktree clean + CI green directly. Process-gap DRIFT-UNCOMMITTED-TEST-EDITS-001 recorded. Awaiting human merge authorization (squash disabled → merge-commit).

---

## D-212 — ICS Fix PR #307 MERGED; F5 COMPLETE (2026-06-23)

ICS tactic-catalog fix PR #307 MERGED to develop via merge commit 029725b (merge-commit; squash disabled; human-authorized admin merge). Worktree + branch cleaned up; develop ff to 029725b. F5 scoped-adversarial COMPLETE: finding F-1 (ICS techniques emitting Enterprise tactic IDs) found, research-validated, comprehensively fixed (3 new MitreTactic variants, 5 techniques remapped), 3-pass converged, security PASS, merged. F6 targeted hardening NEXT (human authorized full F5-F7).

---

## D-213 — F6 Targeted Hardening COMPLETE (2026-06-23)

F6 targeted hardening COMPLETE (all 5 tasks PASS) for the issue #64 feature + ICS catalog fix on develop @ 029725b. Formal: no new VP warranted (mitre_attack path is pure Option-chaining, no panic/indexing/unwrap; technique_tactic_id is compile-exhaustive); VP-007 Kani 4/4 re-verified SUCCESSFUL. Mutation: cargo-mutants on json_dto.rs + mitre.rs = 100% of test-reachable mutants killed (49/53 viable; 4 survivors are #[cfg(kani)] harness bodies = Kani-verified false positives; 0 real test gaps). Fuzz: no JSON-reporter target exists; mitre_attack path panic-free by construction; fuzz_decode_packet 5.84M runs/91s zero crashes. Security: cargo audit 0 vulns, cargo deny clean. Regression: cargo test --all-targets green. Report: cycles/feature-mitre-json-names/f6-hardening.md. F7 delta-convergence + final human gate NEXT.

---

## D-214 — F7 Consistency Audit COMPLETE (2026-06-23)

F7 delta-convergence fresh-context consistency audit COMPLETE. Code/tests/BCs/demo FULLY CONSISTENT (all 5 ICS remaps, 20 TA-ids, EC-010 T0830→TA0100, Display strings, slice order 20, terminal grouping, input-hashes STORY-071/100/114/129 MATCH, no dangling renamed-test refs). 3 doc-accuracy gaps found + fixed: F7-CV-001 (MEDIUM, README ARP table Tactic column showed technique name not tactic for T0830/T1557.002 — fixed to 'Collection (ICS), Credential Access') + F7-CV-003 (LOW, historical design doc wrong TA0111→TA0102 + SUPERSEDED banner) shipped via docs PR docs/f7-mitre-tactic-doc-fixes (commit 05ef2ba, develop PR pending); F7-CV-002 (LOW, STORY-129 Task-1 stale '17 variants'→'20') fixed in this burst. F7 final human gate next.

---

## D-215 — F5 Sibling-Sweep Completion (DF-SIBLING-SWEEP-001) (2026-06-23)

F5 sibling-sweep completion (DF-SIBLING-SWEEP-001): propagated the 17→20 MitreTactic variant-count correction across 10 spec artifacts (vp-016 v2.6, BC-2.10.004 v1.6, cap-10 v2.0, cap-11 v1.3, ent-04 v1.4, ent-05 v1.2, nfr-catalog v2.4, test-vectors v2.3, prd v1.35, module-criticality v1.5) — these L2/VP/NFR specs still asserted '17 variants / 14 Enterprise + 3 ICS', contradicting the implemented 20-variant enum. Caught by orchestrator grep during F7 (the F7 consistency-validator scoped to the feature delta and missed the broad count references). Input-hash recomputed for affected stories: STORY-071 d630ed0 (MATCH after recompute), STORY-129 93eba63→b8da7e1 (recomputed). This was an incomplete-propagation gap from the F5 fix; recorded as Lesson 2 in cycles/feature-mitre-json-names/lessons.md. F7 final gate after re-verify.

---

## D-216 — F7 CONVERGED; Cycle CONVERGED; Human Authorized Release v0.9.4 (2026-06-23)

F7 delta-convergence APPROVED; docs PR #308 MERGED to develop (760b6ca); feature cycle feature-mitre-json-names CONVERGED across all 5 dimensions; human authorized close cycle + release v0.9.4; release prep in progress.

---

## D-217 — v0.9.4 RELEASED; Cycle CLOSED (2026-06-23)

v0.9.4 RELEASED. PR #309 (release/0.9.4) merged to main 96b49e8; annotated tag v0.9.4 on 96b49e8; release.yml run 28053327452 SUCCESS, 4 binaries published (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu); release URL https://github.com/Zious11/wirerust/releases/tag/v0.9.4 (published); develop back-merged 0115d0e.

Feature cycle feature-mitre-json-names CONVERGED + RELEASED + CLOSED: delivered mitre_attack JSON enrichment (issue #64, STORY-129) + ICS-matrix tactic-ID correctness fix (F5 F-1, incl. T0830/T0831 corrections). stories_delivered=78. Pipeline quiesced.

Cycle-closing checklist (S-7.02) CONFIRMED: all 4 process-gap findings have documented deferrals:
- DRIFT-UNCOMMITTED-TEST-EDITS-001 — DEFERRED MEDIUM, engine codification
- DRIFT-BC-TEMPLATE-EC-VP-MAP-001 — DEFERRED LOW, engine/template
- DRIFT-MITRE-SUBSET-COUNT-TESTS-001 — DEFERRED LOW, future maintenance
- DRIFT-ARP-DEMO-FIXTURE-001 — DEFERRED LOW, future cycle

Lessons 1 & 2 in cycles/feature-mitre-json-names/lessons.md are policy candidates; recorded and deferred to engine codification pass.

Decisions D-206..D-217 archived to `cycles/feature-mitre-json-names/decisions-archive.md`.
