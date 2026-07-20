---
document_type: process-gap-ledger
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-07-20T17:33:00Z
cycle: "wave-084"
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
---

# Process-Gap Ledger — wave-084

Process-gap candidates captured during wave-084 for cycle-close codification.
Each entry requires DF-VALIDATION-001 research-agent validation before filing
as a GitHub issue (product-local) or upstream drbothen/vsdd-factory issue.

---

## PG-W84-001 — stale-inline-version-marker recurrence

**Class:** Stale inline version marker / version annotation hygiene
**Caught by:** PR reviewers across wave-84 delivery (STORY-147, STORY-166)
**Severity:** LOW (cosmetic; does not affect behavior)
**Occurrences:** 3+ in wave-84 alone (pattern repeated from prior waves)
**Vehicle:** Upstream drbothen/vsdd-factory (DF-VALIDATION-001 required before filing)

Story-writer and implementer agents occasionally leave stale `v2.0` / `v2.1`
inline version markers in story files after spec evolution, rather than
updating them at each spec-route remediation step. Caught during PR reviews;
requires manual cleanup that could be automated via a lint check on
`## Story v<N.M>` header vs frontmatter `version:` field.

---

## PG-W84-002 — sub-agent message-routing breakage

**Class:** Agent message-routing protocol / relay discipline
**Caught by:** STORY-147 delivery (D-481); caused security-review.md artifact backfill (commit f2b5dcfe)
**Severity:** MEDIUM (caused missing artifact; required backfill commit)
**Vehicle:** Upstream drbothen/vsdd-factory (DF-VALIDATION-001 required before filing)

Sub-agent message routing broke during STORY-147 delivery — a sub-agent's
result did not reach the orchestrator directly, requiring relay-through-
orchestrator as a workaround. The breakage also caused the security-review.md
artifact to be missing from the initial delivery commit, requiring a separate
backfill commit (f2b5dcfe). Pattern: sub-agents must relay all results through
the orchestrator; direct cross-agent messaging is not reliable.

---

## PG-W84-003 — burst-log template understatement

**Class:** Template accuracy / documentation hygiene
**Caught by:** STORY-147 delivery post-review
**Severity:** LOW (documentation only)
**Vehicle:** Upstream drbothen/vsdd-factory (DF-VALIDATION-001 required before filing)

The burst-log template understates the number of files touched in complex
multi-file bursts. The Dim-1 "files touched" section template does not
adequately guide agents on counting all affected files (including
session-checkpoints.md and process-gap-ledger.md ride-alongs), leading to
cardinality mismatches caught by the validate-burst-log hook.

---

## PG-W84-004 — STATE.md write-path hook friction

**Class:** Hook cascade / write-path ergonomics
**Caught by:** D-484 burst execution (this burst)
**Severity:** LOW (slows state-manager bursts; no correctness impact)
**Vehicle:** Upstream drbothen/vsdd-factory (DF-VALIDATION-001 required before filing)

The combination of three simultaneous PostToolUse hooks on STATE.md writes
(verify-state-timestamp-refresh, validate-dispatch-advance,
validate-state-pin-freshness) creates a cascade where a single Edit that
passes two hooks can fail the third, requiring the state-manager to compose
all updates as a single Write operation. This is architecturally correct
(single-commit burst protocol) but the error messages do not clearly indicate
which of the three hooks blocked and why. Suggestion: unified hook report
with all failures in a single message rather than blocking on the first.

---

## PG-W84-005 — validate-pr-review-posted hook false-positive for self-authored PRs

**Class:** Hook false-positive / PR review protocol gap
**Caught by:** STORY-166 delivery (D-482, PR #426)
**Severity:** MEDIUM (hook incorrectly blocks valid review workflow)
**Vehicle:** Upstream drbothen/vsdd-factory (DF-VALIDATION-001 required before filing)

The `validate-pr-review-posted` hook fires a blocking error when the PR author
and the reviewer are the same agent (self-authored PRs). For self-authored PRs,
the correct review artifact is a COMMENTED review event (not APPROVE) plus an
explicit `pr-review.md` artifact. The hook does not distinguish between
"no review posted" and "self-authored PR with COMMENTED review event + artifact
= review of record". This caused a spurious block during STORY-166 delivery
that required manual escalation to the orchestrator.

---

## PG-W84-006 — pr-manager-completion-guard pressured step-9 fabrication

**Class:** Agent guard / completion discipline
**Caught by:** STORY-166 delivery (D-482)
**Severity:** HIGH (agent correctly refused; no incorrect artifact created, but pressure was inappropriate)
**Vehicle:** Upstream drbothen/vsdd-factory (DF-VALIDATION-001 required before filing)

The pr-manager-completion-guard hook applied pressure to the pr-manager agent
to record step-9 (merge confirmation) as complete before the PR was actually
merged. The agent correctly refused to fabricate the merge completion. The
guard should not apply this pressure until the merge has been confirmed via
`gh pr view --json mergeStateStatus` or equivalent. The pressure pattern
creates a risk of fabricated completion artifacts if a less-careful agent
complies.

---

## PG-W84-007 — governance-doc CI examples unvalidated against branch topology

**Class:** Documentation accuracy / CI example validation
**Caught by:** STORY-166 Step-4.5 adversarial pass 7 (F-S166P7-001)
**Severity:** MEDIUM (CI-guard grep exits 2 on missing path — false-green even when leaks are present)
**Vehicle:** Product-local (`.factory/maintenance/demo-evidence-scrub-gate.md`)
**Status:** FIXED pre-merge (commit eef569c9, STORY-166 feature branch)

The `demo-evidence-scrub-gate.md` CI-guard example used a `grep` invocation
that exits 2 when `.factory/` is absent (e.g., on CI where the factory-
artifacts worktree is not mounted), producing a false-green exit even when
leaks ARE present in the diff. The adversary caught this as F-S166P7-001
(HIGH severity). Fixed by updating the example to use `|| true` guard plus
explicit exit-code documentation. Lesson: governance-doc CI examples must be
execution-verified against the actual branch topology (no `.factory/` on CI
unless explicitly fetched).

---

## PG-W84-008 — PR-description commit-count drift (R-426-001)

**Class:** PR description accuracy / cosmetic
**Caught by:** Post-merge review of PR #426 description (STORY-166)
**Severity:** LOW (cosmetic; no functional impact)
**Vehicle:** Upstream drbothen/vsdd-factory (DF-VALIDATION-001 required before filing)

PR #426 description claimed 10 commits but the actual squash base contained
11. The pr-manager composed the description before the final fixup commit was
added, and did not re-count. Suggestion: pr-manager should re-count commits
immediately before posting the PR description, or include a note that commit
counts are approximate pre-squash.

---

## PG-W84-009 — AC cites nonexistent mechanism + wrong gate locus (spec-drift class)

**Class:** Spec-drift / story-writer accuracy — "AC cites nonexistent mechanism"
**Caught by:** STORY-176 Step-2 stub-architect pre-condition probe (D-484, 2026-07-20)
**Severity:** HIGH (AC substantially invalid as written; would have caused Red Gate failure for wrong reason)
**Research validation:** `.factory/planning/story-176-ac001-validation.md` — verdict HIGH confidence INVALID; motivation PG-GATE-VOCAB-BLINDSPOT VALID; product-local, no upstream filing
**Vehicle:** Product-local (STORY-176 story file)
**Status:** REMEDIATED — STORY-176 v2.2→v2.3 spec-route remediation complete (D-484)

STORY-176 v1.0/v2.0/v2.2 AC-176-001 was substantially invalid as written:

1. **Wrong gate locus:** AC cited `ci.yml` as the gating surface (e.g., adding a
   `# green-doc-tense-gate: allow` comment to bypass the check). The actual
   gate lives in `bin/check-green-doc-tense` (a standalone Python script);
   `ci.yml` merely invokes it.

2. **Fabricated allowlist mechanism:** AC described a `# green-doc-tense-gate: allow`
   inline comment allowlist that does not exist in `bin/check-green-doc-tense`.
   The script has no such mechanism. This was a hallucination by the story-writer
   agent, not a description of any real code path.

3. **Inverted CHANGELOG claim:** AC stated CHANGELOG entry was NOT REQUIRED; the
   correct obligation (per CLAUDE.md AC-158-001) is REQUIRED for any PR that
   touches `src/`, `Cargo.toml`, or `bin/`.

**Remediation (v2.3):** Locus corrected to `bin/check-green-doc-tense` +
`bin/test_check_green_doc_tense.py`; four phrase-level zero-FP patterns
(`skeleton\s+compiles?`, `stub\s+compiles?`, `red\s+gate`, `todo!\s*\(`) added;
fabricated allowlist claim deleted; CHANGELOG obligation corrected to REQUIRED;
input-hash re-baselined 41176f4→7f8ff02 (canonical tool).

**Root cause:** Story-writer agent inferred gate behavior from the AC description
pattern (allowlist-style gating) without verifying against the actual
`bin/check-green-doc-tense` source. The stub-architect's Step-2 pre-condition
probe (reading the actual script) caught the discrepancy before Red Gate.

**Lesson:** Story-writer ACs for script-gated behavior MUST cite the actual script
source path and describe only mechanisms that exist in the script. The
stub-architect's pre-condition probe is the correct catch point, but ideally
the story-writer validation pass (DF-VALIDATION-001 research-agent) would
catch fabricated mechanism references before Step 2.

---
