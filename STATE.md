---
pipeline: V0.4.0_RELEASED
phase: maintenance-fix-mitre-v19
active_feature: "#8-dnp3"
feature_8_status: "F1-APPROVED-PAUSED (resumes after MITRE-v19 remap release)"
product: wirerust
mode: brownfield
timestamp: 2026-06-10T12:00:00Z
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
develop_head: 33de854
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
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
consistency_audit: CONSISTENT
input_drift_check: "CLEAN — MATCH=57/STALE=0/ERROR=1 (STORY-091 no-inputs pre-existing; re-verified post Burst-4 spec commit c4765e6)"
---

# VSDD Pipeline State — wirerust

## Status

**MITRE v19 remap fix MERGED to develop (PR #223, develop HEAD 33de854); issue #222 CLOSED. Release to main PENDING (human-gated, version TBD v0.4.1/v0.5.0).** 3-pass adversarial CONVERGED. Final pr-review finding (7 stale `t0855` test-fn identifiers in modbus_detection_tests.rs) resolved before merge — renamed to `t1692_001` (commit 14a52c6), completing the symbol-level sweep. Behavioral change shipped to develop: Modbus findings now emit T1692.001 (JSON/CSV/terminal) instead of revoked T0855; envelope conforms to pinned ics-attack-19.1. **Feature #8 (DNP3) is PAUSED at F1-APPROVED** — resumes after fix ships.

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

## Session Resume Checkpoint (2026-06-10 — MITRE v19 remap MERGED to develop — release pending)

**POSITION:** MITRE v19 remap fix (issue #222) MERGED to develop via PR #223 (merge commit 33de854); issue #222 CLOSED. develop HEAD `33de854`; main HEAD `90aa91e` (v0.4.0). NEXT: human-gated gitflow release to main (version disposition v0.4.1 patch vs v0.5.0 minor — output format changed). Feature #8 (DNP3) PAUSED at F1-APPROVED (D-047/D-048) — resumes after fix ships.

**RELEASE HISTORY:** v0.1.0 (2026-06-08) greenfield; v0.2.0 (2026-06-09) timestamp threading; v0.3.0 (2026-06-09) multi-tag MITRE schema; v0.4.0 (2026-06-10) Modbus TCP analyzer.

**RESUME PROTOCOL FOR NEXT SESSION:**
1. Run `vsdd-factory:factory-worktree-health` — verify .factory/ worktree on factory-artifacts BEFORE any factory reads/writes
2. Read STATE.md (this file) — orient to current state
3. Execute gitflow release: cut release/v0.4.1 (or v0.5.0) from develop, PR → main, tag, back-merge to develop.

**CARRY-FORWARD / OPEN ITEMS:**
- STORY-091: draft, P1, 5 pts, E-11 — anchor-validation tooling; deferred; CC-001..CC-004 codification deferred
- Drift items: O-07 (rayon unused), O-08 (dns.rs stale doc), F-W25-S088-P6-001
- RUSTSEC-2026-0097: accepted-transitive; ACTION-PIN-001: dtolnay/rust-toolchain exempt from pin gate (OPEN P3)
- PCAP-CORPUS-001 (TABLED): storage-backend decision pending — detail archived in session-checkpoints.md
- Dependabot PRs #202-#207: disposition before next release — see Deferred Next-Work Backlog

**INPUT-HASH DRIFT (verified 2026-06-10 post c4765e6):** MATCH=57 STALE=0 ERROR=1 (STORY-091 pre-existing no-inputs; known).

Prior checkpoint archived: cycles/v0.1.0-greenfield-spec/session-checkpoints.md.

## Decisions Log

D-001..D-046 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.

| ID | Decision | Date | Rationale |
|----|----------|------|-----------|
| D-047 | Feature #8 (DNP3 analyzer, issue #8) F1 delta analysis APPROVED by human (2026-06-10). Intent=feature, type=backend, non-trivial → full F1-F7. Integration: Dnp3Analyzer implements StreamHandler+StreamAnalyzer, wired into StreamDispatcher as DispatchTarget::Dnp3 (port-20000 Rule 6) — mirrors Modbus (D-032), NOT the UDP/ProtocolAnalyzer path; UDP DNP3 deferred to v2. New: src/analyzer/dnp3.rs, subsystem SS-15 'DNP3/ICS', VP-023 (Kani candidate, parse/classify pure core), ADR-007 (binary-ICS TCP integration). Modified (5): dispatcher.rs (HIGH — DispatchTarget::Dnp3 + port-20000 classification + VP-004 oracle), mitre.rs (HIGH/CRITICAL — VP-007 drift guard; T0803 AND T0828 are NEW to catalog, must seed+emit atomically), analyzer/mod.rs, main.rs, cli.rs. DTU_REQUIRED=false (no external service, confirmed). HUMAN SCOPE DECISIONS: (1) integration = TCP-only first (StreamDispatcher); (2) CRC-16/DNP = structure-only, strip-not-validate in v1; (3) MITRE = EXPANDED set T0803(new)+T0828(new)+T0855+T0814+T0836 — human chose to add T0828 Loss of Control beyond the architect's minimal recommendation; both T0803 and T0828 need ATT&CK-ICS v19.1 confirmation (research dispatched); (4) app-layer parse = FIR=1 first-fragment only; (5) CLI = add --dnp3-direct-operate-threshold (mirrors --modbus-write-burst-threshold). Delta-analysis doc: .factory/phase-f1-delta-analysis/dnp3-delta-analysis.md. | 2026-06-10 | Feature #8 F1 gate APPROVED — full F1-F7, TCP-only, expanded MITRE (T0803+T0828 new) |
| D-048 | Two independent research passes (DF-VALIDATION-001 satisfied) confirmed a release-safety defect: the MITRE catalog emits/seeds technique IDs REVOKED in ATT&CK-for-ICS v19.0 while the envelope advertises ics-attack-19.1. Full 21-ID blast-radius audit (.factory/research/mitre-ics-v19-catalog-audit.md): exactly 2 IDs affected — T0855 Unauthorized Command Message → T1692.001 (EMITTED by Modbus in v0.4.0; catalogued v0.3.0+v0.4.0) and T0856 Spoof Reporting Message → T1692.002 (catalogue-only, both releases); both fold into new ICS parent T1692 'Unauthorized Message' (v19 introduced ICS sub-techniques). Other 19 IDs ACTIVE-unchanged. VP-007 structurally cannot catch this (closed-world consistency proof, no external-currency oracle). HUMAN DECISIONS (2026-06-10): (1) FIX-FIRST — run a scoped maintenance fix cycle now (remap T0855→T1692.001, T0856→T1692.002 across mitre.rs + modbus.rs emission sites + tests + affected BCs SS-09/10/11/14 + VP-007 sub-technique-format acceptance + correct stale attack-ics-version-pin.md), ship as its own release (v0.4.1/v0.5.0 TBD), THEN resume DNP3 on the corrected base — mirrors D-035 'isolate the correctness change' precedent. (2) DNP3 (Feature #8) MITRE set corrected to v19.1-accurate IDs: T1692.001 (unauthorized command), T1691.001 (block command, ex-T0803), T0827 Loss of Control (correlated finding, not per-packet; replaces the T0828 misread), T0814, T0836. Issue #222 filed. Feature #8 PAUSED at F1-APPROVED. | 2026-06-10 | MITRE v19 revocation defect — fix-first; DNP3 paused; corrected IDs locked |
| D-049 | MITRE v19 remap fix (issue #222) CONVERGED. Spec delta + code/test remap (T0855→T1692.001 emitted, T0856→T1692.002 catalogue-only) across ~30 spec files + 6 code files + 8 test files. Adversarial convergence: Pass 1 NOT-CONVERGED (incomplete sibling sweep — ADR-005/006 authoritative emission tables, cap-10 counts, domain-debt staged list, stale test fn name; PG-5 propagation-shadow recurrence); Pass 2 adversary CONVERGED but consistency caught story-writer's wrong AC-014 tactic labels (T1692.001/.002→CommandAndControl, T0836→IcsInhibitResponseFunction, T0888→IcsImpairProcessControl) + AC-015 count 6→13; Pass 3 (final) CONSISTENT. Code: 1339 tests green, clippy/fmt clean, Kani VP-007 4/4 SUCCESSFUL, sub-technique format T[0-9]{4}(\.[0-9]{3})? accepted. develop-branch code on fix/mitre-ics-v19-remap (HEAD post-2fbab82). NEXT: code PR → develop, then release. | 2026-06-10 | MITRE v19 remap CONVERGED — 3-pass adversarial (caught 2 propagation shadows + tactic errors) |
| D-050 | MITRE v19 remap fix (issue #222) MERGED to develop via PR #223 (merge commit 33de854; repo allows merge-commits only, not squash). 9/9 CI green on final commit 14a52c6 (test/clippy/fmt/audit/deny/fuzz-build/semantic-PR/action-pin-gate/trust-boundary), security review PASS, AI review APPROVE (0 blocking). Final pr-review finding (7 stale `t0855` test-fn identifiers in modbus_detection_tests.rs) resolved before merge — renamed to `t1692_001` (commit 14a52c6), completing the symbol-level sweep. Behavioral change shipped to develop: Modbus findings now emit T1692.001 (JSON/CSV/terminal) instead of revoked T0855; envelope conforms to pinned ics-attack-19.1. Issue #222 CLOSED. Spec delta on factory-artifacts (01451fe/d1dabf9/c4765e6). NEXT: human-gated gitflow release to main (version disposition v0.4.1 patch vs v0.5.0 minor — changes emitted output). | 2026-06-10 | MITRE v19 remap MERGED to develop (PR #223, 33de854); release pending |

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
| PCAP-CORPUS-001 | E2E pcap test-corpus storage backend (R2/B2/Drive-SA) — design ready; precursor PR #221 (fb2c875) landed (E2E-PCAPS.md + bin/fetch-e2e-pcaps + mk_modbus_large_pcap.py); large pcaps gitignored. Detail in session-checkpoints.md. | TABLED — human storage-backend decision pending |
| MITRE-V19-REMAP-001 | MITRE ATT&CK-ICS v19 revocation defect (issue #222, D-048/D-049/D-050): T0855→T1692.001 and T0856→T1692.002 remapped across mitre.rs + modbus.rs emission sites + tests + BCs SS-09/10/11/14 + VP-007 sub-technique-format acceptance. Spec commit c4765e6 (factory-artifacts). PR #223 MERGED to develop (33de854); issue #222 CLOSED. | MERGED-TO-DEVELOP — release pending |
| DRIFT-F2-COUNT-001 | Stale "15 seeded IDs" count (true=21) in OUT-OF-SCOPE files: BC-2.10.006.md, prd-supplements/nfr-catalog.md, holdout-scenarios/HS-008 + HS-009. Pre-existing from F2/STORY-100 expansion. Requires DF-VALIDATION-001 before filing. | DEFERRED — separate cleanup, validate before filing |
| DRIFT-SUPERPOWERS-001 | docs/superpowers/specs/2026-04-13-mitre-attack-mapping-design.md + plans/2026-04-13-mitre-attack-mapping.md carry stale pre-F2 catalog (T0855/T0856, singular mitre_technique field). Multiply-stale design drafts. Requires DF-VALIDATION-001 before filing. | DEFERRED — reconcile-or-archive decision pending |

## Deferred Next-Work Backlog (recorded 2026-06-10)

**1. Dependabot PR sweep (6 open PRs)** — disposition before next release. Status: DEFERRED.

| PR | Package | Action |
|----|---------|--------|
| #202 | actions/checkout | MUST close + SHA-pin manually per ACTION-PIN-001 (do NOT merge tag ref) |
| #203 | serde_json | standard cargo bump — review + merge |
| #204 | assert_cmd | standard cargo bump — review + merge |
| #205 | etherparse 0.16→0.20 | 4-minor jump — review API changes before merging |
| #206 | rayon | standard cargo bump — review + merge |
| #207 | clap | standard cargo bump — review + merge |

**2. PCAP-CORPUS-001 storage backend** — Cloudflare R2 (RECOMMENDED) / Backblaze B2 / Drive-SA. Status: TABLED — human decision pending.

**3. Roadmap feature options (post-DNP3)** — candidate next features after Feature #8 ships.

| Issue | Description | Note |
|-------|-------------|------|
| #3 | C2 beaconing detection | — |
| #4 | CSV + SQLite reporters | — |
| #6 | rayon parallel processing | relates to drift O-07 rayon-unused |
| #64/#62/#63 | reporter improvements | — |
| #101 | FP/TP characterization | OPEN-DEBT; blocked on labelled corpus |
| #103 | size-symmetry evasion discriminator | DEFERRED; blocked on labelled corpus |
| FE-001 | pcapng support | deferred v2 |

Status: DEFERRED — pick after Feature #8.

## Cycle-Close Follow-Up Items

CLOSED items (PROCESS-GAP-P5-001, PG-1–PG-4, CC-005, CC-006) archived to `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.

| ID | Description | Status |
|----|-------------|--------|
| PG-5 | DF-SIBLING-SWEEP intra-SS propagation-shadow — 3rd recurrence this cycle. Codify DF-SIBLING-SWEEP-001 v5 (grep-sweep gate after FC-set/title/enum change across intra-SS sibling BCs + VP files + BC-INDEX). | OPEN — codification pending |
| PG-7 | [process-gap] DF-SIBLING-SWEEP-001 does not enumerate architecture-decision records (specs/architecture/decisions/ADR-*.md), domain-debt.md, or docs/superpowers/ design drafts as mandatory sweep targets when a technique-ID/enum changes. This let ADR-005/006 + domain-debt retain the revoked ID through the first sweep (PG-5 lineage, recurrence). Codify: extend DF-SIBLING-SWEEP target list to ADRs + domain-debt + canonical per-event vector tables. | OPEN — codification pending |
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
