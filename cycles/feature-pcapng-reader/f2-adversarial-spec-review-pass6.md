---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
phase: F2
pass: 6
reviewer: adversary (fresh context)
date: 2026-06-20
artifacts_reviewed:
  - ADR-009 rev 8
  - BC-2.01.009 v1.5
  - BC-2.01.010 v2.0
  - BC-2.01.011 v1.5
  - BC-2.01.012 v1.5
  - BC-2.01.013 v1.5
  - BC-2.01.014 v1.5
  - BC-2.01.015 v1.6
  - BC-2.01.016 v1.4
  - BC-2.01.017 v1.4
  - BC-2.01.018 v1.6
  - error-taxonomy v3.4
  - VP-025..031
  - HS-101..108
verdict: NOT CLEAN
finding_counts:
  critical: 0
  high: 4
  medium: 5
  low: 4
  total: 13
trajectory: "P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13 (count plateau; severity declining — criticals: 3/4/1/1/1/0)"
clean_pass_counter: "0/3"
decision: D-155
---

# F2 Adversarial Spec Review — Pass 6

## Verdict: NOT CLEAN

**CRITICAL: 0 (FIRST pass with zero criticals)**
**HIGH: 4 | MEDIUM: 5 | LOW: 4 | TOTAL: 13**

Trajectory: P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13

Count plateau continues (P4/P5/P6 all at 13). However, this is the **first pass with zero
critical findings**. Severity is declining: criticals went 3→4→1→1→1→0 across P1..P6. The
remaining findings concentrate in HIGH (error-code routing mis-specifications) and MEDIUM
(definition conflicts, dead-extraction, unspecified edge cases).

Remediation round-6 is required before clean-pass counter can advance.

---

## HIGH Findings

### F-H1 [HIGH] — BC-2.01.017 PC1 body-decode context strings contradict Decision 20 (un-propagated from pass-4/5 dispatches)

**Location:** BC-2.01.017 v1.4, PC1 (context strings mapped to error codes).

**Finding:** BC-2.01.017 PC1 maps EPB body-decode and SPB body-decode context strings to
**E-INP-010**, contradicting ADR-009 rev-8 Decision 20. Decision 20 established the uniform
body-decode-truncation rule: crate-framed-but-body-too-short for all block types (EPB
body<20, SPB body<4) → **E-INP-008**, not E-INP-010. E-INP-010 is reserved for crate-level
framing failures (btl<12 / misaligned / EOF), which are pre-body events.

**Root cause (process-gap):** BC-2.01.017 was OMITTED from the pass-4 and pass-5 dispatch
checklists. It is the cross-cutting parent BC for pcapng block-level error codes (Lesson 7,
DF-ERROR-CODE-PARENT-BC-SWEEP-001). The pass-4 remediation (D-150) that established
Decision 20 and the pass-5 remediation (D-153) that reclassified padding-overrun both
required sweeping BC-2.01.017, but this BC received no version bump in either burst. This is
a direct recurrence of the C-4 defect pattern from pass-2 (D-144).

**Required fix:** Update BC-2.01.017 v1.4 → v1.5: revise PC1 to map EPB body-decode and SPB
body-decode context strings to **E-INP-008** (not E-INP-010). E-INP-010 context strings in
PC1 must be restricted to crate-framing failures only. E-INP-010 entry for "EPB
interface_id OOB" remains valid (it is not a body-decode failure — it is a field-validation
failure that may appropriately route through the crate framing context).

**Clarification:** E-INP-010 entry for crate framing + EPB interface_id OOB retains its
mapping. Only body-decode context strings require reclassification.

---

### F-H2 [HIGH] — `block_body_available` defined two ways: BC-2.01.013 "btl-16" vs VP-031 "body.len()" — off by 4

**Location:** BC-2.01.013 v1.5 prose; VP-031.

**Finding:** `block_body_available` is defined inconsistently:
- BC-2.01.013 prose (and the SPB formula `min(original_len, block_body_available)`): uses the
  definition `btl - 16` (SPB overhead = 16: 8-byte block header + 4-byte original_len +
  4-byte block trailer).
- VP-031 states `body.len()` as the effective bound (equivalently, as a note).

On the **raw-block path**, `RawBlock.body` is the block body as delivered by the crate after
stripping the 8-byte block header and 4-byte block trailer — so `body.len() = btl - 12`.
The SPB body includes the 4-byte `original_len` field at offset 0. Therefore:

- `body.len()` = btl - 12 (header + trailer stripped)
- The data region (bytes after `original_len`) = `body.len() - 4` = btl - 16

These are **not equivalent**. The formula `min(original_len, block_body_available)` depends
on which definition applies:
- If `block_body_available = btl - 16` (data bytes only): correct for computing `captured_len`
- If `block_body_available = body.len()` (= btl - 12, includes original_len field):
  off by 4 — `captured_len` would over-read by 4 bytes into `original_len`

VP-031 calling it `body.len()` is incorrect. The false "equivalently body.len()" claim in VP-031
Notes creates a 4-byte off-by-one that would cause the implementation to slice 4 bytes into
the `original_len` field.

**Required fix:**
1. Define ONE canonical symbol: `block_body_available = body.len() - 4` (data bytes after
   `original_len` is consumed).
2. Remove all "equivalently body.len()" prose from VP-031 and BC-2.01.013.
3. VP-031 proptest bound should use `min(original_len, body.len() - 4)` not `min(original_len, body.len())`.

---

### F-H3 [HIGH] — HS-107 Case B asserts data.len()==100 but VP-031's formula yields 104 (same root as F-H2)

**Location:** HS-107 v1.4, Case B; VP-031.

**Finding:** HS-107 Case B scenario: `btl=116, original_len=200, snaplen=100`. Case B asserts
`captured_len = min(200, 100) = 100` and `data.len() == 100` (using the two-way min after
snaplen was dropped per Decision 9 amend). However:

- `body.len()` on raw-block path = btl - 12 = 116 - 12 = 104
- `block_body_available` (data bytes after original_len) = 104 - 4 = 100
- `min(original_len=200, block_body_available=100) = 100` → `captured_len = 100` ✓

If VP-031 uses `body.len()` as the bound (the ambiguous form):
- `min(200, 104) = 104` → `captured_len = 104` — **disagrees with HS-107 by 4 bytes**

HS-107 Case B and VP-031 disagree depending on which definition of `block_body_available` is
applied. This is the same root defect as F-H2. The holdout scenario asserts the correct
value (100) but VP-031's "body.len()" notation would implement the wrong value (104).

**Required fix:** Aligns with F-H2 fix — once `block_body_available = body.len() - 4` is
canonical, HS-107 Case B becomes self-consistent. No separate fix needed beyond F-H2, but
HS-107 Case B rationale should be annotated to show `body.len()=104; data_bytes=104-4=100`.

---

### F-H4 [HIGH] — E-INP-009 vs E-INP-010 discriminant for interface_id errors has no VP or HS pinning the exact error code returned

**Location:** ADR-009 rev-8 HS-104 description; VP-027; error-taxonomy E-INP-009/010.

**Finding:** The spec distinguishes:
- E-INP-009: EPB `interface_id` references an **empty interface table** (no IDBs parsed yet)
- E-INP-010: EPB `interface_id` is **out-of-bounds** in a non-empty table

ADR-009 §HS-104 description uses the ambiguous notation "(→ E-INP-009 / E-INP-010)" with
a forward-slash, not specifying which case maps to which code. VP-027 proves no-panic + bounds
enforcement, but does NOT assert the discriminant returned (E-INP-009 vs E-INP-010 are both
valid returns within VP-027's scope as written). No HS-104 case is labeled with an explicit
expected error code.

An implementer reading only ADR-009 and VP-027 cannot determine which error code to return for
the empty-table case. An adversary re-reading this pass would count this as a novel gap
(E-INP-009 orphan pattern recurrence — similar to H-3 in pass-3 which orphaned E-INP-001).

**Required fix:**
1. Add explicit HS-104 sub-cases: "Case X — EPB arrives with empty interface table
   (no IDB seen) → **E-INP-009**" and "Case Y — EPB interface_id=N where N >= table.len() > 0 →
   **E-INP-010**".
2. Extend VP-027 to assert the discriminant (not just no-panic): proptest must distinguish
   empty-table input → E-INP-009 and OOB-non-empty input → E-INP-010.
3. Remove the ADR-009 slash notation "(→ E-INP-009 / E-INP-010)"; replace with two explicit
   case descriptions.

---

## MEDIUM Findings

### F-M1 [MEDIUM] — HS-107 Case E rationale says "below 12-byte minimum" but btl=14 >= 12; real cause is 4-byte alignment violation

**Location:** HS-107 v1.4, Case E.

**Finding:** Case E uses `btl=14`. The stated rationale is "block_total_length below 12-byte
minimum." However, 14 >= 12 (the 12-byte minimum is satisfied). The actual reason btl=14 is
invalid is **4-byte alignment**: pcapng requires block_total_length to be a multiple of 4.
14 % 4 = 2, so btl=14 violates the alignment rule → E-INP-010 (misaligned framing). The
12-byte minimum is not the governing constraint here.

Note: BC-2.01.013 EC-005 uses `btl=8` as the misalignment example (8 < 12, so it is invalid
on BOTH the minimum-length AND alignment axes). HS-107 Case E uses a value that is minimum-
valid but alignment-invalid, making the rationale doubly important to state correctly.

**Required fix (two options):**
- Option A: Correct Case E rationale to: "btl=14 satisfies >=12 minimum but violates 4-byte
  alignment (14 % 4 = 2) → E-INP-010 (misaligned)."
- Option B: Change Case E to btl=8 (matching BC-2.01.013 EC-005) so rationale "below 12"
  is accurate.

---

### F-M2 [MEDIUM, process-gap] — BOM LE/BE on-disk byte ordering restated in prose 4+ times; shared canonical table not referenced by all sites

**Location:** BC-2.01.010 AC-001, PC1; error-taxonomy; HS-103 Case A/C; ADR-009 §BOM.

**Finding:** After three correction rounds (D-143 BOM sweep, D-148, D-151), the BOM byte
ordering is now correct in all locations, but the canonical statement appears in **4+ separate
prose sites** with no "see canonical table" cross-reference. Each site uses slightly different
phrasing ("on-disk bytes `1A 2B 3C 4D`"; "big-endian section header BOM: 1A 2B 3C 4D";
"bytes[0..4] == [0x1A, 0x2B, 0x3C, 0x4D]"). A future correction round would need to update
all sites again. Lesson 5 (DF-WIRE-VALUE-BYTE-SEQUENCE-001) was filed but the remedy (a
single shared canonical table) was not implemented — the prose was corrected in place
repeatedly.

This is categorized MEDIUM because the current content is correct; the risk is future
drift recurrence and editorial overhead.

**Required fix:** Designate ONE site as the canonical BOM table (e.g., BC-2.01.010 §BOM-
Canonical or ADR-009 §BOM). All other sites replace their BOM byte listings with a
cross-reference: "See canonical BOM table: BC-2.01.010 §BOM-Canonical." This closes the
structural anti-pattern that caused the 4-document error chain (D-143).

---

### F-M3 [MEDIUM] — snaplen extracted and stored in InterfaceInfo but nothing consumes it post-Decision-9-amend; dead extraction anti-pattern

**Location:** BC-2.01.011 v1.5, PC4 / AC-003 ("for SPB use"); BC-2.01.013 v1.5.

**Finding:** BC-2.01.011 PC4 specifies that `snaplen` is extracted from IDB bytes 8–11 and
stored in `InterfaceInfo`. The AC-003 "usage" annotation says "for SPB use" (i.e., the three-
way SPB captured_len formula). However, ADR-009 rev-8 Decision 9 amend explicitly drops
`snaplen` from the SPB formula — `captured_len = min(original_len, block_body_available)`.
After this amendment, **no downstream path in any BC consumes `snaplen`** from `InterfaceInfo`.

This is the same anti-pattern that Decision 21 condemned for `if_tsoffset` (extracted but
never applied → silent wrong-timestamp risk). For `snaplen` the risk is different: the
extraction is harmless functionally, but: (a) the "for SPB use" annotation in AC-003 is now
false; (b) the `InterfaceInfo.snaplen` field is dead weight, and (c) an implementer reading
AC-003 may re-introduce snaplen-based truncation, reversing Decision 9.

**Required fix (two options):**
- Option A: Drop `snaplen` extraction from BC-2.01.011 PC4 and remove `InterfaceInfo.snaplen`
  field. Remove "for SPB use" from AC-003.
- Option B: Retain extraction (for observability/logging only) but replace "for SPB use" with
  explicit note: "snaplen is recorded for diagnostic purposes only; it is NOT used in
  captured_len computation per Decision 9 (SPB formula uses only original_len and
  block_body_available). Implementers MUST NOT apply snaplen-based truncation."

---

### F-M4 [MEDIUM] — SHB-only file (no IDB, no packets, no skipped blocks) zero-packet disposition unspecified

**Location:** BC-2.01.009; BC-2.01.010; HS-108.

**Finding:** The spec now covers:
- IDB-only pcapng → Ok + zero-packet notice (HS-108 Case a)
- OPB-only pcapng → Ok + OPB notice (HS-108 Cases d/e)
- EPB-before-IDB → Err E-INP-009 (HS-108 Case c)

But **SHB-only** (a pcapng file containing only a Section Header Block with no IDB, no packet
blocks, no skip blocks) is unspecified. This is a valid pcapng file per RFC 9293. On the
wirerust path, the block-walk loop exits with zero items and zero skips. It is unclear whether
wirerust should: (a) return Ok(zero packets) + a notice (if so, what message?), or (b) return
an error (no IDB → EPB would fail, but no EPB was seen; SHB is not itself an error).

No BC-2.01.009 edge case covers this. No HS-108 case covers it.

**Required fix:** Add BC-2.01.009 edge case: "SHB-only file (no IDB, no packet blocks, no
skipped blocks) → Ok(0 packets) + zero-packet notice per Decision 19 canonical format." Add
HS-108 Case f (or extend Case a): SHB-only file, btl exactly the SHB minimum, no subsequent
blocks → Ok + notice. Alternatively, decide SHB-only → Err and document the rationale.

---

### F-M5 [MEDIUM] — if_tsresol option code 9 with option-length != 1: behavior on raw path unspecified; crate enforces length==1 but raw-path check only verifies length<=remaining

**Location:** BC-2.01.011 v1.5, AC-005 (IDB options-TLV walk); error-taxonomy E-INP-008.

**Finding:** BC-2.01.011 AC-005 specifies that the IDB options-TLV walk validates
`option_length <= remaining_bytes` (no-overread guard). For option code 9 (`if_tsresol`),
the pcapng spec requires `option_length == 1` (exactly one byte for the `if_tsresol` u8
value). A malformed IDB with `if_tsresol` option code but `option_length = 2` (or any value
!= 1) is:
- Invalid per the pcapng specification
- NOT caught by the `length <= remaining` guard (if 2 bytes remain, the check passes)
- Not explicitly routed to any error code in BC-2.01.011 or error-taxonomy

On the crate path, `pcap_file` enforces `length==1` for `if_tsresol` and returns an error.
On the raw path, wirerust parses IDB options manually and AC-005 does not specify what to do
with a non-1-length `if_tsresol` option — it falls through as an unknown option, silently
picking up a wrong `if_tsresol` value or using a default.

**Required fix:** BC-2.01.011 AC-005 (or a new AC-006): "For option code 9 (`if_tsresol`),
if `option_length != 1`, return **E-INP-008** (malformed option: if_tsresol must be exactly 1
byte)." Add to error-taxonomy E-INP-008 trigger list: "IDB if_tsresol option with
option_length != 1."

---

## LOW Findings

### F-L1 [LOW] — VP-025 Kani harness scope note says "if_tsresol=6 path" but pass-5 extended to cover u32::MAX saturation; scope note not updated

**Location:** VP-025, scope/harness note.

**Finding:** VP-025 Kani scope note (added in pass-5 M-3 fix, D-153) references the
`if_tsresol=6` path for µs fast-path saturation. The note does not reflect that the harness
must also cover the general formula branches (base-2 and base-10) for totality under
DF-CANONICAL-FRAME-HOLDOUT-001. The pass-5 fix extended the saturation vector but the scope
note reads as if only if_tsresol=6 was targeted. Minor annotation drift; no functional impact
on VP-025 itself.

**Required fix:** VP-025 scope note: add "General formula branches (base-2 checked_shl;
base-10 saturating_mul) verified for totality over all u8 inputs in the same harness."

---

### F-L2 [LOW] — VP-INDEX count coherence vs actual VPs unverified this pass

**Finding:** VP-INDEX v2.7 states `total_vps: 31` (VP-001..031). This pass did not
independently verify the count against the VP-INDEX table rows. Recommend sweeping before F3
entry per the count-propagation sweep discipline (S-7.02).

**Action:** Count-propagation sweep before F3 entry gate. Not blocking remediation round-6.

---

### F-L3 [LOW, process-gap] — BC-2.01.017 omitted from pass-4 and pass-5 dispatch checklists (root of F-H1); same pattern as C-4 in pass-2

**Finding:** F-H1 documents the functional defect. This LOW process-gap observation records the
structural root cause: BC-2.01.017 is the cross-cutting parent BC for pcapng error codes but
was not included in either the pass-4 dispatch checklist (which introduced Decision 20) or the
pass-5 dispatch checklist (which reclassified padding-overrun). Both bursts (D-150 / D-153)
added E-INP-008 triggers but BC-2.01.017 received no sweep. This is the **third occurrence**
of the cross-cutting-parent-BC-omission pattern (C-4 pass-2, H-3 pass-3, and now F-H1 pass-6).

**Required action (beyond F-H1 fix):** Add mandatory checklist item to every error-code-change
burst: "Did BC-2.01.017 (cross-cutting error-code parent) receive a version bump in this burst?
If any E-INP-NNN code was added, reclassified, or removed, BC-2.01.017 MUST be updated." This
should be codified as an extension to Lesson 7 / DF-ERROR-CODE-PARENT-BC-SWEEP-001.

---

### F-L4 [LOW] — error-taxonomy BC-reference list for E-INP-010 should explicitly exclude body-decode cases now in E-INP-008 to prevent future re-drift

**Location:** error-taxonomy v3.4, E-INP-010 BC-references column.

**Finding:** After D-151 and D-153 reclassifications, E-INP-010 scope is now: "crate-level
framing failures (btl<12, misaligned, EOF)." The BC-references column still lists all BCs
that previously touched E-INP-010, including those whose body-decode paths were moved to
E-INP-008. The stale BC entries create reader confusion about E-INP-010's current scope.

**Required fix:** Audit E-INP-010 BC-references column; remove BCs whose only current
association is through body-decode paths (now E-INP-008). Retain only BCs with genuine
framing-failure routes to E-INP-010.

---

## Process-Gap Observations

| ID | Category | Observation |
|----|----------|-------------|
| PG-1 | process-gap | BC-2.01.017 omitted from dispatch checklists 2 passes running (F-H1 + F-L3). Codified in F-L3 — cross-cutting parent BC must appear in every error-code-change dispatch checklist. Escalate Lesson 7 to mandatory checklist item. |
| PG-2 | process-gap | BOM restated in prose 4+ times (F-M2) — single shared-table anchor not implemented despite Lesson 5 (DF-WIRE-VALUE-BYTE-SEQUENCE-001) being filed. The lesson captured the rule but not the structural fix (anchor-and-reference pattern). |

---

## Summary

| Pass | C | H | M | L | Total |
|------|---|---|---|---|-------|
| P1 | 3 | 6 | 7 | 3 | 23 (+ 4 obs) |
| P2 | 4 | 8 | 6 | 6 | 24 |
| P3 | 1 | 5 | 7 | 4 | 17 |
| P4 | 1 | 4 | 5 | 3 | 13 |
| P5 | 1 | 4 | 5 | 3 | 13 |
| **P6** | **0** | **4** | **5** | **4** | **13** |

Count plateau at 13 (P4/P5/P6). **Severity is declining** — this is the first pass with zero
criticals. Remaining findings are HIGH error-code routing gaps (F-H1/F-H4 are classic
cross-cutting-parent-omission + discriminant-unpinned patterns), MEDIUM definition conflicts
(F-H2/F-H3 share one root cause), and MEDIUM unspecified edge cases (F-M4/F-M5).

All 4 HIGH findings require spec amendments before pass-7 dispatch. F-H2 and F-H3 share a
root fix. F-L3 and F-M2 are process-gap lessons requiring policy extensions, not spec edits.

Clean-pass counter: 0/3. Remediation round-6 pending.
