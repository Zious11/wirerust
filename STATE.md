---
pipeline: V0.4.0_RELEASED
phase: feature-f7
active_feature: null
product: wirerust
mode: brownfield
timestamp: 2026-06-10T22:45:00Z
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
develop_head: fb2c875
main_head: 90aa91e
released_version: v0.4.0
released_at: "2026-06-10"
release_tag: v0.4.0
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.4.0
release_commit: 90aa91e
prior_released_version: v0.3.0
prior_released_at: "2026-06-09"
prior_release_tag: v0.3.0
prior_release_commit: 9ef5af1
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 54
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
convergence_trajectory: "P1-MED|P2-MED|P3-HIGH+LOW|P4-MED|P5-ZERO|P6-HIGH+MED|P7-MED+LOW|P8-HIGH|P9-ZERO|P10-MED+MED+LOW|P11-MED+LOW|P12-CLEAN(1/3)|P13-CLEAN(2/3)|P14-CLEAN(3/3)-GATE-SATISFIED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
consistency_audit: CONSISTENT
input_drift_check: "CLEAN — MATCH=57/STALE=0 (post D-045 blemish-1 fix; STORY-104 recomputed bc3863e after BC-2.14.019 v1.3 update; STORY-091 no-inputs ERROR pre-existing)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline: V0.4.0_RELEASED (D-046). Feature #7 COMPLETE AND RELEASED. issue #7 CLOSED.** wirerust v0.4.0 published 2026-06-10T05:12:40Z — Modbus TCP analyzer (port-502, MBAP/FC parse, transaction correlation, 7 ICS MITRE detectors). Full F1-F7 feature-mode cycle complete. Pipeline IDLE — no in-flight feature work.

**Summary:** 58 stories (48 greenfield + 4 F-cycle + 6 F3-new), 353 pts. 244 BCs, 22 VPs (all 22 verified/locked, 0 draft), 1338 tests green, holdout 0.967. develop HEAD fb2c875; main HEAD 90aa91e (v0.4.0). Feature #7: COMPLETE across 2 releases (v0.3.0 multi-tag schema + v0.4.0 Modbus analyzer). develop is ahead of main by 3 non-release chore commits (eb010a1 .gitignore, 92773a4 E2E-pcap tooling, fb2c875 merge — PR #221 local-only E2E pcap tooling). No release content outstanding; branches are NOT divergent in a problematic way (main has no commits develop lacks).

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
| Feature #100 F4→F7 Cycle | **COMPLETE + RELEASED** 2026-06-09 | STORY-097/098/099 delivered; VP-021 LOCKED; D-030 human gate; shipped in v0.2.0 |
| Release — v0.2.0 | **RELEASED** 2026-06-09 | gitflow-proper: release/0.2.0 → PR #208 → 18be1ba; 4 binaries; run 27216925948; GitHub Release published 2026-06-09T15:28:38Z; D-031 |
| Feature #7 F3 — Incremental Stories | **COMPLETE** 2026-06-09 | 6 new stories STORY-100..105, 58 total / 353 pts; wave schedule + 22 holdout scenarios; D-036 |
| Feature #7 F4 Wave 1 — E-13 Multi-Tag Migration | **DELIVERED** 2026-06-09 | STORY-100+101 PR #209 -> develop c846b3b; 1189 tests; 9/9 CI; AI review APPROVE; OPEN: mitre_attack_version F4-PIN; D-037 |
| Release — v0.3.0 | **RELEASED** 2026-06-09 | gitflow-proper: release/0.3.0 → PR #210 → 9ef5af1; 4 binaries; run 27240476896; GitHub Release published; BREAKING: mitre_techniques array (ECS-aligned); F4-PIN resolved ics-attack-19.1; D-038 |
| Feature #7 F4 Wave 2 — E-14 Modbus Core | **COMPLETE** 2026-06-09 | All 4 stories MERGED: PR #211 (STORY-102, 26d58bb), PR #212 (STORY-103, d894464), PR #213 (STORY-104, dba...), PR #214 (STORY-105, dba5f26). 1324 tests. Modbus LIVE. D-042 |
| Feature #7 F5/F6/F7 — Hardening + Convergence | **F7 CONVERGED** 2026-06-09 | F5 CRITICAL timestamp-units (PR #215); F6 Kani 5/5 + fuzz 3.7M/0 + mutation 100% + audit clean (PR #216); F7 e2e + mod-wrappers (PR #217, 70abc27). Consistency 5-shadow sweep FIXED. 1338 tests. Holdout 0.967. D-044 |
| Release — v0.4.0 | **RELEASED** 2026-06-10 | gitflow-proper: release/0.4.0 → PR #219 → main merge 90aa91e; annotated tag v0.4.0; 4 binaries; run 27254720396; GitHub Release published 2026-06-10T05:12:40Z; Feature #7 COMPLETE + issue #7 CLOSED; main back-merged to develop (8e38041). D-046 |

## Session Resume Checkpoint (2026-06-10 — SESSION-CLEAR — v0.4.0 RELEASED — Pipeline IDLE — PR #221 MERGED — PCAP-CORPUS-001 TABLED)

**POSITION:** wirerust v0.4.0 RELEASED (D-046). Feature #7 (Modbus TCP analyzer, issue #7) COMPLETE AND CLOSED. Pipeline IDLE — no in-flight feature work. develop HEAD `fb2c875`; main HEAD `90aa91e` (v0.4.0). develop is ahead of main by 3 non-release chore commits from PR #221 (E2E pcap tooling only — no release content). This is normal between releases; branches are NOT divergent in a problematic way. Next feature is TBD; pick a roadmap item.

**RELEASE HISTORY:**
- v0.1.0 (2026-06-08): greenfield full-cycle baseline (Phases 0-7)
- v0.2.0 (2026-06-09): Feature #100 — pcap timestamp threading to Finding.timestamp (VP-021 Kani-verified)
- v0.3.0 (2026-06-09): Feature #7 Wave 1 — multi-tag MITRE schema migration (BREAKING: mitre_technique→mitre_techniques array; ECS-aligned)
- v0.4.0 (2026-06-10): Feature #7 Wave 2 — Modbus TCP analyzer (port-502; MBAP/FC parse; transaction correlation; 7 ICS MITRE detectors T0855/T0836/T0835/T0831/T0806/T0814/T0888; dual-window rate detection; VP-022 Kani-verified)

**RESUME PROTOCOL FOR NEXT SESSION (BLOCKING — follow in order):**
1. Run `vsdd-factory:factory-worktree-health` — verify .factory/ worktree on factory-artifacts BEFORE any factory reads/writes
2. Read STATE.md (this file) — orient to current state
3. Factory is IDLE — to start new work: (a) pick a roadmap item below, (b) run `vsdd-factory:phase-f1-delta-analysis` for feature-mode work

**OPEN DEPENDABOT PRs (need disposition before next release):**
- #202 actions/checkout bump — REQUIRES SHA-pin per ACTION-PIN policy (do NOT merge tag ref; close and SHA-pin manually)
- #203 serde_json — standard cargo bump; review + merge
- #204 assert_cmd — standard cargo bump; review + merge
- #205 etherparse 0.16→0.20 (4-minor jump) — review API changes before merging
- #206 rayon — standard cargo bump; review + merge
- #207 clap — standard cargo bump; review + merge

**ROADMAP / NEXT FEATURE OPTIONS:**
- Issue #8: DNP3 analyzer (natural next ICS protocol after Modbus)
- Issue #3: C2 beaconing detection
- Issue #4: CSV + SQLite reporters
- Issues #64/#62/#63: reporter improvements
- Issue #6: rayon parallel processing
- Issue #101: FP/TP characterization (OPEN-DEBT; blocked on labelled corpus)
- Issue #103: size-symmetry evasion discriminator (DEFERRED; blocked on labelled corpus)
- FE-001: pcapng support (deferred v2)

**CARRY-FORWARD / OPEN ITEMS:**
- STORY-091: draft, P1, 5 pts, E-11 — anchor-validation tooling; deferred to next cycle
- Drift items: O-07 (rayon unused), O-08 (dns.rs stale doc), F-W25-S088-P6-001
- RUSTSEC-2026-0097: accepted-transitive (rand 0.8.5 via tls-parser→phf 0.11)
- ACTION-PIN-001: dtolnay/rust-toolchain @stable/@nightly intentionally exempt from pin gate (OPEN P3)
- CC-001..CC-004: process-gap codification deferred (DF-SIBLING-SWEEP extension, VP-lock checklist, PROCESS-ARITHMETIC-REVIEW-001, F5 dispatcher-boundary test gap)
- Sub-second rate precision: deferred (needs timestamp_usecs threaded through on_data)
- Terminal per-ID multi-unknown name resolution (BC-2.11.017): deferred
- PCAP-CORPUS-001 (TABLED 2026-06-10): E2E pcap test-corpus storage backend decision. Design complete: `test-pcaps` orphan-branch as control plane (MANIFEST.yaml per-pcap metadata + fetch.sh + run-corpus.sh; tiered smoke/full; sha256-keyed caching). 4SICS ICS-lab captures (4SICS-GeekLounge-151020/151021/151022, 25/134/200 MB) validated v0.4.0 Modbus analyzer (1.55M pps, deterministic, parse_errors 230/2.25M, DoS cap engaged). Backend options: GitHub Releases REJECTED (2 GiB/file cap, 1000-asset limit), Git LFS REJECTED (cost/quota), Google Drive public REJECTED (daily quota lockout in CI), Drive service-account VIABLE (reuse 5 TB, Drive API bypasses interstitial, needs free GCP project + SA JSON secret), Cloudflare R2 RECOMMENDED for 100s GB ($0.015/GB-mo, zero egress), Backblaze B2 cheapest ($0.006/GB-mo + free via Cloudflare CDN). PENDING: human to pick R2/B2 vs Drive-SA. PRECURSOR LANDED (PR #221, fb2c875, 2026-06-10): lightweight E2E pcap reproducibility layer merged to develop — tracked files: `tests/fixtures/E2E-PCAPS.md` (per-pcap index: size/sha256/source-URL-or-generator/protocols/what-it-validates; 4SICS/CS3Sthlm attribution), `bin/fetch-e2e-pcaps` (downloads real captures + regenerates synthetic into gitignored dir, verifies every sha256), `tests/fixtures/mk_modbus_large_pcap.py` (deterministic synthetic modbus-large.pcap generator). LOCAL-ONLY (gitignored under `tests/fixtures/local-samples/`, never committed): 4SICS-GeekLounge-151020/151021/151022.pcap (25/134/200 MB) + modbus-large.pcap (synthetic) + a local README — to reproduce on fresh checkout: run `bin/fetch-e2e-pcaps`. The `.gitignore` rule for `/tests/fixtures/local-samples/` committed in eb010a1. When PCAP-CORPUS-001 is revisited (backend decision made), migrate the E2E-PCAPS.md rows into the orphan-branch corpus manifest. Also: issue #220 filed (cosmetic Modbus write-burst "0s window" display bug, src/analyzer/modbus.rs L608/L615 — OPEN, good-first-issue).

**INPUT-HASH DRIFT (verified 2026-06-10):** MATCH=57 STALE=0 ERROR=1 (STORY-091 pre-existing no-inputs; known).

Prior checkpoint archived: cycles/v0.1.0-greenfield-spec/session-checkpoints.md.

## Decisions Log

D-001..D-029 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.

| ID | Decision | Date | Rationale |
|----|----------|------|-----------|
| D-030 | Human authorized Feature #100 F7 delta convergence gate (2026-06-09): all 5 dimensions PASS (spec novelty 0.0, mutation 100% effective, 0 impl defects, VP-021 locked, holdout 0.99), regression 1147 green, consistency CONSISTENT via 2 audits. Feature #100 CONVERGED and CLOSED. Human elected to release now (v0.1.1 or v0.2.0 — semver disposition pending confirmation). F4->F7 full feature-mode cycle complete. | 2026-06-09 | Feature #100 F7 human authorization gate APPROVED — feature-mode cycle COMPLETE |
| D-031 | Published wirerust v0.2.0 (gitflow-proper). Feature #100 (pcap timestamp threading -> Finding.timestamp) shipped as MINOR. release/0.2.0 -> PR #208 -> main merge 18be1ba; annotated tag v0.2.0 on 18be1ba; release.yml run 27216925948 built+attached 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); GitHub Release live (published 2026-06-09T15:28:38Z, not draft). CHANGELOG [0.2.0] covers #100 feature (PRs #197/#198/#199 + hardening #200/#201) + fixes #102/#104 (PRs #194/#195) + CI/supply-chain (#192/#196). main back-merged into develop (4cfc4c4) — branches in sync, no divergence. | 2026-06-09 | v0.2.0 release — Feature #100 cycle fully closed |
| D-032 | Feature #7 (Modbus TCP analyzer) F1 delta analysis APPROVED by human (2026-06-09). Intent=feature, type=backend, non-trivial -> full F1-F7. Integration: ModbusAnalyzer implements StreamHandler+StreamAnalyzer (like HTTP/TLS), wired into StreamDispatcher (NOT the ProtocolAnalyzer/DNS UDP path). New: src/analyzer/modbus.rs, subsystem SS-14 'Modbus/ICS', VP-022 (Kani P1). Modified (5): dispatcher.rs (HIGH risk — DispatchTarget::Modbus + classify port 502 + VP-004 Kani oracle), mitre.rs (CRITICAL — VP-007 drift guard; T0836 absent + T0855 not in EMITTED_IDS must be fixed atomically), analyzer/mod.rs, main.rs, cli.rs. ADR required: binary-ICS integration + port-only classification as documented exception to ADR-0001 content-first. HUMAN SCOPE DECISIONS: (1) MITRE coverage = FULL cheap set of 6 ICS techniques: T0855 Unauthorized Command Message, T0836 Modify Parameter, T0814 DoS (Diagnostics 0x08 force-listen-only/restart), T0806 Brute Force I/O, T0835 Manipulate I/O Image, T0831 Manipulation of Control; (2) request/response = FULL transaction correlation (per-connection Transaction-ID+Unit-ID+FC table, not stateless-per-PDU); (3) write-burst threshold = CLI-configurable now (--modbus-write-threshold flag, default >10/s sustained >=2s or >20 in 1s window). MITRE type needs a matrix discriminator (ICS T0xxx vs enterprise Txxxx). Research: .factory/research/modbus-tcp-research.md (Modbus.org V1.1b3 + MITRE ATT&CK for ICS sourced). | 2026-06-09 | Feature #7 F1 human gate approval — full F1-F7 scope authorized |
| D-033 | Feature #7 Modbus F2 spec evolution CONVERGED (2026-06-09). 25 SS-14 BCs (BC-2.14.001-025), PRD v1.1 (BC total 244), VP-022 (Kani P1, 3 sub-properties, bcs 001-008), ADR-005 (binary-ICS integration), MITRE MitreMatrix{Enterprise,Ics} discriminator + 7 ICS techniques emitted (T0855/T0836/T0814/T0806/T0835/T0831/T0846; SEEDED 20/EMITTED 13; VP-007 atomic update + positive-coverage obligation). Adversarial convergence: 3 rounds (4 CRIT+8 HIGH round1 → CONVERGED), Claude adversary + consistency-validator. Key scope: single configurable --modbus-write-threshold (1s window, default 10); co-emission cap (most-specific write-technique/PDU, T0855 once/burst); full transaction correlation; desync-safe parse (is_non_modbus bail). VP-INDEX 22 (Kani 9). develop HEAD 4cfc4c4. | 2026-06-09 | Feature #7 F2 spec evolution convergence |
| D-034 | Feature #7 Modbus F2 spec REVISION CONVERGED (2026-06-09). Research-validated design adoptions: (1) dual-window write threshold (burst >20/1s + sustained >10/s/>=2s; microsecond math; wrapping_sub; 2 CLI flags), (2) T0846→T0888 recon fix (SEEDED 21/EMITTED 13), (3) multi-tag `mitre_techniques: Vec<String>` (ADR-006; breaking schema → v0.3.0; cross-cuts SS-09/10/11/14). ~28 BCs revised. Convergence: 3 Claude revision rounds + Gemini cross-model hybrid (2 slices, blind) caught HIGH truncation-bias + off-by-six ADU + BC-018/019 gap + length-gate off-by-one + FC-0x17 T0831 evasion; all fixed. Total 37 real findings resolved. Input-hash MATCH=51/STALE=0 (13 recomputed). develop HEAD 4cfc4c4. NEXT: F2 human gate. | 2026-06-09 | Feature #7 F2 revision — dual-window + T0888 + multi-tag; Claude+Gemini hybrid (D-034) |
| D-035 | Feature #7 F2 gate APPROVED with research-driven RELEASE SPLIT (2026-06-09). Three research-agent reports (bundle-vs-split, multi-tag-schema, decomposition-sequencing) → human decisions: (1) SPLIT the breaking change — v0.3.0 = multi-tag Finding schema migration ISOLATED (SS-09/10/11 + all existing analyzers/reporters/catalog + 6 existing stories STORY-069/070/071/078/079/080), v0.4.0 = Modbus analyzer ADDITIVE on the stable type (SS-14). Rationale: multi-tag motivated-by but not required-by Modbus; isolating the break = honest semver + single clean migration note (Trivy/Zeek precedent). (2) Schema design VALIDATED sound as-is (mitre_techniques flat array == Elastic ECS threat.technique.id; semicolon-CSV de-facto standard; absent-when-empty + canonical order correct). (3) Decomposition = ATOMIC rename (not parallel-change; Rust forces lockstep, no 0.x external consumers), migration FIRST as its own wave merged green, THEN Modbus; 6 shipped stories' tests updated inside the migration commit. ADD-ONS applied: report-envelope mitre_domain='ics-attack' + mitre_attack_version (placeholder 'ics-attack-v15', F4 must pin to authoritative ATT&CK-ICS version covering T0806/T0814/T0831/T0835/T0836/T0855/T0888); CSV empty-string (not null). F2 spec CONVERGED (Claude 3 rounds + Gemini cross-model) + research-validated. Input-hash recomputed: MATCH=51/STALE=0/ERROR=1 (10 stories: STORY-001/002/003/004/005/076/077/078/079/080 — BC-2.11.001/024 changed). NEXT: F3 decomposition — Wave 1 (multi-tag migration → v0.3.0), Wave 2 (Modbus → v0.4.0). | 2026-06-09 | Feature #7 F2 gate APPROVED — release split v0.3.0 schema / v0.4.0 Modbus + ECS schema add-ons |
| D-036 | Feature #7 F3 story decomposition COMPLETE (2026-06-09). 6 new stories, acyclic graph (STORY-100 -> {101 || 102} -> 103 -> 104 -> 105). E-13 Multi-Tag Finding Schema Migration (v0.3.0, Wave 31): STORY-100 (core type+catalog+analyzers migration, 13pts) + STORY-101 (reporters+ECS envelope add-ons, 8pts). E-14 Modbus Analyzer (v0.4.0, Waves 32-34): STORY-102 (MBAP parse+FC classify+VP-022 Kani, 8pts), STORY-103 (flow state+txn correlation, 8pts), STORY-104 (7 detectors+dual-window+co-emission+summary, 13pts), STORY-105 (dispatcher+CLI+VP-004 oracle, 8pts). Migration-first enforced (STORY-102 depends_on STORY-100). 6 existing stories (069/070/071/078/079/080) noted: tests migrated to multi-tag by STORY-100. STORY-INDEX 52->58 stories / 305->353 pts. Wave schedule + 22 wave-holdout scenarios written. F4-pin flag: mitre_attack_version. Input-hash CLEAN: MATCH=57/STALE=0/ERROR=1 (6 hashes written: STORY-100 270dd80, STORY-101 dccf659, STORY-102 6dc856b, STORY-103 4a0438e, STORY-104 4f1ebad, STORY-105 8955282). NEXT: F3 human gate, then F4 TDD Wave 1 (v0.3.0). | 2026-06-09 | Feature #7 F3 story decomposition — 6 stories, 2 epics, wave schedule, 22 holdout scenarios |
| D-037 | Feature #7 F4 Wave 1 (E-13 multi-tag Finding migration) DELIVERED. STORY-100+STORY-101 delivered atomically (Rust compiler forces lockstep on the core-type change) via PR #209 -> develop c846b3b. Finding.mitre_technique:Option<String> -> mitre_techniques:Vec<String> (JSON scalar->array + key rename; CSV column rename + semicolon-join; report-envelope mitre_domain/mitre_attack_version); MITRE catalog 15->21 seeded / 6->13 emitted (+T0888 +5 ICS arms + T0855 gap-fix). 1189 tests green (was 1147; +42), clippy+fmt clean, 9/9 CI, AI review APPROVE. Per-story adversarial convergence: Claude (fixed C-1 mitre_tests-migration + I-1 weakened-assertions) + Gemini cross-model (HIGH 'broken build' REFUTED as hallucination — code compiles green; doc nits fixed). Deferred: O-1 (EMITTED_IDS names 7 ICS not-yet-emitted until Modbus STORY-104 -> phase-5); terminal per-ID multi-unknown name resolution -> STORY-104 (BC-2.11.017). OPEN release-blocker: mitre_attack_version F4-PIN (placeholder ics-attack-v15) must be set to authoritative ATT&CK-ICS version before v0.3.0 tag. develop HEAD c846b3b. | 2026-06-09 | Feature #7 F4 Wave 1 multi-tag migration DELIVERED (PR #209, develop c846b3b) |
| D-038 | Published wirerust v0.3.0 (gitflow-proper) — Feature #7 Wave 1 / E-13 multi-tag Finding schema migration (BREAKING). release/0.3.0 -> PR #210 -> main merge 9ef5af1; annotated tag v0.3.0; release.yml run 27240476896 built+attached 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); GitHub Release live. CHANGELOG [0.3.0] BREAKING: mitre_technique scalar -> mitre_techniques array (ECS-aligned); CSV column rename + semicolon-join; report-envelope mitre_domain=ics-attack + mitre_attack_version=ics-attack-19.1 (F4-PIN resolved: ATT&CK v19.1, all 7 ICS IDs valid, per .factory/research/attack-ics-version-pin.md); MITRE catalog 15->21 seeded / 6->13 emitted (6 ICS staged for Modbus + T0855 gap-fix). 1189 tests green; 9/9 CI. main back-merged into develop (9ef5af1, fast-forward, no divergence). | 2026-06-09 | v0.3.0 release — Feature #7 Wave 1 multi-tag MITRE schema (BREAKING) |
| D-039 | Feature #7 STORY-102 (Modbus MBAP parse + FC classify) per-story adversarial convergence — Claude (implementation airtight: parse safety no-panic/no-OOB, classify_fc total + correct sets, exception biconditional) + Gemini cross-model (caught Kani harness under-binding Claude missed: verify_classify_fc_total had tautological final assert + did not prove undefined-FC->Unknown; strengthened to full 256-value expected-mapping proof; removed tautological assert in exception harness). Spec off-by-one reconciled: VP-022 v1.1 + STORY-102 v1.1 length-gate 253->254 to match authoritative BC-2.14.004 (max Length = UnitID 1 + PDU 253 = 254; was pre-F2-fix stale value). Propagation sweep: BC-INDEX.md BC-2.14.004 title + BC-2.14.013.md precondition + BC-2.14.001.md EC-008 also corrected 253->254 (3 propagation gaps closed). Implementation green: 1224 tests, clippy+fmt clean; VP-022 Kani run deferred to F6. STORY-102 delivering via PR (pr-manager, parallel). Gemini hybrid again caught formal-harness rigor class (tautological asserts) that Claude pass missed. | 2026-06-09 | Feature #7 STORY-102 per-story adversarial convergence + spec off-by-one fix |
| D-040 | Feature #7 Wave 2 progress — STORY-102 (Modbus MBAP parse + FC classify) MERGED via PR #211 (develop 26d58bb); 1224 tests. STORY-103 (Modbus flow state + transaction correlation) converged + delivering: ModbusFlowState 18-field struct, pending table keyed (txn_id,unit_id) bounded 256 drop-not-evict (DoS-resistant), FC-echo response match, exception attribution with original_fc==stored_fc spoof gate, duplicate-in-flight return signal. Per-story convergence: Claude (security perimeters airtight; caught duplicate_inflight_txn false-green — counter wiring is STORY-104 on_data scope, test made honest as return-value signal + STORY-104 obligation filed) + Gemini cross-model (independently CONVERGED). 1247 tests green. STORY-104 OBLIGATION: on_data must increment duplicate_inflight_txn on insert_request overwrite + test it. SS-14 BC input-hashes recomputed (STORY-102 TBD->d5c8642; STORY-104 TBD->56a3714); MATCH=57/STALE=0/ERROR=1. | 2026-06-09 | Feature #7 Wave 2 — STORY-102 merged + STORY-103 converged + SS-14 BC hashes (D-040) |
| D-041 | Feature #7 Wave 2 — STORY-103 (transaction correlation) MERGED via PR #212 (develop d894464). STORY-104 (Modbus detection engine) CONVERGED + delivering: process_pdu with 7 ICS MITRE detectors (T0855/T0836/T0835/T0831/T0806/T0814/T0888), multi-tag union co-emission (1 finding/write PDU, DoS-bounded), dual-window burst(1s)+sustained(>=2s truncation-free microsecond math) rate detection, exception-burst anomaly (per-code 10s window), MAX_FINDINGS cap, summarize 6-key. 1296 tests. Per-story convergence: Claude + Gemini cross-model BOTH independently caught 2 blocking defects (source_ip=None on all findings; exception-window start_ts never anchored -> infinite 10s window) + dead per-flow counters; all fixed + 9 binding tests; re-pass CONVERGED. Strong cross-model agreement (both families found the same root defects). BC-DISCREPANCY-001 resolved: FC 0x17 Read/Write Multiple Registers writes holding registers -> [T0855,T0836]; reconciled BC-2.14.013/014/015 v2.1 so T0836 set=={0x06,0x10,0x16,0x17}==T0831 set, T0835 coil-only {0x05,0x0F}, 0x15 T0855-only. 3 LOW deferred: recon test ==1; DF-TEST-NAMESPACE-001 modbus_detection_tests flat namespace; 0xFF exception sentinel. dual-window/cap/co-emission verified regression-free. STORY-104 input-hash recomputed: 56a3714->e89c401 (BC-2.14.013/014/015 v2.1 inputs changed); MATCH=57/STALE=0/ERROR=1. | 2026-06-09 | Feature #7 Wave 2 — STORY-103 merged + STORY-104 converged + BC-DISCREPANCY-001 (0x17) reconciled |
| D-042 | Feature #7 Wave 2 (E-14 Modbus TCP Analyzer) COMPLETE — all 4 stories delivered, analyzer LIVE end-to-end. STORY-104 (detection engine) MERGED PR #213; STORY-105 (dispatcher integration + CLI) MERGED PR #214 (develop dba5f26). The Modbus analyzer: MBAP parse + FC classify (VP-022 Kani) [102], transaction correlation + bounded pending [103], 7 ICS MITRE detectors + dual-window + multi-tag co-emission + summary [104], dispatcher port-502 Rule-5 + StreamHandler with segment-spanning carry buffer + CLI [105]. 1324 tests green; clippy+fmt clean; all CI green. Per-story adversarial: Claude + Gemini cross-model on EVERY story — caught real defects each pass (STORY-102 Kani-harness tautology; STORY-103 duplicate-counter false-green; STORY-104 source_ip=None + infinite exception window [both models]; STORY-105 serde-rename breaking regression + partial-ADU buffering [both models]). The hybrid repeatedly caught defect classes one model missed; Gemini hallucinations caught by verification. BC-DISCREPANCY-001 (0x17) resolved. DEFERRED to v0.4.0 release hardening: VP-022 + VP-004 Kani run (kani not in local env -> F6); e2e port-502 pcap fixture (F-105-003); DF-TEST-NAMESPACE-001 modbus test mod-wrappers; terminal per-ID multi-unknown name resolution; O-1 EMITTED-naming now satisfied (Modbus emits the 7 ICS techniques). | 2026-06-09 | Feature #7 Wave 2 COMPLETE — Modbus analyzer live (STORY-102/103/104/105) |
| D-043 | Feature #7 v0.4.0 F5 combined-delta adversarial CONVERGED. Claude + Gemini cross-model on the WHOLE Modbus analyzer caught a CRITICAL the per-story reviews missed: timestamp units mismatch (process_pdu treated on_data timestamp as microseconds; pipeline delivers seconds per BC-2.09.007) — wrong finding timestamps + non-functional rate-detection windows. Both models independently rated CRITICAL. Fixed: code seconds-based windows + DateTime::from_timestamp(ts,0) + e2e dispatcher test; SS-14 BCs reconciled to seconds (BC-2.14.016 v2.1, BC-2.14.017 v2.2, BC-2.14.019 v1.2, BC-2.14.013 v2.2); f2-fix-directives §11.5/§11.5b F5-correction banners; spec-changelog [1.5]. Also fixed: is_non_modbus latch on length-invalid ADU (F-DELTA-003), source_ip from Direction not non-existent flow_key.client_ip() (F-DELTA-005, BCs reconciled), BC-2.14.021 post.3 struct mismatch + dead total_flows_analyzed counter (F-DELTA-002), flush granularity on on_close (F-DELTA-004). 78 test timestamps legitimately corrected micros->seconds, not weakened. Sub-second rate precision deferred (needs timestamp_usecs threaded through on_data). F5 fix delivered via PR fix/f5-modbus-timestamp-units. Review artifact: phase-f5-adversarial/modbus-delta-review.md. NEXT: F6 targeted hardening (VP-022/VP-004 Kani — kani IS installed; fuzz; mutation; cargo audit/deny; DF-TEST-NAMESPACE-001 mod-wrappers; e2e port-502 pcap fixture). | 2026-06-09 | Feature #7 F5 combined-delta adversarial CONVERGED — timestamp units micros->seconds (Claude+Gemini CRITICAL) + BC reconciliation |
| D-044 | Feature #7 v0.4.0 Modbus F6 PASS + F7 CONVERGED. F6: Kani 5/5 SUCCESSFUL (VP-022 LOCKED @68a3306 + VP-004 precedence proven after port-502 Rule-5; cargo kani ran for real — cargo-kani 0.67.0, CBMC 140+ SAT checks), fuzz_modbus_parse 3.7M execs/0 crashes, mutation 100% effective kill (163 viable; verifier caught a parallel-run false-kill + manually verified 5 genuine gaps -> 3 killing tests, PR #216), audit/deny clean. F7: 5-dim convergence all PASS — holdout 0.967 (timestamp-year correct end-to-end confirming F5 units fix; regression intact), e2e port-502 pcap->finding acceptance test added (PR #217), DF-TEST-NAMESPACE-001 mod-wrappers added. Fresh consistency audit found 5 spec-doc propagation-shadows (VP-022 index lock not propagated; BC-2.14.014/015 client_ip->direction-resolved; BC-INDEX 0x17 title; f2-directives micros residue; BC-2.14.017 burst-summary ms-vs-s) — ALL FIXED (code was correct throughout). Input-hash scan: MATCH=57/STALE=0/ERROR=1 (STORY-100..105 rewritten post BC-version-bumps). 1338 tests green. NEXT: v0.4.0 human gate -> release. | 2026-06-09 | Feature #7 v0.4.0 Modbus F6 PASS + F7 CONVERGED — 5-dim all PASS + consistency sweep (D-044) |
| D-045 | Feature #7 v0.4.0 human gate APPROVED (human authorized release, conditioned on fixing the 2 holdout blemishes first). Blemish-1 FIXED: exception-burst recon anomaly (Illegal Function 0x01 = FC scanning; Illegal Data Address 0x02 = register-map enumeration) now emits T0888 Remote System Information Discovery (consistent with the recon-FC mapping for FC 0x11/0x2B in BC-2.14.020 Decision 12); Clear-Counters 0x000A + other exception codes stay untagged. BC-2.14.019 v1.3; spec-changelog [1.6] updated; STORY-104 input-hash recomputed bc3863e; delivered via PR. Blemish-2 (port-502 service label in summary, src/decoder.rs:112) ASSESSED CORRECT-BY-DESIGN — standard IANA port-service hint (parallel to 443->HTTPS, port-name independent of analyzer), NOT a defect, no change. Both blemishes dispositioned. NEXT: v0.4.0 gitflow release. | 2026-06-09 | Feature #7 v0.4.0 human gate APPROVED + blemishes dispositioned |
| D-046 | Published wirerust v0.4.0 (gitflow-proper) — Modbus TCP analyzer (Feature #7, issue #7 CLOSED). release/0.4.0 -> PR #219 -> main merge 90aa91e; annotated tag v0.4.0; release.yml run 27254720396 built+attached 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); GitHub Release live (published 2026-06-10T05:12:40Z). CHANGELOG [0.4.0]: Modbus TCP analyzer — port-502 detection, MBAP/FC parse, transaction correlation, 7 MITRE ATT&CK-for-ICS technique detectors (T0855/T0836/T0835/T0831/T0806/T0814/T0888), multi-tag co-emission, dual-window rate detection, exception-burst anomaly, CLI flags; ADR-005/006; VP-022 Kani-verified, fuzz 3.7M/0 crashes, mutation 100% effective. Full F1-F7 feature-mode cycle: F5 combined-delta caught the CRITICAL timestamp-units defect (both Claude+Gemini), F6 ran Kani/fuzz/mutation for real, F7 5-dim CONVERGED (holdout 0.967). main back-merged to develop (8e38041), branches in sync. Feature #7 COMPLETE across 2 releases: v0.3.0 (multi-tag schema migration) + v0.4.0 (Modbus analyzer). | 2026-06-10 | v0.4.0 release — Modbus TCP analyzer (Feature #7 COMPLETE, issue #7 CLOSED) |

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
| RUSTSEC-2026-0097 | rand 0.8.5 unsound (transitive via tls-parser→phf 0.11); upstream-only fix path | ACCEPTED-TRANSITIVE — revisit when tls-parser bumps phf→0.12+ |
| FE-001 | pcapng input format not supported (.pcap-only) — v2 idea; see tech-debt-register.md | deferred / v2 / not-filed |
| ACTION-PIN-001 | dtolnay/rust-toolchain @stable and @nightly remain branch-ref — intentionally exempt in the Action pin gate (toolchain installer, channel-selected). | OPEN P3 — low priority |
| PCAP-CORPUS-001 | E2E pcap test-corpus storage backend (R2/B2 vs Drive-SA) — design ready, orphan-branch `test-pcaps` control plane (MANIFEST.yaml + fetch.sh + tiered runner); 100s of GB expected. 4SICS ICS-lab captures validated v0.4.0 (1.55M pps, 0 crashes). PRECURSOR LANDED (PR #221 fb2c875): lightweight index/fetch layer committed (E2E-PCAPS.md + bin/fetch-e2e-pcaps + mk_modbus_large_pcap.py); large pcaps gitignored under tests/fixtures/local-samples/. Only the shared-corpus STORAGE BACKEND choice (Cloudflare R2 / Backblaze B2 / Google Drive service account) remains tabled. | TABLED — human decision pending (2026-06-10) |

## Cycle-Close Follow-Up Items

CLOSED items (PROCESS-GAP-P5-001, PG-1–PG-4, CC-005, CC-006) archived to `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.

| ID | Description | Status |
|----|-------------|--------|
| PG-5 | DF-SIBLING-SWEEP intra-SS propagation-shadow — 3rd recurrence this cycle. Codify DF-SIBLING-SWEEP-001 v5 (grep-sweep gate after FC-set/title/enum change across intra-SS sibling BCs + VP files + BC-INDEX). | OPEN — codification pending |
| PG-6 | Gemini hybrid caught arithmetic-precision class (truncation-bias, off-by-six ADU, length-gate off-by-one) that 3 Claude rounds missed. Codify PROCESS-ARITHMETIC-REVIEW-001 (dedicated numeric review slice for binary-protocol/threshold features). | OPEN — codification pending |
| CC-001 | DF-SIBLING-SWEEP extension — extend to test-file comments + canonical-vector arithmetic lint (3 recurrences this cycle). | DRAFT — deferred to policy-codification pass |
| CC-002 | VP-lock propagation checklist — must propagate to VP-INDEX, coverage-matrix (tool-column), architecture VP anchors, BC VP-anchor prose, AND recompute consuming-story input-hashes. | DRAFT — deferred to policy-codification pass |
| CC-003 | PROCESS-ARITHMETIC-REVIEW-001 codification — Gemini cross-model slice for detection-math-heavy features (caught truncation/wrap/off-by-one F2; caught seconds-vs-micros CRITICAL F5). | DRAFT — deferred to policy-codification pass |
| CC-004 | F5 dispatcher-boundary test gap — any on_data(timestamp:u32) analyzer needs >=1 test through real dispatcher/reassembler boundary with timestamp_secs-shaped values. | DRAFT — deferred to policy-codification pass |

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
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 1/2 adversary `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`; phase 6 hardening `cycles/v0.1.0-greenfield-spec/hardening/`.
- Issues: #104/#102 CLOSED (PRs #194/#195), #100 RELEASED v0.2.0, #101 OPEN-DEBT, #103 DEFERRED. Dependabot PR #193 CLOSED (SHA-pin). 7/8 actions SHA-pinned; pin gate enforced (PR #196).
