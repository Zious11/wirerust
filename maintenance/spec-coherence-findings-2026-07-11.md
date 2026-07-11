---
title: Spec-Coherence Findings — maint-2026-07-11 Sweep 7
maint-run: maint-2026-07-11
sweep: 7
date: 2026-07-11
artifacts-read:
  - .factory/specs/prd.md (v1.51)
  - .factory/specs/behavioral-contracts/BC-INDEX.md (v2.22)
  - .factory/specs/verification-properties/VP-INDEX.md (v2.40)
  - .factory/specs/architecture/ARCH-INDEX.md (v2.12)
  - .factory/stories/epics.md (v2.1)
  - .factory/stories/STORY-INDEX.md (v3.43)
  - .factory/holdout-scenarios/HS-INDEX.md (v2.13)
  - .factory/tech-debt-register.md (v1.7)
  - .factory/specs/behavioral-contracts/ss-11/ (directory listing — 37 files)
  - .factory/specs/behavioral-contracts/ss-16/ (directory listing — 16 files)
status: FINDINGS-WRITTEN
prop-mandate: PROP-MAINT-03 (counts extracted from artifacts read; artifact+line cited for every count)
---

# Spec-Coherence Findings — maint-2026-07-11 Sweep 7

DF-030 25-check suite run on 2026-07-11. All counts extracted from artifacts actually read;
no count is independently re-evaluated. Every count is cited with artifact and line number
per PROP-MAINT-03.

---

## Executive Summary

| Category | Count |
|----------|-------|
| ROUTE-C-DEFERRED items re-verified | 4 |
| ROUTE-C-DEFERRED STILL-PRESENT | 2 |
| ROUTE-C-DEFERRED RESOLVED | 2 |
| NEW findings (this sweep) | 4 |
| NEW MINOR | 2 |
| NEW NIT | 2 |
| NEW MAJOR / BLOCKER | 0 |

New findings do NOT require issue filing until validated by `vsdd-factory:research-agent`
per policy DF-VALIDATION-001 (`.factory/policies.yaml`).

---

## ROUTE-C-DEFERRED Re-Verification

### RC-1 — HS-INDEX ENIP Waves/Stories Metadata

**Status: STILL-PRESENT**

HS-INDEX v2.13 records the EtherNet/IP feature holdout block (HS-110..HS-122) with the
following story and wave metadata:

- HS-INDEX v2.13, line 777 (intro prose): `Stories: STORY-131..STORY-141 (waves 63-68)`
- HS-INDEX v2.13, line 803 (summary table Stories row): `STORY-131..STORY-141`
- HS-INDEX v2.13, line 765 (maintenance note table): `EtherNet/IP (waves 63-68)`

Correct values per STORY-INDEX v3.43:

- STORY-INDEX v3.43, changelog line 59 (v2.8, 2026-06-24):
  `STORY-130..138 added (E-20, waves 58-61, 66 pts)`
- STORY-INDEX v3.43, changelog line 58 (v2.8, 2026-06-27):
  `STORY-139 added (E-20, wave 62, 8 pts)`
- Correct E-20 range: STORY-130..STORY-139, waves 58-62

Discrepancy: HS-INDEX story range is off by +1 on both ends (131..141 vs 130..139) and the
wave range is wrong (63-68 vs 58-62). The holdout content of HS-110..HS-122 is unaffected
(the individual HS files cite BCs directly), but the summary metadata in HS-INDEX is
inaccurate. STORY-141 in HS-INDEX is actually STORY-141 (E-14, wave 64, RULING-MODBUS-SIBLING-001),
not an ENIP story.

**Severity: MINOR**

---

### RC-2 — epics.md E-5 BC Row and total_bcs Arithmetic vs BC-INDEX Active

**Status: STILL-PRESENT (self-acknowledged in epics.md)**

- epics.md v2.1 frontmatter, line 19: `total_bcs: 337`
- BC-INDEX v2.22, line 17: `Active count: 347`
- Gap: 10 BCs

epics.md v2.1, line 284 (Per-Epic BC table E-5 row): `BC-2.07.001..037 | 37`
(BC-2.07.038..043 — 6 TLS carry-reassembly BCs added by fix-tls-clienthello-frag F3 — absent)

epics.md v2.1, line 364 (Coverage Confirmed arithmetic note) explicitly acknowledges:
> "6 TLS carry-reassembly BCs (BC-2.07.038..043, fix-tls-clienthello-frag F3 2026-06-29) are
> absent from the E-5 Per-Epic BC row and not reflected in this total; true total including
> those BCs = 343; residual gap vs BC-INDEX v2.13 (345 active) = 2 unresolved — deferred to
> next coverage-check reconciliation pass."

Since epics.md v2.1 (2026-07-02), BC-INDEX grew from ~345 to 347 active:
- BC-2.16.016 added (BC-INDEX v2.20, gap-remediation): not in epics.md
- BC-2.11.036/037 added (BC-INDEX v2.21, 2026-07-08, after epics.md v2.1): not in epics.md

The E-5 BC row in epics.md (line 284) remains stale. The total_bcs=337 gap to BC-INDEX 347
is 10. epics.md's own line 364 acknowledges this is a known deferred reconciliation item.

**Severity: NIT** (self-acknowledged; pre-existing deferred item; no story coverage loss)

---

### RC-3 — STORY-INDEX Coverage Tally "335 BCs" Claim

**Status: RESOLVED**

The previously-deferred concern was that STORY-INDEX claimed "335 BCs" covered. STORY-INDEX
v3.43, line 441 now reads:
> "total 337 BCs; explicit tally: 219 greenfield + 25 Modbus + 24 DNP3 + 15 ARP + 5 E-18
> flat-collapse + 5 E-18 grouped-collapse + 1 issue-#64 mitre_attack + 26 ENIP + 6 TLS carry
> reassembly + 4 SS-18 protocol-catalog + 2 SS-05 gap-counters + 3 SS-12 protocols-CLI +
> 2 JSON enum casing = 337"

The "335 BCs" claim no longer exists in STORY-INDEX v3.43. This deferred item is closed.
A new finding (NEW-3) documents the updated tally gap (337 vs BC-INDEX 347).

---

### RC-4 — BC-2.16.016 Narrative Overstatement (SEC-W70-001 CLOSED-BY-DESIGN)

**Status: RESOLVED**

tech-debt-register.md v1.7, line 72: SEC-W70-001 status is `**CLOSED-BY-DESIGN**`.

The concern about BC-2.16.016 narrative overstatement has been adjudicated. This deferred
item is closed.

However, the same tech-debt-register.md v1.7 entry (line 72) contains an outstanding PO
backlog action that has not been completed — see NEW-4.

---

## NEW Findings

### NEW-1 — ARCH-INDEX SS-11 Count Stale

**Severity: MINOR**
**Artifact:** ARCH-INDEX v2.12 (last updated 2026-07-03)

ARCH-INDEX v2.12, Subsystem Registry table, SS-11 row: count = **35**

Correct count per BC-INDEX v2.22, line 22 (v2.21 changelog):
> "SS-11 count 35→37" (BC-2.11.036/037 added 2026-07-08)

On-disk verification: `.factory/specs/behavioral-contracts/ss-11/` contains 37 files
(BC-2.11.001.md through BC-2.11.037.md), confirming BC-INDEX v2.22 active count = 37.

Impact on ARCH-INDEX subsystem arithmetic:
- ARCH-INDEX subsystem registry sum with SS-11=35: **345**
  (17+15+55+11+26+43+4+7+9+**35**+24+4+25+24+16+26+4 = 345; ARCH-INDEX v2.12, lines 159-178)
- BC-INDEX v2.22 active: **347** (line 17)
- Gap = 2, fully explained by the SS-11 lag (BC-2.11.036/037 added to BC-INDEX on 2026-07-08,
  after ARCH-INDEX v2.12 was written on 2026-07-03)

No other subsystem count in ARCH-INDEX is out of sync. SS-16 is correct at 16
(ARCH-INDEX line 176 = 16; BC-INDEX SS-16 active = 16; 16 files on disk in ss-16/).

---

### NEW-2 — epics.md Estimated Story Count Summary Stale

**Severity: MINOR**
**Artifact:** epics.md v2.1 (2026-07-02)

epics.md v2.1, line 641 (Estimated Story Count Summary total row):
> `| **Total** | **107** | Verified against STORY-INDEX v3.12 total_stories=107 |`

Current value per STORY-INDEX v3.43, frontmatter line 7: `total_stories: 117`

Gap: 10 stories added to STORY-INDEX between v3.12 (2026-07-02, when epics.md v2.1 was
written) and v3.43 (2026-07-11). The epics.md citation of "STORY-INDEX v3.12" is itself
accurate for the time of writing; the table has simply not been updated since.

epics.md changelog line 18 (v2.1) confirms the v3.12 reference: "Estimated Story Count
Summary Total 91→107. [...]" — this was the last reconciliation point.

---

### NEW-3 — STORY-INDEX Coverage Tally "337 BCs" Understates Active BC Count

**Severity: NIT**
**Artifact:** STORY-INDEX v3.43, line 441

STORY-INDEX v3.43 line 441 tally total: **337 BCs**
BC-INDEX v2.22 line 17 active count: **347 BCs**
Gap: **10**

The tally uses "219 greenfield" as its base. From epics.md v2.1 line 364, the correct
pre-feature subtotal is 228 (= 219 original + 9 net pcapng BCs: BC-2.01.009..018 +10,
BC-2.01.004 retired -1). The STORY-INDEX tally's "219 greenfield" does not include the 9
net pcapng BCs (BC-2.01.009..018), which ARE covered by E-19 stories STORY-123..128.

Additionally, the tally says "15 ARP" (not 16), so BC-2.16.016 (covered by STORY-156,
delivered PR #378 on 2026-07-08) is absent from the tally.

Breakdown of the 10-BC gap:
- 9 net pcapng BCs (BC-2.01.009..018 covered by STORY-123..128): not in tally
- BC-2.16.016 (covered by STORY-156, merged 2026-07-08): not in tally

All 347 active BCs have story coverage; this is a stale tally comment, not a coverage gap.

---

### NEW-4 — SEC-W70-001 PO Backlog Action Still Outstanding

**Severity: NIT**
**Artifact:** tech-debt-register.md v1.7, line 72 (SEC-W70-001 entry)

tech-debt-register.md v1.7 line 72, SEC-W70-001 entry status = `**CLOSED-BY-DESIGN**`,
with PO backlog note:
> "PO backlog note: author symmetric BC-2.07.NNN 'TlsAnalyzer::all_findings unbounded by
> design' mirroring BC-2.16.016 v1.2 in next spec-coherence sweep."

No BC with identifier `BC-2.07.038..043` range or beyond covers this semantic (those BCs
are the TLS carry-reassembly fragmentation BCs). BC-INDEX v2.22 SS-07 active count = 43
(BC-2.07.001..043). No BC documents `TlsAnalyzer::all_findings` unbounded cap as an
explicit design decision.

The PO backlog note was recorded in SEC-W70-001 to be acted on in the "next spec-coherence
sweep" — that is this sweep. The symmetric BC documenting `TlsAnalyzer::all_findings`
unbounded-by-design has not been authored.

---

## DF-030 Index-Count Arithmetic Summary

Counts verified from artifacts actually read. All arithmetic shown below.

### BC-INDEX Active Count

BC-INDEX v2.22, line 17: active = **347**
(348 on disk; BC-2.01.004 retired → 347 active)

### ARCH-INDEX Subsystem Registry Sum

ARCH-INDEX v2.12, lines 159-178 (Subsystem Registry table):

| SS | Count |
|----|-------|
| SS-01 | 17 |
| SS-02 | 15 |
| SS-04 | 55 |
| SS-05 | 11 |
| SS-06 | 26 |
| SS-07 | 43 |
| SS-08 | 4 |
| SS-09 | 7 |
| SS-10 | 9 |
| SS-11 | 35 (STALE — see NEW-1) |
| SS-12 | 24 |
| SS-13 | 4 |
| SS-14 | 25 |
| SS-15 | 24 |
| SS-16 | 16 |
| SS-17 | 26 |
| SS-18 | 4 |
| **Sum** | **345** |

ARCH-INDEX sum (345) vs BC-INDEX active (347) = gap of **2**, fully explained by SS-11
stale count.

### VP-INDEX Arithmetic

VP-INDEX v2.40, frontmatter lines 9-16:
- total_vps=43, p0_count=8, p1_count=29, test_sufficient_count=6
- kani_count=15, proptest_count=20, fuzz_count=2, integration_unit_count=6

Priority check: 8+29+6 = **43** ✓
Tool check: 15+20+2+6 = **43** ✓
VP-INDEX arithmetic is internally consistent. No discrepancy.

### epics.md total_bcs vs BC-INDEX active

epics.md v2.1 frontmatter line 19: `total_bcs: 337`
BC-INDEX v2.22 line 17: `Active count: 347`
Gap = **10** (see RC-2 for breakdown; gap is known-deferred in epics.md line 364)

### STORY-INDEX total_stories vs epics.md Estimated Story Count Summary

STORY-INDEX v3.43 frontmatter: `total_stories: 117`
epics.md v2.1 line 641: `Total | 107` (see NEW-2)
Gap = **10** stories

### HS-INDEX All-Namespace Total

HS-INDEX v2.13, lines 63-67 (all-namespace breakdown):
greenfield(109) + DNP3(32) + ARP(28) + collapse(13) + ENIP(13) + protocol-coverage(10) = **205** ✓
HS-INDEX v2.13 frontmatter: `all-namespace total = 205` ✓
Arithmetic is consistent; no discrepancy in totals.

---

## Checks with No Finding (DF-030 suite)

The following checks were run and found no discrepancy:

- **VP-INDEX internal arithmetic** — P0/P1/test-sufficient sum and tool-count sum both equal 43 ✓
- **SS-16 BC count** — ARCH-INDEX v2.12 SS-16=16; BC-INDEX SS-16 active=16; 16 files on disk ✓
- **HS-INDEX all-namespace total arithmetic** — 109+32+28+13+13+10=205 matches frontmatter ✓
- **BC-INDEX on-disk vs active delta** — 348 on disk - 1 retired (BC-2.01.004) = 347 active ✓
- **SS-11 on-disk count** — 37 files in ss-11/, matching BC-INDEX v2.22 active=37 ✓
- **SEC-W70-001 disposition** — Confirmed CLOSED-BY-DESIGN in tech-debt-register.md v1.7 ✓
- **RC-3 "335 BCs" claim** — No longer present in STORY-INDEX v3.43 (now says "337 BCs") ✓

---

## Pending Actions

The following items require PO decision before an issue can be filed (per DF-VALIDATION-001):

| Finding | Action Required |
|---------|----------------|
| RC-1 (STILL-PRESENT) | HS-INDEX v2.13 ENIP summary block needs correction: STORY-130..STORY-139, waves 58-62 |
| NEW-1 | ARCH-INDEX v2.12 SS-11 row needs update: 35 → 37 |
| NEW-2 | epics.md v2.1 Estimated Story Count Summary needs update: 107 → 117, cite STORY-INDEX v3.43 |
| NEW-3 | STORY-INDEX v3.43 line 441 tally needs update to include 9 pcapng BCs + BC-2.16.016 |
| NEW-4 | Decide: author BC-2.07.NNN documenting TlsAnalyzer::all_findings unbounded-by-design, or cancel the PO backlog note |
| RC-2 (STILL-PRESENT) | epics.md E-5 BC row update (BC-2.07.038..043 + total_bcs reconciliation) — already deferred |

No fixes applied. No commits made. Per sweep mandate: report only.
