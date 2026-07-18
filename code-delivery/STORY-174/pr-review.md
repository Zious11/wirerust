# PR #409 Review — STORY-174 IEC-104 Formal Hardening

## Verdict: APPROVE

Fresh-eyes review by the pr-reviewer (different model family, information-asymmetry
wall enforced). Scope reviewed: the diff, the PR description, and the test evidence —
no `.factory/` internals or prior adversarial-pass history.

This is a formal-hardening PR: verification artifacts, test-harness upgrades, and CI
tooling only. All 16 changed files reviewed against the 8-item checklist. No blocking
findings.

---

## What was verified (no rubber-stamping)

**Invariant 1 — No new production behavior (CONFIRMED).**
The only production-file change, `src/analyzer/iec104.rs` (+69/-22), is entirely within
the `#[cfg(kani)] mod kani_proofs` block plus module doc-comments. `#[cfg(kani)]` is
compiled out of normal `cargo build`/`test`, so no runtime path is altered. The fuzz
target changed only doc-comment lines (harness body byte-identical). No `Finding.direction`
/ `track_ns_desync` edits.

**Invariant 2 — VP-045 harnesses non-vacuous (CONFIRMED).**
- `proptest_vp045_direction_isolation`: sound witness-replay design (combined interleaved
  run vs. C2S-only and S2C-only witnesses) with `prop_assert_eq!` on carry equality and
  `prop_assert!` on the 255-byte bound. Real isolation oracle, not a tautology.
- `proptest_vp045_independent_run_equivalence`: asserts `carry_c2s`/`carry_s2c`/`frame_count`
  equality across two instances, with explicit `prop_assert!(false, ...)` on the
  mismatched-state arm. Non-vacuous. F-172-003 closed.

**Invariant 3 — VP-044 covers all 5 facets, not 4 (CONFIRMED).**
Explicit `kani::assert` for each of BC-2.19.001–005, including the previously-missing
Facet 5 (`valid → Some`, F-174-001). Evidence file `ac-174-001` shows Check 13
(assertion.13) SUCCESS for "valid input ... must return Some (BC-2.19.005)". 89/89 pass.

**Invariant 4 — CHANGELOG (CONFIRMED).**
`[Unreleased]` entry present; CHANGELOG gate CI job passed.

**Invariant 5 — Scrub gate (CONFIRMED).**
Independent grep of all 9 evidence files for `/Users/`, `/home/`, `/root/`, `C:\` returned
zero hits; transcripts use `<repo>/` placeholders.

**Invariant 6 — IEC104-FINDING-DIRECTION-001 out of scope (CONFIRMED).**
No production emit-site changes anywhere in the diff; routing documented (D-461, VALID-DEFER).

**Also verified:**
- VP-046 proptest is a genuine independent-oracle assertion over all 256 CF1 values (body
  predates this PR; only stale comments scrubbed).
- 2 targeted mutant-kill tests are meaningful: line-1220 test distinguishes `< 2` from
  `<= 2`; partial-cap test distinguishes `len - cap` from `len + cap` (prior tests used
  cap=0 where both compute 0).
- Mutation math self-consistent: 156 = 117 caught + 28 cfg(kani) + 5 prod-equivalent + 6
  unviable; in-scope 122; 117/122 = 95.9% (>= 80% threshold).
- CI: Clippy, Format, Test, CHANGELOG gate, Green-doc-tense gate, Semantic PR, Deny,
  Fuzz build, Trust-boundary — all pass. Only Audit is `pending` (not failed).

---

## Findings

| # | Severity | Category | Finding | Disposition |
|---|----------|----------|---------|-------------|
| 1 | NIT | coverage | Demo evidence is `.txt` command transcripts, not `.gif`/`.webm`. | Accepted. For a formal-hardening story with no visual/interactive surface, textual Kani/fuzz/proptest/mutation transcripts are the correct and only meaningful evidence form. Transcripts are genuine and row-verified against claimed commands and counts. |
| 2 | NIT | size | Diff is 865 additions / 66 deletions (>500-line flag threshold). | Accepted. Fully coherent with story scope — test/harness/tooling only, no production behavior. |

No MINOR/MAJOR/CRITICAL findings.

---

## Notes for pr-manager
- `mergeStateStatus` is `BLOCKED` (awaiting review / Audit pending) — not a diff problem.
- Every per-AC claim in the PR body cross-checks against its evidence artifact
  (89/89, 618,615 fuzz runs, 440 + 122 Kani checks, 72/72 self-test, 95.9% mutation).
  Row-verify mandate (PG-W74-PRDESC-ROW-VERIFY) satisfied.

Recommend merge once Audit resolves green.
