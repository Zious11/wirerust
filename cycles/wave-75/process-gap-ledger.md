---
document_type: process-gap-ledger
cycle: wave-75
created: 2026-07-13
status: open
owner: state-manager
policy: S-7.02
---

# Wave-75 Process-Gap Ledger

Each item below is a research-validated finding from wave-75 STORY-165 adversarial
convergence, proposed for codification as S-7.02 follow-up items at wave close.
All three findings were validated by DF-VALIDATION-001 research report
`.factory/research/pg-validation-wave-75.md` (2026-07-13).

Items are marked **PENDING wave-gate disposition** — resolution (story codification or
justified deferral) required before wave-75 is CLOSED (S-7.02).

---

## PG-W75-VALIDATE-CITATIONS-SYMBOL-GAP — Citation Validator Checks Line-in-Bounds, Not Symbol-at-Line

**Class:** process-gap
**Surfaced:** wave-75 STORY-165 per-story adversary Pass 1 (F-S165P1-001 HIGH fabricated
test name at in-bounds line). Research validated (VALID) by pg-validation-wave-75.md Finding 1.
**Description:** `bin/validate-citations` validates that cited `file:line` references
resolve to a real line within the file's range, but performs no assertion on the content of
that line. A fabricated symbol name (e.g., a test function name that does not exist on the
cited line) passes preflight silently. The failure class is `SYMBOL NOT FOUND AT LINE`, not
`LINE OUT OF RANGE`. F-S165P1-001 instantiated exactly this: a fabricated test function name
was cited at an in-bounds line and passed the existing preflight validation.
**Recommended codification:** Extend `bin/validate-citations` to accept an optional third
field `path:line:anchor`. When anchor is present, read that line and assert it matches the
anchor pattern (e.g., `def test_<anchor>`, `fn <anchor>`, `class <anchor>`). New failure class:
`SYMBOL NOT AT LINE: path:line (expected anchor '<x>')`. Backward-compatible: bare `path:line`
citations remain unchanged. Keep stdlib-only (no ctags / external binary deps).
See pg-validation-wave-75.md §Finding 1 "Recommended minimal design" for full spec.
**Status:** PENDING wave-gate disposition (codification story or justified deferral).

---

## PG-W75-FINDING-ID-DUAL-SCHEME — Wave-74 Artifacts Use Colliding Finding-ID Schemes

**Class:** process-gap
**Surfaced:** wave-75 STORY-165 per-story adversary Pass 4 (F-S165P4-001 HIGH: finding-ID
collision; fabricated ID `F-W74P8-001` used G-less form + wrong pass number). Research
validated (VALID) by pg-validation-wave-75.md Finding 2.
**Description:** Within wave-74, two different ID forms exist for wave-gate findings:
the canonical `F-W<NN>G-P<n>-<seq>` (G-form, used in gate-summary.md and lessons.md)
and the G-less `F-W<NN>P<n>-<seq>` (used in STORY-164 changelog). Both forms are in
live use repo-wide (also present in wave-70 artifacts and policies.yaml). With two forms
sharing the same pass numbers but pointing to different findings, authors reach for the
non-canonical form and misnumber passes — exactly what happened with F-S165P4-001
(fabricated ID `F-W74P8-001` corrected to canonical `F-W74G-P3-001`).

**Remediation-scope refinement from research:** The collision axis is G-less `F-W<NN>P<n>`
vs canonical `F-W<NN>G-P<n>` — not "per-story vs wave." The per-story form `F-S<NNN>P<n>`
is already unambiguous (begins `F-S`).
**Recommended codification:** Add a naming-convention policy to `.factory/policies.yaml` with
three clauses: (1) per-story findings MUST use `F-S<NNN>P<n>-<seq>`; (2) wave-gate findings
MUST use `F-W<NN>G-P<n>-<seq>` (G-form); the G-less `F-W<NN>P<n>` form is deprecated for
wave-gate findings; (3) story-artifact cross-references (e.g., changelog rows) MUST cite the
canonical wave-gate ID. Optional: extend `bin/lint-cycle-artifact` to flag the regex
`F-W[0-9]+P[0-9]` (G-less) as malformed.
See pg-validation-wave-75.md §Finding 2 "Recommended minimal codification" for full spec.
**Status:** PENDING wave-gate disposition (codification story or justified deferral).

---

## PG-W75-GATE-SUMMARY-VERSION-ATTRIBUTION — gate-summary.md:43 Attributes Wrong STORY-INDEX Version to W6 Fix

**Class:** process-gap (documentation error; concrete instance of PG-W75-FINDING-ID-DUAL-SCHEME)
**Surfaced:** wave-75 STORY-165 per-story adversary convergence (Finding 3 added per
team-lead follow-up). Research validated (VALID) by pg-validation-wave-75.md Finding 3.
**Description:** `cycles/wave-74/wave-gate/gate-summary.md:43` states that the W6 finding
produced "STORY-INDEX v3.48 fix applied." However, STORY-INDEX's own changelog records v3.48
as the superseded-row addition (F-W74P3-001, pass 3), not a Delivery-Progress sub-cell fix.
The Delivery-Progress "IN PROGRESS→DELIVERED" edit is at v3.47 (`:18`). The two records
irreconcilably disagree. **gate-summary.md:43 is the artifact in error**; STORY-INDEX
changelog v3.48 is the authoritative source for its own version semantics, corroborated
by three independent loci in the same file.

This is a concrete instance of PG-W75-FINDING-ID-DUAL-SCHEME's root cause: the dual
ID/version-reference system allowed the same W6 pass to be described under two non-matching
referents.
**Recommended codification:** One-line factual correction to `gate-summary.md:43` — update
the version cross-reference from "v3.48" to the STORY-INDEX version that actually carried
the W6 Delivery-Progress sub-cell fix (ground-truth lookup required: scan STORY-INDEX
changelog for the IN PROGRESS→DELIVERED sub-cell correction at wave 74). Finding 3 requires
no separate codification story beyond the gate-summary correction and remediation of
PG-W75-FINDING-ID-DUAL-SCHEME.
See pg-validation-wave-75.md §Finding 3 for full evidence.
**Status:** PENDING wave-gate disposition (one-line factual correction to gate-summary.md:43
+ parent PG-W75-FINDING-ID-DUAL-SCHEME codification).

---

*Wave-75 S-7.02 ledger created 2026-07-13 (D-434 burst). Disposition required at wave-75 gate close.*
