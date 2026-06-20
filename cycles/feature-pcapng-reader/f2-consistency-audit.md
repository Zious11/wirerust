---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
timestamp: 2026-06-19T00:00:00Z
cycle: feature-pcapng-reader
phase: F2
traces_to: .factory/cycles/feature-pcapng-reader/cycle-manifest.md
---

# F2 Consistency Audit — pcapng-reader Feature Cycle

**Audit date:** 2026-06-19  
**Scope:** F2 spec evolution artifacts as they stand on disk  
**Verdict:** NOT CLEAN — 6 findings (1 HIGH, 3 MEDIUM, 2 LOW)

---

## Summary Table

| Check | Result | Notes |
|-------|--------|-------|
| 1. Bidirectional supersession BC-2.01.004 / BC-2.01.009 | PASS | Both directions present and consistent |
| 2. Dangling references to BC-2.01.004 | PASS | All remaining citations are intentionally annotated |
| 3. Stale "pcapng unsupported" assertions | PASS with known-open | BC-2.12.011 is the only remaining stale; correctly logged as F3 task |
| 4. BC-INDEX integrity (10 new rows, retired row, counts) | FAIL | Timestamp stale; total_bcs in BC-INDEX consistent internally but diverges from epics |
| 5. Error-taxonomy integrity | PASS | E-INP-008..011 present, sequential, non-colliding; E-INP-002 note correct |
| 6. ADR-009 traceability | FAIL (partial) | ADR-009 Status section has stale assertion; all new BCs carry ADR-009 refs |
| 7. Story/epic arithmetic | FAIL | epics.md total_bcs 297 diverges from BC-INDEX active 302 by 5 (BC-2.11.030-034) |
| 8. Cross-references resolve | PASS | All BC/ADR/error-code cross-references tested point to existing targets |
| Bonus: HS-001 holdout (not in scope but surfaced) | NOTE | HS-001 cites retired BC-2.01.004 with incorrect pcapng behavior; tracked in STATE.md/cycle-manifest as F3 task |

---

## Findings

### FINDING-001 — HIGH
**ADR-009 "Status as of 2026-06-19" section contains a self-contradictory assertion**

**File:** `.factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md`  
**Line:** 222-223  
**Text:**
```
BC-2.01.004 remains active until STORY-123 retires it.
```

**Why this is wrong:** BC-2.01.004 was retired by this same F2 spec-evolution burst. The ADR was authored as part of F2 and correctly records the retirement in Decision 6 (lines 113-118) and in the Consequences section (line 253: "Affected contract: BC-2.01.004 — retired by this decision; replaced by BC-2.01.009"). However, the Status section was written prospectively and then not updated after the retirement happened within the same burst. The "Status as of 2026-06-19" block now says two contradictory things: the rest of the ADR says BC-2.01.004 is retired, but lines 222-223 say it "remains active."

**Risk:** A reader who reads only the Status section (a common skimming pattern) will conclude BC-2.01.004 is still active. This contradicts BC-2.01.004.md (lifecycle_status: retired), BC-INDEX (RETIRED row), and STORY-001.md (AC-006 annotated as inverted).

**Fix:** Replace lines 220-223 with:
```
Proposed (spec-complete). BC-2.01.004 was retired within this same F2 spec-evolution burst
(lifecycle_status: retired, superseded_by: BC-2.01.009). Implementation is planned for
STORY-123 through STORY-127 (F2-F4 cycle). No pcapng story has yet been assigned for
implementation; src/reader.rs still reflects the pre-F2 classic-pcap-only state.
```

---

### FINDING-002 — HIGH
**epics.md total_bcs 297 disagrees with BC-INDEX active count 302 — 5 BCs (BC-2.11.030-034) missing from epics coverage table**

**File:** `.factory/stories/epics.md`  
**Lines:** 13 (frontmatter `total_bcs: 297`), 291 (TOTAL row), 296 (arithmetic block), 316 (`297 / 297`), 347-348 (Coverage confirmed)

**What happened:** BC-2.11.030-034 (5 grouped-collapse BCs for STORY-119) were added to BC-INDEX in v1.44 (2026-06-18). epics.md was at v1.4 at the time and should have been updated with a new E-19 row or by expanding E-18 to include these 5 BCs. It was not updated. The v1.5 pcapng update then propagated the wrong baseline: it computed `288 + 9 = 297` when the correct pre-pcapng total was `293 + 9 = 302`.

**Evidence:**
- BC-INDEX v1.52 header: "Total active BCs: 293→302 (net +10 new, BC-2.01.004 retired = 1 retired)"
- BC-INDEX "Total BCs on disk: 303. Active: 302." (line 524)
- epics.md `total_bcs: 297`, Coverage confirmed "297 / 297 active BCs assigned"
- BC-2.11.030-034 are assigned to STORY-119 (`bcs:` frontmatter lines 22-26 in STORY-119.md) but appear in NO epic row in epics.md
- SS-11 has 34 BCs on disk (confirmed by `ls ss-11/ | grep -c BC-2.11`); epics counts only E-8 (24) + E-18 (5) = 29 for SS-11

**Risk:** The Coverage Check assertion "0 unassigned" is false. 5 active BCs are unassigned in the epic decomposition. Any F3 planning relying on epics.md totals will undercount by 5. This also means the v1.5 update's arithmetic "288→297" is incorrect: the prior baseline should have been 293.

**Fix:**
1. Add a new row to the Per-Epic BC Assignment table for E-18 extension or a new E-19: `BC-2.11.030..034 | 5`
2. Update arithmetic block: add E-19 (or E-18-B) for 5 BCs; recompute 297 + 5 = 302
3. Update TOTAL row to 302
4. Update `total_bcs:` frontmatter to 302
5. Update Coverage confirmed assertion to "302 / 302"
6. Update E-8 body text and E-18 body text to reference the grouped-collapse BCs' epic home

---

### FINDING-003 — MEDIUM
**prd.md RTM (§7) has BC-2.01.004 as a raw active row and is missing 10 new BC-2.01.009-018 rows**

**File:** `.factory/specs/prd.md`  
**Lines:** 1403 (BC-2.01.004 raw row in RTM); no entries for BC-2.01.009-018 exist in the §7 RTM

**What happened:** prd.md v1.29 delta note (line 414-422) says "10 new BCs added to §2.1 for pcapng block-walk reader." Section §2.1 (lines 552-577) was correctly updated — BC-2.01.004 is struck-through there, and BC-2.01.009-018 are listed. However, §7 Requirements Traceability Matrix was not updated:

- Line 1403: `| BC-2.01.004 | CAP-01 | SS-01 (reader.rs) | P0 | unit |` — raw, not struck-through
- BC-2.01.009 through BC-2.01.018 are entirely absent from the RTM

**Evidence:** The §7 RTM immediately jumps from BC-2.01.008 (line 1407) to BC-2.02.001 (line 1408), skipping all 10 new BCs. BC-2.01.004 at line 1403 is not rendered with strikethrough markup.

**Risk:** The RTM is the canonical machine-readable traceability surface used by the holdout-evaluator and verification passes. A half-updated RTM means 10 new P0/P1 BCs have no RTM row and BC-2.01.004's RTM row does not reflect its retired status.

**Fix:**
1. Retire line 1403: `| ~~BC-2.01.004~~ | ~~CAP-01~~ | ~~SS-01 (reader.rs)~~ | ~~P0~~ | ~~unit~~ |`
2. Insert 10 new rows after line 1407 (BC-2.01.008):
```
| BC-2.01.009 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.010 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.011 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.012 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.013 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.014 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.015 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.016 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.017 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.018 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
```
3. Add a delta note at the top of §7 (or inline) referencing prd.md v1.29 for the pcapng additions.

---

### FINDING-004 — MEDIUM
**BC-2.12.011 carries a stale "related to BC-2.01.004" assertion with now-incorrect rationale and no F3 annotation in the BC body**

**File:** `.factory/specs/behavioral-contracts/ss-12/BC-2.12.011.md`  
**Line:** 96

**Text:**
```
- BC-2.01.004 -- related to (pcapng is rejected at reader level; here it is excluded before reader)
```

**Why this is wrong:** The rationale "pcapng is rejected at reader level" is no longer true — BC-2.01.004 is retired, and pcapng is now ACCEPTED at the reader level (BC-2.01.009). The "excluded before reader" assertion was the redundant-safety-net rationale; that rationale is now semantically inverted. Furthermore, BC-2.12.011's postcondition 2 (`.pcapng files are excluded`) describes behavior that STATE.md lines 108 and cycle-manifest line 64 document as needing revision in F3, but the BC body itself has zero forward-looking annotation about this planned change.

**Note:** The cycle-manifest correctly logs "BC-2.12.011 must be revised/retired+replaced when STORY-127 is decomposed" and STATE.md section D items 1-2 capture this. The gap is that BC-2.12.011 body has no matching annotation, making the spec inconsistent with its own governance documentation.

**Fix:**
1. Update line 96 from:
   `- BC-2.01.004 -- related to (pcapng is rejected at reader level; here it is excluded before reader)`
   to:
   `- ~~BC-2.01.004~~ -- formerly related to (pcapng was rejected at reader level; RETIRED 2026-06-19; pcapng now accepted via BC-2.01.009)`
2. Add an F3 note to the BC body (e.g., at the end of Invariants or in a new "F3 Forward-Action" section):
   `**F3 ACTION (STORY-127):** This BC describes glob exclusion of *.pcapng which was correct while reader.rs rejected pcapng. STORY-127 will update resolve_targets to include *.pcapng now that BC-2.01.009 accepts it. This BC will require revision (update postcondition 2 and test vectors) or retirement + replacement when STORY-127 is decomposed.`

---

### FINDING-005 — LOW
**BC-INDEX.md frontmatter timestamp 2026-06-18 is one day behind its v1.52 version date (2026-06-19)**

**File:** `.factory/specs/behavioral-contracts/BC-INDEX.md`  
**Line:** 7

**Text:** `timestamp: 2026-06-18T00:00:00Z`

**Context:** The v1.52 changelog entry clearly states "v1.52 2026-06-19 (F2 pcapng-reader-support spec evolution — INTEGRATE sub-burst)". All 10 new BC files carry `timestamp: 2026-06-19T00:00:00Z`. The BC-INDEX frontmatter was not updated from the v1.51 timestamp when v1.52 was written.

**Risk:** Timestamp drift in index files causes false ordering when sorting artifact history; minor tooling concern.

**Fix:** Update line 7 to `timestamp: 2026-06-19T00:00:00Z`.

---

### FINDING-006 — LOW
**HS-001 behavioral_contracts and body still cite BC-2.01.004 as requiring pcapng rejection — scenario now tests inverted behavior**

**File:** `.factory/holdout-scenarios/HS-001-pcap-link-type-gating.md`  
**Lines:** 12, 22, 46, 54, 69-70, 76-77, 92 (multiple references)  
**File:** `.factory/holdout-scenarios/HS-INDEX.md`  
**Line:** 151

**Situation:** HS-001 tests that "the pcapng file is rejected at the reader level" (line 46). This was correct behavior under BC-2.01.004 but is now the WRONG expected outcome under BC-2.01.009. The holdout-evaluator running HS-001 against a post-F3 implementation would now correctly see pcapng accepted (exit 0) and score HS-001 as FAIL — a false negative.

**Why this is LOW not CRITICAL:** This is correctly identified and tracked. STATE.md line 109 ("Update HS-001 + HS-INDEX (cite retired BC-2.01.004) — PO action in F3") and cycle-manifest open follow-up item 2 both capture this. The HS-001 file is sealed (not shown to implementer/test-writer); the scenario cannot cause incorrect implementation guidance. The holdout-evaluator is not running against F2 spec artifacts — it runs against implementation that doesn't exist yet. The risk is real but deferred to F3 entry.

**Gap vs. audit scope:** The cycle-manifest explicitly defers HS-001 update to F3. However, HS-INDEX.md line 151 still shows `BC-2.01.001, BC-2.01.004` without any staleness annotation, and HS-001 has no warning comment about its stale state. Given that HS-001 has `lifecycle_status: active` and `must_pass: true`, a reader of HS-INDEX sees an active must-pass scenario citing a retired BC with inverted semantics.

**Fix (F3 entry):**
1. HS-001.md: Add frontmatter annotation `stale_reason: "BC-2.01.004 RETIRED 2026-06-19 F2 — pcapng now accepted. Step 5 and rubric line 1 must be inverted. PO to revise in F3 before Phase-4 holdout run."` and set `staleness_check: 2026-06-19`.
2. HS-001.md: Update behavioral_contracts to replace BC-2.01.004 with BC-2.01.009.
3. HS-001.md: Invert Step 5 (pcapng now exits 0 with analysis) and rubric ("pcapng accepted with correct packet count").
4. HS-INDEX.md line 151: Update BC column from `BC-2.01.001, BC-2.01.004` to `BC-2.01.001, BC-2.01.009`.

---

## Checks That PASSED — Detail

### Check 1: Bidirectional Supersession (PASS)

- **BC-2.01.004.md** frontmatter: `superseded_by: BC-2.01.009`, `lifecycle_status: retired`, `retired: "v0.10.0-pcapng-F2"`. H1 heading includes `[RETIRED — superseded by BC-2.01.009]`. Retirement rationale is complete.
- **BC-2.01.009.md** frontmatter: `supersedes: BC-2.01.004`, `lifecycle_status: active`. Body line 34: "This BC supersedes BC-2.01.004, inverting its postconditions from rejection to acceptance." Related BCs section: "BC-2.01.004 -- supersedes (this BC inverts BC-2.01.004's rejection postconditions)".
- Both directions are present, populated, and consistent. No gap.

### Check 2: Dangling References to BC-2.01.004 (PASS)

All 26 file locations referencing BC-2.01.004 were reviewed. Every reference is intentionally annotated:
- **BC-2.01.004.md** itself — the source file, retained per append-only-numbering policy. PASS.
- **BC-2.01.009.md** — explicit supersedes relationship. PASS.
- **BC-INDEX.md** — struck-through row with RETIRED comment. PASS.
- **prd.md** — §2.1 struck-through; §7 RTM raw row (logged as FINDING-003 above, but is an annotation-deficit finding, not a live dependency). PASS for dangling-reference criterion.
- **test-vectors.md** — struck-through section with STALE annotation. PASS.
- **error-taxonomy.md** — F2 note on E-INP-002 correctly scopes pcapng out of that error path. PASS.
- **module-criticality.md** — F2 delta note annotates BC-2.01.004 RETIRED explicitly. PASS.
- **system-overview.md** — "formerly pcapng rejected ... now accepted" annotation. PASS.
- **ADR-009** — citations in context of "retired by this decision". PASS.
- **ARCH-INDEX.md** — "BC-2.01.004 retired/inverted" in reason note. PASS.
- **cap-01-pcap-ingestion.md** — "BC-2.01.004 is RETIRED" annotation. PASS.
- **STORY-001.md** — struck-through BC table row, AC-006 annotated with inversion note. PASS.
- **epics.md** — struck-through BC-2.01.004 entry with RETIRED annotation. PASS.
- **nfr-catalog.md** — NFR-COMPAT-001 and NFR-VIO-002 both have forward notes. PASS.
- **BC-2.12.011.md** — "related to" reference (stale rationale — logged as FINDING-004, not a live dependency). PASS for dangling criterion; separately flagged for semantic staleness.
- **HS-001.md** — cites BC-2.01.004 in active behavioral_contracts but is a sealed holdout scenario with correct F3 tracking (logged as FINDING-006, deferred to F3). PASS for dangling criterion; separately flagged as a known-open F3 task.

### Check 3: Stale "pcapng unsupported" Assertions (PASS with known-open)

Grep results surveyed. Every remaining "pcapng excluded/rejected/not supported" assertion falls into one of three categories:
1. **Retired BC body (BC-2.01.004.md):** Historical document retained per policy. Body text says "pcapng support is explicitly out of scope" in Invariant 1 — this is the old normative text, preserved for audit trail. Acceptable.
2. **BC-2.12.011.md:** "*.pcapng excluded" is still the CURRENT runtime behavior (STORY-127 hasn't landed). The BC correctly describes current code. The stale rationale line ("pcapng is rejected at reader level") is logged as FINDING-004.
3. **F3-annotated forward references:** cap-01, cap-12, nfr-catalog, spec-changelog all note the glob exclusion and reference STORY-127. All have F3 forward-action annotations. PASS.
4. **test-vectors.md line 378:** BC-2.12.011 test vector shows pcapng excluded from glob — this is CURRENTLY CORRECT behavior. STORY-127 will invert it. No F3 annotation on this specific test vector row, but the BC-2.12.011 section header at line 374 doesn't have a note either. This is consistent with FINDING-004 (body-level annotation deficit) but is not an independent stale assertion.

The question posed — "is BC-2.12.011 the ONLY remaining stale pcapng exclusion and is it logged as an F3 task?" — is CONFIRMED. BC-2.12.011 is the only remaining assertion that pcapng is excluded from the directory glob, and it is correctly logged as an F3 task in STATE.md (line 108), cycle-manifest (line 64), cap-01 (line 35), cap-12 (line 104-105), nfr-catalog NFR-VIO-002, and spec-changelog line 51.

### Check 4: BC-INDEX Integrity (PASS on row content; FAIL on timestamp)

- 10 new rows for BC-2.01.009-018 present, correctly titled, correct priorities, Story anchors assigned. PASS.
- BC-2.01.004 row is struck-through with RETIRED comment. PASS.
- Total active count 302 is internally consistent: 303 on disk, 1 retired. PASS on the count itself.
- SS-01 section header "18 BCs total on disk (17 active + 1 retired)" is correct. PASS.
- Frontmatter timestamp 2026-06-18 does not match v1.52 date of 2026-06-19. See FINDING-005.

### Check 5: Error-Taxonomy Integrity (PASS)

- E-INP-008 through E-INP-011 present in error-taxonomy.md v2.3, lines 74-77. All four entries are sequential, no ID collision. PASS.
- E-INP-008 BC refs: BC-2.01.010, BC-2.01.011, BC-2.01.017 — all three files exist on disk. PASS.
- E-INP-009 BC refs: BC-2.01.012, BC-2.01.017 — both exist. PASS.
- E-INP-010 BC refs: BC-2.01.012, BC-2.01.013, BC-2.01.015, BC-2.01.017 — all exist. PASS.
- E-INP-011 BC refs: BC-2.01.018, BC-2.01.017 — both exist. PASS.
- E-INP-002 note: "pcapng files are NO LONGER a trigger for E-INP-002; the BC-2.01.009 magic-byte probe routes pcapng files to E-INP-008..011 before reaching this path." Correct and present. PASS.
- next_free_error_code: E-INP-012 noted in addendum file. PASS.
- ERROR-TAXONOMY-ADDENDUM-pcapng.md marked CONSUMED, retained for audit trail. Acceptable staging artifact; not registered in BC-INDEX (correct — it is not a BC).

### Check 6: ADR-009 Traceability (PASS on BC linkage; FAIL on self-consistency)

- All 10 new BCs (BC-2.01.009-018) carry explicit ADR-009 references (confirmed by grep: 1-7 references each, all non-zero). PASS.
- ADR-009 file exists at `.factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md`. PASS.
- ARCH-INDEX.md has ADR 0009 row at line 192. PASS.
- ADR numbering sequential: ADR-005 through ADR-009 present, no collision. PASS.
- ADR-009 Status section self-contradiction (BC-2.01.004 "remains active"). See FINDING-001.

### Check 7: Story/Epic Arithmetic (FAIL — see FINDING-002)

The epics.md claims 297/297 active BCs covered. BC-INDEX claims 302 active BCs. The 5-BC gap (BC-2.11.030-034) pre-dates this F2 burst but was incorrectly propagated when the v1.5 pcapng update computed `288 + 9 = 297` instead of `293 + 9 = 302`.

### Check 8: Cross-References Resolve (PASS)

All "see BC-x", "see ADR-x", "see E-INP-x" references added in the F2 burst were verified to point to existing files:
- BC refs in new BCs: all related BCs (BC-2.01.004, BC-2.01.008, BC-2.01.010 through BC-2.01.018) exist on disk. PASS.
- ADR-009 refs in BCs: file exists. PASS.
- ADR-009 Decision 5, Decision 6 cross-references: confirmed present in ADR-009 body. PASS.
- E-INP-008..011 back-references in BC-2.01.017 and BC-2.01.018: E-INP numbers exist in error-taxonomy.md. PASS.
- STORY-123-127 referenced in ADR-009 Consequences: these stories are planned but not yet decomposed; references are intentionally forward-looking. PASS (future state, correctly flagged as planned in cycle-manifest and STATE.md).

---

## Known-Open F3 Items (Not Findings — Tracked Correctly)

These items are deferred to F3 and are correctly tracked in STATE.md section D and cycle-manifest. They are NOT additional findings but are listed here for completeness:

| Item | Location | Tracking |
|------|----------|---------|
| BC-2.12.011 revision/retirement (glob includes *.pcapng) | STATE.md:108, cycle-manifest:64 | F3 entry checklist |
| HS-001 + HS-INDEX update (invert pcapng behavior) | STATE.md:109, cycle-manifest:65 | F3 entry checklist |
| STORY-123..127 input-hash generation | cycle-manifest:66, spec-changelog:52 | F3 entry (compute-input-hash --write --scan) |
| VP assignments for BC-2.01.009-018 | cycle-manifest:67 | Post-F2 architect/VP-INDEX action |

---

## Remediation Priority Order

For the next burst before F3 story decomposition begins:

1. **FINDING-001 (HIGH):** Fix ADR-009 Status section — 5-minute edit, zero-risk. Do this before F3 entry to prevent implementer confusion.
2. **FINDING-002 (HIGH):** Fix epics.md total_bcs/coverage table — adds 5 BCs (030-034) to coverage, corrects total to 302. Required before F3 story decomposition uses epics.md for planning.
3. **FINDING-003 (MEDIUM):** Fix prd.md §7 RTM — strike BC-2.01.004, add 10 new rows. Required before Phase-4 holdout evaluation.
4. **FINDING-004 (MEDIUM):** Annotate BC-2.12.011 Related BCs and add F3 forward-action note. Can be done in same burst as FINDING-003.
5. **FINDING-005 (LOW):** Fix BC-INDEX.md timestamp. Can be done in any cleanup burst.
6. **FINDING-006 (LOW):** Annotate HS-001 and HS-INDEX as stale at F3 entry, then rewrite HS-001 scenario for pcapng acceptance.

---

## Closure Verification — Post-Fix Re-Audit

**Re-audit date:** 2026-06-19
**Auditor:** consistency-validator
**Scope:** Verify all 6 findings from the original report are closed; confirm no new drift was introduced by the 6 edits.

### FINDING-001 — CLOSED

**File:** `.factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md`

The "Status as of 2026-06-19" section (lines 218-225) now reads:

> "Proposed. `pcap-file` 2.0.0's pcapng module is dead code in the compiled binary; `src/reader.rs` does not import it. BC-2.01.004 ("Reject pcapng-Format Input at Reader Level") was RETIRED during F2 spec evolution and is superseded by BC-2.01.009 ("Accept pcapng Format: Transparent Detection via Magic-Byte Probe"); the spec changes are complete. Only the implementation remains pending, scheduled across STORY-123 through STORY-127 (F3 story decomposition forthcoming)."

This correctly states the retirement as a completed fact, with no "remains active until STORY-123" prospective language. The contradiction is resolved.

**LOW follow-up (Decision 6 "supersedes" typo check):** The only `supersedes` text in ADR-009 is at line 118: "The replacement BC (BC-2.01.009, …) supersedes BC-2.01.004." The word "supersedes" appears once and in the correct direction (BC-2.01.009 supersedes BC-2.01.004). The "supersedes BC-2.01.009 [sic] BC-2.01.004" duplication typo referenced in the task does not appear in the file on disk. No typo present. No action required.

**Verdict: CLOSED.**

---

### FINDING-002 — CLOSED

**File:** `.factory/stories/epics.md`

All five affected locations have been updated:

- **Frontmatter `total_bcs:`** — line 14: `total_bcs: 302`. Correct (was 297).
- **v1.6 changelog entry** — present at line 13: "BC-2.11.030–034 (5 grouped-collapse BCs added in BC-INDEX v1.44 for STORY-119) were missing from epics.md. Added to E-18 row. total_bcs corrected 297→302 (verified against BC-INDEX v1.52 ground truth: 302 active BCs)."
- **E-18 row in Per-Epic BC Assignment table** — now reads: "BC-2.11.025..029 (flat-mode collapse, STORY-118), BC-2.11.030..034 (grouped-collapse, STORY-119) | 10" (was "5"). E-18 count column updated to 10.
- **TOTAL row** — "302" with annotation "(297 prior + 5: BC-2.11.030–034 added; pre-pcapng baseline was 293, then +10 BC-2.01.009–018 −1 retired BC-2.01.004 = 302)".
- **Arithmetic Verification block** — E-18 line reads "10 (SS-11, BC-2.11.025..029 flat-collapse + BC-2.11.030..034 grouped-collapse) = 10"; final sum "302 / 302  ✓".
- **Coverage confirmed assertion** — "302 / 302 active BCs assigned, 0 unassigned, 0 double-assigned."

Independent recount against BC-INDEX v1.52: 303 BCs on disk, 1 retired (BC-2.01.004), 302 active. epics.md now agrees.

**Verdict: CLOSED.**

---

### FINDING-003 — CLOSED

**File:** `.factory/specs/prd.md`, §7 RTM (lines 1403-1422 as verified)

- BC-2.01.004 row is now struck through with "~~BC-2.01.004~~ | ~~CAP-01~~ | ~~SS-01 (reader.rs)~~ | ~~P0~~ | ~~unit~~ [RETIRED → BC-2.01.009]" (line 1408).
- 10 new rows BC-2.01.009 through BC-2.01.018 are present immediately following BC-2.01.008 (lines 1413-1422). All 10 carry CAP-01, SS-01 (reader.rs), and the correct priority and test-type values.
- The RTM no longer jumps from BC-2.01.008 to BC-2.02.001; all new pcapng BCs are present.

**Verdict: CLOSED.**

---

### FINDING-004 — CLOSED

**File:** `.factory/specs/behavioral-contracts/ss-12/BC-2.12.011.md`

The file is at v1.4. The modified block contains the entry: "v1.4: F2 audit FINDING-004 — annotate Related BCs BC-2.01.004 ref as STALE (pcapng now accepted via BC-2.01.009); add F3/STORY-127 forward-action note — 2026-06-19".

**Related BCs line (line 97):** Now reads:
> "~~BC-2.01.004~~ -- [STALE — 2026-06-19] related to (pcapng is rejected at reader level; here it is excluded before reader). **This rationale is now inverted**: BC-2.01.004 was RETIRED by the F2 pcapng-reader-support feature (ADR-009); pcapng is now ACCEPTED via BC-2.01.009 magic-byte probe. The `*.pcapng` directory-glob exclusion in this BC will be revised or retired when STORY-127 is decomposed in F3."

**F3 FORWARD ACTION block (lines 99-104):** Present and correctly states:
> "This BC describes `resolve_targets` excluding `*.pcapng` from directory glob expansion. That behavior was correct when reader.rs rejected pcapng. Now that BC-2.01.009 accepts pcapng, STORY-127 will update `resolve_targets` to include `*.pcapng`. At that point this BC requires revision (update Postcondition 2, Invariants, Edge Cases EC-001, and Canonical Test Vectors) or retirement + replacement. Do NOT implement this change before STORY-127 is formally decomposed."

The BC is NOT prematurely retired; `lifecycle_status: active` confirmed in frontmatter (line 14).

**Verdict: CLOSED.**

---

### FINDING-005 — CLOSED

**File:** `.factory/specs/behavioral-contracts/BC-INDEX.md`

Frontmatter timestamp (line 7): `timestamp: 2026-06-19T00:00:00Z`. Matches the v1.52 changelog date. No longer one day behind.

**Verdict: CLOSED.**

---

### FINDING-006 — CLOSED

**File:** `.factory/holdout-scenarios/HS-001-pcap-link-type-gating.md`

- `lifecycle_status: stale` (frontmatter line 23). Was `active`. Correct.
- `stale_reason:` (frontmatter line 27): "pcapng-rejection expectation (Step 5, BC-2.01.004) inverted by F2 pcapng-reader-support (BC-2.01.009 now accepts pcapng). Scenario rewrite is F3 scope (STORY-127)." Present and complete.
- Body banner (lines 35-40): "[STALE — 2026-06-19] This scenario's pcapng-rejection expectation (Step 5 below, and the BC-2.01.004 table row) is INVERTED by the F2 pcapng-reader-support feature (ADR-009). As of BC-2.01.009, pcapng is now an ACCEPTED input format — the `sample.pcapng` file in Step 5 should be ACCEPTED (exit 0), not rejected. BC-2.01.004 is RETIRED. This scenario must be fully rewritten in F3 when STORY-127 is decomposed. Until then, do NOT use this scenario as a gate for pcapng rejection behavior." Present and complete.
- Note: the HS-001 body still describes the old rejection behavior in Steps 5, rubric, and Verification Approach, which is expected — the staleness banner explicitly warns against using this for pcapng behavior gating, and the full rewrite is F3 scope.

**File:** `.factory/holdout-scenarios/HS-INDEX.md` (line 151)

HS-001 row now reads:
> "HS-001 | PCAP Link-Type Boundary — Accepted vs. Rejected at File Open **[STALE — 2026-06-19: pcapng-rejection expectation inverted by BC-2.01.009; rewrite F3/STORY-127]** | integration-boundaries | must-pass | 1 | BC-2.01.001, ~~BC-2.01.004~~ (retired → BC-2.01.009)"

Both staleness banners are present.

**Verdict: CLOSED.**

---

### Regression Checks

**Bidirectional supersession BC-2.01.004 ↔ BC-2.01.009 (re-confirm):**

ADR-009 line 118: "The replacement BC (BC-2.01.009, 'Accept pcapng Format: Transparent Detection via Magic-Byte Probe') supersedes BC-2.01.004." ADR-009 line 255 (Source/Origin section): "Affected contract: BC-2.01.004 ('Reject pcapng-Format Input at Reader Level', SS-01) — retired by this decision; replaced by BC-2.01.009." BC-INDEX row for BC-2.01.004: struck-through with "RETIRED 2026-06-19 F2 pcapng-reader-support: behavioral inversion — pcapng now accepted. superseded_by: BC-2.01.009". BC-INDEX row for BC-2.01.009: "supersedes BC-2.01.004". The bidirectional relationship is intact. No regression.

**New dangling references or arithmetic contradictions:**

- epics.md v1.6 arithmetic: E-18 = 10 (5 flat + 5 grouped); all other rows unchanged from v1.5; TOTAL = 302; Coverage = 302/302. No arithmetic contradiction introduced.
- prd.md §7 RTM: 10 new rows added (BC-2.01.009–018); no row was removed (only BC-2.01.004 gained strikethrough markup). The new test-type column values ("integration") are consistent with the BC body files which describe pcapng integration tests.
- BC-2.12.011 v1.4: stale annotation added to Related BCs, F3 forward-action note added. Postconditions, invariants, and test vectors were not changed (correctly — current behavior is still pcapng-excluded). No normative content disturbed.
- BC-INDEX timestamp corrected from 2026-06-18 to 2026-06-19. No cascading effect.
- HS-001 lifecycle_status changed from active to stale; `stale_reason` field added; body banner added. No normative holdout content removed. No new false references created.

**Version monotonicity check:**

- prd.md: reported as v1.30 in the task. The task states "confirm version bumps are monotonic (prd v1.30, BC-2.12.011 v1.4, epics v1.6)".
- BC-2.12.011: frontmatter line 4 shows `version: "1.4"`. Prior version was v1.3 (per modified block). Monotonic.
- epics.md: frontmatter line 3 shows `version: "1.6"`. Changelog entries show v1.5 → v1.6. Monotonic.
- BC-2.12.011 v1.4 changelog entry: "v1.4: F2 audit FINDING-004 — annotate Related BCs BC-2.01.004 ref as STALE (pcapng now accepted via BC-2.01.009); add F3/STORY-127 forward-action note — 2026-06-19". Present.
- epics.md v1.6 changelog entry: present at frontmatter line 13 (as verified above). Present.

Note: prd.md §7 version was not independently verified in this pass (the task states v1.30 as a given and directs verification of the BC-2.01.004 strikethrough and BC-2.01.009-018 rows, which are confirmed correct). The prd.md changelog entry for the §7 RTM update was not checked as a separate step but the normative RTM content is confirmed correct on disk.

**No new gaps detected.** All 6 edits are self-contained and do not create new cross-reference failures or arithmetic contradictions.

---

## Final Verdict

**CLEAN** — all 6 findings are closed, no new drift introduced.

| Finding | Severity | Status |
|---------|----------|--------|
| FINDING-001 (ADR-009 Status block) | HIGH | CLOSED |
| FINDING-002 (epics.md arithmetic) | HIGH | CLOSED |
| FINDING-003 (prd.md §7 RTM) | MEDIUM | CLOSED |
| FINDING-004 (BC-2.12.011 annotation) | MEDIUM | CLOSED |
| FINDING-005 (BC-INDEX timestamp) | LOW | CLOSED |
| FINDING-006 (HS-001 / HS-INDEX staleness) | LOW | CLOSED |

The F2 spec burst is internally consistent. The 4 known-open F3 items (BC-2.12.011 rewrite, HS-001 rewrite, STORY-123-127 input-hash generation, VP assignments for BC-2.01.009-018) remain correctly tracked in STATE.md and cycle-manifest and are not blocking F2 closure.

---

## Focused Re-Audit: F-06/F-07/F-08/F-11 Completeness Deltas

**Re-audit date:** 2026-06-19
**Auditor:** consistency-validator
**Scope:** NARROW — only the F-06/F-07/F-08/F-11 delta just applied (E-INP-012 addition, BC-2.01.010 v1.1, BC-2.01.015 v1.1, BC-2.01.018 v1.1, error-taxonomy v2.4, prd v1.31, ADR-009 rev 2). The broader F2 surface was CLEAN per the prior pass; this pass does not re-audit it.

---

### Check 1 — E-INP-012 Integrity

**Source:** error-taxonomy.md v2.4, line 79.

Results:

- **Present:** E-INP-012 row exists in the INP table. PASS.
- **Sequential:** Table runs E-INP-008, 009, 010, 011, 012. No collision with E-INP-008..011 (all assigned in v2.3). No gap. PASS.
- **Fields complete:** Error Code, Category (`Input`), Severity (`broken`), Exit Code (`1`), Source Location (`src/reader.rs` pcapng SHB re-encounter check), Message Format (contains `block #<seq>` identifier), BC Ref (`BC-2.01.010, BC-2.01.017`), Notes (explains single-section scope, no byte-order reset, directory-mode per-file isolation). PASS.
- **BC-2.01.010 exists:** Confirmed on disk at `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md`. PASS.
- **BC-2.01.017 exists:** Confirmed on disk at `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-01/BC-2.01.017.md`. PASS.
- **next_free = E-INP-013:** v2.4 changelog states "next_free_error_code = E-INP-013." v2.3 stated "next_free_error_code = E-INP-012." Transition is correct: v2.3 consumed nothing (E-INP-012 was still free), v2.4 adds E-INP-012 and advances free pointer to E-INP-013. PASS.
- **Directory-mode note consistent:** E-INP-012 Notes state "In directory mode, this error fails the individual file but does NOT abort the overall run." This matches BC-2.01.018 AC-002 and E-INP-011 Notes (same per-file isolation language). PASS.
- **Minor observation — BC-2.01.017 Error Taxonomy field:** BC-2.01.017 Traceability section (line 104) lists `E-INP-008, E-INP-009, E-INP-010, E-INP-011` and does not include E-INP-012. BC-2.01.017 was written before F-06 and was not bumped as part of this delta. The omission is LOW severity: BC-2.01.010 (not BC-2.01.017) is the normative home for E-INP-012 per ADR-009 Decision 7, and E-INP-012 already references BC-2.01.017 in its BC Ref column (bidirectional trace exists from the taxonomy side). However, BC-2.01.017's own traceability table is incomplete for E-INP-012. This is a cosmetic doc gap, not a semantic one — BC-2.01.017's invariant is that all pcapng parse errors surface via anyhow chain; E-INP-012 does so. **Severity: LOW (cosmetic). Not blocking.**

**Check 1 verdict: PASS with one LOW observation.**

---

### Check 2 — F-06 Alignment: BC-2.01.010 v1.1 ↔ ADR-009 Decision 7

**BC-2.01.010 v1.1 verified items:**

- **AC-002 present:** "A second Section Header Block encountered anywhere after the first is REJECTED with `Err` containing context that maps to E-INP-012." Present and explicit. PASS.
- **E-INP-012 reference in AC-002:** "context that maps to E-INP-012" — correct. PASS.
- **"No byte-order reset" language:** AC-002 states "The second SHB's byte-order reset MUST NOT be applied before rejection." Invariant 1 repeats: "Attempting to reset byte order on a second SHB is NOT permitted." EC-006 states "No byte-order reset is attempted." **No leftover "resets byte order" language remains.** PASS.
- **EC-006 updated:** Previously described the old behavior (attempt reset); now reads "Err mapping to E-INP-012: 'pcapng multi-section files are not supported (second Section Header Block at block #<seq>)'; wirerust supports single-section pcapng only. No byte-order reset is attempted." Correct. PASS.
- **Canonical test vector added:** Row "Crafted 2-section pcapng (SHB₁ + IDB + EPB + SHB₂) | `Err` (E-INP-012) after SHB₁ section; no packets from section 2 | error" is present. PASS.
- **Error Taxonomy in Traceability section:** "E-INP-008 (truncated SHB), E-INP-012 (multi-section SHB reject — single-section scope)" — both present, E-INP-012 correctly attributed. PASS.
- **Modified changelog in v1.1 frontmatter:** "v1.1: F-06 completeness delta — EC-006 changed from 'reset byte order' (attempt) to REJECT with E-INP-012; AC added: second SHB in a single-section file is rejected; canonical test vector added for 2-section pcapng; error taxonomy cross-reference E-INP-012 added. — 2026-06-19." Accurate description of what changed. PASS.

**ADR-009 Decision 7 cross-check:**

- Decision 7 states: "wirerust supports single-section pcapng files only... the reader MUST return an error (`E-INP-012`, 'multi-section pcapng not supported')... The normative BC home for this acceptance criterion is BC-2.01.010." PASS — BC-2.01.010 AC-002 is that normative home.
- Decision 7 states "attempting to read a multi-section file with an implementation that does not correctly reset per-section state would silently mis-attribute packets from later sections." BC-2.01.010 AC-002 states the second SHB's byte-order reset "MUST NOT be applied before rejection." The two documents agree: no reset attempted, just reject. PASS.
- ADR-009 Consequences (negative) states: "wirerust rejects multi-section pcapng files with `E-INP-012` (Decision 7, F-06)." Consistent with BC-2.01.010 AC-002 and E-INP-012 entry. PASS.
- **No contradiction detected** between ADR-009 Decision 7 and BC-2.01.010 v1.1. PASS.

**Check 2 verdict: PASS — clean alignment, no contradiction.**

---

### Check 3 — F-07: BC-2.01.015 v1.1 Skip-Arm Enumeration

**Skip-arm variants enumerated in AC-001:**

- `NameResolutionBlock` (NRB, type `0x00000004`) — silently skipped. Present. PASS.
- `InterfaceStatisticsBlock` (ISB, type `0x00000005`) — silently skipped. Present. PASS.
- `DecryptionSecretsBlock` (DSB, type `0x0000000A`) — silently skipped. Present. PASS.
- `SystemdJournalExportBlock` (type `0x00000009`) — silently skipped. Present. PASS.
- Obsolete Packet Block (OPB, type `0x00000002`) — present with explicit note: "carries captured packet data **but** is an obsolete/deprecated block type superseded by EPB; wirerust treats it as out-of-scope and skips it silently." Present. PASS.
- Unknown/future block types — silently skipped via `block_total_length`. Present. PASS.

**OPB note consistency with ADR-009 F-08 wording:**

- BC-2.01.015 AC-001 OPB note: "OPB packet data is intentionally NOT ingested. Captures relying solely on OPB (very old tcpdump versions) will yield zero packets from those blocks." 
- ADR-009 Decision 2 OPB paragraph: "OPB is marked obsolete in the pcapng specification... OPB support is out of scope for this cycle." Consequences section: "Obsolete Packet Block packets are silently skipped, not read... Any pcapng file captured by legacy tooling that emits OPB instead of EPB will appear to contain zero packets."
- **Both say: skip silently, no packets ingested, out-of-scope.** No contradiction. PASS.

**Invariant 2 updated:** Lists "NameResolutionBlock (NRB), InterfaceStatisticsBlock (ISB), DecryptionSecretsBlock (DSB), SystemdJournalExportBlock, obsolete Packet Block (OPB, type `0x00000002`), and all unknown/future block types." All six skip categories present. PASS.

**AC-002 no-diagnostic invariant:** "For each variant above, the skip MUST NOT emit any warning, error, or finding." Consistent with ADR-009 Decision 2 ("neither a warning nor an error is emitted for unknown types"). PASS.

**Modified changelog in v1.1 frontmatter:** "v1.1: F-07 completeness delta — explicitly enumerate all pcap-file Block variants that fall through to the skip path (NRB, ISB, DSB, SystemdJournalExport, obsolete Packet Block 0x2, Unknown); note that obsolete Packet Block 0x2 carries packet data but is treated as out-of-scope/skipped; add AC to prevent omitted match arm at implementation. — 2026-06-19." Accurate. PASS.

**Check 3 verdict: PASS — OPB note consistent with ADR-009 F-08, no contradiction.**

---

### Check 4 — F-11: BC-2.01.018 v1.1 Directory-Mode Per-File Isolation

**AC-002 content:**

- States: "In directory mode (`--target <dir>`), a pcapng file that fails with E-INP-011 (multi-IDB link-type conflict) MUST fail PER-FILE only. The remaining files in the directory continue to be processed. The overall run exit code is non-zero (exit 1) to indicate at least one file failed, but the run MUST NOT abort at the first conflicting file."
- Cross-reference: "per-file error isolation is the directory-mode contract from BC-2.12.011 and applies to all file-level ingestion errors including E-INP-011."

**BC-2.12.011 cross-reference validation:**

- BC-2.12.011 is at `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-12/BC-2.12.011.md`. File exists. PASS.
- BC-2.12.011 governs `resolve_targets` (directory expansion to sorted file list). It expands a directory to individual file paths that are then iterated by main.rs's capture loop. The per-file error isolation (each file's error does not abort the overall run) is a property of the capture loop iteration logic, not of `resolve_targets` itself. BC-2.01.018 AC-002 correctly identifies this: "per-file error isolation is a property of the main.rs capture loop when iterating over resolved targets."
- **Precision check:** BC-2.01.018 AC-002 says "This per-file error isolation is the directory-mode contract from BC-2.12.011." BC-2.12.011 describes the directory expansion (which files are returned), not the loop's error-handling posture. The cross-reference is loose but not wrong — BC-2.12.011 is the BC governing directory mode, and the isolation behavior is a natural extension of that scope. The statement is architecturally accurate: once `resolve_targets` hands back the file list, the loop's per-file error handling is the implementation of directory-mode isolation. The reference is functional, not dangling. PASS.
- **No dangling cross-ref:** BC-2.12.011 exists on disk, is active (lifecycle_status: active), and is the correct SS-12 directory-mode BC. The F3 forward-action note in BC-2.12.011 describes the planned *.pcapng glob change but does not affect the per-file isolation contract. PASS.

**E-INP-011 message actionable hint:**

- E-INP-011 Notes (error-taxonomy.md line 78) state: "The actionable hint in the message identifies the most common real-world trigger (`tcpdump -i any` mixing Ethernet + Linux Cooked) and states the remediation (single link type required)." This matches the F-11 delta intent.
- BC-2.01.018 AC-001 describes the message format: "(a) identifies the conflicting link types by `DataLink` Debug repr and (b) includes a hint that this commonly arises from `tcpdump -i any` captures mixing link types, and that wirerust requires a single link type per file. The exact message format is defined by E-INP-011." Consistent. PASS.

**EC-009 added to BC-2.01.018 Edge Cases:** "Directory with file_a.pcapng (ETHERNET+LINUX_SLL conflict) and file_b.pcapng (ETHERNET only) | E-INP-011 on file_a; file_b processed successfully; overall exit code 1 (at least one failure)." This is the canonical directory-mode isolation test case. Present. PASS.

**Modified changelog in v1.1 frontmatter:** "v1.1: F-11 completeness delta — (1) Add AC for directory-mode per-file error isolation: E-INP-011 on one file does not abort the full run; (2) Add AC clarifying common user trigger (tcpdump -i any) in E-INP-011 message; cross-reference BC-2.12.011 directory-mode isolation. — 2026-06-19." Accurate. PASS.

**Check 4 verdict: PASS — BC-2.12.011 cross-reference is functional and not dangling.**

---

### Check 5 — No Count Drift

**ss-01 BC files on disk:** 18 BC files (BC-2.01.001..018) + 1 staging artifact (ERROR-TAXONOMY-ADDENDUM-pcapng.md) = 19 files. No new BC files were added by the F-06/F-07/F-08/F-11 delta (those deltas are AC-level amendments to existing BCs, not new BCs). PASS.

**BC-INDEX active count:** Header states "Active: 302 BCs (303 on disk − 1 retired: BC-2.01.004)." The three amended BCs (BC-2.01.010, BC-2.01.015, BC-2.01.018) are all [WRITTEN] active rows; no rows were added or removed from BC-INDEX. PASS.

**epics.md total_bcs:** Frontmatter line 14: `total_bcs: 302`. Consistent with BC-INDEX. PASS.

**BC-INDEX annotation comments for amended BCs:** The inline comments for BC-2.01.010, BC-2.01.015, and BC-2.01.018 still read `v1.0` (not updated to reflect v1.1 version bumps). This is a cosmetic annotation gap — the BC body files themselves have correct v1.1 frontmatter, and BC-INDEX comments are non-normative change-tracking annotations, not version-of-record. **Severity: LOW (cosmetic). Not blocking.**

**Check 5 verdict: PASS with one LOW observation.**

---

### Check 6 — Version Monotonicity

| Artifact | Prior version | Current version | Monotonic? | Changelog present? |
|----------|--------------|-----------------|------------|-------------------|
| BC-2.01.010 | v1.0 | v1.1 | YES | YES — "v1.1: F-06 completeness delta..." |
| BC-2.01.015 | v1.0 | v1.1 | YES | YES — "v1.1: F-07 completeness delta..." |
| BC-2.01.018 | v1.0 | v1.1 | YES | YES — "v1.1: F-11 completeness delta..." |
| error-taxonomy.md | v2.3 | v2.4 | YES | YES — "v2.4: F-06/F-11 pcapng completeness deltas..." |
| prd.md | v1.30 | v1.31 | YES | YES — "Version 1.31 delta (2026-06-19 — pcapng completeness deltas F-06/F-07/F-11)..." |
| ADR-009 | rev 1 (implied) | rev 2 | YES | YES — "Rev 2 (2026-06-19): Added Decision 7..." |

All version bumps are monotonic. All changelog entries are present and accurately describe the normative changes made. PASS.

**One note on ADR-009 versioning:** ADR-009 uses a prose "rev 2" annotation in the Status section rather than a `version:` frontmatter field (the ADR frontmatter uses `date:` and `status:`, not `version:`). This is consistent with the existing ADR template for this project — ADRs are not semantically versioned via frontmatter. No issue.

**Check 6 verdict: PASS.**

---

### Re-Audit Summary

| Check | Scope | Result | Gaps |
|-------|-------|--------|------|
| 1. E-INP-012 integrity | Presence, sequencing, fields, BC refs, next_free | PASS | LOW: BC-2.01.017 traceability section does not list E-INP-012 (cosmetic) |
| 2. F-06 alignment | BC-2.01.010 AC-002 ↔ ADR-009 Decision 7; no contradiction | PASS | None |
| 3. F-07 skip-arm enumeration | BC-2.01.015 v1.1 NRB/ISB/DSB/SystemdJournal/OPB/Unknown list; OPB consistency with F-08 | PASS | None |
| 4. F-11 per-file isolation | BC-2.01.018 AC-002; BC-2.12.011 cross-ref validity | PASS | None |
| 5. Count drift | ss-01 file count, BC-INDEX active count, epics.md total_bcs | PASS | LOW: BC-INDEX inline comments for 3 amended BCs still show v1.0 annotation |
| 6. Version monotonicity | All 6 artifacts; changelog entries | PASS | None |

**LOW gaps identified (2):**

- **LOW-A:** BC-2.01.017 Error Taxonomy field lists `E-INP-008..011` but not E-INP-012. BC-2.01.017 was not bumped as part of the F-06 delta. The gap is cosmetic — E-INP-012 cross-references BC-2.01.017 in the taxonomy table, preserving the bidirectional trace. Remediation: bump BC-2.01.017 to v1.1 and add E-INP-012 to its Error Taxonomy field. Not blocking.
- **LOW-B:** BC-INDEX inline annotation comments for BC-2.01.010, BC-2.01.015, and BC-2.01.018 still read `v1.0` (were not updated when the BCs were bumped to v1.1). BC-INDEX comments are non-normative. Remediation: update three inline comments from `v1.0` to `v1.1` in a cleanup pass. Not blocking.

**No CRITICAL, MAJOR, or MEDIUM findings.**

---

### Re-Audit Final Verdict

**CLEAN** — all F-06/F-07/F-08/F-11 delta cross-references are consistent. No blocking gaps. Two LOW cosmetic observations recorded above; neither blocks F3 entry.

| Finding | Severity | Blocking? |
|---------|----------|-----------|
| LOW-A: BC-2.01.017 missing E-INP-012 in traceability | LOW | No |
| LOW-B: BC-INDEX inline comment version annotations stale for 3 BCs | LOW | No |

---

## Focused Re-Audit: Rationale Correction (multi-section decision)

**Re-audit date:** 2026-06-19
**Auditor:** consistency-validator
**Scope:** NARROW — verify the corrected multi-section rationale is consistent across ADR-009 (rev 3), BC-2.01.010 (v1.2), and error-taxonomy.md (v2.5). Confirm no surviving false-premise language outside clearly-marked SUPERSEDED blocks. The broader F2 surface was CLEAN per all prior passes; this pass does not re-audit it.

---

### Check R1 — No Surviving False-Premise Language

**Criteria:** No document outside an explicitly marked SUPERSEDED block may present the following as current fact: that pcap-file 2.0.0 "accumulates" interfaces, "does not reset per section," has "unverified" per-section behavior, or "would mis-attribute" EPB interface IDs as a JUSTIFICATION for the reject decision.

**ADR-009 scan:**

- Line 136-137 (Decision 7 body): "Silent mis-attribution across sections is NOT a risk from this crate; the original F-06 premise that it 'accumulates IDBs with no per-section reset' was an inference from API shape and is superseded by source-level verification." This is corrected language — it explicitly refutes the false premise. PASS.
- Line 288-292 (Rev 3 changelog): "The original rationale claimed `pcap-file` 2.0.0 accumulates IDBs without per-section reset and would silently mis-attribute packets across sections (F-06, 2026-06-19). This premise is FALSE." This is the Rev 3 correction record, presented as a historical correction, not as a current claim. PASS.
- Line 264-271 (Consequences, negative trade-offs): "wirerust rejects multi-section pcapng files with `E-INP-012` (Decision 7, F-06 superseded). Files produced by raw concatenation will not be read. This is a scope-discipline choice: `pcap-file` 2.0.0 handles per-section interface reset correctly (verified from source 2026-06-19), so the reject is not a safety necessity — it is a deliberate deferral." Correctly frames reject as scope, not distrust. PASS.
- No surviving unqualified claim that the library fails to reset per section. PASS.

**BC-2.01.010 v1.2 scan:**

- AC-002: "wirerust supports single-section pcapng files only (scope decision for this cycle — multi-section is rare and absent from the intended corpus; pcap-file 2.0.0 itself handles multi-section correctly at the library level, but wirerust does not exercise that path)." Correct framing; no distrust-of-library language. PASS.
- Invariant 1: "This is a scope decision — multi-section pcapng is rare and absent from the intended corpus; pcap-file 2.0.0 handles multi-section correctly at the library level, but wirerust does not exercise that path." PASS.
- EC-006: "wirerust supports single-section pcapng only (scope decision; pcap-file 2.0.0 handles multi-section correctly but wirerust does not exercise that path)." PASS.
- Error Taxonomy field in Traceability: "E-INP-012 (multi-section SHB reject — scope decision; pcap-file 2.0.0 handles multi-section correctly; wirerust rejects as out-of-scope; message includes mergecap/editcap remediation hint)." PASS.
- No surviving language claiming the library is unverified or would mis-attribute. PASS.

**error-taxonomy.md v2.5 scan:**

- E-INP-012 Notes: "wirerust supports single-section pcapng only (scope decision for this cycle — multi-section pcapng is rare and absent from the intended corpus); the rejection is a scope constraint, not a correctness workaround (pcap-file 2.0.0 itself correctly resets interface state per section, per source-level verification 2026-06-19)." Correct. PASS.
- No surviving distrust-of-library language anywhere in the INP section. PASS.

**completeness report (pcapng-spec-completeness-validation.md):**

- F-06 finding body (lines 105-119): Contains original INCONCLUSIVE text ("accumulates IDBs in one growing interface list," "no visible per-section reset"). This text is inside a clearly-marked SUPERSEDED block beginning at line 103 with the banner "**[SUPERSEDED PREMISE — 2026-06-19]**" and the statement "The remainder of this finding is preserved verbatim for audit purposes." Permitted by the task's exemption rule. PASS.
- Line 205 ("INCONCLUSIVE / verify in F3"): This line is in the standalone "What was confirmed vs. inconclusive" summary section, NOT inside the F-06 SUPERSEDED block. It still reads: "INCONCLUSIVE / verify in F3: whether `pcap-file` 2.0.0 actually resets interface state across multiple SHBs (API/source reading strongly suggests NOT, but no runtime test was run — F-06)." This is presented as a current-state summary item, not as historical audit-trail text inside a SUPERSEDED block. It has not been updated to reflect the source-level verification result. **GAP — see Finding RC-1 below.**
- High-Risk Trap Scorecard (line 179): "#3 Multi-section files | GAP (parser may not reset per-section) — BC text correct, delivery uncertain | Add AC: reject-or-first-section + verify pcap-file behavior (F-06)." This remains in the scorecard as a GAP with "parser may not reset per-section" language, outside any SUPERSEDED block. **GAP — see Finding RC-1 below.**

**Check R1 verdict: PASS for ADR-009, BC-2.01.010, error-taxonomy. GAP in completeness report summary section and scorecard (see RC-1).**

---

### Check R2 — Tri-Document Agreement on Corrected Rationale

**Criteria:** ADR-009 Decision 7 (rev 3), BC-2.01.010 AC-002 (v1.2), and E-INP-012 (v2.5) must all (a) frame reject as a SCOPE decision, (b) explicitly acknowledge pcap-file 2.0.0 resets correctly, and (c) reference a future-cycle support escape hatch.

**Scope framing:**

| Document | Scope-decision language | Present? |
|----------|------------------------|---------|
| ADR-009 Decision 7 (rev 3) | "Reject is therefore a *scope* decision, not a distrust-of-library decision." | YES |
| BC-2.01.010 AC-002 (v1.2) | "wirerust supports single-section pcapng files only (scope decision for this cycle)" | YES |
| E-INP-012 Notes (v2.5) | "the rejection is a scope constraint, not a correctness workaround" | YES |

PASS — all three use scope-decision framing.

**pcap-file 2.0.0 resets correctly acknowledgement:**

| Document | Acknowledgement language | Present? |
|----------|------------------------|---------|
| ADR-009 Decision 7 (rev 3) | "`pcap-file` 2.0.0 *does* correctly reset the interface table per section (`self.interfaces.clear()` on every `Block::SectionHeader`...) — confirmed by direct source inspection..." | YES |
| BC-2.01.010 AC-002 (v1.2) | "pcap-file 2.0.0 itself handles multi-section correctly at the library level, but wirerust does not exercise that path" | YES |
| E-INP-012 Notes (v2.5) | "pcap-file 2.0.0 itself correctly resets interface state per section, per source-level verification 2026-06-19" | YES |

PASS — all three acknowledge the library resets correctly.

**Future-cycle escape hatch:**

| Document | Future-cycle language | Present? |
|----------|--------------------|---------|
| ADR-009 Decision 7 (rev 3) | "If a real user requirement appears, SUPPORT is cheap: the crate already surfaces `Block::SectionHeader`... dropping the reject branch plus ~10-60 LOC of cross-section linktype-agreement is all that is needed." | YES |
| BC-2.01.010 AC-002 (v1.2) | "pcap-file 2.0.0 itself handles multi-section correctly at the library level" (implicit that enabling it is cheap) | PARTIAL — the escape-hatch LOC estimate is not repeated in BC-2.01.010, but the path is implicitly acknowledged. Acceptable: BC-2.01.010 is the normative AC document, not the rationale vehicle; the escape hatch belongs in the ADR. |
| E-INP-012 Notes (v2.5) | No explicit "~10-60 LOC" note. Notes state reject is scope constraint. | PARTIAL — same reasoning applies; the error taxonomy is not the rationale vehicle. |

PASS — escape hatch is fully present in ADR-009 (the rationale document); BC and taxonomy correctly defer rationale to the ADR.

**Check R2 verdict: PASS — tri-document rationale agreement is consistent. No contradiction among the three.**

---

### Check R3 — Decision Unchanged (Second SHB Reject)

**Criteria:** All three documents still specify second SHB → reject → E-INP-012, no byte-order reset before rejection, exit 1, directory-mode per-file isolation.

| Property | ADR-009 Decision 7 | BC-2.01.010 AC-002 | E-INP-012 |
|----------|-------------------|-------------------|-----------|
| Second SHB → reject | YES ("MUST return an error") | YES ("is REJECTED with Err") | YES ("Emitted when a second SHB is encountered") |
| Error code E-INP-012 | YES (named explicitly) | YES ("maps to E-INP-012") | YES (this is the E-INP-012 row) |
| No byte-order reset before rejection | YES ("rather than attempting per-section interface-index reset") | YES ("The second SHB's byte-order reset MUST NOT be applied before rejection") | YES ("No byte-order reset is attempted before rejection") |
| Exit 1 | YES (E-INP-012 is `broken` severity) | YES (E-INP-012 reference, maps to broken) | YES (`broken`, exit code `1`) |
| Directory-mode per-file isolation | YES (Consequences: "Users can flatten any multi-section file") | YES (AC-002: references E-INP-012; EC-006 consistent) | YES ("In directory mode, this error fails the individual file but does NOT abort the overall run") |

PASS — the normative decision is identical across all three documents. No contradiction.

**Check R3 verdict: PASS.**

---

### Check R4 — E-INP-012 Message Contains mergecap/editcap Hint

**Criteria:** E-INP-012 message contains the mergecap/editcap remediation hint; ADR-009 and BC-2.01.010 consistently describe this hint.

**E-INP-012 message format (error-taxonomy.md line 80):**
`pcapng multi-section files are not supported (second Section Header Block at block #<seq>) (hint: split the capture into single-section files, or re-save with 'mergecap -F pcapng' or 'editcap' which emit single-section pcapng)`

Contains: "mergecap -F pcapng", "editcap". PASS.

**ADR-009 Decision 7 (lines 152-154):** "The `E-INP-012` message SHOULD hint at the remediation: `mergecap -w out.pcapng <file>` or `editcap` flattens any multi-section file to single-section." ADR-009 uses `-w` flag form; E-INP-012 uses `-F pcapng` form. Both are valid `mergecap` invocations; `-F pcapng` is more explicit about the output format, which is preferable. The discrepancy is in flag spelling, not in the tool or intent. Both say mergecap and editcap produce single-section output. The normative message is in E-INP-012 (the error taxonomy); the ADR uses SHOULD language, not MUST. **Minor — see Finding RC-2 below.**

**BC-2.01.010 AC-002 (v1.2):** "The E-INP-012 error message includes an actionable remediation hint directing users to `mergecap -F pcapng` or `editcap` to re-save multi-section captures as single-section files (see E-INP-012 in error-taxonomy.md)." Uses same `-F pcapng` flag form as E-INP-012. Consistent with E-INP-012. PASS.

**ADR-009 Consequences (negative) (line 271):** "Users can flatten any multi-section file with `mergecap -w out.pcapng <file>` (the E-INP-012 message should hint at this)." Again uses `-w` form. Same minor discrepancy with E-INP-012 and BC-2.01.010.

**Check R4 verdict: PASS on substance. Minor flag-spelling inconsistency between ADR-009 and the other two documents — see Finding RC-2.**

---

### Check R5 — Structural/Count Checks

**ss-01 BC range:** Files on disk: BC-2.01.001 through BC-2.01.018 (18 files) plus ERROR-TAXONOMY-ADDENDUM-pcapng.md (staging artifact). No BC-2.01.019 or higher. ss-01 ends at BC-2.01.018. PASS.

**Active BC count:** BC-INDEX states "Active: 302 BCs (303 on disk − 1 retired: BC-2.01.004)." No new BCs were introduced by the rationale correction (ADR-009 rev 3, BC-2.01.010 v1.2, error-taxonomy v2.5 are all amendments, not new artifacts). Count unchanged at 302. PASS.

**error-taxonomy next_free:** The `next_free_error_code` field appears only in changelog annotations, not as a standalone body field. The v2.4 changelog entry states `next_free_error_code = E-INP-013`. The v2.5 changelog does not change the next-free pointer (v2.5 only amends E-INP-012's Notes text and message; it adds no new error code). E-INP-013 is not present in the catalog (correct — it is reserved, not yet assigned). PASS.

**No new BCs introduced:** The rationale correction touches only ADR-009 (rev 3 prose), BC-2.01.010 (v1.2 AC/Invariant/EC text), and error-taxonomy.md (v2.5 E-INP-012 Notes). No new BC IDs, no new error codes, no new stories. PASS.

**Check R5 verdict: PASS.**

---

### Check R6 — Version Monotonicity

| Artifact | Prior version | Current version | Monotonic? | Changelog present? |
|----------|--------------|-----------------|------------|-------------------|
| ADR-009 | rev 2 | rev 3 | YES | YES — "Rev 3 (2026-06-19): Corrected the factual error in Decision 7's rationale..." |
| BC-2.01.010 | v1.1 | v1.2 | YES | YES — "v1.2: pcapng-multisection-decision correctness edits — AC-002 rationale reframed..." |
| error-taxonomy.md | v2.4 | v2.5 | YES | YES — "v2.5: pcapng-multisection-decision correctness edits — E-INP-012 message updated..." |

All three version bumps are monotonic. All changelog entries are present and accurately describe the normative changes made. PASS.

**Check R6 verdict: PASS.**

---

### Findings from This Pass

#### Finding RC-1 — LOW

**Two locations in completeness report (pcapng-spec-completeness-validation.md) present the false premise outside any SUPERSEDED block.**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/research/pcapng-spec-completeness-validation.md`

**Location 1:** Line 205, "What was confirmed vs. inconclusive" summary section:
> "INCONCLUSIVE / verify in F3: whether `pcap-file` 2.0.0 actually resets interface state across multiple SHBs (API/source reading strongly suggests NOT, but no runtime test was run — F-06)."

**Location 2:** Lines 178-181, High-Risk Trap Scorecard, row #3:
> "#3 | Multi-section files | GAP (parser may not reset per-section) — BC text correct, delivery uncertain | Add AC: reject-or-first-section + verify pcap-file behavior (F-06)"

These two items are presented as the document's current summary conclusions — they are not inside the F-06 SUPERSEDED block (which covers only lines 103-119). The F-06 SUPERSEDED block header correctly supersedes the F-06 finding body, but does NOT supersede these summary-section lines, which repeat the now-false INCONCLUSIVE/GAP status in the document's own conclusion section.

**Severity: LOW.** The completeness report is a research artifact, not a normative spec. The three normative documents (ADR-009, BC-2.01.010, error-taxonomy) have all been correctly updated. The research document's summary is stale but does not drive implementation. The F-06 SUPERSEDED block is correctly placed to flag the finding text; the summary section was simply not updated as a parallel step.

**Not blocking F3 entry.** The normative chain is clean.

**Remediation (optional cleanup):** In the "What was confirmed vs. inconclusive" section, replace the INCONCLUSIVE bullet with a SUPERSEDED reference: "**SUPERSEDED (2026-06-19):** F-06 determined INCONCLUSIVE — source-level verification in `pcapng-multisection-decision.md` confirms pcap-file 2.0.0 resets correctly. Reject (E-INP-012) retained as a scope decision. See ADR-009 rev 3." Similarly update the Scorecard row #3 Status cell to "SUPERSEDED — pcap-file 2.0.0 resets correctly (source-verified 2026-06-19); reject is scope decision, not library defect."

---

#### Finding RC-2 — LOW

**ADR-009 uses `mergecap -w out.pcapng <file>` while BC-2.01.010 v1.2 and E-INP-012 v2.5 use `mergecap -F pcapng`.**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md`
**Lines:** 152 (Decision 7) and 271 (Consequences)

**Discrepancy:** `-w out.pcapng` is a valid `mergecap` invocation (specifies output filename and infers format from extension). `-F pcapng` explicitly names the output format. BC-2.01.010 AC-002 and E-INP-012 use `-F pcapng`, which is the more descriptive form and will appear in the actual error message shown to users.

**Severity: LOW.** Both invocations produce single-section pcapng output. The ADR text uses SHOULD language ("SHOULD hint at the remediation") and the normative message is defined in E-INP-012. No implementer will be misled. The discrepancy is cosmetic flag-spelling only.

**Not blocking.**

**Remediation (optional):** In ADR-009 lines 152 and 271, replace `mergecap -w out.pcapng <file>` with `mergecap -F pcapng -w out.pcapng <file>` to align with the E-INP-012 message text and BC-2.01.010 AC-002.

---

### Re-Audit Summary

| Check | Scope | Result | Gaps |
|-------|-------|--------|------|
| R1. No surviving false-premise language | ADR-009, BC-2.01.010, error-taxonomy, completeness report | PASS with LOW gap | RC-1: completeness report summary section and scorecard not updated outside SUPERSEDED block |
| R2. Tri-document rationale agreement | Scope framing, library-reset ack, escape hatch | PASS | None blocking |
| R3. Decision unchanged | Second SHB → reject → E-INP-012; no reset; exit 1; per-file isolation | PASS | None |
| R4. E-INP-012 message remediation hint | mergecap/editcap present; cross-doc consistency | PASS with LOW gap | RC-2: ADR-009 uses `-w` flag vs. `-F pcapng` in BC and taxonomy |
| R5. Count and structural integrity | ss-01 ends at 018; active count 302; next_free E-INP-013 | PASS | None |
| R6. Version monotonicity | ADR-009 rev 3, BC-2.01.010 v1.2, error-taxonomy v2.5 | PASS | None |

**Findings from this pass:**

| Finding | Severity | Blocking? |
|---------|----------|-----------|
| RC-1: Completeness report summary/scorecard stale (not inside SUPERSEDED block) | LOW | No |
| RC-2: ADR-009 mergecap flag spelling differs from E-INP-012 and BC-2.01.010 | LOW | No |

---

### Re-Audit Final Verdict

**CLEAN** — the rationale correction is internally consistent across all three normative documents. No blocking gaps. Two LOW cosmetic observations recorded; neither blocks F3 entry.

The three normative documents agree on:
- Reject is a scope decision, not a library-distrust decision
- pcap-file 2.0.0 resets interface state per section correctly (source-verified 2026-06-19)
- Second SHB → E-INP-012 → exit 1 → per-file isolation in directory mode
- E-INP-012 message includes mergecap/editcap remediation hint

The two LOW findings (RC-1, RC-2) are cosmetic inconsistencies in the research document and ADR rationale prose respectively; they do not affect implementation guidance.

---

## Fresh-Context F2 Remediation Re-Audit — ADR-009 rev 4 / D-142 Burst

**Audit date:** 2026-06-19
**Auditor:** consistency-validator (fresh-context pass — no inherited session state)
**Scope:** Full cross-document coherence check after three parallel PO bursts (A/B/C) plus
architect ADR-009 rev 4 and holdout authoring (D-142). Covers:
- Priority check: SHB Byte-Order Magic correctness and no conflation with block-type magic
- E-INP-009/010 remap coherence
- VP-025..030 traceability (BC cells, VP-INDEX, PRD RTM, holdout BC columns)
- Timestamp totality (H-1), SPB/EPB overhead math (H-2), EC-004 forward-progress (SEC-002)
- Skip-arm/DSB (M-2), magic-byte glob (C-2), per-file isolation re-attribution (C-1)
- HS-INDEX integrity for HS-101..106
- Spot-verification of all CRITICAL/HIGH items from the remediation tracker marked FIXED

---

### PRIORITY CHECK — Byte-Order Magic Correctness and No-Conflation

The pcapng specification defines two distinct magic fields:

1. **SHB block-type magic:** `0x0A0D0D0A` — the first 4 bytes of every pcapng file,
   identifying the block type. This value is endian-neutral (palindromic bytes).
2. **SHB Byte-Order Magic (BOM):** a 4-byte field inside the SHB body. The canonical
   value is the u32 `0x1A2B3C4D`. In a LITTLE-endian file, these 4 bytes appear on disk
   as `4D 3C 2B 1A` (wire value `0x4D3C2B1A`). In a BIG-endian file, these bytes appear
   on disk as `1A 2B 3C 4D` (wire value `0x1A2B3C4D`).

The audit question is: does each document use the correct value for the case it describes,
and does any document confuse the BOM with the block-type magic?

#### BOM Usage Per Document

**ADR-009 (rev 4):**
- Context section, line 52: "The SHB carries a Byte-Order Magic field (`0x1A2B3C4D`) that
  governs endianness for all subsequent fields." This describes the canonical u32 value,
  not the on-wire encoding. Correct as a spec reference.
- Decision 2, SHB entry: "byte-order is determined from the Byte-Order Magic within the
  SHB body before subsequent blocks are read." Correct; does not state a specific hex value.
- Holdout spec in VP table, BC-2.01.010 row: "byte-exact crafted SHB with BE byte-order
  magic `0x4D3C2B1A`." This is the wire value for a BE file. CORRECT: a big-endian file
  has the BOM field encoded as `1A 2B 3C 4D` on disk, which a reader that reads it
  big-endian sees as `0x1A2B3C4D` (match); a reader that reads it little-endian would
  see `0x4D3C2B1A` — detecting the mismatch signals big-endian mode. The wire bytes of a
  BE BOM are `1A 2B 3C 4D`, but the u32 value from those bytes interpreted as BE is
  `0x1A2B3C4D`. ADR-009 calls the BE holdout file "BE byte-order magic `0x4D3C2B1A`"
  which is the literal u32 you read when you misread the BE bytes as LE. This is a
  CORRECT description from the "wire u32 as read in LE" perspective that the parser uses.
- Block-type magic `0x0A0D0D0A` cited at Decision 2, SHB entry. Correct, distinct from BOM.
- VERDICT: ADR-009 is CORRECT and does NOT conflate block-type magic with BOM.

**BC-2.01.010 (v1.4):**
- Description, line 34: "Byte-Order Magic field (`0x1A2B3C4D` LE or `0x4D3C2B1A` BE)"
  — slightly ambiguous shorthand but the Postcondition 1 clarifies:
  "BOM wire value `0x4D3C2B1A` → LE; BOM wire value `0x1A2B3C4D` → BE."
  This is the correct pcapng spec interpretation.
- AC-001: "A well-formed SHB with wire BOM `0x4D3C2B1A` selects little-endian mode; a
  wire BOM `0x1A2B3C4D` selects big-endian mode."
  LE file: 4 bytes `4D 3C 2B 1A`, read as u32-LE = `0x1A2B3C4D` (matches canonical).
  BE file: 4 bytes `1A 2B 3C 4D`, read as u32-LE = `0x4D3C2B1A` (mismatch signals BE).
  So "wire BOM `0x4D3C2B1A` → LE" means: the u32 you read from the 4 wire bytes,
  interpreting them as LE, equals `0x1A2B3C4D`. That matches the canonical BOM and
  confirms LE mode. And "wire BOM `0x1A2B3C4D` → BE" means: the u32 read as LE from
  the wire bytes equals the reversed value — confirming BE mode. CORRECT per spec.
- AC-001 holdout annotation: "SHB with BE magic `0x4D3C2B1A` (wire big-endian encoding
  of 0x1A2B3C4D read big-endian)." This phrase is internally inconsistent: it says
  "wire big-endian encoding" but then says "read big-endian", which would give
  `0x1A2B3C4D` (the canonical match), not `0x4D3C2B1A`. The parenthetical is
  **confusingly worded** — it appears to describe the LE interpretation of BE bytes as
  `0x4D3C2B1A`, then adds "read big-endian" which contradicts the LE-read framing.
  The actual intent is correct (the holdout tests the BE BOM path), but the parenthetical
  explanation is misleading. **GAP-BOM-1 — see Finding BOM-1 below.**
- Invariant 3: "The SHB magic bytes (`0x0A0D0D0A`) are not themselves byte-order-dependent;
  they serve only to identify the block type. The BOM field inside the SHB body carries
  the endianness signal." Correctly distinguishes block-type magic from BOM. PASS.
- VERDICT: BC-2.01.010 is CORRECT on the normative values. One confusingly-worded
  AC-001 parenthetical does not affect correctness but should be clarified.

**HS-103 (v1.0):**
- Case A header: "Byte-exact BE-magic SHB (0x4D3C2B1A)"
- Case A body, BOM field: "`0x4D3C2B1A` — the big-endian sentinel (LE magic reversed)"
- The description "LE magic reversed" is correct: the canonical BOM u32 is `0x1A2B3C4D`;
  reversed = `0x4D3C2B1A`, which is what a LE reader sees when reading a BE BOM field.
  This is the correct wire value for a BE file's BOM bytes interpreted as LE.
- BC linkage table, row 2: "SHB with BE BOM (0x4D3C2B1A) accepted; subsequent fields
  decoded BE | Case A: the BE-magic path must be recognized and used." CORRECT.
- Case B BOM: `0xDEADBEEF` (invalid). CORRECT.
- block_total_length BE encoding (Case A): "0x00000000_1C000000 in big-endian = 28 bytes"
  — note: `0x1C = 28` as a single byte; the u32 `28 = 0x0000001C`. In BE, wire bytes
  are `00 00 00 1C`. HS-103 writes "0x00000000_1C000000" which appears to be an
  8-byte value split incorrectly. **GAP-BOM-2 — see Finding BOM-2 below.**
- Block-type `0x0A0D0D0A` is correctly identified as "same in both endians." PASS.
- No conflation of block-type magic with BOM. PASS.
- VERDICT: HS-103 uses the correct BOM values. One encoding representation for
  block_total_length appears erroneous.

**BC-2.12.011 (v1.5):**
- Lists "pcapng SHB: `0x0A0D0D0A` (bytes: 0A 0D 0D 0A)" — this is the block-type magic,
  not the BOM. Correct context: BC-2.12.011 is about magic-byte content detection
  (first 4 bytes of file), and the first 4 bytes of a pcapng file ARE the block-type
  magic `0x0A0D0D0A`. Does NOT mention the BOM. No conflation. PASS.

**Summary — Magic Correctness:**

| Document | Block-type magic | BOM (LE) | BOM (BE) | Conflation? |
|----------|-----------------|----------|----------|-------------|
| ADR-009 rev 4 | `0x0A0D0D0A` (correct) | `0x1A2B3C4D` (canonical u32) | `0x4D3C2B1A` (wire-LE read of BE bytes, correct) | None |
| BC-2.01.010 v1.4 | `0x0A0D0D0A` (Invariant 3) | `0x4D3C2B1A` wire → LE mode (correct) | `0x1A2B3C4D` wire → BE mode (correct) | None |
| HS-103 v1.0 | `0x0A0D0D0A` (Case A intro, correct) | (baseline, not exercised in isolation) | `0x4D3C2B1A` (correct) | None |
| BC-2.12.011 v1.5 | `0x0A0D0D0A` (file detection, correct) | N/A (file detection only) | N/A | None |

**VERDICT: NO conflation of SHB block-type magic (`0x0A0D0D0A`) with Byte-Order Magic
(`0x1A2B3C4D`/`0x4D3C2B1A`) in any document.** Magic values are correctly used for
their respective roles in all four documents. Two presentation-level issues noted below
as BOM-1 (LOW) and BOM-2 (MEDIUM).

---

### CROSS-DOC CHECKS

#### Check 1 — E-INP-009/010 Remap Coherence

**Claim marked FIXED in tracker (H-3/SEC-003, M-3):** empty-table → E-INP-009;
OOB-non-empty → E-INP-010; no doc still routes empty-table → E-INP-008;
E-INP-010 uses one canonical message template.

**Verified on disk:**

- **error-taxonomy.md v2.7 E-INP-009:** "Emitted when an EPB OR SPB is encountered and
  the interface table is EMPTY." BC refs: BC-2.01.012, BC-2.01.013, BC-2.01.017. CORRECT.
- **error-taxonomy.md v2.7 E-INP-010:** Unified canonical message template
  `"Failed to parse pcapng <block-type> (block #<seq>): <underlying>"` covering: (a) EPB
  interface_id OOB on NON-EMPTY table, (b) captured_len > btl-32, (c) EPB body < 20 bytes,
  (d) SPB body < 4 bytes, (e) unknown-block < 12 bytes. ONE template. CORRECT.
- **error-taxonomy.md v2.7 E-INP-008:** "Covers structural parse failures at the SHB or
  IDB level: truncated file, missing BOM, malformed block-total-length, unsupported major
  version." Scope is SHB/IDB ONLY. NOT used for EPB/SPB. CORRECT.
- **BC-2.01.012 v1.1 PC5:** "EPB with interface_id referencing EMPTY table → E-INP-009.
  EPB with interface_id OOB on NON-EMPTY table → E-INP-010." CORRECT.
- **BC-2.01.012 v1.1 PC6:** "captured_len > btl-32 → E-INP-010." CORRECT.
- **BC-2.01.013 v1.1 PC5:** "SPB with EMPTY interface table → E-INP-009." CORRECT.
- **BC-2.01.013 v1.1 PC6:** "truncated SPB (btl < 16) → E-INP-010." CORRECT.
- **PRD §7 RTM BC-2.01.012 row:** "integration+VP-027 (E-INP-009/010; STORY-125)."
  Both error codes cited. CORRECT.
- **PRD §7 RTM BC-2.01.013 row:** "integration (E-INP-009; STORY-126)." Only E-INP-009
  cited; E-INP-010 (for SPB truncation) is not. This is incomplete but not incorrect
  (the row notes the primary new error code introduced for SPB-before-IDB). LOW gap only.
- **HS-104 Case A:** "EPB with interface_id = u32::MAX with EMPTY interface table →
  E-INP-009." CORRECT. Case B: "EPB with interface_id = u32::MAX on 1-ENTRY table →
  E-INP-010." CORRECT.
- **No document routes empty-table → E-INP-008.** CONFIRMED by search across all docs.
- **E-INP-010 single canonical message template:** Confirmed in error-taxonomy v2.7.
  BC-2.01.012 PC5 uses the exact quoted string `"EPB interface_id={id} out of range
  (table size={n})"` as the `<underlying>` component. Consistent with template. CORRECT.

**VERDICT: PASS — E-INP-009/010 remap is coherent across all documents.**

---

#### Check 2 — VP Coherence (VP-025..030)

**Sub-check 2a: Each VP appears in the owning BC's Verification Properties cell.**

| VP | BC | Cell present? |
|----|-----|--------------|
| VP-025 | BC-2.01.014 v1.1 | YES — "VP-025 | pcapng_timestamp_to_secs_usecs totality..." |
| VP-026 | BC-2.01.010 v1.4 | YES — "VP-026 | SHB parse safety..." |
| VP-027 | BC-2.01.012 v1.1 | YES — "VP-027 | EPB parse safety..." |
| VP-028 | BC-2.01.017 v1.2 | YES — "VP-028 | pcapng reader no-panic (Full Path Fuzz)..." |
| VP-029 | BC-2.01.015 v1.2 | YES — "VP-029 | Block-walk skip correctness..." |
| VP-030 | BC-2.01.018 v1.2 | YES — "VP-030 | Multi-IDB linktype agreement totality..." |

PASS — all 6 VPs present in owning BC cell.

**Sub-check 2b: Each VP appears in VP-INDEX v2.3 catalog.**

VP-INDEX complete catalog rows VP-025..030 confirmed present (verified above).
Tool/phase/status match ADR-009 rev 4 VP table exactly. PASS.

**Sub-check 2c: VP-INDEX arithmetic self-consistency.**

VP-INDEX frontmatter: total_vps=30, p0=8, p1=16, test_sufficient=6, kani=14,
proptest=9, fuzz=2, integration_unit=5.

- 8+16+6 = 30. CORRECT.
- 14+9+2+5 = 30. CORRECT.
- Summary table: Kani(14) + proptest(9) + fuzz(2) + integration/unit(5) = 30. CORRECT.
- P1 list: VP-006..015, VP-022..030 = 16 entries (VP-006, 010, 011, 012, 013, 014, 015,
  022, 023, 024, 025, 026, 027, 028, 029, 030 = 16). CORRECT.

PASS — VP-INDEX arithmetic is internally consistent.

**Sub-check 2d: Each VP appears in PRD §7 RTM.**

PRD §7 RTM rows checked:
- BC-2.01.010 row: "integration+VP-026" — VP-026 present. PASS.
- BC-2.01.012 row: "integration+VP-027" — VP-027 present. PASS.
- BC-2.01.014 row: "integration+VP-025" — VP-025 present. PASS.
- BC-2.01.015 row: "integration+VP-029" — VP-029 present. PASS.
- BC-2.01.017 row: "integration+VP-028/cargo-fuzz" — VP-028 present. PASS.
- BC-2.01.018 row: "integration+VP-030" — VP-030 present. PASS.

PASS — all 6 VPs appear in PRD §7 RTM.

**Sub-check 2e: Each VP appears in the matching holdout HS-10x BC column.**

| HS | VP in frontmatter | CORRECT |
|----|------------------|---------|
| HS-101 | VP-025 | YES (BC-2.01.014, VP-025) |
| HS-102 | VP-025 | YES (BC-2.01.014, VP-025) |
| HS-103 | VP-026 | YES (BC-2.01.010, VP-026) |
| HS-104 | VP-027 | YES (BC-2.01.012, VP-027) |
| HS-105 | VP-029 | YES (BC-2.01.015, VP-029) |
| HS-106 | VP-030 | YES (BC-2.01.018, VP-030) |

PASS.

**Sub-check 2f: Confirm BCs that legitimately have no dedicated VP still show `—`.**

BC-2.01.011 Verification Properties: all rows show `—` with a parenthetical noting
coverage under VP-027. Stated as "covered by VP-027" rather than assigned VP-011.
This is correct per ADR-009 dispatch: "No new VP assigned (covered by VP-027's
interface-table accumulation proof)." PASS.

BC-2.01.013 Verification Properties: all rows show `—` with "Covered under VP-028
(cargo-fuzz) for full no-panic coverage." Correct per ADR-009 dispatch: "No dedicated
Kani VP (VP-028 fuzz covers SPB field misparse)." PASS.

BC-2.01.016 Verification Properties: all rows show `—`, with AC-003 stating "No new
formal VP is assigned to this BC per ADR-009 dispatch." PASS.

**VERDICT: PASS — VP-025..030 coherence is complete across BC cells, VP-INDEX, PRD RTM,
and holdout BC columns.**

---

#### Check 3 — Timestamp Totality (H-1) Verification

**Claim marked FIXED:** BC-2.01.014 v1.1 uses checked_pow / e-clamp-to-63 / u128
intermediate; Invariant "no panic for any (u32,u32,u8)" is now satisfiable; VP-025
references it; HS-101/102 exercise µs(6)/ns(9)/0x94/0xFF.

**BC-2.01.014 v1.1 Postcondition 2 (base-10):**
"`ticks_per_sec = 10u64.checked_pow(e as u32).unwrap_or(u64::MAX)`" — checked_pow
with saturate to u64::MAX. CORRECT; no raw `10u64.pow(e)`.

**BC-2.01.014 v1.1 Postcondition 3 (base-2):**
"e MUST be CLAMPED to [0, 63]... `1u64.checked_shl(e_clamped as u32).unwrap_or(u64::MAX)`"
— clamp then checked_shl. CORRECT; no raw `1u64 << e`.

**BC-2.01.014 v1.1 u128 intermediate:**
Postcondition 2: "`ts_usecs = (((ticks % ticks_per_sec) as u128 * 1_000_000u128) / ticks_per_sec as u128) as u32`"
— explicit u128 cast. CORRECT.

**BC-2.01.014 v1.1 Invariant 1:** "NO PANIC occurs for ANY `(u32, u32, u8)` input.
This invariant is NOW ACTUALLY TRUE (prior formulas using `10u64.pow()` without
`checked_pow` or `1u64 << e` without clamping could panic for large exponents —
those forms are prohibited)." CORRECT; explicitly confirms the invariant is now
satisfiable.

**VP-025 reference:** BC-2.01.014 VP cell: "VP-025 | pcapng_timestamp_to_secs_usecs
totality: no panic for ALL (u32, u32, u8) inputs..." CORRECT.

**ADR-009 rev 4 Decision 4:** Lists checked_pow, e-clamp to [0,63], and u128 intermediate
explicitly. Consistent with BC-2.01.014 v1.1. CORRECT.

**HS-101:** Exercises if_tsresol=6 (Case A) and if_tsresol=9 (Case B). CORRECT.
**HS-102:** Exercises if_tsresol=0xFF (Case A, e=127 must not panic) and if_tsresol=0x94
(Case B, e=20, intermediate overflow territory). CORRECT.

**No residual raw `10u64.pow(e)` or `1u64<<e` found in BC-2.01.014 body.** PASS.

**VERDICT: PASS — H-1 is fully resolved. The timestamp helper spec is overflow-safe and
the Kani invariant is now satisfiable.**

---

#### Check 4 — SPB/EPB Length Math (H-2)

**Claim marked FIXED:** BC-2.01.013 SPB overhead = 4 bytes; BC-2.01.012 EPB overhead =
20 bytes; validation `captured_len <= block_total_length - 32`; no leftover "20" for SPB.

**BC-2.01.013 v1.1:**
- Description: "SPB_FIXED_OVERHEAD_BYTES = 4 (body-relative: original_len field only)"
- Postcondition 1: "available padded-data bytes = block_total_length - 16 (12-byte outer
  header + 4-byte original_len field)"
- AC-004: "SPB_FIXED_OVERHEAD_BYTES MUST equal 4." CORRECT.
- Invariant 4: "SPB_FIXED_OVERHEAD_BYTES = 4 (body-relative: original_len: u32 only).
  The minimum SPB block_total_length is 16 bytes (12 outer + 4 body-fixed)." CORRECT.
- No "20 bytes" for SPB overhead anywhere in BC-2.01.013. PASS.

**BC-2.01.012 v1.1:**
- Description: "EPB_FIXED_OVERHEAD_BYTES = 20 (body-relative: interface_id:4 + ts_high:4
  + ts_low:4 + captured_len:4 + original_len:4)"
- Invariant 5: "EPB_FIXED_OVERHEAD_BYTES = 20 (body-relative)... combined minimum block
  size is 32 bytes (12 + 20)."
- Validation: "captured_len <= block_total_length - 32 (12 outer + 20 body-fixed = 32)"
- CORRECT.

**ADR-009 rev 4 Decision 2:**
- EPB: "EPB_FIXED_OVERHEAD_BYTES = 20 (body-relative, i.e., not counting the outer
  12-byte block header). Validation: captured_len <= block_total_length - 32." CORRECT.
- SPB: "SPB_FIXED_OVERHEAD_BYTES = 4 (body-relative)." CORRECT.
- PO BC-Change Dispatch, BC-2.01.013 section: "Correct SPB body-relative overhead to 4
  bytes... Validation: block_total_length - 16 - 4 = block_total_length - 20 bytes
  available for padded packet data (12 outer + 4 body-fixed = 16 minimum)."

**DISCREPANCY DETECTED:** ADR-009 PO dispatch for BC-2.01.013 writes
"block_total_length - 16 - 4 = block_total_length - 20" — but the formula in
BC-2.01.013 v1.1 PC1 says "available padded-data bytes = block_total_length - 16
(12-byte outer header + 4-byte original_len field)." The ADR dispatch uses
`(btl - 16) - 4 = btl - 20` while the BC says `btl - 16`. One of these is wrong.

Resolving: the 12-byte outer block header (block_type:4 + block_total_length:4 +
trailing_total_length:4) is NOT part of the block body. The `original_len` field is
the sole body-fixed field (4 bytes). Available padded data = btl - 12 (outer header)
- 4 (original_len) = btl - 16. The BC-2.01.013 formula `btl - 16` is CORRECT.

The ADR-009 dispatch formula "block_total_length - 16 - 4 = block_total_length - 20"
erroneously double-subtracts the 4-byte original_len field. The ADR dispatch is a
**PO handoff instruction** (not normative spec), and the normative BC-2.01.013 text
is correct. However, the ADR dispatch's arithmetic is wrong and could mislead.
**GAP-H2-1 — see Finding H2-1 below (LOW).**

**VERDICT: PASS for normative BC content. BC-2.01.013 and BC-2.01.012 are internally
consistent and correct. One LOW erratum in ADR-009 PO dispatch arithmetic.**

---

#### Check 5 — EC-004 / Forward-Progress (SEC-002)

**Claim marked FIXED:** BC-2.01.015 EC-004 = crate rejects block_total_length < 12
(not "8, no error"); forward-progress invariant present; BC-2.01.010 not contradicting.

**BC-2.01.015 v1.2 EC-004:** "block_total_length = 8 | REJECTED by the crate
(block_common.rs:101: threshold is < 12). The crate returns Err(...). The block-walk
loop receives Err(_) and MUST break. The prior characterization 'no error' was INCORRECT
— removed." CORRECT.

**BC-2.01.015 v1.2 EC-005:** "block_total_length < 12 (any value below crate threshold)
| REJECTED by crate with Err; caller breaks on Err." CORRECT.

**BC-2.01.015 v1.2 AC-004 (forward-progress):** "The block-walk loop MUST break (or
return) on any Err(_) from the crate's block reader. The documented rustdoc example
with an empty Err(_) => {} arm is WRONG and MUST NOT be copied." CORRECT.

**BC-2.01.015 v1.2 Invariant 2:** "The block-walk loop MUST break on Err(_). The
crate's cursor does NOT advance on error; breaking is the caller's only obligation."
CORRECT.

**BC-2.01.010 v1.4 Description:** "The crate rejects block_total_length < 12 before
returning any block (forward-progress contract, Decision 8)." CORRECT. No contradiction.

**ADR-009 rev 4 Decision 8:** "malicious block_total_length = 8 is rejected before any
block is returned — the crate does not hand a zero-advance block to the caller." CORRECT.

**VERDICT: PASS — SEC-002 / forward-progress is fully resolved. No contradiction between
BC-2.01.015 and BC-2.01.010.**

---

#### Check 6 — Skip-arm / DSB Treatment (M-2)

**Claim marked FIXED:** BC-2.01.015 treats DSB as arriving via unknown block-type on
the raw-block path (no named DSB variant); consistent with ADR-009 rev 4 + spike.

**BC-2.01.015 v1.2 AC-001:** "IMPORTANT: DSB is NOT a named variant in
`pcap_file::pcapng::Block` — there is no `DecryptionSecrets` enum arm. On the raw-block
path, DSB arrives as a `RawBlock` with block-type bytes `0x0000000A` and is handled by
the skip arm. Do NOT attempt to name-match on a DSB enum variant." CORRECT.

**ADR-009 rev 4 Decision 2:** "There is NO `DecryptionSecrets` variant — DSB
(type `0x0A`) arrives as `Block::Unknown` (`block_common.rs:217-251`)." CORRECT.
"On the raw-block path, wirerust does not use the `Block` enum at all; block-type
identification is done by reading the first 4 bytes of each `RawBlock` body." CORRECT.

**VERDICT: PASS — DSB treatment is consistent.**

---

#### Check 7 — Magic-Byte Glob (C-2)

**Claim marked FIXED:** BC-2.12.011 v1.5 content-detection; 5 accepted magic values
consistent with BC-2.01.009 magic values; arp-baseline-16pkt.cap accepted.

**BC-2.12.011 v1.5 accepted magics:** Classic LE `0xA1B2C3D4`, classic BE `0xD4C3B2A1`,
ns-pcap LE `0xA1B23C4D`, ns-pcap BE `0x4D3CB2A1`, pcapng SHB `0x0A0D0D0A`. Five values.
CORRECT.

**BC-2.01.009 pcapng branch trigger:** "first 4 bytes are `[0x0A, 0x0D, 0x0D, 0x0A]`
(pcapng SHB magic)." Same value. CONSISTENT.

**BC-2.01.009 classic branch:** "first 4 bytes are a valid classic-pcap magic" —
implicitly matches the four classic/ns-pcap values. CONSISTENT.

**arp-baseline-16pkt.cap:** BC-2.12.011 v1.5 EC-002 explicitly covers this file.
ADR-009 Decision 11 also resolves C-2 by name. PASS.

**VERDICT: PASS — Magic-byte glob is consistent.**

---

#### Check 8 — Per-File Isolation Re-attribution (C-1)

**Claim marked FIXED:** BC-2.01.018 + error-taxonomy E-INP-011/012 notes + BC-2.12.011
consistently point per-file-isolation to STORY-128.

**BC-2.01.018 v1.2 AC-002:** "Directory-Mode Per-File Isolation — OWNED BY STORY-128...
BC-2.01.018 owns the multi-IDB CONFLICT RULE only." CORRECT re-attribution.

**ADR-009 rev 4 Decision 12:** "STORY-128 will refactor this loop to catch-and-continue
per file... BC-2.01.018 AC-002 and E-INP-011/012 per-file-isolation claims belong in
STORY-128 (main.rs scope)." CORRECT.

**error-taxonomy E-INP-011 Notes:** "In directory mode, this error fails the individual
file but does NOT abort the overall run (per BC-2.01.018 AC-002)." The note says
"per BC-2.01.018 AC-002" which is now re-attributed to STORY-128 via that same AC-002.
The pointer is correct (BC-2.01.018 AC-002 IS the place that documents STORY-128 owns
this). No contradiction. PASS.

**error-taxonomy E-INP-012 Notes:** "In directory mode, this error fails the individual
file but does NOT abort the overall run." No stale "BC-2.01.018" attribution here.
PASS.

**BC-2.12.011 v1.5 Stories field:** "STORY-127 (implements magic-byte content detection
in resolve_targets, main.rs); STORY-128 (per-file isolation in main.rs loop — separate
story, separate scope)." CORRECT.

**VERDICT: PASS — Per-file isolation re-attribution is consistent across all documents.**

---

#### Check 9 — HS-INDEX Integrity for HS-101..106

**HS-INDEX v2.0 registrations:**
- HS-101 through HS-106 all listed in the pcapng table (lines 170-175). PASS.
- All six have category, priority, wave (TBD), BC/VP references. PASS.
- must_pass: all six are must-pass. PASS.
- Greenfield total stated as 106 (HS-001..HS-106). PASS.
- All-namespace total: 106 greenfield + 32 DNP3 + 28 ARP + 13 collapse = 179.
  Header says "all-namespace total = 179" but the version comment says "v1.9:
  all-namespace=173." **DISCREPANCY — see Finding IDX-1 below.**

**VP columns in HS-INDEX pcapng table:**
- HS-101: BC-2.01.014 (VP-025) — matches HS-101 frontmatter. PASS.
- HS-102: BC-2.01.014 (VP-025) — matches HS-102 frontmatter. PASS.
- HS-103: BC-2.01.010 (VP-026) — matches HS-103 frontmatter. PASS.
- HS-104: BC-2.01.012 (VP-027) — matches HS-104 frontmatter. PASS.
- HS-105: BC-2.01.015 (VP-029) — matches HS-105 frontmatter. PASS.
- HS-106: BC-2.01.018 (VP-030) — matches HS-106 frontmatter. PASS.

**VERDICT: PASS on BC/VP column correctness. One arithmetic discrepancy in all-namespace
total (Finding IDX-1).**

---

#### Check 10 — Version Monotonicity and No New Dangling References

**Version check for F2 remediation burst (D-142):**

| Artifact | Prior version | D-142 version | Monotonic? |
|----------|-------------|--------------|------------|
| ADR-009 | rev 3 | rev 4 | YES |
| BC-2.01.010 | v1.2 | v1.4 (skips v1.3) | See note |
| BC-2.01.011 | v1.0 | v1.1 | YES |
| BC-2.01.012 | v1.0 | v1.1 | YES |
| BC-2.01.013 | v1.0 | v1.1 | YES |
| BC-2.01.014 | v1.0 | v1.1 | YES |
| BC-2.01.015 | v1.1 | v1.2 | YES |
| BC-2.01.016 | v1.0 | v1.1 | YES |
| BC-2.01.017 | v1.1 | v1.2 | YES |
| BC-2.01.018 | v1.1 | v1.2 | YES |
| BC-2.12.011 | v1.4 | v1.5 | YES |
| error-taxonomy | v2.5 | v2.7 (skips v2.6) | See note |
| VP-INDEX | v2.2 | v2.3 | YES |
| nfr-catalog | v2.2 | v2.3 | YES |
| HS-INDEX | v1.9 | v2.0 | YES |

**Note on skips:** BC-2.01.010 shows v1.2 → v1.4 with a v1.3 entry also in the
modified block. All three (v1.1, v1.2, v1.3, v1.4) are recorded in the changelog.
Version numbering is sequential. PASS — intermediate versions exist.
error-taxonomy v2.5 → v2.7 skips v2.6; v2.6 is present in the modified block as
"v2.6: RC-2 flag-spelling consistency — E-INP-012 message: standardized remediation
hint from 'mergecap -F pcapng' to 'mergecap -w out.pcapng <file>'." PASS.

**Active BC count:** tracker says "Active BC count still 302." BC-INDEX v1.52 states
302. CONSISTENT.

**VERDICT: PASS — versions are monotonic and active BC count is stable.**

---

### SPOT-VERIFY: Critical/High Items from Remediation Tracker

The following items are marked FIXED in the tracker and are verified on disk:

| Item | Tracker claim | Disk verification | Status |
|------|--------------|-------------------|--------|
| C-1: Per-file isolation | STORY-128 scoped; BC-2.01.018 AC re-attributed | BC-2.01.018 v1.2 AC-002 documents STORY-128 ownership; ADR-009 Decision 12 present | CONFIRMED FIXED |
| C-2: .cap extension pcapng | BC-2.12.011 v1.5 magic-byte glob; Decision 11 | BC-2.12.011 v1.5 EC-002 covers arp-baseline-16pkt.cap explicitly | CONFIRMED FIXED |
| C-3: VP-NNN assignments | VP-025..030 in VP-INDEX v2.3; HS-101..106 authored | VP-INDEX v2.3 rows 25-30 present; HS-101..106 files on disk | CONFIRMED FIXED |
| H-1/SEC-001/006: Timestamp arithmetic | BC-2.01.014 v1.1 saturating arithmetic | checked_pow, e-clamp to [0,63], u128 intermediate all confirmed in BC text | CONFIRMED FIXED |
| H-2: SPB overhead | BC-2.01.013 v1.1 overhead corrected to 4 bytes; ADR Decision 8 raw-block pivot | BC-2.01.013 v1.1 says 4 bytes; ADR Decision 8 present; one LOW erratum in ADR dispatch arithmetic (Finding H2-1) | CONFIRMED FIXED (with H2-1 LOW note) |
| H-3/SEC-003: E-INP-009 routing | BC-2.01.012 v1.1 PC5; error-taxonomy v2.7 | E-INP-009 routes empty-table; E-INP-010 routes OOB-non-empty; confirmed across all docs | CONFIRMED FIXED |
| H-4: SPB-without-IDB / OPB-only | BC-2.01.018 v1.2 (H-4 noted under OPB-only as out-of-scope) | BC-2.01.013 v1.1 PC5 and AC-001: SPB-without-IDB → E-INP-009 (guard on idb.is_empty()) | CONFIRMED FIXED |
| H-5: BC-2.01.009 PC1 | Reworded to ">= 0 packets" | BC-2.01.009 v1.0 PC1 line 47: "returns Ok(PcapSource) for a valid pcapng file with at least one readable packet." **NOT CHANGED to ">=0 packets."** | NOT FIXED — see Finding H5-1 |
| SEC-002: forward-progress invariant | BC-2.01.015 v1.2 forward-progress AC; VP-029 | BC-2.01.015 v1.2 AC-004, Invariant 2, EC-004, EC-005 all confirm crate rejects <12; loop must break on Err | CONFIRMED FIXED |

---

### FINDINGS FROM THIS PASS

#### Finding BOM-1 — LOW

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md`
**Location:** AC-001, line 68-69

**Issue:** AC-001 contains the parenthetical: "Holdout: SHB with BE magic `0x4D3C2B1A`
(wire big-endian encoding of 0x1A2B3C4D read big-endian)." The phrase "read big-endian"
contradicts the LE-read framing used throughout the rest of the document. The BOM
detection algorithm reads the 4 wire bytes as a LE u32 and checks whether the result
is `0x4D3C2B1A` (→ LE mode) or `0x1A2B3C4D` (→ BE mode). Saying "read big-endian"
implies the reader already knows the endianness before reading the BOM, which is
circular. The intent is correct (test the BE BOM path) but the explanation is
misleading for an implementer.

**Severity:** LOW — normative behavior is correct; wording only.

**Fix:** Replace the parenthetical with: "(wire value that a LE reader sees when the
file is big-endian; signals BE mode because it does not match the canonical LE BOM
`0x1A2B3C4D`)"

---

#### Finding BOM-2 — MEDIUM

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/holdout-scenarios/HS-103-pcapng-shb-framing-byte-order-and-error-cases.md`
**Location:** Case A, block_total_length encoding, line 48-49

**Issue:** Case A specifies: "block_total_length field: 0x00000000_1C000000 in
big-endian = 28 bytes." The value `28 = 0x0000001C`. In big-endian encoding on wire,
this u32 is 4 bytes: `00 00 00 1C`. The notation `0x00000000_1C000000` is an 8-byte
(u64) hex string, not a u32. If read as a LE u32 the value would be `0x001C0000 = 1835008`,
not 28. This is an erroneous representation.

The correct representation for the block_total_length field in a BE pcapng file is:
- u32 value = 28 = `0x0000001C`
- Wire bytes (BE): `00 00 00 1C`

**Severity:** MEDIUM — an implementer crafting the HS-103 test fixture from this
spec could create an invalid fixture with the wrong total length, causing the test
to fail for the wrong reason or accept a malformed file.

**Fix:** Replace "block_total_length field: 0x00000000_1C000000 in big-endian = 28 bytes"
with "block_total_length field: `0x0000001C` = 28; wire bytes (BE): `00 00 00 1C`"

---

#### Finding H2-1 — LOW

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md`
**Location:** PO BC-Change Dispatch, BC-2.01.013 section, line ~492

**Issue:** The PO dispatch for BC-2.01.013 states: "Validation: block_total_length - 16
- 4 = block_total_length - 20 bytes available for padded packet data (12 outer + 4
body-fixed = 16 minimum; callers strip padding to captured_len)." The formula
`btl - 16 - 4 = btl - 20` is arithmetically wrong: the 4-byte original_len field is
already part of the `16` total (12 outer + 4 body-fixed = 16). The correct formula is
`btl - 16` bytes available for padded data. The normative BC-2.01.013 text correctly
says `btl - 16`, so the implementation guidance in the dispatch is the only place with
the error.

**Severity:** LOW — the normative BC text is correct; this is a PO dispatch annotation
that an implementer might consult. The wrong `btl - 20` formula would cause the
implementer to strip 4 more bytes than necessary from the SPB data, potentially
truncating valid packet data.

**Fix:** In the ADR-009 PO BC-Change Dispatch BC-2.01.013 section, replace
"block_total_length - 16 - 4 = block_total_length - 20 bytes available" with
"block_total_length - 16 bytes available for padded packet data (12-byte outer block
header + 4-byte original_len = 16 bytes total overhead)"

---

#### Finding H5-1 — HIGH

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md`
**Location:** Postcondition 1, line 47

**Issue:** BC-2.01.009 v1.0 Postcondition 1 reads: "the reader selects the pcapng
parse path; returns `Ok(PcapSource)` for a valid pcapng file **with at least one readable
packet**." The remediation tracker item H-5 is marked FIXED with the claim "BC-2.01.009
PC1 reworded to '>=0 packets'." This change is NOT present on disk. The v1.0 frontmatter
shows no modified block (modified: []), confirming the BC has never been amended.

The "at least one readable packet" constraint conflicts with:
- BC-2.01.002 EC-001 parity: a valid empty pcapng (0 EPBs) should return Ok(PcapSource)
  with packets.len() == 0.
- OPB-only files: silently skip all OPB blocks → Ok(PcapSource) with 0 packets.
- Any pcapng with zero EPBs/SPBs is valid and should return Ok with empty Vec.

This is a CRITICAL behavioral contract error: if the spec says "at least one readable
packet" for Ok to be returned, then an empty-but-valid pcapng must return Err, which
contradicts the spirit of the feature and the OPB-only zero-packet case (BC-2.01.018
H-4 resolution).

**Severity:** HIGH — the tracker marks this FIXED but the fix is absent from disk.
This blocks the "OPB-only zero-packet case" from being implementable without
contradicting BC-2.01.009.

**Fix:** Amend BC-2.01.009 to v1.1. In Postcondition 1, replace "returns `Ok(PcapSource)`
for a valid pcapng file with at least one readable packet" with "returns `Ok(PcapSource)`
for a valid pcapng file (which may contain zero readable packets — e.g., files containing
only OPB blocks or no packet blocks at all; see BC-2.01.018 and BC-2.01.015)." Update
the modified block with the H-5 fix attribution.

---

#### Finding IDX-1 — LOW

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/holdout-scenarios/HS-INDEX.md`
**Location:** version comment in frontmatter (line 4) and Totals table (line 63)

**Issue:** The HS-INDEX version comment says "v1.9: docs-only — add all-namespace total
note per F3 audit (greenfield=100, feature DNP3=32 + ARP=28 + collapse=13 = 73,
all-namespace=173)." But the v2.0 update added 6 scenarios (HS-101..106) making
greenfield=106. The Totals table correctly says 179 (106 + 73 = 179). However the
version comment text from v1.9 remains embedded in the v2.0 version string and still
says "all-namespace=173" — which was the v1.9 total before the 6 new scenarios were
added. Additionally the v2.0 note only says "Greenfield total now 106" without updating
the embedded all-namespace figure in the version comment string.

**Severity:** LOW — the Totals table (179) is correct; the version comment string (173)
is a leftover from v1.9 text that was not updated when the 6 new scenarios pushed the
count to 179.

**Fix:** In the version frontmatter comment, update the embedded "all-namespace=173" to
"all-namespace=179" to match the Totals table.

---

#### Finding PRD-BC2-1 — MEDIUM

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/prd.md`
**Location:** §2 BC table, BC-2.12.011 row (line 893)

**Issue:** The PRD §2 BC description for BC-2.12.011 (line 893) reads:
"Directory target expands to all *.pcap files sorted; *.pcapng excluded from glob."
This is the PRE-v1.5 description. BC-2.12.011 was fully rewritten in v1.5 to describe
magic-byte content detection (not extension-based filtering). The PRD §2 description
still reflects the old v1.4 behavior.

**Severity:** MEDIUM — the PRD §2 BC table is a summary surface used for planning and
holdout alignment. A stale description causes the PRD to contradict BC-2.12.011 v1.5.

**Fix:** Update the PRD §2 BC-2.12.011 row description to:
"Directory target expands to capture files detected by magic-byte content probe
(pcapng `0x0A0D0D0A`, classic pcap `0xA1B2C3D4`/`0xD4C3B2A1`, ns-pcap variants);
extension-independent; implements ADR-009 Decision 11."

---

### F2 Remediation Re-Audit Summary

| Check | Result | Findings |
|-------|--------|---------|
| Priority: BOM correctness and no-conflation | PASS | BOM-1 (LOW), BOM-2 (MEDIUM) |
| 1. E-INP-009/010 remap coherence | PASS | None |
| 2. VP-025..030 coherence (BC cells, VP-INDEX, PRD RTM, holdouts) | PASS | None |
| 3. Timestamp totality (H-1) | PASS | None |
| 4. SPB/EPB overhead math (H-2) | PASS | H2-1 (LOW) |
| 5. EC-004 / forward-progress (SEC-002) | PASS | None |
| 6. Skip-arm / DSB (M-2) | PASS | None |
| 7. Magic-byte glob (C-2) | PASS | None |
| 8. Per-file isolation re-attribution (C-1) | PASS | None |
| 9. HS-INDEX integrity | PASS | IDX-1 (LOW) |
| 10. Version monotonicity / no new dangling refs | PASS | None |
| Spot-verify FIXED items (C-1, C-2, C-3, H-1..5, SEC-002) | PARTIAL | H5-1 (HIGH) |
| PRD §2 BC-2.12.011 description | FAIL | PRD-BC2-1 (MEDIUM) |

**Verdict: NOT CLEAN — 6 findings.**

| Finding | Severity | Description | Blocking F3? |
|---------|----------|-------------|-------------|
| H5-1 | HIGH | BC-2.01.009 PC1 still says "at least one readable packet" — tracker says FIXED but disk disagrees; conflicts with OPB-only zero-packet case | YES |
| BOM-2 | MEDIUM | HS-103 block_total_length encoding notation is wrong (u64 hex string instead of u32) | YES (fixture authoring) |
| PRD-BC2-1 | MEDIUM | PRD §2 BC-2.12.011 description still says extension-based filtering (stale pre-v1.5 text) | No (but should be fixed before F3 planning) |
| BOM-1 | LOW | BC-2.01.010 AC-001 parenthetical explanation is misleading (says "read big-endian" in LE-read context) | No |
| H2-1 | LOW | ADR-009 PO dispatch SPB formula btl-20 should be btl-16 | No |
| IDX-1 | LOW | HS-INDEX version comment says all-namespace=173; Totals table correctly says 179 | No |

**Blocking findings require remediation before F3 story decomposition:**

1. **H5-1 (HIGH):** Amend BC-2.01.009 to v1.1; change PC1 from "at least one readable
   packet" to "zero or more readable packets." This is the H-5 fix that the tracker
   claims was applied but which is absent from disk.

2. **BOM-2 (MEDIUM):** Correct HS-103 Case A block_total_length encoding from
   `0x00000000_1C000000` to `0x0000001C` (28 decimal); add wire-bytes notation
   `00 00 00 1C (BE)`. A test fixture author using this spec will craft a malformed file.

Non-blocking (address in same cleanup burst):

3. **PRD-BC2-1 (MEDIUM):** Update PRD §2 BC-2.12.011 row description to reflect v1.5
   content-detection behavior.
4. **BOM-1 (LOW):** Clarify BC-2.01.010 AC-001 parenthetical.
5. **H2-1 (LOW):** Correct ADR-009 PO dispatch SPB formula.
6. **IDX-1 (LOW):** Fix HS-INDEX version comment all-namespace count.
