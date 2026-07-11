# STORY-164 v1.1 — Pre-Delivery Citation Validation

**Date:** 2026-07-11
**Validator:** df-validation (pre-plan-gate, wave-74 candidate)
**Discipline:** DF-VALIDATION-001 + STORY-163 P1 lesson (validate citations BEFORE the plan gate)
**Story:** `.factory/stories/STORY-164.md` v1.1, 5 ACs, input-hash `3922d6c`
**Verdict:** **FIX-FIRST** — one material citation defect in AC-164-002 evidence prose (meta-ironic: it is the citation-precision story itself). All ACs are otherwise well-founded.

---

## Per-AC verdict table

| Item | Verdict | Evidence |
|------|---------|----------|
| AC-164-001 (PG-W73-STATUS-VOCAB) | **VERIFIED** | `STORY-INDEX.md:14` v3.42 comment matches quoted evidence verbatim; `## Index Table` heading exists at `STORY-INDEX.md:122`; no pre-existing legend |
| AC-164-002 finding (PG-W73-CITATION-VALIDATOR) | **VERIFIED** | `STORY-163/adversary-convergence-state.json` pass 1 + carried_findings confirm F-S163P1-001 CRITICAL and bin/validate-citations recommendation |
| AC-164-002 evidence prose (Background + Prev-Story-Intelligence) | **FABRICATED / IMPRECISE** | Story names `pr-manager-merge-auth-guidance.md:332-333` / "file is only 111 lines". Ground truth (`authoring-evidence.md:113-114`): fabricated anchor was `pr-review.md:332-333`; **pr-review.md** is 111 lines. `pr-manager-merge-auth-guidance.md` is **210 lines**, not 111 |
| AC-164-003 (PG-W73-CHANGELOG-GATE-CONTENT) | **VERIFIED** | `ci.yml:506-509` is presence-only (`grep -q '^CHANGELOG\.md$'` → exit 0), no content assertion; `STORY-162/adversary-convergence-state.json` pass 5 + carried_findings confirm the gap |
| AC-164-004 (CLAUDE.md row) | **VERIFIED** | `CLAUDE.md:237` Project References table; `:248` has the `pr-manager-merge-auth-guidance.md` peer row; no `docs-writer-dispatch-guidance.md` row present. Placement claim valid |
| AC-164-005 (PG-W72-BREAKING-HOLDOUT-SWEEP) | **VERIFIED** (minor note) | `wave-72/lessons.md:46-70` Lesson 2: STORY-160 BREAKING enum casing PascalCase→lowercase/snake_case + schema_version; 13 stale holdouts; HS-021/024/032/033/034/035/050/054/059/064/065/074/075 (count = 13, exact match); repaired by product-owner at gate. **Note:** the tag string `PG-W72-BREAKING-HOLDOUT-SWEEP` is NOT in wave-72/lessons.md (labeled "candidate-codification"); it was minted at maint-2026-07-11. Story Notes (lines 533-537) correctly acknowledge this, so acceptable |
| Frontmatter inputs (6 files) | **VERIFIED** | All 6 input paths exist on disk |
| input-hash `3922d6c` | **VERIFIED** | `bin/compute-input-hash .factory/stories/STORY-164.md` → `3922d6c` (exact match) |
| Cross-refs: docs-writer §4, PG-RA-P3-ARP-REC006-INVERSION-001 | **VERIFIED** | `docs-writer-dispatch-guidance.md:99` "Section 4 — Verification Template for Orchestrator Dispatches" (exact); PG-RA tag present (3×). AC-163-001 authorship of the guidance file confirmed |

---

## The material defect (AC-164-002 evidence prose)

The AC-164-002 requirement (create `bin/validate-citations`) and its underlying finding
(F-S163P1-001 CRITICAL, fabricated citations) are **real and fully verified**. The defect is
in the *illustrative evidence* the story uses to motivate the AC.

**Story claims (two loci):**
- Background, lines 100-105: "the adversary found that `authoring-evidence.md` ... cited
  `pr-manager-merge-auth-guidance.md:332-333` as anchor locations, but the file is only 111 lines."
- Previous Story Intelligence, lines 452-458: "three `pr-manager-merge-auth-guidance.md:332-333`
  anchors were cited when the file is only 111 lines."

**Ground truth** (`.factory/cycles/wave-73/STORY-163/authoring-evidence.md:113-114`):
> "three AC-163-002 citations originally pointed to `pr-review.md:332-333` (nonexistent —
> file is 111 lines); re-anchored to `lessons.md` after verification."

So:
- The fabricated anchor was in **`pr-review.md`** (`.factory/code-delivery/maint-2026-07-09/pr-review.md`, 110-111 lines), **not** `pr-manager-merge-auth-guidance.md`.
- `pr-manager-merge-auth-guidance.md` is **210 lines** (`wc -l`), so the "only 111 lines" claim about that file is factually false.

**Root of the error:** the upstream `adversary-convergence-state.json` P1 note itself is
imprecise — it says "non-existent line numbers in pr-manager-merge-auth-guidance.md". The
story's *Evidence block* (lines 113-118) faithfully quotes that JSON (so the quote itself is
an accurate quotation). But the story's *prose* then invents the specific `:332-333` anchor
and "111 lines" figure — which are real values belonging to `pr-review.md` — and welds them
onto the wrong file name inherited from the JSON.

**Why this is FIX-FIRST, not a nit:** STORY-164 exists *because* STORY-163 shipped a
fabricated citation that was caught only at adversarial review. Shipping STORY-164 with a
demonstrably false citation detail in its own motivating evidence would repeat the exact P1
failure the story is designed to prevent — and it is precisely the class of error
`bin/validate-citations` would flag (`pr-manager-merge-auth-guidance.md:332-333` →
LINE OUT OF RANGE, file has 210 lines). Correct before the plan gate.

**Suggested correction:** in both Background and Previous Story Intelligence, change
`pr-manager-merge-auth-guidance.md:332-333` → `pr-review.md:332-333`
(`.factory/code-delivery/maint-2026-07-09/pr-review.md`, ~111 lines). Optionally note that
the `adversary-convergence-state.json` P1 note has the same misattribution and should be
read against `authoring-evidence.md` as authoritative.

---

## Overall recommendation

**FIX-FIRST.** Correct the AC-164-002 file misattribution (two loci) before the wave-74 plan
gate. Everything else — all five AC findings, the CI-gate presence-only claim, the wave-72
holdout evidence, the frontmatter, and the input-hash — is VERIFIED. Once the `pr-review.md`
correction is applied, STORY-164 is GO.
