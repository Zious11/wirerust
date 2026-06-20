---
pipeline: FEATURE
phase: F2
phase_status: "F2 COMPLETE (spec-complete + consistency-verified + completeness-validated) — pcapng reader feature cycle OPEN (feature-pcapng-reader). F1 delta analysis + F2 spec evolution done. F2 completeness validation CLEAN (D-138). F3 story decomposition NEXT."
product: wirerust
mode: brownfield
timestamp: 2026-06-19T03:00:00Z

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
current_cycle: feature-pcapng-reader
current_wave: F2-complete  # F3 decomposition pending; STORY-123..127 expected

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

## Session Resume Checkpoint (2026-06-19 — F2 COMPLETE + COMPLETENESS-VALIDATED / pcapng-reader cycle OPEN)

**Previous checkpoint (F2 COMPLETE / consistency-verified) archived to:
`.factory/cycles/feature-pcapng-reader/session-checkpoints.md`**

### PIPELINE STATUS: FEATURE MODE — F2 COMPLETE (spec-complete + consistency-verified + completeness-validated), F3 NEXT

Active cycle: **feature-pcapng-reader**. F1 (delta analysis) + F2 (spec evolution) COMPLETE. F2 fresh-context consistency audit COMPLETE — 6 findings ALL CLOSED (D-137). F2 research-agent completeness validation COMPLETE — CLEAN for intended corpus; F-06/F-07/F-08/F-11 applied as AC-level deltas; 302 active BCs unchanged; re-audited CLEAN (D-138). F3 (story decomposition) is next. No in-flight story worktrees. No open PRs.

### A. EXACT POSITION

- **Status:** FEATURE mode — pcapng reader cycle open. F2 complete (spec-complete + consistency-verified + completeness-validated). F3 pending.
- **Active cycle:** `feature-pcapng-reader` (cycle manifest: `.factory/cycles/feature-pcapng-reader/cycle-manifest.md`)
- **Feature:** FE-001 — pcapng capture-format reader support. Status: IN PROGRESS (moved from CANDIDATE).
- **ADR created:** ADR-009 rev 2 — Option A selected (pcap-file 2.0.0, +0 transitive deps). Decision 7 added (multi-section reject; rev 2).
- **BCs added:** BC-2.01.009..018 (10 new). BC-2.01.004 RETIRED (superseded by BC-2.01.009). BC-INDEX v1.52 (inline version comments synced).
- **Spec versions:** prd.md v1.31, error-taxonomy v2.4, BC-2.01.010 v1.1, BC-2.01.015 v1.1, BC-2.01.017 v1.1, BC-2.01.018 v1.1, nfr-catalog v2.2, test-vectors v2.2, STORY-001 v1.6, epics.md v1.6, BC-2.12.011 v1.4.
- **Latest release (prior cycle):** `v0.9.2` (tag obj `a298dbe`, main `b73b242`).
- **develop:** `b73b242` (= main; zero divergence).
- **Completeness report:** `.factory/research/pcapng-spec-completeness-validation.md`
- **F2 audit:** `.factory/cycles/feature-pcapng-reader/f2-consistency-audit.md`

### B. GROUND-TRUTH SHAs / WORKTREE STATE

| Branch | HEAD | Notes |
|--------|------|-------|
| develop | `b73b242` | FF'd to main post-v0.9.2; zero divergence |
| main | `b73b242` | release/0.9.2 PR #280 merged |
| factory-artifacts | `git -C .factory log -1` | current burst |
| origin/develop | `b73b242` | synced |

- **Active worktrees:** EXACTLY TWO — main repo (develop) and `.factory/` (factory-artifacts).
- **Open PRs:** None.

### C. WHAT IS COMPLETE — DO NOT REDO

- F1 delta analysis: COMPLETE. `.factory/phase-f1-delta-analysis/pcapng-reader-support-delta-analysis.md`.
- F2 spec evolution: COMPLETE. ADR-009 rev 2, BC-2.01.009..018, BC-INDEX v1.52, prd.md v1.31, E-INP-008..012, error-taxonomy v2.4, epics.md v1.6.
- F2 consistency audit: COMPLETE. 6 findings ALL CLOSED. Report: `.factory/cycles/feature-pcapng-reader/f2-consistency-audit.md` (D-137).
- F2 completeness validation: COMPLETE (research-agent). F-06/F-07/F-08/F-11 AC deltas applied. 302 active BCs unchanged. Report: `.factory/research/pcapng-spec-completeness-validation.md` (D-138).
- All prior cycles: E-18 F1-F7 RELEASED (v0.9.0/v0.9.1/v0.9.2), maint-2026-06-17 COMPLETE.

### D. ON RESUME — F3 ENTRY CHECKLIST

**REQUIRED BEFORE F3 STORY DECOMPOSITION:**
1. Run `bin/compute-input-hash --write --scan` — generate input-hashes for STORY-123..127 and verify existing stories.
2. Revise/retire BC-2.12.011 (directory glob "*.pcapng excluded") when decomposing STORY-127.
3. Update HS-001 + HS-INDEX (cite retired BC-2.01.004) — PO action in F3.
4. Propagate VP assignments for BC-2.01.009..018 (architect/VP-INDEX).

**COMPLETENESS-DERIVED F3 FOLLOW-UPS (from D-138 — MUST reach F3 stories):**
5. F-06: STORY for SHB parsing (STORY-123) must implement multi-section reject + craft a 2-section test fixture; the multi-section reset behavior of pcap-file 2.0.0 is INCONCLUSIVE — verify at implementation.
6. F-05: BC-2.01.014 Kani proof MUST cover the full u8 if_tsresol space (base-2 branch has no corpus coverage) — enforce in the timestamp story (STORY-125).
7. F-07: implementer must provide explicit match arms for every enumerated skip block (NRB/ISB/DSB/SystemdJournal/obsolete-Packet/Unknown) — no todo!()/wildcard that silently drops.

**OTHER OPEN ITEMS (lower priority):**
- DNS-TUNNELING-COVERAGE-001: OPEN — human decision pending.
- STORY-121 (E-11 process-gap): OPEN DRAFT — scope decision pending.
- Roadmap: Issue #3 C2 beaconing | Issue #4 CSV+SQLite | Issue #6 rayon (O-07).

---

## Status

**FEATURE MODE — pcapng reader cycle OPEN (feature-pcapng-reader). F2 COMPLETE. F3 NEXT.**

Latest release: v0.9.2 (tag obj `a298dbe`, main `b73b242`, 4 binaries). develop = main = `b73b242`. Zero divergence.
Active feature: FE-001 pcapng capture-format reader support. ADR-009, 10 new BCs, 1 retired BC.
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
| **Feature pcapng-reader (F1+F2)** | **F2 COMPLETE (spec-complete + consistency-verified + completeness-validated) — F3 NEXT** | FE-001 IN PROGRESS. ADR-009 rev 2, BC-2.01.009..018 (10 new, 1 retired), prd v1.31, epics v1.6 (302 active BCs). F2 audit CLEAN (D-137). F2 completeness CLEAN (D-138): F-06/F-07/F-08/F-11 AC deltas applied; E-INP-012 added; 302 BCs unchanged. Cycle: feature-pcapng-reader |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`

| ID | Decision | Date |
|----|----------|------|
| D-133 | v0.9.0 RELEASED; E-18/STORY-119 cycle CLOSED. Cycle-closing checklist COMPLETE (S-7.02). DNS-TUNNELING-COVERAGE-001 filed. | 2026-06-19 |
| D-134 | v0.9.1 RELEASED — doc/help patch (--no-collapse help text + README flag names). PRs #277/#278; tag v0.9.1 ad4eec8. | 2026-06-19 |
| D-135 | v0.9.2 RELEASED — DNP3 determinism FIXED (FlowKey Ord + sort; PR #279 commit dd99f58). BUG-DNP3-CONTROL-OP-DETERMINISM-001 CLOSED. PRs #279/#280; tag v0.9.2 obj a298dbe. | 2026-06-19 |
| D-136 | F1+F2 COMPLETE for pcapng reader feature (FE-001). Cycle: feature-pcapng-reader. ADR-009 created. 10 new BCs (BC-2.01.009..018); BC-2.01.004 RETIRED/inverted (superseded by BC-2.01.009). BC-2.01.001/002 extended. E-INP-008..011 added. STORY-001→v1.6, epics.md→v1.5 re-anchored. Option A (pcap-file 2.0.0, +0 deps) selected. Scope includes E2E corpus expansion (human-approved). F3 story decomposition is next. | 2026-06-19 |
| D-137 | F2 fresh-context consistency audit COMPLETE: 6 findings (2 HIGH, 2 MED, 2 LOW), ALL fixed and re-audited CLEAN. Latent pre-existing epics.md arithmetic bug corrected: BC-2.11.030–034 (STORY-119 grouped-collapse) were missing from epics coverage table; active BC total corrected 297→302 (matches BC-INDEX v1.52 ground truth). Artifacts updated: ADR-009→Status block (FINDING-001), epics.md→v1.6 (FINDING-002), prd.md→v1.30 §7 RTM (FINDING-003), BC-2.12.011→v1.4 (FINDING-004), BC-INDEX timestamp (FINDING-005), HS-001 lifecycle_status:stale + HS-INDEX (FINDING-006). Audit report: cycles/feature-pcapng-reader/f2-consistency-audit.md. | 2026-06-19 |
| D-138 | F2 completeness validation (research-agent) COMPLETE for intended corpus. Verdict: COMPLETE. Confirmed-OK: DSB-for-TLS skip (safely out of scope), power-of-2 if_tsresol already covered by BC-2.01.011/BC-2.01.014. Gaps applied as AC-level deltas (no new BCs; active count stays 302): F-06 MEDIUM — single-section-only policy, second SHB rejected with new error E-INP-012, AC added to BC-2.01.010 v1.1, ADR-009 Decision 7 rev 2; F-07 — BC-2.01.015 v1.1 enumerates all skip-arms (NRB/ISB/DSB/SystemdJournal/obsolete-Packet/Unknown); F-08 — ADR-009 rev 2 records obsolete Packet Block (0x2) skipped not read; F-11 — BC-2.01.018 v1.1 actionable E-INP-011 message + directory-mode per-file isolation AC (xref BC-2.12.011). E-INP-012 added to error-taxonomy v2.4; E-INP-011 refined; BC-2.01.017 v1.1 trace updated. prd.md v1.31, ADR-009 rev 2. F2 now spec-complete + consistency-verified + completeness-validated. Report: research/pcapng-spec-completeness-validation.md. | 2026-06-19 |

## Blocking Issues

None open.

## Drift Items / Tech Debt

All items require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Full tech-debt register: `.factory/tech-debt-register.md`.

| ID | Summary | Status |
|----|---------|--------|
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer statistics-only (BC-2.08.004); tunneling detection = new feature; fixtures in E2E-PCAPS.md. | OPEN — human decision |
| STORY-121 | E-11 process-gap follow-ups (D-127 relay-trust; post-fixburst sweep; BC canonical vectors; anchor revalidation). | OPEN DRAFT — scope decision |
| FE-001 (pcapng) | pcapng format unsupported; large TLS + arp-baseline-16pkt.cap blocked. | **IN PROGRESS** — feature-pcapng-reader cycle open; F2 complete |
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

1. **pcapng reader support (FE-001):** IN PROGRESS — feature-pcapng-reader cycle, F3 decomposition next.
2. **DNS-TUNNELING-COVERAGE-001:** OPEN — human decision on feature scope. Fixtures ready.
3. **STORY-121 (E-11 process-gap):** OPEN DRAFT — human decision on scope.
4. **PCAP-CORPUS-001:** TABLED — human decision.
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
