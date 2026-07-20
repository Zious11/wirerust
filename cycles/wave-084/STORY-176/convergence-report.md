---
document_type: per-story-convergence-report
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-07-20T23:59:00Z
phase: step-4.5-per-story-adversarial
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
story: STORY-176
cycle: wave-084
passes_total: 8
verdict: CONVERGED
criterion: BC-5.39.001
clean_streak: [P6, P7, P8]
final_head: ea4bcd8e
base: "fa9be701"
story_version: "2.7"
---

# Convergence Report — STORY-176 (compact)

## Pipeline Run: 2026-07-20
## Product: wirerust — STORY-176 Feature-IEC104 Cycle-Close: Local Gate + Tooling Hygiene Sweeps (E-11, wave-084)
## Iterations: 8

---

## Verdict: CONVERGED — BC-5.39.001 SATISFIED (3 consecutive clean passes: P6/P7/P8)

## Trajectory

`3M/5L → 1M/2L+2obs → 1M → 1M/2L → 1M/1L → NIT(0) → NIT(0) → NIT(0)`

| Pass | Verdict | HIGH | MED | LOW | Code Tip | Part A (prior-pass fix verification) |
|------|---------|------|-----|-----|----------|----------------------------------------|
| P1 | FAIL_FINDINGS | 0 | 3 | 5 | 056afabe | — (first pass) |
| P2 | FAIL_FINDINGS | 0 | 1 | 2 | 08fc7d88 | 8/8 VERIFIED-FIXED (6 fixed + 1 accepted + 1 ledgered) |
| P3 | FAIL_FINDINGS | 0 | 1 | 0 | b583c4b4 | 13/13 VERIFIED (all prior fixable + obs not re-raised) |
| P4 | FAIL_FINDINGS | 0 | 1 | 2 | ea4bcd8e | all prior fixable VERIFIED-FIXED |
| P5 | FAIL_FINDINGS | 0 | 1 | 1 | ea4bcd8e (unchanged) | 3/3 VERIFIED-FIXED |
| P6 | NITPICK_ONLY | 0 | 0 | 0 | ea4bcd8e (unchanged) | 2/2 VERIFIED-FIXED — streak 1/3 |
| P7 | NITPICK_ONLY | 0 | 0 | 0 | ea4bcd8e (unchanged) | spot-checks CLEAN — streak 2/3 |
| P8 | NITPICK_ONLY | 0 | 0 | 0 | ea4bcd8e (unchanged) | spot-checks CLEAN — streak 3/3, CONVERGED |

---

## Headline Narrative

**Pre-implementation catch (before Pass 1):** Step-2 stub-architect pre-condition probe caught a
**spec-flaw cluster** in the v2.2 story under AC-176-001. Research-agent validation (DF-VALIDATION-001,
`planning/story-176-ac001-validation.md`) confirmed the AC was substantially INVALID:
(1) wrong gate locus — the AC cited the source tree instead of `bin/check-green-doc-tense`;
(2) 91 false positives projected under the misidentified locus (the gate scans `*.rs` files, not
the gate binary itself);
(3) fabricated allowlist mechanism — the AC described an "allowlist" that does not exist in the
gate's design;
(4) inverted CHANGELOG claim — the AC stated the gate would NOT enforce the CHANGELOG obligation,
while the story's own CHANGELOG section contradicted this.
The story was corrected to v2.3 before Pass 1 even began, eliminating the spec-flaw cluster
entirely. This was the primary pre-delivery quality catch of the convergence run.

**Pass 1 (3 MEDIUM / 5 LOW):** Three pattern-logic MEDIUMs required code fixes:
- **F-S176P1-001** (MEDIUM) — pattern-29 negative lookahead too narrow; missed bare "fails until wired"
  phrase. Fixed `61f6db4c`.
- **F-S176P1-002** (MEDIUM) — pattern-26 missing trailing `\b` word-boundary anchor; matched "compiled"
  as a false positive inside "compiled". Fixed `61f6db4c`.
- **F-S176P1-003** (MEDIUM) — stale RED-phase prose at 3 loci in `bin/` test files (including a
  STORY-174 sibling); the gate's own test harness contained exactly the language the gate was
  designed to catch. DF-SIBLING-SWEEP verified all 3 loci; fixed `08fc7d88`.

**Pass 4 (CI-inert regression guard — PG-W74-CI-BIN-SELFTEST recurrence):** The new deliverable
`bin/test_gitignore_mutants_glob.py` — added during Steps 1–4 — was not wired into any CI job.
A test file never executed by CI provides no regression guarantee. This is a direct recurrence of
**PG-W74-CI-BIN-SELFTEST** (the class first codified as AC-165-001). The bin-selftest CI job existed
for prior `bin/test_*.py` files but was not extended to cover the new file. Fixed commit `ea4bcd8e`:
bin-selftest job extended per AC-165-001 pattern, job name made count-free to suppress future
count-stale drift, stale `10/14` comment reworded count-free. **PG-W84-011** filed for upstream
engine-level checklist candidacy: per-story-delivery checklist item for any new `bin/test_*.py`.

Passes 2–5 closed out residual CHANGELOG-count propagation, missing regression fixtures,
deliverable-map gaps, and spec-accuracy findings — all adversary-verified fixed before the
next pass opened. Passes 6–8 held code tip `ea4bcd8e` fixed with zero code churn, closing on
three consecutive NITPICK_ONLY passes (BC-5.39.001's 3-clean-streak criterion).

Spec evolved **v2.2 → v2.7** across the 8 passes. The v2.2→v2.3 spec-route remediation
(pre-Pass-1) was the largest structural change; subsequent v2.4–v2.7 advances addressed
AC-ID reuse disambiguation, deliverable-map completeness, traces_to currency, and
ci.yml-diff scoping accuracy.

## Fix Commits

| Commit | Description |
|--------|-------------|
| `61f6db4c` | pattern-29 negative lookahead fix + pattern-26 trailing `\b` + docstring token-list gap |
| `08fc7d88` | stale RED-phase prose scrub at 3 loci + label-blind assertion fix for patterns 26-29 |
| `b583c4b4` | CHANGELOG self-test count sync 89→91 + pattern-26 regression-guard fixture |
| `ea4bcd8e` | bin-selftest CI job extended (AC-165-001 pattern; count-free rename; stale comment reworded) |

## Non-Blocking Residuals (for gate ratification)

- **F-S176P1-004** (LOW, ACCEPTED): Pattern-(b) verb-set narrowing; pattern-(c) independently
  subsumes the are/is cases; AC-permitted design decision, not a coverage gap.
- **F-S176P2-003 + Obs-A + Obs-C** (INFO, ACCEPTED): Pattern-28/29 latent breadth/narrowness on
  inflected object phrases ("wired its/their" FP-latent, "wired the same way" FN-latent). Zero
  current-tree matches confirmed by three independent passes. AC zero-FP constraint satisfied.
- **F-S176P1-007** (LOW, LEDGERED): Gate scan set Rust-only — cannot police `bin/*.py` prose for
  stale RED-phase language. Out of story scope (ACs specify Rust source gating). PG-W84-010 filed.
- **Obs-P7-2** (NITPICK, LEDGERED): `bin-selftest` CI job absent from develop required-status-checks;
  pre-existing pattern since STORY-164/165. PG-W84-012 filed; pending intent verification.
- **Obs-P7-3** (NITPICK, RESOLVED-CLEAN at P8): Pre-existing AC-174-008 fixture coincidentally trips
  pattern-26 producing harmless 2-tuple output — shown UNREACHABLE at Pass 8 (pattern 24 precedes 26
  in the precedence trace; the AC-174-008 fixture cannot fire pattern 26 before pattern 24 fires).

## Dispositions Table

| Finding | Severity | Disposition | Fix |
|---------|----------|-------------|-----|
| F-S176P1-001 | MEDIUM | FIXED | `61f6db4c` + `08fc7d88` |
| F-S176P1-002 | MEDIUM | FIXED | `61f6db4c` |
| F-S176P1-003 | MEDIUM | FIXED | `08fc7d88` |
| F-S176P1-004 | LOW | ACCEPTED | AC-permitted; gate ratification |
| F-S176P1-005 | LOW | FIXED | `61f6db4c` |
| F-S176P1-006 | LOW | FIXED | `08fc7d88` |
| F-S176P1-007 | LOW | LEDGERED | PG-W84-010 |
| F-S176P1-008 | LOW | FIXED | story v2.4 |
| F-S176P2-001 | MEDIUM | FIXED | `b583c4b4` |
| F-S176P2-002 | LOW | FIXED | `b583c4b4` |
| F-S176P2-003 | LOW | ACCEPTED | adversary verdict: informational |
| Obs-1 | LOW | ACCEPTED | consistent with F-S176P1-004 |
| Obs-2 | LOW | ACCEPTED | harmless duplicate |
| F-S176P3-001 | MEDIUM | FIXED | story v2.5 (a90c4b4) |
| Obs-A | INFO | ACCEPTED | analogous to F-S176P2-003 |
| Obs-B | INFO | ACCEPTED | residue of F-S176P1-004 |
| F-S176P4-001 | MEDIUM | FIXED | `ea4bcd8e` + story v2.6 |
| F-S176P4-002 | LOW | FIXED | story v2.6 |
| F-S176P4-003 | LOW | FIXED | story v2.6 |
| Obs-C | INFO | ACCEPTED | consistent with F-S176P2-003 / Obs-A |
| F-S176P5-001 | MEDIUM | FIXED | story v2.7 (6ec8772) |
| F-S176P5-002 | LOW | RESOLVED-CLEAN | execution verification 2026-07-20 |
| Obs-P7-1 | NITPICK | ACCEPTED | sound; no coverage gap |
| Obs-P7-2 | NITPICK | LEDGERED | PG-W84-012; pending intent verification |
| Obs-P7-3 | NITPICK | RESOLVED-CLEAN | UNREACHABLE shown at P8 (pattern 24 precedes 26) |

## Final Verification Evidence

| Check | Result |
|-------|--------|
| Self-test suite | 91/0 pass (exit 0) |
| Gate zero-FP scan | 114 files, 0 false positives |
| Gitignore glob test | 2/0 pass |
| Cargo test suites | 94 suites, all green |
| SHA pins | 18/18 identical |
| Story input-hash | 6ec8772 (canonical Python tool `bin/compute-input-hash`) |
| Code tip | ea4bcd8e (8 commits over develop fa9be701) |
| Story version | v2.7 |

## Process Gaps Noted for Cycle Close

- **PG-W84-010**: Gate scan Rust-only — cannot police `bin/*.py` prose for stale RED-phase language
  (gate's own test harness is out of scope; requires separate story). LEDGERED.
- **PG-W84-011**: CI-inert new `bin/test_*.py` (pattern recurrence of AC-165-001;
  `bin/test_gitignore_mutants_glob.py` delivered without CI wiring; caught at Pass 4).
  Engine-level checklist candidate: add per-story-delivery checklist item for any new `bin/test_*.py`.
- **PG-W84-012**: `bin-selftest` CI job absent from develop required-status-checks
  (pre-existing since STORY-164/165; self-test guards do not gate merges; pending intent verification).
  LEDGERED.
- **PG-HASH-HOOK-DIVERGENCE** reconfirmed: advisory-only hook noise (bash `$(cat)` newline-stripping
  vs. canonical Python `bin/compute-input-hash`) recurred across passes with no actionable content;
  consistent with CLAUDE.md documented advisory-only treatment.

## Traceability

- Full pass-by-pass state: `adversary-convergence-state.json` (this directory)
- Per-pass findings detail: `FINDINGS.md` (this directory)
- Story: `.factory/stories/STORY-176.md` (v2.7)
- STORY-INDEX: `.factory/stories/STORY-INDEX.md` (v3.83)
