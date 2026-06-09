---
pipeline: V0.1.0_RELEASED
phase: complete
product: wirerust
mode: brownfield
timestamp: 2026-06-08T00:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_started: "2026-05-21"
phase_3_completed: "2026-05-31"
phase_3_to_4_gate: PASSED
phase_4_started: "2026-06-01"
phase_4_completed: "2026-06-01"
phase_4_to_5_gate: "PASSED (human-approved 2026-06-01, conditioned on HS-043 regression tests — merged PR #172)"
phase_5_started: "2026-06-01"
phase_5_completed: "2026-06-01"
phase_6_started: "2026-06-02"
phase_6_completed: "2026-06-02"
phase_6_to_7_gate: "PASSED (human-approved 2026-06-02)"
phase_7_to_release_gate: "PASSED (human-approved 2026-06-08 — Approve → release-prep)"
adversary_gate: SATISFIED
develop_head: 256a490
main_head: 2e8d256
released_version: v0.1.0
released_at: "2026-06-08"
release_tag: v0.1.0
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.1.0
release_commit: 2e8d256
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 48
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
convergence_trajectory: "P1-MED|P2-MED|P3-HIGH+LOW|P4-MED|P5-ZERO|P6-HIGH+MED|P7-MED+LOW|P8-HIGH|P9-ZERO|P10-MED+MED+LOW|P11-MED+LOW|P12-CLEAN(1/3)|P13-CLEAN(2/3)|P14-CLEAN(3/3)-GATE-SATISFIED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
consistency_audit: CONSISTENT
input_drift_check: "CLEAN — MATCH=51/STALE=0 (post F7-100 consistency sweep D-028; STORY-091 no-inputs ERROR pre-existing; 3 hashes recomputed: STORY-097 cb2c82d, STORY-098 8b39dcb, STORY-099 5185063)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline: COMPLETE AND RELEASED.** The full 7-phase VSDD pipeline (brownfield cycle v0.1.0-greenfield-spec) is COMPLETE and RELEASED. wirerust v0.1.0 published 2026-06-08. Annotated tag `v0.1.0` on main commit `2e8d256` (gitflow-proper). GitHub Release live with 4 cross-platform binaries (linux x86_64, macos arm64, macos x86_64, windows msvc); run 27155277051 all jobs success.

**Summary:** 48 stories delivered (v0.1.0), 219 BCs (217 greenfield + 2 F2), 21 VPs (21 locked, 0 draft), 1147 tests green, holdout mean 0.949, adversary convergence 6 PASS / 1 non-blocking CONCERN (Performance — no v0.1.0 SLA). F2+F3+F4+F5+F6 complete for issue #100 (STORY-097/098/099 delivered; VP-021 LOCKED). F7 delta convergence NEXT.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements; trajectory: `17→…→0→0→0` |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 49 stories / 11 epics / 27 waves; story-adversary 3/3 SATISFIED; input-hash drift CLEAN |
| Phase 3 — TDD Implementation | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves; develop HEAD 6158e6e (PR#170); detail: cycles/phase-3-tdd/ |
| Phase 4 — Holdout Evaluation | **PASSED** 2026-06-01 | mean 0.949, 0 must-pass <0.6; HS-043 real defect fixed (PR #171/#172); detail: cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md |
| Phase 5 — Adversarial Refinement | **PASSED** 2026-06-01 | Adversary gate 3/3 + secondary review COMPLETE; 4 fix-PRs; trajectory: `P1-MED→…→P14-CLEAN-GATE` |
| Phase 6 — Formal Hardening | **PASSED** 2026-06-02 | 8 Kani VPs proven; 6 proptest VPs; fuzz VP-008 21.7M execs 0 crashes; mutation targets met; RUSTSEC-2025-0119 FIXED; 20 VPs LOCKED (614e0e0) |
| Phase 7 — Convergence | **PASSED + RELEASED** (human-approved 2026-06-08) | 6 PASS / 1 CONCERN (Perf — non-blocking); 1126 tests; consistency CONSISTENT (8/8); 20 VPs locked |
| Release — v0.1.0 | **RELEASED** 2026-06-08 | GitHub Release; 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); run 27155277051 all jobs success |

## Session Resume Checkpoint (2026-06-09 — F7 consistency CONFIRMED CONSISTENT via 2 audits; human authorization gate NEXT)

**POSITION:** F4 DELIVERED (STORY-097/098/099 merged). F5 CONVERGED (D-026, 3-round hybrid). F6 PASS (D-027) — mutation 100% effective kill, VP-021 LOCKED (21/21 VPs verified, 0 draft). F7 consistency audit 1: 8 discrepancies found + fixed (D-028). F7 consistency audit 2 (confirmation re-audit): all 8 D-028 findings RESOLVED; caught + fixed 1 new HIGH (coverage-matrix v1.2→v1.3, VP-021 tool-column reclassified to match VP-INDEX Kani 8/proptest 7/fuzz 1/int-unit 5=21, D-029). F7 consistency CONFIRMED CONSISTENT. input-hash MATCH=51/STALE=0. develop HEAD `256a490`. NEXT = F7 human authorization gate (final).

**VERIFIED-CLEAN FACTS:**
- main HEAD `2e8d256` — v0.1.0 release commit; annotated tag `v0.1.0`
- develop HEAD `256a490` — post PR #201 stale-comment sweep (F-R2-001/002)
- 1147 tests green; clippy clean; fmt clean; 21 VPs locked (VP-021 locked @256a490)
- Feature #100 F2+F3+F4+F5+F6+F7(consistency x2): BC-2.09.007 v1.1.1 / BC-2.09.006 v1.4 / BC-2.04.055 v1.0.1 / VP-021 LOCKED (proof_file_hash=207d3f68) + STORY-097/098/099 completed; 219 BCs / 21 VPs / 52 stories
- Factory-artifacts: F6 gate committed (D-027); F7 consistency sweep 1 committed (D-028); F7 confirmation re-audit committed (D-029) — verification-coverage-matrix v1.3 (Totals 8/7/1/5=21 internally consistent and VP-INDEX-aligned)
- input-hash drift CLEAN: MATCH=51/STALE=0 (STORY-097 cb2c82d, STORY-098 8b39dcb, STORY-099 5185063)
- holdout D5 PASS mean 0.99

**OPEN POST-RELEASE ITEMS (do NOT lose):**
- #100 (thread pcap timestamps): F7 consistency CONSISTENT — human authorization gate NEXT
- #101 (FP/TP rate characterization): OPEN-DEBT — corpus-dependent; blocks #103
- #103 (size-symmetry evasion discriminator): DEFERRED — needs labelled corpus
- STORY-091: draft, P1, 5 pts, E-11 — anchor-validation tooling; deferred to next cycle
- ACTION-PIN-001: dtolnay/rust-toolchain @stable/@nightly intentionally exempt from pin gate
- Drift items: O-07, O-08, F-W25-S088-P6-001
- RUSTSEC-2026-0097: accepted-transitive; revisit when tls-parser bumps phf→0.12+
- Phase-5 tech-debt (P3): CR-002/003/005/006/007/009/012 — see tech-debt-register.md
- [process-gap DRAFT] DF-SIBLING-SWEEP-001 propagation shadow: HIGH-001's sweep corrected spec files + live test assertions but missed 8 doc-comment lines in 2 test files republishing the false BC date vector claim. R2 re-pass + PR #201 AI review caught them. Candidate codification: extend DF-SIBLING-SWEEP-001 source-docstring-propagation to explicitly include test-file doc comments AND inline comments citing canonical vectors; consider ts_sec<->ISO arithmetic lint from ADV-F5-OBS-001. Do not lose — candidate for lessons.md.
- [process-gap DRAFT] VP-lock propagation checklist (D-028): when a VP is locked, required sweep — VP file frontmatter, VP-INDEX, verification-coverage-matrix, verification-architecture, all BCs citing the VP (VP-anchor prose), AND recompute input-hash of every story listing the VP as an input. This is the 2nd propagation-shadow process-gap this cycle (1st: DF-SIBLING-SWEEP-001). Candidate DF policy.

**RESUME PROTOCOL (for F7 human authorization gate):**
1. `vsdd-factory:factory-worktree-health` — BLOCKING
2. Read `STATE.md` — confirm D-029 committed, F7 consistency CONFIRMED CONSISTENT (2 audits), MATCH=51/STALE=0
3. Present F7 convergence summary to human for authorization gate

Prior checkpoint archived: cycles/v0.1.0-greenfield-spec/session-checkpoints.md.

## Decisions Log

| ID | Decision | Date | Rationale |
|----|----------|------|-----------|
| D-001 | Brownfield mode (target == reference) | 2026-05-19 | No parallel reference repo; in-repo formalization only |
| D-002 | DTU not required | 2026-05-20 | No external service clones needed per dtu-assessment |
| D-003 | CI hotfix: cargo audit shell step | 2026-05-22 | rustsec/audit-check@v2.0.0 fails on push events; PR #111 |
| D-004 | Nightly pin nightly-2026-05-21 is periodic-maintenance | 2026-05-22 | Bumping requires verifying fuzz build; do NOT automate |
| D-005 | Demo recordings local-only (gitignored) | 2026-05-22 | factory-artifacts gitignores cycles/**/demos/; 49 prior files untracked |
| D-006 | Wave-20/STORY-076 real merge SHA is e5cb2b1 (PR #157). Two earlier recorded SHAs corrected. | 2026-05-29 | Orchestrator supplied SHA before actual merge |
| D-007 | Deferred-item cleanup: DF-16.B closed; OBS-7 closed; 4 governance policies codified. | 2026-05-30 | STATE.md deferred-item cleanup burst |
| D-008 | STORY-079 input BC corrected v1.2→v1.3; hash not recomputed at time. Re-validated at Phase-4 gate. | 2026-05-30 | STORY-079 Pass-1 adversarial review F-002 |
| D-009 | ADV-HS043-P02-MED-001 accepted offline scope; high-water-clock fix rejected. Human-approved 2026-06-01. | 2026-06-01 | Phase-5 HS043-pass-2 disposition |
| D-010 | CR-004 REFUTED — false positive. serde_json Map=BTreeMap; byte-identical JSON verified. | 2026-06-01 | Phase-5 secondary review CR-004 disposition |
| D-011 | Inter-phase P2 cleanup: CR-010/CR-001/CR-011 closed (PRs #176/#177/#178); develop 68137b4b→eab2eb1. | 2026-06-01 | Three P2 items delivered between Phase 5 close and Phase 6 start |
| D-012 | VP-002 upgraded JUSTIFIED→PROVEN: pure select_gaps extraction + 2 Kani harnesses (180 checks SUCCESSFUL). PR #183. | 2026-06-02 | CRITICAL anti-evasion release-build silent-overwrite risk discharged |
| D-013 | indicatif bumped 0.17→0.18 (PR #185); RUSTSEC-2025-0119 (unmaintained) resolved. --ignore entry removed. | 2026-06-02 | Phase-6 security hardening; no API breakage |
| D-014 | RUSTSEC-2026-0097 (rand 0.8.5 unsound) accepted-transitive: path tls-parser→phf 0.11→rand; upstream-only fix; unreachable (build-time codegen, deterministic seed). --ignore kept. Revisit when tls-parser bumps phf→0.12+. | 2026-06-02 | Phase-6 security scan disposition |
| D-015 | Mutation scope extended to reassembly modules (SS-04): flow 100%, segment ranges_overlap 9/9, mod 98.54%. 16 genuine survivors killed (PR #184); 3 proven-equivalent mutants remain. | 2026-06-02 | PG-1 remediation — CRITICAL anti-evasion modules now mutation-verified |
| D-016 | All 20 VPs locked (verification_lock:true, proof_completed_date:2026-06-02); module-criticality frozen:true; tag phase-6-verified-2026-06-02. Factory commit 614e0e0. | 2026-06-02 | Phase-6 formal hardening gate closure |
| D-017 | NFR catalog validated under DF-VALIDATION-001 (71/79 VALID, recommendation KEEP). Catalog corrected v1.2→v1.3. nfr-story-map.md v1.1 authored. Criterion-38 traceability gap CLOSED: 43 stories +95 nfr: refs. | 2026-06-08 | Phase-7 pre-gate NFR remediation burst |
| D-018 | Human approved Phase-7 convergence gate (6 PASS / 1 non-blocking CONCERN — Performance). Proceed to release-prep then vsdd-factory:release for v0.1.0 tag. | 2026-06-08 | Phase-7 human gate approval |
| D-019 | Corrected main-branch staging to gitflow-proper. main force-reset; branch protection added (PR required, 8 status checks, force-push blocked); v0.1.0 staged via release/v0.1.0 → PR #189 → main merge 8928398. Rule codified in CLAUDE.md (PR #188). | 2026-06-08 | Gitflow main-branch correction |
| D-020 | Published wirerust v0.1.0. Annotated tag v0.1.0 on main 2e8d256 (gitflow-proper). release.yml built + attached 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); GitHub Release live; run 27155277051 all jobs success. CHANGELOG [0.1.0] notes. Human-authorized publish. | 2026-06-08 | v0.1.0 release |
| D-021 | DI-001 closed — release.yml Node20 actions migrated to Node24, SHA-pinned exact soak-clear versions (upload-artifact@043fb46d1a93 v7.0.1, download-artifact@3e5f45b2cfb9 v8.0.1, gh-release@b4309332981a v3.0.0); dependabot.yml added (cargo 7d/30d-major + github-actions 7d cooldowns, jira-cli soak convention). Fix on develop only by design (gitflow — CI change reaches main at v0.1.1; release.yml runs at tag-time; dependabot reads from default branch develop). PR #192 → develop (2fe3440). Validated: workflow_dispatch run 27159378751, 4 build jobs green, upload-artifact@v7 attached all 4 artifacts. | 2026-06-08 | DI-001 Node20→Node24 migration |
| D-022 | Post-release issue work — Cohort A (#100–#104) re-validated against current develop under DF-VALIDATION-001 (all 5 STILL-VALID, line refs refreshed). Fixed 2 quick-fix bugs via TDD fix-PRs: #104 (SNI control-byte summary for mixed control+non-ASCII values, BC-TLS-037, PR #194 cf21168) and #102 (cap weak-cipher evidence vec at 64 + elision; reclassified low-severity hardening NOT CWE-405/DoS, PR #195 153f225). Both closed. Remaining: #100 FEATURE-MODE; #101 OPEN-DEBT; #103 DEFERRED. Dependabot now live (D-021); PR #193 pending disposition. | 2026-06-08 | Cohort A post-release issue triage and quick fixes |
| D-023 | Supply-chain hardening via HYBRID cross-model adversarial review of dependabot PR #193 (actions/checkout 6→6.0.2). Reviewers: Claude `adversary` agent + Gemini CLI (gemini 0.44.1, genuine non-Claude model-family diversity — factory's first true cross-family adversary). Both converged on verdict (b): close #193, SHA-pin instead. The hybrid ALSO caught Gemini hallucinations (fabricated 'pcap-fixture-as-version-string' red flag; wrong guessed SHA 11bd719) — discarded after verification; real SHA resolved from GitHub API. Findings: prior SHA-pin pass (D-021) was only 3/8 complete. Action: closed #193; SHA-pinned 4 more actions (checkout de0fac2e #v6.0.2, Swatinem/rust-cache c1937114 #v2.9.1, EmbarkStudios/cargo-deny-action bb137d7a #v2.0.20, amannn/action-semantic-pull-request 48f25628 #v6.1.1) across ci.yml+release.yml → 7/8 pinned; added 'Action pin gate' CI job enforcing SHA pins (fails on tags); documented policy in CLAUDE.md; gate added to main's required status checks (9 total). PR #196 → develop 77fd45f. | 2026-06-08 | Supply-chain SHA-pin hardening + enforcement gate |
| D-024 | Issue #100 Feature Mode F2+F3 complete — created BC-2.09.007 (Finding.timestamp provenance; ss-09), BC-2.04.055 (on_data timestamp parameter; ss-04), VP-021 (timestamp-provenance-threading; draft/unverified; integration+proptest); updated BC-2.09.001/006 (v1.3) + BC-2.01.005 (v1.6, O-01 resolved at spec level); created STORY-097/098/099 (waves 28-30, acyclic chain 097→098→099, epic E-12). All indexes updated: 219 BCs / 21 VPs / 52 stories. F4 TDD implementation next (STORY-097→STORY-098→STORY-099). | 2026-06-08 | Issue #100 Feature Mode F2+F3 spec + story decomposition delta |
| D-025 | Issue #100 F5 hybrid adversarial review (Claude adversary + Gemini cross-model) returned NOT-CONVERGED: 1 HIGH + 2 MED findings, all spec-corpus. Spec-corpus fix burst: ADV-F5-HIGH-001 — BC-2.09.007 v1.0→v1.1 date-vector correction (ts_sec=1_000_000 maps to 1970-01-12T13:46:40Z, not 2001-09-08; 6-file sweep confirmed); ADV-F5-MED-001 — STORY-098 v1.0→v1.1 emission-site count corrected 4→3; ADV-F5-MED-002 — STORY-099 v1.0→v1.1 AC-002 rewrite. BC-2.09.006 v1.3→v1.4 (delta-analysis date-vector fix). Input-hash recompute: 6 stories rewritten (STORY-001/069/070/097/098/099); post-recompute drift CLEAN MATCH=51/STALE=0. ADV-F5-LOW-002 test-hardening PR #200 in flight on develop (not a spec-corpus item). F5 NOT-CONVERGED — clean re-pass pending. | 2026-06-08 | Issue #100 F5 spec-corpus fix burst + input-hash recompute |
| D-026 | Feature #100 Phase F5 scoped adversarial review CONVERGED after 3 rounds (Claude primary + Gemini cross-model hybrid). Round 1: 1 HIGH (BC-2.09.007 date vector) + 2 MED spec-corpus. Gemini secondary added 0 valid source defects (2 refuted: 1 diff-blindness hallucination, 1 last_ts==0 concern refuted at source) and confirmed test-rigor findings. Spec-corpus fixes (D-025) + 2 test fix-PRs (#200 LOW-002 exact-value binding; #201 F-R2-001/002 stale-comment sweep, AI review caught 2 extra stale lines). Round 3 clean: 0 findings, input-hash MATCH=51/STALE=0. develop HEAD 256a490. | 2026-06-08 | Issue #100 F5 convergence — 3-round hybrid adversarial |
| D-027 | Feature #100 Phase F6 targeted hardening PASS. Mutation --in-diff 100% effective kill (30/30 killable; 2 equivalent survivors at lifecycle.rs:62 documented, independently re-confirming F5 close-flush unreachability). Fuzz 0 crashes; Kani justified-skip (inline-chrono totality via closed-form+proptest+boundary, no debug-guard anti-pattern); cargo audit/deny PASS (RUSTSEC-2026-0097 known-accepted); 1147 tests green. VP-021 LOCKED (verified, lock=true, @256a490) — all 21 VPs now verified, 0 draft. develop HEAD 256a490. | 2026-06-09 | Issue #100 F6 targeted hardening gate PASS + VP-021 locked |
| D-028 | Feature #100 F7 gate fresh-context consistency audit found 8 discrepancies (3 HIGH: VP-021 lock not propagated to coverage-matrix/architecture [F-001], VP-021 proof_file_hash null [F-002], STORY-099 input-hash STALE from VP-021 v2.0 lock [F-003]; 3 MED: delta-analysis §4.5 site-count, BC VP-anchor draft prose, STORY-099 pr-description stale ts; 2 LOW: story status draft→completed, STORY-098 BC-2.04.055 co-trace). All fixed: proof_file_hash=207d3f68; verification-coverage-matrix v1.2 + verification-architecture v1.3 + BC-2.09.007 v1.1.1 + BC-2.04.055 v1.0.1 propagated verified; STORY-097/098/099 status completed; input-hashes recomputed CLEAN (MATCH=51/STALE=0; STORY-097 cb2c82d, STORY-098 8b39dcb, STORY-099 5185063). Lesson: VP-lock bursts must propagate to coverage-matrix, architecture, referencing-BC VP-anchors, AND recompute consuming-story input-hashes (VP files are story inputs). This is the 2nd propagation-shadow process-gap this cycle — candidate DF policy (VP-lock propagation checklist). | 2026-06-09 | Issue #100 F7 consistency sweep — VP-021 lock propagation (8 findings all fixed) + input-hash recompute |
| D-029 | Feature #100 F7 consistency confirmation re-audit: all 8 D-028 findings RESOLVED; caught + fixed 1 new HIGH introduced by the D-028 fix burst — verification-coverage-matrix v1.2→v1.3: VP-021 row reclassified to proptest column to match VP-INDEX invariant (Kani 8 / proptest 7 / fuzz 1 / integration-unit 5 = 21); prior v1.2 had proptest=6/integration=6 contradicting VP-INDEX. Column sums and per-module totals internally consistent post-fix. F7 consistency now CONFIRMED CONSISTENT. Lesson reinforces VP-lock propagation checklist (D-028): the coverage-matrix tool-column counting convention must match VP-INDEX — add explicit tool-column verification step. | 2026-06-09 | Issue #100 F7 confirmation re-audit — coverage-matrix VP-021 tally aligned to VP-INDEX; F7 CONSISTENT |

## Blocking Issues

None open.

## Drift Items / Tech Debt Pointers

All items require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Full tech-debt register: `.factory/tech-debt-register.md`.

| ID | Summary | Status |
|----|---------|--------|
| CR-004 | Inner-HashMap JSON non-determinism claim | REFUTED — false positive |
| ADV-HS043-P02-MED-001 | Idle-flow expiry monotonic watermark stalls on multi-epoch captures | ACCEPTED — gated on live-capture support |
| O-07 | rayon declared in Cargo.toml but unused | OPEN P2 |
| O-08 | dns.rs module doc-comment stale | OPEN P3 |
| F-W25-S088-P6-001 | AC-004 warning .contains() — weaker than count-assertion | OPEN — target next main.rs touch or accept |
| RUSTSEC-2026-0097 | rand 0.8.5 unsound (transitive via tls-parser→phf 0.11); upstream-only fix path | ACCEPTED-TRANSITIVE — revisit when tls-parser bumps phf→0.12+ |
| DI-001 | release.yml Node20 actions migrated to SHA-pinned Node24 (upload-artifact@v7.0.1, download-artifact@v8.0.1, gh-release@v3.0.0); dependabot.yml added. PR #192 → develop (2fe3440). | RESOLVED — closed 2026-06-08 |
| FE-001 | pcapng input format not supported (.pcap-only); parked v2 idea (2026-06-08 demo) — see tech-debt-register.md Future Enhancements | deferred / v2 / not-filed |
| ACTION-PIN-001 | dtolnay/rust-toolchain @stable and @nightly remain branch-ref (not SHA-pinned) — intentionally exempt in the Action pin gate (toolchain installer, channel-selected). Tracked for separate resolution (decide: SHA-pin + toolchain: input, or accept exemption). | OPEN P3 — low priority |

## Cycle-Close Follow-Up Items

| ID | Description | Status |
|----|-------------|--------|
| PROCESS-GAP-P5-001 | Systemic anchor/coherence drift across 11 adversarial passes | CLOSED — STORY-091 disposition committed 2026-06-01 |
| PG-1 | tooling-selection.md mutation scope omits CRITICAL reassembly modules (SS-04) | CLOSED — H-1 fix: tooling-selection.md body now records SS-04 reassembly mutation scope + Phase-6 outcomes (PR #184); 2026-06-08 |
| PG-2 | CRITICAL VP "justified" via debug-only guard — caught at human gate | CLOSED — lesson recorded; hardening-gate checklist recommendation in lessons.md |
| PG-3 | Stale local develop — agents must branch off origin/develop | CLOSED — lesson recorded; DF-DEVELOP-FRESHNESS-001 governs |
| PG-4 | Dependabot does not auto-convert action tags to SHAs — SHA-pin policy was unenforceable by cooldown alone; prior pass (D-021) left 5/8 unpinned [codified] | CODIFIED/CLOSED — 'Action pin gate' CI check + CLAUDE.md policy enforced by PR #196; 2026-06-08 |

## Governance Policy

Full policy text: `.factory/policies.yaml`. Detail: `cycles/phase-3-tdd/governance-policy-detail.md`.

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

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`. SS-03 gap in BC numbering intentional.
- Phase 0 ground truth: `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`. Wave history: `cycles/phase-3-tdd/convergence-trajectory.md`. Phase 1/2 adversary detail: `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`. Phase 4 holdout: `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`. Phase 6 hardening evidence: `cycles/v0.1.0-greenfield-spec/hardening/`.
- GitHub issues Cohort A dispositioned (D-022): #104 CLOSED (PR #194), #102 CLOSED (PR #195), #100 FEATURE-MODE pending, #101 OPEN-DEBT, #103 DEFERRED. Dependabot PR #193 CLOSED (SHA-pin preferred, D-023). Supply-chain: 7/8 actions SHA-pinned; pin gate enforced (PR #196).
