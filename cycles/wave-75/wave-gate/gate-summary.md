---
document_type: wave-gate-summary
wave: 75
stories: [STORY-165]
gate_verdict: PASS
develop_head: fa646ed89cdd1d0e9a703c6d9b30a4c90256dc7f
date: 2026-07-13
decision: D-435
---

# Wave 75 Integration Gate Summary

**GATE VERDICT: PASS — CONVERGED (D-435, human-approved 2026-07-13)**
develop HEAD at gate close: `fa646ed89cdd1d0e9a703c6d9b30a4c90256dc7f`
Develop chain (wave-75 window): `fa646ed` (PR #398 STORY-165, squash-merged 2026-07-13)

---

## Gate Dimensions (a)–(h)

| Dim | Name | Verdict | Key Evidence |
|-----|------|---------|-------------|
| (a) | Full Suite | PASS | CI 13/13 green on PR #398 (bin-selftest first-ever run AC-165-001; changelog-gate PASS; clippy -D warnings clean; fmt clean; action-pin-gate PASS); cargo test + python bin/ self-tests all green on develop tree |
| (b) | Wave Adversarial | PASS | 7 passes; streak 3/3 (W5/W6/W7); trajectory 2→0→0→1→0→0→0; CONVERGED |
| (c) | Code Review | PASS | 0 BLOCKING/MAJOR/MINOR; 1 NIT DEFERRED (human-ratified 2026-07-13; joins ROUTE-W74-DEFERRED, next bin-touch PR); 2 OBS |
| (d) | Security | PASS | PR #398-level security review CLEAN (adjudicated: wave delta is CI-yaml + CLAUDE.md docs only — no new production code paths, no separate wave security pass required; adjudication recorded this document) |
| (e) | Consistency | PASS | Currency sweep (AC-165-003 first mandatory execution) completed; STORY-INDEX v3.56 consistent; no corpus contradictions found |
| (f) | Holdout | N/A | Zero src/ changes (STORY-165 is CI-yaml + CLAUDE.md governance-only); holdout evaluation not applicable |
| (g) | Wave Demos | PASS | 4 demo-evidence artifacts (demo-evidence/story-165/ — AC-165-001/002/003/004); scrub PASS (zero host paths per demo-evidence-scrub-gate.md) |
| (h) | Input-hash | PASS | MATCH=119 STALE=0 (canonical Python tool; 119 total stories scanned post-STORY-166 draft) |

---

## Wave Adversarial Convergence Detail (Dimension b)

| Pass | Develop HEAD | Verdict | Key Findings / Actions |
|------|-------------|---------|----------------------|
| W1 | fa646ed | NOT-CLEAN (LOW×2) | F-W75G-P1-001 LOW: STATE.md D-434 record lacked merge-authorization attribution; appended "squash-merge HUMAN-AUTHORIZED at orchestrator merge-authorization gate — pr-manager executed per DF-MERGE-AUTH-CLASSIFIER-001/DF-PR-MANAGER-COMPLETE-001 steps 8-9" inline, matching D-431 attribution style. Adjudication note added: delivery D-records SHOULD carry merge-auth attribution going forward. F-W75G-P1-002 LOW: currency-sweep.md Step 2 tense-audit paragraph cited STORY-165.md "line 84" for delivery-anchor location; ground truth verified as lines 81-82 (gap-closed annotation spans two lines). Corrected to "lines 81-82". Both fixed. |
| W2 | fa646ed | CLEAN | 0 findings; streak #1 |
| W3 | fa646ed | CLEAN | 0 findings; streak #2. Note: F-W75G-P3-002 filed against process-gap-ledger was research-adjudicated redundant to PG-W75-FINDING-ID-DUAL-SCHEME already in ledger; closed as ledger-redundant (no fix required; streak continues). |
| W4 | fa646ed | NOT-CLEAN (MEDIUM) | F-W75G-P4-001 MEDIUM: currency-sweep.md Step 3 blanket-provenance claim "All four demo-evidence files were captured in worktree `ci/story-165-bin-selftest` at commit 9ae8b04" was incorrect — AC-165-002/003/004.md were captured on factory-artifacts branch (main repo cwd), not the worktree. Sentence split per Method headers; one legitimate single-file attribution for AC-165-001.md retained. Sibling sweep confirmed no other blanket-aggregate sentences. Streak counter reset to 0. |
| W5 | fa646ed | CLEAN | 0 findings; streak #1 |
| W6 | fa646ed | CLEAN | 0 findings; streak #2. OBS-W75-W6: mid-gate CLEAN passes not incrementally recorded in findings.md — streak-persistence visibility gap; codified as STORY-166 AC-166-004. |
| W7 | fa646ed | CLEAN | 0 findings; streak #3. OBS-W75-W7: demo-evidence scrub scope does not explicitly list `.factory/demo-evidence/` root alongside `.factory/cycles/<cycle>/demo-evidence/`; codified as STORY-166 AC-166-003. |

Streak: W5 / W6 / W7 = 3/3. **CONVERGED.**
Trajectory (finding counts per pass): 2→0→0→1→0→0→0

**Substantive defects caught post-merge (3):** Merge-authorization attribution gap ×1
(F-W75G-P1-001); line-citation stale ×1 (F-W75G-P1-002); blanket provenance over-claim ×1
(F-W75G-P4-001). All 3 fixed before CONVERGED declaration.

---

## PRs Merged During Wave Window

| PR | SHA | Title | Notes |
|----|-----|-------|-------|
| #398 | fa646ed | ci: add bin-selftest job + register STORY-165 governance mandates in CLAUDE.md (STORY-165) | STORY-165: `.github/workflows/ci.yml` new `bin-selftest` job (job 13) + 2 CLAUDE.md Project References rows; CI 13/13; per-story adversary 9-pass streak 3/3 (P7/P8/P9); trajectory 1→0→0→2→0→1→0→0→0; PG-W74-PRDESC-ROW-VERIFY first compliant execution (9 rows row-verified) |

---

## S-7.02 Disposition

**SATISFIED.**

STORY-166 drafted at STORY-INDEX v3.56 (119 stories / 731 pts); wave-TBD (E-11, 5 pts, v1.0, hash 8e244ad).
STORY-166 codifies five items identified during wave-75:

| Item | AC | Description |
|------|----|-------------|
| PG-W75-VALIDATE-CITATIONS-SYMBOL-GAP | AC-166-001 | `bin/validate-citations` validates line-in-bounds but not symbol-at-line; fabricated symbol names at in-bounds lines pass preflight silently (F-S165P1-001 instantiated this class) |
| PG-W75-FINDING-ID-DUAL-SCHEME | AC-166-002 | Two colliding finding-ID forms (`F-W<NN>G-P<n>` canonical vs `F-W<NN>P<n>` G-less) exist repo-wide; authors reach for the non-canonical form and misnumber passes (F-S165P4-001 instantiated this) |
| OBS-W75-W7 demo-evidence scrub scope | AC-166-003 | demo-evidence-scrub-gate.md scope does not enumerate `.factory/demo-evidence/` as scrub root alongside cycles path |
| OBS-W75-W6 mid-gate streak persistence | AC-166-004 | Wave-gate findings.md not updated for CLEAN passes; streak persistence opaque mid-gate |
| PG-W75-GATE-SUMMARY-VERSION-ATTRIBUTION | — | One-line factual correction applied in-burst to `cycles/wave-74/wave-gate/gate-summary.md:43` (v3.48 → v3.47 with bracketed note); no separate AC needed |

**Justified deferral (recorded in STATE.md Drift Items):** sprint-state.yaml `merge_sha`/`merge_commit` field inconsistency. The file self-declares it is a vestigial greenfield artifact; STORY-INDEX is the authoritative wave registry. Field-form inconsistency is cosmetic drift in a vestigial file. Target: vestigial-file retirement decision at next housekeeping pass.

---

## Code Review Findings (Dimension c)

All code-review findings human-ratified (2026-07-13).
Full finding text in `cycles/wave-75/wave-gate/code-review.md`.

| ID | Severity | File | Description | Disposition |
|----|----------|------|-------------|-------------|
| NIT-1 | NIT | `.github/workflows/ci.yml:466-467, 477, 479` | Hardcoded test counts `(22 tests)` / `(10 tests)` in comment and step names will silently stale | DEFERRED (human-ratified 2026-07-13) — joins ROUTE-W74-DEFERRED; batch next bin-touch PR |

**ROUTE-W74-DEFERRED carry-forward:** Wave-74 code-review MINOR ×2 + NIT ×4 + OBS ×2 (against `bin/` scripts not touched by wave-75) carry forward to next bin-touch PR unchanged.

---

## Security Adjudication (Dimension d)

Wave-75 delta: `.github/workflows/ci.yml` new `bin-selftest` job + `CLAUDE.md` 2 reference rows.
No new production Rust code paths. No new bin/ script logic. No new external dependencies.
CI-yaml and docs-only changes carry no CWE-class exposure beyond existing repository surface.
PR-level security review (per-story gate) was APPROVE with 0 findings.

**Adjudication:** A separate wave-level security pass is not required for a CI-yaml+docs-only delta. This adjudication recorded per AC-158-006 transparency obligation.

---

## Currency Sweep Summary (Dimension e)

First mandatory execution of `delivery-doc-currency-protocol.md` (AC-165-003) at wave gate.
Currency sweep completed pre-adversarial-pass. Two low-level corrections surfaced and fixed
(F-W75G-P1-002: line citation; F-W75G-P4-001: blanket provenance claim) — both caught at
W1/W4 adversarial passes confirming the sweep does not eliminate all drift.

Input-hash scan MATCH=119 STALE=0 (canonical Python tool, post-STORY-166 draft).

---

## Session Process Observations

| ID | Observation | Mitigation |
|----|-------------|-----------|
| PROC-OBS-W75-001 | Wave-75 gate converged in 7 passes (3 finding passes + 4 CLEAN passes), significantly fewer than wave-74's 13-pass gate. Single-story governance waves with well-prepared delivery docs produce shorter gate runs when the currency sweep pre-pass is executed before adversary Pass 1. | AC-165-003 delivery-doc currency protocol pays forward; Lesson 2 recorded. |
| PROC-OBS-W75-002 | F-W75G-P3-002 finding against the process-gap-ledger was ledger-redundant (already covered by PG-W75-FINDING-ID-DUAL-SCHEME). Research-adjudication confirmed redundancy; no streak impact (streak continued). | Lesson 2 codified: the checker gets checked; redundancy catch by research is correct behavior. |
| PROC-OBS-W75-003 | Row-verify mandate (AC-165-002) first compliant execution on its own delivery PR (STORY-165 PR #398). Mandate described its own evidence correctly; no self-referential gap on the first run. | Lesson 5 recorded as observation. |
