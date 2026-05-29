---
document_type: lessons
cycle: drift-remediation-2026-05-29
date: 2026-05-29
produced_by: state-manager
---

# Drift Remediation 2026-05-29 — Lessons Learned

## DR.L1 — Validate-Before-Fix Pays Off (DF-VALIDATION-001)

A research-agent validation pass before remediation cut the 62-item backlog substantially:
approximately 13 items were already-resolved, invalid, or duplicate before any remediation
work began. DF-VALIDATION-001 prevented wasted effort and false issue-filing. The policy
investment (Phase 0 / PR #99) paid clear dividends at scale.

**Pattern:** When a backlog of deferred findings accumulates across multiple waves, a bulk
validation pass eliminates phantom work before execution.

## DR.L2 — pr-manager APPROVE Recurrence (DF-PR-MANAGER-COMPLETE-001 Upstream)

pr-manager AGAIN stopped at APPROVE on PR #147, triggering a manual orchestrator merge.
This is the Nth recurrence of DF-PR-MANAGER-COMPLETE-001 (W9.L3, W10.L3, W11.L5, confirmed
again here). The policy is correct and injected at dispatch time, but the root-cause fix
requires an agent-prompt change in the vsdd-factory plugin source — not in this repo.

**Pattern:** Policy enforcement that lives only in dispatch-injection text is fragile.
Structural fixes (agent-prompt changes) are upstream; document the workaround clearly.

**Action:** W10-D7 escalation to plugin maintainer confirmed necessary.

## DR.L3 — Feasibility Probe Before Treating Coverage Items as Real Gaps

A documented "coverage gap" (F-W16-S052-P2-002 / BC-2.07.001 EC-002 inner Err arm) turned
out to be UNREACHABLE dead code on investigation — the nom `many0`/`complete` semantics
prevent the branch from being exercised through the public on_data API. The item was
reclassified WONT-FIX-BY-DESIGN.

**Pattern:** Before treating a "needs-test" finding as a real spec gap, probe feasibility:
Can the trigger condition actually be induced through the public API? If not, the branch
is defensive dead code, not a coverage gap. Research-agent feasibility checks on coverage
items should be the first step.

## DR.L4 — Bulk Template Defects Require Repo-Wide Sweep, Not Per-Item Handling

SS-01's capabilities.md citation fix (DF-16.A, 8 files) surfaced a 209-file project-wide
blast radius (DF-16.B) — a single broken citation template at Phase-1a authoring propagated
to every BC across SS-02..SS-13. Fixing one SS per wave would take 12+ waves.

**Pattern:** When a structural defect in a template or authoring tool affects a corpus
of generated files, the correct response is a single bulk find-replace sweep across the
entire corpus, not per-item version-bump handling. DF-16.B is queued for a dedicated
bulk mechanical sweep rather than incremental fixes.

**Action:** DF-16.B added to open Drift Items with explicit "bulk sweep" disposition.

## DR.L5 / PG-HASH-001 — input-hash MUST Be Set via Canonical Tool, Never Hand-Computed

During drift-convergence remediation 2026-05-29, 12 stories were found to have stale
input-hashes (--scan reported STALE=12). The root cause was that story-writer hand-computed
hashes as sha256 over sorted inputs-file names, while the canonical `bin/compute-input-hash`
tool uses MD5 over the inputs-order file list. Every affected story had a plausible-looking
7-char hash in the right field — the error was invisible until a scan ran.

**Pattern:** Tool output and hand-computation diverge even when both appear "reasonable".
Only the canonical tool output is authoritative; any hash not produced by the tool will
fail `--check`/drift-scan silently until a scan is run.

**Rule:** input-hash MUST always be set via `bin/compute-input-hash --update`, NEVER
hand-computed by any agent. All 12 affected stories corrected in the drift-convergence
remediation burst (2026-05-29). Tool verification: --scan reports MATCH=48 STALE=0 after fix.

**Policy-codification candidate:** DF-INPUT-HASH-CANONICAL-001 — to be evaluated at next
governance pass. Story-writer and PO agent prompts should mandate `bin/compute-input-hash
--update` as the only permitted hash-setting mechanism (note: per DF-ADVERSARY-TOOLCHAIN-PAIRING-001,
the adversary read-only profile cannot execute the tool; orchestrator must run it before
adversary dispatch and include the result in the adversary's Supplied Evidence section).

## DR.L6 — PO Citation Fixes: 16 BCs + cap-02 Anchor + 2 LOW BC Prose

Drift-convergence remediation batch (2026-05-29) included uncommitted PO edits:

- **16 BC citation fixes (DF-16.B partial):** SS-04 (BC-2.04.012/019/025/026/041/045/047),
  SS-05 (BC-2.05.001/003/008), SS-06 (BC-2.06.004/005/006/007/020). Each BC updated
  stale `capabilities.md §CAP-NN` citations to `domain/capabilities/cap-NN-<slug>.md` form.
  Version bumps recorded in each BC's changelog.
- **cap-02-link-type-gating.md anchor fix:** reader.rs line reference corrected from
  `:22-35` → `:50-61` (stale anchor from Phase-1a authoring); BC references table extended
  to include BC-2.01.001 per DF-16.A.
- **BC-2.07.034 v1.4 (LOW prose):** Corrected invariant-1 `if done { return; }` prose:
  guard line is tls.rs:722, return line is tls.rs:723 (previously conflated as single "723"
  citation). Capability citation also updated (F-DRIFT2A-001 + F-DRIFT2A-003).
- **BC-2.04.045 v1.5 (LOW prose):** Mid-loop guard anchor corrected from segment.rs:175-179
  → 178-180 (175-179 included loop setup lines; 178-180 is the if-block itself). Capability
  citation also updated.

These edits were committed in the single drift-convergence remediation burst alongside the
input-hash corrections and bookkeeping fixes.

## DR.L7 [process-gap] — STORY-INDEX Status Column Has No Automated Sync-on-Merge

STORY-INDEX.md status column and individual story frontmatter `status:` fields had no
automated mechanism to transition from draft/in-progress to completed when a story PR is
merged. This session reconciled 11 stories (STORY-005/011/012/013/014/015/019/031/032/066/071)
whose STORY-INDEX.md rows still showed `draft` despite sprint-state.yaml showing `status: done`
with a `merge_commit`. STORY-031 and STORY-032 also had stale frontmatter (draft and
in-progress respectively).

**Root cause:** PG-W16-002 data manifestation — the recurring "no pipeline gate transitions
story status on merge" gap (W1.3/W2.5) manifested as ~11 stories left unreconciled across
Waves 3-13.

**Validation-hook candidate:** assert STORY-INDEX status column == frontmatter `status:` for
every row + dependency-completion invariant (all `depends_on` of a `completed` story must
themselves be `completed`). Analogous to `compute-input-hash --scan` producing STALE counts.
Should run after every wave-close burst and report discrepancies before artifacts are committed.

**Scope of fix:** STORY-INDEX.md 11 rows updated to `completed`. STORY-031 frontmatter 1.6→1.8
+ changelog. STORY-032 frontmatter 1.3→1.4 + changelog. sprint-state.yaml unchanged (was
already authoritative).

## DR.L8 [process-gap] — Archive Generation Left Placeholder Headings Inflating Headcount

The closed-items.md archive was generated with 4 `### F-XXX (already listed above)` placeholder
headings that were deduplication artifacts. These inflated the RESOLVED-FIXED-THIS-SESSION
section to 26 heading entries while the real distinct count was 22. The summary table then
over-stated the count as 23 (not 22). Additionally W2.6/W11-D6 appeared in both
RESOLVED-FIXED-THIS-SESSION and DUPLICATE, causing a double-classification that further
distorted totals (claimed 57, corrected to 56).

**Root cause:** Archive-generation step did not re-derive counts from distinct `### ` headings
after deduplication; placeholder headings were not removed before counting.

**Rule:** State-manager archive step MUST re-derive all per-bucket counts from the actual
distinct `### ` headings in the section, excluding any `(already listed above)` or similar
placeholder text, before writing the summary table. Cross-bucket double-classification must
be detected by checking that each item ID appears in exactly one bucket.
