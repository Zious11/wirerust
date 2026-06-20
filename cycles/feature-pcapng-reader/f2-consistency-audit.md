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
