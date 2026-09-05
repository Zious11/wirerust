# Wave-86 Story-Approval Gate — Consistency Audit

Fresh-context `consistency-validator` findings gathered for the human story-approval gate
on STORY-182 (v2.12) and STORY-183 (v2.13), following wave-86 story-level adversarial
convergence (3/3 clean, passes 25/26/27, D-544).

## Scope

Cross-document consistency checks between STORY-182/183 and the surrounding index/registry
surface: STORY-INDEX.md, `.factory/stories/dependency-graph.md`, `.factory/stories/epics.md`,
BC/VP registries, traceability targets, and STATE.md itself.

## Check 1 — STORY-INDEX consistency

**PASS.** STORY-182/183 rows in STORY-INDEX v4.19 agree with the live story files (version
stamps, points, epic, wave, status).

## Check 2 — dependency-graph.md

**FAIL.** `.factory/stories/dependency-graph.md` frontmatter reports `version: "3.9"` — not
v3.10 as claimed by STORY-INDEX (:592) and by STATE.md. The file contains zero mention of
STORY-182, STORY-183, or waves 84/85/86. The file's changelog/modified history ends at v3.9
(2026-07-14, `total_stories: 118`, `total_edges: 137`). This is a pre-existing multi-wave
drift compounded by a version-claim mismatch: STORY-INDEX/STATE assert a v3.10 dep-graph
(138 edges, incl. the STORY-174→STORY-180 edge) that never actually landed in the file.

## Check 3 — epics.md

**FAIL (pre-existing, non-blocking).** `epics.md` is frozen at v2.1 (2026-07-02). The E-11
section lists only 6 stories; STORY-157 through STORY-183 (~17 stories) were never added.

## Check 4 — BC/frontmatter coherence

**PASS.** Both STORY-182 and STORY-183 carry `behavioral_contracts: []` /
`verification_properties: []`. STORY-182's BC-2.19.029/030 references are attributive to
STORY-180's BCs (fixture provenance), not ownership claims. No registry references
STORY-182/183 as a BC/VP source.

## Check 5 — input/traces

**GAP (expected).** STORY-182's `traces_to` target
`.factory/maintenance/fixture-count-gate-entry.md` does not yet exist — this is a forward
reference, created at delivery on the `factory-artifacts` branch per the story's own
annotation. Re-confirm at delivery time; not a defect at story-approval stage.

## Check 6 — inbound references

**FAIL (trivial) — FIXED in this burst.** STATE.md Drift Items rows PG-W84-LOCAL-BATCH,
PG-W85-003, and PG-W85-005 were self-contradicting: they read "Pass-22 REMEDIATED (D-539)
... Pass-23 next" and cited "STORY-183 v2.12", both stale relative to current truth (pass 27
CONVERGED, D-544; STORY-183 v2.13). See STATE.md D-544 burst for the synced rows.

## Check 7 — convention

**GAP — open for human decision.** STORY-182 and STORY-183 both use `level: maintenance` in
frontmatter, while every other E-11 wave-governance story (STORY-157, 158, 159, 161, 162,
163, 164, 165, 166, 176) uses `level: feature`. No defect in story substance — a labeling
convention question for the human story-approval gate to resolve.

## Overall

**ISSUES-FOUND.** Perimeter/records drift (dependency-graph.md v3.9 vs claimed v3.10;
epics.md v2.1 stale) plus one open convention question (`level` field). **No defects found
in STORY-182 or STORY-183 substance.** The self-contradicting STATE.md rows (Check 6) were
fixed in this same burst (D-544).
