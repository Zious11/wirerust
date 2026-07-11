---
document_type: process-gap-ledger
cycle: wave-73
created: 2026-07-11
status: closed
owner: state-manager
policy: S-7.02
---

# Wave-73 Process-Gap Ledger

Each item below requires a follow-up story or justified deferral before wave-73 is
CLOSED (S-7.02 / lessons-codification). Items without resolution at wave close must be
carried as explicit deferred findings into the next cycle's session-checkpoints.

Wave-73 is CLOSED (D-428, 2026-07-11). All four PG items addressed by STORY-164
(drafted at v3.43, wave-TBD). Code-review deferred items tracked below.

---

## PG-W73-STATUS-VOCAB — STORY-INDEX Status-Vocabulary Legend Missing

**Class:** process-gap
**Surfaced:** wave-level adversary Pass 3 (F-W73G-P3-001 HIGH + 38-file corpus sweep)
**Description:** No canonicalized status-vocabulary legend exists in STORY-INDEX. This
allowed corpus-wide drift across 38 story files: status values such as `completed`,
`delivered`, `merged`, and informal variants accumulated without a reference set.
The P3 adversary triggered a 38-file sweep to normalize status across all affected
stories; without a legend, the same drift will recur.
**Codification:** STORY-164 AC-164-001 (PG-W73-STATUS-VOCAB) — adds STORY-INDEX
status-vocabulary legend defining the canonical set of allowed status values.
**Status:** CODIFIED → STORY-164 (wave-TBD, draft). Delivery required to fully close.

---

## PG-W73-CITATION-VALIDATOR — Mechanical Citation Preflight Validator Absent

**Class:** process-gap
**Surfaced:** STORY-163 per-story adversary Pass 1 (F-S163P1-001 CRITICAL fabricated
citations) + STORY-163 delivery meta-observation
**Description:** The citation-mandate story (STORY-163, AC-163-001) was delivered
with fabricated anchor references (line numbers citing non-existent content) in its
authoring-evidence.md. No mechanical preflight tool existed to validate that
cited `file:line` anchors resolve to actual matching content before adversary
dispatch. The adversary caught the fabrication at P1 CRITICAL; without adversarial
review, the fabricated anchors would have persisted silently.

Root cause: citation validation was entirely adversary-dependent, not
tool-enforced. The citation-mandate story that introduced the obligation did not
carry a self-validation mechanism.
**Codification:** STORY-164 AC-164-002 (PG-W73-CITATION-VALIDATOR) — adds
`bin/validate-citations` preflight script that resolves anchor references to actual
file:line content before adversary dispatch.
**Status:** CODIFIED → STORY-164 (wave-TBD, draft). Delivery required to fully close.

---

## PG-W73-CHANGELOG-GATE-CONTENT — Changelog-Gate Is Presence-Only

**Class:** process-gap
**Surfaced:** STORY-162 per-story adversary Pass 5 (process-gap observation)
**Description:** The `changelog-gate` CI job (AC-158-001) checks only that an
`[Unreleased]` section EXISTS in CHANGELOG.md — it does not assert that the entry
contains actual behavioral content. A one-line placeholder entry (e.g., `- placeholder`)
satisfies the gate. This is a pre-existing design limitation of the presence-only
gate; the observation here is that the gap is now explicitly named and tracked.

The P5 observation at STORY-162 delivery identified this as an improvement
opportunity rather than a blocker. The changelog entry for STORY-162 (PR #395)
was substantive and correct; the gap is about what the CI gate enforces, not about
any specific instance of misbehavior.
**Codification:** STORY-164 AC-164-003 (PG-W73-CHANGELOG-GATE-CONTENT) — adds a
changelog-gate content assertion (minimum substantive lines under `[Unreleased]`).
**Status:** CODIFIED → STORY-164 (wave-TBD, draft). Delivery required to fully close.

---

## CLAUDE.md Row Missing for docs-writer-dispatch-guidance.md

**Class:** process-gap (documentation omission)
**Surfaced:** wave-73 consistency audit (gate close review)
**Description:** STORY-163 AC-163-001 created `.factory/maintenance/docs-writer-dispatch-guidance.md`
as the authoritative citation-mandate and docs-dispatch routing guidance. However,
the CLAUDE.md `## Project References` table was not updated to include a row
pointing to this new guidance document. Future contributors and agents will not
discover this guidance unless they happen to explore `.factory/maintenance/`
directly.
**Codification:** STORY-164 AC-164-004 — adds the Project References row for
`docs-writer-dispatch-guidance.md` to CLAUDE.md.
**Status:** CODIFIED → STORY-164 (wave-TBD, draft). Delivery required to fully close.

---

## F-W73G-CR-001 — AC-158-005 Regression Guard Non-Hermetic After Refactor

**Class:** code-review deferred (MINOR)
**Surfaced:** integration-gate code review (wave-73 gate)
**Description:** `bin/test_check_green_doc_tense.py` AC-158-005 test patches
`_collect_rust_files` but NOT `_find_repo_root`. After the STORY-162 refactor,
`main()` calls `_find_repo_root` first; if root discovery fails, exit-2 is returned,
causing the test to pass for the wrong reason (exit-2 ≠ 0 passes the `exit_code != 0`
assertion, masking the zero-file guard). AC-162-003 provides hermetic coverage
(patches both helpers), mitigating the risk in practice.
**Disposition:** DEFERRED (human-ratified 2026-07-11, next maintenance sweep).
**Route:** Next maintenance sweep.

---

## F-W73G-CR-002 — Docstring "6 Levels" Ambiguity vs. `range(6)`

**Class:** code-review deferred (NIT)
**Surfaced:** integration-gate code review (wave-73 gate)
**Description:** `bin/check-green-doc-tense` _find_repo_root docstring reads "Walk
upward up to 6 levels from *start*" and inline comment reads "at most 6 levels up",
but `range(6)` checks start + 5 ancestors = 6 candidates total. If "6 levels" means
6 ancestors above start, the loop would need `range(7)`. Pre-existing off-by-one in
the comment text; behavior was inherited from pre-refactor code.
**Disposition:** DEFERRED (human-ratified 2026-07-11, next maintenance sweep).
**Route:** Next maintenance sweep (documentation cleanup batch).

---

## F-W73G-CR-003 — Test (c) Uses `str.startswith` Instead of `Path.is_relative_to()`

**Class:** code-review deferred (NIT)
**Surfaced:** integration-gate code review (wave-73 gate)
**Description:** `bin/test_check_green_doc_tense.py` AC-162-004 test (c) uses
`not str(_result_c).startswith(str(_root_c))` for path containment. String-prefix
matching is not equivalent to filesystem hierarchy containment (e.g., `/tmp/foobar`
incorrectly matches `/tmp/foo`). `Path.is_relative_to()` (Python 3.9+; project
targets 3.10+) performs correct hierarchy-aware containment.
**Disposition:** DEFERRED (human-ratified 2026-07-11, next maintenance sweep).
**Route:** Next maintenance sweep.

---

*Wave-73 gate deferred items appended 2026-07-11 (D-428 burst).*
