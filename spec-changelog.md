---
document_type: spec-changelog
title: "wirerust Specification Changelog"
status: active
producer: product-owner
---

# wirerust Specification Changelog

All notable changes to the specification artifacts (PRD, BCs, domain spec, architecture)
are recorded here. Entries follow MAJOR.MINOR versioning: MINOR for new capabilities
added without breaking existing BCs; MAJOR for breaking changes (BC retirement, interface
changes, invariant rewrites).

---

## [pcapng-f2-pass6-reaudit-minors-2026-06-20] — 2026-06-20

### PASS-6 RE-AUDIT: 2 Minor FIXED (FINDING-P6-001/002) — D-157

**Trigger:** F2 pass-6 re-audit (consistency-validator) after D-156 remediation burst. 10 seams
checked; 2 Minor findings closed. 0 Major / 0 Critical. Adversary pass-7 pending. Clean-pass
counter 0/3.

**FINDING-P6-001 (Minor — BC-2.01.017 Related-BCs annotations missing E-INP-008):**
BC-2.01.017 v1.5 Related-BCs section listed BC-2.01.012 and BC-2.01.013 with annotations
referencing E-INP-009 and E-INP-010 only. Per Decision 20 and D-156 updates, BC-2.01.012
(EPB body-decode → E-INP-008) and BC-2.01.013 (SPB body-decode → E-INP-008) both route
body-too-short failures to E-INP-008. The Related-BCs annotations were incomplete — omitting
E-INP-008 contradicted this BC's own PC1 error-code split and Error Taxonomy field.
BC-2.01.017 v1.5→v1.6: Related-BCs annotations for BC-2.01.012 and BC-2.01.013 updated to
include E-INP-008 (EPB/SPB body-decode failures) alongside E-INP-009 and E-INP-010.

**FINDING-P6-002 (Minor — per-block body-too-short windows incorrect in BC-2.01.017 PC1):**
BC-2.01.017 v1.5 PC1 body-too-short window descriptions were partially wrong: SPB window
stated "[btl 16<=btl<20]" (which is the IDB window, not SPB); EPB window stated "[btl
32<=btl<52]" (wrong bounds). Correct per-block windows per Decision 20 + block fixed minimums:
SHB 12<=btl<28 (body 0-15; SHB_FIXED_OVERHEAD=16); IDB 12<=btl<20 (body 0-7; IDB_FIXED=8);
EPB 12<=btl<32 (body 0-19; EPB_FIXED_OVERHEAD=20); SPB btl=12 only (body=0 < 4;
SPB_FIXED_OVERHEAD=4; btl=16 is minimal valid SPB). BC-2.01.017 v1.5→v1.6: PC1 per-block
windows corrected. BC-INDEX v1.62→v1.63 (annotation synced).

**Per-block body-too-short windows reconciled (canonical):**
- SHB: 12<=btl<28 (body 0..15 bytes < 16-byte SHB minimum body)
- IDB: 12<=btl<20 (body 0..7 bytes < 8-byte IDB minimum body)
- EPB: 12<=btl<32 (body 0..19 bytes < 20-byte EPB minimum body)
- SPB: btl=12 only (body=0 bytes < 4-byte SPB minimum body; btl=16 is minimal valid SPB)

---

## [pcapng-f2-pass6-remediation-2026-06-20] — 2026-06-20

### PASS-6 REMEDIATION: 4H/5M/4L FINDINGS FIXED — D-156

**Trigger:** F2 pass-6 adversarial review (D-155) — 0C/4H/5M/4L. FIRST zero-critical pass.
Trajectory P1:23/P2:24/P3:17/P4:13/P5:13/P6:13 — count plateau; severity declining
(criticals 3/4/1/1/1/0). Remediation round-6 applied. Adversary pass-7 pending.
Clean-pass counter 0/3.

**F-H1 (HIGH — BC-2.01.017 EPB/SPB body-decode context strings → E-INP-008):**
BC-2.01.017 PC1 mapped EPB/SPB body-decode context strings to E-INP-010, contradicting
ADR-009 Decision 20 (uniform body-decode-truncation rule: body-too-short after crate
framing → E-INP-008 at ALL block types). BC-2.01.017 had been omitted from pass-4 and
pass-5 dispatch checklists — third cross-cutting-parent-BC-omission pattern (C-4/pass-2,
H-3/pass-3, F-H1/pass-6). BC-2.01.017 v1.4→v1.5: EPB/SPB body-decode context strings
corrected to E-INP-008; E-INP-010 scope in this BC now strictly crate-framing failures.
F-L3 process-gap actioned: BC-2.01.017 sweep is now a mandatory per-burst checklist item
per Lesson 12.

**F-H2/F-H3 (HIGH — Decision 22: canonical spb_data_available=body.len()-4):**
`block_body_available` had two definitions: BC-2.01.013 used "btl-16" (data bytes after
original_len header) while VP-031 used bare "body.len()" (crate body slice = btl-12,
includes 4-byte original_len field) — off by 4. HS-107 Case B asserted data.len()==100
(btl-16) but VP-031 form gave min(200, 104)=104, contradicting the holdout. ADR-009
Decision 22 (rev 9) established the canonical definition: `spb_data_available = body.len()-4`
(crate provides a body slice that already omits 8-byte header+trailer, but still includes the
4-byte original_len field; data bytes start after that). BC-2.01.013 v1.5→v1.6, VP-031
formula corrected to min(original_len, body.len()-4). HS-107 v1.4→v1.5: Case B rationale
annotated with derivation (body.len()=104; data_bytes=104-4=100).

**F-H4 (HIGH — interface_id discriminant split E-INP-009/E-INP-010 in VP-027 and HS-104):**
E-INP-009 (empty interface table) vs E-INP-010 (OOB on non-empty table) interface_id
discriminant was unpinned. ADR-009 used ambiguous slash notation "(→ E-INP-009 / E-INP-010)";
VP-027 proved no-panic+bounds but not the discriminant. HS-104 Cases A/B lacked exact code
requirements. BC-2.01.012 v1.5→v1.6: EC-006/PC5 empty-table path explicitly → E-INP-009;
EC-007 OOB path explicitly → E-INP-010; slash ambiguity removed. VP-027 extended to assert
discriminant. HS-104 v1.3→v1.4: Case A renamed "Case (empty)" with E-INP-009 exact;
Case B renamed "Case (OOB)" with E-INP-010 exact; byte-exact discriminant requirements
pinned. ADR-009 rev 9 slash notation replaced with two explicit cases.

**F-M1 (MEDIUM — HS-107 Case E alignment rationale):**
HS-107 Case E (btl=14) rationale said "below 12-byte minimum" but 14>=12 (14 passes the
framing check). Real rejection reason: alignment violation (14%4=2; pcapng requires
4-byte-aligned btl). HS-107 v1.4→v1.5: Case E rationale corrected to alignment violation.

**F-M2 (MEDIUM/process-gap — BOM canonical table + section-wide endianness):**
LE/BE on-disk byte ordering restated across 4+ prose sites in BC-2.01.010, HS-103,
ADR-009 — already corrected 3× but no single canonical anchor. BC-2.01.010 v2.0→v2.1:
BOM canonical table added as §BOM-Canonical; all sibling BOM prose sites carry a
cross-reference to this table per F-M2 canonical-anchor pattern (Lesson 13).

**F-M3 (MEDIUM — snaplen dead extraction after Decision 9 amend):**
BC-2.01.011 extracted snaplen and stored it in InterfaceInfo "for SPB use" but Decision 9
(rev 8) dropped snaplen from SPB captured_len formula. snaplen became dead extraction —
same anti-pattern condemned by Decision 21 for if_tsoffset. BC-2.01.011 v1.5→v1.6: snaplen
extraction annotated as DIAGNOSTIC-ONLY — MUST NOT be applied to captured_len. Note aligned
with Decision 9 amend and Decision 22.

**F-M4 (MEDIUM — SHB-only zero-packet edge case):**
SHB-only pcapng (no IDB, no packets, no subsequent blocks) zero-packet disposition was
unspecified. HS-108 covered IDB-only / OPB-only / EPB-before-IDB but not SHB-only.
HS-108 v1.2→v1.3: Case F (F-M4) added — SHB-only 28-byte pcapng → notice with
skipped_blocks==0 (no parenthetical segment), exit 0. Confirms "valid file + zero packets"
fires even with no IDB and no skip arm traversal. BC-2.01.009 edge case cross-reference added.

**F-M5 (MEDIUM — if_tsresol option_length!=1 → E-INP-008):**
if_tsresol option (code 9) with option_length!=1 was unspecified on the raw path. AC-005
only checked `length <= remaining`; a 2-byte if_tsresol passed the length guard, silently
delivering a wrong tsresol value. BC-2.01.011 v1.5→v1.6: AC-005 extended — option_length!=1
→ E-INP-008 (malformed option; not a semantic failure but a format invariant violation).
E-INP-008 trigger list in error-taxonomy confirmed (no new entry required; covered by
existing "malformed option TLV" Notes already added in v2.9/D-147 M-6).

**F-L1 (LOW — VP-025 scope note):**
VP-025 Kani scope note referenced only "if_tsresol=6 path" after pass-5 saturation
extension; general formula totality branches not listed. VP-025 scope note extended to
list all formula branches (base-10 / base-2 / µs fast-path / saturation).

**F-L2 (LOW — VP-INDEX count sweep):**
VP-INDEX total_vps=31 confirmed by count-propagation sweep per S-7.02 discipline.
VP-INDEX v2.7→v2.8 (VP-027 discriminant extension + VP-031 formula correction).
BC count 302 confirmed (error-taxonomy next_free E-INP-014 unchanged).

**F-L3 (LOW/process-gap — BC-2.01.017 checklist operationalized):**
Root of F-H1: BC-2.01.017 omitted from pass-4 and pass-5 checklists. Per Lesson 12
action, dispatch checklists now include mandatory item "Did BC-2.01.017 receive a version
bump?" for any burst touching Decision 20 routing.

**F-L4 (LOW — error-taxonomy E-INP-010 BC-refs stale):**
E-INP-010 BC-references column included BCs whose body-decode paths moved to E-INP-008
in D-151/D-153. Audit performed: E-INP-010 BC-refs updated to include only BCs with
remaining crate-framing-failure paths.

**Version bumps (~14 artifacts):**
- BC-2.01.009 v1.5→v1.6
- BC-2.01.010 v2.0→v2.1
- BC-2.01.011 v1.5→v1.6
- BC-2.01.012 v1.5→v1.6
- BC-2.01.013 v1.5→v1.6
- BC-2.01.015 v1.6→v1.7
- BC-2.01.017 v1.4→v1.5
- ADR-009 rev 8→rev 9
- VP-INDEX v2.7→v2.8
- verification-architecture.md v2.3→v2.4
- verification-coverage-matrix.md v1.17→v1.18
- HS-104 v1.3→v1.4
- HS-107 v1.4→v1.5
- HS-108 v1.2→v1.3
- BC-INDEX v1.61→v1.62

302 active BCs unchanged. error-taxonomy v3.4 unchanged (next_free E-INP-014 confirmed).

---

## [pcapng-f2-pass5-reaudit-minors-2026-06-20] — 2026-06-20

### PASS-5 RE-AUDIT: 4 MINOR FINDINGS FIXED — 6 SEAMS CLEAN — D-154

**Trigger:** F2 pass-5 re-audit (consistency-validator) — CLEAN on 6 seams; 4 Minor findings
fixed. No Major or Critical findings. Pass-5 fully remediated + consistency-verified. Adversary
pass-6 pending. Clean-pass counter 0/3.

**Version bumps (4 artifacts):**
- BC-2.01.010 v1.9→v2.0
- HS-108 v1.1→v1.2
- error-taxonomy v3.3→v3.4
- BC-INDEX v1.60→v1.61

302 active BCs unchanged.

---

**P5-001 (Minor — HS-108 VP ref→[]):**
HS-108 `verification_properties` field carried a placeholder VP reference. HS-108 is a
seam-level end-to-end holdout (OPB-only notice; zero-packet disambiguation); it exercises
multiple BCs holistically and does not trace to a single formal VP. Corrected to `[]`
(empty list, explicitly intentional per HS-108 design note).

**P5-002 (Minor — HS-108 OPB hint wording aligned to BC-2.01.009):**
HS-108 Cases d/e mergecap hint used informal phrasing. Aligned to the canonical hint wording
established in BC-2.01.009 v1.5 / Decision 19 amend: `mergecap -w out.pcapng <file>`.

**P5-003 (Minor — BC-2.01.010 stale deferral notes replaced with HS-103 case refs):**
BC-2.01.010 body contained stale "deferred to a separate burst" deferral notes referencing
holdout scenarios that now exist. Replaced with explicit HS-103 case cross-references:
HS-103 Case D (btl=16 → E-INP-008 SHB body-too-short constructible case) and HS-103 Case B
(btl=8 → E-INP-008 BOM-bad). BC-2.01.010 v1.9→v2.0. BC-INDEX v1.60→v1.61 (annotation synced).

**P5-004 (Minor — error-taxonomy E-INP-008 BC-ref +BC-2.01.013):**
error-taxonomy E-INP-008 `BC-references` column listed BC-2.01.010/011/012 but omitted
BC-2.01.013. BC-2.01.013 v1.5 now routes SPB body-too-short (block_body_available < 4) to
E-INP-008 per Decision 20 uniform rule. Added BC-2.01.013 to E-INP-008 BC-ref list.
error-taxonomy v3.3→v3.4.

---

## [pcapng-f2-pass5-remediation-2026-06-20] — 2026-06-20

### PASS-5 REMEDIATION: 1C/4H/5M/3L ALL FIXED — PASS-6 PENDING — D-153

**Trigger:** F2 adversary pass-5 (D-152) — NOT CLEAN: 1C/4H/5M/3L, HIGH novelty (trajectory plateau
P1:23/P2:24/P3:17/P4:13/P5:13). All pass-5 findings remediated in this burst. Adversary pass-6 pending.
Clean-pass counter 0/3. F3 still BLOCKED.

**Version bumps (~14 artifacts):**
- BC-2.01.009 v1.4→v1.5
- BC-2.01.012 v1.4→v1.5
- BC-2.01.013 v1.4→v1.5
- BC-2.01.014 v1.4→v1.5
- BC-2.01.015 v1.5→v1.6
- BC-2.01.018 v1.5→v1.6
- BC-INDEX v1.59→v1.60
- error-taxonomy v3.2→v3.3
- ADR-009 rev 7→rev 8
- VP-INDEX v2.6→v2.7
- verification-architecture.md v2.2→v2.3
- verification-coverage-matrix.md v1.16→v1.17
- HS-104 v1.2→v1.3
- HS-107 v1.3→v1.4
- HS-108 v1.0→v1.1

302 active BCs unchanged.

---

**C-1 (CRITICAL — E-INP-010→E-INP-008 reclassification for EPB body-decode failures):**
BC-2.01.012 v1.5: EPB body-decode failures reclassified from E-INP-010 → E-INP-008 at all
body-decode sites. D-150 fixed items (d)/(e) (body-too-short mis-classified in error-taxonomy)
but left item (c) (padding-overrun in EC-010 and HS-104 Case E) still on E-INP-010 — partial-fix
sibling miss. Per Decision 20 uniform rule: the crate has already successfully framed the block
(btl >= 12, aligned, trailing-length match) before any EPB body-decode runs; wirerust body-decode
rejections (bound-by-body: captured_len > body.len() - 20; padding-overrun: 20 + captured_len +
pad_len > body.len()) are body-decode failures → E-INP-008. Updated in BC-2.01.012: PC6a, PC6b,
AC-002, AC-006, EC-010, canonical test vector rows, VP-027. error-taxonomy v3.3: E-INP-010
item (c) reclassified to E-INP-008; E-INP-008 row updated. HS-104 v1.3: Cases D and E
reclassified E-INP-010 → E-INP-008.

**H-1 (HIGH — BC-2.01.018 EC-006/EC-008 contradiction of Decision 17 precedence):**
BC-2.01.018 v1.6: EC-006 step derivation added — ETHERNET (whitelisted) + IEEE802_11
(non-whitelisted): whitelist check step 2 fires on second IDB → E-INP-001; E-INP-011 conflict
check step 3 never reached. EC-008 reclassified E-INP-011 → E-INP-001: two IEEE802_11 IDBs —
first IDB non-whitelisted triggers E-INP-001 at whitelist check step 2 (first-IDB parse time);
second IDB never parsed; agreement unobservable. E-INP-011 conflict check is now correctly
stated as reachable ONLY when BOTH IDBs whitelisted AND differ (e.g. ETHERNET then RAW). Per
Decision 17 precedence: E-INP-013 position-check first → E-INP-001 whitelist second →
E-INP-011 conflict third.

**H-2 (HIGH — OPB-distinct notice + HS-108 Cases d/e):**
BC-2.01.015 v1.6: opb_skipped:u32 sub-counter added — incremented specifically for obsolete
Packet Block (OPB, type 0x00000002) skips; skipped_blocks:u32 remains total-skip counter.
PC9 rewritten: counters surfaced via PcapSource fields (reader has no filename); main.rs reads
PcapSource.skipped_blocks and PcapSource.opb_skipped after from_pcap_reader returns Ok, emits
notice: when opb_skipped > 0, notice appends "(includes N obsolete Packet Blocks whose data was
not analyzed; re-save with mergecap)". AC-006 updated to document both counters. HS-108 v1.1:
Cases d/e added (OPB-only → notice with OPB count + mergecap hint; 2 NRBs + 1 OPB → notice
shows OPB count distinctly from NRB skips).

**H-3/M-2 (HIGH+MEDIUM — SPB snaplen dropped, Decision 9 amendment):**
BC-2.01.013 v1.5: snaplen DROPPED from SPB captured_len formula throughout. ADR Decision 9 states
neither EPB nor SPB enforces snaplen; Decision 9 amended (rev 8) to be explicit that captured_len
for SPB = min(original_len, block_body_available) with no snaplen term. Updated: Description, PC1,
AC-002, EC-007, EC-001, Invariant 2, Canonical Test Vectors, Architecture Anchors. VP-031 updated:
captured_len == min(original_len, body.len() as u32). HS-107 v1.4: Case B rationale updated
(body-bound, not snaplen clamp); all A–F Cases retained; VP-031 property updated.

**H-4 (HIGH — HS-107 VV description corrected + stale deferral notes removed):**
BC-2.01.013 v1.5: VV (Verification Value) table mis-described HS-107 as
"real-world no-false-positives"; corrected to accurate description: "SPB framing truncation,
padding, and no-IDB boundary scenarios (incl. Case F btl=12→E-INP-008)". 4× stale
"HS-107 btl=12 holdout deferred to a separate burst" notes removed — HS-107 Case F now
exists on disk and covers the btl=12→E-INP-008 case.

**M-1 (MEDIUM — Precondition 3 deleted from BC-2.01.009):**
BC-2.01.009 v1.5: Precondition 3 ("at least 4 bytes available") deleted. The precondition
contradicts EC-003 (graceful Err on <4-byte stream) and inverts the trust model — the
<4-byte condition is a runtime error handled by postcondition (returns Err), not a caller
obligation. EC-003 already covers the graceful Err path; removing PC3 aligns the contract.

**M-3 (MEDIUM — µs fast-path saturation):**
BC-2.01.014 v1.5: PC4 if_tsresol=6 fast path now explicitly uses
`(ticks / 1_000_000).min(u32::MAX as u64) as u32` for ts_sec (same saturation as general
formula in PC2/PC3); prior bare `ticks / 1_000_000 as u32` wraps where general formula
saturates, diverging at large ts_high and threatening VP-025 totality. Canonical saturation
test vector added (ts_high large enough that ticks/1_000_000 > u32::MAX). VP-025 Kani harness
scope extended to include if_tsresol=6 path with ts_high=u32::MAX.

**M-4 (MEDIUM — BufReader wrap-site AC):**
BC-2.01.009 v1.5: AC-007 added pinning the BufReader wrap site — from_pcap_reader MUST
internally wrap its R:Read in BufReader and feed the SAME BufReader to both the fill_buf
peek step AND all downstream parsers (pcapng branch and classic-pcap branch). This ensures
the peek is buffered and the downstream read sees un-consumed bytes at offset 0. No separate
unbuffered Read regression test required by this AC; implementation must use single BufReader.

**M-5 (MEDIUM — notice moved to main.rs, PcapSource fields, classic symmetry, Decision 19 amend):**
BC-2.01.009 v1.5 and BC-2.01.015 v1.6 jointly: notice emission responsibility moved from
reader to main.rs (which has the filename). PcapSource exposes skipped_blocks:u32 and
opb_skipped:u32. from_pcap_reader surfaces these fields; main.rs reads them after
from_pcap_reader returns Ok and emits the notice. Canonical format per Decision 19 (amended
rev 8): `"notice: <filename>: 0 packets read from <pcap|pcapng> file"` — classic empty pcap
also triggers notice (classic/pcapng symmetry). When opb_skipped > 0, appends "(includes N
obsolete Packet Blocks whose data was not analyzed; re-save with mergecap)". ADR-009
Decision 19 amended (rev 8) to record emission-site + format canonical form.

**Observations deferred to F3:**
- PG-2 (STORY-128 existence) — BC-2.01.018 traces to STORY-128; verify on-disk before F3 entry.
- PG-3 (arp-baseline-16pkt.cap SHB/IDB params) — verify file SHB/IDB params before F3 entry.
Both carried as F3-entry checklist items, not blocking this burst.

---

## [pcapng-f2-pass4-reaudit-boundary-fixes-2026-06-20] — 2026-06-20

### PASS-4 RE-AUDIT BOUNDARY FIXES: 3 MAJOR GAPS CLOSED — FINDING-P4-001 / FINDING-P4-002 / FINDING-P4-003

**Trigger:** F2 pass-4 cross-seam re-audit identified 3 Major boundary-consistency gaps between
BC-2.01.011 and error-taxonomy after the D-150 pass-4 remediation burst. All 3 gaps closed in
this burst. Re-audit of seams 2-12 was otherwise CLEAN. Adversary pass-5 pending.

**Version bumps (2 artifacts):** BC-2.01.011 v1.4→v1.5; error-taxonomy v3.1→v3.2.
BC-INDEX v1.58→v1.59 (inline annotation synced). 302 active BCs unchanged.

**FINDING-P4-001 (Major — BC-2.01.011 v1.5):** PC5 tail sentence in BC-2.01.011 v1.4 contained a
stale boundary statement: "E-INP-008 covers SHB and IDB structural errors ONLY; EPB/SPB body
truncation routes to E-INP-010 per error-taxonomy." This directly contradicted ADR-009 rev 7
Decision 20 (uniform body-decode-truncation rule), which establishes that crate-framed-but-body-too-short
for ALL block types — including EPB body<20 and SPB body<4 — maps to E-INP-008 (not E-INP-010). The
stale sentence was a copy-carry-forward from BC-2.01.011 v1.2 (pass-2 remediation), not updated when
Decision 20 was applied in v1.4. Fixed: stale PC5 tail sentence removed; normative routing now
consistent with Decision 20 and the error-taxonomy E-INP-008 row.

**FINDING-P4-002 (Major — error-taxonomy v3.2):** E-INP-010 Notes in error-taxonomy v3.1 retained a
stale tail note: "E-INP-008 is RESERVED for SHB/IDB body-decode failures only ... it is NOT used for
EPB/SPB errors." This contradicted the E-INP-008 normative row (which explicitly lists EPB body<20 /
SPB body<4 as E-INP-008 triggers) and contradicted Decision 20. The note was a legacy remnant from
v2.7 that survived the v3.0/v3.1 rewrites. Fixed: stale tail note removed from E-INP-010 Notes.

**FINDING-P4-003 (Major — error-taxonomy v3.2):** E-INP-010 scope in error-taxonomy v3.1 listed items
(d) "EPB body truncated (<20 fixed-field bytes)" and (e) "SPB body truncated (<4 bytes)" as E-INP-010
triggers. Per Decision 20 these are E-INP-008 cases — btl >= 12 so the crate frames the block
successfully; wirerust body-decode finds the body too short → E-INP-008. Items (d) and (e) were
misclassified vestiges of the pre-Decision-20 taxonomy (v2.7). Fixed: items (d) and (e) removed from
E-INP-010. E-INP-010 boundary clarified: btl<12/misaligned/EOF → E-INP-010 (framing); 12<=btl<block-
fixed-min → E-INP-008 (body-decode); EPB block-fixed-min=32, SPB block-fixed-min=16.

**E-INP-008 row integrity confirmed:** The E-INP-008 normative row was NOT modified — it already
correctly listed EPB body<20 / SPB body<4 as E-INP-008 triggers. The fixes above removed contradictions
in E-INP-010 Notes and BC-2.01.011 PC5 that pointed the wrong way.

---

## [pcapng-f2-pass4-remediation-2026-06-20] — 2026-06-20

### PASS-4 ADVERSARIAL REMEDIATION: 1 CRITICAL / 4 HIGH / 5 MEDIUM / 3 LOW — HIGH novelty; D-150

**Trigger:** F2 adversary pass-4 (D-149) identified 1C/4H/5M/3L findings, HIGH novelty class
(EPB/SPB sibling-propagation gap; false-unconstructibility over-correction; VP satisfiability
failure; SOUL #4 holdout gap). All must-fix items remediated. Pass-5 is next; F3 remains
BLOCKED until 3 clean passes.

**Version bumps (~19 artifacts):** BC-2.01.009 v1.3→v1.4; BC-2.01.010 v1.8→v1.9;
BC-2.01.011 v1.3→v1.4; BC-2.01.012 v1.3→v1.4; BC-2.01.013 v1.3→v1.4; BC-2.01.014 v1.3→v1.4;
BC-2.01.015 v1.4→v1.5; BC-2.01.016 v1.3→v1.4; BC-2.01.017 unchanged v1.4;
BC-2.01.018 v1.4→v1.5; error-taxonomy v3.0→v3.1; ADR-009 rev 6→rev 7;
VP-INDEX v2.5→v2.6; verification-architecture.md v2.1→v2.2;
verification-coverage-matrix.md v1.15→v1.16; HS-103 v1.4→v1.5; HS-104 v1.1→v1.2;
HS-107 v1.2→v1.3; HS-108 v1.0 (new); HS-INDEX v2.2→v2.3. BC-INDEX v1.57→v1.58.

**Critical finding (1):**

- **C-1 (EPB padding-aware bound — BC-2.01.012 v1.4):** [Critical] EPB captured_len guard
  was `captured_len <= block_total_length - 32` — missing the 4-byte padding term and lacking
  an unconditional body.len() bound. The SPB three-way min fix (D-147) was not propagated to
  the EPB sibling. A non-multiple-of-4 captured_len at the body boundary could cause padded
  slice overrun. Fixed: padding term `EPB_FIXED + captured_len + pad(captured_len) <= body.len()`
  added; unconditional body.len() bound added as first guard; M-3 PC8 scoped to EC-009 only;
  EC-008 boundary case moved to HS-104 Case E (HS-104 v1.2).

**High findings (4):**

- **H-1 / Decision 20 (uniform body-decode-truncation rule; SHB body-too-short re-added):**
  [High] Pass-3 narrowed SHB E-INP-008 to semantic-only based on the false premise that
  "btl<12 is unconstructible." This was incorrect: btl=16 IS constructible (16 % 4 == 0,
  >= 12; crate accepts; wirerust receives body of 4 bytes < 16-byte SHB minimum) and was
  wrongly removed from spec coverage. ADR-009 rev 7 Decision 20 establishes the uniform
  body-decode-truncation rule: three mutually-exclusive tiers (crate framing failure →
  E-INP-010; aligned-and-framed body-too-short → E-INP-008; semantic failure → E-INP-008).
  SHB body-too-short (btl=16 → body=4 example) re-added as constructible E-INP-008 case.
  HS-103 v1.5 (Case D added). HS-107 v1.3 (Case F added — SPB btl=12 → E-INP-008).
  BC-2.01.009/010/011/015 updated. error-taxonomy v3.1 (uniform rule preamble).

- **H-2 (peek-only probe — BC-2.01.009 v1.4):** [High] BC-2.01.009 probe specified
  consume(4) which advances the stream cursor past the block-type bytes before dispatch.
  Every downstream block parser then receives the wrong bytes. Fixed: probe must be peek-only
  via fill_buf (no cursor advance). BC-2.01.009 v1.4.

- **H-3 (VP-030 restated — BC-2.01.018 v1.5; VP-INDEX v2.6):** [High] VP-030 was specified
  over arbitrary u16 inputs, but non-whitelisted linktypes short-circuit to E-INP-001
  (Decision 17 step 2) before the E-INP-011 conflict check (step 3). VP-030 as written
  was unsatisfiable — virtually all arbitrary u16 pairs hit E-INP-001 and never reach the
  code under test. Fixed: VP-030 domain narrowed to whitelisted DataLink values only;
  non-whitelisted → E-INP-001 explicitly out of VP-030 scope. VP-INDEX v2.6 (H-3 restate).
  BC-2.01.018 v1.5.

- **H-4 (HS-108 zero-packet / Decision 19 — BC-2.01.009 v1.4):** [High] No holdout scenario
  existed for the zero-packet one-shot notice (SOUL #4 property). Pass-3 M-3 broadened the
  notice but no HS covered it. Also: BC-2.01.009 lacked a disambiguation rule between
  zero-packet success (Ok + notice) and EPB-before-IDB error (Err). Fixed: HS-108 authored
  (three cases: IDB-only pcapng → Ok + notice; IDB + 2 unknown-type skipped blocks → Ok +
  notice with skip count; EPB before IDB → Err E-INP-009). ADR-009 Decision 19 formalizes
  the zero-packet notice gating rule. BC-2.01.009 v1.4 adds disambiguation rule.
  HS-INDEX v2.3 (HS-108; all-namespace=181).

**Medium findings (5):**

- **M-1 (crate-enforces over-claim removed):** BC-2.01.011/012/013 attributed body-minimum
  enforcement to the crate; on the raw-block path wirerust performs the guard itself.
  Removed the crate-attribution claim from BC-2.01.011 v1.4, BC-2.01.012 v1.4,
  BC-2.01.013 v1.4.

- **M-2 / Decision 21 (if_tsoffset out-of-scope — BC-2.01.011 v1.4; BC-2.01.014 v1.4):**
  if_tsoffset (IDB option code 10) was extracted in BC-2.01.011 PC6 but the term never
  appeared in the BC-2.01.014 timestamp formula, creating a silent timestamp offset
  wrongness path. Declared out-of-scope: ADR-009 Decision 21 records this as a conscious
  scope decision (zero-offset corpus assumed; if_tsoffset support deferred). BC-2.01.011
  v1.4 and BC-2.01.014 v1.4 document the out-of-scope ruling.

- **M-3 (EC-008 boundary case to HS-104):** BC-2.01.012 PC8 over-claimed coverage of
  both EC-008 (captured_len < original_len) and EC-009 (captured_len == original_len) via
  the arp-baseline fixture, which has no truncated packets. PC8 scoped to EC-009 only;
  EC-008 boundary case moved to HS-104 Case E. BC-2.01.012 v1.4; HS-104 v1.2.

- **M-4 / Decision 19 (zero-packet anchor — BC-2.01.009 v1.4; BC-2.01.018 v1.5):**
  BC-2.01.009 PC6 and BC-2.01.015 PC9 mis-cited ADR Decision 17 (IDB-parse precedence
  order) as the authority for zero-packet notice behavior. Zero-packet notice is governed
  by Decision 19, not Decision 17. Citations corrected. BC-2.01.018 v1.5 adds Decision 19
  anchor.

- **M-5 (#seq convention — error-taxonomy v3.1):** Block sequence numbering used two
  incompatible conventions across error-taxonomy entries: E-INP-012 counted SHB in
  "#seq within file" (SHB = block 1), while E-INP-010 / E-INP-013 counted blocks
  "after SHB." Fixed: error-taxonomy v3.1 adds a preamble pinning the canonical
  convention (all templates count "#N in file, including SHB as block 1"). BC-2.01.015 v1.5,
  BC-2.01.016 v1.4, BC-2.01.018 v1.5 carry cross-reference notes.

**Low findings (3):**

- **L-1 (DLT code verification — BC-2.01.016 v1.4):** Numeric DLT codes in BC-2.01.016
  error message template source-verified against pcap-file 2.0.0 DataLink enum discriminants
  and official linktype registry. Corrections applied where discrepant.

- **L-2 (unescaped pipe — BC-2.01.011 v1.4):** EC-003 in BC-2.01.011 contained unescaped
  pipe `0x80 | 0x0A` in Markdown table cell causing table corruption. Wrapped in code span.
  (Same class as pass-1 L-1 fix for BC-2.01.011 v1.1 that was re-introduced during H-2
  constructible-fixture window edit.)

- **L-3 (process-gap / deferred):** error-taxonomy input-hash still N/A — error contract
  outside drift guard. Pre-F3 obligation; not blocking this remediation burst. Carried
  forward as open obligation.

---

## [pcapng-f2-pass3-reaudit-fixes-2026-06-19] — 2026-06-19

### PASS-3 CROSS-SEAM RE-AUDIT GAP FIXES: 4 ITEMS (1 Major / 2 Minor / 1 Obs) — ALL FIXED

**Trigger:** F2 pass-3 cross-seam re-audit identified 4 prose-layer gaps not caught during the
pass-3 remediation burst (D-147). All 4 gaps are fixed in this burst. 8 of 12 re-audit seams
were already clean. Pass-4 adversary dispatch is next.

- **P3-001 (error-taxonomy E-INP-008 scope note — v3.0):** [Major] E-INP-008 scope note in
  error-taxonomy was ambiguous after the H-1/H-2 semantic-only narrowing applied in D-147. The
  scope note now explicitly states that E-INP-008 fires only for semantic validation failures
  (invalid BOM bytes, major version != 1) and never for framing/length truncation (which routes
  to E-INP-010). error-taxonomy v2.9→v3.0.

- **P3-002 (BC-2.01.018 Related-BCs order fix — v1.4):** [Minor] BC-2.01.018 Related-BCs list
  had an ordering inconsistency introduced during the H-5 dead-spec fix (D-147): the list order
  did not follow the canonical BC numeric sequence. Reordered to canonical form. No normative
  content changed. BC-2.01.018 v1.3→v1.4. BC-INDEX v1.56→v1.57.

- **P3-003 (HS-107 VP-031 traceability — v1.2):** [Minor] HS-107 verification_properties field
  listed only VP-028 (cargo-fuzz) after the pass-3 remediation burst, but VP-031 (SPB
  captured-len proptest, assigned in D-147 M-2) was not added to the HS-107 traceability block.
  Added VP-031 to HS-107 verification_properties. HS-107 v1.1→v1.2. HS-INDEX v2.1→v2.2
  (HS-107 row VP column updated from "(VP-028)" to "(VP-028, VP-031)").

- **P3-004 (HS-107 Case B three-way min — v1.2):** [Obs] HS-107 Case B captured_len
  computation prose referenced the two-way form prior to the C-1/H-4 fix (D-147); updated to
  explicit three-way `min(original_len=200, snaplen=100, block_body_available=100)=100` to match
  BC-2.01.013 PC1 three-way contract. HS-107 v1.2 (same bump as P3-003).

---

## [pcapng-f2-pass3-remediation-2026-06-19] — 2026-06-19

### PASS-3 ADVERSARIAL REMEDIATION: 1 CRITICAL / 5 HIGH / 7 MEDIUM / 4 LOW — HIGH novelty

**Trigger:** F2 adversarial pass-3 identified 1C/5H/7M/4L findings, HIGH novelty class
(partial-fix-propagation + sibling-layer + dead-spec — new class not anticipated by prior passes).
All must-fix items remediated in this burst. Pass-4 is next; F3 remains BLOCKED until 3 clean passes.

**Critical finding (1):**

- **C-1 (SPB three-way min panic fix):** BC-2.01.013 v1.2 changelog falsely claimed PC1 was fixed
  (three-way `min(original_len, snaplen, block_body_available)` applied); on-disk PC1 and AC-002
  still used two-way `min(original_len, snaplen)` → out-of-bounds slice panic on malformed SPB.
  Fix propagated to PC1, AC-002, EC-007, and Case-B. VP-031 (SPB proptest) assigned to cover
  arithmetic correctness that fuzz (VP-028) cannot express. BC-2.01.013 v1.3. ADR-009 rev 6
  Decision 18 (VP-031).

**High findings (5):**

- **H-1 (E-INP-008 narrowed to semantic failures):** BC-2.01.010 PC5/AC-004/EC-005 — E-INP-008
  SHB body-truncation fixture was unconstructible (crate rejects btl<12; framing truncation routes
  to E-INP-010 via crate Err). Fix: E-INP-008 now fires ONLY for semantic failures (invalid BOM,
  major version != 1); all SHB framing/length truncation routes to E-INP-010. BC-2.01.010 v1.8.
  error-taxonomy v2.9 (E-INP-008 Notes updated).
- **H-2 (IDB constructible-fixture window 12<=btl<20):** BC-2.01.011 same unconstructible-fixture
  issue for IDB. Constructible E-INP-008 window is 12<=btl<20 (body 0–7 bytes). BC now states this
  window explicitly; "crate returned a short body" language removed. BC-2.01.011 v1.3. BC-2.01.010
  v1.8 (same fix stated for SHB).
- **H-3 (E-INP-001 wired into taxonomy + BC-2.01.017):** E-INP-001 (linktype whitelist, BC-2.01.016)
  was orphaned — not in error-taxonomy E-INP-001 BC-ref; not in BC-2.01.017 error-code table (which
  covered only E-INP-008..E-INP-013). Fix: BC-2.01.016 added to E-INP-001 BC-ref in error-taxonomy
  v2.9; E-INP-001 added to BC-2.01.017 v1.4 error-code table (range now E-INP-001 + E-INP-008..013).
- **H-4 (SPB EC-007/Case-B three-way min propagation):** BC-2.01.013 EC-007 and Case-B still used
  two-way min (same root as C-1; fix did not propagate). Both updated to three-way form. BC-2.01.013
  v1.3 (same version bump as C-1 fix).
- **H-5 (multi-section interface-table reset dead spec — Decision 16):** BC-2.01.011 Inv 2 +
  BC-2.01.018 Inv 4/EC-005 mandated per-SHB interface-table reset, but Decision 7 rejects the 2nd
  SHB before any reset fires — dead spec. Fix: per-section-reset invariants removed/deferred;
  BC-2.01.018 EC-005 corrected (multi-section → E-INP-012, not "succeeds per section"). ADR-009
  rev 6 Decision 16 (dead-spec deferral). BC-2.01.011 v1.3, BC-2.01.012 v1.3, BC-2.01.015 v1.4,
  BC-2.01.018 v1.3.

**Medium findings (7):**

- **M-2 (VP-031 SPB framing proptest):** HS-107 was bound to VP-028 (fuzz) but asserts byte-exact
  framing arithmetic that fuzz cannot express. Added VP-031 (SPB captured-len computation
  proptest; P1; reader.rs pcapng_pure_core fns; BC-2.01.013). VP-INDEX v2.5 (total 31).
  ADR-009 rev 6 Decision 18.
- **M-3 (zero-packet notice broadened):** One-shot notice only fired when skipped_blocks>0;
  IDB-only/SHB-only valid files silently yielded zero packets (SOUL #4 violation). Notice
  broadened to "valid file, zero packets" regardless of skip count. BC-2.01.011 v1.3.
- **M-4 (timestamp parity scoped to ts_high==0):** BC-2.01.014 Inv 2 over-claimed classic-pcap
  parity for ts_high>0 (classic stores raw u32 secs; pcapng saturates — not equivalent). Parity
  claim scoped to ts_high==0. BC-2.01.014 v1.3.
- **M-5 (N-packet happy-path postcondition + arp-baseline-16pkt.cap):** No BC owned the valid
  single-section N-packet in-order + payload-fidelity happy path. Postcondition added with
  16-packet anchor (arp-baseline-16pkt.cap). BC-2.01.009 v1.3.
- **M-6 (IDB options TLV walking + bounds checks):** IDB if_tsresol is an option carried in a
  TLV structure (code:2+len:2+padded value); raw path must parse options TLV with bounds
  checks. No spec for malformed-option-length → E-INP-008. Fix: IDB options-walk postcondition
  added; malformed option length routes to E-INP-008. BC-2.01.011 v1.3. error-taxonomy v2.9.
- **M-7 (IDB-parse error-code precedence — Decision 17):** Precedence undefined among E-INP-013
  (interleaved IDB), E-INP-001 (whitelist), E-INP-011 (conflict) at IDB-parse time. Fix: order
  defined as E-INP-013 position-check first, then E-INP-001 whitelist, then E-INP-011 conflict.
  ADR-009 rev 6 Decision 17. BC-2.01.016 v1.3 / BC-2.01.018 v1.3 (precedence note).
- **M-1 (BC-2.01.013 traceability path):** HS-107 path in traceability cited
  `.factory/specs/holdout-scenarios/` (does not exist); corrected to `.factory/holdout-scenarios/`.
  BC-2.01.009 v1.3 / BC-2.01.013 v1.3.

**Low/observation findings (4):**

- **O-1 (HS-104 PC5 re-cite):** HS-104 cited BC-2.01.012 PC3/PC4 for interface_id cases;
  corrected to PC5. HS-104 v1.1.
- **O-2 (HS-107 stale byte lines):** HS-107 Case A/D contained stale pre-correction hex lines
  (pre-D-143/D-144). Removed; only corrected values remain. HS-107 v1.1.
- **O-3 (stale forward-reference notes):** Stale "taxonomy updated in separate burst" notes for
  error codes that have since landed. Removed from affected BCs. BC-2.01.017 v1.4.
- **O-4 (VP-INDEX arithmetic):** CLOSED — no action required.

**Version bumps in this burst (~18 artifacts):**

| Artifact | Version |
|----------|---------|
| BC-2.01.009 | v1.2 → v1.3 |
| BC-2.01.010 | v1.7 → v1.8 |
| BC-2.01.011 | v1.2 → v1.3 |
| BC-2.01.012 | v1.2 → v1.3 |
| BC-2.01.013 | v1.2 → v1.3 |
| BC-2.01.014 | v1.2 → v1.3 |
| BC-2.01.015 | v1.3 → v1.4 |
| BC-2.01.016 | v1.2 → v1.3 |
| BC-2.01.017 | v1.3 → v1.4 |
| BC-2.01.018 | v1.2 → v1.3 |
| error-taxonomy | v2.8 → v2.9 |
| ADR-009 | rev 5 → rev 6 |
| VP-INDEX | v2.4 → v2.5 |
| verification-architecture.md | v2.0 → v2.1 |
| verification-coverage-matrix.md | v1.14 → v1.15 |
| HS-103 | v1.3 → v1.4 |
| HS-104 | v1.0 → v1.1 |
| HS-107 | v1.0 → v1.1 |
| BC-INDEX | v1.55 → v1.56 |

**Decision log:** D-147.

---

## [pcapng-f2-pass2-remediation-2026-06-19] — 2026-06-19

### PASS-2 ADVERSARIAL REMEDIATION: 4 CRITICAL / 8 HIGH / 6 MEDIUM / 6 LOW — HIGH novelty

**Trigger:** F2 adversarial pass-2 identified 4C/8H/6M/6L findings, HIGH overall novelty class
(new wire-format + partial-fix-regression findings not anticipated by pass-1 remediation). All
must-fix items remediated in this burst. Pass-3 is next; F3 remains BLOCKED until 3 clean passes.

**Critical findings (4):**

- **C-1 (pass-2 re-verification):** IDB snaplen offset 4-7 — BC-2.01.010 had the wrong byte
  offset for the `snaplen` field in the SHB body. Corrected to offset 4–7 (bytes within the
  SHB body after the byte-order magic u32). BC-2.01.010 v1.7.
- **C-2 (pass-2 re-verification):** HS-107 authored for SPB framing/snaplen holdout
  (BC-2.01.013 / VP-028 gap). HS-INDEX v2.1 (107 scenarios, all-namespace 180).
- **C-3 (pass-2 re-verification):** EPB frame overhead confirmed 12 bytes above payload
  (EPB fixed header is 20 bytes: block_type 4 + block_total_length 4 + interface_id 4 +
  timestamp_high 4 + timestamp_low 4 + captured_packet_length 4 + original_packet_length 4 =
  28 bytes total; data padding is after payload; overhead above aligned payload is 12 bytes
  of fixed header after the 16-byte prefix). ADR-009 rev 5 Decision 8 updated.
- **C-4 (NEW — stale error codes in BC-2.01.017):** BC-2.01.017 v1.2 listed only
  E-INP-008..E-INP-011; E-INP-012 and E-INP-013 were missing from the error-code table.
  This is a partial-fix-regression: the D-142 remediation added E-INP-012 to error-taxonomy
  but failed to sweep the cross-cutting parent BC-2.01.017 (the "errors surface" BC that
  must enumerate all error codes). Fixed: BC-2.01.017 v1.3 (E-INP-013 added; full table
  E-INP-008..E-INP-013). Triggered sibling-sweep lesson: cross-cutting parent BCs must be
  included in every error-code routing fix sweep.

**High findings (8):**

- **I-1 (VP re-anchor):** VP-025..030 annotations in VP-INDEX desynchronized from on-disk
  BC versions after the D-142 remediation. VP-INDEX v2.4 re-anchors all VP-to-BC citations.
- **I-2 (Kani unwind note):** BC-2.01.014 / BC-2.01.010 both lacked a Kani `#[kani::unwind]`
  bound specification for the base-10 power-of-ten loop; without a bound the harness loops
  forever. ADR-009 rev 5 §VP-025 note added; BC-2.01.014 v1.2 and BC-2.01.010 v1.7 include
  the impl note.
- **I-3 (zero-packet one-shot OPB notice):** BC-2.01.011 did not document the one-shot
  observation emitted when an OPB-only pcapng produces zero packets (SOUL #4 silent-failure
  scope). BC-2.01.011 v1.2 adds the notice.
- **I-4 (SPB bound re-statement):** SPB 16-byte overhead not clearly distinguished from the
  20-byte figure mentioned in adjacent prose. BC-2.01.013 v1.2 clarifies.
- **I-5/I-6 (interleaved-IDB → E-INP-013; ADR-009 Decision 15):** No BC specified what
  happens when an IDB appears after the first packet block. BC-2.01.015 v1.3 and BC-2.01.016
  v1.2 add the route; ADR-009 rev 5 Decision 15 is the architectural record. This was a
  HIGH-novelty finding: the interleaved-IDB scenario is a valid pcapng file that wirerust
  must explicitly reject rather than silently misbehave on.
- **I-7 (E-INP-008/010 boundary):** The threshold description for E-INP-008 vs E-INP-010
  was ambiguous when block_total_length is exactly at the SHB minimum (28 bytes). BC-2.01.010
  v1.7 and BC-2.01.012 v1.2 sharpen the boundary language.
- **I-8 (HS-completeness map):** ADR-009 rev 5 adds a forward-reference HS-completeness map
  (§HS-Completeness Map) listing all framing BCs and their required holdout scenarios;
  resolves I-14. This map was absent from rev 4, leaving no machine-checkable record of
  which BCs lacked holdout coverage.

**Medium and Low findings (12):** I-9 (EPB boundary semantics), I-10 (OPB zero-packet
clarification), I-11 (verification-architecture.md v2.0 VP coherence), I-12
(verification-coverage-matrix.md v1.14 coverage update), I-13 (VP-INDEX v2.4 stale citations),
I-14 (HS-completeness gap — HS-107 now authored), plus 6 LOW items resolved inline in the
BC pass-2 annotations above.

**O-5 (verification-architecture/coverage-matrix VP coherence):** Addressed by the architect's
v2.0/v1.14 updates (verification-architecture.md v2.0, verification-coverage-matrix.md v1.14).

#### Version Bumps

| Artifact | Change | Version |
|----------|--------|---------|
| `specs/behavioral-contracts/ss-01/BC-2.01.009.md` | C-4 stale error-code sweep — E-INP-008..E-INP-013 added to error-code table. | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.010.md` | VP re-anchor (I-3); Kani unwind note (I-2); E-INP-008/E-INP-010 boundary clarification (I-9). | v1.6 → v1.7 |
| `specs/behavioral-contracts/ss-01/BC-2.01.011.md` | Zero-packet one-shot OPB-only notice (I-10); E-INP-013 cite for interleaved-IDB (Decision 15). | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.012.md` | EPB boundary I-9 clarification (captured_len vs block_total_length boundary semantics). | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.013.md` | SPB 16-byte bound re-stated for clarity; HS-107 authored (C-2/I-14). | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.014.md` | Kani unwind bound note (I-2); VP-025 precomputed base-10 table impl note. | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.015.md` | Interleaved-IDB E-INP-013 route (I-5/I-6; ADR-009 Decision 15). | v1.2 → v1.3 |
| `specs/behavioral-contracts/ss-01/BC-2.01.016.md` | Linktype-whitelist timing at IDB-parse time (I-5; ADR-009 Decision 15 amendment). | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.017.md` | C-4 stale codes — E-INP-013 added to error-code table; full table E-INP-008..E-INP-013; error-taxonomy v2.8. | v1.2 → v1.3 |
| `specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md` | Rev 5 — Decision 15 (interleaved-IDB → E-INP-013 reject); linktype-whitelist timing amendment; HS-Completeness Map; VP-025 Kani unwind note. | rev 4 → rev 5 |
| `specs/prd-supplements/error-taxonomy.md` | v2.8 — E-INP-013 added (interleaved-IDB late IDB reject; Decision 15); next_free = E-INP-014. | v2.7 → v2.8 |
| `specs/verification-properties/VP-INDEX.md` | v2.4 — VP re-anchor to on-disk BC versions; VP-025 Kani unwind note. | v2.3 → v2.4 |
| `specs/architecture/verification-architecture.md` | v2.0 — VP coherence update (O-5). | prior → v2.0 |
| `specs/architecture/verification-coverage-matrix.md` | v1.14 — Coverage update (O-5, I-12). | prior → v1.14 |
| `holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md` | New — SPB framing/snaplen holdout for BC-2.01.013 / VP-028 / C-2/I-14 gap. | new v1.0 |
| `holdout-scenarios/HS-INDEX.md` | v2.1 — HS-107 added; greenfield total 107; all-namespace total 180. | v2.0 → v2.1 |
| `specs/behavioral-contracts/BC-INDEX.md` | v1.55 — inline version annotations synced for BC-2.01.009..017 (9 BCs). | v1.54 → v1.55 |

#### Active BC Count

302 active BCs — unchanged.

---

## [pcapng-f2-reaudit-fixes-2026-06-19] — 2026-06-19

### RE-AUDIT CONSISTENCY FIXES: 6 findings + BOM-mapping contradiction chain resolved

**Trigger:** Post-remediation re-audit (consistency-validator pass) of the D-142 remediation
burst identified 6 findings + a 4-document byte-order-magic contradiction chain. All findings
fixed in this burst. Adversary reconvergence (pass 2) is next.

**Findings resolved:**

- **H5-1 (HIGH):** BC-2.01.009 PC1 over-promised "at least one readable packet" — contradicts
  valid empty pcapng (BC-2.01.002 EC-001 parity) and OPB-only zero-packet case. Fixed: PC1
  reworded to ">=0 packets" in BC-2.01.009 v1.1.
- **BOM-mapping contradiction chain (MEDIUM aggregate):** 4-document error chain where BE/LE
  byte-order-magic shorthand ("0xVALUE") was read-convention-dependent, causing mutually
  contradictory assertions across ADR-009, BC-2.01.010, and HS-103. Root cause: ADR-009 had
  a mislabeled BE magic value that propagated to BC-2.01.010 v1.4 and HS-103 v1.0. Fixed:
  - BOM-3 (HS-103): ADR-009 rev 4 minor correction 2 — BE byte-order magic correctly stated
    as on-disk bytes `1A 2B 3C 4D`; HS-103 v1.2 corrects Case A BOM bytes to `1A 2B 3C 4D`.
  - BOM-2 (MEDIUM, HS-103): block_total_length encoding notation corrected (u32 not u64).
  - BOM-1 (LOW, BC-2.01.010): AC-001 parenthetical circular phrasing removed (v1.5).
  - BOM consistency sweep (BC-2.01.010 v1.6): 9 statements normalized to unambiguous
    on-disk byte-sequence form; v1.4 annotation corrected.
- **PRD-BC2-1 (MEDIUM):** PRD §2.1 BC-2.12.011 description still said extension-based
  filtering (stale pre-v1.5 text). Fixed: prd.md v1.33 §2.1 updated to magic-byte
  content detection; §7 RTM also synced (v1.32→v1.33).
- **H2-1 (LOW):** ADR-009 PO dispatch SPB formula used `btl-20` (wrong); corrected to
  `btl-16` (ADR-009 rev 4 minor correction 1).
- **IDX-1 (LOW):** HS-INDEX version comment said all-namespace=173; Totals table correctly
  shows 179. Fixed: version comment corrected to 179 in HS-INDEX v2.0 annotation.

#### Version Bumps

| Artifact | Change | Version |
|----------|--------|---------|
| `specs/prd.md` | §2.1 BC-2.12.011 description updated to magic-byte detection; §7 RTM synced (D-142 error-code routing + VP/story anchors). | v1.32 → v1.33 |
| `specs/behavioral-contracts/ss-01/BC-2.01.009.md` | PC1 over-promise fixed: ">=0 packets"; H5-1 FIXED. | v1.0 → v1.1 |
| `specs/behavioral-contracts/ss-01/BC-2.01.010.md` | BOM byte-sequence consistency sweep — 9 statements normalized; circular AC-001 parenthetical removed; BOM-1 + BOM-mapping contradiction chain FIXED. | v1.4 → v1.6 (via v1.5) |
| `specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md` | Rev 4 minor corrections 1+2: SPB overhead formula corrected to btl-16 (H2-1); BE byte-order magic corrected to on-disk bytes `1A 2B 3C 4D` (BOM-3/BOM-mapping root cause). | rev 4 (minor corrections) |
| `holdout-scenarios/HS-103-pcapng-shb-framing-byte-order-and-error-cases.md` | Case A BOM on-disk bytes corrected from `4D 3C 2B 1A` (LE) to `1A 2B 3C 4D` (BE); block_total_length encoding corrected (BOM-2 + BOM-3 FIXED). | v1.0 → v1.2 |
| `holdout-scenarios/HS-INDEX.md` | Version comment all-namespace count corrected: 173 → 179 (IDX-1 FIXED). | v2.0 (comment corrected) |
| `specs/behavioral-contracts/BC-INDEX.md` | Inline version annotations synced: BC-2.01.009 v1.0→v1.1; BC-2.01.010 v1.4→v1.6. | v1.53 → v1.54 |
| `cycles/feature-pcapng-reader/f2-consistency-audit.md` | Re-audit section appended: 6 findings documented, status FIXED. | updated |

#### Active BC Count

302 active BCs — unchanged.

---

## [pcapng-f2-remediation-2026-06-19] — 2026-06-19

### REMEDIATION: F2 Deep-Validation Findings — raw-block pivot + 3 PO bursts + holdout authoring

**Trigger:** F2 deep-validation (adversary + security + performance) identified 29 unique findings (3C/6H/7M/3L
adversary, 0C/2H/4M/3L security, 3H/2M/1L performance). This entry consolidates the remediation burst:
ADR-009 rev 4 (architectural raw-block pivot), VP-025..030 assignment (resolves C-3/DF-CANONICAL-FRAME-HOLDOUT-001),
BC and taxonomy revisions, HS-101..106 holdout authoring, NFR additions, STORY-128 scoping.

#### Version Bumps

| Artifact | Change | Version |
|----------|--------|---------|
| `specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md` | Rev 4 ARCHITECTURAL PIVOT — raw-block API (Decision 2 revised); SPB overhead 16 bytes (Decision 8, resolves H-2); snaplen enforcement via raw block (Decision 9, resolves O-4); panic surface documented (Decision 10, resolves SEC-008); directory glob extended to .pcapng/.cap (Decision 11, resolves C-2). | rev 3 → rev 4 |
| `specs/verification-properties/VP-INDEX.md` | VP-025..030 added: VP-025 Kani timestamp totality (BC-2.01.014), VP-026 Kani SHB parse safety (BC-2.01.010), VP-027 Kani EPB parse safety (BC-2.01.012), VP-028 cargo-fuzz no-panic (BC-2.01.017), VP-029 proptest block-walk skip (BC-2.01.015), VP-030 proptest multi-IDB totality (BC-2.01.018). Resolves C-3/DF-CANONICAL-FRAME-HOLDOUT-001. total 24→30. | v2.2 → v2.3 |
| `specs/behavioral-contracts/ss-01/BC-2.01.010.md` | Saturating arithmetic reference (ts formula); no-panic AC (SEC-005); VP-026 assigned. | v1.2 → v1.4 |
| `specs/behavioral-contracts/ss-01/BC-2.01.011.md` | if_tsresol API-spike result documented; VP-025 scope note. | v1.0 → v1.1 |
| `specs/behavioral-contracts/ss-01/BC-2.01.012.md` | EPB_FIXED_OVERHEAD_BYTES=28 named constant; guard-before-allocate AC; E-INP-009 routing corrected (H-3/SEC-003); VP-027 assigned. | v1.0 → v1.1 |
| `specs/behavioral-contracts/ss-01/BC-2.01.013.md` | SPB overhead corrected to 16 bytes (H-2); no-panic AC (SEC-005). | v1.0 → v1.1 |
| `specs/behavioral-contracts/ss-01/BC-2.01.014.md` | Saturating arithmetic for base-2 (checked_shl) and base-10 branches; overflow guards; VP-025 Kani obligation (full u8 space). | v1.0 → v1.1 |
| `specs/behavioral-contracts/ss-01/BC-2.01.015.md` | Forward-progress invariant AC: block_total_length>=8 guard (SEC-002/CWE-835); VP-029 assigned. | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.016.md` | No-panic AC (SEC-005). | v1.0 → v1.1 |
| `specs/behavioral-contracts/ss-01/BC-2.01.017.md` | VP-028 cargo-fuzz target assigned; E-INP-009 routing corrected. | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-01/BC-2.01.018.md` | Per-file isolation AC re-attributed to STORY-128 (main.rs loop); OPB-only zero-packet case documented (H-4/H-5); VP-030 assigned. | v1.1 → v1.2 |
| `specs/behavioral-contracts/ss-12/BC-2.12.011.md` | Magic-byte content detection for .pcapng/.cap files in directory mode; glob expansion revised to include pcapng; C-2 resolved; STORY-127 scope anchor. | v1.4 → v1.5 |
| `specs/behavioral-contracts/BC-INDEX.md` | Inline version annotations synced to on-disk frontmatter for all 10 changed BCs + BC-2.12.011. Active BC count unchanged: 302. | v1.52 → v1.53 |
| `specs/prd-supplements/error-taxonomy.md` | E-INP-009 (EPB-before-IDB orphan resolved; H-3/SEC-003); E-INP-010 (3-failure-mode/2-template ambiguity resolved; EPB interface_id OOB added; M-3); E-INP-012 Notes corrected (routing clarified). | v2.6 → v2.7 |
| `specs/prd-supplements/nfr-catalog.md` | NFR-PERF-005 (eager memory model explicit declaration); NFR-PERF-006 (throughput floor >=500 MB/s); NFR-PERF-007 (pcapng vs. classic regression budget <=10%). | v2.2 → v2.3 |
| `holdout-scenarios/HS-101..106` | NEW — 6 pcapng holdout scenarios authored: HS-101 (tsresol microsecond regression), HS-102 (timestamp overflow saturating guards), HS-103 (SHB framing/byte-order), HS-104 (EPB framing/interface-id bounds), HS-105 (block-walk skip/forward-progress), HS-106 (multi-IDB linktype agreement). Tied to VP-025..030. DF-CANONICAL-FRAME-HOLDOUT-001 satisfied. | n/a → v1.0 each |
| `holdout-scenarios/HS-INDEX.md` | HS-101..106 registered. Greenfield total: 106. | v1.x → v2.0 |
| `specs/architecture/verification-architecture.md` | VP-025..030 rows added to Provable Properties Catalog. | updated |
| `specs/architecture/verification-coverage-matrix.md` | VP-025..030 rows added; reader.rs module row added; totals updated. | updated |

#### Active BC Count

302 active BCs — unchanged. No new BCs created; no retirements this burst.

#### F3 Scope Items (recorded, not yet decomposed)

- **STORY-128** (NEW — main.rs per-file isolation loop): per-file isolation from BC-2.01.018 AC re-attributed here. F3 decomposition scope.
- **STORY-127** (magic-byte glob + corpus + BC-2.12.011 impl): scope confirmed. F3 decomposition scope.
- **HS-001 rewrite**: deferred to F3 (cites retired BC-2.01.004; PO action).
- **PRD §7 RTM sync**: may need follow-up for new error-codes (E-INP-009/010 routing) and VP/story anchors (VP-025..030, STORY-128). Flagged for orchestrator routing to PO.

#### cargo-fuzz deferral

Fuzz harness implementation (VP-028) deferred to F6 (formal hardening phase), per pipeline convention. Spec obligation recorded in VP-028 and BC-2.01.017. cargo-fuzz toolchain provisioning is an F6 prerequisite.

---

## [pcapng-multisection-decision-correctness-2026-06-19] — 2026-06-19

### PATCH: Correctness Edits from pcapng Multi-Section Decision Research

**Trigger:** Source-level verification of pcap-file 2.0.0 (`.factory/research/pcapng-multisection-decision.md`)
confirmed that the INCONCLUSIVE premise in F-06 of the completeness validation was FALSE: pcap-file 2.0.0
DOES correctly reset interface state per section. F-06's feared mis-attribution (analogous to a historical
Wireshark bug) does not apply to the library wirerust actually uses.

**Decision unchanged:** REJECT second SHB → E-INP-012 is retained. The rationale is now correctly framed
as a SCOPE decision (multi-section pcapng is rare, absent from corpus, and out of scope for this cycle),
not a correctness workaround for a library defect.

#### Changes

| Artifact | Change | Version |
|----------|--------|---------|
| `prd-supplements/error-taxonomy.md` | E-INP-012 message updated: added actionable remediation hint (`mergecap -F pcapng` / `editcap`). Notes reframed: rejection is scope decision, not correctness workaround; pcap-file 2.0.0 handles multi-section correctly (source-verified 2026-06-19). | v2.4→v2.5 |
| `ss-01/BC-2.01.010.md` | AC-002 rationale corrected: scope decision framing; pcap-file 2.0.0 library correctness noted. AC-002 and EC-006 updated to reference E-INP-012 remediation hint. Invariant 1 rewritten to distinguish scope vs. correctness. Error Taxonomy row updated. Normative behavior (reject second SHB → E-INP-012) unchanged. | v1.1→v1.2 |
| `research/pcapng-spec-completeness-validation.md` | F-06 annotated with SUPERSEDED PREMISE notice: INCONCLUSIVE finding overturned by source-level verification; library does reset correctly; reject retained as scope decision. | annotation only (no version bump) |
| `spec-changelog.md` | This entry | — |

#### Normative behavior: UNCHANGED

- BC-2.01.010 AC-002: second SHB → `Err` → E-INP-012. Unchanged.
- E-INP-012: severity `broken`, exit 1. Unchanged.
- BC count: 302 active. Unchanged.
- ADR-009: not touched (architect owns).

#### What changed

- **E-INP-012 message:** added ` (hint: split the capture into single-section files, or re-save with 'mergecap -F pcapng' or 'editcap' which emit single-section pcapng)` to the message format.
- **Rationale text** in BC-2.01.010 AC-002, Invariant 1, EC-006, and error-taxonomy.md Notes: changed from implying library-level uncertainty to explicitly stating this is a scope constraint with a correct library underneath.
- **F-06 annotation:** completeness-validation research doc updated to record the superseded premise for audit continuity.

---

## [pcapng-completeness-f06-f07-f11-2026-06-19] — 2026-06-19

### PATCH: pcapng Completeness Deltas F-06, F-07, F-11 — AC-Level Additions Only

**Trigger:** pcapng-spec-completeness-validation identified three MEDIUM-severity gaps in the
F2 pcapng BC set. All three are addressed as AC additions to existing BCs and an E-INP-012
error taxonomy entry. No new BCs were created; active BC count is unchanged at 302.

#### Changes

| Artifact | Change | Version |
|----------|--------|---------|
| `ss-01/BC-2.01.010.md` | F-06: AC section added (AC-001..004); EC-006 changed from "reset byte order" to REJECT with E-INP-012; Canonical Test Vector added for 2-section pcapng; Verification Property added for second-SHB Err guarantee; Invariant 1 rewritten to reflect single-section scope; E-INP-012 cross-reference added to Traceability | v1.0→v1.1 |
| `ss-01/BC-2.01.015.md` | F-07: AC section added (AC-001..003) enumerating ALL skip-arm variants (NRB, ISB, DSB, SystemdJournalExport, OPB 0x2, Unknown); Invariant 2 rewritten with full variant list + type codes; EC-008..011 added for NRB, DSB, SystemdJournal, and mixed-OPB+EPB | v1.0→v1.1 |
| `ss-01/BC-2.01.018.md` | F-11: AC section added (AC-001..002); AC-001 refines E-INP-011 message hint (tcpdump -i any actionable guidance); AC-002 specifies per-file error isolation in directory mode (E-INP-011 fails per-file, not aborting full run); EC-009 added for directory mode partial-failure scenario | v1.0→v1.1 |
| `prd-supplements/error-taxonomy.md` | F-06: E-INP-012 added (multi-section SHB reject, broken, exit 1); F-11: E-INP-011 Notes refined with actionable tcpdump hint and per-file isolation note; next_free = E-INP-013 | v2.3→v2.4 |
| `prd.md` | Version 1.30→1.31 delta note added (no RTM row changes required — no new BCs, no new subsystems; E-INP-012 is an error taxonomy entry only) | v1.30→v1.31 |
| `spec-changelog.md` | This entry | — |

#### F-06: Multi-Section SHB Reject (BC-2.01.010, E-INP-012)

**Decision:** REJECT second/mid-stream SHB with a clear error rather than attempt to verify
and reset byte-order context. Rationale: the entire intended corpus is single-section pcapng;
multi-section support would require non-trivial interface-table bookkeeping and is out of scope.

**E-INP-012 definition:**
- Code: `E-INP-012`
- Category: `INP`
- Severity: `broken`
- Exit code: 1
- Message format: `pcapng multi-section files are not supported (second Section Header Block at block #<seq>)`
- BC reference: BC-2.01.010, BC-2.01.017

#### F-07: Enumerate Skip-Arms (BC-2.01.015)

All pcap-file Block variants that are NOT SHB/IDB/EPB/SPB must have explicit match arms in
the implementation to prevent omitted-arm panics. Enumerated in AC-001:
- NameResolutionBlock (NRB, `0x00000004`) — no packet data
- InterfaceStatisticsBlock (ISB, `0x00000005`) — no packet data
- DecryptionSecretsBlock (DSB, `0x0000000A`) — no packet data
- SystemdJournalExportBlock (`0x00000009`) — no packet data
- Obsolete Packet Block (OPB, `0x00000002`) — carries packet data but is obsolete/out-of-scope; skipped; EPB interpretation unaffected
- Unknown/future block types — skipped via block_total_length

#### F-11: Actionable Conflict Error + Per-File Failure (BC-2.01.018, E-INP-011)

E-INP-011 message now includes an actionable hint identifying `tcpdump -i any` as the
most common trigger and stating the wirerust requirement (single link type per file).
BC-2.01.018 AC-002 explicitly specifies that in directory mode, E-INP-011 on one file
does NOT abort the full run (per-file isolation consistent with BC-2.12.011 directory-mode
contract).

#### No-Touch Scope

- ADR-009: not touched (architect owns)
- epics.md: not touched
- BC count: 302 active (unchanged)

---

## [pcapng-f2-2026-06-19] — 2026-06-19

### MINOR: F2 pcapng Reader Support — Spec Evolution INTEGRATE Sub-Burst (FE-001, ADR-009)

**Trigger:** F2 pcapng-reader-support CREATE sub-burst produced 10 new BC files (BC-2.01.009–018) and staged error taxonomy addendum. This INTEGRATE sub-burst splices all artifacts into master spec indexes.

#### Changes

| Artifact | Change | Version |
|----------|--------|---------|
| `BC-INDEX.md` | 10 new SS-01 rows (BC-2.01.009–018); BC-2.01.004 struck as [RETIRED]; BC-2.01.001 v1.6→v1.7 annotation; BC-2.01.002 v1.5→v1.6 annotation; total active BCs 293→302; header status updated | v1.51→v1.52 |
| `ss-01/BC-2.01.004.md` | RETIRED: lifecycle_status active→retired; deprecated_by/replacement/retired/removal_reason set; superseded_by: BC-2.01.009; v1.4→v1.5 | v1.4→v1.5 |
| `ss-01/BC-2.01.001.md` | EC-005 updated: pcapng now routed via BC-2.01.009 before reaching this BC's classic-pcap path | v1.6→v1.7 |
| `ss-01/BC-2.01.002.md` | Description/Preconditions scoped to classic-pcap branch; F2 scope note added | v1.5→v1.6 |
| `prd-supplements/error-taxonomy.md` | E-INP-008..011 added (pcapng block parse failures); E-INP-002 "or pcapng format" note removed; staging addendum marked CONSUMED | v2.2→v2.3 |
| `prd-supplements/nfr-catalog.md` | NFR-COMPAT-001 revised: pcapng is now SUPPORTED; test reference updated | v2.1→v2.2 |
| `prd-supplements/test-vectors.md` | BC-2.01.004 test vector struck/annotated STALE; BC-2.01.009 test vectors added | v2.1→v2.2 |
| `prd.md` | §1.5 pcapng out-of-scope item struck/removed; §2.1 table updated (BC-2.01.004 retired, BC-2.01.009–018 added) | v1.28→v1.29 |
| `specs/module-criticality.md` | PCAP reader row: BC-2.01.004 reference updated to BC-2.01.009 | v1.4 (note added) |

#### BC Retirement

- **BC-2.01.004** "Reject pcapng-Format Input at Reader Level" — **RETIRED** (behavioral inversion).
  - Reason: pcapng is now accepted, not rejected. The postconditions of BC-2.01.004 (pcapng → Err) are exactly inverted by BC-2.01.009 (pcapng → Ok via magic-byte probe).
  - `superseded_by: BC-2.01.009`
  - `test_BC_2_01_004_rejects_pcapng` must be rewritten as a positive acceptance test (STORY-123 scope).
  - ID BC-2.01.004 is never reused (append-only-numbering policy).

#### New BCs

10 new greenfield BCs (SS-01): BC-2.01.009 (magic probe), BC-2.01.010 (SHB parse), BC-2.01.011 (IDB parse), BC-2.01.012 (EPB parse), BC-2.01.013 (SPB parse), BC-2.01.014 (timestamp normalization, Kani target), BC-2.01.015 (unknown block skip), BC-2.01.016 (pcapng link-type gating, CAP-02), BC-2.01.017 (error surface), BC-2.01.018 (multi-IDB agreement, fail-closed).

#### Deferred / Not Yet Addressed

- **BC-2.12.011** ("Directory Target Expands to *.pcap Sorted; *.pcapng Excluded"): This BC still describes the OLD behavior (pcapng excluded from glob). The F1 delta specifies that STORY-127 must update the glob to include `*.pcapng`. BC-2.12.011 will need a revision (or retirement + replacement) when STORY-127 is decomposed. **Not touched in this burst** — STORY-127 is the implementing story; BC-2.12.011 retirement/revision is an F3 task.
- **Downstream story input-hashes**: STORY-123, STORY-124, STORY-125, STORY-126, STORY-127 (when decomposed) will need their `input-hash:` fields recomputed after this INTEGRATE burst. Run `bin/compute-input-hash --write --scan` at F3 entry per CLAUDE.md policy.
- **Stories citing BC-2.01.004**: STORY-001 body cites BC-2.01.004 in its BC table and AC-006. Story-writer must re-anchor STORY-001 per bc_array_changes_propagate_to_body_and_acs policy (see task output below).

---

## [story-119-f2-adv-round5-remediation-2026-06-18] — 2026-06-18

### PATCH: STORY-119 F2 Adversarial Round-5 Remediation — BC-030 Content-Based Citation (ends line-number churn); STORY-119 BC-030 Stamp Re-Sync v1.2→v1.4; ADR-0003 Binary-Crate False Premise Corrected; Research-Doc Correction Note

**Trigger:** F2 adversarial round-5 triple — Pass A CLEAN; Pass B found BC-030 stamp drift in STORY-119 body table + ADR-0003 binary-crate false premise; Pass C found BC-INDEX grouped-collapse entry line-citation STILL wrong (churns on every changelog prepend). ALL REMEDIATED. BC-INDEX v1.48 → v1.49. Re-streak pending (round-6 triple).

#### Root Cause

Pass A was CLEAN (all round-4 fixes verified). Passes B and C independently found three self-inflicted churn defects:

1. **MED (F-R5B-001): STORY-119.md BC-030 stamp drift.** The round-4 burst bumped BC-030 from v1.2 to v1.3, but the STORY-119.md body BC table still showed v1.2. This is the same consuming-surface propagation gap (PG-62-F2-BOOKKEEPING-SWEEP-001 family) — the BC file version advanced but the story's version-stamp column was not updated. Re-synced to v1.4 (which also incorporates the round-5 content-based citation fix).

2. **HIGH (F-R5B-002): ADR-0003 alternatives note false premise — 'binary crate / no [lib] target'.** The ADR-0003 alternatives-rejected section (added during round-4 attribution-correction burst) stated that wirerust has no `[lib]` target and therefore FindingsRender has no downstream semver consumers. This is incorrect: `src/lib.rs` exists and is a public library; `FindingsRender` is exported by it and is part of the public API surface. The DECISION REMAINS VALID because v0.9.0 is still unreleased (the breaking change is fully contained by the 0.9.0 version bump). The false-premise prose was corrected in ADR-0003 and a correction note was added to the research doc.

3. **[process-gap] (F-R5C-001): BC-INDEX grouped-collapse line-citation still wrong — churns on every prepend.** The v1.48 BC-INDEX changelog entry cited "line 273" as the location of the grouped-collapse v0.9.0 entry. The v1.47 entry cited ":271"; v1.46 said ":269". All were wrong at the time they were written — each changelog prepend shifts all subsequent line numbers. The BC-030 body stanzas (v1.1/v1.2/v1.3) also carried the successive wrong line numbers as their provenance citations. PERMANENTLY FIXED by switching ALL grouped-collapse provenance citations to content-based references: the canonical citation is now 'BCs 030-034: grouped-collapse (greenfield, STORY-119, v0.9.0)' — the content of that BC-INDEX section heading, which does not shift when entries are prepended.

#### Changed Artifacts (this burst)

**BC-2.11.030.md v1.3→v1.4 — content-based citation (permanent fix):**
- v1.4 stanza added: grouped-collapse provenance citation switched from fragile BC-INDEX line number to content-based reference: BC-INDEX entry 'BCs 030-034: grouped-collapse (greenfield, STORY-119, v0.9.0)'.
- All ":NNN" line-number citations for this provenance removed from the v1.1 stanza.
- v1.2 and v1.3 stanzas annotated as SUPERSEDED (the successive :271 and :273 corrections were each themselves wrong due to changelog-prepend drift) but retained as audit trail of the round-3 and round-4 intermediate errors.

**STORY-119.md v1.3→v1.4 — BC-030 stamp re-sync:**
- BC-030 version cell in the body Behavioral Contracts table corrected from v1.2 to v1.4 (incorporating round-3 →v1.2, round-4 →v1.3, and round-5 →v1.4 in one update).
- All other BC version stamps confirmed correct (no other drift).

**ADR-0003 (on develop, rides F4 PR) — binary-crate false premise corrected:**
- The alternatives-rejected section's "binary crate / no [lib] target" premise corrected to acknowledge src/lib.rs is a public library and FindingsRender is public API.
- Decision rationale updated: the breaking change is contained by the unreleased v0.9.0 version bump, not by the absence of consumers.

**Research doc story-119-render-mode-typedesign.md — correction note added:**
- Correction note prepended to the "Key Finding First" section explaining the false premise, why the decision remains valid, and that the correction is preserved as audit trail.

**BC-INDEX.md v1.48→v1.49:**
- v1.49 changelog entry prepended (this burst).
- v1.48 entry's ":273 / line 273" BC-030 citation reworded to content-based phrasing (no line number).
- BC-030 row annotation extended with v1.4 content-based citation note.
- Version stamp bumped to v1.49.

**Artifacts NOT changed this round:**
- BC-2.11.031, 032, 033, 034 (untouched)
- BC-2.11.014 (untouched)
- HS-081 (untouched)
- PRD-delta (untouched)
- No new BCs; total 293 unchanged.

---

## [story-119-f2-adv-round4-remediation-2026-06-18] — 2026-06-18

### PATCH: STORY-119 F2 Adversarial Round-4 Remediation — Type-Attribution Inversion Fix; BC-030 Provenance Citation Corrected; Design-Note §5.1 Sort-Clause Dedup; BC-INDEX:032 Annotation Alignment; "9-BC"→"12-BC" Count Fix

**Trigger:** F2 adversarial round-4 triple — Pass A CLEAN; Pass C CLEAN (LOW only); Pass B found HIGH type-attribution inversion + MED stale BC count + LOW BC-030 provenance citation + 2 NITs. BC-INDEX v1.47 → v1.48. ADR untouched this round.

#### Root Cause

Round-4 Passes A and C were CLEAN. Pass B found four defect classes:

1. **HIGH (F-R4B-001): STORY-119.md type-attribution inversion.** The narrative (~8 occurrences) incorrectly attributed the FindingsRender STRUCT introduction to STORY-120. Correct: STORY-120 introduces the three-variant FindingsRender ENUM (`FindingsRender::Grouped`, `FindingsRender::FlatCollapsed`, `FindingsRender::FlatExpanded`); STORY-119 then evolves that enum into the struct-of-orthogonal-enums (`FindingsRender { grouping: Grouping, collapse: Collapse }`). The Scope note also contained a self-contradiction: said "v0.9.0 dispatched the enum" but implied `{Grouped,Collapsed}` was already representable (it was not — that illegal state is what STORY-119 resolves).
2. **MED: Stale "9-BC set" in v1.1 stanza** of STORY-119.md frontmatter comment. The v1.1 de-stale added the full 12-BC set but the description said "9-BC"; corrected to "12-BC set".
3. **LOW: BC-030 v1.2 provenance citation wrong.** The v1.1 stanza in BC-2.11.030.md cited "BC-INDEX:271" as the location of the grouped-collapse v0.9.0 entry; the v1.2 stanza retained that wrong cite. Verified: the live BC-INDEX shows the grouped-collapse v0.9.0 entry at line 273 (not :271). Both stanzas now cite :273 with verified entry content.
4. **NIT×2:** design-note §5.1 contained a duplicated sort clause (second occurrence removed); BC-INDEX:032 annotation used "consistent with BC-026 PC-7" language that was ambiguous — aligned to the corrected body wording ("shares the positional members[0] mechanic with BC-026 PC-7 but uses post-sort (not emission) order").

#### Changed Artifacts (this burst)

**STORY-119.md v1.2→v1.3 — type-attribution inversion fix:**
- All ~8 occurrences of incorrect attribution corrected throughout: frontmatter comment, status callout, narrative body, Implementation Scope, Dependencies section, Architecture Mapping table, Dependency Rationale section.
- Correct attribution everywhere: STORY-120 introduces the three-variant `FindingsRender` ENUM; STORY-119 evolves it into the struct-of-orthogonal-enums.
- Scope-note self-contradiction removed: clarified that `{Grouped,Collapsed}` was an unrepresentable/illegal state under the v0.9.0 three-variant enum (it was not merely "illegal at dispatch time" — the state did not exist).
- Stale "9-BC set" in v1.1 stanza corrected to "12-BC set".
- `depends_on: [STORY-120]` unchanged (correct).

**BC-2.11.030.md v1.2→v1.3 — provenance citation corrected:**
- `BC-2.11.030`: v1.1 stanza BC-INDEX line citation corrected from ":271" to ":273" with verified entry content ("BCs 030-034: grouped-collapse (greenfield, STORY-119, v0.9.0)").
- v1.2 stanza retained as audit trail of the round-3 intermediate error (the :271 fix was itself wrong — the round-3 NIT-fix overcorrected from :269 to :271 but the actual line is :273).

**Design-note §5.1 duplicate sort clause removed (NIT):**
- `story-119-type-design.md` §5.1: one duplicated sort-clause sentence removed (the paragraph stated the sort criterion twice in consecutive sentences).

**BC-INDEX.md v1.47→v1.48 — annotation alignment (NIT):**
- BC-2.11.032 row annotation aligned to corrected body wording: "...post-sort severity order; shares the positional members[0] mechanic with BC-026 PC-7 but uses post-sort (not emission) order."
- BC-2.11.030 row annotation extended with v1.3 citation note.
- Version stamp bumped to v1.48; round-4 changelog note prepended.

**Artifacts NOT changed this round:**
- ADR-0003 (no ADR change this round)
- BC-2.11.031, 033, 034 (untouched)
- HS-081 (untouched)
- PRD-delta (untouched)
- No new BCs; total 293 unchanged.

---

## [story-119-f2-adv-round3-remediation-2026-06-18] — 2026-06-18

### PATCH: STORY-119 F2 Adversarial Round-3 Remediation — STORY-119.md BC-Table Role-Description Verbatim-Title Fix, Design-Note Doc-Comment Annotation Correction, BC-032/034 Representative-Ordering Clarification, BC-030 Index-Anchor Nit

**Trigger:** F2 adversarial round-3 triple — Pass A CLEAN (all round-2 fixes verified); Passes B and C each found a NEW defect introduced by the round-2 STORY-119.md de-stale. BC-INDEX v1.46 → v1.47. Consuming-surface sweep CONFIRMED FULLY CONVERGED (Pass C exhaustive enum→struct census: zero forward-facing stale refs).

#### Root Cause

Round-3 Pass A confirmed round-2 remediation correct. Passes B and C independently found three defect classes:
1. STORY-119.md BC-table role-description column carried mis-anchored descriptions (10/12 rows wrong) — the same D-095 "from-memory" class, where the story-writer paraphrased role descriptions rather than reading verbatim BC H1 titles.
2. The design-note struct doc-comment BC annotations (§2.1) were mislabeled in the round-2 R2-7 fix (labels did not match the actual BC H1 titles for BC-2.11.030–034).
3. BC-2.11.032 and BC-2.11.034 overstated the representative sourcing in Inv3 as "consistent with BC-026 PC-7" without distinguishing: flat=emission-order, grouped-collapse=post-sort severity order (the two differ and BC-026 covers flat only).

#### Changed Artifacts (this burst)

**STORY-119.md BC-table role mis-anchor fix (v1.1→v1.2):**
- All 12 BC-table role-description cells corrected to verbatim BC H1 titles (extracted
  directly from the canonical BC files — D-095 lesson applied). Affected rows: all 12
  entries in the behavioral_contracts traceability table.
- Body text "9 behavioral contracts" corrected to "12 behavioral contracts" (9-BC count
  was a carry-over from the v1.0 stub that listed only [013,025,026]).
- VP-deferred comment added to clarify verification-property scope for STORY-119 (no new
  VP; F3 will carry the impl test obligations).

**Design-note struct doc-comment BC annotation correction:**
- `story-119-type-design.md` §2.1: `FindingsRender` struct doc-comment BC list corrected.
  Round-2 R2-7 added BC annotations but the labels for BC-2.11.030–034 were mislabeled
  (wrong titles copied). Corrected to verbatim H1 titles from each BC file.

**BC-2.11.032 v1.2→v1.3 — representative-ordering clarification:**
- `BC-2.11.032`: Invariant 3 reworded to clarify that "positionally first" applies
  WITHIN the post-sort order (severity-ascending, matching BC-2.11.014) — not
  emission-order. Added explicit note: flat=emission-order sourcing (BC-026 PC-7);
  grouped-collapse=post-sort severity order. This removes ambiguity against BC-026.

**BC-2.11.034 v1.2→v1.3 — representative-ordering clarification:**
- `BC-2.11.034`: Invariant 3 and Related-BCs updated to be explicit that the grouped
  representative (`members[0]`) is the first member in post-sort severity order, not
  emission-order. Related-BCs now documents the sourcing/ordering distinction between
  flat (BC-026, emission-order) and grouped-collapse (BC-032, post-sort).

**BC-2.11.030 v1.1→v1.2 — index-anchor nit:**
- `BC-2.11.030`: BC-INDEX annotation corrected from ":269" to ":271" (the nit-level
  index line-anchor was pointing at the wrong line in BC-INDEX).

**BC-INDEX v1.46→v1.47:**
- Table row annotations updated: BC-2.11.030 v1.2, BC-2.11.032 v1.3, BC-2.11.034 v1.3.
- v1.47 navigation changelog entry added.

---

## [story-119-f2-adv-round2-remediation-2026-06-18] — 2026-06-18

### PATCH: STORY-119 F2 Adversarial Round-2 Remediation — Verdict-Rank Table Corrected (Possible Added), Version-Pins v0.9.0, HS-081 Enum→Struct Sweep, STORY-119 De-Stale, BC-034 MITRE Cross-Ref Scoping, BC-031 Observable-Behavior Reword, Design-Note Struct Doc-Comment

#### Root Cause

F2 adversarial round-2 (all 3 passes NOT CLEAN) identified 7 findings (R2-1..R2-7) across
brownfield-extraction staleness, version-pin errors, unconsumed holdout sweep, stale story
stub, cross-reference scoping confusion, over-specified invariants, and doc-comment gaps.

#### Changed Artifacts (this burst)

**R2-1 CRITICAL — Verdict-rank table corrected (BC-2.11.014 v1.9→v2.0; propagated to BC-031/032/033 + design-note + ADR):**
- `BC-2.11.014`: Invariant 1 corrected from stale Likely=0/Inconclusive=1/Unlikely=2 to
  source-confirmed Likely=0/Possible=1/Inconclusive=2/Unlikely=3 per terminal.rs:447-454.
  Possible variant was added in STORY-109 but not reflected in the brownfield extraction.
  Description paragraph updated to include Possible in the rank ordering.
- `BC-2.11.031` v1.1→v1.2: PC-4 sort direction now lists all four verdicts
  (Likely=0/Possible=1/Inconclusive=2/Unlikely=3). Invariants 4 and 5 reworded to
  observable-behavior form (removed implementation-sharing/no-duplication prescriptions;
  stated externally testable color-selection and K=3 cap invariants instead — R2-6).
- `BC-2.11.032` v1.1→v1.2: Invariant 3 post-sort description lists all four verdicts.
- `BC-2.11.033` v1.1→v1.2: Description, PC-5, and Invariant 4 list all four verdicts.
- Design-note §5.1: sort-precision updated to list all four verdicts.
- ADR-0003: verdict-rank propagation (on develop working tree — rides F4 PR).

**R2-2 HIGH — Version-pin corrected to v0.9.0 (BC-2.11.030–034):**
- All five new BCs had `introduced: v0.10.0` (typo); corrected to `introduced: v0.9.0`
  per ADR-0003 §Semver, design-note §7, and D-110 bundling decision.
  BC-030 v1.0→v1.1; BC-031/032/033/034 v1.1→v1.2 (combined with R2-1 changes above).

**R2-3 HIGH — HS-081 holdout enum→struct migration (v1.0→v1.1; input-hash 9df8300→e62a96d):**
- `HS-081`: Verification Approach (unit-level construction call) migrated from
  `render = FindingsRender::Grouped` to `render = FindingsRender { grouping:
  Grouping::Grouped, collapse: Collapse::Expanded }`. Edge Conditions migrated from
  `render != FindingsRender::Grouped` to `render.grouping != Grouping::Grouped`.
  Input-hash recomputed after BC-2.11.014 (an HS-081 input) updated to v2.0.

**R2-4 HIGH — STORY-119.md de-stale (v1.0→v1.1):**
- `STORY-119.md`: Expanded `behavioral_contracts` list from [013,025,026] to full 12-BC
  set (adds 028,030,031,032,033,034). Removed `deferred: true` flag and do-not-dispatch
  language. Updated struct vocabulary throughout (enum variant refs → struct form).
  Updated terminal.rs anchor references (~272-323→~432-483). Removed false "grouped
  bypasses collapse" invariant. `deferred_reason` updated to F1/F2-complete status note.
  Full AC/task decomposition remains pending F3.

**R2-5 HIGH — BC-2.11.034 MITRE cross-ref scoping (v1.1→v1.2):**
- `BC-2.11.034`: Invariant 3 rescoped: BC-2.11.026 reference narrowed to representative
  SOURCING principle only (use members[0] as source); em-dash FORMAT precedent moved to
  BC-2.11.016 (the authoritative grouped-mode MITRE format contract). Related-BCs table
  updated to document the sourcing/format distinction explicitly.

**R2-6 MEDIUM — BC-2.11.031 observable-behavior reword (v1.1→v1.2; combined with R2-1):**
- `BC-2.11.031`: Invariants 4 and 5 converted from implementation-sharing prescriptions
  ("no duplication", "shared COLLAPSE_EVIDENCE_SAMPLES constant") to observable-behavior
  form: Inv4 states K=3 cap equivalence; Inv5 states identical color-selection outcome.

**R2-7 MEDIUM — Design-note struct doc-comment updated (R2-7):**
- `story-119-type-design.md` §2.1: `FindingsRender` struct doc-comment BC list extended
  to include all ten consuming BCs: BC-2.11.013/025/026/027/028/030/031/032/033/034.

**BC-INDEX v1.45→v1.46:**
- Version stamps updated: BC-2.11.014 v2.0, BC-2.11.030 v1.1, BC-2.11.031 v1.2,
  BC-2.11.032 v1.2, BC-2.11.033 v1.2, BC-2.11.034 v1.2.

---

## [story-119-f2-adv-round1-remediation-2026-06-18] — 2026-06-18

### PATCH: STORY-119 F2 Adversarial Round-1 Remediation — Sort-Direction Correction, Stale Enum Consuming-Surface Sweep, PRD-Delta Fixes, BC-026 PC-4 Flat-MITRE Reconcile, BC-034 EC-008, Test-Anchor Renumbering

**Trigger:** F2 adversarial round-1 triple — all 3 passes NOT CLEAN. All findings remediated; re-streak pending (round-2 triple). BC-INDEX v1.44 → v1.45. VP-016 v2.4 → v2.5.

#### CRITICAL — Sort-Direction desc↔asc Contradiction vs BC-2.11.014 (5 artifacts)

BC-2.11.014 (v1.9, code-extracted) prescribes within-bucket sort ascending by (verdict_rank, confidence_rank, emission order) — Likely=0/High=0 appear first. BC-2.11.031/032/033 + design note (`story-119-type-design.md`) + ADR-0003 (develop, uncommitted) all described the within-bucket sort as "descending" — directly contradicting BC-2.11.014.

Corrected: BC-2.11.031 v1.0→v1.1, BC-2.11.032 v1.0→v1.1, BC-2.11.033 v1.0→v1.1 — "descending" replaced with "ascending (Likely=0/High=0 first, matching BC-2.11.014)". Design note and ADR-0003 (develop) corrected to ascending. Test anchors in BC-2.11.031/033 renumbered.

| BC | Change | Before | After |
|----|--------|--------|-------|
| BC-2.11.031 | Within-bucket sort: desc → asc (matches BC-2.11.014); test anchors renumbered | v1.0 | v1.1 |
| BC-2.11.032 | Within-bucket sort: desc → asc (matches BC-2.11.014) | v1.0 | v1.1 |
| BC-2.11.033 | Within-bucket sort: desc → asc (matches BC-2.11.014); test anchors renumbered | v1.0 | v1.1 |

Artifacts also corrected: `.factory/phase-f2-spec-evolution/story-119-type-design.md` (design note). ADR-0003 on develop (uncommitted — rides with the F4 PR).

#### HIGH — PRD-Delta Default-Mode Correction ({Flat,Expanded} → {Flat,Collapsed})

PRD-delta §2.2 "default render mode for plain invocation" mistakenly listed `{Flat, Expanded}` as the neither-flag default. D-110 establishes `{Flat, Collapsed}` as the default (--mitre-less invocation) and `{Grouped, Collapsed}` as the --mitre default. Corrected in `.factory/phase-f2-spec-evolution/story-119-prd-delta.md`.

#### HIGH — PRD-Delta BC-034 Phantom Header Format Corrected

PRD-delta §4 "BC-2.11.034 MITRE line format" incorrectly described the grouped-collapse MITRE line as having a standalone header format (`<tactic>: (<xN>)`). The actual contract per BC-2.11.034 is that the MITRE expansion line (`— <technique name>`) is sourced from `members[0]` with no `(xN)` suffix on the MITRE line itself; (xN) appears only on the finding header line (BC-2.11.031). Corrected in `.factory/phase-f2-spec-evolution/story-119-prd-delta.md`.

#### HIGH (consuming-surface misses, recurrence of PG-62-F5) — Stale FindingsRender Enum-Variant Refs in BC-017/026/028 + VP-016

The F2 spec-evolution vocab-sweep (v1.44) migrated normative body text but left stale enum-variant refs in test-vector/EC body cells and VP-016 test-spec snippets:

- **BC-2.11.017** (v1.16→v1.17): test vector cells still referenced `FindingsRender::FlatExpanded` / `FindingsRender::FlatCollapsed` (old enum variants); migrated to `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }` / `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`.
- **BC-2.11.026** (v1.12→v1.13): PC-4 flat-MITRE behavior reconciled with BC-2.11.016/017 (PC-4 prose was inconsistent); stale enum-variant refs in test cells migrated to struct form.
- **BC-2.11.028** (v1.8→v1.9): stale enum-variant refs in EC cells migrated to struct form; test anchor renumbered.
- **VP-016** (v2.4→v2.5): test-spec code-block snippets at lines 116 and 147 still used `FindingsRender::Grouped` (old enum variant) as construction examples; migrated to `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.

| Artifact | Change | Before | After |
|----------|--------|--------|-------|
| BC-2.11.017 | Enum-variant refs in test-vector cells → struct form | v1.16 | v1.17 |
| BC-2.11.026 | PC-4 flat-MITRE reconciled; enum refs → struct form | v1.12 | v1.13 |
| BC-2.11.028 | Enum refs in EC cells → struct form; test anchor renumbered | v1.8 | v1.9 |
| VP-016 | Test-spec snippets :116/:147 enum→struct | v2.4 | v2.5 |

#### MEDIUM — BC-2.11.026 PC-4 Flat-MITRE Reconciliation

PC-4 of BC-2.11.026 described flat-mode MITRE behavior inconsistently with BC-2.11.016 (em-dash expansion) and BC-2.11.017 (no em-dash in default). Reconciled so PC-4 explicitly cites the per-mode behavior — MITRE line in grouped mode uses em-dash expansion (BC-2.11.016); MITRE line in flat mode emits ID(s) only (BC-2.11.017). Included in BC-2.11.026 v1.13.

#### MEDIUM — BC-2.11.034 EC-008 Multi-Tag Members Sharing [0]

EC-008 was absent from BC-2.11.034. Added: "Multi-tag finding where multiple members share the same `members[0]`: the MITRE line renders the technique name from `members[0].mitre_techniques[0]` (first technique of first member); additional techniques of `members[0]` are not rendered on the MITRE line." Included in BC-2.11.034 v1.1.

#### MEDIUM — Test-Anchor Renumbering (BC-2.11.031/033/034)

Test anchors in BC-2.11.031, BC-2.11.033, and BC-2.11.034 were renumbered to match the corrected sort-direction and EC additions. No behavioral change; anchor renumbering only.

#### BC-INDEX

BC-INDEX.md updated: v1.44 → v1.45. Inline comment stamps updated for 7 BCs (017 v1.17, 026 v1.13, 028 v1.9, 031 v1.1, 032 v1.1, 033 v1.1, 034 v1.1). Total BC count: 293 (unchanged; no new BCs).

**Root-cause note:** Consuming-surface sweep during F2 spec-evolution covered BC normative bodies but missed test-vector/EC cells and VP docs that embed struct construction snippets. Reinforces PG-62-F5-POSTMERGE-ANCHOR-001 / STORY-121 scope: consuming-surface sweep MUST cover VP docs, PRD-delta sections, and all test-vector/EC body cells, not just BC normative bodies.

---

## [story-119-f2-spec-evolution-2026-06-18] — 2026-06-18

### MINOR: STORY-119 F2 Spec-Evolution — Grouped-Collapse BCs, enum→struct Vocabulary Migration, Deferral-Clause Revisions, --no-collapse Dual-Scope

**Trigger:** STORY-119 F2 spec-evolution (D-111). FindingsRender reshaped from a 3-variant enum to `struct FindingsRender { grouping: Grouping, collapse: Collapse }` on develop (ADR-0003 updated, uncommitted). This makes grouping×collapse orthogonal (all 4 combos valid) — the struct is the idiomatic Rust representation. 5 new BCs authored for grouped-collapse behavior. 4 deferral/scope BCs revised. 8 BCs vocab-swept.

#### New BCs — Grouped-Collapse (5 new BCs, SS-11 29→34)

| BC | Title | Version |
|----|-------|---------|
| BC-2.11.030 | `--mitre` Default Maps to {Grouped, Collapsed}; `--mitre --no-collapse` Maps to {Grouped, Expanded} — CLI-to-Render Mode Mapping Contract | v1.0 |
| BC-2.11.031 | Per-Bucket Count Suffix — N≥2 Group Within a Tactic Bucket Renders Header with ` (xN)` Suffix; Singleton (N=1) Renders Without Suffix | v1.0 |
| BC-2.11.032 | Per-Bucket Evidence Sampling in Grouped-Collapse Mode — First min(N,K=3) Members Positionally; No Sliding Window | v1.0 |
| BC-2.11.033 | Tactic-Bucket Ordering Invariant Under Grouped-Collapse — Bucket Sequence Unchanged; Collapse Operates Within Buckets Only | v1.0 |
| BC-2.11.034 | MITRE Line Format in Grouped-Collapse — Em-Dash Name Expansion Sourced from Group Representative (`members[0]`); No `(xN)` on MITRE Line | v1.0 |

#### Revised BCs — Deferral-Clause Revisions and --no-collapse Dual-Scope (4 BCs revised)

| BC | Change | Before | After |
|----|--------|--------|-------|
| BC-2.11.013 | Deferral clause removed — grouped-collapse now supported (BC-2.11.030–034 cover it); scope restricted to flat-mode grouping order | v1.13 | v1.14 |
| BC-2.11.025 | Scope clarified as flat-mode only; grouped-collapse handled by BC-2.11.030–034; --no-collapse dual-scope (suppresses collapse in both grouped and flat modes) | v1.10 | v1.11 |
| BC-2.11.026 | Scope clarified as flat-mode only; per-bucket (xN) suffix in grouped mode addressed by BC-2.11.031 | v1.11 | v1.12 |
| BC-2.11.028 | --no-collapse dual-scope: suppresses collapse in BOTH grouped and flat modes; grouped-collapse now supported (was previously deferred/excluded) | v1.7 | v1.8 |

#### Vocab-Swept BCs — enum→struct{grouping:Grouping, collapse:Collapse} (8 BCs swept)

Exhaustive consuming-surface sweep: all occurrences of the old 3-variant `FindingsRender` enum variant names (`FlatCollapsed`, `FlatExpanded`, `Grouped`) replaced with struct-field access (`FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` etc.). Zero old enum-variant refs remain in SS-11 BCs.

| BC | Before | After |
|----|--------|-------|
| BC-2.11.010 | v1.10 | v1.11 |
| BC-2.11.014 | v1.8 | v1.9 |
| BC-2.11.015 | v1.9 | v1.10 |
| BC-2.11.016 | v1.8 | v1.9 |
| BC-2.11.017 | v1.15 | v1.16 |
| BC-2.11.019 | v1.9 | v1.10 |
| BC-2.11.027 | v1.6 | v1.7 |
| BC-2.11.029 | v1.6 | v1.7 |

#### PRD-Delta

`.factory/phase-f2-spec-evolution/story-119-prd-delta.md` written. Documents FindingsRender struct-of-orthogonal-enums type design decision (D-110), grouped-collapse CLI/UX mapping (--mitre now collapses by default), and the 5 new grouped-collapse BCs.

#### BC-INDEX

BC-INDEX.md updated: v1.43 → v1.44; SS-11 count 29→34; 5 new rows added (BC-2.11.030–034); 12 existing BC annotations bumped; total BC count 288→293.

**total_bcs=293 (5 new BCs 030–034; 12 revised/swept; no retirements).**

---

## [f3-adv-round6-bc-029-sibling-sweep-2026-06-18] — 2026-06-18

### PATCH: F3 Adversarial Round-6 — BC-2.11.029 Sibling-Sweep Completion (Issue #62)

**Trigger:** F3 round-4 fixed BC-2.11.028's wiring expression scope defect but did NOT sweep sibling BC-2.11.029, which carried the identical defect. Round-6 completes the sibling sweep.

#### Finding 1 (MEDIUM) — BC-2.11.029 Architecture Anchor Wiring Expression Used Out-of-Scope Variable Names

The Architecture Anchor `src/main.rs:~373` REFACTOR TARGET bullet in BC-2.11.029 prescribed the wiring expression as `render: if *mitre { FindingsRender::Grouped } else if !no_collapse { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }`. The `*mitre` and `no_collapse` (negated) names are `Commands::Analyze` destructured fields scoped to `main()` only (src/main.rs:55-56) — they are NOT in scope inside `run_analyze` at the TerminalReporter construction site (~main.rs:373). Inside `run_analyze`, the already-resolved bool params are `show_mitre_grouping` (line 107) and `collapse_findings` (line 108). The v1.4 changelog entry claimed "aligned to BC-2.11.028 sibling treatment" but copied the expression from before BC-2.11.028's own round-4 correction, so the defect persisted.

Corrected to: `render: if show_mitre_grouping { FindingsRender::Grouped } else if collapse_findings { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }`, with an explicit note that the `--mitre`/`--no-collapse`→bool resolution happens at the `main()` call site (lines 79-80, UNCHANGED): `show_mitre_grouping == *mitre` and `collapse_findings == !no_collapse` (via `collapse_findings_from_flag`). Scope/naming correction only; no behavioral change.

**Exhaustive sibling grep result** (`grep -rn '\*mitre\|no_collapse' .factory/specs/behavioral-contracts/ss-11/`):

| File | Line | Hit text | Classification |
|------|------|----------|----------------|
| BC-2.11.028.md | 58 | `show_mitre_grouping == *mitre` and `collapse_findings == !no_collapse` | SAFE — prose describing main() call-site resolution (not a normative wiring expression) |
| BC-2.11.028.md | 81 | `no_collapse` field on `Commands::Analyze` is a boolean | SAFE — prose describing CLI field definition |
| BC-2.11.028.md | 83-84 | `show_mitre_grouping == *mitre` and `collapse_findings == !no_collapse` | SAFE — Invariant 6 prose describing main() resolution (UNCHANGED) |
| BC-2.11.028.md | 93 | `summary` subcommand has no `no_collapse` | SAFE — EC-008 prose about absent field on summary subcommand |
| BC-2.11.028.md | 98 | `no_collapse` field in `cli.rs` MUST be wired | SAFE — prose referencing the CLI field name by name |
| BC-2.11.028.md | 101 | `*mitre`/`no_collapse` names from `Commands::Analyze` are resolved at | SAFE — Invariant 6 prose explaining the scope boundary (UNCHANGED) |
| BC-2.11.028.md | 116 | EC-008 summary subcommand (no `--no-collapse` field) | SAFE — edge case prose about absent flag |
| BC-2.11.028.md | 133 | `render = FindingsRender::FlatExpanded` (--no-collapse) | SAFE — VP description of observable behavior |
| BC-2.11.028.md | 136 | `no_collapse=true` → `render = FindingsRender::FlatExpanded` | SAFE — VP description at the CLI/flag level (not the run_analyze wiring expression) |
| BC-2.11.028.md | 159 | `same pattern as `no_collapse`` | SAFE — arch anchor prose comparing CLI field patterns |
| BC-2.11.028.md | 160 | `*mitre` → `show_mitre_grouping`; `collapse_findings_from_flag(*no_collapse)` → `collapse_findings` | SAFE — arch anchor main() call-site resolution note (UNCHANGED; this IS at main() scope) |
| BC-2.11.029.md | 123 | `--no-collapse flag` | SAFE — VP description of the CLI flag name |
| BC-2.11.029.md | 150 | `render: if *mitre { ... } else if !no_collapse { ... }` | **DEFECT — FIXED** (this entry; corrected to `show_mitre_grouping`/`collapse_findings`) |

Zero remaining defect expressions after this fix.

#### BC Version Summary

| BC/Doc | Before | After | Change |
|--------|--------|-------|--------|
| BC-2.11.029 | v1.4 | v1.5 | Architecture Anchor main.rs:~373 wiring expression: `*mitre`/`!no_collapse` → `show_mitre_grouping`/`collapse_findings` (in-scope run_analyze params); main() call-site resolution note added |
| BC-INDEX.md | — | — | BC-2.11.029 annotation bumped with v1.5 sibling-sweep note |

**total_bcs=288 unchanged** (scope/naming correction only; no new BCs, no retirements).

---

## [f3-adv-round4-bc-scope-anchor-fixes-2026-06-18] — 2026-06-18

### PATCH: F3 Adversarial Round-4 — Two BC Correctness Fixes (Issue #62)

**Trigger:** F3 adversarial round-4 surfaced two MEDIUM findings against STORY-120's anchored BC set.

#### Finding 1 (MEDIUM) — BC-2.11.028 Wiring Expression Used Out-of-Scope Variable Names

PC3, Invariant 1, Invariant 6, and the Architecture Anchor prescribed the `run_analyze` wiring
as `if *mitre { FindingsRender::Grouped } else if !no_collapse { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }`. But `*mitre` and `no_collapse` are `Commands::Analyze` destructured fields scoped to `main()` only (src/main.rs:55-56). Inside `run_analyze`, the resolved bool params are `show_mitre_grouping` (line 107) and `collapse_findings` (line 108). The out-of-scope names would not compile at the construction site (~main.rs:373). Corrected to: `if show_mitre_grouping { FindingsRender::Grouped } else if collapse_findings { FindingsRender::FlatCollapsed } else { FindingsRender::FlatExpanded }`. Added explicit note that `--mitre`/`--no-collapse`→bool resolution happens at `main()` lines 79-80 (UNCHANGED); `run_analyze` signature is UNCHANGED. Behavior identical — scope/naming correction only.

**Sibling BC-2.11.029:** Round-4 fixed BC-2.11.028 but did NOT sweep its sibling BC-2.11.029, which carried the identical defect in its Architecture Anchor main.rs:~373 bullet. The sibling-sweep was completed in F3 round-6 (see section `[f3-adv-round6-bc-029-sibling-sweep-2026-06-18]` below).

#### Finding 2 (MEDIUM) — Stale FINDINGS Dispatch Anchor `terminal.rs:149-162` Points at HOSTS Section

BC-2.11.025 Architecture Anchor, BC-2.11.026 Architecture Anchor, and BC-2.11.019 Invariant 7 cited the FINDINGS dispatch block as `src/reporter/terminal.rs:149-162`. Verified against source: line 149 is `if self.show_hosts_breakdown {` (HOSTS section). The actual FINDINGS dispatch if-chain (`if !findings.is_empty()`) begins at line 185 and ends at line 207. All three BCs re-anchored to `185-207`. DF-SIBLING-SWEEP-001 sweep confirmed these three are the only `149-162` occurrences in the 29 SS-11 BCs. Stories sweep (`grep -rn '149-162' .factory/stories/`) found hits in STORY-118 only; STORY-118 is frozen (pre-#62 implementing story) — not modified per scope constraint.

#### BC Version Summary

| BC/Doc | Before | After | Change |
|--------|--------|-------|--------|
| BC-2.11.028 | v1.5 | v1.6 | PC3 + Invariant 1 + Invariant 6 + Architecture Anchor: wiring expression variable names corrected to `show_mitre_grouping`/`collapse_findings` (in-scope params) |
| BC-2.11.019 | v1.7 | v1.8 | Invariant 7 dispatch anchor: terminal.rs:149-162 → 185-207 |
| BC-2.11.025 | v1.8 | v1.9 | Architecture Anchor dispatch range: terminal.rs:149-162 → 185-207 |
| BC-2.11.026 | v1.9 | v1.10 | Architecture Anchor dispatch range: terminal.rs:149-162 → 185-207 |
| BC-INDEX.md | — | — | Annotations updated for all 4 BCs |

**total_bcs=288 unchanged** (correctness/anchor fixes only; no new BCs, no retirements).

---

## [issue-62-enum-modes-bc-reanchor-2026-06-17] — 2026-06-17

### PATCH: Issue #62 BC Re-anchoring — FindingsRender Enum Precondition Update

**Trigger:** Issue #62 F2 spec evolution (BC re-anchoring pass). The approved design replaces
the three render bools (`show_mitre_grouping: bool`, `collapse_findings: bool`, plus implicit
flat-expanded default) with `pub enum FindingsRender { Grouped, FlatCollapsed, FlatExpanded }`
and reduces `TerminalReporter` to two orthogonal bools + one enum field. This is a
behavior-preserving type-system refactor — illegal-state elimination: the previous struct
permitted `show_mitre_grouping = true && collapse_findings = true`, a combination silently
handled by dispatch order but never valid. `FindingsRender::Grouped` makes that combination
structurally unrepresentable.

**Migration map (verified against `src/reporter/terminal.rs:187-197` dispatch):**
- `show_mitre_grouping = true` (any `collapse_findings`) → `render = FindingsRender::Grouped`
- `show_mitre_grouping = false, collapse_findings = true` → `render = FindingsRender::FlatCollapsed`
- `show_mitre_grouping = false, collapse_findings = false` → `render = FindingsRender::FlatExpanded`
- `run_summary` construction site → `render: FindingsRender::FlatCollapsed` by convention (inert — `run_summary` emits no FINDINGS section; no BC governs this path; the value is a structural placeholder distinguishing it from the dispatch-derived `run_analyze` mapping above)

**Scope:** Precondition field-name re-anchoring only. No new postconditions, no new test
vectors, no new invariants. No behavioral change. No new BCs. Architecture: ADR-0003
amended (new "Render-Mode Enum (Issue #62 — v0.9.0)" subsection + Binding Rule 5);
ARCH-INDEX SS-11/ADR-0003 row updated. See PRD-delta §Architecture Delta.

> **Bookkeeping correction (2026-06-18):** The original Scope line read "Architecture
> unchanged." — this was FALSE. ADR-0003 was amended during the human gate review (after
> the initial F2 pass) with a new subsection and Binding Rule 5. The Version Summary table
> below has been extended to include ADR-0003 and ARCH-INDEX as they were omitted in the
> original entry. No BC postconditions or test vectors changed.

#### BC Version Summary

| BC/Doc | Before | After | Change |
|--------|--------|-------|--------|
| BC-2.11.013 | v1.11 | v1.12 | Description + Precondition 1 + Invariant 4 + EC-007 |
| BC-2.11.014 | v1.6  | v1.7  | Precondition 1 only |
| BC-2.11.017 | v1.13 | v1.14 | Description + Precondition 1 + Postcondition 6 + Invariants 1/5 + EC-004/007/008 + test vectors |
| BC-2.11.019 | v1.6  | v1.7  | Postcondition 9 + Invariant 7 + EC-008/EC-009 |
| BC-2.11.025 | v1.6  | v1.7  | Description + Preconditions 1-2 + Invariant 5 + EC-011 |
| BC-2.11.026 | v1.8  | v1.9  | Preconditions 1-2 + EC-006/EC-007/EC-009 |
| BC-2.11.027 | v1.4  | v1.5  | Preconditions 1-2 + EC-008 |
| BC-2.11.028 | v1.4  | v1.5  | Description + Preconditions + Postconditions + Invariants 1-2 + EC-001..005 + Architecture Anchors |
| BC-INDEX.md | v1.38 | v1.39 | Comment annotations updated for all 8 BCs |
| BC-2.11.010 | v1.8  | v1.9  | Invariant 4 + EC-006 + EC-007 (fix-burst: missed in pass-1) |
| BC-2.11.015 | v1.7  | v1.8  | Precondition 1 (fix-burst: missed in pass-1) |
| BC-2.11.016 | v1.6  | v1.7  | Precondition 1 (fix-burst: missed in pass-1) |
| BC-2.11.029 | v1.2  | v1.3  | Precondition 4 + PC-1 inline qualifier + Architecture Anchors (fix-burst: missed in pass-1) |
| BC-INDEX.md | v1.39 | v1.40 | Comment annotations updated for 4 fix-burst BCs |
| BC-2.11.025 | v1.7  | v1.8  | VP-table row: 'show_mitre_grouping=true suppresses collapse' → 'render = FindingsRender::Grouped suppresses collapse' (F2 adv-pass-2 fix F-6) |
| BC-INDEX.md | v1.40 | v1.41 | BC-2.11.025 annotation updated with v1.8 note |
| BC-2.11.029 | v1.3  | v1.4  | Architecture Anchors block corrected (F-1, adv-pass-2): INSERTION TARGET/STORY-118 → REFACTOR TARGET/STORY-120; stale three-field parenthetical corrected to four-field v0.8.0 struct (use_color, show_mitre_grouping, show_hosts_breakdown, collapse_findings); line ranges aligned to BC-2.11.028 sibling (terminal.rs:91-110, main.rs ~373). The v1.3 changelog falsely claimed anchors had been updated; v1.4 is the genuine anchor-block correction. |
| BC-INDEX.md | v1.41 | v1.42 | BC-2.11.029 annotation updated with v1.4 note |
| ADR-0003 | — | amended | "Render-Mode Enum (Issue #62 — v0.9.0)" subsection added; Binding Rule 5 (render-mode type) added; illegal-state elimination rationale, migration map, semver v0.9.0 consequence, Default omission decision documented. (human gate fix-burst 2026-06-17) |
| ARCH-INDEX.md | v1.5 | v1.5* | ADR-0003 table row extended: v0.9.0 render-mode enum subsection appended to description; `modified[]` entry added for issue #62. (*version field unchanged; modified[] records the extension.) |

**total_bcs=288 unchanged** (re-anchoring pass; no new BCs, no retirements).

---

## [issue-259-collapse-bc025-evidence-fix-2026-06-17] — 2026-06-17

### PATCH: Issue #259 BC-2.11.025 Evidence Format Accuracy Fix (F-F3-001)

**Trigger:** BC-2.11.025 canonical flood vector specified evidence as `["GET /a HTTP/1.1"]` but `http.rs:365` emits `format!("{} {}", parsed.method, parsed.uri)` → `"GET /a"` (method + URI only, no HTTP version token). Self-contradiction with "mirroring http.rs" annotation. Partial fix from F-O-02 missed BC-2.11.025. total_bcs=288 unchanged.

#### Finding Dispositions

| Finding | Severity | File(s) Changed | Resolution |
|---------|----------|----------------|-----------|
| F-F3-001 | MEDIUM | BC-2.11.025 v1.5→v1.6 | Flood canonical test vector evidence: `["GET /a HTTP/1.1"]`…`["GET /e HTTP/1.1"]` → `["GET /a"]`…`["GET /e"]`. Added annotation: `format!("{} {}", parsed.method, parsed.uri)` → method+URI only, NO HTTP version token. |

#### BC Version Summary

| BC/Doc | Before | After |
|--------|--------|-------|
| BC-2.11.025 | v1.5 | v1.6 |
| BC-INDEX.md | v1.37 | v1.38 |

#### HTTP Version String Sweep — All 29 ss-11 BCs

| File | Line | Hit | Verdict |
|------|------|-----|---------|
| BC-2.11.025:155 | `["GET /a HTTP/1.1"]`…`["GET /e HTTP/1.1"]` | EVIDENCE-RESIDUE — **FIXED** → `["GET /a"]`…`["GET /e"]` |
| BC-2.11.019:117 | `DNS/HTTP/Modbus/SSH=5 each` | PROSE/WIRE — SAFE (service name in count table) |
| All other BC-2.11.001..029 | (no hits) | SAFE |

No evidence-residue remains. All ss-11 BC evidence examples now use `"{method} {uri}"` format.

---

## [issue-259-collapse-escape-notation-fix-2026-06-17] — 2026-06-17

### PATCH: Issue #259 BC-2.11.027 Escape Notation Accuracy Fix

**Trigger:** BC-2.11.027 EC-007 and canonical test vector used `\x1b` for the escaped ESC byte output. Ground truth from `terminal.rs` `escapes_esc_byte` test: `escape_for_terminal("\x1b[31mRED\x1b[0m") == "\u{1b}[31mRED\u{1b}[0m"` — `char::escape_default` renders ESC byte as `\u{1b}`, not `\x1b`. Root of STORY-118 AC-028 defect. total_bcs=288 unchanged.

#### Finding Dispositions

| Finding | Severity | File(s) Changed | Resolution |
|---------|----------|----------------|-----------|
| Escape notation accuracy | BC accuracy | BC-2.11.027 v1.3→v1.4 | EC-007: added "ESC byte `0x1b` renders as `\u{1b}` via `char::escape_default`; full line `> \u{1b}[31m` — NOT `> \x1b[31m`". Canonical test vector: `> \\x1b[31m` → `> \u{1b}[31m`. |

#### BC Version Summary

| BC/Doc | Before | After |
|--------|--------|-------|
| BC-2.11.027 | v1.3 | v1.4 |
| BC-INDEX.md | v1.36 | v1.37 |

#### Escape Notation Sibling Sweep

| File | Line | Hit | Verdict |
|------|------|-----|---------|
| BC-2.11.027:128 | `> \\x1b[31m` (escaped output form) | FIXED → `> \u{1b}[31m` |
| BC-2.11.027:116 | EC-007 (no explicit output form) | FIXED → added `\u{1b}` output form |
| BC-2.11.010:92-94 | `"\x1bESC"`, `["\x1b", "\x07"]` | SAFE — raw INPUT literals (before escaping) |
| BC-2.11.025:79,149,161 | `summary="x\x1b"` | SAFE — raw collapse key input value |
| ADR-0003:17,152,190 | `\x1b[31m...` | SAFE — attack vector / input byte sequences |
| ADR-0003:75,197 | `\u{1b}` | SAFE — already correct output form |

Verified against `terminal.rs:481-484` (`escapes_esc_byte` test).

---

## [issue-259-collapse-advpass12-14-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Passes 12-14 Remediation — 3 MEDIUM resolved

**Trigger:** 3 parallel passes each found 1 distinct MEDIUM. All remediated in one burst.
total_bcs=288 unchanged. SS-11=29 unchanged.

#### Finding Dispositions

| Finding | Severity | File(s) Changed | Resolution |
|---------|----------|----------------|-----------|
| F-PA-A01 | MEDIUM | BC-2.11.026 v1.7→v1.8; BC-2.11.017 v1.12→v1.13; BC-2.11.025 v1.4→v1.5 | Defined "representative finding" as `group_members[0]` (first in emission order) for all N≥1. Added normative PC-7 to BC-2.11.026 and updated BC-2.11.017 PC-6 + EC-007 to cite group_members[0]. BC-2.11.025 Invariant 6 generalized from N=1 singleton to all N≥1. Canonical test vector added to both BC-2.11.026 and BC-2.11.017: member[0].mitre=["T1036"], member[1].mitre=[], member[2].mitre=["T1059"] → MITRE line reads "MITRE: T1036"; others elided from terminal but preserved in JSON/CSV. |
| F-PB-01 | MEDIUM | BC-2.11.028 v1.3→v1.4 | Dropped "--no-color/--no-reassemble convention" citation (those are global flags on Cli; no_collapse is subcommand-scoped). Replaced with correct precedent: `#[arg(long)] mitre: bool` / `dns: bool` on `Commands::Analyze` (cli.rs:150-152), destructured as args.no_collapse. Fixed stale Architecture Anchor: cli.rs:151-153 no_reassemble → cli.rs:150-152 mitre: bool. Precondition 3 (args.no_collapse) and Invariant 4 (summary has no no_collapse field) confirmed consistent with subcommand-scoped model. |
| F-C01 | MEDIUM | verification-coverage-matrix.md v1.11→v1.12 | Current-state coverage note at matrix:155 cited "BC-2.11.010 v1.7"; live BC-2.11.010 is v1.8. Updated to v1.8. Frozen dated changelog reason fields (matrix:54) left unchanged. |

#### BC Version Summary

| BC/Doc | Before | After |
|--------|--------|-------|
| BC-2.11.017 | v1.12 | v1.13 |
| BC-2.11.025 | v1.4 | v1.5 |
| BC-2.11.026 | v1.7 | v1.8 |
| BC-2.11.028 | v1.3 | v1.4 |
| verification-coverage-matrix.md | v1.11 | v1.12 |
| BC-INDEX.md | v1.35 | v1.36 |
| prd.md | v1.27 | v1.28 |

#### Sibling Sweep Results

**Sweep 1 — "representative":** BC-2.11.025 Invariant 6, BC-2.11.026 PC-7, BC-2.11.017 PC-6/EC-007 now all consistently define/cite group_members[0] for N≥2; no remaining undefined use of "representative" for multi-member groups.

**Sweep 2 — no_reassemble/no_color/global/Commands::Analyze:** Stale "no_color/no_reassemble convention" and cli.rs:151-153 anchor removed from BC-2.11.028. Replaced with correct cli.rs:150-152 mitre: bool subcommand-scoped precedent. ADR-0003 line 294 already uses "Commands::Analyze" correctly; no change needed there.

**Sweep 3 — current-state BC version citations:** verification-coverage-matrix.md:155 updated BC-2.11.010 v1.7→v1.8. No other current-state BC version stamps found to lag. Verified: .010 v1.8 ✓, .013 v1.11 ✓, .017 v1.13 ✓, .025 v1.5 ✓, .026 v1.8 ✓, .027 v1.3 ✓, .028 v1.4 ✓, .029 v1.2 ✓, .019 v1.6 ✓.

---

## [issue-259-collapse-advpass9-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Pass-9 Remediation — 1 MEDIUM + 2 LOW resolved

**Trigger:** Passes 10 and 11 came back clean; pass 9 found 1 MEDIUM (implicit colorization requirement)
and 2 LOW (timestamp claim softening; default-output edge case). All remediated in one burst.
total_bcs=288 unchanged. SS-11=29 unchanged.

#### Finding Dispositions

| Finding | Severity | File(s) Changed | Resolution |
|---------|----------|----------------|-----------|
| F-PA-01 | MEDIUM | BC-2.11.026 v1.6→v1.7; BC-2.11.017 v1.11→v1.12 | Added normative PC-6 to BC-2.11.026 making the color-ladder requirement explicit: the collapse header path MUST apply terminal.rs:209-221 color-selection logic (Likely+High→red().bold(), Likely+other→yellow, Possible→yellow, Inconclusive→cyan, Unlikely→dimmed) to the pre-color string AFTER the ` (xN)` suffix has been appended. Appending the suffix after the ANSI reset is NON-CONFORMANT. BC-2.11.017 Invariant 5 cross-references BC-2.11.026 PC-6 for the full ladder. |
| F-PA-02 | LOW | BC-2.11.025 v1.3→v1.4 | Softened flood canonical test vector timestamp claim from "DIFFERING per-request timestamps" (implying they always differ) to "timestamps MAY differ across requests/flows (timestamp is a NON-KEY field; collapse is invariant to it regardless)." The real http.rs:368 code uses flow's `last_ts`; same-flow findings can share timestamp. |
| F-PA-03 | LOW | BC-2.11.028 v1.2→v1.3 | Added EC-010: "--no-collapse absent, default --output (terminal) → collapse applies (default-on), TerminalReporter.collapse_findings=true." Explicit EC covering the most common invocation path was missing. |

#### BC Version Summary

| BC/Doc | Before | After |
|--------|--------|-------|
| BC-2.11.017 | v1.11 | v1.12 |
| BC-2.11.025 | v1.3 | v1.4 |
| BC-2.11.026 | v1.6 | v1.7 |
| BC-2.11.028 | v1.2 | v1.3 |
| BC-INDEX.md | v1.34 | v1.35 |
| prd.md | v1.26 | v1.27 |

---

## [issue-259-collapse-advpass6-8-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Passes 6-8 Remediation — 1 MEDIUM + 2 LOW resolved

**Trigger:** Passes 6 and 7 came back clean; pass 8 found 1 MEDIUM (stale count) missed by
prior passes. Two cheap LOW hardening items also applied. All remediated in one burst.
total_bcs=288 unchanged. SS-11=29 unchanged.

#### Finding Dispositions

| Finding | Severity | File(s) Changed | Resolution |
|---------|----------|----------------|-----------|
| F1 (pass 8) | MEDIUM | BC-INDEX.md v1.33→v1.34 | Line 17 "verified on disk for all 283 entries" → "verified on disk for all 288 entries (283 prior + 5 new BC-2.11.025–029 for issue #259)". The line 34 current-state block already said 288; line 17 was the stale current-total claim. All other 283 references in BC-INDEX and prd.md are historical ARP-era delta rows that correctly record the intermediate milestone count and remain unchanged. |
| LOW-1 (pass 7 O-1) | LOW | BC-2.11.026 v1.5→v1.6 | Added canonical test vector exercising the `Likely/High → red().bold()` color branch: "2 findings (Reconnaissance, Likely, High, 'Port scan'), use_color=true → header + ` (x2)` suffix both wrapped in the red-bold span." EC-008 previously only exercised INCONCLUSIVE (cyan) branch. |
| LOW-2 (pass 7 O-2) | LOW | ADR-0003 | Example header line at ADR line 262 had no leading indent. Added two leading spaces (`  [Anomaly] INCONCLUSIVE...`) with annotation "(two leading spaces per `out.push_str("  {colored}\n")` in BC-2.11.026 PC-1/PC-2)" to match the binding contract. |

#### BC Version Summary

| BC/Doc | Before | After |
|--------|--------|-------|
| BC-2.11.026 | v1.5 | v1.6 |
| BC-INDEX.md | v1.33 | v1.34 |

#### "283" Sibling Sweep Result (BC-INDEX.md + prd.md)

| Location | Text | Verdict |
|----------|------|---------|
| BC-INDEX.md:17 | "verified on disk for all 283 entries" | FIXED → 288 |
| BC-INDEX.md:31 | "total to 283 active L3 BCs" | SAFE — ARP-era derivation step (intermediate count) |
| BC-INDEX.md:34 | "288 BCs (...283 prior + 5 new...)" | SAFE — correct current total |
| BC-INDEX.md:142 | "attempts-remove :147→:283" | SAFE — Rust line number, not a BC count |
| BC-INDEX.md:480 | "= 283 active BCs; + 5 Feature Mode... = 288" | SAFE — canonical derivation intermediate |
| prd.md:126 | "Total BC count: 283 (was 268)" | SAFE — ARP F2 delta row |
| prd.md:195 | "Total BC count: 283 (unchanged)" | SAFE — ARP pass-1 delta row |
| prd.md:232 | "Total BC count: 283 (unchanged)" | SAFE — ARP pass-2 delta row |
| prd.md:243 | "Total BC count: 283 (unchanged)" | SAFE — ARP pass-4 delta row |
| prd.md:257 | "283-total derivation" | SAFE — ARP-era derivation reference |
| prd.md:259 | "total remains 283" | SAFE — ARP pass-11 delta row |
| prd.md:388 | "Total BC count: 288 (was 283)" | SAFE — correct current total |
| prd.md:411 | "no BC count change (283)" | SAFE — pre-#259 ARP-era note |

Confirmation: the ONLY current-state total now reads 288 everywhere. All 283 residuals are historical ARP-era delta rows legitimately recording the intermediate count.

#### Files Changed

- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.33 → v1.34; line 17 count 283→288)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md` (v1.5 → v1.6; red-bold test vector added)
- `docs/adr/0003-reporting-pipeline-layering.md` (example header line prefixed with two leading spaces + annotation)
- `.factory/specs/prd.md` (delta pair BC-2.11.026 updated to v1.0→v1.6)

---

## [issue-259-collapse-advpass5-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Pass-5 Remediation — 1 MEDIUM + 1 LOW resolved

**Trigger:** F2 adversarial pass-5 found 1 MEDIUM + 1 LOW (last loose threads from pass-4
observable-behavior refactor). Corpus near-converged. All remediated in one burst.
total_bcs=288 unchanged. SS-11=29 unchanged.

#### Finding Dispositions

| Finding | Severity | BC(s) Changed | Resolution |
|---------|----------|--------------|-----------|
| F1 | MEDIUM | BC-2.11.010 v1.7→v1.8, BC-2.11.026 v1.4→v1.5, ADR-0003 | Residual "path-(b)" label references removed from 3 normative-body locations: (1) BC-2.11.010 Invariant 4: "(BC-2.11.026 path-(b))" parenthetical deleted — sentence now reads "The flat collapse wrapper calls `escape_for_terminal` on each sampled evidence line directly (per BC-2.11.026 PC-4 observable line-order contract)"; (2) BC-2.11.026 EC-009: "enforced by path-(b) separation" → "enforced by the grouped path being structurally suffix-free (it never appends a count suffix)"; (3) ADR-0003 line 276: "The collapse wrapper (path-(b), BC-2.11.026 PC-4)" → "The collapse path (per BC-2.11.026 PC-4 observable line-order contract)". |
| F2 | LOW | verification-coverage-matrix.md v1.10→v1.11 | `modified:` reason field for v1.8→v1.9 entry said "reporter/terminal.rs row unit count grows 1→6" — misstated the formal VP row count (VP row stays 2: VP-012 + VP-016; the new tests are test-sufficient unit tests, not formal VPs). Corrected to: "reporter/terminal.rs gains ~5 collapse UNIT TESTS (test-sufficient, not new formal VPs); VP-row total unchanged at 2 (VP-012 + VP-016); total VPs unchanged at 24." |

#### BC Version Summary

| BC/Doc | Before | After |
|--------|--------|-------|
| BC-2.11.010 | v1.7 | v1.8 |
| BC-2.11.026 | v1.4 | v1.5 |
| verification-coverage-matrix.md | v1.10 | v1.11 |

#### Sibling Sweep Result (path-(a)/path-(b) — COMPLETE)

Full grep across `.factory/specs/` and `docs/adr/0003*` for:
`path-(b)`, `path (b)`, `path-b`, `path-(a)`, `path (a)`, `path-a`

| Location | Hit | Verdict |
|----------|-----|---------|
| `prd.md:381` | "adv-pass-2 path-(b) EC-007" | SAFE — `modified:` history delta note |
| `prd.md:382` | "adv-pass-2 path-(b))" | SAFE — `modified:` history delta note |
| `BC-INDEX.md:278` BC-2.11.013 comment | "structural suffix-free guarantee via path-(b)" | SAFE — HTML comment changelog annotation |
| `BC-INDEX.md:282` BC-2.11.017 comment | "Invariant 5 aligned to path-(b) wrapper" | SAFE — HTML comment changelog annotation |
| `BC-INDEX.md:291` BC-2.11.026 comment | "path-(b) wrapper canonical in PC-4" + "remove 'path-(b) function-call graph'" | SAFE — HTML comment changelog annotations |
| `BC-2.11.010.md:65` (Invariant 4) | "The flat collapse wrapper (BC-2.11.026 path-(b))" | FIXED → "The flat collapse wrapper calls...directly (per BC-2.11.026 PC-4 observable line-order contract)" |
| `BC-2.11.013.md:25` (`modified:` array) | "via path-(b) wrapper" | SAFE — `modified:` history entry |
| `BC-2.11.026.md:15` (`modified:` array) | multiple "path-(b)" refs | SAFE — `modified:` history entries |
| `BC-2.11.026.md:105` (EC-009 body) | "enforced by path-(b) separation" | FIXED → "enforced by the grouped path being structurally suffix-free" |
| `BC-2.11.017.md:25` (`modified:` array) | "path-(b) collapse-aware wrapper" | SAFE — `modified:` history entry |
| `docs/adr/0003:276` | "The collapse wrapper (path-(b), BC-2.11.026 PC-4)" | FIXED → "The collapse path (per BC-2.11.026 PC-4 observable line-order contract)" |

ZERO normative-body residuals remain after this burst.

#### Files Changed

- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md` (v1.7 → v1.8)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md` (v1.4 → v1.5)
- `.factory/specs/architecture/verification-coverage-matrix.md` (v1.10 → v1.11)
- `docs/adr/0003-reporting-pipeline-layering.md` (path-(b) label removed from escape-reuse paragraph)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.32 → v1.33; BC-010/026 row annotations updated)
- `.factory/specs/prd.md` (delta block: BC-2.11.010 pair v1.4→v1.8; BC-2.11.026 pair v1.0→v1.5)

---

## [issue-259-collapse-advpass4-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Pass-4 Remediation — 2 HIGH + 2 LOW resolved

**Trigger:** F2 adversarial pass-4 found 2 HIGH + 2 LOW findings (0 CRITICAL). All remediated
in one burst. META-FIX: pass-4 exposed that the spec was over-specifying INTERNAL CALL
STRUCTURE (render_finding_flat / render_finding_prefix call graph). All such claims converted
to observable-behavior contracts with non-normative implementation notes per adjudicated model.
total_bcs=288 unchanged. SS-11=29 unchanged.

#### Finding Dispositions

| Finding | Severity | BC(s) Changed | Resolution |
|---------|----------|--------------|-----------|
| F-F2-A01 | HIGH | BC-2.11.017 v1.10→v1.11, BC-2.11.026 v1.3→v1.4, BC-2.11.025 v1.2→v1.3, BC-2.11.013 v1.10→v1.11 | Conflicting normative internal-call-structure claims removed throughout. BC-2.11.017 Invariant 5 rewritten as OBSERVABLE LINE ORDER (header → K-sampled evidence → MITRE line); PC-6 added (MITRE line contract for collapse path); Description paragraph updated to match. BC-2.11.026 PC-4 rewritten as OBSERVABLE LINE ORDER + non-normative note: "F4 MAY reimplement... provided observable line order holds, suffix flat-only, evidence K-capped, every evidence line through escape_for_terminal." BC-2.11.026 PC-2 singleton ref updated to drop "render_finding_prefix" name; Invariant 4 render_finding_prefix implementation detail removed; EC-007 converted from STRUCTURAL GUARANTEE to OBSERVABLE GUARANTEE. BC-2.11.013 EC-007 converted from "STRUCTURAL guarantee: render_finding_prefix itself is UNCHANGED; the collapse-aware flat wrapper is never called from the grouped path" to OBSERVABLE GUARANTEE matching BC-2.11.026 EC-007. BC-2.11.025 Invariant 6 "byte-identical to calling render_finding_flat directly" converted to output-observable form. |
| F-F2-A02 | HIGH | verification-coverage-matrix.md v1.9→v1.10 | Stale "collapse path calls same render_finding_prefix code path" claim corrected to: "the escape_for_terminal FUNCTION invariant is unchanged — the collapse path calls escape_for_terminal directly on each sampled evidence line and does NOT delegate to render_finding_prefix's evidence loop (BC-2.11.010 v1.7 / BC-2.11.027 v1.3 / ADR-0003)." |
| F-F2-O01 | LOW | BC-2.11.025 v1.2→v1.3, BC-2.11.026 v1.3→v1.4, BC-2.11.010 v1.6→v1.7, BC-2.11.012 v1.5→v1.6 | anchor `terminal.rs:203-226` → `terminal.rs:203-227` in all four files (confirmed against source: fn opens at :203, closing brace at :227). Source Evidence path in BC-2.11.010 also updated. BC-2.11.012 discovered via sibling sweep. |
| F-F2-O02 | LOW | BC-2.11.025 v1.2→v1.3 | Canonical flood test vector: "same timestamp" → "DIFFERING per-request timestamps (non-key field, excluded from collapse key)" — mirrors actual empty-UA emission at http.rs:359-371 where each request has a distinct timestamp; expected output (1 group, count 5) unchanged; this demonstrates timestamp variance does NOT block collapse. |

#### BC Version Summary

| BC/Doc | Before | After |
|--------|--------|-------|
| BC-2.11.010 | v1.6 | v1.7 |
| BC-2.11.012 | v1.5 | v1.6 |
| BC-2.11.013 | v1.10 | v1.11 |
| BC-2.11.017 | v1.10 | v1.11 |
| BC-2.11.025 | v1.2 | v1.3 |
| BC-2.11.026 | v1.3 | v1.4 |
| verification-coverage-matrix.md | v1.9 | v1.10 |

#### ADR-0003 Check

No additional changes needed. ADR-0003 was already corrected in pass-3 for the escape-reuse prose. The call-structure claims in ADR-0003 are now consistent with the updated observable-behavior BCs.

#### Sibling Sweep Result (COMPLETE)

Full grep across `.factory/specs/` and `docs/adr/0003*` for:
`render_finding_flat`, `render_finding_prefix`, `call site`, `same code path`, `calls same`,
`is called once per`, `replaces the direct`, `via this wrapper`, `same call`

Normative call-structure claims eliminated:
- `BC-2.11.017:88-98` — Invariant 5 "render_finding_flat is called once per collapsed group via this wrapper" → REMOVED; replaced with observable line order + non-normative note
- `BC-2.11.026:64-78` (PC-4) — "path-(b) wrapper / path-(a) prohibited / CANONICAL implementation" call graph → REMOVED; replaced with observable line order + non-normative note
- `BC-2.11.026:93-94` (Invariant 4) — "render_finding_prefix implementation builds line atomically" → REMOVED
- `BC-2.11.026:108` (EC-007) — "STRUCTURAL guarantee: render_finding_prefix UNCHANGED; wrapper never called from grouped path" → CONVERTED to OBSERVABLE GUARANTEE
- `BC-2.11.026:153-154` (Arch Anchors) — "count-annotated render replaces the direct render_finding_flat call" → REMOVED; plain anchor only
- `BC-2.11.013:107` (EC-007) — "STRUCTURAL guarantee: render_finding_prefix itself is UNCHANGED; the collapse-aware flat wrapper is never called from the grouped path" → CONVERTED to OBSERVABLE GUARANTEE
- `BC-2.11.025:112` (Invariant 6) — "byte-identical to calling render_finding_flat directly" → CONVERTED to output-observable form (no function name)
- `BC-2.11.025:191-192` (Arch Anchors) — "called per collapsed group representative" removed from render_finding_prefix anchor comment
- `verification-coverage-matrix.md:146-147` — "collapse path calls same render_finding_prefix code path" → CORRECTED (F-F2-A02)

Remaining `render_finding_prefix` / `render_finding_flat` occurrences after the sweep are all:
(a) Architecture Anchor lines pointing to the function (not normative call-structure claims), OR
(b) Invariant/Postcondition descriptions of GROUPED MODE behavior (e.g., "called by render_finding_grouped" — true and must stay), OR
(c) Historical `modified:` frontmatter entries recording what was wrong/fixed.
None of these are normative claims about the collapse path's internal call structure.

`escape_for_terminal` guarantee confirmed PRESERVED in BC-2.11.010 Invariant 4, BC-2.11.027 PC-1/PC-6/Invariant-5, and ADR-0003.

#### Files Changed

- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md` (v1.6 → v1.7)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.012.md` (v1.5 → v1.6)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md` (v1.10 → v1.11)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md` (v1.10 → v1.11)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.025.md` (v1.2 → v1.3)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md` (v1.3 → v1.4)
- `.factory/specs/architecture/verification-coverage-matrix.md` (v1.9 → v1.10)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.31 → v1.32; BC-010/012/013/017/025/026 row annotations updated)
- `.factory/specs/prd.md` (delta block: all affected BC pairs updated to final adv-pass-4 versions)

---

## [issue-259-collapse-advpass3-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Pass-3 Remediation — 1 HIGH + 2 LOW resolved

**Trigger:** F2 adversarial pass-3 found 1 HIGH + 2 LOW findings (0 CRITICAL — all pass-1/pass-2
findings confirmed fixed). All remediated in one burst. No F1 locked decisions changed.
total_bcs=288 unchanged. SS-11=29 unchanged.

#### Finding Dispositions

| Finding | Severity | BC(s) Changed | Resolution |
|---------|----------|--------------|-----------|
| F-F2X-01 | HIGH | BC-2.11.010 v1.5→v1.6, BC-2.11.026 v1.2→v1.3, BC-2.11.027 v1.2→v1.3, ADR-0003 | False claim "same call site in render_finding_prefix / same code path" corrected throughout. Path-(b) flat collapse wrapper calls `escape_for_terminal` DIRECTLY on each sampled evidence line. It does NOT delegate to `render_finding_prefix`'s evidence loop (that loop renders all entries of ONE finding; collapse samples `evidence[0]` across up to K different member findings). Escape reuse is FUNCTION-level, not call-site-level. Every affected statement corrected: BC-2.11.010 Invariant 4 + EC-007; BC-2.11.026 PC-4 evidence emission sentence; BC-2.11.027 PC-1 + PC-6 + Invariant 5; ADR-0003 Display-Layer Aggregation subsection prose. |
| F-F2X-03 | LOW | BC-2.11.026 v1.2→v1.3 | PC-4 was silent on HOW the collapse wrapper emits evidence lines. Added explicit sentence: "After emitting the colorized header line, the same wrapper emits the K-sampled evidence lines per BC-2.11.027 DIRECTLY (calling `escape_for_terminal` on each sampled line), NOT via `render_finding_prefix`." |
| F-F2X-02 | LOW | BC-2.11.026 v1.2→v1.3 | EC-008 and EC-009 were listed out of monotonic order (EC-009 before EC-008). Rows swapped to restore EC-008, EC-009 ascending order. |

#### BC Version Summary

| BC | Before | After |
|----|--------|-------|
| BC-2.11.010 | v1.5 | v1.6 |
| BC-2.11.026 | v1.2 | v1.3 |
| BC-2.11.027 | v1.2 | v1.3 |

#### ADR-0003 Change

The Display-Layer Aggregation subsection previously stated: "The collapse path calls `render_finding_prefix` through the same code path as uncollapsed rendering; there is no bypass of the escape helper." This was FALSE. Corrected to: "The collapse wrapper (path-(b), BC-2.11.026 PC-4) calls `escape_for_terminal` DIRECTLY on each sampled evidence line — it does NOT delegate to `render_finding_prefix`'s evidence loop, because that loop renders all entries of a single finding whereas collapse samples `evidence[0]` across up to K member findings (BC-2.11.027 positional model). There is no bypass of the escape helper."

#### Sibling Sweep Result

Grepped all touched BC files and ADR-0003 for: "render_finding_prefix", "call site", "same code path", "same call", "identical to the existing", "per-finding evidence rendering". All remaining occurrences confirmed as correct structural descriptions (render_finding_prefix is called by render_finding_grouped — that is true and must stay). No residual false claims found.

#### Files Changed

- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md` (v1.5 → v1.6)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md` (v1.2 → v1.3)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md` (v1.2 → v1.3)
- `docs/adr/0003-reporting-pipeline-layering.md` (Display-Layer Aggregation subsection — escape reuse prose corrected)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.30 → v1.31; BC-2.11.010/026/027 row annotations updated)
- `.factory/specs/prd.md` (delta block updated: BC-2.11.010 pair v1.4→v1.5 → v1.4→v1.6; BC-2.11.026/027 pairs v1.0→v1.2 → v1.0→v1.3)

---

## [issue-259-collapse-advpass2-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Pass-2 Remediation — 4 MEDIUM + 3 LOW resolved

**Trigger:** F2 adversarial pass-2 found 4 MEDIUM + 3 LOW findings (no CRITICAL/HIGH — pass-1
CRITICAL confirmed fixed). All remediated in one burst. No F1 locked decisions changed.
total_bcs=288 unchanged. SS-11=29 unchanged.

#### Finding Dispositions

| Finding | Severity | BC(s) Changed | Resolution |
|---------|----------|--------------|-----------|
| F-A01 | MEDIUM | BC-2.11.025 v1.1→v1.2 | `Vec<(CollapseKey, Vec<&Finding>)>` accumulator is now CANONICAL in PC-9 and Invariant 7. `IndexMap` demoted to parenthetical: "only viable if Hash derived AND indexmap crate added — NOT done in v0.8.0." Key enums derive only PartialEq; Vec accumulator uses linear-scan PartialEq matching. |
| F-A02 | MEDIUM | prd.md v1.26 delta block | BC-2.11.017 pair corrected `v1.7→v1.8` → `v1.7→v1.10`; all greenfield BC pairs corrected to v1.2; BC-2.11.013 corrected to v1.10; BC-2.11.019 corrected to v1.6. Notes added for pass-1 and pass-2 bumps. |
| F-A03 | MEDIUM | BC-2.11.026 v1.1→v1.2, BC-2.11.017 v1.9→v1.10, BC-2.11.013 v1.9→v1.10 | Path-(b) CANONICAL: collapse-aware FLAT-ONLY wrapper; `render_finding_prefix` unchanged; grouped mode structurally suffix-free. Non-canonical path-(a) prohibited unless sentinel used. EC-007/EC-009 in BC-2.11.026 and EC-007 in BC-2.11.013 assert structural suffix-free guarantee. |
| F-A04 | MEDIUM | BC-2.11.025 v1.1→v1.2 | Primary flood test vector strengthened: specifies IDENTICAL 4-tuple, DISTINCT evidence URIs per finding (mirrors http.rs:359-371 empty-UA emission), cross-links to BC-2.11.027 for evidence sampling. |
| F-A05 | LOW | BC-2.11.025 v1.1→v1.2, BC-2.11.026 v1.1→v1.2, BC-2.11.019 v1.5→v1.6 | All three `terminal.rs:149-160` anchor citations updated to `terminal.rs:149-162`. |
| F-A06 | LOW | BC-2.11.026 v1.1→v1.2 | EC-005 canonical test vector added: summary=`"Empty UA "` (trailing space) + 2 findings → header ends with `"Empty UA  (x2)\n"` (double-space, no trimming). Decision pinned. |
| F-A07 | LOW | prd.md | Resolved by F-A02 (PRD per-feature delta block now in fix-burst propagation target set). No additional spec change. |

#### BC Version Summary

| BC | Before | After |
|----|--------|-------|
| BC-2.11.013 | v1.9 | v1.10 |
| BC-2.11.017 | v1.9 | v1.10 |
| BC-2.11.019 | v1.5 | v1.6 |
| BC-2.11.025 | v1.1 | v1.2 |
| BC-2.11.026 | v1.1 | v1.2 |

#### ADR-0003 Check

No changes needed. ADR-0003 has no IndexMap/grouping-structure prose (F-A01 is spec-only).
ADR-0003 does not describe render_finding_prefix implementation details (F-A03 is spec-only).

#### Files Changed

- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md` (v1.9 → v1.10)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md` (v1.9 → v1.10)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.019.md` (v1.5 → v1.6)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.025.md` (v1.1 → v1.2)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md` (v1.1 → v1.2)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.29 → v1.30; 5 BC row annotations updated)
- `.factory/specs/prd.md` (delta block updated: BC-2.11.017 pair and greenfield BC pairs corrected)
- `.factory/spec-changelog.md` (this entry)

---

## [issue-259-collapse-advpass1-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Adversarial Pass-1 Remediation — 9 findings resolved

**Trigger:** F2 adversarial pass-1 found 9 findings (1 CRITICAL, 4 HIGH, 4 MEDIUM). All
remediated in one burst. No F1 locked decisions changed. total_bcs=288 unchanged.

#### Finding Dispositions

| Finding | Severity | BC(s) Changed | Resolution |
|---------|----------|--------------|-----------|
| F-259-01 | CRITICAL | BC-2.11.027 v1.1→v1.2, ADR-0003 | Positional first-K-members model enforced everywhere. EC-004 corrected: N=5/member[0]-empty → 2 lines (not 3). PC-2, Invariant-2, PC-5 rewritten to state the model unambiguously. No sliding window. |
| F-259-02 | HIGH | BC-2.11.026 v1.0→v1.1, BC-2.11.017 v1.8→v1.9 | `(xN)` suffix IS colorized with header line. Updated Invariant 4, PC-4, EC-008 in BC-2.11.026; updated Invariant 5 in BC-2.11.017. Unimplementable "not colorized" language removed. |
| F-259-03 | HIGH | BC-2.11.025 v1.0→v1.1 | Added Invariant 6: singleton ORIGINAL `&Finding` is passed unchanged; collapse pass MUST NOT clone/reorder/modify singleton. N=1 render is byte-identical to `render_finding_flat(&original_finding)`. |
| F-259-04 | HIGH | BC-2.11.025 v1.0→v1.1 | Added PC-7: collapse is SEVERITY-AGNOSTIC — no "low-value" filter. Added EC-014 and test vector: two `Likely/High` identical findings → `(x2)`. |
| F-259-05 | LOW | BC-2.11.029 v1.1→v1.2 | CSV Architecture Anchors updated with precise line refs: csv.rs:40 (`neutralize_csv_injection`) and csv.rs:76 (findings render loop). Both confirmed in source. |
| F-259-06 | MEDIUM | BC-2.11.025 v1.0→v1.1 | Added PC-9: determinism conditional on input slice order; implementation MUST use insertion-ordered structure (not HashMap). Added as Invariant 7. |
| F-259-07 | MEDIUM | BC-2.11.027 v1.1→v1.2 | Added N=3 (N≤K boundary) and N=4 (N>K boundary) canonical test vectors. |
| F-259-08 | MEDIUM | BC-2.11.028 v1.1→v1.2, BC-2.11.029 v1.1→v1.2 | PC-3 in BC-2.11.028 changed from indicative to imperative. Architecture Anchors for `collapse_findings` field and main.rs wiring marked "INSERTION TARGET (code TBD by STORY-118)". `collapse_findings` confirmed absent from TerminalReporter struct. |
| F-259-09 | MEDIUM | BC-2.11.025 v1.0→v1.1 | Added PC-8: raw-byte key vs escaped-display divergence is intentional (forensic fidelity). Added EC-015 and test vector: `summary="x\x1b"` vs `summary="x"` → two distinct groups. |

#### ADR-0003 Prose Fix

Evidence sampling section updated: "first min(N, K) findings" clarified to state the
positional algorithm — empty-evidence member contributes 0 lines, window does NOT slide.
Example added: N=5, member[0] empty → 2 lines (not 3).

#### BC Version Summary

| BC | Before | After |
|----|--------|-------|
| BC-2.11.025 | v1.0 | v1.1 |
| BC-2.11.026 | v1.0 | v1.1 |
| BC-2.11.027 | v1.1 | v1.2 |
| BC-2.11.028 | v1.1 | v1.2 |
| BC-2.11.029 | v1.1 | v1.2 |
| BC-2.11.017 | v1.8 | v1.9 |

#### Files Changed

- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.025.md` (v1.0 → v1.1)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md` (v1.0 → v1.1)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md` (v1.1 → v1.2)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.028.md` (v1.1 → v1.2)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.029.md` (v1.1 → v1.2)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md` (v1.8 → v1.9)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.28 → v1.29; all 6 BC row annotations updated)
- `docs/adr/0003-reporting-pipeline-layering.md` (evidence sampling prose clarified)
- `.factory/spec-changelog.md` (this entry)

---

## [issue-259-collapse-remediation-2026-06-17] — 2026-06-17

### PATCH: Issue #259 F2 Consistency Audit Remediation — BC-2.11.027/028/029 v1.0→v1.1

**Trigger:** Fresh-context consistency audit on the #259 F2 delta found three specification
defects introduced during BC creation. All defects remediated in one burst per adjudicated
N=1/N≥2 model. **No design decisions changed.**

#### Adjudicated Model (authoritative, non-negotiable)

- **N=1 (singleton):** Renders BYTE-IDENTICALLY to current non-collapse behavior. The K=3 cap
  does NOT apply. Evidence renders exactly as existing `render_finding_prefix` (BC-2.11.010,
  unchanged). A singleton with 5 evidence lines shows all 5 lines, same as pre-v0.8.0.
- **N≥2 (collapsed group):** K=3 cap applies. Render shared header with `(xN)`, then at most
  K=3 representative evidence lines: `evidence[0]` from each of the first min(N,3) distinct
  member findings, in original emission order.

#### Defects Fixed

| BC | Finding | Fix |
|----|---------|-----|
| BC-2.11.027 | Invariant 6 said "up to K lines capped, but N=1 so at most 1 evidence line" — implies K-cap framework applies to singletons (WRONG) | Rewritten: K-cap NOT applied to singletons; evidence renders identically to pre-v0.8.0 `render_finding_prefix` |
| BC-2.11.027 | EC-001 said "Group with N=1 member, 5 evidence lines → At most K=3 evidence lines rendered" — explicitly wrong | Fixed: "All 5 evidence lines rendered, unchanged from pre-v0.8.0 behavior (K-cap does NOT apply to singletons)" |
| BC-2.11.027 | Last test vector "1 finding with evidence=[a,b,c,d] → at most K=3 rendered" — wrong per adjudicated model | Removed; replaced with correct N≥2 test vector demonstrating evidence[0]-from-first-3-members pattern |
| BC-2.11.029 | Postcondition 3: "Its evidence is rendered in full (not sampled, because N=1 ≤ K=3)" — misleading; implies K-cap was evaluated and happened not to truncate | Rewritten: "Its terminal rendering is identical to the pre-v0.8.0 output for that finding — the collapse feature does not alter the rendering of non-repeated findings" |
| BC-2.11.028 | Related BCs listed "BC-2.13.001 -- context (--verbose is absent/rejected; ...)" — BC-2.13.001 is `--threats` absent, NOT `--verbose`; `--verbose` absent is BC-2.13.004 | Fixed: BC-2.13.001 → BC-2.13.004 |

#### BC-INDEX.md Fix

- SS-11 section header: stale "24 BCs total; 24 fully written" → "29 BCs total; 29 fully written"
- BCs 025-029 description line added to header block
- BC-2.11.027/028/029 row annotations bumped to note v1.1 and audit fix

#### Sibling Sweep Results (DF-SIBLING-SWEEP-001)

All other touched BCs confirmed clean:
- BC-2.11.025/026: no evidence-sampling claims; CLEAN
- BC-2.11.010: Invariant 4 correctly scoped to "collapsed group"; CLEAN
- BC-2.11.013/017/019: no evidence-sampling claims; CLEAN
- BC-INDEX.md: SS-15 section header "24 BCs" is correct (DNP3, independent subsystem); not changed
- prd.md: CLEAN
- verification-coverage-matrix.md: CLEAN

#### Files Changed

- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md` (v1.0 → v1.1)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.028.md` (v1.0 → v1.1)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.029.md` (v1.0 → v1.1)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.27 → v1.28; SS-11 header 24→29 + row annotations)
- `.factory/spec-changelog.md` (this entry)

---

## [issue-259-collapse-f2-2026-06-17] — 2026-06-17

### MINOR: Feature #259 — Terminal Finding Collapse (v0.8.0) — F2 Spec Evolution Complete

**Feature:** Collapse repeated identical terminal findings into counted summary lines.
**Issue:** GitHub #259. **Release target:** v0.8.0. **Epic:** E-8 (Reporting and Output Formats).
**Stories:** STORY-118 (primary), STORY-119 (grouped-mode extension, deferred).

#### Locked Design Decisions (F1-gated)

| Decision | Choice | Governing BC |
|----------|--------|--------------|
| OQ-1: Default-on vs opt-in | DEFAULT-ON; `--no-collapse` opt-out | BC-2.11.028 |
| OQ-2: Threshold vs always-collapse | ALWAYS-COLLAPSE; N=1 singleton renders without suffix | BC-2.11.026 |
| OQ-3: Grouped-mode interaction | FLAT-MODE ONLY v0.8.0; grouped/`--mitre` bypasses collapse | BC-2.11.013 v1.9 Invariant 4 |
| OQ-4: Evidence sample count | K=3 hardcoded constant | BC-2.11.027 |

#### New BCs (5 greenfield, BC-2.11.025–029)

| BC ID | Title | Priority |
|-------|-------|---------|
| BC-2.11.025 | Flat-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key; First-Occurrence Order; Deterministic | P0 |
| BC-2.11.026 | Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix | P0 |
| BC-2.11.027 | Collapsed Group Retains at Most K=3 Representative Evidence Lines; Remainder Elided from Terminal Display | P1 |
| BC-2.11.028 | --no-collapse Opt-Out Flag Disables Terminal Collapse and Restores One-Line-Per-Finding Rendering; JSON/CSV Unaffected | P0 |
| BC-2.11.029 | Collapse is Display-Layer Only; JSON/CSV Reporters Receive Unmodified findings Slice; Non-Repeated Findings Individually Visible in All Outputs | P0 |

#### Extended BCs (4 sibling BCs bumped)

| BC ID | Old Version | New Version | Extension Summary |
|-------|------------|------------|-------------------|
| BC-2.11.010 | v1.4 | v1.5 | Invariant 4 + EC-006/EC-007: evidence sampling under collapse bounded to K=3; escape_for_terminal invariant unchanged |
| BC-2.11.013 | v1.8 | v1.9 | Invariant 4 + EC-007: show_mitre_grouping=true suppresses collapse pass; grouped-mode collapse deferred to STORY-119 |
| BC-2.11.017 | v1.7 | v1.8 | Description updated + Invariant 5 + EC-007/EC-008: render_finding_flat called per collapsed group; (xN) suffix on header line only |
| BC-2.11.019 | v1.4 | v1.5 | Postcondition 9 + Invariant 7 + EC-008/EC-009: flat FINDINGS dispatch routes through collapse pass; section ordering unchanged |

#### Verification

**No new formal VP.** Feature is test-sufficient per F1 §8:
- New unit tests required per BC-2.11.025–029 Verification Properties sections.
- VP-012 (`escape_for_terminal`, proptest, P1) unchanged.
- Integration test mandate: N identical findings → terminal 1 collapsed + JSON N objects (BC-2.11.029).

#### Architecture

ADR-0003 extended by architect (display-layer aggregation subsection). Not changed by this burst.
Aggregation key: `(category, verdict, confidence, summary)`. Evidence/mitre_techniques/source_ip/
timestamp/direction are NOT key fields. Collapse is private to TerminalReporter; JSON/CSV reporters
receive unmodified slice per ADR-0003 binding rule.

#### Files Changed

- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.025.md` (new)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.026.md` (new)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.027.md` (new)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.028.md` (new)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.029.md` (new)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md` (v1.4 → v1.5)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md` (v1.8 → v1.9)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md` (v1.7 → v1.8)
- `.factory/specs/behavioral-contracts/ss-11/BC-2.11.019.md` (v1.4 → v1.5)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (v1.26 → v1.27; 283→288 total)
- `.factory/specs/prd.md` (v1.25 → v1.26; §2.11 + delta note + 288 total)
- `.factory/specs/architecture/verification-coverage-matrix.md` (v1.8 → v1.9; issue-#259 coverage note)
- `.factory/spec-changelog.md` (this entry)

---

## [bc-2.14.017-v2.6-burst-summary-window-width-2026-06-17] — 2026-06-17

### PATCH: BC-2.14.017 v2.5→v2.6 — Burst summary string: elapsed span → configured window width (issue #220)

**Trigger:** GitHub issue #220 — Modbus write-burst "0s window" cosmetic display bug.
**Root cause:** When ≥(threshold+1) writes share the same integer-second pcap timestamp (single
pcap flush), `elapsed_secs = now_ts.wrapping_sub(window_start_ts) = 0`, producing the misleading
display "21 writes in 0s window". Detection math is correct; only the display string was wrong.
**D-043 lineage:** This is the display-string leg of the elapsed_ms→seconds correction chain
started in v2.2 (D-043). The elapsed span was always unreliable as a window-width proxy when
same-second writes occur; v2.6 severs the dependency by reporting the constant instead.

**Changes:**

- **BC-2.14.017 Postcondition 1 (Burst finding summary string):** Changed from
  `"Modbus write burst: {count} writes in {elapsed_secs}s window (unit {unit_id}, threshold {threshold}/s)"`
  to
  `"Modbus write burst: {count} writes within {window_secs}s window (unit {unit_id}, threshold {threshold}/s)"`
  where `{window_secs}` is the constant `WRITE_BURST_WINDOW_SECS` (= 1). The `{elapsed_secs}`
  variable is removed from the summary string entirely. Detection logic is unchanged.
- **BC-2.14.017 EC-011 (new):** Documents the same-second / elapsed==0 case explicitly — confirms
  the burst detection FIRES correctly on same-timestamp writes and the summary correctly reports
  the window width (1), not the elapsed span (0).
- **BC-2.14.017 Canonical test vector updated:** The `elapsed_secs=0` happy-path row now includes
  the expected summary string verbatim and cross-references EC-011.
- **SUSTAINED detector postcondition:** UNCHANGED — `"Modbus write burst: {count} writes over
  {elapsed_s}s window"` legitimately reports elapsed span and is not affected.
- **BC-INDEX.md:** BC-2.14.017 comment annotation updated to note v2.6 change.

**Files changed:**
- `.factory/specs/behavioral-contracts/ss-14/BC-2.14.017.md` (v2.5 → v2.6)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` (BC-2.14.017 annotation)
- `.factory/spec-changelog.md` (this entry)

---

## [e17-f3-story-backlink-update-2026-06-17] — 2026-06-17

### PATCH: BC-2.16.009 v1.9→v1.10 + BC-2.16.015 v1.8→v1.9 — E-17 F3 story-backlink update (BC Backlink Update Obligation)

**Feature cycle:** E-17 "ARP Decoder VLAN/QinQ/MACsec Offset Hardening" — F3 traceability completion (E-17 issue #253).
**Trigger:** STORY-116 and STORY-117 (E-17 epic) were created in the preceding burst and consume
BC-2.16.009 and BC-2.16.015. Their IDs were not yet reflected in either BC's Traceability "Stories:"
field (BC Backlink Update Obligation, scope-lock E-17).

**Changes:**

- BC-2.16.009 v1.9 → v1.10: `Stories` traceability row updated from `STORY-113` to
  `STORY-113, STORY-116, STORY-117`. No content change to preconditions, postconditions, invariants,
  edge cases, or verification properties.
- BC-2.16.015 v1.8 → v1.9: `Stories` traceability row updated from `STORY-112` to
  `STORY-112, STORY-116, STORY-117` (DF-SIBLING-SWEEP with BC-2.16.009 v1.10). No content change
  to preconditions, postconditions, invariants, edge cases, or verification properties.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.16.009 | v1.9 → v1.10: Stories row += STORY-116, STORY-117 | `.factory/specs/behavioral-contracts/ss-16/BC-2.16.009.md` |
| BC-2.16.015 | v1.8 → v1.9: Stories row += STORY-116, STORY-117 | `.factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md` |

**Story input-hash note:** Editing BC-2.16.009 and BC-2.16.015 invalidates the STORY-116/STORY-117
`input-hash` field (previously `46dcf52`). Re-stamping is deferred to the story-writer in the next
burst per CLAUDE.md input-hash policy — do NOT re-stamp here.

**Pre-existing gap (flagged, not fixed):** STORY-114 and STORY-115 also consume these BCs but are
absent from the Stories traceability rows (STORY-113-only for BC-2.16.009; STORY-112-only for
BC-2.16.015 prior to this entry). These omissions pre-date this burst. They are outside the
scope-lock for E-17 F3 and must be addressed in a separate traceability sweep.

---

## [e17-f2-qinq-macsec-documented-limitation-2026-06-16] — 2026-06-16

### PATCH: BC-2.16.009 v1.7→v1.9 + BC-2.16.015 v1.6→v1.8 — E-17 QinQ/MACsec offset confirmed + MACsec documented-limitation clause (final versions after DF-CONSISTENCY-AUDIT follow-on bumps)

**Feature cycle:** E-17 "ARP Decoder VLAN/QinQ/MACsec Offset Hardening" (issue #253)
**F1 human gate outcome:** PASSED — documented-limitation, no src/ production change, v0.7.1 patch target.
**Evidence base:** etherparse 0.20.2 source (`macsec_header_slice.rs:246-248`); etherparse upstream
proptest (`macsec_header.rs:340-347`); etherparse conformance test (`lax_packet_headers.rs:1371-1419`);
wirerust observe-only probe test `test_BC_2_16_015_macsec_arp_lax_parse_probe` in
`tests/bc_2_16_qinq_macsec_offset_tests.rs` (PR #258, 4 tests: QinQ behavioral, QinQ model-pin,
QinQ malformed→D11, MACsec observe-only probe — no offset assertion); wirerust offset-assertion
tests `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` and
`test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` in
`tests/bc_2_16_e17_macsec_offset_tests.rs` (6 tests, branch test/arp-qinq-macsec-fixtures,
extends PR #258, committed in F4). The offset==22 and offset==30 arithmetic assertions reside
ONLY in `bc_2_16_e17_macsec_offset_tests.rs`; the qinq file's MACsec test is observe-only.

**Summary of changes:**

Both BCs receive an incremental documented-limitation clause for stacked link extensions
covering QinQ and MACsec, satisfying the F2 spec-evolution obligation from the F1 delta
analysis (§2.2 and §3.4). No observable D11 outcome or precondition gate is changed.

**New EC-009 in both BCs (identical substance; DF-SIBLING-SWEEP):**

> For MACsec-tagged ARP frames (EtherType 0x88E5):
> (a) Offset correctness — PROVEN: `LaxLinkExtSlice::header_len()` returns 8 for Unmodified/no-SCI
> (6-byte SecTag + 2-byte next-EtherType) and 16 for Unmodified/SCI-present (6-byte SecTag +
> 8-byte SCI + 2-byte next-EtherType). The SCI bytes ARE included. Confirmed arp_offset values
> over Ethernet2: 22 (no-SCI) and 30 (SCI-present), landing exactly on ARP byte 0.
> (b) Encrypted/Modified payloads — safe by construction: `stop_err == Layer::Arp` is unreachable
> for Modified, Encrypted, and EncryptedUnmodified MACsec variants (etherparse lax driver executes
> `return result` before the inner-ARP parse block). These fall to the generic decode path — a
> security property, not a gap.
> (c) DOCUMENTED-UNVERIFIED: no public on-wire MACsec-over-ARP PCAP exists; correctness is proven
> by etherparse source + upstream proptest + synthetic probe test, not real-traffic fixture. No code
> change planned until a failing real-world test demonstrates a defect.

**Confirmed QinQ offset (also added):** QinQ (outer 0x88a8 + inner 0x8100, two Vlan link_exts)
= offset 22 (14+4+4); the +8 link-exts sum is confirmed by
`test_BC_2_16_015_qinq_link_exts_offset_formula_pin`; the full offset-22 ARP byte-read is
confirmed by `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11`.
(Citation corrected in E-17 F2 adversarial finding M-1.)

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.16.009 | v1.7 → v1.9: Precondition 2 lax-path updated with confirmed QinQ/MACsec offset values; EC-008 updated with full offset table and MACsec Encrypted/Modified note; EC-009 added (documented-limitation clause) [v1.8 = initial E-17 burst; v1.9 = DF-CONSISTENCY-AUDIT follow-on — see sub-note below] | `.factory/specs/behavioral-contracts/ss-16/BC-2.16.009.md` |
| BC-2.16.015 | v1.6 → v1.8: PC-7a updated with confirmed offset values (QinQ=22, MACsec Unmodified/no-SCI=22, MACsec Unmodified/SCI=30); 7b distinction note updated to name QinQ/MACsec explicitly; EC-008 updated with confirmed offset table; EC-009 added (identical substance to BC-2.16.009 v1.9 EC-009; DF-SIBLING-SWEEP) [v1.7 = initial E-17 burst; v1.8 = DF-CONSISTENCY-AUDIT follow-on — see sub-note below] | `.factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md` |
| arp-architecture-delta.md | v1.17 → v1.19: v1.18 = initial E-17 changelog reference (architect burst); v1.19 = DF-CONSISTENCY-AUDIT §2.2 None-arm snippet corrected to `Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))` matching shipped src/decoder.rs:209/256 (adversarial finding H-1) | `.factory/specs/architecture/arp-architecture-delta.md` |
| VP-024 (vp-024-arp-parse-safety.md) | v2.4 (final after DF-CONSISTENCY-AUDIT follow-on bumps) | `.factory/specs/verification-properties/vp-024-arp-parse-safety.md` |
| verification-coverage-matrix.md | v1.8 (final after DF-CONSISTENCY-AUDIT follow-on bumps — E-17 governance note added to VP-024 row; Coverage Notes updated) | `.factory/specs/architecture/verification-coverage-matrix.md` |

**DF-CONSISTENCY-AUDIT follow-on bumps (previously unlogged):**

The initial E-17 burst (above) recorded BC-2.16.009 at v1.8 and BC-2.16.015 at v1.7. A
subsequent DF-CONSISTENCY-AUDIT pass produced additional incremental bumps that were not
propagated to this central changelog entry until now (adversarial finding F-1):

(a) **BC-2.16.009 v1.8→v1.9 and BC-2.16.015 v1.7→v1.8:** input-hash TBD→null. BC files are
    exempt from the story input-hash mechanism per CLAUDE.md (input-hash tracks story source
    inputs, not BC artifacts themselves).

(b) **BC-2.16.015 PC-7a QinQ offset-22 citation split:** The QinQ offset-22 confirmation was
    split into two distinct test citations — `test_BC_2_16_015_qinq_link_exts_offset_formula_pin`
    (the +8 link-exts-sum arithmetic) and
    `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11` (the byte-read at
    offset 22 confirming no false-positive D11). Each citation is now anchored to exactly one
    test function.

(c) **arch-delta v1.18→v1.19:** §2.2 authoritative code snippet corrected — the None-arm
    return value was shown as `None`; corrected to `Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))`
    to match shipped `src/decoder.rs:209/256` (adversarial finding H-1). This is a documentation
    correction to the architecture delta document only; no source code was changed.

**Story-writer handoff:** no story frontmatter `bcs:` array changes (EC additions are additive,
no existing BC IDs changed). Architect handoff consolidated above.

---

## [bc-2.16.009-015-d078-lax-malformed-d11-2026-06-15] — 2026-06-15

### PATCH: BC-2.16.009 v1.4→v1.5 + BC-2.16.015 v1.3→v1.4 — D-078 lax-built malformed ARP routes to D11 (F5 O-A fix)

**Decision:** D-078 (F5 finding O-A, human-adjudicated FIX). Spec-layer only; no code changes.

**Root cause:** The lax `Err(SliceError::Len(_))` arm in `decode_packet` handles two sub-cases
that were previously conflated. When the lax parser builds a `LaxNetSlice::Arp(arp)` slice but
`extract_arp_frame` returns `None` (because the 4-part type/size guard fails), this is a **D11
malformed condition** — the same as the strict path. But the code was emitting
`Err("truncated ARP frame")` for this sub-case, which is NOT routed to `record_malformed` (D11).
The correct behavior: `Err("Non-Ethernet/IPv4 ARP frame")` → D11 malformed finding, regardless
of which decode arm (strict or lax) built the `ArpPacketSlice`.

The ONLY genuine truncation case (→ `"truncated ARP frame"`, NOT D11) is when the lax parser
cannot build an `ArpPacketSlice` at all (`lax.net == None`, `stop_err == Layer::Arp`). That
path is unchanged.

#### BC-2.16.009 changes (v1.4 → v1.5)

- **Precondition 2 clarified:** Now states explicitly that `extract_arp_frame` receives an
  `ArpPacketSlice` from either the strict path or the lax path — path-independence is normative.
- **Precondition 3 (new D-078 note):** Added `**Path-independence (D-078)**` paragraph stating
  that the D11 condition applies regardless of which decode arm produced the slice.
- **EC-008 added:** "Snaplen-truncated ARP capture where lax parser builds an `ArpPacketSlice`
  but `extract_arp_frame` returns `None` (bad type/size fields) → D11 malformed finding
  (Err("Non-Ethernet/IPv4 ARP frame") + record_malformed). Distinct from EC-007 where lax parser
  cannot build a slice at all."

#### BC-2.16.015 changes (v1.3 → v1.4)

- **Postcondition 7 (formerly a single bullet) split into PC-7a and PC-7b:**
  - **(7a) Lax-built slice, extract fails → D11:** `Err("Non-Ethernet/IPv4 ARP frame")` →
    `record_malformed` → LOW D11 finding. (Old behavior: incorrectly said "truncated ARP frame".)
  - **(7b) Lax parser cannot build a slice → generic decode-error (NOT D11):**
    `Err("truncated ARP frame")` → existing generic decode-error handling, no D11 finding.
  - Added `**The distinction (D-078)**` paragraph: error string is the routing key.
- **EC-008 added:** Lax-built-slice + extract None sub-case → Err("Non-Ethernet/IPv4 ARP frame")
  → D11 (PC-7a). Documents the case that was previously absorbed as a generic decode-error.

**Stories referencing old behavior:** STORY-112 (AC-007/AC-008 reference `"truncated ARP frame"`
for the lax `None` case; these ACs must be updated to reflect the PC-7a/7b split). Story-writer
dispatch required for STORY-112 AC propagation.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/behavioral-contracts/ss-16/BC-2.16.009.md` | v1.4→v1.5; PC-2 and PC-3 path-independence note; EC-008 added |
| `.factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md` | v1.3→v1.4; PC-7 split into PC-7a/PC-7b; EC-008 added |
| `.factory/spec-changelog.md` | this entry |

---

## [prd-v1.25-ss15-titlesync-2026-06-14] — 2026-06-14

### PATCH: PRD v1.24→v1.25 — §2.15 BC Index Title-Sync (post-Pass-26 consistency flush)

**Scope:** `prd.md` §2.15 BC index rows BC-2.15.009 and BC-2.15.016 synced to their
canonical H1 headings. No BC files modified; no BC count change (still 283).

#### FIX-1 — §2.15.C BC-2.15.009 row title synced to H1

The PRD §2.15.C index row subtitle had drifted from the canonical BC H1. The stale
"first 16 bytes" framing was replaced with the correct H1 wording:
"Initial-Delivery No-Sync (One-Shot, First Delivery Only)".

#### FIX-2 — §2.15.F BC-2.15.016 row title synced to H1

The PRD §2.15.F index row for BC-2.15.016 was missing the "master_addrs ≤64,
pending_requests ≤256" bounds that appear in the canonical BC H1. The row title has
been updated to include these bounds.

**Context:** This title-sync is part of the broader post-Pass-26 full-corpus consistency
flush, which also covered: VP-006 Must→Should table correction, src-citation
symbol-anchoring across multiple BCs, and removal of stale line-pin references. Only
the PRD §2.15 title-sync rows constitute the 1.24→1.25 bump; the other flush items were
committed in the same burst under their own version records.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/prd.md` | §2.15.C BC-2.15.009 row title synced to H1 ("Initial-Delivery No-Sync (One-Shot, First Delivery Only)"; removed stale "first 16 bytes" framing) [FIX-1]; §2.15.F BC-2.15.016 row title synced to H1 (added master_addrs ≤64 / pending_requests ≤256 bounds) [FIX-2]; "Version 1.25 delta" note added; frontmatter v1.24→v1.25 |
| `.factory/spec-changelog.md` | this entry |

---

## [pass-25-f3-convergence-2026-06-14] — 2026-06-14

### PATCH: Pass-25 F3-Convergence Remediation (holdout/story layer)

**Scope:** wave-40-44-holdout.md (3 corrections); STORY-114 (1 correction). Performed by
story-writer in the same burst as Pass-25 spec-changelog creation. No new BCs; no BC count
change (still 283).

#### FIX-1 (HIGH) — wave-40-44-holdout.md: Remove re-introduced non-existent detection ID "D14"

Three occurrences of the non-existent detection ID "D14" had been re-introduced into
`wave-40-44-holdout.md`, contradicting the canonical F-ARP-C2 correction that removed "D14"
in a prior burst. This was a sibling-sweep miss: the F-ARP-C2 fix removed "D14" from some
locations but not all three occurrences.

**Corrections applied (3 occurrences):**
- First occurrence: `"D14"` → `"BC-2.16.014"` (canonical detection reference)
- Second occurrence: `"D14"` → `"BC-2.16.014"` (canonical detection reference)
- Third occurrence: `"(D2 + D14)"` → `"(D2 + BC-2.16.014)"` (canonical compound reference)

All three corrections align with the F-ARP-C2 canonical form: detection IDs use the full
BC-S.SS.NNN identifier, not shorthand aliases like "D14".

#### FIX-2 (LOW) — wave-40-44-holdout.md: De-pin stale cross-file line citation + tighten storm frame-count narrative

Two related issues in `wave-40-44-holdout.md` HS-W44-ARP-D3 Scenario A:

1. **Stale cross-file line citation removed:** `STORY-113:293` pinned to a specific line
   number was stale (story files move as content evolves). Replaced with stable concept anchor
   referencing the STORY-113 storm-detection implementation section — no line number.

2. **Storm frame-count narrative tightened:** The Scenario A description stated the finding
   fires at "frame 101" which was inconsistent with the canonical D3 storm detection behavior.
   Corrected to "frame 50" — the finding fires when the storm threshold is crossed at frame 50,
   not after a full 101-frame sequence.

#### FIX-3 (LOW) — STORY-114: De-pin stale HS-008 line citations

`STORY-114` contained two stale ":75" line-number citations referencing
`HS-008-mitre-tactic-display-and-kill-chain-order.md:75`. Line citations to holdout scenario
files are brittle (scenario files evolve independently). Replaced both ":75" pins with stable
concept anchors referencing the IcsImpact display obligation in HS-008. The string
`"Impact (ICS)"` is confirmed correct and unchanged per D-069.

**Artifacts changed in this burst:**

| Artifact | Change |
|----------|--------|
| `.factory/feature/wave-holdout-scenarios/wave-40-44-holdout.md` | FIX-1: 3 occurrences of "D14" → "BC-2.16.014"; FIX-2: de-pinned STORY-113:293 stale line cite; tightened HS-W44-ARP-D3 Scenario A frame-count narrative (frame 50, not 101) |
| `.factory/stories/STORY-114.md` | FIX-3: 2 stale HS-008:75 line citations → stable concept anchors |

---

## [pass-24-f3-convergence-2026-06-14] — 2026-06-14

### PATCH: Pass-24 F3-Convergence Remediation (FIX-1, FIX-2)

**Scope:** BC-2.15.017 v1.3→v1.4 (CRITICAL spec↔code mis-anchor revert);
PRD §2.16.F BC-2.16.010 title-sync enrichment (LOW). No new BCs; no BC count change (still 283).

#### FIX-1 (CRITICAL) — BC-2.15.017 v1.3→v1.4: Revert erroneous Pass-22 constant rename

The Pass-22 F3-convergence burst erroneously renamed the DNP3 direct-operate threshold
constant in BC-2.15.017 from `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` to
`DNP3_DIRECT_OPERATE_THRESHOLD_DEFAULT`. This rename was a spec↔code mis-anchor:
`DNPXX_` is the actual shipped constant name in the codebase, not `DNP3_`.

**Source-of-truth verification:**
- `src/analyzer/dnp3.rs:169` — defines `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`
- `src/cli.rs:16` and `src/cli.rs:183` — references `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`
- `STORY-110` — implements using `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`

The `DNPXX_` prefix is intentional in the shipped code (the prefix reflects the DNP3
protocol's cross-protocol abbreviation convention). It is NOT a typo; the `DNP3_` form
does not exist in the codebase. The Pass-22 rename introduced a false divergence between
spec and code.

**Fix:** All three live occurrences in BC-2.15.017 restored to `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`:
- Precondition 2
- Architecture Anchor: cli.rs reference
- Architecture Anchor: dnp3-architecture-delta.md reference

BC-2.15.017 bumped v1.3 → v1.4. The sealed historical v1.3 changelog entry is preserved as-is.

**Note for orchestrator:** The `DNPXX_` prefix is a code-quality tech-debt candidate for a
future code-cleanup pass (the prefix is non-standard for a DNP3-only constant). It is NOT
an F3 fix target — changing it would require coordinated source + spec + story updates.

#### FIX-2 (LOW) — PRD §2.16.F BC-2.16.010 title-sync "(11 Keys)" enrichment

The §2.16.F index row for BC-2.16.010 had title:
> "ArpAnalyzer::summarize() returns AnalysisSummary with required keys"

The canonical BC H1 (BC-2.16.010.md) and BC-INDEX both include the "(11 Keys)" enrichment
per Criterion-75 (BC H1 is title source of truth). The PRD index row was out of sync.

**Fix:** PRD §2.16.F BC-2.16.010 index row title updated to:
> "ArpAnalyzer::summarize() returns AnalysisSummary with required keys (11 Keys)"

This matches BC-2.16.010.md H1 and BC-INDEX verbatim. No BC content changed; no BC count change.

**Artifacts changed in this burst:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.017.md` | v1.3→v1.4: 3 occurrences of `DNP3_DIRECT_OPERATE_THRESHOLD_DEFAULT` reverted to `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`; v1.4 modified-log entry added |
| `.factory/specs/prd.md` | v1.24 delta note added; §2.16.F BC-2.16.010 row title enriched with "(11 Keys)" |

---

## [pass-22-f3-convergence-2026-06-14] — 2026-06-14

### PATCH: Pass-22 F3-Convergence PRD Reconciliation (FIX-1, FIX-2, FIX-3)

**Scope:** prd.md v1.22 delta note; DRIFT-PRD-ARP-SEED-COUNT-001; DRIFT-PRD-V120-MBAPFRAMER-001;
BC-2.02.009 v1.6 annotation.

#### FIX-1 (HIGH) — ARP holdout seed-count reconciliation: 26 → 28

The v1.10 and v1.11 delta notes in prd.md recorded the total ARP holdout seed count as 26
(24 P0, 2 P1) and listed HS-W44-001 as P1. These notes accurately reflected the HS-INDEX
state at that time (v1.2–v1.3). The HS-INDEX was subsequently expanded to v1.6, adding
HS-W44-004 through HS-W44-007 (7 seeds in wave 44 vs. the prior 3) and reclassifying
HS-W44-001 from P1 to P0.

Canonical HS-INDEX v1.6 values:
- **Total ARP feature holdout seeds = 28 (27 P0, 1 P1)**
- Single P1 seed: **HS-W44-003** (--arp-storm-rate override) ONLY
- **HS-W44-001 is P0** (D3 storm detection)
- frontmatter `arp_waves_40_44 = 28`
- Wave breakdown: W40=4, W41=4, W42=8, W43=5, W44=7

The v1.10/v1.11 historical notes are preserved as-is (immutable history). The v1.22 delta note
is the authoritative reconciliation record. **DRIFT-PRD-ARP-SEED-COUNT-001 CLOSED.**

#### FIX-2 (LOW) — BC-2.02.009 v1.6 annotation

The v1.13 delta note cites "canonical v1.5 H1/BC-INDEX title" for BC-2.02.009. That title was
subsequently superseded: BC-2.02.009 was further revised to v1.6 (per BC-INDEX.md, ARCH-INDEX
ADR-008, and spec-changelog). The v1.13 historical note is preserved; this annotation records
that BC-2.02.009 was bumped to v1.6 after the v1.13 pass. The §2.2 live-body row title already
reflects the current BC-INDEX H1 — no live-body change required.

#### FIX-3 (LOW) — v1.20 MbapFramer prose corrected

The v1.20 delta note (Pass-24) erroneously stated "C-23 was MbapFramer, a Modbus component."
No MbapFramer component ever existed in the architecture. The correct history: C-23 was
previously assigned to SS-15/DNP3 (Dnp3Analyzer); SS-15/DNP3 was renumbered from C-23 to C-24
when the ARP analyzer (SS-16/ArpAnalyzer) claimed C-23. The v1.20 prose error is corrected in
the v1.20 delta text. **DRIFT-PRD-V120-MBAPFRAMER-001 CLOSED.**

**Note — other Pass-22 fixes recorded by their owners:** SS-15 story-anchor back-fill (story-writer
burst); VP-024 module anchor (architect burst); VP-INDEX 5-BC footnote clarification (architect
burst); dep-graph-extended superseded (architect burst). Those changes are in their respective
agent's changelog entries.

**Artifacts changed in this burst:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/prd.md` | v1.22 delta note added (FIX-1/FIX-2/FIX-3); DRIFT-PRD-ARP-SEED-COUNT-001 CLOSED; DRIFT-PRD-V120-MBAPFRAMER-001 CLOSED; BC-2.02.009 v1.6 annotation |

---

## [pass-5-propagation-gap-fixes-2026-06-14] — 2026-06-14

### PATCH: Pass-5 Adversarial Propagation-Gap Remediation (F-A-1, F-A-2, F-C-1, F-C-2, F-C-3)

**Scope:** BC-2.16.003 Architecture Anchors, HS-025, HS-INDEX HS-W42-001, spec-changelog.

#### F-A-1 (MEDIUM) — BC-2.16.003 v1.6 → v1.7: Architecture Anchors §3.3 conditional form

The D-068 sweep updated all body sections (Description, PC5, Invariants, EC, test vectors,
Traceability) but left Architecture Anchors line for `arp-architecture-delta.md §3.3` as
unconditional "D2 confidence=LOW, finding_type=Anomaly, MITRE T0830+T1557.002" — contradicting
the v1.6 body which asserts `mitre_techniques: []` for benign GARP.

**Fix:** Architecture Anchors §3.3 line replaced with conditional form:
"D2 confidence=LOW (base), finding_type=Anomaly; MITRE: none (mitre_techniques=[]) for benign
non-conflicting GARP; T0830+T1557.002 emitted ONLY on GARP-that-conflicts escalation path
(BC-2.16.014)".

Full-file grep confirmed no other unconditional benign-GARP MITRE assertions remain in
BC-2.16.003. All other T0830/T1557.002 references are correctly gated on the conflict path or
are historical changelog entries.

BC-2.16.003 bumped v1.6 → v1.7.

**STORY-113 re-stamp COMPLETED:** BC-2.16.003 is a declared input of STORY-113. STORY-113
was re-stamped after the v1.7 change — `bin/compute-input-hash --scan` confirmed MATCH (current
hash: see STORY-113 frontmatter — frontmatter is the canonical source per DF-INPUT-HASH-CANONICAL-001).

#### F-A-2 (LOW) — spec-changelog D-068 entry: dangling "See input-hash re-stamp list below"

The D-068 entry (line ~99) referenced "See input-hash re-stamp list below" but no list
followed. Replaced with the actual re-stamp list: STORY-071, STORY-100, STORY-106 through
STORY-115. Added note that STORY-113 requires additional re-stamping after BC-2.16.003 v1.7
(F-A-1 fix).

#### F-C-1 (LOW) — HS-025 v1.1 → v1.2: IcsImpact D-069 disambiguation clause added

HS-025 tested BC-2.10.002 only for the no-"ICS:"-prefix rule but did not surface that
IcsImpact now displays as "Impact (ICS)" (D-069 / BC-2.10.002 v1.5). Added explicit clause:

- Scenario step 1 extended: IcsImpact renders as "Impact (ICS)" (parenthetical ICS qualifier,
  NOT an "ICS:" prefix); distinguishes ICS Impact TA0105 from Enterprise Impact TA0040.
- BC table: BC-2.10.002 row extended to cite PC3 (IcsImpact = "Impact (ICS)") and PC4
  (the "(ICS)" qualifier does not constitute an "ICS:" prefix).
- Verification Approach: added note that "Impact (ICS)" passes the no-"ICS:"-prefix check.
- Evaluation Rubric data integrity row: added expected strings for all three ICS tactics and
  Enterprise Impact, making the rubric self-consistent with D-069.
- Failure Guidance: updated to mention IcsImpact bare-"Impact" failure mode.

#### F-C-2 (LOW) — HS-INDEX HS-W42-001: "no MITRE techniques" added to seed title

HS-W42-001 "GARP Benign Baseline" seed title did not surface the D-068 mitre_techniques: []
requirement. Added "mitre_techniques: [] (no MITRE techniques attributed to benign GARP per
D-068)" to the seed title so the index communicates the no-tag obligation.

#### F-C-3 (LOW NOTE) — HS-008 and HS-025 identical input-hash: informational note added

Both HS-008 and HS-025 carry identical input-hash values (current values: see each file's frontmatter — frontmatter is the canonical source per DF-INPUT-HASH-CANONICAL-001). Per CLAUDE.md, bin/compute-input-hash
scopes to .factory/stories/ only — holdout scenario hashes are NOT maintained by the canonical
drift tool. Added a one-line YAML comment below each file's input-hash field clarifying the
field is informational only and not tool-maintained.

**Artifacts changed in this burst:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/behavioral-contracts/ss-16/BC-2.16.003.md` | v1.6 → v1.7: Architecture Anchors §3.3 line conditional; v1.7 modified-log entry added |
| `.factory/holdout-scenarios/HS-025-ics-tactic-display-and-non-exhaustive.md` | v1.1 → v1.2: IcsImpact D-069 disambiguation clause added to Scenario, BC table, Verification Approach, Evaluation Rubric, Failure Guidance; input-hash informational note added |
| `.factory/holdout-scenarios/HS-008-mitre-tactic-display-and-kill-chain-order.md` | input-hash informational note added (no content change) |
| `.factory/holdout-scenarios/HS-INDEX.md` | HS-W42-001 seed title: "mitre_techniques: []" clause added |
| `.factory/spec-changelog.md` | D-068 entry: dangling "See ... below" replaced with actual re-stamp list; this Pass-5 entry added |

**Story-writer handoff COMPLETED:** STORY-113 input-hash re-stamping for BC-2.16.003 v1.7 is
done. `bin/compute-input-hash --scan` confirmed MATCH for all ARP stories 111-115 (current
hashes: see each story's frontmatter — frontmatter is the canonical source per
DF-INPUT-HASH-CANONICAL-001). No further action required.

---

## [d-069-icsimpact-display-impact-ics-2026-06-14] — 2026-06-14

### ADJUDICATION D-069: IcsImpact Display = "Impact (ICS)" — SUPERSEDES D-067

**Research basis:** `.factory/research/mitre-impact-tactic-disambiguation.md`
(WCAG 2.4.6 unique headings; MITRE ATT&CK TA0040 vs TA0105; MITRE Navigator single-domain
model; cross-matrix tool conventions).

**Decision:** `MitreTactic::IcsImpact` MUST render as `"Impact (ICS)"`, NOT bare `"Impact"`.
A grouped findings report that co-renders Enterprise Impact (TA0040) and ICS Impact (TA0105)
in the same output without a matrix-selection guard creates two identically-named section
headers — a WCAG 2.4.6 violation and a high-blast-radius reader confusion risk in the security
context (Enterprise ransomware/data-destruction vs ICS physical-process manipulation).
`src/mitre.rs:91 = "Impact (ICS)"` is CORRECT. The prior D-067 adjudication ("Impact" bare,
spec correct; code deviant) is REVERSED. D-069 SUPERSEDES D-067 in all respects.

**ADR-008 Decision 5 impact:** Architect handles ADR-008 update in parallel under D-069.

**Artifacts corrected in this burst:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.10.002 | v1.4 → v1.5: H1 title updated; Description rewritten; PC3 changed from "Impact" to "Impact (ICS)"; PC4 updated; Invariant 2 updated; EC-003 updated; canonical test vector updated. Modified-log entry citing D-069 added. | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.002.md` |
| PRD v1.5 delta note (IcsImpact variant) | `Display "Impact"` → `Display "Impact (ICS)"` | `.factory/specs/prd.md` |
| PRD §2.15 'New ICS tactic variant' note (IcsImpact) | `Display "Impact"` → `Display "Impact (ICS)"`; D-069 note added | `.factory/specs/prd.md` |
| arp-architecture-delta.md §5.0 brownfield-debt table | Resolution direction REVERSED: `"Impact (ICS)"` is correct; `"Impact"` (bare) was wrong. Table updated to RESOLVED. Narrative text updated. | `.factory/specs/architecture/arp-architecture-delta.md` §5.0 |
| arp-architecture-delta.md §7 | v1.12 changelog entry added documenting D-069 resolution | `.factory/specs/architecture/arp-architecture-delta.md` §7 |
| ADR-007 Decision 5 | Code snippet `MitreTactic::IcsImpact => "Impact"` → `"Impact (ICS)"`. Modified log entry added superseding F-A13-001 note. | `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md` |
| HS-008 (line ~75) | VERIFIED correct already — `"Impact (ICS)"` is present and MUST NOT be changed | `.factory/holdout-scenarios/HS-008-mitre-tactic-display-and-kill-chain-order.md` |

**Story-writer obligations (next burst):**
STORY-114 body references D-067 obligations that are now REVERSED by D-069. The
"IcsImpact Display fix" task in STORY-114 must be corrected from "change src/mitre.rs:91 to
'Impact'" to "VERIFY src/mitre.rs:91 = 'Impact (ICS)' (no change required; code is correct)".
The test obligation changes from `assert_eq!(format!("{}",
MitreTactic::IcsImpact), "Impact")` to `assert_eq!(format!("{}",
MitreTactic::IcsImpact), "Impact (ICS)")`. See STORY-114 body for full scope.

**State-manager obligation:**
STATE.md lines 99-100, 117-119 record D-067 adjudication with the now-reversed conclusion.
State-manager must update STATE.md to note D-069 supersession of D-067, and correct the
STORY-114 carry-forward obligations accordingly.

---

## [d-068-benign-garp-no-mitre-2026-06-14] — 2026-06-14

### ADJUDICATION D-068: Benign Gratuitous ARP emits mitre_techniques: [] (no MITRE attribution)

**Research basis:** `.factory/research/arp-garp-mitre-attribution.md`
(MITRE T1557.002 DET0387 AN1091-AN1093; T0830 detection guidance; arpwatch/Zeek/Suricata/Snort
convention; RFC 826; RFC 5227 ACD).

**Decision:** A benign (non-conflicting) Gratuitous ARP (sender_ip == target_ip, with NO prior
binding conflict) MUST NOT be attributed to MITRE T0830 or T1557.002. Those techniques describe
the *malicious/overriding* ARP announcement — a GARP claiming an IP already bound to a different
MAC. Benign GARP (first announcement or re-announcement consistent with the known binding) is
routine network traffic emitted at link-up, DHCP lease, RFC 5227 ACD, and HA/VRRP failover.
MITRE's own DET0387 analytics and all reference IDS tools (arpwatch, Zeek, Suricata, Snort)
gate AiTM attribution on *binding conflict*, not on the GARP mechanism itself.

D2 GARP finding (non-conflicting): `mitre_techniques: []` (empty).
T0830 + T1557.002 attribution: exclusively the GARP-that-conflicts escalation (BC-2.16.014,
which co-triggers D1). BC-2.16.014 is UNCHANGED and CORRECT — it already emits T0830+T1557.002
on the conflict path.

**ADR-008 Decision 5 impact:** Architect handles ADR-008 update in parallel under D-068.

**Artifacts corrected in this burst:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.16.003 | v1.5 → v1.6: Description, PC5, Invariant 2, Invariant 3, EC-001, EC-002, EC-007, and all canonical test vectors updated to emit mitre_techniques=[] for benign GARP. Traceability MITRE Techniques field updated. Source Evidence updated. Modified-log entry citing D-068 added. | `.factory/specs/behavioral-contracts/ss-16/BC-2.16.003.md` |
| BC-2.16.014 | VERIFIED correct (no change) — conflict path emits T0830+T1557.002 on both GARP MEDIUM finding and D1 spoof finding. PC1, PC2, Invariant 4 all correct. | `.factory/specs/behavioral-contracts/ss-16/BC-2.16.014.md` |
| HS-INDEX (wave seeds) | VERIFIED — HS-W42-001 title "GARP with no conflict produces LOW finding; no D1 spoof; VP-024 Sub-B" does NOT assert MITRE techniques in the benign case. Consistent with D-068. HS-W43-003 "GARP-That-Conflicts" and HS-W43-001/002 (D1 spoof) assert T0830+T1557.002, which is correct. | `.factory/holdout-scenarios/HS-INDEX.md` |

**No holdout scenario files required changes:** ARP holdout scenarios are seeds in HS-INDEX only;
no concrete wave-holdout.md files exist yet. The seeds are already consistent with D-068 (benign
GARP seed does not assert MITRE techniques; conflict seeds correctly assert T0830+T1557.002).

**Story-writer obligations (next burst):**
BC-2.16.003 is a declared input of STORY-113 only (verified by grepping each story's `inputs:`
list — STORY-071, STORY-100, STORY-106 through STORY-112, STORY-114, STORY-115 do NOT cite
BC-2.16.003). The broader re-stamp in this burst covered those other stories because of their
own changed inputs (arp-architecture-delta.md, BC-2.10.002, ADR-007, and others); each story
was re-stamped per its own declared inputs. STORY-113 was additionally re-stamped after
Pass-5 F-A-1 (BC-2.16.003 v1.7 Architecture Anchors fix — see pass-5-fixes-2026-06-14 entry)
— COMPLETED (current hash: see STORY-113 frontmatter — frontmatter is the canonical source per
DF-INPUT-HASH-CANONICAL-001).

---

## [pass-30-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-30 — Three FlowKey non-existent-accessor fixes (B-01/B-02/B-03 HIGH) + Story YAML dedup (C-01 HIGH)

#### B-01/B-02 HIGH — BC-2.14.020: source_ip uses non-existent FlowKey accessors (v2.2 → v2.3)

`FlowKey` exposes only `lower_ip()`, `upper_ip()`, `lower_port()`, `upper_port()`. The
`client_ip()` and `server_ip()` accessors referenced in BC-2.14.020 do not exist. The
F5 spec defect fix swept sibling BCs BC-2.14.013/014/015/017 and BC-2.14.019 but missed
BC-2.14.020.

- **B-01** (Unknown FC path postcondition): `source_ip: Some(flow_key.client_ip()) if ClientToServer; Some(flow_key.server_ip()) if ServerToClient` replaced with Direction-resolved endpoint description: resolve source endpoint from `direction` arg + `flow_key.lower_ip()`/`upper_ip()`, matching BC-2.14.019 §Path A/B and `src/analyzer/modbus.rs:374-381`.
- **B-02** (EC-010 edge case): `source_ip = flow_key.server_ip()` replaced with direction-resolved server/responder endpoint from `Direction::ServerToClient` and `flow_key.lower_ip()`/`flow_key.upper_ip()`, citing `FlowKey` has no `server_ip()` accessor.

**BC-2.14.020 bumped v2.2 → v2.3.** BC-INDEX annotation updated.

**Verified against exemplar:**
- BC-2.14.019 §Path A line ~104: "resolve server endpoint from `direction` arg and `flow_key.lower_ip()` / `flow_key.upper_ip()`"
- BC-2.14.019 §Path B line ~125-127: "resolved from `Direction::ClientToServer` and `flow_key.lower_ip()` / `flow_key.upper_ip()`"
- `src/analyzer/modbus.rs:374-381`: confirms `FlowKey` has no `client_ip()`/`server_ip()` (comment at line 375 notes this); resolves via port-502 heuristic + `lower_ip()`/`upper_ip()`.

#### B-03 HIGH — BC-2.14.018: source_ip uses non-existent FlowKey accessor (v1.2 → v1.3)

- **B-03** (postcondition line ~78): `source_ip: Some(flow_key.client_ip())` replaced with Direction-resolved client/initiator endpoint description matching BC-2.14.019 §Path B and `src/analyzer/modbus.rs:374-381`.

**BC-2.14.018 bumped v1.2 → v1.3.** BC-INDEX annotation updated.

**H1 titles unchanged:** BC-2.14.018 "Diagnostics FC 0x08 Sub-Function 0x0004 or 0x0001 Emits T0814 Denial of Service Finding" and BC-2.14.020 "Reconnaissance Function Codes (0x11, 0x2B/0x0E) Emit T0888 Remote System Information Discovery Finding" — confirmed identical to pre-patch.

#### C-01 HIGH — STORY-100..105: duplicate input-hash YAML keys removed

Each of the 6 story files had two top-level `input-hash:` keys in frontmatter — an early `input-hash: TBD` placeholder that was not removed when the real computed hash was added. Duplicate YAML mapping keys are invalid per the YAML 1.2 spec (behavior is parser-defined and typically last-value-wins, but the TBD placement made the intent ambiguous and is non-conforming).

**Fix:** Removed the `input-hash: TBD` line from each file, retaining the single real computed hash. Then verified and re-stamped all hashes using `bin/compute-input-hash --write` (canonical tool per CLAUDE.md).

| File | TBD removed | Result |
|------|-------------|--------|
| STORY-100.md | yes | MATCH (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-101.md | yes | MATCH (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-102.md | yes | MATCH (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-103.md | yes | MATCH (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-104.md | yes | MATCH (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-105.md | yes | MATCH (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |

STORY-103 hash was already correct from the original real hash. STORY-104 hash changed from the prior stamped value because BC-2.14.018 and BC-2.14.020 (both in its `inputs:` list) were modified by B-01/B-02/B-03 in this pass — re-stamp is correct and expected. STORY-100/101/102/105 hashes also drifted from their pre-dedup values, indicating the BC/input files changed since they were originally stamped (unrelated prior drift, correctly resolved by re-stamping).

No story body content was modified — this is a frontmatter YAML-dedup + hash-verification only. No story-writer propagation needed.

#### Architect A-01 bump (logged per task)

ADR-006 bump noted by architect in this pass cycle. No direct file touch by product-owner (architect owns ADR-006); logged here per F-D4-I1 obligation.

| Artifact | Change |
|----------|--------|
| BC-2.14.018 | v1.2 → v1.3 (B-03: source_ip Direction-resolved) |
| BC-2.14.020 | v2.2 → v2.3 (B-01/B-02: source_ip Direction-resolved) |
| BC-INDEX.md | v1.3 annotation added for BC-2.14.018; v2.3 annotation added for BC-2.14.020 |
| STORY-100.md | TBD dedup; re-stamped (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-101.md | TBD dedup; re-stamped (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-102.md | TBD dedup; re-stamped (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-103.md | TBD dedup; hash unchanged — was already correct (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-104.md | TBD dedup; re-stamped (BC-018/020 inputs changed; current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| STORY-105.md | TBD dedup; re-stamped (current hash: see story frontmatter — canonical source per DF-INPUT-HASH-CANONICAL-001) |
| spec-changelog.md | This entry |

---

## [pass-29-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-29 PRD findings D-01 (MED FC 0x17 omitted from write-set) + D-02 (LOW broken changelog anchor)

**D-01 MED — PRD omits FC 0x17 from BC-2.14.014 holding-register write set**

BC-2.14.014 v2.1 (BC-DISCREPANCY-001 reconciliation) defines the holding-register write set as
{0x06, 0x10, 0x16, 0x17}. FC 0x17 (Read/Write Multiple Registers) was added at v2.1 with ruling:
"0x17 writes holding registers → Modify Parameter (T0836)." The PRD listed only 0x06/0x10/0x16 in
four locations, omitting 0x17. All four locations corrected:

- §2 v2 co-emission box: `0x06/0x10/0x16` → `0x06/0x10/0x16/0x17`
- §2.14.D group header: `Holding-register FCs (0x06/0x10/0x16)` → `(0x06/0x10/0x16/0x17)`
- §2.14.D BC-2.14.014 index row title: `Write FC 0x06/0x10/0x16` → `Write FC 0x06/0x10/0x16/0x17`
- §6.5 KD-005 BC-2.14.014 row: `Holding-register writes (0x06/0x10/0x16)` → `(0x06/0x10/0x16/0x17)`

Verified against BC-2.14.014 v2.3 H1 (`# BC-2.14.014: Write FC 0x06/0x10/0x16/0x17 ...`),
Description §57-58, Precondition 3, Invariant 1, and Invariant 4 — all confirm the 4-FC set.
MITRE tags unchanged: `["T1692.001","T0836"]`.

**D-02 LOW — broken changelog anchor in v1.16 delta note (prd.md:~261)**

The v1.16 delta cited `spec-changelog.md §[pass-13-2026-06-13]` — a slug that does not exist.
The resolving slug is `[pass-13-corpus-cleanup-2026-06-13]` (spec-changelog.md line 2366, confirmed).
Corrected to the resolving slug.

**Architect P29 A-01 architecture doc bumps (logged per task):**
module-decomposition, system-overview, purity-boundary-map, module-criticality bumped per architect
P29 A-01 burst. Version numbers per architect report.

**prd.md:** v1.20 → v1.21 (v1.20 delta note also backfilled — it was missing from inline history).

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/prd.md` | D-01: 0x17 added to write-set in 4 locations; D-02: anchor slug corrected; v1.21 delta note added; v1.20 delta note backfilled; frontmatter v1.20 → v1.21 |
| `.factory/spec-changelog.md` | This entry |

---

## [pass-27-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-27 holdout layer fixes (C-01 MED kill-chain order, D-01 MED BC-version-pin flush)

Two targeted fixes to the holdout layer. No behavioral contract bodies or src changes.

**C-01 MED — HS-008: wrong kill-chain order narrative corrected (v1.1 → v1.2)**

Verification approach step 1 stated "Exfiltration, Command and Control at the end" — placing C2
after Exfiltration and implying C2 was the terminal Enterprise tactic. This is wrong.

Canonical order per `src/mitre.rs` `all_tactics_in_report_order` (verified lines 100-120):
Enterprise tactics end: ...Collection → CommandAndControl → Exfiltration → Impact, followed by
three ICS tactics (IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact).
So C2 (#12) precedes Exfiltration (#13); Impact (#14) is the last Enterprise tactic; ICS tactics
are appended after. The terminal tactic overall is IcsImpact.

Before (wrong): "... 'Exfiltration', 'Command and Control' at the end."
After (correct): "... 'Collection', 'Command and Control', 'Exfiltration', 'Impact' (in that order);
ICS tactics follow after all Enterprise tactics. 'Command and Control' precedes 'Exfiltration';
'Impact' is the last Enterprise tactic."

Cross-checked against: HS-081 (Defense Evasion before Command and Control — consistent);
test-vectors.md:319 (C2, Exfil, Impact order — consistent).

**D-01 MED — HS-INDEX: stale BC-2.02.009 v1.5 pin dropped; holdout BC-version-pin sweep (v1.3 → v1.4)**

HS-INDEX line 489 (HS-W44-007) cited "BC-2.02.009 v1.5" while the BC body is v1.6. Rather than
re-pinning to v1.6 (which lags again on the next BC bump), the version pin was dropped:
"BC-2.02.009 v1.5" → "BC-2.02.009 (revised)" matching the robust sibling pattern at lines 428-430.

**Holdout BC-version-pin sweep results:**

Grep pattern: `BC-2\.\d+\.\d+ v\d` across `/Users/zious/Documents/GITHUB/wirerust/.factory/holdout-scenarios/`

Active holdout files scanned: all .md files under `.factory/holdout-scenarios/`
Pins found: 1 (HS-INDEX:489, HS-W44-007, "BC-2.02.009 v1.5") — DROPPED (see D-01 above)

Additional hits in non-holdout factory paths:
- `.factory/cycles/` — historical session checkpoints and convergence trajectory (frozen-eval context) — PRESERVED
- `.factory/code-delivery/HS-043/` — PR description archive (frozen-eval context) — PRESERVED

No other active (non-historical) holdout files contained BC-version pins. The holdout
BC-version-pin lag class is fully flushed from the active holdout layer.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/holdout-scenarios/HS-008-mitre-tactic-display-and-kill-chain-order.md` | C-01: kill-chain order narrative corrected; v1.1 → v1.2 |
| `.factory/holdout-scenarios/HS-INDEX.md` | D-01: BC-2.02.009 v1.5 pin → (revised); v1.3 → v1.4 |
| `.factory/spec-changelog.md` | This entry |

---

## [pass-25-changelog-path-flush-2026-06-13] — 2026-06-13

### PATCH: Pass-25 comprehensive phantom-path flush in spec-changelog.md (D-01, D-02 + residual historical rows)

Four non-resolving paths were identified across spec-changelog.md. Two were active ledger "File"
column references (D-01 VP-023, D-02 VP-022); two were residual phantom strings appearing only
inside "corrected-from" audit-trail prose in the pass-24-fixes entry (D-03 arp-architecture-delta,
D-04 module-criticality). The corrected-from prose rows were preserved intact — they correctly
document the prior corrections; erasing them would destroy the audit trail.

**D-01 MED — spec-changelog.md: phantom VP-023 path corrected in DNP3 Feature #8 artifact table**

Active "File" column reference at the DNP3 burst entry cited
`.factory/specs/verification-properties/VP-023.md` (no such path). Corrected to
`.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md` (verified to exist).

**D-02 MED — spec-changelog.md: phantom VP-022 path corrected in Modbus Feature #7 artifact table**

Active "File" column reference at the Modbus burst entry cited
`.factory/specs/verification-properties/VP-022.md` (pending) (no such path; file was authored
with kebab-case slug). Corrected to `.factory/specs/verification-properties/vp-022-modbus-parse-safety.md`
(verified to exist). The "(pending)" annotation was removed as the file now exists.

**D-03 PRESERVED — pass-24-fixes entry: arp-architecture-delta phantom string in corrected-from prose**

The pass-24 D-03 fix description references `.factory/phase-f2-spec-evolution/arp-architecture-delta.md`
as the OLD (phantom) value in a "no such path → corrected to" audit entry. This is corrected-from
prose, not an active ledger reference. Preserved without change; the correct path
`.factory/specs/architecture/arp-architecture-delta.md` is already cited as the fixed target.

**D-04 PRESERVED — pass-24-fixes entry: module-criticality phantom string in corrected-from prose**

The pass-24 D-02 fix description references `.factory/specs/architecture/module-criticality.md`
as the OLD (phantom) value in a "no such path → corrected to" audit entry. This is corrected-from
prose, not an active ledger reference. Preserved without change; the correct path
`.factory/specs/module-criticality.md` is already cited as the fixed target.

**Verification:**

- `.factory/specs/verification-properties/vp-022-modbus-parse-safety.md` — confirmed exists
- `.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md` — confirmed exists
- `.factory/specs/module-criticality.md` — confirmed exists
- `.factory/specs/architecture/arp-architecture-delta.md` — confirmed exists
- Zero remaining active "File" column references point at non-resolving paths.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/spec-changelog.md` | Pass-25 path flush: 2 active phantom VP paths corrected (D-01 VP-023, D-02 VP-022); 2 corrected-from prose occurrences preserved (D-03, D-04) |

---

## [pass-24-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-24 remediation (D-01 HIGH, D-02 MED, D-03 MED) + architect A-01 bump logging

Three targeted fixes. No behavioral changes; component ID correction, phantom path corrections only.

**D-01 HIGH — DNP3 component mislabel sweep: C-23 → C-24 in all ss-15 BCs + prd.md (v1.19 → v1.20)**

The ARP analyzer (Feature #9) claimed C-23 as its component ID (ArpAnalyzer), which displaced the
DNP3 analyzer (Feature #8, Dnp3Analyzer) to C-24. This renumbering was never propagated to the 24
ss-15 behavioral contracts or to prd.md §2.15. Every Architecture Module traceability row in
BC-2.15.001..024 incorrectly cited "C-23" for the DNP3 component; prd.md §2.15 cited "C-26" (a
phantom that has never existed). All 24 ss-15 BCs corrected C-23 → C-24; prd.md corrected
C-26 → C-24. No legitimate C-23 (ArpAnalyzer) or C-22 (ModbusAnalyzer) references were touched —
all C-23 hits in ss-15 were exclusively "analyzer/dnp3.rs, C-23" mislabels. H1 titles unchanged.

**Architect A-01 note (Pass-23, previously unlogged):** arp-architecture-delta.md §7 table row
reorder was applied by architect in Pass-23 (A-05 ledger entry). Architect bumped
`.factory/specs/architecture/arp-architecture-delta.md` v1.10 → v1.11 as part of that pass.
That bump is now recorded in the corrected P23 ledger row (D-03 fix below).

**D-02 MED — spec-changelog.md Pass-23 ledger: phantom path corrected**

Row for A-04 cited `.factory/specs/architecture/module-criticality.md` (no such path). Corrected
to `.factory/specs/module-criticality.md` (verified to exist).

**D-03 MED — spec-changelog.md Pass-23 ledger: phantom path corrected**

Row for A-05 cited `.factory/phase-f2-spec-evolution/arp-architecture-delta.md` (no such path).
Corrected to `.factory/specs/architecture/arp-architecture-delta.md` (verified to exist).

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/prd.md` | `version: 1.19 → 1.20`; §2.15 C-26 → C-24 Dnp3Analyzer (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.001.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.002.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.003.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.004.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.005.md` | `version: 1.2 → 1.3`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.006.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.007.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.008.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.009.md` | `version: 1.3 → 1.4`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.010.md` | `version: 1.5 → 1.6`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.011.md` | `version: 1.2 → 1.3`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.012.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.013.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.014.md` | `version: 1.6 → 1.7`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.015.md` | `version: 1.5 → 1.6`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.016.md` | `version: 1.3 → 1.4`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.017.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.018.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.019.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.020.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.021.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.022.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.023.md` | `version: 1.1 → 1.2`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/specs/behavioral-contracts/ss-15/BC-2.15.024.md` | `version: 1.3 → 1.4`; Architecture Module C-23 → C-24 (D-01) |
| `.factory/spec-changelog.md` | P23 ledger rows A-04/A-05 phantom paths corrected (D-02/D-03); P24-fixes entry added |

---

## [pass-23-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-23 Slice-A architect bumps (A-01 through A-05) — Slices B/C/D CLEAN

Five architect-owned fixes in Slice A only. Slices B, C, D all returned CLEAN. No behavioral
changes; coverage-matrix attribution, footnote, code-fence, technique enumeration, and
draft-as-authoritative note only.

**A-01 MED — verification-coverage-matrix.md: VP-024 lock-note STORY-112/F6 → STORY-113/F6 (v1.5 → v1.6)**

The VP-024 draft lock-pending note added at Pass-22 (A-02) cited STORY-112/F6 as the expected
locking story. The correct story for VP-024 formal verification is STORY-113/F6 (ARP Kani
proofs). STORY-112 is the ArpAnalyzer implementation story; STORY-113 is the formal hardening
story. Attribution corrected to STORY-113/F6. NOTE: this finding is self-induced churn — the
P22 A-02 fix introduced the incorrect STORY number; this corrects it. Underlying VP-024 draft
status and lock-pending semantics are unchanged.

**A-02 LOW — verification-coverage-matrix.md: decoder.rs Sub-A attribution footnote added (v1.5 → v1.6)**

Decoder.rs etherparse migration (Sub-A) is tested under VP-024 coverage but was not mentioned
in the matrix row. Added a footnote clarifying decoder.rs Sub-A scope within the VP-024 entry.
(Combined with A-01 in the same v1.6 bump.)

**A-03 LOW — verification-architecture.md: VP-005 harness skeleton markdown code-fence fixed (v1.6 → v1.7)**

The VP-005 harness skeleton section contained a malformed markdown code-fence (missing closing
triple-backtick). Fixed to properly fence the skeleton block.

**A-04 LOW — module-criticality.md: C-22 Modbus emitted technique-ID enumeration harmonized with C-23/C-24 (v1.2 → v1.3)**

C-22 (ModbusAnalyzer) entry listed the emitted technique IDs in a format inconsistent with the
C-23 (ArpAnalyzer) and C-24 (Dnp3Analyzer) sibling entries. Enumeration prose harmonized
to match the C-23/C-24 style.

**A-05 LOW — arp-architecture-delta.md: §6 draft-as-authoritative intentionality note added (v1.10 → v1.11)**

Section 6 of arp-architecture-delta uses future-tense/draft language for STORY-111..115
implementation obligations. Added a brief note clarifying that this section is intentionally
draft-as-authoritative (F2 spec forward-declarations for F4 implementation) to prevent future
adversary passes from re-flagging it as unfinished.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/architecture/verification-coverage-matrix.md` | `version: 1.5 → 1.6`; VP-024 lock-note STORY-112→STORY-113 (A-01); decoder.rs Sub-A footnote added (A-02) |
| `.factory/specs/architecture/verification-architecture.md` | `version: 1.6 → 1.7`; VP-005 harness skeleton code-fence fixed (A-03) |
| `.factory/specs/module-criticality.md` | `version: 1.2 → 1.3`; C-22 Modbus technique enumeration harmonized with C-23/C-24 (A-04) |
| `.factory/specs/architecture/arp-architecture-delta.md` | `version: 1.10 → 1.11`; §6 draft-as-authoritative intentionality note added (A-05) |
| `.factory/spec-changelog.md` | P23-fixes entry added; all architect P23 bumps recorded |

---

## [pass-22-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-22 remediation (C-01, D-01, B-01) + architect P22 bumps (A-01, A-02) + proactive version-citation hardening

Five targeted fixes across three artifacts plus architect version bump logging. No behavioral
changes; stale count, stale version pin, formatting, and version-citation hardening only.

**C-01 MED — domain-debt.md O-04: technique count updated 21→23 (v1.2 → v1.3)**

O-04 stated 21 IDs (15 brownfield + 6 Modbus ICS Feature #7). Feature #8 (DNP3 ICS analyzer)
added T1691.001 (IcsInhibitResponseFunction) and T0827 (IcsImpact) to technique_info, bringing
the catalog to 23 IDs. Verified: src/mitre.rs line 341 `SEEDED_TECHNIQUE_ID_COUNT: usize = 23`.
The 8 staged/never-emitted count is unchanged; the seeded total grows 21→23.

Before: `technique_info contains 21 IDs (15 brownfield + 6 Modbus ICS techniques added in Feature #7)`
After: `technique_info contains 23 IDs (15 brownfield + 6 Modbus ICS techniques added in Feature #7 + 2 DNP3 ICS techniques added in Feature #8: T1691.001 and T0827)`

**D-01 LOW — BC-INDEX.md: PRD version pin removed to prevent recurrence (v1.25 → v1.26)**

Status line cited `UPDATED (v1.18)` while PRD is now v1.19. Rather than update to (v1.19)
and face the same lag next pass, the version pin was dropped entirely. The line now reads
`PRD index (prd.md): UPDATED -- all 283 L3 BC IDs are registered` — robust to all future
PRD version bumps.

**B-01 LOW — BC-INDEX.md: double blank line between ss-11 and ss-12 removed (v1.25 → v1.26)**

Lines 289-290 contained two consecutive blank lines between the ss-11 table end and the
`## ss-12` section header. Every other section transition uses a single blank line.
The extra blank line was removed for consistency. (Combined with D-01 in the same v1.26 bump.)

**A-01 (architect) — verification-architecture.md: v1.5 → v1.6**

Architect P22 bump. This entry logs the architect-owned artifact version transition;
verification-architecture.md is at v1.6 as of Pass-22.

**A-02 (architect) — verification-coverage-matrix.md: v1.4 → v1.5**

Architect P22 bump. verification-coverage-matrix.md is at v1.5 as of Pass-22.

**PROACTIVE: version-citation robustness sweep**

Grepped all .factory/specs/ files for parenthesized version pins that cross-reference other
documents at a specific version (e.g. `(v1.1x)`, `prd.md v1.`, `BC-INDEX v1.`). Findings:
- `BC-INDEX.md line 36`: sole current-state pin found — fixed (D-01 above).
- `prd.md line 113` `BC-INDEX v1.6`: this is a historical changelog note in a version delta
  block, correctly recording the state at a past point-in-time — left unchanged (correct use).
- All other matches were within individual BC body files (inline version annotations on BC
  revision history, e.g. `v1.0`, `v2.0`) or within spec-changelog itself (historical entries).
  None are cross-document current-state pins; all left unchanged.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/domain/domain-debt.md` | `version: 1.2 → 1.3`; O-04 technique count 21→23 (Feature #8 DNP3 additions T1691.001 + T0827) |
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | `version: 1.25 → 1.26`; PRD version pin removed (D-01); double blank line before ss-12 removed (B-01) |
| `.factory/spec-changelog.md` | P22-fixes entry added; architect P22 bumps A-01/A-02 logged |

---

## [pass-21-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-21 ledger hygiene (B-01, D-01, D-02, D-03, D-04)

Five targeted fixes across three artifacts. No behavioral changes; table formatting, historical
path/slug corrections, and version-history sync only.

**B-01 MED — BC-INDEX.md: stray blank line inside ss-11 table deleted (v1.24 → v1.25)**

A blank line between the BC-2.11.001 row and the BC-2.11.002 row split the Markdown table,
causing BC-2.11.002..024 to render without a header. The blank line was removed; the ss-11
table is now one contiguous block.

**D-01 MED — spec-changelog.md Pass-13 ledger: ARCH-INDEX path corrected**

Historical Pass-13 architect-burst row cited `specs/behavioral-contracts/ARCH-INDEX.md`.
Correct path is `specs/architecture/ARCH-INDEX.md`. Corrected to match sibling rows at
changelog lines ~2117 and ~2158 which already used the right directory.

**D-02 MED — spec-changelog.md Pass-13 ledger: vp-005 slug corrected**

Historical Pass-13 architect-burst row cited `specs/verification-properties/vp-005-no-panic-guarantee.md`.
Real file is `vp-005-sni-four-way-classification.md`. Corrected.

**D-03 MED — spec-changelog.md Pass-13 ledger: vp-008 slug corrected**

Historical Pass-13 architect-burst row cited `specs/verification-properties/vp-008-all-analyzers-pure.md`.
Real file is `vp-008-decode-packet-no-panic.md`. Corrected.

**D-04 LOW — prd.md: missing body delta notes for versions 1.13/1.14/1.15/1.16/1.18 added (v1.18 → v1.19)**

The inline version history jumped from 1.12 → 1.17 with no delta notes for versions 1.13, 1.14,
1.15, 1.16, or 1.18 — all real bumps per changelog. Added concise delta notes for each, sourced
from the corresponding spec-changelog entries. Version 1.19 delta note added for this sync fix.
Version history is now contiguous from 1.1 through 1.19.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | `version: 1.24 → 1.25`; stray blank line inside ss-11 table removed |
| `.factory/spec-changelog.md` | Pass-13 architect-burst table: ARCH-INDEX path corrected (D-01), vp-005 slug corrected (D-02), vp-008 slug corrected (D-03) |
| `.factory/specs/prd.md` | `version: 1.18 → 1.19`; body delta notes added for versions 1.13/1.14/1.15/1.16/1.18/1.19 |

---

## [pass-20-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-20 small anchor/version stragglers (D-01, B-01..B-05)

Six targeted fixes across five artifacts. No behavioral changes; line number and version
field corrections only. All verified against current src before edit.

**D-01 HIGH — cap-09-finding-emission.md version field straggler**

| Field | Before | After |
|-------|--------|-------|
| `version:` | `"1.1"` | `"1.2"` |

Root cause: the P19 straggler anchor sweep applied body updates and added a second
`modified[]` entry to cap-09 but never incremented the frontmatter `version:` field.
The changelog and body state were at 1.2; the frontmatter field was left at 1.1.
Exactly one `version:` key present after fix (confirmed).

**B-01 LOW — BC-2.04.012 v1.9→v2.0: Invariant 1 latch line 618→647**

Invariant 1 prose cited `self.finalized = true` at `mod.rs:618`. Actual is `mod.rs:647`
(verified: `grep -n "finalized" src/reassembly/mod.rs` returns `647: self.finalized = true`).
The Refactoring Notes already cited 647 correctly; only the body Invariant sentence was missed
in the P19 sweep.

**B-02 MEDIUM — BC-2.04.013 v1.8→v1.9: expire call-site :166-169 → :168-171 (two occurrences)**

Architecture Module row and Source Evidence row both cited `process_packet` call site at
`mod.rs:166-169`. Actual call site (the `expire_idle_by_timeout` invocation) is at
`mod.rs:168-171` (verified: `sed -n '160,175p'` shows the guard condition at 168-170 and
the call at 170; the block spans 168-171). Architecture Anchors and prose already had 168-171
correct. Fixed both stale occurrences.

**B-03 MEDIUM — BC-2.04.014 v1.5→v1.6: lifecycle.rs:60 → lifecycle.rs:66**

Architecture Module row and Architecture Anchors bullet cited `lifecycle.rs:60` for
`total_memory -= flow_mem on close`. Actual is `lifecycle.rs:66` (verified:
`grep -n "total_memory"` returns `66: self.total_memory -= flow_mem`; line 60 is
`let flushed = flow_dir.flush_contiguous()`).

**B-04 MEDIUM — BC-2.12.005 v1.4→v1.5: main.rs:87-122 → 139-166; Invariant 4 104-117 → 147-161**

- Architecture Anchor `src/main.rs:87-122` (described as "reassembly configuration applied
  in run_analyze") was stale. The `ReassemblyConfig` struct literal is at lines 140-144;
  CLI override `if let Some(v)` blocks run 147-161; `flow_timeout_secs` wire at 165;
  `TcpReassembler::new` at 166. Correct range: `main.rs:139-166`. Fixed.
- Invariant 4 cited `main.rs:104-117` for CLI override application. Actual override block
  is `main.rs:147-161`. Fixed.

**B-05 LOW — BC-2.12.005 (same bump): cli.rs:71-122 → 73-124**

Architecture Anchor and Source Evidence cited `cli.rs:71-122` for the reassembly flag block.
Line 71 is the `--csv` flag tail; the `--reassemble` `#[arg]` annotation starts at line 73.
The block ends with `pub flow_timeout: u64` at line 124. Correct range: `cli.rs:73-124`. Fixed.

**Architect D-02 logged (ADR-008 modified[] — T0830 ICS/Enterprise matrix-label reconciliation)**

Per architect task output for P20 D-02: ADR-008 received a `modified[]` entry for
T0830 ICS/Enterprise matrix-label reconciliation (timing T0830 ICS/Enterprise). No BC
file changes were required for D-02; this entry records the architect's bump for auditability.

**Artifacts changed:**

| Artifact | Change |
|----------|--------|
| `.factory/specs/domain/capabilities/cap-09-finding-emission.md` | `version: 1.1 → 1.2`; third `modified[]` entry added |
| `.factory/specs/behavioral-contracts/ss-04/BC-2.04.012.md` | `version: 1.9 → 2.0`; Invariant 1 latch prose `mod.rs:618 → mod.rs:647` |
| `.factory/specs/behavioral-contracts/ss-04/BC-2.04.013.md` | `version: 1.8 → 1.9`; Architecture Module + Source Evidence call-site `:166-169 → :168-171` (2 occurrences) |
| `.factory/specs/behavioral-contracts/ss-04/BC-2.04.014.md` | `version: 1.5 → 1.6`; Architecture Module + Architecture Anchors `lifecycle.rs:60 → lifecycle.rs:66` |
| `.factory/specs/behavioral-contracts/ss-12/BC-2.12.005.md` | `version: 1.4 → 1.5`; Invariant 4 `main.rs:104-117 → 147-161`; Arch Anchor `main.rs:87-122 → 139-166`; Arch Anchor + Source Evidence `cli.rs:71-122 → 73-124` |
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | Inline version bump annotations added for all four changed BCs |

---

## [pg-arp-f2-007-ss11-full-reanchor-2026-06-13] — 2026-06-13

### PATCH: PG-ARP-F2-007 — comprehensive ss-11 (reporting/output) src-line re-anchor, affected BCs

Root cause: STORY-100 (F2 multi-tag mitre_techniques column) added one column to csv.rs
write_record and added render_finding_grouped / render_finding_flat MITRE expansion logic
to terminal.rs, shifting downstream line numbers. Previous DF-SIBLING-SWEEP-001 pass
(2026-06-01) corrected lines through HEAD cfe0112a but did not account for STORY-100
additions that landed after. This pass re-anchors all stale ss-11 BC citations to verified
current HEAD.

**Clean (no change needed): BC-2.11.001..006, 007, 008, 010, 011, 012, 019, 020, 023**
- json.rs BCs (001-005): no changes to json.rs structure
- terminal.rs anchors already correct for: escape_for_terminal fn (:44), C1 predicate (:52),
  skipped_packets guard (:94), render_finding_prefix (:203-226), analyzer detail loop
  (:165-181), section() fn (:190-195), render full body (:83-186), section ordering (:113/125/138/149/165)
- csv.rs BCs: header write (:62-73), neutralize fn (:40-45), Reporter impl (:51-106) correct

**Changed BCs (10 files):**

| BC | Old anchor(s) | New anchor(s) | Root cause |
|----|---------------|---------------|-----------|
| BC-2.11.009 v1.4→v1.5 | test fns :375/:388, range :367-396 | test fns :544/:556, range :544-565 | test block shifted by ~170 lines (proptest suite additions + new test fns added in STORY-100/F2 passes) |
| BC-2.11.013 v1.7→v1.8 | render_findings_grouped :260-304; tactic loop :290 | :272-323; :309 | render_finding_grouped fn (22 lines) inserted above render_findings_grouped in STORY-100 multi-tag impl |
| BC-2.11.014 v1.5→v1.6 | verdict_rank :269-275; confidence_rank :276-282; sort_by_key :284-288; bucket push line 266 | :287-293; :295-301; :303-307; line 284 | same insertion above render_findings_grouped |
| BC-2.11.015 v1.6→v1.7 | render_finding_grouped :244-252; unknown arm :249; Uncategorized :298-303; Path :244-303 | :247-263; :260; :317-322; :247-323 | render_finding_grouped expanded from simple to multi-tag form; Uncategorized shifted by full grouped fn expansion |
| BC-2.11.016 v1.5→v1.6 | expansion range :246-251; em-dash :248 | :249-261; :259 | render_finding_grouped body rewritten with ids join + first-technique name lookup (multi-tag logic) |
| BC-2.11.017 v1.6→v1.7 | render_finding_flat :230-235 | :232-238 | render_finding_flat fn decl shifted by 2 lines (render_finding_prefix body expanded for multi-tag evidence loop) |
| BC-2.11.018 v1.3→v1.4 | colorization block :209-220 | :209-222 (if-else closes at 222) | block end was off by 2 lines (else branch at 220-221 + closing at 222 not counted) |
| BC-2.11.021 v1.3→v1.4 | neutralize write_record :89-97 | :92-103 | STORY-100 added mitre_techniques column (9th column); write_record data-row block expanded; neutralize calls now at :94-:102 |
| BC-2.11.022 v1.3→v1.4 | evidence neutralize pc4 :93 | :98 | evidence moved from column 5 to column 5 (unchanged position) but csv.rs line shifted from :93 to :98 as mitre column inserted before it |
| BC-2.11.024 v1.7→v1.8 | neutralize optional-derived strings :94-97; pc1 "neutralize at csv.rs:87" | :99-102; clarified join@:87 vs neutralize@:99 | same column-addition shift; mitre neutralize :99, source_ip :100, direction :101, timestamp :102 |

**H1 titles: ALL UNCHANGED** — this pass corrects line anchors only; no semantic content modified.

**BC-INDEX updated:** inline annotations added for all 10 changed BCs.

**Files touched:** BC-2.11.009, BC-2.11.013, BC-2.11.014, BC-2.11.015, BC-2.11.016,
BC-2.11.017, BC-2.11.018, BC-2.11.021, BC-2.11.022, BC-2.11.024, BC-INDEX.md,
spec-changelog.md

| Artifact | Version | Notes |
|----------|---------|-------|
| BC-2.11.009 | 1.4→1.5 | test fn anchors re-anchored |
| BC-2.11.013 | 1.7→1.8 | render_findings_grouped + tactic loop re-anchored |
| BC-2.11.014 | 1.5→1.6 | sort-closure range + bucket push line re-anchored |
| BC-2.11.015 | 1.6→1.7 | render_finding_grouped + Uncategorized bucket re-anchored |
| BC-2.11.016 | 1.5→1.6 | MITRE expansion range + em-dash literal re-anchored |
| BC-2.11.017 | 1.6→1.7 | render_finding_flat range re-anchored |
| BC-2.11.018 | 1.3→1.4 | colorization block end corrected |
| BC-2.11.021 | 1.3→1.4 | neutralize write_record range re-anchored |
| BC-2.11.022 | 1.3→1.4 | evidence neutralize line re-anchored |
| BC-2.11.024 | 1.7→1.8 | optional-derived neutralize range re-anchored; pc1 clarified |
| BC-INDEX.md | — | inline annotations added for 10 changed BCs |
| spec-changelog.md | — | this entry |

---

## [pg-arp-f2-007-ss04-full-reanchor-2026-06-13] — 2026-06-13

### PATCH: PG-ARP-F2-007 — comprehensive ss-04 (TCP reassembly) src-line re-anchor, all BCs

**Root cause:** `src/reassembly/mod.rs`, `segment.rs`, `lifecycle.rs`, and `flow.rs` shifted substantially due to F2 timestamp-wiring refactors (STORY-097/098/099) and prior HS-043 idle-expiry wiring (DF-SIBLING-SWEEP-001, +29 lines at process_packet entry). The DF-SIBLING-SWEEP-001 corrected mod.rs anchors for a subset of BCs; F2 then shifted segment.rs by ~139 lines (insert_segment relocated from ~line 50 to line 189). All remaining ss-04 BCs had stale Architecture Module, Architecture Anchors, Source Evidence Path, and inline prose citations.

**Scope:** BC-2.04.002 through BC-2.04.054. Skipped: BC-2.04.020, BC-2.04.024, BC-2.04.055 (already corrected in Pass-19 B-07/B-09). Clean (no anchor changes needed): BC-2.04.001, BC-2.04.003, BC-2.04.004, BC-2.04.009, BC-2.04.029, BC-2.04.031, BC-2.04.039, BC-2.04.049, BC-2.04.050, BC-2.04.053.

**Key line mappings applied (mod.rs):**
- `process_packet` fn: 144-211 (packets_processed++ at 150, extract_tcp_context at 174, insert_payload_segment call at 191 → 193, FIN removal 198-205, memcap eviction 208-210)
- `extract_tcp_context`: 217-244 (non-TCP skip at 219)
- `get_or_create_flow`: 250-273
- `apply_handshake_flags`: 279-321 (SYN 289-294, SYN+ACK 297-302, RST 305-310, FIN 313-319)
- `insert_payload_segment`: 332-445 (on_data_without_syn block 342-349, small_segment_run update 393-407)
- `check_anomaly_thresholds`: 461-566 (overlap guard 477-499, small-segment guard 506-538, OOW guard 540-565)
- `flush_contiguous_data`: 574-591 (total_memory-= at 585, bytes_reassembled++ at 588, on_data at 589)
- `expire_idle_by_timeout`: 604-619
- `expire_flows` (pub): 622-638
- `finalize`: 643-677 (finalized latch at 644-647, count check at 658, unconditional push at 659-676)
- `plural_s` fn: 68-70
- `summarize`: 735-773
- `impl Drop`: 1038-1052
- ConflictingOverlap match arm: 416-418
- DepthExceeded match arm: 424-426
- `total_memory` tracking: mod.rs:376 (add bytes_added), mod.rs:585 (subtract flush)

**Key line mappings applied (segment.rs):**
- `ranges_overlap` fn (half-open interval test): line 43
- `segment_overlap` fn: 57-84
- `select_gaps` fn: 140-184
- `insert_segment` fn: 189-365
  - empty data return: 197-199
  - ISN check (IsnMissing guard): 201-208 (swap at 204)
  - out-of-window check: 213-217
  - segment limit check (early guard): 220-222
  - remaining_depth check: 229-235
  - truncation logic: 238-253
  - overlap detection loop: 259-284
  - fully_covered / has_conflict returns: 286-303
  - gap insertion loop: 308-332 (mid-loop limit guard at 311-313)
  - !had_gap return arm: 334-344 (actual return at 335)
  - no-overlap insert path: 348-364 (buffered_bytes += at 358)
- `flush_contiguous` fn: 369-381

**Key line mappings applied (lifecycle.rs):**
- `close_flow` fn: 38-68 (let-else 44-52, close_timestamp at 56, flush loop 58-65, bytes_reassembled++ at 62)
- `evict_flows`: 73-98 (sort_by comparator at 85-88)
- `generate_conflicting_overlap_finding`: 105-128 (guard 111-114, push 115-127)
- `generate_truncated_finding`: 135-158 (guard 141-144, push 145-157)

**Files changed:**
- BC-2.04.002, 005, 006, 007, 008, 010, 011, 012, 013, 014, 015, 016, 017, 018, 019, 021, 022, 023, 025, 026, 027, 028, 030 (prior session)
- BC-2.04.029, 032, 033, 034, 035, 036, 037, 038, 040, 041, 042, 043, 044, 045, 046, 047, 048, 051, 052, 054 (this session)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` — PG-ARP-F2-007 annotations added for all changed ss-04 BCs

---

## [pg-arp-f2-007-ss07-full-reanchor-2026-06-13] — 2026-06-13

### PATCH: PG-ARP-F2-007 — comprehensive ss-07 (TLS analyzer) src-line re-anchor, all 37 BCs

**Root cause:** `src/analyzer/tls.rs` (1385 lines) shifted 10-60 lines due to F2 timestamp-wiring refactors (STORY-097/098/099). The Pass-19 B-10 fix only corrected BC-2.07.008/016/037. All remaining 34 BCs had stale Architecture Module, Architecture Anchors, Source Evidence Path, and inline prose citations.

**Scope:** BC-2.07.001 through BC-2.07.037. BC-016 and BC-030 were already clean (no changes); all others updated.

**Key line mappings applied:**
- `handle_client_hello` fn: 379-540 → 389-580
- `handle_server_hello` fn: 542-604 → 586-651
- Deprecated client version: 519-539 → 559-579
- Deprecated server version: 584-604 → 630-650
- Weak cipher client: 497-517 → 530-556
- Weak cipher server: 570-582 → 615-627
- `summarize` fn: 763-808 → 853-897
- `on_flow_close` fn: 752-754 → 841-843
- `on_data` done-check: 718-724 → 807-810
- Non-handshake skip: 678-682 → 718-736
- nom error arms: 700-712 → 783-790
- MAX_RECORD_PAYLOAD guard: 641-653 / 643-653 → 689-699
- `extract_sni` fn: 246/247 (already correct from P19 B-10)
- SNI match block: 251-265 → 252-266 (arm 1: 253; arm 3: 258-261; arm 4: 262-265)
- AsciiWithControl emission: 424-447 / 426 → 437-459 / 437
- NonAsciiUtf8 emission: 449-467 → 461-492
- NonUtf8 emission: 469-488 → 494-514
- Key selection block: 402-416 / 410-415 → 421-427
- `TlsAnalyzer::increment`: 372-376 → 379-383
- `compute_ja3` fn: 92-151 → 93-152 (already correct from P19 B-10)
- `compute_ja3s` fn: 153-173 → 154-174 (already correct from P19 B-10)
- GREASE filter sub-regions: 50-52 → 51-53; 100-143 → 102-144 (already correct from P19 B-10)
- `cipher_name` fn: 77-83 → 79-84
- version/increment lines: 386-387 → 397-398
- `is_weak_cipher`: 56-64 → 57-65 (already correct)
- `is_weak_server_cipher`: 66-75 → 68-76 (already correct)

**Files changed:**
- BC-2.07.001 through BC-2.07.037 (35 files updated; 016, 030 unchanged)
- `.factory/specs/behavioral-contracts/BC-INDEX.md` — PG-ARP-F2-007 annotation added

---

## [pass-19-straggler-domain-reanchor-2026-06-13] — 2026-06-13

### PATCH: P19 straggler — comprehensive domain + prd-supplement src-line anchor sweep (PG-ARP-F2-007)

**Summary:** Completes the anchor-drift remediation left over after Pass 19. F2 feature cycles (Modbus STORY-105, DNP3 STORY-110, timestamp wiring STORY-097/098/099, multi-tag mitre_techniques STORY-100) shifted source line numbers throughout the codebase, leaving capability docs, entity docs, invariants, and prd-supplement docs with stale anchors. This sweep corrects all identified stale citations. No src files touched. No BC subsystem files touched. No VP files touched (architect already completed VP-sweep in the preceding burst). No commit issued in this step.

---

**Architect VP-sweep bumps (logged here per PO changelog ownership):**

Applied in the preceding burst by architect agent. VP version bumps from VP-sweep (src-line anchor correction across VP files):

| File | Before | After |
|------|--------|-------|
| vp-003 | v2.0 | v2.1 |
| vp-004 | v2.1 | v2.2 |
| vp-006 | v2.0 | v2.1 |
| vp-010 | v2.0 | v2.1 |
| vp-011 | v2.0 | v2.1 |
| vp-013 | v2.0 | v2.1 |
| vp-014 | v2.0 | v2.1 |
| vp-015 | v2.0 | v2.1 |
| vp-021 | v2.0 | v2.1 |

Note: purity-boundary-map v1.4→v1.5 (P19 A-01/A-02) was already logged in `[pass-19-c-fixes-2026-06-13]` — not re-logged here.

---

**cap-05-content-first-dispatch.md (applied in prior burst, logged here):**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| None-cache block in `on_data` | dispatcher.rs:137-148 | dispatcher.rs:269-290 |

| File | Version |
|------|---------|
| cap-05-content-first-dispatch.md | — → 1.1 |

---

**cap-06-http-analysis.md:**

All 10 anomaly detection table source-line anchors updated. UA rationale prose anchor updated.

| Detection | Old | New |
|-----------|-----|-----|
| Path traversal | http.rs:187-203 | http.rs:200-218 |
| Web shell | http.rs:218-233 | http.rs:221-248 |
| Admin panel | http.rs:237-249 | http.rs:250-264 |
| Unusual method | http.rs:253-265 | http.rs:266-280 |
| Missing/Empty Host | http.rs:283-301 | http.rs:282-317 |
| Abnormally long URI | http.rs:305-317 | http.rs:319-332 |
| Empty UA | http.rs:344-356 | http.rs:359-371 |
| Too-many-headers (request) | http.rs:416-428 | http.rs:435-449 |
| Too-many-headers (response) | http.rs:475-487 | http.rs:496-509 |
| UA rationale prose | http.rs:319-343 | http.rs:334-358 |

| File | Version |
|------|---------|
| cap-06-http-analysis.md | — → 1.1 |

---

**cap-07-tls-analysis.md:**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| TlsFlowState::done() | tls.rs:291-293 | tls.rs:298-300 |
| Early-exit guard in on_data | tls.rs:721-724 | tls.rs:807-810 |
| truncated_records field | tls.rs:312 | tls.rs:319 |
| truncated_records += 1 | tls.rs:645 | tls.rs:691 |
| summarize() truncated_records insertion | tls.rs:798-801 | tls.rs:887-890 |
| SNI AsciiWithControl finding block | tls.rs:426-448 | tls.rs:437-460 |
| SNI NonAsciiUtf8 finding block | tls.rs:449-468 | tls.rs:461-493 |
| SNI NonUtf8 finding block | tls.rs:469-489 | tls.rs:494-514 |
| Weak ClientHello ciphers block | tls.rs:504-517 | tls.rs:542-556 |
| Deprecated ClientHello version block | tls.rs:526-539 | tls.rs:559-579 |
| Weak ServerHello cipher block | tls.rs:571-582 | tls.rs:614-627 |
| Deprecated ServerHello version block | tls.rs:591-604 | tls.rs:630-650 |
| O-06 weak-cipher evidence note | tls.rs:504-517 | tls.rs:542-556 |

| File | Version |
|------|---------|
| cap-07-tls-analysis.md | — → 1.1 |

---

**cap-09-finding-emission.md:**

All 22 emission site line numbers updated (9 http.rs, 7 tls.rs, 4 mod.rs, 2 lifecycle.rs). Notable-properties refs also updated.

| File | Version |
|------|---------|
| cap-09-finding-emission.md | 1.1 → 1.2 (second modified entry added) |

---

**ent-03-dispatch-analysis.md:**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| E-21 StreamDispatcher struct | dispatcher.rs:42-54 | dispatcher.rs:60-78 |
| E-22 DispatchTarget enum | dispatcher.rs:23-28 | dispatcher.rs:38-46 |
| E-22 None-cache prose | dispatcher.rs:137-148 | dispatcher.rs:269-290 |
| E-31 HttpAnalyzer struct | http.rs:114 | http.rs:122 |
| E-32 HttpFlowState struct | http.rs:82 | http.rs:84 |
| E-33 TlsAnalyzer struct | tls.rs:298 | tls.rs:305 |
| E-34 TlsFlowState struct | tls.rs:273 | tls.rs:274 |
| E-35 SniValue enum | tls.rs:200 | tls.rs:201 |
| E-40 UA rationale prose | http.rs:319-343 | http.rs:334-358 |

E-22 enum body updated to include Modbus and Dnp3 variants (per F2 extension).

| File | Version |
|------|---------|
| ent-03-dispatch-analysis.md | — → 1.1 |

---

**ent-04-findings-output.md:**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| E-23 Verdict enum | findings.rs:30-40 | findings.rs:32-46 |
| E-24 Confidence enum | findings.rs:57-66 | findings.rs:66-73 |
| E-27 MitreTactic enum | mitre.rs:45-66 | mitre.rs:47-70 |

| File | Version |
|------|---------|
| ent-04-findings-output.md | 1.2 → 1.3 |

---

**inv-01-core-invariants.md:**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| INV-2 HTTP/ arm inline cite | dispatcher.rs:104 | dispatcher.rs:199 |
| INV-5 SniValue enum | tls.rs:200 | tls.rs:201 |
| INV-5 extract_sni match block | tls.rs:251-265 | tls.rs:252-266 |
| INV-6 MAX_FINDINGS const | mod.rs:54 | mod.rs:56 |
| INV-6 guard sites | mod.rs:461,495,524 + lifecycle.rs:101,121 | mod.rs:479,515,546 + lifecycle.rs:111,141 |
| INV-8 request poison transition | http.rs:408-409 | http.rs:427-428 |
| INV-8 response poison transition | http.rs:467-468 | http.rs:488-489 |

| File | Version |
|------|---------|
| inv-01-core-invariants.md | 1.3 → 1.4 |

---

**nfr-catalog.md:**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| NFR-RES-011 MAX_HEADER_BUF const | http.rs:21 | http.rs:23 |
| NFR-RES-011 buffer cap sites | http.rs:513, 525 | http.rs:546, 558 |
| NFR-RES-012 MAX_HEADERS const | http.rs:22 | http.rs:24 |
| NFR-RES-012 TooManyHeaders request | http.rs:416-428 | http.rs:435-449 |
| NFR-RES-012 TooManyHeaders response | http.rs:475-487 | http.rs:496-509 |
| NFR-RES-013 MAX_URIS const | http.rs:23 | http.rs:25 |
| NFR-RES-013 URI cap guard | http.rs:391-393 | http.rs:406 |
| NFR-RES-014 MAX_MAP_ENTRIES (http) | http.rs:24 | http.rs:26 |
| NFR-RES-014 MAX_MAP_ENTRIES (tls) | tls.rs:30 | tls.rs:31 |
| NFR-RES-015 MAX_BUF const | tls.rs:29 | tls.rs:30 |
| NFR-RES-015 buffer cap sites | tls.rs:761, 768 | tls.rs:822, 829 |
| NFR-RES-016 MAX_RECORD_PAYLOAD const | tls.rs:31-33 | tls.rs:32-34 |
| NFR-RES-016 oversized record guard | tls.rs:643-653 | tls.rs:689 |
| NFR-RES-017 POISON_THRESHOLD const | http.rs:80 | http.rs:82 |

| File | Version |
|------|---------|
| nfr-catalog.md | 2.0 → 2.1 |

---

**error-taxonomy.md:**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| E-ANA-001 parse_errors increment | http.rs:405, 463 | http.rs:424, 484 |
| E-ANA-002 request poison block | http.rs:406-415 | http.rs:424-434 |
| E-ANA-002 response poison block | http.rs:464-473 | http.rs:484-494 |
| E-ANA-003 oversized record path | tls.rs:643-653 | tls.rs:689-699 |
| E-ANA-006 per-map cap guard (http) | http.rs:375-389 | http.rs:390-394 |
| E-ANA-007 increment helper | tls.rs:372-375 | tls.rs:379-384 |
| E-ANA-007 call sites | tls.rs:387,416,494,549,564,568 | tls.rs:398,427,520,593,608,612 |
| E-ANA-008 URI cap guard | http.rs:391-392 | http.rs:406 |
| E-RAS-003 mod.rs guard sites | mod.rs:461,495,524 | mod.rs:479,515,546 |
| E-RAS-003 lifecycle guard sites | lifecycle.rs:101,121 | lifecycle.rs:111,141 |

| File | Version |
|------|---------|
| error-taxonomy.md | 2.0 → 2.1 |

---

**test-vectors.md:**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| BC-2.06.005 path-traversal push | http.rs:193 | http.rs:205 |
| BC-2.07.014 AsciiWithControl block | tls.rs:426-448 | tls.rs:437-460 |
| BC-2.07.017 NonAsciiUtf8 block | tls.rs:449-468 | tls.rs:461-493 |
| BC-2.07.037 extract_sni match | tls.rs:251-258 | tls.rs:252-266 |

| File | Version |
|------|---------|
| test-vectors.md | 2.0 → 2.1 |

---

## [pass-19-c-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-19 C-01/C-02/C-03 — MITRE fact, dispatcher.rs anchor drift (prd-supplements + inv-01), HS-009 tactic fix; Architect P19 A-01/A-02 bump

**Summary:** Remediates three Pass-19 findings (C-01 HIGH MITRE fact error in HS-009; C-02 HIGH dispatcher.rs anchor drift in nfr-catalog and nfr-story-map; C-03 MED dispatcher.rs anchor drift in inv-01). Also logs the architect's P19 A-01/A-02 bump to purity-boundary-map.md (v1.4→v1.5), which was applied in the preceding burst. No src files touched. No BCs touched. No stories touched.

---

**C-01 (HIGH) — HS-009: T1083 tactic "Reconnaissance" → "Discovery"**

Root cause: Step 3 of HS-009 stated "T1083 -> Reconnaissance". This is factually wrong.
`src/mitre.rs:141` seeded T1083 as `("File and Directory Discovery", MitreTactic::Discovery)`.
Discovery is the correct parent tactic per MITRE ATT&CK Enterprise. The HTTP analyzer uses
"Reconnaissance" as a ThreatCategory label (a different axis); that had no bearing on the
technique_tactic() return value for T1083.

| File | Before | After |
|------|--------|-------|
| HS-009 line 49 | `T1083 -> Reconnaissance` | `T1083 -> Discovery` |
| HS-009 version | 1.2 | 1.3 |

Verified: `src/mitre.rs:141` → `"T1083" => ("File and Directory Discovery", MitreTactic::Discovery)`.

---

**C-02 (HIGH) — nfr-catalog.md + nfr-story-map.md: dispatcher.rs anchor drift**

Root cause: The Modbus (STORY-105) and DNP3 (STORY-110) additions extended
`src/dispatcher.rs` significantly, shifting the struct field positions and the
`on_data` cache-lookup block down from where they were when the NFR anchors were first written.

**NFR-PERF-003 (nfr-catalog.md):**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| `routes: HashMap<FlowKey, DispatchTarget>` struct field | dispatcher.rs:43 | dispatcher.rs:61 |
| cache lookup block (`let target = if let Some(&cached)`) | dispatcher.rs:133-154 | dispatcher.rs:269-290 |

**NFR-OBS-005 (nfr-catalog.md):**

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| `unclassified_flows: u64` struct field | dispatcher.rs:53 | dispatcher.rs:77 |
| accessor fn `unclassified_flows()` | dispatcher.rs:80-81 | dispatcher.rs:117-119 |
| `self.unclassified_flows += 1` (on_flow_close None arm) | dispatcher.rs:188-191 | dispatcher.rs:357 |

**NFR-OBS-005 (nfr-story-map.md line 79):**

| Old text | New text |
|---------|---------|
| `dispatcher.rs:53` | `dispatcher.rs:77` |

| File | Version |
|------|---------|
| nfr-catalog.md | 1.9 → 2.0 |
| nfr-story-map.md | 1.2 → 1.3 |

---

**C-03 (MED) — inv-01-core-invariants.md INV-2: dispatcher.rs anchor drift**

Root cause: Same Modbus/DNP3 extension as C-02. INV-2 enforcement anchors for the `classify`
function and the `on_data` cache-lookup block were stale.

| Cited symbol | Old anchor | Actual anchor (verified) |
|-------------|-----------|------------------------|
| `fn classify` | dispatcher.rs:90-117 | dispatcher.rs:184-242 |
| cache-lookup/retry-budget block in `on_data` | dispatcher.rs:133-154 ("block starts at line 133") | dispatcher.rs:269-290 ("block starts at line 269") |

| File | Version |
|------|---------|
| inv-01-core-invariants.md | 1.2 → 1.3 |

Confirmed: inv-01 has exactly ONE `version:` key post-edit (no duplicate introduced).
INV-9 mitre.rs anchors (v1.1/v1.2 corrections) left unchanged — confirmed still current.

---

**Architect P19 A-01/A-02 bump — purity-boundary-map.md v1.4 → v1.5**

Applied in the preceding burst by architect agent. Logged here per changelog ownership.

| Finding | Change |
|---------|--------|
| A-01 (MED) | VP-024 sub-letter corrected C→D; Sub-C clause language added to clarify the read-only observer contract |
| A-02 (LOW) | None-caching anchor updated: dispatcher.rs:146-148→:279-282 (verified — the `self.routes.insert(flow_key.clone(), DispatchTarget::None)` site in on_data's None branch) |

---

## [pass-19-ss04-ss07-reanchor-2026-06-13] — 2026-06-13

### PATCH: Pass-19 B-07/B-09/B-10 — ss-04 and ss-07 stale src anchor remediation (6 BCs)

**Summary:** Remediates Pass-19 findings B-07 (BC-2.04.055 sibling stale anchors), B-09 (ss-04
mod.rs anchors shifted by F2 timestamp wiring), and B-10 (ss-07 tls.rs off-by-one anchors).
All line numbers verified against current source before editing.

**Root cause:** The F2 timestamp wiring burst (STORY-097/098/099) inserted new state fields
and call-site code into `src/reassembly/mod.rs` (~18 additional lines, shifting mod.rs content
downward), `src/analyzer/http.rs` (new `last_ts` field and assignment), and `src/analyzer/tls.rs`
(new `last_ts` field and assignment). A separate minor edit to tls.rs produced 1-line off-by-ones
in the SNI classification functions.

**Changes applied:**

| BC | Finding | Change |
|----|---------|--------|
| BC-2.04.055 v1.0.2→v1.0.3 | B-07 | `http.rs:501→:524` (HttpAnalyzer::on_data); `tls.rs:771→:798` (TlsAnalyzer::on_data) |
| BC-2.04.024 v1.3→v1.4 | B-09 | `mod.rs:54→:56` (MAX_FINDINGS const); guard sites `mod.rs:461,495,524→:479,515,546` |
| BC-2.04.020 v1.5→v1.6 | B-09 | small-segment block `mod.rs:486-517→:506-538`; counter maintenance `mod.rs:385-399→:402-405` |
| BC-2.07.037 v1.2→v1.3 | B-10 | `fn extract_sni tls.rs:246→:247`; match block `tls.rs:251-265→:252-269`; module range `:200-265→:200-269` |
| BC-2.07.016 v1.2→v1.3 | B-10 | `contains_c0_or_del tls.rs:231-238→:232-239` |
| BC-2.07.008 v1.3→v1.4 | B-10 | format string `tls.rs:171→:172`; `Md5::digest tls.rs:172→:173` |

**H1 titles:** Unchanged on all 6 BCs.

**Scope:** BC files + BC-INDEX.md inline annotations only. No src, stories, or other subsystems touched.

---

## [pass-19-ss06-reanchor-2026-06-13] — 2026-06-13

### PATCH: Pass-19 B-08 — ss-06 (http.rs) systematic line anchor re-sync across all 26 BCs

**Summary:** Remediates Pass-19 finding B-08. The F2 timestamp wiring (STORY-097/098/099)
and subsequent refactors extended `src/analyzer/http.rs` to 1044 lines, shifting every
line number cited by the 26 ss-06 behavioral contracts. This entry records the full
authoritative line-map correction applied to all BCs in BC-2.06.001 through BC-2.06.026.

**Root cause:** The F2 timestamp wiring burst inserted ~70+ lines into http.rs (new `last_ts`
field in HttpFlowState, `state.last_ts = timestamp` in `on_data`, associated struct fields),
shifting all prior cited line numbers. Every ss-06 BC carrying an `http.rs:NNN` anchor was
stale against the current 1044-line file.

**Authoritative current line map (verified against src/analyzer/http.rs at 1044 lines):**

| Symbol | Old anchor | New anchor |
|--------|-----------|------------|
| `MAX_HEADER_BUF` const | `:21` | `:23` |
| `MAX_HEADERS` const | `:22` | `:24` |
| `MAX_URIS` const | `:23` | `:25` |
| `MAX_MAP_ENTRIES` const | `:24` | `:26` |
| `POISON_THRESHOLD` const | `:80` | `:82` |
| `find_header` fn | `:70-75` | `:72-77` |
| `HttpFlowState` struct | `:84` | `:84` (unchanged) |
| `counted_as_non_http` field | `:89` | `:91` |
| `HttpAnalyzer` struct | `:114-126` | `:122-134` |
| `check_request_detections` fn | `:183-357` | `:191-372` |
| path traversal contains() calls | `:187-190` | `:200-203` |
| path traversal finding push block | `:192-203` | `:205-218` |
| web shell shell_patterns array | `:206-217` | `:221-232` |
| web shell guard + finding push | `:218-233` | `:233-248` |
| admin panel admin_patterns + block | `:235-249` | `:250-264` |
| unusual methods block | `:251-265` | `:266-280` |
| host anomaly block (full incl. RFC comment) | `:283-302` | `:282-317` |
| long URI block | `:304-317` | `:319-332` |
| Kheir rationale comments | `:319-343` | `:334-358` |
| empty UA detection | `:344-356` | `:359-371` |
| `try_parse_requests` fn | `:359-438` | `:374-459` |
| `had_success` local var decl (req) | `:364` | `:379` |
| `!had_success` guard (req) | `:403-408` | `:422-427` |
| error_count increment + threshold block | `:406-414` | `:425-434` |
| POISON_THRESHOLD check (req) | `:408-409` | `:427-428` |
| `counted_as_non_http` latch | `:410-413` | `:429-432` |
| TooManyHeaders request finding | `:416-428` | `:435-449` |
| `try_parse_responses` fn | `:440-497` | `:461-520` |
| `had_success` local var decl (resp) | `:441` | `:462` |
| transactions + status_codes lines | `:450-452` | `:471-474` |
| `!had_success` guard (resp) | `:462` | `:483` |
| POISON_THRESHOLD check (resp) | `:467-468` | `:488-489` |
| TooManyHeaders response finding | `:475-487` | `:496-509` |
| `on_data` fn | `:506-540` (approx) | `:524-571` |
| request_poisoned gate | `:509-512` | `:542-545` |
| request buf cap block | (part of on_data) | `:546-551` |
| response_poisoned gate | `:521-524` | `:554-557` |
| response buf cap + full buffer cap | `:513-529` | `:532-565` |
| `on_flow_close` fn | `:540-542` | `:573-575` |
| `summarize()` fn | `:550-601` | `:583-634` |
| top_hosts sort | `:571-573` | `:604-606` |
| `findings()` fn | (approx `:610`) | `:636-638` |
| map entry guards (MAX_MAP_ENTRIES) | `:375-389` | `:390-408` |
| uris push guard (MAX_URIS) | `:391-393` | `:406-408` |

**Per-BC anchor corrections and version bumps:**

| BC | Fields corrected | Old version | New version |
|----|-----------------|-------------|-------------|
| BC-2.06.001 | Architecture Module, Architecture Anchors, Source Evidence Path; find_header :70-75→:72-77; try_parse_requests :359-438→:374-459; parse_one_request fn range | v1.2 | v1.3 |
| BC-2.06.002 | had_success decl :364→:379; !had_success guard :404→:423; resp analog :462→:483; try_parse_requests :359-438→:374-459 | v1.4 | v1.5 |
| BC-2.06.003 | Partial request return :402→:421; Partial response return :460→:481 | v1.3 | v1.4 |
| BC-2.06.004 | status_codes :452→:473; resp had_success decl :441→:462; resp guard :462→:483; try_parse_responses :440-497→:461-520; transactions :450-452→:471-474 | v1.8 | v1.9 |
| BC-2.06.005 | path traversal contains() :187-190→:200-203; opening brace :191→:204; finding push :192-203→:205-218; arch module/path :186-203→:200-218 | v1.8 | v1.9 |
| BC-2.06.006 | shell_patterns :206-217→:221-232; guard :218→:233; finding push :219-232→:234-248; overall :206-233→:220-248 | v1.5 | v1.6 |
| BC-2.06.007 | admin_patterns :236→:251; guard :237→:252; finding push :238-248→:253-264; EC-005 inline cite; overall :235-249→:250-264 | v1.6 | v1.7 |
| BC-2.06.008 | unusual methods block :251-265→:266-280 | v1.4 | v1.5 |
| BC-2.06.009 | host anomaly block :283-302→:282-317 (expanded to include RFC comment lines) | v1.4 | v1.5 |
| BC-2.06.010 | long URI block :304-317→:319-332 | v1.4 | v1.5 |
| BC-2.06.011 | empty UA :344-356→:359-371; Kheir comments :319-343→:334-358 | v1.4 | v1.5 |
| BC-2.06.012 | check_request_detections :183-357→:191-372 | v1.2 | v1.3 |
| BC-2.06.013 | had_success :364/:403-408→:379/:422-427; Err arm :403-434→:422-459 | v1.2 | v1.3 |
| BC-2.06.014 | TooManyHeaders req :416-428→:435-449; resp :475-487→:496-509 | v1.3 | v1.4 |
| BC-2.06.015 | POISON_THRESHOLD :80→:82; req poison :408-409→:427-428; resp poison :467-468→:488-489 | v1.3 | v1.4 |
| BC-2.06.016 | error_count increment+threshold block :406-414→:425-434 | v1.2 | v1.3 |
| BC-2.06.017 | req_poisoned gate :509-512→:542-545; resp_poisoned gate :521-524→:554-557; arch module/path :509-523→:542-556 | v1.3 | v1.4 |
| BC-2.06.018 | counted_as_non_http latch :410-413→:429-432; field decl :89→:91 | v1.2 | v1.3 |
| BC-2.06.019 | on_flow_close :540-542→:573-575 | v1.2 | v1.3 |
| BC-2.06.020 | had_success decl :362-364→:379-380; guard :403-408→:422-427; req :362-408→:374-427; resp :441-462→:462-483 | v1.4 | v1.5 |
| BC-2.06.021 | flows HashMap+HttpAnalyzer struct :114-126→:122-134 | v1.2 | v1.3 |
| BC-2.06.022 | MAX_HEADER_BUF :21→:23; buffer cap in on_data :513-529→:532-565 | v1.2 | v1.3 |
| BC-2.06.023 | summarize() :550-601→:583-634; top_hosts sort :571-573→:604-606 | v1.4 | v1.5 |
| BC-2.06.024 | MAX_MAP_ENTRIES :24→:26; map entry guard :375-378→:390-392; overall :375-389→:390-408 | v1.2 | v1.3 |
| BC-2.06.025 | MAX_URIS :23→:25; uris push guard :391-393→:406-408 | v1.2 | v1.3 |
| BC-2.06.026 | find_header :70-75→:72-77 | v1.3 | v1.4 |

**H1 titles:** Unchanged on all 26 BCs (anchor-only edits).

**BC-INDEX:** ss-06 inline annotations updated for all 26 BCs with new version numbers.

---

## [pass-19-ss09-reanchor-2026-06-13] — 2026-06-13

### PATCH: Pass-19 B-01..B-06 — ss-09 stale line anchors + Possible variant content gap

**Summary:** Remediates Pass-19 findings B-01 through B-06 against the seven ss-09 BCs
(BC-2.09.001..007). The root cause is the STORY-100 (multi-tag mitre_techniques) comment block
inserted at findings.rs:142-147, which shifted all subsequent code down by 6+ lines. Pass-18
re-anchors (DF-SIBLING-SWEEP-001 in v1.4/v1.6) used the cfe0112a HEAD but were themselves
stale for this file relative to the current HEAD.

**Anchor fixes applied (all verified against current src/findings.rs):**

| BC | Finding | Old anchor | New anchor |
|----|---------|------------|------------|
| BC-2.09.001 | B-01 | `findings.rs:119-146` (struct Finding) | `findings.rs:135-162` |
| BC-2.09.002 | B-02 | `findings.rs:159-170` (Display for Finding) | `findings.rs:173-184` |
| BC-2.09.003 | B-03 (anchor) | `findings.rs:43-50` (Display for Verdict) | `findings.rs:48-57` |
| BC-2.09.004 | B-04 | `findings.rs:68-76` (Display for Confidence) | `findings.rs:75-83` |
| BC-2.09.005 | B-05 | struct `:120` → `:135`; fields `:124-125` → `:140-141`; doc-comment `:150-158` → `:164-172`; cited line `:157` → `:171`; struct Source Evidence `:120-145` → `:135-162`; terminal.rs call site 3 `:223` → `:224`. Invariant 1 call-site list corrected from `172, 197, 216` to `179, 204, 224`. |
| BC-2.09.006 | — | No line-specific anchors — confirmed clean. No changes. |
| BC-2.09.007 | B-06 | `findings.rs:119-146` (Finding struct in Architecture Anchors) | `findings.rs:135-162` |

**Content gap B-03 resolved — Verdict::Possible variant:**

`src/findings.rs:45` confirms `Possible` is a current variant of the `Verdict` enum (added
STORY-109). It renders as "POSSIBLE" (`findings.rs:54`). BC-2.09.003 previously listed only
three variants (Likely/Unlikely/Inconclusive) in Description, Postconditions, Invariants, Edge
Cases, and Canonical Test Vectors — a content gap that made the BC falsifiable against current
source. Additions made:
- Description: added `Possible => "POSSIBLE"` and STORY-109 provenance.
- Postcondition 4: `Verdict::Possible` displays as "POSSIBLE".
- Postcondition 5: renumbered "No other strings" to reflect four current variants.
- Invariant 2: updated to list all four variants; Invariant 4 added (Possible rank + terminal sort).
- EC-004: `Verdict::Possible` → "POSSIBLE" (with STORY-109 usage context).
- Canonical Test Vectors: row added for `format!("{}", Verdict::Possible)` → "POSSIBLE".
- Architecture Anchors: added `enum Verdict` anchor at `findings.rs:32-46`; Display anchor updated to `:48-57`.

BC-2.09.002 Postcondition 3 also updated to include "POSSIBLE" in the verdict token list.

**Version bumps:**
- BC-2.09.001: v1.5 → v1.6
- BC-2.09.002: v1.4 → v1.5
- BC-2.09.003: v1.2 → v1.3
- BC-2.09.004: v1.2 → v1.3
- BC-2.09.005: v1.6 → v1.7
- BC-2.09.006: no change (clean)
- BC-2.09.007: v1.1.1 → v1.1.2

**H1 titles:** Unchanged on all seven BCs.

**BC-INDEX:** ss-09 inline annotations updated for all changed BCs.

---

## [pass-18-c-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-18 C-01/D-01/C-02 + carry-over anchor + pending architect-A and proactive version-bump log

**Summary:** Remediates two Pass-18 findings (C-01/D-01 STORY-INDEX 48/49 split ambiguity; C-02
stale self-referential line anchors in cap-10 changelog entries), one carry-over anchor finding
from the ss-05 re-anchor burst (BC-2.04.055 dispatcher.rs:144→:245), and logs all pending
version bumps from architect-A (Pass-18 A-01/A-02/A-03) and proactive pre-Pass-18 story-anchor
bumps that were not yet recorded.

No story bodies touched. No ss-05 BCs touched (covered by pass-18-b entry).

---

**C-01/D-01 (MED/LOW) — STORY-INDEX.md line 20: 48 vs 49 split clarifier**

Root cause: line 20 said "(49 stories)" but line 21 immediately below said "All 48 greenfield
stories", creating an adjacent contradiction resolvable only by reading line 217.

| Location | Before | After |
|----------|--------|-------|
| STORY-INDEX.md line 20 (lede parenthetical) | "(49 stories)" | "(48 greenfield product + 1 tooling STORY-091 = 49 stories)" |

Arithmetic verified from line 217 Coverage Verification section:
`63 total = 48 greenfield product + 1 tooling STORY-091 + 3 F3 (STORY-097/098/099) + 6 E-13/E-14 (STORY-100..105) + 5 E-15 (STORY-106..110)`.
The v0.1.0-greenfield-spec cycle block is 48 + 1 = 49; "48 greenfield" on line 21 remains
correct and is now consistent with the explicit split on line 20.

| Artifact | Version change |
|----------|---------------|
| STORY-INDEX.md | prose patch only (no version field) |

---

**C-02 (LOW) — cap-10-mitre-mapping.md: stale self-referential line anchors in Pass-9/10/11 changelog entries**

Root cause: Pass-14 (+ARP-F2 CLI --mitre prose fix) added ~6 lines to cap-10, shifting the
`## MitreTactic enum (E-27)` header from line 81 to line 87 and ICS-unique variant prose from
lines 83-85 to lines 89-91. The Pass-9/10/11 changelog entries cited lines 81-85 and remained
stale.

Verified from reading the file: header is at line 87, ICS-unique prose at lines 89-91.

| Changelog entry | Before | After |
|-----------------|--------|-------|
| Pass-9 entry | "lines 81-85" | "lines 87-91, corrected by Pass-18 C-02" |
| Pass-10 entry | "lines 80-82 (subsequently corrected to 'lines 81-85')" | trailing note added: "and to 'lines 87-91' by Pass-18 C-02" |
| Pass-11 entry | "header is at line 81 and the variant prose spans lines 83-85" | note added that header shifted to line 87 after Pass-14 |

| Artifact | Version change |
|----------|---------------|
| cap-10-mitre-mapping.md | v1.8 → v1.9 |

---

**Carry-over anchor — BC-2.04.055 dispatcher.rs:144→:245**

Found during ss-05 re-anchor burst: BC-2.04.055 Architecture Anchors cited
`src/dispatcher.rs:144 — StreamDispatcher::on_data`; the `fn on_data` opening brace now sits
at line 245 (verified by `grep -n "fn on_data" src/dispatcher.rs`). Root cause: Feature #7
(Modbus) and Feature #8 (DNP3) added new struct fields, accessor methods, dispatch arms, and
an expanded early-exit guard — shifting content ~101 lines below the original 144.

| Location | Before | After |
|----------|--------|-------|
| BC-2.04.055 Architecture Anchors | `src/dispatcher.rs:144` | `src/dispatcher.rs:245` |

No other dispatcher.rs anchors appear in BC-2.04.055.

| Artifact | Version change |
|----------|---------------|
| BC-2.04.055 | v1.0.1 → v1.0.2 |
| BC-INDEX.md | inline annotation added to BC-2.04.055 row |

---

**Pending version bumps — architect-A (Pass-18 A-01/A-02/A-03)**

These bumps were applied by the architect agent in the Pass-18 A burst but not yet recorded
in spec-changelog.md:

| Artifact | Before | After | Reason |
|----------|--------|-------|--------|
| `architecture/dependency-graph.md` | v1.5 | v1.6 | A-01: indicatif 0.17→0.18 dependency updated |
| `architecture/verification-coverage-matrix.md` | v1.3 | v1.4 | A-02: VP-023 lock note added |
| `architecture/purity-boundary-map.md` | v1.3 | v1.4 | A-03: VP-024 bullet added |

---

**Pending version bumps — proactive pre-Pass-18 story-anchor bumps (not previously logged)**

These bumps were applied during the arp.rs STORY-111→112 re-anchor sweep before Pass-18 was
submitted, but were not captured in the spec-changelog:

| Artifact | Before | After | Reason |
|----------|--------|-------|--------|
| `architecture/system-overview.md` | v1.3 | v1.4 | arp.rs STORY-111→112 re-anchor (story reference updated) |
| `architecture/purity-boundary-map.md` | v1.2 | v1.3 | arp.rs STORY-111→112 re-anchor (story reference updated) |

Note: purity-boundary-map.md full chain in this cycle: v1.2 → v1.3 (proactive arp.rs re-anchor)
→ v1.4 (A-03 VP-024 bullet, Pass-18 architect-A). Both increments are now recorded.

---

## [pass-18-b-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-18 B-01/B-02/B-03 — ss-05 dispatcher.rs source-line anchor re-sync + four-analyzer guard prose (all 9 BCs)

**Summary:** Remediates Pass-18 findings B-01 (systematic stale line anchors in ss-05 BCs),
B-02 (same root cause — last anchor sync was v1.3 pre-ICS, before Modbus Rule 5 + DNP3 Rule 6
insertions + new accessor methods shifted `src/dispatcher.rs` by ~94-235 lines), and B-03
(BC-2.05.007 and BC-2.05.008 prose described the unconfigured-dispatcher guard as a
two-analyzer check `http/tls`; shipped code checks all four analyzers `http/tls/modbus/dnp3`).

No H1 titles changed. No story bodies touched. No other subsystems' BCs touched.

**Verified current `src/dispatcher.rs` line map (built from reading the actual file):**

| Item | Stale (pre-ICS) | Current |
|------|----------------|---------|
| `DEFAULT_MAX_CLASSIFICATION_ATTEMPTS = 8` | `:40` | `:58` |
| `fn classify(...)` | `:90` | `:184` |
| TLS check (`data[0]==0x16 && data[1]==0x03`) | `:92-94` | `:186-187` |
| HTTP method prefix block | `:95-107` | `:190-202` |
| Port fallback (Rules 3+4: 443/8443/80/8080) | `:108-116` | `:204-212` |
| `DispatchTarget::None` return | `:116` | `:241` |
| `classify()` call in `on_data` | `:136` | `:272` |
| None branch (count increment + cap logic) | `:137-148` | `:273-284` |
| `routes.insert(None)` permanent | `:146` | `:282` |
| `classification_attempts.remove` (None branch) | `:147` | `:283` |
| Non-None branch: `routes.insert` + `remove` | `:149-151` | `:286-287` |
| Early-return guard in `on_data` | `:121-123` | `:256-259` |
| `fn on_flow_close` full range | `:171-194` | `:322-361` |
| `classification_attempts.remove` + `routes.remove` | `:175-176` | `:326-327` |
| Unclassified guard in `on_flow_close` | `:188-191` | `:352-356` |

**B-03 prose change (BC-2.05.007 and BC-2.05.008):**

BC-2.05.007 Precondition 3 BEFORE:
> "At least one of `self.http` or `self.tls` is configured (the counter does not increment for unconfigured dispatchers -- dispatcher.rs:188-191)."

BC-2.05.007 Precondition 3 AFTER:
> "At least one of `self.http`, `self.tls`, `self.modbus`, or `self.dnp3` is configured (the counter does not increment for fully-unconfigured dispatchers -- dispatcher.rs:352-356)."

BC-2.05.007 Invariant 3 BEFORE:
> "The counter increments only when at least one analyzer is configured (guard at dispatcher.rs:188-191: `if self.http.is_some() || self.tls.is_some()`)."

BC-2.05.007 Invariant 3 AFTER:
> "The counter increments only when at least one analyzer is configured (guard at dispatcher.rs:352-356: `if self.http.is_some() || self.tls.is_some() || self.modbus.is_some() || self.dnp3.is_some()`)."

BC-2.05.008 Description/Preconditions/Postconditions/Invariants/EC-001/EC-002/Evidence similarly widened from `http/tls` two-analyzer to `http/tls/modbus/dnp3` four-analyzer throughout.

**Version bumps:**

| BC | Old | New |
|----|-----|-----|
| BC-2.05.001 | 1.6 | 1.7 |
| BC-2.05.002 | 1.5 | 1.6 |
| BC-2.05.003 | 1.6 | 1.7 |
| BC-2.05.004 | 1.4 | 1.5 |
| BC-2.05.005 | 1.4 | 1.5 |
| BC-2.05.006 | 1.4 | 1.5 |
| BC-2.05.007 | 1.3 | 1.4 |
| BC-2.05.008 | 1.5 | 1.6 |
| BC-2.05.009 | 1.3 | 1.4 |

BC-INDEX ss-05 section updated with inline `<!-- vN.N: ... -->` annotations on all 9 rows.

---

## [pass-17-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-17 Remediation — Holdout MITRE-catalog counts (C-01/C-02/C-03/C-04) + two LOWs (D-01/D-02) + architect module-decomposition bump log

**Summary:** Remediates five Pass-17 findings (4 CRITICAL/MED holdout-count staleness + 2 LOWs)
and logs the architect's module-decomposition.md v1.6→v1.7 bump (Pass-17 C-05/C-23 PLANNED
markers). No story bodies touched.

**Shipped truth verified from src/mitre.rs (SEEDED_TECHNIQUE_IDS, EMITTED_IDS, MitreTactic enum):**

| Fact | Value | Source |
|------|-------|--------|
| Seeded technique IDs | 23 (11 Enterprise + 12 ICS) | `SEEDED_TECHNIQUE_ID_COUNT = 23`; `SEEDED_TECHNIQUE_IDS` array at src/mitre.rs:305-333 |
| Emitted technique IDs | 15 (6 Enterprise + 7 ICS + 2 STORY-109) | `EMITTED_IDS` array at src/mitre.rs:221-240 |
| Catalogue-only (never emitted) | 8 (23 − 15): T1040, T1071, T1071.001, T1071.004, T1573, T0846, T1692.002, T0885 | Derived |
| MitreTactic variants | 17 (14 Enterprise + 3 ICS: IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact) | `all_tactics_in_report_order` at src/mitre.rs:100-120 |
| T0886 in catalog? | NO — not present in any match arm of `technique_info` | src/mitre.rs:129-179 |
| T0885 in catalog? | YES — "Commonly Used Port" at src/mitre.rs:158 | src/mitre.rs:158 |
| T0888 in catalog? | YES — "Remote System Information Discovery" at src/mitre.rs:168-171 | src/mitre.rs:168-171 |

The stale values in the holdouts (16 tactics, 15 seeded, 5 emitted, 9 catalogue-only) predate
Modbus (v0.4) + DNP3 (v0.6) + STORY-109; they are greenfield-era counts that contradict the
holdouts' own anchored BCs (BC-2.10.004 = 17 variants; BC-2.10.005 = 23 seeded/current).

**Architect bump logged (Pass-17):**

| Artifact | Change | Reason |
|----------|--------|--------|
| `architecture/module-decomposition.md` | v1.6 → v1.7 | Pass-17 C-05/C-23 PLANNED markers added |

**C-01 (CRITICAL) — HS-025 stale tactic count (16→17) and ICS-unique count (2→3):**

| Location | Before | After |
|----------|--------|-------|
| Scenario step 2 | "exactly 16 entries (14 Enterprise + 2 ICS-unique)" | "exactly 17 entries (14 Enterprise + 3 ICS-unique)" |
| BC table BC-2.10.004 row | "(16 total)" | "(17 total)" |
| Verification approach | "Count: exactly 16 tactics." | "Count: exactly 17 tactics." |
| Evaluation Rubric (3 occurrences) | "exactly 16 entries", "16 canonical ATT&CK", "all 16 tactics" | "exactly 17 entries", "17 canonical ATT&CK", "all 17 tactics" |
| Edge Conditions | "The two ICS-unique tactics" | "The three ICS-unique tactics" |
| Failure Guidance | "count is not 16" | "count is not 17" |

**C-02 (CRITICAL) — HS-008 stale tactic count and seeded count:**

| Location | Before | After |
|----------|--------|-------|
| BC table BC-2.10.004 row | "(16 total)" | "(17 total)" |
| BC table BC-2.10.005 row | "all 15 seeded IDs" | "all 23 seeded IDs" |

**C-03 (CRITICAL) — HS-009 stale seeded count, stale emitted list (5→15), stale catalogue-only count (9→8):**

| Location | Before | After |
|----------|--------|-------|
| BC table BC-2.10.005 row | "all 15 seeded IDs" | "all 23 seeded IDs" |
| Verification approach | "5 currently-emitted technique IDs (T1083, T1505.003, T1046, T1036, T1027). All 5 must resolve." | "15 currently-emitted technique IDs (T1027, T1036, T1046, T1083, T1499.002, T1505.003, T1692.001, T0836, T0814, T0806, T0835, T0831, T0888, T1691.001, T0827). All 15 must resolve." |
| Edge Conditions | "9 catalogued-but-never-emitted IDs (T1040, T1071, etc.)" | "8 catalogued-but-never-emitted IDs (T1040, T1071, T1071.001, T1071.004, T1573, T0846, T1692.002, T0885)" |

**C-04 (MEDIUM) — HS-009 T0886 false catalog-membership claim:**

T0886 is NOT in the catalog (no match arm in `technique_info`). The old text claimed "T0886
or similar ICS technique IDs ... ICS IDs are in the catalog" — a false assertion when evaluated.

| Location | Before | After |
|----------|--------|-------|
| Edge Conditions first bullet | "T0886 or similar ICS technique IDs should not confuse the lookup (ICS IDs are in the catalog)." | "T0885 and other catalogued ICS technique IDs should not confuse the lookup. T0886 is NOT in the catalog (not a seeded ID); use T9999 or another explicitly unregistered ID to test the unknown-ID path. Catalogued ICS IDs that are staged (not yet emitted) include T0846, T1692.002, T0885." |

**D-01 (LOW) — nfr-catalog.md NFR-OBS-010 AC cell "all four fields" disambiguation:**

Parallel fix to the Pass-15 C-05 sibling fix in interface-definitions.md:360.

| Location | Before | After |
|----------|--------|-------|
| NFR-OBS-010 Target cell | "downstream consumers must handle key-absent for all four fields." | "downstream consumers must handle key-absent rather than key-present-but-null for all four optional-presence fields — mitre_techniques (omitted when empty via Vec::is_empty) and the three Option fields source_ip/timestamp/direction (omitted when None via Option::is_none)." |

**D-02 (LOW) — domain-spec.md Summary-Metrics component count (21 vs "24"):**

**Classification: FROZEN pre-F2 ingestion baseline.**

Evidence: domain-spec.md is the brownfield ingestion output at develop@0082a0c (2026-05-20),
version 1.0, with no modifications list. The 217 BCs, 282 tests, and ADRs 0001-0004 counts
are all pre-F2 ingestion-era figures. The document has no live-update mechanism.

The "21 components (C-1..C-20 + C-21)" row is internally correct: the 24 source files (row 1)
map to 21 C-numbered components because config.rs, stats.rs, and csv.rs are unnumbered
data/support modules — this is explicitly noted in the row text. The Pass-17 finding's
"24 components" figure is a misread of "24 source files" as "24 components." The
C-numbering ends at C-21; C-22/C-23/C-24 do not exist in any spec artifact.

**Action: ERRATUM NOTE (no row content changed).** Added a dated HTML comment erratum block
immediately before the metrics table in domain-spec.md, stating: (a) this is the pre-F2
ingestion baseline at develop@0082a0c; (b) counts are FROZEN; (c) current values are in
ARCH-INDEX.md and BC-INDEX.md; (d) the 24-source-files / 21-C-components mapping is explained.
Individual table rows NOT rewritten. Domain-spec.md version field NOT bumped (frozen baseline).

**Version bumps:**

| Document | Before | After |
|----------|--------|-------|
| HS-025-ics-tactic-display-and-non-exhaustive.md | 1.0 | 1.1 |
| HS-008-mitre-tactic-display-and-kill-chain-order.md | 1.0 | 1.1 |
| HS-009-mitre-technique-lookup-unknown-ids.md | 1.1 | 1.2 |
| prd-supplements/nfr-catalog.md | 1.8 | 1.9 |
| specs/domain/domain-spec.md | 1.0 | 1.0 (frozen; erratum note added before metrics table; no version bump on frozen baseline) |

**BC-INDEX rows:** None.

**Story body changes:** None (constraint per task).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| HS-025 | v1.0→v1.1; 16→17 tactic count; 2→3 ICS-unique count (7 occurrences total) | `.factory/holdout-scenarios/HS-025-ics-tactic-display-and-non-exhaustive.md` |
| HS-008 | v1.0→v1.1; BC table (16→17 total); BC table (15→23 seeded IDs) | `.factory/holdout-scenarios/HS-008-mitre-tactic-display-and-kill-chain-order.md` |
| HS-009 | v1.1→v1.2; BC table 15→23 seeded IDs; Verification 5→15 emitted IDs (full list); Edge Conditions 9→8 catalogue-only (full list); T0886 false claim replaced with T0885 accurate example + T0886 negation | `.factory/holdout-scenarios/HS-009-mitre-technique-lookup-unknown-ids.md` |
| nfr-catalog.md | v1.8→v1.9; NFR-OBS-010 AC cell "all four fields" → explicit Vec+3-Option enumeration | `.factory/specs/prd-supplements/nfr-catalog.md` |
| domain-spec.md | Frozen; HTML comment erratum block added before metrics table (no row content changed; no version bump) | `.factory/specs/domain/domain-spec.md` |
| spec-changelog.md | Pass-17-fixes entry prepended; architect module-decomposition v1.6→v1.7 bump logged | `.factory/spec-changelog.md` |

---

## [pass-16-fixes-2026-06-13] — 2026-06-13

### ERRATUM + PATCH: Pass-16 C-01 Remediation — chunk3-reeval.md frozen-record erratum + architect bump log

**Summary:** Remediates Pass-16 finding C-01 (MEDIUM). Adds a dated erratum note to
`chunk3-reeval.md` (the `*-reeval.md` sibling missed by the Pass-15 H3 sweep), and logs
the architect's five Pass-16 version bumps. No other files touched.

**C-01 (MEDIUM) — chunk3-reeval.md HS-058 row "mitre=null" — frozen-record erratum:**

Classification: FROZEN historical run-record. Evidence: header states "All scores are by
OBSERVED behavior of `target/debug/wirerust analyze`..."; per-scenario table contains
satisfaction scores, PASS/FAIL verdicts, and verbatim observed-output quotes for a specific
past binary build. Same structural pattern as chunk1-eval.md and chunk3-eval.md (classified
as frozen in Pass-15 H3). In-place rewrite of historical verdicts is not permitted.

The Pass-15 H3 erratum sweep covered chunk1-eval.md and chunk3-eval.md but missed
chunk3-reeval.md because the sweep pattern matched `chunk*-eval.md` and excluded this
`*-reeval.md` sibling.

Stale reference in HS-058 row: "all mitre=null" — pre-v0.3.0 schema language.

Current schema truth (ADR-006 / STORY-100 v0.3.0): `mitre_techniques: Vec<String>` with
`skip_serializing_if = "Vec::is_empty"` — the key is ABSENT (not null) when the Vec is
empty. JSON `null` is not a valid serialized form for an absent-when-empty Vec field.

Action: Added HTML comment erratum block immediately after the H1 heading (before the
re-evaluator intro paragraph), contextualizing the stale schema language in the HS-058 row
and explaining that this is a frozen record. Historical content untouched.

| Before | After |
|--------|-------|
| No erratum note; HS-058 row contains "all mitre=null" without qualification | HTML comment erratum block added after H1, stating: frozen run-record; "mitre=null" reflects pre-v0.3.0 schema language; current schema uses `mitre_techniques: Vec<String>` with key ABSENT when empty, per ADR-006 |

**Architect bumps logged (Pass-16):**

| Artifact | Change | Reason |
|----------|--------|--------|
| `architecture/tooling-selection.md` | v1.2 → v1.3 | Pass-16 architect bump |
| `architecture/system-overview.md` | v1.2 → v1.3 | Pass-16 architect bump |
| `architecture/api-surface.md` | v1.3 → v1.4 | Pass-16 architect bump |
| `architecture/dependency-graph.md` | v1.4 → v1.5 | Pass-16 architect bump |
| `specs/architecture/ADR-005.md` | modified[] entry appended for D-01 line-74 [2,253]→[2,254] | Pass-16 D-01 remediation |

**Version bumps (spec artifacts):**

No versioned spec artifact was modified. chunk3-reeval.md is a run-record without a version
field (same as chunk1-eval.md / chunk3-eval.md — no version bump applicable).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| chunk3-reeval.md | HTML comment erratum note added after H1 (frozen record; historical content untouched) | `.factory/holdout-scenarios/evaluations/chunk3-reeval.md` |
| spec-changelog.md | Pass-16-fixes entry prepended; architect Pass-16 bumps logged | `.factory/spec-changelog.md` |

---

## [pass-15-h3-eval-erratum-2026-06-13] — 2026-06-13

### ERRATUM: Pass-15 C-01 Burst H3 — Frozen Evaluation Records (chunk1-eval.md, chunk3-eval.md)

**Summary:** Pass-15 C-01 burst H3 of 3. Assessed two holdout EVALUATION-RESULT files for
stale singular `mitre_technique` references. Both files were classified as FROZEN HISTORICAL
RUN-RECORDS and treated with erratum notes rather than in-place field fixes.

**Shipped truth (unchanged):** `mitre_techniques: Vec<String>` per ADR-006/STORY-100 (v0.3.0);
scalar `mitre_technique` removed. Key is absent when Vec is empty
(`skip_serializing_if = "Vec::is_empty"`).

**Classification decisions:**

| File | Classification | Evidence | Action |
|------|---------------|----------|--------|
| `evaluations/chunk1-eval.md` | FROZEN historical run-record (type a) | Header states "Date: 2026-06-01"; documents evaluator verdicts from a specific binary build; per-scenario table contains verbatim observed-output quotes and satisfaction scores | Erratum note added to header block |
| `evaluations/chunk3-eval.md` | FROZEN historical run-record (type a) | Same structure: "Evaluator: black-box holdout...Binary: target/debug/wirerust @ develop"; per-scenario table contains satisfaction scores, PASS/FAIL verdicts, and evaluator-observed output quotes for a specific past run | HTML comment erratum note added after header block |

**Stale references identified (NOT changed — frozen record):**

| File | Location | Stale text | Schema truth |
|------|----------|------------|--------------|
| chunk1-eval.md | HS-007 row | "direction/mitre_technique present only when Some" | `mitre_techniques: Vec<String>` absent when empty; `direction` is `Option` absent when None |
| chunk1-eval.md | HS-016 row | "mitre_technique=T1036 (verified)" | `mitre_techniques` array containing "T1036" |
| chunk1-eval.md | HS-017 row | "all mitre_technique IDs resolve to catalog names in --mitre" | `mitre_techniques` array entries resolve to names |
| chunk3-eval.md | HS-059 row | "all mitre_technique absent/None" | `mitre_techniques` key absent (empty Vec, Vec::is_empty) |
| chunk3-eval.md | HS-074 row | "mitre_technique null" | `mitre_techniques` key absent (empty Vec, Vec::is_empty) |

**Actions taken:**

- `chunk1-eval.md`: Added dated erratum line (`erratum: 2026-06-13`) in the header metadata block, contextualizing stale schema references in HS-007/016/017 rows. Historical content untouched.
- `chunk3-eval.md`: Added HTML comment erratum block immediately after the header paragraph, contextualizing stale schema references in HS-059/074 rows. Historical content untouched.
- No version field exists in either file (they are run-records, not versioned spec artifacts). No version bump applicable.

**Before → after (erratum notes only; no content changes):**

chunk1-eval.md header: added `erratum: 2026-06-13 (Pass-15 H3)` line after `Date: 2026-06-01` stating this is a frozen record and the stale `mitre_technique` refs reflect pre-v0.3.0 schema.

chunk3-eval.md: added HTML comment erratum block after the header paragraph (before Per-Scenario Results) stating the same, referencing HS-059 and HS-074 rows specifically.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| chunk1-eval.md | Erratum note added to header (frozen record; historical content untouched) | `.factory/holdout-scenarios/evaluations/chunk1-eval.md` |
| chunk3-eval.md | HTML comment erratum note added after header paragraph (frozen record; historical content untouched) | `.factory/holdout-scenarios/evaluations/chunk3-eval.md` |
| spec-changelog.md | Pass-15 H3 entry prepended | `.factory/spec-changelog.md` |

---

## [pass-15-h2-holdout-sweep-2026-06-13] — 2026-06-13

### PATCH: Pass-15 C-01/C-02/C-03 Remediation — Holdout-Scenarios H2 Burst (CSV-schema + phantom-field + timestamp sweep)

**Summary:** Remediates Pass-15 findings C-01 (CSV column header singular → plural),
C-02 (phantom fields `mitre_technique_id` / `mitre_tactic` that never existed in Finding schema),
and C-03 (stale O-01 "timestamp always None" claim; O-01 is closed, timestamp is wired at 21/22
emission sites via STORY-097/098/099). Eight HS files touched; no BC-INDEX rows for holdouts.

**Shipped truth confirmed against src/findings.rs + src/reporter/csv.rs:**
- Finding JSON: `mitre_techniques` (Vec<String>, key absent when empty via `skip_serializing_if = "Vec::is_empty"`);
  `source_ip` / `timestamp` / `direction` (Option, key absent when None via `skip_serializing_if = "Option::is_none"`).
  No `mitre_technique` scalar, no `mitre_technique_id`, no `mitre_tactic` — those fields never existed.
- CSV column order (csv.rs:63-73): `category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp`
  (9 columns; column 6 is `mitre_techniques` plural, semicolon-joined; empty vec → empty string).
- timestamp IS wired (O-01 CLOSED); BC-2.04.054 segment-limit summary finding is the sole by-design timestamp=None exception.

**Discrimination rule applied:**
- STALE (fixed): live assertions / jq selectors / JSON key specs referencing `mitre_technique` (scalar),
  `mitre_technique_id`, `mitre_tactic`, CSV header with `mitre_technique` (singular), and "timestamp always None / absent from ALL findings".
- PRESERVED: natural-language prose ("MITRE technique T1036"), lookup-function references
  (technique_name / technique_tactic in mitre.rs), explicit changelog/history, negation statements
  ("There is no `mitre_technique_id` field...").

**Per-file findings (STALE fixed vs PRESERVED):**

| File | STALE fixed | PRESERVED | Notes |
|------|-------------|-----------|-------|
| HS-074-tls-ssl30-real-world-pcap.md | 1 | 0 | Step 6: `mitre_technique is null` → `mitre_techniques key is absent` (Vec::is_empty) |
| HS-080-csv-nine-column-schema-stable.md | 1 | 0 | Scenario item 3 header string: `mitre_technique` → `mitre_techniques`; byte-for-byte header now matches csv.rs |
| HS-083-csv-optional-fields-none-encoded-as-empty.md | 5 | 0 | Scenario step 1 scalar → Vec; step 3 "four optional" → explicit Vec+Option framing; BC table row; Verification construct; Edge Conditions injection example |
| HS-098-end-to-end-pcap-to-csv-report.md | 1 | 0 | Verification step 5 header array assertion: `mitre_technique` → `mitre_techniques` |
| HS-007-json-serialization-skip-none-fields.md | 6 | 0 | C-02: scenario step 3 rewritten to real schema; Verification mitre/timestamp assertions rewritten; Edge Conditions phantom-field refs replaced with negation statements; C-03: "timestamp always None / O-01 limitation" → wired-timestamp behavior |
| HS-009-mitre-technique-lookup-unknown-ids.md | 1 | 3 | Verification "for each finding with a `mitre_technique_id`" → "non-empty `mitre_techniques` array"; prose "T1083/T1036" natural lang PRESERVED; lookup-function refs PRESERVED |
| HS-016-real-world-corpus-evasion-pcap.md | 3 | 0 | jq selector `select(.mitre_technique_id == "T1036")` → `select(.mitre_techniques // [] \| index("T1036"))`; prose "mitre_technique_id=T1036" → "mitre_techniques containing T1036" (x2 incl. rubric) |
| HS-017-cross-subsystem-e1-e7-finding-construction.md | 5 | 0 | C-02: Verification step 3 `mitre_technique_id` → `non-empty mitre_techniques`; BC table row updated; C-03: step 4 / step 2 / BC table / Failure Guidance stale "timestamp absent from ALL" → per-finding wired-timestamp behavior |

**STALE → fixed detail (before → after):**

HS-074:
- `Assert mitre_technique is null for all cipher/protocol weakness findings.`
  → `Assert mitre_techniques key is absent for all cipher/protocol weakness findings (empty Vec omitted via skip_serializing_if = "Vec::is_empty").`

HS-080:
- `category,verdict,confidence,summary,evidence,mitre_technique,source_ip,direction,timestamp`
  → `category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp`

HS-083 (five changes):
- Scenario step 1: `mitre_technique = None` → `mitre_techniques = vec![]`
- Scenario step 3: `four optional columns` → `four columns 6–9 (one Vec-backed, three Option-backed)`
- BC table Postcondition 1: `None mitre_technique → empty string` → `empty mitre_techniques Vec → empty string (join of empty vec)`
- Verification construct: `A Finding where all four Option fields are None` → `A Finding where mitre_techniques = vec![] and all three Option fields are None`
- Edge Conditions: `mitre_technique = Some("=HYPERLINK(...)")` → `mitre_techniques = vec!["=HYPERLINK(...)"]` with join/neutralize logic

HS-098:
- `["category","verdict","confidence","summary","evidence","mitre_technique","source_ip","direction","timestamp"]`
  → `["category","verdict","confidence","summary","evidence","mitre_techniques","source_ip","direction","timestamp"]`

HS-007 (six changes):
- Scenario step 3: phantom-field list `mitre_technique_id / mitre_tactic` → real schema (mitre_techniques Vec + 3 Options); added "those never existed" negation
- BC-2.09.006 table row: updated to reflect Vec + Option coverage
- Verification: "timestamp does NOT appear in any finding" → per-finding wired-timestamp logic; mitre key absence rewritten
- Edge Conditions: "both mitre_technique_id and mitre_tactic set" → "non-empty mitre_techniques vec" + negation; "Timestamp is always None (O-01)" → "O-01 is CLOSED / sole by-design case is BC-2.04.054"

HS-009:
- `For each finding with a mitre_technique_id:` → `For each finding with a non-empty mitre_techniques array:`

HS-016:
- `jq '.findings[] | select(.mitre_technique_id == "T1036")'`
  → `jq '.findings[] | select(.mitre_techniques // [] | index("T1036"))'`
- `mitre_technique_id=T1036` (prose, Verification section) → `mitre_techniques containing "T1036"`
- `mitre_technique_id (T1036)` (Evaluation Rubric) → `mitre_techniques containing "T1036"`

HS-017 (five changes):
- Verification step 3: `mitre_technique_id` → `non-empty mitre_techniques array`
- BC-2.09.006 table row: updated to reflect Vec + Option coverage
- Scenario step 4: "timestamp absent from ALL findings" → per-finding presence/absence per shipped truth
- Verification step 2: "each finding lacks timestamp key" → per-finding wired-timestamp jq check
- Failure Guidance: "shows null timestamp" → "null (absent not null) or wrong timestamp presence/absence"

**Version bumps:**

| File | Before | After |
|------|--------|-------|
| HS-074-tls-ssl30-real-world-pcap.md | 1.0 | 1.1 |
| HS-080-csv-nine-column-schema-stable.md | 1.0 | 1.1 |
| HS-083-csv-optional-fields-none-encoded-as-empty.md | 1.0 | 1.1 |
| HS-098-end-to-end-pcap-to-csv-report.md | 1.0 | 1.1 |
| HS-007-json-serialization-skip-none-fields.md | 1.0 | 1.1 |
| HS-009-mitre-technique-lookup-unknown-ids.md | 1.0 | 1.1 |
| HS-016-real-world-corpus-evasion-pcap.md | 1.1 | 1.2 |
| HS-017-cross-subsystem-e1-e7-finding-construction.md | 1.0 | 1.1 |

**BC-INDEX rows:** None (holdout files have no BC-INDEX rows per constraint).

**Story body changes:** None (constraint per task).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| HS-074 | v1.0→v1.1; step 6 `mitre_technique` → `mitre_techniques` key-absent assertion | `.factory/holdout-scenarios/HS-074-tls-ssl30-real-world-pcap.md` |
| HS-080 | v1.0→v1.1; header string `mitre_technique` → `mitre_techniques` | `.factory/holdout-scenarios/HS-080-csv-nine-column-schema-stable.md` |
| HS-083 | v1.0→v1.1; 5 stale scalar/4-Option-field refs corrected to Vec/3-Option schema | `.factory/holdout-scenarios/HS-083-csv-optional-fields-none-encoded-as-empty.md` |
| HS-098 | v1.0→v1.1; header array assertion `mitre_technique` → `mitre_techniques` | `.factory/holdout-scenarios/HS-098-end-to-end-pcap-to-csv-report.md` |
| HS-007 | v1.0→v1.1; 6 phantom-field + stale-timestamp assertions rewritten to real schema + wired-timestamp truth | `.factory/holdout-scenarios/HS-007-json-serialization-skip-none-fields.md` |
| HS-009 | v1.0→v1.1; 1 `mitre_technique_id` → `mitre_techniques` field ref fixed; lookup-function refs PRESERVED | `.factory/holdout-scenarios/HS-009-mitre-technique-lookup-unknown-ids.md` |
| HS-016 | v1.1→v1.2; jq selector + 2 prose refs `mitre_technique_id` → `mitre_techniques` | `.factory/holdout-scenarios/HS-016-real-world-corpus-evasion-pcap.md` |
| HS-017 | v1.0→v1.1; 2 phantom-field refs + 3 stale-timestamp assertions corrected | `.factory/holdout-scenarios/HS-017-cross-subsystem-e1-e7-finding-construction.md` |
| spec-changelog.md | Pass-15 H2 entry prepended | `.factory/spec-changelog.md` |

---

## [pass-15-h1-holdout-sweep-2026-06-13] — 2026-06-13

### PATCH: Pass-15 C-01 Remediation — Holdout-Scenarios H1 Burst (8 HS files, mitre_technique → mitre_techniques)

**Summary:** Remediates Pass-15 finding C-01 (CRITICAL, holdout-scenarios sweep — burst H1 of 3).
The Pass-14 field rename (`mitre_technique: Option<String>` → `mitre_techniques: Vec<String>`) was
propagated to `.factory/specs/` but NOT to `.factory/holdout-scenarios/`, leaving stale F4-evaluator
assertions. This burst fixes all 8 designated HS files.

Shipped truth: Finding JSON key is `mitre_techniques` (array of technique-ID strings); empty → key
absent (`skip_serializing_if = "Vec::is_empty"`). The scalar `mitre_technique` field no longer exists.

**Discrimination rule applied:**
- STALE (fixed): any reference to the Finding field as `mitre_technique` — JSON key, jq selector
  (`.mitre_technique == "X"`), prose assertion ("mitre_technique is null/None"), struct snippet.
- PRESERVED: natural-language prose naming the MITRE technique ("MITRE technique T1036", "MITRE code
  is T1499.002", "T1036/ConflictingOverlap findings count", "MITRE technique is null for all four
  header anomaly findings"); references to mitre.rs lookup functions; changelog/history.

**Per-file findings:**

| File | STALE fixed | PRESERVED | Notes |
|------|-------------|-----------|-------|
| HS-032-tcp-evasion-detection.md | 1 | 2 | Verification prose `mitre_technique` field ref → `mitre_techniques` array; prose "T1036" natural language x 2 PRESERVED |
| HS-046-real-world-clean-pcap.md | 1 | 2 | jq `.mitre_technique == "T1036"` → `.mitre_techniques // [] \| index("T1036")` selector; "T1036/ConflictingOverlap findings" prose x 2 PRESERVED |
| HS-047-real-world-evasion-corpus.md | 2 | 0 | Scenario step field ref + jq selector both STALE; both fixed to plural array form |
| HS-056-sni-control-byte-detection.md | 1 | 0 | Verification step 2 `mitre_technique == "T1027"` → array index assertion |
| HS-057-sni-non-ascii-utf8-arm3.md | 1 | 1 | Verification step 5 `mitre_technique == "T1027"` → array index; Scenario prose "MITRE technique T1027" PRESERVED |
| HS-058-http-header-anomaly-detections.md | 1 | 1 | Verification step 2 `mitre_technique is null/None` → key-absent assertion; Evaluation Rubric "MITRE technique is null" prose PRESERVED |
| HS-059-tls-weak-cipher-findings.md | 2 | 1 | Scenario step 5 field ref + Verification step 2 field ref both STALE; Evaluation Rubric "MITRE technique is null" prose PRESERVED |
| HS-065-http-too-many-headers-finding.md | 1 | 1 | Verification step 1 `mitre_technique == "T1499.002"` → array index; Rubric "MITRE code is T1499.002" prose PRESERVED |

**STALE → fixed detail (before → after):**

HS-032:
- `The finding's mitre_technique field contains "T1036".`
  → `The finding's mitre_techniques array contains "T1036" (i.e., select(.mitre_techniques | index("T1036"))).`

HS-046:
- `jq '.findings | map(select(.mitre_technique == "T1036")) | length'`
  → `jq '.findings | map(select(.mitre_techniques // [] | index("T1036"))) | length'`

HS-047:
- `findings array contains at least one finding with mitre_technique == "T1036"`
  → `findings array contains at least one finding whose mitre_techniques array contains "T1036"`
- `jq '.findings | map(select(.mitre_technique == "T1036")) | length'`
  → `jq '.findings | map(select(.mitre_techniques // [] | index("T1036"))) | length'`

HS-056:
- `Assert that finding has mitre_technique == "T1027"`
  → `Assert that finding has mitre_techniques array containing "T1027" (i.e., select(.mitre_techniques | index("T1027")))`

HS-057:
- `Assert all three findings have mitre_technique == "T1027"`
  → `Assert all three findings have mitre_techniques array containing "T1027" (i.e., select(.mitre_techniques | index("T1027")) matches all three)`

HS-058:
- `mitre_technique is null/None`
  → `mitre_techniques key is absent (empty vec, omitted via skip_serializing_if)`

HS-059:
- `All Session 2 findings have mitre_technique as null`
  → `All Session 2 findings have mitre_techniques key absent (empty vec, omitted via skip_serializing_if = "Vec::is_empty")`
- `mitre_technique == null` in Verification step 2
  → `mitre_techniques key absent (empty vec omitted)`

HS-065:
- `findings contains exactly 3 entries with mitre_technique == "T1499.002"`
  → `findings contains exactly 3 entries whose mitre_techniques array contains "T1499.002" (i.e., select(.mitre_techniques | index("T1499.002")))`

**Version bumps:**

| File | Before | After |
|------|--------|-------|
| HS-032-tcp-evasion-detection.md | 1.0 | 1.1 |
| HS-046-real-world-clean-pcap.md | 1.0 | 1.1 |
| HS-047-real-world-evasion-corpus.md | 1.0 | 1.1 |
| HS-056-sni-control-byte-detection.md | 1.0 | 1.1 |
| HS-057-sni-non-ascii-utf8-arm3.md | 1.0 | 1.1 |
| HS-058-http-header-anomaly-detections.md | 1.0 | 1.1 |
| HS-059-tls-weak-cipher-findings.md | 1.0 | 1.1 |
| HS-065-http-too-many-headers-finding.md | 1.0 | 1.1 |

**BC-INDEX rows:** None (holdout files have no BC-INDEX rows per constraint).

**Story body changes:** None (constraint per task).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| HS-032 | v1.0→v1.1; 1 STALE field ref fixed | `.factory/holdout-scenarios/HS-032-tcp-evasion-detection.md` |
| HS-046 | v1.0→v1.1; 1 STALE jq selector fixed | `.factory/holdout-scenarios/HS-046-real-world-clean-pcap.md` |
| HS-047 | v1.0→v1.1; 2 STALE refs fixed (prose + jq) | `.factory/holdout-scenarios/HS-047-real-world-evasion-corpus.md` |
| HS-056 | v1.0→v1.1; 1 STALE assertion fixed | `.factory/holdout-scenarios/HS-056-sni-control-byte-detection.md` |
| HS-057 | v1.0→v1.1; 1 STALE assertion fixed | `.factory/holdout-scenarios/HS-057-sni-non-ascii-utf8-arm3.md` |
| HS-058 | v1.0→v1.1; 1 STALE null-assertion fixed | `.factory/holdout-scenarios/HS-058-http-header-anomaly-detections.md` |
| HS-059 | v1.0→v1.1; 2 STALE null-assertions fixed | `.factory/holdout-scenarios/HS-059-tls-weak-cipher-findings.md` |
| HS-065 | v1.0→v1.1; 1 STALE assertion fixed | `.factory/holdout-scenarios/HS-065-http-too-many-headers-finding.md` |
| spec-changelog.md | Pass-15 H1 entry prepended | `.factory/spec-changelog.md` |

---

## [pass-15-fixes-2026-06-13] — 2026-06-13

### PATCH: Pass-15 Remediation — Four PO findings (C-04/D-01/B-01/C-05) + Architect A-01 bump log

**Summary:** Remediates four Pass-15 product-owner bucket findings and logs the architect's
VP-INDEX v2.1→v2.2 bump (Pass-15 A-01). No story bodies touched. No holdout-scenarios touched.

**Architect bump logged (A-01):**

| Artifact | Change | Reason |
|----------|--------|--------|
| `specs/verification-properties/VP-INDEX.md` | v2.1 → v2.2 | Pass-15 A-01: VP-024 Verified-BCs reconciled to BCs 001-006; BC-2.16.007 footnote clarified as test-sufficient/non-Kani |

**C-04 (MEDIUM) — inv-01-core-invariants.md duplicate `version:` key (REGRESSION from Pass-14):**

| Before | After |
|--------|-------|
| Frontmatter had two top-level `version:` keys: `version: "1.1"` at line 7 and `version: "1.2"` at line 11 | Single `version: "1.2"` field retained; stale `version: "1.1"` key removed; YAML frontmatter now parses cleanly with one version key. Body/changelog already reflected v1.2; no further content bump warranted. |

**D-01 (MEDIUM) — BC-2.11.024 Evidence Types Used stale guard-clause description:**

| Before | After |
|--------|-------|
| `"guard clause": explicit unwrap_or("") / unwrap_or_default() calls at csv.rs:82-85 for all four Option fields` | `"guard clause": mitre_techniques.join(";") at csv.rs:87 for the Vec<String> field (empty vec → ""); unwrap_or_default() calls at csv.rs:88-90 for the three Option fields (source_ip, direction, timestamp)` |

Verified against `src/reporter/csv.rs`: join at :87, source_ip at :88, direction at :89, timestamp at :90. BC-2.11.024 bumped v1.6→v1.7. BC-INDEX annotation updated.

**B-01 (LOW) — BC-INDEX narrative prose "v1.4→v1.5" for BC-2.02.009 (three instances):**

Three narrative references in BC-INDEX incorrectly described BC-2.02.009 as "revised v1.4→v1.5".
The BC-2.02.009 body is at v1.6; the BC-INDEX row inline comment correctly says v1.6.

| Location | Before | After |
|----------|--------|-------|
| BC-INDEX header blockquote (~line 28) | `was revised v1.4→v1.5 in Feature Mode F2` | `was revised to v1.6 in Feature Mode F2` |
| Ingestion-to-L3 Mapping table (~line 456) | `BC-2.02.009 revised v1.4→v1.5 in F2 ARP delta` | `BC-2.02.009 revised to v1.6 in F2 ARP delta` |
| Canonical derivation paragraph (~line 473) | `BC-2.02.009 was revised v1.4→v1.5 (ADR-008 Decision 1...)` | `BC-2.02.009 was revised to v1.6 (ADR-008 Decision 1...)` |

BC-INDEX bumped v1.23→v1.24.

**C-05 (LOW) — interface-definitions.md "all four fields" ambiguous prose (~line 360):**

| Before | After |
|--------|-------|
| `Downstream consumers MUST handle key-absent rather than key-present-but-null for all four fields.` | `Downstream consumers MUST handle key-absent rather than key-present-but-null for all four optional-presence fields -- mitre_techniques (omitted when empty via Vec::is_empty) and the three Option fields source_ip/timestamp/direction (omitted when None via Option::is_none).` |

interface-definitions.md bumped v1.1→v1.2.

**Version bumps:**

| Document | Before | After |
|----------|--------|-------|
| `specs/domain/invariants/inv-01-core-invariants.md` | v1.2 (duplicate key) | v1.2 (single key; no content change) |
| `specs/behavioral-contracts/ss-11/BC-2.11.024.md` | v1.6 | v1.7 |
| `specs/behavioral-contracts/BC-INDEX.md` | v1.23 | v1.24 |
| `specs/prd-supplements/interface-definitions.md` | v1.1 | v1.2 |

**Story body changes:** None (constraint per task).

**Holdout scenario changes:** None (separate burst).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| inv-01-core-invariants.md | Duplicate `version:` key removed; single `version: "1.2"` retained | `.factory/specs/domain/invariants/inv-01-core-invariants.md` |
| BC-2.11.024.md | v1.6→v1.7; Evidence Types Used guard clause updated to csv.rs:87-90 current shape | `.factory/specs/behavioral-contracts/ss-11/BC-2.11.024.md` |
| BC-INDEX.md | v1.23→v1.24; three "v1.4→v1.5" narrative references → "to v1.6"; BC-2.11.024 annotation updated | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| interface-definitions.md | v1.1→v1.2; "all four fields" ambiguous prose → explicit Vec + 3 Option enumeration | `.factory/specs/prd-supplements/interface-definitions.md` |
| spec-changelog.md | Pass-15-fixes entry appended; architect VP-INDEX v2.1→v2.2 bump logged | `.factory/spec-changelog.md` |

---

## [arp-f2-pass-14-po-burst-10-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 10 — O-01-closure propagation (domain/, prd-supplements/ — final sweep)

**Summary:** Closes the O-01 propagation in domain entity/capability docs and prd-supplements
that were not addressed in Bursts 1-9. Domain-debt O-01 (Finding.timestamp universally None)
is CLOSED — timestamp wired by STORY-097/098/099 (http/tls/reassembly) and STORY-102..110
(modbus/dnp3); 21 of 22 emission sites set timestamp:Some; BC-2.04.054 segment-limit summary
retains timestamp:None by design as the sole exception. All open-framing of O-01 removed.
Historical/changelog and "O-01 CLOSED" references preserved.

**Architect artifact bumps logged (architecture/ files — touched by architect, recorded here):**

| Artifact | Change | Reason |
|----------|--------|--------|
| `architecture/ARCH-INDEX.md` | v1.4 → v1.5 | O-01 closure reflected in subsystem annotations |
| `architecture/module-decomposition.md` | v1.5 → v1.6 | timestamp threading noted in module descriptions |
| `architecture/dependency-graph.md` | v1.3 → v1.4 | timestamp data-flow edges updated |

**domain/ fixes — 2 files changed:**

| Finding | File | Before | After |
|---------|------|--------|-------|
| E-4 RawPacket open-O-01 note | `domain/entities/ent-01-ingestion-decoding.md:88` | `timestamp_secs is read but never threaded to any Finding constructor (open item O-01).` | Reflects O-01 CLOSED: timestamp_secs threaded via STORY-097/098/099 + STORY-102..110; BC-2.04.054 sole exception noted |
| CAP-01 scope/limitations timestamp note | `domain/capabilities/cap-01-pcap-ingestion.md:~60` | `NEVER threaded through to Finding.timestamp at any emission site. See domain-debt.md item O-01.` | Reflects O-01 CLOSED: wired at 21/22 sites; BC-2.04.054 sole exception; references BC-2.09.007 and domain-debt.md RETIRED entry |

**prd-supplements/ fixes — 1 file changed (nfr-catalog.md):**

| Finding | Location | Before | After |
|---------|----------|--------|-------|
| NFR-PERF-002 mapping row | NFR-to-Module Mapping ~line 191 | `streaming refactor is O-01 class debt` | `streaming refactor is separate architectural debt (NFR-VIO-001) — unrelated to O-01 (timestamp threading; CLOSED)` |
| NFR-VIO-001 disposition row | NFR Violation Dispositions ~line 227 | `OPEN-DEBT (O-01 class)` | `OPEN-DEBT -- eager full-file load is a separate architectural concern from O-01 (timestamp threading, CLOSED); streaming refactor deferred` |

**Version bumps:**

| Document | Before | After |
|----------|--------|-------|
| `specs/domain/entities/ent-01-ingestion-decoding.md` | (no version field) | v1.1 |
| `specs/domain/capabilities/cap-01-pcap-ingestion.md` | (no version field) | v1.1 |
| `specs/prd-supplements/nfr-catalog.md` | 1.7 | 1.8 |

**BC body changes:** None. All O-01 references in BC bodies (ss-01, ss-04, ss-09) are
already correctly framed as "O-01 resolved by feature-100" — confirmed by grep; no
BC bodies touched.

**Story body changes:** None.

**H1 title changes:** None.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| ent-01-ingestion-decoding.md | v(none)→v1.1; E-4 open-O-01 note updated to CLOSED | `.factory/specs/domain/entities/ent-01-ingestion-decoding.md` |
| cap-01-pcap-ingestion.md | v(none)→v1.1; scope/limitations timestamp note updated to CLOSED | `.factory/specs/domain/capabilities/cap-01-pcap-ingestion.md` |
| nfr-catalog.md | v1.7→v1.8; NFR-PERF-002 mapping row + NFR-VIO-001 disposition: O-01 class framing removed | `.factory/specs/prd-supplements/nfr-catalog.md` |
| spec-changelog.md | Burst-10 entry appended; architect ARCH-INDEX v1.4→v1.5, module-decomposition v1.5→v1.6, dependency-graph v1.3→v1.4 bumps logged | `.factory/spec-changelog.md` |

---

## [arp-f2-pass-14-po-burst-9-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 9 (final) — prd.md O-01 stale claims + api-surface.md bump log

**Summary:** Final micro-burst of the ARP-F2 Pass-14 whole-corpus remediation. Removes three
residual O-01 stale current-state claims from prd.md (F2/F3/F4 findings), logs the architect's
api-surface.md v1.2→v1.3 bump, bumps prd.md v1.17→v1.18, and syncs BC-INDEX:36 to "(v1.18)".
Domain-debt O-01 (Finding.timestamp always None) is CLOSED — timestamp wired by STORY-097/098/099
across all applicable emission sites; BC-2.04.054 retains timestamp:None by design as the sole
exception.

**Architect artifact logged (F1):**

| Artifact | Change | Reason |
|----------|--------|--------|
| `architecture/api-surface.md` | v1.2 → v1.3 | timestamp annotation updated to reflect O-01 closed: `Finding.timestamp: Option<DateTime<Utc>>` now populated at 21/22 emission sites; BC-2.04.054 by-design exception noted |

**prd.md — 3 stale O-01 current-state items fixed (F2/F3/F4):**

| Finding | Location | Before | After |
|---------|----------|--------|-------|
| F2 LOW | §1.5 Out of Scope (~line 321) | `- Per-packet timestamp in findings (Finding.timestamp is always None; O-01)` | `- Per-packet timestamp in findings: RESOLVED — BC-2.09.007 (F2) wired timestamp ... domain-debt O-01 CLOSED. Exception: BC-2.04.054 retains timestamp:None by design.` |
| F3 LOW | §8 Domain Debt Index (~line 1496) | `\| O-01 \| Finding.timestamp always None; ... \| BC-2.09.001, BC-2.09.006 \|` | Row struck through; Description column appended `**[CLOSED — STORY-097/098/099; BC-2.04.054 retains timestamp:None by design]**` |
| F4 LOW | §2.9 ss-09 note (~lines 601-603) | "Known limitation: All 22 emission sites set timestamp: None (domain-debt O-01)... Finding.timestamp field exists but is never populated." | Replaced with: "BC-2.09.007 (F2) wired timestamp ... (STORY-097/098/099); domain-debt O-01 CLOSED. The segment-limit summary finding (BC-2.04.054) retains timestamp:None by design as the sole exception." |

Also cleaned up a co-located stale reference: NFR-VIO-001 note (~line 1084) had "(eager full-file load; O-01 context)" — the O-01 parenthetical was inaccurate (O-01 was about timestamps, not loading). Removed the O-01 reference; note now reads "(eager full-file load)" only.

**Post-edit grep confirmation:** No current-state "timestamp always None", "never populated", or open-O-01 claims remain in prd.md. Remaining O-01 occurrences are: range enumeration O-01..O-08 (line 40), two "O-01 CLOSED" statements (lines 321, 602), and one struck-through "[CLOSED]" row (line 1496).

**Version bumps:**

| Document | Before | After |
|----------|--------|-------|
| `specs/prd.md` | 1.17 | 1.18 |
| `specs/behavioral-contracts/BC-INDEX.md` | (v1.17) citation at line 36 | (v1.18) |

**Story body changes:** None. No BC body files touched.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| prd.md | v1.17→v1.18; F2/F3/F4 O-01 stale claims removed; NFR-VIO-001 O-01 parenthetical cleaned up | `.factory/specs/prd.md` |
| BC-INDEX.md | BC-INDEX:36 status-line citation synced (v1.17)→(v1.18) | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| spec-changelog.md | Burst-9 entry appended; architect api-surface.md v1.2→v1.3 bump logged | `.factory/spec-changelog.md` |

---

## [arp-f2-pass-14-po-burst-8-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 8 — prd-supplements cleanup (interface-definitions.md + nfr-catalog.md)

**Summary:** Final cleanup burst of the ARP-F2 Pass-14 whole-corpus remediation. Remediates
three stale items in interface-definitions.md and one stale item in nfr-catalog.md — the
two prd-supplements files missed in Bursts 1-7. All changes reflect post-ADR-006 (Decision 13,
STORY-100 AC-008) shipped reality: `mitre_techniques: Vec<String>` with
`skip_serializing_if = "Vec::is_empty"` and exactly three remaining Option fields
(`source_ip`, `timestamp`, `direction`) each with `skip_serializing_if = "Option::is_none"`.
Domain-debt O-01 CLOSED (timestamp wired via STORY-097/098/099). Verified against
src/findings.rs lines 148-161.

**Shipped-anchor verification (src/findings.rs):**

| Field | Line range | Attribute |
|-------|-----------|-----------|
| `mitre_techniques: Vec<String>` | :148-149 | `#[serde(skip_serializing_if = "Vec::is_empty")]` |
| `source_ip: Option<IpAddr>` | :150-151 | `#[serde(skip_serializing_if = "Option::is_none")]` |
| `timestamp: Option<DateTime<Utc>>` | :152-153 | `#[serde(skip_serializing_if = "Option::is_none")]` |
| `direction: Option<Direction>` | :160-161 | `#[serde(skip_serializing_if = "Option::is_none")]` |

No serde rename on `mitre_techniques`; JSON key is `mitre_techniques` verbatim. Scalar
`mitre_technique: Option<String>` is absent from the struct — confirmed.

Stale line anchor in NFR-OBS-010 (`src/findings.rs:132-145`) corrected to `:148-161`
(the annotated block for mitre_techniques comment + all four serialization-annotated fields).

**Phantom-input check:** Neither file declares `src/analyzer/arp.rs` in its `inputs:` list.
No phantom-input fix needed (Burst 1 issue does not apply here).

**interface-definitions.md — 3 stale items fixed:**

| Item | Location | Before | After |
|------|----------|--------|-------|
| JSON schema property | ~line 231 (schema properties block) | `"mitre_technique": { "type": "string", ... }` scalar string | `"mitre_techniques": { "type": "array", "items": { "type": "string", "pattern": "..." }, "description": "... omitted when empty (Vec::is_empty) ..." }` |
| timestamp description | ~lines 243-244 | "currently always absent (domain-debt O-01); all emission sites set timestamp: None" | "Packet-derived timestamp in RFC 3339 format. Present when emission site populates it; omitted when None. Domain-debt O-01 is CLOSED; wired STORY-097/098/099." |
| Field-list section | ~line 351 | "All three Option<_> fields ... `mitre_technique: Option<String>` -- omitted when None; ... (always None today per O-01)" | Corrected to one Vec field (Vec::is_empty) + three Option fields (Option::is_none); scalar `mitre_technique` removal noted; O-01 CLOSED noted; src/findings.rs line anchors added |

**nfr-catalog.md — 1 stale item fixed:**

| Item | Location | Before | After |
|------|----------|--------|-------|
| NFR-OBS-010 row | ~line 110 | "ALL four Option fields: `mitre_technique`, `source_ip`, `timestamp`, `direction`"; Status cell "src/findings.rs:132-145 shows all four Option fields" | Three Option fields (source_ip, timestamp, direction) + Vec field (mitre_techniques, Vec::is_empty); line anchor corrected to :148-161; ADR-006 Decision 13 / STORY-100 AC-008 cited; LESSON-P1.02 historical note preserved |

Also updated NFR-to-Module Mapping row for NFR-OBS-010 to describe the post-ADR-006 contract
accurately (Vec::is_empty + Option::is_none; scalar removed).

**Version bumps:**

| Document | Before | After |
|----------|--------|-------|
| `specs/prd-supplements/interface-definitions.md` | 1.0 | 1.1 |
| `specs/prd-supplements/nfr-catalog.md` | 1.6 | 1.7 |

**Story body changes:** None. No BC files touched.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| interface-definitions.md | v1.0→v1.1; 3 stale items fixed (mitre_technique schema property, timestamp description, field-list section) | `.factory/specs/prd-supplements/interface-definitions.md` |
| nfr-catalog.md | v1.6→v1.7; NFR-OBS-010 corrected to Vec+3-Option reality; line anchor :132-145→:148-161; NFR-to-Module Mapping row updated | `.factory/specs/prd-supplements/nfr-catalog.md` |

---

## [arp-f2-pass-14-po-burst-7-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 7 — ss-11 BC bodies `mitre_technique` singular sweep (final BC-body burst)

**Summary:** Final BC-body burst of the ARP-F2 Pass-14 whole-corpus remediation. Sweeps
ss-11 (reporting/output) BC bodies for stale `mitre_technique` (singular, Option<String>)
snippets. Shipped Finding struct is `mitre_techniques: Vec<String>` with three remaining
Option fields: `source_ip`, `timestamp`, `direction`. Shipped CSV header column 6 is
`mitre_techniques` (semicolon-joined; src/reporter/csv.rs). The singular scalar field no
longer exists. Applies stale-vs-history discrimination per ARP-F2 rules: PREVIOUS VERSION
SUMMARY blocks, changelog frontmatter, "field renamed X→Y" contrast prose, and
Refactoring Notes migration paragraphs are HISTORY (preserved). Only current-state
Precondition/Postcondition/EC/Test-Vector snippets presenting the old scalar shape are
STALE (fixed).

**Finding discrimination results:**

| File | Occurrences | STALE fixed | HISTORY preserved | Notes |
|------|-------------|-------------|-------------------|-------|
| BC-2.11.013.md | 2 | 0 | 2 | PREVIOUS VERSION SUMMARY block (v1.5→v1.6): "mitre_technique is None" / "mitre_technique set" — changelog contrast prose; body already uses mitre_techniques throughout |
| BC-2.11.015.md | 3 | 0 | 3 | PREVIOUS VERSION SUMMARY block (v1.5→v1.6): three "mitre_technique = None" / "mitre_technique=None" occurrences — changelog contrast prose; body already uses mitre_techniques throughout |
| BC-2.11.016.md | 6 | 6 | 0 | Precondition 2: `mitre_technique set` → `non-empty mitre_techniques vec`; Postcondition 4: `mitre_technique = None` → `mitre_techniques = vec![]`; EC-003: `mitre_technique = None` → `mitre_techniques = vec![]`; Test Vectors rows 1-3: `mitre_technique="T1036"/"T9999"/None` → `mitre_techniques=["T1036"]/["T9999"]/vec![]` |
| BC-2.11.017.md | 2 | 0 | 2 | PREVIOUS VERSION SUMMARY block (v1.4→v1.5): "mitre_technique=None" / EC-003 contrast — changelog; body already uses mitre_techniques throughout |
| BC-2.11.020.md | 5 | 0 | 5 | PREVIOUS VERSION SUMMARY block (v1.4→v1.5): column rename contrast prose (5 occurrences of mitre_technique as BEFORE state); body (Description, Postconditions, Invariants, ECs, Test Vectors) already uses mitre_techniques throughout |
| BC-2.11.024.md | 6 | 0 | 6 | PREVIOUS VERSION SUMMARY block (v1.3→v1.4): field rename + EC/precondition contrast prose (4 occurrences); Refactoring Notes migration paragraph (1 occurrence: `mitre_technique.as_deref().unwrap_or("")` as the REPLACED expression); Architecture Anchors comment (1 occurrence: `replaces f.mitre_technique.as_deref()` annotation) — all HISTORY; body already uses mitre_techniques throughout |

**Version bumps:**

| BC | Before | After |
|----|--------|-------|
| BC-2.11.016 | 1.4 | 1.5 |
| BC-2.11.013 | 1.7 | unchanged (all HISTORY) |
| BC-2.11.015 | 1.6 | unchanged (all HISTORY) |
| BC-2.11.017 | 1.6 | unchanged (all HISTORY) |
| BC-2.11.020 | 1.6 | unchanged (all HISTORY) |
| BC-2.11.024 | 1.6 | unchanged (all HISTORY) |

**BC-INDEX annotations added (DF-SIBLING-SWEEP-001):**
- BC-2.11.016 — inline comment `<!-- v1.5: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B7 -->`
- BC-INDEX bumped v1.22 → v1.23

**H1 titles:** All unchanged. BC-2.11.016 H1 "MITRE Grouping Expands Per-Finding Line with Em-Dash and Name" does not reference the field name; no H1 change needed.

**Story body changes:** None (constraint: touch only ss-11 BC files + BC-INDEX + spec-changelog).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.11.016 | v1.4→v1.5; 6 stale mitre_technique singular snippets → mitre_techniques Vec form | `.factory/specs/behavioral-contracts/ss-11/BC-2.11.016.md` |
| BC-INDEX | v1.22→v1.23; BC-2.11.016 annotation added | `.factory/specs/behavioral-contracts/BC-INDEX.md` |

---

## [arp-f2-pass-14-po-burst-6-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 6 — ss-07 BC bodies `mitre_technique` singular sweep

**Summary:** Remediates stale `mitre_technique` (singular, Option<String>) postcondition
snippets in ss-07 BC bodies. Shipped Finding struct is `mitre_techniques: Vec<String>`
(ADR-006 / Decision 13, v0.3.0 BREAKING). Applies stale-vs-history discrimination per
ARP-F2 rules: changelog frontmatter, "field renamed X→Y" contrast prose, and grep-pattern
migration notes are HISTORY (preserved). Only postcondition field-listing snippets
presenting the old scalar shape as the current expected shape are STALE (fixed).

**Finding discrimination results:**

| File | Occurrences | STALE fixed | HISTORY preserved | Notes |
|------|-------------|-------------|-------------------|-------|
| BC-2.07.009.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.07.010.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.07.011.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.07.012.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.07.014.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1027")` → `mitre_techniques: vec!["T1027"]` |
| BC-2.07.017.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1027")` → `mitre_techniques: vec!["T1027"]` |
| BC-2.07.019.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1027")` → `mitre_techniques: vec!["T1027"]` |

**Version bumps:**

| BC | Before | After |
|----|--------|-------|
| BC-2.07.009 | 1.2 | 1.3 |
| BC-2.07.010 | 1.2 | 1.3 |
| BC-2.07.011 | 1.3 | 1.4 |
| BC-2.07.012 | 1.4 | 1.5 |
| BC-2.07.014 | 1.2 | 1.3 |
| BC-2.07.017 | 1.3 | 1.4 |
| BC-2.07.019 | 1.3 | 1.4 |

**BC-INDEX annotations added (DF-SIBLING-SWEEP-001):**
- BC-2.07.009 — inline comment `<!-- v1.3: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 -->`
- BC-2.07.010 — inline comment `<!-- v1.3: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 -->`
- BC-2.07.011 — inline comment `<!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 -->`
- BC-2.07.012 — inline comment `<!-- v1.5: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 -->`
- BC-2.07.014 — inline comment `<!-- v1.3: mitre_technique→mitre_techniques vec!["T1027"]; ARP-F2 P14 B6 -->`
- BC-2.07.017 — inline comment `<!-- v1.4: mitre_technique→mitre_techniques vec!["T1027"]; ARP-F2 P14 B6 -->`
- BC-2.07.019 — inline comment `<!-- v1.4: mitre_technique→mitre_techniques vec!["T1027"]; ARP-F2 P14 B6 -->`
- BC-INDEX bumped v1.21 → v1.22

**H1 titles:** All unchanged (postcondition snippet-only fixes; titles not affected).

**Story body changes:** None (constraint: touch only ss-07 BC files + BC-INDEX + spec-changelog).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.07.009 | v1.2→v1.3; mitre_technique None → mitre_techniques vec![] | `.factory/specs/behavioral-contracts/ss-07/BC-2.07.009.md` |
| BC-2.07.010 | v1.2→v1.3; mitre_technique None → mitre_techniques vec![] | `.factory/specs/behavioral-contracts/ss-07/BC-2.07.010.md` |
| BC-2.07.011 | v1.3→v1.4; mitre_technique None → mitre_techniques vec![] | `.factory/specs/behavioral-contracts/ss-07/BC-2.07.011.md` |
| BC-2.07.012 | v1.4→v1.5; mitre_technique None → mitre_techniques vec![] | `.factory/specs/behavioral-contracts/ss-07/BC-2.07.012.md` |
| BC-2.07.014 | v1.2→v1.3; mitre_technique Some("T1027") → mitre_techniques vec!["T1027"] | `.factory/specs/behavioral-contracts/ss-07/BC-2.07.014.md` |
| BC-2.07.017 | v1.3→v1.4; mitre_technique Some("T1027") → mitre_techniques vec!["T1027"] | `.factory/specs/behavioral-contracts/ss-07/BC-2.07.017.md` |
| BC-2.07.019 | v1.3→v1.4; mitre_technique Some("T1027") → mitre_techniques vec!["T1027"] | `.factory/specs/behavioral-contracts/ss-07/BC-2.07.019.md` |
| BC-INDEX | v1.21→v1.22; version annotations added for 7 ss-07 BCs | `.factory/specs/behavioral-contracts/BC-INDEX.md` |

---

## [arp-f2-pass-14-po-burst-5-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 5 — ss-06 + ss-10 BC bodies `mitre_technique` singular sweep

**Summary:** Remediates stale `mitre_technique` (singular, Option<String>) postcondition
and invariant snippets in ss-06 and ss-10 BC bodies. Shipped Finding struct is
`mitre_techniques: Vec<String>` (ADR-006 / Decision 13, v0.3.0 BREAKING). Applies
stale-vs-history discrimination per ARP-F2 rules: changelog frontmatter, PREVIOUS VERSION
SUMMARY blocks, grep-pattern migration notes, and "field renamed X→Y" contrast prose are
HISTORY (preserved). Only postcondition/invariant field-listing snippets presenting the old
scalar shape as the current expected shape are STALE (fixed).

**Finding discrimination results:**

| File | Occurrences | STALE fixed | HISTORY preserved | Notes |
|------|-------------|-------------|-------------------|-------|
| BC-2.06.005.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1083")` → `mitre_techniques: vec!["T1083"]` |
| BC-2.06.006.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1505.003")` → `mitre_techniques: vec!["T1505.003"]` |
| BC-2.06.007.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1046")` → `mitre_techniques: vec!["T1046"]` |
| BC-2.06.008.md | 2 | 2 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]`; Invariant 3 prose updated |
| BC-2.06.009.md | 2 | 2 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]`; Invariant 3 prose updated |
| BC-2.06.010.md | 2 | 2 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]`; Invariant 4 prose updated |
| BC-2.06.011.md | 2 | 2 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]`; Invariant 3 prose updated |
| BC-2.06.014.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1499.002")` → `mitre_techniques: vec!["T1499.002"]` |
| BC-2.10.008.md | 6 | 0 | 6 | All occurrences: PREVIOUS VERSION SUMMARY block (line 41), Invariant 3 old grep pattern (line 97), Documentation line (line 185), Refactoring Notes section (lines 199-200) — all are migration contrast prose or changelog history |

**Version bumps:**

| BC | Before | After |
|----|--------|-------|
| BC-2.06.005 | 1.7 | 1.8 |
| BC-2.06.006 | 1.4 | 1.5 |
| BC-2.06.007 | 1.5 | 1.6 |
| BC-2.06.008 | 1.3 | 1.4 |
| BC-2.06.009 | 1.3 | 1.4 |
| BC-2.06.010 | 1.3 | 1.4 |
| BC-2.06.011 | 1.3 | 1.4 |
| BC-2.06.014 | 1.2 | 1.3 |
| BC-2.10.008 | 1.12 | unchanged (all HISTORY) |

**BC-INDEX annotations added (DF-SIBLING-SWEEP-001):**
- BC-2.06.005 — inline comment `<!-- v1.8: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-2.06.006 — inline comment `<!-- v1.5: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-2.06.007 — inline comment `<!-- v1.6: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-2.06.008 — inline comment `<!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-2.06.009 — inline comment `<!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-2.06.010 — inline comment `<!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-2.06.011 — inline comment `<!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-2.06.014 — inline comment `<!-- v1.3: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5 -->`
- BC-INDEX bumped v1.20 → v1.21

**H1 titles:** All unchanged (snippet-only fix; titles not affected).

**Story body changes:** None (constraint: touch only ss-06/ss-10 BC files + BC-INDEX + spec-changelog).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.06.005 | v1.7→v1.8; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.005.md` |
| BC-2.06.006 | v1.4→v1.5; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.006.md` |
| BC-2.06.007 | v1.5→v1.6; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.007.md` |
| BC-2.06.008 | v1.3→v1.4; mitre_technique postcondition + invariant prose → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.008.md` |
| BC-2.06.009 | v1.3→v1.4; mitre_technique postcondition + invariant prose → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.009.md` |
| BC-2.06.010 | v1.3→v1.4; mitre_technique postcondition + invariant prose → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.010.md` |
| BC-2.06.011 | v1.3→v1.4; mitre_technique postcondition + invariant prose → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.011.md` |
| BC-2.06.014 | v1.2→v1.3; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-06/BC-2.06.014.md` |
| BC-INDEX | v1.20→v1.21; version annotations added for 8 ss-06 BCs | `.factory/specs/behavioral-contracts/BC-INDEX.md` |

---

## [arp-f2-pass-14-po-burst-4-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 4 — ss-04 + ss-09 BC bodies `mitre_technique` singular sweep

**Summary:** Remediates stale `mitre_technique` (singular, Option<String>) postcondition
snippets in ss-04 and ss-09 BC bodies. Shipped Finding struct is
`mitre_techniques: Vec<String>` (ADR-006 / Decision 13, v0.3.0 BREAKING). Applies
stale-vs-history discrimination per ARP-F2 rules: changelog frontmatter, PREVIOUS VERSION
SUMMARY blocks, and "field renamed X→Y" prose are HISTORY (preserved). Only postcondition
field-listing snippets presenting the old scalar shape as the current expected shape are STALE
(fixed).

**Finding discrimination results:**

| File | Occurrences | STALE fixed | HISTORY preserved | Notes |
|------|-------------|-------------|-------------------|-------|
| BC-2.04.018.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1036")` → `mitre_techniques: vec!["T1036"]` |
| BC-2.04.019.md | 1 | 1 | 0 | Postcondition: `mitre_technique: Some("T1036")` → `mitre_techniques: vec!["T1036"]` |
| BC-2.04.020.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.04.021.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.04.023.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.04.025.md | 1 | 1 | 0 | Postcondition: `mitre_technique: None` → `mitre_techniques: vec![]` |
| BC-2.09.001.md | 4 | 0 | 4 | All occurrences in modified: frontmatter and PREVIOUS VERSION SUMMARY block (changelog/history) |
| BC-2.09.006.md | 8 | 0 | 8 | All occurrences in modified: frontmatter, PREVIOUS VERSION SUMMARY block, and "field renamed"/"renames mitre_technique to" contrast prose (changelog/history) |

**Version bumps (ss-04 only; ss-09 no-change):**

| BC | Before | After |
|----|--------|-------|
| BC-2.04.018 | 1.4 | 1.5 |
| BC-2.04.019 | 1.6 | 1.7 |
| BC-2.04.020 | 1.4 | 1.5 |
| BC-2.04.021 | 1.3 | 1.4 |
| BC-2.04.023 | 1.3 | 1.4 |
| BC-2.04.025 | 1.5 | 1.6 |

**BC-INDEX annotations added (DF-SIBLING-SWEEP-001):**
- BC-2.04.018, .019, .020, .021, .023, .025 — inline version comments added
- BC-2.09.001, BC-2.09.006 — already annotated from prior bursts; no change

**H1 titles:** All unchanged (snippet-only fix; titles not affected).

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.04.018 | v1.4→v1.5; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md` |
| BC-2.04.019 | v1.6→v1.7; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-04/BC-2.04.019.md` |
| BC-2.04.020 | v1.4→v1.5; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-04/BC-2.04.020.md` |
| BC-2.04.021 | v1.3→v1.4; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-04/BC-2.04.021.md` |
| BC-2.04.023 | v1.3→v1.4; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-04/BC-2.04.023.md` |
| BC-2.04.025 | v1.5→v1.6; mitre_technique postcondition → plural form | `.factory/specs/behavioral-contracts/ss-04/BC-2.04.025.md` |
| BC-INDEX | Version annotations added for 6 ss-04 BCs | `.factory/specs/behavioral-contracts/BC-INDEX.md` |

---

## [arp-f2-pass-14-po-burst-3-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 3 — ss-14 Modbus BC bodies

**Summary:** Remediates PRODUCT-OWNER bucket findings from ARP-F2 Pass-14 whole-corpus adversarial
remediation targeting ss-14 (Modbus) BC bodies. Covers substantive findings B-01 through B-04 and
D-01-sibling plus snippet sweep of stale singular `mitre_technique` references.

**B-01 (MEDIUM) — BC-2.14.017 MITRE Techniques traceability stale T1692.001 display name:**
Traceability field T1692.001 display name corrected from stale revoked-T0855 form
"Unauthorized Command Message" to canonical ATT&CK v19 name "Unauthorized Message: Command Message".
Verified against BC-2.14.013:~56/187, BC-2.14.014:~28/44, BC-2.14.016 siblings which already used
the correct name. Technique ID T1692.001 unchanged. BC-2.14.017 bumped v2.4→v2.5.

**B-02 (MEDIUM) — BC-2.14.024 MITRE Techniques traceability stale T1692.001 display name:**
Same stale revoked-T0855 form corrected in BC-2.14.024 MITRE Techniques traceability field:
"Unauthorized Command Message" → "Unauthorized Message: Command Message". BC-2.14.024 bumped v2.1→v2.2.

**B-03 (MEDIUM) — BC-2.14.020 Invariant 6 stale SEEDED/EMITTED counts:**
Invariant 6 counts updated from Decision-12-era "SEEDED_TECHNIQUE_IDS (21 total)" and
"EMITTED_IDS (13 total, 7 ICS)" to canonical "SEEDED 25 / EMITTED 17 (7 Enterprise + 10 ICS)"
with forward-declaration note (current src 23/15, target 25/17 via STORY-114), consistent
with BC-2.10.005/007/008 phrasing. T0888 presence in both sets assertion unchanged.

**B-04 (LOW) — BC-2.14.020 Source Evidence path stale SEEDED/EMITTED annotation:**
Source Evidence path for architecture-delta.md §4.3 annotated as Decision-12-era counts
(SEEDED=21, EMITTED=13 at time of Decision 12 — superseded by canonical 25/17 after ARP feature
per BC-2.10.005/008). Historical citation meaning preserved; superseded note added.
BC-2.14.020 bumped v2.1→v2.2.

**D-01-sibling (LOW) — BC-2.14.004 Source Evidence path stale length range annotation:**
Source Evidence path annotation "modbus-tcp-research.md §1 (Length range [2,253]..." corrected
to "[2,254]" matching the H1 title (already correct), Confidence note, and spec definition.
H1 unchanged. BC-2.14.004 bumped v1.0→v1.1.

**SNIPPET SWEEP — All singular `mitre_technique` occurrences in BC-2.14.013 through BC-2.14.020:**
All occurrences audited. All singular `mitre_technique` usages in these files are HISTORY
(changelog entries, prior-version documentation, and regression-guard notes explaining the
Option<String>→Vec<String> rename). No STALE current-state snippets found requiring conversion.
No changes made to BC-2.14.013, BC-2.14.014, BC-2.14.015, BC-2.14.016, BC-2.14.018, BC-2.14.019
for snippet sweep (zero STALE instances). BC-2.14.017:~273 is also HISTORY (ADR-006 migration
prose, not a current-state snippet).

**BC-INDEX SYNC:** Version annotations updated for BC-2.14.004, BC-2.14.017, BC-2.14.020,
BC-2.14.024. No H1 title changes in this burst; BC-INDEX title columns unchanged.

---

## [arp-f2-pass-14-po-burst-2-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 2 — PRD + BC-INDEX + VPs + F1 delta-analysis remediation

**Summary:** Remediates PRODUCT-OWNER bucket findings from ARP-F2 Pass-14 whole-corpus adversarial
remediation covering the PRD index, BC-INDEX, three verification properties, and the ARP F1
delta-analysis frontmatter. Burst 1 (domain-spec/supplements) was applied separately.

**D-01 (HIGH) — PRD §2.14.A BC-2.14.004 row stale length range:**
BC-2.14.004 summary in PRD §2.14.A corrected: "outside [2, 253]" → "outside [2, 254]".
Canonical range per BC-2.14.004 H1, ECs, VP-022:117, and BC-INDEX:344. Length field=254 is valid
(unit-id byte + 253-byte PDU); first invalid value is 255.

**D-02 (LOW) — BC-INDEX:36 stale PRD version reference:**
Status block PRD version note corrected: "(v1.15)" → "(v1.17)" to match actual prd.md version
after D-01 fix.

**VP-007 (STALE field reference in Sub-property B):**
Property Statement Sub-property B: "Finding.mitre_technique" (singular, stale) → "Finding.mitre_techniques"
(plural Vec<String> per ADR-006 Decision 13). Lines 27 and 258 (grep-pattern migration notes
"mitre_technique:Some → mitre_techniques:vec!") are HISTORY — preserved unchanged.

**VP-016 (2 STALE field references in Test Specification):**
Two test code Finding struct initializations corrected:
- "mitre_technique: None" → "mitre_techniques: vec![]"
- "mitre_technique: technique.map(|s| s.to_string())" → "mitre_techniques: technique.map(|s| vec![s.to_string()]).unwrap_or_default()"
Shipped struct uses Vec<String> per ADR-006 Decision 13.

**VP-020 (STALE field name in Property Statement item 3):**
Field list in item 3 corrected: "mitre_technique" → "mitre_techniques (semicolon-joined)" per
BC-2.11.020/024 and the actual CSV header (csv.rs:69).

**C-06 (LOW) — arp-analyzer-delta-analysis.md stale mitre_research_status:**
frontmatter mitre_research_status updated from TBD-pending placeholder to VALIDATION COMPLETE.
Cites mitre-arp-research.md (2026-06-12, Confidence HIGH). T0830 and T1557.002 confirmed active
in ICS ATT&CK v19.1 and Enterprise ATT&CK respectively. A `modified:` list added to frontmatter.
No F1 analytical conclusions altered.

**mitre_technique STALE vs HISTORY classification for prd.md and BC-INDEX:**
All `mitre_technique` (singular) occurrences in prd.md (lines 55, 327, 329, 333, 368) and
BC-INDEX (lines 233, 283) are HISTORY — they appear inside changelog blockquotes or HTML comments
documenting the ADR-006 Decision 13 rename. These were PRESERVED unchanged per stale-vs-history
discrimination rule.

**Version bumps in this burst:**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/prd.md` | 1.16 | 1.17 | D-01: BC-2.14.004 row length range [2,253]→[2,254]; v1.17 delta note added |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.19 | 1.20 | D-02: PRD version note (v1.15)→(v1.17) |
| `specs/verification-properties/vp-007-mitre-technique-id-format.md` | 2.4 | 2.5 | Stale singular field reference in Sub-property B corrected |
| `specs/verification-properties/vp-016-mitre-tactic-grouping-order.md` | 2.1 | 2.2 | 2 stale Finding field references in Test Specification corrected |
| `specs/verification-properties/vp-020-csv-injection-neutralization.md` | 2.0 | 2.1 | Stale field name in Property Statement item 3 corrected |
| `phase-f1-delta-analysis/arp-analyzer-delta-analysis.md` | (none → modified[] added) | — | C-06: mitre_research_status TBD→VALIDATION COMPLETE |

---

## [arp-f2-pass-14-po-burst-1-2026-06-13] — 2026-06-13

### PATCH: ARP-F2 Pass-14 Product-Owner Bucket Burst 1 — Authoritative schema/supplement docs remediation

**Summary:** Remediates PRODUCT-OWNER bucket findings C-01 through C-07 from ARP-F2 Pass-14
whole-corpus adversarial remediation. Covers the authoritative schema/supplement docs
(domain/, prd-supplements/). Architect bucket already applied separately. No BC-body files,
story bodies, PRD indexes, or VPs touched in this burst.

**C-01 (CRITICAL) — cap-09 schema block stale:** `mitre_technique: Option<String>` →
`mitre_techniques: Vec<String>` with `skip_serializing_if = "Vec::is_empty"`. "Four Option
fields" corrected to "three remaining Option fields" (source_ip, timestamp, direction).
"22 sites set timestamp:None (O-01)" framing updated — O-01 is closed; Modbus+DNP3 sites
added by STORY-102..110. Site count updated to "≥22 includes modbus/dnp3 analyzers". BC refs
extended to BC-2.09.001..007. Version (none)→1.1.

**C-02 (CRITICAL) — cap-09 22-authoritative framing + timestamp:None universal claim:**
Resolved inline with C-01 above — emission site inventory framing updated from "22 sites
(authoritative)" to "≥22 (includes modbus/dnp3)"; timestamp:None universal claim replaced
with "O-01 closed, wired STORY-097/098/099 + STORY-102..110".

**C-03 (HIGH) — ent-04 E-26 "all four Option fields":** E-26 schema description updated —
`mitre_techniques: Vec<String>` (not an Option); "all four Option fields" → "three remaining
Option fields"; O-01 closed note added. Version 1.1→1.2.

**C-04 (HIGH) — domain-debt O-01 still shown as OPEN:** O-01 moved from OPEN ITEMS to
RETIRED ITEMS table (closed by STORY-097/098/099 + STORY-102..110, Option A complete).
Version 1.1→1.2.

**cap-10 stale singular (STALE):** CLI --mitre flag section prose "mitre_technique:
Option<String>" → "mitre_techniques: Vec<String> (empty vec → key absent; ADR-006 Decision
13)". Version 1.7→1.8.

**cap-11 CSV header + JSON Option prose (STALE):** CSV header column `mitre_technique` →
`mitre_techniques` (verified csv.rs:69). Added semicolon-join note and EC-015 consumer
guard. "All four Option fields" → "three remaining Option fields". Timestamp field note
updated: O-01 closed. Version 1.1→1.2.

**inv-01 INV-9 singular (STALE):** "Finding.mitre_technique" → "Finding.mitre_techniques".
Version 1.1→1.2.

**C-05 (test-vectors.md) — input-hash: TBD / phantom arp.rs input:** `src/analyzer/arp.rs`
removed from `inputs:` list (does not exist in develop HEAD; forward-reference to STORY-111).
`input-hash: TBD` → `input-hash: N/A` with rationale comment explaining deferral until
STORY-111 lands. Version 1.9→2.0.

**C-05 (12 stale snippets in test-vectors.md):** All 12 `mitre_technique: Some("X")` /
`"mitre_technique":"X"` occurrences converted to plural array form. Lines 279/280 (BC-2.09.006
section) fully replaced to reflect Vec semantics (empty-vec → key absent; singleton → JSON
array; co-attribution → multi-element array). Line 342 (BC-2.11.013) "no mitre_technique" →
"mitre_techniques: vec![] (empty)". Lines 439/445 (Integration 1/2) JSON snippets updated.
Version 1.9→2.0.

**C-05 (error-taxonomy.md) — input-hash: TBD / phantom arp.rs input:** Same fix as
test-vectors.md — `src/analyzer/arp.rs` removed, `input-hash: N/A` with rationale comment.
Version 1.9→2.0.

**C-07 (LOW) — E-ARP-002 awkward prose:** "within the average since window-start within
the 60-second flap window" rewritten to explicit rate formula
`count_in_window / max(1, ts - window_start_ts)` with note that this is an average-rate
detector, not a sliding-window detector. Semantics preserved. Version 1.9→2.0.

**Architect-bucket version bumps recorded (architect burst, not PO):**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/architecture/api-surface.md` | 1.1 | 1.2 | A-01/A-02/A-06 |
| `specs/architecture/purity-boundary-map.md` | 1.1 | 1.2 | A-03/A-06/A-09 |
| `specs/architecture/system-overview.md` | 1.1 | 1.2 | A-04/A-05 |
| `specs/architecture/module-decomposition.md` | 1.4 | 1.5 | A-08 |
| `specs/architecture/dependency-graph.md` | 1.2 | 1.3 | A-07 |
| `specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` | — | modified[] | D-OBS-01 |

**PO-owned documents updated in this burst:**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/domain/capabilities/cap-09-finding-emission.md` | (none) | 1.1 | C-01/C-02: schema block + emission site inventory + O-01 closure |
| `specs/domain/capabilities/cap-10-mitre-mapping.md` | 1.7 | 1.8 | Stale singular `mitre_technique: Option<String>` → plural Vec form in CLI --mitre section |
| `specs/domain/capabilities/cap-11-reporting-output.md` | 1.1 | 1.2 | CSV header mitre_technique→mitre_techniques; JSON "four Option" → "three remaining Option"; timestamp O-01 closed |
| `specs/domain/entities/ent-04-findings-output.md` | 1.1 | 1.2 | C-03: E-26 four→three Option fields; Vec semantics; O-01 closed |
| `specs/domain/domain-debt.md` | 1.1 | 1.2 | C-04: O-01 moved from OPEN to RETIRED table (Option A complete) |
| `specs/domain/invariants/inv-01-core-invariants.md` | 1.1 | 1.2 | INV-9 "Finding.mitre_technique" → "Finding.mitre_techniques" |
| `specs/prd-supplements/test-vectors.md` | 1.9 | 2.0 | C-05: arp.rs removed from inputs, input-hash→N/A; 12 stale snippets converted to plural array form |
| `specs/prd-supplements/error-taxonomy.md` | 1.9 | 2.0 | C-05: arp.rs removed from inputs, input-hash→N/A; C-07: E-ARP-002 prose rewritten for clarity |
| `spec-changelog.md` | — | — | This pass-14 burst-1 entry |

---

## [pass-13-corpus-cleanup-2026-06-13] — 2026-06-13

### PATCH: Pass-13 Corpus Cleanup — BC-2.10.006 anchor + count fix (F-C-P13-001), NFR-OBS-004 seeded/emitted label (F-C-P13-002), STORY-071 variant/seeded count reconciliation (F-C-P13-003), BC-INDEX BC-2.14.016 T0855→T1692.001 annotation (F-D13-001), BC-INDEX inline version annotation refresh (Slice-B), PRD EC-008→EC-002 citation alignment (Slice-D)

**Summary:** Remediates all PRODUCT-OWNER-bucket findings from whole-corpus adversarial Pass 13. No source, architecture docs, VPs, or STATE.md modified.

**F-C-P13-001 (MEDIUM):** BC-2.10.006 was the only SS-10 BC not updated during the F2+Pass-12 sibling sweep (remained at v1.2). Three classes of staleness fixed:
- Architecture Anchors `src/mitre.rs:153` → `src/mitre.rs:179` (live `_ => return None` wildcard arm per live grep).
- Source Evidence Path `src/mitre.rs:153` → `src/mitre.rs:179`.
- Description, Precondition 2, and Invariant 2: "15-entry" / "15 seeded IDs" → "23-entry (current; 25 after STORY-114 — PLANNED)" matching BC-2.10.005/007 PLANNED forward-declaration pattern.
BC-2.10.006 bumped v1.2→v1.3.

**F-C-P13-002 (MEDIUM):** NFR-OBS-004 Target cell read "All 15 seeded technique IDs resolve; no force-fit" — mislabels seeded vs emitted. The test `known_emitted_technique_ids_resolve_in_lookup` tests EMITTED IDs (15 current / 17 after STORY-114), not seeded (23/25). Target corrected to "All 15 emitted technique IDs resolve in lookup (current; 17 after STORY-114 — PLANNED); no force-fit" per BC-2.10.008.
nfr-catalog.md bumped v1.5→v1.6.

**F-C-P13-003 (MEDIUM):** STORY-071 body carried internally-contradictory stale counts: "16 MITRE tactic variants (14 Enterprise + 2 ICS)" in Narrative + "21 seeded technique IDs" — these never co-existed (16-variant era predates 21-seeded; 21-seeded era has 17 variants). Reconciled to current BCs: BC-2.10.004 (17 variants = 14E+3 ICS incl. IcsImpact) and BC-2.10.005 (23 seeded current; 25 PLANNED after STORY-114). Changes: Narrative 16/2ICS/21 → 17/3ICS/23 PLANNED; AC-005/008/009 "16"→"17"; AC-007 ICS positions [14],[15]→[14],[15],[16]+IcsImpact; AC-010/011 test ref 21→23; AC-011 ICS split 10→12 with T1691.001+T0827; AC-014 "21"→"23" + ICS 10→12 + assignments for T1691.001/T0827 added; Tasks 3/4/5/6/8 updated; Architecture Compliance Rules 21/16→23/17. STORY-071 bumped v1.9→v1.10.

**F-D13-001 (MEDIUM):** BC-INDEX:356 BC-2.14.016 annotation embedded stale literal `mitre_techniques: ["T0855","T0836","T0831"]` — contradicting the v2.2 remap note on the same line and the BC body (which uses T1692.001). Fixed: `"T0855"` → `"T1692.001"` in the annotation example. Absorbed into BC-INDEX bump below.

**Slice-B (LOW):** Two BC-INDEX inline version annotations lagged their file versions:
- BC-INDEX:63 (BC-2.02.009) comment "v1.5" → "v1.6" (file is v1.6).
- BC-INDEX:252 (BC-2.10.008) comment "v1.11" → "v1.12" (file is v1.12).
BC-INDEX bumped v1.18→v1.19 (absorbs F-D13-001 + Slice-B).

**Slice-D (LOW):** PRD:138 cited "EC-008 of BC-2.16.008" for the same-second storm-denominator resolution. HS-INDEX:484 already cites "BC-2.16.008 EC-002" (canonical fire-at-50 same-second vector). Aligned PRD to use EC-002 (the canonical cite). PRD bumped v1.15→v1.16.

**Architect-burst changes recorded (do not modify):**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md` | (modified entry) | v2.1 | Architect Pass-13 burst |
| `specs/verification-properties/vp-005-sni-four-way-classification.md` | 2.0 | 2.1 | Architect Pass-13 burst |
| `specs/architecture/verification-architecture.md` | 1.4 | 1.5 | Architect Pass-13 burst |
| `specs/verification-properties/vp-008-decode-packet-no-panic.md` | 2.0 | 2.1 | Architect Pass-13 burst |
| `specs/architecture/ARCH-INDEX.md` | 1.3 | 1.4 | Architect Pass-13 burst |

**PO-owned documents updated:**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/behavioral-contracts/ss-10/BC-2.10.006.md` | 1.2 | 1.3 | F-C-P13-001: line anchor :153→:179; count 15-entry/15 seeded → 23-entry (current; 25 after STORY-114 PLANNED) |
| `specs/prd-supplements/nfr-catalog.md` | 1.5 | 1.6 | F-C-P13-002: NFR-OBS-004 Target "15 seeded" → "15 emitted (current; 17 after STORY-114 PLANNED)" |
| `stories/STORY-071.md` | 1.9 | 1.10 | F-C-P13-003: variant/seeded count reconciliation to 17 variants / 23 seeded throughout body |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.18 | 1.19 | F-D13-001: BC-2.14.016 annotation T0855→T1692.001; Slice-B: BC-2.02.009 v1.5→v1.6, BC-2.10.008 v1.11→v1.12 |
| `specs/prd.md` | 1.15 | 1.16 | Slice-D: BC-2.16.008 citation EC-008→EC-002 (same-second storm denominator) |
| `spec-changelog.md` | — | — | This pass-13 entry |

---

## [pass-12-corpus-debt-cleanup-2026-06-13] — 2026-06-13

### PATCH: Pass-12 Corpus Debt Cleanup — SS-14 BC-INDEX title sync (F-B12-001), SS-15 MITRE count note (F-B12-002), BC-2.16.008 ARP_FLAP_WINDOW_SECS anchor (F-B12-003), stale src/mitre.rs line anchors (F-C-P12-001..005), spec-changelog phantom VP path (O-D12-01)

**Summary:** Remediates all PRODUCT-OWNER-bucket findings from whole-corpus adversarial Pass 12. No source, architecture docs, VPs, or STATE.md modified.

**F-B12-001 (HIGH):** Six SS-14 BC-INDEX rows had H1 ↔ BC-INDEX title desync (enrichments present in H1 but missing from INDEX rows). Per `bc_h1_is_title_source_of_truth` policy (Criterion-75 precedent), INDEX rows updated to match H1s verbatim: BC-2.14.002 (Truncation Safety suffix added), BC-2.14.003 (renamed to ADU/3-Point Gate form), BC-2.14.004 (renamed to ADU/3-Point Gate form), BC-2.14.005 (replaced long parenthetical with em-dash form), BC-2.14.011 (Pending Table Lookup → (Transaction ID, Unit ID) Lookup), BC-2.14.012 (dropped "(Not Evicting)" parenthetical to match H1). Rows 001/006-010/013-025 already matched; no change. BC INDEX bumped 1.17→1.18. No BC file bumps (H1 unchanged; INDEX-only change).

**F-B12-002 (LOW):** SS-15 subsection note (BC-INDEX ~line 383) cited stale MITRE counts "23 seeded / 15 emitted / 8 catalogue-only" without acknowledging the ARP issue #9 raise. Note extended with "(counts current as of issue #8 post-gate; raised to 25 seeded / 17 emitted by issue #9 ARP — see BC-2.10.005/008; PLANNED until STORY-114)". BC-INDEX bump absorbed into same 1.17→1.18 bump.

**F-B12-003 (LOW):** BC-2.16.008 Architecture Anchors omitted `ARP_FLAP_WINDOW_SECS` despite the BC using it in postconditions and edge cases. Anchor added: `src/analyzer/arp.rs — const ARP_FLAP_WINDOW_SECS: u32 = 60` with cross-ref to BC-2.16.004 as authoritative definition. BC-2.16.008 bumped 1.5→1.6.

**F-C-P12-001 (HIGH):** Stale `technique_info :122-156` anchor in three PO-owned files. Live source: technique_info fn@:128, let info=match id@:129, _ => return None@:179, closing }@:182.
- inv-01-core-invariants.md: INV-9 Enforcement line re-anchored :122-156 (:123/:153/:156) → :128-182 (:128/:129/:179/:182). Version field added (no prior version); bumped (none)→1.1.
- nfr-catalog.md: NFR-MNT-009 source column re-anchored :122-156 → :128-182; projections corrected from "160-167" to technique_name@:186-188, technique_tactic@:192-194. Bumped 1.4→1.5.
- STORY-071.md: Architecture Mapping table row for technique_info re-anchored :122-156 → :128-182 (covered in same bump as F-C-P12-002 below).

**F-C-P12-002 (HIGH):** Stale `technique_tactic :166-168` anchor in two files. Live source: technique_tactic@:192-194.
- BC-2.10.007.md: Architecture Anchors + Source Evidence re-anchored :166-168 → :192-194. Bumped 1.6→1.7.
- STORY-071.md: Architecture Mapping table row for technique_tactic re-anchored :166-168 → :192-194; also all_tactics row :95-114 → :100-120 and technique_name row :160-162 → :186-188 swept in same pass. STORY-071 bumped 1.8→1.9.

**F-C-P12-003 (MEDIUM):** BC-2.10.007 lacked PLANNED forward-declaration marker (siblings BC-2.10.005/008 have it). Added to Description section: STORY-114 adds T0830→LateralMovement and T1557.002→CredentialAccess arms; seeded count 23→25 post-STORY-114. Absorbed into same BC-2.10.007 bump 1.6→1.7.

**F-C-P12-004 (LOW):** BC-2.10.003 all_tactics_in_report_order anchor :95-114 (stale; sibling BC-2.10.004 had :100-120 already). Re-anchored to :100-120. BC-2.10.003 bumped 1.3→1.4.

**F-C-P12-005 (LOW):** BC-2.10.001 Display impl anchor :68-90 imprecise. Live: impl fmt::Display block :72-95. Re-anchored to :72-95. BC-2.10.001 bumped 1.2→1.3.

**O-D12-01 (MEDIUM):** spec-changelog ~lines 233 and 251 cited phantom path `vp-016-mitre-tactic-display.md`; actual file is `vp-016-mitre-tactic-grouping-order.md`. Both lines corrected (replace_all). Historical correction only — no VP version change.

**Architect-burst changes recorded (do not modify):**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/architecture/ARCH-INDEX.md` | 1.2 | 1.3 | Architect Pass-12 burst |
| `specs/architecture/tooling-selection.md` | 1.1 | 1.2 | Architect Pass-12 burst |
| `specs/architecture/dependency-graph.md` | 1.1 | 1.2 | Architect Pass-12 burst |
| `specs/verification-properties/vp-007-mitre-technique-id-format.md` | 2.3 | 2.4 | Architect Pass-12 burst |
| `specs/architecture/arp-architecture-delta.md` | §7 v1.10 row-add | v1.10 (no version change) | Architect Pass-12 burst: §7 row added |
| `specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` | — | accepted | Architect Pass-12 burst |
| `specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md` | — | accepted | Architect Pass-12 burst |
| `specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md` | — | accepted | Architect Pass-12 burst |

**PO-owned documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/behavioral-contracts/BC-INDEX.md` | 1.17 → 1.18 | F-B12-001: SS-14 rows 002/003/004/005/011/012 title-synced to H1 verbatim; F-B12-002: SS-15 MITRE count note extended |
| `specs/behavioral-contracts/ss-16/BC-2.16.008.md` | 1.5 → 1.6 | F-B12-003: ARP_FLAP_WINDOW_SECS anchor added to Architecture Anchors |
| `specs/domain/invariants/inv-01-core-invariants.md` | (none) → 1.1 | F-C-P12-001: INV-9 enforcement anchor :122-156 → :128-182; version field introduced |
| `specs/prd-supplements/nfr-catalog.md` | 1.4 → 1.5 | F-C-P12-001: NFR-MNT-009 anchor :122-156 → :128-182; projections 160-167 → :186-188/:192-194 |
| `specs/behavioral-contracts/ss-10/BC-2.10.007.md` | 1.6 → 1.7 | F-C-P12-002: technique_tactic anchor :166-168 → :192-194; F-C-P12-003: PLANNED forward-declaration added |
| `specs/behavioral-contracts/ss-10/BC-2.10.003.md` | 1.3 → 1.4 | F-C-P12-004: all_tactics anchor :95-114 → :100-120 |
| `specs/behavioral-contracts/ss-10/BC-2.10.001.md` | 1.2 → 1.3 | F-C-P12-005: Display impl anchor :68-90 → :72-95 |
| `stories/STORY-071.md` | 1.8 → 1.9 | F-C-P12-001/F-C-P12-002: Architecture Mapping table all four mitre.rs anchors re-anchored (all_tactics :95-114→:100-120; technique_info :122-156→:128-182; technique_name :160-162→:186-188; technique_tactic :166-168→:192-194) |
| `spec-changelog.md` | — | O-D12-01: phantom path vp-016-mitre-tactic-display.md → vp-016-mitre-tactic-grouping-order.md corrected (2 occurrences); this pass-12 entry |

---

## [corpus-consistency-audit-2026-06-13] — 2026-06-13

### PATCH: Corpus Consistency Audit 2026-06-13 — BC-2.16.010 H1 title enrichment (Criterion-75) + BC-INDEX version-suffix removal

**Summary:** Remediates the PRODUCT-OWNER-bucket defects identified in the 2026-06-13 corpus consistency audit. Two defects, two file changes.

PR-1a (Criterion-75): BC-2.16.010 H1 was missing the "(11 Keys)" enrichment that appeared only in downstream indexes. Per the `bc_h1_is_title_source_of_truth` policy, title enrichment must live in the H1, not only in downstream references. H1 updated from "ArpAnalyzer::summarize() Returns AnalysisSummary with Required Keys" to "ArpAnalyzer::summarize() Returns AnalysisSummary with Required Keys (11 Keys)". BC-2.16.010 bumped v1.5→v1.6.

PR-1b: BC-INDEX row for BC-2.16.010 carried a non-standard "; v1.5" version suffix in the title field. Version tokens belong in frontmatter only, not in title fields. The suffix was removed and the title field synced to the new H1 canonical form. BC-INDEX bumped v1.16→v1.17.

Note: The STORY-114 PLANNED code-vs-spec gap (CD-1/CD-2) is intentional and NOT fixed in this audit pass — it is forward-declared with an F4 obligation and is tracked separately.

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/architecture/ARCH-INDEX.md` | 1.1 → 1.2 | Architect burst: SS-04 BC count 54→55, SS-09 BC count 6→7, SS-16 BC count TBD→15 |
| `specs/architecture/module-decomposition.md` | 1.3 → 1.4 | Architect burst: CD-6 prose updated; CD-7 C-24 DNP3 component added |
| `specs/verification-properties/VP-INDEX.md` | 2.0 → 2.1 | Architect burst: VP-023 lifecycle note qualified |
| `specs/verification-properties/vp-007-mitre-technique-id-format.md` | 2.2 → 2.3 | Architect burst: Post-ARP F4 obligation added + CC-003 |
| `specs/module-criticality.md` | 1.1 → 1.2 | Architect burst: C-23 ARP and C-24 DNP3 components added |
| `specs/behavioral-contracts/ss-16/BC-2.16.010.md` | 1.5 → 1.6 | PR-1a: H1 enriched with "(11 Keys)" per Criterion-75 (bc_h1_is_title_source_of_truth) |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.16 → 1.17 | PR-1b: BC-2.16.010 title version-suffix "; v1.5" removed; title synced to new H1 canonical form |
| `spec-changelog.md` | — | This corpus-audit remediation entry |

---

## [arp-f2-pass11-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 11 (ARP analyzer) — E-ARP-004 3-condition escalation, BC-2.16.014 verdict token, test-vectors BC-2.16.010 version citation, PRD BC-2.04.055/BC-2.09.007 registration, PRD §2.9 range note, PRD T0846 enumeration, BC-INDEX PRD version, cap-10 MitreTactic enum line citation

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 11 findings (ARP analyzer pass).
F-C-P11-001: error-taxonomy.md E-ARP-004 escalation rule had only 2 conditions ("rebind_count >= spoof_threshold AND !spoof_high_emitted") but BC-2.16.004 PC1.c requires 3 (the flap-window term). Corrected to "HIGH iff rebind_count >= spoof_threshold AND (timestamp_secs - first_rebind_ts <= ARP_FLAP_WINDOW_SECS) AND !spoof_high_emitted, else MEDIUM (per BC-2.16.004 PC1.c)".
F-B11-L01: BC-2.16.014 Source Evidence carried retired "LOW/Inconclusive" verdict token; corrected to "LOW/Anomaly (confidence: LOW, finding_type: Anomaly)" matching BC-2.16.003 v1.5 normalization.
F-D11-M02: test-vectors.md cited "BC-2.16.010 v1.2" at line ~421; file is v1.5; corrected.
F-D11-H01: PRD §2.4 BC table and §7 RTM both omitted BC-2.04.055 (issue-#100 propagation gap); BC-2.04.055 row added after BC-2.04.054. PRD §2.9 BC table and §7 RTM both omitted BC-2.09.007; BC-2.09.007 row added after BC-2.09.006. BC count claim "all 283 registered" now accurate.
F-D11-M01: PRD §2.9 range note read "through BC-2.09.006.md" excluding BC-2.09.007; updated to "through BC-2.09.007.md (BC-2.09.007 added Feature Mode F2 issue #100)".
O-D11-01: BC-INDEX status line PRD version updated from "(v1.9)" to "(v1.15)" reflecting current PRD version after F-D11-H01 bump; "all 283 registered" claim now accurate.
O-D11-02: PRD §1 (~line 317) catalogued-but-never-emitted primary list enumerated 7 IDs omitting T0846; T0846 added to enumeration (8 IDs total; O-04 canonical).
F-C-P11-002: cap-10 pass-9/10 changelog line citation for the MitreTactic enum section said "lines 80-82"; the ## MitreTactic enum (E-27) header is at line 81 and variant prose spans lines 83-85; corrected to "lines 81-85" in both the pass-9 reason entry and a new pass-11 entry.
Architect-burst change arch-delta v1.9→v1.10 recorded below per spec-changelog responsibility.

**F-C-P11-001 (MEDIUM) — error-taxonomy.md E-ARP-004 escalation rule corrected to 3 conditions:**

E-ARP-004 Notes cell read "Severity = HIGH iff `rebind_count >= spoof_threshold AND !spoof_high_emitted`" — only 2 conditions. BC-2.16.004 PC1.c (Step 3) specifies all 3: `rebind_count >= spoof_threshold AND (timestamp_secs - first_rebind_ts <= ARP_FLAP_WINDOW_SECS) AND spoof_high_emitted == false`. The missing flap-window term means the 2-condition rule would incorrectly upgrade to HIGH even after the 60-second window expires.

- error-taxonomy.md bumped v1.8→v1.9.

**F-B11-L01 (LOW) — BC-2.16.014 Source Evidence verdict token "LOW/Inconclusive" → "LOW/Anomaly":**

Source Evidence Path cell read "D2 GARP confidence LOW/Inconclusive" — "Inconclusive" is a Verdict enum value, not a finding_type. BC-2.16.003 v1.5 normalization (Pass-10 F-D10-L01) established the canonical triple: confidence=LOW, finding_type=Anomaly (no Verdict token). Corrected to "LOW/Anomaly (confidence: LOW, finding_type: Anomaly)".

- BC-2.16.014 bumped v1.4→v1.5.

**F-D11-M02 (MEDIUM) — test-vectors.md BC-2.16.010 version citation "v1.2" → "v1.5":**

ARP-AMB-004 RESOLVED note at line ~421 cited "BC-2.16.010 v1.2"; BC-2.16.010 is at v1.5. Corrected.

- test-vectors.md bumped v1.8→v1.9.

**F-D11-H01 (HIGH) — PRD §2.4 and §2.9 BC tables + §7 RTM add BC-2.04.055 and BC-2.09.007:**

BC-INDEX:36 claimed "all 283 registered" but PRD body had 281 registered IDs: BC-2.04.055 ("StreamHandler::on_data Carries Capture-Relative Timestamp Parameter", P1, SS-04) and BC-2.09.007 ("Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site", P1, SS-09) were missing from both the §2.4/§2.9 index tables and the §7 RTM. Added:
- §2.4: BC-2.04.055 row after BC-2.04.054 (P1, BC-RAS-055)
- §2.9: BC-2.09.007 row after BC-2.09.006 (P1, BC-FND-007)
- §7 RTM: BC-2.04.055 (CAP-04, SS-04, P1, integration) and BC-2.09.007 (CAP-09, SS-09, P1, integration)

- prd.md bumped v1.14→v1.15.

**F-D11-M01 (MEDIUM) — PRD §2.9 range note "through BC-2.09.006.md" → "through BC-2.09.007.md":**

Range note read "through `BC-2.09.006.md`" excluding BC-2.09.007. Updated to "through `BC-2.09.007.md` (BC-2.09.007 added Feature Mode F2 issue #100)". (Same PRD bump as F-D11-H01.)

**O-D11-01 (LOW) — BC-INDEX PRD version "(v1.9)" → "(v1.15)":**

BC-INDEX status line read "UPDATED (v1.9)" — stale from a prior pass. Updated to "(v1.15)" matching PRD current version. The "all 283 registered" claim is now accurate after F-D11-H01.

- BC-INDEX bumped v1.15→v1.16.

**O-D11-02 (LOW) — PRD §1 T0846 added to 7-ID enumeration:**

Line ~317 listed 7 catalogued-but-never-emitted IDs (T1040, T1071, T1071.001, T1071.004, T1573, T1692.002, T0885); T0846 was mentioned in the annotation but not in the primary list. O-04 canonical count is 8. T0846 added between T1573 and T1692.002. (Same PRD bump as F-D11-H01.)

**F-C-P11-002 (LOW) — cap-10 pass-9/10 changelog line citation corrected to "lines 81-85":**

Pass-10 F-C-P10-002 corrected pass-9's "lines 76-77" to "lines 80-82". Pass-11 F-C-P11-002 corrects this further: the `## MitreTactic enum (E-27)` header is at line 81 of cap-10; the variant prose spans lines 83-85. Corrected in the pass-9 reason entry from "lines 80-82" to "lines 81-85". Pass-10 entry updated to reflect subsequent Pass-11 correction.

- cap-10-mitre-mapping.md bumped v1.6→v1.7.

**Architect-burst changes (recorded here per spec-changelog responsibility):**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/architecture/arp-architecture-delta.md` | 1.9 | 1.10 | Architect Pass-11 burst |

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/prd-supplements/error-taxonomy.md` | 1.8 → 1.9 | F-C-P11-001: E-ARP-004 escalation rule 2-condition → 3-condition (flap-window term added) |
| `specs/behavioral-contracts/ss-16/BC-2.16.014.md` | 1.4 → 1.5 | F-B11-L01: Source Evidence "LOW/Inconclusive" verdict token → "LOW/Anomaly (confidence: LOW, finding_type: Anomaly)" |
| `specs/prd-supplements/test-vectors.md` | 1.8 → 1.9 | F-D11-M02: BC-2.16.010 version citation "v1.2" → "v1.5" |
| `specs/prd.md` | 1.14 → 1.15 | F-D11-H01: BC-2.04.055 and BC-2.09.007 rows added to §2.4, §2.9, §7 RTM; F-D11-M01: §2.9 range note; O-D11-02: T0846 added to §1 enumeration |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.15 → 1.16 | O-D11-01: PRD version updated "(v1.9)" → "(v1.15)"; "all 283 registered" claim now accurate |
| `specs/domain/capabilities/cap-10-mitre-mapping.md` | 1.6 → 1.7 | F-C-P11-002: pass-9 changelog line citation "lines 80-82" → "lines 81-85"; pass-10 entry annotated |
| `spec-changelog.md` | — | This pass-11 remediation entry |

---

## [arp-f2-pass10-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 10 (ARP analyzer) — test-vectors subset count, cap-10 line citation, BC-2.10.008 PLANNED qualifier, PRD §2.10 ICS mislabel, BC-INDEX issue-#100 rows, BC-2.02.009 lax-arm wording, BC-2.16.003 verdict normalization, 16→17 MitreTactic stale-count sweep

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 10 findings (ARP analyzer pass).
F-C-P10-001: test-vectors.md SS-10 subset comment said "6 of 25" but the table has 8 Some-returning rows (original 6 + T0830 + T1557.002 added in F2); corrected to "8 of 25".
F-C-P10-002: cap-10 pass-9 changelog reason cited stale "lines 76-77" for the MitreTactic enum description; corrected to "lines 80-82".
F-C-P10-003: BC-2.10.008 Description arp.rs emission bullet was presented without a PLANNED qualifier despite arp.rs not existing in develop HEAD until STORY-114; lead-in updated to distinguish grep-verified (Modbus/DNP3) from PLANNED (arp.rs); arp.rs bullet labelled "PLANNED — STORY-114".
F-D10-M01: PRD §2.10 O-04 note mislabelled T0885 as "(Enterprise)"; T0885 is ICS (CommandAndControl, ICS matrix); T1692.002 is also ICS (IcsImpairProcessControl); both labelled correctly; T0846 ICS label preserved; 12E+13I arithmetic unaffected.
F-D10-M02: BC-INDEX Ingestion-to-L3 Mapping Coverage table omitted BC-2.04.055 and BC-2.09.007 (issue-#100 F2 additions already counted in the 283-total derivation prose); BC-RAS row updated to 55/BC-2.04.001..055; BC-FND row updated to 7/BC-2.09.001..007.
F-D10-M03: BC-2.02.009 Description (~lines 41-42) and Invariants 2-4 incorrectly stated both strict and lax ARP arms are unreachable!; ADR-008 Decision 3 v1.6 specifies: strict_ip_triple NetSlice::Arp = compile-safety unreachable! (ARP routed out before that function is called); lax_ip_triple LaxNetSlice::Arp = explicit routing (NOT unreachable!) because a snaplen-truncated ARP frame reaches lax_ip_triple; explicit routing to extract_arp_frame → Err on bad size, no panic, VP-008/VP-024 Sub-A no-panic preserved. Description, Invariants 2-4, and Architecture Anchors corrected to canonical ADR-008 wording.
F-D10-L01: BC-2.16.003 mixed "LOW/Inconclusive" (Description, Invariant 4, Architecture Anchor) with "LOW/Anomaly" (PC5, EC-001, canonical vectors); PC5 is authoritative (confidence:LOW, finding_type:Anomaly); normalized all occurrences to "LOW/Anomaly (confidence: LOW, finding_type: Anomaly)"; stray "Inconclusive" verdict token removed.
F-D10-L02: Pre-existing DNP3-era drift "16 MitreTactic variants (14 Enterprise + 2 ICS)" stale in three consuming docs; corrected to "17 MitreTactic variants (14 Enterprise + 3 ICS-unique incl. IcsImpact)" in ent-05, cap-11, and nfr-catalog.md; IcsImpact added in Feature #8.
Architect-burst changes ADR-008 v1.7→v1.8 and vp-016 v2.0→v2.1 recorded below per spec-changelog responsibility (product-owner does not modify those files).

**F-C-P10-001 (HIGH) — test-vectors.md SS-10 subset count corrected "6 of 25" → "8 of 25":**

Line ~291 read "Representative subset (6 of 25 total seeded IDs shown" but the table contained 8
Some-returning rows: the original 6 (T1036, T1027, T1083, T1499.002, T1505.003, T1046) plus
T0830 and T1557.002 added in F2 ARP remediation. Fixed to "8 of 25".

- test-vectors.md bumped v1.7→v1.8.

**F-C-P10-002 (LOW) — cap-10 pass-9 changelog line citation corrected "lines 76-77" → "lines 80-82":**

The pass-9 changelog entry in cap-10's `modified` block cited "lines 76-77" as the location of
the MitreTactic enum description section. The actual section begins at line 80 of cap-10
("## MitreTactic enum (E-27)"). Corrected to "lines 80-82" (the section header through the
`#[non_exhaustive]` sentence).

- cap-10-mitre-mapping.md bumped v1.5→v1.6.

**F-C-P10-003 (LOW) — BC-2.10.008 Description arp.rs emission bullet given PLANNED qualifier:**

The Description section (line ~62-73) lead-in read "Emission sites after F2 ARP (verified via
`grep -rn 'mitre_techniques: vec!' src/`)" — presenting all sites including arp.rs as
grep-verified. arp.rs does not exist in develop HEAD; it is created by STORY-114. Changed lead-in
to "Emission sites after F2 ARP (Modbus/DNP3 verified via grep; arp.rs PLANNED STORY-114)"; arp.rs
bullet updated from "(F2 Feature #9 new)" to "(F2 Feature #9 PLANNED — STORY-114)".

- BC-2.10.008 bumped v1.11→v1.12.

**F-D10-M01 (MEDIUM) — PRD §2.10 T0885 mislabel "(Enterprise)" → "(ICS)":**

The O-04 domain-debt note in PRD §2.10 (~lines 613-614) listed the 8 catalogued-but-never-emitted
IDs with "T0885 (Enterprise)". T0885 (Commonly Used Port) is an ICS-matrix technique mapping to
CommandAndControl in the ICS namespace; T1692.002 (Unauthorized Message: Reporting Message) maps
to IcsImpairProcessControl (also ICS). The "(Enterprise)" label broke the 12E+13I seeded-ID split.
Corrected: T1573 labelled "(Enterprise)"; T1692.002 labelled "(ICS — IcsImpairProcessControl)";
T0885 labelled "(ICS — CommandAndControl)"; T0846 retains "(ICS)" label. 12E+13I arithmetic confirmed
unaffected — the labels were cosmetic errors, not count errors.

- prd.md bumped v1.13→v1.14.

**F-D10-M02 (MEDIUM) — BC-INDEX Ingestion-to-L3 table adds BC-2.04.055 and BC-2.09.007:**

The Ingestion-to-L3 Mapping Coverage table omitted the two issue-#100 F2 additions that ARE
already counted in the 283-total derivation prose (line 473: "+ 2 Feature Mode F2 additions
(BC-2.04.055, BC-2.09.007) for issue #100 = 219 active BCs"). The BC-RAS row showed "54 / BC-2.04.001..054"
and BC-FND showed "6 / BC-2.09.001..006". Updated: BC-RAS row → "55 / BC-2.04.001..055 (+ issue-#100 F2)";
BC-FND row → "7 / BC-2.09.001..007 (+ issue-#100 F2)". Total 283 unchanged.

- BC-INDEX bumped v1.14→v1.15.

**F-D10-M03 (MEDIUM) — BC-2.02.009 Description and Invariants corrected for lax-arm routing:**

Description (~lines 41-42) stated "the ARP path exits before `strict_ip_triple` or `lax_ip_triple`
are called (ADR-008 Decision 3: `unreachable!` arms)" — implying both arms are unreachable!.
ADR-008 Decision 3 v1.6 revised this: the strict arm is a compile-safety unreachable! (ARP routed
out before strict_ip_triple); the lax arm MUST NOT be unreachable! (truncated ARP reaches
lax_ip_triple; explicit routing to extract_arp_frame → Err, no panic). Invariant 2 "both unreachable!",
Invariant 3 "lax retry unchanged for ARP", Invariant 4 "LaxNetSlice::Arp unreachable! in lax_ip_triple",
and Architecture Anchor for lax_ip_triple all corrected to ADR-008 canonical wording.

- BC-2.02.009 bumped v1.5→v1.6.

**F-D10-L01 (LOW) — BC-2.16.003 verdict-triple normalized to "LOW/Anomaly":**

"LOW/Inconclusive" appeared in Description, Invariant 4, and Architecture Anchor; "LOW/Anomaly"
appeared in PC5, EC-001, and canonical vectors. PC5 specifies confidence:LOW, finding_type:Anomaly
(no verdict token). "Inconclusive" is the Verdict enum value — not the finding_type axis. Normalized
Description to "LOW/Anomaly (confidence: LOW, finding_type: Anomaly)"; Invariant 4 to "confidence: LOW,
finding_type: Anomaly"; Architecture Anchor to "confidence=LOW, finding_type=Anomaly".

- BC-2.16.003 bumped v1.4→v1.5.

**F-D10-L02 (LOW) — "16 MitreTactic variants (14 Enterprise + 2 ICS)" stale count in 3 docs:**

Pre-existing DNP3-era drift: IcsImpact (3rd ICS-unique variant) was added in Feature #8 (issue #8,
ADR-007), making the correct count 17 variants (14 Enterprise + 3 ICS-unique). Three consuming docs
still said 16/2 ICS: ent-05-enums-value-objects.md (enum table row), cap-11-reporting-output.md
(`all_tactics_in_report_order` sentence), nfr-catalog.md NFR-OBS-008 target column. All corrected to
"17 MitreTactic variants (14 Enterprise + 3 ICS-unique incl. IcsImpact)". nfr-catalog also corrected
"16 tactic headers" to "17 tactic headers" in the NFR-OBS-008 target.

- ent-05-enums-value-objects.md bumped (no prior version) → v1.1.
- cap-11-reporting-output.md bumped (no prior version) → v1.1.
- nfr-catalog.md bumped v1.3→v1.4.
- ent-04-findings-output.md: additional sibling-sweep instance found at E-27 section (~line 61);
  "16-variant enum (14 Enterprise + 2 ICS)" corrected to "17-variant enum (14 Enterprise + 3
  ICS-unique incl. IcsImpact)"; bumped (no prior version) → v1.1.

**Architect-burst changes (recorded here per spec-changelog responsibility):**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.7 | 1.8 | Architect Pass-10 burst |
| `specs/verification-properties/vp-016-mitre-tactic-grouping-order.md` | 2.0 | 2.1 | Architect Pass-10 burst |

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/prd-supplements/test-vectors.md` | 1.7 → 1.8 | F-C-P10-001: "6 of 25" → "8 of 25" in SS-10 BC-2.10.005 subset comment |
| `specs/domain/capabilities/cap-10-mitre-mapping.md` | 1.5 → 1.6 | F-C-P10-002: pass-9 changelog reason line citation "lines 76-77" → "lines 80-82" |
| `specs/behavioral-contracts/ss-10/BC-2.10.008.md` | 1.11 → 1.12 | F-C-P10-003: arp.rs emission bullet PLANNED qualifier added |
| `specs/prd.md` | 1.13 → 1.14 | F-D10-M01: T0885/T1692.002 "(Enterprise)" mislabel corrected to "(ICS)" in §2.10 O-04 note |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.14 → 1.15 | F-D10-M02: BC-RAS row +BC-2.04.055; BC-FND row +BC-2.09.007 |
| `specs/behavioral-contracts/ss-02/BC-2.02.009.md` | 1.5 → 1.6 | F-D10-M03: lax_ip_triple ARP arm Description/Invariants/Anchors corrected to explicit routing (NOT unreachable!) per ADR-008 Decision 3 v1.6+ |
| `specs/behavioral-contracts/ss-16/BC-2.16.003.md` | 1.4 → 1.5 | F-D10-L01: "LOW/Inconclusive" verdict token normalized to "LOW/Anomaly (confidence:LOW, finding_type:Anomaly)" |
| `specs/domain/entities/ent-05-enums-value-objects.md` | (none) → 1.1 | F-D10-L02: MitreTactic (E-27) "16 variants (14 Enterprise + 2 ICS)" → "17 variants (14 Enterprise + 3 ICS-unique incl. IcsImpact)" |
| `specs/domain/capabilities/cap-11-reporting-output.md` | (none) → 1.1 | F-D10-L02: "16 MitreTactic variants" → "17 MitreTactic variants" |
| `specs/prd-supplements/nfr-catalog.md` | 1.3 → 1.4 | F-D10-L02: NFR-OBS-008 target "16 tactic headers" → "17 tactic headers" |
| `specs/domain/entities/ent-04-findings-output.md` | (none) → 1.1 | F-D10-L02 sibling-sweep: E-27 "16-variant enum (14 Enterprise + 2 ICS)" → "17-variant enum (14 Enterprise + 3 ICS-unique incl. IcsImpact)" |
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.7 → 1.8 | Architect Pass-10 burst (product-owner records; do not modify) |
| `specs/verification-properties/vp-016-mitre-tactic-grouping-order.md` | 2.0 → 2.1 | Architect Pass-10 burst (product-owner records; do not modify) |
| `spec-changelog.md` | — | This pass-10 remediation entry |

---

## [arp-f2-pass9-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 9 — cap-10 IcsDiscovery→Discovery (T0846/T0888), test-vectors BC-2.10.004 v1.4→v1.5 + T9999 Canary, BC-2.16.010 EC-003 Frame-vs-Finding Clarification, BC-2.16.003 EC Monotonic Numbering

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 9 findings.
F-C-P9-001: cap-10-mitre-mapping.md tactic column for T0846 and T0888 read "IcsDiscovery" — a
non-existent MitreTactic enum variant; the enum has 3 ICS-unique variants
(IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact) per both src/mitre.rs and
cap-10's own enum description (lines 76-77); corrected to "Discovery". F-C-P9-002: test-vectors.md
line ~293 cited "BC-2.10.004 v1.4" but the file is at v1.5 (bumped in Pass-7 F-C-P7-003);
updated to v1.5. F-B9-M04: BC-2.16.010 EC-003 could be misread as 108 frames (50+50+5+3);
clarifying clause added: the 5 GARP and 3 spoof findings are detection classifications of frames
already counted among the 100 request/reply frames — they are NOT additional frames; reconciliation
invariant counts frames, not findings. F-B9-L01: BC-2.16.003 EC table was non-monotonic
(EC-009 inserted between EC-003 and EC-004); EC-009 moved to end after EC-008; all EC
content and citations unchanged. F-B9-L03 (process-gap): BC-2.02.009 (brownfield SS-02)
intentionally omits inputs:/input-hash: frontmatter — predates the F2 convention; by-design,
not drift. No BC edit. F-C-P9-004: test-vectors.md SS-10 unknown-ID canary changed from
"UNKNOWN999" (happy-path) to "T9999" (edge-case) to align with BC-2.10.005's canonical canary
and mitre.rs Kani verify_unknown_id_returns_none_no_panic harness.
F-B9-M02 confirmation (no edit): BC-2.16.010 key 11 (`malformed_frames`) is defined as
`extract_arp_frame → None` counts only; BC-2.16.009 EC-007 explicitly excludes etherparse-reject
frames (they never reach extract_arp_frame; routed to existing decode error path). Both BCs
already match ADR-008 Decision 7 key 11 narrowing. No changes needed.
ADR-008 v1.6→v1.7 and arch-delta v1.8→v1.9 bumped by architect in this pass (architect burst;
product-owner records here per spec-changelog responsibility).

**F-C-P9-001 (MEDIUM) — cap-10 IcsDiscovery tactic corrected for T0846 and T0888:**

T0846 (Remote System Discovery) and T0888 (Remote System Information Discovery) were mapped to
`IcsDiscovery` in the tactic column. No such variant exists in the `MitreTactic` enum — the enum
has 17 variants: 14 Enterprise ATT&CK tactics plus 3 ICS-unique variants
(`IcsInhibitResponseFunction`, `IcsImpairProcessControl`, `IcsImpact`). The correct tactic for
both T0846 and T0888 is `Discovery` (the Enterprise Discovery tactic). This is confirmed by
src/mitre.rs mapping and cap-10's own enum description in lines 76-77.

- cap-10-mitre-mapping.md bumped v1.4→v1.5.

**F-C-P9-002 (MEDIUM) — test-vectors.md BC-2.10.004 version citation corrected v1.4→v1.5:**

Line ~293 read "BC-2.10.004 v1.4" but the file has been at v1.5 since Pass-7 remediation
F-C-P7-003 (Architecture Anchors and Source Evidence re-anchored). Updated to v1.5.

(Same test-vectors.md bump as F-C-P9-004 below.)

**F-B9-M04 (MEDIUM) — BC-2.16.010 EC-003 frame-vs-finding double-count ambiguity:**

EC-003 described 50 requests, 50 replies, 5 GARPs, 3 spoofs → frames_analyzed=100. A reader
could sum 50+50+5+3=108, contradicting the reconciliation invariant (50+50=100). Clarifying
clause added: "(the 5 GARP and 3 spoof findings are detection classifications of frames already
counted among the 100 request/reply frames — they are NOT additional frames; the reconciliation
invariant counts frames, not findings)."

- BC-2.16.010 bumped v1.4→v1.5.

**F-B9-M02 confirmation (no change) — BC-2.16.010 key 11 and BC-2.16.009 EC-007 etherparse-parse-failure exclusion:**

Verified: BC-2.16.010 Postcondition 1 key `malformed_frames` definition reads "count of ARP
frames with non-Ethernet/IPv4 hw/proto sizes (extract_arp_frame → None)". BC-2.16.009 EC-007
explicitly states: "etherparse rejects the frame entirely (malformed EtherType, truncated
payload) — etherparse returns Err (not ArpPacketSlice); the frame never reaches
extract_arp_frame; handled by existing decode error path (not D11)." Both BCs correctly
exclude etherparse-parse-failure from `malformed_frames` and D11, consistent with ADR-008
Decision 7 key 11 narrowing. No changes needed.

**F-B9-L01 (LOW) — BC-2.16.003 EC table non-monotonic numbering:**

EC-009 (Real RFC 5227 ACD probe) was inserted between EC-003 and EC-004 in a prior pass
(Pass-2 F-B-008 remediation), creating the sequence EC-001, EC-002, EC-003, EC-009, EC-004,
EC-005, EC-006, EC-007, EC-008. Fix: EC-009 moved to end after EC-008, restoring monotonic
order: EC-001..EC-009. All EC content and citations unchanged.

- BC-2.16.003 bumped v1.3→v1.4.

**F-B9-L03 (LOW, process-gap) — BC-2.02.009 missing inputs:/input-hash: governance note:**

BC-2.02.009 (brownfield SS-02) intentionally omits `inputs:` and `input-hash:` frontmatter
fields — it predates the F2 convention; by-design, not drift. No BC edit.

**F-C-P9-004 (LOW) — test-vectors.md SS-10 unknown-ID canary corrected:**

Line ~305 had `technique_name("UNKNOWN999")` with category "happy-path". Aligned to
BC-2.10.005's canonical canary "T9999" and category "edge-case" (matches the mitre.rs Kani
harness `verify_unknown_id_returns_none_no_panic` which uses T9999 as the representative
unknown ID). Notes field updated to reference BC-2.10.006 and the Kani harness.

- test-vectors.md bumped v1.6→v1.7 (same bump as F-C-P9-002 above).

**Architect-burst changes (recorded here per spec-changelog responsibility):**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.6 | 1.7 | Architect Pass-9 burst |
| `specs/architecture/arp-architecture-delta.md` | 1.8 | 1.9 | Architect Pass-9 burst |

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/domain/capabilities/cap-10-mitre-mapping.md` | 1.4 → 1.5 | F-C-P9-001: T0846 and T0888 tactic column IcsDiscovery → Discovery |
| `specs/prd-supplements/test-vectors.md` | 1.6 → 1.7 | F-C-P9-002: BC-2.10.004 citation v1.4→v1.5; F-C-P9-004: UNKNOWN999 → T9999, happy-path → edge-case |
| `specs/behavioral-contracts/ss-16/BC-2.16.010.md` | 1.4 → 1.5 | F-B9-M04: EC-003 clarifying clause added — GARP/spoof findings are classifications of frames already in frames_analyzed |
| `specs/behavioral-contracts/ss-16/BC-2.16.003.md` | 1.3 → 1.4 | F-B9-L01: EC-009 moved from between EC-003/EC-004 to after EC-008 — monotonic EC numbering restored |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.13 → 1.14 | BC-2.16.003 annotation v1.3→v1.4; BC-2.16.010 annotation v1.4→v1.5 |
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.6 → 1.7 | Architect Pass-9 burst (product-owner records; do not modify) |
| `specs/architecture/arp-architecture-delta.md` | 1.8 → 1.9 | Architect Pass-9 burst (product-owner records; do not modify) |
| `spec-changelog.md` | — | This pass-9 remediation entry |

---

## [arp-f2-pass8-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 8 — BC-2.16.005 PC1 Broadcast Exclusion, BC-2.10.002 Line Re-Anchor, PRD §2.2 BC-2.02.009 Title Sync, VP-024 Test-Infrastructure Note, BC-2.16.003 PC6 One-Shot Clarification, BC-2.16.009 PC4 --arp-absent Scope Note, input-hash:TBD Governance Note

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 8 findings.
F-B8-M01: BC-2.16.005 PC1 contradicted Invariant 5 — the precondition required only
"non-zero sender_ip", which broadcast (255.255.255.255) satisfies, implying a binding must
be inserted; Invariant 5 forbids inserting broadcast IPs. PC1 tightened to exclude both
0.0.0.0 and 255.255.255.255. VP-024 test-infrastructure affordances (new_for_test(),
process_arp_for_test(), bindings_snapshot()) documented in BC-2.16.005 Architecture Anchors
per ADR-008 Decision 4 (folds into BC-2.16.005 bump). F-C-P8-M01: BC-2.10.002 Architecture
Anchors and Source Evidence cited stale src/mitre.rs:85-87; verified in develop HEAD: lines
85-88 are Enterprise arms (Collection, CommandAndControl, Exfiltration, Impact); ICS arms are
at 89-91 (IcsInhibitResponseFunction :89, IcsImpairProcessControl :90, IcsImpact :91). Re-anchored
to :89-91. F-D8-M01: PRD §2.2 row for BC-2.02.009 read "Surface No IP layer found error" (stale
v1.4 title); updated to the BC-INDEX/H1 canonical title "Non-IP Non-ARP Frames Return No-IP-Layer
Error; ARP Frames Return DecodedFrame::Arp". F-B8-L01: BC-2.16.003 PC6 self-referential one-shot
phrasing reworded to "exactly one GARP finding is emitted per GARP frame; no cross-frame one-shot
guard (unlike D1/D3)". F-B8-L02: BC-2.16.009 PC4 clarified — --arp-absent sub-clause describes
the unconditional malformed_frames counter behavior, which operates outside the --arp-active gate
of Precondition 4; not a contradiction. F-B8-L03: governance note recorded here (see below) for
input-hash: TBD convention — no per-BC change needed.
ADR-008 v1.5→v1.6, arch-delta v1.7→v1.8, VP-024 v1.3→v1.4 bumped by architect in this pass
(architect burst; product-owner records here per spec-changelog responsibility).

**F-B8-M01 (MEDIUM) — BC-2.16.005 PC1 contradicts Invariant 5 for broadcast sender IP:**

PC1 previously read "a non-zero `sender_ip`", which the broadcast address 255.255.255.255
satisfies (it is non-zero), implying a binding MUST be inserted for it. Invariant 5 explicitly
states that broadcast sender IPs MUST NOT be inserted. Fix: PC1 reworded to "a `sender_ip` that
is neither all-zero (0.0.0.0 / [0,0,0,0]) nor broadcast (255.255.255.255 / [255,255,255,255])";
added note that PC1 applies only to admissible sender IPs per Invariant 5.

VP-024 test-infrastructure note added to Architecture Anchors (ADR-008 Decision 4):
`new_for_test()`, `process_arp_for_test(&frame, ts)`, and `bindings_snapshot()` are
`#[cfg(test)]` affordances declared as ADR-008 Decision 4 extensions and used by the
VP-024 Sub-C proptest. F3/F4 implementers need to know these exist.

- BC-2.16.005 bumped v1.3→v1.4.

**F-C-P8-M01 (MEDIUM) — BC-2.10.002 stale line anchor (85-87 → 89-91):**

Verified in develop HEAD src/mitre.rs:
- Lines 85-88: Enterprise Display arms (Collection :85, CommandAndControl :86,
  Exfiltration :87, Impact :88)
- Line 89: `MitreTactic::IcsInhibitResponseFunction => "Inhibit Response Function"`
- Line 90: `MitreTactic::IcsImpairProcessControl => "Impair Process Control"`
- Line 91: `MitreTactic::IcsImpact => "Impact (ICS)"`

Architecture Anchors and Source Evidence both re-anchored from :85-87 to :89-91 with inline
per-line annotation. BC-2.10.002 Display string for "Impact" (PC3) is correct per spec;
the src/mitre.rs "Impact (ICS)" mismatch is recorded as a STORY-114 F4 obligation by the
architect and is NOT touched here.

- BC-2.10.002 bumped v1.3→v1.4.

**F-D8-M01 (MEDIUM) — PRD §2.2 BC-2.02.009 summary title stale:**

PRD §2.2 SS-02 table row for BC-2.02.009 read "Surface No IP layer found error" (the v1.4
single-postcondition title). BC was revised to v1.5 in F2 ARP delta with a three-way
postcondition; BC-INDEX and the BC H1 both carry the updated title. PRD row updated to match:
"Non-IP Non-ARP Frames Return No-IP-Layer Error; ARP Frames Return DecodedFrame::Arp".

- PRD bumped v1.12→v1.13.

**LOW findings:**

F-B8-L01 (BC-2.16.003 PC6 one-shot reword):

PC6 previously read: "A GARP finding is emitted at most once per unique GARP event (not
deduplicated beyond what the reporting pipeline provides). The one-shot guard for GARP is not
required; GARP findings are emitted on every GARP frame observed (to preserve forensic record
of all occurrences)." The self-referential structure ("at most once … one-shot guard … not
required") was circular and confusing. Reworded to: "Exactly one GARP finding is emitted per
GARP frame; there is no cross-frame one-shot guard for GARP (unlike detections D1 and D3,
which carry per-IP or per-rate deduplication guards)."

- BC-2.16.003 bumped v1.2→v1.3.

F-B8-L02 (BC-2.16.009 PC4 --arp-absent scope note):

PC4 includes a sub-clause describing --arp-absent counter behavior, but PC4 is positioned
within a contract whose Precondition 4 requires "--arp active". This appeared contradictory.
Clarification note added: the --arp-absent sub-clause describes the unconditional
malformed_frames counter behavior, which is separable from the --arp-active analysis gate;
malformed_frames increments unconditionally; malformed_findings increments only under the gate.

- BC-2.16.009 bumped v1.2→v1.3.

F-B8-L03 (process-gap) — input-hash: TBD governance note:

The `input-hash: TBD` value appearing in SS-16 BC frontmatter is the intentional F2 draft
placeholder for greenfield BCs whose inputs are architecture artifacts not yet committed to
develop HEAD. The actual hash is populated by `bin/compute-input-hash --write` at the
factory-artifacts step before Phase-4 entry (per CLAUDE.md §Input Hash Computation). No
per-BC change is needed; this note documents the convention for future maintainers.

**BC-INDEX updated:**
- BC-2.10.002 annotation: no prior inline comment → v1.4 annotation added
- BC-2.16.003 annotation: no prior inline comment → v1.3 annotation added
- BC-2.16.005 annotation: no prior inline comment → v1.4 annotation added
- BC-2.16.009 annotation: no prior inline comment → v1.3 annotation added
- BC-INDEX bumped v1.12→v1.13.

**Architect-burst changes (recorded here per spec-changelog responsibility):**

| Document | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.5 | 1.6 | Architect Pass-8 burst |
| `specs/architecture/arp-architecture-delta.md` | 1.7 | 1.8 | Architect Pass-8 burst |
| `specs/verification-properties/vp-024-arp-parse-safety.md` | 1.3 | 1.4 | Architect Pass-8 burst |

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/behavioral-contracts/ss-16/BC-2.16.005.md` | 1.3 → 1.4 | F-B8-M01: PC1 tightened to exclude 0.0.0.0 and 255.255.255.255; VP-024 Sub-C test-infrastructure affordances note added (ADR-008 Decision 4) |
| `specs/behavioral-contracts/ss-10/BC-2.10.002.md` | 1.3 → 1.4 | F-C-P8-M01: Architecture Anchors and Source Evidence re-anchored from stale :85-87 to verified :89-91 |
| `specs/prd.md` | 1.12 → 1.13 | F-D8-M01: §2.2 BC-2.02.009 row title updated from stale v1.4 title to canonical v1.5 H1/BC-INDEX title |
| `specs/behavioral-contracts/ss-16/BC-2.16.003.md` | 1.2 → 1.3 | F-B8-L01: PC6 one-shot phrasing reworded — exactly one finding per GARP frame; no cross-frame one-shot guard unlike D1/D3 |
| `specs/behavioral-contracts/ss-16/BC-2.16.009.md` | 1.2 → 1.3 | F-B8-L02: PC4 --arp-absent sub-clause scope note added — malformed_frames unconditional vs malformed_findings gated |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.12 → 1.13 | Version annotations added for BC-2.10.002 (v1.4), BC-2.16.003 (v1.3), BC-2.16.005 (v1.4), BC-2.16.009 (v1.3) |
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.5 → 1.6 | Architect Pass-8 burst (product-owner records; do not modify) |
| `specs/architecture/arp-architecture-delta.md` | 1.7 → 1.8 | Architect Pass-8 burst (product-owner records; do not modify) |
| `specs/verification-properties/vp-024-arp-parse-safety.md` | 1.3 → 1.4 | Architect Pass-8 burst (product-owner records; do not modify) |
| `spec-changelog.md` | — | This pass-8 remediation entry |

---

## [arp-f2-pass7-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 7 — MITRE Tactic Anchors for SS-16 BCs, cap-10 Forward-Declaration, test-vectors Attribution Fix, BC-2.10.004 Line Re-Anchor, BC-INDEX Annotation Sync

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 7 findings.
F-B7-H01/F-B7-H02: four MITRE-emitting SS-16 BCs lacked the tactic-anchor cross-reference
specifying which MitreTactic variant each technique ID maps to; tactic-anchor paragraph added
to Invariant 3/4 of each. F-C-P7-001: cap-10-mitre-mapping.md was frozen at the DNP3 era
(23 seeded / 15 emitted), contradicting BC-2.10.004/005/008 by 2 IDs; PLANNED forward-declaration
appended for T0830 and T1557.002. F-C-P7-002: test-vectors.md incorrectly attributed the 25-seeded-ID
count to BC-2.10.004 (which owns the 17-tactic-variant count); attributions separated into two
independent citations. F-C-P7-003: BC-2.10.004 Architecture Anchors and Source Evidence cited
stale `src/mitre.rs:95-114`; re-anchored to verified `src/mitre.rs:100-120` (function lines).
F-D7-M01: BC-INDEX inline version annotations for BC-2.10.005 ("v1.9") and BC-2.10.008 ("v1.10")
were one version behind file state; updated to v1.10 and v1.11 respectively.
ADR-008 bumped — architect Pass-7 burst, F-SA7-HIGH-01.

**F-B7-H01/F-B7-H02 (HIGH) — SS-16 MITRE tactic-anchor cross-reference missing:**

Added the following paragraph to the MITRE tagging invariant in each of the four BCs:

> "Tactic anchors (ADR-008 Decision 6 — merge-by-name policy): T0830 maps to
> `MitreTactic::LateralMovement` and T1557.002 maps to `MitreTactic::CredentialAccess`; the
> F3/STORY-114 implementer wires these in `technique_info`. Normative source: ADR-008 Decision 6."

The instruction is to cite ADR-008 Decision 6 as normative source only, not to duplicate the
full mapping logic. The tactic-anchor paragraph does not repeat the technique-to-tactic
derivation beyond the cross-reference.

- BC-2.16.003 Invariant 3 (MITRE tagging): tactic-anchor paragraph appended. Bumped v1.1→v1.2.
- BC-2.16.004 Invariant 4 (MITRE tagging): tactic-anchor paragraph appended. Bumped v1.4→v1.5.
- BC-2.16.007 Invariant 4 (MITRE tagging): tactic-anchor paragraph appended. Bumped v1.0→v1.1.
- BC-2.16.014 Invariant 4 (MITRE tagging): tactic-anchor paragraph appended. Bumped v1.3→v1.4.

**F-C-P7-001 (MEDIUM) — cap-10-mitre-mapping.md frozen at DNP3 era:**

The cap-10 document was the declared L2 anchor for BC-2.10.004/005/008, but contained only 23 IDs
(pre-ARP state). Since T0830 and T1557.002 are PLANNED (not yet in develop HEAD until STORY-114),
the fix is a PLANNED forward-declaration matching the style used in BC-2.10.005/008:

- Technique catalog: two rows appended with PLANNED STORY-114 annotation:
  T0830 → LateralMovement (*PLANNED STORY-114 (ARP F2); not in develop HEAD until STORY-114*)
  T1557.002 → CredentialAccess (*PLANNED STORY-114 (ARP F2); not in develop HEAD until STORY-114*)
- Emitted count line: "Emitted (15)" → "Emitted (15 current / 17 after STORY-114)"
- Preamble sentence: "23 IDs" → "23 current IDs ... expanding to 25 total after STORY-114"
- BC references line: "23 total" → "25 total after STORY-114; 23 current"
- cap-10 bumped v1.3→v1.4.

**F-C-P7-002 (MEDIUM) — test-vectors.md mis-attributes seeded-ID count to BC-2.10.004:**

Line ~291 said: "Full seeded count is 25 (12 Enterprise + 13 ICS) per BC-2.10.005 v1.10 and
BC-2.10.004 v1.4." BC-2.10.004 owns the 17-tactic-variant count, not the seeded-ID count.

Fixed to: "per BC-2.10.005 v1.10 (25 seeded IDs) and BC-2.10.004 v1.4 (17 tactic variants)"
— the two counts are now attributed to their respective authoritative BCs independently.
test-vectors.md bumped v1.5→v1.6.

**F-C-P7-003 (MEDIUM) — BC-2.10.004 Architecture Anchors + Source Evidence stale line range:**

Previous citations: `src/mitre.rs:95-114`. Verified against develop HEAD src/mitre.rs:
- Line 95: `}` (closing brace of MitreTactic enum — NOT the function)
- Line 100: `pub fn all_tactics_in_report_order() -> &'static [MitreTactic] {` (function declaration)
- Lines 101–119: slice literal (`&[` to `]`)
- Line 120: `}` (function closing brace)

Re-anchored to `src/mitre.rs:100-120` with inline annotation:
"function declaration line 100, slice literal lines 101-119, closing brace line 120".
Both Architecture Anchors and Source Evidence updated.
BC-2.10.004 bumped v1.4→v1.5.

**F-D7-M01 (MEDIUM) — BC-INDEX inline version annotations stale for SS-10:**

- BC-2.10.005 annotation: "v1.9" → "v1.10" (file is at v1.10 per pass-4 remediation)
- BC-2.10.008 annotation: "v1.10" → "v1.11" (file is at v1.11 per pass-4 remediation)
- BC-INDEX bumped v1.11→v1.12.

**ADR-008 bumped v1.4 → v1.5 — F-SA7-HIGH-01:** Decision 4 malformed_frames field doc-comment reworded to conditional equality.

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/behavioral-contracts/ss-16/BC-2.16.003.md` | 1.1 → 1.2 | F-B7-H01/H02: tactic-anchor paragraph added to Invariant 3 (T0830→LateralMovement, T1557.002→CredentialAccess, per ADR-008 Decision 6) |
| `specs/behavioral-contracts/ss-16/BC-2.16.004.md` | 1.4 → 1.5 | F-B7-H01/H02: tactic-anchor paragraph added to Invariant 4 |
| `specs/behavioral-contracts/ss-16/BC-2.16.007.md` | 1.0 → 1.1 | F-B7-H01/H02: tactic-anchor paragraph added to Invariant 4 |
| `specs/behavioral-contracts/ss-16/BC-2.16.014.md` | 1.3 → 1.4 | F-B7-H01/H02: tactic-anchor paragraph added to Invariant 4 |
| `specs/domain/capabilities/cap-10-mitre-mapping.md` | 1.3 → 1.4 | F-C-P7-001: T0830/T1557.002 PLANNED forward-declaration appended; emitted/seeded counts updated to current/after-STORY-114 form; BC references line updated |
| `specs/prd-supplements/test-vectors.md` | 1.5 → 1.6 | F-C-P7-002: BC-2.10.005/BC-2.10.004 attributions separated — seeded-ID count to BC-2.10.005, tactic-variant count to BC-2.10.004 |
| `specs/behavioral-contracts/ss-10/BC-2.10.004.md` | 1.4 → 1.5 | F-C-P7-003: Architecture Anchors and Source Evidence re-anchored from stale :95-114 to verified :100-120 |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.11 → 1.12 | F-D7-M01: BC-2.10.005 annotation v1.9→v1.10; BC-2.10.008 annotation v1.10→v1.11 |
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.4 → 1.5 | F-SA7-HIGH-01 malformed_frames doc-comment conditional-equality fix |
| `spec-changelog.md` | — | This pass-7 remediation entry |

---

## [arp-f2-pass6-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 6 — BindingEntry last_seen_ts Sibling (BC-2.16.005), insert_binding_lru ts-responsibility Note (BC-2.16.005/006), malformed_findings Conditional Equality (BC-2.16.010), E-ARP-003 MITRE Tag, Changelog Ledger Reconciliation

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 6 findings.
F-B6-H01: BC-2.16.005 Architecture Anchors BindingEntry struct was missing `last_seen_ts: u32`
(the pass-4 F-B4-H01 fix reached BC-2.16.004 but not this sibling). F-B6-M01: BC-2.16.005 and
BC-2.16.006 both lacked the ADR-008 Decision 4 normative note on `insert_binding_lru` ts-write
responsibility. F-B6-H02: BC-2.16.010 Invariant 4 (now Invariant 5) ambiguously implied
unconditional equality between malformed_findings and malformed_frames; corrected to conditional
phrasing per ADR-008 Decision 7 key 11 and BC-2.16.009 PC4. F-C-P6-MED-001: E-ARP-003 Notes
lacked the dual MITRE tag statement that its AiTM siblings (E-ARP-004/005) carry; appended per
BC-2.16.007 PC1. F-D6-H1/O-D6-2: spec-changelog ledger reconciliation — pass-5 arch-delta
placeholder replaced with exact 1.5→1.6, VP-024 1.1→1.2 row added (was omitted from pass-5
table entirely); BC-2.16.010 BC-INDEX annotation updated to v1.4.

**F-B6-H01 (HIGH) — BC-2.16.005 BindingEntry missing last_seen_ts:**
- Architecture Anchors BindingEntry struct field list: added `last_seen_ts: u32` as fifth field
  (was 4 fields; ADR-008 Decision 4 and BC-2.16.004:183 both list 5 fields). The struct now reads:
  `{ mac: [u8; 6], rebind_count: u32, first_rebind_ts: Option<u32>, spoof_high_emitted: bool, last_seen_ts: u32 }`.
- BC-2.16.005 bumped v1.2→v1.3.

**F-B6-M01 (MEDIUM) — insert_binding_lru ts-write responsibility normative note:**
- BC-2.16.005 Architecture Anchors `insert_binding_lru` bullet: normative note appended —
  "`insert_binding_lru` has no `ts` parameter; `last_seen_ts` is written by `process_arp` on
  every observation and read by `insert_binding_lru` only during the eviction scan (per ADR-008
  Decision 4)."
- BC-2.16.006 Architecture Anchors `insert_binding_lru` bullet: same normative note appended.
- BC-2.16.005 bumped v1.2→v1.3 (same bump as F-B6-H01 above).
- BC-2.16.006 bumped v1.1→v1.2.

**F-B6-H02 (HIGH) — BC-2.16.010 unconditional malformed_findings == malformed_frames:**
- Invariant 5 added: "`malformed_findings <= malformed_frames`; equality holds only when `--arp`
  is active. When `--arp` is absent, `malformed_frames` still increments (unconditional frame
  counter) but no D11 finding is emitted, so `malformed_findings` remains lower. No invariant or
  test vector may assert unconditional equality between the two counts (per ADR-008 Decision 7
  key 11 and BC-2.16.009 PC4)."
- Existing Invariants 1–4 unchanged. No existing test vector asserts unconditional equality
  (vectors are already --arp-active scenarios where equality holds; the new invariant formalizes
  the conditional scope).
- BC-2.16.010 bumped v1.3→v1.4.

**F-C-P6-MED-001 (MEDIUM) — E-ARP-003 missing MITRE tag statement:**
- E-ARP-003 Notes: appended "MITRE techniques T0830 (Adversary-in-the-Middle, ICS) and T1557.002
  (ARP Cache Poisoning, Enterprise) attached (per BC-2.16.007 PC1)." Matches phrasing of E-ARP-004
  and E-ARP-005. BC-2.16.007 PC1 mandates both techniques on all D12 mismatch findings; the Notes
  field now reflects this.
- error-taxonomy bumped v1.7→v1.8.

**F-D6-H1 (HIGH) + O-D6-2 (LOW) — spec-changelog ledger reconciliation:**
- Pass-5 "Documents updated" table: arch-delta row placeholder "bumped — see architect burst"
  replaced with exact `1.5 → 1.6`. VP-024 row `1.1 → 1.2` added (was omitted entirely from
  pass-5 table). Both were architect-burst changes that pass-5 had not yet resolved.
- BC-INDEX BC-2.16.010 inline annotation "v1.2" updated to "v1.4" (final version after this pass).
- BC-INDEX bumped v1.10→v1.11.
- This pass-6 entry records all files bumped this round with exact versions (see table below).

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.3 → 1.4 | F-B6-H02: Decision 7 key 11 malformed_findings/malformed_frames conditional equality; F-B6-M01: Decision 4 normative note on last_seen_ts write responsibility (architect burst) |
| `specs/verification-properties/vp-024-arp-parse-safety.md` | 1.2 → 1.3 | F-B6-M01: Source Location anchor for insert_binding_lru updated; last_seen_ts write-responsibility note; Sub-D harness skeleton comment updated (architect burst) |
| `specs/architecture/arp-architecture-delta.md` | 1.6 → 1.7 | OBS-2: ARP_STORM_RATE_DEFAULT doc-comment and Decision 5 D3 trigger aligned to average-frames-per-second phrasing (architect burst) |
| `specs/behavioral-contracts/ss-16/BC-2.16.005.md` | 1.2 → 1.3 | F-B6-H01: BindingEntry last_seen_ts added; F-B6-M01: insert_binding_lru ts normative note added |
| `specs/behavioral-contracts/ss-16/BC-2.16.006.md` | 1.1 → 1.2 | F-B6-M01: insert_binding_lru ts normative note added |
| `specs/behavioral-contracts/ss-16/BC-2.16.010.md` | 1.3 → 1.4 | F-B6-H02: Invariant 5 malformed_findings <= malformed_frames conditional phrasing added |
| `specs/prd-supplements/error-taxonomy.md` | 1.7 → 1.8 | F-C-P6-MED-001: E-ARP-003 Notes MITRE T0830/T1557.002 tag appended |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.10 → 1.11 | O-D6-2: BC-2.16.010 inline annotation v1.2 → v1.4 |
| `spec-changelog.md` | — | This pass-6 remediation entry; pass-5 ledger corrections (arch-delta placeholder resolved 1.5→1.6; VP-024 1.1→1.2 row added) |

---

## [arp-f2-pass5-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 5 — Cross-Reference Postcondition Fixes, E-ARP-003 Verdict Triple, SS-10 Test-Vector Catch-Up, BC-2.16.008 Step Sequence + First-Observation Init

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 5 findings.
Cross-reference postcondition numbers in BC-2.16.012 and BC-2.16.014 were mis-cited
following BC-2.16.004 v1.4's Step restructuring. E-ARP-003 verdict triple carried a stray
"Likely" token not present in BC-2.16.007. test-vectors.md SS-10 section was pre-F2 stale
(15 seeded IDs / 16 MitreTactic variants). BC-2.16.008 lacked a first-observation StormCounter
init postcondition, an explicit ordered step sequence, and its Description contradicted Invariant 2.
Also records the architect's arch-delta version bump (unknown at time of writing — see row note).

**F-B5-M01 (MEDIUM) — BC-2.16.012 PC3 mis-citation:**
- BC-2.16.012 Postcondition 3 cited "BC-2.16.004 Postcondition 5" (the flap-window reset) as
  the location of the escalation rule. Corrected to: "BC-2.16.004 Postcondition 1 (Step 3 / 1.c —
  escalation evaluation)" with the full three-condition expression quoted.
- BC-2.16.012 bumped v1.0→v1.1.

**F-B5-M02 (MEDIUM) — BC-2.16.014 PC2 mis-citation:**
- BC-2.16.014 Postcondition 2 cited "BC-2.16.004 Postcondition 1.b" (the first_rebind_ts setter)
  as the location of the three-condition escalation evaluation. Corrected to: "Postcondition 1.c
  (Step 3 — escalation evaluation)".
- BC-2.16.014 bumped v1.2→v1.3.

**F-B5-M03 (MEDIUM) — BC-2.16.014 PC4 mis-citation:**
- BC-2.16.014 Postcondition 4 cited "BC-2.16.004 Postcondition 3" (first_rebind_ts semantics)
  for the rebind_count increment. Corrected to: "BC-2.16.004 Postcondition 1 (Step 1 / 1.a —
  rebind_count increment)".
- BC-2.16.014 bumped v1.2→v1.3 (same bump as F-B5-M02 above).

**F-C5-MED-001 (MEDIUM) — error-taxonomy E-ARP-003 stray "Likely" token:**
- E-ARP-003 Signal Type changed from "Anomaly/Likely/MEDIUM" to "Anomaly/MEDIUM". BC-2.16.007
  specifies `confidence: MEDIUM` only — no "Likely" confidence token. Siblings E-ARP-004/005
  confirm pattern: no "Likely" on D12. Consistent with how E-ARP-004/005 are written.
- error-taxonomy bumped v1.6→v1.7.

**F-D5-M01 (MEDIUM) — test-vectors.md SS-10 section stale (pre-F2 drift):**
- BC-2.10.005 header "All 15 Seeded IDs" → "All 25 Seeded IDs" (subset table retained with note).
- Added representative subset note clarifying 6-of-25 IDs shown; added T0830 and T1557.002 rows
  to the representative table.
- `all_tactics_in_report_order()` vector: "exactly 16 MitreTactic variants" → "exactly 17
  MitreTactic variants". Kill-chain ordering note updated: IcsInhibitResponseFunction,
  IcsImpairProcessControl, IcsImpact [index 16] listed explicitly (IcsImpact added F2 DNP3).
- test-vectors bumped v1.4→v1.5.

**F-B5-L01 (LOW) — BC-2.16.008 missing first-observation StormCounter init postcondition:**
- Step 1 of the new intra-frame ordered sequence specifies: a never-before-seen MAC initializes
  a new StormCounter with `count_in_window=1`, `window_start_ts=timestamp_secs`, `storm_emitted=false`.
  This is symmetric to Invariant 6 (eviction re-init) and makes the postcondition complete.
- BC-2.16.008 bumped v1.4→v1.5.

**F-B5-L02 (LOW) — BC-2.16.008 intra-frame step ordering not pinned:**
- Explicit ordered step sequence added (mirroring BC-2.16.004 Step pattern):
  Step 1 = window-expiry check → reset (count=1, new window_start) or init new MAC or continue;
  Step 2 = increment count_in_window (if Step 1 determined window is active and entry exists);
  Step 3 = evaluate rate after increment.
- Vectors verified arithmetically valid: all 8 existing canonical vectors produce identical
  outcomes under the new Step structure.
- BC-2.16.008 bumped v1.4→v1.5 (same bump as F-B5-L01 above).

**(LOW) BC-2.16.008 Description "sliding window" contradiction:**
- Description "per-MAC sliding window counter" corrected to "per-MAC 60-second flap-window
  counter" to avoid the implicit contradiction with Invariant 2 ("not a sliding-window detector").
  The description was loose terminology; Invariant 2 is the authoritative characterization.
- BC-2.16.008 bumped v1.4→v1.5 (same bump as above).

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/behavioral-contracts/ss-16/BC-2.16.012.md` | 1.0 → 1.1 | F-B5-M01: PC3 citation corrected from "Postcondition 5" to "Postcondition 1 (Step 3 / 1.c — escalation evaluation)"; full three-condition expression quoted |
| `specs/behavioral-contracts/ss-16/BC-2.16.014.md` | 1.2 → 1.3 | F-B5-M02: PC2 citation corrected from "Postcondition 1.b" to "Postcondition 1.c (Step 3)"; F-B5-M03: PC4 citation corrected from "Postcondition 3" to "Postcondition 1 (Step 1 / 1.a)" |
| `specs/behavioral-contracts/ss-16/BC-2.16.008.md` | 1.4 → 1.5 | F-B5-L01: Step 1 init clause for first-observation MAC; F-B5-L02: explicit 3-step ordered sequence; (LOW) Description "sliding window" → "60-second flap-window counter" |
| `specs/prd-supplements/error-taxonomy.md` | 1.6 → 1.7 | F-C5-MED-001: E-ARP-003 "Anomaly/Likely/MEDIUM" → "Anomaly/MEDIUM" |
| `specs/prd-supplements/test-vectors.md` | 1.4 → 1.5 | F-D5-M01: BC-2.10.005 header "15 Seeded" → "25 Seeded"; T0830/T1557.002 rows added to representative subset; all_tactics_in_report_order "16" → "17"; IcsImpact and ICS ordering note added |
| `specs/architecture/arp-architecture-delta.md` | 1.5 → 1.6 | F-SA5 architect pass: §5.0(b) duplicate row targeting src/mitre.rs line 301 deleted; §3.3 D2 GARP confidence cell updated to LOW base / MEDIUM on conflict |
| `specs/verification-properties/vp-024-arp-parse-safety.md` | 1.1 → 1.2 | F-SA5 architect pass: Sub-A negative harness vacuous-satisfiability risk note added; F4 obligation to confirm Ok-arm reachability or restructure with kani::cover! |
| `spec-changelog.md` | — | This pass-5 remediation entry |

---

## [arp-f2-pass4-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 4 — Full Propagation Sweep + BC Body Fixes

**Summary:** Completes propagation of pass-3 Enterprise/ICS split corrections into all
consuming documents (PRD, BC-INDEX inline comments). Remediates all product-owner-routed
F2 adversarial Pass 4 findings: F-D4-C1/C2 (propagation), F-D4-I1/I2 (spec-changelog
completeness, Source Evidence path), F-B4-H01/H02 (BindingEntry last_seen_ts, BC-2.16.013
storm formula), F-B4-M01..M06 (BC-2.16.008 vector pin, EC-011 contrast, BC-2.16.009 PC4,
BC-2.16.004 mac-update timing, BC-2.16.008 eviction re-init), F-C-P4-HIGH-002/003
(BC-2.10.008 parenthetical, PLANNED markers), F-C-P4-MEDIUM-001/002/003 (error-taxonomy
E-ARP-002 sliding-window / Likely, PLANNED NOTE). Also records architect's arch-delta
v1.4→v1.5 and ADR-008 v1.2→v1.3 bumps (F-D4-I1 obligation).
BC-2.02.009 missing input-hash/inputs is intentional (brownfield BC predating input-hash
convention per F-B4-L03); no fabricated inputs added.

**F-D4-C1 (HIGH) — PRD §2.10 O-04 and §6.5 KD-005 Enterprise/ICS split:**
- PRD §2.10 O-04 note (v1.6→v1.9 label): "11 Enterprise + 14 ICS seeded" → "12 Enterprise + 13 ICS seeded"; "6 Enterprise + 11 ICS emitted" → "7 Enterprise + 10 ICS emitted". Authoritative split from BC-2.10.005 v1.9 / BC-2.10.008 v1.10 (pass-3). T1557.002=Enterprise; T0830=ICS. Arithmetic: 12E+13I=25 seeded; 7E+10I=17 emitted; 25−17=8 catalogued-only.
- PRD §6.5 KD-005 BC-2.10.005 row: "11 Enterprise + 14 ICS" → "12 Enterprise + 13 ICS; T0830 [ICS] + T1557.002 [Enterprise] new ARP F2".
- PRD bumped to v1.12.

**F-D4-C2 (HIGH) — BC-INDEX BC-2.10.005 title "(23 Total)" → "(25 Total)":**
- BC-INDEX line for BC-2.10.005: title "(23 Total)" → "(25 Total)"; inline comment refreshed to v1.9 (T0830+T1557.002 added, 12E+13I).
- BC-INDEX line for BC-2.10.008: inline comment refreshed from "v1.8 / 15 total emitted" → v1.10 / 17 emitted; PLANNED forward-declaration in STORY-114 noted.
- BC-INDEX bumped to v1.10.

**F-D4-I1 (MEDIUM) — spec-changelog "Documents updated" tables: BC-INDEX bumps added:**
- pass-2 table: BC-INDEX 1.7→1.8 row added (pass-2 propagated SEEDED=25 to BC-2.10.005 title; previously omitted).
- pass-3 table: BC-INDEX 1.8→1.9 row added (pass-3 corrected "(23 Total)" → "(25 Total)"; previously omitted).
- This pass-4 entry records ALL bumped files including architect's arch-delta v1.4→v1.5 and ADR-008 v1.2→v1.3.

**F-D4-I2 (LOW) — BC-2.10.008 Source Evidence path stale:**
- Source Evidence `src/mitre.rs:123-154` → `src/mitre.rs:128-181`. Matches pass-3-corrected Architecture Anchors and BC-2.10.005 sibling row.

**F-B4-H01 (HIGH) — BC-2.16.004 BindingEntry missing `last_seen_ts`:**
- Architecture Anchors struct field list: added `last_seen_ts: u32` to BindingEntry (per ADR-008 Decision 4; used for LRU eviction heuristic by BC-2.16.006). BC-2.16.004 bumped v1.3→v1.4.

**F-B4-H02 (HIGH) — BC-2.16.013 PC3 retired storm formula:**
- PC3 rewritten: removed independent restatement of `count_in_window / window_duration_secs >= storm_rate` (divide-by-zero risk). Replaced with cross-reference: "per BC-2.16.008 Postcondition 3 / Note 6 formula". BC-2.16.013 bumped v1.0→v1.1.

**F-B4-M01 (MEDIUM) — BC-2.16.008 canonical vector row 4 under-specified:**
- Vector row 4 pinned: "50 frames spanning ts=100 and ts=101" → "25 frames at ts=100 (rate peaks 25/1=25 — no fire), then 25 at ts=101 (count=50, elapsed=1, rate=50/1=50 — fires at 50th frame)". BC-2.16.008 bumped v1.3→v1.4.

**F-B4-M02 (MEDIUM) — BC-2.16.008 EC-011 reader-confusion vs EC-002:**
- EC-011 contrast note added: "Contrast EC-002: the same 50-frame same-second burst fires when window_start_ts equals the burst second (denominator=1); here window_start_ts=100 dilutes denominator to 59. This is the accepted average-since-window-start limitation (Invariant 2)."

**F-B4-M03 (MEDIUM) — BC-2.16.004 mac-update timing not in Step sequence:**
- Postcondition 1 Step 4 added: "`bindings[sender_ip].mac` updated AFTER escalation evaluation and finding emission; occurs exactly once per frame." PC2 updated to cross-reference Step 4.

**F-B4-M04 (MEDIUM) — BC-2.16.010 canonical vector row 2 self-contradictory:**
- Removed "3 additional malformed frames (total 3 malformed frames)" clause from vector row 2 input description. Replaced with "no other-opcode frames". malformed_findings:3/malformed_frames:3 already encodes intent. BC-2.16.010 bumped v1.2→v1.3.

**F-B4-M05 (MEDIUM) — BC-2.16.009 PC4 "increment together" contradiction:**
- PC4 reworded: "When --arp is active, one malformed_findings increment accompanies each malformed_frames increment. When --arp is absent, malformed_frames still increments but no finding is emitted (malformed_findings unchanged), per BC-2.16.010 key 11 and ADR-008 Decision 7." BC-2.16.009 bumped v1.1→v1.2.

**F-B4-M06 (MEDIUM) — BC-2.16.008 storm_counter re-init after LRU eviction unspecified:**
- Invariant 6 added: "When an evicted MAC reappears, a new StormCounter is initialized: count_in_window=1, window_start_ts=timestamp_secs, storm_emitted=false (first-time observation). Analogous to BC-2.16.005 Invariant 4."

**F-C-P4-HIGH-002 (HIGH) — BC-2.10.008 Description parenthetical breakdown:**
- Description reconciliation parenthetical added: "(pre-F2: 6 Enterprise; Modbus F2: 7 ICS; DNP3 F2: +2 ICS [T1691.001, T0827]; ARP F2: +2 = 1 Enterprise [T1557.002] + 1 ICS [T0830]) → 7 Enterprise + 10 ICS = 17". BC-2.10.008 bumped v1.10→v1.11.

**F-C-P4-HIGH-003 / forward-declaration target (MEDIUM) — PLANNED markers augmented:**
- BC-2.10.005 PLANNED marker: "current code 23/15" → "current code 23 seeded / 15 emitted → target 25 seeded / 17 emitted after STORY-114 5-part atomic update". BC-2.10.005 bumped v1.9→v1.10.
- BC-2.10.008 PLANNED marker: same augmentation.

**F-C-P4-MEDIUM-001 (MEDIUM) — error-taxonomy E-ARP-002 "sliding window" + "Likely":**
- E-ARP-002 Notes: "sliding window" → "average since window-start within the 60-second flap window (per BC-2.16.008 Invariant 2; not a sliding-window detector)".
- E-ARP-002 Signal Type: "Anomaly/Likely/MEDIUM" → "Anomaly/MEDIUM" (BC-2.16.008 PC3 has no Likely token). error-taxonomy bumped v1.5→v1.6.

**F-C-P4-MEDIUM-002/003 (MEDIUM) — error-taxonomy ARP rows unshipped (flat present tense):**
- NOTE block added at head of ARP section: "ARP decode + analyzer behavior and T0830/T1557.002 MITRE arms are PLANNED in STORY-111..115 (v0.7.0); not present in current develop HEAD. `technique_name` returns `None` for T0830/T1557.002 until STORY-114."
- E-DEC-003 and E-DEC-004 Notes updated with "PLANNED (STORY-111)" inline markers.

**F-B4-L03 (LOW) — BC-2.02.009 missing input-hash/inputs fields (brownfield BC):**
- Left as-is per instructions. Intentional: brownfield BC predating the input-hash convention. No fabricated inputs added.

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/prd.md` | 1.11 → 1.12 | F-D4-C1: §2.10 O-04 12E+13I seeded / 7E+10I emitted; §6.5 KD-005 split updated; v1.12 delta note |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.9 → 1.10 | F-D4-C2: BC-2.10.005 title "(23 Total)"→"(25 Total)"; BC-2.10.008 inline comment refreshed to v1.10 / 17 emitted |
| `specs/behavioral-contracts/ss-10/BC-2.10.005.md` | 1.9 → 1.10 | F-C-P4-HIGH-003: PLANNED marker augmented with current 23/15 → target 25/17 |
| `specs/behavioral-contracts/ss-10/BC-2.10.008.md` | 1.10 → 1.11 | F-C-P4-HIGH-002: Description parenthetical reconciliation added; F-C-P4-HIGH-003: PLANNED marker augmented; F-D4-I2: Source Evidence path corrected 123-154→128-181 |
| `specs/behavioral-contracts/ss-16/BC-2.16.004.md` | 1.3 → 1.4 | F-B4-H01: BindingEntry last_seen_ts added to Architecture Anchors; F-B4-M03: Step 4 mac-update added to intra-event sequence |
| `specs/behavioral-contracts/ss-16/BC-2.16.008.md` | 1.3 → 1.4 | F-B4-M01: vector row 4 pinned (25@ts=100 + 25@ts=101); F-B4-M02: EC-011 contrast note added; F-B4-M06: Invariant 6 storm counter re-init after eviction |
| `specs/behavioral-contracts/ss-16/BC-2.16.009.md` | 1.1 → 1.2 | F-B4-M05: PC4 "increment together" → conditional on --arp flag |
| `specs/behavioral-contracts/ss-16/BC-2.16.010.md` | 1.2 → 1.3 | F-B4-M04: vector row 2 self-contradiction resolved |
| `specs/behavioral-contracts/ss-16/BC-2.16.013.md` | 1.0 → 1.1 | F-B4-H02: PC3 storm formula → cross-reference to BC-2.16.008 PC3 |
| `specs/prd-supplements/error-taxonomy.md` | 1.5 → 1.6 | F-C-P4-MEDIUM-001: E-ARP-002 "sliding window"→"average-since-window-start"; "Likely" dropped; F-C-P4-MEDIUM-002/003: NOTE block added to ARP section; E-DEC-003/004 PLANNED markers |
| `specs/architecture/arp-architecture-delta.md` | 1.4 → 1.5 | (architect pass; recorded here per F-D4-I1 obligation) |
| `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md` | 1.2 → 1.3 | (architect pass; recorded here per F-D4-I1 obligation) |
| `spec-changelog.md` | — | This pass-4 remediation entry; pass-2 BC-INDEX 1.7→1.8 row added; pass-3 BC-INDEX 1.8→1.9 row added |

---

## [arp-f2-pass3-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 3 (Sliced) Remediation — Storm Wording, PC3/PC4, MITRE Enterprise/ICS Split, Canonical Names, Forward-Declaration Markers

**Summary:** Remediates all product-owner-routed F2 adversarial Pass 3 findings:
F-B03-001 (BC-2.16.008 sustained wording + late-burst EC), F-B03-002 (BC-2.16.004 PC3 over-broad),
F-B03-003/F-D9-H2 (BC-2.16.009 PC4 F3-detail conditional + Invariant 1 re-anchor),
F-B03-004 (BC-2.16.008 boundary vectors), F-C3 (BC-2.10.008 T1557.002 canonical name),
F-C4 (BC-2.10.005+008 Enterprise/ICS split), F-C5 (BC-2.10.005 EC/vector rows for T0830/T1557.002),
F-C6/O-D-1 (BC-2.10.005+008 Architecture Anchors), F-C1(b) (PLANNED forward-declaration markers),
F-D3-H1 (test-vectors.md "1s window"→"60s window").

**F-B03-001 (HIGH) — BC-2.16.008 sustained wording:**
- Description reworded: removed "sustained", added "average ARP frame rate since window_start_ts
  within the 60-second flap window". Rate metric explicitly identified as average-since-window-start,
  not a sustained-rate detector.
- Invariant 2 rewritten: late-burst/window-averaging suppression documented as ACCEPTED v0.7.0
  behavior. Added late-burst example (49 frames ts=100, 50 more at ts=159, rate=99/59≈1.68 < 50
  → no storm finding despite burst).
- EC-011 added: late-burst suppression scenario (ACCEPTED limitation).

**F-B03-002 (MEDIUM) — BC-2.16.004 PC3 over-broad:**
- PC3 amended: "set on the first rebind of a flap window (when first_rebind_ts is None per Step 2);
  not updated on subsequent rebinds within the same window; re-set on the first rebind after a
  window reset per Postcondition 5."

**F-B03-003/F-D9-H2 (HIGH) — BC-2.16.009 PC4 + Invariant 1 re-anchor:**
- PC4: "if a malformed_count field is added; this is an F3 detail" replaced with:
  "`malformed_frames` and `malformed_findings` increment together (one finding per rejected
  frame when --arp active). ARP-AMB-004 RESOLVED in F2." BC-2.16.010 and ADR-008 Decision 7
  cited normatively.
- Invariant 1: re-anchored from "BC-2.16.001 Invariant 4, EC-007, EC-008" (wrong — Invariant 4
  is about extraction agnosticism, not panic-freedom) to: VP-024 Sub-A Kani harness
  `verify_extract_arp_frame_none_on_bad_size` + BC-2.16.001 EC-007/EC-008 + BC-2.16.001/002
  generally.

**F-B03-004 (MEDIUM) — BC-2.16.008 boundary vectors:**
- EC-009 added: ts-window_start_ts==60 (in-window per <= boundary; no reset; rate≈0.83 < 50).
- EC-010 added: ts-window_start_ts==61 (window resets; storm_emitted cleared; count=1).
- Canonical vectors: two boundary rows added (ts=160/window_start=100; ts=161/window_start=100).
- PC2's <= boundary confirmed intentional (consistent with existing EC-004).

**F-C3 (HIGH) — BC-2.10.008 T1557.002 canonical name:**
- EC-017 expected output corrected: Some("ARP Cache Poisoning") → Some("Adversary-in-the-Middle: ARP Cache Poisoning").
- Canonical vector for T1557.002 corrected: same change.
- Authoritative source: arch-delta §5 line ~267 + mitre-arp-research.md §2.

**F-C4 (HIGH) — BC-2.10.005 + BC-2.10.008 Enterprise/ICS split:**
- T1557.002 reclassified Enterprise (sub-technique of T1557 "Adversary-in-the-Middle").
- T0830 confirmed ICS (T08xx prefix, ICS matrix).
- Corrected split: Enterprise 12 (+1 T1557.002), ICS 13 (-1 T1557.002). Total = 25. ✓
- Emitted split: Enterprise 7 (+1 T1557.002), ICS 10 (-1 T1557.002). Total = 17. ✓
- BC-2.10.005: Postcondition 3, Invariant 1, Invariant 3, Description, changelog comment updated.
- BC-2.10.008: Description, Postcondition 1, Invariant 1, changelog updated.

**F-C5 (HIGH) — BC-2.10.005 EC + canonical vectors for T0830/T1557.002:**
- EC-011 added: T0830 → Some("Adversary-in-the-Middle").
- EC-012 added: T1557.002 → Some("Adversary-in-the-Middle: ARP Cache Poisoning").
- Canonical vectors: T0830 and T1557.002 rows added.
- VP table: "All 23 seeded IDs" → "All 25 seeded IDs".

**F-C6/O-D-1 (HIGH/LOW) — Architecture Anchors re-anchored to current mitre.rs:**
- BC-2.10.005: `:122`→`:128` (function declaration), `:123-156`→`:129-181`, T0885 `:152`→`:158`,
  `_ => return None` `:153`→`:179`. Source Evidence range updated to `:128-181`.
- BC-2.10.008: `:123-154` with "all 13 emitted IDs" → `:128` decl + `:129-181` match table
  with "17 emitted IDs"; T0830/T1557.002 noted as PLANNED in STORY-114.

**F-C1(b) — PLANNED forward-declaration markers:**
- BC-2.10.005: PLANNED marker added to Description.
- BC-2.10.008: PLANNED marker added to Description.
- Text: "PLANNED — implemented in STORY-114; current code 23/15. src/mitre.rs remains at
  SEEDED=23/EMITTED=15 until STORY-114 lands the 5-part atomic update;
  vp007_catalog_drift_guard enforces consistency at implementation time."

**F-D3-H1 (HIGH) — test-vectors.md SS-16 storm table "Timing" column:**
- All 4 occurrences of "1s window" replaced with "60s window (same integer second; ts==window_start_ts)".
- Section header cite updated: "EC-008" → "EC-002/EC-008".

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/behavioral-contracts/ss-16/BC-2.16.008.md` | 1.2 → 1.3 | F-B03-001 sustained wording; Invariant 2 late-burst accepted limitation; EC-009/010/011 added; boundary canonical vectors added |
| `specs/behavioral-contracts/ss-16/BC-2.16.004.md` | 1.2 → 1.3 | F-B03-002 PC3 over-broad amended |
| `specs/behavioral-contracts/ss-16/BC-2.16.009.md` | 1.0 → 1.1 | F-B03-003 PC4 F3-conditional removed; normative BC-2.16.010/ADR-008-D7 citation added; Invariant 1 re-anchored to VP-024 Sub-A |
| `specs/behavioral-contracts/ss-10/BC-2.10.008.md` | 1.8 → 1.10 | F-C3 T1557.002 canonical name; F-C4 Enterprise/ICS split (7E+10I); F-C6 line anchors; F-C1(b) PLANNED marker |
| `specs/behavioral-contracts/ss-10/BC-2.10.005.md` | 1.7 → 1.9 | F-C4 Enterprise/ICS split (12E+13I); F-C5 EC-011/012 + vectors for T0830/T1557.002; F-C6 line anchors; F-C1(b) PLANNED marker; VP table 25 seeded IDs |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.8 → 1.9 | BC-2.10.005 row title "(23 Total)" → "(25 Total)" reflecting F-C4 12E+13I split from pass-3 |
| `specs/prd-supplements/test-vectors.md` | 1.3 → 1.4 | F-D3-H1 "1s window"→"60s window (same integer second; ts==window_start_ts)" (4x); section header EC cite updated |
| `spec-changelog.md` | — | This pass-3 remediation entry |

---

## [arp-f2-pass2-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 2 (Sliced) Remediation + ADR-008 Decision 7 Propagation

**Summary:** Propagates ADR-008 Decision 7 canonical 11-key summarize() set (adds
`other_opcode_count`); remediates all product-owner-routed F2 adversarial Pass 2 findings
(F-B-003/004/005/007/008/009, C-CRIT-001, C-IMP-002, F-D-C1/C2/H1/H2/H3/H4/M1/M2, O-D1/D3).
PRD bumped to v1.11. BC-2.16.003 bumped to v1.1. BC-2.16.004/005/008/014 bumped to v1.2.
BC-2.16.010 bumped to v1.2. HS-INDEX bumped to v1.3. spec-changelog updated with RESOLVED
annotations and pass-1 Documents updated table (F-D-H2/H4). error-taxonomy.md E-ARP-002
corrected (O-D3). test-vectors.md ARP-AMB-004 note updated (F-D-H3).

**ADR-008 Decision 7 propagation (canonical 11-key set):**

- **other_opcode_count added (F-B-001/F-B-006/F-D-M2):** BC-2.16.010 updated from 10 to 11
  keys. `other_opcode_count` (frames with operation != 1 and != 2) added as key 4 in the
  canonical order. Reconciliation invariant explicitly stated:
  `request_count + reply_count + other_opcode_count == frames_analyzed` (malformed_frames
  excluded from frames_analyzed entirely). Description, Postconditions, Invariant 1,
  EC-001/003/005, canonical test vectors, and VP table updated.

**F2 adversarial Pass 2 findings:**

- **F-B-003 (HIGH):** BC-2.16.014 Postcondition 2 repaired. The D1 escalation condition
  previously stated only 2 terms. Now reproduces all 3 terms verbatim from BC-2.16.004
  Postcondition 1.b: `rebind_count >= spoof_threshold AND (timestamp_secs - first_rebind_ts
  <= ARP_FLAP_WINDOW_SECS) AND !spoof_high_emitted`.
- **F-B-004 (MEDIUM):** BC-2.16.004 intra-event ordering made explicit. New Steps 1/2/3:
  (1) increment rebind_count; (2) set first_rebind_ts if unset; (3) evaluate 3-term HIGH
  condition. EC-008 (threshold=1 → HIGH on first rebind) updated to show elapsed=0 from
  Step 2 satisfying the window condition.
- **F-B-005 (HIGH):** BC-2.16.008 Postcondition 3 prefaced with "rate is evaluated after
  each frame increment, using timestamp_secs of the frame just processed." The 2-second burst
  vector (row 4) annotated with unambiguous elapsed denominator calculation.
- **F-B-007 (MEDIUM):** BC-2.16.010 test vector row 2 was contradictory (input "0 Malformed"
  but malformed_findings:3). Row 2 rewritten with consistent inputs: 3 malformed frames all
  produce findings; all 11 keys consistent with those inputs.
- **F-B-008 (MEDIUM):** BC-2.16.003 EC-003 label "RFC 5227 probe" dropped for both-zero case
  (sender_ip=0=target_ip is NOT a real RFC 5227 probe). EC-009 added for real RFC 5227 ACD
  probe (sender_ip=0.0.0.0, target_ip=192.0.2.1) → is_gratuitous_arp=false.
- **F-B-009 (MEDIUM):** BC-2.16.005 pins zero/broadcast sender IP admissibility rule:
  0.0.0.0 and 255.255.255.255 are filtered at `process_arp` entry (not inserted into binding
  table). Invariant 5 added. EC-006/007 updated. BC-2.16.004 EC-010 cross-references
  BC-2.16.005 instead of independently deferring.
- **C-CRIT-001/F-D-H1 (CRITICAL):** HS-INDEX ARP seed counts reconciled. Actual row count:
  W40=4, W41=4, W42=7, W43=4, W44=7 → total=26, P0=24, P1=2.
  Frontmatter `arp_waves_40_44` updated 20→26. Summary table updated 22→26 total, 20→24 P0.
  STORY-113 row HS-W42-006 updated: "11 required keys" with reconciliation invariant.
- **C-IMP-002 (MEDIUM):** HS-W43-004 bare SEEDED=25/EMITTED=17 values qualified with
  "after STORY-114 merges" post-impl note.
- **F-D-C1 (CRITICAL):** PRD §2.10 BC-2.10.005 table row updated "23 Total"→"25 Total".
  O-04 domain debt note updated: SEEDED=25, EMITTED=17; §6.5 KD-005 table updated.
- **F-D-C2 (CRITICAL):** PRD F-ARP-O5 note corrected. P1 seeds = HS-W44-001 and HS-W44-003.
  HS-W42-002 and HS-W43-003 are P0 (were incorrectly cited as P1 in the pass-1 note).
- **F-D-H2 (HIGH):** spec-changelog ARP-AMB-003 and ARP-AMB-004 entries annotated with
  "RESOLVED in F2 — see [arp-f2-pass1-remediation-2026-06-12]" in the arp-f2-2026-06-12
  ambiguities section. History preserved; resolution pointer added.
- **F-D-H3 (HIGH):** test-vectors.md ARP-AMB-004 edge case note at ~line 411 updated:
  "RESOLVED in F2: malformed frames excluded from frames_analyzed; counted in separate
  malformed_frames key (BC-2.16.010)", mirroring the ARP-AMB-003 RESOLVED note at ~line 374.
- **F-D-H4 (HIGH):** spec-changelog [arp-f2-pass1-remediation-2026-06-12] entry updated with
  "Documents updated" version table (was missing). Includes test-vectors.md 1.1→1.2 entry.
- **F-D-M1 (MEDIUM):** PRD §2.16 "detects 5 MITRE ATT&CK techniques" corrected to "has 5
  detection types (D1, D2, D3, D11, D12) and emits 2 MITRE techniques (T0830, T1557.002)".
- **F-D-M2:** Covered under F-B-001/F-B-006 above (BC-2.16.010 nine/ten→eleven).
- **O-D1 (LOW):** PRD §2.16 Detection surface GARP bullet prefixed with "D2:".
- **O-D3 (LOW):** error-taxonomy.md E-ARP-002: "exceeds" corrected to "meets or exceeds"
  (BC-2.16.008 uses >= not >).

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/prd.md` | 1.10 → 1.11 | Pass 2 findings; 11-key summarize; §2.10 SEEDED/EMITTED 23→25/15→17; §2.16 5-detection-types/2-MITRE-techniques; D2 label; F-ARP-O5 P1 correction; v1.11 delta note |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.7 → 1.8 | (Note: BC-2.10.005 row title "21 Total" → "25 Total" propagation; reflects pass-2 F-D-C1 SEEDED=25 update already in prd.md; BC-INDEX inline comment refreshed to match) |
| `specs/behavioral-contracts/ss-16/BC-2.16.003.md` | 1.0 → 1.1 | EC-003 both-zero label; EC-009 real RFC5227 probe added |
| `specs/behavioral-contracts/ss-16/BC-2.16.004.md` | 1.1 → 1.2 | Explicit intra-event ordering Steps 1/2/3; EC-008/010 updated |
| `specs/behavioral-contracts/ss-16/BC-2.16.005.md` | 1.1 → 1.2 | Invariant 5 (zero/broadcast filter); EC-006/007 updated |
| `specs/behavioral-contracts/ss-16/BC-2.16.008.md` | 1.1 → 1.2 | Rate evaluated after each frame; 2-second burst vector annotated |
| `specs/behavioral-contracts/ss-16/BC-2.16.010.md` | 1.1 → 1.2 | 11 keys; other_opcode_count added; reconciliation invariant; consistent test vectors |
| `specs/behavioral-contracts/ss-16/BC-2.16.014.md` | 1.1 → 1.2 | Postcondition 2 all 3 terms reproduced |
| `specs/prd-supplements/error-taxonomy.md` | 1.4 → 1.5 | E-ARP-002 "exceeds" → "meets or exceeds" |
| `specs/prd-supplements/test-vectors.md` | 1.2 → 1.3 | ARP-AMB-004 note updated to RESOLVED |
| `holdout-scenarios/HS-INDEX.md` | 1.2 → 1.3 | 26 total/24 P0/2 P1; HS-W42-006 11 keys; HS-W43-004 post-impl qualifier; BC-2.16.010 "11 keys" in BC-2.16.016 note |
| `spec-changelog.md` | — | This entry; ARP-AMB-003/004 RESOLVED annotations; pass-1 Documents updated table |

---

## [arp-f2-pass1-remediation-2026-06-12] — 2026-06-12

### PATCH: F2 Adversarial Pass 1 Remediation + Architect Decision Propagation

**Summary:** Propagates architect decisions from `arp-architecture-delta.md §5-6` and
remediates all product-owner-routed F2 adversarial Pass 1 findings. PRD bumped to v1.10.
No new BCs added. BC-2.16.004, BC-2.16.005, BC-2.16.006, BC-2.16.008, BC-2.16.010,
BC-2.16.014 bumped to v1.1. error-taxonomy.md bumped to v1.4. HS-INDEX bumped to v1.2.

**Architect decision propagation (arch-delta §5-6):**

- **T0830 tactic corrected:** `IcsImpairProcessControl` was incorrect for T0830.
  Canonical mapping per `mitre.rs` merge-by-name convention:
  `T0830` → `MitreTactic::LateralMovement` (ICS lateral movement, TA0109 maps to Enterprise
  Lateral Movement TA0008 in the mitre.rs variant). All PRD, HS-INDEX, and spec-changelog
  occurrences updated.
- **T1557.002 tactic corrected:** `LateralMovement/CredentialAccess` (dual-tactic notation)
  was ambiguous. Canonical mapping: `T1557.002` → `MitreTactic::CredentialAccess` only.
- **BC-2.16.006 eviction claim downgraded:** "evicts the least-recently-accessed entry"
  changed to "evicts the entry with the minimum `last_seen_ts` timestamp (heuristic LRU
  approximation)". VP-024 Sub-D proves only `len <= cap`; no formal LRU ordering proven.
  BTreeMap noted as Kani surrogate only (not production substrate).
- **BC-2.16.005 Architecture Anchor:** `insert_binding_lru` signature updated to use
  `HashMap<[u8;4], BindingEntry>` (production substrate).
- **HS-INDEX waves 40-44 rewritten:** Match arch-delta §6 canonical story decomposition.
  BC-2.16.016 reconciliation: no such BC exists; STORY-115 arch-delta citation maps to
  BC-2.16.010 (storm_findings already a required summarize() key) and BC-2.16.013 (storm
  CLI flag). BC-2.16.014 is GARP-that-conflicts, not storm CLI flag.

**F2 adversarial Pass 1 findings (F-ARP-C2, C3, H5, H6, H7, H8, O1, O4, O5):**

- **F-ARP-C2 (CRITICAL):** PRD §2.16 "GARP-that-conflicts D14 paths" → "GARP-that-conflicts
  (BC-2.16.014) paths". There is no detection "D14".
- **F-ARP-C3 (CRITICAL):** PRD §2.16 VP-024 sub-property labels corrected to match VP-024
  source of truth: Sub-A=extraction; Sub-B=GARP biconditional; Sub-C=binding last-write-wins
  (proptest); Sub-D=MAX_ARP_BINDINGS cap (scaled Kani).
- **F-ARP-H5 (HIGH):** BC-2.16.008 storm-rate formula corrected from
  `count / (elapsed + 1)` to `count / max(1, elapsed)`. EC-001, EC-002, and all canonical
  test vectors made arithmetically valid. ARP-AMB-003 reclassified RESOLVED in F2.
- **F-ARP-H6 (HIGH):** error-taxonomy.md v1.3→v1.4: added E-ARP-004 (D1 spoof finding:
  Anomaly/MEDIUM or HIGH; T0830+T1557.002) and E-ARP-005 (D2 GARP finding: Anomaly/LOW or
  MEDIUM; T0830+T1557.002). E-ARP-001 (D11) verdict triple corrected: "Anomaly/Inconclusive/LOW"
  → "Anomaly/LOW" to match BC-2.16.009.
- **F-ARP-H7 (HIGH):** BC-2.16.010 v1.0→v1.1: `malformed_frames` added as 10th summary key
  (distinct from `malformed_findings`); `frames_analyzed` explicitly defined to exclude
  malformed frames. Invariant 3 updated: no ARP-AMB-004 dependency. ARP-AMB-004 RESOLVED.
- **F-ARP-H8 (HIGH):** BC-2.16.004 v1.0→v1.1: exactly-one-finding-per-rebind rule stated
  explicitly. Severity deterministic: HIGH iff `rebind_count >= spoof_threshold &&
  !spoof_high_emitted`, else MEDIUM. Unconditional "first rebind = MEDIUM" language removed.
  BC-2.16.014 Postcondition 2 aligned. EC-008 aligned (threshold=1 → HIGH on first rebind).
- **F-ARP-O1 (MEDIUM/process-gap):** ARP-AMB-003 and ARP-AMB-004 reclassified RESOLVED in F2
  in PRD v1.10 delta notes and spec-changelog. ARP-AMB-001/002/005/006 remain F3 choices.
- **F-ARP-O4 (LOW):** PRD RTM BC-2.16.004 and BC-2.16.005 verification-method updated to
  "unit+proptest" (VP-024 Sub-C proptest anchors both).
- **F-ARP-O5 (LOW):** HS-INDEX P1 seed count corrected. Previous table said "2 (HS-W42-002,
  HS-W43-003, HS-W44-003)" — three IDs for count=2. Revised wave tables have P1 count = 2
  (HS-W44-001, HS-W44-003); total seeds = 22 (20 P0 + 2 P1).
  (Subsequent Pass 2 remediation further corrected to 26 total / 24 P0 / 2 P1; see
  [arp-f2-pass2-remediation-2026-06-12].)

**Finding F-ARP-O2 (input-hash: TBD):** Not addressed per instructions — release-gate item
resolved at convergence via `bin/compute-input-hash`.

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/prd.md` | 1.9 → 1.10 | Pass 1 remediation: VP-024 labels, GARP-that-conflicts D14 fix, MITRE tactic corrections, ARP-AMB-003/004 RESOLVED, F-ARP-H5..H8/O1/O4/O5 |
| `specs/behavioral-contracts/ss-16/BC-2.16.004.md` | 1.0 → 1.1 | Exactly-one-finding rule; HIGH iff rebind_count >= threshold && !spoof_high_emitted; EC-008 threshold=1 aligned |
| `specs/behavioral-contracts/ss-16/BC-2.16.005.md` | 1.0 → 1.1 | insert_binding_lru signature corrected (HashMap production substrate) |
| `specs/behavioral-contracts/ss-16/BC-2.16.006.md` | 1.0 → 1.1 | Eviction claim downgraded to heuristic LRU; BTreeMap noted as Kani surrogate only |
| `specs/behavioral-contracts/ss-16/BC-2.16.008.md` | 1.0 → 1.1 | Storm-rate formula corrected to count/max(1,elapsed); EC-001/002 arithmetic aligned |
| `specs/behavioral-contracts/ss-16/BC-2.16.010.md` | 1.0 → 1.1 | malformed_frames added as 10th key; frames_analyzed exclusion stated; ARP-AMB-004 RESOLVED |
| `specs/behavioral-contracts/ss-16/BC-2.16.014.md` | 1.0 → 1.1 | EC-008 threshold=1 aligned; Postcondition 2 severity terms aligned |
| `specs/prd-supplements/error-taxonomy.md` | 1.3 → 1.4 | E-ARP-004 (D1 spoof), E-ARP-005 (D2 GARP) added; E-ARP-001 verdict triple corrected |
| `specs/prd-supplements/test-vectors.md` | 1.1 → 1.2 | ARP-AMB-003 RESOLVED note added; storm-rate vectors made arithmetically consistent |
| `holdout-scenarios/HS-INDEX.md` | 1.1 → 1.2 | Waves 40-44 rewritten per arch-delta §6 canonical order; T0830/T1557.002 tactic corrections |
| `spec-changelog.md` | — | This entry |

---

## [arp-f2-2026-06-12] — 2026-06-12

### MINOR + BREAKING-DECODER: Feature #9 ARP Security Analyzer (SS-16, v0.7.0)

**Summary:** 15 new behavioral contracts (BC-2.16.001..015) covering the ARP security analyzer
(SS-16, C-23 ArpAnalyzer). BC-2.02.009 revised v1.4→v1.5 (ADR-008 Decision 1: three-way
postcondition). This is a MINOR addition at the spec level (no existing BC retired; no existing
interface key removed), but the decoder change constitutes a BREAKING CHANGE at the Rust type
level: `decode_packet` return type changes from `Result<ParsedPacket>` to `Result<DecodedFrame>`.
All consumers of `decode_packet` (main.rs analysis loop, cargo-fuzz VP-008 harness) must be
updated in STORY-111.

**New BCs added (15):**

| BC ID | Title | Group |
|-------|-------|-------|
| BC-2.16.001 | ARP Request Frame Correctly Parsed from ArpPacketSlice | A — extraction |
| BC-2.16.002 | ARP Reply Frame Correctly Parsed from ArpPacketSlice | A — extraction |
| BC-2.16.003 | Gratuitous ARP Detection — sender_ip == target_ip | B — binding/detect |
| BC-2.16.004 | ARP Spoof Detection — IP→MAC Rebind MEDIUM→HIGH | B — binding/detect |
| BC-2.16.005 | Binding-Table Update — Last-Seen MAC Wins | B — binding/detect |
| BC-2.16.006 | Binding-Table Cap — MAX_ARP_BINDINGS=65,536 via LRU | B — resource |
| BC-2.16.007 | D12 L2/L3 Sender Mismatch | C — mismatch |
| BC-2.16.008 | D3 ARP Storm Rate Detection | D — storm |
| BC-2.16.009 | D11 Malformed ARP — Non-Ethernet/IPv4 Sizes → LOW Finding | E — malformed |
| BC-2.16.010 | ArpAnalyzer::summarize() Required Keys | F — summary |
| BC-2.16.011 | --arp CLI Flag Gates ARP Analysis | G — CLI |
| BC-2.16.012 | --arp-spoof-threshold Overrides Default | G — CLI |
| BC-2.16.013 | --arp-storm-rate Overrides Default | G — CLI |
| BC-2.16.014 | GARP-That-Conflicts Upgrades to MEDIUM + D1 Spoof Finding | H — escalation |
| BC-2.16.015 | Decode-vs-Analysis Separation — DecodedFrame::Arp Always Produced | I — invariant |

**BC revised (1):**

- **BC-2.02.009 v1.4 → v1.5** (ADR-008 Decision 1): `decode_packet` return type changes from
  `Result<ParsedPacket>` to `Result<DecodedFrame>`. Three-way postcondition:
  - Path 1 (new): Ethernet/IPv4 ARP frame → `Ok(DecodedFrame::Arp(ArpFrame))`.
  - Path 2 (new): Non-Ethernet/IPv4 ARP frame → `Err("Non-Ethernet/IPv4 ARP frame")` (E-DEC-004).
  - Path 3 (unchanged): Non-IP non-ARP frame → `Err("No IP layer found")` (E-DEC-003).
  Previous behavior (ARP frames returning `Err("No IP layer found")`) is retired.
  VP-008 cargo-fuzz harness update required (accept `Result<DecodedFrame>`).

**MITRE catalog changes:**

Two new techniques enter the seeded catalog (first use in SS-16 ARP analyzer):
- **T0830** — Adversary-in-the-Middle (`MitreTactic::LateralMovement`): emitted by D1 (spoof)
  and D12 (L2/L3 mismatch) detection paths. (Note: earlier drafts incorrectly listed tactic
  as IcsImpairProcessControl; corrected in arp-f2-pass1-remediation-2026-06-12.)
- **T1557.002** — ARP Cache Poisoning (`MitreTactic::CredentialAccess`): co-emitted on
  all spoof findings alongside T0830. (Note: earlier drafts listed LateralMovement/CredentialAccess
  dual notation; canonical mapping is CredentialAccess only.)

Updated counts: **SEEDED=25** (was 23), **EMITTED=17** (was 15), **CATALOGUE-ONLY=8** (unchanged).
BC-2.10.005 and BC-2.10.008 must be updated by story-writer to reflect the new seeded IDs.

**Error taxonomy changes (error-taxonomy.md v1.2 → v1.3):**

- New error code **E-DEC-004** (`Decoder`, `degraded`): "Non-Ethernet/IPv4 ARP frame" — anyhow
  error returned by `decode_packet` Path 2; counted as skipped packet.
- New ARP error section added: **E-ARP-001** (D11 malformed finding), **E-ARP-002** (D3 storm
  finding), **E-ARP-003** (D12 mismatch finding).
- New category `ARP` added to the category table.

**CLI surface changes (to be implemented in STORY-115):**

- `--arp` flag added to `analyze` subcommand (boolean, default false; NOT included in `--all`).
- `--arp-spoof-threshold N` flag (u32, default 3 rebinds within 60s; BC-2.16.012).
- `--arp-storm-rate N` flag (u32, default 50 frames/sec; BC-2.16.013).
- `needs_reassembly` expression unchanged (ARP does not require TCP stream reassembly).

**Decoder changes (STORY-111):**

- New `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }` in `src/decoder.rs`.
- New `pub struct ArpFrame { operation, sender_mac, sender_ip, target_mac, target_ip, outer_src_mac, packet_len }` in `src/decoder.rs`.
- New `fn extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>` in `src/decoder.rs`.
- `unreachable!` arms added to `strict_ip_triple` and `lax_ip_triple` for `NetSlice::Arp` /
  `LaxNetSlice::Arp` (ADR-008 Decision 3).
- etherparse version bumped to 0.20 in `Cargo.toml` (from 0.16) to access `NetSlice::Arp`.

**Formal verification (VP-024):**

Four sub-properties verified (per VP-024 source of truth):
- Sub-A: `extract_arp_frame` parse safety — no-panic; field-copy correctness (Request +
  Reply paths); None for non-Ethernet/IPv4 inputs. (Kani, anchors BC-2.16.001/002)
- Sub-B: GARP detection totality — `is_gratuitous_arp` biconditional (sender_ip==target_ip),
  opcode-agnostic over all 65,536 u16 operation values. (Kani, anchors BC-2.16.003)
- Sub-C: Binding-table last-write-wins determinism — arbitrary Vec<ArpFrame> sequences;
  bindings[ip].mac equals last-frame MAC; no duplicate keys. (proptest, anchors BC-2.16.004/005)
- Sub-D: MAX_ARP_BINDINGS cap — `bindings.len()` never exceeds cap; LRU evicts one entry
  on overflow. (Kani scaled: TEST_MAX_ARP_BINDINGS=8; `#[kani::unwind(12)]`. Anchors BC-2.16.006)
(Note: earlier draft incorrectly labeled Sub-B=Reply extraction and Sub-C=GARP; corrected
in arp-f2-pass1-remediation-2026-06-12 per VP-024 source of truth.)

**F3 implementation ambiguities (record only — not spec defects):**

These are F3 story-writer and implementer choices, not spec gaps. Recorded here so F3 inherits
them without requiring re-discovery:

- **ARP-AMB-001:** LRU substrate for binding table (indexmap-based HashMap LRU vs BTreeMap vs
  custom doubly-linked list). BC-2.16.006 specifies cap invariant only. F3 story must pin.
- **ARP-AMB-002:** Malformed-frame integration mechanism — whether D11 finding is emitted
  inside `decode_packet` (decoder), inside `ArpAnalyzer::process_arp` (analyzer), or via a
  separate hook. BC-2.16.009 and BC-2.02.009 are silent on call site. F3 STORY-111 must decide.
- **ARP-AMB-003:** Sub-second rate denominator for storm detection — EC-008 specifies count/1
  when `ts == window_start_ts`; the formula for frames spanning <1s within the window is
  unspecified. F3 story must define clamping.
  **RESOLVED in F2 — see [arp-f2-pass1-remediation-2026-06-12].** Formula is
  `rate = count_in_window / max(1, ts - window_start_ts)` (integer-seconds, no sub-second
  ambiguity). BC-2.16.008 updated; ARP-AMB-003 closed.
- **ARP-AMB-004:** Whether malformed frames (extract_arp_frame → None) count toward
  `frames_analyzed` in summarize(). BC-2.16.010 is silent. F3 STORY-111 must decide.
  **RESOLVED in F2 — see [arp-f2-pass1-remediation-2026-06-12].** Malformed frames excluded
  from `frames_analyzed`; tracked separately in `malformed_frames` key. BC-2.16.010 updated;
  ARP-AMB-004 closed.
- **ARP-AMB-005:** Stale line-number anchors in BC-2.02.009 Architecture Anchors — will be
  invalidated by STORY-111's DecodedFrame addition. F3 story-writer must update after implementation.
- **ARP-AMB-006:** Stories STORY-111..STORY-115 (estimated waves 40-44) have TBD Story Anchor
  in all SS-16 BCs. F3 story decomposition assigns these.

**Test vectors supplement changes (test-vectors.md v1.0 → v1.1):**

- SS-16 section added with: same-second storm denominator edge cases; GARP escalation table;
  binding-table LRU eviction table; malformed ARP table; SLL outer_src_mac=None table.

**Holdout seeds added (HS-INDEX.md v1.0 → v1.1):**

- 20 ARP feature holdout seeds registered (HS-W40-NNN through HS-W44-NNN, waves 40-44).
- Seeds categorized across 5 waves matching estimated STORY-111..115 decomposition.
- 2 real-world corpus seeds: known-good (clean LAN ARP) and known-problematic (ARP poisoning pcap).
- Full scenarios to be authored by holdout-evaluator in Phase 4.

**Documents updated:**

| Document | Version | Change |
|----------|---------|--------|
| `specs/prd.md` | 1.8 → 1.9 | Added Section 2.16 (15 BC summary table), SS-16 RTM rows, MITRE O-04 update, v1.9 delta note |
| `specs/behavioral-contracts/BC-INDEX.md` | 1.6 → 1.7 | Added ss-16 section (15 rows), updated BC-2.02.009 title row (v1.5), updated count 268→283 |
| `specs/prd-supplements/error-taxonomy.md` | 1.2 → 1.3 | Added E-DEC-004, ARP category, E-ARP-001..003 |
| `specs/prd-supplements/test-vectors.md` | 1.0 → 1.1 | Added SS-16 edge-case vectors section |
| `holdout-scenarios/HS-INDEX.md` | 1.0 → 1.1 | Added ARP feature holdout seeds (waves 40-44) |
| `spec-changelog.md` | — | This entry |

---

## [dnp3-f2-mustadds-c2fix-2026-06-10] — 2026-06-10

### MINOR: BC-2.15.024 C-2 Remediation — Separate Windowed Counter + Two-Counter Model

**Summary:** Adversarial pass C-2 identified that the original BC-2.15.024 (v1.0, recorded in
`§[dnp3-f2-mustadds-2026-06-10]` below) reused `parse_errors: u64` as a windowed threshold
counter. This was incorrect: `parse_errors` is a lifetime monotonic counter consumed by
BC-2.15.020's summarize() output; resetting it would corrupt the lifetime parse-error summary
(BC-2.15.020 postcondition), and since BC-2.15.015 is the single reset owner of correlated
window fields, the absence of a dedicated windowed field left EC-005 unsatisfiable. This entry
records the corrected two-counter model that resolves both C-1 (orphaned fields) and C-2
(reset owner + parse_errors collision).

**Key design change — two-counter model:**
- `parse_errors: u64` — LIFETIME monotonic counter. Never reset. Consumed exclusively by
  BC-2.15.020 (`summarize()` detail map key `"parse_errors"`). Unchanged from original intent
  except it is now explicitly forbidden from serving as a windowed threshold counter.
- `malformed_in_window: u64` — NEW windowed threshold counter. Incremented on each
  structural-reject path (LENGTH<5, frame-length/block-count mismatch, sync-loss) within the
  active 300s correlation window. Threshold: `const MALFORMED_ANOMALY_THRESHOLD: u64 = 3`.
  Reset by BC-2.15.015 (single reset owner) at the 300s CORRELATION_WINDOW_SECS expiry.
- `malformed_anomaly_emitted: bool` — NEW one-shot guard. Prevents repeated T0814 anomaly
  emission within the same window once threshold is reached. Reset by BC-2.15.015 alongside
  all other windowed fields.

**Updated Dnp3FlowState canonical field set (architecture-delta v1.2 + ADR-007):**
BC-2.15.015 is now the single reset owner of **6 windowed fields** (was 4):
`restart_event_count: u64`, `block_event_count: u64`,
`pending_requests: HashMap<(u16,u8), u32>`,
`block_finding_emitted_this_window: bool`,
`loss_of_control_emitted: bool`,
`correlation_window_start_ts: u32`,
`malformed_in_window: u64` ← NEW,
`malformed_anomaly_emitted: bool` ← NEW.
(`parse_errors: u64` and `malformed_anomaly_emitted: bool` registration also confirmed in ADR-007.)

**Rationale for the fix:**
Reusing `parse_errors` as a windowed counter would have: (1) silently corrupted BC-2.15.020's
lifetime parse-error summary whenever BC-2.15.015 reset the window — a cross-BC invariant
violation; (2) left `malformed_anomaly_emitted: bool` with no reset path under the original
design, making EC-005 (second burst in new window fires again) unsatisfiable. The separate
`malformed_in_window` counter eliminates both issues cleanly.

**Adversarial findings resolved:**
- **C-1** (orphaned fields): `malformed_anomaly_emitted` is now explicitly registered in
  Dnp3FlowState and listed as a BC-2.15.015 reset target.
- **C-2** (reset owner + parse_errors collision): `parse_errors` is now read-only from
  BC-2.15.024's perspective; the new `malformed_in_window` counter is owned and reset by
  BC-2.15.015.

**Files changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| BC-2.15.024 | v1.0 → v1.1 | Precondition 3 updated: threshold counter is `malformed_in_window`, not `parse_errors`. Postcondition 1 updated: fires when `malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD`. Invariant 3 added: `parse_errors` is lifetime-only and must never be used as windowed threshold. Invariant 4 added: `malformed_in_window` is the sole windowed structural-reject counter. EC-005 updated: second-burst scenario now satisfiable via `malformed_in_window` reset. |
| BC-2.15.015 | v1.4 → v1.5 | Reset owner postcondition updated: now resets 6 windowed fields (adds `malformed_in_window` and `malformed_anomaly_emitted`). Invariant 6 updated: field list extended. Architecture Anchors updated with both new fields. |
| dnp3-architecture-delta | v1.1 → v1.2 | Dnp3FlowState field table: `malformed_in_window: u64` and `malformed_anomaly_emitted: bool` added; `parse_errors` annotated as lifetime-only (not reset). BC-2.15.015 reset-owner table updated to 6 windowed fields. |
| ADR-007 | existing | Consequences section updated: two-counter model rationale recorded; `parse_errors` immutability invariant noted. |
| BC-INDEX | v1.5 → v1.6 | BC-2.15.024 title updated to reflect two-counter model; BC-2.15.015 annotation updated. |
| prd.md | v1.6 → v1.7 | Section 2.15 BC-2.15.024 summary updated; delta note for C-2 fix added. |

**Historical note:** The v1.0 design recorded in `§[dnp3-f2-mustadds-2026-06-10]` (below) is
correct as a historical record of the original must-add design at that point in time. It is
intentionally preserved unchanged. This entry supersedes it for the two-counter model going
forward.

---

## [dnp3-f2-mustadds-2026-06-10] — 2026-06-10

### MINOR: DNP3 F2 Research Must-Add Detections — issue #8 Post-Gate Scope Validation

**Summary:** Two research-validated must-add detections added to SS-15 based on
`dnp3-f2-scope-threshold-validation.md` (verified against Crain/Sistrunk S4x14, Chipkin
DNP3 Quick Reference, Zeek/icsnpp-dnp3 tool coverage). Both new BCs map to existing T0814
(Denial of Service / Inhibit Response Function). MITRE catalog counts remain 23 seeded /
15 emitted / 8 catalogue-only — UNCHANGED. No changes to `classify_dnp3_fc`, VP-023, or
BC-2.15.006.

**BC-2.15.023 — Unsolicited-Response Enable/Disable Abuse → T0814:**
DISABLE_UNSOLICITED (FC 0x15) is the classic alarm-suppression / event-blinding primitive:
an attacker sends it to silence outstation event reporting. ENABLE_UNSOLICITED (FC 0x14) is
the control-plane counterpart. Detection keys on the RAW FC byte directly (NOT via
classify_dnp3_fc — 0x14/0x15 return `Management` from the classifier, which is correct and
unchanged). Per-occurrence detection mirroring BC-2.15.011 (restart) style. Severity split:
DISABLE_UNSOLICITED → Likely/Medium; ENABLE_UNSOLICITED → Possible/Low. Source: [VERIFIED]
Chipkin DNP3 Quick Reference FC table; Crain/Bratus: disproportionate share of DNP3
application-layer vulns in unsolicited response functions.

**BC-2.15.024 — Malformed/Structural DNP3 Anomaly → T0814 [F2-GATE-DEFAULT: ≥3/300s]:**
Surfaces the parser's EXISTING structural-reject paths (LENGTH<5, frame-length/block-count
mismatch, sync-loss) as a low-confidence T0814 anomaly when `flow.parse_errors` reaches
MALFORMED_ANOMALY_THRESHOLD **[F2-GATE-DEFAULT: ≥3 malformed frames within 300s]**. This is
the ONLY coverage for the Crain-Sistrunk "Project Robus" class (~28-30 DNP3 vulns, 16+ ICS-CERT
advisories). Critically, Crain-Sistrunk attack frames carry VALID CRCs — CRC deferral does NOT
excuse this blind spot (the two are orthogonal). New flow-state fields: `malformed_anomaly_emitted:
bool` (one-shot guard); uses existing `parse_errors: u64` counter and shared 300s
CORRELATION_WINDOW_SECS window (BC-2.15.015 is single reset owner). Deep object-level
malformation analysis deferred to v2 (JUDGMENT, per research).

**Threshold clarifications applied to existing BCs:**
- BC-2.15.010 v1.2: 10/60s threshold is a FLOOD GUARD, not the primary unauthorized-source
  detector. Unauthorized control from an UNEXPECTED SOURCE fires at count=1, independent of
  the rate threshold. ~5/60s available for quiet transmission profiles via CLI flag. 10/60s
  default CONFIRMED (do not raise above 10).
- BC-2.15.014 v1.4: DIRECT_OPERATE_NR (0x06) exclusion from block-command timeout count is
  research-CONFIRMED [VERIFIED: dnp3-f2-scope-threshold-validation.md §Q2 Threshold-2]. The
  exclusion was already present in Precondition 1 and Invariant 1 since v1.0; this records
  the explicit research-backed validation.
- BC-2.15.015 v1.4: ≥3 combined restart+block events must be DISTINCT impact events — a single
  underlying incident cannot be double-counted. The implementation is already correct (two
  independent increment paths); this clarification makes the semantic intent explicit.

**Files changed:**
- NEW: `.factory/specs/behavioral-contracts/ss-15/BC-2.15.023.md` (v1.0)
- NEW: `.factory/specs/behavioral-contracts/ss-15/BC-2.15.024.md` (v1.0)
- UPDATED: `BC-2.15.010.md` v1.1 → v1.2 (threshold clarification)
- UPDATED: `BC-2.15.014.md` v1.3 → v1.4 (DIRECT_OPERATE_NR confirmation)
- UPDATED: `BC-2.15.015.md` v1.3 → v1.4 (distinct-events clarification + Invariant 7)
- UPDATED: `BC-INDEX.md` v1.4 → v1.5 (SS-15 24 BCs; total 266 → 268)
- UPDATED: `prd.md` v1.5 → v1.6 (Section 2.15.I, RTM rows, KD-005 rows, delta note)

---

## [dnp3-f2-pass2-2026-06-10] — 2026-06-10

### MINOR: DNP3 F2 Pass-2 Adversarial Remediation — CRITICAL-1 + CRITICAL-2

**Summary:** Two blocking adversarial findings from the Pass-2 review of issue #8 DNP3 spec.

**CRITICAL-1 (fabricated T1691.001 name in spec-changelog):** Two live references to the
fabricated name "Unauthorized Message: Inhibit Response Function" in the dnp3-f2-2026-06-10
changelog entry (MITRE catalog section and summary list) have been corrected to the authoritative
"Block Operational Technology Message: Command Message". The fabricated name now survives ONLY
in `modified:` audit-trail entries in BC-2.10.005, BC-2.10.007, BC-2.10.008 that explicitly
document the correction from the Pass-1 burst.

**CRITICAL-2 (window-reset contradiction on block_event_count):** The dual-window model
(BC-2.15.014 using BLOCK_CMD_WINDOW_SECS=120s; BC-2.15.015 using T0827_WINDOW_SECS=300s)
caused block events spaced 120–300s apart to be silently discarded by the 120s sub-window
reset before T0827 could see them. The separate BLOCK_CMD_WINDOW_SECS=120s window is eliminated.
The model now uses a single shared CORRELATION_WINDOW_SECS=300s [F2-GATE: human to confirm]
tracked by correlation_window_start_ts: u32. BC-2.15.015 is the single reset owner: it resets
all four correlated-state fields together (restart_event_count, block_event_count,
block_finding_emitted_this_window, loss_of_control_emitted) at the 300s expiry.

The T1691.001 "sustained pattern" is now 3-of-300s (was 3-of-120s in v1.1). This change
is explicitly flagged [F2-GATE] for human confirmation.

Both T0827 traces verified end-to-end under the single-window model:
- Trace A (3 restarts within 300s): fires correctly.
- Trace B (2 block events spaced 150s apart + 1 restart at 200s): fires correctly.
  (Under old model: block_event_count reset at 120s → combined=1 at t=200 → T0827 suppressed.)

**Canonical per-flow correlation field set (agreed with architect):**
`restart_event_count: u64`, `block_event_count: u64`,
`pending_requests: HashMap<(u16,u8), u32>`,
`block_finding_emitted_this_window: bool`,
`loss_of_control_emitted: bool`,
`correlation_window_start_ts: u32`.
(BC-2.15.010 keeps its own separate 60s window/window_start_ts/direct_operate_count — not merged.)

**Artifacts changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| spec-changelog.md | this entry | CRITICAL-1: two live fabricated T1691.001 names corrected in dnp3-f2-2026-06-10 entry (lines ~43 and ~74). CRITICAL-2: this entry added. |
| BC-2.15.011 | v1.0 → v1.1 | Added single-window reference to Description, Postcondition 2, Invariant 6; added EC-007 (spaced block+restart trace); updated Architecture Anchors with correlation_window_start_ts. |
| BC-2.15.014 | v1.1 → v1.2 | Eliminated BLOCK_CMD_WINDOW_SECS=120s; T1691.001 threshold now over CORRELATION_WINDOW_SECS=300s; Invariant 7 rewritten (single reset owner); EC-006 updated (300s); EC-008 added (key fix trace: 2 blocks at t=0/150s + restart at t=200s → T0827 fires); canonical test vectors updated. |
| BC-2.15.015 | v1.1 → v1.2 | Named as single reset owner; Postcondition 3 added (window-expiry reset spec); Invariant 6 rewritten; T0827_WINDOW_SECS removed (now CORRELATION_WINDOW_SECS); EC-008 and EC-009 added; Traces A-D documented end-to-end; Architecture Anchors list all six canonical correlation fields. |

---

## [dnp3-f2-2026-06-10] — 2026-06-10

### MINOR: Feature #8 DNP3/ICS Analyzer — SS-15 BCs + PRD Section 2.15 + MITRE Catalog Update

**Summary:** Feature #8 (issue #8) adds the DNP3 TCP protocol analyzer (SS-15). 22 behavioral
contracts (BC-2.15.001..022) specify parsing, detection, correlated-finding, and CLI integration
behavior. Two new MITRE ATT&CK for ICS techniques (T1691.001 and T0827) are seeded and emitted.
A new ICS-unique `MitreTactic` variant (`IcsImpact`) is added to the enum. All MITRE catalog BCs
and the PRD are updated to reflect the new counts (SEEDED=23, EMITTED=15, CATALOGUE-ONLY=8).

**BC-2.15 group structure:**
- Group A+C (BCs 001–004): DL header parse safety and validity gate. VP-023 target.
- Group B (BCs 005–007): FC classification totality and frame-length arithmetic. VP-023 target.
- Group C (BCs 008–009): Transport FIR gating and desync-safe bail.
- Group D (BCs 010–013): Direct-detection findings — T1692.001 (control threshold), T0814
  (restart/DoS), T0836 (write FC), co-emission ordering.
- Group E (BCs 014–015): Inferred/correlated findings — T1691.001 (block-command inference,
  per-flow request/response correlation), T0827 (derived loss-of-control, N-event aggregation).
- Group F (BCs 016–017): Bounded resource (292-byte carry buffer, 64-entry master_addrs_seen)
  and CLI flag (`--dnp3-direct-operate-threshold`).
- Group G (BCs 018–019): Anomaly detection (broadcast DEST, unsolicited response).
- Group H (BCs 020–022): summarize() stats, port-20000 dispatcher Rule 6, MAX_FINDINGS cap.

**MITRE catalog changes (SEEDED 21→23, EMITTED 13→15):**

New seeded + emitted ICS techniques:
- **T1691.001** — "Block Operational Technology Message: Command Message" (ICS sub-technique, v19)
  → tactic `IcsInhibitResponseFunction`. Emitted by: BC-2.15.014 (inferred block-command).
- **T0827** — "Loss of Control" (ICS Impact tactic TA0105)
  → tactic `IcsImpact` (NEW enum variant). Emitted by: BC-2.15.015 (derived correlated finding).

New MitreTactic variant:
- **IcsImpact** — Display "Impact" (canonical ICS TA0105 name, no prefix). Third ICS-unique
  variant after IcsInhibitResponseFunction and IcsImpairProcessControl.
  `all_tactics_in_report_order` grows from 16 to 17 elements; IcsImpact at position [16].

Arithmetic verification: SEEDED=23 (11 Enterprise + 12 ICS), EMITTED=15 (6 Enterprise + 9 ICS),
CATALOGUE-ONLY = 23 − 15 = 8. Previous: SEEDED=21, EMITTED=13, CATALOGUE-ONLY=8.
The catalogue-only count is unchanged because both new techniques are immediately emitted.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| PRD | Version bump 1.4 → 1.5; Section 2.15 added (22 BCs); Section 7 RTM extended (22 rows); KD-003 and KD-005 and KD-007 updated with DNP3 BC references; O-04 updated SEEDED/EMITTED counts; total BC count 244 → 266 | `.factory/specs/prd.md` |
| BC-INDEX | Version bump 1.3 → 1.4; ss-15 subsystem section added (22 rows); total BC count 244 → 266 | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| BC-2.15.001..022 | Created (F2 DNP3 create burst, Groups A-H) | `.factory/specs/behavioral-contracts/ss-15/` |
| BC-2.10.005 | v1.5 → v1.6: SEEDED 21→23; added T1691.001 + T0827 to seeded set; postconditions, invariants, edge cases, test vectors updated | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.005.md` |
| BC-2.10.007 | v1.4 → v1.5: Added T1691.001→IcsInhibitResponseFunction, T0827→IcsImpact tactic assignments; invariant 3 extended with IcsImpact note | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.007.md` |
| BC-2.10.008 | v1.6 → v1.7: EMITTED 13→15; added T1691.001 + T0827 to emitted set; description emission sites updated with dnp3.rs; postconditions, invariants, EC-014/015, test vectors updated | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.008.md` |
| BC-2.10.002 | v1.2 → v1.3: Added IcsImpact variant; description, preconditions, postconditions, invariants, edge cases, test vectors updated; slice length 16→17 noted | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.002.md` |
| BC-2.10.003 | v1.2 → v1.3: Slice length 16→17; element [16] = IcsImpact; description, postconditions, invariants, edge cases, test vectors, VP-016 updated | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.003.md` |
| BC-2.10.004 | v1.2 → v1.3: Variant count 16→17; description, postconditions, invariants, edge cases, test vectors updated | `.factory/specs/behavioral-contracts/ss-10/BC-2.10.004.md` |
| ADR-007 | Created (binary ICS protocol integration decision for DNP3 TCP) | `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md` |
| VP-023 | Designed (parse_dnp3_dl_header, classify_dnp3_fc, is_valid_dnp3_frame_header, compute_dnp3_frame_len) | `.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md` |

**New MITRE ATT&CK for ICS techniques (2 total, Feature #8):**
- T1691.001 — Block Operational Technology Message: Command Message (IcsInhibitResponseFunction, v19 ICS sub-technique)
- T0827 — Loss of Control (IcsImpact, ICS Impact tactic TA0105)

**MITRE catalog size:** 21 → 23 seeded technique IDs; 13 → 15 emitted IDs; 8 catalogue-only unchanged.

**New ICS tactic variant:** IcsImpact (Display "Impact") — third ICS-unique variant.

**Key constants introduced:**
- `MAX_DNP3_FRAME_LEN = 292` (per-flow carry buffer cap; matches DNP3 link-layer max)
- `MAX_MASTER_ADDRS = 64` (per-flow master-address tracking cap)
- `DNP3_TCP_PORT = 20000` (dispatcher Rule 6 port)
- `DEFAULT_DIRECT_OPERATE_THRESHOLD` (--dnp3-direct-operate-threshold default; exact value TBD at F3)

**CLI surface changes:**
- `--dnp3` flag added to `analyze` subcommand (boolean, default false)
- `--dnp3-direct-operate-threshold N` flag added (u32, default TBD; zero rejected)
- `--all` expansion updated to include `--dnp3`
- `needs_reassembly` expression updated: `enable_http || enable_tls || enable_modbus || enable_dnp3`

**Dispatcher changes:**
- `DispatchTarget::Dnp3` variant added (5th variant)
- `StreamDispatcher.dnp3: Option<Dnp3Analyzer>` field added
- `classify` Rule 6: port 20000 → `DispatchTarget::Dnp3` (after all prior rules 1-5)
- `dnp3_analyzer()` and `take_dnp3_analyzer()` accessors added
- `on_data` and `on_flow_close` DNP3 routing arms added

---

## [v19-remap-2026-06-10] — 2026-06-10

### MINOR: MITRE ATT&CK for ICS v19 Remap — T0855 → T1692.001, T0856 → T1692.002

**Summary:** 1:1 technique-ID remap driven by DF-VALIDATION-001-validated defect (issue #222).
MITRE ATT&CK for ICS v19.0 (released 2026-04-28) introduced sub-techniques to the ICS matrix
for the first time and simultaneously revoked T0855 and T0856:

- **T0855 "Unauthorized Command Message" (REVOKED)** → **T1692.001 "Unauthorized Message:
  Command Message"** (ICS sub-technique under parent T1692 "Unauthorized Message")
- **T0856 "Spoof Reporting Message" (REVOKED)** → **T1692.002 "Unauthorized Message:
  Reporting Message"** (ICS sub-technique under T1692)

Tactic assignment unchanged for both: `IcsImpairProcessControl`.

**Scope:** Spec artifacts only. Source code (src/), test files, and story bodies are out of
scope for this burst; those are handled by implementer/story-writer in subsequent bursts.

**Authoritative research docs:**
- `.factory/research/mitre-ics-v19-catalog-audit.md` (audit of all affected IDs)
- `.factory/research/dnp3-mitre-verification.md` (cross-verification for DNP3 techniques)

**BCs updated (all T0855 → T1692.001 in live spec body):**
- SS-14 (Modbus): BC-2.14.006 v1.1, BC-2.14.007 v1.1, BC-2.14.008 v1.1, BC-2.14.011 v1.1,
  BC-2.14.013 v2.3, BC-2.14.014 v2.3, BC-2.14.015 v2.3, BC-2.14.016 v2.2, BC-2.14.017 v2.4,
  BC-2.14.018 v1.2, BC-2.14.019 v1.4, BC-2.14.020 v2.1, BC-2.14.022 v2.1, BC-2.14.024 v2.1
- SS-11 (Reporting): BC-2.11.001 v1.6, BC-2.11.013 v1.7, BC-2.11.017 v1.6, BC-2.11.020 v1.6,
  BC-2.11.024 v1.6
- SS-10 (MITRE catalog): BC-2.10.008 v1.6
- SS-09 (Finding model): BC-2.09.001 v1.5, BC-2.09.006 v1.6

**Other artifacts updated:**

*Spec/architecture:*
- BC-INDEX.md: T0855 → T1692.001 in title/group notes for affected BCs
- prd.md: version bumped to 1.4; all live body T0855 references updated to T1692.001;
  version 1.4 delta note added
- VP-007: technique-ID regex updated to accept `T\d{4}\.\d{3}` sub-technique format
- verification-architecture.md: VP-007 entry updated for sub-technique format
- verification-coverage-matrix.md: T0855 → T1692.001 in coverage rows
- module-decomposition.md: C-22 (Modbus analyzer) technique-ID references updated
- cap-10-mitre-mapping.md: T0855 → T1692.001, T0856 → T1692.002 rows updated
- domain-debt.md: O-04 note updated to reflect T0855 status as T1692.001 (revoked parent)
- ADR-005: historical T0855 reference annotated as revoked → T1692.001
- ADR-006: historical T0855 reference annotated as revoked → T1692.001

*Stories:*
- STORY-071: BC table and acceptance criteria updated for T0855 → T1692.001
- STORY-078: BC table and acceptance criteria updated for T0855 → T1692.001
- STORY-079: BC table and acceptance criteria updated for T0855 → T1692.001
- STORY-100: BC table and acceptance criteria updated for T0855 → T1692.001
- STORY-101: BC table and acceptance criteria updated for T0855 → T1692.001
- STORY-104: BC table and acceptance criteria updated for T0855 → T1692.001

*Wave/holdout:*
- wave-31-holdout.md: expected MITRE output updated to T1692.001
- wave-32-34-holdout.md: expected MITRE output updated to T1692.001

*Code (branch fix/mitre-ics-v19-remap — separate burst):*
- src/mitre.rs: T0855 → T1692.001, T0856 → T1692.002 catalog entries
- src/analyzer/modbus.rs: technique-ID string literals updated
- src/cli.rs: help-text / validation references updated
- src/reporter/json.rs: technique-ID output updated
- src/reporter/terminal.rs: technique-ID display updated
- src/findings.rs: technique-ID constants/defaults updated
- CHANGELOG.md: v19 remap entry added
- tests/: golden output fixtures and assertion strings updated

*Meta:*
- spec-changelog.md: this entry

**Historical references intentionally preserved (not updated):**
- All `modified:` YAML entries predating this change that mention T0855 in their `change:` text
- HTML `<!-- Previous version... -->` comments in BC files (they describe historical spec state)
- v1.5 modified entry in BC-2.11.017 mentioning "MITRE: T0855, T0836" format
- Prior changelog entries (lines below this entry)

**Note on ICS sub-technique format:** T1692.001 and T1692.002 use the `Txxxx.NNN`
sub-technique format introduced in ATT&CK v19 for the ICS matrix. Any BC or validator
that documents the allowed MITRE ID format/regex must accept this format (coordinate with
architect's VP-007 update).

---

## [1.6] — 2026-06-09

### MINOR: Holdout Blemish-1 Fix — BC-2.14.019 Exception-Burst Recon 0x01/0x02 → T0888

**Summary:** Holdout evaluation blemish-1 for Feature #7 v0.4.0 (Modbus TCP analyzer).
The exception-burst anomaly detection for exception codes 0x01 (Illegal Function = FC scanning)
and 0x02 (Illegal Data Address = register-map enumeration) was previously untagged
(`mitre_techniques: vec![]`), citing "no clean single ICS indicator" per research §7.

Orchestrator decision: both anomaly patterns ARE a form of Remote System Information Discovery
(T0888, TA0102 Discovery) and now map to T0888, consistent with the established
recon→T0888 mapping for FCs 0x11 and 0x2B/0x0E (BC-2.14.020, Decision 12):
- Exception 0x01 (Illegal Function): FC scanning discovers which function codes the device
  supports — exactly the query-device-capabilities behavior that T0888 covers.
- Exception 0x02 (Illegal Data Address): register-map enumeration discovers the device's
  address layout — exactly the query-device-address-space behavior that T0888 covers.

Other exception codes (0x03, 0x04, 0x05, 0x06, etc.) and the Clear Counters 0x000A
anti-forensic path retain `mitre_techniques: vec![]` (no clean ICS ATT&CK mapping).

T0888 is already in the SS-14 emitted set (BC-2.14.020, BC-2.10.008 emitted-ID list,
SEEDED_TECHNIQUE_ID_COUNT and EMITTED_IDS unchanged — this is not a catalog expansion).

**Note for the record — holdout blemish-2 disposition:** Port-502 service label in
summary (src/decoder.rs:112) was assessed as CORRECT-BY-DESIGN (standard IANA port-service
hint, parallel to 443→HTTPS). Not a defect; no spec or code change.

**Artifacts changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| BC-2.14.019 | v1.2 → v1.3 | Postcondition Path A: mitre_techniques for exception code 0x01 → vec!["T0888"]; exception code 0x02 → vec!["T0888"]; other codes retain vec![]. Research note updated. Canonical test vectors for 0x01/0x02 cases updated to show ["T0888"]. Traceability MITRE field updated. Path B (Clear Counters) unchanged. |

**Impact:** MINOR. No BC semantics removed. Downstream stories targeting BC-2.14.019 Path A
with exception codes 0x01 or 0x02 must update acceptance criteria to expect
`mitre_techniques=["T0888"]` instead of `mitre_techniques=[]`. Clear Counters and other
exception code paths are unaffected.

**Emitted technique set:** Unchanged (T0888 was already emitted by BC-2.14.020). No change
to `SEEDED_TECHNIQUE_ID_COUNT`, `EMITTED_IDS`, or BC-2.10.008.

---

## [1.5] — 2026-06-09

### MINOR: F5 Combined-Delta Spec Defect Fixes — SS-14 Modbus v0.4.0

**Summary:** Four spec defects discovered during the F5 combined-delta review of Feature #7
(Modbus TCP analyzer). These defects existed in the SS-14 BC corpus and are corrected here
without changing any implementation behavior (the implementation is being authored in parallel
to align with this correction). All changes are MINOR (no BC semantics removed; existing
downstream story acceptance criteria remain valid with updated formulas).

**Defect 1 — Timestamp units: microseconds→seconds (BC-2.14.016, BC-2.14.017, BC-2.14.019)**

The f2-fix-directives §11.5 introduced a microsecond-scale assumption for window math
(`*1_000_000`, `elapsed_us`, `>= 2_000_000`). This was wrong. The pipeline's
`StreamHandler::on_data` delivers `timestamp_secs` (seconds, per BC-2.09.007); TLS/HTTP/
reassembler all confirm this via `DateTime::from_timestamp(ts, 0)`. All four Modbus window
computations have been corrected to seconds-scale math:

| Window | Old check (wrong) | New check (correct) |
|--------|------------------|---------------------|
| T0831 5s coordinated-write | `elapsed > T0831_WINDOW_SECS * 1_000_000` | `elapsed_secs > T0831_WINDOW_SECS` |
| Burst 1s write-rate | `elapsed > WRITE_BURST_WINDOW_SECS * 1_000_000` | `elapsed_secs > WRITE_BURST_WINDOW_SECS` |
| Sustained ≥2s write-rate | `elapsed_us >= 2_000_000 AND count*1_000_000 > threshold*elapsed_us` | `elapsed_secs >= WRITE_SUSTAINED_WINDOW_SECS AND count > threshold * elapsed_secs` |
| Exception 10s recon | `elapsed > 10_000_000` | `elapsed_secs > EXCEPTION_WINDOW_SECS` |

`wrapping_sub` is retained on all windows (u32 second timestamps wrap at ~136 years —
effectively never in practice, but the policy is kept for correctness).
Sub-second rate precision is a future enhancement requiring `timestamp_usecs` threading.

**Defect 2 — Non-existent FlowKey accessor: flow_key.client_ip() / flow_key.server_ip()
(BC-2.14.013, BC-2.14.017, BC-2.14.019)**

The `source_ip` postconditions in three BCs cited `flow_key.client_ip()` and
`flow_key.server_ip()`. These methods DO NOT EXIST on `FlowKey` (which has only
`lower_ip`, `upper_ip`, `lower_port`, `upper_port` plus a `Direction`). Corrected:
`source_ip` is now resolved from the `direction` arg passed to `on_data` combined with
`flow_key.lower_ip()`/`upper_ip()`. The mapping is:
- `Direction::ClientToServer` → initiator/client endpoint (write-class findings, BC-2.14.013; burst/sustained findings, BC-2.14.017; Clear Counters path, BC-2.14.019)
- `Direction::ServerToClient` → responder/server endpoint (exception-response findings, BC-2.14.019 Path A)

**Defect 3 — AnalysisSummary top-level field hallucination (BC-2.14.021)**

BC-2.14.021 postcondition 3 (v1.0) cited `findings_count`, `flows_analyzed`, and `protocol`
as top-level fields of `AnalysisSummary`. These fields DO NOT EXIST in the shared struct
(`src/analyzer/mod.rs`): the struct has exactly three fields — `analyzer_name: String`,
`packets_analyzed: u64`, `detail: BTreeMap<String, Value>`. Postcondition 3 has been
completely rewritten to match the real struct.

The SIX authoritative detail keys (post.1) are UNCHANGED and remain the authoritative
contract for the Modbus summary `detail` map:

| Key | Type | Semantics |
|-----|------|-----------|
| `"pdu_count"` | `Value::Number(u64)` | Total valid ADUs past the three-point gate |
| `"write_count"` | `Value::Number(u64)` | Total write-class FC PDUs |
| `"exception_count"` | `Value::Number(u64)` | Total exception-response PDUs (FC >= 0x80) |
| `"parse_errors"` | `Value::Number(u64)` | Total ADUs failing the validity gate |
| `"function_code_distribution"` | `Value::Object(map)` | FC → count map (hex-string keys, zero-count FCs omitted) |
| `"dropped_findings"` | `Value::Number(u64)` | Findings dropped due to MAX_FINDINGS cap (ALWAYS present, even 0) |

**Defect 4 — f2-fix-directives §11.5 / §11.5b microsecond-scale math superseded**

Added F5-correction banners to §11.5 and §11.5b in f2-fix-directives.md. The microsecond
math (`elapsed_us`, `*1_000_000`) is superseded by the seconds form. The corrected canonical
specification is BC-2.14.017 v2.2.

**Artifacts changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| BC-2.14.013 | v2.1 → v2.2 | source_ip: flow_key.client_ip() → Direction-resolved client endpoint |
| BC-2.14.016 | v2.0 → v2.1 | Window math: microseconds → seconds; edge cases EC-004/005/010 updated; test vectors updated to second-scale timestamps |
| BC-2.14.017 | v2.1 → v2.2 | Window math: microseconds → seconds (both burst and sustained); source_ip: flow_key.client_ip() → Direction-resolved; edge cases EC-004/004b/005/006/010 updated; test vectors updated; constants clarified as seconds |
| BC-2.14.019 | v1.1 → v1.2 | Window math: microseconds → seconds; source_ip Path A: flow_key.server_ip() → Direction-resolved server endpoint; source_ip Path B: flow_key.client_ip() → Direction-resolved; EC-009 updated |
| BC-2.14.021 | v1.0 → v1.1 | post.3 completely rewritten: remove non-existent flows_analyzed/findings_count/protocol fields; align to real AnalysisSummary struct (analyzer_name, packets_analyzed, detail only); six detail keys in post.1 unchanged and remain authoritative |
| f2-fix-directives.md | §11.5, §11.5b | F5-correction banners added; microsecond math identified as superseded; corrected form is seconds-scale per BC-2.14.017 v2.2 |

**Impact:** MINOR. No BC semantics removed; existing downstream story acceptance criteria
remain valid after updating formulas from microsecond to second scale. The implementation
(authored in parallel) is being aligned to seconds-scale math simultaneously with this
spec correction.

---

## [1.4] — 2026-06-09

### MINOR: BC-DISCREPANCY-001 — FC 0x17 Register-Write Set Reconciliation

**Summary:** Reconciled a discrepancy in the FC 0x17 (Read/Write Multiple Registers)
technique-tag mapping across BC-2.14.013, BC-2.14.014, and BC-2.14.015. Per orchestrator
ruling: FC 0x17 writes holding registers in its write phase and is therefore a
Modify-Parameter (T0836) operation. It participates in the T0831 register-write window
set {0x06, 0x10, 0x16, 0x17} and emits the union [T0855, T0836] per PDU. BC-2.14.016
already correctly included 0x17 in this set; the discrepancy was in the other three BCs.

**Root cause:** BC-2.14.013 EC-001 and Invariant 2 grouped FC 0x15 (Write File Record)
and FC 0x17 together as "File/multi writes → [T0855] only". This was stale/wrong for 0x17:
Write File Record targets file records (correctly T0855-only), but Read/Write Multiple
Registers writes holding registers (should carry T0836). BC-2.14.014 and BC-2.14.015
propagated the same error in their FC set definitions.

**Artifacts changed:**

| Artifact | Version | Change |
|----------|---------|--------|
| BC-2.14.013 | v2.0 → v2.1 | EC-001 corrected: 0x17 → ["T0855","T0836"]; Postcondition 1 tag-union bullet updated; Invariant 2 split: {0x06,0x10,0x16,0x17} → T0836; {0x15} → T0855 only; test vector for 0x17 added |
| BC-2.14.014 | v2.0 → v2.1 | Title updated to include 0x17; Description FC set updated to {0x06,0x10,0x16,0x17}; Precondition 3 updated; Invariant 1 updated; Invariant 2 T0836 set updated; Invariant 4 corrected (0x17 is IN T0836 set, not T0855-only); test vector for 0x17 added |
| BC-2.14.015 | v2.0 → v2.1 | Precondition 3 corrected: 0x17 now referenced as holding-register write; Invariant 2 (0x17 entry) updated to T0855+T0836; Invariant 4 disjoint-set updated to include 0x17 in T0836 set; EC-004 and 0x17 test vector corrected |
| BC-2.14.016 | v2.0 (unchanged) | Already correct: FC set {0x06,0x10,0x16,0x17} used throughout; no changes needed |

**Consistency result after reconciliation:**

| Technique | FC set | Authoritative source |
|-----------|--------|---------------------|
| T0855 (Unauthorized Command Message) | {0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17} — all Write class | BC-2.14.013 |
| T0836 (Modify Parameter) | {0x06, 0x10, 0x16, 0x17} — holding-register writes | BC-2.14.014 v2.1 |
| T0835 (Manipulate I/O Image) | {0x05, 0x0F} — coil writes only | BC-2.14.015 |
| T0831 window set | {0x06, 0x10, 0x16, 0x17} — holding-register writes (same as T0836) | BC-2.14.016 |
| T0855-only Write FCs | {0x15} — Write File Record (file records, not registers/coils) | BC-2.14.013 |

T0836 set == T0831 window set == {0x06, 0x10, 0x16, 0x17}. No overlaps between T0836 and
T0835 sets. These three sets are now consistent across all four BCs.

**Impact:** MINOR (backward-compatible addition — extends 0x17's tag set from [T0855] to
[T0855, T0836]; no existing BC semantics removed). Downstream stories that test FC 0x17
behavior must be updated to expect ["T0855","T0836"] instead of ["T0855"] only.

---

## [1.3] — 2026-06-09

### ADDITIVE: F2 Schema Add-Ons + v0.3.0/v0.4.0 Release Split Tagging

**Summary:** Two research-backed schema add-ons from `f2-multitag-schema.md` applied to
existing BCs, plus release sequencing recorded across prd.md and prd-delta.md per human
decision (f2-bundle-vs-split.md B2 — Trivy/Zeek pattern).

**ADD-ON 1 — JSON report envelope fields (BC-2.11.001 v1.5):**

Two top-level JSON report envelope fields added (ONCE per report, NOT per-finding):
- `mitre_domain: "ics-attack"` — identifies the ATT&CK matrix; constant.
- `mitre_attack_version: "ics-attack-v15"` — placeholder; **FLAG for F4 to pin** against
  deployed catalog before v0.3.0 release tag.

Basis: ECS/OCSF recommendation to declare domain+version at envelope level rather than
redundantly per-technique (`T0xxx` prefix already unambiguously identifies ICS matrix).
CSV reporters carry no envelope fields (JSON-only).

**ADD-ON 2 — CSV empty-string clarification (BC-2.11.024 v1.5):**

Existing EC-001 strengthened + EC-015 added:
- When `mitre_techniques = vec![]`, the CSV cell is `""` (empty string) — NOT `"null"`,
  `"[]"`, `"N/A"`, or any sentinel.
- EC-015: Documents required consumer guard: `str.split(';')` on `""` produces `['']` in
  most languages; consumers MUST check `if cell.is_empty()` before splitting and return
  an empty collection, not `['']`.

**Release split tagging (v0.3.0/v0.4.0):**

Feature #7 is split into two releases:
- **v0.3.0** (schema migration; breaking): SS-09 + SS-10 + SS-11 BCs + ADD-ONs.
  Existing analyzers migrated; no new protocol analyzer.
  Compat: `--compat-mitre-scalar` flag for deprecation window.
- **v0.4.0** (Modbus; additive): all SS-14 BCs (BC-2.14.001..025).
  Built on stable v0.3.0 schema; no `**Breaking:**` in v0.4.0 changelog.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| BC-2.11.001 | v1.4 → v1.5: envelope fields; H1 title updated; PC 7-8; Inv 4-6; EC-006, EC-007 | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.024 | v1.4 → v1.5: EC-001 strengthened; EC-015 added (consumer split guard); Inv 4 updated | `.factory/specs/behavioral-contracts/ss-11/` |
| prd.md | v1.2 → v1.3 note added; BREAKING box updated (envelope fields + CSV EC-015 ref); RELEASE SEQUENCING box added after BREAKING box; Section 2.14 release-target note added | `.factory/specs/prd.md` |
| prd-delta.md | new_prd_version 1.2 → 1.3; §5.3 ADD-ON details; §6 Release Sequencing; old §6 → §7 | `.factory/phase-f2-spec-evolution/prd-delta.md` |

**FLAG — mitre_attack_version not pinned:**
The value `"ics-attack-v15"` is a placeholder. F4 must verify the authoritative MITRE
ATT&CK for ICS version at attack.mitre.org/resources/attack-data-and-tools/ that covers
T0888, T0855, T0836, T0835, T0831, T0814, T0806, and update the constant in
`src/reporter/json.rs` before the v0.3.0 tag.

---

## [1.2] — 2026-06-09

### BREAKING: F2 Modbus Revision — Decisions 11-13 (ADR-006) — targets v0.3.0

**Summary:** Adopts three architect-approved decisions from `f2-fix-directives.md` v2.
Decision 13 is a breaking change to the `Finding` output schema targeting v0.3.0.
Revises 10 existing BCs (SS-09/SS-10/SS-11) + 8 SS-14 BCs already applied to BC body files.

**Adopted decisions:**

| Decision | Summary |
|----------|---------|
| D11 (supersedes D5) | Dual-window write-burst detection: `--modbus-write-burst-threshold` (default 20, 1s) + `--modbus-write-sustained-threshold` (default 10, >=2s). Old `--modbus-write-threshold` removed. |
| D12 (supersedes D8) | T0846 → T0888 correctness fix for recon FCs 0x11 and 0x2B/0x0E. T0888 = Remote System Information Discovery (TA0102 Discovery). T0846 remains seeded but is not emitted by Modbus. FC 0x07 excluded as standalone recon indicator. |
| D13 (supersedes D7) | Multi-tag Finding attribution: `Finding.mitre_technique: Option<String>` → `Finding.mitre_techniques: Vec<String>`. One finding per write PDU with ALL applicable technique tags. Volume control via burst aggregation, not tag-suppression. |

**BREAKING output schema changes (v0.3.0):**
- JSON: `"mitre_technique": "T0836"` → `"mitre_techniques": ["T0836"]` (key rename + type change)
- JSON: field absent when empty (same as prior `None` — `skip_serializing_if = "Vec::is_empty"`)
- JSON: multi-tag: `"mitre_techniques": ["T0855", "T0836"]`
- CSV: column-6 header renamed `mitre_technique` → `mitre_techniques`; multiple values semicolon-joined
- Rust: `Finding.mitre_technique: Option<String>` → `Finding.mitre_techniques: Vec<String>` (all emission sites + test helpers updated)

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| PRD | Version bump 1.1 → 1.2; Section 2 breaking-schema note added; Section 1.5, 2.10, 2.14 (D-H groups), 6.5, 8 updated | `.factory/specs/prd.md` |
| BC-INDEX | Version bump 1.1 → 1.2; SS-09/SS-10/SS-11 rows updated; SS-14 section header + BC-013/014/015/016/017/020/024 rows updated | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| prd-delta.md | Updated: new_prd_version 1.1→1.2; §5.2 added (10-BC revision table + 8 SS-14 BC revision table + affected-stories list) | `.factory/phase-f2-spec-evolution/prd-delta.md` |
| BC-2.09.001 | v1.4: `mitre_technique` field → `mitre_techniques` Vec | `.factory/specs/behavioral-contracts/ss-09/` |
| BC-2.09.006 | v1.5: `skip_serializing_if = "Vec::is_empty"`; multi-tag JSON output | `.factory/specs/behavioral-contracts/ss-09/` |
| BC-2.10.005 | v1.4: count 15 → 21 | `.factory/specs/behavioral-contracts/ss-10/` |
| BC-2.10.007 | v1.3: T0888 → Discovery row | `.factory/specs/behavioral-contracts/ss-10/` |
| BC-2.10.008 | v1.5: grep pattern + T0888 replaces T0846 in emitted list; 13 emitted | `.factory/specs/behavioral-contracts/ss-10/` |
| BC-2.11.013 | v1.6: multi-techniques tactic grouping by `[0]` | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.015 | v1.6: empty `mitre_techniques` vec → Uncategorized | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.017 | v1.5: multi-ID rendering `"MITRE: T0855, T0836"` | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.020 | v1.5: column-6 header rename | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.11.024 | v1.4: `mitre_techniques vec![]`; semicolon-join | `.factory/specs/behavioral-contracts/ss-11/` |
| BC-2.14.013..017,020,022,024 | v2.0: co-emission model; T0888; dual-window (bodies already revised) | `.factory/specs/behavioral-contracts/ss-14/` |
| ADR-006 | Registered in ARCH-INDEX ADR table | `.factory/specs/architecture/ARCH-INDEX.md` (already present) |

**MITRE catalog size change:**

| Metric | v1.1 | v1.2 |
|--------|------|------|
| `SEEDED_TECHNIQUE_ID_COUNT` | 20 | **21** (T0888 added) |
| `EMITTED_IDS` count | 12 | **13** (T0888 replaces T0846 in ICS emitted set) |
| ICS SEEDED | 9 | **10** (T0888 added; T0846 already seeded) |
| ICS EMITTED | 6 | **7** {T0855, T0836, T0814, T0806, T0835, T0831, T0888} |
| T0846 status | emitted | **seeded-not-emitted** |

**Affected stories (story-writer must propagate BC table + AC changes):**
STORY-069, STORY-070, STORY-071, STORY-078, STORY-079, STORY-080.

**ADR reference:** ADR-006 — Multi-Technique Finding Attribution
(`.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md`)

---

## [1.1] — 2026-06-09

### MINOR: SS-14 Modbus/ICS Analyzer — Feature #7

**Summary:** Added Modbus TCP protocol analyzer (SS-14, C-22) with 25 behavioral contracts,
VP-022 formal verification target, ADR-005 architecture decision, and 6 MITRE ATT&CK for
ICS technique mappings.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| PRD | Version bump 1.0 → 1.1; Section 2.14 added (25 BCs); Section 7 RTM extended (25 rows); KD-003 and KD-005 sections updated | `.factory/specs/prd.md` |
| BC-INDEX | Version bump 1.0 → 1.1; SS-14 subsystem section added (25 rows); total BC count 219 → 244 | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| BC-2.14.001..022 | Created (F2 create burst, Groups A-G) | `.factory/specs/behavioral-contracts/ss-14/` |
| BC-2.14.023 | Created (Group H: --modbus CLI flag enablement) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.023.md` |
| BC-2.14.024 | Created (Group H: --modbus-write-threshold CLI flag) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.024.md` |
| BC-2.14.025 | Created (Group H: StreamDispatcher port-502 Rule 5 classification) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.025.md` |
| Architecture Delta | Created | `.factory/phase-f2-spec-evolution/architecture-delta.md` |
| PRD Delta | Created | `.factory/phase-f2-spec-evolution/prd-delta.md` |
| VP-022 | Designed (to be authored by formal-verifier in parallel) | `.factory/specs/verification-properties/vp-022-modbus-parse-safety.md` |
| ADR-005 | Created (binary ICS protocol integration decision) | `.factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` |

**New MITRE ATT&CK for ICS techniques (6 total):**
- T0855 — Unauthorized Command Message (IcsImpairProcessControl)
- T0836 — Modify Parameter (IcsImpairProcessControl)
- T0814 — Denial of Service (IcsInhibitResponseFunction)
- T0806 — Brute Force I/O (IcsImpairProcessControl)
- T0835 — Manipulate I/O Image (IcsImpairProcessControl)
- T0831 — Manipulation of Control (IcsImpairProcessControl)

**MITRE catalog size:** 15 → 20 seeded technique IDs
(`SEEDED_TECHNIQUE_ID_COUNT = 15 → 20`; `EMITTED_IDS` extended from 6 to 12).

**Key constants introduced:**
- `MAX_PENDING_TRANSACTIONS = 256` (per-flow pending table cap)
- `WRITE_RATE_WINDOW_SECS = 1` (burst detection window)
- `DEFAULT_MODBUS_WRITE_THRESHOLD = 10` (writes/second before T0806 fires)

**CLI surface changes:**
- `--modbus` flag added to `analyze` subcommand (boolean, default false)
- `--modbus-write-threshold N` flag added (u32, default 10; zero rejected)
- `--all` expansion updated to include `--modbus`
- `needs_reassembly` expression updated: `enable_http || enable_tls || enable_modbus`

**Dispatcher changes:**
- `DispatchTarget::Modbus` variant added (4th variant after Http, Tls, None)
- `StreamDispatcher.modbus: Option<ModbusAnalyzer>` field added
- `classify` Rule 5: port 502 → `DispatchTarget::Modbus` (after content rules 1-2 and TLS/HTTP port rules 3-4)
- `modbus_analyzer()` and `take_modbus_analyzer()` accessors added
- `on_data` and `on_flow_close` Modbus routing arms added
- VP-004 `classify_oracle` must be extended with Rule 5

**Spec debt resolved:**
- O-04 partially resolved: T0855 (previously catalogued-but-never-emitted) is now actively
  emitted by ModbusAnalyzer. Updated in PRD Section 1.5 Out of Scope note.

---

## [1.0] — 2026-05-20

### Initial specification (brownfield ingestion)

Initial PRD and BC set produced by brownfield ingestion of develop HEAD. 219 active BCs
across ss-01 through ss-13 (BC-2.01.001..BC-2.13.004). Includes: 218 ingestion-batch BCs,
6 retired (BC-ABS-004..009), 5 pass-4 additions (BC-2.11.020..024), 2 F2 pcap-timestamp
additions (BC-2.04.055, BC-2.09.007).
