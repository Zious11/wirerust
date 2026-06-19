---
pipeline: STEADY_STATE
phase: IDLE
phase_status: "AT REST — NO phase in progress. E-18 grouped-collapse cycle CLOSED (D-133). v0.9.0/v0.9.1/v0.9.2 all RELEASED 2026-06-19. Next action is HUMAN-DIRECTED."
product: wirerust
mode: brownfield
timestamp: 2026-06-19T00:00:10Z

# Release chain
released_version: v0.9.2
released_at: "2026-06-19"
release_tag: v0.9.2
release_tag_object: a298dbe
release_commit: b73b242
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.9.2
release_yml_run: "27852584971 SUCCESS — 4 binaries published"
prior_released_version: v0.9.1
prior_released_at: "2026-06-19"
prior_release_tag: v0.9.1
prior_release_commit: ad4eec8
v090_release_tag: v0.9.0
v090_release_commit: 986e148

# Ground-truth HEADs (verified 2026-06-19 post-v0.9.2 release)
develop_head: b73b242
main_head: b73b242
factory_artifacts_head: "run: git -C .factory log -1 --format='%h %s'"

# Pipeline completion
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

# Story tracking
stories_delivered: 71
current_cycle: feature-story-119-grouped-collapse  # CLOSED
current_wave: 50  # FINAL — CLOSED (STORY-INDEX.md: 75 stories / 50 waves)

# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []

# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-06-17
maintenance_completed_at: "2026-06-17"
maintenance_findings_count: 48
maintenance_blocking: false

# Convergence (archive pointer)
adversary_convergence_counter: SATISFIED
convergence_trajectory: "Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
---

# VSDD Pipeline State — wirerust

## Session Resume Checkpoint (2026-06-19 — AT REST / STEADY-STATE)

**Previous checkpoint (v0.9.2 RELEASED; BUG-DNP3 closed) archived to:
`.factory/cycles/feature-story-119-grouped-collapse/session-checkpoints.md`**

### PIPELINE STATUS: AT REST

The factory is **idle / steady-state**. NO phase is in progress. NO in-flight story worktrees exist. NO open PRs. The E-18 grouped-collapse cycle is CLOSED. Three releases shipped 2026-06-19: v0.9.0, v0.9.1, v0.9.2.

**On resume, the pipeline is HUMAN-DIRECTED. Present the open items (Section D) and await direction.**

### A. EXACT POSITION

- **Status:** IDLE — AT REST. All feature cycles closed. No active worktrees beyond main + .factory.
- **Latest release:** `v0.9.2` (tag object `a298dbe`) on main commit `b73b242`. 4 binaries published. GitHub Release isDraft=false. `release.yml` run 27852584971 SUCCESS.
- **Release chain (this session):** v0.9.0 (main 986e148) → v0.9.1 (main ad4eec8) → v0.9.2 (main b73b242). Prior chain intact: v0.8.0/v0.7.1/v0.7.0/v0.6.0/v0.5.0/v0.4.0/v0.3.0/v0.2.0/v0.1.0.
- **Cycle closed:** E-18 / STORY-119 grouped-collapse. STORY-120 (PRs #266/#267), STORY-122 (PR #268), STORY-119/B (PR #269). F1-F7 ALL COMPLETE (D-133).
- **BUG resolved:** BUG-DNP3-CONTROL-OP-DETERMINISM-001 CLOSED in v0.9.2 (PR #279, commit `dd99f58`; FlowKey derives Ord + sort-before-enumerate; 5 identical md5 runs verified; 3 regression tests in `tests/dnp3_determinism_tests.rs`).

### B. GROUND-TRUTH SHAs / WORKTREE STATE

| Branch | HEAD | Notes |
|--------|------|-------|
| develop | `b73b242` | FF'd to main post-v0.9.2; zero divergence |
| main | `b73b242` | release/0.9.2 PR #280 merged |
| origin/develop | `b73b242` | synced |
| origin/main | `b73b242` | synced |
| factory-artifacts | `git -C .factory log -1` | this commit |

- **Tag v0.9.2:** tag object `a298dbe` on commit `b73b242`. Latest tag.
- **Active worktrees:** EXACTLY TWO — main repo (develop) and `.factory/` (factory-artifacts). No `.worktrees/*` story/release worktrees.
- **Open PRs:** None.

### C. WHAT IS COMPLETE — DO NOT REDO

- E-18 full cycle F1-F7: COMPLETE and RELEASED (D-133). PRs #266-#270/#273/#274/#275.
- v0.9.1 patch (D-134): PRs #277/#278; tag v0.9.1 on main ad4eec8.
- v0.9.2 patch (D-135): PRs #279/#280; tag v0.9.2 on main b73b242; DNP3 determinism fixed.
- Maintenance maint-2026-06-17: COMPLETE (PRs #261/#262; 5 items deferred; 0 blocking).
- F7 convergence rounds 1-5: R5 triple CLEAN 3/3 (develop 1c89b52). develop sync: FF to main b73b242.

### D. ON RESUME — OPEN ITEMS (present these to human)

**1. DNS-TUNNELING-COVERAGE-001 (OPEN — HUMAN DECISION REQUIRED)**
DNS analyzer is statistics-only by design (BC-2.08.004, `src/analyzer/dns.rs` returns empty findings). Tunneling detection (qname entropy/length, label cardinality, record-type skew, query-rate/NXDOMAIN, up/down byte ratio) = NEW FEATURE. Fixtures ready: `tests/fixtures/E2E-PCAPS.md` has `dns-tunnel-iodine.pcap` + dnscat2 links (bundled v0.9.2). Human decision on whether to scope via F1-F7 is PENDING.

**2. STORY-121 (OPEN DRAFT — HUMAN DECISION REQUIRED)**
E-11 self-improvement draft holds process-gap follow-ups: D-127 orchestrator relay-trust handoff; post-fixburst consuming-artifact sweep; BC canonical-test-vector verbatim citation (PG-62-PERSTORY-TESTS-TO-BUG); post-merge anchor revalidation (PG-62-F5-POSTMERGE-ANCHOR-001). Human decision: widen scope or split.

**3. pcapng reader support (CANDIDATE FEATURE — HUMAN DECISION REQUIRED)**
Large TLS-heavy captures and `arp-baseline-16pkt.cap` rejected (pcapng format). Reference: `tests/fixtures/E2E-PCAPS.md` "Coverage gaps" + `.factory/research/e2e-pcap-candidates.md`. Candidate for next feature cycle.

**4. Roadmap candidates (post-v0.9.2):**
Issue #3 C2 beaconing | Issue #4 CSV+SQLite reporters | Issue #6 rayon parallel (O-07) | PCAP-CORPUS-001 storage backend (TABLED).

**5. Pre-existing deferred (visibility only):**
SEC-001 (MITRE IDs not escaped — LOW, non-exploitable; DF-VALIDATION-001 before issue); W7.1 cargo-public-api; input-hash CI gate; 5 TD-MAINT-* items; TD-E18-SEMVER-CHECKS-001; F7 LOW residuals (ADR banner, no-compile_fail doctest ACCEPTED).

---

## Status

**wirerust v0.9.2 RELEASED 2026-06-19 (D-135). AT REST — IDLE. No phase in progress.**

Latest: v0.9.2 (tag obj `a298dbe`, main `b73b242`, 4 binaries). develop = main = `b73b242`. Zero divergence.
Maintenance maint-2026-06-17: COMPLETE. NON-BLOCKING. Report: `.factory/maintenance/sweep-report-2026-06-17.md`.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog | PASSED | 30/30 lessons; PRs #69-#99 |
| Phase 1 — Spec Crystallization | PASSED 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs |
| Phase 2 — Story Decomposition | PASSED 2026-05-21 | 49 stories / 11 epics / 27 waves |
| Phase 3 — TDD Implementation | PASSED 2026-05-31 | 48/48 stories, 27/27 waves |
| Phase 4 — Holdout Evaluation | PASSED 2026-06-01 | mean 0.949; detail: cycles/v0.1.0-greenfield-spec/ |
| Phase 5 — Adversarial Refinement | PASSED 2026-06-01 | Adversary gate 3/3 SATISFIED |
| Phase 6 — Formal Hardening | PASSED 2026-06-02 | 8 Kani VPs; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0 | RELEASED 2026-06-12 | SS-15 24 BCs; F7 5-dim; tag v0.6.0. Detail: cycles/feature-8-dnp3-v0.5.0/ |
| Feature ARP (E-16) + v0.7.0 | RELEASED 2026-06-16 | STORY-111..115; VP-024 LOCKED. Detail: cycles/feature-arp-v0.7.0/ |
| E-17 ARP QinQ/MACsec + v0.7.1 | RELEASED 2026-06-17 | STORY-116/117; tag v0.7.1 b98a72f |
| Maintenance maint-2026-06-17 | COMPLETE 2026-06-17 | 2 PRs (#261/#262); 5 deferred; 0 blocking |
| E-18 finding-collapse (STORY-118) + v0.8.0 | RELEASED 2026-06-17 | STORY-118; SS-11=29 BCs. Detail: cycles/feature-collapse-v0.8.0/ |
| E-18/E-8 STORY-119 cycle (F1-F7) + v0.9.0 | **RELEASED + CLOSED 2026-06-19** | STORY-120/122/119; 293 BCs; tag v0.9.0 986e148. Detail: cycles/feature-story-119-grouped-collapse/ |
| v0.9.1 patch | **RELEASED 2026-06-19** | Doc/help; PRs #277/#278; tag v0.9.1 ad4eec8 |
| v0.9.2 patch | **RELEASED 2026-06-19** | DNP3 determinism + E2E fixtures; PRs #279/#280; tag v0.9.2 b73b242 |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`

| ID | Decision | Date |
|----|----------|------|
| D-133 | v0.9.0 RELEASED; E-18/STORY-119 cycle CLOSED. Cycle-closing checklist COMPLETE (S-7.02). DNS-TUNNELING-COVERAGE-001 filed. | 2026-06-19 |
| D-134 | v0.9.1 RELEASED — doc/help patch (--no-collapse help text + README flag names). PRs #277/#278; tag v0.9.1 ad4eec8. | 2026-06-19 |
| D-135 | v0.9.2 RELEASED — DNP3 determinism FIXED (FlowKey Ord + sort; PR #279 commit dd99f58). BUG-DNP3-CONTROL-OP-DETERMINISM-001 CLOSED. PRs #279/#280; tag v0.9.2 obj a298dbe. | 2026-06-19 |

## Blocking Issues

None open.

## Drift Items / Tech Debt

All items require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Full tech-debt register: `.factory/tech-debt-register.md`.

| ID | Summary | Status |
|----|---------|--------|
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer statistics-only (BC-2.08.004); tunneling detection = new feature; fixtures in E2E-PCAPS.md. | OPEN — human decision |
| STORY-121 | E-11 process-gap follow-ups (D-127 relay-trust; post-fixburst sweep; BC canonical vectors; anchor revalidation). | OPEN DRAFT — scope decision |
| FE-001 (pcapng) | pcapng format unsupported; large TLS + arp-baseline-16pkt.cap blocked. | CANDIDATE FEATURE |
| SEC-001 | MITRE IDs not escaped — LOW, non-exploitable (embedded lookup). DF-VALIDATION-001 required before issue. | DEFERRED LOW |
| W7.1 | `cargo public-api` baseline not established. | DEFERRED |
| INPUT-HASH-CI-GATE | Input-hash drift check not in develop CI (factory-artifacts not in develop tree). | DEFERRED |
| PCAP-CORPUS-001 | E2E pcap corpus storage backend. | TABLED |
| TD-MAINT-PC001-DNP3-STREAMTRAIT | DNP3 StreamTrait abstraction debt. | DEFERRED LOW |
| TD-MAINT-PC006-MODBUS-NAME-CASING | Modbus name casing. | DEFERRED LOW |
| TD-MAINT-PC003-DNP3-DROPPED-COUNTER | DNP3 dropped packet counter. | DEFERRED LOW |
| TD-MAINT-PERF-ARP-HOTPATH | ARP hot-path performance. | DEFERRED LOW |
| TD-MAINT-RISK-REGISTRY-BACKFILL | Risk registry backfill. | DEFERRED LOW |
| TD-E18-SEMVER-CHECKS-001 | Semver checks for E-18 public API change. | DEFERRED |
| F7-LOW-ADR-BANNER | ADR migration-map banner (DRIFT-ADR0007-D2-PROSE-001). | DEFERRED LOW |
| F7-LOW-NOCOMPILE-DOCTEST | No `compile_fail` doctest for `#[non_exhaustive] FindingsRender`. | ACCEPTED |
| O-07 | rayon in Cargo.toml but unused. | OPEN P2 |
| FU-JSON-CASING | Align serde enum casing (issue #255 filed). | POST-RELEASE |
| DRIFT-DEVELOP-SYNC-BPBYPASS-001 | main→develop sync bypassed branch-protection; informational. | INFORMATIONAL |
| PG-F7-R4-POST-FIXBURST-SIBLING-SWEEP-001 | Post-fix bursts must sweep consuming BCs + story post-delivery notes + VP docs. | CODIFIED |
| BUG-DNP3-CONTROL-OP-DETERMINISM-001 | RESOLVED in v0.9.2 (D-135). FlowKey Ord + sort; 3 regression tests. | RESOLVED |

*(Engine-notes and additional low-severity drift items: cycles/feature-story-119-grouped-collapse/ and cycles/feature-arp-v0.7.0/.)*

## Deferred Next-Work Backlog

1. **PCAP-CORPUS-001:** TABLED — human decision.
2. **DNS-TUNNELING-COVERAGE-001:** OPEN — human decision on feature scope. Fixtures ready.
3. **STORY-121 (E-11 process-gap):** OPEN DRAFT — human decision on scope.
4. **pcapng reader support:** CANDIDATE — human decision.
5. **Roadmap:** Issue #3 C2 beaconing | Issue #4 CSV+SQLite | Issue #6 rayon.

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
| DF-GREEN-DOC-TENSE-SWEEP (v1) | HIGH |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`.
- Issues: #104/#102 CLOSED; all actions SHA-pinned; pin gate enforced; dtolnay/rust-toolchain @stable/@nightly exempted.
- sprint-state.yaml: vestigial greenfield artifact. STORY-INDEX.md is authoritative (75 stories / 50 waves). STORY-119/120/122 status=done confirmed.
