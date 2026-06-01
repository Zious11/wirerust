---
document_type: burst-log
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-22T00:00:00Z
cycle: v0.1.0-greenfield-spec
traces_to: STATE.md
---

# Burst Log — v0.1.0-greenfield-spec

## Burst 2 (2026-05-22) — Wave 2 In-Progress Checkpoint

**Agents dispatched:** test-writer (STORY-002, STORY-003, STORY-004, STORY-070), implementer (STORY-003 fuzz harness + CI job), spec-writer (story v1.2 bumps, BC v1.3 bumps, VP-008 v1.1), adversary (pass 1 + pass 2 all four stories), state-manager (checkpoint)
**Files touched:** stories/STORY-002.md, stories/STORY-003.md, stories/STORY-004.md, stories/STORY-070.md (v1.2); specs/behavioral-contracts/ss-02/BC-2.02.005.md, BC-2.02.013.md (v1.3); specs/behavioral-contracts/ss-09/BC-2.09.005.md (v1.3); specs/verification-properties/vp-008-decode-packet-no-panic.md (v1.1); cycles/v0.1.0-greenfield-spec/STORY-002/, STORY-003/, STORY-004/, STORY-070/ (cycle dirs + red-gate logs)

### Summary

Wave 2 is in active per-story convergence. All four stories have tests written (uncommitted in worktrees) and two adversarial passes completed. STORY-002 reached pass-2 CLEAN (1/3). STORY-003, STORY-004, and STORY-070 need pass-2 remediation before continuing. Spec artifacts version-bumped and committed to factory-artifacts.

- **STORY-002** (decoder Ethernet/RAW/IPv6, BC-2.02.001..005): 23 tests in tests/bc_2_02_story002_tests.rs. Pass 1 findings remediated; pass 2 CLEAN — streak 1/3.
- **STORY-003** (decoder SLL/no-panic/VP-008, BC-2.02.006..009): 18 tests in tests/bc_2_02_story003_tests.rs. Fuzz harness fuzz/fuzz_targets/fuzz_decode_packet.rs created + committed (c729579); CI fuzz-build job added to .github/workflows/ci.yml (60ee28b). Pass 1: 1C+2M remediated. Pass 2: NOT CLEAN — M-1 fuzz harness exercises only the 5 whitelisted DataLink variants, never an unsupported variant (VP-008 BC-2.02.008 source contract uncovered). Also 3 Minor, 2 Nit. Pass-2 remediation required.
- **STORY-004** (decoder ICMP/Other/port-table, BC-2.02.010..013): 17 tests in tests/bc_2_02_story004_tests.rs. Pass 1 findings remediated. Pass 2: 0C/0M — 3 Minor (comment/scoping) + 1 process-gap OBS-5 (story frontmatter has no per-input BC version pin). Light remediation pending then pass 3.
- **STORY-070** (findings JSON skip_serializing_if, BC-2.09.005..006): tests added to tests/reporter_tests.rs; src comment fix committed (eb83551). Pass 1: 1M remediated. Pass 2: NOT CLEAN — M-1 story Task 6 says "exactly one call site", contradicting BC-2.09.005 v1.3 "all call sites". Also 3 Minor, 1 Nit. Pass-2 remediation required.

**Spec artifact versions bumped this wave:**
- STORY-002, STORY-003, STORY-004, STORY-070 → v1.2
- BC-2.02.005, BC-2.02.013, BC-2.09.005 → v1.3
- VP-008 → v1.1

**Process gaps observed this burst (added to Cycle-Close Follow-Up):**
- W2.1: Presence-assertion tests must be paired with content greps (STORY-003 pass-2)
- W2.2: CI regression-detector jobs lack positive-coverage assertions (STORY-003 pass-2)
- W2.3: Story frontmatter has no per-input BC version pin (STORY-004 OBS-5)

### Details

| Agent | Task | Output |
|-------|------|--------|
| test-writer | STORY-002: write 23 tests covering BC-2.02.001..005 (Ethernet/RAW/IPv6 paths) | tests/bc_2_02_story002_tests.rs (worktree, uncommitted) |
| test-writer | STORY-003: write 18 tests covering BC-2.02.006..009 (SLL/no-panic/VP-008) | tests/bc_2_02_story003_tests.rs (worktree, uncommitted) |
| implementer | STORY-003: create fuzz harness + CI fuzz-build job | fuzz/fuzz_targets/fuzz_decode_packet.rs (c729579), .github/workflows/ci.yml (60ee28b) in worktree |
| test-writer | STORY-004: write 17 tests covering BC-2.02.010..013 (ICMP/Other/port-table) | tests/bc_2_02_story004_tests.rs (worktree, uncommitted) |
| test-writer | STORY-070: add tests for BC-2.09.005..006 (skip_serializing_if) | tests/reporter_tests.rs (worktree, uncommitted); src comment fix eb83551 |
| adversary | STORY-002 pass 1+2 | Pass 2 CLEAN (1/3) |
| adversary | STORY-003 pass 1+2 | Pass 2 NOT CLEAN — M-1 variant coverage gap in fuzz harness |
| adversary | STORY-004 pass 1+2 | Pass 2 NOT CLEAN — 3 Minor + OBS-5 process gap |
| adversary | STORY-070 pass 1+2 | Pass 2 NOT CLEAN — M-1 story Task 6 contradicts BC-2.09.005 v1.3 |
| spec-writer | Bump story versions to v1.2; BC-2.02.005/013, BC-2.09.005 to v1.3; VP-008 to v1.1 | factory-artifacts |
| state-manager | Wave 2 in-progress checkpoint: STATE.md + burst-log + cycle dirs | factory-artifacts (this commit) |

---

## Burst 1 (2026-05-20) — Adversarial Pass 1 Remediation

**Agents dispatched:** spec-writer (architecture), spec-writer (domain), spec-writer (prd), spec-writer (behavioral-contracts), spec-writer (verification-properties)
**Files touched:** 32 files across specs/architecture/, specs/domain/, specs/prd.md, specs/behavioral-contracts/, specs/verification-properties/

### Summary

Addressed all 17 findings from adversarial spec-convergence pass 1 (2C/8H/5M/2L).
Primary work: re-anchored line citations in ~22 BC body files post-refactor; rebuilt
CLI flag table in api-surface.md from source; fixed VP count arithmetic in
verification-architecture.md and verification-coverage-matrix.md; completed INV-2
invariant body; documented ADR 0004; aligned prd.md rayon claim with src/;
rewrote BC-2.05.006 two-phase-commit contract; added tech-debt O-07 (rayon unused).

### Details

| Agent | Task | Output |
|-------|------|--------|
| spec-writer (arch) | Rebuild api-surface.md CLI flag table from source | `specs/architecture/api-surface.md` |
| spec-writer (arch) | Fix VP count arithmetic, update cross-refs | `specs/architecture/verification-architecture.md`, `specs/architecture/verification-coverage-matrix.md` |
| spec-writer (domain) | Fix INV-2 invariant body, file counts, ADR 0004, O-07 debt | `specs/domain/domain-spec.md`, `specs/domain/invariants/inv-01-core-invariants.md`, `specs/domain/domain-debt.md` |
| spec-writer (prd) | Align file count, rayon claim, §2.13 section titles | `specs/prd.md` |
| spec-writer (bcs) | Fix BC-INDEX.md titles/header counts; flip all 212 rows to [WRITTEN] | `specs/behavioral-contracts/BC-INDEX.md` |
| spec-writer (bcs) | Re-anchor line citations in ~22 BC body files; rewrite BC-2.05.006 | `specs/behavioral-contracts/ss-01..ss-13/` (22 files) |
| spec-writer (vps) | Fix VP-INDEX.md stale entries; update vp-005 | `specs/verification-properties/VP-INDEX.md`, `specs/verification-properties/vp-005-sni-four-way-classification.md` |

---

## Burst 3 (2026-06-01) — Phase-5 HS043 Fresh-Context Pass 2 Disposition

**Agents dispatched:** adversary (HS043-pass-2 fresh-context), implementer (throwaway fix branch — discarded), state-manager (STATE.md update + D-009 decisions log)
**Files touched:** `.factory/STATE.md` (drift items added: ADV-HS043-P02-MED-001, ADV-HS043-P02-LOW-001; D-009 added; session checkpoint updated; Phase-5 progress row updated); `.factory/cycles/v0.1.0-greenfield-spec/burst-log.md` (this entry)
**Versions bumped:** none — no spec or source artifacts modified; develop HEAD unchanged (e0451ef)

### Summary

HS043 fresh-context adversarial Pass 2 (HS043-pass-2) completed — result: NOT CLEAN (1 MED, 1 LOW). Both findings dispositioned/accepted by human decision 2026-06-01. No code merged to develop; throwaway fix branch and worktree discarded.

**ADV-HS043-P02-MED-001 (ACCEPTED — GATED ON LIVE-CAPTURE SUPPORT)**

Finding: The idle-flow expiry sweep gate `timestamp > last_expiry_sweep_secs` is a strictly-monotonic watermark. On out-of-order, multi-epoch, or clock-regressing pcap captures, the watermark stalls and idle sweeps stop firing for the rest of the run (`flows_expired` stuck at 0).

Disposition rationale:
1. **Current scope is offline pcap analysis.** Input is finite; `finalize()` reclaims all remaining flows at end-of-capture. No unbounded memory growth is possible. The sweep is a latency optimization, not the only memory backstop.
2. **The probe's premise is flawed.** 20 flows created at t=10 are brand-new (not idle); a correctly-tuned idle timeout (default 300 s) would not trigger on them. The adversary's scenario conflates "new flows at low timestamp" with "idle flows that need expiry."
3. **The proposed high-water-clock fix was empirically disqualified.** Measuring idle against the clock high-watermark would wrongly expire flows in legitimate multi-epoch analysis: e.g., processing a.pcap (epoch=2003) then b.pcap (epoch=0) — the high-water clock from the first file sits far above b.pcap timestamps, causing the idle check `(high_water - pkt_time) > timeout` to be TRUE for ALL b.pcap flows immediately, expiring them before the HTTP analyzer can process them. The story-088 http-ooo test suite (`test_BC_2_12_010_*` family) confirmed this failure empirically on the throwaway branch.
4. **No unbounded growth, no correctness defect in current scope** — the finding is a speculative risk for a not-yet-existent live-capture mode.

Trigger for re-activation as a real defect: **WHEN live-capture support is added.** Live capture never calls `finalize()` (runs indefinitely); a clock regression (e.g., NTP step backward) stalls the monotonic gate and grows memory unbounded. The correct fix is NOT the high-water-clock approach — it must be a file/epoch-boundary flush OR a wall-clock-based sweep tick decoupled from packet timestamps. This must be re-opened and addressed before or within the live-capture feature. No GitHub issue filed per DF-VALIDATION-001 (requires research-agent validation; feature does not yet exist).

**ADV-HS043-P02-LOW-001 (ACCEPTED — non-blocking)**

Finding: BC-2.04.013 PC0 literally names `expire_flows` but the implementation wires `expire_idle_by_timeout`; the split is justified (keeps the Closed-clause off the hot path per BC-2.04.017) and documented in source. Optional one-line BC wording note.

Disposition: Accepted as non-blocking documentation reconciliation. Optional update to BC-2.04.013 PC0 wording to name `expire_idle_by_timeout` at a convenient future spec touch.

**Phase-5 convergence status after this burst:**
- HS043-pass-2: COMPLETE, NOT CLEAN (1 MED + 1 LOW, both ACCEPTED)
- Whole-implementation fresh-context adversarial review: NOT YET STARTED
- Convergence loop: minimum 3 consecutive clean passes required; clock starts at 0
- NEXT ACTION: broad fresh-context whole-implementation adversarial pass (not another HS-043-only pass)

### Details

| Agent | Task | Output |
|-------|------|--------|
| adversary | HS043-pass-2 fresh-context adversarial review of idle-flow expiry surface | 1 MED (ADV-HS043-P02-MED-001) + 1 LOW (ADV-HS043-P02-LOW-001) |
| implementer | Throwaway high-water-clock fix branch | Discarded — broke story-088 http-ooo tests; not merged |
| state-manager | STATE.md: drift items, D-009 decision, checkpoint, Phase-5 row | `.factory/STATE.md` updated |
| state-manager | Burst-log entry for HS043-pass-2 disposition | `.factory/cycles/v0.1.0-greenfield-spec/burst-log.md` (this entry) |

---

## Burst 4 (2026-05-22) — Wave 3 STORY-071 Merge + STORY-005 Per-Story Convergence

**Agents dispatched:** test-writer (STORY-071), adversary (STORY-071 passes 1/2/3; STORY-005 passes 1–8), state-manager (sprint-state + STATE.md + burst-log)
**Files touched:** stories/sprint-state.yaml (STORY-071 status→done, pr=113, merge_commit=991e821); STATE.md (develop HEAD, Status, Phase Progress, Current Phase Steps, Wave 3 delivery summary, session checkpoint, W3.1 process-gap); cycles/v0.1.0-greenfield-spec/burst-log.md (this entry)

### Summary

STORY-071 squash-merged to develop as PR #113 (991e821, 2026-05-22). 19 behavioral-contract tests in tests/mitre_tests.rs formalizing src/mitre.rs (brownfield — no src/ changes). Per-story adversarial convergence achieved in 3 consecutive clean passes (BC-5.39.001). CI run 26304328447 all jobs green. Demo evidence recorded local-only (gitignored).

STORY-005 (feature/story-005-decoder-packetlen-tcp, .worktrees/story-005) achieved per-story adversarial convergence: 8 total passes; final 3 passes (6, 7, 8) VERDICT: CLEAN on frozen artifact (story v1.6, test commit a959dee). Story is now in demo-recording / PR delivery stage.

Process-gap W3.1 raised during STORY-005 pass-8 adversarial review: the `ecNNN` suffix in test names tracks story edge-case IDs, not BC edge-case IDs — a story that renumbers its ECs produces misleading test names (e.g., `test_BC_2_02_015_ec005_...` where `ec005` refers to story EC-005, not BC-2.02.015 EC-005). Recorded in Cycle-Close Follow-Up Items; requires a follow-up story or justified deferral before cycle-close. GitHub issue filing blocked until research-agent validates per policy DF-VALIDATION-001.

Wave 3 status: STORY-071 done; STORY-005 in demo/PR stage. Wave-level adversarial convergence pending after STORY-005 merges.

**Develop HEAD:** 991e821 (was 5d4c2c6). 7 stories merged total (STORY-001/069/002/003/004/070/071).

---

## Burst P5-1 (2026-06-01) — Phase-5 whole-impl Pass 1 Remediation (ADV-IMPL-P01-MED-001)

**Agents dispatched:** adversary (whole-impl Pass 1), state-manager (BC re-anchor sweep + STATE.md + burst-log)
**Files touched:** 32 BC files in .factory/specs/behavioral-contracts/ss-04/ (source-line anchors updated); STATE.md (Phase 5 row, session checkpoint, drift items); cycles/v0.1.0-greenfield-spec/burst-log.md (this entry)
**Factory-artifacts commit:** 2b33284 — "fix(specs): re-anchor SS-04 BCs after HS-043 shifts (ADV-IMPL-P01-MED-001, DF-SIBLING-SWEEP-001)"

### Summary

Whole-implementation adversarial Pass 1 returned NOT_CONVERGED: 0 CRIT / 0 HIGH / 1 MED / 2 LOW. MED finding ADV-IMPL-P01-MED-001: 32 SS-04 BCs had stale source-line anchors for src/reassembly/mod.rs after HS-043 merges (PR #171 + #172) shifted code. No semantic/PC/invariant changes. Re-anchor sweep performed; all 32 BCs updated and committed (2b33284). Both LOW findings accepted (LOW-001: findings.rs stale doc-comment, same class as O-08; LOW-002: folded into sweep). CLEAN-PASS COUNTER = 0.

Root cause: HS-043 was dispatched without a mandatory SS-04 sibling re-sweep step (DF-SIBLING-SWEEP-001). Process gap raised as PROCESS-GAP-P5-001; requires self-improvement story or justified deferral before Phase-5 cycle close.

**Develop HEAD:** e0451ef (unchanged — no source code modified).

---

## Burst P5-2 (2026-06-01) — Phase-5 whole-impl Pass 2 Remediation (ADV-IMPL-P02-MED-001)

**Agents dispatched:** adversary (whole-impl Pass 2), state-manager (BC re-anchor + STATE.md + burst-log)
**Files touched:** specs/behavioral-contracts/ss-04/BC-2.04.052.md (v1.3→v1.4: 2 anchors — traceability row + Architecture Anchors section, mod.rs:306-312 → mod.rs:335-341); specs/behavioral-contracts/ss-04/BC-2.04.032.md (v1.2→v1.3: 1 prose anchor in Invariants, mod.rs:306-319 → mod.rs:335-349); STATE.md (Phase 5 row, session checkpoint, drift items, PROCESS-GAP-P5-001); cycles/v0.1.0-greenfield-spec/burst-log.md (this entry)
**Factory-artifacts commit:** aa6d73b — "fix(specs): re-anchor residual SS-04 anchors BC-2.04.052/.032 (ADV-IMPL-P02-MED-001, DF-SIBLING-SWEEP-001)"

### Summary

Whole-implementation adversarial Pass 2 returned NOT_CONVERGED: 0 CRIT / 0 HIGH / 1 MED. MED finding ADV-IMPL-P02-MED-001: residual SS-04 anchor drift missed by the v1.6 sweep — 51/52 SS-04 anchors were verified correct; the one remaining gap was BC-2.04.052 (2 stale anchor strings: traceability row + Architecture Anchors) and BC-2.04.032 (1 stale prose anchor in Invariants). All security surfaces (TLS, DNS, HTTP, MITRE, anomaly detection) reviewed and found robust. No semantic/PC/inv changes. Two BC files re-anchored and committed (aa6d73b). CLEAN-PASS COUNTER = 0.

Defensive sweep confirmed: no other live spec sections contain mod.rs:306 references. Remaining mod.rs:306 strings in the two BC files are intentional changelog entries (immutable audit trail).

Recurring root cause: both Pass-1 and Pass-2 findings trace to the same HS-043 mod.rs line insertion — PROCESS-GAP-P5-001 (DF-SIBLING-SWEEP-001 enforcement gap on cross-cutting reassembly merges) remains OPEN and reinforced by this recurrence.

**Develop HEAD:** e0451ef (unchanged — no source code modified).
**CLEAN-PASS COUNTER:** 0. Next action: whole-impl adversarial Pass 3 (fresh context).

---
