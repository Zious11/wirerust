---
document_type: pipeline-state
level: ops
version: "2.0"
producer: state-manager
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: wirerust
mode: steady-state
phase: "feature-iec104/F7-CONVERGED-release-held"
status: active
current_step: "D-472 PR #407 security-triaged SAFE-WITH-CHANGES, DEFERRED by human (open, no disposition). feature-iec104 F7-CONVERGED, v0.13.0 release HELD. develop=0b65e8e (13 unreleased). trajectory-tail →0→0→0→0"
current_cycle: "feature-iec104"
pipeline: IN PROGRESS
timestamp: 2026-07-18T06:30:00Z
released_version: v0.12.1
released_at: "2026-07-13"
release_tag: v0.12.1
release_tag_object: d687a77d911503e67a8d171c00536bd710762bba
release_commit: fedcea4ab17d9b3257c9903636aec0c0fd08f147
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.12.1
prior_released_version: v0.12.0
prior_released_at: "2026-07-10"
main_head: fedcea4ab17d9b3257c9903636aec0c0fd08f147
develop_head: 0b65e8e
cargo_version_main: "0.12.1"
cargo_version_develop: "0.12.1"
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
stories_delivered: 113
story_index_version: "v3.76"
total_stories: 127
story_index_note: "127 stories / 83 waves / 765 pts. v3.76 (2026-07-17): STORY-174 DELIVERED (D-463, PR #409 547deba, wave-83 SATISFIED; F4 COMPLETE 8/8). FIX-P4-001 delivered D-464 (fix PR, not a story). See cycles/feature-iec104/ for full F2/F3 history."
bc_index_version: "v2.33"
vp_index_version: "v2.46"
arch_index_version: "v2.19"
prd_version: "v1.56"
epics_version: v2.1
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-07-11
maintenance_started_at: "2026-07-11"
maintenance_completed_at: "2026-07-11"
maintenance_prior_run: maint-2026-07-09
---

<!--
  STATE.md SIZE BUDGET (per D-421(c)):
  Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 272 = 228 (dual-margin form). 272 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-472 PR #407 (external fork ArcavenAE/wirerust, fork-friendly release-ops, 2221 adds/15 files: signing/Homebrew/sync/signing-guard CI) security-triaged (2026-07-18): SAFE-WITH-CHANGES, 0 blocking vulns. All author claims verified: SHA-pinning (all `uses:` 40-char SHA-pinned; dtolnay pinned to fa04a14); CWE-77 env-binding (untrusted context expressions env-bound in all secret-bearing blocks); no pull_request_target anywhere; inert-by-default (sign/publish gated on vars.SIGNING_ENABLED=='true', sync gated on vars.SYNC_UPSTREAM_REPO!=''; only signing-guard linter runs without opt-in). 3 required-if-adopted changes: (1) sed-escape $VERSION in create-app.sh:236; (2) resolve bundle-id com.arcavenae.wirerust; (3) confirm Release trigger restricted to protected v* tags. DEFERRED by human — PR left OPEN, no disposition; governance question (adopt fork release-ops upstream?) unresolved. Triage: .factory/planning/pr-407-security-triage.md (not to be re-done). trajectory-tail →0→0→0→0**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); F4 COMPLETE + FIX-P4-001 DELIVERED (D-464); F5 CONVERGED (D-468); F6 PASS (D-469); **F7 (delta-convergence) CONVERGED (D-470): 5/5 dims PASS; holdout 0.99 RELEASE-READY; RELEASE HELD by human; v0.13.0 cut deferred; D-471 E2E coverage merged; D-472 PR #407 triaged SAFE-WITH-CHANGES DEFERRED** |
| Version | 0.12.1 (released 2026-07-13; main=fedcea4; develop=0b65e8e — 13 unreleased commits; DRIFT-BACKMERGE-SQUASH-001) |
| Main HEAD | `fedcea4ab17d9b3257c9903636aec0c0fd08f147` |
| Develop HEAD | `0b65e8e` — PR #416 E2E IEC-104 fixtures squash 2026-07-17; DRIFT-BACKMERGE-SQUASH-001 |
| Spec versions | BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 |
| Stories | 113 delivered / 127 total (STORY-INDEX v3.76, dep-graph v3.9, 765 pts) |
| **Last Updated** | 2026-07-18 — STATE.md compacted (D-437 through D-458 archived; phase-progress granular rows archived; convergence status archived). D-472 PR #407 triaged SAFE-WITH-CHANGES, DEFERRED. develop 13 unreleased. trajectory-tail →0→0→0→0 |

---

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0–7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0..v0.11.0 | RELEASED | Details: cycles/ subdirs |
| Maintenance maint-2026-06-22 + maint-2026-07-01 | COMPLETE | Details: cycles/ subdirs |
| Feature fix-tls-clienthello-frag (F1–F7) | CONVERGED/CLOSED | v0.11.1 released (D-316); see cycles/fix-tls-clienthello-frag/ |
| Feature feature-protocol-coverage E-21 (F1–F7) | CONVERGED/CLOSED | v0.11.2 released (D-382); see cycles/feature-protocol-coverage/ |
| Out-of-cycle + v0.11.3..v0.11.5 + maint-2026-07-06 | COMPLETE/RELEASED | D-383..D-393 (exhaustive); see cycles/history/decision-log-archive.md |
| Wave 70 (STORY-149) + maint-2026-07-08 + issue triage | COMPLETE | D-394..D-407 (exhaustive); see cycles/history/decision-log-archive.md |
| Wave 71 (v0.12.0: STORY-150/156/157) | CLOSED (D-404) | PRs #378-381; develop=b642c0f; 3/3 adversary; S-7.02 SATISFIED |
| Wave 72 (v0.12.0: STORY-158/159/160/161) | CLOSED (D-416) | PRs #387-391+gate-fix #391; gate all-green; S-7.02 SATISFIED |
| v0.12.0 RELEASED | RELEASED 2026-07-10 | PR #394 fedcea4; BREAKING JSON BC-2.11.036/037; histories reunified |
| Wave 73: STORY-162/163 | CLOSED (D-428) | PRs #395 b5e1e15 + factory-only D-427; adversary 6-pass streak 3/3 |
| Maintenance maint-2026-07-11 | COMPLETE (D-429) | PR #396 6779be6; 17 findings; ROUTES B/C DEFERRED |
| Wave 74: STORY-164 | CLOSED (D-432) | PR #397 d6e3be8; adversary 13-pass streak 3/3; ROUTE-W74-DEFERRED |
| Wave 75: STORY-165 | CLOSED (D-435) | PR #398 fa646ed; adversary 7-pass streak 3/3; S-7.02: STORY-166 drafted |
| v0.12.1 RELEASED | RELEASED 2026-07-13 | PR #399 fedcea4 main; back-merge #400 7b11b83; DRIFT-BACKMERGE-SQUASH-001 |
| feature-iec104 — F1 (delta-analysis) | DONE/APPROVED (2026-07-14) | 30 new BCs; SS-19; ADR-013; VP-044..047 |
| feature-iec104 — F2 (spec-evolution) | **APPROVED (D-439) CLOSED** | BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56; Option<u16>; MITRE ics-attack-19.1 confirmed |
| feature-iec104 — F3 (incremental-stories) | **APPROVED (D-440)** | STORY-167..174 (8 stories/36 pts/waves 76–83); dep-graph v3.9 (137 edges) |
| feature-iec104 — F4 (delta-implementation) | **COMPLETE (D-463)** | Waves 76–83 DELIVERED (D-441/443/445/447/448/455/458/463): STORY-167..174 PRs #401-409. 8/8. trajectory-tail →0→0→0→0 |
| feature-iec104 — STORY-173 pre-merge LOW fix burst | **COMPLETE** | 3 LOWs FIXED pre-merge (human approved all 3): LOW#1 flows_analyzed real cumulative counter (mirrors ENIP; 0bfc977); LOW#2 packets_analyzed valid-APDU frame counter (mirrors DNP3; 5325cf2); SEC-001/A-173-A-01 is_valid_iec104_frame doc overstated gate role + BC-2.19.006 v1.2; BC-INDEX v2.32→v2.33 (3ec6ac1). 2602→2604 tests. Triggered fresh A/B/C re-convergence. |
| **Other granular F4 rows (D-451 burst, per-story adversary STORY-172/173/174, wave gates 76–83, FIX-P4-001, FIX-F5-001..004)** | ARCHIVED | `cycles/feature-iec104/phase-progress-archive.md` |
| feature-iec104 — F5 (scoped adversarial) | **CONVERGED (D-468)** | 5 rounds; FIX-F5-001..004 delivered; code frozen since R2 (9c5aa9a); R5 NITPICK_ONLY (0 CRIT/HIGH/MED, 1 LOW non-blocking); BC-completeness 31/31 + canonical-frame 19 byte-exact clean |
| feature-iec104 — F6 (targeted-hardening) | **PASS (D-469)** | Kani/fuzz/mutation/audit/regression all green; VPs re-run post-fix on b36b884 |
| feature-iec104 — F7 (delta-convergence) | **CONVERGED (D-470)** | 5/5 dims PASS; holdout 0.99 RELEASE-READY; RELEASE HELD by human; v0.13.0 cut deferred |
| E2E IEC-104 coverage (human-directed, post-F7) | **MERGED (D-471)** | PR #416 0b65e8e; 4 fixtures + analyzer-level real-pcap test |
| PR #407 security triage (external fork, post-F7) | **TRIAGED SAFE-WITH-CHANGES (D-472)** | ArcavenAE fork-friendly release-ops; DEFERRED by human; triage preserved |

---

## Convergence Status

Per-story F4 convergence details archived to `cycles/feature-iec104/convergence-trajectory.md`.
F5 phase-level trajectory: 5 rounds, code frozen R2, `5H/M→2M→1H→1M→1L(NB)` — CONVERGED (D-468).

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | F7 CONVERGED (D-470) — 5/5 dims; holdout 0.99; RELEASE HELD; v0.13.0 cut pending human auth; D-471 E2E coverage merged (0b65e8e); D-472 PR #407 triaged SAFE-WITH-CHANGES DEFERRED | develop |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-472 PR #407 (external fork ArcavenAE/wirerust, fork-friendly release-ops, 2221 adds/15 files: signing/Homebrew/sync/signing-guard CI) security-triaged (2026-07-18): SAFE-WITH-CHANGES, 0 blocking vulns. Author claims verified (SHA-pinning, CWE-77 env-binding, no pull_request_target, inert-by-default). 3 required-if-adopted: sed-escape create-app.sh:236; bundle-id com.arcavenae→upstream domain; confirm Release trigger restricted to protected v* tags. DEFERRED by human — PR left OPEN, no disposition; governance question (adopt fork release-ops upstream?) unresolved. Triage: .factory/planning/pr-407-security-triage.md.** | **TRIAGED SAFE-WITH-CHANGES (D-472)** | PR left OPEN, DEFERRED by human; governance decision pending. trajectory-tail →0→0→0→0 |
| **D-471 E2E IEC-104 coverage merged (PR #416 0b65e8e squash-merged to develop, 2026-07-17, human-executed merge). Human-directed post-F7 task. Closed the IEC-104 e2e gap: corpus had zero IEC-104 captures; 4SICS captures confirmed no port-2404 traffic. Added 4 real captures to LOCAL-ONLY corpus (gitignored; sha256-pinned in bin/fetch-e2e-pcaps): iec104.pcap + IEC104_SQ.pcapng (Wireshark, local-use credit), 090813_diverse.pcap + TestDissectIec104.pcap (ITI CC-BY-4.0). Reader-level corpus smoke-test: 105/1/173/147 pins (39 total, 0 mismatch, 0 panic). NEW tests/iec104_e2e_real_pcaps_tests.rs (in-process pipeline, CI-safe self-skip mirroring ENIP test, DF-TEST-NAMESPACE-001 mod wrapper): pins per-technique detection on real traffic — iec104.pcap T0836×24+T1692.001×42=66; iti-diverse T0836×10+T1692.001×21=31; iti-dissect T0814×2+T1692.001×9=11; sq.pcapng 0 benign; all dropped_findings=0, 0 parse_errors. Complements STORY-174 synthetic-frame holdout (0.99) with real-world capture validation. CI 13/13. iec104_analyzer_tests 221/0. develop=0b65e8e (13 unreleased).** | **MERGED (D-471)** | 4 fixtures + analyzer-level real-pcap test; develop=0b65e8e (13 unreleased). trajectory-tail →0→0→0→0 |
| **D-470 feature-iec104 F7 CONVERGED (2026-07-17). All 5 dimensions PASS: Spec novelty LOW (F5 R5); Test mutation 95.9% (F6); Impl 0 open HIGH; Verification F6 all-green (Kani 5 harnesses, fuzz 2.64M/0 crashes, audit 0 vulns); Holdout black-box acceptance mean 0.99 RELEASE-READY (holdout-evaluator, info-asymmetry, canonical IEC-104 frames; must-pass #1/#4/#6 all 1.0). Regression 2627/0. Input-hash drift resolved: STORY-167..172 re-baselined BENIGN (consistency audit); STORY-164/165 out-of-scope. Consistency audit 2 MINOR doc-only (B-001/B-002, deferred cycle-close). RELEASE HELD (human direction) — v0.13.0 MINOR cut deferred. F7 human gate: convergence approved, release-cut deferred.** | **CONVERGED (D-470)** | 5/5 dims PASS; holdout 0.99; RELEASE HELD; v0.13.0 deferred. trajectory-tail →0→0→0→0 |
| **D-469 feature-iec104 F6 targeted hardening PASS (2026-07-17). All VPs re-confirmed against post-fix develop b36b884 (iec104.rs emit sites changed by FIX-P4-001/FIX-F5-001 — iec104-dependent checks RE-RUN not assumed): Kani VP-044/004/007 all SUCCESSFUL; VP-045/046 proptest pass; VP-047 fuzz 2.64M runs/5min/0 crashes; cargo-mutants iec104.rs 95.9% (118/123, 0 killable, 5 equivalent-justified); cargo-audit 0 vulns/193 deps; regression 2627/0; clippy+fmt clean. semgrep skipped (absent; cargo-audit + per-PR security-reviews cover surface). Info-asymmetry wall honored (F5 findings not consulted). Verifier self-corrected a -j4 mutation false-timeout measurement error, re-ran scoped for authoritative 95.9%. No BLOCKERs. Artifacts .factory/phase-f6-hardening/. F6 gate PASSED — F7 delta convergence next.** | **PASS (D-469)** | Kani/fuzz/mutation/audit/regression all green; No BLOCKERs; F7 next. trajectory-tail →0→0→0→0 |
| **D-468 feature-iec104 F5 scoped adversarial CONVERGED (2026-07-17). 5 rounds. FIX-F5-002 (PR #412 b356545), FIX-F5-003 (PR #413 9eab53f), FIX-F5-004 (PR #415 b36b884) all DELIVERED (human-executed merges). R5 NITPICK_ONLY: 0 CRITICAL/HIGH/MEDIUM; 1 LOW non-blocking (TypeID 45 demo-prose mislabel, code correct at iec104.rs:744-748). Feature code frozen since R2 (9c5aa9a); R3-R5 tail was demo-evidence/CHANGELOG doc-accuracy only (root cause PG-DEMO-JSON-FABRICATION). BC-completeness 31/31 + canonical-frame 19 byte-exact clean. develop=b36b884 (12 unreleased: STORY-167..174 + FIX-P4-001 + FIX-F5-001/002/003/004). F5 gate PASSED — F6 targeted hardening next.** | **CONVERGED (D-468)** | R5 NITPICK_ONLY; 0 CRIT/HIGH/MED; F6 next. trajectory-tail →0→0→0→0 |

## Decisions Log

| ID | Decision | Date |
|----|----------|------|
| D-001..D-301 (exhaustive). Greenfield through feature-enip-v0.11.0; see cycles/*/decisions-archive.md for full range. | — | — |
| D-302..D-436 (exhaustive). Fix-tls through feature-protocol-coverage through v0.12.1; see cycles/history/decision-log-archive.md for full range. | — | — |
| D-437..D-458 (exhaustive). feature-iec104 F1 engine triage through F4 delivery; see cycles/feature-iec104/decisions-archive.md for full range. | — | — |
| D-460 | Session RESUMED (human-approved, 2026-07-16). Worktree health PASS; develop=084ff93 verified; no story worktrees; only open PR is external #407 (deferred post-wave-83 by human). STORY-174 wave-83 begins with research-agent validation of carry-forward scope items before any story realignment (human-directed). | 2026-07-16 |
| D-461 | STORY-174 pre-delivery realignment COMPLETE (research-validated, human-approved 2026-07-16). DF-VALIDATION-001 research 2 passes (story-174-scope-validation.md + -followup.md; all HIGH confidence): (1) PG-REDGREEN-COMMENT-CLEANUP VALID-INCLUDE — codified as AC-174-008 extending existing green-doc-tense-gate token list (3 patterns; zero tree-wide false positives; no allowlist change) + scrub of 3 baseline stale headers + CHANGELOG entry; (2) F-172-003 VP-045 vacuity VALID-INCLUDE — AC-174-002 amended with non-vacuity/interleaved-generator/state-comparison requirements (carry fields already pub; zero production code); (3) IEC104-FINDING-DIRECTION-001 VALID-DEFER out of STORY-174 — routed to dedicated pre-F5 fix-PR inside feature-iec104 via fix-pr-delivery (ENIP D-262 PR #331 precedent; PG-W72 holdout sweep near-empty, additive JSON key). STORY-174 v2.0 input-hash de9d14e→27c86aa (also resolved genuine BC-2.19.006 v1.2 input drift from D-458). STORY-INDEX v3.72→v3.73. Points unchanged (5). | 2026-07-16 |
| D-462 | STORY-174 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 7 passes; streak P5/P6/P7; final HEAD e62701f; base 084ff93; 2600+/0 tests (92 suites). Trajectory P1(1M F-174-001)->P2(1M F-174-002)->P3(NITPICK_ONLY)->P4(1M F-174-P4-001)->P5/P6/P7 CLEAN. F-174-001 MEDIUM VP-044 valid→Some facet missing (Kani 82→89 checks; 1071de4); F-174-002 MEDIUM stale skeleton/false CI-wiring prose + 8-site sibling sweep (038286a); F-174-P4-001 MEDIUM BC-2.19.025 invariant-2 mis-anchor from v1.3 renumbering re-cited to VP-045 harness registration (e62701f 8 test + 2 story sites). Story v2.2; STORY-INDEX v3.75. Kani non-vacuity 3/3 every pass. Mutation 117/122=95.9%. Fuzz 1.35M execs clean. PG-GATE-VOCAB-BLINDSPOT filed (green-doc-tense gate misses "skeleton"/"seam" phrasing; 2 independent obs P2+P4). Demos/PR next. | 2026-07-16 |
| D-463 | STORY-174 DELIVERED (PR #409 547deba squash-merged to develop, 2026-07-17, human-authorized per-PR — human executed merge directly in main thread after TWO classifier halts: DF-MERGE-AUTH-CLASSIFIER-001 condition-4 wave-grant-absent, then PG-MERGE-AUTH-SUBAGENT-CLASSIFIER harness deny of subagent --admin merge on relayed consent; orchestrator-direct attempt also denied on unnamed --admin bypass; bypass tagged per DF-PR-MANAGER-COMPLETE-001(b)). Per-story adversarial CONVERGED 3-clean D-462 (7 passes P5/P6/P7). Security APPROVE (1 LOW SEC-001 CWE-22 bin path-prefix accepted, joins SEC-001-S158 class). pr-reviewer APPROVE (2 NITs accepted). CI 13/13 + post-merge develop CI SUCCESS. Demos 9 artifacts/8 ACs scrub PASS. Kani VP-044 89 checks (5 facets) + VP-004 (440/407/183) + VP-007 (122, SEEDED=29); VP-045/046 non-vacuous proptests (F-172-003 RESOLVED); VP-047 fuzz 1.35M execs clean; cargo-mutants 117/122=95.9%; green-doc-tense gate patterns 23-25 + baseline scrub (PG-REDGREEN-COMMENT-CLEANUP CODIFIED-DELIVERED; PG-REDGREEN-SIBLING-SWEEP RESOLVED). 8th of 8 IEC-104 stories. develop=547deba (8 unreleased: STORY-167..174); stories_delivered 112→113. Wave-83 gate SATISFIED (single-story wave: per-story 3-clean == wave-level on identical diff, per waves 79-82 precedent). F4 delta-implementation COMPLETE. New process-gap: PG-MERGE-AUTH-SUBAGENT-CLASSIFIER (subagent cannot execute --admin merge on relayed human consent; orchestrator-direct attempt also denied on unnamed --admin bypass. Resolution path = human-direct in main thread (per D-463). Codify at cycle-close as AC for E-11 follow-up story. STORY-INDEX v3.76. | 2026-07-17 |
| D-464 | FIX-P4-001 DELIVERED (PR #410 7e95f71 squash-merged to develop, 2026-07-17, human-executed merge per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER). fix-pr-delivery flow (D-461 routing; ENIP D-262 PR #331 precedent). IEC104-FINDING-DIRECTION-001 RESOLVED — all 10 IEC-104 emit sites now direction: Some(...) (was None); direction threaded into process_u_frame + detect_iec104_threats; redundant direction-in-evidence strings dropped; 11 direction-assertion tests (mod fix_p4_001, red-first); additive `direction` JSON key documented in CHANGELOG; holdout-expectations sweep COMPLETE (PG-W72; zero IEC-104 holdout scenarios, subset assertions unaffected; docs/holdout-expectations-sweep-FIX-P4-001.md). Security review PASS 0 findings. pr-reviewer APPROVE (2 NITs accepted). CI 13/13 + post-merge develop CI SUCCESS. Demo evidence 3 artifacts scrub PASS. develop=7e95f71 (9 unreleased: STORY-167..174 + FIX-P4-001). F5 scoped adversarial UNBLOCKED. | 2026-07-17 |
| D-465 | feature-iec104 F5 scoped adversarial OPENED (2026-07-17). Round 1 @ develop 7e95f71: BC-set completeness sweep 31/31 PASS (no missing-feature blocker); canonical-frame sweep 19 invariants byte-exact vs IEC 60870-5-104 (no DNP3-DIR-class defect); findings 1H+4M — F-01 HIGH BC-2.19.011 PC-3 source_ip unmet (untested blind spot) + F-02 source_ip/timestamp parity + F-03 stale prose (+4 new siblings) + F-04 false forward-ref + F-05 stale count. All 5 batched to FIX-F5-001 (in progress). MITRE EXECUTION-REQUIRED axis closed via D-439 v19.1 pin research. Phase frontmatter → feature-iec104/F5. | 2026-07-17 |
| D-466 | FIX-F5-001 DELIVERED (PR #411 9c5aa9a squash-merged to develop, 2026-07-17, human-executed merge). Batches F5 Round-1 findings F-01 HIGH + F-02/03/04/05 MEDIUM: source_ip + timestamp enrichment threaded through all 10 IEC-104 emit sites (8 function + 2 inline; DNP3/ENIP house-parity pattern) — BC-2.19.011 PC-3 SATISFIED; 10 red-first tests mod fix_f5_001 (each asserts source_ip+timestamp per finding family); 9 stale-prose sites scrubbed GREEN + protocols_tests count comment fixed; false forward-ref comment removed; additive JSON keys source_ip/timestamp documented in CHANGELOG; holdout-expectations sweep COMPLETE (PG-W72; docs/holdout-expectations-sweep-FIX-F5-001.md). Security PASS 0 findings. pr-reviewer APPROVE (MINOR count-prose + NIT timestamp-type both remediated in-file, orchestrator row-verified per PG-W74). CI 13/13 + post-merge SUCCESS. Demo before/after JSON scrub PASS. develop=9c5aa9a (10 unreleased: STORY-167..174 + FIX-P4-001 + FIX-F5-001). F5 Round 2 next (fresh adversary on fixed files). | 2026-07-17 |
| D-467 | F5 scoped adversarial Rounds 2-3 (2026-07-17). R2 @ 9c5aa9a: code CONVERGED (all 5 R1 findings verified fixed by FIX-F5-001; 0 code findings; direction→source_ip DNP3-parity-exact; tests non-vacuous) + 2 MEDIUM doc-accuracy findings (F5R2-01 wrong provenance, F5R2-02 fabricated T0881 JSON) → FIX-F5-002 (PR #412 b356545 merged). R3 @ b356545: R2 doc-fixes verified; NEW F-B1 HIGH — FIX-P4-001 demo-evidence artifacts still fabricated (category 'Protocol'/verdict 'Anomaly'/confidence 'High' — non-existent variants, non-compiling demo .rs, wrong MITRE technique) + FIX-F5-002 CHANGELOG false correction claim. 3rd fabricated-demo-JSON occurrence → root cause demo-recorder hand-writing JSON not deriving from real serde output. Routed to FIX-F5-003 (comprehensive demo-evidence JSON-accuracy sweep across full feature tree, in progress). Feature CODE/tests CONVERGED since R2; F5 gate blocked only on docs accuracy. New process-gap PG-DEMO-JSON-FABRICATION filed. | 2026-07-17 |
| D-468 | feature-iec104 F5 scoped adversarial CONVERGED (2026-07-17). 5 rounds. FIX-F5-002 (#412 b356545), FIX-F5-003 (#413 9eab53f), FIX-F5-004 (#415 b36b884) all DELIVERED (human-executed merges). R5 NITPICK_ONLY: 0 CRITICAL/HIGH/MEDIUM; 1 LOW non-blocking (TypeID 45 C_SC_NA_1 described as "monitoring direction" in docs/demo-evidence/FIX-P4-001/evidence-report.md:46 + AC-P4-001-test-results.txt:61 — code correct at iec104.rs:744-748; prose-only, non-blocking). Feature code frozen since R2 (9c5aa9a); R3-R5 tail was demo-evidence/CHANGELOG doc-accuracy only (root cause PG-DEMO-JSON-FABRICATION). BC-completeness 31/31 + canonical-frame 19 byte-exact clean. develop=b36b884 (12 unreleased: STORY-167..174 + FIX-P4-001 + FIX-F5-001/002/003/004). F5 gate PASSED — F6 targeted hardening next. trajectory: findings decayed 5→2→1(HIGH,docs)→1(MEDIUM,docs)→0(1 LOW). | 2026-07-17 |
| D-469 | feature-iec104 F6 targeted hardening PASS (2026-07-17). All VPs re-confirmed against post-fix develop b36b884 (iec104.rs emit sites changed by FIX-P4-001/FIX-F5-001 → iec104-dependent checks RE-RUN not assumed): Kani VP-044/004/007 all SUCCESSFUL; VP-045/046 proptest pass; VP-047 fuzz 2.64M runs/5min/0 crashes; cargo-mutants iec104.rs 95.9% (118/123, 0 killable, 5 equivalent-justified); cargo-audit 0 vulns/193 deps; regression 2627/0; clippy+fmt clean. semgrep skipped (absent; cargo-audit + per-PR security-reviews cover surface). Info-asymmetry wall honored (F5 findings not consulted). Verifier self-corrected a -j4 mutation false-timeout measurement error, re-ran scoped for authoritative 95.9%. No BLOCKERs. Artifacts .factory/phase-f6-hardening/. F6 gate PASSED — F7 delta convergence next. | 2026-07-17 |
| D-470 | feature-iec104 F7 delta convergence CONVERGED (2026-07-17). All 5 dimensions PASS: Spec novelty LOW (F5 R5); Test mutation 95.9% (F6); Impl 0 open HIGH; Verification F6 all-green (Kani 5 harnesses, fuzz 2.64M/0 crashes, audit 0 vulns); Holdout black-box acceptance mean 0.99 RELEASE-READY (holdout-evaluator, info-asymmetry, canonical IEC-104 frames; must-pass #1/#4/#6 all 1.0). Regression 2627/0. Input-hash drift resolved: STORY-167..172 re-baselined BENIGN (consistency audit); STORY-164/165 out-of-scope. Consistency audit 2 MINOR doc-only (B-001/B-002, deferred cycle-close). RELEASE HELD (human direction) — v0.13.0 MINOR cut deferred. F7 human gate: convergence approved, release-cut deferred. | 2026-07-17 |
| D-471 | E2E IEC-104 coverage merged (PR #416 0b65e8e squash-merged to develop, 2026-07-17, human-executed merge; human-directed task, post-F7). Closed the IEC-104 e2e gap (corpus had zero IEC-104 captures; 4SICS ICS captures confirmed no port-2404 traffic). Added 4 real captures to the LOCAL-ONLY corpus mechanism (gitignored; sha256-pinned in bin/fetch-e2e-pcaps): iec104.pcap + IEC104_SQ.pcapng (Wireshark, local-use credit), 090813_diverse.pcap + TestDissectIec104.pcap (ITI CC-BY-4.0). Reader-level corpus smoke-test pins (105/1/173/147; full corpus 39 pins 0 mismatch 0 panic). NEW analyzer-level tests/iec104_e2e_real_pcaps_tests.rs (in-process pipeline, CI-safe self-skip mirroring ENIP test, DF-TEST-NAMESPACE-001 mod wrapper): pins per-technique detection on real traffic — iec104.pcap T0836×24+T1692.001×42=66; iti-diverse T0836×10+T1692.001×21=31; iti-dissect T0814×2+T1692.001×9=11; sq.pcapng 0 benign; all dropped_findings=0, 0 parse_errors. Complements STORY-174 synthetic-frame holdout (0.99) with real-world capture validation. CI 13/13. iec104_analyzer_tests 221/0. develop=0b65e8e (13 unreleased). | 2026-07-17 |
| D-472 | PR #407 (external fork ArcavenAE/wirerust, fork-friendly release-ops, 2221 adds/15 files: signing/Homebrew/sync/signing-guard CI) security-triaged (2026-07-18): SAFE-WITH-CHANGES, 0 blocking vulns; author claims (SHA-pinning, CWE-77 env-binding, no pull_request_target, inert-by-default) all VERIFIED. 3 required-if-adopted changes (sed-escape create-app.sh:236; bundle-id com.arcavenae→upstream domain; confirm Release trigger restricted to protected v* tags). DEFERRED by human — PR left OPEN, no disposition; governance question (adopt fork release-ops upstream?) unresolved. Triage report: .factory/planning/pr-407-security-triage.md (preserved, not to be re-done). Note: fork PR workflows require maintainer approval to run (currently 'no checks'). | 2026-07-18 |

---

## Skip Log

| Step | Justification |
|------|---------------|
| crates.io publish (v0.11.0) | Human declined at D-300 — not published |
| Holdout formal eval HS-110..122 | Deferred post-release per D-267; 10/13 behaviors covered by unit tests |
| DTU creation | Not required (passive analyzer; no external service calls) — D-dtu-assessment 2026-05-20 |

---

## Blocking Issues

| ID | Issue | Severity | Owner | Resolution |
|----|-------|----------|-------|------------|
| (none) | No open blocking issues. | — | — | — |

---

## Drift Items

| ID | Summary | Source | Target |
|----|---------|--------|--------|
| DRIFT-SPRINT-STATE-FIELD-FORM-001 | sprint-state.yaml uses both `merge_sha:` and `merge_commit:` field names inconsistently across wave entries. Vestigial file (STORY-INDEX is authoritative wave registry). | wave-75 gate S-7.02 justified deferral (D-435) | vestigial-file retirement at next housekeeping pass |
| DRIFT-BACKMERGE-SQUASH-001 | v0.12.1 back-merge PR #400 was squash-merged; main (fedcea4) NOT ancestor of develop (0b65e8e). Trees ARE identical for released content — history-only divergence. | v0.12.1 release (D-436, 2026-07-13) | resolve at next release cut |
| DRIFT-VP039-BC207038-TLS-TODO-001 | VP-INDEX carries stale present-tense "PO must add BC-2.07.038 postcondition/EC + Red-Gate test name" TODOs for VP-039 (TLS reassembly). Out of feature-iec104 scope. | feature-iec104 F2 review (D-438, 2026-07-14) | SS-07 TLS owner — next TLS maintenance sweep |

---

## Active Carry-Forwards

| ID | Summary | Target |
|----|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11 (human decision). | Next maintenance run |
| ROUTE-W74-DEFERRED | Code-review 1 NIT deferred from wave-74 gate (human-ratified); joins wave-75 NIT. | Next bin-touch PR |
| PERF-RERUN-001 | AC-149-003 quiescent re-run pending (load avg 52.57 at maint-2026-07-11; human deferred). | Next maintenance run |
| SEC-001 | SEC-001-ENIP (split-borrow) deferred from maint-2026-07-11; next feature wave. | Next feature wave or maintenance |
| STORY-166 | E-11, 3 pts, wave-TBD, hash b56924f; S-7.02 carry from wave-75. | Next wave after feature-iec104 release |
| F3-handoff cleanup | F-F3P12-002 (STORY-151 pointer note), F-F3P13-002 (STORY-154 frontmatter SS-05), F-F3P17-001 (STORY-154 cross-layer trace). | F4 implementation per-story |
| SEC-001-S158 / SEC-002-S158 | CWE-22 LOW advisories in `bin/lint-cycle-artifact` (deferred until mandatory CI wiring). DF-VALIDATION-001-gated. | bin/lint-cycle-artifact CI wiring |
| F3-DECOMPOSITION-BC-FIDELITY | **4 CONFIRMED occurrences: STORY-169** (flat vs broken-out fields; wrong guards) **+ STORY-170** (false-positive T0827; confidence Possible→Likely; reserved-TypeID scope; naming) **+ STORY-172** (FlowId→FlowKey nonexistent; carry-overflow discard-all-new semantics; malformed-LEN PC4 contradiction) **+ STORY-173** (T0881 tactic string "impact" → MitreTactic; compilation blocker). All corrected pre-delivery. **CODIFY-NOW.** Codification: mandatory pre-delivery AC↔BC fidelity check as F3/F4 gate step. Vehicle: cycle-close E-11 follow-up. | Cycle-close codification |
| IEC104-TIMED-CMD-GAP-001 | (DETECTION GAP, security-relevant, DEFERRED) TypeIDs 58–64 (timed control variants C_SC_TA_1=58 .. C_BO_TA_1=64) fall into detect_iec104_threats `_` silent arm; no T1692.001/T0836 findings emitted. Out of scope per BC-2.19.019. Evasion gap: control commands via timed variants bypass detection. Source: sec-review-170 L-001 (PR #404). DF-VALIDATION-001 required before filing any GitHub issue. | Follow-on detection story (new BC + detection arm for TypeIDs 58–64, or feature-cycle extension) |
| IEC104-DEMO-TYPEID45-MISLABEL | (LOW, non-blocking) TypeID 45 (C_SC_NA_1 control command) described as "monitoring direction" in docs/demo-evidence/FIX-P4-001/evidence-report.md:46 + AC-P4-001-test-results.txt:61; code correct (iec104.rs:744-748); prose-only, non-blocking. Added D-468. | Next docs-currency sweep or cycle-close |
| PG-SPEC-VERSION-CITATION-CURRENCY | Spec-version bumps must include src/ comments and CHANGELOG entries in the citation-currency sweep set (surfaced by F-172-301 NIT, D-454). | cycle-close lessons codification |
| PG-DOC-CURRENCY-SWEEP | Post-adversarial doc-accuracy drift consumed 12 of 17 STORY-173 passes. A pre-adversarial code-comments/test-header doc sweep would reduce adversarial pass count. | Cycle-close codification |
| PG-ADVERSARY-IDLE-NO-REPORT | Adversary agents completing CLEAN passes sometimes emitted no report, making CLEAN vs idle indistinguishable. Recurring behavior flagged across multiple STORY-173 passes. | Cycle-close lessons codification |
| PG-ADVERSARY-SEVERITY-CALIBRATION | Whole-source doc sweeps at late passes generated advisory findings against code FROZEN since P2. Adversary instances diverging on severity calibration for code that hasn't changed. | Cycle-close lessons codification |
| PG-STATE-RECOVERY-SCOPE | Session-boundary state recovery must verify ALL worktrees and the main develop checkout simultaneously. Omitting the main checkout created the stray-commit 105497f gap (D-458). | Cycle-close codification |
| PG-VERIFY-ALL-WORKTREES | Post-agent verification must span ALL worktrees and the main develop checkout. A fix agent committed to the main develop checkout (not a worktree), creating stray commit 105497f which had to be discarded. | Cycle-close codification |
| PG-GATE-VOCAB-BLINDSPOT | Green-doc-tense gate (AC-174-008) misses "skeleton" and "seam" phrasing (stub-era language surviving into green deliveries). 2 independent adversary observations: P2 Obs-1 + P4 obs on STORY-174. Token list must be extended. | Cycle-close codification; extend AC-174-008 token list |
| PG-MERGE-AUTH-SUBAGENT-CLASSIFIER | Subagent cannot execute --admin merge on relayed human consent; orchestrator-direct attempt also denied on unnamed --admin bypass. Resolution path = human-direct in main thread (per D-463). Codify at cycle-close as AC for E-11 follow-up story. | Cycle-close codification |
| PG-DEMO-JSON-FABRICATION | Demo-recorder produced illustrative JSON/enum values by hand rather than deriving from real serialized output; 3 occurrences in feature-iec104 (FIX-F5-001 report R2 F5R2-02, FIX-P4-001 ×3 artifacts R3 F-B1). Codify at cycle-close: demo-recorder must generate JSON evidence from actual cargo-run/test serialization, and must reference only real enum variants. Vehicle: cycle-close lessons + demo-recording skill update. | Cycle-close lessons codification + demo-recording skill update |
| B-001 | (MINOR, doc-only, non-blocking) PRD RTM entry for BC-2.19.006 carries stale title text; code behavior correct. Surfaced by F7 consistency audit (consistency-audit.md). | Cycle-close spec-doc-currency sweep |
| B-002 | (MINOR, doc-only, non-blocking) BC-2.19.002 PC-2 references T0814 (superseded by BC-2.19.026 PC-4 for reserved TypeIDs); no code impact. Surfaced by F7 consistency audit. | Cycle-close spec-doc-currency sweep |
| STORY-164/165 input-hash staleness | STORY-164 and STORY-165 report STALE input-hashes per `bin/compute-input-hash --scan`; pre-feature stories, out of feature-iec104 F7 gate scope. Separate re-baseline pass required. | Separate re-baseline pass (not gating release) |
| feature-iec104 RELEASE-PENDING | v0.13.0 MINOR cut deferred by human direction (D-470); develop=0b65e8e (13 unreleased). Resume via /vsdd-factory:release when authorized. | v0.13.0 release cut — awaiting human authorization |
| PR-407-FORK-RELEASE-OPS | external ArcavenAE PR #407 security-triaged SAFE-WITH-CHANGES (D-472; triage at .factory/planning/pr-407-security-triage.md); DEFERRED — governance decision (adopt upstream? request 3 changes? decline?) + fork-PR workflow-approval pending. Resume without re-running security review. | governance decision when authorized |

---

## Session Resume Checkpoint

**D-472 PR #407 (external fork ArcavenAE/wirerust, fork-friendly release-ops, 2221 adds/15 files: signing/Homebrew/sync/signing-guard CI) security-triaged (2026-07-18): SAFE-WITH-CHANGES, 0 blocking vulns. All author claims verified (SHA-pinning, CWE-77 env-binding, no pull_request_target, inert-by-default). 3 required-if-adopted changes: (1) sed-escape $VERSION in create-app.sh:236; (2) resolve bundle-id com.arcavenae.wirerust; (3) confirm Release trigger restricted to protected v* tags. DEFERRED by human — PR left OPEN, no disposition; governance question (adopt fork release-ops upstream?) unresolved. Triage: .factory/planning/pr-407-security-triage.md (not to be re-done). trajectory-tail →0→0→0→0**

Prior checkpoint (D-471 E2E IEC-104 coverage merged, 2026-07-17) archived to `cycles/feature-iec104/session-checkpoints.md`.

- **Date:** 2026-07-18. Position: D-472 PR #407 security-triaged SAFE-WITH-CHANGES, DEFERRED by human; feature at F7-CONVERGED, RELEASE HELD; v0.13.0 cut deferred pending human auth. develop=0b65e8e (13 unreleased). trajectory-tail →0→0→0→0
- **Ground truth:** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `0b65e8e`. DRIFT-BACKMERGE-SQUASH-001 still applies. 13 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2) + STORY-170 (PR #404 0bd93f8) + STORY-171 (PR #405 1a64380) + STORY-172 (PR #406 d64e5fe) + STORY-173 (PR #408 084ff93) + STORY-174 (PR #409 547deba) + FIX-P4-001 (PR #410 7e95f71) + FIX-F5-001 (PR #411 9c5aa9a) + FIX-F5-002 (PR #412 b356545) + FIX-F5-003 (PR #413 9eab53f) + FIX-F5-004 (PR #415 b36b884) + e2e-iec104-fixtures (PR #416 0b65e8e).
- **Wave status:** Waves 76–83 DELIVERED (D-441/443/445/447/448/455/458/463): STORY-167..174. Wave-83 SATISFIED. F4 COMPLETE. FIX-P4-001 DELIVERED (D-464). F5 CONVERGED (D-468): 5 rounds, NITPICK_ONLY; FIX-F5-001..004 all delivered. F6 PASS (D-469). F7 CONVERGED (D-470): 5/5 dims PASS; holdout 0.99; RELEASE HELD. D-471 E2E IEC-104 coverage merged (4 fixtures + analyzer-level real-pcap test).
- **Remaining delivery sequence:** (release HELD) v0.13.0 MINOR release cut when human authorizes (via /vsdd-factory:release), then cycle-close lessons codification (B-001/B-002/TypeID-45/process-gaps). PR #407 security-triaged SAFE-WITH-CHANGES and DEFERRED by human — governance decision pending (no re-triage needed).
- **Carry-forwards:** See Active Carry-Forwards table above.
- **Spec versions:** BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 / STORY-INDEX v3.76 / dep-graph v3.9 (137 edges).
- **Resume command:** `/vsdd-factory:release` (when authorized) or `/vsdd-factory:next-step`

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001.

---

## Historical Content

| Content | Location |
|---------|----------|
| **Decision Log D-302..D-436 (exhaustive)** | `cycles/history/decision-log-archive.md` (fix-tls through feature-protocol-coverage through v0.12.1) |
| **feature-iec104 decisions archive (exhaustive)** | `cycles/feature-iec104/decisions-archive.md` (D-437 through D-458: F1 engine triage through F4 delivery) |
| **Resolved Open Items (pre-feature-iec104)** | `cycles/history/open-items-archive.md` |
| **Resolved Carry-Forwards (feature-iec104)** | `cycles/feature-iec104/blocking-issues-resolved.md` (IEC104-FINDING-DIRECTION-001; F5R2-01/02/F-B1/F-B2) |
| **Phase Progress granular rows (F4 waves/adversary/fixes)** | `cycles/feature-iec104/phase-progress-archive.md` (D-451 burst, wave-79..83, STORY-172/173/174 per-story adversary, FIX-P4-001/F5-001..004) |
| **Convergence Trajectory (F4 per-story + F5 phase)** | `cycles/feature-iec104/convergence-trajectory.md` |
| feature-iec104 F2 convergence report | `cycles/feature-iec104/adversarial/f2-convergence-report.md` (12 passes, CONVERGED P10/P11/P12, D-438) |
| feature-iec104 F2 gate review (first-frame guard) | `cycles/feature-iec104/adversarial/f2-first-frame-guard-review.md` (CLEAN; 2 LOW applied; D-439) |
| feature-iec104 MITRE pin confirmation | `cycles/feature-iec104/research/f2-mitre-pin-confirmation.md` (8 techniques CONFIRMED-AT-v19.1; D-439) |
| Session checkpoints (feature-iec104, all prior) | `cycles/feature-iec104/session-checkpoints.md` (waves 76–83 era + D-471 E2E checkpoint) |
| Wave 75 gate files | `cycles/wave-75/wave-gate/` (gate-summary.md D-435, code-review.md, findings.md) |
| Wave 75 lessons + process-gap ledger | `cycles/wave-75/lessons.md` + `cycles/wave-75/process-gap-ledger.md` |
| Wave 74 gate files | `cycles/wave-74/wave-gate/` (gate-summary.md D-432) |
| Wave 73 gate files + lessons | `cycles/wave-73/wave-gate/` + `cycles/wave-73/lessons.md` |
| Wave 72 gate files + lessons | `cycles/wave-72/wave-gate/` (gate-summary.md, code-review.md, demo-evidence/) + `cycles/wave-72/lessons.md` |
| Wave 71 gate files + lessons + checkpoints | `cycles/wave-71/wave-gate/` + `cycles/wave-71/session-checkpoints.md` + `cycles/wave-71/lessons.md` |
| Wave 70 gate files + checkpoints | `cycles/wave-70-story-149/wave-gate/` + `cycles/wave-70-story-149/session-checkpoints.md` |
| Burst history + lessons (maint-2026-07-06) | `cycles/maint-2026-07-06/burst-log.md` + `cycles/maint-2026-07-06/lessons.md` |
| Session checkpoints + lessons (maint-2026-07-08) | `cycles/maint-2026-07-08/session-checkpoints.md` + `cycles/maint-2026-07-08/lessons.md` |
| Wave 82 gate files | `cycles/wave-082/wave-gate/` (gate-summary.md D-458, code-review.md) |
| Wave 83 gate files | `cycles/wave-083/wave-gate/` (gate-summary.md D-463, code-review.md) |
| STORY-174 per-story convergence report | `cycles/feature-iec104/STORY-174/convergence-report.md` (7 passes, CONVERGED P5/P6/P7, D-462) |
| feature-iec104 F5 adversarial reviews | `.factory/phase-f5-adversarial/round-1-review.md` through `round-5-review.md`; `convergence-summary.md` (D-468) |
| feature-iec104 F6 gate verdict + hardening artifacts | `.factory/phase-f6-hardening/f6-gate-verdict-iec104.md` (D-469 PASS); `kani-results.md`, `fuzz-results.md`, `mutation-results.md`, `security-scan-results.md` |
| feature-iec104 F7 convergence artifacts | `.factory/phase-f7-convergence/delta-convergence-report.md` (D-470 CONVERGED); `traceability-chain-delta.md`; `consistency-audit.md` |
