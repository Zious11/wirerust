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
