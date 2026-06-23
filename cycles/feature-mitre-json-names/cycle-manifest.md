---
document_type: cycle-manifest
cycle_id: feature-mitre-json-names
cycle_type: feature
version: v0.9.4
status: CLOSED
started: 2026-06-23T01:00:00Z
completed: 2026-06-23T00:00:00Z
producer: orchestrator
github_issue: 64
release_tag: v0.9.4
release_commit: 96b49e8
release_pr: 309
release_yml_run: "28053327452 SUCCESS — 4 binaries"
---

# Cycle Manifest: feature-mitre-json-names (Feature)

## Summary

GitHub issue #64: Add per-finding `mitre_attack` array to JSON output. Each element
carries resolved technique objects (id, name?, tactic_id?, tactic_name?, reference)
for every ID in `mitre_techniques`. Unknown IDs emit partial objects (id+reference
only). Empty `mitre_techniques` vecs omit the `mitre_attack` key entirely. Additive
and non-breaking per F1 schema-compat verdict.

## Phase Status

| Phase | Status | Notes |
|-------|--------|-------|
| F1 — Delta Analysis | PASSED 2026-06-23 | 1 BC, 1 story, additive/non-breaking. Research-agent override: array design. |
| F2 — Spec Evolution | PASSED 2026-06-23 | BC-2.11.035 v1.0 (10 ACs); BC-INDEX v1.70; PRD v1.34; interface-definitions v1.3; BC-2.11.001 v1.7. |
| F3 — Incremental Stories | PASSED 2026-06-23 | STORY-129 (Wave 57, 5 pts, input-hash 2a5cee9, depends_on []); STORY-INDEX v2.7. |
| F4 — TDD Implementation | PASSED 2026-06-23 | STORY-129 delivered; 13 tests; 3-pass converged; PR #306 → develop 2fa6606 (D-208) |
| F5 — Scoped Adversarial | PASSED 2026-06-23 | HIGH F-1 ICS tactic-catalog fix; PR #307 → develop 029725b (D-212) |
| F6 — Targeted Hardening | PASSED 2026-06-23 | Formal/mutation/fuzz/security/regression all PASS (D-213) |
| F7 — Delta Convergence | CONVERGED 2026-06-23 | docs PR #308 → develop 760b6ca; human-approved; released v0.9.4 (D-216/D-217) |

## Delivered (FINAL — cycle CLOSED)

| Metric | Value |
|--------|-------|
| Stories delivered | STORY-129 (stories_delivered total=78) |
| BCs created | 1 new (BC-2.11.035 v1.1 final) |
| BCs modified | 5 (BC-2.11.001 v1.7, BC-2.10.002 v1.6, BC-2.10.003 v1.5, BC-2.10.007 v1.9, BC-2.16.004 v1.8) |
| MitreTactic variants added | 3 (IcsDiscovery TA0102, IcsCollection TA0100, IcsCommandAndControl TA0101) |
| VPs created | 0 (test-sufficient classification) |
| Story points | 5 pts (Wave 57) |
| Release version | v0.9.4 (tag on main 96b49e8; 4 binaries; run 28053327452) |
| PRs merged | #306 (mitre_attack impl), #307 (ICS tactic fix), #308 (docs), #309 (release) |
| GitHub issue closed | #64 (PR #306) |

## Spec Changes

| Artifact | Change | Before | After |
|----------|--------|--------|-------|
| prd.md | Added per-finding `mitre_attack` array spec to JSON output section | v1.33 | v1.34 |
| BC-INDEX.md | Added BC-2.11.035 row; SS-11 count 34→35; total 302→303 | v1.69 | v1.70 |
| BC-2.11.035.md | New BC authored (10 ACs; catalog extension; architecture anchors) | (absent) | v1.0 |
| BC-2.11.001.md | Added pointer note to BC-2.11.035 for per-finding computed fields | v1.6 | v1.7 |
| prd-supplements/interface-definitions.md | Added `MitreAttackEntry` / `FindingJsonDto` interface definitions | v1.2 | v1.3 |
| stories/STORY-129.md | New story authored (Wave 57, 5 pts, BC-2.11.035) | (absent) | v1.0 |
| stories/STORY-INDEX.md | Added STORY-129; totals 81→82 stories, 56→57 waves, 521→526 pts | v2.6 | v2.7 |

## Research Artifacts

| File | Description |
|------|-------------|
| `cycles/feature-mitre-json-names/f1-delta-analysis.md` | F1 delta analysis — impact boundary, spec delta, ATT&CK version, story scope, regression risk, schema-compat verdict |
| `cycles/feature-mitre-json-names/mitre-json-shape-research.md` | Research-agent investigation — best JSON shape for MITRE data (overrides initial flat-field design; adopts array-of-objects per ECS/OCSF) |

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Array of objects under `mitre_attack` (not flat `mitre_tactic`/`mitre_name` fields) | Research-agent override: every technique resolved; ECS/OCSF alignment; LLM-agent, human, and SIEM consumers all benefit from structured array |
| `reference` always synthesized from ID | URL is `format!("https://attack.mitre.org/techniques/{}/", id)` for ALL IDs — never from catalog |
| Unknown IDs produce partial objects (id + reference only) | ID is never lost; name/tactic fields absent via `skip_serializing_if = "Option::is_none"` |
| `technique_tactic_id()` catalog extension in src/mitre.rs | tactic_id not currently exposed; new pure accessor matching on `MitreTactic` variants |
| No new VP (test-sufficient) | DTO logic is pure Option-chaining over already Kani-verified VP-007 `technique_info` |

## Open Items (F4+)

| ID | Description |
|----|-------------|
| STORY-129/Task-1 | Add `technique_tactic_id()` to src/mitre.rs; extend `vp007_catalog_drift_guard` |
| STORY-129/Task-2 | Create src/reporter/json_dto.rs (`MitreAttackEntry` + `FindingJsonDto<'a>` + `From`) |
| STORY-129/Task-3 | Modify src/reporter/json.rs — swap raw slice to `Vec<FindingJsonDto>` |
| STORY-129/Task-4 | Add 10 AC tests to tests/reporter_json_tests.rs |

## Notes

- `mitre_techniques` raw field is unchanged (additive/non-breaking).
- No new crate dependencies required (`serde` / `serde_json` already present).
- input-hash for STORY-129 verified at D-206: `2a5cee9` (MATCH).
