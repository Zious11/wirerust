---
document_type: process-gap-ledger
cycle: wave-75
created: 2026-07-13
status: closed
closed_date: 2026-07-13
decision: D-435
owner: state-manager
policy: S-7.02
---

# Wave-75 Process-Gap Ledger

Each item below is a research-validated finding from wave-75 STORY-165 adversarial
convergence, proposed for codification as S-7.02 follow-up items at wave close.
All three findings were validated by DF-VALIDATION-001 research report
`.factory/research/pg-validation-wave-75.md` (2026-07-13).

Items were marked **PENDING wave-gate disposition** — all dispositioned at wave-75 gate
close (D-435, 2026-07-13). See STORY-166 (wave-TBD, E-11, 5 pts, v1.0, hash 8e244ad) for
full codification, and STATE.md Drift Items for justified-deferral rows.

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
**Status:** DISPOSITIONED → STORY-166 AC-166-001 (wave-75 gate close D-435, 2026-07-13).

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
**Status:** DISPOSITIONED → STORY-166 AC-166-002 (wave-75 gate close D-435, 2026-07-13). Canonical G-form IDs (`F-W<NN>G-P<n>-<seq>`) used throughout wave-75 gate artifacts as dogfood-fix ahead of policy codification.

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
**Status:** DISPOSITIONED — factual correction applied to `cycles/wave-74/wave-gate/gate-summary.md:43` in wave-75 gate-close burst (D-435, 2026-07-13). Bracketed correction note added citing PG-W75-GATE-SUMMARY-VERSION-ATTRIBUTION + research evidence (pg-validation-wave-75.md Finding 3). No separate codification story beyond gate-summary correction + parent PG-W75-FINDING-ID-DUAL-SCHEME (STORY-166 AC-166-002).

---

## Gate Observation: OBS-W75-W6 — Mid-Gate Streak Persistence Gap

**Class:** gate-observation
**Surfaced:** wave-75 gate pass W6 (CLEAN pass; streak #2).
**Description:** The wave gate `findings.md` log was updated at findings-only passes but CLEAN
passes were not incrementally recorded. A reader mid-gate could not determine whether the
streak had persisted through intervening CLEAN passes without reading multiple separate
artifacts. This is distinct from the F-W75G-P4-001 fix (which corrected a blanket-provenance
claim); this observation is about gate-progress logging completeness.
**Recommended codification:** Extend wave-gate `findings.md` (and per-story findings logs)
to record a row for every pass verdict, not only finding passes — one CLEAN row per clean pass
with the running streak count. Makes mid-gate progress legible without separate state audits.
**Status:** DISPOSITIONED → STORY-166 AC-166-004 (wave-75 gate close D-435, 2026-07-13).

---

## Gate Observation: OBS-W75-W7 — Demo-Evidence Scrub Scope Gap

**Class:** gate-observation
**Surfaced:** wave-75 gate pass W7 (CLEAN pass; streak #3).
**Description:** The demo-evidence-scrub-gate.md mandate covers `demo-evidence/` paths under
`.factory/cycles/<cycle>/demo-evidence/`. New demo captures written to `.factory/demo-evidence/`
(a different root path introduced for some E-11 governance stories) were not explicitly listed
in the scrub scope. The scrub gate should be extended to enumerate `.factory/demo-evidence/`
as a second root alongside `.factory/cycles/<cycle>/demo-evidence/`.
**Recommended codification:** Amend demo-evidence-scrub-gate.md scope section to add
`.factory/demo-evidence/` as an explicit scrub target root.
**Status:** DISPOSITIONED → STORY-166 AC-166-003 (wave-75 gate close D-435, 2026-07-13).

---

## Ledger Note: F-W75G-P3-002 Redundancy Resolved

**Wave-75 gate pass W3** filed a finding F-W75G-P3-002 against the process-gap-ledger that
overlapped with PG-W75-FINDING-ID-DUAL-SCHEME already present in this ledger. The finding
was research-adjudicated as redundant: pg-validation-wave-75.md Finding 2 fully covers the
dual-scheme issue; F-W75G-P3-002 introduced no additional evidence or scope. The ledger item
PG-W75-FINDING-ID-DUAL-SCHEME above is the authoritative record; F-W75G-P3-002 is closed as
ledger-redundant. No separate action beyond STORY-166 AC-166-002 codification.

---

*Wave-75 S-7.02 ledger created 2026-07-13 (D-434 burst). All items DISPOSITIONED at wave-75 gate close D-435, 2026-07-13.*
