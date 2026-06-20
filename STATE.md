---
pipeline: FEATURE
phase: F2
phase_status: "F2 pass-5 remediation applied (D-153); adversary pass-6 pending; clean-pass 0/3; trajectory plateau 23/24/17/13/13."
product: wirerust
mode: brownfield
timestamp: 2026-06-20T06:00:00Z

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

## Session Resume Checkpoint (2026-06-20 — F2 PASS-5 REMEDIATION COMPLETE / PASS-6 PENDING / F3 BLOCKED)

**Previous checkpoint (F2 PASS-5 NOT CLEAN / TRAJECTORY PLATEAU / REMEDIATION ROUND-5 PENDING / F3 BLOCKED) archived to:
`.factory/cycles/feature-pcapng-reader/session-checkpoints.md`**

### PIPELINE STATUS: FEATURE MODE — F2 PASS-5 REMEDIATION COMPLETE (D-153): REMEDIATED — AWAITING PASS-6; TRAJECTORY PLATEAU (23/24/17/13/13); CLEAN-PASS COUNTER 0/3; F3 BLOCKED

Active cycle: **feature-pcapng-reader**. F2 pass-5 remediation complete (D-153) — all 1C/4H/5M/3L findings FIXED pending pass-6 verification. Trajectory plateau: P4:13 / P5:13 — remediation applied; pass-6 will determine if plateau broken. Key remediations: C-1 (E-INP-010→E-INP-008 reclassification for EPB body-decode failures; error-taxonomy v3.3; HS-104 v1.3); H-1 (BC-2.01.018 EC-006 step derivation + EC-008 reclassified E-INP-011→E-INP-001 per Decision 17); H-2 (OPB-distinct notice — opb_skipped field + HS-108 Cases d/e); H-3 (SPB snaplen dropped from formula — Decision 9 amendment; HS-107 v1.4); H-4 (HS-107 VV corrected; stale deferral notes removed); M-1..M-5 fixed. Clean-pass counter 0/3. Adversary pass-6 pending. STORY-128 + STORY-127 scoped for F3. No in-flight story worktrees. No open PRs. **BEHAVIORAL DECISIONS TO SURFACE AT F2 HUMAN GATE: Decision 15 (interleaved-IDB reject → E-INP-013); Decision 16 (per-SHB reset dead-spec deferred); Decision 17 (IDB-parse precedence order); Decision 19 (zero-packet notice gating — amended rev 8: emission from main.rs, canonical format); Decision 20 (uniform block error-code rule); Decision 21 (if_tsoffset out-of-scope).**

### A. EXACT POSITION

- **Status:** FEATURE mode — pcapng reader cycle open. F2 adversarial reconvergence in progress (pass-5 remediation COMPLETE — D-153; pass-6 pending). F3 BLOCKED.
- **Active cycle:** `feature-pcapng-reader` (cycle manifest: `.factory/cycles/feature-pcapng-reader/cycle-manifest.md`)
- **Feature:** FE-001 — pcapng capture-format reader support. Status: IN PROGRESS (moved from CANDIDATE).
- **ADR created:** ADR-009 rev 3 — Option A selected (pcap-file 2.0.0, +0 transitive deps). Decision 7: multi-section reject as SCOPE decision (rev 2); rationale corrected via source research (rev 3 — F-06 SUPERSEDED; pcap-file 2.0.0 resets correctly).
- **BCs added:** BC-2.01.009..018 (10 new). BC-2.01.004 RETIRED (superseded by BC-2.01.009). BC-INDEX v1.52 (inline version comments synced).
- **Spec versions:** prd.md v1.33, error-taxonomy v3.3 (C-1 E-INP-010 item c reclassified → E-INP-008; E-INP-008 row updated; next_free E-INP-014), nfr-catalog v2.3, ADR-009 rev 8 (Decision 9 amend: snaplen dropped from SPB; Decision 19 amend: notice format canonical + emission from main.rs), VP-INDEX v2.7 (total 31), BC-INDEX v1.60, BC-2.01.009 v1.5, BC-2.01.010 v1.9, BC-2.01.011 v1.5, BC-2.01.012 v1.5, BC-2.01.013 v1.5, BC-2.01.014 v1.5, BC-2.01.015 v1.6, BC-2.01.016 v1.4, BC-2.01.017 v1.4, BC-2.01.018 v1.6, BC-2.12.011 v1.5, HS-103 v1.5, HS-104 v1.3, HS-107 v1.4, HS-108 v1.1, HS-INDEX v2.3 (all-namespace=181), verification-architecture v2.3, verification-coverage-matrix v1.17. STORY-001 v1.6, epics.md v1.6, test-vectors v2.2.
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
- F2 spec evolution: COMPLETE. ADR-009 rev 3, BC-2.01.009..018, BC-INDEX v1.52, prd.md v1.31, E-INP-008..012, error-taxonomy v2.6, epics.md v1.6.
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
5. F-06: STORY for SHB parsing (STORY-123) must implement multi-section reject (E-INP-012) + craft a 2-section pcapng test fixture. pcap-file 2.0.0 multi-section reset behavior is RESOLVED (source-verified 2026-06-19 — correctly resets per section); reject retained as a scope decision (D-139). No runtime probe required for the reject path; a 2-section regression fixture remains nice-to-have only if SUPPORT is ever adopted. BC-2.01.010 AC-002 reject assertion ships as-is.
6. F-05: BC-2.01.014 Kani proof MUST cover the full u8 if_tsresol space (base-2 branch has no corpus coverage) — enforce in the timestamp story (STORY-125).
7. F-07: implementer must provide explicit match arms for every enumerated skip block (NRB/ISB/DSB/SystemdJournal/obsolete-Packet/Unknown) — no todo!()/wildcard that silently drops.

**OTHER OPEN ITEMS (lower priority):**
- DNS-TUNNELING-COVERAGE-001: OPEN — human decision pending.
- STORY-121 (E-11 process-gap): OPEN DRAFT — scope decision pending.
- Roadmap: Issue #3 C2 beaconing | Issue #4 CSV+SQLite | Issue #6 rayon (O-07).

---

## Status

**FEATURE MODE — pcapng reader cycle OPEN (feature-pcapng-reader). F2 PASS-5 REMEDIATION COMPLETE (D-153). TRAJECTORY PLATEAU (23/24/17/13/13) — pass-6 pending; trajectory break TBD. Clean-pass counter 0/3. F3 BLOCKED — adversarial reconvergence (3 clean passes) required.**

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
| **Feature pcapng-reader (F1+F2)** | **F2 PASS-5 REMEDIATED (D-153) — AWAITING PASS-6; clean-pass 0/3; TRAJECTORY PLATEAU (23/24/17/13/13); F3 BLOCKED** | FE-001 IN PROGRESS. F2 remediation COMPLETE (D-142). Re-audit ALL FIXED (D-143). Pass-2 remediation COMPLETE (D-144): 4C/8H/6M/6L; ADR-009 rev 5. Pass-2 cross-seam re-audit CLEAN (D-145). Pass-3 NOT CLEAN (D-146): 1C/5H/7M/4L. Pass-3 remediation COMPLETE (D-147). Pass-3 cross-seam re-audit gap fixes COMPLETE (D-148): 4 gaps (1 Major/2 Minor/1 Obs). Pass-4 NOT CLEAN (D-149): 1C/4H/5M/3L HIGH novelty. Pass-4 remediation COMPLETE (D-150): EPB padding-aware bound (C-1); Decision 20 uniform error-code rule + SHB E-INP-008 body-too-short re-added correcting pass-3 over-narrowing (H-1); peek-only probe (H-2); VP-030 restated — whitelisted DataLink (H-3); HS-108 zero-packet notice (H-4); ADR-009 rev 7 Decisions 19/20/21. Pass-4 re-audit boundary fixes COMPLETE (D-151): 3 Major gaps — FINDING-P4-001 (BC-2.01.011 v1.5 stale PC5 tail); FINDING-P4-002/003 (error-taxonomy v3.2 stale SHB/IDB-only note + E-INP-010 items d/e mis-classified). Pass-5 NOT CLEAN (D-152): 1C/4H/5M/3L HIGH novelty — PLATEAU (P4:13/P5:13). Pass-5 remediation COMPLETE (D-153): C-1 E-INP-010 item c reclassified → E-INP-008 (error-taxonomy v3.3; HS-104 v1.3); H-1 BC-2.01.018 EC-008 reclassified E-INP-011→E-INP-001 (Decision 17); H-2 OPB-distinct notice (opb_skipped field; HS-108 v1.1); H-3 SPB snaplen dropped (Decision 9 amend; HS-107 v1.4); H-4 HS-107 VV corrected + stale notes removed; M-1..M-5 FIXED; ADR-009 rev 8; BC-INDEX v1.60. Pass-6 pending. Cycle: feature-pcapng-reader |

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
| D-139 | Multi-section pcapng question RESOLVED via source-level research (pcap-file 2.0.0 resets interface table per section correctly — `self.interfaces.clear()` on every `Block::SectionHeader`; confirmed from source, 2026-06-19). The feared mis-attribution class of bug (F-06 MEDIUM) was a Wireshark-era defect, not a pcap-file defect. REJECT decision retained as a SCOPE decision — multi-section is rare and absent from the intended corpus; support is a cheap future-cycle escape hatch (~10-60 LOC). ADR-009 → rev 3 (Decision 7 rationale corrected; F-06 SUPERSEDED; mergecap hint canonical: `mergecap -w out.pcapng <file>`). BC-2.01.010 → v1.3 (AC-002 reframed as scope decision; library-reset acknowledgement added; mergecap hint canonical: `mergecap -w out.pcapng <file>`). error-taxonomy → v2.6 (E-INP-012 Notes corrected; mergecap hint canonical: `mergecap -w out.pcapng <file>`). Research: research/pcapng-multisection-decision.md (RESEARCH-INDEX updated). F2 completeness report annotated: F-06 SUPERSEDED block + scorecard/summary updated to RESOLVED. f2-consistency-audit.md: rationale-correction re-audit added — CLEAN (2 LOW cosmetics RC-1/RC-2 both closed). BC count unchanged (302). F2 fully spec-complete, consistency- and completeness-validated. F3 NEXT. RC-2 final-aligned (ADR-009 mergecap reverted to canonical `-w` form, 2026-06-19). | 2026-06-19 |

| D-140 | Post-D-139 reconciliation: STATE.md spec-version references corrected to on-disk truth (BC-2.01.010 v1.3, error-taxonomy v2.6, ADR-009 rev 3); mergecap remediation hint standardized to `mergecap -w out.pcapng <file>` across all 3 docs (ADR-009 architect-reverted from an out-of-lane state-manager edit). PROCESS-GAP[process-gap]: state-manager performed ADR content edits during the D-139 commit burst — state-manager must not edit spec/ADR content; only state/index files. Logged for cycle-close lessons (cycles/feature-pcapng-reader/lessons.md). | 2026-06-19 |
| D-141 | F2 deep-validation pass (adversary + security + performance, human-requested): adversary 3C/6H/7M/3L, security 0C/2H/4M/3L, performance 3H/2M/1L. Cross-confirmed timestamp-overflow defect (Kani-breaking; adv H-1 == sec SEC-001/006). Consolidated remediation tracker created (29 unique findings, deduplicated): cycles/feature-pcapng-reader/f2-review-remediation-tracker.md. Adversary pass-1 persisted: cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass1.md. pcap-file API spike dispatched (keystone — unblocks H-1 final form, H-2, H-6, M-2, SEC-002, SEC-008). F2 NOT yet converged — remediation + adversarial reconvergence required (3 clean passes, per DF-ADVERSARY-METHODOLOGY-001) before F3. Three [process-gap] items logged to lessons.md: O-1 (VP placeholders reached WRITTEN with VP-NNN=—), O-2 (error-taxonomy input-hash N/A violates DF-INPUT-HASH-CANONICAL-001), C-1 (per-file-isolation claim inserted without owning implementation story). | 2026-06-19 |
| D-142 | F2 remediation executed: ADR-009 rev 4 (architectural raw-block pivot — resolves H-2 SPB overhead, SEC-008 panic surface, C-2 directory glob, O-4 snaplen); VP-025..030 assigned (resolves C-3/DF-CANONICAL-FRAME-HOLDOUT-001); BC-2.01.010..018 + BC-2.12.011 revised (saturating arithmetic, guard-before-allocate, SEC-005 no-panic ACs, STORY-128 per-file isolation re-attribution, magic-byte glob); error-taxonomy v2.7 (E-INP-009/010 routing resolved, H-3/SEC-003/M-3); nfr-catalog v2.3 (NFR-PERF-005/006/007 added, F-PERF-001/002/003); HS-101..106 authored (tied to VP-025..030); STORY-128 (main.rs per-file isolation loop) + STORY-127 (magic-byte glob + corpus) recorded for F3. BC-INDEX v1.53 (inline version annotations synced). Must-fix items C-1/C-3/H-1/H-3/H-5/M-1/M-3/M-7/SEC-002/SEC-003/SEC-005/F-PERF-001/002/003 ADDRESSED. Items still BLOCKED-ON-SPIKE (H-2 final form, H-6/M-2/partial SEC-002) remain in tracker for pass-2 verification. Adversary reconvergence pending (pass 2 not dispatched). PRD §7 RTM staleness flagged for PO: new error-code routing (E-INP-009/010), VP-025..030 story-anchors, STORY-128 anchor not yet reflected. HS-001 rewrite deferred to F3. | 2026-06-19 |
| D-143 | F2 remediation re-audit: 6 findings + BOM-mapping contradiction chain (BC-2.01.010 self-contradiction, HS-103 BE bytes, ADR-009 BE-magic root cause) — ALL FIXED. H5-1 (BC-2.01.009 PC1 ">=0 packets" v1.1); BOM-1 (BC-2.01.010 AC-001 circular phrasing removed v1.5); BOM consistency sweep (BC-2.01.010 v1.6 — 9 statements on-disk byte-sequence canonical); BOM-3+BOM-2 (HS-103 v1.2 — BE BOM bytes corrected 4D3C2B1A→1A2B3C4D + btl encoding fix); ADR-009 rev 4 minor corrections 1+2 (SPB formula btl-16; BE magic `1A 2B 3C 4D`); PRD-BC2-1 (prd.md v1.33 §2.1 magic-byte detection + §7 RTM sync); IDX-1 (HS-INDEX all-namespace=179). BOM now byte-sequence-canonical across ADR-009/BC-2.01.010/HS-103: BE=on-disk `1A 2B 3C 4D`, LE=on-disk `4D 3C 2B 1A`. BC-INDEX v1.54. Adversary reconvergence (pass 2) NEXT. | 2026-06-19 |
| D-144 | F2 adversary pass-2: 4C/8H/6M/6L, HIGH novelty (new wire-format + partial-fix-regression findings). Remediated: ADR-009 rev 5 (Decision 15 interleaved-IDB → E-INP-013 reject; linktype-whitelist timing at IDB-parse; HS-completeness map); C-1 IDB snaplen offset 4–7 (BC-2.01.010 v1.7); C-2 HS-107 authored (SPB framing/snaplen holdout; HS-INDEX v2.1 greenfield=107/all-namespace=180); C-3 frame-overhead 12 bytes (ADR-009 rev 5 Decision 8 update); C-4 stale codes BC-2.01.017 (E-INP-013 added; error-table now E-INP-008..E-INP-013; v1.3); VP-INDEX v2.4 re-anchor + Kani unwind note (I-1/I-2); zero-packet one-shot notice BC-2.01.011 v1.2 (I-3); E-INP-008/010 boundary BC-2.01.010/012 (I-9); verification-architecture v2.0 + verification-coverage-matrix v1.14 (O-5). error-taxonomy v2.8 (E-INP-013 added; next_free E-INP-014). BC-INDEX v1.55 (9 BCs synced). All pass-2 C/I items marked FIXED pending pass-3 verification. **NEW BEHAVIORAL DECISION (flag for F2 human gate): Decision 15 — interleaved-IDB reject (IDB after first packet block → E-INP-013 fail-closed; full interleaved-IDB support deferred to future live-capture cycle).** Pass-3 next. | 2026-06-19 |
| D-145 | F2 pass-2 remediation consistency-verified CLEAN: cross-seam re-audit CLEAN on all 12 seams (f2-consistency-audit.md v2.0); 1 LOW finding — ADR-009 HS-map HS-107 status showed DRAFT (stale). Fixed by architect: ADR-009 rev 5 HS-map updated — HS-107 now AUTHORED. ADR stays rev 5. Adversary pass-3 pending. | 2026-06-19 |
| D-146 | F2 adversary pass-3: NOT CLEAN. 1C/5H/7M/4L, novelty HIGH (partial-fix-propagation + sibling-layer + dead-spec class). C-1: BC-2.01.013 v1.2 changelog FALSELY claimed PC1 fixed (three-way min); on-disk PC1+AC-002 still use two-way min → out-of-bounds slice panic on malformed SPB. H-1/H-2: E-INP-008 body-truncation fixtures unconstructible for SHB/IDB (crate rejects framing truncation; routes to E-INP-010). H-3: E-INP-001 linktype-whitelist orphaned from error-taxonomy BC-ref and BC-2.01.017. H-4: BC-2.01.013 EC-007/Case-B not updated with three-way min (same root as C-1). H-5: per-SHB interface-table reset is dead spec (Decision 7 rejects 2nd SHB before reset fires). M-1..M-7: path error, VP gap, zero-packet notice breadth, parity over-claim, happy-path postcondition missing, options-TLV parse unspecified, error-code precedence undefined. O-1/O-2: stale HS-104 citation + HS-107 byte lines. O-3: stale forward-reference notes + no forward-reference validator. Process-gap: changelog claims must be disk-verified before any pass is declared complete. Clean-pass counter: 0/3. Remediation round-3 pending OR human strategy decision. Pass-3 record: cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass3.md. | 2026-06-19 |
| D-151 | F2 pass-4 re-audit boundary fixes COMPLETE. 3 Major gaps closed: FINDING-P4-001 — BC-2.01.011 v1.4→v1.5: stale PC5 tail sentence "E-INP-008 covers SHB/IDB ONLY; EPB/SPB body truncation → E-INP-010" removed (contradicted Decision 20 uniform rule); FINDING-P4-002 — error-taxonomy v3.2: stale tail note "E-INP-008 is RESERVED for SHB/IDB body-decode failures ... NOT used for EPB/SPB errors" removed from E-INP-010 Notes; FINDING-P4-003 — error-taxonomy v3.2: items (d) "EPB body truncated (<20 bytes)" and (e) "SPB body truncated (<4 bytes)" removed from E-INP-010 scope (per Decision 20 these are E-INP-008 body-decode cases). E-INP-008 row confirmed intact (EPB body<20 / SPB body<4 correctly listed). Cross-seam re-audit seams 2-12 CLEAN. BC-INDEX v1.59. 302 BCs unchanged. Adversary pass-5 is next. | 2026-06-20 |
| D-153 | F2 pass-5 remediation COMPLETE (D-152 findings). C-1: error-taxonomy v3.2→v3.3 — E-INP-010 item (c) padding-overrun reclassified → E-INP-008 (per Decision 20 uniform rule; crate already framed block; wirerust body-decode rejection); E-INP-008 row updated; HS-104 v1.3 Cases D and E reclassified E-INP-010→E-INP-008; BC-2.01.012 v1.5 EC-010 + AC-002 + AC-006 + VP-027 updated. H-1/Decision 17: BC-2.01.018 v1.6 — EC-006 step derivation added (whitelist preempts conflict); EC-008 reclassified E-INP-011→E-INP-001 (non-whitelisted first IDB → E-INP-001 at whitelist check step 2; second IDB never parsed). H-2: BC-2.01.015 v1.6 — opb_skipped:u32 sub-counter added; PC9 rewritten: counters surfaced via PcapSource; main.rs emits notice post-Ok; HS-108 v1.1 Cases d/e (OPB-only; mixed NRB+OPB). H-3/M-2/Decision 9 amend: BC-2.01.013 v1.5 — snaplen dropped from SPB formula; captured_len = min(original_len, block_body_available); VP-031 updated; HS-107 v1.4 Case B rationale corrected; 4× stale deferral notes removed. H-4: BC-2.01.013 v1.5 HS-107 VV description corrected (truncation/padding/no-IDB boundary, not real-world). M-1: BC-2.01.009 v1.5 — Precondition 3 deleted (contradicts EC-003 graceful-Err trust model). M-3: BC-2.01.014 v1.5 — µs fast-path ts_sec uses .min(u32::MAX as u64) as u32; saturation vector added; VP-025 scope extended. M-4: BC-2.01.009 v1.5 — AC-007 pinning BufReader wrap-site (same BufReader for fill_buf + downstream parsers). M-5/Decision 19 amend: BC-2.01.009 v1.5 + BC-2.01.015 v1.6 — notice emission moved to main.rs; PcapSource.skipped_blocks + opb_skipped fields; Decision 19 canonical format "notice: <filename>: 0 packets read from <pcap|pcapng> file"; classic empty-pcap symmetry; ADR-009 Decision 19 amended (rev 8). ADR-009 rev 8 (Decision 9 amend + Decision 19 amend). VP-INDEX v2.7. verification-architecture.md v2.3. verification-coverage-matrix.md v1.17. BC-INDEX v1.60. All pass-5 findings marked FIXED pending pass-6. PG-2 (STORY-128 verify) + PG-3 (arp fixture params) deferred to F3-entry checklist. Clean-pass counter 0/3. Adversary pass-6 next. | 2026-06-20 |
| D-152 | F2 adversary pass-5 (fresh context; ADR-009 rev 7, BC-2.01.009..018, error-taxonomy v3.2, VP-INDEX v2.6, HS-101..108): NOT CLEAN. 1C/4H/5M/3L, HIGH novelty. TRAJECTORY PLATEAU: P1:23/P2:24/P3:17/P4:13/P5:13 — persistent 1C+4-5H last 3 passes. C-1: E-INP-010 item (c) padding-overrun partial-fix sibling miss — D-151 closed items (d)/(e) but left padding-overrun on E-INP-010; per Decision 20 this is Tier-2 body-decode → E-INP-008. H-1: BC-2.01.018 EC-006/EC-008 contradict Decision 17 precedence — EC-008 claims E-INP-011 for non-whitelisted first IDB; correct per Decision 17 step 2 is E-INP-001 at first-IDB-parse. H-2: OPB-only file → silent data loss (SOUL #4 incomplete); generic zero-packet notice does not flag obsolete-Packet-Block data not ingested. H-3: SPB snaplen-clamping in three-way min contradicts ADR Decision 9 ("neither EPB nor SPB enforces snaplen") and creates EPB/SPB asymmetry. H-4: BC-2.01.013 VV table mis-describes HS-107; 4× stale "deferred to a separate burst" notes (HS-107 Case F now exists). M-1..M-5 / L-1..L-3: BC-2.01.009 precondition trust model inversion; µs fast-path saturation gap; BufReader wrap-site unspecified; zero-packet notice layering/format conflict; count propagation / dual framing / cosmetic. 3 process-gap obs: tracked-deferral idiom missing; STORY-128 unconfirmed; arp-baseline-16pkt.cap params unverified. 2 lessons added to cycles/feature-pcapng-reader/lessons.md (partial-fix sibling miss; tracked-deferral idiom). Full record: cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass5.md. Remediation round-5 pending. Clean-pass counter 0/3. | 2026-06-20 |
| D-150 | F2 pass-4 remediation COMPLETE (D-149 findings). C-1: BC-2.01.012 v1.4 — EPB padding-aware bound added (`EPB_FIXED + captured_len + pad(captured_len) <= body.len()`; unconditional body.len() guard); HS-104 v1.2 Case E (non-mult-of-4 padding-aware boundary → E-INP-010). H-1/Decision 20: ADR-009 rev 7 — uniform body-decode-truncation rule established (3 tiers: crate-framing-fail→E-INP-010; aligned-framed-body-short→E-INP-008; semantic-fail→E-INP-008); SHB body-too-short (btl=16→body=4) re-added as constructible E-INP-008 case correcting pass-3 over-narrowing; HS-103 v1.5 Case D; HS-107 v1.3 Case F; error-taxonomy v3.1 (Decision 20 preamble). H-2: BC-2.01.009 v1.4 — probe consume(4) removed; peek-only via fill_buf specified. H-3: VP-030 restated — domain narrowed to whitelisted DataLink values only; non-whitelisted → E-INP-001 out of VP-030 scope; VP-INDEX v2.6; BC-2.01.018 v1.5. H-4/Decision 19: ADR-009 rev 7 — zero-packet notice gating formalized; HS-108 v1.0 authored (3 cases: IDB-only → Ok+notice; IDB+2-skipped → Ok+notice+skip-count; EPB-before-IDB → Err); BC-2.01.009 v1.4 disambiguation rule; HS-INDEX v2.3 (all-namespace=181). M-1: crate-enforces over-claim removed from BC-2.01.011/012/013 v1.4. M-2/Decision 21: if_tsoffset declared out-of-scope (ADR-009 Decision 21; BC-2.01.011 v1.4; BC-2.01.014 v1.4). M-3: BC-2.01.012 PC8 scoped to EC-009 only; EC-008 moved to HS-104 Case E. M-4/Decision 19: zero-packet notice anchor corrected from Decision 17 to Decision 19 in BC-2.01.009/018. M-5: error-taxonomy v3.1 #seq-convention preamble pinned (all templates count "#N in file, SHB = block 1"). L-1: BC-2.01.016 v1.4 DLT codes source-verified. L-2: BC-2.01.011 v1.4 EC-003 pipe escaped. L-3 (process-gap/deferred): error-taxonomy input-hash N/A — carried as pre-F3 obligation. Decisions added: Decision 19 (zero-packet notice gating), Decision 20 (uniform block error-code rule), Decision 21 (if_tsoffset out-of-scope). BC-INDEX v1.58. All pass-4 findings marked FIXED pending pass-5. **NEW BEHAVIORAL DECISIONS for F2 human gate: Decision 19, Decision 20, Decision 21.** Pass-5 adversary dispatch next. Clean-pass counter 0/3. | 2026-06-20 |
| D-149 | F2 adversary pass-4: NOT CLEAN. 1C/4H/5M/3L, novelty HIGH (EPB/SPB sibling-propagation gap; false-unconstructibility over-correction; VP satisfiability failure; SOUL #4 holdout gap). C-1: BC-2.01.012 EPB captured_len guard missing padding term + unconditional body.len() bound (SPB three-way min fix not propagated to EPB sibling; HS-104 only tests mult-of-4 so cannot catch). H-1: SHB E-INP-008 narrowing (D-147) based on false "btl<12 unconstructible" premise — btl=16 IS constructible (valid framing; short body=4 < 16-byte minimum); need uniform body-decode-truncation rule. H-2: BC-2.01.009 probe consume(4) breaks stream cursor invariant — dispatch requires byte-0 un-consumed; probe must be peek-only via fill_buf. H-3: VP-030 unsatisfiable over arbitrary u16 — non-whitelisted linktypes short-circuit to E-INP-001 before E-INP-011 conflict check (VP cannot cover code under test). H-4: no holdout for zero-packet one-shot notice (SOUL #4); BC-2.01.009 missing disambiguation rule between zero-packet success (Ok+notice) and EPB-before-IDB error (Err). M-1..M-5: crate-enforces over-claim, if_tsoffset extracted but never applied, EC-008 boundary test vector absent, ADR Decision 17 mis-cited for zero-packet notice, block-seq numbering convention inconsistent. L-1/L-2/L-3(process-gap): DLT code verification, unescaped pipe, error-taxonomy input-hash still N/A. Pass-4 record: cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass4.md. Clean-pass counter: 0/3. Remediation round-4 pending. | 2026-06-19 |
| D-148 | F2 pass-3 cross-seam re-audit gap fixes applied (4 gaps: 1 Major/2 Minor/1 Obs, all FIXED). P3-001: error-taxonomy v2.9→v3.0 — E-INP-008 scope note clarified to semantic-only (invalid BOM / major!=1); framing truncation never triggers E-INP-008. P3-002: BC-2.01.018 v1.3→v1.4 — Related-BCs list reordered to canonical BC numeric sequence; no normative change. BC-INDEX v1.56→v1.57 (annotation synced). P3-003: HS-107 v1.1→v1.2 — VP-031 (SPB captured-len proptest) added to verification_properties traceability; was absent after D-147 M-2 assigned it. P3-004: HS-107 v1.2 (same bump) — Case B captured_len restated as explicit three-way min(original_len=200, snaplen=100, block_body_available=100)=100 to match BC-2.01.013 PC1 contract. HS-INDEX v2.1→v2.2 (HS-107 row VP column updated VP-028→VP-028,VP-031). spec-changelog prepended [pcapng-f2-pass3-reaudit-fixes-2026-06-19]. Cross-seam re-audit otherwise CLEAN: 8/12 seams clean, 4 prose-layer gaps now fixed. Pass-4 adversary dispatch is next. | 2026-06-19 |
| D-147 | F2 adversary pass-3 (1C/5H/7M/4L) remediated: ADR-009 rev 6 (Decision 16 per-SHB-reset dead-spec deferred; Decision 17 IDB-parse precedence E-INP-013→001→011; Decision 18 VP-031 SPB captured-len proptest). C-1 SPB three-way min(original_len, snaplen, block_body_available) panic fix propagated to BC-2.01.013 PC1+AC-002+EC-007+Case-B (v1.3). H-1/H-2 E-INP-008 narrowed to semantic-only (invalid BOM / major!=1); SHB/IDB framing truncation routes to E-INP-010; IDB constructible-fixture window 12<=btl<20 stated (BC-2.01.010 v1.8, BC-2.01.011 v1.3). H-3 E-INP-001 wired: BC-2.01.016 added to error-taxonomy E-INP-001 BC-ref; E-INP-001 added to BC-2.01.017 v1.4 error-code table. H-4 EC-007/Case-B three-way min propagated (BC-2.01.013 v1.3). H-5 per-SHB reset dead-spec removed from BC-2.01.011/012/015/018; BC-2.01.018 EC-005 corrected to E-INP-012 reject. M-2 VP-031 added (VP-INDEX v2.5; total 31). M-3 zero-packet notice broadened to "valid file, zero packets" regardless of skip count. M-4 ts_high==0 parity scope (BC-2.01.014 v1.3). M-5 N-packet postcondition + arp-baseline-16pkt.cap anchor (BC-2.01.009 v1.3). M-6 IDB options-TLV walking + malformed-option-length → E-INP-008 spec (BC-2.01.011 v1.3; error-taxonomy v2.9). M-7 IDB-parse precedence defined: E-INP-013 first, E-INP-001 second, E-INP-011 third (ADR-009 Decision 17). O-1 HS-104 v1.1 PC5 re-cite. O-2 HS-107 v1.1 stale hex lines removed. O-3 stale forward-reference notes removed. BC-INDEX v1.56 (all 10 BCs synced). All pass-3 findings marked FIXED pending pass-4. **BEHAVIORAL DECISIONS flagged for F2 human gate: Decision 15 (interleaved-IDB), Decision 16 (dead-spec deferral), Decision 17 (precedence order).** Pass-4 adversary dispatch next. Clean-pass counter: 0/3. | 2026-06-19 |

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
