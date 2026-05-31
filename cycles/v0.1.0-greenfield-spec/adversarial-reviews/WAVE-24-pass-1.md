# Adversarial Review — WAVE-24 Pass 1

- **Target:** WAVE-LEVEL convergence — STORY-087 + STORY-096 (Wave 24)
- **Scope:** full (the two stories merged this wave + their cli.rs surface + indexes)
- **Develop tip reviewed:** `9954d44c92b8df149c1d6d969f832eb2ac0552b2` (verified local == origin/develop)
- **Subsystems:** SS-12 (E-9, STORY-087) + SS-13 (E-10, STORY-096); target_module `cli`
- **Lenses:** (1) CONSISTENCY, (2) INTEGRATION-STATIC, (3) TRACEABILITY
- **Date:** 2026-05-31
- **Reviewer:** orchestrator-executed adversarial protocol

> **Fresh-context disclosure (methodology honesty per skill Iron Law):** This
> harness exposes no primitive to spawn a transcript-isolated sub-agent. The
> review was therefore executed by the orchestrator directly, but ground truth
> was rebuilt exclusively from PRIMARY sources (the two test files, `src/cli.rs`,
> the three/four input BC files, the story files, STORY-INDEX, BC-INDEX, STATE.md)
> — NOT from the pre-existing per-story pass files (`pass-*-STORY-087.md`,
> `pass-*-STORY-096.md`). Prior per-story findings were not read before forming
> this pass's findings. The information-asymmetry intent is approximated as far
> as the environment allows; the limitation is recorded here so the convergence
> record is not overstated.

## Method / Evidence Gathered

| Check | Command / Source | Result |
|-------|------------------|--------|
| Develop freshness | `git rev-parse HEAD` vs `origin/develop` | both `9954d44` ✓ |
| Full suite green | `cargo test --all-targets` | **1015 passed, 0 failed** ✓ |
| Clippy gate | `cargo clippy --all-targets -- -D warnings` | clean ✓ |
| Fmt gate | `cargo fmt --check` | clean ✓ |
| Module wrappers | grep `^mod ` in both test files | `mod story_087`, `mod story_096` (distinct) ✓ |
| Test counts | grep `fn test` | 087=16, 096=14 (matches docstrings) ✓ |
| Cross-tree fn-name uniqueness | each 087/096 fn name grepped across `tests/` | every name resolves to exactly 1 file ✓ |
| AC→test-name sync (DF-AC-TEST-NAME-SYNC-001) | every story `**Test:**` citation grepped in cited file | 12/12 (087) + 10/10 (096) resolve uniquely ✓ |
| Shared cli.rs surface | field types in `src/cli.rs` vs test assertions | all match (see CONSISTENCY) ✓ |
| BC inputs exist | 7 BC files under ss-12/ss-13 | all present ✓ |
| Merge commits on develop | `git log develop` | #164→`c2445dc` (087), #165→`9954d44` (096) ✓ |
| Per-story convergence reports | `adversarial-reviews/` | 087: pass-2/3/4; 096: pass-1..6 present ✓ |

## Lens 1 — CONSISTENCY (cross-story coherence)

Both stories assert against the SAME shared `src/cli.rs` `Cli` struct. Verified the
two stories' assertions about that shared surface do not contradict each other:

- `output_format: Option<OutputFormat>` (cli.rs:49) — 087 asserts Some(Json)/Some(Csv)/None; 096 does not touch it. No conflict.
- `reassemble`/`no_reassemble` bools with `conflicts_with` (cli.rs:62-67) — 087 only. No conflict.
- Threshold field TYPES agree exactly with test literals: `overlap_threshold: Option<u32>` (test `Some(255u32)`), `small_segment_max_bytes: Option<u16>` (test `Some(0u16)`), `small_segment_ignore_ports: Option<Vec<u16>>` (test `Some(vec![23u16,513u16])`), `out_of_window_threshold: Option<u32>`. ✓
- Subcommand-choice divergence: 087 uses `summary` in argv vectors; 096 uses `analyze`. NOT a contradiction — all flags under test are `global = true`, so either subcommand is valid; the BC canonical vectors themselves use `analyze` (BC-2.12.005 line 74) and `summary` is equally valid. No finding.
- Both files use identical `parse_ok`/`parse_err` helper idiom and `#![allow(non_snake_case)]` + per-story `mod` wrapper. Consistent style. ✓

**CONSISTENCY verdict: no contradictions between the two test files or against the shared cli.rs surface.**

## Lens 2 — INTEGRATION-STATIC (coexistence)

- `mod story_087` and `mod story_096` live in separate files; no flat-namespace tests (DF-TEST-NAMESPACE-001 satisfied). ✓
- No `fn test_*` name from either file collides with any other function across the entire `tests/` tree (verified by per-name grep). ✓
- Full `cargo test --all-targets` compiles and runs all 1015 tests green, including both new modules. ✓
- clippy `-D warnings` and `cargo fmt --check` both clean — the two modules coexist without lint/format regressions. ✓

**INTEGRATION-STATIC verdict: both modules coexist cleanly; full suite green; no collisions.**

## Lens 3 — TRACEABILITY

Every BC postcondition/invariant/EC cited maps to a real test, and every story
`**Test:**` citation resolves. BC anchors spot-verified against source:

| BC | Claim | Test | BC source verified |
|----|-------|------|--------------------|
| BC-2.12.004 | PC1/2/3/4: output-format json/csv/None/xml-reject | AC-001..004 | ✓ |
| BC-2.12.005 | PC3 depth=10, PC4 memcap=1024, PC5 thresholds None-when-absent, PC6 overlap 0-255 | AC-005..009 | BC lines 45-49 + inv3 match ✓ |
| BC-2.12.005 | EC overlap 256 reject / 255 accept | AC-008 + EC-003 | BC EC table line 66-67 ✓ |
| BC-2.12.007 | PC1 ArgumentConflict, inv1 symmetric, EC-003 "--reassemble alone OK" | AC-010..012 | BC lines 51/56/66 ✓; `conflicts_with` at cli.rs:62 ✓ |
| VP-018 (087 frontmatter) | both flags → ArgumentConflict | anchored in BC-2.12.007 line 82 | ✓ |
| BC-2.13.001..004 | UnknownArgument rejection of --threats/--beacon/--filter/--verbose + absence invariants | 096 AC-001..010 + EC-001..004 | LESSON-P1.04 comment present cli.rs:22-35; flags absent ✓ |
| STORY/BC-INDEX | both stories + all 7 BCs have INDEX rows | STORY-INDEX L73/77, BC-INDEX L283-309 | rows resolve ✓ |
| demo-evidence epic-rollup (F-W22-T1 cross-check) | none found for Wave-24 / E-9 / E-10 | N/A — no rollup artifact exists to contradict | not applicable |

## Findings

### F-W24-P1-001 — STATE.md + STORY-INDEX stale: STORY-096 merged but recorded as pending/in-progress  [MEDIUM]
- **Lens:** TRACEABILITY (+ DF-CONVERGENCE-BEFORE-MERGE-001 detection signal).
- **Evidence:** Actual develop tip is `9954d44` with STORY-096 squash-merged via PR #165. But:
  - `STORY-INDEX.md` L77 shows STORY-096 status `in-progress` (should be `completed`/merged).
  - `STATE.md` L37/40/41/58/71/82 all say "develop HEAD c2445dc", "STORY-096 NEXT/pending", "Wave 24 IN PROGRESS". These reflect the PRE-096-merge snapshot.
- **Why it matters:** DF-CONVERGENCE-BEFORE-MERGE-001 lists "merge commit SHAs present on develop for stories whose status is not advanced" as a violation/detection signal. The merge is legitimate (per-story convergence reports pass-1..6-STORY-096 exist), so this is a STATE-update lag, not a premature merge — but the index/STATE must be reconciled to `9954d44` and STORY-096 marked merged, and the Wave-24 wave-level convergence status updated once this review concludes.
- **Severity rationale:** MEDIUM — no code/test defect; pure state-record drift that would mislead a resuming session about what is merged. Not blocking the tests' correctness.
- **Disposition:** Route to state-manager to (a) advance STORY-096 to merged in STORY-INDEX, (b) update STATE.md develop HEAD to `9954d44` and Wave-24 narrative to "both stories merged; wave-level convergence in progress".

### F-W24-P1-002 — FSR / Token-Budget rows in both stories cite `tests/cli_tests.rs`; tests actually live in dedicated per-story files  [LOW]
- **Lens:** TRACEABILITY (+ DF-SIBLING-SWEEP-001 — same pattern in both stories).
- **Evidence:** STORY-087 L133 & L179 and STORY-096 L126 & L173 all cite `tests/cli_tests.rs` (modify). Tests actually landed in `tests/cli_story_087_tests.rs` and `tests/cli_story_096_tests.rs` (mandated by DF-TEST-NAMESPACE-001, correctly applied in the test files).
- **Why it matters:** A reader following the File Structure Requirements row would look in the wrong file. The story body's own placement decision (dedicated file) contradicts its FSR/Token-Budget rows.
- **Severity rationale:** LOW — non-load-bearing documentation drift; the authoritative DF-TEST-NAMESPACE-001 placement is correctly realized in code. Per-story STORY-087 convergence already accepted the identical FSR-row drift as "Low non-blocking (namespace-policy precedent)" (STATE.md L80, F-S087-P2-001). This pass records the STORY-096 sibling instance for completeness and uniform disposition.
- **Disposition:** Optional cleanup — update both stories' FSR + Token-Budget rows to the actual `cli_story_08{7}/096_tests.rs` filenames in a single story-writer burst (DF-SIBLING-SWEEP-001 single-burst). Non-blocking for wave convergence.

## Non-findings (investigated, dismissed)

- **BC-2.13.004 BC-INDEX priority `P2` vs STORY-096 frontmatter `priority: P1`** — NOT a contradiction. The BC-INDEX P2 is the behavioral-contract's own priority tag; the story `priority` is delivery priority. The BC file itself carries no conflicting per-BC priority field. Dismissed.
- **087 EC-001 prose says "0 is a valid u64" but field is `usize`** — cosmetic prose imprecision (`usize`/`u64` both unsigned, both accept 0); the test asserts the correct behavior (`reassembly_depth == 0`). Not worth a finding.
- **087/096 subcommand divergence (`summary` vs `analyze`)** — both valid for global flags; BC vectors use both. Dismissed (see CONSISTENCY).

## Policy Rubric Compliance (12 policies in .factory/policies.yaml)

| Policy | Verdict |
|--------|---------|
| DF-VALIDATION-001 (research-validate deferred findings before issue) | N/A this pass — no issues filed; F-W24-P1-001/002 are state/doc fixes, not deferred-finding issues. |
| DF-SIBLING-SWEEP-001 (sibling sweep) | F-W24-P1-002 IS a sibling instance (same FSR drift in both stories) — flagged for single-burst fix. Otherwise satisfied (per-story `mod` applied to both). |
| DF-PR-MANAGER-COMPLETE-001 | Satisfied — both PRs (#164, #165) merged, branches deleted, develop advanced. |
| DF-AC-TEST-NAME-SYNC-001 (+v2 unique-resolution) | Satisfied — 22/22 citations resolve to exactly one fn; no cross-module ambiguity. |
| DF-ADVERSARY-METHODOLOGY-001 (absolute paths) | Satisfied — all file ops used absolute paths; no cross-repo `cd` drift. |
| DF-CONVERGENCE-BEFORE-MERGE-001 | Merges are backed by per-story convergence reports (087 3-clean P2/P3/P4; 096 pass-1..6). F-W24-P1-001 is the post-merge STATE-lag detection signal, not a premature-merge violation. |
| DF-DEVELOP-FRESHNESS-001 | Satisfied — reviewed tip `9954d44` == origin/develop; reported in header. |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | N/A (single reviewer; no toolchain pairing claim made). |
| DF-INPUT-HASH-CANONICAL-001 | 087 input-hash `1de3972`, 096 `86fa3d1` present in frontmatter; not recomputed this pass (no input BC content changed). |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | Satisfied — clean tree, frozen tip. |
| DF-TEST-CITATION-SWEEP-001 | No test-citation re-pointing performed this pass. F-W24-P1-002 fix (if taken) must sweep all 5 locations. |
| DF-TEST-NAMESPACE-001 | Satisfied in CODE (both test files use per-story `mod`). The story FSR rows (F-W24-P1-002) still name the old flat file — the doc-side drift. |

## Trajectory

- Pass 1 findings: **2** (1 MEDIUM state-drift, 1 LOW doc-drift). 0 Critical, 0 High.
- Both findings are documentation/state-record reconciliation, not test-correctness or integration defects. The wave's actual deliverables (30 tests across 2 modules, full suite green, clippy/fmt clean, complete BC traceability) are sound.
