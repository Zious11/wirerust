---
document_type: consistency-reaudit
level: ops
version: "1.0"
status: gaps-found
producer: consistency-validator
timestamp: 2026-07-13T00:00:00Z
phase: 2
inputs:
  - src/mitre.rs
  - src/dispatcher.rs
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
  - .factory/specs/prd.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/decisions/ADR-013-iec104-stream-dispatch-and-parser-design.md
traces_to: .factory/cycles/feature-iec104/
cycle: feature-iec104
audit_scope: Phase F2 spec-evolution delta re-audit (post-fix-burst) — IEC 60870-5-104 analyzer
input-hash: "7274000"
---

# Consistency Re-Audit Report: wirerust — Feature IEC-104 Phase F2 Delta (Post-Fix-Burst)

## Report Metadata

| Field | Value |
|-------|-------|
| **Product** | wirerust |
| **Generated** | 2026-07-13T00:00:00Z |
| **Generator** | consistency-validator |
| **Prior Audit** | `.factory/cycles/feature-iec104/consistency/f2-consistency-audit.md` |
| **Artifacts Scanned** | src/mitre.rs, src/dispatcher.rs, 8 index/top-level docs, 2 ADR-013 files, 31 BC files (BC-2.19.001..027 + BC-2.05.012 + BC-2.10.010 + BC-2.12.025 + BC-2.18.003 + BC-2.18.004) |
| **Prior Gap Count** | 12 (GAP-F2-001..012) |
| **Gate Result** | **GAPS-FOUND** — 1 prior gap still open (GAP-F2-008), 2 new minor gaps |

---

## 1. Prior Gap Status Table

| Gap ID | Severity | Description | Status | Evidence |
|--------|----------|-------------|--------|---------|
| GAP-F2-001 | CRITICAL | BC-2.10.010 tactic must be `IcsInhibitResponseFunction` for T0881 | **CLOSED** | BC-2.10.010 line 11 `traces_to: .factory/specs/domain/domain-spec.md`; H1 title "T0881 'Service Stop'"; line 57 `MitreTactic::IcsInhibitResponseFunction`. All three locations corrected from T0809/IcsImpact to T0881/IcsInhibitResponseFunction. |
| GAP-F2-002 | MINOR | VP-046 must be in proptest block, NOT Kani | **CLOSED** | VP-INDEX v2.42 modification note: "verify_classify_frame_format_totality removed from #[cfg(kani)] block and replaced with proptest! illustration (VP-046 is proptest P1, not Kani)". ADR-013 Decision 8 line 247: `// VP-046 (proptest), not Kani.` verification-coverage-matrix.md line 225 shows VP-046 tool=proptest. |
| GAP-F2-003 | MAJOR | ARCH-INDEX SS-05 count must be 12 | **CLOSED** | ARCH-INDEX v2.14 line 171: `\| SS-05 \| Protocol Dispatch \| CAP-05 \| dispatcher.rs, analyzer/mod.rs \| 12 \|`. Changelog entry (2026-07-13): "SS-05 11→12 (BC-2.05.012 dispatcher oracle)". |
| GAP-F2-004 | MAJOR | ARCH-INDEX SS-10 count must be 10 | **CLOSED** | ARCH-INDEX v2.14 line 176: `\| SS-10 \| MITRE Mapping \| CAP-10 \| mitre.rs \| 10 \|`. Changelog entry (2026-07-13): "SS-10 9→10 (BC-2.10.010 VP-007 six-part atomic)". |
| GAP-F2-005 | MAJOR | ARCH-INDEX SS-12 count must be 25 | **CLOSED** | ARCH-INDEX v2.14 line 178: `\| SS-12 \| CLI / Entry \| CAP-12 \| main.rs, cli.rs, lib.rs, summary.rs \| 25 \|`. Changelog entry (2026-07-13): "SS-12 24→25 (BC-2.12.025 CLI integration BC)". |
| GAP-F2-006 | MAJOR | ARCH-INDEX SS-19 count must be 27 | **CLOSED** | ARCH-INDEX v2.14 line 185: `\| SS-19 \| IEC-104 Analysis \| CAP-19 \| analyzer/iec104.rs \| 27 \|`. Changelog entry (2026-07-13): "SS-19 TBD→27". |
| GAP-F2-007 | MAJOR | O-04 debt row SEEDED=29, EMITTED=21, delta=8 | **CLOSED** | ARCH-INDEX v2.14 line 292: `\| O-04: MITRE techniques staged but never emitted — post-feature-iec104: SEEDED 29 − EMITTED 21 = 8 catalogue-only (T0881 newly seeded+emitted via feature-iec104...) \|`. PRD v1.53 §2.10 lines 1084-1085: "SEEDED=29, EMITTED=21, CATALOGUE-ONLY=29−21=8." See §5 for mitre.rs cross-check observation. |
| GAP-F2-008 | MAJOR | BC-INDEX derivation paragraph must show 378/377 | **STILL OPEN** | BC-INDEX v2.24 line 924 still reads "**Total BCs on disk: 346. Active: 345.**" — stale by 32 on-disk entries (misses +2 from v2.21: BC-2.11.036..037 and +30 from v2.23: feature-iec104). The header (lines 17-25) correctly states 378/377 but the derivation paragraph was not updated. Expected: 378 on disk, 377 active. |
| GAP-F2-009 | MAJOR | PRD O-04 CATALOGUE-ONLY=8; T0881 newly registered note | **CLOSED** | PRD v1.53 §2.10 line 1085: "feature-iec104 adds T0881 (v1.53: SEEDED 28→29, EMITTED 20→21, CATALOGUE-ONLY remains 8)." Line 674 retained-in-catalog note also correct. T0809 references replaced with T0881 throughout §2.19. |
| GAP-F2-010 | MAJOR | Factory mirror ADR-013 must exist in `.factory/specs/architecture/decisions/` | **CLOSED** | File exists at `.factory/specs/architecture/decisions/ADR-013-iec104-stream-dispatch-and-parser-design.md`. Title, status (accepted), all 10 Decision sections, and Consequences bullets are identical to `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`. Factory mirror additionally contains `## Rationale`, `## Alternatives Considered`, and `## Source / Origin` sections not present in docs/adr (this is a documented factory mirror extension pattern, not a contradiction). |
| GAP-F2-011 | MAJOR | BC-2.05.012, BC-2.10.010, BC-2.12.025 `traces_to:` must reference domain-spec.md | **CLOSED** | All three files confirmed line 11: `traces_to: .factory/specs/domain/domain-spec.md`. BC-INDEX v2.24 changelog (line 25): "All 30 IEC-104-related BCs … traces_to corrected from ADR path to `.factory/specs/domain/domain-spec.md`." |
| GAP-F2-012 | MINOR | BC-2.19.001..027 `traces_to:` must reference domain-spec.md | **CLOSED** | All 27 files in `.factory/specs/behavioral-contracts/ss-19/` confirmed line 11: `traces_to: .factory/specs/domain/domain-spec.md` (verified by loop over all 27 files). |

**Summary: 11 of 12 prior gaps CLOSED. 1 gap (GAP-F2-008) still open.**

---

## 2. New Gaps Found

### NEW-GAP-001 (MINOR): BC-2.18.003 v1.4 modification entry absent from `modified:` list

- **File:** `.factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md`
- **Location:** Frontmatter `modified:` field
- **Finding:** BC-2.18.003 frontmatter declares `version: "1.4"` but the `modified:` list contains only three entries: v1.1, v1.2, v1.3. The v1.4 amendment (SUPPORTED_PORTS adds port 2404, supported entries 7→8, implemented as part of feature-iec104) is documented in BC-INDEX v2.23 and PRD v1.52 but not recorded in BC-2.18.003's own frontmatter modification history.
- **Correct value:** A v1.4 `modified:` entry should be present, e.g. `"v1.4: feature-iec104 SUPPORTED_PORTS adds port 2404 (DispatchTarget::Iec104 port-2404 Rule 8); supported entries 7→8. 2026-07-13"`.
- **Impact:** BC version 1.4 is unverifiable by inspection of the BC file itself — the modification source can only be cross-referenced from BC-INDEX.

### NEW-GAP-002 (MINOR): BC-2.18.003 and BC-2.18.004 missing `input-hash:` frontmatter

- **Files:** `.factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md`, `.factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md`
- **Location:** Frontmatter — no `input-hash:` field present (confirmed by grep returning no matches)
- **Finding:** The task specifies checking that the amended BCs (BC-2.05.012, BC-2.10.010, BC-2.12.025, BC-2.18.003, BC-2.18.004) all have `input-hash:` in their frontmatter. BC-2.05.012, BC-2.10.010, and BC-2.12.025 each have `input-hash: "c5be398"`. BC-2.18.003 (version "1.4", introduced: feature-protocol-coverage-F2) and BC-2.18.004 (version "1.3", introduced: feature-protocol-coverage-F2) have neither `input-hash:` nor `inputs:` fields.
- **Context:** These BCs predate feature-iec104 and were created without input-hash under feature-protocol-coverage-F2. The fix burst amended their content but did not backfill the `input-hash:` field.
- **Correct action:** Compute canonical input-hash values using `bin/compute-input-hash --write .factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md` and same for BC-2.18.004 (requires an `inputs:` list to be added first).

---

## 3. Observation (Not a Gap): mitre.rs pre-implementation state vs spec

This cross-check is required by the GAP-F2-007 verification procedure.

- **ARCH-INDEX O-04 / PRD §2.10** (spec state): SEEDED=29, EMITTED=21, CATALOGUE-ONLY=8 (T0881 newly added)
- **mitre.rs** (code state): `SEEDED_TECHNIQUE_ID_COUNT = 28` (line 474); `SEEDED_TECHNIQUE_IDS` array has 28 entries — no `"T0881"` entry; `EMITTED_IDS` array has 20 entries — no `"T0881"` entry; kani proof comment line 317 says "the seeded set is finite (28)"

**Assessment:** NOT a gap. BC-2.10.010 line 106 explicitly states "Story Anchor: (TBD — F3 story decomposition)". The IEC-104 feature has not yet been implemented; the BC-2.10.010 obligation to add T0881 to `src/mitre.rs` is a future-story requirement. The spec being ahead of the code is the intended feature-mode spec-first workflow. mitre.rs will be updated when the F3 implementation story for BC-2.10.010 runs. The mitre.rs comment at line 317 saying "28" will become stale post-implementation but is accurate for current HEAD.

---

## 4. Counts Coherence Table

| Metric | Expected | Actual | Source | Status |
|--------|----------|--------|--------|--------|
| BC files on disk (actual) | 378 | 378 | `find .factory/specs/behavioral-contracts -name "BC-*.md" \| wc -l` = 379 minus BC-INDEX.md = 378 | ✓ MATCH |
| BC-INDEX stated on-disk | 378 | 378 | BC-INDEX v2.24 header line 17: "378 entries" | ✓ MATCH |
| BC-INDEX stated active | 377 | 377 | BC-INDEX v2.24 header: "Active count: 377" | ✓ MATCH |
| BC-INDEX derivation paragraph | 378 on disk / 377 active | **346 on disk / 345 active** | BC-INDEX v2.24 line 924 | ✗ STALE (GAP-F2-008) |
| SS-19 BCs in BC-INDEX table | 27 | 27 | BC-INDEX lines 863-889 (BC-2.19.001..027); summary row line 922 | ✓ MATCH |
| SS-19 BC files on disk | 27 | 27 | `ls .factory/specs/behavioral-contracts/ss-19/ \| wc -l` = 27 | ✓ MATCH |
| ARCH-INDEX SS-05 BC count | 12 | 12 | ARCH-INDEX v2.14 line 171 | ✓ MATCH |
| ARCH-INDEX SS-10 BC count | 10 | 10 | ARCH-INDEX v2.14 line 176 | ✓ MATCH |
| ARCH-INDEX SS-12 BC count | 25 | 25 | ARCH-INDEX v2.14 line 178 | ✓ MATCH |
| ARCH-INDEX SS-19 BC count | 27 | 27 | ARCH-INDEX v2.14 line 185 | ✓ MATCH |
| O-04 SEEDED count (spec) | 29 | 29 | ARCH-INDEX line 292; PRD line 1084 | ✓ MATCH |
| O-04 EMITTED count (spec) | 21 | 21 | ARCH-INDEX line 292; PRD line 1084 | ✓ MATCH |
| O-04 CATALOGUE-ONLY (spec) | 8 | 8 | ARCH-INDEX line 292; PRD line 1084 | ✓ MATCH |
| mitre.rs SEEDED count (code) | 29 (post-impl) | **28** | `SEEDED_TECHNIQUE_ID_COUNT` line 474; array count | pre-impl (see §3) |
| mitre.rs EMITTED count (code) | 21 (post-impl) | **20** | `EMITTED_IDS` array count | pre-impl (see §3) |
| VP total | 47 | 47 | VP-INDEX v2.42; verification-architecture.md v2.30 line ~146; verification-coverage-matrix.md v1.45 line 254 | ✓ MATCH |
| VP Kani count | 16 | 16 | VP-INDEX v2.42; verification-coverage-matrix.md line 254 | ✓ MATCH |
| VP proptest count | 22 | 22 | VP-INDEX v2.42; verification-coverage-matrix.md line 254 | ✓ MATCH |
| VP cargo-fuzz count | 3 | 3 | VP-INDEX v2.42; verification-coverage-matrix.md line 254 | ✓ MATCH |
| VP integration_unit count | 6 | 6 | VP-INDEX v2.42; verification-coverage-matrix.md line 254 | ✓ MATCH |
| SUPPORTED_PORTS protocols count | 8 | 8 | SS-19 doc line 136: "supported count 7→8"; BC-2.18.003 PC description; PRD BC-INDEX row v2.23 note | ✓ MATCH |

---

## 5. Version Coherence Table

| Document | Expected Version | Actual Version | Source Line | Status |
|----------|-----------------|---------------|-------------|--------|
| BC-INDEX | v2.24 | v2.24 | BC-INDEX frontmatter line 4 | ✓ MATCH |
| PRD | v1.53 | v1.53 | PRD changelog line 428: "Version 1.53 delta (2026-07-13)" | ✓ MATCH |
| VP-INDEX | v2.42 | v2.42 | VP-INDEX frontmatter line 4 | ✓ MATCH |
| ARCH-INDEX | v2.14 | v2.14 | ARCH-INDEX frontmatter line 4 | ✓ MATCH |
| verification-architecture.md | v2.30 | v2.30 | verification-architecture.md frontmatter line 8 | ✓ MATCH |
| verification-coverage-matrix.md | v1.45 | v1.45 | verification-coverage-matrix.md frontmatter line 8 | ✓ MATCH |

All six version stamps match expected values. ✓

---

## 6. Cross-Reference Integrity Table

| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| VP-004 references BC-2.05.012 | Yes | VP-INDEX v2.42 modification: "VP-004 Verified BCs updated — BC-2.05.012 added (dispatcher oracle obligation for DispatchTarget::Iec104; harness verify_content_first_precedence_exhaustive)" | ✓ CLOSED |
| BC-2.05.012 cites VP-004 harness `verify_content_first_precedence_exhaustive` | Yes | BC-2.05.012 line 82: `\| VP-004 \| ... Kani: \`verify_content_first_precedence_exhaustive\` \|` | ✓ CLOSED (prior fix: was `verify_classify_correctness`) |
| VP-007 references BC-2.10.010 | Yes | VP-INDEX v2.42 modification: "VP-007 Verified BCs updated — BC-2.10.010 added (VP-007 is Kani with harnesses verify_all_seeded_ids_match_format + verify_all_seeded_ids_resolve)" | ✓ CLOSED |
| BC-2.10.010 cites VP-007 as Kani | Yes | BC-2.10.010 line 79: `\| VP-007 \| ... \| Kani: \`verify_all_seeded_ids_match_format\` + \`verify_all_seeded_ids_resolve\` \|` | ✓ CLOSED |
| VP-046 tool is proptest (not Kani) | proptest | verification-coverage-matrix.md line 225: `\| VP-046 \| ... \| proptest \| P1 \|`; ADR-013 Decision 8 line 247: `// VP-046 (proptest), not Kani.` | ✓ CLOSED |
| VP-044 scoped to parse_apci_header ONLY | Yes | verification-architecture.md VP-044 row: "SCOPE: this Kani harness covers parse_apci_header ONLY"; ADR-013 Decision 8: "verify_classify_frame_format_totality removed from Kani block" | ✓ CLOSED |
| All BC-2.19.* covered by ≥1 VP | Yes | VP-044 anchors BC-2.19.001..018 (APCI parse); VP-045 anchors BC-2.19.005..009 (carry); VP-046 anchors BC-2.19.010..014 (frame format); VP-047 anchors BC-2.19.015..027 (fuzz safety) | ✓ PASS |
| BC-INDEX SS-19 subsystem column all "SS-19" | Yes | All 27 BC-2.19.* have `subsystem: SS-19` in frontmatter (confirmed by `grep subsystem` across ss-19/ directory) | ✓ PASS |
| BC-2.19.001..027 all in BC-INDEX table | 27 rows | BC-INDEX ss-19 section lines 863-889 lists all 27; summary row line 922: "feature-iec104 (SS-19) \| 27 \| BC-2.19.001..027" | ✓ PASS |
| No BC-2.19.* file missing from disk | 27 files | All 27 BC-2.19.001..027 confirmed present in `.factory/specs/behavioral-contracts/ss-19/` | ✓ PASS |
| No BC-2.19.* ID collision with other subsystems | None | BC-2.19 namespace is unique; ss-19 is the only subdirectory with 2.19.* IDs; no cross-subsystem collision possible | ✓ PASS |
| input-hash present: BC-2.19.001..027 | All 27 | `grep -l "input-hash" ss-19/*.md \| wc -l` = 27; all have `input-hash: "803f8d5"` at frontmatter line 27 | ✓ PASS |
| input-hash present: BC-2.05.012, BC-2.10.010, BC-2.12.025 | All 3 | All three have `input-hash: "c5be398"` at frontmatter line 26 | ✓ PASS |
| input-hash present: BC-2.18.003, BC-2.18.004 | Both | **MISSING** — neither file has `input-hash:` frontmatter field | ✗ NEW-GAP-002 |
| ADR-013 factory mirror Title matches docs/adr | Identical | Both: `# ADR-013: IEC-104 Stream Dispatch and Parser Design` | ✓ MATCH |
| ADR-013 factory mirror Status matches docs/adr | accepted | Both frontmatter: `status: accepted` | ✓ MATCH |
| ADR-013 factory mirror Decisions match docs/adr | 10 decisions | Section diff shows factory mirror adds `## Rationale`, `## Alternatives Considered`, `## Source / Origin` beyond docs/adr (standard factory mirror extension); core Decisions 1-10 and Consequences section identical | ✓ MATCH (extension only) |
| T0881 in MITRE table of both ADR-013 files | T0881 Service Stop | Both files: `\| **T0881** \| **Service Stop** \| **STOPDT-act observed...** \| **NEW — add via Decision 10** \|` | ✓ MATCH |
| VP-046 tool label in both ADR-013 files | proptest | Both files: `\| VP-046 \| proptest \| P1 \| U/S/I discrimination totality — \`classify_frame_format\` total over all 256 CF1 values \|` | ✓ MATCH |

---

## 7. Final Verdict

**GAPS-FOUND** — Total 3 gaps (1 still-open from prior audit + 2 new):

### Open Gaps by Category

| ID | Category | Severity | File | Description |
|----|----------|----------|------|-------------|
| GAP-F2-008 | count-drift | MAJOR | `.factory/specs/behavioral-contracts/BC-INDEX.md` line 924 | Derivation paragraph still says "Total BCs on disk: 346. Active: 345." — stale by 32 (should be 378/377). Header correctly says 378/377 but the running derivation was not updated during the fix burst. |
| NEW-GAP-001 | frontmatter-drift | MINOR | `.factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md` | version "1.4" declared but `modified:` list contains only entries for v1.1, v1.2, v1.3; v1.4 amendment (SUPPORTED_PORTS adds port 2404) not recorded in the BC's own modification history. |
| NEW-GAP-002 | input-hash-absent | MINOR | `.factory/specs/behavioral-contracts/ss-18/BC-2.18.003.md` AND `.factory/specs/behavioral-contracts/ss-18/BC-2.18.004.md` | Both amended BCs lack `input-hash:` frontmatter field. BC-2.05.012, BC-2.10.010, BC-2.12.025 (also amended in this feature) each have `input-hash:`. These were pre-existing BCs from feature-protocol-coverage-F2 that were amended without backfilling input-hash. |

### Gap Count Summary

| Category | Count |
|----------|-------|
| Prior gaps CLOSED | 11 |
| Prior gaps STILL OPEN | 1 (GAP-F2-008) |
| New gaps introduced by fix burst | 2 (NEW-GAP-001, NEW-GAP-002) |
| **Total open gaps** | **3** |

### Blocked Paths

The single still-open MAJOR gap (GAP-F2-008) does not block Phase 3 story-writer entry since the BC-INDEX table (lines 863-889) is correct and the header accurately states 378/377. The derivation paragraph is an internal consistency artifact. Recommend fixing before next wave gate.

The two new MINOR gaps do not block Phase 3 entry.

---

## Appendix A: Methodology

All artifact reads were performed using the Read tool. Source-of-truth tier checked first (mitre.rs, dispatcher.rs), then indexes, then architecture/spec documents, then individual BCs. No files were modified. BC count was verified by `find .factory/specs/behavioral-contracts -name "BC-*.md"` (379 total, minus BC-INDEX.md = 378 actual BC files on disk). All 27 BC-2.19.* files had their `traces_to:` verified by shell loop. ADR-013 both copies compared with diff on section headings.

## Appendix B: Closed Gap Evidence Index

| Gap | Closing File(s) | Closing Evidence |
|-----|-----------------|-----------------|
| GAP-F2-001 | BC-2.10.010.md line 29 (title), 50, 57 | "T0881 'Service Stop'"; IcsInhibitResponseFunction in all three body locations |
| GAP-F2-002 | VP-INDEX.md v2.42 changelog; ADR-013 Decision 8 line 247; verification-coverage-matrix.md line 225 | "VP-046 is proptest P1, not Kani"; proptest tool column |
| GAP-F2-003 | ARCH-INDEX.md v2.14 line 171 | `\| SS-05 \| ... \| 12 \|` |
| GAP-F2-004 | ARCH-INDEX.md v2.14 line 176 | `\| SS-10 \| ... \| 10 \|` |
| GAP-F2-005 | ARCH-INDEX.md v2.14 line 178 | `\| SS-12 \| ... \| 25 \|` |
| GAP-F2-006 | ARCH-INDEX.md v2.14 line 185 | `\| SS-19 \| ... \| 27 \|` |
| GAP-F2-007 | ARCH-INDEX.md v2.14 line 292; PRD.md lines 1084-1085 | "SEEDED 29 − EMITTED 21 = 8"; "SEEDED=29, EMITTED=21, CATALOGUE-ONLY=8" |
| GAP-F2-009 | PRD.md v1.53 line 1085 | "feature-iec104 adds T0881 (v1.53: SEEDED 28→29, EMITTED 20→21, CATALOGUE-ONLY remains 8)" |
| GAP-F2-010 | `.factory/specs/architecture/decisions/ADR-013-iec104-stream-dispatch-and-parser-design.md` existence | File present; title/status/decisions/consequences identical to source |
| GAP-F2-011 | BC-2.05.012.md line 11; BC-2.10.010.md line 11; BC-2.12.025.md line 11 | All three: `traces_to: .factory/specs/domain/domain-spec.md` |
| GAP-F2-012 | All 27 files in `.factory/specs/behavioral-contracts/ss-19/`, line 11 | All: `traces_to: .factory/specs/domain/domain-spec.md` |
