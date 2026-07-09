---
document_type: process-gap-ledger
cycle: wave-72
created: 2026-07-09
status: open
owner: state-manager
policy: S-7.02
---

# Wave-72 Process-Gap Ledger

Each item below requires a follow-up story or justified deferral before wave-72 is
CLOSED (S-7.02 / lessons-codification). Items without resolution at wave close must be
carried as explicit deferred findings into the next cycle's session-checkpoints.

---

## F-W72-P1-002 — Advisory-pointer idiom not in sibling-sweep targets

**Class:** process-gap  
**Surfaced:** adversary Pass 1  
**Description:** The advisory-pointer idiom (citing a sibling story by ID when the
finding is out of scope for the current story but relevant to a sibling) is not
included in the sibling-sweep target checklist used by story-writer and adversary
agents. Adversaries have re-derived this requirement independently on multiple waves.  
**Required resolution:** Either codify the idiom in the relevant story-writer or
adversary checklist artifact, or file a justified deferral with a target wave.

---

## F-W72-P1-005 — E-11 empty-BC convention vs. tooling interaction

**Class:** process-gap  
**Surfaced:** adversary Pass 1  
**Description:** E-11 stories use `inputs: []` (empty BC list) because they have no
spec inputs. The canonical input-hash tool correctly produces `d41d8cd` for these, but
the bash hook's divergent algorithm (PG-HASH-HOOK-DIVERGENCE) may emit false-positive
drift warnings on E-11 stories in certain tooling contexts. The interaction between the
E-11 convention and the hook divergence is not explicitly documented in the E-11 story
template.  
**Required resolution:** Add a note to the E-11 story template (or the hook-divergence
advisory guidance) clarifying that E-11 `inputs: []` stories will always produce
`d41d8cd` from the canonical tool and any hook discrepancy is a known false positive.

---

## F-W72-P2-006 — Scheduling body/frontmatter drift

**Class:** process-gap  
**Surfaced:** adversary Pass 2  
**Description:** Wave scheduling metadata (story ordering, dependency edges, wave
assignments) can drift between frontmatter fields and the scheduling body prose within
the same story file, because story amendments update one but not the other. No lint
rule or adversary axis explicitly checks for intra-story frontmatter/body scheduling
consistency.  
**Required resolution:** Add an adversary axis or story-writer lint step that checks
wave/dependency frontmatter fields against the scheduling body section of each story.
Or file a justified deferral identifying this as an acceptable known gap.

---

## F-W72-P3-001 — Source citation-style drift class

**Class:** process-gap  
**Surfaced:** adversary Pass 3  
**Description:** Factory documents (stories, BCs, VP shards) use inconsistent citation
styles when referencing external sources — some use inline URLs, some use footnote
style, some use bare identifiers. This inconsistency accumulates across waves and makes
cross-document traceability harder. No citation-style convention is codified in the
story or BC templates.  
**Required resolution:** Either codify a citation-style convention in the relevant
templates (story-writer, BC authoring guide), or file a justified deferral accepting
the current style diversity as non-blocking.

---

## F-W72-P4-001 — Bootstrap own-PR gate axis unrepresented in DF-* policies

**Class:** process-gap  
**Surfaced:** adversary Pass 4  
**Description:** The "bootstrap own-PR gate" axis (a story that codifies tooling must
itself be gated by the tooling it introduces, where feasible) is not represented as a
named policy in `.factory/policies.yaml`. This axis was surfaced as an adversarial
concern during wave-72 but has no durable machine-enforceable form.  
**Required resolution:** Add a DF-* policy entry capturing the bootstrap own-PR gate
requirement, or file a justified deferral explaining why this is handled adequately by
existing adversary pass coverage.

---

## PG-W72-CODIFICATION-FIDELITY / F-W72-P6-001 — Codification stories must be audited against their motivating defect

**Class:** process-gap (primary — highest priority in this ledger)  
**Surfaced:** adversary Pass 6  
**Description:** Codification stories (E-11 type) are intended to durably fix a
diagnosed process gap. Pass 6 found that a codification story's acceptance criteria
had drifted from the specific defect it was meant to close — the ACs addressed a
related but narrower concern, leaving the original gap partially open. The adversary
pass surfaced this as F-W72-P6-001. Root cause: no gate requires the story-writer to
explicitly cross-reference the motivating defect ID (finding ID or process-gap ID) in
each AC and confirm coverage.  
**Required resolution (mandatory before wave-72 CLOSED):** Add a story-writer rule or
adversary axis requiring that every E-11 codification story's ACs explicitly cite the
motivating defect ID and that at least one AC directly closes the diagnosed root cause.
This is the highest-priority item in this ledger and must not be carried as an open
deferral without an explicit justification reviewed at wave close.

---

## F-W72G-CR-004 — ADR-012 Decision 3a/3c Duplication

**Class:** code-review deferred (NIT→MINOR)
**Surfaced:** integration-gate code review
**Description:** `docs/adr/0012.md` Decisions 3a and 3c duplicate each other in
intent — both address the same "unclassified port" scenario with near-identical
wording, creating reader confusion about which decision is authoritative. One should
be primary; the other should reference it.
**Route:** Next maintenance sweep (documentation cleanup batch).

---

## F-W72G-CR-006 — TC2 Fixture Duplicate Assertion

**Class:** code-review deferred (NIT)
**Surfaced:** integration-gate code review
**Description:** `bin/test_lint_cycle_artifact.py` TC2 contains a duplicate assertion
(`assert result.returncode == 0` appears twice consecutively) and a comment that
describes the wrong test case (copy-paste from TC1). Cosmetic but reduces
test-suite readability.
**Route:** Next maintenance sweep.

---

## F-W72G-CR-007 — _PARSE_ERRORS Tuple Inside main()

**Class:** code-review deferred (NIT)
**Surfaced:** integration-gate code review
**Description:** `bin/lint-cycle-artifact` defines `_PARSE_ERRORS` tuple inside
`main()` rather than at module level. Tuple literals defined inside function bodies
are re-constructed on each call; at module level they are constructed once and reused.
**Route:** Next maintenance sweep.

---

## F-W72G-CR-008 — SEC-001 Guard Idiom Duplication

**Class:** code-review deferred (NIT)
**Surfaced:** integration-gate code review
**Description:** `bin/lint-cycle-artifact` SEC-001 path-guard idiom
(`if not path.resolve().is_relative_to(root)`) appears at two sites without a shared
helper or cross-reference comment. Future maintainers may update one site without the
other.
**Route:** Next maintenance sweep.

---

## F-W72G-CR-009 — Redundant contains-key Asserts in TC7

**Class:** code-review deferred (NIT)
**Surfaced:** integration-gate code review
**Description:** `bin/test_lint_cycle_artifact.py` TC7 asserts
`expected_key in story_bcs_set` after the set-equality check that already implies this.
The redundant contains-key asserts add verbosity without additional coverage.
**Route:** Next maintenance sweep.

---

## F-W72G-P3-OBS-001 — HS-082 Terminal-Case Example Specificity

**Class:** advisory observation
**Surfaced:** integration-gate adversary Pass 3
**Description:** HS-082 terminal-case example scenario description could benefit from
additional specificity to distinguish the terminal-case condition from the adjacent
non-terminal variant.
**Route:** Next maintenance sweep (holdout-scenario quality batch).

---

## F-W72G-P3-OBS-002 — STORY-INDEX BC-Tally 337 vs BC-INDEX v2.22 Count 347

**Class:** count-drift (MEDIUM, pre-existing)
**Surfaced:** integration-gate adversary Pass 3 (same class as EPICS-TOTAL-BCS-DRIFT-001)
**Description:** STORY-INDEX epics.md v2.1 `total_bcs: 337` disagrees with BC-INDEX
v2.22 active-BC count of 347. This is the same pre-existing drift as
EPICS-TOTAL-BCS-DRIFT-001 (pending intent, confirmed deferred Route C 2026-07-08).
The counts have diverged by a further 10 since the last confirmed batch
(wave-72 BCs add BC-2.11.001 v1.9 amendment).
**Route:** Next spec-coherence sweep (batch with EPICS-TOTAL-BCS-DRIFT-001).

---

## F-W72G-P3-OBS-003 — CHANGELOG Routing Note Strip-Before-Release

**Class:** advisory observation (LOW)
**Surfaced:** integration-gate adversary Pass 3
**Description:** The CHANGELOG contains a routing note about BREAKING change placement
that is advisory for developers but should be stripped before release publication.
No automation strips these routing notes when preparing the release section.
**Route:** Next maintenance sweep (CHANGELOG hygiene batch).

---

## F-W72G-P3-OBS-004 — Sentinel-Asymmetry Docstring

**Class:** advisory observation (LOW)
**Surfaced:** integration-gate adversary Pass 3
**Description:** The dispatcher sentinel value has a doc-comment that does not name
its paired invariant, creating asymmetric documentation coverage relative to adjacent
sentinel values that do name their invariants.
**Route:** Next maintenance sweep (documentation cleanup batch).

---

*Wave-72 gate deferred items appended 2026-07-09 (D-415 burst). Resolves F-W72G-P3-004
(adversary P3 finding: deferred-items ledger missing code-review and P3/P4 observations).*
